# Agent Instructions

Every coding agent reads this file before it works in this repository. `CLAUDE.md` imports it, so Claude Code loads the same instructions.

Escher is a Rust application for creating symmetry-based graphics, like those in some works of M. C. Escher, with the SymmHub project as prior art. It covers the planar symmetries that orbifold notation expresses, in Euclidean, spherical, and hyperbolic geometry. Its goal is a tool in which an artist draws a motif on one or more fundamental domains of a symmetry and sees it replicated live over the tiling, reshapes the tiles, animates the result, and renders it as raster images: first stills, then video, and possibly other outputs later. Tilings without a symmetry group, such as aperiodic tilings, are not a goal for now.

The repository holds scaffolding, the command-line tool `escher`, a library that parses and classifies orbifold signatures, a library of hyperbolic geometry, research documents, the planning documents of the proof of concept, and the design documents of its first two milestones, as Project Status states. The rest of this introduction states goals that the owner has set, and the tool meets none of them in full.

The owner has set these phases and platforms:

| Phase | Form | Platforms |
| --- | --- | --- |
| Proof of concept | Command-line tool | Linux |
| Development | Graphical user interface (GUI) application | Linux |
| First release | GUI application | Linux, macOS, and Windows |
| Later release | GUI application | Linux, macOS, Windows, Android, and iOS |

The owner has set these goals for the proof of concept:

- It produces PNG images of a symmetry given in orbifold notation, at a configurable resolution with a default of 512 by 512 pixels.
- It draws the polygons with black borders and fills a representative set of polygons with colours that the user chooses. The user can also set properties such as the border thickness.
- A separate command reports information on a symmetry: how many polygons are in the representative set, which says how many colours can be chosen, and whether the symmetry is spherical, Euclidean, or hyperbolic.
- It renders hyperbolic symmetries in the Poincare disk model first and by default, then in the half-plane, band, and Klein models, and after the proof of concept in other models, such as the star model that Vladimir Bulatov describes.
- It renders spherical symmetries with simple Phong shading.

Development happens inside the development container, whose image is defined outside this repository (Section 10 of `docs/design/tooling.md`).

Best Current Practice (BCP) 14 defines the key words in the next paragraph.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

## Hard Rules

1. **Documentation changes with the code, once.** In the same change as the code, the agent MUST update the design document of each component or tool that the change alters, and every statement in another document that the change would otherwise make false. The next agent reads the documentation as true and builds on it, so a false statement misleads it.

   `AGENTS.md`, `docs/design/tooling.md`, and `README.md` change only for such a statement, or when the change alters a rule, the repository layout, or what a user can do.

   The agent MUST NOT mirror a fact into several documents, because every copy can diverge. It states the fact once and cites that place. Documentation Maintenance lists what the agent re-reads.
2. **Documentation follows the writing guide.** The agent MUST write every document, and all application programming interface (API) documentation written with rustdoc, in accordance with `docs/writing-documentation.md`, because the agents and users that read a document depend on its conventions to interpret it. This includes the metadata rules for design documents and the API documentation rules of Section 7 of the writing guide. This file is an instruction file under Section 3.4 of that guide, so it has no metadata table or abstract, and its unnumbered headings are cited by name.
3. **Rust code follows the coding standard.** The agent MUST write all Rust code in accordance with `docs/rust-coding-standards.md`, because a reviewer checks every change against it and rejects code that breaks it. The standard has two kinds of rules.

   An agent learns the rules that a lint or tool enforces from the failing check, by reading the lint's row in Section 4.5 and following Section 4.4. The judgement rules, listed in Appendix A, are what an agent reads before it writes Rust.

   A reviewer checks each change against them and cites the rule identifier in each finding. Section 1.2 of the standard says which sections to read for a change.
4. **Deterministic checks belong to tools.** The agent MUST leave every check that a deterministic tool can perform to that tool, and MUST NOT send anything that such a tool can find to an agent or a large language model (LLM) for review. A reviewer misses findings that a tool reports every time and spends effort that only judgement can use.

   An agent that checks by hand something a tool could check adds the tool check in the same change, or proposes it in its report when it cannot add it. A review agent does not report a finding that a configured tool already reports.

   A new rule that a tool can check goes into the tool configuration, not into a list of judgement rules, as Section 16 of the coding standard and Section 3.5 of the writing guide require. A rule that a tool could check but that no configured tool checks yet stays a judgement rule until a tool covers it.
5. **Comments follow the writing guide.** Rust source contains only the comments that Section 7.2 of the writing guide lists, which are API documentation on public items and `# Safety` sections and `// SAFETY:` comments on unsafe code, and never a `TODO` or a `FIXME`. In the `justfile` and the scripts, the only comments are text addressed to a human, such as the help comment of a recipe or the usage text of a script. No comment cites a project document, as Hard Rule 15 states.
6. **Lint exceptions use `#[expect]`.** The agent MUST record every lint exception as `#[expect(lint, reason = "...")]` on the narrowest scope that needs it and MUST NOT use `#[allow]`, because an `#[allow]` hides the lint silently for as long as it stays, while a stale `#[expect]` fails the check.

   No agent weakens the lint or tool configuration to make a check pass. Rules LINT-1 and LINT-2 in Section 4.4 of the coding standard define the form of an exception and when the configuration may change. The `reason` gives the cause in plain words and names no rule identifier, as Hard Rule 15 requires.
7. **Soundness and security by default.** When two approaches are otherwise comparable, the agent MUST choose the sounder and more secure one and say so in its report, because unsound or insecure code that works is rarely revisited.

   The workspace denies `unsafe_code`, and no crate opts out. A crate that needs `unsafe` code records the reason in its design document first and then opts out as Section 4.4 of the coding standard states.

   No file in the repository contains a secret, and no workflow reads one (TOOL-22 of `docs/design/tooling.md`).

   A continuous integration (CI) workflow MUST verify the SHA-256 checksum of every download whose upstream publishes one, and MUST stop the job before it uses a file whose checksum differs, because a tampered file runs with the permissions of the job (TOOL-21). A workflow pins every action to a full commit SHA and grants its token read access only (TOOL-19 and TOOL-20). Section 7.2 of `docs/design/tooling.md` specifies these rules, and `check-workflow-pins` enforces the parts that a tool can check.
8. **Every action is a `just` recipe.** Every command, build step, and check is a recipe in the root `justfile`, which is one flat file with `[group]` attributes and no `just` modules, because modules hide their recipes from `just --list`. The agent invokes no script under `scripts/` directly when a recipe wraps it.

   Every recipe has a help comment, and every shell script starts with `#!/usr/bin/env bash` and `set -euo pipefail`. Every shell script and every recipe that contains shell is `shellcheck`-clean. Section 3.2 of `docs/design/tooling.md` lists every recipe, and Validating Changes names the recipes to run for each kind of change.
9. **No attribution.** No output contains a generated-by line, a `Co-Authored-By` trailer, or a link to an agent session, because such a line says nothing about the change and a session link can expose a private session. This covers commits, pull requests, issues, and files. The `commit-msg` hook rejects such lines, and `just merge` strips them.
10. **No extra documentation.** The agent writes no document beyond `README.md`, `AGENTS.md`, `CLAUDE.md`, and the `docs/` tree, with design documents in `docs/design/` and research documents in `docs/research/`, because every extra document is one more place that can diverge from the code.
11. **Work in a worktree.** Files change only in a git worktree under `.worktrees/`, never in the main checkout, because the main checkout is where `just merge` runs. Branches are named `feature/<short-description>`, lowercase and hyphenated, because the repository has no ticket keys.

   The agent commits without asking and pushes its branch to run CI, because the project opens no pull requests and a push to a branch that matches `feature/**` is what runs CI on a branch.

   The agent MUST run `just merge` only after the CI run for the last commit of the branch concluded `success` with every job succeeded, because a merge without that result can put a failing commit on `main`. A fresh branch that holds the tree of a verified branch counts as verified when that branch contains the current `main` (Section 14.7 of `docs/design/tooling.md`).

   `just merge` pushes `main` after the merge, and the agent MAY push `main` itself when the push did not happen. Agents open no pull requests and never force-push.
12. **Canadian English and ASCII punctuation.** All prose, including API documentation, commit messages, and recipe help text, uses Canadian English spelling and ASCII punctuation, as Section 8 of the writing guide defines. One spelling lets an agent find every occurrence of a word with one search.
13. **Consult the references before inventing a pattern.** When an agent is about to use a design pattern, or to make a design choice that the coding standard does not cover, it MUST look the topic up in `docs/rust-references.md` and read the sources starting at the highest tier. It MUST then extend the standard in the same change.

   A reviewer cannot check code against a rule that does not exist yet. Section 1.1 of the reference list gives the lookup steps, and Section 16 of the coding standard gives the steps to add a rule.
14. **Mathematics comes from cited sources.** The subject-matter references under `docs/research/` state the mathematics that the code implements. They follow Section 3.7 of the writing guide and the rules of its Sections 5.4 and 5.5. An implementer builds code on a formula in such a reference, so a claim that no source backs becomes a defect that no test reveals.

   The agent MUST NOT write an original proof or derivation in prose, because prose has no source or machine check to verify it against. Each result that the project needs is cited to a source that proves it, proved in a machine-checked Lean development, or recorded as unverified in the Open Questions and Unverified Claims section of the reference. When a proof needs Lean and Lean is not installed, the agent installs it or asks the owner to install it.

   When the agent cannot read a source, it records the source in `docs/research/source-requests.md` and continues the work, because a missing source MUST NOT block the research. The agent MUST NOT commit anything from `references/`, because the repository has no right to redistribute it (Section 14.8 of `docs/design/tooling.md`).
15. **Code cites no project document.** The agent MUST NOT cite, in a code file or a comment, a project document, which is a Markdown file under `docs/`, a section of one, or a rule identifier that a project document defines, such as `TOOL-14` or `API-3`. A reader of the code or of a published crate sees it without the repository's documents, and a citation breaks when a document changes.

   The rule covers API documentation, ordinary comments, the `reason` strings of lint exceptions, error and panic messages, recipe help comments, script usage text, and comments in configuration files. Code also MUST NOT hold project history wording, such as "this version", a milestone, or a step.

   Where a citation would have carried a reason, the code MUST state the reason itself in plain words. The documents cite the code, never the reverse.

   The checker `check-doc-references` reports the path or file name of a project document, a section cited by number, and a rule identifier that a project document defines. `just check` and the `pre-commit` hook run it. No tool checks history wording, so that part stays a judgement rule.

## Project Status

As of 2026-11-16, the repository holds its scaffolding, the package `escher-cli` with the command-line tool `escher`, the library crates `escher-signature` and `escher-geometry`, the research documents under `docs/research/`, the planning documents of the proof of concept, and the design documents of milestones `M1` and `M2`. The tool reports on a signature and draws no image. The agent updates this section, and its date, in the same change that alters what exists.

What exists and works:

- The scaffolding that Repository Layout lists: the standards documents, the tooling design, the tool configuration, the `justfile` and its recipes, the custom checkers and their self-tests, the git hooks, the Claude Code settings, and the CI workflow.
- The workspace member `crates/escher-cli`, whose binary `escher` has the command `escher info <SIGNATURE>`. It prints the canonical form of the signature, its geometry, its Euler characteristic, whether it is compact, and, where they apply, whether it is a frieze or general non-compact, its group order, its area, and its deformation dimension. A malformed signature or a bad orbifold gives a diagnostic that locates the offending characters and exit status 1, and a usage error gives exit status 2. `escher --version` prints `escher-cli 0.1.0`. The library target exposes only `run` and `Status`, as `docs/design/cli.md` specifies.
- The workspace member `crates/escher-signature`, a library that parses a signature in orbifold notation into a `Signature` that is valid and canonical by construction, prints its canonical form, gives its exact Euler characteristic, and classifies it. The classification gives the geometry, whether the signature is compact, a frieze, or general non-compact, and the group order, area, and deformation dimension where `docs/design/signature.md` defines them. `escher info` uses the crate.
- The workspace member `crates/escher-geometry`, a library that holds the linear model of the hyperbolic plane over the `glam` types: the bilinear form, the side test, point normalization, distance, the wall through two points, the basic isometries and their inverse, an isometry check, and the map to and from the Poincare disk. `docs/design/geometry.md` states the accuracy of each function, measured against high-precision reference arithmetic. No other crate uses it yet.
- The `[workspace.dependencies]` table, with `thiserror` and the dev-dependency `proptest`, which `escher-signature` and `escher-geometry` use, `glam` and the dev-dependency `astro-float-num`, which `escher-geometry` uses, and `anyhow`, `clap`, `escher-signature`, and the dev-dependency `insta`, which `escher-cli` uses.
- The tests of `escher-geometry`: unit tests and one integration-test binary, with tests against the reference arithmetic, property tests, and deterministic sweeps. `check-miri` skips the reference tests and the sweeps, which run loops too long for the Miri interpreter, except the self-check of the reference arithmetic and one check of the disk map.
- The tests of `escher-signature`: unit tests and one integration-test binary, with property tests and enumeration tests. `check-miri` skips the enumeration tests, which run a loop too long for the Miri interpreter.
- The tests of `escher-cli`: one integration-test binary that calls `run` with in-memory streams, with `insta` snapshot tests of the information text, the diagnostics, and the help and usage text, and plain tests of the statuses and of failed writes. `check-miri` skips the snapshot tests, because they open files that the isolation of Miri refuses.
- `just check`, `just check-features`, `just check-msrv`, and `just coverage`, which all pass in the development container. `just check` runs the tests and doctests of the three crates, and runs under Miri the tests that do not skip themselves.
- The twelve research documents under `docs/research/` that Repository Layout lists: seven subject-matter references, four surveys, and the working list `docs/research/source-requests.md`.
- The planning documents of the proof of concept: `docs/design/architecture.md`, `docs/design/poc-plan.md`, and `docs/design/roadmap.md`.
- The design documents of milestone `M1`: `docs/design/signature.md` for the crate `escher-signature`, and `docs/design/cli.md` for the crate `escher-cli` and the command `escher info`.
- The design documents of milestone `M2`: `docs/design/geometry.md` for the crate `escher-geometry`, `docs/design/domain.md` for the planned crate `escher-domain`, and the extension of `docs/design/cli.md` with the domain summary and the option `--check` of `escher info`. `docs/design/cli.md` therefore specifies lines and an option that the binary does not have yet.

The research documents record what their cited sources establish, including the sources that the owner supplied in `references/`, and mark each claim that no source establishes as unverified. Each of them except `docs/research/source-requests.md` has an Open Questions and Unverified Claims section, which names what would close each question. An agent reads that section before it builds on a claim, because a claim listed there is not established.

What does not exist yet:

- Any code that computes a tiling or renders an image.
- The rest of milestone `M2`: the crate `escher-domain`, and the domain summary and the option `--check` of `escher info`.
- Any command of `escher` other than `info`, such as `render`.
- The design document of each component other than `escher-signature`, `escher-cli`, `escher-geometry`, and `escher-domain`, which its milestone writes before the code of the component.
- A Lean development.

The owner reviewed the plan of the proof of concept on 2026-11-16 and decided `DEC-3`, `DEC-4`, `DEC-5`, `DEC-18`, and `DEC-19`, which Section 10.1 of `docs/design/architecture.md` states. The other proposed defaults stand until the owner changes them. The owner stated the scope after the proof of concept on 2026-11-16, and `docs/design/roadmap.md` records it with the proposals that the owner has yet to confirm. Milestone `M1` is complete: its design documents, the crate `escher-signature`, and the command `escher info` exist, and every test that its field "Tests and acceptance" names exists and passes. Milestone `M2` is in progress: its design documents and the crate `escher-geometry` exist, and the crate `escher-domain` and the new lines and option of `escher info` do not. No other milestone has started. The change that starts or completes a milestone updates this section, and no design document records that progress.

Section 7.11 of the writing guide and Section 4.8 of the coding standard say which part of each rule a tool or checker enforces. Every rule that no configured tool checks stays a judgement rule.

The agent MUST NOT assume that a planned crate, recipe, file, or tool exists, because an agent that builds on something absent produces code that cannot work.

## Repository Layout

- `AGENTS.md`: This file.
- `CLAUDE.md`: A one-line file that imports `AGENTS.md` for Claude Code.
- `Cargo.toml`: The workspace manifest, with resolver 3, the members `crates/escher-cli`, `crates/escher-geometry`, and `crates/escher-signature`, the `[workspace.package]` fields that every member inherits, such as edition 2024, `rust-version`, and the licence, the `[workspace.dependencies]` table with `anyhow`, `astro-float-num`, `clap`, `escher-signature`, `glam`, `insta`, `proptest`, and `thiserror`, the workspace lint tables of Section 4.2 of the coding standard, and the table `[profile.dev.package.astro-float-num]`, which optimizes that dev-dependency in the `dev` profile (Section 11 of `docs/design/geometry.md`).
- `Cargo.lock`: The lock file of the workspace.
- `LICENSE`: The text of version 3 of the GNU General Public License.
- `README.md`: The README, which is also the readme of `escher-cli`, `escher-geometry`, and `escher-signature` through the inherited `readme` field.
- `crates/escher-cli/Cargo.toml`: The manifest of the package `escher-cli`, which sets `publish = false`, inherits the workspace fields and lints, declares the binary `escher`, depends on `anyhow`, `clap`, and `escher-signature`, has the dev-dependency `insta`, and declares no Cargo feature.
- `crates/escher-cli/src/`: The library target `lib.rs` with `run` and `Status`, its private modules `args.rs`, `info.rs`, and `diagnostic.rs`, and the binary `escher` in `main.rs`, which only calls `run`, as Section 3 of `docs/design/cli.md` specifies.
- `crates/escher-cli/tests/cli/`: The one integration-test binary of the crate, `main.rs`, with the test modules and the shared helpers in `common.rs` (TEST-10 of the coding standard), and the committed `insta` snapshot files in `snapshots/`.
- `crates/escher-geometry/Cargo.toml`: The manifest of the package `escher-geometry`, which sets `publish = false`, inherits the workspace fields and lints, depends on `glam` and `thiserror`, has the dev-dependencies `astro-float-num` and `proptest`, and declares no Cargo feature.
- `crates/escher-geometry/src/`: The library, with the modules that Section 3.3 of `docs/design/geometry.md` lists and their unit tests.
- `crates/escher-geometry/tests/geometry/`: The one integration-test binary of the crate, `main.rs`, with the test modules, the reference arithmetic in `reference.rs`, and the shared helpers in `common.rs` (TEST-10 of the coding standard).
- `crates/escher-signature/Cargo.toml`: The manifest of the package `escher-signature`, which sets `publish = false`, inherits the workspace fields and lints, depends on `thiserror`, has the dev-dependency `proptest`, and declares no Cargo feature.
- `crates/escher-signature/src/`: The library, with the modules that Section 3.3 of `docs/design/signature.md` lists and their unit tests.
- `crates/escher-signature/tests/signature/`: The one integration-test binary of the crate, `main.rs`, with the test modules and the shared helpers in `common.rs` (TEST-10 of the coding standard).
- `clippy.toml`: The Clippy configuration of Section 4.3 of the coding standard.
- `deny.toml`: The cargo-deny configuration of Section 8 of `docs/design/tooling.md`, which also checks dev-dependencies.
- `.config/nextest.toml`: The nextest profiles `default` and `default-miri` of Section 4 of `docs/design/tooling.md`.
- `taplo.toml`: The taplo configuration, with four-space indentation and the exclusion of `target/` and `.worktrees/`.
- `typos.toml`: The typos configuration, which accepts correct text that typos would otherwise report, as a pattern, words, and identifiers (TOOL-30 of `docs/design/tooling.md`).
- `docs/writing-documentation.md`: The normative writing guide.
- `docs/rust-coding-standards.md`: The normative Rust coding standard, with the specification of the lint and tool configuration.
- `docs/rust-references.md`: The reference list of external sources behind the coding standard, with a topic index.
- `docs/design/tooling.md`: The tooling design, which is authoritative for the toolchain configuration, the recipes, the hooks, the custom checkers, the CI workflow, the dependency policy, and the requirements on the container image.
- `docs/design/architecture.md`: The architecture design of the proof of concept, which is authoritative for the crates of the proof of concept, the data flow, the conventions between crates, the testing strategy, performance, numerical robustness, the owner's decisions, and the proposed defaults.
- `docs/design/poc-plan.md`: The milestone plan of the proof of concept, which is authoritative for the workflow of a milestone, the owner's staging, the research prerequisites, the milestones, the documentation obligations of each milestone, and the unverified statements that the proof of concept relies on.
- `docs/design/roadmap.md`: The roadmap design, which is authoritative for the order of the phases after the proof of concept, the workstreams, the deep-research items, and the closing of unverified statements by sources and by a Lean development.
- `docs/design/signature.md`: The design of the crate `escher-signature`, which is authoritative for its public surface, the input grammar of a signature, parsing and its errors, the canonical form, the exact Euler characteristic, the classification, and the limits on untrusted input.
- `docs/design/cli.md`: The design of the package `escher-cli`, which is authoritative for the public surface of its library target, the binary `escher`, the commands and options, the information text of `escher info` with the domain summary and the verification lines of `--check`, the diagnostics and the replacement of control characters, the exit statuses, the tests and the form of a snapshot, the behaviour of `insta` under nextest, and the dependencies of the package.
- `docs/design/geometry.md`: The design of the crate `escher-geometry`, which is authoritative for its public surface over the `glam` types, the conventions and sources of each formula of the hyperbolic linear model, the disk map, numerical robustness and accuracy, the error type, the tests with reference arithmetic, and the dependencies.
- `docs/design/domain.md`: The design of the planned crate `escher-domain`, which is authoritative for its public surface, the signatures that have a fundamental domain, the construction and placement of the hyperbolic reflection triangle, its measurements, the checks of its verification with their tolerances, numerical accuracy, the error type and its text, the limit on the corner order, the tests, and the dependencies.
- `docs/research/orbifold-notation.md`: The subject-matter reference on Conway's orbifold notation, with the grammar of a signature, equivalent signatures, the Euler characteristic, the classification by geometry, the lists of spherical, wallpaper, and frieze signatures, presentations, and deformations.
- `docs/research/geometry-models.md`: The subject-matter reference on the models of the hyperbolic plane, the sphere, and the Euclidean plane, with the conversions between models, isometries, constructions, numerical robustness, distances, and trigonometry.
- `docs/research/fundamental-domains.md`: The subject-matter reference on building a fundamental polygon and its side-pairing isometries from a signature, with Poincare's polygon theorem, moduli, building blocks, algorithms, and the classes of signature that have a sourced construction.
- `docs/research/rendering.md`: The subject-matter reference on drawing a tiling into a raster image, with per-pixel folding, forward generation, borders, antialiasing, per-pixel model maps, the sphere, pattern fills, and performance.
- `docs/research/colourings.md`: The subject-matter reference on tilings associated with a group and colourings that respect their symmetry, with three candidate designs for the polygons of an image.
- `docs/research/symmhub.md`: The subject-matter reference that studies the SymmHub source code as prior art.
- `docs/research/prior-art.md`: The subject-matter reference that surveys existing software for symmetry-based graphics.
- `docs/research/rust-ecosystem.md`: The survey of the Rust crates and tools that the project could use, in the skeleton of a subject-matter reference. Its recommendations are not decisions.
- `docs/research/aperiodic-tilings.md`: The survey of tilings without a symmetry group, such as the Penrose tilings and the hat and spectre monotiles, with their construction methods, rendering, and existing software. The roadmap does not use it, because aperiodic tilings are out of scope for now.
- `docs/research/pdes-on-orbifolds.md`: The survey for the roadmap of the simulation of partial differential equations on orbifolds, with the candidate numerical methods, pattern formation on curved surfaces, and GPU practice.
- `docs/research/export-and-animation.md`: The survey of vector export, animation, and shader export, with Slang as an export target. The roadmap uses its parts on animation and shader export, and Escher has no vector output.
- `docs/research/source-requests.md`: The working list of sources that the research could not read in full, by priority, for the owner to supply in `references/` (Section 5.5 of the writing guide).
- `justfile`: The root `justfile`, with the groups `checks`, `rust`, and `git` (Section 3.1 of `docs/design/tooling.md`).
- `.github/workflows/ci.yml`: The CI workflow, with the jobs `check`, `ci-only`, and `coverage` on hosted Linux runners, which Section 7 of `docs/design/tooling.md` specifies.
- `scripts/check-prose`, `scripts/prose-wordlist.txt`: The prose checker for Markdown files and for the documentation comments of Rust files, and its list of non-Canadian spellings.
- `scripts/check-rust-comments`: The checker that Rust source holds only API documentation and safety comments.
- `scripts/check-doc-headings`: The checker of the headings in item documentation.
- `scripts/check-doctest-attributes`: The checker that no documentation code block is marked `ignore`.
- `scripts/check-feature-docs`: The checker that the crate-level documentation of each library names every Cargo feature.
- `scripts/check-workflow-pins`: The checker that every action is pinned by a full commit SHA, and that no workflow uses `pull_request_target` or grants more than read access at its top level.
- `scripts/check-doc-references`: The checker that code and configuration cite no project document, section, or rule identifier (Hard Rule 15). It reads the rule identifiers from the tables of the documents under `docs/` on every run.
- `scripts/rust_lexer.py`: The Python module that the custom checkers share, which reads Rust tokens and documentation comments.
- `scripts/self-test-checker`: The runner of the self-test of each custom checker.
- `scripts/fixtures/`: The cases of each self-test, one directory per checker and per case, with input files that end in `.fixture`.
- `scripts/check-shell`, `scripts/justfile-recipes`: The `shellcheck` run over every shell script and every recipe.
- `scripts/check-commit-message`: The commit message checker.
- `scripts/claude-hook-prose`: The Claude Code hook that runs the prose checker after an edit to a Markdown file or a Rust file.
- `scripts/merge-branch`: The squash merge that `just merge` runs.
- `scripts/worktree-prune`: The cleanup of squash-merged branches that `just worktree-prune` and the `post-merge` hook run.
- `.pre-commit-config.yaml`: The `pre-commit`, `commit-msg`, and `post-merge` hooks.
- `.claude/settings.json`: The Claude Code settings, with the commands that run without asking and the hook that runs the prose checker after every edit (Section 5 of `docs/design/tooling.md`).
- `.editorconfig`: The character set, line endings, final newline, and indentation for each file type.
- `.gitattributes`: The single line `* text=auto eol=lf`, which keeps LF line endings in every checkout on every platform (Section 14.3 of `docs/design/tooling.md`).
- `.gitignore`: The files that git ignores, including the pending snapshot files `*.snap.new` and `*.pending-snap` of `insta`.

`target/`, `.worktrees/`, `.code-review/`, `.idea/`, and `.claude/settings.local.json` are local and ignored by git. `references/` is also ignored. It may exist in a checkout to hold copyrighted sources that the owner supplies, and nothing in it is ever committed (Hard Rule 14).

These locations are planned and do not exist yet:

- `docs/design/group.md` and `docs/design/render.md`: The design documents of the other components, which Section 1.5 and `DEC-1` of `docs/design/architecture.md` name.
- `crates/escher-group`, `crates/escher-domain`, and `crates/escher-render`: The planned crates, which Section 3 of `docs/design/architecture.md` specifies.

## Technology Decisions

`docs/design/tooling.md` is the design for the toolchain configuration, the recipes, the hooks, the CI workflow, and the requirements on the container image, and it prevails over this section on those topics.

These decisions are in force:

- Development uses the latest stable Rust toolchain. The workspace declares the minimum supported Rust version (MSRV) 1.99 in `rust-version`, and `check-msrv` verifies it with cargo-hack. The nightly toolchain serves only checkers such as Miri and never builds the crates.
- The crates use edition 2024 and Cargo resolver 3. The licence is GPL-3.0-or-later, and `LICENSE` holds its text.
- The workspace denies `unsafe_code` (Hard Rule 7).
- CI runs on GitHub Actions on hosted Linux runners, with the jobs of Section 7.1 of `docs/design/tooling.md`.
- `.config/nextest.toml` defines the profiles `default`, for `test` and `coverage`, and `default-miri`, which Miri uses. Neither has a filter, so each selects every test, and a test that is too slow for Miri skips itself there (TEST-11 of the coding standard), as does a snapshot test (Section 8.3 of `docs/design/cli.md`). `just check` passes without a graphics processing unit (GPU) or a display (TOOL-8).
- No crate is published, and `escher-cli`, `escher-geometry`, and `escher-signature` set `publish = false`. Nothing is published without the owner's explicit go-ahead.

The development container provides this environment:

- The stable Rust toolchain, the default, with Clippy, rustfmt, and llvm-tools.
- A nightly toolchain with Miri and rust-src, used only for checkers.
- cargo-nextest, cargo-deny, cargo-insta, cargo-semver-checks, cargo-hack, cargo-machete, cargo-llvm-cov, typos, taplo, actionlint, `just`, pre-commit, shellcheck, the GitHub command-line tool `gh`, which the verification procedure of Section 14.7 of `docs/design/tooling.md` uses, and Python 3.
- GitHub's Secure Shell (SSH) host keys in the known hosts file.

CI installs the latest version of each tool on its runners, so a tool version in CI can differ from the one in the container (Section 7.1 of `docs/design/tooling.md`). Section 4.8 of the coding standard lists what each tool checks, and its Section 1.1 records the versions that were verified. Section 10 of `docs/design/tooling.md` lists every requirement on the image.

The owner has made these decisions for the proof of concept and the later phases. The workspace has started to grow into several crates, as Project Status states, and none of the other decisions is implemented yet:

- The workspace grows into several crates, so that each crate holds one concern.
- Until live rendering exists, the engine renders to PNG files on disk.
- Slint provides the user interface of the GUI application.
- Live rendering of graphics and animations uses Slang, through the crates `slango` and `slang-slint` of the sibling project slang-rust, once those crates are published.
- The proof of concept fills polygons with solid colours only.
- All rendering is raster and per pixel. Escher has no vector output and no forward rendering path.
- Aperiodic tilings are out of scope for now.

Section 3 of `docs/design/roadmap.md` states the decisions for the later phases, marks which of them are the owner's, and puts them in order. Section 10.1 of `docs/design/architecture.md` records the owner's decisions for the proof of concept, and its Section 10.2 records the proposed defaults.

## Development Workflow

A change starts with the section of the design document that covers it. A change that adds a component without a design document writes that document first, in `docs/design/`, and a change that implements mathematics starts from its subject-matter reference (Hard Rule 14). Work on the proof of concept follows the milestones of `docs/design/poc-plan.md` in the order of their dependencies, and each milestone begins with its design branch, as Section 3 of that document states.

Before it writes Rust, the agent reads the sections of the coding standard that Section 1.2 of that standard names. The agent makes the change in a worktree, as Hard Rule 11 requires, runs the checks in Validating Changes, and fixes whatever they report. The agent updates the documentation as Hard Rule 1 requires before it considers the change done.

`just hooks-install` installs the git hooks once per clone. The `pre-commit` hooks run the checks that do not compile the workspace, the `commit-msg` hook runs `check-commit-message`, and the `post-merge` hook runs `just worktree-prune`. Section 5 of `docs/design/tooling.md` lists each hook and when it runs.

CI runs the workflow `.github/workflows/ci.yml` on every push to `main`, every push to a branch that matches `feature/**`, and every pull request. A new push to the same feature branch cancels the earlier run, and no run on `main` is cancelled. Every check in every job is a `just` recipe, so a failure in CI has the same command to reproduce it in the container. Section 7.1 of `docs/design/tooling.md` specifies the jobs.

Commits are small, and each holds one group of logically related changes, because a later reader reads history one commit at a time. The subject line is in the imperative mood, such as `Add the PNG renderer`, and the body says why the change was made.

`just merge BRANCH` squash-merges the branch onto `main` from the main checkout with the subject and body of every commit in the message. It then removes the worktree and the branch, pushes `main`, and deletes the branch on `origin` when its tip there is part of the squash. `just worktree-prune` removes branches that were squash-merged some other way.

The agent verifies a branch in CI before it merges the branch, with the procedure of Section 14.7 of `docs/design/tooling.md`. That procedure reads the CI run itself and never the exit status of a piped command, and it merges a branch with several work-in-progress commits through a fresh branch without rewriting pushed history. A job that GitHub cancelled because no runner acquired it is rerun, and the merge waits for the rerun (Section 14.4 of `docs/design/tooling.md`).

The agent MUST fix every review finding labelled Important before the merge, because such a finding is a defect that would reach `main`. The agent defers the nits and lists them in its report.

## Validating Changes

The agent runs the checks that match what changed, and `just check` before every merge. Section 3.2 of `docs/design/tooling.md` lists every recipe, and its Section 14.4 says what a failure of each one means and what to do.

To run every check:

```sh
just check
```

Section 3.3 of `docs/design/tooling.md` lists the recipes that `just check` runs, in order. It needs network access, because `check-deny` fetches the advisory database on every run, and the nightly toolchain, because `check-miri` runs Miri on it (Section 14.6 of `docs/design/tooling.md`).

To run only the Rust checks, which are `check-rust-comments`, `check-doc-headings`, `check-doctest-attributes`, `check-feature-docs`, `check-fmt`, `check-unused-deps`, `check-deny`, `check-clippy`, `check-doc`, `check-doctest`, `test`, and `check-miri`, in that order:

```sh
just check-rust
```

To format every Rust file that git tracks or does not ignore, and every TOML file, which fixes what `check-fmt` and `check-toml` report:

```sh
just fmt
```

To run the lints and the tests after a change to Rust code or its API documentation:

```sh
just check-clippy
just check-doc
just check-doctest
just test
```

When a snapshot test fails because the change to the output is intended, `insta` writes the new snapshot beside the committed one as a `.snap.new` file. Review and accept it with this command, which no recipe wraps (Section 8.2 of `docs/design/cli.md`), and commit the updated `.snap` file:

```sh
cargo insta review
```

`just test` captures the output of a test that passes, so it hides the largest errors that the accuracy tests of `escher-geometry` print. To print them, with the margin and the worst input of each bound of Sections 8.4 and 10.3 of `docs/design/geometry.md`, run this command, which no recipe wraps. A filter such as `-E 'test(wall_through_close)'` selects one test:

```sh
cargo nextest run -p escher-geometry --no-capture
```

To check the dependencies after a change to a manifest:

```sh
just check-deny
just check-unused-deps
```

To run the tests under Miri after a change to unsafe code, run this recipe. The profile `default-miri` selects every test, and a test that calls a native library or starts a process fails under Miri. A test that runs a loop too long for the interpreter, or a snapshot test, is skipped there, and property tests run fewer cases (Section 4 of `docs/design/tooling.md`):

```sh
just check-miri
```

To check the spelling of every file, and the formatting of every TOML file:

```sh
just check-typos
just check-toml
```

To check the prose of a Markdown file, or of the documentation comments of a Rust file, run this command. The Claude Code hook also runs it after every edit of a Markdown file or a Rust file:

```sh
just check-prose docs/rust-coding-standards.md
```

To check the comments, the headings of item documentation, the doctest attributes, and the feature documentation after a change to Rust code or its API documentation, run these recipes. Each also takes the paths of files to check, and `check-feature-docs` takes the paths of Cargo manifests:

```sh
just check-rust-comments
just check-doc-headings
just check-doctest-attributes
just check-feature-docs
```

To check a workflow after a change under `.github/`:

```sh
just check-workflows
just check-workflow-pins
```

To check that code and configuration cite no project document, section, or rule identifier, run this recipe after a change to code, configuration, a script, the `justfile`, or a rule table of a document. It also takes the paths of files to check:

```sh
just check-doc-references
```

To run the self-test of a checker after a change to the checker, to `scripts/rust_lexer.py`, or to its cases under `scripts/fixtures/`:

```sh
just check-prose-self-test
just check-rust-comments-self-test
just check-doc-headings-self-test
just check-doctest-attributes-self-test
just check-feature-docs-self-test
just check-workflow-pins-self-test
just check-doc-references-self-test
```

To check a shell script, or a recipe in the `justfile`:

```sh
just check-shell
```

To check `scripts/check-commit-message`:

```sh
just check-commit-message-self-test
```

Three more recipes are not part of `just check`, because they are meant for CI, and CI runs all three in every run of the workflow:

- `check-features` compiles the workspace once for each combination of Cargo features.
- `check-msrv` checks the workspace with the toolchain that `rust-version` names, and its first run installs that toolchain with rustup, which needs network access.
- `coverage` writes a coverage report to `target/lcov.info` and prints no coverage summary, and the CI job `coverage` uploads that report as an artifact.

An agent runs `check-features` after a change to the Cargo features and `check-msrv` after a change that may need a newer toolchain:

```sh
just check-features
just check-msrv
just coverage
```

## Documentation Maintenance

Hard Rule 1 is the rule an agent is most likely to skip, because the code works without it. After every change, and before it reports the change complete, the agent MUST re-read and correct these documents, once each, because a stale document misleads the next agent:

1. The design document of each component or tool that the change alters, including its design content, its assumptions, and its open questions. Progress and status are recorded only in Project Status of this file, never in a design document (Section 3.6 of the writing guide).
2. Every statement in another document that the change makes false, which the agent finds by searching for the names that the change touches. `README.md` is also the readme of every member of the workspace.
3. `docs/rust-coding-standards.md` with its Appendix A, `docs/rust-references.md`, or `docs/writing-documentation.md`, only when the change alters a rule, a lint, a tool, a source, or a documentation convention.
4. The Date field of the metadata table of each touched document that has one.

## Contribution Policy

The project is developed by its owner with coding agents, under the rules in this file. It has no contribution policy yet and opens no pull requests for now. A pull request from a fork runs CI with a read-only token and no secrets. A future contributing policy may require disclosure of artificial intelligence (AI) usage in pull request descriptions.

## References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.
