use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use escher_group::IndexLimit;

const SIGNATURE_HELP: &str = "The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'";

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
    Render(RenderArgs),
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
        value_enum,
        help = "One colour for every tile, or two colours by parity. The default is parity when a side pairing of the domain reverses orientation, and one otherwise"
    )]
    pub(crate) colouring: Option<Colouring>,
    #[arg(
        long,
        help = "The fill colours, separated by commas, each as #rgb or #rrggbb. In a POSIX shell, put the colours in single quotes, as in '#e9c46a,#2a9d8f'"
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
        help = "Set a shape parameter, as NAME=VALUE, such as aspect=1.5. The command info lists the parameters of a signature. The option can be given once for each parameter"
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum Colouring {
    One,
    Parity,
}

impl Colouring {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Parity => "parity",
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

    use super::{Setting, parse_max_colours, parse_setting, parse_triple};

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
