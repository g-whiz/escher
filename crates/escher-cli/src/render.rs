use core::fmt::Display;
use std::collections::HashMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use escher_domain::{FundamentalDomain, ParameterSpec, ShapeParameters, parameters};
use escher_group::{
    ColouringId, ColouringSet, ColouringsOfIndex, IndexLimit, SearchBudget, colourings_of_index,
};
use escher_render::{
    Colour, CosetFill, Fill, RenderOptions, RenderStats, Renderer, Shading, encode_png,
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

pub(crate) fn render(args: &RenderArgs, stdout: &mut dyn Write) -> anyhow::Result<()> {
    let statistics = Statistics::from_flag(args.stats);
    let signature: Signature = args
        .signature
        .parse()
        .map_err(|error| SignatureError::new(&args.signature, error))?;
    let border_colour = parse_colour(&args.border_colour).ok_or_else(|| {
        cannot_render(
            &signature,
            "--border-colour is not of the form #rgb or #rrggbb",
        )
    })?;
    let background = parse_colour(&args.background).ok_or_else(|| {
        cannot_render(
            &signature,
            "--background is not of the form #rgb or #rrggbb",
        )
    })?;
    let parameters =
        shape_parameters(&args.set).map_err(|detail| cannot_render(&signature, detail))?;
    let domain = FundamentalDomain::build_with(&signature, &parameters).map_err(|error| {
        if matches!(
            error.kind(),
            escher_domain::ErrorKind::UnknownParameter
                | escher_domain::ErrorKind::ParameterOutOfRange
        ) && let Some(detail) = setting_text(&signature, &args.set)
        {
            return cannot_render(&signature, detail);
        }
        let context = if error.kind() == escher_domain::ErrorKind::UnsupportedSignature {
            format!("cannot render {signature}: rendering this signature is not supported")
        } else {
            format!("cannot render {signature}")
        };
        anyhow::Error::new(error).context(context)
    })?;
    let colouring = colouring(args.colouring, domain.reverses_orientation())
        .map_err(|detail| cannot_render(&signature, detail))?;
    let colours = fill_colours(colouring, args.colours.as_deref())
        .map_err(|detail| cannot_render(&signature, detail))?;
    let options_with = |fill| {
        render_options(args, fill, border_colour, background)
            .with_context(|| format!("cannot render {signature}"))
    };
    options_with(Fill::One(border_colour))?;
    let fill = fill(
        &domain,
        &signature,
        colouring,
        colours,
        SearchBudget::DEFAULT,
    )?;
    let options = options_with(fill)?;
    let threads = args.threads.map_or_else(default_threads, usize::from);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|error| thread_pool_error(&signature, &error))?;
    let (width, height) = (options.width(), options.height());
    let renderer =
        Renderer::new(&domain, options).with_context(|| format!("cannot render {signature}"))?;
    let start = Instant::now();
    let (pixels, stats) = pool
        .install(|| render_bands(&renderer, width, height))
        .with_context(|| format!("cannot render {signature}"))?;
    let elapsed = start.elapsed();
    let png =
        encode_png(width, height, &pixels).with_context(|| format!("cannot render {signature}"))?;
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

fn render_options(
    args: &RenderArgs,
    fill: Fill,
    border_colour: Colour,
    background: Colour,
) -> escher_render::Result<RenderOptions> {
    let [yaw, pitch, roll] = args.rotation;
    RenderOptions::builder(fill)
        .size(args.size.width, args.size.height)
        .samples(args.samples)
        .border_width(args.border_width)
        .border_colour(border_colour)
        .background(background)
        .max_folds(args.max_folds)
        .rotation(yaw.to_radians(), pitch.to_radians(), roll.to_radians())
        .light(args.light.into())
        .shading(match args.shading {
            ShadingChoice::Phong => Shading::Phong,
            ShadingChoice::Flat => Shading::Flat,
        })
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

fn thread_pool_error(signature: &Signature, error: &dyn Display) -> anyhow::Error {
    anyhow::anyhow!("{error}")
        .context("cannot start the thread pool")
        .context(format!("cannot render {signature}"))
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

fn cannot_render(signature: &Signature, detail: impl Display) -> anyhow::Error {
    anyhow::anyhow!("cannot render {signature}: {detail}")
}

fn colouring(
    requested: Option<Colouring>,
    reverses_orientation: bool,
) -> Result<Colouring, String> {
    match (requested, reverses_orientation) {
        (Some(Colouring::Parity), false) => Err(
            "the colouring parity does not exist, because no side pairing reverses orientation"
                .to_owned(),
        ),
        (Some(colouring), _) => Ok(colouring),
        (None, true) => Ok(Colouring::Parity),
        (None, false) => Ok(Colouring::One),
    }
}

fn fill(
    domain: &FundamentalDomain,
    signature: &Signature,
    colouring: Colouring,
    colours: Vec<Colour>,
    budget: SearchBudget,
) -> anyhow::Result<Fill> {
    let wrong_count = |colours: Vec<Colour>| {
        cannot_render(
            signature,
            format!(
                "internal error: the colouring {colouring} was given {} colours",
                colours.len()
            ),
        )
    };
    match colouring {
        Colouring::One => <[Colour; 1]>::try_from(colours)
            .map(|[colour]| Fill::One(colour))
            .map_err(wrong_count),
        Colouring::Parity => <[Colour; 2]>::try_from(colours)
            .map(Fill::Parity)
            .map_err(wrong_count),
        Colouring::Id(id) => {
            let found = resolve(domain, signature, id, budget)?;
            Ok(Fill::Cosets(
                CosetFill::new(domain, found, colours)
                    .with_context(|| format!("cannot render {signature}"))?,
            ))
        }
    }
}

fn resolve(
    domain: &FundamentalDomain,
    signature: &Signature,
    id: ColouringId,
    budget: SearchBudget,
) -> anyhow::Result<escher_group::Colouring> {
    let context = || format!("cannot render {signature}");
    let presentation = domain.side_presentation().with_context(context)?;
    let colours = IndexLimit::new(id.colours()).with_context(context)?;
    let found = colourings_of_index(&presentation, ColouringSet::All, colours, budget)
        .with_context(context)?;
    found_colouring(&found, id).map_err(|detail| cannot_render(signature, detail))
}

fn found_colouring(
    found: &ColouringsOfIndex,
    id: ColouringId,
) -> Result<escher_group::Colouring, String> {
    match found.find(id) {
        Some(colouring) => Ok(colouring.clone()),
        None if found.is_complete() => Err(missing_text(id, found.list().len())),
        None => Err(format!(
            "the search for the colouring {id} stopped at a limit before it found it"
        )),
    }
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

fn fill_colours(colouring: Colouring, colours: Option<&str>) -> Result<Vec<Colour>, String> {
    let needed = colouring.colours();
    let Some(text) = colours else {
        return usize::try_from(needed)
            .ok()
            .and_then(|needed| DEFAULT_COLOURS.get(..needed))
            .map(|entries| {
                entries
                    .iter()
                    .map(|&[red, green, blue]| Colour::from_srgb8(red, green, blue))
                    .collect()
            })
            .ok_or_else(|| {
                format!(
                    "the colouring {colouring} needs {needed} colours, more than the {} default colours, so give {needed} colours with --colours",
                    DEFAULT_COLOURS.len()
                )
            });
    };
    let items: Vec<&str> = text.split(',').collect();
    if u64::try_from(items.len()).ok() != Some(u64::from(needed)) {
        return Err(format!(
            "the colouring {colouring} needs {}, and {} given",
            count_of_colours(u64::from(needed)),
            given(items.len())
        ));
    }
    items
        .iter()
        .enumerate()
        .map(|(position, item)| fill_colour(position + 1, item))
        .collect()
}

fn fill_colour(position: usize, text: &str) -> Result<Colour, String> {
    parse_colour(text)
        .ok_or_else(|| format!("colour {position} of --colours is not of the form #rgb or #rrggbb"))
}

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
    use escher_render::Colour;
    use escher_signature::Signature;

    use super::{
        DEFAULT_COLOURS, fill_colours, missing_text, parse_colour, resolve, setting_text,
        shape_parameters, statistics_text, thread_pool_error,
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
        let status = report(&thread_pool_error(&signature, &error), &mut stderr);
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

    #[test]
    fn fill_colours_reports_the_position_of_the_first_bad_colour() {
        assert_eq!(
            fill_colours(Colouring::Parity, Some("red,#e9c")),
            Err("colour 1 of --colours is not of the form #rgb or #rrggbb".to_owned())
        );
        assert_eq!(
            fill_colours(Colouring::Parity, Some("#e9c,")),
            Err("colour 2 of --colours is not of the form #rgb or #rrggbb".to_owned())
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
        resolve(&domain(text), &signature, colouring_id(colouring), budget)
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
}
