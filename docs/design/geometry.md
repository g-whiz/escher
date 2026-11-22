# Geometry Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-22 |
| Applies to | `crates/escher-geometry` |

## Abstract

This document specifies the crate `escher-geometry`, which holds the linear model of the hyperbolic plane, with points on the hyperboloid, walls as unit normals, and isometries as 3x3 matrices, the affine model of the Euclidean plane, with points on the plane of last coordinate 1, lines as covectors, and isometries as 3x3 matrices, and the unit sphere, with points as unit vectors, great circles as unit normals, and isometries as orthogonal 3x3 matrices, over the double-precision vector and matrix types of `glam`. It defines the public functions and their domains, the formula behind each function with the convention of its source, the numerical form that each function uses and its behaviour at the edge of its domain, the map between the hyperboloid and the Poincare disk in both directions, the error type, the accuracy that the crate guarantees, and the layers of the tests, including the tests against high-precision reference arithmetic. It also states the dependencies, the unverified statements that the code relies on, the assumptions and choices of this design, and the points that the owner decides.

## 1. Introduction

### 1.1. Problem

Every later crate computes with points, walls, and isometries of a geometry. The research states each formula in the convention of its source, and the sources disagree on the position of the time coordinate, the sign of the bilinear form, the direction of a normal, and whether a matrix takes the full or half distance (Section 2.4 of the reference). A formula taken from one source and evaluated in the convention of another gives a wrong result without any error. Floating-point evaluation adds a second risk: a form that cancels near the limit circle or for close points loses digits that no later step recovers.

### 1.2. Purpose

`escher-geometry` is the one place where the conventions of Section 5 of the architecture become code. It exposes the bilinear form, the side test, point normalization, distance, the wall through two points, the basic isometries, the inverse, an isometry check, and the disk map, each as a free function over `glam` types. Its module `euclidean` exposes the side test, the distance, the line through two points, the reflection, the rotation, the translation, and an isometry check of the Euclidean plane. Its module `spherical` exposes the side test, point normalization, distance, the great circle through two points, the basic isometries, the inverse, and an isometry check of the sphere. Each function evaluates a formula in a numerical form chosen for its domain. A formula that the reference does not state in the project's convention is flagged as unverified or as an assumption, and a test covers it. Section 3 of the architecture places the crate in the workspace as a root with no internal dependency.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-domain` and `escher-render`, which call the crate, and the authors of the later changes that add further model maps.

### 1.4. Scope

This document covers the modules `hyperbolic`, `euclidean`, and `spherical`, the crate root with its error type, the disk map in both directions, the numerical robustness of each function, the tests, and the dependencies.

### 1.5. Non-Goals

These topics are owned elsewhere or come later:

- The fundamental domain, its construction, its verification, and every tolerance of that verification are specified in `docs/design/domain.md`.
- The fold, its wall tolerance, and its normalization policy belong to `escher-render` (Sections 9.2 and 9.3 of the architecture).
- The lift of a screen position onto the ball, its tangents, and the viewing rotation of the sphere belong to `escher-render` and are specified in Section 5.9 of `docs/design/render.md`. The module `spherical` has no model map.
- The Euclidean functions that no caller needs, such as a form, an inverse, and a point normalization, are not part of the module `euclidean` (Section 3.4).
- The Klein, half-plane, and band maps and the Jacobians of the model maps are added by the milestone of Section 5.12 of the plan. That change also records the image of the disk centre under the map to the half-plane and tests its round trip, as the row "Disk and half-plane" of Section 5.10 of the architecture requires.
- A distance form that is accurate for close points is not part of this design (Section 5.2).
- The command line and its text are specified in `docs/design/cli.md`.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md`, the terms of Section 2.3 of `docs/research/geometry-models.md`, such as model, geodesic, isometry, Lorentz matrix, and boost, and the terms linear model, model map, wall, covector, side, and reference arithmetic of Section 2.1 of `docs/design/architecture.md`. It cites `docs/research/geometry-models.md` as the reference, `docs/design/architecture.md` as the architecture, `docs/design/poc-plan.md` as the plan, and `docs/rust-coding-standards.md` as the coding standard. The citation tags of mathematical sources, such as [CFKP] and [KC24], are those of the reference.

It adds these terms:

- form: The Minkowski bilinear form $\langle x, y \rangle = x_1 y_1 + x_2 y_2 - x_3 y_3$ of Section 5.1 of the architecture, which is $x \ast y$ of [CFKP] in the reference.
- time coordinate: The coordinate with the negative sign in the form. In this crate it is the last coordinate, `.z` of a `DVec3`.
- hyperboloid: The set $\lbrace x : \langle x, x \rangle = -1, \ x_3 > 0 \rbrace$.
- point: A `DVec3` that lies on the hyperboloid up to rounding.
- unit normal: A `DVec3` $u$ with $\langle u, u \rangle = 1$ up to rounding, which represents the wall $\lbrace x : \langle x, u \rangle = 0 \rbrace$.
- isometry: A `DMat3` that meets the two conditions of Section 6.2, up to rounding.
- disk position: A `DVec2` in the open unit disk, a point of the Poincare disk model.
- Euclidean point: A `DVec3` $(x, y, 1)$, a point of the Euclidean plane as the affine plane $x_3 = 1$ of Section 5.1 of the architecture.
- line covector: A `DVec3` $(a_1, a_2, -t)$ with $a_1^2 + a_2^2 = 1$ up to rounding, which represents the line $P(a, t) = \lbrace p : a \cdot p = t \rbrace$ of Section 12 of the reference.
- Euclidean isometry: A `DMat3` whose last row is $(0, 0, 1)$ and whose upper-left $2 \times 2$ block is orthogonal, up to rounding.
- sphere point: A `DVec3` $x$ with $x \cdot x = 1$ up to rounding, a point of the unit sphere $S^2$ of Section 4.6. Its last coordinate may have any sign.
- great-circle normal: A sphere point $u$, which represents the great circle $\lbrace x : u \cdot x = 0 \rbrace$ of $S^2$. It is its own covector (Section 4.6).
- spherical isometry: A `DMat3` $M$ with $M^t M = I$ up to rounding, an orthogonal matrix of determinant $1$ or $-1$.
- $J$: The matrix $\operatorname{diag}(1, 1, -1)$ of the form, as Section 2.2 of the reference defines it.
- $\varepsilon$: The machine epsilon of `f64`, `f64::EPSILON`, which is $2^{-52}$.
- reference value: A value computed in the reference arithmetic of Section 10.2 from the exact `f64` values of the inputs, and of the result where the quantity names it.
- $\kappa$: For a vector $x$, the reference value of $x_3^2 / (-\langle x, x \rangle)$. It scales the accuracy of `normalize_point`.
- $K$: For two points $a$ and $b$, the reference value of $a_3 b_3 / \sinh d$, where $d$ is the distance between the exact `f64` values of $a$ and $b$, the reference value of $\operatorname{arcosh}(\max(1, -\langle a, b \rangle))$. A test computes it from its inputs. It scales the accuracy of `wall_through`.
- $\sigma_i$: For a vector $u$, its covector $c$, and a vector $x$, the reference value of $\lvert x_i \rvert + 2 \lvert u_i \rvert \sum_j \lvert c_j x_j \rvert$, the sum of the absolute values of the terms of coordinate $i$ of the matrix product `reflection(u) * x`. It scales the accuracy of `reflection`.
- $s_i$: For coordinate $i$ of a formula, the largest absolute value among the terms of the formula for that coordinate, as a reference value.
- $\mu$: For a matrix, the largest absolute value of its entries.
- $g$: For a product of matrices, the product of the values $\mu$ of its factors.
- total function: A function that returns a value for every `f64` input, without a panic. Outside its stated domain the value is unspecified.

## 3. Public Surface

### 3.1. Items

GEO-1. The crate MUST export exactly the items of this section, each through one path, and MUST define no vector or matrix type of its own, because `DEC-4` makes the `glam` types the representation of every crate, and a second path or a second type lets two crates disagree on what a point is (API-10 of the coding standard).

This listing shows each public item with its signature. Bodies are omitted:

```rust
// crate root: escher_geometry
pub mod euclidean;
pub mod hyperbolic;
pub mod spherical;

pub struct Error { /* private */ }
// derives Clone, Debug, PartialEq, Eq, and std::error::Error and Display through thiserror
impl Error {
    #[must_use] pub fn kind(&self) -> ErrorKind;
}

#[non_exhaustive]
pub enum ErrorKind { NotAPoint, DegenerateWall }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;

// escher_geometry::hyperbolic
use glam::{DMat3, DVec2, DVec3};

pub const ORIGIN: DVec3;

#[must_use] pub fn form(x: DVec3, y: DVec3) -> f64;
#[must_use] pub fn covector(normal: DVec3) -> DVec3;
#[must_use] pub fn side(covector: DVec3, point: DVec3) -> f64;
pub fn normalize_point(vector: DVec3) -> Result<DVec3>;
#[must_use] pub fn distance(first: DVec3, second: DVec3) -> f64;
pub fn wall_through(first: DVec3, second: DVec3) -> Result<DVec3>;
#[must_use] pub fn boost(length: f64) -> DMat3;
#[must_use] pub fn rotation(angle: f64) -> DMat3;
#[must_use] pub fn translation(target: DVec3) -> DMat3;
#[must_use] pub fn reflection(normal: DVec3) -> DMat3;
#[must_use] pub fn inverse(isometry: DMat3) -> DMat3;
#[must_use] pub fn isometry_defect(matrix: DMat3) -> f64;
#[must_use] pub fn to_disk(point: DVec3) -> DVec2;
#[must_use] pub fn from_disk(position: DVec2) -> Option<DVec3>;

// escher_geometry::euclidean
use glam::{DMat3, DVec3};

pub const ORIGIN: DVec3;

#[must_use] pub fn side(covector: DVec3, point: DVec3) -> f64;
pub fn wall_through(first: DVec3, second: DVec3) -> Result<DVec3>;
#[must_use] pub fn reflection(covector: DVec3) -> DMat3;
#[must_use] pub fn rotation(angle: f64) -> DMat3;
#[must_use] pub fn translation(target: DVec3) -> DMat3;
#[must_use] pub fn distance(first: DVec3, second: DVec3) -> f64;
#[must_use] pub fn isometry_defect(matrix: DMat3) -> f64;

// escher_geometry::spherical
use glam::{DMat3, DVec3};

pub const ORIGIN: DVec3;

#[must_use] pub fn side(covector: DVec3, point: DVec3) -> f64;
pub fn normalize_point(vector: DVec3) -> Result<DVec3>;
#[must_use] pub fn distance(first: DVec3, second: DVec3) -> f64;
pub fn wall_through(first: DVec3, second: DVec3) -> Result<DVec3>;
#[must_use] pub fn boost(angle: f64) -> DMat3;
#[must_use] pub fn rotation(angle: f64) -> DMat3;
#[must_use] pub fn translation(target: DVec3) -> DMat3;
#[must_use] pub fn reflection(normal: DVec3) -> DMat3;
#[must_use] pub fn inverse(isometry: DMat3) -> DMat3;
#[must_use] pub fn isometry_defect(matrix: DMat3) -> f64;
```

The module `hyperbolic` has one constant and 14 functions, the module `euclidean` has one constant and 7 functions, and the module `spherical` has one constant and 10 functions. The crate root re-exports `Error`, `ErrorKind`, and `Result` from the private module `error`, and nothing else.

### 3.2. Meaning and Invariants

Section 3.2 and `DEC-4` of the architecture decide that a point, a unit normal, and a covector are a `DVec3`, an isometry is a `DMat3`, and a position in a planar model is a `DVec2`. This crate adds no wrapper around them, so a `DVec3` carries no proof that it is a point. Each function therefore states its domain:

| Item | Meaning | Domain | Result |
| --- | --- | --- | --- |
| `ORIGIN` | The point $C_0 = (0, 0, 1)$ | Not applicable | Exact |
| `form` | $\langle x, y \rangle$ | Any vectors | Total |
| `covector` | The vector $c$ with $c \cdot x = \langle x, u \rangle$ for every $x$ | Any vector | Total and exact |
| `side` | The side test $\langle x, u \rangle$ from the covector $c$ of $u$ | A covector and a point | Total. Positive means outside the wall of an outward normal (Section 5.2 of the architecture) |
| `normalize_point` | The point on the ray of the vector | A vector with $-\langle x, x \rangle$ positive and finite, and $x_3 > 0$ | A point, or `NotAPoint` (Section 5.1) |
| `distance` | The hyperbolic distance $d(x, y)$ | Two points | Total. At least 0 |
| `wall_through` | A unit normal of the wall that contains both points | Two distinct points | A unit normal, or `DegenerateWall` (Section 5.3) |
| `boost` | The boost $T^\alpha$ along the first axis by the full distance $\alpha$ | Any finite length | An isometry |
| `rotation` | The rotation $R^\alpha$ about `ORIGIN` by the angle $\alpha$ in radians | Any finite angle | An isometry |
| `translation` | An isometry that maps `ORIGIN` to the point with the spatial coordinates of the target | A point. Its last coordinate is not read | An isometry (Section 6.4) |
| `reflection` | The reflection in the wall of a unit normal | A unit normal. It is not checked | An isometry of determinant $-1$ when the normal is a unit normal |
| `inverse` | The inverse of an isometry | An isometry | Total and exact (Section 6.1) |
| `isometry_defect` | How far a matrix is from an isometry | Any matrix | Total. 0 for an exact isometry, and `f64::INFINITY` when the second condition of Section 6.2 fails or the result is not a number |
| `to_disk` | The Poincare disk position of a point | A point | Total |
| `from_disk` | The point of a disk position | Any position | `Some` point inside the disk, and `None` on and outside the unit circle (Section 7.2) |
| `euclidean::ORIGIN` | The point $(0, 0, 1)$ | Not applicable | Exact |
| `euclidean::side` | The side value $c \cdot x = a \cdot p - t$ of the point $p$ of $x$ for the line covector $c$ | A line covector and a Euclidean point | Total. Positive means outside a line whose covector is oriented outward (Section 5.2 of the architecture) |
| `euclidean::wall_through` | A line covector of the line through both points | Two distinct Euclidean points | A line covector, or `DegenerateWall` (Section 5.4) |
| `euclidean::reflection` | The reflection in the line of a covector | A line covector. It is not checked | A Euclidean isometry of determinant $-1$ when the covector is a line covector (Section 6.6) |
| `euclidean::rotation` | The rotation about `euclidean::ORIGIN` by the angle in radians | Any finite angle | A Euclidean isometry, the same matrix as `hyperbolic::rotation` (Section 6.6) |
| `euclidean::translation` | The translation that maps `euclidean::ORIGIN` to $(t_1, t_2, 1)$ | Any finite target. Its last coordinate is not read | A Euclidean isometry (Section 6.6) |
| `euclidean::distance` | The Euclidean distance $d_E$ | Two Euclidean points. The last coordinates are not read | Total. At least 0, or NaN (Section 5.4) |
| `euclidean::isometry_defect` | How far a matrix is from a Euclidean isometry | Any matrix | Total. 0 for an exact Euclidean isometry, and `f64::INFINITY` when an entry is not finite or the result is not a number (Section 6.6) |
| `spherical::ORIGIN` | The point $C_0 = (0, 0, 1)$ of the sphere | Not applicable | Exact |
| `spherical::side` | The side value $u \cdot x$ of the point $x$ for the great-circle normal $u$, passed as its covector | A great-circle normal and a sphere point | Total. Positive means outside a great circle whose normal is oriented outward (Section 5.2 of the architecture) |
| `spherical::normalize_point` | The sphere point on the ray of the vector | A vector with $v \cdot v$ positive and finite | A sphere point, or `NotAPoint` (Section 5.5) |
| `spherical::distance` | The spherical distance $d_S(x, y)$ | Two sphere points | Total. In $[0, \pi]$, or NaN for a NaN input (Section 5.5) |
| `spherical::wall_through` | A great-circle normal of the great circle that contains both points | Two sphere points that are neither equal nor antipodal | A great-circle normal, or `DegenerateWall` (Section 5.5) |
| `spherical::boost` | The translation $T^\alpha$ of [KC24] along the first axis by the angle $\alpha$ in radians | Any finite angle | A spherical isometry of determinant 1 (Section 6.7) |
| `spherical::rotation` | The rotation $R^\alpha$ about `spherical::ORIGIN` by the angle in radians | Any finite angle | The same matrix as `hyperbolic::rotation` (Section 6.7) |
| `spherical::translation` | A spherical isometry that maps `spherical::ORIGIN` to the sphere point on the ray of the target | Any finite target. All three coordinates are read | A spherical isometry of determinant 1, and the identity for every zero vector (Section 6.7) |
| `spherical::reflection` | The reflection in the great circle of a normal | A great-circle normal. It is not checked | A spherical isometry of determinant $-1$ when the normal is a sphere point (Section 6.7) |
| `spherical::inverse` | The inverse of a spherical isometry | A spherical isometry | Total and exact, the transpose (Section 6.7) |
| `spherical::isometry_defect` | How far a matrix is from a spherical isometry | Any matrix | Total. 0 for an exact spherical isometry, and `f64::INFINITY` when an entry is not finite or the result is not a number (Section 6.7) |
| `Error` | Why a vector is not a point, or two points span no wall | Not applicable | Holds no `glam` type (ERR-2 of the coding standard) |
| `ErrorKind` | What a caller matches on | Not applicable | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

GEO-2. Every function of the crate MUST return without a panic for every input, including NaN, infinities, and subnormal values, because the inputs derive from untrusted signatures and options, and a panic stops the caller's whole program (ERR-5 of the coding standard). A total function MAY return any value for an input outside its domain, including NaN.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers `Error` and `ErrorKind` (CONC-1 of the coding standard). The `glam` types are `Copy`, so every function takes its arguments by value.

### 3.3. Source Layout

The crate has these modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, `pub mod euclidean;`, `pub mod hyperbolic;`, `pub mod spherical;`, the re-exports of `Error`, `ErrorKind`, and `Result`, and the unit test "Auto traits" of Section 10.1.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/hyperbolic.rs`: The constant, the 14 functions, and their unit tests. The file form, not `hyperbolic/mod.rs`, is what the workspace lint `mod_module_files` requires.
- `src/euclidean.rs`: The constant, the 7 functions of the module `euclidean`, and their unit tests.
- `src/spherical.rs`: The constant, the 10 functions of the module `spherical`, and their unit tests.

### 3.4. Modules of the Geometries

Each geometry has one module: `hyperbolic`, `euclidean`, and `spherical`. A module uses the name of `hyperbolic` for the same operation, such as `side`, `reflection`, `rotation`, `translation`, `distance`, and `isometry_defect`, so that a caller changes only the module path. A module holds only the functions that a caller needs, so `euclidean` has no `form`, `covector`, `normalize_point`, `inverse`, `boost`, or model map, and `spherical` has no `form`, `covector`, or model map, because its form is the dot product and a great-circle normal is its own covector. The crate has no trait over the modules and no `Geometry` enum, because the enum lives in `escher-signature` (Section 3.4 of the architecture, `DEC-26`), and no caller is generic over the geometry: the fold of `escher-render` uses dot products and matrix products of the `glam` types, which serve every module, and only its point size depends on the geometry (Section 5.3 of `docs/design/render.md`), and SIMP-1 of the coding standard forbids an abstraction that gives no benefit now.

Model maps are free functions named `to_<model>` and `from_<model>` in the module of their geometry, such as `to_disk` and `from_disk` in `hyperbolic`. A further map, such as one to the Klein model, is a new pair of functions in the same module.

## 4. Conventions

### 4.1. Types and Storage

- A point, a unit normal, and a covector are a `DVec3` with the time coordinate as `.z`, or in the module `euclidean` the last coordinate of Section 4.5, or in the module `spherical` the unit vector of Section 4.6. An isometry is a `DMat3`. A disk position is a `DVec2`.
- Vectors are columns. `matrix * vector` applies the isometry to the vector. `DMat3` is column-major, so `DMat3::from_cols(c0, c1, c2)` takes the images of the three basis vectors.
- Composition is the `glam` product `a * b`, which applies `b` first. The reference states that composition of isometries of the hyperboloid is the matrix product (Section 6.1 of the reference, [CFKP] Theorem 10.2).

GEO-3. The crate MUST evaluate every formula in the conventions of Sections 5.1 and 5.2 of the architecture, with the time coordinate last, the form of Section 2.1, and outward normals in the module `hyperbolic`, the conventions of Section 4.5 in the module `euclidean`, and those of Section 4.6 in the module `spherical`, and MUST NOT expose a function in another convention, because a caller cannot see the convention of a `DVec3`, and a value in the wrong convention gives a wrong image without any error.

### 4.2. Bilinear Form, Covector, and Side Test

The form is the case $n = 2$ of $x \ast y$ of [CFKP], Section 3, p. 64, which puts the time coordinate last (Section 2.2 of the reference):

$$\langle x, y \rangle = x_1 y_1 + x_2 y_2 - x_3 y_3$$

`form` computes this sum of three products in this order. `ORIGIN` is $C_0 = (0, 0, 1)$ of [KC24], p. 3 (Section 2.2 of the reference).

The covector of a wall is the linear form $x \mapsto \langle x, u \rangle$ (Section 2.1 of the architecture). With the matrix $J$ of the form (Section 2.2 of the reference), `covector` returns $J u = (u_1, u_2, -u_3)$, so that `covector(u).dot(x)` is $\langle x, u \rangle$. `side(c, x)` is `c.dot(x)`. The covector costs one negation and is exact, and the side test costs one dot product. A domain stores the covector of each side beside its normal (Section 5.2 of the architecture), so the fold spends no negation per sample.

`side` applies no tolerance. The tolerance of the fold belongs to `escher-render` (Section 9.2 of the architecture).

### 4.3. Units

A length parameter is the full hyperbolic distance in the metric of curvature $-1$, and an angle is in radians (Section 5.5 of the architecture). Section 2.4 of the reference records that Lorentz matrices take the full distance, while $SL(2, \mathbb{R})$ and $SU(1,1)$ matrices take half of it. No function of this crate uses the half distance. In the module `euclidean`, a length is in the unit of the coordinates of the caller.

### 4.4. Sources and Their Conventions

Section 2.4 of the reference gives every formula in the convention of its source and converts none. This section concerns the module `hyperbolic`. Sections 5.4 and 6.6 give the source of each function of the module `euclidean`, and Sections 5.5 and 6.7 those of the module `spherical`. This crate uses a formula in one of three ways:

1. The source states it with the time coordinate last, as the project does. The crate evaluates it as printed. This covers the form, `ORIGIN`, the distance, the normalization (except the sign rule of Section 5.1, which is a choice), the boost, the rotation, the inverse, the isometry conditions, and both disk maps. The covector and the side test are definitions of Sections 2.1 and 5.2 of the architecture with $J$ of the reference, and they use no source formula.
2. The source states it with the form only, so the expression does not mention the position of the time coordinate. The crate evaluates it with the project's form. This covers the reflection (Section 6.5) and rests on an assumption (Section 13.1).
3. The source states it only with the time coordinate first, or not at all on the hyperboloid. The step rests on an unverified statement of Section 12 and a test of Section 10 covers the property that the code relies on. This covers the wall through two points (Section 5.3) and the translation (Section 6.4).

[RAT] states one transfer rule between its conventions, at the start of its Section 4.5: "All the results of Chapter 3 remain true after one reverses the order of the coordinates of $\mathbb{R}^{n+1}$" (Section 2.4 of the reference). The reference restates no single result of that chapter in the time-last form, so this design applies the rule to no formula.

### 4.5. The Euclidean Plane

The module `euclidean` represents the Euclidean plane as the affine plane $x_3 = 1$ of Section 5.1 of the architecture, over the same `glam` types:

- A point is the Euclidean point $(x, y, 1)$.
- A line is the line covector $c = (a_1, a_2, -t)$ of $P(a, t)$, with the unit normal $a$ of [RAT], Section 4.1, p. 105 (Section 12 of the reference). For a Euclidean point $x$ of $p$, $c \cdot x = a \cdot p - t$, and the line is oriented outward when that value is positive outside, as Section 5.2 of the architecture requires.
- A tangent vector is $(v_1, v_2, 0)$.
- An isometry is a Euclidean isometry, the block matrix $\begin{pmatrix} A & b \\ 0 & 1 \end{pmatrix}$ with $A$ orthogonal, which maps $(p, 1)$ to $(Ap + b, 1)$. [RAT], Theorem 1.3.5, p. 19, states that the isometries of the plane are the maps $p \mapsto b + Ap$ with $A$ orthogonal, and the block form rests on [W-AFFINE], a secondary source (Section 12 of the reference).

The representation is a choice of this design (Section 13.3). With it, the fold and the border test of `escher-render` run unchanged, because they use only dot products, matrix products, and the sign of the last coordinate.

GEO-20. Every Euclidean isometry that the module `euclidean` returns MUST have the last row $(0, 0, 1)$ exactly, because the fold of `escher-render` keeps the last coordinate of a Euclidean point at exactly 1 only when every pairing has that row, and a drifting last coordinate changes the side value of every later step.

For a finite point $(x, y, 1)$, a matrix with the last row $(0, 0, 1)$ gives the last coordinate $0 \cdot x + 0 \cdot y + 1 \cdot 1$, which is exactly 1 in IEEE 754 arithmetic.

### 4.6. The Sphere

The module `spherical` represents the sphere as the unit sphere $S^2$ with the dot product, as [KC24], p. 3, treats it, over the same `glam` types (Section 11.1 of the reference, Section 5.1 of the architecture):

- A point is a sphere point. `spherical::ORIGIN` is $C_0 = (0, 0, 1)$ of [KC24]. No coordinate has a sign condition, so a point of the lower hemisphere is as valid as one of the upper.
- A wall is a great circle, the intersection of $S^2$ with a plane through the origin ([RAT], p. 39), represented by a great-circle normal $u$. The side value of a point $x$ is $u \cdot x$, so the covector of $u$ is $u$ itself, and the outward orientation and the sign rule are those of Section 5.2 of the architecture.
- An isometry is a spherical isometry. The isometries of $S^2$ are exactly the orthogonal transformations of $\mathbb{R}^3$ ([RAT], Theorem 2.1.3, p. 38, and [KC24], p. 3).
- An angle is in radians. A length is the spherical distance of [RAT], equation (2.1.6), which is the angle between the two points in the metric of the unit sphere.

The representation is a choice of this design (Section 13.3). With it, the fold and the border test of `escher-render` use the same dot products and matrix products as in the other geometries.

## 5. Points and Walls

### 5.1. Normalization

`normalize_point` returns the point on the ray of a timelike vector with a positive last coordinate:

$$\operatorname{normalize}(x) = \frac{x}{\sqrt{-\langle x, x \rangle}}$$

Sources: `normalize` of [HR] divides by $\operatorname{sign} \cdot \sqrt{-\langle h, h \rangle}$, and `VectorNormalize` of [CS] uses $\sqrt{-\langle v, v \rangle}$ in the hyperbolic case (Section 8.4 of the reference, P-code). `mid` of [KC24], p. 3, divides a sum of points by the same quantity (Section 6.1 of the reference). [HR] and [KC24] put the time coordinate last.

The function computes $q = -\langle x, x \rangle$ with `form` and returns `NotAPoint` unless $q > 0$, $q$ is finite, and $x_3 > 0$. Each comparison is written so that NaN fails it, such as `q > 0.0` and not `!(q <= 0.0)`. An input whose $q$ underflows to 0 or overflows to infinity is rejected.

GEO-4. `normalize_point` MUST return `NotAPoint` for a vector that is spacelike, null, of non-positive last coordinate, infinite, NaN, or such that $q$ underflows or overflows, and MUST NOT flip the sign of a vector with a negative last coordinate, because such a vector is a symptom of an error upstream, and a silent flip would turn it into a valid point on the wrong side of the construction.

The sign factor of [HR] maps a vector with a negative last coordinate onto the hyperboloid. Rejecting it instead is a choice of this design (Section 13.2).

### 5.2. Distance

The reference states the distance on the hyperboloid as $a \ast b = -\cosh d(a, b)$ ([CFKP], Lemma, p. 83, P, Section 3.1 of the reference), with the time coordinate last. `distance` computes:

$$d(x, y) = \operatorname{arcosh}\left(\max\left(1, -\langle x, y \rangle\right)\right)$$

The clamp to at least 1 keeps the result a number when rounding makes $-\langle x, y \rangle$ fall just below 1 for two equal or close points. With $c = -\langle x, y \rangle$, the function clamps with `if c >= 1.0 || c.is_nan() { c } else { 1.0 }`, so a NaN $c$ reaches $\operatorname{arcosh}$ and the result is NaN, which keeps a NaN input visible to the caller. It does not use `f64::max`, which returns the other operand when one is NaN.

GEO-5. `distance` MUST return a number of at least 0 for two points, and NaN for a NaN input, because a NaN from rounding would poison every later comparison, while a NaN input must stay visible.

For two equal points, `distance` returns 0 only when the computed $-\langle x, x \rangle$ is at most 1, which holds for `ORIGIN`. For another point, the computed $-\langle x, x \rangle$ can exceed 1 by a rounding error that grows with $x_3^2$, and `distance(x, x)` is the positive $\operatorname{arcosh}$ of that value. Section 8.4 bounds it.

The inverse hyperbolic cosine is ill-conditioned near 1, and no source of the reference states its conditioning. Measurements in 256-bit arithmetic, at distances $d$ from $10^{-8}$ to 10 and absolute errors $\delta$ in the argument from $10^{-16}$ to $10^{-6}$, found the error of the result to be $\delta / \sinh d$ to within 1 percent where $\delta$ is small against $\sinh^2 d$, and at most $\sqrt{2\delta}$ everywhere.

The argument of `distance` is the form, whose rounding error grows with $x_3 y_3$, so `distance` loses accuracy for close points and for points far from `ORIGIN`. The reference names forms for close points: equation 2 of [KC24] in polar coordinates, which [KC24] recommends for points that are close together (Section 3.2 of the reference). Those forms need polar input, so this crate keeps the one form above. Section 8.4 states the accuracy that the crate guarantees, as the smaller of a bound that scales with $x_3 y_3 / \sinh d + d$ and one that scales with $\sqrt{\varepsilon \, x_3 y_3}$.

### 5.3. Wall Through Two Points

`wall_through(a, b)` returns a unit normal $u$ of the wall that contains $a$ and $b$, with these three properties:

$$\langle u, a \rangle = 0, \qquad \langle u, b \rangle = 0, \qquad \langle u, u \rangle = 1$$

The properties come from these statements of the reference:

- A geodesic of the hyperboloid is its intersection with a two-dimensional linear subspace ([CFKP], p. 80, P, Section 3.1 of the reference).
- The reflection of [DAVIS] (6.6) takes as normal "a unit spacelike vector orthogonal to H" (Section 7.4 of the reference).

The crate computes $w = J(a \times b)$, which is `covector(a.cross(b))`, and returns $u = w / \sqrt{\langle w, w \rangle}$. The formula is the time-last counterpart of the Lorentzian cross product $x \otimes y = J(x \times y)$ of [RAT], equation (3.2.4), p. 64, which is Lorentz orthogonal to both $x$ and $y$ (Section 7.4 of the reference). [RAT] prints it with the time coordinate first and $J = \operatorname{diag}(-1, 1, 1)$, and the reference states no time-last form (Section 7.4 of the reference). The time-last formula of this crate therefore rests on `UNV-26` (Section 12), and the reference and property tests of Section 10.1 check the three properties directly on the result.

The function returns `DegenerateWall` unless $\langle w, w \rangle > 0$ and $\langle w, w \rangle$ is finite. Two equal points give $w = 0$ and this error.

GEO-6. `wall_through` MUST return either a finite vector or `DegenerateWall`, and on the ranges of its rows of Section 8.4 the vector MUST have the three properties of this section within those rows, because a caller builds a pairing isometry from the result, and a vector that is not a unit normal gives a matrix that is not an isometry.

The crate states no orientation for $u$. Swapping the arguments negates $u$ exactly, which a property test checks. A caller orients the result with `side`, such as by negating $u$ when a point known to lie inside gives a positive value, as Section 5.2 of the architecture requires of outward normals.

### 5.4. Euclidean Points and Lines

`euclidean::side(c, x)` is `c.dot(x)`, the side value of Section 4.5. It applies no tolerance.

`euclidean::distance(p, q)` computes `(p.x - q.x).hypot(p.y - q.y)`, the distance $d_E(x, y) = \lvert x - y \rvert$ of [RAT], equation (1.3.4), p. 14 (Section 12 of the reference), and does not read the last coordinates. `hypot` neither overflows nor underflows in the squares, and it returns infinity when one difference is infinite.

`euclidean::wall_through(p, q)` returns a line covector $c$ of the line through $p$ and $q$, with these three properties:

$$c \cdot p = 0, \qquad c \cdot q = 0, \qquad c_1^2 + c_2^2 = 1$$

It computes $w = p \times q$, which is `p.cross(q)`, and $n = w_1^2 + w_2^2$, and returns $w / \sqrt{n}$ when $n$ is positive and finite and that result is finite, and `DegenerateWall` otherwise. Two equal points give $w = 0$ and this error, and so does a NaN or infinite coordinate. The squares are not scaled, so two distinct, finite points also give this error when $n$ underflows to 0, such as points about $10^{-170}$ apart, when $n$ overflows, such as points about $10^{200}$ apart, and when their coordinates are so large that the cross product overflows. The function returns the covector and not a unit normal, as `hyperbolic::wall_through` does, because the offset $t$ of the line is not a function of its normal.

No source states the formula (item 24 of Section 15.1 of the reference). As for `UNV-26` in Section 5.3, the formula only computes a candidate, and the tests of Section 10.1 check the three properties on the result. The function states no orientation. Swapping the arguments negates the result exactly, and a caller orients it with `euclidean::side`.

GEO-21. `euclidean::wall_through` MUST return either a finite vector or `DegenerateWall` for every input, and on the range of its rows of Section 8.5 the vector MUST have the three properties of this section within those rows, because a caller builds a pairing from the result with `euclidean::reflection`, and a covector whose direction part is not a unit vector gives a matrix that is not an isometry. Outside that range the function has no stated accuracy and may return `DegenerateWall` for two distinct points.

### 5.5. Spherical Points and Great Circles

`spherical::side(c, x)` is `c.dot(x)`, the side value of Section 4.6. It applies no tolerance.

`spherical::normalize_point(v)` computes $q = v \cdot v$ and returns $v / \sqrt{q}$ when $q > 0$ and $q$ is finite, and `NotAPoint` otherwise. Each comparison fails for NaN, as in Section 5.1, so a zero, NaN, or infinite vector, and a vector whose $q$ underflows to 0 or overflows to infinity, is rejected. The result is the point of the unit sphere of [KC24], p. 3, on the ray of $v$. The function puts no condition on the sign of a coordinate.

GEO-23. `spherical::normalize_point` MUST return `NotAPoint` for a vector whose computed $v \cdot v$ is not positive and finite, and MUST NOT put a condition on the sign of any coordinate, because a sphere point may lie in either hemisphere, while a vector of no finite length is a symptom of an error upstream.

`spherical::distance(x, y)` computes $c = x \cdot y$ and returns $\arccos$ of $c$ clamped to $[-1, 1]$, the distance $\arccos g^+(x, y)$ of [KC24], p. 3, which is $d_S(x, y)$ of [RAT], equation (2.1.6), p. 37 (Section 11.1 of the reference). The clamp is `c.clamp(-1.0, 1.0)`, which returns NaN for a NaN $c$, so a NaN input stays visible as in Section 5.2. The arccosine is ill-conditioned near 0 and $\pi$, so `distance` loses accuracy for close and for nearly antipodal points. Section 8.6 states the accuracy in a form that grows as $1 / \sin d$, and bounds `distance(x, x)`, which can be positive by rounding.

GEO-24. `spherical::distance` MUST return a number in $[0, \pi]$ when $x \cdot y$ is not NaN, and NaN otherwise, because a caller compares the result with $\pi$ and with tolerances, and a value outside that range from rounding would fail a correct comparison.

`spherical::wall_through(a, b)` returns a great-circle normal $u$ of the great circle that contains $a$ and $b$, with these three properties:

$$u \cdot a = 0, \qquad u \cdot b = 0, \qquad u \cdot u = 1$$

It computes $w = a \times b$, which is `a.cross(b)`, and $n = w \cdot w$, and returns $w / \sqrt{n}$ when $n$ is positive and finite and that result is finite, and `DegenerateWall` otherwise. The properties come from [RAT], Section 2.1 (Section 11.1 of the reference): the cross product of equation (2.1.2) is orthogonal to both factors by equation (2.1.3), its length is $\lvert a \rvert \lvert b \rvert \sin\theta(a, b)$ by equation (2.1.4), and two linearly independent points lie on a unique great circle, p. 39.

The function returns `DegenerateWall` in these cases:

- Two equal points, and two exactly antipodal points, for which every coordinate of the computed cross product is exactly 0. [RAT], p. 39, states that a continuum of great circles passes through two antipodal points, so none is the answer.
- A NaN or infinite coordinate in either point.
- A pair whose $n$ underflows to 0 or overflows.

For two points that are close or nearly antipodal, $\lvert w \rvert$ is about $\sin d$, so the result carries a relative error that grows as $\varepsilon / \sin d$. The rows of Section 8.6 state its accuracy above a separation floor from both 0 and $\pi$. The function states no orientation. Swapping the arguments negates the result exactly, and a caller orients it with `spherical::side`.

GEO-25. `spherical::wall_through` MUST return either a finite vector or `DegenerateWall` for every input, and on the range of its rows of Section 8.6 the vector MUST have the three properties of this section within those rows, because a caller builds a pairing from the result with `spherical::reflection`, and a vector that is not of unit length gives a matrix that is not orthogonal. Outside that range the function has no stated accuracy and may return `DegenerateWall` for two distinct points that are not antipodal.

## 6. Isometries

### 6.1. Composition and Inverse

A matrix $M$ that induces an isometry of the hyperboloid has the inverse $M^{-1} = J M^t J$ ([CFKP], Theorem 10.1, P, Section 6.1 of the reference), with the time coordinate last. `inverse` computes it as the transpose with the four entries that have exactly one index equal to 3 negated. Each entry is copied or negated, so the result has no rounding error relative to its input.

GEO-7. `inverse` MUST compute $J M^t J$ and MUST NOT compute a general matrix inverse, because the general inverse divides by a determinant and rounds every entry, while the reference names the transpose with sign changes as the inverse for better precision than row reduction (`MatrixGeometricInverse` of [CS], Section 8.4 of the reference).

For a matrix that is not an isometry, `inverse` returns $J M^t J$, which is not its inverse. The crate adds no `compose` function, because composition is the `glam` product (Section 4.1) and a second name for one operation is one more path to the same item.

### 6.2. Isometry Defect

[CFKP], Theorem 10.1, states that $M$ induces an isometry of the hyperboloid if and only if two conditions hold (Section 6.1 of the reference):

1. $m_i \ast m_j = e_i \ast e_j$ for the columns $m_i$ of $M$ and the standard basis vectors $e_i$.
2. The last entry of the last column is positive.

`isometry_defect` measures the first condition and checks the second:

$$\operatorname{defect}(M) = \max_{1 \le i \le j \le 3} \left\lvert \langle m_i, m_j \rangle - J_{ij} \right\rvert$$

It returns `f64::INFINITY` when the last entry of the last column is not positive, or when any difference in the maximum is NaN. It applies no tolerance. A caller compares the result with its own tolerance, such as the tolerances of the verification in `docs/design/domain.md`.

GEO-8. `isometry_defect` MUST return `f64::INFINITY` when condition 2 fails or the result is NaN, because a caller compares the result with a tolerance, and a NaN compares false with every tolerance and would otherwise pass a check written as "defect greater than tolerance fails".

### 6.3. Boost and Rotation

[KC24], p. 3, gives these maps with the time coordinate last (P, Section 6.1 of the reference):

$$T^\alpha(x) = (x_1 \cosh \alpha + x_3 \sinh \alpha, \ x_2, \ x_3 \cosh \alpha + x_1 \sinh \alpha)$$

$$R^\alpha(x) = (x_1 \cos \alpha - x_2 \sin \alpha, \ x_2 \cos \alpha + x_1 \sin \alpha, \ x_3)$$

$T^\alpha$ is a boost along $x_1$ by the full distance $\alpha$, and $R^\alpha$ is a rotation about $C_0$. With $\alpha$ the argument, `boost` returns the matrix with the columns $(\cosh\alpha, 0, \sinh\alpha)$, $(0, 1, 0)$, and $(\sinh\alpha, 0, \cosh\alpha)$. `rotation` computes `alpha.sin_cos()` once and returns the columns $(\cos\alpha, \sin\alpha, 0)$, $(-\sin\alpha, \cos\alpha, 0)$, and $(0, 0, 1)$, so it turns $e_1$ towards $e_2$, which is counterclockwise for a positive angle with $x_1$ to the right and $x_2$ up.

`boost` overflows to infinite entries for $\lvert \alpha \rvert$ above about 710.5, where `cosh` exceeds `f64::MAX`, and returns NaN entries for a NaN length. Neither panics.

### 6.4. Translation

`translation(t)` returns an isometry that maps `ORIGIN` to the point $p$ of the hyperboloid whose spatial coordinates are $(t_1, t_2)$:

$$\operatorname{translation}(t) = R^{\varphi} \, T^{\rho} \, R^{-\varphi}, \qquad \rho = \operatorname{arsinh} \sqrt{t_1^2 + t_2^2}, \qquad \varphi = \operatorname{atan2}(t_2, t_1)$$

The polar parameters are read from the linear representation of [KC24], which stores a point at distance $x$ and angle $\alpha$ from $C_0$ as $(\cos\alpha \sinh x, \sin\alpha \sinh x, \cosh x)$ (Section 8.3 of the reference). The crate computes `rho = t.x.hypot(t.y).asinh()` and `phi = t.y.atan2(t.x)`, and does not read $t_3$. `atan2(0, 0)` is 0 in Rust, so `translation(ORIGIN)` is the identity up to rounding, with no special case. For a target that is a point, the image of `ORIGIN` is the target up to a rounding error that grows with $\rho$, which Section 8.4 bounds.

The design relies only on two properties: the result is a product of the isometries of Section 6.3, and it maps `ORIGIN` to $p$. The reference states the composition $R(\varphi) T(\theta) R(-\varphi)$ as the translation in the direction $\varphi$ only in the $SU(1,1)$ form of [HT], equation 16, with the half distance (Section 7.2 of the reference), and states it on the hyperboloid nowhere. So the second property is an assumption (Section 13.1), and a reference test checks it, and the product itself on points off the axis, against $R^{\varphi} T^{\rho} R^{-\varphi}$ in the reference arithmetic. The design does not claim that the result is the translation along the geodesic from `ORIGIN` to $p$.

### 6.5. Reflection

[DAVIS], equation (6.6), gives the reflection in the hyperplane $H$ of the quadratic form model with unit normal $u$, "a unit spacelike vector orthogonal to H" (P, Section 7.4 of the reference):

$$r_H(x) = x - 2 \langle u, x \rangle u$$

Section 5.2 of the architecture adopts this form. The research does not record the position of the time coordinate in [DAVIS] (Section 7.4 of the reference). The expression uses the form only, so the crate evaluates it with the project's form, and Section 13.1 records that reading as an assumption. The matrix that [W-HYPERBOLOID] gives for the same reflection is time first and rests on a secondary source (Section 7.4 of the reference), so the crate does not use it.

`reflection(u)` returns the matrix whose column $j$ is $e_j - 2 c_j u$, where $c$ is `covector(u)`, because $\langle u, e_j \rangle = c_j$. The function does not check that $u$ is a unit normal. For a vector that is not one, the result is not a reflection.

### 6.6. Euclidean Isometries

The functions of this section return the Euclidean isometries of Section 4.5. Each is exact or has the accuracy of a row of Section 8.4 or 8.5:

| Function | Matrix, as its columns | Source |
| --- | --- | --- |
| `euclidean::reflection(c)` | Column $j$ is $e_j - 2 c_j u$ with $u = (c_1, c_2, 0)$, the matrix of $x \mapsto x - 2 (c \cdot x) u$ | The reflection $\rho(x) = x + 2(t - a \cdot x) a$ in $P(a, t)$ of [RAT], equation (4.1.1), p. 105, and equation 6.3 of [DAVIS] (Sections 7.4 and 12 of the reference). Its form as a matrix on the plane $x_3 = 1$ is part of `UNV-26` (Section 12) |
| `euclidean::rotation(angle)` | $(\cos\alpha, \sin\alpha, 0)$, $(-\sin\alpha, \cos\alpha, 0)$, and $(0, 0, 1)$, from one call of `sin_cos`, the expression of `hyperbolic::rotation` (Section 6.3) | [RAT], Theorem 1.3.5, with $b = 0$ and an orthogonal $A$, and the block form of [W-AFFINE] |
| `euclidean::translation(t)` | $(1, 0, 0)$, $(0, 1, 0)$, and $(t_1, t_2, 1)$. It does not read $t_3$ | [RAT], Theorem 1.3.5, with $A = I$, and the block form of [W-AFFINE] |

`euclidean::rotation` turns $e_1$ towards $e_2$, which is counterclockwise for a positive angle. `euclidean::translation` copies its target, so `euclidean::translation(t) * euclidean::ORIGIN` is $(t_1, t_2, 1)$ exactly. `euclidean::reflection` does not check that $c$ is a line covector. For a vector that is not one, the result is not a reflection, and for a finite $c$ its last row is still $(0, 0, 1)$ exactly (GEO-20). The function computes column $j$ as $e_j - (c_1 \cdot 2c_j, \ c_2 \cdot 2c_j, \ 0)$, with a literal 0 as the last entry and not the product $u_3 \cdot 2c_j$, because $2c_j$ overflows to infinity for a large finite $c_j$, and $0 \cdot \infty$ is NaN.

`euclidean::isometry_defect(m)` measures how far $m$ is from a Euclidean isometry. With $A$ the upper-left $2 \times 2$ block of $m$, it returns the largest of the absolute entries of $A^t A - I$ with $i \le j$, and of $\lvert m_{31} \rvert$, $\lvert m_{32} \rvert$, and $\lvert m_{33} - 1 \rvert$ for the last row. The orthogonality of $A$ is the condition of [RAT], Theorem 1.3.5, and the last row is the block form of Section 4.5. It returns `f64::INFINITY` for a matrix with an entry that is not finite, including the two entries of the translation part, $m_{13}$ and $m_{23}$, which the maximum does not read. The function applies no tolerance.

GEO-22. `euclidean::isometry_defect` MUST return `f64::INFINITY` when an entry of the matrix is not finite or the result is NaN, because a NaN compares false with every tolerance and would pass a check written as "defect greater than tolerance fails", as GEO-8 states for `hyperbolic::isometry_defect`, and a non-finite translation entry would otherwise pass a matrix that maps every point to a non-finite one.

### 6.7. Spherical Isometries

The functions of this section return spherical isometries (Section 4.6). Each is exact or has the accuracy of a row of Section 8.6:

| Function | Matrix, as its columns | Source |
| --- | --- | --- |
| `spherical::boost(angle)` | $(\cos\alpha, 0, -\sin\alpha)$, $(0, 1, 0)$, and $(\sin\alpha, 0, \cos\alpha)$, from one call of `sin_cos` | $T^\alpha$ of [KC24], p. 3, as printed (Section 11.1 of the reference) |
| `spherical::rotation(angle)` | The expression of `hyperbolic::rotation` (Section 6.3) | $R^\alpha$ of [KC24], p. 3, which the sphere shares with the hyperboloid |
| `spherical::translation(t)` | `rotation(phi) * boost(rho) * rotation(-phi)`, with $\rho = \operatorname{atan2}(\sqrt{t_1^2 + t_2^2}, t_3)$, the square root computed with `hypot`, or $\rho = 0$ when all three coordinates are zero, and $\varphi = \operatorname{atan2}(t_2, t_1)$, or $\varphi = 0$ when $t_1$ and $t_2$ are zero | No source. Item 27 of Section 15.1 of the reference, and Section 13.1 |
| `spherical::reflection(u)` | Column $j$ is $e_j - 2 u_j u$, the matrix of $x \mapsto x - 2 (u \cdot x) u$ | Equation (6.4) of [DAVIS], p. 75 (Section 7.4 of the reference) |
| `spherical::inverse(m)` | The transpose $m^t$ | The isometries are the orthogonal transformations ([RAT], Theorem 2.1.3, p. 38) |

`spherical::boost` is named after `hyperbolic::boost`, the map $T^\alpha$ of the same source, so a construction changes only the module path (Section 3.4). It moves `spherical::ORIGIN` along the great circle of the first axis, to $(\sin\alpha, 0, \cos\alpha)$. `spherical::rotation` runs the same operations as `hyperbolic::rotation` on the same input, so the two matrices are equal bit for bit.

`spherical::translation` reads all three coordinates of its target, unlike `hyperbolic::translation`, because the polar angle $\rho$ of a sphere point needs its last coordinate. The value of `atan2` at two zeros depends on their signs, and `atan2(+0, -0)` is $\pi$, so without the zero cases a zero vector with a last coordinate of $-0$ would give a half-turn. The zero cases of the row above hold whatever the signs of the zeros, so the translation of every zero vector is the identity exactly, and so is `spherical::translation(spherical::ORIGIN)`. The design relies only on two properties: the result is a product of the isometries above, and it maps `spherical::ORIGIN` to the sphere point on the ray of $t$. No source states the product on the sphere, so the second property is an assumption (Section 13.1), and a reference test checks it.

`spherical::reflection` does not check that $u$ is a great-circle normal. For a vector that is not one, the result is not a reflection.

`spherical::isometry_defect(m)` returns the largest of $\lvert m_i \cdot m_j - \delta_{ij} \rvert$ over the columns $m_i$ and $m_j$ with $i \le j$, the orthogonality of [RAT], Theorem 2.1.3. It accepts a determinant of $-1$, as a reflection has. It returns `f64::INFINITY` for a matrix with an entry that is not finite, or when the result is NaN. It applies no tolerance.

GEO-26. `spherical::isometry_defect` MUST return `f64::INFINITY` when an entry of the matrix is not finite or the result is NaN, because a NaN compares false with every tolerance and would pass a check written as "defect greater than tolerance fails", as GEO-8 and GEO-22 state for the other modules, and a non-finite entry would otherwise pass a matrix that maps every point to a non-finite one.

GEO-27. `spherical::inverse` MUST return the transpose and MUST NOT compute a general matrix inverse, because the transpose is exact relative to its input, while a general inverse divides by a determinant and rounds every entry, as GEO-7 states for `hyperbolic::inverse`.

## 7. Disk Map

### 7.1. Hyperboloid to Disk

The reference states the map from the hyperboloid to the Poincare disk with the time coordinate last (Section 4.2 of the reference). [KC24], p. 3, the composition $\beta \circ \delta$ of [CFKP], [RAT] equation (4.5.3), p. 127, and [DUN86], p. 147, give the same formula (P):

$$\operatorname{to\_disk}(x) = \left(\frac{x_1}{x_3 + 1}, \frac{x_2}{x_3 + 1}\right)$$

`to_disk` computes $s = x_3 + 1$ once and divides both spatial coordinates by it. On the hyperboloid $x_3 \ge 1$, so the denominator is at least 2 and the sum does not cancel. Off the hyperboloid, the function is total: $x_3 = -1$ gives infinite or NaN coordinates, and `to_disk((inf, 0, inf))` is NaN.

### 7.2. Disk to Hyperboloid

[RAT], equation (4.5.2), p. 127, and [DUN86], p. 147, give the inverse map with the time coordinate last (P, Section 4.2 of the reference):

$$\operatorname{from\_disk}(z) = \left(\frac{2 z_1}{1 - \lvert z \rvert^2}, \frac{2 z_2}{1 - \lvert z \rvert^2}, \frac{1 + \lvert z \rvert^2}{1 - \lvert z \rvert^2}\right)$$

`from_disk` computes `s = z.length_squared()` and `d = 1.0 - s`, and returns `None` unless `d > 0.0`. It then divides $2 z_1$, $2 z_2$, and $1 + s$ by $d$.

GEO-9. `from_disk` MUST return `None` exactly when the computed $1 - \lvert z \rvert^2$ is not positive, including for NaN, because a position on or outside the unit circle is outside the model, which a renderer treats as background (step 5.2 of Section 4.1 of the architecture), and a point computed from it would be infinite or on the wrong sheet.

### 7.3. Boundary and Precision

These cases show the behaviour at the edge of each domain:

| Input | Result |
| --- | --- |
| `from_disk((0, 0))` | `Some(ORIGIN)` |
| `from_disk((1, 0))`, on the unit circle | `None` |
| `from_disk((2, 0))`, outside the unit circle | `None` |
| `from_disk((NaN, 0))` | `None` |
| `from_disk((1 - 2^-53, 0))`, the largest `f64` below 1 | `Some`, with a last coordinate of about $9.0 \times 10^{15}$, finite |
| `to_disk(ORIGIN)` | `(0, 0)` |
| `to_disk` of a point at distance 37 from `ORIGIN` | Radius 0.9999999999999998 to within $4\varepsilon$ |
| `to_disk` of a point at distance 38.2 from `ORIGIN` | Radius 1.0 to within $4\varepsilon$ |

The smallest positive value of $d = 1 - s$ is about $1.1 \times 10^{-16}$, so no result of `from_disk` overflows. Near the limit circle, `from_disk` loses accuracy: the computed $d$ carries an absolute error of about $\varepsilon$, so each coordinate of the result carries a relative error of about $\varepsilon / (1 - \lvert z \rvert^2)$, which the row `from_disk` of Section 8.4 bounds. A unit test and a reference test assert the two radii of `to_disk` in the table within $4\varepsilon$. A point at distance $r$ from the centre lies at $1 - \Theta(e^{-r})$ in the disk ([KC24], Section 8.3 of the reference), so for a large distance the radius that `to_disk` returns differs from 1 by rounding alone. The test asserts this at distance 38.2 only. These limits are part of `UNV-4` (Section 9.4 of the architecture), and the renderer measures their effect on an image.

## 8. Numerical Robustness

### 8.1. Input Outside a Domain

Every function is total (GEO-2). Only the functions `normalize_point` and `wall_through` of each module and `from_disk` check their input, because each returns a value that a caller stores and builds on. The other functions return an unspecified value for input outside their domain, because a check in each would cost time in the fold and the caller already holds a value of the right kind:

| Function | Input outside the domain | Behaviour |
| --- | --- | --- |
| `normalize_point` | Spacelike, null, non-positive last coordinate, NaN, infinite, or $q$ underflowing or overflowing | `NotAPoint` |
| `wall_through` | Equal points, or a pair whose $w$ has a non-positive or non-finite form | `DegenerateWall` |
| `from_disk` | On or outside the unit circle, NaN | `None` |
| `distance` | A vector off the hyperboloid | A number from the clamped formula, or NaN for NaN input |
| `to_disk` | A vector off the hyperboloid | The formula, possibly infinite or NaN |
| `reflection` | A normal that is not a unit normal | A matrix that is not a reflection |
| `inverse` | A matrix that is not an isometry | $J M^t J$ |
| `translation` | A target off the hyperboloid | The isometry of its spatial coordinates |
| `boost`, `rotation`, `translation` | A non-finite parameter or target | Infinite or NaN entries |
| `euclidean::wall_through` | Equal points, a pair whose $w_1^2 + w_2^2$ is not positive and finite, or a pair whose result is not finite, which includes every NaN or infinite coordinate | `DegenerateWall` |
| `euclidean::side`, `euclidean::distance` | A vector that is not a Euclidean point, or a covector that is not a line covector | The formula, possibly infinite or NaN |
| `euclidean::reflection` | A covector that is not a line covector | A matrix that is not a reflection, with the last row $(0, 0, 1)$ for a finite covector |
| `euclidean::rotation`, `euclidean::translation` | A non-finite angle or target | Infinite or NaN entries |
| `spherical::normalize_point` | A vector whose $v \cdot v$ is not positive and finite, which includes the zero vector and every NaN or infinite coordinate | `NotAPoint` |
| `spherical::wall_through` | Equal or exactly antipodal points, a pair whose $n$ is not positive and finite, or a pair whose result is not finite | `DegenerateWall` |
| `spherical::side`, `spherical::distance` | A vector that is not a sphere point | The formula, possibly NaN. `distance` stays in $[0, \pi]$ unless $x \cdot y$ is NaN |
| `spherical::reflection` | A normal that is not a sphere point | A matrix that is not a reflection |
| `spherical::inverse` | A matrix that is not a spherical isometry | Its transpose |
| `spherical::boost`, `spherical::rotation`, `spherical::translation` | A non-finite angle or target | Infinite or NaN entries |

### 8.2. Forms That Avoid Cancellation

The crate chooses these forms:

- `inverse` is $J M^t J$, exact relative to its input (GEO-7).
- `translation` reads the polar radius as $\operatorname{arsinh}$ of the spatial length, from the linear representation of [KC24] (Section 8.3 of the reference), and not as $\operatorname{arcosh} t_3$, which is ill-conditioned near $t_3 = 1$ and would lose about half the digits of a target near `ORIGIN`.
- `to_disk` adds 1 to $x_3 \ge 1$, which does not cancel.
- `from_disk` cancels in $1 - \lvert z \rvert^2$ near the limit circle. No reformulation avoids it, because the input carries the lost digits. Section 7.3 states the loss.
- `distance` keeps the ill-conditioned $\operatorname{arcosh}$ near 1, with the clamp of Section 5.2, because its well-conditioned alternative needs polar input.
- `spherical::distance` keeps the ill-conditioned $\arccos$ near $\pm 1$, with the clamp of Section 5.5, for the same reason. `spherical::translation` reads its polar angle with `atan2` of `hypot` and $t_3$, not as $\arccos t_3$, which is ill-conditioned near $t_3 = \pm 1$.

GEO-10. `translation` MUST compute the polar radius as $\operatorname{arsinh}$ of $\sqrt{t_1^2 + t_2^2}$ with `hypot` and MUST NOT read $t_3$, because the inverse hyperbolic cosine of a last coordinate near 1 loses about half its digits, and `hypot` neither overflows nor underflows in the square.

### 8.3. Normalization Policy

GEO-11. Every function other than the two functions `normalize_point` and the three functions `wall_through` MUST NOT divide its result by a norm, because where and how often a point or matrix is renormalized is the caller's policy, such as the fold, which renormalizes nothing (Section 9.3 of the architecture), and a hidden renormalization moves a point that the caller relies on, such as a vertex of a domain.

`hyperbolic::wall_through` divides by $\sqrt{\langle w, w \rangle}$ because its contract is a unit normal, and `euclidean::wall_through` divides by $\sqrt{w_1^2 + w_2^2}$ because its contract is a line covector. `spherical::normalize_point` and `spherical::wall_through` divide by a Euclidean length because their contracts are a sphere point and a great-circle normal. The crate has no function that renormalizes a matrix, such as the Gram-Schmidt process of `fixmatrix` in [HR] (Section 8.4 of the reference). A later change adds one when a caller needs it.

### 8.4. Accuracy

GEO-12. Each function MUST meet the accuracy of this table over the stated range of inputs, because `docs/design/domain.md` sets its own tolerances on top of these, and a function that is less accurate than stated makes a correct domain fail its verification.

The tests of Section 10.1 assert each bound against the reference arithmetic of Section 10.2 over the range of its row, with 10,000 or 100,000 random cases from a fixed seed and the edge cases that the row names, and print the largest error with its input and its margin. Each test also asserts that it measured at least a stated number of cases, so a test that measures nothing fails. On Linux, where the `f64` methods of Rust call the transcendental functions of the C library, every bound holds with a measured margin of at least 8, as TEST-13 of the coding standard requires. Each bound uses the scales of Section 2.1:

| Function | Range of inputs | Accuracy |
| --- | --- | --- |
| `form` | Points within distance 20 of `ORIGIN`, each multiplied by a factor in $[0.5, 2]$, in pairs. Also every ordered pair of the vectors of the point table of Section 10.1 | Absolute error at most $16\varepsilon \sum_i \lvert x_i y_i \rvert$ |
| `side`, triangles | The covectors $c$ of the walls of the triangle set of Section 10.1, with each vertex and the centre of the triangle as $x$ | Absolute error at most $8\varepsilon \sum_i \lvert c_i x_i \rvert$ |
| `side`, general | The covectors $c$ of walls at distance up to 10 from `ORIGIN`, and points $x$ within distance 10 of `ORIGIN` | Absolute error at most $16\varepsilon \sum_i \lvert c_i x_i \rvert$ |
| `normalize_point` | A point within distance 15 of `ORIGIN`, rounded to `f64`, both unscaled and multiplied by a factor in $[0.5, 2]$. Also the points of the point table within distance 15, each multiplied by 0.5, 1, and 2 | Each coordinate within $16\varepsilon \kappa \, \hat{x}_3$ of the reference value $\hat{x}$ of $x / \sqrt{-\langle x, x \rangle}$ |
| `distance` | $x$ within distance 10 of `ORIGIN`, and $y$ at a distance from $10^{-8}$ to 20 from $x$, log-uniform, each rounded to `f64`. Also the pairs of different entries of the point table within distance 10 | Within $\min(64\varepsilon (x_3 y_3 / \sinh d + d), \ 16\sqrt{\varepsilon \, x_3 y_3})$ of the reference value $d$ of $\operatorname{arcosh}(\max(1, -\langle x, y \rangle))$. Within $16\sqrt{\varepsilon \, x_3 y_3}$ when $d = 0$ |
| `distance(x, x)` | Points within distance 20 of `ORIGIN`, and the points of the point table | At least 0 and at most $32\sqrt{\varepsilon} \, x_3$ |
| `wall_through`, orthogonality, close points | $a$ within distance 10 of `ORIGIN`, and $b$ at a distance from $10^{-5}$ to 5 from $a$, log-uniform | $\lvert \langle u, a \rangle \rvert$ and $\lvert \langle u, b \rangle \rvert$ each at most $256\varepsilon K \max(a_3, b_3)$, as reference values from the exact `f64` values of $u$, $a$, and $b$. At most 136 of 10,000 results are `DegenerateWall` |
| `wall_through`, orthogonality, near `ORIGIN` | $a$ and $b$ within distance 3 of `ORIGIN` and at least 0.1 apart | The quantity of the row above, at most $32\varepsilon K \max(a_3, b_3)$ |
| `wall_through`, norm | The ranges of the two rows above | $\lvert \langle u, u \rangle - 1 \rvert$ at most $64\varepsilon K$, as a reference value |
| `boost` | $\lvert l \rvert \le 700$, half uniform and half log-uniform in $\lvert l \rvert$ from $10^{-300}$ with either sign. Also the lengths 0, $\pm 700$, $\pm 10^{-300}$, $5 \times 10^{-324}$, and $\pm 1$ | Each entry within a relative error of $16\varepsilon$ of the reference value of $\cosh l$ or $\sinh l$. The other five entries are exactly 0 or 1 |
| `rotation` | Angles of absolute value up to 1000. Also the angles 0, $\pm 1000$, $\pi$, and $10^{-300}$. The function accepts any finite angle, and this row states its accuracy only on this range, because the reference arithmetic of Section 10.2 works at 256 bits and reducing a larger angle needs more digits of $\pi$ than that | Each entry within $10^{-14}$ of its reference value. The other five entries are exactly 0 or 1 |
| `translation` | Targets $t$ with $r$, the reference value of $\rho$ of Section 6.4, from 0.1 to 709, half uniform and half log-uniform, and the targets $(0, 0, 1)$, $(10^{-300}, 0, 1)$, and $(5 \times 10^{-324}, 0, 1)$ | Each coordinate of `translation(t) * ORIGIN` within $8\varepsilon (1 + r) \, p_3$ of the point $p$ of Section 6.4. For the three listed targets, each coordinate is also within one rounding unit, $\varepsilon \lvert p_i \rvert$, of $p$, which makes the second coordinate exactly 0 |
| `translation`, off the axis | Targets $t$ with $r$ from 0.1 to 700, half uniform and half log-uniform. Points $x$ within distance 3 of `ORIGIN`, and in one case of four a point at the angle $\pi/2$ and a distance from 0.5 to 3 | Each coordinate of `translation(t) * x` within $16\varepsilon (1 + r) \cosh r \, x_3$ of the reference value of $R^{\varphi} T^{\rho} R^{-\varphi} x$ |
| `translation`, isometry | Targets $t$ with $r$ from 0.1 to 350, half uniform and half log-uniform | `isometry_defect(translation(t))` at most $512\varepsilon g^2$, the bound of the row `isometry_defect`, longer boosts |
| `translation`, inverse | Targets $t$ at a distance $r$ from `ORIGIN` from 0.1 to 350, half uniform and half log-uniform | Each coordinate of `inverse(translation(t)) * t` within $16\varepsilon (1 + r) \, t_3^2$ of `ORIGIN` |
| `reflection` | Unit normals $u$ of walls at distance up to 10 from `ORIGIN`. Points $x$ within distance 3 of `ORIGIN`, and points at an offset of $10^{-12}$ to 1 of either sign along $u$ from a point of the wall at a parameter from $-3$ to 3 along the wall | Each coordinate of `reflection(u) * x` within $16\varepsilon \sigma_i$ of the reference value of $x - 2\langle u, x \rangle u$ |
| `reflection`, normal | The unit normals of the row above | Each coordinate of `reflection(u) * u` within $32\varepsilon \lvert u_i \rvert (1 + 2\sum_j u_j^2)$ of $-u_i$ |
| `inverse` | Any matrix | Exact relative to the input, and `inverse(inverse(m))` is bit for bit $m$ |
| `inverse`, product | Products $m$ of one to three factors, each a boost of length at most 2, a rotation, or the reflection in the wall through two points within distance 3 of `ORIGIN` | Each entry of `inverse(m) * m` within $10^{-11} \mu^2$ of the identity, with $\mu$ of $m$ |
| `isometry_defect` | The products of the row above | At most $10^{-11} \mu^2$ |
| `isometry_defect`, longer boosts | The products of the row above, with boosts of length at most 5 | At most $512\varepsilon g^2$ |
| `to_disk` | Points within distance 38 of `ORIGIN`. Also `ORIGIN` and a point at distance 38 | Each coordinate within $8\varepsilon$ times the absolute value of its reference value |
| `from_disk` | Positions $z$ with $1 - \lvert z \rvert$ from $10^{-12}$ to 1, log-uniform. Also the positions $(0, 0)$, $(1 - 10^{-12}, 0)$, and $(1 - 10^{-9}, 0)$ | Each coordinate within a relative error of $8\varepsilon / (1 - \lvert z \rvert^2)$ |
| `from_disk` then `to_disk` | The positions of the row above | Each coordinate within $8\varepsilon$ absolute |
| `to_disk` then `from_disk` | Points within distance 12 of `ORIGIN` | Each coordinate within $32\varepsilon \, x_3^2$ absolute, which is a relative error of $32\varepsilon \, x_3$ |

In the rows of `translation`, $r$ is the value of $\rho$ of Section 6.4 for the target. The reference value of $R^{\varphi} T^{\rho} R^{-\varphi} x$ takes $\cosh \rho = \sqrt{1 + t_1^2 + t_2^2}$, $\sinh \rho = \sqrt{t_1^2 + t_2^2}$, $\cos \varphi = t_1 / \sinh \rho$, and $\sin \varphi = t_2 / \sinh \rho$, with $\cos \varphi = 1$ and $\sin \varphi = 0$ when $\sinh \rho = 0$. In the row `translation`, isometry, $g$ of Section 2.1 is $\cosh r$, which the test computes in `f64` as $\sqrt{1 + t_1^2 + t_2^2}$.

The table guarantees nothing outside its ranges. The renderer calls `from_disk`, and applies the pairings that `reflection` builds, outside the ranges of their rows, for points up to the rim of the disk, and states its own end-to-end bounds in Section 6.4 of `docs/design/render.md`. The tests of Section 10 compute $K$ as Section 2.1 defines it, from the distance between the exact `f64` values of the points, and skip a pair whose reference distance is 0.

`normalize_point` has no stated accuracy beyond distance 15 from `ORIGIN`. There, the function may return `NotAPoint` for the vector of a point, as GEO-4 requires of a vector whose computed $q$ is not positive. The test of that range, at distances from 15 to 20, asserts only that each rejected vector fails the conditions of Section 5.1 on its computed $q$ and its last coordinate, and that each accepted result is finite with a positive last coordinate.

The first form of the bound of `distance` carries the term $d$, because without it the measured margin on the range of the row fell below 8 for points about 10 from `ORIGIN`.

`wall_through` has no stated accuracy for two points less than $10^{-5}$ apart. For such points it returns a finite vector or `DegenerateWall`, and the tests below that separation assert only that. No bound is stated there, because over separations down to $10^{-6}$ the largest orthogonality error grew with each new sample. GEO-6 allows a result of `DegenerateWall`, and the rows of `wall_through` bound every other result on their ranges.

The bound of `isometry_defect` and of `inverse(m) * m` in $\mu^2$ holds only for the products of its row, and the bound in $g^2$ covers the products with longer boosts. The table states no bound for a product of more than three factors.

`docs/design/domain.md` calls `side`, `distance`, `wall_through`, `translation`, and `reflection` on the vertices of its triangles and on their centre. Section 9.1 of `docs/design/domain.md` states that those vertices leave the hyperboloid by a small amount, which is largest near the order limit. The reference tests build the triangle set of Section 10.1 with the functions of this crate, by the steps of Section 6.1 of `docs/design/domain.md`, and run each of the five functions on it. Some of those inputs lie outside the ranges that the rows state, and every result is within the bound of its row: `side` within the row `side`, triangles, `wall_through` within the rows near `ORIGIN`, and `distance`, `translation`, and `reflection` within their rows. The tests do not measure `normalize_point` on the sums of vertices of that construction, or `isometry_defect` on its pairings.

### 8.5. Accuracy of the Module `euclidean`

The tests measure each row of this table as Section 8.4 states, and GEO-12 applies to each row as it applies to Section 8.4. A point "within 64" is a Euclidean point at a distance of at most 64 from `euclidean::ORIGIN`, and each quantity is a reference value from the exact `f64` values of the inputs and the result. In the row of orthogonality, $\lvert p \rvert$ and $\lvert q \rvert$ are the lengths of the first two coordinates of the points:

| Function | Range of inputs | Accuracy |
| --- | --- | --- |
| `euclidean::reflection` | Line covectors $(\cos\theta, \sin\theta, -t)$ at 64 angles $\theta$ and offsets with $\lvert t \rvert \le 64$, and points within 64 | Each coordinate of `euclidean::reflection(c) * x` within $16\varepsilon \sigma_i$ of $\rho$ of [RAT] (4.1.1), with $\sigma_i$ of Section 2.1 and $u = (c_1, c_2, 0)$ |
| `euclidean::reflection`, isometry | The covectors of the row above | `euclidean::isometry_defect(euclidean::reflection(c))` at most $32\varepsilon$ |
| `euclidean::wall_through`, orthogonality | Pairs of points within 64, from 0.1 to 128 apart, log-uniform | $\lvert c \cdot p \rvert$ and $\lvert c \cdot q \rvert$ each at most $8\varepsilon (1 + \lvert p \rvert + \lvert q \rvert + \lvert p \rvert \lvert q \rvert / \lvert p - q \rvert)$ |
| `euclidean::wall_through`, norm | The pairs of the row above | $\lvert c_1^2 + c_2^2 - 1 \rvert$ at most $32\varepsilon$ |
| `euclidean::distance` | The pairs of the row above | A relative error of at most $16\varepsilon$ against $\lvert p - q \rvert$ |

`euclidean::rotation` runs the same operations as `hyperbolic::rotation` on the same input, so the row `rotation` of Section 8.4 states its accuracy, and a unit test asserts that the two matrices are equal bit for bit. `euclidean::translation` is exact (Section 6.6).

### 8.6. Accuracy of the Module `spherical`

The tests measure each row of this table as Section 8.4 states, and GEO-12 applies to each row as it applies to Section 8.4. A random sphere point is a vector drawn uniformly on the sphere and rounded to `f64`. Each reference value is computed from the exact `f64` values of the inputs and the result. $d$ is the reference value of $\arccos(x \cdot y)$ for the exact inputs, and $\sigma_i$ is that of Section 2.1 with the covector $c = u$:

| Function | Range of inputs | Accuracy |
| --- | --- | --- |
| `spherical::side` | Random great-circle normals $u$ and random sphere points $x$, and the normals, vertices, and centres of the spherical domains of `docs/design/domain.md` | Absolute error at most $16\varepsilon \sum_i \lvert u_i x_i \rvert$ |
| `spherical::normalize_point` | Random sphere points multiplied by a factor in $[0.5, 2]$, and sums of two and of three random sphere points that are at least $10^{-3}$ from the zero vector | Each coordinate within $16\varepsilon$ of the reference value of $v / \sqrt{v \cdot v}$ |
| `spherical::distance` | A random sphere point $x$, and $y$ at a distance from $10^{-3}$ to $\pi - 10^{-3}$ from $x$, log-uniform toward each end, rounded to `f64` | Within $16\varepsilon / \sin d$ of $d$ |
| `spherical::distance(x, x)` | Random sphere points, and `spherical::ORIGIN` | At least 0 and at most $32\sqrt{\varepsilon}$ |
| `spherical::wall_through`, orthogonality | The pairs of the row `spherical::distance` | $\lvert u \cdot a \rvert$ and $\lvert u \cdot b \rvert$ each at most $8\varepsilon / \sin d$ |
| `spherical::wall_through`, norm | The pairs of the row `spherical::distance` | $\lvert u \cdot u - 1 \rvert$ at most $32\varepsilon$ |
| `spherical::boost` | The angles of the row `rotation` of Section 8.4 | Each entry within $10^{-14}$ of the reference value of $\cos\alpha$ or $\pm\sin\alpha$. The other five entries are exactly 0 or 1 |
| `spherical::translation` | Random sphere points $t$, and $t$ equal to `spherical::ORIGIN`, to its negation, and to three points within $10^{-300}$ of each | Each coordinate of `translation(t) * ORIGIN` within $16\varepsilon$ of the reference value of $t / \lvert t \rvert$ |
| `spherical::translation`, off the axis | The targets of the row above, each paired with one random sphere point $x$ | Each coordinate of `translation(t) * x` within $32\varepsilon$ of the reference value of $R^{\varphi} T^{\rho} R^{-\varphi} x$ |
| `spherical::translation`, isometry | The targets of the row `spherical::translation` | `spherical::isometry_defect(translation(t))` at most $32\varepsilon$ |
| `spherical::reflection` | Random great-circle normals $u$. Random sphere points $x$, and points at an offset of $10^{-12}$ to 1 of either sign along $u$ from a point of the great circle | Each coordinate of `reflection(u) * x` within $16\varepsilon \sigma_i$ of the reference value of $x - 2 (u \cdot x) u$ |
| `spherical::reflection`, isometry | The normals of the row above | `spherical::isometry_defect(reflection(u))` at most $64\varepsilon$ |
| `spherical::isometry_defect` | Products of one to three factors, each with probability 1/3 a boost, a rotation, or a reflection. A boost and a rotation have an angle uniform in $[-10, 10]$, and a reflection is in the great circle through two random sphere points at a distance from $10^{-3}$ to $\pi - 10^{-3}$ | At most $128\varepsilon$ |
| `spherical::inverse` | Any matrix | Exact relative to the input, and `inverse(inverse(m))` is bit for bit $m$ |

The pairs of the row `spherical::distance` are drawn in two halves. In one half $d$ is log-uniform in $[10^{-3}, \pi/2]$, and in the other the distance is $\pi - d$ with $d$ drawn the same way. The second point is $y = x \cos d + t \sin d$ for a random unit tangent $t$ at $x$. A point at an offset $o$ along $u$ from a point $p$ of the great circle is $p \cos o + u \sin o$.

The reference value of $R^{\varphi} T^{\rho} R^{-\varphi} x$ takes $\cos\rho = t_3 / \lvert t \rvert$, $\sin\rho = \sqrt{t_1^2 + t_2^2} / \lvert t \rvert$, $\cos\varphi = t_1 / \sqrt{t_1^2 + t_2^2}$, and $\sin\varphi = t_2 / \sqrt{t_1^2 + t_2^2}$, with $\cos\varphi = 1$ and $\sin\varphi = 0$ when $t_1 = t_2 = 0$.

`spherical::rotation` runs the same operations as `hyperbolic::rotation`, so the row `rotation` of Section 8.4 states its accuracy, and a unit test asserts that the two matrices are equal bit for bit.

The table guarantees nothing outside its ranges. `spherical::wall_through` has no stated accuracy for two points less than $10^{-3}$ apart or less than $10^{-3}$ from antipodal. For such points it returns a finite vector or `DegenerateWall`, and the tests below that floor assert only that.

## 9. Errors

### 9.1. Structure

GEO-13. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"normalize point"` for `NotAPoint` and `"find wall through two points"` for `DegenerateWall`.
- `detail`: A private enum with one variant for each cause, which carries no data: a vector that is not a point of the hyperboloid, a vector with no finite, positive length, and two points that span no wall. The first two have the kind `NotAPoint`.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm, so the kind is not stored twice (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives `Display` and `std::error::Error` on `Error` and on the detail enum, in the pattern of `escher-signature` (Section 10.1 of `docs/design/signature.md`). No error of this crate has a cause, so `source()` returns `None` and the struct has no source field. No `glam` type and no value of the input appears in the error (ERR-2 of the coding standard).

`from_disk` returns an `Option` and not a `Result`, because a position outside the disk is a normal outcome of every image, not a failure.

### 9.2. Kinds and Text

GEO-14. `Display` for `Error` MUST be `cannot `, the operation, `: `, and the text of the detail in this table, one row for each cause, because `escher-domain` carries the error as the `source()` of its own, and `escher-cli` prints that chain.

| Kind | Returned by | `Display` text |
| --- | --- | --- |
| `NotAPoint` | `hyperbolic::normalize_point` | `cannot normalize point: the vector is not timelike with a positive last coordinate` |
| `NotAPoint` | `spherical::normalize_point` | `cannot normalize point: the vector has no finite, positive length` |
| `DegenerateWall` | `hyperbolic::wall_through`, `euclidean::wall_through`, and `spherical::wall_through` | `cannot find wall through two points: the points do not span a wall` |

Each text is in lower case and has no trailing punctuation (ERR-4 of the coding standard).

## 10. Testing

### 10.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-geometry/tests/geometry/main.rs`, with the modules `common`, `reference`, `formulas`, and `properties` (TEST-10). Its root has a `//!` line, because `missing_docs` covers test roots. `reference` holds the high-precision implementation of Section 10.2. `formulas` holds the tests that compare the crate with it. `properties` holds the property tests and the deterministic sweeps of 100,000 cases.

`common` holds the shared helpers and the named input sets:

- The random ranges of the rows of Sections 8.4 to 8.6. A reference test or sweep takes its random inputs from a SplitMix64 generator with a fixed seed of its own, so every run draws the same cases. A helper that redraws an input until it meets a condition, such as two points at least 0.1 apart, stops after 1,000 attempts with a panic, so a defect that rejects every input fails the test instead of hanging it.
- The point table: the points at distances 0, 0.1, 1, 5, 10, and 20 from `ORIGIN`, each at eight angles, and as vectors those points multiplied by 0.5, 1, and 2.
- The triangle set: the triangles of the ten triples $(7, 3, 2)$, $(100, 3, 2)$, $(100, 100, 100)$, $(98, 95, 88)$, $(100, 89, 87)$, $(100, 98, 95)$, $(99, 98, 87)$, $(99, 4, 4)$, $(93, 4, 3)$, and $(100, 7, 6)$, built with the functions of this crate by the steps of Section 6.1 of `docs/design/domain.md`.

The tests form these layers:

| Layer | What it checks | What it catches |
| --- | --- | --- |
| Reference arithmetic | Each row of Sections 8.4 to 8.6 against the reference arithmetic of Section 10.2, on the inputs of the row and within its bound, and the five functions of Section 8.4 on the triangle set. The maps of [KC24] and the disk maps against their formulas in the reference type, and the three properties of each wall evaluated in the reference type from the exact `f64` values of the result and the points | A wrong formula, a wrong convention, and a lost digit |
| Property tests and sweeps | Deterministic sweeps of 100,000 cases, and `proptest` on 256 cases, of relations that hold in exact arithmetic: a reflection is an involution, fixes the points of its wall, and has a small `isometry_defect`, a product of isometries preserves `form` and `distance`, `distance(ORIGIN, boost(l) * ORIGIN)` is $\lvert l \rvert$, `distance` agrees with the disk formula of [RAT], Theorem 4.5.1, both disk round trips return their input, the product of the reflections in two walls at the angle $\pi/n$ raised to the power $n$ is the identity, and swapping the arguments of each `wall_through` negates its result exactly. Below the separation floors of Sections 8.4 and 8.6, and beyond distance 15 for `normalize_point`, the tests assert only a finite result or the error | A formula that holds on the reference inputs but breaks a law elsewhere in its range |
| Exact cases | Unit tests of the values that IEEE 754 arithmetic gives exactly: dyadic reflections, copies, negations, the identity from `spherical::translation` of `spherical::ORIGIN` and of every zero vector, the transpose, `rotation` of each module equal bit for bit to `hyperbolic::rotation`, and the last row of `euclidean::reflection`. Further unit tests assert `isometry_defect` within $10^{-12}$ of its value on six matrices that each break one of its equations | A wrong sign, a wrong constant, and a special case that rounds |
| Failure paths and any input | `NotAPoint` and `DegenerateWall` for every input that Sections 5.1, 5.3 to 5.5, and 8.1 name, including underflow and overflow, the results of Section 8.1 for non-finite input, the rows of the table of Section 7.3, and the error texts of Section 9.2. A property test passes any `f64` bit pattern, NaN, infinity, signed zero, or subnormal value to every function and asserts no panic and the invariants of GEO-2, GEO-20, GEO-21, and GEO-23 to GEO-26 | A missing check, a panic, and a changed text |
| Other tests | A compile-time assertion that `Error` and `ErrorKind` are `Send`, `Sync`, and `'static`, the doctests, and a self-check of the reference arithmetic: $\pi$ against a literal with 80 digits after the decimal point, $\cos(\pi/3) = 1/2$, $\cosh(\operatorname{arcosh} 2) = 2$, and $\sqrt{2}^2 = 2$, each to $10^{-70}$ | A wrong auto trait, a stale example, and a wrong precision or rounding argument of the reference |

Each reference test and sweep that asserts a bound prints its largest error, in the form of the bound, with its margin and the input that gave it. The test harness captures that output for a test that passes. The tests of `docs/design/domain.md` check its triangles themselves.

A unit test asserts that `euclidean::isometry_defect` exceeds $10^{-9}$, the tolerance of check 1 of `docs/design/domain.md`, for a reflection scaled by $1 + 10^{-6}$ and for a reflection whose last row is set to $(0, 10^{-6}, 1)$.

The property tests commit their regression files (TEST-9 of the coding standard). They run with the default configuration of `proptest`.

The self-check of the reference arithmetic does not test correct rounding, which Section 12 treats under `UNV-25`.

GEO-17. Withdrawn on 2026-11-19.

### 10.2. Reference Arithmetic

The reference arithmetic uses the crate `astro-float-num`, the number crate of the astro-float project (Section 11).

GEO-15. The module `reference` MUST implement each formula componentwise from the reference, in the high-precision type, and MUST call no function of this crate and none of `glam`, because the reference arithmetic is an independent implementation (Section 7.1 of the architecture), and shared code would share a transcription error with the code under test.

These rules apply to the module:

- Precision: 256 bits, with the rounding mode `RoundingMode::ToEven`. The largest amplification of an error in the tests of this crate is that of $\operatorname{arcosh}$ near 1 for points $10^{-8}$ apart, about $10^{8}$, and of $1 / (1 - \lvert z \rvert^2)$ at radius $1 - 10^{-12}$, about $5 \times 10^{11}$, which leaves more than 60 correct digits.
- Points are arrays of three numbers of the high-precision type.
- Comparison: `astro-float-num` has no public conversion from its number type `BigFloat` to `f64`, while `BigFloat::from_f64` is public and `BigFloat` implements `PartialEq` and `PartialOrd`. A test therefore converts each `f64` result of the crate into the high-precision type, which is exact, computes the difference there, and compares it with the tolerance with `PartialOrd`. A test that prints a measured error formats the number with `format`.
- Constants and functions: `Consts` provides $\pi$, and the number type provides `add`, `sub`, `mul`, `div`, `sqrt`, `cos`, `sin`, `cosh`, `sinh`, `acosh`, `asinh`, and `acos`, each with a precision and a rounding mode. The tests of the module `spherical` use `acos` for the reference distance. `BigFloat::parse` reads the decimal literals of the self-check. The module converts each `f64` input with `BigFloat::from_f64`, which is exact, and imports nothing from this crate or from `glam`.

### 10.3. Tolerances

A test of a quantity that a row of Sections 8.4 to 8.6 bounds takes the tolerance and the range of inputs of that row, and the property tests assert the same tolerances on 256 cases. The sweeps and property tests of relations that no row bounds, such as preserved quantities, involutions, points of a wall, products of reflections, and the disk formula, hold their own tolerances over named input sets. Each such tolerance is at least 8 times the largest error measured, and each test prints its largest error with its input and its margin. These tolerances bound the tests' own relations and are not part of a contract of the crate, so GEO-16 does not apply to them and this document does not record them (GEO-19).

The tolerance for a point of a wall carries a term in $K \max(a_3, b_3)$, because the points lie on the wall only to within the error of `wall_through`, and without that term the measured margin fell below 8 for walls through close points.

A test compares bit for bit only where TEST-13 of the coding standard allows it, because each case is exact in IEEE 754 arithmetic: the exact cases of Section 10.1, which are dyadic sums, copies, negations, and the same operations on the same inputs, and the swap of the arguments of each `wall_through`. The cross product of the swapped points is the exact negation of the cross product of the points, and the covector, the form, and the square root that follow keep that relation.

GEO-19. A test that asserts a tolerance on a quantity MUST name its input set and print its largest error with its input and its margin, as TEST-13 of the coding standard requires. This document MUST record a tolerance only where it is part of a contract of the crate, because the test repeats the measurement on every run, while a figure copied into this document goes stale with the next change to the code.

GEO-16. Every test that compares a floating-point result MUST use the tolerance that this document states for its quantity as part of a contract of the crate, and MUST compare a result bit for bit only where TEST-13 of the coding standard allows it, because a test that compares within a looser tolerance passes a result that breaks the contract.

TEST-13 of the coding standard states the general rule that GEO-15, GEO-16, and GEO-19 apply to this crate.

## 11. Dependencies

GEO-18. The crate MUST declare `glam` and `thiserror` as its only normal dependencies and MUST NOT re-export `glam`, because each further dependency adds build time, licence obligations, and supply-chain risk (DEP-2 of the coding standard), and a re-export would be a second path to the `glam` types that every crate declares itself (Section 3.1 of the architecture, API-10).

The crate declares these dependencies, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml` (DEP-2):

| Dependency | Kind | Version and features | Why |
| --- | --- | --- | --- |
| `glam` | Normal, public | `0.34.3`, a caret requirement, `default-features = false`, with `std` and `float-types` | The types `DVec2`, `DVec3`, and `DMat3` (`DEC-4`). The default features also turn on the integer and size types, which nothing uses |
| `thiserror` | Normal | As the workspace, default features | Derives `Display` and `std::error::Error`, in the pattern of `escher-signature` |
| `proptest` | Dev | As the workspace | The property tests (Section 7.1 of the architecture) |
| `astro-float-num` | Dev | `0.3.7`, a caret requirement, `default-features = false`, with `std` only | The reference arithmetic of `DEC-3` (Section 10.2). Its default features are `std`, `random`, and `serde`, and this crate turns off `random` and `serde` |

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

`glam` types appear in the public signatures of this crate, so `glam` is a public dependency under DEP-3 of the coding standard, with the consequences that Section 3.5 of the architecture states for the version of this crate. No `glam` type appears in `Error` or `ErrorKind` (ERR-2).

`DEC-3` names the crate `astro-float`. This design uses `astro-float-num`, the number crate of the same project, without the macro crate. `astro-float` 0.9.6 depends on `astro-float-macro`, which pulls `syn` 1, and the dependency policy rejects that as a duplicate beside the `syn` 3 of the workspace (TOOL-26 of `docs/design/tooling.md`), while `astro-float-num` with the feature `std` resolves with no duplicate. Section 13.3 raises the change with the owner.

With the features of the table, `glam` has no transitive dependency, and it computes `dot`, `cross`, and the product of a matrix and a vector with plain multiplications and additions, with no fused multiply-add. `astro-float-num` without `random` and `serde` adds no `rand` 0.8 beside the `rand` of the workspace. `astro-float` 0.9.6 fails the ban on duplicates with `syn` 1 even without its default features. The licences of both crates and of their dependencies are on the allow-list of Section 8 of `docs/design/tooling.md`, so `deny.toml` needs no new licence and no exception.

The root `Cargo.toml` builds `astro-float-num` with `opt-level = 3` in the `dev` profile, in the table `[profile.dev.package.astro-float-num]`, because the high-precision arithmetic of the reference tests is slow without optimization. The setting also shortens `just coverage`. Section 4 of `docs/design/tooling.md` states the other levels of the profile.

## 12. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, or a candidate statement of the reference without a label there, the sections of this document that rely on it, and what the code does about it:

| Label | What the code relies on | Relied on in | What the code does |
| --- | --- | --- | --- |
| `UNV-26` | The normal through two points with the time coordinate last, $w = J(a \times b)$ with $J = \operatorname{diag}(1, 1, -1)$, where [RAT] states the Lorentzian cross product with the time coordinate first (Section 7.4 of the reference). Also the Euclidean reflection of [RAT] (4.1.1) as the matrix of `euclidean::reflection` on the plane $x_3 = 1$ | 5.3, 6.6 | Uses the formula of the normal only to compute a candidate, checks that its form is positive and finite, and normalizes it. The tests check the three properties of Section 5.3 on the result, which are what callers rely on. They compare the Euclidean reflection applied to points with (4.1.1) in the reference arithmetic, and check that it is an involution that fixes the points of its line. The second part of `UNV-26`, the relation between normals and the corner angle, is not used by this crate |
| None | The line through two points as a normalized cross product, item 24 of Section 15.1 of the reference | 5.4 | Uses the formula only to compute a candidate. The tests check the three properties of Section 5.4 on the result |
| `UNV-25` | Correct rounding of the reference-arithmetic crate, in tests only | 10.2 | Works at 256 bits against tolerances of a few multiples of $\varepsilon$ at the tightest, so a wrong last bit of the reference cannot change a verdict. The self-check of Section 10.1 checks the precision and rounding arguments. A comparison of six values with the MPFR library 4.2.2 at 200 bits, called through its C interface, agreed exactly to more than 60 significant digits, and item 2 of Section 13.3 asks the owner whether it closes `UNV-25` |
| `UNV-4` | The precision of the disk map near the limit circle | 7.3, 8.4 | States the loss of `from_disk` and the distance beyond which the radius of `to_disk` is within rounding of 1 (Section 7.3), and tests both against the reference arithmetic at radius up to $1 - 10^{-12}$. The effect on an image is measured by the renderer |

`UNV-28` does not apply. No test of this crate uses SymmHub's triangle as an oracle.

## 13. Assumptions, Choices, and Points for the Owner

### 13.1. Assumptions

These readings of this design rest on no sentence of a source. Each names the test that checks it and what would settle it:

- The reflection of [DAVIS] (6.6) holds with the time-last form of this project. The research does not record the position of the time coordinate in [DAVIS], and the expression uses the form only. A reference test checks that the result maps $u$ to $-u$, and property tests that it is an involution that fixes the points of its wall and that a product of reflections preserves the form. A reading of [DAVIS] that records its convention would settle it.
- `translation(t)` maps `ORIGIN` to the point with the spatial coordinates of $t$. The reference states the product $R(\varphi) T(\theta) R(-\varphi)$ only in the $SU(1,1)$ form of [HT] (Section 7.2 of the reference). A reference test checks the property. A source that states the composition on the hyperboloid would settle it.
- `spherical::translation(t)` maps `spherical::ORIGIN` to the sphere point on the ray of $t$. [KC24] states the factors and not their product on the sphere (item 27 of Section 15.1 of the reference). A reference test checks the property and the product. A source that states the composition on the sphere, or a Lean proof, would settle it.

### 13.2. Choices of This Design

No other document dictates these choices. Each looks like something a change could simplify away, and its section gives the full reason:

| Choice and reason | Section |
| --- | --- |
| No type of the crate's own and no wrapper around the `glam` types, because Section 3.2 and `DEC-4` of the architecture fix the types of points and isometries | 3.2 |
| One module per geometry, with flat functions `to_<model>` and `from_<model>`, no trait, and no `compose` function, because no caller is generic over the geometry and composition is the `glam` product | 3.4, 6.1 |
| `from_disk` returns an `Option`, because a position outside the disk is a normal outcome of every image | 7.2, 9.1 |
| `normalize_point` rejects a vector with a non-positive last coordinate and does not flip it, because such a vector is a symptom of an error upstream | 5.1 |
| `distance` has one form, because the forms for close points need polar input, clamped to at least 1 so that rounding cannot make the result NaN, and keeps a NaN input visible | 5.2 |
| `wall_through` states no orientation, because the caller orients the result with `side` from a point that it knows to lie inside | 5.3 |
| `translation` reads no last coordinate, and the design relies only on its product and its image of `ORIGIN`, because no source states the product on the hyperboloid | 6.4 |
| `isometry_defect` returns a number, with infinity for a failed condition, because each caller has its own tolerance | 6.2 |
| No function except `normalize_point` and `wall_through` renormalizes, because renormalization is the caller's policy | 8.3 |
| Every accuracy bound of Sections 8.4 to 8.6 with its range of inputs, each constant set from the largest error measured on the code with a margin of at least 8, as TEST-13 of the coding standard requires | 8.4 to 8.6 |
| No stated accuracy of `wall_through` for points less than $10^{-5}$ apart, where the measured error grew with each sample, or of `normalize_point` beyond distance 15, where it may reject the vector of a point | 8.4 |
| The term $d$ in the first form of the bound of `distance`, and the term in $K$ in the tolerance for a point of a wall, because without them the margin falls below 8 | 8.4, 10.3 |
| An exact assertion for the swap of the arguments of `wall_through`, because IEEE 754 arithmetic keeps the negation exactly | 10.3 |
| Inputs from a SplitMix64 generator with a fixed seed per test, so every run draws the same cases | 10.1 |
| `astro-float-num` instead of `astro-float`, because `astro-float` brings a duplicate `syn` | 10.2, 11 |
| `glam` without its default features, because nothing uses the integer and size types | 11 |
| `opt-level = 3` for `astro-float-num` in the `dev` profile, because the reference tests are slow without it | 11 |
| The representations of the Euclidean plane and of the sphere, the items of the modules `euclidean` and `spherical`, the formula of `euclidean::wall_through`, and the name `boost` on the sphere, which Section 13.3 puts to the owner | 3.4, 4.5, 4.6, 5.4, 6.7 |

### 13.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. `astro-float-num` instead of `astro-float`. `DEC-3` names `astro-float`, whose macro crate pulls `syn` 1. The design depends on `astro-float-num`, the number crate of the same project without the macro, which resolves with no duplicate (Section 11). The alternative is `astro-float` with a `skip` entry for `syn` 1 in `deny.toml`, which TOOL-26 of `docs/design/tooling.md` allows with a reason. The design takes `astro-float-num`, because it needs no exception to the dependency policy.
2. The comparison for `UNV-25`. It used the MPFR library of the container, called through its C interface from a script outside the repository (Section 12). The design treats the six agreeing values as the one comparison that the plan asks for. If the owner requires the comparison through another tool, such as `rug`, `UNV-25` stays open and the tests are unchanged.
3. A gradient through the disk map. The renderer draws borders from a distance estimate that needs the derivative of the disk map (`DEC-28`), while the plan adds Jacobians to this crate only with the further model maps of Section 5.12 of the plan. This design adds none. The design of the renderer holds the derivative in `escher-render`, which Section 5.2 of `docs/design/render.md` specifies, until a later milestone moves it to this crate with the other Jacobians.
4. The separation floor of `wall_through`. The crate states no accuracy for two points less than $10^{-5}$ apart (Section 8.4), and Section 1.5 excludes a distance form for close points. The design keeps the floor and the one formula of Section 5.3. The alternative is a form for close points in a later change, if a caller needs walls through such points.
5. The `dev` profile of `astro-float-num`. The root `Cargo.toml` builds that dev-dependency with `opt-level = 3` (Section 11). The design keeps it, because the reference tests are slow without it. The alternative is to build it unoptimized.
6. The representation of the Euclidean plane. The design uses the affine plane $x_3 = 1$ over the `glam` types of the hyperbolic module (Section 4.5), so that the fold and the border test of `escher-render` run unchanged, which a prototype outside the repository confirmed on the Euclidean triangles and the rectangle. The alternative is a `DVec2` point with a `DMat2` and `DVec2` pair for an isometry, which needs a second fold, a second side type, and a second border test.
7. The items of the module `euclidean`. The design exports the constant and the seven functions of Section 3.1, with no `form`, `inverse`, or `normalize_point`, because no caller needs them (Section 3.4). The alternative is every name of `hyperbolic` that has a Euclidean counterpart. A later change adds `inverse` when a pairing is not an involution.
8. The line through two points. No source states the cross-product formula of `euclidean::wall_through` (Section 5.4). The design uses it to compute a candidate and tests the three properties on the result, as it does for `UNV-26` in Section 5.3. The alternative is to wait for a source or a Lean proof, which would block the Euclidean domains of `docs/design/domain.md`.
9. The representation of the sphere. The design uses unit vectors with the dot product, great-circle normals that are their own covectors, and orthogonal matrices (Section 4.6), so that the fold and the border test of `escher-render` keep their dot products and matrix products. The alternative is unit quaternions for the isometries (Section 11.1 of the reference), which needs a second fold.
10. The items of the module `spherical`. The design exports the constant and the ten functions of Section 3.1, with no `form`, no `covector`, no model map, and no rotation about a general axis, because no caller needs them (Section 3.4). The alternative is every name of `hyperbolic` that has a spherical counterpart, or a public rotation about any axis for the view of `escher-render`.
11. The name `boost` on the sphere. The design names $T^\alpha$ of [KC24] `boost` in both modules, so that the spherical construction of `docs/design/domain.md` changes only the module path (Section 6.7). The alternative is a name without the hyperbolic sense, such as `shift`.

## 14. Security Considerations

The threat model is a caller that passes values derived from untrusted input to this crate, such as coordinates computed from a signature with large orders, or a disk position computed from a pixel of an image whose size the user chose. The attacker can choose any `f64` bit pattern in any argument. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are a panic that stops the caller, and a non-finite value that propagates into a stored result. The crate counters them with these measures:

- No function panics for any input (GEO-2), and a property test checks every function with arbitrary bit patterns (Section 10.1).
- The six functions whose result a caller stores and builds on, the two functions `normalize_point`, the three functions `wall_through`, and `from_disk`, reject every input outside their domain, including NaN and infinities (GEO-4, GEO-6, GEO-9, GEO-21, GEO-23, GEO-25).
- Every function runs in constant time and allocates nothing, so no input exhausts memory or time. The crate therefore needs no limit of its own. The limits on the orders of a signature, which bound the values that reach this crate, are stated in the Security Considerations of `docs/design/domain.md`.

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library. `glam` with the features of Section 11 uses no SIMD intrinsics in its double-precision types.
- Untrusted input and file paths: the arguments are the only input. The crate reads and writes no file and keeps no state between calls.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate builds and prints nothing.

The residual risk is a wrong result without an error. A total function given input outside its domain returns an unspecified value (Section 8.1), and a caller that skips the checks of `normalize_point` or `isometry_defect` builds on it. A second residual risk is that a formula that rests on `UNV-26` or on an assumption of Section 13.1 is wrong in a way that the property tests do not reveal.

## 15. Operational Considerations

The crate declares no Cargo feature, reads no environment variable, and has no build script. It is pure Rust with no platform-specific code, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log.

Every failure is an `ErrorKind`, a `None`, or a non-finite value:

| Failure | Reported as | Recovery |
| --- | --- | --- |
| A vector that is not timelike with a positive last coordinate, or for `spherical::normalize_point` a vector with no finite, positive length | `NotAPoint` | Check the computation that produced the vector. The error indicates a defect upstream |
| Two points that span no wall, such as two equal points, or two antipodal points on the sphere | `DegenerateWall` | Pass two distinct points, which on the sphere are also not antipodal |
| A disk position on or outside the unit circle, or NaN | `None` from `from_disk` | Treat the position as outside the model, such as background |
| A matrix that is not an isometry | A large or infinite `isometry_defect` | Rebuild the matrix from its factors |
| A point far beyond a distance of 38 from `ORIGIN` | `to_disk` returns a position within rounding of the unit circle, which `from_disk` can reject | None. The disk cannot represent the point in `f64` (Section 7.3) |
| A boost length above about 710.5 | Infinite entries | None. The hyperboloid cannot represent the point in `f64` |

## 16. References

### 16.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 16.2. Informative References

The tags of the mathematical sources and source code resolve to the reference sections of `docs/research/geometry-models.md`:

- [CFKP] Section 16.1 of `docs/research/geometry-models.md`.
- [CS] Section 16.1 of `docs/research/geometry-models.md`.
- [DAVIS] Section 16.1 of `docs/research/geometry-models.md`.
- [DUN86] Section 16.1 of `docs/research/geometry-models.md`.
- [HR] Section 16.1 of `docs/research/geometry-models.md`.
- [HT] Section 16.1 of `docs/research/geometry-models.md`.
- [KC24] Section 16.1 of `docs/research/geometry-models.md`.
- [RAT] Section 16.1 of `docs/research/geometry-models.md`.
- [W-AFFINE] Section 16.3 of `docs/research/geometry-models.md`.
- [W-HYPERBOLOID] Section 16.3 of `docs/research/geometry-models.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| GEO-1 | The crate exports exactly the items of Section 3.1, each through one path, with no vector or matrix type of its own | 3.1 |
| GEO-2 | No function panics for any input | 3.2 |
| GEO-3 | Every formula is evaluated with the time coordinate last, the project's form, and outward normals | 4.1 |
| GEO-4 | `normalize_point` rejects every vector that is not timelike with a positive last coordinate, or whose $q$ is not finite and positive, and flips no sign | 5.1 |
| GEO-5 | `distance` returns a number of at least 0 for two points, and NaN for a NaN input | 5.2 |
| GEO-6 | `wall_through` returns a finite vector or `DegenerateWall`, and on the ranges of Section 8.4 a unit normal orthogonal to both points | 5.3 |
| GEO-7 | `inverse` computes $J M^t J$ and never a general inverse | 6.1 |
| GEO-8 | `isometry_defect` returns infinity when the second condition fails or the result is NaN | 6.2 |
| GEO-9 | `from_disk` returns `None` exactly when the computed $1 - \lvert z \rvert^2$ is not positive | 7.2 |
| GEO-10 | `translation` computes the polar radius with `hypot` and $\operatorname{arsinh}$, and reads no last coordinate | 8.2 |
| GEO-11 | Only the two functions `normalize_point` and the three functions `wall_through` divide by a norm | 8.3 |
| GEO-12 | Each function meets the accuracy of Sections 8.4, 8.5, and 8.6 | 8.4 |
| GEO-13 | `Error` follows ERR-1, with the operation that failed | 9.1 |
| GEO-14 | The `Display` text of each kind is the one stated | 9.2 |
| GEO-15 | The reference arithmetic implements each formula independently, without this crate or `glam` | 10.2 |
| GEO-16 | A comparison of a quantity with a tolerance stated as part of the contract uses that tolerance, and a bit-for-bit comparison is made only where TEST-13 allows it | 10.3 |
| GEO-17 | Withdrawn on 2026-11-19. | 10.1 |
| GEO-18 | `glam` and `thiserror` are the only normal dependencies, and `glam` is not re-exported | 11 |
| GEO-19 | A test of a tolerance names its input set and prints its largest error, and this document records the tolerance only where it is part of a contract | 10.3 |
| GEO-20 | Every Euclidean isometry of the module `euclidean` has the last row $(0, 0, 1)$ exactly | 4.5 |
| GEO-21 | `euclidean::wall_through` returns a finite vector or `DegenerateWall` for every input, and on the range of Section 8.5 a line covector through both points | 5.4 |
| GEO-22 | `euclidean::isometry_defect` returns infinity when an entry is not finite or the result is NaN | 6.6 |
| GEO-23 | `spherical::normalize_point` rejects every vector whose $v \cdot v$ is not positive and finite, and puts no condition on the sign of a coordinate | 5.5 |
| GEO-24 | `spherical::distance` returns a number in $[0, \pi]$, and NaN only for a NaN dot product | 5.5 |
| GEO-25 | `spherical::wall_through` returns a finite vector or `DegenerateWall` for every input, and on the range of Section 8.6 a great-circle normal through both points | 5.5 |
| GEO-26 | `spherical::isometry_defect` returns infinity when an entry is not finite or the result is NaN | 6.7 |
| GEO-27 | `spherical::inverse` returns the transpose and never a general inverse | 6.7 |
