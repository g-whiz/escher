# Domain Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-17 |
| Applies to | `crates/escher-domain` |

## Abstract

This document specifies the crate `escher-domain`, which builds the fundamental domain of an orbifold signature as a polygon in the linear model, with a wall, a pairing isometry, and a partner for each side, and verifies it numerically. It covers the compact hyperbolic reflection triangle `*pqr`, the three Euclidean reflection triangles `*632`, `*442`, and `*333`, the rectangle `*2222` with its shape parameter `aspect`, the spherical reflection triangles `*332`, `*432`, `*532`, and `*n22`, and the lune of `*nn`. It also covers the rotation signatures `pqr`, `632`, `442`, `333`, `2222`, `332`, `432`, `532`, `n22`, and `nn`, whose domain is the double of the reflection domain of the starred signature across one wall, with pairings that are products of two reflections. It defines the public types and operations, the exact set of signatures that have a domain, the shape parameters and their validation, each construction as a numbered algorithm with the source of each step, the placement of the polygon, the measurements of its sides, angles, and area, and the six numeric checks of the verification with their tolerances and the defects that each one detects. It specifies the side presentation of a domain, a presentation of `escher-group` with one generator for each side and the relators of the corners or of the vertex cycles. It also states the numerical accuracy over the supported orders, the error type, the resource limits, the tests, the dependencies, the unverified statements that the code relies on, the assumptions, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

The renderer folds every pixel into the fundamental domain and colours it by the side pairings that the fold applied. It needs a polygon whose sides carry exact walls and pairing isometries, placed so that the canonical view is centred on it. The research gives the angles of the reflection triangle, its side lengths by the law of cosines, and the theorems that make it a fundamental domain, but it gives no placement in the linear model and no numerical tolerance (Section 10 of the geometry reference and Section 4.8 of the domain reference). A wrong normal, a wrong angle, or a wrong pairing produces an image that looks plausible, so the crate must check its own result.

### 1.2. Purpose

`escher-domain` is the one place where a `Signature` becomes a `FundamentalDomain`. It builds the polygon of a supported signature, returns an error kind for every other signature, and verifies a domain against the conditions of the polygon theorems that the domain reference cites. Section 3 of the architecture places the crate in the workspace. It depends on `escher-signature` for the signature and on `escher-geometry` for the linear model, and the renderer and `escher-cli` consume it.

### 1.3. Audience

The primary readers are the agent that changes the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-cli`, which prints the domain summary and the result of the verification, and of `escher-render`, which folds into the domain.

### 1.4. Scope

This document covers the public surface of the crate, the supported signatures, the shape parameter of `*2222`, the conventions of the sides, the vertices, and the pairings, the construction of the compact hyperbolic reflection triangle, the Euclidean triangle, the rectangle, the spherical triangle, the lune, and the double of each across one wall, their placement, their measurements, the side presentation, the verification, the numerical accuracy, the error type, the resource limits, the tests, and the dependencies.

The supported signatures are the compact hyperbolic `*pqr` with $1/p + 1/q + 1/r < 1$, the Euclidean `*632`, `*442`, `*333`, and `*2222`, the spherical `*332`, `*432`, `*532`, `*n22`, and `*nn`, and the rotation signature of each of them, which is the signature without the star, such as `732` for `*732` (Section 4).

### 1.5. Non-Goals

These topics are owned elsewhere or are outside this document:

- The bilinear form, the side test, the distance, the isometries, the reflection, the wall through two points, and the disk map, with their formulas and tolerances, belong to `escher-geometry` and are specified in `docs/design/geometry.md`. This document calls those functions by name and restates none of their formulas.
- The lines that `escher info` prints for a domain and for `--check`, their format, and the exit status are specified in `docs/design/cli.md`.
- Every shape parameter other than `aspect` of `*2222` and `2222`, including the skew of `2222`, and the kind and unit of a `ParameterSpec` that Section 3.2 of the architecture names. A reflection triangle has no free parameter ("triangles are rigid", Section 10.1 of the geometry reference).
- Words, relators, presentations, their text, and coset enumeration, which belong to `escher-group` and are specified in Sections 4 to 7 of `docs/design/group.md`. This document specifies only which presentation a domain gives (Section 5.4).
- Ideal vertices, the Euclidean and spherical signatures other than those of Section 4, and every other class of signature in the plan. Each returns `UnsupportedSignature` (Section 4).
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

- supported signature: A signature that meets the predicate of Section 4. `FundamentalDomain::build` constructs a domain for it when its corner orders are at most 1000 (DOM-3).
- corner $j$: For a reflection signature, the corner of the canonical signature at index $j$ of `boundaries()[0].corner_orders()`, with the order $n_j$ and the angle $\alpha_j = \pi/n_j$. For a rotation signature, the corner of its starred signature at that index, whose order is the cone order at index $j$ of `cone_orders()`.
- $m$: The number of sides of a domain, which is also its number of vertices: 2 for a lune and for the double of a lune, 3 for a triangle, 4 for the rectangle and for a kite, and 6 for the double of the rectangle.
- reflection signature: A supported signature with one boundary and no cone point, such as `*732`. Its domain is a reflection domain, every side of which is a mirror.
- rotation signature: A supported signature with cone points only, such as `732`. Its domain is a double (Section 6.8).
- starred signature: For a rotation signature, the reflection signature with a star before the same orders, so `*732` for `732` and `*2222` for `2222`.
- generating domain: The reflection domain $F$ of the starred signature, as Sections 6.1 to 6.7 build it, from which the double of a rotation signature is built.
- double: The union $D = F \cup R_1(F)$ of the generating domain and its image under the reflection $R_1$ in the wall of its side 1 (Section 6.8).
- kite: The double of a triangle, with four vertices.
- flat vertex: A vertex of a double at which two sides on one wall meet at the angle $\pi$ (Section 6.8).
- copies: The number of corners of $F$ that a vertex of a double joins, 2 for the two vertices on the doubled wall and 1 for every other vertex. The expected angle of a vertex is its copies times $\pi$ divided by its order.
- vertex cycle: The sequence of vertices that the side pairings carry into one another, as Section 4.1 of the domain reference defines it after Definition 11 of [SOLOMYAK2020].
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
pub enum PairingKind { Mirror, Rotation, Translation }
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
| `build_with` | The constructor with settings | Returns a domain exactly for a supported signature with corner orders of at most 1000 (Section 4) and valid settings (Section 6.6), and an `Error` otherwise. Never panics |
| `geometry()` | The geometry of the signature | `Hyperbolic`, `Euclidean`, or `Spherical` |
| `sides()` | The sides, in the order of Section 5.1 | $m$ sides, of kind `Mirror` for a reflection domain, and of kind `Rotation` or `Translation` for a double (DOM-27) |
| `vertices()` | The vertices, in the order of Section 5.1 | As many as the sides. For a reflection domain, vertex $j$ is corner $j$, and for a double, the vertices follow Section 6.8. The vertices are counterclockwise in the disk, in the plane for a Euclidean domain, where each is a Euclidean point $(x, y, 1)$, and as seen from outside the sphere for a spherical domain, where each is a sphere point whose last coordinate may be zero or negative (Section 5.2). Each has the accuracy of Section 9.3, 9.4, or 9.5, and a caller does not renormalize it (Section 9.1) |
| `centre()` | The centre of `DEC-12`: the normalized sum of the vertices, for a Euclidean domain their mean, for a lune `spherical::ORIGIN` (Section 6.7), and for a double the centre of its generating domain (Section 6.8) | Within $10^{-9}$ of the origin in each coordinate for a hyperbolic domain (Section 6.3), within the bound of Section 9.4 for a Euclidean one, within the bound of Section 9.5 for a spherical triangle, and exactly the origin for a lune. The centre of a double is bit for bit the centre of its generating domain |
| `Side::normal` | The unit normal of the wall of the side, for a Euclidean domain the unit normal $(a_1, a_2, 0)$ of its line, and for a spherical domain the great-circle normal of its great circle | Outward (Section 5.2), and spacelike for a hyperbolic domain |
| `Side::covector` | The covector of the wall | `hyperbolic::covector(normal)` for a hyperbolic domain, the line covector $(a_1, a_2, -t)$ for a Euclidean one, and the normal itself for a spherical one |
| `Side::pairing` | The pairing isometry $P_j$ of the side (Section 5.3 of the architecture) | For a mirror, `hyperbolic::reflection(normal)`, `euclidean::reflection(covector)`, or `spherical::reflection(normal)`, an orthogonal matrix. For a side of a double, the compensated product of two reflections of its generating domain (DOM-25) |
| `Side::partner` | The index of the side onto which $P_j$ maps side $j$ | For a mirror, $j$ itself. For a side of a double, the other side of its pair (DOM-24), never $j$ itself (DOM-26) |
| `Side::kind` | The kind of the pairing | `Mirror` for every side of a reflection triangle, of the rectangle, and of a lune. `Translation` for sides 2 and 5 of the double of the rectangle, and `Rotation` for every other side of a double |
| `Side::orientation` | Whether $P_j$ preserves or reverses orientation | `Reversing` for a mirror, and `Preserving` for a rotation or a translation |
| `side_lengths()` | The length of each side, in side order | Section 7 |
| `corner_angles()` | The angle at each vertex in radians, in vertex order | Section 7 |
| `area()` | The area measured from the corner angles, or for a Euclidean domain from the vertices | Section 7. For a spherical domain, in the metric of the unit sphere |
| `reverses_orientation()` | Whether some side has the orientation `Reversing` | True for every reflection domain and false for every double. It is the one place that decides whether a parity colouring exists, which `escher-cli` reads |
| `verify()` | The result of the six checks of Section 8.2 | Pure and never panics (DOM-8) |
| `side_presentation()` | The side presentation of Section 5.4 | Pure and never panics. Returns `InvalidPairing` for a domain whose sides give no presentation (DOM-35), which no domain that `build_with` returns does |
| `Verification::passed` | Whether every check passed | True exactly when each of the six `Check::passed` is true |
| `Check::passed` | Whether the residual is within the tolerance | `residual() <= tolerance()`. The residual is never NaN, and an infinite residual fails |
| `parameters` | The shape parameters of a signature, in a fixed order | Total. One `ParameterSpec` for `*2222` and for `2222`, and an empty list for every other signature, supported or not (Section 6.6) |
| `ParameterSpec` | The name, range, and default value of one shape parameter | `minimum() <= default_value() <= maximum()`, each finite. The name is the plain name of Section 5.6 of the architecture |
| `ShapeParameters` | The settings that a caller passes to `build_with` | Holds each name at most once. `new()` equals `Default::default()`, an empty set. `set` stores the name and the value that it is given and returns the value that it replaces, if any. It checks nothing, and `build_with` validates (Section 6.6) |
| `Error` | Why no domain was built | Section 10 |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

`Orientation` has both variants, because the renderer colours by the parity of the orientation of the pairings (Section 5.3 of the plan), and the doubles of Section 6.8 have `Preserving` sides. It is a closed set, so it is exhaustive under a lint exception (API-9 of the coding standard). `PairingKind` has the variants `Mirror`, `Rotation`, and `Translation` and grows with later classes, such as glide reflections, so it is `#[non_exhaustive]`. It has `Translation` beside `Rotation`, because the pairing of sides 2 and 5 of the double of the rectangle is the product of the reflections in two parallel walls and fixes no point, so one kind for every orientation-preserving pairing would call a translation a rotation. This split is a proposed default of this design (Section 15.3).

The corner of a vertex, with its order and its copies, is a private type. No public accessor exposes it, because `corner_angles()` already gives what `escher-cli` prints, and the checks of Section 8.2 read it inside the crate.

`reverses_orientation()` is a proposed default of this design. A caller could scan `sides()` for a `Reversing` side instead, and the accessor keeps that rule in one place for `escher-cli`.

`Verification` has one accessor for each check and no enum of checks, so a caller needs no wildcard arm, and a later check is a new accessor.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, and the unit test "Auto traits" of Section 12.1.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/domain.rs`: `FundamentalDomain`, `Side`, `PairingKind`, `Orientation`, the accessors, and the measurements of Section 7.
- `src/triangle.rs`: The supported-signature predicate of Section 4, and the construction of Section 6.1. The predicate returns a private enum with the two variants `Reflection` and `Rotation`, each over a private enum of the five shapes: the hyperbolic triangle, the Euclidean triangle, the rectangle, the spherical triangle, and the lune. So no match needs an arm for a class that cannot occur. The shape of a `Rotation` carries the area of the rotation signature where its geometry has one, and the generating domain is built with half of it, which is exact in IEEE 754 arithmetic and equals the area of the starred signature.
- `src/double.rs`: The construction of the double of Section 6.8, shared by every rotation class, with the compensated product of two reflections of DOM-25: an error-free product through `f64::mul_add`, an error-free sum, and a compensated dot product of three terms for each entry. Its unit tests "Compensated product" of Section 12.1 test these parts.
- `src/polygon.rs`: The constructions of the Euclidean triangle and the rectangle of Section 6.5, and the unit test "Exact rectangle" of Section 12.1.
- `src/spherical.rs`: The constructions of the spherical triangle and the lune of Section 6.7.
- `src/parameters.rs`: `parameters`, `ParameterSpec`, `ShapeParameters`, the private enum of the parameters with the name, the range, and the default of each, and the validation of Section 6.6.
- `src/verify.rs`: `Verification`, `Check`, and the checks of Section 8, including the walk of the vertex cycles of check 4, and the translation test of DOM-27, which `src/double.rs` also calls to set the kind of each side.
- `src/presentation.rs`: `side_presentation` of Section 5.4, which calls the walk of the vertex cycles of `src/verify.rs`, so that check 4 and the cycle relators come from one walk, and the unit test "Long cycle word" of Section 12.1.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, because the supported signatures do not need them:

- The kind and the unit of a `ParameterSpec`. `aspect` is a ratio, so it needs neither. The class that first needs them adds them as accessors, which does not break a caller. The crate sets `publish = false`, so a change affects only the workspace.
- Ideal vertices, which the triangle with an ideal corner adds.

`side_presentation()` returns a `Result`, as Section 3.2 of the architecture states. A domain whose sides mix mirrors and paired sides, or whose walk does not close, has no side presentation of Section 5.4, and an infallible function would have to invent one or panic (DOM-35). This is a proposed default of this design (Section 15.3).

`build` and `build_with` are inherent constructors and return a `Result`. `build_with` is a secondary constructor in the sense of API-5 of the coding standard. `ShapeParameters` takes the place of the options struct of API-6, with a private map and the method `set` and not public fields, because the names of the parameters depend on the signature. `build` stays, so that its callers do not change. Section 3.2 of the architecture lets the crate design refine the names and signatures.

`parameters` returns a `Vec` and `ParameterSpec::name` a `&str`, because later classes have parameter names that `escher-domain` derives from the signature, such as `slice1.length` (Section 5.6 of the architecture). `ShapeParameters` is a map, so it cannot hold a name twice (TYPE-3 of the coding standard), and the return value of `set` lets a caller detect a name set twice.

## 4. Supported Signatures

DOM-2. `FundamentalDomain::build` MUST construct a domain only for a signature that meets the predicate of this section, subject to DOM-3, and MUST return `UnsupportedSignature` for every other valid signature without a panic, because a caller that receives a domain for a signature outside the construction folds into a polygon that does not tile, and a panic stops the caller's whole program (ERR-5 of the coding standard).

A signature is supported exactly when all of these hold, each read from the public accessors of `escher-signature` (Section 3.1 of the signature design):

1. `handles()` is 0.
2. `crosscaps()` is 0.
3. One of these holds:
   - Reflection signature: `cone_orders()` is empty, and `boundaries()` has exactly one element, whose `corner_orders()` are the orders.
   - Rotation signature: `boundaries()` is empty, and `cone_orders()` are the orders.
4. There are two, three, or four orders.
5. `finite()` of each order is `Some`. This holds exactly when `is_compact()` is true for such a signature, and the crate reads `finite()` so that it needs no `expect` later.
6. One of these holds:
   - three orders, and `geometry()` is `Geometry::Hyperbolic`.
   - three orders, and `geometry()` is `Geometry::Euclidean`.
   - four orders, and `geometry()` is `Geometry::Euclidean`.
   - three orders, and `geometry()` is `Geometry::Spherical`.
   - two equal orders, and `geometry()` is `Geometry::Spherical`.

For a hyperbolic or spherical signature, the predicate also needs `area_over_pi()` to be `Some`, which it is for every such signature that meets conditions 1 to 6 (Section 9.4 of the signature design), so this condition makes no further signature unsupported. Step 1 of Section 6.1 converts the area. For a rotation signature, `area_over_pi()` is twice that of its starred signature, so `732` has the area $\pi/21$ and `*732` the area $\pi/42$.

The cone orders of a canonical rotation signature are in descending order, as the corner orders of a canonical reflection signature are, so the orders of a rotation signature are the corner orders of its starred signature in the same order. The predicate reads `geometry()` of the rotation signature itself and relates no property of it to its starred signature. The supported rotation signatures are these:

| Orders | Geometry | Rotation signatures | Generating domain |
| --- | --- | --- | --- |
| 3 | Hyperbolic | `pqr` with $1/p + 1/q + 1/r < 1$ | The triangle of Section 6.1 |
| 3 | Euclidean | `632`, `442`, and `333` | The triangle of Section 6.5 |
| 3 | Spherical | `332`, `432`, `532`, and `n22` with $n \ge 2$, which include `222` | The triangle of Section 6.7 |
| 4 | Euclidean | `2222` | The rectangle of Section 6.5 |
| 2, equal | Spherical | `nn` with $n \ge 2$, which include `22` | The lune of Section 6.7 |

`escher-signature` rejects `mn` with $m \ne n$ and a single cone point, such as `23` and `3`, as bad orbifolds, so no input reaches those cases.

The geometry is the exact rational classification of `escher-signature` (Section 9.1 of the signature design), so no floating-point comparison decides support. For three finite corners, the hyperbolic case of condition 6 is the condition $1/p + 1/q + 1/r < 1$ of Section 9.1 of the domain reference and Section 10.1 of the geometry reference. The Euclidean case with three corners is exactly `*632`, `*442`, and `*333`, the three solutions of $1/p + 1/q + 1/r = 1$ in Example 6.5.2 of [DAVIS2008] and in Section 7.2 of [RAT], p. 279 (Section 9.1 of the domain reference). The Euclidean case with four corners is exactly `*2222`, the only signature of one boundary with four corners among the 17 Euclidean signatures of Section 9.5 of the domain reference. The spherical case with three corners is exactly `*332`, `*432`, `*532`, and `*n22` with $n \ge 2$, the triples of Example 6.5.1 of [DAVIS2008] (Section 9.1 of the domain reference), which include `*222`. The spherical case with two corners is exactly `*nn` with $n \ge 2$, which include `*22`, because `escher-signature` rejects `*mn` with $m \ne n$ as a bad orbifold. The predicate also requires the two orders to be equal, because the lune of Section 6.7 stores one order for both corners, so a lune built from the first order of an unequal pair would pass its checks for a signature that it does not fit. No input reaches that case, so no test covers it.

Every other valid signature returns `UnsupportedSignature`. This covers the other Euclidean signatures, such as `*x` and `3*3`, the other spherical signatures, such as `3*2` and `*`, the hyperbolic `*3333`, `3222`, and `4*3`, every signature that mixes cone points with a boundary, the triangle with an ideal corner `*(inf)32`, the friezes, the trivial signature `1`, and every general non-compact signature. A general non-compact signature returns this kind throughout the proof of concept (`DEC-19`, Section 5.8 of the architecture).

DOM-3. `build` MUST return `LimitExceeded` for a supported signature with a corner order above 1000, and MUST apply the predicate of this section first, because the accuracy of Section 9.3 is established only up to that order, and a domain outside it could pass its checks by a smaller margin than the tolerances assume. A signature that is both unsupported and over the limit gets `UnsupportedSignature`, because the limit is a property of this construction. Section 16 states the reason for the number. The orders of a supported Euclidean signature are at most 6, so the limit does not bind for it. It binds for `*n22` and `*nn`, so `*(1001)22` and `*(1001)(1001)` return `LimitExceeded`. It applies in the same way to a rotation signature, so `(1001)32`, `(1001)22`, and `(1001)(1001)` return `LimitExceeded`.

## 5. Sides, Vertices, and Pairings

### 5.1. Index Convention

The canonical form of `*pqr` lists its corners with $n_0 \ge n_1 \ge n_2$ (Section 7.2 of the signature design, step 4). So `*237` and `*327` both become `*732`, and vertex 0 has the smallest angle and vertex 2 the largest.

DOM-4. The crate MUST number the vertices and sides of a reflection domain by this convention, and those of a double by the convention of Section 6.8 (DOM-24), because `escher-cli` prints the angles in vertex order and the side lengths in side order, and the renderer records the side index of every fold step, so a second numbering gives two readings of one printed value:

- For a reflection domain, vertex $j$ is corner $j$, with the order $n_j$.
- Vertex $j$ is the common point of side $j$ and side $j + 1$, with indexes modulo $m$. So side $j$ joins vertex $j - 1$ and vertex $j$.
- For a reflection domain, corner $j$ lies between side $j$ and side $j + 1$, so its relator is $(P_j P_{j+1})^{n_j}$. The relators of a double come from its vertex cycles (Section 8.2, check 4).

Section 9.1 of the notation reference reads a boundary with corners of orders $a$, $b$, and $c$ as the Latin reflections $P$, $Q$, $R$, and $S$ with the relators $(PQ)^a$, $(QR)^b$, and $(RS)^c$. Side 0 is $P$, side 1 is $Q$, and side 2 is $R$. For a triangle the convention reads $S$ as $P$ again, and for the rectangle side 3 is $S$. A lune has $m = 2$, so side 0 joins vertex 1 and vertex 0, side 1 joins vertex 0 and vertex 1, and its relators are $(P_0 P_1)^{n}$ at corner 0 and $(P_1 P_0)^{n}$ at corner 1. Section 15.1 records this reading as an assumption. The relators that the checks use come from Section 4.4 of the domain reference, which has three reflections and does not depend on it.

### 5.2. Orientation and Outward Normals

The vertices of a hyperbolic domain are counterclockwise in the disk: the planar cross product of `hyperbolic::to_disk(V1) - hyperbolic::to_disk(V0)` and `hyperbolic::to_disk(V2) - hyperbolic::to_disk(V0)` is positive. The vertices of a Euclidean domain are counterclockwise in the plane: the same cross product of the first two coordinates of $V_1 - V_0$ and $V_2 - V_0$ is positive. The vertices of a spherical triangle are counterclockwise as seen from outside the sphere: $(V_0 \times V_1) \cdot V_2$ is positive. The two vertices of a lune do not fix an orientation, and its placement in Section 6.7 numbers its sides counterclockwise as seen from outside the sphere at its centre.

A double with at least three vertices meets the same test of its geometry on its vertices $W_0$, $W_1$, and $W_2$ of Section 6.8, with a positive result. The double of a lune has the orientation of its generating lune. The test "Rotation table" of Section 12.1 asserts the orientation of each double.

The normal of each side points outward, as Section 5.2 of the architecture requires. A point $x$ is inside the domain where `hyperbolic::side(covector, x)`, `euclidean::side(covector, x)`, or `spherical::side(covector, x)` is at most 0 for every side, and a positive value means that the point is outside that side. Section 2.4 of the geometry reference records that [RAT] orients normals inward. This crate uses no statement of [RAT] that depends on that orientation.

### 5.3. Pairings

Side $j$ carries the isometry $P_j$ that maps the neighbouring copy of the domain across side $j$ onto the domain (Section 5.3 of the architecture). For a mirror side, $P_j$ is the reflection in its wall, `hyperbolic::reflection(normal)`, which is the reflection of equation (6.6) of [DAVIS] (Section 7.4 of the geometry reference) as Section 5.2 of the architecture adopts it. For a Euclidean mirror side, $P_j$ is `euclidean::reflection(covector)`, the reflection of equation (4.1.1) of [RAT] and equation 6.3 of [DAVIS] (Sections 7.4 and 12 of the geometry reference), whose form as a matrix is part of `UNV-26` (Section 14). For a spherical mirror side, $P_j$ is `spherical::reflection(normal)`, the reflection of equation (6.4) of [DAVIS] (Section 7.4 of the geometry reference). The partner of a mirror side is the side itself.

A side of a double carries the product of two reflections of its generating domain, which preserves orientation, and its partner is another side of the double (Section 6.8). The pairing of the partner is the inverse, as item 1 of Section 4.8 of the domain reference and Condition 3.2 (d) of [EPSTEIN-PETRONIO1994] require.

### 5.4. Side Presentation

`side_presentation()` reads a presentation of the group from the sides of the domain, as item 7 of Section 4.8 of the domain reference states it: the pairing relators together with the cycle relators. It returns a `Presentation` of `escher-group`, whose words, relators, text, and validation Sections 4.2 to 4.4 of the group design specify (GRP-3 to GRP-5). The command `escher info --presentation` prints it and enumerates its cosets (`docs/design/cli.md`), and the colourings of a later milestone read the side sequence of the fold as a word in its generators.

DOM-32. `side_presentation()` MUST give the presentation one generator for each side, in side order, named `s` followed by the side index in decimal from 0, such as `s0`, and the generator of side $i$ MUST stand for $P_i^{-1}$, the element of Section 5.3 of the architecture whose image of the domain is the neighbouring copy across side $i$, with a word read left to right as the product left to right, because the renderer records the fold as a sequence of side indexes, and a colouring that reads that sequence as a word needs the side index, the generator, and the order of the product to agree. For a mirror, $P_i^{-1} = P_i$.

DOM-33. For a domain whose every side is a mirror, the relators MUST be $s_j^2$ for each side $j$ in side order, followed by $(s_j s_{j+1})^{n_j}$ for each corner $j$ in corner order, with indexes modulo $m$ and $n_j$ the stored order of corner $j$, because these are the Coxeter relators of the reflection polygon (Theorem 6.4.3 of [DAVIS2008], Section 4.4 of [MASKIT2001], and Proposition 13.3.1(iii) of [TH], in Sections 4.4 and 4.5 of the domain reference and Section 9.4 of the notation reference), and a fixed order makes the text of `escher info` reproducible. Corner $j$ lies between side $j$ and side $j + 1$ (Section 5.1), so a lune gives $(s_0 s_1)^n$ at corner 0 and $(s_1 s_0)^n$ at corner 1, on `UNV-29`.

DOM-34. For a domain with no mirror side, the relators MUST be the pair relator $s_i s_p$ for each side $i$ whose partner $p$ is larger than $i$, in increasing order of $i$, followed by one cycle relator for each vertex cycle, in the order in which the walk of DOM-28 starts the cycles. The base of a cycle relator MUST be the generators of the sides $s$ that the walk applies, in the order in which it applies them, and its exponent the order $m_c$ of the cycle, because the walk is the one that check 4 evaluates, so the relators and the check describe the same cycles, and the presentation is the pairing relators together with the cycle relators (Theorem 12 (3) of [SOLOMYAK2020], Theorem 6.14 (iii) of [SERIES2013], and Theorem 4.14 of [EPSTEIN-PETRONIO1994], in Sections 4.1, 4.2, 4.8, and 4.9 of the domain reference). The pair relator writes $P_p = P_i^{-1}$ (Section 5.3) as a word.

No source read states the cycle relators as words in this convention. That the walk order of the base, with the generator of side $s$ for $P_s^{-1}$, gives the relator of the cycle is the reading of this design, with SymmHub's `groupPresentation` as precedent in code (Section 14.2 of `docs/research/symmhub.md`). It is `UNV-32` (Section 14). A reversed base, and the opposite convention in which the generator of side $i$ stands for $P_i$, present isomorphic groups, so no order or index distinguishes them. Only a numeric evaluation of the words does, and DOM-36 states which test establishes what.

DOM-35. `side_presentation()` MUST return an `Error` of the kind `InvalidPairing`, and MUST NOT panic, for a domain with both mirror and other sides, for a walk of DOM-28 that does not return to its start within $m$ steps, for a cycle whose vertices differ in order or whose copies do not sum to 2, and when `Relator::new` or `Presentation::new` of `escher-group` rejects a relator or the presentation (Sections 4.2 and 4.3 of the group design), because such a domain has no side presentation of this section, and `escher info` prints the result for any signature that it reads. These are the structural conditions of check 4, so a domain that passes check 4 structurally gets a presentation. No domain that `build_with` returns gives this error, and the test "Invalid pairing" reaches it through private access (Section 12.1).

The side presentation keeps a generator for each side of a pair, with the pair relator, as SymmHub does, so the generator of a side is found by the side index alone. Section 15.3 records the alternative of one generator for each pair (`DEC-36`).

These are the side presentations of four domains, in the text of Section 4.4 of the group design:

| Signature | Generators | Relators |
| --- | --- | --- |
| `*532` | `s0`, `s1`, `s2` | `s0^2, s1^2, s2^2, (s0 s1)^5, (s1 s2)^3, (s2 s0)^2` |
| `532` | `s0` to `s3` | `s0 s1, s2 s3, s1^5, (s2 s0)^2, s3^3` |
| `2222` | `s0` to `s5` | `s0 s1, s2 s5, s3 s4, s1^2, (s2 s0)^2, (s3 s5)^2, s4^2` |
| `22` | `s0`, `s1` | `s0 s1, s1^2, s0^2` |

The construction gives no relator that is the empty word, and it reduces no relator: a relator of the walk with one letter, such as `s1^5`, stays as it is.

Every vertex cycle of every domain of Section 4 visits one or two sides, as the table of Section 6.8 shows. For a base of one or two letters, the reversed base is a cyclic conjugate of the base, and the pair relator $s_i s_p$ has the same property. So no domain that this crate builds distinguishes the convention of DOM-32 from its reversal or from the opposite convention. A prototype outside the repository confirmed this on every built domain (Section 14).

## 6. Construction

### 6.1. Algorithm

DOM-5. `build_with` MUST construct a hyperbolic reflection domain by the steps of this section, a Euclidean one by the steps of Section 6.5, a spherical one by the steps of Section 6.7, and the domain of a rotation signature by the steps of Section 6.8, each in its order, because the accuracy of Sections 9.3 to 9.6 and the margins of Section 8.4 are measured for exactly these steps, and another order of the same operations gives other rounding errors that no test measures.

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
- The final position moves the centre of `DEC-12` to the origin. The owner decided on one canonical view centred on the domain (Section 10.1 of the architecture), and Section 5.3 of the plan places the centre of the domain at the centre of the disk. With this placement, the canonical view of the disk is the plain disk map. `centre()` stays public, so `View` in `escher-render` keeps the freedom that Section 3.2 of the architecture gives it.

The double of a rotation signature leaves its generating domain where this section places it, and its centre is the centre of the generating domain. The owner decided on 2026-11-17 that the canonical view of a rotation signature is the view of its starred signature, centred on the generating domain, so that `732` and `*732` show the same tiling in the same place (Section 10.1 of the architecture). The centre of the generating domain lies strictly inside the double, which check 2 tests on every build.

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

The vertex coordinates of the triangle have no source, so the build is certified three ways: check 3 tests each vertex against its two walls, check 5 tests each angle against $\pi/n_j$ by equation (1.3.3) of [RAT], and the test "SymmHub lines" of Section 12.1 compares the built walls with the three lines that SymmHub's code gives for this triangle (Section 9.1 of the domain reference), moved by the translation of step 4. Building the walls from those lines and the vertices by intersection would not close the gap, because the intersection of two lines has no source either.

### 6.6. Shape Parameter and Validation

`*2222` and `2222` each have the one shape parameter `aspect`, and every other signature has none:

| Name | Minimum | Maximum | Default | Meaning |
| --- | --- | --- | --- | --- |
| `aspect` | 0.125 | 8 | 1 | The length of side 0 divided by the length of side 1 of the rectangle. For `2222`, the same ratio of its generating rectangle, so the double is $2 \cdot$ `aspect` times as wide as it is high |

For `2222`, `aspect` is the aspect of the generating rectangle, with the area 1, and the double has the area 2. This is a proposed default of this design (Section 15.3). Side 1 of the generating rectangle is always its right side, so `aspect` and its reciprocal give doubles of different shapes, and the quarter-turn property of item 7 of Section 15.3 holds for `*2222` only. The double of the rectangle is one member of the family of `2222`, whose other parameter is a skew: [VONGAGERN2009] p. 7 gives p2 "two geometric parameters" (Section 9.5 of the domain reference), and whether that count includes the scale is `UNV-10`. This design builds no skewed domain.

The range and the default are choices of this design (Section 15.3). The default is the square, as `DEC-13` asks for a regular shape where the orders allow it. Both ends are exact in `f64`. The scale is fixed by the area 1 and is not a parameter, and the design asserts no count of parameters, so `deformation_dimension()` of `escher-signature` stays as the signature design states (`UNV-10`).

DOM-18. `build_with` MUST check in this order, and MUST return the first error that applies, because `escher-cli` prints the error of a combination of a signature and settings, and its snapshots fix which kind each combination gives:

1. The predicate of Section 4, else `UnsupportedSignature`.
2. The limit of DOM-3, else `LimitExceeded`.
3. Every name of a setting is the name of a `ParameterSpec` of the signature, else `UnknownParameter`.
4. Every value is finite and within the closed range of its `ParameterSpec`, else `ParameterOutOfRange`. A NaN or an infinite value is out of range.
5. A parameter without a setting takes its default value.

A name matches only when it is equal, byte for byte, to the name of a `ParameterSpec`, so `Aspect` and `aspect ` with a trailing space are unknown. A rotation signature passes the same steps, with the `ParameterSpec` values of its own, so `aspect` on `732` gives `UnknownParameter`.

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

### 6.8. Rotation Construction

DOM-23. `build_with` MUST build the domain of a rotation signature as the double $D = F \cup R_1(F)$ of the generating domain $F$ across the wall of its side 1, MUST leave $F$ where Sections 6.1 to 6.7 place it, and MUST give the double the centre of $F$, because the owner decided that `732` and `*732` show the same tiling in the same place (Section 6.3), the test "Border subset" of `escher-render` compares the two images in one view, and the measurements of Sections 8.4 and 9.6 cover this wall and this placement only.

For a rotation signature, `build_with` runs steps 1 to 5 of DOM-18, computes the area of the rotation signature as step 1 of Section 6.1 does, and builds $F$ by the construction of its starred signature with the same orders, and for `2222` with the validated value of `aspect`. Let $F$ have the $n$ vertices $V_0, \dots, V_{n-1}$, counterclockwise, with the corner orders $n_0, \dots, n_{n-1}$, and the sides $S_j$ from $V_{j-1}$ to $V_j$, with indexes modulo $n$, each with its normal $u_j$, its covector $c_j$, and its reflection $R_j$, the pairing that $F$ stores. Then:

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
5. Give the side that is $S_j$ the pairing $R_1 R_j$, and the side that is the image of $S_j$ the pairing $R_j R_1$, each by DOM-25. Give each side the orientation `Preserving`, and the kind `Translation` when its pairing passes the translation test of DOM-27, and `Rotation` otherwise. The construction and check 2 call the same test, and it gives `Translation` for sides 2 and 5 of the double of the rectangle, whose walls do not meet the wall of side 1, and for no other side of the doubles of the test "Rotation table".
6. Give vertex $W_0$ the order $n_0$ and the copies 2, vertex $W_{n-1}$ the order $n_1$ and the copies 2, and every other vertex the order of the vertex of $F$ that it is or that it is the image of, and the copies 1.
7. Give $D$ the centre of $F$, bit for bit.
8. Return `Numerical` if any stored number is not finite, or if the side value of the centre is not strictly negative for every side.

The steps give these three classes. $V_j'$ is $R_1 V_j$:

| Class | $n$ | Vertices of $D$ | Side pairs and kinds | Flat vertices | Vertex cycles |
| --- | --- | --- | --- | --- | --- |
| Kite | 3 | $V_0$, $V_2'$, $V_1$, $V_2$ | 0 and 1, 2 and 3, by rotation | $W_2$ when $n_1 = 2$, so for `n22`, and also $W_0$ for `222` | $\{W_0\}$, $\{W_2\}$, $\{W_1, W_3\}$ |
| Double of the rectangle | 4 | $V_0$, $V_3'$, $V_2'$, $V_1$, $V_2$, $V_3$ | 0 and 1 by rotation, 2 and 5 by translation, 3 and 4 by rotation | $W_0$ and $W_3$ | $\{W_0\}$, $\{W_3\}$, $\{W_1, W_5\}$, $\{W_2, W_4\}$ |
| Double of the lune | 2 | $V_0$, $V_1$ | 0 and 1, by rotation | Both, for `22` | $\{W_0\}$, $\{W_1\}$ |

No hyperbolic or Euclidean kite has a flat vertex, because a flat vertex needs a corner of order 2 at $V_0$ or $V_1$. The vertex cycles of the table are the ones that `UNV-31` states. Check 4 walks them on every build (Section 8.2), and the test "Cycle sets" of Section 12.1 asserts them. `732` has the corner angles of 51.428571, 90, 120, and 90 degrees and the area $\pi/21$.

Side 0 of a kite is $S_0$, so side 0 of a Euclidean kite has the length 1 (Section 6.5). The double of the rectangle with the width $w$ and the height $h$ of Section 6.5 is the rectangle from $x = -w/2$ to $x = 3w/2$, with the height $h$ and the area 2.

The wall of side 1 is the wall through $V_0$ and $V_1$: for a triangle the side between the two corners of the largest orders, for the rectangle its right side, and for the lune its side with the normal $(-c, 0, -s)$ of Section 6.7. The choice of side 1 is a proposed default of this design, which Section 15.3 puts to the owner. `DEC-11` leaves the choice of the wall to this document (Section 10.2 of the architecture). On a prototype outside the repository, the double of `*(1000)32` across side 2 failed check 4 and left folds unconverged in band 6 of Section 6.4 of `docs/design/render.md`, while the double across side 1 passed every check (Section 14 gives the figures). SymmHub's `getGroup_KLM` also doubles the triangle across one of its sides, with pairings that are products of two reflections (Section 7.2.7 of `docs/research/symmhub.md`), which is a precedent in code and not a source.

DOM-24. The vertices, the sides, and the partners of a double MUST follow steps 2 to 4 of this section, with the partner $(2n - 1 - k) \bmod (2n - 2)$ of side $k$, because the renderer records the side index of every fold step, `escher-cli` prints the angles and the side lengths in this order, and check 4 finds the vertex cycles by walking these partners.

DOM-25. The pairing of a side of a double that is $S_j$ MUST be $R_1 R_j$, and that of a side that is the image of $S_j$ MUST be $R_j R_1$, each the compensated product of the two reflections that $F$ stores: every entry MUST be the compensated dot product of a row and a column of the two `f64` matrices, with error-free products by a fused multiply-add and a compensated sum of the three products and their errors. The reason is the fold. On a prototype outside the repository, one fold of `(1000)32` within the range of accuracy of Section 6.4 of `docs/design/render.md` failed to converge with plain double-precision products as pairings, while products computed in 256-bit arithmetic from the two stored reflections and rounded once to the nearest `f64` left no fold of the fold set unconverged (Section 14 gives the figures).

Compensated arithmetic does not guarantee correct rounding. For example, the compensated dot product of $(1, 2^{-53}, 2^{-110})$ and $(1, 1, 1)$ returns 1, while the exact sum rounds to $1 + 2^{-52}$. What is established is narrower: the test "Compensated pairings" of Section 12.1 asserts that every entry of every pairing of the doubles of Section 9.6 is the exact product of the two stored reflections rounded to the nearest `f64` with ties to even. Section 15.1 records the doubles outside that set as an assumption.

The crate computes each entry at construction only. The compensation runs once per side, so the fold costs the same. `f64::mul_add` of the standard library gives the fused multiply-add, which IEEE 754 defines exactly, so the product is deterministic.

The compensated product does not reduce the isometry defect of a pairing in general. Over the cone triples of the 30 named triples, the largest isometry defect is the same with compensated and with plain products, and at some doubles the compensated product has the larger defect (Section 9.6 gives the figures). The isometry defect of a pairing of a double therefore rests only on the bounds that Section 9.6 sets from measurement. This requirement is a proposed default of this design, and Section 15.3 states the alternatives.

DOM-26. Where the wall of a side $S_j$ meets the wall of side 1 at a corner of order 2, at $V_0$ or $V_1$, the double MUST keep $S_j$ and its image as two sides that are partners of each other and meet at a flat vertex, and MUST NOT make any side its own partner, because Theorem 12 of [SOLOMYAK2020] requires that no side is paired with itself (Section 4.1 of the domain reference), and check 2 treats a side that is its own partner as a mirror. Item 2 of Section 4.8 of the domain reference permits this split and does not require it, so the split is a choice of the plan (Section 5.6 of the plan).

The image vertices, normals, and covectors are plain double-precision products. Neither a prototype outside the repository nor a test of this crate measured whether correctly rounded image vertices and normals improve the margin of the vertices against the tolerance of the fold (Section 15.3, item 27).

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

## 7. Measurements

The crate measures each quantity from the stored vertices, so that `verify()` and `escher-cli` see the same numbers:

- `side_lengths()` returns `hyperbolic::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the angle $\theta_j$ in $[0, \pi]$ for which the first law of cosines of Section 10.2 of the geometry reference ([BEA] Section 7.12, p. 148, and [RAT] Theorem 3.5.3, p. 86) holds with the three measured side lengths. With $c$ the length of the side opposite vertex $j$ and $a$ and $b$ the other two, it is $\arccos\big((\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)\big)$, with the argument clamped to $[-1, 1]$.
- `area()` returns $\pi - \theta_0 - \theta_1 - \theta_2$, the area of a triangle by Section 10.1 of the geometry reference.

For a hyperbolic domain, `corner_angles()` and `area()` are defined for exactly three vertices, and for the four vertices of a kite as DOM-29 states. With any other number of vertices, `corner_angles()` returns NaN for every vertex and `area()` returns NaN.

The angles are measured from distances so that no step uses a relation between normals and angles (DOM-6). The form of the first law solved for the angle is a rearrangement that no source prints, and Section 15.1 records it.

For a Euclidean domain, with $(x_j, y_j)$ the first two coordinates of $V_j$:

- `side_lengths()` returns `euclidean::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the Euclidean angle $\theta$ of equation (1.3.3) of [RAT] (Section 12 of the geometry reference) between $V_{j+1} - V_j$ and $V_{j-1} - V_j$, as $\arccos$ of their dot product divided by the product of their lengths, with the argument clamped to $[-1, 1]$. It uses no normal, as DOM-6 requires.
- `area()` returns $\frac{1}{2} \left\lvert \sum_j (x_j y_{j+1} - x_{j+1} y_j) \right\rvert$, with indexes modulo $m$. No source states this formula (item 25 of Section 15.1 of the geometry reference), and Section 14 records it.

For a spherical domain, the measurements read the theorems of [RAT], Section 2.5 (Section 11.3 of the geometry reference), with lengths and areas in the metric of the unit sphere:

- `side_lengths()` returns `spherical::distance(V_{j-1}, V_j)` for each side $j$. Both sides of a lune have the length $\pi$.
- `corner_angles()` returns, for each vertex $j$ of a triangle, $\arccos\big((\cos c - \cos a \cos b) / (\sin a \sin b)\big)$, with the argument clamped to $[-1, 1]$, where $c$ is the measured length of the side opposite vertex $j$ and $a$ and $b$ the other two. This is the first law of cosines, [RAT], Theorem 2.5.3, p. 50, which prints the cosine of the angle in this form. For a lune, and for the double of a lune, it returns one angle $\theta$ for both vertices, from the normals $u_0$ and $u_1$ of its two sides by the form of DOM-20.
- `area()` returns $\theta_0 + \theta_1 + \theta_2 - \pi$ for a triangle, [RAT], Theorem 2.5.5, p. 51, and $2\theta_0$ for a lune, the area of a lune of [RAT], p. 51.

For a spherical domain with a number of vertices other than two, three, or the four of a kite, `corner_angles()` returns NaN for every vertex and `area()` returns NaN.

DOM-20. `corner_angles()` MUST measure the angle of a domain with two vertices from the normals $u_0$ and $u_1$ of its two sides: for a lune as $\arccos(-u_0 \cdot u_1)$ with the argument clamped to $[-1, 1]$, and for the double of a lune as $\operatorname{atan2}(\lvert u_0 \times u_1 \rvert, -u_0 \cdot u_1)$. It MUST NOT use a relation between normals and angles for any other domain. The two vertices of such a domain are antipodal and so determine neither side, while for every other domain the vertices determine the angle and DOM-6 applies. The double uses `atan2`, because its angle is $\pi$ for `22`, where the argument of the arccosine is $-1$ and the arccosine magnifies an error in the last place of the normals to about $1.5 \times 10^{-8}$, above the tolerance $10^{-9}$ of checks 5 and 6. The lune keeps the arccosine, so that its values stay bit for bit (DOM-30). The code chooses the form by the kind of side 0: `Mirror` for a lune, and `Rotation` for its double.

The relation between the normals and the angle has no source (item 28 of Section 15.1 of the geometry reference), and Section 15.1 records the `atan2` form as an assumption. Check 4 and the test "Exhaustive certificate" test the same angle without it. The area of the double of a lune is $2\theta_0$, the area of a lune of [RAT], p. 51, as for a lune.

A probe that moved each vertex coordinate and each normal component by one unit in the last place gives the largest change of the angle in this table. No committed test repeats it:

| Double | Arccosine | `atan2` |
| --- | --- | --- |
| `22` | $1.490 \times 10^{-8}$ | $4.441 \times 10^{-16}$ |
| `(1000)(1000)` | $1.767 \times 10^{-14}$ | $1.735 \times 10^{-18}$ |

The probe also covered `55`, `2222`, `222`, `(12)22`, `(1000)22`, `*22`, and `*55`, and on each of them the largest change is the same with either form. The largest of these is $7.068 \times 10^{-14}$, at the kite `(1000)22`.

A double measures its side lengths with the distance of its geometry, as above. A Euclidean double measures its angles and its area by the Euclidean formulas above, which apply to any number of vertices. A kite in the hyperbolic plane or on the sphere has four vertices, and the formulas above measure triangles only, so it is measured as two triangles:

DOM-29. For a hyperbolic or spherical kite, `corner_angles()` MUST split the kite along the diagonal from $W_0$ to $W_2$, which lies on the doubled wall, into the triangles $(W_0, W_1, W_2)$ and $(W_0, W_2, W_3)$, measure the angles of each triangle by the first law of cosines of its geometry as above, return the angle of $W_1$ and of $W_3$ from its one triangle, and return the angle of $W_0$ and of $W_2$ as the sum of its two angles, and `area()` MUST return the sum of the areas of the two triangles, each measured as above, because DOM-6 forbids angles from normals for a domain whose vertices determine its sides, and the sourced laws of cosines and area formulas of the research measure triangles only. No angle of a kite comes from its normals. At a flat vertex each triangle has a right angle, where the arccosine is well conditioned.

That the angle of a kite at $W_0$ or $W_2$ is the sum of the angles of its two triangles there rests on no sentence of a source, and Section 15.1 records it.

## 8. Verification

### 8.1. Report Types

`verify()` returns a `Verification` that holds one `Check` for each of the six checks of Section 8.2. Each `Check` holds its residual and its tolerance. A check passes when its residual is at most its tolerance, so an infinite residual fails. Section 2.1 defines the residual, which is never NaN.

DOM-7. `verify()` MUST compute every check of Section 8.2 on every call and MUST report each one separately, because `escher info --check` prints the result of each check, and a verification that stops at the first failure hides whether the other conditions hold.

DOM-8. `verify()` MUST be pure, MUST NOT panic for any domain, and MUST NOT renormalize any stored value, because it runs on domains that tests corrupt on purpose, and a renormalized value would hide the defect that the check exists to find.

### 8.2. Checks

Every check runs on the stored values of the domain in the final position, unless the check names another frame. Checks 2 to 6 read the vertices, so each first requires at least one side, as many vertices and corners as sides, and every vertex to have finite coordinates and, for a hyperbolic or Euclidean domain, a positive last coordinate, or for a spherical domain, not to be the zero vector (DOM-21). Its residual is infinity when any of these fails. Check 1 reads only the pairings. A residual that is not finite after the computation of its check is reported as infinity:

1. **Isometries.** For each side, compute `hyperbolic::isometry_defect(pairing)`. The residual is the largest value. Tolerance $10^{-9}$. Condition: Theorem 10.1 of [CFKP] (Section 6.1 of the geometry reference).
2. **Pairings.** The residual is infinity unless every structural condition holds, and otherwise the largest of the numeric parts. Tolerance $10^{-9}$. Conditions: items 1 and 2 of Section 4.8 of the domain reference, and Condition 3.2 (a) to (d) of [EPSTEIN-PETRONIO1994] (Section 4.9 of the domain reference).
   - Structural: `partner(partner(i)) == i` for every side.
   - Structural: with $c$ the stored centre, `hyperbolic::side(covector_i, c)` is negative for every side $i$, and `hyperbolic::side(covector_partner(i), P_i c)` is positive.
   - Numeric: the largest absolute entry of $P_i P_{\text{partner}(i)} - I$ over the sides. For a mirror this is the relator $r_i^2 = 1$.
   - Structural: the kind, the partner, the orientation, and the translation test of every side agree, as DOM-27 states, and the sign of the determinant of $P_i$ is positive for `Preserving` and negative for `Reversing`.
   - Numeric: for a mirror side $i$, the largest absolute coordinate of $P_i E - E$ divided by the last coordinate of $E$, for the endpoints $E = V_{i-1}$ and $E = V_i$.
   - Numeric: for a side $i$ that is not a mirror, with $p$ its partner, the largest absolute coordinate of $P_i V_{i-1} - V_p$ divided by the last coordinate of $V_p$, and of $P_i V_i - V_{p-1}$ divided by the last coordinate of $V_{p-1}$, so that $P_i$ maps the side onto its partner with its endpoints exchanged (Condition 3.2 (c) of [EPSTEIN-PETRONIO1994]).
   - Numeric: for a side $i$ that is not a mirror, with $p$ its partner, the wall term: the largest absolute coordinate of $P_i^{T} c_p + c_i$, for the covectors $c_i$ and $c_p$, with no division. It is 0 when $P_i$ carries the wall of side $i$ onto the wall of side $p$ with the domain on opposite sides (Condition 3.2 (b) and (c)). The endpoint terms alone also pass a pairing that maps the two endpoints correctly but not the wall, such as the half-turn about the axis through the two vertices of the double of a lune (Section 8.3).
3. **Walls.** For each side $j$, compute these five values. The residual is the largest of them over the sides. Tolerance $10^{-9}$. Condition: a wall is a unit normal stored with its covector (Sections 2.1 and 5.2 of the architecture), it is orthogonal to the points of its side (equation (6.6) of [DAVIS], Section 7.4 of the geometry reference), the pairing of a mirror is the reflection in that wall (Section 5.3), and the polygon is convex, so no vertex lies outside a wall (the convex polyhedron of Definition 2.1 of [EPSTEIN-PETRONIO1994], Section 4.9 of the domain reference, and `UNV-31` for a double).
   - The absolute value of `hyperbolic::side(covector_j, V_{j-1})` divided by the last coordinate of $V_{j-1}$.
   - The same for $V_j$.
   - For a side of the kind `Mirror`, the largest absolute entry of `pairing_j - hyperbolic::reflection(normal_j)`, and 0 for every other side.
   - The largest absolute coordinate of `covector_j - hyperbolic::covector(normal_j)`.
   - The convexity term: the largest of 0 and `hyperbolic::side(covector_j, V_k)` divided by the last coordinate of $V_k$, over every vertex $V_k$ of the domain.
4. **Relators.** For a domain whose sides are all mirrors, for each corner $j$, let $F =$ `hyperbolic::translation(V_j)`, which reads only the first two coordinates of $V_j$, and let $C =$ `hyperbolic::inverse(F) * P_j * P_{j+1} * F`. Compute $C^{n_j}$ by $n_j - 1$ successive multiplications by $C$. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. Tolerance $2 \times 10^{-6}$. Condition: the relator $(r_j r_{j+1})^{n_j} = 1$ of the presentation of Section 4.4 of the domain reference ([MASKIT2001] Section 4.4), and item 4 of its Section 4.8.
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
| 3. Walls | For each side $j$, the largest of: the absolute value of `euclidean::side(covector_j, E)` divided by the last coordinate of $E$ for $E = V_{j-1}$ and $E = V_j$, the largest absolute entry of `pairing_j - euclidean::reflection(covector_j)`, the largest absolute coordinate of `normal_j - (c_1, c_2, 0)` for the covector $c$, $\lvert c_1^2 + c_2^2 - 1 \rvert$, and the convexity term with `euclidean::side` | A line covector of a line through the endpoints of its side, stored with its unit direction, whose reflection is the pairing (Sections 4.5 and 6.6 of the geometry design) |
| 4. Relators | For each corner $j$, $C = P_j P_{j+1}$, and $C^{n_j}$ by $n_j - 1$ successive multiplications. The residual is the largest absolute entry of $C^{n_j} - I$ divided by $n_j$, maximized over the corners. There is no corner frame | Item 4 of Section 4.8 of the domain reference, and the local group $\langle a, b : a^2 = b^2 = (ab)^n = 1 \rangle$ of a corner reflector of order $n$ of [TH], Proposition 13.3.1(iii), p. 308 (Section 9.4 of the notation reference) |
| 5. Angles | The largest $\lvert \theta_j - \pi/n_j \rvert$, with $\theta_j$ of Section 7. The condition on the angle sum of a hyperbolic triangle does not apply | Every corner has the angle $\pi/n_j$ (item 4 of Section 4.8 of the domain reference) |
| 6. Area | $\lvert (m - 2)\pi - \sum_j \theta_j \rvert$, against the value $-2\pi\chi = 0$ of a Euclidean signature | A consistency check only, as below |

Check 6 of a Euclidean domain has the residual 0 on every Euclidean domain of Section 9.4 (Section 8.4), and passes on the triangle of `*442` labelled `*632`, where checks 4 and 5 fail (Section 8.3). The design keeps it so that the six checks exist for every geometry, and Section 15.3 puts that to the owner.

For a spherical domain, `verify()` computes the same six checks with the same tolerances, and reads each one in this way, with indexes modulo $m$:

| Check | Spherical reading | Condition |
| --- | --- | --- |
| 1. Isometries | `spherical::isometry_defect` of each pairing | [RAT], Theorem 2.1.3 (Section 11.1 of the geometry reference) |
| 2. Pairings | As for a hyperbolic domain, with `spherical::side`. The endpoint part is the largest absolute coordinate of $P_i E - E$, with no division by a last coordinate | As for a hyperbolic domain |
| 3. Walls | For each side $j$, the largest of: the absolute value of `spherical::side(covector_j, E)` for $E = V_{j-1}$ and $E = V_j$, with no division, the largest absolute entry of `pairing_j - spherical::reflection(normal_j)`, the largest absolute coordinate of `covector_j - normal_j`, $\lvert \text{normal}_j \cdot \text{normal}_j - 1 \rvert$, and the convexity term with `spherical::side` and no division. Also $\lvert V \cdot V - 1 \rvert$ for each vertex $V$, and, for a lune, the largest absolute coordinate of $V_0 + V_1$ | A great-circle normal of a great circle through the endpoints of its side, stored as its own covector, whose reflection is the pairing (Sections 4.6 and 6.7 of the geometry design). Each vertex is a sphere point, and the two vertices of a lune are antipodal (Section 6.7) |
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

DOM-27. Check 2 MUST fail structurally unless the kind, the partner, the orientation, and the pairing of every side agree: a side of the kind `Mirror` is its own partner and has the orientation `Reversing`, a side of the kind `Rotation` or `Translation` has a partner other than itself and the orientation `Preserving`, and a side has the kind `Translation` exactly when its pairing passes the translation test. The translation test holds when the domain is Euclidean and each entry of the linear part of the pairing, its upper-left 2 by 2 block, is within the tolerance $10^{-9}$ of check 2 of the identity. The reason is that the renderer reads the partner, the orientation, and the pairing of a side and never its kind, so a kind that these contradict would go undetected, and `escher info` prints the kind of each pair in its `Side pairs` line, which that kind would make false.

DOM-28. Check 4 of a domain with no mirror side MUST walk the vertex cycles and evaluate the relator of each cycle, and check 4 of a domain with both mirror and other sides MUST have the residual infinity, because a corner relator $(P_j P_{j+1})^{n_j}$ holds only between two mirrors, and no supported signature builds a domain with both kinds, so a residual for one would rest on no stated condition.

The walk follows Definition 11 of [SOLOMYAK2020], with $P_s$ as the side pairing. For each vertex `start` that no earlier cycle visited, it sets $v$ to `start` and $g$ to the identity, and repeats these steps until $v$ is `start` again: $s = (v + 1) \bmod m$, then $g \leftarrow P_s g$, then $v = $ `partner(s)`. A walk that takes more than $m$ steps fails structurally. For each cycle:

- Structural: every vertex of the cycle has the same order, and the copies of its vertices sum to 2. That order is the order $m_c$ of the cycle.
- Numeric: the largest absolute coordinate of $g V_{\text{start}} - V_{\text{start}}$, divided by the last coordinate of $V_{\text{start}}$ for a hyperbolic or Euclidean domain. This is the condition that the cycle transformation fixes its vertex (Condition 3.10 of [EPSTEIN-PETRONIO1994]).
- Numeric: the largest absolute entry of $C^{m_c} - I$ divided by $m_c$, with $C^{m_c}$ by $m_c - 1$ successive multiplications. For a hyperbolic domain, $C = F^{-1} g F$ in the corner frame of $V_{\text{start}}$, with $F =$ `hyperbolic::translation(V_start)`, and otherwise $C = g$.

The residual is the largest numeric part over the cycles, or infinity when a structural condition fails. The expected cycles are those of the table of Section 6.8. For a kite, the cycles $\{W_0\}$ and $\{W_2\}$ are the rotations about corners 0 and 1 of the generating triangle, and the cycle $\{W_1, W_3\}$ is a product of four reflections.

At a flat vertex of a Euclidean double, check 5 compares an arccosine of an argument close to $-1$ with $\pi$, where the arccosine is ill-conditioned. A small tilt of a wall can therefore pass check 5, so checks 2 and 3 carry that case through the vertices and the pairings. A kite measures its flat vertex as two right angles (DOM-29), and the double of `22` measures its angle with `atan2` (DOM-20), so this does not apply to either.

The walk needs one product for each step of each cycle, at most $m$ in all. With the frames, the powers, and check 2, the cost of `verify()` of a double at the order limit stays about 3000 matrix products, as for a reflection domain, because the powers dominate and a kite has three cycles.

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

The test also injects these defects into spherical domains, which pass every check before the defect: the domains of `*532`, `*722`, and `*55`, unless the row names others. A tilt is a rotation of the normal about the origin by `spherical::rotation`. On each domain of its row, exactly the checks of the row fail:

| Defect | `*532` | `*722` | `*55` |
| --- | --- | --- | --- |
| The normal of side 0 tilted by $10^{-7}$, with its covector and pairing computed from the tilted normal | 2 and 3 | 2 and 3 | 2 and 3 |
| The pairing of side 0 scaled by $1 + 10^{-6}$ | 1, 2, and 3 | 1, 2, and 3 | 1, 2, and 3 |
| The triangle of `*432` labelled with the orders and the area $\pi/30$ of `*532` | 4, 5, and 6 | Not applied | Not applied |
| The lune of the angle $2\pi/n$ labelled with the order $n$, for $n = 5$ and $n = 12$ | Not applied | Not applied | 5 and 6, for both orders |
| The vertices of the lune set to $(0, 1, 0)$ and $(0, 1, 0)$, which coincide, or to $(0, 0.5, 0)$ and $(0, -3, 0)$, which lie off the sphere | Not applied | Not applied | 3, for both pairs |
| The partner of side 0 set to side 1 | 2 | 2 | 2 |
| The centre set to its negation, outside the domain | 2 | 2 | 2 |
| Vertex 0 set to the zero vector | 2, 3, 4, 5, and 6, each with an infinite residual | 2, 3, 4, 5, and 6, each with an infinite residual | 2, 3, 4, 5, and 6, each with an infinite residual |

Check 4 passes on several of the spherical defects. No committed test prints the residuals below, which were printed once by hand:

- On the relabelled lunes, check 4 passes with the residuals $1.8 \times 10^{-16}$ and $3.2 \times 10^{-16}$.
- On the scaled pairing, check 4 passes with $1.000002 \times 10^{-6}$ at `*532`, $1.000003 \times 10^{-6}$ at `*722`, and $1.000002 \times 10^{-6}$ at `*55`.
- On the tilted wall, checks 2 and 3 fail with the residuals $7.064 \times 10^{-8}$ and $3.560 \times 10^{-8}$ at `*532`, $1.699 \times 10^{-7}$ and $8.675 \times 10^{-8}$ at `*722`, and $1.809 \times 10^{-7}$ and $9.511 \times 10^{-8}$ at `*55`, each at least 35 times the tolerance of $10^{-9}$. Check 4 passes with $1.901 \times 10^{-7}$ at `*532` and $1.505 \times 10^{-7}$ at `*722`, 10.5 and 13.3 times below its tolerance of $2 \times 10^{-6}$, and with $1.5 \times 10^{-14}$ at `*55`.

In the hyperbolic table, check 4 passes on the scaled pairing at all three triangles, below its tolerance of $2 \times 10^{-6}$, and checks 1, 2, and 3 catch that defect. The residual of check 4 is $1.000003 \times 10^{-6}$ at $(7, 3, 2)$, a figure that no committed test prints.

Check 3 exists for the row with the pairing left as constructed. The renderer tests a point with the covector and moves it with the pairing, so a domain whose covector and pairing disagree corrupts the image while the other five checks pass. The tilted wall does not pass through the endpoints of its side, so the first two values of check 3 exceed its tolerance.

The test also injects these defects into doubles, which pass every check before the defect: the kites of `732`, `442`, `532`, and `722`, the double of the rectangle of `2222` at `aspect` 1, and the doubles of the lunes of `44` and `55`, as each row names. On each domain of its row, exactly the checks of the row fail:

| Defect | Domains | Checks that fail |
| --- | --- | --- |
| The partner of side 0 set to side 0 itself | The kites | 2 and 4, each with an infinite residual |
| The kind of side 0 set to `Mirror`, with its pairing left as constructed | The kites | 2, 3, and 4, with an infinite residual of check 4, because the domain then mixes kinds |
| The pairing of side 0 replaced by its inverse | The kites | 2 and 4 |
| Side 1 given the normal and the covector of side 0 of the generating domain, as if the image were not taken | The kites | 2, with an infinite residual, and 3. Check 2 fails structurally, because the pairing of side 0 then moves the centre to the wrong side of the wall of its partner |
| A kite labelled with one order off by one: the kites of $(8, 3, 2)$, $(7, 4, 2)$, and $(7, 3, 3)$ labelled `732`, and the kites of $(4, 3, 2)$ and $(5, 2, 2)$ labelled `532` | The kites of these orders | 4, 5, and 6 |
| The copies of $W_0$ set to 1 | The kites and the double of `55` | 4 with an infinite residual, and 5 |
| The two halves of a split side merged into one side that is its own partner, with the kind `Rotation` | `722` and `2222` | 2 |
| The translation of sides 2 and 5 replaced by the half-turn about the midpoint of side 2, with the kind `Translation` kept | `2222` | 2 and 4, with no residual asserted |
| Both pairings replaced by the half-turn $\operatorname{diag}(-1, 1, -1)$ about the axis through the two vertices | `44` | 2 |
| The pairings of sides 0 and 1 replaced by the half-turn about $W_0$ | `732` | 2 and 4 |
| The kinds of sides 2 and 5 set to `Rotation` on `2222`, and of sides 0 and 1 set to `Translation` on `442` | `2222` at `aspect` 1 and `442` | 2, with an infinite residual |
| Vertex $W_3$ moved to $W_4 - (0.5, 0, 0)$, along the line of sides 3 and 4 and 0.5 outside the wall of side 5, which is not adjacent to it | `2222` at `aspect` 1 | 2, 3, 4, 5, and 6 |
| A NaN in the first coordinate of vertex 1 | Every double of this paragraph | 2, 3, 4, 5, and 6, each with an infinite residual |

No order of `532` can be raised by one, because $(6, 3, 2)$, $(5, 4, 2)$, and $(5, 3, 3)$ are not spherical, so the row of an order off by one lowers the orders of `532` where a spherical triangle exists. No such kite changes the order of corner 2 of `532`.

The row of `44` tests the wall term of check 2. The half-turn fixes both vertices, as the endpoint terms require of a pairing of that double, so only the wall term fails, and without it the domain passes all six checks. The unit tests print the residual of check 2: 1.307 on `44`, and 1.597 on `732`. The row of the half-turn on `2222` keeps the kind `Translation` on a pairing that is not a translation, which the structural test of DOM-27 covers, and its test asserts which checks fail and no residual. The row of the relabelled kinds tests the translation test of DOM-27.

The row of the moved vertex tests the convexity term of check 3. $W_3$ stays on the line of sides 3 and 4, so the endpoint terms of check 3 stay 0, and check 3 fails only through its convexity term. The unit test prints the residuals: 0 for check 1, which passes, and 3, 0.5, 3, $\pi$, and $\pi$ for checks 2 to 6. It asserts that each check clears its tolerance by at least a factor of 8, on the side of a pass or of a failure.

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

On a built domain, the third and fourth values of check 3 are 0, because the construction stores the pairing of a mirror as `hyperbolic::reflection(normal)` and the covector as `hyperbolic::covector(normal)`. Its residual is therefore the largest of its first two values, the side test of a vertex against its wall divided by the last coordinate of the vertex, and the convexity term. DOM-30 requires that the convexity term leaves the residual of every reflection domain as the table states, and the test "Checks at the worst cases" prints the largest residuals of this table for the reflection triangles. On a double, the collinear halves of a split side put a vertex on the line of the other half, so the term can add a rounding error there. Section 9.3 states how the accuracy rows of the geometry design relate to the inputs of this crate.

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

A spherical domain uses the same tolerances. The test "Spherical domains" of Section 12.1 prints the largest residual of each check over the 18 spherical domains of Section 9.5, and every domain passes every check:

| Check | Tolerance | Largest residual | Where | Margin |
| --- | --- | --- | --- | --- |
| 1. Isometries | $10^{-9}$ | $1.554 \times 10^{-15}$ | `*522` | $6.43 \times 10^{5}$ |
| 2. Pairings | $10^{-9}$ | $1.554 \times 10^{-15}$ | `*522` | $6.43 \times 10^{5}$ |
| 3. Walls | $10^{-9}$ | $5.551 \times 10^{-16}$ | `*522` | $1.80 \times 10^{6}$ |
| 4. Relators | $2 \times 10^{-6}$ | $3.097 \times 10^{-14}$ | `*(999)22` | $6.46 \times 10^{7}$ |
| 5. Angles | $10^{-9}$ | $2.038 \times 10^{-14}$ | `*(1000)(1000)` | $4.91 \times 10^{4}$ |
| 6. Area | $10^{-9}$ | $4.075 \times 10^{-14}$ | `*(1000)(1000)` | $2.45 \times 10^{4}$ |

A double uses the same tolerances. The tests "Checks at the worst cases" and "Rotation domains" of Section 12.1 print the largest residual of each check over the doubles of the sets of Section 9.6, and every double passes every check. Every margin is at least 8, as TEST-13 of the coding standard requires:

| Check | Tolerance | Hyperbolic kites: largest residual, where, margin | Euclidean doubles: largest residual, where, margin | Spherical doubles: largest residual, where, margin |
| --- | --- | --- | --- | --- |
| 1. Isometries | $10^{-9}$ | $1.819 \times 10^{-11}$, `(928)43`, 55.0 | $4.441 \times 10^{-16}$, `442`, $2.25 \times 10^{6}$ | $1.443 \times 10^{-15}$, `522`, $6.93 \times 10^{5}$ |
| 2. Pairings | $10^{-9}$ | $1.819 \times 10^{-11}$, `(928)43`, 55.0 | $5.551 \times 10^{-16}$, `442`, $1.80 \times 10^{6}$ | $1.443 \times 10^{-15}$, `522`, $6.93 \times 10^{5}$ |
| 3. Walls | $10^{-9}$ | $5.994 \times 10^{-15}$, `(923)65`, $1.67 \times 10^{5}$ | $4.441 \times 10^{-16}$, `442`, $2.25 \times 10^{6}$ | $8.882 \times 10^{-16}$, `222`, $1.13 \times 10^{6}$ |
| 4. Relators | $2 \times 10^{-6}$ | $1.338 \times 10^{-7}$, `(998)(981)(961)`, 15.0. Over the cycles of two vertices only: $1.094 \times 10^{-8}$, `(973)(972)(964)`, 182.8 | $6.661 \times 10^{-16}$, `442`, $3.00 \times 10^{9}$ | $3.097 \times 10^{-14}$, `(999)22`, $6.46 \times 10^{7}$ |
| 5. Angles | $10^{-9}$ | $9.090 \times 10^{-13}$, `(978)(977)(971)`, 1100.1 | $4.441 \times 10^{-16}$, `333`, $2.25 \times 10^{6}$ | $7.061 \times 10^{-14}$, `(1000)22`, $1.42 \times 10^{4}$ |
| 6. Area | $10^{-9}$ | $1.946 \times 10^{-12}$, `(997)(931)(906)`, 513.9 | 0 on every double | $4.938 \times 10^{-15}$, `(1000)22`, $2.03 \times 10^{5}$ |

The wall term of check 2 is below the largest residual of check 2 in every column. Over the doubles of Section 9.6, its largest value is $1.046 \times 10^{-12}$ at `(868)44`, a margin of 956, for the kites, $4.996 \times 10^{-16}$ at `442` for the Euclidean doubles, and $8.882 \times 10^{-16}$ at `522` for the spherical doubles. It adds nothing to a reflection domain.

The largest residual of check 4 of the kites is that of a cycle of one vertex, the rotation about a corner of the generating triangle, and equals the largest residual of check 4 of the reflection triangles. The tolerances of this section therefore hold for the doubles unchanged.

DOM-30. The stored values of a reflection domain and the residuals of its checks MUST NOT depend on the code that builds and checks the doubles, so that every figure of Sections 8.4 and 9.3 to 9.5 for a reflection domain still holds bit for bit, because the golden images of `escher-render` compare reflection domains byte for byte, and the generating domain of each double must equal the reflection domain of its starred signature. The test "Generating domain" of Section 12.1 checks the second part, and the tests of the reflection domains, which print the figures of these sections, check the first.

## 9. Numerical Robustness

### 9.1. Renormalization

- `hyperbolic::normalize_point` runs twice, in steps 6 and 10 of Section 6.1, each time on a sum of vertices.
- `hyperbolic::wall_through` divides each normal by its norm, and `euclidean::wall_through` divides each covector by the length of its first two coordinates (the geometry design).
- The Euclidean construction divides a sum of vertices by its last coordinate in steps 3 and 5 of Section 6.5.
- `spherical::normalize_point` runs twice for a spherical triangle, in steps 4 and 6 of Section 6.7, each time on a sum of vertices, and `spherical::wall_through` divides each normal by its length. The lune renormalizes nothing.
- The double of Section 6.8 renormalizes nothing. Its image vertices and normals are plain products of $R_1$ and the values of its generating domain.
- `verify()` renormalizes nothing (DOM-8).

DOM-10. The construction MUST NOT renormalize the vertices after step 8, after step 4 of Section 6.5, or in step 2 of Section 6.8, because a renormalized vertex lies on the hyperboloid but far from the true corner. Section 14 records the figures for $(1000, 1000, 1000)$, which no measurement of the code covers.

A vertex therefore leaves the hyperboloid by a small amount. Over the triples of the reference sweep of Section 9.3, $\lvert \langle V, V \rangle + 1 \rvert$ reaches $4.113 \times 10^{-5}$, at $(995, 987, 976)$. The wall through two vertices depends only on the plane that they span, so the normals are not affected.

### 9.2. Forms

- `hyperbolic::inverse` computes the geometric inverse $J M^t J$ (Section 6.1 of the geometry reference), never a general matrix inverse.
- The relators are evaluated in the corner frame (Section 8.2, check 4).
- The corner angles come from the distances between the vertices (Section 7).
- $\operatorname{arcosh} X$ in step 4 needs no special form. The smallest $X$ that the construction computes over the triples of the reference sweep of Section 9.3 is 1.040349, at `*732`.
- $\arccos X$ in step 2 of the spherical triangle is clamped to $[-1, 1]$. For `*n22` with a large $n$, $X_b$ comes close to 1, where the arccosine is ill-conditioned, and $\cos(\pi/2)$ is not 0 in `f64`, so Section 9.5 measures the triangle up to $n = 1000$.

### 9.3. Accuracy over the Supported Range

DOM-11. The values that `build_with` returns MUST lie within the bounds of this table, or for a Euclidean domain of Section 9.4, for a spherical domain of Section 9.5, and for a double also of Section 9.6, of the values of the exact polygon in the final position, because the renderer and `escher info` use them without a check, and the reference-arithmetic tests of Section 12.1 assert these bounds.

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

DOM-19. A test that asserts a quantity of a domain for which this document states no measured bound MUST set its bound and record it in this document as TEST-13 of the coding standard requires, over an input set that this document states, in the same change as the test, because Sections 8.4 and 9.3 to 9.6 are the one record of the tolerances of this crate, and a bound outside them has no measurement for a reviewer to check.

### 9.5. Accuracy of the Spherical Domains

The spherical domains of the tests are those of `*332`, `*432`, `*532`, and `*222`, and of `*n22` and `*nn` for $n$ = 2, 3, 5, 12, 100, 999, and 1000. Over these domains, the test "Spherical domains" of Section 12.1 compares each quantity of this table with the reference construction of Section 12.2, asserts each bound, and prints the largest error with its domain and its margin. The test builds `*222` twice, as `*222` and as `*n22` with $n = 2$, so it measures 18 domains with 47 sides. The bounds are choices of this design, set from the largest errors measured, and every margin is at least 8, as TEST-13 of the coding standard and DOM-19 require:

| Quantity | Measure | Bound | Largest error | Where | Margin |
| --- | --- | --- | --- | --- | --- |
| Vertex | Largest coordinate error | $4 \times 10^{-13}$ | $2.785 \times 10^{-14}$ | `*(1000)22` | 14.4 |
| Normal | Largest coordinate error | $10^{-13}$ | $7.764 \times 10^{-15}$ | `*(999)22` | 12.9 |
| Side length | Absolute error | $4 \times 10^{-13}$ | $3.892 \times 10^{-14}$ | `*(999)22` | 10.3 |
| Centre | Largest coordinate error against the reference centre | $2 \times 10^{-15}$ | $1.241 \times 10^{-16}$ | `*(1000)22` | 16.1 |
| Corner angle | Absolute error against $\pi/n_j$ | $2 \times 10^{-13}$ | $2.038 \times 10^{-14}$ | `*(1000)22` | 9.8 |
| Area | Absolute error against $2\pi\chi$ | $4 \times 10^{-13}$ | $4.075 \times 10^{-14}$ | `*(1000)(1000)` | 9.8 |

The vertices and the centre of a lune are exact values that the construction stores, and the dot product of its two vertices is exactly $-1$. The test asserts these equalities exactly, as TEST-13 of the coding standard allows, and compares the side lengths of a lune with $\pi$ within the bound of the side length.

### 9.6. Accuracy of the Rotation Domains

The vertices, normals, and covectors that a double takes from its generating domain are those of Sections 9.3 to 9.5 bit for bit (DOM-30), and so is its centre. This section bounds what the doubling adds: the image vertices, normals, and covectors, the pairings of DOM-25, and the measurements of Section 7.

The doubles of the tests are those of the cone triples of the set of 294 triples of Section 12.1, of `632`, `442`, `333`, and of `2222` at the 69 values of `aspect` of Section 9.4, and of `332`, `432`, `532`, `222`, and of `n22` and `nn` for $n$ = 2, 3, 5, 12, 100, 999, and 1000. The test "Rotation domains" of Section 12.1 compares each quantity of this table with the reference of Section 12.2, which builds the double itself, asserts each bound, and prints the largest error with its domain and its margin. The test "Compensated pairings" measures the rows of the pairing and of the isometry defect over the same doubles. The bounds are choices of this design, each the smallest value of the form 1, 2, or 4 times a power of 10 that is at least 8 times the largest error measured, except the area bound of the spherical doubles, which is the area bound $4 \times 10^{-13}$ of Section 9.5. Every margin is at least 8, as TEST-13 of the coding standard and DOM-19 require. The hyperbolic kites are the 294 doubles of the cone triples, the Euclidean doubles are 72, and the spherical doubles 18:

| Quantity | Measure | Hyperbolic kites: bound, largest error, where, margin | Euclidean doubles: bound, largest error, where, margin | Spherical doubles: bound, largest error, where, margin |
| --- | --- | --- | --- | --- |
| Image vertex | Largest coordinate error, relative to the largest coordinate for a hyperbolic domain | $4 \times 10^{-9}$, $3.234 \times 10^{-10}$, `(998)(998)(958)`, 12.4 | $10^{-14}$, $5.187 \times 10^{-16}$, `2222` at `aspect` 5.0397, 19.3 | $4 \times 10^{-13}$, $2.686 \times 10^{-14}$, `(999)22`, 14.9 |
| Image normal or covector | Largest coordinate error of the normal of a hyperbolic or spherical double, and of the covector of a Euclidean double | $10^{-8}$, $6.223 \times 10^{-10}$, `(995)(983)(970)`, 16.1 | $10^{-14}$, $5.187 \times 10^{-16}$, `2222` at `aspect` 5.0397, 19.3 | $2 \times 10^{-13}$, $2.329 \times 10^{-14}$, `(999)22`, 8.6 |
| Pairing | Largest entry error against the product, in the reference arithmetic, of the two reflections that the domain stores | $2 \times 10^{-13}$, $1.363 \times 10^{-14}$, side 2 of `(923)65`, 14.7 | $10^{-15}$, $5.794 \times 10^{-17}$, side 0 of `333`, 17.3 | $10^{-15}$, $5.485 \times 10^{-17}$, side 2 of `222`, 18.2 |
| Isometry defect of a pairing | The value of the isometry check of the geometry | $2 \times 10^{-10}$, $1.819 \times 10^{-11}$, side 2 of `(928)43`, 11.0 | $4 \times 10^{-15}$, $4.441 \times 10^{-16}$, side 0 of `632`, 9.0 | $2 \times 10^{-14}$, $1.443 \times 10^{-15}$, side 2 of `522`, 13.9 |
| Side length | Absolute error | $2 \times 10^{-9}$, $1.444 \times 10^{-10}$, `(985)(985)(971)`, 13.8 | $10^{-14}$, $6.419 \times 10^{-16}$, `2222` at `aspect` 5.0397, 15.6 | $4 \times 10^{-13}$, $3.914 \times 10^{-14}$, `(1000)22`, 10.2 |
| Corner angle | Absolute error against the copies times $\pi$ divided by the order | $10^{-11}$, $9.090 \times 10^{-13}$, `(978)(977)(971)`, 11.0 | $4 \times 10^{-15}$, $3.369 \times 10^{-16}$, `333`, 11.9 | $10^{-12}$, $7.073 \times 10^{-14}$, `(1000)22`, 14.1 |
| Flat vertex | Absolute error of the angle against $\pi$ | Not applicable, because no hyperbolic kite has a flat vertex | $10^{-15}$, $1.225 \times 10^{-16}$, `2222` at `aspect` 1, 8.2 | $10^{-12}$, $7.073 \times 10^{-14}$, `(1000)22`, 14.1 |
| Area | Absolute error against the area of the signature, or for a Euclidean double against the area of the reference polygon | $2 \times 10^{-11}$, $1.946 \times 10^{-12}$, `(997)(931)(906)`, 10.3 | $2 \times 10^{-15}$, $2.220 \times 10^{-16}$, `2222` at `aspect` 0.21197, 9.0 | $4 \times 10^{-13}$, $4.938 \times 10^{-15}$, `(1000)22`, 81.0 |

The test asserts its smallest numbers of samples. For the hyperbolic kites, these are 294 image vertices, 588 image normals, and 1176 sides. For the Euclidean doubles, they are 141 image vertices, 213 image covectors, 426 sides, and 138 flat vertices. For the spherical doubles, they are 11 image vertices, 29 image normals, 58 sides, and 12 flat vertices. The row of the image normal or covector compares the covector of a Euclidean double, because its normal is derived from its covector (step 3 of Section 6.8). The error of a flat vertex of a Euclidean double is measured against the exact $\pi$, so it includes the difference between the `f64` value of $\pi$ and $\pi$.

The pairing row compares with the product of the two `f64` reflections that the domain stores, because DOM-25 computes that product and not the product of the reflections of the reference. The test "Compensated pairings" also asserts that every entry of every pairing is that exact product rounded to the nearest `f64`, and prints how many entries differ from it, for the compensated and for the plain product:

| Doubles | Entries | Entries that differ, compensated product | Entries that differ, plain product |
| --- | --- | --- | --- |
| Hyperbolic kites | 10,584 | 0 | 4,050 |
| Euclidean doubles | 3,834 | 0 | 9 |
| Spherical doubles | 522 | 0 | 160 |

The isometry row is the defect that remains with the compensated product. Measurements that no test of the crate repeats compare the largest isometry defect of the pairings of a double with the compensated and with the plain product, for these doubles:

| Double | Compensated product | Plain product |
| --- | --- | --- |
| `(1000)32` | $9.095 \times 10^{-13}$ | $2.728 \times 10^{-12}$ |
| `(928)43` | $1.819 \times 10^{-11}$ | $1.819 \times 10^{-11}$ |
| `(868)44` | $1.455 \times 10^{-11}$ | $1.091 \times 10^{-11}$ |
| `(998)44` | $7.276 \times 10^{-12}$ | $1.455 \times 10^{-11}$ |
| `(923)65` | $7.276 \times 10^{-12}$ | $1.091 \times 10^{-11}$ |
| `(1000)(1000)2` | $2.220 \times 10^{-15}$ | $1.332 \times 10^{-15}$ |
| `(973)(972)(964)` | $7.105 \times 10^{-15}$ | $5.329 \times 10^{-15}$ |

Over the cone triples of the 30 named triples, the largest defect is $1.819 \times 10^{-11}$, at `(928)43`, for both products. At `(1000)32`, the defect of the compensated product equals that of the reflections that the domain stores. The compensated product therefore lowers the defect at some doubles and raises it at others, and does not lower the largest defect over those triples. The test "Compensated pairings" bounds the defect of the compensated product, which the construction uses, and with plain products it would fail on the entries that differ.

## 10. Errors

### 10.1. Structure

DOM-12. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"derive side presentation"` for an error of `side_presentation()` and `"build fundamental domain"` for every other error of this crate.
- `detail`: A private enum with one variant for each cause, which carries the data that the text of Section 10.2 shows: the canonical string of the signature, the order over the limit, the identity of a parameter from which its name and range derive, the error of `escher-geometry` that caused a numerical failure, or the error of `escher-group` that rejected a side presentation.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, because a derived `source()` can return only a field of `Error`, not the `escher-geometry` error that one variant of the detail holds, and a separate source field beside the detail would hold one fact in two fields. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `escher-geometry` error for a `Numerical` error that one caused, the `escher-group` error for an `InvalidPairing` error that one caused, and `None` otherwise. No type from a dependency appears in the public surface of the error (ERR-2).

`InvalidPairing` is an error and not a panic for the reason that `Numerical` is: no domain that `build_with` returns produces it, but `side_presentation()` runs on any value of the type, and `escher info` prints its result (DOM-35).

`Numerical` is an error and not a panic. No supported signature is known to produce it, but the construction calls fallible functions of `escher-geometry`, and a panic would stop `escher info`, so returning an error is the sounder option (ERR-5 of the coding standard, Hard Rule 7 of `AGENTS.md`).

### 10.2. Kinds and Text

DOM-13. `Display` for `Error` MUST be `cannot derive side presentation: ` for the kind `InvalidPairing` and `cannot build fundamental domain: ` for every other kind, followed by the text of the detail in this table, because `escher-cli` prints it and its snapshots fix it.

| Kind | When | Detail carries | Text after the prefix |
| --- | --- | --- | --- |
| `InvalidPairing` | DOM-35 | The cause: mixed kinds of side, a walk that does not close, a cycle with unequal orders or copies, or the `escher-group` error | `the domain has both mirror and paired sides`, `the side pairings do not close a vertex cycle`, `a vertex cycle joins corners of different orders`, or `the side relators do not form a presentation` |
| `UnsupportedSignature` | The predicate of Section 4 fails, which includes every general non-compact signature (`DEC-19`) | The canonical string | `the signature 3*2 is not supported`, with the canonical string of the signature in place of `3*2` |
| `LimitExceeded` | A corner order of a supported signature exceeds 1000 | The largest corner order | `order 1001 is larger than 1000`, with the largest corner order in place of `1001` |
| `Numerical` | Step 11 of Section 6.1, step 7 of Section 6.5, step 7 of the triangle or step 4 of the lune of Section 6.7, or step 8 of Section 6.8 | The `escher-geometry` error, if one caused it | `the construction is numerically degenerate` |
| `UnknownParameter` | Step 3 of DOM-18 | The canonical string | `the signature *632 has no shape parameter with one of the given names`, with the canonical string of the signature in place of `*632` |
| `ParameterOutOfRange` | Step 4 of DOM-18 | The identity of the parameter, a private enum from which the name, the minimum, and the maximum of its `ParameterSpec` derive, so that `Error` keeps `Eq` | `the parameter aspect is outside its range of 0.125 to 8`, with the name, the minimum, and the maximum in place of `aspect`, `0.125`, and `8` |

The text `a vertex cycle joins corners of different orders` also covers a cycle whose copies do not sum to 2. The texts of `InvalidPairing` name no side and no number, because no supported input reaches them, and the tests that do through private access assert the kind and the prefix.

The texts of `UnknownParameter` and `ParameterOutOfRange` contain no name or value of a setting, because a caller's text may hold any character, and the printed name is that of the `ParameterSpec`. They write each number with the `{}` format of `f64`, which prints the two ends of the range of `aspect` as `0.125` and `8`.

So `FundamentalDomain::build` on `3*2` returns an `Error` whose `kind()` is `UnsupportedSignature` and whose `Display` text is exactly `cannot build fundamental domain: the signature 3*2 is not supported`. On `*(1001)32` it returns `LimitExceeded` with the text `cannot build fundamental domain: order 1001 is larger than 1000`. What `escher-cli` prints for either, and its exit status, are for the command-line design to state.

Every fixed text is in lower case and has no trailing punctuation (ERR-4 of the coding standard). The canonical string contains only printable ASCII (Section 7.3 of the signature design), so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

## 11. Resource Limits

The crate limits the largest corner order, which `build_with` checks at step 2 of DOM-18, and the value of each shape parameter, which it checks at step 4. Section 16 states the value and its reason, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 12. Testing

### 12.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-domain/tests/domain/main.rs`, with the modules `common`, `reference`, `triangles`, `regular`, `properties`, `unsupported`, `euclidean`, `parameters`, `spherical`, `rotation`, and `presentation` (TEST-10). Its root has a `//!` line. The snapshot tests of the domain summary live in `escher-cli`, and the command-line design specifies them.

The module `common` defines the sets of triples that the tests use:

- The plan triples: $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(1000, 3, 2)$. The plan writes $(3, 3, 4)$, whose canonical signature is `*433`.
- The 24 worst cases: $(995, 983, 970)$, $(994, 991, 970)$, $(997, 996, 970)$, $(997, 980, 973)$, $(992, 990, 979)$, $(995, 987, 976)$, $(928, 4, 3)$, $(999, 7, 6)$, $(998, 4, 4)$, $(1000, 4, 4)$, $(1000, 1000, 2)$, $(1000, 1000, 1000)$, $(868, 4, 4)$, $(923, 6, 5)$, $(998, 981, 961)$, $(998, 998, 958)$, $(997, 931, 906)$, $(992, 991, 975)$, $(989, 976, 974)$, $(978, 977, 971)$, $(985, 985, 971)$, $(973, 972, 964)$, $(979, 979, 971)$, and $(987, 984, 970)$. They include every triple at which the sweeps of Sections 8.4 and 9.3 have a largest value.
- The 30 named triples: the plan triples and the worst cases.
- The orders up to 12: all 269 hyperbolic triples $n_0 \ge n_1 \ge n_2$ with orders up to 12.
- The set of 294 triples: the orders up to 12 and the 25 named triples with an order above 12.
- The cone triples: the same triples read as the orders of rotation signatures, such as `732` for $(7, 3, 2)$, whose domains are kites.

Each test names the item of the field "Tests and acceptance" of Section 5.2 of the plan that it serves, or of Section 5.4, 5.5, 5.6, or 5.7 of the plan where the item says so. The tests "Triangles", "Third side", "Checks at the worst cases", "Relator agreement", and the two tests of $\{7, 3\}$ also assert their smallest number of samples, and print each largest value with its triple and its margin. The test harness captures that output for a test that passes:

| Test | Plan item | Kind and place | Asserts |
| --- | --- | --- | --- |
| Triangles | Reference arithmetic | Reference test, `triangles` | Vertices, normals, side lengths, the centre, corner angles, and the area are within the bounds of each column of Section 9.3, against the reference of Section 12.2, for the set of 294 triples |
| Third side | Reference arithmetic | Reference test, `triangles` | For the set of 294 triples: in the reference, the side from vertex 0 to vertex 1 of the standard position agrees within $10^{-60}$ with the side opposite vertex 2 by the law of cosines of Section 10.2 of the geometry reference, which checks the placement. The distance from vertex 0 to vertex 1 of the built domain, which the construction never sets, is within the bounds of the side length of Section 9.3 for the orders up to 12 and for the set |
| Checks at the worst cases | Reference arithmetic | Integration test, `triangles` | For the set of 294 triples, and for its cone triples: each check of Section 8.2 passes and `verify().passed()` is true. It prints the largest residual of each check, for the reflection triangles and for the kites apart, which Section 8.4 states, and for the kites the largest residual of check 4 over the cycles of two vertices |
| Edge of $\{7, 3\}$ | Numeric checks for $\{7, 3\}$ | Reference test, `regular` | Twice the distance from vertex 1 to vertex 2 of `*732` is within $2 \times 10^{-13}$ of equation 29 of [HT] at $(p, q) = (7, 3)$ |
| Circumradius of $\{7, 3\}$ | Numeric checks for $\{7, 3\}$ | Reference test, `regular` | The length of `hyperbolic::to_disk(hyperbolic::inverse(hyperbolic::translation(V0)) * V1)` of `*732` is within $10^{-13}$ of equation 24 of [HT] at $(p, q) = (7, 3)$ |
| Random triples | Properties | `proptest`, `properties` | For orders from 2 to 1000 with $1/p + 1/q + 1/r < 1$, drawn with half of the cases from orders 900 to 1000, each read once as a reflection signature and once as a rotation signature: `build` succeeds, `verify().passed()`, the vertices are counterclockwise, every vertex is inside or on every side within the tolerance of check 3, and the centre is within $10^{-9}$ of `hyperbolic::ORIGIN` |
| Relator agreement | Properties | Reference test, `properties` | For the 30 named triples, with the residual of each frame computed in the reference from the exact power of the `f64` matrices and divided by $n_j$: the residual of check 4 is within $10^{-9}$ of the largest residual in the corner frame, and at each corner the residual in either frame is at most 2.0 times the residual in the other frame times 9 times the square of the largest absolute entry of $F$. This tests the assumption on conjugates of Section 15.1. Measured: $1.459 \times 10^{-11}$ against $10^{-9}$, a margin of 68.5, at $(1000, 1000, 2)$, and 0.217 against 2.0, a margin of 9.2, at corner 1 of $(1000, 1000, 2)$ |
| Relators in the domain frame | None | Reference test, `properties` | The figures of the table of Section 8.2. At $(998, 981, 961)$, the largest exact residual is at least $2 \times 10^{-6}$ in the domain frame and at most $2 \times 10^{-6}$ in the corner frame. At $(1000, 3, 2)$, it is at most $10^{-10}$ in the domain frame and at most $10^{-12}$ in the corner frame. It prints both residuals for both triples |
| Failing domains | Failure paths | Unit tests in `verify.rs` | The domains of $(7, 3, 2)$, $(4, 4, 4)$, and $(1000, 3, 2)$, and the spherical domains and the doubles of Section 8.3, built through private access, pass every check. For each defect of Section 8.3, on the domains of its row, exactly the checks of the row fail and `passed()` is false. For a row that names an infinite residual, each failed check has the residual `f64::INFINITY`, and no check has a NaN residual. The rows with a vertex removed and the relabelled rectangle give domains whose numbers of sides, vertices, and corner orders disagree, and `verify()` returns on them without a panic (DOM-8) |
| Check results | Failure paths | Unit tests in `verify.rs` | A `Check` built through private access with a NaN residual fails, a residual equal to the tolerance passes, and an infinite residual fails |
| Unsupported | Failure paths | Table test, `unsupported` | `*22222`, `3*2`, `3*3`, `*3333`, `3222`, `22222`, `4*3`, `2*22`, `*(inf)32`, `(inf)32`, `*(inf)(inf)`, `(inf)(inf)`, `*22(inf)`, `1`, `*`, `o`, `*732*`, `7*32`, and `x732` return `UnsupportedSignature` with the text of Section 10.2. `*(1001)32`, `*(1001)22`, `*(1001)(1001)`, `(1001)32`, `(1001)22`, and `(1001)(1001)` return `LimitExceeded` with its text, `*(1000)32`, `*(1000)22`, `*(1000)(1000)`, `(1000)32`, `(1000)22`, and `(1000)(1000)` build, and `*(1001)(1001)(1001)(1001)` and `(1001)(1001)(1001)(1001)` return `UnsupportedSignature` |
| Limit | Failure paths | Unit tests in `domain.rs` | With the private limit constant: `*(1000)32` and `(1000)32` build, `*(1001)32` and `(1001)32` return `LimitExceeded`, and `*(1001)(1001)(1001)(1001)` returns `UnsupportedSignature`, because support is checked before the limit |
| Sides and vertices | None | Unit test in `domain.rs` | For `*732`, each side is its own partner, and `hyperbolic::side` of its covector at each of its two vertices is below $10^{-12}$ in absolute value |
| Corner order and area | None | Unit tests in `triangle.rs` | The private predicate of Section 4 gives the orders $(7, 3, 2)$ and the area `PI / 42.0` for `*237`, the Euclidean triangle with the orders $(6, 3, 2)$ for `*236`, the rectangle for `*2222`, the spherical triangle with the orders $(5, 3, 2)$ and the area `PI / 30.0` for `*532`, the lune of the order 5 for `*55`, the hyperbolic kite with the orders $(7, 3, 2)$ and the area `PI / 21.0` for `237`, the double of the rectangle for `2222`, the double of the lune of the order 2 for `22`, and nothing for `3*2`, `*3333`, `3222`, and `*(inf)32`. The area is compared exactly, because the numerator 1 and the denominators 42 and 21 convert exactly and multiplying by 1 is exact, so IEEE 754 arithmetic computes the same quotient as `PI / 42.0` (TEST-13 of the coding standard) |
| Conversion | None | Unit test in `triangle.rs` | The private signed conversion of step 1 of Section 6.1 gives exactly 0, $-1$, and $-4294967297$ for those integers, $2^{53}$ for $2^{53} + 1$ by ties to even, $2^{63}$ for `i64::MAX`, and $-2^{63}$ for `i64::MIN`. The unsigned conversion gives exactly 1 and $4294967297$ for those integers, and $2^{64}$ for `u64::MAX`. The test compares exactly, because the sum of the two parts is the only rounding, and IEEE 754 arithmetic fixes the result of that rounding to nearest, bit for bit (TEST-13 of the coding standard) |
| Numerical | Failure paths | Unit test in `triangle.rs` | The private construction of steps 3 to 11, given the spherical orders $(5, 3, 2)$, fails with the `escher-geometry` error of kind `NotAPoint`, which `build` reports as `Numerical`, because the law of cosines then gives a hyperbolic cosine below 1 |
| Any signature | Failure paths | `proptest`, `properties` | For any text that parses as a signature, `build` returns without a panic, and a returned domain passes `verify()`. The generated signatures have up to four cone points, so that kites and `2222` occur |
| Error text | Failure paths | Unit tests in `error.rs` | The `Display` of `InvalidPairing` starts with `cannot derive side presentation: ` and that of every other kind with `cannot build fundamental domain: `. Each is in lower case apart from the canonical string, and is the text of Section 10.2. `kind()` matches each cause, and `source()` returns the `escher-geometry` error only for a `Numerical` error that one caused, and the `escher-group` error only for an `InvalidPairing` error that one caused |
| Euclidean table | Relators, of Section 5.4 of the plan | Table test, `euclidean` | `*632`, `*442`, `*333`, and `*2222` build and pass `verify()`. They have 3, 3, 3, and 4 sides and vertices, each side its own partner, and the corner angles of 30, 60, and 90 degrees, 45, 45, and 90 degrees, 60 degrees three times, and 90 degrees four times, within the bound of Section 9.4. Every other spelling of `*632` and `*442`, such as `*236`, builds the domain of its canonical form, with every vertex within twice the vertex bound of Section 9.4 |
| Euclidean domains | Relators, of Section 5.4 of the plan | Reference test, `euclidean` | Each quantity of Section 9.4 other than the lines of SymmHub is within its bound against the reference of Section 12.2, over the Euclidean domains of Section 9.4. Each check passes, and the test prints the largest residual of each check with its margin, which Section 8.4 records (DOM-19) |
| SymmHub lines | None | Reference test, `euclidean` | For each Euclidean triangle, every wall agrees within the bound of Section 9.4, up to sign, with one of the three lines that SymmHub's code gives for the triangle (Section 4.3 of `docs/research/rendering.md`), $y = 0$, $-x \sin a_0 + y \cos a_0 = 0$, and $x \sin a_1 + y \cos a_1 = e$ with $a_0 = \alpha_2$, $a_1 = \alpha_0$, and $e = \sin\alpha_0$, each moved by the translation of step 4 of Section 6.5, and each line matches a different side. It prints which line matches each side. This tests the candidate vertices of `UNV-28` |
| Exact rectangle | None | Unit test in `polygon.rs` | At `aspect` 1 and 4, the vertices and covectors of the rectangle are exactly those of Section 9.4 |
| Rectangle aspect | Properties | `proptest`, `euclidean` | For a value of `aspect` within its range, the rectangle passes `verify()`, has four right angles within the bound of the corner angle of Section 9.4, and the length of side 0 divided by that of side 1 is the value within the bound of the aspect of Section 9.4 |
| Parameters | Range errors, of Section 5.4 of the plan | Table test, `parameters` | `parameters` gives one `ParameterSpec` for `*2222`, with the name `aspect`, the range 0.125 to 8, and the default 1, and the same for `2222`, and an empty list for `*632`, `*442`, `*333`, `*732`, `*532`, `732`, and `532`. `build_with` returns `ParameterOutOfRange` for `aspect` set to 0.1249, 8.0001, 0, $-1$, NaN, and both infinities, on `*2222` and on `2222`, and builds both at 0.125 and 8. It returns `UnknownParameter` for the names `Aspect`, `aspect ` with a trailing space, and the empty name on `*2222`, and for `aspect` on `*632`, `*732`, and `732`. An unsupported signature with a setting returns `UnsupportedSignature`. For `*2222` and `*632`, `build` gives the domain of `build_with` with no setting, with every vertex within twice the vertex bound of Section 9.4, and `set` returns the value that it replaces. Each error has the text of Section 10.2 |
| Spherical table | None | Table test, `spherical` | `*332`, `*432`, `*532`, `*222`, `*722`, `*22`, and `*55` build and pass `verify()`, with 3 sides and vertices for a triangle and 2 for a lune, each side its own partner, `parameters` empty, and the corner angles $\pi/n_j$ within the bound of Section 9.5. Each triangle has $(V_0 \times V_1) \cdot V_2 > 0$. Every other spelling of a signature, such as `*22n` for `*n22` and `*221` for `*22`, builds the domain of its canonical form, with every vertex within twice the vertex bound of Section 9.5 |
| Spherical domains | None | Reference test, `spherical` | Each quantity of Section 9.5 is within its bound against the reference of Section 12.2, over the spherical domains of Section 9.5. Each check passes, and the test prints the largest residual of each check with its margin, which Section 8.4 records (DOM-19) |
| Exhaustive certificate | Exhaustive certificate, of Section 5.5 of the plan, and Spherical orders, of Section 5.6 of the plan | Integration test, `spherical` | For `*332`, `*432`, `*532`, and `*n22` and `*nn` with $n$ from 2 to 12, and for the rotation signatures `332`, `432`, `532`, and `n22` and `nn` with $n$ from 2 to 12: a breadth-first enumeration of the products of the pairings, which merges two matrices whose entries agree within $10^{-9}$, has exactly as many elements as `group_order()` of the signature. The images of the centre under these elements are pairwise further apart than the floor of the test, which is $2 \times 10^{-2}$ for a reflection domain and $4 \times 10^{-2}$ for a double (table below the test descriptions). Each image of the centre folds back to the centre within the tolerance of check 3, for a reflection domain in as many steps as its breadth-first depth, and for a double in at least as many steps as its breadth-first depth, by the fold of the test that this section describes below the table, and no such fold reaches the default largest number of folds of `escher-render`, 1500 (Section 4.1 of `docs/design/render.md`). For each lune and each double of a lune, a dense sample folds into the domain within that number of folds. The sample is 2000 points of a Fibonacci lattice on the sphere, the two vertices, the points $10^{-3}$, $10^{-10}$, $10^{-30}$, $10^{-100}$, $10^{-200}$, and $10^{-300}$ from each vertex in five directions, the centre and its negation, and 33 points on each side, each also at the offsets $\pm 10^{-12}$ and $\pm 10^{-6}$ along the normal of the side |
| Exhaustive certificate at the limit | Exhaustive certificate, of Section 5.5 of the plan, and Spherical orders, of Section 5.6 of the plan | Integration test, `spherical` | The assertions of "Exhaustive certificate" for `*(1000)22`, `*(1000)(1000)`, `(1000)22`, and `(1000)(1000)`, with the floor $2 \times 10^{-4}$ for the reflection domains and $4 \times 10^{-4}$ for the doubles |
| Rotation table | The checklist as `verify()`, of Section 5.6 of the plan | Table test, `rotation` | `732`, `542`, `444`, `433`, `(12)32`, `632`, `442`, `333`, `2222`, `332`, `432`, `532`, `222`, `722`, `22`, and `55` build and pass `verify()`. Each has the vertices, the side pairs, the kinds, the orientation `Preserving`, the flat vertices, and the copies of the table of Section 6.8, `reverses_orientation()` is false, the orientation of Section 5.2 holds, and the corner angles are the copies times $\pi$ divided by the order within the corner-angle bound of Section 9.6. A vertex is flat when its angle is within that bound of $\pi$, and the flat vertices found are exactly those of the table. Each image vertex is within twice the image-vertex bound of Section 9.6 of $R_1$ applied to its vertex of the generating domain, relative to the largest coordinate when that exceeds 1. The spellings `237`, `372`, `236`, `244`, `235`, and `227` build the domain of their canonical form, with every vertex within twice the image-vertex bound of Section 9.6. The plan triples, `*632`, `*442`, `*333`, `*2222`, `*532`, and `*55` have `reverses_orientation()` true |
| Generating domain | None | Table test, `rotation` | For each signature of "Rotation table" and its starred signature, and for `2222` and `*2222` at the values 0.125, 1.618, and 8 of `aspect`: the sides of the double that are sides of $F$, and the vertices that are vertices of $F$, have the normal, the covector, and the coordinates of the domain of the starred signature bit for bit, and the centres are equal bit for bit (DOM-30) |
| Rotation domains | The checklist as `verify()`, of Section 5.6 of the plan | Reference test, `rotation` | Each quantity of Section 9.6 other than the pairing and the isometry defect is within its bound against the reference of Section 12.2, over the doubles of Section 9.6. Each check passes, and the test prints the largest residual of each check with its margin, which Section 8.4 records (DOM-19) |
| Cycle sets | Cycle relators, of Section 5.6 of the plan | Unit test in `verify.rs` and reference test, `rotation` | The walk of DOM-28 finds the cycles and the orders of the table of Section 6.8 for each double of "Rotation table", including `222`, `22`, and `2222`. For the cone triples of the 30 named triples, the walk of a kite starts its cycles at the vertices 0, 1, and 2, and the residual of check 4 is within $10^{-9}$ of the largest residual of the cycles computed in the reference from the exact power of the `f64` cycle transformation, as "Relator agreement" does. Measured: $2.928 \times 10^{-11}$ against $10^{-9}$, a margin of 34.2, at `(995)(987)(976)` |
| Compensated pairings | None | Reference test, `rotation` | For the doubles of Section 9.6, every entry of every pairing equals the exact product, in the reference arithmetic, of the two reflections that the domain stores, rounded to the nearest `f64` with ties to even (DOM-25). The test decides this by comparing the exact product with the exact midpoints between the entry and its two neighbouring `f64` values. Each pairing is within the bound of the pairing row of Section 9.6 of that exact product, and its isometry defect is within the bound of its row. The test prints the number of entries that differ from the rounded exact product, for the compensated and for the plain product, and both largest errors |
| Compensated product | None | Unit tests in `double.rs` | The error-free product of $(1 + 2^{-30})^2$ gives the rounding error $2^{-60}$ exactly. The compensated dot product keeps a sum that plain arithmetic cancels. The product of two matrices whose exact product is representable is that product exactly. The expected constants are built from exact quotients such as `1.0 / 1_073_741_824.0`, because the interpreter of Miri perturbs `powi` |
| Random rotation domains | Properties | `proptest`, `rotation` | For a value of `aspect` within its range, the double of `2222` passes `verify()`, has the angles $\pi$ at $W_0$ and $W_3$ and $\pi/2$ at the other four vertices within the bound of Section 9.6, and has the area 2 within the bound of the area |
| Side presentation table | None | Table test, `presentation` | The generator names and the relator text of `*532`, `532`, `2222`, and `22` are those of the table of Section 5.4, in the text of the group design. For `*732`, `732`, `*2222`, `*632`, `632`, `*55`, and `55`, the presentation has one generator for each side, named `s0` upward, and as many relators as Section 5.4 gives: twice the number of sides for a reflection domain, and the number of pairs plus the number of vertex cycles for a double. Every domain of "Rotation table" and its starred signature gives `Ok` |
| Side orders | Side presentations give the same orders, of Section 5.7 of the plan | Integration test, `presentation` | For `*332`, `*432`, `*532`, `332`, `432`, and `532`, for `*n22`, `*nn`, `n22`, and `nn` with $n$ from 2 to 12, and for `*(1000)22`, `*(1000)(1000)`, `(1000)22`, and `(1000)(1000)`: `enumerate` of `escher-group` on the side presentation, over the trivial subgroup, with `CosetLimit::DEFAULT`, returns a table whose `index()` equals `group_order()` of the signature. It prints the time of the slowest case, which this document records once measured |
| Side order of `22` | Side presentations give the same orders, of Section 5.7 of the plan | Integration test, `presentation` | The assertion of "Side orders" for `22` alone, so that one enumeration of a side presentation runs under Miri |
| Relators evaluate | None | Integration test, `presentation` | For the 30 named triples, each read as a reflection and as a rotation signature, for `*632`, `*442`, `*333`, `*2222`, `632`, `442`, `333`, and `2222`, and for the spherical signatures of "Exhaustive certificate" and "Exhaustive certificate at the limit": every relator of the side presentation evaluates to the identity under the convention of DOM-32. Each letter $s_i$ is the matrix $P_i^{-1}$, by `hyperbolic::inverse` or `spherical::inverse`, and for a Euclidean domain by the matrix inverse of `glam`. The base is multiplied left to right. A corner relator or a cycle relator is evaluated as check 4 evaluates it: for a hyperbolic domain in the corner frame of the vertex of its corner or of the start vertex of its cycle, with the power by successive multiplication and the residual divided by the exponent, against the tolerance of check 4. A square and a pair relator are evaluated with no frame, as the residual of check 2, against the tolerance of check 2. It prints the largest residual of each kind of relator, with its signature and its margin, which this document records once measured (DOM-19) |
| Long cycle word | None | Unit test in `presentation.rs` | A domain built through private access, with side pairings that preserve orientation and a vertex cycle that visits at least three sides, gives a cycle relator whose base evaluates under the convention of DOM-32 within the tolerance of check 4, while the reversed base, and the base under the opposite convention in which $s_i$ is $P_i$, each give a residual of more than 8 times that tolerance. The code branch chooses the domain and records it in this row |
| Index two | None | Integration test, `presentation` | For the reflection domains of the plan triples, of `*632`, `*442`, `*333`, and `*2222`, and of `*532`, `*432`, `*332`, `*722`, and `*55`: `enumerate` of the side presentation over the subgroup generated by the words $s_j s_{j+1}$ of the corners, with `CosetLimit::DEFAULT`, returns a table of index 2. This is evidence for the side presentation of an infinite group by a finite enumeration, and not a source. That these words generate the subgroup of index 2 is unverified (Section 14) |
| Invalid pairing | Failure paths | Unit tests in `presentation.rs` | Domains built through private access: a reflection triangle with one side turned into a side of the kind `Rotation` paired with another, a double whose partners make the walk of DOM-28 run past $m$ steps, and a double with the order of one vertex of a cycle of two vertices changed. Each gives `InvalidPairing` with its text of Section 10.2, and none panics |
| Reference self-check | None | Reference test, `reference` | The four known values of Section 12.2 |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs` | Each public type is `Send`, `Sync`, and `'static` (CONC-1 of the coding standard) |
| Doctests | None | Doctests | The examples of the crate-level and item documentation |

DOM-22. The tests "Exhaustive certificate" and "Exhaustive certificate at the limit" MUST cover every lune of their sets and MUST stay while `UNV-29` is open, because no source makes the lune a fundamental domain (Section 6.7), and these tests are the only evidence that its two reflections generate a group of order $2n$ whose copies of the lune cover the sphere.

DOM-31. The tests "Exhaustive certificate" and "Exhaustive certificate at the limit" MUST cover every spherical double of their sets, `332`, `432`, `532`, `n22`, and `nn`, and MUST stay while `UNV-22` or `UNV-30` is open, because no statement in orbifold notation gives the signature of the group that the pairings of a double generate, no source makes the double of a lune a fundamental domain, and the enumeration of the group with `group_order()` elements and the dense sample are the only evidence for both on the sphere.

DOM-36. The test "Relators evaluate" MUST cover every domain of its sets, hyperbolic, Euclidean, and spherical, reflection and rotation, and the test "Long cycle word" MUST exist, and both MUST stay while `UNV-32` is open, because no source states the cycle relators as words in the convention of DOM-32, and a reversed relator or the opposite convention presents a group of the same order, which no enumeration detects.

The two tests establish different things. "Relators evaluate" establishes that every relator of every domain of its sets evaluates to the identity under the convention of DOM-32, within the tolerances of checks 2 and 4. It does not fix the convention: every vertex cycle of these domains visits one or two sides (Section 5.4), so the reversed words and the opposite convention pass it with the same residuals. Its largest residuals are to be measured, and Section 14 records those of a prototype. "Long cycle word" establishes the convention numerically on one domain with a cycle of at least three sides, where the reversed word and the opposite convention fail. That test is a proposed default of this design. The alternative is to defer it to the first milestone that builds a domain with such a cycle, and to rely on "Relators evaluate" alone until then (Section 15.3). Neither test closes `UNV-32`, which a source or a Lean proof closes.

"Relators evaluate" evaluates a hyperbolic corner or cycle relator in the corner frame of check 4, because without the frame the residual at the largest orders fails the tolerance of check 4 by far, as the prototype figures of Section 14 show.

The fold of the two certificate tests is a private copy in the tests of this crate. For every finite point other than the zero vector it applies the same rule as the fold of `escher-render` (Section 5.3 of `docs/design/render.md`). It differs only on degenerate input: it has no stop for a point that is not finite or for the zero vector, and it treats a NaN side value as inside the side, where the fold of `escher-render` returns an unconverged result for each of these. Side $i$ is violated at a point $x$ when $s_i > \tau \, \sigma_i \, \lambda(x)$, where $s_i$ is `spherical::side` of the covector of the side at $x$, $\tau = 4 \times 10^{-15}$ is a copy of the constant of `escher-render`, $\sigma_i = \sum_j \lvert c_{ij} \rvert$ for the covector $c_i$, and $\lambda(x) = \max_j \lvert x_j \rvert$. The fold applies the pairing of the first violated side, and stops when no side is violated. A fold that applies the pairing of the first side with a positive side value does not stop for a point on a wall. On `*33`, the point $(0.14514, 0.95694, 0.25139)$ of side 0 has the side value $+2.78 \times 10^{-17}$, and the reflection maps it to itself bit for bit, so such a fold cycles until the cap. The images of the centre lie far from every wall, so the tolerance does not change their depth.

The floors are set from the smallest distance measured between two images of the centre, with a margin of at least 8 as TEST-13 of the coding standard requires:

| Test | Floor | Smallest distance | Where | Margin |
| --- | --- | --- | --- | --- |
| Exhaustive certificate | $2 \times 10^{-2}$ | 0.2168 | `*532` | 10.8 |
| Exhaustive certificate at the limit | $2 \times 10^{-4}$ | $2.810 \times 10^{-3}$ | `*(1000)22` | 14.0 |
| Exhaustive certificate, doubles | $4 \times 10^{-2}$ | 0.4472 | `532` | 11.2 |
| Exhaustive certificate at the limit, doubles | $4 \times 10^{-4}$ | $5.620 \times 10^{-3}$ | `(1000)22` | 14.0 |

At the limit, `*(1000)22` has the group order 4000 and `*(1000)(1000)` the order 2000. The largest breadth-first depth there is about 1001, at `*(1000)22`, below the default largest number of folds of 1500.

The claim that a fold returns in its breadth-first depth and never reaches the cap holds for the images of the centre of a reflection domain, which the tests assert. For a double, the fold can take more steps than the breadth-first depth, so the tests assert only that it takes at least as many steps and converges within the cap. Each test prints the largest number of steps above the depth over its doubles: at most 3, at `532`, for orders up to 12, and 0 at the limit. It does not hold for every point: near a vertex of very high order, a fold can fail to converge, which belongs to the unverified statements on termination, `UNV-1` and, for a double, `UNV-2` (Section 14).

The plan item "Properties" also names the inverse $J M^t J$ and the round trip of the disk map. Those are properties of `escher-geometry`, and the geometry design tests them.

The expected values for $\{7, 3\}$ come from the split of the polygon into $2p$ right triangles (Section 10.1 of the geometry reference), which puts the centre of the polygon at vertex 0 of `*732`, a vertex of the polygon at vertex 1, and the midpoint of an edge at vertex 2. The tests evaluate equations 29 and 24 of [HT] (Section 10.4 of the geometry reference) in the reference arithmetic, with the curvature radius $\ell = 1$. Computed from those equations, the edge length is 0.566256306735315 and the circumradius in the disk is 0.300742618746379. The tests measure an error of $1.548 \times 10^{-16}$ in the edge against its bound of $2 \times 10^{-13}$, and of $2.025 \times 10^{-16}$ in the circumradius against its bound of $10^{-13}$. No value that Table 2 of [HT] prints serves as an expected value, because that table and equation 29 disagree on the edge length of $\{7, 3\}$ (item 23 of Section 15.1 of the geometry reference).

DOM-14. A test MUST NOT take an expected value from Table 2 of [HT], because the research records that the table and equation 29 disagree for $\{7, 3\}$, and a test built on the wrong one passes a wrong domain or fails a correct one.

The property tests commit their regression files (TEST-9 of the coding standard).

### 12.2. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture, with the precision and tolerances that this section states:

- Structure. A module `reference` in the test binary holds points as arrays of three high-precision numbers and writes each formula componentwise from the research documents. It calls no function of `escher-domain`, `escher-geometry`, or `glam`.
- Independence. The reference builds the standard position from $\cosh L$ and $\sqrt{\cosh^2 L - 1}$ without $\operatorname{arcosh}$, and centres the triangle with $\cos\varphi = c_1/h$, $\sin\varphi = c_2/h$, $\sinh\rho = h$, and $\cosh\rho = c_3$, where $h$ is the length of the first two coordinates of the centre, read from the polar form of Section 8.3 of the geometry reference. The crate takes the other path in both places, so a transcription error is not shared. For a Euclidean domain, the reference computes the vertices of steps 1 to 5 of Section 6.5 componentwise, and the line of each side as $P(a, t)$ of Section 12 of the geometry reference, with $a$ the unit vector perpendicular to the side and $t = a \cdot V$ for an endpoint $V$, without a cross product. For a spherical triangle, the reference builds the standard position from $\cos L$ and $\sqrt{1 - \cos^2 L}$ without $\arccos$, centres it with $\cos\rho = c_3$, $\sin\rho = h$, $\cos\varphi = c_1/h$, and $\sin\varphi = c_2/h$ for the reference centre $c$ of unit length, without $\operatorname{atan2}$. The reference finds each normal by the Gram-Schmidt process, without the cross product of `spherical::wall_through`. For the side from $V_{j-1}$ to $V_j$, it takes $e_1 = V_{j-1} / \lvert V_{j-1} \rvert$ and $e_2$ the unit vector of $V_j - (V_j \cdot e_1) e_1$, removes from the opposite vertex its components along $e_1$ and $e_2$, and takes the negated unit vector of the remainder, so the normal points away from the opposite vertex. The reference side lengths are the arccosines of the three values $X_a$, $X_c$, and $X_b$ of the law of cosines, in the arccosine of the reference type. The reference angles of a triangle are $\pi/n_j$, and its reference area is $\sum_j \pi/n_j - \pi$. For a lune, it computes the vertices and normals of Section 6.7 from the reference values of $\cos(\theta/2)$ and $\sin(\theta/2)$. For a double, it builds its own generating domain as above, reflects the vertices and normals componentwise in the reference wall of side 1 by the reflection formula of its geometry, and multiplies the reference reflections for the pairings, so it shares no code with step 3 of Section 6.8. For the test "Compensated pairings", it also forms the product of the two `f64` reflections that the domain stores in the reference arithmetic, at 256 bits.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 10.2 of the geometry design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there, because `astro-float-num` has no public conversion back to `f64`. A message that prints a measured error formats the number as decimal text.
- Self-check. The test "Reference self-check" in `reference` checks the reference library itself: $\pi$ against a literal with 80 digits after the decimal point, $\cos(\pi/3) = 1/2$, $\cosh(\operatorname{arcosh} 2) = 2$, and $\sqrt{2}^2 = 2$, each to $10^{-70}$. The largest of the four errors is $1.7 \times 10^{-77}$. It guards against a wrong precision or rounding argument and does not close `UNV-25`.

DOM-15. The reference module MUST share no code with `escher-domain` or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture).

### 12.3. Tests Under Miri

`DEC-5` and TEST-11 of the coding standard apply to the tests of this crate in this way:

| Tests | Under Miri | Attribute or configuration |
| --- | --- | --- |
| "Triangles", "Third side", "Edge of $\{7, 3\}$", "Circumradius of $\{7, 3\}$", "Relator agreement", "Relators in the domain frame", "Euclidean domains", "SymmHub lines", "Spherical domains", "Rotation domains", the reference part of "Cycle sets", "Compensated pairings", and "Reference self-check" | Ignored | `#[cfg_attr(miri, ignore = "the high-precision arithmetic runs loops too long for the interpreter")]` |
| "Checks at the worst cases", "Exhaustive certificate at the limit", and "Relators evaluate" | Ignored | `#[cfg_attr(miri, ignore = "the checks of the largest orders run loops too long for the interpreter")]` |
| "Side orders" and "Index two" | Ignored | `#[cfg_attr(miri, ignore = "the coset enumeration runs loops too long for the interpreter")]` |
| "Exhaustive certificate" | Ignored, because its reflection domains alone take 332 seconds under Miri, and its doubles add to that. The reason of the row above does not apply, because its orders stop at 12 | `#[cfg_attr(miri, ignore = "the enumeration of the groups runs loops too long for the interpreter")]` |
| "Random triples", "Rectangle aspect", "Random rotation domains", and "Any signature" | Run with 8 cases and no failure persistence | The helper of DOM-16 |
| The unit tests, which include "Failing domains" with the three tests of a corner order off by one, the tests of a negated vertex and of NaN entries, and the defects of the doubles with the moved flat vertex, the unit part of "Cycle sets", "Corner order and area", "Conversion", "Exact rectangle", "Compensated product", "Long cycle word", and "Invalid pairing", the table tests "Unsupported", "Euclidean table", "Parameters", "Spherical table", "Rotation table", "Generating domain", and "Side presentation table", and the integration test "Side order of `22`" | Run unchanged | None |
| The doctests | Not run, because Miri runs the tests through nextest, which runs no doctest | None |

Each reason in an attribute is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

No test asserts a bit-exact result of a transcendental function, because the interpreter need not reproduce the last bits of the host library, and every tolerance of this document leaves room for that. For the same reason, the tests "Euclidean table", "Parameters", "Spherical table", and "Rotation table" compare two builds of one domain within twice the vertex bound of Section 9.4 or 9.5, or twice the image-vertex bound of Section 9.6, and not bit for bit: under Miri, two calls of a transcendental function on the same input can round differently, so two builds can differ in the last bits. The test "Generating domain" compares bit for bit outside Miri, and under Miri within twice the vertex bound of Section 9.3, 9.4, or 9.5 of the geometry of the domain.

Under Miri, the arccosine of an argument close to $-1$ at the flat vertex $W_3$ of the double of `2222` falls outside the flat-vertex bound of $10^{-15}$ of Section 9.6. The test "Rotation table" therefore finds a flat vertex within the corner-angle bound of its geometry, which is $4 \times 10^{-15}$ for a Euclidean double, and the test "Rotation domains", which asserts the flat-vertex bound, does not run under Miri. Under Miri, the error-free product of the test "Compensated product" also gives the exact rounding error $2^{-60}$, so the interpreter computes `f64::mul_add` as one fused, correctly rounded operation on that input.

DOM-16. The property tests MUST take their configuration from one helper in `common`, with the numbers of cases and the failure persistence that Section 4 of `docs/design/tooling.md` states, because the file persistence of `proptest` asks for the current directory, which the isolation of Miri refuses, and a full set of cases takes too long in the interpreter.

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

The crate declares `escher-group` in the same change as `side_presentation()`, which first uses it, as Section 3.3 of the architecture states, because an unused dependency fails `check-unused-deps`. `escher-group` depends on `escher-signature` only, so the graph has no cycle. `Presentation` appears in a public signature of this crate, so DEP-3 of the coding standard makes `escher-group` a public dependency: a breaking change of `escher-group` is a breaking change of this crate. Both crates set `publish = false`, so the cost stays inside the workspace.

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
| `UNV-26` | The angle of a lune, and of the double of a lune, from its two outward normals (item 28 of Section 15.1 of the geometry reference), in `corner_angles()` and check 5 (DOM-20) | Check 4 tests the same angle from the pairings alone, and the exhaustive certificate tests it through the group order |
| `UNV-1` | The termination of the fold that the test "Exhaustive certificate" writes, for the images of the centre | Asserts it for those images only. On a prototype outside the repository, the fold of `*(1000)22` did not converge within the default cap for some points within $10^{-14}$ of its vertex of order 1000, and four such points cycled under a cap of 100,000 |
| None | That `spherical::translation` maps the origin to the sphere point of its target, in step 4 of the spherical triangle | Section 13.1 of the geometry design records the assumption. The test "Spherical domains" compares every vertex and the centre with the reference, which centres without it |
| None | That the angles of a Euclidean polygon with $m$ vertices sum to $(m - 2)\pi$, which check 6 reads as its residual (Section 8.2). No source read states it for a polygon | Reports the residual as a consistency check only. Section 8.4 records its measured residuals, and Section 8.3 the defects that it detects. A source or a Lean proof would close it |
| `UNV-22` | That the group that the pairings of a double generate has the rotation signature, `pqr`, `n22`, `nn`, or `2222` (gap G10 of the domain reference) | Builds the double of the starred signature for every supported rotation signature. Check 6 compares its area with the area of the rotation signature, and the exhaustive certificates enumerate exactly `group_order()` elements on the sphere (DOM-31) |
| `UNV-30` | That the double of the lune of the angle $\pi/n$, a lune of the angle $2\pi/n$ with one rotation pairing, including the hemisphere of `22`, is a fundamental domain of the cyclic group of order $n$ (item 10 of Section 13.3 of the domain reference) | The exhaustive certificates enumerate the group, check that the images of the centre are distinct and fold back, and fold a dense sample of the sphere into the double (DOM-31). Checks 4, 5, and 6 run on every build |
| `UNV-31` | That the double of a convex reflection polygon across one wall is convex, with the vertex cycles of the table of Section 6.8 (item 11 of Section 13.3 of the domain reference), and that Definition 2.8 of [EPSTEIN-PETRONIO1994] admits its flat vertices | The convexity term of check 3 tests every vertex against every wall, check 4 walks the cycles and fails structurally when a cycle has unequal orders or copies that do not sum to 2, and check 5 tests every angle, on every build. The test "Failing domains" shows that the convexity term fails a double with a vertex outside a wall that is not adjacent to it (Section 8.3) |
| `UNV-2` | The termination of the fold of the certificate tests with the rotation pairings of a spherical double, for the images of the centre and the dense sample | Asserts it for those points only, with the steps at least the breadth-first depth. Section 5.3 of `docs/design/render.md` owns the fold of the renderer |
| `UNV-32` | The cycle relators of a double as words: one generator for each side, the pair relators, and the cycle relators with their bases in walk order, under the convention of DOM-32 (Section 5.4) | The tests "Relators evaluate" and "Long cycle word" evaluate the words numerically (DOM-36), and "Side orders" compares the order that the words present with `group_order()` on the sphere. None of them closes it. A source that states the cycle relators as words in this convention, or a Lean proof, would |
| `UNV-19` | The steps of the coset enumeration of `escher-group`, in the tests "Side orders", "Side order of `22`", and "Index two" only | Section 13 of the group design states how `escher-group` relies on it. A wrong enumeration fails these tests or passes a wrong presentation of the same order, which "Relators evaluate" would still catch for a wrong relator |
| None | That the words $s_j s_{j+1}$ of the corners generate the subgroup of index 2 of the orientation-preserving elements, in the test "Index two". Its index 2 is sourced (Section 9.3 of the notation reference) for the hyperbolic and Euclidean reflection groups, and the orders of Section 8.2 of the notation reference give the ratio 2 for the spherical ones | The test asserts the index only. A source or a Lean proof would close it |

The hyperbolic construction does not rely on `UNV-28`, because it uses SymmHub's triangle neither in the code nor as a test oracle (Section 6.3). `UNV-4` concerns the precision of the disk map near the limit circle, and the crate maps to the disk only in the test "Circumradius of $\{7, 3\}$", at a point near the origin.

These statements of this design rest on figures that no measurement of the code covers. They come from a double-precision model of the algorithm outside the repository, and each supports only the choice that it names:

| Statement | Section | Figures of the model |
| --- | --- | --- |
| A domain left with vertex 2 at the origin has larger normal components and fails checks that the final position passes | 6.3 | At $(1000, 3, 2)$, a largest normal component of 1 and a relator residual of $7.28 \times 10^{-12}$, which passes. At $(1000, 100, 100)$, 63.6 and $6.23 \times 10^{-4}$, which fails. At $(1000, 1000, 1000)$, 637 and an infinite residual. In the family $(n, n, n)$, $(60, 60, 60)$ and $(70, 70, 70)$ fail checks 1 and 2, $(80, 80, 80)$ also fails check 4, and every larger order fails. The families $(n, 3, 2)$, $(n, 4, 4)$, and $(n, n, 2)$ fail no check up to $n = 1000$ |
| Repeated multiplication evaluates the relator more accurately than repeated squaring | 8.2 | At corner 0 of $(1000, 3, 2)$, a largest entry of $C^{1000} - I$ of $3.46 \times 10^{-9}$ in the domain frame by repeated multiplication and $7.77 \times 10^{-8}$ by repeated squaring, against $2.33 \times 10^{-11}$ in the corner frame by either method |
| The residual of check 6 stays close to that of check 5 | 8.2 | Never more than 2.91 times the residual of check 5, over 300 random triples |
| Renormalizing the vertices after step 8 makes them less accurate (DOM-10) | 9.1 | At $(1000, 1000, 1000)$, renormalization raised the largest relative vertex error from $5.7 \times 10^{-11}$ to $8.9 \times 10^{-6}$ and the angle error from $1.9 \times 10^{-13}$ to $2.8 \times 10^{-8}$ |

A second prototype outside the repository built the doubles of Section 6.8 from the public values of the reflection domains of this crate, with plain double-precision products as pairings except where a row names rounded products, which it computed in 256-bit arithmetic from the exact values of the two stored reflections and rounded once to the nearest `f64`, and copied the fold of `escher-render`. It checked its copies of `verify()` and of the fold against those of the crates, bit for bit, on the reflection domains. These statements of this design rest on its figures, which no measurement of the code covers. Each figure was measured on that prototype, and each supports only the choice that it names:

| Statement | Section | Figures of the prototype |
| --- | --- | --- |
| Side 1 is the wall to double | 6.8 | The double of `*(1000)32` across side 2 fails check 4 with the residual $2.42 \times 10^{-6}$ against the tolerance $2 \times 10^{-6}$, and 338 of its folds do not converge in band 6 of Section 6.4 of `docs/design/render.md` |
| The pairings are compensated products (DOM-25) | 6.8 | The plain products of the double of `(1000)32` have isometry defects up to $2.7 \times 10^{-12}$, and one fold of its fold set within the range of accuracy does not converge. With the rounded products, no fold of the fold set fails to converge |
| The six checks keep their tolerances for the kites of every hyperbolic triple | 8.4 | Over all 166,665,495 hyperbolic triples: isometries and pairings $2.547 \times 10^{-11}$ at `(992)63`, a margin of 39.3, walls $8.719 \times 10^{-15}$ at `(952)53`, relators, angles, and area as over the cone triples of the set of 294 (Section 8.4), and no failure |
| The spherical doubles keep the tolerances at orders outside the set of Section 9.6 | 8.4 | Over 37 spherical doubles: isometries $2.0 \times 10^{-15}$ at `(10)22`, relators $3.1 \times 10^{-14}$ at `(999)22`, angles $7.06 \times 10^{-14}$ at `(998)22` and `(1000)22`, and area $4.24 \times 10^{-14}$ at `(998)(998)` |
| The vertices of a double lie within the tolerance of the fold | 15.3 | The largest ratio of the test "Vertices" of Section 6.2 of `docs/design/render.md` is $3.22 \times 10^{-16}$ over its named sets, a margin of 12.4, and $4.11 \times 10^{-16}$ over the cone triples of the set of 294, a margin of 9.7. Over all 166,665,495 hyperbolic triples it is $1.224 \times 10^{-15}$ at `(868)(416)5`, a margin of 3.27, and 680,942 doubles exceed one eighth of the tolerance $4 \times 10^{-15}$. No vertex of any double exceeds the tolerance |

The tests of `escher-render` cover two figures of this table with the domains that this crate builds: every fold of the rotation fold sets, including that of `(1000)32`, converges with the compensated products, and the largest ratio at a vertex of `(868)(416)5` is the same as on the prototype (Sections 6.2 and 6.8 of `docs/design/render.md`). The measurements of the crate in Sections 8.4, 9.6, and 12.1 cover the other figures of this prototype: the six checks of the kites over the cone triples of the set of 294, the cycles of two vertices, the Euclidean doubles, the spherical doubles of Section 9.6, the angle of a flat vertex, the effect of the convexity term on the reflection domains (DOM-30), and the fold steps of the certificate tests. The largest residual of check 4 over the cycles of two vertices is $1.094 \times 10^{-8}$, at `(973)(972)(964)`, where the prototype reported at most $1.024 \times 10^{-8}$, at `(995)(987)(976)`.

A third prototype outside the repository built the side presentations of Section 5.4 from the public values of the domains of this crate, with each corner order recovered from the corner angles, enumerated them with its own implementation of the enumeration of Section 7 of the group design, and evaluated their relators. These statements of this design rest on its figures, which no measurement of the code covers. Each figure was measured on that prototype, and each supports only the choice that it names. Every figure of this crate is to be measured, and the code branch records it here:

| Statement | Section | Figures of the prototype |
| --- | --- | --- |
| The side presentations give the group order on the sphere (test "Side orders") | 5.4, 12.1 | The 52 spherical domains of the set of "Side orders" other than `*(1000)(1000)` and `(1000)(1000)`, which the prototype did not run, give `group_order()`, and each enumeration defines exactly that many cosets. `*(1000)22` takes 83 ms in a release build and 309 ms in a debug build |
| The relators evaluate to the identity (test "Relators evaluate") | 12.1 | Largest residual $6 \times 10^{-15}$ at orders up to 12, with exactly 0 for `*732`, `2222`, and `*2222`, and $2.7 \times 10^{-11}$ at the spherical order 1000 |
| No built domain fixes the convention (DOM-36) | 5.4, 12.1 | Every vertex cycle of every built double visits one or two sides, and the reversed bases and the opposite convention give residuals equal to those of DOM-32 |
| The hyperbolic relators need the corner frame (DOM-36) | 12.1 | In the domain frame, the residual reaches 3.3 for `(1000)(1000)(1000)` and $4 \times 10^{-2}$ for `*(1000)(1000)(1000)`, where check 4 reports $5.6 \times 10^{-8}$ |

## 15. Assumptions, Choices, and Points for the Owner

### 15.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- The triangle placed by the two sides $L_a$ and $L_b$ with the included angle $\alpha_2$ is the triangle with the angles $\alpha_0$, $\alpha_1$, and $\alpha_2$. The research states uniqueness given the three angles (Section 10.1 of the geometry reference), but no statement on two sides and the included angle. The test "Third side" and checks 4 and 5 check it. The same reading applies to the spherical triangle of Section 6.7, which checks 4 and 5 and the test "Spherical domains" check.
- The angle at a vertex is the arccosine of $(\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)$. This is the first law of cosines of Section 10.2 of the geometry reference solved for the angle, which no source prints in this form. The test "Triangles" compares the angles with $\pi/n_j$ in the reference.
- The normalized sum of the vertices lies strictly inside the triangle, hyperbolic or spherical. Check 2 fails if it does not, and the tests "Random triples" and "Spherical table" check it.
- A power of a matrix is the identity exactly when the same power of its conjugate is. This is matrix algebra, not a statement of the geometry reference. The test "Relator agreement" checks it on the 30 named triples.
- Side 0, side 1, and side 2 are the Latin reflections $P$, $Q$, and $R$ of step 3 of Section 9.1 of the notation reference, and its fourth reflection $S$ is $P$ again for a triangle and side 3 for the rectangle. The relators that the checks use come from Section 4.4 of the domain reference and do not depend on this reading.
- `hyperbolic::translation(c)` maps `hyperbolic::ORIGIN` to the point with the spatial coordinates of $c$, as Section 6.4 of the geometry design states, and this is $c$ itself only when $c$ lies on the hyperboloid. Section 13.1 of the geometry design records this property as an assumption and names the test that checks it.
- The angle of a kite at $W_0$ or $W_2$ is the sum of the angles of its two triangles there, and its area is the sum of their areas (DOM-29). The test "Rotation domains" compares the angles and the area with the reference, and check 5 compares each angle with the copies times $\pi$ divided by the order.
- The image of a wall under $R_1$ has the normal $R_1 u$ for the normal $u$, which is outward for the image side, and in the Euclidean plane the covector $R_1^{T} c$ for the covector $c$ (step 3 of Section 6.8). No source prints these forms. Checks 2 and 3 test each image wall against its vertices and the centre, and the test "Rotation domains" compares it with the reference, which reflects the normals componentwise.
- The centre of the generating domain lies strictly inside its double. Check 2 fails if it does not.
- For the unit normals $u_0$ and $u_1$ of the double of a lune, $\operatorname{atan2}(\lvert u_0 \times u_1 \rvert, -u_0 \cdot u_1)$ is the same angle as $\arccos(-u_0 \cdot u_1)$ (DOM-20). No source read states this form. Checks 4 and 5, the test "Rotation domains", which compares each angle with the copies times $\pi$ divided by the order, and the exhaustive certificates check it. A source that states it, or a Lean proof, would close it.
- The wall term of check 2, $P_i^{T} c_p + c_i$, is 0 when $P_i$ carries the wall of side $i$ onto the wall of its partner $p$ with the domain on opposite sides (Section 8.2). No source read states this form. The defect of the half-turn on `44` fails check 2 through this term alone, and every double of Section 9.6 passes it. A source that states it, or a Lean proof, would close it.
- The compensated product of DOM-25 is correctly rounded for every entry of every pairing of a double. The test "Compensated pairings" asserts it for the doubles of Section 9.6 only. Compensated arithmetic does not guarantee it in general, and no test covers the other doubles, such as the kites of the cone triples outside the set of 294 triples, or `2222` at other values of `aspect`. A proof of correct rounding for the entries of products of two reflections, or a test over every supported double, would close it.
- A canonical rotation signature lists its cone orders in descending order, so the orders of a rotation signature are the corner orders of its starred signature in the same order (Section 4). The test "Rotation table" checks it through the spellings `237` and `22n`.

### 15.2. Choices of This Design

No other document dictates these choices:

| Choice | Section |
| --- | --- |
| `FundamentalDomain::build` as an inherent constructor with one argument that returns a `Result` | 3.1, 3.4 |
| The measurements `side_lengths`, `corner_angles`, and `area` as methods of the domain, so that `escher-cli` needs neither `glam` nor `escher-geometry` | 3.1, 7 |
| `Orientation` with both variants, and `PairingKind` with three (item 20 of Section 15.3) | 3.2 |
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
| Every tolerance and bound of Sections 8.4 and 9.3 to 9.6, and the floors of the exhaustive certificates | 8.4, 9.3 to 9.6, 12.1 |
| The tolerance of the fold of the exhaustive certificate, copied from `escher-render` | 12.1 |
| 256 bits for the reference arithmetic | 12.2 |
| The sets of triples of Section 12.1, and the weighting of the test "Random triples" toward large orders | 12.1 |
| `astro-float-num` in place of `astro-float` | 13 |
| The choices of the Euclidean domains and of the shape parameter that Section 15.3 puts to the owner, in items 6 to 12 | 3.4, 6.5, 6.6, 7, 8.2 |
| The choices of the spherical domains that Section 15.3 puts to the owner, in items 13 to 18 | 4, 6.7, 7, 8.2, 8.4, 9.5 |
| The choices of the rotation domains that Section 15.3 puts to the owner, in items 19 to 28 | 3.1, 3.2, 6.6, 6.8, 8.2, 8.4, 9.6 |
| The private corner with its order and its copies, in place of the corner orders | 3.2, 6.8 |
| The two triangles of a kite for its angles and its area, and the angle sum of each triangle for check 5 | 7, 8.2 |
| The angle of the double of a lune by `atan2`, and of a lune by the arccosine | 7 |
| The wall term of check 2, and the kind `Translation` from the translation test of DOM-27 | 6.8, 8.2 |
| The defects of the doubles in Section 8.3, including the lowered orders of `532` and the moved flat vertex of `2222` | 8.3 |
| The sets of doubles of Section 9.6 | 9.6, 12.1 |
| The private enum of the supported classes as two variants, reflection and rotation, over five shapes | 3.3 |
| The choices of the side presentation that Section 15.3 puts to the owner, in items 29 to 32 | 3.4, 5.4, 10, 12.1 |
| The order of the relators of the side presentation: squares before corner relators, and pair relators before cycle relators in the order of the walk | 5.4 |
| The kinds of relator in "Relators evaluate" evaluated as checks 2 and 4 evaluate them, with their tolerances | 12.1 |

### 15.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Reference crate. `DEC-3` names `astro-float`. The design uses `astro-float-num` (Section 13). The alternative is a `skip` entry in `deny.toml` for `syn` 1 with a reason. Section 13.3 of the geometry design raises the same point.
2. Largest order and the tolerance of check 4. The design supports corner orders up to 1000 and returns `LimitExceeded` above it (DOM-3). It sets the tolerance of check 4 to $2 \times 10^{-6}$, because the largest residual of that check would leave a margin of 7.5 at $10^{-6}$, below the 8 that TEST-13 of the coding standard requires, and the margin at $2 \times 10^{-6}$ is 15.0, with check 4 still failing every triangle near the limit with one corner order off by one (Sections 8.3 and 8.4). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin.
3. Comparison with MPFR for `UNV-25`. The geometry design records the comparison (its Section 12) and asks whether it closes `UNV-25`. This design relies on the same answer.
4. Check 6. It detects nothing that check 5 does not, apart from a disagreement with `escher-signature` (Section 8.2). The design keeps it because the plan requires an area check. The alternatives are to drop it, or to measure the area by an independent formula from the vertices, which needs a source that the research does not hold.
5. Placement. The final position makes the centre of the canonical view a property of the domain, while Section 3.2 of the architecture places the canonical placement in `View`. `View` can still move the domain, because `centre()` is public.
6. Vertices of the Euclidean triangle. No source gives them, so the design builds the triangle from the candidate of `UNV-28` and certifies each build by checks 3 and 5 and the test "SymmHub lines" (Section 6.5). The alternative is to build only `*2222` until a source or a Lean proof closes the candidate.
7. Units of the Euclidean domains. The triangle has side 0 of length 1, and the rectangle has the area 1 (Section 6.5), so `aspect` and its reciprocal give the same rectangle turned a quarter turn. This does not hold for the double of `2222`, whose doubled wall is always the right side of its generating rectangle (item 24). The alternative for the rectangle is the height 1 and the width `aspect`, which makes the size of a tile depend on `aspect`.
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
18. The tolerances and the limit. The design keeps every tolerance of Section 8.4 and the limit of 1000 for the spherical domains, and sets the bounds of Section 9.5 from the largest errors measured. Every check passes on the spherical domains of Section 9.5 with a margin of at least $2.45 \times 10^{4}$. The alternative is tolerances of their own for the sphere.
19. The doubled wall. The design doubles across side 1, the wall through $V_0$ and $V_1$ (DOM-23, Section 6.8), because on a prototype the double of `*(1000)32` across side 2 failed check 4 and left folds unconverged (Section 14). The alternative is side 2, the wall opposite the corner of the largest order.
20. The kinds of a pairing. The design adds the kinds `Rotation` and `Translation` (Section 3.2). The alternative is one kind for every orientation-preserving pairing, which would call the translation of the double of `2222` a rotation.
21. The accessor `reverses_orientation()`. The design adds it, so that `escher-cli` decides in one place whether a parity colouring exists (Section 3.2). The alternative is that each caller scans `sides()`.
22. The corner of a vertex. The design replaces the stored corner orders with a private corner that holds the order and the copies (Section 3.2). The alternative is to store the expected angle of each vertex, which check 4 would then have to turn back into an order.
23. The pairings of a double as compensated products (DOM-25). The design computes each pairing with compensated arithmetic at construction, because the plain product left one fold of `(1000)32` unconverged within the range of accuracy on a prototype (Section 14). The test "Compensated pairings" asserts that every entry of the compensated products of the doubles of Section 9.6 is correctly rounded, while the plain products misround entries in every geometry. Compensated arithmetic does not guarantee correct rounding for the other doubles, which Section 15.1 records. Section 9.6 also records that the compensated product does not lower the largest isometry defect over the cone triples of the 30 named triples: it lowers the defect at some doubles, such as `(1000)32`, and raises it at others, such as `(868)44`. The research documents cite no source for the compensated dot product, which is a numerical technique and not a statement of the geometry. The alternatives are plain products with a narrower range of accuracy for the orders near 1000, which `docs/design/render.md` would state, or a fold with the reflections of the generating domain that applies the reflection in the doubled wall when the parity is odd, which needs no rotation pairing in the fold.
24. `aspect` of `2222`. The design makes it the aspect of the generating rectangle, with the area 1, so the double has the area 2 and is twice as wide (Section 6.6). The alternative is a double with the area 1 and `aspect` as the ratio of its own sides, which would change the generating rectangle of `2222` away from that of `*2222` with the same setting.
25. The convexity term of check 3. The design evaluates it for every domain (Section 8.2), and it leaves the residuals of the reflection domains unchanged (DOM-30). The alternative is to evaluate it for the doubles only.
26. The double of the lune on an unverified statement. No source makes it a fundamental domain, so the design builds it on `UNV-30` and certifies it by the tests of DOM-31. The alternative is to leave `nn` unsupported until a source or a Lean proof closes the statement.
27. The margin of the vertices of a double against the tolerance of the fold. On a prototype, no vertex of a double exceeded the tolerance of the fold over every hyperbolic triple, but the margin over that sweep fell below the factor 8 of TEST-13 of the coding standard, while the named sets stayed above it (Section 14 gives the figures). `docs/design/render.md` owns the tolerance and the test "Vertices", and the tolerance cannot rise without changing the images of the reflection domains. The options are to accept the measured margin over every triple, to restrict the claim of the margin to the named sets, or to compute the image vertices and normals as correctly rounded products too, which no prototype or test measured.
28. The tolerances of the doubles. The design keeps every tolerance of Section 8.4 and the limit of 1000 for the doubles, and sets the bounds of Section 9.6 from the largest errors measured. Every check passes on the doubles of Section 9.6 with a margin of at least 15.0 (Section 8.4). The alternative is tolerances of their own for the doubles.
29. One generator for each side (`DEC-36` of the architecture). The side presentation keeps both generators of a pair with the pair relator (Section 5.4), so a colouring finds the generator of a fold step by its side index. The alternative is one generator for each pair, which halves the generators of a double and the columns of its coset tables, and needs a map from a side to a letter and its inverse.
30. `side_presentation()` returns a `Result` with the kind `InvalidPairing` (DOM-35). The alternative is an infallible function that is total on the domains that `build_with` returns, which would have to invent a presentation or panic for a domain corrupted through private access.
31. The test "Long cycle word" (DOM-36). No domain that this crate builds fixes the convention of DOM-32, so the design requires one unit test on a domain built through private access with a vertex cycle of at least three sides. The alternative is to defer that test to the first milestone that builds a domain with such a cycle, and to rely on "Relators evaluate" until then, which leaves the convention untested.
32. The generator names `s0` upward, from the side index (`DEC-34` of the architecture). The alternative is names from 1, which would differ from the side index that the renderer records and `escher info` prints.

## 16. Security Considerations

The threat model is a caller that passes a `Signature` parsed from untrusted text, and settings with untrusted names and values, to `FundamentalDomain::build_with`, and then calls `verify()`. `escher-signature` has already rejected malformed text and bounded the signature (Section 16 of the signature design). The attacker can choose any valid signature. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of time or memory through a large order, a panic that stops the caller, and a wrong domain that the caller uses as correct. The crate counters them with the limit of this section, the rule that it never panics (DOM-2 and DOM-8), and the checks of Section 8.

DOM-3 requires the limit of this table. The input is untrusted, the cost of `verify()` grows with the order, and the accuracy of Section 9.3 is established only up to that order:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Largest corner order | 1000 | The construction costs the same for every order. `verify()` costs at most about 3000 matrix products at this limit (Section 8.2), and the relator check keeps a margin of 15.0 at its tolerance of $2 \times 10^{-6}$ over every supported triple (Section 8.4). The plan tests up to this order. The number is a choice of this design (Section 15.3). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin |
| `aspect` | 0.125 to 8, inclusive | A finite value in a bounded range keeps the rectangle within the scale that the tests of Section 9.4 measure. NaN and infinities are rejected (DOM-18). The range is a choice of this design (Section 15.3) |

The limit is a private constant, and unit tests inside the crate read it. It applies in the same way to the orders of `*n22` and `*nn` and of every rotation signature, and `verify()` of a spherical domain or of a double costs about as many products as at the hyperbolic limit (Section 8.2). `build_with` allocates at most six sides and six vertices, for the double of the rectangle, so the memory of a domain does not depend on the input. The walk of check 4 stops after $m$ steps, so a corrupted partner cannot make it loop. A setting is only compared with the names of the `ParameterSpec` values and stored, and the error text echoes no setting (Section 10.2), so the number and the length of the settings need no limit.

`side_presentation()` builds at most six generators and eight relators, each base of at most six letters, with an exponent of at most 1000, so its memory does not depend on the input beyond the order limit. Its walk stops after $m$ steps, as that of check 4 does. It runs no coset enumeration. The cost of an enumeration of the presentation, and its limits, belong to `escher-group` (Section 15 of the group design).

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
| A domain without a side presentation | `InvalidPairing`, from `side_presentation()` | None. No domain that `build_with` returns produces it, and one that does is a defect of the crate |
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
- [RATCLIFFE] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SERIES2013] Section 14.1 of `docs/research/fundamental-domains.md`.
- [SOLOMYAK2020] Section 14.1 of `docs/research/fundamental-domains.md`.
- [TH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [VOIGHT2009] Section 14.1 of `docs/research/fundamental-domains.md`.
- [VONGAGERN2009] Section 14.1 of `docs/research/fundamental-domains.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| DOM-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| DOM-2 | `build` constructs a domain only for a signature that meets the predicate of Section 4, and returns `UnsupportedSignature` for every other valid signature without a panic | 4 |
| DOM-3 | `build` returns `LimitExceeded` for a supported signature with an order above 1000, after the support check | 4 |
| DOM-4 | Vertices and sides of a reflection domain follow the index convention of Section 5.1, and those of a double that of Section 6.8 | 5.1 |
| DOM-5 | `build_with` constructs a hyperbolic reflection domain by the steps of Section 6.1, a Euclidean one by those of Section 6.5, a spherical one by those of Section 6.7, and a double by those of Section 6.8, in order | 6.1 |
| DOM-6 | No relation between the inner product of two normals and the corner angle is used, except for the domain with two vertices of DOM-20 | 6.1 |
| DOM-7 | `verify()` computes and reports every check on every call | 8.1 |
| DOM-8 | `verify()` is pure, never panics, and renormalizes nothing | 8.1 |
| DOM-9 | Each check uses the tolerance of Section 8.4 | 8.4 |
| DOM-10 | The vertices are not renormalized after centring | 9.1 |
| DOM-11 | The values that `build_with` returns are within the bounds of Section 9.3, of Section 9.4 for a Euclidean domain, or of Section 9.5 for a spherical one, and for a double also of Section 9.6 | 9.3 |
| DOM-12 | `Error` follows ERR-1, with the operation that failed | 10.1 |
| DOM-13 | The `Display` text of each kind is the one stated, with the prefix of `side_presentation()` for `InvalidPairing` | 10.2 |
| DOM-14 | No test takes an expected value from Table 2 of [HT] | 12.1 |
| DOM-15 | The reference module shares no code with the crates under test | 12.2 |
| DOM-16 | Property tests take their configuration from one helper that is Miri-aware | 12.3 |
| DOM-17 | The crate declares only the dependencies of Section 13, each from the workspace table | 13 |
| DOM-18 | `build_with` checks support, the order limit, the names, and the values, in this order, and returns the first error that applies | 6.6 |
| DOM-19 | A test of a quantity without a measured bound here sets and records its bound as TEST-13 requires, in the same change | 9.4 |
| DOM-20 | The angle of a domain with two vertices is measured from its two normals, by the clamped arccosine for a lune and by `atan2` for its double, and no other angle is | 7 |
| DOM-21 | For a spherical domain, the vertex precondition of checks 2 to 6 is finite and not the zero vector, with no condition on the last coordinate | 8.2 |
| DOM-22 | The exhaustive certificates cover every lune of their sets and stay while `UNV-29` is open | 12.1 |
| DOM-23 | A rotation signature is built as the double of the reflection domain of its starred signature across the wall of side 1, with that domain left in place and its centre as the centre | 6.8 |
| DOM-24 | Vertices, sides, and partners of a double follow Section 6.8, with the partner $(2n - 1 - k) \bmod (2n - 2)$ of side $k$ | 6.8 |
| DOM-25 | The pairing of an original side is $R_1 R_j$ and of an image side $R_j R_1$, each the compensated product of the two stored reflections, entry by entry | 6.8 |
| DOM-26 | Two halves on one wall at a corner of order 2 stay two sides that meet at a flat vertex, and no side of a double is its own partner | 6.8 |
| DOM-27 | Check 2 fails unless the kind, the partner, the orientation, and the pairing of every side agree, with the kind `Translation` exactly for a pairing that passes the translation test | 8.2 |
| DOM-28 | Check 4 of a domain without a mirror walks the vertex cycles and evaluates each cycle relator, and a domain that mixes kinds gets an infinite residual | 8.2 |
| DOM-29 | The angles and the area of a hyperbolic or spherical kite are measured from its two triangles across the doubled wall | 7 |
| DOM-30 | The stored values and the check residuals of every reflection domain do not depend on the code of the doubles | 8.4 |
| DOM-31 | The exhaustive certificates cover the spherical doubles and stay while `UNV-22` or `UNV-30` is open | 12.1 |
| DOM-32 | `side_presentation()` gives one generator for each side, named `s<i>`, for $P_i^{-1}$, with words read left to right as products | 5.4 |
| DOM-33 | A reflection domain gives the squares $s_j^2$ and then the corner relators $(s_j s_{j+1})^{n_j}$, in that order | 5.4 |
| DOM-34 | A domain without mirrors gives the pair relators and then the cycle relators, with each base in walk order and the order of the cycle as exponent | 5.4 |
| DOM-35 | A domain with both kinds of side, a walk that does not close, a cycle with unequal orders or copies, or a presentation that `escher-group` rejects gives `InvalidPairing`, without a panic | 5.4 |
| DOM-36 | "Relators evaluate" covers every domain of its sets and "Long cycle word" exists, and both stay while `UNV-32` is open | 12.1 |
