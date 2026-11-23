use core::fmt::{self, Display};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use escher_domain::{FundamentalDomain, Orientation, ParameterSpec, ShapeParameters, parameters};
use escher_group::{
    ColouringId, ColouringSet, ColouringsOfIndex, IndexLimit, SearchBudget, colourings_of_index,
    free_classes_of_index,
};
use escher_render::{
    Colour, CosetFill, Fill, Image, Paint, RenderOptions, RenderStats, Renderer, Shading,
    WholeFace, WholeFaceFill, Wythoff, WythoffFill, encode_png,
};
use escher_signature::Signature;
use rayon::iter::{IndexedParallelIterator as _, ParallelIterator as _};
use rayon::slice::ParallelSliceMut as _;

use crate::args::{Colouring, RenderArgs, Setting, ShadingChoice};
use crate::count_of_colours;
use crate::diagnostic::{SignatureError, echoed};

const MAX_THREADS: usize = 256;

const DEFAULT_COLOURS: [[u8; 3]; 12] = [
    [0xe9, 0xc4, 0x6a],
    [0x2a, 0x9d, 0x8f],
    [0x91, 0x36, 0x71],
    [0x91, 0xab, 0x3b],
    [0x3b, 0x70, 0xc2],
    [0xff, 0xa2, 0x89],
    [0x77, 0x42, 0x96],
    [0xc2, 0x98, 0x10],
    [0x07, 0x7d, 0xa0],
    [0xfe, 0x9c, 0xb5],
    [0x51, 0x9a, 0x47],
    [0x4f, 0x51, 0xaa],
];

const IMAGE_PREFIX: &str = "image:";

pub(crate) const ARRANGEMENT_NEEDS_WYTHOFF: &str = "--arrangement needs --wythoff";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Statistics {
    Skip,
    Print,
}

impl Statistics {
    const fn from_flag(stats: bool) -> Self {
        if stats { Self::Print } else { Self::Skip }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Task<'a> {
    Render(&'a Signature),
    Masks(&'a Signature),
}

impl Task<'_> {
    pub(crate) fn failed(self) -> String {
        match self {
            Self::Render(signature) => format!("cannot render {signature}"),
            Self::Masks(signature) => format!("cannot render the masks of {signature}"),
        }
    }
}

pub(crate) fn render(args: &RenderArgs, stdout: &mut dyn Write) -> anyhow::Result<()> {
    let statistics = Statistics::from_flag(args.stats);
    let signature = parse_signature(&args.tiling.signature)?;
    let task = Task::Render(&signature);
    let border_colour = parse_colour(&args.border_colour)
        .ok_or_else(|| cannot_render(task, "--border-colour is not of the form #rgb or #rrggbb"))?;
    let background = parse_colour(&args.background)
        .ok_or_else(|| cannot_render(task, "--background is not of the form #rgb or #rrggbb"))?;
    let domain = build_domain(task, &signature, &args.tiling.set)?;
    let painting = match (args.tiling.wythoff, args.tiling.arrangement) {
        (None, Some(_)) => return Err(cannot_render(task, ARRANGEMENT_NEEDS_WYTHOFF)),
        (None, None) => {
            let (colouring, found_parity) =
                chosen_colouring(&domain, task, args.tiling.colouring, SearchBudget::DEFAULT)?;
            let items = fill_items(colouring, args.colours.as_deref())
                .map_err(|detail| cannot_render(task, detail))?;
            Painting::Tiles {
                colouring,
                found_parity,
                paints: read_paints(task, items)?,
            }
        }
        (Some(marks), None) => wythoff_painting(&domain, task, marks, args)?,
        (Some(marks), Some(id)) => whole_face_painting(&domain, task, marks, id, args)?,
    };
    let options_with =
        |fill| render_options(args, fill, border_colour, background).with_context(|| task.failed());
    options_with(Fill::One(border_colour))?;
    let fill = match painting {
        Painting::Tiles {
            colouring,
            found_parity,
            paints,
        } => fill(
            &domain,
            task,
            colouring,
            found_parity,
            paints,
            SearchBudget::DEFAULT,
        )?,
        Painting::Wythoff { wythoff, colours } => {
            Fill::Wythoff(WythoffFill::new(*wythoff, colours).with_context(|| task.failed())?)
        }
        Painting::WholeFace { face, id, paint } => {
            let arrangement = arrangement(&face, task, id, SearchBudget::DEFAULT)?;
            Fill::WholeFace(
                WholeFaceFill::new(*face, arrangement, vec![paint])
                    .with_context(|| task.failed())?,
            )
        }
    };
    let options = options_with(fill)?;
    let threads = args.threads.map_or_else(default_threads, usize::from);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|error| thread_pool_error(task, &error))?;
    let (width, height) = (options.width(), options.height());
    let renderer = Renderer::new(&domain, options).with_context(|| task.failed())?;
    let start = Instant::now();
    let (pixels, stats) = pool
        .install(|| render_bands(&renderer, width, height))
        .with_context(|| task.failed())?;
    let elapsed = start.elapsed();
    let png = encode_png(width, height, &pixels).with_context(|| task.failed())?;
    std::fs::write(&args.output, png).context("cannot write the output file")?;
    match statistics {
        Statistics::Skip => Ok(()),
        Statistics::Print => crate::write_stdout(
            stdout,
            &statistics_text(
                stats.samples(),
                stats.outside(),
                stats.folds(),
                stats.largest_fold(),
                stats.unconverged(),
                elapsed,
            ),
        ),
    }
}

pub(crate) fn parse_signature(text: &str) -> anyhow::Result<Signature> {
    text.parse()
        .map_err(|error| anyhow::Error::new(SignatureError::new(text, error)))
}

pub(crate) fn build_domain(
    task: Task<'_>,
    signature: &Signature,
    settings: &[Setting],
) -> anyhow::Result<FundamentalDomain> {
    let parameters = shape_parameters(settings).map_err(|detail| cannot_render(task, detail))?;
    FundamentalDomain::build_with(signature, &parameters).map_err(|error| {
        if matches!(
            error.kind(),
            escher_domain::ErrorKind::UnknownParameter
                | escher_domain::ErrorKind::ParameterOutOfRange
        ) && let Some(detail) = setting_text(signature, settings)
        {
            return cannot_render(task, detail);
        }
        if error.kind() == escher_domain::ErrorKind::UnsupportedSignature {
            anyhow::Error::new(error).context(format!(
                "{}: rendering this signature is not supported",
                task.failed()
            ))
        } else {
            anyhow::Error::new(error).context(task.failed())
        }
    })
}

fn render_options(
    args: &RenderArgs,
    fill: Fill,
    border_colour: Colour,
    background: Colour,
) -> escher_render::Result<RenderOptions> {
    let [yaw, pitch, roll] = args.rotation;
    RenderOptions::builder(fill)
        .size(args.tiling.size.width, args.tiling.size.height)
        .samples(args.samples)
        .border_width(
            args.border_width
                .unwrap_or_else(|| args.border_unit.default_width()),
        )
        .border_unit(args.border_unit.unit())
        .border_colour(border_colour)
        .background(background)
        .max_folds(args.max_folds)
        .rotation(yaw.to_radians(), pitch.to_radians(), roll.to_radians())
        .light(args.light.into())
        .shading(match args.shading {
            ShadingChoice::Phong => Shading::Phong,
            ShadingChoice::Flat => Shading::Flat,
        })
        .model(args.model.model())
        .build()
}

fn shape_parameters(settings: &[Setting]) -> Result<ShapeParameters, String> {
    let mut parameters = ShapeParameters::new();
    let mut first_positions: HashMap<&str, usize> = HashMap::new();
    for (position, setting) in settings.iter().enumerate() {
        if let Some(earlier) = first_positions.insert(&setting.name, position) {
            return Err(format!(
                "setting {} of --set names the same parameter as setting {}",
                position + 1,
                earlier + 1
            ));
        }
        parameters.set(setting.name.clone(), setting.value);
    }
    Ok(parameters)
}

fn render_bands(
    renderer: &Renderer,
    width: u32,
    height: u32,
) -> escher_render::Result<(Vec<u8>, RenderStats)> {
    let row_bytes = 3 * usize_from(width);
    let mut pixels = vec![0; row_bytes * usize_from(height)];
    let stats = pixels
        .par_chunks_mut(row_bytes)
        .zip(0..height)
        .map(|(band, row)| renderer.render_rows(row, band))
        .try_reduce(RenderStats::default, |first, second| {
            Ok(first.merged(second))
        })?;
    Ok((pixels, stats))
}

#[expect(
    clippy::expect_used,
    reason = "the renderer limits the width and the height to 16384, which fits every usize of a supported platform"
)]
fn usize_from(value: u32) -> usize {
    usize::try_from(value).expect("internal error: an image side fits a usize")
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map_or(1, NonZeroUsize::get)
        .min(MAX_THREADS)
}

fn thread_pool_error(task: Task<'_>, error: &dyn Display) -> anyhow::Error {
    anyhow::anyhow!("{error}")
        .context("cannot start the thread pool")
        .context(task.failed())
}

fn setting_text(signature: &Signature, settings: &[Setting]) -> Option<String> {
    let specs = parameters(signature);
    let names: Vec<&str> = specs.iter().map(ParameterSpec::name).collect();
    if let Some(unknown) = settings
        .iter()
        .find(|setting| !names.contains(&setting.name.as_str()))
    {
        let name: String = unknown.name.chars().map(echoed).collect();
        let taken = match names.as_slice() {
            [] => "nor any other".to_owned(),
            [only] => format!("and its parameter is {only}"),
            [first, second] => format!("and its parameters are {first} and {second}"),
            [first @ .., last] => {
                format!("and its parameters are {}, and {last}", first.join(", "))
            }
        };
        return Some(format!(
            "the signature has no shape parameter {name}, {taken}"
        ));
    }
    settings.iter().find_map(|setting| {
        let spec = specs.iter().find(|spec| spec.name() == setting.name)?;
        let range = spec.minimum()..=spec.maximum();
        (!range.contains(&setting.value)).then(|| {
            format!(
                "the value {} of the parameter {} is outside its range of {} to {}",
                setting.value,
                spec.name(),
                spec.minimum(),
                spec.maximum()
            )
        })
    })
}

pub(crate) fn cannot_render(task: Task<'_>, detail: impl Display) -> anyhow::Error {
    anyhow::anyhow!("{}: {detail}", task.failed())
}

#[derive(Clone, Debug)]
pub(crate) enum Parity {
    Reversals,
    Cosets(escher_group::Colouring),
}

fn is_all_reversing(domain: &FundamentalDomain) -> bool {
    let sides = domain.sides();
    !sides.is_empty()
        && sides
            .iter()
            .all(|side| side.orientation() == Orientation::Reversing)
}

fn parity(
    domain: &FundamentalDomain,
    task: Task<'_>,
    budget: SearchBudget,
) -> anyhow::Result<Option<Parity>> {
    if is_all_reversing(domain) {
        return Ok(Some(Parity::Reversals));
    }
    Ok(parity_cosets(domain, task, budget)?.map(Parity::Cosets))
}

fn parity_cosets(
    domain: &FundamentalDomain,
    task: Task<'_>,
    budget: SearchBudget,
) -> anyhow::Result<Option<escher_group::Colouring>> {
    let found = search(domain, task, crate::COLOURING_SET, 2, budget)?;
    match found.list().first() {
        Some(colouring) => Ok(Some(colouring.clone())),
        None if found.is_complete() => Ok(None),
        None => Err(cannot_render(task, stopped_text("parity"))),
    }
}

fn missing_parity(task: Task<'_>) -> anyhow::Error {
    cannot_render(
        task,
        "the colouring parity does not exist, because the signature has no colouring with 2 colours",
    )
}

pub(crate) fn chosen_colouring(
    domain: &FundamentalDomain,
    task: Task<'_>,
    requested: Option<Colouring>,
    budget: SearchBudget,
) -> anyhow::Result<(Colouring, Option<Parity>)> {
    if let Some(colouring) = requested {
        return Ok((colouring, None));
    }
    Ok(match parity(domain, task, budget)? {
        Some(found) => (Colouring::Parity, Some(found)),
        None => (Colouring::One, None),
    })
}

#[derive(Debug)]
enum Painting {
    Tiles {
        colouring: Colouring,
        found_parity: Option<Parity>,
        paints: Paints,
    },
    Wythoff {
        wythoff: Box<Wythoff>,
        colours: Vec<Colour>,
    },
    WholeFace {
        face: Box<WholeFace>,
        id: ColouringId,
        paint: Paint,
    },
}

pub(crate) fn refuse_colouring(task: Task<'_>, colouring: Option<Colouring>) -> anyhow::Result<()> {
    match colouring {
        Some(_) => Err(cannot_render(
            task,
            "--wythoff cannot be combined with --colouring",
        )),
        None => Ok(()),
    }
}

fn wythoff_painting(
    domain: &FundamentalDomain,
    task: Task<'_>,
    marks: [bool; 3],
    args: &RenderArgs,
) -> anyhow::Result<Painting> {
    refuse_colouring(task, args.tiling.colouring)?;
    let wythoff = Box::new(Wythoff::new(domain, marks).with_context(|| task.failed())?);
    let colours = face_colours(wythoff.face_types(), args.colours.as_deref())
        .map_err(|detail| cannot_render(task, detail))?;
    Ok(Painting::Wythoff { wythoff, colours })
}

fn whole_face_painting(
    domain: &FundamentalDomain,
    task: Task<'_>,
    marks: [bool; 3],
    id: ColouringId,
    args: &RenderArgs,
) -> anyhow::Result<Painting> {
    refuse_colouring(task, args.tiling.colouring)?;
    let face = Box::new(whole_face(domain, marks).with_context(|| task.failed())?);
    let paint = match picture_item(id, args.colours.as_deref())
        .map_err(|detail| cannot_render(task, detail))?
    {
        Item::Colour(colour) => Paint::Solid(colour),
        Item::Image(path) => Paint::Image(open_image(task, 1, path)?),
    };
    Ok(Painting::WholeFace { face, id, paint })
}

pub(crate) fn whole_face(
    domain: &FundamentalDomain,
    marks: [bool; 3],
) -> escher_render::Result<WholeFace> {
    WholeFace::new(domain, Wythoff::new(domain, marks)?)
}

pub(crate) fn arrangements(
    face: &WholeFace,
    budget: SearchBudget,
) -> escher_group::Result<ColouringsOfIndex> {
    let index = IndexLimit::new(face.index())?;
    free_classes_of_index(face.presentation(), face.stabilizer(), index, budget)
}

pub(crate) fn arrangement(
    face: &WholeFace,
    task: Task<'_>,
    id: ColouringId,
    budget: SearchBudget,
) -> anyhow::Result<escher_group::Colouring> {
    let found = arrangements(face, budget).with_context(|| task.failed())?;
    found_arrangement(&found, id).map_err(|detail| cannot_render(task, detail))
}

fn found_arrangement(
    found: &ColouringsOfIndex,
    id: ColouringId,
) -> Result<escher_group::Colouring, String> {
    if let Some(arrangement) = found.find(id) {
        return Ok(arrangement.clone());
    }
    if !found.is_complete() {
        return Err(format!(
            "the search for the arrangement {id} stopped at a limit before it found it"
        ));
    }
    match found.list().len() {
        0 => Err("the tiling has no arrangement that puts one picture on every polygon".to_owned()),
        1 => Err(format!(
            "the arrangement {id} does not exist, because the tiling has 1 arrangement"
        )),
        count => Err(format!(
            "the arrangement {id} does not exist, because the tiling has {count} arrangements"
        )),
    }
}

#[derive(Debug)]
enum Paints {
    Solid(Vec<Colour>),
    Images(Vec<Paint>),
}

fn fill(
    domain: &FundamentalDomain,
    task: Task<'_>,
    colouring: Colouring,
    found_parity: Option<Parity>,
    paints: Paints,
    budget: SearchBudget,
) -> anyhow::Result<Fill> {
    let colours = match paints {
        Paints::Solid(colours) => colours,
        Paints::Images(paints) => {
            return coset_fill(domain, task, colouring, found_parity, paints, budget);
        }
    };
    let wrong_count = |colours: Vec<Colour>| {
        cannot_render(
            task,
            format!(
                "internal error: the colouring {colouring} was given {} colours",
                colours.len()
            ),
        )
    };
    let found = match colouring {
        Colouring::One => return one_colour(colours).map_err(wrong_count),
        Colouring::Id(id) if is_one_colour(id) => {
            return one_colour(colours).map_err(wrong_count);
        }
        Colouring::Id(id) => resolve(domain, task, id, budget)?,
        Colouring::Parity => {
            let found = match found_parity {
                Some(found) => found,
                None => parity(domain, task, budget)?.ok_or_else(|| missing_parity(task))?,
            };
            match found {
                Parity::Reversals => {
                    return <[Colour; 2]>::try_from(colours)
                        .map(Fill::Parity)
                        .map_err(wrong_count);
                }
                Parity::Cosets(colouring) => colouring,
            }
        }
    };
    cosets(
        domain,
        task,
        found,
        colours.into_iter().map(Paint::Solid).collect(),
    )
}

fn coset_fill(
    domain: &FundamentalDomain,
    task: Task<'_>,
    colouring: Colouring,
    found_parity: Option<Parity>,
    paints: Vec<Paint>,
    budget: SearchBudget,
) -> anyhow::Result<Fill> {
    let found = coset_colouring(domain, task, colouring, found_parity, budget)?;
    cosets(domain, task, found, paints)
}

fn cosets(
    domain: &FundamentalDomain,
    task: Task<'_>,
    colouring: escher_group::Colouring,
    paints: Vec<Paint>,
) -> anyhow::Result<Fill> {
    Ok(Fill::Cosets(
        CosetFill::new(domain, colouring, paints).with_context(|| task.failed())?,
    ))
}

pub(crate) fn coset_colouring(
    domain: &FundamentalDomain,
    task: Task<'_>,
    colouring: Colouring,
    found_parity: Option<Parity>,
    budget: SearchBudget,
) -> anyhow::Result<escher_group::Colouring> {
    match colouring {
        Colouring::One => trivial_colouring(domain, task, colouring, budget),
        Colouring::Id(id) if is_one_colour(id) => {
            trivial_colouring(domain, task, colouring, budget)
        }
        Colouring::Id(id) => resolve(domain, task, id, budget),
        Colouring::Parity => match found_parity {
            Some(Parity::Cosets(found)) => Ok(found),
            Some(Parity::Reversals) | None => {
                parity_cosets(domain, task, budget)?.ok_or_else(|| missing_parity(task))
            }
        },
    }
}

fn trivial_colouring(
    domain: &FundamentalDomain,
    task: Task<'_>,
    colouring: Colouring,
    budget: SearchBudget,
) -> anyhow::Result<escher_group::Colouring> {
    let found = search(domain, task, ColouringSet::All, 1, budget)?;
    match found.list().first() {
        Some(trivial) => Ok(trivial.clone()),
        None if found.is_complete() => Err(cannot_render(
            task,
            format!(
                "the colouring {colouring} does not exist, because the signature has no colouring with 1 colour"
            ),
        )),
        None => Err(cannot_render(task, stopped_text(colouring))),
    }
}

fn is_one_colour(id: ColouringId) -> bool {
    (id.colours(), id.ordinal()) == (1, 1)
}

fn one_colour(colours: Vec<Colour>) -> Result<Fill, Vec<Colour>> {
    <[Colour; 1]>::try_from(colours).map(|[colour]| Fill::One(colour))
}

fn search(
    domain: &FundamentalDomain,
    task: Task<'_>,
    set: ColouringSet,
    colours: u32,
    budget: SearchBudget,
) -> anyhow::Result<ColouringsOfIndex> {
    let presentation = domain.side_presentation().with_context(|| task.failed())?;
    let colours = IndexLimit::new(colours).with_context(|| task.failed())?;
    colourings_of_index(&presentation, set, colours, budget).with_context(|| task.failed())
}

fn resolve(
    domain: &FundamentalDomain,
    task: Task<'_>,
    id: ColouringId,
    budget: SearchBudget,
) -> anyhow::Result<escher_group::Colouring> {
    let found = search(domain, task, crate::COLOURING_SET, id.colours(), budget)?;
    found_colouring(&found, id).map_err(|detail| cannot_render(task, detail))
}

fn found_colouring(
    found: &ColouringsOfIndex,
    id: ColouringId,
) -> Result<escher_group::Colouring, String> {
    match found.find(id) {
        Some(colouring) => Ok(colouring.clone()),
        None if found.is_complete() => Err(missing_text(id, found.list().len())),
        None => Err(stopped_text(id)),
    }
}

fn stopped_text(colouring: impl Display) -> String {
    format!("the search for the colouring {colouring} stopped at a limit before it found it")
}

fn missing_text(id: ColouringId, count: usize) -> String {
    let colourings = match count {
        0 => "no colouring".to_owned(),
        1 => "1 colouring".to_owned(),
        _ => format!("{count} colourings"),
    };
    format!(
        "the colouring {id} does not exist, because the signature has {colourings} with {}",
        count_of_colours(u64::from(id.colours()))
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Item<'a> {
    Colour(Colour),
    Image(&'a str),
}

fn fill_items(colouring: Colouring, colours: Option<&str>) -> Result<Vec<Item<'_>>, String> {
    let needed = colouring.colours();
    let Some(text) = colours else {
        return default_colours(needed)
            .map(|colours| colours.into_iter().map(Item::Colour).collect())
            .ok_or_else(|| {
                format!(
                    "the colouring {colouring} needs {needed} colours, more than the {} default colours, so give {needed} colours with --colours",
                    DEFAULT_COLOURS.len()
                )
            });
    };
    counted_items(text, needed, format_args!("the colouring {colouring}"))?
        .iter()
        .enumerate()
        .map(|(position, item)| fill_item(position + 1, item))
        .collect()
}

fn face_colours(face_types: u32, colours: Option<&str>) -> Result<Vec<Colour>, String> {
    let Some(text) = colours else {
        return default_colours(face_types).ok_or_else(|| {
            format!("internal error: the Wythoff construction has {face_types} face types")
        });
    };
    counted_items(text, face_types, "the Wythoff construction")?
        .iter()
        .enumerate()
        .map(|(index, item)| {
            parse_colour(item).ok_or_else(|| {
                format!(
                    "colour {} of --colours is not of the form #rgb or #rrggbb",
                    index + 1
                )
            })
        })
        .collect()
}

fn picture_item(id: ColouringId, colours: Option<&str>) -> Result<Item<'_>, String> {
    let Some(text) = colours else {
        let [red, green, blue] = DEFAULT_COLOURS[0];
        return Ok(Item::Colour(Colour::from_srgb8(red, green, blue)));
    };
    fill_item(
        1,
        counted_items(text, 1, format_args!("the arrangement {id}"))?[0],
    )
}

fn default_colours(needed: u32) -> Option<Vec<Colour>> {
    let entries = DEFAULT_COLOURS.get(..usize::try_from(needed).ok()?)?;
    Some(
        entries
            .iter()
            .map(|&[red, green, blue]| Colour::from_srgb8(red, green, blue))
            .collect(),
    )
}

fn counted_items(text: &str, needed: u32, subject: impl Display) -> Result<Vec<&str>, String> {
    let items: Vec<&str> = text.split(',').collect();
    if u64::try_from(items.len()).ok() != Some(u64::from(needed)) {
        return Err(format!(
            "{subject} needs {}, and {} given",
            count_of_colours(u64::from(needed)),
            given(items.len())
        ));
    }
    Ok(items)
}

fn fill_item(position: usize, text: &str) -> Result<Item<'_>, String> {
    if let Some(path) = text.strip_prefix(IMAGE_PREFIX)
        && !path.is_empty()
    {
        return Ok(Item::Image(path));
    }
    parse_colour(text).map(Item::Colour).ok_or_else(|| {
        format!("colour {position} of --colours is not of the form #rgb, #rrggbb, or image:FILE")
    })
}

fn read_paints(task: Task<'_>, items: Vec<Item<'_>>) -> anyhow::Result<Paints> {
    let solid: Option<Vec<Colour>> = items
        .iter()
        .map(|item| match item {
            Item::Colour(colour) => Some(*colour),
            Item::Image(_) => None,
        })
        .collect();
    if let Some(colours) = solid {
        return Ok(Paints::Solid(colours));
    }
    let paints = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| match item {
            Item::Colour(colour) => Ok(Paint::Solid(colour)),
            Item::Image(path) => open_image(task, index + 1, path).map(Paint::Image),
        })
        .collect::<anyhow::Result<Vec<Paint>>>()?;
    Ok(Paints::Images(paints))
}

fn open_image(task: Task<'_>, position: usize, path: &str) -> anyhow::Result<Image> {
    let file = std::fs::File::open(path).with_context(|| {
        format!(
            "{}: the image of colour {position} of --colours cannot be opened",
            task.failed()
        )
    })?;
    decode_image(task, position, &mut std::io::BufReader::new(file))
}

fn decode_image(task: Task<'_>, position: usize, reader: &mut dyn Read) -> anyhow::Result<Image> {
    Image::decode_png(reader).map_err(|error| {
        anyhow::Error::new(ImageError::new(error)).context(format!(
            "{}: the image of colour {position} of --colours cannot be used",
            task.failed()
        ))
    })
}

#[derive(Debug)]
struct ImageError {
    error: escher_render::Error,
    cause: Option<PrintableText>,
}

impl ImageError {
    fn new(error: escher_render::Error) -> Self {
        let cause = core::error::Error::source(&error)
            .map(|source| PrintableText(source.to_string().chars().map(echoed).collect()));
        Self { error, cause }
    }
}

impl Display for ImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error, formatter)
    }
}

impl core::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        let cause: &PrintableText = self.cause.as_ref()?;
        Some(cause)
    }
}

#[derive(Debug)]
struct PrintableText(String);

impl Display for PrintableText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl core::error::Error for PrintableText {}

fn given(count: usize) -> String {
    if count == 1 {
        "1 was".to_owned()
    } else {
        format!("{count} were")
    }
}

fn parse_colour(text: &str) -> Option<Colour> {
    let digits = text
        .strip_prefix('#')?
        .bytes()
        .map(hex_digit)
        .collect::<Option<Vec<u8>>>()?;
    match *digits.as_slice() {
        [red, green, blue] => Some(Colour::from_srgb8(red * 17, green * 17, blue * 17)),
        [
            red_high,
            red_low,
            green_high,
            green_low,
            blue_high,
            blue_low,
        ] => Some(Colour::from_srgb8(
            red_high * 16 + red_low,
            green_high * 16 + green_low,
            blue_high * 16 + blue_low,
        )),
        _ => None,
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn statistics_text(
    samples: u64,
    outside: u64,
    folds: u64,
    largest_fold: u32,
    unconverged: u64,
    elapsed: Duration,
) -> String {
    let inside = u128::from(samples.saturating_sub(outside));
    let mean = (u128::from(folds) * 100).checked_div(inside).unwrap_or(0);
    format!(
        "Samples: {samples}\n\
         Mean folds: {}.{:02}\n\
         Maximum folds: {largest_fold}\n\
         Unconverged samples: {unconverged}\n\
         Time: {:.3} s\n",
        mean / 100,
        mean % 100,
        elapsed.as_secs_f64()
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use escher_domain::FundamentalDomain;
    use escher_group::SearchBudget;
    use escher_render::{Colour, Fill, Paint, RenderOptions, Renderer};
    use escher_signature::Signature;

    use super::{
        DEFAULT_COLOURS, Item, Paints, Parity, Task, arrangement, arrangements, chosen_colouring,
        coset_colouring, coset_fill, decode_image, face_colours, fill, fill_item, fill_items,
        found_arrangement, missing_text, parse_colour, picture_item, resolve, setting_text,
        shape_parameters, statistics_text, thread_pool_error, whole_face,
    };
    use crate::Status;
    use crate::args::{Colouring, Setting};
    use crate::colouring_id;
    use crate::diagnostic::report;

    #[derive(Debug)]
    struct PoolBuildError(std::io::Error);

    impl core::fmt::Display for PoolBuildError {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Display::fmt(&self.0, formatter)
        }
    }

    impl core::error::Error for PoolBuildError {
        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn thread_pool_error_names_the_pool_and_gives_the_cause_once() {
        let signature = "*732".parse().expect("the signature is valid");
        let error = PoolBuildError(std::io::Error::other("no thread can be started"));
        let mut stderr = Vec::new();
        let status = report(
            &thread_pool_error(Task::Render(&signature), &error),
            &mut stderr,
        );
        assert_eq!(status, Status::Failure);
        assert_eq!(
            String::from_utf8(stderr).expect("standard error is UTF-8"),
            "error: cannot render *732\n  caused by: cannot start the thread pool\n  caused by: no thread can be started\n"
        );
    }

    #[test]
    fn parse_colour_reads_six_digits_in_either_case() {
        let expected = Some([0xe9, 0xc4, 0x6a]);
        assert_eq!(parse_colour("#e9c46a").map(Colour::to_srgb8), expected);
        assert_eq!(parse_colour("#E9C46A").map(Colour::to_srgb8), expected);
    }

    #[test]
    fn parse_colour_reads_three_digits_as_doubled_digits() {
        assert_eq!(
            parse_colour("#e9c").map(Colour::to_srgb8),
            Some([0xee, 0x99, 0xcc])
        );
    }

    #[test]
    fn parse_colour_rejects_other_forms() {
        for text in [
            "e9c46a",
            "#",
            "#e9c4",
            "#e9c46a0",
            "#ggg",
            "",
            " #e9c46a",
            "#e9c46a ",
            "#e9c4\u{0666}a",
        ] {
            assert_eq!(parse_colour(text), None, "{text:?}");
        }
    }

    fn fill_colours(colouring: Colouring, text: Option<&str>) -> Result<Vec<Colour>, String> {
        fill_items(colouring, text).map(|items| {
            items
                .into_iter()
                .map(|item| match item {
                    Item::Colour(colour) => colour,
                    Item::Image(path) => panic!("the item names the image {path}"),
                })
                .collect()
        })
    }

    #[test]
    fn fill_colours_reports_the_position_of_the_first_bad_colour() {
        assert_eq!(
            fill_colours(Colouring::Parity, Some("red,#e9c")),
            Err("colour 1 of --colours is not of the form #rgb, #rrggbb, or image:FILE".to_owned())
        );
        assert_eq!(
            fill_colours(Colouring::Parity, Some("#e9c,")),
            Err("colour 2 of --colours is not of the form #rgb, #rrggbb, or image:FILE".to_owned())
        );
    }

    #[test]
    fn fill_colours_names_one_colour_in_the_singular() {
        assert_eq!(
            fill_colours(Colouring::One, Some("#e9c,#2a9")),
            Err("the colouring one needs 1 colour, and 2 were given".to_owned())
        );
        assert_eq!(
            fill_colours(Colouring::Parity, Some("#e9c")),
            Err("the colouring parity needs 2 colours, and 1 was given".to_owned())
        );
    }

    #[test]
    fn face_colours_count_the_face_types_in_words() {
        assert_eq!(
            face_colours(1, Some("#e9c,#2a9")),
            Err("the Wythoff construction needs 1 colour, and 2 were given".to_owned())
        );
        assert_eq!(
            face_colours(2, Some("#e9c")),
            Err("the Wythoff construction needs 2 colours, and 1 was given".to_owned())
        );
    }

    #[test]
    fn face_colours_read_the_colours_in_order_and_no_image_item() {
        assert_eq!(
            face_colours(2, Some("#000,#fff")),
            Ok(vec![Colour::BLACK, Colour::WHITE])
        );
        assert_eq!(
            face_colours(2, Some("#000,image:fill.png")),
            Err("colour 2 of --colours is not of the form #rgb or #rrggbb".to_owned())
        );
    }

    #[test]
    fn fill_colours_names_the_id_of_a_wrong_count() {
        assert_eq!(
            fill_colours(id("3a"), Some("#e9c,#2a9")),
            Err("the colouring 3a needs 3 colours, and 2 were given".to_owned())
        );
    }

    #[test]
    fn fill_colours_reads_the_colours_in_order() {
        let colours = fill_colours(id("3a"), Some("#123,#fed,#000000")).map(|colours| {
            colours
                .into_iter()
                .map(Colour::to_srgb8)
                .collect::<Vec<_>>()
        });
        assert_eq!(
            colours,
            Ok(vec![[0x11, 0x22, 0x33], [0xff, 0xee, 0xdd], [0, 0, 0]])
        );
    }

    fn id(text: &str) -> Colouring {
        Colouring::Id(colouring_id(text))
    }

    fn srgb8(colours: Result<Vec<Colour>, String>) -> Result<Vec<[u8; 3]>, String> {
        colours.map(|colours| colours.into_iter().map(Colour::to_srgb8).collect())
    }

    const DESIGN_COLOURS: [&str; 12] = [
        "#e9c46a", "#2a9d8f", "#913671", "#91ab3b", "#3b70c2", "#ffa289", "#774296", "#c29810",
        "#077da0", "#fe9cb5", "#519a47", "#4f51aa",
    ];

    fn design_colours(count: usize) -> Vec<[u8; 3]> {
        DESIGN_COLOURS[..count]
            .iter()
            .map(|text| {
                parse_colour(text)
                    .map(Colour::to_srgb8)
                    .expect("the colour is valid")
            })
            .collect()
    }

    #[test]
    fn default_colours_are_the_twelve_colours_of_the_palette() {
        assert_eq!(DEFAULT_COLOURS.to_vec(), design_colours(12));
    }

    #[test]
    fn fill_colours_takes_the_first_default_colours() {
        for count in 1..=12_usize {
            assert_eq!(
                srgb8(fill_colours(id(&format!("{count}a")), None)),
                Ok(design_colours(count)),
                "{count}"
            );
        }
        assert_eq!(
            srgb8(fill_colours(Colouring::One, None)),
            Ok(vec![[0xe9, 0xc4, 0x6a]])
        );
        assert_eq!(
            srgb8(fill_colours(Colouring::Parity, None)),
            Ok(vec![[0xe9, 0xc4, 0x6a], [0x2a, 0x9d, 0x8f]])
        );
    }

    #[test]
    fn fill_colours_asks_for_the_colours_of_more_than_twelve() {
        assert_eq!(
            fill_colours(id("13b"), None),
            Err("the colouring 13b needs 13 colours, more than the 12 default colours, so give 13 colours with --colours".to_owned())
        );
    }

    #[test]
    fn missing_text_counts_the_colourings_in_words() {
        assert_eq!(
            missing_text(colouring_id("3d"), 3),
            "the colouring 3d does not exist, because the signature has 3 colourings with 3 colours"
        );
        assert_eq!(
            missing_text(colouring_id("2b"), 1),
            "the colouring 2b does not exist, because the signature has 1 colouring with 2 colours"
        );
        assert_eq!(
            missing_text(colouring_id("1b"), 0),
            "the colouring 1b does not exist, because the signature has no colouring with 1 colour"
        );
    }

    fn domain(text: &str) -> FundamentalDomain {
        let signature: Signature = text.parse().expect("the signature is valid");
        FundamentalDomain::build(&signature).expect("the signature has a domain")
    }

    fn resolved(text: &str, colouring: &str, steps: u64) -> Result<String, String> {
        let signature: Signature = text.parse().expect("the signature is valid");
        let budget = SearchBudget::new(steps).expect("the budget is valid");
        resolve(
            &domain(text),
            Task::Render(&signature),
            colouring_id(colouring),
            budget,
        )
        .map(|found| found.id().to_string())
        .map_err(|error| error.to_string())
    }

    #[test]
    fn resolve_finds_an_id_and_names_one_that_does_not_exist() {
        assert_eq!(resolved("22", "2a", 10_000), Ok("2a".to_owned()));
        assert_eq!(
            resolved("22", "2b", 10_000),
            Err("cannot render 22: the colouring 2b does not exist, because the signature has 1 colouring with 2 colours".to_owned())
        );
    }

    #[test]
    fn resolve_claims_nothing_when_the_search_stops_at_a_limit() {
        assert_eq!(
            resolved("*632", "2a", 100),
            Err("cannot render *632: the search for the colouring 2a stopped at a limit before it found it".to_owned())
        );
    }

    fn default_colouring(text: &str, steps: u64) -> Result<String, String> {
        let signature: Signature = text.parse().expect("the signature is valid");
        let budget = SearchBudget::new(steps).expect("the budget is valid");
        match chosen_colouring(&domain(text), Task::Render(&signature), None, budget) {
            Ok((colouring, None)) => Ok(colouring.to_string()),
            Ok((colouring, Some(Parity::Reversals))) => Ok(format!("{colouring} by reversals")),
            Ok((colouring, Some(Parity::Cosets(found)))) => {
                Ok(format!("{colouring} {}", found.id()))
            }
            Err(error) => Err(error.to_string()),
        }
    }

    #[test]
    fn default_colouring_is_parity_without_a_search_on_an_all_reversing_domain() {
        for signature in ["*632", "*2222", "*532", "*", "x"] {
            assert_eq!(
                default_colouring(signature, 1),
                Ok("parity by reversals".to_owned()),
                "{signature}"
            );
        }
    }

    #[test]
    fn default_colouring_is_the_valid_colouring_with_two_colours_when_one_exists() {
        for signature in ["442", "2222", "o", "4*2", "22*"] {
            assert_eq!(
                default_colouring(signature, 10_000_000),
                Ok("parity 2a".to_owned()),
                "{signature}"
            );
        }
    }

    #[test]
    fn default_colouring_is_one_when_no_valid_colouring_with_two_colours_exists() {
        for signature in ["1", "632", "3*3", "732", "532"] {
            assert_eq!(
                default_colouring(signature, 10_000_000),
                Ok("one".to_owned()),
                "{signature}"
            );
        }
    }

    #[test]
    fn default_colouring_fails_when_its_search_stops_at_a_limit() {
        assert_eq!(
            default_colouring("442", 1),
            Err("cannot render 442: the search for the colouring parity stopped at a limit before it found it".to_owned())
        );
    }

    fn parity_fill(text: &str, steps: u64) -> Result<(), String> {
        let signature: Signature = text.parse().expect("the signature is valid");
        let budget = SearchBudget::new(steps).expect("the budget is valid");
        let colours = fill_colours(Colouring::Parity, None).expect("parity has default colours");
        fill(
            &domain(text),
            Task::Render(&signature),
            Colouring::Parity,
            None,
            Paints::Solid(colours),
            budget,
        )
        .map(drop)
        .map_err(|error| error.to_string())
    }

    #[test]
    fn the_default_parity_is_drawn_without_a_second_search() {
        let text = "442";
        let signature: Signature = text.parse().expect("the signature is valid");
        let domain = domain(text);
        let (colouring, found) = chosen_colouring(
            &domain,
            Task::Render(&signature),
            None,
            SearchBudget::DEFAULT,
        )
        .expect("the search of 442 completes");
        let colours = fill_colours(colouring, None).expect("parity has default colours");
        let budget = SearchBudget::new(1).expect("the budget is valid");
        assert!(matches!(found, Some(Parity::Cosets(_))));
        let task = Task::Render(&signature);
        let paints = Paints::Solid(colours);
        assert!(fill(&domain, task, colouring, found, paints, budget).is_ok());
    }

    #[test]
    fn parity_is_drawn_where_a_valid_colouring_with_two_colours_exists() {
        assert_eq!(parity_fill("*632", 1), Ok(()));
        assert_eq!(parity_fill("442", 10_000_000), Ok(()));
    }

    #[test]
    fn parity_fails_where_no_valid_colouring_with_two_colours_exists() {
        assert_eq!(
            parity_fill("632", 10_000_000),
            Err("cannot render 632: the colouring parity does not exist, because the signature has no colouring with 2 colours".to_owned())
        );
    }

    #[test]
    fn parity_claims_nothing_when_its_search_stops_at_a_limit() {
        assert_eq!(
            parity_fill("442", 1),
            Err("cannot render 442: the search for the colouring parity stopped at a limit before it found it".to_owned())
        );
    }

    #[test]
    fn coset_colouring_of_parity_claims_nothing_when_its_search_stops_at_a_limit() {
        let text = "442";
        let signature: Signature = text.parse().expect("the signature is valid");
        let budget = SearchBudget::new(1).expect("the budget is valid");
        let found = coset_colouring(
            &domain(text),
            Task::Render(&signature),
            Colouring::Parity,
            None,
            budget,
        );
        assert_eq!(
            found.map(|colouring| colouring.id().to_string()).map_err(|error| error.to_string()),
            Err("cannot render 442: the search for the colouring parity stopped at a limit before it found it".to_owned())
        );
    }

    #[test]
    fn statistics_text_writes_the_five_lines() {
        assert_eq!(
            statistics_text(250, 150, 1001, 17, 3, Duration::from_secs_f64(0.3154)),
            "Samples: 250\nMean folds: 10.01\nMaximum folds: 17\nUnconverged samples: 3\nTime: 0.315 s\n"
        );
    }

    #[test]
    fn statistics_text_writes_a_zero_mean_without_inside_samples() {
        assert!(statistics_text(16, 16, 0, 0, 0, Duration::ZERO).contains("\nMean folds: 0.00\n"));
    }

    #[test]
    fn statistics_text_writes_the_mean_of_the_largest_fold_count() {
        assert!(
            statistics_text(u64::MAX, 0, u64::MAX, 1, 0, Duration::ZERO)
                .contains("\nMean folds: 1.00\n")
        );
        assert!(
            statistics_text(3, 0, u64::MAX, 1, 0, Duration::ZERO)
                .contains("\nMean folds: 6148914691236517205.00\n")
        );
    }

    fn setting(name: &str, value: f64) -> Setting {
        Setting {
            name: name.to_owned(),
            value,
        }
    }

    #[test]
    fn shape_parameters_names_the_first_repeated_setting_and_the_earlier_one() {
        let settings = [
            setting("aspect", 2.0),
            setting("other", 1.0),
            setting("aspect", 3.0),
            setting("other", 4.0),
        ];
        assert_eq!(
            shape_parameters(&settings),
            Err("setting 3 of --set names the same parameter as setting 1".to_owned())
        );
    }

    #[test]
    fn shape_parameters_keeps_every_setting_with_a_new_name() {
        let Ok(parameters) = shape_parameters(&[setting("aspect", 2.0), setting("other", 1.0)])
        else {
            panic!("settings with distinct names are kept");
        };
        let mut expected = escher_domain::ShapeParameters::new();
        expected.set("aspect", 2.0);
        expected.set("other", 1.0);
        assert_eq!(parameters, expected);
    }

    fn text_of(signature: &str, settings: &[Setting]) -> Option<String> {
        let Ok(signature) = signature.parse::<Signature>() else {
            panic!("{signature} is a valid signature");
        };
        setting_text(&signature, settings)
    }

    #[test]
    fn setting_text_names_the_first_unknown_setting_and_the_parameters() {
        let cases = [
            (
                "*2222",
                vec![setting("aspect", 9.0), setting("foo", 1.0)],
                "the signature has no shape parameter foo, and its parameter is aspect",
            ),
            (
                "*2222",
                vec![setting("skew", 0.1)],
                "the signature has no shape parameter skew, and its parameter is aspect",
            ),
            (
                "*632",
                vec![setting("aspect", 1.0)],
                "the signature has no shape parameter aspect, nor any other",
            ),
            (
                "o",
                vec![setting("skew", 0.0), setting("Aspect", 1.0)],
                "the signature has no shape parameter Aspect, and its parameters are aspect and skew",
            ),
            (
                "*22222222",
                vec![setting("slice2.length", 1.0)],
                "the signature has no shape parameter slice2.length, and its parameters are \
                 cornerpair1.length, cornerpair2.length, cornerpair3.length, cornerpair4.length, \
                 and slice1.length",
            ),
            (
                "*2222",
                vec![setting("a\u{1b}[31m\u{e9}", 1.0)],
                "the signature has no shape parameter a?[31m?, and its parameter is aspect",
            ),
        ];
        for (signature, settings, expected) in cases {
            assert_eq!(
                text_of(signature, &settings),
                Some(expected.to_owned()),
                "{signature}"
            );
        }
    }

    #[test]
    fn setting_text_names_the_value_and_range_of_the_first_setting_out_of_range() {
        let cases = [
            (
                "*2222",
                vec![setting("aspect", 9.0)],
                "the value 9 of the parameter aspect is outside its range of 0.125 to 8",
            ),
            (
                "o",
                vec![setting("skew", f64::NAN), setting("aspect", 0.0)],
                "the value NaN of the parameter skew is outside its range of -0.5 to 0.5",
            ),
            (
                "o",
                vec![setting("skew", 0.0), setting("aspect", 9.0)],
                "the value 9 of the parameter aspect is outside its range of 0.125 to 8",
            ),
            (
                "*3333",
                vec![setting("slice1.length", f64::INFINITY)],
                "the value inf of the parameter slice1.length is outside its range of 0.5 to 3.5",
            ),
        ];
        for (signature, settings, expected) in cases {
            assert_eq!(
                text_of(signature, &settings),
                Some(expected.to_owned()),
                "{signature}"
            );
        }
    }

    #[test]
    fn setting_text_is_none_for_settings_that_escher_domain_accepts() {
        assert_eq!(text_of("*2222", &[setting("aspect", 8.0)]), None);
        assert_eq!(text_of("o", &[]), None);
    }

    #[test]
    fn fill_item_reads_an_image_item_with_its_path_as_given() {
        assert_eq!(fill_item(1, "image:fill.png"), Ok(Item::Image("fill.png")));
        assert_eq!(fill_item(1, "image:~/a.png"), Ok(Item::Image("~/a.png")));
        assert_eq!(
            fill_item(1, "image:$HOME/*.png"),
            Ok(Item::Image("$HOME/*.png"))
        );
        assert_eq!(fill_item(1, "image: a.png"), Ok(Item::Image(" a.png")));
    }

    #[test]
    fn fill_item_reads_image_without_a_path_as_a_bad_colour() {
        for text in ["image:", "Image:a.png", " image:a.png", "image"] {
            assert_eq!(
                fill_item(3, text),
                Err(
                    "colour 3 of --colours is not of the form #rgb, #rrggbb, or image:FILE"
                        .to_owned()
                ),
                "{text:?}"
            );
        }
    }

    struct FailingReader;

    impl std::io::Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("bad \u{1b}[31m\u{e9}\u{221e} text"))
        }
    }

    #[test]
    fn decode_image_replaces_each_character_outside_printable_ascii_in_the_decoder_line() {
        let signature: Signature = "*732".parse().expect("the signature is valid");
        let Err(error) = decode_image(Task::Render(&signature), 2, &mut FailingReader) else {
            panic!("a reader that fails gives no image");
        };
        let mut stderr = Vec::new();
        assert_eq!(report(&error, &mut stderr), Status::Failure);
        assert_eq!(
            String::from_utf8(stderr).expect("standard error is UTF-8"),
            "error: cannot render *732: the image of colour 2 of --colours cannot be used\n  \
             caused by: cannot decode image: the image cannot be read\n  \
             caused by: bad ?[31m?\u{221e} text\n"
        );
    }

    fn rendered(domain: &FundamentalDomain, fill: Fill) -> Vec<u8> {
        let options = RenderOptions::builder(fill)
            .size(48, 32)
            .samples(1)
            .build()
            .expect("the options are valid");
        Renderer::new(domain, options)
            .expect("the renderer is built")
            .render()
            .0
    }

    #[test]
    fn coset_fill_of_one_colour_draws_the_bytes_of_one() {
        let colour = Colour::from_srgb8(0x91, 0x36, 0x71);
        for text in ["*732", "732", "*2222"] {
            let signature: Signature = text.parse().expect("the signature is valid");
            let domain = domain(text);
            let one = rendered(&domain, Fill::One(colour));
            for colouring in [Colouring::One, id("1a")] {
                let fill = coset_fill(
                    &domain,
                    Task::Render(&signature),
                    colouring,
                    None,
                    vec![Paint::Solid(colour)],
                    SearchBudget::DEFAULT,
                )
                .expect("the trivial colouring is found");
                assert!(matches!(fill, Fill::Cosets(_)), "{text} {colouring}");
                assert_eq!(rendered(&domain, fill), one, "{text} {colouring}");
            }
        }
    }

    fn regular_face(text: &str) -> escher_render::WholeFace {
        whole_face(&domain(text), [false, true, false]).expect("the tiling is regular")
    }

    fn arrangement_text(text: &str, id: &str, steps: u64) -> Result<String, String> {
        task_arrangement_text(|signature| Task::Render(signature), text, id, steps)
    }

    fn task_arrangement_text(
        task: fn(&Signature) -> Task<'_>,
        text: &str,
        id: &str,
        steps: u64,
    ) -> Result<String, String> {
        let signature: Signature = text.parse().expect("the signature is valid");
        let budget = SearchBudget::new(steps).expect("the budget is valid");
        arrangement(
            &regular_face(text),
            task(&signature),
            colouring_id(id),
            budget,
        )
        .map(|found| found.id().to_string())
        .map_err(|error| error.to_string())
    }

    #[test]
    fn an_arrangement_id_resolves_to_the_arrangement_of_that_id() {
        assert_eq!(
            arrangement_text("*632", "12e", SearchBudget::DEFAULT.get()),
            Ok("12e".to_owned())
        );
    }

    #[test]
    fn an_arrangement_search_stopped_at_a_limit_claims_nothing_about_the_id() {
        assert_eq!(
            arrangement_text("*632", "12a", 1),
            Err(
                "cannot render *632: the search for the arrangement 12a stopped at a limit before it found it"
                    .to_owned()
            )
        );
        assert_eq!(
            task_arrangement_text(|signature| Task::Masks(signature), "*632", "12a", 1),
            Err(
                "cannot render the masks of *632: the search for the arrangement 12a stopped at a limit before it found it"
                    .to_owned()
            )
        );
    }

    #[test]
    fn a_missing_arrangement_gives_the_number_of_arrangements_in_words() {
        assert_eq!(
            arrangement_text("*532", "10b", SearchBudget::DEFAULT.get()),
            Err(
                "cannot render *532: the arrangement 10b does not exist, because the tiling has 1 arrangement"
                    .to_owned()
            )
        );
    }

    #[test]
    fn a_complete_search_with_no_arrangement_says_the_tiling_has_none() {
        let face = regular_face("*632");
        let identity: escher_group::Word = core::iter::empty().collect();
        let found = escher_group::free_classes_of_index(
            face.presentation(),
            &[identity],
            escher_group::IndexLimit::new(face.index()).expect("the index is within the limit"),
            SearchBudget::DEFAULT,
        )
        .expect("the search runs");
        assert!(found.is_complete());
        assert_eq!(
            found_arrangement(&found, colouring_id("12a")).map(|found| found.id().to_string()),
            Err("the tiling has no arrangement that puts one picture on every polygon".to_owned())
        );
        let complete = arrangements(&face, SearchBudget::DEFAULT).expect("the search runs");
        assert_eq!(complete.list().len(), 7);
    }

    #[test]
    fn picture_item_takes_one_colour_or_image_item_and_the_first_default_colour() {
        let id = colouring_id("12a");
        let [red, green, blue] = DEFAULT_COLOURS[0];
        assert_eq!(
            picture_item(id, None),
            Ok(Item::Colour(Colour::from_srgb8(red, green, blue)))
        );
        assert_eq!(
            picture_item(id, Some("#123")),
            Ok(Item::Colour(Colour::from_srgb8(0x11, 0x22, 0x33)))
        );
        assert_eq!(
            picture_item(id, Some("image:a.png")),
            Ok(Item::Image("a.png"))
        );
        assert_eq!(
            picture_item(id, Some("#123,#456")),
            Err("the arrangement 12a needs 1 colour, and 2 were given".to_owned())
        );
        assert_eq!(
            picture_item(id, Some("red")),
            Err("colour 1 of --colours is not of the form #rgb, #rrggbb, or image:FILE".to_owned())
        );
    }
}
