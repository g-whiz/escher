# Proof-of-Concept Milestone Plan

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-16 |
| Applies to | The order of work of the proof of concept, from the first crate to the completion audit |

## Abstract

This document is the milestone plan of the Escher proof of concept, a Linux command-line tool that renders PNG images of a planar symmetry group given in orbifold notation and reports information on it. It defines how every milestone proceeds, the research that must precede some milestones, and nineteen milestones in order. For each milestone it states the goal, the commands it adds, its scope, the design documents it writes, the sources and unverified statements it rests on, and the tests that accept it. It also lists the documentation that each milestone updates and every unverified statement that the proof of concept relies on.

## 1. Introduction

### 1.1. Problem

The proof of concept covers every planar symmetry that orbifold notation expresses, in three geometries, and much of its mathematics rests on code or on sources that the research could not read. Without an agreed order, an agent could start work whose mathematics has no subject-matter reference yet, or build on a statement that no source establishes without knowing it.

### 1.2. Purpose

This document fixes the order of work, what each milestone delivers, what it rests on, and how it is accepted. An implementing agent starts a milestone from its subsection in Section 5.

### 1.3. Audience

The primary readers are the coding agents that plan and implement a milestone. The secondary readers are reviewers and the owner of the repository, who approves the plan.

### 1.4. Scope

This document covers the workflow of a milestone, the owner's staging, the research prerequisites, the milestones M1 to M19, the documentation obligations of each milestone, and the unverified statements that the milestones rely on.

### 1.5. Non-Goals

Each fact has one owner, and this document cites the owner instead of restating the fact:

- `docs/design/architecture.md` owns the crates, the data flow, the conventions that cross crates, the testing strategy with the definition of a golden image, performance, numerical robustness, and the decisions. Section 10.1 of it records the owner's decisions, and Section 10.2 the proposed defaults `DEC-1` to `DEC-27`.
- `docs/design/roadmap.md` owns the phases after the proof of concept, the work that closes requested sources, and the Lean workstream.
- The subject-matter references under `docs/research/` own the mathematics.
- `AGENTS.md` owns the hard rules, the development workflow, and Project Status.

This document records no progress. It states the milestones as a plan, and Project Status in `AGENTS.md` is the only place that records which milestone has merged (Section 3.6 of `docs/writing-documentation.md`).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms that Section 2.2 of `docs/writing-documentation.md` defines, and calls that document the writing guide. It calls `docs/rust-coding-standards.md` the coding standard and `docs/design/tooling.md` the tooling design. It adds these terms:

- milestone: A unit of the plan with one goal, labelled `M1` to `M19`, delivered by one design branch and a few code branches. The planning documents of Section 3.3 carry the label `M0` and are not delivered this way.
- design branch: A branch named `feature/<name>-design` that changes only documents. It writes or extends the design documents of a milestone, and the research documents where a research prerequisite applies.
- code branch: A branch in its own worktree that changes code together with its tests and the document updates that Hard Rule 1 of `AGENTS.md` requires.
- research prerequisite: A design branch, labelled `RP-1` to `RP-5`, that adds to a research document what a milestone needs before its code can start (Section 4).
- unverified statement: A statement that a milestone relies on and that no cited source establishes, labelled `UNV-1` to `UNV-28` (Section 7).
- stage: One of the three stages of the owner's staging (Section 3.2).
- proposed default: A decision labelled `DEC-n`, recorded in Section 10.2 of `docs/design/architecture.md`. Section 10.1 of that document states the defaults that the owner has decided, which keep their labels.
- reference arithmetic: The independent high-precision implementation of a few formulas inside a test suite, which Section 7 of `docs/design/architecture.md` defines.
- golden image: The committed reference image of a rendering test, which Section 7 of `docs/design/architecture.md` defines.
- general non-compact signature: A valid signature with an infinite order that is none of the 7 friezes, the triangle groups with ideal corners, and their rotation subgroups, which Section 2.1 of `docs/design/architecture.md` defines.
- corpus: The set of signatures that a test runs over, such as the showcase signatures of SymmHub.
- exhaustive certificate: The test of a finite group that Section 5.5 defines.
- fold: The per-pixel loop that maps a point into the fundamental domain, as Section 4.1 of `rendering.md` describes it.
- Conway presentation: The presentation read from a signature, as Section 9.1 of `orbifold-notation.md` defines it.
- side presentation: The presentation read from the side pairings and vertex cycles of a built domain, as item 7 of Section 4.8 of `fundamental-domains.md` states it.

Research documents are cited by file name, and every one of them lives under `docs/research/`. In field lists and tables, a citation such as `rendering.md` 13.1 item 1 means item 1 of Section 13.1 of `docs/research/rendering.md`. A gap such as G5 is a gap of Section 13.2 of `fundamental-domains.md`. A bracketed tag, such as [DAVIS], names a source whose reference entry is in the research document that Section 10.2 names.

## 3. Milestone Pattern

### 3.1. Workflow of a Milestone

Every milestone proceeds in these steps:

1. A design branch writes or extends the design documents that the milestone names, and the research documents where a research prerequisite applies.
2. A few code branches follow, each in its own worktree, each with its tests and the document updates that Hard Rule 1 of `AGENTS.md` requires.
3. `just check` passes locally, the pushed branch passes continuous integration (CI), and `just merge` runs by the procedure of Section 14.7 of the tooling design.
4. The last code branch updates Project Status in `AGENTS.md`, as Section 6 states.

PLAN-1. An agent MUST merge the design branch of a milestone before it merges any code branch of that milestone, because the Development Workflow of `AGENTS.md` starts a change with its design document, and code merged first has no design for a reviewer to check it against.

PLAN-2. An agent MUST NOT start the code of a milestone before the research prerequisite that Section 4 names for it has merged, because a change that implements mathematics starts from its subject-matter reference (Hard Rule 14 of `AGENTS.md`), and code written first has nothing to be checked against.

PLAN-3. The design document of each crate MUST list every unverified statement of Section 7 that its code relies on, or else the statement cannot be traced and stays open after a source closes it.

PLAN-4. An agent MUST NOT treat a milestone as accepted until every test that its "Tests and acceptance" field names exists and passes in `just check` and in CI, because those tests are the only evidence that the milestone meets its goal.

Hard Rule 15 of `AGENTS.md` bars project documents and milestone wording from code, tests, and configuration, so no label of this document appears there, and a design document is the only place that records an unverified statement.

### 3.2. Staging

The owner decided to stage the proof of concept and to end it with the full notation. It is complete only when every valid signature renders, except the general non-compact signatures, which `DEC-19` of `docs/design/architecture.md` moves to the roadmap. The stages are these:

| Stage | Owner's scope | Milestones |
| --- | --- | --- |
| 1 | Reflection polygons and their rotation subgroups in all three geometries | M1 to M6, completed by M9 |
| 2 | All 17 Euclidean and all 14 spherical groups | M10, completed by M11 |
| 3 | General hyperbolic signatures, including handles and crosscaps, by porting the SymmHub pipeline, which is under the MIT licence | M15, completed for compact signatures by M16, with M17 |

The other milestones carry the owner's decisions on colourings (M7, M8), models (M12), borders (M13), and antialiasing (M14). The parser accepts signatures with infinite orders from M1. Rendering the friezes and the triangle groups with ideal corners is the late milestone M18, and M19 checks the completion criterion.

### 3.3. Planning Documents

The planning documents, labelled M0, are the precondition of M1 and have no code to implement. They are `docs/design/architecture.md`, this plan, and `docs/design/roadmap.md`, which live in `docs/design/` as plans without status (`DEC-2`), together with the updates to `AGENTS.md` (Project Status, and the planned locations in Repository Layout) and to `README.md`. They are accepted when `just check` passes and when they call every crate planned.

### 3.4. Milestone Overview

The column "Depends on" names the milestones and research prerequisites whose results a milestone uses, as Section 5 gives them.

| Label | Name | New command-line capability | Crates touched | Depends on |
| --- | --- | --- | --- | --- |
| M1 | Signature and classification | `escher info` for any signature | `escher-signature` (new), `escher-cli` | M0 |
| M2 | Linear geometry and the hyperbolic reflection triangle | Domain summary and `info --check` for `*pqr` | `escher-geometry` (new), `escher-domain` (new), `escher-cli` | M1 |
| M3 | First image | `escher render` of `*pqr` in the Poincare disk | `escher-render` (new), `escher-cli` | M2 |
| M4 | Euclidean reflection groups and the first shape parameter | Euclidean reflection groups, `--set` | `escher-geometry`, `escher-domain`, `escher-render`, `escher-cli` | M3 |
| M5 | The sphere | Spherical reflection groups, `--rotation`, `--light`, `--shading` | `escher-geometry`, `escher-domain`, `escher-render`, `escher-cli` | M3 |
| M6 | Rotation subgroups | `pqr`, `22n`, `nn`, `2222` | `escher-domain`, `escher-render`, `escher-cli` | M4, M5 |
| M7 | Presentations and coset enumeration | `info --presentation` | `escher-group` (new), `escher-domain`, `escher-cli` | M6, RP-1 |
| M8 | Colouring search with ids | Colouring list in `info`, `--max-colours`, `render --colouring <id>` | `escher-group`, `escher-render`, `escher-cli` | M7 |
| M9 | Hyperbolic reflection polygons with shape parameters | `*p1...pn` with its parameters | `escher-domain`, `escher-cli` | M6, RP-3 (sheet and slice part) |
| M10 | The 17 Euclidean groups | All 17 Euclidean signatures | `escher-domain` | M4, M8, RP-2 |
| M11 | The 14 spherical types | All 14 spherical types | `escher-domain` | M5, M7, RP-2 |
| M12 | Half-plane, band, and Klein models | `--model` | `escher-geometry`, `escher-render`, `escher-cli` | M3 |
| M13 | Intrinsic border width | `--border-unit intrinsic` | `escher-render`, `escher-geometry`, `escher-cli` | M3, with M4 and M5 for the other geometries and M12 for one test |
| M14 | Analytic border antialiasing | `--antialias analytic` | `escher-render`, `escher-cli` | M3, with M4, M5, and M12 for the comparison in each geometry and model |
| M15 | General hyperbolic signatures I | Cone points and mirrors, such as `4*3` and `2223` | `escher-domain` | M8, M9, RP-3 |
| M16 | General hyperbolic signatures II | Handles, crosscaps, several boundaries | `escher-domain` | M15 |
| M17 | Robust reduction for general domains | `--stats` reports no unconverged sample away from the rim | `escher-domain`, `escher-render` | M16 |
| M18 | Infinite-order points | Friezes, and triangle groups with ideal corners | `escher-domain`, `escher-render` | M10, M16, RP-4 |
| M19 | Completion audit | None | All six | M18 |

## 4. Research Prerequisites

The research documents hold no step-level description of Todd-Coxeter coset enumeration or the low-index subgroup search, little of the geometry of SymmHub's Euclidean and spherical catalogue, and no account of five of the nine cut types (`fundamental-domains.md` 8.5 and gap G4). A docs-only research branch therefore precedes each milestone that needs such material.

| Label | Adds | To | From | Before |
| --- | --- | --- | --- | --- |
| RP-1 | A step-level account of Todd-Coxeter coset enumeration and of the low-index subgroup search | `colourings.md` 7 | The `low_index` paper, the manual of Groups, Algorithms, Programming (GAP), and a line-level reading of SymmHub's `lib/sublib/src/sublib_core.js`. `symmhub.md` 17 item 8 records that the `Subgroup*.js` files were not read line by line | M7 |
| RP-2 | The geometry of the functions of SymmHub's `lib/orbilib/WallpaperGroups_NonNegative.js`, of which the research holds little beyond names | `fundamental-domains.md` 9.5 and 9.6, and `symmhub.md` 7.1 | SymmHub source code | M10, M11 |
| RP-3 | `atomizeOrbifold`, the twelve `unfoldAtom` cases, the weld, the walls, and `produceGenerators` at line level (gap G4) | `fundamental-domains.md` 8, and `symmhub.md` 6 | SymmHub source code | M15. Its sheet and slice part comes before M9 |
| RP-4 | `Group_Frieze`, and the ideal corners of the triangle groups and their rotation subgroups that M18 renders | `fundamental-domains.md` and `symmhub.md` | SymmHub source code | M18 |
| RP-5 | What [CH] pp. 255 to 257 states for a boundary without corners (`orbifold-notation.md` 12 item 14), and the clearing of the rows of `source-requests.md` 7, "Obtained Since" | `orbifold-notation.md` 9.1 and 12, and `source-requests.md` | [CH], which is held locally | No milestone. It may run at any time |

The research on the general non-compact signatures is not a prerequisite of any milestone. It is a deep-research item of Section 5 of `docs/design/roadmap.md`.

## 5. Milestones

Every command in this section is planned. Each output is what the milestone delivers, and `docs/design/cli.md` defines the exact text. Each subsection uses these fields in this order: Goal, Command line, Scope, Design documents, Sources, Unverified statements relied on, Tests and acceptance, Dependencies and `deny.toml` work, Risks, and Split where one applies.

### 5.1. M1: Signature and Classification

**Goal.** The full notation is parsed, validated, canonicalized, and classified.

**Command line.** These commands illustrate the milestone:

```sh
escher info '*732'
escher info '*532'
escher info '4*3'
escher info '*3333'
escher info '2*(12)'
escher info '*22(inf)'
escher info '*22∞'
escher info 23
escher --version
```

- `escher info '*732'` prints the canonical form `*732`, hyperbolic, $\chi = -1/84$, compact, area $\pi/42$, and 0 parameters (`orbifold-notation.md` 6.1, 7.3, and 10.1).
- `escher info '*532'` prints spherical, $\chi = 1/60$, and order 120 (`orbifold-notation.md` 6.4).
- `escher info '4*3'` prints $\chi = -1/12$ (`orbifold-notation.md` 6.4).
- `escher info '*3333'` prints 1 parameter (`orbifold-notation.md` 6.4 and 10.1).
- `escher info '2*(12)'` reads the order 12 in parentheses and prints spherical and order 48, the order $4n$ of `2*n` (`orbifold-notation.md` 8.2).
- `escher info '*22(inf)'` and the same signature with the infinity sign print Euclidean, frieze, and non-compact.
- `escher info 23` fails with the message "bad orbifold" and a caret under the signature.
- `escher --version` keeps printing the package name and version.

**Scope.** The new crate `escher-signature`. `escher-cli` gains `clap` subcommands and the `run` entry.

- Parser: hand-written recursive descent over `char_indices`, with byte spans (`rust-ecosystem.md` 9.1).
- Grammar: `orbifold-notation.md` 3, with parentheses for orders of 10 or more (3.4) and both spellings of infinity.
- Euler characteristic: an exact rational by the defect rule, including the two defects of infinite order (`orbifold-notation.md` 6.1).
- Bad orbifolds (7.2), classification (7.1), order and area (7.3), the deformation dimension of compact hyperbolic signatures (10.1), the equivalence rules 1 to 6 (4.1), and a canonical form, all in `orbifold-notation.md`.

**Design documents.** `docs/design/signature.md`, and `docs/design/cli.md` with the commands, options, output format, exit status, and diagnostics. The first tests also carry the Miri policy of `DEC-5`, and Section 6 of `docs/design/architecture.md` lists the documents that change with it.

**Sources.** Every item of the scope is primary in `orbifold-notation.md`, from [CH], [TH], [FG], and [TG].

**Unverified statements relied on.**

- `UNV-20`: the canonical form, parentheses, the spelling of infinity, order 1, and the order of features. No primary source defines them (`orbifold-notation.md` 3.4, 3.5, 5, and 12 items 1 to 4). SymmHub and Tegula are the only precedents.
- `UNV-21`: the geometry of non-compact signatures with $\chi \ne 0$ (`orbifold-notation.md` 12 item 15). The deformation dimension is printed only for compact hyperbolic signatures.

**Tests and acceptance.**

- Tables: the 17 signatures of `orbifold-notation.md` 8.1 have $\chi = 0$. The 14 types of 8.2 give the listed orders for $n$ from 1 to 12. The 7 friezes of 8.3, the examples of 6.4, and the four bad families of 7.2 give their stated results.
- Enumeration: a bounded enumeration of canonical compact signatures finds exactly 17 with $\chi = 0$ (Section 32 of [FG], `orbifold-notation.md` 7.1).
- Properties: a random sequence of the equivalence rules leaves the canonical form and $\chi$ unchanged. The canonical form is idempotent. Display followed by parsing round-trips.
- Snapshots: `insta` snapshots of the info text and of every error message.
- Failure paths with their error kinds (TEST-6).

**Dependencies and `deny.toml` work.** `thiserror`, `clap`, `anyhow`, and the dev-dependencies `insta` and `proptest`. This change adds the first `[workspace.dependencies]` table.

- The licences come from `cargo deny check licenses` (`rust-ecosystem.md` 18.2). The expected ones are `Apache-2.0`, which `insta` needs, possibly `MIT`, and `Unicode-3.0` for `unicode-ident` (`rust-ecosystem.md` 18.1, unverified).
- The change first confirms that each licence is compatible with GPL-3.0-or-later (TOOL-24). `rust-ecosystem.md` 21 records the statements on `MIT` and `Unicode-3.0` as general knowledge.
- The expected duplicate is `syn` 2 and 3, with `proc-macro2` and `quote` (`rust-ecosystem.md` 19, "likely the first failure"). The change adds `skip` entries with reasons only if no alignment of versions removes the duplicate (TOOL-26).
- The change updates the printed `deny.toml` block and the sentences of Section 8 of the tooling design.

**Risks.**

- Rule 4 with a crosscap, and rule 5 (`ox` equals `xxx`, stated only by [TG]), make the canonical form subtle.
- Degenerate inputs: `1`, `*`, `x`, and the empty string.
- The dependency tree may need more exceptions than predicted, because every statement of `rust-ecosystem.md` 19 is unverified.

**Split.** The canonical form can follow as a second branch.

### 5.2. M2: Linear Geometry and the Hyperbolic Reflection Triangle

**Goal.** The linear hyperbolic model and the first domain exist, checked against reference arithmetic.

**Command line.** These commands illustrate the milestone:

```sh
escher info '*732'
escher info --check '*732'
```

- `escher info '*732'` adds a domain summary: three mirror sides, the corner angles, the side lengths, and the area, equal to $-2\pi\chi$ (`orbifold-notation.md` 7.3).
- `escher info --check '*732'` prints the result of the verification.

**Scope.** The new crate `escher-geometry`, with the `glam` types of `DEC-4` and the free functions of the conventions over them (Section 5.1 of `docs/design/architecture.md`), the hyperbolic module, and the disk map in both directions. The new crate `escher-domain`, with the compact `*pqr` for $1/p + 1/q + 1/r < 1$.

- Construction: the side lengths from the angles, the vertices placed with the boost and rotation of [KC24], and each wall normal as the unit vector orthogonal to its two vertices.
- `verify()` checks the isometry condition, the Coxeter relators numerically, the angle sum, and the area.

**Design documents.** `docs/design/geometry.md` and `docs/design/domain.md`.

**Sources.**

- Reflection: equation (6.6) of [DAVIS] (`geometry-models.md` 7.4).
- Isometries and distance: Theorem 10.1 and the lemma on p. 83 of [CFKP] (`geometry-models.md` 3.1 and 6.1). Boost and rotation: p. 3 of [KC24] (`geometry-models.md` 6.1).
- Existence and uniqueness of the triangle: Fact 3 of [CFKP] (`geometry-models.md` 10.1) and Example 6.5.3 of [DAVIS] (`fundamental-domains.md` 9.1).
- Group and strict fundamental domain: Theorem 6.4.3 of [DAVIS] and Section 4.4 of [MASKIT2001] (`fundamental-domains.md` 4.4 and 4.5).
- The class is fully sourced (`fundamental-domains.md` 3 and 13.1).

**Unverified statements relied on.**

- `UNV-12`: the angle form of the law of cosines rests on a secondary source and on HyperRogue's code (`geometry-models.md` 10.2). Remark 5.5 of [MASKIT2001] identifies the same form (`fundamental-domains.md` 9.3).
- `UNV-13`: the direction from the disk to the hyperboloid is secondary (`geometry-models.md` 4.2 and 15.1 item 7).
- `UNV-26`: the normal through two points, and the absence from every research document of a relation between the inner product of two unit normals and the corner angle. The design does not rely on such a relation, and the relator and area checks cover the result.
- `UNV-28` applies if SymmHub's disk triangle (`geometry-models.md` 7.6) serves as a test oracle, because its properties are candidate statements (`geometry-models.md` 15.1 item 5).

**Tests and acceptance.**

- Reference arithmetic: an independent high-precision implementation in the test suite recomputes lengths, vertices, and normals for triples that include (7,3,2), (5,4,2), (4,4,4), (3,3,4), (12,3,2), and (1000,3,2).
- Primary numeric checks for $\{7, 3\}$ from Table 2 and equations 24 and 29 of [HT]: $h = 1.090550$ and $r_0 = 0.300743$ (`geometry-models.md` 10.4), through the split of the polygon into $2p$ right triangles (`geometry-models.md` 10.1).
- Properties: for random admissible triples, the relators hold, the inverse equals $J M^{t} J$, and the disk map round-trips.

**Dependencies and `deny.toml` work.** `glam` (`DEC-4`) for `escher-geometry` and `escher-domain`, and the reference-arithmetic dev-dependency `astro-float` (`DEC-3`). Each is declared once in `[workspace.dependencies]` (DEP-2).

- `glam` is under `MIT OR Apache-2.0`, and `astro-float` under `MIT` (`rust-ecosystem.md` 6, 12, and 18.1). The change confirms with `cargo deny check licenses` that the allow-list covers both, and adds a licence only where the check reports one (TOOL-24).
- `glam` has no default dependencies, and its types `DVec2`, `DVec3`, and `DMat3` come from its feature `float-types`, which is on by default (`rust-ecosystem.md` 6). `rust-ecosystem.md` 19 lists no duplicate version for `glam` or `astro-float`.
- `glam` is a public dependency of `escher-geometry` and `escher-domain` under rule DEP-3, with the obligations that Section 3.5 of `docs/design/architecture.md` states.
- `astro-float` needs no native build (`rust-ecosystem.md` 12).
- The change updates Section 8 of the tooling design for every change to `deny.toml`.

**Risks.** Slips in the conventions, precision at very large orders, and the maturity of the reference crate (`UNV-25`). `glam` uses single instruction, multiple data (SIMD) intrinsics internally, and its feature `scalar-math` disables them (`rust-ecosystem.md` 6). The research does not record whether that code runs under Miri.

### 5.3. M3: First Image

**Goal.** The vertical slice is complete: fold, flat colours, pixel borders, supersampling, and PNG output. The main rendering risks, which are termination of the fold, precision at the limit circle, and speed, are measured.

**Command line.** This command writes a 512 by 512 image in the Poincare disk with black borders:

```sh
escher render '*732' -o tiling.png
```

`render` takes these options:

- `--size 1024x768`: the image size.
- `--colouring one|parity`: one colour, or the two-colour parity colouring.
- `--colours '#e9c46a,#2a9d8f'`: the fill colours.
- `--border-width 1.5`: the border width in pixels.
- `--border-colour`: the border colour.
- `--samples 4`: the sample grid, here 4 by 4.
- `--max-folds`: the fold cap.
- `--background`: the background colour.
- `--threads`: the number of threads (`DEC-21`).
- `--stats`: prints the time, the sample count, the mean and maximum number of folds, and the number of samples that reached the cap.

**Scope.** The new crate `escher-render`. The binary gains `render`.

- View: the canonical view places the centre of the domain at the centre of the disk.
- Fold: the first violated side (`rendering.md` 4.1), with a tolerance, a cap, and periodic renormalization.
- Colour: parity from the orientation of the pairings.
- Border: a distance estimate in screen space from the gradient of the wall function pulled back through the fold (`UNV-15`).
- Sampling: a regular n by n grid averaged in linear light (precedent in `rendering.md` 7.3).
- Samples outside the disk take the background. Samples that reach the cap follow `DEC-7`.

**Design documents.** `docs/design/render.md` and an extended `cli.md`. For golden images, as Section 7 of `docs/design/architecture.md` defines them, the change also extends Sections 3.2 and 14.2 of the tooling design and Section 11 of the coding standard.

**Sources.**

- The per-pixel method: `rendering.md` 3, 4.1, and 4.2.
- Strict fundamental domain: Theorem 6.4.3 of [DAVIS].
- Published caps and behaviour at the cap: `rendering.md` 4.4 and 4.9.
- The disk metric: [CFKP] (`rendering.md` 6.2).
- Border definitions in published code: `rendering.md` 6.1.
- Independence of pixels: `rendering.md` 11.

**Unverified statements relied on.** `UNV-1` (termination and step count), `UNV-3` (points on walls), `UNV-4` (growth of the fold count, and precision), `UNV-15` (distance estimate), and `UNV-18` (parity). Averaging in linear light is a design choice that no research source supports.

**Tests and acceptance.**

- Fold properties: the result lies inside the domain, folding a reflected point gives the same result, and the fold is idempotent.
- Fold results for points up to the disk radius $1 - 10^{-6}$ against reference arithmetic, including words of a few hundred reflections. `symmhub.md` 5.4 reports a trace loss of $5 \times 10^{-5}$ on long words in a hyperboloid factorization.
- Border oracle: the border mask equals, within one pixel, the mask from a change of tile identity between neighbouring samples.
- A symmetric view, available only to tests, gives an image equal to its mirror image.
- Golden images of `*732`, `*542`, and `*433` at a small size.
- The PNG file decodes to the requested size.
- One process-level smoke test of the binary.

The performance expectation of Section 8 of `docs/design/architecture.md` is confirmed here (`DEC-27`).

**Dependencies and `deny.toml` work.** `png`, `palette` without default features, `rayon`, and `csscolorparser`. `escher-render` also declares `glam` from the workspace entry of M2, which adds no licence.

- Predicted duplicates: `miniz_oxide` 0.8 and 0.9, and `phf` 0.13 and 0.14 (`rust-ecosystem.md` 19, both unverified). `cargo tree -d` decides between a choice of features, a pin, or a `skip` with a reason.
- New licences are added only as cargo-deny reports them.

**Risks.** Speed. The appearance of the rim. Golden images that differ between the container and CI because of the platform mathematics library. The estimate near vertices.

**Split.** (a) A single-threaded renderer with one sample per pixel. (b) Supersampling, parallel rows, and statistics.

### 5.4. M4: Euclidean Reflection Groups and the First Shape Parameter

**Goal.** `*333`, `*442`, `*632`, and `*2222`, and the parameter machinery from end to end.

**Command line.** These commands illustrate the milestone:

```sh
escher render '*442' -o e.png
escher info '*2222'
escher render '*2222' --set aspect=1.618 -o r.png
```

- `escher info '*2222'` lists one parameter with its name, range, and default.
- `--set` takes a `name=value` setting.

**Scope.** A Euclidean module in `escher-geometry`, with the affine plane, line covectors, reflection, translation, and rotation. The Euclidean triangle and rectangle in `escher-domain`. A canonical Euclidean view (`DEC-12`). `ParameterSpec` and `--set`.

**Design documents.** Extensions of `geometry.md`, `domain.md`, `render.md`, and `cli.md`.

**Sources.** Section 6.3, Theorem 6.4.3, and Example 6.5.2 of [DAVIS] (`fundamental-domains.md` 9.1). The rectangle: Example 13.1.3 of [TH] (`orbifold-notation.md` 9.4). The rectangle family: pp. 7 and 14 of [VONGAGERN2009] (`fundamental-domains.md` 9.5).

**Unverified statements relied on.** `UNV-10`, because no source states whether the one parameter of `*2222` includes the scale (gap G7). `UNV-26` for the Euclidean reflection as a matrix.

**Tests and acceptance.** Relators. Fold properties. Exact periodicity of the image when the view makes a translation a whole number of pixels. Golden images. A snapshot of the parameter listing. Range errors.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Small. The Euclidean form is degenerate, which `geometry-models.md` 13.6 records as needing special inverses.

### 5.5. M5: The Sphere

**Goal.** The spherical reflection groups `*332`, `*432`, `*532`, `*22n`, and `*nn`, drawn as a lit ball in orthographic projection with Phong shading, one light, and a viewing rotation that the command line sets, as the owner decided.

**Command line.** These commands illustrate the milestone:

```sh
escher render '*532' -o ico.png
escher render '*532' --rotation 20,35,0 -o ico-rotated.png
```

`render` gains these options:

- `--rotation 20,35,0`: the viewing rotation.
- `--light`: the light direction.
- `--shading phong|none`: Phong shading, or none.

**Scope.** A spherical module with the unit sphere, reflections, rotations, and distance. The lift onto the ball, with the rotation. Spherical triangles from the spherical law of cosines, and the lune for `*nn`. A fold with a bound from the group order. Phong shading with one light.

**Design documents.** Extensions of `geometry.md`, `domain.md`, `render.md`, and `cli.md`.

**Sources.**

- The sphere as a linear model: [KC24] (`geometry-models.md` 11.1). Reflection: Section 6.4 of [DAVIS].
- Triangles: Example 6.5.1 of [DAVIS]. The sector of two mirrors: p. 82 of [DAVIS] (`fundamental-domains.md` 4.5).
- Orders: `orbifold-notation.md` 8.2. Finite folds: `rendering.md` 9.3. Phong shading: `geometry-models.md` 11.6.

**Unverified statements relied on.** `UNV-12` (the spherical law of cosines is secondary), `UNV-16` (the lift, the view scale, and the Phong formulas, whose primary paper is unread), and `UNV-14` for the distance to a great circle if it is used.

**Tests and acceptance.**

- Exhaustive certificate: a breadth-first enumeration of the generated matrices has exactly $2/\chi$ elements (`orbifold-notation.md` 7.3), the images of an interior point are distinct, and every sample folds within the bound.
- With shading off, rotating the view by a group element leaves the image unchanged.
- The brightest point follows the light direction.
- Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Aliasing at the limb. Borders of pixel width on a view that is not conformal (`DEC-16`). The defaults for the light and the material.

### 5.6. M6: Rotation Subgroups

**Goal.** The first side pairings that are not reflections: `pqr` in all three geometries (`732`, `333`, `442`, `632`, `332`, `432`, `532`), `22n`, `nn`, and `2222` on the rectangle.

**Command line.** These commands illustrate the milestone:

```sh
escher render 732 -o rot.png
escher render 532 -o i.png
```

`info` shows the paired sides and reports that no parity colouring exists.

**Scope.** The domain is the double of the reflection domain across one wall, with products of two reflections as pairings. The fold runs with general pairings.

**Design documents.** Extensions of `domain.md` and `render.md`. `domain.md` states which wall is doubled, and that a side fixed by a half-turn is split at the fixed point, as item 2 of `fundamental-domains.md` 4.8 requires.

**Sources.**

- Sections 4.4 and 5.9 of [MASKIT2001] (`fundamental-domains.md` 9.1 and 7.6).
- The polygon theorem for orientation-preserving groups: Theorem 12 of [SOLOMYAK2020] and Theorem 6.14 of [SERIES2013] (`fundamental-domains.md` 4.1 and 4.2).
- The kite, and "two copies of the triangle": p. 12 of [VONGAGERN2009] (`fundamental-domains.md` 9.5).
- SymmHub's double triangle (`symmhub.md` 7.2.7).

**Unverified statements relied on.**

- `UNV-2`: the greedy fold with general pairings.
- `UNV-5`: the spherical and Euclidean orientation-preserving cases rest on [JESPERS2015], a secondary source (`fundamental-domains.md` 4.6). The exhaustive certificate closes the spherical cases.
- `UNV-22`: the signature of the rotation subgroup for the general rule (gap G10).

**Tests and acceptance.** The checklist of `fundamental-domains.md` 4.8 as `verify()`. Cycle relators. Spherical orders. Fold properties. Every border pixel of the `pqr` image is a border pixel of the `*pqr` image in the same view. Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** The order of the sides and the split at a half-turn. The first exposure to `UNV-2`.

### 5.7. M7: Presentations and Coset Enumeration

**Goal.** The crate `escher-group` exists, with presentations and Todd-Coxeter coset enumeration, as the base of colourings and as an independent check of domains.

**Command line.** This command prints the generators and relators of a signature:

```sh
escher info --presentation '*532'
```

For a spherical signature it also prints "order by coset enumeration: 120, equal to 2/chi".

**Scope.** `Word`, `Presentation`, the Conway presentation of a signature (`orbifold-notation.md` 9.1), the side presentation of a domain, and bounded Todd-Coxeter enumeration. The side presentation has Coxeter relators for mirrors, and pairing and vertex-cycle relators otherwise, as `symmhub.md` 14.2 describes SymmHub doing.

**Design documents.** `docs/design/group.md`, and an extension of `domain.md`. Prerequisite RP-1.

**Sources.** [CH] pp. 255 to 257. Theorem 6.4.3 of [DAVIS] and the theorems of M6 for side presentations. `colourings.md` 7.2 for coset tables in GAP. `rust-ecosystem.md` 14, which records that no usable crate exists and that `algebraeon-groups` is under GPL-3.0-only and is not copied.

**Unverified statements relied on.** `UNV-19` (the algorithms without their primary texts), `UNV-22` (a boundary without corners), and `UNV-5` (cycle relators in mixed cases, from M10 on).

**Tests and acceptance.**

- The Conway presentations of all 14 spherical types, for $n$ from 1 to 12, enumerate to $2/\chi$.
- The side presentations of every domain built so far give the same orders.
- `x`, `22`, and `*` each have order 2 ([CH] p. 257, `orbifold-notation.md` 4.2).
- An infinite group reaches the bound and returns the limit error.

**Dependencies and `deny.toml` work.** None.

**Risks.** The enumeration strategy, and blow-up on relator orders that suit it poorly.

### 5.8. M8: Colouring Search With Ids

**Goal.** The low-index subgroup search, colouring ids, the listing in `info`, and coset colours in `render`.

**Command line.** These commands illustrate the milestone:

```sh
escher info '*642'
escher info --max-colours 20 '*642'
escher render '*642' --colouring 3a --colours crimson,gold,navy -o c.png
```

- `escher info '*642'` lists every colouring found up to 12 colours, with its id and colour count, and the set of available counts.
- `--max-colours 20` raises the bound to 20 colours.
- `render --colouring 3a` draws the colouring with the id `3a`. Without `--colours`, the built-in palette applies.

**Scope.** A Sims-style backtrack over partial coset tables, with relator checks, pruning by conjugacy, and standardized output. A budget with reported truncation (`DEC-15`). Ids. The coset from the side sequence of the fold. A palette generated in a perceptual colour space with `palette`.

**Design documents.** Extensions of `group.md`, `render.md`, and `cli.md`.

**Sources.** Coset colourings and Lemma 2.1 (`colourings.md` 4.1). The algorithm and its cost (`colourings.md` 7.1 to 7.3 and 7.7). SymmHub's per-pixel colour and assembled algorithm (`colourings.md` 8, `symmhub.md` 12). SymmHub's `sublib` as the implementation to study.

**Unverified statements relied on.** `UNV-17`: Lemma 2.1 is proved only for the triangle tiling of $G_{p,q}$ (`colourings.md` 4.2), and the composition order is the code's own claim (`colourings.md` 8.4). Whether the listing is complete, and how conjugate subgroups relate to colourings, is open (`colourings.md` 3.4 and 14.1 items 1 to 3). Also `UNV-19`.

**Tests and acceptance.**

- Tables 1 and 2 of Frettloh (`colourings.md` 5.6 and 5.7), reproduced up to a feasible index for `*732`, `*832`, and `*542` by keeping the subgroups that contain the generators of Lemma 3.2 or Lemma 4.1.
- The cube: 1, 3, and 6 colours, each unique (`colourings.md` 5.8).
- Seven subgroups of index 2 for an extended triangle group with three even periods ([SINGERMAN2014], `colourings.md` 6).
- Each count in Bulatov's captions is found (`colourings.md` 4.4, existence only).
- Fixtures from GAP or `low_index`, generated offline (`rust-ecosystem.md` 14.4). SymmHub validated its search against 703 GAP files (`prior-art.md` 4.1).
- Perfectness on rendered samples: each generator permutes the colours by one permutation ([FRETTLOH2008] p. 2).
- Ids are identical across runs and across thread counts.

**Dependencies and `deny.toml` work.** None.

**Risks.** The number of subgroups grows fast with the number of generators and the index. `colourings.md` 7.3 gives 653 classes for the free group of rank 3 at index 4, and `colourings.md` 7.1 gives exponential time. The default bound of 12 colours therefore needs the budget for signatures with handles, crosscaps, or many cone points. The test for conjugacy is the hardest part.

**Split.** (a) The search and `info`. (b) `render --colouring` and the palette.

### 5.9. M9: Hyperbolic Reflection Polygons With Shape Parameters

**Goal.** Compact `*p1...pn` with $n \ge 4$ and its $n - 3$ parameters. This milestone completes stage 1.

**Command line.** These commands illustrate the milestone:

```sh
escher info '*2223'
escher render '*22222' -o pent.png
escher render '*3333' --set slice1.length=1.2 -o slice.png
```

- `escher info '*2223'` lists the parameter `slice1.length`.
- The rotation subgroups `p1...pn` render at the default shape by doubling. Their full parameter set arrives in M15.

**Scope.** The construction of the polygon from slices and sheets, as SymmHub does it (`fundamental-domains.md` 8.5 to 8.9), laid out with linear isometries. The default shape (`DEC-13`). The ranges (`DEC-14`).

**Design documents.** An extension of `domain.md`, after the sheet and slice part of RP-3.

**Sources.**

- Existence, discreteness, and strict fundamental domain for compact polygons: Example 6.5.3 and Theorem 6.4.3 of [DAVIS].
- The parameter count $n - 3$: Theorem 13.8.1 and p. 316 of [TH] (`fundamental-domains.md` 6.1 and 9.4).
- Coordinates: only SymmHub (`fundamental-domains.md` 3: "Partly, with no sourced coordinates").

**Unverified statements relied on.** `UNV-6` and `UNV-7` for producing the polygon, `UNV-27` (ranges and default), and `UNV-12` for the pentagon and hexagon relations if they are used (`geometry-models.md` 10.5, read only through summaries). The reliance is limited, because `verify()` checks the hypotheses of Theorem 6.4.3 of [DAVIS] on each built polygon: closure, convexity, and corner angles $\pi/p_i$.

**Tests and acceptance.** Regular right-angled polygons against equations 24 and 29 of [HT]. Area equal to $-2\pi\chi$. Random parameters within range pass `verify()`. Relators. Parameters out of range fail with the documented kind. Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Errors from `UNV-6`. Extreme parameters.

**Split.** As a fallback if the milestone grows, the closure is solved numerically from the side lengths, and SymmHub's formulas are kept for M15.

### 5.10. M10: The 17 Euclidean Groups

**Goal.** The remaining nine Euclidean groups, `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, and the skew of `2222`. Translations and glide reflections become pairings.

**Command line.** These commands illustrate the milestone:

```sh
escher render '3*3' -o p31m.png
escher render o --set skew=0.25 -o p1.png
```

`escher info` reports all 17 Euclidean signatures with their parameters.

**Scope.** Explicit constructions in the affine plane, and parameters by signature. Prerequisite RP-2.

**Design documents.** An extension of `domain.md`.

**Sources.** The signatures: Table II of [CH] and Theorem 13.3.6 of [TH]. The kites of `3*3` and `4*2`, and the relations of index 2: pp. 7 and 12 of [VONGAGERN2009]. The catalogue and its recorded defects: `fundamental-domains.md` 9.5 and 8.14, `symmhub.md` 7.1 and 15.3. Presentations: [CH].

**Unverified statements relied on.** `UNV-10` (parameter counts, from the partial counts of [VONGAGERN2009], the unsourced table of `orbifold-notation.md` 12 item 7, and SymmHub's catalogue), `UNV-11`, `UNV-5`, `UNV-2` for translations and glides in the fold, `UNV-8` for glide pairings, and `UNV-9` for skew and twist.

**Tests and acceptance.** A table-driven test over the 17 signatures of `orbifold-notation.md` 8.1. `verify()`. Relators. Equal counts of subgroup classes up to a small index between the side presentation and the Conway presentation. Periodicity of the image. The borders of `3*3` lie inside the borders of `*632`, and those of `4*2` inside those of `*442`. Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** The catalogue has known wrong entries (`2222`, `2*22`, `22*`), so each construction is designed and verified here and not ported blindly.

**Split.** By family: translations and glides, then the groups with half-turns, then the kites.

### 5.11. M11: The 14 Spherical Types

**Goal.** The remaining `n*`, `nx`, `2*n`, and `3*2`, and the small cases `1`, `*`, `x`, `22`, `*22`, and `2*`. This milestone completes stage 2.

**Command line.** These commands illustrate the milestone:

```sh
escher render '3*2' -o th.png
escher render '5x' -o s10.png
```

`escher info` reports every type.

**Scope.** Constructions on the unit sphere with rotations, reflections, and rotary reflections. Domains that are lunes or hemispheres. Prerequisite RP-2.

**Design documents.** An extension of `domain.md`.

**Sources.** The signatures and orders: Table III of [CH] and Theorem 13.3.6 of [TH]. The reflection triangles are sourced, and the other generators exist only as catalogue code (`fundamental-domains.md` 9.6 and 13.1).

**Unverified statements relied on.** `UNV-11`, `UNV-5` (the exhaustive certificate closes the spherical cases), the coincidences at small $n$ (`orbifold-notation.md` 12 item 8), and `UNV-20` for order 1.

**Tests and acceptance.** For every type and $n$ from 1 to 12, the exhaustive certificate of M5 and the order from Todd-Coxeter enumeration. A table-driven render of all 31 Euclidean and spherical entries. Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Degenerate domains, and the view for them.

### 5.12. M12: Half-Plane, Band, and Klein Models

**Goal.** `--model half-plane|band|klein`, with the Poincare disk as the default.

**Command line.** This command renders in the band model:

```sh
escher render '*732' --model band -o band.png
```

The milestone can move to any place after M3.

**Scope.** Model maps and their Jacobians in `escher-geometry`, a canonical view for each model, and borders through the gradient estimate, so that the Klein model needs no special case.

**Design documents.** Extensions of `geometry.md`, `render.md`, and `cli.md`.

**Sources.** The connecting maps and metrics of [CFKP] (`geometry-models.md` 4.1 and 3.1, `rendering.md` 8.1). [KC24] for the Klein model (`geometry-models.md` 4.2). The band map and its constant (`geometry-models.md` 5.1). The evaluation of the complex inverse hyperbolic tangent by logarithms ([B1], `geometry-models.md` 5.1).

**Unverified statements relied on.** `UNV-13`. The inverse of the band map is stated only in SymmHub's code (`rendering.md` 13.1 item 14), the band metric was read only through a summary, and the correspondence of the half-plane is a project choice (`geometry-models.md` 4.3).

**Tests and acceptance.** Round trips against reference arithmetic. The same hyperbolic point has the same tile colour in all four models. Borders in the Klein model are straight (Section 9 of [CFKP]). Golden images.

**Dependencies and `deny.toml` work.** `num-complex`, with the licence and duplicate checks.

**Risks.** View conventions. Periodicity of the band is not a goal (`geometry-models.md` 5.3).

### 5.13. M13: Intrinsic Border Width

**Goal.** `--border-unit intrinsic`.

**Command line.** This command draws borders 0.05 wide in the metric of the geometry:

```sh
escher render '*732' --border-unit intrinsic --border-width 0.05 -o intrinsic.png
```

**Scope.** The exact distance from the folded point to the nearest wall in each geometry.

**Design documents.** Extensions of `render.md` and `geometry.md`.

**Sources.** Precedents with intrinsic widths (`rendering.md` 6.1). Formulas (`rendering.md` 6.2 and 9.4, `geometry-models.md` 9.3).

**Unverified statements relied on.** `UNV-14`. The form on the hyperboloid rests on a secondary citation and on code, and the form on the sphere is a candidate statement.

**Tests and acceptance.** The formula against an independent high-precision minimization of the distance between points. The band around a vertical geodesic of the half-plane is a wedge (Fact 2 of [CFKP], which needs M12). Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Small. `DEC-17` defines the Euclidean unit.

### 5.14. M14: Analytic Border Antialiasing

**Goal.** One sample per pixel with analytic border coverage, compared with supersampling.

**Command line.** This command renders with analytic antialiasing:

```sh
escher render '*732' --antialias analytic -o analytic.png
```

**Scope.** A coverage ramp one pixel wide on the screen-distance estimate, in SymmHub's form. A policy where tiles are smaller than a pixel. With isometries in a linear model, the factor "tracked through the fold" is the accumulated isometry together with the Jacobian of the model map. The design document states this and marks the estimate as `UNV-15`.

**Design documents.** An extension of `render.md`.

**Sources.** SymmHub's form of the ramp (`rendering.md` 6.1, `symmhub.md` 11.1). Tiles smaller than a pixel (`rendering.md` 7.4).

**Unverified statements relied on.** `UNV-15` (`rendering.md` 13.1 items 19 and 20).

**Tests and acceptance.** The mean and maximum difference against a supersampled reference on an 8 by 8 grid, for each model and geometry, with thresholds in the design document, and separate thresholds near vertices and near the rim.

**Dependencies and `deny.toml` work.** None new.

**Risks.** This method does not smooth colour edges that have no border, and the design document says so.

### 5.15. M15: General Hyperbolic Signatures I, Cone Points and Mirrors

**Goal.** The SymmHub pipeline for genus 0 with one boundary: pillows, hats, sheets, tubes, slices, folds, and cone pairs. Examples are `2223`, `3333`, `22222`, `4*3`, `2*23`, and `23*2`.

**Command line.** These commands illustrate the milestone:

```sh
escher render '4*3' -o a.png
escher info 2223
escher render 2223 --set tube1.twist=0.2 -o twist.png
```

`escher info 2223` lists two parameters with their names.

**Scope.** A port of the normalization, atomization, unfolding, assembly, and generators (`fundamental-domains.md` 8, `symmhub.md` 6 and 8.1). The combinatorics and the scalar formulas are ported. The layout and the generators are rebuilt from linear isometries, so circles and complex numbers appear only where a formula cannot be restated. Prerequisite RP-3.

**Design documents.** An extension of `domain.md`. The first code branch that ports SymmHub code adds the MIT notice in the places that `DEC-18` names.

**Sources.** The classes are code only (`fundamental-domains.md` 3 and 13.1). The sourced parts are the deformation dimension (Corollary 13.3.7 of [TH]), the orientable generators of [MASKIT2001] (`fundamental-domains.md` 7) as a cross-check for signatures without mirrors, and the presentations of [CH].

**Unverified statements relied on.** `UNV-5`, `UNV-6`, `UNV-7`, `UNV-9`, `UNV-2`, `UNV-11` for the mixed signatures, `UNV-26` for the conversion of circles into normals, `UNV-24` for the MIT code, and the defects of `fundamental-domains.md` 8.14 and `symmhub.md` 15.3.

**Tests and acceptance.**

- `verify()` on a corpus that includes SymmHub's showcase signatures (`symmhub.md` 3.5).
- Area equal to $-2\pi\chi$. The parameter count equals the dimension formula.
- Equal low-index counts between the side presentation and the Conway presentation.
- For signatures without mirrors, traces against Maskit's matrices. The worked example needs gap G12 checked first.
- Statistics of fold convergence on the corpus. Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** The largest of the plan: the cut rules that the research has not recovered, assembled domains that may not be convex, and known defects.

**Split.** (a) Pillows, with cone points only. (b) Hats, with cone points and a mirror. (c) Parameters and twists.

### 5.16. M16: General Hyperbolic Signatures II, Handles, Crosscaps, and Several Boundaries

**Goal.** `o2`, `oo`, `xxx`, `23x`, `2*x`, `**2`, and `o*`. This milestone completes stage 3 for compact signatures.

**Command line.** These commands illustrate the milestone:

```sh
escher render o2 -o torus.png
escher render 23x --set cap1.length=1.5 -o cap.png
```

**Scope.** Handle pairs with length and twist, caps as glide reflections, and bands (`fundamental-domains.md` 8.4 and 8.11, `symmhub.md` 6.7).

**Design documents.** An extension of `domain.md`.

**Sources.** Code only. [NORBURY2006], a secondary source, on one-sided geodesics (`fundamental-domains.md` 6.3 and 6.5). Maskit's closers of handles for the orientable cases (`fundamental-domains.md` 7.8 and 7.10).

**Unverified statements relied on.** `UNV-2`, `UNV-5`, `UNV-6`, `UNV-7`, `UNV-8`, `UNV-9`, and gap G15 (several boundaries).

**Tests and acceptance.** As M15, with orientation-reversing cycles in `verify()`, and colourings of these groups under the budget.

**Dependencies and `deny.toml` work.** None new.

**Risks.** As M15, and the explosion of the search of M8 for surface groups.

### 5.17. M17: Robust Reduction for General Domains

**Goal.** Every sample of every corpus signature converges or is reported.

**Command line.** `--stats` reports zero unconverged samples away from the rim for the corpus.

**Scope.** An audit of convexity and convergence. If it is needed, a remedy: walls (`symmhub.md` 9.4), or reduction against a Dirichlet domain with the rule of Algorithm 4.3 of [VOIGHT2009], which decreases the distance (`fundamental-domains.md` 10.1 to 10.3). Its termination argument is sourced for Fuchsian groups only.

**Design documents.** Extensions of `domain.md` and `render.md`.

**Sources.** `symmhub.md` 9.4, and Algorithm 4.3 of [VOIGHT2009] (`fundamental-domains.md` 10.1 to 10.3).

**Unverified statements relied on.** `UNV-2`. `DEC-11` decides which domain is drawn.

**Tests and acceptance.** Convergence on the corpus. Fold properties under random parameters.

**Dependencies and `deny.toml` work.** None new.

**Risks.** If a Dirichlet domain is needed, this is a large milestone.

**Split.** The milestone merges into M15 and M16 if the audit shows that the assembled domains are convex.

### 5.18. M18: Infinite-Order Points, Friezes and Ideal Vertices

**Goal.** Rendering of the friezes and of the triangle groups with ideal corners.

**Command line.** These commands illustrate the milestone:

```sh
escher render '*22(inf)' -o frieze.png
escher render '*23(inf)' -o ideal.png
```

**Scope.** Two parts:

1. The 7 friezes (`orbifold-notation.md` 8.3).
2. Triangle groups with ideal corners and their rotation subgroups (Sections 4.4 and 5.9 of [MASKIT2001], `fundamental-domains.md` 4.4 and 7.6, and SymmHub's ideal triangle, `geometry-models.md` 7.6.5).

The general non-compact signatures are outside the proof of concept and are a workstream of `docs/design/roadmap.md` (`DEC-19` of `docs/design/architecture.md`). The research records no implementation for them, because SymmHub's orbifold parser has no infinite order (`symmhub.md` 4.2), and the only sourced material is Maskit's cusps for orientable signatures (`fundamental-domains.md` 7).

**Design documents.** Extensions of `domain.md` and `render.md`. Prerequisite RP-4.

**Sources.** The sources named in part 2, the frieze list of `orbifold-notation.md` 8.3, and SymmHub's `Group_Frieze` as RP-4 records it.

**Unverified statements relied on.** `UNV-23` for the fold near an ideal vertex (Section 9.5 of `docs/design/architecture.md`), `UNV-21` for the geometry of the triangle signatures with an infinite order, and `UNV-26` for the conversions into the linear model.

**Tests and acceptance.** The parabolic cycle condition in `verify()` (item 5 of `fundamental-domains.md` 4.8). Samples near an ideal point reach the cap and take the colour that `DEC-7` of `docs/design/architecture.md` gives to samples at the cap. Golden images. A test that `escher render` rejects a general non-compact signature as Section 5.8 of `docs/design/architecture.md` states, with an `insta` snapshot of the diagnostic.

**Dependencies and `deny.toml` work.** None new.

**Risks.** No source establishes that the fold applies at an ideal vertex (`UNV-23`), so the images near an ideal corner depend on the behaviour at the fold cap.

### 5.19. M19: Completion Audit

**Goal.** Every valid signature renders, except the general non-compact signatures (`DEC-19`).

**Command line.** No new capability.

**Scope.** A property test that generates valid signatures up to a size bound. It renders each signature that is not general non-compact at a small size without error, and checks that `render` rejects each general non-compact signature as Section 5.8 of `docs/design/architecture.md` states. The snapshot corpus of `info`. The reconciled list of unverified statements across the design documents. The final usage in `README.md`, and Project Status in `AGENTS.md`.

**Design documents.** Every crate design document, for the reconciled list of unverified statements.

**Sources.** None.

**Unverified statements relied on.** None.

**Tests and acceptance.** The property test of the scope passes, and every crate design document meets PLAN-3.

**Dependencies and `deny.toml` work.** None.

**Risks.** The audit exposes classes of signature that the corpus missed.

## 6. Documentation Obligations

Each milestone updates these documents in the same change as its code (Hard Rule 1 of `AGENTS.md`). The documents that mirror configuration, which Section 6 of `docs/design/architecture.md` lists, change in the same change as the first crates, dependencies, and recipes.

1. The design document of each crate that the milestone alters, before the code, with its assumptions, its open questions, and the list of PLAN-3. It records no status.
2. The research documents: the prerequisite readings, and any source read, in the same change as `source-requests.md`.
3. `AGENTS.md`: Project Status with its date, Repository Layout for each new crate, file, and recipe, and Technology Decisions when a decision takes force.
4. `README.md`: the crate table, the requirements, and the usage.
5. The tooling design: Section 8 for every change to `deny.toml`, Section 3.2 for a recipe, Section 4 for the nextest profiles, and Section 14.2 for environment variables.
6. The coding standard: the member list of Section 1.1, and Section 11 for new test rules.
7. `typos.toml` for new correct words.

## 7. Unverified Statements

The owner decided how the proof of concept treats a statement that no source establishes. The milestone proceeds and does not wait for the source. The design document that relies on the statement flags it, and the tests check it numerically against high-precision reference arithmetic. The statement stays tracked until a source or a machine-checked Lean proof closes it. Nothing in this table blocks the proof of concept. `docs/design/roadmap.md` holds the work that closes these statements, and the research location of each row is where the closing is recorded.

| Id | Statement relied on | Research location | Milestones that use it | What closes it |
| --- | --- | --- | --- | --- |
| UNV-1 | The first-violated-wall fold terminates for compact reflection polygons, in a number of steps equal to the word length | `rendering.md` 13.1 item 1, gap G5 | M3 on | Sections 1.6, 1.7, and 5.4 of [HUMPHREYS], or Lean |
| UNV-2 | The same fold terminates for general side pairings and domains that are not convex | `rendering.md` 13.1 item 7, `fundamental-domains.md` 10.7 and 13.3 item 3, `symmhub.md` 17 item 5 | M6, M10, M15 to M17 | A source, a change to Voight's rule, or Lean |
| UNV-3 | Behaviour on or near a wall | `rendering.md` 13.1 item 2 | M3 on | Tests. No source is known |
| UNV-4 | Growth of the fold count, the reading of the exponents, and the limits of precision | `rendering.md` 13.1 items 5 and 6, `geometry-models.md` 15.1 item 10 | M3 defaults | The body of [FWW], and measurement |
| UNV-5 | The polygon theorem with reflections mixed with other pairings, glide cycles, mirror corners beside paired sides, and ideal reflection polygons with more than three sides | `fundamental-domains.md` 4.7, gap G1 | M6, M7, M10, M11, M15, M16 | [EPSTEIN-PETRONIO1994], Theorem 11.2.2 of [RATCLIFFE], Theorem 9.8.4 of [BEARDON] |
| UNV-6 | The generalized law of cosines with truncated vertices, and the pillow and hat formulas | `fundamental-domains.md` 8.7, 8.8, and 13.3 item 1, gap G3 | M9, M15, M16 | [BUSER], Section 3.5 of [RATCLIFFE] |
| UNV-7 | Deaton's cut rules, the twelve atoms, and the twist in three situations | Gap G4, `fundamental-domains.md` 13.3 items 4 and 5 | M9, M15, M16 | [DEATON1993] |
| UNV-8 | The cap generator is a glide reflection consistent with a polygon theorem | Gap G2, `fundamental-domains.md` 13.3 item 2 | M10, M16 | [CHOI2012], [BUSER] |
| UNV-9 | The twist convention | Gap G11 | M10, M15, M16 | A project convention |
| UNV-10 | Euclidean parameter counts | Gap G7, `orbifold-notation.md` 12 item 7 | M4, M10 | [CBG], [KLR], [SCOTT] |
| UNV-11 | Constructions of mixed signatures, and spherical generators | Gap G8 | M10, M11, M15 | [CBG] |
| UNV-12 | Angle forms of the laws of cosines, and other trigonometry from secondary or summarized sources | `geometry-models.md` 10.2, 10.5, 11.3, and 15.2 | M2, M5, M9 | Sections 7.18 to 7.20 of [BEARDON], chapter 2 of [BUSER], Section 3.5 of [RATCLIFFE] |
| UNV-13 | Inverse model maps, the inverse and metric of the band, and the choice for the half-plane | `geometry-models.md` 4.3, 5.1, and 15.1 item 7, `rendering.md` 13.1 item 14 | M2, M12 | [HUB], and round-trip tests |
| UNV-14 | Distance to a geodesic and to a great circle in linear form | `geometry-models.md` 9.3 and 15.1 item 3, `rendering.md` 13.1 items 8 and 10 | M5, M13 | Chapter 2 and Section 3.2 of [RATCLIFFE] |
| UNV-15 | The screen-distance estimate through the fold, analytic antialiasing, and the fade of tiles smaller than a pixel | `rendering.md` 7.1 and 13.1 items 19 and 20 | M3, M14 | Measurement against supersampling |
| UNV-16 | The lift onto the ball, the view scale of the sphere, and the Phong formulas | `rendering.md` 13.1 items 11 and 12, `geometry-models.md` 11.6 | M5 | [PHONG75], [BLINN77] |
| UNV-17 | Coset colourings are perfect for every orbifold group, the composition order, completeness, and conjugacy | `colourings.md` 4.2, 8.4, 8.7, and 14.1 items 1 to 3 | M8 | [DDL2009], [SCHWARZENBERGER1984], [CBGS2008], or Lean |
| UNV-18 | The parity colouring | `colourings.md` 6, `rendering.md` 4.6 | M3 | A primary text on non-Euclidean crystallographic (NEC) groups |
| UNV-19 | Todd-Coxeter enumeration and the low-index search without their primary texts | `colourings.md` 7 and 14.3 | M7, M8 | [SIMS1994], [HOLT2005] |
| UNV-20 | Text conventions and the canonical form | `orbifold-notation.md` 12 items 1 to 4 | M1, M11 | [CBG], [BH], or else a project convention |
| UNV-21 | Geometry and parameters of non-compact signatures with $\chi \ne 0$ | `orbifold-notation.md` 12 item 15 | M1, M18 | A source |
| UNV-22 | The Conway presentation for a boundary without corners, the NEC dictionary, and the signature of the rotation subgroup | `orbifold-notation.md` 12 items 5, 12, and 14, gap G10 | M6, M7 | A re-reading of [CH], [SINGERMAN1974] |
| UNV-23 | Whether the results of [DAVIS] for polytopes and the fold apply to a domain with an ideal vertex | `rendering.md` 13.1 item 3, `geometry-models.md` 7.6.5 | M18 | The sources of UNV-5 |
| UNV-24 | MIT code inside a work under GPL-3.0-or-later | `symmhub.md` 17 item 1 | M15 | The licence texts, and the owner's judgement |
| UNV-25 | Correct rounding of the reference-arithmetic crate | `rust-ecosystem.md` 21 item 5 | M2 on | One comparison with MPFR |
| UNV-26 | Statements on the linear model that the research lacks: the normal through two points, normals and the corner angle, a disk circle to a normal, and the Euclidean reflection as a matrix | None | M2, M4, M15, M18 | Chapter 3 of [RATCLIFFE], or Lean |
| UNV-27 | Construction, ranges, and default shape of polygons with prescribed angles | `geometry-models.md` 15.1 item 14, `fundamental-domains.md` 9.4 | M9 | A detailed reading of the proof in [TH], [BUSER] |
| UNV-28 | Properties of SymmHub's disk triangle, if it is used as an oracle | `geometry-models.md` 15.1 item 5 | M2 tests | Tests only |

## 8. Security Considerations

This document defines no component, so the threat model is that of the tool it plans. The attacker is assumed to control the input of `escher`: the signature string, every option value, every `name=value` setting, and the output path. Every milestone that adds such an input states, in the Security Considerations of its design document, the limits that Section 5.9 of `docs/design/architecture.md` requires: the maximum order, the signature length, the image size, the fold cap, and the search budget. Without these limits a crafted signature or option exhausts memory or time, for example through the colouring search of M8, whose cost grows exponentially with the index.

The output path is the only file that `render` writes, and no milestone reads a file supplied by the user. Every dependency enters through the milestone that first needs it, with the licence and advisory checks of cargo-deny (Section 8 of the tooling design). A native build, such as `rug`, would add code that runs at build time, which is one reason why `DEC-3` chooses the pure-Rust `astro-float`. Ported SymmHub code carries its MIT notice under `DEC-18`, and `UNV-24` tracks the open licence question.

Attacks on the repository itself, such as a malicious change to a document or a compromised CI runner, are out of scope, because `AGENTS.md` and the tooling design govern the repository and its workflow. The residual risk is that a limit chosen in a design document is too high for the machine that runs the tool, and that an unverified statement of Section 7 makes the tool draw a wrong image without any error.

## 9. Operational Considerations

The crates declare no Cargo feature during the proof of concept, as Section 3 of `docs/design/architecture.md` states. M3 adds the environment variable of the recipe that updates golden images, and documents it with its default in Section 14.2 of the tooling design. The only supported platform of the proof of concept is Linux, and rendering runs on the CPU only. The minimum supported Rust version (MSRV) is the one that the workspace declares, and `check-msrv` verifies it.

The plan has these failure modes:

| Failure | Handling |
| --- | --- |
| A milestone grows beyond a design branch and a few code branches | It splits along its Split field |
| A research prerequisite finds no construction | The milestone proceeds on the unverified statements of Section 7, or the owner decides to move the work to the roadmap (`DEC-19`) |
| A test reads a file, starts a process, or runs long numeric loops | It is excluded from Miri under the policy of `DEC-5` |
| A golden image differs between the container and CI | The comparison limits of Section 7 of `docs/design/architecture.md` apply, and a golden file changes only through its recipe |
| A CI job is cancelled because no runner acquired it | The job is rerun, and the merge waits (Section 14.4 of the tooling design) |

The tool emits no logs. `--stats` prints its statistics on request.

## 10. References

### 10.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 10.2. Informative References

- [B1] The entry in Section 16.1 of `docs/research/geometry-models.md`.
- [BEARDON] The entry in Section 14.3 of `docs/research/fundamental-domains.md`. `geometry-models.md` cites the same book as [BEA].
- [BH] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [BLINN77] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [BUSER] The entry in Section 14.3 of `docs/research/fundamental-domains.md`. `geometry-models.md` cites the same book as [BUS].
- [CBG] The entry in Section 13.4 of `docs/research/orbifold-notation.md`. `fundamental-domains.md` cites the same book as [SYMMETRIES-OF-THINGS2008].
- [CBGS2008] The entry in Section 15 of `docs/research/colourings.md`. It is the same book as [CBG].
- [CFKP] The entry in Section 16.1 of `docs/research/geometry-models.md`.
- [CH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`. `fundamental-domains.md` cites the same paper as [CONWAY-HUSON2002].
- [CHOI2012] The entry in Section 14.3 of `docs/research/fundamental-domains.md`.
- [DAVIS] The entry in Section 16.1 of `docs/research/geometry-models.md`. `fundamental-domains.md` cites the same book as [DAVIS2008].
- [DDL2009] The entry in Section 15 of `docs/research/colourings.md`.
- [DEATON1993] The entry in Section 14.3 of `docs/research/fundamental-domains.md`.
- [EPSTEIN-PETRONIO1994] The entry in Section 14.3 of `docs/research/fundamental-domains.md`.
- [FG] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [FRETTLOH2008] The entry in Section 15 of `docs/research/colourings.md`.
- [FWW] The entry in Section 16.2 of `docs/research/geometry-models.md`.
- [HOLT2005] The entry in Section 15 of `docs/research/colourings.md`.
- [HT] The entry in Section 16.1 of `docs/research/geometry-models.md`.
- [HUB] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [HUMPHREYS] The entry in Section 14 of `docs/research/rendering.md`.
- [JESPERS2015] The entry in Section 14.2 of `docs/research/fundamental-domains.md`.
- [KC24] The entry in Section 16.1 of `docs/research/geometry-models.md`. `rendering.md` cites the same paper as [KOPCZ24].
- [KLR] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [MASKIT2001] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [NORBURY2006] The entry in Section 14.2 of `docs/research/fundamental-domains.md`.
- [PHONG75] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [RATCLIFFE] The entry in Section 14.3 of `docs/research/fundamental-domains.md`. `geometry-models.md` cites the same book as [RAT].
- [SCHWARZENBERGER1984] The entry in Section 15 of `docs/research/colourings.md`.
- [SCOTT] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [SERIES2013] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [SIMS1994] The entry in Section 15 of `docs/research/colourings.md`.
- [SINGERMAN1974] The entry in Section 14.3 of `docs/research/fundamental-domains.md`.
- [SINGERMAN2014] The entry in Section 15 of `docs/research/colourings.md`.
- [SOLOMYAK2020] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [TG] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [TH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`. `fundamental-domains.md` cites the same notes as [THURSTON2002].
- [VOIGHT2009] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [VONGAGERN2009] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| PLAN-1 | The design branch of a milestone merges before any of its code branches | 3.1 |
| PLAN-2 | The code of a milestone starts only after its research prerequisite has merged | 3.1 |
| PLAN-3 | Each crate design document lists the unverified statements its code relies on | 3.1 |
| PLAN-4 | A milestone is accepted only when every test it names passes in `just check` and CI | 3.1 |
