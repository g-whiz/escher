# Render Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-17 |
| Applies to | `crates/escher-render` |

## Abstract

This document specifies the crate `escher-render`, which turns the fundamental domain of a compact hyperbolic reflection triangle into the pixels of an image in the Poincare disk, and the fundamental domain of a Euclidean reflection triangle or rectangle into the pixels of an image of the Euclidean plane, and the fundamental domain of a spherical reflection triangle or lune into the pixels of an image of a lit ball, and encodes them as a PNG file in memory. It renders the domain of a rotation subgroup, whose sides are paired by rotations and translations, through the same steps. For every sample of a regular grid in each pixel, the crate maps the sample to a point of the linear model, folds the point into the domain, decides whether the sample lies on a border of a tile, chooses a fill, border, or background colour, and on the ball shades that colour. It then averages the samples of each pixel in linear light and stores the pixel as 8-bit sRGB. The document defines the public types and operations, the steps of the fold and of the border estimate with the source or the unverified status of each formula, the behaviour of the fold for points on walls, near the limit circle, and with non-finite coordinates, the range over which the crate states its accuracy, the error type, the limits on the options, the tests with the golden images and the reference arithmetic, and the dependencies. It also lists the unverified statements that the code relies on, the evidence behind the choices of this design, the assumptions, and the points that the owner decides.

## 1. Introduction

### 1.1. Problem

The proof of concept draws a tiling by pulling each sample back into the fundamental domain, one sample at a time (Section 4.1 of the architecture). The research gives the loop that folds a point into the domain and the theorem that makes the triangle a strict fundamental domain, but no bound on the number of steps, no behaviour for a point on a wall, and no form of a border width in screen pixels through a chain of folds in the linear model (Sections 4.7, 6, 7, and 13.1 of the rendering reference). Floating-point evaluation adds a second risk: near the limit circle the map from the disk to the hyperboloid loses digits, and the fold then applies a different side than exact arithmetic would. A wrong decision produces an image that looks plausible, so the design states where the crate is accurate, what it does beyond that range, and how the tests measure both.

### 1.2. Purpose

`escher-render` is the one place where a `FundamentalDomain` becomes pixels. It exposes the fold as a public function, a renderer that fills a band of rows in a buffer that the caller owns, statistics of the folds, a colour type, and PNG encoding. Section 3 of the architecture places the crate in the workspace. It depends on `escher-geometry` for the linear model and the disk map, on `escher-domain` for the domain, and on `escher-signature` for the geometry of the domain, and `escher-cli` consumes it through the command `escher render`.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the author of the command-line design, which specifies the command `render` on top of this crate, and the authors of the later milestones that add models, intrinsic borders, analytic antialiasing, and colourings by cosets.

### 1.4. Scope

This document covers the public surface of the crate, the options with their defaults and limits, the map from a sample to the disk and to the hyperboloid, to the Euclidean plane, and to the ball, the viewing rotation and the shading of the ball, the fold, the border estimate, the colour of a sample and of a pixel, PNG encoding, the numerical behaviour, the error type, the tests, and the dependencies.

The crate renders every domain that `FundamentalDomain::build` and `FundamentalDomain::build_with` return. These are the compact hyperbolic `*pqr` with corner orders up to 1000, in the canonical view of the Poincare disk, and the Euclidean `*632`, `*442`, `*333`, and `*2222`, the last with any value of its shape parameter, in the canonical view of the Euclidean plane (Section 5.8), and the spherical `*332`, `*432`, `*532`, `*n22`, and `*nn`, with $n$ up to 1000, as a lit ball (Section 5.9). It also renders the rotation domains of `pqr`, `632`, `442`, `333`, `2222`, `332`, `432`, `532`, `n22`, and `nn`, with the same limits, in the view of their geometry (Section 5.11). Section 4 of the domain design lists them.

### 1.5. Non-Goals

These topics are owned elsewhere or come later:

- The command `escher render` is specified in the command-line design: its options and the parsing of colour text (Section 4.6), the parallel split of the image into bands of rows (Section 4.5), the output file and the statistics text (Section 4.7), and the diagnostics (Section 6.8).
- The bilinear form, the side test, the isometries, and the disk map, with their formulas and accuracy, belong to `escher-geometry` and are specified in the geometry design. This document calls those functions by name and restates none of their formulas.
- The construction, placement, and verification of the domain belong to `escher-domain` and are specified in the domain design.
- Models other than the Poincare disk, a public `View`, and borders of intrinsic width come with the milestones of Sections 5.12 and 5.13 of the plan.
- Analytic antialiasing and the fade of tiles smaller than a pixel come with the milestone of Section 5.14 of the plan.
- Colourings by cosets and generated palettes need `escher-group` and come with the milestone of Section 5.8 of the plan.
- The shape parameters of a domain, their names, ranges, and defaults, belong to `escher-domain` and are specified in the domain design.
- A transparent background, the back hemisphere of the ball, a perspective projection, a second light, and options for the material of the ball are outside this design.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md`, the terms linear model, model, model map, fundamental domain, tile, side, wall, covector, pairing isometry, fold, fold cap, sample, colouring, reference arithmetic, golden image, and unverified statement of Section 2.1 of `docs/design/architecture.md`, and the terms point, unit normal, form, and $\varepsilon$ of Section 2.1 of `docs/design/geometry.md`. It cites these documents by short names:

| Document | Short name |
| --- | --- |
| `docs/design/architecture.md` | the architecture |
| `docs/design/poc-plan.md` | the plan |
| `docs/design/geometry.md` | the geometry design |
| `docs/design/domain.md` | the domain design |
| `docs/design/cli.md` | the command-line design |
| `docs/research/rendering.md` | the rendering reference |
| `docs/research/geometry-models.md` | the geometry reference |
| `docs/research/fundamental-domains.md` | the domain reference |
| `docs/research/rust-ecosystem.md` | the ecosystem survey |
| `docs/rust-coding-standards.md` | the coding standard |

A function written as `hyperbolic::name` or `spherical::name` is the function of that name in the module `hyperbolic` or `spherical` of `escher-geometry`.

It adds these terms:

- image size: The width $W$ and the height $H$ of the image in pixels.
- sample grid: The $n$ by $n$ grid of samples in each pixel, with $n$ the value of the option `samples`.
- sample weight: The weight $1/n^2$ of one sample in the mean of its pixel.
- margin: The fraction $m$ of the shorter side of the image that the disk or the ball leaves free on each side (Section 5.1).
- disk scale: The length $h$ in disk units of one pixel (Section 5.1).
- plane scale: The length $g$ in plane units of one pixel of the view of a Euclidean domain (Section 5.8).
- plane position: The position $z$ in the Euclidean plane of a sample of the view of a Euclidean domain (Section 5.8).
- screen tangents: The two vectors $T_1$ and $T_2$ that the crate carries through the fold with the point. Each is the change of the point of the linear model for a step of one pixel along one axis of the screen (Section 5.2).
- ball position: The position $z$ in the unit disk of a sample of the view of a spherical domain, computed as the disk position (Section 5.9).
- rim distance: The distance in pixels from a sample to the rim of the ball, which is $(1 - \lvert z \rvert) / h$ for the ball position $z$.
- rim band: The samples of the ball at one of the rim distances 120, 24, 8, 4, 2, 1, and 0.5 pixels in a 512 by 512 image (Section 6.7).
- viewing matrix: The rotation $V$ that the viewing rotation gives, which turns the ball while the viewer and the light stay fixed (Section 5.9).
- point size: The quantity $\lambda(x)$ that the fold reads from the point $x$: the last coordinate $x_3$ for a hyperbolic or Euclidean domain, and the largest absolute coordinate $\max_j \lvert x_j \rvert$ for a spherical domain (Section 5.3).
- tolerance scale: The sum $\sigma_i$ of the absolute values of the components of the covector of side $i$ (Section 5.3).
- side value: The dot product $f_i = c_i \cdot x$ of the covector $c_i$ of side $i$ and the folded point $x$.
- gradient size: The value $g_i$ of Section 5.4, the change of the side value $f_i$ per pixel.
- half width: Half the border width, in pixels.
- step: One application of a pairing isometry by the fold.
- reversal: A step whose pairing reverses orientation.
- parity: The number of reversals of a fold modulo 2.
- converged: The result of a fold that stopped at a finite point with a positive point size that violates no side (Section 5.3).
- unconverged: The result of every other fold, which reached the fold cap, met a coordinate that is not finite, met a point size that is not positive, or met a side value that is NaN.
- border sample: A converged sample that the border estimate of Section 5.4 places on a border.
- outside sample: A sample whose disk position `hyperbolic::from_disk` rejects, or a sample of the view of a spherical domain that lies on or outside the rim of the ball (Section 5.9).
- tolerance constant: The constant $\tau$ of the side test of the fold (Section 6.2).
- radius band $k$: The disk positions $z$ with $\lvert z \rvert = 1 - 10^{-k}$, for an integer $k$ from 1 to 6.
- range of accuracy: For a hyperbolic domain, the disk positions with $\lvert z \rvert \le 1 - 10^{-6}$ (Section 6.4). For a Euclidean domain, the plane positions within distance 65 of the centre of the image (Section 6.6). For a spherical domain, every vector whose largest absolute coordinate is at least $10^{-300}$ and whose fold converges, which includes the point of every ball position inside the rim whose fold converges (Section 6.1). For the border estimate, the range is the rim bands that Section 6.7 states. A rotation domain has the range of its geometry, except near some vertices of a hyperbolic rotation domain whose corner orders are all near 1000, which RND-35 of Section 6.8 excludes.
- linear light: Colour values proportional to light intensity, in which the crate averages samples.
- 8-bit sRGB: Colour values encoded by the sRGB transfer function and stored as one byte per channel.
- band: A run of whole rows of the image, which `render_rows` fills.
- canonical view: For a hyperbolic domain, the view of the whole disk with the centre of the domain at the centre of the image. For a Euclidean domain, the view of Section 5.8. For a spherical domain, the ball of Section 5.9 with the default viewing rotation. A rotation domain has the canonical view of its starred signature (`DEC-12`, Section 5.11).
- symmetric view: A view available only to the unit tests, which places vertex 2 of the domain at the centre of the image and vertex 0 on the positive first axis (Section 8.1).
- periodic view: A view available only to the unit tests, which is the view of Section 5.8 with another plane scale (Section 8.1).
- rotated view: A view available only to the unit tests, which is the view of Section 5.9 with any orthogonal matrix in place of the viewing matrix (Section 8.1).
- allowance: The two limits of a golden-image comparison, the largest difference of a pixel and the number of pixels that may differ (Section 8.4).
- count bound: The bound on a count that a test measures as $c$, which is the larger of $8c$ and 8 (RND-16).
- prototype: An implementation of Sections 5 and 6 with a 256-bit reference, written outside the repository to test this design. Section 11 states what it measured.
- Euclidean prototype: A copy of the crate outside the repository whose private view gained the plane view of Section 5.8, with private test code that implements the Euclidean domains and their checks, and a 256-bit reference. Section 11 states what it measured.
- Euclidean domain set: The domains of `*632`, `*442`, and `*333`, and of `*2222` with `aspect` 1, 1.618, 0.125, and 8.
- sphere prototype: A copy of the crate outside the repository whose fold gained the point size of Section 5.3, with private test code that implements the spherical domains, the ball view, the viewing matrix, shading, a brute-force border oracle, a breadth-first enumeration of the group, and a 256-bit reference. Section 11 states what it measured.
- spherical domain set: The domains of `*332`, `*432`, and `*532`, and of `*n22` and `*nn` for $n$ of 2, 3, 5, 12, 100, 999, and 1000. `*222` is `*n22` with $n = 2$, and `*22` is `*nn` with $n = 2$.
- shading constants: The ambient weight $k_a$, the diffuse weight $k_d$, the specular weight $k_s$, and the specular exponent $\alpha$ of Section 5.10.
- reflection domain: A domain whose every side has the kind `Mirror`, such as the domain of `*732`.
- rotation domain: A domain that `escher-domain` builds for a signature without reflections, such as `732`, as the double of the reflection domain of its starred signature. No side of it has the kind `Mirror` (Section 5.11).
- starred signature: The signature of the reflection domain from which the domain design builds a rotation domain, such as `*732` for `732`, `*2222` for `2222`, and `*nn` for `nn`.
- doubled wall: The wall of side 1 of the reflection domain, across which the domain design doubles it. It lies inside the rotation domain and is not a wall of any of its sides.
- rotation domain set: The rotation domains of the signatures of the fold set of Section 6.4 without the star, `732`, `542`, `444`, `433`, `(12)32`, `(1000)32`, and `(1000)(1000)(1000)`, of `632`, `442`, `333`, and `2222` with the values of `aspect` of the Euclidean domain set, and of the signatures of the spherical domain set without the star.
- rotation prototype: Code outside the repository that built the rotation domains from the public data of the reflection domains, with a copy of the fold, of the checks of the domain design, and of the logic of a sample of the renderer, and a 256-bit reference. Section 11 states what it measured.

## 3. Public Surface

### 3.1. Items

RND-1. The crate MUST export exactly the items of this section from its root, each through one path, and MUST give no type a public field, because a public field lets a caller build options or statistics that break the invariants of Section 3.2, and the renderer uses them without a check (API-10 of the coding standard).

This listing shows each public item with its signature. Bodies are omitted. `DVec3` is the `glam` type of `DEC-4`:

```rust
pub struct Colour { /* private: linear-light red, green, and blue */ }
// derives Clone, Copy, Debug, PartialEq
impl Colour {
    pub const BLACK: Self;
    pub const WHITE: Self;
    #[must_use] pub fn from_srgb8(red: u8, green: u8, blue: u8) -> Self;
    #[must_use] pub fn to_srgb8(self) -> [u8; 3];
}

#[non_exhaustive]
pub enum Fill { One(Colour), Parity([Colour; 2]) }
// derives Clone, Copy, Debug, PartialEq

#[non_exhaustive]
pub enum Shading { Phong, Flat }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub struct RenderOptions { /* private */ }
// derives Clone, Debug, PartialEq
impl RenderOptions {
    pub fn builder(fill: Fill) -> RenderOptionsBuilder;
    #[must_use] pub fn width(&self) -> u32;
    #[must_use] pub fn height(&self) -> u32;
}

#[must_use]
pub struct RenderOptionsBuilder { /* private */ }
// derives Clone, Debug
impl RenderOptionsBuilder {
    pub fn size(self, width: u32, height: u32) -> Self;
    pub fn samples(self, grid: u32) -> Self;
    pub fn border_width(self, pixels: f64) -> Self;
    pub fn border_colour(self, colour: Colour) -> Self;
    pub fn background(self, colour: Colour) -> Self;
    pub fn max_folds(self, cap: u32) -> Self;
    pub fn rotation(self, yaw: f64, pitch: f64, roll: f64) -> Self;
    pub fn light(self, direction: glam::DVec3) -> Self;
    pub fn shading(self, shading: Shading) -> Self;
    pub fn build(self) -> Result<RenderOptions>;
}

pub struct Renderer { /* private */ }
// derives Clone, Debug
impl Renderer {
    #[must_use] pub fn new(domain: &escher_domain::FundamentalDomain, options: RenderOptions) -> Self;
    pub fn render_rows(&self, first_row: u32, pixels: &mut [u8]) -> Result<RenderStats>;
    #[must_use] pub fn render(&self) -> (Vec<u8>, RenderStats);
}

pub struct RenderStats { /* private */ }
// derives Clone, Copy, Debug, Default, PartialEq, Eq
impl RenderStats {
    #[must_use] pub fn samples(self) -> u64;
    #[must_use] pub fn outside(self) -> u64;
    #[must_use] pub fn folds(self) -> u64;
    #[must_use] pub fn largest_fold(self) -> u32;
    #[must_use] pub fn unconverged(self) -> u64;
    #[must_use] pub fn merged(self, other: Self) -> Self;
}

pub struct Fold { /* private */ }
// derives Clone, Copy, Debug, PartialEq
impl Fold {
    #[must_use] pub fn point(self) -> glam::DVec3;
    #[must_use] pub fn steps(self) -> u32;
    #[must_use] pub fn reversals(self) -> u32;
    #[must_use] pub fn converged(self) -> bool;
}

pub fn fold(
    domain: &escher_domain::FundamentalDomain,
    point: glam::DVec3,
    max_folds: u32,
    visit: impl FnMut(usize),
) -> Fold;

pub fn encode_png(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>>;

pub struct Error { /* private */ }
// derives Debug. Implements Display and std::error::Error by hand (Section 7.1)
impl Error {
    #[must_use] pub fn kind(&self) -> ErrorKind;
}

#[non_exhaustive]
pub enum ErrorKind { InvalidOption, LimitExceeded, InvalidBuffer, Encoding }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

`builder` carries no `#[must_use]` of its own, because its return type is `#[must_use]` and Clippy rejects the second attribute (`clippy::double_must_use`). The crate does not re-export `glam` (API-10 of the coding standard). A caller that names a `glam` type declares `glam` from the workspace entry.

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `Colour` | An opaque colour, stored in linear light | Each channel is finite and in $[0, 1]$, because the only constructors take 8-bit values |
| `Colour::BLACK`, `Colour::WHITE` | Black and white | Equal to `from_srgb8(0, 0, 0)` and `from_srgb8(255, 255, 255)` |
| `from_srgb8` | Converts 8-bit sRGB to linear light by the transfer function of Section 5.6 | Total |
| `to_srgb8` | Converts linear light to 8-bit sRGB by the transfer function of Section 5.6 | `to_srgb8(from_srgb8(r, g, b))` is `[r, g, b]` for every byte value |
| `Fill` | The colouring with its colours | The number of colours matches the colouring by construction (TYPE-3 of the coding standard), so the crate has no error for a wrong number of colours |
| `Shading` | `Phong` shades the ball as Section 5.10 states, and `Flat` leaves each sample colour unchanged, so the ball is a flat disk | Read only for a spherical domain. The name `Flat` avoids a variant `None`, which reads as an `Option` |
| `rotation` | The viewing rotation of the ball, as the yaw, the pitch, and the roll in radians (Section 5.9) | Checked by `build` |
| `light` | The direction from the ball to the light, in the coordinates of the screen of Section 5.9. Its length has no effect | Checked and normalized by `build` |
| `RenderOptions` | Validated options | Built only by `build`, which checks every limit of Section 4.1 |
| `RenderOptions::builder` | Starts from the defaults of Section 4.1 with the given fill | Infallible |
| `build` | Validates the options | Returns `InvalidOption` or `LimitExceeded` as Section 7.2 states. Never panics |
| `Renderer::new` | Copies the covectors, pairings, and orientations of the sides of the domain, chooses the view from `geometry()` of the domain (RND-23), and computes the disk scale, the plane scale, or the ball scale and the viewing matrix, and the tolerance scale of each side | Infallible, because the domain and the options are valid by construction, and every geometry has a view. The renderer keeps no reference to the domain |
| `render_rows` | Renders the rows from `first_row` into the caller's buffer | The buffer holds 8-bit sRGB, 3 bytes per pixel, red first, the top row first. Its length MUST be a positive multiple of $3W$, and the band MUST lie inside the image, else `InvalidBuffer`. It starts no thread (RND-2) |
| `render` | The whole image on the calling thread | Equal, byte for byte, to the concatenation of `render_rows` over any split into bands (RND-3) |
| `RenderStats` | Counts over the samples of a band or an image | Integer counts only, so no value is NaN |
| `samples()` | The number of samples, including outside samples | $W n^2$ times the number of rows of the band |
| `outside()` | The number of outside samples, which are the samples outside the disk or the ball | At most `samples()`. 0 for a Euclidean domain |
| `folds()` | The sum of the steps of the folds of every sample that is not an outside sample | Saturates at `u64::MAX` |
| `largest_fold()` | The largest number of steps of one fold | At most the fold cap |
| `unconverged()` | The number of unconverged samples | At most `samples() - outside()` |
| `merged` | The statistics of two bands together | Adds the counts with saturation and takes the larger `largest_fold()` |
| `fold` | Folds one point into the domain and calls `visit` with the index of each side that it applies, in order | The steps of Section 5.3, with the same decisions as the fold inside the renderer for the same point and cap (RND-7). Never panics (RND-14) |
| `Fold::point` | The folded point | Not renormalized (RND-8). Finite, with a positive point size, when `converged()` is true. For a spherical domain that means a finite vector other than the zero vector, read as its ray |
| `Fold::steps` | The number of steps | At most the cap |
| `Fold::reversals` | The number of reversals | At most `steps()` |
| `Fold::converged` | Whether the fold stopped at a finite point with a positive point size that lies inside every side | False when the point, at the start or after a step, has a coordinate that is not finite or a point size that is not positive, when a side value is NaN, and when the fold reached the cap |
| `encode_png` | Encodes an 8-bit RGB buffer as a PNG file with the sRGB chunk | In memory, so the crate reads and writes no file (RND-12) |
| `Error` | Why an operation failed | Section 7 |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

RND-2. The crate MUST NOT start a thread, because the caller chooses the parallelism and splits the image into bands of rows, as CONC-4 of the coding standard and Section 3.5 of the architecture lay out for the `rayon` of the binary (`DEC-21`), and a thread that the caller cannot see escapes its control of time and memory.

RND-3. The colour of each pixel MUST depend only on the domain, the options, and the position of the pixel, and MUST NOT depend on the band that contains it or on any other sample, because the caller renders bands in parallel in any order and expects the same file for every split, and the test "Bands" of Section 8.1 compares the bytes exactly. The crate therefore does not seed a fold from a neighbouring pixel, which Section 8 of the architecture defers.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, the unit test "Auto traits" of Section 8.1, and, under `#[cfg(test)]`, the modules `reference` and `golden_images`, loaded from `tests/render/reference.rs` and `tests/render/golden_images.rs` through `#[path]` (Section 8.1).
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/colour.rs`: `Colour`, `Fill`, and the sRGB transfer function.
- `src/options.rs`: `RenderOptions`, `RenderOptionsBuilder`, `Shading`, the defaults, and the limits.
- `src/fold.rs`: `Fold`, `fold`, the private fold that carries the screen tangents, the private enum that selects the point size, and the tolerance constant.
- `src/view.rs`: The private view, which maps a sample to a disk position, a plane position, or a ball position and a point of the linear model with its screen tangents, by the geometry of the domain, and the symmetric view, the periodic view, and the rotated view of the unit tests. The view holds one case per geometry: the disk with its disk scale and the placement, the plane with its plane scale, or the ball with its scale and the viewing matrix (Section 5.9).
- `src/renderer.rs`: `Renderer`, `RenderStats`, the border estimate, the colour of a sample, its shading (Section 5.10), and the colour of a pixel.
- `src/encode.rs`: `encode_png`.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, and Section 3.2 of the architecture lets the crate design refine the names and signatures:

- A public `View` with the model and the placement. Each geometry has one canonical view in this design, which the renderer chooses from the domain. The only option that changes a view is the viewing rotation of the ball, which `RenderOptions` carries as three angles, so a public `View` would give a caller nothing more to choose, and SIMP-1 of the coding standard removes an abstraction that gives no benefit now. The disk case of the private view holds the placement isometry, which is the identity for the canonical view. The change that adds the first option that changes the model or the placement makes it public.
- A colouring identifier with a list of colours. `Fill` carries its colours until `escher-group` gives colourings.
- The lift onto the ball as a public function. It lives in the private view beside its screen tangents, as the tangents of the disk do, because the view is its one caller.

`fold` is public so that the integration tests, which use only the public surface (TEST-1 of the coding standard), can read the side sequence for the reference tests and the border oracle.

## 4. Options

### 4.1. Defaults and Limits

RND-4. `RenderOptionsBuilder::build` MUST return `InvalidOption` or `LimitExceeded` for each value outside the column "Accepted" of this table, and the builder MUST start from the column "Default", because the input comes from a user, the limits bound the memory of an image (Section 13), and Section 4.6 of the command-line design relies on these defaults:

| Option | Unit | Default | Accepted | Kind outside the accepted values |
| --- | --- | --- | --- | --- |
| `size` | Pixels | 512 by 512 | Width and height from 1 to 16384 each, and at most $2^{26}$ pixels | `InvalidOption` for 0, `LimitExceeded` above a limit |
| `samples` | Samples along each axis of a pixel | 4 | 1 to 16 | `InvalidOption` for 0, `LimitExceeded` above 16 |
| `border_width` | Output pixels (Section 5.5 of the architecture) | 1 | A finite number from 0 to 1024 | `InvalidOption` for a negative or non-finite number, `LimitExceeded` above 1024 |
| `border_colour` | `Colour` | `Colour::BLACK` | Any | None |
| `background` | `Colour` | `Colour::WHITE` | Any | None |
| `max_folds` | Steps | 1500 | 0 to 100000 | `LimitExceeded` above 100000 |
| `rotation` | Radians, as the yaw, the pitch, and the roll | 0, 0, 0 | Three finite numbers | `InvalidOption` for the first angle that is not finite |
| `light` | A direction in the coordinates of the screen of Section 5.9 | $(-1, 1, 2)$ | Three finite components, not all zero | `InvalidOption` |
| `shading` | `Shading` | `Shading::Phong` | Any | None |
| `fill` | `Fill` | None, given to `builder` | Any | None |

`build` checks the options in the order of the rows of this table and reports the first value that fails. A border width of 0 draws no border, and an unconverged sample still takes the border colour (Section 5.5).

`build` checks `rotation`, `light`, and `shading` for every domain, and the renderer reads them only for a spherical domain. They have no effect on another domain, just as the background has none on a Euclidean domain. `build` normalizes the light direction: it divides the direction by its largest absolute component and then by the length of the result, so that no accepted direction overflows or underflows, and its length has no effect. This order of the two divisions is a choice of this design.

The defaults for the border, the sample grid, the fold cap, and the background are those of `DEC-7`, which proposes an opaque white background and no `transparent` one. Section 11 records the figures behind the cap, and Section 12.3 asks the owner to confirm it. The defaults of the viewing rotation, the light, and the shading are those of `DEC-16`.

### 4.2. Colours

RND-5. The crate MUST take every colour as 8-bit sRGB through `Colour::from_srgb8` and MUST NOT parse colour text, because this design puts the project's parser of colour text (`DEC-29`) in the command line (Section 4.6 of the command-line design), and a parser in the library would add a second place that defines what a colour string means.

`Fill` has no default, so the library states no default fill colour. Section 4.6 of the command-line design states the default colours of `escher render`.

## 5. Rendering a Sample

The crate computes the colour of a pixel in the order of Section 4.1 of the architecture. Sections 5.1 to 5.6 state each part, and Section 5.7 gives the source of each step. Section 5.8 states how each step reads for a Euclidean domain, and where it differs it replaces Sections 5.1 and 5.2. Section 5.9 does the same for a spherical domain, and Section 5.10 states the shading of a sample of the ball. Section 5.11 states how each step reads for a rotation domain.

### 5.1. Screen to Disk

Pixel $(i, j)$ has column $i$ from 0 to $W - 1$, counted from the left, and row $j$ from 0 to $H - 1$, counted from the top. Sample $(a, b)$ of the pixel, with $a$ and $b$ from 0 to $n - 1$, lies at the screen position

$$p = \left(i + \frac{a + 1/2}{n}, \ j + \frac{b + 1/2}{n}\right)$$

in pixels. The disk scale and the disk position of the sample are

$$h = \frac{2}{(1 - 2m)\min(W, H)}, \qquad z = \left(\left(p_1 - \frac{W}{2}\right) h, \ \left(\frac{H}{2} - p_2\right) h\right)$$

with the margin $m = 1/32$. The unit disk then spans $(1 - 2m)$ times the shorter side of the image, centred in the image. The second axis of $z$ points up, which matches the counterclockwise rotation of `hyperbolic::rotation`.

RND-6. The renderer MUST place the samples and compute the disk position by the formulas of this section, because the golden images of Section 8.4 and the snapshots of `escher-cli` fix the position of every sample, and another placement changes every image.

The grid follows the 4 by 4 grid of [NS] (Section 7.3 of the rendering reference). `DEC-12` asks for a small margin on the shorter side, and the value $1/32$ is a choice of this design.

### 5.2. Disk to Hyperboloid

The renderer maps $z$ to the point $x =$ `hyperbolic::from_disk(z)`. When `from_disk` returns `None`, the sample is an outside sample, takes the background colour, and counts in `outside()` (GEO-9 of the geometry design). The canonical view applies no isometry after this map, because the domain design places the centre of the domain at `hyperbolic::ORIGIN` (Section 6.3 of the domain design). The symmetric view of the unit tests applies the isometry of Section 8.1 to $x$ and to both screen tangents.

The renderer also computes the two screen tangents, the partial derivatives of the map of Section 7.2 of the geometry design in $z_1$ and $z_2$, times $h$. With $d = 1 - \lvert z \rvert^2$, the prototype used these formulas:

$$T_1 = h\left(\frac{2}{d} + \frac{4 z_1^2}{d^2}, \ \frac{4 z_1 z_2}{d^2}, \ \frac{4 z_1}{d^2}\right), \qquad T_2 = h\left(\frac{4 z_1 z_2}{d^2}, \ \frac{2}{d} + \frac{4 z_2^2}{d^2}, \ \frac{4 z_2}{d^2}\right)$$

No reference states these derivatives. Section 10 records them as an unverified statement of this design, and the test "Tangents" of Section 8.1 checks them against central differences of the reference map. The renderer computes $d$ as `from_disk` does, so that it divides by the same number.

$T_2$ is the change of $x$ for a step of one pixel up the screen, because the second axis of $z$ points up. The border estimate uses only the size of the change, so the sign does not matter.

### 5.3. Fold

RND-7. The renderer and `fold` MUST fold a point by the steps of this section, in this order, with one private implementation that both call, because the reference tests read the side sequence through `fold` and the images come from the renderer, and two implementations could take different decisions for the same point.

The fold takes a vector $x$, the cap $N$, and, inside the renderer, the screen tangents $T_1$ and $T_2$. For each side $i$ of the domain, in the order of `FundamentalDomain::sides()`, it reads the covector $c_i$, the pairing $P_i$, and the orientation, and computes once the tolerance scale $\sigma_i = \sum_j \lvert c_{ij} \rvert$, the sum of the absolute values of the components of $c_i$:

1. Set the step count and the reversal count to 0.
2. If a coordinate of $x$ is not finite, or the point size $\lambda(x)$ is not positive, stop as unconverged.
3. For each side $i$ in index order, compute the dot product $s_i = c_i \cdot x$. If $s_i$ is NaN, stop as unconverged. If $s_i > \tau \sigma_i \lambda(x)$, side $i$ is violated, so go to step 5 with the first violated side.
4. No side is violated. Stop as converged.
5. If the step count equals $N$, stop as unconverged.
6. Replace $x$ by $P_i x$, and inside the renderer replace $T_1$ by $P_i T_1$ and $T_2$ by $P_i T_2$. Add 1 to the step count, add 1 to the reversal count when the orientation of side $i$ is `Reversing`, and call `visit(i)`. Go to step 2.

The fold returns the vector $x$ as it stands at the stop, with the step count, the reversal count, and whether it converged. Step 2 runs before every side test, so a converged fold returns a finite point with a positive point size, an input whose point size is not positive returns an unconverged result with zero steps, and a cap of 0 returns an unconverged result with zero steps for every point outside the domain.

The point size $\lambda(x)$ is the last coordinate $x_3$ for a hyperbolic or Euclidean domain, and the largest absolute coordinate $\max_j \lvert x_j \rvert$ for a spherical domain, because a point of the sphere with a last coordinate that is not positive is a valid point. The fold computes it once in each turn of the loop, at step 2, and uses it at steps 2 and 3. A private enum with two variants, one for the hyperboloid and the plane and one for the sphere, selects it (TYPE-2 of the coding standard), and `fold` and `Renderer::new` choose the variant from `geometry()` of the domain. For a spherical domain, the point size of a finite vector is positive unless the vector is zero, so the fold stops as unconverged only for a coordinate that is not finite, the zero vector, a NaN side value, or the cap.

RND-26. The fold MUST read the point size at steps 2 and 3 in place of the last coordinate. For a hyperbolic or Euclidean domain it MUST use $x_3$ as the point size, and MUST compute the threshold of step 3 as the product of $\tau$, $\sigma_i$, and $x_3$, taken from left to right, because the committed golden images and the exact tests "Bands", "Scale", and "Periodicity" fix every decision of those domains, and any other order of operations can change one.

A point that violates no side at the start stops at step 4 with zero steps, and `Fold::point` returns its input unchanged, bit for bit, because no arithmetic touches it.

RND-8. The fold MUST NOT renormalize the point or the screen tangents, at any step and at the stop, because renormalization rounds the point by a factor that is noise near the limit circle and changes which sides the fold applies, and on the prototype it produced infinite coordinates (Section 11). The decisions of step 3 do not need it, because both sides of the comparison scale with $x$ (Section 6.1).

The fold computes the dot product itself and calls no side test of `escher-geometry`, so the tolerance of step 3 belongs to this crate, as Section 9.2 of the architecture requires. Section 6.2 states the constant $\tau$.

The parity of a fold is the number of its steps whose pairing reverses orientation, modulo 2, for every domain, because the fold reads the orientation of each side rather than assuming a mirror. A reflection domain has two, three, or four sides of kind `Mirror`, each its own partner, so the fold applies a reflection at every step and its parity equals its step count modulo 2. A rotation domain has two, four, or six sides, and every pairing preserves orientation, so its reversal count stays 0 and its parity is even whatever its step count (Section 5.11). The loop therefore needs no change for a rotation domain, although the accuracy of the fold depends on the pairings (Section 6.8).

### 5.4. Border

RND-9. The renderer MUST mark a converged sample as a border sample exactly when, for some side $i$ of the domain, the expression `!(f_i.abs() >= half_width * g_i)` is true, with $f_i$ the side value at the folded point, $g_i$ the gradient size of this section, and the half width in pixels, because this is the border estimate of `DEC-28`, and the negated comparison makes a NaN side value or a NaN or infinite gradient size give a border sample rather than a fill colour (Section 6.3).

The gradient size of side $i$ is

$$g_i = \sqrt{(c_i \cdot T_1)^2 + (c_i \cdot T_2)^2}$$

with $T_1$ and $T_2$ the screen tangents after the fold. In words, $g_i$ is the change of the side value per pixel on the screen, so $\lvert f_i \rvert / g_i$ estimates the distance in pixels from the sample to the wall of side $i$ in the tile of the sample. The renderer evaluates the condition for every side of the domain, and for no other wall (RND-34 in Section 5.11). The estimate follows the gradient form of Quilez, `abs(v)/length(g)`, and the Jacobian form of Skia (Section 7.1 of the rendering reference). No reference gives it through a chain of folds in the linear model, which is `UNV-15` and item 19 of Section 13.1 of the rendering reference.

Scaling the folded point and both screen tangents by one factor scales $f_i$ and $g_i$ by that factor and leaves the condition unchanged, so a common scale of the unnormalized point and its tangents does not matter (RND-8). No transcendental function runs for a sample, so the platform mathematics library enters the image only through the domain and the sRGB transfer function.

The tests compare this estimate with a second form, the distance form, which only the reference arithmetic evaluates. With $\hat{x}$ the folded point divided by $\sqrt{-\langle x, x \rangle}$ and $\hat{f}_i$ the side value at $\hat{x}$, the distance form marks a border sample when

$$\operatorname{arsinh}\lvert \hat{f}_i \rvert \cdot \frac{1 - \lvert z \rvert^2}{2} < h \cdot w$$

for some side $i$, where $w$ is the half width and $z$ is the disk position of the sample. Its parts are the unit-normal form $\sinh d = \lvert \langle x, u \rangle \rvert$, which is part of `UNV-14` (item 3 of Section 15.1 of the geometry reference), and the disk metric of [CFKP], p. 71 (Section 6.2 of the rendering reference). Zucker converts a hyperbolic distance in the same way (Section 6.1 of the rendering reference). The test "Border forms" of Section 8.1 measures how often the two forms disagree.

### 5.5. Colour of a Sample

RND-10. The renderer MUST give each sample the colour of the first row of this table that applies, because an unconverged sample carries no reliable tile and must not take a fill colour, and the precedents of Section 4.9 of the rendering reference give such a sample a fixed colour:

| Case | Colour |
| --- | --- |
| Outside sample | The background, unshaded |
| Unconverged | The border colour (`DEC-7`) |
| Border sample | The border colour |
| `Fill::One(c)` | $c$ |
| `Fill::Parity([c_0, c_1])` | $c_0$ for an even parity and $c_1$ for an odd parity |

For a spherical domain, the colour of every sample that is not an outside sample is then shaded as Section 5.10 states. The domain itself takes the first colour of `Fill::Parity`, because a point inside it folds in zero steps. Every tile of a rotation domain takes that colour too, because every fold of such a domain has an even parity (RND-33 in Section 5.11). The parity colouring rests on `UNV-18`, with the precedents of the flip parity of [ZUCKER] and [NS] and the swap of two colours of [SYMMHUB] (Section 4.6 of the rendering reference).

### 5.6. Pixel and Image

The colour of a pixel is the mean of the colours of its $n^2$ samples in linear light, each channel separately, after the shading of Section 5.10. The renderer then converts the mean to 8-bit sRGB with `Colour::to_srgb8`, which applies the encoding direction of the sRGB transfer function, clamps the result to the range 0 to 1, and rounds $255$ times it to the nearest integer. `Colour::from_srgb8` applies the decoding direction to each byte divided by 255.

RND-11. The crate MUST implement the sRGB transfer function as Section 7.5 of the rendering reference states it, in both directions, and MUST NOT take a constant of the function from any other place, because a constant that differs from the source shifts every colour of every image without any test that a human would notice.

Averaging in linear light is a choice that no research source supports (Section 5.3 of the plan, Section 4.1 of the architecture).

The buffer holds the pixels row by row, from the top row, each pixel as red, green, and blue bytes. `encode_png` writes it with `png::Encoder`, the colour type RGB, the bit depth 8, and the sRGB chunk through `set_source_srgb` (Section 4 of the ecosystem survey), with the perceptual rendering intent. As a choice of this design, `encode_png` writes the sRGB chunk and no gAMA or cHRM chunk, although the PNG specification recommends that an encoder that writes the sRGB chunk also write gAMA, and optionally cHRM (Section 7.5 of the rendering reference). `encode_png` returns the bytes of the file and touches no file system.

RND-12. `encode_png` MUST write an RGB image of bit depth 8 with the sRGB chunk and no gAMA or cHRM chunk, and MUST return the file as bytes in memory, because the pixels are 8-bit sRGB and a viewer that lacks the chunk may show them in another colour space, and the library leaves the path, the existing file, and the write to the command line.

### 5.7. Source of Each Step

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.1 | A regular grid of samples in each pixel, averaged | [NS] Appendix A.1 (Section 7.3 of the rendering reference) | Sourced precedent. The margin is a choice |
| 5.2 | The map from the disk to the hyperboloid | [RAT] equation (4.5.2), p. 127, and [DUN86], p. 147 (Section 4.2 of the geometry reference), as `hyperbolic::from_disk` | Sourced |
| 5.2 | The screen tangents as partial derivatives of that map | None | Unverified (Section 10) |
| 5.3 | Test each side in order, apply the first violated side, and repeat until no side is violated or the cap is reached | Steps 2 to 5 of Section 4.1 of the rendering reference ([SYMMHUB] `iToFundamentalDomainSampler`), and the reverse pixel lookup of [VGRG] | Sourced loop. Termination and the step count are `UNV-1` |
| 5.3 | The triangle is a strict fundamental domain of the group of its reflections | Theorem 6.4.3 of [DAVIS] (Section 4.2 of the rendering reference) | Sourced |
| 5.3 | The behaviour of a point on or near a wall | None | `UNV-3` |
| 5.3 | The tolerance of the side test | Section 9.2 of the architecture | A choice of this design (Section 6.2) |
| 5.4 | The gradient form of the border estimate through the fold | Quilez and Skia (Section 7.1 of the rendering reference), not through a chain of folds | `UNV-15` |
| 5.4 | The distance form, in the tests only | The disk metric of [CFKP], p. 71 (Section 6.2 of the rendering reference), and the unit-normal form | `UNV-14` |
| 5.5 | An unconverged sample takes a fixed colour | Section 4.9 of the rendering reference | Sourced precedents. The border colour is `DEC-7` |
| 5.5 | The parity colouring | Section 4.6 of the rendering reference | `UNV-18` |
| 5.6 | The sRGB transfer function | Section 7.5 of the rendering reference | Sourced through secondary sources. The rounding of an encoder is item 23 of Section 13.1 of the rendering reference |
| 5.6 | Averaging in linear light | None | A choice of the plan |
| 5.6 | The sRGB chunk through `set_source_srgb` | Section 4 of the ecosystem survey | The API of `png` |

### 5.8. Euclidean Domains

RND-23. `Renderer::new` MUST choose the view from `geometry()` of the domain: the disk view of Sections 5.1 and 5.2 for `Hyperbolic`, the plane view of this section for `Euclidean`, and the ball view of Section 5.9 for `Spherical`, because the fold and the border estimate accept the walls of any domain, and a domain drawn through the view of another geometry gives a plausible but wrong image without an error, as the Euclidean prototype showed for `*632` in the disk (Section 11).

The choice is a `match` on `Geometry` with no wildcard arm (TYPE-7 of the coding standard). Every geometry has a view, so `Renderer::new` stays infallible.

For a Euclidean domain, a point is $(x_1, x_2, 1)$ in the affine plane of Section 5.1 of the architecture, or a positive multiple of it. The covector of side $i$ is $c_i = (a_1, a_2, -t)$ for the wall $a_1 x_1 + a_2 x_2 = t$ with $a_1^2 + a_2^2 = 1$, and each pairing is a matrix whose last row is $(0, 0, 1)$ (Section 6.6 of the geometry design). The domain design places the centre of the domain at the origin $(0, 0, 1)$ (Section 6.5 of the domain design).

The plane scale and the plane position of a sample at the screen position $p$ of Section 5.1 are

$$g = \frac{S}{\min(W, H)}, \qquad z = \left(\left(p_1 - \frac{W}{2}\right) g, \ \left(\frac{H}{2} - p_2\right) g\right)$$

with $S = 8$. The point is $x = (z_1, z_2, 1)$, and the screen tangents are $T_1 = (g, 0, 0)$ and $T_2 = (0, g, 0)$, the change of $x$ for a step of one pixel along each axis of the screen. The view applies no isometry. So the centre of the domain lies at the centre of the image, the shorter side of the image spans 8 units of the plane, and the tiling fills the image with no margin. With the units of `DEC-17`, in which side 0 of a triangle has length 1 and the rectangle of `*2222` has area 1, a tile of `*2222` with `aspect` 1 is 64 pixels square in a 512 by 512 image.

RND-24. For a Euclidean domain, the renderer MUST place the samples by Section 5.1 and compute the plane position, the point, and the screen tangents by the formulas of this section with $S = 8$, because the golden images, the periodicity tests, and the snapshots of `escher-cli` fix the position of every sample, and another placement changes every image. $S$ is a power of two, so that the plane scale is exact for an image whose shorter side is a power of two (Section 6.6).

The other steps read as this table states:

| Step | Reading for a Euclidean domain |
| --- | --- |
| 5.2 | Does not run. The point and the screen tangents are those of this section |
| 5.3 | Unchanged. Step 3 computes the same dot product $c_i \cdot x$, so the one private fold of RND-7 serves both geometries. The last coordinate of the point stays 1 at every step (Section 6.6), so the threshold of the side test is $\tau \sigma_i$ |
| 5.4 | Unchanged (RND-9). The pairings move the screen tangents with the point, so $\lvert f_i \rvert / g_i$ reads as the distance in pixels from the sample to the wall of side $i$ in its tile, as Section 12.1 records |
| 5.5 | Unchanged. No sample is an outside sample, so the background colour does not appear and `outside()` is 0 |
| 5.6 | Unchanged |

A sample whose side value equals the half width times the gradient size is not a border sample, because the comparison of RND-9 then holds. For `*2222` with `aspect` 1 or 4 in a 512 by 512 image with one sample per pixel, every wall lies on a boundary between pixels, so every sample lies exactly half a pixel from the nearest wall. At a border width of 1, no sample is then a border sample and the image shows no border. With a grid of 2 or more samples per pixel the borders show. This is a property of point sampling in this view, and the test "Half-pixel tie" of Section 8.1 records it.

The Euclidean steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.8 | The plane position, the point, and the screen tangents | The affine plane of Section 5.1 of the architecture | A project convention. $S$ is a choice of this design |
| 5.3 | The fold loop | Section 4.1 of the rendering reference, as in Section 5.7 | Sourced loop. Sections 4.5 and 4.7 of the rendering reference give no termination result for the fold of an infinite Euclidean group, so termination and the step count are `UNV-1` |
| 5.3 | The reflection triangles and the rectangle are strict fundamental domains of the groups of their reflections | Theorem 6.4.3 of [DAVIS], which covers the reflection groups of convex polytopes in all three geometries (Section 4.5 of the domain reference) | Sourced |
| 5.4 | The border estimate | SymmHub's tiling overlay, which divides the Euclidean distance to each side by the pixel size (Section 6.1 of the rendering reference), and the gradient form of Section 5.7 | Sourced precedent for the distance. The estimate through a chain of folds is `UNV-15` |
| 5.5 | The parity colouring | Section 4.6 of the rendering reference | `UNV-18` |

### 5.9. Spherical Domains

For a spherical domain, a point is a vector of the unit sphere with the dot product, or a positive multiple of it (Section 5.1 of the architecture). The covector of side $i$ equals its unit normal, and each pairing is the orthogonal matrix of a reflection (Section 6.7 of the geometry design). The domain design places the centre of the domain at $(0, 0, 1)$ (Section 6.7 of the domain design).

The view is an orthographic projection of the ball along the third axis, with the first axis to the right, the second axis up, and the third axis toward the viewer. The ball has the size of the disk of Section 5.1 and the same margin. The ball scale and the ball position of a sample are the disk scale $h$ and the disk position $z$ of Section 5.1. With $d = 1 - \lvert z \rvert^2$, computed with the same operations as `hyperbolic::from_disk` computes it:

- When $d > 0$ is false, which includes the rim itself and a NaN $d$, the sample is an outside sample. It takes the background colour, unshaded, and counts in `outside()`.
- Otherwise, with $\zeta = \sqrt{d}$, the screen point and the screen tangents are

$$s = (z_1, z_2, \zeta), \qquad T_1 = h \left(1, \ 0, \ -\frac{z_1}{\zeta}\right), \qquad T_2 = h \left(0, \ 1, \ -\frac{z_2}{\zeta}\right)$$

- The point is $x = V^{\mathsf{T}} s$, and the screen tangents carried into the fold are $V^{\mathsf{T}} T_1$ and $V^{\mathsf{T}} T_2$, with the viewing matrix $V$.

The ball shows only the front hemisphere of the screen, the points with $s_3 > 0$. The tangents grow as $1/\zeta$ toward the rim, and stay finite for every positive $d$. No source states the lift onto the ball, which only SymmHub's code uses (item 11 of Section 13.1 of the rendering reference), and it is part of `UNV-16`. No source states its tangents either. Section 10 records them as an unverified statement of this design, and the test "Ball tangents" of Section 8.1 checks them against central differences of the reference lift.

The viewing rotation gives three angles in radians: the yaw $a$, the pitch $b$, and the roll $c$. They turn the ball, and the viewer and the light stay fixed. The tiling point $x$ shows at the screen point $V x$, with

$$V = R_z(c) \, R_x(b) \, R_y(a)$$

so the yaw applies first, then the pitch, then the roll, each about a fixed axis of the screen. The signs are defined by the motion on the screen, not by a rule of handedness about an axis:

| Factor | Matrix | A positive angle |
| --- | --- | --- |
| $R_y(a)$ | `spherical::boost(a)` | Moves the front of the ball to the right |
| $R_x(b)$ | The columns $(1, 0, 0)$, $(0, \cos b, -\sin b)$, and $(0, \sin b, \cos b)$ | Moves the front of the ball up |
| $R_z(c)$ | `spherical::rotation(c)` | Turns the image counterclockwise |

$R_x(b)$ is intentionally the matrix that moves the front up for a positive pitch, whatever its handedness about the first axis. Each factor takes its sine and cosine from one call of `f64::sin_cos`. With all three angles 0, every factor has the entries 0 and 1 of the identity, some of them as $-0$, so the default view needs no case of its own. `Renderer::new` builds $V$ once. The rotated view of the unit tests takes any orthogonal matrix in place of $V$.

RND-27. For a spherical domain, the renderer MUST place the samples by Section 5.1, and MUST compute the ball position, $d$, the screen point, the screen tangents, the point, and the viewing matrix by the formulas of this section, because the golden images and the snapshots of `escher-cli` fix the position and the colour of every sample, and another placement or lift changes every image.

RND-28. The viewing matrix MUST apply the yaw, the pitch, and the roll in that order about fixed axes of the screen, with the signs that the table of this section defines by the motion on the screen, and MUST turn the ball while the viewer and the light stay fixed, because the help text of `escher render` describes each angle by that motion (Section 4.6 of the command-line design), and a pitch matrix corrected to a right-handed rotation about the first axis reverses the motion of a positive pitch.

The other steps read as this table states:

| Step | Reading for a spherical domain |
| --- | --- |
| 5.1 | Unchanged. The ball position is the disk position |
| 5.2 | Does not run. The point and the screen tangents are those of this section |
| 5.3 | Unchanged, with the point size of the sphere. Every finite vector other than the zero vector is a point, read as its ray |
| 5.4 | Unchanged (RND-9). The pairings move the screen tangents with the point, as $V^{\mathsf{T}}$ moved them from the screen |
| 5.5 | Unchanged, followed by the shading of Section 5.10 |
| 5.6 | Unchanged. The samples are averaged after shading |

The spherical steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.9 | The lift onto the ball | SymmHub's code only (item 11 of Section 13.1 of the rendering reference) | `UNV-16` |
| 5.9 | The screen tangents of the lift | None | Unverified (Section 10) |
| 5.9 | The viewing matrix and its signs | None | A project convention, and a choice of this design |
| 5.3 | The fold loop on a finite group | Sections 4.1 and 9.3 of the rendering reference | Sourced loop. Termination and the step count are `UNV-1` |
| 5.3 | The point size of the sphere | None | A choice of this design |
| 5.3 | The spherical reflection triangles are strict fundamental domains of the groups of their reflections | Theorem 6.4.3 of [DAVIS] (Section 4.2 of the rendering reference) | Sourced |
| 5.3 | The lune of `*nn` is a strict fundamental domain of the group of its two reflections | None | `UNV-29` |
| 5.4 | The border estimate | The gradient form of Section 5.7 | `UNV-15` |
| 5.5 | The parity colouring | Section 4.6 of the rendering reference | `UNV-18` |
| 5.10 | The Phong reflection model | [W-PHONG], a secondary source, in Section 11.6 of the geometry reference. [PHONG75] is unread | `UNV-16` |
| 5.10 | The shading constants, the two comparisons with 0, and the form of the specular term | None | Choices of this design |

The estimate of RND-9 is first order, and the lift is not, so near the rim the estimate marks more samples as border samples. With a black border the ball shows a faint dark outline about one pixel wide at the rim. Section 6.7 states the range of accuracy of the border estimate by rim distance, and Section 11 records what the sphere prototype measured.

A border of a width in pixels covers a tile that is narrower than the border. At $n = 1000$ the ball of `*n22` and of `*nn` at the default size is therefore entirely border colour, apart from the shading, as the sphere prototype showed (Section 11). This is a property of borders of pixel width on small tiles, and not a defect.

### 5.10. Shading

For a spherical domain with `Shading::Phong`, the renderer shades the colour of each sample that is not an outside sample: fill, border, and unconverged samples alike, as `DEC-16` states for borders. It shades after Section 5.5 chooses the colour and before Section 5.6 averages the samples of the pixel. With `Shading::Flat`, the colour stays unchanged, and the ball is a flat disk.

The normal is the screen point, $N = s$, and $L$ is the normalized light direction of the options, in the same coordinates of the screen. The light is one white light, fixed to the viewer, so the viewing rotation does not move it, and the view direction is $(0, 0, 1)$. The terms are

$$\delta = \max(N \cdot L, \ 0), \qquad \rho = \begin{cases} \max(2 (N \cdot L) N_3 - L_3, \ 0)^{\alpha} & \text{if } N \cdot L > 0 \\ 0 & \text{otherwise} \end{cases}$$

and each linear-light channel $c$ of the colour becomes

$$c' = c \, (k_a + k_d \, \delta) + k_s \, \rho \, (1 - c)$$

with $k_a = 0.25$, $k_d = 0.75$, $k_s = 0.25$, and $\alpha = 32$. The renderer computes the power $\alpha = 32$ by five squarings, so no transcendental function runs for a sample (Section 5.4).

These choices of this design have no source:

- The shading constants. $k_a + k_d = 1$, so a point that faces the light keeps its colour before the specular term.
- The specular term in the form $k_s \rho (1 - c)$, added after the ambient and diffuse terms. In exact arithmetic no channel then leaves the range 0 to 1, so the form needs no clamp. The form of the Phong model with a white specular term, $c (k_a + k_d \delta) + k_s \rho$, needs a clamp at 1, and on the sphere prototype that clamp clipped channels of the default fill colours and turned a white fill into a flat white patch (Section 11).
- The two comparisons with 0, and reading $k_a$ and $k_d$ as multiples of the colour of the sample, which the recorded formula does not state.

A black border gets a grey highlight near the specular peak, because $c = 0$ gives $c' = k_s \rho$.

RND-29. For a spherical domain with `Shading::Phong`, the renderer MUST shade every sample that is not an outside sample by the formula of this section, with these shading constants, before the samples of a pixel are averaged, and MUST compute the specular power by five squarings, because the golden images fix the shaded colour of every sample, and a transcendental function per sample would bring the platform mathematics library into every pixel. With `Shading::Flat` it MUST leave the colour unchanged.

### 5.11. Rotation Domains

For a signature without reflections, the domain design builds a rotation domain: the double of the reflection domain of its starred signature across the doubled wall, with the reflection domain left in place (Section 6.8 of the domain design, DOM-23 to DOM-26). That section states the sides of the double, their partners, their pairings, which are products of two reflections, and their kinds. Every pairing of a rotation domain preserves orientation. Where a corner of order 2 lies on the doubled wall, two sides of the double lie on one wall and meet at a vertex of angle $\pi$.

The renderer chooses the view from `geometry()` of the domain (RND-23), as for every domain. The rotation domain has the centre of its reflection domain, so the canonical view of a rotation signature is the view of its starred signature, and `732` and `*732` show the same tiling in the same place. The owner decided this view (`DEC-12`). The view therefore needs no change for a rotation domain.

The steps of Sections 5.1 to 5.10 read as this table states:

| Step | Reading for a rotation domain |
| --- | --- |
| 5.1, 5.2, 5.8, 5.9 | Unchanged, by the geometry of the domain |
| 5.3 | Unchanged (RND-31). A step applies a pairing that preserves orientation, so the reversal count stays 0. The step counts of a point and of its image under a pairing need not differ by 1 (Section 8.1) |
| 5.4 | Unchanged, over the sides of the rotation domain only (RND-34) |
| 5.5 | Unchanged. `Fill::Parity` gives its first colour to every tile (RND-33) |
| 5.6, 5.10 | Unchanged |

RND-31. The renderer and `fold` MUST fold a point of a rotation domain by the steps of Section 5.3, with no branch on the kind of a side, with the tolerance constant of Section 6.2, and with the order of operations of RND-26, because the loop already reads each pairing as a matrix and each orientation as a flag, and a second fold for rotation domains would be the second implementation that RND-7 forbids.

RND-32. The support of rotation domains MUST NOT change any decision, colour, or statistic that the renderer and `fold` give for a reflection domain, because the committed golden images and the exact tests "Bands", "Scale", and "Periodicity" fix every decision of those domains, and `just update-golden` rewrites every golden file, so a changed image of a reflection domain could enter the repository in the change that adds the new golden files. The acceptance is that no committed golden file of a reflection domain changes.

RND-33. For a domain with no side whose orientation is `Reversing`, `Fill::Parity([c_0, c_1])` MUST give $c_0$ to every converged sample that is not a border sample, and `Renderer::new` MUST stay infallible, because the parity of every fold of such a domain is even, so the rule of Section 5.5 already gives $c_0$, and an error for a fill that the library can draw would make `Renderer::new` fallible for a choice that belongs to the command line (`DEC-32`).

RND-34. The renderer MUST evaluate the condition of RND-9 over the sides of the domain only, so that the doubled wall, which lies inside a rotation domain, draws no border, because the tiles of a rotation signature are copies of its rotation domain (Section 10.1 of the architecture), and a border on the doubled wall would draw the tiling of the reflection domain. Both sides that lie on one wall at a vertex of angle $\pi$ draw that wall.

The estimate of RND-9 tests the whole wall of each side, not only the side itself. This is correct for a domain whose sides lie on its supporting walls, which holds for a convex domain, and the convexity of the double is part of `UNV-31`.

The test "Border subset" of Section 8.1 checks the borders of a rotation domain against those of its reflection domain in the same view. It counts a pixel as a border pixel when it is not the fill colour. Every border pixel of the rotation domain must be a border pixel of the reflection domain, and the rotation domain has at most as many border pixels. At the corner orders near 1000 on the sphere, every pixel of both images within radius 0.9 is a border pixel, so the counts can be equal.

The rotation steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.11 | The double is a fundamental domain of the group of its pairings | Theorem 12 of [SOLOMYAK2020] and Theorem 6.14 of [SERIES2013] for the hyperbolic plane (Sections 4.1 and 4.2 of the domain reference), and Sections 4.9 and 4.10 of the domain reference for the Euclidean plane and the sphere | Sourced theorems. That the double meets their hypotheses is `UNV-31`, and the lune double is `UNV-30` |
| 5.3 | The fold terminates with pairings that are not reflections | None | `UNV-2` |
| 5.4 | The border estimate | The gradient form of Section 5.7 | `UNV-15` |
| 5.5 | One fill colour for a group whose pairings all preserve orientation | None | A consequence of Section 5.5, and `DEC-32` |

## 6. Numerical Behaviour

### 6.1. Scale and Renormalization

The side test of step 3 of Section 5.3 compares $c_i \cdot x$ with $\tau \sigma_i \lambda(x)$, and both scale with $x$. The fold decides on the ray of $x$, not on its point of the hyperboloid or the sphere, so it needs no renormalization. The point size of the sphere, the largest absolute coordinate, is exact and scales exactly with a power of two, as the last coordinate does. Multiplication by a power of two is exact in IEEE 754 arithmetic when no overflow or underflow occurs. So while no product or sum of the side test and of the pairing underflows or overflows, a point scaled by $2^k$ gives each of them scaled by exactly $2^k$, and the fold takes the same decisions. The test "Scale" of Section 8.1 asserts this exactly, and states that reason.

For a spherical domain, every finite vector other than the zero vector is a point, so a small vector reaches the subnormal range, where the products round and the scaling is not exact. A probe outside the repository measured where the decisions change. It took the 17 domains of the spherical domain set and 42,878 points: for each domain, 2000 points of a Fibonacci sphere, the centre and its antipode, the vertices, and the first 200 points of the Fibonacci sphere projected onto each wall. It scaled each point by $2^k$ exactly:

- No $k$ from $-1000$ to 0 changed the result of any point.
- No $k$ from 0 to 1023 changed the result of any point.
- Going down, the first change was at $2^{-1001}$, for point 1579 of the Fibonacci sphere of `*(1000)22`.

The probe points have length 1, so the scaled points have largest absolute coordinates from below $10^{-301}$ up to $2^{1023}$. `fold` states its accuracy for a spherical domain only for a vector whose largest absolute coordinate is at least $10^{-300}$, a limit of this design inside the measured range. The limit rests on that measurement and not on exact arithmetic, because at a point size of $10^{-300}$ the threshold $\tau \sigma_i \lambda(x)$ is below $10^{-314}$, in the subnormal range. Above the limit, a fold can still fail to converge within the cap near a vertex of very high order (Section 6.7), so the range of accuracy of Section 2.1 also requires a converged fold. Point 34 of Section 12.3 gives the alternative of rescaling a small vector.

The point drifts off the hyperboloid during a long fold. The fold does not correct the drift, and Section 6.4 measures the folded point against the reference, which carries the same unnormalized ray, as a hyperbolic distance between the normalized points.

`DEC-31` and Section 9.3 of the architecture state that the fold renormalizes nothing, and that the renderer accumulates no matrix. The screen tangents replace the matrix that a border estimate would need (`DEC-28`), and RND-8 states the rule on renormalization. `DEC-31` is a proposed default, so Section 12.3 asks the owner to confirm it.

### 6.2. Tolerance of the Side Test

RND-13. The tolerance constant $\tau$ MUST be a positive private constant, and the side test MUST use $\tau \sigma_i \lambda(x)$ as its threshold, because a point exactly on a wall or at a vertex of the domain otherwise takes steps between walls that it lies on, and the threshold must scale with the point (Section 6.1).

$\tau$ is a small robustness constant for points on walls and at vertices. It is not a bound on the rounding error of the side test, and no constant $\tau$ bounds that error across the range of accuracy (Section 11). Points within $\tau$ of a wall, relative to $\sigma_i \lambda(x)$, count as inside that side.

$\tau$ is $4 \times 10^{-15}$, a choice of this design. The test "Vertices" of Section 8.1 measures, at every vertex $v$ of the domain of each triple of its set and for every side $i$, the ratio $s_i(v) / (\sigma_i \lambda(v))$, and asserts that 8 times the largest positive ratio is at most $\tau$ and that every vertex folds in zero steps:

| Input set | Largest ratio | Where | Margin of $\tau$ |
| --- | --- | --- | --- |
| The vertices of the 7 triples of the fold set (Section 6.4) and of the 25 vertex triples of Section 8.1 | $3.047 \times 10^{-16}$ | Vertex 1, side 2, of $(923, 6, 5)$ | 13.1 |
| The vertices of the Euclidean domain set, of `*2222` with `aspect` 4, and of `*2222` at 64 values of `aspect` evenly spaced in its logarithm over its range | $6.235 \times 10^{-17}$ | Vertex 0, side 1, of `*2222` with `aspect` 2.438027308408951 | 64.2 |
| The vertices of the spherical domain set, which include the two poles of each lune | $9.308 \times 10^{-17}$ | Vertex 1, side 2, of `*522` | 43.0 |

$\tau$ keeps its value for the Euclidean and the spherical domains, because another value moves the decisions of the hyperbolic fold and changes the committed golden images. Every Euclidean vertex folds in zero steps, and so does every spherical vertex. The margin at the spherical vertices is above 8, so the point size of the sphere stays the largest absolute coordinate (point 22 of Section 12.3 gives the alternative).

The rotation domains do not reach the margin of 8. Their vertex set is the vertices of the rotation domain set, of the rotation domains of the 25 vertex triples of Section 8.1 and of $(868, 416, 5)$, and of `2222` with `aspect` 4 and with the 64 values of `aspect` of the Euclidean row of the table above. On it the test "Vertices" asserts that every vertex folds in zero steps, which holds when the largest positive ratio is at most $\tau$, and prints the margin. It does not assert the factor 8, because the largest ratio of this set leaves a margin of only 3.27. The code measures it with the image vertices and covectors that the domain design builds. Over all 166,665,495 hyperbolic triples with orders up to 1000, the rotation prototype measured the same largest ratio at the same vertex, and no vertex above $\tau$, while on a sample of 235,925 reflection triples it measured a margin of 18 (Section 11):

| Input set | Largest ratio | Where | Margin of $\tau$ |
| --- | --- | --- | --- |
| The vertex set of the rotation domains | $1.224 \times 10^{-15}$ | Vertex 1, side 1, of the rotation domain of $(868, 416, 5)$, an image vertex | 3.27 |

$\tau$ keeps its value for the rotation domains, because a larger value moves the decisions of the reflection domains and changes their committed golden images, which RND-32 forbids. Point 35 of Section 12.3 puts the shortfall to the owner.

With $\tau = 0$ the test "Vertices" fails, because a vertex then takes steps. Over the fold set alone the largest ratio is $7.2 \times 10^{-17}$, and over all 10,622 hyperbolic triples with orders up to 40 and the fold set it is $1.85 \times 10^{-16}$, at vertex 2, side 2, of $(38, 20, 4)$. Those two sweeps ran outside the repository, and the vertex triples include the triple at which each has its largest ratio.

### 6.3. Non-Finite Values

The crate turns a non-finite point and every NaN into an unconverged fold, a border sample, or an outside sample, never into a fill colour, and no non-finite value causes a panic:

| Value | Where | Result |
| --- | --- | --- |
| A non-finite coordinate of the input or of the point after a step | Step 2 of Section 5.3 | Unconverged |
| A point size that is not positive, of the input or of the point after a step: for a hyperbolic or Euclidean domain a last coordinate that is not positive, including a zero of either sign, and for a spherical domain the zero vector | Step 2 of Section 5.3 | Unconverged |
| For a spherical domain, a finite vector with subnormal coordinates, such as $(0, 0, -10^{-320})$ | Step 2 of Section 5.3 | Folded as any other ray. It can converge (Section 11), but it lies below the limit of Section 6.1 and outside the range of accuracy. The argument of Section 6.1 for exact scaling does not hold there, so the test "Scale" draws no such point |
| A side value that is NaN | Step 3 of Section 5.3 | Unconverged |
| A side value that overflows to an infinity for a finite point | Step 3 of Section 5.3 | Compared as a number. A violated side leads to a step, and step 2 then checks the new point |
| A NaN $f_i$, or a NaN $g_i$ | Section 5.4 | Border sample, because a comparison with NaN is false |
| An infinite $g_i$ with a finite $f_i$ | Section 5.4 | Border sample, because $w g_i$ is infinite, or NaN for a half width $w$ of 0 |
| An infinite $f_i$ with a finite $g_i$ | Section 5.4 | Not a border sample by side $i$, because $\lvert f_i \rvert \ge w g_i$ holds |
| A disk position that is NaN | `hyperbolic::from_disk` | Outside sample |
| A ball position that is NaN | Section 5.9 | Outside sample, because $d > 0$ is false |

A rotation angle or a light direction that is not finite never reaches the renderer, because `build` rejects it (Section 4.1). Every other combination of infinite values follows the comparison of RND-9 under IEEE 754. The unit test "NaN border" of Section 8.1 covers a NaN side value, a NaN gradient, and an infinite gradient with a finite side value.

RND-14. `fold` and the renderer MUST NOT panic for any input, including any `f64` bit pattern of the point and any domain that `build` returns, because a panic stops the caller's whole program (ERR-5 of the coding standard), and the input of `fold` is arbitrary in the tests and near the limit circle.

`RenderStats` holds integers only, so no statistic is NaN. A caller computes a mean from `folds()` and `samples() - outside()`.

### 6.4. Range of Accuracy

For a hyperbolic domain, the crate states its accuracy for disk positions up to the radius $1 - 10^{-6}$, the radius band 6. That radius corresponds to a hyperbolic distance of about 14.5 from the centre by the relation of [CFKP], Fact 5 (Section 4.7 of the rendering reference). Beyond it, the prototype measured that the fold in double precision applies a different side than a 256-bit fold, and that `from_disk` returns vectors that are not points from $1 - 10^{-8}$ outward, which never converge (Section 11). Section 6.6 states the range of accuracy for a Euclidean domain. The bounds of this section, and those of Sections 6.6 and 6.7, are those of the reflection domains, and Section 6.8 states the bounds of the rotation domains.

RND-15. For a hyperbolic domain, the renderer and `fold` MUST meet the bounds of the tables of this section in each radius band from 1 to 6, and for a disk position beyond the range of accuracy MUST only return without a panic and report the fold as converged or unconverged, because the bounds are measured only up to that radius, and a promise beyond it would be false for many samples.

The fold applies the pairings that `hyperbolic::reflection` builds to points outside the range of its row in Section 8.4 of the geometry design, which covers distance 3 for the points of `reflection`. This crate relies on its own end-to-end measurements, as Section 9.3 of the domain design does for its construction.

The fold set is the plan triples of Section 12.1 of the domain design and the triple $(1000, 1000, 1000)$, with disk positions at 512 evenly spaced angles in each radius band, which gives 3584 folds per band. Each bound of this section is set as RND-16 states. Each quantity is measured against the reference arithmetic of Section 8.3:

| Quantity | Measure | Measured against | Input set |
| --- | --- | --- | --- |
| Folded point | The hyperbolic distance between the normalized folded point and the normalized result of the reference replay | The replay, in the reference arithmetic, of the side sequence of the fold on the exact `f64` input point and the exact `f64` pairings | The fold set. Every fold of the set converges |
| Fold decisions | The number of folds whose side sequence differs from that of an independent 256-bit fold of the same input point | The 256-bit fold of Section 8.3, with $\tau = 0$ | The fold set |
| Tangents | The largest component of the difference of each tangent, divided by the largest component of the reference tangent | Central differences of the reference `from_disk`, times $h$ | 2048 disk positions with $1 - \lvert z \rvert$ from $10^{-6}$ to 1, log-uniform, at angles that step by the golden angle, with $h$ of a 512 by 512 image |
| Border estimate | The number of samples that the gradient form and the distance form classify differently, at half widths 0.5 and 1.5 pixels | The distance form in the reference | The fold set, keeping the samples whose distance form is under 4 pixels, with $h$ of a 512 by 512 image |
| Colour | The largest difference from the reference transfer function, and the round trip of every byte | The transfer function in the reference | All 256 values in each direction, and 65 evenly spaced values between the current and the obsolete threshold in each direction |
| Fold count | The mean and the largest number of steps | None | The fold set, and the samples of the canonical image of `*(1000)32` at 512 by 512 pixels with a 4 by 4 grid |

The folded point stays within these bounds. Each cell gives the largest distance from the replay, then the bound:

| Triple | Band 1 | Band 2 | Band 3 | Band 4 | Band 5 | Band 6 |
| --- | --- | --- | --- | --- | --- | --- |
| `*732` | $4.0 \times 10^{-14}$, $4 \times 10^{-13}$ | $4.7 \times 10^{-12}$, $4 \times 10^{-11}$ | $3.3 \times 10^{-10}$, $3 \times 10^{-9}$ | $4.4 \times 10^{-8}$, $4 \times 10^{-7}$ | $4.0 \times 10^{-6}$, $4 \times 10^{-5}$ | $3.9 \times 10^{-4}$, $4 \times 10^{-3}$ |
| `*542` | $2.8 \times 10^{-14}$, $3 \times 10^{-13}$ | $3.6 \times 10^{-12}$, $3 \times 10^{-11}$ | $2.9 \times 10^{-10}$, $3 \times 10^{-9}$ | $3.8 \times 10^{-8}$, $4 \times 10^{-7}$ | $3.6 \times 10^{-6}$, $3 \times 10^{-5}$ | $3.2 \times 10^{-4}$, $3 \times 10^{-3}$ |
| `*444` | $3.0 \times 10^{-14}$, $3 \times 10^{-13}$ | $2.6 \times 10^{-12}$, $3 \times 10^{-11}$ | $2.7 \times 10^{-10}$, $3 \times 10^{-9}$ | $2.1 \times 10^{-8}$, $2 \times 10^{-7}$ | $3.0 \times 10^{-6}$, $3 \times 10^{-5}$ | $2.7 \times 10^{-4}$, $3 \times 10^{-3}$ |
| `*433` | $2.8 \times 10^{-14}$, $3 \times 10^{-13}$ | $2.9 \times 10^{-12}$, $3 \times 10^{-11}$ | $2.5 \times 10^{-10}$, $2 \times 10^{-9}$ | $3.5 \times 10^{-8}$, $3 \times 10^{-7}$ | $3.4 \times 10^{-6}$, $3 \times 10^{-5}$ | $3.4 \times 10^{-4}$, $3 \times 10^{-3}$ |
| `*(12)32` | $2.8 \times 10^{-14}$, $3 \times 10^{-13}$ | $3.6 \times 10^{-12}$, $3 \times 10^{-11}$ | $4.6 \times 10^{-10}$, $4 \times 10^{-9}$ | $4.8 \times 10^{-8}$, $4 \times 10^{-7}$ | $3.6 \times 10^{-6}$, $3 \times 10^{-5}$ | $3.9 \times 10^{-4}$, $4 \times 10^{-3}$ |
| `*(1000)32` | $1.3 \times 10^{-12}$, $2 \times 10^{-11}$ | $2.4 \times 10^{-11}$, $2 \times 10^{-10}$ | $5.2 \times 10^{-9}$, $5 \times 10^{-8}$ | $3.4 \times 10^{-7}$, $3 \times 10^{-6}$ | $3.9 \times 10^{-5}$, $4 \times 10^{-4}$ | $3.5 \times 10^{-3}$, $3 \times 10^{-2}$ |
| $(1000, 1000, 1000)$ | $5.2 \times 10^{-14}$, $5 \times 10^{-13}$ | $1.1 \times 10^{-11}$, $9 \times 10^{-11}$ | $8.8 \times 10^{-10}$, $8 \times 10^{-9}$ | $1.5 \times 10^{-7}$, $2 \times 10^{-6}$ | $1.9 \times 10^{-5}$, $2 \times 10^{-4}$ | $1.6 \times 10^{-3}$, $2 \times 10^{-2}$ |

Each bound is 8 times the largest distance, rounded up to one significant digit, so the margins range from 8.1 to 15.3.

The other quantities stay within these bounds:

| Quantity | Largest value | Where | Bound | Margin |
| --- | --- | --- | --- | --- |
| Fold decisions, bands 1 to 5 | 0 in each band | None | 8 in each band, the count bound of 0 | Not applicable |
| Fold decisions, band 6 | 6 | 2 folds of `*732`, 1 of `*542`, 1 of `*433`, and 2 of `*(1000)32` | 48 | 8 |
| Tangents | $7.94 \times 10^{-11}$ | $1 - \lvert z \rvert = 10^{-5.9}$ | $10^{-9}$ | 12.6 |
| Colour, decoding, in linear light | $4.62 \times 10^{-16}$ | Byte 244 | $4 \times 10^{-15}$ | 8.7 |
| Colour, encoding at the 256 stored values, in encoded units | $1.89 \times 10^{-16}$ | Byte 221 | $2 \times 10^{-15}$ | 10.6 |
| Colour, distance of 255 times the encoded value from the byte | $5.57 \times 10^{-14}$ | Byte 221 | $5 \times 10^{-13}$ | 9.0 |
| Colour, decoding between the thresholds 0.03928 and 0.04045 | $2.26 \times 10^{-19}$ | None recorded | $4 \times 10^{-15}$ | 17,733 |
| Colour, encoding between the thresholds 0.00304 and 0.0031308 | $3.57 \times 10^{-18}$ | None recorded | $2 \times 10^{-15}$ | 560 |

The round trip `to_srgb8(from_srgb8(v))` gives `v` for every byte. No byte lies between the current and the obsolete threshold of the sRGB transfer function, so only the 65 values between the thresholds can reveal an obsolete threshold.

The border estimate stays within these count bounds. A cell gives the number of samples that the two forms classify differently, then the count bound:

| Band | Samples within 4 pixels | Half width 0.5 | Half width 1.5 | Largest relative difference of the two distances |
| --- | --- | --- | --- | --- |
| 1 | 3220 | 0, 8 | 2, 16 | 1.0 percent |
| 2 | 3584 | 7, 56 | 0, 8 | 6.7 percent |
| 3 to 6 | 3584 each | 0, 8 | 0, 8 | 6.7 to 8.7 percent, the largest at $(1000, 1000, 1000)$ |

In bands 3 to 6 every sample of the set lies under 0.13 pixels from a wall, so both forms classify every sample as a border sample and the measurement cannot tell the forms apart there. The test "Border oracle" of Section 8.1 checks the border estimate on whole images instead.

The fold counts are these. Each cell gives the mean, then the largest number of steps:

| Triple | Band 1 | Band 2 | Band 3 | Band 4 | Band 5 | Band 6 |
| --- | --- | --- | --- | --- | --- | --- |
| `*732` | 18.1, 22 | 32.6, 36 | 46.9, 51 | 61.1, 66 | 75.3, 79 | 89.4, 95 |
| `*542` | 11.9, 15 | 21.4, 25 | 30.8, 34 | 40.2, 44 | 49.4, 53 | 58.8, 62 |
| `*444` | 5.4, 7 | 9.8, 12 | 14.1, 17 | 18.3, 21 | 22.5, 25 | 26.8, 30 |
| `*433` | 8.7, 11 | 15.7, 19 | 22.6, 26 | 29.5, 32 | 36.3, 39 | 43.3, 46 |
| `*(12)32` | 11.1, 17 | 20.2, 27 | 28.9, 37 | 37.8, 47 | 46.6, 58 | 55.3, 65 |
| `*(1000)32` | 47.9, 259 | 71.8, 968 | 83.4, 968 | 103.0, 971 | 121.7, 977 | 137.5, 991 |
| $(1000, 1000, 1000)$ | 4.2, 8 | 11.4, 87 | 21.6, 637 | 30.6, 698 | 38.5, 748 | 46.8, 749 |

Every fold of the set converges within the default cap of 1500. The canonical image of `*(1000)32` at 512 by 512 pixels with a 4 by 4 grid has 2,895,324 samples inside the disk, with a mean of 31.4 steps, a largest fold of 1000 steps, and no unconverged sample.

Beyond the range of accuracy, the test "Rim" of Section 8.1 asserts no bound. It asserts that each fold returns, that a converged fold has a finite point and finite side values, and that the renderer gives each sample one of the colours of Section 5.5.

### 6.5. Measured Figures

RND-16. A test that asserts a quantity for which this document states no measured bound MUST set its bound and record it in this document as TEST-13 of the coding standard requires, over an input set that this document states, in the same change as the test, and a bound on a count MUST be the count bound of the measured count, because Sections 6.2, 6.4, 6.6, 6.7, 6.8, 8.1, and 8.4 are the one record of the bounds of this crate, a bound outside them has no measurement for a reviewer to check, and 8 times a count of 0 is a bound of 0, which fails on the first decision that differs on another platform.

The count bound is a choice of this design. Each bound of a floating-point quantity is at least 8 times its largest value, as TEST-13 requires. The tests print each largest value with its input and its margin, and the Validating Changes section of `AGENTS.md` gives the command that shows that output.

The figures of Sections 6.2, 6.4, 6.6, 6.7, 6.8, 8.1, and 8.4 were measured on Linux x86-64 with the code of the crate. The figures of Section 11 come from the prototype, the Euclidean prototype, the sphere prototype, and the rotation prototype. Where a figure of the crate differs from one of a prototype, the figure of the crate is the fact, and the prototype's remains only as the evidence that motivated a choice. No bound takes a figure from a prototype, except the golden-image allowance, which Section 8.4 explains.

### 6.6. Euclidean Domains

For a Euclidean domain, precision is lost in these places:

- The sines and cosines of the construction come from the platform mathematics library, as for a hyperbolic domain.
- Each step of the fold rounds with an absolute error that grows with the coordinates of the point, and the step count grows with the distance from the centre, so the error of the folded point grows faster than linearly in that distance (Section 11).
- The side test compares $c_i \cdot x$ with $\tau \sigma_i$, a threshold that does not grow with the coordinates of the point. Section 6.2 measures the ratio at the vertices of the domain.

The last coordinate does not drift. The last row of every Euclidean pairing is exactly $(0, 0, 1)$, so a step on a finite point with last coordinate 1 returns a point with last coordinate exactly 1 in IEEE 754 arithmetic. The test "Euclidean fold properties" of Section 8.1 asserts this exactly.

For `*2222` with `aspect` 1 or 4, an image whose sides are powers of two, and a sample grid of 1, 2, 4, 8, or 16, every value of the construction, of the view, of the fold, and of the border estimate is a dyadic rational that `f64` holds exactly, and every square root that runs has an exact result. IEEE 754 arithmetic then rounds no operation, so the image does not depend on the platform. The test "Periodicity" of Section 8.1 asserts exact equality for these cases only (TEST-13 of the coding standard), and Section 12.1 records the reading.

RND-25. For a Euclidean domain, the renderer and `fold` MUST meet the bounds of the tables of this section for plane positions within distance 65 of the centre of the image, and for a plane position beyond that distance MUST only return without a panic and report the fold as converged or unconverged, because the bounds are measured only up to that distance, and a promise beyond it would be false for samples that reach the fold cap.

The canonical view of an image whose longer side is at most 16 times its shorter side lies within the range of accuracy, because the corner sample of such an image lies less than 65 from the centre. In a more elongated image, such as 16384 by 1 pixels, samples lie far beyond the range, many of them reach the fold cap, and they take the border colour (`DEC-7`). The rows of Section 6.3 apply unchanged.

The Euclidean fold set is the Euclidean domain set at plane positions at 512 evenly spaced angles at each distance 1, 4, 16, and 65 from the centre. The fold cap, the tolerance constant, and the golden allowance keep their values. Each bound is set as RND-16 states, against the reference arithmetic of Section 8.3:

| Quantity | Measure | Measured against | Input set |
| --- | --- | --- | --- |
| Folded point | The distance in the plane between the folded point and the result of the reference replay, for each domain and distance | The replay of the side sequence of the fold in the reference arithmetic | The Euclidean fold set. Every fold of the set converges |
| Fold decisions | The number of folds whose side sequence differs from that of an independent 256-bit fold, for each distance | The 256-bit fold with $\tau = 0$ | The Euclidean fold set |
| Border estimate | The number of samples that the gradient form and the Euclidean distance form of Section 8.1 classify differently, at half widths 0.5 and 1.5 pixels | The Euclidean distance form in the reference | The Euclidean fold set, keeping the samples whose distance form is under 4 pixels, with $g$ of a 512 by 512 image |
| Fold count | The mean and the largest number of steps at each distance | None | The Euclidean fold set, and the samples of the canonical image of each domain of the Euclidean domain set at 512 by 512 pixels with a 4 by 4 grid |

The folded point stays within these bounds. Each cell gives the largest distance from the replay, then the bound, then the margin:

| Domain | Distance 1 | Distance 4 | Distance 16 | Distance 65 |
| --- | --- | --- | --- | --- |
| `*632` | $1.764 \times 10^{-16}$, $2 \times 10^{-15}$, 11.3 | $1.714 \times 10^{-15}$, $2 \times 10^{-14}$, 11.7 | $1.624 \times 10^{-14}$, $2 \times 10^{-13}$, 12.3 | $2.554 \times 10^{-13}$, $3 \times 10^{-12}$, 11.7 |
| `*442` | $2.597 \times 10^{-16}$, $3 \times 10^{-15}$, 11.6 | $9.665 \times 10^{-16}$, $8 \times 10^{-15}$, 8.3 | $7.907 \times 10^{-15}$, $7 \times 10^{-14}$, 8.9 | $1.321 \times 10^{-13}$, $2 \times 10^{-12}$, 15.1 |
| `*333` | $1.572 \times 10^{-16}$, $2 \times 10^{-15}$, 12.7 | $1.152 \times 10^{-15}$, $1 \times 10^{-14}$, 8.7 | $1.006 \times 10^{-14}$, $9 \times 10^{-14}$, 8.9 | $1.434 \times 10^{-13}$, $2 \times 10^{-12}$, 13.9 |
| `*2222`, `aspect` 1 | 0, $3 \times 10^{-15}$ | 0, $2 \times 10^{-14}$ | 0, $2 \times 10^{-13}$ | 0, $3 \times 10^{-12}$ |
| `*2222`, `aspect` 1.618 | 0, $3 \times 10^{-15}$ | $3.331 \times 10^{-16}$, $3 \times 10^{-15}$, 9.0 | $4.330 \times 10^{-15}$, $4 \times 10^{-14}$, 9.2 | $1.075 \times 10^{-13}$, $9 \times 10^{-13}$, 8.4 |
| `*2222`, `aspect` 0.125 | $5.551 \times 10^{-17}$, $5 \times 10^{-16}$, 9.0 | $6.106 \times 10^{-16}$, $5 \times 10^{-15}$, 8.2 | $1.327 \times 10^{-14}$, $2 \times 10^{-13}$, 15.1 | $2.175 \times 10^{-13}$, $2 \times 10^{-12}$, 9.2 |
| `*2222`, `aspect` 8 | 0, $3 \times 10^{-15}$ | $1.110 \times 10^{-15}$, $9 \times 10^{-15}$, 8.1 | $1.088 \times 10^{-14}$, $9 \times 10^{-14}$, 8.3 | $2.276 \times 10^{-13}$, $2 \times 10^{-12}$, 8.8 |

A bound of a cell with a measured distance above 0 is 8 times that distance, rounded up to one significant digit. 8 times a distance of exactly 0 gives no usable bound, so a cell measured as exactly 0 takes the largest bound at the same distance, a choice of this design. Every cell of `*2222` with `aspect` 1, and the cell at distance 1 of `*2222` with `aspect` 1.618 and with `aspect` 8, measured exactly 0.

The other quantities stay within these bounds:

| Quantity | Largest value | Bound |
| --- | --- | --- |
| Fold decisions, at each distance | 0 of 3584 folds | 8, the count bound of 0 |
| Border estimate, at each distance and each half width | 0 samples | 8, the count bound of 0 |
| Fold count | 201 steps, for `*632` at distance 65 | The default cap of 1500, 7.5 times the largest |

The border estimate kept 1106, 1164, 1236, and 1247 samples within 4 pixels at the distances 1, 4, 16, and 65, and the test asserts at least 1000 at each. The largest relative difference of the two distances was 1.0 at distance 1 and $1.006 \times 10^{-4}$ at distance 65, each at the sample of `*632` at angle 256 of 512, which lies on a mirror, where both distances are close to 0. At distances 4 and 16 it was at most $9.362 \times 10^{-14}$. The test prints that difference and asserts no bound on it.

The fold counts are these. Each cell gives the mean, then the largest number of steps:

| Domain | Distance 1 | Distance 4 | Distance 16 | Distance 65 | Image at 512 by 512 with a 4 by 4 grid |
| --- | --- | --- | --- | --- | --- |
| `*632` | 3.0, 6 | 12.1, 15 | 48.2, 52 | 195.9, 201 | 9.22, 18 |
| `*442` | 2.2, 4 | 8.7, 11 | 34.8, 37 | 141.3, 147 | 6.65, 13 |
| `*333` | 2.1, 3 | 8.8, 10 | 35.3, 38 | 143.4, 151 | 6.73, 13 |
| `*2222`, `aspect` 1 | 1.3, 2 | 5.1, 6 | 20.4, 23 | 82.8, 92 | 4.00, 8 |
| `*2222`, `aspect` 1.618 | 1.3, 2 | 5.2, 6 | 20.9, 24 | 85.1, 97 | 4.11, 8 |
| `*2222`, `aspect` 0.125 and 8 | 1.8, 3 | 7.9, 12 | 32.4, 46 | 131.7, 186 | 6.30, 12 |

Every fold of the set converges, and the images have no unconverged and no outside sample.

### 6.7. Spherical Domains

For a spherical domain, precision is lost in these places:

- The sines and cosines of the construction and of the viewing matrix come from the platform mathematics library.
- Each step of the fold rounds, and a fold of `*n22` or `*nn` takes up to about $n$ steps.
- Near the rim, $d = 1 - \lvert z \rvert^2$ cancels, and the screen tangents grow as $1/\zeta$.
- The border estimate is first order, and the lift is not, so it degrades toward the rim.
- One sample spacing near the rim covers a long arc of the sphere, so the limb aliases. This is a property of supersampling, and no bound covers it.
- The shading takes about ten multiplications per channel.

The spherical fold set is the spherical domain set at 512 evenly spaced angles in each rim band of a 512 by 512 image, with the viewing matrix of the default rotation and of the rotation 20, 35, 0 degrees. The fold cap, the tolerance constant, and the golden allowance keep their values. Each bound is set as RND-16 states, against the reference arithmetic of Section 8.3:

| Quantity | Measure | Measured against | Input set |
| --- | --- | --- | --- |
| Folded point | The angle between the normalized folded point and the normalized result of the reference replay, for each domain | The replay of the side sequence of the fold in the reference arithmetic | The spherical fold set |
| Fold decisions | The number of folds whose side sequence differs from that of an independent 256-bit fold, for each domain | The 256-bit fold with $\tau = 0$ | The spherical fold set |
| Lift | The angle between the lifted point and the reference lift, in each rim band | The lift of Section 5.9 in the reference | The spherical fold set |
| Ball tangents | The largest component of the difference of each tangent, divided by the largest component of the reference tangent, in each bin of rim distances | Central differences of the reference lift, times $h$ | 2048 ball positions with rim distances from 0.01 to 240 pixels, log-uniform, at angles that step by the golden angle, with $h$ of a 512 by 512 image |
| Border estimate | The number of samples that the gradient form and the brute-force oracle of the test "Spherical border forms" classify differently, at half widths 0.5 and 1.5 pixels, in each rim band at least one half width from the rim | The oracle | 4000 samples in each rim band of `*532` and of each domain of the spherical golden images |
| Border share | The fraction of border samples in each rim band, at a half width of 0.5 pixels | None. The test prints it and asserts no bound | As the row above |
| Shading | The largest difference of a shaded linear-light channel from the reference | The formula of Section 5.10 in the reference | A grid of normals of the front hemisphere, the default light and three other light directions, and channel values from 0 to 1 |
| Fold count | The mean and the largest number of steps | None | The spherical fold set, and the samples of the canonical image of each domain of the spherical golden images, of `*(1000)22`, and of `*(1000)(1000)`, at 512 by 512 pixels with a 4 by 4 grid |

The folded point and the fold decisions stay within these bounds, over the 7168 folds of each domain. Each bound of the folded point is 8 times the largest angle, rounded up to one significant digit. A decision cell gives the number of folds whose side sequence differs from the 256-bit fold, then the count bound. The last column gives the mean, then the largest number of steps, which the test prints:

| Domain | Largest angle in radians | Bound | Margin | Fold decisions | Fold count |
| --- | --- | --- | --- | --- | --- |
| `*332` | $2.929 \times 10^{-16}$ | $3 \times 10^{-15}$ | 10.2 | 0, 8 | 2.4, 5 |
| `*432` | $3.146 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.5 | 0, 8 | 3.7, 7 |
| `*532` | $4.314 \times 10^{-16}$ | $4 \times 10^{-15}$ | 9.3 | 0, 8 | 6.1, 11 |
| `*222` | $2.776 \times 10^{-16}$ | $3 \times 10^{-15}$ | 10.8 | 0, 8 | 1.2, 2 |
| `*322` | $2.587 \times 10^{-16}$ | $3 \times 10^{-15}$ | 11.6 | 0, 8 | 1.6, 3 |
| `*522` | $2.797 \times 10^{-16}$ | $3 \times 10^{-15}$ | 10.7 | 0, 8 | 2.3, 5 |
| `*(12)22` | $4.561 \times 10^{-16}$ | $4 \times 10^{-15}$ | 8.8 | 0, 8 | 5.0, 12 |
| `*(100)22` | $1.452 \times 10^{-15}$ | $2 \times 10^{-14}$ | 13.8 | 0, 8 | 38.2, 100 |
| `*(999)22` | $4.128 \times 10^{-15}$ | $4 \times 10^{-14}$ | 9.7 | 0, 8 | 377.6, 999 |
| `*(1000)22` | $5.145 \times 10^{-15}$ | $5 \times 10^{-14}$ | 9.7 | 0, 8 | 378.0, 1000 |
| `*22` | $9.949 \times 10^{-17}$ | $8 \times 10^{-16}$ | 8.0 | 0, 8 | 0.8, 2 |
| `*33` | $1.860 \times 10^{-16}$ | $2 \times 10^{-15}$ | 10.8 | 2, 16 | 1.0, 3 |
| `*55` | $2.008 \times 10^{-16}$ | $2 \times 10^{-15}$ | 10.0 | 0, 8 | 1.8, 5 |
| `*(12)(12)` | $3.110 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.6 | 0, 8 | 4.5, 12 |
| `*(100)(100)` | $1.022 \times 10^{-15}$ | $9 \times 10^{-15}$ | 8.8 | 0, 8 | 37.4, 100 |
| `*(999)(999)` | $2.654 \times 10^{-15}$ | $3 \times 10^{-14}$ | 11.3 | 2, 16 | 373.3, 997 |
| `*(1000)(1000)` | $2.622 \times 10^{-15}$ | $3 \times 10^{-14}$ | 11.4 | 0, 8 | 373.6, 998 |

Every fold of the spherical fold set converges within the default cap of 1500. The test prints the worst input of each domain.

The lift stays within these bounds, as the angle from the reference lift over the spherical fold set:

| Rim distance in pixels | Largest angle in radians | Where | Bound | Margin |
| --- | --- | --- | --- | --- |
| 120 | $1.476 \times 10^{-16}$ | Rotation 20, 35, 0 degrees, angle 462 | $2 \times 10^{-15}$ | 13.6 |
| 24 | $2.169 \times 10^{-16}$ | Rotation 20, 35, 0 degrees, angle 9 | $2 \times 10^{-15}$ | 9.2 |
| 8 | $3.134 \times 10^{-16}$ | Rotation 20, 35, 0 degrees, angle 339 | $3 \times 10^{-15}$ | 9.6 |
| 4 | $3.945 \times 10^{-16}$ | Rotation 20, 35, 0 degrees, angle 436 | $4 \times 10^{-15}$ | 10.1 |
| 2 | $4.960 \times 10^{-16}$ | Rotation 20, 35, 0 degrees, angle 160 | $4 \times 10^{-15}$ | 8.06 |
| 1 | $6.973 \times 10^{-16}$ | Default rotation, angle 208 | $6 \times 10^{-15}$ | 8.6 |
| 0.5 | $1.041 \times 10^{-15}$ | Default rotation, angle 330 | $9 \times 10^{-15}$ | 8.6 |

The ball positions of the test "Ball tangents" have continuous rim distances, so the test sorts them into bins, a choice of this design. A position falls in the bin whose lower end is the largest rim distance of the rim bands that does not exceed its own, and a position less than 0.5 pixels from the rim falls in a bin of its own. The ball tangents stay within these bounds:

| Bin of the rim distance in pixels | Largest relative error | Where, as rim distance in pixels | Bound | Margin |
| --- | --- | --- | --- | --- |
| At least 120 | $1.188 \times 10^{-16}$ | 127.74 | $1 \times 10^{-15}$ | 8.4 |
| 24 to less than 120 | $3.535 \times 10^{-16}$ | 24.28 | $3 \times 10^{-15}$ | 8.5 |
| 8 to less than 24 | $7.639 \times 10^{-16}$ | 10.71 | $7 \times 10^{-15}$ | 9.2 |
| 4 to less than 8 | $1.508 \times 10^{-15}$ | 4.35 | $2 \times 10^{-14}$ | 13.3 |
| 2 to less than 4 | $3.094 \times 10^{-15}$ | 2.24 | $3 \times 10^{-14}$ | 9.7 |
| 1 to less than 2 | $6.694 \times 10^{-15}$ | 1.20 | $6 \times 10^{-14}$ | 9.0 |
| 0.5 to less than 1 | $1.108 \times 10^{-14}$ | 0.549 | $9 \times 10^{-14}$ | 8.1 |
| Less than 0.5 | $6.411 \times 10^{-13}$ | 0.0104 | $6 \times 10^{-12}$ | 9.4 |

The viewing matrix and the shading stay within these bounds:

| Quantity | Largest value | Where | Bound | Margin |
| --- | --- | --- | --- | --- |
| Viewing matrix, largest entry of $V^{\mathsf{T}} V - I$, over the seven angle triples of the test "Viewing matrix" | $4.441 \times 10^{-16}$ | $(-7, 100, -0.25)$ radians | $4 \times 10^{-15}$ | 9.0 |
| Viewing matrix, distance of the determinant from 1, over the same triples | $4.441 \times 10^{-16}$ | $(-7, 100, -0.25)$ radians | $4 \times 10^{-15}$ | 9.0 |
| Shading, absolute difference of a shaded linear-light channel | $1.372 \times 10^{-15}$ | Light $(0, -1, 1)$, normal $(0, -0.375, 0.927)$, channel 0 | $2 \times 10^{-14}$ | 14.6 |

The seven angle triples, in radians, are $(0, 0, 0)$, the rotation 20, 35, 0 degrees, $(1, 2, 3)$, $(-2.5, 0.7, 5)$, $(1000, -1000, 0.1)$, $(\pi/2, \pi/2, \pi/2)$, and $(-7, 100, -0.25)$. The shading is measured over the normals of the front hemisphere that lie above a grid of 33 values along each axis of the screen, the default light and the lights $(1, 1, 1)$, $(0, -1, 1)$, and $(2, 0, 1)$, and the channel values $0, 1/16, \ldots, 1$. The reference takes the same `f64` normal and the same normalized `f64` light as the crate, so the test measures the shading arithmetic only. Every shaded channel stays in the range 0 to 1.

The border estimate stays within these count bounds. The test "Spherical border forms" draws 4000 samples in each rim band for each of the five domains of its set, 20,000 samples per cell. A cell gives the number of samples that the two classify differently, then the count bound. A band closer to the rim than the half width has no cell, because the test skips it. The last column gives the border share at a half width of 0.5, which the test prints and does not assert:

| Rim distance in pixels | Half width 0.5 | Half width 1.5 | Border share |
| --- | --- | --- | --- |
| 120 | 0, 8 | 3, 24 | 0.0233 |
| 24 | 3, 24 | 23, 184 | 0.0291 |
| 8 | 8, 64 | 99, 792 | 0.0476 |
| 4 | 39, 312 | 336, 2688 | 0.0653 |
| 2 | 92, 736 | 956, 7648 | 0.0851 |
| 1 | 278, 2224 | None, skipped | 0.1071 |
| 0.5 | 1593, 12744 | None, skipped | 0.1510 |

The range of accuracy of the border estimate on the ball is the rim bands of this table in which its count bound holds. Each count bound is set from the count measured in its own band, so it holds in all seven bands. Point 33 of Section 12.3 asks the owner how to define this range. Closer to the rim, the renderer and `fold` promise only a result without a panic. Section 11 records what the sphere prototype measured.

RND-30. For a spherical domain, `fold` MUST meet the bounds of this section for every vector whose largest absolute coordinate is at least $10^{-300}$ and whose fold converges, the renderer MUST meet them for every ball position inside the rim whose fold converges, and the border estimate MUST meet its count bound in each rim band of the range of accuracy, and for a sample closer to the rim MUST only return without a panic and give the sample one of the colours of Section 5.5. A smaller vector lies beyond the measurement of Section 6.1, an unconverged fold stops at no reliable point, and the border estimate degrades toward the rim, so a promise beyond these limits would be unsupported or false for many inputs.

The fold counts over the spherical fold set are in the first table of this section. The canonical images at 512 by 512 pixels with a 4 by 4 grid, which the test "Spherical statistics" renders, each have 2,895,324 samples inside the ball and no unconverged sample, with these fold counts:

| Domain | Mean steps | Largest steps | Group order |
| --- | --- | --- | --- |
| `*532` | 3.70 | 10 | 120 |
| `*432` | 2.19 | 6 | 48 |
| `*332` | 1.42 | 4 | 24 |
| `*722` | 1.77 | 7 | 28 |
| `*55` | 0.88 | 2 | 10 |
| `*(1000)22` | 215.57 | 1000 | 4000 |
| `*(1000)(1000)` | 181.69 | 499 | 2000 |

The fold of a finite group stops within the cap for the samples of these images and for the images of the centre of the domain, which the test "Centre images" of Section 8.1 folds. It need not stop for every point. On the sphere prototype, for `*(1000)22`, some points within $10^{-14}$ radians of the vertex of order 1000 stayed unconverged even at a cap of 100000, and for `*(999)22` some points took more steps than the longest word of the group (Section 11). In the crate, the test "Near vertex" left 135 of 1000 points near the vertex of order 1000 of `*(1000)22` unconverged at the default cap, and 140 of 1000 near its antipode, each within the cap and without a panic. Such a sample takes the border colour, which is the colour that a sample on a vertex shows. This is part of `UNV-1` (Section 10).

The images of the centre need bounds of their own at $n = 1000$. For `*332`, `*432`, `*532`, and for `*n22` and `*nn` with $n$ from 2 to 12, each image of the centre folds back to within the bound of the folded point of its domain in the first table of this section. A domain whose $n$ is not in the spherical domain set takes the bound of the next larger $n$ of that set, so $n = 4$ takes the bound of $n = 5$, and $n$ from 6 to 11 the bound of $n = 12$. Over these domains the largest angle from the centre was $2.629 \times 10^{-15}$, at `*(10)22`. At $n = 1000$ the images of the centre fold back farther than the bound of the folded point, so the test "Centre images" uses these bounds, measured separately and set as RND-16 states:

| Domain | Largest angle from the centre in radians | Bound of the folded point | Bound of the images of the centre | Margin |
| --- | --- | --- | --- | --- |
| `*(1000)22` | $1.100 \times 10^{-13}$ | $5 \times 10^{-14}$ | $9 \times 10^{-13}$ | 8.2 |
| `*(1000)(1000)` | $3.555 \times 10^{-14}$ | $3 \times 10^{-14}$ | $3 \times 10^{-13}$ | 8.4 |

The likely cause, which no test has confirmed, is that the `f64` pairings are not exact involutions, so the error of the image grows with its depth, which reaches 1001 at `*(1000)22` and 1000 at `*(1000)(1000)`. The replay of the test "Spherical fold replay" multiplies by the same `f64` matrices, so it cannot show this error. No bound from the group order enters the code, and the fold cap stays the only bound. The test "Spherical statistics" compares the largest fold count of each image with the group order.

### 6.8. Rotation Domains

A rotation domain loses precision in the places that Sections 6.4, 6.6, and 6.7 name for its geometry, and in these places of its own:

- Each pairing is the product of two reflections, so it carries the rounding of a product of matrices on top of that of the reflections.
- Near the rim of the disk, the fold of a rotation domain can apply two sides in turn for hundreds of steps, so the defect of a pairing from an isometry can add up. On the rotation prototype, whose pairings were plain `f64` products of the stored reflections, the point of one fold of `(1000)32` in band 6 drifted from the hyperboloid until it was no point, and the fold stopped unconverged, while the fold of the same point for `*(1000)32` converged. With each entry of the products computed in 256-bit arithmetic from the stored reflections and rounded once to the nearest `f64`, the same fold converged, which is why the domain design computes the pairings as compensated products (DOM-25, and Section 14 of the domain design). The compensated product does not lower the largest isometry defect of the pairings (Section 9.6 of the domain design), so the fold of `(1000)32` is the evidence for it. With the compensated products, every fold of the rotation fold sets converges in the crate, including every fold of `(1000)32` in band 6.
- The image vertices of a double lie against image walls with smaller margins than the vertices of a reflection domain (Section 6.2).

The rotation fold sets are the rotation domain set at the positions of the fold sets of the reflection domains: the hyperbolic rotation domains at 512 evenly spaced angles in each radius band from 1 to 6, the Euclidean rotation domains at 512 evenly spaced angles at the distances 1, 4, 16, and 65, and the spherical rotation domains at the positions of the spherical fold set of Section 6.7. The fold cap, the tolerance constant, and the golden allowance keep their values. Each bound is set as RND-16 states, against the reference arithmetic of Section 8.3:

| Quantity | Measure | Measured against | Input set |
| --- | --- | --- | --- |
| Folded point | The measure of Section 6.4, 6.6, or 6.7 for the geometry, for each domain and each band, distance, or domain | The replay of the side sequence of the fold in the reference arithmetic | The rotation fold sets |
| Fold decisions | The number of folds whose side sequence differs from that of an independent 256-bit fold, for each domain and each band, distance, or domain, not summed over the domains as in Section 6.4 | The 256-bit fold with $\tau = 0$ | The rotation fold sets |
| Pairing on the sphere | The angle between the folded points of $x$ and of $P_i x$, for each spherical rotation domain | The distance in the reference | The grid of the test "Spherical rotation pairing" of Section 8.1 |
| Double oracle | The number of points whose folds in the rotation domain and in its reflection domain disagree beyond the sum of their bounds of the folded point, as the test "Double oracle" of Section 8.1 defines it | The fold of the reflection domain | The rotation fold sets, keeping the points whose clearance is that of the test |
| Images of the centre | The angle from the centre of each image of the centre of a spherical rotation domain after its fold | The centre | The products of the test "Centre images" |
| Border subset | The number of border pixels of a rotation domain that are not border pixels of its reflection domain, and the number of border pixels of each | The image of the reflection domain | The images of the test "Border subset" |
| Fold count | The mean and the largest number of steps | None | The rotation fold sets, and the canonical images of `(1000)32`, `(1000)(1000)(1000)`, `(1000)22`, and `(1000)(1000)` at 512 by 512 pixels with a 4 by 4 grid |

The rotation domains take bounds of their own. The folded point of `(1000)32` in band 6 lies up to $4.182 \times 10^{-2}$ from its replay, 1.39 times the bound $3 \times 10^{-2}$ of `*(1000)32` in Section 6.4, and that of $(1000, 1000, 1000)$ in band 3 lies up to $1.910 \times 10^{-8}$ from its replay, 2.39 times the bound $8 \times 10^{-9}$ of $(1000, 1000, 1000)$ in Section 6.4. The images of the centre of `(1000)22` fold back to $2.677 \times 10^{-11}$ radians from the centre, against $1.100 \times 10^{-13}$ for `*(1000)22` in Section 6.7. The likely cause, which no test has confirmed, is that the elements of the largest depths fold back the other way round around the cyclic group, so their error is that of a high power of a pairing.

Each bound of a floating-point quantity in this section is 8 times its largest value, rounded up to one significant digit, unless a table states otherwise. The folded point of a hyperbolic rotation domain stays within these bounds. Each cell gives the largest hyperbolic distance from the replay, then the bound, then the margin:

| Domain | Band 1 | Band 2 | Band 3 | Band 4 | Band 5 | Band 6 |
| --- | --- | --- | --- | --- | --- | --- |
| `732` | $3.134 \times 10^{-14}$, $3 \times 10^{-13}$, 9.6 | $3.442 \times 10^{-12}$, $3 \times 10^{-11}$, 8.7 | $2.582 \times 10^{-10}$, $3 \times 10^{-9}$, 11.6 | $3.559 \times 10^{-8}$, $3 \times 10^{-7}$, 8.4 | $3.915 \times 10^{-6}$, $4 \times 10^{-5}$, 10.2 | $4.849 \times 10^{-4}$, $4 \times 10^{-3}$, 8.2 |
| `542` | $3.667 \times 10^{-14}$, $3 \times 10^{-13}$, 8.2 | $3.923 \times 10^{-12}$, $4 \times 10^{-11}$, 10.2 | $4.179 \times 10^{-10}$, $4 \times 10^{-9}$, 9.6 | $3.355 \times 10^{-8}$, $3 \times 10^{-7}$, 8.9 | $4.606 \times 10^{-6}$, $4 \times 10^{-5}$, 8.7 | $4.086 \times 10^{-4}$, $4 \times 10^{-3}$, 9.8 |
| `444` | $6.115 \times 10^{-14}$, $5 \times 10^{-13}$, 8.2 | $7.011 \times 10^{-12}$, $6 \times 10^{-11}$, 8.6 | $7.770 \times 10^{-10}$, $7 \times 10^{-9}$, 9.0 | $9.661 \times 10^{-8}$, $8 \times 10^{-7}$, 8.3 | $6.816 \times 10^{-6}$, $6 \times 10^{-5}$, 8.8 | $5.436 \times 10^{-4}$, $5 \times 10^{-3}$, 9.2 |
| `433` | $3.094 \times 10^{-14}$, $3 \times 10^{-13}$, 9.7 | $5.935 \times 10^{-12}$, $5 \times 10^{-11}$, 8.4 | $4.848 \times 10^{-10}$, $4 \times 10^{-9}$, 8.3 | $4.472 \times 10^{-8}$, $4 \times 10^{-7}$, 8.9 | $4.142 \times 10^{-6}$, $4 \times 10^{-5}$, 9.7 | $4.816 \times 10^{-4}$, $4 \times 10^{-3}$, 8.3 |
| `(12)32` | $3.187 \times 10^{-14}$, $3 \times 10^{-13}$, 9.4 | $3.390 \times 10^{-12}$, $3 \times 10^{-11}$, 8.8 | $4.995 \times 10^{-10}$, $4 \times 10^{-9}$, 8.0 | $4.668 \times 10^{-8}$, $4 \times 10^{-7}$, 8.6 | $4.445 \times 10^{-6}$, $4 \times 10^{-5}$, 9.0 | $4.206 \times 10^{-4}$, $4 \times 10^{-3}$, 9.5 |
| `(1000)32` | $1.592 \times 10^{-12}$, $2 \times 10^{-11}$, 12.6 | $2.344 \times 10^{-10}$, $2 \times 10^{-9}$, 8.5 | $2.253 \times 10^{-8}$, $2 \times 10^{-7}$, 8.9 | $6.465 \times 10^{-6}$, $6 \times 10^{-5}$, 9.3 | $2.866 \times 10^{-4}$, $3 \times 10^{-3}$, 10.5 | $4.182 \times 10^{-2}$, $4 \times 10^{-1}$, 9.6 |
| `(1000)(1000)(1000)` | $2.536 \times 10^{-13}$, $3 \times 10^{-12}$, 11.8 | $2.864 \times 10^{-11}$, $3 \times 10^{-10}$, 10.5 | $1.910 \times 10^{-8}$, $2 \times 10^{-7}$, 10.5 | $9.185 \times 10^{-7}$, $8 \times 10^{-6}$, 8.7 | $3.336 \times 10^{-5}$, $3 \times 10^{-4}$, 9.0 | $2.443 \times 10^{-2}$, $2 \times 10^{-1}$, 8.2 |

The folded point of a Euclidean rotation domain stays within these bounds. Each cell gives the largest distance in the plane from the replay, then the bound, then the margin:

| Domain | Distance 1 | Distance 4 | Distance 16 | Distance 65 |
| --- | --- | --- | --- | --- |
| `632` | $2.507 \times 10^{-16}$, $3 \times 10^{-15}$, 12.0 | $1.577 \times 10^{-15}$, $2 \times 10^{-14}$, 12.7 | $1.092 \times 10^{-14}$, $9 \times 10^{-14}$, 8.2 | $1.677 \times 10^{-13}$, $2 \times 10^{-12}$, 11.9 |
| `442` | $2.909 \times 10^{-16}$, $3 \times 10^{-15}$, 10.3 | $1.075 \times 10^{-15}$, $9 \times 10^{-15}$, 8.4 | $1.266 \times 10^{-14}$, $2 \times 10^{-13}$, 15.8 | $1.919 \times 10^{-13}$, $2 \times 10^{-12}$, 10.4 |
| `333` | $2.713 \times 10^{-16}$, $3 \times 10^{-15}$, 11.1 | $1.493 \times 10^{-15}$, $2 \times 10^{-14}$, 13.4 | $1.172 \times 10^{-14}$, $1 \times 10^{-13}$, 8.5 | $1.339 \times 10^{-13}$, $2 \times 10^{-12}$, 14.9 |
| `2222`, `aspect` 1 | $1.110 \times 10^{-16}$, $9 \times 10^{-16}$, 8.1 | $4.441 \times 10^{-16}$, $4 \times 10^{-15}$, 9.0 | $1.776 \times 10^{-15}$, $2 \times 10^{-14}$, 11.3 | $7.105 \times 10^{-15}$, $6 \times 10^{-14}$, 8.4 |
| `2222`, `aspect` 1.618 | $2.220 \times 10^{-16}$, $2 \times 10^{-15}$, 9.0 | $4.965 \times 10^{-16}$, $4 \times 10^{-15}$, 8.1 | $4.336 \times 10^{-15}$, $4 \times 10^{-14}$, 9.2 | $1.075 \times 10^{-13}$, $9 \times 10^{-13}$, 8.4 |
| `2222`, `aspect` 0.125 | $2.776 \times 10^{-17}$, $3 \times 10^{-16}$, 10.8 | $6.106 \times 10^{-16}$, $5 \times 10^{-15}$, 8.2 | $5.940 \times 10^{-15}$, $5 \times 10^{-14}$, 8.4 | $9.587 \times 10^{-14}$, $8 \times 10^{-13}$, 8.3 |
| `2222`, `aspect` 8 | $2.220 \times 10^{-16}$, $2 \times 10^{-15}$, 9.0 | $1.256 \times 10^{-15}$, $2 \times 10^{-14}$, 15.9 | $1.090 \times 10^{-14}$, $9 \times 10^{-14}$, 8.3 | $2.276 \times 10^{-13}$, $2 \times 10^{-12}$, 8.8 |

No cell of `2222` measured exactly 0, unlike the cells of `*2222` with `aspect` 1 in Section 6.6, because the pairings of the double are products and not exact, so the rule of Section 6.6 for a cell of 0 has no case here.

The folded point, the fold decisions, and the fold counts of a spherical rotation domain are these, over the 7168 folds of each domain. A decision cell gives the number of folds whose side sequence differs from the 256-bit fold, then the count bound. The last column gives the mean, then the largest number of steps:

| Domain | Largest angle in radians | Bound | Margin | Fold decisions | Fold count |
| --- | --- | --- | --- | --- | --- |
| `332` | $3.310 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.1 | 0, 8 | 1.7, 4 |
| `432` | $3.214 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.3 | 0, 8 | 2.4, 5 |
| `532` | $3.890 \times 10^{-16}$ | $4 \times 10^{-15}$ | 10.3 | 0, 8 | 3.9, 9 |
| `222` | $2.320 \times 10^{-16}$ | $2 \times 10^{-15}$ | 8.6 | 0, 8 | 0.9, 2 |
| `322` | $2.359 \times 10^{-16}$ | $2 \times 10^{-15}$ | 8.5 | 0, 8 | 1.0, 2 |
| `522` | $3.135 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.6 | 0, 8 | 1.4, 3 |
| `(12)22` | $3.457 \times 10^{-16}$ | $3 \times 10^{-15}$ | 8.7 | 0, 8 | 2.7, 6 |
| `(100)22` | $8.952 \times 10^{-16}$ | $8 \times 10^{-15}$ | 8.9 | 0, 8 | 19.4, 50 |
| `(999)22` | $2.872 \times 10^{-15}$ | $3 \times 10^{-14}$ | 10.4 | 0, 8 | 189.1, 500 |
| `(1000)22` | $2.769 \times 10^{-15}$ | $3 \times 10^{-14}$ | 10.8 | 0, 8 | 189.3, 500 |
| `22` | $6.914 \times 10^{-17}$ | $6 \times 10^{-16}$ | 8.7 | 0, 8 | 0.4, 1 |
| `33` | $1.238 \times 10^{-16}$ | $1 \times 10^{-15}$ | 8.1 | 1, 8 | 0.5, 2 |
| `55` | $1.462 \times 10^{-16}$ | $2 \times 10^{-15}$ | 13.7 | 0, 8 | 0.9, 3 |
| `(12)(12)` | $2.562 \times 10^{-16}$ | $3 \times 10^{-15}$ | 11.7 | 0, 8 | 2.2, 6 |
| `(100)(100)` | $7.191 \times 10^{-16}$ | $6 \times 10^{-15}$ | 8.3 | 0, 8 | 18.7, 50 |
| `(999)(999)` | $1.977 \times 10^{-15}$ | $2 \times 10^{-14}$ | 10.1 | 1, 8 | 186.6, 499 |
| `(1000)(1000)` | $2.525 \times 10^{-15}$ | $3 \times 10^{-14}$ | 11.9 | 0, 8 | 186.8, 499 |

The fold decisions of the hyperbolic and Euclidean rotation domains differ from the 256-bit fold in 1 fold of `542` in band 6, 2 folds of `(1000)32` in band 6, and 1 fold of `632` at distance 1. Their count bounds are 8, 16, and 8. Every other domain and band or distance has 0 differing folds and the count bound 8.

The fold counts of the hyperbolic rotation domains are these. Each cell gives the mean, then the largest number of steps:

| Domain | Band 1 | Band 2 | Band 3 | Band 4 | Band 5 | Band 6 |
| --- | --- | --- | --- | --- | --- | --- |
| `732` | 11.4, 16 | 20.4, 26 | 29.3, 36 | 38.1, 46 | 47.1, 56 | 55.9, 67 |
| `542` | 7.1, 9 | 12.9, 15 | 18.5, 22 | 24.2, 27 | 29.8, 34 | 35.5, 40 |
| `444` | 3.6, 6 | 6.5, 9 | 9.4, 12 | 12.3, 16 | 15.0, 20 | 17.9, 24 |
| `433` | 6.1, 8 | 11.1, 13 | 16.0, 18 | 21.0, 24 | 25.9, 29 | 30.8, 35 |
| `(12)32` | 7.1, 14 | 12.7, 22 | 18.1, 29 | 23.6, 34 | 29.1, 44 | 34.4, 49 |
| `(1000)32` | 24.1, 130 | 37.2, 484 | 45.1, 484 | 59.1, 759 | 72.5, 841 | 83.9, 863 |
| `(1000)(1000)(1000)` | 2.8, 8 | 7.6, 87 | 14.6, 637 | 20.9, 666 | 26.1, 667 | 31.0, 668 |

The fold counts of the Euclidean rotation domains are these, in the same form:

| Domain | Distance 1 | Distance 4 | Distance 16 | Distance 65 |
| --- | --- | --- | --- | --- |
| `632` | 1.8, 3 | 7.4, 11 | 27.3, 31 | 108.3, 116 |
| `442` | 1.3, 2 | 5.1, 6 | 20.4, 23 | 82.8, 93 |
| `333` | 1.5, 3 | 7.0, 9 | 29.2, 34 | 120.2, 133 |
| `2222`, `aspect` 1 | 1.0, 2 | 3.8, 5 | 15.3, 19 | 62.1, 73 |
| `2222`, `aspect` 1.618 | 1.0, 2 | 4.2, 6 | 16.9, 22 | 68.9, 87 |
| `2222`, `aspect` 0.125 | 0.9, 2 | 4.4, 7 | 18.0, 24 | 73.2, 95 |
| `2222`, `aspect` 8 | 1.8, 3 | 7.6, 12 | 30.6, 46 | 124.4, 185 |

The canonical images at 512 by 512 pixels with a 4 by 4 grid, which the test "Cap" renders, each have 2,895,324 samples inside the view and no unconverged sample, with these fold counts:

| Domain | Mean steps | Largest steps |
| --- | --- | --- |
| `(1000)32` | 15.83 | 959 |
| `(1000)(1000)(1000)` | 1.80 | 998 |
| `(1000)22` | 107.93 | 500 |
| `(1000)(1000)` | 90.85 | 250 |

The double oracle found no kept point that fails in any domain and band, distance, or domain, so every count bound is 8. Most groups kept 508 to 512 of their 512 points, or 7157 to 7168 of the 7168 points of a spherical domain. Band 6 kept fewer: 388 points of `732`, 416 of `542`, 476 of `444`, 441 of `433`, 426 of `(12)32`, and 10 of `(1000)(1000)(1000)`. Band 5 of `(1000)32` kept 451. Band 6 of `(1000)32` keeps no point, because no point of its fold set lies at its clearance, the larger of $10^{-6}$ and $2(3 \times 10^{-2} + 4 \times 10^{-1}) = 0.86$, from every wall of the domain of `*(1000)32`. So the test "Double oracle" asserts at least one kept point for each domain, and not for each band, a choice of this design.

The images of the centre of the spherical rotation domains, which the test "Centre images" folds, stay within these bounds of their own. The last two columns give the largest depth of an element in the enumeration and the most steps of a fold:

| Domain | Largest angle from the centre in radians | Bound | Margin | Largest depth | Most steps |
| --- | --- | --- | --- | --- | --- |
| `332` | $8.131 \times 10^{-16}$ | $7 \times 10^{-15}$ | 8.6 | 3 | 4 |
| `432` | $1.767 \times 10^{-15}$ | $2 \times 10^{-14}$ | 11.3 | 5 | 6 |
| `532` | $3.298 \times 10^{-15}$ | $3 \times 10^{-14}$ | 9.1 | 8 | 10 |
| `222` | $6.950 \times 10^{-16}$ | $6 \times 10^{-15}$ | 8.6 | 2 | 2 |
| `322` | $8.632 \times 10^{-16}$ | $7 \times 10^{-15}$ | 8.1 | 2 | 3 |
| `422` | $1.221 \times 10^{-15}$ | $1 \times 10^{-14}$ | 8.2 | 3 | 3 |
| `522` | $1.085 \times 10^{-15}$ | $9 \times 10^{-15}$ | 8.3 | 3 | 4 |
| `622` | $1.976 \times 10^{-15}$ | $2 \times 10^{-14}$ | 10.1 | 4 | 4 |
| `722` | $5.576 \times 10^{-16}$ | $5 \times 10^{-15}$ | 9.0 | 4 | 5 |
| `822` | $1.425 \times 10^{-15}$ | $2 \times 10^{-14}$ | 14.0 | 5 | 5 |
| `922` | $3.122 \times 10^{-15}$ | $3 \times 10^{-14}$ | 9.6 | 5 | 6 |
| `(10)22` | $3.597 \times 10^{-15}$ | $3 \times 10^{-14}$ | 8.3 | 6 | 6 |
| `(11)22` | $1.518 \times 10^{-15}$ | $2 \times 10^{-14}$ | 13.2 | 6 | 7 |
| `(12)22` | $2.641 \times 10^{-15}$ | $3 \times 10^{-14}$ | 11.4 | 7 | 7 |
| `(1000)22` | $2.677 \times 10^{-11}$, at depth 500 | $3 \times 10^{-10}$ | 11.2 | 501 | 501 |
| `22` | $8.882 \times 10^{-16}$ | $8 \times 10^{-15}$ | 9.0 | 1 | 1 |
| `33` | $1.110 \times 10^{-16}$ | $9 \times 10^{-16}$ | 8.1 | 1 | 1 |
| `44` | $6.280 \times 10^{-16}$ | $6 \times 10^{-15}$ | 9.6 | 2 | 2 |
| `55` | $3.331 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.0 | 2 | 2 |
| `66` | $9.437 \times 10^{-16}$ | $8 \times 10^{-15}$ | 8.5 | 3 | 3 |
| `77` | $3.331 \times 10^{-16}$ | $3 \times 10^{-15}$ | 9.0 | 3 | 3 |
| `88` | $6.661 \times 10^{-16}$ | $6 \times 10^{-15}$ | 9.0 | 4 | 4 |
| `99` | $4.996 \times 10^{-16}$ | $4 \times 10^{-15}$ | 8.0 | 4 | 4 |
| `(10)(10)` | $3.886 \times 10^{-16}$ | $4 \times 10^{-15}$ | 10.3 | 5 | 5 |
| `(11)(11)` | $1.665 \times 10^{-16}$ | $2 \times 10^{-15}$ | 12.0 | 5 | 5 |
| `(12)(12)` | $4.441 \times 10^{-16}$ | $4 \times 10^{-15}$ | 9.0 | 6 | 6 |
| `(1000)(1000)` | $3.469 \times 10^{-14}$, at depth 252 | $3 \times 10^{-13}$ | 8.6 | 500 | 500 |

Every image of the centre converges, with a step count of at least its depth. The step count exceeds the depth for some elements of `332`, `432`, `532`, `322`, `522`, `722`, `922`, and `(11)22`.

The test "Rotation pairing" takes, for a hyperbolic or Euclidean rotation domain, the bound $b$ of the folded point of the tables above. On a scan outside the repository over the grid of the test "Spherical rotation pairing", the largest ratio of the distance between the two folded points to $b$ was 0.506, for `732`, and 0.331, for `442`, so $b$ holds there with a margin of about 2 to 3. For a spherical rotation domain, $b$ does not bound that distance. The same scan measured ratios of 1.26 for `532`, 1.57 for `22`, 1.35 for `33`, 34 for `(100)22`, 1152 for `(999)22`, and 997 for `(1000)22`, and the property test found a case of `22` at the south pole with a distance of $8.88 \times 10^{-16}$ against a bound of $6 \times 10^{-16}$. The likely cause, which no test has confirmed, is that the error of the `f64` pairings themselves grows with the length of the word, as for the images of the centre. The replay of the test "Rotation fold replay" multiplies by the same pairings, so it cannot show that error.

So the design specifies a pairing bound of its own for each spherical rotation domain, which the test "Rotation pairing" uses for such a domain in place of $b$. The test "Spherical rotation pairing" measures it over a grid of 33 heights and 32 turns, with each kept point paired across every side, and each bound is 8 times the largest distance, rounded up to one significant digit, or $b$ when $b$ is larger. Each domain kept 2078 to 4224 points. The pairing bounds are these:

| Domain | Largest distance in radians | Pairing bound | Margin |
| --- | --- | --- | --- |
| `332` | $1.951 \times 10^{-15}$ | $2 \times 10^{-14}$ | 10.3 |
| `432` | $2.541 \times 10^{-15}$ | $3 \times 10^{-14}$ | 11.8 |
| `532` | $5.051 \times 10^{-15}$ | $5 \times 10^{-14}$ | 9.9 |
| `222` | $1.028 \times 10^{-15}$ | $9 \times 10^{-15}$ | 8.8 |
| `322` | $1.343 \times 10^{-15}$ | $2 \times 10^{-14}$ | 14.9 |
| `522` | $1.443 \times 10^{-15}$ | $2 \times 10^{-14}$ | 13.9 |
| `(12)22` | $2.672 \times 10^{-15}$ | $3 \times 10^{-14}$ | 11.2 |
| `(100)22` | $2.731 \times 10^{-13}$ | $3 \times 10^{-12}$ | 11.0 |
| `(999)22` | $3.456 \times 10^{-11}$ | $3 \times 10^{-10}$ | 8.7 |
| `(1000)22` | $2.990 \times 10^{-11}$ | $3 \times 10^{-10}$ | 10.0 |
| `22` | $9.420 \times 10^{-16}$ | $8 \times 10^{-15}$ | 8.5 |
| `33` | $1.350 \times 10^{-15}$ | $2 \times 10^{-14}$ | 14.8 |
| `55` | $5.512 \times 10^{-16}$ | $5 \times 10^{-15}$ | 9.1 |
| `(12)(12)` | $7.382 \times 10^{-16}$ | $6 \times 10^{-15}$ | 8.1 |
| `(100)(100)` | $1.040 \times 10^{-15}$ | $9 \times 10^{-15}$ | 8.7 |
| `(999)(999)` | $1.963 \times 10^{-15}$ | $2 \times 10^{-14}$ | 10.2 |
| `(1000)(1000)` | $1.330 \times 10^{-15}$ | $3 \times 10^{-14}$, the bound $b$ | 22.6 |

The largest distances of `(999)22` and `(1000)22` lie at height 2, turn 0, and side 2, near the vertex of order $n$. The test prints the worst input of each domain.

The test "Border subset" found no border pixel of a rotation domain outside the border pixels of its reflection domain in any domain, so every count bound is 8. It counted these border pixels:

| Domain | Rotation domain | Reflection domain |
| --- | --- | --- |
| `732` | 4780 | 7819 |
| `542` | 3314 | 5468 |
| `444` | 1764 | 2623 |
| `433` | 2731 | 4106 |
| `(12)32` | 2958 | 5104 |
| `(1000)32` | 7298 | 11248 |
| `(1000)(1000)(1000)` | 1491 | 2222 |
| `632` | 4115 | 7015 |
| `442` | 3024 | 5029 |
| `333` | 3515 | 5121 |
| `2222`, `aspect` 1 | 2284 | 3024 |
| `2222`, `aspect` 1.618 | 2479 | 3028 |
| `2222`, `aspect` 0.125 | 2487 | 4588 |
| `2222`, `aspect` 8 | 4417 | 4588 |
| `2222`, `aspect` 4 | 3442 | 3796 |
| `332` | 487 | 800 |
| `432` | 731 | 1223 |
| `532` | 1167 | 2007 |
| `222` | 312 | 473 |
| `322` | 379 | 587 |
| `522` | 514 | 855 |
| `(12)22` | 972 | 1757 |
| `(100)22` | 6373 | 11595 |
| `(999)22` | 20636 | 20636 |
| `(1000)22` | 20636 | 20636 |
| `22` | 150 | 300 |
| `33` | 164 | 328 |
| `55` | 294 | 588 |
| `(12)(12)` | 612 | 1224 |
| `(100)(100)` | 5298 | 10592 |
| `(999)(999)` | 20636 | 20636 |
| `(1000)(1000)` | 20636 | 20636 |

The four spherical domains of order 999 or 1000 have equal counts, because every pixel within ball radius 0.9 is a border pixel of both images (Section 5.11).

Every fold of the rotation fold sets converges within the default cap, and so does every 256-bit fold of the test "Rotation fold replay". The range of accuracy of every rotation domain is therefore that of its geometry, except near the vertices that RND-35 names. The compensated pairings remove the failure of the rotation prototype, whose fold of `(1000)32` in band 6 did not converge with plain products (Section 11).

RND-35. For a rotation domain, the renderer and `fold` MUST meet the bounds of this section in its range of accuracy, and for a point beyond that range MUST only return without a panic and report the fold as converged or unconverged, because the bounds of Sections 6.4, 6.6, and 6.7 do not hold for a double, whose pairings are products of reflections, and a promise beyond the measured range would be false for many samples. For a hyperbolic rotation domain whose corner orders are all near 1000, such as that of `(1000)(1000)(1000)` or `(1000)(1000)(999)`, the range excludes the points within a hyperbolic distance of $10^{-6}$ of a vertex of order near 1000 other than vertex 0, because the fold of such a point can still lie outside a side after the default cap.

Near those vertices the fold of a rotation domain can fail to stop within the default cap, and at some vertices it does so at larger distances than the fold of the same points in its reflection domain. The test "Near vertex" of Section 8.1 folds the points at 256 evenly spaced directions at each of 13 hyperbolic distances from every vertex of the rotation domains of `(1000)(1000)(1000)`, `(1000)(1000)(999)`, `(998)(981)(961)`, and `(1000)32`, in the rotation domain and in its reflection domain. It measured these numbers of unconverged folds out of 256:

| Domain | Vertex | Folded in | $10^{-5}$ | $3 \times 10^{-6}$ | $10^{-6}$ | $6 \times 10^{-7}$ | $3 \times 10^{-7}$ | $2 \times 10^{-7}$ | $10^{-7}$ | $3 \times 10^{-8}$ | $2 \times 10^{-8}$ | $10^{-8}$ | $10^{-9}$ | $10^{-10}$ | $10^{-11}$ |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `(1000)(1000)(1000)` | 0 | Both domains | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `(1000)(1000)(1000)` | 1 | Rotation domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 124 | 215 | 246 | 204 | 0 |
| `(1000)(1000)(1000)` | 1 | Reflection domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 65 | 244 | 240 | 101 |
| `(1000)(1000)(1000)` | 2 | Rotation domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 111 | 70 | 54 |
| `(1000)(1000)(1000)` | 2 | Reflection domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `(1000)(1000)(1000)` | 3 | Rotation domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 122 | 212 | 250 | 218 | 0 |
| `(1000)(1000)(1000)` | 3 | Reflection domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 71 | 246 | 250 | 204 |
| `(1000)(1000)(999)` | 0 | Both domains | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `(1000)(1000)(999)` | 1 | Rotation domain | 0 | 0 | 0 | 0 | 0 | 0 | 153 | 240 | 246 | 251 | 253 | 226 | 0 |
| `(1000)(1000)(999)` | 1 | Reflection domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 40 | 252 | 251 | 215 |
| `(1000)(1000)(999)` | 2 | Rotation domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `(1000)(1000)(999)` | 2 | Reflection domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 212 | 248 | 197 |
| `(1000)(1000)(999)` | 3 | Rotation domain | 0 | 0 | 0 | 0 | 0 | 0 | 150 | 241 | 246 | 250 | 252 | 226 | 0 |
| `(1000)(1000)(999)` | 3 | Reflection domain | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 35 | 248 | 250 | 204 |
| `(998)(981)(961)` | Every | Both domains | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `(1000)32` | Every | Both domains | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

The rotation domain of `(1000)(1000)(999)` leaves folds unconverged out to $10^{-7}$, and that of `(1000)(1000)(1000)` out to $3 \times 10^{-8}$, while their reflection domains do so only out to $10^{-8}$. Vertex 2 differs between the two signatures. In `(1000)(1000)(1000)` only the rotation domain fails there, and in `(1000)(1000)(999)` only the reflection domain does. Over the 13,312 points of each domain, the rotation domain of `(1000)(1000)(1000)` leaves 1835 unconverged against 1421 for its reflection domain, and that of `(1000)(1000)(999)` leaves 2734 against 2152. The failure is not limited to the cycle of two vertices 1 and 3 of the kite (Section 6.8 of the domain design), because vertex 2 of `(1000)(1000)(1000)`, a cycle of one vertex, fails as well, while vertex 0, also a cycle of one vertex, does not. The exception of RND-35 lies at $10^{-6}$, which is 10 times the largest distance, $10^{-7}$, at which a fold of a rotation domain failed. For each signature, the test asserts that the rotation domain leaves no fold unconverged from the next distance of the list beyond the largest that failed, which is $10^{-7}$ for `(1000)(1000)(1000)` and $2 \times 10^{-7}$ for `(1000)(1000)(999)`. For the other two signatures, where no fold failed, it asserts this at every distance (Section 8.1).

On the rotation prototype, with plain products as pairings, 12 points near the vertices of `(998)(981)(961)` stayed unconverged against 0 for `*(998)(981)(961)`, and with the compensated products the crate leaves none. Such a sample takes the border colour. This is part of `UNV-1` and `UNV-2` (Section 10).

## 7. Errors

### 7.1. Structure

RND-17. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"build render options"`, `"render rows"`, or `"encode image"`.
- `detail`: A private enum with one variant for each cause, which carries the values that the text of Section 7.2 shows, or the error of `png` that caused an encoding failure.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, as Section 10.1 of the domain design states for the same reason: `source()` returns the `png` error that one variant of the detail holds. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `png` error for an `Encoding` error and `None` otherwise. No type from a dependency appears in the public surface of the error (ERR-2).

`Error` derives only `Debug`, because the `png` error that it may hold, `png::EncodingError`, derives only `Debug` and implements neither `Clone` nor `PartialEq`.

### 7.2. Kinds and Text

RND-18. `Display` for `Error` MUST be `cannot `, the operation, `: `, and the text of the detail in this table, because `escher-cli` prints it after `caused by: ` (Section 6.8 of the command-line design) and its snapshots fix it:

| Kind | Operation | When | Text of the detail |
| --- | --- | --- | --- |
| `InvalidOption` | `build render options` | A width or a height of 0 | `the image size 0x512 is empty`, with the size in place of `0x512` |
| `InvalidOption` | `build render options` | A sample grid of 0 | `the sample grid 0 is empty` |
| `InvalidOption` | `build render options` | A negative or non-finite border width | `the border width -1 is negative or not finite`, with the `Display` of the `f64` in place of `-1` |
| `LimitExceeded` | `build render options` | A width or a height above 16384 | `the image size 16385x512 is larger than 16384 pixels on a side`, with the size in place of `16385x512` |
| `LimitExceeded` | `build render options` | More than $2^{26}$ pixels | `the image size 16384x8192 has more than 67108864 pixels`, with the size in place of `16384x8192` |
| `LimitExceeded` | `build render options` | A sample grid above 16 | `the sample grid 17 is larger than 16` |
| `LimitExceeded` | `build render options` | A border width above 1024 | `the border width 1025 is larger than 1024` |
| `LimitExceeded` | `build render options` | A fold cap above 100000 | `the fold cap 100001 is larger than 100000` |
| `InvalidOption` | `build render options` | A rotation angle that is not finite | `the rotation angle NaN is not finite`, with the `Display` of the first such angle, in the order yaw, pitch, roll, in place of `NaN` |
| `InvalidOption` | `build render options` | A light direction with a component that is not finite, or with every component zero | `the light direction is zero or not finite` |
| `InvalidBuffer` | `render rows` | A buffer whose length is 0 or not a multiple of $3W$ | `the buffer of 1535 bytes does not hold whole rows of 1536 bytes`, with the two lengths |
| `InvalidBuffer` | `render rows` | A band that ends below the last row | `the rows 510 to 513 are outside the image of 512 rows`, with the first and last row of the band and the height |
| `InvalidBuffer` | `encode image` | A width or a height of 0, or a buffer whose length is not $3WH$ | `the buffer of 1535 bytes does not hold an image of 16x32 pixels`, with the length and the size |
| `Encoding` | `encode image` | `png` reports an error | `the PNG encoder failed` |

So `RenderOptions::builder(fill).samples(17).build()` returns an `Error` whose `kind()` is `LimitExceeded` and whose `Display` text is exactly `cannot build render options: the sample grid 17 is larger than 16`.

Every fixed text is in lower case apart from `PNG` and has no trailing punctuation (ERR-4 of the coding standard). The values in the text are numbers that the crate formats, so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

`fold`, `Renderer::new`, and `render` return no error. `fold` reports a failure as an unconverged result.

## 8. Testing

### 8.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-render/tests/render/main.rs`, with the modules `common`, `reference`, `golden_images`, `folds`, `properties`, `borders`, `images`, and `encoding` (TEST-10). Its root has a `//!` line. The snapshot tests of the command and the process-level smoke test live in `escher-cli`, and Section 8.1 of the command-line design specifies them.

The module `common` defines a copy of the tolerance constant $\tau$, the bounds of the folded point of Sections 6.4 and 6.6, the helper of RND-21, and the sets that the tests use:

- The plan triples of Section 12.1 of the domain design: $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(1000, 3, 2)$.
- The fold set of Section 6.4: the plan triples and $(1000, 1000, 1000)$, at 512 evenly spaced angles in each radius band from 1 to 6.
- The rim set: the plan triples and $(1000, 1000, 1000)$, at 64 evenly spaced angles at each radius $1 - 10^{-k}$ for $k$ from 7 to 15, and at the largest `f64` below 1 on each axis, which gives 578 positions.
- The 25 vertex triples: $(38, 20, 4)$, which has the largest ratio of Section 6.2 among the hyperbolic triples with orders up to 40, and the 24 worst cases of Section 12.1 of the domain design.
- The Euclidean domain set of Section 2.1 and the Euclidean fold set of Section 6.6.
- The spherical domain set of Section 2.1 and the spherical fold set of Section 6.7.
- The rotation domain set of Section 2.1, the rotation fold sets of Section 6.8, the vertex set of the rotation domains of Section 6.2, and the bounds of the folded point, the counts of the fold decisions and of the double oracle, and the ranges of accuracy of Section 6.8.

The bounds of the images of the centre of the rotation domains are constants of the module `folds`, and the pairing bounds of the spherical rotation domains are constants of the module `properties`, each beside the test that uses it.

The module `reference` defines the count bound once, and both the unit tests and the integration tests take it from there.

The module `golden_images`, in `tests/render/golden_images.rs`, defines once the golden configuration of Section 8.4, with the signatures, their shape parameters, and their file names, the fill colours, the size, the sample grid, and the options, the allowance of 16 pixels, the linear-light table of RND-20, and the comparison. It also defines the helpers of the test "Periodicity": `domain_with_aspect`, which builds a domain with a value of `aspect`, `translation`, which multiplies the pairings of a word and returns the offset of the product with its two defects from a translation, `shifted_pairs`, which pairs each pixel with the pixel at a shift, and the translation tolerance `TRANSLATION_TOLERANCE`. The tests "Golden images", "Perturbation", "Symmetric view", and "Periodicity" take them from there.

Each test names the item of the field "Tests and acceptance" of Section 5.3, 5.4, 5.5, or 5.6 of the plan that it serves. The reference tests print each largest value with its input and its margin, and assert the smallest number of samples that they measure. The test harness captures that output for a test that passes:

| Test | Plan item | Kind and place | Asserts |
| --- | --- | --- | --- |
| Fold replay | Reference arithmetic | Reference test, `folds` | For the fold set, the folded point is within the bound of its band and triple of Section 6.4 of the reference replay of the same side sequence. This covers words of a few hundred reflections, and the longer words of $(1000, 3, 2)$ |
| Fold decisions | Reference arithmetic | Reference test, `folds` | For the fold set, the number of folds whose side sequence differs from the 256-bit fold, summed over the triples, is within the count bound of its band of Section 6.4 |
| Tangents | Reference arithmetic | Reference test, unit test in `view.rs` | The screen tangents of Section 5.2 agree with central differences of the reference `from_disk`, times $h$, within the bound of Section 6.4. This tests the unverified statement of Section 10 on the tangents |
| Border forms | Border oracle | Reference test, unit test in `renderer.rs` | For the samples of the fold set whose distance form is under 4 pixels, at least 1000 in each band, the number of samples that the gradient form and the distance form classify differently at each half width is within the count bound of Section 6.4 |
| Colour | None | Reference test, unit test in `colour.rs` | The transfer function agrees with the reference within the bounds of Section 6.4 for every byte, in both directions, and `to_srgb8(from_srgb8(v))` is `v` for every byte |
| Thresholds | None | Reference test, unit test in `colour.rs` | The private decoding and encoding functions agree with the reference within the bounds of Section 6.4 at 65 evenly spaced values between the current and the obsolete threshold of each direction, which no byte reaches |
| Colour constants | None | Unit tests in `colour.rs` | `BLACK` and `WHITE` equal their constructors, `to_srgb8(from_srgb8(v))` is `v` for every byte, and encoding clamps values outside the range 0 to 1 |
| Reference self-check | None | Reference test, `reference`, which runs in the unit-test binary and in the integration-test binary | The known values of Section 8.3 |
| Inside | Fold properties | `proptest`, `properties` | For a point drawn in the range of accuracy, a converged fold gives a point whose side value for every side is at most $\tau \sigma_i x_3$ |
| Idempotent | Fold properties | `proptest`, `properties` | Folding the folded point of a converged fold takes zero steps and returns the same bits. The comparison is exact, because no arithmetic touches a point that violates no side (Section 5.3) |
| Pairing | Fold properties | `proptest`, `properties` | For a reflection domain of the fold set, a point $x$ in the range of accuracy and a side $i$ with $P_i x$ also in the range of accuracy, let $b$ be the bound of the folded point of Section 6.4 for the triple and the outer of the bands of $x$ and $P_i x$. When the folded point lies, in the reference, at a hyperbolic distance of at least the larger of $10^{-6}$ and $2b$ from every wall of the domain: their folded points are within $b$ of each other in the reference, their step counts differ by exactly 1, and their parities differ. Other cases are discarded |
| Scale | Fold properties | `proptest`, `properties` | For a point in the range of accuracy and each $k$ from $-40$ to 40, the fold of $2^k x$ visits the same sides, takes the same steps and reversals, and has the same `converged()`, and its point is exactly $2^k$ times the folded point. The comparison is exact for the reason of Section 6.1. The test builds $2^k$ by $\lvert k \rvert$ exact doublings or halvings of 1 (Section 8.5) |
| Any input | Fold properties | `proptest`, `properties` | For a hyperbolic domain, any three `f64` bit patterns as the point, and any cap up to 2000, `fold` returns, `steps()` is at most the cap, `reversals()` is at most `steps()`, a converged result has a finite point with a positive last coordinate, and a point whose last coordinate is not positive gives an unconverged result with zero steps |
| Vertices | Fold properties | Two integration tests, `folds`: one for the reflection domains and one for the rotation domains | Every vertex of the domain of each triple of the fold set, of the 25 vertex triples, of the Euclidean domains of Section 6.2, and of the spherical domain set folds converged with zero steps and returns its input, and 8 times the largest ratio of Section 6.2 is at most $\tau$, for the hyperbolic, the Euclidean, and the spherical set apart. Every vertex of the vertex set of the rotation domains of Section 6.2 folds converged with zero steps and returns its input, so its largest ratio is at most $\tau$, with no assertion of the factor 8. It prints one line for each set, with that ratio, its vertex, its side, and its margin |
| Wrong inputs | Failure paths | Integration tests in `folds`, `images`, and `encoding`, and unit tests | The rows of Section 8.2 |
| Rim | Precision at the limit circle | Integration test, `folds` | For the rim set, `fold` returns without a panic, and a converged result has a finite point and finite side values. The test then finds the rows of an image of `*732` at 4096 by 4096 pixels with one sample that hold a sample beyond the range of accuracy, asserts that there is at least one, and renders only those rows with `render_rows`. Every pixel is the fill colour, `Colour::BLACK`, or `Colour::WHITE`. No bound |
| Cap | Termination | Reference test, `folds`, and one integration test, `folds`, for each of the four rotation images, which renders its image in 8 bands of 64 rows on scoped threads | Every fold of the fold set and every sample of the canonical image of `*(1000)32` at 512 by 512 pixels with a 4 by 4 grid within the range of accuracy converges within the default cap of 1500. It prints the mean and the largest step count of each band, which Section 6.4 records. The same holds for every sample of the canonical images of `(1000)32`, `(1000)(1000)(1000)`, `(1000)22`, and `(1000)(1000)` at 512 by 512 pixels with a 4 by 4 grid within their range of accuracy, and the test prints their fold counts, which Section 6.8 records |
| Border oracle | Border oracle | Integration test, `borders` | At one sample per pixel, for each plan triple at 192 by 192 pixels, consider the pixels whose 3 by 3 neighbourhood lies within disk radius 0.9. A change is a pair of neighbouring pixels, to the right or below, whose tiles differ. Every change lies within one pixel of a border pixel, every border pixel lies within one pixel of a change, and both exist. The tile of a sample is the image of the domain centre under the inverse of the pairings that `fold` reports, in reverse order. Two tiles differ when their centres are at least $10^{-4}$ apart |
| Bands | None | Integration test, `images` | For `*732`, and for `*532` with the rotation 20, 35, 0 degrees, at 64 by 48 pixels with a 3 by 3 grid and `Fill::Parity`, `render` equals, byte for byte, the concatenation of `render_rows` over a split into bands of one row, the uneven split into 7, 1, 23, 2, and 15 rows, and one band, and the merged statistics equal those of `render`. The comparison is exact, because each sample is computed by the same instructions whatever the band (RND-3) |
| Symmetric view | Symmetric view | Unit test in `view.rs` | For each plan triple at 192 by 192 pixels, a 4 by 4 grid, and `Fill::One`, the top 96 rows of the image of the symmetric view match the bottom 96 rows mirrored in the first axis, row $j$ against row $H - 1 - j$, within the allowance of Section 8.4: at most 16 differing pixels, each within the limit of RND-20 in each channel |
| Golden images | Golden images | Integration test, `images` | Section 8.4 |
| Perturbation | Golden images | Unit test in `renderer.rs` | For the hyperbolic and Euclidean golden images, adding one unit in the last place to every component of every normal copied into the renderer for a hyperbolic domain, and of every covector for a Euclidean domain, changes each image within the allowance of Section 8.4: at most 16 differing pixels, each within the limit of RND-20 in each channel. For the spherical golden images, the same holds when every component of every normal, of the normalized light direction, and of the viewing matrix moves by one unit in the last place. For the rotation golden images, the same holds when every component of every covector and every entry of every pairing copied into the renderer moves by one unit in the last place, together with the light and the viewing matrix for a spherical one |
| PNG size | PNG decodes to the requested size | Integration test, `encoding` | `encode_png` of a 16 by 32 image decodes with `png` to 16 by 32 pixels, RGB, bit depth 8, with the sRGB chunk and the same bytes |
| Sample position | None | Unit tests in `view.rs` | The position of each sample follows the grid and the margin of Section 5.1, and a sample outside the disk has no point |
| Fold steps | None | Unit tests in `fold.rs` | The private fold carries the screen tangents through each pairing, and counts as reversals only the steps whose pairing reverses orientation |
| NaN border | None | Unit test in `renderer.rs` | The rows of Section 6.3 for a NaN side value, a NaN gradient, and an infinite gradient with a finite side value each give a border sample, and a finite sample inside the domain does not |
| Options | Failure paths | Unit tests in `options.rs` | Each row of Section 4.1 at its limits and one past them, with the kind and the text of Section 7.2, the defaults, the colours and fill kept, and the first failing option in the order of the table. A NaN, positive infinite, and negative infinite value in each angle of `rotation` and in each component of `light`, and the zero light direction, give their texts. The light directions $(10^{300}, 10^{300}, 0)$, $(10^{-310}, 0, 0)$, and $(-4, 4, 8)$ are accepted and normalized to the same bits as $(1, 1, 0)$, $(1, 0, 0)$, and $(-1, 1, 2)$, and the length of each normalized direction lies within $9 \times 10^{-16}$ of 1. The largest measured distance from 1 was $1.110 \times 10^{-16}$, at $(10^{300}, 10^{300}, 0)$, a margin of 8.1 |
| Statistics | None | Unit tests in `renderer.rs` | `merged` adds the counts, saturates at `u64::MAX`, and takes the larger `largest_fold()`. For a 4 by 4 image with one sample of `*732`, `samples()` is 16 and `outside()` counts the corners of the image outside the disk |
| Error text | Failure paths | Unit tests in `error.rs` | The `Display` of every row of Section 7.2, in lower case apart from `PNG` and without trailing punctuation, `kind()` for each cause, and `source()` only for `Encoding` |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs` | Each public type is `Send`, `Sync`, and `'static` (CONC-1 of the coding standard) |
| Plane view | None | Unit tests in `view.rs` | The plane position, the point, and the screen tangents of a sample of a Euclidean domain follow Section 5.8, and the view of each value of `Geometry` is the one that RND-23 names. `render` of `*442` at 4 by 4 pixels with one sample counts no outside sample |
| Euclidean fold properties | Fold properties | `proptest`, `properties` | "Inside", "Idempotent", "Scale", and "Any input" for each domain of the Euclidean domain set, with points drawn within the Euclidean range of accuracy, and a converged fold of such a point has a last coordinate of exactly 1. The comparison is exact for the reason of Section 6.6 |
| Euclidean pairing | Fold properties | `proptest`, `properties` | "Pairing" for each domain of the Euclidean domain set, with the distance in the plane in place of the hyperbolic distance, and with the bound of the folded point of Section 6.6 for the smallest of the distances 1, 4, 16, and 65 that is at least the larger of the distances of $x$ and $P_i x$ from the centre. A case with no such distance is discarded |
| Euclidean fold replay | Reference arithmetic | Reference test, `folds` | For the Euclidean fold set, the folded point lies within the bound of Section 6.6 of the reference replay, the number of differing fold decisions is within its count bound of Section 6.6, and every fold converges within the default cap of 1500. It prints the mean and the largest step count at each distance, which Section 6.6 records |
| Euclidean border forms | Border oracle | Reference test, unit test in `renderer.rs` | As "Border forms", for the Euclidean fold set, against the Euclidean distance form, within the count bounds of Section 6.6. The Euclidean distance form marks a border sample when $\lvert \hat{f}_i \rvert < g w$ for some side $i$, with $\hat{f}_i$ the side value at the folded point divided by its last coordinate, $g$ the plane scale, and $w$ the half width |
| Euclidean border oracle | Border oracle | Integration test, `borders` | As "Border oracle", for each domain of the Euclidean domain set at 193 by 193 pixels with one sample per pixel, over every pixel whose 3 by 3 neighbourhood lies inside the image. The size is odd for the reason that follows this table |
| Rectangle grid | Border oracle | Integration test, `borders` | For `*2222` with `aspect` 1, 4, 1.618, 0.125, 8, and 0.3, at 512 by 512 pixels with one sample per pixel and border widths 1, 1.5, and 3, the number of pixels that differ from the image of an oracle that runs no fold is within 8, the count bound of the measured count of 0 in each of the 18 cases. With $w$ and $h$ the lengths of sides 0 and 1, and $z$ the plane position of the pixel centre, the oracle gives the parity $(i + j) \bmod 2$ with $i = \lfloor (z_1 + w/2)/w \rfloor$ and $j = \lfloor (z_2 + h/2)/h \rfloor$, and marks a border pixel when the distance from $z$ to the nearest line $z_1 = w/2 + kw$ or $z_2 = h/2 + kh$, for an integer $k$, is less than $g$ times the half width |
| Periodicity | Periodicity | Integration test, `images`, for the canonical view, and unit test in `view.rs` for the periodic view | The cases of the table below this one |
| Half-pixel tie | None | Integration test, `images` | `*2222` with `aspect` 1 at 512 by 512 pixels, with one sample per pixel and a border width of 1, has no border pixel, and with a 2 by 2 grid it has border pixels. The assertion of no border pixel is exact for the reason of Section 6.6. Section 5.8 explains the case. The measurement found 0 pixels that differ from the fill at one sample per pixel, and 16128 with the 2 by 2 grid |
| Spherical fold replay | Reference arithmetic | Reference test, `folds` | For the spherical fold set, the folded point lies within the bound of Section 6.7 of the reference replay, the number of differing fold decisions is within its count bound, and every fold converges within the default cap of 1500. It prints the mean and the largest step count of each domain, which Section 6.7 records |
| Spherical fold properties | Fold properties | `proptest`, `properties` | "Inside", "Idempotent", "Scale", and "Any input" for each domain of the spherical domain set, with points drawn in every direction with a length of $2^e$ for an integer $e$ from $-100$ to 100, so that every point that "Scale" folds lies far above the limit of Section 6.1. "Inside" uses the threshold $\tau \sigma_i \lambda(x)$. "Any input" asserts that a converged result has a finite point other than the zero vector, and that the zero vector gives an unconverged result with zero steps |
| Centre images | Exhaustive certificate | Integration test, `folds` | For `*332`, `*432`, `*532`, and `*n22` and `*nn` with $n$ from 2 to 12 and $n = 1000$, each product of the pairings that a breadth-first enumeration finds, with matrices equal within $10^{-9}$ merged, maps the centre of the domain to a point that folds back to within the bound of the images of the centre of Section 6.7, with a step count equal to the depth of the product in the enumeration, and never reaches the cap. For `332`, `432`, `532`, and `n22` and `nn` with the same $n$, each image of the centre folds back to within the bound of the images of the centre of Section 6.8, with a step count at least the depth of the product, and never reaches the cap. The enumeration itself and its count against `group_order()` belong to the domain design |
| Near vertex | Termination | Integration test, `folds` | For `*(1000)22`, 1000 points within $10^{-14}$ radians of the vertex of order 1000 and 1000 points within $10^{-14}$ radians of its antipode each fold without a panic, with at most the default cap of steps, and an unconverged result is allowed. A test of its own folds the points at 256 evenly spaced directions at each of the 13 hyperbolic distances of Section 6.8, from $10^{-5}$ to $10^{-11}$, from every vertex of the rotation domains of `(1000)(1000)(1000)`, `(1000)(1000)(999)`, `(998)(981)(961)`, and `(1000)32`, in the rotation domain and in its reflection domain, each with at most the default cap of steps. It asserts that the rotation domain leaves no fold unconverged at a distance of at least $10^{-7}$ for `(1000)(1000)(1000)`, $2 \times 10^{-7}$ for `(1000)(1000)(999)`, and $10^{-11}$ for `(998)(981)(961)` and `(1000)32`, allows unconverged folds closer to a vertex and in the reflection domain, and prints the counts of both domains for each vertex |
| Ball view | None | Unit tests in `view.rs` | The ball position of each sample follows Section 5.1. The centre sample of an image with odd sides and one sample per pixel has exactly the point $(0, 0, 1)$ and the tangents $(h, 0, 0)$ and $(0, h, 0)$ with the default rotation. A sample on the rim, beyond it, or at a NaN position is an outside sample |
| Ball tangents | Reference arithmetic | Reference test, unit test in `view.rs` | The screen tangents of Section 5.9 agree with central differences of the reference lift, times $h$, within the bound of Section 6.7 in each bin of rim distances, and the lifted point agrees with the reference lift. This tests the unverified statement of Section 10 on the tangents of the ball |
| Viewing matrix | None | Unit tests in `view.rs` | For the seven triples of angles of Section 6.7, $V$ is orthogonal and has determinant 1, within the bounds of Section 6.7. A positive yaw moves the image of $(0, 0, 1)$ to a positive first coordinate, a positive pitch to a positive second coordinate, and a positive roll turns $(1, 0, 0)$ toward a positive second coordinate. With all angles 0, $V s$ equals $s$ for every screen point of the test |
| Rotation by a group element | Rotating the view by a group element leaves the image unchanged, with shading on and off | Unit test in `view.rs`, through the rotated view | For `*532` with the default rotation and with the rotation 20, 35, 0 degrees, for `*55`, and for `*722`, at 192 by 192 pixels with a 4 by 4 grid, with `Shading::Phong` and `Shading::Flat`: for $g$ a product of two pairings and for a product of four pairings, the image with $V g$ in place of $V$ equals the image with $V$, and for $g$ one pairing, the image with `Fill::Parity` equals the image with $V$ and the two fill colours exchanged. Each comparison is within the golden allowance of Section 8.4, because the folded points differ in their last bits. The sphere prototype measured no differing pixel (Section 11) |
| Brightest point | The brightest point follows the light direction | Unit tests in `renderer.rs` | Over a grid of normals of the front hemisphere, the private shading function has its diffuse term largest at $N = L$ and its specular term largest where $N$ is the normalized sum of $L$ and $(0, 0, 1)$. For `*532` with `Fill::One` of the mid-grey $(128, 128, 128)$, a border width of 0, a 4 by 4 grid, and a black background, at 192 by 192 pixels, the centroid of the pixels that tie for the brightest red byte lies within one pixel along each axis of the predicted pixel, for the default light and the lights $(1, 1, 1)$, $(0, -1, 1)$, and $(2, 0, 1)$. The predicted pixel is the first pixel, in row order, whose centre gives the largest value of the formula of Section 5.10 for the normal of the ball at that centre. The image with the light turned a quarter turn about the third axis equals the image turned a quarter turn, within the golden allowance |
| Shading | Reference arithmetic | Reference test and unit tests in `renderer.rs` | The shaded channel agrees with the reference within the bound of Section 6.7. Every shaded channel of the grid of that row lies in the range 0 to 1 in `f64`. `Shading::Flat` leaves every colour unchanged. With `Shading::Phong`, the shading function gives these values, which the test asserts as exact equalities: with the normal and the light both $(0, 0, 1)$, the channels 0, 0.5, and 1 give 0.25, 0.625, and 1, and with the normal $(1, 0, 0)$, the light $(-1, 0, 0)$ or $(0, 0, 1)$ gives 0 for the channel 0 and 0.125 for the channel 0.5, and the light $(0.6, 0, 0.8)$ gives 0 for the channel 0. The last input faces the light but reflects it away from the viewer, so the specular term is 0. The reference test and the range test of this row cover the general formula |
| Spherical border forms | Border oracle | Reference test, unit test in `renderer.rs` | For the samples of each rim band of Section 6.7 that lies at least one half width from the rim, the number of samples that the gradient form and the brute-force oracle classify differently is within the count bound of Section 6.7, at half widths 0.5 and 1.5. The oracle marks a sample as near a wall when some of 64 evenly spaced points on the circle of the half width around its screen position lifts to another tile than the sample. A point of the circle outside the ball is skipped, so leaving the ball does not count as a border, and a point of the circle whose fold does not converge counts as another tile. The sample itself must converge, or the test fails. The tile is that of the test "Border oracle". The test skips the bands closer to the rim than the half width and prints that it does. It also prints the border share of each band |
| Spherical border oracle | Border oracle | Integration test, `borders` | As "Border oracle", for `*532`, `*432`, `*332`, `*722`, and `*55` at 192 by 192 pixels with one sample per pixel, the default rotation, and `Shading::Flat`, over the pixels whose 3 by 3 neighbourhood lies within ball radius 0.9. The tile of a sample is the image of the domain centre under the inverse of the pairings that `fold` reports, in reverse order, and two tiles differ when their centres are at least $10^{-4}$ apart |
| Spherical statistics | Every sample folds within the bound | Three integration tests, `images`: one for the domains of the spherical reflection golden images, one for `*(1000)22`, and one for `*(1000)(1000)` | For the canonical image of each domain of the spherical reflection golden images, whose signatures start with `*`, of `*(1000)22`, and of `*(1000)(1000)` at 512 by 512 pixels with a 4 by 4 grid, `largest_fold()` is less than `group_order()` of the signature, and `unconverged()` is 0. `outside()` of a 4 by 4 image of `*532` with one sample counts the four corners, in the test of the golden domains. Each test prints the fold counts that Section 6.7 records |
| Rotation fold replay | Reference arithmetic | Four reference tests, `folds`: the hyperbolic, the Euclidean, the first ten spherical, and the last seven spherical rotation domains, which are the lunes | For the rotation fold sets, the folded point lies within the bound of Section 6.8 of the reference replay, the number of differing fold decisions is within its count bound, and every fold in the range of accuracy converges within the default cap of 1500. It prints the mean and the largest step count of each domain and band, distance, or domain, which Section 6.8 records |
| Rotation fold properties | Fold properties | `proptest`, `properties` | "Inside", "Idempotent", "Scale", and "Any input" for each domain of the rotation domain set, with points drawn as the rows of its geometry draw them. "Inside" discards a case only when the domain is the hyperbolic rotation domain of `(1000)(1000)(1000)` and its fold stopped unconverged after exactly the default cap of steps, because the range of accuracy of that domain excludes points near its vertices (RND-35) and a drawn point can fold there. The discard cannot hide a defect of another domain, or of a fold that stops in another way, because each of those cases fails the test when unconverged, and the tests "Rotation fold replay" and "Cap" assert that every fold of the fold set and every sample of the canonical image of `(1000)(1000)(1000)` converge. A converged fold of a Euclidean rotation domain has a last coordinate of exactly 1, because the last row of a product of two Euclidean pairings is exactly $(0, 0, 1)$ |
| Rotation pairing | Fold properties | `proptest`, `properties` | For a rotation domain of the rotation domain set, a point $x$ in the range of accuracy, and a side $i$ with $P_i x$ also in the range of accuracy, let $b$ be the bound of the folded point of Section 6.8 for the domain and for the outer band or distance of $x$ and $P_i x$, or the pairing bound of Section 6.8 for a spherical rotation domain. When the folded point lies, in the reference, at a distance of at least the larger of $10^{-6}$ and $2b$ from every wall of the domain, in the measure of its geometry: their folded points are within $b$ of each other in the reference, and their parities are equal. The test asserts nothing about their step counts. Other cases are discarded, and so is a case whose fold of $x$ or of $P_i x$ stops as "Inside" of "Rotation fold properties" allows, for the reason and with the limits that row states |
| Spherical rotation pairing | Fold properties | Reference test, `properties` | For each spherical rotation domain, at the points of a grid of 33 heights from $-1$ to 1 and 32 evenly spaced turns, every fold converges. A point $x$ is kept when its folded point lies, in the reference, at an angle of at least the larger of $10^{-6}$ and twice the pairing bound of Section 6.8 from every wall of the domain. For each kept point and each side $i$, the fold of $P_i x$ converges, and the angle between the two folded points is within the pairing bound of the domain. It prints the largest angle of each domain with its input |
| Double oracle | Fold properties | Four reference tests, `folds`, split as "Rotation fold replay" is | For each point $x$ of the rotation fold sets, let $b_F$ be the bound of the folded point of the reflection domain for its band, distance, or domain in Sections 6.4, 6.6, and 6.7, and $b_D$ that of the rotation domain in Section 6.8. Keep $x$ when the fold of $x$ in the reflection domain converges and its folded point lies, in the reference, at a distance of at least the larger of $10^{-6}$ and $2(b_F + b_D)$ from every wall of the reflection domain. The point of the fold of $x$ in the rotation domain must lie within $b_F + b_D$ of the folded point of the reflection domain when that fold has an even parity, and of its image under the pairing of side 1 of the reflection domain when the parity is odd. The number of kept points that fail is within the count bound of Section 6.8 for each band, distance, or domain, at least one point of each domain is kept, and the test prints the points that fail. It uses only the public surface, because the reflection domain lies inside the rotation domain in the same place |
| Border subset | Every border pixel of the `pqr` image is a border pixel of the `*pqr` image in the same view | Integration test, `borders` | For each domain of the rotation domain set, with `2222` also at `aspect` 4, render it and its reflection domain with the same options: `Fill::One`, one sample per pixel, and `Shading::Flat`, at 193 by 193 pixels for a Euclidean domain and 192 by 192 pixels otherwise. Count only the pixels within disk or ball radius 0.9 for a hyperbolic or spherical domain. A border pixel is a pixel that is not the fill colour. The number of border pixels of the rotation domain that are not border pixels of the reflection domain is within the count bound of Section 6.8, the rotation domain has at least one border pixel, and it has at most as many border pixels as its reflection domain. The test prints both counts of each domain |
| One-colour parity | None | Integration test, `images` | For `732` and `55` at 64 by 64 pixels with a 2 by 2 grid, the image with `Fill::Parity` of two different colours equals, byte for byte, the image with `Fill::One` of its first colour. The comparison is exact, because each sample takes the same colour by the same instructions (RND-33) |
| Doctests | None | Doctests | The examples of the crate-level and item documentation |

The tests "Tangents", "Border forms", "Colour", "Thresholds", "Colour constants", "Perturbation", and "Symmetric view" read private items, such as the screen tangents, the linear value of a `Colour`, and the walls inside the renderer, so they are unit tests. `src/lib.rs` loads `tests/render/reference.rs` and `tests/render/golden_images.rs` under `#[cfg(test)]` through `#[path]`, so the unit tests and the integration tests use the same reference module, which shares no code with the crate (RND-19), and the same golden configuration, allowance, and comparison. The module `reference` carries `#![expect(dead_code, reason = "the unit tests of the crate and the integration tests each use a different part of this module")]`, because each test binary uses a different part of it, and its self-check runs in both binaries.

The test "Border oracle" measured these counts of pixels:

| Triple | Changes | Border pixels |
| --- | --- | --- |
| `*732` | 11653 | 7317 |
| `*542` | 8463 | 5110 |
| `*444` | 4266 | 2444 |
| `*433` | 6534 | 3844 |
| `*(12)32` | 7974 | 4760 |
| `*(1000)32` | 14776 | 10861 |

The test "Euclidean border oracle" runs at 193 by 193 pixels, a choice of this design. At 192 by 192 pixels the vertical mirrors of `*632` fall exactly on boundaries between pixels, so the two neighbouring pixel centres lie half a pixel from the mirror, and rounding decides which of them is a border pixel. The measurement at that size found 172 pixels of `*632` where the tile changes with no border pixel within one pixel, while the other six domains passed. At 193 by 193 pixels all seven domains pass, with these counts of pixels:

| Domain | Changes | Border pixels |
| --- | --- | --- |
| `*632` | 12001 | 6893 |
| `*442` | 8560 | 4953 |
| `*333` | 8900 | 5045 |
| `*2222`, `aspect` 1 | 5856 | 2992 |
| `*2222`, `aspect` 1.618 | 5872 | 2996 |
| `*2222`, `aspect` 0.125 and 8 | 8992 | 4540 |

The test "Spherical border oracle" renders its border mask with `Shading::Flat`, a choice of this design, because Phong shading turns a black border grey near the highlight, so a border pixel there is not exactly black. It measured these counts of pixels:

| Domain | Changes | Border pixels |
| --- | --- | --- |
| `*532` | 3434 | 1941 |
| `*432` | 2093 | 1177 |
| `*332` | 1418 | 778 |
| `*722` | 1915 | 1039 |
| `*55` | 1068 | 548 |

The test "Brightest point" compares the centroid of the tied brightest pixels, a choice of this design, because the 8-bit values form a plateau at the highlight on which several pixels tie, and the tied pixels can span 2 pixels. A rule that every tied pixel lie within one pixel fails for the light $(0, -1, 1)$, with a tied pixel at $(97, 130)$ against the predicted $(95, 130)$. The measured predicted pixels were $(76, 76)$, $(125, 66)$, $(95, 130)$, and $(143, 95)$ for the four lights, and each centroid lay within one pixel. The background is black, because a white background would be brighter than any pixel of the ball. The image with the light turned a quarter turn differed from the turned image in no pixel.

The test "Rotation by a group element" uses the words $(0)$, $(0, 1)$, and $(0, 1, 0, 2)$ of side indices, where an index past the last side of the domain is replaced by the last side, so the lune `*55` takes $(0, 1, 0, 1)$ as its word of four pairings. All 24 comparisons measured no differing pixel. The test "Perturbation" measured no differing pixel in any of the 20 golden images.

In the test "Rectangle grid", no pixel differed from the oracle in any case. The oracle gave these counts of border pixels:

| `aspect` | Width 1 | Width 1.5 | Width 3 |
| --- | --- | --- | --- |
| 1 | 0 | 16128 | 16128 |
| 4 | 0 | 20224 | 20224 |
| 1.618 | 8132 | 12160 | 24036 |
| 0.125 and 8 | 12244 | 17344 | 36468 |
| 0.3 | 9160 | 11192 | 27144 |

The test "Pairing" keeps a clearance from the walls of at least twice the bound of the folded point, because a clearance of $10^{-6}$ alone is far below the error of the fold in bands 5 and 6. With that clearance alone, a stress run of about 95,000 cases failed once, for `*(1000)32` at $1 - \lvert z \rvert = 10^{-5.17}$ and side 2, where the folded point lay $1.26 \times 10^{-5}$ from wall 1 and the two folds ended with the same parity. With the wider clearance, 20 runs of 5000 cases passed. It discards a case whose $P_i x$ lies beyond $1 - 10^{-6}$, and it takes the bound of the outer band, because $P_i x$ can lie farther out than $x$: with the band of $x$, the folds of `*(1000)32` in band 1 differed by $2.7 \times 10^{-11}$ against a bound of $2 \times 10^{-11}$.

The test "Rotation pairing" asserts nothing about the step counts, because the step count of a rotation domain does not change by 1 across a side. On the rotation prototype, with the clearance of that test and the bounds of the reflection domains, the step counts of $x$ and $P_i x$ differed by $-10$ to $+12$ for `732`, and by up to $+490$ for `(1000)32`. The parity never changed. Some folded points of `(1000)32` lay farther apart than the bound of `*(1000)32`, which is why the test takes the bound of Section 6.8. For a spherical rotation domain it takes the pairing bound of Section 6.8, because the bound of the folded point does not hold there, as that section shows.

The test "Border subset" runs the Euclidean domains at 193 by 193 pixels for the reason of the test "Euclidean border oracle". At 192 by 192 pixels the rotation prototype found 180 border pixels of `632` and 155 of `442` that were not border pixels of the reflection domain, and at the sizes of the test it found none in any domain that it rendered. At $n = 1000$ on the sphere, every pixel within ball radius 0.9 is a border pixel of both images, so the test asserts at most as many border pixels, and not fewer. Section 6.8 records the counts of border pixels of each domain.

The test "Rim" renders only some rows, because at 512, 768, 1024, 1536, and 2048 pixels with one sample no sample lies beyond $1 - 10^{-6}$, the closest being $3.8 \times 10^{-6}$ at 2048. At 4096 pixels, 16 samples do, in rows 163, 588, 800, 1680, 2415, 3295, 3507, and 3932.

The symmetric view maps a point $x$ of the screen to $Q x$ before the fold, and the tangents to $Q T_1$ and $Q T_2$. $Q$ is the isometry that maps `hyperbolic::ORIGIN` to vertex 2 of the domain and the positive first axis to the geodesic ray from vertex 2 through vertex 0, built from `hyperbolic::translation`, `hyperbolic::inverse`, and `hyperbolic::rotation`. The wall of side 0 then lies on the first axis. The test uses `Fill::One`, because a mirror of the group swaps the two parity colours (Section 11). It uses an even image size, so that no sample lies on the first axis. The wall of side 0 after $Q$ is not exactly the first axis in `f64`, so the equality of the two halves is not a property of IEEE 754 arithmetic, and the test asserts the golden allowance rather than no differing pixel, because a bound of 0 pixels fails on the first platform that moves one sample across a border (Section 8.4). Each of the six images matched its mirror with no differing pixel.

The test "Periodicity" renders each case of this table at 512 by 512 pixels with a 4 by 4 grid and `Fill::Parity`. For each word, it multiplies the pairings in `f64`, asserts that the product is a translation along one axis of the plane, with the difference of its linear part from the identity, its off-axis component, and the difference of its last row from $(0, 0, 1)$ each within the translation tolerance of $6 \times 10^{-15}$. It asserts that the size of the translation divided by the plane scale rounds to the shift of the table, which gives the size of the shift and not its sign. It then compares pixel $(i, j)$ with the pixel shifted by the signed shift along that axis, for every pair inside the image:

| Domain | View | Word along the first axis | Word along the second axis | Shift in pixels | Assertion |
| --- | --- | --- | --- | --- | --- |
| `*2222`, `aspect` 1 | Canonical | $P_1 P_3$ | $P_0 P_2$ | 128 and 128 | Equal byte for byte |
| `*2222`, `aspect` 4 | Canonical | $P_1 P_3$ | $P_0 P_2$ | 256 and 64 | Equal byte for byte |
| `*442` | Canonical | $P_2 P_1 P_0 P_1$ | $P_0 P_1 P_2 P_1$ | 128 and 128 | Within the golden allowance |
| `*632` | Canonical | $P_2 P_1 P_0 P_1 P_0 P_1$ | None | 128 | Within the golden allowance |
| `*333` | Canonical | $P_2 P_0 P_1 P_2 P_0 P_1$ | None | 192 | Within the golden allowance |
| `*2222`, `aspect` 1.618 | Periodic, with the plane scale $2w/128$ | $P_1 P_3$ | None | 128 | Within the golden allowance |

The exact rows rest on the reason of Section 6.6, which holds only for those two values of `aspect`. The other rows involve sines, cosines, or square roots that round, so IEEE 754 arithmetic does not guarantee equality (TEST-13 of the coding standard), and they assert the allowance of Section 8.4: at most 16 differing pixels, each within the limit of RND-20 in each channel. Every row measured no differing pixel. The signed shifts were $+128$ columns and $+128$ rows for `aspect` 1, $-128$ columns and $+128$ rows for `*442`, $-128$ columns for `*632`, and $-192$ columns for `*333`. The translation tolerance has a margin of 9.0 over the largest measured difference, $6.66 \times 10^{-16}$ in the off-axis component of `*632`. The largest difference of the linear part was $4.44 \times 10^{-16}$, for `*442` and `*632`, and that of the last row was 0. In the canonical view the period of `*2222` with `aspect` 1.618 is about 162.8 pixels, which is not whole, so that row uses the periodic view, a view available only to the unit tests that differs from the canonical view only in its plane scale. The test asserts numerically that each word is a translation, so it relies on no statement about the group.

The property tests commit their regression files (TEST-9 of the coding standard).

### 8.2. Deliberately Wrong Inputs

| Input | Expected |
| --- | --- |
| A point with a NaN, positive infinite, or negative infinite value in each coordinate in turn | `converged()` false, zero steps |
| The zero vector, $(0.5, 0.25, 0)$, $(0.5, 0.25, -0)$, the spacelike vector $(1, 0, 0)$, and `-hyperbolic::ORIGIN` on the other sheet of the hyperboloid, each with a last coordinate that is not positive | `converged()` false, zero steps |
| The spacelike vector $(3, -2, 0.5)$, with a positive last coordinate | No panic. Unconverged, with at most the default cap of steps |
| A cap of 0 with a point outside the domain | Unconverged, zero steps |
| A cap one below the steps that a point needs | Unconverged, `steps()` equal to the cap |
| `(f64::MAX, 0, f64::MAX)` | Unconverged, because the point becomes infinite during the fold (Section 11) |
| A vertex of the domain | Converged with zero steps, because $\tau > 0$ (Section 6.2) |
| For `*532`, the zero vector, and a NaN, positive infinite, or negative infinite value in each coordinate in turn | `converged()` false, zero steps |
| For `*532`, $(0, 0, -1)$, the antipode of the centre | Converged, with at most the default cap of steps |
| For `*532`, a cap of 0 with a point outside the domain, and a cap one below the steps that a point needs | Unconverged, with zero steps and with `steps()` equal to the cap |
| A buffer one byte short of a row, an empty buffer, and a band past the last row | `InvalidBuffer` with the text of Section 7.2 |
| `encode_png` with a size of 0 or a buffer of the wrong length | `InvalidBuffer` |
| A size of 0 by 512, a grid of 17, a width of 16385, a NaN border width, a cap of 100001, a NaN rotation angle, and a zero light direction | The kinds and texts of Section 7.2 |

### 8.3. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture:

- Structure. The module `reference`, in `tests/render/reference.rs`, which the integration-test binary and the unit tests load (Section 8.1), holds points, vectors, and matrices as arrays of high-precision numbers and writes each formula componentwise from the research documents: the map from the disk to the hyperboloid, the side test, the reflection, the fold of Section 5.3 with the point size of each geometry, the distance form of Section 5.4, the lift onto the ball of Section 5.9, the shading of Section 5.10, and the sRGB transfer function. The shading of the reference holds the specular weight $k_s$ as a value of its own, apart from the ambient weight $k_a$, although both are 0.25, so that it follows the formula of Section 5.10 term by term. It calls no function of `escher-render`, `escher-domain`, `escher-geometry`, or `glam`.
- Inputs. The reference takes the exact `f64` values of the inputs, and of the pairings and covectors of the domain, which it reads through the public accessors of `Side`.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 12.2 of the domain design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there.
- Self-check. The test "Reference self-check" checks the reference library on known values, as Section 12.2 of the domain design does: $\pi$ and $\operatorname{arsinh} 1$ against literals with 80 digits after the decimal point, $\cos(\pi/3) = 1/2$, $\sqrt{2}^2 = 2$, $\sinh(\operatorname{arsinh} 1) = 1$, $\operatorname{arcosh}(\cosh 1) = 1$ with $\cosh 1$ computed from a literal of $e$, and $2^{1/2} = \sqrt{2}$, each to $10^{-70}$. It does not close `UNV-25`.

RND-19. The reference module MUST share no code with `escher-render`, `escher-domain`, or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture, TEST-13 of the coding standard).

### 8.4. Golden Images

The golden images are `*732`, `*542`, and `*433` in the canonical view at 192 by 192 pixels, with a 4 by 4 grid, a border width of 1 pixel, `Fill::Parity` with the sRGB colours $(245, 222, 179)$ for an even parity and $(70, 130, 180)$ for an odd parity, and the default border colour, background, and cap. They are committed as `7-3-2.png`, `5-4-2.png`, and `4-3-3.png` under `crates/escher-render/tests/render/golden/`, as Section 7.2 of the architecture and TEST-14 of the coding standard state. The Euclidean golden images are `*632`, `*442`, `*333`, and `*2222` with `aspect` 1.618, with the same options in the canonical view of Section 5.8, committed in the same directory as `6-3-2.png`, `4-4-2.png`, `3-3-3.png`, and `2-2-2-2.png`. The spherical golden images are `*532`, `*432`, `*332`, `*722`, and `*55` with the default rotation, light, and shading, `*532` with the rotation 20, 35, 0 degrees, and `*532` with `Shading::Flat`, with the same options in the view of Section 5.9, committed in the same directory as `5-3-2.png`, `4-3-2.png`, `3-3-2.png`, `7-2-2.png`, `5-5.png`, `5-3-2-rotated.png`, and `5-3-2-flat.png`. The golden configuration of each image holds its rotation and its shading. The rotation golden images are `732`, `442`, `2222` with `aspect` 1.618, `532`, `722`, and `55`, with the same options in the canonical view of their geometry, the spherical ones with the default rotation, light, and shading, committed in the same directory as `rot-7-3-2.png`, `rot-4-4-2.png`, `rot-2-2-2-2.png`, `rot-5-3-2.png`, `rot-7-2-2.png`, and `rot-5-5.png`. Their configuration keeps `Fill::Parity`, so they show the first fill colour only (RND-33), and no border lies on a doubled wall (RND-34).

RND-20. The golden-image test MUST decode the committed file with `png`, convert both images to linear light through a table of the 256 byte values that the reference transfer function of Section 8.3 gives, and pass only when at most the allowed number of pixels differ and each differing pixel differs in each channel by at most one sample weight plus the linear-light size of one 8-bit step at the larger of the two stored values of that channel, because samples are averaged in linear light and stored as 8-bit sRGB, so a sample that changes colour moves the stored value by an amount that depends on the colours, a limit in stored units would be too loose for dark values and too tight for bright ones, and an integration test cannot read the linear value of a `Colour`.

The allowed number of pixels is 16 for each image, a choice of this design. The evidence for it is:

- The test "Perturbation" of Section 8.1 applies `f64::next_up` to every component of every normal copied into the renderer and rebuilds the covector and the reflection of each side from the moved normal. That perturbation is the size of a difference in the platform mathematics library, which the plan names as a risk. It changed no pixel of the three hyperbolic images, and the test asserts the allowance. For the four Euclidean images it moves every covector instead, because the normal of a Euclidean wall does not carry its offset, and rebuilds the reflection of each side from the moved covector. It changed no pixel of the four Euclidean images either. For the seven spherical images it also moves the normalized light direction and the entries of the viewing matrix, and it changed no pixel of them. For the six rotation images it moves every covector and every entry of every pairing, with the light and the viewing matrix for a spherical one, and it changed no pixel of them.
- The committed images match a fresh rendering with no differing pixel.
- On the prototype, a rotation of $10^{-6}$ changed 4 to 9 pixels by at most one sample each (Section 11). 16 pixels admit a difference between platforms of that size.
- Every defect tried that changes an image changes at least 828 pixels of it, so 16 pixels hide none of them.

An allowance of 0 pixels, which 8 times the measured count gives, would fail on the first platform whose mathematics library moves one sample across a border. The count bound of RND-16 does not apply here, because the allowance covers a difference between platforms that no measurement on one platform can show.

Each defect of this table was applied alone to the crate, the tests of the crate were run, and the crate was restored. A cell gives the number of pixels of the golden image that differ from the committed file:

| Defect | `*732` | `*542` | `*433` | Tests that failed |
| --- | --- | --- | --- | --- |
| No parity flip | 10,114 | 10,994 | 11,430 | "Golden images" and "Pairing" |
| Tangent not carried through side 2 | 12,953 | 12,054 | 11,932 | "Golden images", "Border oracle", and "Border forms" |
| Sample offset 1.5 instead of 0.5 | 12,754 | 11,205 | 9,964 | "Golden images", "Border oracle", "Sample position", and "Statistics" |
| Offset 0.056 in the transfer function | 3,341 | 2,961 | 2,503 | "Golden images", "Colour", and "Colour constants" |
| Exponent 2.2 in the transfer function | 12,718 | 10,988 | 9,648 | "Golden images" and "Colour" |

The same procedure gave these counts for the Euclidean golden images:

| Defect | `*632` | `*442` | `*333` | `*2222`, `aspect` 1.618 | Tests that failed |
| --- | --- | --- | --- | --- | --- |
| No parity flip | 18,156 | 19,072 | 18,544 | 18,228 | "Golden images", "Rectangle grid", "Euclidean pairing", and "Pairing" |
| Tangent not carried through side 2 | 0 | 0 | 0 | 0 | "Golden images", "Border oracle", and "Border forms", for the hyperbolic images only |
| Plane scale $S$ of 7 in place of 8 | 29,312 | 26,250 | 26,800 | 19,816 | "Golden images", "Plane view", "Euclidean border forms", "Euclidean border oracle", "Rectangle grid", "Periodicity", and "Half-pixel tie" |
| The two axes of the plane position swapped | 28,914 | 0 | 26,714 | 22,952 | "Golden images", "Plane view", "Euclidean border oracle", "Rectangle grid", and "Periodicity" |

The defect "Tangent not carried" changes no pixel of a Euclidean image, because the gradient size of a Euclidean domain does not depend on it, so no Euclidean test catches it. The hyperbolic tests of the first table catch it.

The same procedure gave these counts for the spherical golden images. The columns `rotated` and `flat` are `*532` with the rotation 20, 35, 0 degrees and `*532` with `Shading::Flat`:

| Defect | `*532` | `*432` | `*332` | `*722` | `*55` | `rotated` | `flat` | Tests that failed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| No parity flip | 12,594 | 12,643 | 12,443 | 12,832 | 12,692 | 12,641 | 12,594 | "Golden images", "Rectangle grid", and "Rotation by a group element" |
| Screen tangents not turned by $V^{\mathsf{T}}$ | 0 | 0 | 0 | 0 | 0 | 4,139 | 0 | "Golden images" and "Rotation by a group element" |
| $V$ in place of $V^{\mathsf{T}}$ | 0 | 0 | 0 | 0 | 0 | 16,759 | 0 | "Golden images", "Perturbation", and "Ball tangents" |
| The opposite sign of the pitch | 0 | 0 | 0 | 0 | 0 | 18,435 | 0 | "Golden images", "Ball tangents", and "Viewing matrix" |
| The test of a positive last coordinate kept for a spherical domain | 0 | 0 | 0 | 0 | 0 | 2,981 | 0 | "Golden images", the unit test of the point size in `fold.rs`, "Vertices", "Centre images", "Wrong inputs", "Spherical fold replay", "Inside" of "Spherical fold properties", and "Rotation by a group element" |
| The factor $(1 - c)$ dropped from the specular term | 1,757 | 1,735 | 1,774 | 1,761 | 1,776 | 1,748 | 0 | "Golden images", the reference test and the range test of "Shading", and the test of the shading function in "Brightest point" |
| Shading applied after the samples are averaged | 1,960 | 1,654 | 1,370 | 1,592 | 1,182 | 1,916 | 0 | "Golden images" only |

The defects of the view change only the rotated image, because the default rotation gives a viewing matrix with the entries of the identity (Section 5.9). The golden allowance does not hide shading after averaging at 192 by 192 pixels, because the defect changes between 1,182 and 1,960 pixels of each shaded image. On the hyperbolic and Euclidean golden images, the defect "No parity flip" gives the counts of the tables above.

These defects change no pixel of the golden images, and another test fails for each:

| Defect | Test that failed |
| --- | --- |
| Sides tested in reverse order | "Fold decisions" |
| $\tau = 0$ | "Vertices", "Euclidean fold replay", and the unit test of the point size in `fold.rs` |
| The comparison of RND-9 without its negation for NaN | "NaN border" |
| The obsolete decoding threshold 0.03928 | "Thresholds" |
| The obsolete encoding threshold 0.00304 | "Thresholds" |

The same procedure gave these counts for the rotation golden images:

| Defect | `rot-7-3-2` | `rot-4-4-2` | `rot-2-2-2-2` | `rot-5-3-2` | `rot-7-2-2` | `rot-5-5` | Tests that failed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Every step counted as a reversal, whatever the orientation of its side | 12,062 | 18,432 | 18,210 | 12,798 | 13,086 | 11,356 | "Golden images", "One-colour parity", "Rotation pairing", "Inside" and "Any input" of "Rotation fold properties", and the unit test of "Fold steps" that counts only reversing pairings as reversals |
| The screen tangents not carried through a step whose pairing preserves orientation | 12,631 | 0 | 0 | 3,053 | 1,254 | 828 | "Golden images" and "Border subset" |

The first defect changes no pixel of a golden image of a reflection domain. The second changes no pixel of a Euclidean rotation image, for the reason that the defect "Tangent not carried" changes no Euclidean reflection image. It changes 828 pixels of `rot-5-5`, the smallest count of any defect tried that changes an image, which is still far above the allowance of 16 pixels.

A defect of the public `fold` alone, which applies the pairing of the partner side in place of the pairing of the side, changes no result of a reflection domain. It failed the tests "Rotation fold replay", "Double oracle", "Centre images" of the rotation domains, "Inside" and "Idempotent" of "Rotation fold properties", "Rotation pairing", and "Spherical rotation pairing".

The golden-image test reads the environment variable `ESCHER_UPDATE_GOLDEN`. With exactly the value `1`, it writes each rendered image over its committed file and does not compare. With any other value, or unset, it compares. A committed file that cannot be read or decoded, or whose width, height, colour type, bit depth, or buffer length differs from the rendered image, fails that image with a message that names the recipe `update-golden`, and such a file never reaches the comparison or the encoder. Section 14.9 of the tooling design gives the procedure.

When an image differs beyond the allowance, the test writes `actual-<name>.png`, the rendered image, and `difference-<name>.png`, the absolute difference of each byte, under the directory that `CARGO_TARGET_TMPDIR` names, where `<name>` is the name of the golden file without `.png`. It checks every golden image before it fails, prints the number of differing pixels of each image that it compares, and reports every image that failed.

### 8.5. Tests Under Miri

`DEC-5` and TEST-11 of the coding standard apply to the tests of this crate in this way:

| Tests | Under Miri | Attribute or configuration |
| --- | --- | --- |
| "Fold replay", "Fold decisions", "Cap", "Tangents", "Border forms", "Colour", "Thresholds", "Reference self-check" in both binaries, "Pairing", "Euclidean pairing", "Euclidean fold replay", "Euclidean border forms", "Spherical fold replay", "Ball tangents", "Shading" against the reference, "Spherical border forms", "Rotation fold replay", "Rotation pairing", "Spherical rotation pairing", and "Double oracle" | Ignored | `#[cfg_attr(miri, ignore = "the high-precision arithmetic runs loops too long for the interpreter")]` |
| "Rim", "Border oracle", "Bands", "Symmetric view", "Perturbation", "Euclidean border oracle", "Rectangle grid", "Periodicity", "Half-pixel tie", "Rotation by a group element", the image test of "Brightest point", "Spherical border oracle", the three tests of "Spherical statistics", the four tests of the rotation images of "Cap", "Border subset", and "One-colour parity" | Ignored | `#[cfg_attr(miri, ignore = "rendering whole images runs loops too long for the interpreter")]` |
| The two tests of "Centre images" | Ignored | `#[cfg_attr(miri, ignore = "the enumeration of the group runs loops too long for the interpreter")]` |
| The two tests of "Near vertex" | Ignored | `#[cfg_attr(miri, ignore = "folding thousands of points of a large group runs loops too long for the interpreter")]` |
| "Golden images" | Ignored | `#[cfg_attr(miri, ignore = "rendering whole images runs loops too long for the interpreter, and the test reads files")]` |
| "Inside", "Idempotent", "Scale", "Any input", "Euclidean fold properties", "Spherical fold properties", and "Rotation fold properties" | Run with 8 cases and no failure persistence | The helper of RND-21 |
| The two tests of "Vertices", "Wrong inputs", "Sample position", "Fold steps", "NaN border", "Colour constants", "Options", "Statistics", "Error text", "Auto traits", "PNG size", "Plane view", "Ball view", "Viewing matrix", the unit tests of "Shading", and the test of the shading function in "Brightest point" | Run unchanged | None |
| The doctests | Not run, because Miri runs the tests through nextest, which runs no doctest | None |

Each reason in an attribute is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

The exact assertions of "Ball view" and "Viewing matrix" at the angles 0 rely on `f64::sin_cos` of 0 giving exactly 0 and 1. If the interpreter perturbs that result, as it perturbs other results of the platform mathematics library, those assertions move to a test that carries an `ignore` attribute under Miri with that reason in plain words.

"Pairing" is a property test, but each case folds in the 256-bit reference, and 8 cases took 287 seconds under Miri, so it carries the attribute instead of running with 8 cases. "Scale" builds its powers of two by exact doubling and halving, because Miri perturbs the result of `powi`, and the exact comparison would then fail. On an unloaded machine, the Euclidean "Scale" took 87.8 seconds under Miri, and "Vertices" took 46 seconds. In one run of `just check`, the spherical "Scale" took 137 seconds under Miri, the longest of the tests that run there, against 128 seconds for the Euclidean "Scale" in the same run. The range test of "Shading" took 104 seconds and the test of the shading function in "Brightest point" 108 seconds. In another run of `just check`, the rotation test of "Vertices" took 82.5 seconds under Miri, and "Scale" of "Rotation fold properties" 150 seconds. Their grids are those of Section 8.1, and a smaller grid would shorten them.

No test asserts a bit-exact result of a transcendental function. The exact comparisons of Section 8.1 involve no transcendental function, and each states why IEEE 754 arithmetic makes it exact.

RND-21. The property tests MUST take their configuration from one helper in `common`, with the numbers of cases and the failure persistence that Section 4 of `docs/design/tooling.md` states, because the file persistence of `proptest` asks for the current directory, which the isolation of Miri refuses, and a full set of cases takes too long in the interpreter.

## 9. Dependencies

RND-22. The crate MUST declare only the dependencies of this section, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml`, because each further dependency adds build time, licence obligations, and supply-chain risk, and a version declared twice can diverge (DEP-2 of the coding standard).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-domain` | Normal, with a version on the path dependency (TOOL-25 of `docs/design/tooling.md`) | None | The input type `FundamentalDomain` and its sides |
| `escher-geometry` | Normal, with a version on the path dependency (TOOL-25) | None | The disk map, and `spherical::boost` and `spherical::rotation` for the viewing matrix |
| `glam` | Normal, public | The workspace entry | `DVec3` in the signatures of `fold` and `light` (`DEC-4`) |
| `thiserror` | Normal | Default | Derives the `Display` of the private detail of the error (Section 7.1) |
| `png` | Normal | The workspace entry, version 0.18.1, which enables no feature, because `png` has no default feature | PNG encoding with the sRGB chunk (Section 5.6), and decoding in the tests |
| `escher-signature` | Normal, with a version on the path dependency (TOOL-25) | None | The type `Geometry`, on which `Renderer::new` chooses the view (RND-23), and the signatures from which the tests build their domains |
| `proptest` | Dev | The workspace entry | The property tests |
| `astro-float-num` | Dev | The workspace entry | The reference arithmetic (Section 8.3) |

`png` is the one new dependency of the workspace. `Cargo.lock` resolves it with these crates:

- `png` 0.18.1, which depends on `bitflags`, `crc32fast`, `fdeflate`, `flate2`, and `miniz_oxide`.
- `flate2` 1.1.9, which depends on `crc32fast` and `miniz_oxide`.
- `miniz_oxide` 0.8.9, which depends on `adler2` and `simd-adler32`.
- `fdeflate` 0.3.7, which depends on `simd-adler32`.
- `crc32fast` 1.5.5, which depends on `cfg-if`.
- `simd-adler32` 0.3.10, which has no dependency.

The newest resolution of `png` brings two versions of `miniz_oxide`, which fails the ban on duplicates of `cargo deny`. This design holds `flate2` at version 1.1.9 in the lock file, as Section 5.3 of the plan states, so that one version of `miniz_oxide` resolves. With that pin, `cargo tree -d` finds no duplicate and `check-deny` passes with no change to `deny.toml`. A plain `cargo update` resolves `flate2` to a newer version and brings the duplicate back. Section 8 of the tooling design gives the command that restores the pin. Section 12.3 keeps the alternative, a `skip` entry in `deny.toml`, as a point for the owner.

`palette` and `csscolorparser` are not dependencies of this crate, because `DEC-29` adds no colour crate in the first image milestone. The crate implements the sRGB transfer function itself (RND-11), and colour text belongs to the command line (RND-5). Section 3.1 of the architecture lists `escher-geometry`, `escher-domain`, `escher-signature`, and `escher-group` from `M8` as internal dependencies of the crate. `escher-signature` is a normal dependency, not a dev-dependency, because the `match` of RND-23 names `Geometry`, a choice of this design. The crate does not declare `escher-group` until a function uses it, as Section 3.3 of the architecture states, because an unused dependency fails `check-unused-deps`.

`glam` types appear in the public signatures of this crate, so DEP-3 of the coding standard makes `glam` a public dependency, with the obligations that Section 3.5 of the architecture states. No type of `png` appears in a public signature (Section 3.5 of the architecture).

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

## 10. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, or a statement of this design that no source establishes, and what the code does about it:

| Label | What the code relies on | What the code does |
| --- | --- | --- |
| `UNV-1` | The fold terminates for a compact hyperbolic reflection triangle, with a step count that grows toward the limit circle, for a Euclidean reflection triangle or rectangle, with a step count that grows with the distance from the centre, and for a spherical reflection triangle or lune, for the samples of the images that the tests render and the images of the centre. Near a vertex of very high order on the sphere the fold can fail to stop, and can take more steps than the longest word of the group (Section 6.7). The fold of `*(1000)(1000)(1000)` and of `*(1000)(1000)(999)` can fail to stop within $10^{-8}$ of some vertices of their rotation domains (Section 6.8) | Stops at the fold cap and reports the sample as unconverged. The default cap comes from the prototype (Section 11), and the tests "Cap", "Euclidean fold replay", "Spherical fold replay", "Rotation fold replay", "Centre images", and "Spherical statistics" measure the step counts and that every fold of their sets converges within the cap (Sections 6.4, 6.6, 6.7, and 6.8). The test "Near vertex" allows unconverged results in the reflection domains |
| `UNV-2` | The fold terminates for the pairings of a rotation domain, which are rotations and translations, for the samples of the images that the tests render and the images of the centre. On the rotation prototype, with plain `f64` products as pairings, one fold of `(1000)32` in band 6 did not converge. With the compensated products, every fold of the rotation fold sets converges. Near the vertices other than vertex 0 of a hyperbolic rotation domain whose corner orders are all near 1000, the fold of the rotation domain can fail to stop at a larger distance from a vertex than the fold of the same points in its reflection domain (Section 6.8) | Folds by the one loop of RND-31, stops at the fold cap, and reports the sample as unconverged. The domain design requires accurately rounded products as pairings. RND-35 excludes the points near those vertices from the range of accuracy. The tests "Rotation fold replay", "Cap", "Double oracle", "Centre images", "Spherical rotation pairing", and "Near vertex" measure the step counts, the accuracy, and the unconverged points, and "Near vertex" asserts that no fold of a rotation domain is unconverged beyond the distances that Section 8.1 states |
| `UNV-30`, `UNV-31` | The lune double is a fundamental domain of the cyclic group, and the double of a convex reflection polygon is convex with the stated vertex cycles, through the domain that `escher-domain` builds | Renders the walls and pairings that the domain gives, and tests the border estimate over the whole wall of each side, which is correct for a convex domain (Section 5.11). The domain design states which checks and tests cover them, and the tests "Double oracle" and "Border subset" check the tiling that results |
| `UNV-3` | The behaviour of a point on or near a wall | Uses the tolerance constant of Section 6.2. The tests "Vertices" and "Wrong inputs" cover the vertices of the domain, and the test "Pairing" keeps its points at a clearance from the walls that grows with the error of the fold |
| `UNV-4` | The growth of the fold count and the limits of precision near the limit circle | States accuracy only in the range of accuracy (RND-15). The tests "Fold replay", "Fold decisions", "Cap", and "Rim" measure each band |
| `UNV-14` | The unit-normal form $\sinh d = \lvert \langle x, u \rangle \rvert$, in the distance form of the tests only | Compares the gradient form with it in the test "Border forms". The code of the crate does not use it |
| `UNV-15` | The gradient form of the border estimate through a chain of folds in the linear model | Tests it against the distance form in the tests "Border forms" and "Euclidean border forms", against tile identity in the tests "Border oracle", "Euclidean border oracle", "Spherical border forms", and "Spherical border oracle", and against the grid of the rectangle in the test "Rectangle grid". On the ball it states its accuracy by rim band (Section 6.7). The comparison with the distance form tells the two forms apart only in bands 1 and 2, because in bands 3 to 6 every sample of its set lies within 0.13 pixels of a wall (Section 6.4), so no test compares the forms beyond band 2 |
| `UNV-18` | The parity colouring as the colouring by the two cosets of the orientation-preserving subgroup | Colours by the parity of the reversals. The test "Pairing" checks that a pairing of a reflection domain changes the parity, the test "Rotation pairing" that a pairing of a rotation domain keeps it, and the golden images fix the result |
| `UNV-25` | The correct rounding of the reference crate, in tests only | Uses 256 bits and the self-check of Section 8.3 |
| This design | The bounds of Sections 6.2, 6.4, 6.6, 6.7, and 6.8 and the allowance of Section 8.4 hold on platforms other than Linux x86-64, where they were measured | Sets each bound with a margin of at least 8, except the tolerance constant at the vertices of the rotation domains, whose margin is 3.27 (Section 6.2), and the allowance of 16 pixels for a difference between platforms. CI runs the tests on Linux x86-64 only |
| This design | The screen tangents of Section 5.2 are the partial derivatives of the map of Section 7.2 of the geometry design, times $h$ | The test "Tangents" compares them with central differences of the reference map |
| This design | For a Euclidean domain, the side value at a point with last coordinate 1 is the distance from the point to the wall, in the tests only | The Euclidean distance form of the test "Euclidean border forms" uses it. The code of the crate does not use it |
| This design | The mirrors of `*2222` are the lines of the grid of its rectangle, in the tests only | The oracle of the test "Rectangle grid" uses it. The test "Euclidean border oracle" checks the borders of the rectangle against tile identity without it |
| `UNV-26`, `UNV-28` | The Euclidean reflection as a matrix, and the vertex coordinates of the Euclidean triangle, through the domain that `escher-domain` builds | Renders the walls and pairings that the domain gives. The domain design states which checks and tests cover them, and the tests "Periodicity" and "Euclidean border oracle" check the tiling that results |
| `UNV-26`, `UNV-29` | The angle of a lune measured from its normals, and the lune as a strict fundamental domain of the group of its two reflections, through the domain that `escher-domain` builds | Renders the walls and pairings that the domain gives. The domain design states which checks and tests cover them, and the tests "Centre images", "Rotation by a group element", and "Spherical border oracle" check the tiling that results |
| `UNV-16` | The lift onto the ball and the Phong reflection model, whose primary paper is unread | Lifts as Section 5.9 states and shades as Section 5.10 states. The tests "Ball tangents", "Brightest point", and "Shading" check them against the reference |
| This design | The screen tangents of Section 5.9 are the partial derivatives of the lift onto the ball, times $h$, turned by $V^{\mathsf{T}}$ | The test "Ball tangents" compares them with central differences of the reference lift |
| Rendering reference | The constants of the sRGB transfer function, which rest on secondary sources until the standard is read (items 22 and 25 of Section 13.1 of the rendering reference) | Takes them from Section 7.5 of the rendering reference only (RND-11). The tests "Colour" and "Thresholds" compare the crate with the reference transfer function |
| Rendering reference | Rounding to the nearest integer, after clamping to the range 0 to 1, when encoding an 8-bit sample (item 23 of Section 13.1 of the rendering reference) | Rounds as Section 5.6 states. The test "Colour" checks the round trip of every byte |

Averaging in linear light is a choice of the plan that no source supports (item 24 of Section 13.1 of the rendering reference). It states no fact, so it is a choice rather than an unverified statement. The same holds for the point size of the sphere, the viewing matrix, the shading constants, and the form of the specular term.

## 11. Evidence Behind the Choices

This section is non-normative. It records figures measured on the prototype, which motivated the choices of this design. Each supports only the choice that it names, and none is a property of the crate. Where the crate measures the same quantity, Sections 6.2, 6.4, 6.6, 6.7, 6.8, and 8 give its figures, which are the fact, and the figures of this section remain only as the evidence behind the choice.

The prototype implemented Sections 5.1 to 5.5 in Rust on Linux x86-64 with a 256-bit reference. Unless a row states otherwise, it used a margin of $1/32$, $\tau = 0$, a 512 by 512 view with a 4 by 4 sample grid where it rendered an image, and the six triples $(7, 3, 2)$, $(5, 4, 2)$, $(4, 3, 3)$, $(12, 3, 2)$, $(1000, 3, 2)$, and $(1000, 1000, 1000)$:

| Choice | Section | Input set | Figures of the prototype |
| --- | --- | --- | --- |
| No renormalization in the fold | 5.3, 6.1 | 2048 angles in each radius band, with $K$ of 1, 4, 16, and 64 | Renormalizing every $K$ steps changed the side sequence of folds against no renormalization. At $1 - 10^{-6}$: 7, 6, 3, and 0 folds of `*(1000)32` for $K$ of 1, 4, 16, and 64. At $1 - 10^{-7}$: 131, 46, 1, and 0 folds of `*732`, and 378, 284, 118, and 21 of `*(1000)32`. From $1 - 10^{-8}$ outward, hundreds per band, with the parity changed in most. `hyperbolic::normalize_point` returned an error up to 5.5 million times per band, and with $K$ at most 16 some folds returned infinite coordinates |
| Decisions on the ray | 6.1 | The folds of the row above | Scaling the point by $2^k$ for $k$ from $-40$ to 40 changed no side sequence |
| A converged fold has a finite point | 5.3, 6.3 | The input $(f_{\max}, 0, f_{\max})$, with $f_{\max}$ the largest finite `f64` and $\tau = 10^{-12}$, for `*732` and `*(1000)32`. Also 12 million fuzzed inputs | Without the check of step 2 after each step, the input returned converged after 1 step with an infinite last coordinate. NaN never returned converged in the fuzzed inputs |
| A positive $\tau$, not a bound on error | 6.2 | The vertices of the domains of the six triples, with $\tau$ of 0, $10^{-15}$, and $10^{-12}$, and the side tests of folds at 512 angles in each radius band | With $\tau = 0$, the vertices of the domains took 1 or 2 steps, and vertex 2 of $(1000, 1000, 1000)$ took 1935. With $\tau$ of $10^{-15}$ or $10^{-12}$, every vertex of a domain took zero steps. The crate uses $4 \times 10^{-15}$, from the measurement of Section 6.2. The ratio of the error of the side value to $\sigma_i x_3$ reached $1.5 \times 10^{-3}$ for `*732` and $5.0 \times 10^{-3}$ for `*(1000)32` at $1 - 10^{-6}$, and grew about 100 times per decade of $1 - \lvert z \rvert$ |
| A default cap of 1500 | 4.1 | A sweep of the radius bands, and the samples of the 512 by 512 image of each triple | The largest step count of the sweep was 1214, for `*(1000)32` beyond $1 - 10^{-7}$. A corner of order 1000 needed 1000 steps. The smallest cap at which every sample of the image converged was 89 for `*732`, 1000 for `*(1000)32`, and 998 for $(1000, 1000, 1000)$. At a cap of 500, 7956 of 2,895,324 samples inside the disk of `*(1000)32` did not converge, and no pixel differed from a cap of $10^6$ at border widths of 0.1, 0.3, 0.5, and 1 pixel. At a border width of 0, 541 pixels differed |
| The range of accuracy ends at $1 - 10^{-6}$ | 6.4 | 512 angles in each radius band from $1 - 10^{-1}$ to $1 - 10^{-12}$ | The largest distance of the folded point from the 256-bit replay grew from between $1.1 \times 10^{-13}$ and $1.3 \times 10^{-12}$ at $1 - 10^{-1}$ to $1.6 \times 10^{-3}$ for `*732`, $5.0 \times 10^{-3}$ for `*(1000)32`, and $1.3 \times 10^{-2}$ for $(1000, 1000, 1000)$ at $1 - 10^{-6}$, and was 0.06 to 0.7 at $1 - 10^{-7}$. At $1 - 10^{-6}$ the crate measures lower distances for these three triples (Section 6.4). The first side that differed from a 256-bit fold appeared in up to 8 of 512 folds per triple at $1 - 10^{-6}$ and in 24 to 265 at $1 - 10^{-7}$. `from_disk` returned a point for every position up to $1 - 10^{-7}$, and a vector that is not a point for 141 to 452 of 512 positions per band from $1 - 10^{-8}$ to $1 - 10^{-12}$. Such samples did not converge at a cap of $10^6$ |
| The gradient form of the border | 5.4 | 600 samples in each radius band, at half widths of 0.5 and 1.5 pixels | Both forms agreed with the exact screen distance to the walls of the tile within 0.3 percent beyond 100 pixels from the rim of `*732`, within 4 percent at 10 pixels, and within 5 percent within 2 pixels, and within 10 percent within 5 pixels for `*(1000)32`. The two forms differed by less than 2 percent on this set, while the crate measures up to 8.7 percent at $(1000, 1000, 1000)$ (Section 6.4). At most 10 of 600 samples per band were misclassified. The gradient form with the tangent formulas of Section 5.2 matched the exact distance to $10^{-3}$ near the centre |
| The pairing property | 8.1 | $x$ and its reflection in a random side, with a band of $10^{-6}$ around the walls excluded, 20000 cases for each of `*732` and $(1000, 1000, 1000)$ | Step counts differed by $+1$ in 10947 and by $-1$ in 9053 cases for `*732`, and in 14272 and 5728 for $(1000, 1000, 1000)$, with no other difference and the parity changed in every case |
| The parity colouring is consistent | 5.5 | Tiles centred within disk radius 0.999, about 41,000 to 200,000 points per triple | No inconsistency within a tile, and no two adjacent tiles with the same parity |
| One fill in the symmetric view | 8.1 | The images of `*732`, `*542`, and `*433` at 192 by 192 | With `Fill::Parity`, 20200 to 22712 of 36864 pixels differed from the mirror image, and every single-fill pixel pair matched once the two fill colours were swapped. With an odd image size and an odd grid, such as 191 and 3, a row of samples lay on the axis |
| The golden-image allowance | 8.4 | The images of `*732`, `*542`, and `*433` at 192 by 192, with a 4 by 4 grid and a 1-pixel border | Adding one unit in the last place to every component of every normal changed no pixel. A rotation of $10^{-6}$ changed 4, 9, and 9 pixels, by up to one sample, and a rotation of $10^{-3}$ changed more than 5000 |
| `png` with `flate2` at 1.1.9 | 9 | The dependencies of `png`, `rayon`, and a reduced `csscolorparser` in a project outside the repository | The newest resolution failed the duplicate check with `miniz_oxide` 0.8.9 and 0.9.1. With `flate2` at 1.1.9, the checks of bans and licences passed, and encoding and decoding a 4 by 4 image passed Miri |

The Euclidean prototype changed only the private view of a copy of the crate, which gained the plane view of Section 5.8 with $S = 8$. The fold, the border estimate, the colour of a sample, and the encoding were the code of the crate, unchanged. It built the Euclidean domains with the candidate corners and the rectangle that the domain design specifies, as private test code, and ran in release mode on Linux x86-64. Unless a row states otherwise, it used a 512 by 512 view with a 4 by 4 grid and the eight domains `*632`, `*442`, `*333`, and `*2222` with `aspect` 1, 1.618, 0.125, 8, and 4:

| Choice | Section | Input set | Figures of the Euclidean prototype |
| --- | --- | --- | --- |
| The fold and the border estimate unchanged | 5.8 | The eight domains | No outside and no unconverged sample. The largest fold was 18 steps for `*632`, 13 for `*442` and `*333`, 12 for `*2222` with `aspect` 0.125 and 8, 10 with 4, and 8 with 1 and 1.618, and the mean was 9.22 steps for `*632`. The last coordinate of every folded point stayed exactly 1, and that of both tangents exactly 0, over the 4,194,304 samples of each domain. In the images, colours alternate across every edge and the borders are thin and even |
| The span $S = 8$ with the units of `DEC-17` | 5.8 | The eight domains | Side 0 of each triangle had length 1, and the corner angles were 30, 60, and 90 degrees for `*632`, 45, 45, and 90 for `*442`, and 60 for each corner of `*333`. The areas from the vertices were 0.288675135, 0.5, and 0.433012702 for the three triangles and 1 for every `aspect`. The window of 8 by 8 units held about 221.7, 128.0, 147.8, and 64.0 tiles of `*632`, `*442`, `*333`, and `*2222`. At the ends of the range of `aspect`, a tile was 181.0 by 22.6 pixels |
| The view chosen from the geometry (RND-23) | 5.8 | `*632` and `*2222` with `aspect` 1 drawn through the unchanged disk view | No error. For `*632`, 1,298,980 samples were outside the disk and none was unconverged, and the image was a disk with a plausible but wrong curved pattern |
| $\tau$ kept for the Euclidean domains | 6.2 | The vertices of the eight domains and of `*2222` at 64 values of `aspect` evenly spaced in its logarithm | No vertex took a step. The largest ratio was $4.16 \times 10^{-17}$, for `*632`, 12 times under $\tau / 8$. The crate measures a larger ratio, $6.235 \times 10^{-17}$ for `*2222` (Section 6.2) |
| The range of accuracy of 65, and the cap of 1500 | 6.6 | 512 angles at the distances 1, 4, 16, and 64, and 64 angles for the distance at which the cap is first reached | At distance 64, the mean and the largest step count were 192.8 and 197 for `*632`, 139.1 and 145 for `*442`, 141.2 and 149 for `*333`, 81.5 and 91 for `*2222` with `aspect` 1, 83.8 and 96 with 1.618, and 129.6 and 183 with 0.125 and 8. No fold was unconverged up to distance 256. The cap was first reached at distance 490.7 for `*632`, 526.4 for `*2222` with `aspect` 0.125 and 8, 649.6 for `*333`, 671.1 for `*442`, 728.0 for `*2222` with 4, 1003.2 with 1.618, and 1061.6 with 1. The corner sample of an image of 8192 by 512 pixels lies at distance 64.12. An image of 16384 by 1 pixels reached distance 65532, with 16258 of 16384 samples unconverged |
| Accuracy of the fold | 6.6 | 64 points at each distance and domain | No fold decision differed from a 256-bit fold. The largest distance of the folded point from the 256-bit replay was $1.5 \times 10^{-16}$ at distance 1, about $10^{-15}$ at 4, about $10^{-14}$ at 16, and $2.2 \times 10^{-13}$ at 64, and exactly 0 for `*2222` with `aspect` 1 and 4. Scaling a point by $2^{-40}$, $2^{-3}$, $2^{5}$, or $2^{40}$ changed no decision. $(10^{300}, 0, 1)$ stopped at the cap with finite coordinates, and $(f_{\max}, 0, 1)$ became non-finite after 5 steps for `*632` |
| Periodicity in the canonical view | 8.1 | The rows of the periodicity table | No differing pixel in any row. In the canonical view, `*2222` with `aspect` 1.618 shifted by 163 pixels gave 4064 differing pixels, with a largest difference of 131 in one byte |
| The words of the periodicity table | 8.1 | Words of 2 to 6 pairings | $P_2 P_1 P_0 P_1$ of `*442` was the translation $(-2, 5.6 \times 10^{-17})$, with a linear part within $4.4 \times 10^{-16}$ of the identity. $P_1 P_3$ of `*2222` was exactly the translation $(2w, 0)$. For `*632`, no word of 2 to 5 pairings was a translation along an axis, and $P_2 P_1 P_0 P_1 P_0 P_1$ was $(-2, 0)$. The shortest translation of `*333` along the first axis was 3, by $P_2 P_0 P_1 P_2 P_0 P_1$ |
| The rectangle grid oracle | 8.1 | `*2222` with `aspect` 1, 4, 1.618, 0.125, 8, and 0.3, at border widths 1, 1.5, and 3, with one sample per pixel | No differing pixel of 262,144 in any of the 18 cases |
| The half-pixel tie | 5.8 | `*2222` with `aspect` 1 and 4, border width 1 | With one sample per pixel, no border pixel, and the grid oracle agreed. With 2, 3, or 4 samples per pixel, 16,128 pixels were not a pure fill colour |

The sphere prototype changed the fold of a copy of the crate to read the point size of Section 5.3, and implemented the rest of Sections 5.9 and 5.10 and the spherical domains as private test code, with the form of the specular term $\min(c (k_a + k_d \delta) + k_s \rho, \ 1)$. It ran in release mode on Linux x86-64. Unless a row states otherwise, it used a 512 by 512 view with one sample per pixel:

| Choice | Section | Input set | Figures of the sphere prototype |
| --- | --- | --- | --- |
| The point size, and no other change to the fold | 5.3 | The tests of the crate, its seven golden images, and the files that `escher render` writes for `*732`, `*542`, `*433`, `*632`, `*442`, `*333`, and `*2222` at the defaults | All 69 tests of the crate passed, and their filtered output before and after the change was identical. Each golden image had no differing pixel. `cmp` reported every file identical before and after the change |
| The fold on the whole sphere | 5.3, 6.3 | 18 spherical domains, each with a Fibonacci sphere of 400,000 points, the poles, the axes, $(0, 0, -1)$, points from $10^{-3}$ to $10^{-300}$ from the poles of a lune, points on and $10^{-15}$ off each wall, and the vertices with their antipodes | Every point converged with no side violated: 405,391 points for each triangle and 403,853 for each lune. $(0, 0, -1)$ converged in 15 steps for `*532`. The zero vector, NaN, and infinity gave unconverged results with zero steps. $(0, 0, -10^{-320})$ converged in 15 steps |
| $\tau$ kept for the sphere | 6.2 | The vertices of the spherical domains | The largest ratio was $9.3 \times 10^{-17}$, at `*522`, a margin of 43 of $\tau$. The poles of each lune gave exactly 0 |
| Termination near a vertex of high order | 6.7, 10 | 2000 points within $10^{-14}$ radians of the vertex of order 1000 of `*(1000)22` and of its antipode, and 508,000 points of `*(999)22` | 213 of 2000 points stayed unconverged at the cap of 1500, and 132 of 2000 at the antipode. The crate measures 135 of 1000 and 140 of 1000 on its own points (Section 6.7). At a cap of 100000, 4 points still did not converge. For `*(999)22`, a fold took up to 1389 steps where the longest word of the group is 1000, and 5889 points took more steps than the longest word. The lune `*(1000)(1000)` took at most 1000 steps. `*532` and every $n$ up to 100 never took more steps than the longest word |
| No bound from the group order in the code | 6.7 | The front hemisphere of the image of each signature, 180,960 samples | The mean and the largest step count were 1.42 and 4 for `*332`, 2.19 and 6 for `*432`, 3.70 and 10 for `*532`, 1.77 and 7 for `*722`, 215.6 and 999 for `*(1000)22`, 0.88 and 2 for `*55`, and 181.7 and 497 for `*(1000)(1000)`, every largest count below the group order and the cap, with no unconverged sample. Over the whole sphere, `*(1000)22` had a mean of 500.5 and a largest count of 1001. The crate measures a largest count of 1000 for `*(1000)22` and 499 for `*(1000)(1000)` in the image with a 4 by 4 grid (Section 6.7) |
| The images of the centre | 8.1 | `*332`, `*432`, `*532`, `*n22`, and `*nn` | The breadth-first enumeration found 24, 48, and 120 elements for `*332`, `*432`, and `*532`, and $2n$ for every `*nn`, equal to the group order. Each image of the centre folded back in exactly its depth, to within $1.8 \times 10^{-15}$ of the centre. The largest depths were 6, 9, and 15. The crate measures up to $2.629 \times 10^{-15}$ for $n$ up to 12, and up to $1.100 \times 10^{-13}$ at $n = 1000$ (Section 6.7) |
| The viewing matrix and its signs | 5.9 | The centroid of the tile of the domain at 256 by 256 pixels, predicted at (128, 128) | Yaw 30 degrees moved it to (186.3, 127.7), predicted (188, 128). Pitch 30 degrees moved it to (128.3, 69.1), predicted (128, 68). Yaw 30 and then roll 90 degrees moved it to (127.7, 69.7). The rotation 20, 35, 0 degrees moved it to (168.3, 64.3), predicted (169.0, 63.3). The determinant of $V$ was 1 within $1.1 \times 10^{-16}$. The yaw and the roll are right-handed about the second and third axes, and the pitch about the negative first axis |
| Rotation by a group element | 8.1 | `*532` with the default rotation and with 20, 35, 0 degrees, `*55`, and `*722`, with shading on and off | The image with $V g$ equalled the image with $V$ with no differing pixel, for $g$ a product of two pairings and of four pairings. For $g$ one pairing, the parity image equalled the image with the colours exchanged with no differing pixel. The test was not vacuous: the folded points differed by up to $1.56 \times 10^{-15}$, only 1062 of 25,448 were equal bit for bit, and a rotation of $10^{-3}$ radians changed 2069 pixels |
| The specular term with the factor $(1 - c)$ | 5.10 | The default images, and a white fill | With the clamped form, a channel clips at the half vector when its linear value is above 0.7774, and the default fill `#e9c46a` has red at 0.815. In `*532` 3211 sample channels reached the clamp, and 203 pixels had a channel at 255. `*432`, `*332`, and `*55` gave 6104, 7026, and 7026 sample channels. A white fill clipped 273,921 sample channels, about 3.2 percent of the pixels of the ball, as a flat white patch at the highlight |
| Shading before averaging | 5.10 | `*532` | Shading after averaging differed in 2354 of 262,144 pixels, by at most 3 in a byte, which is not visible |
| The default light $(-1, 1, 2)$ | 5.10 | `*532` | The highlight lay at the upper left, at about pixel (205, 205) |
| The margin of the ball | 5.9 | `*532` | The radius of the ball was 240 pixels |
| The border near the rim | 6.7 | 4000 samples of `*532` at each rim distance, with an oracle that counted leaving the ball as a border | The fractions of samples that the gradient form and the oracle classified differently were 0, 0, 0.0013, 0.0032, 0.011, 0.024, and 0.146 at 120, 24, 8, 4, 2, 1, and 0.5 pixels for a half width of 0.5, and 0, 0.0005, 0.0065, 0.039, 0.073, 0.480, and 0.280 for a half width of 1.5. Within one half width of the rim nearly every difference came from the oracle alone, because the oracle marked every such sample as a border sample. The share of border samples at a half width of 0.5 was 0.041 at 120 pixels, 0.184 at 1 pixel, 0.574 at 0.1 pixels, and 0.999 at 0.01 pixels |
| The dark outline at the rim | 5.9 | A flat render with a white fill and a border width of 1 | The outermost ring of pixels averaged 218 against 245 in the interior, a faint, broken grey ring about 1 pixel wide. Pixels 2 to 6 pixels from the rim averaged 230 to 234. With a border width of 0 the rim was 255. With the default colours, the mean was 117 at 1 pixel against 152 in the interior |
| The border share at large $n$ | 5.9 | The ball at a border width of 1 | The share of border samples was 1.0 for `*(1000)22` and `*(1000)(1000)`, 0.678 for `*(300)22`, 0.657 for `*(300)(300)`, 0.298 for `*(100)22`, and 0.048 for `*532`. The images at $n = 1000$ were a black ball with a grey highlight |

The rotation prototype built each double from the public data of the reflection domain of the crate, as the domain design specifies it, with plain `f64` products of the stored reflections as pairings unless a row states otherwise. Its fold was a copy of the fold of the crate, which matched `fold` bit for bit on the reflection domains that it compared, and its renderer was a copy of the logic of a sample. Every figure of this table was measured on the rotation prototype, outside the repository:

| Choice | Section | Input set | Figures of the rotation prototype |
| --- | --- | --- | --- |
| The fold loop unchanged | 5.3, 5.11 | The rotation fold set of Section 6.8 with the hyperbolic triples | The largest step count in bands 1 to 6 was 16 to 67 for `732`, against 22 to 95 for `*732`, 9 to 40 for `542`, 6 to 24 for `444`, 8 to 35 for `433`, and 14 to 49 for `(12)32`. For `(1000)32` it was 130, 484, 484, 759, 841, and 863, against up to 991 for `*(1000)32`, and for `(1000)(1000)(1000)` 8, 87, 637, 666, 667, and 668. The crate measures the same largest step counts (Section 6.8) |
| The fold cap of 1500 | 4.1, 6.8 | The fold sets of the 294 hyperbolic triples of the domain design, and the canonical images at 512 by 512 pixels with a 4 by 4 grid | The largest step count over the fold sets was 883, at $(992, 990, 979)$, and one fold did not converge, for `(1000)32` in band 6. No sample of a canonical image was unconverged. The largest step counts of the images were 959 for `(1000)32`, 998 for `(1000)(1000)(1000)`, 500 for `(1000)22`, and 250 for `(1000)(1000)`, and the mean step counts were about half those of the reflection domains. With the compensated products, the crate measures no unconverged fold of its rotation fold sets and the same largest step counts of the images (Section 6.8) |
| The Euclidean range of accuracy and the cap | 6.8 | The Euclidean rotation fold set | At distance 65, the largest step count was 116 for `632`, 93 for `442`, 133 for `333`, and 73, 87, 95, and 185 for `2222` with `aspect` 1, 1.618, 0.125, and 8. The crate measures the same (Section 6.8) |
| The spherical rotation domains and the cap | 6.8 | The spherical rotation fold set, and dense samples of the sphere | The largest step count was 4 for `332`, 5 for `432` and 6 on the dense samples, 9 for `532` and 10 on the dense samples, 500 for `(1000)22` and 501 on the dense samples, 499 for `(1000)(1000)` and 500 on the dense samples, 1 for `22`, and 3 for `55`. The crate measures the same over the spherical rotation fold set (Section 6.8) |
| Bounds of their own for the rotation domains | 6.8 | The rotation fold set of `(1000)32` and `(1000)(1000)(1000)` | The largest distance of the folded point from the replay in bands 1 to 6 was $1.7 \times 10^{-12}$, $2.8 \times 10^{-10}$, $2.9 \times 10^{-8}$, $5.5 \times 10^{-6}$, $5.0 \times 10^{-4}$, and $3.0 \times 10^{-2}$ for `(1000)32`, up to 1.84 times the bound of Section 6.4 of `*(1000)32`, and $2.6 \times 10^{-13}$, $4.0 \times 10^{-11}$, $8.9 \times 10^{-9}$, $1.4 \times 10^{-6}$, $1.0 \times 10^{-4}$, and $4.8 \times 10^{-3}$ for `(1000)(1000)(1000)`. The rotation domains of the small triples reached about 0.07 to 0.44 of the bounds of their reflection domains. With the compensated products, the crate still measures distances above the bounds of its reflection domains in some bands (Section 6.8) |
| No assertion on the step counts of "Rotation pairing" | 8.1 | Points $x$ and $P_i x$ with the clearance of that test | The step counts differed by $-10$ to $+12$ for `732`, and by up to $+490$ for `(1000)32` |
| $\tau$ kept, with no factor 8 for the rotation domains | 6.2 | The vertices of the rotation domains of the fold set and the vertex triples, of the 294 triples of the domain design, and of all 166,665,495 hyperbolic triples with orders up to 1000 | The largest ratio was $3.22 \times 10^{-16}$ on the first set, a margin of 12.4, $4.11 \times 10^{-16}$ on the 294 triples, a margin of 9.7, and $1.224 \times 10^{-15}$ over all triples, at $(868, 416, 5)$, a margin of 3.27. Over all triples, 680,942 domains had a ratio above $\tau / 8$, and no vertex had a ratio above $\tau$. The crate measures the same largest ratio at $(868, 416, 5)$ (Section 6.2) |
| The images of the centre with a bound of their own, and a step count of at least the depth | 6.8, 8.1 | The breadth-first enumeration of `332`, `432`, `532`, `n22`, and `nn` | The enumeration reached `group_order()` for every signature. The step count of an image of the centre exceeded its depth by up to 3, for `532`. The largest angle of a folded image of the centre of `(1000)22` was $2.68 \times 10^{-11}$. The crate measures $2.677 \times 10^{-11}$ (Section 6.8) |
| The test "Border subset" at 193 by 193 pixels for the Euclidean domains | 8.1 | The plan triples at 192 by 192 pixels, `632`, `442`, `333`, and `2222` with five values of `aspect` at 193 by 193 pixels, and `532`, `722`, `55`, `332`, `432`, `222`, `22`, `(12)22`, `(1000)22`, and `(1000)(1000)` at 192 by 192 pixels | No pixel violated the subset. At 192 by 192 pixels, 180 pixels of `632` and 155 of `442` did. The crate measures no violation in any domain of the test (Section 6.8) |
| The half-pixel tie of `2222` | 5.8 | `2222` with `aspect` 1 and 4 | With one sample per pixel no pixel was a border pixel, and with a 2 by 2 grid 12,160 and 18,304 pixels were |
| The doubled wall is side 1 | 5.11 | The doubles across side 2 of the hyperbolic triples | The double of `*(1000)32` across side 2 failed check 4 of the domain design with a residual of $2.42 \times 10^{-6}$ against a tolerance of $2 \times 10^{-6}$, and 338 of its folds in band 6 did not converge |

## 12. Assumptions, Choices, and Points for the Owner

### 12.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- The screen tangents of Section 5.2 are the partial derivatives of the map from the disk to the hyperboloid, times $h$. The test "Tangents" checks them (Section 10).
- Applying a pairing to the screen tangents gives the screen tangents of the folded point, because a pairing is linear on the vectors of the linear model. The tests "Border forms" and "Border oracle" check the result through the border estimate.
- The tile of a sample is identified by the image of the domain centre under the inverse of the pairings that the fold applied. The test "Border oracle" relies on it and checks it together with the border estimate.
- The symmetric view has a mirror of the group in the first axis, because the wall of side 0 passes through vertex 2 and vertex 0. The test "Symmetric view" checks it.
- The bounds and the allowance measured on Linux x86-64 hold on the other platforms of the project. Section 10 lists it as an unverified statement.
- For a Euclidean domain, $\lvert f_i \rvert / g_i$ is the distance in pixels from the sample to the wall of side $i$ in its tile, up to rounding (Section 5.8). The tests "Euclidean border forms", "Euclidean border oracle", and "Rectangle grid" check it.
- The mirrors of `*2222` are the lines of the grid of its rectangle. The test "Rectangle grid" relies on it, and Section 10 lists it.
- For `*2222` with `aspect` 1 or 4, a power-of-two image size, and a grid of 1, 2, 4, 8, or 16, no operation of the construction, the view, the fold, or the border estimate rounds (Section 6.6). The exact rows of the test "Periodicity" and the test "Half-pixel tie" rely on it.
- The screen tangents of Section 5.9 are the partial derivatives of the lift onto the ball, times $h$, and $V^{\mathsf{T}}$ turns them with the point, because $V^{\mathsf{T}}$ is linear. The test "Ball tangents" checks the first part, and the tests "Spherical border forms" and "Spherical border oracle" check the result through the border estimate.
- A sample whose circle of the half width leaves the ball is near a wall only if a point of the circle inside the ball lies in another tile. The oracle of the test "Spherical border forms" relies on it, and the test skips the bands where that reading and the renderer cannot agree.
- Rotating the view by an element of the group leaves the tiling unchanged, and a reversing element exchanges the two parity colours. The test "Rotation by a group element" relies on it and checks it numerically.
- The fold of a point in a rotation domain lands where its fold in the reflection domain lands when that fold has an even parity, and at the image of that point under the reflection in the doubled wall when the parity is odd. The test "Double oracle" relies on it and checks it numerically.
- Every side of a rotation domain lies on a wall of a tile of its reflection domain, so every border of the rotation tiling is a border of the reflection tiling in the same view. The test "Border subset" relies on it and checks it numerically.

These readings are confirmed by the crate:

- The `png` error type, `png::EncodingError`, derives only `Debug`, so `Error` derives only `Debug` (Section 7.1).
- The sRGB transfer function maps the bytes 0 and 255 to exactly 0 and 1 in linear light, so `Colour::BLACK` and `Colour::WHITE` are constants equal to their constructors. The test "Colour constants" checks it.

### 12.2. Choices of This Design

No other document dictates these choices:

| Choice | Section |
| --- | --- |
| No public `View` or model type, and a public `fold` with a visitor | 3.1, 3.4 |
| `RenderOptions::builder` takes the fill, so the library has no default fill colour | 3.1, 4.2 |
| `Renderer::new` is infallible and copies what it needs from the domain | 3.2 |
| `RenderStats` holds integer counts only | 3.2 |
| The crate starts no thread, and the caller splits the image into bands | 3.2 |
| Colour text is parsed in the command line, and the library takes 8-bit sRGB | 4.2 |
| A largest fold cap of 100000, and the other limits of Section 4.1 | 4.1 |
| The margin of $1/32$ of the shorter side on each side of the disk | 5.1 |
| The screen tangents carried through the fold in place of an accumulated matrix | 5.2, 6.1 |
| No renormalization in the fold or of its result | 5.3, 6.1 |
| A fold converges only at a finite point with a positive point size inside every side, and every other stop is unconverged | 5.3, 6.3 |
| A positive tolerance constant relative to $\sigma_i \lambda(x)$, as a robustness constant and not a bound on error, with the value $4 \times 10^{-15}$, a margin of 13.1 over the largest ratio measured at a vertex | 6.2 |
| A NaN side value, or a NaN or infinite gradient size, makes a border sample | 5.4, 6.3 |
| A count bound of the larger of 8 times the measured count and 8 | 6.5 |
| The perceptual rendering intent of the sRGB chunk | 5.6 |
| The sRGB chunk without a gAMA or cHRM chunk | 5.6 |
| The range of accuracy ends at $1 - 10^{-6}$ | 6.4 |
| The error texts of Section 7.2 | 7.2 |
| Golden images compared in linear light through a table from the reference transfer function, with an allowance of 16 pixels per image | 8.4 |
| The symmetric view with one fill colour and an even image size, compared within the golden allowance | 8.1 |
| One definition of the golden configuration, the allowance, the linear-light table, and the comparison, loaded by the unit tests through `#[path]` | 8.1 |
| The pairing property of a reflection domain with step counts that differ by exactly 1 and different parities, for points whose clearance from every wall is at least the larger of $10^{-6}$ and twice the bound of the outer band | 8.1 |
| Unit tests for the quantities that only private items expose, with one reference module loaded through `#[path]` | 8.1 |
| The test "Rim" renders only the rows that hold a sample beyond the range of accuracy | 8.1 |
| `png` as the one new dependency, with `flate2` held at 1.1.9 in the lock file | 9 |
| The affine plane over the same types, so that the fold and the border estimate serve a Euclidean domain unchanged | 5.8 |
| The Euclidean view: no margin, the centre of the domain at the centre of the image, and $S = 8$ units across the shorter side | 5.8 |
| `Renderer::new` stays infallible and chooses the view from the geometry, and every geometry has a view | 5.8 |
| No border at a tie of exactly the half width, so `*2222` with `aspect` 1 or 4 at one sample per pixel shows no border | 5.8 |
| The range of accuracy for a Euclidean domain ends at distance 65 from the centre | 6.6 |
| The tolerance constant, the fold cap, and the golden allowance keep their values for a Euclidean domain | 6.2, 6.6, 8.4 |
| The periodicity tests in the canonical view, exact only for `*2222` with `aspect` 1 and 4, and a periodic view only for `aspect` 1.618 | 8.1 |
| Four Euclidean golden images, and the oracle of the rectangle grid | 8.1, 8.4 |
| The bound of a cell of the Euclidean fold replay measured as exactly 0 is the largest bound at the same distance | 6.6 |
| The Euclidean border oracle at 193 by 193 pixels, so that no mirror of `*632` falls on a boundary between pixels | 8.1 |
| `escher-signature` as a normal dependency, because `Renderer::new` names `Geometry` | 9 |
| The point size of the sphere as the largest absolute coordinate, selected by a private enum, so that the hyperbolic and Euclidean decisions stay the same bit for bit | 5.3 |
| No bound from the group order in the code. The fold cap is the only bound, and the tests compare the largest fold count with the group order | 6.7 |
| The ball view: an orthographic projection along the third axis, with the size and the margin of the disk, the front hemisphere only, and a sample on or outside the rim as an outside sample with the background colour, unshaded | 5.9 |
| The lift onto the ball and its tangents private in the view | 3.4, 5.9 |
| The viewing rotation as the yaw, the pitch, and the roll, which turn the ball about fixed axes of the screen in that order, with signs defined by the motion on the screen | 5.9 |
| One white light fixed to the viewer, with the default direction $(-1, 1, 2)$, normalized by `build` | 4.1, 5.10 |
| The shading constants $k_a = 0.25$, $k_d = 0.75$, $k_s = 0.25$, and $\alpha = 32$ by five squarings, with no option to change them | 5.10 |
| The specular term as $k_s \rho (1 - c)$, so that no channel leaves the range 0 to 1 and no clamp is needed | 5.10 |
| Shading of each sample, of fill, border, and unconverged samples alike, before the samples are averaged | 5.10 |
| `Shading::Flat` for no shading | 3.2 |
| The options of the ball checked for every domain and read only for a spherical one | 4.1 |
| The range of accuracy of the border estimate on the ball stated by rim band, and a border oracle that does not count leaving the ball as a border and skips the bands closer to the rim than the half width | 6.7, 8.1 |
| Seven spherical golden images, and the test "Rotation by a group element" within the golden allowance | 8.1, 8.4 |
| The bins of the test "Ball tangents": the rim bands as lower ends, and one bin below 0.5 pixels | 6.7 |
| Bounds of their own for the images of the centre at $n = 1000$, and for an $n$ outside the spherical domain set the bound of the next larger $n$ of the set | 6.7 |
| The test "Brightest point" compares the centroid of the tied brightest pixels, on a black background | 8.1 |
| The test "Spherical border oracle" renders its border mask with `Shading::Flat` | 8.1 |
| The accuracy of `fold` for a spherical domain stated only for a vector whose largest absolute coordinate is at least $10^{-300}$ and whose fold converges, with no rescaling of a smaller vector | 2.1, 6.1 |
| One fold for every pairing kind, with no fallback that folds in the reflection domain and applies the reflection in the doubled wall on an odd parity | 5.3, 5.11 |
| `Fill::Parity` on a domain with no reversing side gives its first colour, with no error in the library | 5.11 |
| Bounds of their own for the rotation domains, and the range of accuracy of the geometry unless a measured fold narrows it | 6.8 |
| The range of accuracy of a hyperbolic rotation domain whose corner orders are all near 1000 excludes a radius of $10^{-6}$ around its vertices of order near 1000 other than vertex 0, 10 times the largest distance at which a fold of a rotation domain failed | 2.1, 6.8 |
| The test "Near vertex" asserts convergence of the rotation domains from the next measured distance beyond the largest that failed, and at every measured distance for a signature where none failed | 6.8, 8.1 |
| The rotation properties "Inside" and "Rotation pairing" discard only the folds of `(1000)(1000)(1000)` that stop unconverged at exactly the default cap | 8.1 |
| The test "Vertices" asserts for the rotation domains only that every vertex folds in zero steps, with no factor 8 | 6.2, 8.1 |
| The test "Rotation pairing" asserts the folded point and the parity, and nothing about the step counts | 8.1 |
| The test "Double oracle" compares within the sum of the bounds of the two domains, with a count bound on the points that fail, and asserts at least one kept point for each domain, not for each band | 6.8, 8.1 |
| A pairing bound of its own for each spherical rotation domain in the test "Rotation pairing", measured over the grid of the test "Spherical rotation pairing" and never below the bound of the folded point | 6.8, 8.1 |
| The counts of fold decisions and of the double oracle kept for each rotation domain and each band, distance, or domain, not summed over the domains | 6.8 |
| The test "Border subset" asserts at most as many border pixels, and runs the Euclidean domains at 193 by 193 pixels | 8.1 |
| The test "Perturbation" moves the covectors and the pairings of a rotation domain, because the renderer holds no reflection of it | 8.1 |
| Six rotation golden images in the configuration of the existing ones | 8.4 |

### 12.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Renormalization. `DEC-31` and Section 9.3 of the architecture propose that the fold renormalizes nothing. This design follows it on the evidence of Section 11, and keeps no accumulated matrix (Section 6.1).
2. Unconverged samples. A fold that reaches the cap, meets a NaN side value, or meets a point that is not finite or whose point size is not positive is unconverged, and the sample takes the border colour (Section 5.5).
3. The tolerance constant. It is a positive robustness constant of $4 \times 10^{-15}$, 13.1 times the largest ratio measured at a vertex of a domain, $3.047 \times 10^{-16}$ (Section 6.2). The alternative is a smaller value with a smaller margin, such as $10^{-15}$, whose margin over the same set is 3.3.
4. The default fold cap of 1500 and the largest cap of 100000 (Section 4.1). A cap of 500 changes no pixel of the canonical view of the plan triples at border widths of at least 0.1 pixel on the prototype, but leaves samples near a corner of order 1000 unconverged (Section 11). The alternatives are a cap of 500, or a default derived from the largest corner order of the signature.
5. The appearance near a corner of high order. With a border of 1 pixel, `*(1000)32` shows a solid region of border colour about 110 pixels wide around its corner of order 1000 on the prototype at 512 by 512 pixels. It follows from the definition of a border width in pixels and does not depend on the cap.
6. The shape of the library. It has no public `View` or model and a public `fold` (Section 3.4), starts no thread (RND-2), and parses no colour text (RND-5).
7. The `flate2` pin of Section 9. A pin in the lock file is lost on a plain `cargo update`. A `skip` entry for `miniz_oxide` 0.8 in `deny.toml` is durable but is an exception to the policy of Section 8 of `docs/design/tooling.md`.
8. Golden images and tests. The images are compared in linear light, with an allowance of 16 pixels per image (Section 8.4). The perturbation of one unit in the last place changed no pixel, so 8 times the measured count gives 0, which fails on the first platform difference, and the test "Perturbation" asserts the allowance. 16 pixels cover a difference of the size of the rotation of $10^{-6}$ that changed 4 to 9 pixels on the prototype, and every defect tried that changes an image changes at least 828 pixels of it. The symmetric view uses one fill colour and asserts the same allowance, because its equality with the mirror image, which held with no differing pixel on the measured platform, is not exact in IEEE 754 arithmetic (Section 8.1). The pairing property of a reflection domain asserts that the folded points agree within the bound of Section 6.4, that the step counts differ by exactly 1, up or down, and that the parities differ (Section 8.1).
9. The range of accuracy ends at $1 - 10^{-6}$, the radius that the plan names, and beyond it the crate promises only a result without a panic (Section 6.4).
10. The margin of $1/32$, the perceptual rendering intent, the limits of Section 4.1, and the error texts of Section 7.2 are constants with no source. The PNG file has no gAMA or cHRM chunk beside the sRGB chunk, against the recommendation of the PNG specification (Section 5.6).
11. The count bound. A count measured as $c$ has the bound of the larger of $8c$ and 8, because 8 times a count of 0 is not a usable bound (RND-16). It allows up to 8 differing fold decisions in bands 1 to 5, where none was measured.
12. The clearance of the pairing property. The test keeps only points whose distance from every wall is at least the larger of $10^{-6}$ and twice the bound of the folded point of the outer band, because near the rim the error of the fold exceeds a clearance of $10^{-6}$ (Section 8.1). The property is then not tested for points close to a wall in bands 5 and 6.
13. The Euclidean view (Section 5.8, `DEC-12`). The tiling fills the image with no margin, the centre of the domain is at the centre of the image, and the shorter side spans 8 units. The alternatives are another span, such as one that shows a fixed number of tiles, or a window that depends on the size of the domain.
14. The ball view (Section 5.9). An orthographic projection of the front hemisphere, with the size and the margin of the disk, and a sample on or outside the rim takes the background colour, unshaded. Every geometry has a view, so `Renderer::new` stays infallible. The alternatives are a view of the back hemisphere, or a margin of its own for the ball.
15. The range of accuracy for a Euclidean domain ends at distance 65, which covers the canonical view of an image whose sides are in a ratio of at most 16 to 1 (Section 6.6). The alternative is 64 along the longer axis, which leaves the corners of such an image outside the range.
16. The constants kept. The tolerance constant, the fold cap, and the golden allowance keep the values measured for hyperbolic domains. Every Euclidean bound has a margin of at least 8, and $\tau$ has a margin of 64.2 at the Euclidean vertices (Sections 6.2 and 6.6).
17. Periodicity (Section 8.1). Only `*2222` with `aspect` 1 and 4 asserts exact equality, and the triangles assert the golden allowance, because IEEE 754 arithmetic does not guarantee their equality.
18. No border at one sample per pixel for `*2222` with `aspect` 1 or 4 (Section 5.8). The design records it as a property of point sampling. The alternative is a span $S$ that keeps the walls off the pixel centres, which loses the exact periodicity of these two cases.
19. The size of the Euclidean border oracle (Section 8.1). The test runs at 193 by 193 pixels, because at 192 by 192 rounding decides the border pixels at the mirrors of `*632`, which lie exactly on boundaries between pixels, and 172 pixels where the tile changes had no border pixel within one pixel. The alternatives at 192 by 192 are an exemption for a change whose two pixels both lie half a pixel from a mirror, a count bound of 1376 on such changes, or 2 samples per pixel.
20. `escher-signature` as a normal dependency (Section 9). `Renderer::new` names `Geometry` in its `match` (RND-23), so the crate cannot keep `escher-signature` as a dev-dependency.
21. The bound of a cell of the Euclidean fold replay measured as exactly 0 is the largest bound at the same distance (Section 6.6), because 8 times 0 gives a bound of 0 that fails on the first platform that rounds one operation differently.
22. The point size of the sphere (Section 5.3). The largest absolute coordinate, which changed no hyperbolic or Euclidean decision on the sphere prototype. The alternative is the sum of the absolute coordinates.
23. No bound from the group order in the code (Section 6.7). The fold cap stays the only bound, and a sample near a vertex of very high order can stay unconverged and take the border colour. The alternative is a default cap derived from the group order.
24. The viewing rotation (Section 5.9). The yaw, the pitch, and the roll turn the ball about fixed axes of the screen, in that order, with signs defined by the motion on the screen. The alternative is the angles of a rotation, a tilt, and a second rotation, which need one matrix fewer but make the first and the third angle do the same thing when the tilt is 0.
25. The light (Sections 4.1 and 5.10). One white light fixed to the viewer, from the default direction $(-1, 1, 2)$, at the upper left front of `DEC-16`. The alternatives are a light fixed to the ball, or another default direction.
26. The material (Section 5.10). The constants $k_a = 0.25$, $k_d = 0.75$, $k_s = 0.25$, and $\alpha = 32$, with no option to change them. The alternative is an option for each constant.
27. The specular term (Section 5.10). The form $k_s \rho (1 - c)$ never clips a channel. The alternatives are the clamped form of the Phong model, which clipped channels of the default fill colours on the sphere prototype, or constants with $k_a + k_d + k_s \le 1$, which dim every colour.
28. Shaded borders (Section 5.10). Borders and unconverged samples are shaded like fills, as `DEC-16` proposes, so a black border shows a grey highlight. The alternative is unshaded borders.
29. The options of the ball on another geometry (Section 4.1). They are checked and have no effect. The alternative is an error.
30. The dark outline at the rim (Section 5.9). The design accepts it and states the range of accuracy of the border estimate by rim band. The alternative is a fade or a cutoff of the border near the rim.
31. Borders at large $n$ (Section 5.9). At $n = 1000$ the ball is entirely border colour at the default size. The alternatives are a lower limit on $n$, or a warning when every sample is a border sample.
32. Shading before averaging (Section 5.10). The alternative is shading the averaged pixel, which changed pixels by at most 3 in a byte on the sphere prototype.
33. The range of accuracy of the border estimate on the ball (Section 6.7). It is defined as the rim bands in which the count bound holds, but each count bound is set from the count measured in its own band, so the definition holds in every band by construction and does not limit the range. The measured shares of differing samples, out of 20,000 per band, are these:

    | Rim distance in pixels | Differing at half width 0.5 | Differing at half width 1.5 |
    | --- | --- | --- |
    | 120 | 0 percent | 0.015 percent |
    | 24 | 0.015 percent | 0.115 percent |
    | 8 | 0.04 percent | 0.495 percent |
    | 4 | 0.195 percent | 1.68 percent |
    | 2 | 0.46 percent | 4.78 percent |
    | 1 | 1.39 percent | Skipped |
    | 0.5 | 7.965 percent | Skipped |

    The design keeps the definition until the owner decides. One alternative is a largest share of differing samples, such as 1 percent. With that share, the range would end at 2 pixels from the rim for the half width 0.5, and at 8 pixels for the half width 1.5.
34. Small vectors on the sphere (Sections 2.1 and 6.1). `fold` states its accuracy for a spherical domain only for a vector whose largest absolute coordinate is at least $10^{-300}$, and folds a smaller vector without rescaling it, so its decisions can differ from those of the same ray at length 1. The renderer passes only points of length about 1. The alternative is to multiply the point and the screen tangents once by $2^{1000}$ before the loop when the point size is below a threshold such as $2^{-900}$. The multiplication is exact and costs one comparison per fold, but `Fold::point` of such an input then returns the rescaled vector.
35. The margin of $\tau$ at the vertices of the rotation domains (Section 6.2). The crate measures a margin of 3.27 at the vertices of the rotation domains, below the factor 8 that the other vertex sets meet (Section 6.2 gives the figures of the crate and of the prototype). Every vertex still folds in zero steps. The design keeps $\tau = 4 \times 10^{-15}$, because a larger value changes the images of the reflection domains (RND-32), and the test asserts only that every vertex folds in zero steps. The alternatives are a larger $\tau$ with new golden images of the reflection domains, or a construction of the image vertices and covectors that the domain design rounds more accurately, which the prototype did not test.
36. The fold of a rotation domain (Section 5.11). One loop serves every pairing kind (RND-31), and the rotation domains have bounds of their own (Section 6.8). The alternative folds in the reflection domain and applies the reflection in the doubled wall on an odd parity, which rests on `UNV-1` alone, but the later classes of signature need the fold with general pairings anyway.
37. The parity colouring of a rotation domain (Section 5.11). `Fill::Parity` gives the first colour to every tile, and the library reports no error, while the command line refuses `--colouring parity` for such a domain (`DEC-32`). The alternative is an error kind of the library.

## 13. Security Considerations

The threat model is a caller that passes options and a domain derived from untrusted text to the crate, such as `escher-cli` with the arguments of a user. `escher-signature` has bounded the signature, and `escher-domain` has rejected every signature without a domain, every corner order above 1000, and every value of a shape parameter outside its range (Section 16 of the domain design). The attacker can choose any domain that `build` returns and any value of each option. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of memory through a large image, exhaustion of time through a large image, a fine sample grid, and a high fold cap, a panic that stops the caller, and an image that the caller trusts as correct although it is wrong. The crate counters them with the limits of this section, which RND-4 requires, the rule that it never panics (RND-14), and the tests of Section 8:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Width and height | 16384 pixels each | A choice of this design. The pixel limit bounds the memory |
| Pixels | $2^{26}$ | The buffer of `render` holds 3 bytes per pixel, about 201 MB at this limit. A choice of this design |
| Sample grid | 16 by 16 | 256 samples per pixel, far beyond the 16 of [NS]. A choice of this design |
| Border width | 1024 pixels | The largest image side divided by 16. A choice of this design |
| Fold cap | 100000 | About 80 times the largest step count that the prototype measured. The time of an unconverged sample is proportional to the cap. A choice of this design |

The limits bound memory. They do not bound time usefully when every option is at its limit together: $2^{26}$ pixels, 256 samples per pixel, and a cap of 100000 allow about $1.7 \times 10^{15}$ steps for an image whose samples do not converge. This is a residual risk. A caller that renders untrusted input with a deadline applies its own limits on top of these.

The crate allocates the buffer of `render` and nothing that depends on the domain beyond its two to six sides. For a spherical domain the work of a sample is bounded by the fold cap, as for the other geometries, and the options of the ball add no limit, because `build` rejects every value that is not finite and each costs a fixed number of operations per sample. `render_rows` writes only into the buffer of the caller.

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library. `png` forbids unsafe code in its own source. The crates `crc32fast` and `simd-adler32`, on which it depends, hold architecture-specific code behind runtime detection.
- Untrusted input and file paths: the options, the domain, the points of `fold`, and the buffers are the inputs. The crate reads and writes no file, keeps no state between calls, and returns the PNG file as bytes. Section 4.7 of the command-line design states how the file is written.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only. Among `png` and its dependencies, only `crc32fast` 1.5.5 has a build script.
- Dependencies: `Cargo.lock` fixes every version, including `flate2` at 1.1.9, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate neither builds nor writes anything outside the memory that the caller gives it.

The residual risks are three. First, the limits do not bound time, as stated above. Second, an image is wrong where the fold decides wrongly: beyond the range of accuracy the crate states no accuracy, and within it the bounds of Section 6.4 allow a stated number of differing decisions per band. Third, the border estimate and the parity colouring rest on unverified statements (Section 10), which the tests check numerically but no source establishes.

## 14. Operational Considerations

The crate declares no Cargo feature and has no build script. The golden-image test reads `CARGO_TARGET_TMPDIR`, which Cargo sets for integration tests, and `ESCHER_UPDATE_GOLDEN`, which the recipe `update-golden` sets (Section 8.4). The library reads no environment variable. It is pure Rust with no platform-specific code of its own, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log. `RenderStats` is the measurement that the caller reports.

| Failure | Reported as | Recovery |
| --- | --- | --- |
| An option outside its accepted values | `InvalidOption` | Use a value in the column "Accepted" of Section 4.1 |
| An option above its limit | `LimitExceeded` | Use a value within the limit |
| A buffer or band that does not fit the image | `InvalidBuffer` | None. It is a defect of the caller |
| `png` fails to encode | `Encoding`, with the `png` error as `source()` | None. It is a defect of the crate or of `png` |
| A sample that does not converge | `Fold::converged` is false, the sample takes the border colour, and `RenderStats::unconverged` counts it | Raise the fold cap, or accept the border colour near the limit circle, far from the centre of a very elongated Euclidean image, or near a vertex of very high order on the ball or of a rotation domain |
| A sample outside the disk or the ball | The background colour, counted by `RenderStats::outside` | None needed |

## 15. References

### 15.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 15.2. Informative References

The tags of the mathematical and software sources resolve to the reference sections of the research documents:

- [CFKP] Section 14 of `docs/research/rendering.md`, and Section 16.1 of `docs/research/geometry-models.md`.
- [DAVIS] Section 14 of `docs/research/rendering.md`.
- [DUN86] Section 16.1 of `docs/research/geometry-models.md`.
- [NS] Section 14 of `docs/research/rendering.md`.
- [PHONG75] Section 16.4 of `docs/research/geometry-models.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [SERIES2013] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SOLOMYAK2020] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SYMMHUB] Section 14 of `docs/research/rendering.md`.
- [VGRG] Section 14 of `docs/research/rendering.md`.
- [W-PHONG] Section 16.3 of `docs/research/geometry-models.md`.
- [ZUCKER] Section 14 of `docs/research/rendering.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| RND-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| RND-2 | The crate starts no thread | 3.2 |
| RND-3 | The colour of a pixel depends only on the domain, the options, and its position, never on its band or another sample | 3.2 |
| RND-4 | `build` rejects each value outside the accepted values of Section 4.1, and the builder starts from its defaults | 4.1 |
| RND-5 | Colours enter as 8-bit sRGB, and the crate parses no colour text | 4.2 |
| RND-6 | Samples and disk positions follow the formulas of Section 5.1 | 5.1 |
| RND-7 | The renderer and `fold` fold by the steps of Section 5.3, through one implementation | 5.3 |
| RND-8 | The fold renormalizes neither the point nor the screen tangents | 5.3 |
| RND-9 | A converged sample is a border sample by the negated gradient comparison | 5.4 |
| RND-10 | Each sample takes the colour of the first row of Section 5.5 that applies | 5.5 |
| RND-11 | The sRGB transfer function comes from Section 7.5 of the rendering reference | 5.6 |
| RND-12 | `encode_png` writes RGB at bit depth 8 with the sRGB chunk and no gAMA or cHRM chunk, in memory | 5.6 |
| RND-13 | The tolerance constant is positive and the threshold is $\tau \sigma_i \lambda(x)$ | 6.2 |
| RND-14 | `fold` and the renderer never panic | 6.3 |
| RND-15 | The bounds of Section 6.4 hold in each radius band, and beyond the range of accuracy only a result without a panic is promised | 6.4 |
| RND-16 | A test of a quantity without a measured bound here sets and records its bound as TEST-13 requires, in the same change, and a bound on a count is the count bound | 6.5 |
| RND-17 | `Error` follows ERR-1, with the operation that failed | 7.1 |
| RND-18 | The `Display` text of each error is the one stated | 7.2 |
| RND-19 | The reference module shares no code with the crates under test | 8.3 |
| RND-20 | Golden images are compared in linear light, through a table from the reference transfer function, with the allowance of Section 8.4 | 8.4 |
| RND-21 | Property tests take their configuration from one helper that is Miri-aware | 8.5 |
| RND-22 | The crate declares only the dependencies of Section 9, each from the workspace table | 9 |
| RND-23 | `Renderer::new` chooses the view from the geometry of the domain: the disk, the plane, or the ball | 5.8 |
| RND-24 | For a Euclidean domain, samples, plane positions, points, and screen tangents follow the formulas of Section 5.8 with $S = 8$ | 5.8 |
| RND-25 | For a Euclidean domain, the bounds of Section 6.6 hold within distance 65 of the centre, and beyond it only a result without a panic is promised | 6.6 |
| RND-26 | The fold reads the point size at steps 2 and 3, which for a hyperbolic or Euclidean domain is $x_3$ with the threshold $\tau \sigma_i x_3$ taken from left to right | 5.3 |
| RND-27 | For a spherical domain, samples, ball positions, the lift, the screen tangents, and the viewing matrix follow the formulas of Section 5.9 | 5.9 |
| RND-28 | The viewing matrix applies the yaw, the pitch, and the roll in that order about fixed screen axes, with signs defined by the motion on the screen | 5.9 |
| RND-29 | With `Shading::Phong`, every sample of the ball is shaded by the formula and constants of Section 5.10 before averaging, with the power by squaring, and `Shading::Flat` changes no colour | 5.10 |
| RND-30 | For a spherical domain, the bounds of Section 6.7 hold for a converged fold of a vector whose largest absolute coordinate is at least $10^{-300}$ and of a ball position inside the rim, the border count bounds hold in the rim bands of the range of accuracy, and closer to the rim only a result without a panic is promised | 6.7 |
| RND-31 | The fold of a rotation domain follows Section 5.3, with no branch on the kind of a side and no change to $\tau$ or the order of operations | 5.11 |
| RND-32 | Support for rotation domains changes no decision, colour, or statistic of a reflection domain | 5.11 |
| RND-33 | `Fill::Parity` on a domain with no reversing side gives its first colour, and `Renderer::new` stays infallible | 5.11 |
| RND-34 | The border estimate tests the sides of the domain only, so the doubled wall draws no border | 5.11 |
| RND-35 | For a rotation domain, the bounds of Section 6.8 hold in its range of accuracy, and beyond it only a result without a panic is promised. For a hyperbolic rotation domain whose corner orders are all near 1000, the range excludes the points within a hyperbolic distance of $10^{-6}$ of a vertex of order near 1000 other than vertex 0 | 6.8 |
