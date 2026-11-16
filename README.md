# Escher

Escher is a Rust application for creating symmetry-based graphics, like those in some works of M. C. Escher. Its aim is to create and render graphics with the planar symmetries of Euclidean, spherical, and hyperbolic geometry, written in orbifold notation. The source code is in the repository at <https://github.com/g-whiz/escher>.

## Contents

The repository holds the tooling, the documents that set its standards, the research behind the project, the plan of the proof of concept, the command-line program `escher`, a crate that parses and classifies orbifold signatures, a crate of hyperbolic geometry, and a crate that builds the fundamental domain of a hyperbolic reflection triangle. It holds no code that computes a tiling or renders a symmetry.

| Crate | Purpose |
| --- | --- |
| `escher-cli` | The command-line program `escher`, whose command `escher info` prints the properties of an orbifold signature. It is not published. |
| `escher-domain` | A library that builds the fundamental domain of a compact hyperbolic reflection triangle, such as `*732`, with a wall and a reflection for each side, measures its side lengths, corner angles, and area, and checks it numerically. It is not published. |
| `escher-geometry` | A library of the hyperbolic plane in the hyperboloid model over the vector and matrix types of `glam`: points, distances, walls, reflections and other isometries, and the map to and from the Poincare disk. It is not published. |
| `escher-signature` | A library that parses an orbifold signature, such as `*632`, into its canonical form, computes its exact Euler characteristic, and classifies it as spherical, Euclidean, or hyperbolic, with its group order, area, and deformation dimension where they apply. It is not published. |

The directory `docs/research/` holds references on the mathematics of orbifolds, geometry models, fundamental domains, rendering, and colourings, and studies of SymmHub and other existing software. It also holds a survey of Rust crates, three surveys of topics beyond the proof of concept, and a list of sources that the research could not read in full. These three surveys cover aperiodic tilings, partial differential equations on orbifolds, and vector export, animation, and shader export.

The planning documents in `docs/design/` are the architecture design and the milestone plan of the proof of concept, and the roadmap beyond it.

## Requirements

Building the workspace needs Rust 1.99 or later, as the `rust-version` field of the workspace manifest declares. Cargo downloads the crates that the workspace depends on from crates.io, so the first build needs network access.

This command builds and runs the program from the workspace root, and prints its help:

```sh
cargo run -p escher-cli --bin escher -- --help
```

Running the checks also needs these tools:

- `just`, a command runner, and Python 3, which runs the custom checkers.
- The Rust components Clippy and rustfmt, and a nightly Rust toolchain with Miri and `rust-src`.
- cargo-nextest, cargo-deny, cargo-machete, typos, taplo, actionlint, and shellcheck.
- cargo-hack and cargo-llvm-cov, for the checks that continuous integration runs outside `just check`.
- pre-commit, for the git hooks.

`just check` also needs network access, because cargo-deny fetches the security advisory database on every run.

## Usage

`escher info` prints the canonical form of a signature in orbifold notation, its geometry, its Euler characteristic, and the properties that depend on them. The examples call the built program as `escher`. From the workspace root, `cargo run -p escher-cli --bin escher --` followed by the same arguments runs it too.

In a POSIX shell, `*`, `(`, and `)` are special characters, so put a signature that contains any of them in single quotes. This command prints the properties of a spherical signature:

```sh
escher info '*532'
```

```text
Signature: *532
Geometry: spherical
Euler characteristic: 1/60
Compact: yes
Group order: 120
Area: pi/30
```

An infinite order is written `(inf)` or `∞`, and an order of 10 or more in parentheses. This command prints a frieze, whose canonical form puts the infinite order first:

```sh
escher info '*22(inf)'
```

```text
Signature: *(inf)22
Geometry: Euclidean
Euler characteristic: 0
Compact: no
Frieze: yes
General non-compact: no
```

A signature that is not valid gives an error on standard error, with a caret under the offending characters, and exit status 1. This command shows the error for an order of 12 without parentheses:

```sh
escher info '2*12'
```

```text
error: cannot parse orbifold signature: a digit 1 followed by another digit is ambiguous, write an order of 10 or more in parentheses, such as (12)
  2*12
    ^^
```

A wrong command line, such as an unknown command, gives exit status 2.

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
