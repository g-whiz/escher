# Fundamental Domains and Side-Pairing Isometries from an Orbifold Signature

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-18 |
| Applies to | The construction of a fundamental polygon, its side-pairing isometries, and point reduction from an orbifold signature in Escher |

## Abstract

Escher renders a planar symmetry group named by its orbifold signature. To do that it turns the signature and any free shape parameters into a fundamental polygon and a set of side-pairing isometries, and then maps every point of the plane into that polygon. This document collects what the literature and one existing implementation state about each step. It covers Poincare's polygon theorem with its hypotheses, the presentations read from a signature, the dimension and coordinates of the deformation space, and explicit matrices for the orientable building blocks. It also covers the cut-and-weld construction that handles mirrors and cross-caps, the triangle, Euclidean, and spherical cases, and the algorithms for point reduction, Dirichlet domains, and discreteness checks. Every claim carries its source and a status. The document marks the parts of the construction that have no primary source, so that an implementer can tell established constructions from those that rest only on code or on unproven candidates.

## 1. Scope

This document covers the construction that starts from an orbifold signature with its free parameters and ends with a fundamental polygon, its side-pairing isometries, and a procedure that reduces a point into the polygon. It covers hyperbolic, Euclidean, and spherical geometry. It states results, formulas, and algorithms with citations and proves nothing.

It leaves these topics to other documents:

- `docs/research/orbifold-notation.md`: the grammar of signatures, defects and the Euler characteristic, the classification by geometry, the bad orbifolds, the dictionary between NEC signatures and Conway signatures, which it records as unverified, and the presentation read from a signature. This document repeats only what the construction uses.
- `docs/research/geometry-models.md`: the models of the hyperbolic plane, the formulas of isometries in each model, conversions between models, and hyperbolic trigonometry.
- `docs/research/rendering.md`: the per-pixel folding loop on the GPU.
- `docs/research/symmhub.md`: the SymmHub source code at line level. This document describes the SymmHub pipeline only at the level an implementer needs to compare it with the literature.
- `docs/research/source-requests.md`: the sources that could not be read in full, with what the project needs from each.

Frieze groups, whose signatures contain infinity, are out of scope, except for cusps as [MASKIT2001] treats them. Delaney-Dress symbols are covered at survey level only, in Section 11.

## 2. Notation and Conventions

### 2.1. Status Markers

Every cited claim carries one of these status markers after its citation:

- {P}: verified against a primary source that was read in full, or against the cited pages of a primary source.
- {N}: verified against a primary source, [CONWAY-HUSON2002], [THURSTON2002], [TEGULA2020], or [FIELD-GUIDE2018], by the research behind `docs/research/orbifold-notation.md`, and not reread for this document.
- {C}: read line by line in the SymmHub source code at commit `f2d19cb`. It establishes what the code does. It does not establish that the method is correct.
- {S}: rests only on a secondary source, or on a search result that relays a source that was not read.

A statement that the research assembled from several cited items, such as the checklist in Section 4.8, says so and carries the markers of its parts. A result that a numeric port of the SymmHub code found, such as those of `docs/research/symmhub.md` Section 6.9, says that it is numeric evidence from a port. It is neither a source nor a proof, and a claim that rests on it stays in Section 13.

### 2.2. Transcription

Names with accented letters are written in ASCII with the accents dropped: Poincare, Teichmuller, Mobius, Jorgensen, del Rio, and the French words in titles. Quotations are verbatim, with three exceptions. Mathematical symbols are transcribed into LaTeX. The glyphs for orbifold features are written as the ASCII characters `o`, `*`, and `x`. Where a source puts a semicolon inside a formula, this document either writes a comma and says so, or puts the whole expression in a code span, because the repository's prose checker rejects semicolons.

Voight writes a semicolon between the two arguments of the distance function $\rho$ and of the reduction $\mathrm{red}_G$. This document writes $\rho(\gamma, z)$ and $\mathrm{red}_G(\gamma, z)$ instead. Signature tuples that contain semicolons, such as `(g; m_1, ..., m_t; s)`, appear in code spans.

### 2.3. Symbols

| Symbol | Meaning |
| --- | --- |
| $\mathbb H$ | The upper half-plane model of the hyperbolic plane |
| $\mathfrak D$ | The unit disc model of the hyperbolic plane, as in [VOIGHT2009] |
| $X^n$ | The sphere $S^n$, Euclidean space $E^n$, or hyperbolic space $H^n$, as in [DAVIS2008] |
| $\Gamma$, $G$ | A discrete group of isometries |
| $D$, $R$, $P$ | A polygon, usually a candidate fundamental domain |
| $\gamma_s$, $g_s$ | The side-pairing transformation of side $s$ |
| $m$, $m_j$, $\alpha$, $\alpha_k$ | The order of a cone point, a corner, or a vertex cycle. Section 5.1 reuses $\alpha_i$, $\beta_i$, $\gamma_i$ for the generators of Voight's presentation, and Section 5.2 uses Greek letters for the generators of Conway and Huson. |
| $\chi(O)$ | The orbifold Euler characteristic of the orbifold $O$ |
| $\chi(X_O)$ | The Euler characteristic of the underlying surface $X_O$ of $O$ |
| $k$, $l$ | In [THURSTON2002] Corollary 13.3.7: the number of cone points and the number of corners |
| $g$ | Genus |
| $\lambda_i$ | A size in the sense of [MASKIT2001], defined in Section 2.4 |
| $\sigma$, $\tau$ | Maskit's size of a coordinate geodesic and half its twist |
| $s_i$, $t_i$ | Maskit's coordinates $s_i = \sinh\sigma_i$ and $t_i = \sinh\tau_i$ |
| $\mu$, $\nu$ | Auxiliary quantities of Maskit's pants matrices, defined by equation (1) and equation (2) |
| $p$, $q$ | In Section 7 only: Maskit's numbers of coordinate geodesics and of pants |
| $\tilde a$ | A matrix in $SL(2,\mathbb R)$, or in $S^{\pm}L(2,\mathbb R)$ for a reflection, that represents the map $a$ |
| $\mathrm{tr}$ | The trace of a matrix |
| $\rho(\gamma, z)$ | $d(\gamma z, 0)$ in $\mathfrak D$, as in [VOIGHT2009] Section 4 |
| $I(g)$ | The isometric circle of $g$ |
| $\mathrm{ext}(G)$ | The intersection of the exteriors of the isometric circles of the elements of $G$ |
| $\infty_\ell$ | A boundary token of length $\ell$ produced by a geodesic cut, in the SymmHub tutorial |
| $Q$, $Z$ | In [EPSTEIN-PETRONIO1994]: the quotient of the disjoint union of the cells by the face pairing, and the space that Section 4 of the paper builds from the cells, the face pairing, and the group of its Definition 4.2 |
| $(R, A)$ | In [EPSTEIN-PETRONIO1994]: a face pairing, with $R$ the involution on the faces and $A$ the isometry of each face |

### 2.4. Terms

- orbifold signature: A name of a planar symmetry group in Conway's orbifold notation, written with `o` for a handle, `*` for the start of a mirror boundary, `x` for a cross-cap, and digits for cone points and corners. `docs/research/orbifold-notation.md` defines the grammar.
- cone point: A point of the orbifold whose stabilizer is a cyclic rotation group, written as a digit before any `*`.
- corner: A point on a mirror boundary where two mirrors meet, written as a digit after a `*`.
- cusp: A cone point of infinite order, whose cycle transformation is parabolic.
- Fuchsian group: The term that [SOLOMYAK2020] and [SERIES2013] use for a discrete group of orientation-preserving isometries of the hyperbolic plane.
- NEC group: A non-Euclidean crystallographic group, a discrete group of isometries of the hyperbolic plane that may contain orientation-reversing elements. [CONWAY-HUSON2002] p. 247 calls the orbifold symbol "an elegant form of Macbeath's 'signature'" {N}. [SINGERMAN1974] p. 233 and [MACBEATH1967] pp. 1192 to 1193 {P} require in addition that the quotient is compact, and Singerman calls an NEC group that contains orientation-reversing elements a "proper NEC group". This document uses the term without the compactness requirement, except where it cites those two papers.
- subproper side pairing: In [RATCLIFFE] p. 695 {P}: a side pairing $\Phi$ such that "each cycle of $\Phi$ is finite, each dihedral ridge cycle of $\Phi$ has dihedral angle sum a submultiple of $\pi$, and each cyclic ridge cycle has dihedral angle sum a submultiple of $2\pi$." A submultiple of $\beta$ is $\beta/k$ for a positive integer $k$, or $\beta/\infty = 0$ (p. 265).
- dihedral ridge cycle: In [RATCLIFFE] p. 695 {P}: a ridge cycle $\{x_1, \dots, x_m\}$ such that $x_1$ lies on a side whose pairing is the reflection in that side, and $x_m$ lies on such a side. In the plane a ridge is a vertex.
- cyclic ridge cycle: In [RATCLIFFE] p. 695 {P}: a ridge cycle that is not dihedral.
- fundamental domain: In [VOIGHT2009] Section 1 {P}: "a closed domain D such that: (i) $\Gamma D = \mathbb H$, and (ii) $gD^\circ \cap D^\circ = \emptyset$ for all $g \in \Gamma \setminus \{1\}$." In [DAVIS2008] Definition 5.1.3, printed p. 64 {P}: "A closed subset C in Y is a fundamental domain for G on Y if each G-orbit intersects C and if for each point x in the interior of C, $Gx \cap C = \{x\}$."
- strict fundamental domain: In [DAVIS2008] Definition 5.1.3 {P}: a fundamental domain that "intersects each G-orbit in exactly one point."
- side: In [SERIES2013] Definition 6.1, p. 85 {P}: "a segment $s \subset \partial R$ of positive length such that $s \subset \bar R \cap g(\bar R)$ for some $g \in \Gamma$." A side need not be a maximal geodesic segment, so "an edge may be a union of several sides."
- side pairing: In [SERIES2013] Definition 6.1 {P}: "an element $g \in \Gamma$ such that $\bar R \cap g(\bar R)$ is a side." In [VOIGHT2009] Section 1 {P}: a set $P = \{(g, s, s^*) : s^* = g(s)\}$ that partitions the sides into pairs.
- vertex cycle: The sequence of vertices that the side pairings carry into one another, as Section 4.1 defines. [SOLOMYAK2020] Definition 11 calls a finite one an elliptic cycle.
- cycle transformation: The product of the side pairings along a vertex cycle.
- Poincare triangle: In [MASKIT2001] Section 4.4 {P}: a triangle whose "interior angle at every elliptic vertex is of the form $\pi/\alpha$, $\alpha \in \mathbb Z$, $\alpha \ge 2$."
- pair of pants: A sphere with three boundary elements, each a closed geodesic, a cusp, or a cone point, as [MASKIT2001] Section 2 uses the term.
- size: In [MASKIT2001] Section 2 {P}: for a boundary generator $a$, the number $\sigma$ with $2\cosh\sigma = |\mathrm{tr}(\tilde a)|$, which is half the length of the closed geodesic. In Section 5 it extends to $\lambda = 0$ for a cusp and $\lambda = i\pi/\alpha$ for a cone point of order $\alpha$.
- primitive elliptic: In [MASKIT2001] Section 4 {P}: "An elliptic transformation a of order $\alpha$ is primitive if $|\mathrm{tr}(a)| = 2\cos(\pi/\alpha)$."
- Fenchel-Nielsen coordinates: The lengths and twists of a set of disjoint closed geodesics that cut an orbifold into pairs of pants, as [MASKIT2001] Section 2.2 defines them.
- twist: The displacement along a cut geodesic with which two pieces are glued. Section 2.5 lists the conventions.
- Teichmuller space: The space of hyperbolic structures on an orbifold, $\mathcal T(O)$ in [THURSTON2002] Corollary 13.3.7.
- cut: In SymmHub, a geodesic arc or closed geodesic along which the orbifold is split. A `tube` runs from cone to cone and carries a length and a twist. A `slice` runs from a cone to a mirror, or from corner to corner, and carries a length only.
- atom: In SymmHub, one of the twelve elementary pieces that remain after all cuts: a sheet `*abc`, a pillow `ABC`, or a hat `A*b`.
- corner pair: In SymmHub, the feature that takes the place of two consecutive corners of order 2 on a mirror boundary. It stands for a mirror side of variable length between two right angles.
- splane: In SymmHub, a sphere or a plane in $\mathbb R^3$ whose intersection with the plane $z = 0$ is a circle or a line of reflection.
- point reduction: The computation of an element $g$ of the group and the point $g(z)$ in the fundamental domain for a given point $z$.
- $(G, z)$-reduced: In [VOIGHT2009] Definition 4.1, quoted in Section 10.3.
- isometric circle: In [VOIGHT2009] Section 1 {P}: for $g$ acting on $\mathfrak D$ with matrix entries $c$ and $d$ in the bottom row, $I(g) = \{z : |cz + d| = 1\}$, with radius $1/|c|$ and centre $-d/c$.

### 2.5. Conventions on Which Sources Differ

| Topic | Convention | Source |
| --- | --- | --- |
| Model | Upper half-plane, $SL(2,\mathbb R)$ matrices, and $S^{\pm}L(2,\mathbb R)$ matrices of determinant $-1$ for reflections. The three-cone triangle case of [MASKIT2001] Section 5.9 uses the unit disc. | [MASKIT2001] Sections 4.3, 5 {P} |
| Model | Upper half-plane, moved to the unit disc by $\phi(z) = \frac{z - p}{z - \bar p}$, so that $\Gamma$ lands in $PSU(1,1)$ | [VOIGHT2009] Section 1 {P} |
| Model | Poincare disc. Each isometry is an ordered list of reflections in splanes, composed in the hyperboloid form $X \mapsto X - 2P\langle P, X\rangle/\langle P, P\rangle$ | [SYMMHUB] `lib/invlib/Inversive.js:134` {C} |
| Model, spherical | Stereographic plane, "scaled with the equator at the unit circle" | [SYMMHUB] `lib/orbilib/WallpaperGroups_NonNegative.js` {C} |
| Distance on the disc | A point at distance $d$ from the origin on the real axis is `poincarePt(d) = tanh(d/2)` | [SYMMHUB] `lib/invlib/ComplexArithmetic.js:237` {C} |
| Side of a splane | The sign of a sphere's radius selects the solid side. The hyperbolic triangle uses a negative radius, so the domain lies outside the circle. | [SYMMHUB] `lib/shaders/isplane.glsl.mjs` {C} |
| Length | Maskit works with sizes, half of the geodesic length. SymmHub sliders set edge lengths. | [MASKIT2001] Section 2 {P}, [SYMMHUB] {C} |
| Twist | The twist is $2\tau$, a hyperbolic distance "independent of the length of the geodesic we are twisting about", after Wolpert, with coordinate $t = \sinh\tau$ | [MASKIT2001] Section 2.2 {P} |
| Twist | A signed distance | Wikipedia, page not recorded {S} |
| Twist | Three conventions in one code base: a raw distance for tubes, a fraction of the edge length for handles, and a wrapped fraction for cone pairs, with sliders in $[-0.5, 0.5]$ | [SYMMHUB], Section 8.13 {C} |
| Twist | A real twist coefficient $t_i = a_i - b_i$ of a closed orientable surface, "congruent modulo $2\pi$ to the angle that $Q_i$ must rotate around $A_i$ so that the corresponding seams of $P_i$ and $Q_i$ match up" | [RATCLIFFE] Section 9.7, pp. 411 to 412, Theorem 9.7.4 {P} |
| Side-pairing direction | Section 9.3 sets $s = P \cap g_s(P)$, so that $g_s(s') = s$. Section 9.8, condition (A2), sets $g_s(s) = s'$. | [BEARDON] pp. 219 to 220, p. 242 {P} |
| Twist matrix | Maskit's Section 5.5 sets $\tilde f_\tau = \mathrm{diag}(e^{\tau}, e^{-\tau})$, which twists by $2\tau$. His Section 7.2 sets $\tilde f_\tau = \mathrm{diag}(e^{-\tau}, e^{\tau})$, which "twists by $-2\tau$". These are different maps with the same name. | [MASKIT2001] Sections 5.5, 7.2 {P} |
| Commutator | $[\alpha, \beta] = \alpha\beta\alpha^{-1}\beta^{-1}$ | [VOIGHT2009] equation (5.2) {P} |
| Commutator | $[X, Y] = X^{-1}Y^{-1}XY$ | [CONWAY-HUSON2002] p. 255 {N} |
| Conjugation of a boundary | $\lambda^{-1}P\lambda = S$ | [CONWAY-HUSON2002] p. 255 {N} |
| Conjugation of a boundary | $c_s = e\,c_0\,e^{-1}$ | [ARXIV-1702.02342] Section 3.2 {S} |
| Letters in Thurston | Formula 13.3.4 uses $k$ corners and $l$ cone points. The table of 13.3.6 lists cone orders first. Corollary 13.3.7 uses $k$ cone points and $l$ corners. This document uses the letters of Corollary 13.3.7. | [THURSTON2002] {N} |
| Area | Curvature $-1$ and area $2\pi\lvert\chi\rvert$ | [THURSTON2002] Theorem 13.3.5 {N} |
| Area | The normalized measure $\frac{1}{2\pi}\iint dx\,dy/y^2$, under which "an ideal triangle has area 1/2" | [VOIGHT2009] equation (3.2) {P} |
| Order of cone orders | Maskit lists the orders $\alpha_i$ "in decreasing order" | [MASKIT2001] Section 2 {P} |

This document follows Maskit's typography, in which $\tilde a$ is a matrix and $a$ is the map it induces. That reading of the notation is an assumption recorded in Section 13.5.

## 3. Problem Statement and Source Map

Escher receives an orbifold signature and, for signatures with a deformation space of positive dimension, values for the free parameters. It needs three things from them: a polygon $D$ that is a fundamental domain, one isometry for each side of $D$ that pairs it with its partner side, and a procedure that takes any point $z$ to a point $g(z) \in D$ with $g$ in the group. The construction is correct when the side pairings generate a discrete group for which $D$ is a fundamental domain. Poincare's polygon theorem, Section 4, is the tool that certifies this.

The sources differ sharply by class of signature. This table shows, for each class, where each part of the construction comes from and whether the class can be implemented from sources today. Section 13.1 lists the gaps that block each class.

| Class of signature | Polygon geometry | Side pairings | Presentation | Moduli | Implementable from sources today |
| --- | --- | --- | --- | --- | --- |
| Reflection triangle groups `*pqr` | [MASKIT2001] Section 4.4 {P}, [DAVIS2008] Theorem 6.4.3, Section 6.5 {P}, placement formula [SYMMHUB] {C} | Reflection matrices [MASKIT2001] Section 4.3 {P} | [MASKIT2001] Section 4.4 {P} | Unique up to congruence, [DAVIS2008] Example 6.5.3 {P} | Yes |
| Rotation triangle groups `pqr` | [MASKIT2001] Section 5.9 {P} | [MASKIT2001] Section 5.9 {P} | [SOLOMYAK2020] Theorem 12, [SERIES2013] Theorem 6.14 {P} | [THURSTON2002] Corollary 13.3.7 {N} | Yes for the group. The double of the reflection triangle as a fundamental domain is a candidate (Section 13.3, item 11) |
| Polygon reflection groups `*p1...pn`, one boundary, no cone points, handles, or cross-caps | Existence of a convex realization [DAVIS2008] Example 6.5.3, [BEARDON] Theorem 7.16.2 {P}, one polygon for each set of angles in the proof of [BEARDON] Theorem 7.16.2 {P}, the rest of the family only [SYMMHUB] sheets and slices {C} | Single reflections, [DAVIS2008] Theorem 6.4.3, [RATCLIFFE] Theorems 7.1.3 and 13.5.2 {P}, matrices [MASKIT2001] Section 4.3 {P} | Coxeter presentation, [DAVIS2008] Theorem 6.4.3, [RATCLIFFE] Theorem 7.1.4 {P} | $n - 3$ dimensions, [THURSTON2002] Theorem 13.8.1 {N}, [DAVIS2008] Example 6.5.3 {P} | Partly, one polygon for each set of angles |
| Orientable hyperbolic, with handles, cone points, and cusps, no mirrors | One polygon from the signature in the proof of [BEARDON] Theorem 10.4.2, with maps given by figures only {P} (Section 5.5), a Dirichlet domain of the generators (Section 10.6) | Generators, not side pairings of a polygon, [MASKIT2001] Section 8 {P} | [VOIGHT2009] equations (5.1)-(5.2) {P}, [CONWAY-HUSON2002] {N} | [MASKIT2001] Section 2.2, Theorem 10.1 {P} | Partly, generators only |
| Mixed signatures with cone points and mirrors, such as `4*3` | [SYMMHUB] hats {C} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N} | Code only |
| Several mirror boundaries, such as `**` in hyperbolic signatures | [SYMMHUB] bands {C} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N} | Code only |
| Mirrors together with handles | [SYMMHUB] {C} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N} | Code only |
| Non-orientable, with cross-caps | [SYMMHUB] caps {C} | Glide reflections [SYMMHUB] {C}, antipodal gluing [NORBURY2006] {S} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N}, [NORBURY2006] {S} | Code only |
| The 17 Euclidean groups | [SYMMHUB] catalogue {C}, triangle and kite regions [VONGAGERN2009] p. 12 {P} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | Partial, [VONGAGERN2009] p. 7, [SYMMETRIES-OF-THINGS2008] pp. 237 to 238 {P} | Partly, catalogue code |
| The 14 spherical families | [SYMMHUB] catalogue {C}, triangles [DAVIS2008] Example 6.5.1 {P} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | Not sourced | Partly, catalogue code |
| Bad orbifolds `m`, `mn`, `*m`, `*mn` with $m \ne n$ | No group exists, [CONWAY-HUSON2002] p. 254 {N} | None | None | None | Not applicable |

Only the triangle groups have a complete construction in primary sources. The orientable class has sourced generators, and one sourced polygon whose maps are given by figures only. Polygon reflection groups have sourced existence, discreteness, and parameter count, and one sourced polygon for each set of angles, but no coordinates for the rest of the family. The classes with mirrors together with cone points, handles, or other boundaries, and the non-orientable class, rest on the SymmHub code for their geometry and generators. Primary sources state the polygon theorem that certifies a polygon built for these classes (Section 4.7). The Euclidean and spherical classes rest on catalogue code beyond their reflection triangles.

## 4. Poincare's Polygon Theorem

### 4.1. Orientation-Preserving Statement of Solomyak

[SOLOMYAK2020] slides 19 to 22 {P} state the theorem for convex polygons with finitely many sides. The setup reads: "Let D a convex hyperbolic polygon, equipped with side-pairing transformations. That is, for each side s of D we have $\gamma_s \in PSL_2(\mathbb R)$ such that $s' = \gamma_s(s)$ is another side of D."

The vertex cycle is built in three steps:

1. Start from a vertex-side pair $(v_0, s_0)$ and apply $\gamma_1$ to get $(v_1, s_1)$.
2. "Now pass to the pair $(v_1, *s_1)$, where $*s_1$ is another side with the vertex $v_1$."
3. Repeat until the pair $(v_0, s_0)$ returns.

Definition 11 reads: "The sequence of vertices $E = v_0 \to v_1 \to \dots \to v_{n-1}$ is called an elliptic cycle. The transformation $\gamma_n\gamma_{n-1}\cdots\gamma_1$ is called an elliptic cycle transformation ... Say that the elliptic cycle condition holds for E if there exists $m = m(E) \ge 1$ such that $m \cdot \mathrm{sum}(E) := m \cdot (\angle v_0 + \dots + \angle v_{n-1}) = 2\pi$."

Theorem 12 has these hypotheses:

- $D \subseteq \mathbb H$ is a convex hyperbolic polygon with finitely many sides.
- $D$ is equipped with a collection $G$ of side-pairing transformations, and no side is paired with itself.
- $E_1, \dots, E_r$ are all the elliptic cycles, and each satisfies the elliptic cycle condition $m_j \cdot \mathrm{sum}(E_j) = 2\pi$ with $m_j \ge 1$.

Its conclusions are these:

1. "the subgroup $\Gamma = \langle G \rangle$ is a Fuchsian group".
2. "D is a fundamental domain for $\Gamma$".
3. $\Gamma = \langle \gamma_s \in G \mid \gamma_1^{m_1} = \dots = \gamma_r^{m_r} = e \rangle$, where $\gamma_j$ is the cycle transformation of a vertex on $E_j$.

Slide 22 adds two remarks. The first reads: "if a side is paired in itself by $\gamma$, we can introduce an extra vertex with angle $\pi$ in the middle, fixed by $\gamma$, and the theorem applies." This $\gamma$ is an orientation-preserving half-turn, not a reflection. The second reads: "There is a version of Poincare's Theorem for non-uniform lattices, when some of the vertices are in $\partial\mathbb H$."

Slide 12 attributes the theorem to "the paper by Poincare (1882)" and the first rigorous proof to "Maskit (1971)". [SOLOMYAK2020] covers $PSL_2(\mathbb R)$ only. Reflections and glide reflections are outside its scope.

### 4.2. Orientation-Preserving Statement of Series

[SERIES2013] Section 6.1, pp. 85 to 103 {P}, gives a second statement that also covers ideal vertices and sides on the boundary circle. Its setting is a "finite sided convex geodesic polygon which is a fundamental domain for some discrete group $\Gamma \subset \mathrm{Isom}^+\mathbb H$." Its definition of a side, which Section 2.4 of this document quotes, lets a side be part of an edge, and the text notes that "the definition of side-pairings allows a side to be paired to itself."

The side-pairing convention on p. 86 reads: "the side-pairing transformation associated to a side s of $\bar R$ is the unique element $g_s \in \Gamma$ which carries s to another side of $\bar R$." Lemma 6.2 states that $s = \bar R \cap g_s^{-1}\bar R$ and $g_s(s) = \bar R \cap g_s\bar R$, and that "If $g_s$ is a side pairing, then so is $g_s^{-1}$. Moreover $g_s^{-1}$ is the side-pairing associated to the side $g_s(s)$." The set of side pairings is therefore closed under inverses.

Section 6.1.6, p. 103, states two necessary conditions for "a convex polygon which is a locally finite fundamental domain for a discrete group G acting on $\mathbb H$":

- (a) For each vertex cycle with vertex $v \in \mathbb H$, the angle sum is $\sum_{i=0}^{k-1}\theta_i = \frac{2\pi}{p}$ with $p \in \mathbb N$. If $p = 1$ the cycle gives the relation $e_{k-1}\cdots e_0 = \mathrm{id}$. If $p > 1$, then $e_{k-1}\cdots e_0$ "is elliptic of order p, so that $(e_{k-1}\cdots e_0)^p = \mathrm{id}$."
- (b) "if R is finite sided and has an ideal vertex $v \in \partial\mathbb H$, then the vertex cycle corresponding to v is parabolic."

Theorem 6.14, p. 103, reads: "Suppose that $R \subset \mathbb H$ is finite sided convex polygon whose sides are identified in pairings by isometries $\{g_1, \dots, g_r\} = G^*$. Say the above conditions (a) and (b) hold." Its conclusions are these:

1. "$G^*$ generates a Fuchsian group G".
2. "R is a locally finite fundamental domain for the G action".
3. "$G = \langle g_1, \dots, g_r : \text{vertex relations}\rangle$, that is the vertex relations are all you need!"

The text after the theorem reads: "Notice that there is no requirement on cycles associated to sides of $\bar R$ contained in $\partial\mathbb H$. If such sides exist, then either G is elementary or the cycle is automatically hyperbolic." Proposition 6.11, p. 100, shows that a locally finite tessellation forces the cycle transformation $e_{k-1}\cdots e_0$ at an ideal vertex to be parabolic. Proposition 6.13, p. 102, reads: "If $\partial\bar R$ contains an interval in $\partial\mathbb H$, then either the corresponding vertex cycle h is hyperbolic or G is elementary." The proof of Theorem 6.14 is in Section 7.6, pp. 123 to 126, which was not read.

[SERIES2013] restricts $\Gamma$ to $\mathrm{Isom}^+\mathbb H$, so it does not cover reflections or glide reflections either.

### 4.3. Half-Turns and the Converse in Voight

[VOIGHT2009] Section 1 {P} splits a side that a half-turn fixes: "if $g \in \Gamma$ is an element of order 2 which fixes a side s of D, and s contains the fixed point of g, we instead consider s to be the union of two sides meeting at the fixed point of g." [SERIES2013] achieves the same effect through its definition of a side, and [SOLOMYAK2020] through its first remark.

Proposition 1.1 reads: "The Dirichlet domain D(p) has a side pairing P, and the set G(P) generates $\Gamma$. Conversely, let $D \subset \mathbb H$ be a hyperbolic polygon, and let P be a side pairing for D. Then D is a fundamental domain for the group generated by G(P)." The proof cites [BEARDON] Theorem 9.8.4 "and the accompanying exercises: the condition that $\mu(D) < \infty$ ensures that any vertex which lies on the circle at infinity is fixed by a hyperbolic element". Remark 1.2 says that the converse "extends to a larger class of polygons (see [2, Section 9.8])".

The converse as quoted states no cycle condition. Voight leaves that condition to [BEARDON] Theorem 9.8.4, whose hypotheses are a local tessellation condition and not an angle-sum rule (Section 4.11). Beardon's Theorem 9.3.8 says that such a vertex is fixed by a parabolic element and not by a hyperbolic one, whereas Voight's proof says "hyperbolic" (Section 12).

### 4.4. Reflection Triangle Groups in Maskit

[MASKIT2001] Section 4.4, p. 276 {P}, states the theorem for triangles bounded by mirrors. Let $r_i$ be the reflection in side $M_i$ and set $a_1 = r_2r_3$, $a_2 = r_3r_1$, and $a_3 = r_1r_2$. If $D$ is a Poincare triangle, then "Poincare's polygon theorem asserts" three things:

1. The group $\hat J = \langle r_1, r_2, r_3 \rangle$ is discrete.
2. $D$ is a fundamental polygon for $\hat J$.
3. $\hat J$ has this presentation:

$$\hat J = \langle r_1, r_2, r_3 : r_1^2 = r_2^2 = r_3^2 = (r_1r_2)^{\alpha_3} = (r_2r_3)^{\alpha_1} = (r_3r_1)^{\alpha_2} = 1 \rangle$$

The relation $(r_ir_j)^{\alpha_k} = 1$ "has its usual meaning if the vertex $v_{i,j}$ is elliptic of order $\alpha_k$. It means that $a_k$ is parabolic if $v_{i,j}$ is parabolic, and it has no meaning if $v_{i,j}$ is hyperbolic." A hyperbolic vertex is a pair of disjoint sides joined by their common orthogonal.

The orientation-preserving half $J$ satisfies: "If D is a Poincare triangle, then $\mathbb H^2/J$ is a pair of pants". The traces of its generators are $|\mathrm{tr}(\tilde a_k)| = 2\cosh\lambda_k$ when $\lambda_k$ is the distance between the sides, $2$ for a parabolic vertex, and $2\cos(\pi/\alpha_k)$ for an elliptic vertex.

[MASKIT2001] Section 4.3 {P} gives the reflection matrices. A reflection is a matrix in $S^{\pm}L(2,\mathbb R)$ with determinant $-1$ and trace $0$. For the line with endpoints $x$, the "positive endpoint", and $y$:

$$\tilde r = \frac{1}{x - y}\begin{pmatrix} x + y & -2xy \\ 2 & -x - y \end{pmatrix}$$

For the line with endpoints $\infty$ and $y$:

$$\tilde r = \begin{pmatrix} 1 & -2y \\ 0 & -1 \end{pmatrix}$$

Proposition 4.3 states that for lines that cross at angle $\theta$, $\mathrm{tr}(\tilde r_1\tilde r_2) = 2\cos\theta$. Equation (3) gives the reflection in the axis of a hyperbolic $\tilde a = \begin{pmatrix} \alpha & \beta \\ \gamma & \delta \end{pmatrix}$ with $\beta\gamma \ne 0$:

$$\tilde r_A = \frac{1}{\sqrt{(\alpha + \delta)^2 - 4}}\begin{pmatrix} \alpha - \delta & 2\beta \\ 2\gamma & \delta - \alpha \end{pmatrix}$$

### 4.5. Reflection Groups of Convex Polytopes in Davis

[DAVIS2008] Theorem 6.4.3, printed p. 83 {P}, covers reflection groups of convex polytopes in all three geometries. Its hypotheses are these:

- $P^n$ is a simple convex polytope in $X^n$, with $n \ge 2$.
- The dihedral angles have the form $\pi/m_{ij}$, with $m_{ij} \in \mathbb N$ whenever $F_i \cap F_j$ is nonempty.
- If $F_i \cap F_j$ is empty, $m_{ij} = \infty$.
- $W$ is the Coxeter group defined by the Coxeter matrix $(m_{ij})$, and $\bar W$ is the subgroup of $\mathrm{Isom}(X^n)$ generated by the reflections in the faces.

The theorem states that the natural map $\tilde\imath : U(W, P^n) \to X^n$ is a homeomorphism. It then lists three consequences:

1. "$\phi : W \to \bar W$ is an isomorphism."
2. "$\bar W$ acts properly on $X^n$."
3. "$P^n$ is a strict fundamental domain for the $\bar W$-action on $X^n$ (as in Definition 5.1.3)."

"Properly" is Definition 5.1.5, pp. 65 to 66: the orbit space is Hausdorff, every isotropy subgroup $G_y$ is finite, and $y$ has a $G_y$-stable neighbourhood $U_y$ with $gU_y \cap U_y$ empty for $g$ outside $G_y$. Remark 6.4.5, p. 84, adds: "(a) Theorem 6.4.3 (ii) implies that $\bar W$ is a discrete subgroup of $\mathrm{Isom}(X^n)$. (b) Part (iii) of the same theorem implies that $X^n$ is tiled by congruent copies of $P^n$." Definition 6.4.4 calls such an action "A geometric reflection group".

Two more statements in [DAVIS2008] describe the stabilizers {P}. After Definition 5.2.6, p. 68: "The isotropy subgroup at a point $[w, x]$ in $U(W, X)$ is $wW_{S(x)}w^{-1}$ ... each isotropy subgroup is a parabolic subgroup", where $S(x) = \{s \in S \mid x \in X_s\}$ is the set of mirrors through $x$ (equation (5.1), p. 63). A parabolic subgroup is a conjugate of a subgroup generated by a subset of $S$ (Section 4.5, p. 51). Lemma 5.3.1, p. 69, reads: "For any w in W, $X \cap wX = X_{S(w)}$." The research did not record how Davis defines $S(w)$ for an element $w$.

The preamble of Section 6.4, p. 82, explains why the angle condition is necessary: the subgroup generated by two reflections in intersecting faces "must be finite and the sector bounded by these hyperplanes which contains $P^n$ must be a fundamental domain", which "implies that the dihedral angle between these hyperplanes is an integral submultiple of $\pi$". Proposition 6.3.9, p. 80, shows that a convex polytope with nonobtuse dihedral angles is simple.

[DAVIS2008] Section 6.5 applies the theorem to polygons. Section 9.1 of this document gives its triangle cases, and Section 9.4 its polygon cases.

The preamble of p. 82 refers to Example 6.4.2, p. 82, the one-dimensional sphere: a circular arc of length $\pi/m$ with the reflections of $S^1$ across its endpoints, which generate the dihedral group $D_m$, "although the endpoints of I do not intersect" {P}. The proof of Theorem 6.4.3, p. 83, starts its induction from that example and remarks: "The same argument works even if the polyhedral cone is not essential, i.e., if $C^{n+1}$ stands for the intersection of k half-spaces, k < n + 1, in general position in $X^{n+1}$ with a ball about some point in the intersection of supporting hyperplanes." {P}, read on the page image.

The book defines a Euclidean convex polytope as "a compact intersection of a finite number of half-spaces" (p. 74), a spherical one as "the intersection of $S^n$ with an essential convex polyhedral cone in $\mathbb{R}^{n+1}$", where a polyhedral cone "is essential if it contains no line" (p. 75), and a hyperbolic one through a polyhedral cone inside the light cone (p. 76) {P}. A lune of $S^2$ is the intersection of two hemispheres ([RATCLIFFE] p. 51, Section 11.3 of `docs/research/geometry-models.md`), and by a reader's reading its cone contains the line through its two vertices, so it is not a convex polytope in this sense. None of pp. 82 to 84 mentions a lune or a group generated by two reflections of $S^2$ {P}. Item 9 of Section 13.3 records the lune as a candidate statement. The research found no discussion of ideal vertices in Chapter 6, so it is not established that Theorem 6.4.3 covers hyperbolic polygons with ideal vertices. This document applies it to compact polygons only. Section 4.7 gives the sources for reflection polygons with ideal vertices.

### 4.6. Variants in the Literature

These variants rest on secondary sources:

- Presentation for polyhedra in all three geometries: [JESPERS2015] Theorem 5.5 {S} states that for a locally finite fundamental polyhedron in Euclidean, hyperbolic, or spherical space, "the pairing and cycle relations form a complete set of relations". A self-paired side gives $\gamma_S^2 = 1$, a "reflection relation", and a cycle gives $(\gamma_{S_1}\cdots\gamma_{S_k})^t = 1$.
- Ideal vertices: de Rham 1971, as relayed by [ARXIV-MATH-0202156] App. B {S}, reads "If all the vertex-cycle transformations in G are parabolic, then G is a discrete subgroup." [ARXIV-2512.00879] App. A {S} replaces Maskit's completeness condition with a "parabolic cycle condition".
- General isometric face pairings: [ARXIV-0812.4161] Theorem 3.5 {S} requires that "the total interior angle equals $2\pi$ at some point of an edge for every cycle".

SymmHub follows the theorem in `lib/grouplib/GroupPresentation.js` ([SYMMHUB] lines 14-60 and 353 {C}). Its side-pairing relators are $w_iw_j = 1$, with $w_i^2 = 1$ for a self-paired side. It finds corner-cycle relators by walking vertices, with angle sum $2\pi/m$ and the $m$-th power as the relator. For reflection groups it uses Coxeter relators from the mirror angles. It checks every relator numerically and throws when the domain does not tile, with the constants `EPS=1e-7`, `MAX_CYCLE_LENGTH=100`, and `IDEAL_ANGLE=1e-5`. A domain twisted until its cells overlap fails that check, and such a group is derived from an untwisted copy through `derivedPresentation` (line 639).

### 4.7. Orientation-Reversing and Mixed Side Pairings

The orientation-preserving theorem is verified twice, in [SOLOMYAK2020] and [SERIES2013], including ideal vertices and sides on the boundary circle. The reflection group of a compact convex polygon is verified in [DAVIS2008], and the triangle case with ideal and ultra-ideal vertices in [MASKIT2001]. Four more cases arise in Escher's signatures. This table gives the statements that cover each case and a worked example where a source has one. Sections 4.9 to 4.11 state the theorems with their hypotheses.

| Case | Covered by {P} | Worked example {P} |
| --- | --- | --- |
| A polygon whose side pairings include both reflections and orientation-preserving maps, as in a signature with a cone point and a mirror | [EPSTEIN-PETRONIO1994] Theorem 4.14, or Theorem 8.1 with Theorem 5.5 and a completeness test when the polygon has an ideal vertex. [RATCLIFFE] Theorem 13.5.3. | [RATCLIFFE] Example 1 of Section 13.4, pp. 700 to 701, with its presentation in Example 1 of Section 13.5, p. 711. Example 6 of Section 13.4, p. 706, has a cusp as well. |
| Vertex cycles that pass through glide-reflection pairings, as in a signature with a cross-cap | [EPSTEIN-PETRONIO1994] Theorems 4.14 and 8.1, whose hypotheses put no condition on orientation. [RATCLIFFE] Theorem 13.5.3, and Theorem 11.2.2 for a group that acts freely. | [RATCLIFFE] Example 5 of Section 9.2, p. 383, which pairs consecutive sides of a regular $2n$-gon by "a rotation about the origin followed by the reflection in the corresponding side". The book does not call these maps glide reflections. |
| Corners with angle $\pi/m$ between mirror sides in a polygon that also has paired sides, as in a signature with a mirror and a handle | [RATCLIFFE] Theorem 13.5.3, through the dihedral ridge cycle of Section 2.4. The cycle conditions of [EPSTEIN-PETRONIO1994] (Section 4.9). | [RATCLIFFE] Example 1 of Section 13.4, which has one mirror side and two sides paired by a rotation |
| Reflection polygons with ideal vertices and more than three sides | [RATCLIFFE] Theorems 7.1.3 and 7.1.4 for a finite-sided polygon of finite area, and Theorem 13.5.2 for any convex polygon. In [EPSTEIN-PETRONIO1994], a chain of statements: the remarks on pp. 161 and 166, Lemma 5.4, and Theorem 5.5. | No polygon with more than three sides. [RATCLIFFE] Figure 7.3.1, p. 298, draws the Coxeter graphs of triangles with ideal vertices. |

Each statement has hypotheses that a construction must meet, which Sections 4.9 and 4.10 state. [EPSTEIN-PETRONIO1994] and [RATCLIFFE] both require a convex polygon. [EPSTEIN-PETRONIO1994] Theorem 4.14 (l) does not apply when two sides meet only at an ideal point (Lemma 3.26).

For a polygon with an ideal vertex, [EPSTEIN-PETRONIO1994] Theorem 8.1 gives a hyperbolic orbifold and a discrete group but not a tessellation (Remark 8.2). The tessellation needs Theorem 5.5 with a complete $Z$. [RATCLIFFE] Theorem 13.5.3 requires a complete glued orbifold, which Theorems 13.4.4 and 13.4.6 decide at the cusp points.

The case of reflection polygons with ideal vertices rests in [EPSTEIN-PETRONIO1994] on the chain of statements in the table, and not on one numbered theorem. The proof of [RATCLIFFE] Theorem 13.5.3 is a pointer (Section 4.10). Neither source works an example that combines a cross-cap with a cone point or a mirror. [BEARDON] Theorem 9.8.4 is stated for any isometries, but the book works only a polygon whose pairings are all reflections (Section 4.11), so this document does not cite it for these cases. Gap G1 in Section 13.2 records what stays open.

### 4.8. Implementer's Checklist

This checklist is assembled by the research from the statements of Sections 4.1 to 4.6 and 4.9 to 4.11, for a single convex polygon. Each item is sourced. [EPSTEIN-PETRONIO1994] Theorems 4.14 and 5.5 and [RATCLIFFE] Theorem 13.5.3 state the theorem for the general orientation-reversing case, and their hypotheses prevail over this list.

1. Every side is paired with exactly one side, possibly itself, and the pairing of the partner side is the inverse map ([SOLOMYAK2020] Definition 5 and Lemma 6, [SERIES2013] Lemma 6.2, [VOIGHT2009] Section 1, [EPSTEIN-PETRONIO1994] Condition 3.2 (a) and (d) {P}). Each pairing maps the interior of the polygon "to the other side of" the partner side, and "gives an isomorphism between the face structure" of the side and that of its partner ([EPSTEIN-PETRONIO1994] Condition 3.2 (b) and (c), p. 126 {P}). [BEARDON] p. 248 assumes implicitly, at an interior point $x$ of a side, that $g_1(N_1) \cup g_2(N_2)$ "is a neighbourhood of $x$", where $N_1$ and $N_2$ are the sets of its condition (A6) {P}.
2. A side paired with itself has a pairing of order two, which need not be the reflection in that side ([EPSTEIN-PETRONIO1994] Remark 3.3, p. 126, [RATCLIFFE] condition (4) on p. 694 and the proof of Theorem 13.5.1, p. 710 {P}). The sources permit a split of a side fixed by a half-turn at its fixed point, with a new vertex of angle $\pi$ ([VOIGHT2009] Section 1, [SOLOMYAK2020] slide 22, [SERIES2013] Definition 6.1 {P}). None of them requires the split: Solomyak writes "we can introduce an extra vertex", and [EPSTEIN-PETRONIO1994] Definition 4.2 and [RATCLIFFE] p. 694 treat a side paired with itself without one {P}.
3. Every vertex cycle at a point of the plane is finite, and the product of the pairings along it is a rotation through the angle sum of the cycle, which has the form $2\pi/m$ for an integer $m \ne 0$ ([EPSTEIN-PETRONIO1994] Conditions 3.7, 3.10, and 3.13 and Lemma 3.12, pp. 127 to 129 {P}). [RATCLIFFE] p. 695 {P} requires the angle sum to be a submultiple of $2\pi$ for a cyclic ridge cycle and of $\pi$ for a dihedral ridge cycle (Section 2.4). For orientation-preserving pairings, $m \cdot \sum\angle = 2\pi$ with $m \ge 1$, and the cycle transformation has order $m$ ([SOLOMYAK2020] Theorem 12, [SERIES2013] condition (a) {P}).
4. In a reflection polygon, every corner has angle $\pi/m$, and the relator is $(r_ir_j)^m$ ([DAVIS2008] Theorem 6.4.3 for compact polygons, [MASKIT2001] Section 4.4 for triangles, [RATCLIFFE] Theorems 7.1.3 and 7.1.4 for finite-sided polygons of finite area {P}). Theorem 7.1.4 deletes the relator when $m = \infty$.
5. A closed cycle of ideal vertices has a parabolic cycle transformation ([SERIES2013] condition (b) and Proposition 6.11, [MASKIT2001] Section 4.4 {P}, [RATCLIFFE] Theorems 9.8.3 and 9.8.5 for surfaces {P}, de Rham {S}). The condition concerns a closed cycle. The proof of [EPSTEIN-PETRONIO1994] Theorem 8.1, pp. 162 to 164, gets a cusp from parabolic holonomy for pieces at an ideal point that glue in a cyclic fashion, leaves $Q$ incomplete when the holonomy is hyperbolic, and states that pieces that do not glue in a cyclic fashion are complete (cases (a) to (c), Section 4.9) {P}. [RATCLIFFE] Theorems 13.4.4 and 13.4.6 decide completeness at a cusp point of an orbifold by a condition on links, not on a cycle transformation {P}. The test to run where a mirror ends at a cusp is open (Section 13.3, item 8).
6. A side on the boundary circle needs no condition, because its cycle is hyperbolic or the group is elementary ([SERIES2013] Theorem 6.14 and Proposition 6.13 {P}). [EPSTEIN-PETRONIO1994] Theorem 8.1 puts no hypothesis on such sides, and case (c) of its proof covers a chain that ends at a ray whose ideal point is not a peak {P}.
7. The presentation is the pairing relators together with the cycle relators ([SOLOMYAK2020] Theorem 12 (3), [SERIES2013] Theorem 6.14 (iii), [RATCLIFFE] Theorem 13.5.3 {P}, [JESPERS2015] {S}). In [EPSTEIN-PETRONIO1994], the relators are those of Definition 4.2, and they present the group generated by the pairings when the holonomy is injective, which Theorem 4.14 (g) states under the hypothesis Metric {P}.

### 4.9. Polyhedron Theorem of Epstein and Petronio

[EPSTEIN-PETRONIO1994] {P} states the theorem in $X^n$, which is $E^n$, $S^n$, or $H^n$, with $n \ge 2$ (p. 125). The input is a finite or countable set of thick convex cells. A convex polyhedron is an intersection of half-spaces in which each point has a neighbourhood that meets finitely many of their boundaries, and it is thick when its interior is non-empty (Definition 2.1, pp. 116 to 117). A convex cell is a convex polyhedron "whose proper faces may have been subdivided" (Definition 2.8, p. 121).

Condition 3.2, Pairing, p. 126, asks for a map $R$ on the faces and a map $A$ to the isometries of $X^n$ with these properties, where $R(F, P) = (F', P')$:

- (a) $R$ "is an involution, that is $R \circ R$ is the identity."
- (b) $A(F, P)$ "maps $F$ onto $F'$ and maps the interior of $P$ to the other side of $F'$ from the interior of $P'$."
- (c) $A(F, P)$ "gives an isomorphism between the face structure of $F$ and the face structure of $F'$."
- (d) $A(R(F, P)) = A(F, P)^{-1}$.

No condition restricts the orientation of $A(F, P)$. Remark 3.3 reads: "In case $R(F,P) = (F,P)$ Condition 3.2(d) implies that $A(F,P)$ is a mapping of order two. Note that in this special situation $A(F,P)$ is not necessarily the reflection in the face $F$, though that is a common application of this theory." Condition 3.5, Connected, asks that "any two elements of $\mathscr P$ are joined by a sequence of face-pairings."

The cycle conditions start from a cell $P_1$, a face $F_1$, and a codimension-one face $C_1$ of $F_1$, which is a vertex in the plane. The cycle passes to the other face $F_1'$ of $P_1$ that contains $C_1$, applies $g_1 = A(F_1', P_1)$, and continues, so that $g_{i-1} \circ \dots \circ g_1(C_1) = C_i$ (p. 127). The conditions are these:

- FirstCyclic, Condition 3.7, p. 127: the sequence returns to its start after some $r \ge 1$ steps.
- SecondCyclic, Condition 3.10, p. 128: for some $r \ge 1$ the sequence returns to its start and $g_r \circ \dots \circ g_1$ is the identity on $C_1$. The least such $r$ is the second cycle length.
- ThirdCyclic, Condition 3.13, p. 129: the map $g = g_r \circ \dots \circ g_1$ of Lemma 3.12 is "a rotation through an angle of the form $2\pi/m$ for some non-zero $m \in \mathbb Z$."
- Cyclic, Condition 3.19, p. 132: the conjunction of the three.

Lemma 3.12, p. 129, states that with $r$ the second cycle length and $\theta_i$ the dihedral angle of $P_i$ along $C_i$, the map $g$ "is a rotation through an angle $\sum\theta_i$ around the codimension-two subspace of $X^n$ containing $C_1$." Examples 3.15 to 3.18, pp. 131 to 132, work these conditions for a dihedral wedge in $E^3$, and Example 3.15 shows that reflections in both faces give the condition $\varphi = \pi/m$.

Condition 3.9, Finite, p. 128, asks that the set of cells is finite and each cell has finitely many faces. FirstMetric, Condition 3.20, p. 132, asks for an $\varepsilon > 0$ such that disjoint faces of a cell are at distance at least $\varepsilon$, and Metric, Condition 3.23, adds SecondMetric. Remark 3.24, p. 134, states that Finite implies Metric "unless $X^n = H^n$". Lemma 3.26, p. 134, reads: "Let $P$ be a convex cell in $H^n$ with finitely many faces. Two disjoint faces of $P$ can have at most one common point at infinity, and they are a positive distance apart if and only if they have no common point at infinity." Condition 3.29, p. 135, states that in dimension two LocallyFinite is equivalent to FirstCyclic.

Theorem 4.14, Poincare's Theorem Version 1, p. 142, assumes Pairing, Connected, Cyclic, and Metric. It concludes these, among others:

- (f) $Z$ carries an $X^n$-structure for which the developing map $D_Z : Z \to X^n$ is a bijective isometry.
- (g) The holonomy $h : G \to \mathrm{Isom}(X^n)$ is injective, and its image is a discrete subgroup.
- (h) $D_Y$ is injective on the interior of $Y$, "so that $D_Y(Y)$ or its interior can be considered as a fundamental domain for the action of $h(G)$ on $X^n$, depending on the precise definition of that concept". Example 4.12 shows that $D_Y$ need not be injective on all of $Y$.

The hypotheses hold when Pairing, Connected, and Cyclic hold together with either (k) $X^n = E^n$ or $S^n$ and Finite, or (l) $X^n = H^n$, Finite, and FirstMetric. The group $G$ of Definition 4.2, pp. 137 to 138, depends on a maximal tree $T$ in the graph $\Gamma(\mathscr P, R)$. It has one generator $\alpha_{(F,P)}$ for each face, with $\alpha_{(F,P)} = \mathrm{id}$ when the edge $e(F,P)$ lies in $T$. For each other face with $R(F,P) = (F',P')$ it has the relation $\alpha_{(F,P)}\alpha_{(F',P')} = \mathrm{id}$, so that a face paired with itself gives a generator of order two. For each cell $P_1$ and each codimension-two face $C_1$ of $P_1$ it has the relation $(\alpha_{(F_r',P_r)} \cdots \alpha_{(F_1',P_1)})^m = \mathrm{id}$, in the notation of Conditions 3.7 and 3.13. Lemma 4.4 sends each generator to $A(F, P)$. The introduction, p. 116, adds: "Our proof shows that the usual checks for Poincare's Theorem, in the case where there is only one convex piece, in any case imply the embeddability of the potential fundamental domain, so no special separate check is necessary."

Theorem 5.5, Poincare's Theorem Version 2, p. 148, assumes Pairing, Connected, Cyclic, and LocallyFinite, and reads: "If $Z$ is complete, then it is isometric to $X^n$." Lemma 5.4, p. 147, states that under the same four hypotheses $Q$ is complete if and only if $Z$ is complete. Section 6 opens, p. 148, with the statement that completeness follows from Finite in the Euclidean and spherical cases. For hyperbolic cells that satisfy Pairing, Connected, Cyclic, and Finite, Theorem 6.3, pp. 152 to 153, gives four equivalent conditions, among them "(a) $Z$ is complete" and "(b) For each $P$ and each boundary point $p$ of $P$, $Z_p$ has a consistent horosphere". Its condition (c) can be checked by an algorithm, and uses the words "hyperbolic or loxodromic", which the paper does not define.

Theorem 8.1, p. 161, which the paper calls "essentially due to de Rham", reads in part: "Suppose we have a finite set $\mathscr P$ of finite-sided polygons in $H^2$ and an edge-pairing $(R,A)$ of the boundary edges satisfying Pairing, Connected and Cyclic. Then the quotient $Q$ ... is a two-dimensional hyperbolic orbifold which is obtained from a complete orbifold with geodesic boundary by removing the compact boundary components. ... The group $G$ generated by the edge-pairings in the manner described in Definition 4.2 is discrete. If all the polygons are compact, then $Q$ is a compact orbifold without boundary. (But it may have mirrors.)" Remark 8.2 states that the theorem describes $Q$ even when $Q$ is not complete, and refers to Lemma 5.4 and Theorem 6.3 for completeness. Remark 8.3 states that a side paired with itself by a reflection gives a mirror.

The proof of Theorem 8.1, pp. 162 to 164, removes a horodisc at each peak, an ideal point that "is the end of two distinct boundary components" of a polygon, and distinguishes three cases for the pieces at a peak:

- (a) The pieces glue in a cyclic fashion with parabolic holonomy. The piece is complete, and gives a cusp.
- (b) The pieces glue in a cyclic fashion with hyperbolic holonomy. The piece is incomplete, and the completion of $Q$ adds a compact geodesic boundary component.
- (c) The pieces do not glue in a cyclic fashion, and in both directions reach a ray "which is glued either to itself (by a reflection) or to a geodesic ray whose point at infinity is not a peak". The piece is complete.

The first paragraph of Section 8, p. 161, reads: "One case of Poincare's Theorem which is often used is the case where there is a single element of $\mathscr P$ and all face-pairings are reflections. In that case completeness is a consequence of Lemma 5.4, provided the other axioms are satisfied." Page 166 repeats this for a single polyhedron with each face-pairing equal to the reflection in that face, where "The only condition to check is that the angles at codimension 2 faces have the form $\pi/m$ for some integer $m$". It adds that this version "is readily deduced from Theorem 5.5 using 5.4", because "LocallyFinite is obvious in this case and the quotient space is complete as it is identified with the polyhedron itself". The two pages state this as remarks and not as a numbered theorem.

Two examples serve as negative tests:

- Remark 3.14, p. 130: a wedge in $E^2$ of angle $2\pi/3$ with reflections as face pairings has the angle sum $4\pi/3$, which is not of the required form, although the images of the wedge tile $E^2$.
- Example 3.27, p. 135: the polygon $[1,2] \times (0,\infty)$ in the upper half-plane with the face pairing $z \mapsto 2z$ satisfies Pairing and Cyclic, "However the images of $P$ under the powers of the pairing cover only the right half of the half-plane." [BEARDON] Exercise 9.8, item 3, pp. 251 to 252, poses the same example.

### 4.10. Orbifold Theorem of Ratcliffe

[RATCLIFFE] {P}, in its first edition, states the theorems for $X = S^n$, $E^n$, or $H^n$ with $n > 1$ (p. 709 and p. 511). A polyhedron is a nonempty closed convex subset of $X$ whose collection of sides is locally finite (Section 6.3, p. 205). Theorem 13.4.1, p. 695, states that for a group $G$ of similarities of $X$, gluing a disjoint set of $n$-dimensional convex polyhedra by a subproper $G$-side-pairing gives an $(X, G)$-orbifold.

Theorem 13.5.3, p. 711, reads: "Let $\Phi$ be a subproper $I(X)$-side-pairing for an $n$-dimensional, convex polyhedron $P$ in $X$ such that the $(X, I(X))$-orbifold $M$ obtained from $P$ by gluing together the sides of $P$ by $\Phi$ is complete. Then the group $\Gamma$ generated by $\Phi$ is discrete, $P$ is an exact, convex, fundamental polyhedron for $\Gamma$, and if $S$ is the set of sides of $P$ and $R$ is the set of words in $S$ corresponding to all the side-pairing and cycle relations of $\Gamma$, then $(S, R)$ is a group presentation for $\Gamma$ under the mapping $S \mapsto g_S$." The book writes a semicolon where this document writes the comma in $(S, R)$. The side-pairing relation is $g_Sg_{S'} = 1$, (6.7.4), and the cycle relation is $(g_{S_1} \cdots g_{S_\ell})^k = 1$ with $k$ the order of the cycle transformation, (6.7.3), p. 259. The proof is a short pointer and not a full argument. It reads that it is "essentially the same as the proof of Theorem 11.2.2", with one change in its step (10).

Theorem 11.2.2, p. 513, has the same form for a proper side pairing and a complete manifold, and concludes that $\Gamma$ "is discrete and acts freely". A proper side pairing has finite cycles with angle sum $2\pi$ at each vertex cycle in the plane (p. 505 and p. 375). The theorem therefore excludes mirrors and cone points, and its cycle relation is $g_{S_1} \cdots g_{S_\ell} = 1$.

Theorem 13.5.2, p. 710, reads: "Let $P$ be an $n$-dimensional convex polyhedron in $X$ all of whose dihedral angles are submultiples of $\pi$. Then the group $\Gamma$ generated by the reflections of $X$ in the sides of $P$ is a discrete reflection group with respect to the polyhedron $P$." It has no finiteness hypothesis and gives no presentation. Theorem 7.1.3, p. 265, states the same for a finite-sided polyhedron of finite volume, and its proof treats ideal vertices with horoballs (p. 269). Theorem 7.1.4, p. 274, gives the presentation `<S_i ; (S_i S_j)^(k_ij)>` with $k_{ij} = \pi/\theta(S_i, S_j)$ for a discrete reflection group with respect to a finite-sided polyhedron of finite volume. It is written in a code span because the book uses a semicolon. The relator is deleted when $k_{ij} = \infty$. Exercises 7.1.11 to 7.1.13, p. 276, state as exercises that the finite-volume hypothesis can be dropped.

The dihedral angle of two sides is defined by (6.7.2), p. 255. Two sides are adjacent when their intersection is a side of both, and two distinct sides that are not adjacent have angle $0$.

At ideal points, a cusp point of $P$ is "a point $c$ of $\bar P \cap S^{n-1}$ that is the intersection of the Euclidean closures of all the sides of $P$ incident with $c$" (p. 507 and p. 705). Theorem 13.4.4, p. 705, reads: "The link $L[c]$ of a cusp point $[c]$ of $M$ is complete if and only if the links $\{L(b)\}$ for the points in $[c]$ can be chosen so that $\Phi$ restricts to a side-pairing for $\{L(b)\}$." Theorem 13.4.6, p. 706, states that a hyperbolic orbifold glued from a finite family of disjoint, finite-sided convex polyhedra in $B^n$ by a subproper side pairing is complete if and only if $L[c]$ is complete for each cusp point. For surfaces, Theorem 9.8.3, p. 419, states that the gluing invariant $d(v)$ of an ideal vertex is zero if and only if the cycle transformation is parabolic. Theorem 9.8.5, p. 421, states that a hyperbolic 2-manifold glued from a finite family of disjoint, convex, finite-sided polygons of finite area by a proper $I(H^2)$-side-pairing is complete if and only if $d(v) = 0$ for each ideal vertex $v$. Theorem 9.8.7, p. 424, states that for one such polygon whose gluing invariants are all zero, the group is discrete and torsion-free and the polygon is an exact convex fundamental polygon. These proper $I(H^2)$-side-pairings may reverse orientation.

### 4.11. Poincare's Theorem in Beardon

[BEARDON] Theorem 9.8.4, p. 249 {P}, reads: "For a polygon $P$ with a side-pairing $\Phi$ satisfying (A6)' and (A7), $G$ is discrete and $P$ is a fundamental polygon for $G$." Its hypotheses are these:

- (A1) to (A5), pp. 242 to 244: $P$ is an open, connected, abstract polygon in a metric space $X$, with an involution $s \mapsto s'$ on its sides and isometries $g_s$ of $X$ onto itself with $g_s(s) = s'$ and $g_{s'} = (g_s)^{-1}$. $G$ is the group that the $g_s$ generate.
- The setting of p. 248: $X$ is the hyperbolic plane, and "the argument will work equally well in the Euclidean plane or in the sphere $S^2$". $P$ is a hyperbolic polygon, "possibly with vertices and free sides on the circle at infinity", and "there is no need to check (A6) at points on the circle at infinity."
- The implicit assumption of p. 248, stated in item 1 of Section 4.8.
- (A6)', p. 249: for each vertex $x$ of $P$ there are vertices $x_0 = x, x_1, \dots, x_n$ of $P$ and elements $f_0 = I, \dots, f_n$ of $G$ such that the images $f_j(N_j)$ of the sets $N_j$ of condition (A6) do not overlap and their union is the disc $B(x, \varepsilon)$, and each $f_{j+1}$ has the form $f_jg_s$ for some side $s$ ($j = 1, \dots, n$, with $f_{n+1} = I$).
- (A7), p. 249: "the $\varepsilon$ in (A6) can be chosen independently of $x$ in $\bar P$".

A remark after the theorem reads: "If $P$ has no vertices in $X$, then (A6)' is automatically satisfied. However, (A7) need not hold." The hypothesis is a local tessellation condition, not an angle-sum rule, and the theorem states no presentation. The book defines a fundamental domain and discreteness for groups of orientation-preserving maps (Definition 9.1.1, p. 204, and Chapter 5). Its only worked case with reflections is Example 9.8.5, p. 249: a polygon with angles $\pi/n_j$ at its vertices, whose pairings are the reflections in its sides. Exercise 9.8, items 1 to 3, pp. 251 to 252, asks the reader to extend Example 9.8.5 to vertices on the circle at infinity "fixed by a parabolic elements", to generalize to arbitrary polygons with such vertices, and to show that the parabolic condition is essential. The exercise has no printed solution.

For a convex fundamental polygon of a Fuchsian group, Theorem 9.3.5, p. 221, states $\theta(C) = 2\pi/\mathrm{ord}(C)$ for a cycle $C$ of vertices. Theorem 9.3.8, p. 224, reads: "Let $v$ be any point of $E$ that is fixed by some non-trivial element of $G$. Then $v$ is fixed by a parabolic element of $G$ (and not by any hyperbolic element)." Here $E$ is the part of the boundary of the polygon on the circle at infinity (p. 223).

## 5. Canonical Polygons and Presentations

### 5.1. Fuchsian Canonical Presentation

[VOIGHT2009] Section 5, equations (5.1) and (5.2) {P}, gives the presentation of a Fuchsian group of signature `(g; m_1, ..., m_t; s)`. The generators are $\alpha_1, \dots, \alpha_g$, $\beta_1, \dots, \beta_g$, $\gamma_1, \dots, \gamma_t$, and $\gamma_{t+1}, \dots, \gamma_{t+s}$. The relations are:

$$\gamma_1^{m_1} = \dots = \gamma_t^{m_t} = [\alpha_1, \beta_1]\cdots[\alpha_g, \beta_g]\,\gamma_1\cdots\gamma_{t+s} = 1$$

Here $[\alpha, \beta] = \alpha\beta\alpha^{-1}\beta^{-1}$. "One obtains a minimal set of generators from this presentation by eliminating $\gamma_{t+s}$ whenever $t + s > 0$." The area in the normalized measure of Section 2.5 is $A = 2g - 2 + \sum_q e_q(1 - 1/q) + e_\infty$ ([VOIGHT2009] equation (3.2) {P}).

Voight states that the presentation his Dirichlet algorithm returns "is not necessarily of the form (5.1)-(5.2)", and points to [IMBERT2001] for the canonical form. The edge word $a_1b_1a_1^{-1}b_1^{-1}\cdots$ of the canonical polygon of the genus part appears only in [WIKI-FUNDAMENTAL-POLYGON] {S}.

### 5.2. Conway Presentation Read from the Signature

[CONWAY-HUSON2002] pp. 255 to 257 {N} reads a presentation from the signature. Each feature gets one Greek generator "that circumnavigates that feature in the positive direction", and possibly Latin generators. `docs/research/orbifold-notation.md` gives the full treatment. The construction needs this summary:

| Feature | Latin generators | Relations |
| --- | --- | --- |
| Handle `o` | $X$, $Y$ | $[X, Y] = X^{-1}Y^{-1}XY = \alpha$ |
| Cone point of order $A$ | None | $\gamma^A = 1$ |
| Boundary `*abc...` with $n$ corners | $n + 1$ reflections $P, Q, \dots, S$ | $P^2 = (PQ)^a = Q^2 = (QR)^b = \dots = S^2 = 1$ and $\lambda^{-1}P\lambda = S$ |
| Cross-cap `x` | $Z$ | $Z^2 = \omega$ |
| Whole signature | None | "the product of all Greek generators is 1": $\alpha\cdots\gamma\cdots\lambda\cdots\omega = 1$ |

[CONWAY-HUSON2002] Figure 21, p. 255, fixes the base point and the order of the features in the global relation.

### 5.3. Groups Generated by Reflections

[THURSTON2002] p. 320 {N} reads: "When O has the combinatorial type of a polygon ... $\pi_1 O$ is generated by reflections". That is the class of signatures with a single boundary `*n1...ns` and no other feature. [THURSTON2002] Proposition 13.3.1(iii), p. 308 {N}, gives the local group of a corner reflector as $\langle a, b : a^2 = b^2 = (ab)^n = 1\rangle$. [SYMMETRIES-OF-THINGS2008] p. 127 {P} lists `*632`, `*442`, `*333`, `*2222`, `*532`, `*432`, `*332`, `*22N`, and `*NN` as the groups with a presentation by reflections of this type. For `**`, p. 120 {P} reads: "Each * is caused by a reflection, but those two reflections do not suffice to generate the group!" Whether some other set of reflections generates `**`, or any other group with several boundaries, is not stated by any source read (gap G15).

### 5.4. Area and Euler Characteristic as a Check

A constructed polygon can be checked against the area that the signature predicts. [THURSTON2002] Theorem 13.3.5 {N} states $\int_O K\,dA = 2\pi\chi(O)$, so a hyperbolic orbifold of curvature $-1$ has area $2\pi\lvert\chi(O)\rvert$, with no assumption of orientability. A spherical group has order $2/\chi$ ([CONWAY-HUSON2002] pp. 252 to 253 {N}). `docs/research/orbifold-notation.md` gives $\chi$ for each signature.

### 5.5. Canonical Polygon Geometry

No source that was read gives coordinates or matrices for the canonical polygon of a signature with cone points and cusps, the polygon whose edge word realizes equations (5.1) and (5.2). Section 13 records this as gap G6.

[BEARDON] {P} constructs one polygon from a signature for orientation-preserving groups. Definition 10.4.1, p. 268, writes the signature as $(g : m_1, \dots, m_r, s, t)$, with semicolons in place of the last two commas, where $s$ counts the classes of parabolic cyclic subgroups and $t$ the classes of boundary hyperbolic ones. Theorem 10.4.2, p. 268, reads: "There is a non-elementary finitely generated Fuchsian group with signature (10.4.1) and $m_j \ge 2$ if and only if $2g - 2 + s + t + \sum_{j=1}^r(1 - \frac{1}{m_j}) > 0$." Corollary 10.4.4, p. 269, gives the area $2\pi[2g - 2 + s + \sum_{j=1}^r(1 - 1/m_j)]$ of any convex fundamental polygon of a group of the first kind.

The proof of sufficiency of Theorem 10.4.2, pp. 271 to 274, builds the polygon in these steps:

1. Place $4g + r + s + t$ points $z_j$ equally spaced on the hyperbolic circle $\rho(z, 0) = d$, so that each arc subtends $2\theta$ at the origin with $\theta = 2\pi/(8g + 2r + 2s + 2t)$.
2. Use the first four arcs for a handle with hyperbolic maps $h_1$ and $h_2$ (Figure 10.4.1), and repeat for each of the $g$ handles.
3. Use the next $r$ arcs for the cone points, each with "an elliptic element $e_i$ of order $m_i$" and the angle $2\pi/m_j$ at its fixed point $w_j$ (Figure 10.4.2).
4. Use the next $s$ arcs for the cusps, with $w_i$ on the unit circle, angle zero, and a parabolic map $p_i$.
5. Use the last $t$ arcs for hyperbolic maps $b_j$ with free sides (Figure 10.4.3).
6. Choose $d$ so that the angle sum $\phi(d) = 8g\alpha + 2(\beta_1 + \dots + \beta_{r+s+t})$ at the points $z_j$ equals $2\pi$, where $\cosh d = \cot\theta\cot\alpha$ and $\cosh d = (\cos\theta\cos\beta_j + \cos(\pi/m_j))/(\sin\theta\sin\beta_j)$, with $\cos(\pi/m_j)$ replaced by 1 for a cusp. The proof prints a third relation, with an angle $\theta_1 = \frac{1 + d}{1 + 2d}\theta$, for the free sides.

The proof finds $d$ by the intermediate value theorem, since $\phi(d)$ exceeds $2\pi$ as $d \to 0$ and tends to 0 as $d \to +\infty$. It gives the maps by figures only, with no matrices, and it invokes Poincare's theorem through Exercise 9.8, item 2, which has no printed solution. It covers no mirror and no cross-cap. The research did not compare the edge word of this polygon with equations (5.1) and (5.2).

### 5.6. NEC Signatures and Their Edge Words

[SINGERMAN1974] and [MACBEATH1967] {P} give the presentation and the combinatorial edge word of a fundamental polygon of an NEC group with compact quotient. They give no side lengths, no coordinates, and no construction of a polygon, and they exclude parabolic elements by their definition of an NEC group (Section 2.4).

Singerman writes the signature as `(g; +; [m_1, ..., m_r]; {(n_11, ..., n_1s_1), ..., (n_k1, ..., n_ks_k)})`, or with $-$ for a non-orientable quotient (p. 233). The generators are $x_i$ for the periods, $e_i$ and $c_{i0}, \dots, c_{is_i}$ for each period cycle, and $a_i, b_i$ for the genus, or only $a_i$, glide reflections, for the sign $-$ (p. 234). For the sign $+$ the relations of (5), p. 234, are:

$$x_i^{m_i} = 1, \quad c_{is_i} = e_i^{-1}c_{i0}e_i, \quad c_{i,j-1}^2 = c_{ij}^2 = (c_{i,j-1}c_{ij})^{n_{ij}} = 1 \ (j = 1, \dots, s_i)$$

$$x_1 \cdots x_r\, e_1 \cdots e_k\, a_1b_1a_1^{-1}b_1^{-1} \cdots a_gb_ga_g^{-1}b_g^{-1} = 1$$

For the sign $-$, (6) replaces the last relation by $x_1 \cdots x_r\, e_1 \cdots e_k\, a_1^2 \cdots a_g^2 = 1$. Singerman prints a comma after $x_1x_2$ in (5), and Macbeath's table on p. 1197 prints the relation without it. Neither prints a relation $c_{i0}^2 = 1$ for a period cycle with no periods.

The edge word (3), p. 234, for the sign $+$ is:

$$\xi_1\xi_1' \cdots \xi_r\xi_r'\ \epsilon_1\gamma_{10} \cdots \gamma_{1s_1}\epsilon_1' \cdots \epsilon_k\gamma_{k0} \cdots \gamma_{ks_k}\epsilon_k'\ \alpha_1\beta_1\alpha_1'\beta_1' \cdots \alpha_g\beta_g\alpha_g'\beta_g'$$

Here $x_i(\xi_i') = \xi_i$, $e_i(\epsilon_i') = \epsilon_i$, $a_i(\alpha_i') = \alpha_i$, and $b_i(\beta_i') = \beta_i$, and "The sides labelled $\gamma_{ij}$ are fixed pointwise by a reflexion $c_{ij}$." For the sign $-$, (4) ends with $\alpha_1\alpha_1^* \cdots \alpha_g\alpha_g^*$, with $a_i(\alpha_i^*) = \alpha_i$ "identified non-orientably". [MACBEATH1967] p. 1196 writes the genus part as $\alpha_1\beta_1'\alpha_1'\beta_1 \cdots$ instead. Singerman p. 235 gives the order of the stabilizer at each class of vertices: $m_i$ at $(\xi_i, \xi_i')$, $2n_{ij}$ at $(\gamma_{i,j-1}, \gamma_{ij})$, and 2 at $(\epsilon_i, \gamma_{i0})$ and at $(\gamma_{is_i}, \epsilon_i')$.

Singerman's Theorem 1, p. 235, gives the area of a fundamental polygon for the sign $+$ as $2\pi(2g - 2 + \sum_{i=1}^r(1 - 1/m_i) + k + \sum_{i=1}^k\sum_{j=1}^{s_i}\frac12(1 - 1/n_{ij}))$, and for the sign $-$ with $g - 2$ in place of $2g - 2$.

For a proper NEC group, Singerman p. 235 states that $\Gamma^+ = \Gamma \cap \mathcal G^+$ has index two. His Theorem 2, p. 236, gives the Fuchsian signature of $\Gamma^+$ as $(2g + k - 1, m_1, m_1, \dots, m_r, m_r, n_{11}, \dots, n_{ks_k})$ for the sign $+$, and with $g + k - 1$ for the sign $-$, writing a semicolon after the genus. The hypothesis that $\Gamma$ is proper stands in the opening of Section 4 and in the sentence before the theorem, not in its printed statement. Singerman has no Theorem 2.4.

## 6. Moduli

### 6.1. Deformation Dimension

[THURSTON2002] Corollary 13.3.7, p. 318 {N}, reads: "The Teichmuller space $\mathcal T(O)$ of an orbifold O with $\chi(O) < 0$ is homeomorphic to Euclidean space of dimension $-3\chi(X_O) + 2k + l$, where k is the number of elliptic points and l is the number of corner reflectors." The proof cuts $O$ into primitive pieces, and the parameters are the lengths and twists of closed geodesics and the lengths of arcs. The hypothesis $\chi(O) < 0$ excludes the Euclidean and spherical groups.

Two cases in [THURSTON2002] support the formula {N}. On p. 316, the disc `D^2( ; m1...ml)` is parametrized by $(\mathbb R_+)^{l-3}$. Theorem 13.8.1, p. 347, states that for an $n$-gonal orbifold $P$, $N(P) \cong D^{n-3}$ with interior $\mathcal T(P)$. [DAVIS2008] Example 6.5.3 {P} agrees that "If m > 3, there is a continuous family (moduli) of hyperbolic polygons with the same angles".

[SYMMETRIES-OF-THINGS2008] Chapter 18, p. 237 {P}, gives a rule that it attributes to Thurston: "the space of solutions, counted up to scale, is homeomorphic to a Euclidean space R^n ... namely, $\chi_3 = -n/3$, where $\chi_3$ is the orbifold Euler characteristic of the orbifold obtained from the given one by replacing each of its numerals by 3." Its worked example `4*235*74xx` has $n = 13$. Page 238 reads: "The argument doesn't work for the last four spherical cases, *NN, N*, Nx, and NN (where it would make the number of parameters negative!), and the last four Euclidean ones, **, *x, xx, and o, which have respectively one, one, one, and two parameters rather than the zero given by the rule." The book does not say whether these counts include the scale, and it does not state that the rule holds for the other Euclidean groups. For `*2222`, p. 41 reads: "Here the set of symmetries is no longer unique up to scale".

SymmHub computes $-6 + 6h + 3x + 2c + 3K + \sum \text{corners}$, with $h$ handles, $x$ cross-caps, $c$ cone points, and $K$ boundaries ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:270` {C}). It uses the number only in messages of the user interface. The research checked by substitution that with $\chi(X_O) = 2 - 2h - x - K$ this count equals Thurston's formula. The SymmHub tutorial cites Thurston Corollary 13.3.7 for it.

### 6.2. Fenchel-Nielsen Coordinates in Maskit

[MASKIT2001] Sections 2 and 3 {P} set up the coordinates. The scope reads: "We assume throughout that all orbifolds are complete, orientable, of dimension 2". Cone points and cusps are boundary elements of elliptic or parabolic type.

The signature is `(g, n, m; alpha_1, ..., alpha_n)`, where $n$ counts cusps and cone points, $m$ counts holes, and the orders are in decreasing order. There are $p = 3g - 3 + n + m$ coordinate geodesics and $q = 2g - 2 + n + m$ pants (Section 2.1). The coordinates are:

$$\Phi = (s_1, \dots, s_{p+m}, t_1, \dots, t_p) \in (\mathbb R^+)^{p+m} \times \mathbb R^p$$

The length of $L_i$ is $2\sigma_i$ with $s_i = \sinh\sigma_i$, and the twist about $L_i$ is $2\tau_i$ with $t_i = \sinh\tau_i$ (Section 2.2). "Following Wolpert, we choose the twist to be independent of the length of the geodesic we are twisting about."

### 6.3. Which Cuts Carry a Twist

Every closed two-sided geodesic of a cut system in [MASKIT2001] carries a twist {P}, and Maskit has no mirrors or cross-caps. For the other features, the statements are these:

- SymmHub's tutorial states that the twist appears in exactly three situations: "the cut around a handle, the cut between two order-2 cone points, and the cut around a pair of cone points" ([SYMMHUB] `tutorials/orbifold-notation.html` {C}). Its code sets `twistKeys = ["handle","conePair","tube"]` ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:46-53` {C}).
- A cross-cap has no twist, because "slicing a Mobius band down the middle leaves a single piece" ([SYMMHUB] tutorial {C}). [NORBURY2006] {S} agrees: "The gluing along a one-sided geodesic is rigid and hence does not allow rotations", and "a one-sided geodesic must glue to itself via the antipodal map". [GENDULPHE2017] {S} reads: "The twist flow is only defined along two-sided curves."
- A cut that ends on a mirror has no twist, because "the gluing must carry mirror to mirror, freezing the offset" ([SYMMHUB] tutorial {C}).

The claim that these are the only three situations rests only on the SymmHub tutorial, which attributes the rules to [DEATON1993] (gap G4).

### 6.4. Cone Points and Cusps

[MASKIT2001] Section 5 {P} treats cone points and cusps as boundaries of a pair of pants with an extended size: "three numbers $\lambda_1$, $\lambda_2$ and $\lambda_3$, where either $\lambda_i \ge 0$, or $\lambda_i = i\pi/\alpha$, $\alpha \in \mathbb Z$, $\alpha \ge 2$". A cusp has $\lambda = 0$. A cone point of order $\alpha$ has $\lambda = i\pi/\alpha$, so that $\cosh\lambda = \cos(\pi/\alpha)$. The required traces are $|\mathrm{tr}\,\tilde a_k| = 2\cosh\lambda_k$.

[DRYDEN-PARLIER2004] {S} states that "a boundary geodesic can be replaced by a cone point" under the hypothesis that "all cone angles are less than $\pi$", and gives collar formulas for trirectangles.

### 6.5. Mirrors and Cross-Caps

[CHOI-GOLDMAN2001] {S} cuts a 2-orbifold into twelve elementary orbifolds, named P1 to P4, A1 to A4, and D1 to D4. The hyperbolic deformation space of each piece equals the space of its boundary lengths, and twists arise only at gluing. Their Proposition 5.1 gives the dimension $-3\chi(X) + 2k + l + 2n$, with $n$ the number of boundary full 1-orbifolds. The research did not verify the dimensions of the A-type pieces.

[NORBURY2006] {S} gives the dimension $3k - 6 + 2n$ for non-orientable genus $k$. A one-sided class has determinant $-1$ and $\mathrm{tr} = 2\sinh\frac{l}{2}$. [NORBURY2006] also gives this right-angled hexagon formula:

$$\cosh y' = \frac{\cosh y + \cosh\frac{x}{2}\cosh z}{\sinh\frac{x}{2}\sinh z}$$

### 6.6. Decomposition into Elementary Pieces

Two decompositions into twelve kinds of piece appear in the sources. [CHOI-GOLDMAN2001] {S} uses twelve elementary orbifolds. SymmHub, after [DEATON1993], uses twelve atoms: four sheets `*abc`, four pillows `ABC`, and four hats `A*b` ([SYMMHUB] tutorial {C}). No source that was read states whether the two lists correspond (Section 13.3).

## 7. Orientable Building Blocks from Maskit

### 7.1. Scope

[MASKIT2001] gives explicit $SL(2,\mathbb R)$ matrices for every type of pair of pants, the conjugators that glue them with a twist, the maps that close handles, and an algorithm that assembles generators for any complete orientable hyperbolic orbifold from its Fenchel-Nielsen coordinates {P}. Section 6.2 gives the signature and the coordinates. The construction covers no mirrors and no cross-caps.

### 7.2. Normalization of a Pair of Pants

[MASKIT2001] Section 5.1 {P} places the axis $A_1$ on the imaginary axis, the common orthogonal $M_3$ on the unit circle, and the axis $A_2$ in the right half-plane.

### 7.3. Three Hyperbolic Boundaries

[MASKIT2001] Section 5.2, equation (1) {P}, defines $\mu > 0$ by:

$$\coth\mu = \frac{\cosh\lambda_1\cosh\lambda_2 + \cosh\lambda_3}{\sinh\lambda_1\sinh\lambda_2}$$

The generators are:

$$\tilde a_1 = \begin{pmatrix} e^{\lambda_1} & 0 \\ 0 & e^{-\lambda_1} \end{pmatrix}, \quad \tilde a_2 = \frac{1}{\sinh\mu}\begin{pmatrix} \sinh(\mu - \lambda_2) & \sinh\lambda_2 \\ -\sinh\lambda_2 & \sinh(\mu + \lambda_2) \end{pmatrix}$$

They satisfy $\mathrm{tr}(\tilde a_1\tilde a_2) = -2\cosh\lambda_3$. Remark 5.1 reads: "$\mu$ is related to $\delta$, the distance between $A_1$ and $A_2$, by $\coth\mu = \cosh\delta$ ... The RHS of equation (1) is the well-known formula for the hyperbolic cosine of the length of one side of a hexagon with all right angles, given the lengths of three non-adjacent sides."

Equation (2) defines $\nu$ and the third generator:

$$\coth\nu = \frac{\cosh\lambda_1\cosh\lambda_3 + \cosh\lambda_2}{\sinh\lambda_1\sinh\lambda_3}$$

$$\tilde a_3 = -\tilde a_2^{-1}\tilde a_1^{-1} = \frac{1}{\sinh\nu}\begin{pmatrix} \sinh(\nu - \lambda_3) & e^{\lambda_1}\sinh\lambda_3 \\ -e^{-\lambda_1}\sinh\lambda_3 & \sinh(\nu + \lambda_3) \end{pmatrix}$$

### 7.4. Two Hyperbolic Boundaries and One Cusp or Cone Point

[MASKIT2001] Section 5.4, equation (5) {P}, keeps $\tilde a_1$, $\tilde a_2$, and the formula for $\mu$ of Section 7.3 (Remark 5.2), with $\lambda_3 = 0$ or $\lambda_3 = i\pi/\alpha$.

### 7.5. One Hyperbolic Boundary

These cases come from [MASKIT2001] {P}:

- One hyperbolic boundary, one cusp, and a third boundary that is a cusp or a cone point (Section 5.6):

$$\tilde a_2 = \begin{pmatrix} 1 + \beta & -\beta \\ \beta & 1 - \beta \end{pmatrix}, \quad \beta = -\frac{\cosh\lambda_1 + \cosh\lambda_3}{\sinh\lambda_1}$$

- One hyperbolic boundary and two cusps (Section 5.7): $\tilde a_2 = \begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$, and the group is elementary.
- One hyperbolic boundary and two cone points (Section 5.8, equation (7)): the same form as equation (1). Remark 5.4 reads: "Here, $\coth\mu = -i\sinh\delta$ ... one side of a quadrilateral with two adjacent right angles".

The research recorded only $\tilde a_2$ for Sections 5.6 and 5.7. An implementer reads $\tilde a_1$ for these cases from [MASKIT2001] itself.

### 7.6. Triangle Groups

[MASKIT2001] Section 5.9 {P} gives the cases with no hyperbolic boundary:

- Two cusps and one cone point:

$$\tilde a_1 = \begin{pmatrix} 1 & 1 \\ 0 & 1 \end{pmatrix}, \quad \tilde a_2 = \begin{pmatrix} 1 & 0 \\ -2 - 2\cosh\lambda_3 & 1 \end{pmatrix}$$

- One cusp and two cone points:

$$\tilde a_1 = \begin{pmatrix} 1 & \frac{2\cosh\lambda_2 + 2\cosh\lambda_3}{-i\sinh\lambda_2} \\ 0 & 1 \end{pmatrix}, \quad \tilde a_2 = \begin{pmatrix} \cosh\lambda_2 & -i\sinh\lambda_2 \\ i\sinh\lambda_2 & \cosh\lambda_2 \end{pmatrix}$$

- Three cone points, in the unit disc, equation (8): $a_1(z) = e^{2\pi i/\alpha_1}z$ and

$$\tilde a_2 = \frac{1}{\sinh\mu}\begin{pmatrix} \sinh(\mu + \lambda_2) & -\sinh\lambda_2 \\ \sinh\lambda_2 & \sinh(\mu - \lambda_2) \end{pmatrix}, \quad \coth\mu = -\frac{\cosh\lambda_1\cosh\lambda_2 + \cosh\lambda_3}{\sinh\lambda_1\sinh\lambda_2}$$

For the three-cone case, "we can solve equation (8) for $\mu > 0$ if and only if the right-hand side is > 1, which occurs if and only if $\pi/\alpha_1 + \pi/\alpha_2 + \pi/\alpha_3 < \pi$." Remark 5.5 identifies equation (8) as a hyperbolic law of cosines.

### 7.7. Negative Trace Theorem

[MASKIT2001] Theorem 4.1 {P} gives a local discreteness test for each pair of pants. If the axes are disjoint, $a_1$ and $a_2$ are geometric generators of a pants group if and only if both conditions hold:

1. $T = \mathrm{tr}(\tilde a_1)\,\mathrm{tr}(\tilde a_2)\,\mathrm{tr}(\tilde a_1\tilde a_2) \le 0$.
2. "if any of $a_1$, $a_2$ or $a_1a_2$ is elliptic, then it is primitive."

If the axes are not disjoint, this happens only when "$a_1$ is hyperbolic and $a_2$ is elliptic of order 2".

### 7.8. One-Holed Torus and Torus with a Cusp or Cone Point

For the one-holed torus, [MASKIT2001] Section 5.3 {P} sets $\lambda_2 = \lambda_3$ and closes the handle with equation (4), $\tilde d = \tilde r\,\tilde r_2\,\tilde e_\tau$, where:

$$\tilde e_\tau = \frac{1}{\sinh\mu}\begin{pmatrix} \sinh(\mu - \tau) & \sinh\tau \\ -\sinh\tau & \sinh(\mu + \tau) \end{pmatrix}, \quad \tilde r = \begin{pmatrix} 0 & e^{\lambda_1/2} \\ e^{-\lambda_1/2} & 0 \end{pmatrix}, \quad \tilde r_2 = \frac{1}{\sinh\mu}\begin{pmatrix} -\cosh\mu & 1 \\ -1 & \cosh\mu \end{pmatrix}$$

For the torus with one cusp or one cone point, [MASKIT2001] Section 5.5 {P} makes $a_1$ parabolic or elliptic, and equation (6) defines $\mu$ by $\sinh^2\mu = \frac{2\sinh^2\lambda_2}{\cosh\lambda_1 + 1}$. The generators are:

$$\tilde a_{2,3} = \frac{1}{\sinh\mu}\begin{pmatrix} \sinh(\mu \pm \lambda_2) & \sinh\lambda_2 \\ -\sinh\lambda_2 & \sinh(\mu \mp \lambda_2) \end{pmatrix}$$

The handle closer is $d = r_0f_\tau r_2$, with $\tilde r_0 = \mathrm{diag}(1, -1)$, $\tilde f_\tau = \mathrm{diag}(e^{\tau}, e^{-\tau})$, and:

$$\tilde r_2 = \frac{1}{\sinh\mu}\begin{pmatrix} \cosh\mu & 1 \\ -1 & -\cosh\mu \end{pmatrix}$$

### 7.9. Gluing Pants and Closing Handles

[MASKIT2001] Section 6.5 {P} glues the pants one at a time. The conjugator $c_{k+1}$ maps the left half-plane onto the action half-plane of the matching axis, and moves the base point $i$ a distance $2\tau_k$ along that axis. The group $J_{k+1} = \langle J_k, c_{k+1}H_{k+1}c_{k+1}^{-1}\rangle$ is Fuchsian by the combination theorem for amalgamated free products. It satisfies the additional relation $a^k_{k+j} = (c_{k+1}a_{k+1,1}c_{k+1}^{-1})^{-1}$.

[MASKIT2001] Section 6.6 {P} closes each handle with a map "$d_1$ ... mapping the action half-plane of $a^0_q$ onto the boundary half-plane of $a^0_{q+1}$, while mapping the point at distance $-2\tau_q$ from the base point on $A^0_q$ to the base point on $A^0_{q+1}$." This uses the HNN combination theorem and adds the relation $a^0_{q+1} = d_1(a^0_q)^{-1}d_1^{-1}$.

### 7.10. Explicit Conjugators

[MASKIT2001] Section 7 {P} gives the conjugators in closed form. The universal twist is $\tilde f_\tau = \mathrm{diag}(e^{-\tau}, e^{\tau})$, which "twists by $-2\tau$", and the interchange is $\tilde g = \begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$. The other matrices are:

$$\tilde e_{2,1} = \begin{pmatrix} 0 & -e^{\tau_1} \\ e^{-\tau_1} & 0 \end{pmatrix}$$

$$\tilde e^0_{i,2} = \frac{1}{\sqrt{2\sinh\mu_i}}\begin{pmatrix} e^{\mu_i/2} & e^{-\mu_i/2} \\ e^{-\mu_i/2} & e^{\mu_i/2} \end{pmatrix} \quad \text{(16)}, \qquad \tilde e_{i,2} = \tilde e^0_{i,2}\tilde f_{\tau_j} \quad \text{(17)}$$

$$\tilde e^0_{i,3} = \frac{1}{\sqrt{2\sinh\nu_i}}\begin{pmatrix} e^{\nu_i/2 + \lambda_i} & e^{-\nu_i/2} \\ e^{-\nu_i/2} & e^{\nu_i/2 - \lambda_i} \end{pmatrix} \quad \text{(18)}, \qquad \tilde e_{i,3} = \tilde e^0_{i,3}\tilde f_{\tau_j} \quad \text{(19)}$$

The conjugators follow by equation (9), $\tilde c_{j+1} = \tilde c_i\tilde e_{i,k}$. A handle closer whose two sides lie in the same pair of pants is equation (10), $\tilde d_j = \tilde c_i\tilde d\tilde c_i^{-1}$. Otherwise it is equation (14):

$$\tilde d_j = \tilde c_{i'}\,\tilde e^0_{i',k'}\,\tilde g\,\tilde f_{\tau_j}\,(\tilde c_i\tilde e^0_{i,k})^{-1}$$

Remark 7.1 notes that square roots enter at equation (15).

### 7.11. Assembly Algorithm

[MASKIT2001] Section 8 {P} takes three inputs: the signature, a pairing table with $q$ rows, one per pair of pants, and 3 columns, one per boundary, each entry $b_{i,k}$ naming a coordinate geodesic $L_j$ or a boundary $b_j$, and the coordinates $\Phi$. It runs these steps:

1. Build each pants group from the matrices of Sections 7.3 to 7.6.
2. Set $c_1 = 1$ and $c_2 = e_{2,1}$, then compute the other conjugators by equation (9).
3. Form the attaching generators $\tilde a_1 = \tilde a_{1,1}$ and $\tilde a_j = \tilde c_{j+1}\tilde a_{j+1,1}^{-1}\tilde c_{j+1}^{-1}$.
4. Form the handle generators $\tilde c_i\tilde a_{i,k}\tilde c_i^{-1}$.
5. Form the handle closers by equation (10) or equation (14).
6. Form the boundary generators $\tilde c_i\tilde a_{i,k}\tilde c_i^{-1}$.

The output has $d = 2q + 1 = 4g - 3 + 2n + 2m$ generators, "in general far from minimal". Theorem 10.1 states that the algorithm "defines a canonical algebraic diffeomorphism of $DF(G_0)$ onto $(\mathbb R^+)^{m+p} \times \mathbb R^p$", where $DF(G_0)$ is the space that Maskit defines.

### 7.12. Variations, Examples, and the Inverse Map

[MASKIT2001] {P} also gives these:

- Worked examples for genus 3 (Section 11) and for genus 2 with the first variation (Section 13), including:

$$\tilde a_4 = \frac{1}{\sinh\mu}\begin{pmatrix} e^{\tau_1}\cosh(\mu - \tau_2) & -e^{\tau_1}\cosh\tau_2 \\ -e^{-\tau_1}\cosh\tau_2 & e^{-\tau_1}\cosh(\mu + \tau_2) \end{pmatrix}$$

- A second variation (Section 14), in which the untwisted handle closer is the square root of $h = r_{A'}r_A$. It preserves the common orthogonal but loses closed-form entries. Remark 6.1 gives the motivation.
- The inverse map (Section 9.2), which recovers the twists from the half-turn axes of the "Fenchel-Jorgensen commutator, $r' = aa' - a'a$".

### 7.13. Caveats

The printed text of [MASKIT2001] contains these inconsistencies {P}. Every section number in the list is Maskit's.

- Section 2 has $2\cosh\sigma = |\mathrm{tr}|$, but Section 9.2 prints "$\cosh\sigma_i = 2|\mathrm{tr}(\tilde a_i)|$".
- Section 10.1 prints $\Phi \in (\mathbb R^+)^{p+m} \times \mathbb R^m$, but Section 2.2 has $\mathbb R^p$.
- Equation (15) prints "$\coth_i\mu$".
- The Section 11 matrix for $\tilde e_{3,3}$ contains $\nu_2$ and a placement of $\sigma_2$ that does not match equation (19).

Where the printed text disagrees, the research recommends following Maskit's Section 2 and equations (16) to (19). The twist matrices that Maskit's Sections 5.5 and 7.2 both name $\tilde f_\tau$ have opposite signs (Section 2.5). Square roots enter the conjugators at equation (15), so the implementation fixes the sign of each root.

## 8. The SymmHub and Deaton Route

### 8.1. Status

SymmHub, by V. Bulatov, C. Goodman-Strauss, and S. Vorthmann, is the only implementation found that builds a fundamental domain and generators for hyperbolic signatures with mirrors and cross-caps. Its orbifold engine is by Goodman-Strauss. Every statement of this section about SymmHub is {C}. It describes what the code does. No primary source that was read establishes the method. The few statements from published sources that this section compares with the code, in Sections 8.4 and 8.7, carry their own markers. The tutorial attributes the cut rules to [DEATON1993], which was not read. `docs/research/symmhub.md` gives the line-level detail.

### 8.2. Pipeline

`lib/orbilib/OrbifoldGeometrization.js` runs six stages ([SYMMHUB] {C}):

1. `hashOrbifoldString` (line 77) parses the signature.
2. `hashOrbifold` (line 279) normalizes the features.
3. `atomizeOrbifold` (line 452) cuts the orbifold into atoms.
4. `unfoldAtom` (line 648) lays out each atom as a hyperbolic polygon.
5. `assembleFundamentalDomain` (line 1195) welds the atoms.
6. `produceGenerators` (line 1387) gives one generator per surviving edge.

This route handles hyperbolic signatures only. Spherical and Euclidean signatures go to the fixed catalogue in `lib/orbilib/WallpaperGroups_NonNegative.js` (lines 29 to 165 and 823), described in Sections 9.5 and 9.6.

### 8.3. Parser and Classification

The parser reads `o` or `O` as a handle, `x` or `X` as a cross-cap, and `*` as the start of a boundary. Digits before any `*`, or after `o` or `x`, are cone points, and digits after a `*` are its corners. An order of 10 or more is written in parentheses, such as `(12)`. Orders 0 and 1 are rejected, and a space is an error ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:77-170` {C}).

The parser rejects the empty signature, `*n` as "Single kaleidoscopic point -- not yet implemented" (line 191), `n` as "Single rotation point -- not yet implemented" (line 209), and the teardrops `pq` and `*pq` with $p \ne q$ as "Poorly formed orbifold". The classification computes `eulerchar = 2 - 2*handlesN - crosscapsN - kaleidoscopes.length - sum(1-1/n) - 0.5*sum(1-1/m)` and sets the curvature to 0 when `-0.001 < eulerchar < 0.01`. A comment notes that `*237` with $\chi = -1/84$ and `*235` with $\chi = 1/60$ bound these cut-offs (lines 226 to 250).

### 8.4. Normalization

`hashOrbifold` follows Deaton, according to its comments at lines 289 to 297 ([SYMMHUB] {C}). It applies these rules:

- It pairs cone points of order 2 into `conePair` entries. A single remaining cone point of order 2 becomes a `fold` on a mirror, or stays a `conePt` when there is no mirror.
- It turns each handle into two `["handle",i]` entries and each cross-cap into one `["cap",i]` entry.
- It turns every boundary after the first into a `band`, and stacks all corners on the first mirror.
- It pairs consecutive corners of order 2 into `cornerPair` entries.

The key lists are at lines 46 to 53:

- `keys = ["handle","cap","conePt","conePair","tube","mirror","mirrorRedundant","cornerPair","fold","band","slice","cuttingEdge","edge"]`
- `lengthKeys = ["handle","cap","fold","conePt","conePair","tube","band","slice","cornerPair"]`
- `twistKeys = ["handle","conePair","tube"]`

For a single kaleidoscope, `hashOrbifold` keeps the corners in the order of the input and replaces consecutive corners of order 2, paired greedily from the left and without wrapping around, by one corner pair each ([SYMMHUB], `docs/research/symmhub.md` Section 6.2.1 {C}). A corner pair becomes a mirror side of variable length between two right angles. [THURSTON2002] p. 316 {P} states a rule of this kind for the polygon orbifold: "Special care must be taken when, say, $m_1 = m_2 = 2$. Then one of the cuts must be omitted, and an edge length becomes a parameter." That SymmHub's corner pair is this edge length is a reader's reading (Section 13.3, item 12).

### 8.5. Cut Rules

`atomizeOrbifold` repeatedly splits the feature lists with `tube` cuts, from cone to cone with a length and a twist, and `slice` cuts, from a cone to a mirror or from corner to corner with a length only ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:452` {C}). The comments at lines 437 to 447 choose "option #2": cut the cone points out to the mirrors first, then cut the mirrors. That order needs few twist parameters and keeps the graph of cuts a tree.

The tutorial calls Deaton's rules "a small rewriting system on the notation itself", with "Nine types of geodesic cut", "each cut introducing a boundary token $\infty_\ell$ of length $\ell$" ([SYMMHUB] `tutorials/orbifold-notation.html` {C}). Its examples are $o \to \infty_\ell\infty_\ell$ and $ABCD \to \infty_\ell AB + \infty_\ell CD$ (`tutorials/orbifold-notation.html:243-245`), and `tutorials/orbifold-showcase.html:200` gives the example $*2345 \to *23\infty + *\infty 45$ {C}. The research did not recover five of the nine cut types, or the rules for cross-caps and cone pairs. It recovered the rules for corners from the code for a single kaleidoscope without cone points only (gap G4).

For a kaleidoscope without cone points, every cut is a slice between two mirror sides. While more than three features remain, the code cuts off the last two by a slice and puts the slice at the front of the list, and the last three features form the last sheet ([SYMMHUB], `docs/research/symmhub.md` Section 6.3.1 {C}). Its first cut of `*2345` separates the corners 5 and 4 from the corners 3 and 2, as the example of `tutorials/orbifold-showcase.html:200` does. Section 9.4.1 states the steps for an implementer.

### 8.6. The Twelve Atoms

The cuts end in "twelve types of atoms": four sheets `*abc`, four pillows `ABC`, and four pillowcases or hats `A*b` ([SYMMHUB] tutorial {C}). In each kind, the four types differ by how many vertices are real and how many are imaginary. A real vertex carries an angle `PI/n`. An imaginary vertex carries a length from `getLength(key)` and stands for a perpendicular geodesic edge of that length, so that a cut boundary becomes a truncated vertex. `unfoldAtom` has one case per atom type ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:648` {C}). A kaleidoscope without cone points produces sheets only, in the four cases A to D of `docs/research/symmhub.md` Section 6.4.1, with three, two, one, and no real vertices {C}.

### 8.7. Generalized Law of Cosines

`edgeLength(p,q,r)` computes the edge between two vertices of a generalized triangle ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:588` {C}). For each vertex $v$, the code sets $(c_v, s_v) = (\cos v, \sin v)$ when $v$ is a real angle and $(c_v, s_v) = (\cosh v, \sinh v)$ when $v$ is an imaginary value, a length. It then computes:

$$\ell\ell = \frac{c_pc_q + c_r}{s_ps_q}$$

It returns $\operatorname{arccosh}(\ell\ell)$ when $p$ and $q$ are both real or both imaginary, and $\operatorname{arcsinh}(\ell\ell)$ otherwise.

Maskit's equation (1) for three hyperbolic boundaries and equation (8) for three cone points have the same algebraic form, and [MASKIT2001] Remarks 5.1 and 5.5 identify them as the right-angled hexagon formula and a hyperbolic law of cosines {P}. [RATCLIFFE] and [BEARDON] state formulas for triangles, quadrilaterals, pentagons, and hexagons (Section 9.3). No primary source that was read states the cases that the code sends to $\operatorname{arcsinh}$, or that the other cases match those formulas (gap G3).

`docs/research/symmhub.md` Section 6.4.2 lists the calls of the four sheet cases with their branches {C}. A reader observed that each $\operatorname{arccosh}$ call of a sheet has the algebraic form of a published formula, with each real vertex read as an angle and each imaginary vertex read as the length of the side that stands for it. This table records that correspondence of form. It is a reader's observation, not a source's statement, and the labels of each source's figure were not matched against the layout of the sheet (gap G3, Section 13.3, item 1):

| Sheet case and call | Expression in the code | Published formula of the same form |
| --- | --- | --- |
| A, `R` and `P` | $\operatorname{arccosh} \dfrac{\cos p \cos q + \cos r}{\sin p \sin q}$ | The triangle with angles $\alpha$, $\beta$, $\gamma$: $\cosh c = \frac{\cos\alpha\cos\beta + \cos\gamma}{\sin\alpha\sin\beta}$, [RATCLIFFE] Theorem 3.5.4, p. 86, [BEARDON] Section 7.12, p. 148 {P} |
| B, `R` | $\operatorname{arccosh} \dfrac{\cos p \cos q + \cosh r}{\sin p \sin q}$ | The quadrilateral with two adjacent right angles, the side $c$ between the angles $\alpha$ and $\beta$, and the side $d$ between the right angles: $\cosh c = \frac{\cos\alpha\cos\beta + \cosh d}{\sin\alpha\sin\beta}$, [RATCLIFFE] Theorem 3.5.7, p. 92 {P} |
| C, `P` | $\operatorname{arccosh} \dfrac{\cosh q \cosh r + \cos p}{\sinh q \sinh r}$ | The pentagon with four right angles and the angle $\gamma$, with $c'$ the side opposite $\gamma$: $\cosh c' = \frac{\cosh a\cosh b + \cos\gamma}{\sinh a\sinh b}$, [RATCLIFFE] Theorem 3.5.11, p. 97 {P}, and [BEARDON] Theorem 7.18.1(i), p. 159, in other labels {P} |
| D, `R`, `P`, and `Q` | $\operatorname{arccosh} \dfrac{\cosh p \cosh q + \cosh r}{\sinh p \sinh q}$ | The right-angled hexagon: $\cosh c' = \frac{\cosh a\cosh b + \cosh c}{\sinh a\sinh b}$, [RATCLIFFE] Theorem 3.5.13, p. 100, and [BEARDON] Theorem 7.19.2, p. 161, in other labels {P} |
| B, `P` | $\operatorname{arsinh} \dfrac{\cos q \cosh r + \cos p}{\sin q \sinh r}$ | None found in the sources read |
| C, `R` | $\operatorname{arsinh} \dfrac{\cos p \cosh q + \cosh r}{\sin p \sinh q}$ | The pentagon with one angle that is not right, $\sinh \delta = \frac{\cosh c_i + \cos\alpha_i \cosh L_i'}{\sin\alpha_i \sinh L_i'}$, which `docs/research/geometry-models.md` Section 10.5 relays from a secondary source and leaves open in its Section 15.1, item 12 |

A port of the code compared every call of each case, the two $\operatorname{arsinh}$ calls included, with an independent construction in the hyperboloid model, and `docs/research/symmhub.md` Section 6.9 gives the largest difference for each case. This is numeric evidence from a port, and the formulas stay open (gap G3).

The first sheet of a kaleidoscope with $n \ge 4$ corners and no corner pair holds two corners and a slice, which is case B (`docs/research/symmhub.md` Section 6.3.1), so the code reading inferred that every such polygon uses the $\operatorname{arsinh}$ call of case B. By the code reading's hand trace, the signature `*222222` gives a single sheet of three corner pairs in case D, which uses no $\operatorname{arsinh}$ call. The port of `docs/research/symmhub.md` Section 6.9 agrees with the trace. Section 13.3, item 13, records what the construction needs beyond these formulas.

### 8.8. Pillow and Hat Formulas

A pillow is split into two generalized triangles by a "median". For three real cone points, the code computes ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:784` {C}):

- `A2 = asin(sqrt(1/(1+Math.pow(cot(p.re)+cos(q.re)/sin(p.re)/cos(r.re),2))))`
- `A1 = p.re-A2`
- `M = acosh(cos(q.re)/sin(A1))`
- `a1 = acosh(cos(A1)/sin(q.re))`
- `a2 = acosh(cos(A2)/sin(r.re))`

The imaginary analogues use `cosh(x/2)` (lines 829 to 833 and 880 to 884). For three imaginary vertices, line 936 computes `A2 = asinh(sqrt(1/(-1+Math.pow(coth(p.im/2)+cosh(q.im/2)/sinh(p.im/2)/cosh(r.im/2),2))))`. Hats use right-angled triangles with `HPI` and `PI/b[1]` (lines 1000 to 1130), and an imaginary cone uses the half length `q.im = getLength(b,group)/2`.

### 8.9. Layout and Walls

The vertices of each atom are placed in the Poincare disc by "turtle" moves, `poincareTurtleMove(last,current,angle,distance)` ([SYMMHUB] `lib/invlib/ComplexArithmetic.js:459` {C}). `useMirrorQ` (line 639 of `OrbifoldGeometrization.js`) marks which atom edges are real mirrors and which are `mirrorRedundant`. For the sheets of a kaleidoscope, `docs/research/symmhub.md` Section 6.4.1 gives each turtle move with its turn and length {C}. The interior angle at each vertex is the magnitude of the turn, and every sheet is counterclockwise, as the code reading inferred from the maps and a port of the code confirmed numerically (`docs/research/symmhub.md` Section 6.9).

A cutting edge of a pillow can carry a "wall", a perpendicular half-plane defined by two extra points (comments at lines 770 and 811 to 860). The assembled domain may therefore be non-convex, and the wall tells the GPU loop which generator to apply.

### 8.10. Assembly

`assembleFundamentalDomain` unfolds all atoms and welds them along open `tube` and `slice` attachments with `poincareMobiusEdgeToEdge([v1,v2],[w1,w2],shift)` ([SYMMHUB] `lib/invlib/ComplexArithmetic.js:472` {C}). For a tube, `shift` is the raw hyperbolic twist distance, applied as `transX = tanh(shift/2)`. Tube and slice edges vanish from the final polygon, a tube at lines 1301 to 1334 and a slice at lines 1335 to 1343 of `OrbifoldGeometrization.js`, with the splices themselves at lines 1332 to 1333 and 1340 to 1341.

After each weld, the code recentres the domain twice by moving the Euclidean mean of its vertices to the origin, "to keep the domain from growing off into a numerically less stable regime, closer to the boundary of the Poincare disk" (lines 1136 to 1138 and 1351 to 1365). After the last weld it applies the map that takes the point `(-.001,-.002)` to the origin, for general position (lines 1370 to 1371).

For a slice the `shift` is 0. The map sends the first end of the slice of the new sheet to the second end of the slice of the welded polygon and the reverse, and the code reading inferred from this that the sheet lands on the far side of the slice. The splice drops the two ends of the slice, the slice, and the edge after it, and keeps the edge before it in each polygon, which then runs on to the next vertex of the other polygon (`docs/research/symmhub.md` Section 6.5.1 {C}). The code reading inferred that the welded polygon has no vertex at the end of a slice, because both sheets have right angles there (Section 13.3, item 14).

### 8.11. Generators per Edge Type

`produceGenerators` gives one generator per surviving edge, written as an ordered list of reflections ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:1387` {C}):

| Edge key | Generator |
| --- | --- |
| `mirror`, `mirrorRedundant`, `cornerPair` | `[sPlaneReflectAcross(p,q)]`, a single reflection |
| `handle`, `cuttingEdge`, `band`, with paired edges $i$ and $j$ | `sPlanesMovingEdge1ToEdge2([p1,p2],[q2,q1])`, two reflections. For a handle, `p1` and `p2` first slide along the edge by `shift*getLength` (lines 1439 to 1445). |
| `cap` | With $e$ the edge length, $v = p + e/4$ and $w = p + e/2$ along $pq$, the list `[sPlaneSwapping(v,p), sPlaneSwapping(v,w), sPlaneReflectAcross(p,q)]` (line 1467). This is a glide reflection of length $e/2$. |
| `conePair` | A half-turn `sPlanesOfRotation(point,PI)` at the point $q + dd \cdot e$ along $qr$, with `dd = d<-.25? d+1: d<.25? d+.5: d`. Only one of the two cone points gives a generator. |
| `fold` | A half-turn at the midpoint of the edge |
| `conePt` | No generator. The vertex angle encodes the cone point. |

The code gives a `mirrorRedundant` edge a reflection like any other mirror and skips no edge. For a kaleidoscope, the splice of Section 8.10 drops the edge after each slice, so the generators are the $n$ reflections in the sides of the polygon, as the code reading inferred (`docs/research/symmhub.md` Section 8.1 {C}).

The helpers are in `lib/invlib/ComplexArithmetic.js` ([SYMMHUB] {C}). `sPlaneSwapping(pt1,pt2)` (line 644) is the circle of inversion centred at `[(b*(AA-1)-a*(BB-1))/DD, (B*(AA-1)-A*(BB-1))/DD]`, with `AA=|pt1|^2`, `BB=|pt2|^2`, and `DD=AA-BB`. It becomes a line when `|DD| < 1e-7`. `sPlanesMovingEdge1ToEdge2` (line 789) tries a single reflection first and otherwise uses two swaps. Its comment at line 787 reads "NOTE: this is not correct if p1=q1".

### 8.12. Representation of Transforms

A transform is a list of splanes applied in order, and its inverse is the reversed list ([SYMMHUB] `lib/invlib/ITransform.js:33`, `lib/grouplib/GroupUtils.js:164` {C}). Composition runs in the hyperboloid form of Section 2.5. `lib/invlib/MobiusMatrix.js` exists because the hyperboloid factorization "lost the half-turn trace to 5e-5" on long words (lines 10 to 17), so the code also classifies transforms by their $2 \times 2$ complex matrices.

### 8.13. Parameters and Their Ranges

The sliders are named `{key}_{i}_l` for lengths and `{key}_{i}_t` for twists ([SYMMHUB] `lib/orbilib/WallPaperGroup_General.js:26-29` {C}). Their ranges and defaults are these:

| Parameter | Range or value | Location |
| --- | --- | --- |
| Length | 0.3 to 4 | `lib/orbilib/WallPaperGroup_General.js:26-29` |
| Twist | -0.5 to 0.5, for `handle`, `conePair`, and `tube` only | `lib/orbilib/WallPaperGroup_General.js:26-29` |
| Initial length | `2 + 0.001*(random()-.5)` | `lib/orbilib/WallPaperGroup_General.js:266` |
| Length when a key index exceeds `parCounts` | 1.1 | `lib/orbilib/OrbifoldGeometrization.js:625-627` |
| Drag speed of a length | 0.005 | `lib/orbilib/OrbifoldUITool.js:61-69` |
| Drag speed of a twist | 0.002 | `lib/orbilib/OrbifoldUITool.js:61-69` |

The editing tool clamps a length to the range on a drag (`lib/orbilib/OrbifoldUITool.js:457`) and on a wheel step (line 487) {C}. A kaleidoscope has the parameters `slice_<i>_l` and `cornerPair_<k>_l` and no twist, and `docs/research/symmhub.md` Section 6.6.1 lists them for `*2223`, `*22222`, `*3333`, `*2345`, and `*23456` {C}.

The three twist conventions differ ({C}). A tube twist is a raw distance. A handle twist is a fraction of the edge length, applied as `shift*getLength`. A cone-pair twist is a fraction wrapped by the `dd` formula of Section 8.11. A cross-cap carries a length, the full edge length, and no twist.

### 8.14. Known Defects

The code and its notes record these defects that affect the construction ([SYMMHUB] {C}):

- `sPlanesMovingEdge1ToEdge2` is "not correct if p1=q1" (`lib/invlib/ComplexArithmetic.js:787`).
- `iGroup_2222_` is "wrong, missing one parameter" (`lib/orbilib/WallpaperGroups_NonNegative.js:386`).
- `apps/orbifold/bug list.txt` reads: "2*22 does not get the slice parameter correctly ... 22* as well."
- The signatures `n` and `*n` are not implemented, and no orbifold-specific unit tests were found.
- `willOrbifoldFitQ` always returns false (`lib/orbilib/OrbifoldGeometrization.js:1555`).
- The bounded reduction `iToFundDomainWBounds` assumes a convex domain, with the comment "for the moment, screw the bounds! Just assume we're convex" (`lib/invlib/Inversive.js:1245-1249`).
- `poincareMobiusFromSplane` assumes that the splane preserves the unit disc, with the comment "THIS WILL NOT WORK IN GENERAL" (`lib/invlib/ComplexArithmetic.js:820`).
- A reflection in a circle of very large radius loses precision, and the comment at `lib/invlib/ComplexArithmetic.js:519` advises keeping the construction in general position.
- The twist conventions are inconsistent, as Section 8.13 states.
- The swaps that put the corners of a sheet first reverse one sheet relative to the others, and the welded polygon then has the corner orders in another cyclic order (`lib/orbilib/OrbifoldGeometrization.js:538-543` and 553-567). `docs/research/symmhub.md` Section 6.8 states for which signatures this happens, with the counts of a port of the code.
- The pairing of corners of order 2 does not wrap around, so `*2233` and `*2332`, which name one orbifold, get different parameters (`docs/research/symmhub.md` Section 6.2.1).
- No sheet computes its last side or checks its closure, and no line checks that a sheet exists for its lengths (`docs/research/symmhub.md` Section 6.4.1). `docs/research/symmhub.md` Section 6.9 gives what a port of the code measured for lengths in 0.3 to 4, including the runs that failed the geometry check of the port near the boundary of the disc.

## 9. Explicit Simple Cases

### 9.1. Triangle Groups

The reflection triangle group `*pqr` has the presentation and conditions of Section 4.4 ([MASKIT2001] Section 4.4 {P}). Its rotation subgroup `pqr` has a pair of pants with cone points as its quotient, with generators $a_1 = r_2r_3$ and $a_2 = r_3r_1$. Section 7.6 gives matrices for every mix of cusps and cone points ([MASKIT2001] Section 5.9 {P}). The triangle is hyperbolic when $\pi/\alpha_1 + \pi/\alpha_2 + \pi/\alpha_3 < \pi$. [WIKI-TRIANGLE-GROUP] and [ARXIV-2105.01087] equations 34 and 35 {S} give the same presentation.

[DAVIS2008] Section 6.5 {P} lists the triangles by geometry, with integers $m_i \ge 2$:

| Geometry | Condition | Triples |
| --- | --- | --- |
| Spherical, Example 6.5.1 | $1/m_1 + 1/m_2 + 1/m_3 > 1$ | $(2,3,3)$, $(2,3,4)$, $(2,3,5)$, $(2,2,n)$ |
| Euclidean, Example 6.5.2 | $1/m_1 + 1/m_2 + 1/m_3 = 1$ | $(2,3,6)$, $(2,4,4)$, $(3,3,3)$, and also the rectangle |
| Hyperbolic, Example 6.5.3 | $\sum 1/m_i < m - 2$ for an $m$-gon | Infinitely many |

Example 6.5.3 reads: "Given any assignment of angles of the form $\pi/m_i$ to the vertices of a combinatorial m-gon so that $\sum_{i=1}^{m} 1/m_i < m - 2$, we can find a convex realization of it in $H^2$. By Theorem 6.4.3, this yields a corresponding reflection group on $H^2$." It continues: "Since a hyperbolic (or spherical) triangle is determined, up to congruence, by its angles, when m = 3 we get a discrete subgroup of $\mathrm{Isom}(H^2)$, well defined up to conjugation."

Example 6.5.1, p. 86 {P}, read on the page image, writes the spherical condition as $1/m_1 + 1/m_2 + 1/m_3 > 1$, equation (6.10), lists the triples $(2, 3, 3)$, $(2, 3, 4)$, $(2, 3, 5)$, and $(2, 2, n)$ with $n \ge 2$, and states: "By Theorem 6.4.3, each such triple corresponds to a spherical reflection group." It gives no separate statement that the triangle exists. [RATCLIFFE] states existence and congruence for spherical triangles only as exercises, Exercises 2.5.7 and 2.5.8, p. 53 (Section 11.3 of `docs/research/geometry-models.md`).

[RATCLIFFE] Section 7.2, p. 277 {P}, writes $T(a, b, c)$ for a triangle with the angles $\pi/a$, $\pi/b$, and $\pi/c$, for integers $2 \le a \le b \le c$, in a sphere, a Euclidean space, or a hyperbolic space $X$, and states: "Note that $T(a,b,c)$ is determined up to similarity in $X$ by the integers $a, b, c$." It denotes the group generated by the reflections in the sides of $T(a, b, c)$ by $G(a, b, c)$. Under the heading "Euclidean Triangle Reflection Groups", p. 279 {P}, it lists the three solutions of $1/a + 1/b + 1/c = 1$ and states: "$T(3,3,3)$ is an equilateral triangle, $T(2,4,4)$ is an isosceles right triangle, and $T(2,3,6)$ is a $30^\circ$-$60^\circ$ right triangle." No source read gives the coordinates of the vertices of a Euclidean triangle from its angles, and item 26 of Section 15.1 of `docs/research/geometry-models.md` records the candidate.

SymmHub places a triangle with angles `a0`, `a1`, and `a2` in the disc with these sides ([SYMMHUB] `lib/grouplib/Group_KLM.js:371-385` {C}):

- `iPlane([0,-1,0,0])`, the real axis.
- `iPlane([-sin(a0),cos(a0),0,0])`.
- The circle `iSphere([r*ax, r*ay, 0, -r])`, with `ax=(cos(a2)+cos(a1)*cos(a0))/sin(a0)`, `ay=cos(a1)`, and `r=1/sqrt(ax*ax+ay*ay-1)`.

The spherical case uses `r = e2*sin(a0)/(cos(a2)+cos(a1+a0))` (lines 340 to 346), and the Euclidean case uses `iPlane([sin(a1),cos(a1),0,e])` (line 355). The code chooses the geometry by the angle sum with `EPSILON=1e-6` (lines 296 to 310).

### 9.2. Regular Tilings and Polygons

A regular tiling $\{n, k\}$ is hyperbolic when "$2\pi n/k < (n - 2)\pi \iff 1/n + 1/k < 1/2$" ([SOLOMYAK2020] slide 8 {P}). The distance from the centre to a vertex of the regular polygon in the disc is $r_0 = \sqrt{\cos(\pi/p + \pi/q)/\cos(\pi/p - \pi/q)}$ for the tiling $\{p, q\}$ ([ARXIV-2105.01087] equation 9, [ARXIV-2309.10844] equation 25 {S}).

### 9.3. Trigonometric Formulas

`docs/research/geometry-models.md` covers hyperbolic trigonometry. These formulas appear in the construction:

- The angle form of the law of cosines, $\cosh c = \frac{\cos\alpha\cos\beta + \cos\gamma}{\sin\alpha\sin\beta}$: [MASKIT2001] equation (8) and Remark 5.5, [RATCLIFFE] Theorem 3.5.4, p. 86, [BEARDON] Section 7.12, p. 148 {P}.
- The side form $\cosh c = \cosh a\cosh b - \sinh a\sinh b\cos\gamma$: [BEARDON] Section 7.12, p. 148 {P}, [ARXIV-1410.6735] {S}.
- A triangle with angles $\alpha$, $\beta$, and 0, and finite side $c$: $\cosh c = \frac{1 + \cos\alpha\cos\beta}{\sin\alpha\sin\beta}$ ([RATCLIFFE] Theorem 3.5.6, p. 92, [BEARDON] Theorem 7.10.1, p. 146 {P}).
- A quadrilateral with two right angles: $\cosh c = \frac{\cos\alpha\cos\beta + \cosh d}{\sin\alpha\sin\beta}$ ([RATCLIFFE] Theorem 3.5.7, p. 92 {P}). The book's figure defines the labels.
- The Lambert quadrilateral: $\sinh a_1\sinh a_2 = \cos\phi$, where $a_1$ and $a_2$ meet at the right angle opposite the angle $\phi$ ([BEARDON] Theorem 7.17.1, p. 157 {P}). Wikipedia {S}, page not recorded, gives the same relation as $\cos\angle F = \sinh OA\,\sinh OB$.
- A pentagon with four right angles and the angle $\gamma$: $\cosh c' = \frac{\cosh a\cosh b + \cos\gamma}{\sinh a\sinh b}$ ([RATCLIFFE] Theorem 3.5.11, p. 97 {P}). [BEARDON] Theorem 7.18.1, p. 159 {P}, gives $\cosh a\cosh c + \cos\phi = \sinh a\cosh b\sinh c$ for a pentagon with four right angles, where $a$ and $c$ are the sides adjacent to $b$ and $\phi$ is the vertex opposite $b$.
- The right-angled hexagon formula: [MASKIT2001] equation (1) and Remark 5.1, [RATCLIFFE] Theorem 3.5.13, p. 100, $\cosh c' = \frac{\cosh a\cosh b + \cosh c}{\sinh a\sinh b}$, and [BEARDON] Theorem 7.19.2, p. 161 {P}.
- The spherical triangle: the two laws of cosines, the angle sum, and the areas of a triangle and of a lune, [RATCLIFFE] Theorems 2.5.1, 2.5.3, 2.5.4, and 2.5.5, pp. 50 to 51 {P}, which Section 11.3 of `docs/research/geometry-models.md` states.

### 9.4. Polygon Reflection Groups

The signatures `*p1...pn` have $n - 3$ free parameters ([THURSTON2002] Theorem 13.8.1 {N}). [THURSTON2002] p. 316 {P}, read in the text layer, states that the polygon orbifold with corner reflectors of orders $m_1, \dots, m_l$ "also can be decomposed into 'generalized triangles'", and that "the orbifold has hyperbolic structures (provided $\chi < 0$) parametrized by the lengths of the cuts, that is, $(\mathbb R^+)^{l-3}$", where this document writes a comma for the source's semicolon. Its Corollary 13.3.7, p. 318 {P}, gives the dimension $-3\chi(X_O) + 2k + l$ of the Teichmuller space, and a reader's substitution of $\chi(X_O) = 1$ and $k = 0$ for a disc with $l$ corner reflectors gives $l - 3$. [DAVIS2008] Example 6.5.3 {P} guarantees a convex realization in the hyperbolic plane whenever $\sum 1/m_i < m - 2$, and states that for $m > 3$ "there is a continuous family (moduli) of hyperbolic polygons with the same angles". No source read gives the vertices of the members of the family. SymmHub computes them by cutting the polygon with slices into sheets, with one length per slice and one per corner pair (Section 9.4.1).

[BEARDON] Theorem 7.16.2, p. 155 {P}, reads: "Let $\theta_1, \dots, \theta_n$ be any ordered $n$-tuple with $0 \le \theta_j < \pi$, $j = 1, \dots, n$. Then there exists a polygon $P$ with interior angles $\theta_1, \dots, \theta_n$, occurring in this order around $\partial P$, if and only if $\theta_1 + \dots + \theta_n < (n - 2)\pi$." The proof, pp. 155 to 156, builds one convex polygon with an inscribed disc from $n$ quadrilaterals $Q_j$. Each $Q_j$ has a vertex at the origin, two sides of length $d$ from the origin that end in right angles, and the angle $\theta_j$ at its far vertex. The angle of $Q_j$ at the origin is $2\alpha_j$, with $\sin\alpha_j = \cos(\theta_j/2)/\cosh d$, (7.16.3), and the quadrilaterals close up when $\sum_j\alpha_j = \pi$, (7.16.2). The proof finds $d$ by the intermediate value theorem. The theorem states existence only, and the proof gives one member of the family. For a convex polygon with angles $\pi/m_j$, [RATCLIFFE] Theorem 13.5.2 (Section 4.10) applies to the reflections in its sides.

#### 9.4.1. The Sheet and Slice Construction

This subsection states SymmHub's construction of the polygon of `*p1...pn`, with $n \ge 4$ corners and without cone points, handles, or cross-caps, as steps for an implementer. Every step is {C}: it states what the code does and does not establish that the method is correct. `docs/research/symmhub.md` gives the line-level reading of each step, and its Section 6.8 traces the examples `*2223`, `*22222`, `*3333`, `*2345`, `*23456`, and `*45322`.

1. Read the corners in the order of the signature. Replace each two consecutive corners of order 2, paired greedily from the left and without wrapping around, by one corner pair, a feature that stands for a mirror side of free length $\ell$ between two right angles (`docs/research/symmhub.md` Section 6.2.1).
2. While more than three features remain, remove the last two, record them with a new slice as a sheet, and put the slice at the front of the list (`docs/research/symmhub.md` Section 6.3.1). The slice is a feature that stands for a geodesic segment of free length $\ell$, laid out in step 4 with a right angle at each end. That it is the common perpendicular of the two mirror sides that it joins is a reader's inference (Section 13.3, item 14).
3. Record the last three features as the last sheet.
4. Lay out each sheet as a generalized triangle whose vertices are its three features, with the corners first. A corner of order $m$ is a vertex with the angle $\pi/m$. A corner pair or a slice of length $\ell$ is a side of length $\ell$ with a right angle at each end. The side between two features has the length of Section 8.7, and the sheet is counterclockwise (`docs/research/symmhub.md` Sections 6.4.1 and 6.4.2). The code puts the corners first by exchanging two features, which causes the defect of Section 8.14. In a port of the code, a variant that puts the corners first by a cyclic rotation of the three features, never by an exchange, gave no wrong corner order and no failed geometry check. It ran 47,641 corner sequences with four to eight corners, as typed and not reduced to canonical form, each with every length equal to 2 (`docs/research/symmhub.md` Sections 6.8 and 6.9). This is numeric evidence from a port, not a proof (Section 13.3, item 13).
5. Weld the sheets along their slices, starting from the first sheet. For each slice, apply to the other sheet that holds that slice the orientation-preserving isometry that maps each end of its slice to the opposite end of the same slice in the welded polygon. Drop the slice, its ends, and the side after the slice in each polygon, and keep the side before it, which then runs on to the next vertex of the other polygon (`docs/research/symmhub.md` Section 6.5.1). That the side kept and the side dropped at each end lie on one geodesic is open (Section 13.3, item 14).
6. Move the Euclidean mean of the vertices of the first sheet to the origin of the disc before any weld, do the same twice after each weld, and after the last weld apply the map that takes the point $(-0.001, -0.002)$ to the origin (`docs/research/symmhub.md` Section 6.5).
7. Take the reflection in each side of the result as a generator (Section 8.11).

A signature with $k$ corner pairs gives $n - k - 3$ slices and $n - k - 2$ sheets, and $n - 3$ lengths in all. As coded, step 4 can reverse the order of the features of one sheet, which changes the cyclic order of the corners of the result for the signatures that `docs/research/symmhub.md` Section 6.8 lists (Section 8.14). Step 1 gives one orbifold two parameter sets when its corners of order 2 meet across the end of the signature, as `*2233` and `*2332` do. The code checks neither the closure of a sheet nor the existence of the polygon for given lengths (Section 13.3, item 13).

#### 9.4.2. What the Published Sources Establish

The published sources establish these statements independently of SymmHub:

| Statement | Source | Limit |
| --- | --- | --- |
| A convex polygon with the angles $\pi/m_j$ in a given order exists when $\sum 1/m_j < n - 2$ | [DAVIS2008] Example 6.5.3, p. 87, and [BEARDON] Theorem 7.16.2, p. 155 {P}, which admits any ordered angles $0 \le \theta_j < \pi$ with $\sum \theta_j < (n - 2)\pi$ | Existence only |
| The reflections in the sides of such a polygon generate a discrete group, with the polygon as a strict fundamental domain | [DAVIS2008] Theorem 6.4.3 {P} (Section 4.5) | Applies to each polygon built, and gives no coordinates |
| One polygon for each ordered set of angles, with an inscribed disc | The proof of [BEARDON] Theorem 7.16.2, pp. 155 to 156 {P} | One member of the family, with the distance $d$ found by the intermediate value theorem |
| The structures of the polygon orbifold are parametrized by the lengths of the cuts into generalized triangles, $(\mathbb R^+)^{l-3}$, and one cut is omitted where two corners of order 2 meet, with an edge length as a parameter | [THURSTON2002] p. 316 {P} | The cut pattern of its figure was not compared with SymmHub's order of cuts. The text gives no formula for a vertex |
| The deformation space has dimension $l - 3$ for a disc with $l$ corner reflectors | [THURSTON2002] Corollary 13.3.7, p. 318 {P}, with the reader's substitution of Section 9.4 | Dimension only |
| A right-angled convex hexagon exists for any positive lengths of three alternate sides, and is unique up to congruence | [RATCLIFFE] Theorem 3.5.14, p. 100 {P} | The theorem states both. The book proves existence and leaves uniqueness as an exercise. That this covers every sheet of case D is a reader's reading |
| A right-angled $m$-gon exists if and only if $m \ge 5$ | [DAVIS2008] Corollary 6.11.7, p. 115, and the corollary of [BEARDON] Theorem 7.16.2, p. 156 {P} | Covers the polygon of `*22222` and its relatives |

Among the sources read, only SymmHub gives vertices for members of the family. It also gives an order of the cuts, the range 0.3 to 4 of the lengths, and the default length 2. All of these rest on code (gaps G3 and G4, Section 13.3, items 12 to 14).

#### 9.4.3. Conventions of SymmHub's Construction

These are facts about SymmHub, recorded so that a port does not carry a convention over by accident. Escher's own conventions are in its design documents.

| Topic | SymmHub | Source |
| --- | --- | --- |
| Model | The Poincare disc, with points as complex numbers and isometries as Mobius maps that preserve the disc | `docs/research/symmhub.md` Sections 5.4 and 6.4.1 {C} |
| Generators | A reflection in a circle or line of the disc, as one splane for each side | `docs/research/symmhub.md` Section 8.1 {C} |
| Polygon lists | A vertex list and an edge list, with edge $i$ from vertex $i$ to vertex $i + 1$ | `docs/research/symmhub.md` Section 2.4, item 8 {C} |
| Orientation and first vertex | Counterclockwise, with a first vertex that depends on the order of the welds, and the corners read clockwise in the order of the signature where no defect applies | `docs/research/symmhub.md` Sections 6.5.1 and 6.8 {C} |
| Parameter names | `slice_<i>_l` and `cornerPair_<k>_l`, numbered in the order of the cuts and of the pairs | `docs/research/symmhub.md` Section 6.6.1 {C} |
| Corner order of the input | Kept as typed, without a canonical rotation | `docs/research/symmhub.md` Section 6.2.1 {C} |

### 9.5. The 17 Euclidean Groups

The signatures are verified ([CONWAY-HUSON2002] Table II, [THURSTON2002] Theorem 13.3.6 {N}). The crystallographic (IUC) names are those of [SYMMETRIES-OF-THINGS2008] Table A.1, p. 416 {P}, where a bracket aligns `p3m1` with `*333` and `p31m` with `3*3`. [MACBEATH1967] Table I disagrees on that pair (Section 12). The SymmHub catalogue builds each group from a function in `lib/orbilib/WallpaperGroups_NonNegative.js` ([SYMMHUB] {C}). The free parameters in the last column come from [VONGAGERN2009] p. 7 {P}.

| IUC name | Signature | SymmHub function | Free parameters in [VONGAGERN2009] |
| --- | --- | --- | --- |
| p1 | `o` | `iGroup_O(a,b)`, with basis `[1,0]`, `[1+twist, length]` (line 852) | Excluded from the paper |
| p2 | `2222` | `iGroup_2222(a,b,c)`, with `c` the shear | "two geometric parameters" |
| pm | `**` | `iGroup_SS(a,b)` | Excluded from the paper |
| pg | `xx` | `iGroup_XX(a,b)` | Excluded from the paper |
| cm | `*x` | `iGroup_SX(a,b)` | Excluded from the paper |
| pmm | `*2222` | `iGroup_S2222(a,b)` | "a one parameter family of conjugacy classes" |
| pmg | `22*` | `iGroup_22S(a,b)` | "a one parameter family of geometric conjugacy classes" |
| pgg | `22x` | `iGroup_22X(a,b)` | "a one parameter family of geometric conjugacy classes" |
| cmm | `2*22` | `iGroup_2S22(a,b)` | "a one parameter family of geometric conjugacy classes" |
| p4 | `442` | `iGroup_442(a)` | Not stated |
| p4m | `*442` | `iGroup_S442(a)` | Not stated |
| p4g | `4*2` | `iGroup_4S2(a)` | Not stated |
| p3 | `333` | `iGroup_333(a)` | Not stated |
| p3m1 | `*333` | `iGroup_S333(a)` | Not stated |
| p31m | `3*3` | `iGroup_3S3(a)` | Not stated |
| p6 | `632` | `iGroup_632(a)` | Not stated |
| p6m | `*632` | `iGroup_S632(a)` | Not stated |

[VONGAGERN2009] Section 6.4, p. 28, excludes p1, pm, pg, and cm because "They do not have any rotation centers." The square and hexagonal groups are rigid in SymmHub, with hard-coded positions such as `d = a*sqrt(2.)/4.` (line 202). [CONWAY-HUSON2002] {N} states that each Euclidean signature names a single group "up to isotopy (i.e., continuous variation)". [SYMMETRIES-OF-THINGS2008] pp. 237 to 238 {P} gives a rule for the number of parameters and the counts for `**`, `*x`, `xx`, and `o` (Section 6.1). No primary source states a deformation dimension for every Euclidean group (gap G7).

[VONGAGERN2009] p. 7 {P} also relates the groups to reflection groups: "The groups p4m, p3m1 and p6m correspond to the three Euclidean triangular kaleidoscopic fundamental regions with corner angles (90, 45, 45), (60, 60, 60) and (90, 60, 30), respectively. The groups p4, p3 and p6 are the corresponding reflection free subgroups." The groups p31m and p4g are index-two subgroups of p6m and p4m. The rectangle group pmm "corresponds to a rectangular kaleidoscope with four 90 corners."

[VONGAGERN2009] p. 12 {P} lists fundamental regions built from triangles that carry over to hyperbolic relatives when the orders change:

| Region | Groups | Shape and angle condition |
| --- | --- | --- |
| Reflected triangle | p6m, p3m1, p4m | Angles $\pi/k$, $\pi/m$, $\pi/n$. "Any integral values that fulfill the angle sum inequality for a hyperbolic triangle are valid." |
| Rotated kite | p6, p3, p4 | A deltoid with two equal angles $\pi/k$ and the angles $2\pi/m$ and $2\pi/n$ on its axis |
| Reflected kite | p31m, p4g | The symmetric angle is a right angle, with the angles $\pi/m$ and $2\pi/n$ on the axis. "The angle sum inequality has to be fulfilled for one half of the kite." |

The same page states that "the fundamental cells of the groups p3, p4, p6, p31m, p4g consist of two copies of the triangles for p3m1, p4m or p6m." For pmm, p. 14 states that "every corner angle must be a divisor of $\pi$", and that the hyperbolic length of one side together with the corner angles determines the tile. The hyperbolic triangle groups of p. 8 are those with angles $\pi/k$, $\pi/m$, $\pi/n$ and $1/k + 1/m + 1/n < 1$.

### 9.6. The 14 Spherical Families

The signatures and orders are verified ([CONWAY-HUSON2002] Table III, [THURSTON2002] Theorem 13.3.6 {N}), with order $2/\chi$. The Schoenflies names are those of [SYMMETRIES-OF-THINGS2008] Table A.2, p. 416 {P}.

| Signature | Order | Schoenflies name |
| --- | --- | --- |
| `nn` | $n$ | $C_n$ |
| `*nn` | $2n$ | $C_{nv}$ |
| `n*` | $2n$ | $C_{nh}$ |
| `nx` | $2n$ | $S_{2n}$ |
| `22n` | $2n$ | $D_n$ |
| `*22n` | $4n$ | $D_{nh}$ |
| `2*n` | $4n$ | $D_{nd}$ |
| `332` | 12 | $T$ |
| `*332` | 24 | $T_d$ |
| `3*2` | 24 | $T_h$ |
| `432` | 24 | $O$ |
| `*432` | 48 | $O_h$ |
| `532` | 60 | $I$ |
| `*532` | 120 | $I_h$ |

Generators exist only as SymmHub catalogue code, in the stereographic plane with the equator at the unit circle and fixed constants, such as `iSphere([-1.61803398874989,0,0,1.90211303259031])` for `532` ([SYMMHUB] `lib/orbilib/WallpaperGroups_NonNegative.js:711` {C}). The reflection triangles of the spherical reflection groups are those of Section 9.1, and [DAVIS2008] Theorem 6.4.3 {P} certifies them as strict fundamental domains. `*nn` has two corners and no triangle. Its candidate domain is the lune with the angle $\pi/n$, which no source read certifies (Section 4.5 and item 9 of Section 13.3).

### 9.7. Mixed Signatures

[CONWAY-HUSON2002] {N} gives the presentation of every mixed signature, such as `p*q` and `2*pq` (Section 5.2). SymmHub builds the geometry with the hat atom `A*b` ([SYMMHUB] {C}). [SYMMETRIES-OF-THINGS2008] pp. 125 to 131 {P} gives the presentations of the Euclidean and spherical mixed groups, and p. 237 describes Thurston's decomposition into "triangles" with real and imaginary angles without a formula. [RATCLIFFE] Example 1 of Section 13.4, pp. 700 to 701 {P}, is a worked mixed case: a triangle with angles $\alpha$, $\alpha$, and $2\pi/3$ at $x$, $y$, and $z$, whose side $[x, z]$ is paired with $[y, z]$ by the rotation of $2\pi/3$ about $z$, and whose side $[x, y]$ is paired with itself by the reflection in it. The dihedral ridge cycle $\{x, y\}$ gives $2\alpha = \pi/k$, and Example 1 of Section 13.5, p. 711, gives the presentation `<L, R, S ; LR, S^2, R^3, (RS)^(2k)>`, written in a code span because the book uses a semicolon. No source read gives a construction from the signature for mixed signatures in general (gap G8).

### 9.8. Signatures with Cusps

[RATCLIFFE] and [BEARDON] {P} give theorems and worked examples for groups with cusps. The theorems are those of Sections 4.10 and 4.11, the existence condition and area of Section 5.5, and [RATCLIFFE] Theorems 7.1.3 and 13.5.2 for reflection polygons with ideal vertices. The worked examples in [RATCLIFFE] {P} are these:

- The modular group, pp. 247 and 259: the Dirichlet polygon with centre $ti$, $t > 1$, is the triangle with vertices $\pm\frac12 + \frac{\sqrt3}{2}i$ and $\infty$, which Exercise 6.6.3 asks the reader to prove, with side pairings $g_R(z) = z + 1$ and $g_S(z) = -1/z$, and "The relations $(g_Sg_R)^3 = 1$ and $g_S^2 = 1$ form a complete set of relations."
- The once-punctured torus, Example 1 of Section 9.8, p. 417: the ideal square with vertices $\pm e_1$ and $\pm e_2$ in the disc, whose opposite sides are paired "by first reflecting in the lines $y = \pm x$ and then reflecting in the corresponding side of $P$".
- The thrice-punctured sphere, Example 2 of Section 9.8, p. 425, with the maps $f(z) = z + 2$ and $g(z) = \frac{z}{-2z + 1}$ in the proof of Theorem 9.8.8, pp. 426 to 427.
- A cone point, a mirror, and a cusp on the mirror, Example 6 of Section 13.4, p. 706: a generalized triangle with angles 0, 0, and $2\pi/3$ at $x$, $y$, and $z$, with the side pairings of the example in Section 9.7. The cusp points $x$ and $y$ form one cusp point of the orbifold, whose link is complete by Theorem 13.4.4, so the orbifold is complete by Theorem 13.4.6.
- The reflection group of the triangle $T(2, 4, \infty)$, p. 299, which contains the reflection with the Lorentz matrix $A = \begin{pmatrix} -1 & -2 & 2 \\ -2 & -1 & 2 \\ -2 & -2 & 3 \end{pmatrix}$ that takes $e_3$ to $(2, 2, 3)$, with $A = A^{-1} = (JAJ)^t$ and $J = \mathrm{diag}(1, 1, -1)$.

[BEARDON] Theorem 10.4.2 builds one polygon from the signature of an orientation-preserving group with cusps (Section 5.5). No source read gives a general construction of a polygon with side pairings from a signature that mixes cusps with mirrors or cross-caps. [SINGERMAN1974] and [MACBEATH1967] give only the combinatorial edge word (Section 5.6) and exclude non-compact groups by definition.

## 10. Algorithms

### 10.1. Dirichlet Domains

[VOIGHT2009] Section 1 {P} defines the Dirichlet domain $D(p) = \{z \in \mathbb H : d(z, p) \le d(gz, p) \text{ for all } g \in \Gamma\}$, "a hyperbolically convex fundamental domain". A "random" point $p$ has trivial stabilizer "with probability 1". The map $\phi(z) = \frac{z - p}{z - \bar p}$ moves $p$ to $0$ and $\Gamma$ into $PSU(1,1)$.

Proposition 1.3 states that $D(0)$ is the closure of $\bigcap_{g \ne 1}\mathrm{ext}(I(g))$, and that $d(z, 0)$ is less than, equal to, or greater than $d(gz, 0)$ as $z$ lies in $\mathrm{ext}(I(g))$, on $I(g)$, or in $\mathrm{int}(I(g))$. Corollary 1.4 states that $gI(g) = I(g^{-1})$.

Remark 1.5 gives the bisector for the base point $p = i$ as the half-circle "of radius $(a^2 + b^2 + c^2 + d^2 - 2)/(a^2 + c^2 - 1)^2$ centered at $(ab + cd)/(a^2 + c^2 - 1)$". That expression is as printed, and Section 12 records a doubt about it (gap G9).

[RATCLIFFE] p. 243 {P} defines the Dirichlet domain $D(u) = \bigcap_{g \ne 1} H_g(u)$, with $H_g(u) = \{x : d(x, u) < d(x, gu)\}$, for a centre $u$ whose stabilizer $\Gamma_u$ is trivial. Theorem 6.5.16, p. 245, states that $D(u)$ is the set of points $x$ that are a nearest point of their orbit $\Gamma x$ to $u$. Theorem 6.6.2, p. 248, states that each side $S$ of $D(u)$ lies on the bisector of $u$ and $gu$ for a unique $g \ne 1$, with $S = D(u) \cap gD(u)$, and that $g^{-1}S$ is a side of $D(u)$. Exercise 7.1.1, p. 276, asks the reader to prove that the polyhedron of a discrete reflection group is its Dirichlet polyhedron with centre any interior point.

[DAVIS2008] Complement 6.6.10 {P} notes that "a Dirichlet domain is a fundamental domain ... however, there is no reason for it to be a strict fundamental domain."

SymmHub computes Dirichlet domains with the base point on the plane, so that each side is an Apollonius sphere $|x - T(a)|^2 = |T'(a)|\,|x - a|^2$ ([SYMMHUB] `lib/grouplib/DirichletDomain.js:1-60` {C}).

### 10.2. Normalized Boundary

[VOIGHT2009] Definition 2.3 {P} defines the normalized boundary $U = g_1, \dots, g_n$ as the sequence of isometric circles that hold the consecutive sides in counter-clockwise order, starting at the vertex with the smallest argument in $(0, 2\pi)$. Algorithm 2.5 computes it by sweeping the angle from 0, and the paper proves it correct.

### 10.3. Point Reduction in Voight

[VOIGHT2009] Section 4 {P} sets $\rho(\gamma, z) = d(\gamma z, 0)$. Definition 4.1 reads: "Let $z \in \mathfrak D$. An element $\gamma \in \Gamma$ is $(G, z)$-reduced if for all $g \in G$, we have $\rho(\gamma, z) \le \rho(g\gamma, z)$, and $\gamma$ is G-reduced if it is $(G, 0)$-reduced." Remark 4.2 reads: "$\gamma$ is $(G, z)$-reduced if and only if $\gamma z \in \mathrm{ext}(G)$."

Algorithm 4.3 reads: "Let $\gamma \in \Gamma$ and let $z \in \mathfrak D$. This algorithm returns elements $\bar\gamma \in \Gamma$ and $\delta \in \langle G\rangle$ such that $\bar\gamma$ is $(G, z)$-reduced and $\bar\gamma = \delta\gamma$."

1. "Initialize $\bar\gamma := \gamma$ and $\delta := 1$."
2. "If $\rho(\bar\gamma, z) \le \rho(g\bar\gamma, z)$ for all $g \in G$, return $\bar\gamma$, $\delta$. Otherwise, let g in G be the first element in G such that $\rho(g\bar\gamma, z) = \min_i \rho(g_i\bar\gamma, z)$. Let $\bar\gamma := g_i\bar\gamma$ and $\delta := g_i\delta$, and return to Step 2."

The termination argument reads: "The algorithm terminates because if $\bar\gamma_1, \bar\gamma_2, \dots$ are the elements that arise in the iteration of Step 2, then $\rho(\bar\gamma_1, z) > \rho(\bar\gamma_2, z) > \dots$, however, the action of $\Gamma$ is discrete, so among the points $\{\bar\gamma_i(z)\}_i$, only finitely many are distinct." In the original, a semicolon precedes "however".

To reduce a point, the algorithm runs with $\gamma = 1$. By Remark 4.2 the output $\bar\gamma z$ lies in $\mathrm{ext}(G)$. The introduction states the corollary that it "returns a point $z' \in D(p)$ and $g \in \Gamma$ such that $z' = g(z)$".

### 10.4. Independence of Order and Bases

[VOIGHT2009] Proposition 4.4 {P} reads: "Suppose that $\mathrm{ext}(G)$ is a fundamental domain for $\langle G\rangle$. Then for almost all $z \in \mathfrak D$, $\mathrm{red}_G(\gamma, z)$ as an element of $\Gamma$ is independent of the ordering of G for all $\gamma \in \langle G\rangle$. Moreover, for all $\gamma \in \Gamma$, we have $\mathrm{red}_G(\gamma) = 1$ if and only if $\gamma \in \langle G\rangle$." Here "almost all" means $z$ in the $\Gamma$-orbit of the interior of $\mathrm{ext}(G)$.

Definition 4.5 reads: "A set G is a basis for $\Gamma$ if $\mathrm{ext}(G)$ is a fundamental domain for $\langle G\rangle = \Gamma$." Remark 4.6 states that a normalized basis solves the word problem.

### 10.5. Normalized Basis

[VOIGHT2009] Algorithm 4.7 {P} computes a normalized basis:

1. "Let $G := \{g_1, \dots, g_t, g_1^{-1}, \dots, g_t^{-1}\}$."
2. "Compute the normalized boundary U of $\mathrm{ext}(G)$ by Algorithm 2.5."
3. "Let $G' := U$. For each $g \in G$, compute $\bar g = \mathrm{red}_{G \setminus \{g\}}(g)$ using Algorithm 4.3. If $\bar g \ne 1$, set $G' := G' \cup \{\bar g\}$."
4. Compute the normalized boundary $U'$ of $\mathrm{ext}(G')$. If $U' = U$, set $G := G'$ and proceed to Step 5. Otherwise set $U := U'$ and return to Step 3.
5. "If all vertices of $E = \mathrm{ext}(U)$ are paired, return U. Otherwise, for each $g \in G$ with a vertex $v \in I(g)$ which is not paired, compute $\bar g := \mathrm{red}_G(g, v)$, where if v is a vertex at infinity we replace v by a nearby point in $I(g^{-1}) \setminus E \subset \mathfrak D$. Add the reductions $\bar g$ for each nonpaired vertex v to G and return to Step 2."

Step 4 is quoted with its punctuation changed. A vertex $v$ is paired "if and only if for every side $s \subset I(g)$ containing v, we have that $gv \in I(g^{-1})$ is a vertex of E". The algorithm terminates because $\mathrm{ext}(G \cup \{\bar g\}) \subsetneq \mathrm{ext}(G)$ while $\langle G\rangle$ stays unchanged and finitely generated.

### 10.6. Presentation from a Dirichlet Domain

[VOIGHT2009] Theorem 5.1 {P} reads: "There exists an algorithm which, given a finitely generated Fuchsian group $\Gamma$ and a point $p \in \mathbb H$ with $\Gamma_p = \{1\}$, returns the Dirichlet domain D(p), a side pairing for D(p), and a finite presentation for $\Gamma$ with a minimal set of generators." Algorithm 5.2 walks the minimal vertex cycles.

Proposition 5.4, citing [BEARDON] Theorem 9.4.5, states that for $p$ outside a set of area zero, as items (i) to (iii) that are items (1), (2), and (5) of Beardon's theorem on p. 232 {P}:

- "Every elliptic cycle has length 1".
- "Every accidental cycle has length 3".
- "Every parabolic cycle has length 1."

An accidental cycle gives the relation $g = 1$, an elliptic cycle gives $g^k = 1$, and a parabolic cycle gives the empty relation, detected by "$\mathrm{trd}(g) = \pm 2$". Algorithm 5.7 removes the accidental relations by back-substitution to reach a minimal presentation.

### 10.7. Greedy Reduction in SymmHub

SymmHub reduces a point by a different rule, which no source proves to terminate. On the CPU, `toFundDomain` ([SYMMHUB] `lib/invlib/Group.js:191` {C}) runs these steps with `maxIterations = 20`:

1. For each side in order, test whether `sigDistanceSP(fd[i],pnt) > 0`, which means that the point lies outside that side (line 211).
2. If it does, apply the reverse transform of that side, accumulate the transform, and restart from side 0.
3. If a full pass finds no such side, report that the point is in the domain.

The CPU test has no tolerance and assumes a convex domain. The GPU loop uses `EPS=1e-7` and defaults to 10 iterations ([SYMMHUB] `lib/shaders/inversiveSampler.glsl.mjs:182`, `SymRenderer.js:393` {C}). The legacy shader uses `MAX_ITERATIONS 100` and `EPSILON 1e-10`, and its walled variant skips a side when the point lies inside any of that side's wall splanes ([SYMMHUB] `apps/orbifold/js/shaders/inversive.glsl.mjs:15-16, 735-800` {C}). The header of `Group_ON.js` asserts that "the fold still converges since every transform is a group element", without proof. `docs/research/rendering.md` covers the GPU loop.

Voight's termination argument covers the rule that applies the generator that most decreases the distance to the centre. It does not cover the rule that applies the first violated side. A termination argument for the greedy rule, for convex reflection domains and for general side-pairing polygons, is gap G5, which lists the sources searched without result. [DAVIS2008] contains no algorithm that folds a point across walls and no bound on the number of steps {P}. [EPSTEIN-PETRONIO1994] Theorem 4.14 (f) gives only that every point lies in some image of the polygon, which says that a group element exists {P}.

### 10.8. Kleinian Analogue

[PAGE2012] Algorithm 2 and Proposition 10 {S} reduce a point for a Kleinian group by the same strict-decrease argument: "while the algorithm runs the distance $d(w', 0)$ decreases ... this orbit is discrete, the algorithm terminates". Proposition 32 bounds the effect of floating point with a factor $1 + \beta$.

### 10.9. Verifying Discreteness

These checks are available:

- The conditions of Poincare's theorem, Section 4.8, which certify discreteness with the hypotheses of Sections 4.7 and 4.9 to 4.11 {P}. [EPSTEIN-PETRONIO1994] p. 160 {P} checks Cyclic by multiplying the pairing matrices, checks that a product is the identity on a face by a linear independence calculation, and checks that a rotation angle has the form $2\pi/m$ by testing whether the $m$-th power of a group element is the identity, with floating point to find which $m$ to test. Page 161 adds that algebraic-number entries allow the check in exact integer arithmetic.
- The negative trace theorem for each pair of pants, Section 7.7 ([MASKIT2001] Theorem 4.1 {P}).
- Jorgensen's inequality, which is a necessary condition only {S}.
- SymmHub's numerical check of every relator in `GroupPresentation.js`, which throws when the domain does not tile (Section 4.6, {C}).

## 11. Delaney-Dress Symbols

This section is a survey for orientation. [TEGULA2020] {N} defines a Delaney-Dress symbol as "a finite set $\mathcal D$, together with the action of a certain free group $\Sigma$, together with maps $m_{01}, m_{12}, m_{02} : D \to \mathbb N$". Its nodes are classes of chambers.

The orbifold is read from the symbol through the relation $m_{ij} = v_{ij}r_{ij}$. "Reflections are mapped onto boundary segments, rotational centers are mapped onto cones", and "glide-reflections give rise to crosscaps". The curvature is $\mathcal K = \sum_D(1/m_{01} + 1/m_{12} - 1/2)$, and its sign gives the geometry ([TEGULA2020] p. 6 {N}).

[BALKE-HUSON1996] p. 95 {P} defines $\Sigma = \langle\sigma_0, \sigma_1, \sigma_2 \mid \sigma_i^2 = 1\rangle$ as "the Coxeter group freely generated by three involutions" (Section 12 compares the two wordings). It defines the branching number $v_{ij} = m_{ij}/r_{ij}$, where $r_{ij}(D)$ is the least $r \ge 1$ with $D(\sigma_i\sigma_j)^r = D$. Page 96 defines the curvature as $\sum_D(1/m_{01}(D) + 1/m_{12}(D) + 1/m_{02}(D) - 1)$ with $m_{02} = 2$ by axiom (DS3), and Theorem 4.1, p. 97, states that the curvature equals twice the orbifold Euler characteristic. Algorithm 5.1, p. 99, finds the boundary components of the orbifold from the symbol.

Tegula, a Java program under GPL-3.0, implements these symbols. Its canonicalization in `OrbifoldGroupName.java` is known only from a secondary code reading {S}.

## 12. Conflicts Between Sources

Each conflict is stated with the recommendation of the research where it gives one.

- Self-paired sides: [SOLOMYAK2020] Theorem 12 excludes them. [SOLOMYAK2020] slide 22, [VOIGHT2009] Section 1, and [SERIES2013] Definition 6.1 handle a side paired with itself by a half-turn by splitting it at the fixed point {P}. [JESPERS2015] {S} and SymmHub {C} allow a reflection to pair a side with itself, with relator $w^2 = 1$. [EPSTEIN-PETRONIO1994] Remark 3.3 and [RATCLIFFE] p. 694 {P} allow any pairing of order two for a side paired with itself, without a split. The research concludes that the sources agree once half-turns and reflections are kept apart, and that none of them requires the split (Section 4.8, item 2).
- Completeness against parabolic cycles: [MASKIT2001] Section 4.4 and [SERIES2013] condition (b) {P} both require parabolic cycle transformations at ideal vertices, as does de Rham {S}. [MASKIT1971], which was not read, uses a completeness condition according to secondary citations. [VOIGHT2009] Proposition 1.1 relies on finite area instead. [EPSTEIN-PETRONIO1994] Theorem 6.3 and [RATCLIFFE] Theorems 13.4.4 and 13.4.6 {P} state completeness conditions through horospheres, the group $G_p$, and links, and the proof of [EPSTEIN-PETRONIO1994] Theorem 8.1 gets a cusp from parabolic holonomy only for pieces that glue in a cyclic fashion (Section 4.8, item 5).
- Element that fixes an ideal vertex: the proof of [VOIGHT2009] Proposition 1.1 prints that "the condition that $\mu(D) < \infty$ ensures that any vertex which lies on the circle at infinity is fixed by a hyperbolic element", citing Beardon. [BEARDON] Theorem 9.3.8, p. 224, prints that such a point "is fixed by a parabolic element of $G$ (and not by any hyperbolic element)", for a convex fundamental polygon of a Fuchsian group, and Exercise 9.8, items 2 and 3, ask for parabolic elements {P}. The research does not decide between them (gap G1).
- Bisector of Voight's Remark 1.5: [VOIGHT2009] p. 5 prints that $\{z : d(z, p) = d(gz, p)\}$ "corresponds in $\mathfrak H$ to the perpendicular bisector of the geodesic between $p$ and $g(p)$", and then gives for $p = i$ the radius $(a^2 + b^2 + c^2 + d^2 - 2)/(a^2 + c^2 - 1)^2$ and the centre $(ab + cd)/(a^2 + c^2 - 1)$ {P}. A reader's check against [BEARDON] Section 7.21 found that the radius cannot hold together with the first statement. That check is a computation, not a source statement (gap G9).
- Names of two plane groups: [SYMMETRIES-OF-THINGS2008] Table A.1, p. 416, aligns `p3m1` with `*333` and `p31m` with `3*3` {P}. [MACBEATH1967] Table I, p. 1204, gives the name `p3m1` to the signature that the dictionary of `docs/research/orbifold-notation.md` reads as `3*3`, and `p31m` to the one it reads as `*333` {P}. [SYMMETRIES-OF-THINGS2008] p. 415 notes that "The notations p3m1 and p1m3 were inadvertently interchanged by Niggli", an error "repeated in editions of Coxeter and Moser before 1980" {P}. The research does not decide between the sources. The table of Section 9.5 gives the names of Table A.1 and does not resolve the disagreement.
- The group of a Delaney-Dress symbol: [TEGULA2020] {N} calls $\Sigma$ "a certain free group". [BALKE-HUSON1996] p. 95 {P} calls it "the Coxeter group freely generated by three involutions".
- Twist normalization: [MASKIT2001] {P} uses $2\tau$ as a distance independent of length, with coordinate $t = \sinh\tau$. Wikipedia {S} uses a signed distance. SymmHub {C} is inconsistent internally (Section 8.13). The research makes no recommendation (gap G11).
- Twist matrix sign within Maskit: Sections 5.5 and 7.2 of [MASKIT2001] use $\tilde f_\tau$ for two different maps {P}. An implementation keeps them apart.
- Internal typos in Maskit: Section 7.13 lists them and the recommendation.
- Chapter of Thurston: [CHOI-GOLDMAN2001] {S} cites chapter 5. The electronic version 1.1 of the notes has the result as Corollary 13.3.7, p. 318 {N}, and SymmHub cites "Corollary 13.3.7". The research takes chapter 13 as the verified reference and considers chapter 5 to be another version of the notes, which is unverified.
- Euler characteristic of NEC signatures: a first reading of [HUGHES2019] {S} had a sign flip on the corner term for sign $-$. [THURSTON2002] 13.3.4 and [CONWAY-HUSON2002] {N} give $-\frac{1}{2}\sum(1 - 1/n_i)$ regardless of orientability. The research resolved this as an extraction error.
- Deaton: the SymmHub tutorial names "A. A. Deaton, Hyperbolic Orbifolds and Patterns, Princeton senior thesis 1993", `outline.html` names "Derek Deaton", and the code header (`OrbifoldGeometrization.js:22`) gives 1994 ([SYMMHUB] {C}).
- Glyphs: [CONWAY-HUSON2002] and [TEGULA2020] write a handle as `o` and a cross-cap as `x`. [FIELD-GUIDE2018] Section 30 uses a filled spot for a handle and a small circle for a cross-cap {N}. A parser must not read `o` as a cross-cap.
- Conjugation direction: [CONWAY-HUSON2002] writes $\lambda^{-1}P\lambda = S$ and [ARXIV-1702.02342] writes $c_s = e\,c_0\,e^{-1}$. This is a convention only.
- Commutator: [VOIGHT2009] and [CONWAY-HUSON2002] use opposite conventions (Section 2.5). This is a convention only.
- Definition of a fundamental domain: [VOIGHT2009] and [DAVIS2008] state it differently, and [DAVIS2008] adds the strict form (Section 2.4). A Dirichlet domain need not be strict ([DAVIS2008] Complement 6.6.10 {P}).

## 13. Open Questions and Unverified Claims

### 13.1. Gaps by Class of Signature

This table lists the classes of Section 3 in the same order, with the gaps of Section 13.2 that block each one when only primary sources are used. The last column of the table in Section 3 gives the resulting status.

| Class of signature | Blocking gaps | Effect |
| --- | --- | --- |
| Reflection triangle groups `*pqr` | None | Fully sourced |
| Rotation triangle groups `pqr` | None | The group, its generators, and its presentation are sourced. The convexity and the vertex cycles of the double of the reflection triangle as a fundamental domain are not (Section 13.3, item 11). |
| Polygon reflection groups `*p1...pn` | G3 for the two $\operatorname{arsinh}$ calls of the sheets, G4 for SymmHub's order of cuts | Existence, discreteness, parameter count, the parametrization by the lengths of the cuts, and one polygon for each set of angles are sourced, including ideal corners (Section 9.4.2). SymmHub's coordinates of the whole family are read line by line (Section 9.4.1) and rest on code. |
| Orientable hyperbolic, no mirrors | G6 for a canonical polygon with coordinates or matrices, G11 for the twist convention | Generators are sourced, except for the choice of twist convention. One polygon is sourced, with its maps given by figures only. |
| Mixed signatures with cone points and mirrors | G2, G3, G4, G8, and G1 where a cusp lies on a mirror | Only SymmHub code for the geometry. The polygon theorem that certifies it is sourced. |
| Several mirror boundaries | G2, G3, G4, G15, and G1 where a cusp lies on a mirror | Only SymmHub code for the geometry. The polygon theorem that certifies it is sourced. |
| Mirrors together with handles | G2, G3, G4, and G1 where a cusp lies on a mirror | Only SymmHub code for the geometry. The polygon theorem that certifies it is sourced. |
| Non-orientable, with cross-caps | G2, G3, G4 | Only SymmHub code for the geometry. The polygon theorem that certifies it is sourced. |
| The 17 Euclidean groups | G7 | Catalogue code, with partial moduli counts |
| The 14 spherical families | G8 | Reflection triangles are sourced. The lune of `*nn` is not (Section 13.3, item 9), nor the lune of `nn` with one rotation pairing (item 10). The other generators are catalogue code. |
| Bad orbifolds | None | No group exists |
| Any class, point reduction with the greedy rule | G5 | Termination unproven |

### 13.2. Gaps

- G1, Poincare's theorem for orientation-reversing groups: Sourced for the four cases of Section 4.7 by [EPSTEIN-PETRONIO1994] and [RATCLIFFE], with the hypotheses that Sections 4.7, 4.9, and 4.10 state. Two points stay open. The first is the test to run at an ideal point where a mirror ends at a cusp (Section 13.3, item 8). The second is the disagreement between [VOIGHT2009] and [BEARDON] on the element that fixes an ideal vertex (Section 12). Needed: a source or a machine-checked proof for each.
- G2, matrices for pieces with mirrors and cross-caps: [MASKIT2001] covers orientable pieces only. Needed: an extension that uses the reflection matrices of [MASKIT2001] Section 4.3, and a gluing of one-sided geodesics by glide reflections. [NORBURY2006] {S} states that the gluing is antipodal with no twist. SymmHub's cap construction (Section 8.11) is the only explicit one, and it is unproven. [RATCLIFFE], [BEARDON], and [EPSTEIN-PETRONIO1994] give no such matrices. [RATCLIFFE] Example 5 of Section 9.2 is their only explicit pairing of a polygon for a non-orientable surface (Section 4.7). Candidate sources: [CHOI2012], [BUSER].
- G3, the generalized law of cosines with imaginary angles: SymmHub's `edgeLength` mixes real angles with truncation lengths. Maskit's continuation $\lambda = i\pi/\alpha$ covers cone points inside pants matrices. [RATCLIFFE] and [BEARDON] state the triangle, quadrilateral, pentagon, and hexagon formulas of Section 9.3. No source read states a formula for the cases that `edgeLength` sends to $\operatorname{arcsinh}$. That each $\operatorname{arccosh}$ case of `edgeLength` coincides with one of the formulas of Section 9.3 is a reader's reading, not a source statement. Needed: a source for the $\operatorname{arcsinh}$ cases and for the correspondence, or a machine-checked proof. Candidate source: [BUSER] Formula Glossary 2.2 to 2.4. For the sheets of a kaleidoscope, Section 8.7 names the two $\operatorname{arcsinh}$ calls, in cases B and C, and the published formulas whose form each $\operatorname{arccosh}$ call shares. The $\operatorname{arcsinh}$ call of case C has the form of a formula that rests on a secondary source only. A port of the code found every call of the sheets, the two $\operatorname{arcsinh}$ calls included, in agreement with an independent construction, to the differences that `docs/research/symmhub.md` Section 6.9 gives (Section 8.7). That is numeric evidence from a port and closes nothing.
- G4, Deaton's rules: Five of the nine cut types, the rules for cross-caps, cone pairs, and corners, and the claim that the twist appears in exactly three situations rest only on SymmHub. Candidate source: [DEATON1993]. For a single kaleidoscope, [THURSTON2002] p. 316 states the decomposition into generalized triangles, the parametrization by the lengths of the cuts, and the omission of a cut where two corners of order 2 meet (Section 9.4.2). SymmHub's order of cuts, and that its slices and corner pairs are the cuts and edge lengths of that statement, rest on code (Section 13.3, item 12).
- G5, termination of the greedy reduction: Section 10.7 states the gap. No source read states that repeated application of side pairings moves a point into the polygon, or in how many steps. [RATCLIFFE], [BEARDON], [EPSTEIN-PETRONIO1994], and [SYMMETRIES-OF-THINGS2008] were searched without result. Section 13.3, item 6, records a candidate argument for reflection groups. Needed: a termination argument for convex reflection domains and for general side-pairing polygons.
- G6, geometry of the canonical Fricke polygon: The polygon with the edge word of equations (5.1) and (5.2), with elliptic and parabolic vertices. [VOIGHT2009] gives only the presentation and defers to [IMBERT2001]. [BEARDON] builds one polygon from the signature, with its maps given by figures only (Section 5.5), and [SINGERMAN1974] gives the combinatorial edge word for compact quotients (Section 5.6). Open: coordinates or matrices for the polygon with that edge word, and whether Beardon's polygon has it.
- G7, Euclidean moduli: [VONGAGERN2009] p. 7 gives the counts for p2, pmm, pmg, pgg, and cmm. [SYMMETRIES-OF-THINGS2008] pp. 237 to 238 gives a rule and the counts one, one, one, and two for `**`, `*x`, `xx`, and `o` (Section 6.1). Neither source states whether a count includes the overall scale, and neither states the counts of the other Euclidean groups. Whether Thurston's formula extends to $\chi = 0$ is unverified. Candidate sources: [KULKARNI-LEE-RAYMOND1985], [SCOTT1983].
- G8, geometric constructions for mixed signatures and spherical generators: `p*q`, `2*pq`, spheres with cone points, and the generators of the spherical groups have only presentations and catalogue code {C}. [CONWAY-HUSON2002] {N} and [SYMMETRIES-OF-THINGS2008] {P} give the presentations, the latter with a description of the geometry without formulas, and [RATCLIFFE] works one mixed triangle (Section 9.7). No source read gives a construction from the signature.
- G9, the bisector radius of Voight Remark 1.5: The printed radius and the printed definition of the set disagree according to a reader's computation (Section 12). Needed: a corrected source, or a machine-checked proof of the radius and centre, before use.
- G10, the signature of the orientation-preserving subgroup in Conway terms: [SINGERMAN1974] Theorem 2 states it in NEC signatures for a proper NEC group with compact quotient (Section 5.6). [SYMMETRIES-OF-THINGS2008] Table 3.2, p. 40, lists the rotation subgroup of each plane group at the bottom of its column. Open: a statement in orbifold notation for hyperbolic signatures, which `docs/research/orbifold-notation.md` handles, and any statement for signatures with cusps.
- G11, the twist convention for Escher: The three conventions of Section 2.5 are all in use, and [RATCLIFFE] Section 9.7 has a fourth for closed orientable surfaces. No source chooses one for an implementation that also has mirrors and cross-caps.
- G12, the Maskit worked example: The Section 11 matrix for $\tilde e_{3,3}$ does not match equation (19), and needs a check by computation before it serves as a test case.
- G13, the dimensions of the A-type pieces of [CHOI-GOLDMAN2001]: Not verified.
- G14, the chapter number in [CHOI-GOLDMAN2001]: Its citation of chapter 5 of Thurston's notes is unverified.
- G15, reflection-generated groups with several boundaries: Which signatures with several boundaries, such as `**`, give groups generated by reflections. [CONWAY-HUSON2002] suggests that they need generators other than reflections. [SYMMETRIES-OF-THINGS2008] p. 120 states only that the two reflections of `**` do not generate it (Section 5.3). Whether any set of reflections generates such a group is not stated by any source read.

### 13.3. Candidate Statements That Need a Source

These statements appear in the research or in code, and no cited source states them. Each needs a source or a machine-checked proof before the body can use it as a fact.

1. SymmHub's `edgeLength` formula of Section 8.7 gives the correct edge length in every mixed case of real and imaginary vertices.
2. SymmHub's cap generator of Section 8.11 is a glide reflection that, together with the other side pairings, satisfies the hypotheses of [EPSTEIN-PETRONIO1994] Theorem 4.14 or [RATCLIFFE] Theorem 13.5.3 (Sections 4.9 and 4.10).
3. The greedy reduction of Section 10.7 terminates for every point and every domain that SymmHub builds.
4. The twelve atoms of SymmHub correspond one to one to the twelve elementary orbifolds of [CHOI-GOLDMAN2001].
5. The twist appears in exactly three situations, as the SymmHub tutorial states.
6. For a discrete reflection group with fundamental polygon $P$, the fold that applies the reflection in any violated side terminates. A reader argued this from [RATCLIFFE] Theorem 6.6.2, the definition of $D(u)$ on p. 243, which requires a centre with trivial stabilizer, and Exercise 7.1.1, an exercise without a printed proof (Section 10.1). The argument is a derivation, not a source statement, and it gives no bound on the number of steps.
7. Two sides of a polygon that meet only at an ideal vertex have the dihedral angle 0 in the definition of [RATCLIFFE] (6.7.2), so that Theorem 7.1.4 deletes their relator. This is a reader's reading of the definition of adjacent sides on p. 255.
8. Item 5 of Section 4.8 fails, or is not the right test, at an ideal point where a mirror ends at a cusp. A reader claimed this from [RATCLIFFE] Example 6 of Section 13.4, which decides completeness through Theorem 13.4.4 and does not mention parabolic elements. The book does not decide the claim, and it does not discuss whether a cycle through the mirrors at such a point gives a parabolic element.
9. For an integer $n \ge 2$, the lune of $S^2$ with the angle $\pi/n$ is a strict fundamental domain for the group generated by the reflections in its two sides, a dihedral group of order $2n$. [DAVIS2008] Theorem 6.4.3 requires a convex polytope, and a lune is not one in the sense of p. 75 by a reader's reading. Example 6.4.2 states the case of $S^1$ only, and the remark in the proof on cones that are not essential does not name $S^2$ or a lune (Section 4.5). A source that states it, or a Lean proof, would close it.
10. For an integer $n \ge 2$, the lune of $S^2$ with the angle $2\pi/n$, whose two sides are paired with each other by the rotation through $2\pi/n$ about its vertices, is a fundamental domain for the cyclic group of order $n$ that this rotation generates. For $n = 2$ the lune is a hemisphere, and its two sides lie on one great circle. The lune is the union of the lune of item 9 and its image in one of its sides. The polygon theorems of Sections 4.9 and 4.10 require a convex polyhedron, and whether Definition 2.1 of [EPSTEIN-PETRONIO1994] and Theorem 4.14 with (k) admit a lune is not established, for the reason that item 9 gives. A source that states it, or a Lean proof, would close it.
11. Let $P$ be a convex polygon in $H^2$, $E^2$, or $S^2$ whose angles are $\pi/m_j$, and let $R$ be the reflection in the wall of one of its sides, through its vertices $v$ and $w$. Then $P \cup R(P)$ is convex, its sides are the sides of $P$ and their images other than that side, with the side pairings $R R_s$ on a side $s$ of $P$ and $R_s R$ on its image, and its vertex cycles are $\{v\}$ with the angle $2\pi/m_v$, $\{w\}$ with the angle $2\pi/m_w$, and $\{u, R(u)\}$ for each other vertex $u$ with the angle sum $2\pi/m_u$. Where a side of $P$ meets that wall at a right angle, the side and its image lie on one geodesic and meet at a vertex of the angle $\pi$. [VONGAGERN2009] p. 12 states that the fundamental cells of p3, p4, and p6 "consist of two copies of the triangles" of the reflection groups (Section 9.5), and [MASKIT2001] Section 4.4 gives the rotation generators as products of two reflections (Section 4.4), but neither states the convexity of the union or its vertex cycles. [EPSTEIN-PETRONIO1994] Definition 2.8 admits a convex cell "whose proper faces may have been subdivided" (Section 4.9). That this definition covers a vertex of the angle $\pi$ that splits a side of the union is a reader's reading. A source that states the convexity and the cycles, or a Lean proof, would close it.
12. SymmHub's slices of a kaleidoscope are cuts of the decomposition into generalized triangles that [THURSTON2002] p. 316 describes, and its corner pairs are the edge lengths that the same page makes parameters where two corners of order 2 meet, so that SymmHub's lengths are coordinates of the family $(\mathbb R^+)^{l-3}$ (Sections 8.4 and 9.4.1). The figure of that page was not compared with SymmHub's order of cuts. A comparison of the figure, a source on the cut pattern such as [DEATON1993], or a Lean proof would close it.
13. For every choice of lengths of the slices and corner pairs in the range 0.3 to 4, each sheet of Section 9.4.1 exists and closes with the angle $\pi/m$ at each corner and right angles at the ends of each side of length $\ell$, and the welded polygon is convex, with the angles $\pi/p_j$ in the order of the signature. The code computes no last side of a sheet and checks none of this (`docs/research/symmhub.md` Section 6.4.1), and Section 8.14 records that the order fails for many signatures. A port of the code measured the closure of the sheets, the convexity and the angles of the polygons, and their order, on the signatures with four to eight corners and the lengths from 0.3 to 4 that `docs/research/symmhub.md` Sections 6.8 and 6.9 list, with the results and the failed checks given there. With the features of a sheet reordered by a cyclic rotation only (Section 9.4.1, step 4), it found the order right on all 47,641 corner sequences that it tried, as typed and not reduced to canonical form. A later prototype counted them as 4,421 canonical signatures. That is numeric evidence from a port. The formulas of Section 8.7 and a source or a Lean proof for the existence of each sheet would close it.

   The same statement covers the closing gap of each sheet, the side that SymmHub's layout does not compute: that it is the function of Section 8.7 with its three features in the cyclic order of the sheet, which is an $\operatorname{arsinh}$ call in a sheet with two corners or one corner and an $\operatorname{arcosh}$ call in a sheet with none. A prototype for the design of `escher-domain` compared it with the sheet layouts of a port of the code and found a relative difference of at most $5.6 \times 10^{-8}$, $2.0 \times 10^{-7}$, and $7.6 \times 10^{-11}$ for two, one, and no corners, over 12,199 sheets, none with three corners. The design of `escher-domain` reads a cut sheet in the counterclockwise order of the polygon, with the last two features $y$ and $x$ of the list, then the slice that stands for the complementary arc. It places the gap between two consecutive features $f$ and $g$ on the side after the last vertex of $f$. That reading is a reader's construction, which no source and no line of the code states. Its only support is numeric. On the same prototype, the walks built with it closed, while the walks built from the order in which the code pops the features failed to close by 2.0 to 9.2 on `*3333`, `*5432`, `*22222`, and `*65432`. This is numeric evidence from a prototype, not a source or a proof.

14. At each end of a welded slice, the side kept and the side dropped lie on one geodesic, so the welded polygon has no vertex there (Section 8.10). The code reading inferred this from the right angles at the ends of the slice in both sheets, which rest on item 13. A port of the code measured the angle at both ends of every welded slice that it built and found it equal to $\pi$ within the bound that `docs/research/symmhub.md` Section 6.9 gives. That is numeric evidence from a port. A source or a Lean proof would close it.

   The design of `escher-domain` builds the polygon without welds, by one walk along its boundary, in which each side not inside a corner pair has as its length the sum of the gaps of the sheets that lie on it, and the walk turns by $\pi - \pi/p_j$ at each corner. That the walk gives the welded polygon is a reader's construction, which needs this statement and item 13 and which no source states. Its only support is numeric. A prototype for the design of `escher-domain` built 600 random signatures with 4 to 12 corners, orders up to 1000, and lengths from 0.3 to 4 by such a walk at 50 digits, and found closure errors of at most $5.4 \times 10^{-28}$, angles within $1.3 \times 10^{-24}$ of $\pi/p_j$, areas within $2.6 \times 10^{-24}$ of $-2\pi\chi$, and every polygon convex. At 120 digits, with lengths from $10^{-6}$ to 60, every polygon was convex with the right angles to about $10^{-33}$. The distances between the vertices of the walk agreed with those of the welded layout of a port of the code to $2.1 \times 10^{-7}$, the precision of the port, on 11 signatures with three settings each, and in the walk the distance between the two walls just outside the arc of a slice equalled the length of the slice to $7 \times 10^{-27}$, which bears on the inference of Section 9.4.1, step 2. The prototype did not measure the angle at a slice end in high precision. These are numeric evidence from a prototype, not a source or a proof.

### 13.4. Sources Not Read in Full

These sources were not read, or were read only in part. `docs/research/source-requests.md` records each one with what the project needs from it. [SINGERMAN1974], [MACBEATH1967], and [BALKE-HUSON1996] were read in full. [EPSTEIN-PETRONIO1994] was read in full, with the exceptions that the table lists. Section 14.1 says how much of each other source was read.

| Priority | Source | Needed for |
| --- | --- | --- |
| High | [DEATON1993], unpublished, held as a scan in the Mudd Manuscript Library | G4 |
| Medium | [BUSER] Formula Glossary 2.2 to 2.4, Sections 3.1 and 3.3. Paywall. | G2, G3 |
| Medium | [CHOI2012], elementary pieces with mirrors and their deformation spaces | G2, G13 |
| Low | [RATCLIFFE], read in the first edition only, in the sections that Section 14.1 lists. Later editions not read. | The numbering of the cited theorems in the later editions |
| Low | [BEARDON], read in the softcover reprint of the 1983 edition at the cited pages. [VOIGHT2009] cites the 1995 printing, which was not compared. | The numbering of Theorems 9.4.5 and 9.8.4 in the printing that Voight cites |
| Low | [EPSTEIN-PETRONIO1994], printed pp. 118 to 120, 122 to 124, 157 to 159, and 167 to 170, which were checked in the text layer only, and the proofs of Theorems 4.13 and 4.14 on pp. 143 to 147, which were not checked step by step | Lemma 2.7, Propositions 2.5 and 2.11, and Lemma 2.13, which this document uses only through Remark 3.24, and the text of Section 7 on pp. 157 to 159 |
| Low | [SERIES2013], read only in Section 6.1 and Sections 6.1.5 to 6.1.6. The proof in Section 7.6 is unread. | Proof of Theorem 6.14 |
| Low | [IMBERT2001] | G6 |
| Low | [MASKIT1971], which returned HTTP 403 | The original completeness condition |
| Low | [WILKIE1966] | The proofs of Lemmas 1 to 4 that [MACBEATH1967] relays, and the existence of a polygon for an NEC signature |
| Low | [CFKP], available locally but not read | Formulas of models and isometries, needed for no claim here |
| Low | [GILMAN-MASKIT1991] | The two-generator discreteness algorithm |
| Low | [BULATOV2011], the talk and paper PDFs | Bulatov's own account beyond the SymmHub code |
| Low | Dress 1987, Delgado-Friedrichs 2003, and Huson 1993, whose full entries the research did not record | Detail on Delaney-Dress symbols |

### 13.5. Assumptions and Bibliographic Gaps

These assumptions were made in writing this document:

- The reading of Maskit's typography, that $\tilde a$ is a matrix and $a$ its map, follows the usage in the research report and was not checked against the paper.
- The definition of "Fuchsian group" in Section 2.4 summarizes how the sources use the term, without the compact quotient that [SINGERMAN1974] p. 233 requires. The definition of "NEC group" drops the same requirement, as Section 2.4 states.
- The years of the secondary arXiv sources in Section 14.2 are read from their arXiv identifiers.
- [VONGAGERN2009] was read at the cited pages, so its claims carry {P}. The research did not state whether the whole paper was read.
- The statements from [EPSTEIN-PETRONIO1994], [RATCLIFFE], [BEARDON], [SINGERMAN1974], [MACBEATH1967], [SYMMETRIES-OF-THINGS2008], and [BALKE-HUSON1996] carry {P} because each was read by one reader and checked against the PDF by a second reader, in part on the page images and in part on the text layer. Where the two disagreed, this document follows the second reading.
- The locators of [RATCLIFFE] are those of the first edition, which the other research documents also cite. The numbering of the later editions was not compared.

The research did not record the titles of several secondary sources, so their entries in Section 14.2 give the identifier only. The Wikipedia pages for the twist convention and the Lambert quadrilateral are not identified.

## 14. References

### 14.1. Sources Read

- [BALKE-HUSON1996] Balke, L. and D. H. Huson, "Two-dimensional groups, orbifolds and tilings", Geometriae Dedicata 60, 89-106, 1996. Read in full.
- [BEARDON] Beardon, A. F., "The Geometry of Discrete Groups", Graduate Texts in Mathematics 91, Springer, 1983, softcover reprint, ISBN 978-0-387-90788-8. Read at the cited pages of Chapters 7, 9, and 10. Locators give printed pages.
- [CONWAY-HUSON2002] Conway, J. H. and D. H. Huson, "The Orbifold Notation for Two-Dimensional Groups", Structural Chemistry 13(3/4), 247-257, 2002, DOI 10.1023/A:1015851621002, <https://doi.org/10.1023/A:1015851621002>.
- [DAVIS2008] Davis, M. W., "The Geometry and Topology of Coxeter Groups", 2008. Locators give printed pages. The PDF page is the printed page plus 16.
- [EPSTEIN-PETRONIO1994] Epstein, D. B. A. and C. Petronio, "An exposition of Poincare's polyhedron theorem", L'Enseignement Mathematique 40, 113-170, 1994, zbMATH 0808.52011. Read in full, with printed pp. 118 to 120, 122 to 124, 157 to 159, and 167 to 170 checked in the text layer only. Locators give printed pages.
- [FIELD-GUIDE2018] Conway, J. H., Doyle, P., Gilman, J., and W. Thurston, "Geometry and the Imagination in Minneapolis", 1991 handouts, arXiv:1804.03055v1, 2018, <https://arxiv.org/abs/1804.03055>.
- [MACBEATH1967] Macbeath, A. M., Canadian Journal of Mathematics 19, 1192-1205, 1967, DOI 10.4153/CJM-1967-108-5, <https://doi.org/10.4153/CJM-1967-108-5>. Read in full.
- [MASKIT2001] Maskit, B., "Matrices for Fenchel-Nielsen coordinates", Annales Academiae Scientiarum Fennicae Mathematica 26, 267-304, 2001, <http://www.acadsci.fi/mathematica/Vol26/maskit.pdf>.
- [RATCLIFFE] Ratcliffe, J. G., "Foundations of Hyperbolic Manifolds", first edition, Graduate Texts in Mathematics 149, Springer, 1994, ISBN 978-1-4757-4013-4. Read in Sections 2.1, 2.5, 3.5, 6.3, 6.5 to 6.7, 7.1 to 7.3, 9.2, 9.7, 9.8, 11.2, 13.4, and 13.5, and at pp. 505 and 507. Locators give the numbering and the printed pages of the first edition.
- [SERIES2013] Series, C., with assistance from S. Maloni, "Hyperbolic Geometry MA 448", lecture notes, University of Warwick, 2008, revised 2010, last edited 2013-03-01. The printed page is the PDF page minus 17.
- [SINGERMAN1974] Singerman, D., "On the structure of non-Euclidean crystallographic groups", Proceedings of the Cambridge Philosophical Society 76, 233-240, 1974, DOI 10.1017/S0305004100048891, <https://doi.org/10.1017/S0305004100048891>. Read in full.
- [SOLOMYAK2020] Solomyak, B., "Lecture 9: Examples of lattices, Poincare's Theorem", lecture slides, Bar-Ilan University, 2020-12-20.
- [SYMMETRIES-OF-THINGS2008] Conway, J. H., Burgiel, H., and C. Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1-56881-220-5. Read in Chapters 1 to 18 and Appendix A. Locators give printed pages.
- [SYMMHUB] Bulatov, V., Goodman-Strauss, C., and S. Vorthmann, "SymmHub", source code, commit f2d19cb, MIT licence, 2025, <https://github.com/SymmHub/SymmHub>. Paths are relative to the root of the repository.
- [TEGULA2020] Zeller, R., Delgado Friedrichs, O., and D. H. Huson, "Tegula - exploring a galaxy of two-dimensional periodic tilings", arXiv:2007.10625v1, 2020, <https://arxiv.org/abs/2007.10625>.
- [THURSTON2002] Thurston, W. P., "The Geometry and Topology of Three-Manifolds", chapter 13, electronic version 1.1 of the 1980 notes, March 2002, <http://www.msri.org/publications/books/gt3m/>, chapter PDF <https://library.slmath.org/nonmsri/gt3m/PDF/13.pdf>. Pages 311 to 318 of the chapter were read in the text layer for Section 9.4. Locators give printed pages.
- [VOIGHT2009] Voight, J., "Computing fundamental domains for Fuchsian groups", Journal de Theorie des Nombres de Bordeaux 21, 2009, arXiv:0802.0196v4, <https://arxiv.org/abs/0802.0196>.
- [VONGAGERN2009] von Gagern, M. and J. Richter-Gebert, "Hyperbolization of Euclidean Ornaments", Electronic Journal of Combinatorics 16(2), #R12, 2009.

### 14.2. Secondary Sources

- [ARXIV-0812.4161] arXiv:0812.4161, 2008, title not recorded, <https://arxiv.org/abs/0812.4161>.
- [ARXIV-1410.6735] arXiv:1410.6735, 2014, title not recorded, <https://arxiv.org/abs/1410.6735>.
- [ARXIV-1702.02342] arXiv:1702.02342, 2017, title not recorded, <https://arxiv.org/abs/1702.02342>.
- [ARXIV-2105.01087] arXiv:2105.01087, 2021, title not recorded, <https://arxiv.org/abs/2105.01087>.
- [ARXIV-2309.10844] arXiv:2309.10844, 2023, title not recorded, <https://arxiv.org/abs/2309.10844>.
- [ARXIV-2512.00879] arXiv:2512.00879, 2025, title not recorded, <https://arxiv.org/abs/2512.00879>.
- [ARXIV-MATH-0202156] arXiv:math/0202156, 2002, title not recorded, relays de Rham 1971 in its Appendix B, <https://arxiv.org/abs/math/0202156>.
- [CHOI-GOLDMAN2001] Choi, S. and W. M. Goldman, arXiv:math/0107193, 2001, title not recorded, <https://arxiv.org/abs/math/0107193>.
- [DRYDEN-PARLIER2004] Dryden, E. and H. Parlier, arXiv:math/0405129, 2004, title not recorded, <https://arxiv.org/abs/math/0405129>.
- [GENDULPHE2017] Gendulphe, M., arXiv:1706.08798, 2017, title not recorded, <https://arxiv.org/abs/1706.08798>.
- [HUGHES2019] Hughes, arXiv:1910.00519, 2019, title not recorded, <https://arxiv.org/abs/1910.00519>.
- [JESPERS2015] Jespers, E., Kiefer, A., and A. del Rio, arXiv:1504.07779, 2015, title not recorded, <https://arxiv.org/abs/1504.07779>.
- [NORBURY2006] Norbury, P., arXiv:math/0612128, 2006, title not recorded, <https://arxiv.org/abs/math/0612128>.
- [PAGE2012] Page, A., arXiv:1206.0087, 2012, title not recorded, <https://arxiv.org/abs/1206.0087>.
- [WIKI-FUNDAMENTAL-POLYGON] Wikipedia, "Fundamental polygon", <https://en.wikipedia.org/wiki/Fundamental_polygon>.
- [WIKI-TRIANGLE-GROUP] Wikipedia, "Triangle group", <https://en.wikipedia.org/wiki/Triangle_group>.

### 14.3. Sources Not Read

- [BULATOV2011] Bulatov, V., talk of 2011 and "Bending Hyperbolic Kaleidoscopes", Bridges 2011, <http://bulatov.org/math/1101/>, <http://bulatov.org/math/1107/>.
- [BUSER] Buser, P., "Geometry and Spectra of Compact Riemann Surfaces", ISBN 978-0-8176-4992-0.
- [CFKP] Cannon, J. W., Floyd, W. J., Kenyon, R., and W. R. Parry, "Hyperbolic Geometry".
- [CHOI2012] Choi, S., "Geometric Structures on 2-Orbifolds", MSJ Memoirs 27, 2012.
- [DEATON1993] Deaton, A. A., "Hyperbolic Orbifolds and Patterns", senior thesis, Princeton University, 1993, unpublished.
- [GILMAN-MASKIT1991] Gilman, J. and B. Maskit, Michigan Mathematical Journal 38, 13-32, 1991.
- [IMBERT2001] Imbert, M., "Calculs de presentations de groupes fuchsiens via les graphes rubanes", Expositiones Mathematicae 19, 213-227, 2001.
- [KULKARNI-LEE-RAYMOND1985] Kulkarni, R. S., Lee, K. B., and F. Raymond, "Deformation spaces for Seifert manifolds", Lecture Notes in Mathematics 1167, Springer, 1985.
- [MASKIT1971] Maskit, B., "On Poincare's theorem for fundamental polygons", Advances in Mathematics 7, 219-230, 1971, DOI 10.1016/S0001-8708(71)80003-8, <https://doi.org/10.1016/S0001-8708(71)80003-8>.
- [SCOTT1983] Scott, P., "The geometries of 3-manifolds", Bulletin of the London Mathematical Society 15, 401-487, 1983, DOI 10.1112/blms/15.5.401, <https://doi.org/10.1112/blms/15.5.401>.
- [WILKIE1966] Wilkie, H. C., Mathematische Zeitschrift 91, 87-102, 1966, DOI 10.1007/BF01110157, <https://doi.org/10.1007/BF01110157>.
