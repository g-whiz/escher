use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    },
    #[command(
        about = "Render the tiling of a signature in orbifold notation in the Poincare disk and write it to a PNG file"
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
        default_value_t = Colouring::Parity,
        help = "One colour for every tile, or two colours by parity"
    )]
    pub(crate) colouring: Colouring,
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
        help = "The colour outside the disk, as #rgb or #rrggbb"
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

fn parse_size(text: &str) -> Result<Size, &'static str> {
    let error = "expected two decimal integers joined by x, such as 512x512";
    let (width, height) = text.split_once('x').ok_or(error)?;
    Ok(Size {
        width: decimal(width).ok_or(error)?,
        height: decimal(height).ok_or(error)?,
    })
}

fn decimal(text: &str) -> Option<u32> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}
