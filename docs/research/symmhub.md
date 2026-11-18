# SymmHub Source Code as Prior Art

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-18 |
| Applies to | The design and implementation of Escher's group construction, point folding, colouring, and rendering |

## Abstract

SymmHub is a browser application, written in JavaScript and the OpenGL Shading Language (GLSL), that draws patterns with the symmetry of planar groups in spherical, Euclidean, and hyperbolic geometry. It is the closest existing tool to Escher, and its source code is under the MIT licence. This document records how SymmHub specifies a group, builds a fundamental domain and its side pairings, represents isometries, folds a point into the fundamental domain on the processor and on the graphics card, maps screen pixels through models of the plane, draws borders, and colours tiles. It also records SymmHub's constants, numerical workarounds, known defects, and the features that Escher needs and SymmHub lacks, so that an implementer can decide what to port and how.

## 1. Scope

This document covers the source code of SymmHub at commit `f2d19cb`, as two line-referenced code readings recorded it and a third reading of the source verified and corrected it. A fourth reading read the subgroup library `lib/sublib/src/sublib_core.js` and `lib/sublib/src/reidemeister.js` line by line (Section 12.8). A fifth reading covered the construction of a kaleidoscope `*p1...pn` without cone points, handles, or cross-caps line by line, from the normalization to the generators (Sections 6.2.1, 6.3.1, 6.4.1, 6.4.2, 6.5.1, 6.6.1, and 6.8), and a port of that code checked it numerically (Section 6.9). A sixth reading covered the catalogue of Euclidean and spherical groups line by line, from the dispatch of a signature to the splanes of each group, with hand traces of the domains and pairings, and a port of that code checked it numerically (Sections 7.1.1 to 7.1.9). It describes what the code does, with the file and line of each claim. It covers the group specification, the construction of hyperbolic orbifolds, the catalogue of Euclidean and spherical groups, the triangle groups, generators, point reduction, projections, borders, colouring, and the group-theoretic machinery in detail. It covers simulation, animation, export, and the document format at survey level only.

This document does not prove that any algorithm is correct, and it states no derivation of its own. Where a code reading marked a statement as its own inference, this document keeps that mark. The mathematics behind SymmHub, such as Conway's orbifold notation and Deaton's construction, belongs to other subject-matter references that cite the primary sources. The user interface, the navigation code, the scripting interface, and the bundled third-party libraries are out of scope, except for their licences.

## 2. Notation and Conventions

### 2.1. Citations into the Source

A location such as `lib/invlib/Group.js:191` is a path relative to the root of [SYMMHUB] at commit `f2d19cb`, followed by a line number or a range of lines. Every such location cites [SYMMHUB]. A line given as "about" a number is approximate in the code reading.

### 2.2. Symbols

- $z = x + iy$: A point of the plane, as a complex number.
- $\bar z$: The complex conjugate of $z$.
- $h$, $k$, $b$: The numbers of handles, cross-caps, and kaleidoscopes in an orbifold signature.
- $n$: The order of a cone point. $m$: The order of a corner.
- $\chi$: The orbifold Euler characteristic, which the code stores as `eulerchar`.
- $\kappa$: The curvature class that the code stores as `curvature`, with $-1$ for hyperbolic, $0$ for Euclidean, and $1$ for spherical.
- $p$, $q$, $r$: The vertices of an atom, each with a real angle or an imaginary length.
- $c$, $\rho$: The centre and the signed radius of a splane (Section 2.3) that is a sphere.
- $\mathbf n$, $\delta$: The unit normal and the distance from the origin of a splane that is a plane.
- $s$: The accumulated scale factor of the fold, which is the magnitude of the derivative of the composed maps.
- $\varepsilon$: A tolerance. Each section gives its value.

### 2.3. Terminology

- absolute: The unit circle, which bounds the Poincare disc model of the hyperbolic plane.
- splane: SymmHub's name for a sphere or a plane in $\mathbb R^3$ that acts on the plane $z = 0$ by inversion or reflection. In the plane it is a circle or a line.
- transform: An ordered list of splanes, which acts as the product of the reflections in them. The code calls it `ITransform`.
- fundamental domain: The region $F$ bounded by the sides of the domain, each side a splane, such that the images of $F$ under the group tile the plane.
- side pairing: The transform attached to one side of $F$. It maps the cell on the far side of that side into $F$, which the code calls a reverse transform.
- fold: One application of a side pairing to a point outside $F$. Point reduction is the repetition of folds until the point is in $F$.
- word: A string of generator letters that names a group element.
- atom: One of the twelve small hyperbolic polygons into which SymmHub cuts an orbifold, after Deaton. Atoms are sheets, pillows, and hats.
- tube: A cut between two cone points, with a length and a twist.
- slice: A cut from a cone point to a mirror, or between two corners, with a length only.
- feature: An entry of the lists that `hashOrbifold` returns, a pair `[key, index]` such as `["cornerPair", 1]` or `["slice", 2]`. A corner is `["corner", m]`, with its order $m$ in place of an index.
- corner pair: The feature `["cornerPair", k]` that takes the place of two consecutive corners of order 2. It becomes a mirror side of variable length between two right angles.
- sheet: An atom that holds three features of a mirror boundary and no cone point, written `[[f0, f1, f2], []]` in the code.
- wall: An extra splane on a side of $F$ that limits the side to a half-plane, so that $F$ can be non-convex.
- crown: The list of group elements whose images of $F$ cover a given box, used to draw content that overhangs $F$.
- view anchor: A group element, computed on the processor in double precision, that moves the view centre near $F$ before the per-pixel fold starts.
- layer: One full-screen rendering pass in SymmHub, such as the flat-colour tiles or the overlay.
- group maker: A SymmHub object that builds a group from parameters, with the method `getGroup()`.

### 2.4. Conventions

These conventions differ from those of other sources, and an implementer who reads SymmHub has to keep them.

1. All three geometries share one planar picture. The hyperbolic plane is the Poincare disc, the Euclidean plane is the plane itself, and the sphere is its stereographic image with the equator on the unit circle (Section 5.2).
2. A side test returns a signed Euclidean distance in the model. A positive value means the point is outside that side. The interior of $F$ is where every side gives a negative value, as the comment at `apps/orbifold/js/shaders/inversive.glsl.mjs:756` says: "we move the point into interior of fundamental domain, where all distance should be negative" (`lib/shaders/isplane.glsl.mjs:26-45`).
3. For a plane, the stored normal is the outward normal, so the domain side is where $\langle x, \mathbf n \rangle - \delta < 0$. For a sphere, a positive radius makes the inside of the circle the domain side, and a negative radius makes the outside the domain side (`lib/shaders/isplane.glsl.mjs:29-31`).
4. Each side carries the reverse transform, from the adjacent cell into $F$. The forward transforms are their inverses (`lib/invlib/Group.js:19`).
5. A generator is a lowercase letter, starting at `a` for the first side, and its inverse is the uppercase letter (`lib/invlib/GroupUtils.js:184-225`).
6. A twist is a fraction of the edge length in the range $-0.5$ to $0.5$ for handles, while the weld of a tube takes a raw hyperbolic distance (Section 6.5 to Section 6.7).
7. Folding works in Euclidean distances of the model. Border widths are in screen pixels, through the accumulated scale $s$, and never in hyperbolic distance (Section 11.1).
8. The orbifold construction holds a polygon as a vertex list and an edge list, and edge $i$ runs from vertex $i$ to vertex $i + 1$, as `produceGenerators` reads the ends of edge $i$ as `points[i]` and `points[(i+1)%points.length]` (`lib/orbilib/OrbifoldGeometrization.js:1432-1433` and 1475-1476). The polygon of a kaleidoscope is counterclockwise in the disc, its first vertex depends on the order of the welds, and, where Section 15.3 records no defect, its corners read clockwise follow the order of the signature (Sections 6.5.1 and 6.8).

## 3. Identity, Licence, and Repository

### 3.1. Authors and Licence

The file `LICENSE:1-3` reads "MIT License / Copyright (c) 2025 Vladimir Bulatov". The files `readme.html` and `outline.html` name three authors: Vladimir Bulatov, Chaim Goodman-Strauss, and Scott Vorthmann. The tutorial `tutorials/orbifold-notation.html` assigns `OrbifoldGeometrization.js`, `WallPaperGroup_General.js`, `WallpaperGroups_NonNegative.js`, and the orbifold user interface controller to Goodman-Strauss, and the renderer to Bulatov. It says the orbifold engine is shown "as it was working in Nov 2019".

`LICENSE` is the only licence file. The sources that SymmHub's authors wrote carry no per-file SPDX header.

### 3.2. Bundled Third-Party Code and Assets

This table lists the bundled code whose licence differs from, or is stated apart from, the MIT licence of SymmHub.

| Component | Location | Licence as stated |
| --- | --- | --- |
| dat.gui | `lib/datgui/dat/**`, `lib/extlib/dat.gui.min.js`, `lib/extlib/dat.gui.module.js` | Apache License 2.0, "Copyright 2011 Data Arts Team, Google Creative Lab" (header lines 5-9). Some copies are modified. |
| JSZip 3.10.1 with pako | `lib/extlib/jszip.js:7` | "Dual licenced under the MIT license or GPLv3", and pako under MIT |
| twgl.js 4.21.2 | `lib/extlib/twgl-full.module.js:1-2` | MIT, Gregg Tavares |
| Chart.js | `lib/extlib/chart.js.lib/*.js` | "Released under the MIT License" (line 5) |
| loadJS | `lib/uilib/LoadJS.js:1` | MIT, Scott Jehl, Filament Group |
| JamaJS | `lib/extlib/JamaJS/JamaJS.js` | No licence text. It says "Generated from Java with JSweet 2.0.0". |
| Image and texture assets | `apps/sympix/res/`, such as `haeckel_challenger` and `haeckel_kunst` | No credits and no licence file found |

### 3.3. What the Licence Permits for Escher

Escher is licensed GPL-3.0-or-later. `LICENSE:12-13` states the notice condition of the MIT licence: "The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software." A port of SymmHub code therefore carries the copyright line of `LICENSE:1-3` and the permission notice. Whether MIT code can be part of a GPL-3.0-or-later project is an open question (Section 17, item 1).

The SymmHub-authored files under `lib/` and the shaders fall under `LICENSE` and are the parts that matter for a port. JamaJS and the image assets have no stated licence, so they are not candidates for porting. The dat.gui code is user interface code and out of scope.

### 3.4. Repository Layout

These directories hold the code that this document describes:

- `lib/orbilib`: Orbifold geometrization, which `outline.html` describes as "hyperbolic metric calculations for arbitrary Conway orbifold signatures".
- `lib/grouplib`: Group makers, Dirichlet domains, presentations, subgroups, and colour permutations.
- `lib/invlib`: Inversive geometry, splanes, transforms, the `Group` class, and the packing of a group into a texture.
- `lib/symhublib`: The renderer `SymRenderer`, navigation, visualization layers, the crown, and export.
- `lib/shaders`: GLSL fragments, stored as JavaScript template strings in `.glsl.mjs` files.
- `lib/sublib`: A standalone low-index subgroup search and Reidemeister-Schreier rewriting. It has no Todd-Coxeter coset enumeration, although `outline.html:643` labels it "Standalone coset enumeration engine" (Section 12.8).
- `lib/color_groups`: A catalogue of subgroups with their coset permutations.
- `lib/drawlib`, `lib/uilib`, `lib/datgui`, `lib/extlib`: Drawing, user interface, and third-party code.
- `apps/`: Thin applications. `sympix` is the main image and video application, `orbifold` is the older standalone orbifold explorer, and `symsim` holds the partial differential equation (PDE) simulations. The others are `attractors`, `symdraw`, `overlay`, `animated_patterns`, `groups`, and `sample`.
- `tests/`: Node scripts and browser pages, described in Section 15.4.

`README.md` has 8 lines and says only to run `python3 -m http.server` and open port 8000 on localhost.

### 3.5. Documents in the Repository

These documents are worth reading before the code:

- `tutorials/orbifold-notation.html`, "Two Dollars of Symmetry", 568 lines: The best prose description of the orbifold algorithm and its parameters. Section 4.6 and Section 6 draw on it.
- `outline.html`, 914 lines, mostly CSS: An architecture outline with a module table, the idea of folding each pixel into the fundamental domain on the graphics card, and splanes as 4-vectors $(a, b, c, d)$ in a conformal model.
- `tutorials/orbifold-showcase.html`, 534 lines: A catalogue of more than 30 signatures with $\chi$, parameter counts, and the gestures that change them.
- `tests/groups/view_anchor.mjs` and `tests/symhublib/view_chain.mjs`: The best executable specifications of the fold and of the projection chain.
- `apps/sample/js/sample_main.js`: The smallest complete application.
- `notes.rtf`, 58 lines: Scratch notes on where shader fragments are assembled. It has no mathematics.

The page `tutorials/orbifold-experiments.html` is only a redirect. Comments in the code cite documents such as `docs/infinite_navigation_plan.md`, `docs/conformal_view_editing_assessment.md`, `docs/gpu_editing_overlay_assessment.md`, `docs/symmetry_tool_plan.md`, and `docs/shape_motions_plan.md`, which are not in the repository.

The tutorial `orbifold-notation.html` cites these sources: [CONWAY1992], [CONWAY-HUSON2002], [SOT2008], The Magic Theorem (2025) at themagictheorem.com, [THURSTON-GT3M] chapter 13, [MACBEATH1967], [CDGT2018], [DEATON1993], the page `https://chaimgoodmanstrauss.com/orbifold-geometrization/`, and the aperiodic monotile papers arXiv:2303.10798 and arXiv:2305.17743. The code readings did not read these sources. [DEATON1993] was read in full for `docs/research/fundamental-domains.md`, whose Section 8.15 states what it establishes and where the code departs from it.

The three files that cite Deaton's thesis give different first names and years:

- `lib/orbilib/OrbifoldGeometrization.js:32-33`: "Adam Deaton's 1994 Princeton University undergraduate thesis".
- `tutorials/orbifold-notation.html:420`: "A. A. Deaton, `<i>`Hyperbolic Orbifolds and Patterns`</i>`, senior thesis, Princeton University, 1993". Lines 224-226 name "Adam A. Deaton" and give the year 1993. Lines 229-230, 423, and 439-440 say that the code comment misremembers the year as 1994. Line 109 of the same tutorial still says "a lost 1994 Princeton thesis", and line 407 says "Deaton's own 1993 gallery".
- `outline.html:825`: "according to Derek Deaton's thesis", with no year anywhere in the file.

The title page of the thesis prints "Adam A. Deaton" and "Princeton, 1993" ([DEATON1993]). The tutorial's reference entry is therefore right, and the code comment, the outline, and line 109 of the tutorial are wrong.

## 4. Group Specification

### 4.1. Entry Points

Only the group maker `Group_Orbifold` takes a Conway orbifold signature. Its text box is created in `lib/orbilib/Group_Orbifold.js:112-125` (`createUI`), and its option `opt.orbifold` defaults to `'*442'` (`lib/orbilib/Group_Orbifold.js:62`). It wraps `WallPaperGroup_General` (`lib/orbilib/WallPaperGroup_General.js:75`), whose `updateTheGroup()` starts at about line 200.

A saved document gives the signature in its `name` field and the moduli as parameters named `{key}_{i}_l` for a length and `{key}_{i}_t` for a twist (`getParamsMap`, `lib/orbilib/WallPaperGroup_General.js:110-135`). Examples are in `apps/orbifold/presets/`, such as `23x.json`, `2223_a.json`, and `s532.json`.

### 4.2. Signature Parser and Grammar

The parser is `hashOrbifoldString` (`lib/orbilib/OrbifoldGeometrization.js:77`), a hand-written state machine in lines 77-170. It returns an object with the fields `errorMessage`, `handlesN`, `crosscapsN`, `conepoints`, `kaleidoscopes`, `eulerchar`, and `curvature`. It enforces no order of the features.

The parser reads each character as follows:

1. `o` or `O` adds a handle.
2. `x` or `X` adds a cross-cap.
3. `*` opens a new kaleidoscope with `kaleidoscopes.push([])`.
4. A digit before any `*`, or after an `o` or an `x`, is the order of a cone point.
5. A digit after a `*`, up to the next `o`, `x`, or `*`, is the order of a corner of that kaleidoscope.
6. An order of 10 or more is written in parentheses, such as `(12)`.
7. Any other character gives the error "Poorly formed expression". The parser skips no whitespace, so a space is an error.

The parser keeps two error fields: the success flag `noErrorQ` and the text `errorMessage`. Lexical errors set them as follows, in `lib/orbilib/OrbifoldGeometrization.js`:

| Input | Lines | Effect |
| --- | --- | --- |
| A single digit 0 or 1 | 128-137 | Sets `errorMessage` to `'Cone point of order ' + currentN` or `'Kaleidoscopic point of order ' + currentN` |
| An order of 0 or 1 in parentheses | 102-111 | The same messages, at the closing parenthesis |
| An unknown character | 151-154 | `noErrorQ = false` and `errorMessage = 'Poorly formed expression'` |
| A non-digit inside parentheses | 116-119 | Sets only `errorMessage = 'Poorly formed expression'`, and leaves `noErrorQ` true, so parsing continues |

Line 226 computes $\chi$ only when `errorMessage == ""`, so after any of these errors the returned `eulerchar` stays `NaN`.

### 4.3. Rejected and Unimplemented Signatures

After the lexical pass, the block that starts at line 173, `if (noErrorQ && handlesN == 0 && crosscapsN == 0)`, screens the small signatures without handles or cross-caps. It rejects these signatures, each by setting `noErrorQ = false` and a message:

| Signature | Lines | Message |
| --- | --- | --- |
| The empty signature | Within lines 170-215 | "Empty orbifold!" |
| `*n`, one kaleidoscope with one corner | 187-191, inside `else if (kaleidoscopes.length == 1)` at line 184 | "Single kaleidoscopic point -- not yet implemented" |
| `*pq` with $p \ne q$ | 193-199 | "Poorly formed orbifold", with the comment `// a teardrop of the form *pq, p!=q` |
| `n`, one cone point and no kaleidoscope | 203-211 | "Single rotation point -- not yet implemented" |
| `pq` with $p \ne q$ | 212-219 | "Poorly formed orbifold", with the comment `// a teardrop of the form pq, p!=q` |

These signatures fall through the screen without an error:

- `*pp` and `pp`, the teardrops with equal orders.
- `*` alone. It gives `kaleidoscopes = [[]]`, so `lastScope.length == 0` at line 186 and neither test at lines 187 and 193 matches. Line 228 then gives $\chi = 1$, and lines 242-249 give `curvature = 1`.
- Every signature with a handle or a cross-cap, because the block at line 173 requires both counts to be zero.
- Every signature with three or more cone points, every signature with two or more kaleidoscopes and no cone points, and every signature with both corners and cone points.
- A signature with a non-digit inside parentheses, because that path leaves `noErrorQ` true (Section 4.2).

### 4.4. Euler Characteristic and Classification

Lines 226-237 compute the orbifold Euler characteristic as

$$\chi = 2 - 2h - k - b - \sum_{\text{cones}} \left(1 - \frac{1}{n}\right) - \frac{1}{2} \sum_{\text{corners}} \left(1 - \frac{1}{m}\right)$$

and lines 242-251 classify the geometry with these thresholds:

| Condition | `curvature` | Geometry |
| --- | --- | --- |
| $\chi > -0.001$ and $\chi < 0.01$ | $0$ | Euclidean |
| $\chi > -0.001$ and $\chi \ge 0.01$ | $1$ | Spherical |
| $\chi \le -0.001$ | $-1$ | Hyperbolic |

A comment there states that $*237$, with $\chi = -1/84$, is the smallest negative value and that $*235$, with $\chi = 1/60$, is the smallest positive value. That holds for the particular spherical groups, but not for the seven spherical families, whose $\chi$ is $2/n$ for `nn`, $1/n$ for `*nn`, `n*`, `nx`, and `22n`, and $1/(2n)$ for `2*n` and `*22n` (the orders of Section 8.2 of `docs/research/orbifold-notation.md` with $\chi = 2/\lvert G \rvert$). For a large enough $n$, $\chi$ falls below 0.01, and the code gives such a spherical signature the curvature 0 (Section 7.1.7).

The tutorial `orbifold-notation.html` gives the same accounting as costs: a handle costs 2, a cone point of order $n$ costs $(n-1)/n$, a `*` costs 1, a corner of order $m$ costs $(m-1)/(2m)$, and a cross-cap costs 1. The outline `outline.html` describes the Magic Theorem with a total of 2 for spherical groups, 0 for Euclidean groups, and less for hyperbolic groups.

### 4.5. Parameter Count

`countParameters` (`lib/orbilib/OrbifoldGeometrization.js:266-277`) computes the dimension of the space of moduli as

$$N_{\text{par}} = -6 + 6h + 3k + 2 N_{\text{cones}} + 3b + N_{\text{corners}}$$

The last term is the total number of corners, because the code adds `kal.length` for each kaleidoscope `kal`, whose elements are the corner orders. Its comment at lines 267-268 says the count is "not really used, except in messages in the gui". The tutorial gives the same formula and cites Thurston's Corollary 13.3.7, $\dim T(O) = -3\chi(X_O) + 2k + l$ [THURSTON-GT3M].

### 4.6. Other Group Families

The menu of group makers is the array `groupsData` at `lib/grouplib/GroupMakerFactory.js:37-57`. It has 16 entries: Wallpaper, Frieze, Spherical, KLM, KLMN, KLx, 5 Splanes, 6 Splanes, Orbifold, Cyclic, oN, `*22222`, `22222`, `*222222`, Grandma, and Maskit 2-torus. The factory function at lines 60-65 returns `ObjectFactory({infoArray: groupsData, defaultName: defName})`. Only Orbifold parses a signature, and each other maker takes its own parameters.

| Maker | Group family | Parameters and source |
| --- | --- | --- |
| `Group_WP` | The 17 wallpaper groups by name, with alternative domain shapes | `a`, `b`, `c`, `angle_a`, `angle_b` (`lib/grouplib/WallpaperGroups.js:10-29`, `WallpaperDomainShapes` at line 177) |
| `Group_Frieze` | The 7 frieze groups, named $*22\infty$, $22\infty$, $2{*}\infty$, $\infty{*}$, $*\infty\infty$, $\infty\times$, and $\infty\infty$ | Fixed to the Euclidean plane |
| `Group_Spherical` | `*nn`, `nn`, `nx`, `n*`, `*22n`, `2*n`, `22n`, `*332`, `332`, `3*2`, `*432`, `432`, `*532`, `532` | `lib/grouplib/Group_Spherical.js:26-40` |
| `Group_KLM` | Triangle groups with integers $K$, $L$, $M$ from 0 to 20 | `*` switches between the reflection group and its rotation subgroup of index 2, and `hasR` adds the unit circle as a mirror (`lib/grouplib/Group_KLM.js:42-110`). Section 7.2 describes the construction. |
| `Group_KLMN`, `Group_KLx` | Quadrangle groups with twist, aspect, and bend, and a related family | Not detailed in the code readings |
| `Group_5splanes`, `Group_6splanes` | Reflection groups of 5 or 6 splanes | Angles and radii |
| `Group_Cyclic`, `Group_ON`, `Group_s22222`, `Group_22222`, `Group_s222222` | One loxodromic element, a torus with one cone point, and related moduli families, imported from the "Orb" repository on 2026-10-26 | Bends and twists that can leave the disc, so the groups need not be Fuchsian |
| `Group_Grandma`, `Group_Maskit2` | Two-generator groups from Indra's Pearls, Box 21, and a Maskit family | Not detailed in the code readings |
| `Group_OrbifoldPresented` | An orbifold group with a presentation derived from an untwisted copy (Section 14.2) | A class defined in `lib/grouplib/GroupMakerFactory.js:29-35` |

## 5. Representation of Isometries

### 5.1. Splanes and Transforms

A splane is `iSplane {v:[4], type}` in `lib/invlib/ISplane.js`. Its type is one of these constants, defined in JavaScript at `lib/invlib/ISplane.js:8-12` and in GLSL at `lib/shaders/isplane.glsl.mjs:3-7` with the same values:

| Value | JavaScript | GLSL | Meaning |
| --- | --- | --- | --- |
| 0 | `SPLANE_NONE` | `SPLANE_IDENTITY` | The identity transform |
| 1 | `SPLANE_SPHERE` | `SPLANE_SPHERE` | Reflection in a sphere |
| 2 | `SPLANE_PLANE` | `SPLANE_PLANE` | Reflection in a plane |
| 3 | `SPLANE_POINT` | `SPLANE_POINT` | Not described in the code readings |
| 4 | `SPLANE_INFINITY` | `SPLANE_INFINITY` | Not described in the code readings |

The comment `// 0 - sphere, 1 plane` at `lib/shaders/isplane.glsl.mjs:12` disagrees with the defines, and the defines are what the code uses. The default splane is a sphere with `v = [0,0,0,1]` (`lib/invlib/ISplane.js:22-23`). The constructor `iPlane(v,ends)` (lines 86-95) normalizes only `v[0..2]` and leaves `v[3]` unchanged, and `iSphere(v,ends)` is at line 104. The vector `v` holds $(c_x, c_y, c_z, \rho)$ for a sphere, with a signed radius, and $(n_x, n_y, n_z, \delta)$ for a plane, with a unit normal. The group acts in the plane $z = 0$, so $c_z = 0$ and $n_z = 0$ in practice.

A transform is an ordered array of splanes, applied as the product of the reflections in them (`ITransform`, `lib/invlib/ITransform.js:33`). These are its operations:

- `transform(p)` reflects through each splane of the list in the hyperboloid form, with `iReflectH4` (`lib/invlib/Inversive.js:134`): $X \mapsto X - 2P\langle P, X\rangle / \langle P, P\rangle$.
- `inverse` is the reversed list (`GroupUtils.makeInverseTransform`, `lib/invlib/GroupUtils.js:164`).
- `concat` appends the lists and appends the word of each transform.

### 5.2. One Planar Picture for Three Geometries

SymmHub expresses every group as splanes acting on the plane $z = 0$:

- Hyperbolic: The Poincare disc, with reflection circles orthogonal to the unit circle. The orbifold construction normalizes its domains to the disc, with the unit circle as the absolute (header of `lib/orbilib/OrbifoldUITool.js`).
- Euclidean: Lines, and circles of finite radius, in the same plane.
- Spherical: The stereographic plane, with the equator on the unit circle. A comment in the spherical section of `lib/orbilib/WallpaperGroups_NonNegative.js` says "treated in the plane, scaled with the equator at the unit circle".

The shader does not know which geometry it draws. The geometries differ only in their data. The field `curvature` is still passed to `getTransformsForTexture` and to a few helpers, such as `transformFromCenterToPoint` (`lib/invlib/ComplexArithmetic.js:584`), because the transform that moves the centre to a point differs by geometry. The navigator sets its mode, one of euclidean, hyperbolic, spherical, or inversive, from the kind of the group (`modeFromGroup`, `lib/symhublib/InversiveNavigator.js:441-448`, modes at lines 50-57).

### 5.3. Side Tests and Reflections

The processor and the graphics card use the same side test. On the processor it is `U4.sigDistanceSP`, which is `iDistanceU4` (`lib/invlib/Inversive.js:84`). On the graphics card it is `iDistance` (`lib/shaders/isplane.glsl.mjs:26-45`), with `struct iSPlane {int type; vec3 center; float radius;}` at lines 11-15. For a point $x$:

- Sphere: $d = \lVert x - c \rVert - \lvert \rho \rvert$, negated when $\rho < 0$. The code takes the point as its parameter `v` and shifts it with `v -= s.center` before the distance.
- Plane: $d = \langle x, \mathbf n \rangle - \delta$, with $\mathbf n$ stored in `.center` and $\delta$ in `.radius`. The constructor `iPlane` normalizes the normal (`lib/shaders/isplane.glsl.mjs:52`).
- Any other type: $d = 1$, which the code comments as "outside of undefined object".

The reflection `iReflect` (`lib/shaders/isplane.glsl.mjs:114-133`) also updates the scale factor $s$:

- Plane: $x \mapsto x - 2 \langle x - \delta \mathbf n, \mathbf n \rangle \mathbf n$, with $s$ unchanged.
- Sphere: $x \mapsto c + (x - c)\, \rho^2 / \lVert x - c \rVert^2$, and $s \mapsto s\, \rho^2 / \lVert x - c \rVert^2$.

### 5.4. The Matrix Route

`lib/invlib/MobiusMatrix.js` classifies a planar transform by its $2 \times 2$ complex matrix, and `mobiusToSplanes` (line 208) converts a Moebius matrix back to at most 4 reflections. The header at lines 10-17 explains why the route exists: the hyperboloid factorization of a few hundred reflections lost the trace of a half-turn to an error of $5 \times 10^{-5}$ on long words. Its constants are `MOBIUS_PRECISION=1e-6` (line 225) and `PLANAR_EPS=1e-12`.

`lib/invlib/ComplexArithmetic.js` also has a class for maps that preserve the disc, `diskPreservingMobiusTransform {a,c,opQ}`, which represents

$$z \mapsto \frac{aZ + \bar c}{cZ + \bar a}, \qquad Z = z \text{ or } Z = \bar z$$

with `opQ` choosing between the two forms. Its `composeWith`, `inverse`, and `normalize` are at about lines 300-340. Its `equal` uses `SHORTEPSILON=1e-7` (`lib/invlib/Utilities.js:283`).

## 6. Hyperbolic Orbifold Construction

### 6.1. Pipeline

Every hyperbolic signature, including those with handles, cross-caps, cone points, mirrors, and corners, goes through one pipeline in `lib/orbilib/OrbifoldGeometrization.js`, after Deaton, Thurston, and Goodman-Strauss:

1. `hashOrbifoldString` parses the signature (Section 4.2).
2. `hashOrbifold` normalizes the features (Section 6.2).
3. `atomizeOrbifold` cuts the orbifold into atoms (Section 6.3).
4. `unfoldAtom` lays out each atom in the disc (Section 6.4).
5. `assembleFundamentalDomain` welds the atoms into one polygon (Section 6.5).
6. `produceGenerators` builds one transform per side (Section 8.1).

Euclidean and spherical signatures do not take this pipeline (Section 7).

### 6.2. Normalization

`hashOrbifold` (line 279) follows Deaton, as its comment at lines 289-297 says. It applies these steps:

1. It pairs cone points of order 2 into `conePair` entries (lines 326-345). A single cone point of order 2 left over goes onto the mirror list as a `fold`, or stays a `conePt` when the orbifold has no mirror.
2. It opens each handle into two `["handle",i]` entries and each cross-cap into a `["cap",i]` entry.
3. It merges each extra kaleidoscope as a `band`, with all corners placed on the first mirror. A comment at about line 380 calls this "room for improvement".
4. It pairs consecutive corners of order 2 into `cornerPair` entries.

Lines 46-53 define the keys of edges and parameters:

- `keys = ["handle","cap","conePt","conePair","tube","mirror","mirrorRedundant","cornerPair","fold","band","slice","cuttingEdge","edge"]`
- `lengthKeys = ["handle","cap","fold","conePt","conePair","tube","band","slice","cornerPair"]`
- `twistKeys = ["handle","conePair","tube"]`

#### 6.2.1. A Single Kaleidoscope

For a signature `*p1...pn` with one mirror boundary and no cone points, handles, or cross-caps, `hashOrbifold` returns an empty `conepointList` and builds `kaleidoList` in two passes (`lib/orbilib/OrbifoldGeometrization.js:362-429`):

1. It pushes `["corner", m]` for each corner of the first kaleidoscope, in the order of the input (lines 378-379). The parser appends each digit to its kaleidoscope in input order and sorts nothing (lines 147-165), so the list is neither sorted nor rotated.
2. It scans that list once from the left (lines 402-429). A corner of order 2 that follows a pending corner of order 2 becomes `["cornerPair", k]`, with `k` counted from 1 in `parCounts["cornerPair"]` (lines 404-411). A pending corner of order 2 that no second one follows goes back on the list as `["corner", 2]`, before the next corner (lines 419-421) or at the end (lines 426-429).

The scan pairs greedily and does not wrap around. The comment at lines 399-400 reads "since there are bands at the end of this list, don't have to check first vs last", and a single kaleidoscope has no band, so a corner of order 2 at the end of the input never pairs with one at the start. The plain string `"mirror"` enters the list only for a kaleidoscope without corners (lines 364-372).

A corner pair takes the place of two corners and carries a length, because `cornerPair` is in `lengthKeys` (line 50). It carries no twist, because it is not in `twistKeys` (line 53). `countParameters` gives $-6 + 3 + n = n - 3$ for one kaleidoscope with $n$ corners (lines 270-275), and Section 6.3.1 counts the same total from the slices and pairs.

The code reading traced these lists by hand and did not run the code. In this table and in the rest of Section 6, `c3` stands for `["corner", 3]`, `P1` for `["cornerPair", 1]`, and `s1` for `["slice", 1]`:

| Signature | `kaleidoList` | Corner pairs |
| --- | --- | --- |
| `*2223` | `[P1, c2, c3]` | 1 |
| `*22222` | `[P1, P2, c2]` | 2 |
| `*3333` | `[c3, c3, c3, c3]` | 0 |
| `*2345` | `[c2, c3, c4, c5]` | 0 |
| `*23456` | `[c2, c3, c4, c5, c6]` | 0 |
| `*2233` | `[P1, c3, c3]` | 1 |
| `*2332` | `[c2, c3, c3, c2]` | 0 |

`*2233` and `*2332` name the same orbifold, and the code gives them different features, because the scan does not wrap. `*2222` is Euclidean, so `lib/orbilib/WallPaperGroup_General.js:227` and 252-253 send it to the catalogue of Section 7.1 and not to `hashOrbifold`.

The entry of `apps/orbifold/bug list.txt` at lines 3-4 reads "2*22 does not get the slice parameter correctly. Perhaps it is instead a corner pair." and "22* as well." Both signatures are Euclidean and take the catalogue path, where `lib/orbilib/WallpaperGroups_NonNegative.js:125-127` registers one `slice` parameter for `22*`, `2*22`, and `*2222`, and lines 837-838 read `["slice",1]` for `2*22` and `22*`, so the current code shows no mismatch for these two. The entry's term "corner pair" belongs to `hashOrbifold` only. The code reading inferred that the entry does not concern the hyperbolic pipeline of this section.

### 6.3. Atomization

`atomizeOrbifold` (line 452) splits the feature lists into atoms until none is left. It cuts with tubes, between two cone points, and with slices, from a cone point to a mirror or between two corners. The comment at lines 437-447 justifies "option #2": cut the cone points out to the mirrors first, then cut the mirrors. This order gives few twist parameters, and cutting new slices last keeps the cuts a tree.

The tutorial describes Deaton's 9 cut rules and 12 atoms, which are 4 sheets, 4 pillows, and 4 hats. The thesis states nine cuts and twelve atoms, and calls the hats pillow cases ([DEATON1993] pp. 19 to 22). Its atoms match the tutorial's entry for entry by a reader's comparison. Section 8.15 of `docs/research/fundamental-domains.md` matches each cut with a key of Section 6.2 and records where the code departs from the thesis. Two departures concern this section. The thesis applies its rules in a fixed order in which the cut from a cone point of order 2 to a mirror comes before the pairing of cone points, while `hashOrbifold` pairs them first. The thesis also cuts every pillow and pillowcase in half into sheets, where `unfoldAtom` has a case for each.

#### 6.3.1. Atomization of a Kaleidoscope

For the list of Section 6.2.1, the loop `while ( (cones.length > 0 || kaleidos.length > 0))` (`lib/orbilib/OrbifoldGeometrization.js:461`) skips the branches for cone points at lines 477, 482, 490, and 505. It also skips the branch at line 463, which tests `kaleidos[0]=="mirror"` and so matches only the plain string of a kaleidoscope without corners. Each pass then takes one of three branches:

1. While more than three features remain (line 533), it increments `sliceN`, pops the last feature as `kal0` and the one before it as `kal1` (lines 536-537), and swaps the two when `kal0` is not a corner (lines 538-543). It puts the new slice at the front with `kaleidos.unshift(["slice", sliceN])` (line 544) and pushes the sheet `[[kal0, kal1, ["slice", sliceN]], []]` (line 545).
2. When exactly three features remain (line 547), it pops them as `kal0`, `kal1`, and `kal2`, with `kal0` the last. Three conditional swaps (lines 553-567) move the corners to the front, and it pushes the sheet `[[kal0, kal1, kal2], []]` (line 569).
3. Otherwise it empties the list without an atom, at `else {kaleidos = []}// this should never happen` (line 571).

After the loop it sets `parCounts["slice"] = sliceN` (line 575). The comment at lines 448-449 explains the front insertion, quoted here in a code span because it holds semicolons: `the new slices are put on the other end from what's sliced next; i.e. new slices are sliced last; this ensures a tree-like structure in the cutting.`

Each pass of branch 1 removes two features and adds one, so a list of $f \ge 3$ features gives $f - 3$ slices and $f - 2$ sheets. For $n$ corners with $k$ corner pairs, $f = n - k$, so the code makes $n - k - 3$ slices, each with a length, and the total of lengths is $n - 3$. Each slice appears in two sheets: the sheet that branch 1 makes with it, and the later sheet that pops it from the list. The code reading counted these from the steps of the code.

A sheet lists its features in the order in which the list pops them, which is the reverse of the order of the input, except where a swap moves a feature. The swap of branch 1 acts when the last feature is a corner pair or a slice, and the swaps of branch 2 act when the last feature or the one before it is not a corner. The swaps exist for the precondition of `unfoldAtom` that real features come first (Section 6.4.1). Section 6.8 traces their effect on the order of the corners.

The code reading traced these sheets by hand, in the notation of Section 6.2.1:

| Signature | Sheets in `atomList` order | Slices |
| --- | --- | --- |
| `*2223` | `[c3, c2, P1]` | 0 |
| `*22222` | `[c2, P1, P2]`, after the swap at lines 563-567 | 0 |
| `*3333` | `[c3, c3, s1]`, `[c3, c3, s1]` | 1 |
| `*2345` | `[c5, c4, s1]`, `[c3, c2, s1]` | 1 |
| `*23456` | `[c6, c5, s1]`, `[c4, c3, s2]`, `[c2, s2, s1]`, after the swap at lines 563-567 | 2 |

The first cut of `*2345` separates the corners 5 and 4 from the corners 3 and 2, which agrees with the example $*2345 \to *23\infty + *\infty 45$ of `tutorials/orbifold-showcase.html:200` and with the same example in [DEATON1993] p. 25. The thesis writes this cut as the rule $*abcd \dots \to *\infty_l ab + *\infty_l cd \dots$ and does not say where in the list of corners it matches, so it neither confirms nor contradicts the choice of the last two features for five or more corners. It does not say whether two corners of order 2 pair across the end of the signature (Section 6.2.1).

### 6.4. Atom Geometry

`unfoldAtom` (line 648) has 12 cases: sheet, pillow, and hat, each by how many of its vertices are real or imaginary. A real vertex is a cone point of order $n$ with the angle $\pi/n$, or, in a sheet, a corner of order $m$ with the angle $\pi/m$ (`p.re = PI / a[1]`, line 679). An imaginary vertex $i\ell$ stands for a geodesic edge of length $\ell$ perpendicular to its neighbours, with $\ell$ from `getLength(key,group)`. The tutorial says that atoms are solved by the generalized law of cosines in the hyperboloid model. The thesis gives each parameter of a generalized triangle as the angle at a real vertex or the length of an imaginary vertex. That this is the code's distinction between real and imaginary vertices is a reader's reading. The thesis solves only the generalized triangle with two imaginary vertices, in the hyperboloid model ([DEATON1993] pp. 6 to 9). Section 8.15.1 of `docs/research/fundamental-domains.md` gives its formula, whose sign before the cosine differs from case C of Section 6.4.2.

The generalized law of cosines is `edgeLength(p,q,r)` at line 588. For each vertex it takes

$$(c_v, s_v) = \begin{cases} (\cos v, \sin v) & \text{for a real angle } v \\ (\cosh \ell_v, \sinh \ell_v) & \text{for an imaginary vertex } i\ell_v \end{cases}$$

and computes

$$L = \frac{c_p c_q + c_r}{s_p s_q}$$

It returns $\operatorname{arcosh} L$ when $p$ and $q$ are both real or both imaginary, and $\operatorname{arsinh} L$ otherwise.

A pillow is split into two generalized triangles around a median. For three cone points (line 784), with real angles $p$, $q$, and $r$:

$$A_2 = \arcsin \sqrt{\frac{1}{1 + \left(\cot p + \dfrac{\cos q}{\sin p \cos r}\right)^2}}, \qquad A_1 = p - A_2$$

$$M = \operatorname{arcosh} \frac{\cos q}{\sin A_1}, \qquad a_1 = \operatorname{arcosh} \frac{\cos A_1}{\sin q}, \qquad a_2 = \operatorname{arcosh} \frac{\cos A_2}{\sin r}$$

Imaginary vertices enter with half lengths, as $\cosh(\ell/2)$ (for example lines 829-833 and 880-884). For three imaginary vertices of lengths $\ell_p$, $\ell_q$, and $\ell_r$ (line 936):

$$A_2 = \operatorname{arsinh} \sqrt{\frac{1}{-1 + \left(\coth \dfrac{\ell_p}{2} + \dfrac{\cosh(\ell_q/2)}{\sinh(\ell_p/2) \cosh(\ell_r/2)}\right)^2}}$$

Hats use right-angled triangles with the angles $\pi/2$ (`HPI`) and $\pi/n$ (`PI/b[1]`) at lines 1000-1130. For an imaginary cone point, a hat takes half the length, `q.im = getLength(b,group)/2`.

The vertices of each polygon are laid out in the disc by turtle moves. `poincareTurtleMove(last,current,angle,distance)` (`lib/invlib/ComplexArithmetic.js:459`) turns by the angle and moves by the hyperbolic distance. `poincarePt(d)` puts a point at hyperbolic distance $d$ from the origin on the real axis, at $\tanh(d/2)$ (line 237).

`useMirrorQ` (line 639) marks which edges of an atom are real mirrors and which are `mirrorRedundant`.

A cutting edge of a pillow can carry two extra points that define a perpendicular half-plane, the wall (comments at lines 770 and 811-860). Walls let $F$ be non-convex while the graphics card still picks the right generator (Section 9.4).

#### 6.4.1. The Four Sheet Cases

`unfoldAtom` treats an atom as a sheet when its list of cone points is empty (`lib/orbilib/OrbifoldGeometrization.js:670`) and names its features `a`, `b`, and `c` (lines 673-675). The comment at line 653 states a precondition: "we assume we've gotten our features in order of real first, then imaginary". Only a feature with the key `"corner"` is real, with the angle `PI / m` for the order `m`. Every other feature is imaginary, with the length `getLength(key, group)`. The case depends on how many leading features are corners, and `mirror` stands for `["mirror",0]` in this table:

| Case | Test | Real features | Vertices | Edge list |
| --- | --- | --- | --- | --- |
| A | `c` is a corner (line 676) | `a`, `b`, `c` | 3 | `[mirror, mirror, mirror]` (line 688) |
| B | `b` is a corner (line 691) | `a`, `b` | 4 | `[mirror, mirror, c, useMirrorQ(c)]` (line 707) |
| C | `a` is a corner (line 710) | `a` | 5 | `[mirror, b, useMirrorQ(b), c, useMirrorQ(c)]` (line 730) |
| D | Otherwise (line 732) | None | 6 | `[useMirrorQ(a), b, useMirrorQ(b), c, useMirrorQ(c), a]` (line 761) |

Edge $i$ runs from vertex $i$ to vertex $i + 1$, and the last edge closes back to vertex 0 (Section 2.4, item 8). An edge labelled with a feature key, such as `["slice",1]` or `["cornerPair",1]`, is the side that stands for that imaginary vertex. The edge after it is labelled `useMirrorQ` of that key.

`useMirrorQ` (lines 639-643) returns `["mirror",0]` for the keys `corner`, `fold`, `cornerPair`, and `band`, and `["mirrorRedundant",0]` for every other key, as line 641 states: `if (["corner","fold","cornerPair","band"/*,"conjCuttingEdge"*/].indexOf(key[0]) < 0) {return ["mirrorRedundant",0]}`. Its comment at line 640 reads "In an unfolded atom, some mirror lines are redundant and will not be used". In a kaleidoscope, the edge after a corner pair is a mirror and the edge after a slice is redundant.

Every case puts vertex 0 at the origin `O` and vertex 1 at `poincarePt(R)` on the positive real axis, with `R = edgeLength(p, q, r)`. Each later vertex is a turtle move:

- Case A (lines 679-688): `p`, `q`, and `r` are the angles of `a`, `b`, and `c`. With `P = edgeLength(q, r, p)`, vertex 2 is `poincareTurtleMove(O, y, -q.re, P)`.
- Case B (lines 694-707): `r.im = getLength(c,group)`. Vertex 2 is `z = poincareTurtleMove(O, y, -q.re, P)`, and vertex 3 is `poincareTurtleMove(y, z, -HPI, r.im)`, so the side from vertex 2 to vertex 3 is the side of `c`.
- Case C (lines 713-730): `q.im` and `r.im` are the lengths of `b` and `c`. Vertex 2 is `poincareTurtleMove(O, y, -HPI, q.im)`, which ends the side of `b`. Vertex 3 is a move of length `P` with the turn `-HPI`, and vertex 4 a move of length `r.im` with the turn `-HPI`, which ends the side of `c`.
- Case D (lines 734-761): `p.im`, `q.im`, and `r.im` are the three lengths, with `Q = edgeLength(r, p, q)`. Four moves, each with the turn `-HPI`, add the vertices at the lengths `q.im`, `P`, `r.im`, and `Q` (lines 744-758). The closing side from vertex 5 to the origin is the side of `a`.

No case computes its last side, or checks that the sheet closes with the angle of `a` at the origin and right angles at the ends of each imaginary side. The closure rests on the formulas of Section 6.4.2 (Section 17, item 11).

`poincareTurtleMove(last,current,angle,distance)` (`lib/invlib/ComplexArithmetic.js:459-467`) rotates `last` about `current` by `angle`, with `poincareMobiusRotationAboutPoint` (lines 407-414), and moves the image along the geodesic ray from `current` through it, to the hyperbolic distance `distance` from `current` (lines 460-466). The new side therefore makes the angle $\lvert$`angle`$\rvert$ with the side back to `last`, which is the interior angle at `current`. The comment at lines 457-458, "starting at current, facing away from last, turn by angle and move distance", disagrees with the code: an angle of 0 sends the turtle back towards `last`, and an angle of $\pi$ continues straight on. At the origin the rotation is $z \mapsto e^{i\theta} z$, so a negative angle turns clockwise, and from the origin and a vertex 1 on the positive real axis, vertex 2 lies above the real axis. Each sheet is therefore counterclockwise in the disc. The code reading inferred the action of the rotation and the orientation from the formulas, and no comment states them.

#### 6.4.2. Edge Lengths of a Sheet

`edgeLength` (`lib/orbilib/OrbifoldGeometrization.js:588-619`) decides whether a vertex is real by the test `.re != 0` (lines 590, 598, and 606). The third argument `r` enters only through $\cos r$ or $\cosh r$ (lines 606-611), so it never changes the choice between $\operatorname{arcosh}$ and $\operatorname{arsinh}$ (lines 612-618). A length of exactly 0 has `.re == 0`, so the code treats it as imaginary and divides by $\sinh 0 = 0$, as the code reading inferred. The comment at lines 584-586 reads "The length of the edge between p and q in a triangle with opposite angle r", and adds "Some details are missing."

The sheet cases make these calls, with $p$, $q$, and $r$ the angles or lengths of `a`, `b`, and `c` in the order of each call:

| Case | Call | Kinds of the first two arguments | Function | Expression |
| --- | --- | --- | --- | --- |
| A | `R = edgeLength(p,q,r)`, `P = edgeLength(q,r,p)` | Real, real | $\operatorname{arcosh}$ | $\dfrac{\cos p \cos q + \cos r}{\sin p \sin q}$, and the same with $(q, r, p)$ |
| B | `R = edgeLength(p,q,r)` | Real, real | $\operatorname{arcosh}$ | $\dfrac{\cos p \cos q + \cosh r}{\sin p \sin q}$ |
| B | `P = edgeLength(q,r,p)` | Real, imaginary | $\operatorname{arsinh}$ | $\dfrac{\cos q \cosh r + \cos p}{\sin q \sinh r}$ |
| C | `R = edgeLength(p,q,r)` | Real, imaginary | $\operatorname{arsinh}$ | $\dfrac{\cos p \cosh q + \cosh r}{\sin p \sinh q}$ |
| C | `P = edgeLength(q,r,p)` | Imaginary, imaginary | $\operatorname{arcosh}$ | $\dfrac{\cosh q \cosh r + \cos p}{\sinh q \sinh r}$ |
| D | `R`, `P`, `Q` | Imaginary, imaginary | $\operatorname{arcosh}$ | $\dfrac{\cosh p \cosh q + \cosh r}{\sinh p \sinh q}$, and its rotations |

Section 8.7 of `docs/research/fundamental-domains.md` compares each row with the published formulas.

`getLength(key,group)` (lines 624-630) returns `group.guiParams[key[0]+"_"+abs(key[1]).toString()+"_l"]` (line 629), or 1.1 when `group.parCounts[key[0]]` is less than `abs(key[1])` (lines 625-627). For a kaleidoscope, `parCounts["slice"]` and `parCounts["cornerPair"]` equal the numbers of slices and pairs (Sections 6.2.1 and 6.3.1), so a sheet does not reach the value 1.1, as the code reading inferred.

### 6.5. Assembly and Welding

`assembleFundamentalDomain` (line 1195) applies these steps:

1. It unfolds every atom, in the order of `atomList` (line 1209), and takes the first as the working polygon.
2. It recentres that first atom with `recenterDomain`, which moves the Euclidean mean of its vertices in the disc to the origin with `poincareMobiusTranslateToO` (lines 1145-1164 and 1215).
3. It queues the tube and slice edges of the working polygon (lines 1229-1232) and welds one of them at a time to the atom that holds the same edge, with `poincareMobiusEdgeToEdge([v1,v2],[w1,w2],shift)` (`lib/invlib/ComplexArithmetic.js:472`). The `shift` is the twist of a tube as a raw hyperbolic distance, applied as the translation $\tanh(\text{shift}/2)$ (line 482), and 0 for a slice (`lib/orbilib/OrbifoldGeometrization.js:1287-1294`).
4. It removes the welded edge from the polygon, a tube at lines 1301-1334 and a slice at lines 1335-1343. The edges that remain keep keys such as `handle`, `cuttingEdge`, `band`, `cap`, `conePair`, `fold`, and `mirror`.
5. After each weld it recentres twice (`recenteringN=2`, lines 1351-1365), each time moving the Euclidean mean of the vertices to the origin.
6. After the last weld it applies `poincareMobiusTranslateToO(new complexN(-.001,-.002))`, the map that takes the point $(-0.001, -0.002)$ to the origin, as a small shift for general position (lines 1370-1371).

The comment at lines 1136-1138 explains the recentring: it keeps "the domain from growing off into a numerically less stable regime, closer to the boundary of the Poincare disk".

#### 6.5.1. Welding Sheets Along a Slice

For a kaleidoscope, every open attachment is a slice. Each pass of the loop at line 1234 takes these steps:

1. It takes the last queued slice with `openAttachments.pop()` (line 1237).
2. It rotates the vertex and edge lists of the working polygon together until edge 0 is that slice (lines 1240-1244). Edge 0 then runs from vertex 0 to vertex 1. `isSameEdgeTypeQ` (lines 1187-1193) compares the key and the index, and its comment at line 1188 reads "this is a stupid kludge".
3. It takes atoms from the front of the queue of unwelded atoms and rotates each in the same way until edge 0 is the same slice (lines 1251-1269). An atom without that slice goes to the back of the queue (lines 1266-1268). The code reading inferred that no line guards against a slice that no atom holds.
4. It queues the other slices of the found atom (lines 1278-1282).
5. It maps the atom with `poincareMobiusEdgeToEdge([nextAtom[0][0],nextAtom[0][1]],[workingFD[0][0],workingFD[0][1]],shift)` and `moveDomain` (lines 1295-1298), with `shift` equal to 0.
6. It records the two ends of the slice in `internalFDedges` for drawing (line 1337).
7. It splices the two lists at lines 1340-1341: `workingFD = [(workingFD[0].slice(2)).concat(newAtom[0].slice(2)), (workingFD[1].slice(2)).concat(newAtom[1].slice(2))]`.

`poincareMobiusEdgeToEdge` composes five maps (`lib/invlib/ComplexArithmetic.js:474-490`): it moves $v_1$ to the origin, rotates $v_2$ onto the positive real axis, translates along the real axis by `shift`, rotates the positive real axis onto the direction of $w_1$ as seen from $w_2$, and moves the origin to $w_2$. So $v_1$ goes to $w_2$, and when the two arcs have the same length and `shift` is 0, $v_2$ goes to $w_1$. Every map has `opQ` true, so the composition preserves orientation. The comment at lines 469-470 reads `This takes the arc v1-v2 to the arc w1-w2; It is presumed that the arcs have the same length.`, and the code does not check the lengths. The code reading inferred the endpoint correspondence from the composition, and no comment states it. From this map and the counterclockwise sheets of Section 6.4.1, it also inferred that the atom lands on the far side of the slice. The port of Section 6.9 agrees numerically with both inferences.

The splice drops the first two vertices and the first two edges of each list. Those are the two ends of the slice, the slice edge, and the edge after the slice. Section 6.4.1 labels that last edge `mirrorRedundant`. The edge before the slice in each list stays, and now runs to the vertex after the far end of the slice. Each sheet has a right angle at both ends of its slice (Section 6.4.1), so the code reading inferred that the edge kept and the edge dropped at each end lie on one geodesic and that the welded polygon has no vertex at the end of a slice (Section 17, item 11).

The welded polygon lists the rest of the working polygon from its vertex 2, then the rest of the atom from its vertex 2. No line rotates the final lists into a fixed order (lines 1369-1381), so the first vertex depends on the rotations of step 2. For a kaleidoscope without a slice, such as `*2223` and `*22222`, the loop does not run, and the polygon is the single sheet after the recentring and the shift of steps 2 and 6 of Section 6.5.

### 6.6. Exposed Moduli

The user changes the shape of a hyperbolic orbifold through two kinds of parameters, defined in `lib/orbilib/WallPaperGroup_General.js`:

| Parameter | Range | Applies to | Source |
| --- | --- | --- | --- |
| `{key}_{i}_l`, a length | `LENGTHMINVALUE=0.3` to `LENGTHMAXVALUE=4` | Every key in `lengthKeys` | Lines 26-29 |
| `{key}_{i}_t`, a twist | $-0.5$ to $0.5$ | `handle`, `conePair`, and `tube` | Lines 26-29 |

The initial length is $2 + 0.001(u - 0.5)$ with $u$ uniform in $[0, 1)$, a jitter for general position (line 266, with `randomOffsetRange=.001` at line 52). The slider step is `DEFAULT_INCREMENT=1e-8` (line 20). When the index of a key exceeds `parCounts`, `getLength` returns 1.1 (`lib/orbilib/OrbifoldGeometrization.js:625-627`). The slider takes the range as its minimum and maximum (line 268), and the editing tool clamps a length to it with `newvalue = Math.min(Math.max(newvalue, LENGTHMINVALUE), LENGTHMAXVALUE);` on a drag (`lib/orbilib/OrbifoldUITool.js:457`, in `onParamDrag`) and on a wheel step (line 487, in `onWheel`). The code reading found no other handling of the ends of the range.

The tutorial states that the length and twist sliders are Fenchel-Nielsen coordinates, and that a cut anchored on a mirror freezes its twist. The editing tool `lib/orbilib/OrbifoldUITool.js:61-69` sets `LENGTH_DRAG_SPEED=0.005`, `TWIST_DRAG_SPEED=0.002`, wheel speeds of 0.03 and 0.01, and `HIT_TOLERANCE=7` pixels.

#### 6.6.1. Parameters of a Kaleidoscope

A kaleidoscope `*p1...pn` without cone points, handles, or cross-caps has one length parameter for each slice and one for each corner pair, and no twist (Sections 6.2.1 and 6.3.1). Each is named `slice_<i>_l` or `cornerPair_<k>_l` (`lib/orbilib/WallPaperGroup_General.js:266`, `lib/orbilib/OrbifoldGeometrization.js:629`), has the range 0.3 to 4, and starts at $2 + 0.001(u - 0.5)$ (Section 6.6). A slice length is the length of the side that stands for the slice in its two sheets, which the code lays out with a right angle at each end (Section 6.4.1). That this side is the common perpendicular of the two mirror sides that the slice joins is the code reading's inference. A corner pair length is the length of the mirror side between its two right angles (Section 6.4.1). The tutorial's table describes `slice / band` as mirror-anchored seams whose twist the mirrors freeze, and `cornerPair` as the edge "between paired corners" (`tutorials/orbifold-notation.html:328-329`).

| Signature | Parameters |
| --- | --- |
| `*2223` | `cornerPair_1_l` |
| `*22222` | `cornerPair_1_l`, `cornerPair_2_l` |
| `*3333` | `slice_1_l` |
| `*2345` | `slice_1_l` |
| `*23456` | `slice_1_l`, `slice_2_l` |
| `*2233` | `cornerPair_1_l` |
| `*2332` | `slice_1_l` |

No line checks that the sheets exist for given lengths, such as that each argument of $\operatorname{arcosh}$ is at least 1, or that the welded polygon is convex.

### 6.7. Handles and Cross-Caps

A handle is two paired `["handle",i]` edges with a length and a twist. The twist shifts one edge along itself by `shift*length` (lines 1439-1445). A cross-cap is one `["cap",i]` edge with a length and no twist. The length is the full edge length, and the glide is half of it (Section 8.1). Euclidean and spherical groups map these parameters by name (Section 7.1).

### 6.8. Worked Examples of Kaleidoscopes

The code reading traced these examples by hand through Sections 6.2.1 to 6.5.1 and did not run the code. Each row gives the sheets with their cases of Section 6.4.1, the orders of the corners of the welded polygon in the order of its vertex list, which is counterclockwise, and its sides in the order of its edge list:

| Signature | Sheets and cases | Corners, counterclockwise | Sides |
| --- | --- | --- | --- |
| `*2223` | `[c3, c2, P1]`, case B | 3, 2, 2, 2 | `mirror`, `mirror`, `["cornerPair",1]`, `mirror` |
| `*22222` | `[c2, P1, P2]`, case C | 2, 2, 2, 2, 2 | `mirror`, `["cornerPair",1]`, `mirror`, `["cornerPair",2]`, `mirror` |
| `*3333` | `[c3, c3, s1]` twice, case B | 3, 3, 3, 3 | Four `mirror` |
| `*2345` | `[c5, c4, s1]` and `[c3, c2, s1]`, case B | 5, 4, 3, 2 | Four `mirror` |
| `*23456` | `[c6, c5, s1]` and `[c4, c3, s2]`, case B, and `[c2, s2, s1]`, case C | 6, 5, 2, 4, 3 | Five `mirror` |
| `*45322` | `[c3, P1, s1]`, case C, and `[c5, c4, s1]`, case B | 3, 2, 2, 5, 4 | `mirror`, `["cornerPair",1]`, `mirror`, `mirror`, `mirror` |

In the first four rows, the corners read clockwise are a cyclic rotation of the corners of the signature, as the signature requires. The two corners of order 2 at the ends of a corner pair are the right angles of Section 6.4.1.

In the row of `*23456`, the corners read clockwise are 3, 4, 2, 5, 6, which no rotation or reversal makes into 2, 3, 4, 5, 6. Without the swap at lines 563-567, the third sheet would be `[c2, s1, s2]`, and the trace gives the corners 2, 6, 5, 4, 3, which read clockwise are a rotation of 2, 3, 4, 5, 6. In the row of `*45322`, the swap at lines 538-543 makes the first sheet `[c3, P1, s1]` instead of `[P1, c3, s1]`, and the corners read clockwise are 4, 5, 2, 2, 3, which is the signature `*45223`. In both rows the swap reverses the order of the features of one sheet relative to the others, and the welded polygon has the corner orders of the signature in another cyclic order. The port of Section 6.9 gave the same sheets, corners, and sides for every row of the table. For the polygon of `*23456` at the default lengths, it measured the corner angles of 30, 36, 90, 45, and 60 degrees in the order of the vertex list, a convex polygon, the area $-2\pi\chi$ to within $4 \times 10^{-14}$, and products of the reflections in consecutive sides of the orders 6, 5, 2, 4, and 3. A convex polygon with these angles meets the hypotheses of Theorem 6.4.3 of [DAVIS2008] (Section 4.5 of `docs/research/fundamental-domains.md`), which would make the reflections in its sides generate a discrete group with the signature `*25634` and not `*23456`. That the measured polygon meets those hypotheses exactly rests on numeric evidence from the port and is no proof (Section 17, item 12).

The port of Section 6.9 measured the scope of this defect over every signature of the next table, with each order in the range given:

| $n$ | Orders tried | Signatures | Wrong cyclic order |
| --- | --- | --- | --- |
| 4 | 2 to 7 | 1295 | 0 |
| 5, all orders distinct | 2 to 7 | 720 | 720 |
| 5, a repeated order | 2 to 7 | 7056 | 5910 |
| 6 | 2 to 6 | 15625 | 12956 |
| 7 | 2 to 5 | 16384 | 14589 |
| 8 | 2 to 4 | 6561 | 5082 |

For four or fewer corners, the code reading found that the code makes either a single sheet, whose corners keep their cyclic order up to reversal, or, for four corners without a corner pair, two sheets whose features no swap exchanges (Section 6.3.1). The port found no wrong order among the 1295 signatures with four corners that it tried. With five or more corners, the port found the defect in every signature that it tried with five corners of distinct orders, and in most of the signatures that it tried with five to eight corners, within the orders of the table. For five corners without a corner pair, every input `*p1p2p3p4p5` that the port tried came out as `*p1p4p5p2p3`. With repeated orders, `*22323` came out with the corners 3, 2, 2, 2, 3 counterclockwise, which is wrong, while `*33333`, `*22223`, `*23333`, and `*34343` came out right. The code reading inferred that these four come out right because the reordering leaves their corner sequences unchanged up to rotation and reversal.

In the same port, a variant that reorders the three features of a sheet only by a cyclic rotation until the corners come first, and never by exchanging two of them, gave no wrong cyclic order and no failed geometry check over all 47,641 signatures of the table, each at one set of lengths. This is numeric evidence from a port, not a proof.

### 6.9. Numeric Check by a Port

A verification pass outside the repository ported the code of Sections 6.2.1 to 6.5.1 line by line to Python, keeping the source's order of operations, and ran the port in double precision. The port covers the kaleidoscope part of `hashOrbifold`, `atomizeOrbifold`, `edgeLength`, `getLength`, `useMirrorQ`, the sheet branch of `unfoldAtom`, the turtle and Moebius helpers, the slice path of `assembleFundamentalDomain`, and the reflection circles of the sides. It was run on the examples of Section 6.8, on the 47,641 hyperbolic signatures of the table of Section 6.8, and on lengths from 0.3 to 4 with corner orders from 2 to 12. Its results were compared with independent constructions in the hyperboloid model, with Lorentz frame matrices and Newton's method. The SymmHub code itself was not run. These results are numeric evidence, and none is a source or a proof:

- Every hand trace of Sections 6.2.1, 6.3.1, 6.6.1, and 6.8 agrees with the port.
- Every call of `edgeLength` agrees with the independent construction, with the largest differences $2.4 \times 10^{-14}$ in case A, $1.6 \times 10^{-13}$ in case B, $9.5 \times 10^{-13}$ in case C, and $2.2 \times 10^{-12}$ in case D. This includes the two calls of cases B and C that go to $\operatorname{arsinh}$.
- In the examples of Section 6.8, every sheet closes, with the angle $\pi/m$ at the origin and right angles at the ends of each imaginary side. The side that case D does not compute has the length `p.im`, and every sheet is counterclockwise. In the sweep of lengths of this section, the largest error of the closure of a sheet is $7.9 \times 10^{-6}$, for `*222222`.
- `poincareMobiusEdgeToEdge` maps $v_1$ to $w_2$ and $v_2$ to $w_1$ to within $7 \times 10^{-11}$ and preserves orientation. At both ends of every welded slice the angle is $\pi$ to within $6 \times 10^{-10}$ radians, so the ends of a slice are no vertices of the welded polygon.
- `poincareTurtleMove` with the angle 0 goes back towards `last`, and with the angle $\pi$ continues straight on.
- The 47,641 signatures of the table of Section 6.8 were each built with every length equal to 2. Each polygon had $n$ vertices, every angle below $\pi - 10^{-9}$, the area $-2\pi\chi$ to within $10^{-9}$, and the corner orders of the signature as a multiset, and none held a `mirrorRedundant` edge.
- Over corner orders from 2 to 12 and lengths from 0.3 to 4, the smallest arguments of $\operatorname{arcosh}$ are 1.207 for `R` of case B, 1.0013 for `P` of case C, and 1.0027 in case D, and both $\operatorname{arsinh}$ results stay positive, so no call of `edgeLength` is undefined there. The only `NaN` comes from rounding in `P` of case C with two lengths of at least 18.93 and the order 2, outside the range.
- The examples of Section 6.8, `*222222`, `*2345678`, and `*23232323` were each run with every length over the grid 0.3, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4 and with 300 random settings in the range, from 300 to 1029 runs for each signature, with no `NaN`. A run passed its geometry check when the polygon had $n$ vertices, every angle below $\pi - 10^{-9}$, and the angles and the area within $10^{-8}$ of their targets. The runs that failed that check were 6 of 381 for `*22222`, 2 of 381 for `*23456`, 57 of 1029 for `*222222`, 1 of 300 for `*2345678`, and 3 of 300 for `*23232323`, and none for the other signatures. The report of the pass states that every polygon of the sweep was convex with $n$ vertices, and it attributes the failed checks to the loss of precision near the boundary of the disc. The error of the angles reaches $7.7 \times 10^{-6}$ radians for `*222222` with the lengths 4, 0.3, and 4, and the modulus of a vertex reaches 0.996 in the sweep.

## 7. Euclidean and Spherical Groups

### 7.1. The Non-Negative Catalogue

The orbifold route builds no Euclidean or spherical domain from a general rule. It maps each such signature to one of the hand-written group functions of `lib/orbilib/WallpaperGroups_NonNegative.js`, and each function returns fixed splanes. All locations in Sections 7.1 to 7.1.9 are in that file unless another file is named.

`WallPaperGroup_General.updateTheGroup` calls `nonNegHashOrbifold(hashed, this.parCounts, this.curvature)` when the curvature of Section 4.4 is 0 or 1, and stores its `standardName` and `nnn` (`lib/orbilib/WallPaperGroup_General.js:252-257`). `updateTheGroupGeometry` then calls `getNonnegativeGroupData(this.standardName, this)` (line 350). The square and hexagonal groups are rigid, the spherical groups have no parameter, and only the rectangular and oblique Euclidean groups read a length or a twist (Section 7.1.3).

Section 4.6 records, from the earlier code readings, that the group makers `Group_WP` and `Group_Spherical` take the 17 wallpaper groups and the 14 spherical types by name, in `lib/grouplib/WallpaperGroups.js` and `lib/grouplib/Group_Spherical.js`. This reading did not read those two files, so it is not known whether they build the domains of this section. Every test and script that calls a function named `iWallpaperGroup` imports it from `lib/grouplib/WallpaperGroups.js`, not from this file (Section 7.1.7).

#### 7.1.1. Dispatch from the Signature

`nonNegHashOrbifold` (lines 29-132) takes the parsed signature of Section 4.2. It sorts the cone orders in descending order, in place, and sorts the corners inside each kaleidoscope in descending order, while the list of kaleidoscopes keeps its order (lines 34-35):

```js
var conepoints=hashed.conepoints.sort(function(a, b){return b - a}); //in descending order
var kaleidoscopes=hashed.kaleidoscopes.map(x=>x.sort(function(a, b){return b - a}));
```

It then tests these conditions in this order and stops at the first that holds. It returns `{"standardName":name,"nnn":nnn}` (line 131), and `nnn` stays 0 unless a branch sets it:

| Lines | Condition | Key | Registered parameter and `nnn` |
| --- | --- | --- | --- |
| 38-41 | At least one handle | `O` | `parCounts["handle"]=1` |
| 42-45 | Two or more cross-caps | `XX` | `parCounts["cap"]=1` |
| 46-50 | A cross-cap and a kaleidoscope | `X*` | `parCounts["cap"]=1` |
| 51-56 | A cross-cap and curvature 0 | `22X` | `parCounts["cap"]=1` |
| 57-67 | A cross-cap and curvature 1 | `NX` with exactly one cone point, otherwise `X` | `nnn` is the cone order for `NX` |
| 69-73 | Two or more kaleidoscopes | `**` | `parCounts["band"]=1` |
| 83-87 | Exactly two cone points | `NN` | `nnn` is the larger cone order |
| 88-91 | The first kaleidoscope has two corners | `*NN` | `nnn` is its larger corner order |
| 92-99 | The first kaleidoscope has no corner | `*` without a cone point, otherwise `N*` | `nnn` is the first cone order for `N*` |
| 100-103 | Three cone points, the middle one of order 2 | `22N` | `nnn` is the largest cone order |
| 104-108 | One cone point, of order 2 | `2*N` | `nnn` is the largest corner order of the first kaleidoscope |
| 109-113 | One kaleidoscope with three corners, the middle one of order 2 | `*22N` | `nnn` is the largest corner order |
| 114-129 | Otherwise | The cone orders, then `*` and the corner orders when a kaleidoscope exists, such as `*632`, `4*2`, or `3*2` | `parCounts["tube"]=1` for `2222`, and `parCounts["slice"]=1` for `22*`, `2*22`, and `*2222` (lines 122-128) |

The tests do not check the curvature, except at line 52. The branches at lines 83 and 88 therefore also catch the Euclidean signatures `22*` and `2*22`, which Section 7.1.7 records as a defect. By a hand trace of the table, each signature reaches this key and function:

| Signatures | Key | Function, from `getNonnegativeGroupData` (lines 823-878) |
| --- | --- | --- |
| `*632`, `632`, `*442`, `442`, `4*2`, `*333`, `333`, `3*3` | The signature | `iGroup_S632(1)` and the like (lines 827-834) |
| `*2222` | `*2222` | `iGroup_S2222(1, getLength(["band",1], group))` (line 835) |
| `2222` | `2222` | `iGroup_2222(1, getLength(["tube",1], group), getTwist(["tube",1], group))` (line 836) |
| `22*` | `NN`, with `nnn` = 2 | `iGroup_NN(2)` (line 855) |
| `2*22` | `*NN`, with `nnn` = 2 | `iGroup_SNN(2)` (line 857) |
| `**` | `**` | `iGroup_SS(1, getLength(["slice",1], group))` (line 839) |
| `*x` | `X*` | `iGroup_SX(1, getLength(["cap",1], group))` (lines 841-844) |
| `22x` | `22X` | `iGroup_22X(1, getLength(["cap",1], group))` (lines 846-847) |
| `xx` | `XX` | `iGroup_XX(1, getLength(["cap",1], group))` (lines 849-850) |
| `o` | `O` | `iGroup_O([1,0], [1+getTwist(["handle",1], group), getLength(["handle",1], group)])` (lines 852-853) |
| `nn` with $n \ge 2$, including `22` | `NN` | `iGroup_NN(n)` (line 855) |
| `*nn` with $n \ge 2$, including `*22` | `*NN` | `iGroup_SNN(n)` (line 857) |
| `*` | `*` | `iGroup_SNN(1)` (line 858) |
| `nx` with $2 \le n \le 100$ | `NX` | `iGroup_NX(n)` (line 860). With $n \ge 101$, `22X` and `iGroup_22X` (Section 7.1.7) |
| `x` | `X` | `iGroup_NX(1)` (line 861) |
| `22n` with $n \ge 2$, including `222` | `22N` | `iGroup_22N(n)` (line 863) |
| `2*n` with $n \ge 2$, including `2*2` | `2*N` | `iGroup_2SN(n)` (line 864) |
| `n*` with $n \ge 2$, including `2*` | `N*` | `iGroup_NS(n)` (line 865) |
| `*22n` with $n \ge 2$, including `*222` | `*22N` | `iGroup_S22N(n)` (line 866) |
| `532`, `*532`, `432`, `*432`, `332`, `*332`, `3*2` | The signature | `iGroup_532()` and the like (lines 867-873) |

Each family except `nx` keeps its key for every $n$. By the threshold of Section 4.4, however, the curvature is 1 only for $n \le 200$ in `nn`, $n \le 100$ in `*nn`, `n*`, `nx`, and `22n`, and $n \le 50$ in `2*n` and `*22n`, by the reader's arithmetic. For a larger $n$ the curvature is 0 (Section 7.1.7). At the bounds, where $\chi$ equals 0.01, the result depends on the rounding of the floating-point sum. The port of Section 7.1.9 tried only `(100)x`, which keeps the curvature 1.

The parser rejects an order of 1 (Section 4.2), so no signature with $n = 1$ reaches the families, and the trivial group `1` reaches no function. Only the keys `*` and `X` call a function with $n = 1$. The branches for $n = 1$ inside `iGroup_NS`, `iGroup_S22N`, `iGroup_22N`, and `iGroup_2SN` (lines 659-660, 671-672, 684-685, and 698-699) are therefore unreachable from the dispatch, by a hand trace.

Any key that no `case` names gives `iGroup_Trivial()`, which is `{s:[], t:[]}` (lines 195-197 and 826). The labels `'*X'`, `'*x'`, `'x*'`, `'22x'`, `'xx'`, and `'o'` at lines 841-853 are aliases that `nonNegHashOrbifold` never produces, because it writes only `X` and `O`.

#### 7.1.2. Representation of a Catalogue Group

Each group function returns a plain object `{s: [...], t: [...]}`. `s` lists the sides of $F$, one splane each, and `t[i]` lists the splanes whose product is the pairing of side `s[i]`, as in lines 209-212:

```js
return {
        s:[s0,s1,s2],  //fund domain
        t:[[s0],[s1],[s2]] // transforms
    };
```

`updateTheGroupGeometry` wraps each splane of `s` in a list, with `x.bounds = [NaN, NaN]`, and keeps `t` (`lib/orbilib/WallPaperGroup_General.js:349-368`). `Group_Orbifold.getGroup` unwraps the sides and builds `new Group({ s: flatS, t: fd.t })` (`lib/orbilib/Group_Orbifold.js:167-172`). `Group` stores `t` as the reverse transforms and their inverses as the forward transforms (`lib/invlib/Group.js:41-44`). These conventions hold for every catalogue group:

- Splanes. `iPlane([nx, ny, 0, d])` normalizes the normal $(n_x, n_y)$ to $\mathbf n$ and keeps $d$ as given (`lib/invlib/ISplane.js:86-101`), so the side is the line $\langle x, \mathbf n \rangle = d$. Its domain side is $\langle x, \mathbf n \rangle < d$ (`lib/invlib/Inversive.js:103-107`). `iSphere([cx, cy, 0, r])` with $r > 0$ is the circle with centre $(c_x, c_y)$ and radius $r$, and its domain side is the inside (`lib/invlib/Inversive.js:94-98`). Every `iSphere` in the file has a positive radius.
- Products. A list `[a, b, c]` reflects first in `a`, then in `b`, then in `c`, so as a map it is $R_c \circ R_b \circ R_a$, where $R_s$ is the reflection in $s$ (`lib/invlib/Inversive.js:249-251` and `lib/invlib/ITransform.js:113-124`). The inverse reverses the list (`lib/invlib/GroupUtils.js:164-169`).
- Fold. When a point is outside side `s[i]`, the fold applies `t[i]` to it (`lib/invlib/Group.js:211-218`, and `lib/shaders/inversiveSampler.glsl.mjs:207-224` on the graphics card, Section 9). So `t[i]` maps the cell on the far side of `s[i]` into $F$, as Section 2.4, item 4, states.
- Borders. The tile borders of Section 11.1 measure the distance to every splane of `s`, over its whole line or circle, and the renderer packs the same `s` for the fold and the borders (`lib/symhublib/VisualizationImage.js:180`, `lib/symhublib/VisualizationColorSym.js:363`). A mirror that serves only inside a product, such as `sy` in `iGroup_632`, draws no border.
- Euclidean model. The plane itself. `getNonnegativeGroupData` passes the scale `a = 1` to every Euclidean function (lines 827-853).
- Spherical model. The stereographic plane, "treated in the plane, scaled with the equator at the unit circle" (line 628). The poles are the origin and the point at infinity. The code does not say which pole is the origin. Under the inverse stereographic map of Section 11.2 of `docs/research/geometry-models.md`, which that section cites to its source, the origin goes to $(0, 0, -1)$ and the unit circle to the equator. This section uses that map for the sphere coordinates of its examples, as a reader's evaluation.

In Sections 7.1.4 to 7.1.6, a side is written as its line or circle with the domain side as an inequality, and a pairing as the isometry that the hand trace of its list gives. The kind of each product, such as a rotation or a glide reflection, is the reader's name for the map that the trace gives, and no comment in the code states it.

Side $i$ is entry $i$ of the list `s` that the function returns. A name such as `s0`, `s1`, or `sy` is the local variable of the function that holds a splane, and its number need not match a side number. In `iGroup_N`, `s:[s1,s2]` makes the variable `s1` side 0 and `s2` side 1, while `s0` is no side and serves only inside the products. The tables write variables in code font and sides as numbers.

#### 7.1.3. Parameters of the Catalogue

`updateTheGroup` first sets the count of every key of `keys` to 0 (`lib/orbilib/WallPaperGroup_General.js:204-209`), and `keys` holds `band` and `slice` (`lib/orbilib/OrbifoldGeometrization.js:46-47`). `nonNegHashOrbifold` then raises the count of one key to 1 (Section 7.1.1). For each registered key, `updateTheGroup` makes a slider `key_1_l` with the range 0.3 to 4 and the initial value $2 \pm 0.0005$, and for `handle` and `tube` also a slider `key_1_t` with the range $-0.5$ to $0.5$ and the initial value 0 (`lib/orbilib/WallPaperGroup_General.js:263-284`, Section 15.1). `getLength` returns 1.1 when the count of its key is below the index, and otherwise the slider value (`lib/orbilib/OrbifoldGeometrization.js:624-630`):

```js
export function getLength(key,group) {
    if(
  group.parCounts[key[0]]<
  abs(key[1])){return 1.1}
```

`getTwist` returns 0 in the same case (lines 632-637). The parameters of each group are these:

| Signature | Registered | Read | Value used | What it moves |
| --- | --- | --- | --- | --- |
| `o` | `handle` | `handle_1_l` and `handle_1_t` | The sliders | The length $\ell$ and the twist $t$ of the second vector $(1 + t, \ell)$ of the lattice basis |
| `2222` | `tube` | `tube_1_l` and `tube_1_t` | The sliders | The height $b$ of the rectangle and the offset $c$ of its half-turn centres |
| `*2222` | `slice` | `band` | 1.1, always | The height of the rectangle |
| `**` | `band` | `slice` | 1.1, always | The height of the rectangle |
| `*x`, `22x`, `xx` | `cap` | `cap_1_l` | The slider | The height of the rectangle |
| `22*`, `2*22` | Nothing, because the dispatch never reaches lines 122-128 for them | Nothing, because the dispatch never reaches lines 837-838 | None | Nothing (Section 7.1.7) |
| The square and hexagonal groups | Nothing | Nothing | `a = 1` | Nothing |
| Every spherical group | Nothing | Nothing | None | Nothing |

The twist of `o` shifts the second basis vector along the first, whose length is 1, and the twist of `2222` is a raw offset of the half-turn centres. Neither goes through the code of the hyperbolic handles that Section 2.4, item 6, describes.

#### 7.1.4. Euclidean Group Functions

This table gives each Euclidean function with the domain that a hand trace of its splanes gives, at `a = 1`. The sides are listed in the order of `s`, as side 0, side 1, and so on, and each side carries the pairing of its list in `t`. The letters $b$, $c$, $t$, and $\ell$ are the parameters of Section 7.1.3.

| Signature, function, lines | Domain | Sides in the order of `s`, with their pairings |
| --- | --- | --- |
| `*442`, `iGroup_S442`, 202-213 | Triangle $(0, 0)$, $(1/2, 0)$, $(0, 1/2)$, with $d = \sqrt2/4$ at line 204 | 0: $x > 0$, 1: $y > 0$, 2: $x + y < 1/2$. Each pairing is the reflection in its side |
| `442`, `iGroup_442`, 218-233 | Triangle $(0, 1/2)$, $(0, -1/2)$, $(1/2, 0)$ | 0: $x > 0$, by `[s0,s3]`, the half-turn about $(0, 0)$, onto itself. 1: $x + y < 1/2$, by `[s1,s3]`, the rotation through $\pi/2$ about $(1/2, 0)$, onto side 2. 2: $x - y < 1/2$, by `[s2,s3]`, the inverse rotation, onto side 1. `s3` is the line $y = 0$ |
| `4*2`, `iGroup_4S2`, 239-253 | The triangle of `442` | 0: $x > 0$, reflection. 1: by `[sy, s2]`, the rotation through $\pi/2$ about $(1/2, 0)$, onto side 2. 2: by `[s2,sy]`, onto side 1. `sy` is the line $y = 0$ |
| `*632`, `iGroup_S632`, 259-274 | Triangle $(0, 0)$, $(1/2, 0)$, $(0, \sqrt3/2)$, with $d = \sqrt3/4$ at line 263 | 0: $x > 0$, 1: $y > 0$, 2: $\sqrt3 x + y < \sqrt3/2$. Reflections |
| `632`, `iGroup_632`, 279-295 | Triangle $(0, \sqrt3/2)$, $(0, -\sqrt3/2)$, $(1/2, 0)$ | 0: $x > 0$, by `[s0,sy]`, the half-turn about $(0, 0)$, onto itself. 1: $\sqrt3 x + y < \sqrt3/2$, by `[s1,sy]`, the rotation through $2\pi/3$ about $(1/2, 0)$, onto side 2. 2: $\sqrt3 x - y < \sqrt3/2$, by `[s2,sy]`, onto side 1 |
| `3*3`, `iGroup_3S3`, 300-316 | The triangle of `632` | 0: $x > 0$, reflection. 1 and 2: as in `632`, by `[s1,sy]` and `[s2,sy]` |
| `*333`, `iGroup_S333`, 321-335 | Equilateral triangle $(-1/2, 0)$, $(1/2, 0)$, $(0, \sqrt3/2)$ | 0: $-\sqrt3 x + y < \sqrt3/2$, 1: $\sqrt3 x + y < \sqrt3/2$, 2: $y > 0$. Reflections |
| `333`, `iGroup_333`, 340-358 | Rhombus $(1/2, 0)$, $(0, \sqrt3/2)$, $(-1/2, 0)$, $(0, -\sqrt3/2)$ | 0: $\sqrt3 x + y < \sqrt3/2$, by `[s0,sy]`, the rotation through $2\pi/3$ about $(1/2, 0)$, onto side 2. 1: $-\sqrt3 x + y < \sqrt3/2$, by `[s1,sy]`, the rotation about $(-1/2, 0)$, onto side 3. 2: $\sqrt3 x - y < \sqrt3/2$, onto side 0. 3: $-\sqrt3 x - y < \sqrt3/2$, onto side 1. Counterclockwise, the sides run 0, 1, 3, 2 |
| `*2222`, `iGroup_S2222`, 363-378 | Rectangle $[0, 1/2] \times [0, b/2]$ | 0: $x < 1/2$, 1: $x > 0$, 2: $y < b/2$, 3: $y > 0$. Reflections |
| `2222`, `iGroup_2222`, 404-421 | Rectangle $[0, 1] \times [-b/2, b/2]$. Side 0 uses `a`, not `0.5*a` | 0: $x < 1$, by `[s0,ss0]`, the half-turn about $(1, c)$. 1: $x > 0$, by `[s1,ss1]`, the half-turn about $(0, -c)$. 2: $y < b/2$, by `[s2,ss]`, the translation by $(0, -b)$, onto side 3. 3: $y > -b/2$, by `[s3,ss]`, the translation by $(0, b)$, onto side 2. `ss0` and `ss1` are the lines $y = c$ and $y = -c$ |
| `2*22`, `iGroup_2S22`, 426-443, never reached | Rectangle $[0, 1/2] \times [-b/2, b/2]$ | 0: $x < 1/2$, reflection. 1: $x > 0$, by `[s1,sy]`, the half-turn about $(0, 0)$, onto itself. 2: $y < b/2$ and 3: $y > -b/2$, reflections |
| `22*`, `iGroup_22S`, 448-465, never reached | The rectangle of `2*22` | 0: $x < 1/2$, by `[s0,sy]`, the half-turn about $(1/2, 0)$, onto itself. 1: $x > 0$, the half-turn about $(0, 0)$, onto itself. 2 and 3: reflections |
| `**`, `iGroup_SS`, 470-487 | The rectangle of `2*22` | 0: $x < 1/2$ and 1: $x > 0$, reflections. 2: by `[s2,sy]`, the translation by $(0, -b)$, onto side 3. 3: the translation by $(0, b)$, onto side 2 |
| `*x`, `iGroup_SX`, 492-509 | Rectangle $[-1/2, 1/2] \times [-b/2, b/2]$ | 0: $x < 1/2$, by `[s0,sx,sy]`, the glide reflection $(x, y) \mapsto (x - 1, -y)$, onto side 1. 1: $x > -1/2$, by `[s1,sx,sy]`, $(x, y) \mapsto (x + 1, -y)$, onto side 0. 2: $y < b/2$ and 3: $y > -b/2$, reflections. `sx` and `sy` are the lines $x = 0$ and $y = 0$ |
| `22x`, `iGroup_22X`, 514-533 | The rectangle of `*x` | 0 and 1: the glide reflections of `*x`. 2: by `[s2,sy,sx]`, the glide reflection $(x, y) \mapsto (-x, y - b)$, onto side 3. 3: $(x, y) \mapsto (-x, y + b)$, onto side 2 |
| `xx`, `iGroup_XX`, 538-555 | The rectangle of `*x` | 0 and 1: the glide reflections of `*x`. 2: by `[s2,sy]`, the translation by $(0, -b)$, onto side 3. 3: the translation by $(0, b)$, onto side 2 |
| `o`, `iGroup_O`, 560-591 | Parallelogram with the vertices $\pm A/2 \pm B/2$, for $A = (1, 0)$ and $B = (1 + t, \ell)$ | 0: the line through $A/2$ parallel to $B$, by `[sa1,sa0]`, the translation by $-A$, onto side 1. 1: the line through $-A/2$, the translation by $A$, onto side 0. 2: the line through $B/2$ parallel to $A$, by `[sb1,sb0]`, the translation by $-B$, onto side 3. 3: the translation by $B$, onto side 2. `sa0` and `sa1` are the lines perpendicular to $A$ through $0$ and $A/2$, and `sb0` and `sb1` those perpendicular to $B$ |

`iGroup_O` builds the normals of sides 0 and 2 as the unit vectors perpendicular to $B$ and to $A$, the "dual basis" of line 567, and the offset of each side as the dot product of $A/2$ or $B/2$ with that normal (lines 568-577). In `iGroup_333`, line 349 reuses the name `s3` of $\sqrt3$ for the fourth plane, and the right-hand side still reads the number, because `var` assigns after it evaluates the expression.

#### 7.1.5. Spherical Group Functions

This table gives each spherical function in the stereographic plane of Section 7.1.2, with the domain that a hand trace gives. `s0` is the line $y = 0$, with the domain side $y > 0$, and `ss` is the unit circle, which is the equator under the map of Section 7.1.2, with the inside as the domain side. These statements are reader's inferences (Section 17, item 15). A line through the origin is a great circle through both poles. A wedge from the origin to infinity is a lune. A wedge inside the unit circle is a triangle with a vertex at the pole and two on the equator.

| Signatures, function, lines | Domain | Sides in the order of `s`, with their pairings |
| --- | --- | --- |
| `*nn`, `iGroup_SN(n)` through `iGroup_SNN`, 596-605 and 631-637 | The lune $0 < \arg z < \pi/n$ | 0: `s0`, 1: the line at the angle $\pi/n$, with the domain below it. Reflections |
| `*`, `iGroup_SNN(1)`, 633-636 | The half-plane $y > 0$, a hemisphere | 0: `s0`, reflection |
| `nn`, `iGroup_N(n)` through `iGroup_NN`, 612-622 and 640-642 | The lune $-\pi/n < \arg z < \pi/n$ | 0: the line at the angle $\pi/n$, by `[s1,s0]`, the rotation through $2\pi/n$ about the origin, onto side 1. 1: the line at the angle $-\pi/n$, by `[s2,s0]`, onto side 0 |
| `nx`, `x`, `iGroup_NX(n)`, 644-654 | The lune $-\pi/(2n) < \arg z < \pi/(2n)$ | 0: the line at the angle $\pi/(2n)$, by `[s1,s0,ss]`, the rotation through $\pi/n$ about the origin onto the line at $-\pi/(2n)$ followed by the reflection in the unit circle, onto side 1. 1: the line at $-\pi/(2n)$, by `[s2,s0,ss]`, onto side 0 |
| `n*`, `iGroup_NS(n)`, 656-668 | The triangle $-\pi/n < \arg z < \pi/n$, $\lvert z \rvert < 1$ | 0 and 1: the lines at $\pm\pi/n$, by `[s1,s0]` and `[s2,s0]`, the rotations of `nn`. 2: `ss`, reflection |
| `*22n`, `iGroup_S22N(n)`, 670-680 | The triangle $0 < \arg z < \pi/n$, $\lvert z \rvert < 1$ | 0: `s0`, 1: the line at $\pi/n$, 2: `ss`. Reflections |
| `22n`, `iGroup_22N(n)`, 683-694 | The triangle of `n*` | 0 and 1: the rotations of `nn`. 2: `ss`, by `[ss,s0]`, the reflection in the unit circle followed by the reflection in $y = 0$, which maps the arc onto itself and fixes $(1, 0)$ |
| `2*n`, `iGroup_2SN(n)`, 697-708 | The triangle of `*22n` | 0: `s0` and 1: the line at $\pi/n$, reflections. 2: `ss`, by `[ss,sh]`, the reflection in the unit circle followed by the reflection in the line `sh` at the angle $\pi/(2n)$, which maps the arc onto itself and fixes $e^{i\pi/(2n)}$ |
| `*532`, `iGroup_S532`, 711-718 | The triangle bounded by `s0`, the line at $\pi/5$, and the circle with centre $(-1.61803398874989, 0)$ and radius $1.90211303259031$ (line 714) | Reflections |
| `532`, `iGroup_532`, 720-728 | The region $-\pi/5 < \arg z < \pi/5$ inside the circle of `*532` | 0 and 1: the lines at $\pm\pi/5$, by `[s1,s0]` and `[s2,s0]`, the rotations through $2\pi/5$. 2: the circle, by `[s5,s0]`, which maps the arc onto itself |
| `*432`, `iGroup_S432`, 730-737 | As `*532`, with the angle $\pi/4$ and the circle with centre $(-1, 0)$ and radius $1.41421356237310$ (line 733) | Reflections |
| `432`, `iGroup_432`, 739-747 | As `532`, with the angle $\pi/4$ and the circle of `*432` | As `532` |
| `*332`, `iGroup_S332`, 749-756 | As `*532`, with the angle $\pi/3$ and the circle with centre $(-0.707106781186548, 0)$ and radius $1.22474487139159$ (line 752) | Reflections |
| `332`, `iGroup_332`, 758-766 | As `532`, with the angle $\pi/3$ and the circle of `*332` | As `532` |
| `3*2`, `iGroup_3S2`, 768-776 | The region $-\pi/3 < \arg z < \pi/3$ inside the circle with centre $(-1.41421356237310, 0)$ and radius $1.73205080756888$ (line 772) | 0 and 1: the lines at $\pm\pi/3$, by `[s1,s0]` and `[s2,s0]`, the rotations through $2\pi/3$. 2: the circle, by `[s3]`, reflection |

For $n = 2$, `iGroup_N`, `iGroup_NX` with $n = 1$, `iGroup_NS`, and `iGroup_22N` compute the line at $\pi/2$ and the line at $-\pi/2$ from `sin` and `cos`, which give the line $x = 0$ twice, up to the rounding of `cos(PI/2)`. Their lune or triangle then has two sides on one great circle, which meet at the origin at the angle $\pi$. The four circles of the polyhedral groups each satisfy $r^2 = \lvert c \rvert^2 + 1$ to the printed digits, by the reader's arithmetic, so each meets the unit circle at two opposite points. That such a circle is the image of a great circle is a reader's inference (Section 17, item 15). The code states no derivation of the constants.

#### 7.1.6. Worked Examples of the Catalogue

The code reading traced these examples by hand through Sections 7.1.1 to 7.1.5 and did not run the code. The port of Section 7.1.9 gave the same values for every row and for the closing paragraph. A length of 2 stands for the initial value $2 \pm 0.0005$ of Section 7.1.3. Each value is rounded to six decimals, and the sphere points use the map of Section 7.1.2.

| Signature | What the catalogue builds |
| --- | --- |
| `o`, with $t = 0$ and $\ell = 2$ | $A = (1, 0)$ and $B = (1, 2)$. The vertices are $(1, 1)$, $(0, 1)$, $(-1, -1)$, and $(0, -1)$. Side 0 is the line $2x - y = 1$, stored with the unit normal $(0.894427, -0.447214)$ and the offset $0.447214$, and side 1 is $2x - y = -1$. Sides 2 and 3 are $y = 1$ and $y = -1$. The translation of side 2 is by $(-1, -2)$, and it maps $(0, 1)$ to $(-1, -1)$. With $t = 0.25$, $B = (1.25, 2)$ |
| `**`, as coded | The rectangle $[0, 0.5] \times [-0.55, 0.55]$, from the fallback 1.1, with the translation by $(0, -1.1)$ on side 2 |
| `xx`, with $b = 2$ | The rectangle $[-0.5, 0.5] \times [-1, 1]$. Side 0 maps $(0.5, 0.3)$ to $(-0.5, -0.3)$, and side 2 maps $(0.3, 1)$ to $(0.3, -1)$. All four corners form one vertex cycle |
| `*x`, with $b = 2$ | The rectangle of `xx`, with the glide reflections of `xx` on sides 0 and 1 and the mirrors $y = \pm1$. The glide reflection maps the corner $(0.5, 1)$ to $(-0.5, -1)$ |
| `22*`, as coded | `iGroup_N(2)`: the half-plane $x > 0$ with two sides on the line $x = 0$, each paired by the half-turn $(x, y) \mapsto (-x, -y)$. This is a group of order 2, not a wallpaper group |
| `22*`, as `iGroup_22S` would build it with $b = 2$ | The rectangle $[0, 0.5] \times [-1, 1]$, with half-turns about $(0.5, 0)$ and $(0, 0)$ on the vertical sides and mirrors on $y = \pm1$. The half-turn about $(0.5, 0)$ maps $(0.5, 0.3)$ to $(0.5, -0.3)$. The corners $(0, \pm1)$ form one vertex cycle with the angle sum $\pi$, and so do $(0.5, \pm1)$ |
| `22x`, with $b = 2$ | The rectangle of `xx`. Side 2 maps $(0.3, 1)$ to $(-0.3, -1)$. The vertex cycles are $\{(0.5, 1), (-0.5, -1)\}$ and $\{(0.5, -1), (-0.5, 1)\}$, each with the angle sum $\pi$ |
| `2*22`, as coded | `iGroup_SN(2)`: the quadrant $x > 0$, $y > 0$ with two mirrors. This is a group of order 4, not a wallpaper group |
| `2*22`, as `iGroup_2S22` would build it with $b = 2$ | The rectangle $[0, 0.5] \times [-1, 1]$, with the half-turn about $(0, 0)$ on side 1 and mirrors on the other three sides. The corners $(0.5, \pm1)$ have the angle $\pi/2$ between two mirrors, and $(0, \pm1)$ form one vertex cycle with the angle sum $\pi$ |
| `4*2` | The triangle $(0, 0.5)$, $(0, -0.5)$, $(0.5, 0)$, with the angles $\pi/4$, $\pi/4$, and $\pi/2$. The rotation of side 1 maps $(0, 0.5)$ to $(0, -0.5)$, so the two vertices on the mirror form one cycle with the angle sum $\pi/2$ |
| `3*3` | The triangle $(0, 0.866025)$, $(0, -0.866025)$, $(0.5, 0)$, with the angles $\pi/6$, $\pi/6$, and $2\pi/3$. The two vertices on the mirror form one cycle with the angle sum $\pi/3$ |
| `2222`, with $b = 2$ and $c = 0.25$ | The rectangle $[0, 1] \times [-1, 1]$. The half-turn of side 0 is about $(1, 0.25)$ and maps side 0 onto the segment from $(1, -0.5)$ to $(1, 1.5)$, and the half-turn of side 1 is about $(0, -0.25)$ |
| `3*2` | Vertices $(0, 0)$ and $(0.258819, \pm0.448288)$, on the sphere $(0, 0, -1)$ and $(0.408248, \pm0.707107, -0.577350)$. The circle meets $y = 0$ at $(0.317837, 0)$, inside side 2. The angles are $2\pi/3$ at the origin and $\pi/4$ at each other vertex |
| `2*3` | Vertices $(0, 0)$, $(1, 0)$, and $(0.5, 0.866025)$, on the sphere $(0, 0, -1)$, $(1, 0, 0)$, and $(0.5, 0.866025, 0)$, with the angles $\pi/3$, $\pi/2$, and $\pi/2$. `sh` is the line at $\pi/6$, and the pairing of side 2 fixes $(0.866025, 0.5)$, on the sphere $(0.866025, 0.5, 0)$ |
| `3*` | Vertices $(0, 0)$ and $(0.5, \pm0.866025)$, on the sphere $(0, 0, -1)$ and $(0.5, \pm0.866025, 0)$, with the angles $2\pi/3$, $\pi/2$, and $\pi/2$. Side 0 goes to side 1 by the rotation through $2\pi/3$, and side 2 is a mirror |
| `3x` | The lune $-\pi/6 < \arg z < \pi/6$ between the poles $(0, 0, -1)$ and $(0, 0, 1)$. Side 0 maps the point $0.5 e^{i\pi/6} = (0.433013, 0.25)$ to $2 e^{-i\pi/6} = (1.732051, -1)$ |
| `1` | No domain, because the parser rejects the order 1 (Section 4.2) |
| `*` | The half-plane $y > 0$ with one mirror, the great circle $y = 0$ |
| `x` | The half-plane $x > 0$, with both sides on $x = 0$ and the pairing $(x, y) \mapsto -(x, y)/(x^2 + y^2)$. It maps $(0, 0.5)$ to $(0, -2)$, on the sphere $(0, 0.8, -0.6)$ to $(0, -0.8, 0.6)$ |
| `22` | The half-plane $x > 0$, with both sides on $x = 0$ and the half-turn $(x, y) \mapsto (-x, -y)$ |
| `2*` | The half-disc $x > 0$, $\lvert z \rvert < 1$, with vertices $(0, \pm1)$, on the sphere $(0, \pm1, 0)$. Sides 0 and 1 lie on $x = 0$ and are paired by the half-turn about the origin, and side 2, the right half of the unit circle, is a mirror |

For the groups that Escher already builds, the trace gives the shapes of Escher's own domains. `*632`, `*442`, and `*333` are triangles with the angles of their signatures, and `*2222` is a rectangle. `632`, `442`, and `333` are the doubles of the reflection triangles across one side, the line $y = 0$, with that side removed and each other side paired by a product of two reflections. `*332`, `*432`, and `*532` are the triangles with the angles $\pi/p$ at the origin, $\pi/2$ where the circle meets $y = 0$, and $\pi/3$ at the third vertex, and `332`, `432`, and `532` are their doubles across $y = 0$. `*nn` is the lune with the angle $\pi/n$, `nn` the lune with the angle $2\pi/n$, and `*22n` the triangle with the angle $\pi/n$ at the pole and two right angles on the equator. For `*532`, the vertex on $y = 0$ is $(0.284079, 0)$, on the sphere $(0.525731, 0, -0.850651)$, and the third vertex is $0.338261\,e^{i\pi/5}$, on the sphere $(0.491123, 0.356822, -0.794654)$. For `*432` they are $(0.414214, 0)$ and $0.517638\,e^{i\pi/4}$, and for `*332` they are $(0.517638, 0)$ and $0.707107\,e^{i\pi/3}$.

#### 7.1.7. Defects of the Catalogue

The reading found these defects. Section 15.3 lists most of them with their lines.

- `22*` and `2*22` never reach their functions. `22*` has two cone points and is caught at line 83 as `NN` with `nnn` = 2, and `2*22` has a kaleidoscope with two corners and is caught at line 88 as `*NN` with `nnn` = 2. They get `iGroup_N(2)` and `iGroup_SN(2)`, the finite groups of order 2 and 4 of Section 7.1.6, in the Euclidean plane. Their `case` lines at 125-126 and 837-838 are unreachable. The reader takes this to be the cause of the two entries of `apps/orbifold/bug list.txt:3-4`, "2*22 does not get the slice parameter correctly. Perhaps it is instead a corner pair." and "22* as well.", although the note names the parameter and not the group (Section 17, item 15).
- `*2222` registers `slice` at line 127 and reads `band` at line 835, and `**` registers `band` at line 71 and reads `slice` at line 839. In both, the count of the key read is 0, so `getLength` returns 1.1, and the slider that the user sees changes nothing (Section 7.1.3).
- In `iGroup_2222` with $c \ne 0$, the half-turn of side 0 maps the side $x = 1$ onto the segment from $(1, 2c - b/2)$ to $(1, 2c + b/2)$, which is not a side of the rectangle, and the half-turn of side 1 does the same on $x = 0$. Condition 3.2 (b) of the polygon theorem of Section 4.9 of `docs/research/fundamental-domains.md` asks a pairing to map a face onto a face. By the reader's application of that condition to the traced maps, the four sides with these pairings fail it at $c \ne 0$, and the corners $(1, \pm b/2)$ and $(0, \pm b/2)$ form no vertex cycles (Section 17, item 15). The port confirmed the image of side 0. Section 7.1.9 gives its result on the rectangle as a fundamental domain.
- The curvature test of Section 4.4 gives the curvature 0 to a spherical family with a large $n$, because its $\chi$ falls below 0.01 (Section 7.1.1). For `nx` with $n \ge 101$, the branch at lines 51-56 then gives the Euclidean key `22X`, so `(101)x`, a spherical group of order 202, gets `iGroup_22X(1, 2)`, the rectangle of the Euclidean `22x` with the initial height 2. `(100)x` still gets `NX`. The other families keep their keys and functions with the curvature 0, as the port of Section 7.1.9 found for the signatures that it lists. What the curvature 0 changes downstream for these families, such as the mode of the navigator, was not traced.
- `iGroup_2222_` (lines 381-402), whose comment at line 381 reads "group 2222 (wrong, missing one parameter)", has no shear and is never called. `getNonnegativeGroupData` calls `iGroup_2222` (lines 404-421) instead, which has the shear $c$.
- `iWallpaperGroup` (lines 780-819) calls `getParam`, which the file does not import (lines 1-13), and its debug line 791 names an undefined `index`. Nothing in the repository calls this function. Every call of a function of that name imports it from `lib/grouplib/WallpaperGroups.js`. `iGroup_2222_` and the arrays `WallpaperGroups` and `WallpaperGroupNames` with `getWallpaperGroupIndex` (lines 137-190) are also unused, and `WallpaperGroupNames` lists `"333"` twice (lines 147 and 149).
- The comment at `lib/orbilib/WallPaperGroup_General.js:356` refers to a function `getEuclideanGroupData`, which does not exist.

#### 7.1.8. What Was Read and How It Was Checked

The reading covered `lib/orbilib/WallpaperGroups_NonNegative.js` in full, lines 1-880, `lib/orbilib/WallPaperGroup_General.js:200-371`, `lib/orbilib/OrbifoldGeometrization.js:44-55`, 242-251, and 620-638, `lib/orbilib/Group_Orbifold.js:167-172`, and the splane, product, and fold functions of `lib/invlib/` and `lib/shaders/` that Section 7.1.2 cites. Two further readings of the dispatch and of the representation, made independently of the first, agreed with it on every line that this section cites. Every domain, pairing, vertex cycle, and number of Sections 7.1.4 to 7.1.6 was first a hand trace, and the port of Section 7.1.9 then checked them (Section 17, item 14).

#### 7.1.9. Numeric Check by a Port

A verification pass outside the repository ported to Python, line by line, `hashOrbifoldString`, `nonNegHashOrbifold`, `getLength` and `getTwist` with every key set to 0 first as `updateTheGroup` does, every `iGroup_*` function, and `getNonnegativeGroupData`. The port keeps the representation of the code: a plane as a unit normal and an offset, a sphere as a centre and a signed radius, the domain where the side test is negative, products applied from first to last, and a fold that applies `t[i]` on the first side that a point lies outside. It ran in double precision. For each Euclidean group, it folded 4000 random points of the square $[-4, 4]^2$, tested 1500 interior points for a second image inside $F$, and identified the lattice and the point group of the group generated. For each spherical group, it closed the group as matrices of $O(3)$ and compared the order, the number of reflections, and the orders of the rotations with the expected ones, folded 3000 uniform points of the sphere, and tested 1000 points for a second image. The SymmHub code itself was not run. These results are numeric evidence, and none is a source or a proof:

- The dispatch of all 17 Euclidean signatures and of 44 spherical signatures agrees with Section 7.1.1. The 44 are the 35 signatures of the seven families with $n$ from 2 to 6, which include `22`, `2*`, `*22`, and `222` at $n = 2$, the seven polyhedral groups, and `*` and `x`. The agreement covers `22*` as `NN` and `2*22` as `*NN`, both with `nnn` = 2, and the height 1.1 of `*2222` and `**`.
- Every row of the tables of Sections 7.1.4 and 7.1.5 agrees with the port. For the Euclidean functions this holds at the parameter values of the table below, and for the spherical functions at the signatures of the first item. The agreement covers the vertices, the order of the sides, the kind, centre, angle, or vector of each pairing, the side it maps onto, and the inverse pairs. The `var s3` of line 349 is harmless.
- Every worked example of Section 7.1.6 agrees. The pairing of `x`, as a matrix of $O(3)$, is exactly minus the identity.
- For the four circles of the polyhedral groups, $r^2 - \lvert c \rvert^2 - 1$ is $2.6 \times 10^{-14}$, $1.4 \times 10^{-14}$, $1.8 \times 10^{-15}$, and $-4.4 \times 10^{-15}$, so the relation of Section 7.1.5 holds to within $3 \times 10^{-14}$.
- The verdicts on the Euclidean signatures, at the settings that the table gives, are these:

  | Signature | Function reached | Verdict |
  | --- | --- | --- |
  | `*632`, `632`, `*442`, `442`, `4*2`, `*333`, `333`, `3*3` | Its own | A fundamental domain of the group of the signature. The vertex cycles of `632` have the angle sums $\pi/3$ and $2\pi/3$, those of `442` $\pi/2$ and $\pi/2$, the mirror cycle of `4*2` $\pi/2$, that of `3*3` $\pi/3$, and the three cycles of `333` $2\pi/3$ each |
  | `*2222` | `iGroup_S2222(1, 1.1)` | The right group, with the height fixed at 1.1 |
  | `2222` | `iGroup_2222(1, b, c)` | The right group. With $(b, c)$ = $(2, 0.25)$, $(2, 0.5)$, $(0.3, 0.5)$, and $(2, -0.4)$, the fold ended for all 4000 points within 7, 8, 31, and 8 steps, no interior point of $F$ had a second image inside $F$, and the group was `2222` with the lattice generated by $(2, 4c)$ and $(0, b)$, whose covolume $2b$ is twice the area of $F$. $F$ is a fundamental domain of a brick tiling, while the face condition still fails (Section 7.1.7). The image of side 0 at $(2, 0.25)$ is the segment from $(1, -0.5)$ to $(1, 1.5)$ |
  | `**` | `iGroup_SS(1, 1.1)` | The right group, with the height fixed at 1.1 |
  | `*x`, `22x`, `xx` | Its own | The right group, with $b$ = 2 and 0.7 |
  | `o` | `iGroup_O` | The right group, with $(t, \ell)$ = $(0, 2)$, $(0.25, 2)$, $(-0.5, 0.3)$, and $(0.5, 4)$ |
  | `22*` | `iGroup_N(2)` | Wrong: a finite group of order 2, a half-plane with a half-turn |
  | `2*22` | `iGroup_SN(2)` | Wrong: a finite group of order 4, a quadrant with two mirrors |
  | `22*` through `iGroup_22S`, never reached | None | Would give `22*`, with $b$ = 2 and 0.7 |
  | `2*22` through `iGroup_2S22`, never reached | None | Would give `2*22`, with $b$ = 2 and 0.7 |
  | `iGroup_2222_`, never called | None | A fundamental domain of `2222` with the area $b/2$ and no shear |

- All 44 spherical signatures of the first item gave the right group: the orders from 2 to 120 agree, no fold failed, and no point had a second image. The domains for $n = 2$ with two sides on one great circle are fundamental domains of their groups, although the angle at the pole is $\pi$.
- `(100)x` gets `NX` with $n = 100$, and `(101)x` gets `22X` with `iGroup_22X(1, 2.0)`, the Euclidean `22x` (Section 7.1.7). `*(101)(101)`, `(101)*`, `22(101)`, `2*(51)`, and `*22(51)` keep their keys with the curvature 0, and `(101)(101)` keeps the curvature 1.

The pass did not open the fold on the graphics card or the border code that Section 7.1.2 cites, and it did not trace what the curvature 0 changes downstream for the spherical families with a large $n$.

### 7.2. Triangle Groups

`Group_KLM` (`lib/grouplib/Group_KLM.js`) builds the triangle groups with corner orders $K$, $L$, and $M$ in all three geometries. All locations in this section are in that file.

#### 7.2.1. Orders, Angles, and Dispatch

`getAngle(fraction)` (lines 318-324) maps an order to an angle:

```js
if (fraction >= 20 || fraction < 2) return 0; else return Math.PI / fraction;
```

An order of 0, 1, or 20 and above therefore gives the angle 0, which is an ideal corner. `getFD()` (lines 263-268) calls `this.makeTriangle(this.getAngle(par.K), this.getAngle(par.L), this.getAngle(par.M))`, so $a_0 = \pi/K$, $a_1 = \pi/L$, and $a_2 = \pi/M$.

`makeTriangle` (lines 292-312) chooses the geometry:

```js
makeTriangle(a0, a1, a2) {
    if (abs(a0) < EPSILON) {
        return makeIdealTriangle(a0, a1, a2);
    }
    var factor = a0 + a1 + a2;
    if (abs(factor - PI) < EPSILON) {
        return makeEuclideanTriangle(a0, a1, a2);
    } else if (factor > PI + EPSILON) {
        return makeSphericalTriangle(a0, a1, a2);
    } else {
        return makeHyperbolicTriangle(a0, a1, a2);
    }
}
```

The tolerances are `EPSILON = 1.e-6` (line 29) and `EPSILON2 = 1.e-4` (line 28). The ideal branch runs only when the $K$ corner, $a_0$, is ideal. A triangle with an ideal $L$ or $M$ corner and $a_0 > 0$ goes through the angle-sum test. The four builders are free functions:

| Function | Lines |
| --- | --- |
| `makeSphericalTriangle(a0, a1, a2, triSize = 0.25)` | 332-349 |
| `makeEuclideanTriangle(a0, a1, a2, triSize = 0.25)` | 355-366 |
| `makeHyperbolicTriangle(a0, a1, a2)` | 371-389 |
| `makeIdealTriangle(a0, a1, a2)` | 398-436 |

No caller passes `triSize`. `makeTriangle` calls `makeSphericalTriangle(a0, a1, a2)` at line 307.

#### 7.2.2. Sides and Corners

The Euclidean, spherical, and hyperbolic builders share the first two sides (lines 336-338, 361-362, and 375-377):

- $s_0$ = `iPlane([0., -1., 0., 0.])`, the line $y = 0$ with outward normal $(0, -1)$, so the domain side is $y > 0$.
- $s_1$ = `iPlane([-sin(a0), cos(a0), 0., 0.])`, the line through the origin at angle $a_0$ to the $x$ axis, with the domain side $y \cos a_0 < x \sin a_0$.

The sides follow the sign conventions of Section 2.4. The comment at lines 270-277 places the corners: "K at the corner of the sides 0 and 1 (the origin), L of the sides 0 and 2 (on the x axis), M of the sides 1 and 2", and "of two points where sides meet, the corner is the one nearest the origin (the other is its image in the absolute)". `getHandles` (lines 278-290) finds the corners numerically with `cornerOf` (lines 282-283).

#### 7.2.3. Euclidean Triangle

`makeEuclideanTriangle` (lines 355-366) uses a line for the third side:

```js
var e = triSize; // size of triangle
return [
    iPlane([0., -1., 0., 0.]),
    iPlane([-sin(a0), cos(a0), 0., 0.]),
    iPlane([sin(a1), cos(a1), 0., e])
];
```

$s_2$ is the line $x \sin a_1 + y \cos a_1 = e$ at distance $e$ = `triSize` = 0.25 from the origin, with the domain side $x \sin a_1 + y \cos a_1 < e$.

#### 7.2.4. Spherical Triangle

`makeSphericalTriangle` (lines 332-349) uses a circle for the third side:

```js
s[0] = iPlane([0., -1., 0., 0.]);
s[1] = iPlane([-sin(a0), cos(a0), 0., 0.]);
var e2 = triSize; // size of triangle
var r = e2 * sin(a0) / (cos(a2) + cos(a1 + a0));
var cx = r * (cos(a2) + cos(a1) * cos(a0)) / sin(a0);
var cy = r * cos(a1);
s[2] = iSphere([cx, cy, 0, -r]);
```

`e2` is the parameter `triSize` with its default 0.25 (line 340), and it is the only free scale of the spherical triangle. The stored radius is $-r$, so by the convention of Section 2.4 the domain is inside the circle when $-r > 0$.

#### 7.2.5. Hyperbolic Triangle

`makeHyperbolicTriangle` (lines 371-389) uses a circle for the third side:

```js
s[0] = iPlane([0., -1., 0., 0.]);
s[1] = iPlane([-sin(a0), cos(a0), 0., 0.]);
var ax = (cos(a2) + cos(a1) * cos(a0)) / sin(a0);
var ay = cos(a1);
var r = 1. / sqrt(ax * ax + ay * ay - 1.);
var cx = r * ax;
var cy = r * ay;
s[2] = iSphere([cx, cy, 0, -r]);
```

In formulas:

$$a_x = \frac{\cos a_2 + \cos a_1 \cos a_0}{\sin a_0}, \qquad a_y = \cos a_1, \qquad r = \frac{1}{\sqrt{a_x^2 + a_y^2 - 1}}, \qquad c = r\,(a_x, a_y)$$

The stored radius is $-r$ with $r > 0$, so the domain is outside $s_2$. Where $s_2$ meets $s_0$ or $s_1$ at two points, the corner is the intersection nearest the origin, as the comment of Section 7.2.2 says.

#### 7.2.6. Ideal Triangle

`makeIdealTriangle` (lines 398-436) builds a triangle with an ideal $K$ corner:

```js
s[0] = iPlane([0., -1., 0., 0.]);
s[1] = iSphere([-1., 1., 0., -1.]);
var denum = (cos(a1) + cos(a2));
if (abs(denum) < EPSILON) { s[2] = iSphere([0., 0., 0., 1.]); return s; }
var r3 = 2. / denum;
var x3 = 1.;
var y3 = r3 * cos(a1);
//  invert in the sphere S3 = S((-1,0), 2)
var dd = ((x3 + 1.) * (x3 + 1.) + y3 * y3 - r3 * r3);
if (abs(dd) > EPSILON2) {
    var a = 4. / dd;
    x3 = -1. + a * (x3 + 1.);
    y3 *= a;
    r3 *= a;
    s[2] = iSphere([x3, y3, 0, -r3]);
} else {
    if (abs(a2) < EPSILON) { s[2] = iPlane([1., 0., 0, 0.]); }   // a1 = pi/2, a2 = 0
    else                   { s[2] = iPlane([1., 1., 0, 0.]); }   // a1 = pi/4, a2 = pi/2
}
```

The comment at lines 391-396 states the design. $s_0$ is the plane with normal $(0, -1, 0)$ through the origin, $s_1$ is the sphere with centre $(-1, 1, 0)$ and radius 1, and $s_2$ is a sphere or plane that meets them at the given angles. The code builds $s_2$ in these steps:

1. Take the circle with centre $(1, r_3 \cos a_1)$ and radius $r_3 = 2/(\cos a_1 + \cos a_2)$ (the variables `x3`, `y3`, and `r3`) as the first form of $s_2$.
2. Invert it in the sphere with centre $(-1, 0)$ and radius 2, as the comment at line 415 says. With the power $dd$ of the inversion centre, the factor is $\alpha = 4/dd$, the new centre is $(-1 + \alpha(x_3 + 1),\ \alpha y_3)$, and the new radius is $\alpha r_3$.

$s_1$ is stored as `iSphere([-1., 1., 0., -1.])`, which is the circle with centre $(-1, 1)$ and radius 1 and the domain outside it. $s_2$ is stored with the radius $-\alpha r_3$, so its domain side depends on the sign of $\alpha$. Two special cases replace the circle:

- When $\lvert \cos a_1 + \cos a_2 \rvert <$ `EPSILON`, $s_2$ is the unit circle `iSphere([0,0,0,1])`, with the domain inside it.
- When $\lvert dd \rvert \le$ `EPSILON2`, the inversion centre lies on the circle and $s_2$ is a line: `iPlane([1,0,0,0])`, the line $x = 0$, when $a_2$ is near 0, and otherwise `iPlane([1,1,0,0])`, which the code builds with the normal `[1,1]` and comments as the case $a_1 = \pi/4$, $a_2 = \pi/2$.

#### 7.2.7. Groups Built from the Triangle

Two methods build a group from the triangle $[S_0, S_1, S_2]$:

- `getGroup_sKLM` (lines 193-223) builds the reflection group, with `fd = [S0, S1, S2]` and `trans = [[S0],[S1],[S2]]`. With `hasR`, it appends `iSphere([0,0,0, 1.])`, the unit circle, as a fourth side, so points outside the disc fold into it.
- `getGroup_KLM` (lines 228-258) builds the rotation subgroup of index 2. It sets `S1a = iReflectU4(S0, S1)` and `S2a = iReflectU4(S0, S2)`, then `fd = [S1, S2, S1a, S2a]` and `trans = [[S1, S0], [S2, S0], [S0, S1], [S0, S2]]`. The domain is the double triangle across the $x$ axis, with 4 sides, and each side pairing is a product of two reflections.

In `getGroup_KLM`, `new Group({s: fd, t: trans})` at line 246 runs before the `hasR` side is pushed at lines 248-253. Section 15.3 records this.

## 8. Generators and the Group Object

### 8.1. Generators per Edge Type

`produceGenerators` (`lib/orbilib/OrbifoldGeometrization.js:1387`) builds one transform for each side of the assembled polygon from its key. In this list, $p$ and $q$ are the ends of the side, and $e$ is its hyperbolic length.

| Edge key | Isometry | Construction |
| --- | --- | --- |
| `mirror`, `mirrorRedundant`, `cornerPair` | Reflection | `[sPlaneReflectAcross(p,q)]`, one splane |
| `handle`, `cuttingEdge`, `band` | Orientation-preserving map of side $i$ onto its partner $j$, found by matching key and index | `sPlanesMovingEdge1ToEdge2([p1,p2],[q2,q1])` (`lib/invlib/ComplexArithmetic.js:789`), two splanes. For a handle with a twist, $p_1$ and $p_2$ first move along the edge by `shift*getLength(edge,group)`. |
| `conePair` | Half-turn | The centre is `q.applyMobius(poincareMobiusTranslateFromToByD(q,r,dd*e))`, with `dd = d<-.25? d+1: d<.25? d+.5: d` for the twist `d`, then `sPlanesOfRotation(point,PI)`. Only one of the two cone points becomes a generator. |
| `fold` | Half-turn about the midpoint of the edge | $v$ at distance $e/2$, then `sPlanesOfRotation(v,PI)` |
| `cap` | Glide reflection of length $e/2$ | $v = p + e/4$ and $w = p + e/2$ along $p \to q$, then `[sPlaneSwapping(v,p), sPlaneSwapping(v,w), sPlaneReflectAcross(p,q)]` (line 1467) |

`sPlanesOfRotation` returns two splanes through the point at an angle of $\pi/2$ to each other, which compose to the half-turn.

The rows for `cornerPair`, `mirror`, and `mirrorRedundant` share one branch, with the reflection across the line through `points[i]` and `points[(i+1)%points.length]` (lines 1471-1479). The loop at line 1417 gives one row for each edge, in edge order, and skips no redundant edge. The comment at lines 1397-1399 gives the reason: both of two collinear edges get a row, although Bulatov's algorithm uses only one, because a Dirichlet domain of a non-convex domain may need all neighbouring group elements. A key that the switch does not name gets no row, so the row index then differs from the edge index. For a kaleidoscope, the splice of Section 6.5.1 drops the edge after each slice, so the code reading inferred that no `mirrorRedundant` edge reaches `produceGenerators`, and that the generators are the $n$ reflections in the sides of the polygon, in the order of Section 6.8.

### 8.2. Splane Helpers

These helpers in `lib/invlib/ComplexArithmetic.js` build the splanes:

- `sPlanesMovingEdge1ToEdge2` first tries one reflection in a line, with the tolerance `SHORTEPSILON`. Otherwise it swaps $p_1$ with $q_1$, then the image $q_3$ with $q_2$. A comment at line 787 says "NOTE: this is not correct if p1=q1".
- `sPlaneSwapping(pt1,pt2)` (line 644) returns, in the words of its comment at lines 641-642, "the center of the circle to invert through to swap pt1 and pt2, and preserving the unit circle".
  - Names: `var a = pt1.re, A=pt1.im,b=pt2.re,B=pt2.im` (line 646), so lowercase is the real part and uppercase is the imaginary part. `var AA=a*a+A*A, BB = b*b+B*B, DD = AA-BB` (line 647), so `AA` and `BB` are the squared moduli.
  - Result: The centre is `[(b*(AA-1)-a*(BB-1))/DD, (B*(AA-1)-A*(BB-1))/DD]` (line 661), returned as `[center, 1]` (line 662).
  - Degenerate cases: For equal points it returns the line `[[-A,a],0]` (line 652). When $\lvert DD \rvert$ is below `SHORTEPSILON`, it returns the line through the origin `[[a-b,A-B],0]` (line 658).
- `sPlaneThrough(pt1,pt2,ends)` (line 675, documented at lines 667-673) returns the geodesic through two points of the disc.
  - Names: It uses the same `a`, `A`, `b`, and `B` (line 676). In the main branch, `DD=2*(-A*b+a*B)` (line 693), `AA=1+a*a+A*A` and `BB=1+b*b+B*B` (line 704), so here `AA` and `BB` are one plus the squared moduli, unlike in `sPlaneSwapping`.
  - Result: The centre is `[(AA*B-A*BB)/DD,(-AA*b+a*BB)/DD]` (line 705). The sign `r` starts at 1 and becomes $-1$ when `center[0]*(B-A)+center[1]*(a-b)>0` (lines 706-709).
  - Degenerate cases: When one point is the origin, the code reuses `AA` and `BB` as plain moduli, `sqrt(A*A+a*a)` and `sqrt(B*B+b*b)` (lines 684 and 689). When $\lvert DD \rvert$ is below `SHORTEPSILON`, it returns a line.
- `splaneIt` (line 537) turns an output `[c,r]` into a splane.
  - When $r = 0$ it returns `iPlane([c0/|c|, c1/|c|, 0, 0])`.
  - Otherwise it computes `radius = sqrt(magCenterSqd-1)` (line 556) and returns `iSphere([c[0],c[1],0., r*radius])` (line 574), so the sign of `r` becomes the sign of the radius.
  - It also turns a circle whose squared centre exceeds `MAX_RADIUS_SPHERE_SQD = Number.MAX_SAFE_INTEGER/32` into a plane.

### 8.3. The Group Object

`WallPaperGroup_General.updateTheGroupGeometry`, at about lines 296-360, packages the result as `FD = {s: bounds, t: transforms, i: interiors}`. Each entry of `s` is a side, as a list of one primary splane and optional wall splanes. Each entry of `t` is the output of `produceGenerators` for that side. `Group_Orbifold.getGroup()` builds `new Group({s: flatS, t: fd.t})` from the primary splanes only (`lib/orbilib/Group_Orbifold.js:165-168`), so the walls do not reach the current renderer.

The class `Group` (`lib/invlib/Group.js:19`) holds:

- `s`, also called `fundDomain`: The array of sides of $F$.
- `t`, also called `revTransforms`: The reverse transforms, one list of splanes per side, which the code calls "old style".
- `transforms`: Their inverses, with the generator names `a`, `b`, `c`, and so on, one per side.
- `genMap`: The map from word letters to transforms.

A pure reflection group defaults its transforms to `GroupUtils.makeReflections(fundDomain)` (`lib/invlib/Group.js:57-62`).

### 8.4. Handing the Group to the Renderer

`SymRenderer.prepareGroupData` (`lib/symhublib/SymRenderer.js:2384-2418`) applies the user's group transform and an optional Dirichlet domain (`mDirichlet.apply`), then calls `DataPacking.packGroupToSampler(gl, mPackedGroupData, mGroup)` (line 2410). The packing (`lib/invlib/DataPacking.js:150-195`) uploads one row of `RGBA32F` texels with nearest filtering, in this layout:

1. A header: `[domainOffset, transformsOffset]`.
2. The domain: `count`, then 2 texels per side, the `vec4` of the splane and then its `type` in `.x`.
3. The transforms: `count`, then the offsets, then for each transform its `refCount` followed by 2 texels per reflection.

The renderer keeps a second copy, `mTransGroup` (`lib/symhublib/SymRenderer.js:3206-3220`), which is the group carried into the space of the pattern texture by the inverse pose of the pattern box. Simulation and symmetrization use it, and the on-screen render uses `mGroup`.

The older `apps/orbifold` uses plain uniform arrays instead (`apps/orbifold/js/shaders/FDRenderer.glsl.mjs:19-35`), with each splane as 5 floats `[x,y,z,w,type]` and the fixed limits `MAX_GEN_COUNT=4`, `MAX_REF_COUNT=4`, and `MAX_CROWN_COUNT=20` (`lib/shaders/inversive.glsl.mjs:3-20`). `lib/shaders/inversive.glsl.mjs:23-59` still uses the uniform arrays to draw the interior of $F$ and its distance.

## 9. Point Reduction

### 9.1. The Processor Loop

`toFundDomain(params)` (`lib/invlib/Group.js:191`) reduces a point with these steps:

1. Set `maxIterations` to 20 unless the caller gives a value (line 194).
2. For each side $i$ in order, compute $d_i$ = `U4.sigDistanceSP(fd[i], pnt)` (line 211).
3. At the first side with $d_i > 0$, apply its reverse transform `trs[i]`, append it to the total with `totalTrans.concat(tr)`, and restart at side 0.
4. When a full pass finds no side with $d_i > 0$, set `inDomain = true` and stop.
5. Return `{inDomain, transform, pnt, word}`.

The processor loop has no tolerance and assumes that $F$ is convex. Its default point is `iPoint([0,1,0.2,0.3,0.01])`, which has 5 components. `GroupUtils.toFundDomain` (`lib/invlib/GroupUtils.js:460`) has the same logic with a `maxIterations` from the caller.

`iToFundDomainWBounds` (`lib/invlib/Inversive.js:1245`), the variant for orbifold domains with walls, uses only the first splane of each side. Its comment at line 1249 reads "for the moment, screw the bounds! Just assume we're convex ... BUT THIS MIGHT BE A PROBLEM LATER!".

### 9.2. The Shader Loop

`iToFundamentalDomainSampler` (`lib/shaders/inversiveSampler.glsl.mjs:182-236`) runs the same loop per pixel:

```glsl
for(int count = 0; count <  iterations; count++){
    int found = 0;
    for(int g = 0; g < domainSize; g++){
        iSPlane sp = fetchSplane(groupData,domainSplanesOffset + g*2);
        float ip = iDistance(sp, pnt);
        if(ip > EPS){            // EPS = 1.e-7 (line 184)
            ... for r < refCount: iReflect(rsp, pnt, scale);
            refcount++; found = 1; break;
        }
    }
    if(found == 0){ inDomain = 1; break; }
}
```

The loop differs from the processor loop in these points:

- A side counts as violated only when $d > \varepsilon$ with $\varepsilon = 10^{-7}$ (line 184). A commented-out snap, `//if(abs(ip) < EPS) ip = 0.0;`, sits at line 208.
- The outer loop is bounded by the uniform `uIterations`. Its default is 10 (`lib/symhublib/SymRenderer.js:393`), and the user interface allows 0 to 1000 (lines 1385-1393). Presets use larger values, such as 40 for a 442 colour preset. The symmetrization pass uses 2 by default (`lib/symhublib/SymmetrizationWorker.js:70`).
- It tracks the scale $s$ through each reflection, a count of folds `refcount`, and `inDomain`.

When the loop ends without reaching $F$, `inDomain` stays 0, and the colour and image layers write transparent black (`lib/shaders/bufferToScreenImage.glsl.mjs:61-64`, `lib/shaders/colorTiles.glsl.mjs:53-56`). This is why the region near the absolute is blank. `refcount` is computed but never used for colour. The shader tracks no word, no parity of reflections, and no tile identifier.

### 9.3. Convergence

The header of `lib/grouplib/Group_ON.js`, at about lines 52-54, notes that its side S1 appears twice in the side list, and that "the renderer folds a point by the first side it is outside of, and the fold still converges since every transform is a group element". This is an assertion without a proof. The only bound in the code is the iteration cap: the walled shader loop at `apps/orbifold/js/shaders/inversive.glsl.mjs:754` runs at most `iterations` times, from the uniform `u_iterations`, and leaves `inDomain` at 0 (line 747) when it runs out.

### 9.4. The Walled Variant

The older shader `iToWalledFundamentalDomain` (`apps/orbifold/js/shaders/inversive.glsl.mjs:733-802`, with its documentation comment from line 729) folds against sides that are intersections of several splanes. Its parts are at these lines:

- Line 733: The signature, `void iToWalledFundamentalDomain(`, whose last parameter `int iterations` is at line 743.
- Line 754: The outer loop, `for(int count = 0; count < iterations; count++){`.
- Line 756: The comment that states the convention of Section 2.4.
- Line 771: The wall test `if(iDistance(sp, p) <= 0.)`, followed by `check = -1`. A side is skipped when the point is inside any of its wall splanes.
- Line 787: The reflection `iReflect(rsp, p, scale)`.
- Lines 796-799: The exit `if(found == 0){` with `inDomain = 1` and `break`.

Cumulative counts replace per-item headers in its data. This shader file sets `MAX_ITERATIONS 100` (lines 15-16) and `EPSILON = 1.e-10` (line 39). `apps/orbifold/js/shaders/patternFromFDRenderer.glsl.mjs:136` calls it, and so do the `_old` copy at line 128 and the `_v1` copy at line 135.

### 9.5. The View Anchor

The view anchor is in `lib/grouplib/ViewAnchor.js`, 885 lines, the only file of that name in the repository. `lib/grouplib/modules.js:359` imports from it, and `lib/symhublib/modules.js:801` re-exports only `makeViewAnchor`. Its header at lines 1-3 describes it as the element of the group "that folds the view's centre into the fundamental domain F".

A view far from $F$ runs out of iterations and stops drawing (`lib/grouplib/ViewAnchor.js:1-49`). The view anchor fixes this. On the processor, it picks a group element that moves the view centre into $F$, computed in double precision. In the shader, `applyAnchor` (`lib/shaders/inversiveSampler.glsl.mjs:127-176`) applies it to the small offset $w$ of the pixel from the view centre $c$ as

$$a(c + w) = q + \frac{w'}{A + B w'}$$

with an anti-conformal form when the flag `anti` is set. The fold then starts from the anchor's state, which is the colour permutation `uAnchorPerm` and the subgroup class `uAnchorClass`. With `VIEW_OFFSET` defined, the whole chain of the view map and the anchor is composed in double precision and evaluated from `vViewOffset` (lines 140-168).

The anchor is the identity near $F$, and wherever the view straddles a side, such as a limit set across the screen. It is off for spherical groups and while "infinite pan" is off (the comment above `updateViewAnchor`, around `lib/symhublib/SymRenderer.js:2795-2845`). Its budget is 6 milliseconds per update (`DEFAULT_BUDGET_MS` in `lib/grouplib/ViewAnchor.js`).

## 10. Projections and Models

### 10.1. The Projection Chain

A model of the plane is a map applied to the pixel's point before the fold. The disc model is the identity. Every other model is a conformal map from the screen plane into the group's world, which is the disc for hyperbolic groups. `makeProjection(inout vec2 p, inout float scale)` (`lib/shaders/projection_v2.glsl.mjs:205-244`) applies the maps of rows 2 to 10 of the table in Section 10.2, in that order. The sphere map of row 1 is a separate function, `makeSphericalProjection`. Each map also updates the scale $s$. The Euclidean plane uses no map, and the canvas transform provides pan, zoom, and rotation.

The single vertex shader (`lib/shaders/canvasVertexShader.glsl.mjs:17-28`) computes the screen point. It forms $v$ = `u_scale*position.xy*vec2(1.,u_aspect)`, rotates it by `u_rotation` into `vViewOffset`, and sets `vUv = u_center + vViewOffset`. The separate offset keeps float precision at deep zoom (Section 15.2).

### 10.2. The Maps

In this table, $p$ is the point and $\lvert p \rvert^2$ is written `pp` in the code.

| Order | Map | Uniform | Formula | Source |
| --- | --- | --- | --- | --- |
| 1 | Sphere, outside `makeProjection` | `u_sphericalProjectionEnabled` | See below | `makeSphericalProjection`, `projection_v2.glsl.mjs:250-266` |
| 2 | Moebius | `u_hasMoebiusTransform`, `u_moebiusTransformData[20]` | Up to 4 reflections with `iReflect` | `transformPoint`, `projection_v2.glsl.mjs:45-52` |
| 3 | Rational | `uRationalMapEnabled`, `uRationalMapData` | See below | `rationalMap`, `projection_v2.glsl.mjs:121-156` |
| 4 | Upper half-plane to disc | `u_uhpEnabled` | See below | `uhp2disc`, `complex.glsl.mjs:140-147` |
| 5 | Logarithm | `u_expEnabled` | $p \mapsto \log p$ | `plane2band`, `complex.glsl.mjs:157-163` |
| 6 | Loxodromic | `u_loxEnabled` | Inversion in the circle with centre $(-1, 0)$ and radius 2, then $x \mapsto x - 1$, then `plane2band` | `projection_v2.glsl.mjs:159-168` |
| 7 | Scale and shift | `uCScaleEnabled`, `uCScale`, `uCShift` | $p \mapsto (p - p_0)\, \gamma$ | `cScale`, `complex.glsl.mjs:168-171` |
| 8 | Periodic wrap | `u_periodicWrapEnabled`, `u_periodicWrapData[21]` | See below | `periodicWrap`, `projection_v2.glsl.mjs:73-110` |
| 9 | Band to disc | `u_bandEnabled`, `u_bandFraction` | See below | `band2disc`, `complex.glsl.mjs:112-125` |
| 10 | Fixed-points Moebius | `u_fpTransformEnabled`, `u_fpTransformData` | A Moebius map animated by a word, `loopTime`, and `fraction` | `FixedPointsTransform.js` |

The maps compute these formulas:

- Sphere: The code comment calls it a "stereographic spherical projection". `makeSphericalProjection` (`lib/shaders/projection_v2.glsl.mjs:250-266`) reads:

  ```glsl
  float pp = dot(p,p);
  float sdist = sqrt(pp)-1.;
  if(sdist > 0.)
      return sdist;
  float factor = 1./(1. + sqrt(1.-pp));
  float factor2 = 1./sqrt(1.-pp);
  p *= factor;
  //scale *= factor;
  scale *= factor2;
  return sdist;
  ```

  With $z = \sqrt{1 - \lvert p \rvert^2}$, the code multiplies the point by the position factor $1/(1 + z)$ and the scale by the single factor $1/z$:

  $$p \mapsto \frac{p}{1 + z}, \qquad s \mapsto \frac{s}{z}$$

  An earlier version that multiplied the scale by the position factor survives as the commented-out line `//scale *= factor;`. The function returns $\lvert p \rvert - 1$, and a pixel with $\lvert p \rvert > 1$ is transparent, so only one hemisphere is visible. `lib/shaders/colorTiles.glsl.mjs:33`, `lib/shaders/overlay_main.glsl.mjs:26`, `lib/shaders/iso_main.glsl.mjs:314`, and the `bufferToScreen*` shaders call it. The line `#define PROJECTION_SPHERE      6` at `lib/shaders/projection_v2.glsl.mjs:179` is inside the block comment at lines 172-185 (Section 10.3), so it defines nothing in this file. The exact metric factors of this map are an open question (Section 17, item 2).
- Rational: $p \mapsto \prod_i (p - z_i)^{n_i}$ for up to 4 points $z_i$, with $n_i \in \{\pm 1, \pm 2, \pm 3\}$. The scale uses hardware derivatives, `scale *= fwidth(|qp|)/fwidth(|q|)` (line 154).
- Upper half-plane to disc: The comment on `uhp2disc` says "conformally maps upper-half plane y > 0 to the unit disk". With $\mu = x^2 + y^2$ and $D = \mu + 2y + 1$:

  $$(x, y) \mapsto \left( \frac{1 - \mu}{D}, \frac{2x}{D} \right), \qquad s \mapsto \frac{2s}{D}$$

  Section 17, item 9 records an identification of this map with a Moebius formula, which the code does not state.
- Logarithm: The code divides the scale by $\lvert p \rvert$, the modulus of the point before the logarithm, as `scale /= length(p)` before `vec2 pp = cLog(p.xy)` (`lib/shaders/complex.glsl.mjs:157-163`).
- Scale and shift: The factor is $\gamma = (n_1 a_1 + n_2 a_2)/(2\pi i)$, set in `lib/symhublib/ProjectionTransform.js:361-368`.
- Periodic wrap: The code subtracts the centre, converts to the dual basis, subtracts the rounded coordinates, and converts back, as `p -= centre; e = dual.p; e -= round(e); p = e.a`. The processor builds the dual basis (`ProjectionTransform.js:408-439`).
- Band to disc: With the fraction $f$, $\alpha = (\pi/4) f$, and $\beta = 1/\tan \alpha$:

  $$p \mapsto \beta \tanh(\alpha p)$$

  The map is the identity when $\lvert \alpha \rvert < 10^{-4}$. A comment in the code says that $\beta = 1$ when $f = 1$, and the map is then $\tanh(\pi p/4)$. The fixed form at lines 77-82, `vec2 band2disc(vec2 z){ return cTanh(z*(PI/4.)); }`, carries the comment "conformally maps infinite horizontal band -1 < y < 1 into unit disk". Its scale variant at lines 94-100 sets `float coeff = (PI/4.);` and `scale *= coeff*length(vec2(1,0) - cMul(pp, pp));`. `getBandFraction` (`ProjectionTransform.js:456-463`) maps a value $f < 1$ of the user interface to $(1 - 10^{-6f})/(1 - 10^{-6})$.

### 10.3. Processor Mirror and Other Code

`lib/symhublib/ViewChain.js:9-40` documents the order of the chain. `viewChainToWorld` (lines 135-154) and `viewChainScale` (lines 161-166, by central differences) implement it on the processor, and `tests/symhublib/view_chain.mjs` tests it. It is a small reference for these maps that runs on the processor.

The file `lib/shaders/projection.glsl.mjs` is an older copy. The `PROJECTION_*` defines at `projection_v2.glsl.mjs:172-185` are commented out. In unused code, `cSin` and `cCos` at `complex.glsl.mjs:62-74` call `cInverse(c)` where `cInverse(e)` is meant.

The Klein model, the hyperboloid model, the Beltrami model, and a Mercator map are not shader models. The processor uses quadratic forms of the hyperboloid or the sphere for motions (`lib/invlib/Geodesics.js:28`, `lib/invlib/FactorizationU4.js:8`, `lib/symhublib/InversiveNavigator.js:2194`), and `lib/drawlib` uses a Klein or gnomonic chart for shapes (`DrawingGeometry.js:61-67`).

## 11. Borders, Antialiasing, and the Limit Circle

### 11.1. Tile Borders

Tile borders are an overlay item drawn in a separate pass over the flat-colour layer (`lib/shaders/overlay_tiling.glsl.mjs:11-25`):

```glsl
OverlayPoint op = overlayFoldedPoint(pnt, scale, pixelSize);
...
float distPix = abs(iDistance(sp, vec3(op.pfg, 0.)))/(pixelSize*op.gscale);
float dens = smoothstep(0.5,-0.5, distPix - 0.5*uTilingWidth);
tilingDens = max(tilingDens, dens);
```

The method has these steps:

1. Fold the pixel into $F$.
2. For each side of $F$, measure the Euclidean distance from the folded point to the side, in the model.
3. Divide by `pixelSize * gscale` to get the distance in screen pixels.
4. Take the coverage `smoothstep(0.5, -0.5, distPix - 0.5*uTilingWidth)`, a ramp one pixel wide centred on the edge of the border.
5. Keep the maximum coverage over the sides, and draw it in `uTilingColor`, with premultiplied alpha.

The width `uTilingWidth` is in pixels. Its default is 1, and the default colour is `#000000FF` (`lib/symhublib/overlays/OverlayTiling.js:12-31`). The overlay ignores `inDomain` (`lib/shaders/overlay_pre.glsl.mjs:153-162`), so it still draws from the last fold state where the fold did not converge.

### 11.2. Pixel Size and Scale

The overlay computes the size of a pixel in the view plane as `pixelSize = length(dFdx(vViewOffset))`, or `abs(dFdx(pnt.x))` without `VIEW_OFFSET` (`lib/shaders/overlay_main.glsl.mjs:11-14`). The fields `op.pntscale` and `op.gscale` (`lib/shaders/overlay_pre.glsl.mjs:119-147`) carry the magnification of the projection and of the fold. The colour and image shaders instead use the uniform `u_pixelSize`, which is $2/(\text{width} \cdot \text{zoom})$ (`lib/symhublib/CanvasTransform.js:310` sets the uniform, and `lib/invlib/CanvasTransform.js:42` has the same formula).

### 11.3. Antialiasing

All antialiasing is analytic, with one sample per pixel. The canvas is created with `antialias:false` (`lib/symhublib/webgl_utils.js:42`), and `sizeMultiplier`, from 1 to 4, only enlarges the canvas (`lib/symhublib/SymRenderer.js:1228`). These are the techniques:

- Edges use the one-pixel `smoothstep` of Section 11.1.
- The mask of the texture box is `1 - smoothstep(-blur, blur, sdb)` with `blur = 0.5*pixel` (`lib/shaders/bufferToScreenImage.glsl.mjs:68-73`).
- Images choose a mipmap level from the tracked scale, $\text{LOD} = \log_2(\text{TSIZE} \cdot \text{u\_pixelSize} \cdot s \cdot \lVert \text{uBufScale} \rVert \cdot \text{bendScale})$ (`lib/shaders/bufferToScreenImage.glsl.mjs:79-90`).
- The rational map takes its scale from `fwidth`.
- A palette in `apps/animated_patterns/js/shaders/frag_main.glsl.mjs:10-20` fades high-frequency cosines by `smoothstep(6.28,0.0,fwidth(x))`.

### 11.4. Outline of the Fundamental Domain

`lib/shaders/overlay_funddomain.glsl.mjs` takes the signed distance to $F$ as the maximum over the sides of `iDistance/(pixelSize*pntscale)`, and draws the outline with `smoothstep(0.5,-0.5, abs(fdDist) - 0.5*uFDoutlineWidth)` (line 45), plus a shadow band. The generator overlay (`lib/shaders/overlay_generators.glsl.mjs:112-195`) draws sides, cone symbols, mirrors, dashed glide axes, and arrows in pixel units.

### 11.5. The Limit Circle

SymmHub controls the region near the absolute in these ways:

- Pixels whose fold does not converge are transparent in the colour and image layers (Section 9.2). The number of iterations is the only control of how close to the absolute the tiling reaches.
- The limit-set overlay (`lib/shaders/overlay_limitset.glsl.mjs:13-17`) draws where the folded tile is smaller than `uLsThickness` pixels:

  ```glsl
  float lensHeight = 1.;
  float lsDist = lensHeight / (op.scale*pixelSize);
  float lsDens = smoothstep(0.5,-0.5, lsDist - uLsThickness);
  return op.mask * (lsDens * uLsColor);
  ```

- A helper `lens()` and `inv_circle_metric()` exists at `lib/shaders/overlay_pre.glsl.mjs:72-86`. The code reading did not find where it is used.
- The current shaders do not clip the outside of the disc. The older `apps/orbifold/js/shaders/FDRenderer.glsl.mjs:105` clips with `if(p.x*p.x+p.y*p.y>1.){return u_backgroundColor;}`.

## 12. Colouring

### 12.1. Flat Tile Colours

The `colorTiles` layer (`lib/shaders/colorTiles.glsl.mjs:22-63`) sets `color = uCellColors[get_perm_val(currentPerm, uCellColorPermIndex)]`, then `outColor = color * (1. - uTransparency)`. Pixels outside $F$ after the fold are transparent. The layer's uniforms are `uPermData`, `uPermSize`, `uUseOrbit`, `uAnchorPerm`, `uFillCells`, `uCellColors`, and `uCellColorPermIndex` (`lib/symhublib/VisualizationColorTiles.js:169-179`, the layer at lines 156-193).

Colours come only from a permutation or a coset state carried through the fold. SymmHub has no tile identifier by generator index or by word.

### 12.2. Permutations Carried through the Fold

`iToFundamentalDomainSamplerPerm24` (`lib/shaders/permutations24.glsl.mjs:53-130`) is the fold loop of Section 9.2 with the same `EPS`. Each fold composes the current permutation with the permutation `permData[g]` of the side $g$ (lines 114-117), and it does not increment `refcount`:

```glsl
if(useOrbit)
    currentPerm = compose_perms(permData[g], currentPerm, permSize);
else
    currentPerm = compose_perms(currentPerm, permData[g], permSize);
```

These are the data:

- A permutation of up to 24 entries is packed in a `uvec4`, with 5 bits per entry and 6 entries per component (`perm_identity` at lines 15-23 and `get_perm_val` at lines 26-30).
- `compose_perms(outer, inner, permSize)` (lines 34-48) returns the permutation $i \mapsto \text{outer}(\text{inner}(i))$.
- `uniform uvec4 uPermData[MAX_PERM_COUNT]` holds one permutation per side of $F$, with `MAX_PERM_COUNT=32` (line 8) and `MAX_COLORS_COUNT=24` (line 10).
- `uUseOrbit` reverses the order of composition, which colours by orbit instead of by coset. The comment at lines 107-113 explains the reverse composition.

On the processor, a document stores one permutation per generator as text, such as `"ba ba ba "`, in which a letter is its character code minus that of `a` (`strToPermutations`, `lib/grouplib/Permutations.js:108`). The preset `apps/sympix/presets/color/par-26-04-29-12-56-34-719.json` uses this for the KLM group 4 4 2 with `s:true` and a swap of 2 colours on every generator, which gives a checkerboard colouring by parity. `wordToPerm` (lines 52-69) gives the permutation of a word, and `sidePermutations` (lines 87-100) maps the generator permutations to the sides actually drawn, because the sides of a Dirichlet domain are words in the generators. `lib/symhublib/PermutationPacking.js:11-33` packs them for the shader.

### 12.3. Coset Automaton

`subFold` (`lib/shaders/overlay_subgroup.glsl.mjs:60-100`) is the same loop with a coset class instead of a permutation: each fold sets `cls = subNextClass(cls, g)` from a table indexed by class and side, one texel per entry. The header at lines 1-36 describes the data, and `SUB_MAX_SIDES` is 16 (line 36).

### 12.4. Subgroup Catalogue and Enumeration

`lib/color_groups/{klm,sklm,wallpaper}/sub_<name>.json` holds a catalogue of subgroups with their coset permutations. For example, `lib/color_groups/klm/sub_222.json` holds `"generators": "a b", "cosets": "ab ba ab ba"`. The script `lib/color_groups/generate_manifests.js` and the library `lib/sublib` generated the catalogue. `lib/sublib/src/sublib_core.js`, 1254 lines, implements a Sims-style backtrack search for coset tables of low index and their conjugacy classes, with conventions that its comments call identical to GAP's. `lib/sublib/src/reidemeister.js`, 158 lines, implements Reidemeister-Schreier rewriting. Section 12.8 describes both files. `Subgroups.js`, `SubgroupTable(s).js`, `SubgroupDomain.js`, and `SubgroupConvex.js` build coloured and subgroup domains from them. `tests/subgroups/*` and `tests/sublib/*` test this code.

### 12.5. Palettes

`lib/symhublib/ColorTiles.js:72-207` builds a palette of $N \le 24$ colours with the cosine formula

$$\text{colour}_i = a + b \cos\left(2\pi (c\, t_i + d)\right), \qquad t_i = i/N$$

followed by adjustments in OKLCH of hue, saturation, lightness, and contrast. Each slot can be edited by hand and has its own alpha, and a mask string holds one character, 0 or 1, per colour. Built-in palettes are in `lib/symhublib/Palettes.js:41-60`.

### 12.6. Images in the Fundamental Domain

These layers fill $F$ with content other than flat colour:

- Image: `lib/shaders/bufferToScreenImage.glsl.mjs:57-93` folds the point, maps it into the pattern box with `bufBentCoord`, and samples with linear, biquadratic, or box filtering (`lib/shaders/texUtils.glsl.mjs:36-60`). The pattern box is a similarity with an optional Moebius bend (`lib/shaders/bufferPose.glsl.mjs:106-118`, `lib/symhublib/PatternPose.js`).
- Colour map: `lib/shaders/bufferToScreenColormap.glsl.mjs:57-84` maps scalar data to colour with banding and wrap.
- Colour images: `lib/shaders/colorImageArray.glsl.mjs` picks one image of a `sampler2DArray` by `get_perm_val(currentPerm, uTexPermIndex)`. `applyColoring` (`lib/shaders/color_utils.glsl.mjs:124-146`) has the modes none, multiply, screen, and OKLCH hue shift.
- Crown: `lib/shaders/colorImageArray.glsl.mjs:91-130` applies each side pairing to the folded point, reads the images of the neighbour cells, and composites them, so that an image can overhang $F$. An optional depth-sorted variant uses `uTiltVector` and an insertion sort (lines 141-207).

### 12.7. Limits

Colouring is limited to 24 colours by the 5-bit packing and `MAX_COLORS_COUNT`, to 32 sides with permutations, to 16 sides in the coset automaton, and to 20 crown elements.

### 12.8. The Subgroup Library `sublib`

This subsection records the code of `lib/sublib/src/sublib_core.js` (1254 lines, no imports) and `lib/sublib/src/reidemeister.js` (158 lines, no imports). Section 7.19 of `docs/research/colourings.md` states the steps of the search and compares them with the published algorithm. The other files of the directory are these:

- `sublib.js`, the public entry point. It imports from the other files (`lib/sublib/src/sublib.js:18-48`), exports their functions (lines 78-110 and 112-120), and defines its own wrapper `subgroupsData` (lines 73-76).
- `groups_description.js`, which holds the preset presentations.
- `modules.js`, which re-exports only `ParseError`.

No function in `lib/sublib` takes subgroup generators and computes a coset table, and no code there defines cosets by a strategy, queues coincidences, or merges cosets. A case-insensitive search for `todd` and `coxeter` in `lib/sublib` finds nothing.

These are the data:

- A word is an array of nonzero integers, with $+k$ for generator $k$, counted from 1, and $-k$ for its inverse (`sublib_core.js:39-40`). In text a lowercase letter is a generator and an uppercase letter its inverse. `[x,y]` is $x^{-1} y^{-1} x y$ (`sublib_core.js:100`).
- `makePresentation(genText, relText)` returns `{ gens, relators, relStrings }`. A relator `u = v` becomes the free reduction of $u v^{-1}$, and empty relators are dropped from `relators` but not from `relStrings` (`sublib_core.js:177-190`).
- A coset table is a flat `Int32Array` with the entry `tab[c*nCols + col]`, `nCols = 2*nGens`, cosets counted from 0 with coset 0 the subgroup, column $2k$ for generator $k$ counted from 0 and $2k + 1$ for its inverse, and `-1` for an undefined entry (`sublib_core.js:21-27`, 196-197, and 228). The search allocates it once at `maxIndex` rows and fills it in place.
- `subgroupClasses(pres, maxIndex, options)` returns `{ classes, stats }`. A class is `{ index, table, nCols, size, normal, key }`, where `size` is the number of conjugate subgroups, `normal` is `size === 1`, and `key` is `n + '#' + table.join(',')` (`sublib_core.js:477-517`). `classSize` counts the distinct standardizations over all base points (`sublib_core.js:373-384`).
- `subgroupsData(spec)` wraps the search and returns, for each subgroup, `{ subgroup, index, generators, cosets, invcos }`, where `cosets` and `invcos` are permutation strings with one character per coset from a 62-character alphabet (`sublib_core.js:530-550` and 674-737).

These are the limits:

| Limit | Default | Location |
| --- | --- | --- |
| `maxIndex` of `subgroupsData` | 24, and a `RangeError` above 62 | `sublib_core.js:677` and 681-683 |
| `maxIndex` of `subgroupClasses` | Required, with no cap | `sublib_core.js:477` |
| `maxTables` | Unlimited. `0` also means unlimited | `sublib_core.js:205` |
| `deadlineMs` | Unlimited. `0` also means unlimited. Checked every 1024 nodes | `sublib_core.js:206` and 300 |
| `maxSubgroups` | 0, no budget. Otherwise the search reruns from scratch for each index from `min(4, maxIndex)` up to `maxIndex` | `sublib_core.js:605-655` and 678 |

A deadline or `maxTables` stops the search with `stats.aborted` set, and the partial classes are returned without an error (`sublib_core.js:300-311`). `subgroupsData` passes only `deadlineMs` and `maxTables` to the search, so the option `prune: false` is reachable only through `subgroupClasses`. Without pruning, `canonicalForm` merges conjugate tables afterwards (`sublib_core.js:490`). The callers in `lib/grouplib/SubgroupTable.js:29-32` set `DEFAULT_MAX_INDEX = 24`, `TABLE_MAX_SUBGROUPS = 2000`, and `TABLE_DEADLINE_MS = 3000`.

`reidemeisterSchreier({ perms, relators })` takes one permutation per generator, counted from 0, and relators as letter strings (`reidemeister.js:44-59`). It builds a breadth-first spanning tree of the cosets, generators before inverses, and throws "coset table is not transitive" if a coset is unreached (`reidemeister.js:66-90`). Each edge outside the tree gives a Schreier generator, with at most 26 (`reidemeister.js:96-109`). It rewrites each relator at each coset, throws if the relator does not return to its coset, and freely reduces the result without removing duplicates (`reidemeister.js:116-141`). Its only caller is `lib/grouplib/SubgroupNames.js:145`. `abelianInvariants` in `sublib_core.js:1205-1254` repeats the same rewriting on exponent vectors and applies a Smith normal form.

The code reading found these defects and conventions:

- Classes are sorted by index and then by `key` as a string, so "1,10" sorts before "1,2", although the comment at `sublib_core.js:475` says "sorted by index, then by the canonical table" (`sublib_core.js:504`). The ids `<name>.<index>.<k>` follow this order.
- `verifyData` indexes `relStrings` by the index into `relators`, so a trivial relator shifts the names in its messages (`sublib_core.js:973-979`).
- The line `if (j < i) return 1` in `scan` cannot be reached, and the guard `e2 !== e1` in `define` is always true (`sublib_core.js:250` and 263), as the code reading inferred.
- The search does not propagate at the root, only after the first definition (`sublib_core.js:230`).
- `standardize` and `schreierGenerators` assume a complete, standardized table and do not test for `-1` (`sublib_core.js:397` and 429-439), as the code reading inferred.
- With the default `generators: 'gap'`, the generator words of the output are written in reversed letter order, to match the GAP exporter that produced the shipped files (`sublib_core.js:526-528` and 570-578).
- Words act left to right, and "generator x sends coset Hg to Hgx" (`sublib_core.js:766-768`, `reidemeister.js:17`), a right action on right cosets.

## 13. Simulation, Animation, Export, and Document Format

This section is non-normative and surveys features that Escher plans for later phases.

### 13.1. Simulation

SymmHub does not solve a PDE on the orbifold. Each simulation runs on a square texture that covers $[-1, 1]^2$ of the world, with $uv = 0.5p + 0.5$, and texture wrap `REPEAT`, so the default boundary is periodic.

The `Gray-Scott` step (`apps/symsim/gray_scott/js/shaders/grayScottShader.glsl.mjs:44-111`) uses a Laplacian of 5 or 9 points, written `0.8*uv_p0 + 0.2*uv_pp - 4*uv` at line 82, and an explicit Euler step of

$$\dot u = d_r \Delta u - u v^2 + F(1 - u), \qquad \dot v = d_g \Delta v + u v^2 - (F + k) v$$

The feed $F$ and the kill rate $k$ can vary with position. The option `useHMetric`, which the code calls the "H-metric", computes a factor $(1 - \varrho^2)^2$ from `vec2 pp = HMetricScale*(pnt*2. - vec2(1.,1.))` and `factor = f*f` (lines 56-61), so $\varrho = \text{HMetricScale} \cdot \lvert 2\,\text{pnt} - (1, 1) \rvert$. Lines 63-64 multiply both diffusion coefficients by it, as `float dr = DiffR*factor; float dg = DiffG*factor;`. Whether this factor implements the Laplace-Beltrami operator of the Poincare disc is open (Section 17). The options `useBoundary` and `useDisk` add a Dirichlet frame one texel wide or a disc boundary, and both are off by default. The buffers are a `DoubleFBO` in `RG32F` of $512 \times 512$ texels by default.

A complex Ginzburg-Landau simulation (`apps/symsim/ginzburg_landau/js/shaders/gl_step.glsl.mjs`) uses a 9-point Laplacian and the same optional factor, computed at lines 34-39 as `float r2 = dot(pp,pp); float ff = max((1. - r2),0.); factor = ff*ff;` with `pp = pnt` and no scale.

Symmetry is imposed by a separate pass (`lib/shaders/symSamplerShader.glsl.mjs:46-62`, run by `lib/symhublib/SymmetrizationWorker.js:100-127`): for each texel, fold its point into $F$, then mix the original value with the value at the folded point by `uSymMix`. The `Gray-Scott` application runs it every `symInterval` steps, 1000 by default, when `symSim` is on, which it is not by default, and once at the end of each batch (`gray_scott_simulation.js:725-755` and `805-835`). The render pass also folds each pixel, so the image is symmetric even when the buffer is not.

### 13.2. Animation

The clock advances by `animTime += animSpeed * dt` (`lib/symhublib/SymRenderer.js:2924-2928`), and a recorder replaces it with a fixed frame time (lines 2943-2948). A script is a module that returns an object with `setTime(t)`, which sets parameters by dotted path, such as `pattern.patternTransform.centerX` (`lib/symhublib/ScriptAPI.js:1-40`, example in `apps/sympix/js/scripts/circular_motion.js:42-52`). The fixed-points Moebius map of Section 10.2 animates by `fraction` and `loopTime`.

### 13.3. Export

`exportImage` (`lib/symhublib/exportImage.js:197`) resizes the canvas to the export size, renders a frame, checks the limits of the graphics context, and writes `<name>.png`, a sidecar document `<name>.png.json`, and a thumbnail `<name>.png.json.png`. `exportTiledImage` (lines 23-196) renders tiles of 2048 pixels by default and streams a tiled TIFF with a hand-written header. Video export writes JPEG frames into JSZip files of up to 100 MB (`lib/uilib/video_recorder3.js`) and has no encoder. `renderToCanvas(w,h)` (`lib/symhublib/SymRenderer.js:3443`) renders to a 2D canvas. All export runs in the browser through WebGL2.

### 13.4. Document Format

A document is JSON of the form `{ "name": ..., "appInfo": {"appName": ..., "fileFormatRelease": 1}, "params": {...} }` (`getParamsAsJSON`, `lib/symhublib/SymRenderer.js:3527-3535`). The group is under `symmetry`, for example `{"className":"Group_KLM","params":{"s":true,"K":4,"L":4,"M":2,"hasR":false}}`, with `symmetry.options.iterations`. The models are under `tools.params.projection`. A binary sidecar with a "SYMM" header holds large data (`BinaryStore.js:1-20`), and files named `*UpgradeData.js` upgrade old formats. The same tree of parameters drives the user interface, documents, undo, and scripts (`lib/uilib/param.js`).

## 14. Group-Theoretic Machinery

### 14.1. Words

A side's word is `ITransform.word`, concatenated by `concat`. `Group.word2trans` turns a word into a transform. `grouplib` exports `parseWords`, `reduceWord`, `expandWord`, `compactWord`, `normalizeWord`, `sameAction`, `involutionLetters`, and `isValidWord` (the export list of `grouplib.js`, about lines 40-60).

### 14.2. Presentations from the Polygon Theorem

`groupPresentation` (`lib/grouplib/GroupPresentation.js:353`) derives a presentation from $F$ and its side pairings, following Poincare's polygon theorem, as the header at lines 14-60 says:

1. Generator $i$ is the side pairing of side $i$.
2. Each pair of paired sides $i$ and $j$ gives the relator $w_i w_j = 1$, and a self-paired side gives $w_i^2 = 1$.
3. The code walks the vertices to find each corner cycle. A cycle with angle sum $2\pi/m$ gives its word to the power $m$ as a relator.
4. A reflection group gets Coxeter relators from the angles between mirrors (`mirrorOrder`, `MAX_MIRROR_ORDER=1000`).
5. The code checks each relator numerically and throws when $F$ does not tile.

Its constants are `EPS=1e-7`, `MAX_CYCLE_LENGTH=100`, `MAX_FOLD_ITERATIONS=200`, and `IDEAL_ANGLE=1e-5`. A comment at line 639 says that domains "twisted so far that the cells overlap ... fail here", and that such a maker derives its presentation from an untwisted copy of itself with `derivedPresentation`. `Group_OrbifoldPresented` uses this.

### 14.3. Dirichlet Domains

`lib/grouplib/DirichletDomain.js`, 1343 lines, builds Dirichlet domains from the Moebius action of the group on the plane, extended to $H^3$, and treats all three geometries alike (header lines 1-60). The base point lies on the plane, as the limit $w \to 0$, so each side is the Apollonius sphere

$$\lVert x - T(a) \rVert^2 = \lvert T'(a) \rvert \, \lVert x - a \rVert^2$$

The code finds the words of the sides by folding sample points into $F$ (`dirichletCellAt`, `selectDirichletSides`). Its constants (lines 79-94) are `EPS=1e-12`, `ARC_EPS=1e-9`, `NEAR_IDENTITY=1e-6`, `NEAR_RELATION=1e-2`, and `DEFAULT_FOLD_ITERATIONS=200`. Related files are `DirichletSearch.js`, `DirichletWorker.js`, `lib/invlib/DirichletDomainBuilder.js`, and `MobiusGenerators.js`, with `TIME_BUDGET = 2` seconds at line 141.

`GeneratorElements.js` classifies each side pairing as a rotation, reflection, glide reflection, or translation from its Moebius action, fitted from three points.

### 14.4. Crowns

`lib/symhublib/CrownCalculator.js`, lines 1-250, finds the tiles around a pattern box with these steps:

1. Fold the points of a spiral grid over the box into $F$ with `toFundDomain`.
2. Walk across the sides of the tiles found so far to find their neighbours.
3. Treat two elements as equal when their images of the base point agree within `epsilon=1e-7`.

Its constants (lines 3-17) are `DEFAULT_GRID_RADIUS=15`, `DEFAULT_MAX_ITERATIONS=20`, `DEFAULT_MAX_COUNT=1000`, `DEFAULT_MIN_SIZE=1e-3`, `CLOUD_SIZE=32`, `DEFAULT_HOME_STEPS=2000`, and `HOME_CHUNK=200`.

The older orbifold version, `getTransformsForTexture` (`lib/orbilib/OrbifoldGeometrization.js:1589`), samples a grid of $45 \times 45 = 2025$ points from `distancetable45.js`, with the comment "For some long skinny FDs this may not be enough". It folds each point with up to 200 iterations, hashes the image of a registration point at an integer scale of $10^5$ (`registryscalingsize=100000`), and keeps each transform not seen before. `calcCrownTransformsDataFromTransform` is at line 1831.

## 15. Constants, Workarounds, and Known Defects

### 15.1. Constants

| Constant | Value | Purpose | Source |
| --- | --- | --- | --- |
| Classification cut-offs | $-0.001$ and $0.01$ | Separate the three geometries by $\chi$ | `lib/orbilib/OrbifoldGeometrization.js:242-251` |
| `LENGTHMINVALUE`, `LENGTHMAXVALUE` | 0.3, 4 | Range of a length | `lib/orbilib/WallPaperGroup_General.js:26-29` |
| Twist range | $-0.5$ to $0.5$ | Range of a twist | `lib/orbilib/WallPaperGroup_General.js:26-29` |
| Initial length | $2 \pm 0.0005$ | Start length with jitter | `lib/orbilib/WallPaperGroup_General.js:266`, `randomOffsetRange` at line 52 |
| `DEFAULT_INCREMENT` | `1e-8` | Slider step | `lib/orbilib/WallPaperGroup_General.js:20` |
| Fallback length | 1.1 | `getLength` for a key index beyond `parCounts` | `lib/orbilib/OrbifoldGeometrization.js:625-627` |
| `recenteringN` | 2 | Recentrings per assembly | `lib/orbilib/OrbifoldGeometrization.js:1351-1368` |
| Domain nudge | $(-0.001, -0.002)$ | General position after assembly | `lib/orbilib/OrbifoldGeometrization.js:1370` |
| `EPSILON` | `1e-13` | General tolerance | `lib/invlib/Utilities.js:281-284` |
| `SHORTEPSILON` | `1e-7` | Line detection, equality of disc maps | `lib/invlib/Utilities.js:281-284` |
| `SHORTEREPSILON` | `1e-4` | Coarse tolerance | `lib/invlib/Utilities.js:281-284` |
| `MAX_RADIUS_SPHERE_SQD` | `Number.MAX_SAFE_INTEGER/32` | Circle to line conversion | `lib/invlib/ComplexArithmetic.js`, used by `splaneIt` at line 537 |
| `MOBIUS_PRECISION`, `PLANAR_EPS` | `1e-6`, `1e-12` | Matrix route | `lib/invlib/MobiusMatrix.js:225` |
| Triangle `EPSILON` | `1.e-6` | Ideal corner and angle-sum tests | `lib/grouplib/Group_KLM.js:29` |
| Triangle `EPSILON2` | `1.e-4` | Degenerate inversion in the ideal triangle | `lib/grouplib/Group_KLM.js:28` |
| Ideal order | Below 2, or 20 and above | An order that gives the angle 0 | `lib/grouplib/Group_KLM.js:318-324` |
| `triSize` | 0.25 | Size of the Euclidean and spherical triangles | `lib/grouplib/Group_KLM.js:332`, 355 |
| `maxIterations` | 20 | Processor fold | `lib/invlib/Group.js:194` |
| `EPS` | `1.e-7` | Shader side test | `lib/shaders/inversiveSampler.glsl.mjs:184`, `lib/shaders/permutations24.glsl.mjs:66` |
| `symIterations` | 10, range 0 to 1000 | Shader fold depth | `lib/symhublib/SymRenderer.js:393`, 1385-1393 |
| Symmetrization iterations | 2 | Fold depth of the symmetrization pass | `lib/symhublib/SymmetrizationWorker.js:70` |
| `MAX_ITERATIONS`, `EPSILON` | 100, `1.e-10` | Older walled fold | `apps/orbifold/js/shaders/inversive.glsl.mjs:15-16`, 39 |
| `MAX_GEN_COUNT`, `MAX_REF_COUNT`, `MAX_CROWN_COUNT` | 4, 4, 20 | Uniform array sizes | `lib/shaders/inversive.glsl.mjs:3-20` |
| `MAX_PERM_COUNT`, `MAX_COLORS_COUNT` | 32, 24 | Colour permutations | `lib/shaders/permutations24.glsl.mjs:8`, 10 |
| `SUB_MAX_SIDES` | 16 | Coset automaton | `lib/shaders/overlay_subgroup.glsl.mjs:36` |
| Band identity threshold | $10^{-4}$ | `band2disc` | `lib/shaders/complex.glsl.mjs:112-125` |
| `uTilingWidth`, `uTilingColor` | 1 pixel, `#000000FF` | Default border width and colour | `lib/symhublib/overlays/OverlayTiling.js:12-31` |
| `DEFAULT_BUDGET_MS` | 6 | View anchor time per update | `lib/grouplib/ViewAnchor.js` |

### 15.2. Numerical Workarounds

SymmHub works around numerical problems in these ways:

- General position: Lengths are jittered and the assembled domain is nudged off the origin, so that no fixed point falls on the origin or on an edge (Section 6.5 and Section 6.6). A note at `lib/invlib/ComplexArithmetic.js:519` says that a reflection close to, but not across, a line, "i.e. across a very large sphere", has "bad numerical problems", and that it is "MUCH BETTER TO AVOID this by ensuring that we are in general position as much as possible".
- Recentring: Each assembly recentres the domain twice, to keep it away from the absolute (Section 6.5).
- Matrix route: Long words use Moebius matrices, because the hyperboloid factorization lost precision (Section 5.4).
- View offset: The shader keeps the pixel's offset from the view centre apart from `u_center`, so the offset stays exact in float however far the view is (`lib/shaders/viewOffset.glsl.mjs:1-13`). A comment at `lib/shaders/overlay_editing.glsl.mjs:13-14` says that "a deep zoom keeps the float's precision of the view's size, where world coordinates near 1 keep 6e-8".
- View anchor: A group element in double precision moves the view near $F$ before the fold in float (Section 9.5).
- Shader tolerance: `EPS` of $10^{-7}$ is at the resolution of a float.

### 15.3. Known Defects and Limitations

| Defect or limitation | Effect | Source |
| --- | --- | --- |
| "2*22 does not get the slice parameter correctly. Perhaps it is instead a corner pair." and "22* as well." | The dispatch catches `22*` as `NN` and `2*22` as `*NN`, both with $n = 2$, so they get the finite groups of order 2 and 4 in the Euclidean plane, and their own functions and parameters are never used (Section 7.1.7) | `apps/orbifold/bug list.txt:3-4`, `lib/orbilib/WallpaperGroups_NonNegative.js:83-91`, 125-126, 837-838 |
| The catalogue registers `parCounts["slice"]=1` for `*2222` but reads the length of `["band",1]`, and registers `band` for `**` but reads `slice` | `getLength` returns its fallback 1.1, so the height of the rectangle is fixed and the slider changes nothing (Section 7.1.3) | `lib/orbilib/WallpaperGroups_NonNegative.js:71`, 127, 835, 839, `lib/orbilib/OrbifoldGeometrization.js:624-630` |
| `iGroup_2222` with a twist $c \ne 0$ | The half-turns of sides 0 and 1 map those sides onto segments that are not sides of the rectangle. By the reader's application of the face condition of the polygon theorems, the pairings fail it (Section 7.1.7 and Section 17, item 15) | `lib/orbilib/WallpaperGroups_NonNegative.js:404-421` |
| The curvature test gives the curvature 0 to a spherical family whose $\chi$ is below 0.01 | `nx` with $n \ge 101$ reaches the Euclidean key `22X`, so `(101)x` gets `iGroup_22X(1, 2)`. The other families with a large $n$ keep their keys with the curvature 0 (Section 7.1.7) | `lib/orbilib/OrbifoldGeometrization.js:242-251`, `lib/orbilib/WallpaperGroups_NonNegative.js:51-56` |
| `iWallpaperGroup` calls `getParam`, which the file does not import | The function would throw, and nothing calls it (Section 7.1.7) | `lib/orbilib/WallpaperGroups_NonNegative.js:1-13`, 780-819 |
| The swaps that put the corners of a sheet first reverse the order of its features relative to the other sheets | The welded polygon has the corner orders in another cyclic order, so it has another signature, such as `*25634` for `*23456`. Section 6.8 states in which signatures the port of Section 6.9 found it, with four to eight corners and the orders it tried. In that port, reordering the features of a sheet only by a cyclic rotation removed the defect on every signature tried | `lib/orbilib/OrbifoldGeometrization.js:538-543`, 553-567 |
| The pairing of corners of order 2 does not wrap around | `*2233` gets a corner pair and `*2332` a slice, so one orbifold gets two sets of parameters | `lib/orbilib/OrbifoldGeometrization.js:399-429` |
| No sheet computes its last side or checks its closure, and no line checks that a sheet exists for its lengths | In the port of Section 6.9, no length in 0.3 to 4 made a call of `edgeLength` undefined or gave a `NaN`. A few runs failed the geometry check of the port, which the report of the pass attributes to the loss of precision near the boundary of the disc, where the error of the angles grows to about $10^{-5}$ radians | `lib/orbilib/OrbifoldGeometrization.js:676-763` |
| The comment of `poincareTurtleMove` says "facing away from last" | The code turns from the direction towards `last`, so the angle is the interior angle (Section 6.4.1) | `lib/invlib/ComplexArithmetic.js:457-466` |
| `n` and `*n` are not implemented | Signatures rejected | `lib/orbilib/OrbifoldGeometrization.js:187-191`, 203-211 |
| `*` alone and every signature with a handle or a cross-cap skip the screen of small signatures | No check for bad orbifolds among them | `lib/orbilib/OrbifoldGeometrization.js:173`, 186-193 |
| `willOrbifoldFitQ` always returns false | A signature too large for the shader limits does nothing, without an error | `lib/orbilib/OrbifoldGeometrization.js:1555`, `lib/orbilib/WallPaperGroup_General.js:240-248` |
| `iGroup_2222_` misses one parameter | Marked "wrong" at line 381. Nothing calls it, and `iGroup_2222` at lines 404-421 replaces it | `lib/orbilib/WallpaperGroups_NonNegative.js:381-402` |
| `sPlanesMovingEdge1ToEdge2` | "not correct if p1=q1" | `lib/invlib/ComplexArithmetic.js:787` |
| `poincareMobiusFromSplane` | Assumes the splane preserves the unit disc. "REWORK-- THIS WILL NOT WORK IN GENERAL" | `lib/invlib/ComplexArithmetic.js:820` |
| `poincareMobiusTranslateFromTo` | "doesn't have the right framing" | `lib/invlib/ComplexArithmetic.js:435-445` |
| `this.ends = ends` refers to an undefined variable | Throws for any `iPlane` built with ends | `lib/invlib/ISplane.js:25` |
| `new iSplane([1,0,0,0],2)` | Wrong constructor call, because the constructor takes an options object | `lib/invlib/Inversive.js:1255` |
| `iToFundDomainWBounds` ignores walls | Assumes $F$ is convex | `lib/invlib/Inversive.js:1245-1249` |
| Case map of words stops before `Z` | A 26th generator letter is broken | `lib/invlib/GroupUtils.js:193` |
| A non-digit inside parentheses sets `errorMessage` but leaves `noErrorQ` true | Parsing continues, the screen at line 173 still runs, and `eulerchar` is `NaN` | `lib/orbilib/OrbifoldGeometrization.js:116-119`, 226 |
| Comment `// 0 - sphere, 1 plane` | Disagrees with the defines in the same file, which give sphere 1 and plane 2 | `lib/shaders/isplane.glsl.mjs:3-7`, 12 |
| Observation: the `Group_KLM` constructor reads `arg.k` for all three orders, as `K: getParam(arg.k, 3), L: getParam(arg.k, 4), M: getParam(arg.k, 3)` | `L` and `M` ignore `arg.l` and `arg.m` | `lib/grouplib/Group_KLM.js:41-43` |
| Observation: `getGroup_KLM` builds `new Group({s: fd, t: trans})` before it pushes the `hasR` side | Whether the group sees the extra side depends on whether `Group` copies the arrays, which no code reading checked | `lib/grouplib/Group_KLM.js:246`, 248-253 |
| `generateGroup`, `generateGroupFromSplanes` | Dead code that refers to an undefined `globalMobTransform` | `lib/orbilib/OrbifoldGeometrization.js:1499-1553` |
| `cSin`, `cCos` call `cInverse(c)` | Wrong argument in unused code | `lib/shaders/complex.glsl.mjs:62-74` |
| Colour blending near $(1,1,1,0)$ | `vec4` values "composite weirdly" | `apps/orbifold/js/shaders/weird frag bug.txt` |
| `setShift` anchoring | Assumes the navigator preserves the absolute | `lib/orbilib/OrbifoldUITool.js:34-40` |
| `Group_ON` side S1 appears twice | bend2 and bend3 are "untested beyond the browser" | `lib/grouplib/Group_ON.js` header |
| Group data in one texture row | Bounded by `MAX_TEXTURE_SIZE` with no guard | `lib/invlib/DataPacking.js:188-193` |
| Float texture filtering | Falls back to nearest filtering where `OES_texture_float_linear` is missing, and otherwise "every sample reads zero ... no error reported" | `lib/symhublib/webgl_utils.js`, about lines 715-745 |
| Spherical view | Only one hemisphere is visible, and the scale uses one factor $1/z$ (Section 10.2) | `lib/shaders/projection_v2.glsl.mjs:250-266` |
| Export size | Checks `MAX_VIEWPORT_DIMS`, a lost context, and memory, with tiled TIFF as the workaround | `lib/symhublib/exportImage.js`, about lines 215-250 |

### 15.4. Tests

The tests are Node scripts and browser pages, and none runs a shader automatically. `tests/groups/view_anchor.mjs` folds points as the shaders do, with colour permutations, for the groups 632, 333, and the hyperbolic 237, and compares the result with the anchored fold. `tests/symhublib/view_chain.mjs` checks each projection formula and the order of the chain. Other scripts cover Dirichlet domains, presentations, subgroups, transforms, and drawing geometry. `tests/splanes/splanes.html` renders the splane shaders in a browser and is the closest thing to a shader unit test. No test is specific to the orbifold construction.

## 16. What SymmHub Lacks for Escher's Proof of Concept

Escher's first deliverable is a command-line tool that renders the tiling of a signature to a PNG file on the processor, with flat-coloured tiles, black borders of configurable thickness, several models of the hyperbolic plane, and Phong-shaded spheres. The code readings found these gaps:

- SymmHub has no renderer on the processor and writes PNG files only from the browser canvas. The processor fold of Section 9.1 and the chain of Section 10.3 exist, but not a pixel loop around them.
- Euclidean and spherical signatures come from a fixed catalogue, not from the general construction (Section 7.1).
- The signatures `n` and `*n` are not supported (Section 4.3).
- No hyperbolic orbifold is built without the atom-and-weld pipeline, and no closed-form Fricke polygon or map from Fenchel-Nielsen coordinates to matrices exists. The only moduli are the lengths and twists of Section 6.6.
- No Todd-Coxeter coset enumeration exists. `lib/sublib` holds only the low-index subgroup search and Reidemeister-Schreier rewriting (Section 12.8).
- No input by Wythoff construction or Coxeter diagram exists, and no form of signature beyond Conway's notation.
- The shaders carry no word, parity, or tile identifier, so colour comes only from a permutation of at most 24 colours (Section 12).
- Border widths are Euclidean in the model, scaled to pixels. No border has a width in hyperbolic distance.
- The Klein model and the hyperboloid model are not rendering models (Section 10.3).
- No supersampling, multisample antialiasing, or jittered sampling exists. Antialiasing is analytic with one sample (Section 11.3).
- No lit sphere exists. A search of every `*.js`, `*.mjs`, `*.glsl`, and `*.html` file for `phong`, `lambert`, `specular`, `diffuse`, `blinn`, `lightDir`, `shininess`, `u_light`, `ambient`, and the patterns `sphere.*(light|shad)` and `(light|shad).*sphere` found lighting in two places only, and both light a height field, not a sphere:
  - `lib/shaders/bufferToScreenBumpmap.glsl.mjs` lights a bump map. Its normal comes from the gradient of the height field (line 138), with the plane normal `vec3(0,0,-1)` (line 136), a light at `vec3(-0.3, .2, -2)` (line 140), a diffuse term (line 149), and a specular term with exponent 16 (line 154). It calls `makeSphericalProjection` only for the disc cut-out and the height scale (lines 120-124). `lib/symhublib/VisualizationBumpmap.js` and `lib/symhublib/SymRendererPrograms.js` use it.
  - `apps/symsim/ginzburg_landau/js/shaders/render_inc.glsl.mjs:90-119` lights the simulation field as a height surface, with `#define ambient 0.1` and `#define shininess 128.`.

  The spherical view itself, `makeSphericalProjection` (Section 10.2), has no lighting. This rests on a search, not on a full read (Section 17, item 8).
- No vector export of a tiling exists. A search of every `*.js`, `*.mjs`, `*.html`, and `*.json` file for `svg`, `postscript`, `%!PS`, `.eps`, `jspdf`, `pdfkit`, `.pdf`, `toSVG`, and `exportSVG` found only icon and asset file names, icon links in `readme.html` and `outline.html`, and `lib/drawlib/ImageItems.js`, which reads an SVG blob as an input image. The image outputs found are PNG through `toBlob(resolve, 'image/png')` (`lib/symhublib/exportImage.js:39`, 305), frames through `canvas.toBlob` in a zip (`lib/invlib/AnimationControl.js:99-131`), and download helpers (`lib/invlib/Utilities.js:393`, 407, 431). The search did not cover TIFF or JPEG encoders, so the TIFF and JPEG output of Section 13.3 rests on the first code readings only.
- No PDE solver runs on the orbifold, and no boundary condition follows the side pairings (Section 13.1).
- No automated test covers the orbifold construction or a shader.

## 17. Open Questions and Unverified Claims

These questions and claims are open:

1. Licence compatibility: No cited source states whether code under the MIT licence can be part of a project under GPL-3.0-or-later (Section 3.3). This is a legal question that the code cannot settle.
2. Spherical projection: The code multiplies the point by $1/(1 + z)$ and the scale by $1/z$, with $z = \sqrt{1 - \lvert p \rvert^2}$ (Section 10.2). A first code reading described the map as the stereographic image of the orthographic view of the front hemisphere. The same reading derived, as its own inference, an exact radial factor of $1/(z(1+z))$ and a tangential factor of $1/(1+z)$. The code states neither the description nor the derivation, the derived radial factor does not match the code's single scale factor, and no cited source establishes the exact factors.
3. Diffusion factor of the simulation: The code multiplies the diffusion coefficients by $(1 - \varrho^2)^2$ (Section 13.1). Neither the code nor a comment states that this implements the Laplace-Beltrami operator of the Poincare disc, and no cited source establishes it.
4. Fenchel-Nielsen coordinates: Only the tutorial claims that the length and twist sliders are Fenchel-Nielsen coordinates (`tutorials/orbifold-notation.html:285`, 293, 325, and 382). No JavaScript file mentions Fenchel-Nielsen, and nothing proves the claim.
5. Termination of the fold: No cited source proves that the fold of Section 9 terminates for a given domain, or bounds its number of iterations. The comment in `lib/grouplib/Group_ON.js` asserts convergence, and the code bounds the loop only by the iteration cap (Section 9.3).
6. Deaton's thesis: Closed. The three files that cite it disagree on the first name and the year (Section 3.5), and the title page settles them as Adam A. Deaton and 1993 ([DEATON1993]). The item keeps its number so that citations of the later items stay valid.
7. Unread sources: The papers in Section 3.5 other than [DEATON1993] are not in the repository and were not read for this document. Every statement about them is what SymmHub's tutorial says. [DEATON1993] was read for Section 8.15 of `docs/research/fundamental-domains.md`.
8. Unread code: No code reading read `GroupGeometry.js`, `LimitSet.js`, or the `Subgroup*.js` files of `lib/grouplib` line by line. In `lib/sublib`, `src/sublib_core.js` and `src/reidemeister.js` were read line by line (Section 12.8), `src/sublib.js`, `src/groups_description.js`, and `src/modules.js` only for their imports, exports, and the lines cited, and `README.md` only for its section `Compatibility with the GAP-generated files` (item 10). `docs/research/prior-art.md` cites that README under the tag `SYMMHUB-SUBLIB`. For the kaleidoscopes of Sections 6.2.1 to 6.8, `lib/orbilib/OrbifoldGeometrization.js` was read line by line at lines 46-53, 147-165, 266-431, 452-764, 1136-1382, and 1387-1493, and `lib/invlib/ComplexArithmetic.js` at lines 206-239 and 404-491. The pillow and hat cases at lines 766-1133 of the first file were read only at the level of Section 6.4. For the catalogue of Section 7.1, `lib/orbilib/WallpaperGroups_NonNegative.js` was read line by line in full, with the lines of the other files that Section 7.1.8 lists. `lib/grouplib/WallpaperGroups.js` and `lib/grouplib/Group_Spherical.js`, the files of the makers `Group_WP` and `Group_Spherical`, which take the Euclidean and spherical groups by name (Section 4.6), were not read, so it is not known what domains they build. Section 16 states the absence of a lit sphere and of vector export as the result of a keyword search, not of a full read.
9. Candidate statements about the triangles and the half-plane map: The source review derived these statements by algebra, and neither the code nor its comments state them:
   - In the hyperbolic triangle, $\lvert c \rvert^2 = 1 + r^2$ for the centre $c$ of $s_2$, so $s_2$ is orthogonal to the unit circle.
   - In the ideal triangle, the circle $s_1$ with centre $(-1, 1)$ and radius 1 touches the $x$ axis at $(-1, 0)$, which is the ideal $K$ corner.
   - The map `uhp2disc` equals $w = (i - z)/(i + z)$, which is the Cayley map $(z - i)/(z + i)$ multiplied by $-1$. Its scale factor $2/D$ equals $\lvert dw/dz \rvert = 2/\lvert z + i \rvert^2$.
   - In the Euclidean triangle, $L = (e/\sin a_1, 0)$ and $M = (e/\sin a_2)(\cos a_0, \sin a_0)$, with $a_2 = \pi - a_0 - a_1$.
   - In the spherical triangle, the circle $s_2$ passes through $(e_2, 0)$, so $e_2$ is the $x$ coordinate of the corner $L$. For $(K, L, M) = (2, 3, 3)$ the review computed the centre as about $(-0.341, -0.341)$ and the radius as about 0.683.
   - In the spherical triangle, the denominator $\cos a_2 + \cos(a_1 + a_0)$ is negative because the angle sum exceeds $\pi$, so $r < 0$ and the domain is inside the circle.
   - In the ideal triangle, the inversion in the circle with centre $(-1, 0)$ and radius 2 maps the line $x = 1$ to the unit circle, so the absolute is again the unit circle.
10. Claims of the `sublib` comments:
    - The comment at `lib/sublib/src/sublib_core.js:21-27` calls the table conventions "identical to GAP's".
    - The comment at lines 31-33 says that the representative of a class agrees with GAP's for "~99%" of subgroups. The section `Compatibility with the GAP-generated files` of `lib/sublib/README.md` (line 297), which the comment names and which was read for this item only, says "97%" (line 310).
    - The comment at lines 341-346 says that a prefix of a partial table that loses "still loses however it is completed, so cutting the branch there cannot discard a canonical table".

    The research checked none of these. Item 24 of Section 14.1 of `docs/research/colourings.md` records what would close the first two, and its item 22 the third.
11. Closure of a sheet: No case of Section 6.4.1 computes the last side of its sheet, so the angle at the origin, the right angles at the ends of each imaginary side, and the collinear edges at each welded slice (Section 6.5.1) hold only if the formulas of Section 6.4.2 are right. Gap G3 and Section 13.3, items 1 and 13, of `docs/research/fundamental-domains.md` record what would close it. The port of Section 6.9 measured the closure of the sheets for the signatures and lengths that it lists, which is numeric evidence and no source. [DEATON1993] states that each piece can be built because it has a positive angle defect, and that a boundary of a cut meets the mirrors at right angles, but gives no formula for the last side and does not state the collinear edges (Section 8.15 of `docs/research/fundamental-domains.md`).
12. Hand traces: The lists, sheets, polygons, and parameters of Sections 6.2.1, 6.3.1, 6.5.1, 6.6.1, and 6.8, and the defect of the swaps in Section 15.3, rest on traces of the code by hand and on the port of Section 6.9, which agrees with every trace. The SymmHub code itself was not run.
13. Inferred geometry of the maps: The orientation of the turtle and its turn (Section 6.4.1) and the endpoint correspondence of `poincareMobiusEdgeToEdge` (Section 6.5.1) are the code reading's reading of compositions of Moebius maps, which no comment states. The port of Section 6.9 agrees with each, which is numeric evidence and no source.
14. Hand traces of the catalogue: The keys of Section 7.1.1, the domains, pairings, and vertex cycles of Sections 7.1.4 and 7.1.5, and every number of Section 7.1.6 rest on traces of the code by hand and on the port of Section 7.1.9, which agrees with every trace. The SymmHub code itself was not run. The agreement holds only for the signatures and parameter values that Section 7.1.9 lists. The port did not cover the fold on the graphics card or the borders.
15. Inferences of the catalogue reading: The names of the products in Sections 7.1.4 and 7.1.5, such as rotation, glide reflection, and half-turn about an axis, are the reader's names for the maps that the traces give, and no comment states them. The statements below are also inferences that no comment or cited source states. Where the port of Section 7.1.9 bears on one, the bullet says so. That evidence is numeric and is no source or proof:
    - A line through the origin of the stereographic plane is a great circle through both poles, a wedge from the origin to infinity is a lune, and a wedge inside the unit circle is a triangle with a vertex at the pole and two on the equator (Section 7.1.5). Section 11.2 of `docs/research/geometry-models.md` states that the projection is conformal and maps spheres to planes or spheres, but not these cases.
    - Each circle of a polyhedral group, with $r^2 = \lvert c \rvert^2 + 1$, is the stereographic image of a great circle (Section 7.1.5). The port measured the relation to the bound that Section 7.1.9 gives.
    - The pairing of `x` is the antipodal map of the sphere (Section 7.1.6). The port found its matrix to be minus the identity (Section 7.1.9).
    - The four sides of `iGroup_2222` with $c \ne 0$ fail Condition 3.2 (b) of the polygon theorem, and its corners form no vertex cycles (Section 7.1.7). This applies the condition to the traced maps. The port confirmed the image of side 0 only.
    - The rectangle of `iGroup_2222` with $c \ne 0$ is a fundamental domain of the group that its four pairings generate, in a tiling that is not edge to edge. Section 7.1.9 gives the port's result at the settings it tried.
    - Each domain that the catalogue builds for the signatures of Section 7.1.9 is a fundamental domain of the group of its signature, except those of `22*` and `2*22`, at the settings that Section 7.1.9 lists.
    - The dispatch of `22*` and `2*22` is the cause of the two entries of `apps/orbifold/bug list.txt:3-4` (Section 7.1.7). The note does not name the dispatch.
    - `22*` and `2*22` reach `iGroup_N(2)` and `iGroup_SN(2)`, and `nx` with $n \ge 101$ reaches `iGroup_22X`, by the order of the branches of Section 7.1.1 and the curvature test of Section 4.4. The port confirmed `22*`, `2*22`, `(100)x`, and `(101)x`.

## 18. References

- [SYMMHUB] Bulatov, V., Goodman-Strauss, C., and S. Vorthmann, "SymmHub", source code at commit `f2d19cb`, MIT licence, 2025, <https://github.com/SymmHub/SymmHub>.
- [CONWAY1992] Conway, J. H., "The orbifold notation for surface groups", Groups, Combinatorics and Geometry, London Mathematical Society Lecture Note Series 165, pp. 438-447, 1992. Cited by SymmHub, not read.
- [CONWAY-HUSON2002] Conway, J. H. and D. H. Huson, "The orbifold notation for two-dimensional groups", Structural Chemistry 13, p. 247, 2002, DOI 10.1023/A:1015851621002. Cited by SymmHub, not read.
- [SOT2008] Conway, J. H., Burgiel, H., and C. Goodman-Strauss, "The Symmetries of Things", 2008. Cited by SymmHub, not read for this document.
- [THURSTON-GT3M] Thurston, W. P., "The Geometry and Topology of Three-Manifolds", chapter 13, <https://library.slmath.org/nonmsri/gt3m/PDF/13.pdf>. Cited by SymmHub, not read.
- [MACBEATH1967] Macbeath, A. M., Canadian Journal of Mathematics 19, p. 1192, 1967. Cited by SymmHub, not read for this document.
- [CDGT2018] Conway, J. H., Doyle, P., Gilman, J., and W. Thurston, "A field guide to the orbifolds", arXiv:1804.03055, <https://arxiv.org/abs/1804.03055>. Cited by SymmHub, not read.
- [DEATON1993] Deaton, A. A., "Hyperbolic Orbifolds and Patterns", senior thesis, Princeton University, 1993, unpublished. Cited by SymmHub. Read in full for `docs/research/fundamental-domains.md`, whose Section 14.1 gives the full entry.

## Appendix A. Key Files

This table lists the files that an implementer reads first, in the order of the pipeline.

| File | Content |
| --- | --- |
| `lib/orbilib/OrbifoldGeometrization.js` | Parser (77-262), screen of small signatures (173-221), Euler characteristic (226-237), curvature test (242-251), `countParameters` (266-277), `hashOrbifold` (279), `atomizeOrbifold` (452), `edgeLength` (588), `getLength` (624), `useMirrorQ` (639), `unfoldAtom` (648), `assembleFundamentalDomain` (1195), `produceGenerators` (1387), crown sampling (1589) |
| `lib/orbilib/WallPaperGroup_General.js` | Orbifold group wrapper, parameter ranges, packaging of $F$ |
| `lib/orbilib/WallpaperGroups_NonNegative.js` | Euclidean and spherical catalogue: dispatch `nonNegHashOrbifold` (29-132), Euclidean functions (202-591), spherical functions (596-776), `getNonnegativeGroupData` (823-878) |
| `lib/invlib/ComplexArithmetic.js` | Disc Moebius class, turtle moves (459), the map of one edge onto another (472), splane helpers |
| `lib/invlib/ISplane.js`, `lib/invlib/ITransform.js`, `lib/invlib/Inversive.js` | Splanes, transforms, reflection, and distance |
| `lib/invlib/Group.js`, `lib/invlib/GroupUtils.js` | `Group`, `toFundDomain`, words, and inverses |
| `lib/invlib/MobiusMatrix.js` | Matrix classification and `mobiusToSplanes` |
| `lib/invlib/DataPacking.js` | Group to texture encoding |
| `lib/grouplib/GroupMakerFactory.js` | Menu of group makers (37-57) |
| `lib/grouplib/Group_KLM.js` | Triangle groups: `getGroup_sKLM` (193-223), `getGroup_KLM` (228-258), `makeTriangle` (292-312), `getAngle` (318-324), the four builders (332-436) |
| `lib/grouplib/ViewAnchor.js` | View anchor |
| `lib/grouplib/GroupPresentation.js` | Presentations from the polygon theorem |
| `lib/grouplib/DirichletDomain.js` | Dirichlet domains |
| `lib/shaders/isplane.glsl.mjs` | Splane type defines (3-7), shader side test (26-45), and reflection with scale (114-133) |
| `lib/shaders/inversiveSampler.glsl.mjs` | Shader fold loop and `applyAnchor` |
| `lib/shaders/permutations24.glsl.mjs` | Fold with colour permutations |
| `lib/shaders/projection_v2.glsl.mjs`, `lib/shaders/complex.glsl.mjs` | Projection chain and complex helpers |
| `lib/shaders/colorTiles.glsl.mjs`, `lib/shaders/overlay_tiling.glsl.mjs` | Flat colour tiles and borders |
| `lib/symhublib/ViewChain.js` | Processor mirror of the projection chain |
| `lib/symhublib/CrownCalculator.js` | Tiles around a box |
| `lib/sublib/src/sublib_core.js` | Low-index subgroup search: `LowIndexSearch` (199-331), `isCanonical` (351-371), `subgroupClasses` (477-517), `subgroupsData` (674-737) |
| `lib/sublib/src/reidemeister.js` | Reidemeister-Schreier rewriting: `reidemeisterSchreier` (57-149) |
| `tutorials/orbifold-notation.html` | Prose description of the orbifold algorithm |
