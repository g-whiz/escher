# Architecture Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-19 |
| Applies to | The workspace crates of the proof of concept, the conventions between them, and the decisions that bind them |

## Abstract

This document specifies the architecture of the Escher proof of concept, a Linux command-line tool that renders Portable Network Graphics (PNG) images of planar symmetry groups named in orbifold notation and reports information on them. It defines the crates with their responsibilities, public surfaces, and dependencies, the flow of data from a signature to an image and to the information output, and the conventions that every crate shares, such as the linear models, the sign of the side test, and the units. It also states the constraints that the repository's standards place on the design, the testing strategy, the performance expectations, and the numerical robustness measures. It records the decisions of the owner and the proposed defaults for the open decisions.

## 1. Introduction

### 1.1. Problem

The proof of concept spans orbifold notation, three geometries, group theory, the construction of fundamental domains, and per-pixel rendering. Each of these becomes its own crate with its own design document. The research documents under `docs/research/` often give one formula in several conventions, such as the position of the time coordinate or the sign of a side test. Two crates that pick different conventions compile together and produce wrong images with no error.

### 1.2. Purpose

This document fixes, once, what the crate design documents share. These are the crate boundaries, the data flow between crates, the conventions, and the decisions. A crate design document cites this document for each of them and does not restate it.

### 1.3. Audience

The primary readers are the coding agents that write a crate design document or implement a crate. The secondary readers are reviewers and the owner, who approves the decisions of Section 10.

### 1.4. Scope

This document covers the six crates of the proof of concept, their dependencies, the data flow, the cross-crate conventions, the constraints from the standards documents, testing, performance, numerical robustness, and the decisions. It describes every crate, type, function, command, and option that it names as a design, whether or not the code exists. Project Status in `AGENTS.md` says which crates exist. The crate `escher-cli` of Section 3 is the package in `crates/escher-cli`.

### 1.5. Non-Goals

These topics are owned elsewhere, and this document cites them:

- The milestones `M1` to `M19`, the research prerequisites `RP-1` to `RP-5`, and the table of unverified statements `UNV-1` to `UNV-33` are defined in `docs/design/poc-plan.md`.
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
- colouring: An assignment of colours to tiles by the left cosets of a subgroup of finite index (Section 5.4). Each subgroup of finite index gives its own colouring (`DEC-37`).
- coset table: The table of the action of the generators on the cosets of a subgroup, in the standard form of GAP (Groups, Algorithms, Programming) (Section 7.2 of `docs/research/colourings.md`).
- low-index search: The search for the subgroups of index at most a bound, which finds one table for each conjugacy class and expands each class into its subgroups, which give the colourings.
- reference arithmetic: An independent implementation of a formula in a high-precision floating-point type, used only in tests (Section 7).
- golden image: A small committed PNG image that a test compares with a fresh rendering (Section 7.2).
- unverified statement: A statement that the design relies on and that no cited source establishes, labelled `UNV-n` in `docs/design/poc-plan.md`.

## 3. Crates

### 3.1. Crate Table

The workspace of the proof of concept has six members under `crates/`. Every member inherits the workspace package fields and lints, sets `publish = false` (`DEC-22`), declares no Cargo feature, and has the `# Features` section that Section 7.8 of `docs/writing-documentation.md` requires.

| Crate | Responsibility | Internal dependencies | External dependencies |
| --- | --- | --- | --- |
| `escher-signature` | Parse, validate, canonicalize, and classify a signature | None | `thiserror` |
| `escher-geometry` | Linear models and model maps | None | `glam` (`DEC-4`) and `thiserror`, then `num-complex` from `M12` |
| `escher-group` | Presentations, coset enumeration, low-index search, and colourings | `escher-signature` | `thiserror` |
| `escher-domain` | Build the fundamental domain of a signature with its shape parameters | `escher-signature`, `escher-geometry`, `escher-group` | `glam` and `thiserror` |
| `escher-render` | Turn a domain, a colouring, and a view into pixels and a PNG file | `escher-geometry`, `escher-domain`, `escher-signature`, and `escher-group` from `M8` | `glam`, `png`, and `thiserror` (`DEC-39`) |
| `escher-cli` | The binary `escher`, with the commands `info` and `render` | `escher-signature`, `escher-domain`, `escher-render`, and `escher-group` from `M7` | `clap` with the derive API, `anyhow`, and `rayon` (`DEC-21`) |

The dev-dependencies are `insta`, `proptest`, and `astro-float-num`, the reference-arithmetic crate of `DEC-3`.

`escher-domain` and `escher-render` compute with the `glam` types that `escher-geometry` exposes, so each declares `glam` itself, from the one entry in `[workspace.dependencies]` that rule DEP-2 in Section 12 of `docs/rust-coding-standards.md` requires.

Section 5.8 requires an error type in every library crate. The table lists `thiserror` for `escher-signature`, `escher-geometry`, `escher-group`, `escher-domain`, and `escher-render`, whose design documents choose it. Whether another library crate implements its error type with `thiserror` is for its design document to decide.

### 3.2. Public Surface

Each list below is the public surface of one crate as this document designs it, as types and operations. The crate design document refines the names and signatures.

`escher-signature`: `Signature`, valid by construction and in canonical form, with its feature accessors, its exact Euler characteristic, and its classification, and an `Error` that carries the byte span of the offending input. Section 3 of `docs/design/signature.md` specifies the public surface.

`escher-geometry`: no vector or matrix type of its own (`DEC-4`), and one module per geometry, `hyperbolic`, `spherical`, and `euclidean`, of free functions over the `glam` types. They give the bilinear form and the side test (Sections 5.1 and 5.2), point normalization, distance, the reflection from a unit normal, rotation, translation, the inverse, an isometry check, and the model maps in both directions between the hyperboloid and the disk, Klein, half-plane, and band models, with the Jacobians of the maps. The lift onto the ball of the sphere and its screen tangents are private to `escher-render` (Section 5.9 of `docs/design/render.md`). Section 3 of `docs/design/geometry.md` specifies the public surface, and its Section 3.4 states where the other geometries and the further model maps go.

`escher-group`:

- `Letter`, `Word`, `Relator` with a base word and an exponent, `Presentation`, and `conway_presentation(&Signature)`, which returns a `Result`.
- Bounded Todd-Coxeter coset enumeration, `enumerate`, with a `CosetLimit`, which returns a complete and standardized `CosetTable` or an error.
- The low-index search, as three functions with an index limit and a work budget: one that counts the colourings of each colour count, one that lists every colouring up to the limit, and one that lists the colourings of one colour count. A search that stops at its budget or at a limit on what it stores returns a result complete to a lower colour count, and no error.
- `Colouring`, with an id, the id of its conjugacy class, a colour count, and a coset table, and `colour_of`, which maps a side sequence to a colour.

Section 3 of `docs/design/group.md` specifies the public surface.

`escher-domain`: `FundamentalDomain`, built from a signature, which holds its geometry, its sides, its vertices, and a centre, and `verify()`, which evaluates numerically the items of the checklist of Section 4.8 of `docs/research/fundamental-domains.md` that apply to the domain. Each side holds its unit normal, its covector, its pairing isometry, its partner side, its pairing kind, and its orientation. Section 3 of `docs/design/domain.md` specifies the public surface. These parts of the design belong to the same crate, and `docs/design/domain.md` specifies each one in the change that adds the first signature that needs it:

- `ParameterSpec`, with a name, a range, and a default, and with a kind and a unit from the first parameter that needs them, `ShapeParameters`, `parameters(&Signature)`, and `build_with`, the form of `build` that takes the shape parameters.
- The pairing kinds of the rotation subgroups, and the vertex cycles that `verify()` walks for a domain without mirrors.
- Ideal vertices.
- `side_presentation()`, the presentation that the sides give, which returns a `Result` with the `Presentation` of `escher-group` (Section 5.4 of `docs/design/domain.md`).

`escher-render`:

- `View`, with the model and the canonical placement.
- A `RenderOptions` builder, with the size, the sample grid, the border width with its unit and colour, the fold cap, the background, and the viewing rotation, the light, and the shading of the sphere.
- `render_rows`, which renders a band of rows into a buffer that the caller owns, and a convenience operation for the whole image.
- `RenderStats`, its own colour type, and PNG encoding.

`escher-cli`:

- A library target whose only public items are the entry `run(args, stdout, stderr)` and its status type, so that most tests run the commands without starting a process. Section 3 of `docs/design/cli.md` specifies them. Everything else, such as the argument types, the information text, and the diagnostics, is private to the crate.

### 3.3. Dependency Graph

`escher-signature` and `escher-geometry` are the roots and depend on no other member. The column "Internal dependencies" of Section 3.1 gives the other edges, and the graph has no cycle. `escher-group` depends on `escher-signature` only, so `escher-domain` can depend on it. From milestone `M7`, `escher-domain` and `escher-cli` declare `escher-group`, and from milestone `M8`, `escher-render` does. The graph is that of the completed proof of concept, and a crate declares a dependency from the milestone that first uses it (Section 3 of `docs/design/poc-plan.md` gives the order).

Every path dependency between members carries a version, as TOOL-25 in Section 8 of `docs/design/tooling.md` requires.

### 3.4. The Geometry Enum

The enum `Geometry`, with the variants spherical, Euclidean, and hyperbolic, lives once, in `escher-signature`, as the result of the classification (`DEC-26`). `escher-geometry` needs no such enum, because it exposes one module per geometry.

### 3.5. Reasons for the Dependency Choices

`glam` types appear in public signatures (`DEC-4`). `escher-geometry` takes and returns points, normals, covectors, and isometries as `glam` types, and `escher-domain` exposes them through `FundamentalDomain`. Rule DEP-3 in Section 12 of `docs/rust-coding-standards.md` therefore makes `glam` a public dependency of both crates, and of every other library crate whose design document puts a `glam` type in a public signature. For each such crate, DEP-3 requires these points:

- The version of `glam` is part of the crate's own version.
- `glam` is below version 1.0, and the `glam` row of Section 6 of `docs/research/rust-ecosystem.md` records 0.34.3 as its latest version. The crate therefore stays below version 1.0 for as long as it exposes `glam`.
- Every breaking upgrade of `glam` is a breaking change of the crate and is released as one. The same row records a breaking minor release of `glam` every one to three months.

The owner accepts this cost (`DEC-4`).

Rule ERR-2 in Section 7 forbids a dependency type in the public error, so no `glam` type appears in an `Error` or an `ErrorKind`. No type of `num-complex` or `png` appears in a public signature of a library crate.

`rayon` sits in the binary, and the renderer exposes a row-band operation. Rule CONC-4 in Section 10 of `docs/rust-coding-standards.md` forbids a library to start threads that the caller cannot see, and lets the caller choose the parallelism.

## 4. Data Flow

### 4.1. To a PNG Image

The command `render` turns a signature into a PNG file in these steps:

1. `escher-signature` parses the signature string with byte spans, validates it, and gives a `Signature` in canonical form.
2. `parameters` gives the parameter specifications of the signature. The `name=value` settings of the user override the defaults and are checked against each range.
3. `build` gives a `FundamentalDomain`. `verify()` runs in tests and behind `info --check`. For a hyperbolic reflection polygon with four or more corners and for its double, `build_with` also runs `verify()` and refuses a domain that fails it (`DEC-48`).
4. When the user names a colouring by its id, `side_presentation()` and one run of the low-index search for the colour count of the id give the colourings of that count, and the id selects one coset table (`DEC-44`). The colours of the user, or the first entries of the built-in palette of 12 colours, fill its cosets (`DEC-39`). The aliases `one` and `parity`, and the default colouring, need no search.
5. The renderer processes each sample of each pixel in these steps:
   1. It maps the screen position through the view to a position in the model.
   2. It maps the model position to a point of the linear model, or to "outside the model".
   3. It runs the fold. The fold finds the first side whose covector test is violated, applies the pairing matrix of that side, records the side, and stops at the fold cap. It does not renormalize the point (Section 9.3).
   4. For a colouring by cosets, it computes the coset from the recorded side sequence with `colour_of` (Section 5.4).
   5. It computes the screen distance to the nearest side (`DEC-28`).
   6. It chooses the fill colour or the border colour, with shading on the sphere.
6. The renderer averages the samples of each pixel in linear light, encodes the result as 8-bit sRGB (standard Red Green Blue), and writes it with the `png` crate with the sRGB chunk. Section 4 of `docs/research/rust-ecosystem.md` lists the setter `set_source_srgb`.

Averaging in linear light is a project convention that no research source backs.

### 4.2. To the Information Output

The command `info` runs steps 1 to 3 of Section 4.1, reads the side presentation of a built domain, runs the low-index search on it up to the maximum colour count, and renders no pixels. It takes no settings, so step 2 gives the defaults, and its summary of the domain describes the default shape. Section 5 of `docs/design/cli.md` specifies the exact lines that it prints for each kind of signature, and which items arrive with later milestones. Its output holds these items:

- The canonical signature.
- The geometry, the Euler characteristic, compactness, and whether the signature is a frieze.
- The group order, for a spherical signature.
- The area, for a spherical or hyperbolic signature.
- The parameters, with name, range, and default.
- A summary of the domain.
- With `--presentation`, the Conway presentation of the signature and the side presentation of the domain, and for a spherical signature the group order that coset enumeration finds for each (`DEC-35`).
- The number of colourings of each colour count up to a maximum, and with an option every colouring with its id and colour count (`DEC-38`).

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

A wall is a unit normal $u$, spacelike in the hyperbolic case, stored with its covector. A point $x$ is inside the domain where the covector of every side gives $\langle x, u \rangle \le 0$. In the Euclidean plane a normal cannot carry the offset of its line, so the covector of the line $a_1 x_1 + a_2 x_2 = t$ with $a_1^2 + a_2^2 = 1$ is $(a_1, a_2, -t)$, its normal is $(a_1, a_2, 0)$, and the side test is the dot product of the covector with the point $(x_1, x_2, 1)$, with the same sign rule. Normals therefore point outward, and a positive value means that the point is outside that side. Section 9 adds the tolerance.

The reflection in the wall of $u$ is $x - 2 \langle x, u \rangle u$ in the spherical and hyperbolic cases, from equations (6.4) and (6.6) of [DAVIS] (Section 7.4 of `docs/research/geometry-models.md`). The Euclidean form, equation (6.3) of [DAVIS], uses a point $x_0$ of the wall. Its form as a matrix on the plane $x_3 = 1$ is part of `UNV-26`.

The sign follows SymmHub, whose side test is positive outside and whose plane normals point outward (items 2 and 3 of Section 2.4 of `docs/research/symmhub.md`). Section 2.3 of `docs/research/rendering.md` records four sign conventions in published code, finds none of them wrong, and recommends orienting the normals consistently with the chosen test.

### 5.3. Sides, Pairings, and Generators

Side $i$ carries the isometry $P_i$ that maps the neighbouring copy of the domain across that side onto the domain. This is the "reverse transform" of item 4 of Section 2.4 of `docs/research/symmhub.md`. The abstract generator of side $i$ is the group element whose image of the domain is that neighbour, which is $P_i^{-1}$. A tile is $gF$.

A word in the generators is read left to right as the product left to right, so the word $s_a s_b$ is the group element $s_a s_b$, and evaluated as matrices it is the matrix product in that order. This is a convention of this document. Every crate that turns a word into an element, or an element into a word, follows this convention. The side presentation of `escher-domain` has one generator for each side under it (Section 5.4 of `docs/design/domain.md`). No source read states the cycle relators of a domain as words in this convention, which is `UNV-32`.

### 5.4. Colourings

The tile $gF$ takes the colour of the left coset $gS$, as [FRETTLOH2008] p. 3 does (Sections 2.3 and 4.1 of `docs/research/colourings.md`). Each subgroup $S$ of finite index gives its own colouring, every conjugate included (`DEC-37`).

The coset tables of `escher-group` hold right cosets (Section 5.10). For a fold that applies the sides $i_1, \ldots, i_n$ in that order, the colour of the sample is the number of the right coset $S g^{-1}$ with $g = s_{i_1} \cdots s_{i_n}$: the trace from coset 0 of the colouring's own table through the inverse column of each side, in the reverse order of the fold. GRP-38 of `docs/design/group.md` states the rule, and `colour_of` is its one implementation (`DEC-40`). Coset 0 is the colour of the fundamental domain itself. No source states the rule, so it is part of `UNV-17`. Section 8.4 of `docs/research/colourings.md` records SymmHub's reasoning about the order of composition as the code's own claim, which no source read proves.

### 5.5. Units

These units hold in every crate and on the command line:

- A width in pixels is in output pixels, not in samples.
- An intrinsic width is in the metric of curvature $-1$ (Section 2.2 of `docs/research/geometry-models.md`) or of the unit sphere.
- The Euclidean unit is the normalization length of the domain (`DEC-17`).
- Angles are in degrees on the command line and in radians inside the crates.
- A distance parameter of a matrix is the full distance. Section 2.4 of `docs/research/geometry-models.md` records that Lorentz matrices take the full distance and $SL(2, \mathbb{R})$ matrices half of it.

### 5.6. Parameter Names

A shape parameter is named `<cut-kind><index>.<quantity>`, such as `slice1.length`, `tube1.twist`, `handle1.length`, and `cap1.length`. The cut kinds are SymmHub's keys in lower case (Sections 8.4 and 8.13 of `docs/research/fundamental-domains.md`, and Section 6.2.1 of `docs/research/symmhub.md`). The index counts from 1 within each cut kind. Euclidean parameters have plain names, such as `aspect` and `skew`.

From milestone `M9`, the hyperbolic reflection polygons have two cut kinds, both with the quantity `length` (`DEC-47`):

| Cut kind | Example | What it names |
| --- | --- | --- |
| `cornerpair` | `cornerpair1.length` | A corner pair: two consecutive corners of order 2, and the mirror side between them, whose length the parameter sets |
| `slice` | `slice1.length` | A slice: a cut of the polygon, and the distance between the two walls that it joins, which the parameter sets |

The name `cornerpair` stays distinct from the cut kind of a pair of cone points, which SymmHub keys as `conePair` and a later milestone adds. Section 6.6 of `docs/design/domain.md` states the parameters of each signature, their meaning, their order, their ranges, and their defaults (DOM-45).

From milestone `M10`, the Euclidean signatures have two plain parameters. `parameters` lists `aspect` first and `skew` second:

| Name | Signatures | What it names |
| --- | --- | --- |
| `aspect` | `*2222`, `2222`, `o`, `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` | The length of the base of the rectangle or parallelogram of the domain over its height. For `2222`, the `aspect` of the rectangle of `*2222` that its domain doubles |
| `skew` | `o` and `2222` | The first coordinate of the second lattice vector divided by the length of the first (`DEC-52`) |

`4*2`, `3*3`, and the Euclidean reflection triangles have no parameter. DOM-53 of `docs/design/domain.md` states the ranges, the defaults, and the meaning of each parameter of each signature.

`escher-domain` derives the names from the canonical signature only, so equivalent signatures give the same names. One twist unit applies to all cut kinds (`DEC-14`, `UNV-9`). In the Euclidean plane, `skew` settles `UNV-9` as a project convention (`DEC-52`).

### 5.7. Colouring Identity

A colouring id is `<colour count><letter ordinal>`, such as `1a`, `2a`, and `3b`. The ordinal is the rank of the standard coset table of the colouring among the standard tables of all the subgroups of that index, every conjugate included, in the lexicographic order of their entries read row by row over all columns, with the side generators in the order `s0` upward (GRP-37 of `docs/design/group.md`, `DEC-43`). An id depends on the canonical signature and on the numbering of the sides of the domain, and never on parameter values, the search limit, or the budget. The numbering of the sides is therefore part of the contract of `escher-domain` (DOM-37 of `docs/design/domain.md`). The alias `one` names the one-colour colouring, `1a`. The alias `parity` names the orientation-preserving subgroup of index 2 when the group has one. A group whose elements all preserve orientation, such as the group of `732`, has none, so `parity` names no colouring of it (`DEC-32`). CLI-48 of `docs/design/cli.md` states both aliases.

### 5.8. Error Model

Each library crate has one `Error` with an `ErrorKind` (ERR-1), attaches the failed operation at the call site (ERR-3), writes its `Display` text in lowercase (ERR-4), and panics only for a bug in the crate itself (ERR-5). Section 7 of `docs/rust-coding-standards.md` defines these rules. A valid signature whose class has no construction returns the kind "unsupported signature".

A general non-compact signature returns that kind throughout the proof of concept, because its construction is a roadmap workstream (`DEC-19`). `escher info` still parses and classifies it. `escher render` MUST print a diagnostic that names the signature and says that rendering it is not supported, and MUST exit with a non-zero status, so that a caller never mistakes a missing image for a rendered one. The design branch of the milestone that adds `render` writes the text of the diagnostic into `docs/design/cli.md`, and the design branch of each milestone that adds an item of Section 4.2 that needs a domain writes there how `info` reports it.

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
| Cosets of a colouring | Left cosets, numbered through the right coset $S g^{-1}$ of the table (Section 5.4, `DEC-40`) | Sections 2.3 and 4.1 of `docs/research/colourings.md`. The numbering is `UNV-17` |
| Cosets of a coset table | Right cosets and a right action, with coset 0 the subgroup (`DEC-36`) | Section 7.21 of `docs/research/colourings.md`: every source read uses right cosets and a right action, and none uses left cosets |
| Band | The strip $\lvert \operatorname{Im} w \rvert < 1$ with $w = \frac{4}{\pi} \operatorname{artanh} z$ | Sections 5.1 and 14 of `docs/research/geometry-models.md`, Section 2.3 of `docs/research/rendering.md` |
| Disk and half-plane | The connecting maps of [CFKP] through the hemisphere, which connect all five models in one primary source and are already time-last. [BEA] and [RAT] each give a primary map from the half-plane to the disk, and the sources do not agree on one map (Section 4.3 of `docs/research/geometry-models.md`). The design document of `escher-geometry` records the image of the disk centre and tests the round trip | Sections 4.1 and 4.3 of `docs/research/geometry-models.md`, Section 2.3 of `docs/research/rendering.md` |
| Glyphs | `o` for a handle and `x` for a crosscap, never `o` for a crosscap | Sections 2.5 and 3.3 of `docs/research/orbifold-notation.md` |
| Reversal of a single boundary | The conditional rule of [CH] | Sections 2.5 and 4.1 of `docs/research/orbifold-notation.md` |
| Direction of commutator and conjugation | As [CH] gives them for the Conway presentation | Section 9.1 of `docs/research/orbifold-notation.md`, Section 2.5 of `docs/research/fundamental-domains.md` |
| Twist | One Escher convention | Section 12 and gap G11 in Section 13.2 of `docs/research/fundamental-domains.md`, where the research makes no recommendation |

The coset tables of `escher-group` follow the sources and use right cosets, while Section 5.4 specifies a colouring by left cosets. The conversion between the two is the rule of Section 5.4, which GRP-38 of `docs/design/group.md` states and `UNV-17` covers.

Several sources use the band constant $4/\pi$ with this strip, while [B1] prints $2/\pi$ (Section 5.1 of `docs/research/geometry-models.md`). That [B1] is inconsistent is the research's own reasoning, which no source states.

## 6. Constraints From the Standards

Seven facts of the repository shape the design. Each row names the document that imposes the fact and its consequence for the crates.

| Fact | Imposed by | Consequence |
| --- | --- | --- |
| Rust source has no licence header | Hard Rule 5 of `AGENTS.md`, Section 7.2 of `docs/writing-documentation.md`, and `check-rust-comments` | The MIT notice that Section 3.3 of `docs/research/symmhub.md` requires for ported code needs another place (`DEC-18`) |
| Code cites no project document | Hard Rule 15 of `AGENTS.md` | Code and tests use no milestone wording, so unverified statements are tracked only in design documents |
| Documents mirror configuration | Section 8 of `docs/design/tooling.md` states the settings of `deny.toml` and says that it has no `skip` or `ignore` list. Section 3.2 lists every recipe (TOOL-6). Section 1.1 of `docs/rust-coding-standards.md` lists the workspace members. Repository Layout and Project Status of `AGENTS.md` and the crate table of `README.md` do the same | Each of these changes in the same change as the first crates, dependencies, and recipes (Hard Rule 1 of `AGENTS.md`). Section 6 of `docs/design/poc-plan.md` lists the updates of each milestone |
| Design documents hold no progress | Section 3.6 of `docs/writing-documentation.md` | `docs/design/poc-plan.md` states its milestones as a plan. Status goes only into Project Status of `AGENTS.md` |
| No derivation in prose | Hard Rule 14 of `AGENTS.md`, Section 2.4 of `docs/research/geometry-models.md` | The linear models of Section 5.1 need a few statements that the research gives only in another convention or not at all. They are `UNV-26` and follow the owner's decision on unverified statements (Section 10.1) |
| Research prerequisites | Sections 8.5 and 8.15 and gap G4 of `docs/research/fundamental-domains.md`, and Development Workflow of `AGENTS.md` | The research prerequisites of `docs/design/poc-plan.md` supply, before the milestones that need them, a step-level account of Todd-Coxeter and of the low-index search (Sections 7.8 to 7.21 of `docs/research/colourings.md`), and the geometry of SymmHub's Euclidean and spherical catalogue (Sections 7.1.1 to 7.1.9 of `docs/research/symmhub.md` and Sections 9.5 and 9.6 of `docs/research/fundamental-domains.md`). Deaton's thesis sources all nine cut rules and the twelve atoms (Section 8.15 of `docs/research/fundamental-domains.md`). Gap G4 keeps open SymmHub's order of the cuts, its pairing of corners of order 2 across the end of the signature, and its departures from the thesis. A change that implements mathematics starts from its subject-matter reference |
| CI caches nothing | Section 7.1 of `docs/design/tooling.md` | A native build, such as the GNU Multiple Precision Arithmetic Library (GMP) and the GNU Multiple Precision Floating-Point Reliable Library (MPFR) for `rug`, would rebuild in every job on every push, so the reference arithmetic is the pure-Rust astro-float project, through its crate `astro-float-num` (`DEC-3`) |

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

The reference arithmetic is an independent implementation, in a high-precision type inside the test suite, of the few formulas that matter: the forms, the reflections, the constructions of the triangle and of the reflection polygon, the model maps, and the replay of a fold. It is not a generic instantiation of production code. The public API therefore stays concrete (SIMP-1 in Section 13 of `docs/rust-coding-standards.md`), and a transcription error is not shared between the code and its test. The design document of each crate that uses it gives the working precision and the tolerance of each quantity.

Coverage stays a report with no threshold (Section 7.1 of `docs/design/tooling.md`).

### 7.2. Golden Image Tests

TEST-14 in Section 11 of `docs/rust-coding-standards.md` states the general rule for a golden image test (`DEC-6`). The golden image tests of the proof of concept take this form:

- A golden image is a small PNG file, such as 192 by 192 pixels with fixed options, committed under `tests/<suite>/golden/` of its crate.
- The test renders the image, decodes the golden file with `png`, and compares channel by channel with two limits: the largest allowed difference and the number of pixels allowed to differ. Section 11.1 of `docs/research/rust-ecosystem.md` recommends this comparison.
- The comparison is in linear light, because samples are averaged in linear light (Section 4.1), so the weight of one sample in stored 8-bit sRGB units depends on the colours. The largest difference is the weight of one sample in the sample grid plus the size of one 8-bit sRGB step at the stored value, in linear light.
- The design of the crate sets the number of pixels allowed to differ and records the evidence for it. Section 8.4 of `docs/design/render.md` gives the allowance of `escher-render` and the figures behind it.
- On failure, the test writes the actual image and the difference image under the directory that `CARGO_TARGET_TMPDIR` names, which is under `target/`.
- Golden files change only through the recipe `update-golden`. Section 3.2 of `docs/design/tooling.md` lists the recipe, and its Section 14.2 lists its environment variable.

## 8. Performance

This section is non-normative. No published benchmark of a tiling renderer on the central processing unit (CPU) was found (Section 11 of `docs/research/rendering.md`). The design therefore rests on these expectations:

- The default image, 512 by 512 pixels with a 4 by 4 sample grid of a compact hyperbolic triangle group, renders in about one second on all cores.
- The cost is proportional to pixels times samples times the mean number of folds times the number of sides.

The default image of `*732` meets the first expectation in a release build, on all cores and on one thread. No measurement covers the second expectation, so it remains an assumption.

The output of `--stats` is the measurement, read by hand in the development container. No test asserts a time. Measured numbers go into commit messages and not into design documents (Section 3.6 of `docs/writing-documentation.md`).

The binary parallelizes over rows with `rayon` (Section 3.5). Two optimizations wait until a measurement asks for them: seeding the fold from the neighbouring pixel, which [VGRG] publishes (Section 4.1 of `docs/research/rendering.md`), and single instruction, multiple data (SIMD) code (Section 15 of `docs/research/rust-ecosystem.md`).

## 9. Numerical Robustness

### 9.1. Fold Cap

Section 4.4 of `docs/research/rendering.md` gives caps from 10 to 4000 in published code, and its Section 4.7 cites [NS] for the fact that the number of folds has no upper bound in a hyperbolic group. `DEC-7` proposes the default cap and the colour of a sample that reaches it. Section 4.9 of `docs/research/rendering.md` gives the precedents for the behaviour at the cap.

### 9.2. Wall Tolerance

A side counts as violated only when its covector value exceeds a tolerance relative to the size of the coordinates. Tests cover points on walls and on vertices, because no source covers this behaviour (`UNV-3`).

### 9.3. Normalization

The fold does not renormalize the point, neither inside its loop nor after it (`DEC-31`). HyperRogue's `normalize` and the forced variant of [KC24] renormalize a point onto its model surface (Sections 8.2 and 8.4 of `docs/research/geometry-models.md`). A prototype outside the repository measured that renormalization at a fixed interval changes which sides the fold applies near the rim of the disk and can return infinite coordinates, and that scaling the point by a power of two changes no side that the fold applies. Section 11 of `docs/design/render.md` records the figures, and its Section 6.1 states how the renderer decides on the point without normalizing it.

The renderer accumulates no matrix. It carries the vectors that the border estimate needs through the same pairings as the point (Section 5.3 of `docs/design/render.md`). `escher-geometry` has no function that renormalizes a matrix (Section 8.3 of `docs/design/geometry.md`). Long folds are tested against reference arithmetic, because [FWW], of which only the abstract was read, reports a growth of error in Lorentz products. Section 5.4 of `docs/research/symmhub.md` reports a trace error of $5 \times 10^{-5}$ on long words in a hyperboloid factorization.

### 9.4. Limit Circle

A point at distance $d$ from the centre lies at $1 - \Theta(e^{-d})$ in the Poincare disk (Section 8.3 of `docs/research/geometry-models.md`). This is the research's reading of the exponent that [KC24] prints, which Section 2.3 of `docs/research/rendering.md` records, and it is part of `UNV-4`. Samples outside the disk take the background. The rim is where samples reach the fold cap.

### 9.5. Ideal Vertices

No source establishes that the fold applies at an ideal vertex (item 3 of Section 13.1 of `docs/research/rendering.md`, `UNV-23`). The design expects samples near an ideal vertex not to converge, so they reach the fold cap and take the colour of Section 9.1.

### 9.6. General Position

The crates use no jitter (Section 5.9). The linear model uses unit normals and matrices, and no circles. The design therefore expects no counterpart of the circle close to a line that SymmHub's note on general position warns about (Section 15.2 of `docs/research/symmhub.md`). No source confirms this expectation.

## 10. Decisions

### 10.1. Decisions of the Owner

This section records the decisions of the owner for the proof of concept. They bind every design document until the owner changes them. Section 3 of `docs/design/roadmap.md` records the phases after the proof of concept and marks which of their decisions are the owner's.

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
- `info` reports the colourings up to a maximum colour count, 12 by default and changeable by an option, each with a short id, in the form that `DEC-38` states. `render` takes a colouring id and a list of colours, with a built-in palette as the fallback (`DEC-39`).
- Borders have a width in screen pixels by default and first, and an intrinsic width in the metric of the geometry later, as an option.
- Antialiasing uses supersampling with a configurable grid first. Analytic smoothing with a tracked scale factor follows later in the proof of concept, and tests compare the two.
- The hyperbolic models of the proof of concept are the Poincare disk, then the upper half-plane, the band, and the Klein model. The star, ring, and bent band models go to the roadmap.
- The sphere is a lit ball in orthographic projection with Phong shading and a single light, and its viewing rotation is adjustable on the command line.
- Each signature and model has one canonical view, centred on the fundamental domain. View control, which is rotation, shift, and zoom, goes to the roadmap.
- The proof of concept fills each polygon with a solid colour for its coset, so that the mathematical and geometric foundations are settled first. The owner abstracts a later motif as a colour sampler (Section 3.3 of `docs/design/roadmap.md`). The data flow of Section 4.1 already computes the point in the fundamental domain and the coset (steps 5.3 and 5.4) before step 5.6 chooses the colour. The proof of concept defines no interface for a sampler.

Engineering:

- The representation is the linear model: points are 3-vectors with a bilinear form per geometry, reflections are unit normals, and isometries are 3x3 matrices. Conformal circles and complex numbers appear only where a model map or the port of SymmHub needs them.
- The workspace has one crate per concern: signature, geometry, group theory with colourings, construction of the fundamental domain, renderer, and the binary. Each crate has its own design document under `docs/design/`.
- The dependencies are the picks of `docs/research/rust-ecosystem.md`: `png`, `clap`, `rayon`, `num-complex` and `glam` (`DEC-4`), `palette`, `thiserror`, `anyhow`, `proptest`, `insta`, and a reference-arithmetic crate for tests (`DEC-3`). The licence and duplicate-version work in `deny.toml` happens in the milestone where each dependency first appears. `palette` is not a dependency in `M3` (`DEC-29`), and the owner decided at `M8` that no colour crate is a dependency (`DEC-39`).
- Work proceeds on unverified statements. Each is flagged in its design document, tested numerically against high-precision reference arithmetic, and tracked until a source or a Lean proof closes it. No unverified statement blocks the proof of concept.
- Lean is not part of the proof of concept. It is a roadmap workstream that starts after the requested sources are read, beginning with what the fold depends on. The elan installer has no checksum verification, so it is pinned by the digest of its GitHub release asset.

Decided defaults, which keep their labels in Section 10.2 because other documents cite them:

- `DEC-3`, reference arithmetic: astro-float, which is pure Rust under the MIT licence, with one comparison against MPFR recorded (`UNV-25`). The tests use `astro-float-num`, the number crate of the astro-float project, for the reason that Section 11 of `docs/design/geometry.md` states. Section 12.1 of `docs/research/rust-ecosystem.md` recommends `rug` if a C toolchain in CI is acceptable, because MPFR is the reference standard, and `astro-float` for pure Rust, checked once against `rug` or MPFR because nobody has verified its claims of correct rounding. `DEC-3` takes the pure-Rust branch, because a native build rebuilds in every CI job (Section 6, last row). `rug` also adds the licence `LGPL-3.0-or-later`.
- `DEC-4`, linear algebra: the real vectors and matrices are the `glam` types `DVec2`, `DVec3`, and `DMat3`, and the project writes no vector or matrix type of its own. `num-complex` provides complex numbers from `M12`. The owner chose `glam` because much of the logic moves to Slang after the proof of concept, so a vector and matrix library of the project's own would be wasted work. The double-precision types are the ones that Section 6.1 of `docs/research/rust-ecosystem.md` recommends. Section 5.1 states the conventions over these types, and Section 3.5 states what rule DEP-3 requires as a result.
- `DEC-5`, Miri: the owner withdrew this decision on 2026-11-19, as the last list of decisions in this section states.
- `DEC-18`, MIT notice: ported SymmHub code carries its MIT notice in a notices file at the repository root and in a section of the crate-level documentation of each crate with ported code. The notice arrives in the first change that ports SymmHub code, and not before, and that change records it in `AGENTS.md` and `docs/design/tooling.md`. Rust source holds no licence header (Section 6, first row), and `UNV-24` tracks the open licence question.
- `DEC-19`, completion: the general non-compact signatures (Section 2.1) are not part of the proof of concept and are a workstream of Section 4 of `docs/design/roadmap.md`, because the research records no implementation for them (Section 5.18 of `docs/design/poc-plan.md`). The friezes and the triangle groups with ideal corners stay in milestone `M18`, and the completion audit `M19` covers every other valid signature. Section 5.8 states how `info` and `render` treat such a signature.

The owner decided these defaults on 2026-11-16, for milestone `M3`:

- `DEC-21`, threads: `rayon` in the binary. On 2026-11-16, a two-thread pool of `rayon`, run under Miri on a prototype outside the repository, made Miri report undefined behaviour inside `crossbeam-epoch` 0.9.21, a dependency of `rayon`. With the option `-Zmiri-tree-borrows`, Miri reported memory leaks instead. The owner accepts this for the proof of concept and wants developer work kept minimal until a minimum viable product exists. The design gives the binary the option `--threads`. Since the owner removed Miri on 2026-11-19, no check of this repository looks for that undefined behaviour.
- `DEC-28`, border estimate: the gradient form. No source states this construction in the linear model, so it is recorded as an unverified claim (`UNV-15`) and tested against the distance form of Section 6.2 of `docs/research/rendering.md` and against an oracle of tile identity. `docs/design/render.md` specifies the estimate and its tests.
- `DEC-29`, colour input: colours are given in hexadecimal only, in the forms `#rgb` and `#rrggbb`, and a small parser in the project reads them. No colour crate is a dependency in `M3`. A prototype outside the repository measured that `palette` fails the ban on duplicate versions, and that `csscolorparser` fails it with its default features and passes without them but then reads no colour name. Section 9.2 of `docs/design/cli.md` records the measurement. At `M8` the owner kept this decision and added no colour crate (`DEC-39`).
- `DEC-30`, existing output file: `escher render` replaces an existing output file without asking. It has no `--overwrite` option.

The owner decided this amendment of `DEC-12` on 2026-11-17, for milestone `M6`:

- `DEC-12`, view of a rotation signature: the canonical view of a signature without reflections that `M6` builds is the view of its starred signature, centred on the original reflection domain, so that `732` and `*732` show the same tiling in the same place. The starred signature is the reflection signature whose domain `M6` doubles, such as `*732` for `732`. The domain design keeps that reflection domain in place and gives the double its centre, and Section 5.11 of `docs/design/render.md` states the view.

The owner decided these defaults on 2026-11-18, for milestone `M8`:

- `DEC-37`, one colouring for each subgroup: every subgroup of finite index is its own colouring, with its own id and its own line in a listing, every conjugate included. This replaces the part of `DEC-15` that listed one colouring for each conjugacy class. A prototype outside the repository found that conjugate subgroups give different pictures, and that no sampled element of the group related the pictures of two conjugate subgroups (Section 11.3 of `docs/design/group.md`). The search still finds one table for each class and then expands each class into its subgroups, and the id ranks a colouring among all the subgroups of its colour count (GRP-34 and GRP-37 of `docs/design/group.md`).
- `DEC-38`, the colourings in `info`: `escher info` prints by default the number of colourings of each colour count up to the maximum colour count, and an option lists every colouring with its id. `escher-group` therefore counts colourings without storing every table, and lists the colourings of one colour count. CLI-45 and CLI-46 of `docs/design/cli.md` state the lines and the option.
- `DEC-39`, the palette: the default colours are a fixed table of 12 colours in `escher-cli`, with no colour crate, and a colouring with more than 12 colours needs `--colours`. Colours stay hexadecimal only (`DEC-29`). CLI-44 of `docs/design/cli.md` states the table.

The owner made this decision on 2026-11-19:

- `DEC-5`, Miri: withdrawn. The project no longer uses Miri, or the nightly toolchain, which only Miri used. The owner decided this on the recommendation of an agent, which gave these reasons:
  - The workspace denies `unsafe_code`, and no crate opts out, so Miri can find almost nothing in the code of this project.
  - Miri cost minutes in every run of `just check` and of CI, and a skip attribute on about 400 tests.
  - The interpreter's own rounding of floating-point functions gave fragile results.

  The owner added that the Slang work will not use Miri here, because the Slang library does that itself.

  TEST-7 of `docs/rust-coding-standards.md` states the check that a crate that opts out of the denial of `unsafe_code` adds.

The layout of the planning documents is a proposal that the owner may change. It follows the sibling project slang-rust and has three documents under `docs/design/`. This document holds the architecture and the decisions, `docs/design/poc-plan.md` holds the milestones of the proof of concept, and `docs/design/roadmap.md` holds the phases after it.

### 10.2. Proposed Defaults

The table lists every labelled default. The owner has decided `DEC-3`, `DEC-4`, `DEC-18`, `DEC-19`, `DEC-21`, `DEC-28`, `DEC-29`, `DEC-30`, `DEC-37`, `DEC-38`, and `DEC-39`, the view of a rotation signature in `DEC-12`, and the listing of each subgroup in place of each conjugacy class in `DEC-15`, and has withdrawn `DEC-5`. Section 10.1 states these decisions. Every other default is a proposal and holds until the owner changes it. A design document that relies on a default cites its label.

| Label | Topic | Proposed default | Reason |
| --- | --- | --- | --- |
| `DEC-1` | Names | Crates `escher-signature`, `escher-geometry`, `escher-group`, `escher-domain`, `escher-render`, and `escher-cli`. Design documents `architecture.md`, `signature.md`, `geometry.md`, `group.md`, `domain.md`, `render.md`, and `cli.md` | One crate per concern, as the owner decided. Availability on crates.io is checked only before publication |
| `DEC-2` | Location of the plan and roadmap | The milestone plan `docs/design/poc-plan.md` and the roadmap `docs/design/roadmap.md`, next to this document, as plans without status (Section 10.1) | Section 3.6 of `docs/writing-documentation.md` |
| `DEC-3` | Reference-arithmetic crate | Decided by the owner: astro-float. The tests use its number crate `astro-float-num`, which Section 13.3 of `docs/design/geometry.md` raises with the owner | Section 10.1 |
| `DEC-4` | Linear algebra | Decided by the owner: `glam` and `num-complex`, with no vector or matrix type of the project's own | Section 10.1 |
| `DEC-5` | Miri | Withdrawn by the owner on 2026-11-19: the project does not use Miri | Section 10.1 |
| `DEC-6` | Golden images | As Section 7.2, compared in linear light | TEST-14 of `docs/rust-coding-standards.md` defines the test, and samples are averaged in linear light |
| `DEC-7` | Render defaults | A 4 by 4 sample grid, a black border 1 pixel wide, and a fold cap of 1500. Samples at the cap take the border colour. An opaque white background, and no `transparent` background. `--output` is required. The default colouring is `parity` when a side pairing of the domain reverses orientation, else one colour, as CLI-34 of `docs/design/cli.md` states, so a default rendering runs no search (`DEC-44`). The default fill colours are `#e9c46a` and `#2a9d8f` | Section 9.1. Published caps range from 10 to 4000. `escher-domain` accepts a corner of order 1000, which needed 1000 folds on a prototype, and the cap covers every fold count that the prototype measured (Section 11 of `docs/design/render.md`). A fixed cap is simpler than one derived from the signature. Averaging samples with alpha has no source. The fill colours are the example pair of `M3` in `docs/design/poc-plan.md` |
| `DEC-8` | Input text | Features in any order, as SymmHub accepts them, but text with two readings is rejected, such as an order after a `*` and an `o` or `x`, and a `1` directly followed by a digit, so `11` and `*11` are errors. Whitespace and the empty string are rejected. Upper-case `O` and `X` are accepted. Of the characters outside American Standard Code for Information Interchange (ASCII), only the infinity sign is accepted. An order 1 is accepted and dropped, with `1` as the trivial group. Section 4.2 of `docs/design/signature.md` states the reading rules | The SymmHub parser is the precedent (Section 11.1 of `docs/research/orbifold-notation.md`), and the proposal accepts only input whose meaning is unambiguous. `UNV-20` |
| `DEC-9` | Canonical form | The feature order of [CH], cone orders descending with an infinite order above every finite order, each boundary at its largest rotation, boundaries descending, and every handle turned into two crosscaps when a crosscap is present, so `oox` becomes `xxxxx`. Parentheses from order 10, and `(inf)` in output. Section 7 of `docs/design/signature.md` states the algorithm, including the reversal of boundaries | No primary source defines a canonical form (`UNV-20`) |
| `DEC-10` | Signatures with infinity | Hyperbolic when $\chi < 0$, a frieze when $\chi = 0$, and rejected when $\chi > 0$ | Recorded as `UNV-21` |
| `DEC-11` | Domain drawn for general signatures | The assembled polygon, with a Dirichlet domain only if `M17` needs one. The rotation subgroups of `M6` double the reflection domain of their starred signature across the wall of side 1, as Section 6.8 of `docs/design/domain.md` states (DOM-23) | The assembled polygon is what the port of SymmHub produces. SymmHub also doubles its triangle across one of its sides (Section 7.2.7 of `docs/research/symmhub.md`). On a prototype outside the repository, the double of `*(1000)32` across side 2 failed check 4 of the domain design, with a residual of $2.42 \times 10^{-6}$ against a tolerance of $2 \times 10^{-6}$ (Section 11 of `docs/design/render.md`) |
| `DEC-12` | Canonical view | The centre is the normalized sum of the finite vertices, except for the lune of `*nn`, whose two vertices are antipodal, so that their sum is zero: its centre is the point $(0, 0, 1)$, a choice of the spherical construction of `docs/design/domain.md`. The whole disk or ball shows, with a small margin on the shorter side, and the ball has the size and the margin of the disk (Section 5.9 of `docs/design/render.md`). In the Euclidean plane the tiling fills the image with no margin, the centre of the domain is at the centre of the image, and the shorter side of the image spans 8 units of `DEC-17` (Section 5.8 of `docs/design/render.md`). A rotation domain of `M6` has the centre of the reflection domain of its starred signature, which stays in place inside it, so a rotation signature has the view of its starred signature (Section 5.11 of `docs/design/render.md`). A wallpaper domain of `M10` has its centre at the origin, which is the centre of its rectangle or parallelogram, or of the generating triangle of a kite of `4*2` or `3*3`, and not the mean of its vertices. The sheared domain of `2222` keeps the centre of its generating rectangle, which the shear fixes (DOM-50 of `docs/design/domain.md`, Section 5.14 of `docs/design/render.md`) | The owner decided on one canonical view centred on the domain, and on 2026-11-17 that a rotation signature of `M6` has the view of its starred signature, centred on the reflection domain (Section 10.1). The centre rule, the centre of the lune, the margin, the size of the ball, and the Euclidean span are choices of this proposal. A span that is a power of two makes the pixel size exact at power-of-two image sizes. The centre of a wallpaper domain is a choice of this proposal. Of the wallpaper domains, only the five-sided domain of `2*22` has the mean of its vertices elsewhere, at $(0, -h/10)$ for the height $h$ of its rectangle, and the kites have the centre of their generating triangle as the kites of `442` and `632` do |
| `DEC-13` | Default shapes | Regular where the orders allow. Otherwise SymmHub's initial length 2 (Section 8.13 of `docs/research/fundamental-domains.md`) without its jitter, and twist 0. A hyperbolic reflection polygon `*p1...pn` with four or more corners takes the polygon with an inscribed circle from the proof of Theorem 7.16.2 of [BEA], which is the regular polygon when every order is equal. Each default corner-pair length and slice length is the value of that polygon, rounded to three decimals and clamped to its range of `DEC-14` | Determinism (Section 5.9). The default twist 0 is a choice of this proposal, which no source gives. The research gives no formula for the cut lengths of a regular polygon, and Beardon's polygon is sourced and exists for every polygon signature. The rounding keeps each default in the `Parameter` line of `escher info` free of the last bits of floating-point arithmetic (CLI-20 of `docs/design/cli.md`). On a prototype outside the repository, the bisection for the inscribed circle converged for every signature tried, the default polygon was tangent to its circle to $10^{-50}$ in 50-digit arithmetic, `*3333` had the default `slice1.length` 1.317, which is $\operatorname{arcosh} 2$, and `*22222` the corner-pair lengths 1.061. The defaults of every signature with 4 to 8 corners lay from 0.658, for `*3222`, to 3.06, for `*44444444`, so the clamp changes none of them, and the default closest to a rounding boundary of the third decimal lay $9.3 \times 10^{-9}$ from it. After rounding, the side lengths of a default polygon differ from those of the inscribed polygon by up to $8.5 \times 10^{-4}$ |
| `DEC-14` | Parameters | Names as Section 5.6. Every twist is a fraction of its length in $[-0.5, 0.5]$. The corner-pair lengths and slice lengths of a hyperbolic reflection polygon have the closed range 0.5 to 3.5. Other length ranges start from SymmHub's 0.3 to 4. The parameter `aspect` of `*2222`, the length of side 0 over the length of side 1, has the range 0.125 to 8 and the default 1. The parameter `aspect` of `2222` is the `aspect` of the rectangle of `*2222` that its domain doubles, with the same range and default, because side 1 of that rectangle is not a side of the double. The rectangular wallpaper signatures `**`, `xx`, `*x`, `22*`, `22x`, and `2*22`, and `o`, have `aspect` with the same range and default. The parameter `skew` of `o` and `2222` has the closed range -0.5 to 0.5 and the default 0, and follows `aspect` (`DEC-52`). `4*2` and `3*3` have no parameter. `render` takes settings and `info` does not | Section 8.13 of `docs/research/fundamental-domains.md` gives SymmHub's ranges and its inconsistent twist units (`UNV-9`). The range of `aspect` is a choice of this proposal with no source, and its ends are exact in `f64`. On a prototype outside the repository, with every vertex of the polygon computed exactly and rounded once to `f64`, uniform-random settings in 0.3 to 4 with orders 2 to 12 failed `verify()` in 3 percent of the polygons with 6 corners, 11 percent with 7, and 27 percent with 8, and in 0.5 to 3.5 in none with up to 6 corners, 2 percent with 7, and 8 percent with 8. The failed checks were those of the isometries and the pairings, whose reflections have large normals. No box of lengths met a margin of 8 for every setting with 7 or 8 corners: in 1 to 3, 30 of 442 cases with 7 corners and 213 of 714 with 8 stayed below it. The range 0.5 to 3.5 covers every default with up to 8 corners (`DEC-13`), and `DEC-48` refuses the settings in it that fail `verify()`. The range of `skew` has the width of one first lattice vector, and its ends are exact in `f64`. No source gives it. On a prototype outside the repository, every setting of a grid of `aspect` and `skew` over both ranges and the random settings passed `verify()` for `o` and the sheared `2222` (DOM-54 of `docs/design/domain.md`). Item 45 of Section 15.3 of `docs/design/domain.md` puts the convention of `skew` to the owner |
| `DEC-15` | Colourings | A budget on search work with truncation reported, and ids as Section 5.7. The owner replaced one entry per conjugacy class with one entry per subgroup on 2026-11-18 (`DEC-37`) | The time of GAP's low-index search grows exponentially with the index (Section 7.7 of `docs/research/colourings.md`) |
| `DEC-16` | Sphere | The rotation is three angles in degrees, the yaw, the pitch, and the roll, which turn the ball about fixed axes of the screen in that order, with signs defined by the motion on the screen, default 0, 0, 0. One white directional light from the upper left front, fixed to the viewer, with the default direction $(-1, 1, 2)$. Phong shading by default, or none, with the constants $k_a = 0.25$, $k_d = 0.75$, $k_s = 0.25$, and $\alpha = 32$ and a specular term that cannot clip a channel. Borders and unconverged samples are shaded like fills, before the samples of a pixel are averaged. The pixel width uses the same estimate as elsewhere. Section 5.10 of `docs/design/render.md` states the formula, and Section 12.3 of that document the alternatives | The owner decided the lit ball with Phong shading, one light, and a rotation set on the command line (Section 10.1). Every other part of this row is a choice of this proposal with no source. The renderer keeps one border estimate in every model |
| `DEC-17` | Euclidean intrinsic unit | The normalization length of the domain, stated per signature: side 0 of a reflection triangle has length 1, and the rectangle of `*2222` has area 1. The domain of a rotation signature of `M6` keeps the unit of the reflection domain that it doubles, so side 0 of the domains of `632`, `442`, and `333` has length 1, and the domain of `2222` has area 2. The wallpaper domains of `o`, `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` of `M10` have area 1, and the kites of `4*2` and `3*3` keep side 0 of length 1, as the kites of `442` and `632` do. `skew` changes no area | An intrinsic width and the Euclidean view need a unit. This unit is a project convention. With area 1, each rectangular wallpaper domain is the rectangle of `*2222` at the same `aspect`, or that rectangle with split sides, so every side lies on a line of the grid of `*2222`. With area 1, `aspect` and its reciprocal give the same picture of `*2222` turned a quarter turn, which is an inference that no test checks, and the number of tiles in the view does not depend on `aspect`. This does not hold for `2222`, whose doubled wall is always side 1 of the rectangle: on a prototype outside the repository at 193 by 193 pixels, `2222` had 4417 border pixels with `aspect` 8 and 2487 with `aspect` 0.125 |
| `DEC-18` | MIT notice | Decided by the owner: a notices file and a section in the crate-level documentation, from the first change that ports SymmHub code | Section 10.1 |
| `DEC-19` | Completion and the general non-compact signatures | Decided by the owner: the general non-compact signatures move to the roadmap | Section 10.1 |
| `DEC-20` | Other notations in `info` | Omitted until a primary source is read | Tables A.1 and A.2 of [CBG] give primary names for the 17 plane groups and the spherical groups. The frieze names, the order of the numbers in five Coxeter symbols, and the Thurston column have no primary source (item 11 of Section 12 of `docs/research/orbifold-notation.md`) |
| `DEC-21` | Threads | Decided by the owner: `rayon` in the binary, although Miri reported undefined behaviour inside `crossbeam-epoch` | Section 10.1 |
| `DEC-22` | Publication | `publish = false` on every crate through the proof of concept | TOOL-27, and nothing is published without the owner's go-ahead |
| `DEC-23` | Information format | Text, covered by snapshots, and declared unstable. A machine-readable form is a roadmap item | A snapshot shows every change to the text for review |
| `DEC-24` | Bad orbifolds | `info` exits with a non-zero status and a diagnostic | A bad orbifold corresponds to no group (Section 7.2 of `docs/research/orbifold-notation.md`) |
| `DEC-25` | Orders | 32-bit integers, with checked exact rational arithmetic. A signature whose Euler characteristic, twice it, or the absolute value of either does not fit the `i64` over `NonZeroU64` rational type is rejected as an overflow. Section 8 of `docs/design/signature.md` states the detail | An overflow becomes an error and no wrong value |
| `DEC-26` | `Geometry` enum | In `escher-signature` | Section 3.4 |
| `DEC-27` | Performance budget | As Section 8, confirmed at `M3` | No published benchmark exists |
| `DEC-28` | Border estimate | Decided by the owner: the gradient form, recorded as `UNV-15` | Section 10.1 |
| `DEC-29` | Colour input | Decided by the owner: `#rgb` and `#rrggbb` only, read by a parser of the project, and no colour crate in `M3` | Section 10.1 |
| `DEC-30` | Existing output file | Decided by the owner: replaced without asking, with no `--overwrite` option | Section 10.1 |
| `DEC-31` | Normalization in the fold | No renormalization of the point, inside the fold loop or after it | Section 9.3. A prototype measured that renormalization inside the loop changes the sides that the fold applies near the rim |
| `DEC-32` | Colouring of a signature without reflections | By default, a signature without reflections renders in one colour, and `parity` for it is an error of `escher render`. Its colourings by id render as for any other signature (`DEC-44`). `escher-render` draws `parity` for such a domain in its first colour. `docs/design/cli.md` (CLI-33 to CLI-35) and `docs/design/render.md` (RND-33) specify it | The parity colouring needs a pairing that reverses orientation (Section 5.7), and an image in one colour for an explicit `parity` would hide that the requested colouring was not drawn. `DEC-7` already makes one colour the default when no two-colour colouring exists |
| `DEC-33` | Coset enumeration | The Felsch strategy, which defines the first undefined entry of the table, scans the relators that a new entry affects, and before it returns a table scans every relator at every coset in a closing pass. The coset limit counts every coset that a run defines, including coset 0, with the default 16384 and the maximum 1048576. Sections 6.3, 7, and 15 of `docs/design/group.md` state the steps and every limit | Defining the first undefined entry meets the hypothesis of the termination theorems for a finite index (Sections 7.12 and 7.15 of `docs/research/colourings.md`). With the closing pass, a returned table rests on the sourced statement that a closed table gives the index (Sections 7.9 and 7.15), and not on the steps that no book states at step level (`UNV-19`). The default is about four times the largest group order of a domain of `M6`, 4000 for `*(1000)22`. A run that reaches the limit shows nothing about the index, which the handbook of Holt, Eick, and O'Brien states and Sims does not. He says only that such a run stops without a result (Section 7.15 of `docs/research/colourings.md`) |
| `DEC-34` | Generator names and relator text | ASCII names numbered by kind: `x1`, `y1`, and `a1` for a handle, `g1` for a cone point, `r1` upward across the boundaries and `l1` for each boundary, and `z1` and `w1` for a crosscap, in the Conway presentation, and `s0` upward by side index in the side presentation. A relator is written with its letters separated by one space, an inverse as `name^-1`, and an exponent above 1 as `^n` after the base, with parentheses around any base other than one generator, such as `(r1 r2)^5` and `(g1^-1)^3`. Sections 4.4 and 5.1 of `docs/design/group.md` and Section 5.4 of `docs/design/domain.md` state the full forms | ASCII names need no replacement of characters on the command line. [CH] writes Latin and Greek letters (Section 9.1 of `docs/research/orbifold-notation.md`), which this proposal replaces. Side names from 0 match the side index that the fold records |
| `DEC-35` | `escher info --presentation` | The Conway presentation of the signature and, when a domain is built, the side presentation, each with the group order by coset enumeration only for a spherical signature whose group order is at most the default coset limit, and no option for the limit. Section 5.7 of `docs/design/cli.md` states the lines | Two independent readings of the group, from the signature and from the domain, can be compared. Only a spherical signature has a group order to compare with, and a run above the limit cannot finish (CLI-39 of `docs/design/cli.md`) |
| `DEC-36` | Side generators and coset tables | One generator for each side of a domain, with a pair relator for each pair of paired sides. Coset tables over right cosets with a right action, with coset 0 the subgroup and the column of each generator followed by that of its inverse | A colouring finds the generator of a fold step by its side index alone. Every source read uses right cosets and a right action (Section 7.21 of `docs/research/colourings.md`). The conversion to the left cosets of a colouring is the rule of Section 5.4 (`DEC-40`) |
| `DEC-37` | Colourings of conjugate subgroups | Decided by the owner: one colouring, id, and line for each subgroup of finite index, every conjugate included | Section 10.1 |
| `DEC-38` | Colourings in `info` | Decided by the owner: the number of colourings of each colour count by default, and every colouring with its id behind an option | Section 10.1 |
| `DEC-39` | Palette | Decided by the owner: a fixed table of 12 colours in `escher-cli`, no colour crate, `--colours` for more than 12 colours, and hexadecimal colours only | Section 10.1 |
| `DEC-40` | Colour of a sample | The number of the right coset $S g^{-1}$, traced from coset 0 of the colouring's own table through the inverse column of each side of the fold in reverse order, with coset 0 the colour of the domain (Section 5.4, GRP-38 of `docs/design/group.md`) | A prototype found this rule perfect on every class tried, the forward trace not perfect on every class that is not normal, and the reverse trace without the inverse the mirror image of the right colouring on rotation domains (Section 11.3 of `docs/design/group.md`). No source states the rule (`UNV-17`) |
| `DEC-41` | Search | The procedures of the handbook of Holt, Eick, and O'Brien with every relator used for deductions, every deduction pushed, a check of every relator at every coset of a complete table, an undo trail, and no subgroup argument. One table for each conjugacy class, by the first-in-class test that stops at an undefined entry, expanded into the subgroups of the class by relabelling from each coset (Sections 7.10 to 7.13 of `docs/design/group.md`) | The table layout and the relator lists of the enumeration serve the search unchanged. On a prototype, a split into short and long relators cost about 25000 times the steps, and comparing an undefined entry as 0 lost classes. The pruning is stated and argued by the sources without a proof (`UNV-19`) |
| `DEC-42` | Limits of the search | An index limit with the default 12 and the maximum 64, and a budget counted in table reads with the default $2^{28}$ and the maximum $2^{36}$. At most 65536 colourings and $2^{22}$ table entries stored by one call. One run to the limit first, and runs by index on a second budget only when that run stops. A stop at a limit returns a result complete to a lower colour count and is not an error (Sections 7.12 and 7.14 of `docs/design/group.md`) | Steps give the same result on every machine. $2^{28}$ steps took 0.4 to 0.65 s in a release build on a prototype, and runs by index cost 2.2 to 15.8 times one run, so a call pays for them only when needed. A limit on classes would not bound a listing of every subgroup |
| `DEC-43` | Ids | The rank among all the subgroups of a colour count, in the lexicographic order of their standard tables, written in lower-case bijective base 26 with up to 4 letters, and assigned only for a colour count that the search completed (Section 5.7, GRP-37 of `docs/design/group.md`) | Proposition 8.7 of Sims proves that this order is the order of the automata (Section 7.14 of `docs/research/colourings.md`). Assigning ids only for a complete colour count makes an id independent of the limit and the budget |
| `DEC-44` | Colouring of `render` | `render --colouring <id>` runs one search of the colour count of the id with the default budget, and `render` has no option for the budget. The default colouring stays that of CLI-34 of `docs/design/cli.md`, so a default rendering runs no search. CLI-43 of `docs/design/cli.md` states the resolution | The ids of a colour count rank all its subgroups, so the search of that count alone fixes them. The default keeps the bytes of every existing image |
| `DEC-45` | Coset fill | A coset fill in `escher-render`, which records the sides of each fold in a buffer of the sample and calls `colour_of`, and the dependency of `escher-render` on `escher-group`. RND-36 to RND-40 of `docs/design/render.md` state it | The first letter of the fold acts last on the cosets, so the coset cannot be carried forward through the fold (GRP-38 of `docs/design/group.md`) |
| `DEC-46` | Construction of a hyperbolic reflection polygon | SymmHub's corner pairs, slices, and sheets, applied to the corner order of the canonical signature (`DEC-9`), with each sheet turned only by a cyclic rotation and never by an exchange of two features, and SymmHub's lengths of the sheet edges. The side lengths of the polygon follow from the sheets, and one boundary walk in double-double arithmetic over them gives the vertices, each rounded once to `f64`. No weld of sheets and no solve for closure. Section 6.10 of `docs/design/domain.md` states the steps (DOM-39 to DOM-43) | The parameters are the cut lengths of SymmHub (Section 8.4 of `docs/research/fundamental-domains.md`), and the construction of the general hyperbolic signatures ports the same cut tree. SymmHub's code exchanges features and so puts the corners of some signatures in the wrong order (Section 6.8 of `docs/research/symmhub.md`). On a prototype outside the repository, in 50-digit arithmetic, 600 random signatures with 4 to 12 corners and orders up to 1000, at lengths from 0.3 to 4, closed to $5.4 \times 10^{-28}$ with the right angles and area, all convex, and the walk agreed with SymmHub's welded layout. A plain `f64` walk failed `verify()` in 15 to 96 percent of the random settings in 0.3 to 4 with 5 to 8 corners, and at the default for nearly every signature with a corner of order 1000, because the rounding of the factors of the walk breaks their isometry property and the walk amplifies the defect. A walk at 106 bits, which a double-double walk simulates, reached the margins of exact vertices rounded once: `*(1000)(1000)(1000)(1000)` went from a residual of check 5 of $4 \times 10^{-7}$ to $9 \times 10^{-14}$. The prototype implemented no double-double walk, so the crate measures it |
| `DEC-47` | Names of the parameters of a reflection polygon | `cornerpair<k>.length` and `slice<i>.length` (Section 5.6). `parameters` lists the corner pairs in the order of the signature, then the slices in the order of the cuts, $n - 3$ parameters for $n$ corners. So `*3222`, typed as `*2223`, has `cornerpair1.length`, `*22222` has `cornerpair1.length` and `cornerpair2.length`, `*3333` and `*5432` have `slice1.length`, and `*3322` has `cornerpair1.length` | SymmHub's keys in lower case. The names come from the canonical signature, so every spelling of a signature has the same names, and `*2233` and `*2332` both reach the construction as `*3322`. On a prototype outside the repository, 20000 random inputs and every sequence of orders 2 to 4 with 4 to 9 corners gave no orbifold two sets of names |
| `DEC-48` | Guard and limits of a reflection polygon | `build_with` returns the error kind `Numerical`, with the text `the shape is too extreme to build accurately`, unless the built polygon, or the double of one, passes the whole of `verify()` (DOM-44). A polygon has 4 to 8 corners, each of order up to 1000, and a signature with more orders is refused with `LimitExceeded` and a text such as `the signature has 9 orders, more than 8` (DOM-38) | `escher render` does not call `verify()`, so without the guard a setting could render a wrong polygon unnoticed (Hard Rule 7 of `AGENTS.md`). On a prototype outside the repository, a guard of checks 3 and 5 alone accepted `*(1000)2222` at its default, whose check 6 failed with $2.1 \times 10^{-9}$ against $10^{-9}$, and accepted 3451 exact cases that failed checks 1 and 2. On the same prototype, the whole `verify()` took 0.6 ms for a polygon of 63 corners of order 1000. The prototype also measured the margins of the defaults of every signature it tried, in two ways. One used exact vertices rounded once to `f64`. The other used a walk at 106 bits in mpmath, which simulated a double-double walk. Neither is a figure of the crate. With up to 8 corners and orders up to 1000, every default had a margin of at least 17. With up to 12 corners the smallest margin was 2.1, at `*(1000)` followed by eleven corners of order 2. With up to 12 corners and orders up to 200 it was at least 69. With exact vertices, the margin fell below 8 for signatures with 12 or more corners, such as 5.1 at 16 corners of order 1000. From 33 corners of order 1000, the Conway presentation exceeds the limit of 65536 letters of `escher-group`. Item 36 of Section 15.3 of `docs/design/domain.md` gives the alternatives |
| `DEC-49` | Placement of a reflection polygon | The centre, the normalized sum of the vertices, at the origin, and vertex 0 on the positive first axis | The centre rule is that of `DEC-12`. A fixed direction of vertex 0 makes the orientation of the image depend only on the signature and its settings. It is a choice of this proposal, which no source gives |
| `DEC-50` | Rotation signature of a reflection polygon | `p1...pn` with 4 to 8 orders is the double of the polygon of `*p1...pn` at its default shape across the wall of side 1, as `DEC-11` states for the triangles, and has no shape parameter, so every setting gives `UnknownParameter`. Its side numbering, and so its colouring ids (Section 5.7), are provisional: the milestone that builds the general hyperbolic signatures, `M15`, replaces this construction, renumbers its sides, and so changes its ids | The doubling of the triangles serves any number of sides. The parameters of `p1...pn` need the cuts of a sphere with cone points, which `M15` brings (Section 5.15 of `docs/design/poc-plan.md`). On a prototype outside the repository, which doubled polygons built by a plain `f64` walk, 14 of 120 random doubles with 5 orders and 117 of 120 with 8 fell below a margin of 8. The doubles are built from the polygon of `DEC-46`, and the crate measures their margins |
| `DEC-51` | Domains of the Euclidean groups of `M10` | Domains of the project's own, with nothing ported from SymmHub. `**`, `xx`, `*x`, and `22x` take the rectangle of `*2222`, with mirror lines and glide axes vertical where they can be either. `22*` and `2*22` take that rectangle with each horizontal side that a half-turn pairs split at its midpoint, which gives six and five sides. `o` takes a parallelogram with four sides paired by translations. `4*2` and `3*3` take the kites of `442` and `632` with two sides turned into mirrors. `2222` with a skew takes its hexagon of `M6`, sheared. The numbering of the sides depends on the canonical signature only and is part of the contract. DOM-50 and DOM-51 of `docs/design/domain.md` state the vertices, the partners, the kinds, and the pairings | SymmHub's catalogue sends `22*` and `2*22` to spherical functions, and its domains of those groups pair a side with itself, which DOM-26 of `docs/design/domain.md` forbids (Section 7.1 of `docs/research/symmhub.md`). The kites reuse the doubles of `M6`, their placement, and their accuracy, and they lie where the triangles of `*442` and `*632` lie. A rectangle for `o` with a shifted translation needs split sides whose number changes at a skew of 0, which would change the numbering. On a prototype outside the repository, each of the ten domains passed a certificate of one image of each sample point in the domain, of its lattice, and of its point group at every setting of the prototype's grid and its random settings, and the groups of a port of SymmHub's catalogue matched them up to a quarter turn and a scale. DOM-57 of `docs/design/domain.md` keeps that certificate as a test. No source states the pairings of these domains (`UNV-33`) |
| `DEC-52` | Skew | `skew` is the first coordinate of the second lattice vector divided by the length of the first, positive when the second vector leans right, with the range -0.5 to 0.5 and the default 0. The domain of `o` has the lattice vectors $(w, 0)$ and $(\mathrm{skew} \cdot w, h)$. The domain of `2222` is its hexagon of `M6` under the shear $(x, y) \mapsto (x + \mathrm{skew} \cdot \mathrm{aspect} \cdot y, \ y)$, and at a skew of exactly 0 it is the domain of `M6` bit for bit, with its ids and its golden image. This settles `UNV-9` for the Euclidean plane as a project convention. The twists of the hyperbolic cuts of `M15` and `M16` stay open | No source gives a skew convention (gap G11). A fraction of a lattice vector has no unit, so its range does not change with `aspect`, and a shear changes no area. SymmHub moves the half-turn centres of `2222` off the midpoints of its sides, which leaves a side that its pairing does not map onto a side (Section 7.1.7 of `docs/research/symmhub.md`). Section 6.13 of `docs/design/domain.md` states how the shear keeps the sides of `2222` on their partners and its horizontal sides horizontal, and DOM-53 records the reading of the range, and `UNV-33` holds what no source states about the sheared domains. On a prototype outside the repository, every vertex of angle $\pi$ of `22*`, `2*22`, and the sheared `2222` measured exactly $\pi$ at every setting of the prototype's grid and its random settings, a figure of the prototype, which the test "Wallpaper accuracy" of `docs/design/domain.md` must confirm on the crate |
| `DEC-53` | Guard of the wallpaper domains | `build_with` returns each of the ten domains of `M10`, the nine new ones and `2222` at every value of `skew`, only when it passes the whole of `verify()`, and the error kind `Numerical` with the text `the wallpaper domain fails its verification` otherwise (DOM-54 and Section 10.2 of `docs/design/domain.md`). `M10` adds no function to `escher-geometry`: each pairing is built from `euclidean::reflection`, `euclidean::translation`, and the compensated product of two reflections, as DOM-51 of `docs/design/domain.md` states | The constructions rest on `UNV-11` and `UNV-33`, and `escher render` does not call `verify()`, so without the guard a wrong pairing could render unnoticed (Hard Rule 7 of `AGENTS.md`). The cost is a few dozen products of 3 by 3 matrices. On a prototype outside the repository, no setting of the prototype's grid and its random settings failed `verify()`, and the smallest margin was $2.8 \times 10^{5}$, of check 2 on the sheared `2222` (Section 8.4 of `docs/design/domain.md`). `euclidean::rotation` of $\pi$ has the sine $1.2 \times 10^{-16}$ and not 0, so a half-turn is the product of two perpendicular reflections, whose entries are exact |
| `DEC-54` | Domain lines of `info` | The `Domain` line has three forms, chosen by the number of sides of kind `Mirror`: `<n> mirror sides` when every side is a mirror, `<n> sides in <k> pairs` when no side is, and `<n> sides, <m> mirror sides and <k> pairs` otherwise. A `Mirror sides` line then lists the mirror sides. The `Side pairs` line names the kinds `rotation`, `translation`, and `glide reflection`. CLI-32 and CLI-51 of `docs/design/cli.md` state the lines | `reverses_orientation()` is true for every new signature of `M10` except `o`, so it cannot tell a domain of mirrors from a domain with glide reflections. The three forms keep the text of every reflection domain and every rotation domain unchanged |
| `DEC-55` | Borders of a wallpaper domain | Every side of a domain draws a border, mirror or paired, with the same width and colour (RND-44 of `docs/design/render.md`). So several groups share their default picture: with the parity fill, `**` and `22*` give one image, and `*x`, `22x`, and `*2222` give another, and with one colour, the six rectangular groups give the image of `*2222`, `4*2` that of `442`, and `3*3` that of `632`. These groups differ in their colourings by id, and later in a motif. Whether mirror sides are drawn differently from paired sides is left to the owner, with no distinction as the default | A tile is a copy of the domain, so its whole boundary is drawn, as for the rotation domains (RND-34 of `docs/design/render.md`). The tiles and their borders of these groups coincide, and a parity fill sees only the orientation-preserving subgroup. On a prototype outside the repository, these images were equal byte for byte at `aspect` 1, 2, and 0.5 (Section 11 of `docs/design/render.md`). Point 40 of Section 12.3 of `docs/design/render.md` gives the alternative |

Items 34 to 39 of Section 15.3 of `docs/design/domain.md` put the defaults of the reflection polygons to the owner, with their alternatives: `DEC-13` as amended, `DEC-47`, the corner limit and the guard of `DEC-48`, `DEC-14` as amended, and `DEC-50`. Items 43 and 44 of Section 15.3 of `docs/design/domain.md` put the tiles of `DEC-51` to the owner, the kites of `4*2` and `3*3` and the layout of the rectangular groups, item 45 puts the convention of `DEC-52`, and point 40 of Section 12.3 of `docs/design/render.md` puts `DEC-55`. `DEC-51` to `DEC-55` and the amendments of `DEC-12`, `DEC-14`, and `DEC-17` for `M10` are proposals, and the owner has decided none of them.

## 11. Security Considerations

The threat model is a user who passes input from an untrusted party to `escher`: the signature, the `name=value` settings, the colours, and the numeric options. The attacker can choose any string within the limits of the command line. The attacker cannot change the binary, its dependencies, or the files of the repository.

The attacks in scope are exhaustion of memory and time. A large order, a long signature, a large image, a high fold cap, a coset enumeration, or a low-index search over a large index can each consume unbounded resources. A limit on the cosets of an enumeration bounds its memory and not its time, so Section 15 of `docs/design/group.md` states the residual time risk and Section 5.7 of `docs/design/cli.md` the gate of `info`. The low-index search counts its work in steps against a budget and limits what it stores, so its time and memory are bounded (`DEC-42`). Each crate design document therefore states its limits (Section 5.9), and the crate returns an error when an input exceeds one. Orders use checked arithmetic (`DEC-25`), so an overflow is an error.

The remaining topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the workspace denies `unsafe_code`, and no crate of this architecture needs it. No crate calls a native library at run time, and `DEC-3` keeps a native build out of the tests.
- File paths: the binary writes one PNG file to the path that the user gives and reads no file. It replaces an existing file at that path without asking (`DEC-30`), and Section 12 of `docs/design/cli.md` states what follows from that.
- Build scripts: no crate of this architecture has one.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`.
- Prior-art code: SymmHub's code is untrusted. Agents read it as a reference and port it by hand. Nothing in the build or the tests runs it, and ported code passes the same review and checks as any other code. Fixtures from GAP or `low_index` are generated outside the repository and committed as data (Section 14.4 of `docs/research/rust-ecosystem.md`).
- Secrets: no file in the repository contains a secret, and no workflow reads one (TOOL-22 in Section 7.2 of `docs/design/tooling.md`).

Attacks on the repository itself, such as a malicious change, and attacks on the machine that runs the tool are out of scope, because this document governs the crates and not access control.

The residual risk is a limit that a crate design document sets too high for the machine that runs the tool, which then runs slowly or runs out of memory before the limit applies. A second residual risk is a defect in ported code that review does not find.

## 12. Operational Considerations

No crate declares a Cargo feature (Section 3.1) or has a build script (Section 11), so no crate reads an environment variable at build time. The golden image tests read one environment variable at run time, `ESCHER_UPDATE_GOLDEN`, which the recipe of Section 7.2 sets and Section 14.2 of `docs/design/tooling.md` lists.

The supported platform of the proof of concept is Linux. The crates render on the CPU and need no graphics processing unit (GPU) or display, so `just check` runs every test (TOOL-8 in Section 3.1 of `docs/design/tooling.md`). The minimum supported Rust version (MSRV) is the one that Technology Decisions of `AGENTS.md` states.

Each library crate reports a failure through its `Error` and `ErrorKind` (Section 5.8). These failure modes recur across crates:

| Failure | Reported as | Recovery |
| --- | --- | --- |
| Malformed or bad signature | An error of `escher-signature` with a byte span. `info` exits with a non-zero status (`DEC-24`) | Correct the signature |
| Valid signature without a construction | The kind "unsupported signature". `render` exits with a non-zero status (Section 5.8) | None until its milestone in `docs/design/poc-plan.md`. None in the proof of concept for a general non-compact signature (`DEC-19`) |
| Parameter outside its range | An error of `escher-domain` with its documented kind | Choose a value in the range that `info` lists |
| Search stopped at its budget or at a limit on what it stores | No error. A result complete to a lower colour count, which `info` reports with a text that names a limit (`DEC-42`, CLI-47 of `docs/design/cli.md`) | Lower the maximum colour count |
| Coset enumeration over its bound | The limit error of `escher-group`, which claims nothing about the index | None. The error says that the run defined the limit and needed more, and a higher limit may or may not finish. Such a run shows nothing about the index, as the handbook of Holt, Eick, and O'Brien states (Section 7.15 of `docs/research/colourings.md`) |
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
