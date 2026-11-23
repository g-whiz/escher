# Render Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-23 |
| Applies to | `crates/escher-render` |

## Abstract

This document specifies the crate `escher-render`, which turns the fundamental domain of a compact hyperbolic reflection triangle or reflection polygon into the pixels of an image in the Poincare disk, the upper half-plane, the band, or the Klein disk, the fundamental domain of a Euclidean reflection triangle or rectangle into the pixels of an image of the Euclidean plane, and the fundamental domain of a spherical reflection triangle or lune into the pixels of an image of a lit ball, and encodes them as a PNG file in memory. It renders through the same steps the domain of a rotation subgroup, whose sides are paired by rotations and translations, the domains of the remaining Euclidean groups, whose sides are mirrors or are paired by translations, rotations, and glide reflections, the domains of the remaining spherical groups, whose sides are mirrors or are paired by rotations and rotary reflections, and the domain of the trivial group, which has no side. It fills the tiles with one colour, with two colours by parity, or by the cosets of a colouring of `escher-group`, which it reads from the sides that the fold applies. For every sample of a regular grid in each pixel, the crate maps the sample to a point of the linear model, folds the point into the domain, decides whether the sample lies on a border of a tile, whose width is in pixels or in the metric of the geometry, chooses a fill, border, or background colour, and on the ball shades that colour. It then averages the samples of each pixel in linear light and stores the pixel as 8-bit sRGB. For a colouring, it draws the polygon mask of each colour, which shows the domain as a flat polygon with straight edges and a colour-coded border on each edge, and it fills the tiles of a colour with an image decoded from a PNG stream, sampled through a fixed map from the domain onto that flat polygon, so that copies of the masks laid edge to edge show the seams of the tiling. On a reflection triangle, it can instead draw the faces of a Wythoff construction from on or off marks on the three mirrors, with one solid colour for each face type and borders on the edges of the faces. For the faces of a regular tiling, it can put one picture, painted on a flat regular polygon, on every face, turned or mirrored as an arrangement of the faces says, and it draws the face mask on which the user paints that picture. The document defines the public types and operations, the steps of the fold and of the border estimate with the source or the unverified status of each formula, the behaviour of the fold for points on walls, near the limit circle, and with non-finite coordinates, the range over which the crate states its accuracy, the error type, the limits on the options, the test layers with the golden images and the reference arithmetic, and the dependencies. It also lists the unverified statements that the code relies on, the assumptions, the choices, and the points that the owner decides.

## 1. Introduction

### 1.1. Problem

The proof of concept draws a tiling by pulling each sample back into the fundamental domain, one sample at a time (Section 4.1 of the architecture). The research gives the loop that folds a point into the domain and the theorem that makes the triangle a strict fundamental domain, but no bound on the number of steps, no behaviour for a point on a wall, and no form of a border width in screen pixels through a chain of folds in the linear model (Sections 4.7, 6, 7, and 13.1 of the rendering reference). Floating-point evaluation adds a second risk: near the limit circle the map from the disk to the hyperboloid loses digits, and the fold then applies a different side than exact arithmetic would. A wrong decision produces an image that looks plausible, so the design states where the crate is accurate, what it does beyond that range, and how the tests measure both.

### 1.2. Purpose

`escher-render` is the one place where a `FundamentalDomain` becomes pixels. It exposes the fold as a public function, a renderer that fills a band of rows in a buffer that the caller owns, statistics of the folds, a colour type, PNG encoding, the polygon masks of a colouring, and image fills decoded from PNG. Section 3 of the architecture places the crate in the workspace. It depends on `escher-geometry` for the linear model, the disk map, and the maps of the other models to the disk, on `escher-domain` for the domain, on `escher-signature` for the geometry of the domain, and on `escher-group` for the colourings of the coset fill, and `escher-cli` consumes it through the command `escher render`.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the author of the command-line design, which specifies the command `render` on top of this crate, and the authors of the later change that adds analytic antialiasing.

### 1.4. Scope

This document covers the public surface of the crate, the options with their defaults and limits, the map from a sample to the disk and to the hyperboloid, through the half-plane, the band, or the Klein model when the options name one (Section 5.16), to the Euclidean plane, and to the ball, the viewing rotation and the shading of the ball, the fold, the border estimate, the intrinsic border (Section 5.17), the colour of a sample by one colour, by parity, and by cosets (Section 5.12), the colour of a pixel, PNG encoding, the mask polygon of a domain (Section 5.22), the mask map (Section 5.23), the polygon masks (Section 5.19), the image fills and their sampling (Section 5.20), the decoding of an image with its limits (Section 5.21), the Wythoff fill with its faces, borders, and colours (Sections 5.24 to 5.27), the whole-face fill with its edge pairing, the picture of a sample, and the face mask (Sections 5.29 to 5.32), the numerical behaviour, the error type, the tests, and the dependencies.

The crate renders every domain that `FundamentalDomain::build` and `FundamentalDomain::build_with` return. These are the compact hyperbolic `*pqr` with corner orders up to 100, and the compact hyperbolic reflection polygons `*p1...pn` with 4 to 8 corners of orders up to 100 at any shape that `build_with` accepts (Section 5.13), in the canonical view of the Poincare disk or of the model that the options name (Section 5.16), and the Euclidean `*632`, `*442`, `*333`, and `*2222`, the last with any value of its shape parameter, in the canonical view of the Euclidean plane (Section 5.8), and the spherical `*332`, `*432`, `*532`, `*n22`, and `*nn`, with $n$ up to 100, as a lit ball (Section 5.9). It also renders the rotation domains of `pqr`, `p1...pn` with 4 to 8 orders, `632`, `442`, `333`, `2222`, `332`, `432`, `532`, `n22`, and `nn`, with the same limits, in the view of their geometry (Sections 5.11 and 5.13). It renders the wallpaper domains of `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, and of `2222` with any value of `skew`, in the canonical view of the Euclidean plane (Section 5.14). It renders the spherical group domains of `3*2`, of `2*n` with $n$ up to 50, of `n*` and `nx` with $n$ up to 100, and of `*`, `x`, and `1`, as a lit ball (Section 5.15). Section 4 of the domain design lists them.

### 1.5. Non-Goals

These topics are owned elsewhere or come later:

- The command `escher render` is specified in the command-line design: its options and the parsing of colour text (Section 4.6), the parallel split of the image into bands of rows (Section 4.5), the output file and the statistics text (Section 4.7), and the diagnostics (Section 6.8).
- The bilinear form, the side test, the isometries, and the disk map, with their formulas and accuracy, belong to `escher-geometry` and are specified in the geometry design. This document calls those functions by name and restates none of their formulas.
- The construction, placement, and verification of the domain belong to `escher-domain` and are specified in the domain design.
- The distance from a point to a wall, with its formula and accuracy, belongs to `escher-geometry` (Sections 5.6, 8.4 to 8.6, and 12 of the geometry design).
- A public `View` is not part of this design (Section 3.4).
- The maps from the half-plane, the band, and the Klein model to the disk, with their formulas, Jacobians, and accuracy, belong to `escher-geometry` (Sections 7.4 to 7.7 and 8.7 of the geometry design). Other models, such as the star model, are roadmap items.
- Analytic antialiasing and the fade of tiles smaller than a pixel come with the roadmap item `M14` (Section 4.1 of `docs/design/roadmap.md`).
- The search of the colourings, their ids, their limits, and `colour_of`, the map from a side sequence to a colour, belong to `escher-group` and are specified in the group design (GRP-29 to GRP-39). The resolution of a colouring id and the default fill colours belong to the command line (Section 4.6 of the command-line design). This crate takes a `Colouring` and its colours and parses no id.
- A generated palette and names of colours are outside this design.
- The shape parameters of a domain, their names, ranges, and defaults, belong to `escher-domain` and are specified in the domain design.
- A transparent background, the back hemisphere of the ball, a perspective projection, a second light, and options for the material of the ball are outside this design.
- The command `escher render-masks`, the names of the mask files, the form `image:FILE` of `--colours`, and the opening of an image file belong to the command line (Sections 4.6 and 4.8 of the command-line design). This crate reads an image from a reader that the caller gives and opens no file.
- The external tools with which a user paints over a mask are outside this design, as the owner decided (`DEC-66`).
- The option `--wythoff`, the syntax of the marks, the default colours of the face types, and the refusal of `--wythoff` with `--colouring` belong to the command line (Section 4.6 of the command-line design). This crate takes the marks as three `bool` values.
- Group colourings of the faces of a Wythoff construction, snubs, a movable generator point, and constructions on domains other than a reflection triangle are outside this design (`DEC-68`, Section 4.2 of `docs/design/roadmap.md`).
- An image fill of each face type of a Wythoff construction through the mask map of the domain is outside this design. Whole pictures on the faces of a regular tiling are the whole-face fill of Sections 5.29 to 5.32 (`DEC-110`).
- The search of the arrangements belongs to `escher-group` (GRP-45 of the group design), and the option `--arrangement`, the resolution of an arrangement id, and the name of the face mask file belong to the command line (Sections 4.6 and 4.8 of the command-line design). This crate takes an arrangement as a `Colouring` and runs no search.
- Whole-face fills with several pictures of one face, and whole-face fills of constructions with several face types, are milestones `M25` and `M26` of the plan (`DEC-114`, `DEC-121`). This design refuses both (Section 5.29).

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
| `docs/design/group.md` | the group design |
| `docs/research/rendering.md` | the rendering reference |
| `docs/research/colourings.md` | the colourings reference |
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
- half width: Half the border width, in the border unit.
- border unit: The unit of the border width, `BorderUnit::Pixels` or `BorderUnit::Intrinsic` (Section 5.17).
- intrinsic unit: The unit of length in the geometry of the domain: the metric of curvature $-1$, the Euclidean unit of `DEC-17`, or the radian of the unit sphere (Section 5.5 of the architecture).
- pixel scale: The number of pixels per intrinsic unit at a pixel, at its smallest within 4 pixels of that pixel, which the test "Wythoff borders" reads (Section 8.1).
- wall distance: The distance $d_i$ in the intrinsic unit from the folded point to the wall of side $i$, which a function `wall_distance` of `escher-geometry` gives (Section 5.17).
- step: One application of a pairing isometry by the fold.
- reversal: A step whose pairing reverses orientation.
- parity: The number of reversals of a fold modulo 2.
- converged: The result of a fold that stopped at a finite point with a positive point size that violates no side (Section 5.3).
- unconverged: The result of every other fold, which reached the fold cap, met a coordinate that is not finite, met a point size that is not positive, or met a side value that is NaN.
- border sample: A converged sample that the border estimate of Section 5.4, or with the intrinsic unit the test of Section 5.17, places on a border.
- outside sample: A sample of a hyperbolic domain whose model position the map of its model rejects (Section 5.16), or whose disk position `hyperbolic::from_disk` rejects, or a sample of the view of a spherical domain that lies on or outside the rim of the ball (Section 5.9).
- hyperbolic model: The model in which the renderer draws a hyperbolic domain, one of the variants of `HyperbolicModel`: the Poincare disk, the upper half-plane, the band, or the Klein disk (Section 5.16).
- model position: The position $w$ of a sample in the plane of a hyperbolic model other than the disk (Section 5.16).
- model Jacobian: The Jacobian that a function `disk_from_<model>` of `escher-geometry` returns with the disk position of a model position (Section 7.4 of the geometry design).
- tolerance constant: The constant $\tau$ of the side test of the fold (Section 6.2).
- radius band $k$: The disk positions $z$ with $\lvert z \rvert = 1 - 10^{-k}$, for an integer $k$ from 1 to 6.
- range of accuracy: For a hyperbolic domain, the disk positions with $\lvert z \rvert \le 1 - 10^{-6}$ (Section 6.4), and in another hyperbolic model the samples whose disk position lies there (Section 6.12). For a Euclidean domain, the plane positions within distance 65 of the centre of the image (Section 6.6). For a spherical domain, every vector whose largest absolute coordinate is at least $10^{-300}$ and whose fold converges, which includes the point of every ball position inside the rim whose fold converges (Section 6.1). For the border estimate, the range is the rim bands that Section 6.7 states. A rotation domain has the range of its geometry, except near some vertices of a hyperbolic rotation domain whose corner orders are all near 100, which RND-35 of Section 6.8 excludes. A reflection polygon and its double have a range of accuracy only at the shapes of the polygon domain set (RND-42 of Section 6.9). A wallpaper domain has the plane positions within distance 65 of the centre of the image as its range of accuracy only at the settings of the wallpaper domain set (RND-45 of Section 6.10).
- linear light: Colour values proportional to light intensity, in which the crate averages samples.
- 8-bit sRGB: Colour values encoded by the sRGB transfer function and stored as one byte per channel.
- band: A run of whole rows of the image, which `render_rows` fills.
- canonical view: For a hyperbolic domain, the view of the whole disk with the centre of the domain at the centre of the image, or in another hyperbolic model the view of Section 5.16 (`DEC-76`). For a Euclidean domain, the view of Section 5.8. For a spherical domain, the ball of Section 5.9 with the default viewing rotation. A rotation domain has the canonical view of its starred signature (`DEC-12`, Section 5.11). A wallpaper domain has the view of Section 5.8, centred on the centre of its rectangle, parallelogram, or generating triangle (`DEC-12`, Section 5.14).
- symmetric view: A view available only to the unit tests, which places vertex 2 of the domain at the centre of the image and vertex 0 on the positive first axis (Section 8.1).
- periodic view: A view available only to the unit tests, which is the view of Section 5.8 with another plane scale (Section 8.1).
- rotated view: A view available only to the unit tests, which is the view of Section 5.9 with any orthogonal matrix in place of the viewing matrix (Section 8.1).
- allowance: The two limits of a golden-image comparison, the largest difference of a pixel and the number of pixels that may differ (Section 8.4).
- count bound: The bound on a count that a test measures as $c$, which is the larger of $8c$ and 8 (RND-16).
- Euclidean domain set: The domains of `*632`, `*442`, and `*333`, and of `*2222` with `aspect` 1, 1.618, 0.125, and 8.
- spherical domain set: The domains of `*332`, `*432`, and `*532`, and of `*n22` and `*nn` for $n$ of 2, 3, 5, 12, and 100. `*222` is `*n22` with $n = 2$, and `*22` is `*nn` with $n = 2$.
- shading constants: The ambient weight $k_a$, the diffuse weight $k_d$, the specular weight $k_s$, and the specular exponent $\alpha$ of Section 5.10.
- reflection domain: A domain whose every side has the kind `Mirror`, such as the domain of `*732`.
- rotation domain: A domain that `escher-domain` builds for a signature without reflections, such as `732`, as the double of the reflection domain of its starred signature. No side of it has the kind `Mirror` (Section 5.11). `o` and `xx` are not built as doubles, and `2222` with a skew other than 0 is a sheared double. All three are wallpaper domains.
- starred signature: The signature of the reflection domain from which the domain design builds a rotation domain, such as `*732` for `732`, `*2222` for `2222`, and `*nn` for `nn`.
- doubled wall: The wall of side 1 of the reflection domain, across which the domain design doubles it. It lies inside the rotation domain and is not a wall of any of its sides.
- rotation domain set: The rotation domains of the signatures of the fold set of Section 6.4 without the star, `732`, `542`, `444`, `433`, `(12)32`, `(100)32`, and `(100)(100)(100)`, of `632`, `442`, `333`, and `2222` with the values of `aspect` of the Euclidean domain set, and of the signatures of the spherical domain set without the star.
- `Colouring`: The type of `escher-group` for one colouring, which holds an id, a colour count $k$, and a coset table of $k$ rows with one generator for each side of the side presentation (Sections 3.1 and 3.2 of the group design).
- coset fill: The fill `Fill::Cosets`, which gives each tile the colour of its coset in a colouring (Section 5.12).
- side sequence: The indexes of the sides that one fold applies, in the order in which it applies them, which `fold` passes to `visit` (Section 5.3).
- side buffer: The buffer of bytes in which the renderer records the side sequence of one sample for a coset fill (RND-38).
- coset colour: The number from 0 to $k - 1$ that `colour_of` of the colouring gives for the side sequence of a sample (GRP-38 of the group design).
- reflection polygon: The domain of a compact hyperbolic `*p1...pn` with 4 to 8 corners, which `escher-domain` builds with its shape parameters (Section 5.13).
- polygon domain set: The domains of `*3333` at its default and with `slice1.length` 0.5, 2.5, and 3.5, of `*22222` at its default and with both corner-pair lengths 0.5 and both 3.5, of `*5432` at its default and with `slice1.length` 0.5 and 3.5, of `*3222` and `*(100)2222` at their defaults, and of `*22222222` at its default and with every length 0.5 and every length 3.5, together with the rotation domains of `3333`, `22222`, `5432`, `3222`, and `22222222`. A shape that `build_with` refuses would be left out of the set, and it refuses none of these 20 shapes. The module `golden_images` of the tests defines the set, because the unit tests load only that module and `reference` (Section 8.1).
- polygon fold set: The domains of the polygon domain set at 512 evenly spaced angles in each radius band from 1 to 6.
- wallpaper domain: One of the ten domains of Section 6.13 of the domain design: the domain of one of the Euclidean groups `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, or of `2222` at any value of `skew`, which `escher-domain` builds by DOM-50 and DOM-51 of the domain design (Section 5.14). At a skew of 0, the domain of `2222` is also the rotation domain of Section 5.11, bit for bit. Its sides are mirrors, or are paired by translations, half-turns, the rotations of a kite, or glide reflections.
- glide reflection: An isometry of the Euclidean plane that reflects in a line and translates along it. It reverses orientation.
- rectangular groups: The six groups `**`, `xx`, `*x`, `22*`, `22x`, and `2*22`, whose domains are the rectangle of `*2222` at the same `aspect`, or that rectangle with split sides.
- wallpaper domain set: The 27 domains of `o` with the settings of `aspect` and `skew` (1, 0), (1, 0.25), (8, -0.5), and (0.125, 0.5), of each of the rectangular groups with `aspect` 1, 0.125, and 8, of `4*2` and `3*3`, and of `2222` with the settings (1, 0.25), (8, -0.5), and (0.125, 0.5).
- wallpaper fold set: The domains of the wallpaper domain set at plane positions at 512 evenly spaced angles at each distance 1, 4, 16, and 65 from the centre.
- rotary reflection: An isometry of the sphere that rotates about an axis and reflects in the plane through the centre perpendicular to that axis. It reverses orientation. The antipodal map, minus the identity, is the rotary reflection through the angle $\pi$. A pairing of this kind has the kind `PairingKind::RotaryReflection` (`DEC-57`).
- spherical group domain: One of the domains of `3*2`, `2*n`, `n*`, `nx`, `*`, `x`, and `1` that `escher-domain` builds by DOM-58 and DOM-59 of the domain design (Section 5.15). Its sides are mirrors, or are paired by rotations or rotary reflections, and the domain of `1` has no side.
- spherical group domain set: The 19 domains of `3*2`, of `*`, `x`, and `1`, of `n*` and `nx` for $n$ of 2, 3, 5, 12, and 100, and of `2*n` for $n$ of 2, 3, 5, 12, and 50. `2*1` is `2*`, which is `n*` with $n = 2$, and `1*` and `1x` are `*` and `x`.
- spherical group fold set: The domains of the spherical group domain set at the positions and with the viewing matrices of the spherical fold set of Section 6.7, 7168 folds for each domain.
- mask polygon: The convex Euclidean polygon $P$ with straight edges that stands for the domain $D$, with one vertex for each vertex of $D$ and one edge for each side of $D$, in the same cyclic order (Section 5.22).
- edge: Edge $j$ of the mask polygon, the straight segment from its vertex $j - 1$ to its vertex $j$, which stands for side $j$ of the domain.
- generating domain: For a domain that the domain design builds as a double, the reflection domain $F$ from which it is built (Section 6.8 of the domain design).
- mask map: The map $M$ from the domain onto its mask polygon that Section 5.23 constructs.
- chart: The map from a point $x$ of the domain to the plane of Section 5.23 in which the walls of the sides are straight lines.
- mask placement: The scale and the offset that put the mask polygon in an image of a given size, centred and fitted with the margin of Section 5.1 (Section 5.19).
- neighbour colours: For a colour $c$ and a side $j$, the colours that occur across side $j$ of some tile of colour $c$. There can be more than one (Section 5.19).
- meet: Edge $j$ of colour $a$ meets edge $p$ of colour $b$ when some tile of colour $a$ has, across its side $j$, a tile of colour $b$ whose side $p$ lies on that side, with $p$ the partner of $j$ (Section 5.19).
- seam class: A class of the smallest equivalence on the pairs (colour, side) under which two edges that meet are equivalent (Section 5.19).
- code colour: The colour of the border of every edge of one seam class in the polygon masks (Section 5.19).
- mark: The black part of the border of an edge at its marked end, which Section 5.19 chooses so that marked ends meet marked ends.
- polygon mask: The image of one colour that `Masks::render` draws: the mask polygon in white, with a border in the code colour of each edge and its mark, on black (Section 5.19). The plan defines the term for the command line.
- paint: The fill of one coset of a coset fill, `Paint::Solid` or `Paint::Image` (Section 5.20).
- image fill: A paint `Paint::Image`, which gives each sample of its colour a value of its image (Section 5.20).
- texel: One pixel of the image of an image fill, as 8-bit sRGB with an 8-bit alpha (Section 5.21).
- image position: The position $q$, in pixels of the image of an image fill, of the point $M(x)$ of the mask polygon under the mask placement at the size of that image, for a folded point $x$ (Section 5.20).
- fallback: The value of an image fill where the mask map gives no point, or where the image position is not finite or lies outside the image: the background colour (Section 5.20).
- marks: The three on or off values of a Wythoff construction, one for each corner of a reflection triangle. Mark $k$ is on when the mirror opposite corner $k$ is active (Section 5.24).
- active mirror: A side of the triangle whose mark is on. The generator point lies off every active mirror and on every other side.
- generator point: The point $G$ of the triangle whose images under the group are the vertices of the Wythoff tiling (Section 5.24).
- face type: A corner of the triangle that has a face by the face rule of Section 5.24. A face of that type is centred at an image of the corner.
- perpendicular: For an active side $j$, the wall through $G$ and its image $P_j G$ in side $j$, with the covector $q_j$. Its segment from $G$ to side $j$ is half an edge of the tiling.
- foot: The point $F_j$ where the perpendicular of side $j$ meets side $j$.
- regular construction: A Wythoff construction on `*pq2` with $p \ge q \ge 3$ and the marks `010` or `100`, whose faces are the $p$-gons of $\{p, q\}$ or the $q$-gons of $\{q, p\}$ (Section 5.29). The owner calls a face of such a tiling a polygon, and the picture on it a picture. This document says face and image fill.
- face corner: The corner $c$ of the triangle at which the faces of a regular construction are centred, corner 0 for `010` and corner 1 for `100`, with the order $n = n_c$.
- reference face: The face $P$ of a regular construction centred at vertex $c$ of the domain, which holds the domain.
- stabilizer: The group $G_P$ of the elements of the group of the domain that map the reference face onto itself (`UNV-44`).
- centre mirror, vertex mirror, edge side: For a regular construction, the side $m$ through the face corner and corner 2, the side $v$ through the face corner and the third corner $g = 1 - c$, which is side 1, and the side $e$ opposite the face corner, which carries half an edge of the face (Section 5.29).
- arrangement: A colouring of the side presentation of the domain in whose coset table no element of the stabilizer other than the identity fixes a coset, so that its subgroup maps no face onto itself other than by the identity (`UNV-41`, `UNV-42`). An arrangement of index $2n$ puts one picture on every face (`UNV-43`). The owner's "arrangement" is this term (`DEC-111`).
- flat face polygon: The regular $n$-gon $Q$ of Section 5.31, on which the user paints the picture.
- face triangle: The triangle $F'$ of $Q$ to which the face mask map carries the domain (Section 5.31).
- edge of a face: Edge $i$ of the reference face, which stands for edge $i$ of $Q$ (Section 5.29).
- edge pairing: For an arrangement, the edge $\sigma(i)$ of the neighbour face that lies on edge $i$ of the reference face, for each $i$, with a flag that says whether the neighbour is turned or mirrored (Section 5.30).
- turned neighbour, mirrored neighbour: A neighbour face across an edge whose picture is the picture of the reference face moved by an isometry that preserves orientation, or that reverses it. The owner's "turned or mirrored" (`DEC-111`).
- stabilizer element of a coset: For a coset $a$ of an arrangement, the element $w_a$ of the stabilizer whose trace from coset 0 reaches $a$ (Section 5.30).
- face mask: The image that `WholeFace::render_mask` draws for an arrangement: $Q$ in white, with a border in the code colour of the class of each edge and its mark, on black (Section 5.32).
- whole-face fill: The fill `Fill::WholeFace`, which puts the picture on every face of a regular construction as an arrangement says (Section 5.31).

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
pub enum Fill { One(Colour), Parity([Colour; 2]), Cosets(CosetFill), Wythoff(WythoffFill), WholeFace(WholeFaceFill) }
// derives Clone, Debug, PartialEq

pub struct WholeFace { /* private: the construction in a Box, the face corner and its order, the side presentation, the stabilizer words with their flat matrices, the edge words, the flat face polygon, the face triangle, and the face mask map, absent when it cannot be built */ }
// derives Clone, Debug, PartialEq
impl WholeFace {
    pub fn new(domain: &escher_domain::FundamentalDomain, wythoff: Wythoff) -> Result<Self>;
    #[must_use] pub fn presentation(&self) -> &escher_group::Presentation;
    #[must_use] pub fn stabilizer(&self) -> &[escher_group::Word];
    #[must_use] pub fn index(&self) -> u32;
    pub fn edges(&self, arrangement: &escher_group::Colouring) -> Result<Vec<EdgeMeeting>>;
    pub fn render_mask(&self, arrangement: &escher_group::Colouring, width: u32, height: u32) -> Result<Vec<u8>>;
}

pub struct EdgeMeeting { /* private: the partner edge, and whether the neighbour is mirrored */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash
impl EdgeMeeting {
    #[must_use] pub fn partner(self) -> u32;
    #[must_use] pub fn mirrored(self) -> bool;
}

pub struct WholeFaceFill { /* private: the whole face in a Box, the arrangement, the flat matrix of the stabilizer element of each coset, the paints, the mask placement of the flat face polygon for an image paint, and the table that decodes 8-bit sRGB to linear light */ }
// derives Clone, Debug, PartialEq
impl WholeFaceFill {
    pub fn new(face: WholeFace, arrangement: escher_group::Colouring, paints: Vec<Paint>) -> Result<Self>;
}

pub struct Wythoff { /* private: the corners with a face, each with its active sides and their signs of step 7 of Section 5.24, the generator point, the perpendiculars, the feet, and the covectors of the domain */ }
// derives Clone, Debug, PartialEq
impl Wythoff {
    pub fn new(domain: &escher_domain::FundamentalDomain, marks: [bool; 3]) -> Result<Self>;
    #[must_use] pub fn face_types(&self) -> u32;
}

pub struct WythoffFill { /* private: the construction in a Box, and one colour for each face type */ }
// derives Clone, Debug, PartialEq
impl WythoffFill {
    pub fn new(wythoff: Wythoff, colours: Vec<Colour>) -> Result<Self>;
}

pub struct CosetFill { /* private: the side presentation of the domain, the colouring, and its paints */ }
// derives Clone, Debug, PartialEq
impl CosetFill {
    pub fn new(
        domain: &escher_domain::FundamentalDomain,
        colouring: escher_group::Colouring,
        paints: Vec<Paint>,
    ) -> Result<Self>;
}

#[non_exhaustive]
pub enum Paint { Solid(Colour), Image(Image) }
// derives Clone, Debug, PartialEq

pub struct Image { /* private: the width, the height, and the texels */ }
// derives Clone, PartialEq, Eq. Implements Debug by hand (Section 5.21)
impl Image {
    pub fn decode_png(reader: impl std::io::Read) -> Result<Self>;
    #[must_use] pub fn width(&self) -> u32;
    #[must_use] pub fn height(&self) -> u32;
}

pub struct Masks { /* private: the mask polygon, and the code colour and the marked ends of each edge of each colour */ }
// derives Clone, Debug
impl Masks {
    pub fn new(
        domain: &escher_domain::FundamentalDomain,
        colouring: &escher_group::Colouring,
    ) -> Result<Self>;
    #[must_use] pub fn count(&self) -> u32;
    pub fn render(&self, colour: u32, width: u32, height: u32) -> Result<Vec<u8>>;
}

#[non_exhaustive]
pub enum Shading { Phong, Flat }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

#[non_exhaustive]
pub enum HyperbolicModel { Disk, HalfPlane, Band, Klein }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

#[non_exhaustive]
pub enum BorderUnit { Pixels, Intrinsic }
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
    pub fn border_width(self, width: f64) -> Self;
    pub fn border_unit(self, unit: BorderUnit) -> Self;
    pub fn border_colour(self, colour: Colour) -> Self;
    pub fn background(self, colour: Colour) -> Self;
    pub fn max_folds(self, cap: u32) -> Self;
    pub fn rotation(self, yaw: f64, pitch: f64, roll: f64) -> Self;
    pub fn light(self, direction: glam::DVec3) -> Self;
    pub fn shading(self, shading: Shading) -> Self;
    pub fn model(self, model: HyperbolicModel) -> Self;
    pub fn build(self) -> Result<RenderOptions>;
}

pub struct Renderer { /* private */ }
// derives Clone, Debug
impl Renderer {
    pub fn new(domain: &escher_domain::FundamentalDomain, options: RenderOptions) -> Result<Self>;
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
pub enum ErrorKind { InvalidOption, LimitExceeded, InvalidBuffer, Encoding, Decoding, Numerical }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

`builder` carries no `#[must_use]` of its own, because its return type is `#[must_use]` and Clippy rejects the second attribute (`clippy::double_must_use`). The crate does not re-export `glam` (API-10 of the coding standard). A caller that names a `glam` type declares `glam` from the workspace entry. In the same way the crate re-exports neither `escher-domain` nor `escher-group`, and a caller that builds a `CosetFill` declares both.

`Fill` derives no `Copy`, because `CosetFill` holds a side presentation, the coset table of its colouring, and a list of paints, which are not `Copy`. A caller that needs a `Fill` twice clones it, which copies the texels of every image fill.

`Masks::new` takes the colouring by reference, because `Masks` keeps only the mask polygon and the code colours and marks that it computes from the colouring and the domain. `Wythoff` is a type of its own, apart from `WythoffFill`, because the command line needs the number of face types to choose the default colours before it has the colours. `Paint` has no variant for `Fill::One` or `Fill::Parity`: an image fill exists only as a paint of a coset fill (`DEC-91`). `WythoffFill` takes colours and no paints, because the faces of a Wythoff fill take one solid colour each and no image fill (`DEC-68`). It holds its construction in a `Box`, because the construction is much larger than the other variants of `Fill`, and unboxed it would make every `Fill` that large. `WholeFaceFill` holds its `WholeFace` in a `Box` for the same reason.

`WholeFace` is a type of its own, apart from `WholeFaceFill`, because the command line needs its side presentation, its stabilizer words, and its index for the search of the arrangements before it has an arrangement, and its edge pairing and face mask without a fill (`DEC-115`). `WholeFaceFill::new` takes a list of paints, one for each orbit of the arrangement on the faces, so that a later fill with several pictures of one face keeps the signature. This design accepts exactly one paint, because it accepts only arrangements with one orbit (Section 5.29). `escher_group::Presentation` and `escher_group::Word` appear in the public surface, as `Colouring` already does.

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `Colour` | An opaque colour, stored in linear light | Each channel is finite and in $[0, 1]$, because the only constructors take 8-bit values |
| `Colour::BLACK`, `Colour::WHITE` | Black and white | Equal to `from_srgb8(0, 0, 0)` and `from_srgb8(255, 255, 255)` |
| `from_srgb8` | Converts 8-bit sRGB to linear light by the transfer function of Section 5.6 | Total |
| `to_srgb8` | Converts linear light to 8-bit sRGB by the transfer function of Section 5.6 | `to_srgb8(from_srgb8(r, g, b))` is `[r, g, b]` for every byte value |
| `Fill` | The colouring with its colours | For `One` and `Parity`, the number of colours matches the colouring by construction (TYPE-3 of the coding standard). For `Cosets`, `CosetFill::new` has checked it (RND-36), for `Wythoff`, `WythoffFill::new`, and for `WholeFace`, `WholeFaceFill::new` |
| `CosetFill` | A colouring of `escher-group` with one paint for each of its cosets, for the side presentation of the domain that `new` checked it against | Built only by `new`, so the colouring has one generator for each side of that domain, every relator of the side presentation of that domain returns each coset to itself, and the number of paints equals the colour count of the colouring. It keeps the side presentation of the domain and no reference to the domain |
| `CosetFill::new` | Builds the side presentation of the domain, checks the colouring and the paints against it, and keeps the side presentation, the colouring, and the paints | Takes the colouring by value. Returns `InvalidOption` as RND-36 and Section 7.2 state. Never panics |
| `Wythoff` | The Wythoff construction of one reflection triangle for one set of marks (Section 5.24) | Built only by `new`, so the domain has three sides of kind `Mirror`, at least one corner has a face, and the generator point, the perpendiculars, and the feet were computed. It keeps the covectors of the domain and no reference to the domain |
| `Wythoff::new` | Builds the construction by the steps of Section 5.24 | Returns `InvalidOption` or `Numerical` as RND-65 and Section 7.2 state. Never panics |
| `Wythoff::face_types` | The number of face types, which is the number of corners with a face | 1, 2, or 3 |
| `WythoffFill` | A construction with one colour for each face type, in the order of the corners | Built only by `new`, so the number of colours equals `face_types()` |
| `WythoffFill::new` | Checks the number of colours and keeps the construction and the colours | Returns `InvalidOption` with the text of Section 7.2 for another number of colours. Never panics |
| `WholeFace` | The faces of one regular construction with their stabilizer, edges, flat face polygon, and face triangle (Section 5.29) | Built only by `new`, so the construction belongs to the domain, is a regular construction, and the side presentation was built. It keeps no reference to the domain |
| `WholeFace::new` | Builds the whole face by the steps of Section 5.29 | Returns `InvalidOption` as RND-71 and Section 7.2 state. Never panics |
| `WholeFace::presentation` | The side presentation of the domain, which the search of the arrangements takes | Equal to `side_presentation()` of the domain |
| `WholeFace::stabilizer` | The $2n - 1$ elements of the stabilizer other than the identity, as words in the generators of the side presentation, in the order of step 6 of Section 5.29 | $2n - 1$ words of at most $2n - 1$ letters each |
| `WholeFace::index` | The index $2n$ of an arrangement with one picture, which the search takes as its colour count | From 6 to 200 |
| `WholeFace::edges` | The edge pairing of an arrangement, one `EdgeMeeting` for each edge $i$ from 0 to $n - 1$ (Section 5.30) | Checks the arrangement as RND-72 states and returns `InvalidOption` with the texts of Section 7.2. Never panics |
| `EdgeMeeting::partner`, `EdgeMeeting::mirrored` | The edge $\sigma(i)$ of the neighbour that lies on edge $i$, and whether that neighbour is mirrored | The partner is below $n$ |
| `WholeFace::render_mask` | The face mask of an arrangement as a buffer of 8-bit sRGB in the layout of `render_rows`, with the fixed sample grid of Section 5.19 (Section 5.32) | Checks the arrangement as `edges` does, and then the size as `build` checks it. Starts no thread (RND-2) |
| `WholeFaceFill` | A whole face with an arrangement and one paint for each orbit of the arrangement on the faces | Built only by `new`, so the arrangement passed the checks of RND-72 and the number of paints is 1 |
| `WholeFaceFill::new` | Checks the arrangement and the paints, and computes the flat matrix of the stabilizer element of each coset (Section 5.30) | Returns `InvalidOption` with the texts of Section 7.2. Never panics |
| `Paint` | The fill of one coset: `Solid` a colour, and `Image` an image fill (Section 5.20) | Any value. A coset fill of solid paints draws as Section 5.12 states (RND-61) |
| `Image` | A decoded image of at least 1 by 1 texels | Built only by `decode_png`, so its size is within the limits of Section 5.21 and it holds $4WH$ bytes |
| `decode_png` | Reads one PNG image of bit depth 8 or less from the reader, as Section 5.21 states | Checks the size and the bit depth from the header before it allocates the texels (RND-62). Returns `LimitExceeded` or `Decoding` as Section 7.2 states. Never panics |
| `Image::width`, `Image::height` | The size of the image in pixels | From 1 to 16384 each |
| `Masks` | The polygon masks of one colouring of one domain | Built only by `new`, which checks the colouring against the domain as conditions 1 to 3 of `CosetFill::new` do, and builds the mask polygon (RND-63) |
| `Masks::new` | Builds the side presentation, checks the colouring against it, builds the mask polygon of the domain (Section 5.22), and finds the seam class, the code colour, and the marked ends of each edge of each colour (Section 5.19) | Returns `InvalidOption` with the texts of Section 7.2, including for a domain without a mask polygon. Never panics. Keeps no reference to the domain |
| `Masks::count` | The number of masks, which is the colour count of the colouring | From 1 to the colour count limit of `escher-group` |
| `Masks::render` | The polygon mask of one colour as a buffer of 8-bit sRGB in the layout of `render_rows`, for the size given, with the fixed sample grid of Section 5.19 | Returns `InvalidOption` for a colour at or above `count()`, and checks the size as `build` checks it (Section 4.1). Starts no thread (RND-2) |
| `Shading` | `Phong` shades the ball as Section 5.10 states, and `Flat` leaves each sample colour unchanged, so the ball is a flat disk | Read only for a spherical domain. The name `Flat` avoids a variant `None`, which reads as an `Option` |
| `HyperbolicModel` | The model in which the renderer draws a hyperbolic domain: `Disk` the Poincare disk, `HalfPlane` the upper half-plane, `Band` the band, and `Klein` the Klein disk (Section 5.16) | Read only for a hyperbolic domain |
| `model` | The hyperbolic model | Any variant. It has no effect on a Euclidean or spherical domain (`DEC-77`) |
| `BorderUnit` | The unit of the border width: `Pixels` output pixels, and `Intrinsic` the intrinsic unit of the geometry of the domain (Section 5.17) | Read for every geometry |
| `border_width` | The full width of the band around each wall, in the border unit | Checked by `build`, with the same limits in both units (`DEC-78`) |
| `border_unit` | The border unit | Any variant |
| `rotation` | The viewing rotation of the ball, as the yaw, the pitch, and the roll in radians (Section 5.9) | Checked by `build` |
| `light` | The direction from the ball to the light, in the coordinates of the screen of Section 5.9. Its length has no effect | Checked and normalized by `build` |
| `RenderOptions` | Validated options | Built only by `build`, which checks every limit of Section 4.1 |
| `RenderOptions::builder` | Starts from the defaults of Section 4.1 with the given fill | Infallible |
| `build` | Validates the options | Returns `InvalidOption` or `LimitExceeded` as Section 7.2 states. Never panics |
| `Renderer::new` | Copies the covectors, pairings, and orientations of the sides of the domain, chooses the view from `geometry()` of the domain (RND-23) and, for a hyperbolic domain, from the hyperbolic model, and computes the scale of the hyperbolic model, the plane scale, or the ball scale and the viewing matrix, and the tolerance scale of each side | For a coset fill with an image fill, also builds the mask polygon and the mask map of this domain, and the mask placement of each image (Sections 5.20, 5.22, and 5.23). Returns `InvalidOption` for a `Fill::Cosets` whose `CosetFill` was built for another side presentation than that of the domain, for a coset fill with an image fill on a domain without a mask polygon (RND-41), for a `Fill::Wythoff` built for another domain (RND-68), and for a `Fill::WholeFace` built for another domain (RND-76). Every other fill gives no error, because the domain and the options are valid by construction, and every geometry has a view. The renderer keeps no reference to the domain |
| `render_rows` | Renders the rows from `first_row` into the caller's buffer | The buffer holds 8-bit sRGB, 3 bytes per pixel, red first, the top row first. Its length MUST be a positive multiple of $3W$, and the band MUST lie inside the image, else `InvalidBuffer`. It starts no thread (RND-2) |
| `render` | The whole image on the calling thread | Equal, byte for byte, to the concatenation of `render_rows` over any split into bands (RND-3) |
| `RenderStats` | Counts over the samples of a band or an image | Integer counts only, so no value is NaN |
| `samples()` | The number of samples, including outside samples | $W n^2$ times the number of rows of the band |
| `outside()` | The number of outside samples, which are the samples outside the hyperbolic model or the ball | At most `samples()`. 0 for a Euclidean domain |
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

RND-3. The colour of each pixel MUST depend only on the domain, the options, and the position of the pixel, and MUST NOT depend on the band that contains it or on any other sample, because the caller renders bands in parallel in any order and expects the same file for every split, and the tests compare the bytes of every split exactly (Section 8.1). The crate therefore does not seed a fold from a neighbouring pixel, which Section 8 of the architecture defers.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, the compile-time assertion of the auto traits, and, under `#[cfg(test)]`, the modules `reference` and `golden_images`, loaded from `tests/render/reference.rs` and `tests/render/golden_images.rs` through `#[path]` (Section 8.1).
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/colour.rs`: `Colour`, `Fill`, `Paint`, `CosetFill` with its checks, and the sRGB transfer function.
- `src/image.rs`: `Image`, `decode_png` with its limits, the mask placement, and the bilinear sampling of Section 5.20.
- `src/mask.rs`: `Masks`, the private mask polygon of Section 5.22 and mask map of Section 5.23, which `Renderer::new` also builds and `WholeFace::new` builds onto the face triangle, the seam classes and the code colours, and the private mask function of Section 5.19, which takes the sample grid.
- `src/wythoff.rs`: `Wythoff`, `WythoffFill`, the face rule, the generator point, the perpendiculars and the feet of Section 5.24, the face of a sample of Section 5.25, and the perpendiculars through the corner of each face, which the border test of Section 5.26 reads.
- `src/whole_face.rs`: `WholeFace`, `EdgeMeeting`, `WholeFaceFill`, the steps of Section 5.29, the stabilizer element of each coset and the edge pairing of Section 5.30, the flat face polygon, the face triangle, and the picture point of a sample of Section 5.31, and the face mask of Section 5.32, whose classes and marks it computes through the seam classes of `mask.rs` and which it draws through the private mask function of `mask.rs`.
- `src/options.rs`: `RenderOptions`, `RenderOptionsBuilder`, `Shading`, `HyperbolicModel`, `BorderUnit`, the defaults, and the limits.
- `src/fold.rs`: `Fold`, `fold`, the private fold that carries the screen tangents, the private enum that selects the point size, and the tolerance constant.
- `src/view.rs`: The private view, which maps a sample to a disk position, a plane position, or a ball position and a point of the linear model with its screen tangents, by the geometry of the domain, and the symmetric view, the periodic view, and the rotated view of the unit tests. The view holds one case per geometry: the disk with its hyperbolic model, the scale of that model, and the placement, the plane with its plane scale, or the ball with its scale and the viewing matrix (Section 5.9).
- `src/renderer.rs`: `Renderer`, `RenderStats`, the border estimate, the intrinsic border test, the colour of a face of a Wythoff fill, the side buffer, the colour of a sample with its coset colour, its shading (Section 5.10), and the colour of a pixel. The border estimate and the intrinsic border test each take the covectors to test, so the sides of the domain and the perpendiculars of a Wythoff construction share one test.
- `src/encode.rs`: `encode_png`.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, and lets the crate design refine the names and signatures:

- A public `View` with the model and the placement. Each geometry has one canonical view in this design, which the renderer chooses from the domain, and for a hyperbolic domain one canonical view in each hyperbolic model. The options that change a view are the viewing rotation of the ball, which `RenderOptions` carries as three angles, and the hyperbolic model, which it carries as a `HyperbolicModel`, so a public `View` would give a caller nothing more to choose, and SIMP-1 of the coding standard removes an abstraction that gives no benefit now. The disk case of the private view holds the placement isometry, which is the identity for the canonical view. The change that adds the first option that changes the placement makes the view public.
- A colouring id. `CosetFill::new` takes the `Colouring` itself, which `escher-group` finds from an id, so the crate parses no id and runs no search, and the command line owns the resolution of an id and its errors (Section 4.6 of the command-line design).
- A palette. The caller gives every fill colour, so the library states no default colour, as Section 4.2 states.
- The lift onto the ball as a public function. It lives in the private view beside its screen tangents, as the tangents of the disk do, because the view is its one caller.
- A public mask polygon, mask map, or sampler interface. `Renderer::new` builds the mask polygon and the mask map from the domain, so a caller has nothing to choose (`DEC-88`).
- A public generator point, perpendicular, or face of a point. The command line needs only the number of face types, and the unit tests of `wythoff.rs` read the private items.
- A public seam class or code colour. The command line needs only the masks.
- A public flat face polygon, face triangle, stabilizer element of a coset, or picture point. The command line needs the edge pairing and the face mask only, and the unit tests of `whole_face.rs` read the private items.
- A search of the arrangements. The command line calls the search of `escher-group` with the presentation, the stabilizer, and the index of a `WholeFace` (`DEC-115`), so this crate runs no search, as for the colourings.

`fold` is public so that the integration tests, which use only the public surface (TEST-1 of the coding standard), can read the side sequence for the reference tests and the border oracle.

## 4. Options

### 4.1. Defaults and Limits

RND-4. `RenderOptionsBuilder::build` MUST return `InvalidOption` or `LimitExceeded` for each value outside the column "Accepted" of this table, and the builder MUST start from the column "Default", because the input comes from a user, the limits bound the memory of an image (Section 13), and Section 4.6 of the command-line design relies on these defaults:

| Option | Unit | Default | Accepted | Kind outside the accepted values |
| --- | --- | --- | --- | --- |
| `size` | Pixels | 512 by 512 | Width and height from 1 to 16384 each, and at most $2^{26}$ pixels | `InvalidOption` for 0, `LimitExceeded` above a limit |
| `samples` | Samples along each axis of a pixel | 4 | 1 to 16 | `InvalidOption` for 0, `LimitExceeded` above 16 |
| `border_width` | Output pixels, or the intrinsic unit with `BorderUnit::Intrinsic` (Section 5.5 of the architecture) | 1, in either unit (`DEC-79`) | A finite number from 0 to 1024, in either unit (`DEC-78`) | `InvalidOption` for a negative or non-finite number, `LimitExceeded` above 1024 |
| `border_unit` | `BorderUnit` | `BorderUnit::Pixels` | Any | None |
| `border_colour` | `Colour` | `Colour::BLACK` | Any | None |
| `background` | `Colour` | `Colour::WHITE` | Any | None |
| `max_folds` | Steps | 1500 | 0 to 100000 | `LimitExceeded` above 100000 |
| `rotation` | Radians, as the yaw, the pitch, and the roll | 0, 0, 0 | Three finite numbers | `InvalidOption` for the first angle that is not finite |
| `light` | A direction in the coordinates of the screen of Section 5.9 | $(-1, 1, 2)$ | Three finite components, not all zero | `InvalidOption` |
| `shading` | `Shading` | `Shading::Phong` | Any | None |
| `model` | `HyperbolicModel` | `HyperbolicModel::Disk` | Any | None |
| `fill` | `Fill` | None, given to `builder` | Any | None |

`Masks::render` checks its size against the row `size` of this table, with the same kinds and texts under its own operation (Section 7.2), and takes no other option. `Image::decode_png` checks the size of an image against the same row (Section 5.21).

`build` checks the options in the order of the rows of this table and reports the first value that fails. A border width of 0 draws no border in either unit, and an unconverged sample still takes the border colour (Section 5.5). The texts of the two errors of the border width name no unit, so they hold for both units (Section 7.2).

`build` checks `rotation`, `light`, and `shading` for every domain, and the renderer reads them only for a spherical domain. They have no effect on another domain, just as the background has none on a Euclidean domain. In the same way the renderer reads `model` only for a hyperbolic domain, and it has no effect on a Euclidean or spherical domain (`DEC-77`). `build` normalizes the light direction: it divides the direction by its largest absolute component and then by the length of the result, so that no accepted direction overflows or underflows, and its length has no effect. This order of the two divisions is a choice of this design.

The defaults for the border, the sample grid, the fold cap, and the background are those of `DEC-7`, which proposes an opaque white background and no `transparent` one. The default cap of 1500 lies above the 1000 steps that a sample next to a corner of order 1000 needs, and above the step count of every fold of the fold set of Section 6.4. That order lies above the limit of 100 of DOM-3 of the domain design, and the owner kept the cap under that limit (`DEC-69` of the architecture). The defaults of the viewing rotation, the light, and the shading are those of `DEC-16`.

### 4.2. Colours

RND-5. The crate MUST take every colour as 8-bit sRGB through `Colour::from_srgb8` and MUST NOT parse colour text, because this design puts the project's parser of colour text (`DEC-29`) in the command line (Section 4.6 of the command-line design), and a parser in the library would add a second place that defines what a colour string means.

`Fill` has no default, so the library states no default fill colour. Section 4.6 of the command-line design states the default colours of `escher render`.

## 5. Rendering a Sample

The crate computes the colour of a pixel in the order of Section 4.1 of the architecture. Sections 5.1 to 5.6 state each part, and Section 5.7 gives the source of each step. Section 5.8 states how each step reads for a Euclidean domain, and where it differs it replaces Sections 5.1 and 5.2. Section 5.9 does the same for a spherical domain, and Section 5.10 states the shading of a sample of the ball. Sections 5.11 to 5.15 state how each step reads for the rotation domains, the coset fill, the reflection polygons, the wallpaper domains, and the spherical group domains. Section 5.16 states how a hyperbolic domain is drawn in the half-plane, the band, and the Klein model, and where it differs it replaces Sections 5.1 and 5.2. Section 5.17 states the border of intrinsic width, which replaces the border estimate of Section 5.4 with `BorderUnit::Intrinsic`. Section 5.22 states the mask polygon of a domain and Section 5.23 the mask map, on which Section 5.19 builds the polygon masks and Section 5.20 the image fills, and Section 5.21 states the decoding of an image. Sections 5.24 to 5.27 state the Wythoff fill: the construction, the face of a sample, the borders of the faces, which replace those of Sections 5.4 and 5.17 for that fill, and the colours of the faces, and Section 5.28 gives the source of each of their steps. Sections 5.29 to 5.32 state the whole-face fill: the faces of a regular construction with their stabilizer, the arrangements and their edge pairing, the picture of a sample, and the face mask, and Section 5.33 gives the source of each of their steps. Section 5.18 is withdrawn.

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

The renderer also computes the two screen tangents, the partial derivatives of the map of Section 7.2 of the geometry design in $z_1$ and $z_2$, times $h$. With $d = 1 - \lvert z \rvert^2$, they are

$$T_1 = h\left(\frac{2}{d} + \frac{4 z_1^2}{d^2}, \ \frac{4 z_1 z_2}{d^2}, \ \frac{4 z_1}{d^2}\right), \qquad T_2 = h\left(\frac{4 z_1 z_2}{d^2}, \ \frac{2}{d} + \frac{4 z_2^2}{d^2}, \ \frac{4 z_2}{d^2}\right)$$

No reference states these derivatives. Section 10 records them as an unverified statement of this design, and a reference test checks them against central differences of the reference map. The renderer computes $d$ as `from_disk` does, so that it divides by the same number.

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

RND-26. The fold MUST read the point size at steps 2 and 3 in place of the last coordinate. For a hyperbolic or Euclidean domain it MUST use $x_3$ as the point size, and MUST compute the threshold of step 3 as the product of $\tau$, $\sigma_i$, and $x_3$, taken from left to right, because the committed golden images and the exact tests of bands, scale, and periodicity fix every decision of those domains, and any other order of operations can change one.

A point that violates no side at the start stops at step 4 with zero steps, and `Fold::point` returns its input unchanged, bit for bit, because no arithmetic touches it.

RND-8. The fold MUST NOT renormalize the point or the screen tangents, at any step and at the stop, because renormalization rounds the point by a factor that is noise near the limit circle and changes which sides the fold applies, and it can produce infinite coordinates. The decisions of step 3 do not need it, because both sides of the comparison scale with $x$ (Section 6.1).

The fold computes the dot product itself and calls no side test of `escher-geometry`, so the tolerance of step 3 belongs to this crate, as Section 9.2 of the architecture requires. Section 6.2 states the constant $\tau$.

The parity of a fold is the number of its steps whose pairing reverses orientation, modulo 2, for every domain, because the fold reads the orientation of each side rather than assuming a mirror. A reflection domain has from two to eight sides of kind `Mirror`, each its own partner, so the fold applies a reflection at every step and its parity equals its step count modulo 2. A rotation domain has from two to 14 sides, and every pairing preserves orientation, so its reversal count stays 0 and its parity is even whatever its step count (Section 5.11). A wallpaper domain has from four to six sides, which can mix mirrors and paired sides, and its reversal count counts each step across a mirror or a glide reflection (Section 5.14). The loop therefore needs no change for a rotation domain or a wallpaper domain, although the accuracy of the fold depends on the pairings (Sections 6.8 and 6.10).

### 5.4. Border

RND-9. The renderer MUST mark a converged sample as a border sample exactly when, for some side $i$ of the domain, the expression `!(f_i.abs() >= half_width * g_i)` is true, with $f_i$ the side value at the folded point, $g_i$ the gradient size of this section, and the half width in pixels, because this is the border estimate of `DEC-28`, and the negated comparison makes a NaN side value or a NaN or infinite gradient size give a border sample rather than a fill colour (Section 6.3).

The gradient size of side $i$ is

$$g_i = \sqrt{(c_i \cdot T_1)^2 + (c_i \cdot T_2)^2}$$

with $T_1$ and $T_2$ the screen tangents after the fold. In words, $g_i$ is the change of the side value per pixel on the screen, so $\lvert f_i \rvert / g_i$ estimates the distance in pixels from the sample to the wall of side $i$ in the tile of the sample. The renderer evaluates the condition for every side of the domain, and for no other wall (RND-34 in Section 5.11). The estimate follows the gradient form of Quilez, `abs(v)/length(g)`, and the Jacobian form of Skia (Section 7.1 of the rendering reference). No reference gives it through a chain of folds in the linear model, which is `UNV-15` and item 19 of Section 13.1 of the rendering reference.

Scaling the folded point and both screen tangents by one factor scales $f_i$ and $g_i$ by that factor and leaves the condition unchanged, so a common scale of the unnormalized point and its tangents does not matter (RND-8). With the pixel unit, in the disk, the half-plane, and the Klein model, no transcendental function runs for a sample, so the platform mathematics library enters the image only through the domain and the sRGB transfer function. In the band, the complex hyperbolic tangent of each sample calls it too (Section 5.16), and so does the intrinsic unit in every geometry (Section 5.17).

The tests compare this estimate with a second form, the distance form, which only the reference arithmetic evaluates. With $\hat{x}$ the folded point divided by $\sqrt{-\langle x, x \rangle}$ and $\hat{f}_i$ the side value at $\hat{x}$, the distance form marks a border sample when

$$\operatorname{arsinh}\lvert \hat{f}_i \rvert \cdot \frac{1 - \lvert z \rvert^2}{2} < h \cdot w$$

for some side $i$, where $w$ is the half width and $z$ is the disk position of the sample. Its parts are the unit-normal form $\sinh d = \lvert \langle x, u \rangle \rvert$, which is part of `UNV-14` (item 3 of Section 15.1 of the geometry reference), and the disk metric of [CFKP], p. 71 (Section 6.2 of the rendering reference). Zucker converts a hyperbolic distance in the same way (Section 6.1 of the rendering reference). The tests count the samples that the two forms classify differently.

### 5.5. Colour of a Sample

RND-10. The renderer MUST give each sample the colour of the first row of this table that applies, because an unconverged sample carries no reliable tile and must not take a fill colour, and the precedents of Section 4.9 of the rendering reference give such a sample a fixed colour:

| Case | Colour |
| --- | --- |
| Outside sample | The background, unshaded |
| Unconverged | The border colour (`DEC-7`) |
| Border sample | The border colour |
| `Fill::One(c)` | $c$ |
| `Fill::Parity([c_0, c_1])` | $c_0$ for an even parity and $c_1$ for an odd parity |
| `Fill::Cosets(fill)` | The paint of the fill at the index of the coset colour of the side sequence (RND-37, Section 5.12): its colour for `Paint::Solid`, and the value of the image at the folded point for `Paint::Image` (Section 5.20) |
| `Fill::Wythoff(fill)` | The colour of the fill at the index of the face type of the folded point (Sections 5.25 and 5.27) |

With `Fill::Wythoff`, a border sample is one that the test of Section 5.26 marks, in place of RND-9 and RND-54.

For a spherical domain, the colour of every sample that is not an outside sample is then shaded as Section 5.10 states. The domain itself takes the first colour of `Fill::Parity`, because a point inside it folds in zero steps. Every tile of a rotation domain takes that colour too, because every fold of such a domain has an even parity (RND-33 in Section 5.11). The parity colouring rests on `UNV-18`, with the precedents of the flip parity of [ZUCKER] and [NS] and the swap of two colours of [SYMMHUB] (Section 4.6 of the rendering reference).

### 5.6. Pixel and Image

The colour of a pixel is the mean of the colours of its $n^2$ samples in linear light, each channel separately, after the shading of Section 5.10. The renderer then converts the mean to 8-bit sRGB with `Colour::to_srgb8`, which applies the encoding direction of the sRGB transfer function, clamps the result to the range 0 to 1, and rounds $255$ times it to the nearest integer. `Colour::from_srgb8` applies the decoding direction to each byte divided by 255.

RND-11. The crate MUST implement the sRGB transfer function as Section 7.5 of the rendering reference states it, in both directions, and MUST NOT take a constant of the function from any other place, because a constant that differs from the source shifts every colour of every image without any test that a human would notice.

Averaging in linear light is a choice that no research source supports (Section 4.1 of the architecture).

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
| 5.5 | The coset fill | Section 5.12 | `UNV-17` |
| 5.6 | The sRGB transfer function | Section 7.5 of the rendering reference | Sourced through secondary sources. The rounding of an encoder is item 23 of Section 13.1 of the rendering reference |
| 5.6 | Averaging in linear light | None | A choice of the architecture |
| 5.6 | The sRGB chunk through `set_source_srgb` | Section 4 of the ecosystem survey | The API of `png` |
| 5.16 | The maps from the half-plane, the band, and the Klein model to the disk | [RAT], Exercise 4.4.4, p. 126, the code of [SYMMHUB], and [CFKP], p. 71, as `hyperbolic::disk_from_half_plane`, `disk_from_band`, and `disk_from_klein` (Sections 7.5 to 7.7 of the geometry design) | Sourced, except the band map, which is `UNV-13`. The owner chose the half-plane map (`DEC-75`) |
| 5.16 | The model Jacobians, and the screen tangents built from them | None | Unverified (Section 10) |
| 5.16 | The canonical views of the other models | None | A decision of the owner (`DEC-76`) |
| 5.17 | The distance from the folded point to the wall of a side, in each geometry | [RAT], equation (3.2.8), p. 75, for the hyperboloid, and the candidate of item 10 of Section 13.1 of the rendering reference for the sphere, as `hyperbolic::wall_distance` and `spherical::wall_distance` (Section 5.6 of the geometry design). No source for the Euclidean plane, as `euclidean::wall_distance` | `UNV-14` for the hyperboloid and the sphere, and an unverified statement of the geometry design for the plane |
| 5.17 | The distance to the whole wall of a side is the distance to the side for a sample inside a convex domain | Section 5.11 | `UNV-31` for the doubles, as for the border estimate |
| 5.19 | The colour of the neighbour of the domain across side $j$, and the closure of the coset table that gives the pairs of colours that occur across side $j$ | GRP-38 of the group design for the side sequence of length one, and no source for the closure | `UNV-17` for the first, and unverified for the closure (Section 10) |
| 5.19 | A pairing maps the ends of a side onto the ends of its partner, exchanged or in the same order by its kind | Section 5.3 and check 2 of Section 8.2 of the domain design | Check 2 of `verify()` tests it |
| 5.20 | Fold the point, map it into the image, and sample the image there | SymmHub, which maps the folded point into a pattern box with a similarity and samples it with linear, biquadratic, or box filtering (Section 10 of the rendering reference) | Sourced precedent. The mask map, the bilinear filter, the alpha, and the fallback are choices (`DEC-88`, `DEC-89`, `DEC-97`) |
| 5.20 | A tile that is a mirror image of another tile of its colour shows the image mirrored | The flip parity of Nelson and Zucker, which selects a mirrored texture lookup (Section 10 of the rendering reference) | Sourced precedent. The fold gives it, because both tiles fold to the same domain |
| 5.22 | The Klein position of a point of the hyperboloid, and walls as straight chords in the Klein model | [RAT], equation (6.1.2), p. 192, and [CFKP], Section 9 (Sections 3.1 and 4.2 of the geometry reference) | Sourced |
| 5.23 | The mask map is a continuous bijection onto the mask polygon with the edge behaviour of Section 5.23 | None | Unverified (Section 10) |
| 5.21 | The colour types, bit depths, and chunks of a PNG image | The API of `png` | A choice (`DEC-90`). The test "Decoding" pins the texels of each colour type and bit depth that the crate accepts |

### 5.8. Euclidean Domains

RND-23. `Renderer::new` MUST choose the view from `geometry()` of the domain: the disk view of Sections 5.1 and 5.2 for `Hyperbolic`, or the view of Section 5.16 when the hyperbolic model is not `Disk`, the plane view of this section for `Euclidean`, and the ball view of Section 5.9 for `Spherical`, because the fold and the border estimate accept the walls of any domain, and a domain drawn through the view of another geometry gives a plausible but wrong image without an error.

The choice is a `match` on `Geometry` with no wildcard arm (TYPE-7 of the coding standard). Every geometry has a view, so the choice of the view gives `Renderer::new` no error.

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
| 5.4 | Unchanged (RND-9). The pairings move the screen tangents with the point, so $\lvert f_i \rvert / g_i$ reads as the distance in pixels from the sample to the wall of side $i$ in its tile, up to rounding, a reading that no source states and that the Euclidean border oracles check |
| 5.5 | Unchanged. No sample is an outside sample, so the background colour does not appear and `outside()` is 0 |
| 5.6 | Unchanged |

A sample whose side value equals the half width times the gradient size is not a border sample, because the comparison of RND-9 then holds. For `*2222` with `aspect` 1 or 4 in a 512 by 512 image with one sample per pixel, every wall lies on a boundary between pixels, so every sample lies exactly half a pixel from the nearest wall. At a border width of 1, no sample is then a border sample and the image shows no border. With a grid of 2 or more samples per pixel the borders show. This is a property of point sampling in this view, and a test records it.

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

The ball shows only the front hemisphere of the screen, the points with $s_3 > 0$. The tangents grow as $1/\zeta$ toward the rim, and stay finite for every positive $d$. No source states the lift onto the ball, which only SymmHub's code uses (item 11 of Section 13.1 of the rendering reference), and it is part of `UNV-16`. No source states its tangents either. Section 10 records them as an unverified statement of this design, and a reference test checks them against central differences of the reference lift.

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

The estimate of RND-9 is first order, and the lift is not, so near the rim the estimate marks more samples as border samples. With a black border the ball shows a faint dark outline about one pixel wide at the rim. Section 6.7 states the range of accuracy of the border estimate by rim distance.

A border of a width in pixels covers a tile that is narrower than the border. The tiles of `*n22` and of `*nn` meet at a vertex of order $n$ in angles of $\pi/n$, so near that vertex they are narrower than the border, and the ball shows border colour there, apart from the shading. No test measures how large that area is at the default size. This is a property of borders of pixel width on small tiles, and not a defect.

### 5.10. Shading

For a spherical domain with `Shading::Phong`, the renderer shades the colour of each sample that is not an outside sample: fill, border, and unconverged samples alike, as `DEC-16` states for borders. It shades after Section 5.5 chooses the colour and before Section 5.6 averages the samples of the pixel. With `Shading::Flat`, the colour stays unchanged, and the ball is a flat disk.

The normal is the screen point, $N = s$, and $L$ is the normalized light direction of the options, in the same coordinates of the screen. The light is one white light, fixed to the viewer, so the viewing rotation does not move it, and the view direction is $(0, 0, 1)$. The terms are

$$\delta = \max(N \cdot L, \ 0), \qquad \rho = \begin{cases} \max(2 (N \cdot L) N_3 - L_3, \ 0)^{\alpha} & \text{if } N \cdot L > 0 \\ 0 & \text{otherwise} \end{cases}$$

and each linear-light channel $c$ of the colour becomes

$$c' = c \, (k_a + k_d \, \delta) + k_s \, \rho \, (1 - c)$$

with $k_a = 0.25$, $k_d = 0.75$, $k_s = 0.25$, and $\alpha = 32$. The renderer computes the power $\alpha = 32$ by five squarings, so no transcendental function runs for a sample (Section 5.4).

These choices of this design have no source:

- The shading constants. $k_a + k_d = 1$, so a point that faces the light keeps its colour before the specular term.
- The specular term in the form $k_s \rho (1 - c)$, added after the ambient and diffuse terms. In exact arithmetic no channel then leaves the range 0 to 1, so the form needs no clamp. The form of the Phong model with a white specular term, $c (k_a + k_d \delta) + k_s \rho$, needs a clamp at 1, which clips channels of the default fill colours and turns a white fill into a flat white patch at the highlight.
- The two comparisons with 0, and reading $k_a$ and $k_d$ as multiples of the colour of the sample, which the recorded formula does not state.

A black border gets a grey highlight near the specular peak, because $c = 0$ gives $c' = k_s \rho$.

RND-29. For a spherical domain with `Shading::Phong`, the renderer MUST shade every sample that is not an outside sample by the formula of this section, with these shading constants, before the samples of a pixel are averaged, and MUST compute the specular power by five squarings, because the golden images fix the shaded colour of every sample, and a transcendental function per sample would bring the platform mathematics library into every pixel. With `Shading::Flat` it MUST leave the colour unchanged.

### 5.11. Rotation Domains

For a signature without reflections, the domain design builds a rotation domain: the double of the reflection domain of its starred signature across the doubled wall, with the reflection domain left in place (Section 6.8 of the domain design, DOM-23 to DOM-26). That section states the sides of the double, their partners, their pairings, which are products of two reflections, and their kinds. Every pairing of a rotation domain preserves orientation. Where a corner of order 2 lies on the doubled wall, two sides of the double lie on one wall and meet at a vertex of angle $\pi$.

The renderer chooses the view from `geometry()` of the domain (RND-23), as for every domain. The rotation domain has the centre of its reflection domain, so the canonical view of a rotation signature is the view of its starred signature, and `732` and `*732` show the same tiling in the same place. The owner decided this view (`DEC-12`). The view therefore needs no change for a rotation domain.

Two signatures without reflections, `o` and `xx`, are not built as doubles, and `2222` with a skew other than 0 is its double sheared, which is not a union of two rectangles of `*2222` (`DEC-52`). Section 5.14 states them as wallpaper domains. Every step of this section reads the same for them, except that the borders of `2222` are a subset of those of `*2222` at a skew of 0 only.

The steps of Sections 5.1 to 5.10 read as this table states:

| Step | Reading for a rotation domain |
| --- | --- |
| 5.1, 5.2, 5.8, 5.9 | Unchanged, by the geometry of the domain |
| 5.3 | Unchanged (RND-31). A step applies a pairing that preserves orientation, so the reversal count stays 0. The step counts of a point and of its image under a pairing need not differ by 1 |
| 5.4 | Unchanged, over the sides of the rotation domain only (RND-34) |
| 5.5 | Unchanged. `Fill::Parity` gives its first colour to every tile (RND-33), and `Fill::Cosets` colours by cosets as on a reflection domain (Section 5.12) |
| 5.6, 5.10 | Unchanged |

RND-31. The renderer and `fold` MUST fold a point of a rotation domain by the steps of Section 5.3, with no branch on the kind of a side, with the tolerance constant of Section 6.2, and with the order of operations of RND-26, because the loop already reads each pairing as a matrix and each orientation as a flag, and a second fold for rotation domains would be the second implementation that RND-7 forbids.

RND-32. The support of rotation domains MUST NOT change any decision, colour, or statistic that the renderer and `fold` give for a reflection domain, because the committed golden images and the exact tests of bands, scale, and periodicity fix every decision of those domains, and `just update-golden` rewrites every golden file, so a changed image of a reflection domain could enter the repository in the change that adds the new golden files. The acceptance is that no committed golden file of a reflection domain changes.

RND-33. For a domain with no side whose orientation is `Reversing`, `Fill::Parity([c_0, c_1])` MUST give $c_0$ to every converged sample that is not a border sample, and `Renderer::new` MUST NOT refuse such a fill, because the parity of every fold of such a domain is even, so the rule of Section 5.5 already gives $c_0$, and an error for a fill that the library can draw would refuse in the library a choice that belongs to the command line (`DEC-32`).

RND-34. The renderer MUST evaluate the condition of RND-9 over the sides of the domain only, so that the doubled wall, which lies inside a rotation domain, draws no border, because the tiles of a rotation signature are copies of its rotation domain (Section 10.1 of the architecture), and a border on the doubled wall would draw the tiling of the reflection domain. Both sides that lie on one wall at a vertex of angle $\pi$ draw that wall.

The estimate of RND-9 tests the whole wall of each side, not only the side itself. This is correct for a domain whose sides lie on its supporting walls, which holds for a convex domain, and the convexity of the double is part of `UNV-31`.

The rotation steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.11 | The double is a fundamental domain of the group of its pairings | Theorem 12 of [SOLOMYAK2020] and Theorem 6.14 of [SERIES2013] for the hyperbolic plane (Sections 4.1 and 4.2 of the domain reference), and Sections 4.9 and 4.10 of the domain reference for the Euclidean plane and the sphere | Sourced theorems. That the double meets their hypotheses is `UNV-31`, and the lune double is `UNV-30` |
| 5.3 | The fold terminates with pairings that are not reflections | None | `UNV-2` |
| 5.4 | The border estimate | The gradient form of Section 5.7 | `UNV-15` |
| 5.5 | One fill colour for a group whose pairings all preserve orientation | None | A consequence of Section 5.5, and `DEC-32` |

### 5.12. Coset Fill

A coset fill colours the tiling by a colouring of `escher-group`. Each tile takes the colour of its coset of the subgroup of the colouring, a subgroup of finite index $k$ of the group that the side presentation of the domain presents (Section 5.4 of the domain design). `escher-group` searches the colourings, gives each its id, and owns `colour_of`, the rule that maps a side sequence to a coset colour (GRP-37 and GRP-38 of the group design). This crate records the side sequence of each sample and calls `colour_of` once for the sample, so the rule has one implementation. The rule is the proposed default `DEC-40`, and the coset fill with its side buffer is `DEC-45` (Section 10.2 of the architecture).

`CosetFill::new` takes the domain, the colouring by value, and one paint for each coset, a solid colour or an image fill (Section 5.20). It builds the side presentation of the domain and checks four conditions, in this order:

1. The domain has a side presentation.
2. The coset table of the colouring has one generator for each side of the domain.
3. Every relator of the side presentation returns each coset of the colouring to itself.
4. The number of paints equals the colour count $k$ of the colouring.

RND-36. `CosetFill::new` MUST return an error of the kind `InvalidOption` with the text of Section 7.2 for the first of these conditions that fails, and MUST otherwise keep the side presentation, the colouring, and the paints, because a domain without a side presentation has no generators to colour, a colouring with another number of generators or a relator that moves a coset belongs to another presentation, and a list of colours of another length leaves a coset without a colour or a colour without a coset.

Condition 3 refuses a colouring of another domain with the same number of sides, such as a colouring of `*642` with the domain of `*732`. It also refuses a colouring of the side presentation of the same domain with two sides renumbered, unless its table equals that of a colouring of the domain.

The check traces the base of each relator once from each coset with `CosetTable::trace`, which gives a permutation of the cosets. It then follows each cycle of that permutation and requires the exponent of the relator to be a multiple of the cycle length, so that the relator returns each coset to itself. For each relator it reads at most the number of cosets times the length of the base from the table, plus the square of the number of cosets for the cycles. A colouring has at most 64 cosets, and the side presentation of a built domain has at most 16 relators, each base of at most six letters (Section 16 of the domain design), so the check reads at most $16 \cdot 64 \cdot 6 + 16 \cdot 64^2$, about 72,000, table entries. At the input limits of `escher-group`, which are 1024 relators with bases of 4096 letters, the bound is about $2.7 \times 10^8$ reads.

RND-41. `Renderer::new` MUST return an error of the kind `InvalidOption` with the text of Section 7.2 for a `Fill::Cosets` whose `CosetFill` was built for another side presentation than that of the domain, and for a `Fill::Cosets` with an image fill on a domain that has no mask polygon (RND-63), and MUST give no error for every other fill except the Wythoff fill and the whole-face fill of another domain (RND-68, RND-76), because the colouring of such a fill does not colour the tiles of the domain, an image fill takes its value through the mask map of the domain, and `Renderer::new` is the one operation that sees both the fill and the domain. It compares side presentations, not domains, so it accepts a domain with the same side presentation, such as `*2222` with another `aspect`, which shares every colouring.

RND-37. For `Fill::Cosets`, the renderer MUST give a converged sample that is not a border sample the paint at the index that `colour_of` of the colouring returns for the side sequence of its fold, and MUST compute the coset colour through `colour_of` only, because `colour_of` is the one implementation of the rule (GRP-38 of the group design), and a second trace in this crate could read the sequence in another order. RND-36 and RND-41 ensure that the domain has the side presentation of the colouring, so every side sequence of a converged fold has a coset colour below the colour count. The renderer therefore has no fallback colour for a sample without a coset colour. It states the invariant as an `expect` whose message starts with `internal error:`. No input reaches that panic, so the renderer meets RND-14.

The domain itself takes coset colour 0, because a point inside it folds in zero steps, and `colour_of` gives coset 0 for the empty side sequence (GRP-38). The renderer computes the coset colour after the border estimate, so a border sample runs no trace.

The rule reads the side sequence as a word in the generators of the side presentation, with the generator of side $i$ for $P_i^{-1}$ (DOM-32 of the domain design). The first side of the sequence acts last in the element of the tile, so the coset of a sample cannot be carried forward through the fold as one coset number, and the renderer records the sequence and traces it after the fold. The alternative is to carry a permutation of the $k$ cosets through the fold, which SymmHub does (Section 8.4 of the colourings reference) and which costs $k$ entries per step.

RND-38. For a coset fill, the renderer MUST record the side sequence of each sample in a side buffer that each call of `render_rows` or `render` allocates once, clears before each sample, and grows to at most the fold cap, with one byte for each side index, and MUST carry no value from one sample to the next, because the colour of a sample must depend only on its own fold (RND-3), and the command line expects the same bytes for every split into bands and every number of threads (CLI-23 of the command-line design). For `Fill::One` and `Fill::Parity`, the renderer MUST pass the fold a visitor that does nothing.

A side index fits one byte, because a domain with a side presentation has at most 256 sides (DOM-35 of the domain design), and every domain that `build_with` returns has at most 14, the double of a reflection polygon with 8 sides. RND-41 ensures that a coset fill renders only a domain with a side presentation, so the renderer states that the index fits a byte as an `expect` with an `internal error:` message, as RND-37 does, and never truncates an index.

The side buffer is one `Vec<u8>` for each call, which grows by doubling. Its capacity is therefore at most the next power of two at or above the largest fold of the call, and at least 8 bytes. At the fold cap of 100000, the largest of Section 4.1, it is at most 131072 bytes. For `Fill::One` and `Fill::Parity` the buffer is empty, and an empty `Vec` allocates nothing. The command line renders in bands of one row, so it allocates one buffer for each row. The trace adds a chain of dependent table reads, one for each step of the fold, after the fold itself.

RND-39. The coset fill MUST NOT change any decision, colour, or statistic that the renderer and `fold` give with `Fill::One` or `Fill::Parity`, and MUST NOT change any golden file of Section 8.4 that does not name a colouring, because the committed golden images and the exact tests of bands, scale, and periodicity fix every decision of those fills, and `just update-golden` rewrites every golden file, so a changed image could enter the repository unnoticed. The acceptance is that no committed golden file of `Fill::One` or `Fill::Parity` changes.

The coset fill works the same for a rotation domain. Its pairings are not involutions, so the generator of a paired side is not its own inverse, and only a rotation domain tells the trace through the inverse columns from the trace through the generator columns. A trace without the inversion gives a perfect colouring on every domain, which on a rotation domain is the mirror image of the correct one, the colouring of another subgroup. A test that checks only that the colouring is perfect therefore passes with that defect, and the tests assert the exact permutation of the colours by each side (Section 8.1).

Two colourings reproduce the other fills. The colouring `1a` with one colour gives every converged sample that is not a border sample that colour, so its image equals that of `Fill::One` of the colour, byte for byte. The reversal parity colouring is the colouring with two colours in which side $i$ moves coset 0 exactly when side $i$ reverses orientation. Its trace gives, sample by sample, the parity of the reversals of the fold, so its image with two colours equals that of `Fill::Parity` with the same colours, byte for byte, on a reflection domain. The tests assert both equalities, and find both colourings by their tables. On a domain whose sides all reverse orientation, the reversal parity colouring moves every generator off coset 0, so it is valid (GRP-42 of the group design), and the command line names it `parity` (Section 4.6 of the command-line design).

For a spherical domain, the colour of the coset fill is shaded as Section 5.10 states, like every other fill colour.

The coset fill rests on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.12 | The colouring by the cosets of a subgroup of finite index is perfect | Lemma 2.1 of [FRETTLOH2008], for the triangle tilings $T_{p,q}$ under the reflection groups $G_{p,q}$ only (Sections 4.1 and 4.2 of the colourings reference) | Sourced for those tilings. For every other group, `UNV-17` |
| 5.12 | The side sequence of a fold, read as a word in the generators of DOM-32, gives the element that maps the domain onto the tile of the sample | None | `UNV-17`. The tests of the exact permutation check it numerically on reflection and rotation domains |
| 5.12 | The coset colour is the trace of the side sequence in reverse order through the inverse columns, from coset 0 | SymmHub's code comment only (Section 8.4 of the colourings reference) | `UNV-17`. GRP-38 of the group design owns the rule |

### 5.13. Hyperbolic Reflection Polygons

The domain design builds the reflection polygon of a compact hyperbolic `*p1...pn` with 4 to 8 corners, at the shape that the settings of its parameters give, and the rotation domain of `p1...pn` as the double of that polygon at its default shape (Section 6.10 of the domain design with DOM-38 to DOM-45, Section 6.8 of the domain design with DOM-47, and `DEC-46` to `DEC-50`). A reflection polygon has $n$ sides of kind `Mirror`, and its double has $2n - 2$ sides, paired by rotations. `build_with` returns a polygon or the double of one only when it passes the whole of `verify()` (DOM-44 of the domain design, `DEC-48`).

The renderer draws them with no change to its code. Every step of Sections 5.1 to 5.12 already reads the domain as a list of sides of any length: the fold loops over `sides()` and takes the first violated side, the border estimate loops over every side, the coset fill records a side index in one byte, and the view depends only on the geometry and the centre of the domain. Only the symmetric view of the tests assumes three sides, because it reads vertex 2 of a triangle (Section 8.1). The canonical view places the centre of the polygon at the centre of the image and vertex 0 on the positive first axis, as `DEC-49` places it.

The steps of Sections 5.1 to 5.12 read as this table states:

| Step | Reading for a reflection polygon and its double |
| --- | --- |
| 5.1, 5.2 | Unchanged, by the geometry of the domain |
| 5.3 | Unchanged (RND-7). A step costs one side test for each side up to the first violated side, so the cost of a step grows with the number of sides |
| 5.4 | Unchanged, over the sides of the domain (RND-9, RND-34) |
| 5.5, 5.6 | Unchanged |
| 5.11 | Unchanged for the double, which is a rotation domain |
| 5.12 | Unchanged. The side presentation of the polygon gives its colourings (DOM-33 of the domain design) |

Because the code does not change, the polygons change no decision, colour, or statistic of any other domain. The acceptance is that no committed golden file of another domain changes, as for RND-32.

A long side lies far from the centre, so its unit normal has large components, which the tests print for each shape of the polygon domain set. The expected cause of a loss of accuracy on such walls, which no test measures, is that the rounding of a reflection grows with the components of its normal. The settings, which are untrusted, change the walls, so Section 6.9 states the accuracy of the polygons for the shapes of the polygon domain set only (RND-42).

The polygon steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.13 | A convex compact hyperbolic polygon with the angles $\pi/p_i$ is a strict fundamental domain of the group of its reflections | Theorem 6.4.3 of [DAVIS] (Section 4.5 of the domain reference) | Sourced. The guard of DOM-44 of the domain design checks its hypotheses numerically on each built polygon (Section 6.11 of the domain design) |
| 5.13 | The polygon that the construction builds has the corner orders of the signature, in its order | None | `UNV-6`, `UNV-7`, and `UNV-27`. The checks of `verify()`, which the guard of DOM-44 runs, refuse a polygon that misses them |
| 5.13 | The double of a polygon is a fundamental domain of the group of its pairings | As for the doubles of Section 5.11 | `UNV-31` |

### 5.14. Wallpaper Domains

The domain design builds the domains of the nine remaining Euclidean groups, `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, and the domain of `2222` at every value of its parameter `skew`, sheared when `skew` is not 0 (Section 6.13 of the domain design with DOM-50 and DOM-51, `DEC-51` and `DEC-52`). These are the wallpaper domains. Their sides are mirrors, or are paired by translations, by half-turns, by the rotations of a kite, or by glide reflections, and the domains of `**`, `*x`, `22*`, `2*22`, `4*2`, and `3*3` hold mirror sides beside paired sides. `build_with` returns a wallpaper domain only when it passes the whole of `verify()` (DOM-54 of the domain design, `DEC-53`).

The renderer draws them with no change to its code. The fold, the border estimate, the parity fill, and the coset fill read a side only through its covector, its pairing matrix, and its orientation, and none of them reads the kind of a side. A glide reflection is a 3 by 3 matrix whose last row is exactly $(0, 0, 1)$, with the orientation `Reversing`, so the last coordinate of a point stays exactly 1 at every step, as Section 6.6 states for the other Euclidean pairings.

RND-43. The renderer and `fold` MUST fold a point of a wallpaper domain by the steps of Section 5.3, with no branch on the kind of a side, and MUST count a step across a side whose pairing is a glide reflection as a reversal, because the loop reads each pairing as a matrix and each orientation as a flag (RND-31), and a glide reflection reverses orientation, so a fold that did not count it would give the parity fill the wrong coset of the orientation-preserving subgroup.

The steps of Sections 5.1 to 5.12 read as this table states:

| Step | Reading for a wallpaper domain |
| --- | --- |
| 5.1, 5.2 | Do not run. The point and the screen tangents are those of Section 5.8 |
| 5.8 | Unchanged. The view applies no isometry, and the centre of the domain lies at the centre of the image |
| 5.3 | Unchanged (RND-43). A step costs one side test for each side up to the first violated side, of at most six sides |
| 5.4 | Unchanged, over every side of the domain (RND-44) |
| 5.5 | Unchanged. `Fill::Parity` gives its first colour to every tile of `o` and of `2222`, which have no reversing side (RND-33), and colours the tiles of the other eight by the parity of the reversals |
| 5.12 | Unchanged. The side presentation of DOM-52 of the domain design gives the colourings |

The canonical view is the plane view of Section 5.8. The domain design places the centre of each wallpaper domain other than a kite exactly at the origin. That is the centre of its rectangle or parallelogram, or of its generating rectangle for `2222` at every value of `skew`. A kite has the centre of its generating triangle, which is the origin up to rounding (`DEC-12`). So the kites of `4*2` and `3*3` lie where the triangles of `*442` and `*632` lie, as the kites of `442` and `632` do. The centre is not the mean of the vertices for `2*22`, whose five vertices have the mean $(0, -h/10)$ for the height $h$ of its rectangle, or for `2222` at every value of `skew`, whose second rectangle lies to the right of the first. The shear of `2222` fixes the origin, so a skew does not move the view.

RND-44. The renderer MUST evaluate the condition of RND-9 over every side of a wallpaper domain, mirror or paired, with one border width and one border colour, because a tile is a copy of the domain, so its whole boundary is drawn, as RND-34 draws it for a rotation domain (`DEC-55`). Two sides that lie on one wall at a vertex of angle $\pi$ both draw that wall, as RND-34 states.

The tiles of several groups and their borders coincide, and a parity fill sees only the orientation-preserving subgroup of a group. So several groups share a picture. This is a property of this design, and not a defect:

- With `Fill::Parity`, `**` and `22*` give the same image, and `*x`, `22x`, and `*2222` give the same image, at the same `aspect`.
- With `Fill::One`, the six rectangular groups give the image of `*2222` at the same `aspect`, `4*2` gives the image of `442`, and `3*3` the image of `632`.

The tests of `escher-cli` assert through the command line, at the default `aspect`, those of these equalities that its colourings reach (Section 4.6 of the command-line design), except the one-colour images of the six rectangular groups. No test of the crates asserts that the one-colour image of a rectangular group equals the image of `*2222`, so that equality rests on a measurement outside the repository. Of the six rectangular groups, only `xx` and `2*22` have parity images of their own. This has two consequences:

1. A user tells these signatures apart by their colourings by id, and later by a motif, and not by the default picture (Section 4.5 of the command-line design).
2. A golden image of the parity fill cannot tell these groups apart, and a test that every border pixel of one of the six rectangular groups is a border pixel of `*2222` asserts an equality that holds for any of them. A domain built for the wrong group of the same class, such as `22x` with the pairings of `*x`, would pass both. So each of the nine groups has a golden image of a colouring whose picture no other of them gives (Section 8.4), a test asserts that the nine pictures differ, and the border-subset test of the wallpaper domains keeps only the inclusions of the kites, which are proper.

Whether mirror sides are drawn differently from paired sides is point 40 of Section 12.3. Until the owner decides, the renderer draws them alike.

The parity fill colours each tile by its coset of the orientation-preserving subgroup (`UNV-18`). The tests assert that the image of `Fill::Parity` equals byte for byte the image of the coset fill of the reversal parity colouring of Section 5.12. An id depends on the side presentation of its domain (GRP-37 of the group design), so equal pictures, such as those of `**` and `22*`, can have different ids, and the tests find the reversal parity colouring by its table and not by its id.

A point outside a side that is split at a vertex of angle $\pi$, such as side 0 or side 1 of `22*`, violates both sides on that wall, and the fold applies the first of them in index order. Both carry the same half-turn, and their generators are equal in the group, so the coset colour does not depend on which of them the fold applies. `2222` behaves the same way.

The wallpaper steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.14 | A convex polygon whose sides are paired by isometries, with the conditions on its vertex cycles, is a fundamental domain of the group of its pairings | Theorem 4.14 of Epstein and Petronio, whose hypotheses `verify()` checks (Section 4.9 of the domain reference) | Sourced theorem. That each construction gives the group of its signature is `UNV-11`, and the pairings that no source states are `UNV-33` |
| 5.3 | The fold terminates with translations, half-turns, rotations, and glide reflections as pairings | None | `UNV-2`. No argument exists for the kites |
| 5.4 | The border estimate | The gradient form of Section 5.7 | `UNV-15` |
| 5.5 | The parity colouring, with glide reflections among the reversing sides | Section 4.6 of the rendering reference | `UNV-18` |
| 5.12 | The coset fill | Section 5.12 | `UNV-17` |

### 5.15. Spherical Group Domains

The domain design builds the domains of the remaining spherical groups `3*2`, `2*n`, `n*`, `nx`, `*`, `x`, and `1` (Section 6.16 of the domain design with DOM-58 and DOM-59, `DEC-56` to `DEC-62`). These are the spherical group domains:

| Signature | Domain | Sides |
| --- | --- | --- |
| `3*2` | The kite of the double of `432` | Sides 0 and 1 are mirrors, and sides 2 and 3 are paired by the rotation through $2\pi/3$ |
| `2*n`, $n$ up to 50 | The kite of the double of `(2n)22` | Sides 0 and 1 are mirrors, and sides 2 and 3 are paired by a half-turn |
| `n*`, $n$ up to 100 | The kite of the double of `n22` | Sides 0 and 1 are paired by the rotation through $2\pi/n$, and sides 2 and 3 are mirrors, the two halves of one side on the equator |
| `nx`, $n$ up to 100 | The lune of `*nn` | Sides 0 and 1 are paired by rotary reflections |
| `*` | The lune of order 1, a hemisphere | Two mirror sides on one great circle |
| `x` | The lune of order 1 | Two sides on one great circle, paired by the antipodal map |
| `1` | The whole sphere | No side |

`build_with` returns each of them except `*` only when it passes the whole of `verify()` (DOM-60 of the domain design, `DEC-60`). `*` is the lune of `*nn` at order 1, which is not guarded.

The renderer draws them with no change to its code. The fold, the border estimate, the parity fill, and the coset fill read a side only through its covector, its pairing matrix, and its orientation, and none of them reads the kind of a side. A rotary reflection is an orthogonal matrix with the orientation `Reversing`, so it is one more pairing to the loop of Section 5.3.

RND-46. The renderer and `fold` MUST fold a point of a spherical group domain by the steps of Section 5.3, with no branch on the kind of a side, and MUST count a step across a side whose pairing is a rotary reflection, including the antipodal map, as a reversal, because the loop reads each pairing as a matrix and each orientation as a flag (RND-31), and a rotary reflection reverses orientation, so a fold that did not count it would give the parity fill the wrong coset of the orientation-preserving subgroup.

The steps of Sections 5.1 to 5.12 read as this table states:

| Step | Reading for a spherical group domain |
| --- | --- |
| 5.1, 5.9 | Unchanged. The view is the ball view, chosen from the geometry (RND-23) |
| 5.3 | Unchanged (RND-46). A step costs one side test for each side up to the first violated side, of at most four sides. The domain of `1` takes no step (RND-49) |
| 5.4 | Unchanged, over every side of the domain (RND-47) |
| 5.5 | Unchanged. `Fill::Parity` colours the tiles by the parity of the reversals, and gives its first colour to every sample of `1`, which has no reversing side (RND-33) |
| 5.10 | Unchanged |
| 5.12 | Unchanged. The side presentation of Section 5.4 of the domain design gives the colourings. The domain of `1` has only the colouring `1a` |

RND-47. The renderer MUST evaluate the condition of RND-9 over every side of a spherical group domain, mirror or paired, with one border width and one border colour, because a tile is a copy of the domain, so its whole boundary is drawn, as RND-44 states for a wallpaper domain (`DEC-55`, `DEC-61`). Two sides that lie on one wall both draw that wall.

The domain of `1` has no side, so the loop of Section 5.3 finds no violated side at its first test. The border condition of RND-9, which asks for some side, then holds for no sample.

RND-49. For a domain with no side, `fold` MUST return every point that passes the checks of step 2 of Section 5.3 as converged in zero steps, the border estimate MUST mark no sample as a border sample, and `Renderer::new` MUST accept the domain with every fill that it accepts for another spherical domain, because the trivial group has one tile, the whole sphere, which no wall bounds, so the ball of `1` is the fill colour of coset 0, shaded, and an error or a special case would refuse in the library a domain that the loop already draws (`DEC-58`).

The canonical view of a spherical group domain is the ball view of Section 5.9, with the centre of the domain at the front of the ball. The kites keep the placement of the doubles that they reuse, so `3*2` and `2*n` lie where `*432` and `*(2n)22` lie, and `n*` where `*n22` lies. The lunes keep the placement of the lune of `*nn`. The lune of order 1 and the domain of `1` have the centre $(0, 0, 1)$ (`DEC-12`).

Three default pictures are degenerate. In the canonical view, `*` and `x` show one tile, the hemisphere that faces the viewer, with the border on its silhouette, and `1` shows a plain shaded ball, which differs only slightly from `x` with `Fill::One`. With a viewing rotation, `*` and `x` show two hemispheres, still equal byte for byte. The design keeps the canonical view, `--rotation` turns the ball, and point 42 of Section 12.3 puts a tilted default view to the owner (`DEC-61`). The tests of these domains use a viewing rotation where the canonical view would test nothing.

Some sides share a wall: the two mirror halves of `n*`, both sides of `*`, both sides of `x`, sides 0 and 1 of `2*`, which is `n*` with $n = 2$, and sides 2 and 3 of `2*5`, which a half-turn pairs. The tests assert the pairs of sides on one wall for these domains and for `3*2`, `5x`, and `1` only, so the design claims the shared wall of a half-turn pair for `2*5` and for no other `2*n`. A point outside such a wall can violate both sides, and the fold applies the first of them in index order, as for the split sides of the wallpaper domains (Section 5.14). The design relies on three statements here:

1. The generators of two sides on one wall are equal in the group, because the side presentation holds relators that make them so (Section 5.4 of the domain design). So the coset colour does not depend on which of them the fold applies.
2. The two sides have the same orientation, so the parity does not depend on it either.
3. Their walls and pairings are equal up to rounding, so the folded points differ by rounding only. For `*` and `x`, the normals are equal bit for bit, because the lune of order 1 has exact normals (`DEC-62`). For the other pairs, they are not, and the tests print the differences. The design therefore claims no single wall for them.

The tiles of several groups and their borders coincide, so several groups share their default picture, as `DEC-55` accepts for the wallpaper groups. This is a property of this design, and not a defect:

- With `Fill::Parity`, `nx` gives the image of `*nn`, and `x` the image of `*`.
- With `Fill::One`, `n*` gives the image of `n22`, `2*n` the image of `(2n)22`, and `3*2` the image of `432`.

The tests find every such pair equal byte for byte in the canonical view and in two rotated views. A measurement outside the repository tried the orders 2, 3, and 5 for `n*` and 1, 3, and 5 for `nx`. For these orders, every colouring of `n*` gave the picture of a colouring of `n22`, and every colouring of `nx` the picture of a colouring of `*nn`. The measurement tried no other order, so the design states this finding for these orders only. Colourings that no related group gives exist for `2*n` and `3*2`, such as the colourings of `2*5` and `3*2` that the coset golden images of Section 8.4 fix. This has two consequences:

1. For these orders, a user cannot tell `n*` from `n22`, or `nx` from `*nn`, by any picture of the proof of concept, only later by a motif. `3*2` and `2*n` differ from their related groups by a colouring id.
2. No golden image or test of this crate can catch a domain of `n*` or `nx` built for the wrong group of the same order. The certificate of DOM-64 of the domain design, with its census of the reversing elements and the reflections, carries that burden, and the golden images cover `3*2` and `2*5` only (Section 8.4).

The parity fill colours each tile by its coset of the orientation-preserving subgroup (`UNV-18`). The domain of `1` has no reversing side and so no parity colouring. The tests therefore assert the colouring `1a` on every spherical group domain that they render, and the reversal parity colouring, found by its table and not by its id, on every one whose domain has a reversing side.

The spherical group steps rest on these statements:

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.15 | Each construction gives a fundamental domain of the group of its signature | None for the domains and their pairings. The orbifold of `3*2` (p. 187 of [CBG]), the orbifold of `*` (p. 248 of [CH]), and the presentation of `n*` (p. 130 of [CBG]) are sourced (Section 6.16 of the domain design) | `UNV-11` and `UNV-34`, and items 19 and 20 of Section 13.3 of the domain reference. The guard of DOM-60 and the certificate of DOM-64 of the domain design check each build |
| 5.15 | Every orientation-reversing isometry of the sphere is a rotary reflection | None that names a rotary reflection. Theorem 5.4.2 of [RAT] states it in matrix form | Item 33 of Section 15.1 of the geometry reference. The renderer reads only the orientation flag |
| 5.3 | The fold terminates with rotations, half-turns, mirrors, and rotary reflections as pairings | None | `UNV-1` for `*`, and `UNV-2` for the other domains. No argument exists |
| 5.3 | A domain with no side has one tile | None | A consequence of Section 5.3 (RND-49) |
| 5.4 | The border estimate | The gradient form of Section 5.7 | `UNV-15` |
| 5.5 | The parity colouring, with rotary reflections among the reversing sides | Section 4.6 of the rendering reference | `UNV-18` |
| 5.12 | The coset fill | Section 5.12 | `UNV-17` |

### 5.16. Hyperbolic Models

The option `model` chooses the hyperbolic model of a hyperbolic domain. `HyperbolicModel::Disk`, the default, draws the Poincare disk by Sections 5.1 and 5.2. `HalfPlane`, `Band`, and `Klein` draw the upper half-plane, the band, and the Klein disk by this section. Every model leads to the same point of the hyperboloid and the same screen tangents, so the fold of Section 5.3, the border estimate of Section 5.4, and the colour of Section 5.5 run unchanged after the lift.

RND-50. With `HyperbolicModel::Disk`, the renderer MUST compute the sample positions, the disk positions, the points, and the screen tangents of a hyperbolic domain by the operations of Sections 5.1 and 5.2 and no other, because the golden images of the hyperbolic domains and the snapshots of `escher-cli` fix every byte of the images of the disk, and another order of operations, such as a multiplication by an identity Jacobian, can move a sample across a border.

For the other models, the renderer maps each sample in these steps:

1. It computes the screen point $v = \left(\left(p_1 - \frac{W}{2}\right) h, \ \left(\frac{H}{2} - p_2\right) h\right)$ of the sample, with the sample position $p$ and the scale $h$ of the model in the table below. For the Klein model, $v$ is the disk position of Section 5.1 under another name, computed with the same operations.
2. It computes the model position $w$ from $v$ as the table below states.
3. It calls `hyperbolic::disk_from_half_plane`, `disk_from_band`, or `disk_from_klein` on $w$, which returns the disk position $z$ and the model Jacobian $J$, or `None`. On `None` the sample is an outside sample (GEO-28 of the geometry design).
4. It maps $z$ to the point $x =$ `hyperbolic::from_disk(z)`. On `None` the sample is an outside sample, as in Section 5.2.
5. It computes the screen tangents $T_k = h \left(J_{1k} D_1 + J_{2k} D_2\right)$ for $k$ of 1 and 2, where $D_1$ and $D_2$ are the partial derivatives of `from_disk` at $z$, which are $T_1 / h$ and $T_2 / h$ of Section 5.2 at the disk position $z$. Section 10 records these tangents as a statement of this design.

RND-51. For the half-plane, the band, and the Klein model, the renderer MUST compute the model position, the disk position, the point, and the screen tangents of each sample by the steps of this section, MUST take the disk position and the model Jacobian from `escher-geometry` only, and MUST treat a sample whose model position or disk position is rejected as an outside sample, because the golden images of Section 8.4 fix the position of every sample in each model, and a second copy of a model map in this crate could differ from the one that the tests of `escher-geometry` check.

RND-52. The fold, the border estimate, and the colour of a sample MUST NOT read the hyperbolic model, because the model changes only the point and the screen tangents with which a sample enters the fold, and a branch on the model after the lift would give the borders of one model a rule of their own (`DEC-28`, `DEC-74`).

The border estimate is RND-9 for every model. The model Jacobian carries the stretch of the model into the screen tangents, so the gradient size $g_i$ is the change of the side value per pixel of that model. This holds for the Klein model, whose metric differs by direction (Section 3.1 of the geometry reference), because the estimate uses the two screen tangents and no scale factor, so the Klein model needs no case of its own. What changes for a model is only the start value of the screen tangents. The walls of the Klein model are chords ([CFKP], Section 9, Section 3.1 of the geometry reference), so its walls are straight lines in the image, which the test "Straight borders" of Section 8.1 checks.

The canonical view of each model is the owner's decision `DEC-76`:

| Model | Scale $h$ | Model positions of the image | Centre of the image | Edges of the image | Background |
| --- | --- | --- | --- | --- | --- |
| Disk | The disk scale of Section 5.1 | The disk positions of Section 5.1 | The centre of the disk | A margin $m$ on the shorter side | Outside the unit circle |
| Half-plane | $2/H$ | $w = v + (0, 1)$ | The point $i$, the image of the centre of the disk (Section 7.5 of the geometry design) | The real axis lies on the bottom edge | Only where `from_disk` rejects the disk position |
| Band | $2/H$ | $w = v$ | The band position 0, the image of the centre of the disk | The lines $w_2 = \pm 1$ lie on the bottom and top edges | Only where `from_disk` rejects the disk position |
| Klein | The disk scale of Section 5.1 | $w = v$ | The centre of the Klein disk, the image of the centre of the disk | A margin $m$ on the shorter side | Outside the unit circle |

In the half-plane and the band, one model unit spans half the image height whatever the width, so a wider image shows more of either model along the first axis. The centre of the domain lies at the centre of the image in every model, because the domain design places it at `hyperbolic::ORIGIN` (Section 6.3 of the domain design), which `from_disk` gives for the disk position 0. The picture of the half-plane sits the same way up as the picture of the disk (`DEC-75`), an orientation that Section 7.5 of the geometry design states and tests.

The fold cap and the colour of an unconverged sample do not change with the model (RND-10). A sample close to the real axis and far from $i$ in the half-plane, or far along the strip in a wide image of the band, has a disk position close to the unit circle. Where that disk position rounds onto the unit circle or is not finite, `from_disk` rejects it in step 4, so the sample is an outside sample. Section 6.12 states where the renderer is accurate.

The band evaluates the complex hyperbolic tangent for each sample, through `num-complex` in `escher-geometry`, and the half-plane a complex division. The other steps cost what the steps of Section 5.2 cost.

### 5.17. Intrinsic Border

The option `border_unit` chooses the unit of the border width. `BorderUnit::Pixels`, the default, draws the border of Section 5.4 in output pixels. `BorderUnit::Intrinsic` draws a border whose width is a length in the geometry of the domain, by this section, for every geometry:

| Geometry | Wall distance $d_i$ | Intrinsic unit |
| --- | --- | --- |
| Hyperbolic | `hyperbolic::wall_distance(c_i, x)` | The metric of curvature $-1$ (Section 5.5 of the architecture) |
| Euclidean | `euclidean::wall_distance(c_i, x)` | The normalization length of the domain (`DEC-17`) |
| Spherical | `spherical::wall_distance(c_i, x)` | The radian of the unit sphere (Section 5.5 of the architecture) |

Here $x$ is the folded point and $c_i$ the covector of side $i$. The renderer passes the folded point as the fold returns it, without renormalizing it, and the functions of `escher-geometry` read a hyperbolic or spherical vector as its ray (GEO-32 of the geometry design, `DEC-80`). Section 5.6 of the geometry design states the formulas, their sources, and their unverified parts, and its Sections 8.4 to 8.6 state their accuracy.

RND-54. With `BorderUnit::Intrinsic`, the renderer MUST mark a converged sample as a border sample exactly when, for some side $i$ of the domain, the expression `!(d_i >= half_width)` is true, with $d_i$ the wall distance of the geometry of the domain from the table above and the half width in the intrinsic unit, because the width is the full width of the band, centred on each wall, as the width in pixels of RND-9 is, and the negated comparison makes a NaN wall distance give a border sample rather than a fill colour, as RND-9 does for a NaN side value.

A sample exactly half the width from a wall is not a border sample, as for the pixel unit (Section 5.8). A width of 0 draws no border. The test runs over the sides of the domain only, so RND-34, RND-44, and RND-47 hold for it as they hold for RND-9: the doubled wall of a rotation domain draws no border, every side of a wallpaper or spherical group domain draws its wall whether it is a mirror or a paired side, and a domain with no side has no border sample (RND-49). The wall distance is the distance to the whole wall of a side, which RND-34 states is correct for a convex domain, with `UNV-31` for the doubles.

RND-55. With `BorderUnit::Pixels`, the renderer MUST compute every sample by the operations that it uses without the option, because the golden images of Section 8.4 and the snapshots of `escher-cli` fix every byte of the images in pixels, and a border unit that changes one operation, such as a branch that reorders the comparison of RND-9, can move a sample across a border.

RND-56. The intrinsic border test MUST NOT read the hyperbolic model, the image size, the sample grid, or the screen tangents, because the intrinsic width is a property of the geometry, so a point of the hyperboloid is a border sample in every model and at every image size, or in none, up to the rounding with which each model reaches the point.

Only the point and the screen tangents with which a sample enters the fold depend on the model (RND-52), and the intrinsic border test reads only the folded point. The fold carries the screen tangents with either unit, so it keeps one path. The sample grid and the colour of a pixel are those of Sections 5.1 and 5.6 with either unit: each sample is a border sample or not, and the pixel takes the mean of its samples in linear light. Where a tile is smaller than a pixel, such as near the limit circle of the disk, close to the real axis of the half-plane, far along the band, or near the rim of the ball, an intrinsic border is narrower than the spacing of the samples. There it breaks into separate samples or vanishes, and supersampling does not prevent it. Analytic antialiasing is the roadmap item `M14`.

The fold cap, the colour of an unconverged sample, and the background of an outside sample do not change with the unit (RND-10). For each sample and each side, the intrinsic border test calls one inverse hyperbolic sine or one arcsine of the platform mathematics library, or one absolute value in the Euclidean plane. The bytes of an image with the intrinsic unit therefore depend on that library, as those of the band do, and the allowance of Section 8.4 covers the difference.

The renderer states no bound of its own for the intrinsic border test. Its decisions are those of the fold followed by the functions of `escher-geometry` within their accuracy, and the tests of Section 8.1 compare them with the reference arithmetic away from the boundary of the band.

### 5.18. Reference Tiles

Withdrawn on 2026-11-23, when the owner decided that a mask shows the domain as a flat polygon (`DEC-93`), so no mask or image fill refers to a tile of the tiling. The number stays so that the citations of the later sections hold. RND-57 is withdrawn with it.

### 5.19. Polygon Masks

`Masks::render` draws the polygon mask of one colour $c$: an image of the size given that shows the mask polygon of the domain (Section 5.22) in white on black, with a border along each edge, inside the polygon, in the code colour of that edge (`DEC-94`, `DEC-98`). Every mask of one colouring shows the same polygon in the same place. The masks of two colours differ only in the code colours and the marks of their borders. A mask depends only on the domain, the colouring, the colour, and the size.

The mask placement puts the mask polygon $P$ in an image of $W$ by $H$ pixels. With $S = \min(W, H)$, the margin $m = 1/32$ of Section 5.1, and the bounding box of $P$ with the width $w_P$, the height $h_P$, and the centre $b$, a point $p$ of the plane of $P$ has the image position

$$q = \left(\frac{W}{2} + \sigma (p_1 - b_1), \ \frac{H}{2} - \sigma (p_2 - b_2)\right), \qquad \sigma = \min\left(\frac{W - 2mS}{w_P}, \ \frac{H - 2mS}{h_P}\right)$$

in pixels. The polygon keeps the orientation that Section 5.22 gives it, with its second axis up, and it is centred in the image with at least the margin on every side.

The seam classes of a colouring come from its coset table and the pairings of the domain. Across side $j$, the tiles of colour $c$ can have neighbours of different colours, so $(c, j)$ can have several neighbour colours. Side $p$ of the tile across side $j$ lies on side $j$, with $p$ the partner of $j$, which is $j$ itself for a mirror (Section 5.3 of the domain design). Edge $j$ of colour $a$ therefore meets edge $p$ of every colour $b$ among the neighbour colours of $(a, j)$, and an edge can meet several edges. The seam classes are the classes of the smallest equivalence on the pairs (colour, side) under which two edges that meet are equivalent. Every edge of one seam class takes one code colour in every mask file of the colouring, so the artwork along all of them must agree. For a colouring whose subgroup is normal, each seam class has at most two edges, and the artwork along an edge must agree only with the edge that it meets. That bound is a statement of this design that the test "Seam classes" pins (Section 10).

`Masks::new` finds the pairs of colours that occur across side $j$ by this finite computation on the coset table of the colouring:

1. Start from the pair $(0, b_j)$. Colour 0 is the colour of the domain itself, the empty side sequence of GRP-38 of the group design. $b_j$ is `colour_of(&[j])`, which GRP-38 computes as the coset `table.image(0, Letter::inverse_of(j))`. That this is the colour of the neighbour of the domain across side $j$ is part of `UNV-17`.
2. Close the set of pairs under applying the column of each generator of the table to both members of a pair at once.

The pairs reached are the pairs (colour of a tile, colour of its neighbour across side $j$) that occur, and each pair $(a, b)$ makes edge $j$ of colour $a$ meet edge $p$ of colour $b$. That this closure gives exactly the pairs that occur in the tiling is a statement of this design that the test "Seams" pins (Section 10).

The code colours are evenly spaced hues. The seam classes are numbered from 0 to $N - 1$ in the order of their first edge, by colour and then by side. Class $s$ has the hue value $h = 6s/N$, with $k = \lfloor h \rfloor$ and $f = h - k$, and its code colour has these red, green, and blue values $v$, each stored as the 8-bit sRGB value $\mathrm{round}(255 v)$:

| $k$ | 0 | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- | --- |
| Red, green, blue | $(1, f, 0)$ | $(1 - f, 1, 0)$ | $(0, 1, f)$ | $(0, 1 - f, 1)$ | $(f, 0, 1)$ | $(1, 0, 1 - f)$ |

The code colours depend only on the coset table and the partners of the sides, so every run gives the same masks (Section 5.9 of the architecture). That the $N$ code colours of a colouring are pairwise different, and are neither white nor black, for every $N$ up to 896, which is 64 colours times 14 sides, is a statement of this design that the test "Code colours" pins (Section 10).

The marks show how the edges of a seam class meet, so that marked ends meet marked ends wherever two edges of the class meet. The first edge of a class, in the order by colour and then by side, has its marked end at its start, vertex $j - 1$. When edge $j$ of a class has its marked end at a vertex, an edge $p$ that it meets takes its marked end at the vertex onto which $P_j$ maps that vertex: from vertex $j - 1$, vertex $p$ for a pairing of the kind `Rotation` or `Translation`, and vertex $p - 1$ for the kinds `Mirror`, `GlideReflection`, and `RotaryReflection`, and from vertex $j$ the other end (check 2 of Section 8.2 of the domain design). `Masks::new` carries the marked end in this way from the first edge to every edge of the class.

When the carrying reaches an edge of a class with both ends, as it would for an edge that meets itself reversed or a cycle of meetings that reverses an edge, every edge of that class has a mark at both ends, and the artwork along those edges must match itself reversed. Point 51 of Section 12.3 puts this rule to the owner. That in a class with one mark on each edge every two edges that meet have their marked ends together, and that a class has marks at both ends exactly when the carrying reaches one of its edges with both ends, is a statement of this design that the test "Seam classes" pins (Section 10).

For each sample, `Masks::render` takes these steps:

1. It computes the screen position of the sample by Section 5.1 and the point $p$ of the plane of $P$ by the inverse of the mask placement.
2. When $p$ lies strictly outside the line of some edge, the sample is black.
3. Otherwise it takes the edge $j$ whose line is nearest to $p$, the lowest index at a tie, with the Euclidean distance $\delta_j$. When $\sigma \delta_j < S/64$, the sample lies in the border of edge $j$. It is black when the foot of the perpendicular from $p$ to edge $j$ lies at most a quarter of the length of the edge from a marked end, and it takes the code colour of edge $j$ otherwise.
4. Every other sample is white.

The mask runs no fold. It always uses the 4 by 4 sample grid of Section 5.1, the default of the option `samples` (Section 4.1). `Masks::render` calls a private function of `mask.rs` with this grid, and the unit tests of the masks call the same function with one sample per pixel (Section 8.1). Each pixel takes the mean of its samples in linear light, as Section 5.6 states, so the edges and the borders are antialiased by the sample grid, and the pixel is stored as 8-bit sRGB in the layout of `render_rows`, which `encode_png` encodes.

RND-58. `Masks::render` MUST compute each sample by the mask placement and the steps of this section, MUST give every edge of a seam class the same code colour and its marks at the ends that Section 5.19 gives, and MUST read no render option, because an image fill reads its image through the same placement (Section 5.20), the user matches the edges of the masks by their code colours and marks (`DEC-94`), and a mask that depended on the options of a later render would have to be painted again for each of them.

Two copies of masks laid edge to edge, edge $j$ of colour $a$ on an edge $p$ of colour $b$ that it meets, with their marks together, show the seam of the tiling as the render draws it, because the mask map carries side $j$ and side $p$ with the same parametrization (Section 5.23). The copy is turned for a pairing that preserves orientation and mirrored for one that reverses it, as the marks show. Copies need not close up around a vertex (`DEC-94`). A mask relies on neither `UNV-1`, `UNV-2`, nor `UNV-3`, because it runs no fold and no side test.

### 5.20. Image Fills

A paint of a coset fill is a solid colour, `Paint::Solid`, or an image, `Paint::Image`. An image fill of colour $c$ is a texture on the domain: its value at a point $x$ of the domain is the image sampled at the image position of $M(x)$, the point of the mask polygon that the mask map of Section 5.23 gives, under the mask placement of Section 5.19 at the pixel size of the image (`DEC-88`). A converged sample that is not a border sample and whose coset colour is $c$ takes the value of the texture at its folded point. The value is then shaded on the ball as Section 5.10 states, like every fill colour.

The value depends only on the folded point and the image of the colour, so the render needs nothing beyond the folded point and the coset colour that it already has. Every tile of colour $c$ shows the same picture, moved by the isometry that maps the domain onto the tile. A tile that is a mirror image of another tile of its colour shows the image mirrored, because the folds of mirror points land on the same point of the domain, as the owner decided (`DEC-66`). Along a seam, the textures of the two tiles agree with the two masks laid edge to edge (Section 5.19). The tests "Texture contract", "Mirrored", and "Seams" of Section 8.1 pin this contract.

The mask map is a choice of this design, which the owner left open (`DEC-93`, `DEC-97`). The picture inside a tile depends on it, and only its behaviour on the sides is fixed by the owner's purpose.

`Renderer::new` builds the mask polygon and the mask map of its own domain, and the mask placement of each image fill at the size of its image. RND-41 refuses a coset fill with an image fill on a domain without a mask polygon.

The value of an image fill at a folded point $x$ comes from these steps:

1. Compute $M(x)$ by Section 5.23. A step of the mask map that gives the fallback gives it here.
2. Compute the image position $q$ of $M(x)$ by the mask placement at the width $W$ and the height $H$ of the image.
3. When a coordinate of $q$ is not finite, or $q$ lies outside the rectangle $[0, W] \times [0, H]$, the fallback.
4. Otherwise, sample the image bilinearly at $q$.

Texel $(u, v)$, with column $u$ and row $v$ counted from the top left, has its centre at $(u + 1/2, \ v + 1/2)$. Its linear value $t_{uv}$ is its colour $l$ by `Colour::from_srgb8` composited over the background colour $b$ of the options, $t = \omega l + (1 - \omega) b$ with $\omega$ the alpha byte divided by 255, in linear light for each channel. With $a = q_1 - 1/2$, $u_0 = \lfloor a \rfloor$, $f = a - u_0$, and $v_0$ and $e$ formed in the same way from $q_2$, the value is

$$(1 - f)(1 - e) \, t_{u_0 v_0} + f (1 - e) \, t_{u_1 v_0} + (1 - f) e \, t_{u_0 v_1} + f e \, t_{u_1 v_1}$$

with $u_1 = u_0 + 1$ and $v_1 = v_0 + 1$, where each index is clamped to the texels of the image. The fallback is the background colour $b$ (`DEC-89`).

RND-59. For a sample whose coset colour has an image fill, the renderer MUST compute its value by the steps of this section from the folded point and the image of that colour, and the value MUST NOT depend on the hyperbolic model, the viewing rotation, the size of the render, the sample grid, or the border, because the texture is fixed to the domain, so that a render in another model, at another size, or with another border samples the same texture, and an image painted over a mask fills the tiles as the masks show only through the mask map and the mask placement.

RND-60. The sampling MUST interpolate in linear light the texels composited over the background colour, MUST give the background colour as the fallback, and MUST return a finite colour with each channel in $[0, 1]$ for every folded point, because the crate averages samples in linear light (Section 5.6), an interpolation in another space would blend the texels of one sample differently from the samples of one pixel, and a value outside the range of a `Colour` would break the invariant of Section 3.2.

RND-61. The support of image fills MUST NOT change any decision, colour, statistic, or golden file of a coset fill whose paints are all `Paint::Solid`, because the committed golden images and the exact tests fix every decision of those fills, and `just update-golden` rewrites every golden file, so a changed image could enter the repository unnoticed. The acceptance is that no committed golden file changes.

Bilinear sampling is the simplest filter of SymmHub's three (Section 10 of the rendering reference). It reads four texels for each sample, and an image fill adds the mask map, one test for each side and one distance, for each sample. The renderer computes no level of detail, so where a tile is much smaller than the image, as near the limit circle, the samples alias the image, and the sample grid only averages them.

An image of any size is read with the mask polygon placed in it by the mask placement at that size, as in a mask of that size. The user may therefore paint at any resolution, as long as the polygon sits where a mask of that size shows it, and an image scaled with its aspect ratio kept samples the same texture, up to the resampling. A render whose domain has other shape parameters than the domain of the masks keeps the colours, because its side presentation is the same, but its mask polygon can change, so the painted images no longer match the polygon.

### 5.21. Image Decoding

`Image::decode_png` reads one PNG image from the reader that the caller gives, so this crate opens no file (Section 13). It accepts every colour type of PNG at a bit depth of 8 or less: greyscale, truecolour, indexed colour, greyscale with alpha, and truecolour with alpha, interlaced or not, with or without a tRNS chunk. Each texel holds 8-bit red, green, blue, and alpha: indexed colour and bit depths below 8 are expanded, the tRNS chunk becomes alpha, greyscale fills three equal channels, and an image with neither alpha nor tRNS takes the alpha 255. It refuses an image with 16 bits per channel with `Decoding` (`DEC-90`). The test "Decoding" of Section 8.1 pins the texels of each colour type and bit depth that the crate accepts, so the contract does not rest on what a transformation of `png` does. The crate reads the samples as sRGB whatever colour chunk the file holds, and reads only the default image of an animated PNG.

It takes these steps, in this order:

1. Read the width, the height, and the bit depth from the header of the stream.
2. Check the width and the height, each at most 16384, then their product, at most $2^{26}$, which are the limits of the row `size` of Section 4.1, and then the bit depth, at most 8 (`DEC-92`). `build` and `Masks::render` call the same size check, so a decoded image and an output image cannot have different size limits. A header with a zero side never reaches the check, because `png` refuses it with an error that gives `Decoding`.
3. Allocate the texels and decode the image into them.

The code MUST allocate no buffer whose size depends on the width or the height before step 2 passes, whichever API of `png` reads the header, and the header-only test of "Decoding" pins that order.

The crate also sets `png::Limits` to the fixed bound on the working memory of the decoder in Section 13, which is what `png` may allocate besides the texels. The bound does not limit the size of an image, and RND-62 does not rest on it (Section 12.1). With this bound:

- An image in an accepted format within the size limits of step 2 decodes without an Exif chunk or with one of at most half the bound. The test "Decoding" checks this only for images 1 pixel high, and for other images it rests on the assumption of Section 12.1.
- A text chunk or a colour profile chunk of any length is skipped, because the crate tells `png` to ignore both kinds.
- An Exif chunk, the one metadata chunk that `png` keeps, that is larger than the bound gives `LimitExceeded` with the text of the memory limit of Section 7.2.
- An Exif chunk between half the bound and the bound decodes or gives `LimitExceeded`, depending on the decoder, as Section 12.1 assumes.

RND-62. `Image::decode_png` MUST check the width, the height, the number of pixels, and the bit depth from the header against the limits of this section before it allocates the texels, MUST return `LimitExceeded` for an image over a limit and `Decoding` for an image with 16 bits per channel and for a stream that cannot be read or is not a valid PNG image, with the texts of Section 7.2, and MUST NOT panic, because the image file is the first file that a user gives `escher` to read (Section 8 of the plan), and a header that names a large image would otherwise make the crate allocate that memory before any check.

An image fill has exactly the size limits of an output image of the renderer, with the same error texts (Section 7.2), and no other limit, as the owner decided (`DEC-92`). `Debug` for `Image` writes only its size, so that a failing assertion does not print its texels.

### 5.22. Mask Polygon

The mask polygon $P$ of a domain $D$ with $m$ sides is a convex Euclidean polygon with $m$ vertices $P_0, \ldots, P_{m-1}$, counterclockwise, whose edge $j$ runs from $P_{j-1}$ to $P_j$ and stands for side $j$ of $D$, which runs from vertex $j - 1$ to vertex $j$ (DOM-4 of the domain design). Where two sides of $D$ lie on one wall, the vertex between them has the angle $\pi$ in $D$, and its vertex of $P$ may have any angle up to $\pi$. The owner decided that a mask is such a flat polygon (`DEC-93`), and that the copies of two masks laid edge to edge must show the true seam, while they need not close up around a vertex (`DEC-94`). So the two edges of a pair of different sides need equal lengths.

`Masks::new` and `Renderer::new` build $P$ by the first of these rules that applies to the domain (`DEC-95`):

1. Euclidean domain: $P_j$ is the plane position $(V_{j,1}, V_{j,2})$ of vertex $V_j$ of the domain, so $P$ is the domain itself.
2. Hyperbolic or spherical domain whose sides are all mirrors, with three sides: the flat triangle of the angle rule below.
3. Hyperbolic reflection polygon with four or more sides, or the double of one: $P_j$ is the Klein position $(y_1/y_3, \ y_2/y_3)$ of $y = T V_j$, from equation (6.1.2) of [RAT] (Section 4.2 of the geometry reference), with $T$ = `hyperbolic::inverse(hyperbolic::translation(C))`, the translation that takes the centre $C$ of Section 5.23 to the origin. So $P$ is the straight-edged Klein picture of the domain itself, in a chart centred at the centre that the mask map uses. Walls are straight chords in the Klein model ([CFKP], Section 9, in Section 3.1 of the geometry reference). For a reflection polygon, $C$ is the centre that the domain design places at the origin within $10^{-9}$ (DOM-43 of the domain design).
4. Hyperbolic or spherical double of a reflection triangle: the domain is a double of its generating triangle $F$, with $n = 3$ and $m = 2n - 2 = 4$ (Sections 6.8 and 6.16 of the domain design). The vertices $V_0, V_1, V_2$ of $F$ are the vertices $W_0, W_2, W_3$ of the domain, by step 2 of Section 6.8 of the domain design. The flat triangle $P_F$ of $F$ comes from rule 2, which takes the corner angles of $F$ at $V_0$ and $V_1$ as half the corner angles of the domain at $W_0$ and $W_{n-1}$, and the angle at $V_2$ as the corner angle at $W_{n}$. $P$ is $P_F$ together with its reflection in the line of the edge of side 1 of $P_F$, numbered as step 2 of Section 6.8 of the domain design numbers the double: $P$ takes $P_F$ at the vertices $W_0$ and $W_{n-1}$ to $W_{2n-3}$, and the reflection of vertex $n - k$ of $P_F$ at vertex $W_k$ for $k$ from 1 to $n - 2$. Each pair of sides of the double is a side of $F$ and its image under the reflection in side 1 (steps 3 and 4 of Section 6.8 of the domain design), so the edges of a pair are reflections of each other.

The angle rule gives the flat triangle of a triangle domain whose vertex $j$ has the corner order $n_j$ and the corner angle $\alpha_j = \pi / n_j$, which `corner_angles()` gives. The canonical form orders the corners as $n_0 \ge n_1 \ge n_2$, so vertex 2 carries the smallest order and the largest angle (Section 5.1 of the domain design). The owner decided to keep the right angle where there is one, then the angle of the next smallest corner order, and to adjust the last corner (`DEC-94`). The flat triangle therefore has these angles $\beta_j$:

| Case | $\beta_0$ | $\beta_1$ | $\beta_2$ |
| --- | --- | --- | --- |
| $n_2 = 2$ and $n_1 \ge 3$, a right angle at vertex 2 | $\pi - \alpha_1 - \alpha_2$ | $\alpha_1$ | $\alpha_2$ |
| $n_1 = n_2 = 2$, the spherical `*n22` | $\alpha_0$ | $\pi - \alpha_0 - \alpha_2$ | $\alpha_2$ |
| $n_2 \ge 3$, a triangle without a right angle | $\alpha_0$ | $\alpha_1$ | $\pi - \alpha_0 - \alpha_1$ |

The first case keeps the right angle at vertex 2 and the angle $\pi/n_1$ at vertex 1, and vertex 0 takes the rest, so the angles 25.7, 60, and 90 degrees of `*732` at vertices 0, 1, and 2 become 30, 60, and 90 degrees. The second case cannot keep both right angles, so it keeps the right angle at vertex 2 and the angle $\pi/n$ at vertex 0. The third case keeps the two smallest angles and adjusts the largest one, so that the double of the triangle has angles below $\pi$ at $W_0$ and $W_{n-1}$ under rule 4. The triangle is placed as the Euclidean triangle of Section 6.5 of the domain design with the angles $\beta_j$ in place of the corner angles: $P_2$ at the origin, $P_0 = (1, 0)$, and $P_1$ at the distance $\sin \beta_0 / \sin \beta_1$ from $P_2$ in the direction at the angle $\beta_2$ from the first axis. A triangle with an angle $\beta_j \le 0$ has no flat triangle. That happens only for `*222`, whose three corners are right angles.

RND-63. `Masks::new` and `Renderer::new` MUST build the mask polygon by the rules of this section, MUST refuse with `InvalidOption` and the text of Section 7.2 a domain to which no rule applies, and MUST NOT build a polygon in any other way, because the masks and the image fills of one domain must use one polygon for an image painted over a mask to fill the tiles as the mask shows, and the owner's purpose needs equal lengths for the edges that meet.

No rule applies to these domains, which therefore have no mask polygon: the domains with fewer than three sides, which are the lunes of `*nn` and `nn`, the domains of `nx`, `x`, and `*`, and the domain of `1` with no side, and `*222` with the doubles built on it, the kite of `222` and the domain of `2*`. The owner decided to refuse them and deferred a construction for them to the roadmap (`DEC-96`). `Masks::new` and `Renderer::new` also treat a domain as having no mask polygon when the centre $C$ of Section 5.23, or the chart position of $C$ or of a vertex, cannot be computed. Under the statements of Section 10 on the centre, no domain that `build_with` returns and to which a rule applies reaches this state.

The rules cover every other domain that `build_with` returns:

| Rule | Domains |
| --- | --- |
| 1 | Every Euclidean domain: the reflection triangles and the rectangle, their rotation domains, and the wallpaper domains |
| 2 | The hyperbolic `*pqr` and the spherical `*332`, `*432`, `*532`, and `*n22` with $n \ge 3$ |
| 3 | The hyperbolic reflection polygons with 4 to 8 corners, such as `*3333`, and their doubles, the rotation polygons such as `3333` and `22222` |
| 4 | The hyperbolic kites of `pqr`, the spherical kites of `332`, `432`, `532`, and `n22` with $n \ge 3$, and the spherical group domains of `3*2`, every `2*n`, and `n*` with $n \ge 3$ |

Each rule gives $P$ its orientation. Rule 1 keeps the orientation of the canonical view. Rule 3 keeps the orientation that the canonical view would show with $C$ moved to the centre of the disk, because $T$ is a translation, and the disk position and the Klein position of a point of the hyperboloid lie in the same direction from the centre of the disk. Rule 2 puts edge 0 on the first axis. Rule 4 keeps the orientation of $P_F$.

That $P$ is convex with its vertices counterclockwise, and that the two edges of each pair of sides have equal length, is a statement of this design that the test "Polygon rules" pins (Section 10). For a double under rule 3 the equal lengths rest on the mirror symmetry of the double about the wall of side 1 of $F$ holding in the chart centred at $C$, and the domain design does not state that $C$ lies on that wall. The test pins the equal lengths, and a failure of it on a supported domain is a defect of rule 3. The halves of the corner angles in rule 4 read the double as the union of $F$ and its mirror image (DOM-23 of the domain design), an assumption of Section 12.1.

### 5.23. Mask Map

The mask map $M$ carries each point of the domain $D$ to a point of its mask polygon $P$. The owner left its construction open (`DEC-93`), and this section chooses a radial map in a chart in which the walls are straight lines (`DEC-97`). Its contract is what the owner's purpose needs: $M$ is a continuous bijection from $D$ onto $P$, it sends each side onto its edge, and along each side it is proportional to the distance along the side. Two edges that a pairing glues therefore carry the same parametrization, and the masks laid edge to edge show the seam as the render draws it. Smoothness is not required.

The map reads the vertices $V_j$, the covectors $c_j$, and the geometry of the domain, and $P$ with its vertices $P_j$. It uses these points:

- The centre $C$: the vertex sum of the domain normalized with `hyperbolic::normalize_point` or `spherical::normalize_point`, or for a Euclidean domain the mean of its vertices. It is not the point that `centre()` gives for a double, so that the chart of the sphere holds every vertex of a spherical kite.
- The flat centre $C'$: the mean of the vertices of $P$. For a Euclidean domain $C' = C$, because $P$ is the domain.
- The chart position $\hat{y}$ of a vector $y$: $y / y_3$ for a hyperbolic or Euclidean domain, which for a hyperbolic domain is the Klein position, and $y / (C \cdot y)$ for a spherical domain, the projection from the centre of the sphere onto the plane that touches the sphere at $C$.

The value of $M$ at a folded point $x$ comes from these steps:

1. Compute $\hat{x}$ and $\hat{C}$. For a hyperbolic or Euclidean domain, the step assumes a positive last coordinate $x_3$, the point size of Section 5.3, and when $x_3 > 0$ is false or $\hat{x}$ is not finite, it gives the fallback of Section 5.20. For a spherical domain, when $C \cdot x > 0$ is false or $\hat{x}$ is not finite, it gives the fallback.
2. When $\hat{x} = \hat{C}$, $M(x) = C'$.
3. With $r = \hat{x} - \hat{C}$, take the exit side $j$: the first side, in index order, with $\operatorname{turn}(\hat{V}_{j-1} - \hat{C}, r) \ge 0$ and $\operatorname{turn}(r, \hat{V}_j - \hat{C}) > 0$, where $\operatorname{turn}(a, b)$ is $a_1 b_2 - a_2 b_1$ for a hyperbolic or Euclidean domain and $C \cdot (a \times b)$ for a spherical domain. This is the side whose sector of the chart, between the rays from $\hat{C}$ through $\hat{V}_{j-1}$ and through $\hat{V}_j$, holds the ray from $\hat{C}$ in the direction $r$, and a ray through a vertex takes the side that starts at that vertex. When no side meets both conditions, give the fallback. Compute $e_j = c_j \cdot r$, give the fallback unless $e_j > 0$, and set $t^* = -(c_j \cdot \hat{C}) / e_j$. The sector decides, and not the least value of $-(c_k \cdot \hat{C}) / (c_k \cdot r)$ over the sides $k$, because two sides on one wall have equal values, as in `2222`, `22*`, `2*22`, the kites of `n22`, `2*n`, and `n*`, and the doubles of polygons with a corner of order 2 on the doubling wall.
4. Compute the boundary point $\hat{B} = \hat{C} + t^* (\hat{x} - \hat{C})$. For a hyperbolic domain, normalize it to a point $B$ with `hyperbolic::normalize_point`, and for a spherical domain with `spherical::normalize_point`, and give the fallback when that fails. For a Euclidean domain, $\hat{B}$ has the last coordinate 1, because $\hat{x}$ and $\hat{C}$ have it, so $B = \hat{B}$ with no normalization, and `euclidean::distance` reads only its first two coordinates.
5. Compute the fraction $\lambda = d(V_{j-1}, B) / d(V_{j-1}, V_j)$ with the function `distance` of the geometry, `hyperbolic::distance`, `euclidean::distance`, or `spherical::distance`, clamped to $[0, 1]$, and the flat boundary point $B' = P_{j-1} + \lambda (P_j - P_{j-1})$.
6. $M(x) = C' + (B' - C') / t^*$.

$C$ itself maps to $C'$ by step 2, but another point at a subnormal distance from $\hat{C}$ in the chart can reach the fallback of step 3 or step 4, and an image fill then gives it the background.

The point $x$ lies at the fraction $1/t^*$ of the way from $\hat{C}$ to $\hat{B}$ in the chart, and $M(x)$ at the same fraction of the way from $C'$ to $B'$. A point on side $j$ has $t^* = 1$, so $M$ sends it to the point of edge $j$ at the fraction $\lambda$ of its true length from $P_{j-1}$, and a vertex goes to its vertex of $P$ from either side. A folded point just outside $D$, within the tolerance of the fold, has $t^*$ just below 1 and maps just outside $P$, which the margin of the mask placement keeps inside the image.

`hyperbolic::distance` and `spherical::distance` lose accuracy for points close together (Sections 5.2 and 8.2 of the geometry design), so the fraction $\lambda$ of step 5, and with it $M$, is less accurate near a vertex than elsewhere. The tests of Section 8.1 therefore compare $M$, and the image values of the four models, within tolerances that they set as RND-16 states, not exactly.

The domain is the set of points with $c_j \cdot x \le 0$ for every side (Section 5.2 of the domain design), so its chart image is the intersection of half-planes in the chart. For a spherical domain, the chart needs $C \cdot x > 0$ at every point of the domain. That every vertex of each spherical domain with a mask polygon has $C \cdot V_j > 0$, and that $C$ lies strictly inside every side, are statements of this design that the test "Polygon rules" pins.

The radial construction assumes, for every rule of Section 5.22, that $\hat{C}$ lies strictly inside the chart image of the domain and $C'$ strictly inside $P$, and that each ray from $\hat{C}$ in the chart and each ray from $C'$ in $P$ meets the boundary once. These are statements of this design: "Polygon rules" pins that $C'$ lies strictly inside every edge of $P$, and "Round trip" pins the rays (Section 10). That $M$ is a continuous bijection onto $P$ with the behaviour on the sides stated above is a statement of this design that the tests "Round trip", "Edge fraction", and "Seams" pin (Section 10). So is the statement that the side of step 3 is the side through which the ray from $\hat{C}$ leaves the chart image of the domain, which Section 10 lists with the tests that pin it. For a Euclidean domain the chart is the plane and $P$ is the domain, so $M$ is the identity of the plane, which the test "Euclidean identity" checks.

The mask renderer draws $P$ alone and needs no inverse of $M$. The test "Round trip" computes the inverse in the reference arithmetic.

RND-64. The renderer MUST compute the mask map by the steps of this section from the domain and its mask polygon only, because the image fills and the seams of the masks depend on one map, and an image painted over a mask fills the tiles as the masks show only when every render of the domain uses the same map.

The steps take one test of each side, at most one normalization, and one call of `distance` for each sample of an image fill, because $C$, $\hat{C}$, and the side lengths are computed once. They call no function that `escher-geometry` lacks.

### 5.24. Wythoff Construction

A Wythoff fill draws the faces of a Wythoff construction on a reflection triangle in place of its tiles, which the owner decided for milestone `M23` of the plan (`DEC-68`). The user turns each of the three mirrors on or off, and gives one colour for each face type. The construction follows Section 10 of the colourings reference, and Section 5.28 lists the source or the unverified status of each statement.

The marks are indexed by the corners of the triangle. Vertex $k$ is corner $k$ with the corner order $n_k$, and the canonical form orders them as $n_0 \ge n_1 \ge n_2$ (Section 5.1 of the domain design). Side $j$ joins vertex $j - 1$ and vertex $j$, so it lies opposite vertex $j + 1$, with indexes modulo 3. Mark $k$ therefore names side $k + 2$, and side $j$ is active when mark $j + 1$ is on. Corner $k$ lies on sides $k$ and $k + 1$, whose marks are the other two marks, $k + 1$ and $k + 2$ (`DEC-99`).

`Wythoff::new` builds the construction in these steps, in this order:

1. Refuse a domain that does not have exactly three sides, each of kind `Mirror`. The reflection triangles are the hyperbolic `*pqr`, the Euclidean `*632`, `*442`, and `*333`, and the spherical `*332`, `*432`, `*532`, and `*n22` with $n$ from 2 to 100 (`DEC-107`).
2. Read the corner order $n_k$ of each corner as the integer nearest to $\pi / \alpha_k$, with $\alpha_k$ the corner angle that `corner_angles()` gives.
3. Apply the face rule to each corner $k$. Let $a_k$ be the number of active sides through corner $k$. With $a_k = 0$, the corner has no face, because the generator point lies on it. With $a_k = 1$, its face has $n_k$ sides, and with $a_k = 2$, it has $2 n_k$ sides. A face of 2 sides is a digon, which drops out, so a corner with $a_k = 1$ and $n_k = 2$ has no face (`DEC-102`). The face types are the corners with a face, in the order of the corners.
4. Refuse marks that give no face type. These are the marks with every mirror off, for which the generator point would lie on all three mirrors, and on `*n22` the marks with only mark 0 on, and on `*222` every marking with one mark on.
5. Compute the generator point $G$ from the covectors $c_j$ of the sides, which `escher-domain` stores for unit normals, by the number of active sides (`DEC-101`):
   - One active side $j$: $G = V_{j+1}$, the vertex opposite it.
   - Two active sides $a$ and $b$, with the third side $i$ inactive: $G$ = `wall_meet(c_i, c_a - c_b)` of the module of the geometry, the point of side $i$ on the wall of the covector $c_a - c_b$.
   - Three active sides: $G$ = `wall_meet(c_0 - c_1, c_1 - c_2)`.
   - On the sphere, negate the result of `wall_meet` when its dot product with the centre of the domain is negative.
6. For each active side $j$, compute the image $P_j G$ with the pairing matrix of side $j$, which is the reflection in its wall, the perpendicular $q_j$ as the covector of `wall_through(G, P_j G)` of the module of the geometry, through `hyperbolic::covector` for a hyperbolic domain, and the foot $F_j$ = `wall_meet(c_j, q_j)`, negated on the sphere as in step 5.
7. For each corner $k$ with a face and each active side $j$ through it, record the sign $s_{kj}$ of $q_j \cdot V_k$.
8. Return `Numerical` if a call of step 5 or 6 returned an error, or if a $q_j \cdot V_k$ of step 7 is zero or not finite.

Step 8 follows the convention of the domain design for a numerical failure (Section 10.2 of the domain design): an error and not a panic, because no domain may make the crate panic (RND-14). No construction that steps 1 and 4 accept is expected to reach step 8, which is a statement of this design that the test "Face rule" pins on the Wythoff triangle set. A construction that reaches it is a defect of the crate.

RND-65. `Wythoff::new` MUST build the construction by the steps of this section, MUST refuse with `InvalidOption` and the text of Section 7.2 at the first of steps 1 and 4 that refuses, MUST return `Numerical` with the text of Section 7.2 at step 8, and MUST NOT compute the generator point, a perpendicular, or a foot in any other way, because the faces, the borders, and the colours of one construction must all rest on one generator point, and a point that misses a mirror by more than the rounding moves every edge of the tiling.

The points of step 5 are the positions that the sources give for the sphere: the vertex of `p | q r`, the point of the side $PQ$ on the bisector of the angle at $R$ of `p q | r`, and the incentre of `p q r |`. Two preprints define the generator point as the point on the inactive mirrors at equal distance from the active ones, and that the spherical positions are these points is the research's reading (Section 10.1 and item 30 of Section 14.1 of the colourings reference). This design takes the wall of the covector $c_a - c_b$ as the set of points of the triangle at equal distance from the walls of sides $a$ and $b$. That step 5 gives a point of the triangle on the inactive sides at equal distance from the active ones, measured with `wall_distance` (Section 5.6 of the geometry design), in every geometry, is a statement of this design that the test "Generator point" pins (Section 10).

The faces of step 3 and the generator point of step 5 are the generator positions and the face rule of Sections 10.1 and 10.2 of the colourings reference, which [CLM1954] and [HAREL1993] state for the sphere. Their use for a Euclidean or hyperbolic triangle is `UNV-38`, and the position of the point there is `UNV-40`. The edges of the tiling join $G$ to its images $P_j G$ in the active mirrors (Section 10.1 of the colourings reference), so the segment of $q_j$ from $G$ to $F_j$ is half an edge, and each face is a polygon whose vertices are images of $G$.

The marks of the regular tiling $\{p, q\}$ of a signature `*pq2` with $p \ge q \ge 3$, which `escher info` prints as `Regular tiling: {p,q}, dual {q,p}`, have only mark 1 on, `010`, because $\{p, q\}$ is `q | p 2`, whose generator point is the corner of order $q$, and only the side opposite it is active (Sections 10.2 and 10.3 of the colourings reference). The dual $\{q, p\}$ is `p | q 2`, with only mark 0 on, `100`. Section 4.6 of the command-line design gives the seven markings of `*732`.

### 5.25. Face of a Sample

With `Fill::Wythoff`, the renderer finds the face type of a converged sample that is not an outside sample from its folded point $x$, in these steps:

1. For each corner $k$ with a face, in the order of the corners, test each active side $j$ through corner $k$. Corner $k$ matches when no such side gives $s_{kj} (q_j \cdot x) < 0$.
2. The face type is the first corner that matches. When no corner matches, it is the last corner with a face.

The steps read $x$ as the fold returns it. Each test compares the sign of a dot product, which a positive multiple of $x$ does not change, so the steps need no normalization, as the side test of the fold needs none (RND-8). A NaN dot product fails the comparison of step 1, so the corner matches.

RND-66. With `Fill::Wythoff`, the renderer MUST find the face type of each sample by the steps of this section from the folded point and the construction only, through one private function of `wythoff.rs`, because the colour and the border of a sample both read the face type, and two computations could give one sample the colour of one face and the border of another.

The perpendiculars from $G$ to the active mirrors cut the triangle into one region for each corner with a face, and a sample belongs to the face centred at the corner of its region. The research reads two shaders by one author, one on the sphere and one in the hyperbolic plane, as assigning a folded point to a face in this way, and no paper or book states the rule (Section 10.6.3 of the colourings reference). That the steps of this section give each sample the face that contains it, in every geometry, is `UNV-39`, which the tests "Face regions", "Faces around a centre", "Face oracle", and "Known faces" pin (Section 8.1).

Step 2 gives a face type to a sample that no corner matches. Under `UNV-39` such a sample lies within the rounding of a perpendicular, near $G$, or beside a mirror through a corner whose digon dropped out, which are places on an edge of the tiling that a border of positive width covers. The choice of the last corner is a choice of this design, so that every sample has a face type.

### 5.26. Borders of the Faces

With `Fill::Wythoff`, the borders lie on the edges of the faces, and the sides of the triangle draw no border (`DEC-104`). A converged sample with the face type $k$ is a border sample exactly when, for some active side $j$ through corner $k$:

- With `BorderUnit::Pixels`: `!(f.abs() >= half_width * g)` is true, with $f = q_j \cdot x$ and $g = \sqrt{(q_j \cdot T_1)^2 + (q_j \cdot T_2)^2}$, the side value $f_i$ and the gradient size $g_i$ of Section 5.4 with $q_j$ in place of $c_i$.
- With `BorderUnit::Intrinsic`: `!(d_j >= half_width)` is true, with $d_j$ the function `wall_distance` of the geometry of the domain of $q_j$ and $x$, as in Section 5.17.

RND-67. With `Fill::Wythoff`, the renderer MUST mark a converged sample as a border sample by the condition of this section, over the active sides through the corner of its face type only, and MUST NOT evaluate RND-9 or RND-54 over the sides of the domain, because the edges of the Wythoff tiling are the images of the segments from $G$ to the feet, while the mirrors of the triangle cross the faces, and a border on a mirror would draw the triangles that the construction exists to hide.

The test reads the perpendiculars of the sides through the corner of the face of the sample only, because the perpendicular of side $j$ separates the faces of the two corners of side $j$ and, beyond $G$, crosses the region of the third corner, where it is no edge. The test measures the distance to the whole wall of $q_j$, so within the half width of $G$ it can mark a sample that is nearer to the wall than to the segment, which lies inside the border at $G$ that the neighbouring faces draw, as RND-9 marks the samples near a vertex.

Every other part of Sections 5.4, 5.5, and 5.17 stays as it is: the fold, the screen tangents that it carries, the half width in either unit, the negated comparison that makes a NaN a border sample, the border colour of an unconverged sample, the background of an outside sample, and RND-52 and RND-56, which keep the model, the image size, the sample grid, and, for the intrinsic unit, the screen tangents out of the test. The estimate of the pixel unit rests on `UNV-15` and the intrinsic distance on `UNV-14`, as for the sides.

In every construction that `Wythoff::new` accepts, at least one corner of each active side has a face, which is a statement of this design that the test "Face rule" pins.

### 5.27. Face Colours

A Wythoff fill holds one colour for each face type, in the order of the corners. A converged sample that is not a border sample, whose face type is the corner with index $t$ among the face types, takes colour $t$ (`DEC-105`). The faces take no image fill, because the owner dropped image fills from milestone `M23` (`DEC-68`).

RND-68. `Renderer::new` MUST refuse with `InvalidOption` and the text of Section 7.2 a `Fill::Wythoff` whose construction holds other covectors than the sides of its domain, compared bit for bit, as RND-41 refuses a coset fill built for another side presentation, and with `Fill::Wythoff` the renderer MUST give a converged sample that is not a border sample the colour of its face type, because a construction of another domain would place the faces of one triangle on the tiles of another.

RND-69. Withdrawn on 2026-11-23, when the owner dropped image fills and face masks from milestone `M23` (`DEC-68`).

RND-70. The support of Wythoff fills MUST NOT change any decision, colour, statistic, or golden file of `Fill::One`, `Fill::Parity`, or `Fill::Cosets`, or any mask of `Masks::new`, because the committed golden images and the exact tests fix every decision of those fills, and `just update-golden` rewrites every golden file, so a changed image could enter the repository unnoticed. The acceptance is that no committed golden file changes.

### 5.28. Source of Each Wythoff Step

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.24 | A marking names its active mirrors, and the generator point lies on the other sides and off the active ones | [CBGS2008], p. 389, and [SCHULTE2016] (Section 10.2 of the colourings reference) | Sourced: the ring rule is the statement of [CBGS2008]. The rings of each Wythoff symbol, which combine that rule with the positions of [CLM1954], are the research's combination (Section 14.2 of the colourings reference) |
| 5.24 | The generator point is the vertex, the point on the bisector, or the incentre | [CLM1954] and [HAREL1993] for the sphere (Section 10.1 of the colourings reference) | Sourced for the sphere: the positions are the statement of the sources. That they are the points of equal distance of the preprints is the research's reading (item 30 of Section 14.1 and Section 14.2 of the colourings reference). `UNV-40` for the Euclidean and hyperbolic planes |
| 5.24 | The point of equal distance comes from `wall_meet` with $c_a - c_b$ | None | A statement of this design (Section 10). "Generator point" pins it |
| 5.24 | The face rule at each corner, with digons dropped | [CLM1954] and [HAREL1993] for the sphere (Section 10.2 of the colourings reference) | Sourced for the sphere: the faces of each symbol are the statement of the sources. The table of faces at a corner, which restates them by the mirrors through each corner, is the research's (Section 14.2 of the colourings reference). `UNV-38` for the Euclidean and hyperbolic planes |
| 5.24 | The edges join $G$ to its images in the active mirrors | [CLM1954] and [HAREL1993] for the sphere (Section 10.1 of the colourings reference) | Sourced for the sphere. `UNV-38` for the other planes |
| 5.24 | `010` gives $\{p, q\}$ and `100` its dual for `*pq2` | [CLM1954], p. 406, for the sphere, and its Table 8, p. 438, for the Euclidean plane (Section 10.3 of the colourings reference) | Sourced there: the Wythoff symbols of $\{p, q\}$ and of its dual are the statement of [CLM1954]. That only the mirror opposite the generator point is active, which gives `010` and `100`, is the research's combination of those positions with the ring rule of [CBGS2008] (item 31 of Section 14.1 of the colourings reference). `UNV-38` and `UNV-37` for the hyperbolic plane |
| 5.24 | The corner order is the integer nearest to $\pi$ over the corner angle | None | A statement of this design. "Face rule" pins it on every triangle of its sets |
| 5.25 | The perpendiculars cut the triangle into the regions of the faces | Two shaders by one author (Section 10.6.3 of the colourings reference) | `UNV-39` |
| 5.25 | No corner with a face lies on the perpendicular of an active side through it | None | A statement of this design. "Face rule" pins it |
| 5.26 | The border estimate on a perpendicular, and the distance to it | Sections 5.4 and 5.17 | `UNV-15` and `UNV-14`, as for the sides |

### 5.29. Whole Faces

A whole-face fill puts one picture on every face of a regular construction, turned or mirrored as an arrangement says, which the owner decided for milestone `M24` of the plan (`DEC-110` to `DEC-113`). The user paints the picture on the face mask of Section 5.32, with nothing inside it forced to be symmetric, and `Fill::WholeFace` samples it. Section 5.33 lists the source or the unverified status of each step.

The owner decided that the first scope is the regular tilings (`DEC-113`), and that the tilings with several pictures of one polygon or with several kinds of polygon are the last milestones of the proof of concept (`DEC-114`). This design proposes the regular tilings on which one picture suffices as the scope of `M24`, and refuses every other construction (`DEC-120`). `WholeFace::new` therefore accepts exactly the regular constructions. With the canonical order $n_0 \ge n_1 \ge n_2$ of the corners, these are the constructions with exactly one face type, at a corner $c$ with exactly one active side through it and $n_c \ge 3$, on a triangle whose corner 2 has the order 2 and whose third corner $g = 1 - c$ has an order of at least 3. By the face rule of step 3 of Section 5.24, these are the marks `010`, whose faces are the $p$-gons of $\{p, q\}$ centred at corner 0, and `100`, whose faces are the $q$-gons of $\{q, p\}$ centred at corner 1, on `*pq2` with $p \ge q \ge 3$ (`UNV-37`, `UNV-38`). In the three geometries, these are the hyperbolic `*pq2`, such as `*732`, `*542`, and `*642`, the Euclidean `*632` and `*442`, and the spherical `*332`, `*432`, and `*532`. The test "Regular scope" of Section 8.1 checks that the conditions select these and no other construction of the Wythoff triangle set.

`WholeFace::new` refuses every other construction with `InvalidOption` and the text `the construction is not a regular tiling` (`DEC-120`): a construction with two or three face types, which milestone `M26` of the plan plans, and a construction with one face type that is not a regular tiling, such as `010` or `011` on `*n22`, whose faces are the dihedral tilings that `DEC-81` keeps out of the regular tilings. `DEC-81` is a proposal that the owner has not decided, so the refusal of the constructions with one face type on `*n22`, such as `010` on `*722`, is a proposal too, which point 58 of Section 12.3 puts to the owner. A regular tiling that has no arrangement with one picture passes `WholeFace::new`, and the command line refuses it when the search finds no arrangement. $\{4, 5\}$ of `*542` with `100` has no arrangement without a mirrored neighbour, by Theorem 0.1 of [YUNCKEN2003] (Section 5.9.2 of the colourings reference), and no source states whether it has one with mirrored neighbours. Section 4.6 of the command-line design states the refusal. Milestone `M25` of the plan plans the fills with several pictures of one face that such a tiling needs.

For a regular construction, this design names these parts of the triangle. The face corner is $c$, 0 for `010` and 1 for `100`, with the order $n = n_c$. The generator point of the construction is vertex $g$ (step 5 of Section 5.24), which is a vertex of the tiling. The vertex mirror $v$ is side 1, through the face corner and vertex $g$. The centre mirror $m$ is the other side through the face corner, which passes through vertex 2: side 0 for $c = 0$ and side 2 for $c = 1$. The edge side $e$ is the side opposite the face corner: side 2 for $c = 0$ and side 0 for $c = 1$. It joins vertex $g$ and vertex 2, the foot of the perpendicular from $G$ to the active side, and so carries half an edge of the tiling (Section 5.24). The reference face $P$ is the face centred at vertex $c$, which holds the whole domain, because every folded sample takes the one face type (Section 5.25).

`WholeFace::new` takes these steps, in this order:

1. Refuse a construction whose covectors differ from those of the sides of the domain, compared bit for bit, as RND-68 does.
2. Refuse a construction that is not a regular construction, by the conditions above.
3. Name $c$, $n$, $g$, $v$, $m$, and $e$ as above.
4. Build the side presentation of the domain with `side_presentation()`, in which generator $s_j$ stands for side $j$ (DOM-32 of the domain design), and refuse a domain without one, as `CosetFill::new` does.
5. Compute the angle $\psi$, and build the flat face polygon $Q$, the face triangle $F'$, and the flat reflection matrices $A_m$ and $A_v$ of Section 5.31.
6. List the $2n$ elements of the stabilizer as the words $(s_m s_v)^j$ and $(s_m s_v)^j s_m$ for $j$ from 0 to $n - 1$, in the order of $j$, with the rotation word before the reflection word of each $j$, and the flat matrix $A_w$ of each word $w$ (Section 5.31). The first word, $(s_m s_v)^0$, is the empty word, the identity. `stabilizer()` returns the other $2n - 1$ words in this order.
7. Take as $\rho$ the one of the words $s_m s_v$ and $s_v s_m$ whose flat matrix maps edge 0 of $Q$ onto edge 1, found as the one whose flat matrix carries the midpoint of edge 0 nearer to the midpoint of edge 1, with $s_m s_v$ on a tie, and form for each edge $i$ from 0 to $n - 1$ the edge word $r_i = \rho^i \, s_e \, \rho^{-i}$, with $\rho^{-1}$ the other of the two words.
8. Build the face mask map of Section 5.31. When it cannot be built, the whole face keeps none, refuses nothing, and every picture sample of a whole-face fill gives the fallback (Section 5.31).

Edge $i$ of the reference face is the image under $\rho^i$ of the edge that holds side $e$ of the domain, which is edge 0, so edge $i$ of the reference face stands for edge $i$ of $Q$. Its edge word $r_i$ is the reflection in its wall.

That the stabilizer of the reference face is the group of order $2n$ that the reflections in sides $m$ and $v$ generate, that the $2n$ words of step 6 name its $2n$ elements, each once, and that the reference face is the union of the images of the domain under them, is `UNV-44`. The test "Stabilizer" checks parts of it in the reference arithmetic, and the row of `UNV-44` in Section 10 states which parts no test checks. That the flat matrices $A_w$ of the words of step 6 are $2n$ different matrices, so that the map from the words to the matrices is one to one, is a statement of this design with no source, which the same test checks on the whole-face set.

RND-71. `WholeFace::new` MUST build the whole face by the steps of this section, MUST refuse with `InvalidOption` and the texts of Section 7.2 at the first of steps 1, 2, and 4 that refuses, and MUST accept every regular construction of a domain that has a side presentation, because the stabilizer words, the edge words, and the flat face polygon of one face must rest on one numbering of the sides and of the edges, and the command line passes the same words to the search of the arrangements (`DEC-115`).

### 5.30. Arrangements and Edge Pairing

An arrangement of a regular construction is a colouring of its side presentation in whose coset table no element of the stabilizer other than the identity fixes a coset. Under `UNV-41` and `UNV-42`, its subgroup maps no face onto itself other than by the identity, and under `UNV-43` with the order $2n$ of the stabilizer (`UNV-44`), an arrangement of index $2n$ has one orbit on the faces, so one picture covers every face. The owner decided that the arrangements with mirrored neighbours are offered too (`DEC-112`), so the subgroup may hold reflections and glide reflections. The command line finds the arrangements with `free_classes_of_index` of `escher-group` on `presentation()`, with the words of `stabilizer()` and the colour count `index()`, one for each class of subgroups (GRP-45 of the group design, `DEC-115`).

`WholeFace::edges`, `WholeFace::render_mask`, and `WholeFaceFill::new` check an arrangement in this order, and refuse it with `InvalidOption` and the texts of Section 7.2:

1. The coset table has one generator for each side of the domain, and every relator of the side presentation returns each coset to itself, as conditions 2 and 3 of `CosetFill::new` state (RND-36).
2. The colour count of the colouring is $2n$.
3. The traces from coset 0 of the $2n$ words of step 6 of Section 5.29 reach every coset of the table, each once.

The stabilizer element of coset $a$ is the word $w_a$ of step 6 whose trace from coset 0 reaches $a$, with `CosetTable::trace`, and check 3 makes it unique. Under `UNV-42` and `UNV-44`, every colouring that `free_classes_of_index` returns for the words and the index of a `WholeFace` passes check 3, and a colouring that fails it is not an arrangement of the faces.

The edge pairing of an arrangement comes from these steps, for each edge $i$ of the reference face from 0 to $n - 1$:

1. Compute $a$ = `trace(0, r_i)` with the edge word $r_i$ of step 7 of Section 5.29.
2. Take its stabilizer element $w = w_a$.
3. The partner $\sigma(i)$ is the edge of $Q$ onto which $A_w$ maps edge $i$ of $Q$, found as the edge whose midpoint is nearest to the image under $A_w$ of the midpoint of edge $i$.
4. The neighbour is mirrored when $\det A_w = 1$, that is when $w$ is a rotation or the identity, and turned when $\det A_w = -1$, that is when $w$ is a reflection.

Across edge $i$ of the reference face lies a face whose picture is the picture of the reference face moved by an isometry that carries edge $\sigma(i)$ onto edge $i$, turned or mirrored as step 4 says (`DEC-111`). An edge may be paired with itself. A turned neighbour across an edge paired with itself is the picture turned by a half-turn about the midpoint of the edge, as Yuncken notes for the orientation-preserving groups (Section 5.9.2 of the colourings reference), and a mirrored one is the picture reflected in the edge.

That the steps give, for each edge, the edge of the neighbour that lies on it and whether the neighbour is turned or mirrored, that the pairing is an involution, and that it is the pairing $\sigma$ of Lemma 0.2 of [YUNCKEN2003] for an arrangement with no mirrored neighbour, is `UNV-45`. The tests "Edge pairing" and "Pairing geometry" of Section 8.1 check it.

RND-72. `WholeFace::edges`, `WholeFace::render_mask`, and `WholeFaceFill::new` MUST check an arrangement by the three checks of this section, in their order, and MUST compute the stabilizer elements and the edge pairing by the steps of this section only, through one private function of `whole_face.rs`, because the edge pairing that `info` prints, the borders and marks of the face mask, and the picture of each sample must rest on one reading of the coset table, and a second reading could print one pairing and draw another.

### 5.31. Picture of a Sample

The flat face polygon $Q$ is the regular $n$-gon with the vertices $Q_k = (\cos \varphi_k, \ \sin \varphi_k)$, $\varphi_k = \psi + (2k + 1)\pi/n$, for $k$ from 0 to $n - 1$, counterclockwise, whose edge $i$ runs from $Q_{i-1}$ to $Q_i$, with indexes modulo $n$, as the edges of a mask polygon do (Section 5.22). Its centre is the origin, and the midpoint of edge 0 lies at $\cos(\pi/n) \, (\cos \psi, \ \sin \psi)$, at the angle $\psi$ from the first axis. The size of $Q$ has no effect, because the mask placement fits it to the image (`DEC-117`).

The angle $\psi$ turns $Q$ about its centre so that a picture painted on the face mask appears the same way up at the centre of the reference face in the default view of the geometry, as the owner decided (`DEC-122`). The default view is fixed for each geometry, whatever the render options say:

- Hyperbolic: the canonical view of the Poincare disk of Sections 5.1 and 5.2, whatever `model` is given.
- Euclidean: the plane view of Section 5.8.
- Spherical: the ball view of Section 5.9 with the yaw, the pitch, and the roll all 0, so that the viewing matrix is the identity, whatever `rotation` is given.

So $\psi$ reads no render option, and the face mask of a tiling does not depend on the view options.

By this rule, the direction in the face mask from the centre of $Q$ to the midpoint of edge 0 equals the direction in the image of the default view, at the image of vertex $c$ of the domain, the centre of the reference face, of the tangent of the segment from vertex $c$ toward vertex 2 of the domain. Vertex 2 is the midpoint of edge 0 of the reference face, whose half is side $e$ (Section 5.29). The mask placement of Section 5.19 and the disk, plane, and ball positions of Sections 5.1, 5.8, and 5.9 all have the first axis to the right and the second axis up. $\psi$ is the angle from the first axis of the vector $\mu$ that this design takes as the image of the tangent in the default view:

| Geometry | $\mu$ |
| --- | --- |
| Euclidean | $(V_{2,1} - V_{c,1}, \ V_{2,2} - V_{c,2})$, the difference of the plane positions of vertex 2 and vertex $c$ |
| Hyperbolic | `hyperbolic::to_disk(T_c V_2)`, with $T_c$ = `hyperbolic::inverse(hyperbolic::translation(V_c))`, the translation that takes vertex $c$ to the origin |
| Spherical | $(w_1, w_2)$, with $w = V_2 - (V_c \cdot V_2) V_c$ |

`WholeFace::new` computes $\psi$ = `mu.y.atan2(mu.x)` once, at step 5 of Section 5.29. When $\mu_1 = \mu_2 = 0$, the image of the tangent has no direction, which happens on the ball when vertex $c$ lies on the limb. Then $\psi = -\pi/2$, so $Q$ keeps edge 0 horizontal at the bottom.

In the plane, the picture on the reference face is a scaled copy of the painted picture with no turn. In the disk and on the ball, the direction from the centre of the reference face toward the midpoint of its edge 0 agrees at the centre of the face with the direction on the face mask. Away from the centre, directions in the disk and on the ball can differ from the painted ones, so the rule gives the same way up at the centre of the face only. For the hexagons of `*632` with `010`, which stand on a corner in the default view, the face mask stands on a corner too.

That the vector $\mu$ of each row is the image of that tangent in the default view, that with this $\psi$ the picture appears the same way up at the centre of the reference face, and that in the plane it shows no turn, are statements of this design with no source (Section 10). The test "Upright picture" of Section 8.1 checks them in part: on `*732`, `*632`, and `*532` with `010`, it measures the directions to two features placed away from the centre of $Q$, and accepts each within an angle that the test sets.

The face triangle $F'$ has the vertex $F'_c = (0, 0)$, the centre of $Q$, the vertex $F'_2 = \cos(\pi/n) \, (\cos \psi, \ \sin \psi)$, the midpoint of edge 0, and the vertex $F'_g$, which is $Q_{n-1}$ for $c = 0$ and $Q_0$ for $c = 1$. Its vertex $F'_j$ stands for vertex $j$ of the domain, so its vertices run counterclockwise in the order of the vertices of the domain, as a mask polygon does (Section 5.22). It has the angle $\pi/n$ at $F'_c$ and a right angle at $F'_2$. Its side $e$, between $F'_g$ and $F'_2$, is half of edge 0 of $Q$.

The flat reflection matrices are $A_m$, the reflection in the line through $F'_c$ and $F'_2$, and $A_v$, the reflection in the line through $F'_c$ and $F'_g$, each the matrix $2 d d^{\mathsf{T}} - I$ for the unit direction $d$ of its line. The flat matrix of a word $w = t_1 t_2 \cdots t_k$ in $s_m$ and $s_v$ is the product $A_w = A_{t_1} A_{t_2} \cdots A_{t_k}$, in the order of the word, as the matrix of a word of the fold is the product of its pairings in fold order (GRP-38 of the group design). The empty word has the identity.

The face mask map carries a folded point $x$ of the domain to $M_F(x)$, the point of $F'$ that the mask map of Section 5.23 gives with $F'$ in place of the mask polygon of Section 5.22. Every step of Section 5.23 stays as it is: the centre $C$ of the domain, the flat centre $C'$, here the mean of the vertices of $F'$, the chart, the exit side by the sector, the fraction along the side, and the fallbacks (`DEC-116`). Side $e$ of the domain therefore goes onto half of edge 0 of $Q$ in proportion to its length, and sides $m$ and $v$ go onto the lines of $A_m$ and $A_v$.

With `Fill::WholeFace`, the value of a converged sample that is not an outside sample and not a border sample comes from these steps:

1. Compute the coset $a$ that the trace of the side sequence of the fold, in fold order, reaches from coset 0, which is `trace(0, w)` for the word $w = s_{i_1} \cdots s_{i_k}$ of the sides $i_1, \ldots, i_k$ that the fold applies. This is the forward trace, and not the reversed trace of inverse columns of `colour_of` (GRP-38).
2. Take the flat matrix $A_{w_a}$ of the stabilizer element of $a$, which `WholeFaceFill::new` computes once for each coset.
3. Compute $M_F(x)$. A step of Section 5.23 that gives the fallback gives it here, and so does a whole face that holds no face mask map (step 8 of Section 5.29), for a solid paint too.
4. The picture point is $q = A_{w_a} M_F(x)$, a point of $Q$.
5. Take the value of the paint at $q$: for `Paint::Solid`, its colour, and for `Paint::Image`, the value that steps 2 to 4 of Section 5.20 give for the point $q$, with the mask placement of $Q$ at the size of the image in place of that of the mask polygon.

The value is then shaded on the ball as Section 5.10 states, like every fill colour. The face of a sample, its border, and the border colour are those of the Wythoff fill of its construction (Sections 5.25 and 5.26), so the edges of the faces draw their borders and the sides of the domain draw none.

That the sample lies in the face $gP$ for the fold product $g = s_{i_1} \cdots s_{i_k}$, that the element of the arrangement that carries the reference face onto that face is $g w_a^{-1}$, and that the sample therefore shows the point $w_a x$ of the reference face, which the face mask map and $A_{w_a}$ carry to $q$, is `UNV-46`. It also rests on the fold product of `UNV-17`. The tests "Whole picture" and "Face seams" of Section 8.1 check it. Coset 0 has the empty word as its stabilizer element, so the reference face shows the picture as the face mask shows it, unmirrored, and in the default view the same way up as the orientation of $Q$ above states.

The value depends only on the folded point, the side sequence, the arrangement, and the paint, so RND-59 and RND-60 hold for it as for an image fill: no model, rotation, render size, sample grid, or border changes it, the sampling interpolates in linear light, and the fallback is the background colour.

The property that the owner needs from this map is that an arbitrary picture painted on $Q$ appears whole on each face (`DEC-110`, `DEC-111`): each point of the picture shows at one point of the face, the picture neither splits nor doubles at the lines between the $2n$ wedges of the face, and the map forces no symmetry on the picture. In this design the distortion is the same in each wedge up to the turn or mirror of its flat matrix $A_w$. That the map from a face onto $Q$ that these steps give, the face mask map on each copy of the domain in the face followed by the flat matrix of that copy, is one to one from the face onto $Q$ and continuous across the lines between the wedges is a statement of this design with no source. Section 10 records it. The tests "Face triangle map", "Wedge continuity", and "Whole picture" of Section 8.1 check it only at the points and pixels that they sample.

Along edge $i$ of a face, the face mask map takes each half of the edge, a copy of side $e$, onto half of an edge of $Q$ in proportion to its length, so the two faces on an edge read the picture along edges $i$ and $\sigma(i)$ of $Q$ with the same parametrization, in the direction that the flag gives. That a picture that agrees along the edges of $Q$ laid edge to edge as the marks of Section 5.32 say gives a render that is continuous across every edge of the tiling is a statement of this design (Section 10). "Face seams" checks it at three points of each edge of the constructions of "Whole picture".

RND-73. With `Fill::WholeFace`, the renderer MUST compute the value of each sample by the steps of this section from the folded point, the side sequence of its fold, and the fill only, through one private function of `whole_face.rs`, MUST take its face type and border by RND-66 and RND-67 from the construction of the fill, and MUST NOT compute the picture point in any other way, because the face mask, the edge pairing, and the picture of every face must rest on one face mask map and one reading of the coset table, and a picture that the render read through another map would not continue across the edges as the face mask shows.

### 5.32. Face Mask

`WholeFace::render_mask` draws the face mask of an arrangement as `Masks::render` draws a polygon mask (Section 5.19), with $Q$ in place of the mask polygon: the mask placement of $Q$ at the size given, white inside $Q$, black outside it, a border inside each edge of $1/64$ of the shorter side in the code colour of the class of the edge, the quarter of the edge at each marked end in black, and the fixed 4 by 4 sample grid. It reads no render option, and it draws through the private mask function of `mask.rs` with the polygon, the code colours, and the marked ends as its input (`DEC-118`).

The mask placement fits $Q$ turned by $\psi$ as it fits any mask polygon: it reads the bounding box of the turned $Q$, puts the centre of that box at the centre of the image, and scales the polygon so that the whole of it lies in the image with at least the margin on every side (Section 5.19). The picture point of step 5 of Section 5.31 uses the same placement, so an image painted over the face mask lines up with $Q$ for every $\psi$.

The classes of the edges are the orbits of the edge pairing: two paired edges form one class, and an edge paired with itself forms a class of its own, as the owner decided (`DEC-111`). The classes are numbered from 0 to $N - 1$ in the order of their least edge, and class $s$ takes the code colour of the hue table of Section 5.19 for $N$ classes. $N$ is at most $n$, which is at most 100, within the range that the test "Code colours" covers.

The marks follow the carrying of Section 5.19, with a turned neighbour in the place of the kinds `Rotation` and `Translation` and a mirrored neighbour in the place of the kinds `Mirror`, `GlideReflection`, and `RotaryReflection`. The least edge $i$ of a class has its marked end at its start, vertex $i - 1$ of $Q$. Its partner $j = \sigma(i)$ then has its marked end at its end, vertex $j$, when the neighbour is turned, and at its start, vertex $j - 1$, when it is mirrored. So on the mask, the marks of a turned pair lie at the start of one edge and the end of the other in the counterclockwise order of $Q$, and the marks of a mirrored pair lie at the starts of both. An edge paired with itself by a mirrored neighbour has one mark, at its start, and its artwork meets its own mirror image. An edge paired with itself by a turned neighbour is reached by the carrying with both ends, so by the rule of Section 5.19 it has a mark at both ends, and its artwork must match itself turned about the midpoint of the edge. That copies of the face mask laid edge to edge with their marks together show each seam of the render is a statement of this design, which "Face seams" checks at the same points (Section 5.31).

RND-74. `WholeFace::render_mask` MUST draw each sample by the mask placement of $Q$ and the steps of Section 5.19, MUST give the two edges of a pair one code colour and an edge paired with itself a code colour of its own, MUST place the marks by the carrying of this section, and MUST read no render option, because the user matches the edges of the picture by the code colours and the marks (`DEC-111`), and an image painted over the face mask fills the faces as the mask shows only through the placement of $Q$ that the render reads (Section 5.31).

RND-75. The support of whole-face fills MUST NOT change any decision, colour, statistic, or golden file of `Fill::One`, `Fill::Parity`, `Fill::Cosets`, or `Fill::Wythoff`, or any mask of `Masks::new`, because the committed golden images and the exact tests fix every decision of those fills, and `just update-golden` rewrites every golden file, so a changed image could enter the repository unnoticed. The acceptance is that no committed golden file changes.

RND-76. `Renderer::new` MUST refuse with `InvalidOption` and the text of Section 7.2 a `Fill::WholeFace` whose construction holds other covectors than the sides of its domain, compared bit for bit, as RND-68 refuses a Wythoff fill of another domain, because the picture of a face rests on the side sequence of the fold in the domain that the arrangement was found for.

### 5.33. Source of Each Whole-Face Step

| Step | Statement used | Source | Status |
| --- | --- | --- | --- |
| 5.29 | `010` and `100` on `*pq2` give $\{p, q\}$ and $\{q, p\}$, with the faces centred at corners 0 and 1 | Section 5.28 | As for the Wythoff construction: `UNV-37` and `UNV-38` for the hyperbolic plane |
| 5.29 | The conditions on one face type select exactly the regular constructions | None | A statement of this design. "Regular scope" checks it on the Wythoff triangle set |
| 5.29 | The stabilizer of the reference face is the group of order $2n$ of the reflections in sides $m$ and $v$, with the $2n$ words of step 6, and the face is the union of the images of the domain under it | Frettloh states the $q$-gon $F \cup aF \cup abF \cup \dots$ of `(q^p)` only, and not that a group of two generators is the stabilizer of a face (Section 5.3 and item 4 of Section 14.1 of the colourings reference) | `UNV-44` |
| 5.30 | A subgroup acts freely on the faces exactly when no element of the stabilizer other than the identity fixes a coset of its table, and an index of $2n$ gives one orbit | Section 5.9.6 of the colourings reference | `UNV-41`, `UNV-42`, and `UNV-43` |
| 5.30 | The test on one table of a class holds for every subgroup of the class | The conjugates of a subgroup are the stabilizers of the cosets of its table, whose tables are its relabellings ([HOLT2005] p. 190, GRP-34 of the group design) | Sourced for the relabellings. That the test, which reads only the permutations of the table, gives the same answer on every relabelling is a statement of the group design that "Free filter" checks on its inputs |
| 5.30 | The edge pairing, its flags, and that it is an involution | [YUNCKEN2003] states the involution $\sigma$ and the half-turn on an edge paired with itself for orientation-preserving groups (Section 5.9.2 of the colourings reference). No source states the steps from a coset table, or any pairing with a mirrored neighbour | `UNV-45` |
| 5.31 | The face mask map as the mask map of Section 5.23 with $F'$ | Section 5.23 | A choice of this design (`DEC-116`), with the statements of Section 10 on the mask map |
| 5.31 | The picture point of a sample from the forward trace of its fold and the stabilizer element of the coset | None | `UNV-46`, with the fold product of `UNV-17` |
| 5.31 | The vector $\mu$ of each geometry is the image in the default view of the tangent at vertex $c$ toward vertex 2, and with the angle $\psi$ the picture appears the same way up at the centre of the reference face | None | A statement of this design (Section 10). "Upright picture" checks it in part. The orientation itself is the owner's decision `DEC-122` |
| 5.32 | The marks of a turned and a mirrored neighbour | The carrying of Section 5.19 | A statement of this design, as for the polygon masks. "Face seams" checks it |

## 6. Numerical Behaviour

### 6.1. Scale and Renormalization

The side test of step 3 of Section 5.3 compares $c_i \cdot x$ with $\tau \sigma_i \lambda(x)$, and both scale with $x$. The fold decides on the ray of $x$, not on its point of the hyperboloid or the sphere, so it needs no renormalization. The point size of the sphere, the largest absolute coordinate, is exact and scales exactly with a power of two, as the last coordinate does. Multiplication by a power of two is exact in IEEE 754 arithmetic when no overflow or underflow occurs. So while no product or sum of the side test and of the pairing underflows or overflows, a point scaled by $2^k$ gives each of them scaled by exactly $2^k$, and the fold takes the same decisions. A property test asserts this exactly.

For a spherical domain, every finite vector other than the zero vector is a point, so a small vector reaches the subnormal range, where the products round and the scaling is not exact. `fold` states its accuracy for a spherical domain only for a vector whose largest absolute coordinate is at least $10^{-300}$, a limit of this design. At a point size of $10^{-300}$ the threshold $\tau \sigma_i \lambda(x)$ is below $10^{-314}$, in the subnormal range, so exact arithmetic does not justify the limit. The limit keeps the coordinates of the point above the subnormal range, and it lies far below the points of length about 1 that the renderer passes. The property tests assert exact scaling for point sizes from $2^{-140}$ to $2^{140}$ only. Above the limit, a fold can still fail to converge within the cap near a vertex of very high order (Section 6.7), so the range of accuracy of Section 2.1 also requires a converged fold. `fold` does not rescale a smaller vector, because the renderer passes only points of length about 1. Point 34 of Section 12.3 gives the alternative of rescaling.

The point drifts off the hyperboloid during a long fold. The fold does not correct the drift, and the tests measure the folded point against the reference, which carries the same unnormalized ray, as a hyperbolic distance between the normalized points.

`DEC-31` and Section 9.3 of the architecture state that the fold renormalizes nothing, and that the renderer accumulates no matrix. The screen tangents replace the matrix that a border estimate would need (`DEC-28`), and RND-8 states the rule on renormalization. `DEC-31` is a proposed default, so Section 12.3 asks the owner to confirm it.

### 6.2. Tolerance of the Side Test

RND-13. The tolerance constant $\tau$ MUST be a positive private constant, and the side test MUST use $\tau \sigma_i \lambda(x)$ as its threshold, because a point exactly on a wall or at a vertex of the domain otherwise takes steps between walls that it lies on, and the threshold must scale with the point (Section 6.1).

$\tau$ is a small robustness constant for points on walls and at vertices. It is not a bound on the rounding error of the side test, and no constant $\tau$ bounds that error across the range of accuracy, because that error grows by orders of magnitude toward the limit circle. Points within $\tau$ of a wall, relative to $\sigma_i \lambda(x)$, count as inside that side.

$\tau$ is $4 \times 10^{-15}$, a choice of this design. With $\tau = 0$, vertices of the domain take steps. The tests measure, at every vertex $v$ of each domain of a vertex set and for every side $i$, the ratio $s_i(v) / (\sigma_i \lambda(v))$, assert that every vertex folds converged in zero steps and returns its input bit for bit, which holds when the largest positive ratio is at most $\tau$, and print the largest ratio with its vertex, its side, and its margin. Where the vertex set meets it, they also assert that 8 times the largest ratio is at most $\tau$:

| Vertex set | Factor 8 asserted |
| --- | --- |
| The domains of the fold set of Section 6.4 and of the 5 vertex triples of Section 8.1 | Yes |
| The Euclidean domain set, `*2222` with `aspect` 4, and `*2222` at 64 values of `aspect` evenly spaced in its logarithm over its range | Yes |
| The spherical domain set, including the two poles of each lune | Yes |
| The rotation domains of the rotation domain set, of the 5 vertex triples, and of `2222` with the values of `aspect` of the Euclidean row | No. The image vertices of the doubles miss the factor |
| The 15 reflection polygons and, apart, the 5 doubles of the polygon domain set | No for either set. The doubles miss the factor |
| The wallpaper domain set | Yes |
| The spherical group domain set | Yes |

$\tau$ keeps one value for every geometry and every kind of domain, because another value moves the decisions of the hyperbolic fold and changes the committed golden images, which RND-32 forbids. Point 35 of Section 12.3 puts the shortfall of the rotation domains and of the doubles to the owner. The point size of the sphere stays the largest absolute coordinate, which changes no hyperbolic or Euclidean decision.

### 6.3. Non-Finite Values

The crate turns a non-finite point and every NaN into an unconverged fold, a border sample, or an outside sample, never into a fill colour, and no non-finite value causes a panic:

| Value | Where | Result |
| --- | --- | --- |
| A non-finite coordinate of the input or of the point after a step | Step 2 of Section 5.3 | Unconverged |
| A point size that is not positive, of the input or of the point after a step: for a hyperbolic or Euclidean domain a last coordinate that is not positive, including a zero of either sign, and for a spherical domain the zero vector | Step 2 of Section 5.3 | Unconverged |
| For a spherical domain, a finite vector with subnormal coordinates, such as $(0, 0, -10^{-320})$ | Step 2 of Section 5.3 | Folded as any other ray. It can converge, but it lies below the limit of Section 6.1 and outside the range of accuracy. The argument of Section 6.1 for exact scaling does not hold there, so the scale property test draws no such point |
| A side value that is NaN | Step 3 of Section 5.3 | Unconverged |
| A side value that overflows to an infinity for a finite point | Step 3 of Section 5.3 | Compared as a number. A violated side leads to a step, and step 2 then checks the new point |
| A NaN $f_i$, or a NaN $g_i$ | Section 5.4 | Border sample, because a comparison with NaN is false |
| An infinite $g_i$ with a finite $f_i$ | Section 5.4 | Border sample, because $w g_i$ is infinite, or NaN for a half width $w$ of 0 |
| An infinite $f_i$ with a finite $g_i$ | Section 5.4 | Not a border sample by side $i$, because $\lvert f_i \rvert \ge w g_i$ holds |
| A disk position that is NaN | `hyperbolic::from_disk` | Outside sample |
| A ball position that is NaN | Section 5.9 | Outside sample, because $d > 0$ is false |

A rotation angle or a light direction that is not finite never reaches the renderer, because `build` rejects it (Section 4.1). Every other combination of infinite values follows the comparison of RND-9 under IEEE 754.

RND-14. `fold` and the renderer MUST NOT panic for any input, including any `f64` bit pattern of the point and any domain that `build` returns, because a panic stops the caller's whole program (ERR-5 of the coding standard), and the input of `fold` is arbitrary in the tests and near the limit circle.

`RenderStats` holds integers only, so no statistic is NaN. A caller computes a mean from `folds()` and `samples() - outside()`.

### 6.4. Range of Accuracy

For a hyperbolic domain, the crate states its accuracy for disk positions up to the radius $1 - 10^{-6}$, the radius band 6. That radius corresponds to a hyperbolic distance of about 14.5 from the centre by the relation of [CFKP], Fact 5 (Section 4.7 of the rendering reference). Beyond it, the fold in double precision applies a different side than a 256-bit fold for many samples, and from about $1 - 10^{-8}$ outward `from_disk` returns vectors that are not points, which never converge. Section 6.6 states the range of accuracy for a Euclidean domain. This section and Sections 6.6 and 6.7 cover the reflection triangles, rectangles, and lunes. Section 6.8 covers the rotation domains, and Section 6.9 the reflection polygons and their doubles.

RND-15. For a hyperbolic domain, the renderer and `fold` MUST meet the bounds that the tests of this section hold in each radius band from 1 to 6, and for a disk position beyond the range of accuracy MUST only return without a panic and report the fold as converged or unconverged, because the bounds are measured only up to that radius, and a promise beyond it would be false for many samples.

The fold applies the pairings that `hyperbolic::reflection` builds to points outside the range of its row in Section 8.4 of the geometry design, which covers distance 3 for the points of `reflection`. This crate relies on its own end-to-end measurements, as Section 9.3 of the domain design does for its construction.

The fold set is the plan triples of Section 12.1 of the domain design and the triple $(100, 100, 100)$, with disk positions at 512 evenly spaced angles in each radius band, which gives 3584 folds per band. The tests measure these quantities against the reference arithmetic of Section 8.3, each bound set as RND-16 states, and hold the bound of each triple and band:

| Quantity | Measure | Measured against | Input set |
| --- | --- | --- | --- |
| Folded point | The hyperbolic distance between the normalized folded point and the normalized result of the reference replay, for each triple and band | The replay, in the reference arithmetic, of the side sequence of the fold on the exact `f64` input point and the exact `f64` pairings | The fold set. Every fold of the set converges |
| Fold decisions | The number of folds whose side sequence differs from that of an independent 256-bit fold of the same input point, summed over the triples of each band | The 256-bit fold of Section 8.3, with $\tau = 0$ | The fold set |
| Tangents | The largest component of the difference of each tangent, divided by the largest component of the reference tangent | Central differences of the reference `from_disk`, times $h$ | 2048 disk positions with $1 - \lvert z \rvert$ from $10^{-6}$ to 1, log-uniform, at angles that step by the golden angle, with $h$ of a 512 by 512 image |
| Border estimate | The number of samples that the gradient form and the distance form classify differently, at half widths 0.5 and 1.5 pixels | The distance form in the reference | The samples at every second angle of the fold set whose distance form is under 4 pixels, with $h$ of a 512 by 512 image |
| Colour | The largest difference from the reference transfer function, and the round trip of every byte | The transfer function in the reference | All 256 values in each direction, and 65 evenly spaced values between the current and the obsolete threshold in each direction |
| Fold count | The mean and the largest number of steps, printed with no bound | None | The fold set, and the samples of the canonical image of `*(100)32` at 256 by 256 pixels with a 4 by 4 grid |

Every fold of the fold set and every sample of that canonical image converges within the default cap of 1500. No byte lies between the current and the obsolete threshold of the sRGB transfer function, so only the 65 values between the thresholds can reveal an obsolete threshold. In bands 3 to 6 every sample of the set lies within a small fraction of a pixel of a wall, so both forms classify every sample there as a border sample, and the comparison tells the forms apart only in bands 1 and 2. The border oracle of Section 8.1 checks the border estimate on whole images instead.

Beyond the range of accuracy, the tests assert no bound. They assert that each fold returns, that a converged fold has a finite point and finite side values, and that the renderer gives each sample one of the colours of Section 5.5.

### 6.5. Measured Figures

RND-16. A test that asserts a bound on a quantity MUST name its input set and print its largest value with its input and its margin, as TEST-13 of the coding standard requires, and MUST take the count bound of a measured count as the bound on that count, because 8 times a count of 0 is a bound of 0, which fails on the first decision that differs on another platform. This document MUST record a bound only where it is part of a contract of the crate, because the test repeats the measurement on every run, while a figure copied into this document goes stale with the next change to the code.

The count bound is a choice of this design. A bound on a floating-point quantity is at least 8 times its largest value, as TEST-13 requires. The Validating Changes section of `AGENTS.md` gives the command that prints the largest values. The tests measure on Linux x86-64, and Section 10 records that the bounds are unverified on other platforms.

### 6.6. Euclidean Domains

For a Euclidean domain, precision is lost in these places:

- The sines and cosines of the construction come from the platform mathematics library, as for a hyperbolic domain.
- Each step of the fold rounds with an absolute error that grows with the coordinates of the point, and the step count grows with the distance from the centre, so the error of the folded point grows faster than linearly in that distance.
- The side test compares $c_i \cdot x$ with $\tau \sigma_i$, a threshold that does not grow with the coordinates of the point. Section 6.2 states the tests at the vertices of the domain.

The last coordinate does not drift. The last row of every Euclidean pairing is exactly $(0, 0, 1)$, so a step on a finite point with last coordinate 1 returns a point with last coordinate exactly 1 in IEEE 754 arithmetic. The property tests assert this exactly.

For `*2222` with `aspect` 1 or 4, an image whose sides are powers of two, and a sample grid of 1, 2, 4, 8, or 16, every value of the construction, of the view, of the fold, and of the border estimate is a dyadic rational that `f64` holds exactly, and every square root that runs has an exact result. IEEE 754 arithmetic then rounds no operation, so the image does not depend on the platform. The periodicity tests assert exact equality for these cases only (TEST-13 of the coding standard), and Section 12.1 records the reading.

RND-25. For a Euclidean domain, the renderer and `fold` MUST meet the bounds that the tests of this section hold for plane positions within distance 65 of the centre of the image, and for a plane position beyond that distance MUST only return without a panic and report the fold as converged or unconverged, because the bounds are measured only up to that distance, and a promise beyond it would be false for samples that reach the fold cap.

The canonical view of an image whose longer side is at most 16 times its shorter side lies within the range of accuracy, because the corner sample of such an image lies less than 65 from the centre. In a more elongated image, such as 16384 by 1 pixels, samples lie far beyond the range, many of them reach the fold cap, and they take the border colour (`DEC-7`). The rows of Section 6.3 apply unchanged.

The Euclidean fold set is the Euclidean domain set at plane positions at 512 evenly spaced angles at each distance 1, 4, 16, and 65 from the centre. The fold cap, the tolerance constant, and the golden allowance keep their values. The tests measure the quantities of Section 6.4 over this set, with the distance in the plane in place of the hyperbolic distance and the Euclidean distance form of Section 8.1 in place of the distance form, and hold the bounds of each domain and distance. Every fold of the set converges within the default cap. The border comparison keeps at least 1000 samples within 4 pixels at each distance, and the fold counts also cover the canonical images of the Euclidean domain set at 256 by 256 pixels with a 4 by 4 grid.

A bound of a cell with a measured distance above 0 is 8 times that distance, rounded up to one significant digit. 8 times a distance of exactly 0 gives no usable bound, so a cell measured as exactly 0 takes the largest bound at the same distance, a choice of this design. The tests print the difference between the distances of the two border forms and assert no bound on it. They do not divide the difference by the distance to a wall, which is 0 for a sample on a wall.

### 6.7. Spherical Domains

For a spherical domain, precision is lost in these places:

- The sines and cosines of the construction and of the viewing matrix come from the platform mathematics library.
- Each step of the fold rounds, and a fold of `*n22` or `*nn` takes up to about $n$ steps.
- Near the rim, $d = 1 - \lvert z \rvert^2$ cancels, and the screen tangents grow as $1/\zeta$.
- The border estimate is first order, and the lift is not, so it degrades toward the rim.
- One sample spacing near the rim covers a long arc of the sphere, so the limb aliases. This is a property of supersampling, and no bound covers it.
- The shading takes about ten multiplications per channel.

The spherical fold set is the spherical domain set at 512 evenly spaced angles in each rim band of a 512 by 512 image, with the viewing matrix of the default rotation and of the rotation 20, 35, 0 degrees. The fold cap, the tolerance constant, and the golden allowance keep their values. The tests measure the folded point, as an angle between normalized points, and the fold decisions as Section 6.4 does, for each domain of this set, and every fold of the set converges within the default cap. They also measure these quantities against the reference arithmetic of Section 8.3, each bound set as RND-16 states:

| Quantity | Measure | Measured against | Input set |
| --- | --- | --- | --- |
| Lift | The angle between the lifted point and the reference lift, in each rim band | The lift of Section 5.9 in the reference | The spherical fold set |
| Ball tangents | The largest component of the difference of each tangent, divided by the largest component of the reference tangent, in each bin of rim distances | Central differences of the reference lift, times $h$ | 2048 ball positions with rim distances from 0.01 to 240 pixels, log-uniform, at angles that step by the golden angle, with $h$ of a 512 by 512 image |
| Viewing matrix | The largest entry of $V^{\mathsf{T}} V - I$, and the distance of the determinant from 1 | None | The seven angle triples $(0, 0, 0)$, the rotation 20, 35, 0 degrees, $(1, 2, 3)$, $(-2.5, 0.7, 5)$, $(1000, -1000, 0.1)$, $(\pi/2, \pi/2, \pi/2)$, and $(-7, 100, -0.25)$ radians |
| Border estimate | The number of samples that the gradient form and the brute-force oracle of Section 8.1 classify differently, at half widths 0.5 and 1.5 pixels, in each rim band at least one half width from the rim | The oracle | 4000 samples in each rim band of `*532` and of each domain of the spherical golden images |
| Border share | The fraction of border samples in each rim band, at a half width of 0.5 pixels, printed with no bound | None | As the row above |
| Shading | The largest difference of a shaded linear-light channel from the reference | The formula of Section 5.10 in the reference, with the same `f64` normal and normalized light | The normals of the front hemisphere above a grid of 33 values along each axis of the screen, the default light and the lights $(1, 1, 1)$, $(0, -1, 1)$, and $(2, 0, 1)$, and the channel values $0, 1/16, \ldots, 1$ |
| Fold count | The mean and the largest number of steps, with the largest below the group order of the signature | None | The spherical fold set, and the samples of the canonical image of each domain of the spherical golden images, of `*(100)22`, and of `*(100)(100)`, at 256 by 256 pixels with a 4 by 4 grid |

The ball positions of the ball tangents have continuous rim distances, so the test sorts them into bins, a choice of this design. A position falls in the bin whose lower end is the largest rim distance of the rim bands that does not exceed its own, and a position less than 0.5 pixels from the rim falls in a bin of its own.

The range of accuracy of the border estimate on the ball is the rim bands in which its count bound holds. Each count bound is set from the count measured in its own band, so it holds in all seven bands. Point 33 of Section 12.3 asks the owner how to define this range. Closer to the rim, the renderer and `fold` promise only a result without a panic.

RND-30. For a spherical domain, `fold` MUST meet the bounds of this section for every vector whose largest absolute coordinate is at least $10^{-300}$ and whose fold converges, the renderer MUST meet them for every ball position inside the rim whose fold converges, and the border estimate MUST meet its count bound in each rim band of the range of accuracy, and for a sample closer to the rim MUST only return without a panic and give the sample one of the colours of Section 5.5. A smaller vector lies beyond the measurement of Section 6.1, an unconverged fold stops at no reliable point, and the border estimate degrades toward the rim, so a promise beyond these limits would be unsupported or false for many inputs.

The fold of a finite group stops within the cap for the samples of the canonical images and for the images of the centre of the domain. It need not stop for every point. At orders above the limit of DOM-3 of the domain design, some points within $10^{-14}$ radians of the vertex of order 1000 of `*(1000)22` and of its antipode stayed unconverged at the default cap, and near a vertex of `*(999)22` a fold took more steps than the longest word of the group. The tests fold 1000 points within $10^{-14}$ radians of the vertex of order 100 of `*(100)22` and of its antipode, assert that each fold returns within the cap, and print the number of unconverged folds. An unconverged sample takes the border colour, which is the colour that a sample on a vertex shows. This is part of `UNV-1` (Section 10).

For `*332`, `*432`, `*532`, and for `*n22` and `*nn` with $n$ from 2 to 12 and $n = 100$, each image of the centre folds back to within the bound of the folded point of its domain. A domain whose $n$ is not in the spherical domain set takes the bound of the next larger $n$ of that set, so $n = 4$ takes the bound of $n = 5$, and $n$ from 6 to 11 the bound of $n = 12$. No bound from the group order enters the code, and the fold cap stays the only bound.

### 6.8. Rotation Domains

A rotation domain loses precision in the places that Sections 6.4, 6.6, and 6.7 name for its geometry, and in these places of its own:

- Each pairing is the product of two reflections, so it carries the rounding of a product of matrices on top of that of the reflections.
- Near the rim of the disk, the fold of a rotation domain can apply two sides in turn for hundreds of steps, so the defect of a pairing from an isometry can add up. With plain `f64` products of the stored reflections as pairings, the point of a fold of `(1000)32`, an order above the limit of DOM-3 of the domain design, in band 6 drifted off the hyperboloid until it was no point and stopped unconverged, while the fold of the same point for `*(1000)32` converged. With each entry of the products computed in high precision from the stored reflections and rounded once to the nearest `f64`, the same fold converged, which is why the domain design computes the pairings as compensated products (DOM-25, and Section 14 of the domain design). With the compensated products, every fold of the rotation fold sets converges.
- The image vertices of a double lie against image walls with smaller margins than the vertices of a reflection domain (Section 6.2).

This section covers the rotation domains of the rotation domain set. The doubles of the reflection polygons are rotation domains too, and Section 6.9 covers them.

The rotation fold sets are the rotation domain set at the positions of the fold sets of the reflection domains: the hyperbolic rotation domains at 512 evenly spaced angles in each radius band from 1 to 6, the Euclidean rotation domains at 512 evenly spaced angles at the distances 1, 4, 16, and 65, and the spherical rotation domains at the positions of the spherical fold set of Section 6.7. The fold cap, the tolerance constant, and the golden allowance keep their values. The tests measure the folded point and the fold decisions with the measures of Sections 6.4, 6.6, and 6.7 for the geometry, for each domain and each band, distance, or domain, with the counts not summed over the domains. Every fold in the range of accuracy converges within the default cap. They also measure, with bounds set as RND-16 states, the angle between the folded points of $x$ and of $P_i x$ on each spherical rotation domain over a grid of 33 heights and 32 turns, the points of every fifth sample of the rotation fold sets that the double oracle of Section 8.1 rejects, the angle from the centre of each image of the centre of a spherical rotation domain, the border pixels of the border-subset test, and the fold counts of the canonical images of `(100)32`, `(100)(100)(100)`, `(100)22`, and `(100)(100)` at 256 by 256 pixels with a 4 by 4 grid.

The rotation domains take bounds of their own, because the folded points of some rotation domains in some bands lie farther from their replay than the bounds of their reflection domains, and the images of the centre of `(100)22` fold back much farther than those of `*(100)22`. The likely cause, which no test has confirmed, is that the elements of the largest depths fold back the other way round around the cyclic group, so their error is that of a high power of a pairing.

The pairing property of Section 8.1 takes, for a hyperbolic or Euclidean rotation domain, the bound $b$ of the folded point. For a spherical rotation domain, $b$ does not bound the distance between the folded points of $x$ and $P_i x$, which grows with the order of the domain. The likely cause, which no test has confirmed, is that the error of the `f64` pairings themselves grows with the length of the word, as for the images of the centre, and the replay multiplies by the same pairings, so it cannot show that error. So the design specifies a pairing bound of its own for each spherical rotation domain, measured on the grid of the table above, at 8 times the largest distance rounded up to one significant digit, or $b$ when $b$ is larger.

The double oracle keeps few or no points in band 6 of some domains, because few points of the fold set lie at its clearance there. So it asserts at least one kept point for each domain, and not for each band, a choice of this design. The border-subset test asserts at most as many border pixels for the rotation domain as for its reflection domain, and not fewer, because at the corner orders near 1000 on the sphere, above the limit of DOM-3 of the domain design, every pixel of both images within ball radius 0.9 was a border pixel.

RND-35. For a rotation domain, the renderer and `fold` MUST meet the bounds of this section in its range of accuracy, and for a point beyond that range MUST only return without a panic and report the fold as converged or unconverged, because the bounds of Sections 6.4, 6.6, and 6.7 do not hold for a double, whose pairings are products of reflections, and a promise beyond the measured range would be false for many samples. For a hyperbolic rotation domain whose corner orders are all near 100, such as that of `(100)(100)(100)`, the range excludes the points within a hyperbolic distance of $10^{-8}$ of a vertex, because the fold of such a point can still lie outside a side after the default cap. The distance at which a fold fails differs between machines, so $10^{-8}$ keeps a margin over the largest one measured.

The test of Section 8.1 folds points at 13 hyperbolic distances from $10^{-5}$ to $10^{-11}$ from every vertex of the rotation domains of `(100)(100)(100)`, `(100)(100)(99)`, and `(100)32`. For each of the three, it asserts that no fold of the rotation domain is unconverged at a distance of at least $10^{-8}$. It prints the count at every distance and asserts nothing closer, so the exception of RND-35 lies at $10^{-8}$. At orders near 1000, above the limit of DOM-3 of the domain design, unconverged folds were not limited to the cycle of two vertices 1 and 3 of the kite (Section 6.8 of the domain design), and at some vertices they reached larger distances than the folds of the same points in the reflection domain. Such a sample takes the border colour. This is part of `UNV-1` and `UNV-2` (Section 10).

### 6.9. Reflection Polygons

A reflection polygon loses precision in the places that Section 6.4 names, and its double also in those of Section 6.8. The components of its unit normals grow with the distance of a side from the centre. The expected cause of a further loss of precision, which no test measures, is that the rounding of the reflections and of the side values grows with these components (Section 5.13). A wide shape may then fold less accurately than any triangle of Section 6.4. The walls of a shape depend on its settings, which are untrusted.

RND-42. For a reflection polygon and the double of one, the renderer and `fold` MUST meet the bounds of this section for the shapes of the polygon domain set in each radius band from 1 to 6, and for every other shape and beyond the range of accuracy MUST only return without a panic and report the fold as converged or unconverged, as RND-15 requires, because the accuracy of the fold depends on the size of the walls, which the settings change, and a bound measured on some shapes would be false for others.

`build_with` returns only a polygon or a double that passes `verify()` (DOM-44 of the domain design), so every such domain that reaches the renderer has walls, pairings, and angles within the tolerances of the domain design. That bounds the domain, not the fold, which RND-42 bounds only on the measured shapes.

The tests measure the quantities of Section 6.4 over the polygon fold set, the border estimate over every fifth angle of it, for each shape and band and not summed over the shapes, with the vertices of Section 6.2, and print the largest covector component of each shape with no bound. They hold the bounds of the folded point as `POLYGON_REPLAY_BOUNDS` and the count bounds of the fold decisions as `POLYGON_DECISION_COUNTS`. Every fold of the set converges within the default cap, and the canonical images of the polygon domain set at 256 by 256 pixels with a 4 by 4 grid have their unconverged samples within a count bound.

Each bound of the folded point is the smallest of 1, 2, or 4 times a power of ten that is at least 8 times the largest distance. A bound on the sum of the border counts over the shapes could hide a regression in one shape, so the test asserts the count of each shape and band against its own count bound.

The border estimate differs more from the distance form on the polygons than on the triangles, in bands 1 and 2, where the comparison tells the forms apart. A bad normalization of the covectors does not cause the difference, because every covector has a form norm of 1 up to rounding. A possible cause, which no test has confirmed and no source states, is that in these bands a pixel spans a hyperbolic distance of order 1, so the linear gradient form underestimates the distance, and the polygons place samples farther from their nearest wall, in hyperbolic units, than the triangles do.

No measured fold narrows the range of accuracy of a shape of the polygon domain set, so the range of each such shape is that of Section 6.4.

### 6.10. Wallpaper Domains

A wallpaper domain loses precision in the places that Section 6.6 names for a Euclidean domain. The last row of each of its pairings is exactly $(0, 0, 1)$, so the last coordinate does not drift. `aspect` and `skew` change the walls and the pairings, and both are untrusted, so the step count and the error of the fold depend on the settings. The step count grows with the distance from the centre and with the elongation of the domain.

This design assumes that, for the rectangular groups with `aspect` 1 or 4, an image whose sides are powers of two, and a sample grid of 1, 2, 4, 8, or 16, every value of the construction, of the view, of the fold, and of the border estimate is a dyadic rational that `f64` holds exactly, so that IEEE 754 arithmetic rounds no operation. Section 6.6 states the same reading for `*2222`, and DOM-51 of the domain design gives the translations, half-turns, and glide reflections exact entries. Section 12.1 records the assumption. The exact cases of the wallpaper periodicity and glide-symmetry tests rely on it and check it, because a rounded operation would show as a differing pixel. A slanted side of `o` or of the sheared `2222` has a unit normal with a square root that rounds, and the kites hold sines, cosines, and square roots, so the periodicity cases of these domains assert the golden allowance (TEST-13 of the coding standard).

RND-45. For a wallpaper domain, the renderer and `fold` MUST meet the bounds of this section for the settings of the wallpaper domain set at plane positions within distance 65 of the centre of the image, and for every other setting and beyond that distance MUST only return without a panic and report the fold as converged or unconverged, because the step count and the accuracy of the fold depend on `aspect` and `skew`, and a bound measured on some settings would be false for others. For a wallpaper domain, RND-45 takes the place of RND-25.

`build_with` returns only a wallpaper domain that passes `verify()` (DOM-54 of the domain design), so every such domain that reaches the renderer has walls, pairings, and vertex cycles within the tolerances of the domain design. That bounds the domain, not the fold, which RND-45 bounds only on the measured settings.

No source proves that the fold terminates with translations, half-turns, and glide reflections as pairings, and no argument exists for the kites of `4*2` and `3*3` (`UNV-2`). The tests measure the step counts and that every fold of the wallpaper fold set converges.

The tests measure the quantities of Section 6.6 over the wallpaper fold set, for each domain and distance and not summed over the domains, with the vertices of Section 6.2, and hold the bounds in `WALLPAPER_BOUNDS` (Section 8.1). Every fold converges within the default cap with a last coordinate of exactly 1, and an exact fold that does not converge counts as a differing decision. The border comparison keeps at least one sample within 4 pixels for each domain and distance, and at least 1000 for each distance summed over the domains. The largest step count at distance 65 lies within the step bound of its group, and the canonical images of the wallpaper domain set at 256 by 256 pixels with a 4 by 4 grid have no unconverged sample.

Each bound of the folded point is the smallest of 1, 2, or 4 times a power of ten that is at least 8 times the largest distance. A cell that measures exactly 0 takes the largest bound at the same distance over the set, as Section 6.6 does for `*2222`.

The step bound of each group at distance 65 is the smallest multiple of 50 above the largest count of its group. The step bounds do not take the factor 8 of RND-16, because 8 times the largest count of `o` is above the default cap of 1500. Their margins are small, so a change that moves one fold decision near distance 65 can break them. Section 12.1 records this.

No measured fold narrows the range of RND-45.

### 6.11. Spherical Group Domains

A spherical group domain loses precision in the places that Sections 6.7 and 6.8 name for a spherical domain. The pairings of the kites that are kept from the doubles are the compensated products of those doubles, bit for bit, each mirror is the reflection of its stored normal, and each rotary reflection of a lune is the stored reflection of its side with two rows negated, which rounds nothing (DOM-59 of the domain design). A fold of `n*`, `nx`, or `2*n` takes up to about $n$ steps, so its error grows with $n$, as for `*n22` and `*nn`. The domain of `1` takes no step, so its fold is exact.

RND-48. For a spherical group domain, `fold` MUST meet the bounds of this section on the domains of the spherical group domain set, for every vector whose largest absolute coordinate is at least $10^{-300}$ and whose fold converges, and the renderer MUST meet them for every ball position inside the rim whose fold converges. For a domain outside that set, and for a fold that does not converge, they MUST only return without a panic and report the fold as converged or unconverged. A bound measured at some orders $n$ would be unsupported at the others, because the step count and the error grow with $n$, and no source proves that the fold terminates for these domains (`UNV-1`, `UNV-2`). For a spherical group domain, RND-48 takes the place of RND-30 except for the border estimate.

`build_with` returns every spherical group domain except `*` only when it passes `verify()` (DOM-60 of the domain design), so every such domain that reaches the renderer has walls, pairings, and vertex cycles within the tolerances of the domain design. That bounds the domain, not the fold.

The tests measure these quantities against the reference arithmetic of Section 8.3. Each bound other than a step bound is set as RND-16 states, as the smallest value of the form 1, 2, or 4 times a power of 10 that is at least 8 times the largest measured value. A quantity that measures exactly 0 takes the bound 0, which the test asserts exactly, because RND-16 gives no positive bound for it. The tests hold the bounds, including those of the images of the centre, in the keyed table of the module `common` (Section 8.1). They measure, for each domain of the spherical group domain set:

- The folded point and the fold decisions as Section 6.7 does, over the spherical group fold set, with the counts not summed over the domains. Every fold and every 256-bit fold converges.
- The vertices of Section 6.2, and the largest step count against the step bound of the family.
- The angle between the folded points of $x$ and of $P_i x$ on a grid of 17 by 16 points over every side, with a clearance of $10^{-6}$ and at least 500 samples for each domain, and in the property test of Section 8.1.
- The largest angle from the centre of a folded image of the centre, over a breadth-first enumeration of the pairings that finds as many elements as the group order.
- The largest absolute difference between the stored normals of two sides on one wall, for the pairs whose normalized normals agree within $10^{-12}$, printed with no bound.

Each pairing bound was set by RND-16 from the largest angle on a grid of 33 by 32 points, a one-time measurement that no test repeats, and the grid of the test checks it.

The step bound of each family is one step above the largest count of a fold that a measurement outside the repository found on dense samples of the sphere: $\lfloor n/2 \rfloor + 2$ for `n*`, $n + 1$ for `nx`, $n + 2$ for `2*n`, 7 for `3*2`, 2 for `*` and `x`, and 0 for `1`, which takes no step. This is a choice of this design, not a bound set by RND-16. Its margin over the largest count is small by construction, at most one step, because a step count is exact for a given input, so the bound checks that the counts do not grow and gives no allowance for a change of a fold decision. For the same reason the bounds do not take the factor 8 of RND-16. At the order 1000, above the limit of DOM-3 of the domain design, 8 times the largest count of `(1000)x` was also above the default cap of 1500, as for the wallpaper domains (Section 6.10).
The family bounds hold on the domains of the set only. Near a vertex, `(999)*`, an order above the limit of DOM-3, took more steps than $\lfloor n/2 \rfloor + 2 = 501$. The near-vertex test also runs on domains outside the spherical group domain set, such as `(99)*`, and RND-48 limits the bounds to that set, so the test asserts that every fold converges within the cap and asserts no step bound.

The near-vertex test folds 256 directions at eight distances from $10^{-6}$ to $3 \times 10^{-17}$ radians from every vertex of `(100)*`, `(100)x`, `2*(50)`, `(99)*`, `(99)x`, and `2*(49)`, and asserts that no fold is unconverged. A wider measurement outside the repository found no unconverged fold near the vertices of the same families at the orders 1000 and 999, and 500 and 499 for `2*n`, either. This is evidence and no proof, so the design states no range that excludes the points near a vertex.

### 6.12. Hyperbolic Models

The fold and the border estimate of a sample in the half-plane, the band, or the Klein model are those of the sample of the disk with the same disk position, up to the rounding of the model map and of its Jacobian, which Section 8.7 of the geometry design bounds. The range of accuracy of these models is therefore the set of samples whose disk position lies in the range of accuracy of Section 6.4.

RND-53. For a hyperbolic domain in the half-plane, the band, or the Klein model, the renderer MUST meet the bounds of Section 6.4, and of Section 6.9 for a reflection polygon and its double, for every sample whose disk position lies in the range of accuracy, and the bound of each model on the screen tangents of this section on its input set, and for every other sample MUST only return without a panic and give one of the colours of Section 5.5, because the bounds of the fold are measured on disk positions, and the canonical views of the half-plane and the band hold samples, such as those next to the real axis, whose disk positions lie beyond that range.

The tests measure the screen tangents of each of the three models against central differences of the reference map from the model to the hyperboloid, which composes the reference model map of Section 10.2 of the geometry design with the reference `from_disk`, times $h$. The measure is that of the row "Tangents" of Section 6.4. Each model has a bound of its own, set as RND-16 states, which the unit test of the tangents in `view.rs` holds. The bound of the Klein model is the largest, because the bound on the Jacobian of `disk_from_klein` in Section 8.7 of the geometry design grows with $1/(1 - \lvert s \rvert^2)$ near the unit circle. The input set is the disk positions of the row "Tangents" of Section 6.4 and the centre of the disk, each mapped to the model by the reference model map and rounded to `f64`, with the scale $h$ of the view of each model for a 512 by 512 image (Section 5.16). A difference in a tangent changes only the border estimate, and the border oracles of Section 8.1 check that on whole images in each model.

No bound of the fold depends on the model, because the fold reads no model (RND-52). Near the edge of a model, the loss of digits of the model map adds to that of `from_disk` (Sections 7.6 and 7.7 of the geometry design), and the samples where it matters lie beyond the range of accuracy, except for the screen tangents of the Klein model, whose bound above allows for it.

## 7. Errors

### 7.1. Structure

RND-17. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"build render options"`, `"build coset fill"`, `"build renderer"`, `"render rows"`, `"encode image"`, `"build masks"`, `"render mask"`, `"decode image"`, `"build Wythoff construction"`, `"build Wythoff fill"`, `"build whole face"`, `"read edge pairing"`, `"render face mask"`, or `"build whole-face fill"`.
- `detail`: A private enum with one variant for each cause, which carries the values that the text of Section 7.2 shows, the error of `escher-domain` that says why a domain has no side presentation, the error of `escher-geometry` that caused a numerical failure of `Wythoff::new`, or the error of `png` that caused an encoding or a decoding failure.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, as Section 10.1 of the domain design states for the same reason: `source()` returns the error that some variants of the detail hold. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `png` error for an `Encoding` error, for a `Decoding` error that `png` reports, and for the memory limit of the decoder, the `escher-domain` error for a domain without a side presentation, the `escher-geometry` error for a `Numerical` error that one caused, and `None` otherwise. No type from a dependency appears in the public surface of the error (ERR-2).

`Error` derives only `Debug`, because the `png` errors that it may hold, `png::EncodingError` and `png::DecodingError`, implement neither `Clone` nor `PartialEq`.

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
| `InvalidOption` | `build coset fill` | The domain has no side presentation | `the domain has no side presentation`, with the error of `escher-domain` as `source()` |
| `InvalidOption` | `build coset fill` | The coset table of the colouring has another number of generators than the domain has sides | `the colouring has 3 generators, and the domain has 4 sides`, with the two numbers, and `1 generator` and `1 side` for the number 1 |
| `InvalidOption` | `build coset fill` | A relator of the side presentation does not return some coset of the colouring to itself | `relator 3 of the side presentation of the domain moves a coset of the colouring`, with the index of the first such relator, counted from 0 in the order of `Presentation::relators()` |
| `InvalidOption` | `build coset fill` | The number of colours differs from the colour count of the colouring | `the colouring needs 3 colours, and 2 were given`, with the two numbers, and `1 colour` and `1 was given` for the number 1 |
| `InvalidOption` | `build renderer` | The `CosetFill` of a `Fill::Cosets` was built for another side presentation than that of the domain | `the coset fill was built for a domain with another side presentation` |
| `InvalidOption` | `build renderer` | A `Fill::Cosets` has an image fill, and the domain has no mask polygon (RND-63), checked after the side presentation of the row above | `the domain has no mask polygon` |
| `InvalidBuffer` | `render rows` | A buffer whose length is 0 or not a multiple of $3W$ | `the buffer of 1535 bytes does not hold whole rows of 1536 bytes`, with the two lengths |
| `InvalidBuffer` | `render rows` | A band that ends below the last row | `the rows 510 to 513 are outside the image of 512 rows`, with the first and last row of the band and the height |
| `InvalidBuffer` | `encode image` | A width or a height of 0, or a buffer whose length is not $3WH$ | `the buffer of 1535 bytes does not hold an image of 16x32 pixels`, with the length and the size |
| `Encoding` | `encode image` | `png` reports an error | `the PNG encoder failed` |
| `InvalidOption` | `build masks` | The domain has no side presentation, the coset table has another number of generators than the domain has sides, or a relator moves a coset | The texts of the same three conditions of `build coset fill` |
| `InvalidOption` | `build masks` | The domain has no mask polygon (RND-63), checked after the three conditions above | `the domain has no mask polygon` |
| `InvalidOption` | `render mask` | A colour at or above the number of masks | `the colour 4 is not below the 4 colours of the colouring`, with the colour and the count, and `1 colour` for the count 1 |
| `InvalidOption`, `LimitExceeded` | `render mask` | A size outside the row `size` of Section 4.1 | The texts of the same values of `build render options` |
| `LimitExceeded` | `decode image` | A width or a height above 16384 | `the image size 16385x512 is larger than 16384 pixels on a side`, with the size of the header |
| `LimitExceeded` | `decode image` | More than $2^{26}$ pixels | `the image size 16384x8192 has more than 67108864 pixels`, with the size of the header |
| `Decoding` | `decode image` | A header with 16 bits per channel, for an image within the size limits | `the image has 16 bits per channel, and at most 8 are supported` |
| `LimitExceeded` | `decode image` | `png` reports that it reached its memory limit | `the PNG decoder reached its memory limit` |
| `Decoding` | `decode image` | `png` reports an input or output error of the reader | `the image cannot be read` |
| `Decoding` | `decode image` | `png` reports any other error, such as a stream that does not start with the PNG signature, a bad checksum, or a stream that ends early | `the data is not a valid PNG image` |
| `InvalidOption` | `build Wythoff construction` | The domain does not have exactly three sides of kind `Mirror` (step 1 of Section 5.24) | `the domain is not a reflection triangle` |
| `InvalidOption` | `build Wythoff construction` | The marks give no face type (step 4 of Section 5.24) | `the marks give no face` |
| `Numerical` | `build Wythoff construction` | A call of step 5 or 6 returned an error, or a value of step 7 is zero or not finite (step 8 of Section 5.24) | `the construction is numerically degenerate`, the text of the same failure in the domain design, with the error of `escher-geometry` as `source()` when one caused it |
| `InvalidOption` | `build Wythoff fill` | The number of colours differs from the number of face types | `the construction needs 3 colours, and 2 were given`, with the two numbers, and `1 colour` and `1 was given` for the number 1 |
| `InvalidOption` | `build renderer` | A `Fill::Wythoff` whose construction holds other covectors than the sides of the domain (RND-68) | `the Wythoff fill was built for another domain` |
| `InvalidOption` | `build whole face` | The construction holds other covectors than the sides of the domain (step 1 of Section 5.29) | `the Wythoff construction was built for another domain` |
| `InvalidOption` | `build whole face` | The construction is not a regular construction (step 2 of Section 5.29) | `the construction is not a regular tiling` |
| `InvalidOption` | `build whole face` | The domain has no side presentation (step 4 of Section 5.29) | `the domain has no side presentation`, with the error of `escher-domain` as `source()` |
| `InvalidOption` | `read edge pairing`, `render face mask`, `build whole-face fill` | Check 1 of Section 5.30 fails | The texts of the same two conditions of `build coset fill` |
| `InvalidOption` | `read edge pairing`, `render face mask`, `build whole-face fill` | The colour count of the colouring is not $2n$ (check 2 of Section 5.30) | `the colouring has 8 cosets, and an arrangement of these faces has 12`, with the two numbers, and `1 coset` for the number 1 |
| `InvalidOption` | `read edge pairing`, `render face mask`, `build whole-face fill` | The stabilizer does not reach every coset once (check 3 of Section 5.30) | `the colouring is not an arrangement of the faces` |
| `InvalidOption`, `LimitExceeded` | `render face mask` | A size outside the row `size` of Section 4.1 | The texts of the same values of `build render options` |
| `InvalidOption` | `build whole-face fill` | The number of paints differs from the number of orbits of the arrangement, which is 1 | `the arrangement needs 1 paint, and 2 were given`, with the number given, and `1 was given` for the number 1 |
| `InvalidOption` | `build renderer` | A `Fill::WholeFace` whose construction holds other covectors than the sides of the domain (RND-76) | `the whole-face fill was built for another domain` |

So `RenderOptions::builder(fill).samples(17).build()` returns an `Error` whose `kind()` is `LimitExceeded` and whose `Display` text is exactly `cannot build render options: the sample grid 17 is larger than 16`.

Every fixed text is in lower case apart from `PNG` and `Wythoff` and has no trailing punctuation (ERR-4 of the coding standard). The values in the text are numbers that the crate formats, so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

The texts of `decode image` hold no path, because the crate gets a reader and no path, and the command line names the image by its position (Section 4.6 of the command-line design).

`fold` and `render` return no error, and `fold` reports a failure as an unconverged result. `Renderer::new` returns an error only for a coset fill of another side presentation, for an image fill on a domain without a mask polygon (RND-41), for a Wythoff fill of another domain (RND-68), and for a whole-face fill of another domain (RND-76). The texts of the whole face hold no id of an arrangement, for the reason that the next paragraphs give for a colouring.

The size limits of `decode image` have the texts of the same limits of `build render options`, because an image fill has the limits of an output image (`DEC-92`).

The texts of `build coset fill` and `build renderer` hold no id of a colouring, because the command line names the id in its own diagnostic (Section 6.8 of the command-line design), and a second id in the source text would repeat it.

## 8. Testing

### 8.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-render/tests/render/main.rs`, with the modules `common`, `reference`, `golden_images`, `folds`, `properties`, `borders`, `images`, `encoding`, `masks`, `image_fills`, `wythoff`, and `whole_faces` (TEST-10). Its root has a `//!` line. The snapshot tests of the command and the process-level smoke test live in `escher-cli`, and Section 8.1 of the command-line design specifies them.

The tests that read private items, such as the screen tangents, the linear value of a `Colour`, and the walls inside the renderer, are unit tests. `src/lib.rs` loads `tests/render/reference.rs` and `tests/render/golden_images.rs` under `#[cfg(test)]` through `#[path]`, so the unit tests and the integration tests use the same reference module, which shares no code with the crate (RND-19), and the same golden configuration, allowance, and comparison. The module `reference` carries `#![expect(dead_code, reason = "the unit tests of the crate and the integration tests each use a different part of this module")]`, because each test binary uses a different part of it. It also defines the count bound once, for both binaries.

The named input sets live in two modules:

- `common` holds a copy of the tolerance constant $\tau$, the fold set of Section 6.4, whose triples, `FOLD_TRIPLES` of `golden_images`, are the plan triples of Section 12.1 of the domain design, $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(100, 3, 2)$, and $(100, 100, 100)$, and the rim set: the triples of the fold set at 64 evenly spaced angles at each radius $1 - 10^{-k}$ for $k$ from 7 to 15, and at the largest `f64` below 1 on each axis, 578 positions. It holds the 5 vertex triples: $(38, 20, 4)$, which has the largest ratio of Section 6.2 among the hyperbolic triples with orders up to 40, and $(99, 7, 6)$, $(98, 4, 4)$, $(100, 4, 4)$, and $(100, 100, 2)$. It holds the Euclidean, spherical, rotation, polygon, wallpaper, and spherical group fold sets of Section 6, built by shared helpers, and the bounds that the tests of Section 6 hold. The bounds of the wallpaper domains are the table `WALLPAPER_BOUNDS`, keyed by group and setting, those of the spherical domains the table `SPHERICAL_BOUNDS`, and those of the spherical group domains the table `SPHERICAL_GROUP_BOUNDS`, whose rows also hold the bound of the images of the centre and the view of the border test. The rows of these two tables hold, row by row, the signatures of the spherical domain set and the spherical group domain set, `SPHERICAL_DOMAINS` and `SPHERICAL_GROUP_DOMAINS` of `golden_images`, and a compile-time assertion in `common` keeps them in step. `common` re-exports `FOLD_TRIPLES`, `EUCLIDEAN_DOMAINS`, `SPHERICAL_DOMAINS`, `SPHERICAL_GROUP_DOMAINS`, `text`, and `rotation_text` from `golden_images` for the integration tests. The functions `wallpaper_bounds` and `spherical_group_bounds` find the row of a domain by its signature, and for a wallpaper domain by the bits of its `aspect` and `skew`, and panic when a domain has no row, so a reordering of a domain set cannot give a domain the bounds of another. `common` also holds the helpers that render an image in 8 bands on scoped threads, each panic of which starts with the name of the signature or shape. No module holds a set of model positions. The tangent test of Section 6.12, a unit test of `view.rs` because it calls a private function, maps its disk positions to the models with the reference model maps of the module `reference`, and the tests of whole images in a model take the model position of each pixel from `MODEL_MAPS` of `golden_images`.
- `golden_images` holds the golden configuration of Section 8.4 with its signatures, settings, file names, colours, size, sample grid, and options, the coset golden images as a list of their own, `COSET_GOLDEN_IMAGES`, the allowance of 16 pixels, the linear-light table of RND-20, the comparison, and the helpers that find a colouring by its id and build its `CosetFill`. It defines the plan triples as `PLAN_TRIPLES`, the polygon domain set as `POLYGON_DOMAINS`, the wallpaper domain set as `WALLPAPER_DOMAINS`, the fold triples as `FOLD_TRIPLES`, the Euclidean domain set as `EUCLIDEAN_DOMAINS`, the spherical domain set as `SPHERICAL_DOMAINS`, the spherical group domain set as `SPHERICAL_GROUP_DOMAINS`, the functions `text` and `rotation_text`, which write the reflection signature of a triple and strip the star of a signature for its rotation domain, the function `domain_with_settings`, which builds a domain through `build_with`, the function `mask_domains`, which gives the domains of "Polygon rules", `mask_rule` and `exact_mask`, which build the reference mask of a domain, the shared bilinear sampling `bilinear` with `texel_index`, the generated gradient `gradient` with its pixels `gradient_pixels`, and the helpers of the periodicity tests with the translation tolerance `TRANSLATION_TOLERANCE` of $6 \times 10^{-15}$. These sets live here and not in `common`, because unit tests need them, and the unit tests load only `reference` and `golden_images`. For the same reason it holds the pixel size of the disk view and the screen position of a pixel centre, which `MODEL_MAPS` and the pixel helpers of `common` use, the angle count, the radius of each band, and the positions on a circle from which `common` builds the fold set, and the reference walls of a domain, and `MODEL_MAPS`, the one table of the half-plane, the band, and the Klein model. The row of each model holds its variant of `HyperbolicModel`, its model in the module `reference`, its map to the disk in `escher-geometry`, the offset of its model positions, and whether its view fills the image height, and its methods give the scale of the canonical view and the model position of a pixel centre. Every test that pairs a model with a map or a view reads this table, so that no test can pair a model with the map of another.

The bounds of the images of the centre of the rotation domains sit beside their test in the module `folds`, as `ROTATION_CENTRE_BOUNDS`.

The tests form these layers, with the input sets of Section 6:

| Layer | What it checks | What it catches |
| --- | --- | --- |
| Reference replay | The folded point of each fold of a fold set against the replay of its side sequence in the reference arithmetic, and the side sequence against an independent 256-bit fold with $\tau = 0$, for every family. The screen tangents of the disk, of the half-plane, the band, and the Klein model, and of the ball against central differences of the reference maps, the lift, the transfer function in both directions with its thresholds, and the shading against the reference | A wrong formula, a lost digit, a wrong order of the side tests, and a fold decision that moves |
| Fold properties | Property tests for each family. "Inside": a converged fold violates no side by more than the threshold. "Idempotent": folding a folded point takes zero steps and returns the same bits. "Scale": the fold of $2^k x$ for $k$ from $-40$ to 40 takes the same decisions and returns exactly $2^k$ times the point. "Any input": any three `f64` bit patterns and any cap up to 2000 return without a panic, with consistent counts. For the polygons, the wallpaper domains, and the spherical group domains, "Inside" and "Any input" also assert that `reversals()` equals the number of reversing sides that the fold reports | A fold that does not stop inside the domain, renormalizes, panics, or miscounts reversals |
| Pairing | Property tests that fold a point $x$ and its image $P_i x$ and compare the folded points within the bound of the outer band, distance, or domain, for points whose folded point lies, in the reference, at least the larger of $10^{-6}$ and twice that bound from every wall. On a reflection domain the step counts differ by exactly 1 and the parities differ. On a rotation domain the parities are equal, and the step counts are not compared. On a wallpaper or spherical group domain the parity changes exactly when the pairing has a negative determinant, taken from the matrix and not from the kind of the side | A wrong pairing, a wrong orientation flag, and a wrong parity |
| Exact permutation | A property test that, for a domain, a colouring, a point $x$, and a side $i$, the coset colour of $P_i^{-1} x$ is the image of the coset colour of $x$ under the inverse column of the generator of side $i$. It draws reflection, rotation, polygon, wallpaper, and spherical group domains, with colourings whose subgroups are not normal, takes $P_i^{-1} x$ as the pairing of the partner side, keeps hyperbolic points in bands 1 to 3, and discards a case within $10^{-6}$ of a wall in the reference distances | A trace in fold order, a trace without the inversion, a trace from another coset, and a wrong side presentation |
| Valid neighbours | Two tests over reflection, rotation, polygon, wallpaper, and spherical group domains and over every colouring of `ColouringSet::All` up to a colour count that the tests set for each domain. A sample pair for a tile $W$ of the domain $D$ and a side $i$ is $W y$ and $W P_i^{-1} y$ with $y$ in $D$, two points that lie in two tiles that share side $i$. The first test takes $W$ as the identity and $y$ as the centre of $D$ for every side. It asserts that `contained_generator()` is `None` exactly when no pair shares a colour, and that the pair of side $i$ shares a colour exactly when the generator of side $i$ lies in the subgroup. The second is a property test that takes $y$ as a converged fold of a random point and $W$ as a product of up to three side pairings. It asserts that no pair shares a colour when `contained_generator()` is `None`, and, when $W$ is the empty product, that the pair shares a colour exactly when the generator of side $i$ lies in the subgroup | A validity test that disagrees with the colours of the tiles, which pins `UNV-35` on the sampled domains |
| Border oracles | The gradient form against the distance form in the reference, and the Euclidean distance form $\lvert \hat{f}_i \rvert < g w$ with $\hat{f}_i$ the side value divided by the last coordinate. On whole images, every change of tile between neighbouring pixels lies within one pixel of a border pixel, and every border pixel within one pixel of a change. In the disk, in the other models by the disk position of each pixel, and on the ball, the whole-image oracles check the pixels whose 3 by 3 neighbourhood lies within radius 0.9 of the centre, and in the plane every pixel whose 3 by 3 neighbourhood lies in the image. A tile is the image of the domain centre under the inverse of the pairings that `fold` reports, in reverse order, and two tiles differ when their centres are at least $10^{-4}$ apart. On the ball, a brute-force oracle marks a sample as near a wall when one of 64 points on the circle of the half width lifts to another tile. The rectangle of `*2222` against an oracle that runs no fold. The whole-image oracle also runs on the canonical images of `*732` and `732` in the half-plane, the band, and the Klein model | A wrong gradient, tangents not carried through a step, a border on the wrong wall, and a wrong model Jacobian |
| Hyperbolic models | "Each pixel": `*732` is rendered at 48 by 32 pixels in the half-plane, the band, and the Klein model, with one sample per pixel, no border, the parity fill, and a background colour that no fill uses. `MODEL_MAPS` gives the model position of the centre of each pixel. A pixel whose model position the map of `escher-geometry` rejects shows the background. For every other pixel, the reference model map gives its disk position, rounded to `f64`, and the pixel shows the parity colour of the fold of that disk position through `from_disk`. The test asserts, and does not skip, that each such disk position lies at least $10^{-3}$ inside the unit circle and that its fold in the reference converges at least $10^{-6}$ from every wall. "Straight borders", a unit test of `view.rs`: at every sample of the canonical Klein view of `*732` and of `*3333` that has a point $x$, and for each side $i$ of the domain, the sign of $c_i \cdot x$ equals the sign of $c_i \cdot (s_1, s_2, 1)$, which says on which side of the chord of Section 10 the sample position $s$ lies, except where $\lvert c_i \cdot (s_1, s_2, 1) \rvert$ is within a small multiple of the machine epsilon times the tolerance scale $\sigma_i$. "No effect": every variant of `model` gives the bytes of the default on a Euclidean and a spherical domain, and on a hyperbolic domain only `Disk` does | A model map or a view that puts a pixel in another tile, a convention that differs between the crates, a border that bends in the Klein model, and a model that changes another geometry |
| Intrinsic borders | "Wedge", a unit test of `renderer.rs`: the wall through the points that `from_disk` gives for the disk positions of the half-plane positions $i$ and $2i$, built with `hyperbolic::wall_through` and `hyperbolic::covector`, is the wall whose image in the half-plane is the imaginary axis. At each sample of the canonical half-plane view of a 192 by 192 image whose disk position lies in the range of accuracy, the intrinsic border test with this one wall and the width 0.2 marks the sample exactly when $\tan \lvert \theta \rvert < \sinh 0.1$, with $\theta$ the angle at 0 between the imaginary axis and the ray to the half-plane position of the sample, except where the two sides differ by less than a tolerance of the test. The condition rests on equation (7.20.3) of [BEA], and the band that it gives is the wedge of Fact 2 of [CFKP] (Section 6.2 of the rendering reference). "Same point in every model", a unit test of `renderer.rs`: the disk positions of the fold set of `*732` in radius bands 1 to 3, each mapped to the half-plane, the band, and the Klein model with the reference model maps and rounded to `f64`, give the same decision of the intrinsic border test with the width 0.05 through the view of each of the four models and the fold, except where the reference wall distance of a side is within $10^{-9}$ of the half width. That decision also equals the decision of the reference, which marks a position exactly when one of its wall distances at the reference fold is below the half width, so that a wrong wall distance that every model shares fails the test. "Intrinsic each pixel", one test for each picture: `*732` in each of the four models, `*632`, and `*532` with `Shading::Flat`, each at 48 by 32 and at 96 by 64 pixels, with one sample per pixel, the intrinsic width 0.1, the parity fill, and a background colour that no fill uses. A pixel whose centre is not an outside sample, lies at least $10^{-3}$ inside the unit circle or the rim in the reference, and has reference wall distances that all differ from the half width by at least $10^{-9}$ at the reference fold of its centre, shows the border colour exactly when one of those distances is below the half width, and otherwise shows one of the two fill colours. "Pixel unit unchanged": `BorderUnit::Pixels` given explicitly gives the bytes of the default options on every golden configuration with the pixel unit | A border that depends on the model or the image size, a wrong wall distance or unit, a border on the wrong wall, and a border unit that changes an image in pixels |
| Inclusion oracles | Every border pixel of a rotation domain is a border pixel of its reflection domain, and every border pixel of `4*2` or `3*3` is one of `*442` or `*632`. The fold of a point in a rotation domain lands, within the sum of the two bounds, where its fold in the reflection domain lands, or at the image of that point under the reflection in the doubled wall when the parity is odd | A border on the doubled wall, and a double that is not the union of two copies of its reflection domain |
| Invariance under the group | An image equals itself shifted by a translation of the group, with each product of pairings asserted to be a translation within the translation tolerance. The images of `xx`, `*x`, and `22x` equal a second render with the parity colours exchanged after a glide reflection. On the ball, rotating the view by an element of the group leaves the image unchanged, and a reversing element exchanges the parity colours. The symmetric view of a plan triple equals its mirror image. Every image of the centre under a breadth-first enumeration of the pairings folds back to the centre | A wrong view, a wrong pairing, and a wrong group |
| Exact equalities | `render` equals the concatenation of `render_rows` over any split into bands, with or without a coset fill. `Fill::Parity` on a domain without a reversing side equals `Fill::One`. The colouring `1a` equals `Fill::One` and the reversal parity colouring equals `Fill::Parity`, each found by its table. The shared pictures of Sections 5.14 and 5.15 are equal, and the coset golden images of the nine wallpaper groups differ pairwise, and so do the three spherical group golden images | A colour that depends on its band or on another sample, and a coset fill that changes the other fills |
| Statistics and termination | Every fold of each fold set and every sample of the canonical images converges within the default cap, with the largest fold of a finite group below its group order. Folds near vertices of high order stop within the cap, unconverged where Section 6 allows it. Beyond the range of accuracy, the rim set and the rows of a 4096 by 4096 image of `*732` that hold such a sample return one of the colours of Section 5.5. So does every sample of the canonical images of `*732` in the half-plane, the band, and the Klein model at 2048 by 64 and 64 by 2048 pixels. Of these, both images of the Klein model and the image of the band at 2048 by 64 pixels have outside samples, and the others have none. The canonical images of `*732` in the half-plane and the band at 192 by 192 and 256 by 128 pixels with 4 by 4 samples have no outside sample and no pixel of the background | A fold that does not terminate, a panic near the limit circle or the edge of a model, and the background inside the half-plane or the band |
| Golden images | Section 8.4 | A change of any pixel beyond the allowance |
| Perturbation | Moving every normal, covector, and pairing entry that the renderer copies, and for the ball the light and the viewing matrix, by one unit in the last place, with each mirror rebuilt from its moved normal or covector, changes each of seven golden images without a colouring within the allowance: `7-3-2`, `rot-7-3-2`, `6-3-2`, `wp-x-x`, `5-3-2-rotated`, `rot-5-3-2`, and `sg-3-s-2`, which cover the three geometries, mirror and other pairings, and a rotated view | An image that depends on the last bit of the platform mathematics library |
| Unit tests of private items | The position of each sample and the plane and ball views. The point of each model view at three pixels of square and non-square images, which equals the point of the model position that `MODEL_MAPS` gives, with $i$ at the centre of the half-plane view. The views of the half-plane and the band span 2 model units over the image height and have no outside sample at these sizes. The outside samples on and below the real axis, on and beyond the edges of the band, and on and outside the unit circle of the Klein model. The carrying of the screen tangents and the count of reversals by the private fold. The border decision for a NaN side value, a NaN gradient, and an infinite gradient with a finite side value. The intrinsic border decision for a NaN wall distance, which gives a border sample, and for a wall distance of exactly the half width, which does not. The orthogonality, the determinant, and the signs of the viewing matrix, where a positive yaw moves the front of the ball to the right, a positive pitch moves it up, and a positive roll turns the image counterclockwise. The exact values of the shading function, and the brightest pixel of the ball against the light direction | A wrong placement, a wrong sign, a wrong constant, and a lost tangent |
| Mask polygon | "Polygon rules", a unit test of `mask.rs`, on the reflection triangles of the plan triples, the Euclidean, spherical, rotation, polygon, wallpaper, and spherical group domain sets, without the domains of Section 5.22 that have no mask polygon: $P$ has as many vertices as the domain, counterclockwise, and is convex within a tolerance that the test sets, with a vertex of angle $\pi$ allowed. The two edges of each pair of sides have equal length within that tolerance. A Euclidean $P$ equals the plane positions of the vertices exactly, a $P$ of rule 3 the Klein positions of the vertices moved by $T$, on the reflection polygons and on their doubles, and a $P$ of rule 4 the union of $P_F$ and its reflection. A flat triangle has the angles of the table of Section 5.22. The centre $C$ of Section 5.23 lies strictly inside every side, the flat centre $C'$ lies strictly inside every edge of $P$, and for a spherical domain $C \cdot V_j > 0$ holds for every vertex. "No polygon", an integration test: the domains of `*22`, `*55`, `22`, `55`, `5x`, `x`, `*`, `1`, `*222`, `222`, and `2*` give `InvalidOption` with the text of Section 7.2 from `Masks::new` and from `Renderer::new` with an image fill, and `Renderer::new` accepts them with a solid paint. A second integration test gives every domain of "Polygon rules" a mask of 64 by 64 pixels with a black margin and a pixel that is not black, and an image fill that `Renderer::new` accepts | A rule applied to the wrong domain, a wrong vertex order, a double that is not symmetric, unequal edges of a pair of sides, and a chart that fails on a spherical domain |
| Mask map | Unit tests of `mask.rs` on the domains of "Polygon rules". "Round trip": for points of the domain drawn by a property test, and for the vertices and the centre, $M(x)$ lies in $P$ within a tolerance that the test sets, and the inverse of $M$, which the test computes in the reference arithmetic by walking the ray from $C'$ to the boundary of $P$ and back along the chart, returns $x$ within tolerances that the test sets, larger at the vertices than inside the polygon (Section 5.23). $M(C)$ equals $C'$ exactly, and the inverse returns $C$ within the tolerance of the points inside. "Fallbacks": points at a subnormal distance from $\hat{C}$ in the chart of `*2222`, `o`, `*22222`, and `*522`, each a point of its geometry other than the centre, get no point from $M$, through the fallbacks of step 3 where no sector holds the ray and where $e_j$ is not positive, and of step 4 where the hyperbolic or the spherical normalization fails. "Edge fraction": the points of each side at the fractions 0, 1/4, 1/2, 3/4, and 1 of its length map to the points of its edge at the same fractions, within a tolerance that the test sets. "Euclidean identity": on the Euclidean domains, $M(x)$ equals the plane position of $x$ within the rounding of the steps | A map that is not onto or not one to one, a wrong exit side, a fraction measured in the chart instead of along the side, a wrong centre, and a fallback that gives a point |
| Masks | Unit tests of `mask.rs` that call the private mask function with one sample per pixel, at 96 by 96 and 128 by 64 pixels. "Mask content": a pixel whose centre lies outside $P$ by more than one pixel is black, a pixel inside $P$ farther than the border width plus one pixel from every edge is white, and a pixel within the border of one edge, more than one pixel from its other boundaries, shows the code colour of that edge or, in the quarter of the edge at its marked end, black. "Seam classes": on the domains and every colouring of `ColouringSet::ValidClasses` up to a colour count that the test sets, and the colouring `1a`, every edge of a seam class has its code colour, every two edges that meet lie in one class, in a class with one mark on each edge every two edges that meet have their marked ends at the ends that the pairing joins, which the test checks by applying the pairing matrix to the vertices, a class has marks at both ends exactly when the carrying of Section 5.19 reaches one of its edges with both ends, and every class of a colouring whose subgroup is normal has at most two edges. A second unit test drives the private walk over the classes with hand-built meetings, in which the walk reaches an edge of one class with both ends, and asserts the class of each edge, marks at both ends on every edge of that class, and a mark at one end on the other edges. "Code colours": for every $N$ from 1 to 896, the $N$ code colours are pairwise different and none is white or black | A border on the wrong edge, two edges that meet with different colours, a mark at the wrong end, a mark missing at the second end, and code colours that coincide |
| Image fills | The input images are generated by the tests from the masks, so no input image is committed. "Flat image": an opaque image of one colour gives the image of the same coset fill with that solid paint, within the allowance of Section 8.4. "Texture contract", one test for each of a reflection, a rotation, a wallpaper, and a spherical group domain: at one sample per pixel and away from the walls, every second pixel, in a checkerboard pattern, of the canonical image of the domain with image fills, each image with an asymmetric mark painted inside the mask polygon, equals the bilinear value, in the reference, at the image position of $M(x)$ for the folded point $x$ of its centre, so the mark appears in each tile where the mask map puts it. "Mirrored": with the colouring `1a` on `*732`, `*632`, and `*532` with `Shading::Flat`, at 192 by 192 pixels, each pixel of the tile across side 0 of the domain equals the reference value of the image at the image position of $M(P_0 p)$ for its centre $p$, and the mark appears reflected in the wall of side 0. "Seams", a unit test of `mask.rs`, because the closure is private: on `*732` with `2a`, `*642` with `4a`, whose subgroup is not normal, the rotation domain `732`, `o`, `22x`, `4*2`, and `3*2`, each with a valid colouring and image fills of generated linear gradients: the pairs of colours that the closure of Section 5.19 computes for each side equal the pairs observed by folding, for many sampled tiles, a point just inside and a point just outside the middle of each side. For each pair of edges that meet and the fractions 1/4, 1/2, and 3/4 along them, the values of the image fills at a point just inside the side and at the folded point just outside it equal the image values at the same fraction of the two edges of the masks laid edge to edge as their marks say, within a tolerance that the test sets. "Same texture in every model", a unit test of `renderer.rs`: the disk positions of the fold set of `*732` in radius bands 1 to 3, mapped to the half-plane, the band, and the Klein model with the reference model maps, give image values through the view of each of the four models that agree within a tolerance that the test sets (Section 5.23). "Alpha and fallback": an image of one colour with alpha 128 gives the mean in linear light of that colour and the background. In a unit test of `renderer.rs`, the zero vector, points with a coordinate that is not finite, and points with a negative last coordinate give the background, the fallback of step 1 of Section 5.23, and the centre $C$ gives the image value at $C'$. The same test takes `hyperbolic::normalize_point` of $(0, -30, 31)$, a point of the hyperbolic plane far outside `*732`, asserts that the mask map gives it a point whose image position lies outside the image, and asserts that the renderer gives it the background, the fallback of step 3 of Section 5.20 | A texture that moves with the model or the size, a missing mirror, a wrong pair of neighbour colours, a seam that does not match the masks, an interpolation in encoded values, and a fallback that is not the background |
| Decoding | The test images are written with `png` in every colour type and bit depth of 8 or less, with and without tRNS, and interlaced. Each decodes to the expected texels. A truecolour image with 16 bits per channel gives `Decoding` with the text of Section 7.2 for 16 bits. Headers of 16385 by 1, 1 by 16385, 16384 by 16384, 16384 by 4097, 0 by 4, and 4 by 0, a stream without the PNG signature, a stream cut after the header, a bad checksum, and a reader that returns an error give the kinds and texts of Section 7.2 without a panic. A stream that holds only the signature and the header of a 16384 by 4097 image, with no image data, gives `LimitExceeded` with the text of the number of pixels, and neither `Decoding` nor the text of the memory limit of the decoder, which shows that the check comes before the decoder reads the image data. The code reads the width and the height from the header itself, or through a call of `png` that returns before it needs the image data, so a stream that ends after the header still gives the error of the number of pixels for an image over that limit. In images 1 pixel high, an Exif chunk just under the decoder bound and one of half the bound at the largest width decode, and one just over the bound gives `LimitExceeded` with the text of the memory limit. An image with a text chunk or a colour profile chunk just over the bound decodes. A property test changes one byte of a valid image to any value, and asserts that the result is an image within the pixel limit or an error of a documented kind and text, without a panic | A missing check, a check after the allocation, a wrong expansion of a colour type, a panic on a malformed stream, a wrong bound on the decoder, a text or colour profile chunk that the decoder stores, and a wrong kind for its memory limit |
| Wythoff constructions | The Wythoff triangle set is the reflection triangles of the plan triples, `*632`, `*442`, `*333`, `*332`, `*432`, `*532`, and `*n22` for $n$ of 2, 3, 5, 12, and 100, each with the eight markings. "Face rule", a unit test of `wythoff.rs`: the face types and their numbers of sides equal a table that the test writes from the table of faces at a corner of Section 10.2 of the colourings reference, every marking without a face type is refused, every other marking builds without the error `Numerical` of step 8, the corner orders of step 2 equal those of the signature, the images of $G$ under the $2 n_k$ elements of the group of the two mirrors through each corner with a face, taken in the reference, have as many distinct points as the face has sides, every $\lvert q_j \cdot V_k \rvert$ of step 7 exceeds a tolerance that the test sets, and each active side has a corner with a face. "Generator point": in the reference, $G$ lies on every inactive side, at equal `wall_distance` from every active side, and inside the triangle, and each foot $F_j$ lies on side $j$ between its ends and on $q_j$, within a tolerance that the test sets. "Face regions", a property test: a random point of the triangle that lies at least $10^{-9}$ from every perpendicular in the reference gets the face type $k$ of a corner whose sign tests of Section 5.25 all hold in the reference, and the point at a tenth of the way from $V_k$ to $G$ gets the face type $k$ for each corner with a face. "Faces around a centre": for each corner with a face, 64 points on a circle about $V_k$, with a radius of half the least distance from $V_k$ to $G$ and to the feet of the sides through it, fold to points that take the face type $k$, so each face covers its centre from every side of its mirrors. "Face oracle": on whole images of the triangle set at 193 by 193 pixels with one sample per pixel, a border width of 0, and a colour for each face type that no other colour of the image uses, on the pixels whose centre lies within the radius of the layer "Border oracles", or every pixel in the plane, every change of colour between neighbouring pixels lies within one pixel of an image of a segment from $G$ to a foot in the reference, and every pixel farther than one pixel from such a segment shows the colour of the face that the oracle assigns to its centre. The oracle takes $G$ and the feet from the reference, rounded to `f64`, measures distances in `f64` with the functions of `escher-geometry`, and assigns to a folded centre the face of the corner that the segment from the centre to that corner reaches without crossing a segment from $G$ to a foot. "Known faces": in a render, the pixel at the point at a tenth of the way from $V_k$ to $G$ shows the colour of face type $k$. Here and in "Face regions", that point is $0.9 V_k + 0.1 G$, which lies on the segment from $V_k$ to $G$ but in the hyperbolic and spherical planes not in general at a tenth of its length. "Wythoff borders": the whole-image border oracle of the layer "Border oracles", on the pixels that it checks, with the segments from $G$ to the feet as the edges, in both units, with the width of 1 pixel and the intrinsic widths 0.06, 0.1, and 0.03 for the hyperbolic, Euclidean, and spherical triangles. In the intrinsic unit, a change of face needs a border pixel next to it only where the intrinsic width times the pixel scale exceeds 1, because a border narrower than a pixel can fall between pixel centres. No border pixel, including one on a mirror that is not an edge, lies farther than half the border width from every edge, within a tolerance on the distance that the test sets. The distance is measured in pixels at the pixel scale in the pixel unit, and in the geometry in the intrinsic unit | A wrong face rule or generator point, a face assigned to the wrong corner, a face that splits along a mirror, and a border on a mirror or across a face |
| Whole faces | The whole-face set is the regular constructions `*632` with `010` and `100`, `*442` with `010`, `*332` with `010`, `*432` and `*532` with `010` and `100`, and `*732`, `*832`, `*542`, `*552`, and `*642`, each with `010`, and `*732`, `*542`, and `*642` with `100`. Each test takes the arrangements of a construction from `free_classes_of_index` of `escher-group` with the presentation, stabilizer, and index of its `WholeFace`, so the tests cover what the command line lists. "Regular scope", a unit test of `whole_face.rs`: on the Wythoff triangle set with each of its eight markings, `WholeFace::new` accepts exactly `010` and `100` on the triangles `*pq2` with $p \ge q \ge 3$ and refuses every other marking with the text of Section 7.2. The canonical form of a signature orders the corners of a triangle by decreasing order, so $p \ge q$ holds for every domain, and the test asserts that three other spellings of each triangle give the same orders. "Stabilizer", a unit test: in the reference arithmetic, on the whole-face set, the $2n$ matrices of the words of step 6 of Section 5.29 are pairwise different by more than a tolerance that the test sets, each fixes vertex $c$, the images of the centre of the domain under them are $2n$ points at equal distance from vertex $c$, and the $2n$ flat matrices $A_w$ of the reference are pairwise different by more than that tolerance. Each flat matrix of the crate equals its reference matrix within a tolerance that the test sets. The same test takes the reference face as the points on the side of vertex $c$ of each of the $n$ walls of the reflections $r_i$ of step 7 of Section 5.29, and checks on sample points, as many as the test sets, that the image of every sample point of the domain under each of the $2n$ matrices lies in the reference face within a tolerance that the test sets, and that every sample point of the reference face farther than a tolerance that the test sets from every image of a side of the domain lies in exactly one of the $2n$ images of the domain, by the side test of each image side. "Face triangle map", a unit test of `mask.rs`: "Round trip" and "Edge fraction" of the layer "Mask map" run on the domains of the whole-face set with the face triangle $F'$ of Section 5.31 in place of the mask polygon of Section 5.22, with the inverse of the reference arithmetic taken with $F'$, and with tolerances that the test sets. "Edge pairing", a unit test: for every arrangement of the whole-face set, every coset has one stabilizer element, the edge pairing is an involution, both edges of each pair carry the same flag, and for every arrangement with no mirrored neighbour, $(\sigma \rho)^{n_g}$ and $(\rho \sigma)^{n_g}$ are the identity, with $\rho$ the permutation $i \mapsto i + 1$ of the edges and $n_g$ the order of corner $g$, which is the number of faces at a vertex of the tiling and the exponent of the condition of Lemma 0.2 of [YUNCKEN2003]. "Pairing geometry", a unit test: in the reference arithmetic, for each edge $i$ of each arrangement of the whole-face set, the matrix $k$ of the word $r_i w^{-1}$, with $w$ the stabilizer element of step 2 of Section 5.30, maps vertex $c$ to the reflection of vertex $c$ in the wall of edge $i$, maps the midpoint of edge $\sigma(i)$ of the reference face to the midpoint of edge $i$, each midpoint taken as the point midway between vertex $c$ and its reflection in the wall of the edge, and has a determinant whose sign is $-1$ for a mirrored neighbour and $+1$ for a turned one, and the trace of $r_i w^{-1}$ from coset 0 is coset 0. "Whole picture", an integration test: the input image is generated from the reference placement of $Q$, `ExactPlacement`, as a gradient with three discs of different colours, which no symmetry of $Q$ other than the identity maps onto themselves. For each arrangement of `*632` with `010` and with `100`, of `*442` with `010`, of `*732` with `010`, and of `*532` with `010` with `Shading::Flat`, at 192 by 192 pixels with one sample per pixel and a border width of 0, every pixel whose centre lies within radius 0.9 of the centre of the disk or the ball, or anywhere in the plane, and farther than a tolerance that the test sets from every edge of its face, equals in each channel the 8-bit rounding of the bilinear value, in the reference, at the picture point that an oracle gives. A pixel whose bilinear value lies within a band that the test sets of a rounding boundary may differ by one step, and fewer than a fraction that the test sets of the compared pixels may differ. Each arrangement has at least as many compared pixels as the test sets. The oracle reaches the faces of the image by a depth-first walk from the reference face across its edges, with the matrices $k$ of "Pairing geometry" composed in the reference arithmetic, takes the face whose centre is nearest to the point of the pixel, maps the point back into the reference face by the inverse of the matrix of that face, finds the element of the stabilizer that carries it into the domain, and gives the point of $Q$ of the reference face mask map of $F'$ moved by its flat matrix. The walk asserts that every face that it reaches by two paths gets a matrix that carries three points to the same images within a tolerance that the test sets, so every face shows the picture once. The test also asserts handedness from the rendered pixels alone, by the centroids of the pixels of each disc in a face, where every disc shows at least a number of pixels that the test sets. On the reference face the three discs have the orientation that they have in $Q$ when the default view keeps the orientation of the domain, and the opposite one when it reverses it, which the test computes from the vertices of the domain and the screen map. The face across each edge, which the test finds in the reference arithmetic, shows the opposite orientation to the reference face exactly when the `EdgeMeeting` of that edge is mirrored, and each arrangement checks at least one neighbour of each kind, turned and mirrored, that it has. "Wedge continuity", an integration test: the input image is generated in $Q$ as a smooth gradient that no symmetry of $Q$ other than the identity maps onto itself. For the first arrangement of `*732` with `010`, of `*632` with `010`, and of `*532` with `010` with `Shading::Flat`, at 192 by 192 pixels with one sample per pixel and a border width of 0, for every pair of neighbouring pixels whose centres the oracle of "Whole picture" puts in the same face but in images of the domain under two different elements of the stabilizer, within a radius that the test sets in the disk and on the ball, or anywhere in the plane, and farther than a tolerance that the test sets from every edge of the tiling, the values of the two pixels differ by at most a bound that the test sets. The test asserts that each of the three images has at least as many such pairs as a number that the test sets, so that a render with too few pixel pairs across the lines between the wedges fails. "Upright picture", an integration test: the input image is generated over the face mask with two features, each in a colour that no other part of the picture uses, placed in image terms by the image position of the centre of $Q$ under the mask placement, which the test takes from `ExactPlacement` of the reference: the first above that position, at a smaller row in the same column, and the second to its right, at a larger column in the same row. For the first arrangement of `*732` with `010`, of `*632` with `010`, and of `*532` with `010` with `Shading::Flat`, rendered in the default view of Section 5.31 at 192 by 192 pixels with one sample per pixel and a border width of 0, the test takes, for each feature, the centroid of the pixels inside the reference face, as "Stabilizer" defines it, whose colour lies within a tolerance that the test sets of the colour of that feature, and asserts that each feature has at least a number of such pixels that the test sets. The direction from the image position of vertex $c$, which the test computes in the reference arithmetic through the default view, to the centroid of the first feature lies within an angle that the test sets of straight up, toward row 0, and the direction to the centroid of the second lies within that angle of straight right, toward the last column. The test measures these directions from the rendered pixels alone, and reads neither the angle $\psi$ nor the face mask map of the crate. For `*632`, in the plane, it also asserts that each direction lies within a smaller angle that the test sets of straight up or straight right, so the picture shows no turn within that angle. `WholeFace::new` and `WholeFace::render_mask` take no view option (Section 3.1), so the face mask is the same byte for byte under every view option, and no test varies one for the face mask. "Face seams", a unit test of `whole_face.rs`: for each arrangement of the constructions of "Whole picture", with an image fill of a generated linear gradient, for each edge $i$ and the fractions 1/4, 1/2, and 3/4 along it, the value at a point just inside the reference face equals the image value at that fraction of edge $i$ of $Q$, and the value at the folded point just outside it equals the image value at the same fraction of edge $\sigma(i)$ counted from its end for a turned neighbour and from its start for a mirrored one, within a tolerance that the test sets. The image values come from the reference placement of $Q$, `ExactPlacement`, and the shared bilinear sampling `bilinear` of `golden_images`. The test checks the marks on the pixels of the face mask that `render_mask` draws: at sample fractions along each edge, inside its border, the quarter at each end that the rule of Section 5.32 marks for its `EdgeMeeting` is black, and every other sample has the colour of the middle of the edge, which is neither black nor white. It asserts that its edges include each kind: a mirrored pair, a turned pair, an edge that meets itself by a mirrored neighbour, and one that meets itself by a turned neighbour. "Face mask content", a unit test of `mask.rs`, on every arrangement of the whole-face set: as "Mask content" for $Q$, with one sample per pixel, and every edge of a class has its code colour, the two edges of a pair share one class, the classes are numbered in the order of their least edge, an edge paired with itself has a class of its own, an edge paired with itself by a turned neighbour has a mark at both ends, and one paired with itself by a mirrored neighbour has one mark, at its start. "Same picture in every model", a unit test of `renderer.rs`: as "Same texture in every model" with the whole-face fill of an arrangement of `*732` with `010` | A face rule or scope that admits a construction that is not a regular tiling, a wrong stabilizer word, a reversed or inverted trace, a pairing read from the wrong coset, a wrong flag, a picture that does not continue across an edge, a picture split at a line between its wedges, which "Wedge continuity" catches, and a picture doubled inside a face, which "Whole picture" catches, a face that shows the picture twice or not at all, a face mask turned against the default view, which "Upright picture" catches, and a mark at the wrong end |
| Failure paths | The rows of Section 8.2, every option at its limits and one past them with the kinds and texts of Section 7.2, every error text, the conditions of `CosetFill::new` and of `Masks::new` after the side presentation, each in its order, because no domain that `build_with` returns lacks a side presentation, the colour and the size of `Masks::render`, the bit depth of `Image::decode_png`, the refusals of `Wythoff::new` in their order, the number of colours of `WythoffFill::new`, the refusals of `WholeFace::new` and the checks of Section 5.30 in their order, the number of paints of `WholeFaceFill::new`, the size of `WholeFace::render_mask`, and the four refusals of `Renderer::new` | A missing check, a wrong order of checks, and a changed text |

The unit test of the error text takes `4*3` as its signature without a domain.

The border oracles and the border-subset tests run the Euclidean domains at 193 by 193 pixels, a choice of this design, because at 192 by 192 pixels the vertical mirrors of `*632` fall exactly on boundaries between pixels, and rounding then decides which neighbouring pixel is a border pixel. The spherical border oracle renders its border mask with `Shading::Flat`, a choice of this design, because Phong shading turns a black border grey near the highlight. The test of the brightest point compares the centroid of the tied brightest pixels with the predicted pixel, on a black background, a choice of this design, because the 8-bit values form a plateau at the highlight on which several pixels tie. The symmetric view uses `Fill::One` and an even image size, because a mirror of the group swaps the two parity colours and an odd size puts a row of samples on the axis. The wall of side 0 after the isometry of the symmetric view is not exactly the first axis in `f64`, so the test asserts the golden allowance. The symmetric view reads vertex 2 of a triangle, so no test uses it for a reflection polygon.

The symmetric view maps a point $x$ of the screen to $Q x$ before the fold, and the tangents to $Q T_1$ and $Q T_2$. $Q$ is the isometry that maps `hyperbolic::ORIGIN` to vertex 2 of the domain and the positive first axis to the geodesic ray from vertex 2 through vertex 0, built from `hyperbolic::translation`, `hyperbolic::inverse`, and `hyperbolic::rotation`. The wall of side 0 then lies on the first axis. The periodic view differs from the canonical view of Section 5.8 only in its plane scale, so that a period that is not a whole number of pixels in the canonical view, such as that of `*2222` with `aspect` 1.618, becomes one.

The exact comparisons rest on IEEE 754 arithmetic: the periodicity and glide-symmetry cases of Sections 6.6 and 6.10, the bands, the scale property, the equalities of fills, and the shared pictures compare bytes or bits that the same operations give on the same inputs, or that no rounding changes. Every other comparison of images asserts the golden allowance, except the rotation of the view of a spherical group domain by an element of its group, which compares byte for byte. These tests compare a value that passes through a transcendental function exactly with a stored value, and check an assumption that no cited source establishes:

- The exact assertions of the ball view and the viewing matrix at the angles 0, which assume that `f64::sin_cos` of 0 gives exactly 0 and 1 in the platform's library.
- The tests of the colour, which compare `to_srgb8(from_srgb8(v))` against `v` for every byte and `from_srgb8(255, 255, 255)` against `Colour::WHITE` after `powf`, which assumes that `powf` of exactly 1 gives 1 in the platform's library.
- The test of the coset colour of each sample, which compares the bytes of each converged pixel with the literal bytes of its coset colour, after a decoding and an encoding through `powf`.

The property tests commit their regression files (TEST-9 of the coding standard). They run with the default configuration of `proptest`, except "Round trip", which runs 512 cases.

The deliberate defects of Section 8.4 were applied by hand to show which test catches each. They are not part of the test suite.

RND-21. Withdrawn on 2026-11-19.

### 8.2. Deliberately Wrong Inputs

| Input | Expected |
| --- | --- |
| A point with a NaN, positive infinite, or negative infinite value in each coordinate in turn | `converged()` false, zero steps |
| The zero vector, $(0.5, 0.25, 0)$, $(0.5, 0.25, -0)$, the spacelike vector $(1, 0, 0)$, and `-hyperbolic::ORIGIN` on the other sheet of the hyperboloid, each with a last coordinate that is not positive | `converged()` false, zero steps |
| The spacelike vector $(3, -2, 0.5)$, with a positive last coordinate | No panic. Unconverged, with at most the default cap of steps |
| A cap of 0 with a point outside the domain | Unconverged, zero steps |
| A cap one below the steps that a point needs | Unconverged, `steps()` equal to the cap |
| `(f64::MAX, 0, f64::MAX)` | Unconverged, because the point becomes infinite during the fold |
| A vertex of the domain | Converged with zero steps, because $\tau > 0$ (Section 6.2) |
| For `*532`, the zero vector, and a NaN, positive infinite, or negative infinite value in each coordinate in turn | `converged()` false, zero steps |
| For `*532`, $(0, 0, -1)$, the antipode of the centre | Converged, with at most the default cap of steps |
| For `*532`, a cap of 0 with a point outside the domain, and a cap one below the steps that a point needs | Unconverged, with zero steps and with `steps()` equal to the cap |
| A buffer one byte short of a row, an empty buffer, and a band past the last row | `InvalidBuffer` with the text of Section 7.2 |
| `encode_png` with a size of 0 or a buffer of the wrong length | `InvalidBuffer` |
| A size of 0 by 512, a grid of 17, a width of 16385, a NaN border width, a cap of 100001, a NaN rotation angle, and a zero light direction | The kinds and texts of Section 7.2 |
| `Masks::render` with the colour `count()` and a size of 0 by 512 | The kinds and texts of Section 7.2 |
| `Masks::new` on the lune of `*22` | `InvalidOption` with the text `the domain has no mask polygon` |
| `Image::decode_png` with an empty reader, a text file, a PNG cut in its image data, and a header of 16384 by 8192 pixels | `Decoding`, `Decoding`, `Decoding`, and `LimitExceeded` with the text of the number of pixels, with the texts of Section 7.2 |
| An image fill at a point that is the zero vector, has a coordinate that is not finite, or has a negative last coordinate, and at the centre $C$ itself | No panic. The background for the first three, and the image value at $C'$ for the centre |
| `Wythoff::new` with the domains of `732`, `*2222`, `*22`, and `*3333`, and with every mark off on `*732` | `InvalidOption` with the texts `the domain is not a reflection triangle` and `the marks give no face` |
| `Wythoff::new` on `*522` with only mark 0 on, and on `*222` with each single mark | `InvalidOption` with the text `the marks give no face` |
| `WythoffFill::new` with 2 colours for a construction with 3 face types, and `Renderer::new` with the Wythoff fill of `*732` on the domain of `*742`, on that of `*3333`, which has 4 sides, and on a second build of the domain of `*732` | The kinds and texts of Section 7.2, and no error for the second build of `*732` |
| `WholeFace::new` with the constructions of `*732` with `111`, `001`, and `110`, of `*522` with `010`, of `*333` with `100`, and with the construction of `*732` with `010` and the domain of `*742` | `InvalidOption` with the texts `the construction is not a regular tiling` and `the Wythoff construction was built for another domain` |
| `WholeFace::edges`, `WholeFace::render_mask` with a size of 0 by 0, and `WholeFaceFill::new` with no paint, for the regular construction of `*632` with `010` and each of the colouring `1a` of `*3333`, which has 4 generators, an arrangement of `*642` with `010`, one of whose relators moves a coset, the colouring `1a` of `*632`, and a colouring of index 12 that is not an arrangement | `InvalidOption` with the whole text of Section 7.2 of each check, the same for the three calls, so each check comes before the size and the paints |
| `WholeFaceFill::new` with 0 and 2 paints, and `Renderer::new` with the whole-face fill of `*732` with `010` and the domain of `*742` | The kinds and texts of Section 7.2 |

### 8.3. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture:

- Structure. The module `reference`, in `tests/render/reference.rs`, which the integration-test binary and the unit tests load (Section 8.1), holds points, vectors, and matrices as arrays of high-precision numbers and writes each formula componentwise from the research documents: the map from the disk to the hyperboloid, the side test, the reflection, the fold of Section 5.3 with the point size of each geometry, the distance form of Section 5.4, the wall distances of Section 5.17 in each geometry by the formulas of Section 5.6 of the geometry design, the lift onto the ball of Section 5.9, the shading of Section 5.10, the sRGB transfer function, and, as `ExactMask` with its `MaskRule`, the mask polygon of Section 5.22, the mask map of Section 5.23 with the same exit side of step 3, its inverse, the point at a fraction of the length of a side, and the image position under the mask placement of Section 5.19, as `ExactPlacement`. For the whole-face fills it adds the direction of the face, `face_direction`, the unit vector at the angle $\psi$, computed from the vertices of the domain by the table of Section 5.31, with $(0, -1)$ when $\mu$ is zero, and for a hyperbolic domain through the recentring `recentre` that the rule of the Klein model also uses. It turns the flat face polygon $Q$ of Section 5.31 by that direction with `flat_turn`, and adds the face triangle $F'$ of Section 5.31, the mask map onto $F'$ as the rule `MaskRule::FaceTriangle` of `ExactMask`, the flat matrices with their product, determinant, and reflections, and `ExactFace`, which builds the $2n$ elements of the stabilizer of Section 5.29, each as a matrix and a flat matrix, from the pairings of the domain. The shading of the reference holds the specular weight $k_s$ as a value of its own, apart from the ambient weight $k_a$, although both are 0.25, so that it follows the formula of Section 5.10 term by term. It calls no function of `escher-render`, `escher-domain`, `escher-geometry`, or `glam`. It takes the shared core, the complex arithmetic, and the reference maps of the half-plane, the band, and the Klein model and their inverses from the test support crate `escher-reference` (Section 7.1 of the architecture), and adds the constructions of this crate and a dispatch of those maps by model. The tests normalize both points before they take the hyperbolic distance.
- Inputs. The reference takes the exact `f64` values of the inputs, and of the pairings and covectors of the domain, which it reads through the public accessors of `Side`. Its fold also takes an exact vector of the reference type, so that the tests of the intrinsic border can fold a point that no `f64` vector holds, such as the reference map of a pixel centre to the hyperboloid.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 12.2 of the domain design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there.

RND-19. The reference module MUST share no code with `escher-render`, `escher-domain`, or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture, TEST-13 of the coding standard).

### 8.4. Golden Images

The golden images are committed PNG files under `crates/escher-render/tests/render/golden/`, as Section 7.2 of the architecture and TEST-14 of the coding standard state. Each renders in the canonical view of its geometry at 192 by 192 pixels, with a 4 by 4 grid, a border width of 1 pixel unless its configuration names the intrinsic unit, the default border colour, background, and cap, and, for the ball, the default rotation, light, and shading unless its configuration states otherwise. An image without a colouring uses `Fill::Parity` with the sRGB colours $(245, 222, 179)$ for an even parity and $(70, 130, 180)$ for an odd parity. A coset golden image uses `Fill::Cosets` with the first $k$ of eight distinct sRGB colours, a choice of this design: the two parity colours followed by $(220, 20, 60)$, $(46, 139, 87)$, $(255, 215, 0)$, $(106, 90, 205)$, $(255, 140, 0)$, and $(64, 224, 208)$. The golden configuration of each image holds its settings, its rotation, its shading, its hyperbolic model, its colouring, its border unit, and its border width.

The golden images cover each family of domains, and each image adds coverage that no other golden image gives:

- Hyperbolic, Euclidean, and spherical reflection domains, including a ball with a viewing rotation and a ball with `Shading::Flat`.
- Rotation domains of each geometry, which show the first fill colour only (RND-33) and no border on a doubled wall (RND-34).
- Reflection polygons, including one with four different orders, which fixes the cyclic order of the corners, one at a setting that differs visibly from the default, and the double of `3333`.
- Wallpaper domains with the parity fill, one for each parity picture that differs from every other golden image: `o`, `**`, `xx`, `2*22`, `4*2`, `3*3`, and the sheared `2222`. `22*`, `*x`, and `22x` have none, because their parity pictures equal those of `**` and `*2222` (Section 5.14).
- The spherical group domain of `3*2` with the parity fill.
- The reflection triangle `*732` with the parity fill in each of the half-plane, the band, and the Klein model, as `7-3-2-half-plane.png`, `7-3-2-band.png`, and `7-3-2-klein.png`, which fix the canonical view of each model (`DEC-76`), the convention of each model map (GEO-30 of the geometry design), including the orientation of the half-plane picture (`DEC-75`), and the border width through the model Jacobian. The disk keeps its golden images unchanged (RND-50).
- The reflection triangles `*732` in the disk, `*632`, and `*532` with `BorderUnit::Intrinsic` and the width 0.05, as `7-3-2-intrinsic.png`, `6-3-2-intrinsic.png`, and `5-3-2-intrinsic.png`, which fix the wall distance and the intrinsic unit of each geometry (Section 5.17). Every other golden image keeps its bytes (RND-55).
- Coset golden images: a colouring whose subgroup is not normal on a reflection domain, the largest colour count, chiral colourings on spherical rotation domains, a Euclidean colouring, a shaded colouring on the ball, a colouring of `*22222` that a renumbering of its sides changes, so that the image fixes the numbering of the sides that the domain design makes part of its contract (DOM-37), one colouring for each of the nine wallpaper groups that tells it from the related groups, and colourings of `3*2` and `2*5` that no related group gives. `n*` and `nx` have no golden image, because no picture of them is known to differ from that of a related group, and `*`, `x`, and `1` have none, because their default pictures are one tile or a plain ball (Section 5.15).
- Polygon masks: `mask-6-4-2-4a-1.png`, the mask of colour 1 of `4a` of `*642`, whose subgroup is not normal, which fixes the flat triangle, the mask placement, the seam classes with their code colours and marks, and the antialiasing of the fixed sample grid (Sections 5.19 and 5.22).
- Image fills: one image in each geometry, `img-6-4-2-4a.png`, the colouring `4a` of `*642` with an image fill for each colour, `img-6-3-2-2a.png`, the colouring `2a` of `*632` with an image fill for colour 0 and a solid paint for colour 1, and `img-5-3-2-5a.png`, the colouring `5a` of `*532` with an image fill for each colour and the default `Shading::Phong`, each with input images that the test generates from the masks with an asymmetric mark, which fix the mask map, the mirrored tiles, the bilinear sampling, and the shading of an image fill (Sections 5.20 and 5.23).
- Wythoff fills, with the first colours of the coset golden images: `wy-7-3-2-010.png`, `*732` with `010`, the regular tiling `{7,3}` that the owner asked for, `wy-6-3-2-111.png`, `*632` with `111`, and `wy-5-3-2-110.png`, `*532` with `110` on the ball with the default shading, which fix the face rule, the generator point, the face of a sample, the borders on the edges, and the colours of the face types (Sections 5.24 to 5.27).
- Whole-face fills, each with an input image that the test generates from the face mask of its arrangement with an asymmetric mark, as for the image fills: `wf-7-3-2-010-14a.png`, the first arrangement of $\{7,3\}$, `wf-6-3-2-010-12a.png`, the first arrangement of $\{6,3\}$, and `wf-5-3-2-010-10a.png`, the first arrangement of $\{5,3\}$ on the ball with the default shading, which fix the stabilizer elements, the face mask map, and the picture of each face (Section 5.31). The face mask `fmask-7-3-2-010-14a.png` of the first arrangement of $\{7,3\}$ fixes $Q$, its placement, the classes, and the marks (Section 5.32).

A change that adds a golden image follows these rules:

- The image adds coverage that no other golden image gives, and the configuration of `golden_images` holds it with its settings.
- The file name writes the canonical signature with its characters joined by hyphens, followed by the setting or the colouring id. A reflection domain writes no star and takes no prefix. Any other domain writes `*` as `s` and takes the prefix `rot-` for a rotation domain, `wp-` for a wallpaper domain, or `sg-` for a spherical group domain, and a coset golden image takes `cos-` before that prefix. The sheared `2222` keeps the prefix `rot-` of the existing rotation images of `2222`. An image in a hyperbolic model other than the disk ends with the value of `--model` of the command-line design, such as `-band`. An image with the intrinsic unit ends with `-intrinsic`, after the model where it has one. A polygon mask takes the prefix `mask-` before every other prefix and ends with the colouring id and the colour, such as `mask-6-4-2-4a-1.png`, and an image with image fills takes the prefix `img-` in the same place, such as `img-6-4-2-4a.png`. An image of a Wythoff fill takes the prefix `wy-` and ends with its marks as three digits, such as `wy-7-3-2-010.png`. An image of a whole-face fill takes the prefix `wf-` and a face mask the prefix `fmask-`, and each ends with its marks and the id of its arrangement, such as `wf-7-3-2-010-14a.png`. A change of the ids of `free_classes_of_index` renames and regenerates these images in the same change, as for the coset golden images.
- A coset golden image shows a valid colouring and names it by its id under `ColouringSet::ValidClasses` of GRP-37 of the group design, because that is the id that a user passes to the command line, which lists only the colourings of that set. A golden image whose colouring is not valid is replaced by a valid colouring that gives the same coverage, such as a valid colouring of `*22222` that a renumbering of its sides changes, because `*22222` has no valid colouring with 3 colours, and the replaced file is removed.
- A coset golden image of a wallpaper group takes the first id of `ColouringSet::ValidClasses`, by colour count and then in id order, whose generator columns differ for every two sides with different pairing matrices, so that a renumbering of the sides changes the image, as far as the group allows, because some sides of `22*` and `2*22` carry one half-turn. An image chosen for another reason, such as the colouring of `*22222`, states that reason.
- When the colouring of a coset golden image of any other domain must change, the image takes the colouring that `ColouringSet::ValidClasses` keeps for the class of its former colouring, where that class has a valid colouring. Otherwise it takes the first colouring of the set with the same colour count, or else a colouring of the nearest colour count that keeps the purpose of the image, such as `4a` of `*642`, whose subgroup is not normal, and `8ekf` of `*22222`, the first whose colours across the five sides differ pairwise.
- A change of the ids under `ColouringSet::ValidClasses` renames each coset golden image whose id changes and regenerates it through `just update-golden` in the same change.
- A deliberate defect that the image is meant to catch changes it by far more than the allowance.

RND-20. The golden-image test MUST decode the committed file with `png`, convert both images to linear light through a table of the 256 byte values that the reference transfer function of Section 8.3 gives, and pass only when at most the allowed number of pixels differ and each differing pixel differs in each channel by at most one sample weight plus the linear-light size of one 8-bit step at the larger of the two stored values of that channel, because samples are averaged in linear light and stored as 8-bit sRGB, so a sample that changes colour moves the stored value by an amount that depends on the colours, a limit in stored units would be too loose for dark values and too tight for bright ones, and an integration test cannot read the linear value of a `Colour`.

The allowed number of pixels is 16 for each image, a choice of this design and part of the contract of the golden-image test. The reasons for it:

- The perturbation of Section 8.1, which is the size of a difference in the platform mathematics library, changes no pixel of the seven golden images without a colouring that it runs on, on the measured platform.
- A rotation of the view of $10^{-6}$ changed a handful of pixels, by at most one sample each, so 16 pixels admit a difference between platforms of that size.
- Every deliberate defect tried that changes an image changes hundreds or thousands of pixels of it, so 16 pixels hide none of them.

An allowance of 0 pixels, which 8 times the measured count gives, would fail on the first platform whose mathematics library moves one sample across a border. The count bound of RND-16 does not apply here, because the allowance covers a difference between platforms that no measurement on one platform can show.

The deliberate defects that were applied by hand, each alone with the crate restored afterwards, include a missing parity flip, tangents not carried through a step, a wrong sample offset, a wrong constant or exponent of the transfer function, a wrong plane scale, swapped axes of the plane position, tangents not turned by $V^{\mathsf{T}}$, the opposite sign of the pitch, a wrong form or order of the shading, every step counted as a reversal, a glide reflection or a rotary reflection not counted as a reversal, a trace in fold order, a trace without the inversion, a trace from coset 1, a side buffer not cleared, and sides renumbered in the side buffer. Each changed a golden image beyond the allowance or failed another test. A defect that a golden image cannot show, such as a side test in reverse order, $\tau = 0$, a comparison of RND-9 without its negation, or an obsolete threshold of the transfer function, fails a test of another layer of Section 8.1.

The golden-image test reads the environment variable `ESCHER_UPDATE_GOLDEN`. With exactly the value `1`, it writes each rendered image over its committed file and does not compare. With any other value, or unset, it compares. A committed file that cannot be read or decoded, or whose width, height, colour type, bit depth, or buffer length differs from the rendered image, fails that image with a message that names the recipe `update-golden`, and such a file never reaches the comparison or the encoder. Section 14.9 of the tooling design gives the procedure.

When an image differs beyond the allowance, the test writes `actual-<name>.png`, the rendered image, and `difference-<name>.png`, the absolute difference of each byte, under the directory that `CARGO_TARGET_TMPDIR` names, where `<name>` is the name of the golden file without `.png`. It checks every golden image before it fails, prints the number of differing pixels of each image that it compares, and reports every image that failed.

## 9. Dependencies

RND-22. The crate MUST declare only the dependencies of this section, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml`, because each further dependency adds build time, licence obligations, and supply-chain risk, and a version declared twice can diverge (DEP-2 of the coding standard).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-domain` | Normal, with a version on the path dependency (TOOL-25 of `docs/design/tooling.md`) | None | The input type `FundamentalDomain` and its sides |
| `escher-geometry` | Normal, with a version on the path dependency (TOOL-25) | None | The disk map, the maps of the other hyperbolic models to the disk with their Jacobians, and `spherical::boost` and `spherical::rotation` for the viewing matrix |
| `glam` | Normal, public | The workspace entry | `DVec3` in the signatures of `fold` and `light` (`DEC-4`) |
| `thiserror` | Normal | Default | Derives the `Display` of the private detail of the error (Section 7.1) |
| `png` | Normal | The workspace entry, version 0.18.1, which enables no feature, because `png` has no default feature | PNG encoding with the sRGB chunk (Section 5.6), the decoding of an image fill (Section 5.21), and decoding in the tests |
| `escher-signature` | Normal, with a version on the path dependency (TOOL-25) | None | The type `Geometry`, on which `Renderer::new` chooses the view (RND-23), and the signatures from which the tests build their domains |
| `escher-group` | Normal, public, with a version on the path dependency (TOOL-25) | None | The type `Colouring` in the signature of `CosetFill::new`, its coset table, and `colour_of` (Section 5.12), and `Presentation` and `Word` in the signatures of `WholeFace` with `CosetTable::trace` (Sections 5.29 to 5.31) |
| `proptest` | Dev | The workspace entry | The property tests |
| `astro-float-num` | Dev | The workspace entry | The reference arithmetic (Section 8.3) |
| `escher-reference` | Dev, with a version on the path dependency (TOOL-25) | None | The shared core of the reference arithmetic (Section 8.3) |

The newest resolution of `png` brings two versions of `miniz_oxide`, which fails the ban on duplicates of `cargo deny`. This design holds `flate2` at version 1.1.9 in the lock file, as Section 8 of `docs/design/tooling.md` states, so that one version of `miniz_oxide` resolves. With that pin, `cargo tree -d` finds no duplicate and `check-deny` passes with no change to `deny.toml`. A plain `cargo update` resolves `flate2` to a newer version and brings the duplicate back. Section 8 of the tooling design gives the command that restores the pin. Section 12.3 keeps the alternative, a `skip` entry in `deny.toml`, as a point for the owner.

`num-complex` enters the build through `escher-geometry` only (Section 11 of the geometry design), and this crate does not declare it, because no type of it crosses the public surface of `escher-geometry`.

`palette` and `csscolorparser` are not dependencies of this crate, because `DEC-29` adds no colour crate in the first image milestone. The crate implements the sRGB transfer function itself (RND-11), and colour text belongs to the command line (RND-5). Section 3.1 of the architecture lists `escher-geometry`, `escher-domain`, `escher-signature`, and `escher-group` as internal dependencies of the crate. `escher-signature` is a normal dependency, not a dev-dependency, because the `match` of RND-23 names `Geometry`, a choice of this design.

RND-40. The crate MUST declare `escher-group` as a normal dependency from its workspace entry, and MUST take the colouring, its colour count, its generators, and its coset colours from `escher-group` only, because `CosetFill::new` names `Colouring` in its signature, and a second copy of the coset table or of the trace in this crate could diverge from the one that the tests of `escher-group` check. `escher-group` depends only on `escher-signature` and `thiserror` (Section 12 of the group design), so it adds no external crate, no licence, and no duplicate version to the build.

`escher-group` is a public dependency in the sense of DEP-3 of the coding standard, because `Colouring` appears in a public signature, and `escher-domain` is one too, through `FundamentalDomain`.

`glam` types appear in the public signatures of this crate, so DEP-3 of the coding standard makes `glam` a public dependency, with the obligations that Section 3.5 of the architecture states. No type of `png` appears in a public signature (Section 3.5 of the architecture).

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

## 10. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, or a statement of this design that no source establishes, the section of this document that relies on it, and what the code does about it:

| Label | What the code relies on | Relied on in | What the code does |
| --- | --- | --- | --- |
| `UNV-1` | The fold terminates for the reflection domains of every geometry, including the lune of order 1 of `*`, for the samples of the images that the tests render and the images of the centre. The step count grows toward the limit circle for a hyperbolic domain and with the distance from the centre for a Euclidean domain. Near a vertex of very high order on the sphere, and near some vertices of the rotation domain of `(100)(100)(100)`, it can fail to stop | 5.3, 5.20, 5.25, 6.4, 6.6, 6.7, 6.8 | Stops at the fold cap and reports the sample as unconverged. The tests measure the step counts and that every fold of their sets converges within the cap, and allow unconverged results near those vertices |
| `UNV-2` | The fold terminates for pairings that are not reflections: the rotations and translations of a rotation domain, the translations, half-turns, rotations, and glide reflections of a wallpaper domain, and the rotations and rotary reflections of a spherical group domain other than `*` | 5.11, 5.14, 5.15, 5.20, 6.8, 6.10, 6.11 | Folds by the one loop of RND-31 and stops at the fold cap. The domain design requires accurately rounded products as pairings. RND-35, RND-45, and RND-48 limit the range of accuracy, and the tests measure the step counts and the unconverged folds. No argument exists for the kites or the spherical group domains |
| `UNV-3` | The behaviour of a point on or near a wall | 5.3, 5.25, 6.2 | Uses the tolerance constant of Section 6.2. The tests cover the vertices of the domains, and the pairing tests keep their points at a clearance from the walls that grows with the error of the fold |
| `UNV-4` | The growth of the fold count and the limits of precision near the limit circle | 6.4 | States accuracy only in the range of accuracy (RND-15). The tests measure each band, and beyond the range assert only a result without a panic |
| `UNV-6` | The construction of a reflection polygon from SymmHub's cuts, through the domain that `escher-domain` builds | 5.13 | Renders the walls and pairings that the domain gives. `build_with` returns only a polygon or a double that passes `verify()` (DOM-44 of the domain design), and the border oracle and the exact permutation check the tiling that results |
| `UNV-7` | The default shape of a reflection polygon, through the domain that `escher-domain` builds | 5.13 | As for `UNV-6` |
| `UNV-11` | That each wallpaper domain and each spherical group domain gives the group of its signature, through the domain that `escher-domain` builds | 5.14, 5.15 | Renders the walls and pairings that the domain gives. The guards of DOM-54 and DOM-60 and the certificates of DOM-57 and DOM-64 of the domain design check each group, and the tests of Section 8.1 check the tiling that results. No image of this crate can tell `n*` from `n22` or `nx` from `*nn` (Section 5.15) |
| `UNV-13` | The map from the band to the disk, through `hyperbolic::disk_from_band` of `escher-geometry`. The choice of the half-plane map is the owner's decision `DEC-75`, not a fact | 5.16 | Draws the band through that function only (RND-51). Section 12 of the geometry design states its tests, and the golden image of the band and the "Each pixel" test check the image that results |
| `UNV-14` | The unit-normal form $\sinh d = \lvert \langle x, u \rangle \rvert$ on the hyperboloid and the form $\sin d = \lvert x \cdot u \rvert$ on the sphere, through `hyperbolic::wall_distance` and `spherical::wall_distance` of `escher-geometry`, and the unit-normal form in the distance form of the tests | 5.4, 5.17, 5.26 | Decides the intrinsic border through those functions only (RND-54). Section 12 of the geometry design states how their tests pin the forms, and the test "Wedge" compares the hyperbolic border with equation (7.20.3) of [BEA]. The tests also compare the gradient form with the distance form |
| `UNV-15` | The gradient form of the border estimate through a chain of folds in the linear model, in every hyperbolic model, for the walls of the sides and for the perpendiculars of a Wythoff construction | 5.4, 5.16, 5.26 | Tests it against the distance form, against tile identity, against the grid of the rectangle, and on the ball against a brute-force oracle, and on the ball states its accuracy by rim band (Section 6.7). The comparison with the distance form tells the two forms apart only in bands 1 and 2 (Section 6.4) |
| `UNV-16` | The lift onto the ball and the Phong reflection model, whose primary paper is unread | 5.9, 5.10 | Lifts as Section 5.9 states and shades as Section 5.10 states. The tests check the tangents, the brightest point, and the shading against the reference |
| `UNV-17` | That the colouring by the cosets of a subgroup of finite index is perfect for the groups of the built domains beyond Frettloh's triangle tilings, that the side sequence of a fold gives the element of the tile, and that the trace of `colour_of` gives the coset of that element. The seam classes of the polygon masks rest on the same reading for the colour of the neighbour of the domain across side $j$, `colour_of(&[j])`, and the picture of a sample of a whole-face fill on the same reading of the fold product | 5.12, 5.19, 5.31 | Colours by `colour_of` of `escher-group` only (RND-37). The exact-permutation test asserts that each side moves the coset colour exactly as the inverse column of its generator does, and the coset golden images fix the result. "Seams" checks the seam classes of the masks against the render |
| `UNV-18` | The parity colouring as the colouring by the two cosets of the orientation-preserving subgroup, with glide reflections and rotary reflections among the reversing pairings | 5.5, 5.14, 5.15 | Colours by the parity of the reversals. The pairing, glide-symmetry, and equality tests of Section 8.1 check the parity, and the golden images fix the result |
| `UNV-19` | The steps of the low-index search of `escher-group` that the colourings come from, in the tests only | 5.12 | Section 13 of the group design states how `escher-group` relies on it. The tests take each colouring by its id, and a wrong search fails the exact permutation or changes a coset golden image |
| `UNV-35` | That no two tiles that share a side have the same colour in a colouring exactly when no side generator fixes coset 0 of its table, in the tests and the choice of the coset golden images only | 8.1, 8.4 | Draws any colouring that it is given, valid or not. "Valid neighbours" compares the test of GRP-42 of the group design with the colours of sampled neighbouring tiles |
| `UNV-37` | That the regular tiling $\{p, q\}$ of a hyperbolic `*pq2` has the corner of order $q$ at a vertex, on which the markings `010` and `100` as $\{p, q\}$ and its dual rest | 5.24 | Draws the faces that the marks give, whatever tiling they name. "Face rule" checks the number of sides of each face, and the golden image `wy-7-3-2-010.png` fixes the result |
| `UNV-38` | The face rule at a corner of a Euclidean or hyperbolic reflection triangle, with digons dropped, the edges from the generator point to its images in the active mirrors, and the markings `010` and `100` as $\{p, q\}$ and its dual on a hyperbolic `*pq2` | 5.24 | Draws the faces that the face rule gives and no other. "Face rule" checks the number of distinct images of $G$ around each corner against the number of sides, "Faces around a centre" checks each face around its corner, "Face oracle" checks the faces on whole images, and the golden images fix the result |
| `UNV-39` | The perpendiculars from the generator point to the active mirrors cut the triangle into one region for each corner with a face, and a folded point belongs to the face of its region, in every geometry | 5.25, 5.26, 5.27 | Assigns the face by the steps of Section 5.25 only (RND-66). "Face regions", "Faces around a centre", "Face oracle", and "Known faces" check it |
| `UNV-40` | The generator point of a Euclidean or hyperbolic construction is the point on the inactive mirrors at equal distance from the active ones, and that point exists and is unique | 5.24 | Computes the point by step 5 of Section 5.24 only (RND-65). "Generator point" checks the distances in the reference |
| `UNV-41`, `UNV-42`, `UNV-43` | That a colouring in whose table no element of the stabilizer other than the identity fixes a coset maps no face onto itself other than by the identity, and that an index of $2n$ gives one orbit on the faces, so one picture covers every face | 5.30 | Takes the arrangements that `free_classes_of_index` of `escher-group` finds with that filter, and refuses a colouring whose stabilizer elements do not reach every coset once (check 3 of Section 5.30). "Whole picture" checks that every face shows the picture once |
| `UNV-44` | The stabilizer of the reference face of a regular construction is the group of order $2n$ of the reflections in the centre mirror and the vertex mirror, with the $2n$ words of step 6 of Section 5.29, and the face is the union of the images of the domain under it | 5.29, 5.30, 5.31 | Lists the stabilizer by those words only (RND-71). "Stabilizer" checks in the reference arithmetic that the $2n$ elements are different, fix vertex $c$, and carry the domain into the face, and that their images of the domain cover the face once on sample points, and "Whole picture" checks the faces on whole images. No test checks that no other element of the group fixes the face, or that the images cover the face between the sample points |
| `UNV-45` | The edge pairing that the steps of Section 5.30 read from the coset table, with its flags, is the pairing of the faces of the tiling, and it is an involution | 5.30, 5.32 | Computes the pairing by those steps only (RND-72). "Edge pairing" checks the involution and Yuncken's condition, and "Pairing geometry" checks each meeting in the reference arithmetic |
| `UNV-46` | A sample whose fold applies the sides of the word $g$ shows the point $w_a x$ of the reference face, with $a$ the forward trace of $g$ from coset 0 | 5.31 | Computes the picture point by the steps of Section 5.31 only (RND-73). "Whole picture" compares every pixel away from the edges with an oracle that walks the faces by the edge pairing, and "Face seams" checks the seams |
| `UNV-25` | The correct rounding of the reference crate, in tests only | 8.3 | Uses 256 bits and the self-check of `escher-reference` (Section 7.1 of the architecture) |
| `UNV-26` | The Euclidean reflection as a matrix, and the angle of a lune measured from its normals, through the domain that `escher-domain` builds | 5.8, 5.9 | Renders the walls and pairings that the domain gives. The domain design states which checks and tests cover them, and the periodicity tests, the border oracles, and the images of the centre check the tiling that results |
| `UNV-27` | The ranges of the shape parameters of a reflection polygon, through the domain that `escher-domain` builds | 5.13 | As for `UNV-6` |
| `UNV-28` | The vertex coordinates of the Euclidean triangle, through the domain that `escher-domain` builds | 5.8 | As for `UNV-26` |
| `UNV-29` | The lune as a strict fundamental domain of the group of its two reflections, through the domain that `escher-domain` builds | 5.9 | As for `UNV-26`, with the rotation of the view by a group element |
| `UNV-30` | The lune double is a fundamental domain of the cyclic group, through the domain that `escher-domain` builds | 5.11 | Renders the walls and pairings that the domain gives. The double oracle and the border-subset test check the tiling that results |
| `UNV-31` | The double of a convex reflection polygon is convex with the stated vertex cycles, through the domain that `escher-domain` builds | 5.11, 5.13 | Tests the border estimate over the whole wall of each side, which is correct for a convex domain (Section 5.11). The double oracle and the border-subset test check the tiling that results |
| `UNV-33` | The pairings and the sheared domains of the wallpaper groups that no source states, through the domain that `escher-domain` builds | 5.14 | As for `UNV-11` |
| `UNV-34` | The domains and pairings of the spherical group domains that no source states, through the domain that `escher-domain` builds | 5.15 | As for `UNV-11` |
| This design | The bounds of Section 6 and the allowance of Section 8.4 hold on platforms other than Linux x86-64, where they were measured | 6, 8.4 | Sets each bound with a margin of at least 8, except the tolerance constant at the vertices of the rotation domains and of the doubles of the reflection polygons (Section 6.2), and the step bounds of the wallpaper and spherical group domains (Sections 6.10 and 6.11). Sets the allowance of 16 pixels for a difference between platforms. CI runs the tests on Linux x86-64 only |
| This design | The screen tangents of Section 5.2 are the partial derivatives of the map of Section 7.2 of the geometry design, times $h$ | 5.2 | A reference test compares them with central differences |
| This design | The screen tangents of Section 5.16 are the partial derivatives of the map from the screen through the model and the disk to the hyperboloid, times $h$, with the model Jacobians of Sections 7.5 to 7.7 of the geometry design | 5.16 | A reference test compares them with central differences of the reference map in each model (Section 6.12), and the border oracles check the borders in each model |
| This design | The wall of a side in the Klein model is the chord on which $c_i \cdot (s_1, s_2, 1) = 0$, from the map of [RAT], equation (6.1.1), the map from the Klein model to the hyperboloid (Section 4.2 of the geometry reference), which sends a Klein position $s$ to a positive multiple of $(s_1, s_2, 1)$, in the tests only | 8.1 | The test "Straight borders" uses it. The code of the crate does not use it |
| This design | The screen tangents of Section 5.9 are the partial derivatives of the lift onto the ball, times $h$, turned by $V^{\mathsf{T}}$ | 5.9 | A reference test compares them with central differences of the reference lift |
| This design | For a Euclidean domain, the side value at a point with last coordinate 1 is the distance from the point to the wall, through `euclidean::wall_distance` of `escher-geometry`, and in the Euclidean distance form of the tests | 5.8, 5.17, 8.1 | Decides the intrinsic border of a Euclidean domain through that function. Section 12 of the geometry design states how its tests pin the statement, and the Euclidean distance form uses it in the border oracles |
| This design | The mask polygon of each rule of Section 5.22 is convex with its vertices counterclockwise, and the two edges of each pair of sides have equal length | 5.22 | Builds the polygon by the rules only (RND-63). "Polygon rules" checks it on every domain set, including the equal edges of the doubles of rule 3 |
| This design | The centre $C$ of Section 5.23 lies strictly inside every side, and for a spherical domain every vertex has $C \cdot V_j > 0$, so the chart holds the whole domain | 5.23 | Gives the fallback for a point with $C \cdot x \le 0$, or with $x_3 \le 0$ for the other geometries. "Polygon rules" checks the vertices of every spherical domain set |
| This design | For every rule of Section 5.22, $C'$ lies strictly inside $P$, and each ray from $\hat{C}$ in the chart and from $C'$ in $P$ meets the boundary once | 5.23 | Computes the mask map by the steps of Section 5.23 only (RND-64). "Polygon rules" checks $C'$, and "Round trip" checks the rays |
| This design | The closure of Section 5.19 gives exactly the pairs (colour of a tile, colour of its neighbour across side $j$) that occur in the tiling, and every seam class of a colouring whose subgroup is normal has at most two edges | 5.19 | Computes the seam classes by the closure only (RND-58). "Seams" compares the closure with sampled neighbours, on colourings that include `4a` of `*642`, whose subgroup is not normal, and `732`, a rotation domain with side pairings. "Seam classes" checks the classes of the normal colourings |
| This design | In a seam class with one mark on each edge, every two edges that meet have their marked ends at the ends that the pairing joins, and a class has marks at both ends exactly when the carrying of Section 5.19 reaches one of its edges with both ends | 5.19 | Carries the marked ends as Section 5.19 states. "Seam classes" checks it with the pairing matrices |
| This design | The mask map of Section 5.23 is a continuous bijection from the domain onto the mask polygon that sends each side onto its edge, proportionally to the distance along the side | 5.19, 5.20, 5.23 | Computes it by the steps of Section 5.23 only (RND-64). "Round trip", "Edge fraction", "Euclidean identity", and "Seams" check it |
| This design | The side whose sector of the chart holds the ray, by step 3 of Section 5.23, is the side through which the ray from $\hat{C}$ leaves the chart image of the domain, also where two sides lie on one wall | 5.23 | Takes the exit side by the sector only, and the reference arithmetic takes it the same way. "Round trip", "Edge fraction", "Euclidean identity", and "Seams" check it, "Euclidean identity" on `2222`, `22*`, and `2*22` among its domains |
| This design | The $N$ code colours of Section 5.19 are pairwise different, and none is white or black, for every $N$ up to 896 | 5.19 | "Code colours" checks every $N$ in that range |
| This design | Step 5 of Section 5.24 gives a point of the triangle on the inactive sides at equal distance from the active ones, through `wall_meet` and the covector $c_a - c_b$ | 5.24 | Computes the point by that step only (RND-65). "Generator point" checks it in the reference |
| This design | The corner order is the integer nearest to $\pi$ over the corner angle, the calls of steps 5 and 6 of Section 5.24 succeed, no corner with a face lies on the perpendicular of an active side through it, and each active side has a corner with a face | 5.24, 5.26 | Returns `Numerical` for a failed call and for a corner on a perpendicular (step 8 of Section 5.24). "Face rule" checks each of them on the Wythoff triangle set |
| This design | The conditions of Section 5.29 on one face type select exactly `010` and `100` on `*pq2` with $p \ge q \ge 3$, and the $2n$ flat matrices of the stabilizer words are pairwise different | 5.29 | Refuses every other construction (RND-71). "Regular scope" checks the first on the Wythoff triangle set, and "Stabilizer" the second on the whole-face set |
| This design | The map from a face of a whole-face fill onto $Q$, the face mask map of Section 5.31 on each copy of the domain in the face followed by the flat matrix of the stabilizer element of that copy, is one to one from the face onto $Q$ and continuous across the lines between the $2n$ wedges of the face | 5.31 | Computes the picture point by the steps of Section 5.31 only (RND-73). "Face triangle map" checks the face mask map on $F'$ at sample points, and "Wedge continuity" checks, in three renders of a smooth picture, that neighbouring pixels across the lines between the wedges differ by at most a bound that the test sets |
| This design | The vector $\mu$ of Section 5.31 is, in each geometry, the image in the default view of the tangent at vertex $c$ of the segment toward vertex 2, and with $Q$ turned by its angle $\psi$, a picture painted on the face mask appears on the reference face in the default view the same way up as on the face mask: in the plane as a scaled copy with no turn, and in the disk and on the ball with the direction from the centre of the face toward the midpoint of edge 0 that the face mask shows, at the centre of the face only | 5.31, 5.32 | Computes $\psi$ by the table of Section 5.31 only, reading no render option (RND-71, RND-74). "Upright picture" checks, on renders of `*732`, `*632`, and `*532` with `010`, the directions of two features placed away from the centre of the face, each within an angle that the test sets, and a smaller angle in the plane |
| This design | Copies of the face mask laid edge to edge with their marks together show each seam of a whole-face fill, so a picture that agrees along paired edges of $Q$ gives a render that is continuous across every edge of the tiling | 5.31, 5.32 | Draws the face mask and the picture through one face mask map and one edge pairing (RND-72 to RND-74). "Face seams" checks the values on both sides of three points of each edge against the face mask |
| This design | The mirrors of `*2222` are the lines of the grid of its rectangle, in the tests only | 8.1 | The rectangle oracle uses it. The Euclidean border oracle checks the borders of the rectangle against tile identity without it |
| Rendering reference | The constants of the sRGB transfer function, which rest on secondary sources until the standard is read (items 22 and 25 of Section 13.1 of the rendering reference) | 5.6 | Takes them from Section 7.5 of the rendering reference only (RND-11). The tests compare the crate with the reference transfer function |
| Rendering reference | Rounding to the nearest integer, after clamping to the range 0 to 1, when encoding an 8-bit sample (item 23 of Section 13.1 of the rendering reference) | 5.6 | Rounds as Section 5.6 states. The tests check the round trip of every byte |

Averaging in linear light is a choice of the architecture that no source supports (item 24 of Section 13.1 of the rendering reference). It states no fact, so it is a choice rather than an unverified statement. The same holds for the point size of the sphere, the viewing matrix, the shading constants, the form of the specular term, the rules of the mask polygon beyond the owner's decisions, the mask map inside the polygon, the code colours and the marks, the bilinear sampling of an image fill, its alpha over the background, and its fallback.

## 11. Evidence Behind the Choices

Withdrawn on 2026-11-22. The number stays so that the citations of Sections 12 to 15 hold.

## 12. Assumptions, Choices, and Points for the Owner

### 12.1. Assumptions

These readings rest on no sentence of a source. A change that breaks one of them breaks the behaviour that names it:

- A pairing is linear on the vectors of the linear model, and so is $V^{\mathsf{T}}$, so applying it to the screen tangents gives the screen tangents of the moved point. The border estimate relies on it instead of an accumulated matrix (Sections 5.2, 5.9, and 6.1), and the border oracles check the result.
- For `*2222` and the rectangular groups with `aspect` 1 or 4, a power-of-two image size, and a grid of 1, 2, 4, 8, or 16, no operation of the construction, the view, the fold, or the border estimate rounds (Sections 6.6 and 6.10). The exact periodicity and glide-symmetry cases rely on it, so a change of $S$ or of the placement of these domains breaks them.
- The side presentation of each built domain presents its group, so its colourings are those of the group of the tiling. `UNV-32` covers the cycle relators. The certificate and the equal counts of DOM-57 and the evaluation of the relators of the domain design check it.
- The generators of two sides that share a wall are equal in the group and the sides have the same orientation, so the coset colour and the parity of a sample do not depend on which of them the fold applies first (Sections 5.14 and 5.15). The fold therefore needs no rule for a point outside two sides on one wall.
- A double is the union of its generating domain and its mirror image in the wall of side 1 (DOM-23 of the domain design), so the corner angles of the generating domain at the two vertices on that wall are half those of the double, and its third corner of a triangle keeps its angle. Rule 4 of Section 5.22 reads the angles of $F$ this way, and "Polygon rules" checks the flat triangles that result.
- A spherical group domain of `3*2`, `2*n`, or `n*` has the vertices and the sides of the kite on which it is built, with other pairings (Section 6.16 of the domain design), so rule 4 of Section 5.22 applies to it as to the kite.
- What `png` allocates before the crate reads the header is bounded only by the `png::Limits` that the crate sets. From a reading of `png` 0.18.1, the crate assumes that `png` charges against that bound the buffer of the chunk that it reads, which it doubles as the chunk grows, one output row of at most 65536 bytes, and a palette with its transparency table of at most 1024 bytes. It also assumes that `png` does not charge the copy of an Exif chunk that it keeps, so an Exif chunk can take up to twice its length, and that `png` stores no text or colour profile chunk that it is told to ignore. The Exif lengths that Section 5.21 accepts and refuses rest on this assumption, as does an Exif chunk of up to half the bound in an image more than 1 pixel high, and a version of `png` that charges otherwise can change them. The size check of RND-62 bounds the texels whatever `png` does.
- The fold is deterministic, so its step counts stay as they are while the code of the fold and the domains does not change. The step bounds of Sections 6.10 and 6.11 rely on it with small margins, so a change that moves one fold decision can break them, and widening them weakens the check.

### 12.2. Choices of This Design

No other document dictates these choices, and each looks like something a change could simplify away. The numbered rules state the other choices of this design with their reasons. The section of each row states its reason:

| Choice | Section |
| --- | --- |
| No public `View`, the hyperbolic model as an option of `RenderOptions` of the type `HyperbolicModel`, and a public `fold` with a visitor for the side sequence | 3.1, 3.4 |
| The other models through the disk, with the disk path unchanged bit for bit and the derivative of `from_disk` kept in this crate (`DEC-74`) | 5.16 |
| `RenderOptions::builder` takes the fill, so the library has no default fill colour | 3.1, 4.2 |
| `Fill` without `Copy`, and `CosetFill::new` takes the `Colouring` by value, because the fill stores it | 3.1, 5.12 |
| One tolerance constant of $4 \times 10^{-15}$, one fold cap, and one golden allowance for every geometry and every kind of domain, because another value changes committed golden images | 6.2, 8.4 |
| The point size of the sphere as the largest absolute coordinate, selected by a private enum, so that the hyperbolic and Euclidean decisions stay the same bit for bit | 5.3 |
| No gAMA or cHRM chunk beside the sRGB chunk, against the recommendation of the PNG specification | 5.6 |
| No border at a tie of exactly the half width, so `*2222` with `aspect` 1 or 4 at one sample per pixel shows no border | 5.8 |
| The specular term as $k_s \rho (1 - c)$, so that no channel needs a clamp | 5.10 |
| Shading of fill, border, and unconverged samples alike, before the samples are averaged | 5.10 |
| The options of the ball checked for every domain and read only for a spherical one | 4.1 |
| No bound from the group order in the code, so the fold cap is the only bound | 6.7 |
| A side buffer and a trace after the fold, in place of a permutation of the cosets carried through it | 5.12 |
| A side index that does not fit a byte and a side sequence without a coset colour are internal errors stated with `expect`, in place of a truncated index or a fallback colour | 5.12 |
| `escher-signature` as a normal dependency, because `Renderer::new` names `Geometry` | 9 |
| A bound of 8 times the largest value, except a cell measured as exactly 0, which takes the largest bound at the same distance, and the step bounds, which take no factor 8 | 6.6, 6.10, 6.11 |
| The Euclidean border oracles at 193 by 193 pixels, and the spherical border mask with `Shading::Flat` | 8.1 |
| The border unit as an option of its own beside the width, and not an enum that carries the width, so that the builder and the check of the width stay as they are | 3.1, 4.1 |
| The fold carries the screen tangents with the intrinsic unit too, so that it keeps one path, although the intrinsic border test does not read them | 5.17 |
| The intrinsic border test over the walls of the sides, as RND-9, with no fade near the rim or the limit circle, so a border thinner than a sample breaks into separate samples | 5.17 |
| A mask drawn from the mask polygon alone, with no fold and no side test, so a mask relies on no termination of the fold | 5.19 |
| A mask as RGB through `encode_png`, so the crate keeps one encoder | 5.19 |
| Image fills as paints of a coset fill only, so `Fill::One`, `Fill::Parity`, and `Fill::Wythoff` take no image (`DEC-91`, `DEC-68`) | 3.1, 5.20 |
| The mask polygon and the mask map built by `Renderer::new` from its own domain, and kept private (`DEC-88`) | 3.4, 5.20 |
| Bilinear sampling with no level of detail, alpha over the background, and the background as the fallback (`DEC-89`) | 5.20 |
| Every PNG colour type at a bit depth of 8 or less, read as sRGB, and an image with 16 bits per channel refused (`DEC-90`) | 5.21 |
| A fixed 4 by 4 sample grid for every mask, so `Masks::render` takes no grid | 5.19 |
| A border of 1/64 of the shorter side, code colours at evenly spaced hues, the margin of the disk view, and a black quarter of the edge at each marked end as the mark, which draw the owner's decision that the edges that can meet share one code colour and a mark shows how they meet (`DEC-98`) | 5.19 |
| The four rules of the mask polygon, with the domain itself for a Euclidean domain and the Klein positions in a chart centred at $C$ for a reflection polygon and its double (`DEC-95`) | 5.22 |
| The centre of the mask map as the normalized vertex sum, not the point that `centre()` gives, the exit side by the sector of the chart that holds the ray, not the least parameter, and the fraction measured in the chart inside the polygon and along the side on the boundary (`DEC-97`) | 5.23 |
| An image of any size read through the mask placement at its own size | 5.20 |
| The Wythoff fill as a variant of `Fill`, with `Wythoff` as a type of its own so that the command line counts the face types before it has the colours | 3.1 |
| The generator point from `wall_meet` and differences of covectors, in place of the linear solve of pywonderland (`DEC-101`) | 5.24 |
| The corner orders read back from `corner_angles()`, so that `escher-domain` keeps its public surface | 5.24 |
| The face of a sample by sign tests on the perpendiculars, the first matching corner, and the last corner with a face when none matches, in place of the nearest perpendicular of the hyperbolic shader (`DEC-103`) | 5.25 |
| The borders of a Wythoff fill on the whole walls of the perpendiculars of the sides through the corner of the face, in place of the distance to the segments (`DEC-104`) | 5.26 |
| `WholeFace` as a type of its own that exposes the side presentation, the stabilizer words, and the index, so that the command line runs the search, and a list of paints in `WholeFaceFill::new` (`DEC-115`) | 3.1 |
| The face mask map as the mask map of the domain onto the face triangle $F'$, followed by the flat matrix of the stabilizer element of the coset, in place of a radial map of the whole face, whose chart is not symmetric about the centre of the face (`DEC-116`) | 5.31 |
| $Q$ with the edges counterclockwise from edge 0, the edge that holds side $e$ (`DEC-117`), turned by the angle $\psi$ of the default view of its geometry whatever the view options, so that the face mask does not change with `model` or `rotation` (`DEC-122`) | 5.31 |
| The forward trace of the side sequence, with the stabilizer element of each coset computed once by `WholeFaceFill::new`, so a sample reads one trace and one table entry more than a Wythoff fill | 5.31 |

### 12.3. Points for the Owner

Each point is open, rests on a proposed default rather than a decision of the owner, and states the default in use. None blocks the crate. The points keep their numbers, because other documents cite them by number:

Point 1. Renormalization (`DEC-31`, Section 6.1). The fold renormalizes nothing and keeps no accumulated matrix. The alternative is to renormalize, which changes decisions near the limit circle.

Point 2. Unconverged samples (`DEC-7`, Section 5.5). An unconverged sample takes the border colour. The alternative is a colour of its own, such as the background.

Point 7. The `flate2` pin of Section 9. The pin in the lock file is lost on a plain `cargo update`. The alternative is a `skip` entry for `miniz_oxide` 0.8 in `deny.toml`, which is durable but is an exception to the policy of Section 8 of `docs/design/tooling.md`.

Point 13. The Euclidean view (`DEC-12`, Section 5.8). The shorter side spans 8 units with no margin. The alternatives are a span that shows a fixed number of tiles, or a window that depends on the size of the domain.

Point 14. The ball view (`DEC-12`, Section 5.9). The ball shows the front hemisphere with the size and the margin of the disk. The alternatives are a view of the back hemisphere, or a margin of its own for the ball.

Point 18. No border at one sample per pixel for `*2222` with `aspect` 1 or 4 (Section 5.8). The design accepts it as a property of point sampling. The alternative is a span $S$ that keeps the walls off the pixel centres, which loses the exact periodicity of these cases.

Point 24. The viewing rotation (`DEC-16`, Section 5.9). The yaw, the pitch, and the roll turn the ball about fixed axes of the screen. The alternative is a rotation, a tilt, and a second rotation, which need one matrix fewer but make the first and the third angle do the same thing when the tilt is 0.

Point 25. The light (`DEC-16`, Sections 4.1 and 5.10). One white light fixed to the viewer, from the default direction $(-1, 1, 2)$. The alternatives are a light fixed to the ball, or another default direction.

Point 26. The material (`DEC-16`, Section 5.10). The constants $k_a = 0.25$, $k_d = 0.75$, $k_s = 0.25$, and $\alpha = 32$ have no option. The alternative is an option for each constant.

Point 27. The specular term (`DEC-16`, Section 5.10). The form $k_s \rho (1 - c)$ never clips a channel. The alternatives are the clamped form of the Phong model, which clips channels of the default fill colours, or constants with $k_a + k_d + k_s \le 1$, which dim every colour.

Point 28. Shaded borders (`DEC-16`, Section 5.10). Borders and unconverged samples are shaded like fills, before the samples of a pixel are averaged, so a black border shows a grey highlight. The alternatives are unshaded borders, or shading the averaged pixel.

Point 29. The options of the ball on another geometry (Section 4.1). They are checked and have no effect. The alternative is an error.

Point 30. The dark outline at the rim (Section 5.9). The design accepts it. The alternative is a fade or a cutoff of the border near the rim.

Point 31. Borders at large $n$ (Section 5.9). Around a vertex of a large order, the ball shows only border colour at the default size. The alternative is a warning when every sample is a border sample.

Point 33. The range of accuracy of the border estimate on the ball (Section 6.7). It is the rim bands in which the count bound holds, which by construction is every band. The alternative is a largest share of differing samples, such as 1 percent, which would end the range a few pixels from the rim.

Point 34. Small vectors on the sphere (Section 6.1). `fold` states its accuracy only for a vector whose largest absolute coordinate is at least $10^{-300}$, and folds a smaller vector without rescaling it, so its decisions can differ from those of the same ray at length 1. The alternative is to multiply the point and the screen tangents once by $2^{1000}$ before the loop when the point size is below a threshold such as $2^{-900}$. The multiplication is exact, but `Fold::point` of such an input then returns the rescaled vector.

Point 35. The margin of $\tau$ at the vertices of the rotation domains and of the doubles of the reflection polygons (Section 6.2). The design keeps $\tau = 4 \times 10^{-15}$, although the margin there is below the factor 8, because a larger value changes the golden images of the reflection domains (RND-32). The alternatives are a larger $\tau$ with new golden images, or image vertices and covectors that the domain design rounds more accurately.

Point 37. The parity colouring of a rotation domain (`DEC-32`, Section 5.11). `Fill::Parity` gives the first colour to every tile with no error, while the command line never draws `Fill::Parity` on such a domain, and draws `--colouring parity` there as the coset fill of its valid colouring with 2 colours, or refuses it (CLI-35 of the command-line design). The alternative is an error kind of the library.

Point 40. Mirror sides drawn like paired sides (`DEC-55`, Section 5.14). Every side draws the same border, so several groups share their default picture and differ only by their colourings by id. The alternative draws mirror sides with another width or colour, which gives each group a picture of its own but changes the image of every reflection domain and rotation domain, or adds an option.

Point 41. The range of accuracy of the wallpaper domains (Section 6.10). The design states the bounds at the settings of the wallpaper domain set only, within distance 65. The step bounds have small margins, because the factor 8 would exceed the default cap of 1500. The alternatives are a bound that grows with the elongation of the domain, which no measurement supports, and step bounds with a wider margin, which give a weaker check.

Point 42. The default view of `*`, `x`, and `1` (`DEC-61`, Section 5.15). The canonical view shows one tile for `*` and `x` and a plain ball for `1`, and `--rotation` turns the ball. The alternative is a tilted default view for these three signatures, which gives three domains a view rule of their own.

Point 43. Shared pictures of the spherical groups (`DEC-61`, Section 5.15). The design accepts that no picture of the proof of concept is known to tell `n*` from `n22` or `nx` from `*nn`, as `DEC-55` does for the wallpaper groups. The alternative of point 40 would also give `n*` and `nx` pictures of their own.

Point 44. The step bounds of the spherical group domains (Section 6.11). Each is one step above the largest count that a measurement outside the repository found, so a change that moves one fold decision can break it. The alternatives are the cap itself as the bound, which checks only convergence, or a wider margin, which gives a weaker check.

Point 45. Withdrawn on 2026-11-23, when the owner decided that a mask shows the domain as a flat polygon (`DEC-93`), so no mask has a view.

Point 46. The fallback (`DEC-89`, Section 5.20). A point that the mask map cannot carry, or whose image position lies outside the image, takes the background colour. The alternatives are the nearest edge texel, or the solid colour that the colour would have without the image.

Point 47. The alpha of an image fill (`DEC-89`, Section 5.20). Alpha is composited over the background colour of the render. The alternative composites over the default colour of the coset, so that an unpainted part of the mask keeps the colour of that coset.

Point 51. Marks at both ends (`DEC-98`, Section 5.19). A seam class whose meetings reach one of its edges with both ends shows a mark at both ends of every edge, which asks for artwork along those edges that matches itself reversed. The test "Seam classes" prints the number of such classes among the colourings that it covers. The alternatives are to refuse such a colouring in `Masks::new`, or a mark of another form that tells the two directions apart.

Point 52. Withdrawn on 2026-11-23, when the owner dropped image fills from the Wythoff fill (`DEC-68`).

Point 53. Withdrawn on 2026-11-23, when the owner dropped the face masks with the image fills of the Wythoff fill (`DEC-68`).

Point 54. The face mask map (`DEC-116`, Section 5.31). The domain goes onto the face triangle $F'$ by the mask map of Section 5.23, and the picture of the other triangles of a face is its flat mirror image or turn, so the picture inside a face is distorted in $2n$ symmetric wedges, and more so where the corner of $F'$ at the vertex of the tiling, $\pi/2 - \pi/n$, differs from the angle $\pi/n_g$ of the domain, where $n_g$ is the order of corner $g$. The alternative is a radial map of the whole face in a chart centred at the centre of the face, which needs a new chart and a new inverse for the tests.

Point 55. Withdrawn on 2026-11-23, when the owner decided that $Q$ is turned so that the picture appears on the reference face in the default view the same way up as it is painted (`DEC-122`). The numbering of the edges counterclockwise from edge 0 stays a proposal of `DEC-117`, and point 48 of Section 11.3 of the command-line design puts how a user finds edge 0 on the face mask.

Point 56. Marks at both ends of an edge paired with itself by a turned neighbour (`DEC-118`, Section 5.32), which reuse point 51. The alternative is a mark of another form that tells the half-turn apart.

Point 57. One arrangement for each class (`DEC-115`, Section 5.30). The search keeps the class table of each class of subgroups, as the command line does for the colourings (`DEC-37`), so the user chooses among the classes and not among the conjugates of one class. No source read states how the pictures of two conjugate arrangements relate. The alternative lists every subgroup, which multiplies the listing by the class sizes.

Point 58. The constructions with one face type on `*n22` (`DEC-120`, Section 5.29). `WholeFace::new` refuses them, such as `010` on `*722`, with the text `the construction is not a regular tiling`, because `DEC-81` keeps `*n22` out of the regular tilings, and the owner has decided neither. The alternative accepts them as regular tilings, with the alternative of point 41 of Section 11.3 of the command-line design for `DEC-81`.

Point 59. Withdrawn on 2026-11-23, when the owner decided that $Q$ is turned so that the picture appears on the reference face in the default view the same way up as it is painted (`DEC-122`).

## 13. Security Considerations

The threat model is a caller that passes options and a domain derived from untrusted text to the crate, such as `escher-cli` with the arguments of a user. `escher-signature` has bounded the signature, and `escher-domain` has rejected every signature without a domain, every corner order above 100, every signature with more than eight orders, every value of a shape parameter outside its range, every polygon or double of a polygon that fails `verify()`, every wallpaper domain that fails `verify()`, every spherical group domain other than `*` that fails `verify()`, and every `2*n` with $n$ above 50 (DOM-38, DOM-44, DOM-54, DOM-60, DOM-63, and Section 16 of the domain design). The attacker can choose any domain that `build` returns, any value of each option, and every byte of the stream that `Image::decode_png` reads. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of memory through a large image or through the header of a PNG stream that names one, exhaustion of time through a large image, a fine sample grid, and a high fold cap, a panic that stops the caller, including a panic on a malformed PNG stream, and an image that the caller trusts as correct although it is wrong. The crate counters them with the limits of this section, which RND-4 requires, the rule that it never panics (RND-14), and the tests of Section 8:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Width and height | 16384 pixels each | A choice of this design. The limit on pixels bounds the memory |
| Pixels | $2^{26}$ | The buffer of `render` holds 3 bytes per pixel, about 201 MB at this limit. A choice of this design |
| Sample grid | 16 by 16 | 256 samples per pixel, far beyond the 16 of [NS]. A choice of this design |
| Border width | 1024 pixels, or 1024 in the intrinsic unit (`DEC-78`) | The largest image side divided by 16. A choice of this design. An intrinsic width costs a fixed number of operations per side and sample whatever its value |
| Fold cap | 100000 | About 80 times the largest step count that was measured when the limit was set. The time of an unconverged sample is proportional to the cap. A choice of this design |
| Width and height of a decoded image | 16384 pixels each, checked from the header before the texels are allocated (RND-62) | The limit of the rendered image, as the owner decided (`DEC-92`) |
| Pixels of a decoded image | $2^{26}$ | The limit of the rendered image, as the owner decided (`DEC-92`). The texels take 4 bytes per pixel, $2^{28}$ bytes at this limit |
| Working memory of the PNG decoder | $2^{24}$ bytes besides the texels, set as `png::Limits` (Section 5.21) | A choice of this design. It bounds what `png` allocates while it reads the chunks, not the size of an image |
| Size of a mask | The row `size` of Section 4.1, with the sample grid of Section 5.19 | `Masks::render` returns a buffer of 3 bytes per pixel, as `render` does |

The limits bound memory. They do not bound time usefully when every option is at its limit together: $2^{26}$ pixels, 256 samples per pixel, and a cap of 100000 allow about $1.7 \times 10^{15}$ steps for an image whose samples do not converge. This is a residual risk. A caller that renders untrusted input with a deadline applies its own limits on top of these.

The crate allocates the buffer of `render` and nothing that depends on the domain beyond its 0 to 14 sides. The hyperbolic model adds no limit, because each model costs a fixed number of operations per sample before the fold (Section 5.16). For a spherical domain the work of a sample is bounded by the fold cap, as for the other geometries, and the options of the ball add no limit, because `build` rejects every value that is not finite and each costs a fixed number of operations per sample. `render_rows` writes only into the buffer of the caller. A coset fill adds the side buffer of each call, at most the fold cap of bytes, and a trace of at most the fold cap of table reads for each sample. Its coset table is bounded by the limits of `escher-group` (Section 15 of the group design). The relator check of `CosetFill::new` runs once for each fill, and Section 5.12 bounds its table reads at those limits. The closure of `Masks::new` holds at most the square of the colour count of pairs for each side and reads a fixed number of table entries for each pair and generator, so its work, and the seam classes and marks that it finds, are bounded by the colour count limit of `escher-group` and the number of sides. An image fill costs a fixed number of operations per side and sample and allocates nothing per sample. The crate keeps no limit on the number of images, so the texels of the images of one coset fill take up to $2^{28}$ bytes for each colour that has an image fill. `Masks::render` allocates its buffer and nothing per sample, and runs no fold, so its work is bounded by the size. A Wythoff fill adds no limit: `Wythoff::new` computes a fixed number of points and walls, the face of a sample and its border test take at most two dot products and two distances for each corner, and a construction has at most three face types. A whole-face fill adds no limit either. `WholeFace::new` keeps $2n$ words of at most $2n - 1$ letters and $n$ edge words of at most $4n - 1$ letters, with $n$ at most 100 by the limit on corner orders of the domain design, and the checks of Section 5.30 read at most $2n$ traces of those lengths. `WholeFaceFill` keeps one flat matrix for each coset, and a sample adds one trace of its side sequence, at most the fold cap of table reads, as `colour_of` does, and one lookup. The arrangement comes from `escher-group`, whose limits bound its table (Section 15 of the group design), and its index is $2n$, which the index limit of `escher-group` keeps at most 64. The face mask allocates its buffer and nothing per sample, as `Masks::render` does.

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library. `png` forbids unsafe code in its own source. The crates `crc32fast` and `simd-adler32`, on which it depends, hold architecture-specific code behind runtime detection.
- Untrusted input and file paths: the options, the domain, the points of `fold`, the buffers, and the stream of `Image::decode_png` are the inputs. The crate opens no file and takes no path, reads only the reader that the caller gives, keeps no state between calls, and returns each PNG file as bytes. Sections 4.6 to 4.8 of the command-line design state how the files are opened and written. The decoder of `png` parses the stream in safe Rust (`png` forbids unsafe code in its own source).
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only. Among `png` and its dependencies, only `crc32fast` 1.5.5 has a build script.
- Dependencies: `Cargo.lock` fixes every version, including `flate2` at 1.1.9, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate neither builds nor writes anything outside the memory that the caller gives it.

The residual risks are twelve. First, the limits do not bound time, as stated above. Second, an image is wrong where the fold decides wrongly: beyond the range of accuracy the crate states no accuracy, and within it the bounds of Section 6 allow a stated number of differing decisions per band. Third, the border estimate, the intrinsic border, the parity colouring, the coset fill, the seam classes of the masks, the mask polygon, the mask map, the faces of a Wythoff construction, and the stabilizer, the edge pairing, and the picture of a whole-face fill rest on unverified statements (Section 10), which the tests check numerically but no source establishes. Fourth, the time to decode a stream grows with its length, which the crate does not limit, so a long stream of chunks that `png` skips, or a reader that blocks, holds the caller until the stream ends. Fifth, an image fill can give the background to a point at a subnormal distance from the centre of the mask map (Section 5.23). Sixth, no test exercises the negation of the result of `wall_meet` on the sphere in steps 5 and 6 of Section 5.24: on the Wythoff triangle set, `spherical::wall_meet` already returns the generator point and every foot in the hemisphere of the centre, so every test passes without the negation. "Generator point" checks on that set that $G$ and each foot lie inside the triangle, so a result in the other hemisphere there fails it. Seventh, on a construction with one face type, every face has the same colour, so "Face oracle" sees no edge and checks only that each pixel shows that colour. On whole images, only "Wythoff borders" and the golden image `wy-7-3-2-010.png` check the edges of such a construction. Eighth, no tested input reaches the refusal of a domain without a side presentation at step 4 of Section 5.29, or the fallback of a whole face that holds no face mask map (step 8 of Section 5.29), because every domain that `build_with` returns has a side presentation, and no tested domain fails to give a face mask map. Ninth, a wrong flag or a wrong partner in the edge pairing changes only what `edges` reports and the face mask, and not the render, so only "Edge pairing", "Pairing geometry", "Face seams", "Face mask content", the golden face mask, and, for a wrong flag only, the handedness check of "Whole picture" see it. Tenth, no tested input reaches three conditions of the scope of Section 5.29: the order 2 of corner 2, the order of at least 3 of the face corner, and $p \ge q$. On the Wythoff triangle set, a construction that fails one of the first two has two face types or is refused by `Wythoff::new`, and the canonical form of a signature orders the corners of a triangle by decreasing order, so no test fails when one of these conditions is dropped. Eleventh, the dodecahedron of `*532` with `010` has one arrangement, and it has no mirrored neighbour, so no whole-image test covers a mirrored neighbour on the sphere. Twelfth, in the disk and on the ball, the picture appears the same way up only at the centre of the reference face, and "Upright picture" measures directions away from the centre within an angle that the test sets (Section 5.31).

## 14. Operational Considerations

The crate declares no Cargo feature and has no build script. The golden-image test reads `CARGO_TARGET_TMPDIR`, which Cargo sets for integration tests, and `ESCHER_UPDATE_GOLDEN`, which the recipe `update-golden` sets (Section 8.4). The library reads no environment variable. It is pure Rust with no platform-specific code of its own, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log. `RenderStats` is the measurement that the caller reports.

| Failure | Reported as | Recovery |
| --- | --- | --- |
| An option outside its accepted values | `InvalidOption` | Use a value in the column "Accepted" of Section 4.1 |
| An option above its limit | `LimitExceeded` | Use a value within the limit |
| A domain without a side presentation, a colouring with another number of generators than the domain has sides, a colouring that a relator of the side presentation breaks, or the wrong number of colours | `InvalidOption` from `CosetFill::new` | Take the colouring from the side presentation of the same domain, and give one colour for each coset |
| A coset fill used with a domain of another side presentation | `InvalidOption` from `Renderer::new` | Build the renderer with the domain that the fill was built for, or with a domain of the same side presentation |
| A buffer or band that does not fit the image | `InvalidBuffer` | None. It is a defect of the caller |
| `png` fails to encode | `Encoding`, with the `png` error as `source()` | None. It is a defect of the crate or of `png` |
| A sample that does not converge | `Fold::converged` is false, the sample takes the border colour, and `RenderStats::unconverged` counts it | Raise the fold cap, or accept the border colour near the limit circle, far from the centre of a very elongated Euclidean image, or near a vertex of very high order on the ball or of a rotation domain |
| A sample outside the hyperbolic model or the ball | The background colour, counted by `RenderStats::outside` | None needed |
| A colouring that does not belong to the domain of `Masks::new`, or a colour at or above `Masks::count` | `InvalidOption` | Take the colouring from the side presentation of the same domain, and a colour below the count |
| An image over a size limit of Section 5.21 | `LimitExceeded` from `Image::decode_png` | Scale the image down |
| An Exif chunk larger than the working memory of the PNG decoder (Section 5.21) | `LimitExceeded` from `Image::decode_png`, with the text of the memory limit and the `png` error as `source()` | Save the image without that metadata |
| A domain without a mask polygon given to `Masks::new`, or to `Renderer::new` with an image fill | `InvalidOption`, with the texts of `build masks` and `build renderer` of Section 7.2 | None. Use solid paints on such a domain (Section 5.22) |
| A stream that cannot be read or is not a valid PNG image | `Decoding`, with the `png` error as `source()` | Save the image as PNG again |
| An image with 16 bits per channel | `Decoding`, with no `source()` | Save the image with 8 bits per channel |
| An image fill at a point that the mask map cannot carry, or whose image position lies outside its image | The background colour, with no error | None needed. Under the statements of Section 10 on the chart and the mask map, no converged fold reaches it, except at a point at a subnormal distance from the centre of the mask map (Section 5.23) |
| A domain that is not a reflection triangle, or marks that give no face, given to `Wythoff::new` | `InvalidOption` | Use a reflection triangle, and turn on a mirror that gives a face |
| A construction whose generator point, perpendiculars, or feet cannot be computed | `Numerical` from `Wythoff::new` | None. No construction that steps 1 and 4 of Section 5.24 accept is known to produce it, and one that does is a defect of the crate |
| A Wythoff fill with the wrong number of colours, or used with another domain | `InvalidOption` from `WythoffFill::new` or `Renderer::new` | Give one colour for each face type, and build the renderer with the domain of the construction |
| A construction that is not a regular tiling, or of another domain, given to `WholeFace::new` | `InvalidOption` | Use the marks `010` or `100` on `*pq2` with $p \ge q \ge 3$, with the domain of the construction |
| A colouring that is not an arrangement of the faces, given to `WholeFace::edges`, `WholeFace::render_mask`, or `WholeFaceFill::new` | `InvalidOption` | Take the arrangement from `free_classes_of_index` with the presentation, stabilizer, and index of the same `WholeFace` |
| A whole-face fill with other than one paint, or used with another domain | `InvalidOption` from `WholeFaceFill::new` or `Renderer::new` | Give one paint, and build the renderer with the domain of the construction |

## 15. References

### 15.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 15.2. Informative References

The tags of the mathematical and software sources resolve to the reference sections of the research documents:

- [BEA] Section 16.1 of `docs/research/geometry-models.md`.
- [CBG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CFKP] Section 14 of `docs/research/rendering.md`, and Section 16.1 of `docs/research/geometry-models.md`.
- [CBGS2008] Section 15 of `docs/research/colourings.md`.
- [CH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CLM1954] Section 15 of `docs/research/colourings.md`.
- [DAVIS] Section 14 of `docs/research/rendering.md`.
- [DUN86] Section 16.1 of `docs/research/geometry-models.md`.
- [FRETTLOH2008] Section 15 of `docs/research/colourings.md`.
- [HAREL1993] Section 15 of `docs/research/colourings.md`.
- [HOLT2005] Section 15 of `docs/research/colourings.md`.
- [NS] Section 14 of `docs/research/rendering.md`.
- [PHONG75] Section 16.4 of `docs/research/geometry-models.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [SCHULTE2016] Section 15 of `docs/research/colourings.md`.
- [SERIES2013] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SOLOMYAK2020] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SYMMHUB] Section 14 of `docs/research/rendering.md`.
- [VGRG] Section 14 of `docs/research/rendering.md`.
- [W-PHONG] Section 16.3 of `docs/research/geometry-models.md`.
- [YUNCKEN2003] Section 15 of `docs/research/colourings.md`.
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
| RND-16 | A test of a bound names its input set and prints its largest value, a bound on a count is the count bound, and this document records a bound only where it is part of a contract | 6.5 |
| RND-17 | `Error` follows ERR-1, with the operation that failed | 7.1 |
| RND-18 | The `Display` text of each error is the one stated | 7.2 |
| RND-19 | The reference module shares no code with the crates under test | 8.3 |
| RND-20 | Golden images are compared in linear light, through a table from the reference transfer function, with the allowance of Section 8.4 | 8.4 |
| RND-21 | Withdrawn on 2026-11-19. | 8.1 |
| RND-22 | The crate declares only the dependencies of Section 9, each from the workspace table | 9 |
| RND-23 | `Renderer::new` chooses the view from the geometry of the domain: the disk or the view of the hyperbolic model, the plane, or the ball | 5.8 |
| RND-24 | For a Euclidean domain, samples, plane positions, points, and screen tangents follow the formulas of Section 5.8 with $S = 8$ | 5.8 |
| RND-25 | For a Euclidean domain, the bounds of Section 6.6 hold within distance 65 of the centre, and beyond it only a result without a panic is promised | 6.6 |
| RND-26 | The fold reads the point size at steps 2 and 3, which for a hyperbolic or Euclidean domain is $x_3$ with the threshold $\tau \sigma_i x_3$ taken from left to right | 5.3 |
| RND-27 | For a spherical domain, samples, ball positions, the lift, the screen tangents, and the viewing matrix follow the formulas of Section 5.9 | 5.9 |
| RND-28 | The viewing matrix applies the yaw, the pitch, and the roll in that order about fixed screen axes, with signs defined by the motion on the screen | 5.9 |
| RND-29 | With `Shading::Phong`, every sample of the ball is shaded by the formula and constants of Section 5.10 before averaging, with the power by squaring, and `Shading::Flat` changes no colour | 5.10 |
| RND-30 | For a spherical domain, the bounds of Section 6.7 hold for a converged fold of a vector whose largest absolute coordinate is at least $10^{-300}$ and of a ball position inside the rim, the border count bounds hold in the rim bands of the range of accuracy, and closer to the rim only a result without a panic is promised | 6.7 |
| RND-31 | The fold of a rotation domain follows Section 5.3, with no branch on the kind of a side and no change to $\tau$ or the order of operations | 5.11 |
| RND-32 | Support for rotation domains changes no decision, colour, or statistic of a reflection domain | 5.11 |
| RND-33 | `Fill::Parity` on a domain with no reversing side gives its first colour, and `Renderer::new` does not refuse it | 5.11 |
| RND-34 | The border estimate tests the sides of the domain only, so the doubled wall draws no border | 5.11 |
| RND-35 | For a rotation domain, the bounds of Section 6.8 hold in its range of accuracy, and beyond it only a result without a panic is promised. For a hyperbolic rotation domain whose corner orders are all near 100, the range excludes the points within a hyperbolic distance of $10^{-8}$ of a vertex | 6.8 |
| RND-36 | `CosetFill::new` refuses, with `InvalidOption` and in this order, a domain without a side presentation, a colouring with another number of generators than the domain has sides, a colouring that a relator of the side presentation breaks, and a wrong number of colours | 5.12 |
| RND-37 | A coset fill colours a sample by `colour_of` of its side sequence only, with no fallback colour | 5.12 |
| RND-38 | The side sequence goes into a side buffer of each call, cleared before each sample, so no value passes between samples, bands, or threads | 5.12 |
| RND-39 | The coset fill changes no decision, colour, statistic, or golden file of `Fill::One` and `Fill::Parity` | 5.12 |
| RND-40 | The crate declares `escher-group` and takes every colouring and coset colour from it | 9 |
| RND-41 | `Renderer::new` refuses, with `InvalidOption`, a coset fill built for another side presentation than that of the domain and a coset fill with an image fill on a domain without a mask polygon, and gives no error for every other fill except the Wythoff fill and the whole-face fill of another domain (RND-68, RND-76) | 5.12 |
| RND-42 | For a reflection polygon and its double, the bounds of Section 6.9 hold for the shapes of the polygon domain set in each radius band, and for every other shape and beyond the range of accuracy only a result without a panic is promised | 6.9 |
| RND-43 | The fold of a wallpaper domain follows Section 5.3 with no branch on the kind of a side, and a step across a glide reflection counts as a reversal | 5.14 |
| RND-44 | The border estimate tests every side of a wallpaper domain, mirror or paired, with one width and colour | 5.14 |
| RND-45 | For a wallpaper domain, the bounds of Section 6.10 hold for the settings of the wallpaper domain set within distance 65 of the centre, and for every other setting and beyond that distance only a result without a panic is promised | 6.10 |
| RND-46 | The fold of a spherical group domain follows Section 5.3 with no branch on the kind of a side, and a step across a rotary reflection counts as a reversal | 5.15 |
| RND-47 | The border estimate tests every side of a spherical group domain, mirror or paired, with one width and colour | 5.15 |
| RND-48 | For a spherical group domain, the bounds of Section 6.11 hold on the spherical group domain set for a converged fold, and for every other domain and an unconverged fold only a result without a panic is promised | 6.11 |
| RND-49 | For a domain with no side, every point that passes step 2 converges in zero steps, no sample is a border sample, and `Renderer::new` accepts the domain | 5.15 |
| RND-50 | With `HyperbolicModel::Disk`, the default, the samples, disk positions, points, and screen tangents of a hyperbolic domain follow Sections 5.1 and 5.2 with the same operations as before the option | 5.16 |
| RND-51 | In the half-plane, the band, and the Klein model, each sample follows the steps of Section 5.16, with the model map and its Jacobian from `escher-geometry`, and a rejected position is an outside sample | 5.16 |
| RND-52 | The fold, the border estimate, and the colour of a sample do not read the hyperbolic model | 5.16 |
| RND-53 | In the half-plane, the band, and the Klein model, the bounds of Sections 6.4 and 6.9 hold for the samples whose disk position lies in the range of accuracy, the tangent bound of each model of Section 6.12 holds on its input set, and every other sample only returns without a panic with a colour of Section 5.5 | 6.12 |
| RND-54 | With `BorderUnit::Intrinsic`, a converged sample is a border sample by the negated comparison of the wall distance of some side with the half width | 5.17 |
| RND-55 | With `BorderUnit::Pixels`, every sample is computed by the operations used without the option | 5.17 |
| RND-56 | The intrinsic border test reads no hyperbolic model, image size, sample grid, or screen tangent | 5.17 |
| RND-57 | Withdrawn on 2026-11-23. | 5.18 |
| RND-58 | `Masks::render` computes each sample by the mask placement and the steps of Section 5.19, gives both edges of a seam one code colour and matching marks, and reads no render option | 5.19 |
| RND-59 | An image fill takes its value from the folded point and the image of its colour, and depends on no model, rotation, render size, sample grid, or border | 5.20 |
| RND-60 | Image sampling interpolates in linear light the texels composited over the background, falls back to the background, and returns a valid colour for every point | 5.20 |
| RND-61 | A coset fill of solid paints changes no decision, colour, statistic, or golden file | 5.20 |
| RND-62 | `Image::decode_png` checks the size and the bit depth from the header before it allocates the texels, returns `LimitExceeded` or `Decoding`, and never panics | 5.21 |
| RND-63 | `Masks::new` and `Renderer::new` build the mask polygon by the rules of Section 5.22 only, and refuse a domain to which no rule applies | 5.22 |
| RND-64 | The mask map is computed by the steps of Section 5.23 from the domain and its mask polygon only | 5.23 |
| RND-65 | `Wythoff::new` builds the construction by the steps of Section 5.24 only, refuses with `InvalidOption` at the first step that refuses, and returns `Numerical` at step 8 | 5.24 |
| RND-66 | With a Wythoff fill, the face type of a sample comes from the steps of Section 5.25, through one private function | 5.25 |
| RND-67 | With a Wythoff fill, a border sample is decided on the perpendiculars of the active sides through the corner of its face, and never on the sides of the domain | 5.26 |
| RND-68 | `Renderer::new` refuses a Wythoff fill of another domain, and a sample takes the colour of its face type | 5.27 |
| RND-69 | Withdrawn on 2026-11-23. | 5.27 |
| RND-70 | A Wythoff fill changes no decision, colour, statistic, or golden file of the other fills or of the masks | 5.27 |
| RND-71 | `WholeFace::new` builds the whole face by the steps of Section 5.29, refuses at the first step that refuses, and accepts every regular construction of a domain with a side presentation | 5.29 |
| RND-72 | An arrangement is checked by the three checks of Section 5.30 in order, and its stabilizer elements and edge pairing come from the steps of Section 5.30 through one private function | 5.30 |
| RND-73 | With a whole-face fill, the value of a sample comes from the steps of Section 5.31 through one private function, with the face and border of the Wythoff fill | 5.31 |
| RND-74 | The face mask draws $Q$ by the steps of Section 5.19, with one code colour for each pair of edges and for each edge paired with itself, marks by the carrying of Section 5.32, and no render option | 5.32 |
| RND-75 | A whole-face fill changes no decision, colour, statistic, or golden file of the other fills or of the masks | 5.32 |
| RND-76 | `Renderer::new` refuses a whole-face fill of another domain | 5.32 |
