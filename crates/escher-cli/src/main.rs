//! The command-line program `escher`, which prints its package name and
//! version.

#![expect(
    clippy::print_stdout,
    reason = "the program reports its name and version on standard output"
)]

fn main() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
