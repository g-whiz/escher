# Domain Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-17 |
| Applies to | `crates/escher-domain` |

## Abstract

This document specifies the crate `escher-domain`, which builds the fundamental domain of an orbifold signature as a polygon in the linear model, with a wall, a pairing isometry, and a partner for each side, and verifies it numerically. It covers the compact hyperbolic reflection triangle `*pqr`, the three Euclidean reflection triangles `*632`, `*442`, and `*333`, the rectangle `*2222` with its shape parameter `aspect`, the spherical reflection triangles `*332`, `*432`, `*532`, and `*n22`, and the lune of `*nn`. It defines the public types and operations, the exact set of signatures that have a domain, the shape parameters and their validation, each construction as a numbered algorithm with the source of each step, the placement of the polygon, the measurements of its sides, angles, and area, and the six numeric checks of the verification with their tolerances and the defects that each one detects. It also states the numerical accuracy over the supported orders, the error type, the resource limits, the tests, the dependencies, the unverified statements that the code relies on, the assumptions, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

The renderer folds every pixel into the fundamental domain and colours it by the side pairings that the fold applied. It needs a polygon whose sides carry exact walls and pairing isometries, placed so that the canonical view is centred on it. The research gives the angles of the reflection triangle, its side lengths by the law of cosines, and the theorems that make it a fundamental domain, but it gives no placement in the linear model and no numerical tolerance (Section 10 of the geometry reference and Section 4.8 of the domain reference). A wrong normal, a wrong angle, or a wrong pairing produces an image that looks plausible, so the crate must check its own result.

### 1.2. Purpose

`escher-domain` is the one place where a `Signature` becomes a `FundamentalDomain`. It builds the polygon of a supported signature, returns an error kind for every other signature, and verifies a domain against the conditions of the polygon theorems that the domain reference cites. Section 3 of the architecture places the crate in the workspace. It depends on `escher-signature` for the signature and on `escher-geometry` for the linear model, and the renderer and `escher-cli` consume it.

### 1.3. Audience

The primary readers are the agent that changes the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-cli`, which prints the domain summary and the result of the verification, and of `escher-render`, which folds into the domain.

### 1.4. Scope

This document covers the public surface of the crate, the supported signatures, the shape parameter of `*2222`, the conventions of the sides, the vertices, and the pairings, the construction of the compact hyperbolic reflection triangle, the Euclidean triangle, the rectangle, the spherical triangle, and the lune, their placement, their measurements, the verification, the numerical accuracy, the error type, the resource limits, the tests, and the dependencies.

The supported signatures are the compact hyperbolic `*pqr` with $1/p + 1/q + 1/r < 1$, the Euclidean `*632`, `*442`, `*333`, and `*2222`, and the spherical `*332`, `*432`, `*532`, `*n22`, and `*nn` (Section 4).

### 1.5. Non-Goals

These topics are owned elsewhere or are outside this document:

- The bilinear form, the side test, the distance, the isometries, the reflection, the wall through two points, and the disk map, with their formulas and tolerances, belong to `escher-geometry` and are specified in `docs/design/geometry.md`. This document calls those functions by name and restates none of their formulas.
- The lines that `escher info` prints for a domain and for `--check`, their format, and the exit status are specified in `docs/design/cli.md`.
- Every shape parameter other than `aspect` of `*2222`, and the kind and unit of a `ParameterSpec` that Section 3.2 of the architecture names. A reflection triangle has no free parameter ("triangles are rigid", Section 10.1 of the geometry reference).
- The side presentation, which needs `escher-group` (Section 3.2 of the architecture).
- Ideal vertices, the Euclidean and spherical signatures other than those of Section 4, and every other class of signature in the plan. Each returns `UnsupportedSignature` (Section 4).
- The canonical view, the model, and the margin of `DEC-12`, which belong to `View` in `escher-render`. This crate places only the centre (Section 6.3).

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
| `docs/research/geometry-models.md` | the geometry reference |
| `docs/research/fundamental-domains.md` | the domain reference |
| `docs/research/orbifold-notation.md` | the notation reference |
| `docs/rust-coding-standards.md` | the coding standard |

A function written as `hyperbolic::name`, `euclidean::name`, or `spherical::name` is the function of that name in the module `hyperbolic`, `euclidean`, or `spherical` of `escher-geometry`, which the geometry design specifies.

It adds these terms:

- supported signature: A signature that meets the predicate of Section 4. `FundamentalDomain::build` constructs a domain for it when its corner orders are at most 1000 (DOM-3).
- corner $j$: The corner of the canonical signature at index $j$ of `boundaries()[0].corner_orders()`, with the order $n_j$ and the angle $\alpha_j = \pi/n_j$.
- $m$: The number of sides of a domain, which is also its number of vertices and corners: 2 for a lune, 3 for a triangle, and 4 for the rectangle.
- origin: `hyperbolic::ORIGIN` for a hyperbolic domain, `euclidean::ORIGIN` for a Euclidean one, and `spherical::ORIGIN` for a spherical one. All three are $(0, 0, 1)$.
- lune: The intersection of two distinct, nonopposite hemispheres of $S^2$, with two vertices and the same angle at both ([RAT], p. 51, Section 11.3 of the geometry reference).
- standard position: The placement in which vertex 2 is the origin and vertex 0 lies on the positive first axis, for a triangle (Section 6.1, step 5, and Section 6.5, step 2).
- final position: The placement in which the centre of `DEC-12` is the origin (Sections 6.3 and 6.5).
- shape parameter: A named real number that selects one domain of a signature whose domains form a family, such as `aspect` of `*2222` (Section 6.6).
- setting: A name and a value in a `ShapeParameters`.
- corner frame: The frame of corner $j$, in which vertex $j$ is `hyperbolic::ORIGIN` (Section 8.2, check 4).
- check: One numeric condition that `verify()` evaluates, with a residual and a tolerance.
- residual: The non-negative number that a check computes, which is 0 for an exact domain, and infinity when a structural condition fails or when the computation gives no finite number. It is never NaN.
- tolerance: The largest residual for which a check passes.

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
    #[must_use] pub fn verify(&self) -> Verification;
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
pub enum PairingKind { Mirror }
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
pub enum ErrorKind { UnsupportedSignature, LimitExceeded, Numerical, UnknownParameter, ParameterOutOfRange }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

The crate does not re-export `glam` (API-10 of the coding standard). A caller that names a `glam` type declares `glam` from the workspace entry, as Section 3.1 of the architecture states for the crates that compute with these types.

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `FundamentalDomain` | The fundamental domain of a supported signature | Built only by `build` and `build_with`. It holds the geometry and the corner orders of the signature, the area of a hyperbolic or spherical signature, and the sides, the vertices, and the centre |
| `build` | The constructor with no setting | Equal to `build_with` with an empty `ShapeParameters`, so each shape parameter takes its default value |
| `build_with` | The constructor with settings | Returns a domain exactly for a supported signature with corner orders of at most 1000 (Section 4) and valid settings (Section 6.6), and an `Error` otherwise. Never panics |
| `geometry()` | The geometry of the signature | `Hyperbolic`, `Euclidean`, or `Spherical` |
| `sides()` | The sides, in the order of Section 5.1 | $m$ sides of kind `Mirror` |
| `vertices()` | The vertices, in the order of Section 5.1 | As many as the sides. Vertex $j$ is corner $j$. The vertices are counterclockwise in the disk, in the plane for a Euclidean domain, where each is a Euclidean point $(x, y, 1)$, and as seen from outside the sphere for a spherical domain, where each is a sphere point whose last coordinate may be zero or negative (Section 5.2). Each has the accuracy of Section 9.3, 9.4, or 9.5, and a caller does not renormalize it (Section 9.1) |
| `centre()` | The centre of `DEC-12`: the normalized sum of the vertices, for a Euclidean domain their mean, and for a lune `spherical::ORIGIN` (Section 6.7) | Within $10^{-9}$ of the origin in each coordinate for a hyperbolic domain (Section 6.3), within the bound of Section 9.4 for a Euclidean one, within the bound of Section 9.5 for a spherical triangle, and exactly the origin for a lune |
| `Side::normal` | The unit normal of the wall of the side, for a Euclidean domain the unit normal $(a_1, a_2, 0)$ of its line, and for a spherical domain the great-circle normal of its great circle | Outward (Section 5.2), and spacelike for a hyperbolic domain |
| `Side::covector` | The covector of the wall | `hyperbolic::covector(normal)` for a hyperbolic domain, the line covector $(a_1, a_2, -t)$ for a Euclidean one, and the normal itself for a spherical one |
| `Side::pairing` | The pairing isometry $P_j$ of the side (Section 5.3 of the architecture) | For a mirror, `hyperbolic::reflection(normal)`, `euclidean::reflection(covector)`, or `spherical::reflection(normal)`, an orthogonal matrix |
| `Side::partner` | The index of the side onto which $P_j$ maps side $j$ | For a mirror, $j$ itself |
| `Side::kind` | The kind of the pairing | `Mirror` for every side of a reflection triangle, of the rectangle, and of a lune |
| `Side::orientation` | Whether $P_j$ preserves or reverses orientation | `Reversing` for a mirror |
| `side_lengths()` | The length of each side, in side order | Section 7 |
| `corner_angles()` | The angle at each vertex in radians, in corner order | Section 7 |
| `area()` | The area measured from the corner angles, or for a Euclidean domain from the vertices | Section 7. For a spherical domain, in the metric of the unit sphere |
| `verify()` | The result of the six checks of Section 8.2 | Pure and never panics (DOM-8) |
| `Verification::passed` | Whether every check passed | True exactly when each of the six `Check::passed` is true |
| `Check::passed` | Whether the residual is within the tolerance | `residual() <= tolerance()`. The residual is never NaN, and an infinite residual fails |
| `parameters` | The shape parameters of a signature, in a fixed order | Total. One `ParameterSpec` for `*2222`, and an empty list for every other signature, supported or not (Section 6.6) |
| `ParameterSpec` | The name, range, and default value of one shape parameter | `minimum() <= default_value() <= maximum()`, each finite. The name is the plain name of Section 5.6 of the architecture |
| `ShapeParameters` | The settings that a caller passes to `build_with` | Holds each name at most once. `new()` equals `Default::default()`, an empty set. `set` stores the name and the value that it is given and returns the value that it replaces, if any. It checks nothing, and `build_with` validates (Section 6.6) |
| `Error` | Why no domain was built | Section 10 |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

`Orientation` has both variants, although every side of a reflection triangle reverses orientation, because the renderer colours by the parity of the orientation of the pairings (Section 5.3 of the plan), and the classes that have rotations and glide reflections add `Preserving` sides without a change to the type. It is a closed set, so it is exhaustive under a lint exception (API-9 of the coding standard). `PairingKind` has one variant and grows with later classes, so it is `#[non_exhaustive]`.

`Verification` has one accessor for each check and no enum of checks, so a caller needs no wildcard arm, and a later check is a new accessor.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, and the unit test "Auto traits" of Section 12.1.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/domain.rs`: `FundamentalDomain`, `Side`, `PairingKind`, `Orientation`, the accessors, and the measurements of Section 7.
- `src/triangle.rs`: The supported-signature predicate of Section 4, with a private enum of the five supported classes, and the construction of Section 6.1.
- `src/polygon.rs`: The constructions of the Euclidean triangle and the rectangle of Section 6.5, and the unit test "Exact rectangle" of Section 12.1.
- `src/spherical.rs`: The constructions of the spherical triangle and the lune of Section 6.7.
- `src/parameters.rs`: `parameters`, `ParameterSpec`, `ShapeParameters`, the private enum of the parameters with the name, the range, and the default of each, and the validation of Section 6.6.
- `src/verify.rs`: `Verification`, `Check`, and the checks of Section 8.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, because the supported signatures do not need them:

- The kind and the unit of a `ParameterSpec`. `aspect` is a ratio, so it needs neither. The class that first needs them adds them as accessors, which does not break a caller. The crate sets `publish = false`, so a change affects only the workspace.
- `side_presentation()`, which needs `escher-group`.
- Ideal vertices, which the triangle with an ideal corner adds.

`build` and `build_with` are inherent constructors and return a `Result`. `build_with` is a secondary constructor in the sense of API-5 of the coding standard. `ShapeParameters` takes the place of the options struct of API-6, with a private map and the method `set` and not public fields, because the names of the parameters depend on the signature. `build` stays, so that its callers do not change. Section 3.2 of the architecture lets the crate design refine the names and signatures.

`parameters` returns a `Vec` and `ParameterSpec::name` a `&str`, because later classes have parameter names that `escher-domain` derives from the signature, such as `slice1.length` (Section 5.6 of the architecture). `ShapeParameters` is a map, so it cannot hold a name twice (TYPE-3 of the coding standard), and the return value of `set` lets a caller detect a name set twice.

## 4. Supported Signatures

DOM-2. `FundamentalDomain::build` MUST construct a domain only for a signature that meets the predicate of this section, subject to DOM-3, and MUST return `UnsupportedSignature` for every other valid signature without a panic, because a caller that receives a domain for a signature outside the construction folds into a polygon that does not tile, and a panic stops the caller's whole program (ERR-5 of the coding standard).

A signature is supported exactly when all of these hold, each read from the public accessors of `escher-signature` (Section 3.1 of the signature design):

1. `handles()` is 0.
2. `crosscaps()` is 0.
3. `cone_orders()` is empty.
4. `boundaries()` has exactly one element, and its `corner_orders()` has two, three, or four elements.
5. `finite()` of each order is `Some`. This holds exactly when `is_compact()` is true for such a signature, and the crate reads `finite()` so that it needs no `expect` later.
6. One of these holds:
   - three corners, and `geometry()` is `Geometry::Hyperbolic`.
   - three corners, and `geometry()` is `Geometry::Euclidean`.
   - four corners, and `geometry()` is `Geometry::Euclidean`.
   - three corners, and `geometry()` is `Geometry::Spherical`.
   - two corners, and `geometry()` is `Geometry::Spherical`.

For a hyperbolic or spherical signature, the predicate also needs `area_over_pi()` to be `Some`, which it is for every such signature that meets conditions 1 to 6 (Section 9.4 of the signature design), so this condition makes no further signature unsupported. Step 1 of Section 6.1 converts the area.

The geometry is the exact rational classification of `escher-signature` (Section 9.1 of the signature design), so no floating-point comparison decides support. For three finite corners, the hyperbolic case of condition 6 is the condition $1/p + 1/q + 1/r < 1$ of Section 9.1 of the domain reference and Section 10.1 of the geometry reference. The Euclidean case with three corners is exactly `*632`, `*442`, and `*333`, the three solutions of $1/p + 1/q + 1/r = 1$ in Example 6.5.2 of [DAVIS2008] and in Section 7.2 of [RAT], p. 279 (Section 9.1 of the domain reference). The Euclidean case with four corners is exactly `*2222`, the only signature of one boundary with four corners among the 17 Euclidean signatures of Section 9.5 of the domain reference. The spherical case with three corners is exactly `*332`, `*432`, `*532`, and `*n22` with $n \ge 2$, the triples of Example 6.5.1 of [DAVIS2008] (Section 9.1 of the domain reference), which include `*222`. The spherical case with two corners is exactly `*nn` with $n \ge 2$, which include `*22`, because `escher-signature` rejects `*mn` with $m \ne n$ as a bad orbifold.

Every other valid signature returns `UnsupportedSignature`. This covers the other Euclidean signatures, such as `632` and `*x`, the other spherical signatures, such as `532`, `*`, `3*2`, and `22`, the hyperbolic `*3333`, `732`, and `4*3`, the triangle with an ideal corner `*(inf)32`, the friezes, the trivial signature `1`, and every general non-compact signature. A general non-compact signature returns this kind throughout the proof of concept (`DEC-19`, Section 5.8 of the architecture).

DOM-3. `build` MUST return `LimitExceeded` for a supported signature with a corner order above 1000, and MUST apply the predicate of this section first, because the accuracy of Section 9.3 is established only up to that order, and a domain outside it could pass its checks by a smaller margin than the tolerances assume. A signature that is both unsupported and over the limit gets `UnsupportedSignature`, because the limit is a property of this construction. Section 16 states the reason for the number. The orders of a supported Euclidean signature are at most 6, so the limit does not bind for it. It binds for `*n22` and `*nn`, so `*(1001)22` and `*(1001)(1001)` return `LimitExceeded`.

## 5. Sides, Vertices, and Pairings

### 5.1. Index Convention

The canonical form of `*pqr` lists its corners with $n_0 \ge n_1 \ge n_2$ (Section 7.2 of the signature design, step 4). So `*237` and `*327` both become `*732`, and vertex 0 has the smallest angle and vertex 2 the largest.

DOM-4. The crate MUST number the vertices and sides by this convention, because `escher-cli` prints the angles in corner order and the side lengths in side order, and the renderer records the side index of every fold step, so a second numbering gives two readings of one printed value:

- Vertex $j$ is corner $j$, with the order $n_j$.
- Vertex $j$ is the common point of side $j$ and side $j + 1$, with indexes modulo $m$. So side $j$ joins vertex $j - 1$ and vertex $j$.
- Corner $j$ lies between side $j$ and side $j + 1$, so its relator is $(P_j P_{j+1})^{n_j}$.

Section 9.1 of the notation reference reads a boundary with corners of orders $a$, $b$, and $c$ as the Latin reflections $P$, $Q$, $R$, and $S$ with the relators $(PQ)^a$, $(QR)^b$, and $(RS)^c$. Side 0 is $P$, side 1 is $Q$, and side 2 is $R$. For a triangle the convention reads $S$ as $P$ again, and for the rectangle side 3 is $S$. A lune has $m = 2$, so side 0 joins vertex 1 and vertex 0, side 1 joins vertex 0 and vertex 1, and its relators are $(P_0 P_1)^{n}$ at corner 0 and $(P_1 P_0)^{n}$ at corner 1. Section 15.1 records this reading as an assumption. The relators that the checks use come from Section 4.4 of the domain reference, which has three reflections and does not depend on it.

### 5.2. Orientation and Outward Normals

The vertices of a hyperbolic domain are counterclockwise in the disk: the planar cross product of `hyperbolic::to_disk(V1) - hyperbolic::to_disk(V0)` and `hyperbolic::to_disk(V2) - hyperbolic::to_disk(V0)` is positive. The vertices of a Euclidean domain are counterclockwise in the plane: the same cross product of the first two coordinates of $V_1 - V_0$ and $V_2 - V_0$ is positive. The vertices of a spherical triangle are counterclockwise as seen from outside the sphere: $(V_0 \times V_1) \cdot V_2$ is positive. The two vertices of a lune do not fix an orientation, and its placement in Section 6.7 numbers its sides counterclockwise as seen from outside the sphere at its centre.

The normal of each side points outward, as Section 5.2 of the architecture requires. A point $x$ is inside the domain where `hyperbolic::side(covector, x)`, `euclidean::side(covector, x)`, or `spherical::side(covector, x)` is at most 0 for every side, and a positive value means that the point is outside that side. Section 2.4 of the geometry reference records that [RAT] orients normals inward. This crate uses no statement of [RAT] that depends on that orientation.

### 5.3. Pairings

Side $j$ carries the isometry $P_j$ that maps the neighbouring copy of the domain across side $j$ onto the domain (Section 5.3 of the architecture). For a mirror side, $P_j$ is the reflection in its wall, `hyperbolic::reflection(normal)`, which is the reflection of equation (6.6) of [DAVIS] (Section 7.4 of the geometry reference) as Section 5.2 of the architecture adopts it. For a Euclidean mirror side, $P_j$ is `euclidean::reflection(covector)`, the reflection of equation (4.1.1) of [RAT] and equation 6.3 of [DAVIS] (Sections 7.4 and 12 of the geometry reference), whose form as a matrix is part of `UNV-26` (Section 14). For a spherical mirror side, $P_j$ is `spherical::reflection(normal)`, the reflection of equation (6.4) of [DAVIS] (Section 7.4 of the geometry reference). The partner of a mirror side is the side itself.

## 6. Construction

### 6.1. Algorithm

DOM-5. `build_with` MUST construct a hyperbolic domain by the steps of this section, a Euclidean domain by the steps of Section 6.5, and a spherical domain by the steps of Section 6.7, each in its order, because the accuracy of Sections 9.3 to 9.5 and the margins of Section 8.4 are measured for exactly these steps, and another order of the same operations gives other rounding errors that no test measures.

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

The domain stores the geometry of the signature, the corner orders and the area of step 1, and the sides, vertices, and centre of steps 8 to 10. It does not keep the signature, because `build` borrows it and the checks of Section 8.2 need only these values.

The rotation of step 5 is counterclockwise by $\alpha_2$ about the origin, as `hyperbolic::rotation` defines it, so vertex 1 lies counterclockwise from vertex 0 as seen from vertex 2, which gives the orientation of Section 5.2.

DOM-6. The construction and the measurements MUST NOT use a relation between the inner product of two normals and the corner angle, except for the corner angle of a lune as DOM-20 states, because the only such relation in the research is stated with inward normals in the conventions of [RAT] (Section 7.4 of the geometry reference), and its form with the outward normals of this project is part of `UNV-26`. The corner angles are measured from the distances between the vertices instead (Section 7).

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
- The final position moves the centre of `DEC-12` to the origin. The owner decided on one canonical view centred on the domain (Section 10.1 of the architecture), and Section 5.3 of the plan places the centre of the domain at the centre of the disk. With this placement, the canonical view of the disk is the plain disk map. `centre()` stays public, so `View` in `escher-render` keeps the freedom that Section 3.2 of the architecture gives it.

The final position also keeps the walls close to the origin. The entries of `hyperbolic::reflection(u)` grow with the products of the components of $u$, and the renderer applies such a matrix at every fold step. In the final position, the sweep of Section 8.4 finds the largest component of a normal over every supported triple at 8.935, at $(1000, 4, 4)$, and no supported triple fails a check. Section 14 records the figures for a domain left with vertex 2 at the origin, which no measurement of the code covers.

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

The triangle has side 0, from vertex 2 to vertex 0, of length 1. The rectangle has the area 1, side 0 at the bottom and side 1 on the right, and `aspect` is the length of side 0 divided by the length of side 1. These units are choices of this design (`DEC-17`), which Section 15.3 puts to the owner.

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

The vertex coordinates of the triangle have no source, so the build is certified three ways: check 3 tests each vertex against its two walls, check 5 tests each angle against $\pi/n_j$ by equation (1.3.3) of [RAT], and the test "SymmHub lines" of Section 12.1 compares the built walls with the three lines that SymmHub's code gives for this triangle (Section 9.1 of the domain reference), moved by the translation of step 4. Building the walls from those lines and the vertices by intersection would not close the gap, because the intersection of two lines has no source either.

### 6.6. Shape Parameter and Validation

`*2222` has the one shape parameter `aspect`, and every other signature has none:

| Name | Minimum | Maximum | Default | Meaning |
| --- | --- | --- | --- | --- |
| `aspect` | 0.125 | 8 | 1 | The length of side 0 divided by the length of side 1 of the rectangle |

The range and the default are choices of this design (Section 15.3). The default is the square, as `DEC-13` asks for a regular shape where the orders allow it. Both ends are exact in `f64`. The scale is fixed by the area 1 and is not a parameter, and the design asserts no count of parameters, so `deformation_dimension()` of `escher-signature` stays as the signature design states (`UNV-10`).

DOM-18. `build_with` MUST check in this order, and MUST return the first error that applies, because `escher-cli` prints the error of a combination of a signature and settings, and its snapshots fix which kind each combination gives:

1. The predicate of Section 4, else `UnsupportedSignature`.
2. The limit of DOM-3, else `LimitExceeded`.
3. Every name of a setting is the name of a `ParameterSpec` of the signature, else `UnknownParameter`.
4. Every value is finite and within the closed range of its `ParameterSpec`, else `ParameterOutOfRange`. A NaN or an infinite value is out of range.
5. A parameter without a setting takes its default value.

A name matches only when it is equal, byte for byte, to the name of a `ParameterSpec`, so `Aspect` and `aspect ` with a trailing space are unknown.

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

1. Compute $\theta$ as `PI / f64::from(n)`, and $(s, c)$ as `(theta / 2.0).sin_cos()`.
2. Place the lune directly in the final position: $V_0 = (0, 1, 0)$, $V_1 = (0, -1, 0)$, and the centre `spherical::ORIGIN`.
3. Side 0 gets the normal $(c, 0, -s)$ and side 1 the normal $(-c, 0, -s)$. Each side gets its normal as its covector, the pairing `spherical::reflection(normal)`, the partner $j$, the kind `Mirror`, and the orientation `Reversing`.
4. Return `Numerical` if any stored number is not finite, or if the value of `spherical::side` at the centre is not strictly negative for both sides.

The domain stores the geometry, the corner orders, and the area of the signature, and the sides, vertices, and centre. The rule of `DEC-12` gives the zero vector as the centre of two antipodal vertices, so the centre of the lune is a choice of this design (Section 15.3).

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
| L3 | The two sides meet at the angle $\theta$ at each vertex | The definition of a lune and of its angle, [RAT], p. 51. That the outward normals of step L3 give this angle has no source (item 28 of Section 15.1 of the geometry reference, part of `UNV-26`). Checks 4 and 5 and the test "Exhaustive certificate" test it |
| All, triangle | The reflections generate a discrete group, and the triangle is a strict fundamental domain for it | Theorem 6.4.3 of [DAVIS2008], p. 83, for a convex polytope in $S^n$ as p. 75 defines it (Sections 4.5 and 9.6 of the domain reference) |
| All, lune | The two reflections generate a group of order $2n$, and the lune is a strict fundamental domain for it | No source. `UNV-29` (item 9 of Section 13.3 of the domain reference), which the test "Exhaustive certificate" checks (DOM-22) |

## 7. Measurements

The crate measures each quantity from the stored vertices, so that `verify()` and `escher-cli` see the same numbers:

- `side_lengths()` returns `hyperbolic::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the angle $\theta_j$ in $[0, \pi]$ for which the first law of cosines of Section 10.2 of the geometry reference ([BEA] Section 7.12, p. 148, and [RAT] Theorem 3.5.3, p. 86) holds with the three measured side lengths. With $c$ the length of the side opposite vertex $j$ and $a$ and $b$ the other two, it is $\arccos\big((\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)\big)$, with the argument clamped to $[-1, 1]$.
- `area()` returns $\pi - \theta_0 - \theta_1 - \theta_2$, the area of a triangle by Section 10.1 of the geometry reference.

For a hyperbolic domain, `corner_angles()` and `area()` are defined for exactly three vertices. With any other number of vertices, `corner_angles()` returns NaN for every vertex and `area()` returns NaN.

The angles are measured from distances so that no step uses a relation between normals and angles (DOM-6). The form of the first law solved for the angle is a rearrangement that no source prints, and Section 15.1 records it.

For a Euclidean domain, with $(x_j, y_j)$ the first two coordinates of $V_j$:

- `side_lengths()` returns `euclidean::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the Euclidean angle $\theta$ of equation (1.3.3) of [RAT] (Section 12 of the geometry reference) between $V_{j+1} - V_j$ and $V_{j-1} - V_j$, as $\arccos$ of their dot product divided by the product of their lengths, with the argument clamped to $[-1, 1]$. It uses no normal, as DOM-6 requires.
- `area()` returns $\frac{1}{2} \left\lvert \sum_j (x_j y_{j+1} - x_{j+1} y_j) \right\rvert$, with indexes modulo $m$. No source states this formula (item 25 of Section 15.1 of the geometry reference), and Section 14 records it.

For a spherical domain, the measurements read the theorems of [RAT], Section 2.5 (Section 11.3 of the geometry reference), with lengths and areas in the metric of the unit sphere:

- `side_lengths()` returns `spherical::distance(V_{j-1}, V_j)` for each side $j$. Both sides of a lune have the length $\pi$.
- `corner_angles()` returns, for each vertex $j$ of a triangle, $\arccos\big((\cos c - \cos a \cos b) / (\sin a \sin b)\big)$, with the argument clamped to $[-1, 1]$, where $c$ is the measured length of the side opposite vertex $j$ and $a$ and $b$ the other two. This is the first law of cosines, [RAT], Theorem 2.5.3, p. 50, which prints the cosine of the angle in this form. For a lune, it returns the same angle $\theta = \arccos(-u_0 \cdot u_1)$ for both vertices, with the argument clamped to $[-1, 1]$, from the normals $u_0$ and $u_1$ of its two sides (DOM-20).
- `area()` returns $\theta_0 + \theta_1 + \theta_2 - \pi$ for a triangle, [RAT], Theorem 2.5.5, p. 51, and $2\theta_0$ for a lune, the area of a lune of [RAT], p. 51.

For a spherical domain with a number of vertices other than two or three, `corner_angles()` returns NaN for every vertex and `area()` returns NaN.

DOM-20. `corner_angles()` MUST measure the angle of a lune from the normals of its two sides as $\arccos(-u_0 \cdot u_1)$, and MUST NOT use that relation for any other domain, because the two vertices of a lune are antipodal and so determine neither side, while for every other domain the vertices determine the angle and DOM-6 applies. The relation has no source (item 28 of Section 15.1 of the geometry reference). Check 4 and the test "Exhaustive certificate" test the same angle without it.

## 8. Verification

### 8.1. Report Types

`verify()` returns a `Verification` that holds one `Check` for each of the six checks of Section 8.2. Each `Check` holds its residual and its tolerance. A check passes when its residual is at most its tolerance, so an infinite residual fails. Section 2.1 defines the residual, which is never NaN.

DOM-7. `verify()` MUST compute every check of Section 8.2 on every call and MUST report each one separately, because `escher info --check` prints the result of each check, and a verification that stops at the first failure hides whether the other conditions hold.

DOM-8. `verify()` MUST be pure, MUST NOT panic for any domain, and MUST NOT renormalize any stored value, because it runs on domains that tests corrupt on purpose, and a renormalized value would hide the defect that the check exists to find.

### 8.2. Checks

Every check runs on the stored values of the domain in the final position, unless the check names another frame. Checks 2 to 6 read the vertices, so each first requires at least one side, as many vertices and corner orders as sides, and every vertex to have finite coordinates and, for a hyperbolic or Euclidean domain, a positive last coordinate, or for a spherical domain, not to be the zero vector (DOM-21). Its residual is infinity when any of these fails. Check 1 reads only the pairings. A residual that is not finite after the computation of its check is reported as infinity:

1. **Isometries.** For each side, compute `hyperbolic::isometry_defect(pairing)`. The residual is the largest value. Tolerance $10^{-9}$. Condition: Theorem 10.1 of [CFKP] (Section 6.1 of the geometry reference).
2. **Pairings.** The residual is infinity unless both structural conditions hold, and otherwise the larger of the two numeric parts. Tolerance $10^{-9}$. Conditions: items 1 and 2 of Section 4.8 of the domain reference, and Condition 3.2 (a) to (d) of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference).
   - Structural: `partner(partner(i)) == i` for every side.
   - Structural: with $c$ the stored centre, `hyperbolic::side(covector_i, c)` is negative for every side $i$, and `hyperbolic::side(covector_partner(i), P_i c)` is positive.
   - Numeric: the largest absolute entry of $P_i P_{\text{partner}(i)} - I$ over the sides. For a mirror this is the relator $r_i^2 = 1$.
   - Numeric: for a mirror side $i$, the largest absolute coordinate of $P_i E - E$ divided by the last coordinate of $E$, for the endpoints $E = V_{i-1}$ and $E = V_i$.
3. **Walls.** For each side $j$, compute these four values. The residual is the largest of them over the sides. Tolerance $10^{-9}$. Condition: a wall is a unit normal stored with its covector (Sections 2.1 and 5.2 of the architecture), it is orthogonal to the points of its side (equation (6.6) of [DAVIS], Section 7.4 of the geometry reference), and the pairing of a mirror is the reflection in that wall (Section 5.3).
   - The absolute value of `hyperbolic::side(covector_j, V_{j-1})` divided by the last coordinate of $V_{j-1}$.
   - The same for $V_j$.
   - The largest absolute entry of `pairing_j - hyperbolic::reflection(normal_j)`.
   - The largest absolute coordinate of `covector_j - hyperbolic::covector(normal_j)`.
4. **Relators.** For each corner $j$, let $F =$ `hyperbolic::translation(V_j)`, which reads only the first two coordinates of $V_j$, and let $C =$ `hyperbolic::inverse(F) * P_j * P_{j+1} * F`. Compute $C^{n_j}$ by $n_j - 1$ successive multiplications by $C$. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. Tolerance $2 \times 10^{-6}$. Condition: the relator $(r_j r_{j+1})^{n_j} = 1$ of the presentation of Section 4.4 of the domain reference ([MASKIT2001] Section 4.4), and item 4 of its Section 4.8.
5. **Angles.** Measure each $\theta_j$ as Section 7 states, with $n_j$ the stored corner orders, which check 4 also reads. The residual is the largest $\lvert \theta_j - \pi/n_j \rvert$, or infinity when $\theta_0 + \theta_1 + \theta_2$ is not below $\pi$. Tolerance $10^{-9}$. Condition: every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference), and the angle sum of a hyperbolic triangle is below $\pi$ (Section 10.1 of the geometry reference).
6. **Area.** The residual is the absolute difference between `area()` and the area of the signature that step 1 of Section 6.1 computed and the domain stores, which is $\pi$ times `area_over_pi()`, converted with no limit on the size of its numerator or denominator. Tolerance $10^{-9}$. Condition: the area of the triangle is $\pi$ minus its angle sum (Section 10.1 of the geometry reference), and the area of a hyperbolic orbifold is $-2\pi\chi$ (Section 7.3 of the notation reference).

Check 6 is a consistency check between the area that the crate measures and the area that the Euler characteristic of the signature gives, $-2\pi\chi$, by the defect rule of Section 6.1 of the notation reference. It is not an independent test of the geometry. Section 14 records a figure that compares its residual with that of check 5, which no measurement of the code covers. It guards against a disagreement between this crate and `escher-signature` about the orders of the corners or the area of the orbifold. Section 15.3 asks the owner whether to keep it.

Check 4 evaluates the relator in the corner frame, where vertex $j$ is the origin. A power of a matrix is the identity exactly when the same power of its conjugate is, which Section 15.1 records as an assumption. The frame is a choice for accuracy. The test "Relators in the domain frame" of Section 12.1 computes, in the reference arithmetic, the residual of check 4 from the exact power of the `f64` pairings in each frame, the largest over the three corners:

| Triple | Domain frame | Corner frame | Tolerance of check 4 |
| --- | --- | --- | --- |
| $(998, 981, 961)$ | $5.426 \times 10^{-5}$ | $1.338 \times 10^{-7}$ | $2 \times 10^{-6}$ |
| $(1000, 3, 2)$ | $3.286 \times 10^{-12}$ | $8.746 \times 10^{-14}$ | $2 \times 10^{-6}$ |

At $(998, 981, 961)$, the triple with the largest residual of check 4 (Section 8.4), the domain frame fails the tolerance even with exact arithmetic, and the corner frame passes it. Check 4 has the smallest margin of the six. Repeated multiplication is chosen over repeated squaring, on figures that Section 14 records and no measurement of the code covers.

Check 4 alone does not fix the angle, because it also passes for an angle of $k\pi/n_j$. Check 5 closes that gap. Check 5 uses only the vertices, and checks 2, 3, and 4 tie the walls and the pairings to the vertices.

`verify()` costs at most $3 \cdot 999 + 18$ matrix products at the order limit. Each corner needs $n_j - 1$ products for the power, two for its frame, and three for its conjugate, and check 2 needs three more for the pairing relators.

For a Euclidean domain, `verify()` computes the same six checks with the same tolerances, and reads each one in this way. The vertex condition of the checks 2 to 6 holds for a Euclidean point, whose last coordinate is 1, and indexes are modulo $m$:

| Check | Euclidean reading | Condition |
| --- | --- | --- |
| 1. Isometries | `euclidean::isometry_defect` of each pairing | [RAT], Theorem 1.3.5 (Section 12 of the geometry reference) |
| 2. Pairings | As for a hyperbolic domain, with `euclidean::side` | As for a hyperbolic domain |
| 3. Walls | For each side $j$, the largest of: the absolute value of `euclidean::side(covector_j, E)` divided by the last coordinate of $E$ for $E = V_{j-1}$ and $E = V_j$, the largest absolute entry of `pairing_j - euclidean::reflection(covector_j)`, the largest absolute coordinate of `normal_j - (c_1, c_2, 0)` for the covector $c$, and $\lvert c_1^2 + c_2^2 - 1 \rvert$ | A line covector of a line through the endpoints of its side, stored with its unit direction, whose reflection is the pairing (Sections 4.5 and 6.6 of the geometry design) |
| 4. Relators | For each corner $j$, $C = P_j P_{j+1}$, and $C^{n_j}$ by $n_j - 1$ successive multiplications. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. There is no corner frame | Item 4 of Section 4.8 of the domain reference, and the local group $\langle a, b : a^2 = b^2 = (ab)^n = 1 \rangle$ of a corner reflector of order $n$ of [TH], Proposition 13.3.1(iii), p. 308 (Section 9.4 of the notation reference) |
| 5. Angles | The largest $\lvert \theta_j - \pi/n_j \rvert$, with $\theta_j$ of Section 7. The condition on the angle sum of a hyperbolic triangle does not apply | Every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference) |
| 6. Area | $\lvert (m - 2)\pi - \sum_j \theta_j \rvert$, against the value $-2\pi\chi = 0$ of a Euclidean signature | A consistency check only, as below |

Check 6 of a Euclidean domain has the residual 0 on every Euclidean domain of Section 9.4 (Section 8.4), and passes on the triangle of `*442` labelled `*632`, where checks 4 and 5 fail (Section 8.3). The design keeps it so that the six checks exist for every geometry, and Section 15.3 puts that to the owner.

For a spherical domain, `verify()` computes the same six checks with the same tolerances, and reads each one in this way, with indexes modulo $m$:

| Check | Spherical reading | Condition |
| --- | --- | --- |
| 1. Isometries | `spherical::isometry_defect` of each pairing | [RAT], Theorem 2.1.3 (Section 11.1 of the geometry reference) |
| 2. Pairings | As for a hyperbolic domain, with `spherical::side`. The endpoint part is the largest absolute coordinate of $P_i E - E$, with no division by a last coordinate | As for a hyperbolic domain |
| 3. Walls | For each side $j$, the largest of: the absolute value of `spherical::side(covector_j, E)` for $E = V_{j-1}$ and $E = V_j$, with no division, the largest absolute entry of `pairing_j - spherical::reflection(normal_j)`, the largest absolute coordinate of `covector_j - normal_j`, and $\lvert \text{normal}_j \cdot \text{normal}_j - 1 \rvert$ | A great-circle normal of a great circle through the endpoints of its side, stored as its own covector, whose reflection is the pairing (Sections 4.6 and 6.7 of the geometry design) |
| 4. Relators | For each corner $j$, $C = P_j P_{j+1}$, and $C^{n_j}$ by $n_j - 1$ successive multiplications. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. There is no corner frame. For a lune this is $(P_0 P_1)^n$ at corner 0 and $(P_1 P_0)^n$ at corner 1 | Item 4 of Section 4.8 of the domain reference, and the local group of a corner reflector of [TH], Proposition 13.3.1(iii), p. 308 (Section 9.4 of the notation reference) |
| 5. Angles | The largest $\lvert \theta_j - \pi/n_j \rvert$, with $\theta_j$ of Section 7, or infinity when the angle sum of a triangle is not above $\pi$. Both angles of a lune come from its normals (DOM-20) | Every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference), and the angle sum of a spherical triangle exceeds $\pi$ ([RAT], Theorem 2.5.1, p. 50) |
| 6. Area | The absolute difference between `area()` and the stored area of the signature, $\pi$ times `area_over_pi()`, which is $2\pi\chi$ | The areas of a triangle and of a lune of Section 7, and the area $2\pi\lvert\chi\rvert$ of an elliptic orbifold of [TH], p. 312 (Section 7.3 of the notation reference) |

DOM-21. For a spherical domain, the precondition of checks 2 to 6 MUST require each vertex to be finite and not the zero vector, and MUST NOT require a positive last coordinate, because a vertex of a spherical domain may lie on or below the plane $x_3 = 0$, as both vertices of a lune do, and the precondition of the other geometries would give every lune an infinite residual.

`verify()` of a spherical domain costs no more products than the bound above. At the order limit, a lune needs $2 \cdot 999$ products for its two relators.

### 8.3. Defects Each Check Detects

The test "Failing domains" of Section 12.1 injects each defect of this table into a domain built through private access, and asserts that exactly the checks of its row fail and that `passed()` is false. A defect without named triangles is applied to the domains of $(7, 3, 2)$, $(4, 4, 4)$, and $(1000, 3, 2)$. A tilt is a rotation of the normal about the origin by `hyperbolic::rotation`:

| Defect | Checks that fail |
| --- | --- |
| The pairing of side 0 scaled by $1 + 10^{-6}$ | 1, 2, and 3 |
| The partner of side 0 set to side 1 | 2 |
| The partner of side 2 set to 3, outside the range of the sides | 2 |
| Sides 0 and 1 swapped | 2, 3, and 4, and at $(4, 4, 4)$ only 2 and 3 |
| The normal of side 0 tilted by $10^{-6}$, with its covector and pairing computed from the tilted normal | 2, 3, and 4 |
| The normal and covector of side 0 tilted by $10^{-3}$, with the pairing left as constructed | 3 |
| The triangle $(7, 3, 3)$ labelled `*732` | 4, 5, and 6 |
| The triangle $(1001, 3, 2)$ labelled `*(1000)32` | 4, 5, and 6 |
| The triangle with the order of one corner raised by 1, labelled with the orders $(7, 3, 2)$ or $(4, 4, 4)$, for each of the three corners, such as $(7, 4, 2)$ labelled `*732` | 4, 5, and 6 |
| The triangle $(6, 4, 3)$ labelled with the orders $(6, 3, 2)$ of `*632` and its Euclidean area 0 | 4, 5, and 6 |
| Vertex 0 negated, which puts it on the other sheet of the hyperboloid | 2, 3, 4, 5, and 6, each with an infinite residual |
| A NaN in the first coordinate of vertex 1 | 2, 3, 4, 5, and 6, each with an infinite residual |
| A NaN in the normal of side 1 | 3, with an infinite residual |
| A NaN in one entry of the pairing of side 2 | 1, 2, 3, and 4, each with an infinite residual |
| The last vertex removed, on $(7, 3, 2)$ only | 2, 3, 4, 5, and 6, each with an infinite residual |
| Every vertex removed, on $(7, 3, 2)$ only | 2, 3, 4, 5, and 6, each with an infinite residual |

The test also injects these defects into Euclidean domains, which pass every check before the defect: the relabelled triangle is that of `*442`, and the other defects are applied to the domain of `*632` and to the rectangle of `*2222` at `aspect` 1. On each domain of its row, exactly the checks of the row fail:

| Defect | Checks that fail |
| --- | --- |
| The triangle of `*442` labelled with the orders of `*632` | 4 and 5. Check 6 passes |
| The offset $t$ of the covector of side 0 moved by $10^{-6}$, with the pairing left as constructed | 3 |
| The offset $t$ of side 0 moved by $10^{-6}$, with the pairing computed from the moved covector | 2 and 3 |
| The pairing of side 0 scaled by $1 + 10^{-6}$ | 1, 2, and 3. Check 4 passes |
| The partner of side 0 set to side 1 | 2 |
| A NaN in the first coordinate of vertex 1 | 2, 3, 4, 5, and 6, each with an infinite residual |
| The last row of the pairing of side 0 set to $(10^{-6}, 0, 1)$ | 1, 2, and 3 |
| The rectangle of `*2222` labelled with the three orders $(6, 3, 2)$, on the rectangle only | 2, 3, 4, 5, and 6, each with an infinite residual |
| The last vertex removed | 2, 3, 4, 5, and 6, each with an infinite residual |
| Every vertex removed | 2, 3, 4, 5, and 6, each with an infinite residual |

The test also injects these defects into spherical domains, which pass every check before the defect: the domains of `*532`, `*722`, and `*55`, unless the row names others. The column gives the check that each defect targets. The change that adds the test records in this table the checks that fail on each domain, as the tables above record them, and the test then asserts exactly those:

| Defect | Check that it targets |
| --- | --- |
| The normal of side 0 tilted by $10^{-6}$ about the origin, with its covector and pairing computed from the tilted normal | 2 and 3 |
| The pairing of side 0 scaled by $1 + 10^{-6}$ | 1, 2, and 3 |
| The triangle of `*432` labelled with the orders of `*532`, on `*532` only | 4, 5, and 6 |
| The lune of the angle $2\pi/n$ labelled with the order $n$, for $n = 5$ and $n = 12$, on the lunes only | 5 and 6, while check 4 is expected to pass |
| The partner of side 0 set to side 1 | 2 |
| The centre set to its negation, outside the domain | 2 |
| Vertex 0 set to the zero vector | 2, 3, 4, 5, and 6, each with an infinite residual |

Check 4 passes on the scaled pairing at all three triangles, below its tolerance of $2 \times 10^{-6}$, and checks 1, 2, and 3 catch that defect. The residual of check 4 is $1.000003 \times 10^{-6}$ at $(7, 3, 2)$, a figure that no committed test prints.

Check 3 exists for the row with the pairing left as constructed. The renderer tests a point with the covector and moves it with the pairing, so a domain whose covector and pairing disagree corrupts the image while the other five checks pass. The tilted wall no longer passes through the endpoints of its side, so the first two values of check 3 exceed its tolerance.

The tolerances separate an order from its neighbour at the limit. Two measurements outside the repository, which no test repeats, give these figures. At $(1001, 3, 2)$ labelled `*(1000)32`, the residual of check 4 is $6.277 \times 10^{-6}$, which is 3.14 times its tolerance, and the residuals of checks 5 and 6 are each $3.138 \times 10^{-6}$. The second measurement covers every triangle near the limit with one corner order off by one:

- Labels: every hyperbolic $(n_0, n_1, n_2)$ with $n_0$ from 900 to 1000, $n_0 \ge n_1 \ge n_2$, and $n_1$ and $n_2$ from the set $\{2, 3, 4, 5, 6, 7, 8, 10, 12, 20, 30, 50, 100, 200, 300, 500, 700, 900, 950, 970, 980, 990, 995, 998, 999, 1000\}$.
- Triangles: for each label, the triangle with one of its orders raised by 1, which gives 19,728 cases for each corner.

Every case fails checks 4, 5, and 6:

| Raised order | Smallest residual of check 4 over its tolerance | Where | Smallest residual of check 5 or 6 over its tolerance |
| --- | --- | --- | --- |
| Corner 0 | 3.094 | $(999, 970, 950)$ labelled `*(998)(970)(950)` | 3138.5 |
| Corner 1 | 3.110 | $(1000, 999, 950)$ labelled `*(1000)(998)(950)` | 3138.5 |
| Corner 2 | 3.138 | $(1000, 1000, 1001)$ labelled `*(1000)(1000)(1000)` | 3138.5 |

### 8.4. Tolerances

DOM-9. Each check MUST use the tolerance of this table, because `escher info --check` prints the tolerance with a failed check, and the margins below are what make a pass meaningful at every supported order.

| Check | Tolerance | Largest residual | Where | Margin |
| --- | --- | --- | --- | --- |
| 1. Isometries | $10^{-9}$ | $1.819 \times 10^{-11}$ | $(868, 4, 4)$ | 55.0 |
| 2. Pairings | $10^{-9}$ | $1.819 \times 10^{-11}$ | $(868, 4, 4)$ | 55.0 |
| 3. Walls | $10^{-9}$ | $5.203 \times 10^{-15}$ | $(923, 6, 5)$ | $1.92 \times 10^{5}$ |
| 4. Relators | $2 \times 10^{-6}$ | $1.338 \times 10^{-7}$ | $(998, 981, 961)$ | 15.0 |
| 5. Angles | $10^{-9}$ | $5.496 \times 10^{-13}$ | $(998, 998, 958)$ | 1820 |
| 6. Area | $10^{-9}$ | $9.730 \times 10^{-13}$ | $(997, 931, 906)$ | 1028 |

Each figure is the largest residual over every supported triple, which is every $(n_0, n_1, n_2)$ with $1000 \ge n_0 \ge n_1 \ge n_2 \ge 2$ and $1/n_0 + 1/n_1 + 1/n_2 < 1$, 166,665,495 triples. A sweep outside the repository computes them with the code of this crate built in release mode, and no supported triple fails a check. No test in the repository repeats that sweep. The test "Checks at the worst cases" of Section 12.1 runs the six checks on the 294 triples of its set, which include every triple of this table, and prints the same largest residuals, triples, and margins. The margin is the tolerance divided by the largest residual. Every margin is at least 8, as TEST-13 of the coding standard requires.

On a built domain, the last two values of check 3 are 0, because the construction stores the pairing of a mirror as `hyperbolic::reflection(normal)` and the covector as `hyperbolic::covector(normal)`. Its residual is therefore the largest of its first two values, the side test of a vertex against its wall divided by the last coordinate of the vertex. Section 9.3 states how the accuracy rows of the geometry design relate to the inputs of this crate.

The tolerances are choices of this design, and Section 15.3 asks the owner about that of check 4. The tolerance of check 4 is $2 \times 10^{-6}$, because the largest residual would leave a margin of 7.5 at $10^{-6}$, below the 8 that TEST-13 of the coding standard requires. At $2 \times 10^{-6}$ the margin is 15.0, and check 4 still fails every triangle near the limit with one corner order off by one, by at least 3.09 times its tolerance (Section 8.3). A tolerance of $10^{-5}$ would no longer separate the order 1000 from 1001 by check 4 alone, because its residual at $(1001, 3, 2)$ labelled `*(1000)32` is $6.277 \times 10^{-6}$.

The tolerance of check 3 stays at $10^{-9}$, the tolerance of checks 1, 2, 5, and 6. A tolerance of $10^{-12}$ would still leave a margin of 192 and would detect a smaller tilt of a wall. This design keeps $10^{-9}$, because one tolerance for these five checks keeps the output of `escher info --check` uniform, and its margin of $1.92 \times 10^{5}$ already exceeds that of every other check.

A Euclidean domain uses the same tolerances. The test "Euclidean domains" of Section 12.1 prints the largest residual of each check over the 72 Euclidean domains of Section 9.4, and every domain passes every check:

| Check | Tolerance | Largest residual | Where | Margin |
| --- | --- | --- | --- | --- |
| 1. Isometries | $10^{-9}$ | $4.441 \times 10^{-16}$ | `*442` | $2.25 \times 10^{6}$ |
| 2. Pairings | $10^{-9}$ | $4.441 \times 10^{-16}$ | `*442` | $2.25 \times 10^{6}$ |
| 3. Walls | $10^{-9}$ | $2.220 \times 10^{-16}$ | `*442` | $4.50 \times 10^{6}$ |
| 4. Relators | $2 \times 10^{-6}$ | $3.146 \times 10^{-16}$ | `*632` | $6.36 \times 10^{9}$ |
| 5. Angles | $10^{-9}$ | $2.220 \times 10^{-16}$ | `*333` | $4.50 \times 10^{6}$ |
| 6. Area | $10^{-9}$ | 0 | Every domain | Not applicable |

A spherical domain uses the same tolerances. The test "Spherical domains" of Section 12.1 prints the largest residual of each check over the spherical domains of Section 9.5, and every domain passes every check. The change that adds the test records each figure in this table, with a margin of at least 8 as TEST-13 of the coding standard requires:

| Check | Tolerance | Largest residual | Where | Margin |
| --- | --- | --- | --- | --- |
| 1. Isometries | $10^{-9}$ | To be measured | To be measured | To be measured |
| 2. Pairings | $10^{-9}$ | To be measured | To be measured | To be measured |
| 3. Walls | $10^{-9}$ | To be measured | To be measured | To be measured |
| 4. Relators | $2 \times 10^{-6}$ | To be measured | To be measured | To be measured |
| 5. Angles | $10^{-9}$ | To be measured | To be measured | To be measured |
| 6. Area | $10^{-9}$ | To be measured | To be measured | To be measured |

## 9. Numerical Robustness

### 9.1. Renormalization

- `hyperbolic::normalize_point` runs twice, in steps 6 and 10 of Section 6.1, each time on a sum of vertices.
- `hyperbolic::wall_through` divides each normal by its norm, and `euclidean::wall_through` divides each covector by the length of its first two coordinates (the geometry design).
- The Euclidean construction divides a sum of vertices by its last coordinate in steps 3 and 5 of Section 6.5.
- `spherical::normalize_point` runs twice for a spherical triangle, in steps 4 and 6 of Section 6.7, each time on a sum of vertices, and `spherical::wall_through` divides each normal by its length. The lune renormalizes nothing.
- `verify()` renormalizes nothing (DOM-8).

DOM-10. The construction MUST NOT renormalize the vertices after step 8, or after step 4 of Section 6.5, because a renormalized vertex lies on the hyperboloid but far from the true corner. Section 14 records the figures for $(1000, 1000, 1000)$, which no measurement of the code covers.

A vertex therefore leaves the hyperboloid by a small amount. Over the triples of the reference sweep of Section 9.3, $\lvert \langle V, V \rangle + 1 \rvert$ reaches $4.113 \times 10^{-5}$, at $(995, 987, 976)$. The wall through two vertices depends only on the plane that they span, so the normals are not affected.

### 9.2. Forms

- `hyperbolic::inverse` computes the geometric inverse $J M^t J$ (Section 6.1 of the geometry reference), never a general matrix inverse.
- The relators are evaluated in the corner frame (Section 8.2, check 4).
- The corner angles come from the distances between the vertices (Section 7).
- $\operatorname{arcosh} X$ in step 4 needs no special form. The smallest $X$ that the construction computes over the triples of the reference sweep of Section 9.3 is 1.040349, at `*732`.
- $\arccos X$ in step 2 of the spherical triangle is clamped to $[-1, 1]$. For `*n22` with a large $n$, $X_b$ comes close to 1, where the arccosine is ill-conditioned, and $\cos(\pi/2)$ is not 0 in `f64`, so Section 9.5 measures the triangle up to $n = 1000$.

### 9.3. Accuracy over the Supported Range

DOM-11. The values that `build_with` returns MUST lie within the bounds of this table, or for a Euclidean domain of Section 9.4 and a spherical domain of Section 9.5, of the values of the exact polygon in the final position, because the renderer and `escher info` use them without a check, and the reference-arithmetic tests of Section 12.1 assert these bounds.

The rows for the side length, the corner angle, and the area bound the values that `side_lengths()`, `corner_angles()`, and `area()` return, which the crate measures from the stored vertices (Section 7), with lengths and areas at curvature $-1$ and angles in radians.

| Quantity | Measure | Orders up to 12 | $(1000, 3, 2)$ | Every supported triple |
| --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error divided by the largest coordinate | $10^{-13}$ | $10^{-11}$ | $10^{-8}$ |
| Normal | Largest coordinate error | $10^{-13}$ | $10^{-11}$ | $10^{-8}$ |
| Side length | Absolute error | $10^{-13}$ | $10^{-11}$ | $10^{-8}$ |
| Centre | Largest coordinate distance from `hyperbolic::ORIGIN` | $10^{-13}$ | $10^{-11}$ | $10^{-9}$ |
| Corner angle | Absolute error | $10^{-9}$ | $10^{-9}$ | $10^{-9}$ |
| Area | Absolute error | $10^{-9}$ | $10^{-9}$ | $10^{-9}$ |

The bounds are choices of this design. The test "Triangles" of Section 12.1 measures each quantity against the reference arithmetic of Section 12.2, asserts each bound, and prints the largest error with its triple and its margin. The test "Third side" measures the distance from vertex 0 to vertex 1, which the construction never sets, against the bounds of the side length. The error of the area is measured against the area of the reference triangle. The column for orders up to 12 covers all 269 hyperbolic triples with orders up to 12. The column for every supported triple covers the 294 triples of the set of Section 12.1, which are those 269 and the 25 named triples with an order above 12. The margin is the bound divided by the largest error. Every margin is at least 8, as TEST-13 of the coding standard requires:

| Quantity | Orders up to 12: largest error, where, margin | $(1000, 3, 2)$: largest error, margin | Set of 294 triples: largest error, where, margin |
| --- | --- | --- | --- |
| Vertex | $9.104 \times 10^{-15}$, $(11, 10, 10)$, 11.0 | $1.241 \times 10^{-14}$, 806 | $1.573 \times 10^{-10}$, $(997, 931, 906)$, 63.6 |
| Normal | $1.139 \times 10^{-14}$, $(11, 10, 10)$, 8.8 | $5.932 \times 10^{-15}$, 1686 | $1.806 \times 10^{-10}$, $(998, 998, 958)$, 55.4 |
| Side length | $1.233 \times 10^{-14}$, $(11, 10, 10)$, 8.1 | $1.723 \times 10^{-14}$, 580 | $2.016 \times 10^{-10}$, $(978, 977, 971)$, 49.6 |
| Centre | $3.057 \times 10^{-15}$, $(12, 9, 8)$, 32.7 | $2.872 \times 10^{-15}$, 3482 | $1.018 \times 10^{-10}$, $(973, 972, 964)$, 9.8 |
| Corner angle | $2.985 \times 10^{-15}$, $(11, 10, 10)$, $3.35 \times 10^{5}$ | $8.564 \times 10^{-14}$, $1.17 \times 10^{4}$ | $5.496 \times 10^{-13}$, $(998, 998, 958)$, 1820 |
| Area | $4.655 \times 10^{-15}$, $(12, 8, 8)$, $2.15 \times 10^{5}$ | $6.864 \times 10^{-14}$, $1.46 \times 10^{4}$ | $9.732 \times 10^{-13}$, $(997, 931, 906)$, 1028 |
| Third side | $1.233 \times 10^{-14}$, $(11, 10, 10)$, 8.1 | Not measured separately | $2.016 \times 10^{-10}$, $(978, 977, 971)$, 49.6 |

Two sweeps outside the repository extend these figures, and no test in the repository repeats them:

- The reference sweep compares every quantity of the table with the same reference arithmetic on 63,123 distinct triples, and finds no error above the figures of the set of 294 triples. Its largest vertex error is $1.544 \times 10^{-10}$, at $(992, 991, 975)$, and its largest normal error $1.625 \times 10^{-10}$, at $(989, 976, 974)$. Its triples are the union of five sets:
  - every hyperbolic triple with orders up to 30
  - every triple with all three orders from 970 to 1000
  - every triple with $n_0$ from 900 to 1000 and $n_1$ and $n_2$ up to 30
  - every hyperbolic triple of orders from the grid $\{2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 30, 40, 50, 75, 100, 150, 200, 250, 300, 400, 500, 600, 700, 800, 900, 950, 1000\}$
  - the families $(n, 3, 2)$, $(n, 4, 4)$, $(n, n, 2)$, $(n, n, n)$, $(n, 4, 3)$, and $(n, 7, 6)$ for $n$ up to 1000
- The sweep of Section 8.4 over every supported triple needs no reference for the centre, whose exact value is `hyperbolic::ORIGIN`. Its largest centre offset is the figure of the set of 294 triples, $1.018 \times 10^{-10}$ at $(973, 972, 964)$. It measures the angles and the area only through checks 5 and 6, which compare with $\pi/n_j$ and with the area of the signature that the domain stores (Section 8.2).

No measurement covers the vertices, normals, and side lengths of every supported triple. The column "Every supported triple" bounds them on the strength of the set of 294 triples and the reference sweep.

Over the triples of the reference sweep, the vertices of this crate lie up to distance 6.600 from `hyperbolic::ORIGIN`, at $(1000, 1000, 1000)$, and Section 9.1 states how far they leave the hyperboloid. Section 8.4 of the geometry design states that every result of `hyperbolic::side`, `hyperbolic::distance`, `hyperbolic::wall_through`, `hyperbolic::translation`, and `hyperbolic::reflection` on these vertices and the centre was within the bound of its row, although some of these inputs lie outside the ranges of the rows. It states that its tests do not measure `hyperbolic::normalize_point` on the sums of vertices of steps 6 and 10, or `hyperbolic::isometry_defect` on the pairings. The centre and check 1 therefore rest only on the end-to-end measurements of this section and of Section 8.4, and the bounds above keep the end-to-end figures and margins for every quantity.

### 9.4. Accuracy of the Euclidean Domains

The Euclidean domains of the tests are those of `*632`, `*442`, and `*333`, and the rectangle of `*2222` at `aspect` 1, 1.618, 4, 0.125, and 8, and at 64 values spaced logarithmically from 0.125 to 8. Over these domains, the test "Euclidean domains" of Section 12.1 compares each quantity of this table except the last with the reference construction of Section 12.2, asserts each bound, and prints the largest error with its domain and its margin. The 72 domains have 285 sides. The bounds are choices of this design, and every margin is at least 8, as TEST-13 of the coding standard requires:

| Quantity | Measure | Bound | Largest error | Where | Margin |
| --- | --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error | $2 \times 10^{-15}$ | $1.290 \times 10^{-16}$ | `*2222` at `aspect` 0.17388 | 15.5 |
| Covector | Largest coordinate error | $2 \times 10^{-15}$ | $1.290 \times 10^{-16}$ | `*2222` at `aspect` 0.17388 | 15.5 |
| Pairing | Largest entry error | $4 \times 10^{-15}$ | $2.581 \times 10^{-16}$ | `*2222` at `aspect` 0.17388 | 15.5 |
| Side length | Absolute error | $4 \times 10^{-15}$ | $2.581 \times 10^{-16}$ | `*2222` at `aspect` 0.17388 | 15.5 |
| Corner angle | Absolute error | $2 \times 10^{-15}$ | $1.684 \times 10^{-16}$ | `*632` | 11.9 |
| Area | Absolute error, against the area of the reference polygon | $2 \times 10^{-15}$ | $1.110 \times 10^{-16}$ | `*2222` at `aspect` 0.15238 | 18.0 |
| Centre | Largest coordinate distance from `euclidean::ORIGIN` | $10^{-15}$ | $5.551 \times 10^{-17}$ | `*632` | 18.0 |
| Aspect of the rectangle | Relative error of the length of side 0 divided by that of side 1, against `aspect` | $4 \times 10^{-15}$ | $2.220 \times 10^{-16}$ | `*2222` at `aspect` 1.618 | 18.0 |
| SymmHub lines | Largest coordinate error of each wall of a triangle against its line of SymmHub, up to sign (Section 12.1) | $2 \times 10^{-15}$ | $1.110 \times 10^{-16}$ | Side 2 of `*333` | 18.0 |

The test "SymmHub lines" measures the last row, and the three lines of SymmHub match sides 0, 2, and 1, in that order, for each of the three triangles.

At `aspect` 1 and 4, every number of the construction of the rectangle is a dyadic rational and every operation is exact in IEEE 754 arithmetic, and the square roots of 1 and 4 are exact. The test "Exact rectangle" therefore asserts exactly that the vertices are $(\pm w/2, \pm h/2, 1)$ and the covectors are $(\pm 1, 0, -w/2)$ and $(0, \pm 1, -h/2)$, as TEST-13 of the coding standard allows.

DOM-19. A test that asserts a quantity of a domain for which this document states no measured bound MUST set its bound and record it in this document as TEST-13 of the coding standard requires, over an input set that this document states, in the same change as the test, because Sections 8.4 and 9.3 to 9.5 are the one record of the tolerances of this crate, and a bound outside them has no measurement for a reviewer to check.

### 9.5. Accuracy of the Spherical Domains

The spherical domains of the tests are those of `*332`, `*432`, `*532`, and `*222`, and of `*n22` and `*nn` for $n$ = 2, 3, 5, 12, 100, 999, and 1000. Over these domains, the test "Spherical domains" of Section 12.1 compares each quantity of this table with the reference construction of Section 12.2, asserts each bound, and prints the largest error with its domain and its margin. Each bound is set by the change that adds the test, from the largest error that it measures, with a margin of at least 8 as TEST-13 of the coding standard and DOM-19 require, and that change records it here:

| Quantity | Measure | Bound | Largest error | Where | Margin |
| --- | --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error | To be measured | To be measured | To be measured | To be measured |
| Normal | Largest coordinate error | To be measured | To be measured | To be measured | To be measured |
| Side length | Absolute error | To be measured | To be measured | To be measured | To be measured |
| Centre | Largest coordinate error against the reference centre | To be measured | To be measured | To be measured | To be measured |
| Corner angle | Absolute error against $\pi/n_j$ | To be measured | To be measured | To be measured | To be measured |
| Area | Absolute error against $2\pi\chi$ | To be measured | To be measured | To be measured | To be measured |

The vertices and the centre of a lune are exact values that the construction stores, and the dot product of its two vertices is exactly $-1$. The test asserts these equalities exactly, as TEST-13 of the coding standard allows, and compares the side lengths of a lune with $\pi$ within the bound of the side length.

## 10. Errors

### 10.1. Structure

DOM-12. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"build fundamental domain"` for every error of this crate.
- `detail`: A private enum with one variant for each cause, which carries the data that the text of Section 10.2 shows: the canonical string of the signature, the order over the limit, the identity of a parameter from which its name and range derive, or the error of `escher-geometry` that caused a numerical failure.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, because a derived `source()` can return only a field of `Error`, not the `escher-geometry` error that one variant of the detail holds, and a separate source field beside the detail would hold one fact in two fields. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `escher-geometry` error for a `Numerical` error that one caused, and `None` otherwise. No type from a dependency appears in the public surface of the error (ERR-2).

`Numerical` is an error and not a panic. No supported signature is known to produce it, but the construction calls fallible functions of `escher-geometry`, and a panic would stop `escher info`, so returning an error is the sounder option (ERR-5 of the coding standard, Hard Rule 7 of `AGENTS.md`).

### 10.2. Kinds and Text

DOM-13. `Display` for `Error` MUST be `cannot build fundamental domain: ` followed by the text of the detail in this table, because `escher-cli` prints it and its snapshots fix it.

| Kind | When | Detail carries | Text after the prefix |
| --- | --- | --- | --- |
| `UnsupportedSignature` | The predicate of Section 4 fails, which includes every general non-compact signature (`DEC-19`) | The canonical string | `the signature 532 is not supported`, with the canonical string of the signature in place of `532` |
| `LimitExceeded` | A corner order of a supported signature exceeds 1000 | The largest corner order | `order 1001 is larger than 1000`, with the largest corner order in place of `1001` |
| `Numerical` | Step 11 of Section 6.1, step 7 of Section 6.5, or step 7 of the triangle or step 4 of the lune of Section 6.7 | The `escher-geometry` error, if one caused it | `the construction is numerically degenerate` |
| `UnknownParameter` | Step 3 of DOM-18 | The canonical string | `the signature *632 has no shape parameter with one of the given names`, with the canonical string of the signature in place of `*632` |
| `ParameterOutOfRange` | Step 4 of DOM-18 | The identity of the parameter, a private enum from which the name, the minimum, and the maximum of its `ParameterSpec` derive, so that `Error` keeps `Eq` | `the parameter aspect is outside its range of 0.125 to 8`, with the name, the minimum, and the maximum in place of `aspect`, `0.125`, and `8` |

The texts of `UnknownParameter` and `ParameterOutOfRange` contain no name or value of a setting, because a caller's text may hold any character, and the printed name is that of the `ParameterSpec`. They write each number with the `{}` format of `f64`, which prints the two ends of the range of `aspect` as `0.125` and `8`.

So `FundamentalDomain::build` on `532` returns an `Error` whose `kind()` is `UnsupportedSignature` and whose `Display` text is exactly `cannot build fundamental domain: the signature 532 is not supported`. On `*(1001)32` it returns `LimitExceeded` with the text `cannot build fundamental domain: order 1001 is larger than 1000`. What `escher-cli` prints for either, and its exit status, are for the command-line design to state.

Every fixed text is in lower case and has no trailing punctuation (ERR-4 of the coding standard). The canonical string contains only printable ASCII (Section 7.3 of the signature design), so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

## 11. Resource Limits

The crate limits the largest corner order, which `build_with` checks at step 2 of DOM-18, and the value of each shape parameter, which it checks at step 4. Section 16 states the value and its reason, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 12. Testing

### 12.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-domain/tests/domain/main.rs`, with the modules `common`, `reference`, `triangles`, `regular`, `properties`, `unsupported`, `euclidean`, `parameters`, and `spherical` (TEST-10). Its root has a `//!` line. The snapshot tests of the domain summary live in `escher-cli`, and the command-line design specifies them.

The module `common` defines the sets of triples that the tests use:

- The plan triples: $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(1000, 3, 2)$. The plan writes $(3, 3, 4)$, whose canonical signature is `*433`.
- The 24 worst cases: $(995, 983, 970)$, $(994, 991, 970)$, $(997, 996, 970)$, $(997, 980, 973)$, $(992, 990, 979)$, $(995, 987, 976)$, $(928, 4, 3)$, $(999, 7, 6)$, $(998, 4, 4)$, $(1000, 4, 4)$, $(1000, 1000, 2)$, $(1000, 1000, 1000)$, $(868, 4, 4)$, $(923, 6, 5)$, $(998, 981, 961)$, $(998, 998, 958)$, $(997, 931, 906)$, $(992, 991, 975)$, $(989, 976, 974)$, $(978, 977, 971)$, $(985, 985, 971)$, $(973, 972, 964)$, $(979, 979, 971)$, and $(987, 984, 970)$. They include every triple at which the sweeps of Sections 8.4 and 9.3 have a largest value.
- The 30 named triples: the plan triples and the worst cases.
- The orders up to 12: all 269 hyperbolic triples $n_0 \ge n_1 \ge n_2$ with orders up to 12.
- The set of 294 triples: the orders up to 12 and the 25 named triples with an order above 12.

Each test names the item of the field "Tests and acceptance" of Section 5.2 of the plan that it serves, or of Section 5.4 or 5.5 of the plan where the item says so. The tests "Triangles", "Third side", "Checks at the worst cases", "Relator agreement", and the two tests of $\{7, 3\}$ also assert their smallest number of samples, and print each largest value with its triple and its margin. The test harness captures that output for a test that passes:

| Test | Plan item | Kind and place | Asserts |
| --- | --- | --- | --- |
| Triangles | Reference arithmetic | Reference test, `triangles` | Vertices, normals, side lengths, the centre, corner angles, and the area are within the bounds of each column of Section 9.3, against the reference of Section 12.2, for the set of 294 triples |
| Third side | Reference arithmetic | Reference test, `triangles` | For the set of 294 triples: in the reference, the side from vertex 0 to vertex 1 of the standard position agrees within $10^{-60}$ with the side opposite vertex 2 by the law of cosines of Section 10.2 of the geometry reference, which checks the placement. The distance from vertex 0 to vertex 1 of the built domain, which the construction never sets, is within the bounds of the side length of Section 9.3 for the orders up to 12 and for the set |
| Checks at the worst cases | Reference arithmetic | Integration test, `triangles` | For the set of 294 triples: each check of Section 8.2 passes and `verify().passed()` is true. It prints the largest residual of each check, which Section 8.4 states |
| Edge of $\{7, 3\}$ | Numeric checks for $\{7, 3\}$ | Reference test, `regular` | Twice the distance from vertex 1 to vertex 2 of `*732` is within $2 \times 10^{-13}$ of equation 29 of [HT] at $(p, q) = (7, 3)$ |
| Circumradius of $\{7, 3\}$ | Numeric checks for $\{7, 3\}$ | Reference test, `regular` | The length of `hyperbolic::to_disk(hyperbolic::inverse(hyperbolic::translation(V0)) * V1)` of `*732` is within $10^{-13}$ of equation 24 of [HT] at $(p, q) = (7, 3)$ |
| Random triples | Properties | `proptest`, `properties` | For orders from 2 to 1000 with $1/p + 1/q + 1/r < 1$, drawn with half of the cases from orders 900 to 1000: `build` succeeds, `verify().passed()`, the vertices are counterclockwise, every vertex is inside or on every side within the tolerance of check 3, and the centre is within $10^{-9}$ of `hyperbolic::ORIGIN` |
| Relator agreement | Properties | Reference test, `properties` | For the 30 named triples, with the residual of each frame computed in the reference from the exact power of the `f64` matrices and divided by $n_j$: the residual of check 4 is within $10^{-9}$ of the largest residual in the corner frame, and at each corner the residual in either frame is at most 2.0 times the residual in the other frame times 9 times the square of the largest absolute entry of $F$. This tests the assumption on conjugates of Section 15.1. Measured: $1.459 \times 10^{-11}$ against $10^{-9}$, a margin of 68.5, at $(1000, 1000, 2)$, and 0.217 against 2.0, a margin of 9.2, at corner 1 of $(1000, 1000, 2)$ |
| Relators in the domain frame | None | Reference test, `properties` | The figures of the table of Section 8.2. At $(998, 981, 961)$, the largest exact residual is at least $2 \times 10^{-6}$ in the domain frame and at most $2 \times 10^{-6}$ in the corner frame. At $(1000, 3, 2)$, it is at most $10^{-10}$ in the domain frame and at most $10^{-12}$ in the corner frame. It prints both residuals for both triples |
| Failing domains | Failure paths | Unit tests in `verify.rs` | The domains of $(7, 3, 2)$, $(4, 4, 4)$, and $(1000, 3, 2)$, and the spherical domains of Section 8.3, built through private access, pass every check. For each defect of Section 8.3, on the domains of its row, exactly the checks of the row fail and `passed()` is false. For a row that names an infinite residual, each failed check has the residual `f64::INFINITY`, and no check has a NaN residual. The rows with a vertex removed and the relabelled rectangle give domains whose numbers of sides, vertices, and corner orders disagree, and `verify()` returns on them without a panic (DOM-8) |
| Check results | Failure paths | Unit tests in `verify.rs` | A `Check` built through private access with a NaN residual fails, a residual equal to the tolerance passes, and an infinite residual fails |
| Unsupported | Failure paths | Table test, `unsupported` | `632`, `*22222`, `532`, `3*2`, `22`, `*3333`, `732`, `4*3`, `*(inf)32`, `*(inf)(inf)`, `(inf)(inf)`, `*22(inf)`, `1`, `*`, `o`, `*732*`, and `x732` return `UnsupportedSignature` with the text of Section 10.2. `*(1001)32`, `*(1001)22`, and `*(1001)(1001)` return `LimitExceeded` with its text, `*(1000)32`, `*(1000)22`, and `*(1000)(1000)` build, and `*(1001)(1001)(1001)(1001)` returns `UnsupportedSignature` |
| Limit | Failure paths | Unit tests in `domain.rs` | With the private limit constant: `*(1000)32` builds and `*(1001)32` returns `LimitExceeded`, and `*(1001)(1001)(1001)(1001)` returns `UnsupportedSignature`, because support is checked before the limit |
| Sides and vertices | None | Unit test in `domain.rs` | For `*732`, each side is its own partner, and `hyperbolic::side` of its covector at each of its two vertices is below $10^{-12}$ in absolute value |
| Corner order and area | None | Unit tests in `triangle.rs` | The private predicate of Section 4 gives the orders $(7, 3, 2)$ and the area `PI / 42.0` for `*237`, the Euclidean triangle with the orders $(6, 3, 2)$ for `*236`, the rectangle for `*2222`, the spherical triangle with the orders $(5, 3, 2)$ and the area `PI / 30.0` for `*532`, the lune of the order 5 for `*55`, and nothing for `532`, `*3333`, `732`, and `*(inf)32`. The area is compared exactly, because the numerator 1 and the denominator 42 convert exactly and multiplying by 1 is exact, so IEEE 754 arithmetic computes the same quotient as `PI / 42.0` (TEST-13 of the coding standard) |
| Conversion | None | Unit test in `triangle.rs` | The private signed conversion of step 1 of Section 6.1 gives exactly 0, $-1$, and $-4294967297$ for those integers, $2^{53}$ for $2^{53} + 1$ by ties to even, $2^{63}$ for `i64::MAX`, and $-2^{63}$ for `i64::MIN`. The unsigned conversion gives exactly 1 and $4294967297$ for those integers, and $2^{64}$ for `u64::MAX`. The test compares exactly, because the sum of the two parts is the only rounding, and IEEE 754 arithmetic fixes the result of that rounding to nearest, bit for bit (TEST-13 of the coding standard) |
| Numerical | Failure paths | Unit test in `triangle.rs` | The private construction of steps 3 to 11, given the spherical orders $(5, 3, 2)$, fails with the `escher-geometry` error of kind `NotAPoint`, which `build` reports as `Numerical`, because the law of cosines then gives a hyperbolic cosine below 1 |
| Any signature | Failure paths | `proptest`, `properties` | For any text that parses as a signature, `build` returns without a panic, and a returned domain passes `verify()` |
| Error text | Failure paths | Unit tests in `error.rs` | The `Display` of every kind starts with `cannot build fundamental domain: `, is in lower case apart from the canonical string, and is the text of Section 10.2. `kind()` matches each cause, and `source()` returns the `escher-geometry` error only for a `Numerical` error that one caused |
| Euclidean table | Relators, of Section 5.4 of the plan | Table test, `euclidean` | `*632`, `*442`, `*333`, and `*2222` build and pass `verify()`. They have 3, 3, 3, and 4 sides and vertices, each side its own partner, and the corner angles of 30, 60, and 90 degrees, 45, 45, and 90 degrees, 60 degrees three times, and 90 degrees four times, within the bound of Section 9.4. Every other spelling of `*632` and `*442`, such as `*236`, builds the domain of its canonical form, with every vertex within twice the vertex bound of Section 9.4 |
| Euclidean domains | Relators, of Section 5.4 of the plan | Reference test, `euclidean` | Each quantity of Section 9.4 other than the lines of SymmHub is within its bound against the reference of Section 12.2, over the Euclidean domains of Section 9.4. Each check passes, and the test prints the largest residual of each check with its margin, which Section 8.4 records (DOM-19) |
| SymmHub lines | None | Reference test, `euclidean` | For each Euclidean triangle, every wall agrees within the bound of Section 9.4, up to sign, with one of the three lines that SymmHub's code gives for the triangle (Section 4.3 of `docs/research/rendering.md`), $y = 0$, $-x \sin a_0 + y \cos a_0 = 0$, and $x \sin a_1 + y \cos a_1 = e$ with $a_0 = \alpha_2$, $a_1 = \alpha_0$, and $e = \sin\alpha_0$, each moved by the translation of step 4 of Section 6.5, and each line matches a different side. It prints which line matches each side. This tests the candidate vertices of `UNV-28` |
| Exact rectangle | None | Unit test in `polygon.rs` | At `aspect` 1 and 4, the vertices and covectors of the rectangle are exactly those of Section 9.4 |
| Rectangle aspect | Properties | `proptest`, `euclidean` | For a value of `aspect` within its range, the rectangle passes `verify()`, has four right angles within the bound of the corner angle of Section 9.4, and the length of side 0 divided by that of side 1 is the value within the bound of the aspect of Section 9.4 |
| Parameters | Range errors, of Section 5.4 of the plan | Table test, `parameters` | `parameters` gives one `ParameterSpec` for `*2222`, with the name `aspect`, the range 0.125 to 8, and the default 1, and an empty list for `*632`, `*442`, `*333`, `*732`, and `*532`. `build_with` returns `ParameterOutOfRange` for `aspect` set to 0.1249, 8.0001, 0, $-1$, NaN, and both infinities, and builds at 0.125 and 8. It returns `UnknownParameter` for the names `Aspect`, `aspect ` with a trailing space, and the empty name on `*2222`, and for `aspect` on `*632` and `*732`. An unsupported signature with a setting returns `UnsupportedSignature`. For `*2222` and `*632`, `build` gives the domain of `build_with` with no setting, with every vertex within twice the vertex bound of Section 9.4, and `set` returns the value that it replaces. Each error has the text of Section 10.2 |
| Spherical table | None | Table test, `spherical` | `*332`, `*432`, `*532`, `*222`, `*722`, `*22`, and `*55` build and pass `verify()`, with 3 sides and vertices for a triangle and 2 for a lune, each side its own partner, `parameters` empty, and the corner angles $\pi/n_j$ within the bound of Section 9.5. Each triangle has $(V_0 \times V_1) \cdot V_2 > 0$. Every other spelling of a signature, such as `*22n` for `*n22` and `*221` for `*22`, builds the domain of its canonical form, with every vertex within twice the vertex bound of Section 9.5 |
| Spherical domains | None | Reference test, `spherical` | Each quantity of Section 9.5 is within its bound against the reference of Section 12.2, over the spherical domains of Section 9.5. Each check passes, and the test prints the largest residual of each check with its margin, which Section 8.4 records (DOM-19) |
| Exhaustive certificate | Exhaustive certificate, of Section 5.5 of the plan | Integration test, `spherical` | For `*332`, `*432`, `*532`, and `*n22` and `*nn` with $n$ from 2 to 12: a breadth-first enumeration of the products of the pairings, which merges two matrices whose entries agree within $10^{-9}$, has exactly as many elements as `group_order()` of the signature. The images of the centre under these elements are pairwise further apart than a floor that the change that adds the test measures and records here. Each image of the centre folds back to the centre within the tolerance of check 3, in as many steps as its breadth-first depth, by a fold written in the test that applies the pairing of the first side with a positive side value, and no such fold reaches the default largest number of folds of `escher-render` (Section 4.1 of `docs/design/render.md`). For each lune, a dense sample of the sphere, which includes the two vertices, points from $10^{-3}$ to $10^{-300}$ from each vertex, the centre and its negation, and points on and next to each side, folds into the lune |
| Exhaustive certificate at the limit | Exhaustive certificate, of Section 5.5 of the plan | Integration test, `spherical` | The assertions of "Exhaustive certificate" for `*(1000)22` and `*(1000)(1000)` |
| Reference self-check | None | Reference test, `reference` | The four known values of Section 12.2 |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs` | Each public type is `Send`, `Sync`, and `'static` (CONC-1 of the coding standard) |
| Doctests | None | Doctests | The examples of the crate-level and item documentation |

DOM-22. The tests "Exhaustive certificate" and "Exhaustive certificate at the limit" MUST cover every lune of their sets and MUST stay while `UNV-29` is open, because no source makes the lune a fundamental domain (Section 6.7), and these tests are the only evidence that its two reflections generate a group of order $2n$ whose copies of the lune cover the sphere.

The claim that a fold returns in its breadth-first depth and never reaches the cap holds for the images of the centre, which the tests assert. It does not hold for every point: near a vertex of very high order, a fold can fail to converge, which belongs to the unverified statement on termination, `UNV-1` (Section 14).

The plan item "Properties" also names the inverse $J M^t J$ and the round trip of the disk map. Those are properties of `escher-geometry`, and the geometry design tests them.

The expected values for $\{7, 3\}$ come from the split of the polygon into $2p$ right triangles (Section 10.1 of the geometry reference), which puts the centre of the polygon at vertex 0 of `*732`, a vertex of the polygon at vertex 1, and the midpoint of an edge at vertex 2. The tests evaluate equations 29 and 24 of [HT] (Section 10.4 of the geometry reference) in the reference arithmetic, with the curvature radius $\ell = 1$. Computed from those equations, the edge length is 0.566256306735315 and the circumradius in the disk is 0.300742618746379. The tests measure an error of $1.548 \times 10^{-16}$ in the edge against its bound of $2 \times 10^{-13}$, and of $2.025 \times 10^{-16}$ in the circumradius against its bound of $10^{-13}$. No value that Table 2 of [HT] prints serves as an expected value, because that table and equation 29 disagree on the edge length of $\{7, 3\}$ (item 23 of Section 15.1 of the geometry reference).

DOM-14. A test MUST NOT take an expected value from Table 2 of [HT], because the research records that the table and equation 29 disagree for $\{7, 3\}$, and a test built on the wrong one passes a wrong domain or fails a correct one.

The property tests commit their regression files (TEST-9 of the coding standard).

### 12.2. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture, with the precision and tolerances that this section states:

- Structure. A module `reference` in the test binary holds points as arrays of three high-precision numbers and writes each formula componentwise from the research documents. It calls no function of `escher-domain`, `escher-geometry`, or `glam`.
- Independence. The reference builds the standard position from $\cosh L$ and $\sqrt{\cosh^2 L - 1}$ without $\operatorname{arcosh}$, and centres the triangle with $\cos\varphi = c_1/h$, $\sin\varphi = c_2/h$, $\sinh\rho = h$, and $\cosh\rho = c_3$, where $h$ is the length of the first two coordinates of the centre, read from the polar form of Section 8.3 of the geometry reference. The crate takes the other path in both places, so a transcription error is not shared. For a Euclidean domain, the reference computes the vertices of steps 1 to 5 of Section 6.5 componentwise, and the line of each side as $P(a, t)$ of Section 12 of the geometry reference, with $a$ the unit vector perpendicular to the side and $t = a \cdot V$ for an endpoint $V$, without a cross product. For a spherical triangle, the reference builds the standard position from $\cos L$ and $\sqrt{1 - \cos^2 L}$ without $\arccos$, centres it with $\cos\rho = c_3$, $\sin\rho = h$, $\cos\varphi = c_1/h$, and $\sin\varphi = c_2/h$ for the reference centre $c$ of unit length, without $\operatorname{atan2}$. The change that adds the test states here how the reference finds each normal without the cross product of `spherical::wall_through`. The reference computes side lengths and angles with the arccosine of the reference type. For a lune, it computes the vertices and normals of Section 6.7 from the reference values of $\cos(\theta/2)$ and $\sin(\theta/2)$.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 10.2 of the geometry design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there, because `astro-float-num` has no public conversion back to `f64`. A message that prints a measured error formats the number as decimal text.
- Self-check. The test "Reference self-check" in `reference` checks the reference library itself: $\pi$ against a literal with 80 digits after the decimal point, $\cos(\pi/3) = 1/2$, $\cosh(\operatorname{arcosh} 2) = 2$, and $\sqrt{2}^2 = 2$, each to $10^{-70}$. The largest of the four errors is $1.7 \times 10^{-77}$. It guards against a wrong precision or rounding argument and does not close `UNV-25`.

DOM-15. The reference module MUST share no code with `escher-domain` or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture).

### 12.3. Tests Under Miri

`DEC-5` and TEST-11 of the coding standard apply to the tests of this crate in this way:

| Tests | Under Miri | Attribute or configuration |
| --- | --- | --- |
| "Triangles", "Third side", "Edge of $\{7, 3\}$", "Circumradius of $\{7, 3\}$", "Relator agreement", "Relators in the domain frame", "Euclidean domains", "SymmHub lines", "Spherical domains", and "Reference self-check" | Ignored | `#[cfg_attr(miri, ignore = "the high-precision arithmetic runs loops too long for the interpreter")]` |
| "Checks at the worst cases" and "Exhaustive certificate at the limit" | Ignored | `#[cfg_attr(miri, ignore = "the checks of the largest orders run loops too long for the interpreter")]` |
| "Exhaustive certificate" | Run unchanged, unless the change that adds it measures that it runs too long for the interpreter. It is then ignored with the attribute of the row above, and that change records it here | None |
| "Random triples", "Rectangle aspect", and "Any signature" | Run with 8 cases and no failure persistence | The helper of DOM-16 |
| The unit tests, which include "Failing domains" with the three tests of a corner order off by one and the tests of a negated vertex and of NaN entries, "Corner order and area", "Conversion", and "Exact rectangle", and the table tests "Unsupported", "Euclidean table", "Parameters", and "Spherical table" | Run unchanged | None |
| The doctests | Not run, because Miri runs the tests through nextest, which runs no doctest | None |

Each reason in an attribute is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

No test asserts a bit-exact result of a transcendental function, because the interpreter need not reproduce the last bits of the host library, and every tolerance of this document leaves room for that. For the same reason, the tests "Euclidean table", "Parameters", and "Spherical table" compare two builds of one domain within twice the vertex bound of Section 9.4 and not bit for bit: under Miri, two calls of a transcendental function on the same input can round differently, so two builds can differ in the last bits.

DOM-16. The property tests MUST take their configuration from one helper in `common`, with the numbers of cases and the failure persistence that Section 4 of `docs/design/tooling.md` states, because the file persistence of `proptest` asks for the current directory, which the isolation of Miri refuses, and a full set of cases takes too long in the interpreter.

## 13. Dependencies

DOM-17. The crate MUST declare only the dependencies of this section, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml`, because each further dependency adds build time, licence obligations, and supply-chain risk, and a version declared twice can diverge (DEP-2 of the coding standard).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-signature` | Normal, with a version on the path dependency (TOOL-25 of `docs/design/tooling.md`) | None | The input type `Signature` and the `Geometry` that `geometry()` returns |
| `escher-geometry` | Normal, with a version on the path dependency (TOOL-25) | None | The linear model (Section 3.1 of the architecture) |
| `glam` | Normal, public | The workspace entry that the geometry design states, which provides `DVec3` and `DMat3` | The types of the public surface (`DEC-4`) |
| `thiserror` | Normal | Default | Derives the `Display` of the private detail of the error (Section 10.1) |
| `proptest` | Dev | The workspace entry | The property tests |
| `astro-float-num` | Dev | `default-features = false`, with `std` | The reference arithmetic (Section 12.2) |

Section 3.1 of the architecture lists `escher-group` as an internal dependency of the crate. The crate declares it only when a function first uses it, as Section 3.3 of the architecture states, because an unused dependency fails `check-unused-deps`.

`glam` types appear in the public signatures of this crate, so DEP-3 of the coding standard makes `glam` a public dependency, with the obligations that Section 3.5 of the architecture states: its version is part of the version of this crate, the crate stays below 1.0 while `glam` does, and a breaking upgrade of `glam` is a breaking change of the crate.

The reference crate departs from `DEC-3`, which names `astro-float`, for the reason and with the resolution facts that Section 11 of the geometry design records. In short, `astro-float` brings `syn` 1 beside the locked `syn` 3 and fails the duplicate check of `cargo deny`, while `astro-float-num` passes. Section 15.3 asks the owner to confirm the change.

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

## 14. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, or a candidate statement of a reference without a label there, and what the code does about it:

| Label | What the code relies on | What the code does |
| --- | --- | --- |
| `UNV-26` | The wall normals of step 9 of Section 6.1, through `hyperbolic::wall_through`, whose formula [RAT] prints with the time coordinate first | The test "Triangles" compares every normal with the reference, and checks 2, 3, and 4 test each wall against the vertices and the pairings |
| `UNV-26` | Nothing from its second part, the relation between the inner product of two normals and the corner angle | Measures the corner angles from the distances between the vertices by the law of cosines (Section 7, DOM-6), so the angle check does not depend on that relation |
| `UNV-25` | The correct rounding of the reference crate, in tests only | Uses 256 bits against tolerances of $10^{-13}$ at the tightest, and the self-check of Section 12.2. One comparison with MPFR closes it (Section 15.3) |
| `UNV-28` | The vertices of the Euclidean triangle of steps 1 and 2 of Section 6.5, the candidate of item 26 of Section 15.1 of the geometry reference, in production | Checks 3 and 5 test each vertex against its walls and each angle against $\pi/n_j$ on every build, and the test "SymmHub lines" compares the walls with SymmHub's three lines |
| `UNV-26` | The Euclidean reflection as the matrix of `euclidean::reflection` (Section 5.3) | Checks 1, 2, and 3 test each pairing as an isometry, an involution that fixes the endpoints of its side, and the reflection of its covector |
| `UNV-10` | Whether the one parameter of `*2222` includes the scale | Fixes the scale by the area 1, asserts no count of parameters, and leaves `deformation_dimension()` as the signature design states (Section 6.6) |
| None | The line through two vertices as a normalized cross product, item 24 of Section 15.1 of the geometry reference, through `euclidean::wall_through` | Check 3 tests each line against the endpoints of its side, and the test "Euclidean domains" compares every covector with the reference |
| None | The area of a polygon from its vertices, item 25 of Section 15.1 of the geometry reference, in `area()` of a Euclidean domain (Section 7) | The test "Euclidean domains" compares the area with that of the reference polygon, which uses the same formula, so the test checks the arithmetic and not the formula. No check of `verify()` reads it |
| `UNV-29` | That the lune of the angle $\pi/n$ is a strict fundamental domain of the group of its two reflections, of order $2n$ (item 9 of Section 13.3 of the domain reference) | The tests "Exhaustive certificate" and "Exhaustive certificate at the limit" enumerate the group, check that the images of the centre are distinct and fold back, and fold a dense sample of the sphere into the lune (DOM-22). Checks 4, 5, and 6 run on every build |
| `UNV-26` | The angle of a lune from its two outward normals (item 28 of Section 15.1 of the geometry reference), in `corner_angles()` and check 5 (DOM-20) | Check 4 tests the same angle from the pairings alone, and the exhaustive certificate tests it through the group order |
| `UNV-1` | The termination of the fold that the test "Exhaustive certificate" writes, for the images of the centre | Asserts it for those images only. On a prototype outside the repository, the fold of `*(1000)22` did not converge within the default cap for some points within $10^{-14}$ of its vertex of order 1000, and four such points cycled under a cap of 100,000 |
| None | That `spherical::translation` maps the origin to the sphere point of its target, in step 4 of the spherical triangle | Section 13.1 of the geometry design records the assumption. The test "Spherical domains" compares every vertex and the centre with the reference, which centres without it |
| None | That the angles of a Euclidean polygon with $m$ vertices sum to $(m - 2)\pi$, which check 6 reads as its residual (Section 8.2). No source read states it for a polygon | Reports the residual as a consistency check only. Section 8.4 records its measured residuals, and Section 8.3 the defects that it detects. A source or a Lean proof would close it |

The hyperbolic construction does not rely on `UNV-28`, because it uses SymmHub's triangle neither in the code nor as a test oracle (Section 6.3). `UNV-4` concerns the precision of the disk map near the limit circle, and the crate maps to the disk only in the test "Circumradius of $\{7, 3\}$", at a point near the origin.

These statements of this design rest on figures that no measurement of the code covers. They come from a double-precision model of the algorithm outside the repository, and each supports only the choice that it names:

| Statement | Section | Figures of the model |
| --- | --- | --- |
| A domain left with vertex 2 at the origin has larger normal components and fails checks that the final position passes | 6.3 | At $(1000, 3, 2)$, a largest normal component of 1 and a relator residual of $7.28 \times 10^{-12}$, which passes. At $(1000, 100, 100)$, 63.6 and $6.23 \times 10^{-4}$, which fails. At $(1000, 1000, 1000)$, 637 and an infinite residual. In the family $(n, n, n)$, $(60, 60, 60)$ and $(70, 70, 70)$ fail checks 1 and 2, $(80, 80, 80)$ also fails check 4, and every larger order fails. The families $(n, 3, 2)$, $(n, 4, 4)$, and $(n, n, 2)$ fail no check up to $n = 1000$ |
| Repeated multiplication evaluates the relator more accurately than repeated squaring | 8.2 | At corner 0 of $(1000, 3, 2)$, a largest entry of $C^{1000} - I$ of $3.46 \times 10^{-9}$ in the domain frame by repeated multiplication and $7.77 \times 10^{-8}$ by repeated squaring, against $2.33 \times 10^{-11}$ in the corner frame by either method |
| The residual of check 6 stays close to that of check 5 | 8.2 | Never more than 2.91 times the residual of check 5, over 300 random triples |
| The tolerances of Section 8.4 hold for the spherical domains with the margin of 8 | 8.4 | On a prototype of the constructions of Section 6.7 outside the repository, every check passed for `*332`, `*432`, `*532`, `*222`, and `*n22` and `*nn` at $n = 2$, 3, 5, 12, 100, 999, and 1000. At $n = 1000$, the smallest margin was $4.9 \times 10^{4}$ for `*n22`, on checks 5 and 6, and $2.5 \times 10^{4}$ for `*nn`, on check 6 |
| The certificate of DOM-22 passes for the lune, which is evidence for `UNV-29` and not a proof of it | 6.7 | On the same prototype, for the orders tested up to 1000, the enumeration found exactly $2n$ elements, every image of the centre folded back in exactly its depth, and a dense sample of the sphere, with the poles and the walls, folded into the lune |
| Renormalizing the vertices after step 8 makes them less accurate (DOM-10) | 9.1 | At $(1000, 1000, 1000)$, renormalization raised the largest relative vertex error from $5.7 \times 10^{-11}$ to $8.9 \times 10^{-6}$ and the angle error from $1.9 \times 10^{-13}$ to $2.8 \times 10^{-8}$ |

## 15. Assumptions, Choices, and Points for the Owner

### 15.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- The triangle placed by the two sides $L_a$ and $L_b$ with the included angle $\alpha_2$ is the triangle with the angles $\alpha_0$, $\alpha_1$, and $\alpha_2$. The research states uniqueness given the three angles (Section 10.1 of the geometry reference), but no statement on two sides and the included angle. The test "Third side" and checks 4 and 5 check it. The same reading applies to the spherical triangle of Section 6.7, which checks 4 and 5 and the test "Spherical domains" check.
- The angle at a vertex is the arccosine of $(\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)$. This is the first law of cosines of Section 10.2 of the geometry reference solved for the angle, which no source prints in this form. The test "Triangles" compares the angles with $\pi/n_j$ in the reference.
- The normalized sum of the vertices lies strictly inside the triangle, hyperbolic or spherical. Check 2 fails if it does not, and the tests "Random triples" and "Spherical table" check it.
- A power of a matrix is the identity exactly when the same power of its conjugate is. This is matrix algebra, not a statement of the geometry reference. The test "Relator agreement" checks it on the 30 named triples.
- Side 0, side 1, and side 2 are the Latin reflections $P$, $Q$, and $R$ of step 3 of Section 9.1 of the notation reference, and its fourth reflection $S$ is $P$ again for a triangle and side 3 for the rectangle. The relators that the checks use come from Section 4.4 of the domain reference and do not depend on this reading.
- `hyperbolic::translation(c)` maps `hyperbolic::ORIGIN` to the point with the spatial coordinates of $c$, as Section 6.4 of the geometry design states, and this is $c$ itself only when $c$ lies on the hyperboloid. Section 13.1 of the geometry design records this property as an assumption and names the test that checks it.

### 15.2. Choices of This Design

No other document dictates these choices:

| Choice | Section |
| --- | --- |
| `FundamentalDomain::build` as an inherent constructor with one argument that returns a `Result` | 3.1, 3.4 |
| The measurements `side_lengths`, `corner_angles`, and `area` as methods of the domain, so that `escher-cli` needs neither `glam` nor `escher-geometry` | 3.1, 7 |
| `Orientation` with both variants, and `PairingKind` with one | 3.2 |
| The domain stores the geometry and the corner orders of its signature, the area of a hyperbolic or spherical signature, and its vertices, sides, and centre, although all of them follow from the signature. They are the result of the construction, which the renderer would otherwise repeat, and the type is immutable, so they cannot diverge. It stores no signature, because `build` borrows it and the checks need only these values. A reviewer checks this against TYPE-3 of the coding standard | 3.2, 6.1 |
| The area of the signature converted to `f64` once, at construction, with no limit on the size of its numerator or denominator | 6.1 |
| Vertex $j$ is corner $j$, between side $j$ and side $j + 1$, and the vertices are counterclockwise | 5.1, 5.2 |
| Vertex 2 at the origin in the standard position, and the centre at the origin in the final position | 6.3 |
| No renormalization of the vertices after centring | 9.1 |
| The largest order of 1000 | 4, 16 |
| `Numerical` as an error kind | 10.1 |
| `Display` and `std::error::Error` of `Error` written by hand | 10.1 |
| Six checks with one accessor each, including check 3 on the walls | 8.1, 8.2 |
| Check 6 kept as a consistency check | 8.2 |
| The relators in the corner frame, by repeated multiplication, with the residual divided by $n_j$ | 8.2 |
| Every tolerance of Sections 8.4, 9.3, and 9.4 | 8.4, 9.3, 9.4 |
| 256 bits for the reference arithmetic | 12.2 |
| The sets of triples of Section 12.1, and the weighting of the test "Random triples" toward large orders | 12.1 |
| `astro-float-num` in place of `astro-float` | 13 |
| The choices of the Euclidean domains and of the shape parameter that Section 15.3 puts to the owner, in items 6 to 12 | 3.4, 6.5, 6.6, 7, 8.2 |
| The choices of the spherical domains that Section 15.3 puts to the owner, in items 13 to 18 | 4, 6.7, 7, 8.2, 8.4, 9.5 |

### 15.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Reference crate. `DEC-3` names `astro-float`. The design uses `astro-float-num` (Section 13). The alternative is a `skip` entry in `deny.toml` for `syn` 1 with a reason. Section 13.3 of the geometry design raises the same point.
2. Largest order and the tolerance of check 4. The design supports corner orders up to 1000 and returns `LimitExceeded` above it (DOM-3). It sets the tolerance of check 4 to $2 \times 10^{-6}$, because the largest residual of that check would leave a margin of 7.5 at $10^{-6}$, below the 8 that TEST-13 of the coding standard requires, and the margin at $2 \times 10^{-6}$ is 15.0, with check 4 still failing every triangle near the limit with one corner order off by one (Sections 8.3 and 8.4). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin.
3. Comparison with MPFR for `UNV-25`. The geometry design records the comparison (its Section 12) and asks whether it closes `UNV-25`. This design relies on the same answer.
4. Check 6. It detects nothing that check 5 does not, apart from a disagreement with `escher-signature` (Section 8.2). The design keeps it because the plan requires an area check. The alternatives are to drop it, or to measure the area by an independent formula from the vertices, which needs a source that the research does not hold.
5. Placement. The final position makes the centre of the canonical view a property of the domain, while Section 3.2 of the architecture places the canonical placement in `View`. `View` can still move the domain, because `centre()` is public.
6. Vertices of the Euclidean triangle. No source gives them, so the design builds the triangle from the candidate of `UNV-28` and certifies each build by checks 3 and 5 and the test "SymmHub lines" (Section 6.5). The alternative is to build only `*2222` until a source or a Lean proof closes the candidate.
7. Units of the Euclidean domains. The triangle has side 0 of length 1, and the rectangle has the area 1 (Section 6.5), so `aspect` and its reciprocal give the same rectangle turned a quarter turn. The alternative for the rectangle is the height 1 and the width `aspect`, which makes the size of a tile depend on `aspect`.
8. Range and default of `aspect`. The design takes 0.125 to 8 with the default 1, the square (Section 6.6). Any other closed range of positive values that contains 1 is an alternative, and no source gives one.
9. The area of a Euclidean domain. `area()` uses a formula that no source states (Section 7, item 25 of Section 15.1 of the geometry reference). The alternative is to give no area for a Euclidean domain, which removes the line `Domain area` that `escher info` prints for it.
10. Check 6 for a Euclidean domain. The design keeps the angle defect, which fails among the defects of Section 8.3 only for a NaN vertex, so that the six checks exist for every geometry (Section 8.2). The alternative is to compare the measured area with the area that the unit of item 7 implies, which would detect a wrong scale but rests on the area formula of item 9.
11. `build_with` beside `build`. The design adds a second constructor that takes the settings, so that the callers of `build` do not change (Section 3.4). The alternative is to give `build` a second argument and change every call.
12. Validation in `escher-domain`. The crate reports an unknown name and a value out of range (DOM-18), and a caller detects only a name set twice, through the return value of `ShapeParameters::set`. The alternative is to validate in `escher-cli`, which would duplicate the ranges.
13. The spherical domains. The design builds `*332`, `*432`, `*532`, `*222`, `*n22`, and the lune of `*nn` with $n$ up to the limit of 1000, which includes `*22`, with no new public item and no shape parameter (Sections 4 and 6.7). The alternatives are to leave `*22` with the other signatures of two corners of order 2 for a later milestone, or a lower limit for the spherical families, whose tiles at $n = 1000$ are smaller than a pixel of a default image.
14. The lune on an unverified statement. No source makes the lune a fundamental domain, so the design builds it on `UNV-29` and certifies it by the tests of DOM-22 (Section 6.7). The alternative is to build no lune until a source or a Lean proof closes the statement, which leaves `*nn` unsupported.
15. The placement of the lune. The design places the lune directly, with its vertices on the second axis and its centre at the origin (Section 6.7), because the rule of `DEC-12` gives no centre for two antipodal vertices. The alternative is a centre at the midpoint of one side, which the view would not centre on the domain.
16. The angle of the lune from its normals. DOM-20 makes it the one exception to DOM-6, because the vertices do not determine the sides. The alternative is to measure no angle for a lune and to drop check 5 for it, which leaves the angle to check 4 and the certificate alone.
17. The readings of the checks and their precondition for a spherical domain. The design keeps the six checks and reads each per geometry (Section 8.2), with a vertex that is finite and not the zero vector (DOM-21). The alternative is a separate set of checks for the sphere, which `escher info --check` would print differently.
18. The tolerances and the limit. The design keeps every tolerance of Section 8.4 and the limit of 1000 for the spherical domains, and leaves the bounds of Section 9.5 to the measurement of the change that adds the tests. The alternative is tolerances of their own for the sphere, which no measurement of the code supports yet.

## 16. Security Considerations

The threat model is a caller that passes a `Signature` parsed from untrusted text, and settings with untrusted names and values, to `FundamentalDomain::build_with`, and then calls `verify()`. `escher-signature` has already rejected malformed text and bounded the signature (Section 16 of the signature design). The attacker can choose any valid signature. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of time or memory through a large order, a panic that stops the caller, and a wrong domain that the caller uses as correct. The crate counters them with the limit of this section, the rule that it never panics (DOM-2 and DOM-8), and the checks of Section 8.

DOM-3 requires the limit of this table. The input is untrusted, the cost of `verify()` grows with the order, and the accuracy of Section 9.3 is established only up to that order:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Largest corner order | 1000 | The construction costs the same for every order. `verify()` costs at most about 3000 matrix products at this limit (Section 8.2), and the relator check keeps a margin of 15.0 at its tolerance of $2 \times 10^{-6}$ over every supported triple (Section 8.4). The plan tests up to this order. The number is a choice of this design (Section 15.3). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin |
| `aspect` | 0.125 to 8, inclusive | A finite value in a bounded range keeps the rectangle within the scale that the tests of Section 9.4 measure. NaN and infinities are rejected (DOM-18). The range is a choice of this design (Section 15.3) |

The limit is a private constant, and unit tests inside the crate read it. It applies in the same way to the orders of `*n22` and `*nn`, and `verify()` of a spherical domain costs no more products than at the hyperbolic limit (Section 8.2). `build_with` allocates at most four sides and four vertices, so the memory of a domain does not depend on the input. A setting is only compared with the names of the `ParameterSpec` values and stored, and the error text echoes no setting (Section 10.2), so the number and the length of the settings need no limit.

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library.
- Untrusted input and file paths: the signature is the only input. The crate reads and writes no file and keeps no state between calls.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate neither builds nor prints anything, and the command-line design governs what `escher-cli` prints.

The residual risk is a domain that is wrong by less than the tolerances. A defect that moves a wall or an angle by less than $10^{-9}$, or a relator by less than $2 \times 10^{-6}$ after the division by its order, passes every check. A second residual risk is in the measurements. The sweeps of Sections 8.4 and 9.3 run outside the repository, and no test repeats them. The committed tests measure every triple at which the sweeps have a largest value, but no measurement covers the vertices, normals, and side lengths of every supported triple (Section 9.3), and Section 14 lists the figures that no measurement of the code covers.

## 17. Operational Considerations

The crate declares no Cargo feature, reads no environment variable, and has no build script. It is pure Rust with no platform-specific code, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log.

Every failure of `build` is an `Error` with an `ErrorKind`. A failed check is not an error. `verify()` reports it in the `Verification`, and the caller decides what a failed check means:

| Failure | Reported as | Recovery |
| --- | --- | --- |
| A valid signature without a construction | `UnsupportedSignature` | None in this crate. A general non-compact signature has none in the proof of concept (`DEC-19`) |
| A corner order above 1000 | `LimitExceeded` | Use orders of at most 1000 |
| A setting whose name is not a parameter of the signature | `UnknownParameter` | Use a name that `parameters` lists |
| A value that is not finite or is outside the range of its parameter | `ParameterOutOfRange` | Use a value within the range that `parameters` lists |
| A degenerate construction | `Numerical` | None. No supported signature is known to produce it, and one that does is a defect of the crate |
| A check outside its tolerance | `Check::passed` is false | None. It is a defect of the crate, and the residual and the tolerance locate it |

## 18. References

### 18.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 18.2. Informative References

The tags of the mathematical sources resolve to the reference sections of the research documents:

- [BEA] Section 16.1 of `docs/research/geometry-models.md`.
- [CBG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CFKP] Section 16.1 of `docs/research/geometry-models.md`.
- [CH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [DAVIS] Section 16.1 of `docs/research/geometry-models.md`.
- [DAVIS2008] Section 14.1 of `docs/research/fundamental-domains.md`.
- [EPSTEIN-PETRONIO1994] Section 14.1 of `docs/research/fundamental-domains.md`.
- [HT] Section 16.1 of `docs/research/geometry-models.md`.
- [KC24] Section 16.1 of `docs/research/geometry-models.md`.
- [MASKIT2001] Section 14.1 of `docs/research/fundamental-domains.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [TH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [VONGAGERN2009] Section 14.1 of `docs/research/fundamental-domains.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| DOM-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| DOM-2 | `build` constructs a domain only for a signature that meets the predicate of Section 4, and returns `UnsupportedSignature` for every other valid signature without a panic | 4 |
| DOM-3 | `build` returns `LimitExceeded` for a supported signature with an order above 1000, after the support check | 4 |
| DOM-4 | Vertices and sides follow the index convention of Section 5.1 | 5.1 |
| DOM-5 | `build_with` constructs a hyperbolic domain by the steps of Section 6.1, a Euclidean one by those of Section 6.5, and a spherical one by those of Section 6.7, in order | 6.1 |
| DOM-6 | No relation between the inner product of two normals and the corner angle is used, except for the lune of DOM-20 | 6.1 |
| DOM-7 | `verify()` computes and reports every check on every call | 8.1 |
| DOM-8 | `verify()` is pure, never panics, and renormalizes nothing | 8.1 |
| DOM-9 | Each check uses the tolerance of Section 8.4 | 8.4 |
| DOM-10 | The vertices are not renormalized after centring | 9.1 |
| DOM-11 | The values that `build_with` returns are within the bounds of Section 9.3, of Section 9.4 for a Euclidean domain, or of Section 9.5 for a spherical one | 9.3 |
| DOM-12 | `Error` follows ERR-1, with the operation that failed | 10.1 |
| DOM-13 | The `Display` text of each kind is the one stated | 10.2 |
| DOM-14 | No test takes an expected value from Table 2 of [HT] | 12.1 |
| DOM-15 | The reference module shares no code with the crates under test | 12.2 |
| DOM-16 | Property tests take their configuration from one helper that is Miri-aware | 12.3 |
| DOM-17 | The crate declares only the dependencies of Section 13, each from the workspace table | 13 |
| DOM-18 | `build_with` checks support, the order limit, the names, and the values, in this order, and returns the first error that applies | 6.6 |
| DOM-19 | A test of a quantity without a measured bound here sets and records its bound as TEST-13 requires, in the same change | 9.4 |
| DOM-20 | The angle of a lune is measured from its two normals, and no other angle is | 7 |
| DOM-21 | For a spherical domain, the vertex precondition of checks 2 to 6 is finite and not the zero vector, with no condition on the last coordinate | 8.2 |
| DOM-22 | The exhaustive certificates cover every lune of their sets and stay while `UNV-29` is open | 12.1 |
