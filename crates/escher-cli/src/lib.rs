//! Runs the `escher` command-line program on a list of arguments and two
//! output streams, and returns the outcome as a [`Status`].
//!
//! The binary `escher` is a thin wrapper around [`run`], which it calls with
//! the arguments of the process, standard output, and standard error. The
//! crate `escher-signature` parses and classifies an orbifold signature, a
//! short text such as `*632` that names a symmetry group of the plane, the
//! sphere, or the hyperbolic plane. This crate turns those results into text,
//! diagnostics, and an exit status. Use [`run`] to call the program from Rust,
//! such as in a test, without starting a process.
//!
//! The command `escher info <SIGNATURE>` prints the canonical form of the
//! signature, its geometry, its Euler characteristic, and the values that
//! depend on them. A malformed signature produces a diagnostic on standard
//! error that locates the error in the signature.
//!
//! ```
//! use escher_cli::{Status, run};
//!
//! let mut stdout = Vec::new();
//! let mut stderr = Vec::new();
//! let status = run(["escher", "info", "*236"], &mut stdout, &mut stderr);
//! assert_eq!(status, Status::Success);
//! assert_eq!(
//!     String::from_utf8(stdout)?,
//!     "Signature: *632\nGeometry: Euclidean\nEuler characteristic: 0\nCompact: yes\n",
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

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use anyhow::Context as _;
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
    /// The signature is not valid, or a write to standard output failed.
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
/// diagnostics to `stderr`, and to no other stream. `args` holds the program
/// name first, as [`std::env::args_os`] gives it. The function flushes
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
    let result = match cli.command {
        Command::Info { signature } => info::info(&signature, stdout),
    };
    match result {
        Ok(()) => Status::Success,
        Err(error) => diagnostic::report(&error, stderr),
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

fn write_stdout(stdout: &mut dyn Write, text: &str) -> anyhow::Result<()> {
    stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
        .context("cannot write to standard output")
}
