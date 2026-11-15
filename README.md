# Escher

Escher is a Rust application for creating symmetry-based graphics, like those in some works of M. C. Escher. Its aim is to create and render graphics with the planar symmetries of Euclidean, spherical, and hyperbolic geometry, written in orbifold notation. The source code is in the repository at <https://github.com/g-whiz/escher>.

## Contents

The repository holds the tooling, the documents that set its standards, the research behind the project, the plan of the proof of concept, and one placeholder crate. It holds no code that computes or renders a symmetry.

| Crate | Purpose |
| --- | --- |
| `escher-cli` | The command-line program `escher`, which prints its package name and version. It is not published. |

The directory `docs/research/` holds references on the mathematics of orbifolds, geometry models, fundamental domains, rendering, and colourings, and studies of SymmHub and other existing software. It also holds a survey of Rust crates, three surveys for the roadmap, and a list of sources that the research could not read in full. The roadmap surveys cover aperiodic tilings, partial differential equations on orbifolds, and vector export, animation, and shader export.

The planning documents in `docs/design/` are the architecture design and the milestone plan of the proof of concept, and the roadmap beyond it.

## Requirements

Building the workspace needs Rust 1.99 or later, as the `rust-version` field of the workspace manifest declares. The workspace has no other dependency.

This command builds and runs the program from the workspace root:

```sh
cargo run -p escher-cli --bin escher
```

Running the checks also needs these tools:

- `just`, a command runner, and Python 3, which runs the custom checkers.
- The Rust components Clippy and rustfmt, and a nightly Rust toolchain with Miri and `rust-src`.
- cargo-nextest, cargo-deny, cargo-machete, typos, taplo, actionlint, and shellcheck.
- cargo-hack and cargo-llvm-cov, for the checks that continuous integration runs outside `just check`.
- pre-commit, for the git hooks.

`just check` also needs network access, because cargo-deny fetches the security advisory database on every run.

## Development

Every command is a recipe of `just`. This command lists the recipes:

```sh
just --list
```

This command runs every check:

```sh
just check
```

This command installs the git hooks, once per clone:

```sh
just hooks-install
```

[`AGENTS.md`](https://github.com/g-whiz/escher/blob/main/AGENTS.md) holds the rules for working in the repository and the recipe to run for each kind of change. These documents that set the standards are under `docs/`:

- [Tooling design](https://github.com/g-whiz/escher/blob/main/docs/design/tooling.md): The tool configuration, the recipes, the hooks, the custom checkers, and the continuous integration.
- [Rust coding standards](https://github.com/g-whiz/escher/blob/main/docs/rust-coding-standards.md): The rules for Rust code.
- [Writing guide](https://github.com/g-whiz/escher/blob/main/docs/writing-documentation.md): The rules for documentation.
- [Reference list](https://github.com/g-whiz/escher/blob/main/docs/rust-references.md): The external sources behind the coding standards.

## Licence

Escher is licensed under the GNU General Public License, version 3 or later, with the SPDX identifier `GPL-3.0-or-later`. The file [`LICENSE`](https://github.com/g-whiz/escher/blob/main/LICENSE) in the repository holds the text of the licence.
