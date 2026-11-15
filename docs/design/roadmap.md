# Roadmap Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-15 |
| Applies to | The development of Escher after the proof of concept: its phases, workstreams, research, and the closing of unverified statements |

## Abstract

This document orders the development of Escher after the proof of concept. It lists the phases in the order the owner set, from live rendering to releases on five platforms, then vector graphics, other planar tilings, and the simulation of partial differential equations on orbifolds. For each phase and each smaller workstream it states what the work depends on and which research it starts from. It defines the deep-research items that turn each surveyed topic into a subject-matter reference before its design starts, and the order in which sources and a Lean development close the unverified statements. It designs none of this work.

## 1. Introduction

The proof of concept is a Linux command-line tool that renders Portable Network Graphics (PNG) images, as `docs/design/poc-plan.md` plans it. The owner has set goals well beyond it, and some of them rest only on surveys, which are not implementation references. Without a stated order, an agent could start a design that its research does not yet support, or build on a phase that does not exist.

This document gives that order. Its readers are the agents that plan the next piece of work and the owner, who decides it. It states order and dependencies only. It holds no dates, estimates, or status, as Section 3.6 of `docs/writing-documentation.md` requires, and each design it names is written later in its own design document.

`docs/design/architecture.md` owns the crates, the owner's decisions (its Section 10.1), and the proposed defaults `DEC-1` to `DEC-27` (its Section 10.2). `docs/design/poc-plan.md` owns the milestones `M1` to `M19`, the research prerequisites `RP-1` to `RP-5`, and the unverified statements `UNV-1` to `UNV-28`. This document cites those labels and does not restate them.

Research documents are cited by file name, and each is in `docs/research/`.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

- phase: A unit of work that the owner has placed in the order of Section 3.
- workstream: A unit of work that no single phase contains, listed in Section 4.
- survey: A research document that maps a topic for the roadmap and is not an implementation reference, such as `aperiodic-tilings.md`.
- deep-research item: A research task that turns a survey, or part of one, into a subject-matter reference (Section 5).
- owner's decision: A statement that the owner made and that this document records. A table marks it "Owner".
- proposal: A statement of this document or of the planning behind it that the owner has not decided. A table marks it "Proposal", and prose says "the planning proposes".

The abbreviations used are central processing unit (CPU), continuous integration (CI), GNU General Public License (GPL), graphics processing unit (GPU), graphical user interface (GUI), partial differential equation (PDE), Portable Document Format (PDF), and Scalable Vector Graphics (SVG).

## 3. Phases

The owner set the order of the platform phases 1 to 4, and the order of the capability phases 5 to 7 that follow.

One point is open for the owner. Does vector graphics (phase 5) wait for the later release (phase 4), or may it start directly after the proof of concept? The table lists phase 5 after phase 4 and records no dependency between them.

| Order | Phase | Platforms | Depends on | Basis |
| --- | --- | --- | --- | --- |
| 1 | Live rendering | Linux | The proof of concept, and the publication of `slango` and `slang-slint` | Owner |
| 2 | Development GUI application with Slint | Linux | Phase 1 | Owner |
| 3 | First release | Linux, macOS, and Windows | Phase 2 | Owner |
| 4 | Later release | Adds Android and iOS | Phase 3 | Owner |
| 5 | Vector graphics | As released | The vector export reference | Owner for the phase, proposal for the dependency |
| 6 | Other planar tilings, such as aperiodic tilings, in the medium term | As released | The aperiodic tilings reference | Owner for the phase, proposal for the dependency |
| 7 | Simulation of PDEs on orbifolds, with Slang compute shaders, in the long term | As released | Phase 1 and the PDE reference | Owner for the phase, proposal for the dependencies |

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

### 3.3. First Release

This phase releases the application on Linux, macOS, and Windows. The planning proposes a CI matrix over the three platforms, a release procedure, a way for each crate to carry its licence text, and `cargo semver-checks` on the library crates. The release procedure and the licence text are the open questions TQ-4 and TQ-6 of `docs/design/tooling.md`.

The library crates are published only with the owner's explicit go-ahead. Section 14 of `prior-art.md` records, as gap 3, that no published Rust crate covers wallpaper or frieze groups, orbifold notation, or hyperbolic tilings.

### 3.4. Later Release

This phase adds Android and iOS. No research document covers these platforms, and a deep-research item for them is a proposal.

### 3.5. Vector Graphics

This phase exports tilings as vector graphics. It starts from forward generation in Section 5 of `rendering.md` and from enumeration without duplicates in Section 3.6 of `export-and-animation.md`. Section 13 of `rust-ecosystem.md` lists candidate crates, and its Section 13.1 recommends `kurbo`, SVG written directly or with `svg`, and `pdf-writer` for PDF. No source gives a rule for when to stop emitting tiles (item 1 of Section 8.1 of `export-and-animation.md`).

### 3.6. Other Planar Tilings

This phase adds tilings without a symmetry group, such as the Penrose tilings and the hat and spectre monotiles. Section 3 of `aperiodic-tilings.md` explains why this is a different problem from the rest of Escher. Section 6.2 records per-pixel point location for the hat or the spectre as not established, from a limited search. Section 9 lists Simon Tatham's Rust crate `substitution-tiling-transducers`, which covers the Penrose, hat, and spectre tilings.

### 3.7. Simulation of PDEs on Orbifolds

This phase simulates planar PDEs on orbifolds with Slang compute shaders, which is the owner's decision. Section 4 of `pdes-on-orbifolds.md` compares the families of methods. Section 10.2 records that the research found no source for two points. These are an error analysis of covering-space simulation by group averaging or folding (item 1), and the filling of ghost cells across glide-reflection or hyperbolic side pairings (item 3).

## 4. Workstreams

These items are not tied to one phase. Each is a proposal of the planning unless the table marks it "Owner", and the dependencies in the third column are always proposals. Each item depends on the proof of concept, and the third column names any further dependency.

| Workstream | Starts from | Further dependency | Basis |
| --- | --- | --- | --- |
| The star, ring, and bent band models of the hyperbolic plane | Section 8.4 and item 15 of Section 13.1 of `rendering.md`, and item 22 of Section 15.1 of `geometry-models.md`. No source gives the inverses of these maps | The model maps of `M12` | Owner for the item, which moved out of the proof of concept |
| The Gans and hemisphere models, after the three above | Item 16 of Section 13.1 of `rendering.md`, and item 6 of Section 15.1 of `geometry-models.md` | As above | Proposal |
| View control: rotation, shift, and zoom | None named | The command line, then phase 2 | Owner for the item, which moved out of the proof of concept |
| Animation | The animation reference (Section 5) | Phase 1 | Owner for the research item, proposal for the workstream |
| Shader export | The shader export reference (Section 5) | Phase 1 | Owner for the research item, proposal for the workstream |
| Wythoff tilings as a second tiling mode, and colourings of regular and Laves tilings by Frettloh's lemmas | Sections 10 and 12 and items 10 to 12 of Section 14.1 of `colourings.md`, and its Sections 5.2 to 5.5 | `M8` | Proposal |
| Further colourings: conjugate subgroups, larger colour counts, and classification of two-colourings | Items 1, 8, 9, and 6 of Section 14.1 of `colourings.md` | `M8` | Proposal |
| Names of each group in other notations | Item 11 of Section 12 of `orbifold-notation.md` | Reading "The Symmetries of Things" by Conway, Burgiel, and Goodman-Strauss (Section 6.1). `DEC-20` omits these names from `info` until a primary source is read | Proposal |
| Pattern fills and motifs | Section 10 of `rendering.md`, and gaps 1 and 4 of Section 14 of `prior-art.md` | None | Proposal |
| A machine-readable form of the info output | `DEC-23` | None | Proposal |

## 5. Deep Research

The owner asked for a deep-research item for each of the five topics below. Each item produces a subject-matter reference that follows Section 3.7 of the writing guide, and the design cites the reference and not the survey. Section 1 of `export-and-animation.md` states that such a reference prevails over its survey.

The planning proposes this rule, and the columns of the table are proposals too. A design of animation, shader export, vector export, aperiodic tilings, or PDEs on orbifolds MUST NOT start before its deep-research item has produced a reference, because the design would rest on a survey whose claims no one has checked.

| Topic | Survey sections to start from | What the reference must settle first |
| --- | --- | --- |
| Aperiodic tilings | Sections 6, 9, 10, and 12.1 of `aperiodic-tilings.md` | Per-pixel point location for the hat and the spectre, the tile count per substitution level, and precision at deep zoom (items 1, 2, and 4 of Section 12.1) |
| PDEs on orbifolds | Sections 4 and 10 of `pdes-on-orbifolds.md` | The error of symmetrization, ghost cells across glide-reflection and hyperbolic pairings, smoothness at a cone point, and a sourced time-step bound (items 1, 3, 4, and 2 of Section 10.2) |
| Vector export | Section 3 of `export-and-animation.md` | The stopping rule, the numerics of nearly straight SVG arcs, the shapes of geodesics under each projection, and how other tools stop (items 1 to 4 of Section 8.1) |
| Animation | Section 4 of `export-and-animation.md` | Seamless loops, interpolation between isometries, band scrolling, deformation of a group, morphing between geometries, and frame timing (items 5 to 10 of Section 8.1) |
| Shader export | Section 5 of `export-and-animation.md` | Fold bounds and single precision near the limit circle, Slang's loop and precision behaviour on its targets, and the published hyperbolic shaders (items 11 to 13 of Section 8.1) |

## 6. Closing Unverified Statements

### 6.1. Reading Requested Sources

The planning proposes that an agent reads each source of `docs/research/source-requests.md` as the owner supplies it in `references/`, high priority first. In the order of Section 2 of that file, these are "The Symmetries of Things" by Conway, Burgiel, and Goodman-Strauss, then Ratcliffe, Deaton, Beardon, Singerman, and Epstein and Petronio.

The change that reads a source updates every document that waits on it, and the affected `UNV-n` rows of `docs/design/poc-plan.md`. The plan names the closing source of each statement. The rows of Section 7 of `source-requests.md`, sources already obtained, are cleared by `RP-5`.

### 6.2. Lean Workstream

The owner decided that the Lean development is not part of the proof of concept. It starts after the requested sources have been read and begins with what the fold depends on.

The planning proposes that it covers only the statements that still have no source, in this order:

1. `UNV-1` and `UNV-2`, the termination of the fold.
2. `UNV-26` and `UNV-14`, the statements on the linear model and the distance to a geodesic and to a great circle.
3. `UNV-17`, the coset colourings, which is purely group-theoretic.

Section 17 of `rust-ecosystem.md` gives the inputs. Section 17.3 infers that a `lean/` directory with its own `lakefile.toml` and `lean-toolchain` is the established layout. Section 17.4 records that Mathlib has no metric on the Poincare disk, no Fuchsian groups, no exchange condition, no spherical or hyperbolic law of cosines, and no Todd-Coxeter enumeration, so the development proves these itself.

Section 17.2 records that the Lean installer verifies no checksum and that the elan release publishes no checksum file. The owner decided that the container image pins the elan release asset by the SHA-256 digest that GitHub exposes for it. That change is made in the repository that defines the image (TOOL-28), and no recipe runs Lean until the image provides it (TOOL-9).

## 7. Security Considerations

This document defines no component, so its threat model concerns what the roadmap brings in. A dependency added by a phase, such as wgpu, Slint, or the slang-rust crates, may be malicious or compromised and runs at build time. A download, such as the Lean toolchain or a tool on a new CI runner, may be tampered with. A deep-research query leaves the machine and may disclose project details.

Each phase's design document owns its own threat model, so attacks on a component's input or output are out of scope here. The mitigations this document relies on are the cargo-deny policy of Section 8 of the tooling design, the checksum rule TOOL-21 for every CI download, including those on macOS and Windows runners, and the pinned digest of Section 6.2. A deep-research item sends only generic questions, such as a library, a version, or a published result, and never project code.

The residual risk is a released build that still relies on an unverified statement, and an upstream whose release digest changes without notice, which stops the image build until an agent checks the new asset.

## 8. Operational Considerations

This document defines no Cargo feature and no environment variable. Each phase defines its own in its design document, with the default of each.

The supported platforms grow with phases 1 to 4. The tooling design runs CI on hosted Linux runners only, and `just check` stays free of GPU and display needs (TOOL-8) in every phase. A dependency that needs a newer Rust raises `rust-version` only in a change whose purpose is that raise (TOOL-5).

A phase that cannot start because a dependency is missing, such as unpublished slang-rust crates or a reference that does not exist yet, waits. No phase works around a missing dependency.

## 9. References

### 9.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.
