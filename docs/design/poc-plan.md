# Proof-of-Concept Milestone Plan

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-18 |
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

- `docs/design/architecture.md` owns the crates, the data flow, the conventions that cross crates, the testing strategy with the definition of a golden image, performance, numerical robustness, and the decisions. Section 10.1 of it records the owner's decisions, and Section 10.2 the proposed defaults `DEC-1` to `DEC-36`.
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
- unverified statement: A statement that a milestone relies on and that no cited source establishes, labelled `UNV-1` to `UNV-32` (Section 7).
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
| M8 | Colouring search with ids | Colouring counts in `info`, `--colourings`, `--max-colours`, `render --colouring <id>` | `escher-group`, `escher-render`, `escher-cli` | M7 |
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

Some milestones need material that a research document must supply first: a step-level account of Todd-Coxeter coset enumeration, the low-index subgroup search, and Reidemeister-Schreier rewriting, the geometry of SymmHub's Euclidean and spherical catalogue, of which the research documents hold little, and an account of five of the nine cut types, which they lack (`fundamental-domains.md` 8.5 and gap G4). A docs-only research branch therefore precedes each milestone that needs such material.

| Label | Adds | To | From | Before |
| --- | --- | --- | --- | --- |
| RP-1 | A step-level account of Todd-Coxeter coset enumeration, of the low-index subgroup search, and of Reidemeister-Schreier rewriting, with SymmHub's implementation and worked examples | `colourings.md` 7.8 to 7.21, and `symmhub.md` 12.8 | The sources that `colourings.md` 7.8 lists, with a line-level reading of SymmHub's `lib/sublib/src/sublib_core.js` and `reidemeister.js`. `symmhub.md` 17 item 8 records which files were read line by line | M7 |
| RP-2 | The geometry of the functions of SymmHub's `lib/orbilib/WallpaperGroups_NonNegative.js`, of which the research holds little beyond names | `fundamental-domains.md` 9.5 and 9.6, and `symmhub.md` 7.1 | SymmHub source code | M10, M11 |
| RP-3 | `atomizeOrbifold`, the twelve `unfoldAtom` cases, the weld, the walls, and `produceGenerators` at line level (gap G4) | `fundamental-domains.md` 8, and `symmhub.md` 6 | SymmHub source code | M15. Its sheet and slice part comes before M9 |
| RP-4 | `Group_Frieze`, and the ideal corners of the triangle groups and their rotation subgroups that M18 renders | `fundamental-domains.md` and `symmhub.md` | SymmHub source code | M18 |
| RP-5 | What [CH] pp. 255 to 257 states for a boundary without corners (`orbifold-notation.md` 12 item 14), and the clearing of the rows of `source-requests.md` 7, "Obtained Since" | `orbifold-notation.md` 9.1 and 12, and `source-requests.md` | [CH], which is held locally | No milestone. It may run at any time |

The research on the general non-compact signatures is not a prerequisite of any milestone. It is a deep-research item of Section 5 of `docs/design/roadmap.md`.

## 5. Milestones

Each command in this section illustrates its milestone, and Project Status in `AGENTS.md` says which milestones are complete. Each output is what the milestone delivers, and `docs/design/cli.md` defines the exact text. Each subsection uses these fields in this order: Goal, Command line, Scope, Design documents, Sources, Unverified statements relied on, Tests and acceptance, Dependencies and `deny.toml` work, Risks, and Split where one applies.

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

- `escher info '*732'` prints the canonical form `*732`, hyperbolic, $\chi = -1/84$, compact, area $\pi/42$, and deformation dimension 0 (`orbifold-notation.md` 6.1, 7.3, and 10.1).
- `escher info '*532'` prints spherical, $\chi = 1/60$, and order 120 (`orbifold-notation.md` 6.4).
- `escher info '4*3'` prints $\chi = -1/12$ (`orbifold-notation.md` 6.4).
- `escher info '*3333'` prints deformation dimension 1 (`orbifold-notation.md` 6.4 and 10.1).
- `escher info '2*(12)'` reads the order 12 in parentheses and prints spherical and order 48, the order $4n$ of `2*n` (`orbifold-notation.md` 8.2). Without the parentheses, `escher info '2*12'` is an error.
- `escher info '*22(inf)'` and the same signature with the infinity sign print the canonical form `*(inf)22`, Euclidean, frieze, and non-compact.
- `escher info 23` fails with the message "bad orbifold" and a caret under the signature.
- `escher --version` keeps printing the package name and version.

Section 5 of `docs/design/cli.md` gives the exact text of each command, and its Section 6.4 gives the exact diagnostics.

**Scope.** The new crate `escher-signature`. `escher-cli` gains `clap` subcommands and the `run` entry.

- Parser: a hand-written scanner over `char_indices`, with byte spans (`rust-ecosystem.md` 9.1).
- Grammar: `orbifold-notation.md` 3, with parentheses for orders of 10 or more (3.4) and both spellings of infinity.
- Euler characteristic: an exact rational by the defect rule, including the two defects of infinite order (`orbifold-notation.md` 6.1).
- Bad orbifolds (7.2), classification (7.1), order and area (7.3), the deformation dimension of compact hyperbolic signatures (10.1), the equivalence rules 1 to 6 (4.1), and a canonical form, all in `orbifold-notation.md`.

**Design documents.** `docs/design/signature.md` specifies the crate `escher-signature`, and `docs/design/cli.md` specifies the commands, options, output format, exit status, and diagnostics of `escher-cli`. The first tests also carry the Miri policy of `DEC-5`, and Section 6 of `docs/design/architecture.md` lists the documents that change with it.

**Sources.** Every item of the scope is primary in `orbifold-notation.md`, from [CH], [TH], [FG], [TG], [CBG], and [BH], except the parts that `UNV-20` and `UNV-21` name.

**Unverified statements relied on.**

- `UNV-20`: the canonical string, the delimiter for orders of 10 or more, the spelling of infinity, and the treatment of a `1` on its own. No primary source defines them (`orbifold-notation.md` 3.4, 3.5, 5, and 12 items 1 to 4). The omission of an order of 1, the order of features, and the complete list of equivalences are primary (`orbifold-notation.md` 3.2, 3.5, and 4.1). The code of Tegula is the only precedent for a canonical string (`orbifold-notation.md` 5).
- `UNV-21`: the geometry of non-compact signatures with $\chi \ne 0$ (`orbifold-notation.md` 12 item 15). The deformation dimension is printed only for compact hyperbolic signatures.

**Tests and acceptance.**

- Tables: the 17 signatures of `orbifold-notation.md` 8.1 have $\chi = 0$. The 14 types of 8.2 give the listed orders for $n$ from 1 to 12. The 7 friezes of 8.3, the examples of 6.4, and the four bad families of 7.2 give their stated results.
- Enumeration: a bounded enumeration of canonical compact signatures finds exactly 17 with $\chi = 0$ (Section 32 of [FG], `orbifold-notation.md` 7.1).
- Properties: a random sequence of the equivalence rules leaves the canonical form and $\chi$ unchanged. The canonical form is idempotent. Display followed by parsing round-trips.
- Snapshots: `insta` snapshots of the info text and of every error message.
- Failure paths with their error kinds (TEST-6).

**Dependencies and `deny.toml` work.** `thiserror`, `clap`, `anyhow`, and the dev-dependencies `insta` and `proptest`. This change adds the first `[workspace.dependencies]` table.

- A test resolved these dependencies on 2026-11-16. It found no crate in two versions, and `cargo deny check licenses bans` passed with the licences `Apache-2.0`, `MIT`, which `strsim` brings through `clap`, `Unicode-3.0`, which `unicode-ident` brings, and `GPL-3.0-or-later`. Section 9.2 of `docs/design/cli.md` and Section 13 of `docs/design/signature.md` record the versions and the checks.
- `deny.toml` brings dev-dependencies into the licence check and the duplicate check (Section 8 of the tooling design), so `just check-deny` checks `insta`, `proptest`, and their dependencies as it checks the normal dependencies.
- A licence enters the allow-list only after its compatibility with GPL-3.0-or-later is confirmed (TOOL-24). Section 8 of the tooling design records the confirmation for `Apache-2.0`, `MIT`, and `Unicode-3.0`.
- If the change finds a duplicate, it adds `skip` entries with reasons only if no alignment of versions removes the duplicate (TOOL-26).
- The change updates the printed `deny.toml` block and the sentences of Section 8 of the tooling design.

**Risks.**

- Rule 4 with a crosscap, and rule 5 (`ox` equals `xxx`, stated by [TG] and [CBG] and not by [CH]), make the canonical form subtle.
- Degenerate inputs: `1`, `*`, `x`, and the empty string.
- The dependency tree may need exceptions for duplicate versions, which `check-deny` reports for dev-dependencies as well as for normal ones.

**Split.** The canonical form can follow as a second branch.

### 5.2. M2: Linear Geometry and the Hyperbolic Reflection Triangle

**Goal.** The linear hyperbolic model and the first domain exist, checked against reference arithmetic.

**Command line.** These commands illustrate the milestone:

```sh
escher info '*732'
escher info --check '*732'
```

- `escher info '*732'` adds a domain summary: three mirror sides, the corner angles in degrees, the side lengths, and the area of the domain, equal to $-2\pi\chi$ (`orbifold-notation.md` 7.3). A valid signature without a construction in this milestone gets one line that says it has no domain, and the exit status stays 0.
- `escher info --check '*732'` also prints one line for each of the six checks of the verification and a line with the overall result. It exits with status 1 when a check fails or the signature has no domain.

Sections 5 and 6.7 of `docs/design/cli.md` define the exact lines and diagnostics.

**Scope.** The new crate `escher-geometry`, with the `glam` types of `DEC-4` and the free functions of the conventions over them (Section 5.1 of `docs/design/architecture.md`), the hyperbolic module, and the disk map in both directions. The new crate `escher-domain`, with the compact `*pqr` for $1/p + 1/q + 1/r < 1$.

- Construction: the side lengths from the angles, the vertices placed with the boost and rotation of [KC24] and then moved so that the centre of `DEC-12` is at the origin, and each wall normal as the unit vector orthogonal to its two vertices.
- `verify()` runs the six checks of Section 8.2 of `docs/design/domain.md`. `build` returns an error for a corner order above the limit of its Section 4.

**Design documents.** `docs/design/geometry.md` for `escher-geometry`, `docs/design/domain.md` for `escher-domain`, and an extension of `docs/design/cli.md` for the domain summary and the option `--check`.

**Sources.**

- Reflection: equation (6.6) of [DAVIS] (`geometry-models.md` 7.4).
- Isometries and distance: Theorem 10.1 and the lemma on p. 83 of [CFKP] (`geometry-models.md` 3.1 and 6.1). Boost and rotation: p. 3 of [KC24] (`geometry-models.md` 6.1).
- Side lengths from the angles: the law of cosines solved for a side, which `geometry-models.md` 10.2 cites to primary sources.
- The disk map in both directions: `geometry-models.md` 4.2.
- Existence and uniqueness of the triangle: Fact 3 of [CFKP] (`geometry-models.md` 10.1) and Example 6.5.3 of [DAVIS] (`fundamental-domains.md` 9.1).
- Group and strict fundamental domain: Theorem 6.4.3 of [DAVIS] and Section 4.4 of [MASKIT2001] (`fundamental-domains.md` 4.4 and 4.5).
- The class is fully sourced (`fundamental-domains.md` 3 and 13.1).

**Unverified statements relied on.**

- `UNV-26`: the normal through two points, and the relation between the inner product of two unit normals and the corner angle, which `geometry-models.md` 7.4 states only in the conventions of [RATCLIFFE], with inward normals. The design does not rely on such a relation, and Section 14 of `docs/design/domain.md` states which checks cover the result.
- `UNV-28` applies if SymmHub's disk triangle (`geometry-models.md` 7.6) serves as a test oracle, because its properties are candidate statements (`geometry-models.md` 15.1 item 5).

**Tests and acceptance.**

- Reference arithmetic: an independent high-precision implementation in the test suite recomputes lengths, vertices, and normals for triples that include (7,3,2), (5,4,2), (4,4,4), (3,3,4), (12,3,2), and (1000,3,2).
- Numeric checks for $\{7, 3\}$, through the split of the polygon into $2p$ right triangles (`geometry-models.md` 10.1): the expected edge length and circumradius in the disk are equations 29 and 24 of [HT] (`geometry-models.md` 10.4), evaluated with the reference arithmetic of `DEC-3`. No value that Table 2 of [HT] prints serves as an expected value, because the table and equation 29 disagree on the edge length of $\{7, 3\}$ (`geometry-models.md` 15.1 item 23).
- Properties: for random admissible triples, the relators hold, the inverse equals $J M^{t} J$, and the disk map round-trips.

**Dependencies and `deny.toml` work.** `glam` (`DEC-4`) and `thiserror` for `escher-geometry` and `escher-domain`, `escher-domain` for `escher-cli`, and the reference-arithmetic dev-dependency `astro-float-num` of `DEC-3`. Each is declared once in `[workspace.dependencies]` (DEP-2).

- `astro-float-num` is the number crate of the astro-float project. Section 11 of `docs/design/geometry.md` states why the design uses it in place of `astro-float`.
- Section 11 of `docs/design/geometry.md` records the resolution of `glam` and `astro-float-num` with their versions, features, licences, and transitive dependencies, and the run of `glam` arithmetic under Miri. The allow-list of the tooling design already covers their licences.
- `glam` is a public dependency of `escher-geometry` and `escher-domain` under rule DEP-3, with the obligations that Section 3.5 of `docs/design/architecture.md` states.
- `astro-float-num` needs no native build (`rust-ecosystem.md` 12).
- The change updates Section 8 of the tooling design for every change to `deny.toml`.

**Risks.** Slips in the conventions, precision at very large orders, and the maturity of the reference crate (`UNV-25`). The run of `glam` under Miri is recorded in Section 11 of `docs/design/geometry.md`, so the remaining risk is that a test of the crates themselves fails under `just check-miri`.

### 5.3. M3: First Image

**Goal.** The vertical slice is complete: fold, flat colours, pixel borders, supersampling, and PNG output. The main rendering risks, which are termination of the fold, precision at the limit circle, and speed, are measured.

**Command line.** This command writes a 512 by 512 image in the Poincare disk with black borders:

```sh
escher render '*732' -o tiling.png
```

`render` takes these options:

- `--size 1024x768`: the image size.
- `--colouring one|parity`: one colour, or the two-colour parity colouring.
- `--colours '#e9c46a,#2a9d8f'`: the fill colours, each as `#rgb` or `#rrggbb` (`DEC-29`).
- `--border-width 1.5`: the border width in pixels.
- `--border-colour`: the border colour.
- `--samples 4`: the sample grid, here 4 by 4.
- `--max-folds`: the fold cap, 1500 by default (`DEC-7`).
- `--background`: the background colour, which is opaque (`DEC-7`).
- `--threads`: the number of threads (`DEC-21`).
- `--stats`: prints the sample count, the mean and maximum number of folds, the number of samples that did not converge, which includes those that reached the cap, and the time.

The command replaces an existing output file (`DEC-30`), so it can run again with the same path. Sections 4.5 to 4.7 and 6.8 of `docs/design/cli.md` define the exact options, output, and diagnostics.

**Scope.** The new crate `escher-render`. The binary gains `render`.

- View: the canonical view shows the centre of the domain at the centre of the disk. The domain is built with its centre at the origin of the model (Section 6.3 of `docs/design/domain.md`), so the view of the disk does not move it, and the view keeps the placement for the other models.
- Fold: the first violated side (`rendering.md` 4.1), with a tolerance and a cap, and no renormalization of the point (`DEC-31`, Section 9.3 of `docs/design/architecture.md`).
- Colour: parity from the orientation of the pairings, computed in `escher-render`, so `M3` needs no `escher-group`.
- Colour input: hexadecimal colours read by a parser in `escher-cli`, with no dependency (`DEC-29`).
- Border: the gradient form of the distance estimate in screen space (`DEC-28`, `UNV-15`), with the screen tangent vectors carried through the fold. It is tested against the distance form and against the tile oracle.
- Sampling: a regular n by n grid averaged in linear light (precedent in `rendering.md` 7.3).
- Samples outside the disk take the background. Samples that reach the cap follow `DEC-7`.

**Design documents.** `docs/design/render.md`, an extended `cli.md`, and the changes to `docs/design/architecture.md` and `docs/design/geometry.md` that the design makes, all in the design branch. The documents of golden images, as Section 7.2 of `docs/design/architecture.md` defines them, change in the code branch that adds the first golden image, which is the branch of `escher-render`, and not in the design branch: Sections 3.2 and 14.2 of the tooling design, for the recipe and its environment variable, and Section 11 of the coding standard, for the new test rule. Section 3.2 of the tooling design lists only recipes that exist (TOOL-6).

**Sources.**

- The per-pixel method: `rendering.md` 3, 4.1, and 4.2.
- Strict fundamental domain: Theorem 6.4.3 of [DAVIS].
- Published caps and behaviour at the cap: `rendering.md` 4.4 and 4.9.
- The disk metric: [CFKP] (`rendering.md` 6.2).
- Border definitions in published code: `rendering.md` 6.1.
- Independence of pixels: `rendering.md` 11.

**Unverified statements relied on.** `UNV-1` (termination and step count), `UNV-3` (points on walls), `UNV-4` (growth of the fold count, and precision), `UNV-15` (distance estimate), and `UNV-18` (parity). The tests also rely on `UNV-14`, because the distance form against which they test the border estimate uses the hyperboloid form of the distance to a geodesic, and on `UNV-25`, because they compare with the reference-arithmetic crate. Averaging in linear light is a design choice that no research source supports.

**Tests and acceptance.**

- Fold properties: the result lies inside the domain, and the fold is idempotent. A point and its image under a pairing fold to the same point within the measured bound, with step counts that differ by exactly one and different parities.
- Fold results for points up to the disk radius $1 - 10^{-6}$ against reference arithmetic, including words of a few hundred reflections. `symmhub.md` 5.4 reports a trace loss of $5 \times 10^{-5}$ on long words in a hyperboloid factorization.
- Border oracle: the border mask equals, within one pixel, the mask from a change of tile identity between neighbouring samples.
- A symmetric view, available only to tests, gives an image with one fill colour that equals its mirror image within the allowance of the golden images. With the parity colouring the mirror image swaps the two fill colours, as Section 11 of `docs/design/render.md` records.
- Golden images of `*732`, `*542`, and `*433` at a small size.
- The PNG file decodes to the requested size.
- One process-level smoke test of the binary.

The performance expectation of Section 8 of `docs/design/architecture.md` is confirmed here (`DEC-27`).

**Dependencies and `deny.toml` work.** `png` for `escher-render`, and `rayon` for `escher-cli` (`DEC-21`). `escher-render` also declares `glam` from the workspace entry of M2, which adds no licence. `M3` adds no colour crate and no `escher-group`.

- On a prototype outside the repository, the newest resolution of `png` brought two versions of `miniz_oxide`, as `rust-ecosystem.md` 19 predicted, and failed the ban on duplicate versions. `flate2` is therefore held at 1.1.9 in `Cargo.lock`, so that one version of `miniz_oxide` resolves. The lock file is committed, so the pin holds until an update. If an update brings the duplicate back, the change that updates decides between the pin and a `skip` with a reason (TOOL-26). Section 9 of `docs/design/render.md` records the measurement.
- No colour crate is a dependency (`DEC-29`), and at M8 the owner kept it so (`DEC-39`). On the prototype, `palette` failed the ban on duplicate versions, and `csscolorparser` failed it with its default features. Section 9.2 of `docs/design/cli.md` records the resolution of `rayon` and of the two colour crates.
- New licences are added only as cargo-deny reports them.

**Risks.** Speed. The appearance of the rim. Golden images that differ between the container and CI because of the platform mathematics library. The estimate near vertices. `rayon` fails under Miri, so Miri checks no test that reaches the thread pool (`DEC-21`). A plain `cargo update` brings the duplicate `miniz_oxide` back.

**Split.** Three branches, in this order:

1. The design branch `feature/first-image-design`, with the documents of the field "Design documents" and every bound that a test sets stated as to be measured.
2. The branch of the crate `escher-render`, with its tests, its golden images, the documents of golden images, the `deny.toml` work of `png`, and the measured figures written into `docs/design/render.md`. If it grows, it splits into (a) one sample per pixel and (b) supersampling and statistics.
3. The branch of the command `render` in `escher-cli`, with `rayon`, the snapshots, and the smoke test.

### 5.4. M4: Euclidean Reflection Groups and the First Shape Parameter

**Goal.** `*333`, `*442`, `*632`, and `*2222`, and the parameter machinery from end to end.

**Command line.** These commands illustrate the milestone:

```sh
escher render '*442' -o e.png
escher info '*2222'
escher render '*2222' --set aspect=1.618 -o r.png
```

- `escher info '*2222'` lists one parameter with its name, range, and default, and `info` prints a domain summary, and with `--check` the checks, for the four signatures. `info` takes no setting, so it describes the default shape.
- `render --set` takes a `name=value` setting, once for each parameter. `info` has no `--set`.

Sections 4.5, 4.6, 5, and 6.8 of `docs/design/cli.md` define the exact option, lines, and diagnostics.

**Scope.** A Euclidean module in `escher-geometry` over the affine plane, with line covectors, the side test, the wall through two points, reflection, rotation, translation, distance, and an isometry check. It has no inverse, no bilinear form, and no point normalization, because no caller of this milestone needs them, so the special inverses that `geometry-models.md` 13.6 records do not arise. The Euclidean triangle and rectangle in `escher-domain`, with `ParameterSpec`, `ShapeParameters`, `parameters`, and a second constructor that takes the settings. A canonical Euclidean view in `escher-render` (`DEC-12`), through the unchanged fold and border estimate. `--set` on `render` and the `Parameter` line of `info` in `escher-cli`.

**Design documents.** Extensions of `geometry.md`, `domain.md`, `render.md`, and `cli.md`, and the changes to `docs/design/architecture.md` and to this plan that the design makes.

**Sources.** Section 6.3, Theorem 6.4.3, and Example 6.5.2 of [DAVIS] (`fundamental-domains.md` 9.1). The rectangle: Example 13.1.3 of [TH] (`orbifold-notation.md` 9.4). The rectangle family: pp. 7 and 14 of [VONGAGERN2009] (`fundamental-domains.md` 9.5).

**Unverified statements relied on.** `UNV-10`, because no source states whether the one parameter of `*2222` includes the scale (gap G7). `UNV-26` for the Euclidean reflection as a matrix. `UNV-28` for the vertex coordinates of the Euclidean triangle, which the construction uses. `UNV-1`, `UNV-15`, and `UNV-18` for the fold, the border estimate, and the parity colouring in the Euclidean plane.

**Tests and acceptance.**

- Relators, and the checks of `verify()` for the four signatures.
- Fold properties on the Euclidean domains, and the fold against reference arithmetic.
- Border oracle, and the oracle of the grid of the rectangle.
- Periodicity: the image shifted by a translation of the group that is a whole number of pixels equals itself. The test runs in the canonical view for the three triangles and for `*2222` with `aspect` 1 and 4, and in a view available only to tests for `*2222` with `aspect` 1.618. It asserts equality byte for byte only for `*2222` with `aspect` 1 and 4, whose values are all exact in `f64`, and the golden allowance otherwise (Section 8.1 of `docs/design/render.md`).
- Golden images of `*632`, `*442`, `*333`, and `*2222` with `aspect` 1.618.
- A snapshot of the parameter listing. Range errors and unknown names.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Small. The unchanged renderer draws a Euclidean domain through the disk map without an error, as a prototype showed, so the split adds a guard to `escher render` (item 2 of the split). `*2222` with `aspect` 1 or 4 at one sample per pixel shows no border, because every pixel centre lies exactly half a pixel from a wall (Section 5.8 of `docs/design/render.md`).

**Split.** Three branches, in this order:

1. The design branch `feature/euclidean-design`, with the documents of the field "Design documents" and every bound that a test sets stated as to be measured.
2. The branch of the module `euclidean` of `escher-geometry` and the Euclidean domains and shape parameters of `escher-domain`, with their tests and measured figures. It also updates the snapshots of `escher info` that change, and adds a guard to step 3 of `render` (Section 4.5 of `docs/design/cli.md`), in the module `render` of `escher-cli`. The guard refuses a domain whose geometry is not hyperbolic with the diagnostic of an unsupported signature, because otherwise the unchanged renderer draws a Euclidean domain through the disk map without an error.
3. The branch of the Euclidean view of `escher-render`, with its tests, golden images, and measured figures, together with `--set` and the `Parameter` line in `escher-cli`. It narrows the guard to the rule of CLI-30 of `docs/design/cli.md`, which refuses only a domain that is neither hyperbolic nor Euclidean.

### 5.5. M5: The Sphere

**Goal.** The spherical reflection groups `*332`, `*432`, `*532`, `*n22`, and `*nn`, drawn as a lit ball in orthographic projection with Phong shading, one light, and a viewing rotation that the command line sets, as the owner decided.

**Command line.** These commands illustrate the milestone:

```sh
escher render '*532' -o ico.png
escher render '*532' --rotation 20,35,0 -o ico-rotated.png
```

`render` gains these options:

- `--rotation 20,35,0`: the viewing rotation.
- `--light`: the light direction.
- `--shading phong|none`: Phong shading, or none.

`escher info` prints the domain summary, and with `--check` the checks, of each signature of the milestone, and gains no line.

Sections 4.2, 4.6, 5.3, and 6.8 of `docs/design/cli.md` define the exact options, lines, and diagnostics.

**Scope.** A module `spherical` in `escher-geometry` with the unit sphere, the side test, point normalization, distance, the wall through two points, reflections, rotations, and an isometry check. In `escher-domain`, the spherical reflection triangles `*332`, `*432`, `*532`, and `*n22` from the spherical law of cosines, and the lune of `*nn`, which includes `*22` with $n = 2$, each with $n$ up to the existing limit of 1000. No spherical signature has a shape parameter. In `escher-render`, a point size of the sphere in the fold, the ball view with the viewing rotation, and Phong shading with one light (Sections 5.3, 5.9, and 5.10 of `docs/design/render.md`). The fold cap stays the only bound of the fold: no bound from the group order enters the code, and the tests compare the largest fold count of an image with the group order. In `escher-cli`, the options `--rotation`, `--light`, and `--shading`, and the removal of the rule that refuses a domain whose geometry has no view.

**Design documents.** Extensions of `geometry.md`, `domain.md`, `render.md`, and `cli.md`, and the changes to `docs/design/architecture.md`, to this plan, and to `geometry-models.md` and `fundamental-domains.md` that the design makes.

**Sources.**

- The sphere as a linear model: [KC24] (`geometry-models.md` 11.1). Reflection: Section 6.4 of [DAVIS].
- Triangles: Example 6.5.1 of [DAVIS], and Theorem 6.4.3 of [DAVIS] for the triangle as a strict fundamental domain. The sector of two mirrors: p. 82 of [DAVIS] (`fundamental-domains.md` 4.5). The spherical law of cosines solved for a side: Theorem 2.5.4 of [RATCLIFFE] (`geometry-models.md` 11.3). The definition of a convex polytope in the sphere: p. 75 of [DAVIS] (`fundamental-domains.md` 13.3 item 9).
- Orders: `orbifold-notation.md` 8.2. Finite folds: `rendering.md` 9.3. Phong shading: `geometry-models.md` 11.6.

**Unverified statements relied on.** `UNV-16` (the lift, the view scale, and the Phong formulas, whose primary paper is unread), `UNV-26` (the angle of a lune measured from its outward normals), `UNV-29` (the lune as a strict fundamental domain), and `UNV-1`, `UNV-15`, and `UNV-18` for the fold, the border estimate, and the parity colouring on the sphere. The border tests use a brute-force oracle, so `UNV-14` is not used.

**Tests and acceptance.**

- Exhaustive certificate: for `*332`, `*432`, `*532`, and `*n22` and `*nn` with $n$ from 2 to 12 and $n = 1000$, a breadth-first enumeration of the products of the pairings has exactly $2/\chi$ elements (`orbifold-notation.md` 7.3), the images of the centre are pairwise further apart than a stated floor, and every image of the centre folds back to the centre with as many steps as its depth and never reaches the fold cap.
- Every sample of the canonical image of each tested signature folds within the fold cap, with a largest fold count below the group order. The claim covers the samples of these images and the images of the centre, not every point of the sphere (`UNV-1`).
- With shading off, and also with it on, rotating the view by a group element leaves the image unchanged within the golden allowance, and a reversing element exchanges the parity colours.
- The brightest point follows the light direction.
- The fold, the lift and its tangents, the border estimate by rim band, and the shading against reference arithmetic (Section 6.7 of `docs/design/render.md`).
- Golden images of `*532`, `*432`, `*332`, `*722`, and `*55`, of `*532` with the rotation 20, 35, 0 degrees, and of `*532` without shading. The seven existing golden images stay unchanged.
- Snapshots of the domain summaries, of the three options, and of their diagnostics.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Aliasing at the limb. Borders of pixel width on a view that is not conformal (`DEC-16`): a faint dark outline at the rim, and a ball entirely of border colour at $n = 1000$ (Section 5.9 of `docs/design/render.md`). Near a vertex of very high order, a fold can fail to stop within the cap (Section 6.7 of `docs/design/render.md`). The defaults for the rotation, the light, and the material, which have no source.

**Split.** Three branches, in this order:

1. The design branch `feature/sphere-design`, with the documents of the field "Design documents" and every bound that a test sets stated as to be measured.
2. The branch of the module `spherical` of `escher-geometry` and the spherical domains of `escher-domain`, with their tests and measured figures. It also updates the snapshots of `escher info` that change. The existing rule of `escher render` that refuses a domain whose geometry has no view keeps `render` from drawing a spherical domain wrongly until the next branch.
3. The branch of the point size, the ball view, the viewing rotation, and the shading of `escher-render`, with their tests, golden images, and measured figures, together with the options `--rotation`, `--light`, and `--shading` of `escher render`. It removes the rule that refuses a domain whose geometry has no view.

### 5.6. M6: Rotation Subgroups

**Goal.** The first side pairings that are not reflections: `pqr` in all three geometries (`732`, `333`, `442`, `632`, `332`, `432`, `532`), `22n`, `nn`, and `2222` on the rectangle.

**Command line.** These commands illustrate the milestone:

```sh
escher render 732 -o rot.png
escher render 532 -o i.png
```

`info` shows the paired sides and reports that no parity colouring exists.

**Scope.** The domain is the double of the reflection domain across one wall, with products of two reflections as pairings. The fold runs with general pairings.

**Design documents.** Extensions of `domain.md`, `render.md`, and `cli.md`, and the changes to `docs/design/architecture.md`, to this plan, and to `fundamental-domains.md` that the design makes. `domain.md` states that the wall of side 1 of the reflection domain is doubled, and that a side fixed by a half-turn is split at the fixed point. Item 2 of `fundamental-domains.md` 4.8 permits this split and does not require it, so the split is a choice of this plan. `render.md` states the bounds of the fold of a rotation domain, and `cli.md` the lines of `info` and the default colouring of `render`.

**Sources.**

- Sections 4.4 and 5.9 of [MASKIT2001] (`fundamental-domains.md` 9.1 and 7.6).
- The polygon theorem for orientation-preserving groups: Theorem 12 of [SOLOMYAK2020] and Theorem 6.14 of [SERIES2013] (`fundamental-domains.md` 4.1 and 4.2). For the Euclidean plane and the sphere, the polygon theorem of `fundamental-domains.md` 4.9 and 4.10.
- The kite, and "two copies of the triangle": p. 12 of [VONGAGERN2009] (`fundamental-domains.md` 9.5).
- SymmHub's double triangle (`symmhub.md` 7.2.7).

**Unverified statements relied on.**

- `UNV-2`: the greedy fold with general pairings.
- `UNV-22`: the signature of the rotation subgroup in orbifold notation (gap G10).
- `UNV-30`: the lune double of `nn` as a fundamental domain of the cyclic group.
- `UNV-31`: the convexity and the vertex cycles of the double.
- `UNV-1`, `UNV-15`, and `UNV-18` for the fold, the border estimate, and the parity colouring of the reflection domains that the tests compare with.

**Tests and acceptance.** The items of the checklist of `fundamental-domains.md` 4.8 that apply to a compact polygon, as `verify()`. Cycle relators. Spherical orders. Fold properties, with the fold of each rotation domain against reference arithmetic within bounds of its own (Section 6.8 of `docs/design/render.md`). Every border pixel of the `pqr` image is a border pixel of the `*pqr` image in the same view. Golden images, with every existing golden image unchanged. Snapshots of the `Side pairs` and `Parity colouring` lines of `info` and of the diagnostic of `--colouring parity` for a rotation domain.

**Dependencies and `deny.toml` work.** None new.

**Risks.** The order of the sides and the split at a half-turn. The first exposure to `UNV-2`: the pairings are products of two reflections, and on a prototype outside the repository a fold of `(1000)32` with plain `f64` products drifted off the hyperboloid and did not converge, so the domain design requires accurately rounded products and the rotation domains have bounds of their own (Section 6.8 of `docs/design/render.md`). The vertices of the doubles meet the tolerance constant of the fold with a margin of only 3.27 over all hyperbolic triples on that prototype (Section 6.2 of `docs/design/render.md`).

**Split.** Three branches, in this order:

1. The design branch `feature/rotation-design`, with the documents of the field "Design documents" and every bound that a test sets stated as to be measured.
2. The branch of the rotation domains of `escher-domain`, with their tests and measured figures. It also updates the snapshots of `escher info` that change, replaces `532` with `3*2` as the example of a signature without a domain, and adds the interim rule CLI-36 of `docs/design/cli.md` to `escher render`, which refuses a domain with a side that is not a mirror, because the renderer has no measured accuracy for such a domain until the next branch.
3. The branch of the tests, bounds, and golden images of the rotation domains in `escher-render`, whose fold code does not change but whose API documentation of `fold` states the accuracy for a rotation domain, together with the default colouring and the diagnostic of `--colouring parity` in `escher render`. It withdraws CLI-36.

### 5.7. M7: Presentations and Coset Enumeration

**Goal.** The crate `escher-group` exists, with presentations and Todd-Coxeter coset enumeration, as the base of colourings and as an independent check of domains.

**Command line.** This command prints the generators and relators of a signature:

```sh
escher info --presentation '*532'
```

For a spherical signature it also prints the line `Order by coset enumeration: 120, equal to 2/chi`, and the same for the side presentation of its domain.

**Scope.** `Word`, `Presentation`, the Conway presentation of a signature (`orbifold-notation.md` 9.1), the side presentation of a domain, and bounded Todd-Coxeter enumeration. The side presentation has Coxeter relators for mirrors, and pairing and vertex-cycle relators otherwise, as `symmhub.md` 14.2 describes SymmHub doing.

**Design documents.** `docs/design/group.md`, extensions of `domain.md` and `cli.md`, and the changes to `docs/design/architecture.md` and to this plan that the design makes. `domain.md` states the side presentation, and `cli.md` the option `--presentation` of `info`. Prerequisite RP-1.

**Sources.** [CH] pp. 255 to 257. Theorem 6.4.3 of [DAVIS] and the theorems of M6 for side presentations. The polygon theorems of `fundamental-domains.md` 4.9 and 4.10 for the cycle relators of mixed side pairings, from M10 on. `colourings.md` 7.2 for coset tables in GAP, 7.9 to 7.15 for the coset table, Todd-Coxeter enumeration, and its termination, 7.16 for the limits of existing implementations, and 7.20 for worked examples. `colourings.md` 7.15 records that [HOLT2005] states that a run that reaches its limit shows nothing about the index, and that [SIMS1994] states only that such a run stops without a result. `rust-ecosystem.md` 14, which records that no usable crate exists and that `algebraeon-groups` is under GPL-3.0-only and is not copied.

**Unverified statements relied on.** `UNV-19` (the steps of coset enumeration that only code states, and the results that the books state without a proof), `UNV-22` (a boundary without corners), and `UNV-32` (the relators of a double as words). The side presentations also rest on `UNV-29`, `UNV-30`, and `UNV-31` through the domains of M5 and M6.

**Tests and acceptance.**

- The Conway presentations of all 14 spherical types, for $n$ from 1 to 12, enumerate to $2/\chi$.
- The side presentations of every domain built so far give the same orders.
- `x`, `22`, and `*` each have order 2 ([CH] p. 257, `orbifold-notation.md` 4.2).
- An infinite group reaches the bound and returns the limit error.

**Dependencies and `deny.toml` work.** None.

**Risks.** The enumeration strategy, and blow-up on relator orders that suit it poorly. A limit on the cosets bounds the memory of an enumeration and not its time, so `info` enumerates only a spherical signature whose group order is within the limit (`docs/design/cli.md`). No domain of M5 or M6 has a vertex cycle of three or more sides, so no such domain fixes the letter order of the cycle relators (`docs/design/domain.md`, `UNV-32`).

**Split.** Three branches, in this order:

1. The design branch `feature/group-design`, with `docs/design/group.md`, the extensions of `domain.md` and `cli.md`, the changes to `docs/design/architecture.md` and this plan, and the lines of `AGENTS.md` that the documents make false. Every figure of the crates is stated as to be measured.
2. The branch of the crate `escher-group`, with `Word`, `Relator`, `Presentation`, `conway_presentation`, `CosetTable`, `enumerate`, and all their tests, with the workspace manifest, `README.md`, `AGENTS.md`, the member list of the coding standard, and the Miri list of the tooling design. The measured limits and times go into `docs/design/group.md`.
3. The branch of `side_presentation` in `escher-domain`, with its tests and its measured residuals, and of `info --presentation` in `escher-cli`, with its snapshots. It declares `escher-group` in both manifests, because an unused dependency fails `check-unused-deps`, and it updates Project Status in `AGENTS.md` for the completion of the milestone.

### 5.8. M8: Colouring Search With Ids

**Goal.** The low-index subgroup search, colouring ids, the listing in `info`, and coset colours in `render`.

**Command line.** These commands illustrate the milestone. The option `--colourings` lists every colouring. CLI-45 and CLI-46 of `docs/design/cli.md` state it and `--max-colours`:

```sh
escher info '*642'
escher info --max-colours 20 '*642'
escher info --colourings '*642'
escher render '*642' --colouring 3a -o c.png
escher render '*642' --colouring 3a --colours '#dc143c,#ffd700,#000080' -o c.png
```

- `escher info '*642'` prints the number of colourings of each colour count up to 12 (`DEC-38`).
- `--max-colours 20` raises the bound to 20 colours.
- `--colourings` prints every colouring up to the bound, with its id and colour count.
- `render --colouring 3a` draws the colouring with the id `3a` in the first 3 colours of the built-in palette of 12 colours (`DEC-39`).
- `--colours` gives the colours instead, in hexadecimal only (`DEC-29`), exactly as many as the colouring has. A colouring with more than 12 colours needs it.

**Scope.** In `escher-group`: the low-index search over partial coset tables, with relator checks, the first-in-class test, and standard output, which finds one table for each conjugacy class and expands it into one colouring for each subgroup (`DEC-37`). A budget in steps and limits on what a call stores, with a result complete to a colour count when a limit stops it. Counts without stored tables, and the colourings of one colour count. Ids. `colour_of`, the colour from the side sequence of the fold. In `escher-render`: a fill by cosets. In `escher-cli`: the colouring lines of `info`, the options of `DEC-38`, `render --colouring <id>`, and the palette of 12 colours. `docs/design/group.md` (Sections 7.10 to 7.17) and `DEC-37` to `DEC-45` of `docs/design/architecture.md` state the design.

**Design documents.** Extensions of `group.md`, `render.md`, `cli.md`, and `domain.md`, and the changes to `docs/design/architecture.md` and to this plan that the design makes.

**Sources.** Coset colourings and Lemma 2.1 (`colourings.md` 4.1). The algorithm and its cost (`colourings.md` 7.1 to 7.3, 7.7, and 7.17), its standard form and order (`colourings.md` 7.14), and its worked examples (`colourings.md` 7.20). SymmHub's per-pixel colour and assembled algorithm (`colourings.md` 8, `symmhub.md` 12). SymmHub's `sublib` as the implementation to study (`colourings.md` 7.19, `symmhub.md` 12.8).

**Unverified statements relied on.** `UNV-17`: Lemma 2.1 is proved only for the triangle tiling of $G_{p,q}$ (`colourings.md` 4.2), and the rule that turns the sides of a fold into a coset is stated by no source (GRP-38 of `docs/design/group.md`, `colourings.md` 8.4). Whether the listing is complete, and how conjugate subgroups relate to colourings, is open (`colourings.md` 3.4 and 14.1 items 1 to 3). `UNV-19`: the pruning of partial tables, the bijection that the expansion of a class rests on, and the code-only steps of the search. `UNV-32`: the side presentations that the search runs on.

**Tests and acceptance.** Section 10.1 of `docs/design/group.md` names each test after the item that it serves:

- Frettloh: Tables 1 and 2 of Frettloh (`colourings.md` 5.6 and 5.7), reproduced exactly for `*732`, `*832`, and `*542` and their rotation signatures, up to 50, 39, and 26, and 24, 19, and 17 colours, by counting the classes that have a coset fixed by the generators of Lemma 3.2 or Lemma 4.1. Table 2 counts a class and its mirror once.
- Cube: 1, 3, and 6 colours, each unique (`colourings.md` 5.8).
- Two colours: seven subgroups of index 2 for `*642`, `*842`, and `*664` ([SINGERMAN2014], `colourings.md` 6).
- Bulatov: each count in Bulatov's captions is found (`colourings.md` 4.4, existence only).
- Printed searches: the low-index examples of [SIMS1994], [HOLT2005], SymPy, and [GAP] (`colourings.md` 7.20), with their printed tables and counts.
- Perfectness: on rendered samples of rotation domains, each side permutes the colours by the inverse column of its generator, exactly ([FRETTLOH2008] p. 2). This is the test "Exact permutation" of Section 8.1 of `docs/design/render.md`.
- Ids: identical across runs, across thread counts, and for every limit and budget that completes their colour count.
- `--colouring 1a` gives the bytes of `one`, and the id of the parity table the bytes of `parity`.

**Dependencies and `deny.toml` work.** `escher-render` declares `escher-group`, an internal dependency. No external dependency and no change to `deny.toml` (`DEC-39`).

**Risks.** The number of subgroups grows fast with the number of generators and the index, and the listing of every subgroup multiplies the classes by their sizes. `colourings.md` 7.3 records an unverified count of 653 results for the free group of rank 3 at index 4 (`colourings.md` 14.1 item 27), and `colourings.md` 7.1 gives exponential time. On a prototype, the triangle groups with three even corner orders had thousands of classes by 12 colours, such as 4849 classes of 12 colours for `*664`, and the triangles with corner orders near 1000 stopped at the default budget before 12 colours (Section 11.3 of `docs/design/group.md`). The fill by cosets added 40 to 60 % to the time of an image on the prototype, and Section 5.12 of `docs/design/render.md` gives its smaller cost in the crate. The ids depend on the numbering of the sides, so a renumbering in `escher-domain` renames them without an error.

**Split.** Five branches, in this order:

1. The design branch `feature/colouring-design`, with the documents of the field "Design documents" and every figure of the crates stated as to be measured.
2. The branch of the search in `escher-group`, with its tests and measured figures in `docs/design/group.md`.
3. The branch of the colouring lines of `info`, and the branch of the fill by cosets in `escher-render`, in either order or in parallel, each after branch 2.
4. The branch of `render --colouring <id>`, the palette, the snapshots, `README.md`, and Project Status in `AGENTS.md` for the completion of the milestone.

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
- Coordinates: one polygon for each set of angles, from the proof of Theorem 7.16.2 of [BEARDON], and the rest of the family only from SymmHub (`fundamental-domains.md` 3 and 9.4).

**Unverified statements relied on.** `UNV-6` and `UNV-7` for producing the polygon, `UNV-27` (ranges and default), and `UNV-12` for the pentagon relations that `geometry-models.md` 10.5 and 15.1 items 12 and 14 leave open, if they are used. The reliance is limited, because `verify()` checks the hypotheses of Theorem 6.4.3 of [DAVIS] on each built polygon: closure, convexity, and corner angles $\pi/p_i$.

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

**Sources.** The signatures: Table II of [CH] and Theorem 13.3.6 of [TH]. The kites of `3*3` and `4*2`, and the relations of index 2: pp. 7 and 12 of [VONGAGERN2009]. The catalogue and its recorded defects: `fundamental-domains.md` 9.5 and 8.14, `symmhub.md` 7.1 and 15.3. Presentations: [CH]. The polygon theorems for the Euclidean plane, whose hypotheses `verify()` checks: `fundamental-domains.md` 4.7, 4.9, and 4.10. Neither [RATCLIFFE] nor Epstein and Petronio works an example that combines a cross-cap with a cone point or a mirror, as `22x` and `*x` do (`fundamental-domains.md` 4.7).

**Unverified statements relied on.** `UNV-10` (parameter counts, from the partial counts of [VONGAGERN2009] and [CBG], the unsourced table of `orbifold-notation.md` 12 item 7, and SymmHub's catalogue), `UNV-11`, `UNV-2` for translations and glides in the fold, `UNV-8` for glide pairings, and `UNV-9` for skew and twist.

**Tests and acceptance.** A table-driven test over the 17 signatures of `orbifold-notation.md` 8.1. `verify()`. Relators. Equal counts of subgroup classes up to a small index between the side presentation and the Conway presentation. Periodicity of the image. The borders of `3*3` lie inside the borders of `*632`, and those of `4*2` inside those of `*442`. Golden images.

**Dependencies and `deny.toml` work.** None new.

**Risks.** The catalogue has known wrong entries (`2222`, `2*22`, `22*`), so each construction is designed and verified here and not ported blindly.

**Split.** By family: translations and glides, then the groups with half-turns, then the kites.

### 5.11. M11: The 14 Spherical Types

**Goal.** The remaining `n*`, `nx`, `2*n`, and `3*2`, and the small cases `1`, `*`, `x`, `22`, and `2*`. This milestone completes stage 2. `*22` is the lune of `*nn` with $n = 2$, which M5 builds.

**Command line.** These commands illustrate the milestone:

```sh
escher render '3*2' -o th.png
escher render '5x' -o s10.png
```

`escher info` reports every type.

**Scope.** Constructions on the unit sphere with rotations, reflections, and rotary reflections. Domains that are hemispheres, and lunes other than the lune of `*nn` of M5. Prerequisite RP-2.

**Design documents.** An extension of `domain.md`.

**Sources.** The signatures and orders: Table III of [CH] and Theorem 13.3.6 of [TH]. The reflection triangles are sourced, and the other generators exist only as catalogue code (`fundamental-domains.md` 9.6 and 13.1). The polygon theorems for the sphere: `fundamental-domains.md` 4.9 and 4.10. Neither [RATCLIFFE] nor Epstein and Petronio works an example that combines a cross-cap with a cone point or a mirror, as `nx` does (`fundamental-domains.md` 4.7).

**Unverified statements relied on.** `UNV-11`, the coincidences at small $n$ (`orbifold-notation.md` 12 item 8), and `UNV-20` for the trivial group written as `1`.

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

**Unverified statements relied on.** `UNV-13`. The inverse of the band map is stated only in SymmHub's code (`rendering.md` 13.1 item 14), the band metric rests only on a summary, and the correspondence of the half-plane is a project choice (`geometry-models.md` 4.3).

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

**Unverified statements relied on.** `UNV-14`. The primary form on the hyperboloid is time-first and is not stated for unit normals (`geometry-models.md` 9.3 and 15.1 item 3). The form in the disk for a geodesic that is not a diameter, and the form on the sphere, are candidate statements (`rendering.md` 13.1 items 8 and 10).

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

**Sources.** The classes are code only (`fundamental-domains.md` 3 and 13.1). The sourced parts are the deformation dimension (Corollary 13.3.7 of [TH]), the orientable generators of [MASKIT2001] (`fundamental-domains.md` 7) as a cross-check for signatures without mirrors, and the presentations of [CH]. The polygon theorems for mixed side pairings, whose hypotheses `verify()` checks: `fundamental-domains.md` 4.7, 4.9, and 4.10.

**Unverified statements relied on.** `UNV-6`, `UNV-7`, `UNV-9`, `UNV-2`, `UNV-11` for the mixed signatures, `UNV-26` for the conversion of circles into normals, `UNV-24` for the MIT code, and the defects of `fundamental-domains.md` 8.14 and `symmhub.md` 15.3.

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

**Sources.** Code only. [NORBURY2006], a secondary source, on one-sided geodesics (`fundamental-domains.md` 6.3 and 6.5). Maskit's closers of handles for the orientable cases (`fundamental-domains.md` 7.8 and 7.10). The polygon theorems for orientation-reversing side pairings, whose hypotheses `verify()` checks: `fundamental-domains.md` 4.7, 4.9, and 4.10. Neither [RATCLIFFE] nor Epstein and Petronio works an example that combines a cross-cap with a cone point or a mirror, as `23x` and `2*x` do (`fundamental-domains.md` 4.7).

**Unverified statements relied on.** `UNV-2`, `UNV-6`, `UNV-7`, `UNV-8`, `UNV-9`, and gap G15 (several boundaries).

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

The general non-compact signatures are outside the proof of concept and are a workstream of `docs/design/roadmap.md` (`DEC-19` of `docs/design/architecture.md`). The research records no implementation for them, because SymmHub's orbifold parser has no infinite order (`symmhub.md` 4.2). The sourced material is Maskit's cusps for orientable signatures (`fundamental-domains.md` 7), and the worked examples and the polygon for orientation-preserving groups of `fundamental-domains.md` 9.8, which records that no source read constructs a polygon for a signature that mixes cusps with mirrors or crosscaps.

**Design documents.** Extensions of `domain.md` and `render.md`. Prerequisite RP-4.

**Sources.** The sources named in part 2, the polygon theorems for polygons with ideal vertices (`fundamental-domains.md` 4.7), the frieze list of `orbifold-notation.md` 8.3, and SymmHub's `Group_Frieze` as RP-4 records it.

**Unverified statements relied on.** `UNV-23` for the fold near an ideal vertex (Section 9.5 of `docs/design/architecture.md`), `UNV-21` for the geometry of the triangle signatures with an infinite order, and `UNV-26` for the conversions into the linear model. `UNV-5` for the ideal corner of a reflection triangle such as `*23(inf)`, where two mirrors end at a cusp: which test `verify()` runs there is open, and so is the reading that the two sides have the dihedral angle 0 and give no relator, and so is the element that fixes an ideal vertex (`fundamental-domains.md` 13.3 items 7 and 8, gap G1).

**Tests and acceptance.** The parabolic cycle condition in `verify()` for closed cycles at an ideal point, as in the rotation subgroups (item 5 of `fundamental-domains.md` 4.8). At an ideal corner between two mirrors, `verify()` runs the test that the design document of `escher-domain` chooses and marks as `UNV-5`. Samples near an ideal point reach the cap and take the colour that `DEC-7` of `docs/design/architecture.md` gives to samples at the cap. Golden images. A test that `escher render` rejects a general non-compact signature as Section 5.8 of `docs/design/architecture.md` states, with an `insta` snapshot of the diagnostic.

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
| UNV-1 | The first-violated-wall fold terminates for compact reflection polygons, in a number of steps equal to the word length. On the sphere, a prototype found points within $10^{-14}$ radians of a vertex of order 1000 whose fold did not stop even at a cap of 100000, and points that took more steps than the longest word of the group (Section 6.7 of `docs/design/render.md`), so the design relies on it only for the samples of the images that the tests render and the images of the centre | `rendering.md` 4.7 and 13.1 item 1, gap G5, `fundamental-domains.md` 13.3 item 6 | M3 on | Sections 1.6, 1.7, and 5.4 of [HUMPHREYS], or Lean. The sources that do not state it are listed in `rendering.md` 4.7 and gap G5 |
| UNV-2 | The same fold terminates for general side pairings and domains that are not convex. For the rotation domains of M6, a prototype outside the repository found that a fold with pairings that are plain `f64` products of two reflections can drift off the hyperboloid and fail to converge, as one fold of `(1000)32` did in the radius band $1 - 10^{-6}$. With accurately rounded products as pairings, every fold of the rotation fold sets of `escher-render` converges, but near some vertices of a rotation domain whose corner orders are all near 1000 the fold can still fail to stop (Section 6.8 of `docs/design/render.md`). So the design relies on it only for the samples of the images that the tests render and the images of the centre, with accurately rounded products as pairings | `rendering.md` 4.7 and 13.1 item 7, `fundamental-domains.md` 10.7 and 13.3 item 3, `symmhub.md` 17 item 5 | M6, M10, M15 to M17 | A source, a change to Voight's rule, or Lean. The sources that do not state it are listed in `rendering.md` 4.7 and gap G5 |
| UNV-3 | Behaviour on or near a wall | `rendering.md` 13.1 item 2 | M3 on | Tests. No source is known |
| UNV-4 | Growth of the fold count, the reading of the exponents, and the limits of precision | `rendering.md` 13.1 items 5 and 6, `geometry-models.md` 15.1 item 10 | M3 defaults | The body of [FWW], and measurement |
| UNV-5 | The open points of the polygon theorem at ideal points: the test where a mirror ends at a cusp, the reading that two sides that meet only at an ideal vertex have the dihedral angle 0 and give no relator, and the element that fixes an ideal vertex. The four cases with reflections, glide cycles, mirror corners beside paired sides, and ideal reflection polygons are sourced in `fundamental-domains.md` 4.7, and the Euclidean plane and the sphere in its Sections 4.9 and 4.10, all under the hypotheses of those sections | `fundamental-domains.md` 4.7, 4.9, 4.10, 12, and 13.3 items 7 and 8, gap G1 | M18 | A source or Lean for each open point of gap G1 |
| UNV-6 | The cases that SymmHub's `edgeLength` sends to the inverse hyperbolic sine, the correspondence of its other cases with the formulas of `fundamental-domains.md` 9.3, and the pillow and hat formulas. The triangle, quadrilateral, pentagon, and hexagon formulas are sourced in `fundamental-domains.md` 9.3 | `fundamental-domains.md` 8.7, 8.8, 9.3, and 13.3 item 1, gap G3 | M9, M15, M16 | Formula Glossary 2.2 to 2.4 of [BUSER], or Lean |
| UNV-7 | Deaton's cut rules, the twelve atoms, and the twist in three situations | Gap G4, `fundamental-domains.md` 13.3 items 4 and 5 | M9, M15, M16 | [DEATON1993] |
| UNV-8 | The cap generator is a glide reflection that, with the other side pairings, meets the hypotheses of the polygon theorems of `fundamental-domains.md` 4.9 and 4.10 | Gap G2, `fundamental-domains.md` 13.3 item 2 | M10, M16 | [CHOI2012], [BUSER] |
| UNV-9 | The twist convention | Gap G11 | M10, M15, M16 | A project convention |
| UNV-10 | Euclidean parameter counts: whether a count includes the scale, and the counts of the groups for which no source states one. The counts and the rule that are sourced are in `orbifold-notation.md` 10.2 and `fundamental-domains.md` 6.1 | Gap G7, `orbifold-notation.md` 12 item 7 | M4, M10 | [KLR], [SCOTT], or Lean |
| UNV-11 | Constructions of mixed signatures and of the spherical generators from the signature. Their presentations are sourced (`fundamental-domains.md` 9.7) | Gap G8, `fundamental-domains.md` 9.7 | M10, M11, M15 | A source that constructs them, or Lean |
| UNV-12 | Trigonometry that rests on secondary or summarized sources: the Saccheri formulas, four of the six Lambert relations, the pentagon forms of `geometry-models.md` 15.1 items 12 and 14, and the claims that `geometry-models.md` 15.2 lists. The laws of triangles, the right-triangle relations, and the primary pentagon and hexagon laws are in `geometry-models.md` 10.2, 10.3, and 10.5, and the spherical law of cosines solved for a side, Theorem 2.5.4 of [RATCLIFFE], is in `geometry-models.md` 11.3 | `geometry-models.md` 10.5, 10.6, 15.1 items 12 and 14, and 15.2 | M9 | Chapter 2 of [BUSER], or Lean |
| UNV-13 | The inverse and metric of the band, and the choice of map between the disk and the half-plane. The maps between the disk, the hyperboloid, and the Klein model are sourced in `geometry-models.md` 4.2 | `geometry-models.md` 4.3 and 5.1, `rendering.md` 13.1 item 14 | M12 | [HUB], and round-trip tests |
| UNV-14 | The distance to a geodesic in the forms that no source states, which include the hyperboloid form for unit normals and the disk form for a geodesic that is not a diameter, and the distance to a great circle. The sourced forms are in `geometry-models.md` 9.3 and `rendering.md` 6.2 | `geometry-models.md` 9.3 and 15.1 item 3, `rendering.md` 13.1 items 8 and 10 | M3 tests, M13 | A source, or Lean |
| UNV-15 | The screen-distance estimate through the fold, analytic antialiasing, and the fade of tiles smaller than a pixel | `rendering.md` 7.1 and 13.1 items 19 and 20 | M3, M14 | Measurement against supersampling |
| UNV-16 | The lift onto the ball, the view scale of the sphere, and the Phong formulas | `rendering.md` 13.1 items 11 and 12, `geometry-models.md` 11.6 | M5 | [PHONG75], [BLINN77] |
| UNV-17 | Coset colourings are perfect for every orbifold group, the composition order, completeness, and conjugacy. The composition order is the rule of GRP-38 of `docs/design/group.md`: the fold that applies the sides $i_1, \ldots, i_n$ in that order puts the sample in the tile $gF$ with $g = s_{i_1} \cdots s_{i_n}$, and the colour is the number of the right coset $S g^{-1}$, traced from coset 0 through the inverse column of each side in reverse order, which names the left coset $gS$. A prototype outside the repository found the rule perfect, with the exact permutation of each side, on every class that it tried, found the forward trace not perfect on every class that is not normal, and found the reverse trace without the inverse to give the mirror image on rotation domains (Section 11.3 of `docs/design/group.md`). For conjugacy, the design lists every subgroup as its own colouring (`DEC-37`), after the prototype found that no sampled element of the group relates the pictures of two conjugate subgroups | `colourings.md` 4.2, 8.4, 8.7, and 14.1 items 1 to 3 | M8 | [DDL2009], [SCHWARZENBERGER1984], or Lean. [CBGS2008] defines colour types and states none of these (`colourings.md` 3.3 and 14.1 items 1 and 2) |
| UNV-18 | The parity colouring as the colouring by the two cosets of the orientation-preserving subgroup. That subgroup and its signature are sourced for non-Euclidean crystallographic (NEC) groups with a compact quotient (`colourings.md` 6) | `colourings.md` 6 and 14.2, `rendering.md` 4.6 | M3 | A source that links the colouring to the subgroup, or Lean |
| UNV-19 | The steps of Todd-Coxeter enumeration and of the low-index search that only implementations or the design state, which include the timing of the rule for one-letter relators, the order of the steps when the deduction stack overflows, and the distinct cyclic conjugates, as the column "Code-only part" of Section 7.3 of `docs/design/group.md` lists them, the safety of the first-in-class pruning on partial tables, which [HOLT2005] and [SIMS1994] state and argue without a proof, the bijection between subgroups and lenlex standard tables, which both state without a stand-alone proof, and the termination of SymPy's HLT code. For the low-index search, the parts that only the design states are the deductions pushed and found with every relator, the check of every relator at every coset of a complete table, the undo trail, the one run followed by runs by index, and the expansion of a class into its subgroups by relabelling from each coset, whose count of distinct tables rests on the bijection, as the column "Code-only part" of Section 7.11 of `docs/design/group.md` lists them. On a prototype outside the repository, the pruning of partial tables gave the same output as the test on complete tables only on 417 of 417 presentations, the check of complete tables never discarded a table with pushed deductions, and comparing an undefined entry as 0, which is the prototype's reading of the rule of `low_index`, lost classes on 389 of them (Section 11.3 of `docs/design/group.md`). [HOLT2005] and [SIMS1994] state most of the steps of enumeration, and prove termination for finite index under their conditions on the strategy. [HOLT2005] states that a run that stops at its limit shows nothing about the index, and [SIMS1994] states only that such a run stops without a result (`colourings.md` 7.10 to 7.15). The absence of a computable bound is stated by papers and by [HOLT2005] without a proof. On item 23 of `colourings.md` 14.1, a prototype outside the repository gives evidence and no source: a reimplementation of SymPy's Felsch loop that scans every popped deduction with the lists of its caller never finished within 16384 cosets on the side presentations of small finite groups, such as that of `332` of order 12, while scanning with the lists of the letter of the entry, as Note III.22 of [HULPKE2010], p. 166 of [HOLT2005], and p. 234 of [SIMS1994] state, finished on each. Without a closing pass, the lists of the caller also returned tables in which a relator was not closed (Sections 7.4 and 11.1 of `docs/design/group.md`) | `colourings.md` 7.8 to 7.17, 14.1 items 20 to 23 and 26, and 14.3 | M7, M8 | A source that proves the pruning safe or the bijection, a source that discusses the variants of item 23 of `colourings.md` 14.1, or Lean |
| UNV-20 | Text conventions and the canonical form: the canonical string, the delimiter for orders of 10 or more, the spelling of infinity, and a `1` on its own. The equivalences and the omission of an order of 1 are sourced (`orbifold-notation.md` 3.5 and 4.1) | `orbifold-notation.md` 12 items 1 to 4 | M1, M11 | A project convention. [CBG] and [BH] define none of them |
| UNV-21 | Geometry and parameters of non-compact signatures with $\chi \ne 0$: whether a signature with $\infty$, $\chi < 0$, and a mirror boundary or a crosscap gives a group, the geometry when $\chi > 0$, the deformation dimension, and the reading of the existence condition for orientation-preserving groups as $\chi < 0$. That condition is sourced in the terms of [BEARDON] (`orbifold-notation.md` 7.1, `fundamental-domains.md` 5.5) | `orbifold-notation.md` 12 item 15, `fundamental-domains.md` 9.8 | M1, M18 | A source, or Lean |
| UNV-22 | The Conway presentation for a boundary without corners in general, the dictionary between NEC signatures and orbifold signatures, and the signature of the rotation subgroup in orbifold notation for hyperbolic signatures and the spherical groups. The NEC signature, its presentation, and the rotation subgroup of a proper NEC group in NEC terms are sourced (`orbifold-notation.md` 9.2 and 9.3). On a prototype outside the repository, the Conway presentations of `*`, of `n*` for $n$ from 1 to 12, and of `2*`, read with one reflection for a boundary without corners, enumerated to $2/\chi$, which is evidence and no source | `orbifold-notation.md` 12 items 5, 12, and 14, gap G10 | M6, M7 | A re-reading of [CH] (RP-5), a source that states the dictionary, or Lean |
| UNV-23 | Whether the fold applies to a domain with an ideal vertex. The polygon theorem for such a domain is sourced (`fundamental-domains.md` 4.7) | `rendering.md` 13.1 item 3, `geometry-models.md` 7.6.5 | M18 | A source on the fold, or Lean |
| UNV-24 | MIT code inside a work under GPL-3.0-or-later | `symmhub.md` 17 item 1 | M15 | The licence texts, and the owner's judgement |
| UNV-25 | Correct rounding of the reference-arithmetic crate | `rust-ecosystem.md` 21 item 5 | M2 on | One comparison with MPFR |
| UNV-26 | Statements on the linear model in the conventions of Sections 5.1 and 5.2 of `docs/design/architecture.md`, with the time coordinate last and outward normals: the normal through two points, normals and the corner angle, a disk circle to a normal, and the Euclidean reflection as a matrix. [RATCLIFFE] states the cross product with the time coordinate first, the relation between normals and the angle with inward normals, and the Euclidean reflection as a map (`geometry-models.md` 2.4, 7.4, and 12) | `geometry-models.md` 2.4, 7.4, and 12 | M2, M4, M5, M15, M18 | A source in these conventions, or Lean |
| UNV-27 | Coordinates for the family, ranges, and default shape of polygons with prescribed angles. Existence, the admissible angles, and one polygon for each set of angles are sourced (`geometry-models.md` 10.1, `fundamental-domains.md` 9.4) | `geometry-models.md` 15.1 item 14, `fundamental-domains.md` 9.4 | M9 | A detailed reading of the proof in [TH], [BUSER] |
| UNV-28 | Properties of SymmHub's triangle: of its disk triangle, if it is used as an oracle, and in the Euclidean case the vertex coordinates of the triangle that `escher-domain` builds | `geometry-models.md` 15.1 item 5 | M2 tests, M4 | Tests only for the disk triangle. A source or Lean for the Euclidean coordinates |
| UNV-29 | The lune of `*nn`, the intersection of two hemispheres whose walls meet at the angle $\pi/n$, is a strict fundamental domain of the group of its two reflections. Theorem 6.4.3 of [DAVIS] covers convex polytopes, and a lune is not one in the sense of p. 75 of [DAVIS] | `fundamental-domains.md` 13.3 item 9 | M5 | A source that proves it, or Lean |
| UNV-30 | The lune of the angle $2\pi/n$ with its one rotation pairing, including the hemisphere of `22`, is a fundamental domain of the cyclic group of order $n$. The lune of `*nn` is already `UNV-29` | `fundamental-domains.md` 13.3 item 10 | M6 | A source whose polygon theorem admits a lune, or Lean. The exhaustive certificates of `docs/design/domain.md` check each build numerically |
| UNV-31 | The double of a convex reflection polygon across one wall is convex, and its vertex cycles are the vertex of each end of the doubled side alone and each other vertex with its image, with the corner orders of the reflection polygon | `fundamental-domains.md` 13.3 item 11 | M6 | A source, or Lean. Checks 3, 4, and 5 of `verify()` test each build |
| UNV-32 | The relators of a double as words: one generator for each side, standing for the inverse of its pairing as Section 5.3 of `docs/design/architecture.md` states, the pair relator $s_i s_p$ for each pair, and for each vertex cycle the generators of the sides that the walk of the cycle applies, in walk order, to the power of the order of the cycle. The polygon theorems state that the pairing relators and the cycle relators present the group (`fundamental-domains.md` 4.1, 4.2, 4.8 item 7, and 4.9), and SymmHub derives its relators in this way in code (`symmhub.md` 14.2), but no source read writes the cycle relators as words in this convention. Every vertex cycle of the domains of M6 visits one or two sides, so a reversed word or the opposite convention passes every numeric test on them | `fundamental-domains.md` 4.8 item 7, `symmhub.md` 14.2 | M7, M8 | A source that states the cycle relators as words in this convention, or Lean. The tests "Relators evaluate" and "Long cycle word" of `docs/design/domain.md` check it numerically |

## 8. Security Considerations

This document defines no component, so the threat model is that of the tool it plans. The attacker is assumed to control the input of `escher`: the signature string, every option value, every `name=value` setting, and the output path. Every milestone that adds such an input states, in the Security Considerations of its design document, the limits that Section 5.9 of `docs/design/architecture.md` requires: the maximum order, the signature length, the image size, the fold cap, and the search budget. Without these limits a crafted signature or option exhausts memory or time, for example through the colouring search of M8, whose cost grows exponentially with the index.

The output path is the only file that `render` writes, and no milestone reads a file supplied by the user. Every dependency enters through the milestone that first needs it, with the licence and advisory checks of cargo-deny (Section 8 of the tooling design). A native build, such as `rug`, would add code that runs at build time, which is one reason why `DEC-3` chooses the pure-Rust astro-float, used through its crate `astro-float-num`. Ported SymmHub code carries its MIT notice under `DEC-18`, and `UNV-24` tracks the open licence question.

Attacks on the repository itself, such as a malicious change to a document or a compromised CI runner, are out of scope, because `AGENTS.md` and the tooling design govern the repository and its workflow. The residual risk is that a limit chosen in a design document is too high for the machine that runs the tool, and that an unverified statement of Section 7 makes the tool draw a wrong image without any error.

## 9. Operational Considerations

The crates declare no Cargo feature during the proof of concept, as Section 3 of `docs/design/architecture.md` states. M3 adds the environment variable of the recipe that updates golden images, and documents it with its default in Section 14.2 of the tooling design. The only supported platform of the proof of concept is Linux, and rendering runs on the CPU only. The minimum supported Rust version (MSRV) is the one that the workspace declares, and `check-msrv` verifies it.

The plan has these failure modes:

| Failure | Handling |
| --- | --- |
| A milestone grows beyond a design branch and a few code branches | It splits along its Split field |
| A research prerequisite finds no construction | The milestone proceeds on the unverified statements of Section 7, or the owner decides to move the work to the roadmap (`DEC-19`) |
| A test reads a file, starts a process, or runs long numeric loops | It is excluded from Miri under the policy of `DEC-5`, in the form that TEST-11 of the coding standard and Section 4 of the tooling design state |
| A golden image differs between the container and CI | The comparison limits of Section 7 of `docs/design/architecture.md` apply, and a golden file changes only through its recipe |
| A CI job is cancelled because no runner acquired it | The job is rerun, and the merge waits (Section 14.4 of the tooling design) |

The tool emits no logs. `--stats` prints its statistics on request.

## 10. References

### 10.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 10.2. Informative References

- [B1] The entry in Section 16.1 of `docs/research/geometry-models.md`.
- [BEARDON] The entry in Section 14.1 of `docs/research/fundamental-domains.md`. `geometry-models.md` cites the same book as [BEA].
- [BH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [BLINN77] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [BUSER] The entry in Section 14.3 of `docs/research/fundamental-domains.md`. `geometry-models.md` cites the same book as [BUS].
- [CBG] The entry in Section 13.1 of `docs/research/orbifold-notation.md`. `fundamental-domains.md` cites the same book as [SYMMETRIES-OF-THINGS2008].
- [CBGS2008] The entry in Section 15 of `docs/research/colourings.md`. It is the same book as [CBG].
- [CFKP] The entry in Section 16.1 of `docs/research/geometry-models.md`.
- [CH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`. `fundamental-domains.md` cites the same paper as [CONWAY-HUSON2002].
- [CHOI2012] The entry in Section 14.3 of `docs/research/fundamental-domains.md`.
- [DAVIS] The entry in Section 16.1 of `docs/research/geometry-models.md`. `fundamental-domains.md` cites the same book as [DAVIS2008].
- [DDL2009] The entry in Section 15 of `docs/research/colourings.md`.
- [DEATON1993] The entry in Section 14.3 of `docs/research/fundamental-domains.md`.
- [FG] The entry in Section 13.1 of `docs/research/orbifold-notation.md`.
- [FRETTLOH2008] The entry in Section 15 of `docs/research/colourings.md`.
- [FWW] The entry in Section 16.2 of `docs/research/geometry-models.md`.
- [HOLT2005] The entry in Section 15 of `docs/research/colourings.md`.
- [HT] The entry in Section 16.1 of `docs/research/geometry-models.md`.
- [HULPKE2010] The entry in Section 15 of `docs/research/colourings.md`.
- [HUB] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [HUMPHREYS] The entry in Section 14 of `docs/research/rendering.md`.
- [KC24] The entry in Section 16.1 of `docs/research/geometry-models.md`. `rendering.md` cites the same paper as [KOPCZ24].
- [KLR] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [MASKIT2001] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [NORBURY2006] The entry in Section 14.2 of `docs/research/fundamental-domains.md`.
- [PHONG75] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [RATCLIFFE] The entry in Section 14.1 of `docs/research/fundamental-domains.md`, which cites the first edition. `geometry-models.md` cites the same edition as [RAT].
- [SCHWARZENBERGER1984] The entry in Section 15 of `docs/research/colourings.md`.
- [SCOTT] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [SERIES2013] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [SIMS1994] The entry in Section 15 of `docs/research/colourings.md`.
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
