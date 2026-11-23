# Proof-of-Concept Milestone Plan

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-23 |
| Applies to | The order of work of the proof of concept, from the first crate to the completion audit |

## Abstract

This document is the milestone plan of the Escher proof of concept, a Linux command-line tool that renders PNG images of a planar symmetry group given in orbifold notation and reports information on it. It defines how every milestone proceeds, the research that must precede some milestones, and the milestones in their planned order. For each milestone still to start it states the goal, the commands it adds, its scope, the design documents it extends, the sources and unverified statements it rests on, and the tests that accept it. For each earlier milestone it states the goal and what the milestone delivers. It also lists the documentation that each milestone updates and every unverified statement that the milestones rely on.

## 1. Introduction

### 1.1. Problem

The proof of concept covers the 17 Euclidean groups, every spherical signature, and the compact hyperbolic reflection triangles and reflection polygons with their rotation signatures, in three geometries, and much of its mathematics rests on code or on sources that the research could not read. Without an agreed order, an agent could start work whose mathematics has no subject-matter reference yet, or build on a statement that no source establishes without knowing it.

### 1.2. Purpose

This document fixes the order of work, what each milestone delivers, what it rests on, and how it is accepted. An implementing agent starts a milestone from its subsection in Section 5.

### 1.3. Audience

The primary readers are the coding agents that plan and implement a milestone. The secondary readers are reviewers and the owner of the repository, who approves the plan.

### 1.4. Scope

This document covers the workflow of a milestone, the owner's staging, the research prerequisites, the milestones `M1` to `M13` and `M19` to `M23`, the documentation obligations of each milestone, and the unverified statements that the milestones rely on. The milestones `M14` to `M18`, which the owner moved to the roadmap (`DEC-64`), keep their labels and a subsection each that points to the roadmap.

### 1.5. Non-Goals

Each fact has one owner, and this document cites the owner instead of restating the fact:

- `docs/design/architecture.md` owns the crates, the data flow, the conventions that cross crates, the testing strategy with the definition of a golden image, performance, numerical robustness, and the decisions. Section 10.1 of it records the owner's decisions, and Section 10.2 the labelled defaults `DEC-1` to `DEC-98`.
- `docs/design/roadmap.md` owns the phases after the proof of concept, the milestones `M14` to `M18` that the owner moved there, the work that closes requested sources, and the Lean workstream.
- The subject-matter references under `docs/research/` own the mathematics.
- `AGENTS.md` owns the hard rules, the development workflow, and Project Status.

This document records no progress. It states the milestones as a plan, and Project Status in `AGENTS.md` is the only place that records which milestone has merged (Section 3.6 of `docs/writing-documentation.md`).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms that Section 2.2 of `docs/writing-documentation.md` defines, and calls that document the writing guide. It calls `docs/rust-coding-standards.md` the coding standard and `docs/design/tooling.md` the tooling design. It adds these terms:

- milestone: A unit of the plan with one goal, labelled `M1` to `M23`, delivered by one design branch and a few code branches. The labels `M14` to `M18` name work that the owner moved to the roadmap. The planning documents of Section 3.3 carry the label `M0` and are not delivered this way.
- design branch: A branch named `feature/<name>-design` that changes only documents. It writes or extends the design documents of a milestone, and the research documents where a research prerequisite applies.
- code branch: A branch in its own worktree that changes code together with its tests and the document updates that Hard Rule 1 of `AGENTS.md` requires.
- research prerequisite: A design branch, labelled `RP-1` to `RP-6`, that adds to a research document what a milestone needs before its code can start (Section 4).
- unverified statement: A statement that a milestone relies on and that no cited source establishes, labelled `UNV-1` to `UNV-37` (Section 7).
- stage: One of the stages of the owner's staging (Section 3.2).
- proposed default: A decision labelled `DEC-n`, recorded in Section 10.2 of `docs/design/architecture.md`. Section 10.1 of that document states the defaults that the owner has decided, which keep their labels.
- reference arithmetic: The independent high-precision implementation of a few formulas inside a test suite, which Section 7 of `docs/design/architecture.md` defines.
- golden image: The committed reference image of a rendering test, which Section 7 of `docs/design/architecture.md` defines.
- general non-compact signature: A valid signature with an infinite order that is none of the 7 friezes, the triangle groups with ideal corners, and their rotation subgroups, which Section 2.1 of `docs/design/architecture.md` defines.
- corpus: The set of signatures that a test runs over.
- exhaustive certificate: The test of a finite group that enumerates the products of the side pairings breadth first, checks that it has exactly $2/\chi$ elements, and checks that every image of the centre of the domain folds back to the centre. `docs/design/domain.md` states it.
- fold: The per-pixel loop that maps a point into the fundamental domain, as Section 4.1 of `rendering.md` describes it.
- Conway presentation: The presentation read from a signature, as Section 9.1 of `orbifold-notation.md` defines it.
- side presentation: The presentation read from the side pairings and vertex cycles of a built domain, as item 7 of Section 4.8 of `fundamental-domains.md` states it.
- valid colouring: A colouring in which no two polygons that share an edge have the same colour, as `DEC-65` states.
- polygon mask: A PNG image that shows one fundamental polygon of a tiling, one image for each colour of a colouring (`DEC-66`).
- image fill: A fill colour given as an image file, from which a render samples the colour of each point of a tile (`DEC-66`).

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

PLAN-3. The design document of each crate MUST list every unverified statement of Section 7 that its code relies on, one row per label, with the section of the document that relies on it, because without the list a statement cannot be traced and stays open after a source closes it. The list MUST NOT hold tables of evidence for a statement, because evidence copied into a design document goes stale. The evidence belongs in Section 7, the research documents, and the tests, which check the statement numerically.

PLAN-4. An agent MUST NOT treat a milestone as accepted until every test that its "Tests and acceptance" field names exists and passes in `just check` and in CI, because those tests are the only evidence that the milestone meets its goal.

Hard Rule 15 of `AGENTS.md` bars project documents and milestone wording from code, tests, and configuration, so no label of this document appears there, and a design document is the only place that records an unverified statement.

### 3.2. Staging

The owner decided to stage the proof of concept. On 2026-11-22 the owner moved stage 3, and the milestones `M14` to `M18`, to the roadmap (`DEC-64` of `docs/design/architecture.md`). With that move, the completed proof of concept renders every signature of this set:

- The 17 Euclidean groups.
- Every spherical signature, within the limits on its orders.
- The compact hyperbolic reflection triangles `*pqr` and reflection polygons `*p1...pn` with 4 to 8 corners, and their rotation signatures.

Every other valid signature has no domain in the proof of concept. These are the hyperbolic signatures that mix cone points and mirrors or have handles or crosscaps, such as `4*3`, `2*23`, and `o2`, the friezes, the triangle groups with ideal corners, and the general non-compact signatures (`DEC-19`).

| Stage | Owner's scope | Milestones |
| --- | --- | --- |
| 1 | Reflection polygons and their rotation subgroups in all three geometries | M1 to M6, completed by M9 |
| 2 | All 17 Euclidean and all 14 spherical groups | M10, completed by M11 |
| 3 | General hyperbolic signatures, including handles and crosscaps, by porting the SymmHub pipeline, which is under the MIT licence | None. The owner moved it to the roadmap with M15 to M17 (Section 4.1 of `docs/design/roadmap.md`) |

The other milestones carry the owner's decisions on colourings (M7, M8, M20), models (M12), borders (M13), Schlafli symbols (M21), polygon masks and image fills (M22), and Wythoff constructions (M23). The parser accepts signatures with infinite orders from M1, and rendering the friezes and the triangle groups with ideal corners is the roadmap item M18. M19 checks that every signature of the set renders.

### 3.3. Planning Documents

The planning documents, labelled M0, are the precondition of M1 and have no code to implement. They are `docs/design/architecture.md`, this plan, and `docs/design/roadmap.md`, which live in `docs/design/` as plans without status (`DEC-2`), together with the updates to `AGENTS.md` (Project Status and Repository Layout) and to `README.md`. They are accepted when `just check` passes and when they call every crate planned.

### 3.4. Milestone Overview

The owner set the order of the remaining work on 2026-11-22 and placed M23 in it on 2026-11-23 (`DEC-63`). A cleanup comes first and is not a milestone. Then come M20, M12 and M13, M21, M22, and M23, and the completion audit M19 comes last. The order of the new milestones M20, M21, and M22 may still change, and the owner sets it. The table lists the milestones in the planned order. The column "Depends on" names the milestones and research prerequisites whose results a milestone uses, as Section 5 gives them. The milestones M14 to M18 are not in the table, because the owner moved them to the roadmap (Section 3.2).

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
| M9 | Hyperbolic reflection polygons with shape parameters | `*p1...pn` with 4 to 8 corners and its parameters | `escher-domain`, `escher-render` (tests and golden images), `escher-cli` | M6, RP-3 (sheet and slice part) |
| M10 | The 17 Euclidean groups | All 17 Euclidean signatures, and the parameter `skew` of `o` and `2222` | `escher-domain`, `escher-cli`, `escher-render` (tests and golden images) | M4, M6, M8, RP-2 |
| M11 | The 14 spherical types | All 14 spherical types, with `3*2`, `2*n`, `n*`, `nx`, `*`, `x`, and `1` added | `escher-domain`, `escher-cli`, `escher-render` (tests and golden images) | M5, M6, M7, M8, M10, RP-2 |
| M20 | Valid colourings | Colouring counts, listings, and ids restricted to the valid colourings, one for each conjugacy class, with the colouring with one colour still available | `escher-group`, `escher-cli`, `escher-render` (tests and golden images) | M8, M9, M10, M11 |
| M12 | Half-plane, band, and Klein models | `--model` | `escher-geometry`, `escher-render`, `escher-cli` | M3 |
| M13 | Intrinsic border width | `--border-unit intrinsic` | `escher-render`, `escher-geometry`, `escher-cli` | M3, with M4 and M5 for the other geometries and M12 for one test |
| M21 | Schlafli symbols in `escher info` | The Schlafli symbol of the regular tiling of a signature and of its dual | `escher-cli` | M1 |
| M22 | Polygon masks and image fills | `escher render-masks`, and image fills in `escher render` | `escher-render`, `escher-cli` | M8 |
| M23 | Wythoff constructions | Wythoff tilings of `*pqr` from on or off marks on the mirrors, with one fill for each face type, in a form that the design defines | `escher-render` and `escher-cli`, as expected before the design | M2, M22, RP-6, with M4 and M5 if the design covers the other geometries and M6 if it covers the rotation signatures |
| M19 | Completion audit | None | All six | M11, M12, M13, M20, M21, M22, M23 |

## 4. Research Prerequisites

Some milestones need material that a research document must supply first, so a docs-only research branch precedes each milestone that needs such material.

| Label | Adds | To | From | Before |
| --- | --- | --- | --- | --- |
| RP-1 | A step-level account of Todd-Coxeter coset enumeration, of the low-index subgroup search, and of Reidemeister-Schreier rewriting, with SymmHub's implementation and worked examples | `colourings.md` 7.8 to 7.21, and `symmhub.md` 12.8 | The sources that `colourings.md` 7.8 lists, with a line-level reading of SymmHub's `lib/sublib/src/sublib_core.js` and `reidemeister.js`. `symmhub.md` 17 item 8 records which files were read line by line | M7 |
| RP-2 | The geometry of the functions of SymmHub's `lib/orbilib/WallpaperGroups_NonNegative.js`, with the domain and the side pairings of each Euclidean and spherical group at line level | `fundamental-domains.md` 9.5 and 9.6. The line-level reading, worked examples, defects, and a numeric check by a port are in `symmhub.md` 7.1.1 to 7.1.9 and 15.3, and the construction as steps for an implementer, with what the published sources establish, in `fundamental-domains.md` 8.14, 9.5.1 to 9.5.3, and 9.6.1 to 9.6.3 | SymmHub source code, with [CH], [TH], [VONGAGERN2009], [CBG], [RATCLIFFE], and [BEARDON] for what the sources establish. `symmhub.md` 17 items 8, 14, and 15 record which lines were read and what the port tried | M10, M11 |
| RP-3 | `atomizeOrbifold`, the twelve `unfoldAtom` cases, the weld, the walls, and `produceGenerators` at line level, for the part of gap G4 that stays open: SymmHub's order of the cuts, its pairing of corners of order 2 across the end of the signature, and its departures from the cut rules of Deaton's thesis (`fundamental-domains.md` 8.15) | `fundamental-domains.md` 8, and `symmhub.md` 6. The sheet and slice part for `*p1...pn` is in `symmhub.md` 6.2.1 to 6.9, 8.1, and 15.3, and in `fundamental-domains.md` 8.4 to 8.14 and 9.4.1 to 9.4.3 | SymmHub source code, with [TH] pp. 311 to 318, [BEARDON], [RATCLIFFE], and [DAVIS] for the sheet and slice part. `symmhub.md` 17 item 8 records which lines were read | M9 for its sheet and slice part. The rest is for the roadmap item M15 |
| RP-4 | `Group_Frieze`, and the ideal corners of the triangle groups and their rotation subgroups | `fundamental-domains.md` and `symmhub.md` | SymmHub source code | The roadmap item M18 |
| RP-5 | What [CH] pp. 255 to 257 states for a boundary without corners (`orbifold-notation.md` 12 item 14), and the clearing of the rows of `source-requests.md` 7, "Obtained Since" | `orbifold-notation.md` 9.1 and 12, and `source-requests.md` | [CH], which is held locally | No milestone. It may run at any time |
| RP-6 | The Wythoff construction on a reflection triangle from primary sources: the faces that each pattern of marks on the mirrors produces and how they fall into face types (`colourings.md` 14.1 item 10, without the snubs), and a per-pixel classification of a folded point into a face and its type (item 11) | `colourings.md` 10, 13, and 14, and `source-requests.md` | The sources that `colourings.md` 14.3 lists for items 10 and 11, and the passage on Wythoff triangles of [CBGS2008] that it names, with the precedents of `colourings.md` 10.6, `prior-art.md` 6.7, and `rendering.md` 4.4. A source that cannot be read goes into `source-requests.md` | M23 |

RP-3, apart from its sheet and slice part, and RP-4 serve work that the owner moved to the roadmap (Section 4.1 of `docs/design/roadmap.md`), so no milestone of this plan waits on them. RP-6 precedes M23, because M23 draws faces by a face rule for each pattern of marks and by a per-pixel classification of a point into a face, which `colourings.md` 10 sources or records as unverified first. The design branches of M20 and M21 add to the subject-matter references the sources for the mathematics that they rely on, or record it as unverified there (Hard Rule 14 of `AGENTS.md`).

The research on the general non-compact signatures is not a prerequisite of any milestone. It is a deep-research item of Section 5 of `docs/design/roadmap.md`.

## 5. Milestones

Each command in this section illustrates its milestone, and Project Status in `AGENTS.md` says which milestones are complete. Each output is what the milestone delivers, and `docs/design/cli.md` defines the exact text. The subsections of M1 to M11 state only the goal and what the milestone delivers. Every other subsection of a milestone of this plan uses these fields in this order: Goal, Command line, Scope, Design documents, Sources, Unverified statements relied on, Tests and acceptance, Dependencies and `deny.toml` work, Risks, and Split where one applies. The subsections of M14 to M18 point to the roadmap.

### 5.1. M1: Signature and Classification

**Goal.** The full notation is parsed, validated, canonicalized, and classified.

**Delivers.** The crate `escher-signature`, which parses a signature with parentheses for orders of 10 or more and both spellings of infinity, and gives its canonical form, its exact Euler characteristic, and its classification, and the command `escher info`. For example, `escher info '*732'` prints hyperbolic, $\chi = -1/84$, compact, area $\pi/42$, and deformation dimension 0, and `escher info '*3333'` prints deformation dimension 1. `escher --version` prints the package name and version.

### 5.2. M2: Linear Geometry and the Hyperbolic Reflection Triangle

**Goal.** The linear hyperbolic model and the first domain exist, checked against reference arithmetic.

**Delivers.** The crate `escher-geometry`, with the linear model of the hyperbolic plane and the disk map, the crate `escher-domain`, with the compact reflection triangle `*pqr` and its verification, and the domain summary and the option `--check` of `escher info`.

### 5.3. M3: First Image

**Goal.** The vertical slice: fold, flat colours, pixel borders, supersampling, and PNG output.

**Delivers.** The crate `escher-render` and the command `escher render`, which draws `*pqr` in the Poincare disk with the colourings `one` and `parity`, borders of a width in pixels, a sample grid, and statistics, and writes a PNG file.

### 5.4. M4: Euclidean Reflection Groups and the First Shape Parameter

**Goal.** `*333`, `*442`, `*632`, and `*2222`, and the parameter machinery from end to end.

**Delivers.** The module `euclidean` of `escher-geometry`, the Euclidean triangles and the rectangle of `*2222` with its parameter `aspect`, the Euclidean view, the option `--set` of `escher render`, and the `Parameter` lines of `escher info`.

### 5.5. M5: The Sphere

**Goal.** The spherical reflection groups `*332`, `*432`, `*532`, `*n22`, and `*nn`, drawn as a lit ball.

**Delivers.** The module `spherical` of `escher-geometry`, the spherical triangles and the lune, checked by the exhaustive certificate, the ball view with Phong shading, and the options `--rotation`, `--light`, and `--shading` of `escher render`.

### 5.6. M6: Rotation Subgroups

**Goal.** The first side pairings that are not reflections.

**Delivers.** The rotation signatures `pqr`, `22n`, `nn`, and `2222` as doubles of reflection domains, their rendering, the `Side pairs` and `Parity colouring` lines of `escher info`, and the default colouring of `escher render`. M20 withdraws the `Parity colouring` line and changes the default colouring (`DEC-72`).

### 5.7. M7: Presentations and Coset Enumeration

**Goal.** The crate `escher-group` exists, with presentations and Todd-Coxeter coset enumeration.

**Delivers.** `escher-group` with the Conway presentation and bounded coset enumeration, the side presentation of a domain, and the option `--presentation` of `escher info`.

### 5.8. M8: Colouring Search With Ids

**Goal.** The low-index subgroup search, colouring ids, the listing in `info`, and coset colours in `render`.

**Delivers.** The search of `escher-group`, with one colouring for each subgroup of finite index (`DEC-37`), the colouring lines and the options `--max-colours` and `--colourings` of `escher info`, the fill by cosets of `escher-render`, and `escher render --colouring <id>` with a palette of 12 colours (`DEC-39`). M20 restricts these colourings (`DEC-37`, `DEC-65`).

### 5.9. M9: Hyperbolic Reflection Polygons With Shape Parameters

**Goal.** Compact hyperbolic `*p1...pn` with 4 to 8 corners and its $n - 3$ parameters. This milestone completes stage 1.

**Delivers.** The reflection polygons, built from corner pairs, slices, and sheets, with their parameters and their guard (`DEC-46` to `DEC-49`), and their rotation signatures as doubles at the default shape (`DEC-50`).

### 5.10. M10: The 17 Euclidean Groups

**Goal.** The remaining nine Euclidean groups and the parameter `skew` of `o` and `2222`.

**Delivers.** The domains of `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3` and the sheared `2222`, with translations and glide reflections as pairings, behind a guard and the wallpaper certificate (`DEC-51` to `DEC-55`).

### 5.11. M11: The 14 Spherical Types

**Goal.** The remaining spherical groups `3*2`, `2*n`, `n*`, `nx`, `*`, `x`, and `1`. This milestone completes stage 2.

**Delivers.** Their domains, with rotary reflections as pairings and a domain with no side for `1`, behind a guard and a certificate of each group (`DEC-56` to `DEC-62`).

### 5.12. M12: Half-Plane, Band, and Klein Models

**Goal.** `--model disk|half-plane|band|klein`, with the Poincare disk as the default.

**Command line.** This command renders in the band model:

```sh
escher render '*732' --model band -o band.png
```

**Scope.** The maps from the half-plane, the band, and the Klein model to the disk with their Jacobians in `escher-geometry` (Sections 7.4 to 7.7 of `docs/design/geometry.md`). The option `HyperbolicModel` of `escher-render`, a canonical view for each model, and borders through the gradient estimate with screen tangents built from the Jacobian, so that the Klein model needs no special case (Section 5.16 of `docs/design/render.md`). The option `--model` of `escher render` (Section 4.6 of `docs/design/cli.md`). The images of the disk keep their bytes (RND-50 of `docs/design/render.md`).

**Design documents.** Extensions of `geometry.md`, `render.md`, and `cli.md`, and the labels `DEC-74` to `DEC-77` of `docs/design/architecture.md`, with the amendment of its row "Disk and half-plane" of Section 5.10.

**Sources.** The map from the half-plane to the disk of [RATCLIFFE], Exercise 4.4.4, p. 126 (`geometry-models.md` 4.3). The connecting maps of [CFKP] for the map from the Klein model to the disk, and the inverse of [BEARDON], p. 128, for the tests (`geometry-models.md` 4.1 and 4.2, `rendering.md` 8.1). The band map and its constant (`geometry-models.md` 5.1). The evaluation of the complex inverse hyperbolic tangent by logarithms ([B1], `geometry-models.md` 5.1), in the reference arithmetic of the tests only.

**Unverified statements relied on.** `UNV-13`, and `UNV-4` and `UNV-15` through the renderer. The inverse of the band map is stated only in SymmHub's code (`rendering.md` 13.1 item 14), and the choice of the map from the half-plane to the disk is the owner's (`DEC-75`, `geometry-models.md` 4.3). The design uses no band metric, because the border estimate takes the Jacobian of the map. The Jacobians of the three maps, and the screen tangents built from them, are unverified statements of the geometry design and the render design, which their tests check against central differences. The scale relation, the orientation, and the inverse of the half-plane map are unverified statements of the geometry design, which its tests pin.

**Tests and acceptance.** Round trips against reference arithmetic. The same hyperbolic point has the same tile colour in all four models. Borders in the Klein model are straight (Section 9 of [CFKP]). Golden images. Section 10.1 of `docs/design/geometry.md`, Section 8.1 of `docs/design/render.md`, and Section 8.1 of `docs/design/cli.md` name the tests.

**Dependencies and `deny.toml` work.** `num-complex` 0.4.6 in `escher-geometry`, without its default features and with `std`. Its licence, `MIT OR Apache-2.0`, is on the allow-list, and its one dependency, `num-traits`, is already in `Cargo.lock`. `just check-deny` confirms the licence and that no duplicate resolves, and `just check-msrv` the minimum Rust version (Section 11 of `docs/design/geometry.md`).

**Risks.** Periodicity of the band is not a goal (`geometry-models.md` 5.3).

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Geometry: `disk_from_half_plane`, `disk_from_band`, and `disk_from_klein` with their Jacobians, the dependency on `num-complex`, and their tests in `escher-geometry`.
3. Render and command line: `HyperbolicModel` and the views of `escher-render` with their tests and three golden images, `--model` of `escher-cli` with its snapshots and plain tests, and Project Status in `AGENTS.md`.

### 5.13. M13: Intrinsic Border Width

**Goal.** `--border-unit pixels|intrinsic`, with `pixels` as the default. With `intrinsic`, `--border-width` is a length in the metric of the geometry.

**Command line.** This command draws borders 0.05 wide in the metric of the geometry:

```sh
escher render '*732' --border-unit intrinsic --border-width 0.05 -o intrinsic.png
```

**Scope.** The distance from a point to a wall in each geometry, as the functions `wall_distance` of the modules `hyperbolic`, `euclidean`, and `spherical` of `escher-geometry` (Section 5.6 of `docs/design/geometry.md`). The option `BorderUnit` of `escher-render` and the intrinsic border test, which compares the distance from the folded point of a sample to each side of the domain with half the width (Section 5.17 of `docs/design/render.md`). The width is the full width of the band, as a width in pixels is. The pixel unit keeps every image byte for byte (RND-55 of `docs/design/render.md`). The option `--border-unit` of `escher render` (Section 4.6 of `docs/design/cli.md`). The units are those of Section 5.5 of `docs/design/architecture.md`, with the Euclidean unit of `DEC-17`.

**Design documents.** Extensions of `geometry.md`, `render.md`, and `cli.md`, and the labels `DEC-78` to `DEC-80` of `docs/design/architecture.md`.

**Sources.** Precedents with intrinsic widths (`rendering.md` 6.1). The distance from a point to a hyperplane of [RATCLIFFE], equation (3.2.8) and Theorem 3.2.12, p. 75 (`geometry-models.md` 9.3). The form on the sphere from the formula of [MOVABLE] (`rendering.md` 9.4). For the wedge test, the distance to the imaginary axis of [BEARDON], equation (7.20.3), and the equidistant curves of [CFKP], Fact 2 (`rendering.md` 6.2, `geometry-models.md` 9.3).

**Unverified statements relied on.** `UNV-14`: the hyperboloid form for a unit normal with the time coordinate last (`geometry-models.md` 9.3 and 15.1 item 3) and its reading for a vector off the hyperboloid, and the form $\sin d = \lvert x \cdot u \rvert$ on the sphere, a candidate statement (`rendering.md` 13.1 item 10). The design uses no form in the disk, so it does not rely on item 8 of `rendering.md` 13.1. The Euclidean distance $\lvert c \cdot x \rvert$ is an unverified statement of the geometry design without a label (Section 12 of `docs/design/geometry.md`). A test pins each form against a minimization of the distance between points.

**Tests and acceptance.** The test "Distance by minimization" and the accuracy rows of `wall_distance` of Section 10.1 of `docs/design/geometry.md`. The tests "Wedge", "Same point in every model", "Intrinsic each pixel", and "Pixel unit unchanged", and three golden images, of Sections 8.1 and 8.4 of `docs/design/render.md`. The snapshots and plain tests of Section 8.1 of `docs/design/cli.md`.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Small. Where a model shrinks a tile below a pixel, such as near the limit circle, an intrinsic border is narrower than the sample spacing and breaks into separate samples, which supersampling does not prevent (Section 5.17 of `docs/design/render.md`).

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Geometry: the three functions `wall_distance` with their accuracy rows and their tests in `escher-geometry`, including the minimization in the reference arithmetic of the tests.
3. Render and command line: `BorderUnit`, the intrinsic border test, and the tests and three golden images of `escher-render`, `--border-unit` of `escher-cli` with its snapshots and plain tests, and Project Status in `AGENTS.md`.

### 5.14. M14: Analytic Border Antialiasing

The owner moved this milestone to the roadmap (`DEC-64`). Section 4.1 of `docs/design/roadmap.md` states its goal and what it needs.

### 5.15. M15: General Hyperbolic Signatures I, Cone Points and Mirrors

The owner moved this milestone to the roadmap (`DEC-64`). Section 4.1 of `docs/design/roadmap.md` states its goal and what it needs.

### 5.16. M16: General Hyperbolic Signatures II, Handles, Crosscaps, and Several Boundaries

The owner moved this milestone to the roadmap (`DEC-64`). Section 4.1 of `docs/design/roadmap.md` states its goal and what it needs.

### 5.17. M17: Robust Reduction for General Domains

The owner moved this milestone to the roadmap (`DEC-64`). Section 4.1 of `docs/design/roadmap.md` states its goal and what it needs.

### 5.18. M18: Infinite-Order Points, Friezes and Ideal Vertices

The owner moved this milestone to the roadmap (`DEC-64`). Section 4.1 of `docs/design/roadmap.md` states its goal and what it needs, and why the general non-compact signatures stay outside it.

### 5.19. M19: Completion Audit

**Goal.** Every signature of the set of Section 3.2 renders, and `escher render` refuses every other valid signature with a diagnostic.

**Command line.** No new capability.

**Scope.** A property test that generates valid signatures up to a size bound. It renders each signature of the set of Section 3.2 at a small size without error. It checks that `render` refuses every other valid signature with a diagnostic and exit status 1, and a general non-compact signature as Section 5.8 of `docs/design/architecture.md` states. The snapshot corpus of `info`. The reconciled list of unverified statements across the design documents, without the statements that only the roadmap items M14 to M18 rely on. The final usage in `README.md`, and Project Status in `AGENTS.md`.

**Design documents.** Every crate design document, for the reconciled list of unverified statements.

**Sources.** None.

**Unverified statements relied on.** None.

**Tests and acceptance.** The property test of the scope passes, and every crate design document meets PLAN-3.

**Dependencies and `deny.toml` work.** None.

**Risks.** The audit exposes signatures of the set that the corpus missed, such as signatures at the limits on their orders.

### 5.20. M20: Valid Colourings

**Goal.** The colourings that `escher` counts, lists, and draws by id are only the valid colourings, one for each conjugacy class of subgroups (`DEC-37`), and the colouring with one colour stays available. The owner's criterion (`DEC-65`) is "A colouring is valid if and only if no adjacent polygons share a colour." Two polygons are adjacent when they share an edge.

**Command line.** These commands illustrate the milestone. `<id>` stands for an id that `escher info --colourings` lists:

```sh
escher info '*642'
escher info --colourings '*642'
escher render '*642' --colouring <id> -o valid.png
```

- `escher info '*642'` prints the number of valid colourings of each colour count up to the maximum colour count.
- `--colourings` lists only those colourings, each with its id.
- `render --colouring <id>` draws a colouring of that list. `one` and `1a` still draw the colouring with one colour.

**Scope.** The restriction lives in `escher-group`, whose search takes a `ColouringSet` and, for `Valid`, drops each subgroup that contains a side generator as it expands a class into its subgroups (GRP-42 and GRP-43 of `docs/design/group.md`). `All` stays for the tests against the counts of the sources. For `ValidClasses`, the search keeps one valid colouring for each class (GRP-44), and the command line uses that set. The ids number the colourings of a set with no gap (`DEC-71`). `escher info` counts and lists only the valid colourings (`DEC-73`), and `render` without `--colouring` draws the valid colouring with 2 colours when the domain has one, and one colour otherwise (`DEC-72`). Section 4.6 of `docs/design/cli.md` states the aliases `one` and `parity`, and Section 8.4 of `docs/design/render.md` the coset golden images.

**Design documents.** Extensions of `group.md`, `cli.md`, and `render.md`, the amendments of `docs/design/architecture.md` (`DEC-7`, `DEC-15`, `DEC-32`, `DEC-37`, `DEC-38`, `DEC-43`, `DEC-44`, `DEC-65`, and `DEC-71` to `DEC-73`), and `colourings.md` for `UNV-35` and `UNV-36`.

**Sources.** The coset colourings of `colourings.md` 4.1. No source read states the criterion in terms of the side permutations (`UNV-35`), or that a domain has at most one valid colouring with 2 colours (`UNV-36`).

**Unverified statements relied on.** `UNV-35` and `UNV-36`, and `UNV-17`, `UNV-19`, and `UNV-32` through the search of M8.

**Tests and acceptance.** The tests of the row "Valid colourings" of Section 10.1 of `docs/design/group.md`, the test "Valid neighbours" and the regenerated coset golden images of Sections 8.1 and 8.4 of `docs/design/render.md`, and the snapshots and plain tests of Section 8.1 of `docs/design/cli.md`.

**Dependencies and `deny.toml` work.** None new.

**Risks.** A signature may have no valid colouring up to the maximum colour count, which leaves only the colouring with one colour. Command lines that users saved with ids of M8 draw another colouring or fail.

**Split.** Four branches, in this order:

1. Design: the design documents above, and `colourings.md`.
2. Search: `ColouringSet`, `contained_generator`, the filter, and the ids of `escher-group`, with its tests.
3. Classes: `ColouringSet::ValidClasses` of `escher-group`, with its tests.
4. Command line: the counts, the listing, the aliases, and the default colouring of `escher-cli`, the tests and coset golden images of `escher-render`, the snapshots, the examples and images of `README.md`, and `scripts/readme-images`, which renders the colouring `4a` of `*642` to `four.png` and `four-colours.png`, because `*642` has no valid colouring with 3 colours.

### 5.21. M21: Schlafli Symbols in `escher info`

**Goal.** `escher info` prints the Schlafli symbol of the regular tiling of a signature and of its dual, for regular tilings only (`DEC-67`). For `*732` these are `{7,3}` and `{3,7}`. The line shows both symbols, so `info` has no `--dual` option.

**Command line.** These commands illustrate the milestone:

```sh
escher info '*732'
escher info 732
escher info '*442'
```

- `escher info '*732'` prints the line `Regular tiling: {7,3}, dual {3,7}`.
- `escher info 732` prints the same line as `*732`, and `escher info '*442'` prints `Regular tiling: {4,4}, self-dual`.

**Scope.** The line `Regular tiling` of `escher info` (Section 5.9 of `docs/design/cli.md`). It appears for the signatures whose canonical form is `*pq2` or `pq2` with finite $p \ge q \ge 3$, in all three geometries, and for no other signature (`DEC-81`). So `*n22`, `n22`, and every signature with an infinite order print no such line. The larger order names the tiling and the smaller its dual, and a tiling with $p = q$ is marked self-dual (`DEC-82`). The line follows the deformation dimension, needs no domain, and is computed in `escher-cli` from the public accessors of `Signature`, so `escher-signature` does not change (`DEC-83`). The milestone names tilings and draws nothing new.

**Design documents.** Extensions of `cli.md`, the labels `DEC-81` to `DEC-83` of `docs/design/architecture.md`, and item 16 of Section 12 of `orbifold-notation.md`.

**Sources.** For $p, q \ge 3$, the symmetry group of the regular tiling $\{p, q\}$ and of $\{q, p\}$ is the reflection group of the right triangle with the angles $\pi/2$, $\pi/p$, and $\pi/q$, and its rotation group has index 2 (`colourings.md` 5.1 and 5.4). The $p$-gon of $\{p, q\}$ divides into $2p$ right triangles (`geometry-models.md` 10.1, `colourings.md` 5.1). `fundamental-domains.md` 9.1 states the reflection triangle group `*pqr` with its presentation. No source read names that group `*pq2` in any geometry, or names the rotation group `pq2`. For the hyperbolic plane, no source read places the corners of orders $p$ and $q$ or calls $\{q, p\}$ the dual. For the sphere and the Euclidean plane, `colourings.md` 10.3 does both (`UNV-37`, `orbifold-notation.md` 12 item 16).

**Unverified statements relied on.** `UNV-37`, and `UNV-20` through the canonical form of `escher-signature`, on which the rule of the line is stated.

**Tests and acceptance.** The snapshots and unit tests of the line in Section 8.1 of `docs/design/cli.md`.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Small. Every snapshot of `info` whose signature gets the line changes, and so does the example of `*632` in `README.md`.

**Split.** Two branches, in this order:

1. Design: the design documents above.
2. Command line: the line of `escher-cli` with its unit tests, the new and updated snapshots, the example of `escher info` in `README.md`, and Project Status in `AGENTS.md`.

### 5.22. M22: Polygon Masks and Image Fills

**Goal.** The user fills the tiles with images painted over polygon masks, in the owner's flow (`DEC-66`):

1. A new command `render-masks` renders, for a given orbifold and colouring, the polygon masks that correspond to each distinctly coloured polygon of the tiling, with the masks enumerated.
2. With an external tool, the user uses those masks to produce a set of images to sample from.
3. The user passes the masked image file names as fill colours, for example `image:/path/to/file.png`, and the render samples from them.

**Command line.** These commands illustrate the flow. The first writes `mask-0.png` to `mask-3.png`, one for each colour of `4a`, and the second renders the four images that the user painted over them:

```sh
escher render-masks '*642' --colouring 4a -o /tmp/masks/mask.png
escher render '*642' --colouring 4a --colours 'image:/tmp/masks/fill-0.png,image:/tmp/masks/fill-1.png,image:/tmp/masks/fill-2.png,image:/tmp/masks/fill-3.png' -o tiling.png
```

**Scope.** The owner decided these details:

- Each mask is one fundamental polygon, with one PNG file for each colour.
- A tile that is a mirror image of another tile of its colour shows the image mirrored.
- On 2026-11-23: a mask shows the domain as a flat Euclidean polygon with straight edges, mapped from the domain by a map that the design chooses, so that the user can test in an image editor that the fill images tile correctly across borders (`DEC-93`). A flat triangle keeps the right angle, then the angle of the next smallest corner order, and adjusts the last corner. Only edges must match, and copies laid edge to edge need not close up around a corner. Each edge has a colour-coded border (`DEC-94`).
- On 2026-11-23: an image fill has exactly the size limits of an output image, and no other limit (`DEC-92`).

The design settles the open points of the planning as proposals, and on 2026-11-23 the owner accepted three of them: the refusal of `DEC-96`, the thin flat triangles of `DEC-95`, and the part of `DEC-98` in which the edges that can meet share one code colour and a mark shows how they meet. The proposals are these:

- The mask polygon is the domain itself for a Euclidean domain, a flat triangle by the owner's angle rule for a reflection triangle, the Klein positions of the vertices in a chart centred at the normalized sum of the vertices for a reflection polygon and its double, and the flat triangle of the generating domain with its mirror image for a double of a triangle. A domain with fewer than three sides, and `*222` with the domains built on it, has none and is refused (`DEC-95`, `DEC-96`, Section 5.22 of `docs/design/render.md`).
- A radial map in a chart in which the walls are straight lines carries the domain onto the mask polygon, each side onto its edge in proportion to the distance along the side (`DEC-97`, Section 5.23 of `docs/design/render.md`).
- A mask has the size of `--size` and shows the mask polygon in white, centred, on black, with a border inside each edge in the code colour of its seam class, which holds the edges that can meet in the tiling, and a black mark at the marked end of each edge, which shows how the edges meet, antialiased by a fixed 4 by 4 sample grid (`DEC-98`, Section 5.19 of `docs/design/render.md`). The mask of colour $c$ is the file named by the stem of `--output`, `-`, and $c$ from 0, such as `mask-0.png` (`DEC-87`, CLI-58 of `docs/design/cli.md`).
- `--colours` mixes hexadecimal colours and image items `image:FILE`, one item for each colour. An image item makes `render` draw `one` and `parity` through the coset fills of `1a` and `2a` (`DEC-91`, CLI-61 of `docs/design/cli.md`).
- An image fill reads every PNG colour type at a bit depth of 8 or less as 8-bit sRGB with alpha, refuses an image with 16 bits per channel, reads an image of any size with the mask polygon placed in it as in a mask of that size, samples bilinearly in linear light over the background colour, and takes the background colour where a position maps outside the image (`DEC-88` to `DEC-90`, Sections 5.20 and 5.21 of `docs/design/render.md`).

The mask polygon and the mask map stay private, and the faces of a Wythoff tiling will reuse them (`DEC-88`, Section 5.23).

Out of scope: research into external tools, which the owner ruled out (Section 1 of `docs/design/roadmap.md`).

**Design documents.** Sections 5.18 to 5.23 of `render.md` with its other sections that the masks and image fills touch, Sections 4.6, 4.8, and 6.10 of `cli.md` with its other sections that the command touches, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-84` to `DEC-98`, and its crate table and Security Considerations.

**Sources.** The per-pixel fold, which gives the point in the domain (`rendering.md` 4.1). The colour of the neighbour of the domain across a side comes from the coset table by the rule of GRP-38 of `docs/design/group.md` (`UNV-17`), and the pairs of colours that occur across a side from a closure of the table that no source states. The Klein positions and the walls as straight chords in the Klein model (`geometry-models.md` 3.1 and 4.2). SymmHub's sampling of an image in a pattern box, and the mirrored texture lookup of Nelson and Zucker, as precedents (`rendering.md` 10).

**Unverified statements relied on.** `UNV-1` and `UNV-2` for the fold, and `UNV-17` for the colour of the neighbour of the domain across a side, on which the seam classes of the masks rest. The closure that gives the pairs of colours across a side, the marks of the seam classes, the convexity and the equal edges of the mask polygon, the chart of the sphere, the centres and rays of the mask map, the bijection of the mask map, and the distinct code colours are unverified statements of the render design, which its tests check (Section 10 of `docs/design/render.md`). The milestone adds no row to Section 7.

**Tests and acceptance.** The tests "Polygon rules", "No polygon", "Round trip", "Edge fraction", "Euclidean identity", "Mask content", "Seam classes", "Code colours", "Flat image", "Texture contract", "Mirrored", "Seams", "Same texture in every model", and "Alpha and fallback", the decoding and failure tests, and the golden images of a mask and of image fills, of Sections 8.1, 8.2, and 8.4 of `docs/design/render.md`. The snapshots and plain tests of `render-masks` and of the image items in Section 8.1 of `docs/design/cli.md`, which include a missing, malformed, and oversized image file with exit status 1 and no file written.

**Dependencies and `deny.toml` work.** None new. `escher-render` decodes with its existing dependency `png` (Section 9 of `docs/design/render.md`).

**Risks.** The image file is the first input file that `escher` reads (Section 8). A file that blocks or never ends holds `render`, because no limit applies to the length of the file (Section 12 of `docs/design/cli.md`). Bilinear sampling with no level of detail aliases an image where a tile is much smaller than the image. A render with other settings than the masks keeps the colours but can change the mask polygon, so the images no longer match it. The mask map distorts the picture inside a tile, more so for a thin flat triangle, such as that of `*(100)(100)2`. With no budget, the images of one render can take up to $2^{28}$ bytes of texels each (`DEC-92`).

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Render library: `Paint`, `Image` with `decode_png`, the sampling, the error kind `Decoding`, the mask polygon, the mask map, `Masks` with the seam classes and the code colours, the image fills through the mask map, and the tests and golden images of `escher-render`.
3. Command line: `render-masks` and the image items of `--colours` in `escher-cli`, with their snapshots and plain tests, the usage of `render-masks` and of an image fill in `README.md`, and Project Status and Repository Layout in `AGENTS.md` where they change.

### 5.23. M23: Wythoff Constructions

**Goal.** The user specifies a tiling by a Wythoff construction on a reflection triangle, such as the regular tiling `{7,3}` by heptagons with three at each vertex, and `escher` draws its faces with their own borders and fills. The owner moved this work from the roadmap into the proof of concept on 2026-11-23 (`DEC-68`), because the owner wants a way to specify such a tiling. Without it, `escher render` draws a border on the sides of the domain (RND-9 of `docs/design/render.md`), so its picture of `*732` shows triangles and not heptagons.

**Command line.** This command is an illustration only. `--wythoff` and `<marks>` stand for an option and a value whose name and syntax the design decides, and the marks that give `{7,3}` come from the sources of RP-6:

```sh
escher render '*732' --wythoff <marks> -o heptagons.png
```

**Scope.** The owner decided the scope of the first version on 2026-11-22, and it stands (`DEC-68`):

- The triangle groups `*pqr`.
- On or off marks on the mirrors.
- One colour or one image fill for each face type, so up to three fills.

Group colourings of the faces are not part of the first version. The work beyond it stays in the roadmap (Section 4.2 of `docs/design/roadmap.md`).

The design settles these open points as proposals:

- How the marks are given on the command line, and whether they extend `escher render` or form a command of their own.
- How the faces and their types are found from the fold, by the face rule that RP-6 sources or records as unverified.
- Where borders are drawn: on the edges of the faces, and whether the mirrors of the triangle still draw one.
- Which fill goes with which face type, and in what order the user gives the fills.
- What happens for marks that give no face of some type, and for a degenerate construction, such as every mark off.
- Whether the first version covers the rotation signatures `pqr`, and the Euclidean and spherical triangles as well as the hyperbolic ones.
- How the polygon masks and image fills of M22 apply to a face: whether `render-masks` writes a mask for each face type. A face reuses the mask polygon and the mask map of M22, where a regular face takes a flat regular polygon.
- What `escher info` reports for a construction, if anything.

**Design documents.** Extensions of `render.md` and `cli.md`, the labels that the design adds to Section 10.2 of `docs/design/architecture.md`, with its data flow of Section 4.1 where the faces add a step, and `colourings.md` through RP-6. A change to another crate extends that crate's design document too.

**Sources.** `colourings.md` 10 is the only research on the construction, and its Section 10.7 gives the status of each part. Section 10.1 gives the generator position of each Wythoff symbol, the generator point as the point on the inactive mirrors at equal distance from the active ones, and the edges from the generator point to its mirror images. Section 10.1 also gives the faces of each Wythoff symbol with their centres, from Coxeter, Longuet-Higgins, and Miller (1954) and Har'El (1993), for the sphere. Section 10.2 gives the meaning of a ring, the faces from subdiagrams, and the face rule at a corner of the triangle. Section 10.3 gives the symbols and faces of the family `(p q 2)` from the same papers for the sphere, the Euclidean tessellations from the 1954 paper, and the members of `(7 3 2)`, among them `{7,3}` as `3 | 7 2`, from Wikipedia only. Section 10.6 gives the per-pixel fold, the generator point by a linear solve in pywonderland's code, and the assignment of a folded point to a face in two shaders by one author, one spherical and one hyperbolic. No primary source read states the face rule for a hyperbolic triangle, and no paper or book states the assignment of a point to a face (items 10 and 11 of `colourings.md` 14.1). Beyond `colourings.md`, the research names uniform tilings only as precedents: pywonderland takes a triangle triple with its active mirrors (`prior-art.md` 6.7), and its shader folds per pixel (`rendering.md` 4.4). The symmetry group of `{p,q}` is the reflection group of the right triangle with the angles $\pi/2$, $\pi/p$, and $\pi/q$ (`colourings.md` 5.1), and its name `*pq2` is `UNV-37`.

**Unverified statements relied on.** This plan lists none of its own for M23. RP-6 records in `colourings.md` 14.1 each statement that no source establishes. The design branch adds a row to Section 7 for each such statement that the code relies on, and adds M23 to the rows that it shares with earlier milestones, such as `UNV-1` and `UNV-3` for the fold and for a sample on a wall.

**Tests and acceptance.** The design fixes the tests. These are proposals:

- A golden image of `{7,3}` from `*732`, and one golden image for each other form and geometry that the first version covers.
- A test that a sample inside a face of each type, at a point that the face rule of RP-6 places there, takes the fill of that type.
- A test that a construction with no face of a type, or a degenerate one, gives the behaviour that the design specifies, and, where that is a diagnostic, exit status 1 and no file written.
- A test that an image fill on a face follows the mask map of its face, as the test "Texture contract" of M22 does for a tile.
- The snapshots and plain tests of the new options and diagnostics, and of any line that `info` adds, in Section 8.1 of `docs/design/cli.md`.

**Dependencies and `deny.toml` work.** None expected.

**Risks.** RP-6 found primary sources for the face rule on the sphere only, and none for the classification, so a hyperbolic construction proceeds on unverified statements, and a wrong rule draws a wrong image without any error (Section 8). The borders of the faces are no longer the sides of the domain, so the border estimate of `DEC-28`, which RND-9 applies to the sides, may not carry over. The classification adds work to every sample, and its cost and accuracy at the corner orders up to 100 of `DEC-69` are unknown. A movable generator point and snubs lie close to this work and stay out of it.

**Split.** Left open for the design branch, which follows RP-6.

## 6. Documentation Obligations

Each milestone updates these documents in the same change as its code (Hard Rule 1 of `AGENTS.md`). The documents that mirror configuration, which Section 6 of `docs/design/architecture.md` lists, change in the same change as the first crates, dependencies, and recipes.

1. The design document of each crate that the milestone alters, before the code, with its assumptions, its open questions, and the list of PLAN-3. It records no status.
2. The research documents: the prerequisite readings, and any source read, in the same change as `source-requests.md`.
3. `AGENTS.md`: Project Status with its date, Repository Layout for each new top-level file or directory, document, script, or change to the common crate layout, and Technology Decisions when a decision takes force.
4. `README.md`: the crate table, the requirements, and the usage.
5. The tooling design: Section 8 for every change to `deny.toml`, Section 3.2 for a recipe, Section 4 for the nextest profiles, and Section 14.2 for environment variables.
6. The coding standard: the member list of Section 1.1, and Section 11 for new test rules.
7. `typos.toml` for new correct words.

## 7. Unverified Statements

The owner decided how the proof of concept treats a statement that no source establishes. The milestone proceeds and does not wait for the source. The design document that relies on the statement flags it, and the tests check it numerically against high-precision reference arithmetic. The statement stays tracked until a source or a machine-checked Lean proof closes it. Nothing in this table blocks the proof of concept. `docs/design/roadmap.md` holds the work that closes these statements, and the research location of each row is where the closing is recorded.

The table also keeps the rows that serve the roadmap items M14 to M18 (Section 4.1 of `docs/design/roadmap.md`), so that their labels stay stable. Only those items rely on `UNV-5`, `UNV-8`, `UNV-23`, and `UNV-24`, and the column "Milestones that use it" names each roadmap item as such.

| Id | Statement relied on | Research location | Milestones that use it | What closes it |
| --- | --- | --- | --- | --- |
| UNV-1 | The first-violated-wall fold terminates for compact reflection polygons, in a number of steps equal to the word length. On the sphere, at orders above the limit of `DEC-69`, points within $10^{-14}$ radians of a vertex of order 1000 stayed unconverged at the default cap, and near a vertex of order 999 points took more steps than the longest word of the group. Near the vertex of order 100 of `*(100)22`, the tests assert only that each fold returns within the cap (Section 6.7 of `docs/design/render.md`). No source covers every point, so the design relies on it only for the samples of the images that the tests render and the images of the centre | `rendering.md` 4.7 and 13.1 item 1, gap G5, `fundamental-domains.md` 13.3 item 6 | M3 on, including the lune of order 1 of `*` in M11, and M22 | Sections 1.6, 1.7, and 5.4 of [HUMPHREYS], or Lean. The sources that do not state it are listed in `rendering.md` 4.7 and gap G5 |
| UNV-2 | The same fold terminates for general side pairings and domains that are not convex. For the rotation domains of M6, a prototype outside the repository found that a fold with pairings that are plain `f64` products of two reflections can drift off the hyperboloid and fail to converge, as one fold of `(1000)32`, an order above the limit of `DEC-69`, did in the radius band $1 - 10^{-6}$. With accurately rounded products as pairings, every fold of the rotation fold sets of `escher-render` converges, but near some vertices of a rotation domain whose corner orders are all near 100 the fold can still fail to stop (Section 6.8 of `docs/design/render.md`). So the design relies on it only for the samples of the images that the tests render and the images of the centre, with accurately rounded products as pairings. For the wallpaper domains of M10, whose pairings include translations, half-turns, and glide reflections, a prototype outside the repository and the tests of `escher-render` found that every fold of their sets converged up to distance 65 from the centre. No termination argument exists for the kites of `4*2` and `3*3` (Section 6.10 of `docs/design/render.md`). For the spherical group domains of M11, whose pairings include rotary reflections, the tests of `escher-render` and a measurement outside the repository found no unconverged fold, also within $3 \times 10^{-17}$ radians of the vertices of `(100)*`, `(100)x`, and `2*(50)`, and no termination argument exists either (Section 6.11 of `docs/design/render.md`) | `rendering.md` 4.7 and 13.1 item 7, `fundamental-domains.md` 10.7 and 13.3 item 3, `symmhub.md` 17 item 5 | M6, M10, M11, M22, and the roadmap items M15 to M17 | A source, a change to Voight's rule, or Lean. The sources that do not state it are listed in `rendering.md` 4.7 and gap G5 |
| UNV-3 | Behaviour on or near a wall | `rendering.md` 13.1 item 2 | M3 on | Tests. No source is known |
| UNV-4 | Growth of the fold count, the reading of the exponents, and the limits of precision | `rendering.md` 13.1 items 5 and 6, `geometry-models.md` 15.1 item 10 | M3 defaults | The body of [FWW], and measurement |
| UNV-5 | The open points of the polygon theorem at ideal points: the test where a mirror ends at a cusp, the reading that two sides that meet only at an ideal vertex have the dihedral angle 0 and give no relator, and the element that fixes an ideal vertex. The four cases with reflections, glide cycles, mirror corners beside paired sides, and ideal reflection polygons are sourced in `fundamental-domains.md` 4.7, and the Euclidean plane and the sphere in its Sections 4.9 and 4.10, all under the hypotheses of those sections | `fundamental-domains.md` 4.7, 4.9, 4.10, 12, and 13.3 items 7 and 8, gap G1 | The roadmap item M18 | A source or Lean for each open point of gap G1 |
| UNV-6 | The cases that SymmHub's `edgeLength` sends to the inverse hyperbolic sine, the correspondence of its other cases with the formulas of `fundamental-domains.md` 9.3, and the pillow and hat formulas. The triangle, quadrilateral, pentagon, and hexagon formulas are sourced in `fundamental-domains.md` 9.3. [DEATON1993] states the principle of a generalized triangle with real and imaginary vertices and solves only the case with two imaginary vertices, with a sign that differs from case C of the code, and states no inverse hyperbolic sine case (`fundamental-domains.md` 8.15.1) | `fundamental-domains.md` 8.7, 8.8, 8.15.1, 9.3, and 13.3 items 1 and 13, gap G3, `symmhub.md` 6.4.2 | M9, and the roadmap items M15 and M16 | Formula Glossary 2.2 to 2.4 of [BUSER], or Lean |
| UNV-7 | SymmHub's order of the cuts, which fixes what each length means, its pairing of corners of order 2 without wrapping around, the corner pair as a mirror side, and its pairing of cone points of order 2 before the cut to a mirror. The nine cuts, the order in which they apply, the twelve atoms, and the statement that a twist arises only for three cuts are sourced in [DEATON1993]. The thesis does not say where a cut falls in a symbol. It numbers its two tables of cuts differently, so which three cuts carry a twist is a reader's reading (`fundamental-domains.md` 8.15) | Gap G4, `fundamental-domains.md` 8.15, 9.4.1, and 13.3 items 4, 5, and 12 | M9, and the roadmap items M15 and M16 | A project convention for the order of the cuts, and a source or Lean for the meaning of each length |
| UNV-8 | The cap generator is a glide reflection that, with the other side pairings, meets the hypotheses of the polygon theorems of `fundamental-domains.md` 4.9 and 4.10 | Gap G2, `fundamental-domains.md` 13.3 item 2 | The roadmap item M16. M10 does not rely on it, because its glide reflections are explicit matrices that `verify()` checks | [CHOI2012], [BUSER] |
| UNV-9 | The twist convention. For the Euclidean plane, `DEC-52` of `docs/design/architecture.md` settles it as a project convention: `skew` is a fraction of the first lattice vector | Gap G11 | The roadmap items M15 and M16. M10 settles the Euclidean part | A project convention |
| UNV-10 | Euclidean parameter counts: whether a count includes the scale, and the counts of the groups for which no source states one. The counts and the rule that are sourced are in `orbifold-notation.md` 10.2 and `fundamental-domains.md` 6.1. The parameters `aspect` and `skew` of M10 are those of the constructions, and the design asserts no count | Gap G7, `orbifold-notation.md` 12 item 7 | M4, M10 | [KLR], [SCOTT], or Lean |
| UNV-11 | Constructions of mixed signatures and of the spherical generators from the signature. Their presentations are sourced (`fundamental-domains.md` 9.7). For the Euclidean groups of M10, the guard of DOM-54 of `docs/design/domain.md` checks each build, and the wallpaper certificate of DOM-57, with the equal counts up to 16 colours beside it, checks each group numerically. Neither closes it. For the spherical groups of M11, the guard of DOM-60 checks each build, and the certificate of DOM-64, whose census of the reversing elements and the reflections is the only check that tells `n*` and `nx` from other groups of the same order, checks each group numerically | Gap G8, `fundamental-domains.md` 9.7, and for the Euclidean and spherical groups 9.5.2, 9.6.2, and 13.3 items 15 and 16 | M10, M11, and the roadmap item M15 | A source that constructs them, or Lean |
| UNV-12 | Trigonometry that rests on secondary or summarized sources: the Saccheri formulas, four of the six Lambert relations, the pentagon forms of `geometry-models.md` 15.1 items 12 and 14, and the claims that `geometry-models.md` 15.2 lists. The laws of triangles, the right-triangle relations, and the primary pentagon and hexagon laws are in `geometry-models.md` 10.2, 10.3, and 10.5, and the spherical law of cosines solved for a side, Theorem 2.5.4 of [RATCLIFFE], is in `geometry-models.md` 11.3 | `geometry-models.md` 10.5, 10.6, 15.1 items 12 and 14, and 15.2 | None. The construction of M9 does not use it (`DEC-46`) | Chapter 2 of [BUSER], or Lean |
| UNV-13 | The inverse and metric of the band, and the choice of map between the disk and the half-plane. The maps between the disk, the hyperboloid, and the Klein model are sourced in `geometry-models.md` 4.2 | `geometry-models.md` 4.3 and 5.1, `rendering.md` 13.1 item 14 | M12 | [HUB], and round-trip tests |
| UNV-14 | The distance to a geodesic in the forms that no source states, which include the hyperboloid form for unit normals and the disk form for a geodesic that is not a diameter, and the distance to a great circle. For a vector off the hyperboloid, the hyperboloid form reads the norm $\lvert\lvert\lvert y \rvert\rvert\rvert$ of equation (3.2.8) of [RATCLIFFE] as $\sqrt{-\langle y, y \rangle}$, which no research document defines. The sourced forms are in `geometry-models.md` 9.3 and `rendering.md` 6.2 | `geometry-models.md` 9.3 and 15.1 item 3, `rendering.md` 13.1 items 8, 10, and 26 | M3 tests, M13 | A source, or Lean |
| UNV-15 | The screen-distance estimate through the fold, analytic antialiasing, and the fade of tiles smaller than a pixel | `rendering.md` 7.1 and 13.1 items 19 and 20 | M3, and the roadmap item M14 | Measurement against supersampling |
| UNV-16 | The lift onto the ball, the view scale of the sphere, and the Phong formulas | `rendering.md` 13.1 items 11 and 12, `geometry-models.md` 11.6 | M5 | [PHONG75], [BLINN77] |
| UNV-17 | Coset colourings are perfect for every orbifold group, the composition order, completeness, and conjugacy. The composition order is the rule of GRP-38 of `docs/design/group.md`: the fold that applies the sides $i_1, \ldots, i_n$ in that order puts the sample in the tile $gF$ with $g = s_{i_1} \cdots s_{i_n}$, and the colour is the number of the right coset $S g^{-1}$, traced from coset 0 through the inverse column of each side in reverse order, which names the left coset $gS$. A prototype outside the repository found the rule perfect, with the exact permutation of each side, on every class that it tried, found the forward trace not perfect on every class that is not normal, and found the reverse trace without the inverse to give the mirror image on rotation domains (Sections 7.10 and 7.16 of `docs/design/group.md`). For conjugacy, the design lists every subgroup as its own colouring (`DEC-37`), after the prototype found that no sampled element of the group relates the pictures of two conjugate subgroups | `colourings.md` 4.2, 8.4, 8.7, and 14.1 items 1 to 3 | M8, M20, M22 | [DDL2009], [SCHWARZENBERGER1984], or Lean. [CBGS2008] defines colour types and states none of these (`colourings.md` 3.3 and 14.1 items 1 and 2) |
| UNV-18 | The parity colouring as the colouring by the two cosets of the orientation-preserving subgroup. That subgroup and its signature are sourced for non-Euclidean crystallographic (NEC) groups with a compact quotient (`colourings.md` 6) | `colourings.md` 6 and 14.2, `rendering.md` 4.6 | M3 | A source that links the colouring to the subgroup, or Lean |
| UNV-19 | The steps of Todd-Coxeter enumeration and of the low-index search that only implementations or the design state, which include the timing of the rule for one-letter relators, the order of the steps when the deduction stack overflows, and the distinct cyclic conjugates, as the column "Code-only part" of Section 7.3 of `docs/design/group.md` lists them, the safety of the first-in-class pruning on partial tables, which [HOLT2005] and [SIMS1994] state and argue without a proof, the bijection between subgroups and lenlex standard tables, which both state without a stand-alone proof, and the termination of SymPy's HLT code. For the low-index search, the parts that only the design states are the deductions pushed and found with every relator, the check of every relator at every coset of a complete table, the undo trail, the one run followed by runs by index, and the expansion of a class into its subgroups by relabelling from each coset, whose count of distinct tables rests on the bijection, as the column "Code-only part" of Section 7.11 of `docs/design/group.md` lists them. On a prototype outside the repository, the pruning of partial tables gave the same output as the test on complete tables only on 417 of 417 presentations, the check of complete tables never discarded a table with pushed deductions, and comparing an undefined entry as 0, which is the prototype's reading of the rule of `low_index`, lost classes on 389 of them (GRP-32 in Section 7.11 of `docs/design/group.md`). [HOLT2005] and [SIMS1994] state most of the steps of enumeration, and prove termination for finite index under their conditions on the strategy. [HOLT2005] states that a run that stops at its limit shows nothing about the index, and [SIMS1994] states only that such a run stops without a result (`colourings.md` 7.10 to 7.15). The absence of a computable bound is stated by papers and by [HOLT2005] without a proof. On item 23 of `colourings.md` 14.1, a prototype outside the repository gives evidence and no source: a reimplementation of SymPy's Felsch loop that scans every popped deduction with the lists of its caller never finished within 16384 cosets on the side presentations of small finite groups, such as that of `332` of order 12, while scanning with the lists of the letter of the entry, as Note III.22 of [HULPKE2010], p. 166 of [HOLT2005], and p. 234 of [SIMS1994] state, finished on each. Without a closing pass, the lists of the caller also returned tables in which a relator was not closed (Section 7.4 of `docs/design/group.md`) | `colourings.md` 7.8 to 7.17, 14.1 items 20 to 23 and 26, and 14.3 | M7, M8, M20 | A source that proves the pruning safe or the bijection, a source that discusses the variants of item 23 of `colourings.md` 14.1, or Lean |
| UNV-20 | Text conventions and the canonical form: the canonical string, the delimiter for orders of 10 or more, the spelling of infinity, and a `1` on its own. The equivalences and the omission of an order of 1 are sourced (`orbifold-notation.md` 3.5 and 4.1) | `orbifold-notation.md` 12 items 1 to 4 | M1, and M11 for the domain with no side of `1` | A project convention. [CBG] and [BH] define none of them |
| UNV-21 | Geometry and parameters of non-compact signatures with $\chi \ne 0$: whether a signature with $\infty$, $\chi < 0$, and a mirror boundary or a crosscap gives a group, the geometry when $\chi > 0$, the deformation dimension, and the reading of the existence condition for orientation-preserving groups as $\chi < 0$. That condition is sourced in the terms of [BEARDON] (`orbifold-notation.md` 7.1, `fundamental-domains.md` 5.5) | `orbifold-notation.md` 12 item 15, `fundamental-domains.md` 9.8 | M1, and the roadmap item M18 | A source, or Lean |
| UNV-22 | The Conway presentation for a boundary without corners in general, the dictionary between NEC signatures and orbifold signatures, and the signature of the rotation subgroup in orbifold notation for hyperbolic signatures and the spherical groups. The NEC signature, its presentation, and the rotation subgroup of a proper NEC group in NEC terms are sourced (`orbifold-notation.md` 9.2 and 9.3). On a prototype outside the repository, the Conway presentations of `*`, of `n*` for $n$ from 1 to 12, and of `2*`, read with one reflection for a boundary without corners, enumerated to $2/\chi$, which is evidence and no source | `orbifold-notation.md` 12 items 5, 12, and 14, gap G10 | M6, M7 | A re-reading of [CH] (RP-5), a source that states the dictionary, or Lean |
| UNV-23 | Whether the fold applies to a domain with an ideal vertex. The polygon theorem for such a domain is sourced (`fundamental-domains.md` 4.7) | `rendering.md` 13.1 item 3, `geometry-models.md` 7.6.5 | The roadmap item M18 | A source on the fold, or Lean |
| UNV-24 | MIT code inside a work under GPL-3.0-or-later | `symmhub.md` 17 item 1 | The roadmap item M15 | The licence texts, and the owner's judgement |
| UNV-25 | Correct rounding of the reference-arithmetic crate | `rust-ecosystem.md` 21 item 5 | M2 on | One comparison with MPFR |
| UNV-26 | Statements on the linear model in the conventions of Sections 5.1 and 5.2 of `docs/design/architecture.md`, with the time coordinate last and outward normals: the normal through two points, normals and the corner angle, a disk circle to a normal, and the Euclidean reflection as a matrix. [RATCLIFFE] states the cross product with the time coordinate first, the relation between normals and the angle with inward normals, and the Euclidean reflection as a map (`geometry-models.md` 2.4, 7.4, and 12) | `geometry-models.md` 2.4, 7.4, and 12 | M2, M4, M5, and the roadmap items M15 and M18 | A source in these conventions, or Lean |
| UNV-27 | Coordinates for the family, ranges, and default shape of polygons with prescribed angles. Existence, the admissible angles, and one polygon for each set of angles are sourced (`geometry-models.md` 10.1, `fundamental-domains.md` 9.4). [DEATON1993] states that each piece of its decomposition can be built because it has a positive angle defect, that the two sheets of the cut of `*2345` exist for any positive length, and that a cut meets the mirrors at right angles. It gives a short argument only, and states no coordinates, range, or default (`fundamental-domains.md` 8.15) | `geometry-models.md` 15.1 item 14, `fundamental-domains.md` 9.4.2 and 13.3 items 13 and 14 | M9 | A detailed reading of the proof in [TH], [BUSER] |
| UNV-28 | Properties of SymmHub's triangle: of its disk triangle, if it is used as an oracle, and in the Euclidean case the vertex coordinates of the triangle that `escher-domain` builds | `geometry-models.md` 15.1 item 5 | M2 tests, M4 | Tests only for the disk triangle. A source or Lean for the Euclidean coordinates |
| UNV-29 | The lune of `*nn`, the intersection of two hemispheres whose walls meet at the angle $\pi/n$, is a strict fundamental domain of the group of its two reflections. Theorem 6.4.3 of [DAVIS] covers convex polytopes, and a lune is not one in the sense of p. 75 of [DAVIS] | `fundamental-domains.md` 13.3 item 9 | M5, and M11 for the lune of order 1 of `*` | A source that proves it, or Lean |
| UNV-30 | The lune of the angle $2\pi/n$ with its one rotation pairing, including the hemisphere of `22`, is a fundamental domain of the cyclic group of order $n$. The lune of `*nn` is already `UNV-29` | `fundamental-domains.md` 13.3 item 10 | M6, and M11 for the lunes of `nx` and `x`, whose pairings are rotary reflections, which `UNV-34` holds | A source whose polygon theorem admits a lune, or Lean. The exhaustive certificates of `docs/design/domain.md` check each build numerically |
| UNV-31 | The double of a convex reflection polygon across one wall is convex, and its vertex cycles are the vertex of each end of the doubled side alone and each other vertex with its image, with the corner orders of the reflection polygon | `fundamental-domains.md` 13.3 item 11 | M6 | A source, or Lean. Checks 3, 4, and 5 of `verify()` test each build |
| UNV-32 | The relators of a double as words: one generator for each side, standing for the inverse of its pairing as Section 5.3 of `docs/design/architecture.md` states, the pair relator $s_i s_p$ for each pair, and for each vertex cycle the generators of the sides that the walk of the cycle applies, in walk order, to the power of the order of the cycle. The polygon theorems state that the pairing relators and the cycle relators present the group (`fundamental-domains.md` 4.1, 4.2, 4.8 item 7, and 4.9), and SymmHub derives its relators in this way in code (`symmhub.md` 14.2), but no source read writes the cycle relators as words in this convention. Every vertex cycle of the domains of M6 visits one or two sides, so a reversed word or the opposite convention passes every numeric test on them. The walk of DOM-55 of `docs/design/domain.md` gives the wallpaper domains of M10 cycles of four sides, such as that of `o`. A measurement found that on every cycle of three or more sides of `**`, `xx`, `*x`, and `22*` a reversed word and the opposite convention also give the residual 0, so those cycles do not fix the convention either (DOM-36 of `docs/design/domain.md`) | `fundamental-domains.md` 4.8 item 7, `symmhub.md` 14.2 | M7, M8, M10, M11 for the chains of four sides of the spherical kites, and M20 | A source that states the cycle relators as words in this convention, or Lean. The tests "Relators evaluate" and "Long cycle word" of `docs/design/domain.md` check it numerically |
| UNV-33 | The domains and pairings of the wallpaper domains of M10 that no source states: the pairings of `**`, `*x`, `22x`, `22*`, and `2*22`, the pairings of the kites of `4*2` and `3*3` as formulas, and the domains of `o` and `2222` sheared by `skew`, with the reading that the range of `skew` reaches every lattice (DOM-53 of `docs/design/domain.md`). The parallelogram of `o` and the square of `xx` without a skew, the shapes of the kites, and `2222` without a skew are sourced (p. 250 of [CH], p. 399 of [RATCLIFFE], pp. 7 and 12 of [VONGAGERN2009], and Example 13.1.4 of [TH]) | `fundamental-domains.md` 9.5.2 and 13.3 items 17 and 18 | M10 | A source that states them, or Lean. Until then the guard of DOM-54 of `docs/design/domain.md`, which returns a domain only when it passes `verify()`, and the wallpaper certificate of DOM-57 bound the reliance |
| UNV-34 | The domains and pairings of the spherical group domains of M11 that no source states: the kites of `3*2` and `2*n` on the doubles of `432` and `(2n)22`, the kite of `n*` on the double of `n22` with its split mirror side, the lune of `nx` with its pairings by rotary reflections, the lunes of order 1 of `*` and `x`, and the domain with no side of `1`. Also the walk of the vertex cycles of DOM-55 on a lune whose pairings reverse orientation, and the values of the census of DOM-64 of `docs/design/domain.md`, such as the number of reflections of each group and whether it holds minus the identity, which a prototype outside the repository computed and no research document states. The orbifolds of `3*2` and `*` and the presentation of `n*` are sourced (p. 187 of [CBG], p. 248 of [CH], and p. 130 of [CBG]) | `fundamental-domains.md` 9.6.2 and 13.3 items 19 and 20, `geometry-models.md` 15.1 item 33 | M11 | A source that states them, or Lean. Until then the guard of DOM-60 of `docs/design/domain.md`, which returns every new domain except that of `*` only when it passes `verify()`, and the certificate of DOM-64, with its order, its census, and its exactly-once tiling, bound the reliance |
| UNV-35 | In a coset colouring, under the colour rule of `UNV-17`, two tiles that share a side have the same colour exactly when the generator of that side lies in the subgroup of the colouring, which in the coset table is the generator fixing the base coset, so that the criterion of `DEC-65` can be tested on the side generators and the base coset of each listed subgroup (GRP-42 of `docs/design/group.md`) | `colourings.md` 14.1 item 28 | M20 | A source that states it, or Lean. The test "Valid neighbours" of `docs/design/render.md` checks it on samples |
| UNV-36 | A domain has at most one valid colouring with 2 colours, so that the alias `parity` names one colouring (Section 4.6 of `docs/design/cli.md`) | `colourings.md` 14.1 item 29 | M20 | A source that states it, or Lean. The test "One valid 2-colouring" of `docs/design/group.md` checks it on its inputs |
| UNV-37 | The regular tiling $\{p, q\}$ with $p, q \ge 3$ has the symmetry group that `*pq2` names, with the corner of order $p$ at the centre of a tile and the corner of order $q$ at a vertex, its group of orientation-preserving symmetries is the group that `pq2` names, and its dual is $\{q, p\}$. For the sphere and the Euclidean plane, `colourings.md` 10.3 states the corners and the dual, and no source read states them for the hyperbolic plane. No source states the naming in any geometry. A source states the symmetry group as the reflection group of the right triangle with the angles $\pi/2$, $\pi/p$, and $\pi/q$, which is also the group of $\{q, p\}$, and the rotation group as a subgroup of index 2 (`colourings.md` 5.1 and 5.4) | `orbifold-notation.md` 12 item 16 | M21 | A source that states it in orbifold notation, or Lean. No test can check the name of a tiling. The check of the angles of `escher-domain` checks that the domain of `*pq2` has the angles of that triangle, and the tests of Section 8.1 of `docs/design/cli.md` pin which signature gets which symbols |

## 8. Security Considerations

This document defines no component, so the threat model is that of the tool it plans. The attacker is assumed to control the input of `escher`: the signature string, every option value, every `name=value` setting, the output path, and from M22 the image files of an image fill. Every milestone that adds such an input states, in the Security Considerations of its design document, the limits that Section 5.9 of `docs/design/architecture.md` requires: the maximum order, the signature length, the image size, the fold cap, and the search budget. Without these limits a crafted signature or option exhausts memory or time, for example through the colouring search of M8, whose cost grows exponentially with the index.

Before M22, the output path is the only file that `escher` writes, and no command reads a file supplied by the user. M22 changes both: `render-masks` writes one file for each colour, and `render` reads the image files of its image fills. Section 13 of `docs/design/render.md` and Section 12 of `docs/design/cli.md` state the limits on the size of each image that it reads, and which files `render-masks` may write. Every dependency enters through the milestone that first needs it, with the licence and advisory checks of cargo-deny (Section 8 of the tooling design). A native build, such as `rug`, would add code that runs at build time, which is one reason why `DEC-3` chooses the pure-Rust astro-float, used through its crate `astro-float-num`. Code ported from SymmHub, which only roadmap items plan, carries its MIT notice under `DEC-18`, and `UNV-24` tracks the open licence question.

Attacks on the repository itself, such as a malicious change to a document or a compromised CI runner, are out of scope, because `AGENTS.md` and the tooling design govern the repository and its workflow. The residual risk is that a limit chosen in a design document is too high for the machine that runs the tool, that a crafted image file exhausts memory in the decoder, and that an unverified statement of Section 7 makes the tool draw a wrong image without any error.

## 9. Operational Considerations

The crates declare no Cargo feature during the proof of concept, as Section 3 of `docs/design/architecture.md` states. M3 adds the environment variable of the recipe that updates golden images, and documents it with its default in Section 14.2 of the tooling design. The only supported platform of the proof of concept is Linux, and rendering runs on the CPU only. The minimum supported Rust version (MSRV) is the one that the workspace declares, and `check-msrv` verifies it.

The plan has these failure modes:

| Failure | Handling |
| --- | --- |
| A milestone grows beyond a design branch and a few code branches | It splits along its Split field |
| A research prerequisite finds no construction | The milestone proceeds on the unverified statements of Section 7, or the owner decides to move the work to the roadmap (`DEC-19`, `DEC-64`) |
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
- [DEATON1993] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [FWW] The entry in Section 16.2 of `docs/research/geometry-models.md`.
- [HOLT2005] The entry in Section 15 of `docs/research/colourings.md`.
- [HUB] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [HULPKE2010] The entry in Section 15 of `docs/research/colourings.md`.
- [HUMPHREYS] The entry in Section 14 of `docs/research/rendering.md`.
- [KC24] The entry in Section 16.1 of `docs/research/geometry-models.md`. `rendering.md` cites the same paper as [KOPCZ24].
- [KLR] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [PHONG75] The entry in Section 16.4 of `docs/research/geometry-models.md`.
- [RATCLIFFE] The entry in Section 14.1 of `docs/research/fundamental-domains.md`, which cites the first edition. `geometry-models.md` cites the same edition as [RAT].
- [SCHWARZENBERGER1984] The entry in Section 15 of `docs/research/colourings.md`.
- [SCOTT] The entry in Section 13.4 of `docs/research/orbifold-notation.md`.
- [SIMS1994] The entry in Section 15 of `docs/research/colourings.md`.
- [TH] The entry in Section 13.1 of `docs/research/orbifold-notation.md`. `fundamental-domains.md` cites the same notes as [THURSTON2002].
- [VONGAGERN2009] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| PLAN-1 | The design branch of a milestone merges before any of its code branches | 3.1 |
| PLAN-2 | The code of a milestone starts only after its research prerequisite has merged | 3.1 |
| PLAN-3 | Each crate design document lists the unverified statements its code relies on, one row per label with the section that relies on it, and no evidence tables | 3.1 |
| PLAN-4 | A milestone is accepted only when every test it names passes in `just check` and CI | 3.1 |
