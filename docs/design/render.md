# Render Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-16 |
| Applies to | `crates/escher-render` |

## Abstract

This document specifies the crate `escher-render`, which turns the fundamental domain of a compact hyperbolic reflection triangle into the pixels of an image in the Poincare disk and encodes them as a PNG file in memory. For every sample of a regular grid in each pixel, the crate maps the sample to a point of the linear model, folds the point into the domain, decides whether the sample lies on a border of a tile, and chooses a fill, border, or background colour. It then averages the samples of each pixel in linear light and stores the pixel as 8-bit sRGB. The document defines the public types and operations, the steps of the fold and of the border estimate with the source or the unverified status of each formula, the behaviour of the fold for points on walls, near the limit circle, and with non-finite coordinates, the range over which the crate states its accuracy, the error type, the limits on the options, the tests with the golden images and the reference arithmetic, and the dependencies. It also lists the unverified statements that the code relies on, the evidence behind the choices of this design, the assumptions, and the points that the owner decides.

## 1. Introduction

### 1.1. Problem

The proof of concept draws a tiling by pulling each sample back into the fundamental domain, one sample at a time (Section 4.1 of the architecture). The research gives the loop that folds a point into the domain and the theorem that makes the triangle a strict fundamental domain, but no bound on the number of steps, no behaviour for a point on a wall, and no form of a border width in screen pixels through a chain of folds in the linear model (Sections 4.7, 6, 7, and 13.1 of the rendering reference). Floating-point evaluation adds a second risk: near the limit circle the map from the disk to the hyperboloid loses digits, and the fold then applies a different side than exact arithmetic would. A wrong decision produces an image that looks plausible, so the design states where the crate is accurate, what it does beyond that range, and how the tests measure both.

### 1.2. Purpose

`escher-render` is the one place where a `FundamentalDomain` becomes pixels. It exposes the fold as a public function, a renderer that fills a band of rows in a buffer that the caller owns, statistics of the folds, a colour type, and PNG encoding. Section 3 of the architecture places the crate in the workspace. It depends on `escher-geometry` for the linear model and the disk map and on `escher-domain` for the domain, and `escher-cli` consumes it through the command `escher render`.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the author of the command-line design, which specifies the command `render` on top of this crate, and the authors of the later milestones that add models, intrinsic borders, analytic antialiasing, and colourings by cosets.

### 1.4. Scope

This document covers the public surface of the crate, the options with their defaults and limits, the map from a sample to the disk and to the hyperboloid, the fold, the border estimate, the colour of a sample and of a pixel, PNG encoding, the numerical behaviour, the error type, the tests, and the dependencies.

The crate renders every domain that `FundamentalDomain::build` returns, which is the compact hyperbolic `*pqr` with corner orders up to 1000 (Section 4 of the domain design), in the canonical view of the Poincare disk.

### 1.5. Non-Goals

These topics are owned elsewhere or come later:

- The command `escher render` is specified in the command-line design: its options and the parsing of colour text (Section 4.6), the parallel split of the image into bands of rows (Section 4.5), the output file and the statistics text (Section 4.7), and the diagnostics (Section 6.8).
- The bilinear form, the side test, the isometries, and the disk map, with their formulas and accuracy, belong to `escher-geometry` and are specified in the geometry design. This document calls those functions by name and restates none of their formulas.
- The construction, placement, and verification of the domain belong to `escher-domain` and are specified in the domain design.
- Models other than the Poincare disk, a public `View`, and borders of intrinsic width come with the milestones of Sections 5.12 and 5.13 of the plan.
- Analytic antialiasing and the fade of tiles smaller than a pixel come with the milestone of Section 5.14 of the plan.
- Colourings by cosets and generated palettes need `escher-group` and come with the milestone of Section 5.8 of the plan.
- A transparent background and the spherical and Euclidean geometries are outside this design.

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
| `docs/research/rust-ecosystem.md` | the ecosystem survey |
| `docs/rust-coding-standards.md` | the coding standard |

A function written as `hyperbolic::name` is the function of that name in the module `hyperbolic` of `escher-geometry`.

It adds these terms:

- image size: The width $W$ and the height $H$ of the image in pixels.
- sample grid: The $n$ by $n$ grid of samples in each pixel, with $n$ the value of the option `samples`.
- sample weight: The weight $1/n^2$ of one sample in the mean of its pixel.
- margin: The fraction $m$ of the shorter side of the image that the disk leaves free on each side (Section 5.1).
- disk scale: The length $h$ in disk units of one pixel (Section 5.1).
- screen tangents: The two vectors $T_1$ and $T_2$ that the crate carries through the fold with the point. Each is the change of the point of the linear model for a step of one pixel along one axis of the screen (Section 5.2).
- tolerance scale: The sum $\sigma_i$ of the absolute values of the components of the covector of side $i$ (Section 5.3).
- side value: The value $f_i$ of `hyperbolic::side` for the covector of side $i$ at the folded point.
- gradient size: The value $g_i$ of Section 5.4, the change of the side value $f_i$ per pixel.
- half width: Half the border width, in pixels.
- step: One application of a pairing isometry by the fold.
- reversal: A step whose pairing reverses orientation.
- parity: The number of reversals of a fold modulo 2.
- converged: The result of a fold that stopped with a finite point that violates no side (Section 5.3).
- unconverged: The result of every other fold, which reached the fold cap, met a non-finite coordinate, or met a side value that is NaN.
- border sample: A converged sample that the border estimate of Section 5.4 places on a border.
- outside sample: A sample whose disk position `hyperbolic::from_disk` rejects.
- tolerance constant: The constant $\tau$ of the side test of the fold (Section 6.2).
- radius band $k$: The disk positions $z$ with $\lvert z \rvert = 1 - 10^{-k}$, for an integer $k$ from 1 to 6.
- range of accuracy: The disk positions with $\lvert z \rvert \le 1 - 10^{-6}$ (Section 6.4).
- linear light: Colour values proportional to light intensity, in which the crate averages samples.
- 8-bit sRGB: Colour values encoded by the sRGB transfer function and stored as one byte per channel.
- band: A run of whole rows of the image, which `render_rows` fills.
- canonical view: The view of the whole disk with the centre of the domain at the centre of the image (`DEC-12`).
- symmetric view: A view available only to the unit tests, which places vertex 2 of the domain at the centre of the image and vertex 0 on the positive first axis (Section 8.1).
- allowance: The two limits of a golden-image comparison, the largest difference of a pixel and the number of pixels that may differ (Section 8.4).
- prototype: An implementation of Sections 5 and 6 with a 256-bit reference, written outside the repository to test this design. Section 11 states what it measured.

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

pub struct RenderOptions { /* private */ }
// derives Clone, Debug, PartialEq
impl RenderOptions {
    #[must_use] pub fn builder(fill: Fill) -> RenderOptionsBuilder;
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

The crate does not re-export `glam` (API-10 of the coding standard). A caller that names a `glam` type declares `glam` from the workspace entry.

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `Colour` | An opaque colour, stored in linear light | Each channel is finite and in $[0, 1]$, because the only constructors take 8-bit values |
| `Colour::BLACK`, `Colour::WHITE` | Black and white | Equal to `from_srgb8(0, 0, 0)` and `from_srgb8(255, 255, 255)` |
| `from_srgb8` | Converts 8-bit sRGB to linear light by the transfer function of Section 5.6 | Total |
| `to_srgb8` | Converts linear light to 8-bit sRGB by the transfer function of Section 5.6 | `to_srgb8(from_srgb8(r, g, b))` is `[r, g, b]` for every byte value |
| `Fill` | The colouring with its colours | The number of colours matches the colouring by construction (TYPE-3 of the coding standard), so the crate has no error for a wrong number of colours |
| `RenderOptions` | Validated options | Built only by `build`, which checks every limit of Section 4.1 |
| `RenderOptions::builder` | Starts from the defaults of Section 4.1 with the given fill | Infallible |
| `build` | Validates the options | Returns `InvalidOption` or `LimitExceeded` as Section 7.2 states. Never panics |
| `Renderer::new` | Copies the covectors, pairings, and orientations of the sides of the domain and computes the disk scale and the tolerance scale of each side | Infallible, because the domain and the options are valid by construction. The renderer keeps no reference to the domain |
| `render_rows` | Renders the rows from `first_row` into the caller's buffer | The buffer holds 8-bit sRGB, 3 bytes per pixel, red first, the top row first. Its length MUST be a positive multiple of $3W$, and the band MUST lie inside the image, else `InvalidBuffer`. It starts no thread (RND-2) |
| `render` | The whole image on the calling thread | Equal, byte for byte, to the concatenation of `render_rows` over any split into bands (RND-3) |
| `RenderStats` | Counts over the samples of a band or an image | Integer counts only, so no value is NaN |
| `samples()` | The number of samples, including outside samples | $W n^2$ times the number of rows of the band |
| `outside()` | The number of outside samples | At most `samples()` |
| `folds()` | The sum of the steps of the folds of every sample that is not an outside sample | Saturates at `u64::MAX` |
| `largest_fold()` | The largest number of steps of one fold | At most the fold cap |
| `unconverged()` | The number of unconverged samples | At most `samples() - outside()` |
| `merged` | The statistics of two bands together | Adds the counts with saturation and takes the larger `largest_fold()` |
| `fold` | Folds one point into the domain and calls `visit` with the index of each side that it applies, in order | The steps of Section 5.3, with the same decisions as the fold inside the renderer for the same point and cap (RND-7). Never panics (RND-14) |
| `Fold::point` | The folded point | Not renormalized (RND-8). Finite when `converged()` is true |
| `Fold::steps` | The number of steps | At most the cap |
| `Fold::reversals` | The number of reversals | At most `steps()` |
| `Fold::converged` | Whether the fold converged | False for a non-finite input, a NaN side value, a non-finite point after a step, and a fold that reached the cap |
| `encode_png` | Encodes an 8-bit RGB buffer as a PNG file with the sRGB chunk | In memory, so the crate reads and writes no file (RND-12) |
| `Error` | Why an operation failed | Section 7 |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

RND-2. The crate MUST NOT start a thread, because the caller chooses the parallelism and splits the image into bands of rows, as CONC-4 of the coding standard and Section 3.5 of the architecture lay out for the `rayon` of the binary (`DEC-21`), and a thread that the caller cannot see escapes its control of time and memory.

RND-3. The colour of each pixel MUST depend only on the domain, the options, and the position of the pixel, and MUST NOT depend on the band that contains it or on any other sample, because the caller renders bands in parallel in any order and expects the same file for every split, and the test "Bands" of Section 8.1 compares the bytes exactly. The crate therefore does not seed a fold from a neighbouring pixel, which Section 8 of the architecture defers.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, and the unit test "Auto traits" of Section 8.1.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/colour.rs`: `Colour`, `Fill`, and the sRGB transfer function.
- `src/options.rs`: `RenderOptions`, `RenderOptionsBuilder`, the defaults, and the limits.
- `src/fold.rs`: `Fold`, `fold`, the private fold that carries the screen tangents, and the tolerance constant.
- `src/view.rs`: The private view, which maps a sample to a disk position and a point of the linear model with its screen tangents, and the symmetric view of the unit tests.
- `src/renderer.rs`: `Renderer`, `RenderStats`, the border estimate, and the colour of a sample and of a pixel.
- `src/encode.rs`: `encode_png`.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, and Section 3.2 of the architecture lets the crate design refine the names and signatures:

- A public `View` with the model, the placement, and the sphere rotation. One view exists in this design, and SIMP-1 of the coding standard forbids an abstraction before a second case. A private view holds the placement isometry, which is the identity for the canonical view. The change that adds the first option that changes the view makes it public.
- A colouring identifier with a list of colours. `Fill` carries its colours until `escher-group` exists.
- The light of the sphere, which belongs to the spherical geometry.

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
| `fill` | `Fill` | None, given to `builder` | Any | None |

`build` checks the options in the order of the rows of this table and reports the first value that fails. A border width of 0 draws no border, and an unconverged sample still takes the border colour (Section 5.5).

The defaults for the border, the sample grid, the fold cap, and the background are those of `DEC-7`, which proposes an opaque white background and no `transparent` one. Section 11 records the figures behind the cap, and Section 12.3 asks the owner to confirm it.

### 4.2. Colours

RND-5. The crate MUST take every colour as 8-bit sRGB through `Colour::from_srgb8` and MUST NOT parse colour text, because this design puts the project's parser of colour text (`DEC-29`) in the command line (Section 4.6 of the command-line design), and a parser in the library would add a second place that defines what a colour string means.

`Fill` has no default, so the library states no default fill colour. Section 4.6 of the command-line design states the default colours of `escher render`.

## 5. Rendering a Sample

The crate computes the colour of a pixel in the order of Section 4.1 of the architecture. Sections 5.1 to 5.6 state each part, and Section 5.7 gives the source of each step.

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
2. If a coordinate of $x$ is not finite, stop as unconverged.
3. For each side $i$ in index order, compute $s_i =$ `hyperbolic::side(c_i, x)`. If $s_i$ is NaN, stop as unconverged. If $s_i > \tau \sigma_i x_3$, side $i$ is violated, so go to step 5 with the first violated side.
4. No side is violated. Stop as converged.
5. If the step count equals $N$, stop as unconverged.
6. Replace $x$ by $P_i x$, and inside the renderer replace $T_1$ by $P_i T_1$ and $T_2$ by $P_i T_2$. Add 1 to the step count, add 1 to the reversal count when the orientation of side $i$ is `Reversing`, and call `visit(i)`. Go to step 2.

The fold returns the vector $x$ as it stands at the stop, with the step count, the reversal count, and whether it converged. Step 2 runs before every side test, so a converged fold returns a finite point, and a cap of 0 returns an unconverged result with zero steps for every point outside the domain.

A point that violates no side at the start stops at step 4 with zero steps, and `Fold::point` returns its input unchanged, bit for bit, because no arithmetic touches it.

RND-8. The fold MUST NOT renormalize the point or the screen tangents, at any step and at the stop, because renormalization rounds the point by a factor that is noise near the limit circle and changes which sides the fold applies, and on the prototype it produced infinite coordinates (Section 11). The decisions of step 3 do not need it, because both sides of the comparison scale with $x$ (Section 6.1).

`hyperbolic::side` applies no tolerance (Section 4.2 of the geometry design), so the tolerance of step 3 belongs to this crate, as Section 9.2 of the architecture requires. Section 6.2 states the constant $\tau$.

The domain built by `escher-domain` has three sides of kind `Mirror`, each its own partner, so the fold applies a reflection at every step and the parity equals the step count modulo 2. The fold reads the orientation of each side rather than assuming a mirror, so that the classes of signature with rotations and glide reflections need no change to the loop.

### 5.4. Border

RND-9. The renderer MUST mark a converged sample as a border sample exactly when, for some side $i$ of the domain, the expression `!(f_i.abs() >= half_width * g_i)` is true, with $f_i$ the side value at the folded point, $g_i$ the gradient size of this section, and the half width in pixels, because this is the border estimate of `DEC-28`, and the negated comparison makes a NaN or other non-finite quantity give a border sample rather than a fill colour.

The gradient size of side $i$ is

$$g_i = \sqrt{(c_i \cdot T_1)^2 + (c_i \cdot T_2)^2}$$

with $T_1$ and $T_2$ the screen tangents after the fold. In words, $g_i$ is the change of the side value per pixel on the screen, so $\lvert f_i \rvert / g_i$ estimates the distance in pixels from the sample to the wall of side $i$ in the tile of the sample. The renderer evaluates the condition for every side of the domain. The estimate follows the gradient form of Quilez, `abs(v)/length(g)`, and the Jacobian form of Skia (Section 7.1 of the rendering reference). No reference gives it through a chain of folds in the linear model, which is `UNV-15` and item 19 of Section 13.1 of the rendering reference.

Scaling the folded point and both screen tangents by one factor scales $f_i$ and $g_i$ by that factor and leaves the condition unchanged, so a common scale of the unnormalized point and its tangents does not matter (RND-8). No transcendental function runs for a sample, so the platform mathematics library enters the image only through the domain.

The tests compare this estimate with a second form, the distance form, which only the reference arithmetic evaluates. With $\hat{x}$ the folded point divided by $\sqrt{-\langle x, x \rangle}$ and $\hat{f}_i$ the side value at $\hat{x}$, the distance form marks a border sample when

$$\operatorname{arsinh}\lvert \hat{f}_i \rvert \cdot \frac{1 - \lvert z \rvert^2}{2} < h \cdot w$$

for some side $i$, where $w$ is the half width and $z$ is the disk position of the sample. Its parts are the unit-normal form $\sinh d = \lvert \langle x, u \rangle \rvert$, which is part of `UNV-14` (item 3 of Section 15.1 of the geometry reference), and the disk metric of [CFKP], p. 71 (Section 6.2 of the rendering reference). Zucker converts a hyperbolic distance in the same way (Section 6.1 of the rendering reference). The test "Border forms" of Section 8.1 measures how often the two forms disagree.

### 5.5. Colour of a Sample

RND-10. The renderer MUST give each sample the colour of the first row of this table that applies, because an unconverged sample carries no reliable tile and must not take a fill colour, and the precedents of Section 4.9 of the rendering reference give such a sample a fixed colour:

| Case | Colour |
| --- | --- |
| Outside sample | The background |
| Unconverged | The border colour (`DEC-7`) |
| Border sample | The border colour |
| `Fill::One(c)` | $c$ |
| `Fill::Parity([c_0, c_1])` | $c_0$ for an even parity and $c_1$ for an odd parity |

The domain itself takes the first colour of `Fill::Parity`, because a point inside it folds in zero steps. The parity colouring rests on `UNV-18`, with the precedents of the flip parity of [ZUCKER] and [NS] and the swap of two colours of [SYMMHUB] (Section 4.6 of the rendering reference).

### 5.6. Pixel and Image

The colour of a pixel is the mean of the colours of its $n^2$ samples in linear light, each channel separately. The renderer then converts the mean to 8-bit sRGB with `Colour::to_srgb8`, which applies the encoding direction of the sRGB transfer function, clamps the result to the range 0 to 1, and rounds $255$ times it to the nearest integer. `Colour::from_srgb8` applies the decoding direction to each byte divided by 255.

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

## 6. Numerical Behaviour

### 6.1. Scale and Renormalization

The side test of step 3 of Section 5.3 compares $c_i \cdot x$ with $\tau \sigma_i x_3$, and both scale with $x$. The fold decides on the ray of $x$, not on its point of the hyperboloid, so it needs no renormalization. Multiplication by a power of two is exact in IEEE 754 arithmetic when no overflow or underflow occurs, so a point scaled by $2^k$ gives every product and sum of the side test and of the pairing scaled by exactly $2^k$, and the fold takes the same decisions. The test "Scale" of Section 8.1 asserts this exactly, and states that reason.

The point drifts off the hyperboloid during a long fold. The fold does not correct the drift, and Section 6.4 measures the folded point against the reference, which carries the same unnormalized ray, as a hyperbolic distance between the normalized points.

`DEC-31` and Section 9.3 of the architecture state that the fold renormalizes nothing, and that the renderer accumulates no matrix. The screen tangents replace the matrix that a border estimate would need (`DEC-28`), and RND-8 states the rule on renormalization. `DEC-31` is a proposed default, so Section 12.3 asks the owner to confirm it.

### 6.2. Tolerance of the Side Test

RND-13. The tolerance constant $\tau$ MUST be a positive private constant, and the side test MUST use $\tau \sigma_i x_3$ as its threshold, because a point exactly on a wall or at a vertex of the domain otherwise takes steps between walls that it lies on, and the threshold must scale with the point (Section 6.1).

$\tau$ is a small robustness constant for points on walls and at vertices. It is not a bound on the rounding error of the side test, and no constant $\tau$ bounds that error across the range of accuracy (Section 11). Points within $\tau$ of a wall, relative to $\sigma_i x_3$, count as inside that side.

The change that adds the tests sets the value of $\tau$ and records it here, with the measurement that justifies it. The test "Vertices" of Section 8.1 asserts that every vertex of the domain of each triple of its set folds in zero steps with that value. On the prototype, a value of $10^{-15}$ gave that result (Section 11).

### 6.3. Non-Finite Values

The crate turns every non-finite value into an unconverged fold or a border sample, never into a fill colour and never into a panic:

| Value | Where | Result |
| --- | --- | --- |
| A non-finite coordinate of the input or of the point after a step | Step 2 of Section 5.3 | Unconverged |
| A side value that is NaN | Step 3 of Section 5.3 | Unconverged |
| A side value that overflows to an infinity for a finite point | Step 3 of Section 5.3 | Compared as a number. A violated side leads to a step, and step 2 then checks the new point |
| A non-finite $f_i$ or $g_i$ | Section 5.4 | Border sample |
| A disk position that is NaN | `hyperbolic::from_disk` | Outside sample |

RND-14. `fold` and the renderer MUST NOT panic for any input, including any `f64` bit pattern of the point and any domain that `build` returns, because a panic stops the caller's whole program (ERR-5 of the coding standard), and the input of `fold` is arbitrary in the tests and near the limit circle.

`RenderStats` holds integers only, so no statistic is NaN. A caller computes a mean from `folds()` and `samples() - outside()`.

### 6.4. Range of Accuracy

The crate states its accuracy for disk positions up to the radius $1 - 10^{-6}$, the radius band 6. That radius corresponds to a hyperbolic distance of about 14.5 from the centre by the relation of [CFKP], Fact 5 (Section 4.7 of the rendering reference). Beyond it, the prototype measured that the fold in double precision applies a different side than a 256-bit fold, and that `from_disk` returns vectors that are not points from $1 - 10^{-8}$ outward, which never converge (Section 11).

RND-15. The renderer and `fold` MUST meet the bounds of the table of this section in each radius band from 1 to 6, and for a disk position beyond the range of accuracy MUST only return without a panic and report the fold as converged or unconverged, because the bounds are measured only up to that radius, and a promise beyond it would be false for many samples.

The fold calls `hyperbolic::side` and `hyperbolic::reflection` outside the ranges of their rows in Section 8.4 of the geometry design, which cover distance 10 for `side` and distance 3 for the points of `reflection`. This crate relies on its own end-to-end measurements, as Section 9.3 of the domain design does for its construction.

Each bound is to be measured by the change that adds its test (RND-16). The fold set of the table is the plan triples of Section 12.1 of the domain design and the triple $(1000, 1000, 1000)$, with disk positions at 512 evenly spaced angles in each radius band:

| Quantity | Measure | Measured against | Input set | Bound per radius band |
| --- | --- | --- | --- | --- |
| Folded point | The hyperbolic distance between the normalized folded point and the normalized result of the reference replay | The replay, in the reference arithmetic, of the side sequence of the fold on the exact `f64` input point and the exact `f64` pairings | The fold set | To be measured, one bound for each band and each triple |
| Fold decisions | The number of folds whose side sequence first differs from that of an independent 256-bit fold of the same input point | The 256-bit fold of Section 8.3 | The fold set | To be measured, one count for each band |
| Tangents | The largest relative difference of each tangent | Central differences of the reference `from_disk`, times $h$ | Disk positions with $1 - \lvert z \rvert$ from $10^{-6}$ to 1, log-uniform | To be measured |
| Border estimate | The number of samples that the gradient form and the distance form classify differently, at half widths 0.5 and 1.5 | The distance form in the reference | Samples within 4 pixels of a wall, in each radius band | To be measured, with the half width beyond which the forms differ |
| Colour | The largest difference in linear light from the reference transfer function, and the round trip of every byte | The transfer function in the reference | All 256 values in each direction | To be measured |
| Fold count | The mean and the largest number of steps | None | The fold set, and the samples of the canonical image of `*(1000)32` at 512 by 512 pixels with a 4 by 4 grid | To be measured. Every fold of the set converges within the default cap |

Beyond the range of accuracy, the test "Rim" of Section 8.1 asserts no bound. It asserts that each fold returns, that a converged fold has a finite point, and that the renderer gives each sample one of the colours of Section 5.5.

### 6.5. Measured Figures

RND-16. The change that adds a test of this document MUST measure each figure that this document marks as to be measured, set each tolerance to at least 8 times the largest error over the stated deterministic input set, and record in this document the input set, the tolerance, the largest error, and the margin, as TEST-13 of the coding standard requires, because a bound that no measurement of the code supports is a guess that a reviewer cannot check.

The figures of Section 11 come from the prototype. They justify the choices of this design and are not properties of the crate. No bound of Section 6.4 takes a figure from the prototype.

## 7. Errors

### 7.1. Structure

RND-17. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"build render options"`, `"render rows"`, or `"encode image"`.
- `detail`: A private enum with one variant for each cause, which carries the values that the text of Section 7.2 shows, or the error of `png` that caused an encoding failure.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, as Section 10.1 of the domain design states for the same reason: `source()` returns the `png` error that one variant of the detail holds. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `png` error for an `Encoding` error and `None` otherwise. No type from a dependency appears in the public surface of the error (ERR-2).

`Error` derives only `Debug`, because the `png` error that it may hold is not required to implement `Clone` or `PartialEq`. This is an assumption about `png` that the change that adds the crate checks (Section 12.1).

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
| `InvalidBuffer` | `render rows` | A buffer whose length is 0 or not a multiple of $3W$ | `the buffer of 1535 bytes does not hold whole rows of 1536 bytes`, with the two lengths |
| `InvalidBuffer` | `render rows` | A band that ends below the last row | `the rows 510 to 513 are outside the image of 512 rows`, with the first and last row of the band and the height |
| `InvalidBuffer` | `encode image` | A width or a height of 0, or a buffer whose length is not $3WH$ | `the buffer of 1535 bytes does not hold an image of 16x32 pixels`, with the length and the size |
| `Encoding` | `encode image` | `png` reports an error | `the PNG encoder failed` |

So `RenderOptions::builder(fill).samples(17).build()` returns an `Error` whose `kind()` is `LimitExceeded` and whose `Display` text is exactly `cannot build render options: the sample grid 17 is larger than 16`.

Every fixed text is in lower case apart from `PNG` and has no trailing punctuation (ERR-4 of the coding standard). The values in the text are numbers that the crate formats, so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

`fold`, `Renderer::new`, and `render` return no error. `fold` reports a failure as an unconverged result.

## 8. Testing

### 8.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-render/tests/render/main.rs`, with the modules `common`, `reference`, `folds`, `properties`, `borders`, `images`, and `encoding` (TEST-10). Its root has a `//!` line. The snapshot tests of the command and the process-level smoke test live in `escher-cli`, and Section 8.1 of the command-line design specifies them.

The module `common` defines the sets that the tests use:

- The plan triples of Section 12.1 of the domain design: $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(1000, 3, 2)$.
- The fold set of Section 6.4: the plan triples and $(1000, 1000, 1000)$, at 512 evenly spaced angles in each radius band from 1 to 6.
- The rim set: the plan triples and $(1000, 1000, 1000)$, at 64 evenly spaced angles at each radius $1 - 10^{-k}$ for $k$ from 7 to 15, and at the largest `f64` below 1 on each axis.

Each test names the item of the field "Tests and acceptance" of Section 5.3 of the plan that it serves. The reference tests print each largest value with its input and its margin, and assert the smallest number of samples that they measure:

| Test | Plan item | Kind and place | Asserts |
| --- | --- | --- | --- |
| Fold replay | Reference arithmetic | Reference test, `folds` | For the fold set, the folded point is within the bound of its band and triple of Section 6.4 of the reference replay of the same side sequence. This covers words of a few hundred reflections, and the longer words of $(1000, 3, 2)$ |
| Fold decisions | Reference arithmetic | Reference test, `folds` | For the fold set, the number of folds whose side sequence differs from the 256-bit fold is within the count of its band of Section 6.4 |
| Tangents | Reference arithmetic | Reference test, placed as the paragraph after this table states | The screen tangents of Section 5.2 agree with central differences of the reference `from_disk`, times $h$, within the bound of Section 6.4. This tests the unverified statement of Section 10 on the tangents |
| Border forms | Border oracle | Reference test, `borders` | For samples within 4 pixels of a wall in each band, the number of samples that the gradient form and the distance form classify differently is within the bound of Section 6.4 |
| Colour | None | Reference test, `encoding` | `from_srgb8` agrees with the reference transfer function within the bound of Section 6.4 for all 256 values, `to_srgb8(from_srgb8(v))` is `v` for every byte, and `BLACK` and `WHITE` equal their constructors |
| Reference self-check | None | Reference test, `reference` | The known values of Section 8.3 |
| Inside | Fold properties | `proptest`, `properties` | For a point drawn in the range of accuracy, a converged fold gives a point whose side value for every side is at most $\tau \sigma_i x_3$ |
| Idempotent | Fold properties | `proptest`, `properties` | Folding the folded point of a converged fold takes zero steps and returns the same bits. The comparison is exact, because no arithmetic touches a point that violates no side (Section 5.3) |
| Pairing | Fold properties | `proptest`, `properties` | For a point $x$ in the range of accuracy whose folded point lies, in the reference, at a hyperbolic distance of at least $10^{-6}$ from every wall of the domain, and a side $i$: the folds of $x$ and of $P_i x$ both converge, their folded points are within the bound of Section 6.4 for the band of $x$, their step counts differ by exactly 1, and their parities differ |
| Scale | Fold properties | `proptest`, `properties` | For a point in the range of accuracy and each $k$ from $-40$ to 40, the fold of $2^k x$ visits the same sides, takes the same steps, and has the same `converged()`, and its point is exactly $2^k$ times the folded point. The comparison is exact for the reason of Section 6.1 |
| Any input | Fold properties | `proptest`, `properties` | For any three `f64` bit patterns as the point and any cap up to 2000, `fold` returns, `steps()` is at most the cap, `reversals()` is at most `steps()`, and a converged result has a finite point |
| Vertices | Fold properties | Integration test, `folds` | Every vertex of the domain of each plan triple and of $(1000, 1000, 1000)$ folds converged with zero steps, with the value of $\tau$ of Section 6.2 |
| Wrong inputs | Failure paths | Integration and unit tests | The rows of Section 8.2 |
| Rim | Precision at the limit circle | Integration test, `folds` | For the rim set, `fold` returns without a panic, and a converged result has a finite point. A render of `*732` at 4096 by 4096 pixels with one sample, whose outermost samples lie beyond the range of accuracy, returns without a panic. No bound |
| Cap | Termination | Reference test, `folds` | Every fold of the fold set and every sample of the canonical image of `*(1000)32` at 512 by 512 pixels with a 4 by 4 grid within the range of accuracy converges within the default cap of 1500. It prints the mean and the largest step count of each band, which Section 6.4 records |
| Border oracle | Border oracle | Integration test, `images` | At one sample per pixel, for each plan triple at 192 by 192 pixels, every pixel within the radius that the test states where the tile of its sample differs from the tile of a neighbouring sample lies within one pixel of a border pixel, and every border pixel lies within one pixel of such a change. The tile of a sample is the image of the domain centre under the inverse of the pairings that `fold` reports, in reverse order. Two tiles are equal when their centres are closer than a distance that the test states |
| Bands | None | Integration test, `images` | `render` equals, byte for byte, the concatenation of `render_rows` over a split into bands of one row, of an uneven split, and of one band. The comparison is exact, because each sample is computed by the same instructions whatever the band (RND-3) |
| Symmetric view | Symmetric view | Unit test in `view.rs` | For each plan triple at 192 by 192 pixels, a 4 by 4 grid, and `Fill::One`, the image of the symmetric view equals its mirror image in the first axis, row $j$ against row $H - 1 - j$, within the allowance of Section 8.4 |
| Golden images | Golden images | Integration test, `images` | Section 8.4 |
| Perturbation | Golden images | Unit test in `renderer.rs` | The number of pixels that change when one unit in the last place is added to every component of every normal copied into the renderer, for the three golden images, which sets the allowance of Section 8.4 |
| PNG size | PNG decodes to the requested size | Integration test, `encoding` | `encode_png` of a 16 by 32 image decodes with `png` to 16 by 32 pixels, RGB, bit depth 8, with the sRGB chunk and the same bytes |
| Options | Failure paths | Unit tests in `options.rs` | Each row of Section 4.1 at its limits and one past them, with the kind and the text of Section 7.2, and the defaults |
| Statistics | None | Unit tests in `renderer.rs` | `merged` adds the counts, saturates at `u64::MAX`, and takes the larger `largest_fold()`. For a 4 by 4 image with one sample of `*732`, `samples()` is 16 and `outside()` counts the corners of the image outside the disk |
| Error text | Failure paths | Unit tests in `error.rs` | The `Display` of every row of Section 7.2, `kind()` for each cause, and `source()` only for `Encoding` |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs` | Each public type is `Send`, `Sync`, and `'static` (CONC-1 of the coding standard) |
| Doctests | None | Doctests | The examples of the crate-level and item documentation |

The screen tangents are private, so the test "Tangents" needs private access. It is either a unit test in `view.rs` that loads the reference module of the test binary by path, or an integration test in `borders` that reads the tangents through a function of the crate that is public only under a test-only Cargo feature. The second option adds a Cargo feature, so this design prefers the first. The change that adds the test records the choice here. Either way the reference shares no code with the crate (RND-19).

The symmetric view maps a point $x$ of the screen to $Q x$ before the fold, and the tangents to $Q T_1$ and $Q T_2$. $Q$ is the isometry that maps `hyperbolic::ORIGIN` to vertex 2 of the domain and the positive first axis to the geodesic ray from vertex 2 through vertex 0, built from `hyperbolic::translation`, `hyperbolic::inverse`, and `hyperbolic::rotation`. The wall of side 0 then lies on the first axis. The test uses `Fill::One`, because a mirror of the group swaps the two parity colours (Section 11). It uses an even image size, so that no sample lies on the first axis. The image is not expected to equal its mirror image bit for bit, because the wall of side 0 after $Q$ is not exactly the first axis in `f64`.

The property tests commit their regression files (TEST-9 of the coding standard).

### 8.2. Deliberately Wrong Inputs

| Input | Expected |
| --- | --- |
| A point with a NaN or infinite coordinate | `converged()` false, zero steps |
| A point on the other sheet of the hyperboloid, or a spacelike vector | No panic. Converged or unconverged, which the change that adds the test records here |
| A cap of 0 with a point outside the domain | Unconverged, zero steps |
| A cap one below the steps that a point needs | Unconverged, `steps()` equal to the cap |
| `(f64::MAX, 0, f64::MAX)` | Unconverged, because the point becomes infinite during the fold (Section 11) |
| A vertex of the domain | Converged with zero steps, because $\tau > 0$ (Section 6.2) |
| A buffer one byte short of a row, an empty buffer, and a band past the last row | `InvalidBuffer` with the text of Section 7.2 |
| `encode_png` with a size of 0 or a buffer of the wrong length | `InvalidBuffer` |
| A size of 0 by 512, a grid of 17, a width of 16385, a NaN border width, and a cap of 100001 | The kinds and texts of Section 7.2 |

### 8.3. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture:

- Structure. A module `reference` in the test binary holds points, vectors, and matrices as arrays of high-precision numbers and writes each formula componentwise from the research documents: the map from the disk to the hyperboloid, the side test, the reflection, the fold of Section 5.3, the distance form of Section 5.4, and the sRGB transfer function. It calls no function of `escher-render`, `escher-domain`, `escher-geometry`, or `glam`.
- Inputs. The reference takes the exact `f64` values of the inputs, and of the pairings and covectors of the domain, which it reads through the public accessors of `Side`.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 12.2 of the domain design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there.
- Self-check. The test "Reference self-check" checks the reference library on known values, as Section 12.2 of the domain design does. It does not close `UNV-25`.

RND-19. The reference module MUST share no code with `escher-render`, `escher-domain`, or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture, TEST-13 of the coding standard).

### 8.4. Golden Images

The golden images are `*732`, `*542`, and `*433` in the canonical view at 192 by 192 pixels, with a 4 by 4 grid, a border width of 1 pixel, `Fill::Parity` with two fixed colours that the test states, the default border colour, background, and cap. They are committed under `crates/escher-render/tests/render/golden/`, as Section 7.2 of the architecture proposes.

RND-20. The golden-image test MUST decode the committed file with `png`, convert both images to linear light with `Colour::from_srgb8`, and pass only when at most the allowed number of pixels differ and each differing pixel differs in each channel by at most one sample weight plus the linear-light size of one 8-bit step at the larger of the two stored values of that channel, because samples are averaged in linear light and stored as 8-bit sRGB, so a sample that changes colour moves the stored value by an amount that depends on the colours, and a limit in stored units would be too loose for dark values and too tight for bright ones.

The allowed number of pixels comes from the test "Perturbation" of Section 8.1, which adds one unit in the last place to every component of every normal. That perturbation is the size of a difference in the platform mathematics library, which the plan names as a risk. The change that adds the images records the measured number here, and the allowance for each image. On the prototype the perturbation changed no pixel (Section 11).

On failure, the test writes the actual image and the difference image under the directory that `CARGO_TARGET_TMPDIR` names. The golden files change only through a recipe and its environment variable, which the change that adds the images writes into the tooling design, with the new TEST rule of the coding standard, as Section 7.2 of the architecture requires.

### 8.5. Tests Under Miri

`DEC-5` and TEST-11 of the coding standard apply to the tests of this crate in this way:

| Tests | Under Miri | Attribute or configuration |
| --- | --- | --- |
| "Fold replay", "Fold decisions", "Tangents", "Border forms", "Colour", "Reference self-check", and "Cap" | Ignored | `#[cfg_attr(miri, ignore = "the high-precision arithmetic runs loops too long for the interpreter")]` |
| "Rim", "Border oracle", "Bands", "Symmetric view", "Golden images", and "Perturbation" | Ignored | `#[cfg_attr(miri, ignore = "rendering whole images runs loops too long for the interpreter")]`, and the golden images also read files |
| "Inside", "Idempotent", "Pairing", "Scale", and "Any input" | Run with 8 cases and no failure persistence | The helper of RND-21 |
| "Vertices", "Wrong inputs", "Options", "Statistics", "Error text", and "Auto traits" | Run unchanged | None |
| "PNG size" | Run unchanged | None. Encoding a small image through `png` passed Miri on the prototype (Section 11), and the change that adds the test confirms it |
| The doctests | Not run, because Miri runs the tests through nextest, which runs no doctest | None |

Each reason in an attribute is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

No test asserts a bit-exact result of a transcendental function. The exact comparisons of Section 8.1 involve no transcendental function, and each states why IEEE 754 arithmetic makes it exact.

RND-21. The property tests MUST take their configuration from one helper in `common`, with the numbers of cases and the failure persistence that Section 4 of `docs/design/tooling.md` states, because the file persistence of `proptest` asks for the current directory, which the isolation of Miri refuses, and a full set of cases takes too long in the interpreter.

## 9. Dependencies

RND-22. The crate MUST declare only the dependencies of this section, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml`, because each further dependency adds build time, licence obligations, and supply-chain risk, and a version declared twice can diverge (DEP-2 of the coding standard).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-domain` | Normal, with a version on the path dependency (TOOL-25 of `docs/design/tooling.md`) | None | The input type `FundamentalDomain` and its sides |
| `escher-geometry` | Normal, with a version on the path dependency (TOOL-25) | None | The side test and the disk map |
| `glam` | Normal, public | The workspace entry | `DVec3` in the signature of `fold` (`DEC-4`) |
| `thiserror` | Normal | Default | Derives the `Display` of the private detail of the error (Section 7.1) |
| `png` | Normal | The features that the change that adds it records | PNG encoding with the sRGB chunk (Section 5.6), and decoding in the tests |
| `escher-signature` | Dev | None | Builds the domains of the tests from signatures |
| `proptest` | Dev | The workspace entry | The property tests |
| `astro-float-num` | Dev | The workspace entry | The reference arithmetic (Section 8.3) |

`png` is the one new dependency of the workspace. Its newest resolution brings two versions of `miniz_oxide`, which fails the ban on duplicates of `cargo deny`. This design holds `flate2` at version 1.1.9 in the lock file, as Section 5.3 of the plan states, so that one version of `miniz_oxide` resolves. The prototype measured that this resolution passes the checks of bans and licences (Section 11). A plain `cargo update` resolves `flate2` to a newer version and brings the duplicate back, so the change that adds `png` states the command that keeps the pin in the tooling design. Section 12.3 keeps the alternative, a `skip` entry in `deny.toml`, as a point for the owner.

`palette` and `csscolorparser` are not dependencies of this crate, because `DEC-29` adds no colour crate in the first image milestone. The crate implements the sRGB transfer function itself (RND-11), and colour text belongs to the command line (RND-5). Section 3.1 of the architecture lists `escher-group` and `escher-signature` as internal dependencies of the crate. The crate declares neither as a normal dependency until a function uses it, as Section 3.3 of the architecture states, because an unused dependency fails `check-unused-deps`.

`glam` types appear in the public signatures of this crate, so DEP-3 of the coding standard makes `glam` a public dependency, with the obligations that Section 3.5 of the architecture states. No type of `png` appears in a public signature (Section 3.5 of the architecture).

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

## 10. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, or a statement of this design that no source establishes, and what the code does about it:

| Label | What the code relies on | What the code does |
| --- | --- | --- |
| `UNV-1` | The fold terminates for a compact reflection triangle, with a step count that grows toward the limit circle | Stops at the fold cap and reports the sample as unconverged. The test "Cap" measures the step counts, and the default cap comes from the prototype (Section 11) |
| `UNV-3` | The behaviour of a point on or near a wall | Uses the tolerance constant of Section 6.2. The tests "Vertices" and "Wrong inputs" cover the vertices of the domain, and the test "Pairing" keeps its points away from the walls |
| `UNV-4` | The growth of the fold count and the limits of precision near the limit circle | States accuracy only in the range of accuracy (RND-15). The tests "Fold replay", "Fold decisions", "Cap", and "Rim" measure each band |
| `UNV-14` | The unit-normal form $\sinh d = \lvert \langle x, u \rangle \rvert$, in the distance form of the tests only | Compares the gradient form with it in the test "Border forms". The code of the crate does not use it |
| `UNV-15` | The gradient form of the border estimate through a chain of folds in the linear model | Tests it against the distance form in the test "Border forms" and against tile identity in the test "Border oracle" |
| `UNV-18` | The parity colouring as the colouring by the two cosets of the orientation-preserving subgroup | Colours by the parity of the reversals. The test "Pairing" checks that a pairing changes the parity, and the golden images fix the result |
| `UNV-25` | The correct rounding of the reference crate, in tests only | Uses 256 bits and the self-check of Section 8.3 |
| This design | The screen tangents of Section 5.2 are the partial derivatives of the map of Section 7.2 of the geometry design, times $h$ | The test "Tangents" compares them with central differences of the reference map |
| Rendering reference | The constants of the sRGB transfer function, which rest on secondary sources until the standard is read (items 22 and 25 of Section 13.1 of the rendering reference) | Takes them from Section 7.5 of the rendering reference only (RND-11). The test "Colour" compares the crate with the reference transfer function |
| Rendering reference | Rounding to the nearest integer, after clamping to the range 0 to 1, when encoding an 8-bit sample (item 23 of Section 13.1 of the rendering reference) | Rounds as Section 5.6 states. The test "Colour" checks the round trip of every byte |

Averaging in linear light is a choice of the plan that no source supports (item 24 of Section 13.1 of the rendering reference). It states no fact, so it is a choice rather than an unverified statement.

## 11. Evidence Behind the Choices

This section is non-normative. It records figures measured on the prototype, which no measurement of the code covers. Each supports only the choice that it names, and none is a property of the crate.

The prototype implemented Sections 5.1 to 5.5 in Rust on Linux x86-64 with a 256-bit reference. Unless a row states otherwise, it used a margin of $1/32$, $\tau = 0$, a 512 by 512 view with a 4 by 4 sample grid where it rendered an image, and the six triples $(7, 3, 2)$, $(5, 4, 2)$, $(4, 3, 3)$, $(12, 3, 2)$, $(1000, 3, 2)$, and $(1000, 1000, 1000)$:

| Choice | Section | Input set | Figures of the prototype |
| --- | --- | --- | --- |
| No renormalization in the fold | 5.3, 6.1 | 2048 angles in each radius band, with $K$ of 1, 4, 16, and 64 | Renormalizing every $K$ steps changed the side sequence of folds against no renormalization. At $1 - 10^{-6}$: 7, 6, 3, and 0 folds of `*(1000)32` for $K$ of 1, 4, 16, and 64. At $1 - 10^{-7}$: 131, 46, 1, and 0 folds of `*732`, and 378, 284, 118, and 21 of `*(1000)32`. From $1 - 10^{-8}$ outward, hundreds per band, with the parity changed in most. `hyperbolic::normalize_point` returned an error up to 5.5 million times per band, and with $K$ at most 16 some folds returned infinite coordinates |
| Decisions on the ray | 6.1 | The folds of the row above | Scaling the point by $2^k$ for $k$ from $-40$ to 40 changed no side sequence |
| A converged fold has a finite point | 5.3, 6.3 | The input $(f_{\max}, 0, f_{\max})$, with $f_{\max}$ the largest finite `f64` and $\tau = 10^{-12}$, for `*732` and `*(1000)32`. Also 12 million fuzzed inputs | Without the check of step 2 after each step, the input returned converged after 1 step with an infinite last coordinate. NaN never returned converged in the fuzzed inputs |
| A positive $\tau$, not a bound on error | 6.2 | The vertices of the domains of the six triples, with $\tau$ of 0, $10^{-15}$, and $10^{-12}$, and the side tests of folds at 512 angles in each radius band | With $\tau = 0$, the vertices of the domains took 1 or 2 steps, and vertex 2 of $(1000, 1000, 1000)$ took 1935. With $\tau$ of $10^{-15}$ or $10^{-12}$, every vertex of a domain took zero steps. The ratio of the error of the side value to $\sigma_i x_3$ reached $1.5 \times 10^{-3}$ for `*732` and $5.0 \times 10^{-3}$ for `*(1000)32` at $1 - 10^{-6}$, and grew about 100 times per decade of $1 - \lvert z \rvert$ |
| A default cap of 1500 | 4.1 | A sweep of the radius bands, and the samples of the 512 by 512 image of each triple | The largest step count of the sweep was 1214, for `*(1000)32` beyond $1 - 10^{-7}$. A corner of order 1000 needed 1000 steps. The smallest cap at which every sample of the image converged was 89 for `*732`, 1000 for `*(1000)32`, and 998 for $(1000, 1000, 1000)$. At a cap of 500, 7956 of 2,895,324 samples inside the disk of `*(1000)32` did not converge, and no pixel differed from a cap of $10^6$ at border widths of 0.1, 0.3, 0.5, and 1 pixel. At a border width of 0, 541 pixels differed |
| The range of accuracy ends at $1 - 10^{-6}$ | 6.4 | 512 angles in each radius band from $1 - 10^{-1}$ to $1 - 10^{-12}$ | The largest distance of the folded point from the 256-bit replay grew from between $1.1 \times 10^{-13}$ and $1.3 \times 10^{-12}$ at $1 - 10^{-1}$ to $1.6 \times 10^{-3}$ for `*732`, $5.0 \times 10^{-3}$ for `*(1000)32`, and $1.3 \times 10^{-2}$ for $(1000, 1000, 1000)$ at $1 - 10^{-6}$, and was 0.06 to 0.7 at $1 - 10^{-7}$. The first side that differed from a 256-bit fold appeared in up to 8 of 512 folds per triple at $1 - 10^{-6}$ and in 24 to 265 at $1 - 10^{-7}$. `from_disk` returned a point for every position up to $1 - 10^{-7}$, and a vector that is not a point for 141 to 452 of 512 positions per band from $1 - 10^{-8}$ to $1 - 10^{-12}$. Such samples did not converge at a cap of $10^6$ |
| The gradient form of the border | 5.4 | 600 samples in each radius band, at half widths of 0.5 and 1.5 pixels | Both forms agreed with the exact screen distance to the walls of the tile within 0.3 percent beyond 100 pixels from the rim of `*732`, within 4 percent at 10 pixels, and within 5 percent within 2 pixels, and within 10 percent within 5 pixels for `*(1000)32`. The two forms differed by less than 2 percent. At most 10 of 600 samples per band were misclassified. The gradient form with the tangent formulas of Section 5.2 matched the exact distance to $10^{-3}$ near the centre |
| The pairing property | 8.1 | $x$ and its reflection in a random side, with a band of $10^{-6}$ around the walls excluded, 20000 cases for each of `*732` and $(1000, 1000, 1000)$ | Step counts differed by $+1$ in 10947 and by $-1$ in 9053 cases for `*732`, and in 14272 and 5728 for $(1000, 1000, 1000)$, with no other difference and the parity changed in every case |
| The parity colouring is consistent | 5.5 | Tiles centred within disk radius 0.999, about 41,000 to 200,000 points per triple | No inconsistency within a tile, and no two adjacent tiles with the same parity |
| One fill in the symmetric view | 8.1 | The images of `*732`, `*542`, and `*433` at 192 by 192 | With `Fill::Parity`, 20200 to 22712 of 36864 pixels differed from the mirror image, and every single-fill pixel pair matched once the two fill colours were swapped. With an odd image size and an odd grid, such as 191 and 3, a row of samples lay on the axis |
| The golden-image allowance | 8.4 | The images of `*732`, `*542`, and `*433` at 192 by 192, with a 4 by 4 grid and a 1-pixel border | Adding one unit in the last place to every component of every normal changed no pixel. A rotation of $10^{-6}$ changed 4, 9, and 9 pixels, by up to one sample, and a rotation of $10^{-3}$ changed more than 5000 |
| `png` with `flate2` at 1.1.9 | 9 | The dependencies of `png`, `rayon`, and a reduced `csscolorparser` in a project outside the repository | The newest resolution failed the duplicate check with `miniz_oxide` 0.8.9 and 0.9.1. With `flate2` at 1.1.9, the checks of bans and licences passed, and encoding and decoding a 4 by 4 image passed Miri |

## 12. Assumptions, Choices, and Points for the Owner

### 12.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- The screen tangents of Section 5.2 are the partial derivatives of the map from the disk to the hyperboloid, times $h$. The test "Tangents" checks them (Section 10).
- Applying a pairing to the screen tangents gives the screen tangents of the folded point, because a pairing is linear on the vectors of the linear model. The tests "Border forms" and "Border oracle" check the result through the border estimate.
- The tile of a sample is identified by the image of the domain centre under the inverse of the pairings that the fold applied. The test "Border oracle" relies on it and checks it together with the border estimate.
- The symmetric view has a mirror of the group in the first axis, because the wall of side 0 passes through vertex 2 and vertex 0. The test "Symmetric view" checks it.
- The `png` error type does not implement `Clone` and `PartialEq`, so `Error` derives only `Debug`. The change that adds `png` checks it.
- The sRGB transfer function maps the bytes 0 and 255 to exactly 0 and 1 in linear light, so `Colour::BLACK` and `Colour::WHITE` can be constants. The test "Colour" checks it.
- This design does not record whether a dependency of `png` has a build script. The change that adds `png` records it in Section 13.

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
| A fold is unconverged unless its point is finite when it stops, and a NaN side value, a non-finite input, and the cap make it unconverged | 5.3, 6.3 |
| A positive tolerance constant relative to $\sigma_i x_3$, as a robustness constant and not a bound on error | 6.2 |
| A non-finite border quantity makes a border sample | 5.4, 6.3 |
| The perceptual rendering intent of the sRGB chunk | 5.6 |
| The sRGB chunk without a gAMA or cHRM chunk | 5.6 |
| The range of accuracy ends at $1 - 10^{-6}$ | 6.4 |
| The error texts of Section 7.2 | 7.2 |
| Golden images compared in linear light, with the allowance set from a perturbation of one unit in the last place of every normal component | 8.4 |
| The symmetric view with one fill colour and an even image size | 8.1 |
| The pairing property with step counts that differ by exactly 1 and different parities | 8.1 |
| `png` as the one new dependency, with `flate2` held at 1.1.9 in the lock file | 9 |

### 12.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Renormalization. `DEC-31` and Section 9.3 of the architecture propose that the fold renormalizes nothing. This design follows it on the evidence of Section 11, and keeps no accumulated matrix (Section 6.1).
2. Unconverged samples. A fold that reaches the cap, meets a NaN side value, or meets a non-finite point is unconverged, and the sample takes the border colour (Section 5.5).
3. The tolerance constant. It is a positive robustness constant whose value the change that adds the tests sets (Section 6.2).
4. The default fold cap of 1500 and the largest cap of 100000 (Section 4.1). A cap of 500 changes no pixel of the canonical view of the plan triples at border widths of at least 0.1 pixel on the prototype, but leaves samples near a corner of order 1000 unconverged (Section 11). The alternatives are a cap of 500, or a default derived from the largest corner order of the signature.
5. The appearance near a corner of high order. With a border of 1 pixel, `*(1000)32` shows a solid region of border colour about 110 pixels wide around its corner of order 1000 on the prototype at 512 by 512 pixels. It follows from the definition of a border width in pixels and does not depend on the cap.
6. The shape of the library. It has no public `View` or model and a public `fold` (Section 3.4), starts no thread (RND-2), and parses no colour text (RND-5).
7. The `flate2` pin of Section 9. A pin in the lock file is lost on a plain `cargo update`. A `skip` entry for `miniz_oxide` 0.8 in `deny.toml` is durable but is an exception to the policy of Section 8 of `docs/design/tooling.md`.
8. Golden images and tests. The images are compared in linear light, with an allowance from a perturbation of one unit in the last place of every normal component (Section 8.4). The symmetric view uses one fill colour. The pairing property asserts that the folded points agree within the bound of Section 6.4, that the step counts differ by exactly 1, up or down, and that the parities differ (Section 8.1).
9. The range of accuracy ends at $1 - 10^{-6}$, the radius that the plan names, and beyond it the crate promises only a result without a panic (Section 6.4).
10. The margin of $1/32$, the perceptual rendering intent, the limits of Section 4.1, and the error texts of Section 7.2 are constants with no source. The PNG file has no gAMA or cHRM chunk beside the sRGB chunk, against the recommendation of the PNG specification (Section 5.6).

## 13. Security Considerations

The threat model is a caller that passes options and a domain derived from untrusted text to the crate, such as `escher-cli` with the arguments of a user. `escher-signature` has bounded the signature, and `escher-domain` has rejected every signature without a domain and every corner order above 1000 (Section 16 of the domain design). The attacker can choose any domain that `build` returns and any value of each option. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of memory through a large image, exhaustion of time through a large image, a fine sample grid, and a high fold cap, a panic that stops the caller, and an image that the caller trusts as correct although it is wrong. The crate counters them with the limits of this section, which RND-4 requires, the rule that it never panics (RND-14), and the tests of Section 8:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Width and height | 16384 pixels each | A choice of this design. The pixel limit bounds the memory |
| Pixels | $2^{26}$ | The buffer of `render` holds 3 bytes per pixel, about 201 MB at this limit. A choice of this design |
| Sample grid | 16 by 16 | 256 samples per pixel, far beyond the 16 of [NS]. A choice of this design |
| Border width | 1024 pixels | The largest image side divided by 16. A choice of this design |
| Fold cap | 100000 | About 80 times the largest step count that the prototype measured. The time of an unconverged sample is proportional to the cap. A choice of this design |

The limits bound memory. They do not bound time usefully when every option is at its limit together: $2^{26}$ pixels, 256 samples per pixel, and a cap of 100000 allow about $1.7 \times 10^{15}$ steps for an image whose samples do not converge. This is a residual risk. A caller that renders untrusted input with a deadline applies its own limits on top of these.

The crate allocates the buffer of `render` and nothing that depends on the domain beyond its three sides. `render_rows` writes only into the buffer of the caller.

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library. `png` forbids unsafe code in its own source. The crates `crc32fast` and `simd-adler32`, on which it depends, hold architecture-specific code behind runtime detection.
- Untrusted input and file paths: the options, the domain, the points of `fold`, and the buffers are the inputs. The crate reads and writes no file, keeps no state between calls, and returns the PNG file as bytes. Section 4.7 of the command-line design states how the file is written.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only. Section 12.1 records that the build scripts of the dependencies of `png` are not recorded yet.
- Dependencies: `Cargo.lock` fixes every version, including `flate2` at 1.1.9, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate neither builds nor writes anything outside the memory that the caller gives it.

The residual risks are three. First, the limits do not bound time, as stated above. Second, an image is wrong where the fold decides wrongly: beyond the range of accuracy the crate states no accuracy, and within it the bounds of Section 6.4 allow a stated number of differing decisions per band. Third, the border estimate and the parity colouring rest on unverified statements (Section 10), which the tests check numerically but no source establishes.

## 14. Operational Considerations

The crate declares no Cargo feature and has no build script. The golden-image test reads `CARGO_TARGET_TMPDIR`, which Cargo sets for integration tests, and the environment variable that updates the golden files, which the tooling design names. The library reads no environment variable. It is pure Rust with no platform-specific code of its own, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log. `RenderStats` is the measurement that the caller reports.

| Failure | Reported as | Recovery |
| --- | --- | --- |
| An option outside its accepted values | `InvalidOption` | Use a value in the column "Accepted" of Section 4.1 |
| An option above its limit | `LimitExceeded` | Use a value within the limit |
| A buffer or band that does not fit the image | `InvalidBuffer` | None. It is a defect of the caller |
| `png` fails to encode | `Encoding`, with the `png` error as `source()` | None. It is a defect of the crate or of `png` |
| A sample that does not converge | `Fold::converged` is false, the sample takes the border colour, and `RenderStats::unconverged` counts it | Raise the fold cap, or accept the border colour near the limit circle |
| A sample outside the disk | The background colour, counted by `RenderStats::outside` | None needed |

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
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [SYMMHUB] Section 14 of `docs/research/rendering.md`.
- [VGRG] Section 14 of `docs/research/rendering.md`.
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
| RND-13 | The tolerance constant is positive and the threshold is $\tau \sigma_i x_3$ | 6.2 |
| RND-14 | `fold` and the renderer never panic | 6.3 |
| RND-15 | The bounds of Section 6.4 hold in each radius band, and beyond the range of accuracy only a result without a panic is promised | 6.4 |
| RND-16 | The change that adds a test measures and records each figure to be measured, as TEST-13 requires | 6.5 |
| RND-17 | `Error` follows ERR-1, with the operation that failed | 7.1 |
| RND-18 | The `Display` text of each error is the one stated | 7.2 |
| RND-19 | The reference module shares no code with the crates under test | 8.3 |
| RND-20 | Golden images are compared in linear light with the allowance of Section 8.4 | 8.4 |
| RND-21 | Property tests take their configuration from one helper that is Miri-aware | 8.5 |
| RND-22 | The crate declares only the dependencies of Section 9, each from the workspace table | 9 |
