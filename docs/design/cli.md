# Command-Line Interface Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-16 |
| Applies to | `crates/escher-cli`: the library target, the binary `escher`, and its tests |

## Abstract

This document specifies the package `escher-cli`, which provides the command-line program `escher`. It defines the public surface of the library target, the thin binary that calls it, the commands and options, the exact text that the command `info` prints for each kind of signature, the format of every diagnostic, the exit statuses, the tests, and the dependencies.

## 1. Introduction

### 1.1. Problem

The proof of concept is a command-line tool, and its first command reports what a signature in orbifold notation describes. The tool reads untrusted text, prints text that a reviewer checks line by line, and must report every malformed signature with a pointer to the offending characters. A program whose logic lives in `main` can only be tested by starting a process, which is slow and cannot run under Miri.

### 1.2. Purpose

This document fixes the behaviour of `escher` precisely enough that an agent can implement it and a reviewer can check every line of output against it. The crate `escher-signature` parses and classifies a signature. This crate turns its results into text, diagnostics, and an exit status.

### 1.3. Audience

The primary readers are the coding agents that implement or extend `escher-cli`. The secondary readers are reviewers and the owner, who decides the points of Section 11.3.

### 1.4. Scope

This document covers the command `info` with its single argument, the options `--help` and `--version`, the diagnostics, the exit statuses, the tests of the crate, and its dependencies. `info` parses, validates, canonicalizes, and classifies any signature, and prints no item that needs a fundamental domain or a group.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The grammar of a signature, its canonical form, the classification methods, and the error kinds of `escher-signature` with their `Display` text are specified in `docs/design/signature.md`.
- The crates, the data flow, the error model, and the owner's decisions are specified in `docs/design/architecture.md`.
- The mathematics is stated in `docs/research/orbifold-notation.md`.
- The recipes, the nextest profiles, the hooks, and the dependency policy are specified in `docs/design/tooling.md`.

The command `render`, its options, and the lines of `info` that need a domain, a group, or a colouring belong to later milestones (Section 5.4). So do the handling of an existing output file (Section 11 of `docs/design/architecture.md`) and the diagnostic for a general non-compact signature in `render` (Section 5.8 of that document).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md` and of Section 2.1 of `docs/design/architecture.md`, such as signature, canonical signature, geometry, and general non-compact signature. It adds these terms:

- ON: The reference `docs/research/orbifold-notation.md`, cited as `ON` and a section number, such as ON 7.3.
- command: A subcommand of `escher`, such as `info`.
- information text: The text that `escher info` writes to standard output for a valid signature (Section 5).
- diagnostic: The text that `escher` writes to standard error when it fails (Section 6).
- echo line: The line of a diagnostic that repeats the signature (Section 6.2).
- caret line: The line of a diagnostic that marks the span of an error with `^` characters under the echo line (Section 6.3).
- signature error: An error of `escher-signature` for an argument that is not a valid signature, including a bad orbifold.
- usage error: An error that `clap` reports for the command line itself, such as an unknown subcommand or a missing argument.
- output error: A failed write to standard output or standard error.
- broken pipe: The output error whose `std::io::ErrorKind` is `BrokenPipe`, which a write returns when the reader of a pipe has closed it.
- status: The value of the type `Status` that `run` returns, which the binary converts to the exit status of the process.
- snapshot: A file of expected output that an `insta` test compares with the actual output.

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
    T: Into<std::ffi::OsString> + Clone;

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

`run` flushes `stdout` before it returns, so that a write error that buffering delays still reaches the status.

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

The binary prints nothing itself, so `main.rs` carries no lint exception for `clippy::print_stdout`, which would fail as an unfulfilled expectation. Section 4.4 of `docs/rust-coding-standards.md` and the entry of `main.rs` in Repository Layout of `AGENTS.md` describe such an exception, and the change that implements this design updates both.

### 3.5. Private Modules

The library target has the private modules `args`, `info`, and `diagnostic`, as files named `args.rs`, `info.rs`, and `diagnostic.rs` (Section 4.5 of `docs/rust-coding-standards.md`). `args` holds the `clap` types, `info` builds the information text, and `diagnostic` writes the diagnostics.

## 4. Command Line

### 4.1. Synopsis

```text
escher info <SIGNATURE>
escher --help
escher --version
```

`-h` is the short form of `--help` and `-V` of `--version`. `escher info --help` prints the help of `info`.

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

CLI-4. `escher --version` and `escher -V` MUST write the package name, one space, the package version, and a newline to standard output, which is `escher-cli 0.1.0` for version 0.1.0, and return `Success`, because Section 5.1 of `docs/design/poc-plan.md` keeps this output, and a script may read it.

### 4.3. The `info` Command

`info` takes one required positional argument, the signature, as a `String` named `SIGNATURE`. An argument that is not valid UTF-8 is a usage error.

CLI-5. `info` MUST pass its argument unchanged to the `FromStr` implementation of `Signature` from `escher-signature`, and MUST NOT trim, normalize, or check it first, because a second reading of the grammar diverges from the one that `docs/design/signature.md` specifies, and the byte spans of the error then no longer match the argument.

On success, `info` writes the information text of Section 5 to standard output. On a signature error, it writes the diagnostic of Section 6 to standard error. A bad orbifold is a signature error (`DEC-24`).

A signature never starts with `-`, so it never collides with an option. An argument that starts with `-` is read as an option, and an unknown option is a usage error.

### 4.4. Shell Quoting

In a POSIX shell, `*`, `(`, and `)` are special characters, so a signature that contains any of them is quoted in single quotes, as in `escher info '2*(12)'`. The shell passes every character inside single quotes unchanged. The help text of `info` says this, and every example in this document that needs it quotes the signature.

## 5. Information Text

### 5.1. Lines

`info` writes the lines of this table, in this order. Each line is the label, a colon, one space, and the value, followed by a line feed. No other line, blank line, or trailing space appears. The accessors are those of `Signature` (Sections 3.1 and 9.1 of `docs/design/signature.md`).

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

CLI-6. `info` MUST print exactly the lines of this table, in this order and under these conditions, because the snapshots of Section 8 and every reader of the text rely on one layout, and an extra or missing line changes what a reader concludes about the signature. The table has no name of the group in another notation, such as an IUC or Schoenflies name, because `DEC-20` omits them until a primary source is read.

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

### 5.3. Examples

The examples follow the canonical form that `docs/design/signature.md` defines. Each value cites the section of ON that gives it or its formula. The deformation dimension applies the formula of ON 10.1 with the reading of the Euler characteristic of the underlying surface that Section 15.1 of `docs/design/signature.md` records.

This example shows a compact hyperbolic signature:

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
```

The Euler characteristic comes from the defect rule of ON 6.1, the area from ON 7.3, and the dimension from ON 10.1. Section 5.1 of `docs/design/poc-plan.md` states all three values.

This example shows a spherical signature:

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
```

The Euler characteristic 2 is the defect rule of ON 6.1 with no defect. The order is that of `nn` with $n = 1$ in ON 8.2, and the area comes from ON 7.3.

### 5.4. Stability and Omissions

The information text is not a stable interface, and a machine-readable form is a roadmap item (`DEC-23`).

CLI-7. A change that alters the information text MUST update its snapshots in the same change, because the snapshot test fails otherwise, and the snapshot diff is how a reviewer sees each change to the text (`DEC-23`, rule TEST-2 of `docs/rust-coding-standards.md`).

`info` prints no line about parameters, domains, presentations, colourings, polygons, or colours, and no placeholder for them. Each needs a crate that a later milestone adds. Section 3.4 of `docs/design/poc-plan.md` places them: the domain summary in M2 (its Section 5.2), the named parameters in M4 (Section 5.4), the presentation in M7 (Section 5.7), and the list of colourings with their colour counts in M8 (Section 5.8). The owner's goal that `info` reports how many polygons the representative set has, and so which colour counts are possible, is met by that list in M8.

## 6. Diagnostics

### 6.1. Format

CLI-8. `run` MUST write every diagnostic to standard error and none to standard output, because a caller that captures standard output must receive only the information text, or else an error is read as data.

The diagnostic of a signature error or an output error has this form, and Section 6.5 covers a usage error:

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

On an error of `try_parse_from`, `run` writes the text of `error.render()` as plain text. When `error.use_stderr()` is false, which is the case for `--help` and `--version`, the text goes to standard output. Otherwise it goes to standard error. A usage error covers an unknown subcommand or option, a missing signature, an argument that is not UTF-8, and `escher` with no argument. Section 7 gives the status of each case.

### 6.6. Output Errors

CLI-10. `run` MUST return `Failure` when a write to standard output fails, and MUST NOT write a diagnostic when the failure is a broken pipe, because a status of 0 would report output that never arrived, while a diagnostic for a closed pipe, as in `escher info '*632' | head -c0`, reports the reader's choice as an error.

When a write to standard output fails in any other way, `run` writes this diagnostic to standard error:

```text
error: cannot write to standard output
  caused by: <Display text of the std::io::Error>
```

A failed write to standard error is discarded in the form of rule ERR-5 of `docs/rust-coding-standards.md`, and the status stays what it would have been.

This is a choice of this design. It is the simplest behaviour that never panics: Rust ignores the signal `SIGPIPE`, so a write to a closed pipe returns an error, and the `println!` macro panics on that error.

## 7. Streams and Exit Status

`run` returns these statuses. Section 3.3 gives their exit statuses, and Sections 6.5 and 6.6 give the streams.

| Status | When |
| --- | --- |
| `Success` | `info` wrote its text, or `--help` or `--version` was given |
| `Failure` | The argument is a signature error of any kind, including a bad orbifold (`DEC-24`), or a write to standard output failed (Section 6.6) |
| `Usage` | `clap` reports a usage error (Section 6.5) |

CLI-11. `run` MUST return `Failure` for every signature error, including a bad orbifold, because a caller that gets 0 for an invalid signature treats the missing information as valid (`DEC-24`). A caller that needs the error kind uses `escher-signature` directly.

## 8. Testing

### 8.1. Tests

The tests live in one integration-test binary, `crates/escher-cli/tests/cli/main.rs`, with the modules `info`, `diagnostics`, `usage`, and `common`, and the snapshots under `crates/escher-cli/tests/cli/snapshots/` (rule TEST-10 of `docs/rust-coding-standards.md`). `main.rs` has a `//!` line, which the lint `missing_docs` requires. Every test calls `run` with `Vec<u8>` streams, or with a test writer that fails.

The column "Plan item" names the item of the field "Tests and acceptance" of Section 5.1 of `docs/design/poc-plan.md` that each test serves. The tables, the enumeration, and the properties of that field belong to `escher-signature`.

| Test | Plan item | Kind and module | Asserts | Under Miri |
| --- | --- | --- | --- | --- |
| Information snapshots | Snapshots | `insta`, `info` | The text and status for `*732`, `*532`, `4*3`, `*3333`, `2*(12)`, `*22(inf)`, `*632`, `1`, `3*(inf)`, `*(inf)32`, `ox`, and `*236`. By Section 7 of `docs/design/signature.md`, `*22(inf)` prints `*(inf)22`, `ox` prints `xxx`, `*236` prints `*632`, and every other input prints itself | Ignored |
| Diagnostic snapshots | Snapshots, failure paths | `insta`, `diagnostics` | The diagnostic and status of every error kind of `escher-signature`, each text of a kind that has several except the `Overflow` texts for the group order and the deformation dimension, which no known input produces (Sections 9.3 and 9.5 of `docs/design/signature.md`), `23`, `2*12`, an empty argument, an argument longer than 120 characters, a control character, and a span after the infinity sign | Ignored |
| Usage snapshots | Snapshots, failure paths | `insta`, `usage` | `--help`, `-h`, `info --help`, `--version`, `-V`, no argument, an unknown subcommand, the subcommand `help`, an unknown option, and a missing signature | Ignored |
| Status tests | Failure paths | Plain tests, `usage` and `info` | `Success` for `*732`, `Failure` for `23`, `Usage` for an unknown subcommand and for an argument that is not UTF-8, and equal output for `*22(inf)` and the same signature with the infinity sign | Runs |
| Output error tests | Failure paths | Plain tests, `info` | A writer that fails with a broken pipe gives `Failure` and empty standard error. A writer that fails with another error gives `Failure` and the diagnostic of Section 6.6 | Runs |
| Auto traits | None | Compile-time assertion | `Status` is `Send` and `Sync` (rule CONC-1 of `docs/rust-coding-standards.md`) | Runs |
| Doctests | None | Doctests | The crate-level examples | Not run, because Miri runs through nextest, which runs no doctest |

The argument that is not UTF-8 is built with `std::os::unix::ffi::OsStringExt`, so that test compiles only on Unix, which covers Linux, the supported platform.

A snapshot holds the status, the standard output, and the standard error of one `run` call. Every display text of `escher-signature` that a known input produces passes through a diagnostic snapshot, so `escher-signature` needs no `insta` dependency.

This crate has no test that starts a process. The one smoke test of the binary as a process belongs to M3 (Section 5.3 of `docs/design/poc-plan.md`).

### 8.2. Snapshots Under nextest

The recipe `test` runs the snapshot tests through nextest like every other test, with no setting of their own. A snapshot test fails when the output differs from its committed snapshot file, and an agent reviews and accepts a change with `cargo insta review`, which the coding standard lists as run by hand (Section 4.8 of `docs/rust-coding-standards.md`). No recipe wraps `cargo insta`.

A search of the source of `insta` 1.49.0 on 2026-11-16 found that it reads these environment variables:

- `CI`, `INSTA_UPDATE`, `INSTA_FORCE_PASS`, `INSTA_FORCE_UPDATE`, `INSTA_FORCE_UPDATE_SNAPSHOTS`, `INSTA_OUTPUT`, `INSTA_REQUIRE_FULL_MATCH`, and `INSTA_WORKSPACE_ROOT`.
- `INSTA_TEST_RUNNER`, `INSTA_TEST_RUNNER_FALLBACK`, `INSTA_GLOB_FILTER`, `INSTA_GLOB_FAIL_FAST`, `INSTA_PENDING_DIR`, `INSTA_SNAPSHOT_REFERENCES_FILE`, `INSTA_WARNINGS_FILE`, `INSTA_YAML_BLOCK_STYLE`, and `INSTA_DIFF_TOOL`.
- `NEXTEST_RUN_ID`, `TF_BUILD`, and `CARGO`.

That search read the source and ran no test. The change that adds `insta` confirms these points by running the tests:

- The behaviour of each variable when it is unset.
- Whether `insta` writes pending snapshot files under nextest, locally and in CI.
- Whether `insta` starts a `cargo metadata` process to find the workspace root unless `INSTA_WORKSPACE_ROOT` gives it, and whether that process succeeds in a nextest run.

CLI-12. An agent MUST NOT set `INSTA_FORCE_PASS` in a recipe, a hook, or a workflow, because it turns a failing snapshot test into a pass, and a changed text then reaches `main` unreviewed.

The change that adds `insta` updates these statements, which it would otherwise make false:

- Section 14.2 of `docs/design/tooling.md`, which lists every environment variable that the checks read. It gains at least `INSTA_UPDATE`, `CI`, `INSTA_FORCE_PASS`, `INSTA_WORKSPACE_ROOT`, and `INSTA_OUTPUT`, each with its confirmed behaviour when unset.
- Section 8 of `docs/design/tooling.md`, for the licences that `deny.toml` gains (Section 9.2) and the coverage of dev-dependencies (Section 9.3).
- `.gitignore`, which gains the pending snapshot files that the confirmation above finds, expected to be `*.snap.new` and `*.pending-snap`.

Section 6 of `docs/design/architecture.md` names the documents that state how a test that cannot run under Miri is marked, which the first change that adds tests updates.

### 8.3. Tests Under Miri

The snapshot tests carry `#[cfg_attr(miri, ignore = "the snapshot library reads snapshot files and starts a process")]`, because Miri's isolation refuses file access and process creation (`DEC-5`). The reason is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

CLI-13. Every test that uses `insta` MUST carry that attribute, because without it `just check-miri` fails on every snapshot test.

The other tests of Section 8.1 run under Miri unchanged, because they use only memory. The design expects `clap` to run under Miri, which no run has confirmed. A test that fails under Miri only because Miri does not support an operation is marked in the same way, with its own reason.

## 9. Dependencies

### 9.1. Workspace Dependencies

The crate declares these dependencies, each from one entry in the root `[workspace.dependencies]` table (rule DEP-2 of `docs/rust-coding-standards.md`). The table also holds the entries that `docs/design/signature.md` specifies for `escher-signature`.

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
escher-signature = { version = "0.1.0", path = "crates/escher-signature" }
insta = { version = "1.49", default-features = false }
```

| Dependency | Kind | Features | Reason |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | Parses and classifies the signature. The path dependency carries a version (TOOL-25) |
| `clap` | Normal | The list above, without `color`, `env`, and `wrap_help` | The derive API (Section 3.1 of `docs/design/architecture.md`). Without `color`, `anstream` and its dependencies are expected not to be built, which the change confirms with `cargo tree` |
| `anyhow` | Normal | Default | The chain of errors that Section 5.8 of `docs/design/architecture.md` requires. Version 1.0.103 fixes RUSTSEC-2026-0190 (Section 10 of `docs/research/rust-ecosystem.md`) |
| `insta` | Dev | None, so without `colors` | Snapshot tests (rule TEST-2). Without `colors`, `console` is expected not to be built, which the change confirms with `cargo tree` |

The package keeps `publish = false` (`DEC-22`) and declares no Cargo feature.

### 9.2. Verified Resolution

An adversarial test resolved these dependencies with network access on 2026-11-16, together with those of `escher-signature`, to the highest versions compatible with Rust 1.99. It verified these facts:

- The resolved versions include `anyhow` 1.0.104, `clap`, `clap_builder`, and `clap_derive` 4.6.7, `clap_lex` 1.1.1, `strsim` 0.11.1, `heck` 0.5.0, `syn` 3.0.10, `proc-macro2` 1.0.107, `quote` 1.0.47, `unicode-ident` 1.0.27, `insta` 1.49.0, `similar` 2.7.0, and `tempfile` 3.27.0.
- `cargo tree -d --target all` printed no duplicate version. The graph has one major version of `syn`, so `syn` 2 and 3 do not both occur.
- With the allow-list `Apache-2.0`, `GPL-3.0-or-later`, `MIT`, and `Unicode-3.0`, `cargo deny check licenses bans` passed with no unused licence. Without `MIT` it failed only on `strsim`, which the `clap` feature `suggestions` brings. `MIT` is therefore the one licence that this crate adds beyond those of `escher-signature`.
- The advisory and source checks passed.
- The highest minimum Rust version that the dependencies of this crate declare is 1.85, for `clap`, within `rust-version = "1.99"`.
- A prototype of this crate printed `escher-cli 0.1.0` for `--version` and `-V` on standard output with status 0, and the help for `--help` on standard output with status 0. With no argument it printed the help on standard error with status 2. A missing signature, an unknown subcommand, `-x`, and an argument that is not UTF-8 each gave an error on standard error with status 2. The usage line named `escher`.

Before it adds `MIT`, the change confirms that the licence is compatible with GPL-3.0-or-later, as TOOL-24 of `docs/design/tooling.md` requires.

### 9.3. Dependency Risks

In the test of Section 9.2, cargo-deny 0.20.2, the version in the development container, did not report on dev-dependencies. `cargo deny list` named none of `insta`, `similar`, `tempfile`, `proptest`, and `rand`. The licence check passed with an allow-list without `Apache-2.0`, although `insta` and `similar` are under `Apache-2.0` alone. Setting `exclude-dev = false` in the `[graph]` table changed nothing, and the cause was not isolated.

CLI-14. The change that adds `insta` MUST establish whether `just check-deny` checks dev-dependencies, find the setting that includes them if it does not, and record the result in Section 8 of `docs/design/tooling.md`, because otherwise the licences, advisories, and duplicates of the snapshot library go unchecked while the dependency policy appears to cover them.

## 10. Unverified Statements

This section lists the unverified statements of Section 7 of `docs/design/poc-plan.md` that this crate relies on, as PLAN-3 of that document requires. The crate relies on both through `escher-signature`.

| Id | How this crate relies on it |
| --- | --- |
| `UNV-20` | `info` prints the canonical signature, with parentheses for orders of 10 or more and `(inf)` for an infinite order, and accepts both spellings of infinity and `1` as the trivial signature |
| `UNV-21` | `info` prints the geometry and the area of a non-compact signature whose Euler characteristic is not 0, and prints no deformation dimension for a non-compact signature |

## 11. Assumptions, Choices, and Open Questions

### 11.1. Assumptions

- The infinity sign takes one column in the terminal. Unicode gives it an ambiguous East Asian width, so a terminal that renders such characters as wide shifts the caret one column to the right for each infinity sign before the span.
- With its features `env` and `wrap_help` off, `clap` reads no environment variable, so the output does not depend on the environment. No run has confirmed this.
- `clap` echoes the text of an unknown argument in a usage error. Whether it escapes control characters in that text is not verified (Section 12).

### 11.2. Choices of This Design

These choices are made here, and no other document dictates them:

1. `run` returns `Status` and not `std::process::ExitCode`, and the `clap` types are private.
2. `run` flushes standard output before it returns.
3. `clap` has no colour and only the features of Section 9.1, and `insta` has no colour.
4. `escher` with no argument is a usage error that prints the help. The placeholder binary prints its version with no argument instead.
5. The subcommand `help` is disabled.
6. The labels and order of the information text, with `Frieze` and `General non-compact` printed only for a non-compact signature, and the label `Deformation dimension`. In M1 no parameter can be set, and the named list of parameters arrives with the signatures that have them (Section 5.4).
7. The format of the area, such as `2pi/3`.
8. The diagnostic format: the bound of 120 characters, `?` for each unsafe character, and the byte range for a longer argument.
9. Three statuses, with one status for every signature error (Section 7).
10. A failed write to standard output gives `Failure`, and a broken pipe gives no diagnostic (Section 6.6).
11. Snapshots live only in this crate, with one snapshot for each `run` call.
12. No recipe wraps `cargo insta`.

### 11.3. Points for the Owner

None of these blocks the implementation:

1. Order of infinity (Section 15.3, item 1, of `docs/design/signature.md`, which owns it). The canonical form sorts an infinite order above every finite order, so `*22(inf)` prints as `*(inf)22`, and the examples of Section 5.3 assume this. If the owner chooses the other order, the canonical strings in the examples and the snapshots change.
2. Area of a non-compact hyperbolic signature. `info` prints it, as Section 5.3 shows for `3*(inf)`, and the value rests on `UNV-21` (Section 15.3, item 3, of `docs/design/signature.md`). The stricter option prints the area only for a compact signature until a source covers the non-compact case.
3. The trivial group as `11`. The rejection of a digit `1` followed by another digit also rejects `11` and `*11`, which ON 3.5 records as spellings in [CBG] and [CH] (Section 15.3, item 2, of `docs/design/signature.md`, which owns it). This changes what `info` accepts.

## 12. Security Considerations

The threat model is a user who passes a signature from an untrusted party to `escher info`. The attacker can choose any argument within the limits of the command line, including invalid UTF-8, control characters, and very long text. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory or time through the signature, and injection of terminal escape sequences through the diagnostic. `escher-signature` sets the limits on the length of the signature, the number of features, and the size of an order, as Section 16 of `docs/design/signature.md` states, and this crate adds no limit of its own. The echo line repeats at most 120 characters and replaces every character outside printable ASCII and the infinity sign (CLI-9). The caret line holds only spaces and `^`.

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, and the workspace denies `unsafe_code`.
- Files: the crate reads no file and writes no file. It writes only to the two streams that `run` receives.
- Build scripts: the crate has none.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under Section 8 of `docs/design/tooling.md`, with the gap of Section 9.3 for dev-dependencies. Only `insta` is a dev-dependency, and it never reaches the binary.

Attacks on the repository, the build machine, or the terminal itself are out of scope, because this document governs the crate and not access control or the terminal.

The residual risk is a usage error from `clap` that echoes an unknown argument with its control characters unchanged, which is not verified (Section 11.1). A second residual risk is the gap of Section 9.3, until the change that adds `insta` closes it.

## 13. Operational Considerations

The crate declares no Cargo feature and has no build script, so no environment variable affects the build. At run time the crate reads no environment variable (Section 11.1). The snapshot tests read the variables of Section 8.2, which Section 14.2 of `docs/design/tooling.md` lists.

The supported platform is Linux. The minimum supported Rust version is the one that Technology Decisions of `AGENTS.md` states, and the dependencies need no later version (Section 9.2).

| Failure | Reported as | Recovery |
| --- | --- | --- |
| The command line is wrong | The `clap` message on standard error, with `Usage` | Correct the command line, with `escher --help` |
| The signature is malformed, over a limit, or a bad orbifold | The diagnostic of Section 6, with `Failure` | Correct the signature at the caret |
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
| CLI-14 | 9.3 | The change that adds `insta` establishes whether cargo-deny checks dev-dependencies and records it. |
