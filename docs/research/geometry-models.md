# Models of the Hyperbolic Plane, the Sphere, and the Euclidean Plane

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-15 |
| Applies to | The representation of points and isometries in Escher, the model maps, the geometric constructions, and the distance computations |

## Abstract

This document collects the mathematics that Escher needs to represent points and isometries in the three planar geometries and to draw a tiling in each model of them. It gives the domains, metrics, distances, and geodesics of the hyperboloid, Poincare disk, half-plane, Klein, hemisphere, and band models, the maps between them, and the further conformal models that Vladimir Bulatov describes. It gives the matrix, Mobius, inversion, and Clifford representations of hyperbolic isometries with their classification, the standard constructions, the measured precision of each representation, the formulas for circles and scale factors, the trigonometry for building triangles and polygons, and the corresponding material for the sphere and the Euclidean plane. Each formula carries its source and the strength of the evidence for it, and each place where the sources disagree is stated with its conventions.

## 1. Scope

This document covers the models of the hyperbolic plane, the sphere, and the Euclidean plane, the representations of their isometries, the constructions that Escher builds from them, the trigonometry for building fundamental triangles and polygons, and the numerical behaviour of each representation. For the sphere, it also covers ray and sphere intersection, camera rays, projection matrices, and the Phong and Blinn-Phong shading models, because the first renderer draws spheres with Phong shading.

The rendering algorithms are out of scope. The folding of each pixel into a fundamental domain, the drawing of borders, antialiasing, and the per-pixel application of the model maps are in `docs/research/rendering.md`. The structure of the SymmHub source code is in `docs/research/symmhub.md`. This document states a SymmHub fact only where it bears on a formula, a construction, or numerical behaviour.

`docs/research/source-requests.md` lists the sources that nobody could read in full, and Section 15.3 names them.

## 2. Notation and Conventions

### 2.1. Evidence Status

Every claim in the body carries a citation tag, a locator inside the source, and one of these status markers:

- (P): Verified against a primary source read in full, such as a local copy of a paper, an author's page, or a source file.
- (P-code): Verified against source code read in full. The citation gives the file and the lines or function.
- (A): Taken from a research paper read only through a summarizing web fetch. The formula is as the summary gave it.
- (S): Taken from a secondary source, such as Wikipedia, MathWorld, or a tutorial. Such a claim rests only on that secondary source.
- r: Added to a marker, as in (S, r), when a fetch tool reconstructed the formula and nobody has checked it against the page since.

Names with accented letters are written in ASCII transliteration, such as Poincare, Mobius, Celinska-Kopczynska, and Kopczynski. Quotations from sources are also transliterated to ASCII, with Greek letters written as LaTeX commands and an elided passage written as "...". All web sources were accessed on 2026-11-15. Wikipedia and MathWorld URLs are formed from the page titles that the research reports name, and Wikipedia pages carry no revision identifier.

### 2.2. Symbols

- $K$: Gaussian curvature. Every hyperbolic formula uses $K = -1$ unless it says otherwise. HT writes formulas with a curvature radius $\ell$, and setting $\ell = 1$ gives $K = -1$ [HT], Section 4.
- $z, w$: Complex coordinates in a two-dimensional model. $\bar z$ is the complex conjugate, and $\operatorname{Im} z$ is the imaginary part.
- $d(p, q)$: The hyperbolic, spherical, or Euclidean distance between two points.
- $\operatorname{artanh}$, $\operatorname{arcosh}$, $\operatorname{arsinh}$: The inverse hyperbolic functions. Sources also write them $\tanh^{-1}$, $\cosh^{-1}$, and $\sinh^{-1}$.
- $x \ast y$: The Minkowski bilinear form in the convention of [CFKP], Section 3, p. 64, which puts the time coordinate last: $x \ast y = x_1 y_1 + \dots + x_n y_n - x_{n+1} y_{n+1}$.
- $J$: The matrix of that form, $J = \operatorname{diag}(1, \dots, 1, -1)$.
- $g^-$, $g^+$: The Minkowski form and the Euclidean dot product as [KC24], p. 3, writes them, also with the time coordinate last.
- $B(x, y)$: The Minkowski form in the convention of [W-HYPERBOLOID], which puts the time coordinate first: $B(x, y) = -x_0 y_0 + x_1 y_1 + x_2 y_2$.
- $C_0$: The origin of a linear model, $(0, \dots, 0, 1)$ in [KC24].
- $\lambda(z)$: The scale factor of a conformal model, so that $ds = \lambda(z) \lvert dz \rvert$.
- $\ell_g$: The translation length of an isometry $g$, as [MAR] writes it.
- $\{p, q\}$: The regular tiling by $p$-gons with $q$ meeting at each vertex.

### 2.3. Terminology

- model: A domain in $\mathbb{R}^n$ or $\mathbb{C}$ with a metric of constant curvature, together with the map that identifies it with the geometry.
- conformal model: A model whose metric is a scalar multiple of the Euclidean metric at each point, so that it preserves angles.
- scale factor: The factor $\lambda$ of a conformal model, also called the conformal factor.
- geodesic: A curve that locally minimizes length. In the hyperbolic plane each geodesic has two ideal endpoints.
- ideal point: A point on the boundary at infinity of a hyperbolic model, such as a point of the unit circle in the Poincare disk.
- hypercycle: A curve at constant distance from a geodesic, also called an equidistant curve.
- horocycle: A curve that is the limit of circles whose centres move to one ideal point.
- isometry: A map that preserves distance. An isometry is orientation-preserving or orientation-reversing.
- Mobius transformation: A map $z \mapsto (az + b)/(cz + d)$ with $ad - bc \ne 0$.
- anti-Mobius transformation: A map $z \mapsto (a\bar z + b)/(c\bar z + d)$, which reverses orientation.
- Lorentz matrix: A matrix that preserves the Minkowski form.
- boost: A Lorentz matrix that translates along a geodesic through $C_0$.
- elliptic, parabolic, hyperbolic, loxodromic: The classes of an orientation-preserving isometry that Section 6.6 defines.
- glide reflection: A reflection in a line composed with a translation along that line.
- translation length: The infimum of $d(gz, z)$ over all points $z$, for an isometry $g$.
- Clifford representation: The representation of an isometry as an element of a Clifford algebra, as [KC24], p. 5, uses.
- representation variant: The normalization policy that [KC24], Section 3, applies to a representation, such as invariant or forced.

### 2.4. Conventions of This Document

Every formula is given in the convention of its source, and the citation names that source. This document converts no formula from one convention to another, because a conversion would be a derivation that no source checks. An implementer who mixes formulas from two sources converts them through the conventions in this section first.

The main sources differ in four conventions. This table lists them:

| Source | Time coordinate | Sign of the Minkowski form | Matrix normalization | Band strip |
| --- | --- | --- | --- | --- |
| [CFKP] | Last | $x \ast x = -1$ on the hyperboloid | Lorentz matrix with positive last entry of last column | No band model |
| [KC24] | Last | $g^-(x, x) = -1$ on the hyperboloid | $SL(2, \mathbb{R})$ with $ad - bc = 1$ on the half-plane | Band named, no formula |
| [HR] | Last | Signature table with $-1$ last | Matrices renormalized by Minkowski Gram-Schmidt | Not read |
| [HT] | First (equation 1) | Not restated here | $SU(1,1)$, $\begin{pmatrix} a & b \\ \bar b & \bar a \end{pmatrix}$, $\lvert a \rvert^2 - \lvert b \rvert^2 = 1$ | No band model |
| [W-HYPERBOLOID] | First | $(-, +, +)$ | $O^+(1, 2)$ preserves the sign of the first coordinate | Not applicable |
| [OBL] | First | $\eta = \operatorname{diag}(-1, 1, 1)$ | $SL(2, \mathbb{R})$ | Not applicable |
| [CHO] | Not recorded | Opposite sign, spacelike vectors have negative norm | Not applicable | Not applicable |
| [W-HMOTION] | Not applicable | Not applicable | $\begin{pmatrix} a & b^* \\ b & a^* \end{pmatrix}$, with $b$ and $\bar b$ swapped relative to [HT] | Not applicable |
| [B2], [B3], [STRIP], [SYMMHUB] | Not applicable | Not applicable | Not applicable | $\lvert \operatorname{Im} w \rvert < 1$ |
| [KC19] | Not applicable | Not applicable | Not applicable | $0 < \operatorname{Im} z < 1$ |
| [W-BAND] | Not applicable | Not applicable | Not applicable | $\lvert \operatorname{Im} z \rvert < \pi/2$ |

The isometries also differ in how a translation distance enters a matrix. The $SL(2, \mathbb{R})$ and $SU(1,1)$ matrices use half the distance, as in $\cosh(\theta/2)$, while the Lorentz matrices use the full distance ([HT], equations 15-16, [CF] Section 2.1, [OBL] Section 2.2.2, and [W-HYPERBOLOID]). An $SL(2, \mathbb{R})$ rotation by $\theta$ becomes a Lorentz rotation by $2\theta$ [OBL], Section 2.2.2 (A).

Section 14 lists every conflict between sources, including the band constant and the map from the disk to the half-plane.

## 3. Models of the Hyperbolic Plane

### 3.1. Domains, Metrics, Distances, and Geodesics

[CFKP], Section 7, pp. 70-71, defines five models in dimension $n$, with the time coordinate last (P). This table gives each model for the plane, with the metric from [CFKP] and the distance and geodesics from the cited source:

| Model | Domain | Metric $ds^2$ | Distance | Geodesics |
| --- | --- | --- | --- | --- |
| Hyperboloid | $\lbrace x \ast x = -1, x_{n+1} > 0 \rbrace$ | $dx_1^2 + \dots + dx_n^2 - dx_{n+1}^2$ | $a \ast b = -\cosh d(a, b)$ [CFKP], Lemma, p. 83 (P) | Intersections with two-dimensional linear subspaces [CFKP], p. 80 (P) |
| Poincare disk | $\lbrace (x, 0) : \lvert x \rvert < 1 \rbrace$ | $\dfrac{4 \lvert dx \rvert^2}{(1 - \lvert x \rvert^2)^2}$ | $d(w, z) = 2 \operatorname{artanh} \dfrac{\lvert z - w \rvert}{\lvert 1 - z \bar w \rvert}$ [HT], equation 23 (P) | Diameters and arcs of circles orthogonal to the boundary [CFKP], p. 80 (P) |
| Half-space (half-plane) | $\lbrace (1, x_2, \dots, x_{n+1}) : x_{n+1} > 0 \rbrace$ | $\dfrac{dx_2^2 + \dots + dx_{n+1}^2}{x_{n+1}^2}$ | $2 \operatorname{arsinh} \dfrac{\lvert p_2 - p_1 \rvert}{2 \sqrt{y_1 y_2}}$ [W-HALFPLANE] (S) | Vertical lines and semicircles orthogonal to the boundary [CFKP], Theorem 9.3 (P) |
| Hemisphere | $\lbrace \lvert x \rvert = 1, x_{n+1} > 0 \rbrace$ | $\dfrac{dx_1^2 + \dots + dx_{n+1}^2}{x_{n+1}^2}$ | Not given | Not given separately |
| Klein | $\lbrace (x, 1) : \lvert x \rvert < 1 \rbrace$ | $\dfrac{\lvert dx \rvert^2}{1 - \lvert x \rvert^2} + \dfrac{(x \cdot dx)^2}{(1 - \lvert x \rvert^2)^2}$ | $\frac{1}{2} \ln \dfrac{\lvert aq \rvert \lvert pb \rvert}{\lvert ap \rvert \lvert qb \rvert}$ [W-KLEIN] (S) | Straight chords [CFKP], Section 9 (P) |

These notes qualify the table:

- [CFKP], pp. 71-72, proves that the connecting maps of Section 4.1 are isometries by pulling the metric back (P).
- In the Klein distance, $a$ and $b$ are the ideal endpoints of the chord through $p$ and $q$. "The factor of one half is needed to make the curvature -1" [W-KLEIN] (S).
- The Klein model is not conformal [W-KLEIN] (S), and its metric is anisotropic [CFKP], p. 71 (P).
- In the half-plane formula, $y_1$ and $y_2$ are the imaginary parts of $p_1$ and $p_2$ [W-HALFPLANE] (S).
- The band model is in Section 5.

### 3.2. Further Distance Formulas

These formulas give the distance in special positions, or in another form:

| Model | Formula | Source |
| --- | --- | --- |
| Disk, from the origin | $\rho = \ln \dfrac{1 + R}{1 - R}$, $R = \dfrac{e^\rho - 1}{e^\rho + 1} = \dfrac{\cosh \rho - 1}{\sinh \rho}$ | [CFKP], Fact 5, p. 93 (P). [HT], equation 22 gives the same (P) |
| Disk, general | $\operatorname{arcosh}\left(1 + \dfrac{2 \lvert u - v \rvert^2}{(1 - \lvert u \rvert^2)(1 - \lvert v \rvert^2)}\right)$ | [W-DISK] (S) |
| Half-space, along a semicircle from angle $\varphi$ to $\theta$ | $d = \displaystyle\int_\varphi^\theta \frac{d\psi}{\sin \psi} = \ln \frac{\sin \psi}{1 + \cos \psi} \Big\vert_\varphi^\theta$ | [CFKP], Fact 4, p. 90 (P) |
| Half-plane, on a vertical line from height $u$ to $v$ | $\log(v / u)$ | [CAM], Section 4.3 (A) |
| Klein, from the origin | $\operatorname{artanh} r$ | [W-KLEIN] (S) |
| Polar coordinates $(r_1, \varphi_1)$ and $(r_2, \varphi_2)$ | $\cosh d = \cosh(r_1 - r_2) + (1 - \cos \varphi) \sinh r_1 \sinh r_2$ | [KC24], equation 2 (P) |

In the polar formula, $\varphi$ is the angle between the two points as seen from the origin. [KC24] recommends this form for points that are close together (P). Equation 1 of [KC24] has a garbled term, which Section 14 records.

### 3.3. Constructing a Geodesic

Through two points in the half-space, when the points are not on one vertical line [CFKP], p. 79, item 3 (P):

1. Draw the Euclidean segment between the two points.
2. Extend its perpendicular bisector down to the boundary. The point where it meets the boundary is the centre of the geodesic semicircle.

Through two points in the disk [HT], Section 4.2, p. 17 (P):

1. When the two points are collinear with the origin, the geodesic is the straight line through them.
2. Otherwise, invert one point $z$ in the unit circle, to $1/\bar z$.
3. Pass the circle through the two points and the inverted point. Its arc inside the disk is the geodesic.

From its ideal endpoints in the disk, the geodesic with the endpoints $e^{i\theta_1}$ and $e^{i\theta_2}$ is the arc of the circle with this centre and radius, where $d\theta = \frac{1}{2} \lvert \theta_1 - \theta_2 \rvert$ [MW-DISK], equations 2-6 (S):

$$\text{centre} = \sec(d\theta) \, e^{i(\theta_1 + \theta_2)/2}, \qquad \text{radius} = \tan(d\theta)$$

[CFKP], p. 80, states only that the geodesics are diameters and arcs orthogonal to the boundary (P). [W-DISK] describes them as the circles $x^2 + y^2 + ax + by + 1 = 0$ (S).

## 4. Conversions Between Models

### 4.1. CFKP's Connecting Maps

[CFKP], p. 71, connects all five models through the hemisphere $J$ (P). In these formulas $x = (x_1, \dots, x_{n+1})$:

| Map | Direction | Construction | Formula |
| --- | --- | --- | --- |
| $\alpha$ | Hemisphere to half-space | Central projection from $(-1, 0, \dots, 0)$ | $x \mapsto \left(1, \dfrac{2 x_2}{x_1 + 1}, \dots, \dfrac{2 x_{n+1}}{x_1 + 1}\right)$ |
| $\beta$ | Hemisphere to disk | Central projection from $(0, \dots, 0, -1)$ | $x \mapsto \left(\dfrac{x_1}{x_{n+1} + 1}, \dots, \dfrac{x_n}{x_{n+1} + 1}, 0\right)$ |
| $\gamma$ | Klein to hemisphere | Vertical projection | $(x, 1) \mapsto \left(x, \sqrt{1 - \lvert x \rvert^2}\right)$ |
| $\delta$ | Hyperboloid to hemisphere | Central projection from $(0, \dots, 0, -1)$ | $x \mapsto \left(\dfrac{x_1}{x_{n+1}}, \dots, \dfrac{x_n}{x_{n+1}}, \dfrac{1}{x_{n+1}}\right)$ |

[W-HYPGEOM] states the same relations in words: the disk, hemisphere, and hyperboloid models are related by stereographic projection from $-1$, and the Klein model is the orthographic projection of the hemisphere (S).

### 4.2. Direct Formulas in the Plane

This table gives the direct conversions, each with its direction and source. The hyperboloid point is $(x_1, x_2, x_3)$ with the time coordinate last, except in the row from [W-DISK]:

| Direction | Formula | Source |
| --- | --- | --- |
| Hyperboloid to disk | $\left(\dfrac{x_1}{x_3 + 1}, \dfrac{x_2}{x_3 + 1}\right)$ | [KC24], p. 3 (P), and the composition $\beta \circ \delta$ of [CFKP] (P) |
| Disk to hyperboloid, time first | $(t, x) = \dfrac{(1 + \lvert y \rvert^2, 2y)}{1 - \lvert y \rvert^2}$ | [W-DISK] (S) |
| Hyperboloid to Klein | $\left(\dfrac{x_1}{x_3}, \dfrac{x_2}{x_3}\right)$ | [KC24], p. 3 (P) |
| Klein to disk | $u = \dfrac{s}{1 + \sqrt{1 - \lvert s \rvert^2}}$ | The composition $\beta \circ \gamma$ of [CFKP] (P) |
| Disk to Klein | $s = \dfrac{2u}{1 + \lvert u \rvert^2}$ | [W-DISK] (S) |

[KC24], Section 3, adds that its "flattened" variant of the linear representation "is effectively the Beltrami-Klein model" (P).

### 4.3. Disk to Half-Plane: An Unsettled Convention

No source fixes one map between the disk and the half-plane. Escher has to choose one and record the choice. [CFKP] gives only the composition $\alpha \circ \beta^{-1}$ through the hemisphere, with no complex formula (P). [KC24], p. 4, says that the half-plane is "obtained from the Poincare disk model by applying a circle inversion ... The center point of $U^2$ is $i$" (P).

The candidate conventions are these:

| Source | Disk to half-plane | Half-plane to disk | Image of the disk centre | Status |
| --- | --- | --- | --- | --- |
| [CAM], Section 4.3 | $z = i \dfrac{1 + \zeta}{1 - \zeta}$ | $\zeta = \dfrac{z - i}{z + i}$ | $i$ | (A) |
| [W-METRIC] | Not given | $w = \dfrac{iz + 1}{z + i}$ | $i$ maps to 0 | (S) |
| [W-MOBIUS] | $f(z) = \dfrac{z + i}{iz + 1}$, "a concrete isomorphism", with the direction not recorded | Not given | Not recorded | (S) |
| [SYMMHUB] `lib/shaders/complex.glsl.mjs:140-147`, `uhp2disc` | Not given | With $m = x^2 + y^2$ and $D = m + 2y + 1$: $(x, y) \mapsto \left(\dfrac{1 - m}{D}, \dfrac{2x}{D}\right)$, and the scale factor is multiplied by $2/D$ | Not recorded | (P-code) |
| [KC19], p. 92 | Prints $f(z) = \dfrac{1}{z - i} + \dfrac{1}{2}$ | Not given | See Section 14 | (P) as printed, and the typo is unverified |

[CAM] also gives the real forms of its convention (A):

$$(x, y) \mapsto \left(\frac{2x}{x^2 + (1 - y)^2}, \frac{1 - x^2 - y^2}{x^2 + (1 - y)^2}\right) \quad \text{(disk to half-plane)}$$

$$(x, y) \mapsto \left(\frac{2x}{x^2 + (1 + y)^2}, \frac{x^2 + y^2 - 1}{x^2 + (1 + y)^2}\right) \quad \text{(half-plane to disk)}$$

The research reports describe `uhp2disc` as the Cayley map rotated by $\pi$, which is the researchers' own algebra (Section 15.1, item 20). According to the first research report, the conventions of [CAM] and [W-METRIC] both send the disk centre to $i$ and differ by a rotation. No source confirms that statement.

### 4.4. The Gans Model

[HRM] describes the disk models as perspective projections of the hyperboloid from the point $(0, 0, -z)$, with "z = 0: Beltrami-Klein", "z = 1: Poincare", and "z = $\infty$ (orthogonal projection): Gans model" (S). No source gives a formula for the Gans model, and [KC19], Figure 1, only shows it (P). Section 15.1 records the candidate.

## 5. The Band Model and Bulatov's Conformal Models

### 5.1. The Band Model

The band model maps the disk conformally onto a strip. The sources use three strip widths. This table gives each convention as its source prints it:

| Source | Strip | Map | Metric |
| --- | --- | --- | --- |
| [B2], ring page, and [B3] | $\lvert \operatorname{Im} w \rvert < 1$ | "w = (4/$\pi$) tanh^-1(z)", disk to band (P) | Not given |
| [SYMMHUB] `lib/shaders/complex.glsl.mjs:112-125`, `band2disc` | $\lvert \operatorname{Im} w \rvert < 1$ | Band to disk: $a = \frac{\pi}{4} f$, $b = 1/\tan a$, $z = b \tanh(a w)$ (P-code) | The scale factor is multiplied by $a b \lvert 1 - \tanh^2(a w) \rvert$, line 123 (P-code) |
| [STRIP], Section II, equations 1-2 | $\lvert \operatorname{Im} w \rvert < 1$ | "w(z) = (4/$\pi$) arctanh z" (A) | $ds^2 = \left(\frac{\pi}{2}\right)^2 \dfrac{\lvert dw \rvert^2}{\cos^2\left(\frac{\pi}{2} \operatorname{Im} w\right)}$, curvature $-1$ (A) |
| [B1] | Stated as $\lvert \operatorname{Im} w \rvert < 1$ | "(2/$\pi$) tanh^-1(z)" (P) | "(2/$\pi$)\|dz\|/cos(Im(z))" (P) |
| [KC19], p. 92 | $0 < \operatorname{Im} z < 1$ | From the half-plane: $f(z) = \log(z) / \pi$ (P) | Not given |
| [HRM] | Not given | "Take the complex logarithm of the half-plane model" (S) | Not given |
| [W-BAND] | $\lvert \operatorname{Im} z \rvert < \pi/2$ | Not given | $\lvert dz \rvert \sec(\operatorname{Im} z)$ (S) |

For the strip $\lvert \operatorname{Im} w \rvert < 1$, the final research report concludes that the constant is $4/\pi$. The map is that of [B2] and [B3] (P), and the inverse is that of [SYMMHUB] (P-code):

$$w = \frac{4}{\pi} \operatorname{artanh} z, \qquad z = \tanh\left(\frac{\pi w}{4}\right)$$

The conclusion rests on [B2], [B3], [STRIP], and [SYMMHUB], which all use $4/\pi$ with that strip. The report also finds [B1] internally inconsistent, because $\operatorname{artanh}$ maps the disk onto $\lvert \operatorname{Im} \rvert < \pi/4$, so the constant $2/\pi$ gives $\lvert \operatorname{Im} w \rvert < 1/2$. That step is the report's own reasoning and no source states it.

[SYMMHUB] states the inverse in three places in `lib/shaders/complex.glsl.mjs` (P-code):

- Lines 77-82: the comment "conformally maps infinite horizontal band -1 < y < 1 into unit disk" above `vec2 band2disc(vec2 z){ return cTanh(z*(PI/4.)); }`, which is $z = \tanh(\pi w / 4)$.
- Lines 94-100: a version that also updates the scale, with `float coeff = (PI/4.);` and `scale *= coeff*length(vec2(1,0) - cMul(pp, pp));`, which multiplies the scale factor by $\frac{\pi}{4} \lvert 1 - \tanh^2(\pi w/4) \rvert$.
- Lines 112-125: the version with a fraction $f$ that the table gives. Its comment at line 119 says "b = 1 when fraction = 1", so at $f = 1$ it is also $z = \tanh(\pi w / 4)$.

This map is exactly the inverse of the map of [B2]. When $\lvert a \rvert < 10^{-4}$, `band2disc` returns the point unchanged (P-code, lines 115-118). `band2uhp` at `lib/shaders/complex.glsl.mjs:130-135` maps the band to the half-plane by $w \mapsto \exp(w)$ and multiplies the scale factor by $\lvert \exp(w) \rvert$ (P-code).

For evaluation in the complex plane, [B1] gives the form "tanh^-1(z) = (1/2)(log(1+z) - log(1-z))" and calls it the "proper way to calculate mapping by tanh^-1 on complex plane". The two branch cuts run from $-1$ and $1$ on the unit circle to $\infty$ (P).

### 5.2. Geodesics, Hypercycles, and Uses of the Band

These properties of the band come from the cited sources:

- "Every end of a geodesic either meets a boundary of the band at a right angle or is asymptotic to the midline" [W-BAND] (S).
- "Lines parallel to the boundaries of the band within the band are hypercycles" [W-BAND] (S).
- "A chosen hyperbolic straight line and its equidistants are mapped isometrically as horizontal lines" [HRM] (S).
- "At the center line of the band ... the metric is euclidean" [B1] (P).
- "The band model ... represents all points close to the equator Im z = 1/2 with high precision" [KC19], p. 94 (P). This is in the convention $0 < \operatorname{Im} z < 1$.
- [STRIP] states that along the real axis, distances are "Euclidean distances scaled by $\pi/2$" (A).
- [KC24], Section 7, p. 14, names Bulatov's band among the representations that suit work near a line (P).

No source gives the band geodesics in closed form. Section 15.1 records this.

### 5.3. Periodicity

A tiling in the band is periodic only under a condition on the group. These remarks come from [B1] (P):

- "The hyperbolic tiling in the band model is periodic in R2" when the axis of the band is an axis of the group. "In the general case the tiling will not be periodical."
- "Period varies for different pairs of fixed points."
- When the two fixed points are of different types, "we have different periodical structures at the ends".

[KC19], p. 96, maps a band tiling of period $t$ onto a ring with $f(z) = \exp(2\pi i z / t)$, which sends the band $0 < \operatorname{Im} z < 1$ onto $\lbrace \lvert z \rvert \in (e^{-2\pi/t}, 1) \rbrace$ (P).

### 5.4. Bulatov's Further Models

[B1], [B2], and [B3] present these conformal models, which the first research report lists in full (P). The first table gives the main models, and the second gives the models that have no formula:

| Model | Formula as printed | Use and remarks as printed |
| --- | --- | --- |
| Poincare disk | Isometries "M(z) = (az+b)/(cz+d)", $ad - bc = 1$ | "Isometries of the disk model are Moebius transformations with one or 2 fixed points." Used for tilings and for animations of rotations, translations, and parabolic maps |
| Upper half-plane | None | Geodesics are semicircles orthogonal to the real axis |
| Band | Section 5.1 | "The band model is conformal, because it is obtained from disk model using conformal transformation." |
| Bended band | "w = (1/2)(-a log(1-az) - log(1-z))" | "We can move one of the points and the corresponding cut anywhere we want" |
| Star | "w = $\Sigma$ b_i log(1-a_i z)" | "arbitrary number of arms placed at some arbitrary locations". "be careful about the locations of the cuts and take appropriate branch". Examples with 3, 4, 5, 6, and 8 arms |
| Ring | "w = exp(z/a)", applied to the band | "the real parameter a should be divisible by the euclidean period of the tiling in the band model" [B2], ring page. "The period depends on the particular location and orientation of the tiling in the disk model" |
| $H^3$ cylinder | "generalization of the log(1-z) mapping to 3D" | "not conformal, but quasi conformal with anisotropy factor close to 1" |

| Model without a formula | Description as printed |
| --- | --- |
| Arbitrary conformal | "any simply connected noncompact Riemann surface other than the whole complex plane". "The problem is to find the mapping explicitly." |
| Disk with 4 symmetric cuts | An example |
| Half disk with reflections | "Disk is mapped to half disk and 3 reflections in its sides are added" |
| Julia set | "Outside of the unit disk is mapped to the outside of the Julia set of quadratic map z -> z^2+c" |
| Sector | "a sector with angle 360 degrees" |
| Brooch | "Some polynomial mapping of the disk" |
| Polygon | "Mapping of the disk onto (almost) polygonal area" |
| Stairs | "I have no idea how I've got this." |

These maps go from the disk, or from the band, to the model. No source gives the inverse of any of them, except the band, whose inverse Section 5.1 gives. [B3] says "Reflections in lines are replaced with inversions in circles" (P), but no Bulatov source gives a per-pixel algorithm.

[BUL11], p. 2, relates the Klein ball to the disk by an inversion (P). The inversion in the sphere with centre $(0, 0, 1)$ and radius $\sqrt{2}$ maps the boundary of the Klein ball to the plane, which is stereographic projection. The lower hemisphere goes to the tiling of the Poincare disk, and the upper hemisphere goes to its inversion in the unit circle, outside the disk.

## 6. Isometries

### 6.1. Lorentz Matrices on the Hyperboloid

A matrix $M$ induces an isometry of the hyperboloid if and only if two conditions hold [CFKP], Theorem 10.1 (P):

1. $m_i \ast m_j = e_i \ast e_j$ for the columns $m_i$ of $M$ and the standard basis vectors $e_i$. This holds exactly when $M^{-1} = J M^t J$.
2. The last entry of the last column is positive.

Linear, Riemannian, and topological isometries of the hyperboloid coincide, and every isometry is the restriction of a linear map [CFKP], Theorem 10.2, and Section 11, p. 84 (P). The inverse is therefore $J M^t J$, and composition is the matrix product. [CFKP], p. 83, builds isometries by a Minkowski Gram-Schmidt process (P).

[KC24], p. 3, gives these basic isometries with the time coordinate last (P):

$$T^\alpha(x) = (x_1 \cosh \alpha + x_3 \sinh \alpha, \ x_2, \ x_3 \cosh \alpha + x_1 \sinh \alpha)$$

$$R^\alpha(x) = (x_1 \cos \alpha - x_2 \sin \alpha, \ x_2 \cos \alpha + x_1 \sin \alpha, \ x_3)$$

$$\operatorname{mid}(x, y) = \frac{x + y}{\sqrt{-g^-(x + y, x + y)}}$$

Here $T^\alpha$ is a boost along $x_1$, $R^\alpha$ is a rotation about $C_0$, and $\operatorname{mid}$ is the midpoint of two points.

[W-HYPERBOLOID] gives the full group with the time coordinate first (S). $O(1, n)$ is the group of "real (n+1)x(n+1) matrices which preserve the Minkowski bilinear form", and $O^+$ "preserves the sign of the first coordinate". The same page gives the translation $L_t$ of Section 7.2 and the reflection of Section 7.4.

### 6.2. Mobius Transformations on the Disk

The orientation-preserving isometries of the disk are the Mobius transformations in $SU(1,1)$ [HT], equations 8-12, p. 16 (P):

$$z \mapsto \frac{az + b}{\bar b z + \bar a}, \qquad \begin{pmatrix} a & b \\ \bar b & \bar a \end{pmatrix}, \qquad \lvert a \rvert^2 - \lvert b \rvert^2 = 1$$

[VGRG], Section 2.2, p. 8, states which maps are the isometries of the disk (P): "Orientation preserving hyperbolic isometries are given by Mobius transformations mu : H -> H that leave the unit circle as a whole invariant. Orientation reversing hyperbolic isometries are obtained by combining a Mobius transformation with complex conjugation." Here $H$ is the open unit disk, and [VGRG] gives no explicit formula for a reflection.

All Mobius maps $(az + b)/(cz + d)$ form a group isomorphic to $PGL(2, \mathbb{C})$. The disk isometries need $\lvert a \rvert^2 - \lvert b \rvert^2 = 1$, $d = \bar a$, and $c = \bar b$, so the group is $PSU(1,1) = SU(1,1)/\lbrace \pm 1 \rbrace$, which is "isomorphic to PSL(2,R)" [HT] (P).

[CF], Section 2.1, equations 2.3-2.4, gives the same form (A). The full isometry group of the disk is $U(1,1) = SU(1,1) \cup \kappa \cdot SU(1,1)$, where $\kappa$ is $z \mapsto \bar z$ [CF] (A). [BOE], equation 4, gives the equivalent form $e^{i\eta}(a - z)/(1 - z \bar a)$ (A).

The normalization differs between sources. [W-HMOTION] writes the matrix as $\begin{pmatrix} a & b^* \\ b & a^* \end{pmatrix}$, which swaps $b$ and $\bar b$ relative to [HT] (S). [W-MOBIUS] writes $(e^{i\varphi} z + b)/(\bar b z + 1)$ (S). These forms differ by renaming and by scalar multiples, according to the first research report.

### 6.3. Mobius and Anti-Mobius Transformations on the Half-Plane

The orientation-preserving isometries of the half-plane are $z \mapsto (az + b)/(cz + d)$ with real entries and $ad - bc = 1$, which form $SL(2, \mathbb{R})$ [KC24], p. 6 (P), and [ARX2105], Section 1.1 (A).

The full group is $PGL(2, \mathbb{R})$, which acts by [SPR], Section 4 (A):

$$M_A(z) = \begin{cases} \dfrac{az + b}{cz + d} & \det A > 0 \\[2ex] \dfrac{a \bar z + b}{c \bar z + d} & \det A < 0 \end{cases}$$

[SPR], equations 22-23, gives four generators with their matrices (A):

| Map | Matrix |
| --- | --- |
| $z + b$ | $\begin{pmatrix} 1 & b \\ 0 & 1 \end{pmatrix}$ |
| $\lambda z$ | $\begin{pmatrix} \lambda^{1/2} & 0 \\ 0 & \lambda^{-1/2} \end{pmatrix}$ |
| $-\bar z$ | $\begin{pmatrix} -1 & 0 \\ 0 & 1 \end{pmatrix}$ |
| $1 / \bar z$ | $\begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}$ |

Composition of Mobius maps is the matrix product, and the inverse is $f^{-1}(z) = (dz - b)/(-cz + a)$ [W-MOBIUS] (S). The rule for composing with an anti-Mobius map, in which the inner matrix is conjugated, has no source. Section 15.1 records it.

Matrices represent orientation-reversing isometries directly, while the Clifford, half-plane, and polar representations "need to add an extra bit of information" [KC24], Section 7, p. 13 (P).

### 6.4. Inversions in Circles

Inversion in a circle reverses orientation and preserves angles, so it is "anticonformal" [W-INVERSIVE] (S). These facts define it:

- Inversion in the unit circle is $z \mapsto 1/\bar z$ [W-INVERSIVE] (S).
- A circle is invariant under the inversion "if and only if it is orthogonal to the reference circle" [W-INVERSIVE] (S).
- In vector form, inversion in the circle with centre $x_0$ and radius $k$ is $x' = x_0 + k^2 (x - x_0)/\lvert x - x_0 \rvert^2$ [MW-INV], equation 5 (S).
- In the half-space, inversion in a sphere centred on the boundary is an isometry. For centre 0 and radius $r$, it is $x' = (r / \lvert x \rvert)^2 x$ [CFKP], Fact 6, pp. 93-95 (P).

Section 7.4 gives the inversion that reflects in a geodesic of the disk.

### 6.5. The Clifford Representation and the Isomorphisms

[KC24], p. 5, represents isometries in a Clifford algebra (P). The map $w \mapsto v w \bar v$ is "the reflection in the hyperplane orthogonal to v" when $g^-(v, v) = 1$. The even subalgebra stores an orientation-preserving isometry of $H^d$ in $2^d$ reals, against $(d + 1)^2$ for a matrix.

[KC24], p. 6, relates the Clifford element to a half-plane matrix (P):

$$k_0 + k_2 e_1 e_3 + k_1 e_2 e_3 + k_3 e_1 e_2 \quad \longleftrightarrow \quad \begin{pmatrix} k_0 - k_2 & k_1 + k_3 \\ k_1 - k_3 & k_0 + k_2 \end{pmatrix}$$

[OBL], Section 2.2.2, equations 2.12-2.20, gives the isomorphism from $SL(2, \mathbb{R})$ to $SO^+(2, 1)$ with the time coordinate first (A). It uses $\eta = \operatorname{diag}(-1, 1, 1)$, the basis $t_0 = \begin{pmatrix} 0 & 1 \\ -1 & 0 \end{pmatrix}$, $t_1 = \begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}$, and $t_2 = \begin{pmatrix} 1 & 0 \\ 0 & -1 \end{pmatrix}$, with $\det X = -\eta_{\mu\nu} x^\mu x^\nu$ and the action $X \mapsto S X S^{-1}$. For $S = \begin{pmatrix} a & b \\ c & d \end{pmatrix}$ the image is:

$$f[S] = \begin{pmatrix} \frac{1}{2}(a^2 + b^2 + c^2 + d^2) & \frac{1}{2}(a^2 - b^2 + c^2 - d^2) & -ab - cd \\ \frac{1}{2}(a^2 + b^2 - c^2 - d^2) & \frac{1}{2}(a^2 - b^2 - c^2 + d^2) & -ab + cd \\ -ac - bd & bd - ac & ad + bc \end{pmatrix}$$

Under this map, the rotation $\begin{pmatrix} \cos \theta & \sin \theta \\ -\sin \theta & \cos \theta \end{pmatrix}$ goes to a Lorentz rotation by $2\theta$, and $\operatorname{diag}(e^{-\chi/2}, e^{\chi/2})$ goes to the boost $\begin{pmatrix} \cosh \chi & -\sinh \chi & 0 \\ -\sinh \chi & \cosh \chi & 0 \\ 0 & 0 & 1 \end{pmatrix}$ [OBL] (A). [W-SL2R] states that the Killing form "induces an isomorphism between PSL(2, R) and the Lorentz group SO+(2,1)" (S).

The isomorphism from $SU(1,1)$ to $SO^+(2,1)$ in the basis of the disk, and the dictionary between the adjoint action of [OBL] and the action on the half-plane, have no source. Section 15.1 records them.

### 6.6. Classification

[CFKP], Section 12, pp. 84-86, classifies an isometry by its fixed points (P):

| Class | Fixed points | Normal form |
| --- | --- | --- |
| Elliptic | A fixed point inside | After conjugation, the matrix is Euclidean orthogonal |
| Hyperbolic | No interior fixed point, two at infinity | In the half-space, $f(x) = k O(x)$, with an invariant axis and the translation $x \mapsto kx$ |
| Loxodromic | As hyperbolic | Hyperbolic with $O$ nontrivial |
| Parabolic | One fixed point at infinity | $f(x) = k O(x) + v$ with $k = 1$ |

In the plane, a matrix in $SL(2, \mathbb{R})$ with determinant 1 is classified by its trace: "parabolic iff trace^2 = 4, elliptic iff 0 <= trace^2 < 4, hyperbolic iff trace^2 > 4" [ARX2105], Section 1.1 (A). [W-HYPGEOM] names the orientation-preserving classes in words: the identity, a half turn, "rotation around a normal point", a horolation, which is a rotation about an ideal point, and "translation along a straight line" (S). The orientation-reversing classes are "reflection through a line" and a reflection combined with a translation along the same line [W-HYPGEOM] (S).

A matrix with determinant $-1$ is a reflection when its trace is 0, which is "an involution with a geodesic of fixed points". Otherwise it is a glide reflection with "two fixed points on the real axis" [WAK], Section 2, equation 1 (A).

### 6.7. Fixed Points, Rotation Angle, and Translation Length

These formulas apply to a matrix $\begin{pmatrix} a & b \\ c & d \end{pmatrix}$:

| Quantity | Formula | Source |
| --- | --- | --- |
| Fixed points, $c \ne 0$ | The roots of $c \gamma^2 - (a - d)\gamma - b = 0$, with $\Delta = (\operatorname{tr})^2 - 4 \det$ | [W-MOBIUS] (S) |
| Fixed points, $c = 0$ | $\infty$ is fixed | [W-MOBIUS] (S) |
| Parabolic fixed point | A double root | [W-MOBIUS] (S) |
| Hyperbolic fixed points | "exactly two fixed points on the real axis" | [WAK], Section 2 (A) |
| Translation length, determinant 1 | $2 \cosh(\ell_g / 2) = \max\lbrace \lvert \operatorname{tr} g \rvert, 2 \rbrace$, with $\ell_g = \inf_z d(gz, z)$ | [MAR], equations 39-41 (A), and [W-FUCHSIAN] (S) |
| Glide translation length, determinant $-1$ | $\lvert \operatorname{tr} g \rvert = 2 \sinh(t_g / 2)$ | [WAK], equation 1 (A) |
| Elliptic rotation angle $\theta$ | $\operatorname{tr}^2 = 2 + 2 \cos \theta$ | [W-MOBIUS] (S) |
| Hyperbolic, in the same form | $\operatorname{tr}^2 = 2 + 2 \cosh \theta$ | [W-MOBIUS] (S) |

The rotation $R(\varphi) = \operatorname{diag}(e^{i\varphi/2}, e^{-i\varphi/2})$ of [HT], equation 14, acts as $z \mapsto e^{i\varphi} z$ (P).

## 7. Constructions

### 7.1. Rotation About a Point

In the disk, the general orientation-preserving isometry is [HT], equation 13 (P):

$$f(z) = e^{i\varphi} \frac{z - \alpha}{1 - \bar\alpha z}$$

With $\alpha = 0$ it is the rotation about the origin, with the matrix $R(\varphi) = \operatorname{diag}(e^{i\varphi/2}, e^{-i\varphi/2})$ [HT], equation 14 (P).

To rotate about a point $z_0$, [HT] uses "the proper composition of elementary transformations" (p. 17), and Section 5.3.1 applies it to a vertex of a tiling (P):

1. Move $z_0$ to the origin with $f_T$ of Section 7.3.
2. Rotate by the angle, such as $2\pi/q$ about a vertex of $\{p, q\}$.
3. Move the origin back to $z_0$ with $f_T^{-1}$.

On the hyperboloid, $R^\alpha$ of Section 6.1 rotates about $C_0$ [KC24], p. 3 (P).

### 7.2. Translation Along an Axis

In the disk, the translation along the real axis by the distance $\theta$ is [HT], equations 15-16 (P):

$$T_x(\theta) = \begin{pmatrix} \cosh \frac{\theta}{2} & \sinh \frac{\theta}{2} \\ \sinh \frac{\theta}{2} & \cosh \frac{\theta}{2} \end{pmatrix}$$

[HT] calls $\theta$ the "rapidity $\theta$ (not $\theta/2$!)" and says that translations are "elements of SO+(1,2)" (P). A translation in the direction $\varphi$ is $R(\varphi) T(\theta) R(-\varphi)$ [HT], equation 16 (P). [CF], Section 2.1, gives the same boost $a_\tau$, with the entries $\cosh(\tau/2)$ and $\sinh(\tau/2)$ and $a_\tau \cdot 0 = \tanh(\tau/2)$ (A).

On the hyperboloid with the time coordinate first, the translation by the distance $t$ "in the positive x direction" is [W-HYPERBOLOID] (S):

$$L_t = \begin{pmatrix} \cosh t & \sinh t & 0 \\ \sinh t & \cosh t & 0 \\ 0 & 0 & 1 \end{pmatrix}$$

With the time coordinate last, it is $T^\alpha$ of Section 6.1 [KC24], p. 3 (P).

### 7.3. Moving a Point to the Origin

In the disk, this map "moves $z_0$ to the origin", and the second map is its inverse [HT], equations 17-18 (P):

$$f_T(z) = \frac{z - z_0}{1 - z \bar z_0}, \qquad f_T^{-1}(z) = \frac{z + z_0}{1 + z \bar z_0}$$

[UIUC] gives the same pair as $T_{-\alpha}(z) = (z - \alpha)/(1 - \bar\alpha z)$ and $T_\alpha(z) = (z + \alpha)/(\bar\alpha z + 1)$ (S).

### 7.4. Reflection in a Geodesic

[HT], Section 5.5.2, p. 32, reflects a point in the edge between two vertices of the disk with the helper REFLECT (P):

1. Move the first vertex of the edge to 0 with $f_T$.
2. Rotate the image of the second vertex onto the real axis.
3. Apply complex conjugation.
4. Undo the rotation, then move 0 back to the first vertex.

[SYMMHUB] reflects by inversion, in `iReflect` at `lib/shaders/isplane.glsl.mjs:114-133` (P-code):

- A circle with centre $c$ and radius $r$: $v \mapsto c + (v - c) \, r^2 / \lvert v - c \rvert^2$. The running scale factor is multiplied by $r^2 / \lvert v - c \rvert^2$.
- A line through the point $p$ with unit normal $n$: $v \mapsto v - 2((v - p) \cdot n) n$.

Section 7.6 gives the sides of the fundamental triangles that [SYMMHUB] reflects in. No source gives the reflection in a disk geodesic as one closed formula, and Section 15.1 records the candidate.

[DAVIS], Section 6.2, printed pp. 73-77, is the primary source for reflections in all three geometries (P). A hyperplane is "totally geodesic codimension one", and each hyperplane "separates the ambient space into two geodesically convex 'half-spaces'". The reflection $r_H$ in the hyperplane $H$ with unit normal $u$ takes the same form in each geometry:

| Geometry | Reflection $r_H$ | Normal $u$ | Equation |
| --- | --- | --- | --- |
| Euclidean space $E^n$ | $x - 2 \langle u, x - x_0 \rangle u$ | A unit normal, with $x_0$ a point of $H$ | (6.3) |
| Sphere $S^n$ | $x - 2 \langle u, x \rangle u$ | A unit normal | (6.4) |
| Hyperbolic space $H^n$, quadratic form model | $x - 2 \langle u, x \rangle u$ | "a unit spacelike vector orthogonal to H" | (6.6) |

In the hyperbolic row, $\langle \cdot, \cdot \rangle$ is the Minkowski form of the quadratic form model. The research addendum does not record the order of the time coordinate in [DAVIS].

[W-HYPERBOLOID] states the same reflection with the time coordinate first, for the spacelike unit vector $u$ with $B(u, u) = 1$, and adds its matrix (S):

$$x \mapsto x - 2 B(x, u) u, \qquad R = I - 2 u (J u)^T, \quad J = \operatorname{diag}(-1, 1, 1)$$

The page states it as the reflection that swaps two points $p$ and $q$, with $u = (p - q)/\sqrt{Q(p - q)}$ (S). With the time coordinate last, [KC24], p. 5, gives the Clifford reflection of Section 6.5 (P).

In the half-space, the reflection in a geodesic is the inversion of Section 6.4 [CFKP], Fact 6 (P).

### 7.5. Glide Reflection and Products of Reflections

A glide reflection is a reflection combined with a translation along the same line [W-HYPGEOM] (S). Every isometry of the hyperbolic plane "can be realized as the composition of at most three reflections" [W-HYPGEOM] and [W-HMOTION] (S).

A normal form of a glide reflection as $\operatorname{diag}(e^{t/2}, -e^{-t/2})$ rests only on a check by a research agent that agrees with [WAK]. Section 15.1 records it.

### 7.6. The Fundamental Triangle of a Triangle Group in SymmHub

This subsection states what [SYMMHUB] does in `lib/grouplib/Group_KLM.js`, as a research agent read it from the code (P-code). It describes one implementation. No paper states these formulas, and Section 15.1 lists the statements that the researcher derived and the code does not contain. Every line number in this subsection refers to `lib/grouplib/Group_KLM.js` unless the citation names another file.

#### 7.6.1. Sides and Their Sign Conventions

The code stores each side as a line or a circle in the plane of the model:

- `iPlane([nx, ny, nz, d])`: the line with normal $n$ and offset $d$. Its domain side is $x \cdot n - d < 0$, so $n$ is the outward normal.
- `iSphere([cx, cy, cz, rho])`: the circle with centre $c$ and signed radius $\rho$. A positive $\rho$ makes the inside of the circle the domain side, and a negative $\rho$ makes the outside the domain side (`lib/shaders/isplane.glsl.mjs:29-31`).

The fundamental domain is the set where every signed distance `iDistance` is negative (`apps/orbifold/js/shaders/inversive.glsl.mjs:756`).

#### 7.6.2. Angles and Dispatch

`getFD` (`:263-268`) calls `makeTriangle(getAngle(K), getAngle(L), getAngle(M))`. `getAngle` (`:318-324`) returns 0 when its argument is at least 20 or less than 2, and $\pi/n$ for any other argument $n$. A value of 0, 1, or 20 or more for K, L, or M therefore means an ideal corner.

`makeTriangle` (`:292-312`) chooses the geometry with `EPSILON = 1.e-6` (`:29`):

1. If $\lvert a_0 \rvert <$ `EPSILON`, it calls `makeIdealTriangle`.
2. Otherwise it forms $s = a_0 + a_1 + a_2$. If $\lvert s - \pi \rvert <$ `EPSILON`, it calls `makeEuclideanTriangle`.
3. Otherwise, if $s > \pi +$ `EPSILON`, it calls `makeSphericalTriangle`.
4. Otherwise it calls `makeHyperbolicTriangle`.

Only an ideal corner K, with angle $a_0$, selects the ideal branch. An ideal corner L or M with $a_0 > 0$ goes through the test of the angle sum, for example as hyperbolic.

#### 7.6.3. The Two Sides Through the Origin

In the Euclidean, spherical, and hyperbolic cases, the first two sides are lines through the origin (`:336-338`, `:361-362`, `:375-377`):

- $s_0$ = `iPlane([0., -1., 0., 0.])`: the line $y = 0$, with the domain side $y > 0$.
- $s_1$ = `iPlane([-sin(a0), cos(a0), 0., 0.])`: the line through the origin at the angle $a_0$ above the $x$ axis, with the domain side $y \cos a_0 < x \sin a_0$.

These two sides meet at the origin with the interior angle $a_0 = \pi/K$, which is the corner K.

#### 7.6.4. The Third Side

The third side $s_2$ depends on the geometry. The parameter `triSize` defaults to 0.25, and no caller passes it (`:307`):

| Geometry | Function and lines | Third side $s_2$ as the code builds it |
| --- | --- | --- |
| Euclidean | `makeEuclideanTriangle`, `:355-366` | `iPlane([sin(a1), cos(a1), 0., e])` with `e = triSize`: the line $x \sin a_1 + y \cos a_1 = e$, with the domain side $x \sin a_1 + y \cos a_1 < e$ |
| Spherical | `makeSphericalTriangle`, `:332-349` | With `e2 = triSize`: $r = \dfrac{e_2 \sin a_0}{\cos a_2 + \cos(a_1 + a_0)}$, $c_x = \dfrac{r (\cos a_2 + \cos a_1 \cos a_0)}{\sin a_0}$, $c_y = r \cos a_1$, and $s_2$ = `iSphere([cx, cy, 0, -r])` |
| Hyperbolic | `makeHyperbolicTriangle`, `:371-389` | $a_x = \dfrac{\cos a_2 + \cos a_1 \cos a_0}{\sin a_0}$, $a_y = \cos a_1$, $r = 1/\sqrt{a_x^2 + a_y^2 - 1}$, the centre $(r a_x, r a_y)$, and $s_2$ = `iSphere([cx, cy, 0, -r])` |

In the hyperbolic case, $r$ is the reciprocal of a square root and therefore positive, so the radius parameter $-r$ is negative and the domain is outside $s_2$. In the spherical case, the side of $s_2$ that is the domain follows from the sign of $r$, which the code does not state.

#### 7.6.5. The Ideal Case

`makeIdealTriangle` (`:398-436`) builds the triangle with the ideal corner K. Its comments (`:391-396`, `:404`) say that the three sides are first built as a strip, that $s_1$ is the image of the line $y = 2$ (the commented-out code `fd.s[1] = iPlane(vec3(0.,1., 0.),2.)`), and that the strip is then mapped by the inversion in the circle with centre $(-1, 0)$ and radius 2.

The code builds the sides in these steps:

1. $s_0$ = `iPlane([0., -1., 0., 0.])`, the line $y = 0$.
2. $s_1$ = `iSphere([-1., 1., 0., -1.])`, the circle with centre $(-1, 1)$ and radius 1, with the domain outside it.
3. It sets `denum` to $\cos a_1 + \cos a_2$. If the absolute value of `denum` is less than `EPSILON`, it sets $s_2$ = `iSphere([0., 0., 0., 1.])`, the unit circle with the domain inside it, and returns.
4. Otherwise it sets $r_3 = 2/(\cos a_1 + \cos a_2)$, $x_3 = 1$, and $y_3 = r_3 \cos a_1$. The comments describe the circle with centre $(x_3, y_3)$ and radius $r_3$ as the third side of the strip before the inversion.
5. It sets `dd` to $(x_3 + 1)^2 + y_3^2 - r_3^2$.
6. If the absolute value of `dd` is greater than `EPSILON2`, with `EPSILON2 = 1.e-4` (`:28`), it sets $a = 4/\mathrm{dd}$ and replaces $x_3$ by $-1 + a(x_3 + 1)$, $y_3$ by $a y_3$, and $r_3$ by $a r_3$. It then sets $s_2$ = `iSphere([x3, y3, 0, -r3])`. The domain is outside $s_2$ when $a > 0$ and inside when $a < 0$, by the sign convention of Section 7.6.1.
7. Otherwise $s_2$ is a line. It is `iPlane([1., 0., 0, 0.])`, the line $x = 0$, when $\lvert a_2 \rvert <$ `EPSILON`, with the code comment "a1 = pi/2, a2 = 0". In every other case it is `iPlane([1., 1., 0, 0.])`, the line $x + y = 0$, with the code comment "a1 = pi/4, a2 = pi/2".

#### 7.6.6. The Corners

The comment at `:270-277` places the corners: "K at the corner of the sides 0 and 1 (the origin), L of the sides 0 and 2 (on the x axis), M of the sides 1 and 2". It continues: "of two points where sides meet, the corner is the one nearest the origin (the other is its image in the absolute). 0, 1 and 20 are [infinity], an ideal corner."

`getHandles` computes the corners numerically (`:282-283`), with `const corner = (i, j) => cornerOf(tri[i], tri[j], [tri[3 - i - j]], [0, 0]);` and `corners = { K: corner(0, 1), L: corner(0, 2), M: corner(1, 2) }`. The code states no closed form for the corners L and M.

## 8. Numerical Robustness

### 8.1. The Studies

Three sources measure or report the precision of hyperbolic representations:

- [FWW], of which only the abstract was read, reports that "while numerical errors in O(4) are negligible, numerical errors in O(3,1) tend to spiral out of control" (A). [KC24], Section 1, p. 2, notes that [FWW] compares only two representations of isometries and dates from 2002 (P).
- [KC24] compares representations and normalization variants in IEEE 754 double precision, with GMP as the reference (P).
- [KC19], pp. 94-95, gives one concrete figure in double precision (P).

### 8.2. Representations and Variants Compared in KC24

[KC24], Section 2, compares these representations (P):

| Representation | Content |
| --- | --- |
| Linear | Points on the hyperboloid and isometries as matrices |
| Mixed | Isometries in the Clifford algebra |
| Reduced | The midpoint $x'' = \operatorname{mid}(C_0, x)$, whose "distance ... from C0 is half the distance of x ... a serious numerical advantage". Projecting it in the manner of the Klein model gives the Poincare disk |
| Half-plane | $SL(2, \mathbb{R})$ matrices and points of the half-plane |
| Polar | Distance and angle from $C_0$, with the distance of equation 2 in Section 3.2 |

[KC24], Section 3, pp. 6-7, applies these variants to each representation (P):

| Variant | Policy as printed |
| --- | --- |
| Invariant | "Hope that numerical errors do not build up" |
| Careless | Not quoted in the report |
| Flattened | "effectively the Beltrami-Klein model" for the linear representation |
| Forced | "Normalize the output after every computation" |
| Weakly forced | Not quoted in the report |
| Binary | Keeps the leading coordinate in $[0.5, 2)$ |
| linear+F | Matrices are fixed by replacing them with correct isometries. "HyperRogue uses this method" |
| Gyro | Fixes "rotations by 180 degrees ... flattened to infinity" |

Without linear+F, [KC24] reports "the visualization becoming visibly stretched after the user moves sufficiently far away ... it tends to fix itself when the user moves back" (P).

[KC24], Section 4, p. 8, also combines each representation with tile coordinates (P). A point is stored as $(t, p_0)$, with $p = X_t p_0$, and the isometry between adjacent tiles is $X_{t, t'} = R^\alpha T^x R^\beta$.

### 8.3. Results

[KC24], Sections 5-6, pp. 8-12, scores each test by the first distance at which the error exceeds 0.1 (P). Its results are these (P):

- A point at distance $d$ from $C_0$ lies at $\tanh d = 1 - \Theta(e^{-2d})$ in the Klein model and at $1 - \Theta(e^{-d})$ in the Poincare disk. "We can expect the effective distance represented accurately in flattened reduced to be double that of flattened linear." Section 14 records the sign as printed.
- The invariant linear representation stores $(\cos\alpha \sinh x, \sin\alpha \sinh x, \cosh x)$ and can recover large $x$ "as long as cosh(x) fits in the range". This makes it "significantly better than the Poincare disk model ... in some experiments, such as AngleDist", because "floating-point numbers are good at representing large numbers".
- The half-plane gives far points an "imaginary coordinate close to 0 instead" of close to 1.
- "Polar2 mod representation wins, although the halfplane invariant is also very successful and linear+F representations (especially invariant) perform well in walk tests."
- In the walk test, $\{7, 3\}$ is the best tiling, $\{6, 6\}$ is "significantly worse", and "no tessellation is even worse".

[KC24] warns that "numerically computing $AB^{-1}$ for two isometries moving C0 to two points a and b which are close to each other but far from C0 is not likely to yield meaningful results" (P).

[KC19], pp. 94-95, gives these figures (P):

- A point 25 units from the centre lies at about $1 - 2.8 \cdot 10^{-11}$ in the Poincare disk. "This is dangerously close to the precision of the double floating-point format." Section 14 records the formula that [KC19] prints for this point.
- A pair of doubles has "only $2^{128}$ distinct points", while the area of a circle of radius $r$ is about $\pi e^r$.
- A perturbation $\varepsilon$ of a line moves the points at distance $d$ from it by about $\varepsilon \sinh d$.
- The remedy in reference 6 of [KC19] is to store the closest tile vertex and local coordinates, built with finite automata.

[HT], Sections 3.3 and 5.3.3, reports that searches for neighbours in disk coordinates become "inaccurate close to the unit circle" (P). [HT] warns when the Euclidean distance between adjacent cells nears machine precision and tracks the accumulated error layer by layer.

### 8.4. Countermeasures in Mature Implementations

The mature implementations use these countermeasures:

| Implementation | Countermeasure | Source |
| --- | --- | --- |
| HyperRogue | `fixmatrix` calls `orthonormalize`, a Gram-Schmidt process over the columns in the Minkowski form. Its comment says: "The numerical errors tend to accumulate, eventually destroying the projection. This function fixes this problem by replacing T with a 'correct' isometry." | [HR], `hyperpoint.cpp` (P-code) |
| HyperRogue | `normalize` divides a point by $\operatorname{sign} \cdot \sqrt{-\langle h, h \rangle}$ | [HR], `hyperpoint.cpp` (P-code) |
| HyperRogue | The scalar type `ld` is `double` | [HR], `hyper.h` (P-code) |
| HyperRogue | The Euclidean case uses `fixmatrix_euclid` instead | [HR], `hyperpoint.cpp` (P-code) |
| Curved Spaces | `MatrixGeometricInverse` inverts "geometrically for better precision than row-reduction". For $O(3,1)$ it is the transpose "with a few minus signs" | [CS], `CurvedSpacesMatrices.c` (P-code) |
| Curved Spaces | `VectorNormalize` uses $\sqrt{-\langle v, v \rangle}$ in the hyperbolic case. The file has no Gram-Schmidt function | [CS], `CurvedSpacesMatrices.c` (P-code) |
| SymmHub | The shader uses the tolerance `EPS = 1.e-7` in the fold | [SYMMHUB], `lib/shaders/inversiveSampler.glsl.mjs:184` (P-code) |
| SymmHub | The pixel offset from the view centre is kept separately, so that deep zoom stays exact in single precision. A comment says that "world coordinates near 1 keep 6e-8" | [SYMMHUB], `lib/shaders/viewOffset.glsl.mjs:1-13`, `lib/shaders/overlay_editing.glsl.mjs:13-14` (P-code) |
| SymmHub | "A view far from F runs out of steps and stops drawing." The remedy is an anchor group element computed in double precision on the CPU | [SYMMHUB], `lib/grouplib/ViewAnchor.js:1-49` (P-code) |
| SymmHub | The fold runs 10 iterations by default and up to 1000 | [SYMMHUB], as the research report summarizes it (P-code) |

## 9. Distances and Scale for Drawing

### 9.1. Circles

Hyperbolic circles are Euclidean circles in all three conformal models, but "Euclidean and hyperbolic sphere centers need not coincide" [CFKP], Fact 1 (P). [CFKP], Figure 19, constructs the hyperbolic centre in the half-space geometrically (P). The circumference of a circle of radius $\rho$ is $C = 2\pi \sinh \rho$ and its area is $A = 2\pi(\cosh \rho - 1)$ [CFKP], Fact 5 (P). Its geodesic curvature is $\coth r$ [W-HYPGEOM] (S).

This table gives the Euclidean form of a hyperbolic circle in each model:

| Model | Hyperbolic centre and radius to Euclidean | Euclidean to hyperbolic | Source |
| --- | --- | --- | --- |
| Half-plane | Centre $(x, y)$ and radius $r$ give the Euclidean centre $(x, y \cosh r)$ and radius $y \sinh r$ | Euclidean centre $(x_e, y_e)$ and radius $r_e$ give the centre $(x_e, \sqrt{y_e^2 - r_e^2})$ and radius $\frac{1}{2} \ln \frac{y_e + r_e}{y_e - r_e}$ | [W-HALFPLANE] (S), inverse (S, r) |
| Half-plane | Not given | The Euclidean circle with centre $a + ib$ and radius $\rho$ has the hyperbolic centre $a + i\sqrt{b^2 - \rho^2}$, with $\rho = b \tanh R$ | [COR], Exercise 2 (A) |
| Disk, real centre $c$ | Euclidean centre $c' = c \big/ \left(1 + \frac{1}{2}(\cosh r - 1)(1 - c^2)\right)$, and radius $r' = (x_r - x_l)/2$ from the endpoints on the real axis, which [CHEN] solves for | Not given | [CHEN], Appendix A.1.4, equations 55-57 (A) |
| Disk, centre at the origin | The Euclidean radius is $R$ of the formula from the origin in Section 3.2 | Same formula | [CFKP], Fact 5 (P) |

In the disk, the hyperbolic and Euclidean centres lie on the same radius, and the Euclidean centre is closer to the centre of the disk [W-DISK] (S). A closed form for a general centre has no source. Section 15.1 records the candidate.

### 9.2. Hypercycles and Horocycles

In the half-space, the equidistant curves of a vertical geodesic are Euclidean straight lines through its foot [CFKP], Fact 2, Figure 20 (P). In the disk, hypercycles are "lines and circle arcs that intersect the boundary circle at non-right angles", and their axis has the same ideal endpoints. An arc of a hypercycle at distance $r$ over a segment of length $d$ of its axis has the length $l = d \cosh r$ [W-HYPERCYCLE] (S).

Horocycles take these forms [W-HOROCYCLE], Section 2 (S):

| Model | Horocycles |
| --- | --- |
| Disk | Circles tangent to the boundary |
| Half-plane | Circles tangent to the real axis, and horizontal lines |
| Hyperboloid | Plane sections with a null normal |

Every horocycle has geodesic curvature 1 [W-HOROCYCLE] (S).

### 9.3. Distance From a Point to a Geodesic

The sources give the distance from a point to a geodesic in two models (A):

- Hyperboloid: $d = \operatorname{arsinh}\left((w \ast x)/\sqrt{-w \ast w}\right)$ [CHO], Theorem 1, equation 15. [CHO] uses the opposite sign of the Minkowski form, so a spacelike $w$ has $w \ast w < 0$, and its proof calls the result "unsigned".
- Disk: $d = \operatorname{arsinh} \dfrac{2 \lvert \langle -p \oplus x, a \rangle \rvert}{(1 - \lvert -p \oplus x \rvert^2) \lvert a \rvert}$ [GBH], Theorem 5, equation 23, at $c = 1$, where $\oplus$ is Mobius addition and the geodesic passes through $p$ with normal $a$.

No source states the simpler forms in the half-plane, the disk, or the hyperboloid with the time coordinate first. Section 15.1 records them.

The angle of parallelism $\Pi(a)$ at distance $a$ satisfies $\sin \Pi(a) = \operatorname{sech} a$ and $\tan(\Pi/2) = e^{-a}$ [W-PARALLEL] (S).

### 9.4. Scale Factor of Each Model

The scale factor $\lambda$ converts a Euclidean length in a model to a hyperbolic length, $ds = \lambda \lvert dz \rvert$:

| Model | Scale factor | Source |
| --- | --- | --- |
| Disk | $\lambda = \dfrac{2}{1 - \lvert z \rvert^2}$, and radially $ds = 2\,dr/(1 - r^2)$ | [CFKP], p. 71 and p. 92 (P), [GBH], equation 3 (A) |
| Half-plane | $\lambda = 1/y$ | [CFKP], p. 71 (P) |
| Band, $\lvert \operatorname{Im} w \rvert < 1$ | $\lambda = \dfrac{\pi/2}{\cos\left(\frac{\pi}{2} \operatorname{Im} w\right)}$ | [STRIP] (A) |
| Band, $\lvert \operatorname{Im} z \rvert < \pi/2$ | $\lambda = \sec(\operatorname{Im} z)$ | [W-BAND] (S) |
| Klein | Anisotropic, with the metric of Section 3.1, in components $g_{ij} = \dfrac{\delta_{ij}}{1 - \lvert x \rvert^2} + \dfrac{x_i x_j}{(1 - \lvert x \rvert^2)^2}$ | [CFKP], p. 71 (P), [W-KLEIN] (S) |
| $\kappa$-stereographic | $\lambda^\kappa_x = \dfrac{2}{1 + \kappa \lvert x \rvert^2}$ | [BBG], Section 2 (A) |

[SYMMHUB] carries a running scale factor through each model map and each inversion, and multiplies it by the factor of each step. Sections 4.3, 5.1, 7.4, and 11.2 give the factor that each function applies (P-code). The use of the scale factor to convert a line width to pixels is in `docs/research/rendering.md`.

## 10. Hyperbolic Trigonometry for Construction

### 10.1. Triangles and Tiling Conditions

These conditions decide whether a triangle or tiling exists:

- The area of a triangle with angles $\alpha$, $\beta$, $\gamma$ is $\pi - \alpha - \beta - \gamma$. For any angles with sum less than $\pi$, "there is one and only one triangle up to congruence" [CFKP], Fact 3 (P).
- The angles of a kaleidoscope are $\pi/n$ with $n > 1$, or 0, with sum less than $\pi$, and triangles are rigid. A kaleidoscope with $n$ sides "has n-3 degrees of freedom", and $*22222$ has two free sides [BUL11], p. 1 (P).
- A $\{p, q\}$ tiling is hyperbolic when $(p - 2)(q - 2) > 4$ [DUN07], p. 3, and [HT], p. 4 (P). [KC24], Section 4, uses the equivalent condition $1/p + 1/q < 1/2$ (P).
- [HT], Section 7.1, classifies the triangle group $(p, q, r)$ as spherical, Euclidean, or hyperbolic by comparing $1/p + 1/q + 1/r$ with 1 (P). [W-HYPGEOM] states that hyperbolic tilings need $1/p + 1/q + 1/r < 1$ (S).
- The $p$-gon of a $\{p, q\}$ tiling divides into $2p$ right triangles [DUN07], p. 3, and [HT], p. 20 (P).
- For curvature $-R^{-2}$, the area of a triangle is $R^2(\pi - \alpha - \beta - \gamma)$, and lengths scale as $a/R$ [MW-HYPGEOM] (S). [HT], equation 30, gives the area as $(\pi - \alpha - \beta - \gamma) \ell^2$ (P).

### 10.2. Laws of Triangles

For a triangle with sides $a$, $b$, $c$ opposite the angles $A$, $B$, $C$, [W-HTRI] gives these laws (S):

$$\cosh c = \cosh a \cosh b - \sinh a \sinh b \cos C$$

$$\cos C = -\cos A \cos B + \sin A \sin B \cosh c$$

$$\frac{\sin A}{\sinh a} = \frac{\sin B}{\sinh b} = \frac{\sin C}{\sinh c}$$

[HT], equation 26, gives the same law of sines (P).

The side of a triangle with given angles is [W-LAWCOS] (S):

$$\cosh a = \frac{\cos A + \cos B \cos C}{\sin B \sin C}$$

With $A = \pi/p$, $B = \pi/q$, and $C = \pi/r$, this gives the sides of the $(p, q, r)$ triangle. [HR], `edge_of_triangle_with_angles`, implements the same formula as `acos_auto((cos(alpha)+cos(beta)*cos(gamma))/(sin(beta)*sin(gamma)))` (P-code).

### 10.3. Right Triangles

For a right triangle with $C = \pi/2$, [W-HTRI] gives these relations (S):

| Relation | Relation |
| --- | --- |
| $\cosh c = \cosh a \cosh b$ | $\sin A = \sinh a / \sinh c$ |
| $\cos A = \tanh b / \tanh c$ | $\tan A = \tanh a / \sinh b$ |
| $\cos A = \cosh a \sin B$ | $\cosh c = \cot A \cot B$ |

Section 14 records an earlier transcription of the relation $\cos A = \cosh a \sin B$.

### 10.4. Regular Polygons

[HT], Section 4.3, pp. 19-21, gives the data of the $\{p, q\}$ polygon with curvature radius $\ell$ (P):

| Quantity | Formula | Equation of [HT] |
| --- | --- | --- |
| Euclidean circumradius in the disk | $r_0 = \sqrt{\dfrac{\cos\left(\frac{\pi}{p} + \frac{\pi}{q}\right)}{\cos\left(\frac{\pi}{p} - \frac{\pi}{q}\right)}}$ | 24 |
| Euclidean inradius $r_m$ in the disk, implicit | $\cos \dfrac{\pi}{p} = \dfrac{\tanh(2 \operatorname{artanh} r_m)}{\tanh(2 \operatorname{artanh} r_0)}$ | 25 |
| Half the edge | $\sin \dfrac{\pi}{p} = \dfrac{\sinh(h/2)}{\sinh d_0}$, with $d_0$ as [HT] defines it | 28 |
| Edge length | $h = 2 \ell \operatorname{arcosh} \dfrac{\cos(\pi/p)}{\sin(\pi/q)}$ | 29 |
| Area of a triangle | $(\pi - \alpha - \beta - \gamma) \ell^2$ | 30 |

[HT], Table 2, gives these values for $\{7, 3\}$ (P): $h = 1.090550$, $h_r = 0.620673$, and $r_0 = 0.300743$. Section 15.1 records the hyperbolic circumradius and inradius, which no source states.

### 10.5. Right-Angled Pentagons and Hexagons

In a right-angled pentagon with consecutive sides $\ell_1, \dots, \ell_5$, "cosh $\ell_4$ = sinh $\ell_1$ sinh $\ell_2$, and its cyclic rotations", where $\ell_4$ is the side adjacent to neither $\ell_1$ nor $\ell_2$ (A). Given $\ell_1$ and $\ell_2$, the pentagon exists when $\sinh \ell_1 \sinh \ell_2 > 1$, and the other sides are (A):

$$\sinh \ell_4 = \sqrt{\sinh^2 \ell_1 \sinh^2 \ell_2 - 1}, \qquad \sinh \ell_3 = \frac{\cosh \ell_1}{\sinh \ell_4}, \qquad \sinh \ell_5 = \frac{\cosh \ell_2}{\sinh \ell_4}$$

The sources are [RAH], Section 4, [MG], Section 3.1, equation 24, which cites Fenchel [FEN], p. 82, equation 2, and [OC], equation A.2.

In a right-angled hexagon, $\cosh c = \sinh a \sinh b \cosh \gamma - \cosh a \cosh b$ [OC], Section 3.1, equation 3.1 (A). [CONE], equation 9, gives the same and cites Buser [BUS], Theorem 2.4.1(i) (A). [RAH], Section 2.1, equation 1, parametrizes the hexagon by alternate sides (A, r).

A pentagon with one angle $\alpha_i$ that is not right satisfies the following, from [CONE], Lemma 2.7, equation 13, which cites [BUS], Exercise 2.2.7 (A, r):

$$\sinh \delta = \frac{\cosh c_i + \cos \alpha_i \cosh L_i'}{\sin \alpha_i \sinh L_i'}$$

### 10.6. Lambert and Saccheri Quadrilaterals

A Lambert quadrilateral $AOBF$ has right angles at $A$, $O$, and $B$, and an acute angle at $F$. [W-LAMBERT] gives these relations and cites Martin [MAR-GEO], p. 436 (S, r):

| Relation | Relation |
| --- | --- |
| $\sinh AF = \sinh OB \cosh BF$ | $\tanh AF = \cosh OA \tanh OB$ |
| $\cosh OF = \cosh OA \cosh AF$ | $\sin \angle F = \cosh OB / \cosh AF$ |
| $\cos \angle F = \sinh OA \sinh OB$ | $\cot \angle F = \tanh OA / \sinh AF$ |

A Saccheri quadrilateral has a base $b$, two legs of length $l$, and a summit $s$, and "the summit is longer than the base". [W-SACCHERI] gives these relations and cites Buser and Karcher, and Greenberg (S, r):

$$\cosh s = \cosh b \cosh^2 l - \sinh^2 l, \qquad \sinh(s/2) = \cosh l \sinh(b/2)$$

## 11. The Sphere

### 11.1. Points, Distances, and Isometries

[KC24], p. 3, treats the sphere as a linear model (P). It is $S^d = \lbrace g^+(x, x) = 1 \rbrace$ with the origin $C_0 = (0, \dots, 0, 1)$. Its isometries are the orthogonal matrices, its translation is $T^\alpha(x) = (x_1 \cos \alpha + x_3 \sin \alpha, \ x_2, \ x_3 \cos \alpha - x_1 \sin \alpha)$, and its distance is $d(x, y) = \arccos g^+(x, y)$.

Rotations and reflections also have these representations:

| Representation | Formula | Source |
| --- | --- | --- |
| Unit quaternion $q$ | $p' = q p q^{-1}$. "Each rotation is represented by two unit quaternions of opposite sign" | [W-QROT] (S) |
| Rotation angle of a quaternion | Conjugation by a unit quaternion with real part $\cos \varphi$ "is a rotation by an angle $2\varphi$" | [W-QUAT] (S) |
| Quaternion reflection | "r' = -wrw" for a unit vector $w$ | [W-QUAT] (S) |
| Reflection in a great sphere | Section 7.4 | [DAVIS], equation 6.4, printed pp. 73-77 (P) |
| Householder reflection | $H_u(x) = x - 2 \langle x, u \rangle u$, "involutory" and orthogonal | [W-HOUSEHOLDER] (S) |
| $PSU(2)$ on the Riemann sphere | Rotations correspond to $PSU(2) \cong SO(3)$, with coefficients satisfying $\lvert u \rvert^2 + \lvert v \rvert^2 = 1$ | [W-MOBIUS] (S) |

### 11.2. Stereographic Projection

The stereographic projection from the north pole onto the plane through the centre is $y'_i = x_i/(1 - x_{n+1})$ [CFKP], Theorem 8.3 (P). It is conformal [CFKP], Theorem 8.1, and maps spheres to planes or spheres [CFKP], Theorem 8.2 (P). In the plane, [W-STEREO] gives the map and its inverse onto $z = 0$ (S, r):

$$X = \frac{x}{1 - z}, \quad Y = \frac{y}{1 - z}, \qquad (x, y, z) = \frac{(2X, 2Y, X^2 + Y^2 - 1)}{1 + X^2 + Y^2}$$

[SYMMHUB] shows the sphere through stereographic projection in `makeSphericalProjection` at `lib/shaders/projection_v2.glsl.mjs:252-266` (P-code). It reads each pixel $p$ as an orthographic view of the front hemisphere. With `pp` $= \lvert p \rvert^2$ and $z = \sqrt{1 - \lvert p \rvert^2}$, it applies two factors:

- The position: `p *= factor` with `factor = 1./(1. + sqrt(1.-pp))`, which is $1/(1 + z)$.
- The running scale factor: `scale *= factor2` with `factor2 = 1./sqrt(1.-pp)`, which is $1/z$. An earlier line, `//scale *= factor`, is commented out.

When $\lvert p \rvert > 1$, the function leaves $p$ unchanged and returns the positive value `sqrt(pp)-1.`. The final research report states that such pixels are discarded, so that only one hemisphere shows. The exact metric factors of this view are an open question in Section 15.1.

Section 5.4 gives the equivalent inversion of [BUL11].

### 11.3. Spherical Trigonometry for Triangle Groups

The spherical second law of cosines is $\cos C = -\cos A \cos B + \sin A \sin B \cos c$ [W-SPHLAW] (S). Solved for a side, it has the form of the hyperbolic formula of Section 10.2 with $\cos$ in place of $\cosh$, according to the first research report.

A spherical triangle group needs an angle sum above $\pi$, so $1/l + 1/m + 1/n > 1$ [W-TRIGROUP] (S). The spherical Mobius triangles are $(p\,2\,2)$, $(3\,3\,2)$, $(4\,3\,2)$, and $(5\,3\,2)$ [W-SCHWARZ] (S).

### 11.4. Ray and Sphere Intersection

For a ray with origin $O$ and direction $D$, and a sphere with centre $C$ and radius $R$, [SCRATCH-RS] gives two methods (S).

The analytic method solves $a t^2 + b t + c = 0$ with:

$$a = D \cdot D, \qquad b = 2 D \cdot (O - C), \qquad c = \lvert O - C \rvert^2 - R^2$$

A negative discriminant means that the ray misses the sphere.

The geometric method computes:

1. $L = C - O$.
2. $t_{ca} = L \cdot D$.
3. $d^2 = L \cdot L - t_{ca}^2$.
4. $t_{hc} = \sqrt{R^2 - d^2}$.
5. $t_{0,1} = t_{ca} \mp t_{hc}$.

### 11.5. Camera Rays and Projection Matrices

[SCRATCH-CAM] generates the ray through pixel $(i, j)$ of a $W \times H$ image with this code, and then normalizes the direction $(x, y, -1)$ (S):

```text
x = (2 * (i + 0.5) / W - 1) * aspect * tan(fov / 2)
y = (1 - 2 * (j + 0.5) / H) * tan(fov / 2)
```

The prose formula of the article differs from its code, as Section 14 records. This document follows the code.

The fixed-function projection matrices are on the Khronos reference pages [GL21] (P):

- glOrtho: $\begin{pmatrix} \frac{2}{r - l} & 0 & 0 & t_x \\ 0 & \frac{2}{t - b} & 0 & t_y \\ 0 & 0 & \frac{-2}{f - n} & t_z \\ 0 & 0 & 0 & 1 \end{pmatrix}$, with $t_x = -\frac{r + l}{r - l}$, and $t_y$ and $t_z$ in the same form as the page gives them.
- glFrustum: the entries $2n/(r - l)$ and $2n/(t - b)$ on the diagonal, $A$ and $B$ as the page gives them, $C = -(f + n)/(f - n)$, $D = -2fn/(f - n)$, and the bottom row $(0, 0, -1, 0)$.
- gluPerspective: $f = \cot(\text{fovy}/2)$, with the entries $f/\text{aspect}$, $f$, $(z_F + z_N)/(z_N - z_F)$, $2 z_F z_N/(z_N - z_F)$, and $-1$.

### 11.6. Phong and Blinn-Phong Shading

The Phong reflection model is [W-PHONG] (S):

$$I_p = k_a i_a + \sum_m \left(k_d (\hat L_m \cdot \hat N) i_{m,d} + k_s (\hat R_m \cdot \hat V)^\alpha i_{m,s}\right), \qquad \hat R_m = 2 (\hat L_m \cdot \hat N) \hat N - \hat L_m$$

The primary reference is [PHONG75], which nobody has read.

The Blinn-Phong model replaces the reflection vector with the half vector $H = (L + V)/\lVert L + V \rVert$ and uses the specular term $(N \cdot H)^{\alpha'}$, with $\alpha' \approx 4\alpha$ to match Phong [W-BLINN] (S). [LEARNOPENGL] suggests a factor "between 2 and 4 times" (S). The primary reference is [BLINN77], which nobody has read.

## 12. The Euclidean Plane

Section 7.4 gives the reflection in a line [DAVIS], equation 6.3 (P). The other isometries of the Euclidean plane take these forms (S):

- In complex form, $z \mapsto a + \omega z$ or $z \mapsto a + \omega \bar z$ with $\lvert \omega \rvert = 1$. The rotation about $p$ is $z \mapsto \omega(z - p) + p$ [W-EUCLISO].
- There are "four types": translation, rotation, reflection, and glide reflection [W-EUCLISO].
- A glide reflection is the "composition of a reflection across a line and a translation parallel to the line". Reflections and glide reflections are "the only two kinds of indirect" isometries [W-GLIDE].
- In homogeneous coordinates, an isometry is the $3 \times 3$ matrix $\begin{pmatrix} A & b \\ 0 & 1 \end{pmatrix}$ [W-AFFINE].

## 13. Unified Treatments of the Three Geometries

### 13.1. One Fold in One Plane

This subsection is explanatory. [SYMMHUB] treats all three geometries with one code path (P-code). Every group is a list of circles and lines in one two-dimensional plane, and the shader does not know the geometry. The hyperbolic plane is the Poincare disk, and every other model is a conformal map applied before the fold.

The trade-off is one code path and one data format against the loss of intrinsic distance. Distances in the fold are Euclidean in the model and only their sign matters, so the fold gives no intrinsic widths. `docs/research/rendering.md` describes the fold.

### 13.2. Representations That Generalize

[KC24], Section 7, p. 14, compares how each representation extends to the other geometries (P):

- The Poincare model with stereographic projection is chosen in references 1 and 4 of [KC24] "for this reason".
- Polar coordinates, and the hyperboloid, sphere, and plane as linear models, also generalize.
- "Only half-plane and half-space do not easily generalize beyond hyperbolic geometry."
- Lobachevsky coordinates $T^x R^{\pi/2} T^y C_0$, and Bulatov's band, suit work near a line.

### 13.3. Curved Spaces

[CS] uses one matrix type for all three geometries (P-code). Its data types are `typedef enum {SpaceNone, SpaceSpherical, SpaceFlat, SpaceHyperbolic} SpaceType` and `double m[4][4]` with a parity flag. A placement is "an element of Isom(S3) = O(4), Isom(E3) or Isom(H3) = O(3,1)", and "most of the code doesn't need to know the curvature of space".

`MatrixTranslation` builds every translation from one generator (P-code). The code uses row vectors, so the flat translation sits in the last row:

| Geometry | `m[0][3]` | `c1` | `c2` |
| --- | --- | --- | --- |
| Spherical | `-dx` | $\sin L$ | $1 - \cos L$ |
| Flat | `0` | $L$ | $0$ |
| Hyperbolic | `+dx` | $\sinh L$ | $\cosh L - 1$ |

`VectorGeometricDistance` reads the geometry from $w = v_3$. It uses `SafeAcos(w)` if $w < 1$, the flat distance if $w = 1$, and `SafeAcosh(w)` if $w > 1$ (P-code).

`MakeProjectionMatrix` is shared, and only the far clip differs (P-code). It is "WFarClip = 0" for Euclidean space, "WFarClip = -WNearClip" for spherical space, and "WFarClip = 1" for hyperbolic space, "because all of hyperbolic space is contained within a lightcone of angle 45 degrees". The GPU code multiplies each vertex by `tilingGroup[iid]`, and only the fog differs between geometries. The hyperbolic fog uses `log(w)`, since "log(cosh(d)) is approximately d - log 2".

The trade-offs that [CS] records are these. The near clip must stay moderate "to avoid needless loss of precision in the depth buffer". The space $S^3$ needs passes for the front and back hemispheres with the antipodal map. A comment says to "use MetricHorizontal or MetricVertical as in KaleidoTile", which [KT] holds and nobody has read.

### 13.4. HyperRogue

[HR] keeps a signature table for each geometry (P-code). It holds `giEuclid2 {1,1,0}`, `giHyperb2 {1,1,-1}`, and `giSphere2 {1,1,+1}`, in `classes.cpp`. Its inner product `geo_inner` sums `h1[i]*h2[i]*sig(i)`.

`sin_auto` and `cos_auto` return $\sinh$, $x$, or $\sin$, and $\cosh$, 1, or $\cos$, and `curvature()` returns $-1$, 0, or 1 (P-code). The push matrix `cpush` sets `T[L][L] = T[c][c] = cos_auto(alpha)`, `T[c][L] = sin_auto(alpha)`, and `T[L][c] = -curvature() * sin_auto(alpha)`. The distance `hdist` uses `2*asinh(sqrt(iv)/2)` in hyperbolic space and `2*asin_auto_clamp(sqrt(iv)/2)` on the sphere.

The Euclidean signature is degenerate, so the inverse needs special cases, `iso_inverse` and `pseudo_ortho_inverse` (P-code).

### 13.5. Other Unified Treatments

These sources describe other unified treatments (A, except where marked):

- [WK21], Section 4.1: a curved-space game "typically uses exactly the same computer code for all three spaces". For Euclidean space, "append a '1' as a fourth coordinate". The projection divides by $z$ in all three spaces (Section 4.3). Rasterization "remains more efficient on consumer-level VR hardware" than ray tracing.
- [CMST]: $E^3$ is "the affine subspace X = {w = 1} of R4" (Section 7.1), $S^3$ uses the standard scalar product (Section 7.2), and $H^3$ uses a "lorentzian inner product" (Section 7.3). Geodesics are "straight lines in their projective models" (Section 2.3.1).
- [BBG], Section 2, equations 2-7, the $\kappa$-stereographic model: the domain is $\lbrace x : -\kappa \lvert x \rvert^2 < 1 \rbrace$ with the scale factor of Section 9.4, $\tan_\kappa = \kappa^{-1/2} \tan$ for $\kappa > 0$ and $(-\kappa)^{-1/2} \tanh$ for $\kappa < 0$, and the distance is $d_\kappa = 2 \lvert \kappa \rvert^{-1/2} \tan_\kappa^{-1} \lvert -x \oplus_\kappa y \rvert$. Its addition is:

$$x \oplus_\kappa y = \frac{(1 - 2\kappa x^T y - \kappa \lvert y \rvert^2) x + (1 + \kappa \lvert x \rvert^2) y}{1 - 2\kappa x^T y + \kappa^2 \lvert x \rvert^2 \lvert y \rvert^2}$$

  It is "a differentiable interpolation between geometries of constant curvatures irrespective of their signs", and "$\kappa$ = 0 yields all corresponding Euclidean quantities".
- [W-GYRO]: Ungar's gyrovector spaces cover the hyperbolic case only, with Mobius addition $(a + z)/(1 + \bar a z)$ (S).
- [W-CK]: in a Cayley-Klein geometry, the geometry "depends upon the selection of a quadric or conic that becomes the absolute". The page gives no single curvature parameter (S).

### 13.6. Trade-Offs

This table summarizes the trade-offs that the sources state:

| Treatment | Gains | Costs | Source |
| --- | --- | --- | --- |
| One fold in the plane of a conformal model | One code path and one data format | Distances are Euclidean in the model, so intrinsic widths are not available | [SYMMHUB] |
| Linear models with one matrix type | Most code does not know the curvature. Only the far clip and the fog differ | $O(3,1)$ errors grow without renormalization ([FWW], [KC24]). The sphere needs two passes | [CS] |
| Signature table | One formula for each operation in all three geometries | The Euclidean signature is degenerate and needs special inverses | [HR] |
| Half-plane | Accurate far from the origin ([KC24]) | Does not generalize beyond hyperbolic geometry | [KC24] |
| $\kappa$-stereographic | Interpolates smoothly through $\kappa = 0$ | Not measured for precision in any source read | [BBG] |

## 14. Conflicts Between Sources

This table lists each conflict between sources, and what this document does about it:

| Topic | What the sources say | Handling in this document |
| --- | --- | --- |
| Band constant | [B1] prints "(2/$\pi$) tanh^-1(z)" with $\lvert \operatorname{Im} w \rvert < 1$ and the metric "(2/$\pi$)\|dz\|/cos(Im(z))". [B2], [B3], [STRIP] equation 1, and [SYMMHUB] (`tanh(pi w/4)`) all use $4/\pi$ with $\lvert \operatorname{Im} w \rvert < 1$, and the metric of [STRIP] fits that strip | The final research report resolves it as $4/\pi$, on the basis in Section 5.1. A research agent once claimed that $4/\pi$ gives $\lvert \operatorname{Im} w \rvert < 2$, and the report rejects that claim for the same reason |
| Band strip width | $\lvert \operatorname{Im} \rvert < 1$ in [B2], [B3], [STRIP], and [SYMMHUB]. $0 < \operatorname{Im} < 1$ in [KC19]. $\lvert \operatorname{Im} \rvert < \pi/2$ in [W-BAND] | Each formula keeps its own convention (Section 5.1) |
| Disk to half-plane | [CAM] $i(1 + \zeta)/(1 - \zeta)$. [W-METRIC] $(iz + 1)/(z + i)$. [SYMMHUB] `uhp2disc`, which research agents describe as the Cayley map rotated by $\pi$. [CFKP] fixes none | Unsettled. Section 4.3 lists every candidate |
| KC19 half-plane map | [KC19], p. 92, prints $f(z) = 1/(z - i) + 1/2$ | The final research report concludes from its own check that this maps the disk onto $\operatorname{Im} > 1/2$, not onto the half-plane, so that it is likely a typo for a shift by $-i/2$. The check is unverified (Section 15.1) |
| Order of the time coordinate | Last in [CFKP], [KC24], and [HR]. First in [W-HYPERBOLOID], [HT] equation 1, and [OBL]. [CHO] uses the opposite sign of the form | Each formula keeps its own convention (Section 2.4) |
| KC24 hyperboloid distance | [KC24], p. 3, prints the distance as "acosh g^-(x, y)", without a minus sign | [CFKP] gives $a \ast b = -\cosh d$. This document gives the [CFKP] form (Section 3.1) |
| KC24 cosine rule | [KC24], p. 4, equation 1, prints the term "cos(phi_2) sinh(r_2) cosh(r_2)" | The report calls the term garbled. This document gives equation 2 only (Section 3.2) |
| KC24 precision | [KC24], p. 9, prints "1 - $\Theta$(exp(2d))" | The report concludes that the decay is $e^{-2d}$, because $\tanh d$ approaches 1. Section 8.3 gives $e^{-2d}$ |
| KC19 disk radius | [KC19], p. 94, prints "sinh(d)/(1 + cosh(z))" for the disk radius of a point at distance $d$ | The report concludes that the intended value is $\tanh(d/2)$. [CFKP], Fact 5, gives this radius as $(e^d - 1)/(e^d + 1)$ (Section 3.2) |
| HT equation numbers | The HTML rendering that the first report cited numbers the equations differently from the PDF of version 3 | This document uses the numbers of the PDF of version 3: disk distances equations 22-23, isometries equations 8-18, $r_0$ equation 24, $r_m$ equation 25, the law of sines equation 26, the edge length equation 29 |
| Right-triangle relation | One fetch of [W-HTRI] read "cosh a = cos B / sin A". Another read $\cos A = \cosh a \sin B$ | The final research report follows the second, which agrees with [HT] equation 29, where the half edge is opposite the angle $\pi/p$ at the centre (Section 10.3) |
| Disk Mobius normalization | [HT] $\begin{pmatrix} a & b \\ \bar b & \bar a \end{pmatrix}$. [W-HMOTION] swaps $b$ and $\bar b$. [W-MOBIUS] uses $(e^{i\varphi} z + b)/(\bar b z + 1)$ | This document uses the form of [HT] (Section 6.2) |
| Half-distance in matrices | $SL(2, \mathbb{R})$ and $SU(1,1)$ use $\theta/2$. Lorentz matrices use $\theta$ | Each formula keeps its own convention (Section 2.4) |
| Camera rays | The prose formula of [SCRATCH-CAM] applies the remap to the screen twice and disagrees with its code | This document follows the code (Section 11.5) |

## 15. Open Questions and Unverified Claims

### 15.1. Candidate Statements That Need a Source or a Machine-Checked Proof

No cited source states these results. Each is a candidate statement that needs a primary source or a machine-checked Lean proof before the body can state it as a fact:

1. Reflection in a disk geodesic in closed form: $z \mapsto c + r^2/\overline{(z - c)}$ with $r^2 = \lvert c \rvert^2 - 1$. [SYMMHUB] implements the inversion, and [MW-DISK] gives the centre as a secant and the radius as a tangent, but no source states this form.
2. A disk circle with hyperbolic centre $c$ and radius $r$, with $t = \tanh(r/2)$: the Euclidean centre $c(1 - t^2)/(1 - \lvert c \rvert^2 t^2)$ and radius $t(1 - \lvert c \rvert^2)/(1 - \lvert c \rvert^2 t^2)$. [CHEN] gives only the centre for a real $c$ and an implicit radius.
3. The distance to a geodesic in simple forms: in the half-plane, $\cosh d = \lvert z \rvert / \operatorname{Im} z$, or equivalently $\sinh d = \lvert \operatorname{Re} z \rvert / \operatorname{Im} z$, for the imaginary axis. The form for a diameter of the disk. On the hyperboloid with the time coordinate first, $\sinh d = \lvert B(x, n) \rvert$ for a unit spacelike normal $n$.
4. The $\{p, q\}$ circumradius $\cosh R = \cot\frac{\pi}{p} \cot\frac{\pi}{q}$ and inradius $\cosh r = \cos\frac{\pi}{q} / \sin\frac{\pi}{p}$. The research agent computed that $\operatorname{arcosh}(\cot\frac{\pi}{7} \cot\frac{\pi}{3}) = 0.6207$, against $h_r = 0.620673$ in [HT], Table 2, and $\tanh(0.6207/2) = 0.3008$, against $r_0 = 0.300743$. That check is arithmetic by the research agent, and it assumes that $h_r$ is the hyperbolic circumradius.
5. The corners of the fundamental triangle of [SYMMHUB] (Section 7.6), and the properties of its sides that the code does not state. A research agent derived these statements by algebra:
   - Euclidean case: the corner L is $(e/\sin a_1, 0)$, and the corner M is $(e/\sin a_2)(\cos a_0, \sin a_0)$ with $a_2 = \pi - a_0 - a_1$.
   - Spherical case: the circle $s_2$ passes through $(e_2, 0)$ for all angles, so the corner L is $(e_2, 0)$. The researcher checked this numerically for $(K, L, M) = (2, 3, 3)$, with the centre about $(-0.341, -0.341)$ and the radius 0.683. The denominator $\cos a_2 + \cos(a_1 + a_0)$ is negative when the angle sum exceeds $\pi$, so $r < 0$ and the domain is inside $s_2$.
   - Hyperbolic case: the circle $s_2$ is orthogonal to the unit circle, because $\lvert c \rvert^2 = r^2 (a_x^2 + a_y^2) = 1 + r^2$.
   - Ideal case: the inversion in the circle with centre $(-1, 0)$ and radius 2 maps the line $x = 1$ to the unit circle, so the boundary at infinity is again the unit circle. The ideal corner K is $(-1, 0)$, where $s_1$ touches the $x$ axis.
   - No closed form of the corners L and M in the hyperbolic case, which `getHandles` finds numerically.
6. A formula for the Gans model. The description in [HRM] implies the spatial part of the hyperboloid point.
7. The conversions from the disk to the hyperboloid and from the disk to the Klein model, which only secondary sources give (Section 4.2).
8. Line widths in intrinsic units. [SYMMHUB] draws widths that are constant in screen pixels, and no source draws widths in hyperbolic units.
9. The exact metric factors of the stereographic view in [SYMMHUB]. The code applies $1/(1 + z)$ to the position and the single scalar $1/z$ to the scale factor, with $z = \sqrt{1 - \lvert p \rvert^2}$ (Section 11.2). An earlier research report derived a radial factor $1/(z(1 + z))$ and a tangential factor $1/(1 + z)$. The code states neither, and its scalar $1/z$ matches neither.
10. Precision limits. [KC19] gives one figure in double precision. The first research report gives these maximum distances from its own arithmetic, with no source: the Poincare disk about 17.3 in single and 37.4 in double precision, the Klein model about 8.7 and 18.7, and the hyperboloid about 8.3 and 18.4 for cancellation in the constraint and about 89 and 710 for overflow. Guidance for single precision on a GPU beyond the tolerance of [SYMMHUB] has no source.
11. The cost and storage of each representation. Only [KC24] counts storage: $2^d$ reals for the Clifford representation against $(d + 1)^2$ for a matrix.
12. The Lambert and Saccheri formulas and the pentagon with a cone point (Sections 10.5 and 10.6), which need a check against a primary source.
13. The composition rule for anti-Mobius maps, in which the inner matrix is conjugated.
14. The pentagon laws in their coth and cot forms, the law of sines for hexagons, and the existence of polygons with prescribed angles with angle sum less than $(n - 2)\pi$, as a theorem with a construction.
15. The normal form $\operatorname{diag}(e^{t/2}, -e^{-t/2})$ of a glide reflection, which a research agent checked against [WAK].
16. The band geodesics in closed form.
17. The representation of a hypercycle on the hyperboloid, and the distance to a hypercycle.
18. The isomorphism from $SU(1,1)$ to $SO^+(2,1)$ in the basis of the disk, and the dictionary between the adjoint action of [OBL] and the action on the half-plane.
19. The reflection in a pure quaternion as $-nvn$, which has only [W-QUAT] as its source.
20. The identity of `uhp2disc` in [SYMMHUB] with $w = (i - z)/(i + z)$, which is the Cayley map $(z - i)/(z + i)$ multiplied by $-1$, and the identity of its scale factor $2/D$ with $\lvert dw/dz \rvert = 2/\lvert z + i \rvert^2$. A research agent derived both by algebra, and the code states neither.
21. The map of [KC19], p. 92, from the disk to the half-plane, and the conclusion of the research report that it is a typo (Section 14).
22. The inverse of each of Bulatov's models other than the band (Section 5.4).

### 15.2. Claims That Rest on Summaries or Secondary Sources

Every claim marked (A) rests on a summary of a paper, and every claim marked (S) rests only on a secondary source. The most important for Escher are the trace classification and the translation length (Sections 6.6 and 6.7), the pentagon and hexagon laws (Section 10.5), the disk and half-plane circle formulas (Section 9.1), the right-triangle relations (Section 10.3), and the band metric of [STRIP] (Section 5.1).

### 15.3. Sources Not Read in Full

These sources were not read in full, and `docs/research/source-requests.md` records what the project needs from each:

- [FWW]: the body, with its data on the growth of errors and its remedies. Only the abstract was read.
- [BEA]: Section 4 on the trace classification, and Sections 7.18 to 7.20 on pentagons, hexagons, the distance to a line, and circles. It would settle items 1 to 4 and 12 of Section 15.1.
- [BUS]: Chapter 2, with Theorem 2.3.4, Theorem 2.4.1, Exercise 2.2.7, and the trirectangles. It is the primary source for the pentagon, hexagon, Lambert, and Saccheri formulas.
- [RAT]: Sections 3.2 to 3.5, for the hyperboloid formulas and the hexagon theorems.
- [KAT]: Chapters 1 and 2, for the trace formulas and the distance to a geodesic in the half-plane.
- [WEEKS02] and [WEEKS06]: the unified projective pipeline.
- [KT]: its treatment of all three geometries in two dimensions and the functions `MetricHorizontal` and `MetricVertical`.
- [DUN-ISAMA07]: the disk coordinates of the fundamental polygon and its side circles.
- [PHONG75] and [BLINN77]: the original shading formulas.
- [HUB]: the band metric that [B1] cites.
- [FEN] and [MAR-GEO]: the original pentagon and Lambert formulas.
- [MDPI-APOL]: possible closed forms of disk circles, item 2 of Section 15.1.
- Every source marked (A) in this document, which was read only through a summary.

## 16. References

### 16.1. Primary Sources

- [B1] Bulatov, V., "Conformal Models of the Hyperbolic Geometry", Joint Mathematics Meetings, San Francisco, 2010-01-17, <https://www.bulatov.org/math/1001/index.html>.
- [B2] Bulatov, V., "Artistic Models of the Hyperbolic Geometry", Gathering for Gardner 9, Atlanta, 2010-03-25, <https://www.bulatov.org/math/1003/index.html>, with the ring page <https://www.bulatov.org/math/1003/index_ring.html>.
- [B3] Bulatov, V., "Bending Hyperbolic Kaleidoscopes", Bridges 2011 talk, <https://www.bulatov.org/math/1107/index_talk_web.html>.
- [BUL11] Bulatov, V., "Bending Hyperbolic Kaleidoscopes", Bridges 2011 paper, <https://www.bulatov.org/math/1107/Bulatov(2011)Bending_Hyperbolic_Kaleidoscopes_paper.pdf>.
- [CFKP] Cannon, J. W., Floyd, W. J., Kenyon, R., and W. R. Parry, "Hyperbolic Geometry", in Flavors of Geometry, MSRI Publications 31, pp. 59-115, 1997, <https://library.slmath.org/books/Book31/files/cannon.pdf>.
- [CS] Weeks, J., Curved Spaces source code, 2021, GitHub mirror, <https://github.com/swedneck/Curved-Spaces>.
- [DAVIS] Davis, M. W., The Geometry and Topology of Coxeter Groups, 2008. Locators are printed page numbers. The PDF page is the printed page plus 16.
- [DUN07] Dunham, D., "Creating Regular Repeating Hyperbolic Patterns", Math and Design, Blumenau, 2007.
- [GL21] The Khronos Group, OpenGL 2.1 reference pages for glOrtho, glFrustum, and gluPerspective, <https://registry.khronos.org/OpenGL-Refpages/gl2.1/>.
- [HR] Kopczynski, E., et al., HyperRogue source code, master branch, files `hyperpoint.cpp`, `classes.cpp`, and `hyper.h`, <https://github.com/zenorogue/hyperrogue>.
- [HT] Schrauth, M., Thurn, Y., Goth, F., Portela, J. S. E., Herdt, D., and F. Dusel, "HYPERTILING - a high performance Python library for the generation and visualization of hyperbolic lattices", arXiv:2309.10844v3, 2024-06-26, <https://arxiv.org/abs/2309.10844>. Equation numbers are those of the PDF of version 3.
- [KC19] Kopczynski, E. and D. Celinska-Kopczynska, "Conformal Mappings of the Hyperbolic Plane to Arbitrary Shapes", Bridges 2019, pp. 91-98, <https://archive.bridgesmathart.org/2019/bridges2019-91.pdf>.
- [KC24] Celinska-Kopczynska, D. and E. Kopczynski, "Numerical Aspects of Hyperbolic Geometry", arXiv:2404.09039v1, 2024-04-13, <https://arxiv.org/abs/2404.09039>.
- [SYMMHUB] SymmHub source code, commit f2d19cb. File paths are relative to the root of the repository.
- [VGRG] von Gagern, M. and J. Richter-Gebert, "Hyperbolization of Euclidean Ornaments", Electronic Journal of Combinatorics 16(2), #R12, 2009.

### 16.2. Papers Read Through Summaries

- [ARX2105] arXiv:2105.13263, Section 1.1, <https://arxiv.org/abs/2105.13263>. The research reports record no author or title.
- [BBG] Bachmann, G., Becigneul, G., and O.-E. Ganea, "Constant Curvature Graph Convolutional Networks", arXiv:1911.05076v3, 2020, <https://arxiv.org/abs/1911.05076>.
- [BOE] Boettcher, I., et al., arXiv:1910.12318, Appendix A, <https://arxiv.org/abs/1910.12318>. The research reports record no title.
- [CAM] Cambridge Part IB Geometry lecture notes, Section 4.3, <https://dec41.user.srcf.net/h/IB_L/geometry/4_3>.
- [CF] Chossat, P. and G. Faye, arXiv:1304.6005, Section 2.1, <https://arxiv.org/abs/1304.6005>. The research reports record no title.
- [CHEN] Chen, et al., arXiv:2501.17965v2, Proceedings of Machine Learning Research 258, 2025, Appendix A, <https://arxiv.org/html/2501.17965v2>. The research reports record no title.
- [CHO] Cho, H., et al., "Large-Margin Classification in Hyperbolic Space", arXiv:1806.00437, <https://arxiv.org/abs/1806.00437>.
- [CMST] Coulon, R., Matsumoto, E. A., Segerman, H., and S. J. Trettel, "Ray-marching Thurston geometries", arXiv:2010.15801v2, <https://arxiv.org/abs/2010.15801>.
- [CONE] "Volumes of moduli spaces of hyperbolic surfaces with cone points", arXiv:2212.13701, <https://arxiv.org/abs/2212.13701>.
- [COR] Cornell University, Winter 2009, Section 7, <https://pi.math.cornell.edu/~mec/Winter2009/Mihai/section7.html>.
- [FWW] Floyd, W., Weber, B., and J. Weeks, "The Achilles' Heel of O(3,1)?", Experimental Mathematics 11(1):91-97, 2002, DOI 10.1080/10586458.2002.10504472, <https://doi.org/10.1080/10586458.2002.10504472>. Only the abstract was read.
- [GBH] Ganea, O.-E., Becigneul, G., and T. Hofmann, "Hyperbolic Neural Networks", arXiv:1805.09112, <https://arxiv.org/abs/1805.09112>.
- [MAR] Marklof, J., "Selberg's trace formula: an introduction", arXiv:math/0407288, Section 3, <https://arxiv.org/abs/math/0407288>.
- [MG] "The Markov gap for geometric reflected entropy", arXiv:2107.00009, <https://arxiv.org/abs/2107.00009>.
- [OBL] Oblak, B., "From the Lorentz Group to the Celestial Sphere", arXiv:1508.00920, Section 2.2.2, <https://arxiv.org/abs/1508.00920>.
- [OC] "Open-closed Hyperbolic String Vertices", arXiv:1912.00030, <https://arxiv.org/abs/1912.00030>.
- [RAH] "Right-angled hexagon tilings of the hyperbolic plane", arXiv:1503.05510, <https://arxiv.org/abs/1503.05510>.
- [SPR] Springborn, B., arXiv:1702.05061, Sections 4-5, <https://arxiv.org/abs/1702.05061>. The research reports record no title.
- [STRIP] "Elastic hyperbolic strip lattices", arXiv:2410.18420, Section II, <https://arxiv.org/abs/2410.18420>.
- [WAK] Wakuda, "Puncture loops on a non-orientable surface", arXiv:2503.07247, Section 2, <https://arxiv.org/abs/2503.07247>.
- [WK21] Weeks, J., "Body coherence in curved-space virtual reality games", arXiv:2011.00510, <https://arxiv.org/abs/2011.00510>.

### 16.3. Secondary Sources

- [HRM] HyperRogue, "Models of hyperbolic geometry", <https://www.roguetemple.com/z/hyper/models.php>.
- [LEARNOPENGL] de Vries, J., "Advanced Lighting", LearnOpenGL, <https://learnopengl.com/Advanced-Lighting/Advanced-Lighting>.
- [MW-DISK] Weisstein, E. W., "Poincare Hyperbolic Disk", MathWorld, equations 2-6, <https://mathworld.wolfram.com/PoincareHyperbolicDisk.html>.
- [MW-HYPGEOM] Weisstein, E. W., "Hyperbolic Geometry", MathWorld, <https://mathworld.wolfram.com/HyperbolicGeometry.html>.
- [MW-INV] Weisstein, E. W., "Inversion", MathWorld, equation 5, <https://mathworld.wolfram.com/Inversion.html>.
- [SCRATCH-CAM] Scratchapixel, "Generating Camera Rays". The research reports record no URL.
- [SCRATCH-RS] Scratchapixel, "Ray-Sphere Intersection", <https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection.html>.
- [UIUC] University of Illinois, "Hyperbolic translations", <https://new.math.uiuc.edu/oldnew/math402/public/models/htranslation.html>.
- [W-AFFINE] Wikipedia, "Affine transformation", <https://en.wikipedia.org/wiki/Affine_transformation>.
- [W-BAND] Wikipedia, "Band model", <https://en.wikipedia.org/wiki/Band_model>.
- [W-BLINN] Wikipedia, "Blinn-Phong reflection model", <https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model>.
- [W-CK] Wikipedia, "Cayley-Klein metric", <https://en.wikipedia.org/wiki/Cayley%E2%80%93Klein_metric>.
- [W-DISK] Wikipedia, "Poincare disk model", <https://en.wikipedia.org/wiki/Poincar%C3%A9_disk_model>.
- [W-EUCLISO] Wikipedia, "Euclidean plane isometry", <https://en.wikipedia.org/wiki/Euclidean_plane_isometry>.
- [W-FUCHSIAN] Wikipedia, "Fuchsian group", <https://en.wikipedia.org/wiki/Fuchsian_group>.
- [W-GLIDE] Wikipedia, "Glide reflection", <https://en.wikipedia.org/wiki/Glide_reflection>.
- [W-GYRO] Wikipedia, "Gyrovector space", <https://en.wikipedia.org/wiki/Gyrovector_space>.
- [W-HALFPLANE] Wikipedia, "Poincare half-plane model", <https://en.wikipedia.org/wiki/Poincar%C3%A9_half-plane_model>.
- [W-HMOTION] Wikipedia, "Hyperbolic motion", <https://en.wikipedia.org/wiki/Hyperbolic_motion>.
- [W-HOROCYCLE] Wikipedia, "Horocycle", <https://en.wikipedia.org/wiki/Horocycle>.
- [W-HOUSEHOLDER] Wikipedia, "Householder transformation", <https://en.wikipedia.org/wiki/Householder_transformation>.
- [W-HTRI] Wikipedia, "Hyperbolic triangle", <https://en.wikipedia.org/wiki/Hyperbolic_triangle>.
- [W-HYPERBOLOID] Wikipedia, "Hyperboloid model", <https://en.wikipedia.org/wiki/Hyperboloid_model>.
- [W-HYPERCYCLE] Wikipedia, "Hypercycle (geometry)", <https://en.wikipedia.org/wiki/Hypercycle_(geometry)>.
- [W-HYPGEOM] Wikipedia, "Hyperbolic geometry", <https://en.wikipedia.org/wiki/Hyperbolic_geometry>.
- [W-INVERSIVE] Wikipedia, "Inversive geometry", <https://en.wikipedia.org/wiki/Inversive_geometry>.
- [W-KLEIN] Wikipedia, "Beltrami-Klein model", <https://en.wikipedia.org/wiki/Beltrami%E2%80%93Klein_model>.
- [W-LAMBERT] Wikipedia, "Lambert quadrilateral", <https://en.wikipedia.org/wiki/Lambert_quadrilateral>.
- [W-LAWCOS] Wikipedia, "Law of cosines", <https://en.wikipedia.org/wiki/Law_of_cosines>.
- [W-METRIC] Wikipedia, "Poincare metric", <https://en.wikipedia.org/wiki/Poincar%C3%A9_metric>.
- [W-MOBIUS] Wikipedia, "Mobius transformation", <https://en.wikipedia.org/wiki/M%C3%B6bius_transformation>.
- [W-PARALLEL] Wikipedia, "Angle of parallelism", <https://en.wikipedia.org/wiki/Angle_of_parallelism>.
- [W-PHONG] Wikipedia, "Phong reflection model", <https://en.wikipedia.org/wiki/Phong_reflection_model>.
- [W-QROT] Wikipedia, "Quaternions and spatial rotation", <https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation>.
- [W-QUAT] Wikipedia, "Quaternion", <https://en.wikipedia.org/wiki/Quaternion>.
- [W-SACCHERI] Wikipedia, "Saccheri quadrilateral", <https://en.wikipedia.org/wiki/Saccheri_quadrilateral>.
- [W-SCHWARZ] Wikipedia, "Schwarz triangle", <https://en.wikipedia.org/wiki/Schwarz_triangle>.
- [W-SL2R] Wikipedia, "SL2(R)", <https://en.wikipedia.org/wiki/SL2(R)>.
- [W-SPHLAW] Wikipedia, "Spherical law of cosines", <https://en.wikipedia.org/wiki/Spherical_law_of_cosines>.
- [W-STEREO] Wikipedia, "Stereographic projection", <https://en.wikipedia.org/wiki/Stereographic_projection>.
- [W-TRIGROUP] Wikipedia, "Triangle group", <https://en.wikipedia.org/wiki/Triangle_group>.

### 16.4. Sources Not Read in Full

- [BEA] Beardon, A. F., The Geometry of Discrete Groups, Springer Graduate Texts in Mathematics 91, 1983.
- [BLINN77] Blinn, J., "Models of light reflection for computer synthesized pictures", SIGGRAPH 1977, pp. 192-198, DOI 10.1145/563858.563893, <https://doi.org/10.1145/563858.563893>.
- [BUS] Buser, P., Geometry and Spectra of Compact Riemann Surfaces, Birkhauser, 1992.
- [DUN-ISAMA07] Dunham, D., ISAMA 2007 paper on hyperbolic patterns.
- [FEN] Fenchel, W., Elementary Geometry in Hyperbolic Space, de Gruyter, 1989.
- [HUB] Hubbard, J., Teichmuller Theory, volume 1, chapter 2, <https://matrixeditions.com/TVol1.Chap2.pdf>.
- [KAT] Katok, S., Fuchsian Groups, University of Chicago Press, 1992.
- [KT] Weeks, J., KaleidoTile source code, <https://www.geometrygames.org/KaleidoTile/KaleidoTile-Src.zip>.
- [MAR-GEO] Martin, G. E., The Foundations of Geometry and the Non-Euclidean Plane.
- [MDPI-APOL] "Apollonius and the hyperbolic circle", Geometry 3(3):15, MDPI.
- [PHONG75] Phong, B. T., "Illumination for computer generated pictures", Communications of the ACM 18(6):311-317, 1975-06, DOI 10.1145/360825.360839, <https://doi.org/10.1145/360825.360839>.
- [RAT] Ratcliffe, J. G., Foundations of Hyperbolic Manifolds, Springer Graduate Texts in Mathematics 149.
- [WEEKS02] Weeks, J., "Real-time rendering in curved spaces", IEEE Computer Graphics and Applications 22(6), 2002, DOI 10.1109/MCG.2002.1046633, <https://doi.org/10.1109/MCG.2002.1046633>.
- [WEEKS06] Weeks, J., "Real-time animation in hyperbolic, spherical, and product geometries", in Non-Euclidean Geometries, Springer, 2006. The DOI is unverified.
