use core::fmt;
use std::io::{self, Write};

use crate::Status;

const MAX_ECHO_CHARACTERS: usize = 120;

#[derive(Debug)]
pub(crate) struct SignatureError {
    text: String,
    error: escher_signature::Error,
}

impl SignatureError {
    pub(crate) fn new(text: &str, error: escher_signature::Error) -> Self {
        Self {
            text: text.to_owned(),
            error,
        }
    }
}

impl fmt::Display for SignatureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, formatter)
    }
}

impl core::error::Error for SignatureError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        core::error::Error::source(&self.error)
    }
}

pub(crate) fn write_stderr(stderr: &mut dyn Write, text: &str) {
    #[expect(
        clippy::let_underscore_must_use,
        reason = "a failed write to standard error has no other stream to be reported on, and the status already records the outcome"
    )]
    #[expect(
        let_underscore_drop,
        reason = "the error of a failed write to standard error is dropped on purpose, because it has no other stream to be reported on"
    )]
    let _ = stderr.write_all(text.as_bytes());
}

pub(crate) fn report(error: &anyhow::Error, stderr: &mut dyn Write) -> Status {
    if is_broken_pipe(error) {
        return Status::Failure;
    }
    let mut text = format!("error: {error}\n");
    for cause in error.chain().skip(1) {
        text.push_str("  caused by: ");
        text.push_str(&cause.to_string());
        text.push('\n');
    }
    if let Some(signature_error) = error.downcast_ref::<SignatureError>() {
        text.push_str(&location(&signature_error.text, &signature_error.error));
    }
    write_stderr(stderr, &text);
    Status::Failure
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<StdoutError>()
            .is_some_and(|stdout_error| stdout_error.0.kind() == io::ErrorKind::BrokenPipe)
    })
}

#[derive(Debug)]
pub(crate) struct StdoutError(pub(crate) io::Error);

impl fmt::Display for StdoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("cannot write to standard output")
    }
}

impl core::error::Error for StdoutError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.0)
    }
}

fn location(text: &str, error: &escher_signature::Error) -> String {
    let span = error.span();
    let characters = text.chars().count();
    if characters == 0 {
        String::new()
    } else if characters > MAX_ECHO_CHARACTERS {
        format!("  at bytes {}..{}\n", span.start, span.end)
    } else {
        let echo: String = text.chars().map(echoed).collect();
        let before = text
            .char_indices()
            .filter(|&(index, _)| index < span.start)
            .count();
        let within = text
            .char_indices()
            .filter(|&(index, _)| span.contains(&index))
            .count();
        format!("  {echo}\n  {}{}\n", " ".repeat(before), "^".repeat(within))
    }
}

fn echoed(character: char) -> char {
    if matches!(character, ' '..='~' | '\u{221E}') {
        character
    } else {
        '?'
    }
}

pub(crate) fn replace_control_characters(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() && character != '\n' {
                '?'
            } else {
                character
            }
        })
        .collect()
}
