# Domain Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-16 |
| Applies to | `crates/escher-domain` |

## Abstract

This document specifies the crate `escher-domain`, which builds the fundamental domain of an orbifold signature as a polygon in the linear model, with a wall, a pairing isometry, and a partner for each side, and verifies it numerically. It covers the compact hyperbolic reflection triangle `*pqr`. It defines the public types and operations, the exact set of signatures that have a domain, the construction as a numbered algorithm with the source of each step, the placement of the triangle, the measurements of its sides, angles, and area, and the six numeric checks of the verification with their tolerances and the defects that each one detects. It also states the numerical accuracy over the supported orders, the error type, the resource limits, the tests, the dependencies, the unverified statements that the code relies on, the assumptions, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

The renderer folds every pixel into the fundamental domain and colours it by the side pairings that the fold applied. It needs a polygon whose sides carry exact walls and pairing isometries, placed so that the canonical view is centred on it. The research gives the angles of the reflection triangle, its side lengths by the law of cosines, and the theorems that make it a fundamental domain, but it gives no placement in the linear model and no numerical tolerance (Section 10 of the geometry reference and Section 4.8 of the domain reference). A wrong normal, a wrong angle, or a wrong pairing produces an image that looks plausible, so the crate must check its own result.

### 1.2. Purpose

`escher-domain` is the one place where a `Signature` becomes a `FundamentalDomain`. It builds the polygon of a supported signature, returns an error kind for every other signature, and verifies a domain against the conditions of the polygon theorems that the domain reference cites. Section 3 of the architecture places the crate in the workspace. It depends on `escher-signature` for the signature and on `escher-geometry` for the linear model, and the renderer and `escher-cli` consume it.

### 1.3. Audience

The primary readers are the agent that changes the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-cli`, which prints the domain summary and the result of the verification, and of `escher-render`, which folds into the domain.

### 1.4. Scope

This document covers the public surface of the crate, the supported signatures, the conventions of the sides, the vertices, and the pairings, the construction of the compact hyperbolic reflection triangle, its placement, its measurements, the verification, the numerical accuracy, the error type, the resource limits, the tests, and the dependencies.

The supported signatures are the compact hyperbolic `*pqr` with $1/p + 1/q + 1/r < 1$ (Section 4).

### 1.5. Non-Goals

These topics are owned elsewhere or are outside this document:

- The bilinear form, the side test, the distance, the isometries, the reflection, the wall through two points, and the disk map, with their formulas and tolerances, belong to `escher-geometry` and are specified in `docs/design/geometry.md`. This document calls those functions by name and restates none of their formulas.
- The lines that `escher info` prints for a domain and for `--check`, their format, and the exit status are specified in `docs/design/cli.md`.
- Shape parameters, `ParameterSpec`, `ShapeParameters`, and the parameter names of Section 5.6 of the architecture. The reflection triangle has no free parameter ("triangles are rigid", Section 10.1 of the geometry reference).
- The side presentation, which needs `escher-group` (Section 3.2 of the architecture).
- Ideal vertices, Euclidean and spherical domains, and every other class of signature in the plan. Each returns `UnsupportedSignature` (Section 4).
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

A function written as `hyperbolic::name` is the function of that name in the module `hyperbolic` of `escher-geometry`, which the geometry design specifies.

It adds these terms:

- supported signature: A signature that meets the predicate of Section 4. `FundamentalDomain::build` constructs a domain for it when its corner orders are at most 1000 (DOM-3).
- corner $j$: The corner of the canonical signature at index $j$ of `boundaries()[0].corner_orders()`, with the order $n_j$ and the angle $\alpha_j = \pi/n_j$.
- standard position: The placement of the triangle in which vertex 2 is `hyperbolic::ORIGIN` and vertex 0 lies on the positive first axis (Section 6.1, step 5).
- final position: The placement in which the centre of `DEC-12` is `hyperbolic::ORIGIN` (Section 6.3).
- corner frame: The frame of corner $j$, in which vertex $j$ is `hyperbolic::ORIGIN` (Section 8.2, check 4).
- check: One numeric condition that `verify()` evaluates, with a residual and a tolerance.
- residual: The non-negative number that a check computes, which is 0 for an exact domain, and infinity when a structural condition fails or when the computation gives no finite number. It is never NaN.
- tolerance: The largest residual for which a check passes.

## 3. Public Surface

### 3.1. Items

DOM-1. The crate MUST export exactly the items of this section from its root, each through one path, and MUST give no type a public field, because a public field or a second constructor lets a caller build a domain that breaks the invariants of Section 3.2, and the renderer folds into the domain without checking them (API-10 of the coding standard).

This listing shows each public item with its signature. Bodies are omitted. `DVec3` and `DMat3` are the `glam` types of `DEC-4` (Section 3.2 of the architecture):

```rust
pub struct FundamentalDomain { /* private */ }
// derives Clone, Debug, PartialEq
impl FundamentalDomain {
    pub fn build(signature: &escher_signature::Signature) -> Result<Self>;
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

#[non_exhaustive]
pub enum ErrorKind { UnsupportedSignature, LimitExceeded, Numerical }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

The crate does not re-export `glam` (API-10 of the coding standard). A caller that names a `glam` type declares `glam` from the workspace entry, as Section 3.1 of the architecture states for the crates that compute with these types.

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `FundamentalDomain` | The fundamental domain of a supported signature | Built only by `build`. It holds the geometry, the three corner orders, and the area of the signature, and the sides, the vertices, and the centre |
| `build` | The only constructor | Returns a domain exactly for a supported signature with corner orders of at most 1000 (Section 4), and an `Error` otherwise. Never panics |
| `geometry()` | The geometry of the signature | `Hyperbolic` for every domain that `build` returns |
| `sides()` | The sides, in the order of Section 5.1 | Three sides of kind `Mirror` |
| `vertices()` | The vertices, in the order of Section 5.1 | As many as the sides. Vertex $j$ is corner $j$. The vertices are counterclockwise in the disk (Section 5.2). Each has the accuracy of Section 9.3, and a caller does not renormalize it (Section 9.1) |
| `centre()` | The centre of `DEC-12`, the normalized sum of the vertices | Within $10^{-9}$ of `hyperbolic::ORIGIN` in each coordinate (Section 6.3) |
| `Side::normal` | The unit normal of the wall of the side | Spacelike and outward (Section 5.2) |
| `Side::covector` | The covector of the wall | Equal to `hyperbolic::covector(normal)` |
| `Side::pairing` | The pairing isometry $P_j$ of the side (Section 5.3 of the architecture) | For a mirror, `hyperbolic::reflection(normal)` |
| `Side::partner` | The index of the side onto which $P_j$ maps side $j$ | For a mirror, $j$ itself |
| `Side::kind` | The kind of the pairing | `Mirror` for every side of a reflection triangle |
| `Side::orientation` | Whether $P_j$ preserves or reverses orientation | `Reversing` for a mirror |
| `side_lengths()` | The length of each side, in side order | Section 7 |
| `corner_angles()` | The angle at each vertex in radians, in corner order | Section 7 |
| `area()` | The area measured from the corner angles | Section 7 |
| `verify()` | The result of the six checks of Section 8.2 | Pure and never panics (DOM-8) |
| `Verification::passed` | Whether every check passed | True exactly when each of the six `Check::passed` is true |
| `Check::passed` | Whether the residual is within the tolerance | `residual() <= tolerance()`. The residual is never NaN, and an infinite residual fails |
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
- `src/triangle.rs`: The supported-signature predicate of Section 4 and the construction of Section 6.
- `src/verify.rs`: `Verification`, `Check`, and the checks of Section 8.

### 3.4. Items Outside This Surface

Section 3.2 of the architecture lists items that this surface does not have, because the reflection triangle does not need them:

- `parameters(&Signature)`, `ParameterSpec`, and `ShapeParameters`. The triangle has no free parameter. The class that first has a parameter adds them, together with the form of `build` that takes the shape parameters. The crate sets `publish = false`, so the change affects only the workspace.
- `side_presentation()`, which needs `escher-group`.
- Ideal vertices, which the triangle with an ideal corner adds.

`build` is an inherent constructor (API-5 of the coding standard) and returns a `Result`. Section 3.2 of the architecture lets the crate design refine the names and signatures.

## 4. Supported Signatures

DOM-2. `FundamentalDomain::build` MUST construct a domain only for a signature that meets the predicate of this section, subject to DOM-3, and MUST return `UnsupportedSignature` for every other valid signature without a panic, because a caller that receives a domain for a signature outside the construction folds into a polygon that does not tile, and a panic stops the caller's whole program (ERR-5 of the coding standard).

A signature is supported exactly when all of these hold, each read from the public accessors of `escher-signature` (Section 3.1 of the signature design):

1. `handles()` is 0.
2. `crosscaps()` is 0.
3. `cone_orders()` is empty.
4. `boundaries()` has exactly one element, and its `corner_orders()` has exactly three elements.
5. `finite()` of each of the three orders is `Some`. This holds exactly when `is_compact()` is true for such a signature, and the crate reads `finite()` so that it needs no `expect` later.
6. `geometry()` is `Geometry::Hyperbolic`.

The predicate also needs `area_over_pi()` to be `Some`, which it is for every signature that meets conditions 1 to 6, so this condition makes no further signature unsupported. Step 1 of Section 6.1 converts the area.

The geometry is the exact rational classification of `escher-signature` (Section 9.1 of the signature design), so no floating-point comparison decides support. For three finite corners, condition 6 is the condition $1/p + 1/q + 1/r < 1$ of Section 9.1 of the domain reference and Section 10.1 of the geometry reference.

Every other valid signature returns `UnsupportedSignature`. This covers the Euclidean `*632`, the spherical `*532`, the hyperbolic `*3333`, `732`, and `4*3`, the triangle with an ideal corner `*(inf)32`, the friezes, the trivial signature `1`, and every general non-compact signature. A general non-compact signature returns this kind throughout the proof of concept (`DEC-19`, Section 5.8 of the architecture).

DOM-3. `build` MUST return `LimitExceeded` for a supported signature with a corner order above 1000, and MUST apply the predicate of this section first, because the accuracy of Section 9.3 is established only up to that order, and a domain outside it could pass its checks by a smaller margin than the tolerances assume. A signature that is both unsupported and over the limit gets `UnsupportedSignature`, because the limit is a property of this construction. Section 16 states the reason for the number.

## 5. Sides, Vertices, and Pairings

### 5.1. Index Convention

The canonical form of `*pqr` lists its corners with $n_0 \ge n_1 \ge n_2$ (Section 7.2 of the signature design, step 4). So `*237` and `*327` both become `*732`, and vertex 0 has the smallest angle and vertex 2 the largest.

DOM-4. The crate MUST number the vertices and sides by this convention, because `escher-cli` prints the angles in corner order and the side lengths in side order, and the renderer records the side index of every fold step, so a second numbering gives two readings of one printed value:

- Vertex $j$ is corner $j$, with the order $n_j$.
- Vertex $j$ is the common point of side $j$ and side $j + 1$, with indexes modulo 3. So side $j$ joins vertex $j - 1$ and vertex $j$.
- Corner $j$ lies between side $j$ and side $j + 1$, so its relator is $(P_j P_{j+1})^{n_j}$.

Section 9.1 of the notation reference reads a boundary with corners of orders $a$, $b$, and $c$ as the Latin reflections $P$, $Q$, $R$, and $S$ with the relators $(PQ)^a$, $(QR)^b$, and $(RS)^c$. Side 0 is $P$, side 1 is $Q$, and side 2 is $R$, and the convention reads $S$ as $P$ again. Section 15.1 records this reading as an assumption. The relators that the checks use come from Section 4.4 of the domain reference, which has three reflections and does not depend on it.

### 5.2. Orientation and Outward Normals

The vertices are counterclockwise in the disk: the planar cross product of `hyperbolic::to_disk(V1) - hyperbolic::to_disk(V0)` and `hyperbolic::to_disk(V2) - hyperbolic::to_disk(V0)` is positive.

The normal of each side points outward, as Section 5.2 of the architecture requires. A point $x$ is inside the domain where `hyperbolic::side(covector, x)` is at most 0 for every side, and a positive value means that the point is outside that side. Section 2.4 of the geometry reference records that [RAT] orients normals inward. This crate uses no statement of [RAT] that depends on that orientation.

### 5.3. Pairings

Side $j$ carries the isometry $P_j$ that maps the neighbouring copy of the domain across side $j$ onto the domain (Section 5.3 of the architecture). For a mirror side, $P_j$ is the reflection in its wall, `hyperbolic::reflection(normal)`, which is the reflection of equation (6.6) of [DAVIS] (Section 7.4 of the geometry reference) as Section 5.2 of the architecture adopts it. The partner of a mirror side is the side itself.

## 6. Construction

### 6.1. Algorithm

DOM-5. `build` MUST construct the domain by the steps of this section, in this order, because the accuracy of Section 9.3 and the margins of Section 8.4 are measured for exactly these steps, and another order of the same operations gives other rounding errors that no test measures.

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

DOM-6. The construction and the measurements MUST NOT use a relation between the inner product of two normals and the corner angle, because the only such relation in the research is stated with inward normals in the conventions of [RAT] (Section 7.4 of the geometry reference), and its form with the outward normals of this project is part of `UNV-26`. The corner angles are measured from the distances between the vertices instead (Section 7).

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

The placement differs from that of SymmHub, which keeps one corner at the origin (Section 7.6 of the geometry reference). The design does not use SymmHub's triangle as a test oracle, so it does not rely on `UNV-28`.

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

## 7. Measurements

The crate measures each quantity from the stored vertices, so that `verify()` and `escher-cli` see the same numbers:

- `side_lengths()` returns `hyperbolic::distance(V_{j-1}, V_j)` for each side $j$.
- `corner_angles()` returns, for each vertex $j$, the angle $\theta_j$ in $[0, \pi]$ for which the first law of cosines of Section 10.2 of the geometry reference ([BEA] Section 7.12, p. 148, and [RAT] Theorem 3.5.3, p. 86) holds with the three measured side lengths. With $c$ the length of the side opposite vertex $j$ and $a$ and $b$ the other two, it is $\arccos\big((\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)\big)$, with the argument clamped to $[-1, 1]$.
- `area()` returns $\pi - \theta_0 - \theta_1 - \theta_2$, the area of a triangle by Section 10.1 of the geometry reference.

The angles are measured from distances so that no step uses a relation between normals and angles (DOM-6). The form of the first law solved for the angle is a rearrangement that no source prints, and Section 15.1 records it.

## 8. Verification

### 8.1. Report Types

`verify()` returns a `Verification` that holds one `Check` for each of the six checks of Section 8.2. Each `Check` holds its residual and its tolerance. A check passes when its residual is at most its tolerance, so an infinite residual fails. Section 2.1 defines the residual, which is never NaN.

DOM-7. `verify()` MUST compute every check of Section 8.2 on every call and MUST report each one separately, because `escher info --check` prints the result of each check, and a verification that stops at the first failure hides whether the other conditions hold.

DOM-8. `verify()` MUST be pure, MUST NOT panic for any domain, and MUST NOT renormalize any stored value, because it runs on domains that tests corrupt on purpose, and a renormalized value would hide the defect that the check exists to find.

### 8.2. Checks

Every check runs on the stored values of the domain in the final position, unless the check names another frame. Checks 2 to 6 read the vertices, so each first requires every vertex to have finite coordinates and a positive last coordinate, and its residual is infinity when one does not. Check 1 reads only the pairings. A residual that is not finite after the computation of its check is reported as infinity:

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

## 9. Numerical Robustness

### 9.1. Renormalization

- `hyperbolic::normalize_point` runs twice, in steps 6 and 10 of Section 6.1, each time on a sum of vertices.
- `hyperbolic::wall_through` divides each normal by its norm (the geometry design).
- `verify()` renormalizes nothing (DOM-8).

DOM-10. The construction MUST NOT renormalize the vertices after step 8, because a renormalized vertex lies on the hyperboloid but far from the true corner. Section 14 records the figures for $(1000, 1000, 1000)$, which no measurement of the code covers.

A vertex therefore leaves the hyperboloid by a small amount. Over the triples of the reference sweep of Section 9.3, $\lvert \langle V, V \rangle + 1 \rvert$ reaches $4.113 \times 10^{-5}$, at $(995, 987, 976)$. The wall through two vertices depends only on the plane that they span, so the normals are not affected.

### 9.2. Forms

- `hyperbolic::inverse` computes the geometric inverse $J M^t J$ (Section 6.1 of the geometry reference), never a general matrix inverse.
- The relators are evaluated in the corner frame (Section 8.2, check 4).
- The corner angles come from the distances between the vertices (Section 7).
- $\operatorname{arcosh} X$ in step 4 needs no special form. The smallest $X$ that the construction computes over the triples of the reference sweep of Section 9.3 is 1.040349, at `*732`.

### 9.3. Accuracy over the Supported Range

DOM-11. The values that `build` returns MUST lie within the bounds of this table of the values of the exact triangle in the final position, because the renderer and `escher info` use them without a check, and the reference-arithmetic tests of Section 12.1 assert these bounds.

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

## 10. Errors

### 10.1. Structure

DOM-12. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule, and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"build fundamental domain"` for every error of this crate.
- `detail`: A private enum with one variant for each cause, which carries the data that the text of Section 10.2 shows: the canonical string of the signature, the order over the limit, or the error of `escher-geometry` that caused a numerical failure.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives the `Display` of the private detail. `Display` and `std::error::Error` for `Error` are written by hand, because a derived `source()` can return only a field of `Error`, not the `escher-geometry` error that one variant of the detail holds, and a separate source field beside the detail would hold one fact in two fields. `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail. `source()` returns the `escher-geometry` error for a `Numerical` error that one caused, and `None` otherwise. No type from a dependency appears in the public surface of the error (ERR-2).

`Numerical` is an error and not a panic. No supported signature is known to produce it, but the construction calls fallible functions of `escher-geometry`, and a panic would stop `escher info`, so returning an error is the sounder option (ERR-5 of the coding standard, Hard Rule 7 of `AGENTS.md`).

### 10.2. Kinds and Text

DOM-13. `Display` for `Error` MUST be `cannot build fundamental domain: ` followed by the text of the detail in this table, because `escher-cli` prints it and its snapshots fix it.

| Kind | When | Detail carries | Text after the prefix |
| --- | --- | --- | --- |
| `UnsupportedSignature` | The predicate of Section 4 fails, which includes every general non-compact signature (`DEC-19`) | The canonical string | `the signature *632 is not supported`, with the canonical string of the signature in place of `*632` |
| `LimitExceeded` | A corner order of a supported signature exceeds 1000 | The largest corner order | `order 1001 is larger than 1000`, with the largest corner order in place of `1001` |
| `Numerical` | Step 11 of Section 6.1 | The `escher-geometry` error, if one caused it | `the construction is numerically degenerate` |

So `FundamentalDomain::build` on `*632` returns an `Error` whose `kind()` is `UnsupportedSignature` and whose `Display` text is exactly `cannot build fundamental domain: the signature *632 is not supported`. On `*(1001)32` it returns `LimitExceeded` with the text `cannot build fundamental domain: order 1001 is larger than 1000`. What `escher-cli` prints for either, and its exit status, are for the command-line design to state.

Every fixed text is in lower case and has no trailing punctuation (ERR-4 of the coding standard). The canonical string contains only printable ASCII (Section 7.3 of the signature design), so the text needs no escaping. The `Display` text does not repeat the text of the `source()` error (ERR-4).

## 11. Resource Limits

The crate limits the largest corner order. `build` checks it at step 2 of Section 6.1. Section 16 states the value and its reason, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 12. Testing

### 12.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-domain/tests/domain/main.rs`, with the modules `common`, `reference`, `triangles`, `regular`, `properties`, and `unsupported` (TEST-10). Its root has a `//!` line. The snapshot tests of the domain summary live in `escher-cli`, and the command-line design specifies them.

The module `common` defines the sets of triples that the tests use:

- The plan triples: $(7, 3, 2)$, $(5, 4, 2)$, $(4, 4, 4)$, $(4, 3, 3)$, $(12, 3, 2)$, and $(1000, 3, 2)$. The plan writes $(3, 3, 4)$, whose canonical signature is `*433`.
- The 24 worst cases: $(995, 983, 970)$, $(994, 991, 970)$, $(997, 996, 970)$, $(997, 980, 973)$, $(992, 990, 979)$, $(995, 987, 976)$, $(928, 4, 3)$, $(999, 7, 6)$, $(998, 4, 4)$, $(1000, 4, 4)$, $(1000, 1000, 2)$, $(1000, 1000, 1000)$, $(868, 4, 4)$, $(923, 6, 5)$, $(998, 981, 961)$, $(998, 998, 958)$, $(997, 931, 906)$, $(992, 991, 975)$, $(989, 976, 974)$, $(978, 977, 971)$, $(985, 985, 971)$, $(973, 972, 964)$, $(979, 979, 971)$, and $(987, 984, 970)$. They include every triple at which the sweeps of Sections 8.4 and 9.3 have a largest value.
- The 30 named triples: the plan triples and the worst cases.
- The orders up to 12: all 269 hyperbolic triples $n_0 \ge n_1 \ge n_2$ with orders up to 12.
- The set of 294 triples: the orders up to 12 and the 25 named triples with an order above 12.

Each test names the item of the field "Tests and acceptance" of Section 5.2 of the plan that it serves. The tests "Triangles", "Third side", "Checks at the worst cases", "Relator agreement", and the two tests of $\{7, 3\}$ also assert their smallest number of samples, and print each largest value with its triple and its margin. The test harness captures that output for a test that passes:

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
| Failing domains | Failure paths | Unit tests in `verify.rs` | The domains of $(7, 3, 2)$, $(4, 4, 4)$, and $(1000, 3, 2)$, built through private access, pass every check. For each defect of Section 8.3, on the triangles of its row, exactly the checks of the row fail and `passed()` is false. For a row that names an infinite residual, each failed check has the residual `f64::INFINITY`, and no check has a NaN residual |
| Check results | Failure paths | Unit tests in `verify.rs` | A `Check` built through private access with a NaN residual fails, a residual equal to the tolerance passes, and an infinite residual fails |
| Unsupported | Failure paths | Table test, `unsupported` | `*632`, `*532`, `*3333`, `732`, `4*3`, `*(inf)32`, `*(inf)(inf)`, `(inf)(inf)`, `*22(inf)`, `1`, `*`, `o`, `*732*`, and `x732` return `UnsupportedSignature` with the text of Section 10.2. `*(1001)32` returns `LimitExceeded` with its text, `*(1000)32` builds, and `*(1001)(1001)(1001)(1001)` returns `UnsupportedSignature` |
| Limit | Failure paths | Unit tests in `domain.rs` | With the private limit constant: `*(1000)32` builds and `*(1001)32` returns `LimitExceeded`, and `*(1001)(1001)` returns `UnsupportedSignature`, because support is checked before the limit |
| Sides and vertices | None | Unit test in `domain.rs` | For `*732`, each side is its own partner, and `hyperbolic::side` of its covector at each of its two vertices is below $10^{-12}$ in absolute value |
| Corner order and area | None | Unit tests in `triangle.rs` | The private predicate of Section 4 gives the orders $(7, 3, 2)$ and the area `PI / 42.0` for `*237`, and gives nothing for `*632`, `*3333`, `732`, and `*(inf)32`. The area is compared exactly, because the numerator 1 and the denominator 42 convert exactly and multiplying by 1 is exact, so IEEE 754 arithmetic computes the same quotient as `PI / 42.0` (TEST-13 of the coding standard) |
| Conversion | None | Unit test in `triangle.rs` | The private signed conversion of step 1 of Section 6.1 gives exactly 0, $-1$, and $-4294967297$ for those integers, $2^{53}$ for $2^{53} + 1$ by ties to even, $2^{63}$ for `i64::MAX`, and $-2^{63}$ for `i64::MIN`. The unsigned conversion gives exactly 1 and $4294967297$ for those integers, and $2^{64}$ for `u64::MAX`. The test compares exactly, because the sum of the two parts is the only rounding, and IEEE 754 arithmetic fixes the result of that rounding to nearest, bit for bit (TEST-13 of the coding standard) |
| Numerical | Failure paths | Unit test in `triangle.rs` | The private construction of steps 3 to 11, given the spherical orders $(5, 3, 2)$, fails with the `escher-geometry` error of kind `NotAPoint`, which `build` reports as `Numerical`, because the law of cosines then gives a hyperbolic cosine below 1 |
| Any signature | Failure paths | `proptest`, `properties` | For any text that parses as a signature, `build` returns without a panic, and a returned domain passes `verify()` |
| Error text | Failure paths | Unit tests in `error.rs` | The `Display` of every kind starts with `cannot build fundamental domain: `, is in lower case apart from the canonical string, and is the text of Section 10.2. `kind()` matches each cause, and `source()` returns the `escher-geometry` error only for a `Numerical` error that one caused |
| Reference self-check | None | Reference test, `reference` | The four known values of Section 12.2 |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs` | Each public type is `Send`, `Sync`, and `'static` (CONC-1 of the coding standard) |
| Doctests | None | Doctests | The examples of the crate-level and item documentation |

The plan item "Properties" also names the inverse $J M^t J$ and the round trip of the disk map. Those are properties of `escher-geometry`, and the geometry design tests them.

The expected values for $\{7, 3\}$ come from the split of the polygon into $2p$ right triangles (Section 10.1 of the geometry reference), which puts the centre of the polygon at vertex 0 of `*732`, a vertex of the polygon at vertex 1, and the midpoint of an edge at vertex 2. The tests evaluate equations 29 and 24 of [HT] (Section 10.4 of the geometry reference) in the reference arithmetic, with the curvature radius $\ell = 1$. Computed from those equations, the edge length is 0.566256306735315 and the circumradius in the disk is 0.300742618746379. The tests measure an error of $1.548 \times 10^{-16}$ in the edge against its bound of $2 \times 10^{-13}$, and of $2.025 \times 10^{-16}$ in the circumradius against its bound of $10^{-13}$. No value that Table 2 of [HT] prints serves as an expected value, because that table and equation 29 disagree on the edge length of $\{7, 3\}$ (item 23 of Section 15.1 of the geometry reference).

DOM-14. A test MUST NOT take an expected value from Table 2 of [HT], because the research records that the table and equation 29 disagree for $\{7, 3\}$, and a test built on the wrong one passes a wrong domain or fails a correct one.

The property tests commit their regression files (TEST-9 of the coding standard).

### 12.2. Reference Arithmetic

The reference is the independent high-precision implementation of Section 7 of the architecture, with the precision and tolerances that this section states:

- Structure. A module `reference` in the test binary holds points as arrays of three high-precision numbers and writes each formula componentwise from the research documents. It calls no function of `escher-domain`, `escher-geometry`, or `glam`.
- Independence. The reference builds the standard position from $\cosh L$ and $\sqrt{\cosh^2 L - 1}$ without $\operatorname{arcosh}$, and centres the triangle with $\cos\varphi = c_1/h$, $\sin\varphi = c_2/h$, $\sinh\rho = h$, and $\cosh\rho = c_3$, where $h$ is the length of the first two coordinates of the centre, read from the polar form of Section 8.3 of the geometry reference. The crate takes the other path in both places, so a transcription error is not shared.
- Precision. 256 bits, rounding to nearest with ties to even.
- Comparison. As in Section 10.2 of the geometry design, each test converts the `f64` result exactly into the reference type, takes the difference there, and compares it with the tolerance there, because `astro-float-num` has no public conversion back to `f64`. A message that prints a measured error formats the number as decimal text.
- Self-check. The test "Reference self-check" in `reference` checks the reference library itself: $\pi$ against a literal with 80 digits after the decimal point, $\cos(\pi/3) = 1/2$, $\cosh(\operatorname{arcosh} 2) = 2$, and $\sqrt{2}^2 = 2$, each to $10^{-70}$. The largest of the four errors is $1.7 \times 10^{-77}$. It guards against a wrong precision or rounding argument and does not close `UNV-25`.

DOM-15. The reference module MUST share no code with `escher-domain` or `escher-geometry`, because a formula that the crate and its test share passes the test whether or not it is correct (Section 7 of the architecture).

### 12.3. Tests Under Miri

`DEC-5` and TEST-11 of the coding standard apply to the tests of this crate in this way:

| Tests | Under Miri | Attribute or configuration |
| --- | --- | --- |
| "Triangles", "Third side", "Edge of $\{7, 3\}$", "Circumradius of $\{7, 3\}$", "Relator agreement", "Relators in the domain frame", and "Reference self-check" | Ignored | `#[cfg_attr(miri, ignore = "the high-precision arithmetic runs loops too long for the interpreter")]` |
| "Checks at the worst cases" | Ignored | `#[cfg_attr(miri, ignore = "the checks of the largest orders run loops too long for the interpreter")]` |
| "Random triples" and "Any signature" | Run with 8 cases and no failure persistence | The helper of DOM-16 |
| The unit tests, which include "Failing domains" with the three tests of a corner order off by one and the tests of a negated vertex and of NaN entries, "Corner order and area", and "Conversion", and the table test "Unsupported" | Run unchanged | None |
| The doctests | Not run, because Miri runs the tests through nextest, which runs no doctest | None |

Each reason in an attribute is in plain words and cites no document (Hard Rule 15 of `AGENTS.md`).

No test asserts a bit-exact result of a transcendental function, because the interpreter need not reproduce the last bits of the host library, and every tolerance of this document leaves room for that.

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

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, and what the code does about it:

| Label | What the code relies on | What the code does |
| --- | --- | --- |
| `UNV-26` | The wall normals of step 9 of Section 6.1, through `hyperbolic::wall_through`, whose formula [RAT] prints with the time coordinate first | The test "Triangles" compares every normal with the reference, and checks 2, 3, and 4 test each wall against the vertices and the pairings |
| `UNV-26` | Nothing from its second part, the relation between the inner product of two normals and the corner angle | Measures the corner angles from the distances between the vertices by the law of cosines (Section 7, DOM-6), so the angle check does not depend on that relation |
| `UNV-25` | The correct rounding of the reference crate, in tests only | Uses 256 bits against tolerances of $10^{-13}$ at the tightest, and the self-check of Section 12.2. One comparison with MPFR closes it (Section 15.3) |

`UNV-28` does not apply, because the design uses SymmHub's triangle neither in the code nor as a test oracle (Section 6.3). `UNV-4` concerns the precision of the disk map near the limit circle, and the crate maps to the disk only in the test "Circumradius of $\{7, 3\}$", at a point near the origin.

These statements of this design rest on figures that no measurement of the code covers. They come from a double-precision model of the algorithm outside the repository, and each supports only the choice that it names:

| Statement | Section | Figures of the model |
| --- | --- | --- |
| A domain left with vertex 2 at the origin has larger normal components and fails checks that the final position passes | 6.3 | At $(1000, 3, 2)$, a largest normal component of 1 and a relator residual of $7.28 \times 10^{-12}$, which passes. At $(1000, 100, 100)$, 63.6 and $6.23 \times 10^{-4}$, which fails. At $(1000, 1000, 1000)$, 637 and an infinite residual. In the family $(n, n, n)$, $(60, 60, 60)$ and $(70, 70, 70)$ fail checks 1 and 2, $(80, 80, 80)$ also fails check 4, and every larger order fails. The families $(n, 3, 2)$, $(n, 4, 4)$, and $(n, n, 2)$ fail no check up to $n = 1000$ |
| Repeated multiplication evaluates the relator more accurately than repeated squaring | 8.2 | At corner 0 of $(1000, 3, 2)$, a largest entry of $C^{1000} - I$ of $3.46 \times 10^{-9}$ in the domain frame by repeated multiplication and $7.77 \times 10^{-8}$ by repeated squaring, against $2.33 \times 10^{-11}$ in the corner frame by either method |
| The residual of check 6 stays close to that of check 5 | 8.2 | Never more than 2.91 times the residual of check 5, over 300 random triples |
| Renormalizing the vertices after step 8 makes them less accurate (DOM-10) | 9.1 | At $(1000, 1000, 1000)$, renormalization raised the largest relative vertex error from $5.7 \times 10^{-11}$ to $8.9 \times 10^{-6}$ and the angle error from $1.9 \times 10^{-13}$ to $2.8 \times 10^{-8}$ |

## 15. Assumptions, Choices, and Points for the Owner

### 15.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- The triangle placed by the two sides $L_a$ and $L_b$ with the included angle $\alpha_2$ is the triangle with the angles $\alpha_0$, $\alpha_1$, and $\alpha_2$. The research states uniqueness given the three angles (Section 10.1 of the geometry reference), but no statement on two sides and the included angle. The test "Third side" and checks 4 and 5 check it.
- The angle at a vertex is the arccosine of $(\cosh a \cosh b - \cosh c) / (\sinh a \sinh b)$. This is the first law of cosines of Section 10.2 of the geometry reference solved for the angle, which no source prints in this form. The test "Triangles" compares the angles with $\pi/n_j$ in the reference.
- The normalized sum of the vertices lies strictly inside the triangle. Check 2 fails if it does not, and the test "Random triples" checks it.
- A power of a matrix is the identity exactly when the same power of its conjugate is. This is matrix algebra, not a statement of the geometry reference. The test "Relator agreement" checks it on the 30 named triples.
- Side 0, side 1, and side 2 are the Latin reflections $P$, $Q$, and $R$ of step 3 of Section 9.1 of the notation reference, and its fourth reflection $S$ is $P$ again for a triangle. The relators that the checks use come from Section 4.4 of the domain reference and do not depend on this reading.
- `hyperbolic::translation(c)` maps `hyperbolic::ORIGIN` to the point with the spatial coordinates of $c$, as Section 6.4 of the geometry design states, and this is $c$ itself only when $c$ lies on the hyperboloid. Section 13.1 of the geometry design records this property as an assumption and names the test that checks it.

### 15.2. Choices of This Design

No other document dictates these choices:

| Choice | Section |
| --- | --- |
| `FundamentalDomain::build` as an inherent constructor with one argument that returns a `Result` | 3.1, 3.4 |
| The measurements `side_lengths`, `corner_angles`, and `area` as methods of the domain, so that `escher-cli` needs neither `glam` nor `escher-geometry` | 3.1, 7 |
| `Orientation` with both variants, and `PairingKind` with one | 3.2 |
| The domain stores the geometry, the three corner orders, and the area of its signature, and its vertices, sides, and centre, although all of them follow from the signature. They are the result of the construction, which the renderer would otherwise repeat, and the type is immutable, so they cannot diverge. It stores no signature, because `build` borrows it and the checks need only these values. A reviewer checks this against TYPE-3 of the coding standard | 3.2, 6.1 |
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
| Every tolerance of Sections 8.4 and 9.3 | 8.4, 9.3 |
| 256 bits for the reference arithmetic | 12.2 |
| The sets of triples of Section 12.1, and the weighting of the test "Random triples" toward large orders | 12.1 |
| `astro-float-num` in place of `astro-float` | 13 |

### 15.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Reference crate. `DEC-3` names `astro-float`. The design uses `astro-float-num` (Section 13). The alternative is a `skip` entry in `deny.toml` for `syn` 1 with a reason. Section 13.3 of the geometry design raises the same point.
2. Largest order and the tolerance of check 4. The design supports corner orders up to 1000 and returns `LimitExceeded` above it (DOM-3). It sets the tolerance of check 4 to $2 \times 10^{-6}$, because the largest residual of that check would leave a margin of 7.5 at $10^{-6}$, below the 8 that TEST-13 of the coding standard requires, and the margin at $2 \times 10^{-6}$ is 15.0, with check 4 still failing every triangle near the limit with one corner order off by one (Sections 8.3 and 8.4). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin.
3. Comparison with MPFR for `UNV-25`. The geometry design records the comparison (its Section 12) and asks whether it closes `UNV-25`. This design relies on the same answer.
4. Check 6. It detects nothing that check 5 does not, apart from a disagreement with `escher-signature` (Section 8.2). The design keeps it because the plan requires an area check. The alternatives are to drop it, or to measure the area by an independent formula from the vertices, which needs a source that the research does not hold.
5. Placement. The final position makes the centre of the canonical view a property of the domain, while Section 3.2 of the architecture places the canonical placement in `View`. `View` can still move the domain, because `centre()` is public.

## 16. Security Considerations

The threat model is a caller that passes a `Signature` parsed from untrusted text to `FundamentalDomain::build`, and then calls `verify()`. `escher-signature` has already rejected malformed text and bounded the signature (Section 16 of the signature design). The attacker can choose any valid signature. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of time or memory through a large order, a panic that stops the caller, and a wrong domain that the caller uses as correct. The crate counters them with the limit of this section, the rule that it never panics (DOM-2 and DOM-8), and the checks of Section 8.

DOM-3 requires the limit of this table. The input is untrusted, the cost of `verify()` grows with the order, and the accuracy of Section 9.3 is established only up to that order:

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Largest corner order | 1000 | The construction costs the same for every order. `verify()` costs at most about 3000 matrix products at this limit (Section 8.2), and the relator check keeps a margin of 15.0 at its tolerance of $2 \times 10^{-6}$ over every supported triple (Section 8.4). The plan tests up to this order. The number is a choice of this design (Section 15.3). A higher limit needs a construction of the far walls that is better conditioned, which the research does not source, and new measurements of every margin |

The limit is a private constant, and unit tests inside the crate read it. `build` allocates three sides and three vertices, so the memory of a domain does not depend on the input.

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
| A degenerate construction | `Numerical` | None. No supported signature is known to produce it, and one that does is a defect of the crate |
| A check outside its tolerance | `Check::passed` is false | None. It is a defect of the crate, and the residual and the tolerance locate it |

## 18. References

### 18.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 18.2. Informative References

The tags of the mathematical sources resolve to the reference sections of the research documents:

- [BEA] Section 16.1 of `docs/research/geometry-models.md`.
- [CFKP] Section 16.1 of `docs/research/geometry-models.md`.
- [DAVIS] Section 16.1 of `docs/research/geometry-models.md`.
- [DAVIS2008] Section 14.1 of `docs/research/fundamental-domains.md`.
- [EPSTEIN-PETRONIO1994] Section 14.1 of `docs/research/fundamental-domains.md`.
- [HT] Section 16.1 of `docs/research/geometry-models.md`.
- [KC24] Section 16.1 of `docs/research/geometry-models.md`.
- [MASKIT2001] Section 14.1 of `docs/research/fundamental-domains.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [TH] Section 13.1 of `docs/research/orbifold-notation.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| DOM-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| DOM-2 | `build` constructs a domain only for a signature that meets the predicate of Section 4, and returns `UnsupportedSignature` for every other valid signature without a panic | 4 |
| DOM-3 | `build` returns `LimitExceeded` for a supported signature with an order above 1000, after the support check | 4 |
| DOM-4 | Vertices and sides follow the index convention of Section 5.1 | 5.1 |
| DOM-5 | `build` constructs the domain by the steps of Section 6.1, in order | 6.1 |
| DOM-6 | No relation between the inner product of two normals and the corner angle is used | 6.1 |
| DOM-7 | `verify()` computes and reports every check on every call | 8.1 |
| DOM-8 | `verify()` is pure, never panics, and renormalizes nothing | 8.1 |
| DOM-9 | Each check uses the tolerance of Section 8.4 | 8.4 |
| DOM-10 | The vertices are not renormalized after centring | 9.1 |
| DOM-11 | The values that `build` returns are within the bounds of Section 9.3 | 9.3 |
| DOM-12 | `Error` follows ERR-1, with the operation that failed | 10.1 |
| DOM-13 | The `Display` text of each kind is the one stated | 10.2 |
| DOM-14 | No test takes an expected value from Table 2 of [HT] | 12.1 |
| DOM-15 | The reference module shares no code with the crates under test | 12.2 |
| DOM-16 | Property tests take their configuration from one helper that is Miri-aware | 12.3 |
| DOM-17 | The crate declares only the dependencies of Section 13, each from the workspace table | 13 |
