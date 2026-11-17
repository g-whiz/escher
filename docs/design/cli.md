# Command-Line Interface Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-17 |
| Applies to | `crates/escher-cli`: the library target, the binary `escher`, and its tests |

## Abstract

This document specifies the package `escher-cli`, which provides the command-line program `escher`. It defines the public surface of the library target, the thin binary that calls it, the commands and options, the exact text that the command `info` prints for each kind of signature, including the shape parameters, the summary of the fundamental domain, and the result of each check of the option `--check`, the options of the command `render`, including the settings of shape parameters, the PNG file that it writes and the statistics that it prints, the format of every diagnostic, the exit statuses, the tests, and the dependencies.

## 1. Introduction

### 1.1. Problem

The proof of concept is a command-line tool, and its first command reports what a signature in orbifold notation describes. The tool reads untrusted text, prints text that a reviewer checks line by line, and must report every malformed signature with a pointer to the offending characters. A program whose logic lives in `main` can only be tested by starting a process, which is slow and cannot run under Miri.

### 1.2. Purpose

This document fixes the behaviour of `escher` precisely enough that an agent can implement it and a reviewer can check every line of output against it. The crate `escher-signature` parses and classifies a signature, the crate `escher-domain` builds and verifies its fundamental domain, and the crate `escher-render` renders the domain into pixels and encodes them as PNG. This crate turns their results into text, a file, diagnostics, and an exit status.

### 1.3. Audience

The primary readers are the coding agents that implement or extend `escher-cli`. The secondary readers are reviewers and the owner, who decides the points of Section 11.3.

### 1.4. Scope

This document covers the command `info` with its argument and its option `--check`, the command `render` with its argument and its options, the options `--help` and `--version`, the diagnostics, the exit statuses, the tests of the crate, and its dependencies. `info` parses, validates, canonicalizes, and classifies any signature. It prints the shape parameters of the signature. For a signature that `escher-domain` constructs, it also prints a summary of the fundamental domain, and with `--check` the result of each check of its verification. It prints no item that needs a group.

`render` draws, with `escher-render`, the tiling of a signature that `escher-domain` constructs with the shape parameters that the option `--set` gives. It draws the tiling in the Poincare disk for a hyperbolic domain, in the Euclidean plane for a Euclidean one, and as a lit ball for a spherical one, with the viewing rotation, the light, and the shading of its options, and writes it to a PNG file. With `--stats`, it also prints statistics of the rendering.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The grammar of a signature, its canonical form, the classification methods, and the error kinds of `escher-signature` with their `Display` text are specified in `docs/design/signature.md`.
- The signatures that have a fundamental domain, its construction, its measurements, the checks of its verification with their tolerances, the shape parameters with their names, ranges, and defaults, and the error kinds of `escher-domain` with their `Display` text are specified in `docs/design/domain.md`.
- The rendering, the limits on the render options, the statistics that a rendering counts, the PNG encoding, and the error kinds of `escher-render` with their `Display` text are specified in `docs/design/render.md`.
- The crates, the data flow, the error model, and the owner's decisions are specified in `docs/design/architecture.md`.
- The mathematics is stated in `docs/research/orbifold-notation.md` and, for rendering, in `docs/research/rendering.md`.
- The recipes, the nextest profiles, the hooks, and the dependency policy are specified in `docs/design/tooling.md`.

The lines of `info` that need a group or a colouring belong to later milestones (Section 5.4). So do the options of `render` that later milestones add, such as a colouring id in M8 and `--model` in M12 (Section 3.4 of `docs/design/poc-plan.md`).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md` and of Section 2.1 of `docs/design/architecture.md`, such as signature, canonical signature, geometry, fundamental domain, and general non-compact signature. It adds these terms:

- ON: The reference `docs/research/orbifold-notation.md`, cited as `ON` and a section number, such as ON 7.3.
- command: A subcommand of `escher`, such as `info` or `render`.
- information text: The text that `escher info` writes to standard output for a valid signature (Section 5).
- domain: The `FundamentalDomain` that `FundamentalDomain::build` or `FundamentalDomain::build_with` of `escher-domain` returns for a signature.
- shape parameter: A named number that changes the shape of the domain of a signature, which `parameters` of `escher-domain` lists with its range and its default, such as `aspect` of `*2222`.
- setting: One value of the option `--set` of `render`, a parameter name and a number joined by `=` (Section 4.6).
- parameter lines: The lines of the information text that list the shape parameters, row 10 of Section 5.1.
- domain summary: The lines of the information text that describe the domain, rows 11 to 14 of Section 5.1 (Section 5.5).
- check: One numeric condition that `verify()` of `escher-domain` evaluates, with a residual and a tolerance (Section 2.1 of `docs/design/domain.md`).
- verification lines: The lines of the information text that `--check` adds, rows 15 to 21 of Section 5.1 (Section 5.6).
- output file: The file that the option `--output` of `render` names (Section 4.7).
- colour: A value of the form `#rgb` or `#rrggbb` that `render` reads for a fill, the border, or the background (Section 4.6).
- thread pool: The `rayon` thread pool that `render` builds for one call of `run` (Section 4.5).
- statistics lines: The lines that `render --stats` writes to standard output (Section 4.7).
- render error: A failure of `render` after `clap` has parsed the command line, other than a signature error and an output error (Section 6.8).
- diagnostic: The text that `escher` writes to standard error when it fails (Section 6).
- echo line: The line of a diagnostic that repeats the signature (Section 6.2).
- caret line: The line of a diagnostic that marks the span of an error with `^` characters under the echo line (Section 6.3).
- signature error: An error of `escher-signature` for an argument that is not a valid signature, including a bad orbifold.
- usage error: An error that `clap` reports for the command line itself, such as an unknown subcommand or a missing argument.
- output error: A failed write to standard output or standard error.
- broken pipe: The output error whose `std::io::ErrorKind` is `BrokenPipe`, which a write returns when the reader of a pipe has closed it.
- status: The value of the type `Status` that `run` returns, which the binary converts to the exit status of the process.
- snapshot: A file of expected output that an `insta` test compares with the actual output.
- control character: A character of the Unicode general category Cc, such as the escape character U+001B or the line feed.

## 3. Package Structure

### 3.1. Public Surface

The package has a library target, `crates/escher-cli/src/lib.rs`, and the binary `escher`, `crates/escher-cli/src/main.rs`. The library target exposes exactly `run` and `Status`, with the conversion that `Status` implements. This listing shows them with their signatures, and bodies are omitted:

```rust
#[must_use]
pub fn run<I, T>(
    args: I,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> Status
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString>;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Success,
    Failure,
    Usage,
}

impl From<Status> for std::process::ExitCode {}
```

CLI-1. The library target MUST expose `run` and `Status` and no other public item, because every further public item becomes an interface that tests and later crates can come to depend on, while the command line is the only interface that this crate promises. The `clap` argument types, the information text, and the diagnostics are private.

The binary needs a library target so that tests can run every command in the test process, with in-memory streams, and observe its status and output. A test of a binary alone has to start a process, which is slow and fails under Miri (Section 6 of `docs/design/architecture.md`).

### 3.2. The `run` Function

`args` holds the program name first, as `std::env::args_os()` gives it. `run` is a thin generic wrapper that collects `args` into a `Vec<std::ffi::OsString>` and calls a private non-generic function that takes `&mut dyn std::io::Write` for each stream, as rule API-13 of `docs/rust-coding-standards.md` requires.

CLI-2. `run` MUST write only to the two streams it is given and to the output file of `render`, and MUST NOT panic on any input or any output error, or else a test of `run` cannot observe all output, and one malformed argument or closed pipe aborts the program instead of giving a status.

`run` returns no error. Each command returns `anyhow::Result<()>` inside the crate, and one private function writes the diagnostic of Section 6 and chooses the status. No `anyhow::Error` therefore appears in a public signature, which rule ERR-2 of `docs/rust-coding-standards.md` forbids.

Every path of `run` that writes to `stdout` does so through one private helper, which writes the whole text and then flushes `stdout`, so that a write error that buffering delays still reaches the status. A path that writes nothing to `stdout` does not flush it. The helper wraps its `std::io::Error` in a private error type whose `Display` text is `cannot write to standard output` and whose source is that error. The diagnostic code looks for this type to tell a broken pipe on standard output from one on the output file (Section 6.6).

### 3.3. Status

`Status` maps to these exit statuses through `From<Status> for std::process::ExitCode`. Section 7 states when `run` returns each value.

| `Status` | Exit status |
| --- | --- |
| `Success` | 0 |
| `Failure` | 1 |
| `Usage` | 2 |

`run` returns `Status` and not `std::process::ExitCode`, because `Status` names each outcome, so a test asserts which outcome occurred by name, and the mapping from outcomes to exit numbers is stated in one place, the table of this section. `Status` is `#[non_exhaustive]`, so a later milestone can add a value without a breaking change.

### 3.4. Binary

The binary holds a crate-level `//!` line and this function, and nothing else:

```rust
fn main() -> std::process::ExitCode {
    let status = escher_cli::run(
        std::env::args_os(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    std::process::ExitCode::from(status)
}
```

CLI-3. `main.rs` MUST hold only the call of `run` and the conversion of its status, because code in the binary runs only in a test that starts the binary as a process, and this crate has one such test, the smoke test of Section 8.1.

The binary prints nothing itself, so `main.rs` carries no lint exception for `clippy::print_stdout`, which would fail as an unfulfilled expectation.

### 3.5. Private Modules

The library target has the private modules `args`, `info`, `render`, and `diagnostic`, as files named `args.rs`, `info.rs`, `render.rs`, and `diagnostic.rs` (Section 4.5 of `docs/rust-coding-standards.md`). `args` holds the `clap` types and the parsers of `--size`, `--set`, and the triples of `--rotation` and `--light`, `info` builds the information text, including the domain summary and the verification lines, `render` runs the steps of Section 4.5 with the parser of colours and the statistics lines, and `diagnostic` writes the diagnostics.

## 4. Command Line

### 4.1. Synopsis

```text
escher info [--check] <SIGNATURE>
escher render [OPTIONS] --output <FILE> <SIGNATURE>
escher --help
escher --version
```

`-h` is the short form of `--help` and `-V` of `--version`. `escher info --help` prints the help of `info`, and `escher render --help` the help of `render`. `--check` has no short form. `-o` is the short form of `--output`, and no other option of `render` has a short form. The usage line in the help of `info` is the form that `clap` generates, `escher info [OPTIONS] <SIGNATURE>`, and that of `render` is `escher render [OPTIONS] --output <FILE> <SIGNATURE>`.

### 4.2. Parser Settings

The command line is parsed by `clap` with the derive API, which Section 3.1 of `docs/design/architecture.md` names. The argument types are plain structs and enums (rule SIMP-4 of `docs/rust-coding-standards.md`). `run` calls `try_parse_from`, so `clap` never exits the process itself.

These settings differ from the defaults of `clap`:

| Setting | Value | Reason |
| --- | --- | --- |
| `bin_name` | `escher` | The usage line names the binary, whatever path the program name in `args` has |
| `version` | The package version | Enables `--version` and `-V` |
| `arg_required_else_help` | `true` | `escher` with no argument is a usage error that prints the help, instead of printing nothing |
| `disable_help_subcommand` | `true` | Help has one form, the option `--help`, so the surface to test is smaller |
| Cargo feature `color` | Off | The output goes to the given streams as plain text |

The command name stays the package name, `escher-cli`, so that `--version` prints the package name.

The help text has these summaries, and no other text beyond what `clap` generates:

| Item | Summary |
| --- | --- |
| `escher` | The package description, `Command-line program of Escher, a tool for symmetry-based graphics.` |
| `info` | `Print the canonical form, the geometry, and other properties of a signature in orbifold notation` |
| `SIGNATURE` | `The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'` |
| `--check` | `Verify the fundamental domain of the signature and print the result of each check` |
| `render` | `Render the tiling of a signature in orbifold notation and write it to a PNG file` |
| `--output` | `The PNG file to write. An existing file is replaced` |
| `--size` | `The image size in pixels, as WIDTHxHEIGHT` |
| `--colouring` | `One colour for every tile, or two colours by parity` |
| `--colours` | `The fill colours, separated by commas, each as #rgb or #rrggbb. In a POSIX shell, put the colours in single quotes, as in '#e9c46a,#2a9d8f'` |
| `--border-width` | `The border width in pixels. 0 draws no border` |
| `--border-colour` | `The border colour, as #rgb or #rrggbb` |
| `--samples` | `The number of samples along each side of a pixel. The pixel averages N by N samples` |
| `--max-folds` | `The largest number of folds of one sample. A sample that needs more takes the border colour` |
| `--background` | `The colour outside the disk of a hyperbolic tiling or the ball of a spherical tiling, as #rgb or #rrggbb` |
| `--threads` | `The number of threads, from 1 to 256. The default is the available parallelism, at most 256` |
| `--stats` | `Print statistics of the rendering to standard output` |
| `--set` | `Set a shape parameter, as NAME=VALUE, such as aspect=1.5. The command info lists the parameters of a signature. The option can be given once for each parameter` |
| `--rotation` | `The rotation of the ball of a spherical tiling, as three angles in degrees. YAW turns the front of the ball to the right, PITCH turns it up, and ROLL turns the image counterclockwise, in this order` |
| `--light` | `The direction from the ball of a spherical tiling to the light, as X,Y,Z, with X to the right, Y up, and Z toward the viewer. The length is ignored` |
| `--shading` | `Phong shading of the ball of a spherical tiling, or none` |

The help of `render` uses the summary of `SIGNATURE` above. It also shows `[default: <value>]` for each option with a fixed default in the table of Section 4.6, and `[possible values: one, parity]` for `--colouring` and `[possible values: phong, none]` for `--shading`, which `clap` generates (Section 11.1). The argument struct declares `--rotation`, `--light`, and `--shading` after `--set`, so they come last in the help.

CLI-4. `escher --version` and `escher -V` MUST write the package name, one space, the package version, and a newline to standard output, which is `escher-cli 0.1.0` for version 0.1.0, and return `Success`, because Section 5.1 of `docs/design/poc-plan.md` keeps this output, and a script may read it.

### 4.3. The `info` Command

`info` takes one required positional argument, the signature, as a `String` named `SIGNATURE`, and the optional flag `--check`. An argument that is not valid UTF-8 is a usage error. The flag may stand before or after the signature. A repeated `--check` is a usage error, because `clap` rejects a flag that is given more than once, and `info` keeps that behaviour.

CLI-5. `info` and `render` MUST pass their argument unchanged to the `FromStr` implementation of `Signature` from `escher-signature`, and MUST NOT trim, normalize, or check it first, because a second reading of the grammar diverges from the one that `docs/design/signature.md` specifies, and the byte spans of the error then no longer match the argument.

On success, `info` writes the information text of Section 5 to standard output. On a signature error, it writes the diagnostic of Section 6 to standard error and nothing to standard output, with or without `--check`. A bad orbifold is a signature error (`DEC-24`).

`info` takes no setting. It builds the domain with `FundamentalDomain::build`, which gives each shape parameter its default, so the domain summary and the verification lines describe the default shape.

With `--check`, `info` also runs `verify()` on the domain, adds the verification lines of Section 5.6 to the information text, and writes the diagnostic of Section 6.7 when a check fails or the signature has no domain.

The `clap` argument struct holds the flag as a `bool`, because the derive API sets a flag through a `bool` field. `info` converts it at once into a private enum with two variants, which the private functions take (TYPE-2 of `docs/rust-coding-standards.md`).

A signature never starts with `-`, so it never collides with an option. An argument that starts with `-` is read as an option, and an unknown option is a usage error.

### 4.4. Shell Quoting

In a POSIX shell, `*`, `(`, and `)` are special characters, so a signature that contains any of them is quoted in single quotes, as in `escher info '2*(12)'`. The shell passes every character inside single quotes unchanged. The help text of `info` and `render` says this, and every example in this document that needs it quotes the signature.

A word that starts with `#` starts a comment in a POSIX shell, so a colour given as a separate argument is also quoted, as in `--border-colour '#000000'`. The form `--border-colour=#000000` needs no quotes, because there the `#` does not start a word. The help text of `--colours` says this, and every example in this document quotes its colours.

### 4.5. The `render` Command

`render` takes one required positional argument, the signature, as a `String` named `SIGNATURE`, the required option `--output`, and the options of Section 4.6. An argument that is not valid UTF-8 is a usage error, except the value of `--output`, which is any path. The options may stand before or after the signature. An option given more than once is a usage error (Section 11.1), except `--set`, which is the one option that may be given more than once, once for each shape parameter.

This command writes a 512 by 512 image of `*732` with the defaults of Section 4.6:

```sh
escher render '*732' -o tiling.png
```

This command writes an image of `*2222` whose rectangle has the aspect 1.618 in place of the default square:

```sh
escher render '*2222' --set aspect=1.618 -o rectangle.png
```

This command writes an image of the ball of `*532`, turned 20 degrees to the right and 35 degrees up:

```sh
escher render '*532' --rotation 20,35,0 -o ball.png
```

`render` runs these steps in this order. It stops at the first step that fails and writes the diagnostic of Section 6.8, except at step 1, whose diagnostic is that of Section 6.1:

1. Parse the signature with the `FromStr` implementation of `Signature` (CLI-5).
2. Read the fill colours, the border colour, and the background colour (Section 4.6).
3. Put the settings into a `ShapeParameters` of `escher-domain` in the order given, and stop at the first setting whose name an earlier setting already gave. Build the domain with `FundamentalDomain::build_with` and those settings, which checks each name and value.
4. Build the render options of `escher-render` from the size, the sample grid, the fill colours, the border width, the border colour, the background colour, the fold cap, the viewing rotation in radians, the light direction, and the shading. `escher-render` checks every value, for every geometry.
5. Build a thread pool with the number of threads of `--threads`, or with the default of Section 4.6, which this step computes.
6. Render the image on the thread pool, in bands of one row, each with the band operation of `escher-render`, and merge the statistics of the bands. The time of this step is the time of the statistics lines.
7. Encode the image as PNG with `escher-render`.
8. Write the PNG bytes to the output file (Section 4.7).
9. With `--stats`, write the statistics lines to standard output (Section 4.7).

Section 3 of `docs/design/render.md` names the operations of steps 4, 6, and 7 and the statistics that step 6 merges.

CLI-21. `render` MUST NOT create, truncate, or otherwise change the output file before rendering and encoding have succeeded, because a failed `render` then leaves an existing file as it was and creates no empty file that a reader could take for an image.

CLI-22. `render` MUST build its own thread pool for each call, with exactly the number of threads of step 5, and MUST NOT use the global pool of `rayon`, because the global pool is shared by the whole process and is configured once, so two calls of `run` in one test process could not choose their own numbers, and the size of the pool would come from the environment instead of the option.

CLI-23. The bytes of the output file MUST NOT depend on the number of threads, because a user who changes `--threads` for speed expects the same image, and a difference reveals a defect in the split into bands. Section 3.2 of `docs/design/render.md` states that a band renders the same bytes as the same rows of the whole image.

CLI-29. `render` MUST pass every setting to `escher-domain` through `FundamentalDomain::build_with`, and MUST NOT check a parameter name or a value against the parameters of the signature itself, except that it reports a name that two settings give, because a second check in this crate can differ from the specifications that `docs/design/domain.md` states, and a setting would then pass one check and fail the other.

CLI-30. Withdrawn on 2026-11-17.

The `clap` argument struct holds `--stats` as a `bool`, which `render` converts at once into a private enum with two variants, as `info` does for `--check` (Section 4.3).

### 4.6. Option Values

`clap` checks the syntax of every option value, and a malformed value is a usage error (Section 6.5). `render` reads the colours itself in step 2, `escher-domain` checks the settings in step 3, and `escher-render` checks the other values in step 4. This table gives the value syntax and the default of each option of `render`:

| Option | Value | Default |
| --- | --- | --- |
| `-o`, `--output <FILE>` | Any path | None. The option is required (`DEC-7`) |
| `--size <WIDTHxHEIGHT>` | Two decimal integers joined by a lower-case `x`, such as `1024x768`. Each integer has only the digits `0` to `9` and fits a `u32` | `512x512` |
| `--colouring <COLOURING>` | `one` or `parity` | `parity` (`DEC-7`) |
| `--colours <COLOURS>` | Colours separated by commas, as many as the colouring needs: one for `one` and two for `parity` | `#e9c46a` for `one`, and `#e9c46a,#2a9d8f` for `parity` (`DEC-7`) |
| `--border-width <PIXELS>` | A number in a form that the `FromStr` implementation of `f64` reads, such as `1.5`. The value `0` draws no border | `1` (`DEC-7`) |
| `--border-colour <COLOUR>` | One colour | `#000000` (`DEC-7`) |
| `--samples <N>` | A decimal integer that fits a `u32`. The grid of samples in each pixel is N by N | `4` (`DEC-7`) |
| `--max-folds <N>` | A decimal integer that fits a `u32` | `1500` (`DEC-7`) |
| `--background <COLOUR>` | One colour | `#ffffff` (`DEC-7`) |
| `--threads <N>` | A decimal integer from 1 to 256 | The value of `std::thread::available_parallelism`, or 1 when that function returns an error, and at most 256 |
| `--stats` | None, a flag | Off |
| `--set <NAME=VALUE>` | A parameter name and a number joined by `=`. The name is the text before the first `=` and is not empty. The number is the text after it, in a form that the `FromStr` implementation of `f64` reads. The option may be given more than once | None. Each shape parameter has the default that `escher info` lists |
| `--rotation <YAW,PITCH,ROLL>` | A triple: three numbers in degrees | `0,0,0` (`DEC-16`) |
| `--light <X,Y,Z>` | A triple: three numbers | `-1,1,2` (`DEC-16`) |
| `--shading <SHADING>` | `phong` or `none` | `phong` (`DEC-16`) |

`clap` parses `--samples` and `--max-folds` with its parser of `u32`, `--threads` with its parser of a range of `u16` from 1 to 256, whose value converts to `usize` without a fallible conversion, and `--border-width` with its parser of `f64`. A private parser in `args` reads `--size`. For a value of another form it returns the message `expected two decimal integers joined by x, such as 512x512`, which `clap` puts into its usage error. A second private parser in `args` reads each value of `--set`: it splits the value at the first `=`, requires a name that is not empty, and reads the rest with the `FromStr` implementation of `f64`, as `--border-width` is read. For a value of another form, such as `aspect`, `=2`, or `aspect=x`, it returns the message `expected a parameter name and a number joined by =, such as aspect=1.5`. `clap` collects the values of `--set` in the order given, with its action `Append`. A NaN or infinite number reads as a number, and `escher-domain` rejects it as outside the range of its parameter. The value of `--output` is a `std::path::PathBuf`, so a path that is not valid UTF-8 is accepted.

A triple is exactly three items separated by commas, each read by the `FromStr` implementation of `f64`, with nothing trimmed. One private parser in `args` reads the triples of `--rotation` and `--light`. For a value of another form, such as `1,2`, `a,b,c`, `1,2,3,4`, or `1,,2`, it returns the message `expected three numbers separated by commas, such as 20,35,0`. A NaN or infinite number reads as a number, and `escher-render` rejects it. `render` converts each angle of `--rotation` from degrees to radians with `f64::to_radians`. `--shading` is a value enum of `clap`, and the value `none` gives `Shading::Flat` of `escher-render`.

`--rotation` and `--light` set the `clap` setting `allow_hyphen_values`, so that a value that starts with `-` is read as the value, as in `--light -1,1,2` and `--rotation -20,35,0`. The form with `=`, as in `--light=-1,1,2`, works with or without the setting. The cost is that an option name directly after either option is read as its value: `--rotation --stats` reads `--stats` as the triple, which is a usage error, and `--rotation -h` reads `-h` as the triple instead of printing the help.

CLI-31. `--rotation` and `--light` MUST accept a value that starts with `-`, because the default light direction and many rotations have a negative number first, and without the setting `clap` rejects `--light -1,1,2` as an unknown argument.

`render` reads the three options for every signature. For a hyperbolic or Euclidean signature `escher-render` checks them and they have no effect, as `--background` has no effect on a Euclidean tiling.

A colour is `#` followed by exactly three or exactly six hexadecimal digits. The digits are ASCII `0` to `9` and `a` to `f` in either case. In the form `#rrggbb`, each pair of digits is one 8-bit sRGB channel, in the order red, green, and blue. In the form `#rgb`, each digit stands for the channel whose two digits both equal it, so `#e9c` is `#ee99cc`. Nothing is trimmed, so a colour with a space before or after it is not a colour. Every colour is opaque, and `render` has no transparent background (`DEC-7`). The colours are read by a private parser in the module `render`, with no dependency (`DEC-29`).

Step 2 splits the value of `--colours` at each comma. It first compares the number of items with the number that the colouring needs, and then reads each item as a colour, in order. An empty item, such as the one that a trailing comma gives, is not a colour. It then reads `--border-colour` and `--background`.

`escher-render` limits the image size, the number of pixels, the sample grid, the border width, and the fold cap, and returns an error kind for a value outside its limits, such as a width of 0, a grid of 0, a border width that is negative or not finite, a rotation angle that is not finite, or a light direction that is zero or not finite. Section 4.1 of `docs/design/render.md` states each limit and the error kind for a value outside it. `render` reports such an error as Section 6.8 states.

CLI-24. `render` MUST pass the size, the sample grid, the border width, the fold cap, the components of the light direction, and the angles of the viewing rotation after their conversion to radians to `escher-render` unchanged, and MUST NOT check them against a limit itself, because a second check in this crate can differ from the limits that `docs/design/render.md` states, and a value would then pass one check and fail the other.

The limit of 256 threads is this crate's own, because the thread pool belongs to this crate (Section 3.5 of `docs/design/architecture.md`). The default never exceeds it.

### 4.7. Output of `render`

The output file holds the PNG bytes that `escher-render` encodes in step 7, with the size of `--size`. Section 5.6 of `docs/design/render.md` states the colour type, the bit depth, and the chunks. Step 8 writes the bytes with `std::fs::write`, which creates the file when it does not exist and truncates and replaces it when it does. `render` replaces an existing file without asking and has no option to refuse it (`DEC-30`). It creates no directory.

Without `--stats`, a successful `render` writes nothing to standard output or standard error. With `--stats`, step 9 writes the lines of this table, in this order, through the helper of Section 3.2. Each line is the label, a colon, one space, and the value, followed by a line feed, as in Section 5.1. Every count is an integer of the statistics of `escher-render`, which Section 3.2 of `docs/design/render.md` states, and is written in decimal:

| Order | Label | Value |
| --- | --- | --- |
| 1 | `Samples` | The number of samples, including the outside samples of Section 2.1 of `docs/design/render.md` |
| 2 | `Mean folds` | The total number of folds divided by the number of inside samples, which are the samples that are not outside samples, rounded down to two digits after the point, or `0.00` when every sample is an outside sample |
| 3 | `Maximum folds` | The largest number of folds of one sample |
| 4 | `Unconverged samples` | The number of samples that reached the fold cap or met a value that is not finite |
| 5 | `Time` | The wall-clock time of step 6 in seconds, measured with `std::time::Instant`, in fixed notation with three digits after the point, `{:.3}`, followed by one space and `s` |

`Mean folds` is computed in `u128` arithmetic, so it cannot overflow. The quotient $q$ of 100 times the folds by the inside samples, rounded down, is written as the integer part of $q/100$, a point, and the remainder of $q$ by 100 in two digits. For example, 1001 folds over 100 inside samples give `10.01`.

This illustration shows the form of the statistics of the default image. `Samples` is $512 \times 512 \times 16 = 4194304$, and the placeholders in angle brackets stand for the values that vary:

```sh
escher render '*732' --stats -o tiling.png
```

```text
Samples: 4194304
Mean folds: <mean>
Maximum folds: <count>
Unconverged samples: <count>
Time: <seconds> s
```

Standard error is empty and the status is `Success`.

CLI-25. `render` MUST write nothing to standard output without `--stats`, and with `--stats` only the statistics lines, after the output file is written, because a script that captures standard output must not mistake other text for statistics, and statistics written before the write would describe an image that may not exist.

## 5. Information Text

### 5.1. Lines

`info` writes the lines of this table, in this order. Each line is the label, a colon, one space, and the value, followed by a line feed. No other line, blank line, or trailing space appears. The accessors of rows 1 to 9 are those of `Signature` (Sections 3.1 and 9.1 of `docs/design/signature.md`). Row 10 takes its values from `parameters` and `ParameterSpec`, and rows 11 to 21 from `FundamentalDomain`, `Verification`, and `Check` (Section 3.1 of `docs/design/domain.md`), and "a domain is built" means that `FundamentalDomain::build` returned `Ok`.

| Order | Label | Printed when | Value |
| --- | --- | --- | --- |
| 1 | `Signature` | Always | The `Display` text of `Signature`, which is the canonical signature |
| 2 | `Geometry` | Always | `spherical`, `Euclidean`, or `hyperbolic`, from `geometry()` |
| 3 | `Euler characteristic` | Always | `euler_characteristic()`, in the form of Section 5.2 |
| 4 | `Compact` | Always | `yes` or `no`, from `is_compact()` |
| 5 | `Frieze` | `is_compact()` is false | `yes` or `no`, from `is_frieze()` |
| 6 | `General non-compact` | `is_compact()` is false | `yes` or `no`, from `is_general_non_compact()` |
| 7 | `Group order` | `group_order()` returns a value, which it does for a spherical signature | The integer in decimal |
| 8 | `Area` | `area_over_pi()` returns a value, which it does for a spherical or hyperbolic signature | The form of Section 5.2 |
| 9 | `Deformation dimension` | `deformation_dimension()` returns a value, which it does for a compact hyperbolic signature | The integer in decimal |
| 10 | `Parameter` | Once for each item that `parameters` returns for the signature, in its order | The name, `, range `, the minimum, ` to `, the maximum, `, default `, and the default value, with the numbers in the form of Section 5.2, such as `aspect, range 0.125 to 8, default 1` |
| 11 | `Domain` | Always | `<n> mirror sides` when a domain is built, and otherwise `none, ` followed by the `Display` text of the error (Section 5.5) |
| 12 | `Corner angles in degrees` | A domain is built | Each value of `corner_angles()` in degrees, in corner order, in the form of Section 5.2 |
| 13 | `Side lengths` | A domain is built | Each value of `side_lengths()`, in side order, in the form of Section 5.2 |
| 14 | `Domain area` | A domain is built | `area()`, in the form of Section 5.2 |
| 15 | `Check isometries` | `--check` is given and a domain is built | The result of `isometries()`, in the form of Section 5.6 |
| 16 | `Check pairings` | As row 15 | The result of `pairings()` |
| 17 | `Check walls` | As row 15 | The result of `walls()` |
| 18 | `Check relators` | As row 15 | The result of `relators()` |
| 19 | `Check angles` | As row 15 | The result of `angles()` |
| 20 | `Check area` | As row 15 | The result of `area()` of `Verification` |
| 21 | `Verification` | As row 15 | `passed` when `Verification::passed()` is true, and otherwise `failed` |

CLI-6. `info` MUST print exactly the lines of this table, in this order and under these conditions, because the snapshots of Section 8 and every reader of the text rely on one layout, and an extra or missing line changes what a reader concludes about the signature. The table has no name of the group in another notation, such as an IUC or Schoenflies name, because `DEC-20` omits them until a primary source is read.

So a signature without a domain gets the lines of rows 1 to 10 that apply to it and then exactly one more line, the `Domain` line of row 11, with or without `--check`. Rows 15 to 21 follow the order of the checks of Section 8.2 of `docs/design/domain.md`.

### 5.2. Number Formats

Every number is written in ASCII decimal digits with no separator.

The Euler characteristic is the `Display` text of `Rational` from `escher-signature`: a reduced fraction `n/d` with a positive denominator, or the integer `n` when the denominator is 1. A negative value starts with `-`, as in `-1/84`. Zero is `0`.

The area is $2\pi \lvert \chi \rvert$ (ON 7.3). `area_over_pi()` gives the positive rational $n/d$ by which $\pi$ is multiplied, and `info` writes it as this table states:

| Numerator $n$ | Denominator $d$ | Text | Example |
| --- | --- | --- | --- |
| 1 | 1 | `pi` | None among the examples |
| 1 | Greater than 1 | `pi/d` | `pi/42` |
| Greater than 1 | 1 | `npi` | `4pi` |
| Greater than 1 | Greater than 1 | `npi/d` | `2pi/3` |

The floating-point values of the domain summary and the verification lines have these forms:

| Value | Rows | Form | Example |
| --- | --- | --- | --- |
| Minimum, maximum, and default of a parameter | 10 | The `Display` text of `f64`, `{}`, which writes the shortest decimal that reads back as the same value, with no exponent | `0.125`, `8`, `1` |
| Corner angle | 12 | The value of `corner_angles()` in radians converted with `f64::to_degrees`, then written in fixed notation with six digits after the point, `{:.6}` | `25.714286` |
| Side length | 13 | Fixed notation with six digits after the point, `{:.6}` | `0.545275` |
| Domain area | 14 | Fixed notation with six digits after the point, `{:.6}` | `0.074800` |
| Residual and tolerance | 15 to 20 | Scientific notation with one digit after the point, `{:.1e}` of Rust, which writes an infinite value as `inf` | `2.5e-3`, `1.0e-9` |

Rows 12 and 13 separate the values with a comma and one space. Angles are in degrees on the command line, as Section 5.5 of `docs/design/architecture.md` states.

### 5.3. Examples

The examples follow the canonical form that `docs/design/signature.md` defines. Each value cites the section of ON that gives it or its formula. The deformation dimension applies the formula of ON 10.1 with the reading of the Euler characteristic of the underlying surface that Section 15.1 of `docs/design/signature.md` records. The domain summary and the `Domain` line follow Section 5.5, and the verification lines Section 5.6.

This example shows a compact hyperbolic signature with a domain:

```sh
escher info '*732'
```

```text
Signature: *732
Geometry: hyperbolic
Euler characteristic: -1/84
Compact: yes
Area: pi/42
Deformation dimension: 0
Domain: 3 mirror sides
Corner angles in degrees: 25.714286, 60.000000, 90.000000
Side lengths: 0.545275, 0.620672, 0.283128
Domain area: 0.074800
```

The Euler characteristic comes from the defect rule of ON 6.1, the area from ON 7.3, and the dimension from ON 10.1. Section 5.1 of `docs/design/poc-plan.md` states all three values. The corner angles are $180/7$, 60, and 90 degrees, in the corner order of Section 5.1 of `docs/design/domain.md`. A 256-bit computation of the exact triangle gave the side lengths 0.5452748318, 0.6206717376, and 0.2831281534 and the area $\pi/42 = 0.0747998251$, so the printed values are correct to the digits shown. `Domain area` is the area of the triangle measured from its angles, and `Area` is the exact value from the Euler characteristic, so the two agree to the digits shown.

This example shows a spherical signature with a domain:

```sh
escher info '*532'
```

```text
Signature: *532
Geometry: spherical
Euler characteristic: 1/60
Compact: yes
Group order: 120
Area: pi/30
Domain: 3 mirror sides
Corner angles in degrees: 36.000000, 60.000000, 90.000000
Side lengths: 0.553574, 0.652358, 0.364864
Domain area: 0.104720
```

ON 6.4 gives the Euler characteristic $1/60$ and the order 120, which the table of ON 8.2 lists. The area comes from ON 7.3. The domain is the spherical triangle with the angles $\pi/5$, $\pi/3$, and $\pi/2$, in the corner order of the domain design. A 256-bit computation of the exact triangle on a prototype outside the repository gave the side lengths 0.5535743589, 0.6523581398, and 0.3648638281, in side order, and the area $\pi/30 = 0.1047197551$, so the printed values are correct to the digits shown. The snapshot of Section 8.1 fixes every printed value of the other spherical inputs. A spherical signature has no `Deformation dimension` line, and no `Parameter` line, because no spherical signature has a shape parameter.

This example shows a spherical signature without a domain:

```sh
escher info 532
```

```text
Signature: 532
Geometry: spherical
Euler characteristic: 1/30
Compact: yes
Group order: 60
Area: pi/15
Domain: none, cannot build fundamental domain: the signature 532 is not supported
```

ON 6.4 gives the Euler characteristic $1/30$, and the order 60 is $2/\chi$ (ON 7.3), which also gives the area. The signature needs no quotes, because it holds none of the special characters of Section 4.4.

This example shows a hyperbolic signature with a cone point and a corner:

```sh
escher info '4*3'
```

```text
Signature: 4*3
Geometry: hyperbolic
Euler characteristic: -1/12
Compact: yes
Area: pi/6
Deformation dimension: 0
Domain: none, cannot build fundamental domain: the signature 4*3 is not supported
```

ON 6.4 gives the Euler characteristic. The area comes from ON 7.3 and the dimension from ON 10.1.

This example shows a signature with one deformation parameter:

```sh
escher info '*3333'
```

```text
Signature: *3333
Geometry: hyperbolic
Euler characteristic: -1/3
Compact: yes
Area: 2pi/3
Deformation dimension: 1
Domain: none, cannot build fundamental domain: the signature *3333 is not supported
```

ON 6.4 gives the Euler characteristic, and ON 7.3 states the area $2\pi/3$ for this signature. The dimension comes from ON 10.1, and Section 5.1 of `docs/design/poc-plan.md` states it.

This example shows an order of 10 or more, written in parentheses:

```sh
escher info '2*(12)'
```

```text
Signature: 2*(12)
Geometry: spherical
Euler characteristic: 1/24
Compact: yes
Group order: 48
Area: pi/12
Domain: none, cannot build fundamental domain: the signature 2*(12) is not supported
```

The order $4n = 48$ of `2*n` comes from ON 8.2. The Euler characteristic is $2/48 = 1/24$ by ON 7.3, which also gives the area.

This example shows a Euclidean signature with a domain:

```sh
escher info '*632'
```

```text
Signature: *632
Geometry: Euclidean
Euler characteristic: 0
Compact: yes
Domain: 3 mirror sides
Corner angles in degrees: 30.000000, 60.000000, 90.000000
Side lengths: 1.000000, 1.154701, 0.577350
Domain area: 0.288675
```

ON 8.1 lists `*632` among the 17 Euclidean signatures. A Euclidean signature has no `Area` line, because `area_over_pi()` returns no value for it, and no `Deformation dimension` line. The domain is the triangle with the angles $\pi/6$, $\pi/3$, and $\pi/2$, scaled to the unit of `DEC-17`, in which side 0 has length 1. The angles, the length of side 0, and the area follow from the construction of Section 6.5 of `docs/design/domain.md`. The snapshot of Section 8.1 fixes the lengths of sides 1 and 2.

This example shows the Euclidean signature with a shape parameter:

```sh
escher info '*2222'
```

```text
Signature: *2222
Geometry: Euclidean
Euler characteristic: 0
Compact: yes
Parameter: aspect, range 0.125 to 8, default 1
Domain: 4 mirror sides
Corner angles in degrees: 90.000000, 90.000000, 90.000000, 90.000000
Side lengths: 1.000000, 1.000000, 1.000000, 1.000000
Domain area: 1.000000
```

The domain is the square of the default `aspect`, with area 1 in the unit of `DEC-17`. `info` takes no setting (Section 4.3), so it prints the square for every invocation.

These two commands show a frieze, with the infinite order in its two spellings. They print the same text:

```sh
escher info '*22(inf)'
escher info '*22∞'
```

```text
Signature: *(inf)22
Geometry: Euclidean
Euler characteristic: 0
Compact: no
Frieze: yes
General non-compact: no
Domain: none, cannot build fundamental domain: the signature *(inf)22 is not supported
```

ON 6.4 gives the Euler characteristic 0, ON 8.3 lists the signature among the friezes, and ON 7.1 states that a signature with an infinite order is not compact. An infinite order sorts above every finite order in the canonical form, so the canonical signature is `*(inf)22` (Section 11.3).

This example shows a general non-compact signature:

```sh
escher info '3*(inf)'
```

```text
Signature: 3*(inf)
Geometry: hyperbolic
Euler characteristic: -1/6
Compact: no
Frieze: no
General non-compact: yes
Area: pi/3
Domain: none, cannot build fundamental domain: the signature 3*(inf) is not supported
```

The Euler characteristic comes from the defect rule of ON 6.1, including the defect of a corner of infinite order. The geometry follows `DEC-10`, and the geometry and the area rest on `UNV-21` (Section 10). No deformation dimension is printed, because none is known for a non-compact signature (ON 12 item 15).

This example shows the trivial signature:

```sh
escher info 1
```

```text
Signature: 1
Geometry: spherical
Euler characteristic: 2
Compact: yes
Group order: 1
Area: 4pi
Domain: none, cannot build fundamental domain: the signature 1 is not supported
```

The Euler characteristic 2 is the defect rule of ON 6.1 with no defect. The order is that of `nn` with $n = 1$ in ON 8.2, and the area comes from ON 7.3.

For a supported signature whose largest corner order is above the limit of `escher-domain`, such as `*(1001)32`, rows 1 to 9 apply as for `*732`, and the last line is `Domain: none, cannot build fundamental domain: order 1001 is larger than 1000`, as Section 10.2 of `docs/design/domain.md` fixes it.

The option `--check` adds the verification lines after the domain summary. This example shows the form, with `<residual>` in place of each residual, because Section 8.1 replaces the residuals of these lines in the snapshot:

```sh
escher info --check '*732'
```

```text
Signature: *732
Geometry: hyperbolic
Euler characteristic: -1/84
Compact: yes
Area: pi/42
Deformation dimension: 0
Domain: 3 mirror sides
Corner angles in degrees: 25.714286, 60.000000, 90.000000
Side lengths: 0.545275, 0.620672, 0.283128
Domain area: 0.074800
Check isometries: passed, residual <residual>, tolerance 1.0e-9
Check pairings: passed, residual <residual>, tolerance 1.0e-9
Check walls: passed, residual <residual>, tolerance 1.0e-9
Check relators: passed, residual <residual>, tolerance 2.0e-6
Check angles: passed, residual <residual>, tolerance 1.0e-9
Check area: passed, residual <residual>, tolerance 1.0e-9
Verification: passed
```

Standard error is empty and the status is `Success`. The tolerances are those of Section 8.4 of `docs/design/domain.md`.

With `--check`, a signature without a domain prints the same information text as without it, and then the diagnostic of Section 6.7:

```sh
escher info --check 532
```

Standard output:

```text
Signature: 532
Geometry: spherical
Euler characteristic: 1/30
Compact: yes
Group order: 60
Area: pi/15
Domain: none, cannot build fundamental domain: the signature 532 is not supported
```

Standard error:

```text
error: cannot verify fundamental domain
  caused by: cannot build fundamental domain: the signature 532 is not supported
```

The status is `Failure`, so the exit status is 1.

### 5.4. Stability and Omissions

The information text is not a stable interface, and a machine-readable form is a roadmap item (`DEC-23`).

CLI-7. A change that alters the information text MUST update its snapshots in the same change, because the snapshot test fails otherwise, and the snapshot diff is how a reviewer sees each change to the text (`DEC-23`, rule TEST-2 of `docs/rust-coding-standards.md`).

`info` prints no line about presentations, colourings, polygons, or colours, and no placeholder for them. Each needs a crate or a part of a crate that a later milestone adds. Section 3.4 of `docs/design/poc-plan.md` places them: the presentation in M7 (Section 5.7), and the list of colourings with their colour counts in M8 (Section 5.8). The owner's goal that `info` reports how many polygons the representative set has, and so which colour counts are possible, is met by that list in M8. The domain summary covers exactly the signatures that `escher-domain` constructs (Section 4 of `docs/design/domain.md`).

### 5.5. Domain Summary

For every valid signature, `info` calls `parameters` once and prints the parameter lines of row 10, then calls `FundamentalDomain::build` once and prints the `Domain` line of row 11. Rows 12 to 14 follow only when a domain is built.

When a domain is built, the value of the `Domain` line is the number of `sides()` in decimal, one space, and `mirror sides`. Every side of a domain that `escher-domain` builds has the kind `Mirror` (Section 3.2 of `docs/design/domain.md`).

When `build` returns an error, the value is `none, ` followed by the `Display` text of that error, for every error kind. The `Display` text starts with `cannot build fundamental domain: ` (Section 10.2 of `docs/design/domain.md`), so the line reads, for example, `Domain: none, cannot build fundamental domain: the signature 532 is not supported`. The line holds only the `Display` text of the top error and no text of its source. The text holds the canonical signature or a number, both in printable ASCII, so it needs no replacement of characters.

CLI-16. `info` MUST take the values of rows 10 to 21 from `escher-domain` unchanged, except for the conversion of the angles to degrees and the formats of Section 5.2, and MUST NOT measure or check the domain itself, because a second computation in this crate can differ from what `verify()` checks, and the printed values would then describe another domain than the one that passed.

CLI-17. Without `--check`, `info` MUST print the `Domain` line with `none` and return `Success` for a valid signature for which `build` returns an error, because the signature is still parsed, valid, and classified, and a non-zero status would tell a script that the signature itself is invalid.

The values of rows 12 to 14 are rounded to six digits after the point. Sections 9.3, 9.4, and 9.5 of `docs/design/domain.md` bound the error of each value that `build` returns. A printed digit can therefore differ between two correct builds, such as builds on two platforms, only when the exact value lies within that bound of a rounding boundary of the sixth digit. Section 8.1 states how the snapshot inputs avoid that case.

### 5.6. Verification Lines

With `--check`, when a domain is built, `info` calls `verify()` once and prints rows 15 to 21 after the domain summary. The information text and the verification lines form one text, which `info` writes in one write.

The value of each of rows 15 to 20 has this form, where the result is `passed` when `Check::passed()` is true and `failed` otherwise, and the residual and the tolerance have the form of Section 5.2:

```text
<result>, residual <residual>, tolerance <tolerance>
```

For example, a failed relator check prints `Check relators: failed, residual 2.5e-3, tolerance 2.0e-6`. A residual of infinity, which marks a failed structural condition or a computation that gave no finite number (Section 8.1 of `docs/design/domain.md`), prints as `inf`. No residual is NaN.

Row 21 gives the overall result, `Verification: passed` or `Verification: failed`, from `Verification::passed()`.

CLI-18. With `--check`, `info` MUST print one line for each check that `Verification` reports, with its result, its residual, and its tolerance, in the order of Section 8.2 of `docs/design/domain.md`, and then the overall result, because a reader who sees only the overall result cannot tell which condition failed or by how much, and a line that omits the tolerance cannot be read without the design document.

## 6. Diagnostics

### 6.1. Format

CLI-8. `run` MUST write every diagnostic to standard error and none to standard output, because a caller that captures standard output must receive only the information text or the statistics lines, or else an error is read as data.

The diagnostic of a signature error, an output error, a verification error of Section 6.7, or a render error of Section 6.8 has this form, and Section 6.5 covers a usage error:

```text
error: <Display text of the error>
  caused by: <Display text of each further error in the chain>
<echo line>
<caret line>
```

The first line is `error: ` followed by the `Display` text of the top error. Each further error in the `anyhow` chain adds a line that starts with two spaces and `caused by: ` (Section 5.8 of `docs/design/architecture.md`). A signature error has no source, so it has no such line.

For a signature error, a private wrapper holds the text of the argument beside the error of `escher-signature`. Its `Display` text is that of the inner error, so no text repeats. The text after `error: ` is therefore the `Display` text of Section 10.2 of `docs/design/signature.md`, which starts with `cannot parse orbifold signature: `.

The echo line and the caret line are printed only for a signature error, under these conditions:

| Argument | Lines after the first |
| --- | --- |
| Empty | None |
| 1 to 120 characters | The echo line and the caret line |
| More than 120 characters | The line `  at bytes <start>..<end>`, with the byte span in decimal |

### 6.2. Echo of the Signature

The echo line is two spaces followed by the argument, in which each character outside the printable ASCII range `0x20` to `0x7E` is replaced by `?`, except the infinity sign U+221E, which is echoed as it is.

CLI-9. The diagnostic MUST replace every character of the argument outside printable ASCII and the infinity sign with `?`, and MUST NOT echo more than 120 characters, because the argument is untrusted, and an echoed control character can send an escape sequence to the user's terminal.

### 6.3. Caret Line

The caret line is two spaces, then one space for each character of the argument before the span, then one `^` for each character in the span.

The span that `escher-signature` returns is a range `start..end` of bytes on character boundaries. The caret line counts characters, not bytes. The number of spaces after the indent is the number of characters in the argument before byte `start`, and the number of `^` is the number of characters in `start..end`. The infinity sign, which is three bytes in UTF-8, therefore takes one column. Every echoed character takes one column, because the replacement of Section 6.2 turns every other character into `?`. The caret sits under the characters of the span, with the assumption of Section 11.1 about the width of the infinity sign. The caret line is drawn by hand (Section 5.8 of `docs/design/architecture.md`).

### 6.4. Examples

The text after `error: ` in these examples is the `Display` text of `escher-signature`. `docs/design/signature.md` defines it and the byte spans (Section 5.5 of that document) and prevails where the two documents differ. The snapshots follow it.

This example shows a bad orbifold, whose span is the whole signature, bytes 0 to 2 (`DEC-24`, ON 7.2):

```sh
escher info 23
```

```text
error: cannot parse orbifold signature: bad orbifold, no symmetry group has this signature
  23
  ^^
```

This example shows an unexpected character, with the span of bytes 2 to 3:

```sh
escher info '*7q2'
```

```text
error: cannot parse orbifold signature: unexpected character 'q'
  *7q2
    ^
```

This example shows the caret after the infinity sign, which takes one column although it is three bytes. The span of `q` is bytes 6 to 7, and four characters precede it:

```sh
escher info '*22∞q'
```

```text
error: cannot parse orbifold signature: unexpected character 'q'
  *22∞q
      ^
```

This example shows a digit `1` without parentheses that is directly followed by another digit (reading rule 5 of Section 4.2 of `docs/design/signature.md`). The span is the two digits, bytes 2 to 4:

```sh
escher info '2*12'
```

```text
error: cannot parse orbifold signature: a digit 1 followed by another digit is ambiguous, write an order of 10 or more in parentheses, such as (12)
  2*12
    ^^
```

The same group written with parentheses is the spherical group of order 48 of Section 5.3:

```sh
escher info '2*(12)'
```

### 6.5. Usage Errors

On an error of `try_parse_from`, `run` takes the text of `error.render()` as plain text and replaces each control character in it other than the line feed with `?`. When `error.use_stderr()` is false, which is the case for `--help` and `--version`, the text goes to standard output. Otherwise it goes to standard error. A usage error covers an unknown subcommand or option, a missing signature, a repeated `--check` or another repeated option other than `--set`, an argument that is not UTF-8, `escher` with no argument, a missing `--output`, and an option value of `render` whose syntax Section 4.6 does not allow, such as a malformed triple or an unknown shading. Section 7 gives the status of each case, and a failed write of the help or the version text follows Section 6.6.

`clap` copies an unknown argument into its error message with its control characters unchanged. Without the replacement, an argument that starts with `--` and holds an escape sequence reaches standard error unchanged.

CLI-15. `run` MUST replace every control character other than the line feed with `?` in the help, version, and usage text that `clap` produces, before it writes that text, because `clap` echoes an untrusted argument unchanged, and an echoed control character can send an escape sequence to the user's terminal.

The help text and the version text hold no control character other than the line feed, so the replacement leaves them unchanged.

### 6.6. Output Errors

CLI-10. `run` MUST return `Failure` when a write to standard output fails, and MUST NOT write a diagnostic when that failure is a broken pipe on standard output, because a status of 0 would report output that never arrived, while a diagnostic for a closed pipe, as in `escher info '*632' | head -c0`, reports the reader's choice as an error.

CLI-10 covers every write to standard output, including the help and the version text of Section 6.5. A failed flush counts as a failed write. Only a broken pipe on standard output is silent. A broken pipe on the output file of `render`, such as a FIFO or `-o /dev/stdout` whose reader has closed, is an ordinary failed write of that file and gives the diagnostic of Section 6.8 with `Failure`.

When a write to standard output fails in any other way than a broken pipe, `run` writes this diagnostic to standard error:

```text
error: cannot write to standard output
  caused by: <Display text of the std::io::Error>
```

A failed write to standard error is discarded in the form of rule ERR-5 of `docs/rust-coding-standards.md`, and the status stays what it would have been.

This is a choice of this design. It is the simplest behaviour that never panics: Rust ignores the signal `SIGPIPE`, so a write to a closed pipe returns an error, and the `println!` macro panics on that error.

### 6.7. Verification Errors

With `--check`, `info` writes the information text first and then one of these diagnostics:

| Case | Diagnostic |
| --- | --- |
| Every check passes | None |
| At least one check fails | `error: cannot verify fundamental domain: <k> of 6 checks failed`, with `k` the number of checks whose `passed()` is false, in decimal |
| `build` returns an error | `error: cannot verify fundamental domain`, then `  caused by: ` and the `Display` text of the error of `escher-domain`, then one `caused by` line for each further source in its chain, as Section 6.1 states |

For example, a domain with one failed check gives `error: cannot verify fundamental domain: 1 of 6 checks failed`. A `Numerical` error of `escher-domain` with an `escher-geometry` error as its source (Section 10.1 of `docs/design/domain.md`) gives a third line with the `Display` text of the `escher-geometry` error. Neither diagnostic has an echo line or a caret line.

When the write of the information text fails, Section 6.6 applies, the status is `Failure`, and no verification diagnostic is written.

### 6.8. Render Errors

A signature error of `render` gives the diagnostic of Sections 6.1 to 6.3, which is the same as that of `info` for the same argument. Every other failure of the steps of Section 4.5 gives one of the diagnostics of this table. In the table, `*732` and the placeholder `<signature>` stand for the canonical signature, which is the `Display` text of `Signature`. A further `caused by` line follows for each further source in the chain of an error, as Section 6.1 states:

| Case | Step | Diagnostic |
| --- | --- | --- |
| `--colours` has the wrong number of items | 2 | `error: cannot render *732: the colouring parity needs 2 colours, and 3 were given`, with the colouring, the number it needs, and the number of items. The words are `1 colour` and `1 was given` for the number 1 |
| An item of `--colours` is not a colour | 2 | `error: cannot render *732: colour 2 of --colours is not of the form #rgb or #rrggbb`, with the position of the first such item, counted from 1 |
| `--border-colour` is not a colour | 2 | `error: cannot render *732: --border-colour is not of the form #rgb or #rrggbb` |
| `--background` is not a colour | 2 | `error: cannot render *732: --background is not of the form #rgb or #rrggbb` |
| Two settings give the same name | 3 | `error: cannot render *2222: setting 2 of --set names the same parameter as setting 1`, with the position of the first setting whose name an earlier setting gave, and the position of that earlier setting, counted from 1 |
| `build_with` returns `UnsupportedSignature` | 3 | `error: cannot render 532: rendering this signature is not supported`, then `  caused by: ` and the `Display` text of the error of `escher-domain` |
| `build_with` returns any other error kind, such as an unknown parameter name or a value outside the range of its parameter | 3 | `error: cannot render *(1001)32`, then `  caused by: ` and the `Display` text of the error of `escher-domain` |
| `escher-render` rejects an option, including a rotation angle or a light direction | 4 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render` |
| The thread pool cannot be built | 5 | `error: cannot render *732`, then `  caused by: cannot start the thread pool`, then `  caused by: ` and the `Display` text of the error of `rayon`. That text appears once, because the diagnostic takes it without the source of the error |
| `escher-render` returns an error while rendering or encoding | 6, 7 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render` |
| The output file cannot be written | 8 | `error: cannot write the output file`, then `  caused by: ` and the `Display` text of the `std::io::Error` |

Section 7.2 of `docs/design/render.md` fixes the `Display` text of each error of `escher-render`, and Section 10.2 of `docs/design/domain.md` that of `escher-domain`. No render error has an echo line or a caret line.

The row of `UnsupportedSignature` meets the requirement of Section 5.8 of `docs/design/architecture.md` that `render` names the signature and says that rendering it is not supported. It applies to every valid signature without a domain, including every general non-compact signature (`DEC-19`). The row of any other error kind covers a corner order over the limit of `escher-domain`, a numerically degenerate construction, an unknown parameter name, and a value outside the range of its parameter, which are not a lack of support. `escher-domain` checks the support of the signature before the settings, so a setting of a signature without a domain gives the row of `UnsupportedSignature` (DOM-18 in Section 6.6 of `docs/design/domain.md`).

CLI-26. A diagnostic of `render` MUST NOT repeat the value of an option or the output path, because both are untrusted and may hold control characters, while the canonical signature, the option names, and the numbers that the diagnostics hold are printable ASCII. A bad colour is therefore named by its option and its position, and a failed write by the words `the output file`.

This example shows a signature without a domain:

```sh
escher render 532 -o tiling.png
```

```text
error: cannot render 532: rendering this signature is not supported
  caused by: cannot build fundamental domain: the signature 532 is not supported
```

This example shows a rotation angle that is not finite. `--rotation` reads `nan` as a number, and `escher-render` rejects it, for a signature of any geometry:

```sh
escher render '*532' --rotation nan,0,0 -o tiling.png
```

```text
error: cannot render *532
  caused by: cannot build render options: the rotation angle NaN is not finite
```

This example shows a light direction of zero length:

```sh
escher render '*532' --light 0,0,0 -o tiling.png
```

```text
error: cannot render *532
  caused by: cannot build render options: the light direction is zero or not finite
```

Section 7.2 of `docs/design/render.md` fixes the text after `caused by: ` in these two examples, and prevails where the two documents differ.

This example shows a parameter set twice:

```sh
escher render '*2222' --set aspect=2 --set aspect=3 -o tiling.png
```

```text
error: cannot render *2222: setting 2 of --set names the same parameter as setting 1
```

This example shows a name that is not a parameter of the signature:

```sh
escher render '*632' --set aspect=2 -o tiling.png
```

```text
error: cannot render *632
  caused by: cannot build fundamental domain: the signature *632 has no shape parameter with one of the given names
```

This example shows a value outside the range of its parameter:

```sh
escher render '*2222' --set aspect=9 -o tiling.png
```

```text
error: cannot render *2222
  caused by: cannot build fundamental domain: the parameter aspect is outside its range of 0.125 to 8
```

Section 10.2 of `docs/design/domain.md` fixes the text after `caused by: ` in the last two examples, and prevails where the two documents differ. Neither text echoes what the user typed, because the name in the last text is the name of the specification in `escher-domain`, and the numbers are those of its range (CLI-26).

This example shows three colours for the colouring `parity`:

```sh
escher render '*732' --colours '#e9c46a,#2a9d8f,#264653' -o tiling.png
```

```text
error: cannot render *732: the colouring parity needs 2 colours, and 3 were given
```

This example shows a colour name, which is not a colour of Section 4.6:

```sh
escher render '*732' --colours '#e9c46a,red' -o tiling.png
```

```text
error: cannot render *732: colour 2 of --colours is not of the form #rgb or #rrggbb
```

This example shows an order over the limit of `escher-domain`:

```sh
escher render '*(1001)32' -o tiling.png
```

```text
error: cannot render *(1001)32
  caused by: cannot build fundamental domain: order 1001 is larger than 1000
```

This example shows an output path in a directory that does not exist, on Linux, after a successful rendering:

```sh
escher render '*732' -o /tmp/missing/tiling.png
```

```text
error: cannot write the output file
  caused by: No such file or directory (os error 2)
```

Each of these diagnostics gives the status `Failure`. A failure before step 8 creates and changes no file (CLI-21). A write that fails after `std::fs::write` has opened the file can leave it truncated or partly written (Section 12). When `--stats` is given and the write of the statistics lines fails, Section 6.6 applies, and the output file stays written.

## 7. Streams and Exit Status

`run` returns these statuses. Section 3.3 gives their exit statuses, and Sections 4.7, 6.5, 6.6, 6.7, and 6.8 give the streams.

| Status | When |
| --- | --- |
| `Success` | `info` without `--check` wrote its text, whether or not the signature has a domain. `info --check` wrote its text, and every check passed. `render` wrote the output file, and with `--stats` the statistics lines. `--help` or `--version` wrote the help or the version text |
| `Failure` | The argument is a signature error of any kind, including a bad orbifold (`DEC-24`). A write to standard output failed (Section 6.6). With `--check`, a check failed or the signature has no domain (Section 6.7). `render` met a render error (Section 6.8), including a signature without a domain and a bad setting |
| `Usage` | `clap` reports a usage error (Section 6.5) |

CLI-11. `run` MUST return `Failure` for every signature error, including a bad orbifold, because a caller that gets 0 for an invalid signature treats the missing information as valid (`DEC-24`). A caller that needs the error kind uses `escher-signature` directly.

CLI-19. `info --check` MUST return `Failure` when a check fails and when the signature has no domain, and MUST return `Success` only when every check passed, because a script that runs `--check` reads the exit status as the verdict, and a status of 0 for a signature with nothing to check reads as verified. Section 5.8 of `docs/design/architecture.md` gives the same reason for `render`.

## 8. Testing

### 8.1. Tests

The tests live in one integration-test binary, `crates/escher-cli/tests/cli/main.rs`, with the modules `info`, `diagnostics`, `usage`, `render`, `process`, and `common`, and the snapshots under `crates/escher-cli/tests/cli/snapshots/` (rule TEST-10 of `docs/rust-coding-standards.md`). `main.rs` has a `//!` line, which the lint `missing_docs` requires. It declares `common`, `render`, and `process` under `#[cfg(test)]`, because their helpers call `expect`, which the Clippy option `allow-expect-in-tests` accepts only in code compiled for tests. Every test outside `main.rs` and `process` calls `run` with `Vec<u8>` streams, or with a test writer that fails. The two tests in `main.rs` test `Status` alone. The one test in `process` starts the binary.

A test that can write the output file gets a path from a helper in `common`, in a directory of its own, `CARGO_TARGET_TMPDIR/escher-cli/<test>`. The helper empties that directory first, so a file from an earlier run cannot satisfy a test. The render usage snapshots and the render status tests fail before any file access and pass the fixed path `CARGO_TARGET_TMPDIR/escher-cli/never-written.png`, which a constant in `common` holds and no test creates.

The column "Plan item" names the item of the field "Tests and acceptance" of Section 5.1, 5.3, 5.4, or 5.5 of `docs/design/poc-plan.md` that each test serves, or the field "Command line" of its Section 5.2, 5.3, 5.4, or 5.5. The tables, the enumeration, and the properties of the field of Section 5.1 belong to `escher-signature`, and the reference arithmetic of Section 5.2 belongs to `escher-geometry` and `escher-domain`. The fold properties, the reference tests, the border oracle, the symmetric view, the golden images, and the decoding of the pixels of the PNG file of Section 5.3 belong to `escher-render`.

| Test | Plan item | Kind and module | Asserts | Under Miri |
| --- | --- | --- | --- | --- |
| Information snapshots | Snapshots | `insta`, `info` | The text and status for `*732`, `*532`, `532`, `4*3`, `*3333`, `2*(12)`, `*22(inf)`, `*632`, `1`, `3*(inf)`, `*(inf)32`, `ox`, `*236`, and `2*`, whose area is exactly `pi`. By Section 7 of `docs/design/signature.md`, `*22(inf)` prints `*(inf)22`, `ox` prints `xxx`, `*236` prints `*632`, and every other input prints itself. Each text includes its `Domain` line, and the texts of `*732`, `*532`, `*632`, and `*236` their domain summaries | Ignored |
| Domain snapshots | Command line of Sections 5.2, 5.4, and 5.5 of the plan, and a snapshot of the parameter listing | `insta`, `info` | The text and status for `info '*542'`, `info '*433'`, `info '*(1001)32'`, `info '*442'`, `info '*333'`, `info '*2222'`, `info '*332'`, `info '*722'`, `info '*55'`, `info '*22'`, `info --check '*732'`, `info --check 532`, `info --check '*(1000)32'`, `info --check '*2222'`, `info --check '*532'`, `info --check '*55'`, `info --check '*(1000)22'`, and `info --check '*(1001)(1001)'`, which is over the limit of `escher-domain`. The text of `*2222` holds its `Parameter` line. The plan writes the triangle $(3, 3, 4)$, whose canonical signature is `*433` | Ignored |
| Diagnostic snapshots | Snapshots, failure paths | `insta`, `diagnostics` | The diagnostic and status of every error kind of `escher-signature`, each text of a kind that has several except the `Overflow` texts for the group order and the deformation dimension, which no known input produces (Section 12.1 of `docs/design/signature.md`), `23`, `2*12`, an empty argument, an argument longer than 120 characters, a control character, a span after the infinity sign, and a span that is the infinity sign, under one caret | Ignored |
| Usage snapshots | Snapshots, failure paths | `insta`, `usage` | `--help`, `-h`, `info --help`, no argument, an unknown subcommand, the subcommand `help`, an unknown option, a missing signature, and a repeated `--check`. The help of `info` shows `--check` | Ignored |
| Version tests | None | Plain tests, `usage` | `--version` and `-V` give `Success`, empty standard error, and standard output equal to the package name, one space, the package version, and a line feed. Each test takes the name and the version from the package when it compiles, so a new version needs no change to the tests | Runs |
| Echo limit tests | Failure paths | Plain tests, `diagnostics` | The status and the exact diagnostic for a signature of exactly 120 characters, which gets the echo line and the caret line, for one of 121 characters, which gets the byte span, and for one of 42 characters and 124 bytes, built with the infinity sign, which gets the echo line and the caret line because the limit counts characters | Runs |
| Status tests | Failure paths | Plain tests, `main.rs`, `usage`, and `info` | `Success` for `*732`, `Failure` for `23`, `Usage` for an unknown subcommand and for an argument that is not UTF-8, each with the start of the standard output or standard error that it writes, equal output for `*22(inf)` and the same signature with the infinity sign, and the exit statuses 0, 1, and 2 of the three values of `Status` | Runs |
| Verification status tests | Command line of Section 5.2 of the plan, failure paths | Plain tests, `info` | `Success`, empty standard error, and a standard output that ends with `Verification: passed` and a line feed for `info --check '*732'`, and the same with `--check` after the signature. `Failure`, the diagnostic of Section 6.7, and a standard output that ends with the `Domain` line for `info --check 532`. `Success` and empty standard error for `info 532` | Runs |
| Failed check text | Failure paths | Unit tests in `info.rs` | The private formatter, given a failed check as plain numbers, writes `failed, residual 2.5e-3, tolerance 1.0e-6`, writes `inf` for an infinite residual, and writes `NaN` when given a NaN, which no residual of `escher-domain` is, and the private function that writes the diagnostic of Section 6.7 writes `2 of 6 checks failed` for two failed checks. Given rows of label, verdict, residual, and tolerance with one failed check of six, the private function that builds the verification lines adds the six check lines and `Verification: failed` and returns an error. The private function that writes the text writes those lines first and then returns that error, whose diagnostic is `error: cannot verify fundamental domain: 1 of 6 checks failed` | Runs |
| Residual labels | Command line of Section 5.2 of the plan | Plain test, `info` | `Success` for `info --check '*(11)(10)5'`, and each line that starts with `Check ` equals the line that the format of Section 5.6 gives for the residual and the tolerance of the matching accessor of `Verification`, computed in the same process for the same signature | Ignored |
| Output error tests | Failure paths | Plain tests, `info` and `usage` | A writer that fails with a broken pipe gives `Failure` and empty standard error, for `info`, for `info --check`, and for `--version`. A writer that fails with another error gives `Failure` and the diagnostic of Section 6.6, and so does a writer whose flush fails. For `info --check 532`, the failed write gives only that diagnostic and no verification diagnostic. A failing standard error keeps `Failure` after a signature error and `Usage` after a usage error | Runs |
| Control characters | Failure paths | Plain test, `usage` | A usage error for an argument with control characters writes `?` in place of each of them (CLI-15) | Runs |
| Auto traits | None | Plain test, `main.rs` | `Status` is `Send` and `Sync` (rule CONC-1 of `docs/rust-coding-standards.md`). The test calls a generic function whose type parameter is bounded by `Send + Sync`, so it fails to compile if `Status` loses either trait | Runs |
| Render usage snapshots | Command line of Section 5.3 of the plan, failure paths | `insta`, `usage` | `render --help`, and for `render`: a missing signature, a missing `--output`, `--size 512`, `--size 512x`, `--samples x`, `--colouring three`, `--threads 0`, `--threads 257`, a repeated `--stats`, a repeated `--size`, `--set aspect`, `--set =2`, `--rotation 1,2`, `--rotation a,b,c`, `--rotation 1,2,3,4`, `--light 1,,2`, `--shading gouraud`, a repeated `--rotation`, and `--rotation --stats`. Each of these that gives `--output` gives the fixed path that no test creates. The snapshot of `--help` shows `render` among the commands, and the snapshot of `render --help` shows the usage line, the defaults, the possible values of `--colouring` and `--shading`, `--set`, `--rotation`, and `--light` | Ignored |
| Render diagnostic snapshots | Command line of Sections 5.3 and 5.5 of the plan, failure paths | `insta`, `render` | The diagnostic and status of `render 23`, `render 532`, `render '*532' --set aspect=2`, `--rotation nan,0,0`, `--light 0,0,0`, and `--light inf,0,0` for `*532`, `render '3*(inf)'`, `render '*(1001)32'`, the three examples of settings of Section 6.8, three colours and one colour for `parity`, `--colours '#e9c46a,red'`, `--border-colour red`, `--background '#ffff'`, `--size 0x0`, a size over the limit, `--samples 0`, a sample grid over the limit, `--border-width=-1`, and an output path in a directory that does not exist. `--border-width=-1` uses `=`, so that `clap` reads `-1` as the value (Section 11.1) | Ignored |
| Statistics snapshot | Command line of Section 5.3 of the plan | `insta`, `render` | The status, the statistics lines, and the empty standard error of `render '*732' --size 16x16 --samples 1 --stats`, with the values masked as CLI-28 requires. `Samples: 256` stays in the snapshot | Ignored |
| Spherical statistics snapshot | Command line of Section 5.5 of the plan | `insta`, `render` | As "Statistics snapshot", for `render '*532' --size 16x16 --samples 1 --stats`, in a test and a snapshot of its own | Ignored |
| Written file | The PNG file decodes to the requested size | Plain test, `render` | `Success`, empty standard output and standard error, and a file for `render '*732' --size 48x32 --samples 2`. The test reads the file by hand: it starts with the 8-byte PNG signature, its first chunk is `IHDR` with length 13, a width of 48 and a height of 32 as big-endian `u32` values, a bit depth of 8, and the colour type 2, which is RGB, as Section 5.6 of `docs/design/render.md` requires, and a chunk `sRGB` comes before the first `IDAT` chunk. The non-square size catches a swap of width and height | Ignored |
| Thread independence | Command line of Section 5.3 of the plan (CLI-23) | Plain test, `render` | The files of `render '*732' --size 64x64 --samples 2` with `--threads 1` and with `--threads 4` are equal byte for byte, and both equal the PNG bytes that `escher-render` encodes from one call of `render` on the whole image with the same options. The second comparison catches bands that are assembled in the wrong order, which gives the same wrong bytes for every number of threads | Ignored |
| Global pool unused | Command line of Section 5.3 of the plan (CLI-22) | Plain test, `render` | After `render '*732' --size 8x8 --samples 1` returns `Success`, building the global pool of `rayon` still succeeds, so `render` did not use the global pool. The test relies on nextest running each test in a process of its own | Ignored |
| Output file handling | Failure paths (CLI-21, `DEC-30`) | Plain tests, `render` | An existing file at the output path is replaced by the image. An output path that is a symbolic link to a file gives the image in that file and stays a link (Section 11.1). For `render 532`, for `--size 0x0`, for a bad colour, for `render '*2222' --set aspect=9`, and for `render '*532'` with `--rotation nan,0,0` and with `--light 0,0,0`, an existing file keeps its bytes and a missing path stays missing | Ignored |
| Render output errors | Failure paths | Plain tests, `render` | With `--stats`, a writer that fails with a broken pipe gives `Failure` and empty standard error, and the output file is written. A writer that fails with another error gives `Failure` and the diagnostic of Section 6.6. For a broken pipe on the output file, the test makes a FIFO with `mkfifo` in its test directory, a reader thread opens the FIFO and closes it at once, and `render '*732' --size 1024x1024 --samples 1` writes into it. It asserts `Failure`, empty standard output, and a standard error of exactly `error: cannot write the output file`, then `  caused by: Broken pipe (os error 32)` | Ignored |
| Render status tests | Failure paths | Plain tests, `render` | `Failure` and the first line of standard error for `render 532`, for two colours for `one`, for `--colours '#e9c46a,red'`, and for `--size 0x0`, and `Usage` for a missing `--output`. Each fails before step 5, so it starts no thread pool and writes no file. Outside Miri, each test that gives `--output` removes any file at the fixed path first and asserts afterwards that no file exists there | Runs |
| Colour parser | Failure paths | Unit tests in `render.rs` | The private parser reads `#e9c46a`, `#E9C46A`, and `#e9c` as the channels that Section 4.6 gives, and rejects `e9c46a`, `#`, `#e9c4`, `#e9c46a0`, `#ggg`, an empty string, a colour with a space before or after it, and a colour with a non-ASCII digit. The private function that splits `--colours` reports the position of the first bad item, and the text of the wrong count uses `1 colour` and `1 was given` for the number 1 | Runs |
| Statistics text | Command line of Section 5.3 of the plan | Unit tests in `render.rs` | The private formatter, given counts as plain integers and a time, writes the five lines of Section 4.7: `10.01` for 1001 folds over 100 inside samples, `0.00` for no inside sample, a correct mean for `u64::MAX` folds, and `0.315` for a time of 0.3154 seconds | Runs |
| Thread pool diagnostic | Failure paths | Unit test in `render.rs` | The private function that builds the error of a failed thread pool, given a stand-in error whose `Display` text is that of its inner `std::io::Error`, which is also its source, gives the diagnostic `error: cannot render *732`, then `  caused by: cannot start the thread pool`, then `  caused by: no thread can be started`, with the cause once. A stand-in is needed because the error type of `rayon` has no public constructor | Runs |
| Euclidean render | Command line of Section 5.4 of the plan | Plain tests, `render` | `render '*442' --size 48x32 --samples 2` gives `Success` and a file whose header holds the requested size, read as the test "Written file" reads it. The files of `render '*2222' --size 64x64 --samples 2` with `--set aspect=1.618` and with no setting differ, and the file with the setting is equal byte for byte with `--threads 1` and with `--threads 4` | Ignored |
| Setting parser | Range errors, failure paths | Unit tests in `args.rs` | The private parser reads `aspect=1.5` as the name `aspect` and the number 1.5, and `aspect=nan` as a NaN, and rejects `aspect`, `=2`, `aspect=`, `aspect=x`, and `a=b=1`, whose text after the first `=` is not a number, with the message of Section 4.6 | Runs |
| Repeated setting | Failure paths | Unit tests in `render.rs` | The private function that collects the settings of `--set` gives `setting 3 of --set names the same parameter as setting 1` when setting 3 repeats the name of setting 1, and keeps every setting when no name repeats | Runs |
| Spherical render | Command line of Section 5.5 of the plan | Plain tests, `render` | `render '*532' --size 48x32 --samples 2` gives `Success` and a file whose header holds the requested size, read as the test "Written file" reads it. The files of `render '*532' --size 64x64 --samples 2` with `--threads 1` and with `--threads 4` are equal byte for byte. `--rotation -20,35,0` and `--light=-1,1,2` give `Success`, and the file with `--rotation 20,35,0` differs from the file with the default rotation | Ignored |
| Triple parser | Failure paths | Unit tests in `args.rs` | The private parser reads `20,35,0`, `-1,1,2`, and `nan,0,0` as three numbers, and rejects `1,2`, `a,b,c`, `1,2,3,4`, `1,,2`, an empty value, and a value with a space before or after an item, with the message of Section 4.6 | Runs |
| Smoke test | One process-level smoke test of the binary | Plain test, `process` | The binary that Cargo names in `CARGO_BIN_EXE_escher`, run with `render '*732' --size 32x32 -o <path>`, exits with status 0, writes nothing to standard output or standard error, and writes a file that starts with the PNG signature | Ignored |
| Doctests | None | Doctests | The crate-level examples | Not run, because Miri runs through nextest, which runs no doctest |

The argument that is not UTF-8 is built with `std::os::unix::ffi::OsStringExt`, and the symbolic link with `std::os::unix::fs::symlink`, so those tests compile only on Unix, which covers Linux, the supported platform.

No known input makes a check fail, so the failed-check path is tested through private functions, as Section 12.1 of `docs/design/signature.md` does for its `Overflow` texts. The test "Failed check text" covers the formatter of one check, the check lines, the `Verification` line, and the count of failed checks built from the rows, the writing of the text before the verdict is returned, and the diagnostic function.

A snapshot holds the status, the standard output, and the standard error of one `run` call, in this form, which a helper in the module `common` builds (rule TEST-12 of `docs/rust-coding-standards.md`):

```text
status: <Debug text of the Status>
--- stdout
<standard output>--- stderr
<standard error>--- end
```

Each stream follows its marker line unchanged, so a stream that ends with a line feed puts the next marker on a line of its own, and a stream that does not runs into it. The final marker `--- end` is needed because `insta` trims the whitespace at the end of a snapshot before it compares, so without it a missing or extra final line feed of standard error would pass. The writer that fails, `FailingWriter`, is also in `common`.

Every snapshot must be deterministic (rule TEST-2 of `docs/rust-coding-standards.md`), and two kinds of floating-point value in the output need a rule for that:

- The residual of a passed check is a rounding error, whose last digits can depend on the mathematics library of the platform. In every line that starts with `Check `, the snapshot helper replaces the text between `residual ` and the next `,` with `<residual>`, as the example of Section 5.3 shows. The tolerance, the result, and the line `Verification` stay in the snapshot. The test "Failed check text" covers the form of a residual. The test "Residual labels" covers which residual each label prints, without a snapshot, because it compares the output with values computed in the same process. Its input `*(11)(10)5` is the only reflection triangle with corner orders up to 30 whose six residuals print pairwise distinct in the development container, so there a swap of any two checks changes the output. The test does not assert that the residuals are distinct, so on a platform where two of them print the same, a swap of those two checks goes unnoticed.
- The values of rows 12 to 14 are printed with six digits after the point (Section 5.5). A snapshot input is chosen so that the exact value of each of these rows lies farther from a rounding boundary of the sixth digit than the bound of Section 9.3 of `docs/design/domain.md` in the column for the orders of that input, converted to degrees for an angle. The column "Orders up to 12" applies to `*732`, `*542`, and `*433`, and the column "(1000, 3, 2)" to `*(1000)32`. Every correct build then prints the same digits. A 256-bit computation of `*732` confirms this for every value of its summary, as Section 5.3 shows. The test "Triangles" of `docs/design/domain.md` checks the values of the other inputs against the reference arithmetic of `escher-domain`, within the bound of that column (Section 12.1 of that document). For each of these inputs, the distance of every exact value to the nearest rounding boundary of the sixth digit, computed in double precision, is larger than that bound, so every snapshot input meets the condition. For `*632`, `*442`, `*333`, and `*2222`, the bound of each quantity in Section 9.4 of `docs/design/domain.md` applies, and every value of their summaries lies at least $4.36 \times 10^{6}$ times its bound, converted to degrees for an angle, from the nearest rounding boundary of the sixth digit. For `*532`, `*332`, `*722`, `*55`, `*22`, and `*(1000)22`, the bound of each quantity in Section 9.5 of `docs/design/domain.md` applies. Every exact value of their summaries lies at least $8.27 \times 10^{-8}$ from the nearest rounding boundary of the sixth digit, at side 1 of `*332`, whose length is 1.2309594173. That distance is at least $2.07 \times 10^{5}$ times the largest bound of Section 9.5, $4 \times 10^{-13}$. Their angles are whole numbers of degrees or $180/7$ and 0.18 degrees, far from a boundary. The minimum, the maximum, and the default of a parameter are constants of `escher-domain`, whose `Display` text is the same on every platform, so they need no such condition.

CLI-20. A snapshot MUST NOT hold a residual of a check, and its inputs MUST meet the condition on rounding boundaries above, because a value that depends on the last bits of floating-point arithmetic makes the snapshot test fail on a platform whose mathematics library rounds differently, although the program is correct.

The statistics lines need a rule of their own. The time differs on every run. The fold counts can differ between platforms, because a sample on or next to a wall can take a different side when the last bits of a side value differ (`UNV-3`). In every line that starts with `Mean folds: `, `Maximum folds: `, `Unconverged samples: `, or `Time: `, the snapshot helper replaces the text after the first `: ` with `<value>`. The line `Samples` stays, because the number of samples is the product of the size and the grid.

CLI-28. A snapshot MUST NOT hold the value of a statistics line other than `Samples`, because the time differs on every run and the fold counts can differ between correct builds, so a snapshot that held them would fail although the program is correct.

A snapshot of `render` holds no path, because no diagnostic and no statistics line repeats the output path (CLI-26), so the snapshot does not depend on the directory of `CARGO_TARGET_TMPDIR`.

Every display text of `escher-signature` that a known input produces passes through a diagnostic snapshot, so `escher-signature` needs no `insta` dependency. Likewise, the snapshots of the `Domain` line cover the `UnsupportedSignature` and `LimitExceeded` texts of `escher-domain`, and the render diagnostic snapshots its texts of an unknown parameter name and of a value outside the range of its parameter. The render diagnostic snapshots cover the texts of the error kinds of `escher-render` that an option produces.

The smoke test is the one test of this crate that starts the binary, as layer 9 of Section 7.1 of `docs/design/architecture.md`. It checks only that the binary runs `run` and turns its status into the exit status, because every other behaviour is tested in the test process.

The test "Written file" reads the PNG header by hand, so this crate needs no `png` dependency. The tests of `escher-render` decode the image with `png` and compare its pixels (Section 8 of `docs/design/render.md`).

### 8.2. Snapshots Under nextest

The recipe `test` runs the snapshot tests through nextest like every other test, with no setting of their own. A snapshot test fails when the output differs from its committed snapshot file, and an agent reviews and accepts a change with `cargo insta review`, which the coding standard lists as run by hand (Section 4.8 of `docs/rust-coding-standards.md`). No recipe wraps `cargo insta`.

A search of the source of `insta` 1.49.0 on 2026-11-16 found that it reads these environment variables:

- `CI`, `INSTA_UPDATE`, `INSTA_FORCE_PASS`, `INSTA_FORCE_UPDATE`, `INSTA_FORCE_UPDATE_SNAPSHOTS`, `INSTA_OUTPUT`, `INSTA_REQUIRE_FULL_MATCH`, and `INSTA_WORKSPACE_ROOT`.
- `INSTA_TEST_RUNNER`, `INSTA_TEST_RUNNER_FALLBACK`, `INSTA_GLOB_FILTER`, `INSTA_GLOB_FAIL_FAST`, `INSTA_PENDING_DIR`, `INSTA_SNAPSHOT_REFERENCES_FILE`, `INSTA_WARNINGS_FILE`, `INSTA_YAML_BLOCK_STYLE`, and `INSTA_DIFF_TOOL`.
- `NEXTEST_RUN_ID`, `TF_BUILD`, and `CARGO`.

A reading of `src/env.rs`, `src/utils.rs`, `src/snapshot.rs`, and `src/macros.rs` of `insta` 1.49.0 and runs of the snapshot tests of this crate on 2026-11-16 established this behaviour of the variables that matter here:

| Variable | Behaviour when unset |
| --- | --- |
| `INSTA_UPDATE` | `auto`, unless an `insta.yaml` file sets it, and the repository has none. Outside CI, a failing snapshot test writes the new snapshot beside the committed one as a `.snap.new` file. With `CI=true`, a failing run wrote no such file |
| `CI` | Not CI. `insta` counts a run as CI when `CI` has any value other than `false`, `0`, or the empty string, or when `CI` is unset and `TF_BUILD` is set |
| `INSTA_FORCE_PASS` | False. Any value other than `0`, `1`, or the empty string is an error |
| `INSTA_OUTPUT` | `diff` |
| `INSTA_REQUIRE_FULL_MATCH` | False, so `insta` trims the whitespace at the end of a snapshot before it compares, which is why the snapshot form of Section 8.1 ends with a marker line |
| `INSTA_FORCE_UPDATE`, `INSTA_FORCE_UPDATE_SNAPSHOTS` | No effect. Either one set to `1` forces `INSTA_UPDATE=force` |
| `INSTA_WORKSPACE_ROOT` | `insta` runs `cargo metadata --format-version=1 --no-deps` in the manifest directory of the crate, with the `cargo` that `CARGO` names, or plain `cargo` when `CARGO` is unset, to find the workspace root. It runs the command once per process and keeps the result. When the command fails, it uses the manifest directory and writes a warning to standard error. The variable is read at compile time as well as at run time |
| `NEXTEST_RUN_ID` | It gives only the run identifier of pending inline snapshots, which this crate does not use |

A run of the test binary without nextest, with a wrapper for `cargo` set as `CARGO`, showed exactly one `cargo metadata` call. Under nextest, which runs each test in its own process, and for which Cargo sets `CARGO` to its own path, no warning appeared and every snapshot resolved. That the `cargo metadata` call succeeds under nextest, once for each snapshot test, is inferred from that result and was not observed directly.

`.gitignore` ignores `*.snap.new`, which a failing test writes, and `*.pending-snap`, which `insta` writes only for inline snapshots. Section 14.2 of `docs/design/tooling.md` lists the variables that a contributor or CI needs to know.

CLI-12. An agent MUST NOT set `INSTA_FORCE_PASS` in a recipe, a hook, or a workflow, because it turns a failing snapshot test into a pass, and a changed text then reaches `main` unreviewed.

Section 6 of `docs/design/architecture.md` names the documents that state how a test that cannot run under Miri is marked, and the tests of this crate follow them.

### 8.3. Tests Under Miri

The snapshot tests carry `#[cfg_attr(miri, ignore = "the snapshot library reads snapshot files and starts a process")]`, because Miri's isolation refuses file access and process creation (`DEC-5`). The reason is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

CLI-13. Every test that uses `insta` MUST carry that attribute, because without it `just check-miri` fails on every snapshot test.

The test "Residual labels" carries `#[cfg_attr(miri, ignore = "the interpreter rounds the transcendental functions differently on each call, so two builds of one domain differ in the last digits of a residual")]`, because it compares the residuals of two builds of one domain, the one that it makes and the one that `info` makes.

The other tests of Section 8.1 run under Miri unchanged, because they use only memory, and `clap` runs under Miri: on 2026-11-16 every such test of rows 1 to 9, the diagnostics of Sections 6.1 to 6.6, and the usage text passed under `just check-miri`. A snapshot test run under Miri with its attribute overridden failed with `unsupported operation: 'open' not available when isolation is enabled`. A test that fails under Miri only because Miri does not support an operation is marked in the same way, with its own reason.

The plain tests that run `info` on `*732` build its domain under Miri, and those that pass `--check` also verify it, so they run the arithmetic of `glam` and the floating-point functions of the standard library in the interpreter. The verification of `*732` needs few matrix products (Section 8.2 of `docs/design/domain.md`), so these tests need no attribute. They assert the status and the start or the end of the output, and no digit of a floating-point value, because the interpreter need not reproduce the last bits of the host library (Section 12.3 of `docs/design/domain.md`). They pass under `just check-miri`.

Every test of `render` that reaches step 5 of Section 4.5 starts the thread pool, and every such test that succeeds writes the output file. A two-thread pool of `rayon`, run under Miri on a prototype outside the repository on 2026-11-16, made Miri report undefined behaviour inside `crossbeam-epoch` 0.9.21, a dependency of `rayon`. With the option `-Zmiri-tree-borrows`, Miri reported memory leaks instead. The owner accepts this for the proof of concept (`DEC-21`).

CLI-27. Every test that reaches the thread pool MUST carry `#[cfg_attr(miri, ignore = "the thread pool library fails the checks of the interpreter")]`, unless it already carries the attribute of a snapshot test, because without it `just check-miri` fails on a defect that is outside this crate and that the owner has accepted.

The smoke test carries `#[cfg_attr(miri, ignore = "the interpreter cannot start a process")]`. The tests "Output file handling" that fail before step 5 still create and read files, which the isolation of Miri refuses, so they carry `#[cfg_attr(miri, ignore = "the interpreter refuses file access")]`. The test of a broken pipe on the output file reaches the thread pool and also starts the process `mkfifo`. It carries the attribute of CLI-27, because a test carries only one `ignore` attribute. The tests "Render status tests" fail before step 5, so they run under Miri, and they pass under `just check-miri`. Under `cfg!(miri)` they skip the removal of a stale file and the check that no file exists, because the isolation of Miri refuses file access. They build the domain of `*732` in the interpreter, as the plain tests of `info` do.

## 9. Dependencies

### 9.1. Workspace Dependencies

The crate declares these dependencies, each from one entry in the root `[workspace.dependencies]` table (rule DEP-2 of `docs/rust-coding-standards.md`). The table also holds the entries that `docs/design/signature.md`, `docs/design/geometry.md`, `docs/design/domain.md`, and `docs/design/render.md` specify for the other crates.

```toml
[workspace.dependencies]
anyhow = "1.0.103"
clap = { version = "4.6", default-features = false, features = [
    "derive",
    "error-context",
    "help",
    "std",
    "suggestions",
    "usage",
] }
escher-domain = { version = "0.1.0", path = "crates/escher-domain" }
escher-render = { version = "0.1.0", path = "crates/escher-render" }
escher-signature = { version = "0.1.0", path = "crates/escher-signature" }
insta = { version = "1.49", default-features = false }
rayon = "1.12.0"
```

| Dependency | Kind | Features | Reason |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | Parses and classifies the signature. The path dependency carries a version (TOOL-25) |
| `escher-domain` | Normal | None | Lists the shape parameters for the parameter lines, builds and verifies the fundamental domain for the domain summary and `--check`, and builds the domain that `render` draws with the settings. The path dependency carries a version (TOOL-25) |
| `escher-render` | Normal | None | Renders the image, counts its statistics, and encodes it as PNG for `render`. The path dependency carries a version (TOOL-25) |
| `rayon` | Normal | Default | The thread pool of `render` (`DEC-21`, Section 3.5 of `docs/design/architecture.md`) |
| `clap` | Normal | The list above, without `color`, `env`, and `wrap_help` | The derive API (Section 3.1 of `docs/design/architecture.md`). Without `color`, `anstream` and its dependencies are not built (Section 9.2) |
| `anyhow` | Normal | Default | The chain of errors that Section 5.8 of `docs/design/architecture.md` requires. Version 1.0.103 fixes RUSTSEC-2026-0190 (Section 10 of `docs/research/rust-ecosystem.md`) |
| `insta` | Dev | None, so without `colors` | Snapshot tests (rule TEST-2). Without `colors`, `console` is not built (Section 9.2) |

The crate declares neither `escher-geometry` nor `glam`. The measurements of the domain are methods of `FundamentalDomain` that return plain `f64` values, so this crate names no `glam` type (Section 15.2 of `docs/design/domain.md`). `render` passes the domain to `escher-render` unchanged and reads only integers from the statistics, so it names no `glam` type either. `escher-domain` brings `escher-geometry`, `glam`, and `thiserror` into the build of the binary, and `escher-render` brings `png` and its dependencies.

The crate declares no colour crate and no `png`. The private parser of Section 4.6 reads the two hexadecimal forms of a colour (`DEC-29`), and the tests read the PNG header by hand (Section 8.1).

The package keeps `publish = false` (`DEC-22`) and declares no Cargo feature.

### 9.2. Verified Resolution

An adversarial test resolved these dependencies with network access on 2026-11-16, together with those of `escher-signature`, to the highest versions compatible with Rust 1.99. It covered every dependency of Section 9.1 except `escher-domain`, `escher-render`, and `rayon`, and verified these facts:

- The resolved versions include `anyhow` 1.0.104, `clap`, `clap_builder`, and `clap_derive` 4.6.7, `clap_lex` 1.1.1, `strsim` 0.11.1, `heck` 0.5.0, `syn` 3.0.10, `proc-macro2` 1.0.107, `quote` 1.0.47, `unicode-ident` 1.0.27, `insta` 1.49.0, `similar` 2.7.0, and `tempfile` 3.27.0.
- `cargo tree -d --target all` printed no duplicate version. The graph has one major version of `syn`, so `syn` 2 and 3 do not both occur.
- With the allow-list `Apache-2.0`, `GPL-3.0-or-later`, `MIT`, and `Unicode-3.0`, `cargo deny check licenses bans` passed with no unused licence. Without `MIT` it failed only on `strsim`, which the `clap` feature `suggestions` brings. `MIT` is therefore the one licence that this crate adds beyond those of `escher-signature`.
- The advisory and source checks passed.
- The highest minimum Rust version that the dependencies of this crate declare is 1.85, for `clap`, within `rust-version = "1.99"`.
- A prototype of this crate printed `escher-cli 0.1.0` for `--version` and `-V` on standard output with status 0, and the help for `--help` on standard output with status 0. With no argument it printed the help on standard error with status 2. A missing signature, an unknown subcommand, `-x`, and an argument that is not UTF-8 each gave an error on standard error with status 2. The usage line named `escher`.

The build of the package, checked with `cargo tree` on 2026-11-16, has these properties:

- `cargo tree -e features -i clap_builder` shows only the features `derive`, `error-context`, `help`, `std`, `suggestions`, and `usage`.
- `cargo tree --target all` lists none of `anstream`, `console`, `colorchoice`, and `terminal_size`.
- `insta` is built with no features.
- The resolved versions also include `anstyle` 1.0.14, which `clap_builder` always needs, and the dependencies of `insta`: `once_cell` 1.21.4, `fastrand` 2.5.0, `rustix` 1.1.6, `linux-raw-sys` 0.12.1, `errno` 0.3.15, `getrandom` 0.4.3, `windows-sys` 0.61.2, and `windows-link` 0.2.1.
- `cargo tree -d --target all` prints no duplicate version.

`deny.toml` allows `MIT`, and Section 8 of `docs/design/tooling.md` records its compatibility with GPL-3.0-or-later, as TOOL-24 of that document requires. `just check-deny` passes with no unused licence.

The dependencies that `escher-domain` brings were resolved on 2026-11-16 as Section 11 of `docs/design/geometry.md` records: `glam` adds no transitive dependency, no duplicate version, and no licence. `thiserror` is already in the workspace. With `escher-domain` in the package, `just check-deny` and `just check-msrv` passed on 2026-11-16, and `cargo tree -d --target all -p escher-cli` printed no duplicate version.

An adversarial test resolved the dependencies of `render` on 2026-11-16 in a prototype outside the repository:

- `rayon` 1.12.0 resolved with `rayon-core` 1.13.0. It added no duplicate version, `cargo deny check licenses bans` passed, every licence that it brought was `MIT OR Apache-2.0`, and the highest minimum Rust version was 1.80.
- `csscolorparser` 0.9.1 with its default features brought `syn` 2 through `phf_macros`, beside the `syn` 3 of the workspace, and failed the ban on duplicate versions. Without its default features it passed, but read no colour name.
- `palette` 0.7.7 brought `syn` 2 through `palette_derive` with its default features, with `std` alone, and with `libm` alone, and failed the ban on duplicate versions each time.

With `escher-render` and `rayon` in the package, the lock file of the workspace on 2026-11-16 holds `rayon` 1.12.0, `rayon-core` 1.13.0, `crossbeam-deque` 0.8.8, `crossbeam-epoch` 0.9.21, `crossbeam-utils` 0.8.23, and `either` 1.19.0, and keeps `flate2` at 1.1.9. On that date `cargo tree -d --target all` printed no duplicate version, and `just check-deny`, `just check-msrv`, and `just check-features` passed, with no licence added to `deny.toml`.

The owner chose a parser of the project's own for colours (`DEC-29`). Section 9 of `docs/design/render.md` records the resolution of `png` and the dependencies that `escher-render` brings into the binary.

### 9.3. Dependency Risks

cargo-deny leaves dev-dependencies out of the licence check and the duplicate check by default. `deny.toml` overrides both defaults, as Section 8 of `docs/design/tooling.md` states, so `just check-deny` checks dev-dependencies. With the overrides, `cargo deny list` names `insta` 1.49.0, its dependencies such as `similar`, `tempfile`, `once_cell`, `fastrand`, and `rustix`, and `proptest`. In the test of Section 9.2, which ran without the overrides, it named none of `insta`, `similar`, `tempfile`, `proptest`, and `rand`.

CLI-14. The dev-dependencies of this crate MUST stay inside the dependency policy of Section 8 of `docs/design/tooling.md`, so that `cargo deny list` names `insta` and its dependencies, because otherwise the licences, advisories, and duplicates of the snapshot library go unchecked while the dependency policy appears to cover them.

`rayon` fails under Miri (Section 8.3), so Miri checks no code of `rayon` and `crossbeam-epoch` that this crate runs. A defect in that code would reach the binary without any check of this repository finding it. The owner accepts this risk for the proof of concept (`DEC-21`).

## 10. Unverified Statements

This section lists the unverified statements of Section 7 of `docs/design/poc-plan.md` that this crate relies on, as PLAN-3 of that document requires. The crate relies on `UNV-20` and `UNV-21` through `escher-signature`, on `UNV-10`, `UNV-26`, `UNV-28`, and `UNV-29` through `escher-domain`, and on `UNV-1`, `UNV-3`, `UNV-4`, `UNV-15`, `UNV-16`, and `UNV-18` through `escher-render`, whose design states how it relies on each.

| Id | How this crate relies on it |
| --- | --- |
| `UNV-20` | `info` prints the canonical signature, with parentheses for orders of 10 or more and `(inf)` for an infinite order, and accepts both spellings of infinity and `1` as the trivial signature |
| `UNV-21` | `info` prints the geometry and the area of a non-compact signature whose Euler characteristic is not 0, and prints no deformation dimension for a non-compact signature |
| `UNV-26` | `info --check` reports the checks of the walls of the domain, which `escher-domain` builds through the formula of the wall through two points that `UNV-26` covers. The walls, relators, and angles checks test the result (Section 14 of `docs/design/domain.md`). The domain summary does not depend on the walls, because `escher-domain` measures it from the vertices |
| `UNV-10` | `info` prints one `Parameter` line for each parameter that `escher-domain` specifies, such as one for `*2222`. No source states whether the published parameter count of `*2222` includes the scale, so the lines list the parameters of the construction and claim no count |
| `UNV-28` | `info` prints the summary of a Euclidean triangle, and `render` draws it, from the vertex coordinates that `escher-domain` builds, whose checks and tests its design states |
| `UNV-29` | `info` prints the summary of a lune, and `render` draws it as a fundamental domain, which no source states. `info --check` reports the checks that test it |
| `UNV-1` | `render` relies on the fold to stop for the samples of an image of a reflection triangle, rectangle, or lune. `Unconverged samples` counts the samples for which it does not stop within the fold cap, which on the sphere can happen near a vertex of very high order |
| `UNV-16` | The ball that `render` draws comes from the lift onto the ball and the Phong reflection model, whose primary paper is unread |
| `UNV-3` | A sample on or next to a wall can take either neighbouring tile, which is why CLI-28 masks the fold counts |
| `UNV-4` | The default fold cap of `DEC-7` rests on fold counts measured on a prototype, and no source bounds them |
| `UNV-15` | The border that `render` draws comes from the gradient form of the border estimate (`DEC-28`) |
| `UNV-18` | The default colouring `parity` colours the tiles by the parity of the fold |

## 11. Assumptions, Choices, and Open Questions

### 11.1. Assumptions

- The infinity sign takes one column in the terminal. Unicode gives it an ambiguous East Asian width, so a terminal that renders such characters as wide shifts the caret one column to the right for each infinity sign before the span.
- With its features `env` and `wrap_help` off, `clap` reads no environment variable, so the output does not depend on the environment. A reading of the source of `clap_builder` 4.6.7 on 2026-11-16 found that it reads the environment only in `Arg::env`, which needs the feature `env`, and for `COLUMNS` and `LINES`, which need the feature `wrap_help`. No run has confirmed this.
- The residual of a passed check can differ in its last digits between platforms, which is why Section 8.1 replaces it in the snapshots. No run on two platforms has measured this. The replacement is safe either way.
- With the default actions of its derive API, `clap` rejects a flag or an option that is given more than once. The tests "Usage snapshots" and "Render usage snapshots" confirm this: a repeated `--check`, a repeated `--stats`, and a repeated `--size` each give a usage error.
- With the action `Append`, `clap` accepts `--set` more than once and keeps its values in the order given. The render diagnostic snapshot of a parameter set twice relies on that order, because it names the positions of the two settings.
- `clap` reads the value of `--border-width=-1` as `-1`, although it starts with `-`. The test "Render diagnostic snapshots" confirms this: its snapshot holds the diagnostic that the border width -1 is negative or not finite.
- With `allow_hyphen_values`, `clap` reads `--light -1,1,2` as the value `-1,1,2`, and without it rejects the value as an unknown argument. With the setting, `--rotation --stats` takes `--stats` as the value, and `--rotation -h` takes `-h`. A probe of `clap` 4.6.7 outside the repository observed each of these. The tests "Render usage snapshots" and "Spherical render" confirm the cases with the setting that they run, which are `--rotation --stats` and `--rotation -20,35,0`.
- `clap` generates the usage line of `render` that Section 4.1 gives, and the text `[default: <value>]` and `[possible values: one, parity]` in its help. The test "Render usage snapshots" confirms this with the snapshot of `render --help`.
- `rayon` reads the environment variable `RAYON_NUM_THREADS` only for a pool whose builder sets no number of threads, so it has no effect on the pool of `render`, which always sets one. No reading of the source of `rayon-core` and no run has confirmed this, and no test can, because the output does not depend on the number of threads (CLI-23). An error here changes only the number of threads that a call starts.
- `std::fs::write` follows a symbolic link at the path, as the documentation of `std::fs::File::create` states. The test "Output file handling" confirms this: through a link to a file in its own directory, the image is written to that file, and the link stays a link.
- The PNG header that the test "Written file" reads has the layout of the PNG specification: the 8-byte signature, then the `IHDR` chunk with its length, its type, the width, the height, the bit depth, and the colour type. No document under `docs/research/` cites the specification. The test passes on the bytes that `escher-render` writes, which its own tests decode with `png`, so the layout that it reads agrees with the layout that `png` writes.
- The fold counts can differ between platforms (Section 8.1). No run on two platforms has measured this. The masking of CLI-28 is safe either way.
- A write to a FIFO whose reader has closed fails with the error kind `BrokenPipe`, and does not end the process, because Rust ignores the signal `SIGPIPE` (Section 6.6). The test of a broken pipe on the output file in "Render output errors" confirms this. The assumption holds only while the PNG file is larger than the buffer of the pipe, 64 KiB by default on Linux, because a smaller file fits in the buffer and its write succeeds. The test therefore renders 1024 by 1024 pixels, which gives a PNG file of about 160 KB.

### 11.2. Choices of This Design

These choices are made here, and no other document dictates them:

1. `run` returns `Status` and not `std::process::ExitCode`, and the `clap` types are private.
2. `run` flushes standard output after each write to it (Section 3.2).
3. `clap` has no colour and only the features of Section 9.1, and `insta` has no colour.
4. `escher` with no argument is a usage error that prints the help.
5. The subcommand `help` is disabled.
6. The labels and order of the information text, with `Frieze` and `General non-compact` printed only for a non-compact signature, and the label `Deformation dimension`.
7. The format of the area, such as `2pi/3`.
8. The diagnostic format: the bound of 120 characters, `?` for each unsafe character, and the byte range for a longer argument.
9. Three statuses, with one status for every signature error (Section 7).
10. A failed write to standard output, including the help and the version text, gives `Failure`, and a broken pipe on standard output gives no diagnostic (Section 6.6).
11. Snapshots live only in this crate, with one snapshot for each `run` call.
12. No recipe wraps `cargo insta`.
13. The help, version, and usage text of `clap` has each control character other than the line feed replaced with `?` (Section 6.5).
14. A snapshot holds the status and each stream between fixed marker lines, with a final marker line (Section 8.1).
15. A `Domain` line for every valid signature, with `none, ` and the `Display` text of the error of `escher-domain` when no domain is built, and the status `Success` for such a signature without `--check` (Section 5.5).
16. The labels of rows 11 to 21, angles in degrees, and six digits after the point for the domain summary (Sections 5.1 and 5.2).
17. Every check line shows its result, its residual, and its tolerance in the form `{:.1e}`, and a final `Verification` line gives the overall result (Section 5.6).
18. With `--check`, a failed check and a missing domain each give `Failure` and a diagnostic (Section 6.7).
19. The information text is written before the verification diagnostic (Section 6.7).
20. The snapshot helper replaces the residual of each check line, and the snapshot inputs keep the printed digits away from rounding boundaries (Section 8.1).
21. The `clap` flag `--check` is a `bool` converted at once into a private enum (Section 4.3).
22. A repeated `--check` stays the usage error that `clap` gives (Section 4.3).
23. The steps of `render` and their order, with the colours read before the domain is built (Section 4.5).
24. `clap` checks only the syntax of an option value, and `escher-render` checks its range, so a size of `0x0` gives `Failure` and not `Usage` (Section 4.6).
25. The limit of 1 to 256 threads, and the default of the available parallelism, at most 256 (Section 4.6).
26. A thread pool for every call, also for one thread, and bands of one row (Section 4.5).
27. The labels, the order, and the formats of the statistics lines, with the mean rounded down and the time of the rendering step alone (Section 4.7).
28. No diagnostic repeats an option value or the output path, so a bad colour is named by its option and its position (Section 6.8).
29. The write with `std::fs::write`, which creates no directory (Section 4.7).
30. The tests read the PNG header by hand, so the crate has no `png` dev-dependency (Section 8.1).
31. The statistics snapshot masks every value except that of `Samples` (Section 8.1).
32. `--set` on `render` only, and `info` describes the default shape (Sections 4.3 and 4.5).
33. The syntax of a setting, split at the first `=`, with the number read as `--border-width` is, and `--set` as the one option that may be given more than once (Section 4.6).
34. `render` reports a parameter set twice itself, and leaves every other check of a setting to `escher-domain` (CLI-29).
35. The label, the position, and the form of the `Parameter` line, with the numbers in the `Display` form of `f64` (Section 5.1).
36. `render` refuses no domain for its geometry, because `escher-render` has a view for every geometry (RND-23 of `docs/design/render.md`).
37. The examples and snapshots that need a signature without a domain use `532` (Sections 5.3 and 8.1).
38. The options `--rotation`, `--light`, and `--shading`, with their value names, help texts, and defaults (Sections 4.2 and 4.6).
39. One parser for both triples, with exactly three items, nothing trimmed, and one message (Section 4.6).
40. `allow_hyphen_values` on `--rotation` and `--light` (CLI-31).
41. The value `none` of `--shading` for `Shading::Flat` (Section 4.6).
42. The three options are read for every signature, checked by `escher-render`, and have no effect on a hyperbolic or Euclidean signature (Section 4.6).
43. `escher info` prints no line about the view, the light, or the shading (Section 5.1).

### 11.3. Points for the Owner

None of these blocks the implementation:

1. Order of infinity (Section 15.3, item 1, of `docs/design/signature.md`, which owns it). The canonical form sorts an infinite order above every finite order, so `*22(inf)` prints as `*(inf)22`, and the examples of Section 5.3 assume this. If the owner chooses the other order, the canonical strings in the examples and the snapshots change.
2. Area of a non-compact hyperbolic signature. `info` prints it, as Section 5.3 shows for `3*(inf)`, and the value rests on `UNV-21` (Section 15.3, item 3, of `docs/design/signature.md`). The stricter option prints the area only for a compact signature until a source covers the non-compact case.
3. The trivial group as `11`. The rejection of a digit `1` followed by another digit also rejects `11` and `*11`, which ON 3.5 records as spellings in [CBG] and [CH] (Section 15.3, item 2, of `docs/design/signature.md`, which owns it). This changes what `info` accepts.
4. The largest corner order of 1000 (Section 15.3, item 2, of `docs/design/domain.md`, which owns it). `escher info '*(2000)32'` prints `Domain: none, cannot build fundamental domain: order 2000 is larger than 1000` and exits with status 0. A higher limit changes the snapshot of `*(1001)32`.
5. Check 6, the area check (Section 15.3, item 4, of `docs/design/domain.md`, which owns it). If the owner drops it, `--check` prints five check lines, and the diagnostic counts five checks.
6. Two area lines. A signature with a domain prints both `Area`, the exact value from the Euler characteristic, and `Domain area`, the area measured from the angles of the domain. The design keeps both, because their agreement is visible to a reader. The simpler option drops `Domain area`.
7. A `Domain` line for every signature. The design prints `Domain: none, ...` for every valid signature without a domain, so that a reader can tell a missing domain from a missing line. The other option prints no `Domain` line for such a signature, which keeps the text of those signatures unchanged.
8. The default fold cap of 1500 (`DEC-7` of `docs/design/architecture.md`, which owns it). A lower default changes the default of `--max-folds` and the help text.
9. An opaque background only (`DEC-7`). A `transparent` background would add a value to `--background` and need a rule for averaging samples with alpha, which no source gives.
10. The default fill colours `#e9c46a` and `#2a9d8f` (`DEC-7`). Other defaults change the table of Section 4.6 and the help text.
11. The limit of 256 threads (Section 4.6). The design bounds the threads that one call starts. The alternative is no limit beyond `u32`, which lets a caller start as many threads as the system allows.
12. The diagnostic of a failed write does not name the path (CLI-26). The design keeps untrusted text out of the terminal. The alternative repeats the path with the replacement of Section 6.2, which helps a user who runs several commands at once.
13. `--set` on `render` only (Section 4.5). `info` then describes only the default shape, and the library tests of `escher-domain` cover the other shapes. The alternative also gives `info` the option, which needs a new failure path in `info` and an exception to CLI-17.
14. The `Parameter` line, such as `Parameter: aspect, range 0.125 to 8, default 1`, between the deformation dimension and the `Domain` line (Section 5.1). Other forms change the snapshots.
15. The options of the ball (Section 4.6). `--rotation` takes the yaw, the pitch, and the roll in degrees, `--light` takes `X,Y,Z`, and `--shading` takes `phong` or `none`, with the defaults of `DEC-16`. Points 24 to 29 of Section 12.3 of `docs/design/render.md` hold their conventions and constants. The alternatives are other value forms, such as one option per angle.
16. A value that starts with `-` for `--rotation` and `--light` (CLI-31). The design accepts it, so `--light -1,1,2` works, and an option name directly after either option is read as its value. The alternative accepts only the form with `=`, such as `--light=-1,1,2`.
17. The options of the ball on a hyperbolic or Euclidean signature (Section 4.6). They are checked and have no effect. The alternative is an error.

## 12. Security Considerations

The threat model is a user who passes a signature, option values, or an output path from an untrusted party to `escher info` or `escher render`. The attacker can choose any argument within the limits of the command line, including invalid UTF-8, control characters, and very long text. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory or time through the signature or the options of `render`, injection of terminal escape sequences through the diagnostic, and the choice of the file that `render` overwrites. `escher-signature` sets the limits on the length of the signature, the number of features, and the size of an order, as Section 16 of `docs/design/signature.md` states. `escher-domain` limits the largest corner order of a domain, which bounds the cost of `verify()` and of every fold step, as Section 16 of `docs/design/domain.md` states. `escher-render` limits the image size, the number of pixels, the sample grid, the border width, and the fold cap, which bound the memory of the image and the work of each sample, as Section 13 of `docs/design/render.md` states. This crate adds one limit of its own, 256 threads (Section 4.6). `escher-domain` limits the value of each shape parameter to its range. The settings need no limit on their number or length, because each name is only compared, and a repeated or unknown name fails.

The diagnostics of `render` repeat no option value and no path (CLI-26). The echo line repeats at most 120 characters and replaces every character outside printable ASCII and the infinity sign (CLI-9). The caret line holds only spaces and `^`. The `Domain` line and the diagnostics of Section 6.7 hold only fixed text, numbers, and the canonical signature, which is printable ASCII. The help, version, and usage text that `clap` produces has every control character other than the line feed replaced with `?` (CLI-15).

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, and the workspace denies `unsafe_code`.
- Files: the crate reads no file. `info` writes no file. `render` writes one file, the output file, with the permissions of the user, and only after rendering and encoding succeed (CLI-21). It replaces an existing file without asking (`DEC-30`), follows a symbolic link at the path (Section 11.1), and creates no directory. A caller who passes a path from an untrusted party therefore lets that party choose which file of the user `render` overwrites, and checking the path is the caller's task.
- Threads: `render` starts at most 256 threads, in a pool that ends with the call (CLI-22).
- Build scripts: the crate has none.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones (Section 9.3). Only `insta` is a dev-dependency, and it never reaches the binary.

Attacks on the repository, the build machine, or the terminal itself are out of scope, because this document governs the crate and not access control or the terminal.

The residual risk is that a usage error still echoes every character of an unknown argument that is not a control character, including characters outside ASCII, such as the formatting characters of the Unicode general category Cf. The replacement tests only for the category Cc. A second residual risk is a domain that is wrong by less than the tolerances, which `--check` reports as passed (Section 16 of `docs/design/domain.md`).

The limits of `escher-render` bound memory, but at their maximum values together they allow a rendering that runs for a very long time, which only the user can stop. A write that fails after `std::fs::write` has opened the file, such as on a full disk, leaves the file truncated or partly written. Miri checks no code of `rayon` (Section 9.3).

## 13. Operational Considerations

The crate declares no Cargo feature and has no build script, so no environment variable affects the build. At run time the crate reads no environment variable, and, under the assumption of Section 11.1, `RAYON_NUM_THREADS` does not affect the pool of `render`. The snapshot tests read the variables of Section 8.2, which Section 14.2 of `docs/design/tooling.md` lists. The tests that write files use the directory `CARGO_TARGET_TMPDIR/escher-cli`, where Cargo sets `CARGO_TARGET_TMPDIR` for an integration test, and the smoke test uses the binary of `CARGO_BIN_EXE_escher`.

The supported platform is Linux. The minimum supported Rust version is the one that Technology Decisions of `AGENTS.md` states, and the dependencies need no later version (Section 9.2).

| Failure | Reported as | Recovery |
| --- | --- | --- |
| The command line is wrong | The `clap` message on standard error, with `Usage` | Correct the command line, with `escher --help` |
| The signature is malformed, over a limit, or a bad orbifold | The diagnostic of Section 6, with `Failure` | Correct the signature at the caret |
| The signature has no domain, without `--check` | The `Domain` line with `none`, with `Success` | None needed |
| The signature has no domain, with `--check` | The diagnostic of Section 6.7, with `Failure` | Run `info` without `--check`, or use a signature that `escher-domain` constructs |
| A check fails | The check lines with `failed` and the diagnostic of Section 6.7, with `Failure` | None. It is a defect of `escher-domain`, and the residual and the tolerance locate it |
| A write to standard output fails | The diagnostic of Section 6.6, with `Failure` | Fix the output destination |
| The reader of standard output closes the pipe | No diagnostic, with `Failure` | None needed |
| `render` gets a signature without a domain | The diagnostic of Section 6.8, with `Failure` | Use a signature that `escher-domain` constructs, which `escher info` shows in its `Domain` line |
| `render` gets an unknown parameter name, a value outside the range of its parameter, or a parameter set twice | The diagnostic of Section 6.8, with `Failure` | Use the names and ranges that `escher info` lists, each name once |
| `render` gets a bad colour or the wrong number of colours | The diagnostic of Section 6.8, with `Failure` | Give each colour as `#rgb` or `#rrggbb`, quoted, as many as the colouring needs |
| `render` gets an option value outside a limit of `escher-render`, a rotation angle that is not finite, or a light direction that is zero or not finite | The diagnostic of Section 6.8, with `Failure` | Choose a value within the limits of Section 4.1 of `docs/design/render.md` |
| `render` cannot write the output file | The diagnostic of Section 6.8, with `Failure` | Fix the path or its directory, and run the command again |
| Samples reach the fold cap | No error. `Unconverged samples` counts them with `--stats`, and they take the border colour (`DEC-7`) | Raise `--max-folds` |

The crate emits no log. `render --stats` prints the statistics of a rendering on request.

## 14. References

### 14.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 14.2. Informative References

- [CBG] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [CH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.

## Appendix A. Requirement Summary

This table lists every labelled requirement of this document with a one-line summary.

| Label | Section | Summary |
| --- | --- | --- |
| CLI-1 | 3.1 | The library target exposes `run` and `Status` and nothing else. |
| CLI-2 | 3.2 | `run` writes only to its two streams and the output file of `render`, and never panics. |
| CLI-3 | 3.4 | `main.rs` only calls `run` and converts the status. |
| CLI-4 | 4.2 | `--version` and `-V` print the package name and version on standard output with `Success`. |
| CLI-5 | 4.3 | `info` and `render` pass their argument unchanged to `escher-signature`. |
| CLI-6 | 5.1 | `info` prints exactly the lines of Section 5.1, in order and under their conditions, with no name of the group in another notation. |
| CLI-7 | 5.4 | A change to the information text updates its snapshots in the same change. |
| CLI-8 | 6.1 | Every diagnostic goes to standard error. |
| CLI-9 | 6.2 | The echo line replaces unsafe characters with `?` and holds at most 120 characters. |
| CLI-10 | 6.6 | A failed write to standard output gives `Failure`, with no diagnostic for a broken pipe on standard output. |
| CLI-11 | 7 | Every signature error, including a bad orbifold, gives `Failure`. |
| CLI-12 | 8.2 | No recipe, hook, or workflow sets `INSTA_FORCE_PASS`. |
| CLI-13 | 8.3 | Every test that uses `insta` is ignored under Miri with a reason. |
| CLI-14 | 9.3 | The dev-dependencies stay inside the dependency policy, so cargo-deny lists `insta` and its dependencies. |
| CLI-15 | 6.5 | The help, version, and usage text of `clap` has every control character other than the line feed replaced with `?`. |
| CLI-16 | 5.5 | The domain summary and the check lines take their values from `escher-domain`, which this crate does not measure or check again. |
| CLI-17 | 5.5 | Without `--check`, a valid signature without a domain prints `Domain: none, ...` and gives `Success`. |
| CLI-18 | 5.6 | `--check` prints each check with its result, residual, and tolerance, in order, and then the overall result. |
| CLI-19 | 7 | `info --check` gives `Failure` for a failed check or a missing domain, and `Success` only when every check passed. |
| CLI-20 | 8.1 | No snapshot holds a residual, and the snapshot inputs keep the printed digits away from rounding boundaries. |
| CLI-21 | 4.5 | `render` creates or changes the output file only after rendering and encoding succeed. |
| CLI-22 | 4.5 | `render` builds its own thread pool for each call, never the global pool. |
| CLI-23 | 4.5 | The bytes of the output file do not depend on the number of threads. |
| CLI-24 | 4.6 | `render` passes the size, grid, border width, fold cap, light direction, and rotation in radians to `escher-render` unchanged and checks no limit of them itself. |
| CLI-25 | 4.7 | `render` writes to standard output only the statistics lines, with `--stats`, after the output file is written. |
| CLI-26 | 6.8 | No diagnostic of `render` repeats an option value or the output path. |
| CLI-27 | 8.3 | Every test that reaches the thread pool is ignored under Miri with a reason. |
| CLI-28 | 8.1 | No snapshot holds the value of a statistics line other than `Samples`. |
| CLI-29 | 4.5 | `render` passes every setting to `escher-domain` and checks no name or value itself, except a name that two settings give. |
| CLI-30 | 4.5 | Withdrawn on 2026-11-17. |
| CLI-31 | 4.6 | `--rotation` and `--light` accept a value that starts with `-`. |
