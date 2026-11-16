# Architecture Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-16 |
| Applies to | The planned workspace crates of the proof of concept, the conventions between them, and the decisions that bind them |

## Abstract

This document specifies the architecture of the Escher proof of concept, a Linux command-line tool that renders Portable Network Graphics (PNG) images of planar symmetry groups named in orbifold notation and reports information on them. It defines the planned crates with their responsibilities, public surfaces, and dependencies, the flow of data from a signature to an image and to the information output, and the conventions that every crate shares, such as the linear models, the sign of the side test, and the units. It also states the constraints that the repository's standards place on the design, the testing strategy, the performance expectations, and the numerical robustness measures. It records the decisions of the owner and the proposed defaults for the open decisions.

## 1. Introduction

### 1.1. Problem

The proof of concept spans orbifold notation, three geometries, group theory, the construction of fundamental domains, and per-pixel rendering. Each of these becomes its own crate with its own design document. The research documents under `docs/research/` often give one formula in several conventions, such as the position of the time coordinate or the sign of a side test. Two crates that pick different conventions compile together and produce wrong images with no error.

### 1.2. Purpose

This document fixes, once, what the crate design documents share. These are the crate boundaries, the data flow between crates, the conventions, and the decisions. A crate design document cites this document for each of them and does not restate it.

### 1.3. Audience

The primary readers are the coding agents that write a crate design document or implement a crate. The secondary readers are reviewers and the owner, who approves the decisions of Section 10.

### 1.4. Scope

This document covers the six crates of the proof of concept, their dependencies, the data flow, the cross-crate conventions, the constraints from the standards documents, testing, performance, numerical robustness, and the decisions. Every crate, type, function, command, and option that it names is planned. The placeholder `crates/escher-cli` is planned to become the crate `escher-cli` of Section 3.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The milestones `M1` to `M19`, the research prerequisites `RP-1` to `RP-5`, and the table of unverified statements `UNV-1` to `UNV-28` are defined in `docs/design/poc-plan.md`.
- The phases after the proof of concept are defined in `docs/design/roadmap.md`.
- The application programming interface (API) of each crate, beyond the public surface of Section 3.2, is specified in the crate design documents `docs/design/signature.md`, `docs/design/geometry.md`, `docs/design/group.md`, `docs/design/domain.md`, `docs/design/render.md`, and `docs/design/cli.md` (`DEC-1`).
- The commands, options, output format, exit status, and diagnostics of the binary are specified in `docs/design/cli.md`.
- The recipes, the continuous integration (CI) workflow, and the dependency policy are specified in `docs/design/tooling.md`.
- The mathematics is stated in the subject-matter references under `docs/research/`, which this document cites by section.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md`, such as agent, design document, and subject-matter reference. It adds these terms:

- signature: A string in Conway's orbifold notation that names a planar symmetry group, such as `*732`, with the grammar of Section 3 of `docs/research/orbifold-notation.md`.
- canonical signature: The one form of a signature that `escher-signature` produces for every equivalent signature (`DEC-9`).
- general non-compact signature: A valid signature with an infinite order that is none of the 7 friezes, the triangle groups with ideal corners, and their rotation subgroups (`DEC-19`).
- geometry: One of spherical, Euclidean, and hyperbolic, as the sign of the Euler characteristic $\chi$ determines (Section 7.1 of `docs/research/orbifold-notation.md`).
- linear model: The representation of a geometry in which a point is a 3-vector, a wall is a unit normal, and an isometry is a 3x3 matrix (Section 5.1).
- model: A map of a geometry to the screen plane, such as the Poincare disk, the Klein disk, the upper half-plane, the band, or the lit ball for the sphere.
- model map: The map between the linear model and a model, in either direction.
- fundamental domain: A polygon $F$ whose images $gF$ under the group tile the geometry, together with the isometries that pair its sides.
- tile: One image $gF$ of the fundamental domain.
- side: One edge of the fundamental domain, which carries a wall, a pairing isometry, and a partner side.
- wall: The geodesic that contains a side, represented by a unit normal $u$.
- covector: The linear form $x \mapsto \langle x, u \rangle$ of a wall, in the bilinear form of its geometry.
- pairing isometry: The isometry $P_i$ of side $i$ that maps the neighbouring tile across that side onto the domain (Section 5.3).
- fold: The loop that maps a point into the fundamental domain by applying pairing isometries, and records the sequence of sides it applied.
- fold cap: The largest number of steps that the fold takes before it stops.
- sample: One point of the regular grid of points inside one pixel, whose colours are averaged into the colour of the pixel.
- colouring: An assignment of colours to tiles by the left cosets of a subgroup of finite index (Section 5.4).
- coset table: The table of the action of the generators on the cosets of a subgroup, in the standard form of GAP (Groups, Algorithms, Programming) (Section 7.2 of `docs/research/colourings.md`).
- low-index search: The search for the subgroups of index at most a bound, up to conjugacy, which gives the colourings.
- reference arithmetic: An independent implementation of a formula in a high-precision floating-point type, used only in tests (Section 7).
- golden image: A small committed PNG image that a test compares with a fresh rendering (Section 7.2).
- unverified statement: A statement that the design relies on and that no cited source establishes, labelled `UNV-n` in `docs/design/poc-plan.md`.

## 3. Crates

### 3.1. Crate Table

The workspace of the proof of concept has six members under `crates/`. Every member inherits the workspace package fields and lints, sets `publish = false` (`DEC-22`), declares no Cargo feature, and has the `# Features` section that Section 7.8 of `docs/writing-documentation.md` requires.

| Crate | Responsibility | Internal dependencies | External dependencies |
| --- | --- | --- | --- |
| `escher-signature` | Parse, validate, canonicalize, and classify a signature | None | `thiserror` |
| `escher-geometry` | Linear models and model maps | None | `glam` (`DEC-4`), then `num-complex` from `M12` |
| `escher-group` | Presentations, coset enumeration, low-index search, and colourings | `escher-signature` | None |
| `escher-domain` | Build the fundamental domain of a signature with its shape parameters | `escher-signature`, `escher-geometry`, `escher-group` | `glam` |
| `escher-render` | Turn a domain, a colouring, and a view into pixels and a PNG file | `escher-geometry`, `escher-domain`, `escher-group`, `escher-signature` | `glam`, `png`, `palette` |
| `escher-cli` | The binary `escher`, with the commands `info` and `render` | All five | `clap` with the derive API, `anyhow`, `rayon`, `csscolorparser` |

The dev-dependencies are `insta`, `proptest`, and `astro-float`, the reference-arithmetic crate of `DEC-3`.

`escher-domain` and `escher-render` compute with the `glam` types that `escher-geometry` exposes, so each declares `glam` itself, from the one entry in `[workspace.dependencies]` that rule DEP-2 in Section 12 of `docs/rust-coding-standards.md` requires.

Section 5.8 requires an error type in every library crate. The table lists `thiserror` for `escher-signature` only. Whether another library crate implements its error type with `thiserror` is for its design document to decide.

### 3.2. Public Surface

Each list below is the planned public surface of one crate, as types and operations. The crate design document refines the names and signatures.

`escher-signature`:

- `Signature`, valid by construction, with `FromStr` and `Display` in the canonical form.
- Feature accessors for handles, cone orders, boundaries with their corner orders, and crosscaps, with each order finite or infinite.
- `euler_characteristic()`, an exact rational number.
- `geometry() -> Geometry`, `is_compact()`, `is_frieze()`, `group_order()`, `area()`, and `deformation_dimension()`.
- `Error`, which carries the byte span of the offending input.

`escher-geometry`:

- No vector or matrix type of its own (`DEC-4`). A point, a unit normal, and a covector are a `glam` `DVec3`, an isometry is a `DMat3`, and a position in a planar model is a `DVec2`.
- The bilinear form of each geometry and the side test, as free functions over these types (Sections 5.1 and 5.2).
- The modules `hyperbolic`, `spherical`, and `euclidean`. Each has point normalization, the reflection from a unit normal, rotation, translation, point distance, an isometry check, and the inverse.
- Model maps in both directions between the hyperboloid and the disk, Klein, half-plane, and band models, the ball lift for the sphere, and their Jacobians.

`escher-group`:

- `Word`, `Presentation`, and `conway_presentation(&Signature)`.
- Bounded Todd-Coxeter coset enumeration.
- The low-index search, as an iterator with a work budget over standard coset tables, one per conjugacy class.
- `Colouring`, with an id, a colour count, and a coset table, and `colour_of`, which maps a side sequence to a colour.

`escher-domain`:

- `ParameterSpec`, with a name, a kind, a range, a default, and a unit, and `ShapeParameters`.
- `parameters(&Signature)` and `build(&Signature, &ShapeParameters) -> FundamentalDomain`.
- `FundamentalDomain`, which holds its geometry, its sides, its finite and ideal vertices, and a centre. Each side holds its covector, its pairing isometry, its partner side, its pairing kind, and its orientation.
- `side_presentation()`, the presentation that the sides give.
- `verify()`, which evaluates numerically the checklist of Section 4.8 of `docs/research/fundamental-domains.md`.

`escher-render`:

- `View`, with the model, the canonical placement, and the sphere rotation.
- A `RenderOptions` builder, with the size, the sample grid, the border width with its unit and colour, the fold cap, the background, and the light.
- `render_rows`, which renders a band of rows into a buffer that the caller owns, and a convenience operation for the whole image.
- `RenderStats`, its own colour type, and PNG encoding.

`escher-cli`:

- A library target with the entry `run(args, stdout, stderr)`, so that most tests run the commands without starting a process.
- The argument types, the parsing of `name=value` settings and of colours, the information text, diagnostics with a caret under the byte span, and row parallelism.

### 3.3. Dependency Graph

`escher-signature` and `escher-geometry` are the roots and depend on no other member. The column "Internal dependencies" of Section 3.1 gives the other edges, and the graph has no cycle. The graph is that of the completed proof of concept, and a crate declares a dependency from the milestone that first uses it (Section 3 of `docs/design/poc-plan.md` gives the order).

Every path dependency between members carries a version, as TOOL-25 in Section 8 of `docs/design/tooling.md` requires.

### 3.4. The Geometry Enum

The enum `Geometry`, with the variants spherical, Euclidean, and hyperbolic, lives once, in `escher-signature`, as the result of the classification (`DEC-26`). `escher-geometry` needs no such enum, because it exposes one module per geometry.

### 3.5. Reasons for the Dependency Choices

`glam` types appear in public signatures (`DEC-4`). `escher-geometry` takes and returns points, normals, covectors, and isometries as `glam` types, and `escher-domain` exposes them through `FundamentalDomain`. Rule DEP-3 in Section 12 of `docs/rust-coding-standards.md` therefore makes `glam` a public dependency of both crates, and of every other library crate whose design document puts a `glam` type in a public signature. For each such crate, DEP-3 requires these points:

- The version of `glam` is part of the crate's own version.
- `glam` is below version 1.0, and the `glam` row of Section 6 of `docs/research/rust-ecosystem.md` records 0.34.3 as its latest version. The crate therefore stays below version 1.0 for as long as it exposes `glam`.
- Every breaking upgrade of `glam` is a breaking change of the crate and is released as one. The same row records a breaking minor release of `glam` every one to three months.

The owner accepts this cost (`DEC-4`).

Rule ERR-2 in Section 7 forbids a dependency type in the public error, so no `glam` type appears in an `Error` or an `ErrorKind`. No type of `palette`, `num-complex`, or `png` appears in a public signature of a library crate.

`rayon` sits in the binary, and the renderer exposes a row-band operation. Rule CONC-4 in Section 10 of `docs/rust-coding-standards.md` forbids a library to start threads that the caller cannot see, and lets the caller choose the parallelism.

## 4. Data Flow

### 4.1. To a PNG Image

The command `render` turns a signature into a PNG file in these steps:

1. `escher-signature` parses the signature string with byte spans, validates it, and gives a `Signature` in canonical form.
2. `parameters` gives the parameter specifications of the signature. The `name=value` settings of the user override the defaults and are checked against each range.
3. `build` gives a `FundamentalDomain`. `verify()` runs in tests and behind `info --check`.
4. `side_presentation()` and the low-index search give the colourings. The chosen colouring id selects a coset table, and the colours of the user or the built-in palette fill its slots.
5. The renderer processes each sample of each pixel in these steps:
   1. It maps the screen position through the view to a position in the model.
   2. It maps the model position to a point of the linear model, or to "outside the model".
   3. It runs the fold. The fold finds the first side whose covector test is violated, applies the pairing matrix of that side, records the side, renormalizes the point at a fixed interval, and stops at the fold cap.
   4. It computes the coset from the side sequence.
   5. It computes the screen distance to the nearest side.
   6. It chooses the fill colour or the border colour, with shading on the sphere.
6. The renderer averages the samples of each pixel in linear light, encodes the result as 8-bit sRGB (standard Red Green Blue), and writes it with the `png` crate with the sRGB chunk. Section 4 of `docs/research/rust-ecosystem.md` lists the setter `set_source_srgb`.

Averaging in linear light is a project convention that no research source backs.

### 4.2. To the Information Output

The command `info` runs steps 1 to 4 of Section 4.1 and renders no pixels. Its output holds these items:

- The canonical signature.
- The geometry, the Euler characteristic, compactness, and whether the signature is a frieze.
- The group order, for a spherical signature.
- The area, for a spherical or hyperbolic signature.
- The parameters, with name, range, and default.
- A summary of the domain.
- The colourings, each with its id and colour count.

## 5. Cross-Crate Conventions

Every crate follows the conventions of this section. A crate that departs from one of them computes with the same types as its neighbours and gives wrong results with no error, so a crate design document MUST NOT change a convention here except by a change to this document.

### 5.1. Linear Models and the Minkowski Form

The hyperbolic plane uses the Minkowski form with the time coordinate last:

$$\langle x, y \rangle = x_1 y_1 + x_2 y_2 - x_3 y_3$$

This is the case $n = 2$ of the form of [CFKP] that Section 2.2 of `docs/research/geometry-models.md` states. The hyperboloid is $\lbrace x : \langle x, x \rangle = -1, \ x_3 > 0 \rbrace$ (Section 3.1 of the same document). The sphere is the unit sphere with the dot product, as [KC24] treats it (Section 11.1). The Euclidean plane is the affine plane $x_3 = 1$, which is a project convention with no research source.

A point is a `glam` `DVec3` with the time coordinate as its last component (`DEC-4`). The bilinear form of each geometry, the reflections of Section 5.2, and the side test are free functions of `escher-geometry` over the `glam` types, so these conventions stay the project's own.

The support for the time-last order is in `docs/research/geometry-models.md`:

- Section 2.4 records that [CFKP], [KC24], [DUN86], and the code of [HR] put the time coordinate last, and that [RAT] puts it last from its Section 4.5 on.
- Section 6.1 gives the isometry condition and the inverse $J M^t J$ from [CFKP] Theorem 10.1.
- Section 11.1 gives the sphere of [KC24], and Section 13.4 gives the signature table of [HR].
- Section 4.2 gives the forward maps from the hyperboloid to the disk and to the Klein model of [KC24], which are time-last, and the time-last maps of [RAT] and [DUN86] between the hyperboloid and the disk in both directions.

Formulas from the time-first sources [W-HYPERBOLOID], equation 1 of [HT], and [OBL] are not used unless the crate design document records them as unverified. Section 2.4 of `docs/research/geometry-models.md` converts no formula between conventions, because a conversion is a derivation that no source checks.

### 5.2. Walls, Normals, and the Side Test

A wall is a unit normal $u$, spacelike in the hyperbolic case, stored with its covector. A point $x$ is inside the domain where the covector of every side gives $\langle x, u \rangle \le 0$. Normals therefore point outward, and a positive value means that the point is outside that side. Section 9 adds the tolerance.

The reflection in the wall of $u$ is $x - 2 \langle x, u \rangle u$ in the spherical and hyperbolic cases, from equations (6.4) and (6.6) of [DAVIS] (Section 7.4 of `docs/research/geometry-models.md`). The Euclidean form, equation (6.3) of [DAVIS], uses a point $x_0$ of the wall. Its form as a matrix on the plane $x_3 = 1$ is part of `UNV-26`.

The sign follows SymmHub, whose side test is positive outside and whose plane normals point outward (items 2 and 3 of Section 2.4 of `docs/research/symmhub.md`). Section 2.3 of `docs/research/rendering.md` records four sign conventions in published code, finds none of them wrong, and recommends orienting the normals consistently with the chosen test.

### 5.3. Sides, Pairings, and Generators

Side $i$ carries the isometry $P_i$ that maps the neighbouring copy of the domain across that side onto the domain. This is the "reverse transform" of item 4 of Section 2.4 of `docs/research/symmhub.md`. The abstract generator of side $i$ is the group element whose image of the domain is that neighbour. A tile is $gF$.

### 5.4. Colourings

The tile $gF$ takes the colour of the left coset $gS$, as [FRETTLOH2008] p. 3 does (Sections 2.3 and 4.1 of `docs/research/colourings.md`). The design document of `escher-group` fixes the rule that turns the side sequence of the fold into the coset, and marks it as `UNV-17`. Section 8.4 of `docs/research/colourings.md` records SymmHub's reasoning about the order of composition as the code's own claim, which no source read proves.

### 5.5. Units

These units hold in every crate and on the command line:

- A width in pixels is in output pixels, not in samples.
- An intrinsic width is in the metric of curvature $-1$ (Section 2.2 of `docs/research/geometry-models.md`) or of the unit sphere.
- The Euclidean unit is the normalization length of the domain (`DEC-17`).
- Angles are in degrees on the command line and in radians inside the crates.
- A distance parameter of a matrix is the full distance. Section 2.4 of `docs/research/geometry-models.md` records that Lorentz matrices take the full distance and $SL(2, \mathbb{R})$ matrices half of it.

### 5.6. Parameter Names

A shape parameter is named `<cut-kind><index>.<quantity>`, such as `slice1.length`, `tube1.twist`, `handle1.length`, and `cap1.length`. The cut kinds follow SymmHub's keys (Sections 8.4 and 8.13 of `docs/research/fundamental-domains.md`). Euclidean parameters have plain names, such as `aspect` and `skew`.

`escher-domain` derives the names from the canonical signature only, so equivalent signatures give the same names. One twist unit applies to all cut kinds (`DEC-14`, `UNV-9`).

### 5.7. Colouring Identity

A colouring id is `<colour count><letter ordinal>`, such as `1a`, `2a`, and `3b`. The ordinal is the position of the coset table among the standard coset tables of that index, in a fixed order over the side generators (Section 7.2 of `docs/research/colourings.md` describes the standard table of GAP). An id depends on the canonical signature and the combinatorics of the construction, and never on parameter values. The alias `one` names the one-colour colouring. The alias `parity` names the orientation-preserving subgroup when the group has one.

### 5.8. Error Model

Each library crate has one `Error` with an `ErrorKind` (ERR-1), attaches the failed operation at the call site (ERR-3), writes its `Display` text in lowercase (ERR-4), and panics only for a bug in the crate itself (ERR-5). Section 7 of `docs/rust-coding-standards.md` defines these rules. A valid signature whose class has no construction returns the kind "unsupported signature".

A general non-compact signature returns that kind throughout the proof of concept, because its construction is a roadmap workstream (`DEC-19`). `escher info` still parses and classifies it. `escher render` MUST print a diagnostic that names the signature and says that rendering it is not supported, and MUST exit with a non-zero status, so that a caller never mistakes a missing image for a rendered one. `docs/design/cli.md` gives the text of the diagnostic and how `info` reports the items of Section 4.2 that need a domain.

The binary uses `anyhow` and prints the chain of sources. It draws the caret under the byte span by hand. Section 10.1 of `docs/research/rust-ecosystem.md` recommends `codespan-reporting` or `ariadne` first, and notes that a hand-written caret line suffices because the notation is one line. Section 9.1 recommends the hand-written parser that gives the byte spans.

### 5.9. Determinism and Resource Limits

The crates use no random jitter, so two runs with the same input give identical output. SymmHub jitters lengths and nudges the domain (Section 15.2 of `docs/research/symmhub.md`). Escher handles degenerate positions explicitly instead.

Each crate design document states, in its Security Considerations, its limits for untrusted input: the maximum order, the signature length, the image size, the fold cap, and the search budget. Section 11 gives the reason.

### 5.10. Competing Conventions

This table records the pick for each topic on which the sources differ:

| Topic | Pick | Support |
| --- | --- | --- |
| Time coordinate | Last | Sections 2.4 and 14 of `docs/research/geometry-models.md` |
| Sign of the side test | Outward normals, a positive value is outside | Section 2.4 of `docs/research/symmhub.md`, Section 2.3 of `docs/research/rendering.md` |
| Cosets | Left cosets | Sections 2.3 and 4.1 of `docs/research/colourings.md` |
| Band | The strip $\lvert \operatorname{Im} w \rvert < 1$ with $w = \frac{4}{\pi} \operatorname{artanh} z$ | Sections 5.1 and 14 of `docs/research/geometry-models.md`, Section 2.3 of `docs/research/rendering.md` |
| Disk and half-plane | The connecting maps of [CFKP] through the hemisphere, which connect all five models in one primary source and are already time-last. [BEA] and [RAT] each give a primary map from the half-plane to the disk, and the sources do not agree on one map (Section 4.3 of `docs/research/geometry-models.md`). The design document of `escher-geometry` records the image of the disk centre and tests the round trip | Sections 4.1 and 4.3 of `docs/research/geometry-models.md`, Section 2.3 of `docs/research/rendering.md` |
| Glyphs | `o` for a handle and `x` for a crosscap, never `o` for a crosscap | Sections 2.5 and 3.3 of `docs/research/orbifold-notation.md` |
| Reversal of a single boundary | The conditional rule of [CH] | Sections 2.5 and 4.1 of `docs/research/orbifold-notation.md` |
| Direction of commutator and conjugation | As [CH] gives them for the Conway presentation | Section 9.1 of `docs/research/orbifold-notation.md`, Section 2.5 of `docs/research/fundamental-domains.md` |
| Twist | One Escher convention | Section 12 and gap G11 in Section 13.2 of `docs/research/fundamental-domains.md`, where the research makes no recommendation |

Several sources use the band constant $4/\pi$ with this strip, while [B1] prints $2/\pi$ (Section 5.1 of `docs/research/geometry-models.md`). That [B1] is inconsistent is the research's own reasoning, which no source states.

## 6. Constraints From the Standards

Eight facts of the repository shape the design. Each row names the document that imposes the fact and its consequence for the crates.

| Fact | Imposed by | Consequence |
| --- | --- | --- |
| Miri runs every test | The command `cargo +nightly miri nextest run --workspace --no-tests=warn` of Section 4.8 of `docs/rust-coding-standards.md`, and the profile `default-miri` without a filter of Section 4 of `docs/design/tooling.md` | A test that reads a file, starts a process, or calls native code fails under Miri, and a heavy numeric test is too slow for the 60-minute timeout of every CI job (Section 7.1 of `docs/design/tooling.md`). The first change that adds tests changes Section 4 of `docs/design/tooling.md` and Section 11 of `docs/rust-coding-standards.md` (`DEC-5`) |
| Rust source has no licence header | Hard Rule 5 of `AGENTS.md`, Section 7.2 of `docs/writing-documentation.md`, and `check-rust-comments` | The MIT notice that Section 3.3 of `docs/research/symmhub.md` requires for ported code needs another place (`DEC-18`) |
| Code cites no project document | Hard Rule 15 of `AGENTS.md` | Code and tests use no milestone wording, so unverified statements are tracked only in design documents |
| Documents mirror configuration | Section 8 of `docs/design/tooling.md` states the settings of `deny.toml` and says that it has no `skip` or `ignore` list. Section 3.2 lists every recipe (TOOL-6). Section 1.1 of `docs/rust-coding-standards.md` lists the workspace members. Repository Layout and Project Status of `AGENTS.md` and the crate table of `README.md` do the same | Each of these changes in the same change as the first crates, dependencies, and recipes (Hard Rule 1 of `AGENTS.md`). Section 6 of `docs/design/poc-plan.md` lists the updates of each milestone |
| Design documents hold no progress | Section 3.6 of `docs/writing-documentation.md` | `docs/design/poc-plan.md` states its milestones as a plan. Status goes only into Project Status of `AGENTS.md` |
| No derivation in prose | Hard Rule 14 of `AGENTS.md`, Section 2.4 of `docs/research/geometry-models.md` | The linear models of Section 5.1 need a few statements that the research gives only in another convention or not at all. They are `UNV-26` and follow the owner's decision on unverified statements (Section 10.1) |
| Research prerequisites | Section 8.5 and gap G4 of `docs/research/fundamental-domains.md`, and Development Workflow of `AGENTS.md` | The research holds no step-level account of Todd-Coxeter or of the low-index search, of the geometry of SymmHub's Euclidean and spherical catalogue, or of five of the nine cut types. A change that implements mathematics starts from its subject-matter reference, so the research prerequisites `RP-1` to `RP-5` of `docs/design/poc-plan.md` precede the milestones that need them |
| CI caches nothing | Section 7.1 of `docs/design/tooling.md` | A native build, such as the GNU Multiple Precision Arithmetic Library (GMP) and the GNU Multiple Precision Floating-Point Reliable Library (MPFR) for `rug`, would rebuild in every job on every push, so the reference arithmetic is the pure-Rust `astro-float` (`DEC-3`) |

These rules of `docs/rust-coding-standards.md` have an effect on the architecture:

- ERR-1 and ERR-2 (Section 7): Section 5.8 applies them.
- DEP-3 (Section 12) and CONC-4 (Section 10): Section 3.5 applies them.
- TEST-10 (Section 11): each crate has exactly one integration-test binary.
- TEST-2 and TEST-9 (Section 11): `insta` snapshot files and property-test regression files are committed.
- SIMP-2 (Section 13): no backend abstraction exists before a second backend exists.

TOOL-25 (Section 8 of `docs/design/tooling.md`) applies through Section 3.3, and TOOL-27 (Section 9 of that document) requires `publish = false` on every member that is never published (`DEC-22`).

## 7. Testing Strategy

### 7.1. Layers

The tests form these layers, in the order in which they catch defects:

1. Unit tests beside the code (TEST-1), with the failure paths and their error kinds (TEST-6).
2. Property tests with `proptest`, with the regression files committed (TEST-9).
3. `insta` snapshots of the information text and the diagnostics (TEST-2).
4. Reference arithmetic.
5. Domain certificates: `verify()` and the exhaustive check of a finite group, run over corpora of signatures and over random parameters.
6. Tables from the research: the 17 Euclidean signatures, the 14 spherical types with their orders, the 7 friezes, the examples of [CH], values from [HT], and Frettloh's tables.
7. Oracles that compare two independent computations: the border mask against tile identity, the same point across models, the borders of `pqr` inside the borders of `*pqr`, and the side presentation against the Conway presentation.
8. Golden images.
9. One integration-test binary per crate (TEST-10) and one smoke test of the binary as a process.

The reference arithmetic is an independent implementation, in a high-precision type inside the test suite, of the few formulas that matter: the forms, the reflections, the triangle construction, the model maps, and the replay of a fold. It is not a generic instantiation of production code. The public API therefore stays concrete (SIMP-1 in Section 13 of `docs/rust-coding-standards.md`), and a transcription error is not shared between the code and its test. The design document of each crate that uses it gives the working precision and the tolerance of each quantity.

Coverage stays a report with no threshold (Section 7.1 of `docs/design/tooling.md`).

### 7.2. Golden Image Tests

The repository has no definition of a golden image test. This section proposes one. The change that adds the first golden image writes it into Section 11 of `docs/rust-coding-standards.md` as a new TEST rule (Hard Rule 13 of `AGENTS.md`) and into `docs/design/tooling.md` (`DEC-6`):

- A golden image is a small PNG file, such as 192 by 192 pixels with fixed options, committed under `tests/<suite>/golden/` of its crate.
- The test renders the image, decodes the golden file with `png`, and compares channel by channel with two limits: the largest allowed difference and the number of pixels allowed to differ. Section 11.1 of `docs/research/rust-ecosystem.md` recommends this comparison.
- The largest difference is the weight of one sample in the sample grid plus one. The count comes from a perturbation run and is recorded in the test.
- On failure, the test writes the actual image and the difference image under `target/`.
- Golden files change only through a new recipe, which the change adds to the table of Section 3.2 of `docs/design/tooling.md` (TOOL-6), with its environment variable in Section 14.2 of that document. The name of the recipe and of the variable are left to that change.
- Golden image tests and process tests do not run under Miri (`DEC-5`).

## 8. Performance

This section is non-normative. No published benchmark of a tiling renderer on the central processing unit (CPU) was found (Section 11 of `docs/research/rendering.md`). The expectations are therefore assumptions, to be confirmed at `M3`:

- The default image, 512 by 512 pixels with a 4 by 4 sample grid of a compact hyperbolic triangle group, renders in about one second on all cores.
- The cost is proportional to pixels times samples times the mean number of folds times the number of sides.

The output of `--stats` is the measurement, read by hand in the development container. No test asserts a time. Measured numbers go into commit messages and not into design documents (Section 3.6 of `docs/writing-documentation.md`).

The binary parallelizes over rows with `rayon` (Section 3.5). Two optimizations wait until a measurement asks for them: seeding the fold from the neighbouring pixel, which [VGRG] publishes (Section 4.1 of `docs/research/rendering.md`), and single instruction, multiple data (SIMD) code (Section 15 of `docs/research/rust-ecosystem.md`).

## 9. Numerical Robustness

### 9.1. Fold Cap

Section 4.4 of `docs/research/rendering.md` gives caps from 10 to 4000 in published code, and its Section 4.7 cites [NS] for the fact that the number of folds has no upper bound in a hyperbolic group. `DEC-7` proposes the default cap and the colour of a sample that reaches it. Section 4.9 of `docs/research/rendering.md` gives the precedents for the behaviour at the cap.

### 9.2. Wall Tolerance

A side counts as violated only when its covector value exceeds a tolerance relative to the size of the coordinates. Tests cover points on walls and on vertices, because no source covers this behaviour (`UNV-3`).

### 9.3. Normalization

The fold renormalizes the point onto its model surface at a fixed interval, after HyperRogue's `normalize` and the forced variant of [KC24] (Sections 8.2 and 8.4 of `docs/research/geometry-models.md`). Any accumulated matrix is corrected in the same way. Long products are tested against reference arithmetic, because [FWW], of which only the abstract was read, reports a growth of error in Lorentz products. Section 5.4 of `docs/research/symmhub.md` reports a trace error of $5 \times 10^{-5}$ on long words in a hyperboloid factorization.

### 9.4. Limit Circle

A point at distance $d$ from the centre lies at $1 - \Theta(e^{-d})$ in the Poincare disk (Section 8.3 of `docs/research/geometry-models.md`). This is the research's reading of the exponent that [KC24] prints, which Section 2.3 of `docs/research/rendering.md` records, and it is part of `UNV-4`. Samples outside the disk take the background. The rim is where samples reach the fold cap.

### 9.5. Ideal Vertices

No source establishes that the fold applies at an ideal vertex (item 3 of Section 13.1 of `docs/research/rendering.md`, `UNV-23`). The design expects samples near an ideal vertex not to converge, so they reach the fold cap and take the colour of Section 9.1.

### 9.6. General Position

The crates use no jitter (Section 5.9). The linear model uses unit normals and matrices, and no circles. The design therefore expects no counterpart of the circle close to a line that SymmHub's note on general position warns about (Section 15.2 of `docs/research/symmhub.md`). No source confirms this expectation.

## 10. Decisions

### 10.1. Decisions of the Owner

This section records the decisions of the owner for the proof of concept and the roadmap. They bind every design document until the owner changes them.

Goals of the proof of concept:

- The tool is a Linux command-line tool that renders PNG images of a symmetry given in orbifold notation, 512 by 512 pixels by default and configurable.
- It draws polygons with black borders and colours that the user chooses, and the user can set properties such as the border thickness.
- A separate information command reports how many polygons the representative set has, which says which colour counts are possible, and whether the symmetry is spherical, Euclidean, or hyperbolic.
- Hyperbolic symmetries render in the Poincare disk first, and in other models later. Spherical symmetries render with simple Phong shading.

Scope and staging:

- The proof of concept proceeds in three stages and ends with the full notation. Stage 1 covers reflection polygons and their rotation subgroups in all three geometries. Stage 2 covers all 17 Euclidean and 14 spherical groups. Stage 3 covers general hyperbolic signatures, including handles and crosscaps, by porting SymmHub's pipeline, which is under the MIT licence. The proof of concept is complete only when every valid signature renders, except the general non-compact signatures (`DEC-19`).
- From the first milestone, the parser accepts parentheses for orders of 10 or more, and both the infinity sign and `(inf)`, and `info` classifies such signatures. Rendering the friezes and the triangle groups with ideal corners is a late milestone of the proof of concept.
- Every signature has a default, most symmetric shape. `info` reports the number, names, and ranges of the free parameters, and `render` accepts `name=value` settings. Each parameter arrives with the signatures that have it.

Image content:

- The polygons are copies of the fundamental domain. Colourings are coset colourings from subgroups of finite index, found by a low-index search that the project implements itself, because no Rust crate provides one. SymmHub's `lib/sublib` is the reference. One colour and the two-colour parity colouring come first.
- `info` lists every colouring up to a maximum colour count, 12 by default and changeable by an option, each with a short id. `render` takes a colouring id and a list of colours, with a built-in palette as the fallback.
- Borders have a width in screen pixels by default and first, and an intrinsic width in the metric of the geometry later, as an option.
- Antialiasing uses supersampling with a configurable grid first. Analytic smoothing with a tracked scale factor follows later in the proof of concept, and tests compare the two.
- The hyperbolic models of the proof of concept are the Poincare disk, then the upper half-plane, the band, and the Klein model. The star, ring, and bent band models go to the roadmap.
- The sphere is a lit ball in orthographic projection with Phong shading and a single light, and its viewing rotation is adjustable on the command line.
- Each signature and model has one canonical view, centred on the fundamental domain. View control, which is rotation, shift, and zoom, goes to the roadmap.

Engineering:

- The representation is the linear model: points are 3-vectors with a bilinear form per geometry, reflections are unit normals, and isometries are 3x3 matrices. Conformal circles and complex numbers appear only where a model map or the port of SymmHub needs them.
- The workspace has one crate per concern: signature, geometry, group theory with colourings, construction of the fundamental domain, renderer, and the binary. Each crate has its own design document under `docs/design/`.
- The dependencies are the picks of `docs/research/rust-ecosystem.md`: `png`, `clap`, `rayon`, `num-complex` and `glam` (`DEC-4`), `palette`, `thiserror`, `anyhow`, `proptest`, `insta`, and a reference-arithmetic crate for tests (`DEC-3`). The licence and duplicate-version work in `deny.toml` happens in the milestone where each dependency first appears.
- Work proceeds on unverified statements. Each is flagged in its design document, tested numerically against high-precision reference arithmetic, and tracked until a source or a Lean proof closes it. No unverified statement blocks the proof of concept.
- Lean is not part of the proof of concept. It is a roadmap workstream that starts after the requested sources are read, beginning with what the fold depends on. The elan installer has no checksum verification, so it is pinned by the digest of its GitHub release asset.

Roadmap:

- The first phase after the proof of concept is live rendering with wgpu and Slang, through the crates `slango` and `slang-slint` of the slang-rust project once they are published. The Slint development graphical user interface (GUI) on Linux follows, then a release on Linux, macOS, and Windows, and later Android and iOS.
- Vector graphics follow, then other planar tilings in the medium term, then the simulation of partial differential equations on orbifolds in the long term.
- The roadmap has a deep-research item for each topic that the research only surveyed: aperiodic tilings, partial differential equations on orbifolds, vector export, animation, and shader export.

Decided defaults, which keep their labels in Section 10.2 because other documents cite them:

- `DEC-3`, reference arithmetic: `astro-float`, which is pure Rust under the MIT licence, with one comparison against MPFR recorded (`UNV-25`). Section 12.1 of `docs/research/rust-ecosystem.md` recommends `rug` if a C toolchain in CI is acceptable, because MPFR is the reference standard, and `astro-float` for pure Rust, checked once against `rug` or MPFR because nobody has verified its claims of correct rounding. `DEC-3` takes the pure-Rust branch, because a native build rebuilds in every CI job (Section 6, last row) and native code fails under Miri (Section 6, first row). `rug` also adds the licence `LGPL-3.0-or-later`.
- `DEC-4`, linear algebra: the real vectors and matrices are the `glam` types `DVec2`, `DVec3`, and `DMat3`, and the project writes no vector or matrix type of its own. `num-complex` provides complex numbers from `M12`. The owner chose `glam` because much of the logic moves to Slang after the proof of concept, so a vector and matrix library of the project's own would be wasted work. The double-precision types are the ones that Section 6.1 of `docs/research/rust-ecosystem.md` recommends. Section 5.1 states the conventions over these types, and Section 3.5 states what rule DEP-3 requires as a result.
- `DEC-5`, Miri: tests that use files, processes, or long numeric loops carry `#[cfg_attr(miri, ignore = "...")]`, and property tests use fewer cases under Miri. Section 6, first row, gives the reason and says which documents the first change that adds tests updates.
- `DEC-18`, MIT notice: ported SymmHub code carries its MIT notice in a notices file at the repository root and in a section of the crate-level documentation of each crate with ported code. The notice arrives in the first change that ports SymmHub code, and not before, and that change records it in `AGENTS.md` and `docs/design/tooling.md`. Rust source holds no licence header (Section 6, second row), and `UNV-24` tracks the open licence question.
- `DEC-19`, completion: the general non-compact signatures (Section 2.1) are not part of the proof of concept and are a workstream of Section 4 of `docs/design/roadmap.md`, because the research records no implementation for them (Section 5.18 of `docs/design/poc-plan.md`). The friezes and the triangle groups with ideal corners stay in milestone `M18`, and the completion audit `M19` covers every other valid signature. Section 5.8 states how `info` and `render` treat such a signature.

The layout of the planning documents is a proposal that the owner may change. It follows the sibling project slang-rust and has three documents under `docs/design/`. This document holds the architecture and the decisions, `docs/design/poc-plan.md` holds the milestones of the proof of concept, and `docs/design/roadmap.md` holds the phases after it.

### 10.2. Proposed Defaults

The table lists every labelled default. The owner has decided `DEC-3`, `DEC-4`, `DEC-5`, `DEC-18`, and `DEC-19`, and Section 10.1 states them. Every other default is a proposal and holds until the owner changes it. A design document that relies on a default cites its label.

| Label | Topic | Proposed default | Reason |
| --- | --- | --- | --- |
| `DEC-1` | Names | Crates `escher-signature`, `escher-geometry`, `escher-group`, `escher-domain`, `escher-render`, and `escher-cli`. Design documents `architecture.md`, `signature.md`, `geometry.md`, `group.md`, `domain.md`, `render.md`, and `cli.md` | One crate per concern, as the owner decided. Availability on crates.io is checked only before publication |
| `DEC-2` | Location of the plan and roadmap | The milestone plan `docs/design/poc-plan.md` and the roadmap `docs/design/roadmap.md`, next to this document, as plans without status (Section 10.1) | Section 3.6 of `docs/writing-documentation.md` |
| `DEC-3` | Reference-arithmetic crate | Decided by the owner: `astro-float` | Section 10.1 |
| `DEC-4` | Linear algebra | Decided by the owner: `glam` and `num-complex`, with no vector or matrix type of the project's own | Section 10.1 |
| `DEC-5` | Miri | Decided by the owner: tests that cannot run under Miri carry `#[cfg_attr(miri, ignore = "...")]` | Section 10.1 |
| `DEC-6` | Golden images | As Section 7.2 | No golden image test is defined yet |
| `DEC-7` | Render defaults | A 4 by 4 sample grid, a black border 1 pixel wide, and a fold cap of 500. Samples at the cap take the border colour. A white background, with a `transparent` option. `--output` is required. The default colouring is the first two-colour colouring when one exists, else one colour | Section 9.1. Published caps range from 10 to 4000 |
| `DEC-8` | Input text | Features in any unambiguous order, as SymmHub accepts them. Upper-case `O` and `X` are accepted. Of the characters outside American Standard Code for Information Interchange (ASCII), only the infinity sign is accepted. The empty string is rejected. Order 1 is accepted and dropped, with `1` as the trivial group | The SymmHub parser is the precedent (Section 11.1 of `docs/research/orbifold-notation.md`), and the proposal accepts only input whose meaning is unambiguous. `UNV-20` |
| `DEC-9` | Canonical form | The feature order of [CH], cone orders descending, each boundary at its largest rotation with reversal by the rule of [CH], boundaries descending, crosscaps only when any is present, parentheses from order 10, and `(inf)` in output | No primary source defines a canonical form (`UNV-20`) |
| `DEC-10` | Signatures with infinity | Hyperbolic when $\chi < 0$, a frieze when $\chi = 0$, and rejected when $\chi > 0$ | Recorded as `UNV-21` |
| `DEC-11` | Domain drawn for general signatures | The assembled polygon, with a Dirichlet domain only if `M17` needs one. The design document of `escher-domain` fixes which wall the rotation subgroups double | The assembled polygon is what the port of SymmHub produces |
| `DEC-12` | Canonical view | The centre is the normalized sum of the finite vertices. The whole disk or ball shows, with a small margin on the shorter side. The Euclidean window is fixed relative to the size of the domain | The owner decided on one canonical view centred on the domain. The centre rule and the margin are choices of this proposal |
| `DEC-13` | Default shapes | Regular where the orders allow. Otherwise SymmHub's initial length 2 (Section 8.13 of `docs/research/fundamental-domains.md`) without its jitter, and twist 0 | Determinism (Section 5.9). The default twist 0 is a choice of this proposal, which no source gives |
| `DEC-14` | Parameters | Names as Section 5.6. Every twist is a fraction of its length in $[-0.5, 0.5]$. Length ranges start from SymmHub's 0.3 to 4 | Section 8.13 of `docs/research/fundamental-domains.md` gives SymmHub's ranges and its inconsistent twist units (`UNV-9`) |
| `DEC-15` | Colourings | One entry per conjugacy class, a budget on search work with truncation reported, and ids as Section 5.7 | The time of GAP's low-index search grows exponentially with the index (Section 7.7 of `docs/research/colourings.md`) |
| `DEC-16` | Sphere | The rotation is three angles in degrees. One directional light from the upper left front. Borders are shaded like fills. The pixel width uses the same estimate as elsewhere | The renderer keeps one border estimate in every model. The light direction is a choice of this proposal |
| `DEC-17` | Euclidean intrinsic unit | The normalization length of the domain, stated per signature | An intrinsic width needs a unit. This unit is a project convention |
| `DEC-18` | MIT notice | Decided by the owner: a notices file and a section in the crate-level documentation, from the first change that ports SymmHub code | Section 10.1 |
| `DEC-19` | Completion and the general non-compact signatures | Decided by the owner: the general non-compact signatures move to the roadmap | Section 10.1 |
| `DEC-20` | Other notations in `info` | Omitted until a primary source is read | Tables A.1 and A.2 of [CBG] give primary names for the 17 plane groups and the spherical groups. The frieze names, the order of the numbers in five Coxeter symbols, and the Thurston column have no primary source (item 11 of Section 12 of `docs/research/orbifold-notation.md`) |
| `DEC-21` | Threads | `rayon` in the binary, with `--threads` | Section 3.5 |
| `DEC-22` | Publication | `publish = false` on every crate through the proof of concept | TOOL-27, and nothing is published without the owner's go-ahead |
| `DEC-23` | Information format | Text, covered by snapshots, and declared unstable. A machine-readable form is a roadmap item | A snapshot shows every change to the text for review |
| `DEC-24` | Bad orbifolds | `info` exits with a non-zero status and a diagnostic | A bad orbifold corresponds to no group (Section 7.2 of `docs/research/orbifold-notation.md`) |
| `DEC-25` | Orders | 32-bit integers, with checked exact rational arithmetic | An overflow becomes an error and no wrong value |
| `DEC-26` | `Geometry` enum | In `escher-signature` | Section 3.4 |
| `DEC-27` | Performance budget | As Section 8, confirmed at `M3` | No published benchmark exists |

## 11. Security Considerations

The threat model is a user who passes input from an untrusted party to `escher`: the signature, the `name=value` settings, the colours, and the numeric options. The attacker can choose any string within the limits of the command line. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory and time. A large order, a long signature, a large image, a high fold cap, or a low-index search over a large index can each consume unbounded resources. Each crate design document therefore states its limits (Section 5.9), and the crate returns an error when an input exceeds one. Orders use checked arithmetic (`DEC-25`), so an overflow is an error.

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the workspace denies `unsafe_code`, and no crate of this architecture needs it. No crate calls a native library at run time, and `DEC-3` keeps a native build out of the tests.
- File paths: the binary writes one PNG file to the path that the user gives and reads no file. The design document of `escher-cli` states how it handles an existing file at that path.
- Build scripts: no crate of this architecture has one.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`.
- Prior-art code: SymmHub's code is untrusted. Agents read it as a reference and port it by hand. Nothing in the build or the tests runs it, and ported code passes the same review and checks as any other code. Fixtures from GAP or `low_index` are generated outside the repository and committed as data (Section 14.4 of `docs/research/rust-ecosystem.md`).
- Secrets: no file in the repository contains a secret, and no workflow reads one (TOOL-22 in Section 7.2 of `docs/design/tooling.md`).

Attacks on the repository itself, such as a malicious change, and attacks on the machine that runs the tool are out of scope, because this document governs the crates and not access control.

The residual risk is a limit that a crate design document sets too high for the machine that runs the tool, which then runs slowly or runs out of memory before the limit applies. A second residual risk is a defect in ported code that review does not find.

## 12. Operational Considerations

No crate declares a Cargo feature (Section 3.1) or has a build script (Section 11), so no crate reads an environment variable at build time. The golden image recipe of Section 7.2 adds one environment variable, which Section 14.2 of `docs/design/tooling.md` will list.

The supported platform of the proof of concept is Linux. The crates render on the CPU and need no graphics processing unit (GPU) or display, so `just check` runs every test (TOOL-8 in Section 3.1 of `docs/design/tooling.md`). The minimum supported Rust version (MSRV) is the one that Technology Decisions of `AGENTS.md` states.

Each library crate reports a failure through its `Error` and `ErrorKind` (Section 5.8). These failure modes recur across crates:

| Failure | Reported as | Recovery |
| --- | --- | --- |
| Malformed or bad signature | An error of `escher-signature` with a byte span. `info` exits with a non-zero status (`DEC-24`) | Correct the signature |
| Valid signature without a construction | The kind "unsupported signature". `render` exits with a non-zero status (Section 5.8) | None until its milestone in `docs/design/poc-plan.md`. None in the proof of concept for a general non-compact signature (`DEC-19`) |
| Parameter outside its range | An error of `escher-domain` with its documented kind | Choose a value in the range that `info` lists |
| Search budget exhausted | A truncated listing, reported as truncated (`DEC-15`) | Lower the maximum colour count |
| Coset enumeration over its bound | The limit error of `escher-group` | None. The group is infinite or the bound is too low |
| Input over a resource limit | The limit error of the crate | Reduce the input |
| Sample at the fold cap | No error. The sample is counted in `RenderStats` and coloured as Section 9.1 states | Raise the fold cap |

The crates emit no log. The binary prints the statistics of a rendering only with `--stats`.

## 13. References

### 13.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 13.2. Informative References

The tags of the mathematical sources resolve to the reference sections of the research documents. Each entry names the section that holds the full entry.

- [B1] Section 16.1 of `docs/research/geometry-models.md`.
- [BEA] Section 16.1 of `docs/research/geometry-models.md`.
- [CBG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CFKP] Section 16.1 of `docs/research/geometry-models.md`.
- [CH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [DAVIS] Section 16.1 of `docs/research/geometry-models.md`.
- [DUN86] Section 16.1 of `docs/research/geometry-models.md`.
- [FRETTLOH2008] Section 15 of `docs/research/colourings.md`.
- [FWW] Section 16.2 of `docs/research/geometry-models.md`.
- [HR] Section 16.1 of `docs/research/geometry-models.md`.
- [HT] Section 16.1 of `docs/research/geometry-models.md`.
- [KC24] Section 16.1 of `docs/research/geometry-models.md`.
- [NS] Section 14 of `docs/research/rendering.md`.
- [OBL] Section 16.2 of `docs/research/geometry-models.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [VGRG] Section 14 of `docs/research/rendering.md`.
- [W-HYPERBOLOID] Section 16.3 of `docs/research/geometry-models.md`.
