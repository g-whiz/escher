# Vector Export, Animation, and Shader Export Survey

| Field | Value |
| --- | --- |
| Status | Snapshot |
| Date | 2026-11-16 |
| Applies to | Roadmap planning for vector export, animation, and shader export in Escher |

## Abstract

This document surveys three later goals of Escher: exporting a tiling as vector graphics, animating a tiling, and exporting a tiling as a self-contained shader. For vector export it covers how geodesics map onto SVG and PDF path primitives in each model, the error of Bezier approximation, how existing tools decide how many tiles to emit, and how to enumerate each tile once. For animation it covers isometries applied to the view, deformations of the group, morphs between models and between geometries, and the export of frame sequences as video. For shader export it covers what a tiling shader needs, Slang as an export target, and the alternative languages and translation paths. Each part ends with a table of difficulty ratings, which are the researcher's own judgement, and with its main unknown. The document closes with a reading order, the conflicts between sources, and the open questions that a deeper research pass has to close.

## 1. Scope

This document is a survey for the people and agents who write Escher's roadmap. It shows the candidate approaches for each goal, how hard each one looks, what existing tools do, and what to read first. It is not an implementation reference, and it makes no recommendation about what Escher should build. A deeper research pass on each of the three parts is itself a future roadmap item, and each of those passes would produce a subject-matter reference that prevails over this survey.

This document leaves these topics to other documents:

- The fold loop (Section 4.1), forward generation with Dunham's algorithm (Section 5.1), and the per-pixel model maps (Section 8) are in `docs/research/rendering.md`.
- The models of the hyperbolic plane (Section 3), their isometries (Section 6), and the band map with its constant (Section 5.1) are in `docs/research/geometry-models.md`.
- The Rust crates for rasterization, SVG and PDF output, Slint, and wgpu are in `docs/research/rust-ecosystem.md`, Sections 13 and 16.

## 2. Notation and Conventions

### 2.1. Symbols

- $z$: A point of the Poincare disk, as a complex number with $\lvert z \rvert < 1$.
- $w$: A point of the band model.
- $\theta$: The angle that a circular arc subtends at its centre.
- $R$: The radius of a circular arc.
- $h$: The handle length of a cubic Bezier curve that approximates an arc.
- $g$: A hyperbolic element of $SL(2, \mathbb{R})$.
- $L$: The translation length of a hyperbolic isometry.
- $\rho$: The hyperbolic distance.
- $A$: The axis of a hyperbolic isometry.
- $\alpha$: The distance of a translation in the hyperboloid model.
- $X$: An element of the Lie algebra $\mathfrak{sl}(2, \mathbb{R})$.
- $t$, $s$: Real parameters of a one-parameter subgroup.
- $\kappa$: The sectional curvature in the $\kappa$-stereographic model.

### 2.2. Terminology

- arc command: The SVG path command `A`, which draws an elliptical arc between two endpoints.
- cubic Bezier curve: A curve given by two endpoints and two control points, the only curve primitive of a PDF path.
- one-parameter subgroup: A continuous homomorphism $t \mapsto e^{tX}$ from the real numbers into a matrix group.
- translation length: The distance by which a hyperbolic isometry moves each point of its axis.
- frame sequence: A set of numbered image files, one per frame, that a video encoder reads.
- self-contained shader: A single shader source file that draws a tiling with its group data baked in or passed as constants.
- link-time specialization: The Slang feature that resolves constants and types declared `extern` in one module from an `export` in another module when the modules are linked.
- specialization constant: A SPIR-V constant whose value the application sets when it creates the pipeline.
- difficulty rating: The researcher's judgement of how hard an item is, on the scale low, medium, and high. No source gives these ratings.

### 2.3. Source Conventions

One research pass gathered every fact in this document, and it accessed every source on 2026-11-15. The exceptions are [BEARDON] and [DUNHAM-1986], which were read on 2026-11-16 from copies that the owner supplied, and which Sections 3.4, 3.6, 4.1, and 4.2 cite with printed page numbers. The pass ran out of web searches early, so coverage is thin where a section says so. Most quotations passed through a summarizing fetch tool and may not match the page character for character. The research rated its overall confidence as medium.

Each reference entry in Section 9 carries the mark that the research gave its source: primary, secondary, practitioner, academic, or not marked. An entry says "year not recorded" where the research did not record a year. Secondary sources include Wikipedia, blogs, mirrors, product pages, and search snippets. In the body these marks apply:

- (secondary): The claim rests only on a secondary source.
- (inferred): The research concluded the claim from other facts, and no source states it.
- (unverified): The research could not confirm the claim.

Names with accented letters are transliterated to ASCII, such as Poincare, Mobius, Bezier, Teichmuller, Daehlen, Becigneul, and Riskus, because the prose checker rejects non-ASCII characters. A URL or title that contains a spelling that the prose checker rejects appears in a code span.

## 3. Vector Export

### 3.1. Geodesics in Each Model

A vector exporter draws each tile edge as a geodesic of the chosen model. This table gives the shape of a geodesic in each model and the path primitive that can draw it:

| Model or geometry | Shape of a geodesic | Vector primitive | Source |
| --- | --- | --- | --- |
| Poincare disk | Arc of a circle orthogonal to the boundary, or a diameter | SVG arc command, or cubic Bezier curves in PDF | [W-DISK] (secondary), [BULATOV-2010] |
| Upper half-plane | Semicircle centred on the real axis, or a vertical ray | SVG arc command or line | [W-HALF-PLANE] (secondary) |
| Klein model, Euclidean plane | Straight chord or segment | SVG `L`, PDF `l` | [W-KLEIN] (secondary) |
| Sphere, stereographic projection | Circle, or a line when the circle passes through the projection point | Arc or line | [W-STEREO] (secondary). The page states this for circles in general, not for great circles |
| Sphere, orthographic projection | Ellipse, or a segment when seen edge-on | SVG arc command with unequal radii | [W-PARALLEL] (secondary). No source applies this to great circles |
| Band model | Not an arc in general | Polyline or fitted Bezier curves | [BULATOV-2010], [W-BAND] (secondary). No source states that the curves are not circles |

[W-BAND] says that "every end of a geodesic either meets a boundary of the band at a right angle or is asymptotic to the midline". Section 5.1 of `docs/research/geometry-models.md` settles the band map and its constant, so that document prevails over [BULATOV-2010] (Section 7).

### 3.2. The SVG Arc Command

The arc command takes the current point and the parameters `rx ry x-axis-rotation large-arc-flag sweep-flag x y`, and the renderer computes the centre [SVG11-PATHS] Section 8.3.8, [SVG11-IMPL] Appendix F.6. SVG 2 repeats this material [SVG2-IMPL] Appendix B.2.

Out-of-range parameters are handled leniently [SVG11-IMPL] Appendix F.6:

- "If rx = 0 or ry = 0 then this arc is treated as a straight line segment."
- When the radius is too small for the endpoints, "the ellipse is scaled up uniformly until there is exactly one solution."
- The stated reason is rounding: "The inevitable approximations arising from computer arithmetic cannot cause a valid set of values ... to be treated as invalid." The appendix names "a semicircular arc" as a common boundary case.

SVG 2 Appendix B.3, which is informative, says: "The real number precision of SVG is single-precision." It also asks generators to use "at least double-precision" when they build the current transformation matrix (CTM) [SVG2-IMPL].

### 3.3. PDF Paths and Bezier Approximation

A PDF path consists of "straight lines, rectangles, and cubic Bezier curves" [ISO32000-MIRROR] Section 8.5.2 (secondary). PDF has no arc operator (inferred from that operator list). A PDF exporter therefore approximates every circular arc with cubic Bezier curves.

The cairo library splits an arc into pieces and gives each piece the handle length and maximum error below, measured in units of the squared radius and citing Dokken and Daehlen (1990) and Goldapp (1991) [CAIRO-ARC] (secondary):

$$h = \frac{4}{3} R \tan\frac{\theta}{4}, \qquad e_{\max} = \frac{4}{27} \cdot \frac{\sin^6(\theta/4)}{\cos^2(\theta/4)}$$

For a quarter circle the radial error is about $2.7 \times 10^{-4}$ of the radius, and the error grows as $O(\theta^6)$ [RAW-BEZIER] (secondary), [POMAX] (secondary). [MORTENSEN] (secondary) covers the same ground, but its numbers did not come through extraction.

A curve that is not an arc, such as a band-model geodesic, needs curve fitting. Levien states that "subdividing the curve in half reduces the error by a factor of 64" for a fitted cubic Bezier curve [LEVIEN-FIT]. His earlier article derives the number of segments from an error tolerance when a quadratic Bezier curve is flattened [LEVIEN-FLAT] (secondary).

### 3.4. How Many Tiles to Emit

A hyperbolic tiling has infinitely many tiles, so a vector exporter has to stop somewhere. Section 3.5 gives the stopping criterion of each tool that the research examined. Dunham's replication algorithm stops after a given number of layers, and its layer count `nLayers` is its only stopping parameter [DUNHAM-1986] pp. 146-147. No source gives a rule based on size or distance (Section 8.1, item 1).

### 3.5. Existing Tools

| Tool | Vector output | Stopping criterion | Notes | Source |
| --- | --- | --- | --- | --- |
| HyperRogue 12.1q | SVG through `-svgshot`. Each shape is a `<path>` of `M` and `L` commands, with no arcs | Off-screen polygons skipped. The limit of its walk over cells was not traced | Coordinates rounded to `%.1f` or `%.2f` according to `divby` | [HYPERROGUE] `screenshot.cpp` |
| hypertiling (Python, MIT) | SVG through `make_svg` and `write_svg`, in which "edges are drawn as geodesics" | Number of layers | Neighbour search "risks becoming inaccurate close to the unit circle" | [HYPERTILING] |
| cduck/hyperbolic (Python) | SVG through drawsvg, in the Poincare disk and the half-plane | Unverified | The tiling source could not be read | [CDUCK] |
| Hatch, HyperbolicApplet | EPS file `PSDUMP.eps`, from local runs only | "max isometries (how much to draw)" | Quality 0 draws straight segments, and quality 2 draws "properly curved segments" | [HATCH] |
| KaleidoTile (J. Weeks) | None found. An App Store snippet says PNG only (secondary) | Not documented | | [KALEIDOTILE] |
| Honeycombs (R. Nelson) | POV-Ray and STL, in 3D only | Edge resolution | | [HONEYCOMBS] |

The research did not check Tess, Kali, EscherSketch, Inkscape extensions, Bulatov's software, GeoGebra, or Wolfram demonstrations.

### 3.6. Enumerating Tiles Without Duplicates

An exporter that writes each tile exactly once needs a duplicate-free enumeration. The research found three approaches:

| Approach | Mechanism | Source |
| --- | --- | --- |
| Dunham's replication algorithm | A tree recursion that "closes" vertices, so that "no further copies are generated by rotations around a vertex" | [HYPERTILING] Section 5.4 (secondary for Dunham). The 1986 procedure `Replicate` is printed as Fig. 9 of [DUNHAM-1986] (p. 146), for the groups $[p,q]$ and $[p,q]^+$ with $p > 3$ and $q > 3$ only. The paper says that the algorithm "must be modified slightly" for $[p^+,q]$ and for $p = 3$ or $q = 3$, and it does not print the modification (p. 145). Section 5.1 of `docs/research/rendering.md` states the algorithm |
| Automatic structures | "every finitely generated Coxeter group is an automatic group", after Brink and Howlett 1993 | [W-COXETER] (secondary). Casselman's automata paper is the practical reference and was not read. Section 5.2 of `docs/research/rendering.md` covers automata |
| Hash-based deduplication | The "SR" kernels of hypertiling keep a container of cell centres and discard repeats | [HYPERTILING]. Section 5.4 of `docs/research/rendering.md` covers breadth-first search with deduplication |

Todd-Coxeter coset enumeration terminates only when the index of the subgroup is finite [W-TODD-COXETER] (secondary).

### 3.7. Difficulty of Vector Export

These ratings are the researcher's own judgement:

| Item | Difficulty |
| --- | --- |
| Klein, Euclidean, and stereographic output | Low |
| Disk and half-plane arcs with robust handling of the arc flags | Low to medium |
| PDF Bezier curves with an error bound | Medium |
| Band model and other conformal images, through curve fitting | Medium to high |
| Policy for level of detail and culling | Medium, with no prior art found |

The main unknown is the stopping rule, because no source gives one (Section 8.1, item 1).

## 4. Animation

### 4.1. Isometries Applied to the View

The simplest animation moves the view by a continuous family of isometries while the tiling stays fixed. Mobius transformations fall into elliptic, parabolic, hyperbolic, and loxodromic types, which "can be distinguished by looking at the trace", and the same page describes animating one "by breaking it up into steps" [W-MOBIUS] (secondary). Sections 6.6 and 6.7 of `docs/research/geometry-models.md` state the classification and the translation length from their own sources.

Every element of $SL(2, \mathbb{R})$ is conjugate into one of "3 standard one-parameter subgroups", and $PSL(2, \mathbb{R}) \cong SO^+(2,1)$ [W-SL2R] (secondary). In the hyperboloid model a translation by a distance $\alpha$ is a Lorentz boost whose entries are $\cosh\alpha$ and $\sinh\alpha$ [W-HYPERBOLOID] (secondary). The translation length $L$ of a hyperbolic element $g$ satisfies [W-FUCHSIAN] (secondary):

$$\lvert \operatorname{tr} g \rvert = 2 \cosh\frac{L}{2}$$

Beardon states the classification for Mobius transformations, and this relation for orientation-preserving isometries of the hyperbolic plane. He normalizes the trace as $\operatorname{trace}^2(g) = \operatorname{tr}^2(M)/\det(M)$ for any matrix $M$ that projects to $g$ [BEARDON] (4.2.1), p. 61. An element $g \ne I$ is parabolic if and only if $\operatorname{trace}^2(g) = 4$, elliptic if and only if $\operatorname{trace}^2(g) \in [0, 4)$, hyperbolic if and only if $\operatorname{trace}^2(g) \in (4, +\infty)$, and strictly loxodromic if and only if $\operatorname{trace}^2(g) \notin [0, +\infty)$ [BEARDON] Theorem 4.3.4, p. 67.

For a hyperbolic isometry $g$ of the hyperbolic plane, Beardon defines the translation length, which he writes $T$, as $L = \inf_z \rho(z, gz)$. The axis $A$ is the unique $g$-invariant geodesic, and [BEARDON] Section 7.34, p. 173 gives:

$$\frac{1}{2} \lvert \operatorname{trace}(g) \rvert = \cosh\frac{L}{2}$$

An elliptic $g$ with angle of rotation $\theta$ has $\operatorname{trace}^2(g) = 4 \cos^2(\theta/2)$ [BEARDON] Section 7.33, p. 173. The displacement of a point $z$ by a hyperbolic $g$ is [BEARDON] Theorem 7.35.1, p. 174:

$$\sinh\frac{\rho(z, gz)}{2} = \cosh \rho(z, A) \, \sinh\frac{L}{2}$$

Beardon gives no such statement for glide reflections or for matrices of determinant $-1$.

A one-parameter subgroup comes from the matrix exponential, which has a closed form for $2 \times 2$ matrices and satisfies $e^{tX} e^{sX} = e^{(t+s)X}$ [W-MATEXP] (secondary). The fetched page on the matrix logarithm says only that a $2 \times 2$ real matrix with negative determinant "has no real logarithm" [W-MATLOG] (secondary).

Bulatov's slides animate Mobius maps [BULATOV-ANIM], and Leys keeps a gallery of hyperbolic animations without describing a method [LEYS].

### 4.2. Deforming the Group

A second kind of animation changes the group itself. These facts bound what can be deformed:

- Fenchel-Nielsen coordinates give $3g - 3$ lengths and $3g - 3$ twists for a closed surface of genus $g$, and the page says nothing about orbifolds [W-FN] (secondary).
- The sign of the Euler characteristic of an orbifold fixes its geometry [W-ORBIFOLD] (secondary).
- The angle sum of a triangle group decides its geometry, and rational Schwarz triangles give overlapping tilings, whose "symmetries ... are not considered triangle groups" [W-TRIANGLE], [W-SCHWARZ] (secondary).
- A group of isometries of the hyperbolic plane is of type $(\alpha, \beta, \gamma)$ when it is generated by the reflections across the sides of a triangle with angles $\alpha$, $\beta$, and $\gamma$. Any two groups of the same type are conjugate in the group of all isometries, "because two triangles with the same angles are congruent" [BEARDON] Definition 10.6.1, p. 276.
- If a group of type $(\alpha, \beta, \gamma)$ is discrete, each positive angle has the form $k\pi/p$ with $k$ and $p$ coprime, which is "a necessary condition for discreteness but it is not sufficient" [BEARDON] (10.6.1), p. 278. A group of type $(\alpha, \beta, \gamma)$ is discrete if and only if it is also of some, possibly different, type $(\pi/p, \pi/q, \pi/r)$ [BEARDON] p. 278. Theorem 10.6.6 (p. 283) states it for conformal groups: "A conformal group of some type $(\alpha,\beta,\gamma)$ is discrete if and only if it is a Triangle group", where a Triangle group is a conformal group of a type $(\pi/p, \pi/q, \pi/r)$ (Definition 10.6.3, p. 278). Example 10.6.2 (p. 277) shows that the types $(0, \pi/2, \pi/3)$ and $(0, 0, 2\pi/3)$ give the same discrete group.

Bulatov's paper "Bending Hyperbolic Kaleidoscopes" describes bending deformations but could not be read [BULATOV-BEND]. McMullen's gallery has items titled "Maskit slice", "Holomorphic motion", "Quasifuchsian reflections", and "Kleinian movies", and the images themselves were not viewed [MCMULLEN]. Dumas's animated slices were not read [DUMAS].

### 4.3. Morphing Between Models and Between Geometries

A Mobius map "can also transform the disk model into the upper half plane model and into everything in between", and Bulatov shows a "band-stretch" animation from the disk to the band [BULATOV-2010]. Section 5.1 of `docs/research/geometry-models.md` settles the band map and its constant (Section 7). Distance is preserved along the midline of the band [W-HYPGEOM] (secondary).

The $\kappa$-stereographic model unifies the sphere, the Euclidean plane, and the hyperbolic plane. Its authors call it "a differentiable interpolation between all three geometries of constant curvature", and "choosing $\kappa = 0$ yields all corresponding Euclidean quantities" [BACHMANN]. The paper is about graph learning, so applying it to tilings is the research's own inference. The geoopt library implements the model [GEOOPT], and Ungar's gyrovector formalism is the theory beneath it [W-GYRO] (secondary).

KaleidoTile morphs polyhedra and tilings within one symmetry group [KALEIDOTILE]. The research found no evidence that it varies the curvature continuously.

### 4.4. Exporting Frames and Video

An exporter can write a frame sequence and leave the encoding to FFmpeg. The image2 demuxer reads numbered files through a `%0Nd` pattern, `start_number` sets the first index, and `-framerate` "defaults to 25" [FFMPEG-FORMATS] Section image2.

This command, from the FFmpeg documentation, encodes the files `img-001.jpeg`, `img-002.jpeg`, and so on at 10 frames per second:

```sh
ffmpeg -framerate 10 -i 'img-%03d.jpeg' out.mkv
```

The input option `-framerate` differs from the output option `-r`, which will "Duplicate or drop frames ... to achieve constant output frame rate" [FFMPEG]. The libx264 encoder takes the options `crf`, `preset`, `tune`, and `profile`, and the constant rate factor (CRF) range of libaom-av1 is "0 to 63" [FFMPEG-CODECS]. The usual advice to pass `-pix_fmt yuv420p` for compatibility appears only in search snippets of the FFmpeg Trac wiki (secondary).

Motion blur can come from accumulating sub-frames. Unity's HDRP 15.0 accumulation feature can "combine information from multiple intermediate sub-frames", with controls for the shutter interval and the shutter profile [UNITY-ACCUM]. Durand describes motion blur as "the temporal prefilter" [DURAND] Chapter 12.1.

### 4.5. Difficulty of Animation

These ratings are the researcher's own judgement:

| Item | Difficulty |
| --- | --- |
| View isometries and loops along one-parameter subgroups | Low |
| Interpolation by the exponential and the logarithm, including the choice of lift | Medium |
| Morphs between models | Low to medium for Mobius morphs, medium for the band |
| Morphs through curvature zero | Medium to high |
| Teichmuller or bending deformations | High |
| Export through FFmpeg | Low |

The main unknown is how to interpolate between two arbitrary isometries, because no source gives a method (Section 8.1, items 5 and 6).

## 5. Shader Export

### 5.1. What a Self-Contained Tiling Shader Needs

A tiling shader runs the fold loop per pixel. Bulatov states the step as "Reflect point in first found generator which has point in its exterior", and because the work for each pixel is independent, it runs on the GPU [BULATOV-2018]. Bulatov states no cap on the number of iterations. Triangle groups use the same method [ZHAO] (secondary), and a commercial WGSL example says that pixels near the rim "take up to 24 inversions" [SHADCN] (secondary). Sections 4.4, 4.7, and 4.8 of `docs/research/rendering.md` collect published iteration caps, termination results, and precision caveats for the fold loop.

The shading language limits what an exported shader can contain:

| Target | Loops | Precision | Source |
| --- | --- | --- | --- |
| WebGL 1, GLSL ES 1.00 | `for` loops "must conform to the structural constraints in Appendix A", and `while` loops are disallowed | `highp` is optional in fragment shaders, with a float24 minimum and $2^{-16}$ relative precision | [WEBGL1], [MDN-WEBGL] (secondary) |
| WebGL 2, GLSL ES 3.00 | The constant loop-bound rule is dropped | `highp` is IEEE float32 | [WEBGL2], [WEBGL2-FUND] (secondary), [MDN-WEBGL] (secondary) |

MDN gives a minimum of 64 uniform vectors in a fragment shader [MDN-WEBGL] (secondary). GLSL ES has no double type (inferred from its absence). The Shadertoy conventions `mainImage`, `iResolution`, and `iTime` are unverified, because Shadertoy returned HTTP 403.

### 5.2. Slang Release, Maintenance, and Licence

The latest Slang release is v2026.22, dated 2026-11-10, after v2026.21 on 2026-11-04 [SLANG-RELEASES]. Releases come about every one to three weeks, with patch releases in between (inferred from tag dates).

Khronos has hosted Slang since it announced the Slang Initiative on 2024-11-21. NVIDIA contributed the compiler and has supported it since 2017 [KHRONOS-SLANG]. Slang ships in the Vulkan SDK from version 1.3.296.0 [SLANG-README]. Its licence is `Apache-2.0 WITH LLVM-exception` [SLANG-LICENCE].

### 5.3. Slang Targets

This table gives the status of each target as the official documentation states it [SLANG-TARGETS], [SLANG-README]:

| Target | Status |
| --- | --- |
| SPIR-V (Vulkan) | Supported, emitted directly by default (`-emit-spirv-directly`) |
| HLSL (Direct3D 12) | Supported |
| GLSL (OpenGL) | "only limited support" |
| Metal | "a work in progress", with no `double` and `[unroll]` marked "No" |
| WGSL (WebGPU) | "a work in progress", with active fixes in v2026.22 and open issues such as #14187 (float matrix `%`) and #14390 (uniformity) |
| CUDA | Supported, but with no rasterization pipeline |
| CPU (C++) | "functional, but not feature- or performance-complete" |

Section 7 records two conflicts about the WGSL and Metal targets.

### 5.4. Slang Features for Generating Code

These features of the Slang user guide [SLANG-UG] bear on an exporter that generates or specializes shader code:

- Modules and linking: Modules compile to an intermediate representation (IR), and linking also performs link-time specialization (`08-compiling.md`).
- Link-time constants and types: `extern static const int kSampleCount = 2;` is resolved by an `export` in another module, and `extern struct ... : IFoo` works the same way for types (`10-link-time-specialization.md`).
- Generics and interfaces: The compiler "always attempts to determine the actual type of an interface-typed value at compile time" and otherwise falls back to dynamic dispatch, which needs `dyn` from language version 2026 (`06-interfaces-generics.md`, `11-language-version.md`).
- Specialization constants: `[SpecializationConstant]` or `[vk::constant_id(N)]` declares one for SPIR-V [SLANG-SPIRV].
- Parameter blocks: Each `ParameterBlock` gets its own bind group in WGSL and becomes an argument buffer in Metal (`a2-03-wgsl-target-specific.md`, `a2-02-metal-target-specific.md`).
- Reflection: Available through the compilation API, or as JSON from `slangc -reflection-json <path>`, and not otherwise from plain `slangc` output (`09-reflection.md`).
- Language versions: `#language` accepts 2018 (the default, `legacy`), 2025, 2026 (`latest`), and 202c (in development). Version 2025 requires `module` declarations, and 202c removes `$for` (`11-language-version.md`).

### 5.5. Slang Precision and Loops

Only `float` is supported on every target, and support for half and double "depends on the capabilities of each target platform" [SLANG-UG] `02-conventional-features.md`. NaN behaviour is not portable on WGSL [SLANG-UG] `a3-02-numerics.md`. The compiler option `-fp-mode precise|fast|default` selects the floating-point mode [SLANGC-REF].

The `[unroll]` attribute is a hint whose support "is dependent on downstream compiler support which is mixed" [SLANG-COMPAT]. The `[ForceUnroll]` attribute "forces the loop to be fully unrolled" [SLANG-SPEC-LOOP]. The specification adds: "If the loop cannot be unrolled, an error is diagnosed."

### 5.6. Alternatives and Translation Paths

An exporter can also write another language directly or translate compiled code:

| Path | Facts | Source |
| --- | --- | --- |
| WGSL | Float types are `f32` and `f16` only, and `f16` needs `enable f16;`. `override` declarations give constants that the pipeline can override. An array value constructor takes at most 2047 elements | [WGSL], Candidate Recommendation Draft of 2026-11-06 |
| WebGPU limits | `maxUniformBufferBindingSize` is 65536 bytes, or 16384 in compatibility mode | [WEBGPU], Candidate Recommendation Draft of 2026-09-15 |
| naga (wgpu) | Reads SPIR-V, WGSL, and GLSL 440 and later. Writes SPIR-V, MSL, HLSL, WGSL, and "GLSL 330+ and GLSL ES 300+", so it cannot write WebGL 1 shaders. f64 support is experimental behind `Capabilities::FLOAT64` (pull request #4747) | [NAGA] |
| wgpu `SHADER_F64` | Native only, and often "between 16 and 64 times slower" than f32 | [WGPU-FEATURES] |
| Vulkan doubles | Need the `shaderFloat64` device feature | [VK-FEATURES] |
| SPIRV-Cross | Converts SPIR-V to GLSL, HLSL, and MSL | [SPIRV-CROSS] |
| Tint (Dawn) | Has writers for SPIR-V, WGSL, HLSL, GLSL, and MSL | [TINT] |

### 5.7. Difficulty of Shader Export

These ratings are the researcher's own judgement:

| Item | Difficulty |
| --- | --- |
| A single-file GLSL ES 3.00 or WGSL shader with its constants baked in | Low to medium |
| Slang source compiled by `slangc` to SPIR-V and HLSL | Low |
| Slang to WGSL or Metal | Medium, because both targets are a work in progress and have open issues |
| WebGL 1 compatibility, with constant loop bounds and possibly no `highp` | Medium |
| Precision near the boundary in float32 | Not rated, because no source quantifies it (Section 8.1) |

The main unknown is the number of fold iterations that a tiling needs and the float32 precision near the limit circle (Section 8.1, item 11).

## 6. Reading Order

The research suggests reading these sources first, in this order:

| Order | Source | Why read it |
| --- | --- | --- |
| 1 | [SVG11-IMPL] Appendix F.6 | The exact arc semantics and the handling of out-of-range radii |
| 2 | [CAIRO-ARC] | A short error bound for Bezier arcs, with its citations |
| 3 | [LEVIEN-FIT] | Fitting band-model geodesics and other curves that are not arcs |
| 4 | [HYPERTILING] | Duplicate-free enumeration, layers, SVG output, and precision limits in one place |
| 5 | [BULATOV-2010] | The maps between the disk, the half-plane, and the band, and Mobius morph animations |
| 6 | [W-SL2R] and [W-MOBIUS] | Trace classes and one-parameter subgroups, the basis of view motions |
| 7 | [BACHMANN] | The $\kappa$-stereographic formulas for morphing through curvature zero |
| 8 | [BULATOV-2018] | The per-pixel fold loop that an exported shader runs |
| 9 | [SLANG-TARGETS], [SLANG-UG] `10-link-time-specialization.md`, and [SLANG-COMPAT] | What Slang can emit and how to specialize generated code |
| 10 | [WGSL] and [NAGA] | The limits of the web target and the translation paths |
| 11 | [FFMPEG-FORMATS] | Turning a frame sequence into a video |

## 7. Conflicts Between Sources

| Topic | One source | The other source | Handling |
| --- | --- | --- | --- |
| Band constant | [BULATOV-2010] gives $w = \frac{2}{\pi} \operatorname{artanh} z$ | Section 5.1 of `docs/research/geometry-models.md` settles $w = \frac{4}{\pi} \operatorname{artanh} z$ for the strip $\lvert \operatorname{Im} w \rvert < 1$ from several primary sources | `docs/research/geometry-models.md` prevails |
| Slang WGSL fragment outputs | The WGSL target page lists `SV_Target<N>` as "Not supported" [SLANG-UG] `a2-03-wgsl-target-specific.md` | Issue #13924, closed 2026-10-31, shows fragment outputs handled, and issue #14130 says that the WGSL pages lag the compiler [SLANG-ISSUES] | Unresolved |
| Status of the Slang targets | The README calls Metal, CPU, and WebGPU "experimental" [SLANG-README] | The Khronos press release lists WGSL and MSL as plain output targets [KHRONOS-SLANG] | Unresolved |
| Status text of WGSL | The draft of 2026-11-06 says it "will remain a Candidate Recommendation at least until 28 February 2025" | That date has passed | An internal inconsistency in [WGSL] |
| Year of Casselman's automata paper | The research gives 1995, in CMS Conference Proceedings 16 | `docs/research/source-requests.md` gives 1994 | Unresolved |

## 8. Open Questions and Unverified Claims

### 8.1. Questions No Source Answers

The research found no source for these points. Each one is a question for the deeper research pass of its part, and none is established:

1. A stopping rule for vector output, by Euclidean size, hyperbolic distance, or level of detail, and any data on file size.
2. Numerical problems of SVG arcs that are nearly straight or have very large radii, beyond the rationale and single-precision note of Section 3.2. Whether viewers have documented problems drawing tiny arcs.
3. A source that states that great circles under orthographic projection are ellipses, and that band-model geodesics are not circles.
4. The stopping criteria of HyperRogue and cduck/hyperbolic, and the vector export of the tools listed at the end of Section 3.5.
5. The seamless-loop property: that moving along the axis of a group element by its translation length repeats the frame.
6. That the exponential map from $\mathfrak{sl}(2, \mathbb{R})$ to $SL(2, \mathbb{R})$ is not surjective, and a method for interpolating between two arbitrary isometries.
7. That periodic scrolling in the band equals translation along the midline.
8. The Teichmuller dimensions of orbifolds. [BEARDON] states when a triangle group is discrete and that two groups of the same type are conjugate (Section 4.2).
9. A tiling tool that documents a continuous morph from the sphere through the Euclidean plane to the hyperbolic plane.
10. A primary source for deterministic frame timing, $t = \text{frame}/\text{fps}$.
11. Bounds on the number of fold iterations, and the precision of float32 near the limit circle.
12. For Slang: whether loops need constant bounds, documentation of `[MaxIters]`, emission of `f16` in WGSL, and the diagnostic for `double` on WGSL or Metal.
13. The published hyperbolic shaders by M. Arcus, Knighty, and Zucker, and the Shadertoy conventions of Section 5.1.

### 8.2. Claims Resting on Inference or Secondary Sources

- That PDF has no arc operator, inferred from the operator list of [ISO32000-MIRROR], which is itself a mirror.
- That GLSL ES has no double type, inferred from its absence.
- That Slang releases every one to three weeks, inferred from tag dates.
- That the $\kappa$-stereographic model of [BACHMANN] applies to tilings, which is the research's own inference.
- The `-pix_fmt yuv420p` advice, which rests on search snippets of the FFmpeg Trac wiki.
- Every claim marked (secondary) in Sections 3 to 5, which includes most of the geometry and animation material.

### 8.3. Sources Not Read

The research could not read these sources. The priority is the one that the research gave. `docs/research/source-requests.md` is the working list for such sources, and some of these, such as Katok, Casselman, and Bulatov's Bridges 2011 paper, already appear in it. This document does not change that list. The owner supplied copies of Dunham 1986 and Beardon 1983. Dunham 1986 was read in full as [DUNHAM-1986], which Sections 3.4 and 3.6 cite. Beardon 1983 was read in part as [BEARDON], which Sections 4.1 and 4.2 cite. Neither has a row below.

| Citation | What is needed | Priority |
| --- | --- | --- |
| D. Dunham, Bridges 1999 paper, <https://archive.bridgesmathart.org/1999/bridges1999-239.pdf>, and DIMACS talk, <http://www.d.umn.edu/~ddunham/dundimacs.pdf>. The PDFs were downloaded, but their text was not extracted | Stopping criteria based on size | High |
| W. Casselman, "Automata to perform basic calculations in Coxeter groups", CMS Conference Proceedings 16, AMS, 1995, pp. 37-60, <https://personal.math.ubc.ca/~cass/research/pdf/Automata.pdf>. Text not extracted | The construction of the shortlex automaton for duplicate-free enumeration | High |
| Shadertoy, "How to", <https://www.shadertoy.com/howto>, HTTP 403 | The `mainImage` and uniform conventions | High |
| D. Dunham, J. Lindgren, D. Witte, "Creating repeating hyperbolic patterns", SIGGRAPH Computer Graphics 15(3), 215-223, 1981, as reference [12] of [DUNHAM-1986] gives it (p. 153) | The original approach to enumeration and culling | Medium |
| M. Goldapp, "Approximation of circular arcs by cubic polynomials", CAGD 8(3), 227-238, 1991, HTTP 403 | The optimal Bezier constant and its error | Medium |
| P. Schneider, "An algorithm for automatically fitting digitized curves", Graphics Gems, 1990, HTTP 404 | Fitting curves that are not arcs | Medium |
| V. Bulatov, "Bending Hyperbolic Kaleidoscopes", Bridges 2011, pp. 479-482. Text not extracted | How bending deformations work | Medium |
| S. Katok, Fuchsian Groups, University of Chicago Press, 1992. Book, cited from memory | Translation length, triangle groups, and discreteness | Medium |
| D. Mumford, C. Series, D. Wright, Indra's Pearls, Cambridge University Press, 2002. Book, and the website refused the connection | Chapters 3 and 8 to 10 on parameter spaces and deformation animations | Medium |
| W. Thurston, Three-Dimensional Geometry and Topology, Vol. 1, Princeton University Press, 1997. Book | Deformation spaces of 2-orbifolds | Medium |
| Khronos, GLSL ES 1.00 and 3.00 specifications. PDFs not extracted | The exact loop and indexing rules of Appendix A, and the precision tables | Medium |
| B. Brink, R. Howlett, "A finiteness property and an automatic structure for Coxeter groups", Math. Annalen 296, 179-190, 1993. Paywall | The automatic structure | Low |
| T. Dokken, M. Daehlen et al., "Good approximation of circles by curvature-continuous Bezier curves", CAGD 8, 22-41, 1990 | The error bound that cairo uses | Low |
| A. Riskus, "Approximation of a cubic Bezier curve by circular arcs and vice versa", Information Technology and Control 35(4), 371-378, 2006 | Conversion between arcs and Bezier curves | Low |
| A. A. Ungar, Analytic Hyperbolic Geometry and Albert Einstein's Special Theory of Relativity, World Scientific, 2008. Book | Gyrovector formulas with the curvature as a parameter | Low |
| FFmpeg Trac wiki, pages Slideshow and Encode/H.264. Bot protection | The CRF range and the `yuv420p` advice | Low |
| R. Cook, T. Porter, L. Carpenter, "Distributed ray tracing", SIGGRAPH 1984. ACM returned HTTP 403 | Temporal sampling for motion blur | Low |

## 9. References

- [BACHMANN] Bachmann, G., Becigneul, G., Ganea, O.-E., "Constant Curvature Graph Convolutional Networks", ICML 2020, arXiv 1911.05076, <https://ar5iv.labs.arxiv.org/html/1911.05076>. Not marked.
- [BEARDON] Beardon, A. F., "The Geometry of Discrete Groups", Graduate Texts in Mathematics 91, Springer, 1983, softcover reprint of the first edition, cited by printed page. Read in part. Primary.
- [BULATOV-2010] Bulatov, V., "Conformal Models of the Hyperbolic Geometry", slides, 2010, <http://www.bulatov.org/math/1001/> and <https://www.bulatov.org/math/1001/index.html>. Practitioner.
- [BULATOV-2018] Bulatov, V., "Inverse pixel method" article, 2018, <https://www.bulatov.org/math/180110/index_web.html>. Practitioner.
- [BULATOV-ANIM] Bulatov, V., Mobius animation slides, year not recorded, <https://www.bulatov.org/math/1001/moebius-animation1.html>. Not marked.
- [BULATOV-BEND] Bulatov, V., "Bending Hyperbolic Kaleidoscopes", Bridges 2011, pp. 479-482, <https://static1.bridgesmathart.org/2011/cdrom/proceedings/101/paper_101.pdf>. Not read.
- [CAIRO-ARC] cairo, `cairo-arc.c` source comment, year not recorded, <https://searchfox.org/mozilla-central/source/gfx/cairo/cairo/src/cairo-arc.c>. Secondary.
- [CDUCK] cduck, "hyperbolic" (repository), year not recorded, <https://github.com/cduck/hyperbolic>. Primary.
- [DUMAS] Dumas, D., animated slices, year not recorded, <http://www.math.uic.edu/~ddumas/bear/anims>. Not read.
- [DUNHAM-1986] Dunham, D., "Hyperbolic symmetry", Computers and Mathematics with Applications 12B(1/2), 139-153, 1986, DOI 10.1016/0898-1221(86)90147-1. Read in full. Primary.
- [DURAND] Durand, F., et al., computational photography book, Chapter 12.1, year not recorded, <https://people.csail.mit.edu/fredo/comp-photo-book/12-video-01-motion-blur-and-temporal-sampling.html>. Academic.
- [FFMPEG] FFmpeg, "ffmpeg Documentation", year not recorded, <https://ffmpeg.org/ffmpeg.html>. Primary.
- [FFMPEG-CODECS] FFmpeg, "FFmpeg Codecs Documentation", year not recorded, <https://ffmpeg.org/ffmpeg-codecs.html>. Primary.
- [FFMPEG-FORMATS] FFmpeg, "FFmpeg Formats Documentation", year not recorded, <https://ffmpeg.org/ffmpeg-formats.html#image2-1>. Primary.
- [GEOOPT] geoopt, "Stereographic projection model" documentation, year not recorded, <https://geoopt.readthedocs.io/en/stable/extended/stereographic.html>. Not marked.
- [HATCH] Hatch, D., "HyperbolicApplet", undated, <https://www.plunk.org/~hatch/HyperbolicApplet/>. Primary.
- [HONEYCOMBS] Nelson, R., "Honeycombs" (repository), year not recorded, <https://github.com/roice3/Honeycombs>. Not marked.
- [HYPERROGUE] HyperRogue 12.1q, `screenshot.cpp`, Debian sources, year not recorded, <https://sources.debian.org/src/hyperrogue/12.1q-1/screenshot.cpp>. Primary.
- [HYPERTILING] Yasuda, Schrauth, et al., "hypertiling: a high performance Python library", SciPost Physics Codebases 34, arXiv 2309.10844v3, 2023, <https://arxiv.org/html/2309.10844v3>. Primary.
- [ISO32000-MIRROR] ISO 32000-1, Section 8.5.2, read through a mirror, year not recorded, <https://hellowac.github.io/pdf-std-zh-cn/iso32000/c8/s5>. Secondary.
- [KALEIDOTILE] Weeks, J., "KaleidoTile", year not recorded, <https://www.geometrygames.org/KaleidoTile/index.html.en> and <https://www.geometrygames.org/KaleidoTile/>. Not marked.
- [KHRONOS-SLANG] Khronos Group, "Khronos Group Launches Slang Initiative, Hosting Open Source Compiler Contributed by NVIDIA", 2024, <https://www.khronos.org/news/press/khronos-group-launches-slang-initiative-hosting-open-source-compiler-contributed-by-nvidia>. Primary, read through the summarizer.
- [LEVIEN-FIT] Levien, R., "Fitting cubic Bezier curves", 2021, <https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html>. Not marked.
- [LEVIEN-FLAT] Levien, R., "Flattening quadratic Beziers", 2019, <https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html>. Secondary.
- [LEYS] Leys, J., "Hyperbolic animations" gallery, year not recorded, <https://www.josleys.com/show_gallery.php?galid=297>. Not marked.
- [MCMULLEN] McMullen, C., gallery, 2015, <https://people.math.harvard.edu/~ctm/gallery/menu.html>. Primary.
- [MDN-WEBGL] MDN, "WebGL best practices", year not recorded, <https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices>. Secondary.
- [MORTENSEN] Mortensen, S., "Approximate a circle with cubic Bezier curves", 2012, <https://spencermortensen.com/articles/bezier-circle/>. Secondary.
- [NAGA] gfx-rs, "naga README.md", year not recorded, <https://github.com/gfx-rs/wgpu/blob/trunk/naga/README.md>. Primary.
- [POMAX] Pomax, "A Primer on Bezier Curves", circles chapter, year not recorded, <https://raw.githubusercontent.com/Pomax/BezierInfo-2/master/docs/chapters/circles_cubic/content.en-GB.md>. Secondary.
- [RAW-BEZIER] raw.org, "Approximating circles with cubic Bezier curves", year not recorded, <https://raw.org/math/computer-graphics/approximating-circles-with-cubic-bezier-curves/>. Secondary.
- [SHADCN] shadcn.io, "Poincare disc animation shader" (product page), year not recorded, <https://www.shadcn.io/shaders/poincare-disc-animation-shader>. Secondary.
- [SLANG-COMPAT] Slang, "Target compatibility", year not recorded, <https://shader-slang.org/slang/target-compatibility.html>. Primary.
- [SLANG-ISSUES] shader-slang, "slang issues", 2026, <https://github.com/shader-slang/slang/issues>. Primary.
- [SLANG-LICENCE] shader-slang, "slang licence file", year not recorded, `https://raw.githubusercontent.com/shader-slang/slang/master/LICENSE`. Primary.
- [SLANG-README] shader-slang, "slang README", year not recorded, <https://github.com/shader-slang/slang>. Not marked.
- [SLANG-RELEASES] shader-slang, "slang releases", 2026, <https://github.com/shader-slang/slang/releases>. Primary.
- [SLANG-SPEC-LOOP] shader-slang, "Slang specification, loop statements", year not recorded, <https://raw.githubusercontent.com/shader-slang/spec/main/specification/statements-loop.md>. Not marked.
- [SLANG-SPIRV] Slang, "SPIR-V target specific" user guide chapter `a2-01-spirv-target-specific`, year not recorded, <https://docs.shader-slang.org/en/latest/external/slang/docs/user-guide/a2-01-spirv-target-specific.html>. Primary.
- [SLANG-TARGETS] Slang, "Supported Compilation Targets", year not recorded, <https://shader-slang.org/slang/user-guide/targets.html>. Primary.
- [SLANG-UG] shader-slang, Slang user guide, chapters as Markdown files under `docs/user-guide`, year not recorded, <https://raw.githubusercontent.com/shader-slang/slang/master/docs/user-guide/10-link-time-specialization.md>, <https://raw.githubusercontent.com/shader-slang/slang/master/docs/user-guide/11-language-version.md>, and <https://raw.githubusercontent.com/shader-slang/slang/master/docs/user-guide/a2-03-wgsl-target-specific.md>. The research gave full URLs only for these three chapters. Primary.
- [SLANGC-REF] shader-slang, "slangc command-line reference", year not recorded, <https://raw.githubusercontent.com/shader-slang/slang/master/docs/command-line-slangc-reference.md>. Not marked.
- [SPIRV-CROSS] KhronosGroup, "SPIRV-Cross" (repository), year not recorded, <https://github.com/KhronosGroup/SPIRV-Cross>. Not marked.
- [SVG11-IMPL] W3C, "Scalable Vector Graphics (SVG) 1.1 (Second Edition)", Appendix F, 2011, <https://www.w3.org/TR/SVG11/implnote.html>. Primary.
- [SVG11-PATHS] W3C, "Scalable Vector Graphics (SVG) 1.1 (Second Edition)", Section 8, 2011, <https://www.w3.org/TR/SVG11/paths.html>. Primary.
- [SVG2-IMPL] W3C, "Scalable Vector Graphics (SVG) 2", Appendix B, year not recorded, <https://www.w3.org/TR/SVG2/implnote.html>. Primary.
- [TINT] Dawn, "Tint architecture", year not recorded, <https://dawn.googlesource.com/dawn/+/main/docs/tint/arch.md>. Primary.
- [UNITY-ACCUM] Unity, "Accumulation", HDRP 15.0 manual, year not recorded, <https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@15.0/manual/Accumulation.html>. Primary, vendor.
- [VK-FEATURES] Khronos Group, "VkPhysicalDeviceFeatures", year not recorded, <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFeatures.html>. Not marked.
- [WEBGL1] Khronos Group, "WebGL Specification 1.0", year not recorded, <https://registry.khronos.org/webgl/specs/latest/1.0/>. Primary.
- [WEBGL2] Khronos Group, "WebGL 2.0 Specification", year not recorded, <https://registry.khronos.org/webgl/specs/latest/2.0/>. Primary.
- [WEBGL2-FUND] webgl2fundamentals, "WebGL2 What's New", year not recorded, <https://webgl2fundamentals.org/webgl/lessons/webgl2-whats-new.html>. Secondary.
- [WEBGPU] W3C, "WebGPU", Candidate Recommendation Draft, 2026-09-15, <https://www.w3.org/TR/webgpu/>. Primary.
- [WGPU-FEATURES] docs.rs, "wgpu::Features", year not recorded, <https://docs.rs/wgpu/latest/wgpu/struct.Features.html>. Primary.
- [WGSL] W3C, "WebGPU Shading Language", Candidate Recommendation Draft, 2026-11-06, <https://www.w3.org/TR/WGSL/>. Primary.
- [W-BAND] Wikipedia, "Band model", <https://en.wikipedia.org/wiki/Band_model>. Secondary.
- [W-COXETER] Wikipedia, "Coxeter group", <https://en.wikipedia.org/wiki/Coxeter_group>. Secondary.
- [W-DISK] Wikipedia, "Poincare disk model", <https://en.wikipedia.org/wiki/Poincar%C3%A9_disk_model>. Secondary.
- [W-FN] Wikipedia, "Fenchel-Nielsen coordinates", <https://en.wikipedia.org/wiki/Fenchel%E2%80%93Nielsen_coordinates>. Secondary.
- [W-FUCHSIAN] Wikipedia, "Fuchsian group", <https://en.wikipedia.org/wiki/Fuchsian_group>. Secondary.
- [W-GYRO] Wikipedia, "Gyrovector space", <https://en.wikipedia.org/wiki/Gyrovector_space>. Secondary.
- [W-HALF-PLANE] Wikipedia, "Poincare half-plane model", <https://en.wikipedia.org/wiki/Poincar%C3%A9_half-plane_model>. Secondary.
- [W-HYPERBOLOID] Wikipedia, "Hyperboloid model", <https://en.wikipedia.org/wiki/Hyperboloid_model>. Secondary.
- [W-HYPGEOM] Wikipedia, "Hyperbolic geometry", <https://en.wikipedia.org/wiki/Hyperbolic_geometry>. Secondary.
- [W-KLEIN] Wikipedia, "Beltrami-Klein model", <https://en.wikipedia.org/wiki/Beltrami%E2%80%93Klein_model>. Secondary.
- [W-MATEXP] Wikipedia, "Matrix exponential", <https://en.wikipedia.org/wiki/Matrix_exponential>. Secondary.
- [W-MATLOG] Wikipedia, "Logarithm of a matrix", <https://en.wikipedia.org/wiki/Logarithm_of_a_matrix>. Secondary.
- [W-MOBIUS] Wikipedia, "Mobius transformation", <https://en.wikipedia.org/wiki/M%C3%B6bius_transformation>. Secondary.
- [W-ORBIFOLD] Wikipedia, "Orbifold", <https://en.wikipedia.org/wiki/Orbifold>. Secondary.
- [W-PARALLEL] Wikipedia, "Parallel projection", <https://en.wikipedia.org/wiki/Parallel_projection>. Secondary.
- [W-SCHWARZ] Wikipedia, "Schwarz triangle", <https://en.wikipedia.org/wiki/Schwarz_triangle>. Secondary.
- [W-SL2R] Wikipedia, "SL2(R)", <https://en.wikipedia.org/wiki/SL2(R)>. Secondary.
- [W-STEREO] Wikipedia, "Stereographic projection", <https://en.wikipedia.org/wiki/Stereographic_projection>. Secondary.
- [W-TODD-COXETER] Wikipedia, "Todd-Coxeter algorithm", <https://en.wikipedia.org/wiki/Todd%E2%80%93Coxeter_algorithm>. Secondary.
- [W-TRIANGLE] Wikipedia, "Triangle group", <https://en.wikipedia.org/wiki/Triangle_group>. Secondary.
- [ZHAO] Zhao Liang, "uniform tilings", 2019, <https://pywonderland.com/uniform-tilings>. Secondary.
