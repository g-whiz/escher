# Command-Line Interface Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-16 |
| Applies to | `crates/escher-cli`: the library target, the binary `escher`, and its tests |

## Abstract

This document specifies the package `escher-cli`, which provides the command-line program `escher`. It defines the public surface of the library target, the thin binary that calls it, the commands and options, the exact text that the command `info` prints for each kind of signature, including the summary of the fundamental domain and the result of each check of the option `--check`, the format of every diagnostic, the exit statuses, the tests, and the dependencies.

## 1. Introduction

### 1.1. Problem

The proof of concept is a command-line tool, and its first command reports what a signature in orbifold notation describes. The tool reads untrusted text, prints text that a reviewer checks line by line, and must report every malformed signature with a pointer to the offending characters. A program whose logic lives in `main` can only be tested by starting a process, which is slow and cannot run under Miri.

### 1.2. Purpose

This document fixes the behaviour of `escher` precisely enough that an agent can implement it and a reviewer can check every line of output against it. The crate `escher-signature` parses and classifies a signature, and the crate `escher-domain` builds and verifies its fundamental domain. This crate turns their results into text, diagnostics, and an exit status.

### 1.3. Audience

The primary readers are the coding agents that implement or extend `escher-cli`. The secondary readers are reviewers and the owner, who decides the points of Section 11.3.

### 1.4. Scope

This document covers the command `info` with its argument and its option `--check`, the options `--help` and `--version`, the diagnostics, the exit statuses, the tests of the crate, and its dependencies. `info` parses, validates, canonicalizes, and classifies any signature. For a signature that `escher-domain` constructs, it also prints a summary of the fundamental domain, and with `--check` the result of each check of its verification. It prints no item that needs a group.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The grammar of a signature, its canonical form, the classification methods, and the error kinds of `escher-signature` with their `Display` text are specified in `docs/design/signature.md`.
- The signatures that have a fundamental domain, its construction, its measurements, the checks of its verification with their tolerances, and the error kinds of `escher-domain` with their `Display` text are specified in `docs/design/domain.md`.
- The crates, the data flow, the error model, and the owner's decisions are specified in `docs/design/architecture.md`.
- The mathematics is stated in `docs/research/orbifold-notation.md`.
- The recipes, the nextest profiles, the hooks, and the dependency policy are specified in `docs/design/tooling.md`.

The command `render`, its options, and the lines of `info` that need parameters, a group, or a colouring belong to later milestones (Section 5.4). So do the handling of an existing output file (Section 11 of `docs/design/architecture.md`) and the diagnostic for a general non-compact signature in `render` (Section 5.8 of that document).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md` and of Section 2.1 of `docs/design/architecture.md`, such as signature, canonical signature, geometry, fundamental domain, and general non-compact signature. It adds these terms:

- ON: The reference `docs/research/orbifold-notation.md`, cited as `ON` and a section number, such as ON 7.3.
- command: A subcommand of `escher`, such as `info`.
- information text: The text that `escher info` writes to standard output for a valid signature (Section 5).
- domain: The `FundamentalDomain` that `FundamentalDomain::build` of `escher-domain` returns for a signature.
- domain summary: The lines of the information text that describe the domain, rows 10 to 13 of Section 5.1 (Section 5.5).
- check: One numeric condition that `verify()` of `escher-domain` evaluates, with a residual and a tolerance (Section 2.1 of `docs/design/domain.md`).
- verification lines: The lines of the information text that `--check` adds, rows 14 to 20 of Section 5.1 (Section 5.6).
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

CLI-2. `run` MUST write only to the two streams it is given and MUST NOT panic on any input or any output error, or else a test of `run` cannot observe all output, and one malformed argument or closed pipe aborts the program instead of giving a status.

`run` returns no error. Each command returns `anyhow::Result<()>` inside the crate, and one private function writes the diagnostic of Section 6 and chooses the status. No `anyhow::Error` therefore appears in a public signature, which rule ERR-2 of `docs/rust-coding-standards.md` forbids.

Every path of `run` that writes to `stdout` does so through one private helper, which writes the whole text and then flushes `stdout`, so that a write error that buffering delays still reaches the status. A path that writes nothing to `stdout` does not flush it.

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

CLI-3. `main.rs` MUST hold only the call of `run` and the conversion of its status, because code in the binary runs only in a test that starts a process, and this crate has no such test (Section 8.1).

The binary prints nothing itself, so `main.rs` carries no lint exception for `clippy::print_stdout`, which would fail as an unfulfilled expectation.

### 3.5. Private Modules

The library target has the private modules `args`, `info`, and `diagnostic`, as files named `args.rs`, `info.rs`, and `diagnostic.rs` (Section 4.5 of `docs/rust-coding-standards.md`). `args` holds the `clap` types, `info` builds the information text, including the domain summary and the verification lines, and `diagnostic` writes the diagnostics.

## 4. Command Line

### 4.1. Synopsis

```text
escher info [--check] <SIGNATURE>
escher --help
escher --version
```

`-h` is the short form of `--help` and `-V` of `--version`. `escher info --help` prints the help of `info`. `--check` has no short form.

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

CLI-4. `escher --version` and `escher -V` MUST write the package name, one space, the package version, and a newline to standard output, which is `escher-cli 0.1.0` for version 0.1.0, and return `Success`, because Section 5.1 of `docs/design/poc-plan.md` keeps this output, and a script may read it.

### 4.3. The `info` Command

`info` takes one required positional argument, the signature, as a `String` named `SIGNATURE`, and the optional flag `--check`. An argument that is not valid UTF-8 is a usage error. The flag may stand before or after the signature.

CLI-5. `info` MUST pass its argument unchanged to the `FromStr` implementation of `Signature` from `escher-signature`, and MUST NOT trim, normalize, or check it first, because a second reading of the grammar diverges from the one that `docs/design/signature.md` specifies, and the byte spans of the error then no longer match the argument.

On success, `info` writes the information text of Section 5 to standard output. On a signature error, it writes the diagnostic of Section 6 to standard error and nothing to standard output, with or without `--check`. A bad orbifold is a signature error (`DEC-24`).

With `--check`, `info` also runs `verify()` on the domain, adds the verification lines of Section 5.6 to the information text, and writes the diagnostic of Section 6.7 when a check fails or the signature has no domain.

The `clap` argument struct holds the flag as a `bool`, because the derive API sets a flag through a `bool` field. `info` converts it at once into a private enum with two variants, which the private functions take (TYPE-2 of `docs/rust-coding-standards.md`).

A signature never starts with `-`, so it never collides with an option. An argument that starts with `-` is read as an option, and an unknown option is a usage error.

### 4.4. Shell Quoting

In a POSIX shell, `*`, `(`, and `)` are special characters, so a signature that contains any of them is quoted in single quotes, as in `escher info '2*(12)'`. The shell passes every character inside single quotes unchanged. The help text of `info` says this, and every example in this document that needs it quotes the signature.

## 5. Information Text

### 5.1. Lines

`info` writes the lines of this table, in this order. Each line is the label, a colon, one space, and the value, followed by a line feed. No other line, blank line, or trailing space appears. The accessors of rows 1 to 9 are those of `Signature` (Sections 3.1 and 9.1 of `docs/design/signature.md`). The accessors of rows 10 to 20 are those of `FundamentalDomain`, `Verification`, and `Check` (Section 3.1 of `docs/design/domain.md`), and "a domain is built" means that `FundamentalDomain::build` returned `Ok`.

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
| 10 | `Domain` | Always | `<n> mirror sides` when a domain is built, and otherwise `none, ` followed by the `Display` text of the error (Section 5.5) |
| 11 | `Corner angles in degrees` | A domain is built | Each value of `corner_angles()` in degrees, in corner order, in the form of Section 5.2 |
| 12 | `Side lengths` | A domain is built | Each value of `side_lengths()`, in side order, in the form of Section 5.2 |
| 13 | `Domain area` | A domain is built | `area()`, in the form of Section 5.2 |
| 14 | `Check isometries` | `--check` is given and a domain is built | The result of `isometries()`, in the form of Section 5.6 |
| 15 | `Check pairings` | As row 14 | The result of `pairings()` |
| 16 | `Check walls` | As row 14 | The result of `walls()` |
| 17 | `Check relators` | As row 14 | The result of `relators()` |
| 18 | `Check angles` | As row 14 | The result of `angles()` |
| 19 | `Check area` | As row 14 | The result of `area()` of `Verification` |
| 20 | `Verification` | As row 14 | `passed` when `Verification::passed()` is true, and otherwise `failed` |

CLI-6. `info` MUST print exactly the lines of this table, in this order and under these conditions, because the snapshots of Section 8 and every reader of the text rely on one layout, and an extra or missing line changes what a reader concludes about the signature. The table has no name of the group in another notation, such as an IUC or Schoenflies name, because `DEC-20` omits them until a primary source is read.

So a signature without a domain gets the lines of rows 1 to 9 that apply to it and then exactly one more line, the `Domain` line of row 10, with or without `--check`. Rows 14 to 20 follow the order of the checks of Section 8.2 of `docs/design/domain.md`.

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
| Corner angle | 11 | The value of `corner_angles()` in radians converted with `f64::to_degrees`, then written in fixed notation with six digits after the point, `{:.6}` | `25.714286` |
| Side length | 12 | Fixed notation with six digits after the point, `{:.6}` | `0.545275` |
| Domain area | 13 | Fixed notation with six digits after the point, `{:.6}` | `0.074800` |
| Residual and tolerance | 14 to 19 | Scientific notation with one digit after the point, `{:.1e}` of Rust, which writes an infinite value as `inf` | `2.5e-3`, `1.0e-9` |

Rows 11 and 12 separate the values with a comma and one space. Angles are in degrees on the command line, as Section 5.5 of `docs/design/architecture.md` states.

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

This example shows a spherical signature, which has no domain:

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
Domain: none, cannot build fundamental domain: the signature *532 is not supported
```

ON 6.4 gives the Euler characteristic $1/60$ and the order 120, which the table of ON 8.2 lists. The area comes from ON 7.3.

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

This example shows a Euclidean signature:

```sh
escher info '*632'
```

```text
Signature: *632
Geometry: Euclidean
Euler characteristic: 0
Compact: yes
Domain: none, cannot build fundamental domain: the signature *632 is not supported
```

ON 8.1 lists `*632` among the 17 Euclidean signatures.

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
escher info --check '*632'
```

Standard output:

```text
Signature: *632
Geometry: Euclidean
Euler characteristic: 0
Compact: yes
Domain: none, cannot build fundamental domain: the signature *632 is not supported
```

Standard error:

```text
error: cannot verify fundamental domain
  caused by: cannot build fundamental domain: the signature *632 is not supported
```

The status is `Failure`, so the exit status is 1.

### 5.4. Stability and Omissions

The information text is not a stable interface, and a machine-readable form is a roadmap item (`DEC-23`).

CLI-7. A change that alters the information text MUST update its snapshots in the same change, because the snapshot test fails otherwise, and the snapshot diff is how a reviewer sees each change to the text (`DEC-23`, rule TEST-2 of `docs/rust-coding-standards.md`).

`info` prints no line about parameters, presentations, colourings, polygons, or colours, and no placeholder for them. Each needs a crate or a part of a crate that a later milestone adds. Section 3.4 of `docs/design/poc-plan.md` places them: the named parameters in M4 (Section 5.4), the presentation in M7 (Section 5.7), and the list of colourings with their colour counts in M8 (Section 5.8). The owner's goal that `info` reports how many polygons the representative set has, and so which colour counts are possible, is met by that list in M8. The domain summary covers exactly the signatures that `escher-domain` constructs (Section 4 of `docs/design/domain.md`).

### 5.5. Domain Summary

For every valid signature, `info` calls `FundamentalDomain::build` once and prints the `Domain` line of row 10. Rows 11 to 13 follow only when a domain is built.

When a domain is built, the value of the `Domain` line is the number of `sides()` in decimal, one space, and `mirror sides`. Every side of a domain that `escher-domain` builds has the kind `Mirror` (Section 3.2 of `docs/design/domain.md`).

When `build` returns an error, the value is `none, ` followed by the `Display` text of that error, for every error kind. The `Display` text starts with `cannot build fundamental domain: ` (Section 10.2 of `docs/design/domain.md`), so the line reads, for example, `Domain: none, cannot build fundamental domain: the signature *632 is not supported`. The line holds only the `Display` text of the top error and no text of its source. The text holds the canonical signature or a number, both in printable ASCII, so it needs no replacement of characters.

CLI-16. `info` MUST take the values of rows 10 to 20 from `escher-domain` unchanged, except for the conversion of the angles to degrees and the formats of Section 5.2, and MUST NOT measure or check the domain itself, because a second computation in this crate can differ from what `verify()` checks, and the printed values would then describe another domain than the one that passed.

CLI-17. Without `--check`, `info` MUST print the `Domain` line with `none` and return `Success` for a valid signature for which `build` returns an error, because the signature is still parsed, valid, and classified, and a non-zero status would tell a script that the signature itself is invalid.

The values of rows 11 to 13 are rounded to six digits after the point. Section 9.3 of `docs/design/domain.md` bounds the error of each value that `build` returns. A printed digit can therefore differ between two correct builds, such as builds on two platforms, only when the exact value lies within that bound of a rounding boundary of the sixth digit. Section 8.1 states how the snapshot inputs avoid that case.

### 5.6. Verification Lines

With `--check`, when a domain is built, `info` calls `verify()` once and prints rows 14 to 20 after the domain summary. The information text and the verification lines form one text, which `info` writes in one write.

The value of each of rows 14 to 19 has this form, where the result is `passed` when `Check::passed()` is true and `failed` otherwise, and the residual and the tolerance have the form of Section 5.2:

```text
<result>, residual <residual>, tolerance <tolerance>
```

For example, a failed relator check prints `Check relators: failed, residual 2.5e-3, tolerance 2.0e-6`. A residual of infinity, which marks a failed structural condition or a computation that gave no finite number (Section 8.1 of `docs/design/domain.md`), prints as `inf`. No residual is NaN.

Row 20 gives the overall result, `Verification: passed` or `Verification: failed`, from `Verification::passed()`.

CLI-18. With `--check`, `info` MUST print one line for each check that `Verification` reports, with its result, its residual, and its tolerance, in the order of Section 8.2 of `docs/design/domain.md`, and then the overall result, because a reader who sees only the overall result cannot tell which condition failed or by how much, and a line that omits the tolerance cannot be read without the design document.

## 6. Diagnostics

### 6.1. Format

CLI-8. `run` MUST write every diagnostic to standard error and none to standard output, because a caller that captures standard output must receive only the information text, or else an error is read as data.

The diagnostic of a signature error, an output error, or a verification error of Section 6.7 has this form, and Section 6.5 covers a usage error:

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

On an error of `try_parse_from`, `run` takes the text of `error.render()` as plain text and replaces each control character in it other than the line feed with `?`. When `error.use_stderr()` is false, which is the case for `--help` and `--version`, the text goes to standard output. Otherwise it goes to standard error. A usage error covers an unknown subcommand or option, a missing signature, an argument that is not UTF-8, and `escher` with no argument. Section 7 gives the status of each case, and a failed write of the help or the version text follows Section 6.6.

`clap` copies an unknown argument into its error message with its control characters unchanged. Without the replacement, an argument that starts with `--` and holds an escape sequence reaches standard error unchanged.

CLI-15. `run` MUST replace every control character other than the line feed with `?` in the help, version, and usage text that `clap` produces, before it writes that text, because `clap` echoes an untrusted argument unchanged, and an echoed control character can send an escape sequence to the user's terminal.

The help text and the version text hold no control character other than the line feed, so the replacement leaves them unchanged.

### 6.6. Output Errors

CLI-10. `run` MUST return `Failure` when a write to standard output fails, and MUST NOT write a diagnostic when the failure is a broken pipe, because a status of 0 would report output that never arrived, while a diagnostic for a closed pipe, as in `escher info '*632' | head -c0`, reports the reader's choice as an error.

CLI-10 covers every write to standard output, including the help and the version text of Section 6.5. A failed flush counts as a failed write.

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

## 7. Streams and Exit Status

`run` returns these statuses. Section 3.3 gives their exit statuses, and Sections 6.5, 6.6, and 6.7 give the streams.

| Status | When |
| --- | --- |
| `Success` | `info` without `--check` wrote its text, whether or not the signature has a domain. `info --check` wrote its text, and every check passed. `--help` or `--version` wrote the help or the version text |
| `Failure` | The argument is a signature error of any kind, including a bad orbifold (`DEC-24`). A write to standard output failed (Section 6.6). With `--check`, a check failed or the signature has no domain (Section 6.7) |
| `Usage` | `clap` reports a usage error (Section 6.5) |

CLI-11. `run` MUST return `Failure` for every signature error, including a bad orbifold, because a caller that gets 0 for an invalid signature treats the missing information as valid (`DEC-24`). A caller that needs the error kind uses `escher-signature` directly.

CLI-19. `info --check` MUST return `Failure` when a check fails and when the signature has no domain, and MUST return `Success` only when every check passed, because a script that runs `--check` reads the exit status as the verdict, and a status of 0 for a signature with nothing to check reads as verified. Section 5.8 of `docs/design/architecture.md` gives the same reason for `render`.

## 8. Testing

### 8.1. Tests

The tests live in one integration-test binary, `crates/escher-cli/tests/cli/main.rs`, with the modules `info`, `diagnostics`, `usage`, and `common`, and the snapshots under `crates/escher-cli/tests/cli/snapshots/` (rule TEST-10 of `docs/rust-coding-standards.md`). `main.rs` has a `//!` line, which the lint `missing_docs` requires. Every test outside `main.rs` calls `run` with `Vec<u8>` streams, or with a test writer that fails. The two tests in `main.rs` test `Status` alone.

The column "Plan item" names the item of the field "Tests and acceptance" of Section 5.1 of `docs/design/poc-plan.md` that each test serves, or the field "Command line" of its Section 5.2. The tables, the enumeration, and the properties of the field of Section 5.1 belong to `escher-signature`, and the reference arithmetic of Section 5.2 belongs to `escher-geometry` and `escher-domain`.

| Test | Plan item | Kind and module | Asserts | Under Miri |
| --- | --- | --- | --- | --- |
| Information snapshots | Snapshots | `insta`, `info` | The text and status for `*732`, `*532`, `4*3`, `*3333`, `2*(12)`, `*22(inf)`, `*632`, `1`, `3*(inf)`, `*(inf)32`, `ox`, `*236`, and `2*`, whose area is exactly `pi`. By Section 7 of `docs/design/signature.md`, `*22(inf)` prints `*(inf)22`, `ox` prints `xxx`, `*236` prints `*632`, and every other input prints itself. Each text includes its `Domain` line, and the text of `*732` its domain summary | Ignored |
| Domain snapshots | Command line of Section 5.2 of the plan | `insta`, `info` | The text and status for `info '*542'`, `info '*433'`, `info '*(1001)32'`, `info --check '*732'`, `info --check '*632'`, and `info --check '*(1000)32'`. The plan writes the triangle $(3, 3, 4)$, whose canonical signature is `*433` | Ignored |
| Diagnostic snapshots | Snapshots, failure paths | `insta`, `diagnostics` | The diagnostic and status of every error kind of `escher-signature`, each text of a kind that has several except the `Overflow` texts for the group order and the deformation dimension, which no known input produces (Section 12.1 of `docs/design/signature.md`), `23`, `2*12`, an empty argument, an argument longer than 120 characters, a control character, a span after the infinity sign, and a span that is the infinity sign, under one caret | Ignored |
| Usage snapshots | Snapshots, failure paths | `insta`, `usage` | `--help`, `-h`, `info --help`, no argument, an unknown subcommand, the subcommand `help`, an unknown option, and a missing signature. The help of `info` shows `--check` | Ignored |
| Version tests | None | Plain tests, `usage` | `--version` and `-V` give `Success`, empty standard error, and standard output equal to the package name, one space, the package version, and a line feed. Each test takes the name and the version from the package when it compiles, so a new version needs no change to the tests | Runs |
| Echo limit tests | Failure paths | Plain tests, `diagnostics` | The status and the exact diagnostic for a signature of exactly 120 characters, which gets the echo line and the caret line, for one of 121 characters, which gets the byte span, and for one of 42 characters and 124 bytes, built with the infinity sign, which gets the echo line and the caret line because the limit counts characters | Runs |
| Status tests | Failure paths | Plain tests, `main.rs`, `usage`, and `info` | `Success` for `*732`, `Failure` for `23`, `Usage` for an unknown subcommand and for an argument that is not UTF-8, each with the start of the standard output or standard error that it writes, equal output for `*22(inf)` and the same signature with the infinity sign, and the exit statuses 0, 1, and 2 of the three values of `Status` | Runs |
| Verification status tests | Command line of Section 5.2 of the plan, failure paths | Plain tests, `info` | `Success`, empty standard error, and a standard output that ends with `Verification: passed` and a line feed for `info --check '*732'`, and the same with `--check` after the signature. `Failure`, the diagnostic of Section 6.7, and a standard output that ends with the `Domain` line for `info --check '*632'`. `Success` and empty standard error for `info '*632'` | Runs |
| Failed check text | Failure paths | Unit tests in `info.rs` | The private formatter, given a failed check as plain numbers, writes `failed, residual 2.5e-3, tolerance 1.0e-6`, writes `inf` for an infinite residual, and writes `NaN` when given a NaN, which no residual of `escher-domain` is, and the private function that writes the diagnostic of Section 6.7 writes `2 of 6 checks failed` for two failed checks | Runs |
| Output error tests | Failure paths | Plain tests, `info` and `usage` | A writer that fails with a broken pipe gives `Failure` and empty standard error, for `info`, for `info --check`, and for `--version`. A writer that fails with another error gives `Failure` and the diagnostic of Section 6.6, and so does a writer whose flush fails. A failing standard error keeps `Failure` after a signature error and `Usage` after a usage error | Runs |
| Control characters | Failure paths | Plain test, `usage` | A usage error for an argument with control characters writes `?` in place of each of them (CLI-15) | Runs |
| Auto traits | None | Plain test, `main.rs` | `Status` is `Send` and `Sync` (rule CONC-1 of `docs/rust-coding-standards.md`). The test calls a generic function whose type parameter is bounded by `Send + Sync`, so it fails to compile if `Status` loses either trait | Runs |
| Doctests | None | Doctests | The crate-level examples | Not run, because Miri runs through nextest, which runs no doctest |

The argument that is not UTF-8 is built with `std::os::unix::ffi::OsStringExt`, so that test compiles only on Unix, which covers Linux, the supported platform.

No known input makes a check fail, so the failed-check path is tested on the private formatter and the private diagnostic function, as Section 12.1 of `docs/design/signature.md` does for its `Overflow` texts.

A snapshot holds the status, the standard output, and the standard error of one `run` call, in this form, which a helper in the module `common` builds (rule TEST-12 of `docs/rust-coding-standards.md`):

```text
status: <Debug text of the Status>
--- stdout
<standard output>--- stderr
<standard error>--- end
```

Each stream follows its marker line unchanged, so a stream that ends with a line feed puts the next marker on a line of its own, and a stream that does not runs into it. The final marker `--- end` is needed because `insta` trims the whitespace at the end of a snapshot before it compares, so without it a missing or extra final line feed of standard error would pass. The writer that fails, `FailingWriter`, is also in `common`.

Every snapshot must be deterministic (rule TEST-2 of `docs/rust-coding-standards.md`), and two kinds of floating-point value in the output need a rule for that:

- The residual of a passed check is a rounding error, whose last digits can depend on the mathematics library of the platform. In every line that starts with `Check `, the snapshot helper replaces the text between `residual ` and the next `,` with `<residual>`, as the example of Section 5.3 shows. The tolerance, the result, and the line `Verification` stay in the snapshot. The test "Failed check text" covers the form of a residual.
- The values of rows 11 to 13 are printed with six digits after the point (Section 5.5). A snapshot input is chosen so that the exact value of each of these rows lies farther from a rounding boundary of the sixth digit than the bound of Section 9.3 of `docs/design/domain.md` in the column for the orders of that input, converted to degrees for an angle. The column "Orders up to 12" applies to `*732`, `*542`, and `*433`, and the column "(1000, 3, 2)" to `*(1000)32`. Every correct build then prints the same digits. A 256-bit computation of `*732` confirms this for every value of its summary, as Section 5.3 shows. The test "Triangles" of `docs/design/domain.md` checks the values of the other inputs against the reference arithmetic of `escher-domain`, within the bound of that column (Section 12.1 of that document). The change that adds the snapshots replaces an input whose printed value fails the condition.

CLI-20. A snapshot MUST NOT hold a residual of a check, and its inputs MUST meet the condition on rounding boundaries above, because a value that depends on the last bits of floating-point arithmetic makes the snapshot test fail on a platform whose mathematics library rounds differently, although the program is correct.

Every display text of `escher-signature` that a known input produces passes through a diagnostic snapshot, so `escher-signature` needs no `insta` dependency. Likewise, the snapshots of the `Domain` line cover the `UnsupportedSignature` and `LimitExceeded` texts of `escher-domain`.

This crate has no test that starts a process. The one smoke test of the binary as a process belongs to M3 (Section 5.3 of `docs/design/poc-plan.md`).

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

The snapshot tests carry `#[cfg_attr(miri, ignore = "the snapshot library reads snapshot files and starts a process")]`, because Miri's isolation refuses file access and process creation (`DEC-5`). The reason is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`). The domain snapshots carry the same attribute.

CLI-13. Every test that uses `insta` MUST carry that attribute, because without it `just check-miri` fails on every snapshot test.

The other tests of Section 8.1 run under Miri unchanged, because they use only memory, and `clap` runs under Miri: on 2026-11-16 every such test of rows 1 to 9, the diagnostics of Sections 6.1 to 6.6, and the usage text passed under `just check-miri`. A snapshot test run under Miri with its attribute overridden failed with `unsupported operation: 'open' not available when isolation is enabled`. A test that fails under Miri only because Miri does not support an operation is marked in the same way, with its own reason.

The plain tests that run `info` on `*732` build and verify its domain under Miri, and so run the arithmetic of `glam` and the floating-point functions of the standard library in the interpreter. The verification of `*732` needs few matrix products (Section 8.2 of `docs/design/domain.md`), so these tests need no attribute. They assert the status and the start or the end of the output, and no digit of a floating-point value, because the interpreter need not reproduce the last bits of the host library (Section 12.3 of `docs/design/domain.md`). The implementing change confirms that they pass under `just check-miri`.

## 9. Dependencies

### 9.1. Workspace Dependencies

The crate declares these dependencies, each from one entry in the root `[workspace.dependencies]` table (rule DEP-2 of `docs/rust-coding-standards.md`). The table also holds the entries that `docs/design/signature.md`, `docs/design/geometry.md`, and `docs/design/domain.md` specify for the other crates.

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
escher-signature = { version = "0.1.0", path = "crates/escher-signature" }
insta = { version = "1.49", default-features = false }
```

| Dependency | Kind | Features | Reason |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | Parses and classifies the signature. The path dependency carries a version (TOOL-25) |
| `escher-domain` | Normal | None | Builds and verifies the fundamental domain for the domain summary and `--check`. The path dependency carries a version (TOOL-25) |
| `clap` | Normal | The list above, without `color`, `env`, and `wrap_help` | The derive API (Section 3.1 of `docs/design/architecture.md`). Without `color`, `anstream` and its dependencies are not built (Section 9.2) |
| `anyhow` | Normal | Default | The chain of errors that Section 5.8 of `docs/design/architecture.md` requires. Version 1.0.103 fixes RUSTSEC-2026-0190 (Section 10 of `docs/research/rust-ecosystem.md`) |
| `insta` | Dev | None, so without `colors` | Snapshot tests (rule TEST-2). Without `colors`, `console` is not built (Section 9.2) |

The crate declares neither `escher-geometry` nor `glam`. The measurements of the domain are methods of `FundamentalDomain` that return plain `f64` values, so this crate names no `glam` type (Section 15.2 of `docs/design/domain.md`). `escher-domain` brings `escher-geometry`, `glam`, and `thiserror` into the build of the binary.

The package keeps `publish = false` (`DEC-22`) and declares no Cargo feature.

### 9.2. Verified Resolution

An adversarial test resolved these dependencies with network access on 2026-11-16, together with those of `escher-signature`, to the highest versions compatible with Rust 1.99. It covered every dependency of Section 9.1 except `escher-domain`, and verified these facts:

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

The dependencies that `escher-domain` brings were resolved on 2026-11-16 as Section 11 of `docs/design/geometry.md` records: `glam` adds no transitive dependency, no duplicate version, and no licence. `thiserror` is already in the workspace. The change that adds `escher-domain` to this crate confirms the resolution of the whole package with `just check-deny` and `just check-msrv`.

### 9.3. Dependency Risks

cargo-deny leaves dev-dependencies out of the licence check and the duplicate check by default. `deny.toml` overrides both defaults, as Section 8 of `docs/design/tooling.md` states, so `just check-deny` checks dev-dependencies. With the overrides, `cargo deny list` names `insta` 1.49.0, its dependencies such as `similar`, `tempfile`, `once_cell`, `fastrand`, and `rustix`, and `proptest`. In the test of Section 9.2, which ran without the overrides, it named none of `insta`, `similar`, `tempfile`, `proptest`, and `rand`.

CLI-14. The dev-dependencies of this crate MUST stay inside the dependency policy of Section 8 of `docs/design/tooling.md`, so that `cargo deny list` names `insta` and its dependencies, because otherwise the licences, advisories, and duplicates of the snapshot library go unchecked while the dependency policy appears to cover them.

## 10. Unverified Statements

This section lists the unverified statements of Section 7 of `docs/design/poc-plan.md` that this crate relies on, as PLAN-3 of that document requires. The crate relies on `UNV-20` and `UNV-21` through `escher-signature`, and on `UNV-26` through `escher-domain`.

| Id | How this crate relies on it |
| --- | --- |
| `UNV-20` | `info` prints the canonical signature, with parentheses for orders of 10 or more and `(inf)` for an infinite order, and accepts both spellings of infinity and `1` as the trivial signature |
| `UNV-21` | `info` prints the geometry and the area of a non-compact signature whose Euler characteristic is not 0, and prints no deformation dimension for a non-compact signature |
| `UNV-26` | `info --check` reports the checks of the walls of the domain, which `escher-domain` builds through the formula of the wall through two points that `UNV-26` covers. The walls, relators, and angles checks test the result (Section 14 of `docs/design/domain.md`). The domain summary does not depend on the walls, because `escher-domain` measures it from the vertices |

## 11. Assumptions, Choices, and Open Questions

### 11.1. Assumptions

- The infinity sign takes one column in the terminal. Unicode gives it an ambiguous East Asian width, so a terminal that renders such characters as wide shifts the caret one column to the right for each infinity sign before the span.
- With its features `env` and `wrap_help` off, `clap` reads no environment variable, so the output does not depend on the environment. A reading of the source of `clap_builder` 4.6.7 on 2026-11-16 found that it reads the environment only in `Arg::env`, which needs the feature `env`, and for `COLUMNS` and `LINES`, which need the feature `wrap_help`. No run has confirmed this.
- The residual of a passed check can differ in its last digits between platforms, which is why Section 8.1 replaces it in the snapshots. No run on two platforms has measured this. The replacement is safe either way.

### 11.2. Choices of This Design

These choices are made here, and no other document dictates them:

1. `run` returns `Status` and not `std::process::ExitCode`, and the `clap` types are private.
2. `run` flushes standard output after each write to it (Section 3.2).
3. `clap` has no colour and only the features of Section 9.1, and `insta` has no colour.
4. `escher` with no argument is a usage error that prints the help.
5. The subcommand `help` is disabled.
6. The labels and order of the information text, with `Frieze` and `General non-compact` printed only for a non-compact signature, and the label `Deformation dimension`. In M1 no parameter can be set, and the named list of parameters arrives with the signatures that have them (Section 5.4).
7. The format of the area, such as `2pi/3`.
8. The diagnostic format: the bound of 120 characters, `?` for each unsafe character, and the byte range for a longer argument.
9. Three statuses, with one status for every signature error (Section 7).
10. A failed write to standard output, including the help and the version text, gives `Failure`, and a broken pipe gives no diagnostic (Section 6.6).
11. Snapshots live only in this crate, with one snapshot for each `run` call.
12. No recipe wraps `cargo insta`.
13. The help, version, and usage text of `clap` has each control character other than the line feed replaced with `?` (Section 6.5).
14. A snapshot holds the status and each stream between fixed marker lines, with a final marker line (Section 8.1).
15. A `Domain` line for every valid signature, with `none, ` and the `Display` text of the error of `escher-domain` when no domain is built, and the status `Success` for such a signature without `--check` (Section 5.5).
16. The labels of rows 10 to 20, angles in degrees, and six digits after the point for the domain summary (Sections 5.1 and 5.2).
17. Every check line shows its result, its residual, and its tolerance in the form `{:.1e}`, and a final `Verification` line gives the overall result (Section 5.6).
18. With `--check`, a failed check and a missing domain each give `Failure` and a diagnostic (Section 6.7).
19. The information text is written before the verification diagnostic (Section 6.7).
20. The snapshot helper replaces the residual of each check line, and the snapshot inputs keep the printed digits away from rounding boundaries (Section 8.1).
21. The `clap` flag `--check` is a `bool` converted at once into a private enum (Section 4.3).

### 11.3. Points for the Owner

None of these blocks the implementation:

1. Order of infinity (Section 15.3, item 1, of `docs/design/signature.md`, which owns it). The canonical form sorts an infinite order above every finite order, so `*22(inf)` prints as `*(inf)22`, and the examples of Section 5.3 assume this. If the owner chooses the other order, the canonical strings in the examples and the snapshots change.
2. Area of a non-compact hyperbolic signature. `info` prints it, as Section 5.3 shows for `3*(inf)`, and the value rests on `UNV-21` (Section 15.3, item 3, of `docs/design/signature.md`). The stricter option prints the area only for a compact signature until a source covers the non-compact case.
3. The trivial group as `11`. The rejection of a digit `1` followed by another digit also rejects `11` and `*11`, which ON 3.5 records as spellings in [CBG] and [CH] (Section 15.3, item 2, of `docs/design/signature.md`, which owns it). This changes what `info` accepts.
4. The largest corner order of 1000 (Section 15.3, item 2, of `docs/design/domain.md`, which owns it). `escher info '*(2000)32'` prints `Domain: none, cannot build fundamental domain: order 2000 is larger than 1000` and exits with status 0. A higher limit changes the snapshot of `*(1001)32`.
5. Check 6, the area check (Section 15.3, item 4, of `docs/design/domain.md`, which owns it). If the owner drops it, `--check` prints five check lines, and the diagnostic counts five checks.
6. Two area lines. A signature with a domain prints both `Area`, the exact value from the Euler characteristic, and `Domain area`, the area measured from the angles of the domain. The design keeps both, because their agreement is visible to a reader. The simpler option drops `Domain area`.
7. A `Domain` line for every signature. The design prints `Domain: none, ...` for every valid signature without a domain, so that a reader can tell a missing domain from a missing line. The other option prints no `Domain` line for such a signature, which keeps the text of those signatures unchanged.

## 12. Security Considerations

The threat model is a user who passes a signature from an untrusted party to `escher info`. The attacker can choose any argument within the limits of the command line, including invalid UTF-8, control characters, and very long text. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory or time through the signature, and injection of terminal escape sequences through the diagnostic. `escher-signature` sets the limits on the length of the signature, the number of features, and the size of an order, as Section 16 of `docs/design/signature.md` states. `escher-domain` limits the largest corner order of a domain, which bounds the cost of `verify()`, as Section 16 of `docs/design/domain.md` states. This crate adds no limit of its own. The echo line repeats at most 120 characters and replaces every character outside printable ASCII and the infinity sign (CLI-9). The caret line holds only spaces and `^`. The `Domain` line and the diagnostics of Section 6.7 hold only fixed text, numbers, and the canonical signature, which is printable ASCII. The help, version, and usage text that `clap` produces has every control character other than the line feed replaced with `?` (CLI-15).

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, and the workspace denies `unsafe_code`.
- Files: the crate reads no file and writes no file. It writes only to the two streams that `run` receives.
- Build scripts: the crate has none.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones (Section 9.3). Only `insta` is a dev-dependency, and it never reaches the binary.

Attacks on the repository, the build machine, or the terminal itself are out of scope, because this document governs the crate and not access control or the terminal.

The residual risk is that a usage error still echoes every character of an unknown argument that is not a control character, including characters outside ASCII, such as the formatting characters of the Unicode general category Cf. The replacement tests only for the category Cc. A second residual risk is a domain that is wrong by less than the tolerances, which `--check` reports as passed (Section 16 of `docs/design/domain.md`).

## 13. Operational Considerations

The crate declares no Cargo feature and has no build script, so no environment variable affects the build. At run time the crate reads no environment variable (Section 11.1). The snapshot tests read the variables of Section 8.2, which Section 14.2 of `docs/design/tooling.md` lists.

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

The crate emits no log.

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
| CLI-2 | 3.2 | `run` writes only to its two streams and never panics. |
| CLI-3 | 3.4 | `main.rs` only calls `run` and converts the status. |
| CLI-4 | 4.2 | `--version` and `-V` print the package name and version on standard output with `Success`. |
| CLI-5 | 4.3 | `info` passes its argument unchanged to `escher-signature`. |
| CLI-6 | 5.1 | `info` prints exactly the lines of Section 5.1, in order and under their conditions, with no name of the group in another notation. |
| CLI-7 | 5.4 | A change to the information text updates its snapshots in the same change. |
| CLI-8 | 6.1 | Every diagnostic goes to standard error. |
| CLI-9 | 6.2 | The echo line replaces unsafe characters with `?` and holds at most 120 characters. |
| CLI-10 | 6.6 | A failed write to standard output gives `Failure`, with no diagnostic for a broken pipe. |
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
