# Fundamental Domains and Side-Pairing Isometries from an Orbifold Signature

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-15 |
| Applies to | The construction of a fundamental polygon, its side-pairing isometries, and point reduction from an orbifold signature in Escher |

## Abstract

Escher renders a planar symmetry group named by its orbifold signature. To do that it turns the signature and any free shape parameters into a fundamental polygon and a set of side-pairing isometries, and then maps every point of the plane into that polygon. This document collects what the literature and one existing implementation state about each step. It covers Poincare's polygon theorem with its hypotheses, the presentations read from a signature, the dimension and coordinates of the deformation space, and explicit matrices for the orientable building blocks. It also covers the cut-and-weld construction that handles mirrors and cross-caps, the triangle, Euclidean, and spherical cases, and the algorithms for point reduction, Dirichlet domains, and discreteness checks. Every claim carries its source and a status. The document marks the parts of the construction that have no primary source, so that an implementer can tell established constructions from those that rest only on code or on unproven candidates.

## 1. Scope

This document covers the construction that starts from an orbifold signature with its free parameters and ends with a fundamental polygon, its side-pairing isometries, and a procedure that reduces a point into the polygon. It covers hyperbolic, Euclidean, and spherical geometry. It states results, formulas, and algorithms with citations and proves nothing.

It leaves these topics to other documents:

- `docs/research/orbifold-notation.md`: the grammar of signatures, defects and the Euler characteristic, the classification by geometry, the bad orbifolds, the full dictionary between NEC signatures and Conway signatures, and the presentation read from a signature. This document repeats only what the construction uses.
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

A statement that the research assembled from several cited items, such as the checklist in Section 4.8, says so and carries the markers of its parts.

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

### 2.4. Terms

- orbifold signature: A name of a planar symmetry group in Conway's orbifold notation, written with `o` for a handle, `*` for the start of a mirror boundary, `x` for a cross-cap, and digits for cone points and corners. `docs/research/orbifold-notation.md` defines the grammar.
- cone point: A point of the orbifold whose stabilizer is a cyclic rotation group, written as a digit before any `*`.
- corner: A point on a mirror boundary where two mirrors meet, written as a digit after a `*`.
- cusp: A cone point of infinite order, whose cycle transformation is parabolic.
- Fuchsian group: The term that [SOLOMYAK2020] and [SERIES2013] use for a discrete group of orientation-preserving isometries of the hyperbolic plane.
- NEC group: A non-Euclidean crystallographic group, a discrete group of isometries of the hyperbolic plane that may contain orientation-reversing elements. [CONWAY-HUSON2002] p. 247 calls the orbifold symbol "an elegant form of Macbeath's 'signature'" {N}.
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
| Rotation triangle groups `pqr` | [MASKIT2001] Section 5.9 {P} | [MASKIT2001] Section 5.9 {P} | [SOLOMYAK2020] Theorem 12, [SERIES2013] Theorem 6.14 {P} | [THURSTON2002] Corollary 13.3.7 {N} | Yes |
| Polygon reflection groups `*p1...pn`, one boundary, no cone points, handles, or cross-caps | Existence of a convex realization [DAVIS2008] Example 6.5.3 {P}, explicit construction only [SYMMHUB] sheets and slices {C} | Single reflections, [DAVIS2008] Theorem 6.4.3 {P}, matrices [MASKIT2001] Section 4.3 {P} | Coxeter presentation, [DAVIS2008] Theorem 6.4.3 {P} | $n - 3$ dimensions, [THURSTON2002] Theorem 13.8.1 {N}, [DAVIS2008] Example 6.5.3 {P} | Partly, with no sourced coordinates |
| Orientable hyperbolic, with handles, cone points, and cusps, no mirrors | No canonical polygon (Section 5.5), a Dirichlet domain of the generators (Section 10.6) | Generators, not side pairings of a polygon, [MASKIT2001] Section 8 {P} | [VOIGHT2009] equations (5.1)-(5.2) {P}, [CONWAY-HUSON2002] {N} | [MASKIT2001] Section 2.2, Theorem 10.1 {P} | Partly, generators only |
| Mixed signatures with cone points and mirrors, such as `4*3` | [SYMMHUB] hats {C} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N} | Code only |
| Several mirror boundaries, such as `**` in hyperbolic signatures | [SYMMHUB] bands {C} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N} | Code only |
| Mirrors together with handles | [SYMMHUB] {C} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N} | Code only |
| Non-orientable, with cross-caps | [SYMMHUB] caps {C} | Glide reflections [SYMMHUB] {C}, antipodal gluing [NORBURY2006] {S} | [CONWAY-HUSON2002] {N} | [THURSTON2002] Corollary 13.3.7 {N}, [NORBURY2006] {S} | Code only |
| The 17 Euclidean groups | [SYMMHUB] catalogue {C}, triangle and kite regions [VONGAGERN2009] p. 12 {P} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | Partial, [VONGAGERN2009] p. 7 {P} | Partly, catalogue code |
| The 14 spherical families | [SYMMHUB] catalogue {C}, triangles [DAVIS2008] Example 6.5.1 {P} | [SYMMHUB] {C} | [CONWAY-HUSON2002] {N} | Not sourced | Partly, catalogue code |
| Bad orbifolds `m`, `mn`, `*m`, `*mn` with $m \ne n$ | No group exists, [CONWAY-HUSON2002] p. 254 {N} | None | None | None | Not applicable |

Only the triangle groups have a complete construction in primary sources. The orientable class has sourced generators but no canonical polygon. Polygon reflection groups have sourced existence, discreteness, and parameter count, but no coordinates. The classes with mirrors together with cone points, handles, or other boundaries, and the non-orientable class, rest on the SymmHub code for their geometry and generators. No primary source states the polygon theorem that would certify the result. The Euclidean and spherical classes rest on catalogue code beyond their reflection triangles.

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

The converse as quoted states no cycle condition. Voight leaves that condition to [BEARDON] Theorem 9.8.4, which was not read.

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

The book defines a Euclidean convex polytope as "a compact intersection of a finite number of half-spaces" (p. 74) and a hyperbolic one through a polyhedral cone inside the light cone (p. 76). The research found no discussion of ideal vertices in Chapter 6, so it is not established that Theorem 6.4.3 covers hyperbolic polygons with ideal vertices. This document applies it to compact polygons only.

### 4.6. Variants in the Literature

These variants rest on secondary sources:

- Presentation for polyhedra in all three geometries: [JESPERS2015] Theorem 5.5 {S} states that for a locally finite fundamental polyhedron in Euclidean, hyperbolic, or spherical space, "the pairing and cycle relations form a complete set of relations". A self-paired side gives $\gamma_S^2 = 1$, a "reflection relation", and a cycle gives $(\gamma_{S_1}\cdots\gamma_{S_k})^t = 1$.
- Ideal vertices: de Rham 1971, as relayed by [ARXIV-MATH-0202156] App. B {S}, reads "If all the vertex-cycle transformations in G are parabolic, then G is a discrete subgroup." [ARXIV-2512.00879] App. A {S} replaces Maskit's completeness condition with a "parabolic cycle condition".
- General isometric face pairings: [ARXIV-0812.4161] Theorem 3.5 {S} requires that "the total interior angle equals $2\pi$ at some point of an edge for every cycle".

SymmHub follows the theorem in `lib/grouplib/GroupPresentation.js` ([SYMMHUB] lines 14-60 and 353 {C}). Its side-pairing relators are $w_iw_j = 1$, with $w_i^2 = 1$ for a self-paired side. It finds corner-cycle relators by walking vertices, with angle sum $2\pi/m$ and the $m$-th power as the relator. For reflection groups it uses Coxeter relators from the mirror angles. It checks every relator numerically and throws when the domain does not tile, with the constants `EPS=1e-7`, `MAX_CYCLE_LENGTH=100`, and `IDEAL_ANGLE=1e-5`. A domain twisted until its cells overlap fails that check, and such a group is derived from an untwisted copy through `derivedPresentation` (line 639).

### 4.7. What Is Not Sourced

The orientation-preserving theorem is verified twice, in [SOLOMYAK2020] and [SERIES2013], including ideal vertices and sides on the boundary circle. The reflection group of a compact convex polygon is verified in [DAVIS2008], and the triangle case with ideal and ultra-ideal vertices in [MASKIT2001]. No primary source that was read states the theorem for these cases:

- A polygon whose side pairings include both reflections and orientation-preserving maps, as in a signature with a cone point and a mirror.
- Vertex cycles that pass through glide-reflection pairings, as in a signature with a cross-cap.
- Corners with angle $\pi/m$ between mirror sides in a polygon that also has paired sides, as in a signature with a mirror and a handle.
- Reflection polygons with ideal vertices and more than three sides.

Section 13, gap G1, records this and the sources most likely to close it.

### 4.8. Implementer's Checklist

This checklist is assembled by the research from the statements of Sections 4.1 to 4.6. Each item is sourced, but no single source states the list for the general orientation-reversing case.

1. Every side is paired with exactly one side, and the pairing of the partner side is the inverse map ([SOLOMYAK2020] Definition 5 and Lemma 6, [SERIES2013] Lemma 6.2, [VOIGHT2009] Section 1 {P}).
2. A side fixed by an element of order 2 is split at its fixed point, and the new vertex has angle $\pi$ ([VOIGHT2009] Section 1, [SOLOMYAK2020] slide 22, [SERIES2013] Definition 6.1 {P}).
3. Every finite vertex cycle has $m \cdot \sum\angle = 2\pi$ for an integer $m \ge 1$, and its cycle transformation has order $m$ ([SOLOMYAK2020] Theorem 12, [SERIES2013] condition (a) {P}).
4. In a reflection polygon, every corner has angle $\pi/m$, and the relator is $(r_ir_j)^m$ ([DAVIS2008] Theorem 6.4.3 for compact polygons, [MASKIT2001] Section 4.4 for triangles {P}). The cycle rule for polygons that mix mirrors with other side pairings is not sourced (gap G1).
5. Every ideal vertex cycle transformation is parabolic ([SERIES2013] condition (b) and Proposition 6.11, [MASKIT2001] Section 4.4 {P}, de Rham {S}).
6. A side on the boundary circle needs no condition, because its cycle is hyperbolic or the group is elementary ([SERIES2013] Theorem 6.14 and Proposition 6.13 {P}).
7. The presentation is the pairing relators together with the cycle relators ([SOLOMYAK2020] Theorem 12 (3), [SERIES2013] Theorem 6.14 (iii) {P}, [JESPERS2015] {S}).

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

[THURSTON2002] p. 320 {N} reads: "When O has the combinatorial type of a polygon ... $\pi_1 O$ is generated by reflections". That is the class of signatures with a single boundary `*n1...ns` and no other feature. [THURSTON2002] Proposition 13.3.1(iii), p. 308 {N}, gives the local group of a corner reflector as $\langle a, b : a^2 = b^2 = (ab)^n = 1\rangle$. Which signatures with several boundaries are generated by reflections is not stated by any source read (gap G15).

### 5.4. Area and Euler Characteristic as a Check

A constructed polygon can be checked against the area that the signature predicts. [THURSTON2002] Theorem 13.3.5 {N} states $\int_O K\,dA = 2\pi\chi(O)$, so a hyperbolic orbifold of curvature $-1$ has area $2\pi\lvert\chi(O)\rvert$, with no assumption of orientability. A spherical group has order $2/\chi$ ([CONWAY-HUSON2002] pp. 252 to 253 {N}). `docs/research/orbifold-notation.md` gives $\chi$ for each signature.

### 5.5. Canonical Polygon Geometry

No source that was read gives the geometry of the canonical polygon of a signature with cone points and cusps, the polygon whose edge word realizes equations (5.1) and (5.2). Section 13 records this as gap G6.

## 6. Moduli

### 6.1. Deformation Dimension

[THURSTON2002] Corollary 13.3.7, p. 318 {N}, reads: "The Teichmuller space $\mathcal T(O)$ of an orbifold O with $\chi(O) < 0$ is homeomorphic to Euclidean space of dimension $-3\chi(X_O) + 2k + l$, where k is the number of elliptic points and l is the number of corner reflectors." The proof cuts $O$ into primitive pieces, and the parameters are the lengths and twists of closed geodesics and the lengths of arcs. The hypothesis $\chi(O) < 0$ excludes the Euclidean and spherical groups.

Two cases in [THURSTON2002] support the formula {N}. On p. 316, the disc `D^2( ; m1...ml)` is parametrized by $(\mathbb R_+)^{l-3}$. Theorem 13.8.1, p. 347, states that for an $n$-gonal orbifold $P$, $N(P) \cong D^{n-3}$ with interior $\mathcal T(P)$. [DAVIS2008] Example 6.5.3 {P} agrees that "If m > 3, there is a continuous family (moduli) of hyperbolic polygons with the same angles".

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

SymmHub, by V. Bulatov, C. Goodman-Strauss, and S. Vorthmann, is the only implementation found that builds a fundamental domain and generators for hyperbolic signatures with mirrors and cross-caps. Its orbifold engine is by Goodman-Strauss. Everything in this section is {C}. It describes what the code does, and no primary source that was read establishes the method. The tutorial attributes the cut rules to [DEATON1993], which was not read. `docs/research/symmhub.md` gives the line-level detail.

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

### 8.5. Cut Rules

`atomizeOrbifold` repeatedly splits the feature lists with `tube` cuts, from cone to cone with a length and a twist, and `slice` cuts, from a cone to a mirror or from corner to corner with a length only ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:452` {C}). The comments at lines 437 to 447 choose "option #2": cut the cone points out to the mirrors first, then cut the mirrors. That order needs few twist parameters and keeps the graph of cuts a tree.

The tutorial calls Deaton's rules "a small rewriting system on the notation itself", with "Nine types of geodesic cut", "each cut introducing a boundary token $\infty_\ell$ of length $\ell$" ([SYMMHUB] `tutorials/orbifold-notation.html` {C}). Its examples are $o \to \infty_\ell\infty_\ell$, $ABCD \to \infty_\ell AB + \infty_\ell CD$, and $*2345 \to *23\infty + *\infty 45$. The research did not recover five of the nine cut types, or the rules for cross-caps, cone pairs, and corners (gap G4).

### 8.6. The Twelve Atoms

The cuts end in "twelve types of atoms": four sheets `*abc`, four pillows `ABC`, and four pillowcases or hats `A*b` ([SYMMHUB] tutorial {C}). In each kind, the four types differ by how many vertices are real and how many are imaginary. A real vertex carries an angle `PI/n`. An imaginary vertex carries a length from `getLength(key)` and stands for a perpendicular geodesic edge of that length, so that a cut boundary becomes a truncated vertex. `unfoldAtom` has one case per atom type ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:648` {C}).

### 8.7. Generalized Law of Cosines

`edgeLength(p,q,r)` computes the edge between two vertices of a generalized triangle ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:588` {C}). For each vertex $v$, the code sets $(c_v, s_v) = (\cos v, \sin v)$ when $v$ is a real angle and $(c_v, s_v) = (\cosh v, \sinh v)$ when $v$ is an imaginary value, a length. It then computes:

$$\ell\ell = \frac{c_pc_q + c_r}{s_ps_q}$$

It returns $\operatorname{arccosh}(\ell\ell)$ when $p$ and $q$ are both real or both imaginary, and $\operatorname{arcsinh}(\ell\ell)$ otherwise.

Maskit's equation (1) for three hyperbolic boundaries and equation (8) for three cone points have the same algebraic form, and [MASKIT2001] Remarks 5.1 and 5.5 identify them as the right-angled hexagon formula and a hyperbolic law of cosines {P}. No primary source that was read states the mixed cases or the choice between $\operatorname{arccosh}$ and $\operatorname{arcsinh}$ (gap G3).

### 8.8. Pillow and Hat Formulas

A pillow is split into two generalized triangles by a "median". For three real cone points, the code computes ([SYMMHUB] `lib/orbilib/OrbifoldGeometrization.js:784` {C}):

- `A2 = asin(sqrt(1/(1+Math.pow(cot(p.re)+cos(q.re)/sin(p.re)/cos(r.re),2))))`
- `A1 = p.re-A2`
- `M = acosh(cos(q.re)/sin(A1))`
- `a1 = acosh(cos(A1)/sin(q.re))`
- `a2 = acosh(cos(A2)/sin(r.re))`

The imaginary analogues use `cosh(x/2)` (lines 829 to 833 and 880 to 884). For three imaginary vertices, line 936 computes `A2 = asinh(sqrt(1/(-1+Math.pow(coth(p.im/2)+cosh(q.im/2)/sinh(p.im/2)/cosh(r.im/2),2))))`. Hats use right-angled triangles with `HPI` and `PI/b[1]` (lines 1000 to 1130), and an imaginary cone uses the half length `q.im = getLength(b,group)/2`.

### 8.9. Layout and Walls

The vertices of each atom are placed in the Poincare disc by "turtle" moves, `poincareTurtleMove(last,current,angle,distance)` ([SYMMHUB] `lib/invlib/ComplexArithmetic.js:459` {C}). `useMirrorQ` (line 639 of `OrbifoldGeometrization.js`) marks which atom edges are real mirrors and which are `mirrorRedundant`.

A cutting edge of a pillow can carry a "wall", a perpendicular half-plane defined by two extra points (comments at lines 770 and 811 to 860). The assembled domain may therefore be non-convex, and the wall tells the GPU loop which generator to apply.

### 8.10. Assembly

`assembleFundamentalDomain` unfolds all atoms and welds them along open `tube` and `slice` attachments with `poincareMobiusEdgeToEdge([v1,v2],[w1,w2],shift)` ([SYMMHUB] `lib/invlib/ComplexArithmetic.js:472` {C}). For a tube, `shift` is the raw hyperbolic twist distance, applied as `transX = tanh(shift/2)`. Tube and slice edges vanish from the final polygon (lines 1305 to 1335 of `OrbifoldGeometrization.js`).

The code recentres the domain twice by moving the Euclidean centroid of its vertices to the origin, "to keep the domain from growing off into a numerically less stable regime, closer to the boundary of the Poincare disk" (line 1137). It then moves the domain by `(-.001,-.002)` into general position (lines 1351 to 1370).

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
| Length when a key index exceeds `parCounts` | 1.1 | `lib/orbilib/OrbifoldGeometrization.js:626-627` |
| Drag speed of a length | 0.005 | `lib/orbilib/OrbifoldUITool.js:61-69` |
| Drag speed of a twist | 0.002 | `lib/orbilib/OrbifoldUITool.js:61-69` |

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

SymmHub places a triangle with angles `a0`, `a1`, and `a2` in the disc with these sides ([SYMMHUB] `lib/grouplib/Group_KLM.js:371-385` {C}):

- `iPlane([0,-1,0,0])`, the real axis.
- `iPlane([-sin(a0),cos(a0),0,0])`.
- The circle `iSphere([r*ax, r*ay, 0, -r])`, with `ax=(cos(a2)+cos(a1)*cos(a0))/sin(a0)`, `ay=cos(a1)`, and `r=1/sqrt(ax*ax+ay*ay-1)`.

The spherical case uses `r = e2*sin(a0)/(cos(a2)+cos(a1+a0))` (lines 340 to 346), and the Euclidean case uses `iPlane([sin(a1),cos(a1),0,e])` (line 355). The code chooses the geometry by the angle sum with `EPSILON=1e-6` (lines 296 to 310).

### 9.2. Regular Tilings and Polygons

A regular tiling $\{n, k\}$ is hyperbolic when "$2\pi n/k < (n - 2)\pi \iff 1/n + 1/k < 1/2$" ([SOLOMYAK2020] slide 8 {P}). The distance from the centre to a vertex of the regular polygon in the disc is $r_0 = \sqrt{\cos(\pi/p + \pi/q)/\cos(\pi/p - \pi/q)}$ for the tiling $\{p, q\}$ ([ARXIV-2105.01087] equation 9, [ARXIV-2309.10844] equation 25 {S}).

### 9.3. Trigonometric Formulas

`docs/research/geometry-models.md` covers hyperbolic trigonometry. These formulas appear in the construction:

- The angle form of the law of cosines: [MASKIT2001] equation (8) and Remark 5.5 {P}.
- The right-angled hexagon formula: [MASKIT2001] equation (1) and Remark 5.1 {P}.
- The side form $\cosh c = \cosh a\cosh b - \sinh a\sinh b\cos\gamma$: [ARXIV-1410.6735] {S}.
- The Lambert quadrilateral, $\cos\angle F = \sinh OA\,\sinh OB$: Wikipedia, page not recorded {S}.

### 9.4. Polygon Reflection Groups

The signatures `*p1...pn` have $n - 3$ free parameters ([THURSTON2002] Theorem 13.8.1 {N}). [DAVIS2008] Example 6.5.3 {P} guarantees a convex realization in the hyperbolic plane whenever $\sum 1/m_i < m - 2$, and states that for $m > 3$ "there is a continuous family (moduli) of hyperbolic polygons with the same angles". Neither source gives coordinates for the family. SymmHub produces one by cutting the polygon with slices into sheets, with one length per slice (Section 8.5).

### 9.5. The 17 Euclidean Groups

The signatures are verified ([CONWAY-HUSON2002] Table II, [THURSTON2002] Theorem 13.3.6 {N}). The crystallographic (IUC) names rest on secondary sources that the research did not record {S}. The SymmHub catalogue builds each group from a function in `lib/orbilib/WallpaperGroups_NonNegative.js` ([SYMMHUB] {C}). The free parameters in the last column come from [VONGAGERN2009] p. 7 {P}.

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

[VONGAGERN2009] Section 6.4, p. 28, excludes p1, pm, pg, and cm because "They do not have any rotation centers." The square and hexagonal groups are rigid in SymmHub, with hard-coded positions such as `d = a*sqrt(2.)/4.` (line 202). [CONWAY-HUSON2002] {N} states that each Euclidean signature names a single group "up to isotopy (i.e., continuous variation)". No primary source gives a deformation dimension for every Euclidean group (gap G7).

[VONGAGERN2009] p. 7 {P} also relates the groups to reflection groups: "The groups p4m, p3m1 and p6m correspond to the three Euclidean triangular kaleidoscopic fundamental regions with corner angles (90, 45, 45), (60, 60, 60) and (90, 60, 30), respectively. The groups p4, p3 and p6 are the corresponding reflection free subgroups." The groups p31m and p4g are index-two subgroups of p6m and p4m. The rectangle group pmm "corresponds to a rectangular kaleidoscope with four 90 corners."

[VONGAGERN2009] p. 12 {P} lists fundamental regions built from triangles that carry over to hyperbolic relatives when the orders change:

| Region | Groups | Shape and angle condition |
| --- | --- | --- |
| Reflected triangle | p6m, p3m1, p4m | Angles $\pi/k$, $\pi/m$, $\pi/n$. "Any integral values that fulfill the angle sum inequality for a hyperbolic triangle are valid." |
| Rotated kite | p6, p3, p4 | A deltoid with two equal angles $\pi/k$ and the angles $2\pi/m$ and $2\pi/n$ on its axis |
| Reflected kite | p31m, p4g | The symmetric angle is a right angle, with the angles $\pi/m$ and $2\pi/n$ on the axis. "The angle sum inequality has to be fulfilled for one half of the kite." |

The same page states that "the fundamental cells of the groups p3, p4, p6, p31m, p4g consist of two copies of the triangles for p3m1, p4m or p6m." For pmm, p. 14 states that "every corner angle must be a divisor of $\pi$", and that the hyperbolic length of one side together with the corner angles determines the tile. The hyperbolic triangle groups of p. 8 are those with angles $\pi/k$, $\pi/m$, $\pi/n$ and $1/k + 1/m + 1/n < 1$.

### 9.6. The 14 Spherical Families

The signatures and orders are verified ([CONWAY-HUSON2002] Table III, [THURSTON2002] Theorem 13.3.6 {N}), with order $2/\chi$. The Schoenflies names rest on secondary sources that the research did not record {S}.

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

Generators exist only as SymmHub catalogue code, in the stereographic plane with the equator at the unit circle and fixed constants, such as `iSphere([-1.61803398874989,0,0,1.90211303259031])` for `532` ([SYMMHUB] `lib/orbilib/WallpaperGroups_NonNegative.js:711` {C}). The reflection triangles of the spherical reflection groups are those of Section 9.1, and [DAVIS2008] Theorem 6.4.3 {P} certifies them as strict fundamental domains.

### 9.7. Mixed Signatures

[CONWAY-HUSON2002] {N} gives the presentation of every mixed signature, such as `p*q` and `2*pq` (Section 5.2). SymmHub builds the geometry with the hat atom `A*b` ([SYMMHUB] {C}). No primary geometric construction was read (gap G8).

## 10. Algorithms

### 10.1. Dirichlet Domains

[VOIGHT2009] Section 1 {P} defines the Dirichlet domain $D(p) = \{z \in \mathbb H : d(z, p) \le d(gz, p) \text{ for all } g \in \Gamma\}$, "a hyperbolically convex fundamental domain". A "random" point $p$ has trivial stabilizer "with probability 1". The map $\phi(z) = \frac{z - p}{z - \bar p}$ moves $p$ to $0$ and $\Gamma$ into $PSU(1,1)$.

Proposition 1.3 states that $D(0)$ is the closure of $\bigcap_{g \ne 1}\mathrm{ext}(I(g))$, and that $d(z, 0)$ is less than, equal to, or greater than $d(gz, 0)$ as $z$ lies in $\mathrm{ext}(I(g))$, on $I(g)$, or in $\mathrm{int}(I(g))$. Corollary 1.4 states that $gI(g) = I(g^{-1})$.

Remark 1.5 gives the bisector for the base point $p = i$ as the half-circle "of radius $(a^2 + b^2 + c^2 + d^2 - 2)/(a^2 + c^2 - 1)^2$ centered at $(ab + cd)/(a^2 + c^2 - 1)$". That expression is as printed and may contain a typo (gap G9).

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

Proposition 5.4, citing [BEARDON] Theorem 9.4.5, states that for $p$ outside a set of area zero:

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

Voight's termination argument covers the rule that applies the generator that most decreases the distance to the centre. It does not cover the rule that applies the first violated side. A termination argument for the greedy rule, for convex reflection domains and for general side-pairing polygons, is gap G5. [DAVIS2008] contains no algorithm that folds a point across walls and no bound on the number of steps {P}.

### 10.8. Kleinian Analogue

[PAGE2012] Algorithm 2 and Proposition 10 {S} reduce a point for a Kleinian group by the same strict-decrease argument: "while the algorithm runs the distance $d(w', 0)$ decreases ... this orbit is discrete, the algorithm terminates". Proposition 32 bounds the effect of floating point with a factor $1 + \beta$.

### 10.9. Verifying Discreteness

These checks are available:

- The conditions of Poincare's theorem, Section 4.8, which certify discreteness for the classes that Section 4.7 does not exclude {P}.
- The negative trace theorem for each pair of pants, Section 7.7 ([MASKIT2001] Theorem 4.1 {P}).
- Jorgensen's inequality, which is a necessary condition only {S}.
- SymmHub's numerical check of every relator in `GroupPresentation.js`, which throws when the domain does not tile (Section 4.6, {C}).

## 11. Delaney-Dress Symbols

This section is a survey for orientation. [TEGULA2020] {N} defines a Delaney-Dress symbol as "a finite set $\mathcal D$, together with the action of a certain free group $\Sigma$, together with maps $m_{01}, m_{12}, m_{02} : D \to \mathbb N$". Its nodes are classes of chambers.

The orbifold is read from the symbol through the relation $m_{ij} = v_{ij}r_{ij}$. "Reflections are mapped onto boundary segments, rotational centers are mapped onto cones", and "glide-reflections give rise to crosscaps". The curvature is $\mathcal K = \sum_D(1/m_{01} + 1/m_{12} - 1/2)$, and its sign gives the geometry ([TEGULA2020] p. 6 {N}).

Tegula, a Java program under GPL-3.0, implements these symbols. Its canonicalization in `OrbifoldGroupName.java` is known only from a secondary code reading {S}.

## 12. Conflicts Between Sources

Each conflict is stated with the recommendation of the research where it gives one.

- Self-paired sides: [SOLOMYAK2020] Theorem 12 excludes them. [SOLOMYAK2020] slide 22, [VOIGHT2009] Section 1, and [SERIES2013] Definition 6.1 handle a side paired with itself by a half-turn by splitting it at the fixed point {P}. [JESPERS2015] {S} and SymmHub {C} allow a reflection to pair a side with itself, with relator $w^2 = 1$. The research concludes that these agree once half-turns and reflections are kept apart.
- Completeness against parabolic cycles: [MASKIT2001] Section 4.4 and [SERIES2013] condition (b) {P} both require parabolic cycle transformations at ideal vertices, as does de Rham {S}. [MASKIT1971], which was not read, uses a completeness condition according to secondary citations. [VOIGHT2009] Proposition 1.1 relies on finite area instead. The parabolic form is verified in two primary sources.
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
| Rotation triangle groups `pqr` | None | Fully sourced |
| Polygon reflection groups `*p1...pn` | G3 and G4 for explicit coordinates, G1 for ideal corners with more than three sides | Existence, discreteness, and parameter count are sourced for compact polygons. Coordinates are not. |
| Orientable hyperbolic, no mirrors | G6 for a canonical polygon, G11 for the twist convention | Generators are sourced, except for the choice of twist convention |
| Mixed signatures with cone points and mirrors | G1, G2, G3, G4, G8 | Only SymmHub code |
| Several mirror boundaries | G1, G2, G3, G4, G15 | Only SymmHub code |
| Mirrors together with handles | G1, G2, G3, G4 | Only SymmHub code |
| Non-orientable, with cross-caps | G1, G2, G3, G4 | Only SymmHub code |
| The 17 Euclidean groups | G7 | Catalogue code, with partial moduli counts |
| The 14 spherical families | G8 | Reflection triangles are sourced. The other generators are catalogue code. |
| Bad orbifolds | None | No group exists |
| Any class, point reduction with the greedy rule | G5 | Termination unproven |

### 13.2. Gaps

- G1, Poincare's theorem for orientation-reversing groups: The four open cases are those of Section 4.7. The cycle condition that [VOIGHT2009] Proposition 1.1 leaves to [BEARDON] Theorem 9.8.4 is also unread. Candidate sources: [EPSTEIN-PETRONIO1994], [RATCLIFFE] Theorem 11.2.2, and [BEARDON] Section 9.8.
- G2, matrices for pieces with mirrors and cross-caps: [MASKIT2001] covers orientable pieces only. Needed: an extension that uses the reflection matrices of [MASKIT2001] Section 4.3, and a gluing of one-sided geodesics by glide reflections. [NORBURY2006] {S} states that the gluing is antipodal with no twist. SymmHub's cap construction (Section 8.11) is the only explicit one, and it is unproven. Candidate sources: [CHOI2012], [BUSER].
- G3, the generalized law of cosines with imaginary angles: SymmHub's `edgeLength` mixes real angles with truncation lengths. Maskit's continuation $\lambda = i\pi/\alpha$ covers cone points inside pants matrices. The truncated-vertex case and the rule that chooses between $\operatorname{arccosh}$ and $\operatorname{arcsinh}$ are not verified against a primary source. Candidate sources: [BUSER] Formula Glossary 2.2 to 2.4, [RATCLIFFE] Section 3.5.
- G4, Deaton's rules: Five of the nine cut types, the rules for cross-caps, cone pairs, and corners, and the claim that the twist appears in exactly three situations rest only on SymmHub. Candidate source: [DEATON1993].
- G5, termination of the greedy reduction: Section 10.7 states the gap. Needed: a termination argument for convex reflection domains and for general side-pairing polygons.
- G6, geometry of the canonical Fricke polygon: The polygon with the edge word of equations (5.1) and (5.2), with elliptic and parabolic vertices. [VOIGHT2009] gives only the presentation and defers to [IMBERT2001].
- G7, Euclidean moduli: [VONGAGERN2009] p. 7 gives the counts for p2, pmm, pmg, pgg, and cmm. The paper does not state whether a parameter of a "family of conjugacy classes" includes the overall scale, and it gives no counts for p1, pm, pg, and cm or for the rigid groups. Whether Thurston's formula extends to $\chi = 0$ is unverified. Candidate sources: [SYMMETRIES-OF-THINGS2008], [KULKARNI-LEE-RAYMOND1985], [SCOTT1983].
- G8, geometric constructions for mixed signatures and spherical generators: `p*q`, `2*pq`, spheres with cone points, and the generators of the spherical groups have only presentations {N} and catalogue code {C}. Candidate source: [SYMMETRIES-OF-THINGS2008].
- G9, the bisector radius of Voight Remark 1.5: The printed expression needs a check by direct computation, or another source, before use.
- G10, the signature of the orientation-preserving subgroup in Conway terms: It rests on secondary sources only. Candidate source: [SINGERMAN1974].
- G11, the twist convention for Escher: The three conventions of Section 2.5 are all in use. No source chooses one for an implementation that also has mirrors and cross-caps.
- G12, the Maskit worked example: The Section 11 matrix for $\tilde e_{3,3}$ does not match equation (19), and needs a check by computation before it serves as a test case.
- G13, the dimensions of the A-type pieces of [CHOI-GOLDMAN2001]: Not verified.
- G14, the chapter number in [CHOI-GOLDMAN2001]: Its citation of chapter 5 of Thurston's notes is unverified.
- G15, reflection-generated groups with several boundaries: Which signatures with several boundaries, such as `**`, give groups generated by reflections. [CONWAY-HUSON2002] suggests that they need generators other than reflections, but no source states it.

### 13.3. Candidate Statements That Need a Source

These statements appear in the research or in code, and no cited source states them. Each needs a source or a machine-checked proof before the body can use it as a fact.

1. SymmHub's `edgeLength` formula of Section 8.7 gives the correct edge length in every mixed case of real and imaginary vertices.
2. SymmHub's cap generator of Section 8.11 is a glide reflection that, together with the other side pairings, satisfies the hypotheses of a polygon theorem for NEC groups.
3. The greedy reduction of Section 10.7 terminates for every point and every domain that SymmHub builds.
4. The twelve atoms of SymmHub correspond one to one to the twelve elementary orbifolds of [CHOI-GOLDMAN2001].
5. The twist appears in exactly three situations, as the SymmHub tutorial states.

### 13.4. Sources Not Read in Full

These sources could not be read in full. `docs/research/source-requests.md` records each one with what the project needs from it.

| Priority | Source | Needed for |
| --- | --- | --- |
| High | [EPSTEIN-PETRONIO1994], the chapter "Conditions for Poincare's Theorem" from p. 125 and Theorem 8.1. The PDF is blocked by a bot check. | G1 |
| High | [RATCLIFFE] Theorem 11.2.2, Section 3.5, Sections 7.1 to 7.3, and chapter 13. Paywall. | G1, G3 |
| High | [DEATON1993], unpublished, held as a scan in the Mudd Manuscript Library | G4 |
| Medium | [BEARDON] Theorem 9.8.4, Theorem 9.4.5, and Sections 9.3 to 9.4. Paywall. | G1 |
| Medium | [BUSER] Formula Glossary 2.2 to 2.4, Sections 3.1 and 3.3. Paywall. | G2, G3 |
| Medium | [CHOI2012], elementary pieces with mirrors and their deformation spaces | G2, G13 |
| Medium | [SINGERMAN1974] | G10 |
| Medium | [SYMMETRIES-OF-THINGS2008] | G7, G8 |
| Low | [SERIES2013], read only in Section 6.1 and Sections 6.1.5 to 6.1.6. The proof in Section 7.6 is unread. | Proof of Theorem 6.14 |
| Low | [IMBERT2001] | G6 |
| Low | [MASKIT1971], which returned HTTP 403 | The original completeness condition |
| Low | [MACBEATH1967] and [WILKIE1966] | The primary NEC text, largely superseded by [CONWAY-HUSON2002] |
| Low | [CFKP], available locally but not read | Formulas of models and isometries, needed for no claim here |
| Low | [GILMAN-MASKIT1991] | The two-generator discreteness algorithm |
| Low | [BULATOV2011], the talk and paper PDFs | Bulatov's own account beyond the SymmHub code |
| Low | Dress 1987, Delgado-Friedrichs 2003, Huson 1993, and Balke-Huson 1996, whose full entries the research did not record | Detail on Delaney-Dress symbols |

### 13.5. Assumptions and Bibliographic Gaps

These assumptions were made in writing this document:

- The reading of Maskit's typography, that $\tilde a$ is a matrix and $a$ its map, follows the usage in the research report and was not checked against the paper.
- The definitions of "Fuchsian group" and "NEC group" in Section 2.4 summarize how the sources use the terms. No source that was read was quoted for them.
- The years of the secondary arXiv sources in Section 14.2 are read from their arXiv identifiers.
- [VONGAGERN2009] was read at the cited pages, so its claims carry {P}. The research did not state whether the whole paper was read.

The research did not record the titles of several secondary sources, so their entries in Section 14.2 give the identifier only. The Wikipedia pages for the twist convention, the Lambert quadrilateral, and the IUC, Schoenflies, and Coxeter names are not identified.

## 14. References

### 14.1. Sources Read

- [CONWAY-HUSON2002] Conway, J. H. and D. H. Huson, "The Orbifold Notation for Two-Dimensional Groups", Structural Chemistry 13(3/4), 247-257, 2002, DOI 10.1023/A:1015851621002, <https://doi.org/10.1023/A:1015851621002>.
- [DAVIS2008] Davis, M. W., "The Geometry and Topology of Coxeter Groups", 2008. Locators give printed pages. The PDF page is the printed page plus 16.
- [FIELD-GUIDE2018] Conway, J. H., Doyle, P., Gilman, J., and W. Thurston, "Geometry and the Imagination in Minneapolis", 1991 handouts, arXiv:1804.03055v1, 2018, <https://arxiv.org/abs/1804.03055>.
- [MASKIT2001] Maskit, B., "Matrices for Fenchel-Nielsen coordinates", Annales Academiae Scientiarum Fennicae Mathematica 26, 267-304, 2001, <http://www.acadsci.fi/mathematica/Vol26/maskit.pdf>.
- [SERIES2013] Series, C., with assistance from S. Maloni, "Hyperbolic Geometry MA 448", lecture notes, University of Warwick, 2008, revised 2010, last edited 2013-03-01. The printed page is the PDF page minus 17.
- [SOLOMYAK2020] Solomyak, B., "Lecture 9: Examples of lattices, Poincare's Theorem", lecture slides, Bar-Ilan University, 2020-12-20.
- [SYMMHUB] Bulatov, V., Goodman-Strauss, C., and S. Vorthmann, "SymmHub", source code, commit f2d19cb, MIT licence, 2025, <https://github.com/SymmHub/SymmHub>. Paths are relative to the root of the repository.
- [TEGULA2020] Zeller, R., Delgado Friedrichs, O., and D. H. Huson, "Tegula - exploring a galaxy of two-dimensional periodic tilings", arXiv:2007.10625v1, 2020, <https://arxiv.org/abs/2007.10625>.
- [THURSTON2002] Thurston, W. P., "The Geometry and Topology of Three-Manifolds", chapter 13, electronic version 1.1 of the 1980 notes, March 2002, <http://www.msri.org/publications/books/gt3m/>, chapter PDF <https://library.slmath.org/nonmsri/gt3m/PDF/13.pdf>.
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

- [BEARDON] Beardon, A. F., "The Geometry of Discrete Groups", Graduate Texts in Mathematics 91, ISBN 978-0-387-90788-8.
- [BULATOV2011] Bulatov, V., talk of 2011 and "Bending Hyperbolic Kaleidoscopes", Bridges 2011, <http://bulatov.org/math/1101/>, <http://bulatov.org/math/1107/>.
- [BUSER] Buser, P., "Geometry and Spectra of Compact Riemann Surfaces", ISBN 978-0-8176-4992-0.
- [CFKP] Cannon, J. W., Floyd, W. J., Kenyon, R., and W. R. Parry, "Hyperbolic Geometry".
- [CHOI2012] Choi, S., "Geometric Structures on 2-Orbifolds", MSJ Memoirs 27, 2012.
- [DEATON1993] Deaton, A. A., "Hyperbolic Orbifolds and Patterns", senior thesis, Princeton University, 1993, unpublished.
- [EPSTEIN-PETRONIO1994] Epstein, D. B. A. and C. Petronio, "An exposition of Poincare's polyhedron theorem", L'Enseignement Mathematique 40, 113-170, 1994, zbMATH 0808.52011.
- [GILMAN-MASKIT1991] Gilman, J. and B. Maskit, Michigan Mathematical Journal 38, 13-32, 1991.
- [IMBERT2001] Imbert, M., "Calculs de presentations de groupes fuchsiens via les graphes rubanes", Expositiones Mathematicae 19, 213-227, 2001.
- [KULKARNI-LEE-RAYMOND1985] Kulkarni, R. S., Lee, K. B., and F. Raymond, "Deformation spaces for Seifert manifolds", Lecture Notes in Mathematics 1167, Springer, 1985.
- [MACBEATH1967] Macbeath, A. M., Canadian Journal of Mathematics 19, 1192-1205, 1967, DOI 10.4153/CJM-1967-108-5, <https://doi.org/10.4153/CJM-1967-108-5>.
- [MASKIT1971] Maskit, B., "On Poincare's theorem for fundamental polygons", Advances in Mathematics 7, 219-230, 1971, DOI 10.1016/S0001-8708(71)80003-8, <https://doi.org/10.1016/S0001-8708(71)80003-8>.
- [RATCLIFFE] Ratcliffe, J. G., "Foundations of Hyperbolic Manifolds", 3rd edition, ISBN 978-3-030-31596-2, DOI 10.1007/978-3-030-31597-9_11, <https://doi.org/10.1007/978-3-030-31597-9_11>.
- [SCOTT1983] Scott, P., "The geometries of 3-manifolds", Bulletin of the London Mathematical Society 15, 401-487, 1983, DOI 10.1112/blms/15.5.401, <https://doi.org/10.1112/blms/15.5.401>.
- [SINGERMAN1974] Singerman, D., "On the structure of non-Euclidean crystallographic groups", Proceedings of the Cambridge Philosophical Society 76, 233-240, 1974, DOI 10.1017/S0305004100048891, <https://doi.org/10.1017/S0305004100048891>.
- [SYMMETRIES-OF-THINGS2008] Conway, J. H., Burgiel, H., and C. Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1-56881-220-5.
- [WILKIE1966] Wilkie, H. C., Mathematische Zeitschrift 91, 87-102, 1966, DOI 10.1007/BF01110157, <https://doi.org/10.1007/BF01110157>.
