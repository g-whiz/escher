# Escher

Escher is a Rust application for creating symmetry-based graphics, like those in some works of M. C. Escher. Its aim is to create and render graphics with the planar symmetries of Euclidean, spherical, and hyperbolic geometry, written in orbifold notation. The source code is in the repository at <https://github.com/g-whiz/escher>.

## Contents

The repository holds the tooling, the documents that set its standards, the research behind the project, the plan of the proof of concept, the command-line program `escher`, a crate that parses and classifies orbifold signatures, a crate of hyperbolic, Euclidean, and spherical geometry, a crate that builds the fundamental domain of a hyperbolic, Euclidean, or spherical reflection triangle, of the rectangle, and of the lune, of the double of each of these for its rotation signature, of a hyperbolic reflection polygon with four to eight corners and its double for its rotation signature, and of each of the 17 Euclidean signatures, and reads a presentation of its group from its sides, a crate of group presentations, coset enumeration, and the search for the colourings of a tiling, and a crate that renders a hyperbolic, Euclidean, or spherical domain, or the double of one for its rotation signature, as a tiling into a PNG image.

| Crate | Purpose |
| --- | --- |
| `escher-cli` | The command-line program `escher`. Its command `escher info` prints the properties of an orbifold signature, a summary of its fundamental domain, which lists the side pairs of a domain with paired sides and also its mirror sides when it has both, and the colourings of its tiling. With `--colourings` it lists those colourings with their ids, with `--check` it verifies the domain, and with `--presentation` it prints the presentations of the group and, for a spherical signature, counts the elements of the group by coset enumeration. Its command `escher render` writes the tiling of a compact hyperbolic reflection triangle, of a spherical reflection signature with a fundamental domain, of the rotation signature of any of these, of a compact hyperbolic reflection polygon with four to eight corners or its rotation signature, or of any of the 17 Euclidean signatures, to a PNG file, with one colour, two colours by parity, or a colouring given by its id. It is not published. |
| `escher-domain` | A library that builds the fundamental domain of a compact hyperbolic reflection triangle, such as `*732`, of the Euclidean reflection triangles `*632`, `*442`, and `*333`, of the rectangle `*2222`, whose shape parameter `aspect` sets the ratio of its sides, of the spherical reflection triangles `*332`, `*432`, `*532`, and `*n22`, and of the lune `*nn`, with a wall and a reflection for each side. For the rotation signature of each of them, the signature without the star, such as `732`, it builds the double of that domain across one wall, whose sides are paired by rotations and, for `2222`, by a translation. The double of `2222` also has the shape parameter `skew`, which shears it. It also builds the compact hyperbolic reflection polygon of a signature with four to eight corners, such as `*3333`, whose shape parameters `cornerpair<k>.length` and `slice<i>.length` set the lengths of some of its sides and of the cuts across it. For the rotation signature of such a polygon, such as `3333`, it builds the double of the polygon at its default shape, with no shape parameter, whose sides are paired by rotations and translations. It builds the parallelogram of `o`, whose opposite sides are paired by translations and whose shape parameters are `aspect` and `skew`, and the rectangle of `**`, with two mirror sides and two sides paired by a translation, whose shape parameter is `aspect`. On the same rectangle, with the same parameter, it builds the domains of `xx`, with two sides paired by a glide reflection and two by a translation, of `*x`, with two mirror sides and two sides paired by a glide reflection, of `22x`, whose four sides are paired by glide reflections, of `22*`, with two mirror sides and its bottom and top sides each split in two halves paired by a half-turn, and of `2*22`, with three mirror sides and its bottom side split in the same way. For `4*2` and `3*3`, it builds the domain of `442` or `632` with two of its sides as mirrors and the other two paired by a rotation, with no shape parameter. So it builds every one of the 17 Euclidean signatures. It returns a polygon, a double of a polygon, the domain of `2222`, or the domain of `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, or `3*3` only when its numeric checks pass. It measures the side lengths, corner angles, and area of the domain, checks it numerically, and reads a presentation of the group from its sides, with one generator for each side. It is not published. |
| `escher-geometry` | A library of the hyperbolic plane in the hyperboloid model, of the Euclidean plane, and of the sphere, over the vector and matrix types of `glam`: points, distances, walls and lines, reflections and other isometries, and the map between the hyperboloid and the Poincare disk. It is not published. |
| `escher-group` | A library of finitely presented groups: words, relators, and presentations with named generators, the presentation that Conway's rules read from an orbifold signature, Todd-Coxeter coset enumeration with a limit on the number of cosets, which returns a coset table in a standard form, and the low-index subgroup search, which counts or lists the colourings of a tiling, one for each subgroup of finite index, each with an id such as `3a`. It is not published. |
| `escher-render` | A library that renders the tiling of a compact hyperbolic reflection triangle in the Poincare disk, of a Euclidean reflection triangle or rectangle in the plane, of a spherical reflection triangle or lune as a ball with a viewing rotation and Phong shading, of the double of any of these for its rotation signature, of a compact hyperbolic reflection polygon with four to eight corners or its double in the Poincare disk, or of the domain of any other Euclidean signature in the plane, with borders and either one or two fill colours or one colour for each coset of a colouring of `escher-group`, averages several samples in each pixel, and encodes the image as a PNG file in memory. It is not published. |
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
- The Rust components Clippy and rustfmt.
- cargo-nextest, cargo-deny, cargo-machete, typos, taplo, actionlint, and shellcheck.
- cargo-hack and cargo-llvm-cov, for the checks that continuous integration runs outside `just check`.
- pre-commit, for the git hooks.

`just check` also needs network access, because cargo-deny fetches the security advisory database on every run.

## Usage

`escher info` prints the canonical form of a signature in orbifold notation, its geometry, its Euler characteristic, and the properties that depend on them. One `Parameter` line for each shape parameter gives its range and default, such as `aspect` of `*2222` or `slice1.length` of `*3333`. A `Domain` line says whether `escher-domain` builds a fundamental domain for the signature. For a built domain, it gives the number of mirror sides when every side is a mirror, the numbers of sides and pairs when no side is a mirror, as for `732` and `o`, and the numbers of sides, mirror sides, and pairs otherwise, as for `**`. A domain with paired sides also has a `Side pairs` line, which names each pair and the kind of isometry that pairs them, and a domain with mirror sides and paired sides has a `Mirror sides` line before it. When every side pairing preserves orientation, as for `732` and `o`, a `Parity colouring` line says that no parity colouring exists. When a domain is built, the corner angles, the side lengths, and the area of the domain follow. The colouring lines come next. They count the colourings of the tiling, one for each subgroup of finite index, for each number of colours up to 12, or up to the number from 1 to 64 that `--max-colours` sets. With `--colourings`, `escher info` also lists each colouring with its id, such as `3a`. The examples call the built program as `escher`. From the workspace root, `cargo run -p escher-cli --bin escher --` followed by the same arguments runs it too.

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
Colourings: 46, complete to 12 colours
Colourings with 1 colour: 1
Colourings with 2 colours: 1
Colourings with 5 colours: 5
Colourings with 6 colours: 6
Colourings with 10 colours: 15
Colourings with 12 colours: 18
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

After the properties, the domain summary, and the colouring lines, it prints one line for each of the six checks, with its result, residual, and tolerance, and then a `Verification` line with the overall result.

With `--check`, the exit status is 1 when a check fails or the signature has no fundamental domain.

The option `--presentation` prints the presentation of the group that Conway's rules read from the signature and, when a domain is built, the presentation read from the sides of the fundamental domain, with one generator for each side. For a spherical signature whose group order is at most 16384, it also counts the elements of the group of each presentation by coset enumeration, and compares the count with the group order. This command prints the presentations of `*532`:

```sh
escher info --presentation '*532'
```

After the properties, the domain summary, and the colouring lines, it prints these lines:

```text
Presentation: 5 generators, 9 relators
Generators: r1, r2, r3, r4, l1
Relators: r1^2, (r1 r2)^5, r2^2, (r2 r3)^3, r3^2, (r3 r4)^2, r4^2, l1^-1 r1 l1 r4^-1, l1
Order by coset enumeration: 120, equal to 2/chi
Side presentation: 3 generators, 6 relators
Side generators: s0, s1, s2
Side relators: s0^2, s1^2, s2^2, (s0 s1)^5, (s1 s2)^3, (s2 s0)^2
Side order by coset enumeration: 120, equal to 2/chi
```

With `--presentation`, the exit status is 1 when a count differs from the group order.

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

`escher render` draws the tiling of a signature whose `Domain` line in `escher info` names a domain, and writes it to a PNG file. It draws a compact hyperbolic reflection triangle, such as `*732`, in the Poincare disk, and the spherical signatures `*332`, `*432`, `*532`, `*n22`, and `*nn` as a lit ball. It also draws the rotation signature of each of these, such as `732`, in the same view as its reflection signature. It draws a compact hyperbolic reflection polygon with four to eight corners, such as `*3333`, in the Poincare disk, and its rotation signature, such as `3333`, in the same view. It draws all 17 Euclidean signatures in the plane: `*632`, `*442`, `*333`, `*2222`, `632`, `442`, `333`, `2222`, `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`. It gives an error with exit status 1 for any other signature. This command writes a 512 by 512 image to `tiling.png`, and replaces the file if it exists:

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

The signature `o` also has the parameter `skew`, from -0.5 to 0.5 with the default 0, which leans its parallelogram. This command draws `o` with a skew of 0.25:

```sh
escher render o --set skew=0.25 -o lattice.png
```

The signature `2222` has `skew` too, with the same range and default, which shears its tiles into parallelograms:

```sh
escher render 2222 --set skew=0.25 -o sheared.png
```

A reflection polygon has one length parameter for each corner beyond three, each from 0.5 to 3.5. This command draws `*3333` with its one slice longer than the default of 1.317, which changes the shape of each tile:

```sh
escher render '*3333' --set slice1.length=2.5 -o polygon.png
```

A length far from its default can give an error with exit status 1, because `escher-domain` refuses a polygon that it cannot build accurately. The rotation signature of a polygon, such as `3333`, has no length parameter. `escher-domain` refuses a few of them with eight orders of which several are high, such as `(967)(967)(967)(967)(967)(967)(967)(967)`, because it cannot store the side pairings of their doubles accurately enough. Section 8.4 of `docs/design/domain.md` lists which.

This command draws the rotation tiling of `732`:

```sh
escher render 732 -o rotation.png
```

This command draws the ball of `*532`, turned 20 degrees to the right and 35 degrees up:

```sh
escher render '*532' --rotation 20,35,0 -o ball.png
```

The option `--rotation` turns the ball by three angles in degrees, `YAW,PITCH,ROLL`. The option `--light` sets the direction of the light as `X,Y,Z`, with `X` to the right, `Y` up, and `Z` toward the viewer. The option `--shading` takes `phong`, the default, or `none` for a flat disk. These three options change only a spherical tiling, but `escher render` checks their values for every signature, so a bad value is an error for any tiling.

The option `--colouring` takes `one`, `parity`, or a colouring id from the `Colouring` lines of `escher info --colourings`. This command lists the colourings of `*642` with up to 3 colours:

```sh
escher info --colourings --max-colours 3 '*642'
```

This command draws its colouring `3a` in the first three of 12 built-in colours:

```sh
escher render '*642' --colouring 3a -o three.png
```

This command draws the same colouring in colours of its own. `--colours` takes exactly one colour for each colour of the colouring, and a colouring with more than 12 colours needs it:

```sh
escher render '*642' --colouring 3a --colours '#dc143c,#ffd700,#000080' -o three.png
```

By default, the tiles have black borders 1 pixel wide. When a side pairing of the domain reverses orientation, as for every reflection signature and for `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, the tiles alternate between two fill colours by parity. Otherwise, as for a rotation signature and for `o`, every tile has one fill colour, and `--colouring parity` is an error. By default, `*x` and `22x` give the same picture as `*2222`, and `22*` the same picture as `**`. These signatures differ only in their colourings by id. By default, `4*2` and `3*3` are coloured by parity, while `442` and `632` have one colour. With `--colouring one`, `4*2` gives the picture of `442`, and `3*3` that of `632`. A hyperbolic or spherical tiling has a white background outside the disk or the ball. The options `--size`, `--colouring`, `--colours`, `--border-width`, `--border-colour`, `--background`, and `--samples` change the image, and `escher render --help` lists these and the other options.

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
