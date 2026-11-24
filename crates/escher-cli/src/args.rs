use std::path::PathBuf;

use core::fmt;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use escher_group::{ColouringId, IndexLimit};
use escher_render::{BorderUnit, HyperbolicModel};

const SIGNATURE_HELP: &str = "The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'";

const SIZE_HELP: &str = "The image size in pixels, as WIDTHxHEIGHT";

const COLOURING_HELP: &str = "One colour for every tile, parity for the colouring with two colours in which no two neighbouring tiles share a colour, or the id of a colouring that the command info --colourings lists, such as 4a. The default is parity when the tiling has it, and one otherwise";

const FILL_DISPLAY_ORDER: usize = 3;

const SET_DISPLAY_ORDER: usize = 15;

const WYTHOFF_HELP: &str = "Draw the Wythoff construction of a reflection triangle instead of its tiles, as three digits, one for each corner of the signature in the order that info prints, where 1 turns on the mirror opposite that corner, such as 010 for the regular tiling {7,3} of *732. --colours then gives one colour for each face type, in the order of the corners that have a face";

const MASKS_WYTHOFF_HELP: &str = "With --arrangement, write one mask of the polygon of the regular tiling of a Wythoff construction for each picture, as three digits as for render, such as 010 for the heptagons of *732: for one picture to the --output file itself, and for several to files named as the masks of a colouring, mask-0.png for picture A, mask-1.png for B, and so on";

const ARRANGEMENT_HELP: &str = "With --wythoff on a regular tiling, put a picture on every polygon, turned or mirrored as the arrangement with this id says, such as 12a, which the command info --wythoff lists. --colours then gives one colour or image:FILE for each picture of the arrangement, in the order A, B, and so on, each painted over the mask that render-masks writes for that picture";

const MASKS_ARRANGEMENT_HELP: &str = "With --wythoff on a regular tiling, border each edge of the mask of each picture to show how it meets its neighbour in the arrangement with this id, such as 12a, which the command info --wythoff lists";

const SET_HELP: &str = "Set a shape parameter of the tiles, as NAME=VALUE, such as aspect=1.5, once for each parameter. --help lists the parameters, and info <SIGNATURE> prints those of a signature";

const SET_LONG_HELP: &str = "Set a shape parameter, as NAME=VALUE, such as aspect=1.5.

A shape parameter is a number that changes the shape of the tiles without
changing their symmetry. Which parameters exist depends on the signature:

  aspect
      The ratio of the sides of the rectangle or the parallelogram, for
      *2222, 2222, o, **, xx, *x, 22*, 22x, and 2*22.
  skew
      The shear of the parallelogram of o and of the two rectangles of 2222.
  cornerpair<k>.length
      For a hyperbolic reflection polygon *p1...pn with 4 to 8 corners, the
      length of the side between the two right angles of its k-th pair of
      adjacent corners of order 2.
  slice<i>.length
      For the same polygons, the length of the i-th cut across the polygon,
      a common perpendicular of two of its sides. A polygon with n corners
      has n - 3 lengths, corner pairs first.

Every other signature has no shape parameter. escher info <SIGNATURE>
prints one Parameter line for each parameter of the signature, with its
range and default.

Give the option once for each parameter, as in:

  escher render o --set aspect=2 --set skew=0.25 -o /tmp/torus.png
  escher render '*3333' --set slice1.length=2 -o /tmp/square.png";

const CENTRE_HELP: &str = "Centre the picture on corner N of the fundamental domain, counted from 0 in the order of the angles on the line \"Corner angles in degrees\" that info prints, such as 0 for the corner of order 7 of *732. Without it, the picture is centred on the domain. --help gives an example";

const CENTRE_LONG_HELP: &str = "Centre the picture on corner N of the fundamental domain. Without --centre, the picture is centred on the domain.

The corners are numbered from 0, in the order of the angles that escher info <SIGNATURE> prints on its line \"Corner angles in degrees\", one number for each angle. For example, escher info '*732' prints

Corner angles in degrees: 25.714286, 60.000000, 90.000000

so 0 is the corner of order 7, whose angle is 180/7 degrees, 1 is the corner of order 3, and 2 is the corner of order 2. With --wythoff 010, which draws the heptagons of {7,3}, --centre 0 puts a heptagon at the centre, and --centre 1 a vertex where three heptagons meet:

escher render '*732' --wythoff 010 --centre 0 -o /tmp/heptagon.png
escher render '*732' --wythoff 010 --centre 1 -o /tmp/vertex.png

Two corners can have the same order, as all three of *333 do, so the number names a corner, and not its order. A domain can have more corners than its signature has orders, such as the four corners of 732, and info prints an angle for each. A number at or above the number of angles is refused, and so is every number for 1, whose domain has no corner. Only the view moves: the tiling, its colours, and its borders stay the same.";

#[derive(Debug, Parser)]
#[command(
    bin_name = "escher",
    version,
    about,
    arg_required_else_help = true,
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    #[command(
        about = "Print the canonical form, the geometry, and other properties of a signature in orbifold notation"
    )]
    Info {
        #[arg(help = SIGNATURE_HELP)]
        signature: String,
        #[arg(
            long,
            help = "Verify the fundamental domain of the signature and print the result of each check"
        )]
        check: bool,
        #[arg(
            long,
            help = "Print the generators and relators of the signature and of its fundamental domain, and for a spherical signature the group order by coset enumeration"
        )]
        presentation: bool,
        #[arg(
            long,
            help = "Print the valid colourings of the fundamental domain, in which no two neighbouring tiles share a colour, one for each conjugacy class of subgroups, with its id, up to the largest number of colours"
        )]
        colourings: bool,
        #[arg(
            long,
            value_name = "N",
            default_value = "12",
            value_parser = parse_max_colours,
            help = "The largest number of colours of the valid colourings to count and list, and with --wythoff the largest number of pictures of the arrangements to list, from 1 to 64"
        )]
        max_colours: IndexLimit,
        #[arg(
            long,
            value_name = "MARKS",
            value_parser = parse_wythoff,
            help = "List the arrangements of the regular tiling of a Wythoff construction, as three digits as for render, such as 010 for the heptagons of *732. Each arrangement puts one of its pictures, A, B, and so on, on every polygon, and the number of its id is twice the number of sides of a polygon times the number of pictures. For each edge of each picture, it lists the edge of the neighbour that lies on it and whether that neighbour is turned or mirrored, with the edges numbered counterclockwise around each mask from edge 0, the edge with the black tick, and with several pictures each edge written after the letter of its picture, such as B2"
        )]
        wythoff: Option<[bool; 3]>,
    },
    #[command(
        about = "Render the tiling of a signature in orbifold notation and write it to a PNG file"
    )]
    Render(Box<RenderArgs>),
    #[command(
        about = "Write one PNG mask for each colour of a colouring: the tile as a flat polygon in white, over which to paint the image fill of that colour for render, with each edge bordered in the same colour as every edge that it can meet in the tiling",
        mut_arg("wythoff", |arg| arg.help(MASKS_WYTHOFF_HELP)),
        mut_arg("arrangement", |arg| arg.help(MASKS_ARRANGEMENT_HELP))
    )]
    RenderMasks(Box<MasksArgs>),
}

#[derive(Debug, Args)]
pub(crate) struct TilingArgs {
    #[arg(help = SIGNATURE_HELP)]
    pub(crate) signature: String,
    #[arg(
        long,
        value_name = "WIDTHxHEIGHT",
        default_value = "512x512",
        value_parser = parse_size,
        help = SIZE_HELP
    )]
    pub(crate) size: Size,
    #[arg(
        long,
        value_name = "COLOURING",
        value_parser = parse_colouring,
        help = COLOURING_HELP
    )]
    pub(crate) colouring: Option<Colouring>,
    #[arg(
        long,
        value_name = "NAME=VALUE",
        value_parser = parse_setting,
        action = ArgAction::Append,
        display_order = SET_DISPLAY_ORDER,
        help = SET_HELP,
        long_help = SET_LONG_HELP
    )]
    pub(crate) set: Vec<Setting>,
    #[arg(
        long,
        value_name = "MARKS",
        value_parser = parse_wythoff,
        display_order = FILL_DISPLAY_ORDER + 1,
        help = WYTHOFF_HELP
    )]
    pub(crate) wythoff: Option<[bool; 3]>,
    #[arg(
        long,
        value_name = "ID",
        value_parser = parse_arrangement,
        display_order = FILL_DISPLAY_ORDER + 2,
        help = ARRANGEMENT_HELP
    )]
    pub(crate) arrangement: Option<ColouringId>,
}

#[derive(Debug, Args)]
pub(crate) struct MasksArgs {
    #[arg(
        short = 'o',
        long,
        value_name = "FILE",
        help = "The file name from which the masks are named: mask.png gives mask-0.png, mask-1.png, and so on, one for each colour. Existing files are replaced"
    )]
    pub(crate) output: PathBuf,
    #[command(flatten)]
    pub(crate) tiling: TilingArgs,
}

#[derive(Debug, Args)]
pub(crate) struct RenderArgs {
    #[arg(
        short = 'o',
        long,
        value_name = "FILE",
        help = "The PNG file to write. An existing file is replaced"
    )]
    pub(crate) output: PathBuf,
    #[command(flatten)]
    pub(crate) tiling: TilingArgs,
    #[arg(
        long,
        display_order = FILL_DISPLAY_ORDER,
        help = "The fill colours, separated by commas, one for each colour of the colouring, each as #rgb, #rrggbb, or image:FILE for a PNG image painted over the mask of that colour that render-masks writes. The default takes the first colours of a built-in list of 12. In a POSIX shell, put the colours in single quotes, as in '#e9c46a,image:fill-1.png'"
    )]
    pub(crate) colours: Option<String>,
    #[arg(
        long,
        value_name = "WIDTH",
        help = "The border width, in the unit of --border-unit. 0 draws no border. The default is 1 for pixels and 0.05 for intrinsic"
    )]
    pub(crate) border_width: Option<f64>,
    #[arg(
        long,
        value_name = "UNIT",
        value_enum,
        default_value_t = BorderUnitChoice::Pixels,
        help = "The unit of --border-width: pixels of the image, or intrinsic for a length in the geometry of the tiling, such as 0.05"
    )]
    pub(crate) border_unit: BorderUnitChoice,
    #[arg(
        long,
        value_name = "COLOUR",
        default_value = "#000000",
        help = "The border colour, as #rgb or #rrggbb"
    )]
    pub(crate) border_colour: String,
    #[arg(
        long,
        value_name = "N",
        default_value_t = 4,
        help = "The number of samples along each side of a pixel. The pixel averages N by N samples"
    )]
    pub(crate) samples: u32,
    #[arg(
        long,
        value_name = "N",
        default_value_t = 1500,
        help = "The largest number of folds of one sample. A sample that needs more takes the border colour"
    )]
    pub(crate) max_folds: u32,
    #[arg(
        long,
        value_name = "COLOUR",
        default_value = "#ffffff",
        help = "The colour outside the model of a hyperbolic tiling or the ball of a spherical tiling, and where an image fill is transparent or has no pixel, as #rgb or #rrggbb"
    )]
    pub(crate) background: String,
    #[arg(
        long,
        value_name = "N",
        value_parser = clap::value_parser!(u16).range(1..=256),
        help = "The number of threads, from 1 to 256. The default is the available parallelism, at most 256"
    )]
    pub(crate) threads: Option<u16>,
    #[arg(long, help = "Print statistics of the rendering to standard output")]
    pub(crate) stats: bool,
    #[arg(
        long,
        value_name = "N",
        display_order = SET_DISPLAY_ORDER + 1,
        help = CENTRE_HELP,
        long_help = CENTRE_LONG_HELP
    )]
    pub(crate) centre: Option<u32>,
    #[arg(
        long,
        value_enum,
        display_order = SET_DISPLAY_ORDER + 2,
        default_value_t = ModelChoice::Disk,
        help = "The model in which a hyperbolic tiling is drawn: the Poincare disk, the upper half-plane, the band, or the Klein disk"
    )]
    pub(crate) model: ModelChoice,
    #[arg(
        long,
        value_name = "YAW,PITCH,ROLL",
        display_order = SET_DISPLAY_ORDER + 3,
        default_value = "0,0,0",
        value_parser = parse_triple,
        allow_hyphen_values = true,
        help = "The rotation of the ball of a spherical tiling, as three angles in degrees. YAW turns the front of the ball to the right, PITCH turns it up, and ROLL turns the image counterclockwise, in this order"
    )]
    pub(crate) rotation: [f64; 3],
    #[arg(
        long,
        value_name = "X,Y,Z",
        display_order = SET_DISPLAY_ORDER + 4,
        default_value = "-1,1,2",
        value_parser = parse_triple,
        allow_hyphen_values = true,
        help = "The direction from the ball of a spherical tiling to the light, as X,Y,Z, with X to the right, Y up, and Z toward the viewer. The length is ignored"
    )]
    pub(crate) light: [f64; 3],
    #[arg(
        long,
        value_enum,
        default_value_t = ShadingChoice::Phong,
        display_order = SET_DISPLAY_ORDER + 5,
        help = "Phong shading of the ball of a spherical tiling, or none"
    )]
    pub(crate) shading: ShadingChoice,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Setting {
    pub(crate) name: String,
    pub(crate) value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Size {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Colouring {
    One,
    Parity,
    Id(ColouringId),
}

impl Colouring {
    pub(crate) fn colours(self) -> u32 {
        match self {
            Self::One => 1,
            Self::Parity => 2,
            Self::Id(id) => id.colours(),
        }
    }
}

impl fmt::Display for Colouring {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One => formatter.write_str("one"),
            Self::Parity => formatter.write_str("parity"),
            Self::Id(id) => fmt::Display::fmt(id, formatter),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum ShadingChoice {
    Phong,
    #[value(name = "none")]
    Flat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum ModelChoice {
    Disk,
    HalfPlane,
    Band,
    Klein,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum BorderUnitChoice {
    Pixels,
    Intrinsic,
}

impl BorderUnitChoice {
    pub(crate) fn unit(self) -> BorderUnit {
        match self {
            Self::Pixels => BorderUnit::Pixels,
            Self::Intrinsic => BorderUnit::Intrinsic,
        }
    }

    pub(crate) fn default_width(self) -> f64 {
        match self {
            Self::Pixels => 1.0,
            Self::Intrinsic => 0.05,
        }
    }
}

impl ModelChoice {
    pub(crate) fn model(self) -> HyperbolicModel {
        match self {
            Self::Disk => HyperbolicModel::Disk,
            Self::HalfPlane => HyperbolicModel::HalfPlane,
            Self::Band => HyperbolicModel::Band,
            Self::Klein => HyperbolicModel::Klein,
        }
    }
}

fn parse_size(text: &str) -> Result<Size, &'static str> {
    let error = "expected two decimal integers joined by x, such as 512x512";
    let (width, height) = text.split_once('x').ok_or(error)?;
    Ok(Size {
        width: decimal(width).ok_or(error)?,
        height: decimal(height).ok_or(error)?,
    })
}

fn parse_setting(text: &str) -> Result<Setting, &'static str> {
    let error = "expected a parameter name and a number joined by =, such as aspect=1.5";
    let (name, value) = text.split_once('=').ok_or(error)?;
    if name.is_empty() {
        return Err(error);
    }
    Ok(Setting {
        name: name.to_owned(),
        value: value.parse().ok().ok_or(error)?,
    })
}

fn parse_colouring(text: &str) -> Result<Colouring, &'static str> {
    match text {
        "one" => Ok(Colouring::One),
        "parity" => Ok(Colouring::Parity),
        _ => text
            .parse()
            .ok()
            .map(Colouring::Id)
            .ok_or("expected one, parity, or a colouring id such as 4a"),
    }
}

fn parse_wythoff(text: &str) -> Result<[bool; 3], &'static str> {
    let mark = |byte: &u8| match byte {
        b'0' => Some(false),
        b'1' => Some(true),
        _ => None,
    };
    text.as_bytes()
        .iter()
        .map(mark)
        .collect::<Option<Vec<bool>>>()
        .and_then(|marks| <[bool; 3]>::try_from(marks).ok())
        .ok_or("expected three digits 0 or 1, such as 010")
}

fn parse_arrangement(text: &str) -> Result<ColouringId, &'static str> {
    ColouringId::parse_arrangement(text)
        .ok()
        .ok_or("expected an arrangement id such as 12a")
}

fn parse_max_colours(text: &str) -> Result<IndexLimit, String> {
    decimal(text)
        .and_then(|colours| IndexLimit::new(colours).ok())
        .ok_or_else(|| {
            format!(
                "expected a decimal integer from 1 to {}",
                IndexLimit::MAX.get()
            )
        })
}

fn parse_triple(text: &str) -> Result<[f64; 3], &'static str> {
    let error = "expected three numbers separated by commas, such as 20,35,0";
    let mut items = text.split(',');
    let (Some(first), Some(second), Some(third), None) =
        (items.next(), items.next(), items.next(), items.next())
    else {
        return Err(error);
    };
    let number = |item: &str| item.parse::<f64>().ok().ok_or(error);
    Ok([number(first)?, number(second)?, number(third)?])
}

fn decimal(text: &str) -> Option<u32> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

#[cfg(test)]
mod tests {
    use escher_group::IndexLimit;

    use std::collections::BTreeSet;

    use escher_domain::parameters;
    use escher_signature::Signature;

    use clap::ValueEnum;
    use escher_render::{BorderUnit, HyperbolicModel};

    use super::{
        BorderUnitChoice, CENTRE_LONG_HELP, Colouring, ModelChoice, SET_LONG_HELP, Setting,
        parse_arrangement, parse_colouring, parse_max_colours, parse_setting, parse_triple,
        parse_wythoff,
    };

    #[test]
    fn each_model_value_maps_to_the_model_of_the_same_name() {
        let cases = [
            ("disk", HyperbolicModel::Disk),
            ("half-plane", HyperbolicModel::HalfPlane),
            ("band", HyperbolicModel::Band),
            ("klein", HyperbolicModel::Klein),
        ];
        assert_eq!(ModelChoice::value_variants().len(), cases.len());
        for (text, model) in cases {
            match ModelChoice::from_str(text, false) {
                Ok(choice) => assert_eq!(choice.model(), model, "{text}"),
                Err(error) => panic!("{text} is rejected: {error}"),
            }
        }
        for text in ["poincare", "Band", "half_plane", ""] {
            assert!(ModelChoice::from_str(text, false).is_err(), "{text}");
        }
    }

    #[test]
    fn each_border_unit_value_maps_to_the_unit_of_the_same_name() {
        let cases = [
            ("pixels", BorderUnit::Pixels),
            ("intrinsic", BorderUnit::Intrinsic),
        ];
        assert_eq!(BorderUnitChoice::value_variants().len(), cases.len());
        for (text, unit) in cases {
            match BorderUnitChoice::from_str(text, false) {
                Ok(choice) => assert_eq!(choice.unit(), unit, "{text}"),
                Err(error) => panic!("{text} is rejected: {error}"),
            }
        }
        for text in ["px", "Intrinsic", "pixel", ""] {
            assert!(BorderUnitChoice::from_str(text, false).is_err(), "{text}");
        }
    }

    const EUCLIDEAN: [&str; 17] = [
        "*632", "*442", "*333", "*2222", "632", "442", "333", "2222", "o", "**", "xx", "*x", "22*",
        "22x", "2*22", "4*2", "3*3",
    ];

    const OTHERS: [&str; 34] = [
        "*332",
        "*432",
        "*532",
        "*522",
        "*55",
        "332",
        "432",
        "532",
        "522",
        "55",
        "3*2",
        "2*3",
        "3*",
        "3x",
        "*",
        "x",
        "1",
        "*732",
        "732",
        "*3333",
        "*3222",
        "*22222",
        "*5432",
        "*222222",
        "*2222222",
        "*22222222",
        "*33333333",
        "*(100)432",
        "3333",
        "22222",
        "*(101)32",
        "*222222222",
        "3*(inf)",
        "4*3",
    ];

    fn names_of(text: &str) -> Vec<String> {
        let Ok(signature) = text.parse::<Signature>() else {
            panic!("{text} is a valid signature");
        };
        parameters(&signature)
            .iter()
            .map(|spec| spec.name().to_owned())
            .collect()
    }

    fn headers() -> BTreeSet<String> {
        SET_LONG_HELP
            .lines()
            .filter_map(|line| line.strip_prefix("  "))
            .filter(|rest| !rest.starts_with(' ') && !rest.contains(' '))
            .map(|header| header.replace("<k>", "").replace("<i>", ""))
            .collect()
    }

    fn block(header: &str) -> Vec<&'static str> {
        SET_LONG_HELP
            .lines()
            .skip_while(|line| *line != format!("  {header}"))
            .skip(1)
            .take_while(|line| line.starts_with("      "))
            .flat_map(str::split_whitespace)
            .map(|word| word.trim_end_matches([',', '.']))
            .collect()
    }

    #[test]
    fn set_long_help_names_exactly_the_parameters_that_escher_domain_lists() {
        let listed: BTreeSet<String> = EUCLIDEAN
            .iter()
            .chain(&OTHERS)
            .flat_map(|text| names_of(text))
            .map(|name| name.replace(|character: char| character.is_ascii_digit(), ""))
            .collect();
        assert_eq!(headers(), listed);
    }

    #[test]
    fn set_long_help_lists_the_euclidean_signatures_of_aspect_and_skew() {
        for name in ["aspect", "skew"] {
            let words = block(name);
            assert!(!words.is_empty(), "{name}");
            for text in EUCLIDEAN {
                assert_eq!(
                    words.contains(&text),
                    names_of(text).iter().any(|listed| listed == name),
                    "{name} on {text}"
                );
            }
        }
    }

    #[test]
    fn centre_long_help_quotes_the_corner_angles_that_info_prints_for_732() {
        const LABEL: &str = "Corner angles in degrees: ";
        let quoted: Vec<&str> = CENTRE_LONG_HELP
            .lines()
            .filter(|line| line.starts_with(LABEL))
            .collect();
        let mut stdout = Vec::new();
        let status = crate::run(["escher", "info", "*732"], &mut stdout, &mut Vec::new());
        assert_eq!(status, crate::Status::Success);
        let printed = String::from_utf8(stdout).expect("the output of info is UTF-8");
        let printed: Vec<&str> = printed
            .lines()
            .filter(|line| line.starts_with(LABEL))
            .collect();
        assert_eq!(printed.len(), 1);
        assert_eq!(quoted, printed);
    }

    fn id(text: &str) -> Colouring {
        Colouring::Id(crate::colouring_id(text))
    }

    #[test]
    fn parse_colouring_reads_the_aliases_and_the_ids() {
        assert_eq!(parse_colouring("one"), Ok(Colouring::One));
        assert_eq!(parse_colouring("parity"), Ok(Colouring::Parity));
        for text in ["1a", "3a", "12dk", "64zzzz"] {
            let colouring = parse_colouring(text);
            assert_eq!(colouring, Ok(id(text)), "{text:?}");
            assert_eq!(
                colouring.map(|colouring| colouring.to_string()),
                Ok(text.to_owned())
            );
        }
    }

    #[test]
    fn parse_colouring_rejects_other_forms() {
        for text in [
            "three", "3", "03a", "3A", "65a", "0a", "3abcde", "", " 3a", "3a ", " one", "one ",
        ] {
            assert_eq!(
                parse_colouring(text),
                Err("expected one, parity, or a colouring id such as 4a"),
                "{text:?}"
            );
        }
    }

    #[test]
    fn parse_max_colours_reads_the_default_and_the_range() {
        assert_eq!(parse_max_colours("12"), Ok(IndexLimit::DEFAULT));
        assert_eq!(
            parse_max_colours("1"),
            IndexLimit::new(1).map_err(|error| error.to_string())
        );
        assert_eq!(parse_max_colours("64"), Ok(IndexLimit::MAX));
        for text in ["0", "65", "x", "", "+5", " 5", "4294967296"] {
            assert_eq!(
                parse_max_colours(text),
                Err("expected a decimal integer from 1 to 64".to_owned()),
                "{text:?}"
            );
        }
    }

    const MESSAGE: &str = "expected a parameter name and a number joined by =, such as aspect=1.5";

    #[test]
    fn parse_setting_reads_a_name_and_a_number() {
        assert_eq!(
            parse_setting("aspect=1.5"),
            Ok(Setting {
                name: "aspect".to_owned(),
                value: 1.5
            })
        );
    }

    #[test]
    fn parse_setting_reads_nan_as_a_number() {
        let Ok(setting) = parse_setting("aspect=nan") else {
            panic!("nan is a number of the f64 parser");
        };
        assert_eq!(setting.name, "aspect");
        assert!(setting.value.is_nan());
    }

    #[test]
    fn parse_setting_rejects_other_forms() {
        for text in ["aspect", "=2", "aspect=", "aspect=x", "a=b=1"] {
            assert_eq!(parse_setting(text), Err(MESSAGE), "{text:?}");
        }
    }

    const TRIPLE_MESSAGE: &str = "expected three numbers separated by commas, such as 20,35,0";

    #[test]
    fn parse_triple_reads_three_numbers() {
        assert_eq!(parse_triple("20,35,0"), Ok([20.0, 35.0, 0.0]));
        assert_eq!(parse_triple("-1,1,2"), Ok([-1.0, 1.0, 2.0]));
    }

    #[test]
    fn parse_triple_reads_nan_as_a_number() {
        let Ok([first, second, third]) = parse_triple("nan,0,0") else {
            panic!("nan is a number of the f64 parser");
        };
        assert!(first.is_nan());
        assert_eq!(second, 0.0);
        assert_eq!(third, 0.0);
    }

    #[test]
    fn parse_triple_rejects_other_forms() {
        for text in [
            "1,2", "a,b,c", "1,2,3,4", "1,,2", "", " 1,2,3", "1,2,3 ", "1, 2,3",
        ] {
            assert_eq!(parse_triple(text), Err(TRIPLE_MESSAGE), "{text:?}");
        }
    }

    #[test]
    fn parse_wythoff_reads_one_mark_for_each_digit_in_order() {
        assert_eq!(parse_wythoff("000"), Ok([false, false, false]));
        assert_eq!(parse_wythoff("111"), Ok([true, true, true]));
        assert_eq!(parse_wythoff("010"), Ok([false, true, false]));
        assert_eq!(parse_wythoff("110"), Ok([true, true, false]));
    }

    #[test]
    fn parse_wythoff_rejects_other_forms() {
        for text in [
            "01",
            "0101",
            "0a1",
            "0,1,0",
            "",
            " 010",
            "010 ",
            "012",
            "\u{663}10",
        ] {
            assert_eq!(
                parse_wythoff(text),
                Err("expected three digits 0 or 1, such as 010"),
                "{text:?}"
            );
        }
    }

    #[test]
    fn parse_arrangement_reads_the_ids_of_escher_group_only() {
        for text in ["6a", "12a", "14c", "64zzzz", "65a", "84zzzz"] {
            assert_eq!(
                parse_arrangement(text).map(|id| id.to_string()),
                Ok(text.to_owned()),
                "{text:?}"
            );
        }
        for text in [
            "12A", "12", "012a", "85a", "a", "", " 12a", "12a ", "one", "parity",
        ] {
            assert_eq!(
                parse_arrangement(text),
                Err("expected an arrangement id such as 12a"),
                "{text:?}"
            );
        }
    }
}
