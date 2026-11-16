//! The command-line program `escher`, which calls `escher_cli::run` with the
//! arguments, standard output, and standard error of the process.

fn main() -> std::process::ExitCode {
    let status = escher_cli::run(
        std::env::args_os(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    std::process::ExitCode::from(status)
}
