use core::fmt::Display;
use std::io::Write;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use escher_domain::FundamentalDomain;
use escher_render::{Colour, Fill, RenderOptions, RenderStats, Renderer, encode_png};
use escher_signature::{Geometry, Signature};
use rayon::iter::{IndexedParallelIterator as _, ParallelIterator as _};
use rayon::slice::ParallelSliceMut as _;

use crate::args::{Colouring, RenderArgs};
use crate::diagnostic::SignatureError;

const MAX_THREADS: usize = 256;

const NOT_SUPPORTED: &str = "rendering this signature is not supported";

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
    let fill = fill(args.colouring, args.colours.as_deref())
        .map_err(|detail| cannot_render(&signature, detail))?;
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
    let domain = FundamentalDomain::build(&signature).map_err(|error| {
        let context = if error.kind() == escher_domain::ErrorKind::UnsupportedSignature {
            format!("cannot render {signature}: {NOT_SUPPORTED}")
        } else {
            format!("cannot render {signature}")
        };
        anyhow::Error::new(error).context(context)
    })?;
    if domain.geometry() != Geometry::Hyperbolic {
        return Err(cannot_render(&signature, NOT_SUPPORTED));
    }
    let options = RenderOptions::builder(fill)
        .size(args.size.width, args.size.height)
        .samples(args.samples)
        .border_width(args.border_width)
        .border_colour(border_colour)
        .background(background)
        .max_folds(args.max_folds)
        .build()
        .with_context(|| format!("cannot render {signature}"))?;
    let threads = args.threads.map_or_else(default_threads, usize::from);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|error| thread_pool_error(&signature, &error))?;
    let (width, height) = (options.width(), options.height());
    let renderer = Renderer::new(&domain, options);
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

fn cannot_render(signature: &Signature, detail: impl Display) -> anyhow::Error {
    anyhow::anyhow!("cannot render {signature}: {detail}")
}

fn fill(colouring: Colouring, colours: Option<&str>) -> Result<Fill, String> {
    let Some(text) = colours else {
        let first = Colour::from_srgb8(0xe9, 0xc4, 0x6a);
        return Ok(match colouring {
            Colouring::One => Fill::One(first),
            Colouring::Parity => Fill::Parity([first, Colour::from_srgb8(0x2a, 0x9d, 0x8f)]),
        });
    };
    let items: Vec<&str> = text.split(',').collect();
    match (colouring, items.as_slice()) {
        (Colouring::One, &[colour]) => Ok(Fill::One(fill_colour(1, colour)?)),
        (Colouring::Parity, &[first, second]) => Ok(Fill::Parity([
            fill_colour(1, first)?,
            fill_colour(2, second)?,
        ])),
        (Colouring::One | Colouring::Parity, _) => {
            let needed = match colouring {
                Colouring::One => 1,
                Colouring::Parity => 2,
            };
            Err(format!(
                "the colouring {} needs {}, and {} given",
                colouring.name(),
                count_of_colours(needed),
                given(items.len())
            ))
        }
    }
}

fn fill_colour(position: usize, text: &str) -> Result<Colour, String> {
    parse_colour(text)
        .ok_or_else(|| format!("colour {position} of --colours is not of the form #rgb or #rrggbb"))
}

fn count_of_colours(count: usize) -> String {
    if count == 1 {
        "1 colour".to_owned()
    } else {
        format!("{count} colours")
    }
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

    use escher_render::{Colour, Fill};

    use super::{fill, parse_colour, statistics_text, thread_pool_error};
    use crate::Status;
    use crate::args::Colouring;
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
    fn fill_reports_the_position_of_the_first_bad_colour() {
        assert_eq!(
            fill(Colouring::Parity, Some("red,#e9c")),
            Err("colour 1 of --colours is not of the form #rgb or #rrggbb".to_owned())
        );
        assert_eq!(
            fill(Colouring::Parity, Some("#e9c,")),
            Err("colour 2 of --colours is not of the form #rgb or #rrggbb".to_owned())
        );
    }

    #[test]
    fn fill_names_one_colour_in_the_singular() {
        assert_eq!(
            fill(Colouring::One, Some("#e9c,#2a9")),
            Err("the colouring one needs 1 colour, and 2 were given".to_owned())
        );
        assert_eq!(
            fill(Colouring::Parity, Some("#e9c")),
            Err("the colouring parity needs 2 colours, and 1 was given".to_owned())
        );
    }

    #[test]
    fn fill_reads_the_colours_in_order() {
        let Ok(Fill::Parity([first, second])) = fill(Colouring::Parity, Some("#123,#fed")) else {
            panic!("two colours give a parity fill");
        };
        assert_eq!(first.to_srgb8(), [0x11, 0x22, 0x33]);
        assert_eq!(second.to_srgb8(), [0xff, 0xee, 0xdd]);
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
}
