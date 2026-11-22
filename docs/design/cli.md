# Command-Line Interface Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-22 |
| Applies to | `crates/escher-cli`: the library target, the binary `escher`, and its tests |

## Abstract

This document specifies the package `escher-cli`, which provides the command-line program `escher`. It defines the public surface of the library target, the thin binary that calls it, the commands and options, the exact text that the command `info` prints for each kind of signature, including the shape parameters, the summary of the fundamental domain, the number of colourings for each colour count with the options `--max-colours` and `--colourings`, the result of each check of the option `--check`, and the presentations and the group order by coset enumeration of the option `--presentation`, the options of the command `render`, including the settings of shape parameters, the colouring by an id with its default colours, the PNG file that it writes and the statistics that it prints, the format of every diagnostic, the exit statuses, the tests, and the dependencies.

## 1. Introduction

### 1.1. Problem

The proof of concept is a command-line tool, and its first command reports what a signature in orbifold notation describes. The tool reads untrusted text, prints text that a reviewer checks line by line, and must report every malformed signature with a pointer to the offending characters. A program whose logic lives in `main` can only be tested by starting a process, which is slow.

### 1.2. Purpose

This document fixes the behaviour of `escher` precisely enough that an agent can implement it and a reviewer can check every line of output against it. The crate `escher-signature` parses and classifies a signature, the crate `escher-domain` builds and verifies its fundamental domain, the crate `escher-group` derives presentations, enumerates their cosets, and searches their colourings, and the crate `escher-render` renders the domain into pixels and encodes them as PNG. This crate turns their results into text, a file, diagnostics, and an exit status.

### 1.3. Audience

The primary readers are the coding agents that implement or extend `escher-cli`. The secondary readers are reviewers and the owner, who decides the points of Section 11.3.

### 1.4. Scope

This document covers the command `info` with its argument and its options `--check`, `--presentation`, `--colourings`, and `--max-colours`, the command `render` with its argument and its options, the options `--help` and `--version`, the diagnostics, the exit statuses, the tests of the crate, and its dependencies. `info` parses, validates, canonicalizes, and classifies any signature. It prints the shape parameters of the signature. For a signature that `escher-domain` constructs, it also prints a summary of the fundamental domain, the number of its colourings for each colour count, with `--colourings` every colouring with its id, and with `--check` the result of each check of its verification. With `--presentation`, it prints the Conway presentation of the signature and the side presentation of its domain, and for a spherical signature the group order that coset enumeration finds for each.

`render` draws, with `escher-render`, the tiling of a signature that `escher-domain` constructs with the shape parameters that the option `--set` gives, in one colour, in two colours by parity, or in the colouring that an id names. It draws the tiling in the Poincare disk for a hyperbolic domain, in the Euclidean plane for a Euclidean one, and as a lit ball for a spherical one, with the viewing rotation, the light, and the shading of its options, and writes it to a PNG file. With `--stats`, it also prints statistics of the rendering.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The grammar of a signature, its canonical form, the classification methods, and the error kinds of `escher-signature` with their `Display` text are specified in `docs/design/signature.md`.
- The signatures that have a fundamental domain, its construction, its measurements, the checks of its verification with their tolerances, the shape parameters with their names, ranges, and defaults, and the error kinds of `escher-domain` with their `Display` text are specified in `docs/design/domain.md`.
- The side presentation of a domain, its generators, and its relators are specified in `docs/design/domain.md`.
- Words, relators, presentations and their text, the Conway presentation of a signature, coset enumeration with its limit, the search of the colourings with its limits, the text and the order of a colouring id (GRP-37), `colour_of` (GRP-38), and the error kinds of `escher-group` with their `Display` text are specified in `docs/design/group.md`.
- The coset fill that draws a colouring is specified in Section 5.12 of `docs/design/render.md`.
- The rendering, the limits on the render options, the statistics that a rendering counts, the PNG encoding, and the error kinds of `escher-render` with their `Display` text are specified in `docs/design/render.md`.
- The crates, the data flow, the error model, and the owner's decisions are specified in `docs/design/architecture.md`.
- The mathematics is stated in `docs/research/orbifold-notation.md` and, for rendering, in `docs/research/rendering.md`.
- The recipes, the nextest profiles, the hooks, and the dependency policy are specified in `docs/design/tooling.md`.

The options of `render` that later milestones add, such as `--model` in M12, belong to those milestones (Section 3.4 of `docs/design/poc-plan.md`).

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
- domain summary: The lines of the information text that describe the domain, rows 11 to 16 of Section 5.1 (Section 5.5).
- check: One numeric condition that `verify()` of `escher-domain` evaluates, with a residual and a tolerance (Section 2.1 of `docs/design/domain.md`).
- colouring: A colouring of the side presentation of the domain, as `escher-group` finds it: one subgroup of finite index $k$ with its coset table, whose $k$ cosets give the $k$ colours of the tiles (Section 5.12 of `docs/design/render.md`). Every subgroup has a colouring of its own, conjugate subgroups included.
- colour count: The number $k$ of colours of a colouring, the index of its subgroup.
- colouring id: The name of a colouring, its colour count in decimal followed by lower-case letters, such as `3a`, which GRP-37 of `docs/design/group.md` defines with the order of the ids.
- colour limit: The largest colour count $N$ that `info` searches, set by `--max-colours` (Section 5.8).
- search limit: A limit of the search of `escher-group` other than the colour limit, which is the step budget or the limit on the colourings that one call stores (GRP-35 and GRP-36 of `docs/design/group.md`).
- colouring lines: The lines of the information text that describe the colourings, rows 17 to 19 of Section 5.1 (Section 5.8).
- parity colouring: The colouring with 2 colours in which the generator of side $i$ moves coset 0 exactly when side $i$ is a reversing side. It exists exactly when the domain has a reversing side (Section 4.6).
- default colours: The table of 12 colours of CLI-44, from which a colouring takes its fill colours when `--colours` is absent.
- presentation lines: The lines of the information text that `--presentation` adds, rows 20 to 27 of Section 5.1 (Section 5.7).
- verification lines: The lines of the information text that `--check` adds, rows 28 to 34 of Section 5.1 (Section 5.6).
- Conway presentation: The presentation that `conway_presentation` of `escher-group` reads from the signature (Section 5 of `docs/design/group.md`).
- side presentation: The presentation that `side_presentation()` of `escher-domain` reads from the sides of the domain (Section 5.4 of `docs/design/domain.md`).
- order line: Row 23 or row 27 of Section 5.1, which reports the number of cosets of the trivial subgroup that coset enumeration finds for a presentation (Section 5.7).
- coset limit: `CosetLimit::DEFAULT` of `escher-group`, the largest number of cosets that one enumeration of `info` defines, written $L$.
- reversing side: A side of the domain whose `orientation()` is `Orientation::Reversing`. A domain has a reversing side exactly when its `reverses_orientation()` is true, which is the one call that `info` and `render` make to decide (Sections 3.1 and 3.2 of `docs/design/domain.md`).
- reflection domain: A domain whose every side has the kind `Mirror`, as Section 2.1 of `docs/design/render.md` defines it.
- rotation domain: A domain with no side of the kind `Mirror`, which `escher-domain` builds for a signature without reflections, such as `732`, as Section 2.1 of `docs/design/render.md` defines it.
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

The binary needs a library target so that tests can run every command in the test process, with in-memory streams, and observe its status and output. A test of a binary alone has to start a process, which is slow.

### 3.2. The `run` Function

`args` holds the program name first, as `std::env::args_os()` gives it. `run` is a thin generic wrapper that collects `args` into a `Vec<std::ffi::OsString>` and calls a private non-generic function that takes `&mut dyn std::io::Write` for each stream, as rule API-13 of `docs/rust-coding-standards.md` requires.

CLI-2. `run` MUST write only to the two streams it is given and to the output file of `render`, and MUST NOT panic on any input or any output error, or else a test of `run` cannot observe all output, and one malformed argument or closed pipe aborts the program instead of giving a status.

`run` returns no error. Inside the crate, `info` returns a `Vec<anyhow::Error>` that holds its errors in the order of their diagnostics, and `render` returns `anyhow::Result<()>`. `run` passes each error, in that order, to one private function that writes the diagnostic of Section 6, and returns `Failure` when there is at least one error and `Success` otherwise. So `info --check --presentation` can report the order diagnostic of Section 6.9 and then the verification diagnostic of Section 6.7. No `anyhow::Error` therefore appears in a public signature, which rule ERR-2 of `docs/rust-coding-standards.md` forbids.

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

The library target has the private modules `args`, `info`, `render`, and `diagnostic`, as files named `args.rs`, `info.rs`, `render.rs`, and `diagnostic.rs` (Section 4.5 of `docs/rust-coding-standards.md`). `args` holds the `clap` types and the parsers of `--size`, `--set`, `--colouring`, and the triples of `--rotation` and `--light`, `info` builds the information text, including the domain summary, the colouring lines, the presentation lines, and the verification lines, `render` runs the steps of Section 4.5 with the parser of colours, the default colours, the resolution of a colouring id, and the statistics lines, and `diagnostic` writes the diagnostics.

## 4. Command Line

### 4.1. Synopsis

```text
escher info [--check] [--presentation] [--colourings] [--max-colours <N>] <SIGNATURE>
escher render [OPTIONS] --output <FILE> <SIGNATURE>
escher --help
escher --version
```

`-h` is the short form of `--help` and `-V` of `--version`. `escher info --help` prints the help of `info`, and `escher render --help` the help of `render`. `--check`, `--presentation`, `--colourings`, and `--max-colours` have no short form. `-o` is the short form of `--output`, and no other option of `render` has a short form. The usage line in the help of `info` is the form that `clap` generates, `escher info [OPTIONS] <SIGNATURE>`, and that of `render` is `escher render [OPTIONS] --output <FILE> <SIGNATURE>`.

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

The help text has these summaries, the long help of `--set` below the table, and no other text beyond what `clap` generates:

| Item | Summary |
| --- | --- |
| `escher` | The package description, `Command-line program of Escher, a tool for symmetry-based graphics.` |
| `info` | `Print the canonical form, the geometry, and other properties of a signature in orbifold notation` |
| `SIGNATURE` | `The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'` |
| `--check` | `Verify the fundamental domain of the signature and print the result of each check` |
| `--presentation` | `Print the generators and relators of the signature and of its fundamental domain, and for a spherical signature the group order by coset enumeration` |
| `--colourings` | `Print every colouring of the fundamental domain with its id, up to the largest number of colours` |
| `--max-colours` | `The largest number of colours of the colourings to count and list, from 1 to 64` |
| `render` | `Render the tiling of a signature in orbifold notation and write it to a PNG file` |
| `--output` | `The PNG file to write. An existing file is replaced` |
| `--size` | `The image size in pixels, as WIDTHxHEIGHT` |
| `--colouring` | `One colour for every tile, two colours by parity, or the id of a colouring that the command info --colourings lists, such as 3a. The default is parity when a side pairing of the domain reverses orientation, and one otherwise` |
| `--colours` | `The fill colours, separated by commas, each as #rgb or #rrggbb, one for each colour of the colouring. The default takes the first colours of a built-in list of 12. In a POSIX shell, put the colours in single quotes, as in '#e9c46a,#2a9d8f'` |
| `--border-width` | `The border width in pixels. 0 draws no border` |
| `--border-colour` | `The border colour, as #rgb or #rrggbb` |
| `--samples` | `The number of samples along each side of a pixel. The pixel averages N by N samples` |
| `--max-folds` | `The largest number of folds of one sample. A sample that needs more takes the border colour` |
| `--background` | `The colour outside the disk of a hyperbolic tiling or the ball of a spherical tiling, as #rgb or #rrggbb` |
| `--threads` | `The number of threads, from 1 to 256. The default is the available parallelism, at most 256` |
| `--stats` | `Print statistics of the rendering to standard output` |
| `--set` | `Set a shape parameter of the tiles, as NAME=VALUE, such as aspect=1.5, once for each parameter. --help lists the parameters, and info <SIGNATURE> prints those of a signature` |
| `--rotation` | `The rotation of the ball of a spherical tiling, as three angles in degrees. YAW turns the front of the ball to the right, PITCH turns it up, and ROLL turns the image counterclockwise, in this order` |
| `--light` | `The direction from the ball of a spherical tiling to the light, as X,Y,Z, with X to the right, Y up, and Z toward the viewer. The length is ignored` |
| `--shading` | `Phong shading of the ball of a spherical tiling, or none` |

The help of `render` uses the summary of `SIGNATURE` above. It also shows `[default: <value>]` for each option with a fixed default in the table of Section 4.6, which excludes `--colouring` and `--colours`, whose defaults depend on the domain, and `[possible values: phong, none]` for `--shading`, which `clap` generates (Section 11.1). `--colouring` shows no possible values, because its private value parser accepts ids (CLI-42). The help of `info` shows `[default: 12]` for `--max-colours`. The argument struct declares `--rotation`, `--light`, and `--shading` after `--set`, so they come last in the help.

`--set` is the one option with a long help, the constant `SET_LONG_HELP` in `args`. It lists each shape parameter of `escher-domain` with the signatures that take it: `aspect`, `skew`, `cornerpair<k>.length`, and `slice<i>.length`. It then says that every other signature has no shape parameter, that `escher info <SIGNATURE>` prints one `Parameter` line for each parameter with its range and default (Section 5.1), and gives these example commands:

```sh
escher render o --set aspect=2 --set skew=0.25 -o /tmp/torus.png
escher render '*3333' --set slice1.length=2 -o /tmp/square.png
```

Unit tests in `args` check the parameter names and the signatures of `aspect` and `skew` in the long help against `parameters` of `escher-domain`.

Because one option has a long help, `clap` prints the help of `render` in two layouts. `escher render --help` puts each option on its own line, its help on the lines below, and the long help of `--set`. `escher render -h` puts each option on one line, with the short help of `--set` from the table above.

CLI-4. `escher --version` and `escher -V` MUST write the package name, one space, the package version, and a newline to standard output, which is `escher-cli 0.1.0` for version 0.1.0, and return `Success`, because Section 5.1 of `docs/design/poc-plan.md` keeps this output, and a script may read it.

### 4.3. The `info` Command

`info` takes one required positional argument, the signature, as a `String` named `SIGNATURE`, the optional flags `--check`, `--presentation`, and `--colourings`, and the optional option `--max-colours <N>`, in any order. An argument that is not valid UTF-8 is a usage error. Each flag and option may stand before or after the signature. A repeated `--check`, `--presentation`, `--colourings`, or `--max-colours` is a usage error, because `clap` rejects a flag or an option that is given more than once, and `info` keeps that behaviour. The argument struct declares `--presentation` after `--check`, then `--colourings`, then `--max-colours`, so they come in that order in the help.

`--max-colours` takes a decimal integer from 1 to `IndexLimit::MAX` of `escher-group`, which is 64 (Section 7.12 of `docs/design/group.md`), with the default 12. A private value parser in `args` reads the value as ASCII decimal digits only, as `--size` does, and passes the number to `IndexLimit::new`, so `clap` holds an `IndexLimit` and the range is checked once, in `escher-group`. Every other value, such as `0`, `65`, `x`, `+5`, or ` 5`, is a usage error (Section 6.5) with one message, whose upper end the parser formats from `IndexLimit::MAX`:

```text
error: invalid value '65' for '--max-colours <N>': expected a decimal integer from 1 to 64
```

The default is the literal `12` in the argument struct, because `IndexLimit` has no `Display` text, and a unit test asserts that it parses to `IndexLimit::DEFAULT`. The colour limit and `--colourings` change only the colouring lines (Section 5.8), and have no effect on a signature without a domain.

CLI-5. `info` and `render` MUST pass their argument unchanged to the `FromStr` implementation of `Signature` from `escher-signature`, and MUST NOT trim, normalize, or check it first, because a second reading of the grammar diverges from the one that `docs/design/signature.md` specifies, and the byte spans of the error then no longer match the argument.

On success, `info` writes the information text of Section 5 to standard output. On a signature error, it writes the diagnostic of Section 6 to standard error and nothing to standard output, with or without `--check`. A bad orbifold is a signature error (`DEC-24`).

`info` takes no setting. It builds the domain with `FundamentalDomain::build`, which gives each shape parameter its default, so the domain summary and the verification lines describe the default shape.

When a domain is built, `info` always reads its side presentation and searches its colourings up to the colour limit, and adds the colouring lines of Section 5.8 after the domain summary. With `--colourings`, those lines also name every colouring.

With `--presentation`, `info` also reads the Conway presentation of the signature and the side presentation of its domain, enumerates the cosets of each for a spherical signature within the coset limit, adds the presentation lines of Section 5.7 to the information text, and writes the diagnostic of Section 6.9 when an enumeration finds an order other than the group order.

With `--check`, `info` also runs `verify()` on the domain, adds the verification lines of Section 5.6 to the information text, and writes the diagnostic of Section 6.7 when a check fails or the signature has no domain. With both flags, the presentation lines come before the verification lines. The colouring lines come before both.

The `clap` argument struct holds each flag as a `bool`, because the derive API sets a flag through a `bool` field. `info` converts each at once into a private enum with two variants, which the private functions take (TYPE-2 of `docs/rust-coding-standards.md`).

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

This command writes an image of `*642` in its colouring `3a`, which has 3 colours, with three colours of its own:

```sh
escher render '*642' --colouring 3a --colours '#dc143c,#ffd700,#000080' -o three.png
```

Several Euclidean signatures share their default picture, because their tiles and borders coincide and the parity colouring sees only the orientation-preserving subgroup (Section 5.14 of `docs/design/render.md`, `DEC-55`). With the default colouring, `**` and `22*` give the same image, and `*x`, `22x`, and `*2222` give the same image, at the same `aspect`. With `--colouring one`, the six signatures `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` give the image of `*2222`, `4*2` that of `442`, and `3*3` that of `632`. These signatures differ in their colourings, so a user tells them apart with `--colouring` and an id that `escher info --colourings` lists.

`render` runs these steps in this order. It stops at the first step that fails and writes the diagnostic of Section 6.8, except at step 1, whose diagnostic is that of Section 6.1:

1. Parse the signature with the `FromStr` implementation of `Signature` (CLI-5).
2. Read the border colour and the background colour (Section 4.6).
3. Put the settings into a `ShapeParameters` of `escher-domain` in the order given, and stop at the first setting whose name an earlier setting already gave. Build the domain with `FundamentalDomain::build_with` and those settings, which checks each name and value. When `build_with` returns `UnknownParameter` or `ParameterOutOfRange`, find the first rejected setting with `parameters` and `ParameterSpec` of `escher-domain`, as CLI-29 states, and stop with the diagnostic of Section 6.8 that names it.
4. Choose the colouring: the value of `--colouring`, or the default of CLI-34. Stop when the colouring is `parity` and the domain has no reversing side (CLI-35). Then read the fill colours for that colouring, or take the default colours (Section 4.6). The number of colours comes from the colouring alone, so this step runs no search, also for a colouring id.
5. Check the render options: build the render options of `escher-render` from the size, the sample grid, a placeholder fill of one colour, the border colour, the border width, the background colour, the fold cap, the viewing rotation in radians, the light direction, and the shading, and drop the result. `escher-render` checks every value, for every geometry.
6. Build the fill: `Fill::One` for `one` and `Fill::Parity` for `parity`, with the colours of step 4. For a colouring id, resolve the id as CLI-43 states, then build the coset fill of `escher-render` from the domain, the colouring, and the colours. Then build the render options again with this fill and the same values, which gives no error once step 5 has passed.
7. Build a thread pool with the number of threads of `--threads`, or with the default of Section 4.6, which this step computes.
8. Build the renderer, then render the image on the thread pool, in bands of one row, each with the band operation of `escher-render`, and merge the statistics of the bands. The time of the rendering is the time of the statistics lines.
9. Encode the image as PNG with `escher-render`.
10. Write the PNG bytes to the output file (Section 4.7).
11. With `--stats`, write the statistics lines to standard output (Section 4.7).

Section 3 of `docs/design/render.md` names the operations of steps 5, 6, 8, and 9 and the statistics that step 8 merges.

The colouring depends on the domain, so step 4 comes after the domain is built in step 3. Steps 4 and 5 check every input that needs no search before step 6 resolves a colouring id. Such an input error is certain, while a search can take seconds and can stop at a limit, and the diagnostic of that limit claims nothing about the input. So a command with a colouring id and a bad colour, a wrong number of colours, more than 12 colours without `--colours`, or a bad option reports that error at once. It never waits for the search, and the diagnostic of a search that stopped at a limit never takes its place. A command with a bad value of `--colours` and a bad setting reports the setting, and a command with a bad `--border-colour` and a bad setting reports the colour.

CLI-35. When the colouring is `parity` and the domain has no reversing side, `render` MUST stop at step 4 with the diagnostic of Section 6.8 and write no file, because every element of the group of such a domain preserves orientation, so the parity of the reversals has one class only and no parity colouring exists for it (Section 5.7 of `docs/design/architecture.md`), and an image in one colour would hide that the requested colouring was not drawn (`DEC-32`).

To resolve a colouring id with the colour count $k$, step 6 calls `side_presentation()` on the domain. A render with a colouring id builds the side presentation three times: here, in `CosetFill::new`, and in `Renderer::new`, whose two calls Section 5.12 of `docs/design/render.md` states. Step 6 then calls `colourings_of_index` of `escher-group` with $k$ as its `IndexLimit` and the search budget, which searches only the colourings with exactly $k$ colours (GRP-35 of `docs/design/group.md`), and calls `find` with the id on the resulting `ColouringsOfIndex`. A private function takes the budget as a parameter. `render` passes `SearchBudget::DEFAULT` to it, so that a unit test can resolve an id with a small budget (Section 8.1). The outcomes are:

| Outcome | Result |
| --- | --- |
| `find` returns the colouring | Step 6 goes on with it |
| `is_complete()` is true, and `find` returns none | Stop, with the diagnostic of Section 6.8 that the colouring does not exist, which gives `list().len()`, the number of colourings with $k$ colours |
| `is_complete()` is false, because the search stopped at a limit | Stop, with the diagnostic of Section 6.8 that the search stopped at a limit, which claims nothing about the existence of the id |
| `side_presentation()` or `colourings_of_index` returns an error | Stop, with `error: cannot render <signature>` and a `caused by` line with the `Display` text of the error |

This resolution is the proposed default `DEC-44` of `docs/design/architecture.md`.

CLI-43. `render` MUST resolve a colouring id through the side presentation of the domain that it renders and `colourings_of_index` of its colour count, MUST run no search of another colour count, MUST stop at step 6 with a diagnostic and write no file when the colouring does not exist or the search reaches a search limit, and MUST NOT claim that an id does not exist when the search stopped at a limit, because a search of the lower colour counts costs time that the image does not need and can reach a limit before it reaches the id, the colours of the image come from the coset table of the domain that is drawn, and a statement that an id does not exist is true only of a search that finished.

The side presentation and the colouring belong to the domain that step 3 built with the settings of `--set`. The shape parameter of `*2222` does not change its side presentation, so its ids are the same for every value of `aspect`, and the same holds for the parameters of a reflection polygon and for `aspect` and `skew` of the wallpaper domains (Section 11.1).

CLI-21. `render` MUST NOT create, truncate, or otherwise change the output file before rendering and encoding have succeeded, because a failed `render` then leaves an existing file as it was and creates no empty file that a reader could take for an image.

CLI-22. `render` MUST build its own thread pool for each call, with exactly the number of threads of step 7, and MUST NOT use the global pool of `rayon`, because the global pool is shared by the whole process and is configured once, so two calls of `run` in one test process could not choose their own numbers, and the size of the pool would come from the environment instead of the option.

CLI-23. The bytes of the output file MUST NOT depend on the number of threads, because a user who changes `--threads` for speed expects the same image, and a difference reveals a defect in the split into bands. Section 3.2 of `docs/design/render.md` states that a band renders the same bytes as the same rows of the whole image.

CLI-29. `render` MUST pass every setting to `escher-domain` through `FundamentalDomain::build_with`, and MUST let only `build_with` decide whether the settings are accepted. It checks a name against another setting before the call, to report a name that two settings give. After `build_with` returns `UnknownParameter` or `ParameterOutOfRange`, it reads the names and ranges that `parameters` and `ParameterSpec` give, only to find the setting to name in the diagnostic. A second check that decided acceptance could differ from the specifications that `docs/design/domain.md` states, and a setting would then pass one check and fail the other.

The search for the rejected setting takes the settings in the order given. It first looks for a name that is not a parameter of the signature, and only when every name is one does it look for a value outside its range, so an unknown name is reported before a value outside its range. When it finds no such setting, `render` writes the diagnostic of `build_with` with its `Display` text, as for any other error kind. No input reaches that case, because `parameters` and the check of `build_with` take the parameters of a signature from the same list, test every name before any range, and use the same range test.

CLI-30. Withdrawn on 2026-11-17.

CLI-36. Withdrawn on 2026-11-17.

The `clap` argument struct holds `--stats` as a `bool`, which `render` converts at once into a private enum with two variants, as `info` does for `--check` (Section 4.3).

### 4.6. Option Values

`clap` checks the syntax of every option value, and a malformed value is a usage error (Section 6.5). `render` reads the border colour and the background colour itself in step 2, `escher-domain` checks the settings in step 3, `render` chooses the colouring and reads the fill colours in step 4, and `escher-render` checks the other values in step 5. This table gives the value syntax and the default of each option of `render`:

| Option | Value | Default |
| --- | --- | --- |
| `-o`, `--output <FILE>` | Any path | None. The option is required (`DEC-7`) |
| `--size <WIDTHxHEIGHT>` | Two decimal integers joined by a lower-case `x`, such as `1024x768`. Each integer has only the digits `0` to `9` and fits a `u32` | `512x512` |
| `--colouring <COLOURING>` | `one`, `parity`, or a colouring id (CLI-42) | `parity` when the domain has a reversing side, and `one` otherwise (CLI-34, `DEC-7`) |
| `--colours <COLOURS>` | Colours separated by commas, as many as the colouring needs: one for `one`, two for `parity`, and $k$ for a colouring id with the colour count $k$ (`DEC-29`) | The first default colours that the colouring needs (CLI-44): `#e9c46a` for `one`, `#e9c46a,#2a9d8f` for `parity` (`DEC-7`), and the first $k$ default colours for an id with $k$ at most 12 |
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

Step 2 reads `--border-colour` and then `--background`. Step 4 splits the value of `--colours` at each comma. It first compares the number of items with the number that the colouring needs, and then reads each item as a colour, in order. An empty item, such as the one that a trailing comma gives, is not a colour.

The `clap` argument struct holds `--colouring` as an `Option`, with no default of `clap`, and step 4 chooses the colouring when the option is absent.

A private value parser in `args` reads `--colouring` into a private enum with the variants one, parity, and an id. It accepts exactly `one` and `parity`, and otherwise reads the value with the `FromStr` implementation of `ColouringId` of `escher-group`, which accepts a colour count from 1 to `IndexLimit::MAX` in decimal with no leading zero, followed by 1 to 4 lower-case letters (GRP-37). No id equals `one` or `parity`, because an id starts with a digit. For any other value, such as `three`, `3`, `03a`, `3A`, or `65a`, the parser returns the message `expected one, parity, or a colouring id such as 3a`, which `clap` puts into its usage error.

CLI-42. `render` MUST accept as the value of `--colouring` exactly `one`, `parity`, and the text that the `FromStr` implementation of `ColouringId` accepts, MUST read an id through that implementation only, and MUST give a usage error with the message above for every other value, because a second reading of the id grammar in this crate could accept an id that `escher-group` does not, or the reverse, and an id that does not parse names no colouring of any signature.

A colouring id that parses can still name no colouring of the domain, such as `3d` of `*642`, which has 3 colourings with 3 colours. That is a render error of step 6 with the status `Failure` (CLI-43), and not a usage error, because whether it exists depends on the signature.

The aliases `one` and `parity` keep the fills `Fill::One` and `Fill::Parity` of `escher-render`, so the default colouring of CLI-34 runs no search. The ids `1a` and the id of the parity colouring name the same colourings by cosets:

CLI-48. `render` MUST draw `one` and `parity` with `Fill::One` and `Fill::Parity`, and with the same colours `--colouring 1a` MUST give the bytes of `--colouring one`, and the id of the parity colouring the bytes of `--colouring parity`, because a user who copies an id from the lines of `info` expects the picture of its alias, and `info` names the aliases in its `Colouring` lines (Section 5.8). The parity colouring is the colouring with 2 colours in which the generator of side $i$ moves coset 0 exactly when side $i$ is a reversing side (`UNV-18`).

Under the ids of GRP-37, the parity colouring is the last colouring with 2 colours of each reflection domain, and Section 7.15 of `docs/design/group.md` gives its id for each signature, such as `2g` for `*642`.

The default colours are a table of 12 colours in the module `render`, with no colour crate, as the owner decided (`DEC-29` and `DEC-39`):

| Entry | Colour | Entry | Colour |
| --- | --- | --- | --- |
| 1 | `#e9c46a` | 7 | `#774296` |
| 2 | `#2a9d8f` | 8 | `#c29810` |
| 3 | `#913671` | 9 | `#077da0` |
| 4 | `#91ab3b` | 10 | `#fe9cb5` |
| 5 | `#3b70c2` | 11 | `#519a47` |
| 6 | `#ffa289` | 12 | `#4f51aa` |

Entries 1 and 2 are the default fill colours of `DEC-7`. Entries 3 to 12 are values that this design proposes within `DEC-39`. They are chosen in the OKLCH colour space with a golden-angle step after the two fixed entries. The entries are not checked for colour-vision deficiency.

CLI-44. Without `--colours`, `render` MUST take the first $k$ entries of the table of default colours for a colouring with $k$ colours, and for a colouring with more than 12 colours MUST stop at step 4 with the diagnostic of Section 6.8 that asks for $k$ colours with `--colours`, and write no file, because the first two entries keep the images of `one` and `parity` as they are, and a table that cycles or a generated colour would give two cosets colours that a viewer cannot tell apart, while a prototype outside the repository found no 64 colours that are distinct in practice.

With `--colours`, `render` takes exactly as many colours as the colouring has, for every colour count, in the hexadecimal forms of `DEC-29`. A wrong number gives the diagnostic of the wrong number of colours with the id of the colouring, such as `the colouring 3a needs 3 colours, and 2 were given`.

CLI-34. Without `--colouring`, `render` MUST use the colouring `parity` when the domain has a reversing side, and `one` otherwise, because `DEC-7` and `DEC-44` make the parity colouring the default when one exists, and the default then runs no search, and the parity colouring exists only when some pairing reverses orientation (Section 5.7 of `docs/design/architecture.md`).

So `escher render 732 -o rot.png` draws every tile in `#e9c46a`, and `escher render '*732' -o tiling.png` draws two colours. The number of fill colours follows the colouring, so `escher render 732 --colours '#e9c46a,#2a9d8f' -o rot.png` fails at step 4, because the default colouring of `732` is `one`, which needs 1 colour.

`escher-render` limits the image size, the number of pixels, the sample grid, the border width, and the fold cap, and returns an error kind for a value outside its limits, such as a width of 0, a grid of 0, a border width that is negative or not finite, a rotation angle that is not finite, or a light direction that is zero or not finite. Section 4.1 of `docs/design/render.md` states each limit and the error kind for a value outside it. `render` reports such an error as Section 6.8 states.

CLI-24. `render` MUST pass the size, the sample grid, the border width, the fold cap, the components of the light direction, and the angles of the viewing rotation after their conversion to radians to `escher-render` unchanged, and MUST NOT check them against a limit itself, because a second check in this crate can differ from the limits that `docs/design/render.md` states, and a value would then pass one check and fail the other.

The limit of 256 threads is this crate's own, because the thread pool belongs to this crate (Section 3.5 of `docs/design/architecture.md`). The default never exceeds it.

### 4.7. Output of `render`

The output file holds the PNG bytes that `escher-render` encodes in step 9, with the size of `--size`. Section 5.6 of `docs/design/render.md` states the colour type, the bit depth, and the chunks. Step 10 writes the bytes with `std::fs::write`, which creates the file when it does not exist and truncates and replaces it when it does. `render` replaces an existing file without asking and has no option to refuse it (`DEC-30`). It creates no directory.

Without `--stats`, a successful `render` writes nothing to standard output or standard error. With `--stats`, step 11 writes the lines of this table, in this order, through the helper of Section 3.2. Each line is the label, a colon, one space, and the value, followed by a line feed, as in Section 5.1. Every count is an integer of the statistics of `escher-render`, which Section 3.2 of `docs/design/render.md` states, and is written in decimal:

| Order | Label | Value |
| --- | --- | --- |
| 1 | `Samples` | The number of samples, including the outside samples of Section 2.1 of `docs/design/render.md` |
| 2 | `Mean folds` | The total number of folds divided by the number of inside samples, which are the samples that are not outside samples, rounded down to two digits after the point, or `0.00` when every sample is an outside sample |
| 3 | `Maximum folds` | The largest number of folds of one sample |
| 4 | `Unconverged samples` | The number of samples that reached the fold cap or met a value that is not finite |
| 5 | `Time` | The wall-clock time of the rendering of step 8 in seconds, measured with `std::time::Instant`, in fixed notation with three digits after the point, `{:.3}`, followed by one space and `s` |

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

`info` writes the lines of this table, in this order. Each line is the label, a colon, one space, and the value, followed by a line feed. No other line, blank line, or trailing space appears. The accessors of rows 1 to 9 are those of `Signature` (Sections 3.1 and 9.1 of `docs/design/signature.md`). Row 10 takes its values from `parameters` and `ParameterSpec`, rows 11 to 16 and 28 to 34 from `FundamentalDomain`, `Side`, `Verification`, and `Check` (Section 3.1 of `docs/design/domain.md`), rows 17 to 19 from the colouring search of `escher-group` on `side_presentation()` (Section 5.8), and rows 20 to 27 from `escher-group` and `side_presentation()` (Section 5.7). "A domain is built" means that `FundamentalDomain::build` returned `Ok`.

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
| 11 | `Domain` | Always | `no sides` when the domain has no side, `<n> mirror sides` when every side is a mirror, `<n> sides in <k> pairs` when no side is, with `pair` when $k = 1$, as in `2 sides in 1 pair`, `<n> sides, <m> mirror sides and <k> pairs` when the domain has both, and `none, ` followed by the `Display` text of the error when no domain is built (Section 5.5, CLI-51, CLI-53) |
| 12 | `Mirror sides`, then `Side pairs` | A domain with a paired side is built. The `Mirror sides` line appears only when that domain also has a mirror side | Two lines in this order. `Mirror sides` lists the mirror sides, such as `2 and 5`, and `Side pairs` each pair of partner sides once, such as `0 and 1 by rotation, 2 and 3 by rotation`, in the forms of Section 5.5 |
| 13 | `Parity colouring` | A domain is built and it has no reversing side | `none, every side pairing preserves orientation` |
| 14 | `Corner angles in degrees` | A domain with at least one side is built (CLI-53) | Each value of `corner_angles()` in degrees, in corner order, in the form of Section 5.2 |
| 15 | `Side lengths` | As row 14 | Each value of `side_lengths()`, in side order, in the form of Section 5.2 |
| 16 | `Domain area` | A domain is built | `area()`, in the form of Section 5.2 |
| 17 | `Colourings` | A domain is built | The number of colourings and the colour count to which the list is complete, or `none, ` followed by the `Display` text of the error, in the form of Section 5.8 |
| 18 | `Colourings with <k> colours` | Row 17 has a number, once for each colour count $k$ up to the colour count of row 17 that has at least one colouring, in increasing order | The number of colourings with $k$ colours, in the form of Section 5.8 |
| 19 | `Colouring` | `--colourings` is given and row 17 has a number, once for each colouring that row 18 counts, in the order of the ids | The id, the colour count, and an alias, in the form of Section 5.8 |
| 20 | `Presentation` | `--presentation` is given | The size of the Conway presentation, or `none, ` followed by the `Display` text of the error, in the form of Section 5.7 |
| 21 | `Generators` | `--presentation` is given and the Conway presentation exists | Its generator names, in the form of Section 5.7 |
| 22 | `Relators` | As row 21 | Its relators, in the form of Section 5.7 |
| 23 | `Order by coset enumeration` | As row 21, and the signature is spherical | The order line of the Conway presentation, in the form of Section 5.7 |
| 24 | `Side presentation` | `--presentation` is given and a domain is built | The size of the side presentation, or `none, ` followed by the `Display` text of the error |
| 25 | `Side generators` | `--presentation` is given and the side presentation exists | Its generator names |
| 26 | `Side relators` | As row 25 | Its relators |
| 27 | `Side order by coset enumeration` | As row 25, and the signature is spherical | The order line of the side presentation |
| 28 | `Check isometries` | `--check` is given and a domain is built | The result of `isometries()`, in the form of Section 5.6 |
| 29 | `Check pairings` | As row 28 | The result of `pairings()` |
| 30 | `Check walls` | As row 28 | The result of `walls()` |
| 31 | `Check relators` | As row 28 | The result of `relators()` |
| 32 | `Check angles` | As row 28 | The result of `angles()` |
| 33 | `Check area` | As row 28 | The result of `area()` of `Verification` |
| 34 | `Verification` | As row 28 | `passed` when `Verification::passed()` is true, and otherwise `failed` |

Row 12 holds two labels, so that the numbers of the other rows stay those that other sections cite.

CLI-6. `info` MUST print exactly the lines of this table, in this order and under these conditions, with the colouring lines for every built domain, the `Colouring` lines only with `--colourings`, the presentation lines only with `--presentation`, and the verification lines only with `--check`, because the snapshots of Section 8 and every reader of the text rely on one layout, and an extra or missing line changes what a reader concludes about the signature. The table has no name of the group in another notation, such as an IUC or Schoenflies name, because `DEC-20` omits them until a primary source is read.

So a signature without a domain gets the lines of rows 1 to 10 that apply to it, the `Domain` line of row 11, no colouring line, and with `--presentation` the lines of rows 20 to 23 that apply to it, with or without `--check`. Rows 28 to 34 follow the order of the checks of Section 8.2 of `docs/design/domain.md`.

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
| Corner angle | 14 | The value of `corner_angles()` in radians converted with `f64::to_degrees`, then written in fixed notation with six digits after the point, `{:.6}` | `25.714286` |
| Side length | 15 | Fixed notation with six digits after the point, `{:.6}` | `0.545275` |
| Domain area | 16 | Fixed notation with six digits after the point, `{:.6}` | `0.074800` |
| Residual and tolerance | 28 to 33 | Scientific notation with one digit after the point, `{:.1e}` of Rust, which writes an infinite value as `inf` | `2.5e-3`, `1.0e-9` |

Rows 14 and 15 separate the values with a comma and one space. Angles are in degrees on the command line, as Section 5.5 of `docs/design/architecture.md` states. The colouring lines and the presentation lines hold no floating-point value, and every count and order in them is an integer in decimal.

### 5.3. Examples

The snapshot files under `crates/escher-cli/tests/cli/snapshots/` fix the information text of every tested input, so this section shows only enough examples to show each kind of line. The examples follow the canonical form that `docs/design/signature.md` defines. The domain summary and the `Domain` line follow Section 5.5, the colouring lines Section 5.8, and the verification lines Section 5.6. Sections 5.7 and 5.8 show examples of the presentation lines and of the `Colouring` lines.

The option `--check` adds the verification lines after the domain summary and the colouring lines. This example shows a compact hyperbolic signature with a domain, with `<residual>` in place of each residual, because Section 8.1 replaces the residuals of these lines in the snapshot:

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
Colourings: 19, complete to 12 colours
Colourings with 1 colour: 1
Colourings with 2 colours: 1
Colourings with 8 colours: 8
Colourings with 9 colours: 9
Check isometries: passed, residual <residual>, tolerance 1.0e-9
Check pairings: passed, residual <residual>, tolerance 1.0e-9
Check walls: passed, residual <residual>, tolerance 1.0e-9
Check relators: passed, residual <residual>, tolerance 2.0e-6
Check angles: passed, residual <residual>, tolerance 1.0e-9
Check area: passed, residual <residual>, tolerance 1.0e-9
Verification: passed
```

Standard error is empty and the status is `Success`. The Euler characteristic comes from the defect rule of ON 6.1, the area from ON 7.3, and the dimension from ON 10.1. The tolerances are those of Section 8.4 of `docs/design/domain.md`. `Domain area` is the area of the triangle measured from its angles, and `Area` is the exact value from the Euler characteristic.

This example shows a spherical signature with a mirror boundary and a cone point, whose domain is a kite with mirror sides beside paired sides:

```sh
escher info '3*2'
```

```text
Signature: 3*2
Geometry: spherical
Euler characteristic: 1/12
Compact: yes
Group order: 24
Area: pi/6
Domain: 4 sides, 2 mirror sides and 1 pair
Mirror sides: 0 and 1
Side pairs: 2 and 3 by rotation
Corner angles in degrees: 90.000000, 90.000000, 120.000000, 90.000000
Side lengths: 0.785398, 0.785398, 0.615480, 0.615480
Domain area: 0.523599
Colourings: 25, complete to 12 colours
Colourings with 1 colour: 1
Colourings with 2 colours: 1
Colourings with 3 colours: 1
Colourings with 4 colours: 4
Colourings with 6 colours: 7
Colourings with 8 colours: 4
Colourings with 12 colours: 7
```

The table of ON 8.2 gives the order 24, and the Euler characteristic $2/24 = 1/12$ and the area come from ON 7.3. A spherical signature has no `Deformation dimension` line.

This example shows a Euclidean signature with two shape parameters and no reversing side pairing, in which `...` stands for the lines with 5 to 12 colours:

```sh
escher info o
```

```text
Signature: o
Geometry: Euclidean
Euler characteristic: 0
Compact: yes
Parameter: aspect, range 0.125 to 8, default 1
Parameter: skew, range -0.5 to 0.5, default 0
Domain: 4 sides in 2 pairs
Side pairs: 0 and 2 by translation, 1 and 3 by translation
Parity colouring: none, every side pairing preserves orientation
Corner angles in degrees: 90.000000, 90.000000, 90.000000, 90.000000
Side lengths: 1.000000, 1.000000, 1.000000, 1.000000
Domain area: 1.000000
Colourings: 127, complete to 12 colours
Colourings with 1 colour: 1
Colourings with 2 colours: 3
Colourings with 3 colours: 4
Colourings with 4 colours: 7
...
```

A Euclidean signature has no `Area` line, because `area_over_pi()` returns no value for it, and no `Deformation dimension` line. `info` takes no setting (Section 4.3), so it prints the default shape for every invocation.

With `--check`, a signature without a domain prints the same information text as without it, and then the diagnostic of Section 6.7:

```sh
escher info --check '4*3'
```

Standard output:

```text
Signature: 4*3
Geometry: hyperbolic
Euler characteristic: -1/12
Compact: yes
Area: pi/6
Deformation dimension: 0
Domain: none, cannot build fundamental domain: the signature 4*3 is not supported
```

Standard error:

```text
error: cannot verify fundamental domain
  caused by: cannot build fundamental domain: the signature 4*3 is not supported
```

The status is `Failure`, so the exit status is 1. Without `--check`, the same signature prints the same standard output, nothing to standard error, and the status `Success` (CLI-17).

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

An infinite order sorts above every finite order in the canonical form, so the canonical signature is `*(inf)22` (Section 11.3).

For a supported signature whose largest corner order is above the limit of `escher-domain`, such as `*(101)32`, rows 1 to 9 apply as for `*732`, and the last line is `Domain: none, cannot build fundamental domain: order 101 is larger than 100`, as Section 10.2 of `docs/design/domain.md` fixes it. A reflection polygon with more than 8 corners, such as `*222222222`, and its rotation signature, such as `222222222`, print no `Parameter` line, because `parameters` gives none for them, and their last line is `Domain: none, cannot build fundamental domain: the signature has 9 orders, more than 8`, with the number of orders in place of `9` (DOM-38 and Section 10.2 of `docs/design/domain.md`). That limit applies before the limit on the corner order, so `*(101)22222222` gets the same text. `2*n` has a limit of its own, $n$ up to 50 (DOM-63 of `docs/design/domain.md`, `DEC-59`), so `2*(51)` prints the group order 204 and the last line `Domain: none, cannot build fundamental domain: order 51 is larger than 50`, while `(101)*` and `(101)x` print the text of the limit of 100.

### 5.4. Stability and Omissions

The information text is not a stable interface, and a machine-readable form is a roadmap item (`DEC-23`).

CLI-7. A change that alters the information text MUST update its snapshots in the same change, because the snapshot test fails otherwise, and the snapshot diff is how a reviewer sees each change to the text (`DEC-23`, rule TEST-2 of `docs/rust-coding-standards.md`).

`info` prints presentations only with `--presentation` (Section 5.7). It prints the colouring lines for every built domain, and names the colourings only with `--colourings` (Section 5.8). The owner's goal that `info` reports how many polygons the representative set has, and so which colour counts are possible, is met by the `Colourings with <k> colours` lines: each colour count $k$ that they list is a number of colours that `render` can draw with an id. `info` prints no colour and no name of a colouring other than its id and its alias. The domain summary covers exactly the signatures that `escher-domain` constructs (Section 4 of `docs/design/domain.md`).

### 5.5. Domain Summary

For every valid signature, `info` calls `parameters` once and prints the parameter lines of row 10, then calls `FundamentalDomain::build` once and prints the `Domain` line of row 11. Rows 12 to 16 follow only when a domain is built, rows 12 and 13 only under their conditions.

When a domain is built, `info` counts its sides and its sides of kind `PairingKind::Mirror`, and the value of the `Domain` line takes one of four forms. The numbers are in decimal, and the number of pairs is the number of sides that are not mirrors, divided by 2, because every such side has a partner other than itself:

| Mirror sides | Form | Example |
| --- | --- | --- |
| No side exists | `no sides` | `no sides` for `1` |
| Every side | The number of `sides()`, one space, and `mirror sides` | `3 mirror sides` for `*732` |
| None | The number of `sides()`, ` sides in `, the number of pairs, and ` pairs` (` pair` when the number of pairs is 1) | `4 sides in 2 pairs` for `732` and for `xx` |
| Some | The number of `sides()`, ` sides, `, the number of mirror sides, ` mirror sides and `, the number of pairs, and ` pairs` (` mirror side` and ` pair` for the number 1) | `6 sides, 2 mirror sides and 2 pairs` for `22*` |

CLI-51. `info` MUST choose the form of the `Domain` line from the number of sides of kind `PairingKind::Mirror`, and MUST NOT choose it from `reverses_orientation()`, because `reverses_orientation()` is true for every one of the nine new Euclidean domains except that of `o` (Section 3.2 of `docs/design/domain.md`), so a summary chosen from it would print `4 mirror sides` for `xx` and for `22x`, whose sides are all paired, by glide reflections and translations in `xx` and by glide reflections alone in `22x`. When the domain has both mirror sides and paired sides, `info` MUST print the `Mirror sides` line before the `Side pairs` line, because a reader needs to know which sides are mirrors to read the pairs (`DEC-54`).

CLI-53. For a domain with no side, `info` MUST print `Domain: no sides` and MUST NOT print the `Corner angles in degrees` and `Side lengths` lines, because with the forms of CLI-51 the line would read `0 mirror sides`, which suggests mirrors that do not exist, and the two lists would be empty lines with a trailing space, which the snapshots of Section 8.1 would hold as text that no reader can use. For a side of kind `PairingKind::RotaryReflection`, the `Side pairs` line MUST name the kind `rotary reflection`, because the word `another isometry` tells a reader nothing about the pairing, and a rotary reflection, which reverses orientation, is what tells `nx` from `nn` (`DEC-54`, `DEC-57`).

The domain of `1` has no reversing side, so CLI-33 prints its `Parity colouring` line with the existing text `none, every side pairing preserves orientation`, and `render 1 --colouring parity` gives the diagnostic of CLI-35, which ends `because no side pairing reverses orientation`. Both texts are true of a domain with no side pairing, although they say nothing about it in particular. This design keeps both texts unchanged, which is the simplest true wording: it needs no branch for a domain with no side and changes no other text. Point 37 of Section 11.3 puts the alternative to the owner.

The value of the `Mirror sides` line lists the indices of the mirror sides in increasing order, in decimal: one index alone, two joined by ` and `, and three or more separated by a comma and one space, with `, and ` before the last, such as `2 and 5` for `22*` and `2, 3, and 4` for `2*22`.

CLI-32. For a domain with a paired side, `info` MUST print the `Domain` line with the number of pairs, and one `Side pairs` line that names each pair of partner sides once, because a reader of `<n> mirror sides` for such a domain would take its sides for mirrors, and the pairs and their kinds are what tell a group from another with the same tiles, such as a rotation subgroup from its reflection group, or `22*` from `**`.

The value of the `Side pairs` line lists one item for each side $i$ whose `partner()` $j$ is larger than $i$, in increasing order of $i$, separated by a comma and one space. An item is $i$ in decimal, ` and `, $j$ in decimal, ` by `, and the word for `kind()` of side $i$: `rotation` for `PairingKind::Rotation`, `translation` for `PairingKind::Translation`, `glide reflection` for `PairingKind::GlideReflection`, and `rotary reflection` for `PairingKind::RotaryReflection`, the kinds that Sections 3.1 and 3.2 of `docs/design/domain.md` define (DOM-27, DOM-61). For `732` the value is `0 and 1 by rotation, 2 and 3 by rotation`, for `2222` it is `0 and 1 by rotation, 2 and 5 by translation, 3 and 4 by rotation`, for `xx` it is `0 and 2 by glide reflection, 1 and 3 by translation`, and for `5x` and `x` it is `0 and 1 by rotary reflection`. The antipodal map of `x` has the kind `PairingKind::RotaryReflection` (`DEC-57`). A half-turn has the kind `PairingKind::Rotation`. `PairingKind` is `#[non_exhaustive]`, so the word is `reflection` for `PairingKind::Mirror` and `another isometry` for any later kind. No domain that `escher-domain` builds reaches either word, because a mirror is its own partner and appears in the `Mirror sides` line only.

`info` decides whether to print the `Parity colouring` line by `reverses_orientation()` of `FundamentalDomain` (Section 3.2 of `docs/design/domain.md`). Of the ten domains of Section 6.13 of `docs/design/domain.md`, only those of `o` and of `2222` print it. Of the spherical group domains of Section 6.16 of that document, only that of `1` prints it.

CLI-33. `info` MUST print the line `Parity colouring: none, every side pairing preserves orientation` exactly when a domain is built and it has no reversing side, because the parity colouring is the default of `render` whenever it exists (CLI-34), so a reader needs to learn from `info` when it does not exist, and printing the line only then leaves the text of every reflection domain unchanged.

When `build` returns an error, the value is `none, ` followed by the `Display` text of that error, for every error kind. The `Display` text starts with `cannot build fundamental domain: ` (Section 10.2 of `docs/design/domain.md`), so the line reads, for example, `Domain: none, cannot build fundamental domain: the signature 4*3 is not supported`. The line holds only the `Display` text of the top error and no text of its source. The text holds the canonical signature or a number, both in printable ASCII, so it needs no replacement of characters.

CLI-16. `info` MUST take the values of rows 10 to 16 and 28 to 34 from `escher-domain` unchanged, except for the conversion of the angles to degrees and the formats of Section 5.2, and MUST NOT measure or check the domain itself, because a second computation in this crate can differ from what `verify()` checks, and the printed values would then describe another domain than the one that passed.

CLI-17. Without `--check`, `info` MUST print the `Domain` line with `none` and return `Success` for a valid signature for which `build` returns an error, because the signature is still parsed, valid, and classified, and a non-zero status would tell a script that the signature itself is invalid.

The values of rows 14 to 16 are rounded to six digits after the point. Sections 9.3 to 9.6 of `docs/design/domain.md` bound the error of each value that `build` returns. A printed digit can therefore differ between two correct builds, such as builds on two platforms, only when the exact value lies within that bound of a rounding boundary of the sixth digit. Section 8.1 states how the snapshot inputs avoid that case.

### 5.6. Verification Lines

With `--check`, when a domain is built, `info` calls `verify()` once and prints rows 28 to 34 after the domain summary and the colouring lines and, with `--presentation`, after the presentation lines. The information text and the verification lines form one text, which `info` writes in one write.

The value of each of rows 28 to 33 has this form, where the result is `passed` when `Check::passed()` is true and `failed` otherwise, and the residual and the tolerance have the form of Section 5.2:

```text
<result>, residual <residual>, tolerance <tolerance>
```

For example, a failed relator check prints `Check relators: failed, residual 2.5e-3, tolerance 2.0e-6`. A residual of infinity, which marks a failed structural condition or a computation that gave no finite number (Section 8.1 of `docs/design/domain.md`), prints as `inf`. No residual is NaN.

Row 34 gives the overall result, `Verification: passed` or `Verification: failed`, from `Verification::passed()`.

CLI-18. With `--check`, `info` MUST print one line for each check that `Verification` reports, with its result, its residual, and its tolerance, in the order of Section 8.2 of `docs/design/domain.md`, and then the overall result, because a reader who sees only the overall result cannot tell which condition failed or by how much, and a line that omits the tolerance cannot be read without the design document.

### 5.7. Presentation Lines

With `--presentation`, `info` prints rows 20 to 27 after the colouring lines, or after the `Domain` line when no domain is built. It prints both presentations: the Conway presentation, which comes from the signature alone, and the side presentation, which comes from the domain, so that a reader can compare an independent reading of the group with the domain that `escher-domain` built. This is a proposed default (`DEC-35` of `docs/design/architecture.md`).

`info` takes these steps:

1. Call `conway_presentation` of `escher-group` on the signature once, and print row 20. When it returns a presentation, print rows 21 and 22, and for a spherical signature row 23.
2. When a domain is built, take its side presentation, which the colouring lines already read with one call of `side_presentation()` (Section 5.8), and print row 24. When it is a presentation, print rows 25 and 26, and for a spherical signature row 27.

The value of row 20 or row 24 is the number of generators in decimal, ` generators, `, the number of relators in decimal, and ` relators`, with `generator` when the number of generators is 1 and `relator` when the number of relators is 1, such as `3 generators, 6 relators`. When the call returns an error, the value is `none, ` followed by the `Display` text of that error, and the rows that depend on the presentation do not appear.

The value of row 21 or row 25 is the generator names in their order, separated by a comma and one space, or `none` when the presentation has no generator. The value of row 22 or row 26 is the text of each relator, in the order of the presentation, separated by a comma and one space, or `none` when it has no relator. The text of a relator is the one that `display_relator` of `escher-group` gives (Section 4.4 of `docs/design/group.md`, GRP-3), such as `(r1 r2)^5` or `(g1^-1)^3`, and the names are those of `conway_presentation` (Section 5.1 of `docs/design/group.md`, GRP-6) or of `side_presentation()` (DOM-32 of `docs/design/domain.md`). `display_relator` returns `None` only for a relator that uses a generator that the presentation lacks (Section 3.2 of `docs/design/group.md`), which no relator of the presentation itself does, so `info` is not expected to meet it. If it does, `info` prints none of the rows of that presentation, drops no relator, does not panic, and returns the internal error `internal error: a relator of the <subject> names a generator that it does not have`, with `presentation` or `side presentation` as the subject, which gives the status `Failure`. Its diagnostic stands beside any order diagnostic of the other presentation, in the order of Section 6.9. No input reaches this path, because `escher-group` cannot produce it, and only a unit test of the combinations of diagnostics covers its diagnostic. A name matches `[A-Za-z][A-Za-z0-9]*` (Section 4.3 of `docs/design/group.md`, GRP-5), so the lines need no replacement of characters.

CLI-37. With `--presentation`, `info` MUST print rows 20 to 27 under the conditions of Section 5.1 and in the forms of this section, and MUST print none of them without the flag, because a script that reads the text without the flag must not meet lines that the flag adds, and a reader compares the two presentations only when both appear in the stated order.

CLI-38. `info` MUST print the generator names and the relator texts that `escher-group` gives, unchanged, and MUST NOT reduce, reorder, or rewrite a relator, because the text is what a reader checks against the sources of the presentation, and a second writer of relator text in this crate could differ from the one that the tests of `escher-group` fix.

The order line of row 23 or row 27 reports one coset enumeration of the presentation over the trivial subgroup, with the coset limit $L$, `CosetLimit::DEFAULT` of `escher-group`, which Section 6.3 of `docs/design/group.md` states as 16384. $N$ is the value of `group_order()` of the signature, which is $2/\chi$ for a spherical signature (ON 7.3). The value of the line is chosen in this order:

| Case | Value | Status |
| --- | --- | --- |
| $N > L$ | `not run, the group order <N> is above the limit of <L> cosets`, and `info` runs no enumeration | Unchanged |
| `enumerate` returns a table whose `index()` is $N$ | `<N>, equal to 2/chi` | Unchanged |
| `enumerate` returns a table whose `index()` $K$ is not $N$ | `<K>, different from 2/chi, which is <N>` | `Failure`, with the diagnostic of Section 6.9 |
| `enumerate` returns the error kind `CosetLimitReached` | `not found within <L> cosets` | Unchanged |
| `enumerate` returns any other error | `not run, ` followed by the `Display` text of the error | Unchanged |

Each number is in decimal. "Unchanged" means that the line leaves the status as it would be without `--presentation`.

Every other error of `enumerate` comes from an input limit that `escher-group` checks before a run (Section 15 of `docs/design/group.md`, GRP-19), such as the limit on the letters of the expanded relators. The last row gives it the text `not run, ` and not `not found`, because no run took place. No input reaches that row, because a spherical Conway presentation of order $N \le L$ expands to at most $2N + 11$ letters (Section 15 of `docs/design/group.md`), which is below that limit, and a side presentation is short. So the design gives no example text for the row, and no test or snapshot covers it.

The first row is sound only because the limit counts every coset that a run defines, including coset 0, the coset of the subgroup itself (Section 6.3 of `docs/design/group.md`, GRP-16). A closed table over the trivial subgroup has one coset for each element, so its index is the group order (Sections 7.9 and 7.15 of `docs/research/colourings.md`), and no run that defines at most $L$ cosets can close a table of $N > L$ cosets. The run is skipped there, because it would only spend time and then report `not found`.

A run that reaches the limit shows nothing about the index, as the handbook of Holt, Eick, and O'Brien states (Section 7.15 of `docs/research/colourings.md`, Section 7.8 of `docs/design/group.md`, GRP-17), so the line says `not found` and claims nothing more. The enumerations of spherical presentations that Section 6.3 of `docs/design/group.md` records defined exactly $N$ cosets, so no spherical signature with $N \le L$ is expected to reach that line. No test asserts it, because no source bounds the cosets that an enumeration defines (Section 7.15 of `docs/research/colourings.md`).

CLI-39. `info` MUST print an order line only for a spherical signature, and MUST NOT run an enumeration when the group order is above the coset limit, because only a spherical signature has a group order to compare with, an enumeration of a Euclidean or hyperbolic group cannot finish, and a run that cannot finish within the limit costs time and shows nothing.

`info` prints an order line when `group_order()` of the signature returns a value, which it does exactly for a spherical signature (row 7).

CLI-41. `info` MUST enumerate with `CosetLimit::DEFAULT` and MUST offer no option that sets the limit, because the gate of CLI-39 and every text of this section depend on one known limit, and an option would add an input whose cost the design has not measured. The colour limit of `--max-colours` bounds the colouring search of Section 5.8 and has no effect on the coset limit.

The rows of a signature whose presentation `escher-group` cannot derive, such as one with an infinite order, have the text `Presentation: none, ` followed by the `Display` text of `UnsupportedSignature` of `escher-group`, which Section 8.2 of `docs/design/group.md` fixes (GRP-8 and GRP-22), such as `Presentation: none, cannot build presentation: the signature *(inf)22 has an infinite order`.

This example shows the lines that `--presentation` adds, after the lines of Section 5.3 and the colouring lines of Section 5.8, for a spherical signature:

```sh
escher info --presentation '*532'
```

```text
Presentation: 5 generators, 9 relators
Generators: r1, r2, r3, r4, l1
Relators: r1^2, (r1 r2)^5, r2^2, (r2 r3)^3, r3^2, (r3 r4)^2, r4^2, l1^-1 r1 l1 r4^-1, l1
Order by coset enumeration: 120, equal to 2/chi
Side presentation: 3 generators, 6 relators
Side generators: s0, s1, s2
Side relators: s0^2, s1^2, s2^2, (s0 s1)^5, (s1 s2)^3, (s2 s0)^2
Side order by coset enumeration: 120, equal to 2/chi
```

The boundary of `*532` gives the reflections `r1` to `r4` and the generator `l1`, with the relators of step 3 of ON 9.1, and the global relator `l1` of step 5, because the boundary is the only feature ([CH] pp. 255 to 257). The side presentation is that of Section 5.4 of `docs/design/domain.md`. A Euclidean or hyperbolic signature prints the same rows without the two order lines.

This example shows the gate of CLI-39. `*(9000)(9000)` is spherical with the group order 18000, above the limit, and its corner order is above the limit of `escher-domain`, so no side line appears:

```sh
escher info --presentation '*(9000)(9000)'
```

```text
Presentation: 4 generators, 7 relators
Generators: r1, r2, r3, l1
Relators: r1^2, (r1 r2)^9000, r2^2, (r2 r3)^9000, r3^2, l1^-1 r1 l1 r3^-1, l1
Order by coset enumeration: not run, the group order 18000 is above the limit of 16384 cosets
```

### 5.8. Colouring Lines

When a domain is built, `info` prints the colouring lines after the domain summary, with or without any flag, as the owner decided (`DEC-38`). Every subgroup of finite index is a colouring of its own, conjugates included (`DEC-37`). `info` takes these steps:

1. Call `side_presentation()` on the domain once. Section 5.7 reuses the result.
2. When it returns a presentation, call the search of `escher-group` on it once, with the colour limit $N$ of `--max-colours` as its `IndexLimit` and `SearchBudget::DEFAULT`, which no option changes (the rules of the search in `docs/design/group.md`). Without `--colourings` the call is `count_colourings`, which counts the colourings and stores none of them. With `--colourings` it is `colourings`, which returns them in the order of the ids. Either result is complete to a colour count $m$, its `complete_to()`, which is $N$ unless the search stopped at a search limit.
3. Print row 17. When it has a number, print one row 18 for each colour count from 1 to $m$ that has at least one colouring, and with `--colourings` one row 19 for each colouring of the list, in its order, which is the order of the ids of GRP-37 of `docs/design/group.md`.

The value of row 17 is chosen in this order:

| Case | Value |
| --- | --- |
| `side_presentation()` or the search returns an error | `none, ` followed by the `Display` text of the error |
| The search completed every colour count up to $N$ | `<n>, complete to <N> colours` |
| The search reached a search limit after it completed the colour count $m < N$ | `<n>, complete to <m> colours, the search stopped at a limit before <m + 1> colours` |

$n$ is the number of colourings with at most $m$ colours, and $m = N$ in the second row. Each number is in decimal, and the word is `colour` for the number 1, as in `complete to 1 colour` and `before 1 colour`. The third row also covers $m = 0$, which gives `0, complete to 0 colours, the search stopped at a limit before 1 colour`. One wording covers every search limit, the step budget and the limit on the stored colourings alike (GRP-35 and GRP-36 of `docs/design/group.md`), because the text claims only what the search completed.

The label of row 18 is `Colourings with `, the colour count $k$ in decimal, and ` colours`, with ` colour` for $k = 1$, such as `Colourings with 1 colour` and `Colourings with 8 colours`. Its value is the number of colourings with $k$ colours in decimal. A colour count without a colouring has no line, so the lines of row 18 together with row 17 state every colour count up to $m$.

The value of row 19 is the id, `, `, the colour count in decimal, and ` colours`, with ` colour` for 1, then `, alias one` for the colouring `1a` and `, alias parity` for the parity colouring of Section 4.6, such as `1a, 1 colour, alias one`, `2g, 2 colours, alias parity`, or `3a, 3 colours`. An id holds only a decimal number and lower-case letters (GRP-37), so the lines need no replacement of characters.

CLI-45. For every built domain, `info` MUST print row 17 and the rows 18 in the forms of this section, and with `--colourings` the rows 19, MUST take every count, id, and order of the ids from `escher-group` unchanged, and MUST mark exactly the colouring `1a` with `alias one` and exactly the parity colouring with `alias parity`, because the counts are the colour counts that `render` can draw, the ids are what a user passes to `render --colouring`, and a count or an order computed again in this crate could differ from the one that names the colouring in `render`.

CLI-46. `--max-colours` MUST set the colour limit of rows 17 to 19 together, with the default 12 and the range of Section 4.3, and with `--colourings` rows 17 and 18 MUST come from the same call as the rows 19, because the counts and the list then describe one search, and a count that came from another call could disagree with the list beside it. The colour limit changes no id: an id with $k$ colours is the same for every limit of at least $k$ (GRP-37, GRP-39).

`colourings` stops at its limit on the colourings that one call stores (GRP-36 of `docs/design/group.md`), and `count_colourings` stores none, so only its step budget stops it. With `--colourings`, row 17 can therefore name a lower colour count than without the flag, for the same signature and limit. Every colour count that both calls complete has the same number of colourings, because the result for each colour count up to `complete_to()` depends on the presentation alone (GRP-39).

Without `--colourings`, the numbers of rows 17 and 18 come from `ColouringCounts::colourings` of the call. With `--colourings`, they come from `Colourings::count` of the same call that gives the list, and `info` does not count the list again. Row 17 is the sum of the counts up to `complete_to()`.

CLI-47. When the search reaches a search limit, `info` MUST print the third form of row 17 and MUST print no line about a colour count above $m$, and every outcome of the colouring lines, including a search limit and an error, MUST leave the status as it would be without them, because a search that stopped at a limit shows nothing about the colour counts that it did not complete, as an enumeration that reaches its coset limit shows nothing about the index (Section 5.7), and the signature and its domain are still valid.

The colouring lines therefore never give `Failure`, and they write no diagnostic. A domain whose side presentation `escher-domain` refuses is not expected, because every domain that `build` returns has one (DOM-35 of `docs/design/domain.md`).

The ids depend on the numbering of the sides of the domain, because the columns of a coset table follow the generators of the side presentation. DOM-37 of `docs/design/domain.md` makes that numbering part of the contract of `escher-domain`.

This example lists the colourings of `*642` up to 3 colours. The lines of Section 5.3 come first, up to `Domain area`:

```sh
escher info --colourings --max-colours 3 '*642'
```

```text
Colourings: 11, complete to 3 colours
Colourings with 1 colour: 1
Colourings with 2 colours: 7
Colourings with 3 colours: 3
Colouring: 1a, 1 colour, alias one
Colouring: 2a, 2 colours
Colouring: 2b, 2 colours
Colouring: 2c, 2 colours
Colouring: 2d, 2 colours
Colouring: 2e, 2 colours
Colouring: 2f, 2 colours
Colouring: 2g, 2 colours, alias parity
Colouring: 3a, 3 colours
Colouring: 3b, 3 colours
Colouring: 3c, 3 colours
```

Singerman's seven colourings with 2 colours of `*642` (Section 6 of `docs/research/colourings.md`) and Bulatov's colouring with 3 colours (Section 4.4 of that document) appear here. Without `--colourings`, the same command prints only the lines up to the last `Colourings with` line.

A search that stops at a limit prints the third form of row 17:

```text
Colourings: <n>, complete to <m> colours, the search stopped at a limit before <m + 1> colours
```

The status stays as it is without the colouring lines (CLI-47). Up to 12 colours, such a stop happens for signatures with high corner orders, such as `*666` with `--colourings`, which reaches the limit on the stored colourings, and `*(100)(100)(100)`, which reaches the step budget. The reflection polygons have more sides, so their side presentations have more generators, and their searches reach the step budget at lower colour counts.

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

This example shows the caret after the infinity sign, which takes one column although it is three bytes. The span of `q` is bytes 6 to 7, and four characters precede it:

```sh
escher info '*22∞q'
```

```text
error: cannot parse orbifold signature: unexpected character 'q'
  *22∞q
      ^
```

### 6.5. Usage Errors

On an error of `try_parse_from`, `run` takes the text of `error.render()` as plain text and replaces each control character in it other than the line feed with `?`. When `error.use_stderr()` is false, which is the case for `--help` and `--version`, the text goes to standard output. Otherwise it goes to standard error. A usage error covers an unknown subcommand or option, a missing signature, a repeated `--check`, a repeated `--presentation`, a repeated `--colourings`, a repeated `--max-colours`, or another repeated option other than `--set`, an argument that is not UTF-8, `escher` with no argument, a missing `--output`, a value of `--max-colours` that the parser of Section 4.3 rejects, such as `0`, `65`, or `+5`, and an option value of `render` whose syntax Section 4.6 does not allow, such as a malformed triple, an unknown shading, or a value of `--colouring` that is neither `one`, `parity`, nor a colouring id (CLI-42). Section 7 gives the status of each case, and a failed write of the help or the version text follows Section 6.6.

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
| `--border-colour` is not a colour | 2 | `error: cannot render *732: --border-colour is not of the form #rgb or #rrggbb` |
| `--background` is not a colour | 2 | `error: cannot render *732: --background is not of the form #rgb or #rrggbb` |
| Two settings give the same name | 3 | `error: cannot render *2222: setting 2 of --set names the same parameter as setting 1`, with the position of the first setting whose name an earlier setting gave, and the position of that earlier setting, counted from 1 |
| `build_with` returns `UnsupportedSignature` | 3 | `error: cannot render 4*3: rendering this signature is not supported`, then `  caused by: ` and the `Display` text of the error of `escher-domain` |
| `build_with` returns `UnknownParameter` (CLI-29) | 3 | `error: cannot render *2222: the signature has no shape parameter foo, and its parameter is aspect`, with the name of the first setting that names no parameter of the signature, echoed as CLI-26 states. The end names the parameters of the signature in the order that `parameters` gives: `and its parameter is aspect` for one, `and its parameters are aspect and skew` for two, `and its parameters are cornerpair1.length, cornerpair2.length, cornerpair3.length, cornerpair4.length, and slice1.length` for more than two, and `nor any other` for a signature without shape parameters |
| `build_with` returns `ParameterOutOfRange` (CLI-29) | 3 | `error: cannot render *2222: the value 9 of the parameter aspect is outside its range of 0.125 to 8`, with the value of the first setting outside its range, printed from the parsed number, the name of its parameter, and the lower and upper ends of the range |
| `build_with` returns any other error kind, such as a shape of a reflection polygon that fails verification | 3 | `error: cannot render *(101)32`, then `  caused by: ` and the `Display` text of the error of `escher-domain` |
| The colouring is `parity` and the domain has no reversing side (CLI-35) | 4 | `error: cannot render 732: the colouring parity does not exist, because no side pairing reverses orientation` |
| A colouring id with more than 12 colours and no `--colours` (CLI-44) | 4 | `error: cannot render *732: the colouring 15a needs 15 colours, more than the 12 default colours, so give 15 colours with --colours`, with the id and, twice, its colour count |
| `--colours` has the wrong number of items | 4 | `error: cannot render *732: the colouring parity needs 2 colours, and 3 were given`, with the colouring, which is `one`, `parity`, or the id, the number it needs, and the number of items. The words are `1 colour` and `1 was given` for the number 1 |
| An item of `--colours` is not a colour | 4 | `error: cannot render *732: colour 2 of --colours is not of the form #rgb or #rrggbb`, with the position of the first such item, counted from 1 |
| `escher-render` rejects an option, including a rotation angle or a light direction | 5 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render` |
| `side_presentation()` returns an error for a colouring id | 6 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-domain` |
| The colouring id names no colouring of the domain (CLI-43) | 6 | `error: cannot render *642: the colouring 3d does not exist, because the signature has 3 colourings with 3 colours`, with the id, the number of colourings with its colour count, and that count. The words are `1 colouring with` and `no colouring with` for the numbers 1 and 0, and `1 colour` for the colour count 1 |
| The search for a colouring id reaches a search limit before it finds the id (CLI-43) | 6 | `error: cannot render *666: the search for the colouring 12a stopped at a limit before it found it`, with the id |
| `colourings_of_index` returns an error | 6 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-group` |
| `escher-render` rejects the coset fill of a colouring id | 6 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render`. No input reaches this row, because step 6 takes the colouring from the side presentation of the same domain, and step 4 checks the number of colours first |
| The thread pool cannot be built | 7 | `error: cannot render *732`, then `  caused by: cannot start the thread pool`, then `  caused by: ` and the `Display` text of the error of `rayon`. That text appears once, because the diagnostic takes it without the source of the error |
| `escher-render` returns an error while rendering or encoding | 8, 9 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render` |
| The output file cannot be written | 10 | `error: cannot write the output file`, then `  caused by: ` and the `Display` text of the `std::io::Error` |

Section 7.2 of `docs/design/render.md` fixes the `Display` text of each error of `escher-render`, and Section 10.2 of `docs/design/domain.md` that of `escher-domain`. No render error has an echo line or a caret line.

The row of `UnsupportedSignature` meets the requirement of Section 5.8 of `docs/design/architecture.md` that `render` names the signature and says that rendering it is not supported. It applies to every valid signature without a domain, including every general non-compact signature (`DEC-19`). The row of any other error kind covers a corner order over the limit of `escher-domain`, a signature with more than eight orders (DOM-38), a numerically degenerate construction, a polygon or a double of a polygon that fails `verify()` (DOM-44), a wallpaper domain that fails `verify()` (DOM-54), a spherical group domain that fails `verify()` (DOM-60), and the limit of 50 of `2*n` (DOM-63), which are not a lack of support. `escher-domain` checks the support of the signature before the settings, so a setting of a signature without a domain gives the row of `UnsupportedSignature` (DOM-18 in Section 6.6 of `docs/design/domain.md`), and not the row of `UnknownParameter`.

The rows of `UnknownParameter` and `ParameterOutOfRange` have texts of this crate, and not the `Display` text of the error of `escher-domain`, because that text names neither the rejected setting nor the parameters that the signature takes. A user who mistypes a name, or sets a parameter of another signature, sees what to type instead. The value is printed with the `Display` text of `f64`, so the value `9.0` prints as `9`, and a NaN or infinite value as `NaN`, `inf`, or `-inf`. The ends of the range are printed in the same way.

A domain that `build_with` refuses with the error kind `Numerical` gives the row of any other error kind, with the `caused by` line of Section 10.2 of `docs/design/domain.md`: `cannot build fundamental domain: the shape is too extreme to build accurately` for a shape of a reflection polygon, `cannot build fundamental domain: the side pairings of the domain cannot be stored accurately enough` for a double of a polygon, `cannot build fundamental domain: the wallpaper domain fails its verification` for a wallpaper domain, and `cannot build fundamental domain: the spherical domain fails its verification` for a spherical group domain. This crate adds no text of its own for these refusals or for the limit of `2*n`.

CLI-26. A diagnostic of `render` MUST NOT repeat the value of an option or the output path, because both are untrusted and may hold control characters, while the canonical signature, the option names, and the numbers that the diagnostics hold are printable ASCII. A bad colour is therefore named by its option and its position, and a failed write by the words `the output file`. There are three exceptions:

- A colouring id, which CLI-49 governs.
- The name of a setting that names no parameter of the signature (CLI-29). The diagnostic echoes it with every character outside printable ASCII, other than the infinity sign, replaced with `?`, as the echo line does (CLI-9). No echoed character is a control character, so the name cannot send an escape sequence to the terminal.
- The value of a setting outside the range of its parameter. The diagnostic prints the parsed `f64` with its `Display` text and never the text that the user gave. That text holds only digits, `-`, `.`, `NaN`, and `inf`, so it holds no character that a terminal interprets.

CLI-49. A diagnostic of `render` MUST name a colouring only by `one`, `parity`, or the `Display` text of the parsed `ColouringId`, and MUST NOT copy the text of `--colouring` as the user gave it, because the `Display` text of an id is a decimal number and lower-case letters that `escher-group` writes from its parsed value (GRP-37), so it holds no character that a terminal interprets, whatever the user typed. A value that does not parse is a usage error, whose text `clap` writes with the replacement of CLI-15. Every diagnostic of a colouring id therefore holds only the canonical signature, fixed words, numbers, and that `Display` text.

This example shows a signature without a domain:

```sh
escher render '4*3' -o tiling.png
```

```text
error: cannot render 4*3: rendering this signature is not supported
  caused by: cannot build fundamental domain: the signature 4*3 is not supported
```

This example shows the colouring `parity` for a signature whose domain has no reversing side:

```sh
escher render 732 --colouring parity -o rot.png
```

```text
error: cannot render 732: the colouring parity does not exist, because no side pairing reverses orientation
```

This example shows a setting of a signature without shape parameters. Every such signature, such as a spherical signature or the rotation signature of a reflection polygon (`DEC-50`), gives the same form:

```sh
escher render '*632' --set aspect=2 -o tiling.png
```

```text
error: cannot render *632: the signature has no shape parameter aspect, nor any other
```

This example shows a name that is not a parameter of a signature with two parameters. The names are compared exactly, so `Aspect` is not `aspect`:

```sh
escher render o --set Aspect=2 -o tiling.png
```

```text
error: cannot render o: the signature has no shape parameter Aspect, and its parameters are aspect and skew
```

This example shows a value outside the range of its parameter:

```sh
escher render '*2222' --set aspect=9 -o tiling.png
```

```text
error: cannot render *2222: the value 9 of the parameter aspect is outside its range of 0.125 to 8
```

This example shows a colouring id that names no colouring of the signature. `*642` has 3 colourings with 3 colours, `3a` to `3c`:

```sh
escher render '*642' --colouring 3d -o tiling.png
```

```text
error: cannot render *642: the colouring 3d does not exist, because the signature has 3 colourings with 3 colours
```

A malformed id, such as `--colouring 3A`, is a usage error with the message of Section 4.6 and the status `Usage`.

This example shows an output path in a directory that does not exist, on Linux, after a successful rendering:

```sh
escher render '*732' -o /tmp/missing/tiling.png
```

```text
error: cannot write the output file
  caused by: No such file or directory (os error 2)
```

Each of these diagnostics gives the status `Failure`. A failure before step 10 creates and changes no file (CLI-21). A write that fails after `std::fs::write` has opened the file can leave it truncated or partly written (Section 12). When `--stats` is given and the write of the statistics lines fails, Section 6.6 applies, and the output file stays written.

### 6.9. Order Errors

With `--presentation`, `info` writes the information text first, and then this diagnostic when an order line reports an index that differs from the group order (Section 5.7):

```text
error: cannot confirm group order: the presentation gives <K> cosets, and 2/chi is <N>
```

For the side presentation, the text reads `the side presentation gives` in place of `the presentation gives`. The diagnostic has no echo line and no caret line.

`info` writes one diagnostic for each presentation that reports a fault, the Conway presentation first, with one exception. A fault is a different index or the internal error of Section 5.7:

| Conway presentation | Side presentation | Diagnostics written |
| --- | --- | --- |
| Different index | Different index | The diagnostic of the Conway presentation only, because the information text already shows both values |
| Different index | Internal error | Both, the order diagnostic first |
| Internal error | Different index | Both, the internal error first |
| Internal error | Internal error | Both, that of the Conway presentation first |
| A fault | No fault | That of the Conway presentation |
| No fault | A fault | That of the side presentation |

No known input produces a different index, so a unit test reaches each row of the table through private functions.

With `--check` as well, `info` writes the diagnostics of this section first and then the diagnostic of Section 6.7, when there is one, each in the form of Section 6.1.

CLI-40. `info` MUST return `Failure` and write the diagnostic of this section when an enumeration of `--presentation` finds an index other than the group order, and every other outcome of the presentation lines MUST leave the status as it would be without the flag, because a different index means that a presentation, the enumeration, or the classification is wrong, and a script that reads the exit status must learn it, while a skipped run, a run that reaches the limit, a run that an input limit refuses, or a missing presentation shows no defect.

When the write of the information text fails, Section 6.6 applies, the status is `Failure`, and no order diagnostic is written.

## 7. Streams and Exit Status

`run` returns these statuses. Section 3.3 gives their exit statuses, and Sections 4.7, 6.5, 6.6, 6.7, 6.8, and 6.9 give the streams.

| Status | When |
| --- | --- |
| `Success` | `info` without `--check` wrote its text, whether or not the signature has a domain. `info --check` wrote its text and every check passed. With `--presentation`, either one also needs that no order line reported an index other than the group order. The colouring lines never change the status, including a search that stops at a limit (CLI-47). `render` wrote the output file, and with `--stats` the statistics lines. `--help` or `--version` wrote the help or the version text |
| `Failure` | The argument is a signature error of any kind, including a bad orbifold (`DEC-24`). A write to standard output failed (Section 6.6). With `--check`, a check failed or the signature has no domain (Section 6.7). With `--presentation`, an order line reported an index other than the group order (Section 6.9), or a relator had no text (Section 5.7). `render` met a render error (Section 6.8), including a signature without a domain, a bad setting, the colouring `parity` for a domain without a reversing side, a colouring id that names no colouring, a search for an id that stops at a limit, and an id with more than 12 colours without `--colours` |
| `Usage` | `clap` reports a usage error (Section 6.5), including a malformed colouring id and a value of `--max-colours` that Section 4.3 rejects |

CLI-11. `run` MUST return `Failure` for every signature error, including a bad orbifold, because a caller that gets 0 for an invalid signature treats the missing information as valid (`DEC-24`). A caller that needs the error kind uses `escher-signature` directly.

CLI-19. `info --check` MUST return `Failure` when a check fails and when the signature has no domain, and MUST return `Success` only when every check passed, because a script that runs `--check` reads the exit status as the verdict, and a status of 0 for a signature with nothing to check reads as verified. Section 5.8 of `docs/design/architecture.md` gives the same reason for `render`.

## 8. Testing

### 8.1. Tests

The tests live in one integration-test binary, `crates/escher-cli/tests/cli/main.rs`, with the modules `info`, `diagnostics`, `usage`, `render`, `process`, and `common`, and the snapshots under `crates/escher-cli/tests/cli/snapshots/` (rule TEST-10 of `docs/rust-coding-standards.md`). `main.rs` has a `//!` line, which the lint `missing_docs` requires. It declares `common`, `render`, and `process` under `#[cfg(test)]`, because their helpers call `expect`, which the Clippy option `allow-expect-in-tests` accepts only in code compiled for tests. Every test outside `main.rs` and `process` calls `run` with `Vec<u8>` streams, or with a test writer that fails. The two tests in `main.rs` test `Status` alone. The one test in `process` starts the binary.

A test that can write the output file gets a path from a helper in `common`, in a directory of its own, `CARGO_TARGET_TMPDIR/escher-cli/<test>`. The helper empties that directory first, so a file from an earlier run cannot satisfy a test. The render usage snapshots and the render status tests fail before any file access and pass the fixed path `CARGO_TARGET_TMPDIR/escher-cli/never-written.png`, which a constant in `common` holds and no test creates.

The tests have these layers:

| Layer | Form | What it catches |
| --- | --- | --- |
| Snapshot tests | `insta` tests in the modules `info`, `diagnostics`, `usage`, and `render` that call `run` with in-memory streams and compare its status and both streams with a committed snapshot | Any change to the information text, the help, version, and usage text, a diagnostic, or the statistics lines, for each kind of domain and each form of each line of Sections 4.7 and 5 |
| Plain tests | Tests that call `run` and assert the status, the start or the end of a stream, and the file that `render` writes | A wrong status, a failed write with a writer that fails, a PNG header with the wrong size or chunks, read by hand, a file that depends on the number of threads (CLI-23), the aliases of CLI-48, the default colouring of CLI-34, an existing output file that a failure changes (CLI-21), and a use of the global pool of `rayon` (CLI-22), which relies on nextest running each test in a process of its own |
| Unit tests | Tests in `args.rs`, `info.rs`, and `render.rs` of the private parsers, formatters, and functions | A parser of an option value that accepts or rejects the wrong text, a formatter of a line that writes the wrong form, a wrong default colour, and the paths that no known input reaches, such as a failed check, a different index, an internal error of a presentation, and a search that stops at a limit |
| Smoke test | The one test in `process`, which starts the binary that Cargo names in `CARGO_BIN_EXE_escher` | A binary that does not call `run` or does not turn its status into the exit status |
| Doctests | The crate-level examples | An example that no longer compiles or runs |

The argument that is not UTF-8 is built with `std::os::unix::ffi::OsStringExt`, and the symbolic link with `std::os::unix::fs::symlink`, so those tests compile only on Unix, which covers Linux, the supported platform.

No known input makes a check fail, so the failed-check path is tested through private functions, as Section 12.1 of `docs/design/signature.md` does for its `Overflow` texts. The unit tests cover the formatter of one check, the check lines, the `Verification` line, and the count of failed checks built from the rows, the writing of the text before the verdict is returned, and the diagnostic function.

The colouring lines of a search that stops at a limit are tested by unit tests with a small `SearchBudget` and not with a snapshot, because a search that reaches the default step budget is slow even in a release build, and a search that stops only with `--colourings` can print thousands of `Colouring` lines. The diagnostic of `render` for a search that stops at a limit has a snapshot only on an input whose search stops early at the limit on the stored colourings. No snapshot uses a search that stops at the step budget, which can take seconds, and a unit test with a small budget covers that branch of the resolution.

A snapshot holds the status, the standard output, and the standard error of one `run` call, in this form, which a helper in the module `common` builds (rule TEST-12 of `docs/rust-coding-standards.md`):

```text
status: <Debug text of the Status>
--- stdout
<standard output>--- stderr
<standard error>--- end
```

Each stream follows its marker line unchanged, so a stream that ends with a line feed puts the next marker on a line of its own, and a stream that does not runs into it. The one exception is the snapshot of `render --help`, whose lines have their trailing whitespace trimmed, because `clap` indents the blank lines of a long help. Its test asserts that no line of the untrimmed output ends in whitespace unless it holds only whitespace. The final marker `--- end` is needed because `insta` trims the whitespace at the end of a snapshot before it compares, so without it a missing or extra final line feed of standard error would pass. The writer that fails, `FailingWriter`, is also in `common`.

Every snapshot must be deterministic (rule TEST-2 of `docs/rust-coding-standards.md`), and two kinds of floating-point value in the output need a rule for that:

- The residual of a passed check is a rounding error, whose last digits can depend on the mathematics library of the platform. In every line that starts with `Check `, the snapshot helper replaces the text between `residual ` and the next `,` with `<residual>`, as the example of Section 5.3 shows. The tolerance, the result, and the line `Verification` stay in the snapshot. A unit test covers the form of a residual. A plain test covers which residual each label prints, without a snapshot, because it compares each line that starts with `Check ` with the values computed in the same process for `*(11)(10)5`. The input is chosen because its six residuals print pairwise distinct in the development container, so there a swap of any two checks changes the output. The test does not assert that the residuals are distinct, so on a platform where two of them print the same, a swap of those two checks goes unnoticed.
- The values of rows 14 to 16 are printed with six digits after the point (Section 5.5). The colouring lines do not depend on these values. A snapshot input is chosen so that the exact value of each of these rows lies farther from a rounding boundary of the sixth digit than the error bound that Sections 9.3 to 9.9 of `docs/design/domain.md` state for that quantity and that kind of domain, converted to degrees for an angle. Every correct build then prints the same digits. The minimum, the maximum, and the default of a parameter are constants of `escher-domain`, whose `Display` text is the same on every platform, so they need no such condition. The defaults of a reflection polygon are rounded to three decimals by `DEC-13`, so a default could differ between platforms only if it lay within its rounding error of a boundary of the third decimal.

CLI-20. A snapshot MUST NOT hold a residual of a check, and its inputs MUST meet the condition on rounding boundaries above, because a value that depends on the last bits of floating-point arithmetic makes the snapshot test fail on a platform whose mathematics library rounds differently, although the program is correct.

The colouring lines hold only ids and integers, and the search is a function of the side presentation, the colour limit, and the step budget alone, with no dependence on the machine or the number of threads (the rules of the search in `docs/design/group.md`), so they need no masking. The presentation lines hold only names, relator texts, and integers, which do not depend on floating-point arithmetic, and the enumeration is a function of its inputs alone (Section 7.9 of `docs/design/group.md`, GRP-18), so the presentation lines need no masking.

The default colouring search of a reflection polygon or of its double can take seconds, and with many corners it reaches the step budget at a low colour count, so the snapshots of these domains need CLI-50.

CLI-50. A snapshot test of `info` on a reflection polygon or on its double MUST set `--max-colours` to at most 4, to at most 3 for a reflection polygon with 7 or 8 corners and for the double of a polygon with 6 or more corners, and to at most 2 when it gives `--check` or `--presentation`. The snapshot MUST also print only the default shape, or values that meet the condition on rounding boundaries of CLI-20. A larger colour limit can make a test take seconds, or make its search reach the step budget, and a printed value near a rounding boundary makes the snapshot depend on the platform.

The search of the side presentation of a wallpaper domain to 12 colours is fast, so the snapshots of the wallpaper domains need no lower colour limit.

CLI-52. A snapshot test of `info` on a wallpaper domain MUST use the default colour limit of 12, unless its debug time exceeds that of the snapshot of `*2222`, and MUST print only the default settings, because the colouring lines up to 12 colours are what `info` prints by default, the prototype measured that the search of a side presentation to 12 colours took at most 0.057 s in a debug build, and a printed value of another setting could lie near a rounding boundary.

CLI-52 does not govern `2222`, because a wallpaper domain is the domain of one of the nine wallpaper signatures (Section 2.1 of `docs/design/domain.md`), and `2222` is not one of them. Its snapshots use the default colour limit.

The search of the side presentation of a spherical group domain to 12 colours is also fast, so the snapshots of these domains need no lower colour limit.

CLI-54. A snapshot test of `info` on a spherical group domain MUST use the default colour limit of 12 and MUST hold only inputs whose printed values meet the condition on rounding boundaries of CLI-20, because the colouring lines up to 12 colours are what `info` prints by default, the snapshots of `info` on the kites `3*2`, `3*`, `2*3`, `2*`, and `2*(12)` took 0.025 to 0.033 s in one debug run of `just test`, less than the 0.104 s of the snapshot of `*2222` in the same run, the prototype measured at most 18 ms in a debug build for the other spherical group domains with $n$ up to 12, and a printed value near a rounding boundary makes the snapshot depend on the platform.

The statistics lines need a rule of their own. The time differs on every run. The fold counts can differ between platforms, because a sample on or next to a wall can take a different side when the last bits of a side value differ (`UNV-3`). In every line that starts with `Mean folds: `, `Maximum folds: `, `Unconverged samples: `, or `Time: `, the snapshot helper replaces the text after the first `: ` with `<value>`. The line `Samples` stays, because the number of samples is the product of the size and the grid.

CLI-28. A snapshot MUST NOT hold the value of a statistics line other than `Samples`, because the time differs on every run and the fold counts can differ between correct builds, so a snapshot that held them would fail although the program is correct.

A snapshot of `render` holds no path, because no diagnostic and no statistics line repeats the output path (CLI-26), so the snapshot does not depend on the directory of `CARGO_TARGET_TMPDIR`.

When a change adds a snapshot, it adds one snapshot for each `run` call. The input meets the condition on rounding boundaries of CLI-20, the colour limit follows CLI-50, CLI-52, or CLI-54 for its kind of domain, and an input that needs a signature without a domain takes `4*3`, or `2*(51)` for a spherical one (Section 11.2, choice 12). A path that no known input reaches is tested through private functions and not by a snapshot. A change to an existing text updates its snapshot in the same change (CLI-7).

Every display text of `escher-signature` that a known input produces passes through a diagnostic snapshot, so `escher-signature` needs no `insta` dependency. Likewise, the snapshots of the `Domain` line cover the `UnsupportedSignature` and `LimitExceeded` texts of `escher-domain`, including the text of the limit of `2*n`, and the render diagnostic snapshots cover its texts of a refused polygon shape, which has the kind `Numerical`, and of more than 8 orders. No known input reaches the text of a refused double, because no double within the limit on orders is known to be refused (Section 8.4 of `docs/design/domain.md`). No known input reaches the texts of `UnknownParameter` and `ParameterOutOfRange` through `render`, which writes its own diagnostic for them (CLI-29). The render diagnostic snapshots cover the texts of the error kinds of `escher-render` that an option produces.

The smoke test is the one test of this crate that starts the binary, as layer 9 of Section 7.1 of `docs/design/architecture.md`. It checks only that the binary runs `run` and turns its status into the exit status, because every other behaviour is tested in the test process.

The tests read the PNG header by hand, so this crate needs no `png` dependency. The tests of `escher-render` decode the image with `png` and compare its pixels (Section 8 of `docs/design/render.md`).

CLI-13. Withdrawn on 2026-11-19.

CLI-27. Withdrawn on 2026-11-19.

### 8.2. Snapshots Under nextest

The recipe `test` runs the snapshot tests through nextest like every other test, with no setting of their own. A snapshot test fails when the output differs from its committed snapshot file. Outside CI, `insta` then writes the new snapshot beside the committed one as a `.snap.new` file, and with `CI=true` it writes no such file. An agent reviews and accepts a change with `cargo insta review`, which the coding standard lists as run by hand (Section 4.8 of `docs/rust-coding-standards.md`). No recipe wraps `cargo insta`.

These environment variables of `insta` matter here:

| Variable | Behaviour when unset |
| --- | --- |
| `INSTA_UPDATE` | `auto`, because the repository has no `insta.yaml` file, which gives the `.snap.new` file outside CI |
| `CI` | Not CI. `insta` counts a run as CI when `CI` has any value other than `false`, `0`, or the empty string, or when `CI` is unset and `TF_BUILD` is set |
| `INSTA_FORCE_PASS` | False |
| `INSTA_REQUIRE_FULL_MATCH` | False, so `insta` trims the whitespace at the end of a snapshot before it compares, which is why the snapshot form of Section 8.1 ends with a marker line |
| `INSTA_WORKSPACE_ROOT` | `insta` runs `cargo metadata` with the `cargo` that `CARGO` names to find the workspace root, and nextest sets `CARGO` |

`.gitignore` ignores `*.snap.new`, which a failing test writes, and `*.pending-snap`, which `insta` writes only for inline snapshots. Section 14.2 of `docs/design/tooling.md` lists the variables that a contributor or CI needs to know.

CLI-12. An agent MUST NOT set `INSTA_FORCE_PASS` in a recipe, a hook, or a workflow, because it turns a failing snapshot test into a pass, and a changed text then reaches `main` unreviewed.

## 9. Dependencies

### 9.1. Workspace Dependencies

The crate declares these dependencies, each from one entry in the root `[workspace.dependencies]` table (rule DEP-2 of `docs/rust-coding-standards.md`). The table also holds the entries that `docs/design/signature.md`, `docs/design/geometry.md`, `docs/design/group.md`, `docs/design/domain.md`, and `docs/design/render.md` specify for the other crates.

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
escher-group = { version = "0.1.0", path = "crates/escher-group" }
escher-render = { version = "0.1.0", path = "crates/escher-render" }
escher-signature = { version = "0.1.0", path = "crates/escher-signature" }
insta = { version = "1.49", default-features = false }
rayon = "1.12.0"
```

| Dependency | Kind | Features | Reason |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | Parses and classifies the signature. The path dependency carries a version (TOOL-25) |
| `escher-domain` | Normal | None | Lists the shape parameters for the parameter lines, builds and verifies the fundamental domain for the domain summary and `--check`, builds the domain that `render` draws with the settings, and gives the names and ranges that the diagnostic of a rejected setting uses. The path dependency carries a version (TOOL-25) |
| `escher-group` | Normal | None | Derives the Conway presentation, gives the text of the relators, and enumerates the cosets of both presentations for `--presentation`, searches the colourings for the colouring lines, and parses and looks up a colouring id for `render`. The path dependency carries a version (TOOL-25) |
| `escher-render` | Normal | None | Renders the image, with the coset fill for a colouring id, counts its statistics, and encodes it as PNG for `render`. The path dependency carries a version (TOOL-25) |
| `rayon` | Normal | Default | The thread pool of `render` (`DEC-21`, Section 3.5 of `docs/design/architecture.md`) |
| `clap` | Normal | The list above, without `color`, `env`, and `wrap_help` | The derive API (Section 3.1 of `docs/design/architecture.md`). Without `color`, `anstream` and its dependencies are not built (Section 9.2) |
| `anyhow` | Normal | Default | The chain of errors that Section 5.8 of `docs/design/architecture.md` requires. Version 1.0.103 fixes RUSTSEC-2026-0190 (Section 10 of `docs/research/rust-ecosystem.md`) |
| `insta` | Dev | None, so without `colors` | Snapshot tests (rule TEST-2). Without `colors`, `console` is not built (Section 9.2) |

The crate declares neither `escher-geometry` nor `glam`. The measurements of the domain are methods of `FundamentalDomain` that return plain `f64` values, so this crate names no `glam` type (Section 15.2 of `docs/design/domain.md`). `render` passes the domain to `escher-render` unchanged and reads only integers from the statistics, so it names no `glam` type either. `escher-domain` brings `escher-geometry`, `glam`, and `thiserror` into the build of the binary, and `escher-render` brings `png` and its dependencies. `escher-group` depends on `escher-signature` and `thiserror` only (Section 12 of `docs/design/group.md`), so it adds no external crate to the build.

The crate declares no colour crate and no `png`. The private parser of Section 4.6 reads the two hexadecimal forms of a colour (`DEC-29`), the default colours are a table of literal values (CLI-44), and the tests read the PNG header by hand (Section 8.1).

The package keeps `publish = false` (`DEC-22`) and declares no Cargo feature.

### 9.2. Verified Resolution

`just check-deny` checks the licences, advisories, duplicate versions, and sources of these dependencies, and `just check-msrv` checks the workspace with the toolchain that `rust-version` names, so a dependency that needs a later version fails it. `deny.toml` allows `MIT`, which `strsim` brings through the `clap` feature `suggestions`, and Section 8 of `docs/design/tooling.md` records its compatibility with GPL-3.0-or-later, as TOOL-24 of that document requires.

Without the `clap` feature `color` and the `insta` feature `colors`, the build lists none of `anstream`, `console`, `colorchoice`, and `terminal_size`. `clap_builder` is built with only the features `derive`, `error-context`, `help`, `std`, `suggestions`, and `usage`, and `insta` with no features.

The owner chose a parser of the project's own for colours (`DEC-29`). `csscolorparser` with its default features and `palette` in every configuration that was tried brought `syn` 2 beside the `syn` 3 of the workspace and failed the ban on duplicate versions, and `csscolorparser` without its default features read no colour name. Section 9 of `docs/design/render.md` records the resolution of `png` and the dependencies that `escher-render` brings into the binary.

### 9.3. Dependency Risks

cargo-deny leaves dev-dependencies out of the licence check and the duplicate check by default. `deny.toml` overrides both defaults, as Section 8 of `docs/design/tooling.md` states, so `just check-deny` checks dev-dependencies. With the overrides, `cargo deny list` names `insta`, its dependencies such as `similar`, `tempfile`, `once_cell`, `fastrand`, and `rustix`, and `proptest`.

CLI-14. The dev-dependencies of this crate MUST stay inside the dependency policy of Section 8 of `docs/design/tooling.md`, so that `cargo deny list` names `insta` and its dependencies, because otherwise the licences, advisories, and duplicates of the snapshot library go unchecked while the dependency policy appears to cover them.

Section 10.1 of `docs/design/architecture.md` records, under `DEC-21`, that Miri reported undefined behaviour inside `crossbeam-epoch`, a dependency of `rayon`, and that no check of this repository now looks for it. A defect in the code of `rayon` and `crossbeam-epoch` that this crate runs would reach the binary without any check of this repository finding it. The owner accepts this risk for the proof of concept (`DEC-21`).

## 10. Unverified Statements

This section lists the unverified statements of Section 7 of `docs/design/poc-plan.md` that this crate relies on, as PLAN-3 of that document requires. The crate relies on `UNV-20` and `UNV-21` through `escher-signature`, on `UNV-6`, `UNV-7`, `UNV-10`, `UNV-11`, `UNV-22`, `UNV-26`, `UNV-27`, `UNV-28`, `UNV-29`, `UNV-30`, `UNV-31`, `UNV-32`, `UNV-33`, and `UNV-34` through `escher-domain`, on `UNV-17`, `UNV-19`, and `UNV-22` through `escher-group`, and on `UNV-1`, `UNV-2`, `UNV-3`, `UNV-4`, `UNV-15`, `UNV-16`, `UNV-17`, and `UNV-18` through `escher-render`, whose design states how it relies on each.

| Id | How this crate relies on it |
| --- | --- |
| `UNV-20` | `info` prints the canonical signature, with parentheses for orders of 10 or more and `(inf)` for an infinite order, and accepts both spellings of infinity and `1` as the trivial signature, whose domain has no side (CLI-53) |
| `UNV-21` | `info` prints the geometry and the area of a non-compact signature whose Euler characteristic is not 0, and prints no deformation dimension for a non-compact signature |
| `UNV-26` | `info --check` reports the checks of the walls of the domain, which `escher-domain` builds through the formula of the wall through two points that `UNV-26` covers. The walls, relators, and angles checks test the result (Section 14 of `docs/design/domain.md`). The domain summary does not depend on the walls, because `escher-domain` measures it from the vertices |
| `UNV-10` | `info` prints one `Parameter` line for each parameter that `escher-domain` specifies, such as one for `*2222` and two, `aspect` and `skew`, for `o` and `2222`. No source states whether the published parameter count of `*2222` includes the scale, so the lines list the parameters of the construction and claim no count |
| `UNV-11`, `UNV-33` | `info` prints the summary of a wallpaper domain, and `render` draws it as a fundamental domain of the group of its signature, which rests on constructions and pairings that no source states. `build_with` returns only a wallpaper domain that passes `verify()` (DOM-54 of `docs/design/domain.md`), and `info --check` reports the checks |
| `UNV-11`, `UNV-34` | `info` prints the summary of a spherical group domain, with the word `rotary reflection`, and `render` draws it as a fundamental domain of the group of its signature, which rests on domains and pairings that no source states. `build_with` returns every such domain except that of `*` only when it passes `verify()` (DOM-60 of `docs/design/domain.md`), the certificate of DOM-64 of that document checks each group, and `info --check` reports the checks. The order lines of `--presentation` compare the order of the side presentation with the group order, which cannot tell these groups from others of the same order (DOM-64 of `docs/design/domain.md`) |
| `UNV-28` | `info` prints the summary of a Euclidean triangle, and `render` draws it, from the vertex coordinates that `escher-domain` builds, whose checks and tests its design states |
| `UNV-29` | `info` prints the summary of a lune, and `render` draws it as a fundamental domain, which no source states. `info --check` reports the checks that test it |
| `UNV-6`, `UNV-7`, `UNV-27` | `info` prints the parameters and the summary of a reflection polygon, and `render` draws it, from the construction of SymmHub's cuts, the default shape, and the ranges that `escher-domain` builds on them. `build_with` returns only a polygon or a double that passes `verify()` (DOM-44 of `docs/design/domain.md`), and `info --check` reports the checks |
| `UNV-22`, `UNV-30`, `UNV-31` | `info` prints the summary of a rotation domain, and `render` draws it as a fundamental domain of the rotation subgroup, which rests on the signature of that subgroup, on the lune double, and on the convexity and the vertex cycles of the double. `info --check` reports the checks that test each build |
| `UNV-19` | The order lines of `--presentation` come from the coset enumeration of `escher-group`, and the colouring lines and the colourings of `render` from its low-index search, whose reliance on `UNV-19` Section 13 of `docs/design/group.md` states. A run that reaches the limit prints `not found` and claims nothing about the index (Section 5.7), and a search that stops at a limit claims nothing beyond its last complete colour count (CLI-47) |
| `UNV-17` | `info` lists the colourings as the colourings of the tiling, and `render` draws one by its coset fill, which rests on the coset colour of Section 5.12 of `docs/design/render.md`. `info` claims nothing about which colourings are perfect, and prints only the subgroups that the search finds |
| `UNV-22` | `--presentation` prints the Conway presentation of a boundary without corners, such as that of `*`, in the reading that Section 5.2 of `docs/design/group.md` records (GRP-7) |
| `UNV-32` | `--presentation` prints the side presentation of a rotation domain, whose cycle relators as words are the reading of `docs/design/domain.md` (Section 5.4 of that document). The side order line compares the order that they present with the group order |
| `UNV-1` | `render` relies on the fold to stop for the samples of an image of a reflection triangle, polygon, rectangle, or lune. `Unconverged samples` counts the samples for which it does not stop within the fold cap, which on the sphere can happen near a vertex of very high order |
| `UNV-2` | `render` relies on the fold to stop for the samples of an image of a rotation domain, a wallpaper domain, whose pairings include translations and glide reflections, or a spherical group domain, whose pairings include rotary reflections. `Unconverged samples` counts the samples for which it does not stop within the fold cap |
| `UNV-16` | The ball that `render` draws comes from the lift onto the ball and the Phong reflection model, whose primary paper is unread |
| `UNV-3` | A sample on or next to a wall can take either neighbouring tile, which is why CLI-28 masks the fold counts |
| `UNV-4` | The default fold cap of `DEC-7` rests on fold counts measured on a prototype, and no source bounds them |
| `UNV-15` | The border that `render` draws comes from the gradient form of the border estimate (`DEC-28`) |
| `UNV-18` | The default colouring `parity` of a domain with a reversing side, which includes a domain whose sides are paired by glide reflections, colours the tiles by the parity of the fold, and the line `Parity colouring: none` and the diagnostic of CLI-35 rest on the reading that no parity colouring exists when every pairing preserves orientation. The alias `parity` of a `Colouring` line names the colouring with 2 colours that reads the same parity (CLI-48) |

## 11. Assumptions, Choices, and Open Questions

### 11.1. Assumptions

- The infinity sign takes one column in the terminal. Unicode gives it an ambiguous East Asian width, so a terminal that renders such characters as wide shifts the caret one column to the right for each infinity sign before the span.
- With its features `env` and `wrap_help` off, `clap` reads no environment variable, so the output does not depend on the environment. A reading of the source of `clap_builder` supports this, and no run has confirmed it.
- The residual of a passed check and the fold counts can differ in their last digits between platforms, which is why Section 8.1 masks them (CLI-20, CLI-28). No run on two platforms has measured this, and the masking is safe either way.
- With the default actions of its derive API, `clap` rejects a flag or an option that is given more than once. With the action `Append`, it accepts `--set` more than once and keeps its values in the order given, which the diagnostic of a parameter set twice relies on.
- `clap` generates the usage line of `render` that Section 4.1 gives, the text `[default: <value>]` and `[possible values: phong, none]` in its help, and no possible values for `--colouring`, whose value parser is a private function.
- A setting of `--set` does not change the side presentation of a domain, so the colouring ids of a signature do not depend on it. The side presentation reads only the kinds, partners, corner orders, and vertex cycles of the sides (Section 5.4 of `docs/design/domain.md`), and the numbering of the sides depends on the canonical signature alone (DOM-37, DOM-43, and DOM-53 of that document). No test of this crate compares the side presentations of two settings.
- `rayon` reads the environment variable `RAYON_NUM_THREADS` only for a pool whose builder sets no number of threads, so it has no effect on the pool of `render`, which always sets one. Nothing has confirmed this, and an error here changes only the number of threads that a call starts.
- `std::fs::write` follows a symbolic link at the path, as the documentation of `std::fs::File::create` states.
- The PNG header that the tests read has the layout of the PNG specification, which no document under `docs/research/` cites. The tests of `escher-render` decode the same bytes with `png`, so the two layouts agree.
- A write to a FIFO whose reader has closed fails with the error kind `BrokenPipe` and does not end the process, because Rust ignores the signal `SIGPIPE` (Section 6.6). This holds only for a write larger than the buffer of the pipe, 64 KiB by default on Linux.

### 11.2. Choices of This Design

These choices are made here, and no other document dictates them. Each one is easy to undo by mistake, or would otherwise leave a reader asking why the tool behaves as it does:

1. `clap` and `insta` build without their colour features, so the output is plain text and `anstream` and `console` stay out of the build (Section 9.2).
2. Snapshots live only in this crate, with one snapshot for each `run` call, so `escher-signature` needs no `insta` dependency (Section 8.1). No recipe wraps `cargo insta`, because a reviewer accepts a changed snapshot by hand (Section 8.2).
3. `info` returns its errors as a list, and `run` writes one diagnostic for each, in order, so `info --check --presentation` can report an order diagnostic and then a verification diagnostic (Section 3.2).
4. `clap` checks only the syntax of an option value, and `escher-render` checks its range, so a size of `0x0` gives `Failure` and not `Usage` (Section 4.6, CLI-24).
5. `render` reads the border colour and the background colour before it builds the domain, and reads the fill colours and checks the render options before it resolves a colouring id, so a certain input error never waits for a search or gives way to the diagnostic of a search that stopped at a limit (Section 4.5).
6. The render options are checked before the search with a placeholder fill of one colour, which is the border colour, and built a second time with the real fill, because the builder of `escher-render` takes the fill first and the coset fill exists only after the search (Section 4.5).
7. `render` clones the colouring that `find` returns, because `ColouringsOfIndex` only lends it and `CosetFill::new` takes it by value (Section 4.5).
8. The private function that resolves an id takes the step budget as a parameter, and `render` passes `SearchBudget::DEFAULT`, so a unit test can reach the diagnostic of a search that stops at a limit with a small budget (Sections 4.5 and 8.1).
9. `--colouring` has no default of `clap`, because its default depends on the domain (CLI-34).
10. `render` refuses no domain for its geometry or for the kinds of its sides, because `escher-render` has a view for every geometry (RND-23 of `docs/design/render.md`) and renders a rotation domain within the bounds of Section 6.8 of that document (RND-35).
11. The tests read the PNG header by hand, so the crate has no `png` dev-dependency (Section 8.1).
12. The examples and snapshots that need a signature without a domain use `4*3`, and those that need a spherical signature without a domain use `2*(51)`, which is over the limit of `2*n`, so that `info` prints its group order (Sections 5.3 and 8.1).

### 11.3. Points for the Owner

None of these blocks the implementation. Points 1, 3, 4, 5, 8, and 36 belong to the document that each names, and stay here as one line each so that the numbers that other documents cite do not change:

1. The order of infinity in the canonical form belongs to Section 15.3, item 1, of `docs/design/signature.md`. The examples and snapshots assume that an infinite order sorts above every finite order, as in `*(inf)22`.
2. The area of a non-compact hyperbolic signature. `info` prints it, such as `Area: pi/3` for `3*(inf)`, and the value rests on `UNV-21`. The alternative prints the area only for a compact signature until a source covers that case.
3. The trivial group spelled `11` belongs to Section 15.3, item 2, of `docs/design/signature.md`.
4. The largest corner order of 100 is the owner's decision `DEC-69` of `docs/design/architecture.md`.
5. Check 6, the area check, belongs to Section 15.3, item 4, of `docs/design/domain.md`.
6. Two area lines. A signature with a domain prints both `Area`, the exact value from the Euler characteristic, and `Domain area`, measured from the domain, so a reader sees that the two agree. The alternative drops `Domain area`.
7. A `Domain` line for every valid signature, with `none` and the reason when no domain is built, so a reader can tell a missing domain from a missing line. The alternative prints no `Domain` line for such a signature.
8. The default fold cap of 1500 belongs to `DEC-7` of `docs/design/architecture.md`.
9. An opaque background only (`DEC-7`). A `transparent` value of `--background` would need a rule for averaging samples with alpha, which no source gives.
10. The default fill colours `#e9c46a` and `#2a9d8f` (`DEC-7`). Other defaults change the table of Section 4.6 and the help text.
11. The limit of 256 threads (Section 4.6), which bounds the threads that one call starts. The alternative is no limit beyond `u32`.
12. The diagnostic of a failed write does not name the path (CLI-26), which keeps untrusted text out of the terminal. The alternative repeats the path with the replacement of Section 6.2.
13. `--set` on `render` only (Section 4.5), so `info` describes the default shape. The alternative also gives `info` the option, which needs a new failure path in `info` and an exception to CLI-17.
14. The `Parameter` line, such as `Parameter: aspect, range 0.125 to 8, default 1`, between the deformation dimension and the `Domain` line (Section 5.1). Other forms change the snapshots.
15. The value forms of the options of the ball: the yaw, the pitch, and the roll in degrees for `--rotation`, `X,Y,Z` for `--light`, and `phong` or `none` for `--shading`, with the defaults of `DEC-16`. The alternative is other forms, such as one option per angle.
16. A value that starts with `-` for `--rotation` and `--light` (CLI-31), so an option name directly after either option is read as its value. The alternative accepts only the form with `=`, such as `--light=-1,1,2`.
17. The options of the ball on a hyperbolic or Euclidean signature are checked and have no effect (Section 4.6). The alternative is an error.
18. The `Parity colouring` line only for a domain with no reversing side (CLI-33). The alternative prints the line for every built domain, which changes the snapshot of every reflection domain.
19. `--colouring parity` for a domain without a reversing side is an error and writes no file (CLI-35, `DEC-32`). The alternative draws the image in the first colour, as `escher-render` does (RND-33 of `docs/design/render.md`).
20. The fill colours follow the colouring that the domain gives, so `escher render 732 --colours '#e9c46a,#2a9d8f'` fails, because the default colouring of `732` is `one`. The alternative chooses the colouring from the number of colours given.
21. Both presentations with `--presentation` (`DEC-35` of `docs/design/architecture.md`), so the two independent readings of the group can be compared. The alternative prints the Conway presentation only, which drops the check of the domain.
22. No order line outside the sphere (CLI-39). The alternative prints a line that says that the enumeration was not run, which changes the text of every Euclidean and hyperbolic signature.
23. No option for the coset limit (CLI-41). The alternative is an option such as `--max-cosets`.
24. The gate of the enumeration at a group order of 16384, near which `escher info --presentation` takes seconds. The largest group order of a domain that `escher-domain` builds is 400, of `*(100)22`, so the alternative is a lower gate, such as 4096, which keeps every built domain and prints `not run` for the larger orders.
25. Exit status 0 for an enumeration that reaches the coset limit (CLI-40), because no source guarantees that an enumeration finishes within the group order (Section 7.15 of `docs/research/colourings.md`). Such a run within the gate may indicate a defect, such as a dropped relator, so the alternative is exit status 1.
26. Long colouring listings (Section 5.8). `--colourings` prints one line for each subgroup, conjugates included, and some listings up to 12 colours exceed a thousand lines. The alternative marks the class of each colouring in its `Colouring` line from `Colouring::class` of `escher-group`, such as `3b, 3 colours, class 3a`, so a reader can skip the conjugates.
27. The default colours (CLI-44). Entries 3 to 12 were not checked for colour-vision deficiency, and a colouring with more than 12 colours needs `--colours`. The alternative is other or more entries after such a check.
28. No option for the step budget of the colouring search, so a search near the top of the range of `--max-colours` can stop at the budget for signatures with corner orders near 100. The alternative is an option for the budget, which adds an input whose cost is not bounded.
29. A search that stops at a limit leaves the status of `info` unchanged (CLI-47), while `render` with an id gives `Failure`. The alternative for `info` is a status of its own, which a script would have to tell from a failed check.
30. `--colouring parity` keeps `Fill::Parity`, so the default colouring runs no search (CLI-48). The alternative draws `parity` through its id, which runs a search for every default image of a reflection domain.
31. A higher colour limit can complete fewer colour counts (Section 5.8). When the one run at the limit exhausts the budget, the runs by index share one budget (Section 7.14 of `docs/design/group.md`, GRP-35), so a call with a higher limit can complete fewer colour counts than a call with a lower limit whose one run ends, as for `*666` at the limits 13 and 14. This is a known behaviour of the default design, and every line still claims only what its search completed (CLI-47). The alternative that would remove it is left to the owner.
32. The default colour limit of 12 also on the reflection polygons and their doubles, whose default search takes several tenths of a second or more in a release build. The alternative is a default colour limit that falls with the number of generators of the side presentation, which makes the default text depend on the domain.
33. The colouring ids of the doubles of the reflection polygons (`DEC-50` of `docs/design/architecture.md`) hold only until the roadmap item `M15`, which builds the general hyperbolic signatures (Section 4.1 of `docs/design/roadmap.md`), renumbers the sides of their domains, and no line marks them as provisional. The alternative marks them in the `Domain` line or refuses an id for such a signature.
34. Shared default pictures (`DEC-55` of `docs/design/architecture.md`), as Section 4.5 states, which a user tells apart by a colouring id. Point 40 of Section 12.3 of `docs/design/render.md` holds the alternative, which draws mirror sides differently from paired sides.
35. The third form of the `Domain` line with a `Mirror sides` line (CLI-51). The alternative lists the kind of every side in one line, which changes the text of every reflection domain and rotation domain.
36. Tests kept for a reason that does not apply belong to TQ-11 of `docs/design/tooling.md`, which puts them to the owner.
37. The parity texts of `1` (Section 5.5) stay unchanged, which needs no branch. The alternative is a text of its own, such as `Parity colouring: none, the domain has no side`, which adds a branch to CLI-33 and CLI-35.
38. The pictures of the spherical groups (`DEC-61` of `docs/design/architecture.md`). With the default colouring, `5x` renders the image of `*55` and `x` that of `*`, and with `--colouring one`, `3*2` renders the image of `432`. For the orders that Section 5.15 of `docs/design/render.md` treats, no picture of a colouring id tells `n*` from `n22` or `nx` from `*nn`. The default images of `*`, `x`, and `1` differ only in a thin rim, so a user who renders `*` sees no mirror until `--rotation` turns the ball. Points 42 and 43 of Section 12.3 of `docs/design/render.md` hold the alternatives.

## 12. Security Considerations

The threat model is a user who passes a signature, option values, or an output path from an untrusted party to `escher info` or `escher render`. The attacker can choose any argument within the limits of the command line, including invalid UTF-8, control characters, and very long text. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory or time through the signature or the options of `render`, injection of terminal escape sequences through the diagnostic, and the choice of the file that `render` overwrites. `escher-signature` sets the limits on the length of the signature, the number of features, and the size of an order, as Section 16 of `docs/design/signature.md` states. `escher-domain` limits the largest corner order of a domain, which bounds the cost of `verify()` and of every fold step, as Section 16 of `docs/design/domain.md` states. `escher-render` limits the image size, the number of pixels, the sample grid, the border width, and the fold cap, which bound the memory of the image and the work of each sample, as Section 13 of `docs/design/render.md` states. `escher-group` limits the size of a presentation and the cosets of an enumeration (Section 15 of `docs/design/group.md`, GRP-19 and GRP-20). The coset limit bounds the memory of an enumeration and not its time, so `--presentation` runs an enumeration only for a spherical signature with a group order of at most the limit (CLI-39), whose relators the signature bounds. `escher-group` bounds the colouring search by its index limit, its step budget, and its limit on the number of colourings (the rules of the search in `docs/design/group.md`), so `--max-colours` and a colouring id bound its time and memory, and a search that reaches a limit gives a text or a diagnostic, never a run without end. This crate adds one limit of its own, 256 threads (Section 4.6). `escher-domain` limits the value of each shape parameter to its range. The settings need no limit on their number or length, because each name is only compared, and a repeated or unknown name fails. The diagnostic of an unknown name echoes the whole name, with no cap like that of the echo line, so its length is bounded only by the command line.

The diagnostics of `render` repeat no path and no option value, except the three cases of CLI-26. The colouring lines hold only ids, fixed words, and numbers. The echo line repeats at most 120 characters and replaces every character outside printable ASCII and the infinity sign (CLI-9). The caret line holds only spaces and `^`. The `Domain` line and the diagnostics of Section 6.7 hold only fixed text, numbers, and the canonical signature, which is printable ASCII. The help, version, and usage text that `clap` produces has every control character other than the line feed replaced with `?` (CLI-15).

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, and the workspace denies `unsafe_code`.
- Files: the crate reads no file. `info` writes no file. `render` writes one file, the output file, with the permissions of the user, and only after rendering and encoding succeed (CLI-21). It replaces an existing file without asking (`DEC-30`), follows a symbolic link at the path (Section 11.1), and creates no directory. A caller who passes a path from an untrusted party therefore lets that party choose which file of the user `render` overwrites, and checking the path is the caller's task.
- Threads: `render` starts at most 256 threads, in a pool that ends with the call (CLI-22).
- Build scripts: the crate has none.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones (Section 9.3). Only `insta` is a dev-dependency, and it never reaches the binary.

Attacks on the repository, the build machine, or the terminal itself are out of scope, because this document governs the crate and not access control or the terminal.

The residual risk is that a usage error still echoes every character of an unknown argument that is not a control character, including characters outside ASCII, such as the formatting characters of the Unicode general category Cf. The replacement tests only for the category Cc. A second residual risk is a domain that is wrong by less than the tolerances, which `--check` reports as passed (Section 16 of `docs/design/domain.md`).

The limits of `escher-render` bound memory, but at their maximum values together they allow a rendering that runs for a very long time, which only the user can stop. In the same way, `info --presentation` near the gate runs for seconds (Section 11.3, item 24), and every `info` on a built domain runs the colouring search, which its step budget bounds (Section 5.8). A write that fails after `std::fs::write` has opened the file, such as on a full disk, leaves the file truncated or partly written. No check of this repository looks for undefined behaviour in `rayon` (Section 9.3).

## 13. Operational Considerations

The crate declares no Cargo feature and has no build script, so no environment variable affects the build. At run time the crate reads no environment variable, and, under the assumption of Section 11.1, `RAYON_NUM_THREADS` does not affect the pool of `render`. The snapshot tests read the variables of Section 8.2, which Section 14.2 of `docs/design/tooling.md` lists. The tests that write files use the directory `CARGO_TARGET_TMPDIR/escher-cli`, where Cargo sets `CARGO_TARGET_TMPDIR` for an integration test, and the smoke test uses the binary of `CARGO_BIN_EXE_escher`.

The supported platform is Linux. The minimum supported Rust version is the one that Technology Decisions of `AGENTS.md` states, and `just check-msrv` checks that the dependencies need no later version (Section 9.2).

| Failure | Reported as | Recovery |
| --- | --- | --- |
| The command line is wrong | The `clap` message on standard error, with `Usage` | Correct the command line, with `escher --help` |
| The signature is malformed, over a limit, or a bad orbifold | The diagnostic of Section 6, with `Failure` | Correct the signature at the caret |
| The signature has no domain, without `--check` | The `Domain` line with `none`, with `Success` | None needed |
| The signature has no domain, with `--check` | The diagnostic of Section 6.7, with `Failure` | Run `info` without `--check`, or use a signature that `escher-domain` constructs |
| A check fails | The check lines with `failed` and the diagnostic of Section 6.7, with `Failure` | None. It is a defect of `escher-domain`, and the residual and the tolerance locate it |
| An enumeration of `--presentation` finds an index other than the group order | The order line with `different from 2/chi` and the diagnostic of Section 6.9, with `Failure` | None. It is a defect of `escher-group`, `escher-domain`, or `escher-signature`, and the two order lines show which presentation disagrees |
| An enumeration of `--presentation` reaches the coset limit | The order line `not found within <L> cosets`, with the status unchanged | None in `info`, which has no option for the limit (CLI-41) |
| An enumeration of `--presentation` is refused by an input limit of `escher-group`, which no input reaches | The order line `not run, ` and the `Display` text of the error, with the status unchanged | None known |
| A write to standard output fails | The diagnostic of Section 6.6, with `Failure` | Fix the output destination |
| The reader of standard output closes the pipe | No diagnostic, with `Failure` | None needed |
| `render` gets a signature without a domain | The diagnostic of Section 6.8, with `Failure` | Use a signature that `escher-domain` constructs, which `escher info` shows in its `Domain` line |
| `render` gets an unknown parameter name, a value outside the range of its parameter, or a parameter set twice | The diagnostic of Section 6.8, with `Failure` | Use a name that the diagnostic names, each name once, and a value within the range that the diagnostic or `escher info` gives. `escher render --help` lists every parameter |
| `render` gets a bad colour or the wrong number of colours | The diagnostic of Section 6.8, with `Failure` | Give each colour as `#rgb` or `#rrggbb`, quoted, as many as the colouring needs |
| `render` gets `--colouring parity` for a domain without a reversing side | The diagnostic of Section 6.8, with `Failure` | Leave out `--colouring`, or give `--colouring one`. The line `Parity colouring` of `escher info` shows such a domain |
| `render` gets a colouring id that names no colouring of the domain | The diagnostic of Section 6.8, with `Failure` | Use an id that `escher info --colourings` lists for the signature |
| The search for a colouring id stops at a limit | The diagnostic of Section 6.8, with `Failure` | None in `render`, which has no option for the budget. Choose a colouring with fewer colours |
| `render` gets a colouring id with more than 12 colours and no `--colours` | The diagnostic of Section 6.8, with `Failure` | Give as many colours with `--colours` as the colouring has |
| The colouring search of `info` stops at a limit | The `Colourings` line names the last complete colour count, with the status unchanged | Use a lower `--max-colours`, whose lines are then complete |
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
| CLI-6 | 5.1 | `info` prints exactly the lines of Section 5.1, in order and under their conditions, with the colouring lines for every built domain, the `Colouring` lines only with `--colourings`, the presentation lines only with `--presentation`, and no name of the group in another notation. |
| CLI-7 | 5.4 | A change to the information text updates its snapshots in the same change. |
| CLI-8 | 6.1 | Every diagnostic goes to standard error. |
| CLI-9 | 6.2 | The echo line replaces unsafe characters with `?` and holds at most 120 characters. |
| CLI-10 | 6.6 | A failed write to standard output gives `Failure`, with no diagnostic for a broken pipe on standard output. |
| CLI-11 | 7 | Every signature error, including a bad orbifold, gives `Failure`. |
| CLI-12 | 8.2 | No recipe, hook, or workflow sets `INSTA_FORCE_PASS`. |
| CLI-13 | 8.1 | Withdrawn on 2026-11-19. |
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
| CLI-26 | 6.8 | No diagnostic of `render` repeats an option value or the output path, except a colouring id under CLI-49, the echoed name of a setting that names no parameter, and the parsed value of a setting outside its range. |
| CLI-27 | 8.1 | Withdrawn on 2026-11-19. |
| CLI-28 | 8.1 | No snapshot holds the value of a statistics line other than `Samples`. |
| CLI-29 | 4.5 | `render` passes every setting to `escher-domain`, which alone decides whether the settings are accepted, and reads the parameters of the signature only to name a rejected setting. |
| CLI-30 | 4.5 | Withdrawn on 2026-11-17. |
| CLI-31 | 4.6 | `--rotation` and `--light` accept a value that starts with `-`. |
| CLI-32 | 5.5 | `info` prints the number of pairs in the `Domain` line of a domain with a paired side, and one `Side pairs` line with the words `rotation`, `translation`, and `glide reflection`. |
| CLI-33 | 5.5 | `info` prints `Parity colouring: none, every side pairing preserves orientation` exactly when a domain is built and has no reversing side. |
| CLI-34 | 4.6 | Without `--colouring`, `render` uses `parity` when the domain has a reversing side, and `one` otherwise. |
| CLI-35 | 4.5 | `render --colouring parity` for a domain without a reversing side fails at step 4 with its diagnostic and writes no file. |
| CLI-36 | 4.5 | Withdrawn on 2026-11-17. |
| CLI-37 | 5.7 | `--presentation` adds rows 20 to 27 under their conditions and in their forms, and nothing appears without the flag. |
| CLI-38 | 5.7 | The generator names and relator texts are those of `escher-group`, unchanged. |
| CLI-39 | 5.7 | An order line appears only for a spherical signature, and no enumeration runs when the group order is above the coset limit. |
| CLI-40 | 6.9 | An index other than the group order gives the diagnostic and `Failure`, and every other outcome leaves the status unchanged, including a run that an input limit refuses, whose order line reads `not run, ` and the error text. |
| CLI-41 | 5.7 | `info` enumerates with `CosetLimit::DEFAULT` and has no option for the limit. |
| CLI-42 | 4.6 | `--colouring` accepts exactly `one`, `parity`, and what `ColouringId` parses, and gives a usage error for every other value. |
| CLI-43 | 4.5 | `render` resolves an id through the side presentation of its domain and the lookup of its colour count only, and fails with a diagnostic and no file when the id does not exist or the search stops at a limit. |
| CLI-44 | 4.6 | Without `--colours`, a colouring takes the first entries of the 12 default colours, and a colouring with more than 12 colours fails with a diagnostic that asks for `--colours`. |
| CLI-45 | 5.8 | `info` prints the colouring lines for every built domain, with counts, ids, and their order from `escher-group` unchanged, and the aliases `one` and `parity` on exactly their colourings. |
| CLI-46 | 5.8 | `--max-colours` sets the colour limit of every colouring line, and with `--colourings` the counts and the list come from one call. |
| CLI-47 | 5.8 | A search that stops at a limit gives the text of the last complete colour count and no line above it, and no outcome of the colouring lines changes the status. |
| CLI-48 | 4.6 | `one` and `parity` keep their fills, and `1a` and the id of the parity colouring give the same bytes. |
| CLI-49 | 6.8 | A diagnostic names a colouring id only by the `Display` text of its parsed value. |
| CLI-50 | 8.1 | A snapshot of `info` on a reflection polygon or its double sets `--max-colours` to at most 4, to at most 3 for a polygon with 7 or 8 corners or the double of a polygon with 6 or more corners, and to at most 2 with `--check` or `--presentation`. It prints only the default shape or values that meet the rounding condition of CLI-20. |
| CLI-51 | 5.5 | `info` chooses the form of the `Domain` line from the number of mirror sides, not from `reverses_orientation()`, and prints the `Mirror sides` line before the `Side pairs` line for a domain with both kinds of side. |
| CLI-52 | 8.1 | A snapshot of `info` on a wallpaper domain uses the default colour limit of 12, unless its measured debug time exceeds that of the snapshots of `*2222`, and prints only the default settings. |
| CLI-53 | 5.5 | For a domain with no side, `info` prints `Domain: no sides` and no `Corner angles in degrees` and `Side lengths` lines, and the `Side pairs` line names `rotary reflection` for `PairingKind::RotaryReflection`. |
| CLI-54 | 8.1 | A snapshot of `info` on a spherical group domain uses the default colour limit of 12 and holds only inputs whose printed values meet the rounding condition of CLI-20. |
