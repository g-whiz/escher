# Aperiodic Tilings Survey

| Field | Value |
| --- | --- |
| Status | Snapshot |
| Date | 2026-11-15 |
| Applies to | Roadmap planning of graphics based on aperiodic and other non-group planar tilings |

## Abstract

This document surveys aperiodic tilings of the plane for the people and agents who write Escher's roadmap. It covers the main families and catalogues, the construction methods and what each needs computationally, rendering, the structure and colourings that a tool could expose, the analogues on the hyperbolic plane and the sphere, and the existing software with its language, licence, and state. It closes with the research's reading order, the conflicts between sources, and the open questions. Three findings frame the size of the step. First, in the research's assessment these tilings are a different kind of problem from the tilings of a symmetry group, because no group and fundamental domain describe them, so each family needs its own generator. Second, the research found closed-form per-pixel point location for multigrid tilings such as Penrose and recursive-descent shaders for Penrose, and it found no published method for the hat or the spectre. Third, in Rust the research found Tatham's crate `substitution-tiling-transducers`, which covers Penrose, hat, and spectre, and a few smaller crates.

## 1. Scope

This document is a survey for the roadmap and not an implementation reference. It gives each definition in a sentence, the key formula or data structure, and the sources. It does not specify any algorithm in the detail that code needs, and it makes no recommendation about what Escher should build. Where it states how hard a step is or where to start, it reports the research's assessment.

This document does not cover the tilings of discrete symmetry groups, which `docs/research/orbifold-notation.md`, `docs/research/fundamental-domains.md`, and `docs/research/rendering.md` cover. It does not cover the choice of Rust crates beyond listing the aperiodic-tiling software that exists.

## 2. Notation and Conventions

### 2.1. Terminology

- aperiodic set of tiles: A set of prototiles that admits tilings of the plane but no periodic tiling. For a single prototile, Socolar and Taylor phrase it as a tile that "can fill space uniformly but not admit a periodic tiling" [SOCOLAR-TAYLOR-2010].
- einstein: "A shape that admits tilings of the plane, but never periodic tilings" [SMITH-HAT-2023].
- substitution (inflation and deflation): The construction that replaces each prototile by its children, through a tree or recursion over prototile types with an affine map for each child [FRANK-2007].
- substitution matrix: The matrix $M$ in which $M_{ij}$ counts the tiles of type $i$ in the substituted tile of type $j$ [FRANK-2007].
- supertile and metatile: A tile of a higher level of the substitution hierarchy, as the sources use the terms. The hat paper uses metatiles, and the spectre paper uses clusters [SMITH-HAT-2023] [SMITH-SPECTRE-2023].
- matching rules: Decorations on edges or vertices, such as arrows, arcs, and Ammann bars [TE-AMMANN-BARS] (secondary). For the Penrose sets they force non-periodicity [TE-PENROSE].
- mutually locally derivable (MLD): A relation that groups tilings into classes. The Penrose variants "generate the same mld-class" [TE-PENROSE]. The report gives no formal definition, which Baake and Grimm would supply (Section 12.3).
- repetitivity: Every patch $P$ recurs in every ball of some radius $R(P)$. Frank notes the term is "also called almost periodic or the local isomorphism property" [FRANK-2007].
- tiling space: Frank's term for the hull [FRANK-2007].
- model set: The point set that cut-and-project produces (Section 5.2) [TE-CUT-PROJECT] (secondary).
- combinatorial coordinates: The address of a tile as its sequence of labels up the substitution hierarchy [TATHAM-2023].
- finite-state transducer: A machine that maps the address of a tile and one of its edges to the address of the neighbouring tile [TATHAM-TRANSDUCERS-2024].
- weakly and strongly aperiodic: Section 8.1 defines these terms for the hyperbolic plane.

### 2.2. Symbols

- $\phi$: The golden ratio, about 1.618 [TE-PENROSE]. The Frank quotation in Section 5.1 uses the same letter for an object that the report does not define (Section 12.2).
- $\lambda$: The linear expansion factor of a substitution.
- $\gamma_j$: The offset of grid $j$ in a multigrid.
- $e_j$: The direction vector of grid $j$ in the general multigrid form. The report does not define it further (Section 12.2).
- $\zeta$: The complex number that the pentagrid formulas of [DEBRUIJN-SURVEY-2013] rotate by. The research report does not define it (Section 12.2).
- $K_j$: The integer mesh index of a point in grid $j$.
- $L$, $G$, $H$, $W$: The lattice in $G \times H$ and the window of a cut-and-project scheme, with the projections $\pi_G$ and $\pi_H$ onto $G$ and $H$.

### 2.3. Source Conventions

All facts come from one research report. This document calls it the report, calls its author the searcher, and calls its judgements the research's assessment. Every source was accessed on 2026-11-15. The report reached many quotations through a summarizing fetch tool, so a quotation is close to the source but not guaranteed verbatim. The report rated its overall confidence as medium, because its search budget ran out before it could read the key books, de Bruijn's paper, and every Shadertoy source.

Each reference entry in Section 13 carries the report's mark. "Primary" is a paper, the author's own page, source code, or a package registry. "Secondary" is Wikipedia, a blog, or a survey of someone else's work. "Not read" means that the report gives the citation but nobody read the source. A claim in the body that rests only on a secondary source says "(secondary)". The report's own inferences are not stated as facts and appear only in Section 12.

Names that contain accented letters are written in ASCII transliteration: Boroczky, Frettloh, Gahler, Grunbaum, and de las Penas.

## 3. Why This Is a Different Problem

The research assesses that aperiodic tilings are a different kind of problem from a new symmetry group. Escher's model folds each point into a fundamental domain of a group, and an aperiodic tiling has no such group, so that model does not carry over.

The research also assesses that each family needs its own generator, which is a substitution hierarchy, a multigrid or cut-and-project scheme, or both. Since 2023 the hat and spectre also need metatile substitution systems. Per-pixel evaluation works for some families only (Section 6.2). In Rust, the research names Simon Tatham's crate `substitution-tiling-transducers` (Section 9) as the natural starting point. This is a judgement of the research, and this document makes no recommendation.

## 4. Families and Catalogues

### 4.1. Families

| Family | Definition | Expansion factor as sourced | Sources |
| --- | --- | --- | --- |
| Penrose P1, P2, P3 | Penrose's 1973-74 sets: 6 pentagon-based prototiles (P1), kite and dart (P2), and thick and thin rhombs (P3). Matching rules force non-periodicity. All three are MLD. | $\phi$ | [TE-PENROSE], [WP-PENROSE] (secondary) |
| Ammann-Beenker (Ammann A5) | A square and a 45-degree rhomb with "perfect 8fold symmetry". Ammann found it in 1977, and Beenker gave the algebraic and projection treatment in 1982. | $1 + \sqrt{2}$, the silver mean, with a Pell substitution matrix | [TE-AB], [BEENKER-1982] (not read) |
| Socolar-Taylor | A single hexagonal prototile with matching rules. The tile is not connected. | Not stated. The abstract describes honeycombs with lattice constants $2^n a$. | [SOCOLAR-TAYLOR-2010] |
| Hat, turtle, and the Tile(a,b) continuum | The hat is an 8-kite polykite and the first einstein. Tile(a,b) varies the two edge lengths: the hat is Tile($1$, $\sqrt{3}$) and the turtle is Tile($\sqrt{3}$, $1$). The chevron and the comet are the degenerate endpoints. | $\phi^2$ linear, on the H, T, P, and F metatiles, through a summarizing tool with no verbatim quote | [SMITH-HAT-2023], [HAT-PAGE] |
| Spectre | An edge-modified Tile(1,1). "Strictly chiral aperiodic monotiles" that "admit only homochiral non-periodic tilings based on a hierarchical substitution system". | Area grows by $4 + \sqrt{15}$ per stage. No linear factor is stated. | [SMITH-SPECTRE-2023], [SPECTRE-PAGE] |
| Pinwheel | Radin's tiling by the right triangle with sides 1, 2, and $\sqrt{5}$, "the example of substitution tilings with infinite rotations". | Unconfirmed (Section 11) | [RADIN-1994] (not read), [TE-PINWHEEL] |
| Chair | A substitution tiling by the L-tromino. It is non-periodic but limit-periodic. A two-tile matching-rule set exists (Goodman-Strauss 1999). | Not stated in the sources read | [TE-CHAIR], [WP-CHAIR] (secondary) |

### 4.2. Catalogues

The Tilings Encyclopedia by Frettloh, Gahler, and Harriss at Bielefeld shows "a wealth of examples of nonperiodic substitution tilings", and its entries can be filtered by number of prototiles [TE]. It was online on 2026-11-15, shows no update date, offers no data downloads, and carries a CC BY-NC-SA 2.0 badge in its footer.

Wikipedia's "List of aperiodic sets of tiles" is a secondary catalogue [WP-APERIODIC-LIST]. The report did not verify the cells of its table.

## 5. Construction Methods

### 5.1. Substitution

A substitution tiling grows from a tree or recursion over prototile types, with an affine map for each child. For a primitive substitution matrix $M$, "the Perron eigenvalue of M is the volume expansion" $\lvert \det \phi \rvert$, and "the Perron left eigenvector gives the relative volumes of the prototiles" [FRANK-2007]. The Penrose kite and dart have this substitution matrix [WP-PENROSE] (secondary):

$$M = \begin{pmatrix} 2 & 1 \\ 1 & 1 \end{pmatrix}$$

### 5.2. Cut-and-Project

A cut-and-project scheme takes a lattice $L$ in $G \times H$ and a window $W$, and keeps the projections to $G$ of the lattice points whose projection to $H$ falls in $W$ [TE-CUT-PROJECT] (secondary):

$$V = \{ \pi_G(x) : x \in L, \ \pi_H(x) \in W \}$$

The Penrose tiling comes from the lattice $\mathbb{Z}^5$. In Egan's applet, "square faces ... are projected down to tiles if all four of their vertices are selected" [EGAN-APPLET].

### 5.3. De Bruijn Pentagrid and Multigrid

De Bruijn's pentagrid is five grids $G_0$ to $G_4$, and each rhomb of the Penrose tiling is dual to a grid-line intersection. The formulas below come from a 2013 survey of de Bruijn's work, because de Bruijn's 1981 paper was not read [DEBRUIJN-SURVEY-2013], Section 7.6 (secondary), [DEBRUIJN-1981] (not read). The grids are these, for $j = 0, \dots, 4$:

$$G_j = \{ z : \operatorname{Re}(z \zeta^{-j}) + \gamma_j \in \mathbb{Z} \}$$

A Penrose tiling needs $\sum_j \gamma_j = 0$. A pentagrid is regular when no three lines meet. The mesh index and the tile vertex of a point $z$ are these:

$$K_j(z) = \lceil \operatorname{Re}(z \zeta^{-j}) + \gamma_j \rceil, \qquad f(z) = \sum_{j=0}^{4} K_j(z) \zeta^j$$

The report gives the general $n$-fold form as $K_j(x) = \lceil x \cdot e_j + \gamma_j \rceil$, with the tile vertex $\sum_j K_j(x) e_j$. Egan's notes treat general $n$ [EGAN-DEBRUIJN]. Munro states that Ammann-Beenker, Penrose, and Socolar tilings "can be represented by multigrids" [MUNRO] (secondary). The report takes $n = 4$ for Ammann-Beenker and $n = 6$ for Socolar. Gahler and Rhyner prove the grid and projection methods equivalent [GAHLER-RHYNER-1986] (metadata only).

### 5.4. Matching Rules

Matching rules decorate edges or vertices with arrows, arcs, or Ammann bars. Ammann bars join into "a grid of straight lines extending over the whole tiling" [TE-AMMANN-BARS] (secondary).

### 5.5. Combinatorial Coordinates and Transducers

Tatham identifies each tile by its labels up the hierarchy: "each tile of the output can be uniquely identified by the sequence of labels it generated as it was expanded" [TATHAM-2023]. Higher labels are invented lazily. A transducer maps an address and an edge to the neighbour's address [TATHAM-TRANSDUCERS-2024].

The spectre transducer has 151 states, and the Penrose P2 transducer has 31 [TATHAM-ARXIV-2026]. The hat's HTPF system does not give a finite transducer. Refined systems succeed, namely HHTPFFF and the 10-hexagon Klee-Ping system [TATHAM-REFINE-2025] [TATHAM-ARXIV-2026].

Tatham reports an average cost per tile "proportional to the log of the number of output tiles", and "essentially linear time and essentially constant space" for large spectre patches [TATHAM-TRANSDUCERS-2024] [TATHAM-FOLLOWUP-2024].

### 5.6. Hat and Spectre Metatiles

The hat has four metatiles, H, T, P, and F, which hold 1, 2, or 4 hats each [SMITH-HAT-2023]. The H, T, and P metatiles have rotational symmetries. The spectre substitution uses a two-spectre "Mystic" plus 9 hexagon-equivalent cluster types, and one spectre becomes a "Spectre cluster" of a Mystic and seven spectres [SMITH-SPECTRE-2023] [TATHAM-SPECTRE-2023].

Tatham notes that geometric patch methods work "rather badly for hats because of the distortion problem", so he prefers combinatorial methods [TATHAM-SPECTRE-2023].

### 5.7. Summary of Computational Needs

| Method | Data structure | Computational need as the report states it |
| --- | --- | --- |
| Substitution | Prototile types, affine map per child, matrix $M$ | Recursion over the hierarchy |
| Cut-and-project | Lattice, window | Lattice enumeration plus a window test |
| Multigrid | $n$ grid directions and offsets | Closed form per point, intersection enumeration per patch |
| Matching rules | Edge and vertex decorations | Verification and decoration |
| Transducers | Label sequences, state tables | Logarithmic average cost per tile (Section 5.5) |
| Hat and spectre metatiles | Small rule table | Geometric methods suffer from distortion |

## 6. Rendering

### 6.1. Forward Generation

Forward generation repeatedly subdivides a seed patch, such as the wheel of Robinson triangles subdivided 10 times [PRESHING-2011] (secondary). Reusser's instanced WebGPU renderer reaches "~17 M tiles at depth 8" for the spectre [REUSSER-RENDER-2026]. The tile count per level is not stated by any source (Sections 12.1 and 12.2).

### 6.2. Per-Pixel Point Location

This table states which methods can decide the tile that contains a point without generating a patch around it.

| Method | Decides the tile of a point directly | Evidence |
| --- | --- | --- |
| Multigrid dual: Penrose, Ammann-Beenker, Socolar | Yes. The $K_j$ formulas are closed form per point. No published per-pixel pentagrid shader was found, so the method is implied by the formulas. | [DEBRUIJN-SURVEY-2013], Section 7.6 (secondary). Munro: "f, like the n functions K_j, is constant within every polygonal region of the multigrid" [MUNRO] (secondary). |
| Recursive descent through a geometric substitution | Yes. Descent keeps only the child that contains the point, with a bounded number of iterations. | Panzerschrek's GLSL shader for P3 rhombs uses 7 iterations [PANZERSCHREK]. A commercial WGSL "endless Penrose floor" states that "each pixel takes at most ten small steps", at "about 0.45 ms" at 1280 by 720 [SHADCN] (secondary, source not seen). |
| Hat and spectre | Not established. No published shader found. Imperfect metatile geometry ("distortion") and supertile boundaries of "infinite order" complicate naive descent. | [TATHAM-2023], [TATHAM-REFINE-2025]. A GitHub search for hat, spectre, and Ammann-Beenker shaders found none. |
| Combinatorial coordinates | Used to generate a region on demand. Not documented for per-pixel lookup. | Tatham: "generate a patch of tiling that occupies a given region of the plane by generating each tile on demand" [TATHAM-REFINE-2025]. |

The report notes that the absence of a hat, spectre, or Ammann-Beenker shader is weak evidence, because its search was limited.

### 6.3. Zoom and Level of Detail

A Penrose recursive-descent zoom by $\phi$ is seamless: "after two of those steps the floor is exactly where it started, mirrored" [SHADCN] (secondary).

Reusser's "Zooming Aperiodic Monotile" in WebGPU works this way [REUSSER-ZOOM-2026]:

- "Supertiles are generated on demand", and "zooming out wraps the root in a new `Δ` parent".
- For level of detail, "tiles that shrink below a few pixels collapse into instanced hexagons".
- Hierarchy colouring uses "a Gaussian kernel as a function of log-zoom".
- Zoom is limited "to the extent of floating point precision", and no rebasing is described.

For unbounded panning, Tatham notes that an infinite tiling gives each tile an infinite coordinate sequence. Transducers avoid the "infinitely high wall" that recursive coordinate transitions hit [TATHAM-TRANSDUCERS-2024].

### 6.4. Published Renderers

| Renderer | Technique | Source |
| --- | --- | --- |
| Reusser, "Rendering the Aperiodic Monotile" | Instanced WebGPU forward generation of the spectre | [REUSSER-RENDER-2026] |
| Reusser, "Zooming Aperiodic Monotile" | On-demand supertiles, instancing, hexagon level of detail | [REUSSER-ZOOM-2026] |
| Panzerschrek, `penrose_tiling.glsl` | Per-pixel recursive descent, P3 rhombs | [PANZERSCHREK] |
| "Endless Penrose floor" (commercial, WGSL) | Per-pixel recursive descent | [SHADCN] (secondary) |
| `necocen/spectre` | Rust and wgpu, scroll and zoom, also in the browser. Technique not documented. | [NECOCEN-SPECTRE] (secondary) |
| Shadertoy "Penrose Diagram - 5D Projection" and "Impossible Aperiodic Tilings" by neozhaoliang | Not read (HTTP 403) | [SHADERTOY-F3S3DY], [SHADERTOY-NEOZHAOLIANG] |

## 7. Structure and Colouring

### 7.1. Structure a Tool Could Expose

- Local indistinguishability and repetitivity: Frank defines repetitivity as in Section 2.1 [FRANK-2007]. Penrose rhombs admit infinitely many tilings that cannot be told apart locally [WP-APERIODIC] (secondary).
- Global rotational symmetry: "There are only two Penrose tilings (of each type) with global pentagonal symmetry", and the sun and star vertices carry local 5-fold symmetry [WP-PENROSE] (secondary). No source was found for the "cartwheel" patch.
- Inflation symmetry and MLD: The Penrose variants share one MLD class [TE-PENROSE]. Hat tilings and hat metatile tilings are MLD, "with the resulting tiling by metatiles having the same symmetries as the original tiling" [SMITH-HAT-2023].

### 7.2. Colouring Conventions

| Convention | Example | Sources |
| --- | --- | --- |
| By tile type | Three colours for the three P1 pentagon types | [WP-PENROSE] (secondary) |
| By reflection or chirality | Reflected hats in dark blue, "always distributed sparsely" | [SMITH-HAT-2023] |
| By hierarchy or metatile | Metatile outlines in lime, F-metatile hats in black, and "the Mystic and its supertiles are shaded in progressive levels of green" | [SMITH-HAT-2023], [SMITH-SPECTRE-2023] |
| By hierarchy level as a function of zoom | Gaussian weight on log-zoom | [REUSSER-ZOOM-2026] |
| By matching-rule decoration | Arrow and arc markings, two-colour vertices, Ammann bars | [FRANK-2007], [WP-PENROSE] (secondary) |
| Perfect colourings | Colourings of cyclotomic integer rings, applied to quasicrystals | [BUGARIN-2009] (abstract only) |

## 8. Non-Euclidean Analogues

### 8.1. Hyperbolic Plane

The report attributes to Mozes a distinction between two kinds of aperiodic tile set [MOZES-1989] (not read). A weakly aperiodic set admits only tilings "without a co-compact symmetry", which in the hyperbolic plane is "an almost trivial property". A strongly aperiodic set admits only tilings "with no period whatsoever".

| Result | Kind | Sources |
| --- | --- | --- |
| Boroczky's 1974 binary tiling, a single tile | Weakly aperiodic | [WP-BINARY] (secondary) |
| Penrose's hyperbolic tiling, whose C*-algebras Oyono-Oyono and Petite study | Weakly aperiodic | [OYONO-PETITE-2009] |
| Goodman-Strauss, 2005 | Strongly aperiodic set of tiles | [GOODMAN-STRAUSS-2005] (abstract not read) |
| Goodman-Strauss, hierarchical construction with an "n-fold horocyclic tiling" | Strongly aperiodic | [GOODMAN-STRAUSS-HIER], read through a mirror |
| Margenstern: the domino problem is undecidable in the hyperbolic plane | Decidability | [MARGENSTERN-2007], [MARGENSTERN-2022] |
| Cohen and Goodman-Strauss: strongly aperiodic subshifts of finite type on hyperbolic surface groups | Strongly aperiodic | [COHEN-GS-2015] |
| Boyle and Kulp: self-similar quasicrystals on the boundary of hyperbolic honeycombs, not tilings of the hyperbolic plane | Boundary quasicrystals | [BOYLE-KULP-2024] |

The research found no rendering literature for hyperbolic substitution tilings comparable to the planar case, and no hyperbolic aperiodic monotile or hyperbolic hat analogue from 2023 to 2026.

### 8.2. Sphere

The research found no established notion of aperiodicity on the sphere. The nearest work is Prokhoda's "limit series of Penrose type partitions" of the sphere with icosahedral symmetry $I_h$, a single-author preprint in Russian that the report read as an abstract only [PROKHODA-2018].

## 9. Software and Libraries

Every state in this table is as of 2026-11-15.

| Name | Scope | Language | Licence | State | Source |
| --- | --- | --- | --- | --- | --- |
| `substitution-tiling-transducers` (Tatham) | General substitution tilings, with TOML definitions for Penrose, hat, and spectre. Combinatorial coordinates and transducers. | Rust | MIT | v0.2.2, 2026-09-27, 88 downloads, early | [STT-CRATE] |
| Tatham `tilings` | Reference code for the blog series | Sage (Python) | Not stated | No dates found | [TATHAM-TILINGS-REPO] |
| Tatham's Puzzles, Loopy grids | Hat and spectre grids | C (general knowledge of the searcher) | MIT | Page modified 2026-09-23 | [TATHAM-PUZZLES] |
| Kaplan `hatviz` | Hat tiling web app | JavaScript (p5.js) | BSD-3-Clause | Last push 2023-06-26 | [HATVIZ] |
| Kaplan spectre app | Spectre web app | JavaScript | No public source repository found. CC BY 4.0 covers the images only. | Online | [SPECTRE-PAGE] |
| Kaplan Tactile-JS | Periodic isohedral tilings only, not aperiodic | JavaScript | BSD-3-Clause | 2024-08-08 | [TACTILE-JS] |
| `spectre-explorer` (Andrej Bauer) | Spectre, hat, and turtle as SVG, with cropping, colouring, and measuring. Ports Kaplan's `spectre.js` and `h7h8.js`. | Python | Not captured | Pushed 2026-07-31 | [SPECTRE-EXPLORER] (secondary) |
| `hat-amp` | Hat, spectre, and Penrose patches, graphs, SVG, and PNG. Ported from `hatviz`. | Python | Conflicting (Section 11) | 0.2.0, 2026-06-06 | [HAT-AMP] |
| `shrx/spectre` | Spectre script | Python | No licence | 2024-02-12 | [SHRX-SPECTRE] |
| `necocen/spectre` | Spectre viewer with zoom and scroll | Rust and wgpu | Not captured | 2026-11-13 | [NECOCEN-SPECTRE] (secondary) |
| `amosaic` | Hat mosaics | Rust | CC-BY-NC-4.0 (non-commercial) | 1.0.0, 2025-07-02, placeholder repository URL | [AMOSAIC] |
| `tilezz` | Exact cyclotomic-integer polygon tiles. Infrastructure, not a generator. | Rust | MIT | 0.2.0, 2026-07-20 | [TILEZZ] |
| `lau-penrose-growth`, `penrose-lattice` | Vague or spectral-graph Penrose crates | Rust | MIT, not captured | 2026, very low use, low quality, unverified | [CRATES-PENROSE] |
| `slabbe` `MonotileSolver` | Tiles rectangles with the monotile through exact cover, SAT, or MILP. A search tool. | SageMath | GPLv2+ | 0.8.0, 2025-04-03 | [SLABBE] |
| `gautamra/penrose`, `clausm0n/penrose_generator`, `YingHongTham/penrosebud` | Pentagrid Penrose generators that enumerate intersections | Python, Python, JavaScript | Not captured | 2024, 2026, 2021 | [PENTAGRID-REPOS] (secondary) |
| Panzerschrek `shader_bits` | Per-pixel P3 shader | GLSL | Not captured | Undated | [PANZERSCHREK] |

## 10. Reading Order

The research recommends this reading order:

1. Frank's primer, a short, free introduction to substitution matrices, Perron-Frobenius, repetitivity, and the tiling space [FRANK-2007].
2. The survey of de Bruijn's work, Sections 4 and 7.6, for the pentagrid formulas that a per-pixel Penrose renderer needs [DEBRUIJN-SURVEY-2013].
3. Tatham's first blog post, which the research calls the clearest practical account of generating hats and Penrose tilings for a region on demand [TATHAM-2023].
4. The hat paper, for the hat, Tile(a,b), the metatiles, and the substitution [SMITH-HAT-2023].
5. The spectre paper, for the spectre, the Mystic, the nine cluster types, and the hierarchy colouring [SMITH-SPECTRE-2023].
6. Tatham's arXiv paper with the "Refinable Frontier" post, for the formal algorithms behind the Rust crate, including unambiguous hat systems [TATHAM-ARXIV-2026] [TATHAM-REFINE-2025].
7. Reusser's zooming notebook, a working WebGPU design for on-demand supertiles, instancing, and hexagon level of detail, which the research calls close to a wgpu stack [REUSSER-ZOOM-2026].
8. Baake and Grimm, Volume 1, the standard reference for local indistinguishability, MLD, model sets, and the examples. Its Chapter 5 covers patterns and local indistinguishability [BAAKE-GRIMM-2013].
9. The Tilings Encyclopedia, the catalogue of substitution rules to implement and test against [TE].
10. Grunbaum and Shephard, the classical treatment of Penrose tilings, Ammann tilings, and matching rules, in Chapters 10 and 11 by the searcher's brief, with the chapter numbers unverified [GRUNBAUM-SHEPHARD-1987].

## 11. Conflicts Between Sources

| Topic | One source says | Another source says | Treatment |
| --- | --- | --- | --- |
| Pinwheel scale factor | Wikipedia's extracted text reads "a factor of 5" | The tile is the 1, 2, $\sqrt{5}$ triangle, and the Tilings Encyclopedia gives no factor [TE-PINWHEEL] | The report judges $\sqrt{5}$ likely, with the radical lost in extraction. Unconfirmed until Radin 1994 is read. |
| Baake and Grimm, Volume 1, date | Open Library gives 2008 (ISBN 9780521869911) | Crossref gives 2013-08-22 | The report uses Crossref. |
| Grunbaum and Shephard date and edition | Wikipedia cites 1986 | Wikipedia also cites 1987 | Unresolved. Whether the Dover 2016 printing is a second edition is unconfirmed. |
| `hat-amp` licence and Python version | GitHub reports MIT, and the README requires Python 3.13 or later | PyPI states no licence, and the metadata requires 3.11 or later | Both are recorded [HAT-AMP]. |
| "Weakly aperiodic" | Wikipedia calls the 3D Socolar-Taylor extension "only weakly aperiodic", in a Euclidean sense | Section 8.1 uses the term for the hyperbolic plane | A difference in definition context, not a factual conflict. |

## 12. Open Questions and Unverified Claims

### 12.1. Open Questions

1. Is there a published per-pixel point-location method for the hat or the spectre, and could Tatham's transducers run per pixel on a GPU?
2. What is the tile count per substitution level? The report found no explicit formula beyond Frank's volume-expansion theorem.
3. What are the linear inflation factors of the spectre, for which only the area factor $4 + \sqrt{15}$ is stated, and of the chair and the pinwheel?
4. How should a renderer keep precision at deep zoom? Reusser stops at the limit of floating-point precision, and no source found covers exact cyclotomic arithmetic, as in `tilezz`, or hierarchical rebasing.
5. Does Ammann-Beenker have a documented per-pixel multigrid shader for $n = 4$, and what is its offset condition?
6. Does any source state formally that aperiodicity is meaningless on the sphere, and are there spherical substitution analogues beyond Prokhoda's preprint?
7. Do hyperbolic substitution tilings, such as Penrose's hyperbolic tiling and the binary tiling, have rendering-oriented descriptions that could reuse Escher's hyperbolic machinery?
8. What licence does Tatham's Sage `tilings` repository use, and what is the commit history of `tilings-rs`? The host `git.tartarus.org` returned HTTP 403.

### 12.2. Unverified Claims

These claims are the report's own inferences or rest on no source that the report read:

- That exact arithmetic in $\mathbb{Z}[\phi]$ or $\mathbb{Z}[\sqrt{2}]$ avoids drift in substitution generation (inferred).
- That matching rules are a poor generator, because local growth can dead-end (not sourced).
- That the tile count grows by about the Perron eigenvalue per level, which is $\lambda^2$ for a linear factor $\lambda$. This is the report's reading of Frank's volume-expansion theorem, and no source states a tile-count formula.
- That a sphere tiling has finitely many tiles, so periodicity does not apply there (inferred).
- The definitions of $\zeta$ in the pentagrid formulas and of $e_j$ in the general multigrid form, which the report does not give.
- The meaning of $\phi$ in Frank's volume-expansion quotation in Section 5.1, which the report does not define.
- The quoted definitions of weak and strong aperiodicity, which the report attributes to Mozes without naming the text it quoted.
- The hat's linear factor $\phi^2$, which came through a summarizing tool with no verbatim quote.
- The cells of Wikipedia's list of aperiodic sets of tiles, and the state of `lau-penrose-growth` and `penrose-lattice`.
- That Tatham's Puzzles are written in C, which comes from the searcher's own knowledge.
- The chapter numbers of Grunbaum and Shephard.
- The report's assessments in Section 3 and its reading order in Section 10, which are the research's judgement and not sourced facts.

### 12.3. Sources Not Read

These sources could not be read, and the claims that depend on them stay unverified. They belong in `docs/research/source-requests.md`, which records such sources for the owner to supply. This table gives what is needed from each and the report's priority.

| Source | Why not read | Needed for | Priority |
| --- | --- | --- | --- |
| [BAAKE-GRIMM-2013] | Book, cambridge.org returned HTTP 403 | Authoritative definitions of local indistinguishability, MLD, hull, model sets, and inflation, in Chapters 5 to 7 | High |
| [DEBRUIJN-1981] | ScienceDirect returned HTTP 403 | The primary pentagrid formulas and the regularity and offset-sum conditions, now sourced from the 2013 survey only | High |
| [GOODMAN-STRAUSS-2005] | Springer paywall | Construction details, and whether the construction is substitution-like enough to render | Medium |
| [GRUNBAUM-SHEPHARD-1987] | Printed book | The classical Penrose and Ammann chapters, matching rules, colourings, and the chapter numbers | Medium |
| [RADIN-1994] | Not retrieved | The pinwheel scale factor and the infinite-rotation structure, which matters for orientation colouring | Medium |
| [SOCOLAR-STEINHARDT-1986] | Not located within the search budget | The generalized dual method for arbitrary grids, relevant to per-pixel evaluation beyond Penrose | Medium |
| [GAHLER-RHYNER-1986] | IOP page not reached | The formal link between multigrid point location and cut-and-project | Low |
| [AUSTIN-2005] | HTTP 403 | An accessible pentagrid exposition with figures | Low |
| [SHADERTOY-F3S3DY], [SHADERTOY-NEOZHAOLIANG] | HTTP 403 | Whether a published per-pixel cut-and-project or pentagrid shader exists, and its method | Medium |
| [MOZES-1989] | Not retrieved, title from the searcher's recollection | Primary definitions of weak and strong aperiodicity | Low |
| [SMITH-HAT-2023], full text of the H, T, P, and F section | PDF not extractable, and the ar5iv text was truncated | Verbatim metatile definitions and the $\phi^2$ inflation statement | Medium |
| [TATHAM-ARXIV-2026], Sections 7.1, 7.3, and 7.4 | Truncated at 100,000 characters | Transducer state counts and results for Penrose, spectre, and hat | Medium |

## 13. References

Every web source was accessed on 2026-11-15.

- [AMOSAIC] `amosaic` crate, crates.io, 2025. Primary. The report gives no URL.
- [AUSTIN-2005] Austin, D., "Penrose Tilings Tied up in Ribbons", AMS Feature Column, December 2005, <https://www.ams.org/featurecolumn/archive/ribbons.html>. Not read.
- [BAAKE-GRIMM-2013] Baake, M. and Grimm, U., "Aperiodic Order Vol. 1: A Mathematical Invitation", Cambridge University Press, 2013, doi:10.1017/CBO9781139025256. Not read.
- [BEENKER-1982] Beenker, TH-Report 82-WSK04, Eindhoven, 1982. Not read.
- [BOYLE-KULP-2024] Boyle and Kulp, arXiv:2408.15316, 2024, Phys. Rev. D 111 (2025), <https://arxiv.org/abs/2408.15316>. Primary.
- [BUGARIN-2009] Bugarin, de las Penas, and Frettloh, arXiv:0905.4048, 2009, <https://arxiv.org/abs/0905.4048>. Primary, abstract only.
- [COHEN-GS-2015] Cohen and Goodman-Strauss, arXiv:1510.06439, 2015, <https://arxiv.org/abs/1510.06439>. Primary.
- [CRATES-PENROSE] `lau-penrose-growth` and `penrose-lattice` crates, crates.io, 2026. Primary. The report gives no URL.
- [DEBRUIJN-1981] de Bruijn, N. G., "Algebraic theory of Penrose's non-periodic tilings of the plane I, II", Indag. Math. 84 (Proc. KNAW A 84(1)), 1981, 39-66, doi:10.1016/1385-7258(81)90017-2 (part II). Not read.
- [DEBRUIJN-SURVEY-2013] "Quasicrystals: the impact of N.G. de Bruijn", arXiv:1306.6698, 2013, <https://ar5iv.arxiv.org/html/1306.6698>. Secondary.
- [EGAN-APPLET] Egan, applet notes, <https://www.gregegan.net/APPLETS/12/12.html>. Primary.
- [EGAN-DEBRUIJN] Egan, de Bruijn notes, <https://www.gregegan.net/APPLETS/12/deBruijnNotes.html>. Primary.
- [FRANK-2007] Frank, "A primer of substitution tilings of the Euclidean plane", arXiv:0705.1142, 2007, <https://arxiv.org/abs/0705.1142>, text at <https://ar5iv.labs.arxiv.org/html/0705.1142>. Primary.
- [GAHLER-RHYNER-1986] Gahler, F. and Rhyner, J., "Equivalence of the generalised grid and projection methods for the construction of quasiperiodic tilings", J. Phys. A 19 (1986) 267-277, doi:10.1088/0305-4470/19/2/020. Metadata only.
- [GOODMAN-STRAUSS-2005] Goodman-Strauss, C., "A strongly aperiodic set of tiles in the hyperbolic plane", Invent. Math. 161 (2005) 119-132, doi:10.1007/s00222-004-0384-1. Not read.
- [GOODMAN-STRAUSS-HIER] Goodman-Strauss, C., hierarchical strongly aperiodic set of tiles in the hyperbolic plane, title taken from the mirror URL, undated, mirror at <https://api.sambuz.com/doc/a-hierarchical-strongly-aperiodic-set-of-tiles-in-the-pdf-document-690881>. Read through a mirror.
- [GRUNBAUM-SHEPHARD-1987] Grunbaum, B. and Shephard, G. C., "Tilings and Patterns", Freeman, 1987, Dover, 2016, ISBN 9780486469812. Not read.
- [HAT-AMP] `hat-amp`, PyPI, 2026, <https://pypi.org/project/hat-amp/>. Primary.
- [HAT-PAGE] Smith, Myers, Kaplan, and Goodman-Strauss, hat project page, <https://cs.uwaterloo.ca/~csk/hat/>. Primary.
- [HATVIZ] Kaplan, `hatviz`, <https://github.com/isohedral/hatviz>. Primary.
- [MARGENSTERN-2007] Margenstern, arXiv:0706.4161, 2007, <https://arxiv.org/abs/0706.4161>. Primary.
- [MARGENSTERN-2022] Margenstern, arXiv:2205.07317, 2022, <https://arxiv.org/abs/2205.07317>. Primary.
- [MOZES-1989] Mozes, S., "Tilings, substitution systems and dynamical systems generated by them", J. Anal. Math., 1989. Not read, title from the searcher's recollection.
- [MUNRO] Munro, grid tiling page, <https://dhmunro.github.io/tiling/gridtile.html>. Secondary.
- [NECOCEN-SPECTRE] necocen, `spectre`, <https://github.com/necocen/spectre>. Secondary.
- [OYONO-PETITE-2009] Oyono-Oyono and Petite, arXiv:0905.1932, 2009, <https://arxiv.org/abs/0905.1932>. Primary.
- [PANZERSCHREK] Panzerschrek, `penrose_tiling.glsl`, undated, <https://github.com/Panzerschrek/shader_bits/blob/master/penrose_tiling.glsl>. Primary.
- [PENTAGRID-REPOS] `gautamra/penrose`, `clausm0n/penrose_generator`, and `YingHongTham/penrosebud`, GitHub. Secondary. The report gives no URLs.
- [PRESHING-2011] Preshing, "Penrose tiling explained", 2011-08-31, <https://preshing.com/20110831/penrose-tiling-explained>. Secondary.
- [PROKHODA-2018] Prokhoda, arXiv:1804.08384, 2018, <https://arxiv.org/abs/1804.08384>. Primary, abstract only.
- [RADIN-1994] Radin, C., "The pinwheel tilings of the plane", Ann. Math. 139(3) (1994) 661-702. Not read.
- [REUSSER-RENDER-2026] Reusser, "Rendering the Aperiodic Monotile", 2026-04-25, <https://rreusser.github.io/notebooks/aperiodic-monotile/>. Primary.
- [REUSSER-ZOOM-2026] Reusser, "Zooming Aperiodic Monotile", 2026-04-28, <https://rreusser.github.io/notebooks/zooming-aperiodic-monotile/>. Primary.
- [SHADCN] "Penrose tiling shader", <https://www.shadcn.io/shaders/penrose-tiling-shader>. Secondary.
- [SHADERTOY-F3S3DY] "Penrose Diagram - 5D Projection", Shadertoy, <https://www.shadertoy.com/view/f3S3Dy>. Not read.
- [SHADERTOY-NEOZHAOLIANG] neozhaoliang, "Impossible Aperiodic Tilings", Shadertoy. Not read. The report gives no URL.
- [SHRX-SPECTRE] shrx, `spectre`, <https://github.com/shrx/spectre>. Primary.
- [SLABBE] `slabbe` 0.8.0, aperiodic monotile module, <https://www.labri.fr/perso/slabbe/docs/0.8.0/aperiodic_monotile.html>. The report gives no primary or secondary mark.
- [SMITH-HAT-2023] Smith, Myers, Kaplan, and Goodman-Strauss, "An aperiodic monotile", arXiv:2303.10798, v1 2023-03-20, v3 2024-07-03, <https://arxiv.org/abs/2303.10798>. Primary.
- [SMITH-SPECTRE-2023] Smith et al., "A chiral aperiodic monotile", arXiv:2305.17743, v2 2024-09-28, <https://arxiv.org/abs/2305.17743>. Primary.
- [SOCOLAR-STEINHARDT-1986] Socolar, J. E. S. and Steinhardt, P. J., "Quasicrystals II: Unit-cell configurations", Phys. Rev. B 34 (1986). Not read.
- [SOCOLAR-TAYLOR-2010] Socolar and Taylor, "An aperiodic hexagonal tile", arXiv:1003.4279, 2010, J. Combin. Theory A 118 (2011), <https://arxiv.org/abs/1003.4279>. Primary.
- [SPECTRE-PAGE] Kaplan, spectre project page and app, <https://cs.uwaterloo.ca/~csk/spectre/>. Primary.
- [SPECTRE-EXPLORER] Bauer, Andrej, `spectre-explorer`, <https://github.com/andrejbauer/spectre-explorer>. Secondary.
- [STT-CRATE] Tatham, S., `substitution-tiling-transducers`, docs.rs, <https://docs.rs/substitution-tiling-transducers/latest/substitution_tiling_transducers/>. Primary.
- [TACTILE-JS] Kaplan, Tactile-JS, <https://github.com/isohedral/tactile-js>. Primary.
- [TATHAM-2023] Tatham, S., "Generating aperiodic tilings with combinatorial coordinates", 2023-04-10, <https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/aperiodic-tilings/>. Primary.
- [TATHAM-ARXIV-2026] Tatham, S., "Finite-state transducers for substitution tilings", arXiv:2512.16595, v2 2026-03-15, <https://arxiv.org/abs/2512.16595>. Primary.
- [TATHAM-FOLLOWUP-2024] Tatham, S., follow-up post, 2024-07-16, <https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/aperiodic-followup/>. Primary.
- [TATHAM-PUZZLES] Tatham, S., Simon Tatham's Portable Puzzle Collection, <https://www.chiark.greenend.org.uk/~sgtatham/puzzles/>. Primary.
- [TATHAM-REFINE-2025] Tatham, S., "The Refinable Frontier", 2025-09-02, <https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/aperiodic-refine/>. Primary.
- [TATHAM-SPECTRE-2023] Tatham, S., spectre article, 2023-06-16, <https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/aperiodic-spectre/>. Primary.
- [TATHAM-TILINGS-REPO] Tatham, S., `tilings`, at `git.tartarus.org/simon/tilings.git`. Primary, through the blog.
- [TATHAM-TRANSDUCERS-2024] Tatham, S., "Beyond the wall", 2024-06-10, <https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/aperiodic-transducers/>. Primary.
- [TE] Frettloh, Gahler, and Harriss, Tilings Encyclopedia, <https://tilings.math.uni-bielefeld.de/>. Primary.
- [TE-AB] Tilings Encyclopedia, "Ammann-Beenker", <https://tilings.math.uni-bielefeld.de/substitution/ammann-beenker/>. Primary.
- [TE-AMMANN-BARS] Tilings Encyclopedia, glossary, "Ammann bars", <https://tilings.math.uni-bielefeld.de/glossary/ammann-bars/>. Secondary.
- [TE-CHAIR] Tilings Encyclopedia, "Chair", <https://tilings.math.uni-bielefeld.de/substitution/chair/>. Primary.
- [TE-CUT-PROJECT] Tilings Encyclopedia, glossary, "Cut and project", <https://tilings.math.uni-bielefeld.de/glossary/cut-and-project/>. Secondary.
- [TE-PENROSE] Tilings Encyclopedia, "Penrose rhomb", undated, <https://tilings.math.uni-bielefeld.de/substitution/penrose-rhomb/>. Primary.
- [TE-PINWHEEL] Tilings Encyclopedia, "Pinwheel", <https://tilings.math.uni-bielefeld.de/substitution/pinwheel/>. Primary.
- [TILEZZ] apirogov, `tilezz`, <https://github.com/apirogov/tilezz>. Primary.
- [WP-APERIODIC] Wikipedia, "Aperiodic tiling", <https://en.wikipedia.org/wiki/Aperiodic_tiling>. Secondary.
- [WP-APERIODIC-LIST] Wikipedia, "List of aperiodic sets of tiles", <https://en.wikipedia.org/wiki/List_of_aperiodic_sets_of_tiles>. Secondary.
- [WP-BINARY] Wikipedia, "Binary tiling", <https://en.wikipedia.org/wiki/Binary_tiling>. Secondary.
- [WP-CHAIR] Wikipedia, "Chair tiling", <https://en.wikipedia.org/wiki/Chair_tiling>. Secondary.
- [WP-PENROSE] Wikipedia, "Penrose tiling", <https://en.wikipedia.org/wiki/Penrose_tiling>. Secondary.
