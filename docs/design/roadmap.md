# Roadmap Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-18 |
| Applies to | The development of Escher after the proof of concept: its phases, workstreams, research, and the closing of unverified statements |

## Abstract

This document orders the development of Escher after the proof of concept. It lists the phases, from live rendering through motif authoring, tile reshaping, truncated tilings, animation, and video to releases on five platforms, and it marks which parts of the order the owner set and which are proposals. For each phase and each smaller workstream it states what the work depends on and which research it starts from. It defines the deep-research items that turn a surveyed topic into a subject-matter reference before its design starts, and the order in which sources and a Lean development close the unverified statements. It designs none of this work.

## 1. Introduction

The proof of concept is a Linux command-line tool that renders Portable Network Graphics (PNG) images, as `docs/design/poc-plan.md` plans it. The owner has set goals well beyond it, and some of them rest only on surveys, which are not implementation references. Without a stated order, an agent could start a design that its research does not yet support, or build on a phase that does not exist.

This document gives that order. Its readers are the agents that plan the next piece of work and the owner, who decides it. It states order and dependencies only. It holds no dates, estimates, or status, as Section 3.6 of `docs/writing-documentation.md` requires, and each design it names is written later in its own design document.

The owner's goal for the tool is this. An artist draws a motif on one or more fundamental domains, and Escher replicates it live over the tiling to help the artist see the result. The artist reshapes the tiles, animates parameters, and renders the result as raster images, first as PNG stills, then as video, and possibly as other outputs. Section 3 orders this work.

By the owner's decision, the tool is first for the owner, who will use it to make art, and a public release follows at some point.

These topics are out of scope, by the owner's decision:

- Vector output, and any rendering path that generates tiles forward. All rendering is raster and happens per pixel. Vector graphics are only an input, as one medium of a motif (Section 3.3).
- Tilings without a symmetry group, such as aperiodic tilings, for now, because in the owner's words they "don't fit with the orbifold idea".
- Research on integrations with external tools (Section 3.3).

`docs/design/architecture.md` owns the crates, the owner's decisions for the proof of concept (its Section 10.1), and the proposed defaults `DEC-1` to `DEC-31` (its Section 10.2). `docs/design/poc-plan.md` owns the milestones `M1` to `M19`, the research prerequisites `RP-1` to `RP-5`, and the unverified statements `UNV-1` to `UNV-28`. This document cites those labels and does not restate them. It records the owner's decisions for the phases after the proof of concept.

Research documents are cited by file name, and each is in `docs/research/`.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

- phase: A unit of work that has a place in the order of Section 3.
- workstream: A unit of work that no single phase contains, listed in Section 4.
- survey: A research document that maps a topic and is not an implementation reference, such as `export-and-animation.md`.
- deep-research item: A research task that turns a survey, or part of one, into a subject-matter reference, or adds to a subject-matter reference what it lacks for a design (Section 5).
- owner's decision: A statement that the owner made and that this document records. A table marks it "Owner".
- proposal: A statement of this document or of the planning behind it that the owner has not decided. A table marks it "Proposal", and prose says "the planning proposes".
- motif: The artwork that the user draws on one or more fundamental domains and that the tiling replicates. How many domains the user draws on depends on the chosen subgroup.
- colour sampler: The owner's abstraction of a motif, a source of colours for the points of a fundamental domain. The planning proposes that it takes the point that the fold gives and the coset of the tile.
- medium: A kind of content from which a colour sampler takes its colours, such as an image or a video.
- keyframe: A value of an animated parameter at a given time.
- truncated tiling: The owner's "truncated versions of tilings", made with adjustable parameters. The owner has not defined the term, and Section 3.5 records the proposed reading.

The abbreviations used are artificial intelligence (AI), central processing unit (CPU), continuous integration (CI), GNU General Public License (GPL), graphics processing unit (GPU), graphical user interface (GUI), and partial differential equation (PDE).

## 3. Phases

The owner set part of the order in the table. Live rendering comes first, and the development GUI follows it. The first release follows the development GUI, and the later release follows the first release. Results come as PNG stills first, then as video, and possibly as other outputs after video. Animation comes "later down the roadmap", in the owner's words. Every other position, and every dependency that the owner did not state, is a proposal.

| Order | Phase | Platforms | Depends on | Basis |
| --- | --- | --- | --- | --- |
| 1 | Live rendering | Linux | The proof of concept, and the publication of `slango` and `slang-slint` | Owner |
| 2 | Development GUI application with Slint | Linux | Phase 1 | Owner |
| 3 | Motif authoring: the colour sampler and its first media | As released | Phase 2 | Owner for the phase, proposal for its position and dependency |
| 4 | Interactive tile reshaping by orbifold deformations | As released | Phase 2 and the deformation reference (Section 5) | Owner for the phase, proposal for its position and dependencies |
| 5 | Truncated tilings by adjustable parameters | As released | Phase 2 and the truncation reference (Section 5) | Owner for the phase, proposal for its reading, position, and dependencies |
| 6 | First release | Linux, macOS, and Windows | Phase 2 | Owner for the phase and for its order after phase 2, proposal for its order after phases 3 to 5 |
| 7 | Animation with keyframes and Slang expressions of time | As released | Phase 1 and the animation reference (Section 5) | Owner for the phase and for its order "later down the roadmap", proposal for its position among phases 3 to 6 and its dependencies |
| 8 | Video input and video output | As released | Phases 3 and 7 | Owner for video output after stills and for video as a medium, proposal for one phase and its dependencies |
| 9 | Other outputs | As released | Phase 8 and the shader export reference (Section 5) | Owner for the phase as a possibility after video, proposal for its dependencies |
| 10 | Later release | Adds Android and iOS | Phase 6 | Owner for the phase and for its order after phase 6, proposal for its order after phases 7 to 9 |

These points are open for the owner:

- Where the first release sits relative to phases 3 to 5. The table lists it after them, and its only dependency is phase 2.
- What "truncated versions of tilings" means (Section 3.5).
- The order of the media of a motif (Section 3.3).

### 3.1. Live Rendering

This phase renders on the GPU with wgpu and Slang shaders, through the crates `slango` and `slang-slint` of the sibling project slang-rust, once those crates are published. That much is the owner's decision. Until then the engine renders to PNG files on disk.

Section 16.4 of `rust-ecosystem.md` recommends Slint 1.18.1 with its feature `unstable-wgpu-30` and wgpu 30.0.2. Section 16.3 records that this feature "may be removed or changed in future minor releases". A survey's recommendation is not a decision.

The planning proposes these points:

- The CPU renderer of the proof of concept stays the reference that GPU output is compared against.
- The precision design starts from Section 4.8 of `rendering.md` and Section 9.5 of `symmhub.md`. A group element computed in double precision on the CPU moves the view near the domain, so that only a small offset is evaluated in single precision on the GPU.
- Every test that needs a GPU is tagged (TEST-3 of `docs/rust-coding-standards.md`) and stays out of `just check`, which runs without a GPU or a display (TOOL-8 of `docs/design/tooling.md`).
- The backend abstraction between the CPU and GPU renderers is introduced in this phase, because a second backend exists only here (SIMP-2 of the coding standard).

### 3.2. Development GUI Application

This phase builds the Slint application on Linux, which is the owner's decision. The planning proposes signature entry, an info panel, parameter sliders, a colouring picker, a model switch, and view control (Section 4).

Slint is offered under `GPL-3.0-only` among other licences. Section 16.2 of `rust-ecosystem.md` concludes that the combined application is then effectively distributed under GPL version 3, while Escher's own source keeps its `-or-later` grant. That conclusion is inferred from the GNU FAQ, and Section 21 of the survey lists it as unverified. Adding Slint also adds its licence to `deny.toml` under TOOL-24 of the tooling design.

### 3.3. Motif Authoring

This phase lets the user draw a motif on one or more fundamental domains, depending on the chosen subgroup, and see it replicated live. That is the owner's decision. The owner abstracts the motif as a colour sampler. The planning proposes that the renderer folds each pixel into the fundamental domain and that the sampler gives the colour from the folded point and the coset of the tile. The proof of concept uses a solid colour for each coset instead, so that the mathematical and geometric foundations are settled first (Section 10.1 of `docs/design/architecture.md`).

The owner listed seven media that a sampler takes its colours from: hand-drawn raster, vector graphics, images, videos, procedural graphics, AI-generated graphics or style transfer, and a composite of these. Each arrives at its own stage, and the owner set no order among them. How each medium integrates into Escher is still to be explored. Gap 1 of Section 14 of `prior-art.md` records that no desktop tool lets the user paint a motif for an arbitrary orbifold signature. Section 10 of `rendering.md` surveys pattern fills, such as SymmHub's sampling of an image in a pattern box and Farris's wave functions.

The planning proposes this order of the media, from the one that needs the least new machinery to the one that needs the most: images, procedural graphics, hand-drawn raster, vector graphics, video, AI-generated graphics or style transfer, and composites. It also proposes that phase 3 delivers the sampler with the first two media, that video arrives in phase 8 because it needs frames over time, and that each other medium arrives as its own step.

The owner named integrations with external tools as a possibility for some media, with the examples of a Krita plugin for hand-painted graphics, a ComfyUI integration for AI, and a vector editor. Whether such an integration is worth its cost is undecided.

### 3.4. Interactive Tile Reshaping

This phase lets the user reshape the tiles interactively by applying orbifold deformations, which is the owner's decision. In the proof of concept each signature has named shape parameters with ranges, and `render` accepts `name=value` settings (Section 5.6 and `DEC-14` of `docs/design/architecture.md`). The planning proposes that this phase drives these parameters from the GUI and renders each change live.

Section 10 of `orbifold-notation.md` gives the dimension of the space of deformations of a hyperbolic signature, and the counts that "The Symmetries of Things" gives for four Euclidean signatures. Section 6 of `fundamental-domains.md` gives the deformation dimension, Maskit's Fenchel-Nielsen coordinates, which cuts carry a twist, and the cases of cone points, cusps, mirrors, and crosscaps. Section 4.2 of `export-and-animation.md` surveys the deformation of a group as a kind of animation.

### 3.5. Truncated Tilings

This phase lets the user make "truncated versions of tilings" with adjustable parameters, in the owner's words. The owner did not define "truncated".

The planning proposes to read it as the Wythoff construction with a movable generator point. In a reflection triangle, the position of the generator point selects the regular, truncated, rectified, and other uniform forms of the family (Sections 10.1 to 10.3 of `colourings.md`), and the adjustable parameter is that position. The owner confirms or replaces this reading before the design of the phase starts.

On this reading, the phase starts from Section 10 of `colourings.md`. Its Section 10.7 records that most of the construction rests on secondary sources only, and that no source gives the faces that each position produces or a per-pixel classification of the faces (items 10 and 11 of Section 14.1). Item 12 of that section records the construction beyond triangles as open.

### 3.6. First Release

This phase releases the application on Linux, macOS, and Windows. The planning proposes a CI matrix over the three platforms, a release procedure, a way for each crate to carry its licence text, and `cargo semver-checks` on the library crates. The release procedure and the licence text are the open questions TQ-4 and TQ-6 of `docs/design/tooling.md`.

The library crates are published only with the owner's explicit go-ahead. Section 14 of `prior-art.md` records, as gap 3, that no published Rust crate covers wallpaper or frieze groups, orbifold notation, or hyperbolic tilings.

### 3.7. Animation

This phase adds a keyframing system, which is the owner's decision. The owner wants "pretty much anything that can be animatable" to be animatable, and names deformation parameters, raster frames, vector animations, and procedural parameters.

The first way to configure a parameter that is a function of time is a mathematical expression, written as a Slang function. For a video, the parameter is the frame number. The phase starts from the animation reference of Section 5.

### 3.8. Video

This phase writes the result as video, which the owner placed after PNG stills, and takes video as a medium of a motif (Section 3.3). The planning proposes to deliver both in one phase, because both need frames over time, which phase 7 introduces. Section 4.4 of `export-and-animation.md` surveys the export of frames and their encoding.

### 3.9. Other Outputs

The owner names other outputs as a possibility after video, with the examples of a self-contained demo program and a module for a tool such as TouchDesigner. Which outputs Escher produces is undecided. The planning proposes that a self-contained program starts from the shader export reference of Section 5, because Section 5.1 of `export-and-animation.md` surveys what a self-contained tiling shader needs.

### 3.10. Later Release

This phase adds Android and iOS. No research document covers these platforms, and a deep-research item for them is a proposal.

## 4. Workstreams

These items are not tied to one phase. Each is a proposal of the planning unless the table marks it "Owner", and the dependencies in the third column are always proposals. Each item depends on the proof of concept, and the third column names any further dependency.

| Workstream | Starts from | Further dependency | Basis |
| --- | --- | --- | --- |
| The star, ring, and bent band models of the hyperbolic plane | Section 8.4 and item 15 of Section 13.1 of `rendering.md`, and item 22 of Section 15.1 of `geometry-models.md`. No source gives the inverses of these maps | The model maps of `M12` | Owner for the item, which is outside the proof of concept |
| The Gans and hemisphere models, after the three above | Item 16 of Section 13.1 of `rendering.md`, and item 6 of Section 15.1 of `geometry-models.md` | As above | Proposal |
| View control: rotation, shift, and zoom | None named | The command line, then phase 2 | Owner for the item, which is outside the proof of concept |
| Colourings of the faces of truncated tilings, for which a source covers only the perfect colourings of the regular and Laves tilings (Frettloh's lemmas) | Sections 5.2 to 5.5 and 10.7 of `colourings.md`, and item 5 of its Section 14.1 | Phase 5 | Proposal |
| Further colourings: larger colour counts and classification of two-colourings. Conjugate subgroups are colourings of their own from `M8` (`DEC-37`) | Items 8, 9, and 6 of Section 14.1 of `colourings.md` | `M8` | Proposal |
| Names of each group in other notations | Item 11 of Section 12 of `orbifold-notation.md`, which records the names that "The Symmetries of Things" by Conway, Burgiel, and Goodman-Strauss makes primary and the parts that stay open | A primary source for the open parts of item 11, such as the names of the friezes. `DEC-20` governs which names `info` shows | Proposal |
| A machine-readable form of the info output | `DEC-23` | None | Proposal |
| Rendering of the general non-compact signatures, which Section 2.1 of `docs/design/architecture.md` defines | The deep-research item of Section 5. Section 5.18 of `docs/design/poc-plan.md` records why the research knows no construction | `M18` | Owner for the item, which is outside the proof of concept (`DEC-19`) |
| Procedural textures from PDE simulation, built with the features of the application and not a core feature | Sections 4 and 10.2 of `pdes-on-orbifolds.md`, and Section 10 of `rendering.md`, which records SymmHub's reaction-diffusion simulation | Phase 3 with procedural graphics as a medium, and the PDE reference of Section 5 | Owner for the item and that it is not a core feature, proposal for the dependencies |

## 5. Deep Research

Each item below produces a subject-matter reference that follows Section 3.7 of the writing guide, or extends one, and the design cites the reference and not the survey. Section 1 of `export-and-animation.md` states that such a reference prevails over its survey.

The owner asked for a deep-research item for each topic that the research only surveyed. Three of these topics are in scope: animation, shader export, and PDEs on orbifolds. The general non-compact signatures have no survey, and their item follows from `DEC-19` and adds a sourced construction to the subject-matter references that the proof of concept uses. The items for orbifold deformations and for truncated tilings are proposals for phases 4 and 5.

Aperiodic tilings, vector export, and integrations with external tools have no item, because they are out of scope (Section 1).

The planning proposes this rule, and the last two columns of the table are proposals too. A design of a phase or workstream that the table names MUST NOT start before its deep-research item has produced or extended a reference, because the design would rest on claims that no one has checked.

| Topic | Basis | Research sections to start from | What the reference must settle first |
| --- | --- | --- | --- |
| Animation, for phase 7 | Owner for the item, proposal for its link to phase 7 | Section 4 of `export-and-animation.md` | Seamless loops, interpolation between isometries, band scrolling, deformation of a group, morphing between geometries, and frame timing (items 5 to 10 of Section 8.1) |
| Shader export, for the other outputs of phase 9 | Owner for the item, proposal for its link to phase 9 | Section 5 of `export-and-animation.md` | Fold bounds and single precision near the limit circle, Slang's loop and precision behaviour on its targets, and the published hyperbolic shaders (items 11 to 13 of Section 8.1) |
| PDEs on orbifolds, for procedural textures | Owner for the item, proposal for its link to the PDE workstream of Section 4 | Sections 4 and 10 of `pdes-on-orbifolds.md` | The error of symmetrization, ghost cells across glide-reflection and hyperbolic pairings, smoothness at a cone point, and a sourced time-step bound (items 1, 3, 4, and 2 of Section 10.2) |
| General non-compact signatures | Proposal, following `DEC-19` | Section 7 of `fundamental-domains.md` for Maskit's cusps, its Section 4.7 with gaps G1 and G6, item 15 of Section 12 of `orbifold-notation.md`, and item 3 of Section 13.1 of `rendering.md` | A construction of the fundamental domain and its side pairings for each such signature, the open points of the polygon theorem at ideal points (`UNV-5`), the geometry and parameters of these signatures (`UNV-21`), and the fold at ideal vertices (`UNV-23`) |
| Orbifold deformations, for phase 4 | Proposal | Section 10 of `orbifold-notation.md`, Section 6 of `fundamental-domains.md`, and Section 4.2 of `export-and-animation.md` | The deformations of the Euclidean signatures and whether their counts include the scale (item 7 of Section 12 of `orbifold-notation.md`), and the twists, which Deaton's thesis states, with a caveat on its numbering of the cuts (Section 6.3 and gap G4 of `fundamental-domains.md`) |
| Truncated tilings, on the reading of Section 3.5, for phase 5 | Proposal | Section 10 of `colourings.md` | Primary sources for the construction, the faces that each position of the generator point produces, per-pixel classification of the faces, and the construction beyond triangles (Section 10.7 and items 10 to 12 of Section 14.1) |

This roadmap uses Sections 4 and 5 of `export-and-animation.md`, on animation and shader export. It does not use Section 3 of that survey on vector export, and it does not use `aperiodic-tilings.md`.

## 6. Closing Unverified Statements

### 6.1. Reading Requested Sources

The planning proposes that an agent reads each source of `docs/research/source-requests.md` as the owner supplies it in `references/`, high priority first, as Section 2 of that file lists it. Section 8.15 of `fundamental-domains.md` records Deaton's thesis, which was read in full. It sources the cuts, the atoms, and the statement that a twist arises only for three cuts, on which the geometry of the general hyperbolic signatures with mirrors or cross-caps rests. The thesis numbers its two tables of cuts differently, so which three cuts carry a twist is a reader's reading. Gap G4 keeps open SymmHub's order of the cuts and its departures from the thesis.

No cited source states that the fold terminates or how many steps it takes (Section 4.7 of `rendering.md` and gap G5 of `fundamental-domains.md`). `UNV-1` and `UNV-2` therefore stay for the Lean workstream of Section 6.2.

The change that reads a source updates every document that waits on it, and the affected `UNV-n` rows of `docs/design/poc-plan.md`. The plan names the closing source of each statement. The rows of Section 7 of `source-requests.md`, sources already obtained, are cleared by `RP-5`.

### 6.2. Lean Workstream

The owner decided that the Lean development is not part of the proof of concept. It starts after the requested sources have been read and begins with what the fold depends on.

The planning proposes that it covers only the statements that still have no source, in this order:

1. `UNV-1` and `UNV-2`, the termination of the fold.
2. `UNV-26` and `UNV-14`, the statements on the linear model and the distance to a geodesic and to a great circle.
3. `UNV-17`, the coset colourings. It is group-theoretic except for two statements about the fold: that the sides of a fold, read as a word, give the element that maps the domain onto the tile that the fold reaches, and the rule that turns those sides into a coset.

Section 17 of `rust-ecosystem.md` gives the inputs. Section 17.3 infers that a `lean/` directory with its own `lakefile.toml` and `lean-toolchain` is the established layout. Section 17.4 records that Mathlib has no metric on the Poincare disk, no Fuchsian groups, no exchange condition, no spherical or hyperbolic law of cosines, and no Todd-Coxeter enumeration, so the development proves these itself.

Section 17.2 records that the Lean installer verifies no checksum and that the elan release publishes no checksum file. The owner decided that the container image pins the elan release asset by the SHA-256 digest that GitHub exposes for it. That change is made in the repository that defines the image (TOOL-28), and no recipe runs Lean until the image provides it (TOOL-9).

## 7. Security Considerations

This document defines no component, so its threat model concerns what the roadmap brings in. A dependency added by a phase, such as wgpu, Slint, or the slang-rust crates, may be malicious or compromised and runs at build time. A download, such as the Lean toolchain or a tool on a new CI runner, may be tampered with. A deep-research query leaves the machine and may disclose project details.

Each phase's design document owns its own threat model, so attacks on a component's input or output, such as a crafted image or video file that the user loads as a medium, are out of scope here. The mitigations this document relies on are the cargo-deny policy of Section 8 of the tooling design, the checksum rule TOOL-21 for every CI download, including those on macOS and Windows runners, and the pinned digest of Section 6.2. A deep-research item sends only generic questions, such as a library, a version, or a published result, and never project code.

The residual risk is a released build that still relies on an unverified statement, and an upstream whose release digest changes without notice, which stops the image build until an agent checks the new asset.

## 8. Operational Considerations

This document defines no Cargo feature and no environment variable. Each phase defines its own in its design document, with the default of each.

The supported platforms grow with phases 6 and 10. The tooling design runs CI on hosted Linux runners only, and `just check` stays free of GPU and display needs (TOOL-8) in every phase. A dependency that needs a newer Rust raises `rust-version` only in a change whose purpose is that raise (TOOL-5).

A phase that cannot start because a dependency is missing, such as unpublished slang-rust crates or a reference that does not exist yet, waits. No phase works around a missing dependency.

## 9. References

### 9.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.
