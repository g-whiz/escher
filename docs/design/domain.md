# Domain Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-22 |
| Applies to | `crates/escher-domain` |

## Abstract

This document specifies the crate `escher-domain`, which builds the fundamental domain of an orbifold signature as a polygon in the linear model, with a wall, a pairing isometry, and a partner for each side, and verifies it numerically. It covers the compact hyperbolic reflection triangle `*pqr`, the Euclidean reflection triangles `*632`, `*442`, and `*333`, the rectangle `*2222` with its shape parameter `aspect`, the spherical reflection triangles `*332`, `*432`, `*532`, and `*n22`, the lune of `*nn`, and the compact hyperbolic reflection polygons `*p1...pn` with four to eight corners and their $n - 3$ length parameters. It covers the rotation signatures of each of these, whose domain is the double of the reflection domain across one wall. It covers the nine other Euclidean wallpaper signatures, with the shape parameter `skew` of `o` and `2222`, and the seven remaining spherical signatures `n*`, `nx`, `2*n`, `3*2`, `*`, `x`, and `1`. It defines the public types and operations, the exact set of signatures that have a domain, the shape parameters with their ranges, defaults, and validation, and each construction as a numbered algorithm with the source of each step. It specifies the measurements of the sides, angles, and area, the six numeric checks of the verification with their tolerances, the guard that every polygon, wallpaper domain, and spherical group domain passes before `build_with` returns it, and the side presentation of a domain. It also states the numerical accuracy, the error type, the resource limits, the tests, the dependencies, the unverified statements that the code relies on, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

The renderer folds every pixel into the fundamental domain and colours it by the side pairings that the fold applied. It needs a polygon whose sides carry exact walls and pairing isometries, placed so that the canonical view is centred on it. The research gives the angles of the reflection triangle, its side lengths by the law of cosines, and the theorems that make it a fundamental domain, but it gives no placement in the linear model and no numerical tolerance (Section 10 of the geometry reference and Section 4.8 of the domain reference). A wrong normal, a wrong angle, or a wrong pairing produces an image that looks plausible, so the crate must check its own result.

### 1.2. Purpose

`escher-domain` is the one place where a `Signature` becomes a `FundamentalDomain`. It builds the polygon of a supported signature, returns an error kind for every other signature, and verifies a domain against the conditions of the polygon theorems that the domain reference cites. Section 3 of the architecture places the crate in the workspace. It depends on `escher-signature` for the signature and on `escher-geometry` for the linear model, and the renderer and `escher-cli` consume it.

### 1.3. Audience

The primary readers are the agent that changes the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-cli`, which prints the domain summary and the result of the verification, and of `escher-render`, which folds into the domain.

### 1.4. Scope

This document covers the public surface of the crate, the supported signatures, the shape parameters `aspect` and `skew` and the length parameters of the polygons, the conventions of the sides, the vertices, and the pairings, the construction of the compact hyperbolic reflection triangle, the compact hyperbolic reflection polygon, the Euclidean triangle, the rectangle, the spherical triangle, the lune, the double of each across one wall, the domains of the nine other Euclidean wallpaper signatures, and the domains of the seven spherical group signatures, their placement, their measurements, the side presentation, the verification, the numerical accuracy, the error type, the resource limits, the tests, and the dependencies.

The supported signatures are the compact hyperbolic `*pqr` with $1/p + 1/q + 1/r < 1$, the compact hyperbolic `*p1...pn` with four to eight corners, the Euclidean `*632`, `*442`, `*333`, and `*2222`, the spherical `*332`, `*432`, `*532`, `*n22`, and `*nn`, the rotation signature of each of them, which is the signature without the star, such as `732` for `*732`, the Euclidean wallpaper signatures `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, and the spherical group signatures `n*`, `nx`, `2*n`, `3*2`, `*`, `x`, and `1` (Section 4). With `*632`, `*442`, `*333`, `*2222`, `632`, `442`, `333`, and `2222`, the wallpaper signatures are the 17 Euclidean signatures. With `*332`, `*432`, `*532`, `*n22`, `*nn`, and their rotation signatures, the spherical group signatures make every valid spherical signature supported, up to the limits of Section 4.

### 1.5. Non-Goals

These topics are owned elsewhere or are outside this document:

- The bilinear form, the side test, the distance, the isometries, the reflection, the wall through two points, and the disk map, with their formulas and tolerances, belong to `escher-geometry` and are specified in `docs/design/geometry.md`. This document calls those functions by name and restates none of their formulas.
- The lines that `escher info` prints for a domain and for `--check`, their format, and the exit status are specified in `docs/design/cli.md`.
- Every shape parameter other than `aspect`, `skew`, and the lengths of the polygons `*p1...pn` (Section 6.6), including a parameter of the kites of `4*2` and `3*3`, which have none, the parameters of the rotation signatures `p1...pn` beyond their default shape, such as tubes, twists, and cone pairs, and the kind and unit of a `ParameterSpec` that Section 3.2 of the architecture names. A reflection triangle has no free parameter ("triangles are rigid", Section 10.1 of the geometry reference).
- Polygons with more than eight corners, cone points on a mirror, and the pillows and hats of the domain reference (Section 8.6 of the domain reference). Each returns the error kind that Section 4 states.
- Words, relators, presentations, their text, and coset enumeration, which belong to `escher-group` and are specified in Sections 4 to 7 of `docs/design/group.md`. This document specifies only which presentation a domain gives (Section 5.4).
- Ideal vertices, the hyperbolic signatures that mix cone points with a boundary, such as `4*3`, the friezes, and every other class of signature in the plan. Each returns `UnsupportedSignature` (Section 4).
- Any change to `deformation_dimension()` of `escher-signature`, which stays as the signature design states for every Euclidean signature (`UNV-10`), and any new function of `escher-geometry` (`DEC-53` of the architecture).
- The canonical view, the model, and the margin of `DEC-12`, which belong to `View` in `escher-render`. This crate places only the centre (Section 6.3).
- The fold with general pairings, its tolerance, its step counts, and the colouring of a domain without a reversing side, which belong to `escher-render` and are specified in `docs/design/render.md`.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md` and the terms linear model, fundamental domain, tile, side, wall, covector, pairing isometry, and reference arithmetic of Section 2.1 of `docs/design/architecture.md`. It cites these documents by short names:

| Document | Short name |
| --- | --- |
| `docs/design/architecture.md` | the architecture |
| `docs/design/poc-plan.md` | the plan |
| `docs/design/geometry.md` | the geometry design |
| `docs/design/signature.md` | the signature design |
| `docs/design/cli.md` | the command-line design |
| `docs/design/group.md` | the group design |
| `docs/research/geometry-models.md` | the geometry reference |
| `docs/research/fundamental-domains.md` | the domain reference |
| `docs/research/orbifold-notation.md` | the notation reference |
| `docs/rust-coding-standards.md` | the coding standard |

A function written as `hyperbolic::name`, `euclidean::name`, or `spherical::name` is the function of that name in the module `hyperbolic`, `euclidean`, or `spherical` of `escher-geometry`, which the geometry design specifies.

It adds these terms:

- supported signature: A signature that meets the predicate of Section 4. `FundamentalDomain::build` constructs a domain for it when it has at most eight orders and its corner orders are at most 1000 (DOM-3 and DOM-38).
- corner $j$: For a reflection signature, the corner of the canonical signature at index $j$ of `boundaries()[0].corner_orders()`, with the order $n_j$ and the angle $\alpha_j = \pi/n_j$. For a rotation signature, the corner of its starred signature at that index, whose order is the cone order at index $j$ of `cone_orders()`.
- $m$: The number of sides of a domain, which is also its number of vertices: 0 for the whole sphere of `1`, 2 for a lune, for the double of a lune, and for the domains of `nx`, `x`, and `*`, 3 for a triangle, 4 for the rectangle, for a kite, and for the parallelogram of `o`, 5 for the domain of `2*22`, 6 for the double of the rectangle and for the domain of `22*`, $n$ for a polygon with $n$ corners, and $2n - 2$ for its double.
- wallpaper signature: One of the nine Euclidean signatures `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3`, whose structure the predicate of Section 4 matches exactly.
- wallpaper domain: The domain of a wallpaper signature that Section 6.13 builds, whose sides are mirrors or are paired by translations, half-turns, rotations, or glide reflections.
- the ten domains: The nine wallpaper domains and the domain of `2222` at every value of `skew` (DOM-50).
- spherical group signature: One of the spherical signatures `n*` and `nx` with $n \ge 2$, `2*n` with $n \ge 2$, `3*2`, `*`, `x`, and `1`, whose structure the fourth class of the predicate of Section 4 matches exactly.
- spherical group domain: The domain of a spherical group signature that Section 6.16 builds (DOM-58).
- spherical group test set: The spherical group signatures that the tests of this crate build: `3*2`, `*`, `x`, and `1`, the families `n*` and `nx` for $n$ from 2 to 12 and at 1000, and the family `2*n` for $n$ from 2 to 12 and at 500. It is not the spherical domain set or the spherical group domain set of `docs/design/render.md`.
- spherical kite: The domain of `3*2`, `2*n`, or `n*`, which is the kite of a spherical double with one pair of its sides turned into mirrors (Section 6.16).
- lune of order 1: The lune of Section 6.7 at $n = 1$, a hemisphere whose two sides lie on one great circle, with exact normals (Section 6.16).
- whole sphere: The domain of `1`, with no side, no vertex, and no corner.
- rotary reflection: A rotation of the sphere about an axis composed with the reflection in the plane through the centre of the sphere perpendicular to that axis. It reverses orientation. The antipodal map is its case with the rotation through $\pi$ (item 33 of Section 15.1 of the geometry reference).
- census: The counts of the elements of a finite group of isometries of the sphere by kind, which the certificate of DOM-64 asserts.
- glide reflection: A reflection in a line composed with a translation along that line (Section 12 of the geometry reference). It reverses orientation and fixes no point.
- half-turn: The rotation through $\pi$ about a point.
- state: A pair $(v, d)$ of a vertex $v$ and a direction $d$, which is $+$ or $-$, that the walk of DOM-55 visits.
- cycle order: The order that every vertex of a vertex cycle stores, which is the order $m_c$ of its cycle relator.
- chain: A vertex cycle whose walk passes a mirror, so that it visits some of its vertices in both directions. Its order is the order of a corner reflector of the signature, or 1.
- cycle rule: The reading of check 5 that compares the angle sum over the states of a cycle with $2\pi$ divided by its order (DOM-56).
- polygon: The compact hyperbolic reflection polygon of a signature `*p1...pn` with $n$ from 4 to 8 corners, which Section 6.10 builds. A triangle is not a polygon in this sense.
- feature: An element of the cyclic list that Section 6.10 cuts: a corner, a corner pair, or a slice. A feature stands for the arc of consecutive vertices from its first vertex to its last vertex, written $[\mathrm{first}(f) .. \mathrm{last}(f)]$, with indexes modulo $n$.
- corner pair: Two consecutive corners of order 2 that Section 6.10 replaces by one feature, which stands for the mirror side between their two right angles, with its length as a shape parameter.
- slice: A cut of the polygon along the common perpendicular of two of its sides, with its length as a shape parameter (Section 6.10).
- sheet: A generalized triangle of three features that the cuts of Section 6.10 produce.
- gap: The segment of a sheet between two consecutive features. It lies on one side of the polygon.
- double-double number: An unevaluated sum $h + l$ of two `f64` values with $\lvert l \rvert \le \frac{1}{2} \mathrm{ulp}(h)$, which carries about 106 bits of significand.
- guard: The verification of DOM-44, DOM-54, or DOM-60, which `build_with` runs on a polygon or its double, on one of the ten domains, or on a spherical group domain other than that of `*` before it returns it.
- reflection signature: A supported signature with one boundary, no cone point, no handle, and no cross-cap, such as `*732`. Its domain is a reflection domain, every side of which is a mirror. No wallpaper signature is one.
- rotation signature: A supported signature with cone points only, such as `732`. Its domain is a double (Section 6.8). No wallpaper signature is one.
- starred signature: For a rotation signature, the reflection signature with a star before the same orders, so `*732` for `732` and `*2222` for `2222`.
- generating domain: The reflection domain $F$ of the starred signature, as Sections 6.1 to 6.7 and 6.10 build it, from which the double of a rotation signature is built.
- double: The union $D = F \cup R_1(F)$ of the generating domain and its image under the reflection $R_1$ in the wall of its side 1 (Section 6.8).
- kite: The double of a triangle, with four vertices.
- double of a polygon: The double of a polygon of Section 6.10 at its default shape, with $2n - 2$ vertices (DOM-47).
- flat vertex: A vertex of a double, of a wallpaper domain, or of a spherical group domain at which two sides on one wall meet at the angle $\pi$ (Sections 6.8, 6.13, and 6.16).
- copies: The number of corners of $F$ that a vertex of a double joins, 2 for the two vertices on the doubled wall and 1 for every other vertex. The expected angle of a vertex is its copies times $\pi$ divided by its order. A wallpaper domain stores the copies 1 at every vertex, which no check reads (Section 8.2).
- vertex cycle: The sequence of vertices that the side pairings carry into one another, as Section 4.1 of the domain reference defines it after Definition 11 of [SOLOMYAK2020].
- origin: `hyperbolic::ORIGIN` for a hyperbolic domain, `euclidean::ORIGIN` for a Euclidean one, and `spherical::ORIGIN` for a spherical one. All three are $(0, 0, 1)$.
- lune: The intersection of two distinct, nonopposite hemispheres of $S^2$, with two vertices and the same angle at both ([RAT], p. 51, Section 11.3 of the geometry reference).
- standard position: The placement in which vertex 2 is the origin and vertex 0 lies on the positive first axis, for a triangle (Section 6.1, step 5, and Section 6.5, step 2).
- final position: The placement in which the centre of `DEC-12` is the origin (Sections 6.3 and 6.5).
- shape parameter: A named real number that selects one domain of a signature whose domains form a family, such as `aspect` of `*2222` (Section 6.6).
- setting: A name and a value in a `ShapeParameters`.
- corner frame: The frame of corner $j$, in which vertex $j$ is `hyperbolic::ORIGIN` (Section 8.2, check 4, and the angles of DOM-46).
- check: One numeric condition that `verify()` evaluates, with a residual and a tolerance.
- residual: The non-negative number that a check computes, which is 0 for an exact domain, and infinity when a structural condition fails or when the computation gives no finite number. It is never NaN.
- tolerance: The largest residual for which a check passes.
- side presentation: The presentation that `side_presentation()` returns, with one generator for each side (Section 5.4).
- side generator: The generator of side $i$ in the side presentation, named `s<i>`.
- letter, word, relator: The terms of Section 4 of the group design. A relator has a base word and an exponent of at least 1.

## 3. Public Surface

### 3.1. Items

DOM-1. The crate MUST export exactly the items of this section from its root, each through one path, and MUST give no type a public field, because a public field or a constructor other than `build` and `build_with` lets a caller build a domain that breaks the invariants of Section 3.2, and the renderer folds into the domain without checking them (API-10 of the coding standard).

This listing shows each public item with its signature. Bodies are omitted. `DVec3` and `DMat3` are the `glam` types of `DEC-4` (Section 3.2 of the architecture):

```rust
pub struct FundamentalDomain { /* private */ }
// derives Clone, Debug, PartialEq
impl FundamentalDomain {
    pub fn build(signature: &escher_signature::Signature) -> Result<Self>;
    pub fn build_with(signature: &escher_signature::Signature, parameters: &ShapeParameters) -> Result<Self>;
    #[must_use] pub fn geometry(&self) -> escher_signature::Geometry;
    #[must_use] pub fn sides(&self) -> &[Side];
    #[must_use] pub fn vertices(&self) -> &[glam::DVec3];
    #[must_use] pub fn centre(&self) -> glam::DVec3;
    #[must_use] pub fn side_lengths(&self) -> Vec<f64>;
    #[must_use] pub fn corner_angles(&self) -> Vec<f64>;
    #[must_use] pub fn area(&self) -> f64;
    #[must_use] pub fn reverses_orientation(&self) -> bool;
    #[must_use] pub fn verify(&self) -> Verification;
    pub fn side_presentation(&self) -> Result<escher_group::Presentation>;
}

pub struct Side { /* private */ }
// derives Clone, Copy, Debug, PartialEq
impl Side {
    #[must_use] pub fn normal(self) -> glam::DVec3;
    #[must_use] pub fn covector(self) -> glam::DVec3;
    #[must_use] pub fn pairing(self) -> glam::DMat3;
    #[must_use] pub fn partner(self) -> usize;
    #[must_use] pub fn kind(self) -> PairingKind;
    #[must_use] pub fn orientation(self) -> Orientation;
}

#[non_exhaustive]
pub enum PairingKind { Mirror, Rotation, Translation, GlideReflection, RotaryReflection }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

#[expect(clippy::exhaustive_enums, reason = "an isometry either preserves or reverses orientation")]
pub enum Orientation { Preserving, Reversing }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub struct Verification { /* private */ }
// derives Clone, Debug, PartialEq
impl Verification {
    #[must_use] pub fn isometries(&self) -> Check;
    #[must_use] pub fn pairings(&self) -> Check;
    #[must_use] pub fn walls(&self) -> Check;
    #[must_use] pub fn relators(&self) -> Check;
    #[must_use] pub fn angles(&self) -> Check;
    #[must_use] pub fn area(&self) -> Check;
    #[must_use] pub fn passed(&self) -> bool;
}

pub struct Check { /* private */ }
// derives Clone, Copy, Debug, PartialEq
impl Check {
    #[must_use] pub fn residual(self) -> f64;
    #[must_use] pub fn tolerance(self) -> f64;
    #[must_use] pub fn passed(self) -> bool;
}

pub struct Error { /* private */ }
// derives Clone, Debug, PartialEq, Eq. Implements Display and std::error::Error by hand (Section 10.1)
impl Error {
    #[must_use] pub fn kind(&self) -> ErrorKind;
}

#[must_use] pub fn parameters(signature: &escher_signature::Signature) -> Vec<ParameterSpec>;

pub struct ParameterSpec { /* private */ }
// derives Clone, Debug, PartialEq
impl ParameterSpec {
    #[must_use] pub fn name(&self) -> &str;
    #[must_use] pub fn minimum(&self) -> f64;
    #[must_use] pub fn maximum(&self) -> f64;
    #[must_use] pub fn default_value(&self) -> f64;
}

pub struct ShapeParameters { /* private map from name to value */ }
// derives Clone, Debug, Default, PartialEq
impl ShapeParameters {
    #[must_use] pub fn new() -> Self;
    pub fn set(&mut self, name: impl Into<String>, value: f64) -> Option<f64>;
}

#[non_exhaustive]
pub enum ErrorKind { UnsupportedSignature, LimitExceeded, Numerical, UnknownParameter, ParameterOutOfRange, InvalidPairing }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

The crate does not re-export `glam` (API-10 of the coding standard). A caller that names a `glam` type declares `glam` from the workspace entry, as Section 3.1 of the architecture states for the crates that compute with these types. In the same way, the crate does not re-export `escher-group`, and a caller that names `Presentation` or enumerates it declares `escher-group` itself.

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `FundamentalDomain` | The fundamental domain of a supported signature | Built only by `build` and `build_with`. It holds the geometry of the signature, one private corner for each vertex with its order and its copies, the area of a hyperbolic or spherical signature, and the sides, the vertices, and the centre |
| `build` | The constructor with no setting | Equal to `build_with` with an empty `ShapeParameters`, so each shape parameter takes its default value |
| `build_with` | The constructor with settings | Returns a domain exactly for a supported signature with at most eight orders and corner orders of at most 1000, or at most 500 for `2*n` (Section 4), valid settings (Section 6.6), and, for a polygon, its double, one of the ten domains, or a spherical group domain other than that of `*`, a built domain that passes `verify()` (DOM-44, DOM-54, and DOM-60), and an `Error` otherwise. Never panics |
| `geometry()` | The geometry of the signature | `Hyperbolic`, `Euclidean`, or `Spherical` |
| `sides()` | The sides, in the order of Section 5.1 | $m$ sides, of kind `Mirror` for a reflection domain, of kind `Rotation` or `Translation` for a double, of the kinds of the table of Section 6.13 for a wallpaper domain, and of the kinds of the table of DOM-58 for a spherical group domain (DOM-27). None for the whole sphere |
| `vertices()` | The vertices, in the order of Section 5.1 | As many as the sides. For a reflection domain, vertex $j$ is corner $j$, for a double, the vertices follow Section 6.8, for a wallpaper domain and for `2222` with `skew` other than 0, Section 6.13, and for a spherical group domain, the table of DOM-58. The vertices are counterclockwise in the Klein model for a hyperbolic domain, in the plane for a Euclidean domain, where each is a Euclidean point $(x, y, 1)$, and as seen from outside the sphere for a spherical domain, where each is a sphere point whose last coordinate may be zero or negative (Section 5.2). Each has the accuracy of Section 9.3, 9.4, 9.5, or 9.7, and a caller does not renormalize it (Section 9.1) |
| `centre()` | The centre of `DEC-12`: the normalized sum of the vertices, for a Euclidean triangle and the rectangle of `*2222` their mean, for a lune `spherical::ORIGIN` (Section 6.7), for a double the centre of its generating domain (Section 6.8), and for a wallpaper domain the centre of its rectangle, parallelogram, or generating triangle (Section 6.13) | Within $10^{-9}$ of the origin in each coordinate for a hyperbolic domain (Sections 6.3 and 6.10), within the bound of Section 9.4 for a Euclidean one, within the bound of Section 9.5 for a spherical triangle, and exactly the origin for a lune. A wallpaper domain other than a kite, and `2222` at every value of `skew`, have exactly `euclidean::ORIGIN` as the centre. It is not the mean of the vertices of `2*22` or of `2222`. A kite has the centre of its double, so it lies within the bound of Section 9.4 of the origin. The centre of a double is bit for bit the centre of its generating domain, which for a kite is the triangle of `*442` or `*632`. A spherical kite has the centre of its double bit for bit, and the domains of `nx`, `x`, `*`, and `1` have exactly `spherical::ORIGIN` (DOM-58) |
| `Side::normal` | The unit normal of the wall of the side, for a Euclidean domain the unit normal $(a_1, a_2, 0)$ of its line, and for a spherical domain the great-circle normal of its great circle | Outward (Section 5.2), and spacelike for a hyperbolic domain |
| `Side::covector` | The covector of the wall | `hyperbolic::covector(normal)` for a hyperbolic domain, the line covector $(a_1, a_2, -t)$ for a Euclidean one, and the normal itself for a spherical one |
| `Side::pairing` | The pairing isometry $P_j$ of the side (Section 5.3 of the architecture) | For a mirror, `hyperbolic::reflection(normal)`, `euclidean::reflection(covector)`, or `spherical::reflection(normal)`, an orthogonal matrix. For a side of a double, the compensated product of two reflections of its generating domain (DOM-25). For a paired side of a wallpaper domain, the matrix of DOM-51, and of a spherical group domain, the matrix of DOM-59 |
| `Side::partner` | The index of the side onto which $P_j$ maps side $j$ | For a mirror, $j$ itself. For a paired side of a double, of a wallpaper domain, or of a spherical group domain, the other side of its pair (DOM-24, DOM-50, and DOM-58), never $j$ itself (DOM-26) |
| `Side::kind` | The kind of the pairing | `Mirror` for every side of a reflection triangle, of a polygon, of the rectangle, of a lune, and of the domain of `*`. `Translation` for sides 2 and 5 of the double of the rectangle and for every side of the double of a polygon other than sides 0, 1, $n - 1$, and $n$, and `Rotation` for every other side of a double (step 5 of Section 6.8). For a wallpaper domain, the kinds of the table of Section 6.13, which include `GlideReflection`. For a spherical group domain, the kinds of the table of DOM-58, which include `RotaryReflection` |
| `Side::orientation` | Whether $P_j$ preserves or reverses orientation | `Reversing` for a mirror, a glide reflection, and a rotary reflection, and `Preserving` for a rotation or a translation |
| `side_lengths()` | The length of each side, in side order | Section 7 |
| `corner_angles()` | The angle at each vertex in radians, in vertex order | Section 7. Empty for the whole sphere |
| `area()` | The area measured from the corner angles, or for a Euclidean domain from the vertices | Section 7. For a spherical domain, in the metric of the unit sphere, and $4\pi$ for the whole sphere |
| `reverses_orientation()` | Whether some side has the orientation `Reversing` | True for every reflection domain, for every wallpaper domain other than that of `o`, and for every spherical group domain other than the whole sphere, and false for every double, for `o`, and for the whole sphere of `1`. It is true exactly when the group has a parity colouring, the colouring by its orientation-preserving subgroup, and it is the one place that decides this for `escher-cli`. It does not say that every side is a mirror: a caller that needs the kinds reads `Side::kind` |
| `verify()` | The result of the six checks of Section 8.2 | Pure and never panics (DOM-8) |
| `side_presentation()` | The side presentation of Section 5.4 | Pure and never panics. Returns `InvalidPairing` for a domain whose sides give no presentation (DOM-35), which no domain that `build_with` returns does |
| `Verification::passed` | Whether every check passed | True exactly when each of the six `Check::passed` is true |
| `Check::passed` | Whether the residual is within the tolerance | `residual() <= tolerance()`. The residual is never NaN, and an infinite residual fails |
| `parameters` | The shape parameters of a signature, in a fixed order | Total. `aspect` for `*2222`, `**`, `xx`, `*x`, `22*`, `22x`, and `2*22`, `aspect` and then `skew` for `o` and `2222` (DOM-53), $n - 3$ for a polygon `*p1...pn` with four to eight corners, in the order of DOM-45, and an empty list for every other signature, supported or not, which includes `4*2`, `3*3`, every rotation signature other than `2222`, every spherical group signature, and every signature over a limit of Section 4 (Section 6.6) |
| `ParameterSpec` | The name, range, and default value of one shape parameter | `minimum() <= default_value() <= maximum()`, each finite. The name is the plain name of Section 5.6 of the architecture. The default of a length of a polygon depends on the signature (DOM-45) |
| `ShapeParameters` | The settings that a caller passes to `build_with` | Holds each name at most once. `new()` equals `Default::default()`, an empty set. `set` stores the name and the value that it is given and returns the value that it replaces, if any. It checks nothing, and `build_with` validates (Section 6.6) |
| `Error` | Why no domain was built | Section 10 |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

`Orientation` has both variants, because the renderer colours by the parity of the orientation of the pairings (Section 5.5 of `docs/design/render.md`), and the doubles of Section 6.8 have `Preserving` sides. It is a closed set, so it is exhaustive under a lint exception (API-9 of the coding standard). `PairingKind` grows with later classes, so it is `#[non_exhaustive]`.

`GlideReflection` is the one kind of an orientation-reversing pairing of a Euclidean wallpaper domain that is not a mirror, because in the Euclidean plane reflections and glide reflections are the only isometries that reverse orientation (Section 12 of the geometry reference, from a secondary source, and item 32 of its Section 15.1). A pairing of a Euclidean domain with another partner and the orientation `Reversing` is therefore a glide reflection. `RotaryReflection` is the one kind of an orientation-reversing pairing of a spherical domain that is not a mirror, which includes the antipodal map of `x`. That every orientation-reversing isometry of the sphere is a rotary reflection is item 33 of Section 15.1 of the geometry reference, which no source states in words. A glide reflection fixes no point and has an axis in the plane, while a rotary reflection of the sphere fixes the centre of the sphere, so the two names do not describe one isometry, and check 2 ties each kind to its plane (DOM-61). This is `DEC-57` of the architecture.

`PairingKind` has `Translation` beside `Rotation`, because the pairing of sides 2 and 5 of the double of the rectangle is the product of the reflections in two parallel walls, and the pairing of every side of the double of a polygon other than four of its sides is the product of the reflections in two walls that do not meet (step 5 of Section 6.8). Each fixes no point, so one kind for every orientation-preserving pairing would call a translation a rotation. This split is a proposed default of this design (Section 15.3).

The corner of a vertex, with its order and its copies, is a private type. No public accessor exposes it, because `corner_angles()` already gives what `escher-cli` prints, and the checks of Section 8.2 read it inside the crate.

The polygons and the wallpaper domains add no public signature. They need these private items:

- The private enum of the parameters has the variants `Aspect`, `Skew`, `CornerPair(k)`, and `Slice(i)`, the last two each with its number from 1 as a `u8`. It is `Copy` and `Eq`, and the name, the minimum, and the maximum derive from the variant.
- `ParameterSpec` holds the variant, its name as a `String`, and its default value as an `f64`, because the default of a length depends on the orders of the signature (DOM-45) and so is no constant of the variant. `ParameterSpec` derives `Clone`, `Debug`, and `PartialEq`, and is not `Copy`.
- The private shape of the supported classes includes the hyperbolic polygon, which stores its orders as a `[u32; 8]` with the count of the orders used, and the area. A fixed array keeps the shape `Copy` and bounds its size, which the limit of eight orders of DOM-38 allows.
- The private enum of the wallpaper signatures has the nine variants `Torus`, `Annulus`, `KleinBottle`, `MobiusStrip`, `TwoTwoMirror`, `TwoTwoCrossCap`, `TwoMirrorTwoTwo`, `FourMirrorTwo`, and `ThreeMirrorThree`, for `o`, `**`, `xx`, `*x`, `22*`, `22x`, `2*22`, `4*2`, and `3*3` in that order. It is `Copy` and `Eq`. The predicate of Section 4 maps a signature to a variant by matching its structure exactly, so that no other signature reaches the construction of Section 6.13 and no match on it needs a wildcard arm.
- The private enum of the spherical group signatures has the seven variants `ConeMirror(n)`, `ConeCrossCap(n)`, `TwoMirrorN(n)`, `ThreeMirrorTwo`, `Mirror`, `CrossCap`, and `Trivial`, for `n*`, `nx`, `2*n`, `3*2`, `*`, `x`, and `1` in that order, with $n$ as a `u32`. It is `Copy` and `Eq`. The predicate of Section 4 maps a signature to a variant by matching its structure exactly, as it does for the wallpaper signatures, so no other signature reaches the construction of Section 6.16 and no match on it needs a wildcard arm.
- The domain stores, in a private field, the private enum `Closure`, which says which rules checks 4 and 5 use. It has three variants:
  - `PerVertex`: check 4 applies the condition on the copies of DOM-55, and check 5 is per vertex. Every domain other than `2222`, the wallpaper domains, and the spherical group domains with a paired side has it, which includes the domains of `*` and `1`, and so do the domains that the unit tests build through private access.
  - `DoubleByCycles`: check 4 applies the condition on the copies, and check 5 uses the cycle rule of DOM-56. `2222` has it, at every value of `skew`.
  - `Cycles`: check 4 applies no condition on the copies, and check 5 uses the cycle rule. The wallpaper domains, the spherical kites, and the domains of `nx` and `x` have it. The name says the rule and not a class, because two classes share it.

The detail of `ParameterOutOfRange` carries the variant of the parameter, from which its name and its range derive, so `Error` keeps `Eq` (Section 10.2).

`reverses_orientation()` is a proposed default of this design. A caller could scan `sides()` for a `Reversing` side instead, and the accessor keeps that rule in one place for `escher-cli`. A caller must not read `reverses_orientation()` as "every side is a mirror". That reading is false for the eight wallpaper domains with a reversing side, of which `xx` and `22x` have no mirror at all, and for the spherical kites and the domains of `nx` and `x`, of which the last two have no mirror. A caller that decides between the forms of a domain reads the count of the sides of the kind `Mirror` instead. The command-line design states the summary of `escher info` in that way (CLI-51).

`Verification` has one accessor for each check and no enum of checks, so a caller needs no wildcard arm, and a later check is a new accessor.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, and the re-exports.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/domain.rs`: `FundamentalDomain`, `Side`, `PairingKind`, `Orientation`, the private enum `Closure` of Section 3.2, the accessors, and the measurements of Section 7. It also holds the one guard of DOM-44, DOM-54, and DOM-60, the private function `FundamentalDomain::guarded`, which `build_with` calls on every domain that it builds. A domain that fails gets the error of its class (Section 10.2), and every other domain is returned unchecked. The private function `FundamentalDomain::sound` returns `Numerical` when a stored value is not finite or the centre does not lie strictly inside every wall.
- `src/triangle.rs`: The supported-signature predicate of Section 4, the private function `admitted()`, and the construction of Section 6.1. The predicate returns a `Result`. Its `Ok` value is a private enum with the four variants `Reflection`, `Rotation`, `Wallpaper`, and `Sphere`. The first two are each over a private enum of the six shapes: the hyperbolic triangle, the hyperbolic polygon, the Euclidean triangle, the rectangle, the spherical triangle, and the lune. `Wallpaper` is over the private enum of the nine wallpaper signatures of Section 3.2, and `Sphere` over the private enum of the seven spherical group signatures and the area of the signature. So no match needs an arm for a class that cannot occur. Its error is `UnsupportedSignature`, or the `LimitExceeded` of DOM-38, which the predicate applies after its conditions because the fixed array of the polygon shape holds at most eight orders. `admitted()` runs the predicate and then the limit of DOM-3, or for `2*n` that of DOM-63, held as `LARGEST_TWO_MIRROR_ORDER`, half of `LARGEST_ORDER`, so it applies steps 1 and 2 of DOM-18 for both `parameters` and `build_with`. The generating domain of a rotation signature is built with half of its area, which is exact in IEEE 754 arithmetic.
- `src/double.rs`: The construction of the double of Section 6.8, shared by every rotation class. `double()` takes the `Closure` of Section 3.2 as an argument and stores it. `build_with` passes the value of the private function `double_closure` of `src/domain.rs`, an exhaustive match on the shape, so one place decides the closure of every double. The module also holds the compensated product of two reflections of DOM-25: an error-free product through `f64::mul_add`, an error-free sum, and a compensated dot product of three terms for each entry. The compensated product `product` is `pub(crate)`, so that `half_turn` of `src/wallpaper.rs` uses it.
- `src/polygon.rs`: The constructions of the Euclidean triangle and the rectangle of Section 6.5. Its `pub(crate)` function `outward_wall` gives the covector of `euclidean::wall_through` of two vertices, negated when the given centre lies on its positive side. The placement of Section 6.5 and the wallpaper constructions of `o` and the sheared `2222` share it.
- `src/sheets.rs`: The combinatorics and the lengths of the hyperbolic polygon of Section 6.10: the features, the cuts, the sheets, the gap function with its two branches, the side lengths, and the default shape of DOM-45. `Cuts::parameters` is the one ordered list of the parameters of a polygon: the corner pairs numbered from 1, then the slices numbered from 1, each with its feature. The index of a value is the position of its feature in that list (`Cuts::value_index`), and the side lengths, the names, the validation, and the defaults all read that list. The default of each feature comes from the inscribed polygon, whose radius the private function `inradius` finds by bisection.
- `src/walk.rs`: The boundary walk and the placement of the polygon of Section 6.10 in double-double arithmetic, with the double-double operations of DOM-42 and the assembly of the centre and the walls of steps 10 to 12, and the double-double evaluation of check 4 of DOM-49. It reuses `two_product` and `two_sum` of `src/double.rs`, which are `pub(crate)`.
- `src/spherical.rs`: The constructions of the spherical triangle and the lune of Section 6.7, and of the spherical group domains of Section 6.16. The lune takes its order as an argument and gives the order 1 the exact normals $(0, 0, -1)$ (step 2 of Section 6.16). The private function `rotary_lune` turns a lune into the domain of `nx` or `x` by DOM-59, and the private function `whole_sphere` builds the domain of `1`. The `pub(crate)` function `base` maps each spherical group to its base of DOM-58 as the private enum `Base`, with the variants `Kite { orders, pair }`, `RotaryLune(n)`, `MirrorLune`, and `WholeSphere`, on which `FundamentalDomain::spherical` of `src/domain.rs` matches with no wildcard arm. A kite runs the shared `on_kite` of `src/kite.rs` on its double and then step 8 of Section 6.16 through `FundamentalDomain::sound`. The module maps every error of `escher-geometry` to `Numerical` and holds no guard.
- `src/kite.rs`: The one builder of a kite with one pair of its sides turned into mirrors, the private function `on_kite`, which the wallpaper kites of Section 6.13 and the spherical kites of Section 6.16 share. It takes the double, which carries its geometry, and the mirror pair of the private enum `MirrorPair`, `Low` for sides 0 and 1 or `High` for sides 2 and 3. The rule of the corners of DOM-58 follows from the pair, so it is not an argument. It builds each mirror side with `Side::euclidean_mirror` or `Side::spherical_mirror` from the covector or the normal that the double stores, keeps the other two sides, the vertices, and the centre of the double bit for bit, and gives the closure `Cycles`. It refuses a double without four sides and four corners, a hyperbolic double, and, under the pair `Low`, a double whose corner 0 has an odd order, and its caller turns the refusal into `Numerical`.
- `src/wallpaper.rs`: The constructions of the nine wallpaper domains of Section 6.13 and the shear of `2222`, with the half-turn and the glide reflection of DOM-51 written as matrices with exact entries. It reuses the rectangle and `Side::euclidean_mirror` of `src/polygon.rs`, the double that `build_with` passes in, and `Side::mirror`. The private function `on_rectangle` is the one builder of `**`, `xx`, `*x`, `22*`, `22x`, and `2*22`. It takes the value of `aspect` and a layout, which is plain data: a `&'static [Planned]` with one entry for each side. Each entry holds:
  - The end point of the side, as `Point::Corner(i)`, vertex $i$ of the rectangle $F$, or `Point::Middle(i)`, the midpoint of side $i$ of $F$. The wall of the side is the wall of $F$ on which its end point lies.
  - Its pairing, as `Mirror`, `Translation` by a `Step`, `Glide` by a `Step`, or `HalfTurn`, where a `Step` is `Up`, `Right`, `Down`, or `Left`.
  - The cycle order of its end vertex.

  `on_rectangle` derives the partners from the walls: a mirror is its own partner, a translation or a glide reflection pairs with the side on the opposite wall, and a half-turn pairs with the other half of its split wall. The `pub(crate)` functions `half_turn` and `shear` build the half-turn of DOM-51 and the sheared `2222` of step 7 of Section 6.13. The kites come from `on_kite` of `src/kite.rs` with the mirror pair `Low`, on the double of `442` or `632` that `build_with` builds with the private helper `FundamentalDomain::doubled`. Every error of `escher-geometry`, and a kite without a double of four sides, gives `Numerical`, and every construction ends with `FundamentalDomain::sound`. The module holds no guard.
- `src/parameters.rs`: `parameters`, `ParameterSpec`, `ShapeParameters`, the private enum of the parameters with the name, the range, and the default of each, and the validation of Section 6.6, which checks the settings against the parameters of the signature and computes the defaults as DOM-45 states. The private function `validate` returns the value of every parameter of the generating reflection shape, from the settings or the defaults, and for `2222` the values of `aspect` and `skew` of the shear. `build_with` reads each value by its name and not by its position.
- `src/verify.rs`: `Verification`, `Check`, and the checks of Section 8, the translation test of DOM-27, which `src/double.rs` also calls to set the kind of each side of a Euclidean or spherical double, the choice of the arithmetic of check 4 (DOM-49), and the private helpers that check 4 and `side_presentation` share:
  - `side_kinds`, which classifies the sides of a domain as all mirrors, all paired, or mixed. A mixed domain is valid and takes the path of the paired domains (DOM-28 and DOM-52). A side of the kind `RotaryReflection` counts as paired, and a domain with no side counts as all mirrors (DOM-62).
  - Every `match` on `PairingKind` has an arm for each kind and no wildcard arm: the two in `pairings`, one for the agreement of the kind and one for the endpoint term (DOM-61), the one in `kind_agrees`, the one in `walls`, where only `Mirror` has a reflection term, and the one in `side_kinds`.
  - `vertices_are_points`, the precondition of checks 2 to 6, which accepts a spherical domain with no side, no vertex, and no corner, and refuses every other domain with no side (DOM-62).
  - `corner_pairs`, which gives each corner with its two sides and its order, the pairs of the corner relators, or nothing when the number of corners differs from the number of sides.
  - `vertex_cycles`, which walks every vertex cycle by the state walk of DOM-55, applies the structural conditions, and returns each cycle with its start state, its order, its states, its sides, and its transformation, or the defect that stopped it.
  - `framed_power_residual` and `power_residual`, which evaluate a relator as check 4 does, in the corner frame for a hyperbolic domain, with the power by successive multiplication and the residual divided by the exponent.
- `src/presentation.rs`: `side_presentation` of Section 5.4, which takes the kinds of side, the corner pairs, and the cycles from the helpers of `src/verify.rs`, so that check 4 and the relators come from one classification, one set of corner pairs, and one walk. Its one `match` on `PairingKind` treats `RotaryReflection` as a paired side.

Two names in the crate differ in meaning from the terms of Section 2.1. The module `src/polygon.rs` holds the Euclidean triangle and the rectangle, not the polygons of Section 6.10, which live in `src/sheets.rs` and `src/walk.rs`. The helper `corner_pairs` of `src/verify.rs` gives each corner with its two sides, not the corner pairs of Section 6.10. This design keeps both names.

Section 12 describes the files that the tests share.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, because the supported signatures do not need them:

- The kind and the unit of a `ParameterSpec`. `aspect` and `skew` are ratios, and every parameter of a polygon is a hyperbolic length at the curvature $-1$, so the name says the kind and no parameter needs a unit. The class that first needs them adds them as accessors, which does not break a caller. The crate sets `publish = false`, so a change affects only the workspace.
- Ideal vertices, which the triangle with an ideal corner adds.

`side_presentation()` returns a `Result`, as Section 3.2 of the architecture states. A domain whose walk does not close, or whose number of corners differs from its number of sides, has no side presentation of Section 5.4, and an infallible function would have to invent one or panic (DOM-35). This is a proposed default of this design (Section 15.3).

`build` and `build_with` are inherent constructors and return a `Result`. `build_with` is a secondary constructor in the sense of API-5 of the coding standard. `ShapeParameters` takes the place of the options struct of API-6, with a private map and the method `set` and not public fields, because the names of the parameters depend on the signature. Section 3.2 of the architecture lets the crate design refine the names and signatures.

`parameters` returns a `Vec` and `ParameterSpec::name` a `&str`, because later classes have parameter names that `escher-domain` derives from the signature, such as `slice1.length` (Section 5.6 of the architecture). `ShapeParameters` is a map, so it cannot hold a name twice (TYPE-3 of the coding standard), and the return value of `set` lets a caller detect a name set twice.

## 4. Supported Signatures

DOM-2. `FundamentalDomain::build` MUST construct a domain only for a signature that meets the predicate of this section, subject to DOM-3, and MUST return `UnsupportedSignature` for every other valid signature without a panic, because a caller that receives a domain for a signature outside the construction folds into a polygon that does not tile, and a panic stops the caller's whole program (ERR-5 of the coding standard).

A signature is supported exactly when it is a reflection or rotation signature by the conditions below, a wallpaper signature by the table that follows them, or a spherical group signature by the last table of this section. Each condition is read from the public accessors of `escher-signature` (Section 3.1 of the signature design). A reflection or rotation signature meets all of these:

1. `handles()` is 0.
2. `crosscaps()` is 0.
3. One of these holds:
   - Reflection signature: `cone_orders()` is empty, and `boundaries()` has exactly one element, whose `corner_orders()` are the orders.
   - Rotation signature: `boundaries()` is empty, and `cone_orders()` are the orders.
4. There are at least two orders.
5. `finite()` of each order is `Some`. This holds exactly when `is_compact()` is true for such a signature, and the crate reads `finite()` so that it needs no `expect` later.
6. One of these holds:
   - three orders, and `geometry()` is `Geometry::Hyperbolic`.
   - four or more orders, and `geometry()` is `Geometry::Hyperbolic`.
   - three orders, and `geometry()` is `Geometry::Euclidean`.
   - four orders, and `geometry()` is `Geometry::Euclidean`.
   - three orders, and `geometry()` is `Geometry::Spherical`.
   - two equal orders, and `geometry()` is `Geometry::Spherical`.

For a hyperbolic or spherical signature, the predicate also needs `area_over_pi()` to be `Some`, which it is for every such signature that meets conditions 1 to 6 (Section 9.4 of the signature design), so this condition makes no further signature unsupported. Step 1 of Section 6.1 converts the area. For a rotation signature, `area_over_pi()` is twice that of its starred signature, so `732` has the area $\pi/21$ and `*732` the area $\pi/42$.

The cone orders of a canonical rotation signature are in descending order, as the corner orders of a canonical reflection triangle are, so the orders of a rotation signature are the corner orders of its starred signature in the same order. With four or more orders, the starred signature is the star followed by the cone orders in descending order, which is a canonical signature (Section 15.1). The predicate reads `geometry()` of the rotation signature itself and relates no property of it to its starred signature. The supported rotation signatures are these:

| Orders | Geometry | Rotation signatures | Generating domain |
| --- | --- | --- | --- |
| 3 | Hyperbolic | `pqr` with $1/p + 1/q + 1/r < 1$ | The triangle of Section 6.1 |
| 4 to 8 | Hyperbolic | `p1...pn`, such as `3333`, `3222`, and `22222` | The polygon of Section 6.10 at its default shape (DOM-47) |
| 3 | Euclidean | `632`, `442`, and `333` | The triangle of Section 6.5 |
| 3 | Spherical | `332`, `432`, `532`, and `n22` with $n \ge 2$, which include `222` | The triangle of Section 6.7 |
| 4 | Euclidean | `2222` | The rectangle of Section 6.5 |
| 2, equal | Spherical | `nn` with $n \ge 2$, which include `22` | The lune of Section 6.7 |

`escher-signature` rejects `mn` with $m \ne n$ and a single cone point, such as `23` and `3`, as bad orbifolds, so no input reaches those cases.

A wallpaper signature is one whose `geometry()` is `Geometry::Euclidean` and whose structure equals one row of this table exactly. Every count and every list must match: `handles()`, `crosscaps()`, `cone_orders()`, the number of elements of `boundaries()`, and the `corner_orders()` of each boundary, with each order read through `finite()`:

| Signature | `handles()` | `crosscaps()` | `cone_orders()` | `boundaries()` | Typed also as |
| --- | --- | --- | --- | --- | --- |
| `o` | 1 | 0 | None | None | None |
| `**` | 0 | 0 | None | Two, each with no corner | None |
| `xx` | 0 | 2 | None | None | None |
| `*x` | 0 | 1 | None | One, with no corner | `x*` |
| `22*` | 0 | 0 | 2, 2 | One, with no corner | None |
| `22x` | 0 | 1 | 2, 2 | None | `x22`, `2x2` |
| `2*22` | 0 | 0 | 2 | One, with the corners 2, 2 | None |
| `4*2` | 0 | 0 | 4 | One, with the corner 2 | None |
| `3*3` | 0 | 0 | 3 | One, with the corner 3 | None |

The predicate maps a matching signature to the variant of the private enum of the nine wallpaper signatures of Section 3.2, and admits no signature that matches no row, as DOM-2 requires, because the construction of Section 6.13 builds exactly these nine structures, and a signature with another structure that reached it would get the domain of another group. The geometry condition is redundant for these nine rows, which are all Euclidean by the classification of `escher-signature`, and the predicate reads it so that a later change of the classification cannot admit a non-Euclidean signature. `*22 2`, with a space, is a parse error. The wallpaper class meets none of the conditions 1 to 6 above as a whole, so it is a third class beside the reflection and the rotation signatures, and the predicate tests it after them.

A spherical group signature is one whose `geometry()` is `Geometry::Spherical` and whose structure equals one row of this table exactly, with the same counts and lists as for a wallpaper signature. The predicate tests this fourth class last:

| Signature | `handles()` | `crosscaps()` | `cone_orders()` | `boundaries()` | Canonical examples | Typed also as |
| --- | --- | --- | --- | --- | --- | --- |
| `n*`, $n \ge 2$ | 0 | 0 | $n$ | One, with no corner | `2*`, `3*`, `(12)*` | `2*1` for `2*`, `3*1` for `3*` |
| `nx`, $n \ge 2$ | 0 | 1 | $n$ | None | `2x`, `3x`, `(12)x` | `x3` for `3x` |
| `2*n`, $n \ge 2$ | 0 | 0 | 2 | One, with the corner $n$ | `2*2`, `2*3`, `2*(12)` | None |
| `3*2` | 0 | 0 | 3 | One, with the corner 2 | `3*2` | None |
| `*` | 0 | 0 | None | One, with no corner | `*` | `1*`, `*1`, `1*1`, `(1)*` |
| `x` | 0 | 1 | None | None | `x` | `1x`, `x1` |
| `1` | 0 | 0 | None | None | `1` | None |

The predicate maps a matching signature to the variant of the private enum of the seven spherical group signatures of Section 3.2, and admits no signature that matches no row, for the reason that DOM-2 states for the wallpaper class. The geometry condition excludes the Euclidean and hyperbolic signatures of the same shape, such as `4*2`, `3*3`, `4*3`, `5*2`, `3*4`, `2x2`, and `*x`, and a cone order other than 2 or 3 before a boundary with one corner matches no row. `escher-signature` omits an order of 1 in the canonical form, so `2*1` gives `2*`, which is `n*` at $n = 2$, and the coincidences of the Appendix of [CH] (Section 9.6.2 of the domain reference) reach one construction each. `11` and `*11` are parse errors, and `*3`, `*32`, `23`, and `1*2` are bad orbifolds, so no input reaches those cases.

With this class, every valid spherical signature is supported: the 14 families of Table III of [CH] (Section 9.6 of the domain reference) are `nn`, `*nn`, `22n`, `*22n`, `332`, `*332`, `432`, `*432`, `532`, and `*532`, which the first two classes cover, and `n*`, `nx`, `2*n`, and `3*2`, which this class covers, with `*`, `x`, and `1` as the members at $n = 1$ of `n*`, `nx`, and `nn`, whose canonical forms omit the order 1. The spherical signatures of the first two classes, which include `*22`, `22`, `*222`, and `222`, keep their domains and the residuals of their checks bit for bit (DOM-30).

The geometry is the exact rational classification of `escher-signature` (Section 9.1 of the signature design), so no floating-point comparison decides support. For three finite corners, the hyperbolic case of condition 6 is the condition $1/p + 1/q + 1/r < 1$ of Section 9.1 of the domain reference and Section 10.1 of the geometry reference. The Euclidean case with three corners is exactly `*632`, `*442`, and `*333`, the three solutions of $1/p + 1/q + 1/r = 1$ in Example 6.5.2 of [DAVIS2008] and in Section 7.2 of [RAT], p. 279 (Section 9.1 of the domain reference). The Euclidean case with four corners is exactly `*2222`, the only signature of one boundary with four corners among the 17 Euclidean signatures of Section 9.5 of the domain reference. Those 17 signatures are the eight reflection and rotation signatures `*632`, `*442`, `*333`, `*2222`, `632`, `442`, `333`, and `2222`, and the nine wallpaper signatures. They are the 17 of Table II of [CH] and [TH] 13.3.6 (Section 9.5 of the domain reference), so every Euclidean signature without a frieze or a non-compact structure is supported. For a hyperbolic signature of one boundary with four or more finite corners, Example 6.5.3 of [DAVIS2008] gives a convex realization in the hyperbolic plane (Section 9.4 of the domain reference), so the hyperbolic case with four or more orders needs no further condition. The predicate reads `geometry()` and relies on no count of which such signatures are hyperbolic. The spherical case with three corners is exactly `*332`, `*432`, `*532`, and `*n22` with $n \ge 2$, the triples of Example 6.5.1 of [DAVIS2008] (Section 9.1 of the domain reference), which include `*222`. The spherical case with two corners is exactly `*nn` with $n \ge 2$, which include `*22`, because `escher-signature` rejects `*mn` with $m \ne n$ as a bad orbifold. The predicate also requires the two orders to be equal, because the lune of Section 6.7 stores one order for both corners, so a lune built from the first order of an unequal pair would pass its checks for a signature that it does not fit. No input reaches that case, so no test covers it.

Every other valid signature returns `UnsupportedSignature`. This covers the hyperbolic signatures that mix cone points with a boundary, such as `4*3` and `3*22`, every other signature with a handle, a cross-cap, or two or more boundaries, such as `oo` and `xxx`, the triangle with an ideal corner `*(inf)32`, the polygons with an ideal corner, such as `*(inf)222`, and their rotation signatures, such as `(inf)222`, the friezes, and every general non-compact signature. No valid spherical signature is among them. A general non-compact signature returns this kind throughout the proof of concept (`DEC-19`, Section 5.8 of the architecture).

DOM-3. `build` MUST return `LimitExceeded` for a supported signature with a corner order above 1000, and MUST apply the predicate of this section first, because the accuracy of Section 9.3 is established only up to that order, and a domain outside it could pass its checks by a smaller margin than the tolerances assume. A signature that is both unsupported and over the limit gets `UnsupportedSignature`, because the limit is a property of this construction. Section 16 states the reason for the number. The orders of a supported Euclidean signature are at most 6, so the limit does not bind for it. It binds for `*n22` and `*nn`, so `*(1001)22` and `*(1001)(1001)` return `LimitExceeded`. It applies in the same way to a rotation signature, so `(1001)32`, `(1001)22`, and `(1001)(1001)` return `LimitExceeded`, to a polygon and its rotation signature, so `*(1001)222` and `(1001)222` return `LimitExceeded`, and to `n*` and `nx`, so `(1001)*` and `(1001)x` return `LimitExceeded` with the text `order 1001 is larger than 1000`. The limit of `2*n` is that of DOM-63 in place of this one.

DOM-63. `build` MUST return `LimitExceeded` for `2*n` with $n$ above 500, with the text `order 501 is larger than 500` and the order in place of `501`, and MUST apply this limit in place of the limit of DOM-3 for `2*n`, after the predicate, because the base of `2*n` is the double of `*(2n)22` (DOM-58), whose corner of order $2n$ is within the limit of DOM-3 only for $n$ up to 500, so the accuracy of Section 9.5 is established only up to that $n$. The text of DOM-3 would be false for `2*n`: it would give `order 501 is larger than 1000` for `2*(501)`. So `2*(500)` builds, and `2*(501)` and `2*(1001)` return `LimitExceeded` with the text of this rule. A prototype found the base accurate past the limit of DOM-3, so 500 is the simplest limit and not a need of accuracy, because a limit of 1000 would need a new statement of accuracy for spherical triangles of an order up to 2000. It is `DEC-59` of the architecture, and item 48 of Section 15.3 puts it to the owner.

DOM-38. `build` MUST return `LimitExceeded` for a supported signature with more than eight orders, and MUST apply this limit after the predicate and before the limit of DOM-3, because the accuracy and the margins of Sections 8.4 and 9.7 are measured only for polygons with up to eight corners. A signature with nine or more orders and an order above 1000, such as `*(1001)22222222`, gets the text of this limit. The predicate admits it, so it gets this kind and not `UnsupportedSignature`, because its class has a construction and only the accuracy is not established. `*222222222` and `222222222` return `LimitExceeded`. The number 8 is a choice of this design (Section 15.3).

## 5. Sides, Vertices, and Pairings

### 5.1. Index Convention

The canonical form of `*pqr` lists its corners with $n_0 \ge n_1 \ge n_2$ (Section 7.2 of the signature design, step 4). So `*237` and `*327` both become `*732`, and vertex 0 has the smallest angle and vertex 2 the largest. For a signature with four or more corners, the canonical form is the lexicographically greatest of the cyclic rotations of the corner list and of its reverse (step 4 of Section 7.2 of the signature design), so `*2223` becomes `*3222`, `*2345` becomes `*5432`, and `*2233` and `*2332` both become `*3322`. The corners of a polygon are in this cyclic order, which is not in general descending.

DOM-4. The crate MUST number the vertices and sides of a reflection domain by this convention, those of a double by the convention of Section 6.8 (DOM-24), those of a wallpaper domain by the table of Section 6.13 (DOM-50), and those of a spherical group domain by the table of Section 6.16 (DOM-58), because `escher-cli` prints the angles in vertex order and the side lengths in side order, and the renderer records the side index of every fold step, so a second numbering gives two readings of one printed value:

- For a reflection domain, vertex $j$ is corner $j$, with the order $n_j$.
- Vertex $j$ is the common point of side $j$ and side $j + 1$, with indexes modulo $m$. So side $j$ joins vertex $j - 1$ and vertex $j$.
- For a reflection domain, corner $j$ lies between side $j$ and side $j + 1$, so its relator is $(P_j P_{j+1})^{n_j}$. The domain of `*` follows this rule with $n_j = 1$. The relators of a double, of a wallpaper domain, and of a spherical group domain with a paired side come from its vertex cycles (Section 8.2, check 4).

Section 9.1 of the notation reference reads a boundary with corners of orders $a$, $b$, and $c$ as the Latin reflections $P$, $Q$, $R$, and $S$ with the relators $(PQ)^a$, $(QR)^b$, and $(RS)^c$. Side 0 is $P$, side 1 is $Q$, and side 2 is $R$. For a triangle the convention reads $S$ as $P$ again, and for the rectangle side 3 is $S$. A lune has $m = 2$, so side 0 joins vertex 1 and vertex 0, side 1 joins vertex 0 and vertex 1, and its relators are $(P_0 P_1)^{n}$ at corner 0 and $(P_1 P_0)^{n}$ at corner 1. Section 15.1 records this reading as an assumption. The relators that the checks use come from Section 4.4 of the domain reference, which has three reflections and does not depend on it.

### 5.2. Orientation and Outward Normals

The vertices of a hyperbolic domain are counterclockwise in the Klein model: with $k(V)$ the first two coordinates of $V$ divided by its third, the planar cross product of $k(V_1) - k(V_0)$ and $k(V_2) - k(V_0)$ is positive. The Klein model draws every wall as a straight chord, so the sign of this product is the orientation of a convex domain. The same product in the Poincare disk, of `hyperbolic::to_disk(V1) - hyperbolic::to_disk(V0)` and `hyperbolic::to_disk(V2) - hyperbolic::to_disk(V0)`, is not, because the disk draws the walls as arcs: for the double of `8765432`, which is convex and counterclockwise, it is negative. The disk form holds on the triangles and kites. The vertices of a Euclidean domain are counterclockwise in the plane: the same cross product of the first two coordinates of $V_1 - V_0$ and $V_2 - V_0$ is positive. The vertices of a spherical triangle are counterclockwise as seen from outside the sphere: $(V_0 \times V_1) \cdot V_2$ is positive. The two vertices of a lune do not fix an orientation, and its placement in Section 6.7 numbers its sides counterclockwise as seen from outside the sphere at its centre.

A double with at least three vertices meets the same test of its geometry on its vertices $W_0$, $W_1$, and $W_2$ of Section 6.8, with a positive result. Each wallpaper domain, and `2222` at every value of `skew`, has its vertices counterclockwise in the plane, which holds for the cross product of every three consecutive vertices that are not on one line. The double of a lune has the orientation of its generating lune.

The normal of each side points outward, as Section 5.2 of the architecture requires. A point $x$ is inside the domain where `hyperbolic::side(covector, x)`, `euclidean::side(covector, x)`, or `spherical::side(covector, x)` is at most 0 for every side, and a positive value means that the point is outside that side. Section 2.4 of the geometry reference records that [RAT] orients normals inward. This crate uses no statement of [RAT] that depends on that orientation.

### 5.3. Pairings

Side $j$ carries the isometry $P_j$ that maps the neighbouring copy of the domain across side $j$ onto the domain (Section 5.3 of the architecture). For a mirror side, $P_j$ is the reflection in its wall, `hyperbolic::reflection(normal)`, which is the reflection of equation (6.6) of [DAVIS] (Section 7.4 of the geometry reference) as Section 5.2 of the architecture adopts it. For a Euclidean mirror side, $P_j$ is `euclidean::reflection(covector)`, the reflection of equation (4.1.1) of [RAT] and equation 6.3 of [DAVIS] (Sections 7.4 and 12 of the geometry reference), whose form as a matrix is part of `UNV-26` (Section 14). For a spherical mirror side, $P_j$ is `spherical::reflection(normal)`, the reflection of equation (6.4) of [DAVIS] (Section 7.4 of the geometry reference). The partner of a mirror side is the side itself.

A side of a double carries the product of two reflections of its generating domain, which preserves orientation, and its partner is another side of the double (Section 6.8). The pairing of the partner is the inverse, as item 1 of Section 4.8 of the domain reference and Condition 3.2 (d) of [EPSTEIN-PETRONIO1994] require.

A paired side of a wallpaper domain carries a translation, a half-turn, a rotation, or a glide reflection, built as DOM-51 states, and its partner is another side. A glide reflection reverses orientation and maps side $i$ onto its partner $p$ with the endpoints in the same order: $P_i V_{i-1} = V_{p-1}$ and $P_i V_i = V_p$. A preserving pairing exchanges them: $P_i V_{i-1} = V_p$ and $P_i V_i = V_{p-1}$. A mirror is the reversing case with $p = i$, so its endpoint terms are the fixed points of its reflection. Check 2 tests each form (Section 8.2). The pairing of the partner is again the inverse.

A paired side of a spherical group domain carries a rotation, a half-turn, or a rotary reflection, built as DOM-59 states, and its partner is another side. A rotary reflection reverses orientation and maps side $i$ onto its partner with the endpoints in the same order, as a glide reflection does: on the lune of `nx`, $P_0 V_1 = V_0$ and $P_0 V_0 = V_1$. Check 2 tests it with the term of a glide reflection (DOM-61).

### 5.4. Side Presentation

`side_presentation()` reads a presentation of the group from the sides of the domain, as item 7 of Section 4.8 of the domain reference states it: the pairing relators together with the cycle relators. It returns a `Presentation` of `escher-group`, whose words, relators, text, and validation Sections 4.2 to 4.4 of the group design specify (GRP-3 to GRP-5). The command `escher info --presentation` prints it and enumerates its cosets (`docs/design/cli.md`), and `colour_of` of `escher-group` reads the side sequence of the fold as a word in its generators (GRP-38 of the group design).

DOM-32. `side_presentation()` MUST give the presentation one generator for each side, in side order, named `s` followed by the side index in decimal from 0, such as `s0`, and the generator of side $i$ MUST stand for $P_i^{-1}$, the element of Section 5.3 of the architecture whose image of the domain is the neighbouring copy across side $i$, with a word read left to right as the product left to right, because the renderer records the fold as a sequence of side indexes, and a colouring that reads that sequence as a word needs the side index, the generator, and the order of the product to agree. For a mirror, $P_i^{-1} = P_i$.

DOM-33. For a domain whose every side is a mirror, the relators MUST be $s_j^2$ for each side $j$ in side order, followed by $(s_j s_{j+1})^{n_j}$ for each corner $j$ in corner order, with indexes modulo $m$ and $n_j$ the stored order of corner $j$, because these are the Coxeter relators of the reflection polygon (Theorem 6.4.3 of [DAVIS2008], Section 4.4 of [MASKIT2001], and Proposition 13.3.1(iii) of [TH], in Sections 4.4 and 4.5 of the domain reference and Section 9.4 of the notation reference), and a fixed order makes the text of `escher info` reproducible. Corner $j$ lies between side $j$ and side $j + 1$ (Section 5.1), so a lune gives $(s_0 s_1)^n$ at corner 0 and $(s_1 s_0)^n$ at corner 1, on `UNV-29`. The domain of `*`, the lune of order 1 with two mirror sides, follows this rule with $n_j = 1$, so its corner relators have the exponent 1 and print as `s0 s1` and `s1 s0` (Section 4.4 of the group design).

DOM-34. For a domain with no mirror side, the relators MUST be the pair relator $s_i s_p$ for each side $i$ whose partner $p$ is larger than $i$, in increasing order of $i$, followed by one cycle relator for each vertex cycle, in the order in which the walk of DOM-55 starts the cycles. A glide reflection changes nothing in this rule. The base of a cycle relator MUST be the generators of the sides $s$ that the walk applies, in the order in which it applies them, and its exponent the order $m_c$ of the cycle, because the walk is the one that check 4 evaluates, so the relators and the check describe the same cycles, and the presentation is the pairing relators together with the cycle relators (Theorem 12 (3) of [SOLOMYAK2020], Theorem 6.14 (iii) of [SERIES2013], and Theorem 4.14 of [EPSTEIN-PETRONIO1994], in Sections 4.1, 4.2, 4.8, and 4.9 of the domain reference). The pair relator writes $P_p = P_i^{-1}$ (Section 5.3) as a word.

No source read states the cycle relators as words in this convention. That the walk order of the base, with the generator of side $s$ for $P_s^{-1}$, gives the relator of the cycle is the reading of this design, with SymmHub's `groupPresentation` as precedent in code (Section 14.2 of `docs/research/symmhub.md`). It is `UNV-32` (Section 14). A reversed base, and the opposite convention in which the generator of side $i$ stands for $P_i$, present isomorphic groups, so no order or index distinguishes them. Only a numeric evaluation of the words does, and DOM-36 states which test establishes what.

DOM-52. For a domain with both a mirror side and a paired side, the relators MUST be, for each side $i$ in increasing order, $s_i^2$ when side $i$ is a mirror and the pair relator $s_i s_p$ when its partner $p$ is larger than $i$, followed by one cycle relator for each vertex cycle of the walk of DOM-55, in the order in which the walk starts the cycles, with the generators of the sides that the walk applies, in the order in which it applies them, as its base, and the cycle order as its exponent. The reason is that these are the pairing relators, with $P_i^2 = 1$ for a mirror, together with the cycle relators, as for DOM-33 and DOM-34, and Theorem 4.14 of [EPSTEIN-PETRONIO1994] puts no condition on the orientation of a pairing (Section 4.9 of the domain reference), while one walk for check 4 and the relators keeps the two in agreement. For a domain without a mirror, this rule gives the relators of DOM-34. The walk visits a mirror in both directions, so the letter of a mirror can occur twice in one base, as in `s1 s0 s1 s2` of `**`.

The bases of the chains, the cycles that pass a mirror, rest on `UNV-32` as the bases of DOM-34 do (Section 14). No source read states them as words.

DOM-35. `side_presentation()` MUST return an `Error` of the kind `InvalidPairing`, and MUST NOT panic, for a domain whose number of corners differs from its number of sides, for a domain with more than 256 sides, whose side indexes above 255 have no letter of `escher-group` (Section 4.1 of the group design), for a walk of DOM-55 that does not return to its start state within $2m$ steps, for a cycle whose vertices differ in order, for a double whose cycle copies do not sum to 2, or when `Relator::new` or `Presentation::new` of `escher-group` rejects a relator or the presentation (Sections 4.2 and 4.3 of the group design), because such a domain has no side presentation of this section, and `escher info` prints the result for any signature that it reads. The conditions on the walk and on the cycles are the structural conditions of check 4. No domain that `build_with` returns gives this error, and the unit tests reach it through private access (Section 12.1). A domain with both mirror and paired sides is no cause: such a domain has the presentation of DOM-52.

The function tests these conditions in the order of DOM-35 and returns the first failure: the number of corners, the number of sides, and then, for a domain that is not all mirrors, the walk of the cycles, which stops at the first cycle whose walk does not close or whose orders or copies are unequal, before `escher-group` sees any relator. Each of these causes has its own text (Section 10.2). So a partner index out of range gives the text of a walk that does not close, and not that of a rejected presentation. A domain with no side counts as a reflection domain and gives a presentation with no generator and no relator, which `Presentation::new` accepts (Section 4.3 of the group design). The whole sphere of `1` is the one domain that `build_with` returns with no side, and this is its side presentation (DOM-62).

The side presentation keeps a generator for each side of a pair, with the pair relator, as SymmHub does, so the generator of a side is found by the side index alone. Section 15.3 records the alternative of one generator for each pair (`DEC-36`).

DOM-37. The crate MUST keep the numbering of the sides of every domain that `build_with` returns, as DOM-4 and DOM-24 state it, and a change that renumbers the sides of a built domain MUST be treated as a change of the public contract of the crate, because the id of a colouring ranks its coset table among the tables of the same colour count (GRP-37 of the group design), the columns of a table follow the generators of the side presentation in side order (DOM-32), so a renumbering renames ids without an error, and a user who rendered `--colouring 3a` then gets another picture under the same id. The golden images of `escher-render` and the snapshots of `escher-cli` that name an id fix the numbering, so a renumbering fails them. The numbering of a polygon and of its double depends only on the canonical signature and never on the values of the shape parameters (DOM-43), so a setting of `--set` keeps every colouring id. The numbering of a wallpaper domain is that of the table of Section 6.13, and it depends only on the canonical signature and never on `aspect` or `skew` (DOM-50 and DOM-53). `2222` keeps the numbering of Section 6.8 at every value of `skew`, so its ids do not change. The numbering of a spherical group domain is that of the table of DOM-58, and the spherical kites keep the numbering of their doubles. `DEC-50` of the architecture records that a later construction of `p1...pn` with parameters of its own would renumber the sides of its double, which this rule then treats as a change of the contract.

These are the side presentations of six domains, in the text of Section 4.4 of the group design:

| Signature | Generators | Relators |
| --- | --- | --- |
| `*532` | `s0`, `s1`, `s2` | `s0^2, s1^2, s2^2, (s0 s1)^5, (s1 s2)^3, (s2 s0)^2` |
| `*3333` | `s0` to `s3` | `s0^2, s1^2, s2^2, s3^2, (s0 s1)^3, (s1 s2)^3, (s2 s3)^3, (s3 s0)^3` |
| `3333` | `s0` to `s5` | `s0 s1, s2 s5, s3 s4, s1^3, (s2 s0)^3, (s3 s5)^3, s4^3` |
| `532` | `s0` to `s3` | `s0 s1, s2 s3, s1^5, (s2 s0)^2, s3^3` |
| `2222` | `s0` to `s5` | `s0 s1, s2 s5, s3 s4, s1^2, (s2 s0)^2, (s3 s5)^2, s4^2` |
| `22` | `s0`, `s1` | `s0 s1, s1^2, s0^2` |

These are the side presentations of the ten domains of Section 6.13, which do not change with the settings:

| Signature | Generators | Relators |
| --- | --- | --- |
| `o` | `s0` to `s3` | `s0 s2, s1 s3, s1 s0 s3 s2` |
| `**` | `s0` to `s3` | `s0 s2, s1^2, s3^2, s1 s0 s1 s2, s3 s2 s3 s0` |
| `xx` | `s0` to `s3` | `s0 s2, s1 s3, s1 s0 s1 s2` |
| `*x` | `s0` to `s3` | `s0 s2, s1^2, s3^2, s1 s0 s3 s2, s2 s3 s0 s1` |
| `22x` | `s0` to `s3` | `s0 s2, s1 s3, (s1 s2)^2, (s2 s3)^2` |
| `22*` | `s0` to `s5` | `s0 s1, s2^2, s3 s4, s5^2, s1^2, s2 s1 s5 s0, s3 s5 s4 s2, s4^2` |
| `2*22` | `s0` to `s4` | `s0 s1, s2^2, s3^2, s4^2, s1^2, s2 s1 s4 s0, (s3 s2)^2, (s4 s3)^2` |
| `4*2` | `s0` to `s3` | `s0^2, s1^2, s2 s3, (s1 s0)^2, s2 s0 s3 s1, s3^4` |
| `3*3` | `s0` to `s3` | `s0^2, s1^2, s2 s3, (s1 s0)^3, s2 s0 s3 s1, s3^3` |
| `2222` | `s0` to `s5` | `s0 s1, s2 s5, s3 s4, s1^2, (s2 s0)^2, (s3 s5)^2, s4^2`, at every value of `skew` |

These are the side presentations of the spherical group domains of Section 6.16. For $n$ from 2 up to the limits, the text of each of `2*n`, `n*`, and `nx` changes with $n$ only in the exponent that its row names. At $n = 1$, `1*` and `1x` are the signatures `*` and `x`, which have rows and texts of their own, and `2*1` is `2*`:

| Signature | Generators | Relators |
| --- | --- | --- |
| `3*2` | `s0` to `s3` | `s0^2, s1^2, s2 s3, (s1 s0)^2, s2 s0 s3 s1, s3^3` |
| `2*3`, for `2*n` with $n$ as the exponent of `(s1 s0)` | `s0` to `s3` | `s0^2, s1^2, s2 s3, (s1 s0)^3, s2 s0 s3 s1, s3^2` |
| `3*`, for `n*` with $n$ as the exponent of `s1` | `s0` to `s3` | `s0 s1, s2^2, s3^2, s1^3, s2 s1 s3 s0, s3 s2` |
| `2*` | `s0` to `s3` | `s0 s1, s2^2, s3^2, s1^2, s2 s1 s3 s0, s3 s2` |
| `3x`, for `nx` with $n$ as the exponent of `(s1 s1)` | `s0`, `s1` | `s0 s1, (s1 s1)^3` |
| `x` | `s0`, `s1` | `s0 s1, s1 s1` |
| `*` | `s0`, `s1` | `s0^2, s1^2, s0 s1, s1 s0` |
| `1` | None | None |

The spherical kites follow DOM-52, the domains of `nx` and `x` follow DOM-34, `*` follows DOM-33, and `1` has no generator and no relator. The relator `s3 s2` of `n*` is the cycle relator of the flat corner between its two mirror sides, with the order 1, and `s1 s1` of `x` is the cycle relator of its one cycle, with the order 1.

The construction gives no relator that is the empty word, and it reduces no relator: a relator of the walk with one letter, such as `s1^5`, stays as it is.

Every vertex cycle of every reflection domain and double of Section 4 visits one or two sides, as the table of Section 6.8 shows, and a pair relator has two letters. On a prototype, the reversed bases and the opposite convention gave the same residuals as the convention of DOM-32 for every built double (Section 14), so no double distinguishes them, and the tests use a hexagon built through private access, whose vertex cycle visits three sides (Section 12.1). The wallpaper domains have cycles that apply four sides, such as the one cycle of `o` and the cycles 1 and 2 of `22*`. Whether the reversed base or the opposite convention gives a residual above the tolerance on them is not known, and the tests assert only the reading of DOM-32 on cycle 1 of `22*`.

## 6. Construction

### 6.1. Algorithm

DOM-5. `build_with` MUST construct a hyperbolic reflection triangle by the steps of this section, a polygon by the steps of Section 6.10, a Euclidean domain by the steps of Section 6.5, a spherical one by the steps of Section 6.7, the domain of a rotation signature by the steps of Section 6.8, a wallpaper domain and the shear of `2222` by the steps of Section 6.13, and a spherical group domain by the steps of Section 6.16, each in its order, because the accuracy of Sections 9.3 to 9.9 and the margins of Section 8.4 are measured for exactly these steps, and another order of the same operations gives other rounding errors that no test measures.

Let $n_0 \ge n_1 \ge n_2$ be the corner orders of Section 5.1 and $\alpha_j = \pi / n_j$:

1. Return `UnsupportedSignature` unless the signature is supported (Section 4). Read the corner orders $n_j$, and compute the area of the signature as `PI` times the numerator of `area_over_pi()` divided by its denominator. Convert each of the numerator and the denominator to `f64` by splitting the integer into its upper and lower 32 bits and adding the two parts, which rounds once to nearest over the whole range of the integer type, with no limit on its size and no `as` cast.
2. Return `LimitExceeded` when $n_0 > 1000$ (DOM-3).
3. Compute each $\alpha_j$ as `PI / f64::from(n_j)`, and its sine and cosine.
4. Compute two side lengths by the law of cosines solved for a side:
   - $X_a = \dfrac{\cos\alpha_1 + \cos\alpha_2 \cos\alpha_0}{\sin\alpha_2 \sin\alpha_0}$, the hyperbolic cosine of the side from vertex 2 to vertex 0, which is side 0 and lies opposite vertex 1.
   - $X_b = \dfrac{\cos\alpha_0 + \cos\alpha_2 \cos\alpha_1}{\sin\alpha_2 \sin\alpha_1}$, the hyperbolic cosine of the side from vertex 1 to vertex 2, which is side 2 and lies opposite vertex 0.
   - $L_a = \operatorname{arcosh} X_a$ and $L_b = \operatorname{arcosh} X_b$, each computed with `f64::acosh`. Side 1, from vertex 0 to vertex 1, is never set. It follows from the placement of step 5.
5. Place the triangle in the standard position:
   - $W_2 =$ `hyperbolic::ORIGIN`.
   - $W_0 =$ `hyperbolic::boost(L_a) * hyperbolic::ORIGIN`.
   - $W_1 =$ `hyperbolic::rotation(alpha_2) * (hyperbolic::boost(L_b) * hyperbolic::ORIGIN)`.
6. Compute $c =$ `hyperbolic::normalize_point(W_0 + W_1 + W_2)`.
7. Compute $M =$ `hyperbolic::inverse(hyperbolic::translation(c))`.
8. Compute each vertex $V_j = M W_j$. Do not renormalize the result (Section 9.1).
9. For each side $j$ from 0 to 2, with vertex indexes modulo 3, compute $u =$ `hyperbolic::wall_through(V_{j-1}, V_j)`. The caller orients $u$, as the geometry design states, so the order of the two arguments does not matter. If `hyperbolic::side(hyperbolic::covector(u), V_{j+1})` is positive, negate $u$. Vertex $V_{j+1}$ is the vertex of the triangle that is not on side $j$. Side $j$ gets the normal $u$, the covector `hyperbolic::covector(u)`, the pairing `hyperbolic::reflection(u)`, the partner $j$, the kind `Mirror`, and the orientation `Reversing`.
10. Compute the centre as `hyperbolic::normalize_point(V_0 + V_1 + V_2)`.
11. Return `Numerical` if a call of step 6, 9, or 10 returned an error, if any stored number is not finite, or if the value of `hyperbolic::side` at the opposite vertex in step 9 is not strictly negative after the orientation.

The domain stores the geometry of the signature, the corner orders and the area of step 1, each corner with the copies 1, and the sides, vertices, and centre of steps 8 to 10. It does not keep the signature, because `build` borrows it and the checks of Section 8.2 need only these values.

The rotation of step 5 is counterclockwise by $\alpha_2$ about the origin, as `hyperbolic::rotation` defines it, so vertex 1 lies counterclockwise from vertex 0 as seen from vertex 2, which gives the orientation of Section 5.2.

DOM-6. The construction and the measurements MUST NOT use a relation between the inner product of two normals and the corner angle, except for the corner angle of a domain with two vertices, a lune or its double, as DOM-20 states, because the only such relation in the research is stated with inward normals in the conventions of [RAT] (Section 7.4 of the geometry reference), and its form with the outward normals of this project is part of `UNV-26`. The corner angles are measured from the distances between the vertices instead (Section 7).

### 6.2. Source of Each Step

| Step | Statement used | Source | Convention |
| --- | --- | --- | --- |
| 1 | The hyperbolic triangle exists when the angles $\pi/n_j$ sum to less than $\pi$ | Section 9.1 of the domain reference, Example 6.5.3 of [DAVIS2008]. Section 10.1 of the geometry reference, Fact 3 of [CFKP] | None involved |
| 3 | A corner of order $n$ has the angle $\pi/n$ | Section 4.5 of the domain reference, the hypotheses of Theorem 6.4.3 of [DAVIS2008]. Item 4 of Section 4.8 of the domain reference | None involved |
| 4 | The law of cosines solved for a side, $\cosh c = (\cos\alpha\cos\beta + \cos\gamma)/(\sin\alpha\sin\beta)$ with $c$ opposite $\gamma$ | Section 10.2 of the geometry reference: [BEA] Section 7.12, p. 148, and [RAT] Theorem 3.5.4, p. 86 | Intrinsic, curvature $-1$ (Section 2.2 of the geometry reference). The two lengths permute the angles of the printed form |
| 4, 5 | The triangle is unique up to congruence, given its angles | Section 10.1 of the geometry reference, Fact 3 of [CFKP]. Section 9.1 of the domain reference, Example 6.5.3 of [DAVIS2008] | None involved |
| 5 | The boost, the rotation, and the origin | `hyperbolic::boost`, `hyperbolic::rotation`, and `hyperbolic::ORIGIN` of the geometry design, from p. 3 of [KC24] (Section 6.1 of the geometry reference) | Time last, full distance |
| 6, 10 | The centre is the normalized sum of the finite vertices | `DEC-12`, a choice of the architecture. `hyperbolic::normalize_point` of the geometry design | Time last |
| 7 | An isometry that maps the origin to $c$, and its inverse | `hyperbolic::translation` and `hyperbolic::inverse` of the geometry design | Time last |
| 9 | The wall through two points is the unit spacelike vector orthogonal to both | The property: equation (6.6) of [DAVIS] (Section 7.4 of the geometry reference). The formula of `hyperbolic::wall_through` rests on `UNV-26` (Section 14) | Time last in the geometry design |
| 9 | The normal points away from the opposite vertex | Section 5.2 of the architecture | Project convention |
| 9 | The pairing of a mirror is the reflection in its wall | Equation (6.6) of [DAVIS] (Section 7.4 of the geometry reference), and Sections 5.2 and 5.3 of the architecture | `hyperbolic::reflection` of the geometry design |

No step uses a relation between the inner product of two normals and the corner angle (DOM-6).

### 6.3. Placement

The construction places the triangle twice. Both parts are choices of this design:

- The standard position puts vertex 2, the corner of the smallest order and the largest angle, at the origin, and vertex 0 on the positive first axis. The two walls through vertex 2 then pass through the origin.
- The final position moves the centre of `DEC-12` to the origin. The owner decided on one canonical view centred on the domain (Section 10.1 of the architecture). With this placement, the canonical view of the disk is the plain disk map. `centre()` stays public, so `View` in `escher-render` keeps the freedom that Section 3.2 of the architecture gives it.

The double of a rotation signature leaves its generating domain where this section places it, and its centre is the centre of the generating domain. The owner decided that the canonical view of a rotation signature is the view of its starred signature, centred on the generating domain, so that `732` and `*732` show the same tiling in the same place (Section 10.1 of the architecture). The centre of the generating domain lies strictly inside the double, which check 2 tests on every build.

The final position also keeps the walls close to the origin. The entries of `hyperbolic::reflection(u)` grow with the products of the components of $u$, and the renderer applies such a matrix at every fold step. A double-precision model outside the repository found larger normal components for the triangles it tried with vertex 2 left at the origin, and failed checks at large equal orders.

The placement differs from that of SymmHub, which keeps one corner at the origin (Section 7.6 of the geometry reference). The design does not use SymmHub's hyperbolic triangle as a test oracle, so the hyperbolic construction does not rely on `UNV-28`.

### 6.4. Theorems That Make the Triangle a Fundamental Domain

These sourced theorems make the constructed triangle a fundamental domain of the group that its reflections generate. This document adds no proof (Hard Rule 14 of `AGENTS.md`):

| Statement | Source | Hypotheses |
| --- | --- | --- |
| The reflections in the faces generate a discrete group, isomorphic to the Coxeter group, and the polytope is a strict fundamental domain for it | Theorem 6.4.3 of [DAVIS2008], p. 83 (Section 4.5 of the domain reference) | A simple convex polytope in $H^n$, $n \ge 2$, whose dihedral angles are $\pi/m_{ij}$ with integers $m_{ij}$ where two faces meet. The domain reference applies it to compact polygons only |
| The group of a triangle has the presentation $\langle r_1, r_2, r_3 : r_i^2 = (r_i r_j)^{m} = 1 \rangle$ with the orders of the corners | Section 4.4 of [MASKIT2001], p. 276 (Section 4.4 of the domain reference) | A Poincare triangle. The relator has its usual meaning at an elliptic vertex |
| A hyperbolic triangle with angles $\pi/m_i$ and $\sum 1/m_i < 1$ exists, is unique up to congruence, and gives a discrete reflection group | Example 6.5.3 of [DAVIS2008] (Section 9.1 of the domain reference). Fact 3 of [CFKP] (Section 10.1 of the geometry reference) | Integers $m_i \ge 2$ |
| For one polygon whose pairings are all reflections, the only condition to check is that the angles have the form $\pi/m$ | [EPSTEIN-PETRONIO1994], pp. 161 and 166, stated as remarks and not as a numbered theorem (Section 4.9 of the domain reference) | One convex polygon, all face pairings reflections |
| The area of a triangle is $\pi$ minus its angle sum | Fact 3 of [CFKP] and Theorem 7.13.1 of [BEA] (Section 10.1 of the geometry reference) | Curvature $-1$ |
| The area of a hyperbolic orbifold is $-2\pi\chi$ | [TH] 13.3.5, p. 312 (Section 7.3 of the notation reference) | A hyperbolic orbifold |

The checklist of Section 4.8 of the domain reference collects these conditions for one convex polygon. `verify()` evaluates the hypotheses that a computed domain can miss, and leaves the others to the construction:

| Hypothesis | How the design meets it |
| --- | --- |
| Each side is paired, possibly with itself, and the partner's pairing is the inverse (item 1 of Section 4.8, Condition 3.2 (a) and (d) of [EPSTEIN-PETRONIO1994]) | Checked: check 2 |
| A pairing maps the interior to the other side of the partner side (Condition 3.2 (b)), and maps the side onto its partner, which for a mirror fixes its endpoints (Condition 3.2 (c)) | Checked: check 2 |
| Each pairing is an isometry | Checked: check 1 |
| The pairing of a mirror is the reflection in the wall of its side | Checked: check 3 |
| Every corner has the angle $\pi/m$, and the relator $(r_i r_j)^m$ holds (item 4 of Section 4.8) | Checked: checks 4 and 5 |
| The polygon is a triangle, simple and convex | By construction. Check 2 confirms that the centre is inside every side |
| The area agrees with the Euler characteristic | Checked for consistency only: check 6 (Section 8.2) |

The domain reference records the source [CFKP] as available but not read in its Section 13.4, while the geometry reference cites it as primary. The existence and uniqueness of the triangle therefore also rest on Example 6.5.3 of [DAVIS2008], which the domain reference read.

### 6.5. Euclidean Construction

For a Euclidean signature, `build_with` runs steps 1 to 5 of DOM-18 and then these steps. Let $n_0 \ge n_1 \ge n_2$ be the corner orders of a triangle and $\alpha_j = \pi / n_j$, and let $a$ be the validated value of `aspect` for the rectangle:

1. For a triangle, compute each $\alpha_j$ as `PI / f64::from(n_j)`, and $l = \sin\alpha_0 / \sin\alpha_1$.
2. Place the polygon in the standard position:
   - For a triangle: $W_2 =$ `euclidean::ORIGIN`, $W_0 = (1, 0, 1)$, and $W_1 =$ `euclidean::rotation(alpha_2) * (l, 0, 1)`.
   - For the rectangle: $w = \sqrt{a}$ and $h = 1 / w$, and $W_0 = (w/2, -h/2, 1)$, $W_1 = (w/2, h/2, 1)$, $W_2 = (-w/2, h/2, 1)$, and $W_3 = (-w/2, -h/2, 1)$.
3. Compute $s = W_0 + \dots + W_{m-1}$ and the mean $g = s / s_3$.
4. Compute each vertex $V_j =$ `euclidean::translation((-g_1, -g_2, 1)) * W_j`.
5. Compute the centre as the sum of the $V_j$ divided by its last coordinate.
6. For each side $j$ from 0 to $m - 1$, with vertex indexes modulo $m$, compute $c =$ `euclidean::wall_through(V_{j-1}, V_j)`. If `euclidean::side(c, centre)` is positive, negate $c$. Side $j$ gets the normal $(c_1, c_2, 0)$, the covector $c$, the pairing `euclidean::reflection(c)`, the partner $j$, the kind `Mirror`, and the orientation `Reversing`.
7. Return `Numerical` if a call of step 6 returned an error, if any stored number is not finite, or if the value of `euclidean::side` at the centre is not strictly negative for every side after the orientation.

The domain stores the geometry and the corner orders of the signature, and the sides, vertices, and centre of steps 4 to 6. It stores no area of the signature, because check 6 does not read one for a Euclidean domain (Section 8.2). Steps 3 to 7 are steps 6 to 11 of Section 6.1 with the module `euclidean`, except that a side is oriented by the centre and not by the opposite vertex, because the rectangle has no vertex opposite a side.

The triangle has side 0, from vertex 2 to vertex 0, of length 1. The rectangle has the area 1, side 0 at the bottom and side 1 on the right, and `aspect` is the length of side 0 divided by the length of side 1. These units are choices of this design (`DEC-17`), which Section 15.3 puts to the owner. The double of a Euclidean triangle keeps side 0 of length 1, because side 0 of the double is side 0 of its generating triangle. The double of the rectangle has the area 2, because its generating rectangle has the area 1 (Section 6.8).

These statements support the steps:

| Step | Statement used | Source |
| --- | --- | --- |
| 2 | The Euclidean triangle with the angles $\pi/n_j$ exists for exactly the three triples, and is determined up to similarity by its angles | Example 6.5.2 of [DAVIS2008], and Section 7.2 of [RAT], pp. 277 and 279 (Section 9.1 of the domain reference) |
| 1, 2 | The vertices $W_j$ of the triangle | No source. The candidate of item 26 of Section 15.1 of the geometry reference, which `UNV-28` tracks (Section 14) |
| 2 | The reflections in the sides of a rectangle generate the group `*2222` | Example 13.1.3 of [TH], p. 298, and [CH], p. 252 (Section 9.4 of the notation reference), and Example 6.5.2 of [DAVIS2008] (Section 9.1 of the domain reference) |
| 2 | The rectangles form a family of one parameter | [VONGAGERN2009], p. 7 (Section 9.5 of the domain reference), and [CBG], p. 41 (Section 10.2 of the notation reference). Whether the count includes the scale is `UNV-10` |
| 3 to 5 | The centre is the mean of the vertices | `DEC-12`, a choice of the architecture, read for the Euclidean plane by this design |
| 2, 4 | The rotation, the translation, and the origin | `euclidean::rotation`, `euclidean::translation`, and `euclidean::ORIGIN` of the geometry design |
| 6 | The line through two vertices | `euclidean::wall_through` of the geometry design, whose formula has no source (item 24 of Section 15.1 of the geometry reference) |
| 6 | The pairing of a mirror is the reflection in its line | Equation (4.1.1) of [RAT] and equation 6.3 of [DAVIS] (Sections 7.4 and 12 of the geometry reference). The matrix form is part of `UNV-26` |
| All | The reflections generate a discrete group, and the polygon is a strict fundamental domain for it | Theorem 6.4.3 of [DAVIS2008], which covers Euclidean convex polytopes, with $m_{ij} = \infty$ for two sides that do not meet, such as opposite sides of the rectangle (Section 4.5 of the domain reference) |

The vertex coordinates of the triangle have no source, so the build is certified three ways: check 3 tests each vertex against its two walls, check 5 tests each angle against $\pi/n_j$ by equation (1.3.3) of [RAT], and a test compares the built walls with the three lines that SymmHub's code gives for this triangle (Section 9.1 of the domain reference), moved by the translation of step 4. Building the walls from those lines and the vertices by intersection would not close the gap, because the intersection of two lines has no source either.

### 6.6. Shape Parameter and Validation

`*2222`, `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` each have the one shape parameter `aspect`, `o` and `2222` have `aspect` and then `skew`, a polygon `*p1...pn` has the $n - 3$ lengths that the table after DOM-18 states, and every other signature has none, which includes `4*2`, `3*3`, and every spherical group signature:

| Name | Minimum | Maximum | Default | Meaning |
| --- | --- | --- | --- | --- |
| `aspect` | 0.125 | 8 | 1 | The length of side 0 divided by the length of side 1 of the rectangle. For `2222`, the same ratio of its generating rectangle, so the double is $2 \cdot$ `aspect` times as wide as it is high. For a rectangular wallpaper domain, the ratio of the width $w$ to the height $h$ of the rectangle $F$ of Section 6.13. For `o`, the ratio of the length of its first lattice vector to the height of its second |
| `skew` | -0.5 | 0.5 | 0 | The first coordinate of the second lattice vector divided by the length of the first lattice vector, positive when the second vector leans right |

For `2222`, `aspect` is the aspect of the generating rectangle, with the area 1, and the double has the area 2. This is a proposed default of this design (Section 15.3). Side 1 of the generating rectangle is always its right side, so `aspect` and its reciprocal give doubles of different shapes, and the quarter-turn property of item 7 of Section 15.3 holds for `*2222` only. [VONGAGERN2009] p. 7 gives p2 "two geometric parameters" (Section 9.5 of the domain reference), and whether that count includes the scale is `UNV-10`. `skew` is the second parameter of `2222` and of `o` in this design.

DOM-53. `parameters` MUST give `o` and `2222` the parameters `aspect` and then `skew`, MUST give `*2222`, `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` the one parameter `aspect`, and MUST give `4*2` and `3*3` none, with the ranges and defaults of the table above, and the numbering of the sides of each of the ten domains MUST NOT depend on the value of either parameter. For `2222`, `build_with` MUST return the domain of Section 6.8 bit for bit when `skew` is exactly 0, which includes $-0.0$, and MUST apply the shear of Section 6.13 only when it is not. The reasons are that `escher info` prints the parameters in this order and `--set` names them, that a numbering that changed with a setting would rename the colouring ids without an error (DOM-37), and that the golden images and the colouring ids of `2222` at `skew` 0 must not change.

The lattice vectors are $(w, 0)$ and $(\mathrm{skew} \cdot w, h)$ for `o`, and $(2w, 0)$ and $(2 \, \mathrm{skew} \cdot w, 2h)$ for `2222`, with $w = \sqrt{\mathrm{aspect}}$ and $h = 1/w$ as in Section 6.5. The area of the domain does not depend on `skew`, because a shear keeps areas. A shift of the second lattice vector by a whole first vector gives the same lattice, so the range $-0.5$ to $0.5$ reaches every lattice of the family at a given `aspect`. Both ends give the same rhombic lattice, with domains that are mirror images of each other. All four ends of the two ranges are exact in `f64`. The design asserts no count of parameters, and these statements about the range are a reader's reading that `UNV-33` of the plan records with the shear (Section 14).

Item 45 of Section 15.3 puts the convention to the owner. It is `DEC-52` of the architecture, a project convention that settles `UNV-9` of the plan for the Euclidean plane. With `skew` not 0, `2222` is not the double of a rectangle of `*2222`, so DOM-23, and the border oracle of `escher-render` that compares `2222` with `*2222`, hold at `skew` 0 only (Section 6.8).

The range and the default are choices of this design (Section 15.3). The default is the square, as `DEC-13` asks for a regular shape where the orders allow it. The scale is fixed by the area 1 and is not a parameter, and the design asserts no count of parameters, so `deformation_dimension()` of `escher-signature` stays as the signature design states (`UNV-10`).

DOM-18. `build_with` MUST check in this order, and MUST return the first error that applies, because `escher-cli` prints the error of a combination of a signature and settings, and its snapshots fix which kind each combination gives:

1. The predicate of Section 4, else `UnsupportedSignature`.
2. The limit of eight orders of DOM-38, and then the limit of DOM-3, or for `2*n` that of DOM-63, else `LimitExceeded`.
3. Every name of a setting is the name of a `ParameterSpec` of the signature, else `UnknownParameter`.
4. Every value is finite and within the closed range of its `ParameterSpec`, else `ParameterOutOfRange`. A NaN or an infinite value is out of range.
5. A parameter without a setting takes its default value.

A name matches only when it is equal, byte for byte, to the name of a `ParameterSpec`, so `Aspect` and `aspect ` with a trailing space are unknown. A rotation signature passes the same steps, with the `ParameterSpec` values of its own, so `aspect` on `732` gives `UnknownParameter`, and so does any name on `3333`, whose double has no parameter (DOM-47). `skew` on `**` and any name on `4*2` give `UnknownParameter`, and `skew` set to 0.6 on `o` gives `ParameterOutOfRange`. A spherical group signature has no parameter, so any name on it, such as `aspect` on `3*2`, gives `UnknownParameter`. After these steps, the construction runs, and for a polygon or its double the guard of DOM-44, for one of the ten domains the guard of DOM-54, and for a spherical group domain other than that of `*` the guard of DOM-60, runs last, which can return `Numerical`.

A polygon `*p1...pn` with four to eight corners has $n - 3$ shape parameters, each a hyperbolic length at the curvature $-1$, which Section 6.10 derives from the canonical signature:

| Name | Minimum | Maximum | Default | Meaning |
| --- | --- | --- | --- | --- |
| `cornerpair<k>.length` | 0.5 | 3.5 | The length of side $\mathrm{last}$ of the $k$-th corner pair in the default shape, rounded and clamped by DOM-45 | The length of the mirror side between the two right angles of the $k$-th corner pair, which is side $\mathrm{last}$ of the pair, the side from its first vertex to its last vertex |
| `slice<i>.length` | 0.5 | 3.5 | The distance between the two walls of the $i$-th slice in the default shape, rounded and clamped by DOM-45 | The length of the common perpendicular of the two sides just outside the arc that the $i$-th slice cuts off, which are side $\mathrm{first}(y)$ and side $\mathrm{last}(x) + 1$ for the two features $y$ and $x$ that the cut of the slice removes (Section 6.10) |

The corner-pair meaning is the construction itself, because the side of a corner pair gets its parameter as its length. The slice meaning is a reader's inference in the research (item 14 of Section 13.3 of the domain reference), which the tests confirm on the named polygons at all their settings (Section 9.7). For a signature whose corners repeat with a symmetry, two values of one slice can describe congruent polygons: for `*3333`, the value $L$ and the length of the perpendicular across the other pair of sides give the same shape relabelled. This is harmless, and the design does not restrict the range for it.

These are the parameters of some canonical signatures, with the defaults that DOM-45 gives:

| Canonical signature | Typed also as | Features | Parameters | Defaults |
| --- | --- | --- | --- | --- |
| `*3222` | `*2223` | corner 3, pair 1, corner 2 | `cornerpair1.length` | 0.658 |
| `*3322` | `*2233`, `*2332` | corner 3, corner 3, pair 1 | `cornerpair1.length` | 0.867 |
| `*3333` | None | four corners of order 3 | `slice1.length` | 1.317, which is $\operatorname{arcosh} 2$ rounded |
| `*5432` | `*2345` | four corners | `slice1.length` | 1.268 |
| `*22222` | None | pair 1, pair 2, corner 2 | `cornerpair1.length`, `cornerpair2.length` | 1.061 and 1.061 |
| `*222222` | None | pairs 1 to 3 | `cornerpair1.length` to `cornerpair3.length` | 1.317, 1.317, and 1.317 |
| `*2222222` | None | pairs 1 to 3, corner 2 | `cornerpair1.length` to `cornerpair3.length`, `slice1.length` | 1.449, 1.449, 1.449, and 2.076 |

DOM-45. `parameters` MUST give a polygon the parameters `cornerpair<k>.length` for its corner pairs, numbered from 1 in the order of the scan of step 2 of Section 6.10, followed by `slice<i>.length` for its slices, numbered from 1 in the order of the cuts of step 3, each with the closed range 0.5 to 3.5 and the default of the steps below, because `escher info` prints the parameters in this order, `--set` names them, and a name or an order that depended on anything but the canonical signature would give one orbifold two sets of names (SIG-9 of the signature design). The default MUST be computed by these steps, with $\theta_j = \pi / n_j$:

1. Find the inradius $d$ by bisection from $g(d) = \pi$, with $g(t) = \sum_j \arcsin\big(\cos(\theta_j/2) / \cosh t\big)$, equations (7.16.2) and (7.16.3) of [BEA]. Start from the bracket $[0, t_1]$, with $t_1$ the first of $1, 2, 4, \dots$ at which $g(t_1) < \pi$, because $g$ is continuous and decreasing and $g(0) > \pi$ for these angles (proof of Theorem 7.16.2 of [BEA]). The doubling has no cap. It stops at the latest when $t_1$ overflows to infinity, where $g$ is 0, and at once when $g(t_1)$ is NaN. Bisect until the midpoint equals an endpoint of the bracket, at most 200 times.
2. Compute $\alpha_j = \arcsin\big(\cos(\theta_j/2) / \cosh d\big)$ for each corner.
3. Place vertex $j$ at the distance $\rho_j$ from the origin, with $\cosh \rho_j = \cot\alpha_j \cot(\theta_j/2)$, at the direction $\varphi_j$, with $\varphi_0 = 0$ and $\varphi_j = \varphi_{j-1} + \alpha_{j-1} + \alpha_j$, as `hyperbolic::rotation(phi_j) * (hyperbolic::boost(rho_j) * hyperbolic::ORIGIN)`.
4. Take the default of a corner pair as `hyperbolic::distance` between its two vertices, and the default of a slice as $\operatorname{arcosh} \lvert \langle u_a, u_b \rangle \rvert$ for the unit normals $u_a$ and $u_b$ of the two sides of its meaning, from `hyperbolic::wall_through` and the bilinear form of `escher-geometry`.
5. Round each default to three decimals as `(x * 1000.0).round() / 1000.0`, and clamp it to the range 0.5 to 3.5. A default that is NaN becomes the minimum 0.5.

When steps 1 to 4 give a radius, a vertex, or a wall that is not finite or not computed, the default of each parameter that it affects is NaN before step 5, and step 5 makes it the minimum 0.5. This keeps `parameters` total and the invariant of `ParameterSpec` true for any input. The case does not occur over the default sweep of Section 9.7.

`parameters` computes every default. `build_with` computes a default only for a parameter without a setting, and builds the inscribed polygon of steps 1 to 3 at most once for each call, so a build with every parameter set does not compute it.

The steps run in `f64`. The tests check over the named and the small polygons of Section 9.7 that no default is clamped and that each default lies within $5.0 \times 10^{-4}$ of its unrounded value, and a reference part checks the same in the reference arithmetic on a sample of them (Section 15.2). So the rounding errors of `f64` change no default on those sets. The design claims neither for every signature that DOM-38 allows.

The default shape is the polygon with an inscribed circle of the proof of Theorem 7.16.2 of [BEA] (Sections 9.4 and 9.4.2 of the domain reference). It is the regular polygon, the tile of $\{n, 2p\}$, when every order equals $p$. The rounding lets the `{}` format of `f64` print the default without digits that depend on the last bits, which the snapshots of `escher-cli` need. A rounded default is not exactly the inscribed polygon. The default, the range, and the names are proposed defaults of this design (`DEC-13` as amended, `DEC-14` as amended, and `DEC-47` of the architecture), which Section 15.3 puts to the owner.

The relation of step 3 is Theorem 7.11.3 (ii) of [BEA] applied to the right triangle with the vertices at the centre, at the point of tangency, and at vertex $j$, which Figure 7.16.1 of [BEA] shows. Beardon prints only (7.16.3) for that triangle, so the application is a candidate statement (item 29 of Section 15.1 of the geometry reference). The relation of step 4 for a slice is Theorem 3.2.8 of [RAT] with equation (3.2.7), p. 73, stated with the norms and the time coordinate first, and its form for unit normals with the time coordinate last is a candidate statement (item 30 of Section 15.1 of the geometry reference). The tests check both against the reference arithmetic (Section 12.1).

### 6.7. Spherical Construction

For a spherical signature, `build_with` runs steps 1 to 5 of DOM-18, computes the area of the signature as step 1 of Section 6.1 does, and then runs the steps of the triangle or of the lune. A spherical signature has no shape parameter, so `parameters` returns an empty list for it. Let $n_0 \ge n_1 \ge n_2$ be the corner orders of a triangle and $\alpha_j = \pi / n_j$:

1. Compute each $\alpha_j$ as `PI / f64::from(n_j)`, and its sine and cosine.
2. Compute two side lengths by the second law of cosines:
   - $X_a = \dfrac{\cos\alpha_1 + \cos\alpha_2 \cos\alpha_0}{\sin\alpha_2 \sin\alpha_0}$, the cosine of side 0, from vertex 2 to vertex 0, opposite vertex 1.
   - $X_b = \dfrac{\cos\alpha_0 + \cos\alpha_2 \cos\alpha_1}{\sin\alpha_2 \sin\alpha_1}$, the cosine of side 2, from vertex 1 to vertex 2, opposite vertex 0.
   - $L_a = \arccos X_a$ and $L_b = \arccos X_b$, each computed with `f64::acos` on the argument clamped to $[-1, 1]$.
3. Place the triangle in the standard position: $W_2 =$ `spherical::ORIGIN`, $W_0 =$ `spherical::boost(L_a) * spherical::ORIGIN`, and $W_1 =$ `spherical::rotation(alpha_2) * (spherical::boost(L_b) * spherical::ORIGIN)`.
4. Compute $c =$ `spherical::normalize_point(W_0 + W_1 + W_2)`, $M =$ `spherical::inverse(spherical::translation(c))`, and each vertex $V_j = M W_j$, with no renormalization (Section 9.1).
5. For each side $j$ from 0 to 2, with vertex indexes modulo 3, compute $u =$ `spherical::wall_through(V_{j-1}, V_j)`, and negate $u$ if `spherical::side(u, V_{j+1})` is positive. Side $j$ gets the normal $u$, the covector $u$, the pairing `spherical::reflection(u)`, the partner $j$, the kind `Mirror`, and the orientation `Reversing`.
6. Compute the centre as `spherical::normalize_point(V_0 + V_1 + V_2)`.
7. Return `Numerical` if a call of step 4, 5, or 6 returned an error, if any stored number is not finite, or if the value of `spherical::side` at the opposite vertex in step 5 is not strictly negative after the orientation.

Steps 3 to 7 are steps 5 to 11 of Section 6.1 with the module `spherical`. For a lune of the order $n$, with $\theta = \pi / n$:

1. Compute $\theta$ as `PI / f64::from(n)`, and $(s, c)$ as `(theta / 2.0).sin_cos()`. For $n = 1$, which only the domains of `*` and `x` of Section 6.16 reach, set $s = 1$ and $c = 0$ exactly instead.
2. Place the lune directly in the final position: $V_0 = (0, 1, 0)$, $V_1 = (0, -1, 0)$, and the centre `spherical::ORIGIN`.
3. Side 0 gets the normal $(c, 0, -s)$ and side 1 the normal $(-c, 0, -s)$. Each side gets its normal as its covector, the pairing `spherical::reflection(normal)`, the partner $j$, the kind `Mirror`, and the orientation `Reversing`.
4. Return `Numerical` if any stored number is not finite, or if the value of `spherical::side` at the centre is not strictly negative for both sides.

The domain stores the geometry, the corner orders, and the area of the signature, and the sides, vertices, and centre. The rule of `DEC-12` gives the zero vector as the centre of two antipodal vertices, so the centre of the lune is a choice of this design.

These statements support the steps. T names a step of the triangle and L a step of the lune:

| Step | Statement used | Source |
| --- | --- | --- |
| T1 | A corner of order $n$ has the angle $\pi/n$ | Section 4.5 of the domain reference, the hypotheses of Theorem 6.4.3 of [DAVIS2008] |
| T2 | The second law of cosines, $\cos c = (\cos\alpha \cos\beta + \cos\gamma)/(\sin\alpha \sin\beta)$ with $c$ opposite $\gamma$ | [RAT], Theorem 2.5.4, p. 51 (Section 11.3 of the geometry reference). The two lengths permute the angles of the printed form |
| T2, T3 | The triangle exists for these triples and is determined up to congruence by its angles | Example 6.5.1 and the sentence of Example 6.5.3 of [DAVIS2008] (Section 9.1 of the domain reference) |
| T3 | The boost, the rotation, and the origin | `spherical::boost`, `spherical::rotation`, and `spherical::ORIGIN` of the geometry design, from p. 3 of [KC24] |
| T4, T6 | The centre is the normalized sum of the vertices | `DEC-12`, a choice of the architecture |
| T4 | An isometry that maps the origin to $c$, and its inverse | `spherical::translation` and `spherical::inverse` of the geometry design. The image of the origin is an assumption of Section 13.1 of the geometry design |
| T5 | The great circle through two vertices | `spherical::wall_through` of the geometry design, from [RAT], Section 2.1 |
| T5 | The normal points away from the opposite vertex | Section 5.2 of the architecture |
| T5, L3 | The pairing of a mirror is the reflection in its great circle | Equation (6.4) of [DAVIS] (Section 7.4 of the geometry reference) |
| L2 | The vertices and the centre | A choice of this design |
| L3 | The two sides meet at the angle $\theta$ at each vertex | The definition of a lune and of its angle, [RAT], p. 51. That the outward normals of step L3 give this angle has no source (item 28 of Section 15.1 of the geometry reference, part of `UNV-26`). Checks 4 and 5 and the exhaustive certificate of DOM-22 test it |
| All, triangle | The reflections generate a discrete group, and the triangle is a strict fundamental domain for it | Theorem 6.4.3 of [DAVIS2008], p. 83, for a convex polytope in $S^n$ as p. 75 defines it (Sections 4.5 and 9.6 of the domain reference) |
| All, lune | The two reflections generate a group of order $2n$, and the lune is a strict fundamental domain for it | No source. `UNV-29` (item 9 of Section 13.3 of the domain reference), which the exhaustive certificate checks (DOM-22) |

### 6.8. Rotation Construction

DOM-23. `build_with` MUST build the domain of a rotation signature as the double $D = F \cup R_1(F)$ of the generating domain $F$ across the wall of its side 1, MUST leave $F$ where Sections 6.1 to 6.7 place it, and MUST give the double the centre of $F$, except that `2222` with `skew` other than 0 is the shear of that double by Section 6.13 (DOM-53), because the owner decided that `732` and `*732` show the same tiling in the same place (Section 6.3), the border oracle of `escher-render` compares the two images in one view, and the measurements of Sections 8.4 and 9.6 cover this wall and this placement only.

For a rotation signature, `build_with` runs steps 1 to 5 of DOM-18, computes the area of the rotation signature as step 1 of Section 6.1 does, and builds $F$ by the construction of its starred signature with the same orders, for `2222` with the validated value of `aspect`, and for `p1...pn` at the default shape (DOM-47). For `2222` with `skew` other than 0, the steps below run first, and the shear of Section 6.13 then replaces the vertices, the walls, and the pairings. Let $F$ have the $n$ vertices $V_0, \dots, V_{n-1}$, counterclockwise, with the corner orders $n_0, \dots, n_{n-1}$, and the sides $S_j$ from $V_{j-1}$ to $V_j$, with indexes modulo $n$, each with its normal $u_j$, its covector $c_j$, and its reflection $R_j$, the pairing that $F$ stores. Then:

1. Return the error of the construction of $F$, if it returned one.
2. Set the $2n - 2$ vertices of $D$:
   - $W_0 = V_0$.
   - $W_k = R_1 V_{n-k}$ for $k$ from 1 to $n - 2$, as the plain product of the matrix and the vector, with no renormalization (DOM-10).
   - $W_{n-1} = V_1$.
   - $W_{n-1+k} = V_{1+k}$ for $k$ from 1 to $n - 2$.
3. Set side $T_k$ of $D$, from $W_{k-1}$ to $W_k$, with indexes modulo $2n - 2$:
   - $T_0$ is $S_0$, with the normal and the covector of $F$, bit for bit.
   - $T_k$ for $k$ from 1 to $n - 1$ is the image of $S_j$ with $j = (n + 1 - k) \bmod n$. Its normal is $R_1 u_j$. Its covector is `hyperbolic::covector` of that normal for a hyperbolic domain, the normal itself for a spherical domain, and $R_1^{T} c_j$ for a Euclidean domain, whose normal is then $(c'_1, c'_2, 0)$ for that covector $c'$.
   - $T_k$ for $k$ from $n$ to $2n - 3$ is $S_{k - n + 2}$, with the normal and the covector of $F$, bit for bit.
4. Give side $k$ the partner $(2n - 1 - k) \bmod (2n - 2)$.
5. Give the side that is $S_j$ the pairing $R_1 R_j$, and the side that is the image of $S_j$ the pairing $R_j R_1$, each by DOM-25. Give each side the orientation `Preserving`. Give a side of a hyperbolic double the kind `Rotation` when it is $S_j$ or the image of $S_j$ with $j$ equal to 0 or 2, the two sides whose walls meet the wall of side 1 at $V_0$ and $V_1$, and the kind `Translation` for every other $j$. Give a side of a Euclidean or spherical double the kind `Translation` when its pairing passes the translation test of DOM-27, and `Rotation` otherwise. The rule for a hyperbolic double depends only on $j$ and rests on the statement of Section 14 that the walls of two sides of a polygon that are not adjacent do not meet. Check 2 tests it on every build, because check 2 requires the kind to agree with the translation test (DOM-27). The rule gives `Rotation` to every side of a kite, whose only values of $j$ are 0 and 2. The translation test gives `Translation` for sides 2 and 5 of the double of the rectangle, whose walls do not meet the wall of side 1, and for no other side of the Euclidean and spherical doubles that the tests build.
6. Give vertex $W_0$ the order $n_0$ and the copies 2, vertex $W_{n-1}$ the order $n_1$ and the copies 2, and every other vertex the order of the vertex of $F$ that it is or that it is the image of, and the copies 1.
7. Give $D$ the centre of $F$, bit for bit.
8. Return `Numerical` if any stored number is not finite, or if the side value of the centre is not strictly negative for every side.

The steps give these classes. $V_j'$ is $R_1 V_j$:

| Class | $n$ | Vertices of $D$ | Side pairs and kinds | Flat vertices | Vertex cycles |
| --- | --- | --- | --- | --- | --- |
| Kite | 3 | $V_0$, $V_2'$, $V_1$, $V_2$ | 0 and 1, 2 and 3, by rotation | $W_2$ when $n_1 = 2$, so for `n22`, and also $W_0$ for `222` | $\{W_0\}$, $\{W_2\}$, $\{W_1, W_3\}$ |
| Double of the rectangle | 4 | $V_0$, $V_3'$, $V_2'$, $V_1$, $V_2$, $V_3$ | 0 and 1 by rotation, 2 and 5 by translation, 3 and 4 by rotation | $W_0$ and $W_3$ | $\{W_0\}$, $\{W_3\}$, $\{W_1, W_5\}$, $\{W_2, W_4\}$ |
| Double of the lune | 2 | $V_0$, $V_1$ | 0 and 1, by rotation | Both, for `22` | $\{W_0\}$, $\{W_1\}$ |
| Double of a polygon | 4 to 8 | $V_0$, $V_{n-1}'$, ..., $V_2'$, $V_1$, $V_2$, ..., $V_{n-1}$ | $k$ and $(2n - 1 - k) \bmod (2n - 2)$. The pair 0 and 1 and the pair $n - 1$ and $n$ by rotation, and every other pair by translation | $W_{n-1}$ when $n_1 = 2$, such as for `3222`, and also $W_0$ when every order is 2, such as for `22222` | $\{W_0\}$, $\{W_{n-1}\}$, and $\{W_{n-j}, W_{n+j-2}\}$ for $j$ from 2 to $n - 1$ |

No hyperbolic or Euclidean kite has a flat vertex, because a flat vertex needs a corner of order 2 at $V_0$ or $V_1$. The corners of a canonical polygon start with its largest order (Section 5.1), so $V_0$ has the order 2 only when every order is 2. The vertex cycles of the table are the ones that `UNV-31` states. Check 4 walks them on every build (Section 8.2), and the tests assert them. `732` has the corner angles of 51.428571, 90, 120, and 90 degrees and the area $\pi/21$.

Side 0 of a kite is $S_0$, so side 0 of a Euclidean kite has the length 1 (Section 6.5). The double of the rectangle with the width $w$ and the height $h$ of Section 6.5 is the rectangle from $x = -w/2$ to $x = 3w/2$, with the height $h$ and the area 2.

The wall of side 1 is the wall through $V_0$ and $V_1$: for a triangle the side between the two corners of the largest orders, for a polygon the side between its corners 0 and 1, for the rectangle its right side, and for the lune its side with the normal $(-c, 0, -s)$ of Section 6.7. The choice of side 1 is a proposed default of this design, which Section 15.3 puts to the owner. `DEC-11` leaves the choice of the wall to this document (Section 10.2 of the architecture). On a prototype, the double of `*(1000)32` across side 2 failed check 4 and left folds unconverged, while the double across side 1 passed every check. SymmHub's `getGroup_KLM` also doubles the triangle across one of its sides, with pairings that are products of two reflections (Section 7.2.7 of `docs/research/symmhub.md`), which is a precedent in code and not a source.

DOM-24. The vertices, the sides, and the partners of a double MUST follow steps 2 to 4 of this section, with the partner $(2n - 1 - k) \bmod (2n - 2)$ of side $k$, because the renderer records the side index of every fold step, `escher-cli` prints the angles and the side lengths in this order, and check 4 finds the vertex cycles by walking these partners.

DOM-25. The pairing of a side of a double that is $S_j$ MUST be $R_1 R_j$, and that of a side that is the image of $S_j$ MUST be $R_j R_1$, each the compensated product of the two reflections that $F$ stores: every entry MUST be the compensated dot product of a row and a column of the two `f64` matrices, with error-free products by a fused multiply-add and a compensated sum of the three products and their errors. The reason is the fold. On a prototype, one fold of `(1000)32` within the range of accuracy of the renderer failed to converge with plain double-precision products as pairings, while correctly rounded products left no fold unconverged (Section 6.8 of `docs/design/render.md`).

Compensated arithmetic does not guarantee correct rounding. For example, the compensated dot product of $(1, 2^{-53}, 2^{-110})$ and $(1, 1, 1)$ returns 1, while the exact sum rounds to $1 + 2^{-52}$. What is established is narrower: the tests assert that every entry of every pairing of the doubles of Section 9.6 is the exact product of the two stored reflections rounded to the nearest `f64` with ties to even. For the doubles of the named polygons of Section 9.7, a few entries that are cancellations of large products lie one unit in the last place from the nearest `f64`, and the tests bound the absolute error of such an entry. Section 15.1 records the doubles outside these sets as an assumption.

The crate computes each entry at construction only. The compensation runs once per side, so the fold costs the same. `f64::mul_add` of the standard library gives the fused multiply-add, which IEEE 754 defines exactly, so the product is deterministic.

The compensated product does not reduce the isometry defect of a pairing in general. A measurement outside the tests found that over the cone triples of the 30 named triples, the largest isometry defect is the same with compensated and with plain products, and that at some doubles the compensated product has the larger defect. The isometry defect of a pairing of a double therefore rests only on the bounds that the tests set from measurement (Section 9.6). This requirement is a proposed default of this design, and Section 15.3 states the alternatives.

DOM-26. Where the wall of a side $S_j$ meets the wall of side 1 at a corner of order 2, at $V_0$ or $V_1$, the double MUST keep $S_j$ and its image as two sides that are partners of each other and meet at a flat vertex, and MUST NOT make any side its own partner, because Theorem 12 of [SOLOMYAK2020] requires that no side is paired with itself (Section 4.1 of the domain reference), and check 2 treats a side that is its own partner as a mirror. Item 2 of Section 4.8 of the domain reference permits this split and does not require it, so the split is a choice of this design. The same rule holds for a side of a wallpaper domain that a half-turn about its midpoint maps onto itself: the domains of `22*` and `2*22` split it at the centre of the half-turn into two sides that are partners of each other and meet at a flat vertex (Section 6.13).

DOM-47. `build_with` MUST build the domain of a rotation signature `p1...pn` with four to eight orders as the double of this section of the polygon of its starred signature at the default shape of DOM-45, and `parameters` MUST give it no parameter, because the research has no source for parameters of `p1...pn` beyond the lengths of the polygon (Section 9.4.2 of the domain reference), the double of a polygon at another shape is one member of a larger family whose other parameters, such as twists, this design does not build, and a parameter that moved only part of the family would be misleading to name. The double inherits the numbering of DOM-24 and the guard of DOM-44, which runs on the double alone. The generating domain is therefore bit for bit the polygon of steps 1 to 12 of Section 6.10, which is the polygon that `build` returns for the starred signature whenever the guard passes that polygon (DOM-30). Over the sweep of the doubles of Section 8.4, the guard refuses no starred polygon. This is a proposed default of this design (`DEC-50` of the architecture).

The image vertices, normals, and covectors are plain double-precision products. The doubles of polygons that the guard refuses fail check 4 through the `f64` storage of their pairings, and not through their image vertices (Section 8.4). No measurement covers the effect of correctly rounded image vertices and normals on check 4 in the arithmetic of DOM-49, or on the margin of the vertices against the tolerance of the fold (Section 15.3, item 27).

### 6.9. Theorems That Make the Double a Fundamental Domain

These sourced theorems and unverified statements bear on the double. This document adds no proof (Hard Rule 14 of `AGENTS.md`):

| Statement | Source | Status |
| --- | --- | --- |
| A convex hyperbolic polygon with finitely many sides, with side pairings in $PSL_2(\mathbb R)$, no side paired with itself, and every elliptic cycle meeting $m \cdot \mathrm{sum}(E) = 2\pi$, is a fundamental domain of the Fuchsian group that its pairings generate, which the cycle relators present | Theorem 12 and Definition 11 of [SOLOMYAK2020] (Section 4.1 of the domain reference), and Theorem 6.14 with condition (a) of [SERIES2013] (Section 4.2) | Sourced, hyperbolic |
| For the Euclidean plane and the sphere, Pairing, Connected, Cyclic, and Finite give a discrete group with the cell as a fundamental domain | Theorem 4.14 with (k) of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference), and Theorem 13.5.3 of [RATCLIFFE] with its completeness hypothesis (Section 4.10) | Sourced |
| The product of the pairings along a vertex cycle is a rotation through the angle sum of the cycle, of the form $2\pi/m$ | Conditions 3.7, 3.10, and 3.13 and Lemma 3.12 of [EPSTEIN-PETRONIO1994], and item 3 of Section 4.8 of the domain reference | Sourced |
| A side fixed by a half-turn may be split at the fixed point, with a new vertex of the angle $\pi$ | Section 1 of [VOIGHT2009], slide 22 of [SOLOMYAK2020], and Definition 6.1 of [SERIES2013] (item 2 of Section 4.8 and Section 4.3 of the domain reference) | Sourced. Permitted and not required |
| A convex cell is a convex polyhedron whose proper faces may have been subdivided | Definition 2.8 of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference) | Sourced as quoted. That it admits the flat vertices of a double is a reader's reading, recorded with `UNV-31` |
| The rotation triangle group has generators that are products of two reflections, $a_1 = r_2 r_3$ and $a_2 = r_3 r_1$ | Sections 4.4 and 5.9 of [MASKIT2001] (Sections 4.4, 7.6, and 9.1 of the domain reference) | Sourced, hyperbolic |
| The fundamental cells of p3, p4, and p6 consist of two copies of the triangles of p3m1, p4m, and p6m, and the rotated kite has two equal angles $\pi/k$ and the angles $2\pi/m$ and $2\pi/n$ on its axis | [VONGAGERN2009] pp. 7 and 12 (Section 9.5 of the domain reference) | Sourced, Euclidean, with hyperbolic relatives |
| The double triangle with pairings that are products of two reflections | SymmHub's `getGroup_KLM` (Section 7.2.7 of `docs/research/symmhub.md`) | Code precedent only |
| The group of the double has the rotation signature, `pqr`, `n22`, `nn`, or `2222` | No statement in orbifold notation (gap G10 of Section 13.2 of the domain reference) | `UNV-22` |
| The double of the lune of the angle $\pi/n$, a lune of the angle $2\pi/n$ with one rotation pairing, which for `22` is a hemisphere, is a fundamental domain of the cyclic group of order $n$ | None (item 10 of Section 13.3 of the domain reference) | `UNV-30`, which the tests of DOM-31 check |
| The double of a convex reflection polygon across one wall is convex, and its vertex cycles are those of the table of Section 6.8, with the corner orders stated there | None (item 11 of Section 13.3 of the domain reference) | `UNV-31`, which checks 3, 4, and 5 test on every build |
| The walls of two sides of a compact hyperbolic polygon with corner angles of at most $\pi/2$ that are not adjacent do not meet, so the product of the reflections in them is a translation, which fixes no point | A standard result on acute-angled polygons, attributed to Vinberg, which no reference of the project cites | Unverified (Section 14). Check 2 tests the kind that step 5 of Section 6.8 sets from it on every build |

The checklist of Section 4.8 of the domain reference applies to the double in this way:

| Hypothesis | How the design meets it |
| --- | --- |
| Each side is paired with exactly one other side, and the partner's pairing is the inverse (item 1 of Section 4.8, Condition 3.2 (a) and (d) of [EPSTEIN-PETRONIO1994]) | Checked: check 2 |
| A pairing maps the interior to the other side of the partner side, and maps the side onto its partner (Condition 3.2 (b) and (c)) | Checked: check 2, with the endpoint terms and the wall term of a side that is not a mirror |
| No side is paired with itself (Theorem 12 of [SOLOMYAK2020]) | Checked: check 2 (DOM-26 and DOM-27) |
| Each pairing is an isometry | Checked: check 1 |
| Every vertex cycle is finite, and its product is a rotation through the angle sum $2\pi/m$ (item 3 of Section 4.8) | Checked: checks 4 and 5 (DOM-28) |
| The polygon is convex | `UNV-31`. Checked numerically: the convexity term of check 3 |
| The area agrees with the Euler characteristic | Checked for consistency only: check 6 |

### 6.10. Polygon Construction

For a polygon `*p1...pn` with $n$ from 4 to 8 corners, `build_with` runs steps 1 to 5 of DOM-18, computes the area of the signature as step 1 of Section 6.1 does, and then runs the steps of this section. Let $n_0, \dots, n_{n-1}$ be the corner orders in the canonical order of Section 5.1, $\alpha_j = \pi / n_j$, and the lengths the validated values of the parameters of Section 6.6. Indexes of vertices and sides are modulo $n$:

1. Return `UnsupportedSignature` or `LimitExceeded` as DOM-18 requires.
2. Features. Scan the corners from $j = 0$. When $n_j = 2$, $j + 1 < n$, and $n_{j+1} = 2$, append the corner pair $k$, numbered from 1 in the order of the scan, with $\mathrm{first} = j$ and $\mathrm{last} = j + 1$, and continue at $j + 2$. Otherwise append the corner $j$, with $\mathrm{first} = \mathrm{last} = j$, and continue at $j + 1$. The scan does not wrap around. A run of corners of order 2 of odd length leaves its last corner single.
3. Cuts. Let $L$ be the list of step 2. While $L$ has more than three features, let $x$ be its last feature and $y$ the one before it, and remove both. Create the slice $i$, numbered from 1 in the order of the cuts. Record the sheet $(y, x, \bar s_i)$, in which $\bar s_i$ is the slice $i$ standing for the complementary arc $[\mathrm{last}(x) + 1 .. \mathrm{first}(y) - 1]$, and put at the front of $L$ the slice $i$ standing for the arc $[\mathrm{first}(y) .. \mathrm{last}(x)]$. When three features remain, record them in the order of $L$ as the root sheet.
4. For each sheet and each two consecutive features $f$ and $g$ in its cyclic order, from the first to the second, the second to the third, and the third back to the first, require $\mathrm{last}(f) + 1 \equiv \mathrm{first}(g) \pmod n$. The gap between $f$ and $g$ then lies on side $\mathrm{last}(f) + 1$.
5. Gaps. For each sheet and each two consecutive features $f$ and $g$, with $h$ the third feature, give each feature its pair $(c, s)$: $(\cos\alpha, \sin\alpha)$ for a corner of the angle $\alpha$, and $(\cosh \ell, \sinh \ell)$ for a corner pair or a slice of the length $\ell$. Compute $X = (c_f c_g + c_h) / (s_f s_g)$, and the gap as `X.acosh()` when $f$ and $g$ are both corners or both not corners, and as `X.asinh()` otherwise. Every sheet gets all three of its gaps, including the gap that closes it.
6. Side lengths. Give side $\mathrm{last}$ of each corner pair the value of its parameter. Give every other side $j$ the sum of the gaps that lie on it, over the sheets in the order in which step 3 recorded them and over the gaps of a sheet in the order of step 4.
7. Walk, in double-double arithmetic. Let $F$ be the identity and $V_0 =$ `hyperbolic::ORIGIN`. For $j$ from 1 to $n - 1$, set $F \leftarrow F \, B(L_j)$, take the third column of $F$ as $V_j$, and set $F \leftarrow F \, R(\pi - \alpha_j)$. $B(L)$ is the matrix of `hyperbolic::boost(L)` and $R(\beta)$ that of `hyperbolic::rotation(beta)`, with each entry a double-double number. Side 0 is never set. It closes the polygon, as side 1 of the triangle does in step 5 of Section 6.1.
8. Placement, in double-double arithmetic. Compute $c = \sum_j V_j$, normalize it to the hyperboloid by dividing it by $\sqrt{-\langle c, c \rangle}$, and compute $M$ as the inverse of the translation $T$ that maps the origin to $c = (x, y, t)$. Write $T$ in closed form, with the rows $(1 + x^2/(1 + t), xy/(1 + t), x)$, $(xy/(1 + t), 1 + y^2/(1 + t), y)$, and $(x, y, t)$, and take $M$ as $T$ with the off-diagonal entries of its last row and its last column negated. Set $W_j = M V_j$. With $(x, y)$ the first two coordinates of $W_0$ and $r = \sqrt{x^2 + y^2}$, apply to every $W_j$ the rotation about the origin whose cosine is $x / r$ and whose sine is $-y / r$, which puts $W_0$ on the positive first axis.
9. Round each coordinate of each $W_j$ once to the nearest `f64`, and store the result as vertex $V_j$. Do not renormalize it (DOM-10).
10. Compute the centre as `hyperbolic::normalize_point` of the sum of the stored vertices.
11. For each side $j$, compute $u =$ `hyperbolic::wall_through(V_{j-1}, V_j)`, and negate $u$ if `hyperbolic::side(hyperbolic::covector(u), centre)` is positive. Side $j$ gets the normal $u$, the covector `hyperbolic::covector(u)`, the pairing `hyperbolic::reflection(u)`, the partner $j$, the kind `Mirror`, and the orientation `Reversing`.
12. Return `Numerical` if the condition of step 4 fails, if a gap would lie on the side of a corner pair, if a side other than that of a corner pair gets no gap, if a gap or a side length is not finite or not positive, if the placement of steps 8 and 9 fails, if the walk places fewer vertices than the polygon has corners, if a call of step 10 or 11 returned an error, or if the value of `hyperbolic::side` at the centre is not strictly negative for every side after the orientation.
13. Return `Numerical` unless the guard of DOM-44 passes. For the rotation signature `p1...pn`, this step does not run on the polygon: `build_with` doubles the polygon of step 12 by Section 6.8 and runs the guard on the double (DOM-47).

The placement has one failure condition, evaluated after step 9. It succeeds only when the square root of $-\langle c, c \rangle$ for the vertex sum $c$ is positive, the time coordinate of the normalized centre is positive, the radius $r$ of $W_0$ is positive, and every stored vertex is finite.

No step checks separately that every stored number is finite. The placement refuses a vertex that is not finite, and the guard of step 13 refuses a pairing that is not finite, because its residual is infinite.

The domain stores the geometry, the corner orders with the copies 1, the area of the signature, and the sides, vertices, and centre of steps 9 to 11. A side of a corner pair has its parameter as its length, and the side lengths of step 6 serve only the walk. The crate keeps them for a test that compares side 0 with its sum of gaps (Section 12.1).

These are the sheets of some signatures, in the notation $c_j$ for corner $j$, $P_k$ for corner pair $k$, and $s_i$ or $\bar s_i$ for slice $i$:

| Canonical signature | Features of step 2 | Sheets of step 3, in the order recorded |
| --- | --- | --- |
| `*3222` | $c_0$, $P_1 = [1 .. 2]$, $c_3$ | Root $(c_0, P_1, c_3)$ |
| `*3322` | $c_0$, $c_1$, $P_1 = [2 .. 3]$ | Root $(c_0, c_1, P_1)$ |
| `*3333` | $c_0$, $c_1$, $c_2$, $c_3$ | $(c_2, c_3, \bar s_1)$ with $\bar s_1 = [0 .. 1]$, then root $(s_1, c_0, c_1)$ with $s_1 = [2 .. 3]$ |
| `*22222` | $P_1 = [0 .. 1]$, $P_2 = [2 .. 3]$, $c_4$ | Root $(P_1, P_2, c_4)$ |

A sheet holds its features in the counterclockwise order of the polygon. Recording a cut sheet in the order in which the features leave the list, $(x, y, s)$, as SymmHub's code and a literal reading of the domain reference suggest, and then placing the gap between $f$ and $g$ on side $\mathrm{last}(f) + 1$, gives a polygon that does not close, as a prototype found for `*3333`, `*5432`, `*22222`, and `*65432`. With the order of step 3, every gap satisfied step 4 and every walk closed in the runs of the prototype, and the arcs wrapped modulo $n$ from $n = 7$ on. The design does not claim the condition for every signature. DOM-39 turns a gap that breaks it into `Numerical`.

DOM-39. The construction MUST derive the features, the cuts, and the sheets from the canonical corner orders by steps 2 to 4, MUST record each cut sheet as $(y, x, \bar s_i)$ with $\bar s_i$ standing for the complementary arc, and MUST return `Numerical` when a gap fails the condition of step 4, because the side on which a gap lies follows from that order alone, a sheet in another order puts its gaps on the wrong sides and the polygon does not close, and the parameter names of DOM-45 follow from the same scan and cuts.

DOM-40. The construction MUST keep the three features of every sheet in their counterclockwise cyclic order, and MAY start a sheet at any of its features, but MUST NOT exchange two features of a sheet, because an exchange reverses the sheet and puts the corners of the polygon in another cyclic order. That is the defect of SymmHub's code, which a port of the code found for most signatures with five to eight corners (Section 6.8 of `docs/research/symmhub.md`). The gaps of step 5 do not depend on where a sheet starts, because the expression is symmetric in $f$ and $g$.

DOM-41. The gaps MUST be computed by the one function of step 5 for every two consecutive features of every sheet, with its two branches, and the length of a side MUST be the value of its corner pair or the sum of the gaps on it by step 6, because SymmHub's code computes only two gaps of each sheet and lays out the third by its turtle (Section 6.4.1 of `docs/research/symmhub.md`), while the walk of step 7 needs the length of every side, and a closing gap computed in any other form would rest on no statement of the research.

The function of step 5 is the generalized law of cosines of SymmHub's `edgeLength` (Section 8.7 of the domain reference). The closing gap of a sheet, the one gap that SymmHub's layout does not compute, is the same function with its arguments in the cyclic order of the sheet. In the sheets of a polygon with four or more corners, the closing gap is an $\operatorname{arsinh}$ call when the sheet holds two corners or one corner, and an $\operatorname{arcosh}$ call when it holds none. Sheets with three corners did not occur in the sheets that a prototype enumerated. On the same prototype, the closing gap agreed with the sheet layouts of a port of SymmHub's code to the precision of the port, and no argument fell below 1 for $\operatorname{arcosh}$ or below 0 for $\operatorname{arsinh}$ over the range of Section 6.6.

DOM-42. The walk of step 7 and the placement of step 8 MUST compute every entry of every factor matrix and every product in double-double arithmetic, and MUST round each coordinate of each vertex to `f64` once, after the placement, because the plain `f64` walk is too inaccurate: on a prototype, it failed `verify()` for most random settings with six to eight corners and for most default shapes with a corner of order 1000, while the 106-bit walk reached the accuracy of the exact polygon rounded once. The prototype isolated the error in the factors: the exact products of the `f64`-rounded factor matrices were as inaccurate as the `f64` walk, and the exact products of factors computed exactly from the same `f64` lengths were not. The report of the prototype reads this as the rounding of $\cosh$, $\sinh$, $\cos$, and $\sin$ making each factor slightly not an isometry and the walk amplifying that error with the length of the path from vertex 0. The design relies on the measurements and not on that reading.

The lengths $L_j$ of step 6 stay `f64` values. A change of a length keeps each factor an isometry. This is harmless over the default sweep of Section 9.7, and the design does not claim it for every signature.

The walk uses the double-double arithmetic of [QD], which no author of this design read (Section 14). It is built on the error-free sum `two_sum` and the error-free product `two_product` of `src/double.rs`, which DOM-25 also uses, and `src/walk.rs` implements these operations:

- Sum: the accurate sum of [QD], with two calls of `two_sum` and two fast renormalizations. The difference is the sum with the negation.
- Product: `two_product` of the high parts, plus the cross terms through one `f64::mul_add`, then a fast renormalization. The product by an `f64` follows the same pattern.
- Quotient: a long division with three terms. The quotient by an `f64` takes one correction step.
- Square root: one step of Newton's method on the `f64` root, with the residual $a - s^2$ taken exactly from `two_product`.
- $\cosh L$ and $\sinh L$ of an `f64` length $L$. Reduce $L$ by $k \ln 2$, with $\ln 2$ as a double-double number and $k$ rounded to an integer, so that the remainder $r$ has $\lvert r \rvert \le 0.347$. Scale $r$ by $2^{-10}$, which is exact, sum the Taylor series of $e^{r} - 1$ to the 12th power, and undo the scaling with 10 squarings in the form $t \leftarrow 2t + t^2$, which keeps $e^{r} - 1$ free of cancellation. Multiply $e^{r}$ by $2^k$ through a loop of exact doublings. Then $\cosh L = (e^L + e^{-L})/2$ and $\sinh L = (e^L - 1)(1 + e^{-L})/2$, which keeps $\sinh L$ accurate relative to its value at small lengths. A length that is not finite or whose absolute value is above 700 gives NaN.
- $\cos\alpha$ and $\sin\alpha$ of the double-double angle $\alpha = \pi/n_j$, by one Taylor series of each with 22 terms, in double-double arithmetic. The angle is at most $\pi/2$, so the terms stay below 1.6 and no argument reduction is needed. The rotation of step 7 uses $\cos(\pi - \alpha) = -\cos\alpha$ and $\sin(\pi - \alpha) = \sin\alpha$, which are exact.
- The constant $\pi$ as a double-double number whose high part is `core::f64::consts::PI`, and $\alpha_j$ as its quotient by $n_j$.
- The product of two 3 by 3 matrices, each entry the double-double sum of three double-double products.
- The rounding of a normalized double-double number to the nearest `f64`, which is its high part. The code writes it as the `f64` sum of the high part and the low part, which gives the same value.

The closed form of the translation in step 8 is the matrix that `hyperbolic::translation` gives as $R(\varphi) \, B(\rho) \, R(-\varphi)$ from the polar form of $c$ (Section 6.4 of the geometry design), multiplied out. It needs no $\operatorname{atan2}$, $\operatorname{arsinh}$, or `hypot`, which have no double-double form in the crate, and its inverse negates the entries that mix the time coordinate with a space coordinate, as `hyperbolic::inverse` does. Section 15.1 records the closed form as an assumption.

Each function MUST match the reference arithmetic of Section 12.2 within the bound that its test sets, as TEST-13 of the coding standard requires, and the built polygons MUST meet the bounds that the tests of Section 9.7 set.

DOM-43. The construction MUST place the polygon with its centre at the origin and vertex 0 on the positive first axis by step 8, MUST number its vertices with vertex $j$ at corner $j$ of the canonical signature and side $j$ from vertex $j - 1$ to vertex $j$ (DOM-4), and MUST make this numbering depend only on the canonical signature and never on the values of the parameters, because the colouring ids depend on the numbering (DOM-37), and a numbering that changed with `--set` would rename them without an error. The placement of vertex 0 on the first axis makes the orientation of the polygon in the canonical view predictable for the artist. It is a proposed default of this design (`DEC-49` of the architecture).

DOM-44. `build_with` MUST return `Numerical` for a polygon, and for the double of a polygon, unless `verify().passed()` holds for the built domain, because `escher render` folds into a domain without calling `verify()`, the construction rests on unverified statements (Section 14), and a shape that the arithmetic cannot build within the tolerances of Section 8.4 would otherwise reach the renderer as a wrong polygon. The guard runs all six checks, because a guard of checks 3 and 5 alone passes polygons whose other checks fail (Section 6.12). It runs once, in `build_with`, on the polygon for `*p1...pn` and on the double for `p1...pn`. It does not run on the generating polygon of a double.

A refusal by the guard is the documented outcome for a shape too extreme to build accurately. It is not a defect of the crate. On a prototype, the exact polygon rounded once to `f64` failed `verify()` for a small share of random settings with seven and eight corners and for none with up to six, by checks 1 and 2, the isometry defect of reflections in walls far from the centre. The crate refuses about the same shares.

Over the default sweep of Section 9.7, which a program outside the repository ran, the guard refuses no default shape, but the margins of checks 1 and 2 fall below 8 for some signatures with seven or eight corners and a corner of order 1000. They come from storing a polygon with vertices far from the centre in `f64`, and not from the walk. Section 8.4 states the margin that this design claims.

The guard refuses some doubles of polygons at the default shape, which is their only shape (DOM-47). Section 8.4 states which, and why.

The guard costs one call of `verify()`, whose number of matrix products Section 8.2 states. Only the polygons and their doubles run this guard, and the ten domains of Section 6.13 run the guard of DOM-54. The triangles, the other Euclidean and the spherical domains, and their doubles pass every check over their measured ranges (Section 8.4), so the guard would change nothing for them and would double the cost of `escher info --check`.

### 6.11. Theorems That Make the Polygon a Fundamental Domain

These sourced theorems and unverified statements bear on the polygon. This document adds no proof (Hard Rule 14 of `AGENTS.md`):

| Statement | Source | Status |
| --- | --- | --- |
| A convex polygon with the angles $\pi/m_j$ in a given order exists when $\sum_j 1/m_j < n - 2$ | Example 6.5.3 of [DAVIS2008], and Theorem 7.16.2 of [BEA], p. 155 (Sections 9.4 and 9.4.2 of the domain reference) | Sourced, existence only |
| The reflections in the sides of such a polygon generate a discrete group, and the polygon is a strict fundamental domain for it | Theorem 6.4.3 of [DAVIS2008] (Section 4.5 of the domain reference) | Sourced. Applies to each polygon that the guard passes |
| The area of a polygon is $(n - 2)\pi$ minus its angle sum | Theorem 7.15.1 of [BEA], p. 153 (Section 10.1 of the geometry reference) | Sourced |
| The structures of the polygon orbifold are parametrized by the lengths of the cuts into generalized triangles, with one cut omitted and an edge length as a parameter where two corners of order 2 meet | [TH] p. 316 (Sections 9.4 and 9.4.2 of the domain reference) | Sourced. That the slices and corner pairs of Section 6.10 are these cuts and edge lengths is `UNV-7` (item 12 of Section 13.3 of the domain reference) |
| The deformation space of a disc with $l$ corner reflectors has the dimension $l - 3$ | Corollary 13.3.7 of [TH], p. 318, with a reader's substitution (Section 9.4 of the domain reference) | Sourced, dimension only |
| The $\operatorname{arcosh}$ branch of the gap function has the form of the formulas of the triangle, the quadrilateral with two right angles, the pentagon with four right angles, and the right-angled hexagon | Theorems 3.5.4, 3.5.7, 3.5.11, and 3.5.13 of [RATCLIFFE] (Section 8.7 of the domain reference) | The correspondence is a reader's reading, part of `UNV-6` |
| The $\operatorname{arsinh}$ branch of the gap function, including the closing gaps of the sheets with one or two corners | None (gap G3 of Section 13.2 of the domain reference) | `UNV-6` |
| Every sheet exists and closes for lengths in the range, with the angle $\pi/m$ at each corner and right angles at the ends of each length | None (item 13 of Section 13.3 of the domain reference) | `UNV-27`, with `UNV-6` |
| The gaps at the end of a slice lie on one geodesic, so a side is the sum of the gaps on it | None (item 14 of Section 13.3 of the domain reference) | `UNV-27` |
| The polygon with an inscribed circle, with $\sin\alpha_j = \cos(\theta_j/2) / \cosh d$ and $\sum_j \alpha_j = \pi$ | The proof of Theorem 7.16.2 of [BEA], pp. 155 and 156, equations (7.16.2) and (7.16.3) (Section 9.4 of the domain reference) | Sourced, one member of the family |
| The distance from the centre of the inscribed polygon to vertex $j$, $\cosh\rho_j = \cot\alpha_j \cot(\theta_j/2)$ | Theorem 7.11.3 (ii) of [BEA], p. 147, applied to the right triangle of Figure 7.16.1 (Section 10.3 of the geometry reference) | A candidate statement (item 29 of Section 15.1 of the geometry reference) |
| The distance between two ultraparallel walls with the normals $x$ and $y$ is $\eta$ with $\lvert x \circ y \rvert = \lVert x \rVert \, \lVert y \rVert \cosh \eta$, along their common perpendicular | Theorem 3.2.8 and equation (3.2.7) of [RAT], p. 73 (Section 9.3 of the geometry reference) | Sourced, with the time coordinate first. Its form $\operatorname{arcosh} \lvert \langle u, v \rangle \rvert$ for unit normals with the time coordinate last, which step 4 of DOM-45 uses, is a candidate statement (item 30 of Section 15.1 of the geometry reference) |
| The area of a hyperbolic orbifold is $-2\pi\chi$ | [TH] 13.3.5, p. 312 (Section 7.3 of the notation reference) | Sourced |

The checklist of Section 4.8 of the domain reference applies to the polygon in this way:

| Hypothesis | How the design meets it |
| --- | --- |
| Each side is paired with itself by the reflection in its wall, which is an isometry (items 1 and 2 of Section 4.8) | Checked: checks 1, 2, and 3, on every polygon that `build_with` returns (DOM-44) |
| Every corner has the angle $\pi/n_j$ in the canonical order, and the relator $(r_j r_{j+1})^{n_j}$ holds (item 4 of Section 4.8) | Checked: checks 4 and 5 against the stored orders, which are the canonical ones |
| The polygon is convex and simple | Checked numerically: the convexity term of check 3 and the angle sum of check 5. That these make the closed walk simple is an assumption of Section 15.1 |
| The area agrees with the Euler characteristic | Checked for consistency only: check 6 |

The unverified statements `UNV-6`, `UNV-7`, and `UNV-27` bear on how the polygon is found, and not on whether a returned polygon is a fundamental domain. A polygon defined by its $n$ stored vertices that passes the guard meets numerically the hypotheses of Theorem 6.4.3 of [DAVIS2008], whatever the sheet formulas computed. A wrong formula can therefore make a build fail the guard, or make a parameter mean something other than Section 6.6 states, and it cannot produce a wrong group. A wrong cyclic order of the corners, the defect of DOM-40, fails check 5, because the stored corner orders are the canonical ones. `UNV-12`, the pentagon relations that `docs/design/poc-plan.md` names for this class, is not relied on separately: the $\operatorname{arsinh}$ call of a sheet with one corner has the form of its pentagon relation and is part of `UNV-6`.

### 6.12. Alternatives to the Polygon Construction

The design considered these alternatives, and judged them on a prototype outside the repository:

| Alternative | Verdict | Reason |
| --- | --- | --- |
| The walk of step 7 in plain `f64` with `hyperbolic::boost` and `hyperbolic::rotation` | Rejected | It failed `verify()` far more often than the exact polygon rounded once, for random settings from five corners on and for most default shapes with an order of 1000. Exact products of the `f64` factor matrices were as bad, so the error is in the rounded factors and not in the products (DOM-42) |
| The `f64` walk with each frame orthonormalized again, or with each vertex normalized | Rejected | No improvement, and normalized vertices made the result worse |
| A second `f64` walk started at the placed frame of vertex 0 | Rejected | One to two orders of magnitude better, which is not enough |
| SymmHub's layout of each sheet with welds along the slices | Rejected for the build, kept as a test oracle | It needs frames, an inverse, and splice lists, and relies on the same collinearity of the gaps at a slice end (`UNV-27`). The tests compare it with the walk in the reference arithmetic |
| The closure solved numerically from the side lengths | Rejected | The set of admissible side lengths is not a box, so no range can be stated for it, it needs a solver, and it shares nothing with a later construction of the other classes from the same cuts |
| A guard of checks 3 and 5 only | Rejected | Polygons exist, such as the default of `*(1000)2222`, that pass checks 3 and 5 and fail check 6 or checks 1 and 2 |
| No limit on the corners below the 63 corners that `escher-signature` admits, since `*` counts as one feature of its limit of 64 | Rejected | On a prototype, the margins at the default shape fell below 8 for some signatures tried with 12 or more corners and a corner of order 1000, and for every signature tried with 32 or more corners. The Conway presentation of 33 corners of order 1000 expands to 66,073 letters, more than the limit of 65,536 of `escher-group` |
| A limit of 12 corners with orders up to 200 | An alternative for the owner | At the default shape, every signature tried with up to 12 corners and orders up to 200 kept a large margin, while with orders up to 1000 the margin fell below 8 at 12 corners (item 36 of Section 15.3) |
| The range 0.3 to 4 of SymmHub | Rejected | The exact polygon rounded once failed `verify()` for more random settings with six to eight corners than with the range of Section 6.6, and no range gives a margin of 8 for every setting with seven or eight corners |
| The default length 2 of SymmHub, or the regular shape only where the orders are equal, as `DEC-13` stated it before its amendment | Rejected | The length 2 does not give the regular polygon when the orders are equal: the regular `*3333` has `slice1.length` 1.317. The research gives no formula for the cut lengths of a regular shape, and the inscribed polygon gives a default for every signature |
| The angle of a vertex of a polygon from the law of cosines in the triangle of its two neighbours | Rejected | The corner frame is more accurate, and at the flat vertex of the double of `*22222222` the law of cosines gives a residual of check 5 above the tolerance |
| The angle of a vertex with the copies 2 of a double as the sum of the two angles in the corner frame on either side of the diagonal from $W_0$ to $W_{n-1}$ | Rejected | It is no more accurate than one angle in the corner frame, and one angle needs no layout of the double, so DOM-46 uses it at every vertex |

### 6.13. Wallpaper Construction

For a wallpaper signature, `build_with` runs steps 1 to 5 of DOM-18 and then the steps of this section. For `2222` with `skew` other than 0, it builds the double of Section 6.8 and then runs the shear of step 7. The conventions of Section 5 hold: the vertices are counterclockwise, side $j$ runs from vertex $j - 1$ to vertex $j$, $P_j$ maps the neighbouring copy across side $j$ onto the domain, and every normal points outward.

Let $a$ be the validated value of `aspect`, $k$ that of `skew`, $w = \sqrt{a}$, and $h = 1/w$. The rectangle $F$ is the rectangle of `*2222` of Section 6.5 at $a$, bit for bit: $W_0 = (w/2, -h/2, 1)$, $W_1 = (w/2, h/2, 1)$, $W_2 = (-w/2, h/2, 1)$, and $W_3 = (-w/2, -h/2, 1)$, with side 0 at the bottom, side 1 on the right, side 2 at the top, and side 3 on the left. Its centre is `euclidean::ORIGIN`. Where a mirror line or a glide axis can be horizontal or vertical, it is vertical. Section 6.15 states this and the other choices.

DOM-50. `build_with` MUST give each of the ten domains the vertices, the sides, the partners, the kinds, the orientations, and the vertex orders of this table, with the numbering that it states, because the colouring ids depend on the numbering (DOM-37), the renderer reads the partner, the orientation, and the pairing of every side, `escher info` prints the kind of each pair, and check 4 walks the cycles from the partners and compares them with the stored orders. $V_j$ is vertex $j$ of the domain, and "order" is the cycle order of Section 2.1:

| Signature | Vertices $V_0, \dots, V_{m-1}$ | Sides | Vertex cycles, with their orders | Parameters |
| --- | --- | --- | --- | --- |
| `o` | With $A = (w, 0)$ and $B = (k w, h)$: $V_0 = (A - B)/2$, $V_1 = (A + B)/2$, $V_2 = (B - A)/2$, $V_3 = -(A + B)/2$ | 0 and 2 by translation, $P_0$ by $+B$ and $P_2$ by $-B$. 1 and 3 by translation, $P_1$ by $-A$ and $P_3$ by $+A$ | $\{V_0, V_1, V_2, V_3\}$, order 1 | `aspect`, `skew` |
| `**` | $W_0$ to $W_3$ | 1 and 3 mirrors. 0 and 2 by translation, $P_0$ by $(0, +h)$ and $P_2$ by $(0, -h)$ | Chains $\{V_0, V_1\}$ and $\{V_2, V_3\}$, order 1 | `aspect` |
| `xx` | $W_0$ to $W_3$ | 0 and 2 by glide reflection, $P_0(x, y) = (-x, y + h)$ and $P_2(x, y) = (-x, y - h)$. 1 and 3 by translation, $P_1$ by $(-w, 0)$ and $P_3$ by $(+w, 0)$ | $\{V_0, V_1, V_2, V_3\}$, order 1 | `aspect` |
| `*x` | $W_0$ to $W_3$ | 1 and 3 mirrors. 0 and 2 by the glide reflections of `xx` | Chains $\{V_0, V_2\}$ and $\{V_1, V_3\}$, order 1 | `aspect` |
| `22x` | $W_0$ to $W_3$ | 0 and 2 by the glide reflections of `xx`. 1 and 3 by glide reflection, $P_1(x, y) = (x - w, -y)$ and $P_3(x, y) = (x + w, -y)$ | $\{V_0, V_2\}$ and $\{V_1, V_3\}$, order 2 | `aspect` |
| `22*` | $V_0 = (0, -h/2)$, $V_1 = W_0$, $V_2 = W_1$, $V_3 = (0, h/2)$, $V_4 = W_2$, $V_5 = W_3$ | 0 and 1 by the half-turn about $V_0$. 3 and 4 by the half-turn about $V_3$. 2 and 5 mirrors | $\{V_0\}$ and $\{V_3\}$, order 2. Chains $\{V_1, V_5\}$ and $\{V_2, V_4\}$, order 1 | `aspect` |
| `2*22` | $V_0 = (0, -h/2)$, $V_1 = W_0$, $V_2 = W_1$, $V_3 = W_2$, $V_4 = W_3$ | 0 and 1 by the half-turn about $V_0$. 2, 3, and 4 mirrors | $\{V_0\}$, order 2. The corners $\{V_2\}$ and $\{V_3\}$, order 2. Chain $\{V_1, V_4\}$, order 1 | `aspect` |
| `4*2` | The four vertices of the kite of `442` of Section 6.8, the double of the triangle of `*442` | 0 and 1 mirrors. 2 and 3 by the rotation through a quarter turn about vertex 2 | The corner $\{V_0\}$, order 2. $\{V_2\}$, order 4. Chain $\{V_1, V_3\}$, order 1 | None |
| `3*3` | The four vertices of the kite of `632` of Section 6.8, the double of the triangle of `*632` | 0 and 1 mirrors. 2 and 3 by the rotation through a third of a turn about vertex 2 | The corner $\{V_0\}$, order 3. $\{V_2\}$, order 3. Chain $\{V_1, V_3\}$, order 1 | None |
| `2222` | The six vertices of Section 6.8, each mapped by the shear $(x, y) \mapsto (x + (k a) y, y)$, which is the identity at $k = 0$ | As in Section 6.8: 0 and 1 by the half-turn about $V_0$, 3 and 4 by the half-turn about $V_3$, 2 and 5 by translation, $P_2$ by $(-2w, 0)$ and $P_5$ by $(+2w, 0)$ | $\{V_0\}$, $\{V_3\}$, $\{V_1, V_5\}$, and $\{V_2, V_4\}$, each of order 2 | `aspect`, `skew` |

A pair of sides has the kind of its pairing: `Translation`, `Rotation` for a half-turn or another rotation, or `GlideReflection`. A mirror is its own partner with the kind `Mirror`. The orientation is `Reversing` for a mirror and a glide reflection and `Preserving` otherwise. The domains of `22*` and `2*22` split a side at the centre of a half-turn, so $V_0$ and $V_3$ of `22*` and $V_0$ of `2*22` are flat vertices, as $V_0$ and $V_3$ of `2222` are. The kites have no flat vertex. The angle sum over the states of every cycle of the table is $2\pi$ divided by its order (DOM-56). The state walk of DOM-55 visits each of the $2m$ states once.

The construction runs these steps:

1. Return the error of DOM-18, if one applies.
2. Build the base. For `**`, `xx`, `*x`, `22x`, `22*`, and `2*22`, build the rectangle $F$ of `*2222` by Section 6.5 at $a$. For `4*2` and `3*3`, build the kite of `442` or `632` by Section 6.8. For `2222`, build the double of Section 6.8 at $a$.
3. Set the vertices of the table. For `o`, compute $A$ and $B$ in `f64`, with the first coordinate of $B$ as the product $k w$, and each vertex as a sum or difference of $A$ and $B$ multiplied by 0.5.
4. Set the walls. A side of $F$ keeps the normal and the covector of $F$, bit for bit. Each half of a split side of `22*` and `2*22` takes the normal and the covector of the side of $F$ that it halves, bit for bit. A side of a kite keeps the normal and the covector of the kite. For `o`, compute each covector as `euclidean::wall_through(V_{j-1}, V_j)`, negate it when `euclidean::side` of it at `euclidean::ORIGIN` is positive, and take the normal $(c_1, c_2, 0)$, as step 6 of Section 6.5 does.
5. Set the pairings, the partners, the kinds, and the orientations of the table, each pairing by DOM-51.
6. Give each vertex the order of its cycle in the table and the copies 1, and give the domain the centre `euclidean::ORIGIN`. A kite instead keeps the centre of its double, which is the centre of its generating triangle bit for bit and the origin up to rounding (Section 6.5). Its corners are those of the table, with the copies 1, read from the double: vertex 0 has half the order of corner 0 of the double, 2 for `4*2` and 3 for `3*3`, vertex 2 has the order of corner 2 of the double, 4 or 3, and vertices 1 and 3 have the order 1.
7. For `2222` with $k$ not 0, map each vertex of the double by the shear, computed as `(k * a).mul_add(y, x)`, so that the product $k a$ is rounded and then the fused multiply-add rounds once. Compute each wall by `euclidean::wall_through` of the sheared vertices, oriented by `euclidean::ORIGIN` as in step 4, and set the pairings of the table by DOM-51, with the centres of the half-turns at the sheared $V_0$ and $V_3$ and the translations of the exact length $2w$ (Section 6.15). The partners, the kinds, the orientations, the corners with their orders and copies, and the centre `euclidean::ORIGIN` stay as Section 6.8 sets them.
8. Return `Numerical` if a call of step 4 or 7 returned an error, if any stored number is not finite, or if the value of `euclidean::side` at the centre is not strictly negative for every side.
9. Return `Numerical` unless the guard of DOM-54 passes.

The centre of `2*22` is the centre of $F$, not the mean of its five vertices, which is $(0, -h/10)$. The architecture states the centre of a wallpaper domain in `DEC-12` as amended. Every other rectangular domain and `o` have the mean of their vertices at the origin. The centre of `2222`, at every value of `skew`, is the origin, the centre of its generating rectangle. It is not the mean of its six vertices, because the second rectangle lies to the right of the first. A shear keeps the origin, horizontal lines, and areas, so the sheared sides 0, 1, 3, and 4 of `2222` stay horizontal and the area of the double stays 2. Every flat vertex of `22*`, `2*22`, and `2222` measures exactly $\pi$, as `core::f64::consts::PI`, which the tests assert (Section 9.8).

DOM-51. `build_with` MUST build each pairing of the ten domains in this way, and MUST NOT build a half-turn from `euclidean::rotation`:

- Mirror: `euclidean::reflection` of the covector of the side, as step 6 of Section 6.5 does.
- Translation: `euclidean::translation` of the vector of the table.
- Glide reflection: the product of `euclidean::translation(v)` and `euclidean::reflection(c)`, in that order, for the vector $v$ of the table and the covector $c$ of the axis through the origin, $(1, 0, 0)$ for the axis $x = 0$ of the glides of sides 0 and 2, and $(0, 1, 0)$ for the axis $y = 0$ of the glides of sides 1 and 3 of `22x`. Each entry is a sum of products of 0, 1, $-1$, and a coordinate of $v$, so the product is exact in `f64`.
- Half-turn about its centre $(c_x, c_y)$, the vertex that the side shares with its partner: the matrix with the rows $(-1, 0, 2c_x)$, $(0, -1, 2c_y)$, and $(0, 0, 1)$, computed as the product of `euclidean::reflection` of the covector $(0, 1, -c_y)$ of the horizontal line through the centre and `euclidean::reflection` of the covector $(1, 0, -c_x)$ of the vertical line through the centre, by the compensated product of DOM-25. Every entry is $0$, $\pm 1$, $2c_x$, or $2c_y$, and is exact. The half-turn takes no covector from the wall of its side.
- Kite: each mirror side gets `euclidean::reflection` of its own stored covector, $c_0$ for side 0 and $R_1^{T} c_0$ for side 1, so that the reflection term of check 3 is exactly 0. The reflection of side 0 equals the reflection that the generating triangle stores, bit for bit. Side 2 keeps the compensated product $R_2 R_1$ and side 3 keeps $R_1 R_2$, as step 5 of Section 6.8 gives them, bit for bit as in the doubles of `442` and `632`.

The reasons are that exact entries keep the residuals of checks 1 to 4 at the rounding of the vertices and walls alone, that the tests of periodicity of `escher-render` compare images exactly where every value is dyadic, and that the sine of `euclidean::rotation(PI)` is about $1.2 \times 10^{-16}$ and not 0.

A half-turn is built from the lines through its centre and not from the wall of its side, because the wall of a sheared side of `2222` comes from `euclidean::wall_through`, whose offset differs from $-c_y$, the second coordinate of the centre negated, at some sheared settings. A half-turn built from such a wall would not be the half-turn about the stored centre. That consequence is a reading of this design. For `22*`, `2*22`, and `2222` at `skew` 0, the horizontal line through the centre equals the wall of the side, so the two constructions give the same matrix bit for bit. The unit tests assert that every half-turn equals the matrix of its stored centre.

The kites reuse the doubles of `442` and `632`, so their placement carries over, and so does their accuracy of Section 9.6 except for side 1 (Section 9.8).

DOM-54. `build_with` MUST return `Numerical` for each of the ten domains, including `2222` at every value of `skew`, unless `verify().passed()` holds for the built domain, because `escher render` folds into a domain without calling `verify()`, the constructions rest on `UNV-11` and `UNV-33` of the plan, and a wrong domain would otherwise reach the renderer. The guard runs all six checks once, after step 8, and costs a few dozen products of 3 by 3 matrices, because the largest cycle order is 4 (Section 8.2). It is `DEC-53` of the architecture.

Every setting of the grid of Section 8.4 passes every check by a wide margin, so the guard is expected to refuse nothing in the ranges of Section 6.6. The tests reach both outcomes of the guard (Section 12.1).

The construction ports nothing from SymmHub. Each domain is a rectangle, a parallelogram, a split rectangle, a kite, or a sheared double of this crate, with pairings written from their definitions. A port of SymmHub's catalogue served only as a comparison: the groups that the port builds classify as `o`, `**`, `xx`, `*x`, `22x`, `4*2`, `3*3`, and `2222`, the same groups as those of this section up to a quarter turn and a scale. SymmHub's functions for `22*` and `2*22`, which its dispatch never reaches, classify as `22*` and `2*22` when they are called directly, while the dispatch sends `22*` to the function of `22` and `2*22` to that of `*22` (Section 7.1.9 of `docs/research/symmhub.md`).

### 6.14. Theorems That Make the Wallpaper Domains Fundamental Domains

These sourced theorems and unverified statements bear on the ten domains. This document adds no proof (Hard Rule 14 of `AGENTS.md`):

| Statement | Source | Status |
| --- | --- | --- |
| The 17 Euclidean signatures and their groups | Table II of [CH] and [TH] 13.3.6 (Section 9.5 of the domain reference) | Sourced |
| The torus `o` is a parallelogram with opposite sides identified by translations | [CH] p. 250, and Exercise 9.5(1) of [RATCLIFFE], p. 399 (Section 9.5.2 of the domain reference) | Sourced. The skewed parallelogram is a member of this family |
| The Klein bottle `xx` is a square glued by the translation $z \mapsto z + 1$ and the glide reflection $z \mapsto -\bar z + 1 + i$ | [RATCLIFFE] p. 399 (Section 9.5.2 of the domain reference) | Sourced. That the domain of `xx` of Section 6.13 is this square after centring and scaling is a reader's comparison |
| The orbifolds of `*x` and `2*22` from strips and squares of `*2222` | [CH] pp. 249 to 251 (Section 9.5.2 of the domain reference) | Sourced as orbifolds, with no polygon and no pairings as formulas |
| The kite reflected in its axis as the cell of `4*2` and `3*3` | [VONGAGERN2009] p. 12 (Section 9.5.2 of the domain reference) | Sourced as a shape. The pairings as formulas have no source |
| `2222` as two rectangles with half-turns | Example 13.1.4 of [TH] (Section 9.5.2 of the domain reference) | Sourced at `skew` 0, as a reader's comparison. The sheared domain has no source |
| The domains of `**`, `22*`, and `22x` with their pairings, and the pairings of `*x`, `2*22`, `4*2`, and `3*3` as formulas | None (Section 9.5.2 of the domain reference) | `UNV-33` of the plan, and item 17 of Section 13.3 of the domain reference |
| A convex polygon with pairings that meet Pairing, Connected, Cyclic, and Finite is a fundamental domain of the discrete group that they generate, in the Euclidean plane | Theorem 4.14 with (k) of [EPSTEIN-PETRONIO1994], and Theorem 13.5.3 of [RATCLIFFE] (Sections 4.9 and 4.10 of the domain reference) | Sourced. `verify()` checks the hypotheses on every build (DOM-54) |
| The product of the pairings along a cycle is a rotation through the angle sum of the cycle, which must be $2\pi/m$, and two mirrors at a corner give $2\theta = 2\pi/m$ | Conditions 3.7, 3.10, and 3.13, Lemma 3.12, and Example 3.15 of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference) | Sourced. The conditions put no condition on the orientation of a pairing |
| The group that the pairings of each domain generate has its signature | None in orbifold notation | `UNV-11` of the plan. The wallpaper certificate checks it numerically (DOM-57) |
| The product of the reflections in two lines through the origin at the angle $\theta$ is the rotation through $2\theta$ | Example 3.1.2 of [DAVIS] (Section 12 of the geometry reference) | Sourced for lines through the origin. The half-turn about the origin is a reader's substitution of $\theta = \pi/2$. For a half-turn about another point, item 31 of Section 15.1 of the geometry reference |
| A glide reflection is a reflection in a line composed with a translation along it, and reflections and glide reflections are the only orientation-reversing isometries of the plane | Section 12 of the geometry reference, from a secondary source | Item 32 of Section 15.1 of the geometry reference |
| Each shear of the domain of `o` or `2222` is a fundamental domain of a group with the same signature, and the range of `skew` reaches every lattice | None | `UNV-33` of the plan, and item 18 of Section 13.3 of the domain reference |

The checklist of Section 4.8 of the domain reference applies to the ten domains in this way:

| Hypothesis | How the design meets it |
| --- | --- |
| Each side is paired with itself by its reflection or with exactly one other side, and the partner's pairing is the inverse (item 1 of Section 4.8, Condition 3.2 (a) and (d)) | Checked: check 2 |
| A pairing maps the interior to the other side of the partner side and maps the side onto its partner, with the endpoints exchanged for a preserving pairing and in order for a reversing one (Condition 3.2 (b) and (c)) | Checked: check 2, with the endpoint terms of Section 8.2 and the wall term |
| No side other than a mirror is paired with itself | Checked: check 2 (DOM-26 and DOM-27) |
| Each pairing is an isometry | Checked: check 1 |
| The pairing of a mirror is the reflection in its wall | Checked: check 3 |
| Every cycle is finite, its transformation fixes its start vertex, and its power to the cycle order is the identity (item 3 of Section 4.8, Conditions 3.7, 3.10, and 3.13) | Checked: check 4 (DOM-55) |
| The angle sum over the states of a cycle is $2\pi$ divided by its order | Checked: check 5 (DOM-56) |
| The polygon is convex | Checked numerically: the convexity term of check 3 |
| The area agrees with the Euler characteristic | Checked for consistency only: check 6 |

### 6.15. Choices of the Wallpaper Construction

The design considered these alternatives:

| Choice | Alternative | Verdict and reason |
| --- | --- | --- |
| The kite of `442` and `632` for `4*2` and `3*3` | SymmHub's right triangle with two sides paired and the hypotenuse a mirror, which Figure 4 of [VONGAGERN2009] also draws for `4*2` | Kite. It is the kite of Section 6.8 with two pairings exchanged for mirrors, so it reuses `double()`, its placement, and its measured accuracy. It sits where `*442` and `*632` sit, so the view of `DEC-12` carries over. It has no flat vertex, while SymmHub's triangle has a mirror side through a vertex. No source describes SymmHub's triangle for `3*3` (Section 9.5.2 of the domain reference). The cost is that `4*2` with one fill colour draws the borders of `442`. Item 43 of Section 15.3 puts it to the owner |
| The parallelogram for `o` | A rectangle with a shifted translation | Parallelogram. The rectangle needs split sides whose number changes at `skew` 0, which breaks DOM-37. The parallelogram has four sides at every setting, each side maps onto a side, and its fold is the affine image of the fold of the rectangle |
| The rectangle $F$ of `*2222`, with the area 1, for the six rectangular groups | Tiles nested in the grid of `*2222` with the areas 2 and 4, or SymmHub's horizontal layout | $F$. Every tile has the area 1, `aspect` means what it means for `*2222`, and every side lies on a line of the grid of `*2222` at the same `aspect`. Item 44 of Section 15.3 puts it to the owner |
| Mirror lines and glide axes vertical where there is a choice | Horizontal, as in SymmHub | Vertical. The domain of `xx` then matches the square of [RATCLIFFE] p. 399 after centring. `aspect` and its reciprocal cover both up to a quarter turn |
| Half-turn sides split at the centre, with a flat vertex, for `22*` and `2*22` | SymmHub's sides that are their own partners | Split, as DOM-26 requires and as `2222` does. The numbering of `22*` follows that of `2222`: the pairs 0 and 1 and 3 and 4, with sides 2 and 5 between them |
| `skew` of `2222` by a shear of its double | SymmHub's offset of the centres of the half-turns, or a new tile | Shear. The half-turns of `2222` sit on its horizontal sides, and a centre moved off the midpoint leaves a side that is not mapped onto a side (Section 7.1.7 of `docs/research/symmhub.md`). A shear keeps every side on its partner, keeps the horizontal sides horizontal, and keeps the numbering and the ids. Item 45 of Section 15.3 puts the convention to the owner |
| Half-turns as exact matrices | `euclidean::rotation(PI)` | Exact matrices (DOM-51) |
| Half-turn as the product of the reflections in the horizontal and the vertical line through its centre | The product of the reflection in the stored wall of its side and the reflection in the vertical line | The lines through the centre. The wall of a sheared side of `2222` is not always exactly the horizontal line through the centre (Section 6.13). At `skew` 0 both give the same matrix |
| A translation of the sheared `2222` with the exact length $2w$ and its direction from the vertices of the double | A vector taken wholly from the sheared vertices, such as $V_5 - V_1$ | The exact length. A temporary check found that the vector from the vertices differs from $\mp 2w$ by one unit in the last place at some settings, because the vertex coordinate $1.5 w$ is rounded. With the exact length, every pairing equals the translation of the table of DOM-50 bit for bit |
| The shear as one fused multiply-add | A product with $y$ and a sum, each rounded | Fused multiply-add. It saves the rounding of the product with $y$ |
| Guard on the ten domains | No guard, as for the other Euclidean domains | Guard (DOM-54), for the soundness reason that it states |

With the parity fill, `**` and `22*` give the same image, and so do `*x`, `22x`, and `*2222`, and with one fill colour `4*2` gives the image of `442` and `3*3` that of `632`, as the tests of `escher-cli` assert. A colouring by cosets tells the groups apart. `docs/design/render.md` owns the consequences for the tests and the owner's decision on the borders of mirror sides (`DEC-55` of the architecture).

### 6.16. Spherical Group Construction

For a spherical group signature, `build_with` runs steps 1 to 5 of DOM-18, computes the area of the signature as step 1 of Section 6.1 does, and then runs the steps of this section. The conventions of Section 5 hold. Each domain reuses a construction of Section 6.7 or 6.8 and keeps its placement, so the view of `DEC-12` carries over: a spherical kite sits where its double and its starred triangle sit, and the lunes have the centre `spherical::ORIGIN`. The architecture states the centre of the lune of order 1 and of the whole sphere in `DEC-12` as amended. This section ports nothing from SymmHub (Section 6.18).

DOM-58. `build_with` MUST give each spherical group domain the base, the vertices, the sides, the partners, the kinds, the orientations, the vertex orders, and the closure of this table, with the numbering that it states, for the reasons of DOM-50, which hold here unchanged. $W_0$ to $W_3$ are the vertices of the kite of the double of Section 6.8, with $W_0$ the corner of the largest order of the generating triangle and $W_2$ its second corner. $V_0 = (0, 1, 0)$ and $V_1 = (0, -1, 0)$ are the vertices of the lune of Section 6.7. "Order" is the cycle order of Section 2.1:

| Signature | Base | Vertices | Sides | Vertex cycles, with their orders | Stored orders | Closure |
| --- | --- | --- | --- | --- | --- | --- |
| `3*2` | The double of `*432` of Section 6.8, the kite of `432` | $W_0$ to $W_3$ | 0 and 1 mirrors, `Low`. 2 and 3 by the rotation through $2\pi/3$ about $W_2$, the kind `Rotation` | The corner $\{W_0\}$ between the mirrors, order 2. $\{W_2\}$, order 3. Chain $\{W_1, W_3\}$, order 1 | $[2, 1, 3, 1]$ | `Cycles` |
| `2*n` | The double of `*(2n)22`, the kite of `(2n)22` | $W_0$ to $W_3$ | 0 and 1 mirrors, `Low`. 2 and 3 by the half-turn about the flat vertex $W_2$, the kind `Rotation` | The corner $\{W_0\}$, order $n$. $\{W_2\}$, order 2. Chain $\{W_1, W_3\}$, order 1 | $[n, 1, 2, 1]$ | `Cycles` |
| `n*` | The double of `*n22`, the kite of `n22` | $W_0$ to $W_3$ | 0 and 1 by the rotation through $2\pi/n$ about $W_0$, the kind `Rotation`. 2 and 3 mirrors, `High`, which meet at the flat vertex $W_2$ | $\{W_0\}$, order $n$. Chain $\{W_1, W_3\}$, order 1. The flat corner $\{W_2\}$ between the two mirrors, order 1 | $[n, 1, 1, 1]$ | `Cycles` |
| `nx` | The lune of `*nn` of Section 6.7 | $V_0$, $V_1$ | 0 and 1 by rotary reflection, the kind `RotaryReflection` | $\{V_0, V_1\}$, order $n$ | $[n, n]$ | `Cycles` |
| `x` | The lune of order 1 | $V_0$, $V_1$, both flat | 0 and 1 by the antipodal map, the kind `RotaryReflection` | $\{V_0, V_1\}$, order 1 | $[1, 1]$ | `Cycles` |
| `*` | The lune of order 1 | $V_0$, $V_1$, both flat | 0 and 1 mirrors | The corners $\{V_0\}$ and $\{V_1\}$, order 1 | $[1, 1]$ | `PerVertex` |
| `1` | None | None | None | None | None | `PerVertex` |

A pair of sides has the kind of its pairing: `Rotation` for a rotation or a half-turn, and `RotaryReflection` for a rotary reflection, which includes the antipodal map. A mirror is its own partner with the kind `Mirror`. The orientation is `Reversing` for a mirror and a rotary reflection, and `Preserving` for a rotation. Every vertex has the copies 1, which no check of the closure `Cycles` reads. These are the states of each cycle as the walk of DOM-55 visits them, as a start vertex with its direction, $F$ for $+$ and $B$ for $-$, and the sides that it applies:

| Domain | Cycles, as the order, the states, and the sides |
| --- | --- |
| `3*2` | 2: $(0,F)(0,B)$, sides 1 0. 1: $(1,F)(3,F)(3,B)(1,B)$, sides 2 0 3 1. 3: $(2,F)$, side 3 |
| `2*n` | $n$: $(0,F)(0,B)$, sides 1 0. 1: $(1,F)(3,F)(3,B)(1,B)$, sides 2 0 3 1. 2: $(2,F)$, side 3 |
| `n*`, and `2*` | $n$: $(0,F)$, side 1. 1: $(1,F)(1,B)(3,B)(3,F)$, sides 2 1 3 0. 1: $(2,F)(2,B)$, sides 3 2 |
| `nx`, and `x` | $n$: $(0,F)(1,B)$, sides 1 1, with the angle sum $2\pi/n$ |
| `*` | The all-mirror path of DOM-28, with the corner orders $[1, 1]$ |
| `1` | None |

The angle sum over the states of each cycle is $2\pi$ divided by its order, with the walk counting each vertex that it visits in both directions twice: the chain $\{W_1, W_3\}$ gives $4 \cdot \pi/2 = 2\pi$, and the flat corner $\{W_2\}$ of `n*` gives $2\pi$. The walk visits each of the $2m$ states once with its flip, within $2m$ steps. The lune of `nx` closes in two steps with $g = P_1 P_1$, the chains of the kites in four, and the flat corner of `n*` in two.

The corner angles are those of the base: 90, 90, 120, and 90 degrees for `3*2`, $180/n$, 90, 180, and 90 degrees for `2*n`, $360/n$, 90, 180, and 90 degrees for `n*`, $180/n$ at both vertices for `nx`, and 180 degrees at both vertices for `x` and `*`. The area of each domain equals that of its base: $\pi/6$ for `3*2` and `432`, $\pi/n$ for `2*n` and `(2n)22`, $2\pi/n$ for `n*` and `n22`, $2\pi/n$ for `nx` and `*nn`, and $2\pi$ for `x` and `*`. The whole sphere has the area $4\pi$.

These are the flat vertices: $W_2$ of `2*n` and of `n*`, and also $W_0$ of `2*`, the kite of `222`, whose sides 0 and 1, the pair of the half-turn, lie on one great circle, and both vertices of `x` and of `*`. The two mirror sides of `n*` lie on one great circle, the equator of its base, up to rounding, so they are not one wall bit for bit. Only the two sides of `*` and of `x`, with the exact normals of step 2, are one wall bit for bit. The flat vertex $W_2$ of `n*` splits the equator at a point that the group does not fix, so its position is a choice of this design, which `verify()` does not check: a small move of $W_2$ along the equator passes all six checks. The renderer never reads a vertex, so the choice does not change an image.

The construction runs these steps:

1. Return the error of DOM-18, if one applies.
2. Build the base. For `3*2`, `2*n`, and `n*`, build the double of `*432`, `*(2n)22`, or `*n22` by Section 6.8, with the private helper `FundamentalDomain::doubled`, as for the wallpaper kites. For `nx`, build the lune of `*nn` by Section 6.7 at the order $n$. For `*` and `x`, build the lune of order 1, whose step 1 of Section 6.7 sets $s = 1$ and $c = 0$ exactly, so that both sides have the normal $(0, 0, -1)$ exactly. For `1`, build nothing.
3. For a kite, run `on_kite` of `src/kite.rs` on the double, with the mirror pair `Low` for `3*2` and `2*n` and `High` for `n*`. Each mirror side gets its normal and its covector from the double bit for bit, the pairing of DOM-59, its own index as its partner, the kind `Mirror`, and the orientation `Reversing`. The other two sides keep the normal, the covector, the pairing, the partner, the kind, and the orientation of the double bit for bit. The vertices and the centre are those of the double bit for bit. The corners have the copies 1 and the orders of this rule, with $c_0$ and $c_2$ the orders of corners 0 and 2 of the double: for `Low`, $[c_0/2, 1, c_2, 1]$, and for `High`, $[c_0, 1, 1, 1]$. The rule `Low` is the rule of step 6 of Section 6.13, so the wallpaper kites keep their corners.
4. For `nx` and `x`, keep the vertices, the normals, the covectors, and the centre of the lune bit for bit, and give side $i$ the pairing of DOM-59, the partner $1 - i$, the kind `RotaryReflection`, and the orientation `Reversing`. Give both corners the order $n$, or 1 for `x`, and the copies 1.
5. For `*`, keep the lune of order 1 as Section 6.7 builds it, with its mirror sides, and give both corners the order 1 and the copies 1.
6. For `1`, build a domain with the geometry `Spherical`, no side, no vertex, no corner, the centre `spherical::ORIGIN`, and the area of the signature, $4\pi$.
7. Give the domain the closure of the table of DOM-58.
8. Return `Numerical` if a call of `escher-geometry` returned an error, if any stored number is not finite, or if the value of `spherical::side` at the centre is not strictly negative for every side.
9. Return `Numerical` unless the guard of DOM-60 passes, for every domain other than that of `*`.

DOM-59. `build_with` MUST build each pairing of a spherical group domain in this way:

- Mirror: `spherical::reflection` of the normal that the side stores. On a kite with `Low`, side 0 has the normal $u_0$ of the generating triangle, so its reflection equals the reflection that the triangle stores bit for bit, and side 1 has the normal $R_1 u_0$ of its image. On a kite with `High`, side 3 has the normal $u_2$ of the generating triangle and side 2 the normal $R_1 u_2$. Each mirror has the reflection of its own wall, so the reflection term of check 3 is exactly 0.
- Kept pairing of a kite: the compensated product of DOM-25, $R_1 R_j$ or $R_j R_1$, bit for bit that of the double.
- Rotary reflection of `nx` and `x`: $P_i = H R_i$, with $H = \operatorname{diag}(-1, -1, 1)$ and $R_i$ the reflection that the lune stores for side $i$, computed by negating rows 0 and 1 of $R_i$. Negation is exact, so $P_i$ is the exact product of $H$ and the stored reflection.
- `x`: the lune of order 1 has $R_0 = R_1 = \operatorname{diag}(1, 1, -1)$ exactly, so $P_0 = P_1 = -I$, the antipodal map, exactly.
- `*`: the pairing of each side is $\operatorname{diag}(1, 1, -1)$ exactly.

The reasons are that the kept entries keep the accuracy that Sections 9.5 and 9.6 measure for the bases, that an exact product of $H$ and a stored reflection adds no rounding to the pairing of `nx`, and that the exact pairings of `x` and `*` make the claims of DOM-58 and the byte equalities of `escher-render` exact.

That $H R_i$ maps the neighbouring copy of the lune across side $i$ onto the lune is a construction of this design that no source states (`UNV-34` of the plan). Check 2 tests it on every build, through the endpoint term and the wall term, and the certificate of DOM-64 tests the group that the pairings generate.

DOM-60. `build_with` MUST return `Numerical` for the domains of `3*2`, `2*n`, `n*`, `nx`, `x`, and `1` unless `verify().passed()` holds for the built domain, with the text `the spherical domain fails its verification`, because `escher render` folds into a domain without calling `verify()`, the constructions rest on `UNV-34` of the plan, and a wrong domain would otherwise reach the renderer. The domain of `*` is a lune of Section 6.7 with mirror sides, so it stays unguarded like every lune of `*nn`. The guard runs all six checks once, after step 8, and costs at most about 3000 matrix products at the limits, as `verify()` of a double does (Section 8.2). It is `DEC-60` of the architecture.

Every spherical group domain in the range of Section 4 passes every check by a wide margin (Section 8.4), so the guard is expected to refuse none. The tests reach both outcomes of the guard through private access (Section 12.1).

A port of SymmHub's catalogue outside the repository served only as a comparison, and nothing in this section comes from it. SymmHub builds `2*n` as the triangle of `*22n` with its equator side kept whole and fixed by a half-turn, which is the shape of the kite of `(2n)22` with that side not split, `3*2` as a triangle with the angles $2\pi/3$, $\pi/4$, and $\pi/4$, `n*` as the triangle of `n22` with the equator side as one mirror, and `nx` as a lune paired by a rotation through $\pi/n$ and a reflection in the equator (Section 9.6.1 of the domain reference). The port found the pairing of SymmHub's `x` to be the antipodal map, as the pairing of `x` here is. SymmHub builds no domain for `1`, because its parser rejects the order 1.

### 6.17. Theorems That Make the Spherical Group Domains Fundamental Domains

These sourced theorems and unverified statements bear on the spherical group domains. This document adds no proof (Hard Rule 14 of `AGENTS.md`):

| Statement | Source | Status |
| --- | --- | --- |
| The 14 spherical families and their orders $2/\chi$ | Table III of [CH] and [TH] 13.3.6, and Table 4.1 of [CBG] (Section 9.6 of the domain reference) | Sourced |
| The coincidences `2*1 = 2*` and `*11 = 1* = *` of the infinite series | The Appendix of [CH], pp. 256 to 257 (Section 9.6.2 of the domain reference) | Sourced |
| The presentation of `n*`, $1 = \alpha^N = P^2$ and $\alpha P = P \alpha$ | [CBG] p. 130 (Section 9.6.2 of the domain reference) | Sourced. A picture of the domain only, [CBG] p. 56 |
| The orbifold of `2*n` and `3*2` as a disk with one cone point in the interior and one kaleidoscopic point on the boundary, and "similar presentations" to `4*2` | [CBG] pp. 129 and 187 (Section 9.6.2 of the domain reference) | Sourced as an orbifold, with no formula of the presentation and no polygon |
| The orbifold of `*` is a hemisphere | [CH] p. 248 (Section 9.6.2 of the domain reference) | Sourced as an orbifold. The two sides of the lune of order 1 are a choice of this design |
| The kite as the cell of a group with a cone point and a corner reflector | [VONGAGERN2009] p. 12 (Section 9.5 of the domain reference) | Sourced as a shape, in the Euclidean plane only |
| The domains of `3*2`, `2*n`, `n*`, `nx`, `x`, `*`, and `1` with their pairings and vertex cycles, the walk of DOM-55 on a lune with reversing pairings, and the census of each group | None (Sections 4.7 and 9.6.2 and items 19 and 20 of Section 13.3 of the domain reference) | `UNV-34` of the plan. The certificate of DOM-64 checks them numerically |
| The text `1` of the trivial signature | None | `UNV-20` of the plan |
| A convex polygon of $S^2$ with pairings that meet Pairing, Connected, Cyclic, and Finite is a fundamental domain of the discrete group that they generate | Theorem 4.14 with (k) of [EPSTEIN-PETRONIO1994], and Theorem 13.5.3 of [RATCLIFFE] (Sections 4.9 and 4.10 of the domain reference) | Sourced for a convex polyhedron. Whether the theorems admit a lune, and so the domains of `nx`, `x`, and `*`, is not established, as for `UNV-29` and `UNV-30`. `verify()` checks the hypotheses on every build |
| The product of the pairings along a cycle is a rotation through the angle sum of the cycle, which must be $2\pi/m$, and two mirrors at a corner give $2\theta = 2\pi/m$ | Conditions 3.7, 3.10, and 3.13, Lemma 3.12, and Example 3.15 of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference) | Sourced. The conditions put no condition on the orientation of a pairing |
| A mirror side split at a flat vertex, as for `n*`, `x`, and `*` | Definition 2.8 of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference) | A reader's reading, recorded with `UNV-31`. No source works the example |
| The orbifold of a cross-cap with a cone point, `nx` | None. No source read covers it (Section 4.7 of the domain reference) | `UNV-34` of the plan |
| Every orientation-reversing isometry of the sphere is a rotary reflection, with the antipodal map as its case of the angle $\pi$ | [RAT] Theorem 5.4.2 in matrix form, which names no rotary reflection (Section 11.1 of the geometry reference) | Item 33 of Section 15.1 of the geometry reference |
| The group that the pairings of each domain generate has its signature | None in orbifold notation | `UNV-34` of the plan. The spherical certificate checks it numerically (DOM-64) |

The checklist of Section 4.8 of the domain reference applies to the spherical group domains as it applies to the ten domains of Section 6.14, with these readings:

| Hypothesis | How the design meets it |
| --- | --- |
| Each side is paired with itself by its reflection or with exactly one other side, and the partner's pairing is the inverse | Checked: check 2 |
| A pairing maps the interior to the other side of the partner side and maps the side onto its partner, with the endpoints exchanged for a preserving pairing and in order for a reversing one | Checked: check 2, with the endpoint terms of Section 8.2 and the wall term |
| No side other than a mirror is paired with itself | Checked: check 2 (DOM-26 and DOM-27) |
| Each pairing is an isometry | Checked: check 1 |
| The pairing of a mirror is the reflection in its wall | Checked: check 3 |
| Every cycle is finite, its transformation fixes its start vertex, and its power to the cycle order is the identity | Checked: check 4 (DOM-55), and for `*` the corner relators of DOM-28 |
| The angle sum over the states of a cycle is $2\pi$ divided by its order | Checked: check 5 (DOM-56), and per vertex for `*` |
| The polygon is convex | Checked numerically: the convexity term of check 3 and, for the lunes, the term $V_0 + V_1$ |
| The area agrees with the Euler characteristic | Checked for consistency only: check 6 |
| The domain with no side | Every hypothesis holds vacuously. Check 6 compares the area $4\pi$ (DOM-62) |

### 6.18. Choices of the Spherical Group Construction

The design considered these alternatives:

| Choice | Alternative | Verdict and reason |
| --- | --- | --- |
| The kite of `432` for `3*2` | SymmHub's triangle with the angles $2\pi/3$, $\pi/4$, and $\pi/4$ | Kite. It needs no new geometry, sits where `*432` sits, and keeps the measured accuracy of its double. The cost is that `3*2` with one fill colour draws the borders of `432` |
| The kite of `(2n)22` for `2*n` | A split triangle of `*n22`, or SymmHub's triangle of `*22n` with an unsplit equator side fixed by a half-turn | Kite. The unsplit side is its own partner, which DOM-26 forbids. A split triangle of `*n22` would reach $n = 1000$ but needs a new construction and new measurements. The kite forces the limit of DOM-63 |
| `n*` as the kite of `n22` with four sides, the two halves of the equator as two mirrors | Three sides, with the equator as one mirror side, as SymmHub does | Four sides. It is the existing kite, so `--colouring one` gives the picture of `n22`. The measurement of a domain with three vertices by the law of cosines breaks at $n = 2$, where the two vertices on the equator are antipodal and the side between them has the length $\pi$ and the sine 0. Item 50 of Section 15.3 puts it to the owner (`DEC-62` of the architecture) |
| The generalized `on_kite` of `src/kite.rs`, over the geometry and the mirror pair | Reuse of the wallpaper `on_kite` as it is, or a second builder for the sphere | Generalized. The wallpaper builder takes the Euclidean plane, `Side::euclidean_mirror`, and the pair 0 and 1, so it cannot build `n*`. One builder keeps one rule for the corners of every kite |
| The lune of `*nn` with the pairings $H R_i$ for `nx` and `x` | SymmHub's lune between the angles $\pm\pi/(2n)$, paired by a rotation through $\pi/n$ and a reflection in the equator | The lune of `*nn`. It is the built lune bit for bit, so its placement and accuracy carry over, and each pairing is an exact product of a stored reflection |
| The kind `RotaryReflection` | The kind `GlideReflection` on the sphere, which needs no new variant | A new kind, so that `escher info` prints no false name of a pairing. This is `DEC-57` of the architecture |
| The plane rule of check 2: `RotaryReflection` only on a spherical domain, and `GlideReflection` only off it | No rule on the plane of a kind | The plane rule. Without it, `x` with the kind `GlideReflection` passed every check and every enumeration on a prototype (Section 12.1) |
| `*` and `x` as lunes of order 1 with two sides | One side with no vertex | Two sides. Every check and the renderer assume as many vertices as sides. Item 49 of Section 15.3 puts it to the owner (`DEC-62` of the architecture) |
| Exact normals at the order 1 | The lune of Section 6.7 as it stands, with $c = 6.1 \times 10^{-17}$ | Exact normals, so that `x` has exactly $-I$ and `*` exactly one wall. A prototype found every check and every image the same without them |
| `1` built as the whole sphere with no side | `1` left unsupported | Built. It costs two changes of `verify()` and `area()` (DOM-62), and every valid signature then has a domain. Item 47 of Section 15.3 puts it to the owner (`DEC-58` of the architecture) |
| The limit of 500 for `2*n` with its own text | A limit of 1000 with a statement of accuracy for spherical triangles of an order up to 2000 | 500 (DOM-63). Item 48 of Section 15.3 puts it to the owner (`DEC-59` of the architecture) |
| The closure `Cycles` for `nx` and `x` | `PerVertex` | `Cycles`, so that every spherical group domain with a paired side uses one rule. On a prototype, both closures passed `nx` and `x` with the same relator texts, while the kites failed with `PerVertex` |
| The guard on every spherical group domain other than `*` | No guard, as for the other spherical domains | Guard (DOM-60), for the reason that it states. `*` is a lune of Section 6.7 with mirror sides |
| The census as a mandatory part of the certificate | Order and enumeration alone | The census (DOM-64), because five domains of wrong groups pass `verify()` and enumerate to the right order (Section 12.1) |

## 7. Measurements

The crate measures each quantity from the stored vertices, so that `verify()` and `escher-cli` see the same numbers:

- `side_lengths()` returns `hyperbolic::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the angle $\theta_j$ in $[0, \pi]$ for which the first law of cosines of Section 10.2 of the geometry reference ([BEA] Section 7.12, p. 148, and [RAT] Theorem 3.5.3, p. 86) holds with the three measured side lengths. With $c$ the length of the side opposite vertex $j$ and $a$ and $b$ the other two, it is $\arccos\big((\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)\big)$, with the argument clamped to $[-1, 1]$.
- `area()` returns $\pi - \theta_0 - \theta_1 - \theta_2$, the area of a triangle by Section 10.1 of the geometry reference.

For a hyperbolic domain, these formulas apply to three vertices. Four vertices, of a kite or of a polygon with four corners, are measured as two triangles (DOM-29 and DOM-46), and five or more vertices, of a polygon or of the double of a polygon, by DOM-46. For a hyperbolic domain with fewer than three vertices, which only private access builds, `corner_angles()` returns NaN for every vertex and `area()` returns NaN.

DOM-46. For a hyperbolic domain with four vertices, `corner_angles()` and `area()` MUST use the two triangles of DOM-29, for a polygon with four corners as for a kite. For a hyperbolic domain with five or more vertices, `corner_angles()` MUST measure the angle at vertex $j$ in its corner frame, as $\theta_j = \operatorname{atan2}(\lvert a_1 b_2 - a_2 b_1 \rvert, a_1 b_1 + a_2 b_2)$ for the first two coordinates $a$ of $T V_{j+1}$ and $b$ of $T V_{j-1}$, with $T =$ `hyperbolic::inverse(hyperbolic::translation(V_j))`, at every vertex, including a vertex with the copies 2 of a double. `area()` MUST return $(m - 2)\pi - \sum_j \theta_j$ for five or more vertices. The reasons are that the formulas of the triangle measure no polygon, that the triangles and the kites keep their values bit for bit (DOM-30), that the corner frame gives a smaller residual of check 5 than the law of cosines in the triangle of the two neighbours, and that the corner frame stays well conditioned at a flat vertex, where the law of cosines gives a residual above the tolerance of check 5 at the double of `*22222222` (Section 6.12). At a vertex with the copies 2, one angle in the corner frame is as accurate as the sum of two angles across the diagonal from $W_0$ to $W_{n-1}$ and needs no layout of the double, so DOM-46 uses it (Section 6.12).

The angle in the corner frame uses only the vertices, as DOM-6 requires. That it equals the hyperbolic angle at the vertex rests on the conformality of the disk at its origin and on the geodesics through the origin being diameters ([CFKP], p. 80, in Section 3.1 of the geometry reference), which the design reads as giving the angle between the first two coordinates of the two neighbours in the frame. No source states this reading for the hyperboloid, and Section 15.1 records it. The area is Theorem 7.15.1 of [BEA], p. 153 (Section 10.1 of the geometry reference). For a double with a flat vertex, the formula counts that vertex with the angle $\pi$, which adds $\pi$ to both terms, and Section 15.1 records that reading.

The angles are measured from distances so that no step uses a relation between normals and angles (DOM-6). The form of the first law solved for the angle is a rearrangement that no source prints, and Section 15.1 records it.

For a Euclidean domain, with $(x_j, y_j)$ the first two coordinates of $V_j$:

- `side_lengths()` returns `euclidean::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the Euclidean angle $\theta$ of equation (1.3.3) of [RAT] (Section 12 of the geometry reference) between $V_{j+1} - V_j$ and $V_{j-1} - V_j$, as $\arccos$ of their dot product divided by the product of their lengths, with the argument clamped to $[-1, 1]$. It uses no normal, as DOM-6 requires.
- `area()` returns $\frac{1}{2} \left\lvert \sum_j (x_j y_{j+1} - x_{j+1} y_j) \right\rvert$, with indexes modulo $m$. No source states this formula (item 25 of Section 15.1 of the geometry reference), and Section 14 records it.

For a spherical domain, the measurements read the theorems of [RAT], Section 2.5 (Section 11.3 of the geometry reference), with lengths and areas in the metric of the unit sphere:

- `side_lengths()` returns `spherical::distance(V_{j-1}, V_j)` for each side $j$. Both sides of a lune have the length $\pi$.
- `corner_angles()` returns, for each vertex $j$ of a triangle, $\arccos\big((\cos c - \cos a \cos b) / (\sin a \sin b)\big)$, with the argument clamped to $[-1, 1]$, where $c$ is the measured length of the side opposite vertex $j$ and $a$ and $b$ the other two. This is the first law of cosines, [RAT], Theorem 2.5.3, p. 50, which prints the cosine of the angle in this form. For a lune, and for the double of a lune, it returns one angle $\theta$ for both vertices, from the normals $u_0$ and $u_1$ of its two sides by the form of DOM-20.
- `area()` returns $\theta_0 + \theta_1 + \theta_2 - \pi$ for a triangle, [RAT], Theorem 2.5.5, p. 51, and $2\theta_0$ for a lune, the area of a lune of [RAT], p. 51.

For a spherical domain with no vertex, the whole sphere of `1`, `side_lengths()` and `corner_angles()` return empty lists and `area()` returns $4\pi$, as `4.0 * PI`, the area of the unit sphere, which is exact in IEEE 754 arithmetic as a product by a power of 2. For a spherical domain with a number of vertices other than zero, two, three, or four, `corner_angles()` returns NaN for every vertex and `area()` returns NaN. A spherical kite of Section 6.16 is measured as a kite (DOM-29), so its angles and its area are those of its double bit for bit, and the domains of `nx`, `x`, and `*` are measured as lunes.

DOM-20. `corner_angles()` MUST measure the angle of a domain with two vertices from the normals $u_0$ and $u_1$ of its two sides: for a lune, which includes the domains of `nx`, `x`, and `*`, as $\arccos(-u_0 \cdot u_1)$ with the argument clamped to $[-1, 1]$, and for the double of a lune as $\operatorname{atan2}(\lvert u_0 \times u_1 \rvert, -u_0 \cdot u_1)$. It MUST NOT use a relation between normals and angles for any other domain. The two vertices of such a domain are antipodal and so determine neither side, while for every other domain the vertices determine the angle and DOM-6 applies. The double uses `atan2`, because its angle is $\pi$ for `22`, where the argument of the arccosine is $-1$ and the arccosine magnifies an error in the last place of the normals to about $1.5 \times 10^{-8}$, above the tolerance $10^{-9}$ of checks 5 and 6. The lune keeps the arccosine, so that its values stay bit for bit (DOM-30). The code chooses the form by the kind of side 0: the arccosine for `Mirror` (a lune or the domain of `*`) and for `RotaryReflection` (the domains of `nx` and `x`), and `atan2` for `Rotation` (the double of a lune). The domain of `nx` stores the normals of the lune of `*nn` bit for bit, so the arccosine gives it the angles of that lune bit for bit. At the order 1 the exact normals give the argument $-1$ exactly and the angle `core::f64::consts::PI` exactly. Check 5 reads the angle of these domains by the arccosine.

The relation between the normals and the angle has no source (item 28 of Section 15.1 of the geometry reference), and Section 15.1 records the `atan2` form as an assumption. Check 4 and the exhaustive certificate of DOM-22 test the same angle without it. The area of the double of a lune is $2\theta_0$, the area of a lune of [RAT], p. 51, as for a lune.

A probe that moved each vertex coordinate and each normal component by one unit in the last place confirmed that the arccosine magnifies the error of the angle of `22` and that `atan2` does not, and found the same change with either form on the other domains with two or four vertices that it covered. No committed test repeats it.

A double measures its side lengths with the distance of its geometry, as above. A Euclidean double and each of the ten domains of Section 6.13 measure their angles and their area by the Euclidean formulas above, which apply to any number of vertices. A kite in the hyperbolic plane or on the sphere has four vertices, and the formulas above measure triangles only, so it is measured as two triangles:

DOM-29. For a hyperbolic or spherical kite, `corner_angles()` MUST split the kite along the diagonal from $W_0$ to $W_2$, which lies on the doubled wall, into the triangles $(W_0, W_1, W_2)$ and $(W_0, W_2, W_3)$, measure the angles of each triangle by the first law of cosines of its geometry as above, return the angle of $W_1$ and of $W_3$ from its one triangle, and return the angle of $W_0$ and of $W_2$ as the sum of its two angles, and `area()` MUST return the sum of the areas of the two triangles, each measured as above, because DOM-6 forbids angles from normals for a domain whose vertices determine its sides, and the sourced laws of cosines and area formulas of the research measure triangles only. No angle of a kite comes from its normals. At a flat vertex each triangle has a right angle, where the arccosine is well conditioned.

That the angle of a kite at $W_0$ or $W_2$ is the sum of the angles of its two triangles there rests on no sentence of a source, and Section 15.1 records it.

## 8. Verification

### 8.1. Report Types

`verify()` returns a `Verification` that holds one `Check` for each of the six checks of Section 8.2. Each `Check` holds its residual and its tolerance. A check passes when its residual is at most its tolerance, so an infinite residual fails. Section 2.1 defines the residual, which is never NaN.

DOM-7. `verify()` MUST compute every check of Section 8.2 on every call and MUST report each one separately, because `escher info --check` prints the result of each check, and a verification that stops at the first failure hides whether the other conditions hold.

DOM-8. `verify()` MUST be pure, MUST NOT panic for any domain, and MUST NOT renormalize any stored value, because it runs on domains that tests corrupt on purpose, and a renormalized value would hide the defect that the check exists to find.

### 8.2. Checks

Every check runs on the stored values of the domain in the final position, unless the check names another frame. Checks 2 to 6 read the vertices, so each first requires at least one side (except for the spherical domain with no side of DOM-62), as many vertices and corners as sides, and every vertex to have finite coordinates and, for a hyperbolic or Euclidean domain, a positive last coordinate, or for a spherical domain, not to be the zero vector (DOM-21). Its residual is infinity when any of these fails. Check 1 reads only the pairings. A residual that is not finite after the computation of its check is reported as infinity:

1. **Isometries.** For each side, compute `hyperbolic::isometry_defect(pairing)`. The residual is the largest value. Tolerance $10^{-9}$. Condition: Theorem 10.1 of [CFKP] (Section 6.1 of the geometry reference).
2. **Pairings.** The residual is infinity unless every structural condition holds, and otherwise the largest of the numeric parts. Tolerance $10^{-9}$. Conditions: items 1 and 2 of Section 4.8 of the domain reference, and Condition 3.2 (a) to (d) of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference).
   - Structural: `partner(partner(i)) == i` for every side.
   - Structural: with $c$ the stored centre, `hyperbolic::side(covector_i, c)` is negative for every side $i$, and `hyperbolic::side(covector_partner(i), P_i c)` is positive.
   - Numeric: the largest absolute entry of $P_i P_{\text{partner}(i)} - I$ over the sides. For a mirror this is the relator $r_i^2 = 1$.
   - Structural: the kind, the partner, the orientation, and the translation test of every side agree, as DOM-27 states, and the sign of the determinant of $P_i$ is positive for `Preserving` and negative for `Reversing`.
   - Numeric: for a mirror side $i$, the largest absolute coordinate of $P_i E - E$ divided by the last coordinate of $E$, for the endpoints $E = V_{i-1}$ and $E = V_i$.
   - Numeric: for a side $i$ of the kind `Rotation` or `Translation`, with $p$ its partner, the largest absolute coordinate of $P_i V_{i-1} - V_p$ divided by the last coordinate of $V_p$, and of $P_i V_i - V_{p-1}$ divided by the last coordinate of $V_{p-1}$, so that $P_i$ maps the side onto its partner with its endpoints exchanged (Condition 3.2 (c) of [EPSTEIN-PETRONIO1994]).
   - Numeric: for a side $i$ of the kind `GlideReflection` or `RotaryReflection`, with $p$ its partner, the largest absolute coordinate of $P_i V_{i-1} - V_{p-1}$ divided by the last coordinate of $V_{p-1}$, and of $P_i V_i - V_p$ divided by the last coordinate of $V_p$, so that $P_i$ maps the side onto its partner with its endpoints in the same order. The term of a mirror above is the case $p = i$ of this term.
   - Check 2 chooses the endpoint term of a side by its kind and not by its orientation. The structural term of DOM-27 ties each kind to one orientation, so the two choices differ only on a domain whose residual is already infinity.
   - Numeric: for a side $i$ that is not a mirror, with $p$ its partner, the wall term: the largest absolute coordinate of $P_i^{T} c_p + c_i$, for the covectors $c_i$ and $c_p$, with no division. It is 0 when $P_i$ carries the wall of side $i$ onto the wall of side $p$ with the domain on opposite sides (Condition 3.2 (b) and (c)). The endpoint terms alone also pass a pairing that maps the two endpoints correctly but not the wall, such as the half-turn about the axis through the two vertices of the double of a lune (Section 8.3).
3. **Walls.** For each side $j$, compute these five values. The residual is the largest of them over the sides. Tolerance $10^{-9}$. Condition: a wall is a unit normal stored with its covector (Sections 2.1 and 5.2 of the architecture), it is orthogonal to the points of its side (equation (6.6) of [DAVIS], Section 7.4 of the geometry reference), the pairing of a mirror is the reflection in that wall (Section 5.3), and the polygon is convex, so no vertex lies outside a wall (the convex polyhedron of Definition 2.1 of [EPSTEIN-PETRONIO1994], Section 4.9 of the domain reference, and `UNV-31` for a double).
   - The absolute value of `hyperbolic::side(covector_j, V_{j-1})` divided by the last coordinate of $V_{j-1}$.
   - The same for $V_j$.
   - For a side of the kind `Mirror`, the largest absolute entry of `pairing_j - hyperbolic::reflection(normal_j)`, and 0 for every other side.
   - The largest absolute coordinate of `covector_j - hyperbolic::covector(normal_j)`.
   - The convexity term: the largest of 0 and `hyperbolic::side(covector_j, V_k)` divided by the last coordinate of $V_k$, over every vertex $V_k$ of the domain.
4. **Relators.** For a domain with a side that is not a mirror, the walk of DOM-55 and DOM-28. For a domain whose sides are all mirrors, for each corner $j$, let $F =$ `hyperbolic::translation(V_j)`, which reads only the first two coordinates of $V_j$, and let $C =$ `hyperbolic::inverse(F) * P_j * P_{j+1} * F`. Compute $C^{n_j}$ by $n_j - 1$ successive multiplications by $C$. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. Tolerance $2 \times 10^{-6}$. Condition: the relator $(r_j r_{j+1})^{n_j} = 1$ of the presentation of Section 4.4 of the domain reference ([MASKIT2001] Section 4.4), and item 4 of its Section 4.8. For a domain with five or more vertices, DOM-49 evaluates the same quantities in double-double arithmetic.
5. **Angles.** Measure each $\theta_j$ as Section 7 states, with $n_j$ the stored corner orders, which check 4 also reads. The residual is the largest $\lvert \theta_j - \pi/n_j \rvert$, or infinity when $\theta_0 + \theta_1 + \theta_2$ is not below $\pi$. Tolerance $10^{-9}$. Condition: every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference), and the angle sum of a hyperbolic triangle is below $\pi$ (Section 10.1 of the geometry reference).
6. **Area.** The residual is the absolute difference between `area()` and the area of the signature that step 1 of Section 6.1 computed and the domain stores, which is $\pi$ times `area_over_pi()`, converted with no limit on the size of its numerator or denominator. Tolerance $10^{-9}$. Condition: the area of the triangle is $\pi$ minus its angle sum (Section 10.1 of the geometry reference), and the area of a hyperbolic orbifold is $-2\pi\chi$ (Section 7.3 of the notation reference).

Check 6 is a consistency check between the area that the crate measures and the area that the Euler characteristic of the signature gives, $-2\pi\chi$, by the defect rule of Section 6.1 of the notation reference. It is not an independent test of the geometry. It guards against a disagreement between this crate and `escher-signature` about the orders of the corners or the area of the orbifold. Section 15.3 asks the owner whether to keep it.

Check 4 evaluates the relator in the corner frame, where vertex $j$ is the origin. A power of a matrix is the identity exactly when the same power of its conjugate is, which Section 15.1 records as an assumption. The frame is a choice for accuracy: near the order limit, the residual of the exact power of the `f64` pairings in the domain frame exceeds the tolerance, while in the corner frame it passes, which the tests check in the reference arithmetic. Check 4 has the smallest margin of the six. Repeated multiplication is chosen over repeated squaring on a measurement of a prototype.

Check 4 alone does not fix the angle, because it also passes for an angle of $k\pi/n_j$. Check 5 closes that gap. Check 5 uses only the vertices, and checks 2, 3, and 4 tie the walls and the pairings to the vertices.

`verify()` costs at most $3 \cdot 999 + 18$ matrix products at the order limit. Each corner needs $n_j - 1$ products for the power, two for its frame, and three for its conjugate, and check 2 needs three more for the pairing relators.

For a Euclidean domain, `verify()` computes the same six checks with the same tolerances, and reads each one in this way. The vertex condition of the checks 2 to 6 holds for a Euclidean point, whose last coordinate is 1, and indexes are modulo $m$:

| Check | Euclidean reading | Condition |
| --- | --- | --- |
| 1. Isometries | `euclidean::isometry_defect` of each pairing | [RAT], Theorem 1.3.5 (Section 12 of the geometry reference) |
| 2. Pairings | As for a hyperbolic domain, with `euclidean::side` | As for a hyperbolic domain |
| 3. Walls | For each side $j$, the largest of: the absolute value of `euclidean::side(covector_j, E)` divided by the last coordinate of $E$ for $E = V_{j-1}$ and $E = V_j$, the largest absolute entry of `pairing_j - euclidean::reflection(covector_j)`, the largest absolute coordinate of `normal_j - (c_1, c_2, 0)` for the covector $c$, $\lvert c_1^2 + c_2^2 - 1 \rvert$, and the convexity term with `euclidean::side` | A line covector of a line through the endpoints of its side, stored with its unit direction, whose reflection is the pairing (Sections 4.5 and 6.6 of the geometry design) |
| 4. Relators | For each corner $j$, $C = P_j P_{j+1}$, and $C^{n_j}$ by $n_j - 1$ successive multiplications. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. There is no corner frame | Item 4 of Section 4.8 of the domain reference, and the local group $\langle a, b : a^2 = b^2 = (ab)^n = 1 \rangle$ of a corner reflector of order $n$ of [TH], Proposition 13.3.1(iii), p. 308 (Section 9.4 of the notation reference) |
| 5. Angles | The largest $\lvert \theta_j - \pi/n_j \rvert$, with $\theta_j$ of Section 7. The condition on the angle sum of a hyperbolic triangle does not apply | Every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference) |
| 6. Area | $\lvert (m - 2)\pi - \sum_j \theta_j \rvert$, against the value $-2\pi\chi = 0$ of a Euclidean signature | A consistency check only, as below |

Check 6 of a Euclidean domain passes on the triangle of `*442` labelled `*632`, where checks 4 and 5 fail (Section 8.3). The design keeps it so that the six checks exist for every geometry (item 4 of Section 15.3).

For a spherical domain, `verify()` computes the same six checks with the same tolerances, and reads each one in this way, with indexes modulo $m$:

| Check | Spherical reading | Condition |
| --- | --- | --- |
| 1. Isometries | `spherical::isometry_defect` of each pairing | [RAT], Theorem 2.1.3 (Section 11.1 of the geometry reference) |
| 2. Pairings | As for a hyperbolic domain, with `spherical::side`. The endpoint part is the largest absolute coordinate of $P_i E - E$, with no division by a last coordinate | As for a hyperbolic domain |
| 3. Walls | For each side $j$, the largest of: the absolute value of `spherical::side(covector_j, E)` for $E = V_{j-1}$ and $E = V_j$, with no division, the largest absolute entry of `pairing_j - spherical::reflection(normal_j)`, the largest absolute coordinate of `covector_j - normal_j`, $\lvert \text{normal}_j \cdot \text{normal}_j - 1 \rvert$, and the convexity term with `spherical::side` and no division. Also $\lvert V \cdot V - 1 \rvert$ for each vertex $V$, and, for a lune, which includes the domains of `nx`, `x`, and `*`, the largest absolute coordinate of $V_0 + V_1$ | A great-circle normal of a great circle through the endpoints of its side, stored as its own covector, whose reflection is the pairing (Sections 4.6 and 6.7 of the geometry design). Each vertex is a sphere point, and the two vertices of a lune are antipodal (Section 6.7) |
| 4. Relators | For each corner $j$, $C = P_j P_{j+1}$, and $C^{n_j}$ by $n_j - 1$ successive multiplications. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. There is no corner frame. For a lune this is $(P_0 P_1)^n$ at corner 0 and $(P_1 P_0)^n$ at corner 1 | Item 4 of Section 4.8 of the domain reference, and the local group of a corner reflector of [TH], Proposition 13.3.1(iii), p. 308 (Section 9.4 of the notation reference) |
| 5. Angles | The largest $\lvert \theta_j - \pi/n_j \rvert$, with $\theta_j$ of Section 7, or infinity when the angle sum of a triangle is not above $\pi$. Both angles of a lune come from its normals (DOM-20) | Every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference), and the angle sum of a spherical triangle exceeds $\pi$ ([RAT], Theorem 2.5.1, p. 50) |
| 6. Area | The absolute difference between `area()` and the stored area of the signature, $\pi$ times `area_over_pi()`, which is $2\pi\chi$ | The areas of a triangle and of a lune of Section 7, and the area $2\pi\lvert\chi\rvert$ of an elliptic orbifold of [TH], p. 312 (Section 7.3 of the notation reference) |

Check 3 reads the vertices of a spherical domain because both sides of a lune have the side value 0 at every point of the second axis. Without the vertex terms, a lune whose vertices coincide or lie off the sphere would pass every check.

DOM-21. For a spherical domain, the precondition of checks 2 to 6 MUST require each vertex to be finite and not the zero vector, and MUST NOT require a positive last coordinate, because a vertex of a spherical domain may lie on or below the plane $x_3 = 0$, as both vertices of a lune do, and the precondition of the other geometries would give every lune an infinite residual.

`verify()` of a spherical domain costs no more products than the bound above. At the order limit, a lune needs $2 \cdot 999$ products for its two relators.

For a double, `verify()` computes the same six checks with the same tolerances, reads each one in its geometry as above, and changes them in this way:

| Check | Reading for a double | Condition |
| --- | --- | --- |
| 1. Isometries | Unchanged | As above |
| 2. Pairings | With the structural term of DOM-27, and the endpoint terms and the wall term of a side that is not a mirror | Condition 3.2 (a) to (d) of [EPSTEIN-PETRONIO1994], and the hypothesis of Theorem 12 of [SOLOMYAK2020] that no side is paired with itself (Sections 4.1 and 4.9 of the domain reference) |
| 3. Walls | No reflection term, because no side is a mirror. The covector term and the convexity term in the geometry of the domain, with no division for a spherical domain | A wall stored with its covector, and a convex polygon (`UNV-31`) |
| 4. Relators | The walk of the vertex cycles of DOM-28 | Definition 11 and Theorem 12 of [SOLOMYAK2020], and Conditions 3.10 and 3.13 and Lemma 3.12 of [EPSTEIN-PETRONIO1994] (item 3 of Section 4.8 of the domain reference) |
| 5. Angles | The largest $\lvert \theta_k - c_k \pi / n_k \rvert$, with $\theta_k$ of Section 7, $n_k$ the order, and $c_k$ the copies of vertex $k$. For a hyperbolic kite, infinity when the angle sum of either triangle of DOM-29 is not below $\pi$, and for a spherical kite, when it is not above $\pi$ | The cycle condition of item 3 of Section 4.8 of the domain reference, read per vertex through its copies, and the angle sums of a triangle of Section 10.1 of the geometry reference and of [RAT], Theorem 2.5.1, p. 50 |
| 6. Area | The absolute difference between `area()` and the stored area of the rotation signature, and for a Euclidean double $\lvert (m - 2)\pi - \sum_k \theta_k \rvert$ | As for a reflection domain of its geometry |

A reflection domain reads check 5 in the same way, because each of its corners has the copies 1.

DOM-27. Check 2 MUST fail structurally unless the kind, the partner, the orientation, the pairing, and the geometry of every side agree: a side of the kind `Mirror` is its own partner and has the orientation `Reversing`, a side of the kind `GlideReflection` or `RotaryReflection` has a partner other than itself and the orientation `Reversing`, a side of the kind `GlideReflection` belongs to a domain that is not spherical and a side of the kind `RotaryReflection` to a spherical domain (DOM-61), a side of the kind `Rotation` or `Translation` has a partner other than itself and the orientation `Preserving`, and a side has the kind `Translation` exactly when its pairing passes the translation test. The translation test holds when the domain is Euclidean and each entry of the linear part of the pairing, its upper-left 2 by 2 block, is within $10^{-9}$, the tolerance of check 2, of the identity, or when the domain is hyperbolic and the determinant of the pairing is positive and its trace is above 3. It never holds for a spherical domain. A glide reflection never passes the translation test. In the Euclidean plane its linear part is a reflection and not the identity. In the hyperbolic plane its determinant is negative, so it is never a translation, while the trace alone would not exclude it, because the trace of a long glide reflection exceeds 3. A hyperbolic isometry that preserves orientation has the trace $1 + 2\cosh \ell$ when it is a translation by the length $\ell$, which is above 3, and a trace in $[-1, 3)$ when it is a rotation. Section 14 records this classification as unverified. The rule exists because the renderer reads the partner, the orientation, and the pairing of a side and never its kind, so a kind that these contradict would go undetected. `escher info` would then print a false kind in its `Side pairs` line.

DOM-61. Check 2 MUST treat a side of the kind `RotaryReflection` as it treats a side of the kind `GlideReflection`: a partner other than the side itself, the orientation `Reversing`, a pairing with a negative determinant, the endpoint term with the endpoints in the same order, $P_i V_{i-1} = V_{p-1}$ and $P_i V_i = V_p$, the wall term, and a pairing that never passes the translation test, which never holds on a spherical domain. It MUST also fail structurally for a side of the kind `RotaryReflection` on a domain that is not spherical, and for a side of the kind `GlideReflection` on a spherical domain. The reasons are that a rotary reflection of the lune of `nx` maps its side onto its partner with the endpoints in the same order, and that the kind is a label that `escher info` prints and the renderer never reads, so only the plane rule keeps a false label from passing: on a prototype, `x` with its pairings labelled `GlideReflection` passed `verify()` and enumerated to the order 2 without the plane rule. The plane rule is `DEC-57` of the architecture.

DOM-28. Check 4 of a domain with a side that is not a mirror MUST walk the vertex cycles by the state walk of DOM-55 and evaluate the relator of each cycle, and check 4 of a domain whose sides are all mirrors MUST evaluate the corner relators as item 4 of Section 8.2 states, because the corner relator $(P_j P_{j+1})^{n_j}$ holds between two mirrors, while a cycle that meets a paired side has the relator of its walk. A domain that mixes mirrors and paired sides walks its cycles like any other domain with a paired side.

DOM-55. The walk of the vertex cycles MUST run over the states $(v, d)$ of a vertex $v$ and a direction $d$ of $+$ or $-$, by the steps and the table below, with the structural limit of $2m$ steps for each cycle and the structural conditions below, because one walk then covers the doubles, the glide reflections, the mirrors beside paired sides, and the domains with only translations, and it reproduces the walk of the doubles step for step, so that every double keeps its cycles, its relator texts, and its residuals. From the state $(v, +)$ the next side is $s = (v + 1) \bmod m$, and from $(v, -)$ it is $s = v$. The walk applies $g \leftarrow P_s g$, and with $p$ the partner of $s$ it moves to the state of this table:

| $P_s$ | From $(v, +)$ | From $(v, -)$ |
| --- | --- | --- |
| Orientation `Preserving` | $(p, +)$ | $(p - 1, -)$ |
| Orientation `Reversing`, which includes a mirror with $p = s$ | $(p - 1, -)$ | $(p, +)$ |

Indexes are modulo $m$. The walk starts at $(v, +)$ for $v = 0, 1, \dots, m - 1$ in order, skips a start state that an earlier cycle visited, and repeats the step until the state is the start state again. It marks each visited state and its flip $(v, -d)$ as visited, because the reversed walk of a cycle gives the inverse relator. A walk that takes more than $2m$ steps fails structurally. For each cycle:

- Structural: every vertex of the cycle stores the same order, which is the cycle order $m_c$. Every domain without the closure `Cycles` of Section 3.2 also requires the copies of the distinct vertices of the cycle to sum to 2. These domains include every double of Section 6.8 and `2222` at every value of `skew`. The wallpaper domains, the spherical kites, and the domains of `nx` and `x` have the closure `Cycles` and need no such condition. A vertex that the walk visits in both directions adds its copies once.
- Numeric: the largest absolute coordinate of $g V_{\text{start}} - V_{\text{start}}$, divided by the last coordinate of $V_{\text{start}}$ for a hyperbolic or Euclidean domain. This is the condition that the cycle transformation fixes its vertex (Condition 3.10 of [EPSTEIN-PETRONIO1994]).
- Numeric: the largest absolute entry of $C^{m_c} - I$ divided by $m_c$, with $C^{m_c}$ by $m_c - 1$ successive multiplications. For a hyperbolic domain, $C = F^{-1} g F$ in the corner frame of $V_{\text{start}}$, with $F =$ `hyperbolic::translation(V_start)`, and otherwise $C = g$. A hyperbolic domain with five or more vertices evaluates $g$, both numeric parts, and the frame by DOM-49.

The residual is the largest numeric part over the cycles, or infinity when a structural condition fails.

For a domain whose pairings all preserve orientation, the walk stays in the states $(v, +)$ and is the walk of Definition 11 of [SOLOMYAK2020], step for step: $s = (v + 1) \bmod m$, $g \leftarrow P_s g$, and $v = $ `partner(s)`. The walk of a domain with a reversing pairing follows the cycle of Conditions 3.7, 3.10, and 3.13 and Lemma 3.12 of [EPSTEIN-PETRONIO1994], which passes to the other face at each step and puts no condition on the orientation of a pairing (Section 4.9 of the domain reference). A corner between two mirrors gives two states, and its angle $\theta$ counts twice, so the condition is $2\theta = 2\pi/m$, as Example 3.15 of [EPSTEIN-PETRONIO1994] shows. The table of directions and the limit of $2m$ steps are this design's reading of those conditions for a polygon in the plane, which no source states in this form, and Section 14 records it with `UNV-32`.

The condition on the copies stays for the doubles, because without it two defects of the copies that the unit tests inject would pass. The all-mirror path of the corner relators (DOM-33) does not change, so the reflection domains keep their values bit for bit (DOM-30).

DOM-56. Check 5 of a wallpaper domain, of `2222` at every value of `skew`, and of a spherical group domain with the closure `Cycles`, which are the spherical kites and the domains of `nx` and `x`, MUST use the cycle rule: its residual is the largest, over the cycles of DOM-55, of the absolute difference between the angle sum over the states of the cycle and $2\pi / m_c$, where a state $(v, d)$ adds the angle $\theta_v$ of Section 7 of its vertex. Check 5 of every other domain MUST be per vertex, with the copies. The reasons are that the angle at a vertex of a sheared `2222` or of `o` is not a fixed fraction of $\pi$, so no per-vertex rule holds there, while the angle sum of each cycle is $2\pi / m_c$ by Lemma 3.12 and Condition 3.13 of [EPSTEIN-PETRONIO1994], and that the other domains keep their residuals of check 5 bit for bit. When the walk of a domain fails structurally, the residual of check 5 is infinity. A spherical kite needs the cycle rule as a wallpaper kite does, because the vertices of its chain have the order 1 and the right angle, which no per-vertex rule with the copies 1 accepts. On a prototype, the spherical kites with the closure `PerVertex` failed checks 4 and 5. A spherical kite keeps the condition of the table of the doubles, that check 5 is infinity when the angle sum of either of its two triangles is not above $\pi$.

This design chooses the cycle rule for `2222` at every value of `skew`, because the sheared domain needs it and one rule for one signature keeps the check uniform over its parameters. At `skew` 0 the cycle rule and the per-vertex rule give the same residuals, and a unit test MUST confirm that equality (Section 12.1). A chain counts each vertex that it visits in both directions twice: the chain $\{V_0, V_1\}$ of `**` has the states $(0, +)$, $(0, -)$, $(1, -)$, and $(1, +)$ and the angle sum $4 \cdot \pi/2 = 2\pi$. The expected cycles are those of the table of Section 6.8. For a kite, the cycles $\{W_0\}$ and $\{W_2\}$ are the rotations about corners 0 and 1 of the generating triangle, and the cycle $\{W_1, W_3\}$ is a product of four reflections.

At a flat vertex of a Euclidean double, check 5 compares an arccosine of an argument close to $-1$ with $\pi$, where the arccosine is ill-conditioned. A small tilt of a wall can therefore pass check 5, so checks 2 and 3 carry that case through the vertices and the pairings. A kite measures its flat vertex as two right angles (DOM-29), and the double of `22` measures its angle with `atan2` (DOM-20), so this does not apply to either.

A wallpaper domain, and `2222` with `skew` other than 0, measure a flat vertex by the same arccosine. The argument there is exactly $-1$ at the settings of the grid of Section 8.4, so the angle is exactly $\pi$ (Section 6.13), and checks 2 and 3 still carry a tilted wall.

For one of the ten domains of Section 6.13, `verify()` computes the same six checks with the same tolerances, reads each one as for a Euclidean domain, and changes them in this way:

| Check | Reading for one of the ten domains | Condition |
| --- | --- | --- |
| 1. Isometries | Unchanged | As above |
| 2. Pairings | The structural term of DOM-27, the endpoint term of each side by its kind (exchanged for `Rotation` and `Translation`, in order for `GlideReflection` and `Mirror`), and the wall term of every side that is not a mirror | Condition 3.2 (a) to (d) of [EPSTEIN-PETRONIO1994] |
| 3. Walls | The reflection term for the sides of the kind `Mirror` only, and 0 for every other side, with the other terms of a Euclidean domain | As for a Euclidean domain |
| 4. Relators | The state walk of DOM-55, with no condition on the copies except for `2222` | Conditions 3.7, 3.10, and 3.13 and Lemma 3.12 of [EPSTEIN-PETRONIO1994] |
| 5. Angles | The cycle rule of DOM-56 | Lemma 3.12 and Condition 3.13 of [EPSTEIN-PETRONIO1994] |
| 6. Area | $\lvert (m - 2)\pi - \sum_k \theta_k \rvert$, as for every Euclidean domain | A consistency check only |

For a spherical group domain, `verify()` computes the same six checks with the same tolerances, reads each one as for a spherical domain, and changes them in this way:

| Check | Reading for a spherical group domain | Condition |
| --- | --- | --- |
| 1. Isometries | Unchanged | As above |
| 2. Pairings | The structural term of DOM-27 with the plane rule of DOM-61, the endpoint term of each side by its kind (exchanged for `Rotation`, in order for `RotaryReflection` and `Mirror`), and the wall term of every side that is not a mirror | Condition 3.2 (a) to (d) of [EPSTEIN-PETRONIO1994] |
| 3. Walls | The reflection term for the sides of the kind `Mirror` only, the other terms of a spherical domain, and the term $V_0 + V_1$ for the domains of `nx`, `x`, and `*` | As for a spherical domain |
| 4. Relators | The state walk of DOM-55, with no condition on the copies, for the domains with a paired side, and the corner relators of DOM-28 for `*` | Conditions 3.7, 3.10, and 3.13 and Lemma 3.12 of [EPSTEIN-PETRONIO1994], and the local group of a corner reflector |
| 5. Angles | The cycle rule of DOM-56 for the domains with a paired side, with the condition on the triangles of a spherical kite, and per vertex for `*` | Lemma 3.12 and Condition 3.13 of [EPSTEIN-PETRONIO1994] |
| 6. Area | The absolute difference between `area()` and the stored area of the signature, as for every spherical domain | A consistency check only |

The tolerances stay as DOM-9 states.

DOM-62. `verify()` MUST give a spherical domain with no side, no vertex, and no corner the residual 0 in checks 2 to 5, whose conditions hold vacuously, and MUST compute check 6 as for every spherical domain, from `area()` of $4\pi$ (Section 7), while a hyperbolic or Euclidean domain with no side MUST keep the infinite residuals of checks 2 to 6. `side_presentation()` MUST give it the presentation with no generator and no relator of Section 5.4. The reason is that the whole sphere of `1` is the one domain without a side that `build_with` returns, while a domain of another geometry without a side can come only from a defect. Two changes implement it: the precondition `vertices_are_points` of `src/verify.rs` accepts no vertex for a spherical domain with no side, and `area()` returns $4\pi$ for a spherical domain with no vertex. A wrong stored area fails check 6, and a hyperbolic or Euclidean domain emptied of its sides still fails five checks. The defect test that removes every vertex of `*732` keeps its sides, so the changes do not affect it. This is the domain of `DEC-58` of the architecture.

The walk needs one product for each step of each cycle, at most $2m$ in all, which is $m$ for a domain whose pairings all preserve orientation. With the frames, the powers, and check 2, the cost of `verify()` of a double at the order limit stays about 3000 matrix products, as for a reflection domain, because the powers dominate and a kite has three cycles.

For a polygon and for the double of a polygon, `verify()` computes the same six checks with the same tolerances, as for a hyperbolic triangle and a kite, with these readings:

| Check | Reading for a polygon or its double | Condition |
| --- | --- | --- |
| 1 to 4 | Unchanged. Check 4 evaluates a corner relator at each of the $n$ corners of a polygon, and walks the vertex cycles of the table of Section 6.8 for a double | As above |
| 5. Angles | The largest $\lvert \theta_k - c_k \pi / n_k \rvert$, with $\theta_k$ of DOM-46. For four vertices, infinity when the angle sum of either triangle of DOM-29 is not below $\pi$. For five or more vertices, infinity when $\sum_k \theta_k$ is not below $(m - 2)\pi$ | Every corner has the angle $\pi/n_j$, and the angle sum of a hyperbolic polygon is below $(m - 2)\pi$ (Theorem 7.16.2 of [BEA], Section 10.1 of the geometry reference) |
| 6. Area | The absolute difference between `area()` and the stored area of the signature, with `area()` of DOM-46 | The area $(m - 2)\pi - \sum_k \theta_k$ of Theorem 7.15.1 of [BEA], and the area $-2\pi\chi$ of a hyperbolic orbifold |

For five or more vertices, the residual of check 6 is $\lvert \sum_k (\theta_k - c_k \pi / n_k) \rvert$, the sum of the angle errors, which is at most $m$ times the residual of check 5 and has the same tolerance. It adds no independent information about the polygon, and on a prototype it was the check that failed first in most failing cases. The design keeps its tolerance of $10^{-9}$, so check 6 binds the guard, and item 4 of Section 15.3 puts the choice to the owner.

DOM-49. Check 4 of a hyperbolic domain with five or more vertices MUST be evaluated in the double-double arithmetic of Section 6.10, from the stored `f64` values: the corner relator $P_j P_{j+1}$ and the cycle transformation $g$ as products of the stored pairings, the fixed-point term $g V - V$, the corner frame as the translation in closed form of step 8 of Section 6.10 of the vertex normalized in double-double arithmetic, with the inverse that negates the entries that mix the time coordinate with a space coordinate, and the power by $m_c - 1$ successive multiplications. Each entry of a residual MUST be rounded to `f64` once, before the division by the order, and the tolerance stays $2 \times 10^{-6}$. Check 4 of a domain with three or four vertices, and of every Euclidean and spherical domain, MUST stay in `f64` arithmetic as above. The reason is that in `f64`, the product of two pairings of a double with entries of about $10^{2}$ to $10^{4}$, for walls far from the centre, cancels to a rotation with much smaller entries, and the corner frame of a far vertex and the power then amplify the rounding of that product. A temporary test, which is not committed, found check 4 in `f64` failing hundreds of doubles with ordinary orders, such as `99999999`, through that rounding alone, so its outcome near the tolerance depends on it. The path in `f64` keeps every value and residual of the triangles, the kites, the rectangle, the lune, and the polygons with four corners bit for bit (DOM-30). The corner relators of the starred polygons with five or more corners go through the same evaluation.

`verify()` of a polygon costs at most $m \cdot 999 + 6m$ matrix products at the order limit, which is 8040 for eight corners, because each corner needs $n_j - 1$ products for its power and five for its frame and its conjugate, and check 2 needs one for each side. The double of a polygon of eight corners has 14 sides and eight vertex cycles, and costs about as much. With five or more vertices, each product of check 4 is a double-double product (DOM-49), which costs several times an `f64` product.

### 8.3. Defects Each Check Detects

The tests inject defects into domains built through private access, or into domains that `build_with` returns, and assert for each defect that exactly the expected checks fail and that `passed()` is false. Each domain passes every check before its defect. The defects cover every class of domain:

- Triangles, Euclidean domains, spherical triangles, and lunes: a pairing scaled by $1 + 10^{-6}$, which fails checks 1, 2, and 3, a wrong or out-of-range partner, which fails check 2, a tilted wall with its pairing recomputed, which fails checks 2 and 3 and may fail check 4, a wall tilted or moved with its pairing left as constructed, which fails check 3 alone, a hyperbolic or spherical triangle labelled with the orders of another signature, which fails checks 4, 5, and 6, the Euclidean triangle of `*442` labelled `*632`, which fails checks 4 and 5 while check 6 passes, a lune labelled with another order, which fails checks 5 and 6, a vertex off its sheet, the zero vector on the sphere, a NaN, or a removed vertex, which give infinite residuals, and the vertices of a lune made to coincide or to leave the sphere, which fail check 3.
- Doubles: a side made its own partner, a pairing replaced by its inverse or by a half-turn, a kind that contradicts the translation test, wrong copies, merged halves of a split side, a kite labelled with an order off by one, and a vertex moved outside a wall that is not adjacent to it, which fails check 3 through its convexity term.
- Polygons: two adjacent corners exchanged, which fails checks 4 and 5 while check 6 passes, an order raised by 1, a vertex moved toward the centre with its walls left as constructed, which fails checks 2, 3, 5, and 6 while check 4 passes, a removed vertex, and a NaN, on regular polygons with five or more corners and on the polygons that `build_with` returns.
- Wallpaper domains: a glide reflection labelled or built as a translation, a wrong orientation, wrong cycle or corner orders, merged split sides, a half-turn about the wrong centre, and wrong partners.
- Spherical group domains: the lune of `nx` paired by a rotation, a kite with the mirrors of another group, a reversing pairing labelled `Rotation` or `GlideReflection`, a pairing replaced by a reflection that fixes both endpoints, and glide reflections of a Euclidean domain labelled `RotaryReflection`, which the plane rule of DOM-61 catches.

Check 4 passes on several of these defects, such as a scaled pairing, so checks 1, 2, and 3 catch them. Check 3 exists for the defect with the pairing left as constructed: the renderer tests a point with the covector and moves it with the pairing, so a domain whose covector and pairing disagree corrupts the image while the other five checks pass. The half-turn about the axis through the two vertices of the double of `44` fixes both vertices, as the endpoint terms require, so only the wall term of check 2 fails, and without that term the domain passes all six checks.

`side_presentation()` succeeds on most of these defects, so the guards of DOM-54 and DOM-60 come from `verify()` and not from the presentation. None of the spherical defects that `verify()` catches exercises the census of DOM-64, and the defects that only the census catches pass `verify()` by design (Section 12.1).

Two changes are not defects. A move of the flat vertex $W_2$ of `3*` along the equator passes every check, because its position is a choice that `verify()` does not check (Section 6.16), and the tests assert that it passes, so that a change that makes `verify()` read that position is seen. Copying the reflection of side 3 of `3*` onto side 2 also passes, because the two mirror sides of `n*` lie on one great circle and the two reflections differ only by rounding.

The tolerances separate an order from its neighbour at the limit. Measurements outside the repository, which no test repeats, found that, over a sample of labels with a first order from 900 to 1000, every triangle with one order raised by 1 fails checks 4, 5, and 6.

### 8.4. Tolerances

DOM-9. Each check MUST use the tolerance of this table, because `escher info --check` prints the tolerance with a failed check, and the margins that the tests hold under these tolerances are what make a pass meaningful at every supported order.

| Check | Tolerance |
| --- | --- |
| 1. Isometries | $10^{-9}$ |
| 2. Pairings | $10^{-9}$ |
| 3. Walls | $10^{-9}$ |
| 4. Relators | $2 \times 10^{-6}$ |
| 5. Angles | $10^{-9}$ |
| 6. Area | $10^{-9}$ |

Every geometry and every class of domain uses these tolerances. The margin of a check is its tolerance divided by its largest residual over an input set. A sweep outside the repository over every supported triple, every $(n_0, n_1, n_2)$ with $1000 \ge n_0 \ge n_1 \ge n_2 \ge 2$ and $1/n_0 + 1/n_1 + 1/n_2 < 1$, found no triple that fails a check and a margin of at least 8 for every check. The tests run the six checks on the named input sets of Section 12.1, which include the worst triples of that sweep, assert a margin of at least 8 where this section claims one, as TEST-13 of the coding standard requires, and print the largest residual of each check with its input and its margin.

On a built domain, the third and fourth values of check 3 are 0, because the construction stores the pairing of a mirror as `hyperbolic::reflection(normal)` and the covector as `hyperbolic::covector(normal)`. Its residual is therefore the largest of its first two values, the side test of a vertex against its wall divided by the last coordinate of the vertex, and the convexity term. On a double, the collinear halves of a split side put a vertex on the line of the other half, so the term can add a rounding error there. Section 9.3 states how the accuracy rows of the geometry design relate to the inputs of this crate.

The tolerances are choices of this design, and Section 15.3 asks the owner about that of check 4. The tolerance of check 4 is $2 \times 10^{-6}$, because $10^{-6}$ would leave a margin below the 8 that TEST-13 of the coding standard requires at the worst triples near the order limit. At $2 \times 10^{-6}$, check 4 still fails every triangle near the limit with one corner order off by one (Section 8.3). A tolerance of $10^{-5}$ would not separate the order 1000 from 1001 by check 4 alone.

The tolerance of check 3 stays at $10^{-9}$, the tolerance of checks 1, 2, 5, and 6. A smaller tolerance would detect a smaller tilt of a wall. This design keeps $10^{-9}$, because one tolerance for these five checks keeps the output of `escher info --check` uniform, and the margin of check 3 already exceeds that of every other check.

The wall term of check 2 adds nothing to a reflection domain. The largest residual of check 4 of the kites is that of a cycle of one vertex, the rotation about a corner of the generating triangle, and equals the largest residual of check 4 of the reflection triangles, so the tolerances hold for the doubles unchanged. The residual of check 4 of a polygon is that of the double-double check of DOM-49.

This design claims a margin of at least 8 for every check at the default shape of these polygons:

- Every signature with four to eight corners and orders up to 12.
- Every signature with four to six corners and a corner of order 1000.

The claims rest on the named and the small polygons, on a set of every canonical signature with four to six corners, at least one of order 1000, and the other orders from $\{2, 3, 7, 1000\}$, and on the default sweep of Section 9.7, none of which holds every such signature. The design makes no such claim for a signature with seven or eight corners and a corner of order 1000, where some margins fall below 8. Each of those signatures still passes every check, so `build_with` returns it (DOM-44). The named polygon `*(1000)2222222` is one of them.

A margin below 8 that a later measurement finds inside the claim narrows the claim, and not the set of signatures that DOM-38 admits or the range of DOM-45, because the guard of DOM-44 already refuses every polygon that fails a check. A change MUST NOT raise a tolerance to restore a margin, because the tolerances separate an order from its neighbour at the limit (Section 8.3). This is a choice of this design, which Section 15.3 puts to the owner.

Two temporary tests of this crate, which are not committed, swept the doubles of the default shapes in a release build. The guard refused no starred polygon of the doubles that they built, which are every rotation signature with four to eight orders from 2 to 12 other than `2222`, every rotation signature with four to eight orders whose first order is 1000 and whose other orders come from $\{1000, 7, 3, 2\}$, and the regular signatures with $n$ equal orders $p$ for $p$ from 2 to 1000 and $n$ from 4 to 8. It refused some doubles of these sweeps, every one by check 4 alone, and every one with eight orders: the doubles with five or six orders of 1000, such as `(1000)(1000)(1000)(1000)(1000)(1000)77` and `(1000)(1000)(1000)(1000)(1000)332`, and some regular doubles with eight equal orders between 631 and 999. With four to seven orders, it refused no double of the sweeps.

The refusals come from the storage of the pairings in `f64`, and not from the arithmetic of check 4: on the four doubles of a temporary diagnostic, the cycle relators evaluated from the unrounded products of the two stored reflections that each pairing rounds had a residual about ten times smaller than from the stored pairings. A pairing with entries of a few hundred, rounded to `f64`, keeps an error of about $3 \times 10^{-14}$ in each entry. The cycle of two vertices $(R_j R_1)(R_1 R_{j+1})$ cancels to a small rotation, so this error is large against its result, and the corner frame of a vertex far from the centre and the power of an order up to 1000 amplify it. The pairings are already the compensated products of DOM-25, so no rounding to `f64` of these products removes the error. The renderer folds with the same stored pairings.

A rotation signature whose double the guard refuses is therefore refused at its default shape, which is its only shape, with the error `Numerical` and the text of a refused double of Section 10.2. This design raises no tolerance to admit it, because this section forbids raising a tolerance to restore a margin. Removing these refusals needs another form of the stored pairings, which item 40 of Section 15.3 puts to the owner.

This design claims a margin of at least 8 for every check of the doubles of polygons with four to eight orders from 2 to 12, where the sweep holds every such signature. It claims no margin for the doubles with an order above 12, because no sweep holds every such signature, and on the measured sets the smallest margin of such a double falls below 8 from six orders on. Each double that `build_with` returns passes every check, which is what the guard asserts (DOM-44). A double whose smallest margin is below 2 can pass or fail with the last bits of its computation, so the tests accept either outcome for it (Section 12.1). Of the named doubles, only `(1000)(1000)(1000)(1000)(1000)(1000)(1000)(1000)` is such a double.

For settings other than the default, the design claims no margin. A setting either builds a polygon that passes every check, or returns `Numerical` (DOM-44). No range keeps a margin of 8 for every setting with seven or eight corners (Section 6.12), and the tests of random polygons assert only this alternative.

The ten domains of Section 6.13 pass every check at every setting of the grid of `aspect` in $\{0.125, 0.3, 1, 1.618, 4, 8\}$, by `skew` in $\{-0.5, -0.3, 0, 0.25, 0.5\}$ for `o` and `2222`, and at random settings, with margins far above 8. A change MUST NOT loosen a tolerance for these domains. If the crate measures a margin below 8, the change narrows the range of `aspect` or `skew` or changes the construction.

The spherical group domains of Section 6.16 pass every check over the whole range of Section 4, with margins far above 8, and the domains of `*`, `x`, and `1` have the residual 0 in every check. A change MUST NOT loosen a tolerance for these domains. If the crate measures a margin below 8, the change narrows the limits of Section 4 or changes the construction.

The spherical triangles, lunes, and doubles of Sections 6.7 and 6.8 keep their residuals bit for bit (DOM-30).

DOM-30. The stored values of a reflection domain and the residuals of its checks MUST NOT depend on the code that builds and checks the doubles, the polygons, the wallpaper domains, and the spherical group domains, so that every measured value of a reflection domain holds bit for bit, because the golden images of `escher-render` compare reflection domains byte for byte, and the generating domain of each double must equal the reflection domain of its starred signature. A test checks the second part, and the tests of the reflection domains, which print their measured values, check the first.

The code of three and four vertices, of the Euclidean domains, and of the spherical domains does not depend on the measurement of DOM-46 for five or more vertices, on the double-double check 4 of DOM-49, or on the kinds of step 5 of Section 6.8. A comparison outside the tests found the raw bits of the angles, the area, and the residuals of the reflection triangles, the kites, the lunes and their doubles, and the Euclidean and spherical signatures identical with and without that code. No committed test repeats the comparison, because the last bits of the mathematical library can differ between the container and the runner of CI.

## 9. Numerical Robustness

### 9.1. Renormalization

- `hyperbolic::normalize_point` runs twice, in steps 6 and 10 of Section 6.1, each time on a sum of vertices.
- `hyperbolic::wall_through` divides each normal by its norm, and `euclidean::wall_through` divides each covector by the length of its first two coordinates (the geometry design).
- The Euclidean construction divides a sum of vertices by its last coordinate in steps 3 and 5 of Section 6.5.
- `spherical::normalize_point` runs twice for a spherical triangle, in steps 4 and 6 of Section 6.7, each time on a sum of vertices, and `spherical::wall_through` divides each normal by its length. The lune renormalizes nothing.
- The double of Section 6.8 renormalizes nothing. Its image vertices and normals are plain products of $R_1$ and the values of its generating domain.
- The polygon of Section 6.10 normalizes the sum of its vertices once in double-double arithmetic, in step 8, and once in `f64` for the centre, in step 10, and `hyperbolic::wall_through` divides each normal by its norm. It renormalizes no vertex.
- `verify()` renormalizes nothing (DOM-8).

DOM-10. The construction MUST NOT renormalize the vertices after step 8, after step 4 of Section 6.5, in step 2 of Section 6.8, or after step 9 of Section 6.10, because a renormalized vertex lies on the hyperboloid but far from the true corner.

A vertex therefore leaves the hyperboloid by a small amount, which is largest near the order limit. The wall through two vertices depends only on the plane that they span, so the normals are not affected.

### 9.2. Forms

- `hyperbolic::inverse` computes the geometric inverse $J M^t J$ (Section 6.1 of the geometry reference), never a general matrix inverse.
- The relators are evaluated in the corner frame (Section 8.2, check 4).
- The corner angles come from the distances between the vertices (Section 7).
- $\operatorname{arcosh} X$ in step 4 needs no special form. The smallest $X$ that the construction computes over the supported triples is that of `*732`, about 1.04.
- $\operatorname{arcosh} X$ and $\operatorname{arsinh} X$ in step 5 of Section 6.10 need no special form over the range of Section 6.6 (DOM-41). A gap that is not finite or not positive gives `Numerical` in step 12.
- The walk and the placement of a polygon run in double-double arithmetic (DOM-42), and the angles of five or more vertices come from `atan2` in the corner frame (DOM-46).
- $\arccos X$ in step 2 of the spherical triangle is clamped to $[-1, 1]$. For `*n22` with a large $n$, $X_b$ comes close to 1, where the arccosine is ill-conditioned, and $\cos(\pi/2)$ is not 0 in `f64`, so Section 9.5 bounds the triangle up to $n = 1000$.

### 9.3. Accuracy over the Supported Range

DOM-11. The values that `build_with` returns MUST lie within the bounds of this table, or for a Euclidean domain of Section 9.4, for a spherical domain of Section 9.5, for a polygon and its double of Section 9.7, for one of the ten domains of Section 6.13, of Section 9.8, and for a double also of Section 9.6, of the values of the exact polygon in the final position, because the renderer and `escher info` use them without a check, and the reference-arithmetic tests of Section 12.1 assert these bounds.

The rows for the side length, the corner angle, and the area bound the values that `side_lengths()`, `corner_angles()`, and `area()` return, which the crate measures from the stored vertices (Section 7), with lengths and areas at curvature $-1$ and angles in radians.

| Quantity | Measure | Orders up to 12 | $(1000, 3, 2)$ | Every supported triple |
| --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error divided by the largest coordinate | $10^{-13}$ | $10^{-11}$ | $10^{-8}$ |
| Normal | Largest coordinate error | $10^{-13}$ | $10^{-11}$ | $10^{-8}$ |
| Side length | Absolute error | $10^{-13}$ | $10^{-11}$ | $10^{-8}$ |
| Centre | Largest coordinate distance from `hyperbolic::ORIGIN` | $10^{-13}$ | $10^{-11}$ | $10^{-9}$ |
| Corner angle | Absolute error | $10^{-9}$ | $10^{-9}$ | $10^{-9}$ |
| Area | Absolute error | $10^{-9}$ | $10^{-9}$ | $10^{-9}$ |

The bounds are choices of this design. The tests measure each quantity against the reference arithmetic of Section 12.2, assert each bound, and print the largest error with its triple and its margin. They also measure the distance from vertex 0 to vertex 1, which the construction never sets, against the bounds of the side length, and the error of the area against the area of the reference triangle. The column for orders up to 12 covers all hyperbolic triples with orders up to 12, and the column for every supported triple covers the set of triples of Section 12.1, which adds the named triples with an order above 12.

No measurement covers the vertices, normals, and side lengths of every supported triple. The column "Every supported triple" bounds them on the strength of the set of triples and of a reference sweep outside the tests over tens of thousands of triples, which found no larger error. The sweep of Section 8.4 over every supported triple needs no reference for the centre, whose exact value is `hyperbolic::ORIGIN`, and measures the angles and the area only through checks 5 and 6.

Near the order limit, the vertices of this crate lie far from `hyperbolic::ORIGIN`, and Section 9.1 states that they leave the hyperboloid. Section 8.4 of the geometry design states that every result of `hyperbolic::side`, `hyperbolic::distance`, `hyperbolic::wall_through`, `hyperbolic::translation`, and `hyperbolic::reflection` on these vertices and the centre was within the bound of its row, although some of these inputs lie outside the ranges of the rows. It states that its tests do not measure `hyperbolic::normalize_point` on the sums of vertices of steps 6 and 10, or `hyperbolic::isometry_defect` on the pairings. The centre and check 1 therefore rest only on the end-to-end measurements of this section and of Section 8.4, and the bounds above keep the end-to-end margins for every quantity.

### 9.4. Accuracy of the Euclidean Domains

The Euclidean domains of the tests are those of `*632`, `*442`, and `*333`, and the rectangle of `*2222` at `aspect` 1, 1.618, 4, 0.125, and 8, and at 64 values spaced logarithmically from 0.125 to 8. Over these domains, the tests compare each quantity of this table except the last with the reference construction of Section 12.2, assert each bound, and print the largest error with its domain and its margin. The bounds are choices of this design:

| Quantity | Measure | Bound |
| --- | --- | --- |
| Vertex | Largest coordinate error | $2 \times 10^{-15}$ |
| Covector | Largest coordinate error | $2 \times 10^{-15}$ |
| Pairing | Largest entry error | $4 \times 10^{-15}$ |
| Side length | Absolute error | $4 \times 10^{-15}$ |
| Corner angle | Absolute error | $2 \times 10^{-15}$ |
| Area | Absolute error, against the area of the reference polygon | $2 \times 10^{-15}$ |
| Centre | Largest coordinate distance from `euclidean::ORIGIN` | $10^{-15}$ |
| Aspect of the rectangle | Relative error of the length of side 0 divided by that of side 1, against `aspect` | $4 \times 10^{-15}$ |
| SymmHub lines | Largest coordinate error of each wall of a triangle against its line of SymmHub, up to sign (Section 12.1) | $2 \times 10^{-15}$ |

The three lines of SymmHub match sides 0, 2, and 1, in that order, for each of the three triangles.

At `aspect` 1 and 4, every number of the construction of the rectangle is a dyadic rational and every operation is exact in IEEE 754 arithmetic, and the square roots of 1 and 4 are exact. The tests therefore assert exactly that the vertices are $(\pm w/2, \pm h/2, 1)$ and the covectors are $(\pm 1, 0, -w/2)$ and $(0, \pm 1, -h/2)$, as TEST-13 of the coding standard allows.

DOM-19. A test that asserts a bound on a quantity of a domain MUST name its input set and print its largest value with its input and its margin, as TEST-13 of the coding standard requires. This document MUST record a bound only where it is part of a contract of the crate, because the test repeats the measurement on every run, while a figure copied into this document goes stale with the next change to the code.

### 9.5. Accuracy of the Spherical Domains

The spherical domains of the tests are those of `*332`, `*432`, `*532`, and `*222`, and of `*n22` and `*nn` for $n$ = 2, 3, 5, 12, 100, 999, and 1000. Over these domains, the tests compare each quantity of this table with the reference construction of Section 12.2, assert each bound, and print the largest error with its domain and its margin. The bounds are choices of this design:

| Quantity | Measure | Bound |
| --- | --- | --- |
| Vertex | Largest coordinate error | $4 \times 10^{-13}$ |
| Normal | Largest coordinate error | $10^{-13}$ |
| Side length | Absolute error | $4 \times 10^{-13}$ |
| Centre | Largest coordinate error against the reference centre | $2 \times 10^{-15}$ |
| Corner angle | Absolute error against $\pi/n_j$ | $2 \times 10^{-13}$ |
| Area | Absolute error against $2\pi\chi$ | $4 \times 10^{-13}$ |

The vertices and the centre of a lune are exact values that the construction stores, and the dot product of its two vertices is exactly $-1$. The tests assert these equalities exactly, as TEST-13 of the coding standard allows, and compare the side lengths of a lune with $\pi$ within the bound of the side length.

### 9.6. Accuracy of the Rotation Domains

The vertices, normals, and covectors that a double takes from its generating domain are those of Sections 9.3 to 9.5 bit for bit (DOM-30), and so is its centre. This section bounds what the doubling adds: the image vertices, normals, and covectors, the pairings of DOM-25, and the measurements of Section 7.

The doubles of the tests are those of the cone triples of the set of triples of Section 12.1, of `632`, `442`, `333`, and of `2222` at the values of `aspect` of Section 9.4, and of `332`, `432`, `532`, `222`, and of `n22` and `nn` for $n$ = 2, 3, 5, 12, 100, 999, and 1000. The tests compare each quantity of this table with the reference of Section 12.2, which builds the double itself, assert each bound, and print the largest error with its domain and its margin. The bounds are choices of this design, each the smallest value of the form 1, 2, or 4 times a power of 10 that is at least 8 times the largest error measured, except the area bound of the spherical doubles, which is the area bound of Section 9.5:

| Quantity | Measure | Hyperbolic kites | Euclidean doubles | Spherical doubles |
| --- | --- | --- | --- | --- |
| Image vertex | Largest coordinate error, relative to the largest coordinate for a hyperbolic domain | $4 \times 10^{-9}$ | $10^{-14}$ | $4 \times 10^{-13}$ |
| Image normal or covector | Largest coordinate error of the normal of a hyperbolic or spherical double, and of the covector of a Euclidean double | $10^{-8}$ | $10^{-14}$ | $2 \times 10^{-13}$ |
| Pairing | Largest entry error against the product, in the reference arithmetic, of the two reflections that the domain stores | $2 \times 10^{-13}$ | $10^{-15}$ | $10^{-15}$ |
| Isometry defect of a pairing | The value of the isometry check of the geometry | $2 \times 10^{-10}$ | $4 \times 10^{-15}$ | $2 \times 10^{-14}$ |
| Side length | Absolute error | $2 \times 10^{-9}$ | $10^{-14}$ | $4 \times 10^{-13}$ |
| Corner angle | Absolute error against the copies times $\pi$ divided by the order | $10^{-11}$ | $4 \times 10^{-15}$ | $10^{-12}$ |
| Flat vertex | Absolute error of the angle against $\pi$ | Not applicable, because no hyperbolic kite has a flat vertex | $10^{-15}$ | $10^{-12}$ |
| Area | Absolute error against the area of the signature, or for a Euclidean double against the area of the reference polygon | $2 \times 10^{-11}$ | $2 \times 10^{-15}$ | $4 \times 10^{-13}$ |

The row of the image normal or covector compares the covector of a Euclidean double, because its normal is derived from its covector (step 3 of Section 6.8). The error of a flat vertex of a Euclidean double is measured against the exact $\pi$, so it includes the difference between the `f64` value of $\pi$ and $\pi$.

The pairing row compares with the product of the two `f64` reflections that the domain stores, because DOM-25 computes that product and not the product of the reflections of the reference. The tests also assert that every entry of every pairing is that exact product rounded to the nearest `f64`, which the plain product fails on many entries. The isometry row is the defect that remains with the compensated product, which lowers the defect at some doubles and raises it at others (DOM-25).

### 9.7. Accuracy of the Polygons and Their Doubles

The tests of the polygons use these sets. The file `tests/domain/shared.rs` defines the named and the small polygons, so that the unit tests use them too:

- The named polygons: `*3222`, `*3322`, `*3333`, `*4444`, `*5432`, `*99994`, `*22222`, `*33333`, `*222222`, `*444444`, `*2222222`, `*8765432`, `*22222222`, `*33333333`, `*44444444`, `*(12)(12)(12)(12)(12)(12)(12)(12)`, `*(12)(12)333333`, `*(1000)222`, `*(1000)2222`, `*(1000)(1000)(1000)(1000)`, `*(1000)2222222`, and `*(1000)(1000)(1000)(1000)(1000)(1000)(1000)(1000)`.
- The small polygons: every canonical hyperbolic signature of one boundary with four corners of orders from 2 to 7, five corners of orders from 2 to 7, six of orders from 2 to 6, seven of orders from 2 to 5, or eight of orders from 2 to 4. The test asserts and prints its own count.
- The settings of a polygon: the default shape, each parameter alone at 0.5, 1, 2, 3, and 3.5 with the others at their defaults, and 32 settings drawn uniformly from the range with a fixed seed. Each test draws them with a seed of its own, so the numbers of refused settings differ between the tests.
- The doubles of polygons: the rotation signatures of the named polygons and of the small polygons, at the default shape. The double of `*(1000)(1000)(1000)(1000)(1000)(1000)(1000)(1000)` is the one double whose smallest margin is below 2, which the tests accept as built or refused (Section 8.4).
- The default sweep: the canonical signatures with four corners of orders from 2 to 12, five of orders from 2 to 9, six of orders from 2 to 6, seven of orders from 2 to 5, or eight of orders from 2 to 4, and every signature of four to eight corners with at least one corner of order 1000 and the other orders from $\{2, 3, 7\}$. No committed test runs it. A program outside the repository ran it with the code of this crate, and its signatures with a corner of order 1000 are more than this definition gives, so a rerun from this definition does not reproduce that set exactly. The set of every canonical signature with four to six corners, at least one of order 1000, and the other orders from $\{2, 3, 7, 1000\}$, which Section 8.4 names, is the one that a test of the crate measures.

The tests compare each quantity of this table with the reference of Section 12.2, assert each bound, and print the largest error with its signature, its setting, and its margin. The polygons at every setting are the named polygons at each of their settings. The polygons with orders up to 12 are the named polygons whose orders are all at most 12, at their default shapes, in a column of their own, because the bounds over all settings are set by polygons with a corner of order 1000 and would hide a loss of accuracy on ordinary polygons. The doubles are those of the named polygons at their default shapes other than the double at the limit, so that the bounds do not depend on whether it builds. A setting that the guard refuses is counted and asserted to give `Numerical`, and contributes no error. Each bound is the smallest value of the form 1, 2, or 4 times a power of 10 that is at least 8 times the largest error measured:

| Quantity | Measure | Polygons at every setting | Polygons with orders up to 12 at the default shape | Doubles of polygons without the double at the limit |
| --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error, relative to the largest coordinate | $4 \times 10^{-10}$ | $10^{-13}$ | $2 \times 10^{-13}$ |
| Normal | Largest coordinate error | $2 \times 10^{-7}$ | $4 \times 10^{-13}$ | $10^{-11}$ |
| Side length | Absolute error | $2 \times 10^{-8}$ | $4 \times 10^{-13}$ | $4 \times 10^{-12}$ |
| Closing side | Absolute error of the distance from vertex $n - 1$ to vertex 0 against the length of side 0 of step 6 of Section 6.10, in the reference | $4 \times 10^{-12}$ | Not applicable | Not applicable |
| Centre | Largest coordinate distance from `hyperbolic::ORIGIN` | $10^{-15}$, tighter than the $10^{-9}$ of Section 3.2 | $10^{-15}$ | $10^{-15}$, and bit for bit that of the polygon |
| Corner angle | Absolute error against the copies times $\pi$ divided by the order | $10^{-10}$ | $10^{-13}$ | $4 \times 10^{-12}$ |
| Flat vertex | Absolute error of the angle against $\pi$ | Not applicable | Not applicable | $4 \times 10^{-13}$ |
| Area | Absolute error against the area of the signature | $10^{-10}$ | $10^{-13}$ | $10^{-11}$ |

The tests also check in the reference arithmetic:

- The identities of DOM-48: the angles measured from the reference vertices equal $\pi/n_j$, and $(n - 2)\pi$ minus their sum equals $-2\pi\chi$ of `escher-signature`.
- The double-double functions of DOM-42 against the reference over the lengths from 0 to 30 in steps of $1/64$ and over $\pi/n$ for $n$ from 2 to 1000, with $\cos$ and $\sin$ measured in absolute terms, because $\cos(\pi/2)$ is 0. They also check that the double-double $\pi$ lies within 2 units of $2^{-106}$ of the reference $\pi$, and that the sum, the product, the quotient, and the square root are exact on representable results.
- Each length parameter against the length that it names on the built polygon, and $\lvert g(d) - \pi \rvert$ at the radius of the bisection of DOM-45.
- The meaning of a corner pair and of a slice length of Section 6.6 (`UNV-7` and `UNV-27`) on the named polygons at all their settings, including those that the guard refuses.
- The walk against SymmHub's welded layout of the sheets, on the named polygons at their default shapes: the distances between vertices, the angle at each end of a welded slice, and the closure of each laid-out sheet.
- The inscribed polygon of the default shape: its angles, the distance of each side from the centre against the inradius, the slice lengths against the distance between their walls by search, and each default of the crate against its unrounded reference value within $5 \times 10^{-4}$, on the named polygons and some small polygons. An error of an unrounded default of the crate large enough to flip its rounding fails that allowance, so the tests detect it.

Two builds of the crate from different spellings of one signature are compared with a relative vertex bound of their own, because they compare the crate with itself and not with the reference.

A program outside the repository compared the stored vertices of the crate with a walk at 60 digits of the same `f64` side lengths, placed the same way, and found them within one unit in the last place, so the double-double walk does not limit the accuracy of the polygons. The `f64` side lengths of step 6 do move individual residuals, but over the default sweep the share of margins below 8 stays about the same with exact lengths, so the reading of the `f64` lengths as harmless holds over the sweep and not signature by signature.

A program outside the repository ran the default sweep. The guard refuses no default shape, and the binding check is always check 1 or check 2, the isometry defect of a reflection in a wall far from the centre. For five to eight corners, the exact polygon rounded once has the same smallest margins as the crate, so the margins below 8 with seven or eight corners and a corner of order 1000 come from storing the polygon in `f64`, and not from the walk. Section 8.4 states the margin that the design claims from these figures.

The tests measure the corner angles and the area of DOM-46 on polygons built through private access: the regular polygons with $n$ from 5 to 8 corners and every corner of the order $k$, for $k$ in $\{2, 3, 4, 7, 12, 1000\}$, and their doubles. The vertices of the tile of $\{n, 2k\}$ come from equation 24 of [HT], computed in the reference arithmetic of Section 12.2 and rounded once to `f64`, and each wall goes through two rounded vertices. The expected angle is the copies times $\pi/k$, and the expected area is $(n - 2)\pi - n\pi/k$, or twice that for a double. The errors of the doubles are larger than those of the polygons, because the image vertices are plain `f64` products by $R_1$. At the flat vertices of the doubles of `*22222` to `*22222222`, the tests print, and do not assert, the error of the law of cosines in the triangle of the two neighbours, which exceeds the tolerance of check 5. Every regular polygon and every double passes checks 5 and 6, and the doubles with every corner of order 1000 may fail check 4 alone, which the tests print. Their walls are not those of Section 6.10.

A deterministic companion of the property test of random polygons counts, for each number of corners from 4 to 8, the settings that the guard refuses among its draws, with lengths uniform in 0.5 to 3.5 and orders from 2 to 12, and asserts that no count exceeds twice a recorded count, so that a change that makes the construction less accurate fails it.

The tests compare the regular polygons with equations 24 and 29 of [HT] (Section 10.4 of the geometry reference): each side against equation 29 at $(n, 2p)$, and the disk radius of each vertex against equation 24.

### 9.8. Accuracy of the Wallpaper Domains

The tests of this section cover the ten domains of Section 6.13 over the grid of Section 8.4: `aspect` in $\{0.125, 0.3, 1, 1.618, 4, 8\}$, and for `o` and `2222` also `skew` in $\{-0.5, -0.3, 0, 0.25, 0.5\}$, with the kites `4*2` and `3*3` once each. The reference of Section 12.2 builds each domain from its closed form in 256-bit arithmetic: the vertices of the table of DOM-50, the walls through them, and the pairings from their definitions as matrices.

The tests compare each quantity of the table below with the reference, assert each bound, and print the largest error with its domain, its setting, and its margin. Each bound is the smallest value of the form 1, 2, or 4 times a power of 10 that is at least 8 times the largest error that the crate measures. The tests bound `2222`, at every value of `skew`, in a set of its own, because most of its largest errors exceed those of every other domain, so one set of bounds over all the domains would hide a loss of accuracy on the others. They bound the kites in a third set, because their covector and pairing errors exceed those of the first set. The shear rows cover the 24 settings each of `o` and `2222` with `skew` other than 0:

| Quantity | Measure | First set: `o`, `**`, `xx`, `*x`, `22*`, `22x`, and `2*22` | Second set: `2222` at every value of `skew` | Third set: `4*2` and `3*3` |
| --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error | $4 \times 10^{-15}$ | $10^{-14}$ | $10^{-15}$ |
| Covector | Largest coordinate error | $2 \times 10^{-15}$ | $10^{-14}$ | $4 \times 10^{-15}$ |
| Pairing | Largest entry error | $4 \times 10^{-15}$ | $10^{-14}$ | $10^{-14}$ |
| Side length | Absolute error | $4 \times 10^{-15}$ | $10^{-14}$ | $10^{-15}$ |
| Angle sum of a cycle | Absolute error against $2\pi/m_c$ | $10^{-14}$ | $10^{-14}$ | $2 \times 10^{-15}$ |
| Area | Absolute error against 1, against 2 for `2222`, or against the area of the reference kite | $10^{-15}$ | $10^{-15}$ | $10^{-15}$ |
| Shear vertex | Largest coordinate error of a sheared domain, `o` only in the first set | $2 \times 10^{-15}$ | $4 \times 10^{-15}$ | Not applicable |
| Shear area | Absolute error of a sheared domain, `o` only in the first set | $10^{-15}$ | $10^{-15}$ | Not applicable |

The bounds of the second and third sets apply to the reference test alone. The tolerances of the checks are those of DOM-9.

Every set also holds these exact equalities:

- Flat vertex: `corner_angles()` equals `core::f64::consts::PI` at every flat vertex of `22*`, `2*22`, and `2222` at each of their settings. `core::f64::consts::PI` differs from $\pi$ by about $1.2 \times 10^{-16}$, so that is the error of a flat vertex against $\pi$.
- Centre: `centre()` equals `euclidean::ORIGIN` for every domain other than a kite, and each kite has the centre of its built `*442` or `*632`.

The kites keep the vertices, the normals, the covectors, and the pairings of sides 2 and 3 of the kites of `442` and `632` bit for bit, so their errors are those of Section 9.6 except for side 1, whose reflection is computed from its covector (DOM-51).

At `aspect` 1 and 4, and at `skew` 0.25 for `o` and `2222`, every vertex of a rectangular domain, of `o`, and of `2222` is a dyadic rational, and every translation, half-turn, and glide reflection has dyadic entries. The unit tests assert these vertices and pairings exactly, as TEST-13 of the coding standard allows. They assert each half-turn at every setting, against the matrix of DOM-51 built from its stored centre, entry for entry. They assert each glide reflection only at the dyadic settings, against a matrix written out in the test, because a comparison with the product that builds it would only restate the construction.

### 9.9. Accuracy of the Spherical Group Domains

The spherical kites keep the vertices, the normals, the covectors, the centre, and the kept pairings of their doubles bit for bit, so their errors are those of Section 9.6 except for the image mirror side, side 1 for `Low` and side 2 for `High`, whose reflection is computed from its normal (DOM-59). The domains of `nx` keep the vertices, the normals, and the centre of the lune of `*nn` bit for bit, so their errors are those of Section 9.5 except for the pairings, which are exact products of the stored reflections. The domains of `x` and `*` have exact entries, and the whole sphere stores no value but its centre and its area.

The tests of this section run on the spherical group test set of Section 2.1. The reference of Section 12.2 builds each kite as it builds a double, with the mirror reflections from the reference normals, and each lune of `nx` from the reference values of $\cos(\theta/2)$ and $\sin(\theta/2)$, with the pairings $H R_i$ written componentwise. The products of the breadth-first closure of the certificate of DOM-64 drift from the exact group elements by more than the largest residual of check 4, as a prototype outside the repository found at the limits, and that drift is not a residual of `verify()`.

The tests compare each quantity of this table with the reference, assert each bound, and print the largest error with its domain and its margin. Each bound is the smallest value of the form 1, 2, or 4 times a power of 10 that is at least 8 times the largest error that the crate measures. The kites are `3*2`, and `n*` and `2*n` of the test set. The lune groups are `x`, `*`, `1`, and `nx` of the test set:

| Quantity | Measure | Kites | Lune groups |
| --- | --- | --- | --- |
| Vertex | Largest coordinate error | $4 \times 10^{-13}$ | Exactly equal, asserted bit for bit |
| Normal | Largest coordinate error | $2 \times 10^{-13}$ | $10^{-15}$ |
| Pairing | Largest entry error | $4 \times 10^{-13}$ | $2 \times 10^{-15}$ |
| Side length | Absolute error | $4 \times 10^{-13}$ | $10^{-15}$ |
| Angle sum of a cycle | Absolute error against $2\pi/m_c$ | $2 \times 10^{-12}$ | $4 \times 10^{-13}$ |
| Flat vertex | Absolute error of the angle against $\pi$ | $10^{-12}$ | $10^{-15}$, for `x` and `*` |
| Area | Absolute error against the area of the signature | $4 \times 10^{-14}$ | $4 \times 10^{-13}$ |
| Centre | Equality with the centre of the built double, or with `spherical::ORIGIN` for the lunes and the whole sphere | Exactly equal, asserted bit for bit | Exactly equal, asserted bit for bit |

The tests also assert the largest entry of $P_0 P_1 - I$ over `nx` of the test set against the bound $4 \times 10^{-15}$. They print, and assert no bound on, the difference between the stored normals of the two mirror sides of `n*`, which lie on one great circle up to rounding (Section 6.16).

The tests assert these values exactly, as TEST-13 of the coding standard allows for values that the construction stores: the pairings $-I$ of `x` and $\operatorname{diag}(1, 1, -1)$ of `*`, the normals $(0, 0, -1)$ of the lune of order 1, each pairing of `nx` as $H R_i$ of its stored reflection entry for entry, the vertices $(0, \pm 1, 0)$ of every lune, and the area $4\pi$ of the whole sphere as `4.0 * PI`.

## 10. Errors

### 10.1. Structure

DOM-12. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"derive side presentation"` for an error of `side_presentation()` and `"build fundamental domain"` for every other error of this crate.
- `detail`: A private enum with one variant for each cause, which carries the data that the text of Section 10.2 shows: the canonical string of the signature, the order over the limit with the limit that it exceeds, 1000 or the 500 of DOM-63, the number of orders over the limit, the identity of a parameter from which its name and range derive, the error of `escher-geometry` that caused a numerical failure, the refusal of the guard of DOM-44 for a polygon or for a double, the refusal of the guard of DOM-54 for one of the ten domains of Section 6.13, the refusal of the guard of DOM-60 for a spherical group domain, the structural defect of a domain without a side presentation, or the error of `escher-group` that rejected a relator or a side presentation.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, because a derived `source()` can return only a field of `Error`, not the `escher-geometry` error that one variant of the detail holds, and a separate source field beside the detail would hold one fact in two fields. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `escher-geometry` error for a `Numerical` error that one caused, the `escher-group` error for every `InvalidPairing` error whose relator or presentation `escher-group` rejected, and `None` otherwise, which includes the four structural causes of `InvalidPairing`. No type from a dependency appears in the public surface of the error (ERR-2).

`InvalidPairing` is an error and not a panic for the reason that `Numerical` is: no domain that `build_with` returns produces it, but `side_presentation()` runs on any value of the type, and `escher info` prints its result (DOM-35).

`Numerical` is an error and not a panic. No supported triangle, Euclidean or spherical domain, or double of one is known to produce it, but the construction calls fallible functions of `escher-geometry`, and a panic would stop `escher info`, so returning an error is the sounder option (ERR-5 of the coding standard, Hard Rule 7 of `AGENTS.md`). A polygon produces it when the guard of DOM-44 refuses the shape, which is an expected outcome for settings near the ends of the range (Section 9.7). The double of a polygon produces it when the guard refuses the double, which happens for some signatures with eight orders of which several are large (Section 8.4). One of the ten domains of Section 6.13 produces it when the guard of DOM-54 refuses it, and a spherical group domain when the guard of DOM-60 refuses it, which no setting or order in range is known to cause (Section 8.4).

### 10.2. Kinds and Text

DOM-13. `Display` for `Error` MUST be `cannot derive side presentation: ` for the kind `InvalidPairing` and `cannot build fundamental domain: ` for every other kind, followed by the text of the detail in this table, because `escher-cli` prints it and its snapshots fix it. The exceptions are the texts of `UnknownParameter` and `ParameterOutOfRange`, which `escher-cli` replaces with a diagnostic of its own (Section 6.8 of `docs/design/cli.md`).

| Kind | When | Detail carries | Text after the prefix |
| --- | --- | --- | --- |
| `InvalidPairing` | DOM-35 | The cause: a number of corners other than the number of sides, more than 256 sides, a walk that does not close, a cycle with unequal orders or copies, or the `escher-group` error that rejected a relator or the presentation | `the domain has a different number of corners than of sides`, `the domain has more than 256 sides`, `the side pairings do not close a vertex cycle`, `a vertex cycle joins corners of different orders`, or `the side relators do not form a presentation` |
| `UnsupportedSignature` | The predicate of Section 4 fails, which includes every general non-compact signature (`DEC-19`) | The canonical string | `the signature 4*3 is not supported`, with the canonical string of the signature in place of `4*3` |
| `LimitExceeded` | A supported signature has more than eight orders (DOM-38) | The number of orders | `the signature has 9 orders, more than 8`, with the number of orders in place of `9` |
| `LimitExceeded` | A corner order of a supported signature exceeds 1000 (DOM-3), or the order $n$ of `2*n` exceeds 500 (DOM-63) | The largest order and the limit | `order 1001 is larger than 1000`, with the largest order in place of `1001` and the limit in place of `1000`, such as `order 501 is larger than 500` for `2*(501)` |
| `Numerical` | Step 11 of Section 6.1, step 7 of Section 6.5, step 7 of the triangle or step 4 of the lune of Section 6.7, step 8 of Section 6.8, step 12 of Section 6.10, step 8 of Section 6.13, or step 8 of Section 6.16 | The `escher-geometry` error, if one caused it | `the construction is numerically degenerate` |
| `Numerical` | The guard of DOM-44, step 13 of Section 6.10, for a polygon that fails a check | Nothing | `the shape is too extreme to build accurately` |
| `Numerical` | The guard of DOM-44 for the double of a polygon that fails a check (DOM-47) | Nothing | `the side pairings of the domain cannot be stored accurately enough` |
| `Numerical` | The guard of DOM-54, step 9 of Section 6.13, for one of the ten domains that fails a check | Nothing | `the wallpaper domain fails its verification` |
| `Numerical` | The guard of DOM-60, step 9 of Section 6.16, for a spherical group domain that fails a check | Nothing | `the spherical domain fails its verification` |
| `UnknownParameter` | Step 3 of DOM-18 | The canonical string | `the signature *632 has no shape parameter with one of the given names`, with the canonical string of the signature in place of `*632` |
| `ParameterOutOfRange` | Step 4 of DOM-18 | The identity of the parameter, a private enum from which the name, the minimum, and the maximum of its `ParameterSpec` derive, so that `Error` keeps `Eq` | `the parameter aspect is outside its range of 0.125 to 8`, with the name, the minimum, and the maximum in place of `aspect`, `0.125`, and `8`, such as `the parameter slice1.length is outside its range of 0.5 to 3.5` |

The text `a vertex cycle joins corners of different orders` also covers a cycle whose copies do not sum to 2. The text `the side relators do not form a presentation` always has the `escher-group` error as its `source()`. The texts of `InvalidPairing` name no side and no number other than the fixed limit of 256 sides, because no supported input reaches them, and the tests that reach them through private access assert the kind and the full text.

The texts of `UnknownParameter` and `ParameterOutOfRange` contain no name or value of a setting, because a caller's text may hold any character, and the printed name is that of the `ParameterSpec`. They write each number with the `{}` format of `f64`, which prints the two ends of the range of `aspect` as `0.125` and `8`, and those of `skew` as `-0.5` and `0.5`, so the text of `skew` is `the parameter skew is outside its range of -0.5 to 0.5`.

So `FundamentalDomain::build` on `4*3` returns an `Error` whose `kind()` is `UnsupportedSignature` and whose `Display` text is exactly `cannot build fundamental domain: the signature 4*3 is not supported`. On `*(1001)32` it returns `LimitExceeded` with the text `cannot build fundamental domain: order 1001 is larger than 1000`, and on `*222222222` the text `cannot build fundamental domain: the signature has 9 orders, more than 8`. The text of the guard names no check, because a check residual means nothing to a user who chose a length, and the text is enough for the user to choose a shape nearer the default. A double has no shape to choose, so its text names the cause, the side pairings that `f64` cannot store accurately enough (Section 8.4), and names no remedy. The text of a refused wallpaper domain or spherical group domain names the verification, because no setting or order in range is known to fail it, so a refusal would be a defect of the crate and not a shape to change. The text of a refused spherical group domain is `DEC-60` of the architecture. The private details of the guard carry nothing, so their `source()` is `None`. What `escher-cli` prints for either, and its exit status, are for the command-line design to state.

Every fixed text is in lower case and has no trailing punctuation (ERR-4 of the coding standard). The canonical string contains only printable ASCII (Section 7.3 of the signature design), so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

## 11. Resource Limits

The crate limits the number of orders and the largest corner order, which `build_with` checks at step 2 of DOM-18, and the value of each shape parameter, which it checks at step 4. Section 16 states each value and its reason, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 12. Testing

### 12.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-domain/tests/domain/main.rs`, with the modules `common`, `shared`, `reference`, `triangles`, `regular`, `properties`, `unsupported`, `euclidean`, `parameters`, `spherical`, `rotation`, `presentation`, `polygons`, `wallpaper`, and `spherical_groups` (TEST-10). Its root has a `//!` line. The snapshot tests of the domain summary live in `escher-cli`, and the command-line design specifies them. The golden images belong to `escher-render`.

The unit tests and the integration tests share the helpers of `tests/domain/shared.rs`. They build a signature and a domain from text, give the text of each relator and of an order, and hold the named and the small polygons of Section 9.7 and the grid of Section 8.4. `src/lib.rs` loads that file and the reference arithmetic of `tests/domain/reference.rs` as test-only modules through a `#[path]` attribute, with the alias `extern crate self as escher_domain`, so that the files name the crate the same way in both test targets. The reference module carries `#[expect(dead_code)]`, because the unit tests use only a part of it. `tests/domain/common.rs` holds the helpers that only the integration tests use, such as the sets of rotation polygons, kites, and lune groups.

The module `common` defines the sets of triples:

- The plan triples: $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(1000, 3, 2)$.
- The 24 worst cases: $(995, 983, 970)$, $(994, 991, 970)$, $(997, 996, 970)$, $(997, 980, 973)$, $(992, 990, 979)$, $(995, 987, 976)$, $(928, 4, 3)$, $(999, 7, 6)$, $(998, 4, 4)$, $(1000, 4, 4)$, $(1000, 1000, 2)$, $(1000, 1000, 1000)$, $(868, 4, 4)$, $(923, 6, 5)$, $(998, 981, 961)$, $(998, 998, 958)$, $(997, 931, 906)$, $(992, 991, 975)$, $(989, 976, 974)$, $(978, 977, 971)$, $(985, 985, 971)$, $(973, 972, 964)$, $(979, 979, 971)$, and $(987, 984, 970)$. They include every triple at which the sweeps of Sections 8.4 and 9.3 have a largest value.
- The 30 named triples: the plan triples and the worst cases.
- The orders up to 12: all 269 hyperbolic triples $n_0 \ge n_1 \ge n_2$ with orders up to 12.
- The set of 294 triples: the orders up to 12 and the 25 named triples with an order above 12.
- The cone triples: the same triples read as the orders of rotation signatures, such as `732` for $(7, 3, 2)$, whose domains are kites.

The other named input sets are the Euclidean domains of Section 9.4, the spherical domains of Section 9.5, the doubles of Section 9.6, the polygons, settings, doubles, and regular polygons of Section 9.7, the grid of the ten domains of Sections 8.4 and 9.8, and the spherical group test set of Section 2.1.

The tests form these layers:

- Reference arithmetic. The tests compare every stored and measured value of the domains of each set with the independent construction of Section 12.2 and assert the bounds of Sections 9.3 to 9.9. They also compare the polygons with equations 24 and 29 of [HT] and the walls of the Euclidean triangles with SymmHub's lines, and evaluate the identities of DOM-48 that share no formula with the construction. They catch a wrong formula, a wrong placement, and a loss of accuracy.
- Checks and margins. The tests run the six checks on every domain of the sets, assert that each passes, assert a margin of at least 8 where Section 8.4 claims one, and print the largest residual of each check with its input and its margin. They catch a construction that drifts toward its tolerance.
- Property tests. `proptest` draws hyperbolic triples with orders up to 1000, weighted toward large orders and read as reflection and as rotation signatures, any text that parses as a signature, values of `aspect` and `skew` in their ranges, and polygons with four to eight corners, orders from 2 to 12 or from $\{900, 950, 1000\}$, and lengths in their range. They assert that a build succeeds and passes `verify()`, or returns the documented error, and never panics. Over the hyperbolic triples, they also assert that the vertices are counterclockwise and that the centre lies within $10^{-9}$ of the origin. A deterministic companion counts the settings that the guard of DOM-44 refuses and asserts that no count exceeds twice a recorded count. They catch a panic or a failure on an input that no table names.
- Table tests. The tests assert the exact structure of each class: the sides, vertices, partners, kinds, orientations, flat vertices, copies, and vertex cycles of the tables of Sections 6.8, 6.13, and 6.16, the features and sheets of Section 6.10, the parameters with their names, ranges, and defaults, and the spellings that reach each class. They assert exactly the values that are dyadic or that the construction stores, as TEST-13 of the coding standard allows. They catch a renumbering or a wrong pairing.
- Bit-for-bit tests. The tests assert that the generating domain of each double equals the reflection domain of its starred signature, that `2222` at `skew` 0 equals the double of Section 6.8, that the kites keep the entries of their doubles, that the pairings of the doubles are the correctly rounded products of DOM-25, and that the two rules of check 5 agree on `2222` at `skew` 0. They catch a change that moves the values of an existing class (DOM-30).
- Certificates of the groups. On the sphere, the exhaustive certificates of DOM-22 and DOM-31 enumerate the group of each lune and spherical double by a breadth-first closure with exactly `group_order()` elements, check that the images of the centre are distinct, at least a floor apart, and fold back, and fold a dense sample of the sphere into the domain. The wallpaper certificate of DOM-57 enumerates the elements near each of the ten domains, checks that sample points are covered exactly once, and checks the order of the point group, the covolume of the lattice, the centres of the rotations, and the glide reflections whose axes are not mirror lines, against control domains of known groups. The spherical certificate of DOM-64 checks the order by closure, the exactly-once tiling of a sample, and the census of each spherical group domain. They catch a domain of the wrong group, which `verify()` and the enumeration of the presentation do not.
- Presentation tests. The tests assert the text of the side presentations of Section 5.4, enumerate their cosets with `escher-group` and compare the order with `group_order()` on the sphere, drop each cycle relator in turn and check that the enumeration then stops at the coset limit, compare the colouring counts of the side presentation with those of the Conway presentation, and check that the corner words generate the subgroup of index 2. The evaluation of the relators of DOM-36 runs on every domain of its sets in the corner frame of check 4. They catch a dropped or wrong relator.
- Failure paths. The tests corrupt domains through private access with the defects of Section 8.3, and assert the checks that fail, the infinite residuals, and that no residual is NaN. They reach every cause and text of `InvalidPairing`, every error kind and text of Section 10.2, the guards of DOM-44, DOM-54, and DOM-60, the limits and their order, and the unsupported signatures of Section 4. They catch a check that misses a defect or a path that panics.
- Other tests. A compile-time assertion in `src/lib.rs` checks that each public type is `Send`, `Sync`, and `'static` (CONC-1 of the coding standard), the doctests run the examples of the documentation, and a self-check tests the reference arithmetic itself.

The tests that DOM-36, DOM-57, and DOM-64 name keep their names, because `docs/design/group.md` and `docs/design/poc-plan.md` cite them.

The census of a group counts, over the elements of its closure, the rotations, which include the identity, the elements that reverse orientation, the reflections, the presence of $-I$, and the axes of the rotations other than the identity, by order and by whether they lie on a mirror plane. A rotation is an element with the determinant 1, a reflection an element with the determinant $-1$ that is an involution with a fixed plane, and an axis is the line of the fixed points of a rotation. The tests assert the census of each group of the test set and the closed forms of the families. No research document states these values, so they rest on `UNV-34` of the plan.

The negative tests of the census build five domains of wrong groups with the public `build`: `2*2` labelled `4*`, `2*3` labelled `6*`, `6*` labelled `2*3`, the double of the lune of `*(2n)(2n)` labelled `nx`, and the lune of `*nn` labelled `nx`, the last two at $n = 3$ and $n = 4$. Each passes `verify()` and enumerates to the order of its label, and only the census, or the exactly-once test, tells it from the group of its label. On a prototype, the label `GlideReflection` on the pairings of `x` also kept the census without the plane rule, because it changes a label and not a group. The plane rule of DOM-61 catches it.

DOM-57. The tests "Wallpaper certificate" and "Equal counts" MUST cover each of the ten domains of Section 6.13 and MUST stay while `UNV-11` of the plan is open, and the certificate MUST assert the point group and the glide reflections whose axes are not mirror lines, because no source states that the pairings of these domains generate the group of the signature, and the counts alone cannot show it. On a prototype, the counts caught a dropped cycle relator and were equal for `o` and `xx` at every index that it computed. The test "Wallpaper presentations", which asserts the exact relator texts, catches a dropped square of a mirror, and only the certificate tells `o` from `xx`. Neither test closes `UNV-11`, which a source or a Lean proof closes.

DOM-64. The tests "Spherical certificate", "Spherical certificate at the limit", "Census defects", "Spherical side orders", and "Spherical counts" MUST cover every spherical group domain of their sets and MUST stay while `UNV-34` of the plan is open. The certificate MUST assert the order by the breadth-first closure, the exactly-once tiling of the sample, and the census of each group, and the census MUST NOT be left out, because no source states that the pairings of these domains generate the group of the signature, and the order and the presentation do not separate these groups. `432` and `*332` are isomorphic as abstract groups, and so are `2*n` and `(2n)22`, and `*nn` and `n22`. `n*`, `nx`, `n22`, and `*nn` all have the order $2n$, and `x` and `*` have equal tiles and equal colourings. Neither `verify()` nor the enumeration of the side presentation tells `n*` from `2*(n/2)`, `nx` from `*nn` and from `(2n)(2n)`, or `2*n` from `(2n)*`, so the census and the exactly-once test of the certificate are the tests against these mix-ups.

A dropped mirror relator leaves the enumeration at `group_order()` and the colouring counts equal to those of the Conway presentation, so the presentation tests drop cycle relators only, and the assertion of the full relator list catches a mirror relator that is left out. A relator that the presentation leaves out is never evaluated, so the evaluation of the relators cannot catch it.

DOM-22. The exhaustive certificates, one over the orders up to 12 and one at the order limit, MUST cover every lune of their sets and MUST stay while `UNV-29` is open, because no source makes the lune a fundamental domain (Section 6.7), and these tests are the only evidence that its two reflections generate a group of order $2n$ whose copies of the lune cover the sphere.

DOM-31. The exhaustive certificates of DOM-22 MUST cover every spherical double of their sets, `332`, `432`, `532`, `n22`, and `nn`, and MUST stay while `UNV-22` or `UNV-30` is open, because no statement in orbifold notation gives the signature of the group that the pairings of a double generate, no source makes the double of a lune a fundamental domain, and the enumeration of the group with `group_order()` elements and the dense sample are the only evidence for both on the sphere.

DOM-48. The tests of the polygons MUST include a test of the meaning of each length parameter, a comparison of the walk with SymmHub's welded layout, a comparison of the closing side with its sum of gaps, and the identities that share no formula with Section 6.10: the angles $\pi/n_j$ and the area $-2\pi\chi$ measured from the reference vertices, and the tangency of the inscribed polygon. The reason is that the reference of Section 12.2 follows the steps of Section 6.10 in another arithmetic, so it shares their formulas, which rest on `UNV-6`, `UNV-7`, and `UNV-27`, and a wrong formula would pass a comparison with the reference alone.

DOM-36. The test "Relators evaluate" MUST cover every domain of its sets, hyperbolic, Euclidean, and spherical, reflection, rotation, and wallpaper, and the test "Long cycle word" MUST exist, and both MUST stay while `UNV-32` is open, because no source states the cycle relators as words in the convention of DOM-32, and a reversed relator or the opposite convention presents a group of the same order, which no enumeration detects.

The two tests establish different things. "Relators evaluate" establishes that every relator of every domain of its sets evaluates to the identity under the convention of DOM-32, within the tolerances of checks 2 and 4. It does not fix the convention, because on a prototype the reversed words and the opposite convention gave the same residuals on every built double (Section 5.4). "Long cycle word" establishes, for the one cycle of three sides of one hexagon, that the base in walk order under the convention of DOM-32 evaluates to the identity and that its reversal and the opposite convention do not. It also runs on the four-letter cycle 1 of `22*`, `s2 s1 s5 s0`, on the domain that `build_with` returns, where it asserts the reading of DOM-32 and only prints the other two readings. A measurement on `**`, `xx`, `*x`, and `22*` at `aspect` 1, 0.125, and 8 found that all three readings give the residual 0 on every cycle of three or more letters, and no committed test repeats it except on cycle 1 of the built `22*` at `aspect` 1. Neither test closes `UNV-32`, which a source or a Lean proof closes.

The hexagon of "Long cycle word" is cut from the kite of `732`, whose vertices are $W_0$ to $W_3$, with $P_0$ the pairing of side 0, about $W_0$, and $Q$ the pairing of side 3, about $W_2$. The point $X$ lies on side 3 of the kite, at 0.37 of the way from $W_2$ to $W_3$, normalized. The triangle $(W_3, W_0, X)$ is cut off along $W_0 X$ and pasted across side 1 by $P_0$. The hexagon has these values:

- Vertices: $W_0$, $P_0 X$, $W_1$, $Q X$, $W_2$, and $X$.
- Pairings, by side: $P_0$, $P_0^{-1}$, $Q P_0^{-1}$, $P_0 Q^{-1}$, $Q^{-1}$, and $Q$, with the pairs (0, 1), (2, 3), and (4, 5), each of the kind `Rotation`.
- Corners, as order and copies: (7, 2), (1, 1), (2, 2), (1, 1), (3, 2), and (1, 0).
- Vertex cycles: vertices 0, 2, and 4 are cycles of one vertex, and vertices 1, 3, and 5 form one cycle of order 1, whose walk applies sides 2, 4, and 0.

Vertex 5 of the hexagon has 0 copies, because check 4 requires the copies of a cycle to sum to 2 (DOM-55), and a cycle of three vertices cannot meet that with copies of at least 1. The copies 1, 1, and 0 of vertices 1, 3, and 5 sum to 2 but do not describe their angles, so the test asserts check 4 only. Item 33 of Section 15.3 puts the point to the owner. A regular hyperbolic quadrilateral with its opposite sides paired, whose one vertex cycle visits four sides, was measured as a candidate domain and does not serve, because neither of the other two readings fails on its cycle. No test uses it.

"Relators evaluate" evaluates a hyperbolic corner or cycle relator in the corner frame of check 4, because without the frame the residual at the largest orders fails the tolerance of check 4 by far.

The fold of the certificate tests is a private copy in the tests of this crate. For every finite point other than the zero vector it applies the same rule as the fold of `escher-render` (Section 5.3 of `docs/design/render.md`). It differs only on degenerate input: it has no stop for a point that is not finite or for the zero vector, and it treats a NaN side value as inside the side, where the fold of `escher-render` returns an unconverged result for each of these. Side $i$ is violated at a point $x$ when $s_i > \tau \, \sigma_i \, \lambda(x)$, where $s_i$ is `spherical::side` of the covector of the side at $x$, $\tau = 4 \times 10^{-15}$ is a copy of the constant of `escher-render`, $\sigma_i = \sum_j \lvert c_{ij} \rvert$ for the covector $c_i$, and $\lambda(x) = \max_j \lvert x_j \rvert$. The fold applies the pairing of the first violated side, and stops when no side is violated. A fold that applies the pairing of the first side with a positive side value does not stop for a point on a wall, because the reflection can map such a point to itself bit for bit. The images of the centre lie far from every wall, so the tolerance does not change their depth.

The floors of the certificates are set from the smallest distance measured between two images of the centre, with a margin of at least 8 as TEST-13 of the coding standard requires. The claim that a fold returns in its breadth-first depth and never reaches the cap holds for the images of the centre of a reflection domain, which the tests assert. For a double, the fold can take more steps than the breadth-first depth, so the tests assert only that it takes at least as many steps and converges within the cap. It does not hold for every point: near a vertex of very high order, a fold can fail to converge, which belongs to the unverified statements on termination, `UNV-1` and, for a double, `UNV-2` (Section 14).

The inverse $J M^t J$ and the round trip of the disk map are properties of `escher-geometry`, and the geometry design tests them.

The expected values for $\{7, 3\}$ come from the split of the polygon into $2p$ right triangles (Section 10.1 of the geometry reference), which puts the centre of the polygon at vertex 0 of `*732`, a vertex of the polygon at vertex 1, and the midpoint of an edge at vertex 2. The tests evaluate equations 29 and 24 of [HT] (Section 10.4 of the geometry reference) in the reference arithmetic, with the curvature radius $\ell = 1$. No value that Table 2 of [HT] prints serves as an expected value, because that table and equation 29 disagree on the edge length of $\{7, 3\}$ (item 23 of Section 15.1 of the geometry reference).

DOM-14. A test MUST NOT take an expected value from Table 2 of [HT], because the research records that the table and equation 29 disagree for $\{7, 3\}$, and a test built on the wrong one passes a wrong domain or fails a correct one.

The property tests commit their regression files (TEST-9 of the coding standard). Every property test takes its configuration from the one helper `proptest_config()` in `common`, which returns the default configuration of `proptest`.

The tests compare a value that passes through a transcendental function in three ways:

- With a stored value, bit for bit, where the value is exact: the angles 0 and $\pi$ of a pentagon whose corner frames are exact, on the assumption that `atan2` then gives 0 or the `f64` value of $\pi$ in the platform's library, which the test checks and no source cited here establishes, and the defaults of the polygons, which the crate rounds to three decimals, which holds only while each unrounded default lies further from a boundary of the rounding than its error.
- With another result of the same run, bit for bit, where two builds or two rules take the same function of the same argument (DOM-30).
- Within a tolerance, as TEST-13 of the coding standard requires, in every other test. Two builds of one domain are compared within twice the vertex bound of Section 9.4 or 9.5, or twice the image-vertex bound of Section 9.6.

DOM-16. Withdrawn on 2026-11-19.

### 12.2. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture, with the precision and tolerances that this section states:

- Structure. A module `reference` in the test binary holds points as arrays of three high-precision numbers and writes each formula componentwise from the research documents. It calls no function of `escher-domain`, `escher-geometry`, or `glam`.
- Independence. The reference rebuilds every domain from the formulas of the research documents by another path than the crate: it computes the standard position from cosines and square roots without the inverse functions that the crate calls, centres by the polar form of the centre, finds walls by the Gram-Schmidt process or by the perpendicular form of a line in place of cross products, reflects the vertices and normals of a double componentwise, writes each pairing of the ten domains and of the spherical group domains as a matrix from its definition, and measures the distance between two walls by a search over points, with the closest point of the second wall in closed form (Section 14), without the relation of step 4 of DOM-45. For a polygon it follows the steps of Section 6.10 from the same `f64` lengths, so it shares their formulas but none of their code, which is why DOM-48 adds identities that share no formula. A transcription error in the crate is therefore not repeated in the reference, and a test that compares the two catches it. Values that the construction stores exactly, such as those of the lune of order 1 and the whole sphere, need no reference.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 10.2 of the geometry design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there, because `astro-float-num` has no public conversion back to `f64`. A message that prints a measured error formats the number as decimal text.
- Self-check. A test in `reference` checks the reference library itself: $\pi$ against a literal with 80 digits after the decimal point, $\cos(\pi/3) = 1/2$, $\cosh(\operatorname{arcosh} 2) = 2$, and $\sqrt{2}^2 = 2$, each to $10^{-70}$. It guards against a wrong precision or rounding argument and does not close `UNV-25`.

DOM-15. The reference module MUST share no code with `escher-domain` or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture).

## 13. Dependencies

DOM-17. The crate MUST declare only the dependencies of this section, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml`, because each further dependency adds build time, licence obligations, and supply-chain risk, and a version declared twice can diverge (DEP-2 of the coding standard).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-signature` | Normal, with a version on the path dependency (TOOL-25 of `docs/design/tooling.md`) | None | The input type `Signature` and the `Geometry` that `geometry()` returns |
| `escher-geometry` | Normal, with a version on the path dependency (TOOL-25) | None | The linear model (Section 3.1 of the architecture) |
| `escher-group` | Normal, public, with a version on the path dependency (TOOL-25) | None | The type `Presentation` that `side_presentation()` returns, and `enumerate` in the tests of Section 12.1 |
| `glam` | Normal, public | The workspace entry that the geometry design states, which provides `DVec3` and `DMat3` | The types of the public surface (`DEC-4`) |
| `thiserror` | Normal | Default | Derives the `Display` of the private detail of the error (Section 10.1) |
| `proptest` | Dev | The workspace entry | The property tests |
| `astro-float-num` | Dev | `default-features = false`, with `std` | The reference arithmetic (Section 12.2) |

`escher-group` depends on `escher-signature` only, so the graph has no cycle (Section 3.3 of the architecture). `Presentation` appears in a public signature of this crate, so DEP-3 of the coding standard makes `escher-group` a public dependency: a breaking change of `escher-group` is a breaking change of this crate. Both crates set `publish = false`, so the cost stays inside the workspace.

`glam` types appear in the public signatures of this crate, so DEP-3 of the coding standard makes `glam` a public dependency, with the obligations that Section 3.5 of the architecture states: its version is part of the version of this crate, the crate stays below 1.0 while `glam` does, and a breaking upgrade of `glam` is a breaking change of the crate.

The reference crate departs from `DEC-3`, which names `astro-float`, for the reason and with the resolution facts that Section 11 of the geometry design records: `astro-float` brings `syn` 1 beside the locked `syn` 3 and fails the duplicate check of `cargo deny`, while `astro-float-num` passes. Section 15.3 asks the owner to confirm the change.

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

## 14. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, or a candidate statement of a reference without a label there, the sections that rely on it, and what the code does about it. A source or a Lean proof closes each one unless the row says otherwise:

| Label | What the code relies on | Relied on in | What the code does |
| --- | --- | --- | --- |
| `UNV-1` | The termination of the fold, for the images of the centre in the certificate tests and on the spherical group domains | 12.1 | Asserts termination for those images only. On a prototype, the fold of `*(1000)22` failed to converge within the default cap for some points very near its vertex of order 1000. `docs/design/render.md` owns the fold of the renderer |
| `UNV-2` | The termination of the fold with the rotation pairings of a spherical double, on the ten domains, and on the spherical group domains | 12.1 | Asserts it for the points of the certificate tests only, with the steps at least the breadth-first depth. The wallpaper certificate finds images by enumeration and not by the fold. Section 5.3 of `docs/design/render.md` owns the fold (RND-45) |
| `UNV-6` | The gap function of step 5 of Section 6.10: its $\operatorname{arsinh}$ branch, the closing gap of each sheet, and the correspondence of its $\operatorname{arcosh}$ branch with the formulas of Section 9.3 of the domain reference | 6.10, 6.11 | The guard of DOM-44 verifies every returned polygon, so a wrong gap gives `Numerical` or a parameter with another meaning, and not a wrong group. The tests of DOM-48 test the gaps numerically. [DEATON1993] does not close it. Formula Glossary 2.2 to 2.4 of [BUSER] or a Lean proof would |
| `UNV-7` | That the slices and corner pairs of Section 6.10 are the cuts and edge lengths of [TH] p. 316, and SymmHub's order of the cuts | 6.6, 6.10, 6.11 | The names and the order of the parameters follow SymmHub's cuts (DOM-45). A wrong reading changes what a parameter means and not the group. [DEATON1993] does not close the order of the cuts (Section 8.15 of the domain reference). A project convention for the order, and a source or a Lean proof for the meaning, would |
| `UNV-10` | Whether the one parameter of `*2222`, and the two of `2222`, include the scale | 6.5, 6.6 | Fixes the scale by the area 1, asserts no count of parameters, and leaves `deformation_dimension()` as the signature design states |
| `UNV-11` | That the group that the pairings of each of the ten domains generate has the signature of the domain | 6.13, 6.14 | The guard of DOM-54 verifies the hypotheses of Theorem 4.14 of [EPSTEIN-PETRONIO1994] on every build, and the tests of DOM-57 check the group numerically |
| `UNV-19` | The steps of the coset enumeration of `escher-group`, in the presentation tests only | 12.1 | Section 13 of the group design states how `escher-group` relies on it. A wrong relator would still fail the evaluation of the relators |
| `UNV-20` | The text `1` of the trivial signature, which `escher-signature` parses and this crate builds as the whole sphere | 4, 6.16 | The tests assert the domain and the presentation of the whole sphere, and not the reading of the text |
| `UNV-22` | That the group that the pairings of a double generate has the rotation signature (gap G10 of the domain reference) | 6.8, 6.9 | Check 6 compares the area with that of the rotation signature, and the exhaustive certificates enumerate exactly `group_order()` elements on the sphere (DOM-31) |
| `UNV-25` | The correct rounding of the reference crate, in tests only | 12.2 | Uses 256 bits against tolerances of $10^{-13}$ at the tightest, and the self-check of Section 12.2. One comparison with MPFR closes it (Section 15.3) |
| `UNV-26` | The wall normals of `hyperbolic::wall_through`, the Euclidean reflection as a matrix, and the angle of a lune and of its double from its outward normals. Nothing from its part on the relation between normals and the corner angle elsewhere | 5.3, 6.1, 6.2, 6.7, 7 | Compares every normal with the reference, and checks 1 to 4 test each wall and pairing against the vertices. The corner angles of every other domain come from distances (DOM-6), and check 4 and the exhaustive certificate test the angle of a lune |
| `UNV-27` | That every sheet exists and closes for the lengths of the range, that the gaps at a slice end lie on one geodesic, the coordinates of the family, the range 0.5 to 3.5, and the default shape | 6.6, 6.10, 6.11 | The guard refuses a polygon that does not close or is not convex. The tests measure the collinearity and the tangency of the inscribed polygon in the reference arithmetic. [DEATON1993] does not close it |
| `UNV-28` | The vertices of the Euclidean triangle of steps 1 and 2 of Section 6.5, the candidate of item 26 of Section 15.1 of the geometry reference | 6.5 | Checks 3 and 5 test each vertex and angle on every build, and a test compares the walls with SymmHub's three lines. The hyperbolic construction does not rely on it (Section 6.3) |
| `UNV-29` | That the lune is a strict fundamental domain of the group of its two reflections, of order $2n$, and that a lune is a cell of the polygon theorems, which the domains of `nx`, `x`, and `*` also rely on | 6.7, 6.17 | The exhaustive certificates enumerate the group, check that the images of the centre are distinct and fold back, and fold a dense sample of the sphere into the lune (DOM-22), and the certificates of DOM-64 cover the lune groups |
| `UNV-30` | That the double of a lune is a fundamental domain of the cyclic group of order $n$, and that a lune is a cell of the polygon theorems | 6.8, 6.9, 6.17 | The exhaustive certificates as for `UNV-29` (DOM-31) |
| `UNV-31` | That the double of a convex reflection polygon across one wall is convex with the vertex cycles of the table of Section 6.8, and that Definition 2.8 of [EPSTEIN-PETRONIO1994] admits flat vertices and a mirror side split at a flat vertex | 6.8, 6.9, 6.16 | The convexity term of check 3, the walk of check 4, and check 5 run on every build, and a defect test shows that the convexity term fails a vertex outside a wall that is not adjacent to it |
| `UNV-32` | The cycle relators as words in the convention of DOM-32, with their bases in walk order, including the chains of the ten domains, whose walk applies a mirror twice | 5.4, 8.2 | The tests of DOM-36 evaluate the words numerically, the presentation tests compare the orders on the sphere and the colouring counts with the Conway presentation, and the relator texts are asserted. On the built domains a reversed base and the opposite convention give the same residuals, so only the hexagon built through private access fixes the convention |
| `UNV-33` | The domains of `**`, `22*`, and `22x` with their pairings, the pairings of `*x`, `2*22`, `4*2`, and `3*3` as formulas, the shear of `o` and `2222` with the range of `skew`, and the cycles of the table of DOM-50 | 6.6, 6.13, 6.14 | The guard of DOM-54 verifies every built domain, the tests of DOM-57 check the group, and a test compares the sheared domain with the shear of the domain at `skew` 0 |
| `UNV-34` | The domains of `3*2`, `2*n`, `n*`, `nx`, `x`, `*`, and `1` with their pairings and vertex cycles, the pairings $H R_i$, the group that the pairings generate, the walk of DOM-55 on a lune with reversing pairings, and the values of the census | 6.16, 6.17 | The guard of DOM-60 verifies every built domain other than that of `*`, and the tests of DOM-64 check the order, the census, and the exactly-once tiling |
| None | The line through two vertices as a normalized cross product, and the area of a polygon from its vertices (items 24 and 25 of Section 15.1 of the geometry reference) | 6.5, 7 | Check 3 tests each line, and the tests compare each covector and the area with the reference, whose area uses the same formula, so the area test checks the arithmetic and not the formula |
| None | That `spherical::translation` maps the origin to the sphere point of its target, in step 4 of the spherical triangle | 6.7 | Section 13.1 of the geometry design records it. The tests compare every vertex and the centre with the reference, which centres without it |
| None | That the angles of a Euclidean polygon with $m$ vertices sum to $(m - 2)\pi$, which check 6 reads | 8.2 | Reports the residual as a consistency check only |
| None | The method of Section 6.10, which reads a cut sheet as $(y, x, \bar s_i)$ and walks the boundary with each side the sum of its gaps, a reader's construction supported by numeric evidence only | 6.10 | The guard verifies every returned polygon, step 4 of Section 6.10 turns a gap on the wrong side into `Numerical`, and the tests compare the walk with the welded layout in the reference arithmetic |
| None | The placement of the inscribed polygon from Theorem 7.11.3 (ii) of [BEA], and the distance $\operatorname{arcosh} \lvert \langle u, v \rangle \rvert$ between two walls (items 29 and 30 of Section 15.1 of the geometry reference) | 6.6 | Only the defaults of DOM-45 rest on it. The tests check the angles and the tangency of the inscribed polygon in the reference, and the default of a slice against the distance between its walls found without the relation |
| None | The closed form of the smallest hyperbolic cosine of the distance from a point of one wall to the other wall, in the reference of Section 12.2, in tests only | 12.2 | Its distances agree with the relation of item 30 and with the slice lengths, which is numeric evidence only |
| None | That the angle in the corner frame of DOM-46 is the hyperbolic angle at the vertex | 7 | Check 4 tests the corner relators from the pairings alone, and the tests compare each angle with the reference |
| None | That the double-double functions of DOM-42 meet their bounds for every input of the range, and that the double-double operations follow [QD], which no author of this design read | 6.10 | The guard verifies every polygon, so a function that misses its bound gives `Numerical` or a polygon that passes every check. The code relies on the measured bounds of its tests and not on [QD]. A reading of [QD] would close the second part |
| None | That the walls of two non-adjacent sides of a compact hyperbolic polygon with corner angles of at most $\pi/2$ do not meet, a result attributed to Vinberg, and that an orientation-preserving hyperbolic isometry is a translation exactly when its trace is above 3 | 6.8, 8.2 | Check 2 requires the kind of every side to agree with the translation test on every build, so a pair of walls that met would fail check 2 and the guard would refuse the double |
| None | The table of directions of the state walk of DOM-55 and its limit of $2m$ steps, and the cycle rule of check 5 with a corner between two mirrors counted twice | 8.2 | Check 4 tests every cycle that the walk finds, and on the doubles the walk equals the walk of Definition 11 of [SOLOMYAK2020] |
| None | The identity that relates the cycles, the pairs, and the mirrors of a wallpaper domain to its Euler characteristic | 12.1 | A test asserts it on the ten domains as a consistency check only |
| None | The expected values of the wallpaper certificate: the orders of the point groups, and which groups have a glide reflection whose axis is not a mirror line | 12.1 | Control domains of known groups classify correctly. A source that tabulates them would close it |
| None | The half-turn about a point as the product of the reflections in two perpendicular lines through it, and the glide reflection as a translation along its axis composed with the reflection in the axis (items 31 and 32 of Section 15.1 of the geometry reference) | 6.13 | Checks 1 and 2 test each pairing, and the tests assert each entry and compare with the reference, which builds each pairing from its definition |
| None | That the words $s_j s_{j+1}$ of the corners generate the subgroup of index 2 of the orientation-preserving elements | 12.1 | The test asserts the index only |
| None | That every orientation-reversing isometry of the sphere is a rotary reflection (item 33 of Section 15.1 of the geometry reference) | 3.2 | The kind is a label that the renderer never reads, and check 2 ties it to the orientation, the determinant, and the plane (DOM-61) |

`UNV-4` concerns the precision of the disk map near the limit circle, and the crate maps to the disk only in a test of $\{7, 3\}$, at a point near the origin.

These statements of this design rest on measurements of prototypes outside the repository that no test of the crate repeats. Each supports only the choice that it names:

- A domain left with vertex 2 at the origin has larger normal components and fails checks that the final position passes (Section 6.3).
- Repeated multiplication evaluates a relator more accurately than repeated squaring (Section 8.2).
- Renormalizing the vertices after centring makes them less accurate (DOM-10).
- The double across side 2 fails where the double across side 1 passes, and plain products as pairings leave a fold unconverged (DOM-23 and DOM-25).
- The vertices of a double lie within the tolerance of the fold over every hyperbolic triple, with a margin below 8 outside the named sets (item 27 of Section 15.3).
- The plain `f64` walk is too inaccurate, the closing gap agrees with SymmHub's layout, and the cut sheet in the other order does not close (Sections 6.10 and 6.12).
- The limit of 500 of `2*n` is not needed for accuracy up to `2*(2000)` (DOM-63).
- The reversed bases and the opposite convention give the same residuals as the convention of DOM-32 on every built double (Section 5.4), and the spherical kites fail with the closure `PerVertex` (DOM-56 and Section 6.18).
- Without the plane rule of DOM-61, `x` with its pairings labelled `GlideReflection` passes every check and every enumeration, and keeps its census (DOM-61, Section 6.18, and Section 12.1).
- The shares of random settings that the exact polygon rounded once fails for seven and eight corners, and the margins of the default sweep (DOM-44 and Section 9.7).
- The comparison with SymmHub's catalogue (Sections 6.13 and 6.16).

## 15. Assumptions, Choices, and Points for the Owner

### 15.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- The triangle placed by the two sides $L_a$ and $L_b$ with the included angle $\alpha_2$ is the triangle with the angles $\alpha_0$, $\alpha_1$, and $\alpha_2$. The research states uniqueness given the three angles (Section 10.1 of the geometry reference), but no statement on two sides and the included angle. A test of the third side in the reference and checks 4 and 5 check it. The same reading applies to the spherical triangle of Section 6.7, which checks 4 and 5 and the reference tests check.
- The angle at a vertex is the arccosine of $(\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)$, the first law of cosines of Section 10.2 of the geometry reference solved for the angle, which no source prints in this form. The reference tests compare the angles with $\pi/n_j$.
- The normalized sum of the vertices lies strictly inside the triangle, and the centre of the generating domain strictly inside its double. Check 2 fails if either does not.
- A power of a matrix is the identity exactly when the same power of its conjugate is, which lets check 4 evaluate a relator in the corner frame (Section 8.2). A reference test checks it on the named triples.
- Side 0, side 1, and side 2 are the Latin reflections $P$, $Q$, and $R$ of step 3 of Section 9.1 of the notation reference, and its fourth reflection $S$ is $P$ again for a triangle and side 3 for the rectangle. The relators that the checks use come from Section 4.4 of the domain reference and do not depend on this reading.
- `hyperbolic::translation(c)` maps `hyperbolic::ORIGIN` to the point with the spatial coordinates of $c$, as Section 6.4 of the geometry design states, and this is $c$ itself only when $c$ lies on the hyperboloid. Section 13.1 of the geometry design records this property as an assumption and names the test that checks it.
- The angle of a kite at $W_0$ or $W_2$ is the sum of the angles of its two triangles there, and its area is the sum of their areas (DOM-29). The reference tests compare the angles and the area, and check 5 compares each angle with the copies times $\pi$ divided by the order.
- The image of a wall under $R_1$ has the normal $R_1 u$ for the normal $u$, which is outward for the image side, and in the Euclidean plane the covector $R_1^{T} c$ for the covector $c$ (step 3 of Section 6.8). No source prints these forms. Checks 2 and 3 test each image wall against its vertices and the centre, and the reference tests compare it with the reference, which reflects the normals componentwise.
- For the unit normals $u_0$ and $u_1$ of the double of a lune, $\operatorname{atan2}(\lvert u_0 \times u_1 \rvert, -u_0 \cdot u_1)$ is the same angle as $\arccos(-u_0 \cdot u_1)$ (DOM-20). No source read states this form. Checks 4 and 5, the reference tests, and the exhaustive certificates check it.
- The wall term of check 2, $P_i^{T} c_p + c_i$, is 0 when $P_i$ carries the wall of side $i$ onto the wall of its partner $p$ with the domain on opposite sides (Section 8.2). No source read states this form. The defect of the half-turn on `44` fails check 2 through this term alone, and every double of Section 9.6 passes it.
- The compensated product of DOM-25 is correctly rounded for every entry of every pairing of a double other than a double of a polygon. The tests assert it for the doubles of Section 9.6 only. For the doubles of polygons it does not hold for a few entries (Section 6.8), and the tests assert only a bound on their error. Compensated arithmetic does not guarantee correct rounding in general, and no test covers the other doubles, such as the kites of the cone triples outside the set of triples, or `2222` at other values of `aspect`. A proof of correct rounding for the entries of products of two reflections, or a test over every supported double, would close it.
- A canonical rotation signature lists its cone orders in descending order, so the orders of a rotation signature are the corner orders of its starred signature in the same order (Section 4). The table tests check it through the spellings `237` and `22n`. For four or more orders, the starred signature of `p1...pn` is the star followed by its cone orders in descending order, and a descending sequence of corners is its own canonical form under step 4 of Section 7.2 of the signature design, so `3322` gives `*3322` and `5432` gives `*5432`. The tests check that the starred signature of each double is canonical.
- In the corner frame of a vertex, the angle between the first two coordinates of its two neighbours is the hyperbolic angle at the vertex (DOM-46). This reads the conformality of the disk at its origin and its diameters as geodesics ([CFKP], p. 80) through the disk map, which keeps the direction of the first two coordinates. No source states it for the hyperboloid. The reference tests compare each angle, and check 4 tests the same corners from the pairings.
- The area of a double with a flat vertex is $(m - 2)\pi$ minus its angle sum with that vertex counted with the angle $\pi$. Theorem 7.15.1 of [BEA] states the area of a polygon, and a vertex of the angle $\pi$ is not a vertex of a polygon in its sense. The reference tests compare the area with the area of the signature.
- A closed walk whose vertices all lie inside or on every wall of its sides, with angles in $(0, \pi]$ and an angle sum below $(m - 2)\pi$, bounds a simple convex polygon. The guard reads checks 3 and 5 this way (Section 6.11). The reference tests compare every vertex with the reference, whose polygon is convex by construction.
- The closed form of the translation in step 8 of Section 6.10 equals the product $R(\varphi) \, B(\rho) \, R(-\varphi)$ of `hyperbolic::translation`, and its inverse is that matrix with the entries that mix the time coordinate with a space coordinate negated. This is matrix algebra, which no source read prints in this form. The guard checks every placed polygon, and the tests check that the centre lies within $10^{-9}$ of the origin and vertex 0 on the positive first axis.
- The shear $(x, y) \mapsto (x + k a y, y)$ keeps the horizontal sides of `2222` horizontal, so its sheared half-turns are half-turns about the sheared centres, and its flat vertices stay flat. No source read states it for this domain. Checks 2, 4, and 5 test the pairings, the cycles, and the angles on every build, and the tests measure every flat vertex at exactly $\pi$ (Section 6.13).
- The whole sphere with no side satisfies the hypotheses of the polygon theorems vacuously, so it is a fundamental domain of the trivial group (DOM-62). No source states it for a domain with no side. The certificate checks that its closure has one element and that every sample point lies in it.
- The kite of `n22` with the mirrors on sides 2 and 3 is the domain of `n*`, and the kite of `(2n)22` with the mirrors on sides 0 and 1 the domain of `2*n`, and not the other way round. The census of DOM-64 tells them apart, and its negative tests build the swapped domains.

### 15.2. Choices of This Design

No other document dictates these choices. Sections 6.12, 6.15, and 6.18 record the alternatives of the constructions with their reasons, each rule states its own reason, and Section 15.3 lists the choices that the owner can still change. These are the choices that a reader could otherwise undo by mistake:

| Choice | Reason | Section |
| --- | --- | --- |
| The domain stores the geometry, the corner orders, the area, the vertices, the sides, and the centre, although all of them follow from the signature, and stores no signature | They are the result of the construction, which the renderer would otherwise repeat, and the type is immutable, so they cannot diverge (TYPE-3 of the coding standard). `build` only borrows the signature, and the checks need only these values | 3.2, 6.1 |
| The measurements `side_lengths`, `corner_angles`, and `area` are plain `f64` methods of the domain | `escher-cli` then needs neither `glam` nor `escher-geometry` | 3.1, 7 |
| Check 4 in double-double arithmetic only for the hyperbolic domains with five or more vertices | Every other domain keeps its values and residuals bit for bit (DOM-30), and only those domains need it (DOM-49) | 8.2 |
| One state walk for check 4 and the side presentation of every domain that is not all mirrors | The relators and the check then describe the same cycles, and the doubles keep their cycles and residuals (DOM-55) | 8.2, 5.4 |
| The tolerance of the fold of the certificate tests is a copy of the constant of `escher-render` | The private fold must apply the same rule as the renderer, so a change of one constant must change the other | 12.1 |
| Tests accept either outcome for a double whose smallest margin is below 2 | Such a double passes or fails with the last bits of its computation | 8.4, 9.7 |
| The large orders of the property test of random polygons come from $\{900, 950, 1000\}$ | `escher-signature` can reject a signature of seven or eight orders from 900 to 1000, which then never reaches `build_with` | 12.1 |
| The reference part of the test of the default shape runs on the named polygons, every 256th small polygon, and `*422322` and `*432232` | The reference arithmetic is slow, and those two signatures have defaults near a boundary of the rounding | 6.6 |
| Two names keep a meaning that differs from Section 2.1: the module `src/polygon.rs` and the helper `corner_pairs` | Renaming them changes no behaviour, and Section 3.3 states what each holds | 3.3 |

### 15.3. Points for the Owner

None of these blocks the crate. Each item states the default in use until the owner decides, and keeps its number, because other documents cite items by number. A number that no longer names an open point is missing or says where the matter lives.

Item 1. Reference crate. `DEC-3` names `astro-float`, and the tests use `astro-float-num`, because `astro-float` fails the duplicate check of `cargo deny` (Section 13). The alternative is a `skip` entry for `syn` 1 in `deny.toml`, as Section 13.3 of the geometry design also asks.

Item 2. Largest corner order. The design supports orders up to 1000 (DOM-3), with the tolerance $2 \times 10^{-6}$ of check 4 (Section 8.4). A higher limit needs a better conditioned construction of the far walls, which the research does not source, and new measurements of every margin.

Item 3. Comparison with MPFR. The design takes `UNV-25` as open until the owner decides whether the comparison that the geometry design records (its Section 12) closes it.

Item 4. Check 6. The design keeps it, with the tolerance $10^{-9}$ for every domain, although it detects nothing that check 5 does not apart from a disagreement with `escher-signature` (Section 8.2). The alternatives are to drop it, to measure the area by an independent formula from the vertices, which needs a source that the research does not hold, or for a polygon the tolerance $m \cdot 10^{-9}$, which would refuse fewer settings.

Item 6. Vertices of the Euclidean triangle. The design builds the triangle from the candidate of `UNV-28` and certifies each build (Section 6.5). The alternative is to build only `*2222` until a source closes the candidate.

Item 7. Units of the Euclidean domains (`DEC-17`). The triangle has side 0 of length 1 and the rectangle the area 1, so `aspect` and its reciprocal give the same rectangle turned a quarter turn, except for the double of `2222` (Section 6.5). The alternative is the height 1 and the width `aspect`.

Item 8. Range of `aspect`. The design takes 0.125 to 8 with the default 1 (Section 6.6), and no source gives another range.

Item 9. The area of a Euclidean domain. `area()` uses a formula that no source states (Section 7). The alternative is no area for a Euclidean domain, which removes the line `Domain area` of `escher info`.

Item 14. The lunes and their doubles on unverified statements. The design builds the lune of `*nn` on `UNV-29` and the double of a lune on `UNV-30`, each certified by the tests of DOM-22 and DOM-31. The alternative is to leave `*nn` and `nn` unsupported until a source closes the statements.

Item 19. The doubled wall (`DEC-11`). The design doubles across side 1, the wall through $V_0$ and $V_1$ (DOM-23). The alternative is side 2, which failed check 4 for `*(1000)32` on a prototype.

Item 20. The kinds `Rotation` and `Translation`. The design gives a translation its own kind (Section 3.2). The alternative is one kind for every orientation-preserving pairing, which would call a translation a rotation.

Item 21. The accessor `reverses_orientation()`. The design adds it, so that `escher-cli` decides in one place whether a parity colouring exists (Section 3.2). The alternative is that each caller scans `sides()`.

Item 23. Compensated pairings of a double (DOM-25). The design computes each pairing as a compensated product at construction, for the convergence of the fold. The alternatives are plain products with a narrower range of accuracy near the order 1000, or a fold that applies the reflections of the generating domain and needs no rotation pairing.

Item 24. `aspect` of `2222`. The design makes it the aspect of the generating rectangle, so the double has the area 2 (Section 6.6). The alternative is a double with the area 1 and `aspect` as the ratio of its own sides.

Item 27. The margin of the vertices of a double against the tolerance of the fold. On a prototype, no vertex of a double exceeded the tolerance of the fold of `docs/design/render.md`, but over every hyperbolic triple its margin fell below 8, and the design claims the margin for the named sets only. The alternative is to compute the image vertices and normals as correctly rounded products, whose effect no test measures.

Item 29. One generator for each side (`DEC-36`). The side presentation keeps both generators of a pair, so a colouring finds the generator of a fold step by its side index (Section 5.4). The alternative is one generator for each pair, which halves the columns of the coset tables of a double.

Item 30. `side_presentation()` returns a `Result` (DOM-35). The alternative is an infallible function, which would have to invent a presentation or panic for a corrupted domain.

Item 32. The generator names from `s0` (`DEC-34`). The alternative is names from 1, which would differ from the side index that the renderer records and `escher info` prints.

Item 33. The copies of a long vertex cycle. The doubles keep the condition that the copies of a cycle sum to 2 (DOM-55), so a cycle of three or more vertices passes it only when a vertex has 0 copies, which does not describe its angle, as in the hexagon of Section 12.1. No built double has such a cycle. The wallpaper and spherical group domains use the cycle rule of DOM-56 instead. The alternative is to move the doubles to the cycle rule as well.

Item 34. The default shape of a polygon (`DEC-13` as amended). The design takes the polygon with an inscribed circle, rounded to three decimals (DOM-45), which is the regular polygon when the orders are equal. The alternative is the regular shape where the orders are equal and the length 2 otherwise, for which the research has no formula.

Item 35. The parameter names (`DEC-47`). The design takes `cornerpair<k>.length` and `slice<i>.length`, with the corner pairs first (DOM-45). The alternatives are `slice<i>.length` for every parameter, or `edge<k>.length` for a corner pair.

Item 36. The limit on the corners (`DEC-48`). The design admits four to eight corners (DOM-38). The alternatives are 12 corners with orders up to 200, or no limit below the 63 corners of `escher-signature`, which Section 6.12 rejects.

Item 37. The range of the lengths (`DEC-14` as amended). The design takes 0.5 to 3.5 and accepts that the guard refuses some settings with seven and eight corners (Section 9.7). The alternative is a bound on the size of the polygon in place of a box.

Item 38. The guard (`DEC-48`). The design verifies the polygons and their doubles inside `build_with` (DOM-44). The alternatives are a guard on every domain, which doubles the cost of `escher info --check`, or verification by the caller, which `escher render` does not do.

Item 39. The double of `p1...pn` (`DEC-50`). The design builds it at the default shape with no parameter (DOM-47). The alternative is the lengths of its generating polygon, a partial set of the parameters of `p1...pn`.

Item 40. The refused doubles of polygons. The guard refuses some doubles with eight orders at their only shape, because each pairing is stored as one `f64` matrix (Section 8.4), and the design raises no tolerance to admit them. The alternatives are to store a pairing as its two reflections, which changes the stored data of every double, or to build the double across another wall.

Item 41. The margin with seven or eight corners and a corner of order 1000. The design builds every such polygon that the guard passes and claims no margin for it (Section 8.4). The alternatives are a lower order limit, a limit of six corners for these signatures, side lengths of step 6 of Section 6.10 in double-double arithmetic, which would leave about as many signatures below 8 as the exact polygon rounded once has (Section 9.7), and a criterion below 8 for checks 1 and 2.

Item 43. The tile of `4*2` and `3*3`. The design takes the kite of `442` or `632` with sides 0 and 1 as mirrors (Section 6.15). The alternative is SymmHub's right triangle, which has a mirror side through a vertex and no source for `3*3`.

Item 44. The layout of the rectangular groups. The design takes the rectangle $F$ of `*2222` with mirror lines and glide axes vertical (Section 6.15). The alternatives are SymmHub's horizontal layout and tiles nested in the grid of `*2222`. `docs/design/render.md` puts the borders of mirror sides to the owner (`DEC-55`).

Item 45. The convention of `skew` (`DEC-52`). The design takes the first coordinate of the second lattice vector over the length of the first, from $-0.5$ to $0.5$, and shears the double of `2222`, so that its numbering and colouring ids stay (Sections 6.6 and 6.13). The alternative is a new tile for `2222`, which would rename its colouring ids (DOM-37).

Item 46. Closed here. TQ-11 of `docs/design/tooling.md` holds the point on tests kept for a reason that no longer applies.

Item 47. The trivial group `1` (`DEC-58`). The design builds it as the whole sphere with no side (DOM-62). The alternative is to leave `1` unsupported, which leaves `escher render 1` without an image.

Item 48. The limit of `2*n` (`DEC-59`). The design admits $n$ up to 500 (DOM-63). The alternative is 1000, which needs the accuracy of the spherical triangles established up to the order 2000, or a new construction.

Item 49. The lunes of order 1 (`DEC-62`). The design builds `*` and `x` as lunes of order 1 with two sides and exact normals (Section 6.16). The alternative is one side with no vertex, which every check and the renderer would have to accept.

Item 50. The split mirror side of `n*` (`DEC-62`). The design keeps the four sides of the kite of `n22` (Section 6.16). The alternative is a triangle with one mirror side, whose measurement by the law of cosines breaks at $n = 2$.

### 15.4. Points Settled by Measurement

The points of the polygons that measurements of the crate settled live in the tests, which print their measured values (DOM-19), and in Sections 6.10, 8.4, and 9.7, which state the method and the claims. The method of the double-double functions needs no dependency, so DOM-17 stands.

## 16. Security Considerations

The threat model is a caller that passes a `Signature` parsed from untrusted text, and settings with untrusted names and values, to `FundamentalDomain::build_with`, and then calls `verify()`. `escher-signature` has already rejected malformed text and bounded the signature (Section 16 of the signature design). The attacker can choose any valid signature. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of time or memory through a large order, a panic that stops the caller, and a wrong domain that the caller uses as correct. The crate counters them with the limit of this section, the rule that it never panics (DOM-2 and DOM-8), and the checks of Section 8.

DOM-3, DOM-18, DOM-38, and DOM-63 require the limits of this table. The input is untrusted, the cost of `verify()` grows with the order and with the number of corners, and the accuracy of Sections 9.3 and 9.7 is established only up to these limits:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Largest corner order | 1000 | The construction costs the same for every order. `verify()` costs at most about 3000 matrix products at this limit for a triangle and 8040 for a polygon of eight corners (Section 8.2), and the relator check keeps a margin of at least 8 at its tolerance of $2 \times 10^{-6}$ over every supported triple (Section 8.4). The tests reach this order with the plan triple $(1000, 3, 2)$ (Section 12.1). The number is a choice of this design (Section 15.3). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin |
| `aspect` | 0.125 to 8, inclusive | A finite value in a bounded range keeps the rectangle within the scale that the tests of Section 9.4 measure. NaN and infinities are rejected (DOM-18). The range is a choice of this design (Section 15.3) |
| `skew` | $-0.5$ to 0.5, inclusive | The range reaches every lattice of the family (Section 6.6), and a finite value keeps the domain within the grid that Sections 8.4 and 9.8 measure. NaN and infinities are rejected (DOM-18). The range is a choice of this design (item 45 of Section 15.3) |
| Order $n$ of `2*n` | 500 | DOM-63. The base of `2*n` has a corner of the order $2n$, which stays within the largest corner order above. The number is a choice of this design (item 48 of Section 15.3) |
| Number of orders | 8 | DOM-38. Beyond it the margins of the checks can fall below 8 even for an exact construction, and the cost of the construction, of `verify()`, and of the colouring search grows with the corners (Section 6.12). The number is a choice of this design (Section 15.3) |
| `cornerpair<k>.length` and `slice<i>.length` | 0.5 to 3.5, inclusive | A finite value in a bounded range keeps the walls of a polygon close enough to the centre for the guard to pass most settings (Section 9.7). NaN and infinities are rejected (DOM-18). The range is a choice of this design (Section 15.3) |

The limit is a private constant, and unit tests inside the crate read it. It applies in the same way to the orders of `*n22` and `*nn` and of every rotation signature, and `verify()` of a spherical domain or of a double costs about as many products as at the hyperbolic limit (Section 8.2). `build_with` allocates at most 14 sides and 14 vertices, for the double of a polygon with eight corners, and at most 6 sheets and 18 gaps for a polygon, so the memory of a domain does not depend on the input. The bisection of DOM-45 stops after at most 200 steps, so a default cannot loop. The walk of check 4 stops after $2m$ steps for each cycle, so a corrupted partner cannot make it loop. A setting is only compared with the names of the `ParameterSpec` values and stored, and the error text echoes no setting (Section 10.2), so the number and the length of the settings need no limit.

For a domain that `build_with` returns, `side_presentation()` builds at most 14 generators and 16 relators, each base of at most six letters, with an exponent of at most 1000, so its memory does not depend on the input beyond the order limit. A domain with more than 256 sides, which only private access builds, gets `InvalidPairing` before any relator is built. The walk of the cycles is the one of check 4 and stops after $2m$ steps for each cycle. The function runs no coset enumeration. The cost of an enumeration of the presentation, and its limits, belong to `escher-group` (Section 15 of the group design).

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library.
- Untrusted input and file paths: the signature is the only input. The crate reads and writes no file and keeps no state between calls.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate neither builds nor prints anything, and the command-line design governs what `escher-cli` prints.

A polygon built from untrusted lengths is verified before it is returned (DOM-44), so a wrong polygon from an extreme setting reaches no caller. The same holds for the ten domains of Section 6.13 with untrusted values of `aspect` and `skew` (DOM-54), whose guard costs a few dozen matrix products, and for the spherical group domains other than that of `*` (DOM-60), whose guard costs at most about 3000 matrix products at the limits. The cost of the guard is one call of `verify()`, at most 8040 matrix products (Section 8.2).

The residual risk is a domain that is wrong by less than the tolerances. A defect that moves a wall or an angle by less than $10^{-9}$, or a relator by less than $2 \times 10^{-6}$ after the division by its order, passes every check. A second residual risk is in the measurements. The sweeps of Sections 8.4 and 9.3 run outside the repository, and no test repeats them. The committed tests measure every triple at which the sweeps have a largest value, but no measurement covers the vertices, normals, and side lengths of every supported triple (Section 9.3), and Section 14 lists the statements that rest on measurements outside the tests.

## 17. Operational Considerations

The crate declares no Cargo feature, reads no environment variable, and has no build script. It is pure Rust with no platform-specific code, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log.

Every failure of `build` is an `Error` with an `ErrorKind`. A failed check is not an error. `verify()` reports it in the `Verification`, and the caller decides what a failed check means:

| Failure | Reported as | Recovery |
| --- | --- | --- |
| A valid signature without a construction | `UnsupportedSignature` | None in this crate. A general non-compact signature has none in the proof of concept (`DEC-19`) |
| More than eight orders | `LimitExceeded` | Use at most eight corners or cone points |
| A corner order above 1000 | `LimitExceeded` | Use orders of at most 1000 |
| The order $n$ of `2*n` above 500 | `LimitExceeded` | Use $n$ of at most 500 |
| A spherical group domain that fails a check of its guard | `Numerical` | None. No order in range is known to produce it, and one that does is a defect of the crate |
| A polygon or its double that fails a check of its guard | `Numerical` | Choose lengths nearer the defaults that `parameters` lists |
| One of the ten domains of Section 6.13 that fails a check of its guard | `Numerical` | None. No setting in range is known to produce it, and one that does is a defect of the crate |
| A setting whose name is not a parameter of the signature | `UnknownParameter` | Use a name that `parameters` lists |
| A value that is not finite or is outside the range of its parameter | `ParameterOutOfRange` | Use a value within the range that `parameters` lists |
| A degenerate construction | `Numerical` | None. No supported signature is known to produce it outside the guard, and one that does is a defect of the crate |
| A domain without a side presentation | `InvalidPairing`, from `side_presentation()` | None. No domain that `build_with` returns produces it, and one that does is a defect of the crate |
| A check outside its tolerance | `Check::passed` is false | None. It is a defect of the crate, and the residual and the tolerance locate it |

## 18. References

### 18.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 18.2. Informative References

The tags of the mathematical sources resolve to the reference sections of the research documents:

- [BEA] Section 16.1 of `docs/research/geometry-models.md`.
- [BUSER] Section 14.3 of `docs/research/fundamental-domains.md`.
- [CBG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CFKP] Section 16.1 of `docs/research/geometry-models.md`.
- [CH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [DAVIS] Section 16.1 of `docs/research/geometry-models.md`.
- [DAVIS2008] Section 14.1 of `docs/research/fundamental-domains.md`.
- [DEATON1993] Section 14.1 of `docs/research/fundamental-domains.md`.
- [EPSTEIN-PETRONIO1994] Section 14.1 of `docs/research/fundamental-domains.md`.
- [HT] Section 16.1 of `docs/research/geometry-models.md`.
- [KC24] Section 16.1 of `docs/research/geometry-models.md`.
- [MASKIT2001] Section 14.1 of `docs/research/fundamental-domains.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [RATCLIFFE] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SERIES2013] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SOLOMYAK2020] Section 14.1 of `docs/research/fundamental-domains.md`.
- [TH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [VOIGHT2009] Section 14.1 of `docs/research/fundamental-domains.md`.
- [VONGAGERN2009] Section 14.1 of `docs/research/fundamental-domains.md`.

The numerical technique of DOM-42 comes from a source that no research document lists. No author of this design read it, so the entry below rests on memory, and Section 14 lists the statements that depend on it as unverified (Section 5.5 of the writing guide):

- [QD] Hida, Y., Li, X. S., and Bailey, D. H., "Algorithms for Quad-Double Precision Floating Point Arithmetic", Proceedings of the 15th IEEE Symposium on Computer Arithmetic (ARITH-15), 2001, pp. 155-162. Not read. The double-double part of its QD library gives the sum, the product, the quotient, and the square root of Section 6.10.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| DOM-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| DOM-2 | `build` constructs a domain only for a signature that meets the predicate of Section 4, and returns `UnsupportedSignature` for every other valid signature without a panic | 4 |
| DOM-3 | `build` returns `LimitExceeded` for a supported signature with an order above 1000, after the support check, and `2*n` takes the limit of DOM-63 instead | 4 |
| DOM-4 | Vertices and sides of a reflection domain follow the index convention of Section 5.1, those of a double that of Section 6.8, those of a wallpaper domain the table of Section 6.13, and those of a spherical group domain the table of Section 6.16 | 5.1 |
| DOM-5 | `build_with` constructs a hyperbolic reflection triangle by the steps of Section 6.1, a polygon by those of Section 6.10, a Euclidean domain by those of Section 6.5, a spherical one by those of Section 6.7, a double by those of Section 6.8, a wallpaper domain and the shear of `2222` by those of Section 6.13, and a spherical group domain by those of Section 6.16, in order | 6.1 |
| DOM-6 | No relation between the inner product of two normals and the corner angle is used, except for the domain with two vertices of DOM-20 | 6.1 |
| DOM-7 | `verify()` computes and reports every check on every call | 8.1 |
| DOM-8 | `verify()` is pure, never panics, and renormalizes nothing | 8.1 |
| DOM-9 | Each check uses the tolerance of Section 8.4 | 8.4 |
| DOM-10 | The vertices are not renormalized after centring or after the rounding of a polygon | 9.1 |
| DOM-11 | The values that `build_with` returns are within the bounds of Section 9.3, of Section 9.4 for a Euclidean domain, of Section 9.5 for a spherical one, of Section 9.7 for a polygon and its double, of Section 9.8 for one of the ten domains of Section 6.13, or of Section 9.9 for a spherical group domain, and for a double also of Section 9.6 | 9.3 |
| DOM-12 | `Error` follows ERR-1, with the operation that failed | 10.1 |
| DOM-13 | The `Display` text of each kind is the one stated, with the prefix of `side_presentation()` for `InvalidPairing` | 10.2 |
| DOM-14 | No test takes an expected value from Table 2 of [HT] | 12.1 |
| DOM-15 | The reference module shares no code with the crates under test | 12.2 |
| DOM-16 | Withdrawn on 2026-11-19. | 12.1 |
| DOM-17 | The crate declares only the dependencies of Section 13, each from the workspace table | 13 |
| DOM-18 | `build_with` checks support, the limits on the number of orders and on the order, the names, and the values, in this order, and returns the first error that applies | 6.6 |
| DOM-19 | A test of a bound names its input set and prints its largest value, and this document records the bound only where it is part of a contract | 9.4 |
| DOM-20 | The angle of a domain with two vertices is measured from its two normals, by the clamped arccosine for a lune, which includes the domains of `nx`, `x`, and `*`, and by `atan2` for the double of a lune, and no other angle is | 7 |
| DOM-21 | For a spherical domain, the vertex precondition of checks 2 to 6 is finite and not the zero vector, with no condition on the last coordinate | 8.2 |
| DOM-22 | The exhaustive certificates cover every lune of their sets and stay while `UNV-29` is open | 12.1 |
| DOM-23 | A rotation signature is built as the double of the reflection domain of its starred signature across the wall of side 1, with that domain left in place and its centre as the centre, except that `2222` with `skew` other than 0 is the shear of that double | 6.8 |
| DOM-24 | Vertices, sides, and partners of a double follow Section 6.8, with the partner $(2n - 1 - k) \bmod (2n - 2)$ of side $k$ | 6.8 |
| DOM-25 | The pairing of an original side is $R_1 R_j$ and of an image side $R_j R_1$, each the compensated product of the two stored reflections, entry by entry | 6.8 |
| DOM-26 | Two halves on one wall at a corner of order 2, or at the centre of a half-turn of a wallpaper domain, stay two sides that meet at a flat vertex, and no paired side is its own partner | 6.8 |
| DOM-27 | Check 2 fails unless the kind, the partner, the orientation, the pairing, and the geometry of every side agree, with `GlideReflection` and `RotaryReflection` reversing and paired with another side, `RotaryReflection` on a spherical domain only and `GlideReflection` off it, and the kind `Translation` exactly for a pairing that passes the translation test, which for a hyperbolic pairing requires a positive determinant and a trace above 3 | 8.2 |
| DOM-28 | Check 4 of a domain with a side that is not a mirror walks the vertex cycles by DOM-55 and evaluates each cycle relator, and a domain whose sides are all mirrors evaluates its corner relators | 8.2 |
| DOM-29 | The angles and the area of a hyperbolic or spherical kite are measured from its two triangles across the doubled wall | 7 |
| DOM-30 | The stored values and the check residuals of every reflection domain do not depend on the code of the doubles, the polygons, the wallpaper domains, and the spherical group domains | 8.4 |
| DOM-31 | The exhaustive certificates cover the spherical doubles and stay while `UNV-22` or `UNV-30` is open | 12.1 |
| DOM-32 | `side_presentation()` gives one generator for each side, named `s<i>`, for $P_i^{-1}$, with words read left to right as products | 5.4 |
| DOM-33 | A reflection domain gives the squares $s_j^2$ and then the corner relators $(s_j s_{j+1})^{n_j}$, in that order | 5.4 |
| DOM-34 | A domain without mirrors gives the pair relators and then the cycle relators of the walk of DOM-55, with each base in walk order and the order of the cycle as exponent | 5.4 |
| DOM-35 | A domain with a number of corners other than its number of sides, more than 256 sides, a walk that does not close within $2m$ steps, a cycle with unequal orders or, for a double, copies that do not sum to 2, or a relator or presentation that `escher-group` rejects gives `InvalidPairing`, checked in that order, without a panic | 5.4 |
| DOM-36 | "Relators evaluate" covers every domain of its sets and "Long cycle word" exists, and both stay while `UNV-32` is open | 12.1 |
| DOM-37 | The numbering of the sides of a built domain is part of the public contract, and depends on no setting, because the colouring ids depend on it | 5.4 |
| DOM-38 | `build` returns `LimitExceeded` for a supported signature with more than eight orders, after the predicate and before the order limit | 4 |
| DOM-39 | The features, cuts, and sheets of a polygon come from the canonical corner orders, each cut sheet is recorded as $(y, x, \bar s_i)$, and a gap that breaks the contiguity condition gives `Numerical` | 6.10 |
| DOM-40 | The features of a sheet keep their counterclockwise cyclic order and are never exchanged | 6.10 |
| DOM-41 | Every gap of every sheet comes from the one gap function with its two branches, and a side is its corner-pair value or the sum of its gaps | 6.10 |
| DOM-42 | The walk and the placement of a polygon run in double-double arithmetic, with each vertex rounded to `f64` once after the placement | 6.10 |
| DOM-43 | A polygon has its centre at the origin and vertex 0 on the positive first axis, and its numbering depends only on the canonical signature | 6.10 |
| DOM-44 | `build_with` returns `Numerical` for a polygon or its double unless `verify().passed()` holds | 6.10 |
| DOM-45 | A polygon has the parameters `cornerpair<k>.length` and then `slice<i>.length`, with the range 0.5 to 3.5 and the default of the inscribed polygon rounded to three decimals | 6.6 |
| DOM-46 | Four hyperbolic vertices are measured as two triangles, five or more by one angle in the corner frame at every vertex, and the area as $(m - 2)\pi$ minus the angle sum | 7 |
| DOM-47 | A rotation signature `p1...pn` with four to eight orders is the double of its starred polygon at the default shape, with no parameter | 6.8 |
| DOM-48 | The polygon tests include identities that share no formula with the construction | 12.1 |
| DOM-49 | Check 4 of a hyperbolic domain with five or more vertices runs in double-double arithmetic with each residual rounded once and the tolerance unchanged, and every other domain keeps check 4 in `f64` | 8.2 |
| DOM-50 | The ten domains have the vertices, numbering, partners, kinds, orientations, and vertex orders of the table of Section 6.13 | 6.13 |
| DOM-51 | Each pairing of the ten domains is a reflection, a translation, a glide reflection, a half-turn with exact entries built from the lines through its centre, or a rotation that a kite keeps from its double. Each mirror side of a kite has the reflection in its own stored wall. No half-turn comes from `euclidean::rotation` | 6.13 |
| DOM-52 | A domain with mirrors and paired sides gives, in side order, the square of each mirror and the pair relator of each pair, and then the cycle relators of the state walk | 5.4 |
| DOM-53 | `o` and `2222` have `aspect` and then `skew`, the six rectangular wallpaper signatures `aspect`, and the kites none, the numbering never depends on a setting, and `2222` at `skew` 0 is the domain of Section 6.8 bit for bit | 6.6 |
| DOM-54 | `build_with` returns `Numerical` for one of the ten domains unless `verify().passed()` holds | 6.13 |
| DOM-55 | The vertex cycles are walked over states of a vertex and a direction by the table of Section 8.2, within $2m$ steps, with equal orders in each cycle and, for every domain without the closure `Cycles`, copies of its distinct vertices that sum to 2 | 8.2 |
| DOM-56 | Check 5 of the wallpaper domains, of `2222`, and of the spherical group domains with the closure `Cycles` compares the angle sum over the states of each cycle with $2\pi$ over its order, and every other domain uses the per-vertex rule | 8.2 |
| DOM-57 | The tests "Wallpaper certificate" and "Equal counts" cover the ten domains, the certificate asserts the point group and the glide reflections off the mirror lines, and both stay while `UNV-11` is open | 12.1 |
| DOM-58 | The spherical group domains have the base, vertices, numbering, partners, kinds, orientations, vertex orders, and closure of the table of Section 6.16 | 6.16 |
| DOM-59 | Each pairing of a spherical group domain is the reflection of its own stored wall, a kept pairing of the double bit for bit, or $H R_i$ by row negation of the stored reflection, with $-I$ for `x` and $\operatorname{diag}(1, 1, -1)$ for `*` exactly | 6.16 |
| DOM-60 | `build_with` returns `Numerical` with the text of a refused spherical domain for `3*2`, `2*n`, `n*`, `nx`, `x`, and `1` unless `verify().passed()` holds, and `*` is not guarded | 6.16 |
| DOM-61 | Check 2 treats `RotaryReflection` as `GlideReflection`, with the endpoints in the same order and the wall term, and fails a `RotaryReflection` off the sphere and a `GlideReflection` on it | 8.2 |
| DOM-62 | A spherical domain with no side has the residual 0 in checks 2 to 5, check 6 against $4\pi$, and the presentation with no generator and no relator, while a domain of another geometry with no side keeps infinite residuals | 8.2 |
| DOM-63 | `build` returns `LimitExceeded` for `2*n` with $n$ above 500, with the text `order 501 is larger than 500`, in place of the limit of DOM-3 | 4 |
| DOM-64 | The spherical certificates assert the order by closure, the exactly-once tiling, and the census, with the five domains of wrong groups as negative tests and the order by enumeration, cover every spherical group domain of their sets, and stay while `UNV-34` is open | 12.1 |
