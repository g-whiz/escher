# Proof-of-Concept Milestone Plan

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-24 |
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

This document covers the workflow of a milestone, the owner's staging, the research prerequisites, the milestones `M1` to `M13` and `M19` to `M27`, the documentation obligations of each milestone, and the unverified statements that the milestones rely on. The milestones `M14` to `M18`, which the owner moved to the roadmap (`DEC-64`), keep their labels and a subsection each that points to the roadmap.

### 1.5. Non-Goals

Each fact has one owner, and this document cites the owner instead of restating the fact:

- `docs/design/architecture.md` owns the crates, the data flow, the conventions that cross crates, the testing strategy with the definition of a golden image, performance, numerical robustness, and the decisions. Section 10.1 of it records the owner's decisions, and Section 10.2 the labelled defaults `DEC-1` to `DEC-145`.
- `docs/design/roadmap.md` owns the phases after the proof of concept, the milestones `M14` to `M18` that the owner moved there, the work that closes requested sources, and the Lean workstream.
- The subject-matter references under `docs/research/` own the mathematics.
- `AGENTS.md` owns the hard rules, the development workflow, and Project Status.

This document records no progress. It states the milestones as a plan, and Project Status in `AGENTS.md` is the only place that records which milestone has merged (Section 3.6 of `docs/writing-documentation.md`).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms that Section 2.2 of `docs/writing-documentation.md` defines, and calls that document the writing guide. It calls `docs/rust-coding-standards.md` the coding standard and `docs/design/tooling.md` the tooling design. It adds these terms:

- milestone: A unit of the plan with one goal, labelled `M1` to `M27`, delivered by one design branch and a few code branches. The labels `M14` to `M18` name work that the owner moved to the roadmap. The planning documents of Section 3.3 carry the label `M0` and are not delivered this way.
- design branch: A branch named `feature/<name>-design` that changes only documents. It writes or extends the design documents of a milestone, and the research documents where a research prerequisite applies.
- code branch: A branch in its own worktree that changes code together with its tests and the document updates that Hard Rule 1 of `AGENTS.md` requires.
- research prerequisite: A design branch, labelled `RP-1` to `RP-8`, that adds to a research document what a milestone needs before its code can start (Section 4).
- unverified statement: A statement that a milestone relies on and that no cited source establishes, labelled `UNV-1` to `UNV-56` (Section 7).
- arrangement: A subgroup of the symmetry group of a tiling under which no element other than the identity maps a polygon onto itself, which puts one picture on each orbit of the polygons, as Section 5.30 of `docs/design/render.md` defines it for the regular tilings.
- picture: One orbit of the polygons under an arrangement, which shows one painted image, named A, B, C, and so on (Section 5.30 of `docs/design/render.md`).
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

The other milestones carry the owner's decisions on colourings (M7, M8, M20), models (M12), borders (M13), Schlafli symbols (M21), polygon masks and image fills (M22), Wythoff constructions (M23), whole-face image fills (M24, M25, and M26), and renders centred on a corner (M27). The parser accepts signatures with infinite orders from M1, and rendering the friezes and the triangle groups with ideal corners is the roadmap item M18. M19 checks that every signature of the set renders.

### 3.3. Planning Documents

The planning documents, labelled M0, are the precondition of M1 and have no code to implement. They are `docs/design/architecture.md`, this plan, and `docs/design/roadmap.md`, which live in `docs/design/` as plans without status (`DEC-2`), together with the updates to `AGENTS.md` (Project Status and Repository Layout) and to `README.md`. They are accepted when `just check` passes and when they call every crate planned.

### 3.4. Milestone Overview

The owner set the order of the remaining work on 2026-11-22 and placed M23 and M24 in it on 2026-11-23 (`DEC-63`). A cleanup comes first and is not a milestone. Then come M20, M12 and M13, M21, M22, M23, and M24, then M25, M27, and M26, and the completion audit M19 comes last. On 2026-11-23 the owner placed M25 and M26 last before the audit (`DEC-114`), and M27 sits between them. The order of M22 and M23 may still change, and the owner sets it. The split of the work after M24 into M25 and M26, and their order, are a proposal (`DEC-121`). On 2026-11-24 the owner added M27, a small milestone whose design follows the design of M25 and whose code is built alongside the code of M25, independent of it and of M26 (`DEC-133`). The table lists the milestones in the planned order. The column "Depends on" names the milestones and research prerequisites whose results a milestone uses, as Section 5 gives them. The milestones M14 to M18 are not in the table, because the owner moved them to the roadmap (Section 3.2).

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
| M23 | Wythoff constructions | `--wythoff` of `render`: the Wythoff tilings of every reflection triangle from on or off marks on its mirrors, with one colour for each face type | `escher-geometry`, `escher-render`, `escher-cli` | M2, M4, M5, M22, RP-6 |
| M24 | Whole-face image fills | `info --wythoff` lists the arrangements of a regular tiling, `render-masks --wythoff --arrangement` writes its face mask, and `render --wythoff --arrangement` puts one picture on every polygon, turned or mirrored | `escher-group`, `escher-render`, `escher-cli` | M8, M20, M22, M23, RP-7 |
| M25 | Whole-face fills with several pictures of one polygon | `info --wythoff` also lists the arrangements with several pictures, `render-masks --arrangement` writes one face mask for each picture, and `render --arrangement` takes one colour or image for each picture, such as for squares five to a vertex. No new option | `escher-render`, `escher-cli` | M24 |
| M27 | Renders centred on a corner | `--centre` of `render`: the picture centred on a corner of the domain, given by its position in the order in which `info` prints the corners | `escher-render`, `escher-cli` | M11, M12, M22, M23, M24 |
| M26 | Whole-face fills of tilings with several kinds of polygon | `info --wythoff`, `render-masks --arrangement`, and `render --arrangement` also on a Wythoff construction with two or three face types, with one arrangement for every kind and one or more pictures of each kind. No new option | `escher-group`, `escher-render`, `escher-cli` | M24, M25, RP-8 |
| M19 | Completion audit | None | All six crates of the proof of concept | M11, M12, M13, M20, M21, M22, M23, M24, M25, M26, M27 |

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
| RP-7 | The statements about the subgroups of the symmetry group of a tiling under which no element other than the identity maps a polygon of the tiling onto itself: the orbits of the polygons under such a subgroup, whether one exists with one orbit for each kind of polygon, and the relation of such subgroups to the coset tables that the colouring search of M8 and M20 already produces | `colourings.md` 5.9 and 14, and `source-requests.md` | The sources that `colourings.md` 4.1, 5, and 7.17 cite, among them [FRETTLOH2008], with [CBGS2008] of `colourings.md` 3.3, and any further primary source that the research finds. A source that cannot be read goes into `source-requests.md` | M24 |
| RP-8 | The stabilizer of each face of a Wythoff construction with more than one face type on a reflection triangle, and the subgroups under which no element other than the identity maps a face of any type onto itself: their orbits on each face type, whether one exists with one orbit for each face type, and the composition of a face from copies of the triangle (`colourings.md` 14.1 items 4 and 5) | `colourings.md` 5.10 and 14, and `source-requests.md` | The sources that `colourings.md` 5.9 cites, among them [CBGS2008] on isohedral and Heesch types, [FRETTLOH2008] on the Laves tilings, and the sources on Wythoff tilings of `colourings.md` 10. A source that cannot be read goes into `source-requests.md` | M26 |

RP-3, apart from its sheet and slice part, and RP-4 serve work that the owner moved to the roadmap (Section 4.1 of `docs/design/roadmap.md`), so no milestone of this plan waits on them. RP-6 precedes M23, because M23 draws faces by a face rule for each pattern of marks and by a per-pixel classification of a point into a face, which `colourings.md` 10 sources or records as unverified first. RP-7 precedes the design of M24, because the planning's reading of how M24 turns a picture on each polygon rests on such subgroups (Section 5.24), which `colourings.md` sources or records as unverified first. RP-8 precedes the design of M26, because Section 5.9 of `colourings.md` treats the polygons of the regular tilings only: Yuncken's theorem concerns $\{p, q\}$, and Frettloh's lemmas cover the regular and Laves tilings only. M25 needs no research prerequisite, because statements 1 to 3 of `colourings.md` 5.9.6, which `UNV-41` to `UNV-43` record, are stated for any subgroup of finite index, with any number of orbits on one kind of polygon, and the polygons of M25 are those of M24. The design branches of M20 and M21 add to the subject-matter references the sources for the mathematics that they rely on, or record it as unverified there (Hard Rule 14 of `AGENTS.md`).

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

The mask polygon and the mask map stay private (`DEC-88`). M24 plans to reuse the masks for the polygons of a tiling (Section 5.24).

Out of scope: research into external tools, which the owner ruled out (Section 1 of `docs/design/roadmap.md`).

**Design documents.** Sections 5.18 to 5.23 of `render.md` with its other sections that the masks and image fills touch, Sections 4.6, 4.8, and 6.10 of `cli.md` with its other sections that the command touches, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-84` to `DEC-98`, and its crate table and Security Considerations.

**Sources.** The per-pixel fold, which gives the point in the domain (`rendering.md` 4.1). The colour of the neighbour of the domain across a side comes from the coset table by the rule of GRP-38 of `docs/design/group.md` (`UNV-17`), and the pairs of colours that occur across a side from a closure of the table that no source states. The Klein positions and the walls as straight chords in the Klein model (`geometry-models.md` 3.1 and 4.2). SymmHub's sampling of an image in a pattern box, and the mirrored texture lookup of Nelson and Zucker, as precedents (`rendering.md` 10).

**Unverified statements relied on.** `UNV-1` and `UNV-2` for the fold, and `UNV-17` for the colour of the neighbour of the domain across a side, on which the seam classes of the masks rest. The closure that gives the pairs of colours across a side, the marks of the seam classes, the convexity and the equal edges of the mask polygon, the chart of the sphere, the centres and rays of the mask map, the bijection of the mask map, and the distinct code colours are unverified statements of the render design, which its tests check (Section 10 of `docs/design/render.md`). The milestone adds no row to Section 7.

**Tests and acceptance.** The tests "Polygon rules", "No polygon", "Round trip", "Edge fraction", "Euclidean identity", "Mask content", "Seam classes", "Code colours", "Flat image", "Texture contract", "Mirrored", "Seams", "Same texture in every model", and "Alpha and fallback", the decoding and failure tests, and the golden images of a mask and of image fills, of Sections 8.1, 8.2, and 8.4 of `docs/design/render.md`. The snapshots and plain tests of `render-masks` and of the image items in Section 8.1 of `docs/design/cli.md`. They include a missing, a malformed, and an oversized image file, each with exit status 1 and no file written.

**Dependencies and `deny.toml` work.** None new. `escher-render` decodes with its existing dependency `png` (Section 9 of `docs/design/render.md`).

**Risks.** The image file is the first input file that `escher` reads (Section 8). A file that blocks or never ends holds `render`, because no limit applies to the length of the file (Section 12 of `docs/design/cli.md`). Bilinear sampling with no level of detail aliases an image where a tile is much smaller than the image. A render with other settings than the masks keeps the colours but can change the mask polygon, so the images no longer match it. The mask map distorts the picture inside a tile, more so for a thin flat triangle, such as that of `*(100)(100)2`. With no budget, the images of one render can take up to $2^{28}$ bytes of texels each (`DEC-92`).

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Render library: `Paint`, `Image` with `decode_png`, the sampling, the error kind `Decoding`, the mask polygon, the mask map, `Masks` with the seam classes and the code colours, the image fills through the mask map, and the tests and golden images of `escher-render`.
3. Command line: `render-masks` and the image items of `--colours` in `escher-cli`, with their snapshots and plain tests, the usage of `render-masks` and of an image fill in `README.md`, and Project Status and Repository Layout in `AGENTS.md` where they change.

### 5.23. M23: Wythoff Constructions

**Goal.** The user specifies a tiling by a Wythoff construction on a reflection triangle, such as the regular tiling `{7,3}` by heptagons with three at each vertex, and `escher` draws its faces with their own borders and colours. The owner moved this work from the roadmap into the proof of concept on 2026-11-23 (`DEC-68`), because the owner wants a way to specify such a tiling. Without it, `escher render` draws a border on the sides of the domain (RND-9 of `docs/design/render.md`), so its picture of `*732` shows triangles and not heptagons.

**Command line.** These commands illustrate the milestone:

```sh
escher render '*732' --wythoff 010 -o heptagons.png
escher render '*732' --wythoff 111 --colours '#e9c46a,#2a9d8f,#913671' -o omnitruncated.png
```

- The first writes the regular tiling `{7,3}`, heptagons with three at each vertex, in one colour.
- The second writes the tiling `4.6.14` by 14-gons, hexagons, and squares, in the order of the corners of orders 7, 3, and 2.

**Scope.** The owner decided the scope of the first version on 2026-11-22 and narrowed it on 2026-11-23 (`DEC-68`):

- The triangle groups `*pqr`, which the owner widened on 2026-11-23 to every reflection triangle in the three geometries (`DEC-107`).
- On or off marks on the mirrors.
- One solid colour for each face type, so up to three colours. On 2026-11-23 the owner dropped the image fills of the faces from this milestone, and the whole-face image fills are M24 (Section 5.24, `DEC-110`).

Group colourings of the faces are not part of the first version. The work beyond it stays in the roadmap (Section 4.2 of `docs/design/roadmap.md`).

The design branch settled the open points of the planning as proposals. On 2026-11-23 the owner accepted the marks of `DEC-99`, the number of fills of `DEC-105`, and the scope of `DEC-107`, and has decided none of the others (Section 10.1 of `docs/design/architecture.md`):

- The marks are an option `--wythoff` of `render`, with one digit `0` or `1` for each corner in the order of the canonical signature, and `1` turns on the mirror opposite that corner, so `010` gives $\{p, q\}$ and `100` its dual on `*pq2` (`DEC-99`, `DEC-100`, CLI-62 of `docs/design/cli.md`).
- The generator point is a vertex, the meeting of the inactive side with a wall of equal distance, or the meeting of two such walls, through a new function `wall_meet` of `escher-geometry` (`DEC-101`, Section 5.7 of `docs/design/geometry.md`).
- Each face is centred at a corner, a face type is a corner with a face by the face rule, digons drop out, and marks that give no face are refused with a diagnostic, which covers every mark off and a few markings of `*n22` (`DEC-102`, Section 5.24 of `docs/design/render.md`).
- A folded sample takes the face of the first corner that lies on its side of the perpendicular of each active side through that corner (`DEC-103`, Section 5.25 of `docs/design/render.md`).
- The borders lie on the perpendiculars from the generator point to the active mirrors, in both border units, and the mirrors draw none (`DEC-104`, Section 5.26 of `docs/design/render.md`).
- `--colours` gives one colour for each face type, in the order of the corners and exactly as many as there are face types, an item that is not a colour, an image item included, is refused with the existing diagnostic of a bad colour, and `--wythoff` with `--colouring` is refused (`DEC-105`, CLI-63 of `docs/design/cli.md`).
- `Wythoff::new` reports a numerical failure as an error of the kind `Numerical`, as `escher-domain` does, and no accepted construction is expected to reach it (step 8 of Section 5.24 of `docs/design/render.md`).
- The first version covers every reflection triangle in the three geometries, `*pqr`, `*632`, `*442`, `*333`, `*332`, `*432`, `*532`, and `*n22`, and refuses every other signature, including the rotation signatures (`DEC-107`).
- `escher info` prints nothing new (`DEC-108`).

**Design documents.** Sections 5.24 to 5.28 of `render.md` with its other sections that the Wythoff fill touches, Sections 4.5, 4.6, and 6.8 of `cli.md`, Section 5.7 of `geometry.md`, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-99` to `DEC-108`, of which `DEC-106` is withdrawn, and item 10 of Section 14.1 of `colourings.md`.

**Sources.** `colourings.md` 10 is the only research on the construction, and its Section 10.7 gives the status of each part. Section 10.1 gives the generator position of each Wythoff symbol, the generator point as the point on the inactive mirrors at equal distance from the active ones, and the edges from the generator point to its mirror images. Section 10.1 also gives the faces of each Wythoff symbol with their centres, from Coxeter, Longuet-Higgins, and Miller (1954) and Har'El (1993), for the sphere. Section 10.2 gives the meaning of a ring, the faces from subdiagrams, and the face rule at a corner of the triangle. Section 10.3 gives the symbols and faces of the family `(p q 2)` from the same papers for the sphere, the Euclidean tessellations from the 1954 paper, and the members of `(7 3 2)`, among them `{7,3}` as `3 | 7 2`, from Wikipedia only. Section 10.6 gives the per-pixel fold, the generator point by a linear solve in pywonderland's code, and the assignment of a folded point to a face in two shaders by one author, one spherical and one hyperbolic. No primary source read states the face rule for a hyperbolic triangle, and no paper or book states the assignment of a point to a face (items 10 and 11 of `colourings.md` 14.1). Beyond `colourings.md`, the research names uniform tilings only as precedents: pywonderland takes a triangle triple with its active mirrors (`prior-art.md` 6.7), and its shader folds per pixel (`rendering.md` 4.4). The symmetry group of `{p,q}` is the reflection group of the right triangle with the angles $\pi/2$, $\pi/p$, and $\pi/q$ (`colourings.md` 5.1), and its name `*pq2` is `UNV-37`.

**Unverified statements relied on.** `UNV-38` for the face rule, the edges, and the markings of $\{p, q\}$ and its dual on a Euclidean or hyperbolic triangle, `UNV-39` for the face of a folded point, and `UNV-40` for the generator point outside the sphere. The fold rests on `UNV-1` and `UNV-3`, the borders on `UNV-15` and, with the intrinsic unit, `UNV-14`, and the markings of the regular tiling of a hyperbolic `*pq2` also on `UNV-37`. The computation of the generator point, the corner orders read from the corner angles, and the absence of a numerical failure on an accepted construction are unverified statements of the render design, which its tests check (Section 10 of `docs/design/render.md`), and `wall_meet` is one of the geometry design (Section 12 of `docs/design/geometry.md`).

**Tests and acceptance.** The tests of the layer "Wythoff constructions", the Wythoff rows of Section 8.2, and the golden images of Wythoff fills of Section 8.4 of `docs/design/render.md`, among them `wy-7-3-2-010.png`, the regular tiling `{7,3}`, and one image in each geometry. The test "Walls meet" of Section 10.1 of `docs/design/geometry.md`. The snapshots and plain tests of `--wythoff` in Section 8.1 of `docs/design/cli.md`, which include `--wythoff 000`, a signature that is not a reflection triangle, `--wythoff` with `--colouring`, the wrong number of colours, and an image item, each with exit status 1 and no file written.

**Dependencies and `deny.toml` work.** None new.

**Risks.** RP-6 found primary sources for the face rule on the sphere only, and none for the face of a folded point, so a Euclidean or hyperbolic construction proceeds on unverified statements, and a wrong rule draws a wrong image without any error (Section 8). The border estimate of `DEC-28` is applied to walls that are not sides of the domain, which its oracles check only on the images of the tests. Each sample of a Wythoff fill adds up to six sign tests and up to two border tests, and the accuracy at the corner orders up to 100 of `DEC-69` is measured only on the Wythoff triangle set. A movable generator point and snubs stay out of this work.

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Libraries: `wall_meet` of `escher-geometry` with its tests and its rows of accuracy, and `Wythoff`, `WythoffFill`, `Fill::Wythoff`, the face of a sample, the borders, the colours of the face types, the error kind `Numerical`, and the tests and golden images of `escher-render`.
3. Command line: `--wythoff` of `render` in `escher-cli`, with its snapshots and plain tests, the usage of `--wythoff` in `README.md`, and Project Status in `AGENTS.md`.

### 5.24. M24: Whole-Face Image Fills

**Goal.** The user chooses a tiling by polygons, such as heptagons three to a vertex or regular hexagons, and gets a mask for each kind of polygon: a flat regular heptagon, hexagon, square, and so on. The user paints a whole picture on that polygon, with no forced symmetry inside it. The tool puts that picture on every polygon of the tiling and turns each copy as needed. Coloured borders on the mask show which edge meets which, so the user can check in an image editor that the picture continues across the borders. The owner confirmed this goal on 2026-11-23 and placed the milestone after M23 and before M19 (`DEC-110`).

**Command line.** These commands illustrate the milestone. The first lists the arrangements of the heptagons of `*732`, the second writes the face mask of the first, and the third puts the picture that the user painted over that mask on every heptagon:

```sh
escher info '*732' --wythoff 010
escher render-masks '*732' --wythoff 010 --arrangement 14a -o /tmp/masks/heptagon.png
escher render '*732' --wythoff 010 --arrangement 14a --colours 'image:/tmp/masks/heptagon-picture.png' -o heptagons.png
```

**Scope.** The owner decided these details on 2026-11-23:

- The flow: `info` lists the arrangements of a tiling by id, as it lists colourings, a mask command writes one flat regular polygon, white on black, with coloured edge borders, in which edges that meet share a colour, a black mark shows direction, and an edge that meets itself gets a colour of its own, the user paints anything on it, and `render` puts the picture on every polygon, turned or mirrored as the arrangement says (`DEC-111`).
- The arrangements with mirror-image neighbours are offered, not only those of turned copies (`DEC-112`).
- The first scope is the regular tilings (`DEC-113`).
- The tilings that need more than one picture of the same polygon, and the tilings with more than one kind of polygon, are the last milestones of the proof of concept, after M24 and before M19 (`DEC-114`).
- The flat polygon is turned so that what is painted on it appears the same way up on the reference polygon in the default view (`DEC-122`, Section 5.31 of `docs/design/render.md`).
- The face mask marks edge 0 with a short black tick at its midpoint, so that the edge numbers that `info` prints can be placed on the mask (`DEC-123`, Section 5.32 of `docs/design/render.md`).

The design branch settled the open points of the planning as proposals, and the owner has decided none of them:

- An arrangement is a subgroup in whose coset table no element of the stabilizer of a polygon other than the identity fixes a coset, which RP-7 records as `UNV-41` and `UNV-42`. With the index $2n$ of a polygon with $n$ sides, it has one orbit, so one picture (`UNV-43`).
- `escher-group` lists one arrangement for each class of subgroups with a new function `free_classes_of_index`, which takes the words of the stabilizer, with ids in the form of a colouring id whose number is the index, such as `14a` (`DEC-115`, GRP-45 of `docs/design/group.md`).
- The first scope is the regular tilings on which one picture suffices, `010` and `100` on `*pq2` with $p \ge q \ge 3$, and every other tiling is refused, among them the constructions with one face type on `*n22`, which `DEC-81`, itself a proposal, keeps out of the regular tilings. M25 and M26 lift the refusals of the tilings that need several pictures or have several kinds of polygon (`DEC-120`, Section 5.29 of `docs/design/render.md`).
- The edge pairing comes from the coset table, with a flag for a turned or a mirrored neighbour (Section 5.30 of `docs/design/render.md`).
- A sample shows the picture at the point of the flat polygon that the mask map of M22 onto one triangle of the polygon, followed by the flat symmetry of the stabilizer element of its coset, gives (`DEC-116`, Section 5.31 of `docs/design/render.md`).
- The flat polygon is a regular $n$-gon with the edges numbered counterclockwise from edge 0, the edge that holds the half edge of the domain (`DEC-117`). The face mask is a polygon mask of M22 of that polygon, one code colour for each pair of edges and for each edge paired with itself, with the marks of M22, written by `render-masks` to the path of `--output` (`DEC-118`).
- `info --wythoff <MARKS>` prints the arrangement lines, `--arrangement <ID>` on `render` and `render-masks` takes the id, only with `--wythoff`, and `--colours` then takes one colour or one image item (`DEC-119`, Sections 4.6, 4.8, and 5.10 of `docs/design/cli.md`).

The whole-face fills of the fundamental tilings of the signatures that are not reflection triangles are not part of M24, M25, or M26.

**Design documents.** Sections 5.29 to 5.33 of `render.md` with its other sections that the whole-face fill touches, Sections 4.3, 4.5, 4.6, 4.8, 5.10, 6.8, and 6.10 of `cli.md`, Section 7.19 of `group.md`, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-111` to `DEC-123`, and items 42 to 44 of Section 14.1 of `colourings.md`.

**Sources.** `colourings.md` 4.1 gives the coset colouring of the triangle tiling, whose fundamental triangle has a trivial stabilizer, so that its tiles correspond one to one to the elements of the group [FRETTLOH2008]. `colourings.md` 5 gives, for the regular tilings $\{p, q\}$ and $\{q, p\}$ and the Laves tiling `[p.q.p.q]` only, which subgroups of the reflection group of the triangle and of its rotation group induce a perfect colouring of the tiles, by the generators, or for the rotation group the products of two generators, that a subgroup contains (Lemmas 3.2 and 4.1 of [FRETTLOH2008]), and a procedure that lists them with a low-index search. `colourings.md` 7.17 gives the low-index search that `escher-group` implements. `colourings.md` 5.9 gives what the sources state about a subgroup under which no element other than the identity maps a polygon of a tiling onto itself: Yuncken's theorem on which regular tilings $\{p, q\}$ are tilings by fundamental domains of a group of orientation-preserving isometries (5.9.2), Poincare's polygon theorem (5.9.3), and the Heesch and isohedral types of Conway, Burgiel, and Goodman-Strauss, also for the hyperbolic groups (5.9.4). No source read treats such subgroups with orientation-reversing elements, counts them, or joins them to coset tables, and the statements on perfect colourings cover no tiling of a Wythoff construction other than the regular and Laves tilings.

**Unverified statements relied on.** `UNV-41`, `UNV-42`, and `UNV-43` for the arrangements, `UNV-44` for the stabilizer of a polygon, `UNV-45` for the edge pairing, and `UNV-46` for the picture of a sample, which also rests on the fold product of `UNV-17`. Through M23, the faces rest on `UNV-37` to `UNV-40`, and through M22 the mask map rests on the unverified statements of the render design. The selection of the regular constructions, the flat matrices of the stabilizer, the map from a face onto the flat polygon, one to one and continuous across the lines between its wedges, and the seams of the face mask are statements of the render design, which its tests check (Section 10 of `docs/design/render.md`).

**Tests and acceptance.** The tests of the layer "Whole faces" of Section 8.1 of `docs/design/render.md`: "Regular scope", "Stabilizer", "Edge pairing", which checks that every edge pairing is an involution and that every arrangement with no mirrored neighbour meets the vertex condition of Lemma 0.2 of [YUNCKEN2003], "Pairing geometry", "Whole picture", which checks on the renders of five constructions, against an oracle in the reference arithmetic, that a picture with an asymmetric mark appears once on every polygon, turned or mirrored where the arrangement says, "Face seams", which checks at three points of each edge that face masks laid edge to edge as their marks say reproduce the seams of the render, "Face mask content", "Same picture in every model", and "Upright picture", which checks on three tilings that two features of a picture painted upright on the face mask appear on the reference polygon in the default view within an angle of their painted directions, with the rows of Section 8.2 and the golden images `wf-7-3-2-010-14a.png`, `wf-6-3-2-010-12a.png`, `wf-5-3-2-010-10a.png`, and `fmask-7-3-2-010-14a.png` of Section 8.4. The test "Free filter" of Section 10.1 of `docs/design/group.md`. The snapshots and plain tests of `--arrangement` and `info --wythoff` in Section 8.1 of `docs/design/cli.md`, among them the golden listing of the arrangements of the hexagons, seven if the reading of a secondary catalogue in that section holds, the check that the arrangements that `info` lists are exactly those that the search finds, and the refusals, each with exit status 1 and no file written.

**Dependencies and `deny.toml` work.** None new.

**Risks.** An arrangement with one picture may not exist for some regular tilings, such as $\{4, 5\}$ and $\{3, 7\}$, which have no orientation-preserving one by Yuncken's theorem, and M24 refuses such a tiling, which M25 serves. The arrangements with mirrored neighbours, the edge pairing from a coset table, and the picture of a sample rest on no source, so a wrong statement draws a wrong picture without an error, which the oracles of the tests check only on the tilings that they render (Section 8). A polygon with more than 32 sides has an index above the index limit of `escher-group`, so its tiling is refused. The mask map distorts the picture in the same way in each of the $2n$ wedges of a polygon.

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Libraries: `free_classes_of_index` of `escher-group` with its tests, and `WholeFace`, `EdgeMeeting`, `WholeFaceFill`, `Fill::WholeFace`, the edge pairing, the picture of a sample, the face mask, and the tests and golden images of `escher-render`.
3. Command line: `info --wythoff`, `--arrangement` of `render` and `render-masks` in `escher-cli`, with their snapshots and plain tests, the usage of the whole-face fills in `README.md`, and Project Status in `AGENTS.md`.

### 5.25. M25: Whole-Face Fills With Several Pictures of One Polygon

**Goal.** On a regular tiling, the user puts several pictures on the polygons, one for each orbit of an arrangement that maps no polygon onto itself other than by the identity, so that the regular tilings for which the search of M24 finds no arrangement with one picture get whole-face fills, such as squares five to a vertex, $\{4, 5\}$. That the search finds none there is its result, which rests on `UNV-41` to `UNV-43`, and Yuncken's theorem states only that the tiling has no orientation-preserving arrangement with one picture (`colourings.md` 5.9.2). The owner decided on 2026-11-23 that this work is one of the last milestones of the proof of concept and not a roadmap item (`DEC-114`). That it is a milestone of its own is a proposal (`DEC-121`).

**Command line.** These commands illustrate the milestone. The first lists the arrangements of the triangles of `*732`, seven to a vertex, now with those of several pictures, the second writes the three face masks of the arrangement `18a`, and the third puts the three pictures that the user painted over them on the triangles. The id `18a` stands for the first arrangement with three pictures, which the snapshot of the listing fixes:

```sh
escher info '*732' --wythoff 100
escher render-masks '*732' --wythoff 100 --arrangement 18a -o /tmp/masks/triangle.png
escher render '*732' --wythoff 100 --arrangement 18a --colours 'image:/tmp/masks/a.png,image:/tmp/masks/b.png,image:/tmp/masks/c.png' -o triangles.png
```

**Scope.** The owner decided on 2026-11-24 that some regular tilings need more pictures, that the tilings that work with one picture gain choices, such as the checkerboard of the squares, that the user picks an arrangement by id, gets one mask for each picture, and passes as many images, that an edge between two pictures shares a border colour across the two masks, and that the arrangements with two and three pictures are listed beside those with one (`DEC-124`). The owner allowed arrangements with more than three pictures on the same day (`DEC-129`). The design applies the orientation of each mask (`DEC-122`), the tick of edge 0 (`DEC-123`), and the mirrored arrangements (`DEC-112`) to every picture, as part of the proposal `DEC-125`.

The design branch settled the rest as proposals, and the owner has decided none of them:

- A picture is an orbit of the faces under the arrangement, read from the coset table as an orbit of the stabilizer on the cosets. Picture A holds coset 0, each further picture starts at the first coset that the breadth-first traversal of the table reaches outside the pictures before it, and the face of that coset is the reference face of the picture. Each mask shows its reference face as the default view shows it, by `DEC-122` applied to that face, which fixes whether a meeting between two pictures is turned or mirrored (`DEC-125`, Section 5.30 of `docs/design/render.md`).
- `info --wythoff` lists the arrangements with one picture as in M24, then for each further number of pictures a count line and its lines, up to the colour limit of `--max-colours` and the index limit, and stops after the first search that stops at a limit. The id keeps the form of `DEC-115`, so its number, the index, tells how many pictures it needs, and an id with one picture keeps its meaning (`DEC-126`, Section 5.10 of `docs/design/cli.md`).
- `render-masks` writes one face mask for each picture, to the value of `--output` for one picture as before, and with the names of the polygon masks for several, `mask-0.png` for A, `mask-1.png` for B, and so on, and `--colours` takes one item for each picture, in the order A, B, C (`DEC-127`).
- The diagnostic of Section 6.8 of `docs/design/cli.md` that an id does not exist names the number of pictures of the id, also for one picture, and the other diagnostics keep their texts for one picture and name the number of pictures for several (`DEC-128`).
- No new option, limit, or search function. The index limit of 64 keeps its value and bounds the number of pictures of a polygon, as Section 4.6 of `docs/design/cli.md` states.

**Design documents.** Sections 5.29 to 5.33 of `render.md` with its other sections that the pictures touch, Sections 4.2, 4.3, 4.5, 4.6, 4.8, 5.1, 5.10, 6.8, and 6.10 of `cli.md`, Section 7.19 of `group.md`, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-124` to `DEC-129`, and statements 4 to 6 of `colourings.md` 5.9.6 with items 50 to 52 of its Section 14.1.

**Sources.** Those of M24. Statements 1 to 3 of `colourings.md` 5.9.6 are stated for any number of orbits on one kind of polygon. Yuncken's theorem decides which regular tilings have an orientation-preserving arrangement with one orbit, and says nothing about several orbits (`colourings.md` 5.9.2). No source read reads several orbits from a coset table, pairs the edges of different orbits, or treats a pairing with a mirror-image neighbour.

**Unverified statements relied on.** `UNV-41` to `UNV-46`, with `UNV-43` for the number of pictures, and `UNV-51` for the pictures as orbits of the stabilizer on the cosets, `UNV-52` for the base words of even length and the edge meetings between pictures, and `UNV-53` for the picture of a sample. The upright turn of each mask and the seams between masks are statements of the render design, which its tests check (Section 10 of `docs/design/render.md`).

**Tests and acceptance.** The tests of the layer "Whole faces" of Section 8.1 of `docs/design/render.md`, extended to several pictures: "Picture orbits", which checks on every arrangement of the whole-face set with one, two, and three pictures, and on those with four or more pictures of four tilings, that each picture covers $2n$ cosets once and the pictures together every coset, "Edge pairing", which checks that the meetings over all pictures form an involution, "Pairing geometry", "Whole picture", which checks on whole images in the three geometries, against an oracle that walks the faces by the meetings, that each face shows the picture of its orbit once, turned or mirrored as the arrangement says, on arrangements with two, three, and four pictures, with a handedness check from the rendered pixels and one named arrangement for each kind of meeting between two different pictures that the search gives, or an assertion that it gives none, "Face seams", which checks the seams between the masks of two pictures and the marks on the mask pixels, "Upright pictures", which checks every picture in the plane and in the disk, and picture A on the ball, at the centre of its reference face, "Checkerboard", which checks that the checkerboard of the squares with two solid colours equals the coset fill of the colouring with two colours that holds the stabilizer, and "More pictures", which checks that the search finds arrangements with three pictures for `100` on `*732` and `*542` and with two on `*532`, whose search with one picture finds none, as results of the search and not facts with a source. The picture set names tilings whose searches complete under the default budget, and the tests assert that they do. The golden images of Section 8.4 of `docs/design/render.md` for several pictures, the checkerboard of the squares and the mask of its picture B, and the first arrangement with three pictures of the triangles of `*732`, each named with the id that the listing gives it, and every golden image of M24 unchanged. The snapshots and plain tests of Section 8.1 of `docs/design/cli.md`: the listing that equals the search at every listed index, with row `Arrangements` and the lines of one picture byte for byte as in M24 and the new lines after them, the listing of M24 under `--max-colours 1`, the diagnostic that an id does not exist with its number of pictures, and the flows with two and three pictures, whose failures leave every file as it was.

**Dependencies and `deny.toml` work.** None new.

**Risks.** The cost of a low-index search grows exponentially with the index (Section 8), so a listing can stop at the step budget before its largest number of pictures, which `info` reports on the count line of the number of pictures whose search stopped. A tiling whose search finds no arrangement within the index limit gets none, and the tests assert only what the search finds. The edge meetings between pictures, their flags, and the picture of a sample rest on no source, so a wrong statement draws a wrong picture without an error, which the oracles check only on the tilings that they render. The listings grow with the number of pictures, so a user reads many lines to choose one arrangement, and shortens them with `--max-colours`. The whole-image tests cover up to four pictures. On the ball, the reference face of a picture other than A can lie on the far side, out of the default view, or away from the front, where the picture can look turned or squashed although its mask follows the rule of Section 5.31 of `docs/design/render.md`.

**Split.** Three branches, in this order:

1. Design: the design documents above.
2. Library: the pictures, their base words, reference faces, and turns, the edge meetings between pictures, `EdgeMeeting::picture`, the `picture` argument of `WholeFace::render_mask`, one paint for each picture in `WholeFaceFill::new`, the classes and marks over all pictures, and the tests and golden images of `escher-render`.
3. Command line: the listing of the numbers of pictures up to the colour limit in `info --wythoff`, the resolution of an id at its index, the masks of several pictures in `render-masks`, one item of `--colours` for each picture in `render`, their snapshots and plain tests, the usage of several pictures in `README.md`, and Project Status in `AGENTS.md`.

**What M26 needs from this design.** The reading of pictures from a coset table in Section 5.30 of `docs/design/render.md` is stated for faces of one type: one stabilizer, $2n$ cosets in each orbit, one flat polygon $Q$, one centre mirror $m$ for the parity of a base word, and edges that all meet edges of the same polygon. With several face types, each type has its own stabilizer of order $2 n_k$ (`UNV-47`), so the orbits on the cosets are taken for each type, a picture belongs to one type, the base word, the turn, and the mask polygon are per type, and a meeting can join edges of two polygons with different numbers of sides, which the edge words of step 7 of Section 5.29 of that document do not yet describe.

### 5.26. M26: Whole-Face Fills of Tilings With Several Kinds of Polygon

**Goal.** On a Wythoff construction with more than one face type, such as `4.6.14` of `*732` with `111`, the user puts one or more pictures on each kind of polygon, each painted on a flat mask of its polygon, and the tool puts them on every polygon of that kind, with every edge matching across the kinds. The owner decided on 2026-11-23 that this work is one of the last milestones of the proof of concept and not a roadmap item (`DEC-114`), and on 2026-11-24 that one arrangement serves every kind (`DEC-137`). That it is a milestone of its own after M25 is a proposal (`DEC-121`).

**Command line.** These commands illustrate the milestone. The first lists the arrangements of the dodecagons, hexagons, and squares of `*632` with `111`, the second writes the six face masks of the arrangement `12a`, `0A`, `1A`, `1B`, `2A`, `2B`, and `2C`, and the third puts the six pictures that the user painted over them on the tiling. The id `12a` stands for the first arrangement of the least index 12, which the snapshot of the listing fixes:

```sh
escher info '*632' --wythoff 111
escher render-masks '*632' --wythoff 111 --arrangement 12a -o /tmp/masks/tile.png
escher render '*632' --wythoff 111 --arrangement 12a --colours 'image:/tmp/masks/0a.png,image:/tmp/masks/1a.png,image:/tmp/masks/1b.png,image:/tmp/masks/2a.png,image:/tmp/masks/2b.png,image:/tmp/masks/2c.png' -o tiles.png
```

**Scope.** The owner decided on 2026-11-24 that one arrangement serves every kind of polygon, so that every edge matches and a kind can need several pictures, option (a) of the two that the owner was shown (`DEC-137`), and that the limit of the search is raised so that the tiling 4.6.14 of `*732` with `111` can be searched, unless the tests or CI then run too long, in which case it is not raised and that tiling is refused (`DEC-138`). The design applies the orientation of each mask (`DEC-122`), the tick of edge 0 (`DEC-123`), the mirrored arrangements (`DEC-112`), and the pictures of `DEC-124` and `DEC-129` to every kind.

The design branch settled the rest as proposals, and the owner has decided none of them:

- The arrangement index limit is 84 for `free_classes_of_index` only, and every other search, the colouring ids, `--max-colours`, and the arrangements of a regular construction keep 64. The acceptance condition is the owner's rule, read with the threshold of the acceptance below (`DEC-139`, GRP-47 of `docs/design/group.md`).
- `WholeFace::new` accepts every construction with two or three face types, `*n22` and `*222` included, and keeps the regular constructions and the refusals of the other constructions with one face type (`DEC-140`, Section 5.35 of `docs/design/render.md`).
- Each face type has its own centre and vertex mirrors, stabilizer words, flat regular polygon, region of the triangle with its region map, angle of the upright rule, and pictures, read from the one coset table, and the pictures are named by the position of their corner and their letter, such as `1B` (`DEC-141`, Sections 5.35 and 5.36 of `docs/design/render.md`).
- An edge between two kinds is read from the copy of the domain that holds both faces, and an edge on a mirror from the copy across it (`DEC-142`, Section 5.36 of `docs/design/render.md`).
- `info --wythoff` lists the least index $L$, the least common multiple of the $2 n_k$, and each further multiple up to 84 whose total of pictures is within the colour limit of `--max-colours`, in the rows of `M25` with the picture names of each type on each line (`DEC-143`, Section 5.10 of `docs/design/cli.md`).
- The masks and the items of `--colours` follow the order of the picture names, each mask file carries its picture name, every mask of an arrangement with more than one picture, of a regular construction too, has one scale at which every edge has one length in pixels, which changes the masks and renders of `M25` that Section 5.32 of `docs/design/render.md` lists, an arrangement with more than 12 pictures needs `--colours`, and an arrangement id up to 84 parses through `ColouringId::parse_arrangement` (`DEC-144`, `DEC-145`).

**Design documents.** Sections 5.35 to 5.38 of `render.md` with its other sections that the several face types touch, Sections 4.2, 4.5, 4.6, 4.8, 5.1, 5.10, 6.8, 6.10, 8.1, and 11 to 13 of `cli.md`, Sections 3, 7.12, 7.15, 7.19, 8.2, 10.1, and 15 of `group.md`, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-137` to `DEC-145`, and statements 3 to 5 of `colourings.md` 5.10.4 with items 53 to 55 of its Section 14.1.

**Sources.** Those of M23 and M24. `colourings.md` 5.10 gives what the sources state about a Wythoff construction with several face types: the faces at the corners of the triangle for each Wythoff symbol on the sphere [CLM1954] [HAREL1993], the faces of the doily construction of [CBGS2008] as disks of the orbifold with at most one cone point or kaleidoscopic point, in all three geometries, and the terms of the sources for a tiling with several orbits of tiles under a given group (5.10.1 and 5.10.2). It also gives the index bounds for torsion-free subgroups of Fuchsian and NEC groups, read through secondary restatements (5.10.5). No source read states the stabilizer of a face of a Wythoff construction, treats the subgroups that act freely on the faces of several types, or derives how many pictures each kind of face needs (5.10.3, 5.10.4, and 5.10.6). Frettloh's lemmas cover the regular and Laves tilings only (`colourings.md` 5.3).

**Unverified statements relied on.** `UNV-38` to `UNV-40` for the faces, the regions, and the generator point, `UNV-47` for the stabilizer of a face of each type, `UNV-48` for the composition of a face from images of its region of the triangle, `UNV-49` for the subgroups that act freely on the faces of every type and their test on a coset table, `UNV-50` for the number of pictures of each type, `UNV-54` for the pictures of each type read from the coset table, `UNV-55` for the meetings between types and across a mirror, and `UNV-56` for the picture of a sample, which also rests on the fold product of `UNV-17`. The region map, the midpoint of edge 0, the upright turn of each picture, and the seams between masks of two types are statements of the render design, which its tests check (Section 10 of `docs/design/render.md`).

**Tests and acceptance.** The tests of the layer "Several face types" of Section 8.1 of `docs/design/render.md`: "Several scope", "Type stabilizers", "Region map", "Type pictures", which states the numbering of the pictures without the traversal of the code, "Several worked by hand", which holds the coset table, the pictures, and every meeting of the arrangement of the translations of `*442` with `111` as values worked by hand in Section 5.36 of that document, "Type meetings", "Meeting geometry", "Same meetings", "Several whole picture", which compares whole images in the three geometries with an oracle that walks the faces across their edges in the reference arithmetic, with a handedness check from the rendered pixels, "Several wedge continuity", "Known translation fill", which compares that arrangement with an image periodic under the translations that the oracle computes without the coset table, "Several seams", which lays the masks of different types edge to edge, "Several mask content", "Several upright pictures" in the plane and in the disk, and "Index 84", with "Mask edge lengths" of the layer "Whole faces", the golden images of Section 8.4 of that document for several face types, and every golden image of `M27` and earlier unchanged except those of `M25` that the arrangement placement changes (RND-87). The code branch that changes a golden image or a snapshot of `M25` regenerates it and names it in its commit message. The tests "Arrangement index limit", "Arrangement ids", and "Several face types" of Section 10.1 of `docs/design/group.md`. The snapshots and plain tests of several face types of Section 8.1 of `docs/design/cli.md`: the listings that equal the search, the listing of `*732` with `111` at the index 84, the error of the limit on `*752` with `111`, the whole flow with four masks, the order of the items, the diagnostics, and failures that leave every file as it was, with every listing and diagnostic of a regular construction, and every mask and image of one that RND-87 keeps, byte for byte as before (CLI-75).

The acceptance of the arrangement index limit of 84 reads the owner's rule of `DEC-138`, "unless it makes testing / CI run for too long", with a threshold that this plan proposes. Three tests carry the bound, and each of them runs in under 3 seconds when it runs alone under the nextest profile `default`, in the time that nextest reports for it in the development container:

- The unit test "Arrangement index limit" of Section 10.1 of `docs/design/group.md`, which asserts that `free_classes_of_index` on `*732` with `111` at the index 84 returns a complete result. The group branch measures it, and its time decides whether that branch raises the limit.
- The library test "Index 84" of Section 8.1 of `docs/design/render.md`, which the library branch measures.
- The snapshot of `escher info '*732' --wythoff 111` at the index 84, the acceptance test of the command line of Section 8.1 of `docs/design/cli.md`, which the command-line branch measures.

Each measurement has one of two outcomes:

- Under 3 seconds: the arrangement index limit is 84, as GRP-47 and GRP-48 of `docs/design/group.md` state, and the test stays.
- 3 seconds or more: the limit of 84 is not adopted, and the branch that measured the time stops. A limit of 64 changes the values that the design documents derive from 84 and the exception of CLI-75 of `docs/design/cli.md`, so a design branch restates the documents for the limit of 64 before the code continues.

**Dependencies and `deny.toml` work.** None new.

**Risks.** No source states the stabilizers of the faces, the pictures of each type, or the meetings between types, so the milestone rests on unverified statements, which the oracles check only on the tilings that they render. A construction whose least index is above 84, such as `*752` with `111`, gets no arrangement, and the search at the least index may find none, which only the search can tell (item 49 of `colourings.md` 14.1). An arrangement can need up to 63 pictures, so a user paints and passes many images, and above 12 pictures `render` needs `--colours`. At the one scale of the masks of an arrangement, the mask of a small polygon beside a large one fills a small part of its image. The test at the index 84 may take 3 seconds or more, in which case the limit is not raised.

**Split.** Four branches, in this order:

1. Design: the design documents above.
2. Group: the arrangement index limit of `free_classes_of_index` with its index as a `u32`, `ColouringId::parse_arrangement`, their tests, the change of the callers in `escher-cli` with the regular listings unchanged, and the measurement of the test "Arrangement index limit" under the acceptance condition of `DEC-139`.
3. Library: `FaceType`, the steps of several face types in `WholeFace`, the region maps, the pictures and meetings of each type, the face type passed to the fill, the masks, and the tests and golden images of `escher-render`.
4. Command line: the listing of several face types in `info --wythoff`, the resolution of their ids, the masks and the items of `--colours` in the order of the picture names, the diagnostics, their snapshots and plain tests, the usage of several kinds of polygon in `README.md`, and Project Status in `AGENTS.md`.

### 5.27. M27: Renders Centred on a Corner

**Goal.** The user renders a tiling with a corner of the fundamental domain at the centre of the picture, in place of the centre of the domain, such as the heptagons of `{7,3}` with a heptagon or a vertex of the tiling at the centre. The owner asked for this on 2026-11-24 as an option of `render` and decided its main points (`DEC-130` to `DEC-133`).

**Command line.** These commands illustrate the milestone. The first prints the line `Corner angles in degrees: 25.714286, 60.000000, 90.000000`, whose positions 0, 1, and 2 are the corners of orders 7, 3, and 2. The second draws the heptagons of `{7,3}` with a heptagon at the centre, and the third with a vertex at the centre, where three heptagons meet:

```sh
escher info '*732'
escher render '*732' --wythoff 010 --centre 0 -o heptagon.png
escher render '*732' --wythoff 010 --centre 1 -o vertex.png
```

**Scope.** The owner decided these details on 2026-11-24:

- An option `--centre` of `render` puts a corner of the domain at the centre of the picture, as a change of the view only, and without the option every picture stays as it is (`DEC-130`).
- The option names a corner by its position in the order in which `info` prints the corners, and not by its order, because two corners can have the same order, as in `*333`. The help text documents this clearly (`DEC-131`).
- The face masks do not follow `--centre`, so the upright rule of `DEC-122` keeps using the default view (`DEC-132`).
- The work is a milestone of its own, designed after the design of M25 and built alongside the code of M25 (`DEC-133`).

The design branch settles the rest as proposals, and the owner has decided none of them:

- The value is a decimal integer, the position of a vertex of the domain counted from 0 in the order of the angles of the line `Corner angles in degrees` of `info`. Every vertex is a position, a flat vertex of 180 degrees included. A position at or above the number of vertices is refused, and so is every position on the domain of `1`, which has no vertex (`DEC-134`, Section 5.34 of `docs/design/render.md`, Section 4.6 of `docs/design/cli.md`).
- The view applies the translation of `escher-geometry` that maps the origin to the corner to the point of each sample just before the fold, after the map of the hyperbolic model, and on the ball after the transpose of the viewing matrix, so that `--rotation` turns the centred ball. No further turn is added, and the domain stays centred at the origin for the fold (`DEC-135`, Section 5.34 of `docs/design/render.md`).
- The option combines with every option of `render` and changes only the view, `render-masks` has no such option, and a position that the domain does not have stops `render` at step 5, before the searches of step 6 (`DEC-136`, Sections 4.5, 4.6, and 6.8 of `docs/design/cli.md`).

The milestone adds no limit. A free shift, a zoom, and a turn about the corner stay in the roadmap (Section 4 of `docs/design/roadmap.md`), and a centre at a point other than a vertex of the domain, such as the centre of a face of a Wythoff construction that lies at no corner, is not part of it.

**Design documents.** Section 5.34 of `render.md` with its other sections that the centred view touches, Sections 4.2, 4.5, 4.6, 6.8, 8.1, and 11 of `cli.md`, the data flow of Section 4.1 of `docs/design/architecture.md` with the labels `DEC-130` to `DEC-136`, and the row of view control in Section 4 of `docs/design/roadmap.md`.

**Sources.** No research document is needed. The view composes isometries that `escher-geometry` already has, `hyperbolic::translation`, `euclidean::translation`, and `spherical::translation`, each of which maps the origin to its target (Sections 6.4, 6.6, and 6.7 of `docs/design/geometry.md`). No source states the matrices of the hyperbolic and the spherical translation, and reference tests of the geometry design check them (Section 13.1 of `docs/design/geometry.md`). The vertices of each domain and their order come from the domain design (Sections 6.3 and 6.8 of `docs/design/domain.md`).

**Unverified statements relied on.** No new row of Section 7. The fold rests on `UNV-1` and `UNV-2` as without the option, each fill on the statements of its milestone, and the translations on the assumptions of Section 13.1 of `docs/design/geometry.md`. That the corner shows at the centre of the image, that the centre of the domain shows on the side of the centre of the image away from the direction of the corner, and that the centred view draws the same tiling are statements of the render design, which its tests check (Section 10 of `docs/design/render.md`).

**Tests and acceptance.** The tests of the layer "Centred views" of Section 8.1 of `docs/design/render.md`: "Corner at the centre", which checks in the reference arithmetic, in the disk, the half-plane, the band, the Klein model, the plane, and the ball, that the point of the centre of the image is the corner, for each position of `*732`, `*632`, `*532`, and `732`, "Rotation after centring", "Centre direction", "Border at the corner", "Same tiling", which runs the whole-image border oracle on centred images, "Positions differ", "No centre", and the refusals of `Renderer::new` with their texts. The snapshots and plain tests of `--centre` in Section 8.1 of `docs/design/cli.md`: the help of `render` in both layouts, the usage errors of `render` and `render-masks`, the diagnostics of a position that the domain does not have and of the domain of `1`, each with exit status 1 and no file written, and the combinations with `--model`, `--rotation`, `--wythoff`, and `--arrangement`. Every committed golden image and snapshot stays byte for byte as it was, except the two help snapshots of `render`.

**Dependencies and `deny.toml` work.** None new.

**Risks.** Small. The translation adds one rounded matrix product to each sample, and with a corner far from the centre of the domain, samples near the rim of the disk lie farther from the domain than in the canonical view, where the fold is less accurate (Section 6.13 of `docs/design/render.md`). A user may expect positions from 1, or a corner named by its order, which the help text addresses. A rotation signature has several vertices for one cone point, such as positions 1 and 3 of `732`, which give two pictures of the same kind of centre.

**Split.** Two branches, in this order:

1. Design: the design documents above.
2. Library and command line: the option `centre` of `RenderOptionsBuilder`, the placement of the centred view, and the refusal of `Renderer::new` in `escher-render`, with their tests, and `--centre` of `render` in `escher-cli` with its long and short help, the renderer built at step 5, its snapshots and plain tests, one example in `README.md` of the heptagon tiling of `*732` with `--wythoff 010 --centre 0`, centred on a heptagon, with its image written by `just readme-images` through `scripts/readme-images`, and Project Status in `AGENTS.md`. The library and the command line share one branch, because the option has no use in the library without the command line, and each half is small.

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
| UNV-1 | The first-violated-wall fold terminates for compact reflection polygons, in a number of steps equal to the word length. On the sphere, at orders above the limit of `DEC-69`, points within $10^{-14}$ radians of a vertex of order 1000 stayed unconverged at the default cap, and near a vertex of order 999 points took more steps than the longest word of the group. Near the vertex of order 100 of `*(100)22`, the tests assert only that each fold returns within the cap (Section 6.7 of `docs/design/render.md`). No source covers every point, so the design relies on it only for the samples of the images that the tests render and the images of the centre | `rendering.md` 4.7 and 13.1 item 1, gap G5, `fundamental-domains.md` 13.3 item 6 | M3 on, including the lune of order 1 of `*` in M11, M22, and M23 | Sections 1.6, 1.7, and 5.4 of [HUMPHREYS], or Lean. The sources that do not state it are listed in `rendering.md` 4.7 and gap G5 |
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
| UNV-14 | The distance to a geodesic in the forms that no source states, which include the hyperboloid form for unit normals and the disk form for a geodesic that is not a diameter, and the distance to a great circle. For a vector off the hyperboloid, the hyperboloid form reads the norm $\lvert\lvert\lvert y \rvert\rvert\rvert$ of equation (3.2.8) of [RATCLIFFE] as $\sqrt{-\langle y, y \rangle}$, which no research document defines. The sourced forms are in `geometry-models.md` 9.3 and `rendering.md` 6.2 | `geometry-models.md` 9.3 and 15.1 item 3, `rendering.md` 13.1 items 8, 10, and 26 | M3 tests, M13, M23 | A source, or Lean |
| UNV-15 | The screen-distance estimate through the fold, analytic antialiasing, and the fade of tiles smaller than a pixel | `rendering.md` 7.1 and 13.1 items 19 and 20 | M3, M23, and the roadmap item M14 | Measurement against supersampling |
| UNV-16 | The lift onto the ball, the view scale of the sphere, and the Phong formulas | `rendering.md` 13.1 items 11 and 12, `geometry-models.md` 11.6 | M5 | [PHONG75], [BLINN77] |
| UNV-17 | Coset colourings are perfect for every orbifold group, the composition order, completeness, and conjugacy. The composition order is the rule of GRP-38 of `docs/design/group.md`: the fold that applies the sides $i_1, \ldots, i_n$ in that order puts the sample in the tile $gF$ with $g = s_{i_1} \cdots s_{i_n}$, and the colour is the number of the right coset $S g^{-1}$, traced from coset 0 through the inverse column of each side in reverse order, which names the left coset $gS$. A prototype outside the repository found the rule perfect, with the exact permutation of each side, on every class that it tried, found the forward trace not perfect on every class that is not normal, and found the reverse trace without the inverse to give the mirror image on rotation domains (Sections 7.10 and 7.16 of `docs/design/group.md`). For conjugacy, the design lists every subgroup as its own colouring (`DEC-37`), after the prototype found that no sampled element of the group relates the pictures of two conjugate subgroups | `colourings.md` 4.2, 8.4, 8.7, and 14.1 items 1 to 3 | M8, M20, M22, M24 | [DDL2009], [SCHWARZENBERGER1984], or Lean. [CBGS2008] defines colour types and states none of these (`colourings.md` 3.3 and 14.1 items 1 and 2) |
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
| UNV-37 | The regular tiling $\{p, q\}$ with $p, q \ge 3$ has the symmetry group that `*pq2` names, with the corner of order $p$ at the centre of a tile and the corner of order $q$ at a vertex, its group of orientation-preserving symmetries is the group that `pq2` names, and its dual is $\{q, p\}$. For the sphere and the Euclidean plane, `colourings.md` 10.3 states the corners and the dual, and no source read states them for the hyperbolic plane. No source states the naming in any geometry. A source states the symmetry group as the reflection group of the right triangle with the angles $\pi/2$, $\pi/p$, and $\pi/q$, which is also the group of $\{q, p\}$, and the rotation group as a subgroup of index 2 (`colourings.md` 5.1 and 5.4) | `orbifold-notation.md` 12 item 16 | M21, and M23 and M24 for the markings `010` and `100` of a hyperbolic `*pq2` | A source that states it in orbifold notation, or Lean. No test can check the name of a tiling. The check of the angles of `escher-domain` checks that the domain of `*pq2` has the angles of that triangle, and the tests of Section 8.1 of `docs/design/cli.md` pin which signature gets which symbols |
| UNV-38 | The Wythoff construction on a Euclidean or hyperbolic reflection triangle has the faces that the face rule gives for the sphere: at a corner of order $n$, no face when neither mirror through it is active, a regular $n$-gon when one is, and a regular $2n$-gon when both are, with a face of 2 sides dropped, and its edges join the generator point to its images in the active mirrors. On a hyperbolic `*pq2` with $p \ge q \ge 3$, the marking with only the mirror opposite the corner of order $q$ active gives $\{p, q\}$, and the marking with only the mirror opposite the corner of order $p$ active gives $\{q, p\}$. [CLM1954] and [HAREL1993] state the faces, the edges, and the symbols for the sphere, and [CLM1954] the symbols of the Euclidean tessellations, and the members of `(7 3 2)` rest on Wikipedia only | `colourings.md` 10.1 to 10.3, and 14.1 items 10, 31, and 32 | M23, M24, and M26, whose flat polygons are the regular polygons of the face rule | Chapter 10 of [COXETER1999], [CKPS1995], or another primary source on the hyperbolic plane, or Lean. The tests "Face rule", "Faces around a centre", and "Face oracle" of Section 8.1 of `docs/design/render.md` check the number of sides of each face and the faces on whole images |
| UNV-39 | After the fold, the perpendiculars from the generator point to the active mirrors cut the reflection triangle into one region for each corner with a face, and a folded point belongs to the face centred at the corner of its region, in every geometry. Two shaders by one author do this on the sphere and in the hyperbolic plane, and no paper or book states it | `colourings.md` 10.6.3 and 14.1 item 11 | M23, M24, M26 | A paper that states the decomposition, the helpers of the hyperbolic shader read line by line, or Lean. The tests "Face regions", "Faces around a centre", "Face oracle", and "Known faces" of Section 8.1 of `docs/design/render.md` check it |
| UNV-40 | On a Euclidean or hyperbolic reflection triangle, the generator point of a marking is the point on the inactive mirrors at equal distance from the active ones, it exists, and it is unique, as the vertex, the point on the bisector, and the incentre are on the sphere | `colourings.md` 10.1, and 14.1 items 30, 33, and 34 | M23 | A source that proves it, or Lean. The test "Generator point" of Section 8.1 of `docs/design/render.md` checks the distances of the computed point in the reference arithmetic |
| UNV-41 | A subgroup $S$ of finite index of the symmetry group $G$ of a tiling acts freely on the polygons of one orbit of $G$, those of a polygon $P$ with a finite stabilizer $G_P$, exactly when $S \cap g G_P g^{-1}$ is trivial for every $g \in G$. The definition of semiregular and the stabilizer of another point as a conjugate are sourced through lecture notes read through a summarizing tool (`colourings.md` 5.9.1 and 5.9.6) | `colourings.md` 5.9.6 statement 1, and 14.1 item 35 | M24, M25, M26 | A textbook statement, such as one in Dixon and Mortimer, or Lean |
| UNV-42 | In the permutation action of $G$ on the cosets of $S$ that the coset table of $S$ gives, the condition of `UNV-41` holds exactly when no element of $G_P$ other than the identity fixes a coset. That the conjugates of a subgroup are the point stabilizers of its table is sourced (`colourings.md` 7.17), and the stabilizer of a coset under a subgroup rests on a secondary source | `colourings.md` 5.9.6 statement 2, and 14.1 item 36 | M24, M25, M26 | A source that states it, or Lean |
| UNV-43 | When $S$ acts freely on the orbit of $P$, it has $[G : S] / \lvert G_P \rvert$ orbits on it. The orbits of a subgroup as double cosets and the complement of a regular subgroup are sourced through an online book, lecture notes, and a research paper, read through a summarizing tool (`colourings.md` 5.9.6) | `colourings.md` 5.9.6 statement 3, and 14.1 item 37 | M24, M25, M26 | A textbook statement, such as one in Dixon and Mortimer, or Lean |
| UNV-44 | On `*pq2` with $p \ge q \ge 3$, the stabilizer of the face of $\{p, q\}$ or $\{q, p\}$ centred at a corner $c$ of order $n$ of the triangle is the group of order $2n$ that the reflections in the two sides through $c$ generate, the $2n$ words $(s_m s_v)^j$ and $(s_m s_v)^j s_m$ for $j$ from 0 to $n - 1$ name its elements, each once, and the face is the union of the images of the triangle under it, in the three geometries. [FRETTLOH2008] states the $q$-gon $F \cup aF \cup abF \cup \dots$ of `(q^p)` only, and not that a group of two generators is the stabilizer of a face | `colourings.md` 5.3, and 14.1 items 4 and 42 | M24, M25 | A source that states the stabilizer of a face of a regular tiling, or Lean. The test "Stabilizer" of Section 8.1 of `docs/design/render.md` checks parts of it in the reference arithmetic, and the row of `UNV-44` in Section 10 of that document states which parts no test checks |
| UNV-45 | For an arrangement $K$ of a regular tiling, for each edge $i$ of the reference face, exactly one element $s$ of the stabilizer has $r_i s$ in $K$, with $r_i$ the reflection in edge $i$, the element $r_i s$ maps the reference face onto its neighbour across edge $i$ and carries edge $s^{-1}(i)$ onto edge $i$, the neighbour is turned exactly when $s$ reverses orientation, and the pairing of the edges is an involution. For an arrangement with no mirrored neighbour, this is the involution $\sigma$ of Lemma 0.2 of [YUNCKEN2003], which states it for orientation-preserving groups and pairs an edge with itself by a half-turn, and no source reads it from a coset table. No source treats a pairing with a mirrored neighbour | `colourings.md` 5.9.2, and 14.1 item 43 | M24, M25 | A source that states the pairing for groups with orientation-reversing elements and its reading from a coset table, or Lean. The tests "Edge pairing" and "Pairing geometry" of Section 8.1 of `docs/design/render.md` check it |
| UNV-46 | For an arrangement $K$ of a regular tiling, a sample whose fold applies the sides of the word $g$ lies in the face $gP$, the element of $K$ that carries the reference face $P$ onto it is $g w^{-1}$ for the one element $w$ of the stabilizer with $K w = K g$, and the sample shows the point $w x$ of the reference face for its folded point $x$. No source states it. It also rests on the fold product of `UNV-17` | `colourings.md` 14.1 item 44 | M24, M25 | A source, or Lean. The test "Whole picture" of Section 8.1 of `docs/design/render.md` checks it on whole images against an oracle that walks the faces by the edge pairing, and "Face seams" checks it at sample points on the edges of the faces |
| UNV-47 | On a reflection triangle in any of the three geometries, the stabilizer in the reflection group $G$ of the triangle of the face of a Wythoff construction centred at a corner of order $n$ is the dihedral group of order $2n$ that the reflections in the two sides through that corner generate, whether the face has $n$ or $2n$ sides. The faces centred at the images of one corner form one orbit of $G$, and faces centred at the images of different corners lie in different orbits. No source read states it. [HAREL1993] states that the two mirrors through a corner generate the dihedral group of order $2n$, [CBGS2008] that a face of its doily construction holds at most one cone point or kaleidoscopic point, and preprints and a web page, read through a summarizing tool, state the stabilizer of a face as a subgroup generated by mirrors, for any rank or without a proof that the research recorded. `UNV-44` states the regular case | `colourings.md` 5.10.3, and 14.1 item 45 | M26 | A primary source on the Wythoff construction that states the stabilizer of a face, such as Section 5.7 of Coxeter's "Regular Polytopes" or Huson's paper of 1993 on tile-k-transitive tilings if either does, or Lean |
| UNV-48 | The face centred at a corner of order $n$ is the union of the $2n$ images, under the group of `UNV-47`, of the region of the triangle around that corner that `UNV-39` assigns to the face. [CBGS2008] divides each $n$-gonal face of a polyhedron into $2n$ flagstones, and [FRETTLOH2008] each $p$-gon of `(p^q)` into $2p$ triangles, and neither treats a triangle that several faces share | `colourings.md` 5.10.3, and 14.1 item 46 | M26 | A source that states the decomposition of a face of a Wythoff construction with several face types, or Lean |
| UNV-49 | A subgroup $S$ of finite index of $G$ acts freely on the faces of every face type exactly when $S \cap g G_P g^{-1}$ is trivial for every $g \in G$ and for the face $P$ at each corner with a face, with $G_P$ the group of `UNV-47`, and in the permutation action of $G$ on the cosets of $S$ that its coset table gives, this holds exactly when no element other than the identity of any of these groups fixes a coset. `UNV-41` and `UNV-42` state this for one orbit of the symmetry group of a tiling, and no source read identifies $G$ with the full symmetry group of the construction. A search snippet states only that a finite subgroup meets every conjugate of a torsion-free subgroup trivially | `colourings.md` 5.10.4 statement 1, and 14.1 item 47 | M26 | A textbook statement, such as one in Dixon and Mortimer, with a source that treats a group of symmetries that need not be the full group, or Lean |
| UNV-50 | When $S$ acts freely on the faces of every type, it has $[G : S] / (2n)$ orbits on the faces of the type at a corner of order $n$. `UNV-43` states the count for one orbit of the symmetry group of a tiling, and a paper in another field states the number of orbits of a semiregular group on a finite set, read through a summarizing tool | `colourings.md` 5.10.4 statement 2, and 14.1 item 48 | M26 | A textbook statement, such as one in Dixon and Mortimer, or Lean |
| UNV-51 | For an arrangement $K$ of a regular tiling with $m$ pictures, the pictures, which are the orbits of $K$ on the faces, correspond one to one to the orbits of the stabilizer $G_P$ acting on the cosets of $K$ from the right, with the face $gP$ in the picture of the orbit of the coset $Kg$. Each orbit holds $2n$ cosets, and each coset of an orbit is $bw$ for exactly one $w \in G_P$ and any coset $b$ of that orbit. The nearest sourced statements are the orbits of a subgroup as double cosets, read through a summarizing tool. `UNV-43` states the count for one orbit | `colourings.md` 5.9.6 statement 4, and 14.1 item 50 | M25, M26 | A textbook statement, such as one in Dixon and Mortimer, or Lean. The checks of Section 5.30 of `docs/design/render.md` refuse a table that breaks the count, the test "Picture orbits" checks it on every arrangement of the whole-face set with one, two, and three pictures, and on those with four or more pictures of four tilings, and "Whole picture" checks on whole images that each face shows the picture of its orbit once |
| UNV-52 | For an arrangement with several pictures, a base word $h_X$ of even length, made so by appending the centre mirror $s_m$ to a reach word $u_X$ of odd length, preserves orientation and carries $P$ onto $u_X P$, the reference face of picture $X$. With the base words $h_X$ and $h_Y$ of two pictures, when $K h_X r_i = K h_Y w$ with $w$ in the stabilizer, the element $h_X r_i w^{-1} h_Y^{-1}$ of $K$ carries the reference face of picture $Y$ onto the face across edge $i$ of the reference face of picture $X$ and its edge $w(i)$ onto that edge, it is turned exactly when $w$ is a reflection, and the meetings form an involution on the pairs of a picture and an edge, with the same flag at both ends. `UNV-45` states the case of one picture, and [YUNCKEN2003] the involution for one picture of an orientation-preserving group only | `colourings.md` 5.9.6 statement 5, and 14.1 item 51 | M25 | A source that treats the side pairing of a fundamental domain made of several polygons for a group with orientation-reversing elements, read from a permutation representation, or Lean. The tests "Edge pairing", "Pairing geometry", "Whole picture" with its handedness check, and "Face seams" of Section 8.1 of `docs/design/render.md` check it |
| UNV-53 | For an arrangement with several pictures, a sample whose fold applies the sides of the word $g$ lies in the face $gP$, and when $Kg = K h_X w$ for the base word $h_X$ of a picture $X$ and $w$ in the stabilizer, it shows the point $w x$ of the reference face, carried by $h_X$ onto the reference face of picture $X$, for its folded point $x$. `UNV-46` states the case of one picture. It also rests on the fold product of `UNV-17` | `colourings.md` 5.9.6 statement 6, and 14.1 item 52 | M25 | A source, or Lean. The tests "Whole picture", "Face seams", "Checkerboard", and "Upright pictures" of Section 8.1 of `docs/design/render.md` check it |
| UNV-54 | For an arrangement $K$ of a construction with several face types, the pictures of the face type at corner $k$ correspond one to one to the orbits of its stabilizer $G_k$ acting on the cosets of $K$ from the right, each of $2 n_k$ cosets, every coset lies in exactly one picture of each type, and a base word of even length, made so by appending the centre mirror of the type, carries $P_k$ onto the reference face of its picture and preserves orientation. The nearest sourced statements are the orbits of a subgroup as double cosets, read through a summarizing tool, which apply to each $G_k$ apart. `UNV-51` states the case of one face type | `colourings.md` 5.10.4 statement 3, and 14.1 item 53 | M26 | A textbook statement, such as one in Dixon and Mortimer, with a source for `UNV-47`, or Lean. The checks of Section 5.36 of `docs/design/render.md` refuse a table that breaks the count, and the tests "Type pictures", "Several worked by hand", and "Several whole picture" check it |
| UNV-55 | Half an edge of the face at corner $k$ on an active side $j$ lies, in the same copy of the triangle, on the face of the other corner of side $j$ when that corner has a face, and otherwise on the other side through that corner, across which the neighbour is a face of type $k$. The meetings that Section 5.36 of `docs/design/render.md` reads so from the coset table, with the flag from the determinants of the two stabilizer elements, are the meetings of the faces, and they form an involution with the same flag at both ends. The nearest statements are the perpendiculars as the lines between the faces of the two corners of a side, which only two shaders state (`UNV-39`), and the dropped digon of [HAREL1993]. `UNV-52` states the case of one face type | `colourings.md` 5.10.4 statement 4, and 14.1 item 54 | M26 | A source that treats the side pairing of a fundamental domain made of polygons of several kinds for a group with orientation-reversing elements, or Lean. The tests "Type meetings", "Meeting geometry", "Several worked by hand", "Several whole picture" with its handedness check, "Known translation fill", and "Several seams" of Section 8.1 of `docs/design/render.md` check it |
| UNV-56 | For an arrangement of a construction with several face types, a sample of the face type $k$ whose fold applies the sides of the word $g$ lies in the face $g P_k$, and when $Kg = K h_X w$ for the base word $h_X$ of a picture $X$ of type $k$ and $w \in G_k$, it shows the point $w x$ of $P_k$, carried by $h_X$ onto the reference face of $X$, for its folded point $x$. `UNV-53` states the case of one face type. It also rests on the fold product of `UNV-17`, on `UNV-39` for the face type of the sample, and on `UNV-48` | `colourings.md` 5.10.4 statement 5, and 14.1 item 55 | M26 | A source, or Lean. The tests "Several whole picture", "Known translation fill", "Several seams", and "Several upright pictures" of Section 8.1 of `docs/design/render.md` check it |

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
- [CKPS1995] The entry in Section 15 of `docs/research/colourings.md`.
- [CLM1954] The entry in Section 15 of `docs/research/colourings.md`.
- [COXETER1999] The entry in Section 15 of `docs/research/colourings.md`.
- [DAVIS] The entry in Section 16.1 of `docs/research/geometry-models.md`. `fundamental-domains.md` cites the same book as [DAVIS2008].
- [DDL2009] The entry in Section 15 of `docs/research/colourings.md`.
- [DEATON1993] The entry in Section 14.1 of `docs/research/fundamental-domains.md`.
- [FRETTLOH2008] The entry in Section 15 of `docs/research/colourings.md`.
- [FWW] The entry in Section 16.2 of `docs/research/geometry-models.md`.
- [HAREL1993] The entry in Section 15 of `docs/research/colourings.md`.
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
