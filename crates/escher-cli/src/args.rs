use std::path::PathBuf;

use core::fmt;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use escher_group::{ColouringId, IndexLimit};

const SIGNATURE_HELP: &str = "The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'";

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
            help = "Print every colouring of the fundamental domain with its id, up to the largest number of colours"
        )]
        colourings: bool,
        #[arg(
            long,
            value_name = "N",
            default_value = "12",
            value_parser = parse_max_colours,
            help = "The largest number of colours of the colourings to count and list, from 1 to 64"
        )]
        max_colours: IndexLimit,
    },
    #[command(
        about = "Render the tiling of a signature in orbifold notation and write it to a PNG file"
    )]
    Render(Box<RenderArgs>),
}

#[derive(Debug, Args)]
pub(crate) struct RenderArgs {
    #[arg(help = SIGNATURE_HELP)]
    pub(crate) signature: String,
    #[arg(
        short = 'o',
        long,
        value_name = "FILE",
        help = "The PNG file to write. An existing file is replaced"
    )]
    pub(crate) output: PathBuf,
    #[arg(
        long,
        value_name = "WIDTHxHEIGHT",
        default_value = "512x512",
        value_parser = parse_size,
        help = "The image size in pixels, as WIDTHxHEIGHT"
    )]
    pub(crate) size: Size,
    #[arg(
        long,
        value_name = "COLOURING",
        value_parser = parse_colouring,
        help = "One colour for every tile, two colours by parity, or the id of a colouring that the command info --colourings lists, such as 3a. The default is parity when a side pairing of the domain reverses orientation, and one otherwise"
    )]
    pub(crate) colouring: Option<Colouring>,
    #[arg(
        long,
        help = "The fill colours, separated by commas, each as #rgb or #rrggbb, one for each colour of the colouring. The default takes the first colours of a built-in list of 12. In a POSIX shell, put the colours in single quotes, as in '#e9c46a,#2a9d8f'"
    )]
    pub(crate) colours: Option<String>,
    #[arg(
        long,
        value_name = "PIXELS",
        default_value_t = 1.0,
        help = "The border width in pixels. 0 draws no border"
    )]
    pub(crate) border_width: f64,
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
        help = "The colour outside the disk of a hyperbolic tiling or the ball of a spherical tiling, as #rgb or #rrggbb"
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
        value_name = "NAME=VALUE",
        value_parser = parse_setting,
        action = ArgAction::Append,
        help = SET_HELP,
        long_help = SET_LONG_HELP
    )]
    pub(crate) set: Vec<Setting>,
    #[arg(
        long,
        value_name = "YAW,PITCH,ROLL",
        default_value = "0,0,0",
        value_parser = parse_triple,
        allow_hyphen_values = true,
        help = "The rotation of the ball of a spherical tiling, as three angles in degrees. YAW turns the front of the ball to the right, PITCH turns it up, and ROLL turns the image counterclockwise, in this order"
    )]
    pub(crate) rotation: [f64; 3],
    #[arg(
        long,
        value_name = "X,Y,Z",
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
            .ok_or("expected one, parity, or a colouring id such as 3a"),
    }
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

    use super::{
        Colouring, SET_LONG_HELP, Setting, parse_colouring, parse_max_colours, parse_setting,
        parse_triple,
    };

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
        "*(1000)432",
        "3333",
        "22222",
        "*(1001)32",
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
                Err("expected one, parity, or a colouring id such as 3a"),
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
}
