# Command-Line Interface Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-24 |
| Applies to | `crates/escher-cli`: the library target, the binary `escher`, and its tests |

## Abstract

This document specifies the package `escher-cli`, which provides the command-line program `escher`. It defines the public surface of the library target, the thin binary that calls it, the commands and options, the exact text that the command `info` prints for each kind of signature, including the Schlafli symbols of a regular tiling, the shape parameters, the summary of the fundamental domain, the number of valid colourings for each colour count with the options `--max-colours` and `--colourings`, the result of each check of the option `--check`, and the presentations and the group order by coset enumeration of the option `--presentation`, the options of the command `render`, including the settings of shape parameters, the colouring by an id with its default colours and its image fills, the unit of the border width, the Wythoff construction of a reflection triangle from the marks of `--wythoff`, the whole-face fill of a regular tiling or of a construction with several face types from the arrangement of `--arrangement`, the view centred on a corner of `--centre`, the PNG file that it writes and the statistics that it prints, the arrangement lines that `info --wythoff` prints, the command `render-masks` with the polygon masks and the face mask that it writes, the format of every diagnostic, the exit statuses, the tests, and the dependencies.

## 1. Introduction

### 1.1. Problem

The proof of concept is a command-line tool, and its first command reports what a signature in orbifold notation describes. The tool reads untrusted text, prints text that a reviewer checks line by line, and must report every malformed signature with a pointer to the offending characters. A program whose logic lives in `main` can only be tested by starting a process, which is slow.

### 1.2. Purpose

This document fixes the behaviour of `escher` precisely enough that an agent can implement it and a reviewer can check every line of output against it. The crate `escher-signature` parses and classifies a signature, the crate `escher-domain` builds and verifies its fundamental domain, the crate `escher-group` derives presentations, enumerates their cosets, and searches their colourings, and the crate `escher-render` renders the domain into pixels and encodes them as PNG. This crate turns their results into text, a file, diagnostics, and an exit status.

### 1.3. Audience

The primary readers are the coding agents that implement or extend `escher-cli`. The secondary readers are reviewers and the owner, who decides the points of Section 11.3.

### 1.4. Scope

This document covers the command `info` with its argument and its options `--check`, `--presentation`, `--colourings`, `--max-colours`, and `--wythoff`, the command `render` with its argument and its options, the command `render-masks` with its argument and its options, the options `--help` and `--version`, the diagnostics, the exit statuses, the tests of the crate, and its dependencies. `info` parses, validates, canonicalizes, and classifies any signature. It prints the Schlafli symbols of the regular tiling of the signature and of its dual where the signature has one, and the shape parameters of the signature. For a signature that `escher-domain` constructs, it also prints a summary of the fundamental domain, the number of its valid colourings for each colour count, with `--colourings` one valid colouring for each conjugacy class with its id, and with `--check` the result of each check of its verification. With `--presentation`, it prints the Conway presentation of the signature and the side presentation of its domain, and for a spherical signature the group order that coset enumeration finds for each.

`render` draws, with `escher-render`, the tiling of a signature that `escher-domain` constructs with the shape parameters that the option `--set` gives, in one colour, in its valid colouring with two colours, or in the colouring that an id names. It draws a hyperbolic tiling in the Poincare disk, or in the upper half-plane, the band, or the Klein disk that the option `--model` names, a Euclidean tiling in the Euclidean plane, and a spherical tiling as a lit ball, with the viewing rotation, the light, and the shading of its options, with borders whose width is in pixels or, with `--border-unit intrinsic`, in the metric of the geometry, and writes it to a PNG file. With `--centre`, it centres the picture on a corner of the domain, named by its position in the order in which `info` prints the corner angles (`DEC-130`, `DEC-131`). With `--stats`, it also prints statistics of the rendering. A fill colour of `--colours` can be an image file, `image:FILE`, which `render` reads and samples as an image fill. On a reflection triangle, `--wythoff` turns each of the three mirrors on or off and draws the faces of the Wythoff construction, such as the regular tiling `{7,3}` of `*732`, with one colour for each face type, in place of the tiles. On a regular tiling, `--arrangement` with an id that `info --wythoff` lists puts a picture on every face, turned or mirrored as that arrangement says, from one picture for every face or from several pictures, one for each orbit of the arrangement (`DEC-111`, `DEC-119`, `DEC-124`). On a construction with several face types, such as `111` of `*632`, one arrangement puts pictures on the faces of every type, one or more for each type, and every edge matches (`DEC-137`).

`render-masks` writes, for a signature, its shape parameters, and a colouring, one polygon mask for each colour to a PNG file of its own, numbered by the colour. The user paints an image over each mask with an external tool and passes the images to `render` as image fills (`DEC-66`). With `--wythoff` and `--arrangement`, it writes one face mask for each picture of that arrangement, a flat regular polygon over which the user paints that picture.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The grammar of a signature, its canonical form, the classification methods, and the error kinds of `escher-signature` with their `Display` text are specified in `docs/design/signature.md`.
- The signatures that have a fundamental domain, its construction, its measurements, the checks of its verification with their tolerances, the shape parameters with their names, ranges, and defaults, and the error kinds of `escher-domain` with their `Display` text are specified in `docs/design/domain.md`.
- The side presentation of a domain, its generators, and its relators are specified in `docs/design/domain.md`.
- Words, relators, presentations and their text, the Conway presentation of a signature, coset enumeration with its limit, the search of the colourings with its limits, the text and the order of a colouring id (GRP-37), `colour_of` (GRP-38), and the error kinds of `escher-group` with their `Display` text are specified in `docs/design/group.md`.
- The coset fill that draws a colouring is specified in Section 5.12 of `docs/design/render.md`, the polygon masks, the image fills, and the decoding of an image with its limits in Sections 5.19 to 5.21 of that document, and the mask polygon and the mask map in its Sections 5.22 and 5.23.
- The rendering, the limits on the render options, the statistics that a rendering counts, the PNG encoding, and the error kinds of `escher-render` with their `Display` text are specified in `docs/design/render.md`.
- The crates, the data flow, the error model, and the owner's decisions are specified in `docs/design/architecture.md`.
- The mathematics is stated in `docs/research/orbifold-notation.md` and, for rendering, in `docs/research/rendering.md`.
- The recipes, the nextest profiles, the hooks, and the dependency policy are specified in `docs/design/tooling.md`.

The external tools with which a user paints over a mask are outside this design, as the owner decided (`DEC-66`).

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
- Schlafli symbol: The symbol $\{p, q\}$ of the regular tiling by $p$-gons with $q$ meeting at each vertex, written `{p,q}` in the information text (Section 2.2 of `docs/research/geometry-models.md`).
- regular tiling line: The line `Regular tiling` of the information text, row 35 of Section 5.1 (Section 5.9).
- domain summary: The lines of the information text that describe the domain, rows 11 to 16 of Section 5.1 (Section 5.5).
- check: One numeric condition that `verify()` of `escher-domain` evaluates, with a residual and a tolerance (Section 2.1 of `docs/design/domain.md`).
- colouring: A colouring of the side presentation of the domain, as `escher-group` finds it: one subgroup of finite index $k$ with its coset table, whose $k$ cosets give the $k$ colours of the tiles (Section 5.12 of `docs/design/render.md`). `escher-group` gives every subgroup a colouring of its own, conjugate subgroups included.
- valid colouring: A colouring in which no two tiles that share a side have the same colour (`DEC-65`), which `escher-group` tests as GRP-42 of `docs/design/group.md` states. This crate calls the search with `ColouringSet::ValidClasses` only, which keeps one valid colouring for each conjugacy class of subgroups (`DEC-37`, GRP-44 of `docs/design/group.md`). In this document, the valid colourings that the crate counts, lists, or resolves by id are those of that set.
- colour count: The number $k$ of colours of a colouring, the index of its subgroup.
- colouring id: The name of a valid colouring, its colour count in decimal followed by lower-case letters, such as `3a`, which GRP-37 of `docs/design/group.md` defines with the order of the ids under `ColouringSet::ValidClasses`.
- colour limit: The largest colour count $N$ that `info` searches, set by `--max-colours` (Section 5.8).
- search limit: A limit of the search of `escher-group` other than the colour limit, which is the step budget or the limit on the colourings that one call stores (GRP-35 and GRP-36 of `docs/design/group.md`).
- colouring lines: The lines of the information text that describe the colourings, rows 17 to 19 of Section 5.1 (Section 5.8).
- parity colouring: The valid colouring with 2 colours of the domain, `2a`, which some domains lack. A domain has at most one (`UNV-36`). Section 4.6 states how `render` draws it.
- all-reversing domain: A domain with at least one side, every side of which is a reversing side. The domain of `1` has no side and is not one.
- default colours: The table of 12 colours of CLI-44, from which a colouring takes its fill colours when `--colours` is absent.
- presentation lines: The lines of the information text that `--presentation` adds, rows 20 to 27 of Section 5.1 (Section 5.7).
- verification lines: The lines of the information text that `--check` adds, rows 28 to 34 of Section 5.1 (Section 5.6).
- Conway presentation: The presentation that `conway_presentation` of `escher-group` reads from the signature (Section 5 of `docs/design/group.md`).
- side presentation: The presentation that `side_presentation()` of `escher-domain` reads from the sides of the domain (Section 5.4 of `docs/design/domain.md`).
- order line: Row 23 or row 27 of Section 5.1, which reports the number of cosets of the trivial subgroup that coset enumeration finds for a presentation (Section 5.7).
- coset limit: `CosetLimit::DEFAULT` of `escher-group`, the largest number of cosets that one enumeration of `info` defines, written $L$.
- reversing side: A side of the domain whose `orientation()` is `Orientation::Reversing`. `render` reads `orientation()` of every side to decide whether a domain is all-reversing (Sections 3.1 and 3.2 of `docs/design/domain.md`).
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
- colour item: One of the items of `--colours` that the commas separate, a colour or an image item (Section 4.6).
- image item: A colour item of the form `image:FILE`, which names a PNG file as an image fill (Section 4.6).
- image fill: The paint of a coset that an image item gives, which Section 5.20 of `docs/design/render.md` defines.
- polygon mask: The PNG image of one colour that `render-masks` writes, which shows the mask polygon in white with a code-coloured border on each edge, as Section 5.19 of `docs/design/render.md` defines.
- mask files: The files that `render-masks` writes, one for each colour, named from the value of `--output` (CLI-58).
- mask polygon: The flat polygon that stands for the domain in every polygon mask, which Section 5.22 of `docs/design/render.md` defines.
- regular construction: A Wythoff construction whose faces are the polygons of a regular tiling, `010` or `100` on `*pq2` with $p \ge q \ge 3$, as Section 5.29 of `docs/design/render.md` defines it.
- arrangement: A subgroup that maps no face of a regular construction, or no face of any type of a construction with several face types, onto itself other than by the identity, and so puts a picture on every face, turned or mirrored, one picture for each of its orbits on the faces of each type, as Sections 5.30 and 5.35 of `docs/design/render.md` define it. `escher-group` finds one for each conjugacy class of subgroups (GRP-45 of `docs/design/group.md`).
- picture: One orbit of the faces of one type under an arrangement, which shows one image that the user paints, named A, B, C, and so on, as Section 5.30 of `docs/design/render.md` defines it. An arrangement of index $2nm$ on faces of $n$ sides has $m$ pictures. With several face types, an arrangement of index $t$ has $t / (2 n_k)$ pictures of the type at a corner of order $n_k$, each named by its picture name.
- construction with several face types: A Wythoff construction with two or three face types, as Section 5.35 of `docs/design/render.md` defines it.
- least index: The least index $L$ of an arrangement of a construction, $2n$ for a regular construction and the least common multiple of the $2 n_k$ with several face types, which `WholeFace::index` gives (Section 5.35 of `docs/design/render.md`).
- picture name: With several face types, the position of the corner of the type, 0, 1, or 2, in the order that `info` prints, followed by the letter of the picture within its type, such as `1B` (Section 5.36 of `docs/design/render.md`).
- arrangement id: The name of an arrangement, in the form of a colouring id, such as `12a`, whose number is the index of the arrangement, twice the number of sides of a face times the number of pictures for a regular construction, and a multiple of the least index with several face types (Section 4.6).
- edge pairing: For each edge of each picture, the picture and the edge of the neighbour that lies on it and whether the neighbour is turned or mirrored, as Section 5.30 of `docs/design/render.md` defines it.
- arrangement lines: The lines of the information text that `--wythoff` adds, rows 36 to 38 of Section 5.1 (Section 5.10).
- face mask: The PNG image that `render-masks` writes for one picture of an arrangement, which shows its flat face polygon in white with a code-coloured border on each edge, as Section 5.32 of `docs/design/render.md` defines it.
- whole-face fill: The fill that `--arrangement` draws, as Section 5.31 of `docs/design/render.md` defines it.

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

CLI-2. `run` MUST write only to the two streams it is given, to the output file of `render`, and to the mask files or the face masks of `render-masks`, and MUST NOT panic on any input or any output error, or else a test of `run` cannot observe all output, and one malformed argument or closed pipe aborts the program instead of giving a status.

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

The library target has the private modules `args`, `info`, `render`, `masks`, and `diagnostic`, as files named `args.rs`, `info.rs`, `render.rs`, `masks.rs`, and `diagnostic.rs` (Section 4.5 of `docs/rust-coding-standards.md`). `masks` runs the steps of Section 4.8 with the names of the mask files, and `render` also reads the image items and resolves the colouring and the arrangement that `render-masks` shares with it, and that `info` shares for its arrangement lines. `args` holds the `clap` types and the parsers of `--size`, `--set`, `--colouring`, and the triples of `--rotation` and `--light`, `info` builds the information text, including the domain summary, the colouring lines, the presentation lines, and the verification lines, `render` runs the steps of Section 4.5 with the parser of colours, the default colours, the resolution of a colouring id, and the statistics lines, and `diagnostic` writes the diagnostics.

## 4. Command Line

### 4.1. Synopsis

```text
escher info [--check] [--presentation] [--colourings] [--max-colours <N>] [--wythoff <MARKS>] <SIGNATURE>
escher render [OPTIONS] --output <FILE> <SIGNATURE>
escher render-masks [OPTIONS] --output <FILE> <SIGNATURE>
escher --help
escher --version
```

`-h` is the short form of `--help` and `-V` of `--version`. `escher <COMMAND> --help` prints the help of that command. `--check`, `--presentation`, `--colourings`, `--max-colours`, and `--wythoff` of `info` have no short form. `-o` is the short form of `--output` of `render` and of `render-masks`, and no other option of either command has a short form. The usage line in the help of `info` is the form that `clap` generates, `escher info [OPTIONS] <SIGNATURE>`, that of `render` is `escher render [OPTIONS] --output <FILE> <SIGNATURE>`, and that of `render-masks` is `escher render-masks [OPTIONS] --output <FILE> <SIGNATURE>`.

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

The help text has these summaries, the long helps of `--set` and `--centre` below the table, and no other text beyond what `clap` generates:

| Item | Summary |
| --- | --- |
| `escher` | The package description, `Command-line program of Escher, a tool for symmetry-based graphics.` |
| `info` | `Print the canonical form, the geometry, and other properties of a signature in orbifold notation` |
| `SIGNATURE` | `The signature in orbifold notation, such as *632. In a POSIX shell, put a signature that contains *, (, or ) in single quotes, as in '2*(12)'` |
| `--check` | `Verify the fundamental domain of the signature and print the result of each check` |
| `--presentation` | `Print the generators and relators of the signature and of its fundamental domain, and for a spherical signature the group order by coset enumeration` |
| `--colourings` | `Print the valid colourings of the fundamental domain, in which no two neighbouring tiles share a colour, one for each conjugacy class of subgroups, with its id, up to the largest number of colours` |
| `--max-colours` | `The largest number of colours of the valid colourings to count and list, and with --wythoff the largest number of pictures of the arrangements to list, counted over all kinds of polygon, from 1 to 64` |
| `--wythoff` of `info` | `List the arrangements of the polygons of a Wythoff construction, as three digits as for render, such as 010 for the heptagons of *732 or 111 for its 14-gons, hexagons, and squares. Each arrangement puts one of its pictures, A, B, and so on, on every polygon, and with several kinds of polygon each picture name starts with the position of the corner of its kind, such as 1B. The number of its id is twice the number of sides of a polygon times the number of pictures for one kind of polygon, and a multiple of the least index for several kinds. For each edge of each picture, it lists the edge of the neighbour that lies on it and whether that neighbour is turned or mirrored, with the edges numbered counterclockwise around each mask from edge 0, the edge with the black tick, and with several pictures each edge written after the name of its picture, such as B2 or 1B2` |
| `render` | `Render the tiling of a signature in orbifold notation and write it to a PNG file` |
| `--output` | `The PNG file to write. An existing file is replaced` |
| `--size` | `The image size in pixels, as WIDTHxHEIGHT` |
| `--colouring` | `One colour for every tile, parity for the colouring with two colours in which no two neighbouring tiles share a colour, or the id of a colouring that the command info --colourings lists, such as 4a. The default is parity when the tiling has it, and one otherwise` |
| `--colours` | `The fill colours, separated by commas, one for each colour of the colouring, each as #rgb, #rrggbb, or image:FILE for a PNG image painted over the mask of that colour that render-masks writes. The default takes the first colours of a built-in list of 12. In a POSIX shell, put the colours in single quotes, as in '#e9c46a,image:fill-1.png'` |
| `--border-width` | `The border width, in the unit of --border-unit. 0 draws no border. The default is 1 for pixels and 0.05 for intrinsic` |
| `--border-unit` | `The unit of --border-width: pixels of the image, or intrinsic for a length in the geometry of the tiling, such as 0.05` |
| `--border-colour` | `The border colour, as #rgb or #rrggbb` |
| `--samples` | `The number of samples along each side of a pixel. The pixel averages N by N samples` |
| `--max-folds` | `The largest number of folds of one sample. A sample that needs more takes the border colour` |
| `--background` | `The colour outside the model of a hyperbolic tiling or the ball of a spherical tiling, and where an image fill is transparent or has no pixel, as #rgb or #rrggbb` |
| `--threads` | `The number of threads, from 1 to 256. The default is the available parallelism, at most 256` |
| `--stats` | `Print statistics of the rendering to standard output` |
| `--set` | `Set a shape parameter of the tiles, as NAME=VALUE, such as aspect=1.5, once for each parameter. --help lists the parameters, and info <SIGNATURE> prints those of a signature` |
| `--centre` | `Centre the picture on corner N of the fundamental domain, counted from 0 in the order of the angles on the line "Corner angles in degrees" that info prints, such as 0 for the corner of order 7 of *732. Without it, the picture is centred on the domain. --help gives an example` |
| `--model` | `The model in which a hyperbolic tiling is drawn: the Poincare disk, the upper half-plane, the band, or the Klein disk` |
| `--wythoff` | `Draw the Wythoff construction of a reflection triangle instead of its tiles, as three digits, one for each corner of the signature in the order that info prints, where 1 turns on the mirror opposite that corner, such as 010 for the regular tiling {7,3} of *732. --colours then gives one colour for each face type, in the order of the corners that have a face` |
| `--arrangement` | `With --wythoff on a regular tiling or a tiling with several kinds of polygon, put a picture on every polygon, turned or mirrored as the arrangement with this id says, such as 12a, which the command info --wythoff lists. --colours then gives one colour or image:FILE for each picture of the arrangement, in the order A, B, and so on, or with several kinds in the order of the picture names, such as 0A, 1A, 1B, each painted over the mask that render-masks writes for that picture` |
| `--rotation` | `The rotation of the ball of a spherical tiling, as three angles in degrees. YAW turns the front of the ball to the right, PITCH turns it up, and ROLL turns the image counterclockwise, in this order` |
| `--light` | `The direction from the ball of a spherical tiling to the light, as X,Y,Z, with X to the right, Y up, and Z toward the viewer. The length is ignored` |
| `--shading` | `Phong shading of the ball of a spherical tiling, or none` |
| `render-masks` | `Write one PNG mask for each colour of a colouring: the tile as a flat polygon in white, over which to paint the image fill of that colour for render, with each edge bordered in the same colour as every edge that it can meet in the tiling` |
| `--output` of `render-masks` | `The file name from which the masks are named: mask.png gives mask-0.png, mask-1.png, and so on, one for each colour. Existing files are replaced` |

`render-masks` shares the summaries of `SIGNATURE`, `--size`, `--colouring`, and `--set` above, and the long help of `--set`. Its help shows `[default: 512x512]` for `--size`. Its `--wythoff` has this summary: `With --arrangement, write one mask of a polygon of a Wythoff construction for each picture, as three digits as for render, such as 010 for the heptagons of *732: for one picture to the --output file itself, for several pictures of one kind of polygon to files named as the masks of a colouring, mask-0.png for picture A, mask-1.png for B, and so on, and with several kinds of polygon to files named by the picture names that info --wythoff prints, such as mask-0A.png, mask-1A.png, and mask-1B.png`. Its `--arrangement` has this summary: `With --wythoff on a regular tiling or a tiling with several kinds of polygon, border each edge of the mask of each picture to show how it meets its neighbour in the arrangement with this id, such as 12a, which the command info --wythoff lists`.

The help of `render` uses the summary of `SIGNATURE` above. It also shows `[default: <value>]` for each option with a fixed default in the table of Section 4.6, which excludes `--colouring` and `--colours`, whose defaults depend on the domain, and `--border-width`, whose default depends on `--border-unit`, and `[possible values: phong, none]` for `--shading`, `[possible values: disk, half-plane, band, klein]` for `--model`, and `[possible values: pixels, intrinsic]` for `--border-unit`, which `clap` generates (Section 11.1). `--colouring` shows no possible values, because its private value parser accepts ids (CLI-42). The help of `info` shows `[default: 12]` for `--max-colours`. The help lists the options in the order in which they are declared, except where an explicit display order places them. `--set`, `--wythoff`, and `--arrangement` are declared in the argument struct that `render` and `render-masks` share (Section 4.8), so explicit display orders place `--colours`, `--wythoff`, and `--arrangement` directly after `--colouring`, and `--set`, `--centre`, `--model`, `--rotation`, `--light`, and `--shading`, in this order, after `--stats`. `--centre` is declared in the argument struct of `render` only. The argument struct of `render` declares `--border-unit` directly after `--border-width`. So the two options of the border width stand together, the options of the fill stand together, the options of the view follow `--set`, and the options of the ball come last in the help. The help of `render-masks`, which has no `--colours`, lists `--wythoff` and `--arrangement` directly after `--colouring` and before `--set`.

`--set` has a long help, the constant `SET_LONG_HELP` in `args`. It lists each shape parameter of `escher-domain` with the signatures that take it: `aspect`, `skew`, `cornerpair<k>.length`, and `slice<i>.length`. It then says that every other signature has no shape parameter, that `escher info <SIGNATURE>` prints one `Parameter` line for each parameter with its range and default (Section 5.1), and gives these example commands:

```sh
escher render o --set aspect=2 --set skew=0.25 -o /tmp/torus.png
escher render '*3333' --set slice1.length=2 -o /tmp/square.png
```

Unit tests in `args` check the parameter names and the signatures of `aspect` and `skew` in the long help against `parameters` of `escher-domain`.

`--centre` has a long help too, the constant `CENTRE_LONG_HELP` in `args`, because the owner asked that the help document clearly how a corner is named (`DEC-131`). It is this text, with a line break at the end of each paragraph and each command, and an empty line between paragraphs:

```text
Centre the picture on corner N of the fundamental domain. Without --centre, the picture is centred on the domain.

The corners are numbered from 0, in the order of the angles that escher info <SIGNATURE> prints on its line "Corner angles in degrees", one number for each angle. For example, escher info '*732' prints

Corner angles in degrees: 25.714286, 60.000000, 90.000000

so 0 is the corner of order 7, whose angle is 180/7 degrees, 1 is the corner of order 3, and 2 is the corner of order 2. With --wythoff 010, which draws the heptagons of {7,3}, --centre 0 puts a heptagon at the centre, and --centre 1 a vertex where three heptagons meet:

escher render '*732' --wythoff 010 --centre 0 -o /tmp/heptagon.png
escher render '*732' --wythoff 010 --centre 1 -o /tmp/vertex.png

Two corners can have the same order, as all three of *333 do, so the number names a corner, and not its order. A domain can have more corners than its signature has orders, such as the four corners of 732, and info prints an angle for each. A number at or above the number of angles is refused, and so is every number for 1, whose domain has no corner. Only the view moves: the tiling, its colours, and its borders stay the same.
```

A unit test in `args` checks that the line of `*732` in the long help equals the line `Corner angles in degrees` that `info` prints for `*732`.

Because `--set` and `--centre` have a long help, `clap` prints the help of `render` and of `render-masks` in two layouts. `--help` puts each option on its own line, its help on the lines below, and the long help of each option that has one. `-h` puts each option on one line, with the short helps of the table above.

CLI-4. `escher --version` and `escher -V` MUST write the package name, one space, the package version, and a newline to standard output, which is `escher-cli 0.1.0` for version 0.1.0, and return `Success`, because Section 5.1 of `docs/design/poc-plan.md` keeps this output, and a script may read it.

### 4.3. The `info` Command

`info` takes one required positional argument, the signature, as a `String` named `SIGNATURE`, the optional flags `--check`, `--presentation`, and `--colourings`, and the optional options `--max-colours <N>` and `--wythoff <MARKS>`, in any order. `--wythoff` takes the value of `--wythoff` of `render`, with the same private value parser (Section 4.6), adds the arrangement lines of Section 5.10, and changes no other line (`DEC-119`). An argument that is not valid UTF-8 is a usage error. Each flag and option may stand before or after the signature. A repeated `--check`, `--presentation`, `--colourings`, `--max-colours`, or `--wythoff` is a usage error, because `clap` rejects a flag or an option that is given more than once, and `info` keeps that behaviour. The argument struct declares `--presentation` after `--check`, then `--colourings`, then `--max-colours`, then `--wythoff`, so they come in that order in the help.

`--max-colours` takes a decimal integer from 1 to `IndexLimit::MAX` of `escher-group`, which is 64 (Section 7.12 of `docs/design/group.md`), with the default 12. A private value parser in `args` reads the value as ASCII decimal digits only, as `--size` does, and passes the number to `IndexLimit::new`, so `clap` holds an `IndexLimit` and the range is checked once, in `escher-group`. Every other value, such as `0`, `65`, `x`, `+5`, or ` 5`, is a usage error (Section 6.5) with one message, whose upper end the parser formats from `IndexLimit::MAX`:

```text
error: invalid value '65' for '--max-colours <N>': expected a decimal integer from 1 to 64
```

The default is the literal `12` in the argument struct, because `IndexLimit` has no `Display` text, and a unit test asserts that it parses to `IndexLimit::DEFAULT`. `--colourings` changes only the colouring lines (Section 5.8). The colour limit changes the colouring lines and, with `--wythoff`, the largest number of pictures that the arrangement lines list (Section 5.10, `DEC-126`). Neither has an effect on a signature without a domain.

CLI-5. `info` and `render` MUST pass their argument unchanged to the `FromStr` implementation of `Signature` from `escher-signature`, and MUST NOT trim, normalize, or check it first, because a second reading of the grammar diverges from the one that `docs/design/signature.md` specifies, and the byte spans of the error then no longer match the argument.

On success, `info` writes the information text of Section 5 to standard output. On a signature error, it writes the diagnostic of Section 6 to standard error and nothing to standard output, with or without `--check`. A bad orbifold is a signature error (`DEC-24`).

`info` takes no setting. It builds the domain with `FundamentalDomain::build`, which gives each shape parameter its default, so the domain summary and the verification lines describe the default shape.

When a domain is built, `info` always reads its side presentation and searches its valid colourings up to the colour limit, and adds the colouring lines of Section 5.8 after the domain summary. With `--colourings`, those lines also name each colouring that they count.

With `--presentation`, `info` also reads the Conway presentation of the signature and the side presentation of its domain, enumerates the cosets of each for a spherical signature within the coset limit, adds the presentation lines of Section 5.7 to the information text, and writes the diagnostic of Section 6.9 when an enumeration finds an order other than the group order.

With `--check`, `info` also runs `verify()` on the domain, adds the verification lines of Section 5.6 to the information text, and writes the diagnostic of Section 6.7 when a check fails or the signature has no domain. With both flags, the presentation lines come before the verification lines. The colouring lines come before both.

The `clap` argument struct holds each flag as a `bool`, because the derive API sets a flag through a `bool` field. `info` converts each at once into a private enum with two variants, which the private functions take (TYPE-2 of `docs/rust-coding-standards.md`).

A signature never starts with `-`, so it never collides with an option. An argument that starts with `-` is read as an option, and an unknown option is a usage error.

### 4.4. Shell Quoting

In a POSIX shell, `*`, `(`, and `)` are special characters, so a signature that contains any of them is quoted in single quotes, as in `escher info '2*(12)'`. The shell passes every character inside single quotes unchanged. The help text of `info`, `render`, and `render-masks` says this, and every example in this document that needs it quotes the signature.

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

This command writes an image of `*642` in its colouring `4a`, which has 4 colours, with four colours of its own:

```sh
escher render '*642' --colouring 4a --colours '#dc143c,#ffd700,#000080,#2e8b57' -o four.png
```

This command writes an image of `*732` in the band model in place of the Poincare disk:

```sh
escher render '*732' --model band -o band.png
```

This command writes an image of `*732` whose borders are 0.05 wide in the metric of the hyperbolic plane, in place of 1 pixel:

```sh
escher render '*732' --border-unit intrinsic --border-width 0.05 -o intrinsic.png
```

This command writes the regular tiling `{7,3}` by heptagons, three at each vertex, as the Wythoff construction of `*732` with only the mirror opposite the corner of order 3 turned on, in one colour:

```sh
escher render '*732' --wythoff 010 -o heptagons.png
```

This command draws the same heptagons with a heptagon at the centre of the image, because position 0 of `*732` is its corner of order 7, where the heptagons are centred (Section 4.6):

```sh
escher render '*732' --wythoff 010 --centre 0 -o heptagon.png
```

These commands list the arrangements of the hexagons of `*632`, write the face mask of the arrangement `12a` to `/tmp/masks/hexagon.png`, and, after the user has painted a picture over that mask and saved it as `/tmp/masks/hexagon-picture.png`, put the picture on every hexagon as `12a` says:

```sh
escher info '*632' --wythoff 010
escher render-masks '*632' --wythoff 010 --arrangement 12a -o /tmp/masks/hexagon.png
escher render '*632' --wythoff 010 --arrangement 12a --colours 'image:/tmp/masks/hexagon-picture.png' -o hexagons.png
```

These commands do the same for the triangles of `*732`, seven at each vertex, which need three pictures. The second writes the face masks of pictures A, B, and C of the arrangement `18a` to `/tmp/masks/triangle-0.png`, `/tmp/masks/triangle-1.png`, and `/tmp/masks/triangle-2.png`, and the third takes the three pictures that the user painted over them, in the order A, B, C. The id `18a` stands for the first arrangement with three pictures, which the snapshot of the listing fixes:

```sh
escher info '*732' --wythoff 100
escher render-masks '*732' --wythoff 100 --arrangement 18a -o /tmp/masks/triangle.png
escher render '*732' --wythoff 100 --arrangement 18a --colours 'image:/tmp/masks/a.png,image:/tmp/masks/b.png,image:/tmp/masks/c.png' -o triangles.png
```

These commands do the same for the dodecagons, hexagons, and squares of `*632` with `111`, whose least index is 12, so that an arrangement of index 12 has the pictures `0A` of the dodecagons, `1A` and `1B` of the hexagons, and `2A`, `2B`, and `2C` of the squares. The id `12a` stands for the first arrangement of index 12, which the snapshot of the listing fixes. The second command writes the six face masks to `/tmp/masks/tile-0A.png`, `tile-1A.png`, `tile-1B.png`, `tile-2A.png`, `tile-2B.png`, and `tile-2C.png`, one for each picture name (CLI-76), and the third takes six items in the same order:

```sh
escher info '*632' --wythoff 111
escher render-masks '*632' --wythoff 111 --arrangement 12a -o /tmp/masks/tile.png
escher render '*632' --wythoff 111 --arrangement 12a --colours 'image:/tmp/masks/0a.png,image:/tmp/masks/1a.png,image:/tmp/masks/1b.png,image:/tmp/masks/2a.png,image:/tmp/masks/2b.png,image:/tmp/masks/2c.png' -o tiles.png
```

Several Euclidean signatures share a picture, because their tiles and borders coincide (Section 5.14 of `docs/design/render.md`, `DEC-55`). With `--colouring one`, the six signatures `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` give the image of `*2222`, `4*2` that of `442`, and `3*3` that of `632`. These signatures differ in their colourings, so a user tells them apart with `--colouring` and an id that `escher info --colourings` lists.

`render` runs these steps in this order. It stops at the first step that fails and writes the diagnostic of Section 6.8, except at step 1, whose diagnostic is that of Section 6.1:

1. Parse the signature with the `FromStr` implementation of `Signature` (CLI-5).
2. Read the border colour and the background colour (Section 4.6).
3. Put the settings into a `ShapeParameters` of `escher-domain` in the order given, and stop at the first setting whose name an earlier setting already gave. Build the domain with `FundamentalDomain::build_with` and those settings, which checks each name and value. When `build_with` returns `UnknownParameter` or `ParameterOutOfRange`, find the first rejected setting with `parameters` and `ParameterSpec` of `escher-domain`, as CLI-29 states, and stop with the diagnostic of Section 6.8 that names it.
4. Choose the colouring: the value of `--colouring`, or the default of CLI-34. Then read the fill colours for that colouring, or take the default colours (Section 4.6). The number of colours comes from the colouring alone, so this step runs no search, also for a colouring id, except for the default colouring of a domain that is not all-reversing, which needs the search of CLI-34 to know whether it has 1 or 2 colours. Reading the fill colours includes opening and decoding the file of each image item, in order (CLI-60). With `--wythoff`, this step instead stops when `--colouring` is given, then builds the construction with `Wythoff::new` of `escher-render` from the domain and the marks, and without `--arrangement` then reads one colour for each of its face types, with no image item, or takes the default colours, with no search (CLI-62, CLI-63). The construction comes before the colours, because the number of colours is the number of its face types. So `--wythoff 000` with `--colouring` reports the refusal of `--colouring`. With `--arrangement`, this step stops when `--wythoff` is absent, builds the construction and then `WholeFace::new` of `escher-render` from the domain and the construction, which refuses a construction with one face type that is not a regular tiling, and reads exactly one item of `--colours` for each picture of the id, each a colour or an image item that it opens and decodes, or takes the first default colours, one for each picture, with no search (CLI-65, CLI-67). On a regular construction, it refuses an id whose number is from 65 to 84 directly after `WholeFace::new`, before it reads any item (CLI-75). The id has $m$ pictures when its number is $2nm$ for the number of sides $n$ of a face and an integer $m \ge 1$, and one picture otherwise, as in `M24` (CLI-66). With several face types, the id has $t / (2 n_k)$ pictures of each type when its number $t$ is a multiple of the least index $L$, and the pictures of the index $L$ otherwise (CLI-75). Only the whole face gives $n$, or the face types and $L$, so this step counts the items after `WholeFace::new`.
5. Check the render options: build the render options of `escher-render` from the size, the sample grid, a placeholder fill of one colour, the border colour, the border width, the border unit, the background colour, the fold cap, the viewing rotation in radians, the light direction, the shading, the hyperbolic model, and the position of `--centre` when it is given. Then build a renderer from the domain and these options, and drop both. `escher-render` checks every value, for every geometry, and the renderer checks the position against the corners of the domain (CLI-72).
6. Build the fill: without an image item, `Fill::One` for `one` and for the id `1a`, and `Fill::Parity` for `parity` on an all-reversing domain, with the colours of step 4. For `parity` on any other domain, for any other colouring id, and for every colouring when an item of `--colours` is an image item (CLI-61), resolve the colouring as CLI-43 and CLI-61 state, `parity` as the id `2a`, then build the coset fill of `escher-render` from the domain, the colouring, and the paints, `Paint::Solid` for a colour and `Paint::Image` for an image item. With `--wythoff`, build the Wythoff fill of `escher-render` from the construction of step 4 and the colours, and resolve no colouring. With `--arrangement`, resolve the arrangement as CLI-66 states, and build the whole-face fill of `escher-render` from the whole face of step 4, the arrangement, and one paint for each picture, in the order of the items, `Paint::Solid` for a colour and `Paint::Image` for an image item. Then build the render options again with this fill and the same values, which gives no error once step 5 has passed.
7. Build a thread pool with the number of threads of `--threads`, or with the default of Section 4.6, which this step computes.
8. Build the renderer, then render the image on the thread pool, in bands of one row, each with the band operation of `escher-render`, and merge the statistics of the bands. The time of the rendering is the time of the statistics lines.
9. Encode the image as PNG with `escher-render`.
10. Write the PNG bytes to the output file (Section 4.7).
11. With `--stats`, write the statistics lines to standard output (Section 4.7).

Section 3 of `docs/design/render.md` names the operations of steps 5, 6, 8, and 9 and the statistics that step 8 merges.

The colouring depends on the domain, so step 4 comes after the domain is built in step 3. Steps 4 and 5 check every input that needs no search before step 6 resolves a colouring id. Such an input error is certain, while a search can take seconds and can stop at a limit, and the diagnostic of that limit claims nothing about the input. So a command with a colouring id and a bad colour, a wrong number of colours, an image item that cannot be read, more than 12 colours without `--colours`, or a bad option reports that error at once. It never waits for the search, and the diagnostic of a search that stopped at a limit never takes its place. The one exception is a command without `--colouring` on a domain that is not all-reversing, whose number of colours is known only after the search of CLI-34. A command with a bad value of `--colours` and a bad setting reports the setting, and a command with a bad `--border-colour` and a bad setting reports the colour.

CLI-35. When the colouring is `parity` and the domain has no valid colouring with 2 colours, `render` MUST stop at step 6 with the diagnostic of Section 6.8 and write no file, because an image in one colour would hide that the requested colouring was not drawn (`DEC-32`). `parity` exists on every all-reversing domain. On any other domain only the search of 2 colours tells whether it exists, so the stop comes at step 6 with that search, and a search that stops at a limit gives the diagnostic of that limit and claims nothing about the colouring (CLI-43).

To resolve a colouring id with the colour count $k$, step 6 calls `side_presentation()` on the domain. A render with a colouring id builds the side presentation three times: here, in `CosetFill::new`, and in `Renderer::new`, whose two calls Section 5.12 of `docs/design/render.md` states. Step 6 then calls `colourings_of_index` of `escher-group` with `ColouringSet::ValidClasses`, $k$ as its `IndexLimit`, and the search budget, which searches only the colourings with exactly $k$ colours (GRP-35 of `docs/design/group.md`), and calls `find` with the id on the resulting `ColouringsOfIndex`. A private function takes the budget as a parameter. `render` passes `SearchBudget::DEFAULT` to it, so that a unit test can resolve an id with a small budget (Section 8.1). The outcomes are:

| Outcome | Result |
| --- | --- |
| `find` returns the colouring | Step 6 goes on with it |
| `is_complete()` is true, and `find` returns none | Stop, with the diagnostic of Section 6.8 that the colouring does not exist, which gives `list().len()`, the number of valid colourings with $k$ colours |
| `is_complete()` is false, because the search stopped at a limit | Stop, with the diagnostic of Section 6.8 that the search stopped at a limit, which claims nothing about the existence of the id |
| `side_presentation()` or `colourings_of_index` returns an error | Stop, with `error: cannot render <signature>` and a `caused by` line with the `Display` text of the error |

This resolution is the proposed default `DEC-44` of `docs/design/architecture.md`.

CLI-43. `render` MUST resolve a colouring id other than `1a`, and `parity` on a domain that is not all-reversing, through the side presentation of the domain that it renders and `colourings_of_index` of its colour count with `ColouringSet::ValidClasses`, MUST run no search of another colour count, MUST stop at step 6 with a diagnostic and write no file when the colouring does not exist or the search reaches a search limit, and MUST NOT claim that an id does not exist when the search stopped at a limit, because a search of the lower colour counts costs time that the image does not need and can reach a limit before it reaches the id, the colours of the image come from the coset table of the domain that is drawn, a statement that an id does not exist is true only of a search that finished, and the ids that `info` lists are those of `ColouringSet::ValidClasses` (`DEC-37`, `DEC-71`).

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
| `--colouring <COLOURING>` | `one`, `parity`, or a colouring id (CLI-42) | `parity` when the domain has a valid colouring with 2 colours, and `one` otherwise (CLI-34, `DEC-72`) |
| `--colours <COLOURS>` | Colour items separated by commas, each a colour or an image item `image:FILE`, as many as the colouring needs: one for `one`, two for `parity`, and $k$ for a colouring id with the colour count $k$ (`DEC-29`, `DEC-91`). With `--wythoff`, one colour for each face type and no image item (CLI-63). With `--arrangement`, exactly one item for each picture, each a colour or an image item, in the order of the pictures (CLI-67) | The first default colours that the colouring needs (CLI-44): `#e9c46a` for `one`, `#e9c46a,#2a9d8f` for `parity` (`DEC-7`), and the first $k$ default colours for an id with $k$ at most 12. With `--wythoff`, the first default colours, one for each face type. With `--arrangement`, the first default colours, one for each picture, so `#e9c46a` for one picture |
| `--wythoff <MARKS>` | Exactly three characters, each `0` or `1`, one for each corner of the canonical signature, in its order (CLI-62) | None. Without it, `render` draws the tiles of the domain |
| `--arrangement <ID>` | An arrangement id, which `ColouringId` of `escher-group` reads (CLI-65). Only with `--wythoff` | None. Without it, `--wythoff` draws the faces in one colour for each face type |
| `--border-width <WIDTH>` | A number in a form that the `FromStr` implementation of `f64` reads, such as `1.5`, in the unit of `--border-unit`. The value `0` draws no border | `1` with `pixels` and `0.05` with `intrinsic` (`DEC-7`, `DEC-79`) |
| `--border-unit <UNIT>` | `pixels` or `intrinsic` | `pixels` |
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
| `--model <MODEL>` | `disk`, `half-plane`, `band`, or `klein` | `disk` |
| `--centre <N>` | A decimal integer that fits a `u32`: the position of a corner of the domain, counted from 0 (CLI-72) | None. Without it, the picture is centred on the domain |

`clap` parses `--samples` and `--max-folds` with its parser of `u32`, `--threads` with its parser of a range of `u16` from 1 to 256, whose value converts to `usize` without a fallible conversion, and `--border-width` with its parser of `f64`. A private parser in `args` reads `--size`. For a value of another form it returns the message `expected two decimal integers joined by x, such as 512x512`, which `clap` puts into its usage error. A second private parser in `args` reads each value of `--set`: it splits the value at the first `=`, requires a name that is not empty, and reads the rest with the `FromStr` implementation of `f64`, as `--border-width` is read. For a value of another form, such as `aspect`, `=2`, or `aspect=x`, it returns the message `expected a parameter name and a number joined by =, such as aspect=1.5`. `clap` collects the values of `--set` in the order given, with its action `Append`. A NaN or infinite number reads as a number, and `escher-domain` rejects it as outside the range of its parameter. The value of `--output` is a `std::path::PathBuf`, so a path that is not valid UTF-8 is accepted.

A triple is exactly three items separated by commas, each read by the `FromStr` implementation of `f64`, with nothing trimmed. One private parser in `args` reads the triples of `--rotation` and `--light`. For a value of another form, such as `1,2`, `a,b,c`, `1,2,3,4`, or `1,,2`, it returns the message `expected three numbers separated by commas, such as 20,35,0`. A NaN or infinite number reads as a number, and `escher-render` rejects it. `render` converts each angle of `--rotation` from degrees to radians with `f64::to_radians`. `--shading` is a value enum of `clap`, and the value `none` gives `Shading::Flat` of `escher-render`.

`--rotation` and `--light` set the `clap` setting `allow_hyphen_values`, so that a value that starts with `-` is read as the value, as in `--light -1,1,2` and `--rotation -20,35,0`. The form with `=`, as in `--light=-1,1,2`, works with or without the setting. The cost is that an option name directly after either option is read as its value: `--rotation --stats` reads `--stats` as the triple, which is a usage error, and `--rotation -h` reads `-h` as the triple instead of printing the help.

CLI-31. `--rotation` and `--light` MUST accept a value that starts with `-`, because the default light direction and many rotations have a negative number first, and without the setting `clap` rejects `--light -1,1,2` as an unknown argument.

`render` reads the three options for every signature. For a hyperbolic or Euclidean signature `escher-render` checks them and they have no effect, as `--background` has no effect on a Euclidean tiling.

`--model` is a value enum of `clap` with the values `disk`, `half-plane`, `band`, and `klein`, which give the variants `Disk`, `HalfPlane`, `Band`, and `Klein` of `HyperbolicModel` of `escher-render`. A method of the private enum in `args` that holds the value does this mapping, so that the unit test in `args` can call it, whereas `render` maps `--shading` itself. Any other value, such as `poincare` or `Band`, is a usage error, whose message `clap` writes with the possible values. `render` passes the model for every signature, and for a Euclidean or spherical signature it has no effect, as the options of the ball have none on a hyperbolic tiling (`DEC-77`). So `escher render '*632' --model band -o tiling.png` writes the image of `*632` in the Euclidean plane, with no diagnostic.

CLI-55. `render` MUST accept as the value of `--model` exactly `disk`, `half-plane`, `band`, and `klein`, MUST take `disk` when the option is absent, and MUST pass the variant of the same name to `escher-render` for every signature without a check of the geometry, because the default keeps every image of a hyperbolic signature as it was, and `escher-render` alone decides which domains read the model (RND-23 and RND-52 of `docs/design/render.md`).

`--border-unit` is a value enum of `clap` with the values `pixels` and `intrinsic`, which give the variants `Pixels` and `Intrinsic` of `BorderUnit` of `escher-render`, through a method of the private enum in `args`, as for `--model`. Any other value, such as `px` or `Intrinsic`, is a usage error, whose message `clap` writes with the possible values. With `pixels`, `--border-width` is a width in output pixels. With `intrinsic`, it is the full width of the band around each wall in the metric of the geometry of the signature: the metric of curvature $-1$ for a hyperbolic signature, the normalization length of the domain for a Euclidean signature (`DEC-17`), and the radian of the unit sphere for a spherical signature (Section 5.17 of `docs/design/render.md`). The unit changes the default of the width (`DEC-79`) and not its accepted values (`DEC-78`), so `escher render '*732' --border-unit intrinsic -o tiling.png` draws borders 0.05 wide in the hyperbolic metric. The `clap` argument struct holds `--border-width` as an `Option`, with no default of `clap`.

CLI-56. `render` MUST accept as the value of `--border-unit` exactly `pixels` and `intrinsic`, MUST take `pixels` when the option is absent, MUST take the width 1 with `pixels` and 0.05 with `intrinsic` when `--border-width` is absent, and MUST pass the variant of the same name to `escher-render` for every signature, because the default keeps every image as it was (RND-55 of `docs/design/render.md`), a width of 1 in the intrinsic unit can cover whole tiles (`DEC-79`), and the unit applies to every geometry.

`--centre` puts a corner of the domain at the centre of the picture, as a change of the view only, and without it the picture is centred on the domain (`DEC-130`). `clap` reads its value with its parser of `u32`, as it reads `--samples`, so any other value, such as `-1`, `x`, `1.5`, or `4294967296`, is a usage error. The value names a corner by its position and not by its order, because two corners can have the same order (`DEC-131`). The position counts from 0, as the sides in the lines `Mirror sides` and `Side pairs` and the edges of a face mask do, in the order of the angles of the line `Corner angles in degrees` of `info` (row 14 of Section 5.1), one position for each angle, a flat angle of 180 degrees included. That order is the order of `vertices()` of the domain (Section 5.34 of `docs/design/render.md`). The domain of `1` has no corner, so `info` prints no such line for it, and `render` refuses every position (`DEC-134`). These are the positions of some signatures:

| Signature | Line `Corner angles in degrees` of `info` | Positions |
| --- | --- | --- |
| `*732` | `25.714286, 60.000000, 90.000000` | 0 the corner of order 7, 1 the corner of order 3, and 2 the corner of order 2 |
| `*632` | `30.000000, 60.000000, 90.000000` | 0 the corner of order 6, 1 of order 3, and 2 of order 2 |
| `*333` | `60.000000, 60.000000, 60.000000` | 0, 1, and 2, the three corners of the triangle, each of order 3 |
| `732` | `51.428571, 90.000000, 120.000000, 90.000000` | 0 the centre of order 7, 2 the centre of order 3, and 1 and 3 the two vertices of the kite that are centres of order 2 (Section 6.8 of `docs/design/domain.md`) |
| `*532` | `36.000000, 60.000000, 90.000000` | 0 the corner of order 5, 1 of order 3, and 2 of order 2 |
| `*` | `180.000000, 180.000000` | 0 and 1, the two flat vertices of the hemisphere on its mirror |
| `1` | None | None. Every position is refused |

`--centre` combines with every other option of `render` and changes only the view (`DEC-136`). With `--model`, the corner shows where the model shows the centre of the disk, at the centre of the image. With `--rotation`, the centring comes first, so the angles turn the centred ball, and with the angles 0 the corner faces the viewer. The colouring, the colours and image fills, `--wythoff`, `--arrangement`, the border width in both units, the border colour, the background, the sample grid, and the fold cap apply unchanged. The face masks of `render-masks` do not follow `--centre` (`DEC-132`), so `render-masks` has no such option.

CLI-72. `render` MUST accept as the value of `--centre` exactly the text that the parser of `u32` of `clap` accepts, MUST pass the value unchanged to `escher-render` as the position of the centre corner of the render options, MUST NOT compare it with the corners of the domain itself, and MUST build a renderer at step 5 with the placeholder fill, so that `escher-render` refuses a position that the domain does not have before the searches of step 6, because `escher-render` alone reads the vertices of the domain (RND-80 of `docs/design/render.md`), a second check in this crate could differ from it, and a certain input error never waits for a search (Section 4.5).

CLI-73. `render-masks` and `info` MUST NOT accept `--centre`, because the owner decided that the face masks do not follow it (`DEC-132`), a polygon mask has no view, and `info` draws nothing.

A colour is `#` followed by exactly three or exactly six hexadecimal digits. The digits are ASCII `0` to `9` and `a` to `f` in either case. In the form `#rrggbb`, each pair of digits is one 8-bit sRGB channel, in the order red, green, and blue. In the form `#rgb`, each digit stands for the channel whose two digits both equal it, so `#e9c` is `#ee99cc`. Nothing is trimmed, so a colour with a space before or after it is not a colour. Every colour is opaque, and `render` has no transparent background (`DEC-7`). The colours are read by a private parser in the module `render`, with no dependency (`DEC-29`).

Step 2 reads `--border-colour` and then `--background`. Step 4 splits the value of `--colours` at each comma. It first compares the number of items with the number that the colouring needs, and then reads each item, in order: an item that starts with `image:` is an image item, and every other item is read as a colour. An empty item, such as the one that a trailing comma gives, is not a colour. When every item has been read, step 4 opens and decodes the file of each image item, in order. With `--wythoff` and without `--arrangement`, step 4 reads every item as a colour, so an image item, like any other item that is not a colour, gives the diagnostic of Section 6.8 for an item that is not a colour, with no image form in its text (CLI-63).

The file of an image item is the text after `image:`, which is not empty. `render` uses it as a path as given, relative to the working directory, with no expansion of `~`, of a variable, or of a pattern, and opens it with `std::fs::File::open`, which follows a symbolic link. A path cannot hold a comma, because the comma separates the items, and it is UTF-8, because `clap` reads `--colours` as a `String`. `render` passes the open file, through a `std::io::BufReader`, to `Image::decode_png` of `escher-render`, which checks each image against the size limits of an output image (Section 5.21 of `docs/design/render.md`). This crate adds no limit of its own on the images (`DEC-92`). The same path given twice is read twice.

CLI-60. `render` MUST open the file of each image item at the path as given, with no expansion, MUST decode it only through `Image::decode_png`, MUST stop at step 4 with the diagnostic of Section 6.8 and write no file when an image file cannot be opened or decoded, and MUST name the image only by the position of its item, because an expansion of the path would read a file that the user did not name, a second check in this crate could differ from the limits of `escher-render`, and the path is untrusted text that a diagnostic must not repeat (CLI-26).

An image fill exists only as a paint of a coset fill (Section 5.20 of `docs/design/render.md`), so an image item makes `render` draw every colouring with the coset fill.

CLI-61. When an item of `--colours` is an image item, `render` MUST draw the colouring with the coset fill of `escher-render`, MUST resolve `one` and `1a` as the first colouring of `colourings_of_index` with `ColouringSet::All` and the colour count 1, which is `1a`, the one colouring of that count (Section 7.15 of `docs/design/group.md`), and `parity` as the first colouring of the search of CLI-34 with 2 colours, which is `2a`, also on an all-reversing domain, because `Fill::One` and `Fill::Parity` take no image (`DEC-91`). With solid paints only, the coset fill of `1a` MUST give the bytes of `one`, and the coset fill of `2a` the bytes of `parity`, with the same colours, so that an image item changes no tile of another colour. The tests of Section 8.1 pin both equalities.

With an image item, `parity` reuses the colouring that step 4 found when it searched for the default colouring of a domain that is not all-reversing. In every other case, including the default colouring of an all-reversing domain, which step 4 chose without a search, step 6 runs that search. Its diagnostics name `parity`, as without an image item (Section 6.8).

The `clap` argument struct holds `--colouring` as an `Option`, with no default of `clap`, and step 4 chooses the colouring when the option is absent.

A private value parser in `args` reads `--colouring` into a private enum with the variants one, parity, and an id. It accepts exactly `one` and `parity`, and otherwise reads the value with the `FromStr` implementation of `ColouringId` of `escher-group`, which accepts a colour count from 1 to `IndexLimit::MAX` in decimal with no leading zero, followed by 1 to 4 lower-case letters (GRP-37). No id equals `one` or `parity`, because an id starts with a digit. For any other value, such as `three`, `3`, `03a`, `3A`, or `65a`, the parser returns the message `expected one, parity, or a colouring id such as 4a`, which `clap` puts into its usage error.

CLI-42. `render` MUST accept as the value of `--colouring` exactly `one`, `parity`, and the text that the `FromStr` implementation of `ColouringId` accepts, MUST read an id through that implementation only, and MUST give a usage error with the message above for every other value, because a second reading of the id grammar in this crate could accept an id that `escher-group` does not, or the reverse, and an id that does not parse names no colouring of any signature.

A colouring id that parses can still name no colouring of the domain, such as `3a` of `*642`, which has no valid colouring with 3 colours. That is a render error of step 6 with the status `Failure` (CLI-43), and not a usage error, because whether it exists depends on the signature.

The colouring with one colour is valid only for the domain of `1`, so the search lists it for no other domain, and the owner keeps it available as an exception (`DEC-65`). Without an image item, `render` therefore reads both `one` and `1a` as `Fill::One`, for every domain, with no search.

The alias `parity` names the parity colouring, which has the id `2a` under both `ColouringSet::Valid` and `ColouringSet::ValidClasses` (`UNV-36`). Without an image item, `parity` on an all-reversing domain keeps `Fill::Parity`, which colours by the parity of the reversals, so the default colouring of such a domain runs no search. Section 5.12 of `docs/design/render.md` states why that fill is valid there, so it is `2a`. On any other domain, `parity` is the coset fill of `2a` when the search finds it.

CLI-48. Without an image item, `render` MUST draw `one` and `1a` with `Fill::One`, `parity` on an all-reversing domain with `Fill::Parity`, and `parity` on any other domain with the coset fill of the id `2a`, and with the same colours `--colouring 2a` MUST give the bytes of `--colouring parity` on every domain, because a user who copies an id from the lines of `info` expects the picture of its alias, and `info` names the aliases in its `Colouring` lines (Section 5.8). That `Fill::Parity` colours by the cosets of the orientation-preserving subgroup is `UNV-18`, and that `2a` is the only valid colouring with 2 colours is `UNV-36`.

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

CLI-44. Without `--colours`, `render` MUST take the first $k$ entries of the table of default colours for a colouring with $k$ colours, and for a colouring with more than 12 colours MUST stop at step 4 with the diagnostic of Section 6.8 that asks for $k$ colours with `--colours`, and write no file, because the first two entries keep the images of `one` and of `parity` on an all-reversing domain as they are, and a table that cycles or a generated colour would give two cosets colours that a viewer cannot tell apart, while a prototype outside the repository found no 64 colours that are distinct in practice.

With `--colours`, `render` takes exactly as many colour items as the colouring has, for every colour count, in the hexadecimal forms of `DEC-29` or as image items, mixed in any order. A wrong number gives the diagnostic of the wrong number of colours with the id of the colouring, such as `the colouring 4a needs 4 colours, and 2 were given`.

CLI-34. Without `--colouring`, `render` MUST use the colouring `parity` on an all-reversing domain, with no search. On any other domain, it MUST call `colourings_of_index` with `ColouringSet::ValidClasses`, the colour count 2, and `SearchBudget::DEFAULT` at step 4, and MUST use `parity` when the list holds a colouring, `one` when the search is complete and the list is empty, and MUST stop with the diagnostic of Section 6.8 of a search that stopped at a limit, naming `parity`, and write no file when the search is not complete. The owner decided that the default is the valid colouring with 2 colours when the domain has one, and one colour otherwise (`DEC-72`), and an image in one colour after a stopped search would claim that the domain has no such colouring.

So `escher render '*732' -o tiling.png` draws two colours with no search. A domain whose default is `one`, such as that of `1`, which has no colouring with 2 colours, draws every tile in `#e9c46a`, and `escher render 1 --colours '#e9c46a,#2a9d8f' -o one.png` fails at step 4, because the default colouring of `1` is `one`, which needs 1 colour.

`--wythoff` draws the faces of a Wythoff construction on a reflection triangle (Sections 5.24 to 5.27 of `docs/design/render.md`). Its value has one digit for each corner of the signature, in the order of the canonical form that `escher info` prints, which writes the largest order first, so the digits of `*732` stand for the corners of orders 7, 3, and 2. A digit `1` turns on the mirror opposite its corner, and `0` leaves it off (`DEC-99`). The faces are centred at the corners. A corner whose two mirrors are both off has no face, a corner with one mirror on has a face with as many sides as its order, and a corner with both mirrors on has a face with twice as many, except that a corner of order 2 with one mirror on has no face. Each corner with a face is one face type, and `--colours` gives one colour for each face type, in the order of the corners that have a face (`DEC-105`).

For a signature `*pq2` with $p \ge q \ge 3$, for which `escher info` prints `Regular tiling: {p,q}, dual {q,p}`, `010` draws $\{p, q\}$ and `100` its dual $\{q, p\}$. These are the Wythoff symbols `q | p 2` and `p | q 2`, which [CLM1954] gives for the regular tilings of the sphere and of the Euclidean plane (Section 10.3 of `docs/research/colourings.md`). On a hyperbolic triangle the correspondence rests on `UNV-37` and `UNV-38`.

The seven markings of `*732` give these tilings. The faces are listed in the order of the corners, which is the order of `--colours`. The Wythoff symbol of each row is the general symbol of [CLM1954] with $p = 7$ and $q = 3$, a convention stated for the sphere and the Euclidean plane. The vertex configurations come from the infoboxes of Wikipedia, a secondary source, for the members of `(7 3 2)` (Section 10.3 of `docs/research/colourings.md`), and for every row the faces of a hyperbolic triangle rest on `UNV-38`:

| Marks | Active mirrors, opposite the corners of order | Faces, by corner | Vertex configuration | Wythoff symbol | Evidence |
| --- | --- | --- | --- | --- | --- |
| `100` | 7 | Triangles at the corner of order 3 | $3^7$, the regular tiling $\{3,7\}$ | `7 \| 3 2` | The reference records no infobox for $\{3,7\}$. The tiling is the dual of $\{7,3\}$ by [CBGS2008], p. 273, and the symbol is that of [CLM1954], p. 406 |
| `010` | 3 | Heptagons at the corner of order 7 | `7^3`, the regular tiling $\{7,3\}$ | `3 \| 7 2` | Wikipedia, and the symbol of [CLM1954], p. 406 |
| `001` | 2 | Heptagons and triangles | `(3.7)^2` | `2 \| 7 3` | Wikipedia, and the symbol of [CLM1954], p. 406 |
| `110` | 7 and 3 | Heptagons, triangles, and squares | `3.4.7.4` | `7 3 \| 2` | Wikipedia for the configuration. Its infobox gives the symbol `3 \| 7 2`, which the reference reads as wrong against [CLM1954], p. 407 (Section 13 of `docs/research/colourings.md`) |
| `101` | 7 and 2 | Heptagons and hexagons | `7.6.6` | `2 7 \| 3` | Wikipedia, and the symbol of [CLM1954], p. 407 |
| `011` | 3 and 2 | 14-gons and triangles | `3.14.14` | `2 3 \| 7` | Wikipedia, and the symbol of [CLM1954], p. 407 |
| `111` | 7, 3, and 2 | 14-gons, hexagons, and squares | `4.6.14` | `2 7 3 \|` | Wikipedia, and the symbol of [CLM1954], p. 407 |

`000` turns every mirror off, so the generator point would lie on all three mirrors and no corner has a face, and `render` stops with the diagnostic of Section 6.8 at step 4. On `*n22`, `100` also gives no face, because both corners of order 2 drop out, and so does every marking with one `1` on `*222`.

A private value parser in `args` reads `--wythoff` into an array of three `bool` values, `true` for `1`. For a value of another form, such as `01`, `0101`, `0a1`, or `0,1,0`, it returns the message `expected three digits 0 or 1, such as 010`, which `clap` puts into its usage error. `000` has the form, so it is a render error with the status `Failure`, and not a usage error. `--wythoff` refuses a domain that is not a reflection triangle at step 4, such as the domain of `732`, `*2222`, `*3333`, or `*22`, with the error of `escher-render` (`DEC-107`). A Wythoff construction has no colouring, so `--wythoff` and `--colouring` cannot be combined (`DEC-105`).

CLI-62. `render` MUST accept as the value of `--wythoff` exactly three characters, each `0` or `1`, MUST pass the character of position $k$, counted from 0, to `escher-render` as mark $k$, `true` for `1`, and MUST stop at step 4 with the diagnostic of Section 6.8 and write no file when `--colouring` is also given, because the canonical form fixes the order of the corners, so a user predicts the mirror of each digit from the signature that `escher info` prints, `escher-render` alone decides which domains and marks give faces (RND-65 of `docs/design/render.md`), and a colouring of the tiles has no meaning for the faces of a construction.

CLI-63. With `--wythoff` and without `--arrangement`, `render` MUST draw the Wythoff fill of `escher-render`, MUST take exactly as many items of `--colours` as `face_types()` of the construction, each a colour, MUST stop at step 4 with the diagnostic of Section 6.8 and write no file for an item that is not a colour, an image item included, and without `--colours` MUST take the first entries of the table of default colours, one for each face type, and MUST run no colouring search, because each face type takes one solid colour, as the owner decided (`DEC-68`, `DEC-105`), an image item that `render` accepted would give the user an image without the picture and without a diagnostic, a wrong number of colours leaves a face type without a colour or a colour without a face type, and the default colours give the faces the colours of the table of a colouring with as many colours.

`--arrangement` puts one picture on every face of a regular construction, turned or mirrored as an arrangement says (Sections 5.29 to 5.31 of `docs/design/render.md`). The owner decided the flow (`DEC-111`): `info` lists the arrangements of a tiling by id, a mask command writes one face mask for an arrangement, the user paints anything on it, and `render` puts that picture on every face. The arrangements with mirrored neighbours are offered too (`DEC-112`), and the first scope is the regular tilings (`DEC-113`). This design proposes the commands and options that carry the flow (`DEC-119`): `info --wythoff` for the listing, `render-masks` with `--wythoff` and `--arrangement` for the mask, and `render` with the same two options for the picture. It also proposes the scope (`DEC-120`): the regular tilings, with every other construction with one face type refused with a diagnostic, and `DEC-140` adds the constructions with several face types. The owner decided on 2026-11-24 that a regular tiling can use several pictures, one for each orbit of its arrangement, that the user picks an arrangement by id, gets one mask for each picture, and passes as many images, that the arrangements with two and three pictures are listed beside those with one (`DEC-124`), and that arrangements with more than three pictures are allowed (`DEC-129`). The id form, the listing of Section 5.10, the names of the mask files, the order of the items of `--colours`, and the rules below are proposals (`DEC-115`, `DEC-119`, `DEC-125` to `DEC-128`).

An arrangement id has the form of a colouring id, the grammar of Section 7.15 of `docs/design/group.md`, so its number is the index of the arrangement, which is twice the number of sides $n$ of a face times the number of pictures $m$, and its letters rank it among the arrangements of that index (GRP-45 of `docs/design/group.md`). The hexagons of `*632` with `010` have the ids `12a` and on with one picture, `24a` and on with two, and so on, and the triangles of `*632` with `100` the ids `6a` and on with one picture. The number of an id therefore tells how many pictures it needs, and two arrangements with different numbers of pictures have different numbers, so their ids never collide, and an id with one picture keeps the meaning that `M24` gave it (`DEC-126`). For a regular construction, the index limit of `escher-group`, 64, bounds $2nm$, so a face with $n$ sides has arrangements with at most $\lfloor 32/n \rfloor$ pictures: 10 for triangles, 8 for squares, 6 for pentagons, 5 for hexagons, 4 for heptagons and octagons, 3 for faces of 9 and 10 sides, 2 for faces of 11 to 16 sides, and 1 for faces of 17 to 32 sides, and the pictures are named with the letters A to J at most. `render` and `render-masks` accept every such id, and `info --wythoff` lists the numbers of pictures that Section 5.10 states. Other documents cite this sentence for the numbers of pictures that the index limit allows. The same text can name a colouring of `--colouring`, as an id names a colouring of one set only (GRP-37), and the two options never read each other's ids. A private value parser in `args` reads `--arrangement` with `ColouringId::parse_arrangement` of `escher-group`, which accepts the same text with a number up to the arrangement index limit of 84 (GRP-48), and for any other value returns the message `expected an arrangement id such as 12a`, which `clap` puts into its usage error.

The owner decided that a tiling with several kinds of polygon gets one arrangement for every kind (`DEC-137`), and that the limit of the search is raised for it unless the tests then run too long (`DEC-138`). This design proposes the rest (`DEC-139` to `DEC-145`). On a construction with several face types, the id has the same form, its number $t$ is the index of the arrangement, a multiple of the least index $L$, and the arrangement has $t / (2 n_k)$ pictures of the type at a corner of order $n_k$. The search of such a construction takes the arrangement index limit of 84 (GRP-47 of `docs/design/group.md`), so a construction whose least index is above 84, such as `*752` with `111`, whose least index is 140, gets no arrangement. The regular constructions keep the limit of 64 of `IndexLimit` for their listing, their resolution, and their diagnostics (`DEC-139`). The marks decide whether a construction is regular or has several face types, so an id is always read against one of the two, and one text never names an arrangement of both. A number from 65 to 84 parses for every construction, and CLI-75 states what a regular construction does with it.

CLI-65. `render` and `render-masks` MUST accept as the value of `--arrangement` exactly the text that `ColouringId::parse_arrangement` accepts, MUST give a usage error with the message above for every other value, and MUST stop at step 4 with the diagnostic of Section 6.8 or 6.10 and write no file when `--arrangement` is given without `--wythoff`, because an arrangement belongs to the faces of a construction, and a second id grammar could accept an id that `escher-group` cannot rank.

To resolve the arrangement, step 6 takes the presentation, the stabilizer, and the index $2n$ of the whole face of step 4, and the number of pictures $m$ of the id. The index of the search is $2nm$, the number of the id, when that number is a multiple of $2n$, and $2n$ otherwise, as in `M24`. Step 6 makes that index an `IndexLimit`, which keeps the limit of 64 for a regular construction (`DEC-139`), and calls `free_classes_of_index` of `escher-group` with its value with `SearchBudget::DEFAULT` through a private function that takes the budget as a parameter, as for a colouring id (Section 4.5). The outcomes are:

| Outcome | Result |
| --- | --- |
| `IndexLimit::new` refuses the index, which is above 64 for a face with more than 32 sides | Stop, with `error: cannot render <signature>` and a `caused by` line with the `Display` text of the error of `escher-group` |
| The search is complete, and its list is empty | Stop, with the diagnostic of Section 6.8 that the tiling has no arrangement with $m$ pictures |
| The search is complete, and `find` returns no arrangement for the id | Stop, with the diagnostic of Section 6.8 that the arrangement does not exist, which gives the number of arrangements with $m$ pictures |
| `find` returns the arrangement | Step 6 goes on with it |
| The search stopped at a limit | Stop, with the diagnostic of Section 6.8 that the search stopped at a limit, which claims nothing about the id |
| `free_classes_of_index` returns another error | Stop, with `error: cannot render <signature>` and a `caused by` line with the `Display` text of the error |

An id whose number is not a multiple of $2n$ names no arrangement, because the index of an arrangement is a multiple of $2n$ (`UNV-51`). The search at $2n$ still runs for it, as in `M24`, so that the diagnostic can give the number of arrangements with one picture, and its step budget bounds its time as for a colouring id. The diagnostic that an id does not exist names the number of pictures of the id in every case, so its text for one picture gains ` with 1 picture` over that of `M24`, and the other texts for one picture are those of `M24` (`DEC-128`).

CLI-66. `render` and `render-masks` MUST resolve an arrangement id through one call of `free_classes_of_index` with the presentation and the stabilizer words of the `WholeFace` of the domain that they draw, at the index $2nm$ that the number of the id gives when it is a multiple of $2n$ and at $2n$ otherwise, MUST run no other search, MUST stop at step 6 with a diagnostic and write no file when the tiling has no arrangement with that number of pictures, the id names none, the index is above the index limit, or the search stops at a limit, and MUST NOT claim that an arrangement does not exist when the search stopped at a limit, because the ids that `info --wythoff` lists come from the same call at the same index, a search at any other index costs time that the image does not need, and a stopped search shows nothing (GRP-35 of `docs/design/group.md`).

CLI-67. With `--arrangement`, `render` MUST take exactly one item of `--colours` for each picture of the id, each a colour or an image item, which it reads and decodes as CLI-60 states, MUST give the item in position $k$ to picture $k$, counted from A, MUST take the first default colours, one for each picture, without `--colours`, and MUST draw the whole-face fill of `escher-render`, because the owner's flow paints one image over the mask of each picture and passes as many images (`DEC-111`, `DEC-124`), the position of an item names its picture as the number of a mask file does (CLI-71), or with several face types the order of the picture names of the mask files (CLI-76), and a solid colour draws the faces with their borders, as `--wythoff` alone does. An arrangement of a regular construction has at most 10 pictures, so the 12 default colours always suffice for it. With several face types, the items are read in the order of the pictures of Section 5.36 of `docs/design/render.md`, by corner and then by letter, and an arrangement can have up to 63 pictures, so without `--colours` an arrangement with more than 12 pictures stops at step 4 with the diagnostic of Section 6.8 that asks for as many colours with `--colours`, as CLI-44 states for a colouring (CLI-75).

To resolve an arrangement id of a construction with several face types, step 6 calls `free_classes_of_index` with the presentation and the stabilizer words of the whole face of step 4 at the index $t$ of the id when $t$ is a multiple of $L$, and at $L$ otherwise, with `SearchBudget::DEFAULT`, through the private function of CLI-66, with the outcomes of the table above. An index above 84 gives the error of `escher-group` at once, before any search (GRP-47). The diagnostic that an id does not exist names the total number of pictures of the index searched, in a text such as `the tiling has 3 arrangements with 6 pictures`, and that a search finds none, such as `the tiling has no arrangement with 6 pictures`.

An id whose number is not a multiple of $L$ is searched at $L$, as an id whose number is not a multiple of $2n$ is searched at $2n$ for a regular construction. Every arrangement that the search at $L$ returns has an id with the number $L$, so such an id names none, and step 6 stops with the diagnostic that it does not exist, with the number of arrangements at $L$ and their total $m_L$ of pictures. For such an id, step 4 counts $m_L$ items of `--colours`, takes the first $m_L$ default colours without `--colours`, and stops with the diagnostic of CLI-44 when $m_L$ is above 12, so the diagnostics of the items name the same number of pictures as the diagnostic of step 6.

CLI-75. With `--arrangement` on a construction with several face types, `render` and `render-masks` MUST resolve the id through one call of `free_classes_of_index` at the index of the id when it is a multiple of the least index and at the least index otherwise, MUST read one item of `--colours` for each picture of every type in the order of the corners and then of the letters, with the pictures of the least index for an id whose number is not a multiple of it, MUST stop at step 4 with the diagnostic of CLI-44 when `--colours` is absent and the arrangement has more than 12 pictures, and `render-masks` MUST write the masks by CLI-76. On a regular construction, `info`, `render`, and `render-masks` MUST keep every listing and diagnostic of `M25`, and every mask and image of `M25` that RND-87 of `docs/design/render.md` keeps, and the search MUST keep the limit of 64 (`DEC-139`), with one exception: an id whose number is from 65 to 84, which the parser accepts for every construction (CLI-65) and for which `M25` gave a usage error, MUST be refused at step 4, directly after `WholeFace::new` and before any item of `--colours` is read, with the status `Failure` and the diagnostic of Section 6.8 or 6.10 that carries the error of `IndexLimit::new` for the number of the id. The reasons are that the owner chose one arrangement for every kind (`DEC-137`), the order of the items and of the files must be one order that a user can read from the listing, a table that cycles colours would give two pictures colours that a viewer cannot tell apart, every command of a regular construction must stay as it was, except the masks and images that the one scale of an arrangement with several pictures changes (RND-87 of `docs/design/render.md`), and step 4 is the first step that reads the id against the construction: an id above 64 names no arrangement of a regular construction, and refusing it there, with one text for every such id, keeps a regular construction from counting more pictures than the 12 default colours cover (CLI-67).

`--arrangement` combines with every other option of `render` as `--wythoff` does: `--colouring` is refused (CLI-62), every reflection triangle has no shape parameter, so a setting of `--set` gives the diagnostic of a signature without parameters, and the model, the rotation, the light, the shading, the border width with its unit, the border colour, the background, the sample grid, and the fold cap apply unchanged. The picture does not depend on the model, the size, the sample grid, or the border, as an image fill does not (Section 5.31 of `docs/design/render.md`).

`escher-render` limits the image size, the number of pixels, the sample grid, the border width, and the fold cap, and returns an error kind for a value outside its limits, such as a width of 0, a grid of 0, a border width that is negative or not finite, a rotation angle that is not finite, or a light direction that is zero or not finite. Section 4.1 of `docs/design/render.md` states each limit and the error kind for a value outside it. `render` reports such an error as Section 6.8 states. The border width has the same limits in both units, from 0 to 1024, so `--border-unit intrinsic --border-width 1025` gives this diagnostic, as it does with `pixels`:

```text
error: cannot render *732
  caused by: cannot build render options: the border width 1025 is larger than 1024
```

CLI-24. `render` MUST pass the size, the sample grid, the border width with its unit, the fold cap, the components of the light direction, the angles of the viewing rotation after their conversion to radians, the hyperbolic model, and the position of `--centre` to `escher-render` unchanged, and MUST NOT check them against a limit itself, because a second check in this crate can differ from the limits that `docs/design/render.md` states, and a value would then pass one check and fail the other.

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

### 4.8. The `render-masks` Command

`render-masks` takes one required positional argument, the signature, as a `String` named `SIGNATURE`, the required option `--output`, and the options `--size`, `--colouring`, `--set`, `--wythoff`, and `--arrangement`, with the value syntax, the parsers, and the defaults of the same options of `render` (Section 4.6). One `clap` argument struct in `args`, which both commands flatten into their own, defines the signature and these five options, so the syntax, the parsers, and the defaults of the two commands cannot differ. `render-masks` takes no model, centre, rotation, light, shading, border, sample grid, fold cap, colours, background, threads, or statistics, because a mask shows the mask polygon, which depends only on the domain, and its borders, which depend only on the colouring (RND-58 of `docs/design/render.md`), and a face mask depends only on the construction and the arrangement (RND-74 of that document). An option given more than once is a usage error, except `--set`.

This command writes the masks of the colouring `4a` of `*642`, one for each of its 4 colours, to `/tmp/masks/mask-0.png` to `/tmp/masks/mask-3.png`:

```sh
escher render-masks '*642' --colouring 4a -o /tmp/masks/mask.png
```

After the user has painted an image over each mask and saved it under a name of their choice, this command renders the tiling with the four images:

```sh
escher render '*642' --colouring 4a --colours 'image:/tmp/masks/fill-0.png,image:/tmp/masks/fill-1.png,image:/tmp/masks/fill-2.png,image:/tmp/masks/fill-3.png' -o tiling.png
```

When `render` uses the same signature, settings, and colouring as `render-masks`, each image fills every tile of its colour through the mask map, and the images meet across the seams as the masks do when laid edge to edge. `render` samples the same images at any size, model, rotation, or border (RND-59 of `docs/design/render.md`).

`render-masks` runs these steps in this order. It stops at the first step that fails and writes the diagnostic of Section 6.10, except at step 1, whose diagnostic is that of Section 6.1:

1. Parse the signature with the `FromStr` implementation of `Signature` (CLI-5).
2. Find the file name and the extension of the value of `--output` (CLI-58).
3. Build the domain with the settings, as step 3 of `render` does, under CLI-29.
4. Choose the colouring: the value of `--colouring`, or the default of CLI-34, with its search.
5. Check the size: build the render options of `escher-render` from the size and a placeholder fill of white, with every other option at the default of `escher-render`, and drop the result, as step 5 of `render` does.
6. Resolve the colouring as `render` does with an image item: `one` and `1a` as CLI-61 states, `parity` as CLI-61 and the paragraph after it state, and any other id as CLI-43 states, on every domain.
7. Build the masks with `Masks::new` of `escher-render` from the domain and the colouring, which refuses a domain without a mask polygon (RND-63 of `docs/design/render.md`).
8. For each colour from 0 to $k - 1$ in turn, on the calling thread: render its mask with `Masks::render` at the size, encode it with `encode_png`, and write it to its mask file with `std::fs::write`.

A successful `render-masks` writes nothing to standard output or standard error. Steps 2 to 7 check every input before step 8 writes the first file, so a failed check writes no file. Step 8 renders, encodes, and writes one mask at a time, so it holds one mask in memory. A failed write leaves the mask files of the lower colours written, can leave the file of its own colour truncated or partly written, as Section 6.8 states for the output file of `render`, and writes no file of a higher colour.

Without `--wythoff`, `render-masks` refuses `--arrangement` after step 3, before it chooses the colouring. With `--wythoff`, it writes the face masks of the arrangement in place of the polygon masks, one for each picture. The number of pictures of an id depends on the whole face that step 4 builds (CLI-66, CLI-75), so the file names cannot be chosen before step 4. With `--wythoff`, the steps therefore run in the order 1, 3, 4, 2, 5, 6, 7, 8, and steps 2, 4, 6, 7, and 8 are replaced as follows:

- Step 4 refuses `--colouring`, as CLI-62 states, then refuses a missing `--arrangement`, and then builds the construction and the whole face as step 4 of `render` does.
- Step 2, after step 4, finds no file name for an id with one picture, because its face mask is one file, written to the value of `--output` itself (CLI-69). For an id with several pictures, it finds the file names of the masks of pictures 0 to $m - 1$, A to the last picture, by the rule of CLI-58, as for the masks of $m$ colours, and stops when the value of `--output` has no file stem (CLI-71). The number of pictures comes from the number of the id, as step 4 of `render` reads it. A construction with several face types always has several pictures, and step 2 names their masks by their picture names (CLI-76), so `-o tile.png` gives `tile-0A.png` for `0A` and so on.
- Step 6 resolves the arrangement as CLI-66 states.
- Steps 7 and 8 render the face mask of each picture in turn, in the order A, B, C, or with several face types in the order of the picture names of CLI-75, with `WholeFace::render_mask` at the size, encode it with `encode_png`, and write it with `std::fs::write`, to the value of `--output` for one picture and to the name of step 2 for several.

So with `--wythoff`, `render-masks` refuses in this order: a bad signature, a bad setting of `--set`, `--colouring`, a missing `--arrangement`, the refusals of `Wythoff::new` and `WholeFace::new`, an id from 65 to 84 on a regular construction (CLI-75), an output path with no file stem for an id with several pictures, a bad size, and then the failures of the resolution of the arrangement. Without `--wythoff`, an output path with no file stem is refused at step 2, before the domain is built.

CLI-69. With `--wythoff`, `render-masks` MUST require `--arrangement`, and for an arrangement with one picture MUST write exactly one file, the face mask, to the value of `--output` as given, only after steps 1 to 6 have succeeded, and MUST replace an existing file without asking and create no directory, because an arrangement with one picture has one face mask, a name derived from the path would add a rule for one file, and a failed check then changes no file, as for `render` (CLI-21, `DEC-118`).

CLI-71. With `--wythoff` and an arrangement of a regular construction with $m$ pictures for $m \ge 2$, `render-masks` MUST write exactly $m$ files, the face mask of picture $k$, counted from 0 for A, to the name that CLI-58 gives for colour $k$, MUST stop at step 2, which runs after step 4, with the diagnostic of Section 6.10 when the value of `--output` has no file stem, MUST write no file before steps 1 to 6 have succeeded, MUST write the files in the order of the pictures, each after its mask has rendered and encoded, and MUST replace an existing file without asking and create no directory, because the owner asked for one mask for each picture (`DEC-124`), the rule of the polygon masks already names several files from one path, so the user learns no second rule, and the number in the name is the position of the item of `--colours` that takes the picture (CLI-67) (`DEC-127`).

So `-o face.png` gives `face.png` for the arrangement `12a` of the hexagons and `face-0.png`, `face-1.png`, and `face-2.png` for an arrangement with three pictures, such as `18a` of the triangles of `*732`. A failed write leaves the masks of the earlier pictures written, as CLI-59 states for the polygon masks.

With several face types, an arrangement can have up to 63 pictures, and a user who had to count from `tile-0.png` to `tile-40.png` to find a picture would need the listing beside the files. So this design proposes that each mask file carries its picture name, the name that the listing of Section 5.10 prints in ranges such as `0A to 0F of 14 sides`, with the extension rule of CLI-58 (`DEC-144`). Point 62 of Section 11.3 puts the alternative, numbers as for a regular construction, to the owner. A regular construction keeps the names of CLI-69 and CLI-71.

CLI-76. With `--wythoff` and an arrangement of a construction with several face types, `render-masks` MUST write exactly one file for each picture, the face mask of the picture with the picture name $X$, named by the file stem of the value of `--output`, followed by `-`, $X$, and, when the value has an extension, `.` and that extension, in the directory of the value, as CLI-58 names a mask with $X$ in place of the colour. It MUST stop at step 2, which runs after step 4, with the diagnostic of Section 6.10 when the value of `--output` has no file stem, MUST write no file before steps 1 to 6 have succeeded, MUST write the files in the order of the picture names, by corner and then by letter, each after its mask has rendered and encoded, MUST stop at the first write that fails, with the diagnostic of Section 6.10 that names the picture, and write no later file, and MUST replace an existing file without asking and create no directory, because a user finds the mask of each picture by the name that the listing prints for it, the order of the files is the order of the items of `--colours` (CLI-75), so a failed write tells the user which files were written, and a failed check then changes no file, as for the other masks (CLI-59).

So `-o tile.png` gives `tile-0A.png`, `tile-1A.png`, `tile-2A.png`, and `tile-2B.png` for an arrangement of index 8 of `*442` with `111`, in that order, `-o tile` gives `tile-0A` to `tile-2B`, and `-o archive.tar.png` gives `archive.tar-0A.png` and so on.

CLI-64. Withdrawn on 2026-11-23.

CLI-58. `render-masks` MUST name the mask file of colour $c$ by the file stem of the value of `--output`, followed by `-`, $c$ in decimal with no leading zero, and, when the value has an extension, `.` and that extension, in the directory of the value, with the stem and the extension of `std::path::Path::file_stem` and `std::path::Path::extension`, and MUST stop at step 2 with the diagnostic of Section 6.10 when the value has no file stem, because the user finds each mask by the colour that `--colours` takes in the same position, a rule on the path alone tells the user every file that the command can write before it runs, and a path such as `/` or `..` names no file.

So `-o mask.png` gives `mask-0.png` to `mask-3.png` for 4 colours, `-o masks` gives `masks-0` to `masks-3`, `-o archive.tar.png` gives `archive.tar-0.png`, and `-o mask.` gives `mask-0.` to `mask-3.`, because `Path::extension` gives an empty extension for a name that ends in a full stop. The value is any path, as for `render`, and the names keep the bytes of a stem or an extension that is not UTF-8.

CLI-59. `render-masks` MUST NOT create, truncate, or otherwise change a mask file before steps 2 to 7 have succeeded, MUST write the mask files in the order of the colours, each after its mask has rendered and encoded, MUST replace an existing file without asking, and MUST create no directory, because a failed check then changes no file, the order tells the user which files a failed write left, and the files are replaced as the output file of `render` is (`DEC-30`).

The mask files are written as Section 4.7 states for the output file of `render`, with `std::fs::write`. A file of another program that has the name of a mask file is replaced. `Masks::render` uses a fixed 4 by 4 sample grid, the default of `render` (`DEC-7`, Section 5.19 of `docs/design/render.md`), so the edges and borders of a mask are antialiased as a default image of `render` is. `render-masks` starts no thread, because a mask runs no fold and its cost is bounded by the size (Section 13 of `docs/design/render.md`).

## 5. Information Text

### 5.1. Lines

`info` writes the lines of this table, in this order. Each line is the label, a colon, one space, and the value, followed by a line feed. No other line, blank line, or trailing space appears. The accessors of rows 1 to 9 and 35 are those of `Signature` (Sections 3.1 and 9.1 of `docs/design/signature.md`). Row 10 takes its values from `parameters` and `ParameterSpec`, rows 11 to 16 and 28 to 34 from `FundamentalDomain`, `Side`, `Verification`, and `Check` (Section 3.1 of `docs/design/domain.md`), rows 17 to 19 from the colouring search of `escher-group` on `side_presentation()` (Section 5.8), and rows 20 to 27 from `escher-group` and `side_presentation()` (Section 5.7). "A domain is built" means that `FundamentalDomain::build` returned `Ok`.

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
| 35 | `Regular tiling` | The canonical signature is `*pq2` or `pq2` with finite orders $p \ge q \ge 3$ (Section 5.9) | The Schlafli symbols of the tiling and of its dual, such as `{7,3}, dual {3,7}`, or `{4,4}, self-dual` when $p = q$, in the form of Section 5.9 |
| 10 | `Parameter` | Once for each item that `parameters` returns for the signature, in its order | The name, `, range `, the minimum, ` to `, the maximum, `, default `, and the default value, with the numbers in the form of Section 5.2, such as `aspect, range 0.125 to 8, default 1` |
| 11 | `Domain` | Always | `no sides` when the domain has no side, `<n> mirror sides` when every side is a mirror, `<n> sides in <k> pairs` when no side is, with `pair` when $k = 1$, as in `2 sides in 1 pair`, `<n> sides, <m> mirror sides and <k> pairs` when the domain has both, and `none, ` followed by the `Display` text of the error when no domain is built (Section 5.5, CLI-51, CLI-53) |
| 12 | `Mirror sides`, then `Side pairs` | A domain with a paired side is built. The `Mirror sides` line appears only when that domain also has a mirror side | Two lines in this order. `Mirror sides` lists the mirror sides, such as `2 and 5`, and `Side pairs` each pair of partner sides once, such as `0 and 1 by rotation, 2 and 3 by rotation`, in the forms of Section 5.5 |
| 13 | Withdrawn on 2026-11-22 | Never | None. Rows 18 and 19 show whether the parity colouring exists |
| 14 | `Corner angles in degrees` | A domain with at least one side is built (CLI-53) | Each value of `corner_angles()` in degrees, in corner order, in the form of Section 5.2 |
| 15 | `Side lengths` | As row 14 | Each value of `side_lengths()`, in side order, in the form of Section 5.2 |
| 16 | `Domain area` | A domain is built | `area()`, in the form of Section 5.2 |
| 17 | `Colourings` | A domain is built | The number of valid colourings and the colour count to which the list is complete, or `none, ` followed by the `Display` text of the error, in the form of Section 5.8 |
| 18 | `Colourings with <k> colours` | Row 17 has a number, once for each colour count $k$ up to the colour count of row 17 that has at least one valid colouring, in increasing order | The number of valid colourings with $k$ colours, in the form of Section 5.8 |
| 19 | `Colouring` | `--colourings` is given and row 17 has a number, once for each valid colouring that row 18 counts, in the order of the ids | The id, the colour count, and an alias, in the form of Section 5.8 |
| 36 | `Arrangements` | `--wythoff` is given and a domain is built | The number of arrangements at the least index, with one picture for a regular construction, or `none, ` followed by the `Display` text of the error or by `the search stopped at a limit`, in the form of Section 5.10 |
| 37 | `Arrangement` | Row 36 or a row 38 has a number, once for each arrangement that it counts, in the order of the ids, directly after that row | The id, the number of pictures when it is more than one, with several face types the names and the numbers of sides of the pictures of each type, whether a neighbour is mirrored, and the edge pairing, in the form of Section 5.10 |
| 38 | `Arrangements with <m> pictures` | Row 36 has a number, once for each further index that Section 5.10 lists, in increasing order, up to the first search that stops at a limit | The number of arrangements with $m$ pictures in all, or `none, the search stopped at a limit`, in the form of Section 5.10 |
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

Row 12 holds two labels, row 13 stays withdrawn, row 35 stands between rows 9 and 10, and rows 36 to 38 stand between rows 19 and 20, so that the numbers of the other rows stay those that other sections cite. The order of the rows in the table is the order of the lines, except that the arrangement lines repeat: row 36 with its rows 37, then each row 38 with its rows 37 (Section 5.10).

CLI-6. `info` MUST print exactly the lines of this table, in this order and under these conditions, with the colouring lines for every built domain, the `Colouring` lines only with `--colourings`, the arrangement lines only with `--wythoff`, the presentation lines only with `--presentation`, and the verification lines only with `--check`, because the snapshots of Section 8 and every reader of the text rely on one layout, and an extra or missing line changes what a reader concludes about the signature. The table has no name of the group in another notation, such as an IUC or Schoenflies name, because `DEC-20` omits them until a primary source is read. Row 35 names a tiling and not the group, so `DEC-20` does not cover it.

So a signature without a domain gets the lines of rows 1 to 10 and 35 that apply to it, the `Domain` line of row 11, no colouring line, and with `--presentation` the lines of rows 20 to 23 that apply to it, with or without `--check`. Rows 28 to 34 follow the order of the checks of Section 8.2 of `docs/design/domain.md`.

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
Regular tiling: {7,3}, dual {3,7}
Domain: 3 mirror sides
Corner angles in degrees: 25.714286, 60.000000, 90.000000
Side lengths: 0.545275, 0.620672, 0.283128
Domain area: 0.074800
Colourings: 3, complete to 12 colours
Colourings with 2 colours: 1
Colourings with 8 colours: 1
Colourings with 9 colours: 1
Check isometries: passed, residual <residual>, tolerance 1.0e-9
Check pairings: passed, residual <residual>, tolerance 1.0e-9
Check walls: passed, residual <residual>, tolerance 1.0e-9
Check relators: passed, residual <residual>, tolerance 2.0e-6
Check angles: passed, residual <residual>, tolerance 1.0e-9
Check area: passed, residual <residual>, tolerance 1.0e-9
Verification: passed
```

Standard error is empty and the status is `Success`. The Euler characteristic comes from the defect rule of ON 6.1, the area from ON 7.3, and the dimension from ON 10.1. The regular tiling line follows the rule of Section 5.9. The tolerances are those of Section 8.4 of `docs/design/domain.md`. `Domain area` is the area of the triangle measured from its angles, and `Area` is the exact value from the Euler characteristic.

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
Colourings: 7, complete to 12 colours
Colourings with 4 colours: 1
Colourings with 6 colours: 2
Colourings with 8 colours: 1
Colourings with 12 colours: 3
```

The table of ON 8.2 gives the order 24, and the Euler characteristic $2/24 = 1/12$ and the area come from ON 7.3. A spherical signature has no `Deformation dimension` line.

This example shows a Euclidean signature with two shape parameters and no reversing side pairing:

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
Corner angles in degrees: 90.000000, 90.000000, 90.000000, 90.000000
Side lengths: 1.000000, 1.000000, 1.000000, 1.000000
Domain area: 1.000000
Colourings: 104, complete to 12 colours
Colourings with 2 colours: 1
Colourings with 3 colours: 2
Colourings with 4 colours: 5
Colourings with 5 colours: 4
Colourings with 6 colours: 10
Colourings with 7 colours: 6
Colourings with 8 colours: 13
Colourings with 9 colours: 11
Colourings with 10 colours: 16
Colourings with 11 colours: 10
Colourings with 12 colours: 26
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

For a supported signature whose largest corner order is above the limit of `escher-domain`, such as `*(101)32`, rows 1 to 9 and 35 apply as for `*732`, with `Regular tiling: {101,3}, dual {3,101}`, and the last line is `Domain: none, cannot build fundamental domain: order 101 is larger than 100`, as Section 10.2 of `docs/design/domain.md` fixes it. A reflection polygon with more than 8 corners, such as `*222222222`, and its rotation signature, such as `222222222`, print no `Parameter` line, because `parameters` gives none for them, and their last line is `Domain: none, cannot build fundamental domain: the signature has 9 orders, more than 8`, with the number of orders in place of `9` (DOM-38 and Section 10.2 of `docs/design/domain.md`). That limit applies before the limit on the corner order, so `*(101)22222222` gets the same text. `2*n` has a limit of its own, $n$ up to 50 (DOM-63 of `docs/design/domain.md`, `DEC-59`), so `2*(51)` prints the group order 204 and the last line `Domain: none, cannot build fundamental domain: order 51 is larger than 50`, while `(101)*` and `(101)x` print the text of the limit of 100.

### 5.4. Stability and Omissions

The information text is not a stable interface, and a machine-readable form is a roadmap item (`DEC-23`).

CLI-7. A change that alters the information text MUST update its snapshots in the same change, because the snapshot test fails otherwise, and the snapshot diff is how a reviewer sees each change to the text (`DEC-23`, rule TEST-2 of `docs/rust-coding-standards.md`).

`info` prints presentations only with `--presentation` (Section 5.7). It prints the colouring lines for every built domain, and names the colourings only with `--colourings` (Section 5.8). The owner's goal that `info` reports how many polygons the representative set has, and so which colour counts are possible, is met by the `Colourings with <k> colours` lines: each colour count $k$ that they list is a number of colours that `render` can draw with an id in a valid colouring. One colour is always available through `one` and is listed only for `1` (`DEC-73`). `info` prints no colour and no name of a colouring other than its id and its alias. The domain summary covers exactly the signatures that `escher-domain` constructs (Section 4 of `docs/design/domain.md`).

### 5.5. Domain Summary

For every valid signature, `info` calls `parameters` once and prints the parameter lines of row 10, then calls `FundamentalDomain::build` once and prints the `Domain` line of row 11. Rows 12 and 14 to 16 follow only when a domain is built, rows 12, 14, and 15 only under their conditions.

When a domain is built, `info` counts its sides and its sides of kind `PairingKind::Mirror`, and the value of the `Domain` line takes one of four forms. The numbers are in decimal, and the number of pairs is the number of sides that are not mirrors, divided by 2, because every such side has a partner other than itself:

| Mirror sides | Form | Example |
| --- | --- | --- |
| No side exists | `no sides` | `no sides` for `1` |
| Every side | The number of `sides()`, one space, and `mirror sides` | `3 mirror sides` for `*732` |
| None | The number of `sides()`, ` sides in `, the number of pairs, and ` pairs` (` pair` when the number of pairs is 1) | `4 sides in 2 pairs` for `732` and for `xx` |
| Some | The number of `sides()`, ` sides, `, the number of mirror sides, ` mirror sides and `, the number of pairs, and ` pairs` (` mirror side` and ` pair` for the number 1) | `6 sides, 2 mirror sides and 2 pairs` for `22*` |

CLI-51. `info` MUST choose the form of the `Domain` line from the number of sides of kind `PairingKind::Mirror`, and MUST NOT choose it from `reverses_orientation()`, because `reverses_orientation()` is true for every one of the nine new Euclidean domains except that of `o` (Section 3.2 of `docs/design/domain.md`), so a summary chosen from it would print `4 mirror sides` for `xx` and for `22x`, whose sides are all paired, by glide reflections and translations in `xx` and by glide reflections alone in `22x`. When the domain has both mirror sides and paired sides, `info` MUST print the `Mirror sides` line before the `Side pairs` line, because a reader needs to know which sides are mirrors to read the pairs (`DEC-54`).

CLI-53. For a domain with no side, `info` MUST print `Domain: no sides` and MUST NOT print the `Corner angles in degrees` and `Side lengths` lines, because with the forms of CLI-51 the line would read `0 mirror sides`, which suggests mirrors that do not exist, and the two lists would be empty lines with a trailing space, which the snapshots of Section 8.1 would hold as text that no reader can use. For a side of kind `PairingKind::RotaryReflection`, the `Side pairs` line MUST name the kind `rotary reflection`, because the word `another isometry` tells a reader nothing about the pairing, and a rotary reflection, which reverses orientation, is what tells `nx` from `nn` (`DEC-54`, `DEC-57`).

The value of the `Mirror sides` line lists the indices of the mirror sides in increasing order, in decimal: one index alone, two joined by ` and `, and three or more separated by a comma and one space, with `, and ` before the last, such as `2 and 5` for `22*` and `2, 3, and 4` for `2*22`.

CLI-32. For a domain with a paired side, `info` MUST print the `Domain` line with the number of pairs, and one `Side pairs` line that names each pair of partner sides once, because a reader of `<n> mirror sides` for such a domain would take its sides for mirrors, and the pairs and their kinds are what tell a group from another with the same tiles, such as a rotation subgroup from its reflection group, or `22*` from `**`.

The value of the `Side pairs` line lists one item for each side $i$ whose `partner()` $j$ is larger than $i$, in increasing order of $i$, separated by a comma and one space. An item is $i$ in decimal, ` and `, $j$ in decimal, ` by `, and the word for `kind()` of side $i$: `rotation` for `PairingKind::Rotation`, `translation` for `PairingKind::Translation`, `glide reflection` for `PairingKind::GlideReflection`, and `rotary reflection` for `PairingKind::RotaryReflection`, the kinds that Sections 3.1 and 3.2 of `docs/design/domain.md` define (DOM-27, DOM-61). For `732` the value is `0 and 1 by rotation, 2 and 3 by rotation`, for `2222` it is `0 and 1 by rotation, 2 and 5 by translation, 3 and 4 by rotation`, for `xx` it is `0 and 2 by glide reflection, 1 and 3 by translation`, and for `5x` and `x` it is `0 and 1 by rotary reflection`. The antipodal map of `x` has the kind `PairingKind::RotaryReflection` (`DEC-57`). A half-turn has the kind `PairingKind::Rotation`. `PairingKind` is `#[non_exhaustive]`, so the word is `reflection` for `PairingKind::Mirror` and `another isometry` for any later kind. No domain that `escher-domain` builds reaches either word, because a mirror is its own partner and appears in the `Mirror sides` line only.

CLI-33. Withdrawn on 2026-11-22.

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

When a domain is built, `info` prints the colouring lines after the domain summary, with or without any flag, as the owner decided (`DEC-38`). The lines count and list only the valid colourings (`DEC-73`), one for each conjugacy class of subgroups (`DEC-37`). `info` takes these steps:

1. Call `side_presentation()` on the domain once. Section 5.7 reuses the result.
2. When it returns a presentation, call the search of `escher-group` on it once, with `ColouringSet::ValidClasses`, the colour limit $N$ of `--max-colours` as its `IndexLimit`, and `SearchBudget::DEFAULT`, which no option changes (the rules of the search in `docs/design/group.md`). Without `--colourings` the call is `count_colourings`, which counts the colourings and stores none of them. With `--colourings` it is `colourings`, which returns them in the order of the ids. Either result is complete to a colour count $m$, its `complete_to()`, which is $N$ unless the search stopped at a search limit.
3. Print row 17. When it has a number, print one row 18 for each colour count from 1 to $m$ that has at least one colouring, and with `--colourings` one row 19 for each colouring of the list, in its order, which is the order of the ids of GRP-37 of `docs/design/group.md`.

The value of row 17 is chosen in this order:

| Case | Value |
| --- | --- |
| `side_presentation()` or the search returns an error | `none, ` followed by the `Display` text of the error |
| The search completed every colour count up to $N$ | `<n>, complete to <N> colours` |
| The search reached a search limit after it completed the colour count $m < N$ | `<n>, complete to <m> colours, the search stopped at a limit before <m + 1> colours` |

$n$ is the number of valid colourings with at most $m$ colours, and $m = N$ in the second row. Each number is in decimal, and the word is `colour` for the number 1, as in `complete to 1 colour` and `before 1 colour`. The third row also covers $m = 0$, which gives `0, complete to 0 colours, the search stopped at a limit before 1 colour`. One wording covers every search limit, the step budget and the limit on the stored colourings alike (GRP-35 and GRP-36 of `docs/design/group.md`), because the text claims only what the search completed.

The label of row 18 is `Colourings with `, the colour count $k$ in decimal, and ` colours`, with ` colour` for $k = 1$, such as `Colourings with 1 colour` and `Colourings with 8 colours`. Its value is the number of valid colourings with $k$ colours in decimal. A colour count without a valid colouring has no line, which for every domain other than that of `1` includes the colour count 1, so the lines of row 18 together with row 17 state every colour count up to $m$.

The value of row 19 is the id, `, `, the colour count in decimal, and ` colours`, with ` colour` for 1, then `, alias one` for the colouring `1a`, which only the domain of `1` lists, and `, alias parity` for the colouring `2a`, the parity colouring of Section 4.6, such as `1a, 1 colour, alias one`, `2a, 2 colours, alias parity`, or `4a, 4 colours`. An id holds only a decimal number and lower-case letters (GRP-37), so the lines need no replacement of characters.

CLI-45. For every built domain, `info` MUST print row 17 and the rows 18 in the forms of this section, and with `--colourings` the rows 19, MUST take every count, id, and order of the ids from `escher-group` unchanged, and MUST mark exactly the colouring `1a` with `alias one` and exactly the colouring `2a` with `alias parity`, because the counts are the colour counts of the valid colourings that `render` can draw, the ids are what a user passes to `render --colouring`, and a count or an order computed again in this crate could differ from the one that names the colouring in `render`.

CLI-46. `--max-colours` MUST set the colour limit of rows 17 to 19 together, with the default 12 and the range of Section 4.3, and with `--colourings` rows 17 and 18 MUST come from the same call as the rows 19, because the counts and the list then describe one search, and a count that came from another call could disagree with the list beside it. The colour limit changes no id: an id with $k$ colours is the same for every limit of at least $k$ (GRP-37, GRP-39).

`colourings` stops at its limit on the colourings that one call stores (GRP-36 of `docs/design/group.md`), and `count_colourings` stores none, so only its step budget stops it. With `--colourings`, row 17 can therefore name a lower colour count than without the flag, for the same signature and limit. Every colour count that both calls complete has the same number of colourings, because the result for each colour count up to `complete_to()` depends on the presentation alone (GRP-39).

Without `--colourings`, the numbers of rows 17 and 18 come from `ColouringCounts::colourings` of the call. With `--colourings`, they come from `Colourings::count` of the same call that gives the list, and `info` does not count the list again. Row 17 is the sum of the counts up to `complete_to()`.

CLI-47. When the search reaches a search limit, `info` MUST print the third form of row 17 and MUST print no line about a colour count above $m$, and every outcome of the colouring lines, including a search limit and an error, MUST leave the status as it would be without them, because a search that stopped at a limit shows nothing about the colour counts that it did not complete, as an enumeration that reaches its coset limit shows nothing about the index (Section 5.7), and the signature and its domain are still valid.

The colouring lines therefore never give `Failure`, and they write no diagnostic. A domain whose side presentation `escher-domain` refuses is not expected, because every domain that `build` returns has one (DOM-35 of `docs/design/domain.md`).

The ids depend on the numbering of the sides of the domain, because the columns of a coset table follow the generators of the side presentation. DOM-37 of `docs/design/domain.md` makes that numbering part of the contract of `escher-domain`.

This example lists the valid colourings of `*642` up to 4 colours. The lines of Section 5.3 come first, up to `Domain area`:

```sh
escher info --colourings --max-colours 4 '*642'
```

```text
Colourings: 8, complete to 4 colours
Colourings with 2 colours: 1
Colourings with 4 colours: 7
Colouring: 2a, 2 colours, alias parity
Colouring: 4a, 4 colours
Colouring: 4b, 4 colours
Colouring: 4c, 4 colours
Colouring: 4d, 4 colours
Colouring: 4e, 4 colours
Colouring: 4f, 4 colours
Colouring: 4g, 4 colours
```

Of Singerman's seven colourings with 2 colours of `*642` (Section 6 of `docs/research/colourings.md`), only the parity colouring is listed, and `*642` has no valid colouring with 3 colours, so Bulatov's colouring with 3 colours (Section 4.4 of that document) is not listed. Without `--colourings`, the same command prints only the lines up to the last `Colourings with` line.

A search that stops at a limit prints the third form of row 17:

```text
Colourings: <n>, complete to <m> colours, the search stopped at a limit before <m + 1> colours
```

The status stays as it is without the colouring lines (CLI-47). Up to 12 colours, such a stop happens for signatures with high corner orders, such as `*(100)(100)(100)`, which reaches the step budget. The reflection polygons have more sides, so their side presentations have more generators, and their searches reach the step budget at lower colour counts.

### 5.9. Regular Tiling Line

`info` prints row 35 of Section 5.1 when the canonical signature names the symmetry group of a regular tiling, or its rotation group (`DEC-67`). The line depends on the canonical signature alone, so it appears with or without a domain, and every spelling of one signature, such as `*237`, `*372`, and `*732`, gets the same line. A private function of the module `info` computes it from `handles()`, `crosscaps()`, `cone_orders()`, `boundaries()`, `Boundary::corner_orders()`, and `Order::finite()` of `escher-signature`, which needs no change (`DEC-83`).

The line appears exactly when the canonical signature meets all of these conditions (`DEC-81`):

1. `handles()` and `crosscaps()` are 0.
2. Either `cone_orders()` is empty and `boundaries()` holds one boundary with three corner orders, or `boundaries()` is empty and `cone_orders()` holds three orders. These are the orders $p$, $q$, and $r$, in the order of the canonical form.
3. All three orders are finite.
4. $r$ is 2 and $q$ is at least 3.

The canonical form writes the three orders of such a signature in descending order, by steps 3 and 4 of Section 7.2 of `docs/design/signature.md`, which turn `*236` into `*632`. So $p \ge q \ge r$, and the conditions select `*pq2` and `pq2` with $p \ge q \ge 3$. The conditions are the same in all three geometries. They fail for `*n22` and `n22`, whose second order is 2, for `*333`, which has no corner of order 2, for `*2222`, which has four corners, for `3*2`, which has a cone point and a mirror boundary, and for `*(inf)32`, which has an infinite order.

The value is `{p,q}, dual {q,p}` when $p > q$, and `{p,p}, self-dual` when $p = q$, with each order in decimal (`DEC-82`). The first symbol names the tiling by $p$-gons, where $p$ is the larger order, and the second names its dual. Both tilings have the same symmetry group (Section 5.1 of `docs/research/colourings.md`), so no source makes one of them the tiling of the signature. The larger order comes first, as in the canonical signature. The orders carry no parentheses, which the canonical form puts around an order of 10 or more, so `*(101)32` gives `{101,3}, dual {3,101}`.

| Signature | Line |
| --- | --- |
| `*732`, `*237`, `732` | `Regular tiling: {7,3}, dual {3,7}` |
| `*632`, `632` | `Regular tiling: {6,3}, dual {3,6}` |
| `*442`, `442` | `Regular tiling: {4,4}, self-dual` |
| `*532`, `532` | `Regular tiling: {5,3}, dual {3,5}` |
| `*432`, `432` | `Regular tiling: {4,3}, dual {3,4}` |
| `*332`, `332` | `Regular tiling: {3,3}, self-dual` |
| `*722`, `*333`, `*2222`, `3*2`, `*(inf)32` | No line |

Three statements of this section are unverified and form `UNV-37` (Section 10): that `*pq2` names the symmetry group of $\{p, q\}$, with the corner of order $p$ at the centre of a tile, that `pq2` names its rotation group, and that $\{q, p\}$ is its dual. The line never changes the status and writes no diagnostic.

CLI-57. `info` MUST print row 35 exactly for the canonical signatures that meet the conditions of this section, with the value of this section, and MUST compute it from the canonical signature alone, because a line that depended on the spelling of the argument or on the domain would give one group two descriptions, and a symbol for any other signature would name a tiling that no source read ties to its group.

### 5.10. Arrangement Lines

With `--wythoff`, when a domain is built, `info` prints the arrangement lines after the colouring lines. They list the arrangements of the regular tiling of the construction by id, first those with one picture in the form of `M24`, then those with two pictures, three, and so on, each number of pictures after its own count line (`DEC-111`, `DEC-119`, `DEC-124`, `DEC-126`, `DEC-129`). The end of this section states the listing of a construction with several face types, and the rest of it the listing of a regular construction.

The largest number of pictures $M$ is the largest $m$ that is at most the colour limit of `--max-colours` and has $2nm$ at most `IndexLimit::MAX`, which is 64. A face with more than 32 sides has $2n$ above 64, so no $m$ meets the second condition, and $M$ is 1: the listing then prints row 36 with the error of the index limit and no further line (step 2). The colour limit bounds the number of pictures here, and not the index, because a picture is what the user paints for each orbit, as a colour is for each coset of a colouring (`DEC-126`). With the default limit of 12, $M$ is the number of Section 4.6 for every face of at most 32 sides, such as 10 for triangles, 8 for squares, and 4 for heptagons, because no face allows more than 10.

`info` takes these steps:

1. Build the construction with `Wythoff::new` and the whole face with `WholeFace::new` of `escher-render`, which gives the index $2n$ of one picture.
2. For $m$ from 1 to $M$, in increasing order, call `free_classes_of_index` of `escher-group` once, with the presentation, the stabilizer, the index $2nm$ as a `u32`, which `free_classes_of_index` checks against its own arrangement index limit (GRP-47 of `docs/design/group.md`), and `SearchBudget::DEFAULT`, through the private function that `render` uses (CLI-66). Stop after the first call that returns an error or whose result is not complete. For a regular construction, the private function checks the index with `IndexLimit::new` before it calls `free_classes_of_index`, so the arrangement index limit of 84 never applies (`DEC-139`), and with $2n$ above 64 it returns the error of `IndexLimit::new` for the index $2n$ without calling `free_classes_of_index`.
3. Print row 36 for $m = 1$ and, when it has a number, one row 37 for each arrangement of its list, in its order, with the edge pairing from `WholeFace::edges`. Then, for each $m$ from 2 whose call ran, print row 38 and, when it has a number, one row 37 for each arrangement of its list, in its order.

The value of row 36 is chosen in this order, as in `M24`:

| Case | Value |
| --- | --- |
| A step returns an error, such as a construction that is not a regular tiling, or an index $2n$ above the index limit | `none, ` followed by the `Display` text of the error, such as `none, cannot build whole face: the construction is not a regular tiling` |
| The search stopped at a limit | `none, the search stopped at a limit` |
| The search is complete | The number of arrangements with one picture in decimal, which is `0` when the search finds none |

The label of row 38 is `Arrangements with `, $m$ in decimal, and ` pictures`, such as `Arrangements with 2 pictures`. Its value is chosen as that of row 36, from the call at $2nm$: the number of arrangements with $m$ pictures in decimal, `0` when the call finds none, `none, the search stopped at a limit` when the call stopped, or `none, ` followed by the `Display` text of an error of the call. No input is known to give a call at $2nm$ an error after the call at $2n$ gave none, because the calls take the same presentation and words, and $2nm$ is within the index limit.

A count line comes for every number of pictures up to $M$, also when it is 0, so the listing states each number of pictures that it searched. After a row 36 or a row 38 that names a stopped search or an error, no further line comes. The search reaches its budget sooner at a larger index (Section 7.14 of `docs/design/group.md`), so a search after a stopped one would spend its budget and could claim nothing, and stopping is the simpler rule. A listing that stops at $m < M$ says nothing about the numbers of pictures above $m$.

The number of lines grows with the number of pictures that the listing searches. The design adds no limit on the length of a listing. A user shortens it with `--max-colours`, which bounds the number of pictures as above, and `--max-colours 1` gives the listing of one picture only.

The value of row 37 for an arrangement with one picture is the id, `, `, `turned only` when no neighbour is mirrored or `with mirrored neighbours` otherwise, `, edges `, and the edge pairing, each pair once in the order of its lower edge, separated by `, `. A pair of two edges is written `<i> and <j> turned` or `<i> and <j> mirrored`, and an edge paired with itself `<i> turned` or `<i> mirrored` (`DEC-119`). The edges are numbered counterclockwise around the face mask from edge 0, the edge that holds the half edge of the domain (Section 5.31 of `docs/design/render.md`). The face mask shows no numbers, so a user finds edge 0 by the black tick at its midpoint (`DEC-123`, Section 5.32 of `docs/design/render.md`). This form is that of `M24`, unchanged.

For an arrangement with $m \ge 2$ pictures, the value of row 37 is the id, `, `, $m$ in decimal, ` pictures, `, and then the rest of the form above, with each edge written as the letter of its picture followed by its number, such as `B2`. The edges are ordered by picture and then by number, so `A2` comes before `B0`, and each pair once in the order of its lower edge. `turned only` means that no meeting, between two pictures or within one, is mirrored. Each mask numbers its own edges counterclockwise from its own tick (`DEC-126`).

For example, the arrangement of the hexagons in which each edge meets the opposite edge of a turned copy prints this line, one of the lines of the snapshot of `escher info '*632' --wythoff 010`:

```text
Arrangement: 12g, turned only, edges 0 and 3 turned, 1 and 4 turned, 2 and 5 turned
```

That snapshot fixes the ids and the pairings of the seven arrangements of the hexagons with one picture, a number that Section 8.1 compares with a reading of a secondary catalogue, and after them the lines of two to five pictures.

A line with three pictures looks like this. It is the line that `escher` prints for `18a`, the first arrangement with three pictures of the triangles of `*732` with `100`, which the tests of Section 8.1 hold as a fixed value. A prototype outside the repository read the same pairing, by a rule of its own for the base cosets:

```text
Arrangement: 18a, 3 pictures, turned only, edges A0 and A1 turned, A2 and B0 turned, B1 and C0 turned, B2 turned, C1 and C2 turned
```

In this form, edge 2 of picture A meets edge 0 of picture B, so the two edges have one border colour on the masks of A and B, edge 2 of B meets itself by a half turn, and no meeting is mirrored.

CLI-68. With `--wythoff`, `info` MUST print row 36, the rows 37, and the rows 38 in the forms and the order of this section for every built domain, MUST print row 36 and the rows 37 of one picture byte for byte as `M24` printed them, MUST print every line of two or more pictures after them, MUST take every id, its order, and the edge pairing from `escher-group` and `escher-render` unchanged, MUST print no number for a search that stopped at a limit and no line after it, and every outcome of the arrangement lines MUST leave the status as it would be without them, because the ids are what a user passes to `--arrangement`, a script or a snapshot that reads the listing of `M24` must find its lines unchanged, a count or a pairing computed again in this crate could differ from the one that `render` draws, a stopped search shows nothing about its index, and a construction without arrangements is still a valid input, as CLI-47 states for the colouring lines.

CLI-70. With `--wythoff`, `info` MUST search the arrangements with $m$ pictures for $m$ from 1 to the largest number of pictures $M$ of this section, each with its own search at the index $2nm$ and `SearchBudget::DEFAULT` through the private function of step 2, MUST stop after the first call that returns an error or is not complete, and MUST NOT add an option, a limit, or a budget of its own, because the owner allowed more than three pictures (`DEC-129`), the existing option that bounds the colourings bounds the pictures in the same way and lets a user shorten the listing (`DEC-126`), the index limit of `escher-group` is the only limit that the existing design states for the index, each id is ranked within the call of its index alone (GRP-45 of `docs/design/group.md`), so one call cannot change the ids of another, and a call after a stopped one would spend its budget at a larger index and could claim nothing.

For a construction with several face types, which `WholeFace::new` accepts from `M26` on (`DEC-140`), `info` lists the multiples of the least index $L$ (`DEC-143`). The total number of pictures of the index $t$ is $m_t = \sum_k t / (2 n_k)$, over the face types of `WholeFace::face_types`. `info` takes these steps in place of steps 2 and 3 above:

1. Call `free_classes_of_index` at the index $L$, whatever the colour limit, through the private function of CLI-66 with `SearchBudget::DEFAULT`. An $L$ above the arrangement index limit of 84 gives the error of `escher-group` at once, before any search (GRP-47 of `docs/design/group.md`).
2. Then, for each further multiple $t$ of $L$ in increasing order with $t$ at most 84 and $m_t$ at most the colour limit of `--max-colours`, call it at $t$. Stop after the first call that returns an error or whose result is not complete. `info` takes the bound 84 of this step from its private constant `ARRANGEMENT_INDEX_LIMIT`, which repeats the arrangement index limit of `escher-group`, because that crate does not make the limit public.
3. Print row 36 for $L$, with its value chosen as above, and one row 37 for each arrangement of its list. Then, for each further index whose call ran, print row 38 with the label `Arrangements with <m> pictures` for its $m_t$, and one row 37 for each arrangement of its list.

The least index is always searched, because it is the first index that can hold an arrangement, as the index $2n$ of one picture is always searched for a regular construction. So `*732` with `111`, whose least index 84 gives 41 pictures, is searched at 84 under the default colour limit of 12, and a larger multiple appears only when its total of pictures is within the colour limit. Two indices never have the same total, so each row 38 names one index.

Every search of a construction with several face types runs under `SearchBudget::DEFAULT`, as every other search of the listing does. This holds also for the constructions whose least index is from 65 to 84, such as `*732` with `110` and `111`, and `*762`, `*(14)32`, `*(21)32`, and `*(42)32` with `111`. When the search at the least index completes, row 36 prints its count and one row 37 follows for each arrangement. When it stops at the budget, row 36 prints `none, the search stopped at a limit` and no further line follows, as for a regular construction, and the listing claims nothing about that index.

For an arrangement of a construction with several face types, the value of row 37 is the id, `, `, $m_t$ in decimal, ` pictures, `, the pictures of each type in the order of the corners, separated by `, `, each written as its first picture name, ` to ` and its last picture name when the type has more than one picture, ` of `, its number of sides in decimal, and ` sides`, then `, `, `turned only` or `with mirrored neighbours`, `, edges `, and the edge pairing in the form of several pictures above, with each edge written as its picture name followed by its number, such as `1B2`, and the edges ordered by the number of the picture over all types and then by the number of the edge. This is the line that `escher info '*442' --wythoff 111` prints for `8b`, the arrangement of the translations of `*442` with `111` that Section 5.36 of `docs/design/render.md` works by hand:

```text
Arrangement: 8b, 4 pictures, 0A of 8 sides, 1A of 8 sides, 2A to 2B of 4 sides, turned only, edges 0A0 and 1A1 turned, 0A1 and 2A0 turned, 0A2 and 1A3 turned, 0A3 and 2B2 turned, 0A4 and 1A5 turned, 0A5 and 2A2 turned, 0A6 and 1A7 turned, 0A7 and 2B0 turned, 1A0 and 2A1 turned, 1A2 and 2B3 turned, 1A4 and 2A3 turned, 1A6 and 2B1 turned
```

Row 37 names the pictures of each type by range, such as `0A to 0F of 14 sides`, and `render-masks` writes the mask of each picture to a file whose name ends in its picture name (CLI-76), so a user finds the mask of `0C` in `tile-0C.png` with `-o tile.png`, and the help of `render-masks --wythoff` gives this rule (Section 4.2).

A construction whose least index is above 84 prints row 36 with the error of the limit, such as `Arrangements: none, cannot search subgroups: the arrangement index 140 is not between 1 and 84` for `*752` with `111`, and no further line. A construction whose least index holds no arrangement prints `Arrangements: 0`, and the further indices within the limits follow, so a user sees each index that was searched.

CLI-74. With `--wythoff` on a construction with several face types, `info` MUST search the least index and then each further multiple of it within the arrangement index limit whose total of pictures is at most the colour limit, one search for each index through the private function of CLI-66, MUST stop after the first search that returns an error or is not complete, MUST print row 36 for the least index and a row 38 for each further index with the forms of this section, MUST take every id, its order, the face types, and the edge pairing from `escher-group` and `escher-render` unchanged, and MUST leave every listing of a regular construction byte for byte as `M25` printed it, because the owner chose one arrangement for every kind of polygon (`DEC-137`), the least index is the first that can hold one, the total of pictures is what a user paints and passes to `--colours`, and the regular listings are fixed by their snapshots (RND-87 of `docs/design/render.md`).

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
| The colouring is `parity` and the domain has no valid colouring with 2 colours (CLI-35) | 6 | `error: cannot render 1: the colouring parity does not exist, because the signature has no colouring with 2 colours` |
| A colouring id with more than 12 colours and no `--colours` (CLI-44) | 4 | `error: cannot render *732: the colouring 15a needs 15 colours, more than the 12 default colours, so give 15 colours with --colours`, with the id and, twice, its colour count |
| `--colours` has the wrong number of items | 4 | `error: cannot render *732: the colouring parity needs 2 colours, and 3 were given`, with the colouring, which is `one`, `parity`, or the id, the number it needs, and the number of items. The words are `1 colour` and `1 was given` for the number 1 |
| An item of `--colours` is neither a colour nor an image item with a path | 4 | `error: cannot render *732: colour 2 of --colours is not of the form #rgb, #rrggbb, or image:FILE`, with the position of the first such item, counted from 1 |
| The file of an image item cannot be opened (CLI-60) | 4 | `error: cannot render *732: the image of colour 2 of --colours cannot be opened`, then `  caused by: ` and the `Display` text of the `std::io::Error`, with the position of the item, counted from 1 |
| `Image::decode_png` returns an error for the file of an image item, including an image over the limits of `escher-render` (CLI-60) | 4 | `error: cannot render *732: the image of colour 2 of --colours cannot be used`, then `  caused by: ` and the `Display` text of the error of `escher-render`, such as `cannot decode image: the data is not a valid PNG image`, with the position of the item, counted from 1. When the error holds the error of `png` as its source, one further line gives the `Display` text of that error, such as `Invalid PNG signature.`, in which every character outside printable ASCII, other than the infinity sign, is replaced with `?`, as the echo line does (CLI-9), because this crate does not rely on what the text of a `png` error holds. No line follows for a source of the `png` error. An image over a limit, such as one of 16385 by 1 pixels, gives no further line |
| The file of an image item has 16 bits per channel (CLI-60) | 4 | `error: cannot render *732: the image of colour 2 of --colours cannot be used`, then `  caused by: cannot decode image: the image has 16 bits per channel, and at most 8 are supported`, with the position of the item, counted from 1, and no further line |
| `escher-render` rejects an option, including a rotation angle, a light direction, or a border width in either unit | 5 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render` |
| The position of `--centre` is not below the number of corners of the domain (CLI-72) | 5 | `error: cannot render *732`, then `  caused by: cannot build renderer: the corner 3 is not below the 3 corners of the domain`, with the position and the number of corners, which is the number of angles of the line `Corner angles in degrees` of `info` (Section 7.2 of `docs/design/render.md`) |
| `--centre` is given for a domain without a corner, which is the domain of `1` (CLI-72) | 5 | `error: cannot render 1`, then `  caused by: cannot build renderer: the domain has no corner` |
| `side_presentation()` returns an error for a colouring id | 6 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-domain` |
| The colouring id names no colouring of the domain (CLI-43) | 6 | `error: cannot render *642: the colouring 3a does not exist, because the signature has no colouring with 3 colours`, with the id, the number of valid colourings with its colour count, and that count. The words are `1 colouring with` and `no colouring with` for the numbers 1 and 0, and `1 colour` for the colour count 1 |
| The search for a colouring id, for `parity` or the default colouring on a domain that is not all-reversing, or for `parity` with an image item, reaches a search limit before it finds the colouring (CLI-34, CLI-43) | 4 for the default colouring, 6 otherwise | `error: cannot render *(12)(12)6: the search for the colouring 12a stopped at a limit before it found it`, with the id, or `parity` for `parity` and for the default colouring |
| `colourings_of_index` returns an error | 6 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-group` |
| With an image item, the search of `ColouringSet::All` with the colour count 1 finds no colouring (CLI-61) | 6 | `error: cannot render *732: the colouring one does not exist, because the signature has no colouring with 1 colour`, with `one` or `1a` as given, or, when the search stops at a limit, `error: cannot render *732: the search for the colouring one stopped at a limit before it found it`. No input reaches this row, because the colour count 1 has exactly one colouring |
| `escher-render` rejects the coset fill of a colouring id | 6 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render`. No input reaches this row, because step 6 takes the colouring from the side presentation of the same domain, and step 4 checks the number of colours first |
| `--wythoff` and `--colouring` are both given (CLI-62) | 4 | `error: cannot render *732: --wythoff cannot be combined with --colouring` |
| `Wythoff::new` refuses the domain or the marks, or returns `Numerical` (CLI-62) | 4 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render`, such as `cannot build Wythoff construction: the marks give no face` for `--wythoff 000`. A domain that is not a reflection triangle, such as that of `732`, gives `cannot build Wythoff construction: the domain is not a reflection triangle`. No input is known to reach `Numerical`, whose text is `cannot build Wythoff construction: the construction is numerically degenerate` (Section 5.24 of `docs/design/render.md`) |
| With `--wythoff`, `--colours` has the wrong number of items (CLI-63) | 4 | `error: cannot render *732: the Wythoff construction needs 3 colours, and 2 were given`, with the number of face types and the number of items. The words are `1 colour` and `1 was given` for the number 1 |
| With `--wythoff`, an item of `--colours` is not a colour, an image item included (CLI-63) | 4 | `error: cannot render *732: colour 2 of --colours is not of the form #rgb or #rrggbb`, with the position of the first such item, counted from 1 |
| `--arrangement` is given without `--wythoff` (CLI-65) | 4 | `error: cannot render *632: --arrangement needs --wythoff` |
| `WholeFace::new` refuses the construction (CLI-65) | 4 | `error: cannot render *522`, then `  caused by: ` and the `Display` text of the error of `escher-render`, such as `cannot build whole face: the construction is not a regular tiling` for `--wythoff 010`, a construction with one face type that is not a regular tiling |
| With `--arrangement`, `--colours` has another number of items than the id has pictures (CLI-67) | 4 | `error: cannot render *632: the arrangement 12a needs 1 colour, and 2 were given`, with the id, its number of pictures, and the number of items, `colours` for a number above 1, and `1 was given` for the number 1, such as `error: cannot render *732: the arrangement 18a needs 3 colours, and 1 was given`. With several face types, the number of pictures is the total of every type, such as `error: cannot render *442: the arrangement 8a needs 4 colours, and 1 was given` for `--wythoff 111` |
| With `--arrangement`, an item of `--colours` is neither a colour nor an image item with a path, or its image cannot be opened or used (CLI-67) | 4 | The diagnostics of the rows of a bad item, an image that cannot be opened, and an image that cannot be used, above |
| On a regular construction, the number of the `--arrangement` id is from 65 to 84 (CLI-75) | 4 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `IndexLimit::new` for the number of the id, such as `cannot search subgroups: the index limit 70 is not between 1 and 64` for `70a` |
| The index $2n$ of the arrangements is above the index limit (CLI-66) | 6 | `error: cannot render *(33)32`, then `  caused by: ` and the `Display` text of the error of `escher-group`, such as `cannot search subgroups: the index limit 66 is not between 1 and 64` |
| With several face types, the index of the search is above the arrangement index limit (CLI-75) | 6 | `error: cannot render *752`, then `  caused by: ` and the `Display` text of the error of `escher-group`, such as `cannot search subgroups: the arrangement index 140 is not between 1 and 84` |
| With several face types, the complete search finds no arrangement, or the id names none (CLI-75) | 6 | The texts of the two rows of several pictures below with the total number of pictures of the index searched, such as `error: cannot render *332: the tiling has no arrangement with 7 pictures` for `--wythoff 111 --arrangement 12a`, and `error: cannot render *442: the arrangement 8d does not exist, because the tiling has 3 arrangements with 4 pictures` for `--wythoff 111 --arrangement 8d` |
| With `--arrangement` and without `--colours`, the arrangement has more than 12 pictures (CLI-75) | 4 | `error: cannot render *732: the arrangement 84a needs 41 colours, more than the 12 default colours, so give 41 colours with --colours`, with the id and, twice, its number of pictures, in the form of the row of CLI-44 |
| The complete search finds no arrangement with one picture (CLI-66) | 6 | `error: cannot render *732: the tiling has no arrangement that puts one picture on every polygon` |
| The complete search finds no arrangement with $m \ge 2$ pictures (CLI-66) | 6 | `error: cannot render *732: the tiling has no arrangement with 2 pictures`, with the number of pictures of the id, such as for `12a` of `*732` with `100` |
| The arrangement id names no arrangement of the tiling (CLI-66) | 6 | `error: cannot render *632: the arrangement 12h does not exist, because the tiling has 7 arrangements with 1 picture`, with the id, the number of arrangements with the number of pictures of the id, and that number of pictures, which is 1 for an id whose number is not a multiple of $2n$. The words are `1 arrangement` for the number 1 and `1 picture` for one picture, as the colouring text writes `1 colouring` and `1 colour`, such as `error: cannot render *632: the arrangement 24zz does not exist, because the tiling has 11 arrangements with 2 pictures` (`DEC-128`) |
| The search for an arrangement stops at a limit (CLI-66) | 6 | `error: cannot render *632: the search for the arrangement 12a stopped at a limit before it found it`, with the id |
| `free_classes_of_index` returns another error, or `escher-render` rejects the whole-face fill | 6 | `error: cannot render *632`, then `  caused by: ` and the `Display` text of the error. No input reaches this row, because the words and the arrangement come from the same whole face |
| `escher-render` rejects the Wythoff fill | 6 or 8 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render`. No input reaches this row, because step 4 checks the number of colours and the fill is built from the domain that is rendered |
| The thread pool cannot be built | 7 | `error: cannot render *732`, then `  caused by: cannot start the thread pool`, then `  caused by: ` and the `Display` text of the error of `rayon`. That text appears once, because the diagnostic takes it without the source of the error |
| With an image item, `Renderer::new` refuses a domain without a mask polygon, such as the lune of `*22` (RND-63 of `docs/design/render.md`) | 8 | `error: cannot render *22`, then `  caused by: cannot build renderer: the domain has no mask polygon` |
| `escher-render` returns an error while rendering or encoding | 8, 9 | `error: cannot render *732`, then `  caused by: ` and the `Display` text of the error of `escher-render` |
| The output file cannot be written | 10 | `error: cannot write the output file`, then `  caused by: ` and the `Display` text of the `std::io::Error` |

Section 7.2 of `docs/design/render.md` fixes the `Display` text of each error of `escher-render`, and Section 10.2 of `docs/design/domain.md` that of `escher-domain`. No render error has an echo line or a caret line.

The row of `UnsupportedSignature` meets the requirement of Section 5.8 of `docs/design/architecture.md` that `render` names the signature and says that rendering it is not supported. It applies to every valid signature without a domain, including every general non-compact signature (`DEC-19`). The row of any other error kind covers a corner order over the limit of `escher-domain`, a signature with more than eight orders (DOM-38), a numerically degenerate construction, a polygon or a double of a polygon that fails `verify()` (DOM-44), a wallpaper domain that fails `verify()` (DOM-54), a spherical group domain that fails `verify()` (DOM-60), and the limit of 50 of `2*n` (DOM-63), which are not a lack of support. `escher-domain` checks the support of the signature before the settings, so a setting of a signature without a domain gives the row of `UnsupportedSignature` (DOM-18 in Section 6.6 of `docs/design/domain.md`), and not the row of `UnknownParameter`.

The rows of `UnknownParameter` and `ParameterOutOfRange` have texts of this crate, and not the `Display` text of the error of `escher-domain`, because that text names neither the rejected setting nor the parameters that the signature takes. A user who mistypes a name, or sets a parameter of another signature, sees what to type instead. The value is printed with the `Display` text of `f64`, so the value `9.0` prints as `9`, and a NaN or infinite value as `NaN`, `inf`, or `-inf`. The ends of the range are printed in the same way.

A domain that `build_with` refuses with the error kind `Numerical` gives the row of any other error kind, with the `caused by` line of Section 10.2 of `docs/design/domain.md`: `cannot build fundamental domain: the shape is too extreme to build accurately` for a shape of a reflection polygon, `cannot build fundamental domain: the side pairings of the domain cannot be stored accurately enough` for a double of a polygon, `cannot build fundamental domain: the wallpaper domain fails its verification` for a wallpaper domain, and `cannot build fundamental domain: the spherical domain fails its verification` for a spherical group domain. This crate adds no text of its own for these refusals or for the limit of `2*n`.

CLI-26. A diagnostic of `render` or `render-masks` MUST NOT repeat the value of an option, the output path, the path of an image item, or the name of a mask file, because each is untrusted and may hold control characters, while the canonical signature, the option names, and the numbers that the diagnostics hold are printable ASCII. A bad colour and an image item are therefore named by their option and their position, a failed write by the words `the output file`, and a failed write of `render-masks` by the colour of its mask. There are three exceptions:

- A colouring id, which CLI-49 governs, and an arrangement id, which it governs in the same way, through the `Display` text of the parsed `ColouringId`.
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

This example shows the colouring `parity` for a signature whose domain has no valid colouring with 2 colours:

```sh
escher render 1 --colouring parity -o one.png
```

```text
error: cannot render 1: the colouring parity does not exist, because the signature has no colouring with 2 colours
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

This example shows a colouring id that names no colouring of the signature. `*642` has no valid colouring with 3 colours:

```sh
escher render '*642' --colouring 3a -o tiling.png
```

```text
error: cannot render *642: the colouring 3a does not exist, because the signature has no colouring with 3 colours
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

### 6.10. Mask Errors

A signature error of `render-masks` gives the diagnostic of Sections 6.1 to 6.3. A failure of steps 3 to 6 of Section 4.8 gives the diagnostic of the row of Section 6.8 for the same failure, with `cannot render the masks of` in place of `cannot render`, such as `error: cannot render the masks of *642: the colouring 3a does not exist, because the signature has no colouring with 3 colours`. The rows of Section 6.8 for colours, image items, `--wythoff` with the number of colours or a colour item, the thread pool, and the output file do not apply, and its rows for `--wythoff` with `--colouring`, `Wythoff::new`, `--arrangement` without `--wythoff`, `WholeFace::new`, the number of an id on a regular construction, and the resolution of an arrangement apply, such as `error: cannot render the masks of *732: the tiling has no arrangement that puts one picture on every polygon` for `escher render-masks '*732' --wythoff 100 --arrangement 6a -o face.png`. These rows complete the table:

| Case | Step | Diagnostic |
| --- | --- | --- |
| The value of `--output` has no file stem (CLI-58), for the polygon masks or for the face masks of an arrangement with several pictures (CLI-71, CLI-76) | 2, which runs after step 4 with `--wythoff` | `error: cannot render the masks of *732: the output path names no file` |
| `Masks::new` refuses a domain without a mask polygon (RND-63 of `docs/design/render.md`) | 7 | `error: cannot render the masks of *22`, then `  caused by: cannot build masks: the domain has no mask polygon` |
| `Masks::new` returns any other error | 7 | `error: cannot render the masks of *732`, then `  caused by: ` and the `Display` text of the error of `escher-render`. No input reaches this row, because step 6 takes the colouring from the side presentation of the same domain |
| `Masks::render` or `encode_png` returns an error | 8 | `error: cannot render the masks of *732`, then `  caused by: ` and the `Display` text of the error of `escher-render`. No input reaches this row, because step 5 checks the size and the colour is below the count |
| `--wythoff` is given without `--arrangement` (CLI-69) | 4 | `error: cannot render the masks of *632: --wythoff needs --arrangement` |
| `WholeFace::render_mask` or `encode_png` returns an error for a face mask | 7, 8 | `error: cannot render the masks of *632`, then `  caused by: ` and the `Display` text of the error of `escher-render`. No input reaches this row, because step 5 checks the size and the arrangement comes from the same whole face |
| The face mask cannot be written (CLI-69) | 8 | `error: cannot write the face mask`, then `  caused by: ` and the `Display` text of the `std::io::Error` |
| The face mask of a picture of an arrangement with several pictures cannot be written (CLI-71, CLI-76) | 8 | `error: cannot write the face mask of picture B`, then `  caused by: ` and the `Display` text of the `std::io::Error`, with the letter of the picture, or with several face types its picture name, such as `picture 1B` |
| A mask file cannot be written (CLI-59) | 8 | `error: cannot write the mask of colour 2`, then `  caused by: ` and the `Display` text of the `std::io::Error`, with the colour of the mask |

Each of these diagnostics gives the status `Failure`. No render error of `render-masks` has an echo line or a caret line.

This example shows an output path in a directory that does not exist, on Linux:

```sh
escher render-masks '*732' -o /tmp/missing/mask.png
```

```text
error: cannot write the mask of colour 0
  caused by: No such file or directory (os error 2)
```

## 7. Streams and Exit Status

`run` returns these statuses. Section 3.3 gives their exit statuses, and Sections 4.7, 4.8, 6.5, 6.6, 6.7, 6.8, 6.9, and 6.10 give the streams.

| Status | When |
| --- | --- |
| `Success` | `info` without `--check` wrote its text, whether or not the signature has a domain. `info --check` wrote its text and every check passed. With `--presentation`, either one also needs that no order line reported an index other than the group order. The colouring lines and the arrangement lines never change the status, including a search that stops at a limit (CLI-47, CLI-68). `render` wrote the output file, and with `--stats` the statistics lines. `render-masks` wrote every mask file, or every face mask of the arrangement. `--help` or `--version` wrote the help or the version text |
| `Failure` | The argument is a signature error of any kind, including a bad orbifold (`DEC-24`). A write to standard output failed (Section 6.6). With `--check`, a check failed or the signature has no domain (Section 6.7). With `--presentation`, an order line reported an index other than the group order (Section 6.9), or a relator had no text (Section 5.7). `render` met a render error (Section 6.8), including a signature without a domain, a bad setting, the colouring `parity` for a domain without a valid colouring with 2 colours, a colouring id that names no colouring, a search for an id, for `parity`, or for the default colouring that stops at a limit, an id with more than 12 colours without `--colours`, an image item that cannot be opened or decoded, a construction that is not a regular tiling with `--arrangement`, and an arrangement that does not exist or whose search stops at a limit. `render-masks` met an error of Section 6.10 |
| `Usage` | `clap` reports a usage error (Section 6.5), including a malformed colouring id or arrangement id and a value of `--max-colours` that Section 4.3 rejects |

CLI-11. `run` MUST return `Failure` for every signature error, including a bad orbifold, because a caller that gets 0 for an invalid signature treats the missing information as valid (`DEC-24`). A caller that needs the error kind uses `escher-signature` directly.

CLI-19. `info --check` MUST return `Failure` when a check fails and when the signature has no domain, and MUST return `Success` only when every check passed, because a script that runs `--check` reads the exit status as the verdict, and a status of 0 for a signature with nothing to check reads as verified. Section 5.8 of `docs/design/architecture.md` gives the same reason for `render`.

## 8. Testing

### 8.1. Tests

The tests live in one integration-test binary, `crates/escher-cli/tests/cli/main.rs`, with the modules `info`, `diagnostics`, `usage`, `render`, `masks`, `properties`, `process`, and `common`, and the snapshots under `crates/escher-cli/tests/cli/snapshots/` (rule TEST-10 of `docs/rust-coding-standards.md`). `main.rs` has a `//!` line, which the lint `missing_docs` requires. It declares `common`, `info`, `render`, `masks`, `properties`, and `process` under `#[cfg(test)]`, because their helpers call `expect`, which the Clippy option `allow-expect-in-tests` accepts only in code compiled for tests. Every test outside `main.rs` and `process` calls `run` with `Vec<u8>` streams, or with a test writer that fails. The two tests in `main.rs` test `Status` alone. The one test in `process` starts the binary.

A test that can write the output file gets a path from a helper in `common`, in a directory of its own, `CARGO_TARGET_TMPDIR/escher-cli/<test>`. The helper empties that directory first, so a file from an earlier run cannot satisfy a test. The render usage snapshots and the render status tests fail before any file access and pass the fixed path `CARGO_TARGET_TMPDIR/escher-cli/never-written.png`, which a constant in `common` holds and no test creates.

The tests have these layers:

| Layer | Form | What it catches |
| --- | --- | --- |
| Snapshot tests | `insta` tests in the modules `info`, `diagnostics`, `usage`, `render`, and `masks` that call `run` with in-memory streams and compare its status and both streams with a committed snapshot | Any change to the information text, the help, version, and usage text, a diagnostic, or the statistics lines, for each kind of domain and each form of each line of Sections 4.7 and 5 |
| Plain tests | Tests that call `run` and assert the status, the start or the end of a stream, and the files that `render` and `render-masks` write | A wrong status, a failed write with a writer that fails, a PNG header with the wrong size or chunks, read by hand, a file that depends on the number of threads (CLI-23), the aliases of CLI-48, the default colouring of CLI-34, an existing output file that a failure changes (CLI-21), and a use of the global pool of `rayon` (CLI-22), which relies on nextest running each test in a process of its own |
| Unit tests | Tests in `args.rs`, `info.rs`, `render.rs`, and `masks.rs` of the private parsers, formatters, and functions | A parser of an option value that accepts or rejects the wrong text, a formatter of a line that writes the wrong form, a wrong default colour, and the paths that no known input reaches, such as a failed check, a different index, an internal error of a presentation, and a search that stops at a limit |
| Property test | The test of Section 8.3 in `properties`, which runs `render` on generated valid signatures and a fixed list, and compares each outcome with an oracle of its own | A signature of the set of Section 3.2 of `docs/design/poc-plan.md` that `render` refuses, a signature outside it that `render` draws, a refusal with the wrong cause, and a limit off by one |
| Smoke test | The one test in `process`, which starts the binary that Cargo names in `CARGO_BIN_EXE_escher` | A binary that does not call `run` or does not turn its status into the exit status |
| Doctests | The crate-level examples | An example that no longer compiles or runs |

The argument that is not UTF-8 is built with `std::os::unix::ffi::OsStringExt`, and the symbolic link with `std::os::unix::fs::symlink`, so those tests compile only on Unix, which covers Linux, the supported platform.

No known input makes a check fail, so the failed-check path is tested through private functions, as Section 12.1 of `docs/design/signature.md` does for its `Overflow` texts. The unit tests cover the formatter of one check, the check lines, the `Verification` line, and the count of failed checks built from the rows, the writing of the text before the verdict is returned, and the diagnostic function.

The colouring lines of a search that stops at a limit are tested by unit tests with a small `SearchBudget`, not by a snapshot, because a search that reaches the default step budget is slow even in a release build, and a search that stops only with `--colourings` can print many `Colouring` lines. The diagnostic of `render` for a search that stops at a limit has one snapshot, of `escher render '*(12)(12)6' --colouring 12a`, whose search for `12a` runs until the default step budget stops it. Unit tests with a small budget cover the same branch of the resolution for an id, for `parity`, and for the default colouring.

The plain tests that an input error of steps 4 and 5 comes before the search use `*(12)(12)6` with a colouring id. Because the search for `12a` stops at a limit, a search that ran first would replace the input error with the diagnostic of that limit.

The plain tests of the colourings cover these cases: the default colouring and `parity` on an all-reversing domain, which give the bytes of `2a`, on a domain that is not all-reversing and has a valid colouring with 2 colours, on a domain without one, such as that of `1`, and with a search for 2 colours that stops at a limit, through the private function with a small budget. They also cover `1a` on a domain other than that of `1`, which gives the bytes of `one`, and an id that exists under `ColouringSet::All` but not under `ColouringSet::ValidClasses`, which gives the diagnostic of CLI-43. The snapshots of `info` hold the counts and listings of `ColouringSet::ValidClasses`.

For `--model`, the snapshots of `render --help` and `render -h` show the option with its help, its default, and its possible values, and one usage snapshot shows the usage error of `--model poincare`. The plain tests cover these cases: `--model disk` gives the bytes of the same command without the option, `--model half-plane`, `--model band`, and `--model klein` on `*732` each write a PNG file of the size of `--size` whose bytes differ from those of the disk, and `--model band` on `*632` and on `*532` gives the bytes of the same command without the option (CLI-55). A unit test in `args` checks that each value maps to the variant of the same name.

For `--centre`, the snapshots of `render --help` and `render -h` show the option with its long and its short help, and the snapshots of the help of `render-masks` stay as they are. Usage snapshots show the usage errors of `--centre x` on `render` and of `--centre 0` on `render-masks`, both on `*732` (CLI-73). The diagnostic snapshots show `escher render '*732' --centre 3` and `escher render 1 --centre 0`, each with the status `Failure` and no file written. The plain tests cover these cases: `--centre 0`, `--centre 1`, and `--centre 2` on `*732` each write a PNG file of the size of `--size`, and the three files and the file without the option differ pairwise. `--centre 1` on `732`, `--centre 0` on `*632`, and `--centre 2` on `*532` with `--rotation 20,35,0` each write a PNG file. On `*732`, `--centre 0` combines with `--model band`, with `--wythoff 010`, and with `--wythoff 010 --arrangement 14a` and an image item, each writing a PNG file. `--centre 3` on `*(12)(12)6` with the colouring id `12a` fails at step 5 with the diagnostic of the position and not that of the search, which shows that the position is checked before the search of step 6. `escher info '*732' --centre 0` returns the status `Usage` (CLI-73). A unit test in `args` checks the line of `*732` in `CENTRE_LONG_HELP` against the output of `info` (Section 4.2). Every other snapshot and plain test stays as it is, which shows that the output without `--centre` does not change.

For `--border-unit`, the snapshots of `render --help` and `render -h` show the option with its help, its default, and its possible values, and the help of `--border-width` with the value name `WIDTH`. One usage snapshot shows the usage error of `--border-unit px`, and one diagnostic snapshot shows `escher render '*732' --border-unit intrinsic --border-width 1025`. The plain tests cover these cases: `--border-unit pixels` gives the bytes of the same command without the option, and `--border-unit intrinsic --border-width 0.05` on `*732`, `*632`, and `*532` each writes a PNG file of the size of `--size` whose bytes differ from those of the same command without `--border-unit`, and `--border-unit intrinsic` without `--border-width` gives the bytes of the same command with `--border-width 0.05` (CLI-56). A unit test in `args` checks that each value maps to the variant of the same name.

For the regular tiling line, the snapshots of `info` cover `*732`, `732`, `*442`, `*632`, `*532`, `*237` as a spelling that is not canonical, and `*(101)32` as a signature that gets the line without a domain, and, without the line, `*333`, `*2222`, and `*722`, whose second order is 2. An input that already has a snapshot keeps it, and the snapshot changes where the input gets the line (CLI-7). Each input follows the colour limit of its kind of domain (CLI-50, CLI-52, CLI-54). The unit tests of the private function of Section 5.9 cover each condition of that section on valid signatures: `*pq2` and `pq2` with $p > q \ge 3$ and with $p = q$, $q = 2$ with $p \ge 3$ and with $p = 2$, three orders without a 2, four orders, a handle, a crosscap, a cone point beside a boundary, two boundaries, an infinite order among the three, and the largest order, `u32::MAX`, which gives its digits in decimal (CLI-57).

For `render-masks`, the snapshots of `render-masks --help` and `render-masks -h` show its options, one usage snapshot shows a missing `--output`, and the diagnostic snapshots show an output path with no file stem, a colouring id that does not exist, a domain without a mask polygon, `*22`, and a mask file that cannot be written, in a directory that does not exist. The plain tests cover these cases:

- `-o mask.png` with the colouring `4a` of `*642` writes exactly `mask-0.png` to `mask-3.png` in the directory of the test, each a PNG file of the size of `--size`, and nothing else.
- `-o masks` writes files without an extension.
- An existing file with the name of a mask is replaced.
- A failed check writes no file and leaves an existing mask file as it was (CLI-59).
- A write that fails at colour 2, because a directory has the name of its mask file, leaves the files of colours 0 and 1 and writes none of colour 3.
- The default colouring, on an all-reversing domain, on a rotation domain with a parity colouring, and on one without, and `one` and `1a`, each write one file for each of their colours.
- `--colouring parity` on `732`, which has no valid colouring with 2 colours, gives the diagnostic of Section 6.10 for that case and writes no file.
- The bytes of each mask equal those that `Masks::render` and `encode_png` give for the same domain, colouring, and colour.

One plain test runs the whole flow on the colouring `3a` of `*632`. It writes the masks with `render-masks` and checks their bytes as above. It then paints the white pixels of each mask in a colour of its own, writes each painted mask with `encode_png`, and passes the files to `render` as image items, with `--samples 1` and `--border-width 0`. It asserts that the file equals the render of `escher-render` with `Paint::Image` of the decoded files, and that this render equals the render with the three colours as solid paints at every pixel whose 5 by 5 neighbourhood in the solid render has one colour, which is more than a quarter of the pixels, as the test also asserts.

Unit tests in `masks.rs` check the names of CLI-58 for a path with an extension, with and without a directory, a stem without an extension, a stem with two dots, a stem that is not UTF-8, and `/`, `..`, and `/tmp/..`, which have no stem.

For the image items, the plain tests write their image files with `encode_png` of `escher-render` in the directory of the test, so the crate needs no `png` dependency. They cover these cases:

- `--colours` with one image item and one colour writes a file.
- `--colouring one` and `--colouring parity` on the all-reversing domain of `*632`, with an image item in one colour, give the bytes of the same command with that colour as a solid colour (CLI-61).
- `--colouring 2a` and `--colouring parity` with the same image items give the same bytes on `*632` and `442`.
- An image file that is not a PNG image, does not exist, or holds an image of 16385 by 1 pixels leaves an existing output file as it was and creates no output file.
- `--colouring parity` with two image items on `732`, which has no valid colouring with 2 colours, gives the diagnostic of Section 6.8 for that case and writes no file (CLI-35).
- An image item that cannot be opened is reported before the search for the id `12a` of `*(12)(12)6`, which stops at a limit.

For the equalities of CLI-61, a unit test in `render.rs` draws the coset fill of `1a` of `ColouringSet::All` with one solid colour on `*732`, `732`, and `*2222`, through the private function that draws a colouring with the coset fill, and asserts the bytes of `one` with the same colour. The plain test of the colourings in which `parity` on an all-reversing domain gives the bytes of `2a`, with solid colours, pins the equality of `2a` and `parity`. The diagnostic snapshots show an image item without a path, an image file that does not exist, a file that is not a PNG image, an image of 16385 by 1 pixels, and an image item on the lune of `*22`, which has no mask polygon. Unit tests in `render.rs` also check that the colouring `parity` of the coset fill on `442`, with a small search budget, gives the diagnostic of a search that stops at a limit, that `image:` with no path is not an image item, that a path is taken with no expansion, so that `image:~/a.png` names the file `~/a.png` relative to the working directory, and, with a reader that fails, that the line of the decoder error in the decoding diagnostic replaces each character outside printable ASCII, other than the infinity sign.

For `--wythoff`, the snapshots of `render --help` and `render -h` show the option with its help, and usage snapshots show the usage errors of `--wythoff 01` and `--wythoff 0a1`. The diagnostic snapshots show `escher render '*732' --wythoff 000`, `escher render 732 --wythoff 010`, `escher render '*522' --wythoff 100`, `escher render '*732' --wythoff 010 --colouring parity`, `escher render '*732' --wythoff 111 --colours '#000,#fff'`, and `escher render '*732' --wythoff 010 --colours 'image:fill.png'`, each with the status `Failure` and no file written. The plain tests cover these cases:

- Each of the seven markings of `*732` writes a PNG file of the size of `--size`, the seven files differ pairwise, and `010` differs from the image without `--wythoff`.
- `--wythoff 111` on `*632` and `--wythoff 110` on `*532` each write a PNG file of the size of `--size` whose bytes differ from those of the same command without `--wythoff`.
- `--wythoff 110` on `*732` with three colour items, in each of two orders, gives the bytes of the Wythoff fill of `escher-render` with the same colours in the same order, and the two orders give different bytes.
- On `*732`, `--wythoff 010` without `--colours` gives the bytes of `--colours '#e9c46a'`, and `--wythoff 111` without `--colours` gives the bytes of `--colours '#e9c46a,#2a9d8f,#913671'`.

A unit test in `args` checks the parser on `000`, `111`, `010`, and `110`, which is not a palindrome and so pins the order of the digits. A second one checks that it rejects `01`, `0101`, `0a1`, `0,1,0`, the empty value, `010` with a space before or after it, `012`, and a value with a digit outside ASCII. Unit tests in `render.rs` check the words of the wrong number of colours for 1 and 2 face types, that the colours are read in order, and that an image item in position 2 is not a colour (CLI-62, CLI-63).

For `--arrangement` and `info --wythoff`, the snapshots of `render --help`, `render -h`, `render-masks --help`, `render-masks -h`, and `info --help` show the options with their help, and a usage snapshot shows the usage error of `--arrangement 12A`. The snapshot of `escher info '*632' --wythoff 010` is the golden listing of the arrangements of the hexagons, with their ids and edge pairings, with one picture and with two to five pictures. Further `info` snapshots show `*632` with `100`, `*542` and `*732` with `100`, each of which prints `Arrangements: 0` and then the lines of more pictures, `*522` with `010`, which prints the refusal of a construction that is not a regular tiling, and `*(33)32` with `010`, which prints the error of the index limit. The diagnostic snapshots show `escher render '*632' --arrangement 12a`, `escher render '*522' --wythoff 010 --arrangement 10a`, `escher render '*632' --wythoff 010 --arrangement 12h`, `escher render '*(33)32' --wythoff 010 --arrangement 12a`, `escher render '*632' --wythoff 010 --arrangement 12a --colours '#000,#fff'`, `escher render '*732' --wythoff 100 --arrangement 6a`, which gives the diagnostic of a complete search that finds no arrangement, and `escher render-masks '*632' --wythoff 010 -o face.png`, each with the status `Failure` and no file written. For several pictures, the diagnostic snapshots show `escher render '*732' --wythoff 100 --arrangement 18a --colours '#000'`, which gives the wrong number of colours for three pictures, `escher render '*632' --wythoff 010 --arrangement 24zz`, which names no arrangement with two pictures, and `escher render-masks '*732' --wythoff 100 --arrangement 18a -o /`, whose path names no file, `render` and `render-masks` with `*732`, `100`, and `12a`, which give the diagnostic that the tiling has no arrangement with 2 pictures and write no file, each with the status `Failure` and no file written. Of the snapshots of `M24`, the listings of `*632` with `010` and `100` and of `*542` with `100` change only by the lines of two or more pictures after their last line, the snapshot of `escher render '*632' --wythoff 010 --arrangement 12h` changes by ` with 1 picture` at the end of its diagnostic, the help snapshots change by the help texts of Section 4.2, and every other snapshot of `M24` stays as it is. The listings that gain lines run one search for each number of pictures up to the first that stops, each within the step budget, so their time is bounded by the budgets of at most 10 searches, and this design states no measurement of it. The id `12a` of the snapshot of `*(33)32` parses, so the command reaches the error of the index limit $2n = 66$, which step 6 meets before it looks up the id (CLI-66). That the search finds no arrangement with one picture for `*732`, `*542`, and `*532` with `100` is the result of the search, whose filter rests on `UNV-41` to `UNV-43`. No source states it, because [YUNCKEN2003] treats orientation-preserving groups only (Section 5.9.2 of `docs/research/colourings.md`). The plain tests cover these cases:

- With `--max-colours 1`, the arrangement lines that `info` prints for every construction of the whole-face set of Section 8.1 of `docs/design/render.md` are exactly the arrangements that `free_classes_of_index` returns at the index $2n$, in their order, with the edge pairing of `WholeFace::edges`, which pins CLI-68. The test asserts that each search is complete and that the set lists 114 arrangements in all. This test of `M24` stays unchanged, and it shows that `--max-colours 1` gives the listing of `M24`.
- With the default `--max-colours`, on ten constructions of the whole-face set, `*632`, `*432`, `*532`, and `*732` with `010` and `100`, and `*442` and `*332` with `010`, the test reads each arrangement line that `info` prints back into its id, its number of pictures, whether it is `turned only`, and the meeting of every edge, and compares them with the library: for each $m$ from 1 to $M$, the count line of row 36 or row 38 and then the arrangements that `free_classes_of_index` returns at the index $2nm$, in their order, with the meetings of `WholeFace::edges`, and no line after the last, which pins CLI-68 and CLI-70. It also asserts that each line writes its pairs once, in the order of their lower edge. The test asserts that each of these searches is complete under `SearchBudget::DEFAULT`, and that at least one of the constructions has an $M$ above 3. It leaves out the other six constructions of the whole-face set, `*832`, `*542`, `*552`, and `*642` with `010`, and `*542` and `*642` with `100`, because one of their searches stops at a limit before $M$. This is a limit of the test: no test compares the arrangement lines of several pictures on those six tilings with the library. A unit test in `info.rs` checks the stop after a stopped search, below.
- With `--max-colours 2`, `info '*442' --wythoff 010` prints exactly the count lines `Arrangements: 16` and `Arrangements with 2 pictures: 29`, and its last line is the line of `16ac`, fixed values that the test holds (CLI-70). A second test asserts that the default listings of `*442` with `010` and of `*732` with `100` hold the lines of `16ab` and `18a` of the whole flow below.
- `info '*632' --wythoff 010` prints 7 arrangements with one picture, of which 3 are `turned only`, `info '*632' --wythoff 100` prints 6, of which 2 are `turned only`, and `info '*442' --wythoff 010` prints 16, each as the count of row 36. These are the numbers of types of the Euclidean catalogue of Section 5.9.5 of `docs/research/colourings.md` whose tile has no internal symmetry, a secondary source read through a summarizing tool and grouped by tiling by the web research, read as the arrangements of each tiling, and the numbers of those types whose group holds no reflection or glide reflection. A failure means that the code or that reading is wrong.
- With `--max-colours 1`, `info` on `*732`, `*542`, and `*532` with `100` gives the status `Success` and prints `Arrangements: 0` once, as its last line, and no `Arrangement:` line (CLI-68). This test of `M24` stays unchanged.
- With the default `--max-colours`, `info` on `*732` with `100` prints a line `Arrangements with 3 pictures` with a number above 0 and its `Arrangement:` lines, and on `*532` with `100` a line `Arrangements with 2 pictures` with a number above 0 and its lines, the numbers of pictures of the walk-through of `DEC-124`. These expected values are the results of the search, which rest on `UNV-41` to `UNV-43` and `UNV-51`, and no source states them: the test asserts what the tool prints, and a failure means that the search, its filter, or a statement that it rests on is wrong (CLI-68, CLI-70). "More pictures" of Section 8.1 of `docs/design/render.md` checks the same results in the library.
- `info` on the hyperbolic `*732`, `*832`, and `*542` with `010` prints at least one arrangement that is `turned only`. On these tilings the criterion of Theorem 0.1 of [YUNCKEN2003] for orientation-preserving groups holds (Section 5.9.2 of `docs/research/colourings.md`). On `*542` and `*732` with `100`, where the criterion fails, `info` prints the empty listing of the bullet above.
- `render-masks '*632' --wythoff 010 --arrangement 12a -o face.png` writes exactly `face.png`, a PNG file of the size of `--size`, whose bytes equal those of `WholeFace::render_mask` and `encode_png`, and nothing else, and an existing file of that name is replaced.
- `render-masks '*632' --arrangement 12a` gives the diagnostic of `--arrangement` without `--wythoff` and writes no file.
- `render-masks` with `--wythoff` fails on six inputs. On `*632` with `010`, these are the id `12h`, which does not exist, `--colouring 2a`, and no `--arrangement`. On `*732` with `100`, the input is the id `6a`, for which the search finds no arrangement. On `*(33)32` with `010`, it is the id `12a`, which meets the index limit, and on `*522` with `010`, a construction that is not a regular tiling. The test runs each input once with an existing output file and once with an output path where no file exists. Both runs give the status `Failure`, the same whole diagnostic of Section 6.10, and nothing on standard output. The existing file stays as it was, and no file appears at the other path (CLI-66, CLI-69).
- `render-masks` with `--wythoff` and `--arrangement` and an output path in a directory that does not exist gives the diagnostic of a face mask that cannot be written.
- `render '*632' --wythoff 010 --arrangement 12a` with one image item, and with one colour, writes a PNG file of the size of `--size`, without `--colours` gives the bytes of `--colours '#e9c46a'`, and with the image item gives other bytes.
- `render` with `--arrangement` fails on six inputs. On `*632` with `010`, these are the id `12h`, which does not exist, an image item that cannot be decoded, two colour items, `--colouring 2a`, and `--set p=3`. On `*732` with `100`, the input is the id `6a`, for which the search finds no arrangement. The test runs each input once with an existing output file and once with an output path where no file exists. Both runs give the status `Failure`, the same whole diagnostic of Section 6.8, and nothing on standard output. The existing file stays as it was, and no file appears at the other path (CLI-21, CLI-62, CLI-66, CLI-67).
- `render '*732' --wythoff 010 --arrangement 14a --samples 2 --size 48x32` gives the bytes of the whole-face render of `escher-render` with the first default colour. The test runs the same command with `--model half-plane`, with `--border-width 0`, and with `--size 40x40` in place of its size. Each run writes a PNG file of the size of `--size` whose bytes equal those of that render with the same option and differ from those of the first command (CLI-67).
- The whole flow, for `12a` and `12e` of `*632` with `010`: the test checks that `info` lists the id, writes the face mask with `render-masks` and checks its bytes as above, paints the white pixels of the mask with a gradient whose red follows the column and whose green follows the row, writes the picture with `encode_png`, and passes it to `render` as the image item, with `--samples 1` and `--border-width 0`. It asserts that each file equals the render of `escher-render` with the whole-face fill of the decoded picture, and that the files of the two arrangements differ.
- An image item that cannot be opened is reported at step 4, on `*(33)32`, before the error of the index limit of step 6.
- The whole flow with two pictures, for `16ab` of `*442` with `010`, the translation checkerboard, and with three pictures, for `18a` of `*732` with `100`. The test holds each id and its whole arrangement line as fixed values, results of the search. It checks that the default listing of `info` holds that line, writes the masks with `render-masks -o face.png`, and asserts that the directory then holds exactly `face-0.png` to `face-<m-1>.png` and nothing else, each a PNG file of the size of `--size` whose bytes equal those of `WholeFace::render_mask` with that picture and `encode_png`. It paints the white pixels of each mask with a gradient of its own, writes each picture with `encode_png`, and passes the files to `render` as image items in the order of the pictures, with `--samples 1` and `--border-width 0`. It asserts that each file equals the render of `escher-render` with the whole-face fill of the decoded pictures in the same order, and that exchanging the first two items gives other bytes (CLI-67, CLI-71).
- The face masks of an arrangement with three pictures are named `<stem>-0` to `<stem>-2`, with the extension kept, for an output path with an extension, without one, and with two dots (CLI-71).
- With several pictures, `render-masks` refuses an output path with no file stem before an empty size, and refuses a construction that is not a regular tiling before an output path with no file stem.
- For the arrangements `14a` of `*732` with `010` and `12a` and `12g` of `*632` with `010`, each with one picture, the face mask is the output file itself, with no other file beside it, and `render` with one colour gives the bytes of the whole-face render of `escher-render` with that one paint.
- For the same two arrangements, `render` without `--colours` gives the bytes of `--colours` with the first $m$ default colours, and `render` fails on four inputs: one item fewer than the pictures, one item more, the id with its last letter raised past the last arrangement of its number of pictures, and an image item that cannot be decoded. `render-masks` fails on that id and on an output path that names no file, a directory followed by `..`. The test runs each input once with existing files at the output path and at the names of the masks, and once where no file exists. Both runs give the status `Failure`, the same whole diagnostic of Section 6.8 or 6.10, and nothing on standard output. The existing files stay as they were, and no file appears at the other paths (CLI-21, CLI-66, CLI-67, CLI-71).
- `render-masks` with an arrangement with three pictures, where a directory has the name of the mask of picture B, writes the mask of A, gives the diagnostic of a face mask of picture B that cannot be written, and writes no mask of C (CLI-71).
- For several face types (`M26`), the snapshots of `escher info` show `--wythoff 111` on `*442`, `*632`, and `*732`, `--wythoff 011` on `*732`, and `--wythoff 111` on `*752`, which prints the error of the arrangement index limit. The snapshot of `*732` with `111` lists the index 84 and is the acceptance test of the command line for `DEC-139`, under the bound on its run time that Section 5.26 of `docs/design/poc-plan.md` states. The diagnostic snapshots show `escher render '*752' --wythoff 111 --arrangement 140a`, the usage error of an id whose number has three digits, with the status `Usage`, and, with the status `Failure`, `escher render '*732' --wythoff 111 --arrangement 84a`, which asks for 41 colours, and `escher render '*442' --wythoff 111 --arrangement 8a --colours '#000'`, which needs 4 colours. None of the three writes a file. The snapshots of `info --help`, `render --help`, `render -h`, `render-masks --help`, and `render-masks -h` change by the help texts of Section 4.2. The snapshots of `info` and `render` that show the refusal of a construction that is not a regular tiling, and the plain tests of `render-masks` that refuse one, take `*522` with `010`, because `*732` with `111` has several face types and is listed. In the plain tests, the arrangement lines of `*442` and `*632` with `111` and of `*732` with `011`, read back, equal the library, with each search asserted complete, the face types, the picture names, and the meetings of `WholeFace::edges` (CLI-74). With `--max-colours 1`, `*632` with `111` prints the one count line `Arrangements: 3` and three arrangements of index 12. The search at the index 8 of `*442` with `111` holds exactly one arrangement whose coset table is the table of the translations of "Several worked by hand" of Section 8.1 of `docs/design/render.md`, `8b`, and the default listing holds its line of Section 5.10. The whole flow on `8b` checks that `info` lists that line, writes four masks `face-0A.png`, `face-1A.png`, `face-2A.png`, and `face-2B.png` whose bytes equal `WholeFace::render_mask` of pictures 0 to 3, and no other file beside them, paints them and passes them back in that order, and asserts the render equal to the whole-face fill of `escher-render`, and that exchanging the items of `1A` and `2A` gives other bytes. `render '*442' --wythoff 111 --arrangement 8b` with four distinct solid colours, `--samples 1`, `--border-width 0`, and each of `--centre 0`, `--centre 1`, and `--centre 2` gives the bytes of the whole-face render of `escher-render` with the same paints. The pixel at the centre of each image has the colour of item 0, 1, or 2 in turn, because corner $N$ is the centre of octagon `0A`, of octagon `1A`, or of square `2A`. With `--centre 0`, a circle around the centre at $\sqrt{2}$ times the inradius of octagon `0A`, read from the image, crosses the colours of items 2, 1, 3, 1, 2, 1, 3, 1 in turn, the squares `2A` and `2B` alternating with octagons `1A` around `0A` (CLI-75). The four masks of `8b`, written at 256 by 256 pixels, equal `WholeFace::render_mask` byte for byte, and the areas of the convex hulls of their pixels with a channel above `0x40` give the ratio 1 between the two octagons and between the two squares, and $2(1 + \sqrt{2})$, the ratio of the area of a regular octagon to that of a square with the same edge length, between `0A` and `2A`, each within 2 percent, with octagon `0A` larger than a quarter of the image, which shows that the masks share one scale (`DEC-144`). `render-masks` with `8b` and an output path with no file stem, a directory followed by `..`, gives the diagnostic of Section 6.10, once with files at the names of the four masks and once with none, and changes no file (CLI-76). `render-masks` with `12a` of `*632` with `111`, where a directory has the name `face-1B.png` of the mask of `1B`, writes the masks of `0A` and `1A`, gives the diagnostic of picture `1B`, and writes no further mask (CLI-76). On a regular construction, `render` with `70a` of `*732` with `010` and with `66a` of the same tiling, one a multiple of 14 and one not, each without `--colours` and with `--colours '#000'`, and `render-masks` with the same two ids, give the whole diagnostic of the index limit with the status `Failure` and nothing on standard output, and leave the output file and 5 mask files beside it as they were, and create none where none existed (CLI-75). The arrangement `70a` has 5 pictures, so its run with one item shows that the refusal comes before the items of `--colours` are read. With several face types, these failures give their whole diagnostic with the status `Failure` and nothing on standard output, leave the output file and the files at the names of the masks as they were, and create none where none existed (CLI-75): `8d` of `*442` with `111`, which does not exist, with `render` and with `render-masks`, `12a` of the same tiling, whose number is not a multiple of its least index 8, with `render-masks`, which gives the diagnostic that the id does not exist because the tiling has 3 arrangements with 4 pictures, `8b` with three and with five items, `84a` of `*732` with `111` without `--colours`, `12a` of `*332` with `111`, whose search finds no arrangement, and `84a` of `*752` with `111`, whose least index is above 84, with `render` and 59 items and with `render-masks`. The test names no mask file for the last three. For every input, the shared helper of these tests also asserts that the sorted listing of the directory of the test is the same after both runs as before, so a refused run writes no file under any name. Unit tests in `info.rs` check the formatter of row 37 for several face types on the meetings of `8b` of `*442` with `111` and the forms ` to ` and ` of <n> sides` on `16a`, that the listing of `*442` with `111` searches the indices 8, 16, and 24 under the colour limit 12, 8 and 16 under 11, 8 alone under 1, and the multiples of 8 up to 80 under 64, that `*732` with `111` searches 84 alone and `*752` with `111` 140 alone, that `*752` with `111` prints row 36 with the error of the arrangement index limit and no further line, and, with budgets that the tests choose, that a stop at a further index ends the listing. The plain tests above assert the names of CLI-76: the four mask names of `8b` and the masks `face-0A.png` and `face-1A.png` of `12a` of `*632` with `111`. The unit tests of CLI-58 in `masks.rs` cover the shapes of the output path, because the names of CLI-76 come from the same function with the picture name in place of the colour. Unit tests in `render.rs` check that the pictures of `*442` with `111` are named `0A`, `1A`, `2A`, and `2B` at the index 8 and `0A` to `2D`, with their numbers of sides, at 16, while those of the hexagons of `*632` at 24 are `A` and `B`, that an id of `*442` with `111` whose number is not a multiple of 8, such as `12a`, needs the 4 items of the least index and gives the diagnostic that it does not exist with the total of 4 pictures, that `84a` with 41 pictures and no `--colours` gives the diagnostic of CLI-44 while 12 pictures take the 12 default colours, and that the refusal of an id from 65 to 84 applies to `70a` of the heptagons of `*732` and not to `84a` of `*732` with `111` or to `64a`.
- Every command of `M24` gives the bytes or the text that it gave before, with the exceptions of `M25` below and, from `M26`, the listing and the diagnostics of the constructions with several face types, which `M24` refused, the one exception of CLI-75 for an `--arrangement` id with a number from 65 to 84 on a regular construction, and the help texts of Section 4.2. Within `M25`, there were three exceptions that this design makes: a listing whose largest number of pictures is above 1 gains the lines of two or more pictures after its last line of `M24`, the diagnostic that an id does not exist gains ` with 1 picture`, and the help texts of Section 4.2 change. The plain tests and snapshots of `--arrangement`, `info --wythoff`, and `render-masks --wythoff` with one picture keep every other assertion, and the plain tests of the listing pass `--max-colours 1` and stay as they are.

Unit tests in `args.rs` check that the parser of `--arrangement` accepts the ids `6a`, `12a`, `14c`, `64zzzz`, `65a`, and `84zzzz` and rejects `12A`, `12`, `012a`, `85a`, `a`, the empty value, `12a` with a space before or after it, `one`, and `parity`. Unit tests in `render.rs` resolve `12e` of the hexagons of `*632`. With a budget of 1 step, they reach the diagnostic of a search for `12a` of the hexagons that stops at a limit, once for `render` and once for `render-masks`. They also check the one item of `--colours` and its default colour for an arrangement, and reach the diagnostic of a complete search with an empty list through the private function of CLI-66, with a list from a search with stabilizer words that the test chooses. A unit test in `info.rs` checks that a search that stops at a limit at the first index prints `Arrangements: none, the search stopped at a limit` and no other arrangement line, as in `M24`. Others check, with budgets that the tests choose through the private function of CLI-66, that a search that stops at the index $2nm$ for some $m \ge 2$ prints the lines of the lower numbers of pictures, then `Arrangements with <m> pictures: none, the search stopped at a limit`, and no further line, that $M$ is the smaller of the colour limit and $\lfloor 32/n \rfloor$ for faces of up to 32 sides, and that a colour limit of 1 gives the lines of one picture only. They also check the formatter of row 37 on the meetings that `WholeFace::edges` returns for arrangements held as fixed ids: `8e`, `8o`, and `8n` of `*442` with `010` for one picture, `16ab` and `16h` of the same tiling for two, and `18a` of `*732` with `100` for three. Between them, these hold a turned pair, a mirrored pair, an edge that meets itself turned and one that meets itself mirrored, and a turned and a mirrored meeting between two pictures. With one picture, the formatter gives the form of `M24`. With several, it writes each edge with the letter of its picture and the pairs in the order of their lower edge by picture and then by number. It writes `turned only` exactly when no meeting is mirrored. Unit tests in `render.rs` check that `14a` of the hexagons of `*632`, whose number is not a multiple of 12, is resolved at the index 12 with the diagnostic of an id with one picture, that `24k` of the hexagons is resolved at the index 24 and `24l` gives the diagnostic with `11 arrangements with 2 pictures`, that `24a` needs two items of `--colours`, the words `1 arrangement with 1 picture` for `10b` of `*532` with `010`, which the unit test of `M24` asserted without ` with 1 picture`, the words `1 arrangement with 2 pictures` for `16b` of `*432` with `010`, and the words `no arrangement with 2 pictures` through the private function with a list from a search with stabilizer words that the test chooses. Unit tests in `masks.rs` check the names of CLI-71 for three pictures, which are those of CLI-58 for three colours.

A snapshot holds the status, the standard output, and the standard error of one `run` call, in this form, which a helper in the module `common` builds (rule TEST-12 of `docs/rust-coding-standards.md`):

```text
status: <Debug text of the Status>
--- stdout
<standard output>--- stderr
<standard error>--- end
```

Each stream follows its marker line unchanged, so a stream that ends with a line feed puts the next marker on a line of its own, and a stream that does not runs into it. The exceptions are the snapshots of `render --help` and `render-masks --help`, whose lines have their trailing whitespace trimmed, because `clap` indents the blank lines of a long help. Each test asserts that no line of the untrimmed output ends in whitespace unless it holds only whitespace. The final marker `--- end` is needed because `insta` trims the whitespace at the end of a snapshot before it compares, so without it a missing or extra final line feed of standard error would pass. The writer that fails, `FailingWriter`, is also in `common`.

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

### 8.3. Coverage of the Supported Set

Section 3.2 of `docs/design/poc-plan.md` states the set of signatures that the proof of concept renders, and Section 4 of `docs/design/domain.md` specifies it with its limits. Two tests check the commands against that set. The property test runs `render` over generated valid signatures and over a fixed list of signatures at the limits on their orders. The snapshot corpus of `info` holds one snapshot for each class of signature of the set and for each class of refusal.

#### 8.3.1. Generator

The property test is the module `properties` of the integration-test binary, in `crates/escher-cli/tests/cli/properties.rs`, which `main.rs` declares under `#[cfg(test)]` with the other modules. It calls `run` with `Vec<u8>` streams and an output path from the helper of `common`, as the plain tests do.

Its strategies are `order` and `structured_text` of the test support crate `escher-reference`, which also holds the helper `order_text` that they use. The property tests of `escher-domain` draw from the same strategies, so a change to a bound reaches both tests. An integration-test binary cannot import the test code of another crate, and TEST-15 of `docs/rust-coding-standards.md` puts the test helpers that several crates share into that one crate. `structured_text` writes up to 1 handle, up to 4 cone points, up to 2 boundaries of up to 4 corners each, and up to 1 crosscap, in that order. `order` draws an order from 2 to 9 with weight 4, from 10 to 110 with weight 2, and `(inf)` with weight 1. A generated text is a valid signature when the parser of `escher-signature` accepts it. The test runs nothing for a text that the parser refuses, because CLI-11 and the diagnostic snapshots of Section 6.4 cover the signature errors.

The `proptest!` block runs with the default configuration of `proptest`, as the property tests of the other crates do, and the crate commits the regression file that `proptest` writes for it (TEST-9 of `docs/rust-coding-standards.md`). Each case renders an image of 8 by 8 pixels with one sample per pixel, the smallest render of the plain tests, and a refused signature stops at step 3 of Section 4.5, before any search or rendering.

#### 8.3.2. Oracle

The test decides the expected outcome of each valid signature with an oracle of its own, in test code. The oracle reads the parts of the parsed `Signature`, which are `handles`, `crosscaps`, `cone_orders`, and `boundaries` with their `corner_orders`, and its `geometry` and `is_compact`, which the tests of `escher-signature` check. It calls no function of `escher-domain`, so a wrong decision of `build_with` does not become the expectation of the test. It implements Section 4 of `docs/design/domain.md` in these steps:

1. A spherical signature is supported, because every valid spherical signature is. A Euclidean signature is supported when it is compact, because the compact Euclidean signatures are the 17 groups, each of which `escher-domain` builds. A hyperbolic signature is supported when it is compact, has no handle and no crosscap, and has either one boundary and no cone point or no boundary, with at least 3 orders. Every other valid signature is unsupported.
2. A supported signature with more than 8 orders is over a limit (DOM-38 of `docs/design/domain.md`).
3. A supported signature with one cone point of order 2, one boundary with one corner of order $n$, and $n > 50$ is over a limit (DOM-63).
4. Any other supported signature with an order above 100 is over a limit (DOM-3).
5. Every other supported signature is in the set.

The steps follow the order in which `escher-domain` checks, which puts support before the limits (DOM-3), so a signature that is unsupported and over a limit is unsupported. Every general non-compact signature is hyperbolic and not compact, so step 1 makes it unsupported.

#### 8.3.3. Assertions

CLI-77. For each valid signature that the generator or the fixed list gives, the property test MUST run `escher render <signature> --size 8x8 --samples 1 -o <path>` and MUST assert the outcome that the oracle gives:

- In the set: the status `Success`, nothing on standard output or standard error, and a PNG file at the path whose header, read by hand, gives 8 by 8 pixels.
- Unsupported: the status `Failure`, nothing on standard output, no file at the path, and a standard error of exactly two lines, `error: cannot render <canonical>: rendering this signature is not supported` and `  caused by: cannot build fundamental domain: the signature <canonical> is not supported`, where `<canonical>` is the `Display` text of the parsed `Signature`.
- Over a limit: the status `Failure`, nothing on standard output, no file at the path, a first line of standard error that is exactly `error: cannot render <canonical>`, and a second line that starts with `  caused by: cannot build fundamental domain: `.

The assertions follow the two rows of Section 6.8 for a refused domain, so the test fails when `render` refuses a signature of the set, draws a signature outside it, or reports a limit as a lack of support. Without them, a signature of the set that `render` refuses, which is the defect that this test exists to find, would pass unnoticed. The test also asserts that the oracle gives "unsupported" for every valid signature whose `is_general_non_compact` is true, which meets Section 5.8 of `docs/design/architecture.md` for those signatures.

The test does not pin the text after `caused by: ` for a limit, because Section 10.2 of `docs/design/domain.md` owns it and the snapshots of Section 8.1 cover each form. The code branch changes neither the oracle nor the fixed list to make the test pass. A failure is a defect of the code or of Section 4 of `docs/design/domain.md`, and a fix that would change which signatures the proof of concept renders, or add a limit, goes to the owner, who set the set (`DEC-64`).

#### 8.3.4. Fixed List

The generator rarely draws an order near 100 and never draws more than 4 orders of one kind, so a fixed list of signatures at the limits on their orders runs beside it. Each entry carries its outcome, and the test asserts that the oracle gives that outcome before it runs `render`, so an error in the oracle fails the test on the list. Every signature of the list is written as the input text:

| Outcome | Signatures |
| --- | --- |
| In the set, hyperbolic | `*732`, `*(100)32`, `*(100)(100)(100)`, `732`, `(100)(100)(100)`, `*2223`, `*(100)(100)(100)(100)`, `*22222222`, `*(100)(100)(100)(100)(100)(100)(100)(100)`, `2223`, `22222222`, and `(100)(100)(100)(100)(100)(100)(100)(100)` |
| In the set, Euclidean | `o`, `2222`, `333`, `442`, `632`, `*2222`, `*333`, `*442`, `*632`, `2*22`, `3*3`, `4*2`, `22*`, `**`, `*x`, `22x`, and `xx` |
| In the set, spherical | `*332`, `*432`, `*532`, `332`, `432`, `532`, `3*2`, `*222`, `*(100)22`, `222`, `(100)22`, `*22`, `*(100)(100)`, `22`, `(100)(100)`, `2*2`, `2*(50)`, `2*`, `(100)*`, `2x`, `(100)x`, `*`, `x`, and `1` |
| Unsupported | `4*3`, `o2`, `xxx`, `***`, `*(inf)32`, `(inf)32`, the 7 friezes `(inf)(inf)`, `(inf)*`, `(inf)x`, `*(inf)(inf)`, `22(inf)`, `2*(inf)`, and `*22(inf)`, and the general non-compact signatures `*(inf)222`, `(inf)222`, `3*(inf)`, and `(inf)(inf)(inf)(inf)` |
| Over a limit | `*(101)32`, `(101)32`, `*(101)222`, `(101)222`, `*(101)22`, `(101)22`, `*(101)(101)`, `(101)(101)`, `(101)*`, `(101)x`, `2*(51)`, `*222222222`, and `222222222` |

#### 8.3.5. What the Property Test Catches

The test catches a signature of the set that `build_with` refuses or that `render` fails on with the default options, a signature outside the set that `render` draws, a refusal that names the wrong cause, a limit off by one at 100 and 101, at 50 and 51 for `2*n`, and at 8 and 9 orders, and a panic of `run` on any valid signature that it generates (CLI-2).

It cannot show that an image is right, which the golden images and the oracles of Section 8.1 of `docs/design/render.md` check. It runs only the default options, so it says nothing about `--set`, `--model`, `--wythoff`, or a colouring id. The generator draws at most one handle and one crosscap, at most 4 orders of one kind, and no order above 110, so the polygons with 5 to 8 corners and the refusals of 9 orders are covered only by the fixed list. The oracle and `escher-domain` implement the same section of `docs/design/domain.md` and read the same accessors of `escher-signature`, so an error in that section or in the classification of `escher-signature` passes both.

#### 8.3.6. Snapshot Corpus of `info`

CLI-78. The snapshots of `info` MUST hold, for each class of the tables below, a snapshot of `escher info` on the signature that the table names, because a class without a snapshot can change its information text or its `Domain` line with no test failing (CLI-7). A snapshot that this requirement adds runs `escher info <signature>` with no option other than the `--max-colours` that CLI-50, CLI-52, or CLI-54 requires for its kind of domain.

The classes of the set:

| Class | Signature |
| --- | --- |
| Hyperbolic reflection triangle | `*732` |
| Hyperbolic reflection polygon | `*3333` |
| Hyperbolic rotation triangle | `732` |
| Hyperbolic rotation polygon | `3333` |
| Each of the 17 Euclidean groups | `o`, `2222`, `333`, `442`, `632`, `*2222`, `*333`, `*442`, `*632`, `2*22`, `3*3`, `4*2`, `22*`, `**`, `*x`, `22x`, and `xx` |
| `*332`, `*432`, `*532` | `*332`, `*432`, `*532` |
| `332`, `432`, `532` | `332`, `432`, `532` |
| `*n22` | `*722` |
| `*nn` | `*55` |
| `n22` | `(12)22` |
| `nn` | `55` |
| `3*2` | `3*2` |
| `2*n` | `2*3` |
| `n*` | `3*` |
| `nx` | `5x` |
| `*`, `x`, `1` | `*`, `x`, `1` |

The classes of refusal:

| Class | Signature |
| --- | --- |
| Cone points and a boundary | `4*3` |
| A handle | `o2` |
| Crosscaps | `ox`, which prints as `xxx` |
| Two or more boundaries | `***` |
| A frieze | `*22(inf)`, which prints as `*(inf)22` |
| A triangle with an ideal corner | `*(inf)32` |
| The rotation signature of a triangle with an ideal corner | `(inf)32` |
| A general non-compact signature | `3*(inf)` |
| An order above 100 | `*(101)32` |
| An order of `2*n` above 50 | `2*(51)` |
| More than 8 orders | `*222222222`, with `--max-colours 4` |
| A bad orbifold | `23`, a snapshot of the module `diagnostics` |

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
escher-reference = { version = "0.1.0", path = "crates/escher-reference" }
escher-render = { version = "0.1.0", path = "crates/escher-render" }
escher-signature = { version = "0.1.0", path = "crates/escher-signature" }
insta = { version = "1.49", default-features = false }
proptest = { version = "1.12.0", default-features = false, features = ["std"] }
rayon = "1.12.0"
```

| Dependency | Kind | Features | Reason |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | Parses and classifies the signature. The path dependency carries a version (TOOL-25) |
| `escher-domain` | Normal | None | Lists the shape parameters for the parameter lines, builds and verifies the fundamental domain for the domain summary and `--check`, builds the domain that `render` draws with the settings, and gives the names and ranges that the diagnostic of a rejected setting uses. The path dependency carries a version (TOOL-25) |
| `escher-group` | Normal | None | Derives the Conway presentation, gives the text of the relators, and enumerates the cosets of both presentations for `--presentation`, searches the colourings for the colouring lines, and parses and looks up a colouring id for `render`. The path dependency carries a version (TOOL-25) |
| `escher-render` | Normal | None | Renders the image, with the coset fill for a colouring id or an image item, counts its statistics, decodes the files of the image items, and encodes the image as PNG for `render`, and renders and encodes the masks for `render-masks`. The path dependency carries a version (TOOL-25) |
| `rayon` | Normal | Default | The thread pool of `render` (`DEC-21`, Section 3.5 of `docs/design/architecture.md`) |
| `clap` | Normal | The list above, without `color`, `env`, and `wrap_help` | The derive API (Section 3.1 of `docs/design/architecture.md`). Without `color`, `anstream` and its dependencies are not built (Section 9.2) |
| `anyhow` | Normal | Default | The chain of errors that Section 5.8 of `docs/design/architecture.md` requires. Version 1.0.103 fixes RUSTSEC-2026-0190 (Section 10 of `docs/research/rust-ecosystem.md`) |
| `insta` | Dev | None, so without `colors` | Snapshot tests (rule TEST-2). Without `colors`, `console` is not built (Section 9.2) |
| `proptest` | Dev | `std`, without the default features, from the one entry of the workspace that the other crates use | The property test of Section 8.3 (rule TEST-9) |
| `escher-reference` | Dev | None | The signature strategies of the property test (Section 8.3.1, rule TEST-15). The path dependency carries a version (TOOL-25) |

The crate declares neither `escher-geometry` nor `glam`. The measurements of the domain are methods of `FundamentalDomain` that return plain `f64` values, so this crate names no `glam` type (Section 15.2 of `docs/design/domain.md`). `render` passes the domain to `escher-render` unchanged and reads only integers from the statistics, so it names no `glam` type either. `escher-domain` brings `escher-geometry`, `glam`, and `thiserror` into the build of the binary, and `escher-render` brings `png` and its dependencies. `escher-group` depends on `escher-signature` and `thiserror` only (Section 12 of `docs/design/group.md`), so it adds no external crate to the build.

The crate declares no colour crate and no `png`. The private parser of Section 4.6 reads the two hexadecimal forms of a colour (`DEC-29`), the default colours are a table of literal values (CLI-44), `escher-render` decodes the image files, and the tests read the PNG header by hand and write their image files with `encode_png` (Section 8.1).

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

This section lists the unverified statements of Section 7 of `docs/design/poc-plan.md` that this crate relies on, as PLAN-3 of that document requires. The crate relies on `UNV-20` and `UNV-21` through `escher-signature`, on `UNV-6`, `UNV-7`, `UNV-10`, `UNV-11`, `UNV-22`, `UNV-26`, `UNV-27`, `UNV-28`, `UNV-29`, `UNV-30`, `UNV-31`, `UNV-32`, `UNV-33`, and `UNV-34` through `escher-domain`, on `UNV-17`, `UNV-19`, `UNV-22`, `UNV-35`, and `UNV-36` through `escher-group`, on `UNV-37` and `UNV-38` directly, on `UNV-13` and `UNV-14` through `escher-geometry`, and on `UNV-1`, `UNV-2`, `UNV-3`, `UNV-4`, `UNV-13`, `UNV-14`, `UNV-15`, `UNV-16`, `UNV-17`, `UNV-18`, `UNV-38`, `UNV-39`, `UNV-40`, and `UNV-41` to `UNV-56` through `escher-render`, whose design states how it relies on each.

| Label | How this crate relies on it | Relied on in |
| --- | --- | --- |
| `UNV-1` | `render` relies on the fold to stop for the samples of an image of a reflection triangle, polygon, rectangle, or lune. `Unconverged samples` counts the samples for which it does not stop within the fold cap, which on the sphere can happen near a vertex of very high order | 4.5, 4.7 |
| `UNV-2` | `render` relies on the fold to stop for the samples of an image of a rotation domain, a wallpaper domain, whose pairings include translations and glide reflections, or a spherical group domain, whose pairings include rotary reflections. `Unconverged samples` counts the samples for which it does not stop within the fold cap | 4.5, 4.7 |
| `UNV-3` | A sample on or next to a wall can take either neighbouring tile, which is why CLI-28 masks the fold counts | 8.1 |
| `UNV-4` | The default fold cap of `DEC-7` rests on fold counts that no source bounds | 4.6 |
| `UNV-6` | `info` prints the parameters and the summary of a reflection polygon, and `render` draws it, from the construction of SymmHub's cuts that `escher-domain` builds. `build_with` returns only a polygon or a double that passes `verify()` (DOM-44 of `docs/design/domain.md`), and `info --check` reports the checks | 4.5, 5.1, 5.5 |
| `UNV-7` | As for `UNV-6`, for the order of the cuts, which fixes what each parameter of a reflection polygon means | 4.5, 5.1, 5.5 |
| `UNV-10` | `info` prints one `Parameter` line for each parameter that `escher-domain` specifies, such as one for `*2222` and two, `aspect` and `skew`, for `o` and `2222`. No source states whether the published parameter count of `*2222` includes the scale, so the lines list the parameters of the construction and claim no count | 5.1 |
| `UNV-11` | `info` prints the summary of a wallpaper domain and of a spherical group domain, and `render` draws each, from constructions that no source states, as the rows of `UNV-33` and `UNV-34` describe | 4.5, 5.5 |
| `UNV-13` | The image of `--model band` comes from the map from the band to the disk that only the code of SymmHub states (Section 7.6 of `docs/design/geometry.md`) | 4.6 |
| `UNV-14` | The border that `render --border-unit intrinsic` draws on a hyperbolic or spherical signature comes from the distance to a wall of Section 5.6 of `docs/design/geometry.md`, whose forms on the hyperboloid and on the sphere no source states as the code evaluates them | 4.6 |
| `UNV-15` | The border that `render` draws comes from the gradient form of the border estimate (`DEC-28`) | 4.5 |
| `UNV-16` | The ball that `render` draws comes from the lift onto the ball and the Phong reflection model, whose primary paper is unread | 4.6 |
| `UNV-17` | `info` lists the colourings as the colourings of the tiling, and `render` draws one by its coset fill, which rests on the coset colour of Section 5.12 of `docs/design/render.md`. `info` claims nothing about which colourings are perfect, and prints only the subgroups that the search finds. The code colours of the borders of the masks of `render-masks`, which group the edges that can meet, rest on the same reading and on the closure of the coset table that Section 5.19 of `docs/design/render.md` states and its Section 10 lists as unverified | 4.6, 4.8, 5.8 |
| `UNV-18` | The colouring `parity` of an all-reversing domain, which includes a domain whose sides are paired by glide reflections or rotary reflections, colours the tiles by the parity of the fold with `Fill::Parity`, and gives the bytes of the coset fill of `2a` (CLI-48) | 4.6 |
| `UNV-19` | The order lines of `--presentation` come from the coset enumeration of `escher-group`, and the colouring lines and the colourings of `render` from its low-index search, whose reliance on `UNV-19` Section 13 of `docs/design/group.md` states. A run that reaches the limit prints `not found` and claims nothing about the index (Section 5.7), and a search that stops at a limit claims nothing beyond its last complete colour count (CLI-47) | 4.6, 5.7, 5.8 |
| `UNV-20` | `info` prints the canonical signature, with parentheses for orders of 10 or more and `(inf)` for an infinite order, and accepts both spellings of infinity and `1` as the trivial signature, whose domain has no side (CLI-53) | 5.1, 5.5 |
| `UNV-21` | `info` prints the geometry and the area of a non-compact signature whose Euler characteristic is not 0, and prints no deformation dimension for a non-compact signature | 5.1 |
| `UNV-22` | `info` prints the summary of a rotation domain, and `render` draws it as a fundamental domain of the rotation subgroup, which rests on the signature of that subgroup. `--presentation` prints the Conway presentation of a boundary without corners, such as that of `*`, in the reading that Section 5.2 of `docs/design/group.md` records (GRP-7) | 4.5, 5.5, 5.7 |
| `UNV-26` | `info --check` reports the checks of the walls of the domain, which `escher-domain` builds through the formula of the wall through two points that `UNV-26` covers. The walls, relators, and angles checks test the result (Section 14 of `docs/design/domain.md`). The domain summary does not depend on the walls, because `escher-domain` measures it from the vertices | 5.6 |
| `UNV-27` | As for `UNV-6`, for the default shape and the ranges of the parameters of a reflection polygon | 4.5, 5.1, 5.5 |
| `UNV-28` | `info` prints the summary of a Euclidean triangle, and `render` draws it, from the vertex coordinates that `escher-domain` builds, whose checks and tests its design states | 4.5, 5.5 |
| `UNV-29` | `info` prints the summary of a lune, and `render` draws it as a fundamental domain, which no source states. `info --check` reports the checks that test it | 4.5, 5.5 |
| `UNV-30` | `info` prints the summary of a rotation domain built as the double of a lune, and `render` draws it as a fundamental domain of the rotation subgroup. `info --check` reports the checks that test each build | 4.5, 5.5 |
| `UNV-31` | As for `UNV-30`, for the convexity and the vertex cycles of the double of a reflection polygon | 4.5, 5.5 |
| `UNV-32` | `--presentation` prints the side presentation of a rotation domain, whose cycle relators as words are the reading of `docs/design/domain.md` (Section 5.4 of that document). The side order line compares the order that they present with the group order | 5.7 |
| `UNV-33` | `info` prints the summary of a wallpaper domain, and `render` draws it as a fundamental domain of the group of its signature, which rests on constructions and pairings that no source states. `build_with` returns only a wallpaper domain that passes `verify()` (DOM-54 of `docs/design/domain.md`), and `info --check` reports the checks | 4.5, 5.5 |
| `UNV-34` | `info` prints the summary of a spherical group domain, with the word `rotary reflection`, and `render` draws it as a fundamental domain of the group of its signature, which rests on domains and pairings that no source states. `build_with` returns every such domain except that of `*` only when it passes `verify()` (DOM-60 of `docs/design/domain.md`), the certificate of DOM-64 of that document checks each group, and `info --check` reports the checks. The order lines of `--presentation` compare the order of the side presentation with the group order, which cannot tell these groups from others of the same order (DOM-64 of `docs/design/domain.md`) | 4.5, 5.5, 5.7 |
| `UNV-35` | `info` counts and lists, and `render` resolves, only the colourings that `escher-group` finds valid by the condition on the side generators of GRP-42 of `docs/design/group.md`, so a colouring in which two tiles that share a side have the same colour reaches the user only if that condition is wrong | 4.6, 5.8 |
| `UNV-36` | The alias `parity` names `2a`, as the only valid colouring with 2 colours. If a domain had two, `parity` would name the first of them, and the second would keep its own id | 4.6 |
| `UNV-37` | `info` prints the regular tiling line for `*pq2` and `pq2` with $p \ge q \ge 3$, as the Schlafli symbol `{p,q}` and that of its dual, `{q,p}`, which rests on these signatures naming the symmetry group and the rotation group of $\{p, q\}$ (Section 5.9). The line is computed from the signature alone, so no check of the domain tests it, and its tests pin only which signature gets which symbols. The help of `--wythoff` and Section 4.6 tell the user that `010` draws the first of the two symbols and `100` the second | 4.6, 5.9 |
| `UNV-38` | `render --wythoff` draws the faces of the face rule, with the generator point and the face of each sample that `escher-render` computes. On a Euclidean or hyperbolic triangle these rest on statements that the sources make for the sphere only, or that only two shaders make, which the tests of Section 8.1 of `docs/design/render.md` check numerically. The table of the markings of `*732` in Section 4.6 takes its vertex configurations from Wikipedia | 4.6 |
| `UNV-39` | As for `UNV-38`, for the face of each sample | 4.6 |
| `UNV-40` | As for `UNV-38`, for the generator point | 4.6 |
| `UNV-41` | `info --wythoff` lists, and `render --arrangement` resolves, the arrangements that `escher-group` finds with the stabilizer words of `escher-render`, as the subgroups that put one picture on every face of each of their orbits, prints the number of pictures as the index divided by twice the number of sides, and prints the edge pairing, within one picture and between pictures, that `escher-render` reads from each table. `render` draws the picture of each face that `escher-render` computes. These rest on statements that no source makes for subgroups with orientation-reversing elements, or at all for coset tables, which the tests of Section 8.1 of `docs/design/render.md` check, and the plain tests of Section 8.1 compare the counts with the Euclidean catalogue and with Yuncken's theorem | 4.6, 4.8, 5.10, 8.1 |
| `UNV-42` | As for `UNV-41` | 4.6, 4.8, 5.10, 8.1 |
| `UNV-43` | As for `UNV-41`, for the number of pictures | 4.6, 4.8, 5.10, 8.1 |
| `UNV-44` | As for `UNV-41`, for the stabilizer words of a regular tiling | 4.6, 4.8, 5.10 |
| `UNV-45` | As for `UNV-41`, for the edge pairing of one picture | 4.6, 4.8, 5.10 |
| `UNV-46` | As for `UNV-41`, for the picture of each face | 4.6 |
| `UNV-47` | With several face types, `info --wythoff` lists, and `render --arrangement` resolves, the arrangements that `escher-group` finds with the union of the stabilizer words of every type, prints the number of pictures of each type as the index divided by twice the order of its corner, and prints the meetings between the types that `escher-render` reads from each table, and `render` draws the picture of each face that `escher-render` computes. No source states these statements, and the tests of the layer "Several face types" of Section 8.1 of `docs/design/render.md` check them | 4.6, 4.8, 5.10 |
| `UNV-48` | As for `UNV-47`, for the face of each type | 4.6, 4.8 |
| `UNV-49` | As for `UNV-47`, for the arrangements that the union of the stabilizer words finds | 4.6, 5.10 |
| `UNV-50` | As for `UNV-47`, for the number of pictures of each type | 4.6, 5.10 |
| `UNV-51` | As for `UNV-41`, for the pictures of an arrangement with several pictures | 4.6, 4.8, 5.10, 8.1 |
| `UNV-52` | As for `UNV-41`, for the meetings between pictures | 4.6, 4.8, 5.10 |
| `UNV-53` | As for `UNV-41`, for the picture of each face with several pictures | 4.6 |
| `UNV-54` | As for `UNV-47`, for the pictures of each type | 4.6, 4.8, 5.10 |
| `UNV-55` | As for `UNV-47`, for the meetings between types | 4.6, 4.8, 5.10 |
| `UNV-56` | As for `UNV-47`, for the picture of each face | 4.6 |

## 11. Assumptions, Choices, and Open Questions

### 11.1. Assumptions

- The infinity sign takes one column in the terminal. Unicode gives it an ambiguous East Asian width, so a terminal that renders such characters as wide shifts the caret one column to the right for each infinity sign before the span.
- With its features `env` and `wrap_help` off, `clap` reads no environment variable, so the output does not depend on the environment. A reading of the source of `clap_builder` supports this, and no run has confirmed it.
- The residual of a passed check and the fold counts can differ in their last digits between platforms, which is why Section 8.1 masks them (CLI-20, CLI-28). No run on two platforms has measured this, and the masking is safe either way.
- With the default actions of its derive API, `clap` rejects a flag or an option that is given more than once. With the action `Append`, it accepts `--set` more than once and keeps its values in the order given, which the diagnostic of a parameter set twice relies on.
- `clap` generates the usage line of `render` that Section 4.1 gives, the text `[default: <value>]`, `[possible values: phong, none]`, `[possible values: disk, half-plane, band, klein]`, and `[possible values: pixels, intrinsic]` in its help, and no possible values for `--colouring`, whose value parser is a private function.
- A setting of `--set` does not change the side presentation of a domain, so the colouring ids of a signature do not depend on it. The side presentation reads only the kinds, partners, corner orders, and vertex cycles of the sides (Section 5.4 of `docs/design/domain.md`), and the numbering of the sides depends on the canonical signature alone (DOM-37, DOM-43, and DOM-53 of that document). No test of this crate compares the side presentations of two settings.
- `rayon` reads the environment variable `RAYON_NUM_THREADS` only for a pool whose builder sets no number of threads, so it has no effect on the pool of `render`, which always sets one. Nothing has confirmed this, and an error here changes only the number of threads that a call starts.
- `std::fs::write` follows a symbolic link at the path, as the documentation of `std::fs::File::create` states.
- The PNG header that the tests read has the layout of the PNG specification, which no document under `docs/research/` cites. The tests of `escher-render` decode the same bytes with `png`, so the two layouts agree.
- `std::path::Path::file_stem` returns no stem for `/` and `..`, and `std::path::Path::extension` takes the text after the last `.` of the file name, except for a name whose only `.` is its first character, as the documentation of `std::path::Path` states. The names of CLI-58 rely on it.
- `std::fs::File::open` follows a symbolic link and opens a FIFO or a device as well as a regular file, so an image item can name a file that blocks or never ends (Section 12).
- A write to a FIFO whose reader has closed fails with the error kind `BrokenPipe` and does not end the process, because Rust ignores the signal `SIGPIPE` (Section 6.6). This holds only for a write larger than the buffer of the pipe, 64 KiB by default on Linux.

### 11.2. Choices of This Design

These choices are made here, and no other document dictates them. Each one is easy to undo by mistake, or would otherwise leave a reader asking why the tool behaves as it does:

1. `clap` and `insta` build without their colour features, so the output is plain text and `anstream` and `console` stay out of the build (Section 9.2).
2. Snapshots live only in this crate, with one snapshot for each `run` call, so `escher-signature` needs no `insta` dependency (Section 8.1). No recipe wraps `cargo insta`, because a reviewer accepts a changed snapshot by hand (Section 8.2).
3. `info` returns its errors as a list, and `run` writes one diagnostic for each, in order, so `info --check --presentation` can report an order diagnostic and then a verification diagnostic (Section 3.2).
4. `clap` checks only the syntax of an option value, and `escher-render` checks its range, so a size of `0x0` gives `Failure` and not `Usage` (Section 4.6, CLI-24).
5. `render` reads the border colour and the background colour before it builds the domain, and reads the fill colours and checks the render options before it resolves a colouring id, so a certain input error never waits for a search or gives way to the diagnostic of a search that stopped at a limit, except with the default colouring of a domain that is not all-reversing (Section 4.5).
6. The render options are checked before the search with a placeholder fill of one colour, which is the border colour, and built a second time with the real fill, because the builder of `escher-render` takes the fill first and the coset fill exists only after the search (Section 4.5).
7. `render` clones the colouring that `find` returns, because `ColouringsOfIndex` only lends it and `CosetFill::new` takes it by value (Section 4.5).
8. The private function that resolves an id takes the step budget as a parameter, and `render` passes `SearchBudget::DEFAULT`, so a unit test can reach the diagnostic of a search that stops at a limit with a small budget (Sections 4.5 and 8.1).
9. `--colouring` has no default of `clap`, because its default depends on the domain (CLI-34), and `--border-width` has none, because its default depends on `--border-unit` (CLI-56).
10. Without `--wythoff`, `render` refuses no domain for its geometry or for the kinds of its sides, because `escher-render` has a view for every geometry (RND-23 of `docs/design/render.md`) and renders a rotation domain within the bounds of Section 6.8 of that document (RND-35).
11. The tests read the PNG header by hand, so the crate has no `png` dev-dependency (Section 8.1).
12. The examples and snapshots that need a signature without a domain use `4*3`, and those that need a spherical signature without a domain use `2*(51)`, which is over the limit of `2*n`, so that `info` prints its group order (Sections 5.3 and 8.1).
13. `parity` names the valid colouring with 2 colours, `2a`, and is refused where the domain has none, so the alias and the default of `DEC-72` name one colouring. The alternative keeps `parity` for the reversal parity colouring of Section 5.12 of `docs/design/render.md`, which is not valid on every domain, so `info` would have to list it apart from the valid colourings (Section 4.6).
14. `1a` stays accepted on every domain as the colouring with one colour, although `info` lists it only for `1`, so that command lines with `1a` keep working (Section 4.6). The alternative accepts only `one`.
15. The line `Parity colouring` is withdrawn, because rows 18 and 19 show whether `parity` exists (row 13 of Section 5.1). The alternative keeps it with a new condition, which needs the search of 2 colours on every domain that is not all-reversing.
16. `--border-unit` is an option of its own beside `--border-width`, and the width keeps its limits in both units (`DEC-78`), so the parser of the width stays as it is. The alternative is a unit suffix in the value of the width, such as `0.05i`, which needs a private parser (Section 4.6).
17. The regular tiling line is computed in this crate from the public accessors of `Signature`, so `escher-signature` keeps its public surface (`DEC-83`, Section 5.9). The alternative is a public method of `Signature`, which a later caller other than `info` could share.
18. `render-masks` takes only `--size`, `--colouring`, `--set`, `--wythoff`, and `--arrangement`, and renders its masks with the fixed sample grid of `Masks::render` on the calling thread, because a mask depends only on the domain, the colouring or the arrangement, and the size, and runs no fold (Section 4.8). The alternative adds `--samples`, with a sample grid argument of `Masks::render`, and `--threads`.
19. `render-masks` writes each mask as soon as it is encoded, so it holds one mask in memory, and a failed write leaves the masks of the lower colours (CLI-59). The alternative encodes every mask before it writes any, which holds up to 64 encoded masks in memory and still leaves earlier files when a later write fails.
20. An image item names its file after `image:` in `--colours`, so a path cannot hold a comma (Section 4.6). The alternative is an option of its own for each image, such as `--image 2=fill.png`, which needs a second rule for a colour given twice.
21. An image item makes `render` draw `one`, `1a`, and `parity` with the coset fill (CLI-61), so `Fill::One` and `Fill::Parity` of the library take no image (`DEC-91`).
22. The text of a bad colour item names the image form, `#rgb, #rrggbb, or image:FILE`, which changes the snapshot of that diagnostic (Section 6.8). With `--wythoff`, which takes no image item, the text names only `#rgb or #rrggbb`, as the text of a bad `--border-colour` does (CLI-63).
23. The marks are an option of `render`, not a command of their own, so every other option of `render` applies to a construction unchanged (`DEC-100`). The alternative is a command such as `escher wythoff`, which repeats the options of `render`.
24. The marks are digits in the order of the corners of the canonical signature, not the side numbers of the domain, because `escher info` prints the corners and not the sides (`DEC-99`, Section 4.6). The alternative names the sides by their numbers in the domain.
25. `000` is a render error with the status `Failure`, not a usage error, because whether marks give a face depends on the signature, as `100` on `*n22` shows (Section 4.6).
26. `--wythoff` with `--colouring` is refused, not ignored, because a user who gives both expects a colouring that the image would not show (`DEC-105`).
27. `info` prints no line about the markings of a Wythoff construction, because the help of `--wythoff` and the line `Regular tiling` already tell the user which marks give the regular tiling and its dual (`DEC-108`). The alternative is a line that lists the accepted markings of a reflection triangle. With `--wythoff`, `info` prints the arrangement lines of Section 5.10, which `DEC-119` amends `DEC-108` with.
28. `--wythoff` of `info` chooses the tiling whose arrangements it lists, with the value of `--wythoff` of `render`, so the user types the same marks in both commands (`DEC-119`). The alternative is an option of its own, such as `--arrangements <MARKS>`.
29. `--arrangement` is one option with an id, refused without `--wythoff` at step 4 with a render error, as `--wythoff` with `--colouring` is (CLI-62), and not with the `requires` of `clap`, which would give a usage error (`DEC-119`). The alternative reuses `--colouring` for the id when `--wythoff` is given, which gives one option two kinds of id.
30. An arrangement id has the form of a colouring id, with the index of the arrangement as its number, so the parser, the ranking, and the text of `ColouringId` serve it unchanged (`DEC-115`). The alternative is ids that count from 1, which need a grammar of their own.
31. `render-masks` writes the one face mask to the value of `--output` itself, and not by the names of CLI-58, because an arrangement with one picture has one mask (`DEC-118`). The alternative names it `face-0.png` from `-o face.png`, as the mask of colour 0.
32. With `--wythoff` and no `--arrangement`, an image item stays refused with the text of a bad colour (CLI-63), so the text names no `--arrangement`. The alternative is a text of its own that tells the user to give `--arrangement`.
33. The number of pictures comes from the number of the id, so `render` counts the items of `--colours` at step 4 before any search, as it does for a colouring id (Section 4.5). An id whose number is not a multiple of $2n$ is read as an id with one picture and resolved as in `M24`, so its diagnostics are those of an id with one picture (CLI-66).
34. The listing calls `free_classes_of_index` once for each number of pictures, so the ids of each index are those of the call that `render` makes for that index, and a stop at one index leaves the lists of the lower indices complete (CLI-70). The alternative, one call with a bound of $2nM$ that keeps every index, would need a new function of `escher-group` and could stop before it completed any index.
35. The face masks of several pictures take the names of CLI-58 with the number of the picture, and the one face mask of one picture keeps the value of `--output` itself (CLI-69, CLI-71), so no command of `M24` writes another file.
36. The listing stops after the first search that stops at a limit, and runs no search at a larger index, because the search reaches its budget sooner at a larger index, so a later search would most often spend its budget and claim nothing (CLI-70). The alternative runs every search up to $M$ and prints the stopped form for each that stops, which can spend up to $M$ budgets.
37. `render` checks the position of `--centre` by building a renderer with the placeholder fill at step 5, so a position that the domain does not have stops before the searches of step 6, and `escher-render` stays the one place that reads the vertices of the domain (CLI-72). The alternative compares the position with the number of angles of `corner_angles()` in this crate, which CLI-24 rules out.
38. `--centre` has a long help with the example of `*732`, as `--set` has one with its parameters, because the owner asked that the help document clearly how a corner is named (`DEC-131`). The short help names the line of `info` and the corner of order 7 of `*732`, and points to `--help` for the example.
39. With several face types, the listing keeps rows 36 to 38: row 36 holds the least index, each row 38 a further index with its total of pictures, and each row 37 names the pictures of each type with their numbers of sides, so the regular listings keep every line and the new listings add no row (`DEC-143`, Section 5.10). The alternative is a count line for each index that names the pictures of each type, which adds a row.
40. The pictures of several face types are named by the position of their corner and their letter, such as `1B`, and the masks and the items of `--colours` follow their order, so a user reads the order of the files and of the items from the listing (`DEC-141`, `DEC-144`). The mask files carry the picture names, such as `tile-1B.png`, by CLI-76, a rule beside CLI-58 that point 62 of Section 11.3 puts to the owner.
41. The `--arrangement` parser reads ids up to 84 for every construction through `ColouringId::parse_arrangement`, because the parser cannot know the construction, and CLI-75 states the one exception that this makes for a regular construction (CLI-65). The alternative is a second parser for regular constructions, which `clap` cannot choose before the marks are read.

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
18. Withdrawn on 2026-11-22.
19. `--colouring parity` for a domain without a valid colouring with 2 colours is an error and writes no file (CLI-35, `DEC-32`). The alternative draws the image in the first colour.
20. The fill colours follow the colouring that the domain gives, so `escher render 1 --colours '#e9c46a,#2a9d8f'` fails, because the default colouring of `1` is `one`. The alternative chooses the colouring from the number of colours given.
21. Both presentations with `--presentation` (`DEC-35` of `docs/design/architecture.md`), so the two independent readings of the group can be compared. The alternative prints the Conway presentation only, which drops the check of the domain.
22. No order line outside the sphere (CLI-39). The alternative prints a line that says that the enumeration was not run, which changes the text of every Euclidean and hyperbolic signature.
23. No option for the coset limit (CLI-41). The alternative is an option such as `--max-cosets`.
24. The gate of the enumeration at a group order of 16384, near which `escher info --presentation` takes seconds. The largest group order of a domain that `escher-domain` builds is 400, of `*(100)22`, so the alternative is a lower gate, such as 4096, which keeps every built domain and prints `not run` for the larger orders.
25. Exit status 0 for an enumeration that reaches the coset limit (CLI-40), because no source guarantees that an enumeration finishes within the group order (Section 7.15 of `docs/research/colourings.md`). Such a run within the gate may indicate a defect, such as a dropped relator, so the alternative is exit status 1.
26. Withdrawn on 2026-11-22, when the owner decided on one valid colouring for each conjugacy class (`DEC-37`).
27. The default colours (CLI-44). Entries 3 to 12 were not checked for colour-vision deficiency, and a colouring with more than 12 colours needs `--colours`. The alternative is other or more entries after such a check.
28. No option for the step budget of the colouring search, so a search near the top of the range of `--max-colours` can stop at the budget for signatures with corner orders near 100. The alternative is an option for the budget, which adds an input whose cost is not bounded.
29. A search that stops at a limit leaves the status of `info` unchanged (CLI-47), while `render` with an id gives `Failure`. The alternative for `info` is a status of its own, which a script would have to tell from a failed check.
30. `--colouring parity` keeps `Fill::Parity` on an all-reversing domain, so its default colouring runs no search (CLI-48). The alternative draws `parity` through its id `2a` on every domain, which runs a search for every default image.
31. A higher colour limit can complete fewer colour counts than a lower one (Section 5.8). When the one run at the higher limit exhausts the budget, the runs by index share one budget (Section 7.14 of `docs/design/group.md`, GRP-35), while the one run at the lower limit can end within it. Every line still claims only what its search completed (CLI-47). The alternative that would remove it is left to the owner.
32. The default colour limit of 12 also on the reflection polygons and their doubles, whose default search takes several tenths of a second or more in a release build. The alternative is a default colour limit that falls with the number of generators of the side presentation, which makes the default text depend on the domain.
33. The colouring ids of the doubles of the reflection polygons (`DEC-50` of `docs/design/architecture.md`) hold only until the roadmap item `M15`, which builds the general hyperbolic signatures (Section 4.1 of `docs/design/roadmap.md`), renumbers the sides of their domains, and no line marks them as provisional. The alternative marks them in the `Domain` line or refuses an id for such a signature.
34. Shared default pictures (`DEC-55` of `docs/design/architecture.md`), as Section 4.5 states, which a user tells apart by a colouring id. Point 40 of Section 12.3 of `docs/design/render.md` holds the alternative, which draws mirror sides differently from paired sides.
35. The third form of the `Domain` line with a `Mirror sides` line (CLI-51). The alternative lists the kind of every side in one line, which changes the text of every reflection domain and rotation domain.
36. Tests kept for a reason that does not apply belong to TQ-11 of `docs/design/tooling.md`, which puts them to the owner.
37. Withdrawn on 2026-11-22.
38. The pictures of the spherical groups (`DEC-61` of `docs/design/architecture.md`). With the default colouring, `5x` renders the image of `*55` and `x` that of `*`, and with `--colouring one`, `3*2` renders the image of `432`. For the orders that Section 5.15 of `docs/design/render.md` treats, no picture of a colouring id tells `n*` from `n22` or `nx` from `*nn`. The default images of `*`, `x`, and `1` differ only in a thin rim, so a user who renders `*` sees no mirror until `--rotation` turns the ball. Points 42 and 43 of Section 12.3 of `docs/design/render.md` hold the alternatives.
39. Withdrawn on 2026-11-22, when the owner decided that `--model` has no effect on a Euclidean or spherical signature and gives no diagnostic (`DEC-77`).
40. Withdrawn on 2026-11-23, when the owner decided that the default border width is 0.05 with `--border-unit intrinsic` and stays 1 with `pixels` (`DEC-79`).
41. The regular tiling line only for `*pq2` and `pq2` with finite $p \ge q \ge 3$ (`DEC-81`, Section 5.9), the range for which Section 5.1 of `docs/research/colourings.md` states the symmetry group of the tiling. The alternative also prints `{n,2}, dual {2,n}` for `*n22` and `n22`, and a symbol with an infinite order, such as `{(inf),3}` for `*(inf)32`.
42. The larger order first, as `{7,3}, dual {3,7}` for `*732`, and `self-dual` when the two orders are equal (`DEC-82`, Section 5.9). The alternative puts the smaller order first, or prints `{4,4}, dual {4,4}` for `*442`.
43. The names of the mask files (`DEC-87`, CLI-58): the stem of `--output`, `-`, and the colour from 0 with no leading zero, so `mask-10.png` sorts before `mask-2.png` in a listing by name. The alternatives are numbers with leading zeros to the width of the largest colour, or numbers from 1.
44. Withdrawn on 2026-11-23, when the owner decided that an image fill has only the size limits of an output image, with no budget shared by the images of one render (`DEC-92`).
45. The printed form of an arrangement (`DEC-119`, Section 5.10), such as `12a, turned only, edges 0 and 3 turned, 1 and 4 turned, 2 and 5 turned`, with the edges numbered counterclockwise around the face mask from edge 0, the edge with the tick (`DEC-123`). The alternatives are a shorter form, such as `0-3t`, or a line for each edge.
46. The option name `--arrangement` and the use of `--wythoff` on `info` (`DEC-119`). The alternatives are other names, such as `--picture` or `--whole-face`.
47. The regular tilings whose index $2n$ is above 64, those with faces of more than 32 sides, such as $\{33, 3\}$ of `*(33)32` with `010`, are refused through the index limit of `escher-group`, which this design adds no limit to. The alternative is a higher index limit for this search, which the owner decides, because no measurement of the search at such an index exists. The arrangement index limit of 84 of `DEC-139` applies to the constructions with several face types only, so this refusal stays, which point 59 puts to the owner.
48. Withdrawn on 2026-11-23, when the owner decided that the face mask marks edge 0 with a short black tick at its midpoint, so that a user can place the edge numbers of an arrangement line on the mask (`DEC-123`).
49. How many pictures `info` lists (`DEC-126`, `DEC-129`, CLI-70): every number $m$ up to the colour limit of `--max-colours` whose index $2nm$ is within the index limit of 64, up to the first search that stops at a limit, so `--max-colours` bounds the pictures as it bounds the colours. With the default of 12, a listing of triangles searches up to 10 pictures, can hold many lines, and on some hyperbolic tilings spends the whole step budget of its last search before it stops. This reads the owner's "at least one per colour in the colouring" as the likeness of a picture of an arrangement to a colour of a colouring, which the owner may mean otherwise. The alternatives are a new option for the number of pictures, which this design avoids, the index limit alone, which lists up to 10 pictures for triangles whatever `--max-colours` says, or a shorter default for the listing, such as listing the arrangements with at most 3 pictures unless `--max-colours` is given, which keeps a plain `escher info ... --wythoff ...` short and quick at the cost of a default that differs from that of `--colourings`. `M26` adds to it the arrangements of the constructions with several face types at the multiples of their least index up to 84, whose least index is listed whatever `--max-colours` says (CLI-74).
50. The printed form of an arrangement with several pictures (`DEC-126`, Section 5.10): `, <m> pictures` after the id, and each edge written as the letter of its picture and its number, such as `A2 and B0 turned`, with the form of one picture unchanged. The alternatives are a letter on every edge also for one picture, which changes every line of `M24`, or one line for each picture.
51. The names of the pictures, A, B, C, in the listing and the diagnostics, with the masks numbered from 0 in their file names and the items of `--colours` counted from 1 in the diagnostics (`DEC-127`). The alternatives are mask files named with the letter, such as `face-A.png`, which adds a naming rule beside CLI-58, or pictures numbered from 0 in the listing.
52. The diagnostic that an id does not exist names the number of pictures also for one picture, `the tiling has 7 arrangements with 1 picture` (`DEC-128`), so its snapshot of `M24` changes, because without the clause the count would read as every arrangement of the tiling. The other diagnostics of one picture keep the texts of `M24`. The alternative keeps the text of `M24`, in which `arrangements` means those with one picture.
53. Row 36 and the lines of one picture keep the form of `M24`, and each further number of pictures follows with a count line of its own, such as `Arrangements with 2 pictures: <k>` (Section 5.10), so a listing of `M24` only gains lines after its last line. The alternative is one count line for all numbers of pictures, which changes the line `Arrangements` of every listing of `M24`.
54. The positions of `--centre` (`DEC-134`, Section 4.6): a decimal integer counted from 0, as the sides and edges are, for every vertex of the domain in the order of the line `Corner angles in degrees`, flat vertices included, with a refusal for a position that the domain does not have and for the domain of `1`. The alternatives are positions counted from 1, which differ from every other number that `info` prints, only the vertices that are not flat, which needs a rule to say which angles count, or the domain centre for the domain of `1` with no diagnostic.
55. The help text of `--centre` (`DEC-131`, Section 4.2): a short help that names the line of `info` and one example, and a long help with the line of `*732`, the corner of each position, two commands with `--wythoff 010`, and the refusals. The owner asked that the help document the option clearly and has not seen this text. The alternative is a line of `info` that prints the position of each corner, which would change the information text of every signature with a domain.
56. What `--max-colours` bounds with several face types (`DEC-143`, CLI-74): the total number of pictures over all types, which is the number of images that a user paints and passes to `--colours`. The alternative bounds the largest number of pictures of any one type, which lists more indices for the same value.
57. The default colours with more than 12 pictures (`DEC-144`, CLI-75): `render` stops and asks for as many colours with `--colours`, as for a colouring with more than 12 colours. The alternative is more default colours, after the check of point 27.
58. The printed form of an arrangement of several face types (`DEC-143`, Section 5.10): the total of pictures, the picture names of each type with ` to ` and ` of <n> sides`, and each edge written as its picture name and number, such as `1B2`. The alternatives are a line for each type, or a separator between the picture name and the edge number, such as `1B.2`.
59. The limit of 84 for several face types only (`DEC-139`): the regular constructions keep 64, so their listings, ids, and diagnostics stay as they are, and a regular tiling with faces of 33 to 42 sides stays refused. The alternative applies 84 to them too, which lengthens their listings and changes the diagnostic of point 47.
60. The ids from `65a` to `84zzzz` on a regular construction (CLI-65, CLI-75, choice 41 of Section 11.2): they parse, and step 4 refuses them with the error of the index limit. The alternative keeps the usage error for them, which needs the construction before the parser runs.
61. The least index is always listed (`DEC-143`, CLI-74), also when its total of pictures is above the colour limit, such as the 41 pictures of `*732` with `111`. The alternative lists nothing above the colour limit and prints a count line that says so.
62. The names of the mask files of several face types (`DEC-144`, CLI-76): each file carries its picture name, such as `tile-0A.png` and `tile-2B.png`, so a user finds the mask of a picture among up to 63 files by the name that the listing prints. The alternative numbers the files from 0, as for a regular construction, such as `tile-0.png` to `tile-40.png`, which keeps one naming rule but makes a user count to find a picture.

## 12. Security Considerations

The threat model is a user who passes a signature, option values, an output path, or an image file from an untrusted party to `escher info`, `escher render`, or `escher render-masks`. The attacker can choose any argument within the limits of the command line, including invalid UTF-8, control characters, and very long text. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory or time through the signature or the options of `render`, injection of terminal escape sequences through the diagnostic, and the choice of the file that `render` overwrites. `escher-signature` sets the limits on the length of the signature, the number of features, and the size of an order, as Section 16 of `docs/design/signature.md` states. `escher-domain` limits the largest corner order of a domain, which bounds the cost of `verify()` and of every fold step, as Section 16 of `docs/design/domain.md` states. `escher-render` limits the image size, the number of pixels, the sample grid, the border width, and the fold cap, which bound the memory of the image and the work of each sample, as Section 13 of `docs/design/render.md` states. `escher-group` limits the size of a presentation and the cosets of an enumeration (Section 15 of `docs/design/group.md`, GRP-19 and GRP-20). The coset limit bounds the memory of an enumeration and not its time, so `--presentation` runs an enumeration only for a spherical signature with a group order of at most the limit (CLI-39), whose relators the signature bounds. `escher-group` bounds the colouring search by its index limit, its step budget, and its limit on the number of colourings (the rules of the search in `docs/design/group.md`), so `--max-colours` and a colouring id bound its time and memory, and a search that reaches a limit gives a text or a diagnostic, never a run without end. This crate adds one limit of its own, 256 threads (Section 4.6). `escher-render` checks the size of each image from its header against the limits of an output image before it allocates the texels (RND-62 of `docs/design/render.md`), so the texels of one image take at most $2^{28}$ bytes. A render reads at most one image for each colour, and a colouring has at most 64 colours, the largest index of `escher-group`, so the texels of all images of one render take at most $2^{34}$ bytes. The owner decided that the images have no further limit (`DEC-92`). With `--wythoff`, a render reads no image and runs no colouring search, so the option adds no limit. With `--arrangement`, a render reads one image for each picture, at most 10, and `render` and `render-masks` run one search of the arrangements, which the index limit, the step budget, and the stored limits of `escher-group` bound, so the option adds no limit either. `info --wythoff` runs one such search for each number of pictures up to $M$, at most $\lfloor 32/n \rfloor$ searches and so at most 10, each with its own budget, and stops after the first that stops at a limit, so its time is bounded by those searches (CLI-70). The number of its lines grows with the number of pictures, and it has no limit of its own. `--max-colours` lowers both. `render-masks` with `--arrangement` writes one file for each picture, at most 10. With several face types, an arrangement has an index of at most 84 (GRP-47 of `docs/design/group.md`), so a render reads at most 63 images and `render-masks` writes at most 63 files, and `info --wythoff` runs at most one search for each multiple of the least index up to 84, at most 21 because the least index is at least 4, each with its own budget, and fewer under the colour limit (CLI-74). `render-masks` writes at most one file for each colour. `escher-domain` limits the value of each shape parameter to its range. The settings need no limit on their number or length, because each name is only compared, and a repeated or unknown name fails. The diagnostic of an unknown name echoes the whole name, with no cap like that of the echo line, so its length is bounded only by the command line.

The diagnostics of `render` repeat no path and no option value, except the three cases of CLI-26. The colouring lines hold only ids, fixed words, and numbers. The echo line repeats at most 120 characters and replaces every character outside printable ASCII and the infinity sign (CLI-9). The caret line holds only spaces and `^`. The `Domain` line and the diagnostics of Section 6.7 hold only fixed text, numbers, and the canonical signature, which is printable ASCII. The help, version, and usage text that `clap` produces has every control character other than the line feed replaced with `?` (CLI-15).

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, and the workspace denies `unsafe_code`.
- Files: `render` reads the file of each image item, at the path as given, with the permissions of the user, and nothing else (CLI-60). `info` reads and writes no file. `render` writes one file, the output file, with the permissions of the user, and only after rendering and encoding succeed (CLI-21). `render-masks` writes the mask files of CLI-58, or the face masks of CLI-69, CLI-71, and CLI-76, and no other file, in the directory of `--output`, only after every check has passed (CLI-59). Both replace an existing file without asking (`DEC-30`), follow a symbolic link at the path (Section 11.1), and create no directory. A caller who passes a path from an untrusted party therefore lets that party choose which file of the user `render` reads or overwrites, or which files `render-masks` overwrites, and checking the path is the caller's task.
- Threads: `render` starts at most 256 threads, in a pool that ends with the call (CLI-22).
- Build scripts: the crate has none.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones (Section 9.3). The dev-dependencies `insta`, `proptest`, and `escher-reference` never reach the binary.

Attacks on the repository, the build machine, or the terminal itself are out of scope, because this document governs the crate and not access control or the terminal.

The residual risk is that a usage error still echoes every character of an unknown argument that is not a control character, including characters outside ASCII, such as the formatting characters of the Unicode general category Cf. The replacement tests only for the category Cc. A second residual risk is a domain that is wrong by less than the tolerances, which `--check` reports as passed (Section 16 of `docs/design/domain.md`).

The limits of `escher-render` bound memory, but at their maximum values together they allow a rendering that runs for a very long time, which only the user can stop. In the same way, `info --presentation` near the gate runs for seconds (Section 11.3, item 24), and every `info` on a built domain runs the colouring search, which its step budget bounds (Section 5.8). A write that fails after `std::fs::write` has opened the file, such as on a full disk, leaves the file truncated or partly written, and a failed write of `render-masks` also leaves the mask files of the lower colours. An image item that names a FIFO or a device, such as a FIFO that no program writes, makes `render` wait or read without end, because the decoder reads the file until the image ends and no limit applies to the length of the file (Section 13 of `docs/design/render.md`). No check of this repository looks for undefined behaviour in `rayon` (Section 9.3).

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
| `render` gets a bad colour or the wrong number of colours | The diagnostic of Section 6.8, with `Failure` | Give each colour as `#rgb`, `#rrggbb`, or `image:FILE`, quoted, as many as the colouring needs |
| `render` gets `--colouring parity` for a domain without a valid colouring with 2 colours | The diagnostic of Section 6.8, with `Failure` | Leave out `--colouring`, or give `--colouring one`. `escher info` prints no line `Colourings with 2 colours` for such a domain |
| `render` gets a colouring id that names no colouring of the domain | The diagnostic of Section 6.8, with `Failure` | Use an id that `escher info --colourings` lists for the signature |
| The search for a colouring id stops at a limit | The diagnostic of Section 6.8, with `Failure` | None in `render`, which has no option for the budget. Choose a colouring with fewer colours |
| The search for `parity`, or for the default colouring of a domain that is not all-reversing, stops at a limit | The diagnostic of Section 6.8, naming `parity`, with `Failure` | None in `render`, which has no option for the budget. Give `--colouring one`, which runs no search unless `--colours` holds an image item |
| `render` gets a colouring id with more than 12 colours and no `--colours` | The diagnostic of Section 6.8, with `Failure` | Give as many colours with `--colours` as the colouring has |
| The colouring search of `info` stops at a limit | The `Colourings` line names the last complete colour count, with the status unchanged | Use a lower `--max-colours`, whose lines are then complete |
| `render` gets an option value outside a limit of `escher-render`, a rotation angle that is not finite, or a light direction that is zero or not finite | The diagnostic of Section 6.8, with `Failure` | Choose a value within the limits of Section 4.1 of `docs/design/render.md` |
| `render` gets `--centre` with a number at or above the number of corners of the domain, or any `--centre` for `1`, whose domain has no corner | The diagnostic of Section 6.8, with `Failure` | Run `escher info` on the signature, count the angles on its line `Corner angles in degrees` from 0, and give the number of the corner to centre on. Leave out `--centre` for `1` |
| `render` cannot write the output file | The diagnostic of Section 6.8, with `Failure` | Fix the path or its directory, and run the command again |
| `render` cannot open or decode the file of an image item, or the image is over a limit | The diagnostic of Section 6.8, naming the position of the item, with `Failure` | Fix the path, save the image as PNG again, or scale the image down |
| `render` with an image item, or `render-masks`, gets a domain without a mask polygon | The diagnostic of Section 6.8 or 6.10, with `Failure` | None. Use colours on such a domain |
| `render` gets `--wythoff` for a signature that is not a reflection triangle, marks that give no face, or `--wythoff` with `--colouring` | The diagnostic of Section 6.8, with `Failure` | Use a reflection triangle, turn on a mirror that gives a face, as Section 4.6 states, and leave out `--colouring` |
| `render --wythoff` gets the wrong number of colours, or an item that is not a colour, such as an image item | The diagnostic of Section 6.8, with `Failure` | Give one colour for each face type, as `#rgb` or `#rrggbb`, in the order of the corners that have a face |
| `render --arrangement` or `render-masks --wythoff` gets a construction with one face type that is not a regular tiling, `--arrangement` without `--wythoff`, or `--wythoff` without `--arrangement` for `render-masks` | The diagnostic of Section 6.8 or 6.10, with `Failure` | Use `010` or `100` on `*pq2` with $p \ge q \ge 3$, or marks with two or three face types, and give both options |
| A construction with several face types has a least index above 84 | `Arrangements: none` with the error of the arrangement index limit in `info`, and the diagnostic of Section 6.8 or 6.10 with `Failure` | None. Its arrangements cannot be searched, so no whole picture can be placed on it |
| `render --arrangement` without `--colours` gets an arrangement with more than 12 pictures | The diagnostic of Section 6.8, with `Failure` | Give one colour or image for each picture with `--colours`, in the order of the picture names |
| The search finds no arrangement with the number of pictures of the id, the id names none, or the index is above the index limit | The diagnostic of Section 6.8 or 6.10, with `Failure` | Use an id that `escher info --wythoff` lists, with as many pictures as it needs. Where the listing holds none, no whole picture can be placed |
| `render --arrangement` gets another number of items of `--colours` than the id has pictures | The diagnostic of Section 6.8, with `Failure` | Give one colour or image for each picture, in the order A, B, C |
| `render-masks` cannot write the face mask of a picture | The diagnostic of Section 6.10, naming the picture, with `Failure`. The masks of the earlier pictures stay written | Fix the path or its directory, and run the command again |
| The search for an arrangement stops at a limit | The diagnostic of Section 6.8 or 6.10, or `none, the search stopped at a limit` on the line `Arrangements` or on a line `Arrangements with <m> pictures` in `info`, after which the listing ends | None, because no option sets the budget. Choose an arrangement with fewer pictures |
| `info --wythoff` prints a long listing | The arrangement lines of Section 5.10, with `Success` | Give a lower `--max-colours`, which bounds the number of pictures listed |
| `render-masks` gets an output path that names no file | The diagnostic of Section 6.10, with `Failure` | Give a file name, such as `mask.png` |
| `render-masks` cannot write a mask file | The diagnostic of Section 6.10, naming the colour, with `Failure`. The masks of the lower colours stay written | Fix the path or its directory, and run the command again |
| Samples reach the fold cap | No error. `Unconverged samples` counts them with `--stats`, and they take the border colour (`DEC-7`) | Raise `--max-folds` |

The crate emits no log. `render --stats` prints the statistics of a rendering on request.

## 14. References

### 14.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 14.2. Informative References

- [CBG] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [CBGS2008] The entry in Section 15 of `docs/research/colourings.md`. It is the same book as [CBG].
- [CH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [CLM1954] The entry in Section 15 of `docs/research/colourings.md`.
- [YUNCKEN2003] The entry in Section 15 of `docs/research/colourings.md`.

## Appendix A. Requirement Summary

This table lists every labelled requirement of this document with a one-line summary.

| Label | Section | Summary |
| --- | --- | --- |
| CLI-1 | 3.1 | The library target exposes `run` and `Status` and nothing else. |
| CLI-2 | 3.2 | `run` writes only to its two streams, the output file of `render`, and the mask files or the face masks of `render-masks`, and never panics. |
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
| CLI-24 | 4.6 | `render` passes the size, grid, border width and unit, fold cap, light direction, rotation in radians, hyperbolic model, and position of `--centre` to `escher-render` unchanged and checks no limit of them itself. |
| CLI-25 | 4.7 | `render` writes to standard output only the statistics lines, with `--stats`, after the output file is written. |
| CLI-26 | 6.8 | No diagnostic of `render` or `render-masks` repeats an option value, the output path, the path of an image item, or the name of a mask file, except a colouring id under CLI-49, the echoed name of a setting that names no parameter, and the parsed value of a setting outside its range. |
| CLI-27 | 8.1 | Withdrawn on 2026-11-19. |
| CLI-28 | 8.1 | No snapshot holds the value of a statistics line other than `Samples`. |
| CLI-29 | 4.5 | `render` passes every setting to `escher-domain`, which alone decides whether the settings are accepted, and reads the parameters of the signature only to name a rejected setting. |
| CLI-30 | 4.5 | Withdrawn on 2026-11-17. |
| CLI-31 | 4.6 | `--rotation` and `--light` accept a value that starts with `-`. |
| CLI-32 | 5.5 | `info` prints the number of pairs in the `Domain` line of a domain with a paired side, and one `Side pairs` line with the words `rotation`, `translation`, and `glide reflection`. |
| CLI-33 | 5.5 | Withdrawn on 2026-11-22. |
| CLI-34 | 4.6 | Without `--colouring`, `render` uses `parity` on an all-reversing domain, and elsewhere `parity` or `one` as the search of 2 colours finds, failing with no file when that search stops at a limit. |
| CLI-35 | 4.5 | `render --colouring parity` for a domain without a valid colouring with 2 colours fails at step 6 with its diagnostic and writes no file. |
| CLI-36 | 4.5 | Withdrawn on 2026-11-17. |
| CLI-37 | 5.7 | `--presentation` adds rows 20 to 27 under their conditions and in their forms, and nothing appears without the flag. |
| CLI-38 | 5.7 | The generator names and relator texts are those of `escher-group`, unchanged. |
| CLI-39 | 5.7 | An order line appears only for a spherical signature, and no enumeration runs when the group order is above the coset limit. |
| CLI-40 | 6.9 | An index other than the group order gives the diagnostic and `Failure`, and every other outcome leaves the status unchanged, including a run that an input limit refuses, whose order line reads `not run, ` and the error text. |
| CLI-41 | 5.7 | `info` enumerates with `CosetLimit::DEFAULT` and has no option for the limit. |
| CLI-42 | 4.6 | `--colouring` accepts exactly `one`, `parity`, and what `ColouringId` parses, and gives a usage error for every other value. |
| CLI-43 | 4.5 | `render` resolves an id other than `1a`, and `parity` off an all-reversing domain, through the side presentation of its domain and the lookup of the valid colourings of its colour count only, and fails with a diagnostic and no file when the id does not exist or the search stops at a limit. |
| CLI-44 | 4.6 | Without `--colours`, a colouring takes the first entries of the 12 default colours, and a colouring with more than 12 colours fails with a diagnostic that asks for `--colours`. |
| CLI-45 | 5.8 | `info` prints the colouring lines of the valid colourings for every built domain, with counts, ids, and their order from `escher-group` unchanged, and the aliases `one` and `parity` on exactly `1a` and `2a`. |
| CLI-46 | 5.8 | `--max-colours` sets the colour limit of every colouring line, and with `--colourings` the counts and the list come from one call. |
| CLI-47 | 5.8 | A search that stops at a limit gives the text of the last complete colour count and no line above it, and no outcome of the colouring lines changes the status. |
| CLI-48 | 4.6 | Without an image item, `one` and `1a` draw `Fill::One`, `parity` draws `Fill::Parity` on an all-reversing domain and the coset fill of `2a` elsewhere, and `2a` gives the bytes of `parity`. |
| CLI-49 | 6.8 | A diagnostic names a colouring id only by the `Display` text of its parsed value. |
| CLI-50 | 8.1 | A snapshot of `info` on a reflection polygon or its double sets `--max-colours` to at most 4, to at most 3 for a polygon with 7 or 8 corners or the double of a polygon with 6 or more corners, and to at most 2 with `--check` or `--presentation`. It prints only the default shape or values that meet the rounding condition of CLI-20. |
| CLI-51 | 5.5 | `info` chooses the form of the `Domain` line from the number of mirror sides, not from `reverses_orientation()`, and prints the `Mirror sides` line before the `Side pairs` line for a domain with both kinds of side. |
| CLI-52 | 8.1 | A snapshot of `info` on a wallpaper domain uses the default colour limit of 12, unless its measured debug time exceeds that of the snapshots of `*2222`, and prints only the default settings. |
| CLI-53 | 5.5 | For a domain with no side, `info` prints `Domain: no sides` and no `Corner angles in degrees` and `Side lengths` lines, and the `Side pairs` line names `rotary reflection` for `PairingKind::RotaryReflection`. |
| CLI-54 | 8.1 | A snapshot of `info` on a spherical group domain uses the default colour limit of 12 and holds only inputs whose printed values meet the rounding condition of CLI-20. |
| CLI-55 | 4.6 | `--model` accepts exactly `disk`, `half-plane`, `band`, and `klein`, defaults to `disk`, and passes the variant of the same name to `escher-render` for every signature. |
| CLI-56 | 4.6 | `--border-unit` accepts exactly `pixels` and `intrinsic`, defaults to `pixels`, makes the default width 1 with `pixels` and 0.05 with `intrinsic`, and passes the variant of the same name to `escher-render` for every signature. |
| CLI-57 | 5.9 | `info` prints the regular tiling line exactly for the canonical signatures `*pq2` and `pq2` with finite $p \ge q \ge 3$, with the value of Section 5.9, computed from the canonical signature alone. |
| CLI-58 | 4.8 | `render-masks` names the mask of colour $c$ by the stem of `--output`, `-`, $c$ in decimal, and the extension, and fails with no file when the path has no stem. |
| CLI-59 | 4.8 | `render-masks` changes no mask file before every check has passed, then writes the masks in colour order, replacing existing files and creating no directory. |
| CLI-60 | 4.6 | `render` opens each image item at the path as given, decodes it only through `Image::decode_png`, fails at step 4 with no file when it cannot, and names the image only by its position. |
| CLI-61 | 4.6 | With an image item, `render` draws every colouring with the coset fill, `one` and `1a` as `1a` of `ColouringSet::All`, and `parity` as `2a` on every domain. |
| CLI-62 | 4.6 | `render` accepts as `--wythoff` exactly three digits `0` or `1`, passes digit $k$ as mark $k$, and refuses `--wythoff` with `--colouring`. |
| CLI-63 | 4.6 | With `--wythoff`, `render` draws the Wythoff fill with one colour for each face type, refuses an item that is not a colour, an image item included, takes the first default colours without `--colours`, and runs no search. |
| CLI-64 | 4.8 | Withdrawn on 2026-11-23. |
| CLI-65 | 4.6 | `--arrangement` accepts exactly what `ColouringId::parse_arrangement` parses, gives a usage error otherwise, and is refused without `--wythoff`. |
| CLI-66 | 4.6 | An arrangement id is resolved through one call of `free_classes_of_index` with the words of the whole face, at the index that its number gives when it is a multiple of $2n$ and at $2n$ otherwise, and a missing arrangement, an index over the limit, or a stopped search fails with no file and no claim beyond the search. |
| CLI-67 | 4.6 | With `--arrangement`, `render` takes exactly one colour or image item for each picture of the id, in the order of the pictures, the first default colours without `--colours`, and draws the whole-face fill. |
| CLI-68 | 5.10 | With `--wythoff`, `info` prints the arrangement lines with ids, order, and edge pairing from the libraries unchanged, row 36 and the lines of one picture byte for byte as in `M24`, the lines of more pictures after them, no number and no further line after a stopped search, and they never change the status. |
| CLI-69 | 4.8 | With `--wythoff`, `render-masks` requires `--arrangement`, and for one picture writes the one face mask to the value of `--output` after every check. |
| CLI-70 | 5.10 | With `--wythoff`, `info` searches each number of pictures from 1 up to the smaller of the colour limit and the largest that the index limit allows, one call each with the default budget, and stops after the first call that fails or is not complete. |
| CLI-71 | 4.8 | For an arrangement of a regular construction with several pictures, `render-masks` writes one face mask for each picture, named by CLI-58 with the number of the picture, in order, after every check. |
| CLI-72 | 4.6 | `--centre` accepts exactly a `u32`, passes it unchanged to `escher-render`, compares it with no corner itself, and `render` builds a renderer at step 5 so that a position that the domain does not have stops before any search. |
| CLI-73 | 4.6 | `render-masks` and `info` accept no `--centre`. |
| CLI-74 | 5.10 | With several face types, `info` searches the least index and each further multiple within 84 whose total of pictures is within the colour limit, stops after the first call that fails or is not complete, prints rows 36 to 38 in their forms, and leaves every regular listing as it was. |
| CLI-75 | 4.6 | With several face types, `render` and `render-masks` resolve the id at its index or at the least index, read one item for each picture in the order of the picture names, write the masks by CLI-76, and refuse more than 12 pictures without `--colours`. A regular construction keeps every listing and diagnostic, every mask and image that RND-87 of `docs/design/render.md` keeps, and the limit of 64, except that an id whose number is from 65 to 84 is refused at step 4 with the error of `IndexLimit::new`. |
| CLI-76 | 4.8 | With several face types, `render-masks` names the mask of each picture by the stem of `--output`, `-`, the picture name, and the extension, and writes the masks in the order of the picture names after every check, stopping at the first write that fails. |
| CLI-77 | 8.3.3 | The property test renders every valid signature that it generates or lists, asserts success with a PNG file of 8 by 8 pixels for the set, and the diagnostic of an unsupported signature or of a limit, with no file, for every other one, as its own oracle decides. |
| CLI-78 | 8.3.6 | The snapshots of `info` hold one snapshot for each class of the set and of refusal that Section 8.3.6 names. |
