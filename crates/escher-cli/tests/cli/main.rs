//! Integration tests of `escher-cli` that call `run` with in-memory streams.

#[cfg(test)]
mod common;
mod diagnostics;
mod info;
mod usage;

fn assert_auto_traits<T: Send + Sync>() {}

#[test]
fn status_is_send_and_sync() {
    assert_auto_traits::<escher_cli::Status>();
}

#[test]
fn status_converts_to_exit_codes_zero_one_and_two() {
    use std::process::ExitCode;

    use escher_cli::Status;

    assert_eq!(ExitCode::from(Status::Success), ExitCode::from(0));
    assert_eq!(ExitCode::from(Status::Failure), ExitCode::from(1));
    assert_eq!(ExitCode::from(Status::Usage), ExitCode::from(2));
}
