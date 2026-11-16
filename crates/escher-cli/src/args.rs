use clap::{Parser, Subcommand};

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
        #[arg(
            help = "The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'"
        )]
        signature: String,
        #[arg(
            long,
            help = "Verify the fundamental domain of the signature and print the result of each check"
        )]
        check: bool,
    },
}
