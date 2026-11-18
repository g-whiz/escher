# Escher

Escher is a Rust application for creating symmetry-based graphics, like those in some works of M. C. Escher. Its aim is to create and render graphics with the planar symmetries of Euclidean, spherical, and hyperbolic geometry, written in orbifold notation. The source code is in the repository at <https://github.com/g-whiz/escher>.

## Contents

The repository holds the tooling, the documents that set its standards, the research behind the project, the plan of the proof of concept, the command-line program `escher`, a crate that parses and classifies orbifold signatures, a crate of hyperbolic, Euclidean, and spherical geometry, a crate that builds the fundamental domain of a hyperbolic, Euclidean, or spherical reflection triangle, of the rectangle, and of the lune, a crate of group presentations and coset enumeration, and a crate that renders a hyperbolic, Euclidean, or spherical domain as a tiling into a PNG image.

| Crate | Purpose |
| --- | --- |
| `escher-cli` | The command-line program `escher`, whose command `escher info` prints the properties of an orbifold signature and a summary of its fundamental domain, with the side pairs of a rotation domain, and, with `--check`, verifies that domain, and whose command `escher render` writes the tiling of a compact hyperbolic reflection triangle, of a Euclidean or spherical reflection signature with a fundamental domain, or of the rotation signature of any of these, to a PNG file. It is not published. |
| `escher-domain` | A library that builds the fundamental domain of a compact hyperbolic reflection triangle, such as `*732`, of the Euclidean reflection triangles `*632`, `*442`, and `*333`, of the rectangle `*2222`, whose shape parameter `aspect` sets the ratio of its sides, of the spherical reflection triangles `*332`, `*432`, `*532`, and `*n22`, and of the lune `*nn`, with a wall and a reflection for each side. For the rotation signature of each of them, the signature without the star, such as `732`, it builds the double of that domain across one wall, whose sides are paired by rotations and, for `2222`, a translation. It measures the side lengths, corner angles, and area of the domain, and checks it numerically. It is not published. |
| `escher-geometry` | A library of the hyperbolic plane in the hyperboloid model, of the Euclidean plane, and of the sphere, over the vector and matrix types of `glam`: points, distances, walls and lines, reflections and other isometries, and the map between the hyperboloid and the Poincare disk. It is not published. |
| `escher-group` | A library of finitely presented groups: words, relators, and presentations with named generators, the presentation that Conway's rules read from an orbifold signature, and Todd-Coxeter coset enumeration with a limit on the number of cosets, which returns a coset table in a standard form. It is not published. |
| `escher-render` | A library that renders the tiling of a compact hyperbolic reflection triangle in the Poincare disk, of a Euclidean reflection triangle or rectangle in the plane, or of a spherical reflection triangle or lune as a ball with a viewing rotation and Phong shading, or of the double of any of these for its rotation signature, with borders and one or two fill colours, averages several samples in each pixel, and encodes the image as a PNG file in memory. It is not published. |
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

`escher info` prints the canonical form of a signature in orbifold notation, its geometry, its Euler characteristic, and the properties that depend on them. One `Parameter` line for each shape parameter gives its range and default, such as `aspect` of `*2222`. A `Domain` line says whether `escher-domain` builds a fundamental domain for the signature. For a rotation signature whose domain is built, such as `732`, a `Side pairs` line then names each pair of sides and the kind of isometry that pairs them, and a `Parity colouring` line says that no parity colouring exists, because every side pairing preserves orientation. When a domain is built, the corner angles, the side lengths, and the area of the domain follow. The examples call the built program as `escher`. From the workspace root, `cargo run -p escher-cli --bin escher --` followed by the same arguments runs it too.

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
Domain: 3 mirror sides
Corner angles in degrees: 36.000000, 60.000000, 90.000000
Side lengths: 0.553574, 0.652358, 0.364864
Domain area: 0.104720
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
Domain: none, cannot build fundamental domain: the signature *(inf)22 is not supported
```

The option `--check` verifies the fundamental domain and prints the result of each check. This command verifies the domain of a hyperbolic reflection triangle:

```sh
escher info --check '*732'
```

After the properties and the domain summary, it prints one line for each of the six checks, with its result, residual, and tolerance, and then a `Verification` line with the overall result.

With `--check`, the exit status is 1 when a check fails or the signature has no fundamental domain.

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

### Rendering a Tiling

`escher render` draws the tiling of a signature whose `Domain` line in `escher info` names a domain, and writes it to a PNG file. It draws a compact hyperbolic reflection triangle, such as `*732`, in the Poincare disk, the Euclidean signatures `*632`, `*442`, `*333`, and `*2222` in the plane, and the spherical signatures `*332`, `*432`, `*532`, `*n22`, and `*nn` as a lit ball. It also draws the rotation signature of each of these, such as `732`, in the same view as its reflection signature. It gives an error with exit status 1 for any other signature. This command writes a 512 by 512 image to `tiling.png`, and replaces the file if it exists:

```sh
escher render '*732' -o tiling.png
```

This command draws the Euclidean tiling of `*632`:

```sh
escher render '*632' -o tiling.png
```

The option `--set` sets a shape parameter, as `NAME=VALUE`. This command draws `*2222` with rectangles whose sides are in the ratio `aspect` of 1.618:

```sh
escher render '*2222' --set aspect=1.618 -o tiling.png
```

This command draws the rotation tiling of `732`:

```sh
escher render 732 -o rotation.png
```

This command draws the ball of `*532`, turned 20 degrees to the right and 35 degrees up:

```sh
escher render '*532' --rotation 20,35,0 -o ball.png
```

The option `--rotation` turns the ball by three angles in degrees, `YAW,PITCH,ROLL`. The option `--light` sets the direction of the light as `X,Y,Z`, with `X` to the right, `Y` up, and `Z` toward the viewer. The option `--shading` takes `phong`, the default, or `none` for a flat disk. These three options change only a spherical tiling, but `escher render` checks their values for every signature, so a bad value is an error for any tiling.

By default, the tiles have black borders 1 pixel wide. When a side pairing of the domain reverses orientation, as for every reflection signature, the tiles alternate between two fill colours by parity. Otherwise, as for a rotation signature, every tile has one fill colour, and `--colouring parity` is an error. A hyperbolic or spherical tiling has a white background outside the disk or the ball. The options `--size`, `--colouring`, `--colours`, `--border-width`, `--border-colour`, `--background`, and `--samples` change the image, and `escher render --help` lists these and the other options.

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
