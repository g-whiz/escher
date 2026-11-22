# Writing READMEs

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-22 |
| Applies to | The root `README.md`, and the `README.md` of each crate once a crate has its own |

## Abstract

This guide defines what the READMEs of this repository contain, in which order, and how they link and render on GitHub. Today the repository has one README at its root, which every crate uses through the inherited `readme` field of its manifest, and no crate is published. The root README helps a visitor on GitHub decide whether to use the command-line tool `escher`, build it, and run a first command. Each fact that users need has one home, and the README points to it instead of copying it. Some rules govern crate READMEs, crates.io, versions, README doctests, and hosted API documentation, and each of them applies from the first publication of a crate. The guide marks each rule as checked by a tool or by a reviewer.

## 1. Introduction

A README is the first page of the project that a visitor sees on GitHub. A visitor decides from it within minutes whether the project fits, so a README that buries its purpose or its first step loses the visitor.

The product is the command-line tool `escher` of the package `escher-cli`. Every crate of the workspace sets `publish = false` and inherits the workspace field `readme = "README.md"`, so the root README is the only README, and no crate has hosted API documentation.

A published crate changes this. The published copy of a crate README cannot be corrected without a new release, because Cargo transfers the file to the registry when the crate is published [CARGO-MANIFEST]. The rules that only a published crate needs stay in this guide with their identifiers, so that the change that publishes a crate renumbers nothing. Each such rule says that it applies from the first publication of a crate, and Appendix A marks it.

This guide governs the content, structure, examples, and rendering of every README. `docs/writing-documentation.md` governs their language and punctuation, as Section 8 of that guide defines.

## 2. Requirements Language

### 2.1. Key Words

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.2. Terminology

- root README: The file `README.md` at the root of the repository, which GitHub shows as the project page.
- crate README: A file `README.md` in the package directory of a crate, such as `crates/escher-render/README.md`, which Cargo packages and crates.io renders as the page of the crate. No crate README exists yet.
- README: The root README, or a crate README once one exists. This extends the term of Section 2.2 of the writing guide, which names only the root README.
- first publication: The first release that publishes a crate of this repository on crates.io. No crate has had one.
- home: The one place where a fact is stated in full.
- pointer: One sentence that names a fact and links to its home.
- alert: A block quote that starts with a marker such as `> [!WARNING]`, which GitHub and crates.io render as a highlighted box.
- status notice: The alert that states the status of the project or a crate.
- hosted API documentation: A rustdoc site that a workflow builds from each release tag. The repository has none, and the change that makes the first publication of a crate decides its host and address.

## 3. Readers and Purpose

A README is a landing page [YEGOR256]. It says what the project or crate is, why a reader would use it, how to take the first step, and where to go next, which are the questions that common guidance on READMEs asks [OSG] [GITHUB-README]. A classification of the content of GitHub READMEs lists the purpose of the project among the common kinds of content [PRANA].

RDM-1. Each README MUST answer, in this order, what its subject is, why a reader would use it, how to take the first step, and where to go next, for the reader of its kind. The reader of the root README is a visitor who wants to render tilings with `escher`. A README that answers these questions out of order, or for another kind of reader, loses the reader before the first step.

Reference material, tutorials, and explanation are distinct kinds of documentation, and Diataxis advises against mixing them [DIATAXIS]. The README therefore links to the design documents for reference material, such as every option of `escher` [MAKE-README] [18F].

Checked by: judgement.

## 4. Where Each Fact Lives

### 4.1. Homes

RDM-2. Each fact in Table 1 MUST be stated in full in its home only. Two statements of one fact diverge, and after the first publication of a crate, the copy in its published README cannot be fixed until the next release.

Table 1. Where each fact lives.

| Fact | Home | Elsewhere |
| --- | --- | --- |
| The status of the project: a proof of concept, nothing published, and commands, options, and output that can change in any commit | Section `Status` of the root README | From the first publication of a crate, the status notice of its crate README (Section 6.3) |
| Every command, option, output line, diagnostic, and exit status of `escher` | `docs/design/cli.md` | Commands and their output as examples in `Getting Started` and `Examples` of the root README, and a link in `Documentation` |
| The signatures that have a fundamental domain, and the shape parameters of each with their ranges and defaults | `docs/design/domain.md` | `Supported Signatures` of the root README, as a table by geometry with examples and the limits on corner orders, and a list of the signatures that take each shape parameter, without ranges or defaults |
| Minimum supported Rust version (MSRV) | `Requirements` of the root README, matching `rust-version` in the workspace manifest | None |
| Supported operating systems | `Requirements` of the root README | From the first publication of a crate, `Requirements` of its crate README states only what differs for it |
| Licence of the project | `Licence` of the root README, matching the `license` field of the workspace manifest | From the first publication of a crate, `Licence` of its crate README |
| The API, errors, and Cargo features of a library crate | Crate-level and item documentation of the crate | None |
| How the crates are built and why | Design documents under `docs/design/` | A link in `Documentation` of the root README |
| The mathematics that the code implements, with its sources | Research documents under `docs/research/` | A link in `Documentation` of the root README |
| The rules of the repository for contributors | `AGENTS.md` | A link in `Documentation` of the root README |
| Changes between releases | From the first publication of a crate, the GitHub release of each tag | None |

### 4.2. Pointers

RDM-3. Outside its home, a document MAY state a fact of Table 1 only in the form that the Elsewhere column gives. Where that form is a pointer or a link, the document MUST NOT restate details of the fact, such as a list of options, a range, a default, a limit, or a condition. A pointer keeps the reader one click from the full fact, while a partial copy reads as complete and goes stale without notice.

This illustration shows a pointer in the section `Documentation` of the root README:

```markdown
The [command-line design](https://github.com/g-whiz/escher/blob/main/docs/design/cli.md) describes every command, option, output line, and exit status of `escher`.
```

### 4.3. What Moves Out of a README

RDM-4. A README MUST NOT hold the list of Cargo features, API reference material, the full list of options of a command, error messages or troubleshooting steps, environment variables, or build internals. These belong in the homes of Table 1, where rustdoc checks the links and examples of API documentation and the snapshot tests of `escher-cli` fix its help text. A copy in a README is checked by nothing [YEGOR256].

Checked by: judgement, for Sections 4.1 to 4.3.

## 5. The Root README

### 5.1. Sections and Order

RDM-5. The root README MUST have the sections of Table 2, in that order, with exactly these headings, and MUST NOT add other sections. A link to a heading by its anchor breaks when the heading changes, and a reader who knows the README finds the same content in the same place after each change.

Table 2. Sections of the root README.

| Order | Heading | Presence | Content |
| --- | --- | --- | --- |
| 1 | `# Escher` | Required | The title, optionally followed by the badge of Section 8.6 |
| 2 | None | Required | The opening paragraph of Section 5.2 |
| 3 | `## Status` | Required | The status notice of Section 5.3 |
| 4 | `## Crates` | Required | The table of Section 5.4 |
| 5 | `## Requirements` | Required | The MSRV, what the first build downloads, and the supported platforms |
| 6 | `## Getting Started` | Required | The first step of Section 5.5, with the image of its first render (RDM-29) |
| 7 | `## Examples` | Optional | Commands of `escher` in the form that RDM-23 gives, each with the output that RDM-19 allows or the image that RDM-29 requires |
| 8 | `## Supported Signatures` | Required | The summary of the signatures and shape parameters that Table 1 allows |
| 9 | `## Documentation` | Required | Links to the design document of the command-line tool, the design documents, the research documents, and, for contributors, `AGENTS.md`. From the first publication of a crate, also a link to its hosted API documentation |
| 10 | `## Licence` | Required | The licence of the project, with a link to its text |

The licence comes last, as Standard Readme requires [STANDARD-README]. A Contributing section is absent while the project has no contribution policy. GitHub builds a table of contents from the headings [GITHUB-README], so the README has no table of contents of its own.

### 5.2. Opening

RDM-6. The first sentence of the root README MUST say what the project is by its category, in at most 120 characters, and the second sentence MUST say what it covers. A visitor reads the first sentence to decide whether to read on, and Standard Readme asks for a short description first [STANDARD-README].

This illustration shows the form of the opening:

```markdown
Escher is a command-line tool that renders symmetric tilings into PNG images. You name a symmetry in Conway's orbifold notation, and Escher draws its tiling in the Euclidean plane, on a shaded ball, or in the Poincare disk.
```

### 5.3. Status

RDM-7. The section `Status` MUST hold one alert of type `WARNING` that states that Escher is a proof of concept, that no crate or binary is published, and that the commands, options, and output can change in any commit. A visitor who expects a stable tool otherwise builds a script on output that the next commit changes.

From the first publication of a crate, the alert also states that the published crates are pre-releases and that a dependent names the full pre-release version, such as `"0.2.0-alpha.1"`. Cargo matches a pre-release only from a requirement that names a pre-release, so a dependent who writes `"0.2"` gets a resolution error.

### 5.4. Crates

RDM-8. The section `Crates` MUST hold one table with one row for each crate of the workspace and the columns Crate and Use it to. The Crate column links the crate name to its source directory on GitHub, and the Use it to column has one sentence. From the first publication of a crate, its row links to its crate README, and the table gains the column API documentation, which links to the page of each published crate on the hosted API documentation.

The rows start with `escher-cli`, which holds the tool that a visitor runs, followed by the libraries, each above every library that it depends on: `escher-render`, `escher-domain`, `escher-group`, `escher-signature`, and `escher-geometry`. The test support crate `escher-reference` comes last. A visitor scans the table top down from the product to its parts.
### 5.5. Getting Started

RDM-9. The section `Getting Started` MUST show, as commands in a `sh` block, how to build `escher` from a clone, and MUST show one command that renders a first image. Guidance on documentation asks that a new user reach a first success quickly [KAPLAN-MOSS] [WTD]. From the first publication of `escher-cli`, the section also shows the command that installs the published tool with its full version.

This illustration shows the build command:

```sh
cargo build --release -p escher-cli
```

Checked by: judgement, for Sections 5.1 to 5.5. A tool could check the headings and their order, but no configured tool does.

## 6. Crate READMEs

No crate README exists, and every crate uses the root README through the inherited `readme` field. Each rule of this section applies from the first publication of a crate, and only to the crate README of a published crate.

### 6.1. Sections and Order

RDM-10. From the first publication of a crate, its crate README MUST have the sections of Table 3, in that order, with exactly these headings, and MUST NOT add other sections. A reader who moves between the crates of this project finds the same content under the same heading on each page.

Table 3. Sections of a crate README.

| Order | Heading | Presence | Content |
| --- | --- | --- | --- |
| 1 | `# <crate-name>` | Required | The package name, exactly as in the manifest |
| 2 | None | Required | The opening paragraph of Section 6.2 |
| 3 | None | Required | The status notice of Section 6.3 |
| 4 | `## Installation` | Required | The snippet of Section 6.4 |
| 5 | `## Example` | Required | The example of Section 7 |
| 6 | `## Requirements` | Required when the crate needs more than Cargo | Pointers to what the crate needs beyond Cargo |
| 7 | `## Documentation` | Required | Links to the page of the crate on the hosted API documentation and to the root README |
| 8 | `## Licence` | Required | The licence of the crate and a link to its text |

### 6.2. Opening

RDM-11. From the first publication of a crate, the first sentence of its crate README MUST say what the crate is, with the meaning of the `description` field of its manifest, and the second sentence MUST say where the crate fits and which crate a reader uses instead for a different need. A reader who reaches a library from a search, such as `escher-geometry`, otherwise depends on the wrong crate when the tool `escher` is what the reader wants.

### 6.3. Status Notice

RDM-12. From the first publication of a crate, directly under the opening, its crate README MUST hold one alert of type `WARNING` of at most two sentences, which states the status of the crate and links to `https://github.com/g-whiz/escher#status`. A reader on crates.io does not see the root README, so without the notice the reader expects a stable API.

### 6.4. Installation

RDM-13. From the first publication of a library crate, the section `Installation` of its crate README MUST hold a `toml` block with the dependency table that the crate is meant for and one line that names the crate with the full version of its manifest. A requirement without the full version of a pre-release fails to resolve, as Section 5.3 explains.

The one exception is `escher-cli`, whose section `Installation` holds, in a `sh` block, the command that installs the tool `escher` with the full version of its manifest. The library target of `escher-cli` exposes only what its binary needs, so a dependency snippet would invite a use that the package does not support.

### 6.5. Content by Kind of Crate

RDM-14. From the first publication of a crate, its crate README MUST meet the row of Table 4 for its kind. Each kind has a different first step and a different crate that most readers want instead.

Table 4. Content of a crate README by kind of crate.

| Kind | Crates | The opening says | The example |
| --- | --- | --- | --- |
| Command-line tool | `escher-cli` | That the package installs the tool `escher`, which renders the tiling of a signature in orbifold notation into a PNG file | One `escher render` command in a `sh` block, instead of a Rust program |
| Renderer | `escher-render` | That the crate renders the tiling of a fundamental domain into an image, and that most users run `escher render` instead | Rendering one domain and encoding the image as PNG in memory |
| Domain library | `escher-domain` | That the crate builds and verifies the fundamental domain of a signature, and that `escher-render` draws it | Building the domain of one signature and printing a measurement of it |
| Group library | `escher-group` | That the crate works with group presentations, coset enumeration, and the colourings of a tiling, and that `escher-domain` gives the presentation of a domain | Building the Conway presentation of one signature and counting its colourings |
| Signature library | `escher-signature` | That the crate parses and classifies a signature in orbifold notation, and that `escher-domain` builds its domain | Parsing one signature and printing its canonical form and geometry |
| Geometry library | `escher-geometry` | That the crate computes in the hyperbolic plane, the Euclidean plane, and on the sphere, and that `escher-domain` builds domains from it | One distance or isometry in one geometry |

### 6.6. Manifest

RDM-15. From the first publication of a crate, its manifest MUST set `readme = "README.md"` to its own crate README and MUST NOT inherit the `readme` field of the workspace. Cargo packages the file that the field names and crates.io renders it as the page of the crate [CARGO-MANIFEST], so an inherited root README shows every crate the same project page. Until then, every crate inherits the field, so that Cargo finds a README for each crate.

Checked by: judgement, for Sections 6.1 to 6.6. A tool could check that each packaged crate holds a `README.md` as a regular file, but no configured tool does.

## 7. Examples

### 7.1. Form

RDM-16. From the first publication of a library crate, the example of its crate README MUST be one complete program of 5 to 30 lines, counting its `use` declarations and its `main`, that performs the first task a reader of that crate needs. It MUST use `?` and MUST NOT use `unwrap` or `expect`, because readers copy the example into their programs [MAKE-README].
RDM-17. A README example in Rust MUST NOT contain hidden lines, which start with `# `, and MUST show its `use` declarations and a `fn main() -> Result<(), Box<dyn std::error::Error>>` instead. GitHub renders a hidden line as written, so the reader sees `# ` lines that rustdoc would hide. This replaces, for README examples only, the hidden `Ok` line of Section 7.5 of `docs/writing-documentation.md`. The rule applies from the first publication of a crate, because only a crate README holds Rust code (RDM-22).

RDM-18. In every other respect, a README example in Rust MUST follow Section 7.5 of `docs/writing-documentation.md`, including `no_run` with its reason in the lead-in, never `ignore`, and the `text` block for an example that cannot compile as a doctest. Every code block of a README, in any language, has a language tag. A reader cannot tell a broken example from a correct one, so these rules keep the compiler checking it.

RDM-19. When an example or a command prints, the README SHOULD show what it prints in a `text` block after it, with a lead-in that says so, and the output MUST be what the example or command prints [MAKE-README]. A reader compares the output with the result on their own machine, so a wrong output reads as a failure.

### 7.2. Doctests

RDM-20. From the first publication of a crate, a crate whose crate README holds Rust code MUST compile that code as doctests by including the README in `src/lib.rs` in this form only:

```rust
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
```

A README example that no build compiles goes stale with the next change to the API. Rustdoc compiles every Rust block of the included file as a doctest [RUSTDOC] [RUST-1.54], and the item exists only while rustdoc collects doctests.

RDM-21. A crate MUST NOT use a README as its crate-level documentation, such as with `#![doc = include_str!("../README.md")]`. The two pages serve different readers with different link forms: a README needs absolute URLs and visible example lines, while crate-level documentation needs intra-doc links and the sections of Section 7.4 of `docs/writing-documentation.md`. The checkers of that guide also do not read documentation written as a `#[doc]` attribute, so their enforced rules would become judgement rules.

RDM-22. The root README MUST NOT hold a Rust code block, because no crate includes it, so nothing compiles that code. It holds `sh`, `text`, and `toml` blocks only.

### 7.3. Commands

RDM-23. Every command in a README MUST be exact, in a `sh` block, with a lead-in that says what it does and the directory it runs from. A block of several commands MAY instead have one lead-in that names the directory, with a comment line above each command that says what it does. A section of several commands MAY name the directory once, in the lead-in of the section. An author MUST run every command that a change adds or alters before the change is done, because no tool runs README commands and a reader runs them as written.

A signature that contains `*`, `(`, or `)` is in single quotes in every command, because a POSIX shell treats these characters as special and would otherwise pass other arguments to `escher`.

This illustration shows the form of a block of several commands:

````markdown
Run these commands in any directory:

```sh
# Print the properties of *632.
escher info '*632'
# List the colourings of *642 with up to 3 colours.
escher info --colourings --max-colours 3 '*642'
```
````

A command that renders an image stays in a block of its own, because RDM-29 puts its image directly under it.

Checked by: `check-doctest-attributes` reports a code block marked `ignore` in every `README.md`. From the first publication of a crate, `cargo test --doc` compiles the Rust code of each included crate README, and `check-doctest` runs it. The presence of the include, hidden lines, example length and style, printed output, quoting, and commands are judgement rules.

## 8. Links and Rendering

### 8.1. Absolute Links

RDM-24. Every link and every image in a README MUST use an absolute `https` URL, except an in-page anchor that RDM-26 allows. crates.io resolves relative links only in limited cases, and a relative image in a README at the root of a workspace can fail to resolve [CRATESIO-MD] [CARGO-16865]. The root README is the readme of every crate today, so a relative link breaks on the first publication of a crate that still inherits it. A link to a file in the repository uses the form `https://github.com/g-whiz/escher/blob/main/<path>`, and a link to a directory the form `https://github.com/g-whiz/escher/tree/main/<path>`.

A link to `main` shows the current code. From the first publication of a crate, the code on `main` can differ from the released version, so a fact that must match a release lives in the hosted API documentation, which each release tag rebuilds, as Section 2.2 defines.

### 8.2. API Documentation Links

RDM-25. A README MUST NOT link to API documentation that does not exist, such as a docs.rs page of a crate that is not published, because a reader who follows such a link finds no documentation. The owner has not decided where API documentation will be hosted. The change that first publishes a crate decides the host, and from then on a README links to the API documentation of that crate at that host. A README MAY link to the crates.io page and the API documentation of a crate in the change that publishes it, before the release makes the links resolve.

### 8.3. In-Page Anchors

RDM-26. A README MAY link to a heading of the same page with `#<anchor>` and to a heading of the root README with `https://github.com/g-whiz/escher#<anchor>`. crates.io rewrites in-page links to match its own heading identifiers, so an in-page anchor works on both sites [CRATESIO-MD]. Tables 2 and 3 fix the headings, so the anchors stay stable.

### 8.4. Alerts

RDM-27. A README MUST use an alert for its status notice and MAY use one more alert, only for a fact whose omission harms the reader. GitHub and crates.io both render alerts [CRATESIO-MD], and a page with many alerts teaches the reader to skip them.

### 8.5. HTML

RDM-28. A README MUST NOT contain raw HTML. crates.io sanitizes HTML and keeps only a short list of elements [CRATESIO-MD], so an element that GitHub shows can disappear from the page of a published crate that uses the README.

### 8.6. Images and Badges

RDM-29. The root README MUST show, directly under the code block of each command in `Getting Started` and `Examples` that renders an image, the image that the command writes. Each such command is in a code block of its own. Each image MUST meet all of these conditions:

- It is a file under `docs/images/` that the recipe `readme-images` writes with exactly the command shown.
- It is linked by an absolute URL of the form `https://raw.githubusercontent.com/g-whiz/escher/main/docs/images/<name>.png`.
- Its alternative text describes what the image shows, in one short sentence.

The README MUST NOT hold an image other than these and the badge of RDM-30, and a crate README holds no image. A change that alters a shown command or the output of `escher render` MUST run `readme-images` and commit the changed images in the same change. A reader sees what each command draws before running it and compares the result with the image, so an image that another command wrote reads as a failure. An image that shows nothing about use adds length [YEGOR256].

RDM-30. Only the root README MAY hold a badge, at most one, which is the status of the CI workflow at `https://github.com/g-whiz/escher/actions/workflows/ci.yml/badge.svg`, directly under the title. A README MUST NOT load a badge from a third-party service. A badge signals project quality to a visitor [TROCKMAN], but crates.io already shows the version and licence of a crate, and a third-party image tells another host who reads the page.

Checked by: judgement, for Sections 8.1 to 8.6. A tool could check absolute URLs, raw HTML, images, badges, and that each link resolves, but no configured tool does.

## 9. What a README Never Contains

RDM-31. A README MUST NOT contain any of these items, because each goes stale with the next change, and from the first publication of a crate a published copy stays stale until the next release:

- Progress records: milestones, what was tried, completion dates, dated measurements, identifiers of CI runs, and counts of tests. The status notice of RDM-7 and the summary of `Supported Signatures` state what a user can do now, and they are not progress records.
- A changelog, which belongs in the GitHub release of each tag.
- API reference material, which belongs in the crate documentation.
- Marketing adjectives, such as the promotional adjectives of `docs/avoiding-claudelish.md` and `simple`, `easy`, `modern`, and `production-ready`. A README states a property as a fact the reader can check.
- The section number or rule identifier of a document under `docs/`, or the path of such a document written as text. A link by absolute URL that Table 1 allows is the one way a README names such a document, because a reader on crates.io cannot open a path.
- A path from a local machine, a `TODO`, a `FIXME`, or an attribution line.

Checked by: judgement. `check-doc-references` does not read READMEs.

## 10. Language and Maintenance

RDM-32. A README MUST follow Section 8 of `docs/writing-documentation.md` and `docs/avoiding-claudelish.md`, so that every page of the project uses one spelling and ASCII punctuation and none uses the phrases that the second guide lists.

RDM-33. A change that alters a fact MUST update its home, and every pointer that the change makes false, in the same change, as Hard Rule 1 of `AGENTS.md` requires. From the first publication of a crate, versions follow two rules. A crate README ships inside its package, so each example in it that names a version of the crates MUST name the version of its package, as the API documentation does, and the change that raises that version raises the example. The root README is not packaged on its own, so each example in it MUST name the latest published version, and the change that follows a publication raises it, so that a visitor can install what the README shows.

Checked by: `check-prose` for ASCII punctuation, semicolons, and the listed spellings in every Markdown file, and `check-typos` for misspellings. The phrases of `docs/avoiding-claudelish.md` and the versions that RDM-33 requires are judgement rules, because no configured tool checks them.

## 11. Security Considerations

This guide defines no component. Its first security consideration is that a README tells readers which commands to run, so a wrong command leads a reader to run something they did not intend. `escher render` replaces an existing output file, and an unquoted signature such as `*632` lets the shell expand `*` into the names of the files in the current directory that match. RDM-23 keeps commands exact and quoted, and RDM-24 keeps links absolute and checkable.

Its second security consideration is that the first build downloads the dependencies from crates.io. `Requirements` of the root README states this, so that a reader who builds in an isolated environment knows before the build fails. RDM-30 keeps third-party images, which report each page view to their host, out of every README. The images of RDM-29 are files of this repository that GitHub serves, so they add no third-party host.

The threat model is an author who writes a wrong or stale instruction. An attacker, and a compromise of GitHub or crates.io, are out of scope, because this guide governs writing and not hosting. The residual risk is a command or a pointer that a reviewer accepts without running or following it, because no configured tool checks links or commands.

## 12. References

### 12.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 12.2. Informative References

- [18F] 18F, "Making READMEs Readable", archived 2018-07-12, <https://github.com/18F/open-source-guide/blob/18f-pages/pages/making-readmes-readable.md>.
- [CARGO-16865] The Cargo Team, "Issue 16865", rust-lang/cargo, opened 2026-04-09, <https://github.com/rust-lang/cargo/issues/16865>.
- [CARGO-MANIFEST] The Cargo Team, "The Manifest Format", The Cargo Book, <https://doc.rust-lang.org/cargo/reference/manifest.html>.
- [CRATESIO-MD] The crates.io Team, "crates_io_markdown", source code of the crates.io Markdown renderer, <https://raw.githubusercontent.com/rust-lang/crates.io/main/crates/crates_io_markdown/src/lib.rs>.
- [DIATAXIS] Procida, D., "Diataxis", <https://diataxis.fr/>.
- [GITHUB-README] GitHub, "About READMEs", GitHub Docs, accessed 2026-11-17, <https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes>.
- [KAPLAN-MOSS] Kaplan-Moss, J., "Writing great documentation", 2009-11-10, <https://jacobian.org/2009/nov/10/what-to-write/>.
- [MAKE-README] "Make a README", <https://www.makeareadme.com/>.
- [OSG] Open Source Guides, "Starting an Open Source Project", <https://opensource.guide/starting-a-project/>.
- [PRANA] Prana et al., "Categorizing the Content of GitHub README Files", 2018, <https://arxiv.org/abs/1802.06997>.
- [RUST-1.54] The Rust Release Team, "Announcing Rust 1.54.0", 2021-07-29, <https://blog.rust-lang.org/2021/07/29/Rust-1.54.0/>.
- [RUSTDOC] The Rust Project, "The rustdoc book", <https://doc.rust-lang.org/rustdoc/>.
- [STANDARD-README] Littauer, R., "Standard Readme", <https://github.com/RichardLitt/standard-readme>.
- [TROCKMAN] Trockman, A., et al., "Adding Sparkle to Social Coding", ICSE 2018, <https://cmustrudel.github.io/projects/badges>.
- [WTD] Write the Docs, "A beginner's guide to writing documentation", <https://www.writethedocs.org/guide/writing/beginners-guide-to-docs/>.
- [YEGOR256] Bugayenko, Y., "Elegant README", 2019-04-23, <https://www.yegor256.com/2019/04/23/elegant-readme.html>.

## Appendix A. Requirement Summary

The column Applies says "Now" for a rule that governs the root README today, and "First publication" for a rule that applies from the first publication of a crate.

| Label | Section | Requirement | Applies | Checked by |
| --- | --- | --- | --- | --- |
| RDM-1 | 3 | What, why, first step, and where next, in order, for one reader | Now | Judgement |
| RDM-2 | 4.1 | Each fact of Table 1 in full in its home only | Now | Judgement |
| RDM-3 | 4.2 | Elsewhere, only the form that Table 1 gives, and a pointer restates no details | Now | Judgement |
| RDM-4 | 4.3 | No feature list, API reference, full option list, errors, environment variables, or build internals | Now | Judgement |
| RDM-5 | 5.1 | Root README sections, order, and headings of Table 2 | Now | Judgement |
| RDM-6 | 5.2 | Root opening: category in at most 120 characters, then scope | Now | Judgement |
| RDM-7 | 5.3 | Status alert of a proof of concept, and the full pre-release version from the first publication | Now | Judgement |
| RDM-8 | 5.4 | Table of crates with links, in the stated order | Now | Judgement |
| RDM-9 | 5.5 | Build from a clone and a first render as commands | Now | Judgement |
| RDM-10 | 6.1 | Crate README sections, order, and headings of Table 3 | First publication | Judgement |
| RDM-11 | 6.2 | Crate opening: what it is, then where it fits | First publication | Judgement |
| RDM-12 | 6.3 | Status notice with a link to the root Status | First publication | Judgement |
| RDM-13 | 6.4 | Dependency snippet with the full version, or for `escher-cli` the install command | First publication | Judgement |
| RDM-14 | 6.5 | The row of Table 4 for the kind of crate | First publication | Judgement |
| RDM-15 | 6.6 | Own `readme` field, not inherited | First publication | Judgement |
| RDM-16 | 7.1 | One complete program of 5 to 30 lines, with `?` and no `unwrap` or `expect` | First publication | Judgement |
| RDM-17 | 7.1 | No hidden lines, a visible `fn main` that returns `Result` | First publication | Judgement |
| RDM-18 | 7.1 | Section 7.5 of the writing guide otherwise, and a language tag on every code block | Now | `check-doctest-attributes`, and from the first publication `cargo test --doc`, judgement for the rest |
| RDM-19 | 7.1 | Printed output shown and correct | Now | Judgement |
| RDM-20 | 7.2 | Crate README included as doctests through `ReadmeDoctests` | First publication | Judgement, and `cargo test --doc` for the code |
| RDM-21 | 7.2 | README not used as crate-level documentation | Now | Judgement |
| RDM-22 | 7.2 | No Rust code block in the root README | Now | Judgement |
| RDM-23 | 7.3 | Exact, quoted commands with a lead-in, run before the change is done | Now | Judgement |
| RDM-24 | 8.1 | Absolute `https` URLs for links and images, except in-page anchors | Now | Judgement |
| RDM-25 | 8.2 | No link to API documentation that does not exist, and the host that the first publication decides | Now | Judgement |
| RDM-26 | 8.3 | In-page anchors only to the fixed headings | Now | Judgement |
| RDM-27 | 8.4 | Status alert and at most one more alert | Now | Judgement |
| RDM-28 | 8.5 | No raw HTML | Now | Judgement |
| RDM-29 | 8.6 | Directly under each rendering command of the root README, the image that `readme-images` writes with it, linked by absolute URL with alternative text, and no other image than these and the badge | Now | Judgement |
| RDM-30 | 8.6 | At most one badge, the CI status, in the root README | Now | Judgement |
| RDM-31 | 9 | No progress records, changelog, API reference, marketing, document citations, local paths, or attribution | Now | Judgement |
| RDM-32 | 10 | Language of Section 8 of the writing guide and the phrases of `docs/avoiding-claudelish.md` | Now | `check-prose`, `check-typos`, judgement for the rest |
| RDM-33 | 10 | Homes and pointers updated in the same change, and from the first publication the version that each README names | Now | Judgement |

## Appendix B. Crate README Skeleton

This skeleton shows the form of the crate README of a published library crate, with placeholders in angle brackets. It is an illustration, and the placeholders name no real item:

````markdown
# <crate-name>

<What the crate is, in one sentence.> <Where it fits, and which crate to use instead.>

> [!WARNING]
> <The status of the crate, in one sentence.> Read the [project status](https://github.com/g-whiz/escher#status) before you depend on it.

## Installation

```toml
[dependencies]
<crate-name> = "<full version>"
```

## Example

<One lead-in sentence that says what the example does, and why it is `no_run` when it is.>

```rust,no_run
use <crate_name>::<Item>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    <the steps of the first task>
    Ok(())
}
```

## Requirements

<One pointer for each requirement beyond Cargo.>

## Documentation

The [API documentation](<address of the page of the crate on the hosted API documentation>) describes every item. The [project README](https://github.com/g-whiz/escher) lists the other crates.

## Licence

<The licence of the crate, with a link to its text.>
````
