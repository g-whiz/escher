use std::ffi::OsString;
use std::io::{self, Write};

use escher_cli::{Status, run};

pub(crate) struct Outcome {
    pub(crate) status: Status,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn run_os(args: Vec<OsString>) -> Outcome {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut all = vec![OsString::from("escher")];
    all.extend(args);
    let status = run(all, &mut stdout, &mut stderr);
    Outcome {
        status,
        stdout: String::from_utf8(stdout).expect("standard output is UTF-8"),
        stderr: String::from_utf8(stderr).expect("standard error is UTF-8"),
    }
}

pub(crate) fn run_args(args: &[&str]) -> Outcome {
    run_os(args.iter().map(OsString::from).collect())
}

pub(crate) fn snapshot(args: &[&str]) -> String {
    let outcome = run_args(args);
    format!(
        "status: {:?}\n--- stdout\n{}--- stderr\n{}--- end\n",
        outcome.status, outcome.stdout, outcome.stderr
    )
}

#[derive(Debug, Default)]
pub(crate) struct FailingWriter {
    pub(crate) write_error: Option<io::ErrorKind>,
    pub(crate) flush_error: Option<io::ErrorKind>,
    pub(crate) written: Vec<u8>,
}

impl FailingWriter {
    pub(crate) fn failing_write(kind: io::ErrorKind) -> Self {
        Self {
            write_error: Some(kind),
            ..Self::default()
        }
    }

    pub(crate) fn failing_flush(kind: io::ErrorKind) -> Self {
        Self {
            flush_error: Some(kind),
            ..Self::default()
        }
    }
}

impl Write for FailingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self.write_error {
            Some(kind) => Err(io::Error::from(kind)),
            None => self.written.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.flush_error {
            Some(kind) => Err(io::Error::from(kind)),
            None => Ok(()),
        }
    }
}
