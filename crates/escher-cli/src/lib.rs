//! Runs the `escher` command-line program on a list of arguments and two
//! output streams, and returns the outcome as a [`Status`].
//!
//! The binary `escher` is a thin wrapper around [`run`], which it calls with
//! the arguments of the process, standard output, and standard error. The
//! crate `escher-signature` parses and classifies an orbifold signature, a
//! short text such as `*632` that names a symmetry group of the Euclidean
//! plane, the sphere, or the hyperbolic plane. The crate `escher-domain`
//! builds and verifies the fundamental domain of a signature, a region whose
//! copies under the group tile the plane, the sphere, or the hyperbolic plane,
//! and the crate `escher-render` draws the tiling of a domain. This crate
//! turns those results into text, an image file, diagnostics, and an exit
//! status. Use [`run`] to call the program from Rust, such as in a test,
//! without starting a process.
//!
//! The command `escher info <SIGNATURE>` prints the canonical form of the
//! signature, its geometry, its Euler characteristic, the values that depend
//! on them, and the shape parameters of the signature with their ranges and
//! default values. It then prints a summary of the fundamental domain, or
//! `none` and the reason when the signature has no domain. A domain whose
//! sides are not all mirrors, the sides where the group reflects, such as the
//! domain of `732` or of `**`, has sides that the side pairings join in pairs.
//! The summary then lists the mirror sides, if the domain has any, and the
//! pairs of sides. When no side pairing reverses orientation, as for `732` and
//! `o`, the summary also states that the domain has no colouring by the parity
//! of reflections. For a domain, the command also
//! counts the colourings of the tiles that the symmetry respects, one for each
//! subgroup of finite index, for each number of colours up to the limit that
//! the option `--max-colours` sets, 12 by default. With the option
//! `--colourings`, it lists each colouring with its id, such as `3a`. A search
//! that stops at its limit on work or memory says up to which number of
//! colours it is complete. A malformed signature produces a
//! diagnostic on standard error that locates the error in the signature. With
//! the option `--check`, the command also verifies the domain, prints the
//! result of each check, and fails when a check fails or the signature has no
//! domain. With the option `--presentation`, it prints the generators and the
//! relators of a presentation of the group read from the signature, and of
//! one read from the sides of the domain. For a spherical signature, it also
//! counts the elements of the group of each presentation by coset
//! enumeration, and fails when a count differs from the group order.
//!
//! The command `escher render --output <FILE> <SIGNATURE>` draws the tiling of
//! the fundamental domain in the Poincare disk, a picture of the whole
//! hyperbolic plane inside a disk, for a hyperbolic domain, in a window of the
//! plane for a Euclidean domain, and on a lit ball for a spherical domain. The
//! options `--rotation`, `--light`, and `--shading` set the viewing rotation,
//! the light direction, and the shading of the ball. The option
//! `--set NAME=VALUE` sets a shape parameter of the domain, such as
//! `aspect=1.5` for `*2222`. The command draws on a pool of threads that it
//! builds for the call. It writes the PNG file only after the rendering and
//! the encoding succeed, and it replaces a file that exists. The option
//! `--colouring` takes `one`, `parity`, or a colouring id that
//! `escher info --colourings` lists, such as `3a`. Without it, the command
//! colours the tiles by the parity of reflections when a side pairing of the
//! domain reverses orientation, and in one colour otherwise, as for the domain
//! of `732`. For an id, it gives each tile the colour of its coset in that
//! colouring. The option `--colours` gives one colour for each colour of the
//! colouring, and without it the command takes the first colours of a
//! built-in list of 12. It fails for a signature without a domain, for a
//! setting that the domain does not accept, for the colouring `parity` of a
//! domain whose side pairings all preserve orientation, for an id that names
//! no colouring of the domain or whose search stops at a limit, for an id
//! with more than 12 colours without `--colours`, and for a number of colours
//! that differs from that of the colouring. It prints nothing on standard
//! output unless the option `--stats` is given, which prints statistics of the
//! rendering after the file is written.
//!
//! ```
//! use escher_cli::{Status, run};
//!
//! let mut stdout = Vec::new();
//! let mut stderr = Vec::new();
//! let status = run(["escher", "info", "3*2"], &mut stdout, &mut stderr);
//! assert_eq!(status, Status::Success);
//! assert_eq!(
//!     String::from_utf8(stdout)?,
//!     "Signature: 3*2\nGeometry: spherical\nEuler characteristic: 1/12\nCompact: yes\n\
//!      Group order: 24\nArea: pi/6\n\
//!      Domain: none, cannot build fundamental domain: the signature 3*2 is not supported\n",
//! );
//! assert!(stderr.is_empty());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! This crate has no Cargo features.
//!
//! # Platform support
//!
//! The crate is supported on Linux. It holds no platform-specific code.

mod args;
mod diagnostic;
mod info;
mod render;

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use clap::Parser as _;

use crate::args::{Cli, Command};

/// The outcome of one call of [`run`], which the binary turns into the exit
/// status of the process.
///
/// Convert a status to an exit status with [`ExitCode::from`]. `Success`
/// gives 0, `Failure` gives 1, and `Usage` gives 2. The enum is
/// `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use std::process::ExitCode;
///
/// use escher_cli::{Status, run};
///
/// let status = run(["escher", "info", "23"], &mut Vec::new(), &mut Vec::new());
/// assert_eq!(status, Status::Failure);
/// assert_eq!(ExitCode::from(status), ExitCode::from(1));
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// The command wrote its output, or it printed the help or the version.
    Success,
    /// The signature is not valid, a write to standard output failed, the
    /// option `--check` found a failed check or no domain to verify, the
    /// option `--presentation` counted a group order that differs from that of
    /// the signature, or the command `render` could not render the signature
    /// or write its file.
    Failure,
    /// The command line itself is wrong, such as an unknown subcommand or a
    /// missing argument.
    Usage,
}

/// Maps `Success` to 0, `Failure` to 1, and `Usage` to 2.
impl From<Status> for ExitCode {
    fn from(status: Status) -> Self {
        match status {
            Status::Success => Self::SUCCESS,
            Status::Failure => Self::FAILURE,
            Status::Usage => Self::from(2),
        }
    }
}

/// Runs the `escher` program in the current process and returns its
/// [`Status`].
///
/// Use this to run the program with streams that the caller chooses, such as a
/// `Vec<u8>` in a test. The function writes the output to `stdout` and the
/// diagnostics to `stderr`, and to no other stream. The command `render` also
/// writes its output file. `args` holds the program name first, as
/// [`std::env::args_os`] gives it. The function flushes
/// `stdout` after it writes to it, and it never panics or exits the process. A
/// failed write to `stdout` gives [`Status::Failure`], with a diagnostic on
/// `stderr` unless the reader closed the pipe. A failed write to `stderr` is
/// ignored. In the help, version, and usage text that the argument parser
/// produces, the function replaces each control character other than the line
/// feed with `?`.
///
/// # Examples
///
/// ```
/// use escher_cli::{Status, run};
///
/// let mut stdout = Vec::new();
/// let mut stderr = Vec::new();
/// let status = run(["escher", "info", "*7q2"], &mut stdout, &mut stderr);
/// assert_eq!(status, Status::Failure);
/// assert!(stdout.is_empty());
/// assert_eq!(
///     String::from_utf8(stderr)?,
///     "error: cannot parse orbifold signature: unexpected character 'q'\n  *7q2\n    ^\n",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn run<I, T>(args: I, stdout: &mut impl Write, stderr: &mut impl Write) -> Status
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    run_args(args.into_iter().map(Into::into).collect(), stdout, stderr)
}

fn run_args(args: Vec<OsString>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Status {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => return usage(&error, stdout, stderr),
    };
    let errors = match cli.command {
        Command::Info {
            signature,
            check,
            presentation,
            colourings,
            max_colours,
        } => info::info(
            &signature,
            info::Options {
                verify: info::Verify::from_flag(check),
                presentations: info::Presentations::from_flag(presentation),
                listing: info::Listing::from_flag(colourings),
                max_colours,
            },
            stdout,
        ),
        Command::Render(render_args) => render::render(&render_args, stdout)
            .err()
            .into_iter()
            .collect(),
    };
    if errors.is_empty() {
        Status::Success
    } else {
        for error in &errors {
            diagnostic::report(error, stderr);
        }
        Status::Failure
    }
}

fn usage(error: &clap::Error, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Status {
    let text = diagnostic::replace_control_characters(&error.render().to_string());
    if error.use_stderr() {
        diagnostic::write_stderr(stderr, &text);
        Status::Usage
    } else {
        match write_stdout(stdout, &text) {
            Ok(()) => Status::Success,
            Err(write_error) => diagnostic::report(&write_error, stderr),
        }
    }
}

fn count_of_colours(count: u64) -> String {
    if count == 1 {
        "1 colour".to_owned()
    } else {
        format!("{count} colours")
    }
}

#[cfg(test)]
fn colouring_id(text: &str) -> escher_group::ColouringId {
    text.parse().expect("the id is valid")
}

fn write_stdout(stdout: &mut dyn Write, text: &str) -> anyhow::Result<()> {
    stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|error| anyhow::Error::new(diagnostic::StdoutError(error)))
}
