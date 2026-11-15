# Partial Differential Equations on Orbifolds: A Survey

| Field | Value |
| --- | --- |
| Status | Snapshot |
| Date | 2026-11-15 |
| Applies to | Roadmap planning for the simulation of planar partial differential equations on orbifolds, with GPU compute through Slang |

## Abstract

This document surveys how planar partial differential equations, such as reaction-diffusion equations, can be simulated on two-dimensional orbifolds of the sphere, the Euclidean plane, and the hyperbolic plane, with time stepping on the GPU. It states the mathematical setting and compares five families of numerical methods. The research orders them by how readily they fit a per-pixel renderer that folds each point into a fundamental domain. In that order they are covering-space simulation with symmetrization, finite differences with ghost cells, finite elements or discrete exterior calculus, graph Laplacians on the tiling, and spectral methods. The research found no error analysis of symmetrization, the method that SymmHub uses, and almost no study of pattern formation on hyperbolic surfaces. The document also summarizes pattern formation on curved surfaces, existing software, GPU practice, and a reading order. It closes with the conflicts between sources, the open questions, and the sources that could not be read. Its readers are the people and agents who write the roadmap. It is a survey and not an implementation reference, and it makes no recommendation or decision about what Escher builds.

## 1. Scope

This document covers the Laplace-Beltrami operator on orbifolds in conformal charts, the boundary and identification conditions that the features of an orbifold signature impose, five method families with their trade-offs, pattern formation on curved surfaces, existing software and published work, and GPU practice for explicit time stepping. Where the document orders methods by fit or assesses difficulty, it reports the assessment of the research and not a decision.

It does not repeat what `docs/research/symmhub.md` records about the SymmHub simulations. Section 13.1 of that document describes the SymmHub `Gray-Scott` and Ginzburg-Landau steps, the hyperbolic metric option, and the symmetrization pass, and this document adds only the facts that it lacks. The construction of fundamental domains belongs to `docs/research/fundamental-domains.md`, and orbifold notation belongs to `docs/research/orbifold-notation.md`.

All facts come from one research pass, run by eight web-searcher agents, with every source accessed on 2026-11-15. The research rated its overall confidence as medium. The setting, the software facts, and the method landscape rest on primary sources, but no error or stability analysis specific to orbifolds was found. Many arXiv and journal PDFs could not be decoded, so several key texts were read only as abstracts, summaries, ar5iv HTML, or citing papers. Section 10 lists the open questions, the gaps, and the sources not read.

## 2. Notation and Conventions

### 2.1. Symbols

- $X$: The covering space, which is the sphere $\mathbb{S}^2$, the Euclidean plane $\mathbb{E}^2$, or the hyperbolic plane $\mathbb{H}^2$.
- $\Gamma$: A discrete group of isometries of $X$, named by an orbifold signature.
- $z = x + iy$: A point in a conformal chart.
- $g = \lambda^2 |dz|^2$: A conformal metric, with conformal factor $\lambda(z) > 0$.
- $\Delta$: The Laplace-Beltrami operator, in the sign convention of the cited sources.
- $\Delta_\Gamma$: The Laplace-Beltrami operator of the orbifold $\Gamma \backslash X$.
- $D$: A diffusion coefficient.
- $h$: The grid spacing, or the mesh size of a triangulation.
- $\Delta t$: The time step.
- $\mathbf{M}$, $\mathbf{K}$: The mass matrix and the stiffness matrix of a finite element method.

### 2.2. Terminology

- orbifold: The quotient $\Gamma \backslash X$, here a two-dimensional orbifold.
- fundamental domain: A region of $X$ that contains one point of each orbit of $\Gamma$, up to its boundary.
- side pairing: The element of $\Gamma$ that carries one side of a fundamental polygon to its paired side.
- invariant function: A function $f$ on $X$ with $f(\gamma z) = f(z)$ for every $\gamma \in \Gamma$.
- cone point: A point of the orbifold whose neighbourhood is a disk quotiented by a cyclic group of rotations of order $n$.
- Friedrichs extension: The self-adjoint extension of the Laplacian that the cited sources use on surfaces with conical points.
- covering-space simulation: Time stepping on a region of $X$ or on a periodic texture, with the symmetry of $\Gamma$ imposed afterwards.
- symmetrization: The step that replaces each value of a field by the value at its folded point in the fundamental domain, or mixes the two.
- ghost cell: A grid cell outside the computational domain whose value is copied from inside the domain before each stencil step.
- finite element method (FEM): Discretization by piecewise polynomial functions on a triangulation.
- discrete exterior calculus (DEC): A discretization of differential forms on a simplicial mesh, which gives the cotangent Laplacian on a triangle mesh.
- method of particular solutions (MPS): A spectral method that expands a solution in functions that satisfy the equation exactly.
- `{p,q}`: The Schlafli symbol of the regular tiling by `p`-gons with `q` at each vertex.

### 2.3. Conventions

- This document writes orbifold signatures as ASCII code spans, with `o` for a handle, `*` for a mirror boundary, and `x` for a crosscap, as `docs/research/orbifold-notation.md` does.
- Names with accented letters are written in ASCII transliteration, such as Poincare, Berard, Stromberg, Aragon, Sanchez-Garduno, and Kollar, because the prose checker rejects non-ASCII characters.
- The proper name `Gray-Scott` is written as a code span, because the prose checker rejects its spelling.
- Sources normalize the curvature of the Poincare disk differently, to $-1$ or to $-4$. Section 3.1 gives both, and Section 9 records the consequence for SymmHub.
- "(secondary)" marks a fact that rests on a secondary source, such as Wikipedia. "(unverified)" marks a fact that the research knew only from a search snippet or a summary by its fetch tool. "(not read)" marks a source that the research cited but did not read. Each claim keeps the mark that the research gave it.

## 3. Mathematical Setting

### 3.1. The Laplace-Beltrami Operator in Conformal Charts

A field on an orbifold is a field on the covering space that is invariant under the group. In a conformal chart with metric $g = \lambda^2 |dz|^2$, the Laplace-Beltrami operator is a scaled flat Laplacian [WP-POINCARE-METRIC] (secondary, formula reconstructed from flattened text):

$$\Delta = 4 \lambda^{-2} \partial_z \partial_{\bar z} = \lambda^{-2} \left( \frac{\partial^2}{\partial x^2} + \frac{\partial^2}{\partial y^2} \right)$$

In general coordinates the operator is [WP-LAPLACE-BELTRAMI] (secondary):

$$\Delta f = \frac{1}{\sqrt{|g|}} \partial_i \left( \sqrt{|g|} \, g^{ij} \partial_j f \right)$$

This table gives the metric and the operator in each chart that the sources state:

| Chart | Metric | Conformal factor or operator | Sources |
| --- | --- | --- | --- |
| Poincare disk, curvature $-1$ | $ds^2 = 4(dx^2 + dy^2)/(1 - \lvert z \rvert^2)^2$ | $\lambda = 2/(1 - \lvert z \rvert^2)$ | [WP-POINCARE-METRIC], [WP-POINCARE-DISK] (secondary) |
| Poincare disk, curvature $-4$ | $ds^2 = dz \, d\bar z/(1 - z \bar z)^2$ | $\Delta = (1 - \lvert z \rvert^2)^2 (\partial_x^2 + \partial_y^2)$ | [PETERMANN-HINRICHSEN], [BOETTCHER2020] |
| Upper half-plane | Not recorded | $\Delta = y^2 (\partial_x^2 + \partial_y^2)$ | [MARKLOF2004], [BUCATARU-MANEA] |
| Sphere, stereographic chart | $ds^2 = 4(du^2 + dv^2)/(1 + u^2 + v^2)^2$ | $\lambda = 2/(1 + \lvert z \rvert^2)$ | [WP-RIEMANN-SPHERE] (secondary) |

No source in the research states the operator for the curvature $-1$ disk explicitly. Section 10.1 records the candidate formula.

### 3.2. Invariant Functions

The Laplacian of an orbifold is the Laplacian of the covering space restricted to invariant functions: "the Laplace-Beltrami operator $\Delta_\Gamma$ on $(\Gamma \backslash M, g)$ coincides with $\Delta$ restricted to $\Gamma$-invariant smooth functions on $M$" [CYCLIC2015]. A simulation on the orbifold is therefore a simulation on the covering space whose fields stay invariant.

### 3.3. Boundary and Identification Conditions

Each feature of a signature becomes a condition on the boundary of a fundamental domain:

| Feature | Condition | Sources |
| --- | --- | --- |
| Mirror, `*` | Neumann condition: "a $\Gamma$-invariant smooth function on R restricts to a function on $\lvert \mathcal{O} \rvert$ with zero normal derivative on the mirror edge" | [BERARD-WEBB2020], [FARSI2016] |
| Paired sides, from translations or a handle `o` | Periodic identification. "The sides are matched in distinct pairs, so that there is a unique element of $\Gamma$ carrying a side to the paired side." On the hyperbolic plane the identification is made "by gluing paired edges of the fundamental polygon", and the generators "do not commute". | [WP-FUNDAMENTAL-POLYGON] (secondary), [SAUSSET-TARJUS2007] |
| Glide reflection or crosscap `x` | "an $\times$ indicates a glide reflection". In a fundamental polygon it is a side pairing with the same exponent, as in $ABAB$ for the projective plane and $ABAB^{-1}$ for the Klein bottle, which identifies the sides with reversed orientation. | [WP-ORBIFOLD-NOTATION] (secondary), [CBG2008] (not read) |
| Cone point of order $n$ | The local model is "the origin of R2 quotiented out by a cyclic group of order n of rotations". On surfaces with conical points "we denote by $\Delta$ the Friedrichs extension", and "the functions from the domain of the Friedrichs extension are bounded near the vertex". Kokotov treats general cone angles, not only $2\pi/n$. | [WP-ORBIFOLD] (secondary), [KOKOTOV2009] |

## 4. Method Families

### 4.1. Comparison

The research found five method families. It ordered them by how readily they fit a per-pixel renderer that folds each point into a fundamental domain, in the order (a), (b), (c), (e), and (d). That order is the research's assessment, and the table keeps the labels of the research. Method (a) is what SymmHub does.

| Method | Idea | Accuracy | Stability and time step | Cost | Fit for GPU compute | Sources |
| --- | --- | --- | --- | --- | --- | --- |
| (a) Covering-space simulation with symmetrization | Simulate on a periodic texture, which is a torus `o`. Every $N$ steps, replace each texel by the interpolated value at its folded point. | No error analysis found. The author acknowledges interpolation errors and persistent seams. | Explicit Euler on a flat grid | Very low. Symmetrization steps "add very little cost". | Excellent | [BULATOV-MAA2025], [SYMMHUB-REPO] |
| (b) Finite differences on a chart with ghost cells | A grid on a conformal chart, with the flat stencil multiplied by $\lambda^{-2}$. Ghost cells are filled through the side pairings. | $O(h^2)$ consistency and convergence for a finite-difference Laplacian on $\mathbb{H}^2$, in the half-plane with a $\theta$-scheme | No sourced bound for a conformal chart (Section 10.2) | Low | Good. Irregular ghost lookups across hyperbolic pairings need interpolation. | [BUCATARU-MANEA]. No source found for ghost cells across glide or hyperbolic pairings. |
| (c) FEM or DEC with the cotangent Laplacian | Triangulate the fundamental domain and identify the paired boundary vertices. Mirrors give natural Neumann conditions. | Converges in a Sobolev or weak sense, not pointwise | Forward Euler needs $\Delta t \le 2/\lambda_{\max}(\mathbf{M}^{-1}\mathbf{K})$, with $\lambda_{\max} \sim h_{\min}^{-2}$ | One sparse matrix-vector product per step | Fair. Each node gathers over a fixed list of neighbours. | [WP-DISCRETE-LAPLACE] (secondary), [HPW2006] (unverified), [DZIUK-ELLIOTT2013] (not read), [CFL2025], [FREEFEM-BLOCH] |
| (d) Spectral or automorphic | Expand in invariant eigenfunctions, by Hejhal's method or the MPS. | The most accurate of the five. "converges exponentially fast with the number of basis functions" | Not oriented to time stepping | High setup cost | Poor for dynamics | [STROHMAIER-USKI2013], [STROMBERG2006], [THEN2005] |
| (e) Lattice or graph on the tiling | Use the graph Laplacian on the vertices or tiles of a `{p,q}` tiling. | A fetch-tool reading of [BOETTCHER2020] gives a continuum limit that approximates $\Delta_g$, which is unverified (Section 10.1). The spacing is "fixed by hyperbolic geometry and cannot be tuned". | No source found for the explicit limit of graph diffusion | Low | Good, with an adjacency buffer | [BOETTCHER2020], [PETERMANN-HINRICHSEN], [NAKAO-MIKHAILOV2010] |

### 4.2. Notes on Method (a)

The research read the main branch of [SYMMHUB-REPO] through its fetch tool and found these facts in addition to Section 13.1 of `docs/research/symmhub.md`:

- The `Gray-Scott` simulation sets `symIterations: 2` and `deltaT: 0.8` in `apps/symsim/gray_scott/js/gray_scott_simulation.js`.
- The 9-point stencil carries no grid spacing.
- The Ginzburg-Landau step in `gl_step.glsl.mjs` symmetrizes after every batch of 4 steps.

The SymSim slides state that "On practice the symmetrization step can be used only once per 1000 simulation steps" [BULATOV-MAA2025]. An earlier talk states that "Simple Symmetrization works only for 4 out of 17 wallpaper groups" [BULATOV-G4G15]. The slides also report persistent discontinuities on the boundaries of the fundamental domain [BULATOV-MAA2025].

### 4.3. Notes on Method (c)

FreeFEM supports twisted pairings, such as `periodic=[[4,y],[2,1-y]]` [FREEFEM-BLOCH]. One user reports a limit of 10 periodic pairs, which is unconfirmed [FREEFEM-PERIODIC]. Strohmaier writes that the Bolza surface "can be done rather quickly in the freely available FreeFEM++" [STROHMAIER2016].

Maciejko and Rayan used FreeFEM++ on the Bolza surface "with strictly automorphic boundary conditions" [MACIEJKO-RAYAN2021]. Chossat and Faye meshed "the full octagon with 3641 nodes" with "periodic boundary conditions" [CHOSSAT-FAYE2013].

### 4.4. Notes on Method (d)

The MPS "can not distinguish between multiplicities and close eigenvalues" [STROHMAIER-USKI2013]. Hejhal's method gives eigenvalues to 12 digits, but "the coefficients are only accurate to three digits" [THEN2005] (summarized quote).

### 4.5. Notes on Method (e)

On regular lattices, "Turing patterns on the lattices should therefore have almost the same properties as in the continuous media" [NAKAO-MIKHAILOV2010]. Lux and Prodan give periodic boundary conditions on `{p,q}` lattices [LUX-PRODAN2023].

## 5. Pattern Formation on Curved Surfaces

This section is non-normative. The research found that pattern formation is well studied on spheres and almost not at all on hyperbolic surfaces. It found no study of Turing, `Gray-Scott`, or Ginzburg-Landau patterns on a hyperbolic orbifold beyond Bulatov's art pieces (Section 6).

| Topic | Finding | Sources |
| --- | --- | --- |
| Spheres | Curvature limits the shapes of patterns. | [VAREA1999] (abstract only) |
| Spheres | Reaction-diffusion on the sphere has been solved with spherical harmonics and fast Fourier transforms. | [CHAPLAIN2001] |
| Equivariant selection | Equivariant bifurcation for harmonics of degree $l$ shows that the icosahedral pattern is preferred for $l = 6$, $10$, and $12$. | [MATTHEWS2003] (abstract only) |
| Equivariant selection | Patterns are tied to isotropy subgroups of $O(3)$. | [ISOTROPY2019] (secondary) |
| Degeneracy | "a particular spatial eigenvalue on the sphere will, in general, have many distinct eigenfunctions", and "curvature can influence mode selection as well as the structure of patterned states". | [KRAUSE2021] (review) |
| Varying curvature | "diffusion is enhanced (diminished) where Gaussian curvature is negative (positive)", and bulges "break symmetry and can pin a pattern". | [FRANK2019] |
| Torus | The "outer side of the torus contain more number of spots than inner side". | [NAMPOOTHIRI2017] |
| Varying curvature | Curvature can drive oscillatory and chaotic dynamics. | [NISHIDE-ISHIHARA2024] |
| Growth | Growth and curvature play a crucial role in pattern selection. | [PLAZA2004] (paraphrase, not read) |
| Growing sphere | The destabilization of stripes is "an effect due to the curvature of the domain not present in planar isotropic growth". | [KRAUSE2019] |
| Hyperbolic plane | "there are significant differences in the results with the Euclidean case". The Swift-Hohenberg equation undergoes a Hopf bifurcation to travelling waves that are invariant along horocycles (paraphrased). The octagonal lattice has the symmetry group $GL(2,3)$ of order 48. | [CHOSSAT-FAYE2013] |
| Hyperbolic plane | Bifurcation of hyperbolic planforms. The results "would apply equally to ... reaction-diffusion". | [CHOSSAT2011] |
| `Gray-Scott` on a sphere | Solved with COMSOL on 6670 triangles. | [MEMBRILLO-SOLIS2025] |

## 6. Existing Software and Published Work

| Item | What it does | Sources |
| --- | --- | --- |
| SymSim and SymmHub, by Bulatov and Goodman-Strauss | WebGL2 `Gray-Scott` and Ginzburg-Landau simulations on orbifolds, with Euclidean, hyperbolic, and inversive groups, under the MIT licence | [SYMSIM-SITE], [SYMMHUB-REPO], [SYMMHUB-SITE] |
| Bulatov, `Gray-Scott` on the orbifold `3222` | "We run simulation of the equation on the hyperbolic orbifold 3222." | [BULATOV-JMM2025] |
| Circuit quantum electrodynamics (QED) hyperbolic lattices | Networks of resonators that form lattices in an effective hyperbolic space | [KOLLAR2019] |
| Hyperbolic band theory | Bloch theory on quotients of higher genus, solved with FreeFEM++ | [MACIEJKO-RAYAN2021], [ATTAR-BOETTCHER2022] |
| HYPERTILING | Generates `{p,q}` lattices with "millions of vertices" in Python, and lists reaction-diffusion on the lattice as a use case | [HYPERTILING2024] |
| HyperBloch and HyperCells | Mathematica and GAP tools for the band structures of supercells | [HYPERBLOCH] |
| Strohmaier and Uski code | Rigorous eigenvalues on hyperbolic surfaces | [STROHMAIER-USKI2013] |
| HyperRogue and RogueViz | A hyperbolic game and visualization engine. No cellular automaton feature was confirmed. | [HYPERROGUE] |

## 7. GPU Practice

This section is non-normative and records the practice that the sources describe for explicit stencil simulations on the GPU.

A simulation step reads one buffer and writes another, and the two swap roles each step, because "the results of rendering to a texture while it is bound for reading are undefined", and "The swap in this case is merely a swap of texture IDs" [HARRIS2004]. The WebGPU Game of Life sample alternates bind groups over two storage buffers, in `gameOfLife/main.ts` [WEBGPU-SAMPLES].

The common `Gray-Scott` setup uses a $3 \times 3$ Laplacian with weights $-1$, $0.2$, and $0.05$, with $\Delta t = 1.0$, $D_A = 1.0$, and $D_B = 0.5$ [SIMS-RD]. Munafo reports that "All projects discussed here have employed the explicit Euler method" [MUNAFO].

Tiles in shared memory with halos cut redundant reads of global memory. Harris gives the example of a $16 \times 16$ tile that reloads half its values [HARRIS-CUDA] (unverified).

These texture formats bear on a simulation that keeps its state in storage textures:

| Platform | Formats | Sources |
| --- | --- | --- |
| WebGPU core | `read_write` storage only for `r32float`, `r32sint`, and `r32uint` | [WEBGPUFUND-STORAGE] (secondary) |
| WebGPU with `texture-formats-tier2` | Adds `read_write` storage for `rgba16float` and `rgba32float` | [MDN-CREATETEXTURE] (secondary) |
| Vulkan | `STORAGE_IMAGE` is mandatory for `R16G16B16A16_SFLOAT` and `R32G32B32A32_SFLOAT` | [VULKAN-DOCS] |
| binary16 | An 11-bit significand | [WP-HALF] (secondary) |

These time-stepping facts were found:

| Topic | Finding | Sources |
| --- | --- | --- |
| Explicit stability | The explicit limit in one dimension is $r \le 1/2$, with $r$ as the source defines it. | [WP-VON-NEUMANN] (secondary) |
| Exponential integrators | They target $u' = Lu + N(u)$. ETDRK4 is covered by Kassam and Trefethen. | [WP-EXPINT] (secondary), [KASSAM-TREFETHEN2005] (not read) |
| Adaptive steps | They use a local error estimate. | [WP-ADAPTIVE] (secondary) |
| Multi-rate or local steps | No source was found for reaction-diffusion. | None |

## 8. Reading Order

The research recommends this reading order. Each entry gives the reason that the research gave:

1. [BULATOV-MAA2025] and the SymmHub shaders in [SYMMHUB-REPO]: the baseline from which the roadmap starts.
2. [WP-POINCARE-METRIC] and [BERARD-WEBB2020]: the conformal Laplacian, invariant functions, and mirror Neumann conditions, in two short reads.
3. [SAUSSET-TARJUS2007]: a practical guide to side pairing in the disk, and the template for ghost cells.
4. [CHOSSAT-FAYE2013]: the only pattern-formation study on a hyperbolic lattice, with both FEM under periodic octagon conditions and time stepping.
5. [CRANE-DDG]: the cotangent Laplacian and DEC, for method (c). The research did not read it.
6. [BOETTCHER2020]: how the graph Laplacian of a tiling approximates $\Delta_g$, for method (e).
7. [KRAUSE2021]: a survey of Turing theory, including curved domains.
8. [MATTHEWS2003]: the template for equivariant pattern selection under finite symmetry groups.
9. [BUCATARU-MANEA]: convergence of finite differences for the hyperbolic Laplacian.
10. [STROHMAIER2016]: spectral methods and FEM on hyperbolic surfaces, and eigenvalue references for validation.

## 9. Conflicts Between Sources

This section records conflicts and does not resolve them.

Whether symmetrization yields true solutions on the orbifold is disputed. Bulatov's slides claim that "The solutions obtained using SymSim procedure are true solutions of the original equations" [BULATOV-MAA2025]. The brief that commissioned the research, and Section 13.1 of `docs/research/symmhub.md`, state that SymmHub symmetrizes the field and does not solve a PDE on the orbifold. The same slides acknowledge interpolation errors and persistent discontinuities, and the research found no independent analysis.

Where the explicit time step tightens on the Poincare disk is also in conflict. The brief and an inference of the research disagree, and neither has a source, so Section 10.1 records both.

The size of the conformal factor differs between sources. The SymmHub code multiplies the diffusion coefficients by $(1 - r^2)^2$, with no division by 4, clamped to 0 outside the disk [SYMMHUB-REPO]. The operator weight is $(1 - \lvert z \rvert^2)^2$ for the curvature $-4$ disk [BOETTCHER2020] [PETERMANN-HINRICHSEN]. The metric of the curvature $-1$ disk [WP-POINCARE-METRIC] is normalized differently, and Section 10.1 records the research's comparison of the resulting factors.

The roles of `o` and `x` differ between MathWorld and Wikipedia. The text extracted from MathWorld appeared to swap them, while [WP-ORBIFOLD-NOTATION] gives `x` as the glide reflection or crosscap and `o` as the handle. The research judged the MathWorld reading to be probably an extraction error.

## 10. Open Questions and Unverified Claims

### 10.1. Candidate Statements

The research made these statements as its own inference, substitution, or comparison, or took them from a fetch-tool reading. No cited source establishes them:

1. Substituting $\lambda = 2/(1 - \lvert z \rvert^2)$ into Section 3.1 gives $\Delta = \frac{(1 - \lvert z \rvert^2)^2}{4} (\partial_x^2 + \partial_y^2)$ on the curvature $-1$ disk.
2. The SymmHub factor $(1 - r^2)^2$ equals $\lambda^{-2}$ in the curvature $-4$ normalization. For curvature $-1$ the factor would be $(1 - r^2)^2/4$, so the difference is a curvature normalization that only rescales $D$ by a constant.
3. On a uniform Euclidean grid on the disk, the effective coefficient $D \lambda^{-2}$ is largest at the centre, so the explicit time step is limited at the centre and not near the boundary. The problem near the boundary is then under-resolution, because the hyperbolic size of a cell grows, and not the time step. The brief that commissioned the research assumed the opposite, that the time step tightens near the boundary. No source states either version. On the half-plane the weight $y^2$ grows with $y$.
4. A conformal chart has the explicit bound $\Delta t \le h^2 \min(\lambda^2)/(4D)$.
5. On the `{7,3}` lattice the adjacency operator satisfies $A \approx 3 + \frac{3}{4} h^2 \Delta_g$, as the fetch tool read [BOETTCHER2020]. This reading needs verification.
6. Small diffusion increments are lost to rounding in binary16, given its 11-bit significand.

### 10.2. Gaps

The research looked for these and found no source:

1. An error analysis of covering-space simulation with periodic group averaging or replacement by folding.
2. A sourced explicit bound for a conformal chart, and the two-dimensional bound $\Delta t \le h^2/(4D)$ in a primary source.
3. The filling of ghost cells or halos across glide-reflection or hyperbolic side pairings in finite differences.
4. A statement that invariant functions are smooth in the orbifold chart at a cone point of angle $2\pi/n$. [KOKOTOV2009] covers general conical points only.
5. A link between the explicit step limit of the cotangent Laplacian and the minimum or obtuse angle of a mesh.
6. A study of Turing, `Gray-Scott`, or Ginzburg-Landau patterns on hyperbolic orbifolds or the pseudosphere, other than the Swift-Hohenberg work of [CHOSSAT-FAYE2013].
7. Bases of spherical harmonics invariant under a point group, and Fourier bases invariant under a wallpaper group.
8. Multi-rate or local time stepping for reaction-diffusion, and stencil tiling specific to compute shaders.
9. The internals of the SymmHub symmetrization and folding shaders in `SymRenderer`, which the research did not read.

### 10.3. Sources Not Read

The research could not read these sources. Each belongs in `docs/research/source-requests.md`, with what is needed and the priority that the research gave:

| Source | Needed | Priority |
| --- | --- | --- |
| [CBG2008] | Orbifold notation, the side pairings of each symbol, and fundamental domains | High |
| [DZIUK-ELLIOTT2013] | Error orders, mass lumping, and time stepping for surface reaction-diffusion | High |
| [CRANE-DDG] | The cotangent formula, the DEC Laplacian, and boundary handling | High |
| [BULATOV-G4G15-PAPER] | Method details, and why simple symmetrization fails for 13 wallpaper groups | High |
| [WARDETZKY2008] | Precise convergence statements and failure modes | Medium |
| [HPW2006] | Conditions for the convergence of the Laplacian | Medium |
| [MATTHEWS2003] | The full equivariant analysis | Medium |
| [BARREIRA2011] | The surface FEM recipe for reaction-diffusion | Medium |
| [KASSAM-TREFETHEN2005] | ETDRK4 for stiff reaction-diffusion | Medium |
| [THURSTON-GT3M] | Orbifold definitions and the local models of cone points | Medium |
| [VAREA1999] | The numerical method and the results on curvature | Low |
| [PLAZA2004] | The results on selection by curvature | Low |
| [BETCKE-TREFETHEN2005] | The accuracy of the MPS, on which [STROHMAIER-USKI2013] builds | Low |
| [ASCHER1995] | Implicit-explicit (IMEX) schemes | Low |
| [HEJHAL1992] | The original automorphic algorithm | Low |

## 11. References

- [ASCHER1995] Ascher, Ruuth, and Wetton, "Implicit-explicit methods for time-dependent PDEs", SIAM Journal on Numerical Analysis 32, 1995. Not read (HTTP 403).
- [ATTAR-BOETTCHER2022] Attar and Boettcher, Physical Review E 106, 2022, <https://arxiv.org/abs/2201.06587>.
- [BARREIRA2011] Barreira, Elliott, and Madzvamuse, "The surface finite element method for pattern formation on evolving biological surfaces", Journal of Mathematical Biology 63, 2011, DOI 10.1007/s00285-011-0401-0. Not read (blocked).
- [BERARD-WEBB2020] Berard and Webb, "One can't hear orientability of surfaces", arXiv 2008.12498, 2020, <https://arxiv.org/abs/2008.12498>.
- [BETCKE-TREFETHEN2005] Betcke and Trefethen, "Reviving the method of particular solutions", SIAM Review 47, 2005, DOI 10.1137/S0036144503437336. Not read (HTTP 403).
- [BOETTCHER2020] Boettcher et al., Physical Review A 102, 032208, 2020, <https://arxiv.org/abs/1910.12318>.
- [BUCATARU-MANEA] Bucataru and Manea, "Discrete Laplacians on the hyperbolic space - a comparative study", arXiv 2409.01211, 2024 to 2026, <https://arxiv.org/abs/2409.01211>.
- [BULATOV-G4G15] Bulatov, V., SymSim slides, G4G15, 2024-02-22, <https://bulatov.org/symsim/240222_G4G15/index.html>.
- [BULATOV-G4G15-PAPER] Bulatov, V., G4G15 exchange paper, 2024. Not read (PDF over 10 MB).
- [BULATOV-JMM2025] Bulatov, V., Bridges 2025 JMM art exhibition gallery, <https://gallery.bridgesmathart.org/exhibitions/2025-jmm-art-exhibition/vladimir-bulatov>.
- [BULATOV-MAA2025] Bulatov, V., SymSim talk at the MAA, 2025-01-08, <https://www.bulatov.org/symsim/250108_MAA/talk/index.html>.
- [CBG2008] Conway, Burgiel, and Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1568812205. Not read (book).
- [CFL2025] arXiv 2510.06653, 2025, on the Courant-Friedrichs-Lewy limit of forward Euler with FEM. Title not recorded.
- [CHAPLAIN2001] Chaplain, Ganesh, and Graham, "Spatio-temporal pattern formation on spherical surfaces", Journal of Mathematical Biology 42, 2001, <https://researchportal.bath.ac.uk/en/publications/spatio-temporal-pattern-formation-on-spherical-surfaces-numerical/>.
- [CHOSSAT-FAYE2013] Chossat and Faye, the Swift-Hohenberg equation on the hyperbolic plane, arXiv 1304.6005, 2013, <https://arxiv.org/abs/1304.6005>.
- [CHOSSAT2011] Chossat, Faye, and Faugeras, "Bifurcation of hyperbolic planforms", Journal of Nonlinear Science, 2011, <https://arxiv.org/abs/1007.2492>.
- [CRANE-DDG] Crane, "Discrete Differential Geometry: An Applied Introduction", Carnegie Mellon University, last updated 2025-01-29, <https://www.cs.cmu.edu/~kmcrane/Projects/DDG/>. Not read (PDF undecodable).
- [CYCLIC2015] "Spectra of orbifolds with cyclic fundamental groups", arXiv 1510.05948, 2015, <https://arxiv.org/abs/1510.05948>.
- [DZIUK-ELLIOTT2013] Dziuk and Elliott, "Finite element methods for surface PDEs", Acta Numerica 22, 289-396, 2013, DOI 10.1017/S0962492913000056. Not read (PDF undecodable, publisher HTTP 403).
- [FARSI2016] Farsi, Proctor, and Seaton, arXiv 1611.07676, 2016. Title not recorded.
- [FRANK2019] Frank, Guven, Kardar, and Shackleton, EPL 127, 2019, <https://arxiv.org/abs/1901.09900>.
- [FREEFEM-BLOCH] Hecht, Bloch boundary conditions thread, FreeFEM forum, 2020-09-01, <https://community.freefem.org/t/bloch-boundary-conditions/280>.
- [FREEFEM-PERIODIC] FreeFEM forum, "Eliminate degrees of freedom using periodic fespace", <https://community.freefem.org/t/eliminate-degrees-of-freedom-using-periodic-fespace/4238>.
- [HARRIS-CUDA] Harris, finite difference methods in CUDA C/C++, part 1, NVIDIA blog, <https://developer.nvidia.com/blog/finite-difference-methods-cuda-cc-part-1/>.
- [HARRIS2004] Harris, fast fluid dynamics simulation on the GPU, GPU Gems, chapter 38, 2004, <https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu>.
- [HEJHAL1992] Hejhal, "Eigenvalues of the Laplacian for Hecke triangle groups", Memoirs of the AMS 469, 1992. Not read (not found online).
- [HPW2006] Hildebrandt, Polthier, and Wardetzky, "On the convergence of metric and geometric properties of polyhedral surfaces", Geometriae Dedicata 123, 2006. Not read (blocked).
- [HYPERBLOCH] HyperCells, "HyperBloch" GitHub repository, <https://github.com/HyperCells/HyperBloch>.
- [HYPERROGUE] zenorogue, "hyperrogue" GitHub repository, <https://github.com/zenorogue/hyperrogue>.
- [HYPERTILING2024] Schrauth et al., HYPERTILING, SciPost Physics Codebases 34, 2024, <https://arxiv.org/abs/2309.10844>.
- [ISOTROPY2019] arXiv 1905.04485, 2019. Title not recorded.
- [KASSAM-TREFETHEN2005] Kassam and Trefethen, "Fourth-order time-stepping for stiff PDEs", SIAM Journal on Scientific Computing 26, 2005. Not read (PDF undecodable).
- [KOKOTOV2009] Kokotov, "Polyhedral surfaces and determinant of Laplacian", arXiv 0906.0717, 2009, <https://arxiv.org/abs/0906.0717>.
- [KOLLAR2019] Kollar, Fitzpatrick, and Houck, Nature 571, 2019, <https://arxiv.org/abs/1802.09549>.
- [KRAUSE2019] Krause, Ellis, and Van Gorder, Bulletin of Mathematical Biology 81, 2019, <https://pmc.ncbi.nlm.nih.gov/articles/PMC6373535>.
- [KRAUSE2021] Krause, Gaffney, Maini, and Klika, arXiv 2106.08375, 2021, <https://arxiv.org/abs/2106.08375>.
- [LUX-PRODAN2023] Lux and Prodan, Physical Review Letters 131, 2023, <https://arxiv.org/abs/2303.15611>.
- [MACIEJKO-RAYAN2021] Maciejko and Rayan, Science Advances 7, eabe9170, 2021, <https://arxiv.org/abs/2008.05489>.
- [MARKLOF2004] Marklof, "Selberg's trace formula: an introduction", arXiv math/0407288, 2004, <https://arxiv.org/abs/math/0407288>.
- [MATTHEWS2003] Matthews, "Pattern formation on a sphere", Physical Review E 67, 036206, 2003. Abstract only (paywall).
- [MDN-CREATETEXTURE] MDN, `GPUDevice.createTexture` page, <https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/createTexture>.
- [MEMBRILLO-SOLIS2025] Membrillo Solis et al., arXiv 2512.16763, 2025, <https://arxiv.org/abs/2512.16763>.
- [MUNAFO] Munafo, Xmorphia page, <https://www.mrob.com/pub/comp/xmorphia/index.html>.
- [NAKAO-MIKHAILOV2010] Nakao and Mikhailov, Nature Physics, 2010, <https://arxiv.org/abs/1005.1986>.
- [NAMPOOTHIRI2017] Nampoothiri and Medhi, arXiv 1705.02119, 2017. Title not recorded.
- [NISHIDE-ISHIHARA2024] Nishide and Ishihara, arXiv 2403.12442, 2024.
- [PETERMANN-HINRICHSEN] Petermann and Hinrichsen, "Eigenmodes of the Laplacian on Hyperbolic Lattices", arXiv 2306.08248, 2023, Physical Review D 109, 2024, <https://arxiv.org/abs/2306.08248>.
- [PLAZA2004] Plaza, Sanchez-Garduno, Padilla, Barrio, and Maini, "The effect of growth and curvature on pattern formation", Journal of Dynamics and Differential Equations 16, 2004. Not read (paywall).
- [SAUSSET-TARJUS2007] Sausset and Tarjus, "Periodic boundary conditions on the pseudosphere", Journal of Physics A 40, 2007, <https://arxiv.org/abs/cond-mat/0703326>.
- [SIMS-RD] Sims, reaction-diffusion page, undated, <https://karlsims.com/rd.html>.
- [STROHMAIER-USKI2013] Strohmaier and Uski, Communications in Mathematical Physics 317, 2013, <https://arxiv.org/abs/1110.2150>.
- [STROHMAIER2016] Strohmaier, arXiv 1604.02722, 2016. Title not recorded.
- [STROMBERG2006] Stromberg, arXiv math/0611146, 2006.
- [SYMMHUB-REPO] SymmHub, "SymmHub" GitHub repository, main branch, <https://github.com/SymmHub/SymmHub>.
- [SYMMHUB-SITE] SymmHub, project site, <https://symmhub.github.io/SymmHub/>.
- [SYMSIM-SITE] Bulatov, V., SymSim, <https://www.bulatov.org/symsim/>.
- [THEN2005] Then, Mathematics of Computation 74, 2005, <https://arxiv.org/abs/math-ph/0305047>.
- [THURSTON-GT3M] Thurston, "The Geometry and Topology of Three-Manifolds", chapter 13, <https://library.slmath.org/nonmsri/gt3m/PDF/13.pdf>. Not read (PDF undecodable).
- [VAREA1999] Varea, Aragon, and Barrio, "Turing patterns on a sphere", Physical Review E 60, 4588, 1999, <https://journals.aps.org/pre/abstract/10.1103/PhysRevE.60.4588>. Abstract only (paywall).
- [VULKAN-DOCS] Khronos Group, "Vulkan-Docs", `formats.adoc`, main branch, <https://github.com/KhronosGroup/Vulkan-Docs>.
- [WARDETZKY2008] Wardetzky, "Convergence of the cotangent formula: an overview", in "Discrete Differential Geometry", Oberwolfach Seminars 38, Birkhauser, 2008. Not read (HTTP 429).
- [WEBGPU-SAMPLES] WebGPU, "webgpu-samples" GitHub repository, main branch, <https://github.com/webgpu/webgpu-samples>.
- [WEBGPUFUND-STORAGE] WebGPU Fundamentals, "Storage Textures", <https://webgpufundamentals.org/webgpu/lessons/webgpu-storage-textures.html>.
- [WP-ADAPTIVE] Wikipedia, "Adaptive step size", <https://en.wikipedia.org/wiki/Adaptive_step_size>.
- [WP-DISCRETE-LAPLACE] Wikipedia, "Discrete Laplace operator".
- [WP-EXPINT] Wikipedia, "Exponential integrator", <https://en.wikipedia.org/wiki/Exponential_integrator>.
- [WP-FUNDAMENTAL-POLYGON] Wikipedia, "Fundamental polygon", <https://en.wikipedia.org/wiki/Fundamental_polygon>.
- [WP-HALF] Wikipedia, "Half-precision floating-point format", <https://en.wikipedia.org/wiki/Half-precision_floating-point_format>.
- [WP-LAPLACE-BELTRAMI] Wikipedia, "Laplace-Beltrami operator", <https://en.wikipedia.org/wiki/Laplace%E2%80%93Beltrami_operator>.
- [WP-ORBIFOLD] Wikipedia, "Orbifold", <https://en.wikipedia.org/wiki/Orbifold>.
- [WP-ORBIFOLD-NOTATION] Wikipedia, "Orbifold notation", <https://en.wikipedia.org/wiki/Orbifold_notation>.
- [WP-POINCARE-DISK] Wikipedia, "Poincare disk model", <https://en.wikipedia.org/wiki/Poincar%C3%A9_disk_model>.
- [WP-POINCARE-METRIC] Wikipedia, "Poincare metric", <https://en.wikipedia.org/wiki/Poincar%C3%A9_metric>.
- [WP-RIEMANN-SPHERE] Wikipedia, "Riemann sphere", <https://en.wikipedia.org/wiki/Riemann_sphere>.
- [WP-VON-NEUMANN] Wikipedia, "Von Neumann stability analysis", <https://en.wikipedia.org/wiki/Von_Neumann_stability_analysis>.
