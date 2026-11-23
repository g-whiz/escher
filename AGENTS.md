# Agent Instructions

Every coding agent reads this file before it works in this repository. `CLAUDE.md` imports it, so Claude Code loads the same instructions.

Escher is a Rust application for creating symmetry-based graphics, like those in some works of M. C. Escher, with the SymmHub project as prior art. It covers the planar symmetries that orbifold notation expresses, in Euclidean, spherical, and hyperbolic geometry. The goal is a tool in which an artist draws a motif on one or more fundamental domains of a symmetry, sees it replicated live over the tiling, reshapes the tiles, animates the result, and renders it as raster images, first stills and then video. Tilings without a symmetry group, such as aperiodic tilings, are not a goal for now.

The repository holds the command-line tool `escher`, five library crates behind it, a test support crate, research documents, and design documents. Project Status states what exists. The goals below are the owner's, and the tool meets none of them in full.

The owner has set these phases and platforms:

| Phase | Form | Platforms |
| --- | --- | --- |
| Proof of concept | Command-line tool | Linux |
| Development | Graphical user interface (GUI) application | Linux |
| First release | GUI application | Linux, macOS, and Windows |
| Later release | GUI application | Linux, macOS, Windows, Android, and iOS |

The owner's goals for the proof of concept:

- PNG images of a symmetry given in orbifold notation, at a configurable resolution with a default of 512 by 512 pixels.
- Polygons with black borders, a representative set of them filled with colours that the user chooses, and settings such as the border thickness.
- A separate command that reports how many polygons the representative set has, which is how many colours can be chosen, and whether the symmetry is spherical, Euclidean, or hyperbolic.
- Hyperbolic symmetries in the Poincare disk model first and by default, then in the half-plane, band, and Klein models, and after the proof of concept in other models, such as Vladimir Bulatov's star model.
- Spherical symmetries with simple Phong shading.

Development happens inside the development container, whose image is defined outside this repository (Section 10 of `docs/design/tooling.md`).

Best Current Practice (BCP) 14 defines the key words in the next paragraph.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

## Hard Rules

1. **Documentation changes with the code, once.** In the same change as the code, the agent MUST update the design document of each component or tool that the change alters, and every statement in another document that the change would otherwise make false. The next agent reads the documentation as true and builds on it, so a false statement misleads it.

   `AGENTS.md`, `docs/design/tooling.md`, and `README.md` change only for such a statement, or when the change alters a rule, the repository layout, or what a user can do. In `AGENTS.md`, Project Status and Repository Layout keep the short form that those sections define.

   The agent MUST NOT mirror a fact into several documents, because every copy can diverge. It states the fact once and cites that place. Documentation Maintenance lists what the agent re-reads.
2. **Documentation follows the writing guide.** The agent MUST write every document, and all application programming interface (API) documentation written with rustdoc, in accordance with `docs/writing-documentation.md`, because the agents and users that read a document depend on its conventions to interpret it. This includes the metadata rules for design documents and the API documentation rules of Section 7 of the writing guide. This file is an instruction file under Section 3.4 of that guide, so it has no metadata table or abstract, and its unnumbered headings are cited by name. Every document and all API documentation also follow `docs/avoiding-claudelish.md`, and `README.md` also follows `docs/writing-readmes.md`.
3. **Rust code follows the coding standard.** The agent MUST write all Rust code in accordance with `docs/rust-coding-standards.md`, because a reviewer checks every change against it and rejects code that breaks it. The standard has two kinds of rules.

   An agent learns a rule that a lint or tool enforces from the failing check, by reading the lint's row in Section 4.5 and following Section 4.4. The judgement rules, listed in Appendix A, are what an agent reads before it writes Rust. A reviewer checks each change against them and cites the rule identifier in each finding. Section 1.2 of the standard says which sections to read for a change.
4. **Deterministic checks belong to tools.** The agent MUST leave every check that a deterministic tool can perform to that tool, and MUST NOT send anything that such a tool can find to an agent or a large language model (LLM) for review. A reviewer misses findings that a tool reports every time and spends effort that only judgement can use.

   An agent that checks by hand something a tool could check adds the tool check in the same change, or proposes it in its report when it cannot add it. A review agent does not report a finding that a configured tool already reports.

   A new rule that a tool can check goes into the tool configuration, not into a list of judgement rules, as Section 16 of the coding standard and Section 3.5 of the writing guide require. A rule that a tool could check but that no configured tool checks yet stays a judgement rule until a tool covers it. Section 7.11 of the writing guide and Section 4.8 of the coding standard say which part of each rule a tool enforces.
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
10. **No extra documentation.** The agent writes no document beyond `README.md`, `AGENTS.md`, `CLAUDE.md`, and the `docs/` tree, with design documents in `docs/design/` and research documents in `docs/research/`, because every extra document is one more place that can diverge from the code.11. **Work in a worktree.** Files change only in a git worktree under `.worktrees/`, never in the main checkout, because the main checkout is where `just merge` runs. Branches are named `feature/<short-description>`, lowercase and hyphenated, because the repository has no ticket keys.

   The agent commits without asking and pushes its branch to run CI, because the project opens no pull requests and a push to a branch that matches `feature/**` is what runs CI on a branch.

   The agent MUST run `just merge` only after the CI run for the last commit of the branch concluded `success` with every job succeeded, because a merge without that result can put a failing commit on `main`. A fresh branch that holds the tree of a verified branch counts as verified when that branch contains the current `main` (Section 14.7 of `docs/design/tooling.md`).

   `just merge` pushes `main` after the merge, and the agent MAY push `main` itself when the push did not happen. Agents open no pull requests and never force-push.
12. **Canadian English and ASCII punctuation.** All prose, including API documentation, commit messages, and recipe help text, uses Canadian English spelling and ASCII punctuation, as Section 8 of the writing guide defines. One spelling lets an agent find every occurrence of a word with one search.
13. **Consult the references before inventing a pattern.** When an agent is about to use a design pattern, or to make a design choice that the coding standard does not cover, it MUST look the topic up in `docs/rust-references.md`, read the sources starting at the highest tier, and then extend the standard in the same change. A reviewer cannot check code against a rule that does not exist yet. Section 1.1 of the reference list gives the lookup steps, and Section 16 of the coding standard gives the steps to add a rule.
14. **Mathematics comes from cited sources.** The subject-matter references under `docs/research/` state the mathematics that the code implements. They follow Section 3.7 of the writing guide and the rules of its Sections 5.4 and 5.5. An implementer builds code on a formula in such a reference, so a claim that no source backs becomes a defect that no test reveals.

   The agent MUST NOT write an original proof or derivation in prose, because prose has no source or machine check to verify it against. Each result that the project needs is cited to a source that proves it, proved in a machine-checked Lean development, or recorded as unverified in the Open Questions and Unverified Claims section of the reference. When a proof needs Lean and Lean is not installed, the agent installs it or asks the owner to install it.

   When the agent cannot read a source, it records the source in `docs/research/source-requests.md` and continues the work, because a missing source MUST NOT block the research. The agent MUST NOT commit anything from `references/`, because the repository has no right to redistribute it (Section 14.8 of `docs/design/tooling.md`).
15. **Code cites no project document.** The agent MUST NOT cite, in a code file or a comment, a project document, which is a Markdown file under `docs/`, a section of one, or a rule identifier that a project document defines, such as `TOOL-14` or `API-3`. A reader of the code or of a published crate sees it without the repository's documents, and a citation breaks when a document changes.

   The rule covers API documentation, ordinary comments, the `reason` strings of lint exceptions, error and panic messages, recipe help comments, script usage text, and comments in configuration files. Code also MUST NOT hold project history wording, such as "this version", a milestone, or a step. Where a citation would have carried a reason, the code MUST state the reason itself in plain words. The documents cite the code, never the reverse.

   The checker `check-doc-references` reports the path or file name of a project document, a section cited by number, and a rule identifier that a project document defines. `just check` and the `pre-commit` hook run it. No tool checks history wording, so that part stays a judgement rule.

## Project Status

As of 2026-11-23, the tool `escher` has three commands:

- `escher info` reports on a signature: its canonical form, geometry, Euler characteristic, classification, the Schlafli symbols of its regular tiling and of the dual where the signature has one, and, for a signature with a fundamental domain, its shape parameters, a summary of the domain, and the count of its valid colourings, those in which no two tiles that share a side have the same colour, with one for each conjugacy class. `--colourings` lists them by id, `--presentation` prints the group presentations, and `--check` prints the verification of the domain.
- `escher render` writes the tiling of a domain to a PNG file: a hyperbolic domain in the Poincare disk, a Euclidean domain in the plane, and a spherical domain as a lit ball. `--model` draws a hyperbolic domain in the half-plane, the band, or the Klein disk instead, and `--border-unit intrinsic` measures the border width in the geometry of the tiling instead of in pixels. By default it draws the valid colouring with 2 colours where the domain has one, and one colour otherwise. It can instead draw a colouring given by its id, in default or user-given colours, and `--set` sets the shape parameters. `--colours` also accepts `image:FILE` items, which fill the tiles of their colour with a PNG image, such as a painted mask of that colour. On a reflection triangle, `--wythoff` draws the faces of the Wythoff construction that three digits select, one for each corner, with the borders on the edges of the faces and one colour for each face type.
- `escher render-masks` writes, for a signature and a colouring, one PNG mask for each colour: a flat polygon that stands for the tile, with a coloured border on each edge that shows which edges can meet.

`info` and `render` support these signatures, with corner orders up to 100:

- Hyperbolic: the compact reflection triangles `*pqr` and the compact reflection polygons with four to eight corners, such as `*3333`, and the rotation signature of each, such as `732` and `3333`.
- Euclidean: all 17 groups.
- Spherical: every spherical signature, which is `*332`, `*432`, `*532`, `*n22`, `*nn`, the rotation signature of each, `3*2`, `2*n`, `n*`, `nx`, `*`, `x`, and `1`, with orders up to 100, and up to 50 for `n` in `2*n`.

`render-masks` and the image fills of `render` support the same signatures, except those whose domain has no mask polygon, which Section 5.22 of `docs/design/render.md` lists.

`docs/design/cli.md` specifies the commands, and `docs/design/domain.md` lists the supported signatures and their parameters.

| Crate | Purpose | Design document |
| --- | --- | --- |
| `escher-cli` | The binary `escher` and its commands | `docs/design/cli.md` |
| `escher-signature` | Parses, prints, and classifies a signature in orbifold notation | `docs/design/signature.md` |
| `escher-geometry` | The hyperbolic, Euclidean, and spherical geometry over the `glam` types | `docs/design/geometry.md` |
| `escher-group` | Group presentations, coset enumeration, and the low-index search for colourings | `docs/design/group.md` |
| `escher-domain` | Builds and verifies a fundamental domain with its side pairings and side presentation | `docs/design/domain.md` |
| `escher-render` | Renders the tiling of a domain per pixel and encodes it as PNG | `docs/design/render.md` |
| `escher-reference` | Test support: the shared reference arithmetic of the accuracy tests | `docs/design/architecture.md` |

Milestones `M1` to `M13` and `M20` to `M23` of `docs/design/poc-plan.md` are complete, and no other milestone has started. Section 3.4 of that document gives the order of the rest.

What does not exist yet:

- Every command of `escher` other than `info`, `render`, and `render-masks`.
- The design document of each component other than the crates above, which its milestone writes before the code of the component.
- A Lean development.

Section 10.1 of `docs/design/architecture.md` records the owner's decisions, and its Section 10.2 the proposed defaults, which stand until the owner changes them. `docs/design/roadmap.md` records the scope after the proof of concept.

Each research document records what its sources establish. Each one except `docs/research/source-requests.md` has an Open Questions and Unverified Claims section, which an agent reads before it builds on a claim, because a claim listed there is not established.

The agent MUST NOT assume that a planned crate, recipe, file, or tool exists, because an agent that builds on something absent produces code that cannot work.

The change that alters what exists updates this section in the same change: the date, the statement of what the tool does, the milestone line, the crate table, and the list of what does not exist. Progress goes only here, never into a design document. This section holds no inventory of tests, snapshots, golden images, or files, because the code and the design documents hold them.

## Repository Layout

| Path | Contents |
| --- | --- |
| `AGENTS.md`, `CLAUDE.md` | These instructions, and the one-line file that imports them for Claude Code |
| `README.md` | The README, also the readme of every crate |
| `LICENSE` | The GNU General Public License, version 3 |
| `Cargo.toml`, `Cargo.lock` | The workspace manifest, with the shared package fields, dependencies, and lint tables, and its lock file. `Cargo.toml` sets the optimization levels of the `dev` profile (Section 4 of `docs/design/tooling.md`), and `Cargo.lock` holds `flate2` at 1.1.9 for the reason that Section 8 of that document states |
| `crates/` | The seven crates of Project Status |
| `docs/` | The standards, design documents, and research documents |
| `scripts/` | The custom checkers and the scripts that recipes run |
| `justfile` | Every recipe, in the groups `checks`, `rust`, and `git` |
| `.github/workflows/ci.yml` | The CI workflow, with the jobs `check`, `ci-only`, and `coverage` |
| `clippy.toml`, `deny.toml`, `taplo.toml`, `typos.toml` | The configuration of Clippy, cargo-deny, taplo, and typos |
| `.config/nextest.toml` | The nextest profile `default` |
| `.pre-commit-config.yaml` | The `pre-commit`, `commit-msg`, and `post-merge` hooks |
| `.claude/settings.json` | The Claude Code settings, with the commands that run without asking and the prose hook |
| `.editorconfig`, `.gitattributes`, `.gitignore` | Editor settings, LF line endings, and ignored files |

Each crate `crates/escher-<name>/` has a `Cargo.toml` with `publish = false`, a `src/` directory with the modules that Section 3 of its design document lists, and one integration-test binary in `tests/<name>/`, with `main.rs` and the shared helpers in `common.rs`. The test support crate `escher-reference` has only `src/lib.rs` and no integration-test binary.

The documents:

| Document | Authoritative for |
| --- | --- |
| `docs/writing-documentation.md` | The writing guide |
| `docs/avoiding-claudelish.md` | The prose habits of unedited LLM output, and the rules that keep them out |
| `docs/writing-readmes.md` | The content, order, and links of `README.md` |
| `docs/rust-coding-standards.md` | The Rust coding standard and the lint and tool configuration |
| `docs/rust-references.md` | The external sources behind the coding standard |
| `docs/design/tooling.md` | The toolchain, recipes, hooks, checkers, CI, dependency policy, and container image |
| `docs/design/architecture.md` | The crates, data flow, conventions between crates, owner's decisions, and proposed defaults |
| `docs/design/poc-plan.md` | The milestones of the proof of concept and their workflow |
| `docs/design/roadmap.md` | The phases after the proof of concept |
| `docs/design/signature.md`, `cli.md`, `geometry.md`, `group.md`, `domain.md`, `render.md` | One crate each, as the crate table of Project Status shows |
| `docs/research/` | The subject-matter references, the surveys, and the list of sources to supply, `source-requests.md` |
| `docs/images/` | The images that `README.md` shows, which `just readme-images` writes |

The scripts under `scripts/`:

- The checkers `check-prose` with `prose-wordlist.txt`, `check-rust-comments`, `check-doc-headings`, `check-doctest-attributes`, `check-feature-docs`, `check-workflow-pins`, `check-doc-references`, and `check-commit-message`, with the shared lexer `rust_lexer.py`.
- `self-test-checker` and the cases in `fixtures/`, for the self-tests of the checkers.
- `check-shell` and `justfile-recipes`, for `shellcheck`.
- `claude-hook-prose`, `merge-branch`, `readme-images`, and `worktree-prune`.

`target/`, `.worktrees/`, `.code-review/`, `.idea/`, `.claude/settings.local.json`, and `renders/` are local and ignored by git. `references/` is also ignored. It may hold copyrighted sources that the owner supplies, and nothing in it is ever committed (Hard Rule 14).

A change updates this section only when it adds, removes, or renames a top-level file or directory, a document, a script, or a part of the common crate layout. It adds no list of modules, tests, or dependencies, because the code and the manifests hold them.

## Technology Decisions

`docs/design/tooling.md` prevails over this section on the toolchain, the recipes, the hooks, CI, and the container image.

These decisions are in force:

- Development uses the latest stable Rust toolchain. The workspace declares the minimum supported Rust version (MSRV) 1.99 in `rust-version`, and `check-msrv` verifies it with cargo-hack. The crates build on stable toolchains only, and DEP-4 of the coding standard states which tools may need another.
- The crates use edition 2024 and Cargo resolver 3. The licence is GPL-3.0-or-later.
- The workspace denies `unsafe_code` (Hard Rule 7).
- CI runs on GitHub Actions on hosted Linux runners, with the jobs of Section 7.1 of `docs/design/tooling.md`.
- `.config/nextest.toml` defines one profile, `default`, with no filter. `just check` passes without a graphics processing unit (GPU) or a display (TOOL-8).
- Every crate sets `publish = false`. Nothing is published without the owner's explicit go-ahead.

The development container provides:

- The stable Rust toolchain, the default, with Clippy, rustfmt, and llvm-tools.
- cargo-nextest, cargo-deny, cargo-insta, cargo-semver-checks, cargo-hack, cargo-machete, cargo-llvm-cov, typos, taplo, actionlint, `just`, pre-commit, shellcheck, the GitHub command-line tool `gh`, and Python 3.
- GitHub's Secure Shell (SSH) host keys in the known hosts file.

CI installs the latest version of each tool, so a version in CI can differ from the one in the container. Section 4.8 of the coding standard lists what each tool checks, and Section 10 of `docs/design/tooling.md` lists every requirement on the image.

The owner's decisions for the product, of which only those on the user interface and live rendering are not implemented yet:

- The workspace grows into several crates, each with one concern.
- Until live rendering exists, the engine renders to PNG files on disk.
- Slint provides the user interface of the GUI application.
- Live rendering uses Slang, through the crates `slango` and `slang-slint` of the sibling project slang-rust, once those crates are published.
- The proof of concept fills polygons with solid colours, except for the image fills of milestones `M22`, `M24`, `M25`, and `M26` of `docs/design/poc-plan.md`.
- All rendering is raster and per pixel. Escher has no vector output and no forward rendering path.
- Aperiodic tilings are out of scope for now.

Section 3 of `docs/design/roadmap.md` states the decisions for the later phases.

## Development Workflow

1. Start from the section of the design document that covers the change. A new component gets its design document in `docs/design/` first, and mathematics starts from its subject-matter reference (Hard Rule 14). Work on the proof of concept follows the milestones of `docs/design/poc-plan.md` in dependency order, and each milestone begins with its design branch (Section 3 of that document).
2. Before writing Rust, read the sections of the coding standard that its Section 1.2 names.
3. Make the change in a worktree (Hard Rule 11), run the checks in Validating Changes, and fix what they report.
4. Update the documentation as Hard Rule 1 and Documentation Maintenance require.
5. Commit in small commits, each with one group of related changes, because a later reader reads history one commit at a time. The subject line is in the imperative mood, such as `Add the PNG renderer`, and the body says why.
6. Push the branch, and verify it in CI with the procedure of Section 14.7 of `docs/design/tooling.md`. That procedure reads the CI run itself, never the exit status of a piped command, and merges a branch with several work-in-progress commits through a fresh branch without rewriting pushed history. A job that GitHub cancelled because no runner acquired it is rerun, and the merge waits for the rerun (Section 14.4).
7. The agent MUST fix every review finding labelled Important before the merge, because such a finding is a defect that would reach `main`. Defer the nits and list them in the report.
8. Run `just merge BRANCH` from the main checkout. It squash-merges the branch onto `main` with the subject and body of every commit, removes the worktree and the branch, pushes `main`, and deletes the branch on `origin` when its tip there is part of the squash.

`just hooks-install` installs the git hooks once per clone. The `pre-commit` hooks run the checks that do not compile the workspace, `commit-msg` runs `check-commit-message`, and `post-merge` runs `just worktree-prune`, which removes branches that were squash-merged some other way (Section 5 of `docs/design/tooling.md`).

CI runs `.github/workflows/ci.yml` on every push to `main` or to a `feature/**` branch, and on every pull request. A new push to a feature branch cancels its earlier run, and no run on `main` is cancelled. Every check in CI is a `just` recipe, so a CI failure reproduces in the container with the same command.

## Validating Changes

Run the checks that match the change, and `just check` before every merge. `just check` runs the recipes of Section 3.3 of `docs/design/tooling.md` in order and needs network access, because `check-deny` fetches the advisory database (Section 14.6). Section 3.2 lists every recipe, and Section 14.4 says what each failure means.

| When | Command |
| --- | --- |
| Before every merge | `just check` |
| Rust checks only: comments, headings, doctest attributes, feature docs, format, dependencies, Clippy, docs, doctests, tests | `just check-rust` |
| Format Rust and TOML files | `just fmt` |
| Rust code or API documentation changed | `just check-clippy`, `just check-doc`, `just check-doctest`, `just test` |
| Rust code or API documentation changed, for the comment, heading, doctest attribute, and feature documentation checks (each takes file paths, and `check-feature-docs` takes manifests) | `just check-rust-comments`, `just check-doc-headings`, `just check-doctest-attributes`, `just check-feature-docs` |
| A manifest changed | `just check-deny`, `just check-unused-deps` |
| Spelling, and TOML formatting | `just check-typos`, `just check-toml` |
| Prose of a Markdown file or of Rust doc comments (a Claude Code hook also runs it after each edit) | `just check-prose <path>` |
| A workflow under `.github/` changed | `just check-workflows`, `just check-workflow-pins` |
| Code, configuration, a script, the `justfile`, or a rule table of a document changed | `just check-doc-references` |
| A shell script or a recipe changed | `just check-shell` |
| An `escher render` or `escher render-masks` command of `README.md`, or the output of either command, changed (commit the changed images) | `just readme-images` |
| A checker, `scripts/rust_lexer.py`, or its cases under `scripts/fixtures/` changed | `just check-<checker>-self-test`, for `prose`, `rust-comments`, `doc-headings`, `doctest-attributes`, `feature-docs`, `workflow-pins`, and `doc-references` |
| `scripts/check-commit-message` changed | `just check-commit-message-self-test` |
| Cargo features changed | `just check-features` |
| The change may need a newer toolchain | `just check-msrv` |
| A coverage report is wanted (it goes to `target/lcov.info`, with no summary) | `just coverage` |

`check-features`, `check-msrv`, and `coverage` are not part of `just check`, and CI runs all three in every run. The first run of `check-msrv` installs its toolchain with rustup, which needs network access.

When a snapshot test fails because the output change is intended, `insta` writes a `.snap.new` file beside the committed one. Review and accept it with this command, which no recipe wraps (Section 8.2 of `docs/design/cli.md`), and commit the updated `.snap` file:

```sh
cargo insta review
```

`just test` hides the largest errors that the accuracy tests of `escher-geometry`, `escher-domain`, and `escher-render` print. This command, which no recipe wraps, prints them with the margin and worst input of each bound in the design documents of those crates. A filter such as `-E 'test(wall_through_close)'` selects one test:

```sh
cargo nextest run -p escher-geometry -p escher-domain -p escher-render --no-capture
```

When a golden image test of `escher-render` fails because the image change is intended, inspect the actual and difference images that it writes, run `just update-golden`, which overwrites each committed golden image with the rendered one, and commit the changed PNG files (Section 14.9 of `docs/design/tooling.md`).

## Documentation Maintenance

Hard Rule 1 is the rule an agent is most likely to skip, because the code works without it. After every change, and before it reports the change complete, the agent MUST re-read and correct these documents, once each, because a stale document misleads the next agent:

1. The design document of each component or tool that the change alters, including its design content, assumptions, and open questions. Progress and status go only into Project Status of this file, never into a design document (Section 3.6 of the writing guide).
2. Every statement in another document that the change makes false, found by searching for the names that the change touches. `README.md` is also the readme of every crate.
3. Project Status and Repository Layout of this file, in the short form that each section defines, with no inventory of tests or files.
4. `docs/rust-coding-standards.md` with its Appendix A, `docs/rust-references.md`, or `docs/writing-documentation.md`, only when the change alters a rule, a lint, a tool, a source, or a documentation convention.
5. The Date field of the metadata table of each touched document that has one.

## Contribution Policy

The owner develops the project with coding agents, under the rules in this file. It has no contribution policy yet and opens no pull requests for now. A pull request from a fork runs CI with a read-only token and no secrets. A future policy may require disclosure of artificial intelligence (AI) usage in pull request descriptions.

## References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.
