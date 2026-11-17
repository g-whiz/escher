# Colourings of Tilings Associated with a Symmetry Group

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-17 |
| Applies to | The choice of the polygons that the Escher renderer colours, and the computation of their colours and of the available numbers of colours |

## Abstract

This document states what the literature and existing software establish about colouring the tiles of a tiling associated with a planar symmetry group so that every symmetry of the tiling permutes the colours. It gives the definitions of colourings, colour symmetry, and perfect colourings, the lemmas that turn a subgroup of finite index into a perfect colouring and the scope in which each is proved, the published tables of available numbers of colours, the algorithms and software that enumerate subgroups of low index, and the per-pixel colour computation of SymmHub. It gives a step-level account of Todd-Coxeter coset enumeration, of the low-index subgroup search, and of Reidemeister-Schreier rewriting, with their termination, their limits in existing implementations, SymmHub's implementation, and worked examples. It then summarizes other tilings associated with a group, the Wythoff construction, and the colouring behaviour of existing tools, and compares three candidate designs for the polygons that Escher draws, limited to what the sources establish. Every claim that no source establishes is listed as an open question.

## 1. Scope

This document covers three candidate designs for the polygons that the first Escher deliverable draws and colours:

- Design (a): the tiling by copies of the fundamental domain, with a perfect colouring.
- Design (b): a fundamental domain cut into several polygons, each with one colour.
- Design (c): a uniform tiling from the Wythoff construction, with one colour per face orbit.

For each design it states what determines the available numbers of colours, the machinery that the colour computation needs, and the gaps that the sources leave open. It makes no recommendation. Section 7 states that machinery at step level: coset enumeration, the low-index subgroup search, and Reidemeister-Schreier rewriting.

It leaves these topics to other documents:

- Orbifold signatures and group presentations: `docs/research/orbifold-notation.md`.
- The folding loop, the models, and the drawing of tile borders: `docs/research/rendering.md`.
- The SymmHub source code in depth: `docs/research/symmhub.md`.
- Sources that the research could not read in full: `docs/research/source-requests.md`.

This section is non-normative. The document uses no requirement key words, so it has no Requirements Language section.

## 2. Notation and Conventions

### 2.1. Symbols

- $G$: a planar symmetry group, acting on the sphere, the Euclidean plane, or the hyperbolic plane.
- $X$: the set of tiles of a tiling with symmetry group $G$.
- $F$: a fundamental domain of $G$, or a tile of $X$, as each section states.
- $k$: a number of colours.
- $c$: a colouring, a surjective map $c : X \to \{1, \dots, k\}$.
- $S_k$: the symmetric group on the $k$ colours.
- $\pi$, $\pi_h$: a permutation in $S_k$, and the colour permutation that the symmetry $h$ induces.
- $S$: a subgroup of $G$, usually of index $k$. Frettloh uses $S$ for the subgroup that induces a colouring [FRETTLOH2008].
- $[G : S]$: the index of $S$ in $G$.
- $C(X)$: Frettloh's colour symmetry group, a set of pairs $(h, \pi)$ [FRETTLOH2008].
- $H$: the set of $h \in G$ that appear in a pair of $C(X)$ in Frettloh's notation [FRETTLOH2008], the stabilizer of a partition in Felix and Loquias's notation [FELIX2008], and the stabilizer of one colour in the notation of Conway, Burgiel, and Goodman-Strauss [CBGS2008, p. 155].
- $K$: the symmetry group of the coloured pattern in Frettloh's introduction, the set of group elements whose triangles carry colour 1 in Frettloh's proof of Lemma 3.2 (written $K_1$, with colour classes $K_i$), the kernel of the map to colour permutations in Felix and Loquias, and the group of the symmetries that fix every colour in Conway, Burgiel, and Goodman-Strauss [FRETTLOH2008] [FELIX2008] [CBGS2008, p. 155]. Section 2.3 lists this clash.
- $G_{p,q}$: the Coxeter group of the regular tiling `(p^q)`, with generators $a$, $b$, $c$ (Section 5.1).
- $\bar{G}_{p,q}$: the rotation subgroup of $G_{p,q}$ (Section 5.4).
- $G^2$: the subgroup that the source of the index-2 count writes as $G^2$ (Section 6). The research for this document did not record its definition.
- $a_k(G)$: the number of subgroups of index $k$ in $G$ (Section 7.5).
- $h_k(G)$: the number of homomorphisms from $G$ to $S_k$, that is, $|\mathrm{Hom}(G, S_k)|$ (Section 7.5).
- $\rho$: a homomorphism from $G$ to $S_k$, with one permutation $\rho(s_i)$ for each side pairing $s_i$ of $F$ (Section 8.7).
- $\sigma$: the colour permutation that the fold loop carries for one pixel (Section 8.7). The letter $\sigma$ keeps the state apart from a colour permutation $\pi_h$.
- $\mathcal{G}$, $\mathcal{G}^+$: Singerman's group of all isometries of the upper half-plane and its subgroup of index two of orientation-preserving isometries (Section 6) [SINGERMAN1974, p. 233].
- $\Gamma$, $\Gamma^+$: an NEC group and its subgroup $\Gamma \cap \mathcal{G}^+$ (Section 6) [SINGERMAN1974, p. 235].
- $\mathcal{D}$, $m_{ij}$, $\mathcal{K}$: a Delaney-Dress symbol, its labelling maps, and its curvature (Section 9.2).
- $\chi_{\mathrm{orb}}$: the orbifold Euler characteristic of the orbifold of a Delaney-Dress symbol (Section 9.2) [BALKE1996, p. 97].

### 2.2. Terminology

- colouring: A surjective map from the tiles of a tiling to a set of $k$ colours [FRETTLOH2008, p. 1]. Felix and Loquias instead treat a colouring as a partition of $G$ [FELIX2008, Section 2].
- colour symmetry: A symmetry $h$ of the uncoloured tiling for which a permutation $\pi$ of the colours exists with $c(h(x)) = \pi(c(x))$ for every tile $x$ [FRETTLOH2008, p. 2].
- colour symmetry group: Frettloh's group $C(X)$ of the pairs $(h, \pi)$ that satisfy the condition of a colour symmetry [FRETTLOH2008, p. 2].
- perfect colouring: A colouring in which every symmetry of the uncoloured tiling is a colour symmetry [FRETTLOH2008, p. 2]. Section 3 gives the variants.
- semiperfect colouring: A colouring whose partition stabilizer has index 2 in $G$ [FELIX2008, Section 2].
- colour permutation group: The image of the colour symmetries in the permutation group of the colours, isomorphic to $H/K$ [FELIX2008, Section 1].
- coset colouring: The colouring of the tiles $gF$ that gives each tile the coset of a fixed subgroup $S$ that contains $g$ (Section 4.1). Frettloh calls it the left coset colouring method [FRETTLOH2008, p. 3].
- fundamental-domain tiling: The tiling by the images $gF$ of a fundamental domain $F$ under the elements $g$ of $G$. Frettloh writes $T_{p,q}$ for this tiling when $G = G_{p,q}$ [FRETTLOH2008, p. 3].
- tile stabilizer: The subgroup of the elements of $G$ that map a given tile onto itself.
- regular tiling: A tiling `(p^q)` by regular $p$-gons with $q$ tiles at each vertex [FRETTLOH2008, p. 2].
- Laves tiling: The tiling `[p.q.p.q]` by congruent quadrilaterals that Frettloh obtains from `(p^q)` by joining the centre of each $p$-gon to each of its vertices [FRETTLOH2008, p. 2].
- index: The number $[G : S]$ of cosets of a subgroup $S$ in $G$.
- coset table: A table that gives, for each coset of a subgroup and each generator, the coset that the generator maps it to [GAP, Section 47.6].
- coset enumeration: The construction of a complete coset table of a subgroup given by generators, from a presentation of the group, also called Todd-Coxeter enumeration (Section 7.10) [HOLT2013, Section 5] [CANNON-HAVAS1992, Section 2.2].
- definition: A step of coset enumeration that adjoins a new coset as the image of a live coset under a generator or its inverse (Section 7.10).
- deduction: A table entry that a relator or a subgroup generator forces (Section 7.10). Section 7.21 lists the narrower and wider senses in the sources.
- coincidence: The discovery that two coset numbers name the same coset (Section 7.10) [HOLT2013, p. 17].
- live coset: A coset number that no coincidence has eliminated [SYMPY-CT, lines 88-91].
- scan: The trace of a relator or subgroup generator through the table from a coset, forwards and backwards (Section 7.10).
- HLT strategy: The relator-based strategy of coset enumeration, after Haselgrove, Leech, and Trotter (Section 7.11).
- Felsch strategy: The coset-table-based strategy, which defines the first undefined entry of the table (Section 7.12).
- deduction stack: The list of entries assigned and not yet checked against the relators, in SymPy's Felsch enumeration (Section 7.12).
- compression: The removal of dead rows and the renumbering of the live cosets without gaps (Section 7.14) [BROWN2011, p. 5].
- standardized coset table: A table whose cosets are numbered in the order of their first appearance, read row by row (Section 7.14).
- lenlex, semilenlex: GAP's standard forms, which read all columns or only the generator columns (Section 7.14) [GAP, Section 47.7-1].
- low-index subgroup enumeration: The algorithm of Sims that lists the subgroups of a finitely presented group up to a bound on the index, up to conjugacy [GAP, Section 47.10-1] [LOWINDEX].
- first-in-class test: The test of the low-index search that keeps a table only if no relabelling from another base point gives a smaller table (Section 7.17).
- Schreier generator: A generator of a subgroup formed from a coset representative, a generator of the group, and the representative of their product (Section 7.18).
- representative set: The set of polygons, one from each class of polygons that the Escher renderer colours alike, whose size the first deliverable reports to the user as the available number of colours. This is a project term, and no source defines it.
- face orbit: The set of the images of one face of a tiling under its symmetry group.
- Wythoff construction: The construction of a uniform tiling from a reflection triangle and a generator point inside the triangle or on its boundary (Section 10) [WP-WYTHOFF-SYMBOL].
- isohedral tiling type: One of the 93 classes of tilings whose symmetry group acts transitively on the tiles, in the classification of Grunbaum and Shephard (Section 9.1) [KAPLAN2025].
- Delaney-Dress symbol: The combinatorial description of a periodic tiling that Section 9.2 defines [TEGULA2020, Section 4] [BALKE1996, Section 3, p. 95].
- colour type: The class of a colouring in the classification of Conway, Burgiel, and Goodman-Strauss, determined by a homomorphism from the symmetry group to a group of permutations of the colours (Section 3.3) [CBGS2008, pp. 136 and 154].
- NEC group: A non-Euclidean crystallographic group, a discrete subgroup $\Gamma$ of $\mathcal{G}$ for which the quotient of the upper half-plane by $\Gamma$ is compact [SINGERMAN1974, p. 233].
- Fuchsian group: An NEC group contained in $\mathcal{G}^+$ [SINGERMAN1974, p. 233].
- proper NEC group: An NEC group that contains an element of $\mathcal{G} - \mathcal{G}^+$, that is, an orientation-reversing isometry [SINGERMAN1974, p. 233].

### 2.3. Conventions on Which Sources Differ

- Left or right cosets: Frettloh colours the tile $gF$ by the left coset $gS$ [FRETTLOH2008, p. 3]. Felix and Loquias use a complete set of right coset representatives of $H$ in $G$, with the left action $gP$ on partitions [FELIX2008, Sections 2 and 3]. De Las Penas, Felix, and Laigo use right coset colourings [DLP2006] (abstract summary only). SymmHub offers both orders of composition through `uUseOrbit` (Section 8.4). The order changes the formula that composes permutations along a fold.
- Definition of perfect: Section 3 gives three definitions. They agree that every symmetry of the tiling is a colour symmetry and differ in their framework. Frettloh's definition does not require equally frequent colours (Section 3.1).
- What is counted: Frettloh counts colour symmetry groups up to conjugacy, and Felix and Loquias count colourings up to the action of $G$ (Section 3.4). A table from one source is not comparable with a count from the other without conversion.
- The letter $K$: Frettloh uses $K$ for the symmetry group of the coloured pattern and $K_1$ for the colour class of colour 1, while Felix and Loquias, and Conway, Burgiel, and Goodman-Strauss, use $K$ for a kernel (Section 2.1).
- The word curvature: Balke and Huson prove that the curvature of a Delaney-Dress symbol is twice the orbifold Euler characteristic [BALKE1996, Theorem 4.1, p. 97]. The quantity that [EPINET-ORB] calls curvature is stated for a signature, and its relation to that curvature is open (Section 9.2).
- Coset tables: the sources differ on the numbering of cosets, the storage of the columns, the standard form, and the meanings of HLT and of a deduction. Section 7.21 lists the differences.
- Tiling symbols: Frettloh writes `(p^q)` for the regular tiling by $p$-gons with $q$ at each vertex, following Grunbaum and Shephard [FRETTLOH2008, p. 2]. So `(7^3)` is the heptagonal tiling.

### 2.4. Spelling of Names and Symbols

Names with accented letters are written in ASCII transliteration: Frettloh, Grunbaum, De Las Penas, Gorner, Dobcsanyi, Neubuser, Schlafli, Poincare, and Mobius. Orbifold signatures, Schlafli symbols, vertex configurations, tiling symbols, and Wythoff symbols are written as code spans, such as `*642`, `{7,3}`, `3.3.p.3.q`, `(7^3)`, and `p | q r`.

Quoted text keeps the spelling of its source, and its mathematical symbols are written in ASCII, such as "in" for the element sign. Where a source title or a web address contains a spelling that the prose checker rejects, it is written as a code span.

## 3. Colourings and Colour Symmetry

### 3.1. Frettloh's Definitions

Frettloh defines a colouring as follows: "A k-colouring (X, c) of X is a surjective map c : X -> {1, . . . , k}" [FRETTLOH2008, p. 1]. He then considers the symmetries $h \in G$ that respect the colour classes [FRETTLOH2008, p. 1]:

$$\forall x, y \in X : c(x) = c(y) \Rightarrow c(h(x)) = c(h(y))$$

The colour symmetry group is the set of pairs [FRETTLOH2008, p. 2]:

$$C(X) = \{(h, \pi) \mid h \in G, \pi \in S_k : c(h(x)) = \pi(c(x)) \text{ for all } x \in X\}$$

With $H = \{h \mid (h, \pi) \in C(X)\}$, the map $h \mapsto \pi_h$ is a group homomorphism and $H$ is isomorphic to $C(X)$ [FRETTLOH2008, p. 2]. "A colouring is called perfect, if H = G, or equivalently: if every symmetry of X is a colour symmetry of (X, c)" [FRETTLOH2008, p. 2].

The introduction states the consequence for the two groups: "a perfect colouring yields a simple algebraic relation between the symmetry group G of the uncoloured pattern and the symmetry group K of the coloured pattern: The factor group G/K is a permutation group" [FRETTLOH2008, p. 1].

The definition does not require that $G$ permutes the colours transitively. For regular and Laves tilings, Lemma 3.1 states that "all colours occur with the same frequency" in a perfect colouring, and Frettloh adds that "The last result is not true in general for perfect colourings of other tilings" [FRETTLOH2008, p. 3].

This example from Frettloh's Figure 1 shows a colour permutation for a two-colouring: "A reflection R_a in a interchanges the colours ... $\pi_{R_a} = (1, 2)$. The reflections R_b and R_c both fix the colours" [FRETTLOH2008, p. 2].

### 3.2. Felix and Loquias's Definitions

Felix and Loquias treat a colouring as a partition $P$ of $G$, and $H$ is the stabilizer of $P$ [FELIX2008, Section 2]. They use "a complete set of right coset representatives of H in G" [FELIX2008, Section 2].

- Perfect: "When [G:H]=1 or H=G, a coloring associated with P is called perfect" [FELIX2008, Section 2].
- Semiperfect: "If [G:H]=2, we call a coloring associated with P semiperfect" [FELIX2008, Section 2].
- Colour permutation group: the image $f(H)$ of the map $f$ from $H$ to the permutations of the colours is isomorphic to $H/K$, where "The kernel of f is K" [FELIX2008, Section 1].
- Equivalence: "If P and Q are partitions of G and there is a g in G such that Q=gP, then we say that P and Q" are equivalent [FELIX2008, Section 3].

For a semiperfect colouring, "there are only two partitions of G that are equivalent to P, namely, P and yP", and the proof uses that $H$ is normal in $G$ [FELIX2008, Section 3, Theorem 1]. For index-2 subgroups $H' = H^\alpha$ with $\alpha$ in the normalizer of $G$ in the full isometry group, the colourings correspond and "the action of H on C is equivalent to the action of H' on C'" [FELIX2008, Section 6, Theorem 7].

### 3.3. Other Definitions

Bulatov gives an informal definition: a colouring is perfect when each symmetry transformation maps the tiles of one colour onto tiles of one colour, which may equal or differ from the original colour [BULATOV2010, opening slide].

The Waerden-Burckhardt framework determines a $k$-colour group by "a subgroup H of index k in the symmetry group G" [WP-POLYCHROMATIC]. This claim rests on a secondary source only.

Conway, Burgiel, and Goodman-Strauss define colourings through homomorphisms. For two colours, the colour type "amounts to specifying a homomorphism from G into the group permuting {a, b} whose kernel is K", and two colourings $G/K$ and $G'/K'$ have the same colour type when $G$ can be isotopically reshaped into $G'$ in a way that takes $K$ to $K'$ [CBGS2008, p. 136]. In an $n$-fold colouring the symmetries take any one of the $n$ colours to any other, which the book calls being "transitive" on the colours [CBGS2008, pp. 153-154]. For three colours, the type "is determined by a homomorphism from G onto some subgroup of the group S[3]" [CBGS2008, p. 154].

The book names three groups: the full group $G$, the kernel $K$ of the symmetries that fix each colour, and the stabilizer $H$ of one chosen colour. The stabilizers of the different colours are "conjugate in the group-theoretical sense", and a type is written $G^n/H/K$, shortened to $G^n/K$ when $H = K$ and to $G^n//K$ otherwise [CBGS2008, p. 155]. For a prime number $p$ of colours, "We need only seek the stabilizer subgroup H of index p in G that fixes a given" colour, because it determines the type [CBGS2008, p. 161]. For hyperbolic groups, "By replacing the names of generators by permutations that satisfy the relations, we obtain an annotated symbol that always characterizes the" type [CBGS2008, p. 228].

Felix and Loquias quote Senechal that classifying colour groups and classifying coloured patterns are not the same [FELIX2008, Section 1, citing SENECHAL1988]. Bulatov dates the publications on perfect colourings of hyperbolic tilings from 2006, naming De Las Penas, Felix, and Laigo (2006) and Frettloh (2008) [BULATOV2010, slide "Notes on a previous publications"].

### 3.4. What Is Counted

Frettloh counts colour symmetry groups up to conjugacy, not colourings: "So what we count here are not different colourings, but different colour symmetry groups, where different means non-conjugate. This is the usual approach, compare [2], [10]" [FRETTLOH2008, Section 3, p. 3]. He identifies subgroups "if they are conjugate in G_{p,q}" [FRETTLOH2008, p. 4].

The same paragraph states that "different colourings may possess the same colour symmetry group" and that colourings with the same colour symmetry group "are strongly related, but not necessarily congruent" [FRETTLOH2008, Section 3, p. 3]. The conclusion instead speaks of "a bijection between perfect colourings of these tilings and subgroups of G_{p,q}" [FRETTLOH2008, Conclusion, p. 5]. This document reports both statements and does not resolve the tension between them (Section 14.1).

Felix and Loquias identify colourings that $G$ relates, and for index 2 also colourings that the normalizer relates (Section 3.2). Neither count equals the number of raw colourings or the number of colour permutation groups.

## 4. Design (a): The Fundamental-Domain Tiling

### 4.1. Coset Colourings

Frettloh's left coset colouring method colours the triangle tiling $T_{p,q}$, which is the orbit of the fundamental triangle $F = F_{p,q}$ under $G_{p,q}$ [FRETTLOH2008, p. 3]. Because the stabilizer of $F$ is trivial, the orbit gives "a bijection between elements of G_{p,q} and elements of T_{p,q}" [FRETTLOH2008, p. 3].

These steps give the coset colouring of a subgroup $S$ of index $k$ [FRETTLOH2008, p. 3]:

1. Number the left cosets of $S$ as $S_1 = S, S_2, \dots, S_k$.
2. Set $c(fF) = 1$ for each $f \in S$.
3. Set $c(fF) = i$ when $f$ is an element of the $i$-th left coset $S_i$.

Lemma 2.1 states: "All colourings of a tiling obtained by the left coset colouring method are perfect" [FRETTLOH2008, Lemma 2.1, p. 3]. The proof rests on the fact that "c(fF) = c(f'F) is equivalent to f and f' being in the same left coset. That is, $f^{-1} f' \in S$" [FRETTLOH2008, p. 3].

Bulatov describes the same method for hyperbolic tilings as the "cosets algorithm": compute low-index subgroups of the finitely presented group with a computer algebra package such as GAP, and give the tiles obtained from the base tile by each coset a different colour [BULATOV2010, slides "Coloring and subgroups" and "Conclusion"]. De Las Penas, Felix, and Laigo use right coset colourings to study the subgroup structure of a hyperbolic crystallographic group on the `3.4.3.4.3.3` tiling [DLP2006] (abstract summary only).

### 4.2. Scope of Lemma 2.1

Lemma 2.1 is stated and proved for $T_{p,q}$ under the Coxeter group $G_{p,q}$ [FRETTLOH2008, p. 3]. Frettloh says that it "is true in a more general context than that of this article" and gives no detail [FRETTLOH2008, p. 3].

Bulatov's slides apply the cosets algorithm to hyperbolic tilings of groups that are not reflection groups, such as `433` and `32x`, without a statement of a lemma or its scope [BULATOV2010]. No source read states Lemma 2.1 for the fundamental-domain tiling of an arbitrary orbifold group, including groups with rotations, glide reflections, handles, or cross-caps (Section 14.1).

### 4.3. Available Numbers of Colours

Each subgroup of index $k$ gives a $k$-colouring by the method of Section 4.1 [FRETTLOH2008, p. 3]. The numbers of colours that coset colourings of a fundamental-domain tiling offer are therefore the indices of the subgroups of finite index of $G$, within the scope of Section 4.2.

For an arbitrary orbifold group, no source read establishes that every perfect colouring of a fundamental-domain tiling arises from a subgroup, or that the colourings correspond to transitive permutation representations of $G$ (Section 14.1, items 1, 2, and 16). Frettloh's conclusion speaks of a bijection between perfect colourings of the tilings of his paper and subgroups of $G_{p,q}$ (Section 3.4). His paper does not mention transitive permutation representations [FRETTLOH2008].

No source gives a closed rule for which indices occur for a given group. Frettloh's paper gives no general statement about which $k$ can occur [FRETTLOH2008], and Bulatov concludes only that "There are infinite number of" coloured symmetries of hyperbolic tilings [BULATOV2010, slide "Conclusion"]. The available indices are found by enumeration (Section 7).

### 4.4. Example Counts from Bulatov

Bulatov's captions give these numbers of colours for coset colourings of hyperbolic tilings [BULATOV2010]. The captions do not say that the lists are complete, and the research for this document recorded no statement of which tiling is coloured beyond the base tile and its images.

| Signature | Numbers of colours shown |
| --- | --- |
| `433` | 4, 3 |
| `32x` | 2 |
| `*642` | 12, 8, 3 |
| `642` | 7 |
| `3*4` | 3 |
| `*542` | 2, 4, 5, 6, 10, 11, 12, 15, 16 (gallery) |

## 5. Tiles with a Non-Trivial Stabilizer

This section bears on design (c), the uniform (Wythoff) tilings, because the lemmas that it cites concern the tiles of regular tilings and of their Laves duals, and those tiles have a non-trivial stabilizer. No source read addresses tile stabilizers for design (b), a fundamental domain cut into several polygons. All its results are proved only for the regular tilings `(p^q)` and `(q^p)` and the Laves tiling `[p.q.p.q]` under $G_{p,q}$ and its rotation subgroup.

### 5.1. The Coxeter Group and Its Generators

For integers $p, q \ge 3$, the regular tiling `(p^q)` is spherical, Euclidean, or hyperbolic as $1/p + 1/q$ is greater than, equal to, or less than $1/2$ [FRETTLOH2008, p. 2]. Its symmetry group is the Coxeter group [FRETTLOH2008, eq. (1), p. 3]:

$$G_{p,q} = \langle a, b, c \mid a^2 = b^2 = c^2 = (ab)^q = (ac)^2 = (bc)^p = 1 \rangle$$

"G_{p,q} is also the symmetry group of both (q^p) and [p.q.p.q]" [FRETTLOH2008, p. 3].

A fundamental domain of $G_{p,q}$ is the right triangle $F_{p,q}$ with angles $\pi/2$, $\pi/p$, and $\pi/q$ [FRETTLOH2008, p. 3]. The generators are the reflections in its edges [FRETTLOH2008, p. 3]:

| Generator | Mirror |
| --- | --- |
| $a$ | The edge opposite the angle $\pi/p$ |
| $b$ | The edge opposite the angle $\pi/2$ |
| $c$ | The edge opposite the angle $\pi/q$ |

The triangle tiling $T_{p,q}$ "can be obtained from (p^q) by dividing each p-gon into 2p triangles congruent to F", and a colouring of `(p^q)` is a colouring of $T_{p,q}$ in which "all 2p triangles forming a p-gon possess the same colour" [FRETTLOH2008, Section 3, p. 3]. The paper says nothing about the edges or vertices of these tilings or their stabilizers [FRETTLOH2008].

### 5.2. Regular and Laves Tilings

The Laves tiling `[p.q.p.q]` for $p < q$, written `[q.p.q.p]` otherwise, is the tiling "by congruent quadrilaterals, with either p or q tiles at each vertex" [FRETTLOH2008, p. 2]. "In R^2 there are only three different regular tilings and eleven different Laves tilings", and the sphere has five regular tilings and eight Laves tilings [FRETTLOH2008, p. 2].

Lemma 3.1 states: "In a perfect colouring of a regular tiling (p^q) or a Laves tiling [p.q.p.q], all colours occur with the same frequency" [FRETTLOH2008, Lemma 3.1, p. 3]. The proof uses that "G_{p,q} acts transitively on the tiles" [FRETTLOH2008, p. 3].

### 5.3. Perfect Colourings Under the Full Group

Lemma 3.2 states [FRETTLOH2008, Lemma 3.2, pp. 3-4]: "Let S be a subgroup of G_{p,q} of index k. S induces a perfect k-colouring of (q^p), if and only if a, b in S. S induces a perfect k-colouring of (p^q), if and only if b, c in S. S induces a perfect k-colouring of [p.q.p.q], if and only if a, c in S."

| Tiling | Generators that $S$ contains |
| --- | --- |
| `(q^p)` | $a$, $b$ |
| `(p^q)` | $b$, $c$ |
| `[p.q.p.q]` | $a$, $c$ |

The proof covers `(q^p)` only, and "The other cases are completely analogous" [FRETTLOH2008, p. 4]. Its key step uses the $q$-gon $F \cup aF \cup abF \cup abaF \cup \dots$, whose triangles all carry colour 1, so that $a, ab, aba, \dots \in K_1$ [FRETTLOH2008, p. 4]. This is the only tile whose composition from copies of $F$ the paper states explicitly.

For the heptagonal tiling `(7^3)`, which has $p = 7$ and $q = 3$, a subgroup of $G_{7,3}$ induces a perfect colouring when it contains $b$ and $c$, and the first available numbers of colours are the `(7^3)` row of Table 1 [FRETTLOH2008, p. 2, Lemma 3.2, and Table 1].

### 5.4. Perfect Colourings Under the Rotation Group

The rotation group $\bar{G}_{p,q}$ "is spanned in G_{p,q} by ab, ac" and is "a subgroup of G_{p,q} of index 2" [FRETTLOH2008, p. 4]. Lemma 4.1 states [FRETTLOH2008, Lemma 4.1, p. 4]: "Let S be a subgroup of $\bar{G}_{p,q}$ of index k. S induces a perfect k-colouring of (q^p) if and only if ab in S. S induces a perfect k-colouring of [p.q.p.q] if and only if ac in S. S induces a perfect k-colouring of (p^q) if and only if bc in S."

| Tiling | Element that $S$ contains |
| --- | --- |
| `(q^p)` | $ab$ |
| `[p.q.p.q]` | $ac$ |
| `(p^q)` | $bc$ |

Here "'perfect' is to be read here as 'perfect with respect to the rotation group'" [FRETTLOH2008, p. 5]. The proof uses "the fact, that $F \cup aF$ and $F \cup bF$ are both fundamental domains of $\bar{G}_{p,q}$" [FRETTLOH2008, p. 5].

These colourings "include of course all colourings of the last section, but also many pairs of enantiomorphic colourings, where one is obtained from the other by a reflection" [FRETTLOH2008, p. 4]. Two subgroups are identified "not only if they are conjugate in $\bar{G}_{p,q}$, but also if they are conjugate in G_{p,q}" [FRETTLOH2008, Section 4]. This quotation comes from a condensed extraction of the paper, and its page was not re-checked.

### 5.5. Procedure

Frettloh obtains "all perfect k-colourings of (q^p) (respectively [p.q.p.q], respectively (p^q)) by listing the subgroups of G_{pq} which contain a, b (respectively a, c, respectively b, c)", which GAP can carry out [FRETTLOH2008, p. 4]. These steps restate that procedure for one tiling and one bound on $k$:

1. Write the presentation of $G_{p,q}$ from Section 5.1.
2. List the subgroups of $G_{p,q}$ of index at most the bound, up to conjugacy in $G_{p,q}$, with a low-index subgroup enumeration (Section 7).
3. Keep the subgroups that contain the two generators of Section 5.3 for the chosen tiling.
4. Report each kept index $k$, with the number of kept conjugacy classes of that index.

For the rotation group, step 2 lists subgroups of $\bar{G}_{p,q}$, step 3 tests the element of Section 5.4, and step 4 also identifies subgroups that are conjugate in $G_{p,q}$. This "requires some programming in GAP, and the computations become even harder", and "there are 119 subgroups of index 20 in $\bar{G}_{5,4}$ inducing perfect colourings of [4.5.4.5]" that "have to be checked pairwise for being conjugate in G_{5,4}" [FRETTLOH2008, p. 5]. "The program code can be requested from the author" [FRETTLOH2008, p. 5].

### 5.6. Frettloh's Table 1: The Full Group

Table 1 lists "the first possible values k for perfect k-colourings of some hyperbolic regular and Laves tilings" [FRETTLOH2008, Table 1, p. 4]. An entry written `n (m)` here is the paper's $n^m$: there are $m \ge 2$ non-conjugate colour symmetry groups for $k = n$. An entry without a count has exactly one colour symmetry group up to conjugacy. The values were checked against the page images of the paper.

| Tiling | Values of $k$ |
| --- | --- |
| `(7^3)` | 1, 8, 15, 22, 24, 30, 36 (2), 44, 50 (5), ... |
| `[3.7.3.7]` | 1, 9, 15, 21, 28 (2), 30, 35 (2), 36, 37, 42 (5), 49 (8), 50 (3), ... |
| `(3^7)` | 1, 22, 28 (5), 37, 42 (4), 44, 49 (7), 50 (3), ... |
| `(8^3)` | 1, 3, 6, 12, 17, 21 (4), 24, 25 (5), 27 (3), 29 (4), 31 (4), 33 (6), 37 (6), 39 (8), ... |
| `[3.8.3.8]` | 1, 3, 6, 12 (3), 17, 18, 21 (4), 24 (15), 25 (5), 27 (3), 28 (4), 29 (4), 30 (7), ... |
| `(3^8)` | 1, 2, 4, 8, 10 (2), 12, 14, 16 (2), 18, 20 (4), 24 (3), 25 (5), 26, 28 (12), 29, 30 (2), ... |
| `(5^4)` | 1, 2, 6, 11, 12, 16 (2), 21 (3), 22 (5), 24, 26 (9), 28, ... |
| `[4.5.4.5]` | 1, 5 (2), 10 (5), 11, 15 (7), 20 (22), 21 (3), 22 (4), 25 (27), 26 (4), 27 (3), 30 (63), ... |
| `(4^5)` | 1, 5 (2), 10 (4), 11, 15 (7), 16, 20 (9), 21 (3), 22, 25 (27), 26, 27 (3), 30 (38), ... |

### 5.7. Frettloh's Table 2: The Rotation Group

Table 2 lists "the first possible values k of perfect k-colourings of some hyperbolic regular and Laves tilings, using the same scheme as Table 1", and "The groups in Table 1 are contained in this table" [FRETTLOH2008, Table 2, p. 5]. The entries use the convention of Section 5.6.

| Tiling | Values of $k$ |
| --- | --- |
| `(7^3)` | 1, 8, 9, 15 (2), 22 (7), 24, ... |
| `[3.7.3.7]` | 1, 7, 9, 14 (6), 15 (2), 21 (5), 22 (7), ... |
| `(3^7)` | 1, 7, 8, 14 (6), 21 (2), 22 (7), ... |
| `(8^3)` | 1, 3, 6, 9, 10, 12, 13 (2), 15, 17 (5), 18 (5), 19 (5), ... |
| `[3.8.3.8]` | 1, 3, 4, 6, 8 (3), 9, 10 (2), 12 (4), 13 (2), 15, 16 (9), 17 (5), 18 (12), ... |
| `(3^8)` | 1, 2, 4, 8 (4), 10 (3), 12, 13 (2), 14 (2), 16 (12), 17 (5), 18, 19 (5), ... |
| `(5^4)` | 1, 2, 6 (2), 11 (3), 12 (6), 16 (12), 17 (4), ... |
| `[4.5.4.5]` | 1, 5 (2), 6, 10 (9), 11 (3), 12 (4), 15 (15), 16 (10), 17 (4), ... |
| `(4^5)` | 1, 5 (2), 6, 10 (6), 11 (3), 15 (15), 16 (2), 17 (4), ... |

### 5.8. Spherical and Euclidean Cases

Both tables cover hyperbolic tilings only, and the paper has no Euclidean or spherical rows [FRETTLOH2008]. For the sphere, "the cube (4^3) allows perfect colourings with 1, 3 and 6 colours, each one unique, up to permutation of colours" [FRETTLOH2008, p. 4], and the icosahedron `(3^5)` "allows unique perfect colourings with 1, 10 and 20 colours" [FRETTLOH2008, Sections 1 and 2]. Frettloh calls the spherical case "somehow trivial" [FRETTLOH2008, Sections 1 and 2]. The last two quotations come from a condensed extraction of the paper and were not re-checked in the PDF.

For Euclidean tilings, Frettloh writes that "The case of Euclidean tilings is well known for long, see [10] for a good survey" [FRETTLOH2008, p. 1], where [10] is [SCHWARZENBERGER1984]. That survey was not read (Section 14.3). The procedure is said to be general: "In principle, all colour symmetries of all plane regular tilings can be obtained in this manner, be it hyperbolic, Euclidean or spherical tilings" [FRETTLOH2008, p. 4].

## 6. Two Colours

For a finite group $G$, "the index 2 subgroups correspond to hyperplanes in the finite vector space G/G^2", so their number is [ARXIV1806.06911, Section 2, Theorem 2.1]:

$$|\{S \le G : [G : S] = 2\}| = [G : G^2] - 1$$

The source attributes the result to Nganou and states it for finite $G$ only. The groups that Escher draws in Euclidean and hyperbolic geometry are infinite, so the statement does not cover them as stated.

A group "must contain 0,1,3,7,15,... index 2 subgroups", the group $\mathbb{Z} \oplus \mathbb{Z}$ "has three subgroups of index 2", and every subgroup of index 2 is normal [WP-INDEX]. These claims rest on a secondary source only.

For the extended triangle group with all three periods even, Singerman and Watson find seven epimorphisms onto $C_2$: "Altogether this gives us our seven subgroups of index two in the extended triangle group" [SINGERMAN2014, Section 2, Example 1 and Theorem 1]. The epimorphism that sends every reflection $c_i$ to the generator $t$ gives the orientation-preserving triangle group, and the parity condition reads "as $\theta(c_1 c_3) = \theta(c_2 c_3) = t$, $n_2$ and $n_3$ must be even" [SINGERMAN2014]. The signatures of the subgroups follow from Hoare's theorem and the Riemann-Hurwitz formula, for example `(0,+,[n1],{(n2/2,n3/2)})` [SINGERMAN2014].

Singerman states the subgroup of orientation-preserving elements for every proper NEC group: "Let $\Gamma$ be a proper NEC group. Then $\Gamma$ has a Fuchsian subgroup $\Gamma^+$ of index two, namely $\Gamma^+ = \Gamma \cap \mathcal{G}^+$" [SINGERMAN1974, Section 4, p. 235]. An NEC group in Singerman's sense has a compact quotient (Section 2.2), so the statement covers only such groups.

Singerman's Theorem 2 gives the signature of $\Gamma^+$. For the signature with sign $+$, genus $g$, proper periods $m_1, \dots, m_r$, and period cycles $(n_{11}, \dots, n_{1s_1}), \dots, (n_{k1}, \dots, n_{ks_k})$, the Fuchsian signature of $\Gamma^+$ has genus $2g + k - 1$ and the periods $m_1, m_1, \dots, m_r, m_r, n_{11}, \dots, n_{ks_k}$. For the sign $-$ the genus is $g + k - 1$ with the same periods [SINGERMAN1974, Theorem 2, p. 236]. The hypothesis that $\Gamma$ is proper stands in the opening of Section 4 and in the sentence before the theorem, not in the printed statement of the theorem [SINGERMAN1974, pp. 235-236]. Singerman's paper does not mention colourings.

Conway, Burgiel, and Goodman-Strauss classify the twofold colourings of the 17 plane groups in Table 11.1 [CBGS2008, pp. 140-141]. The table lists the types group by group, with `**/**(1)` and `**/**(2)` as two rows, because the case `**/**` "denotes two types that may be distinguished by their numbers of colorings" [CBGS2008, p. 139]. The book states no total number of types, and the readers of this research counted 46 rows. Table 11.2 gives the twofold types of the spherical groups, for example `*532/532`, `*432/432`, `*332/332`, and `3*2/332` [CBGS2008, p. 145]. Its cases with $N = M = \infty$ are the seven twofold frieze types, shown on p. 148 [CBGS2008, p. 145]. Table 3.2 lists the 17 plane signatures with "The nonreflecting elements of any of these groups" as the rotation subgroup "at the bottom of the column" [CBGS2008, Table 3.2, footnote 1, p. 40].

Grunbaum and Shephard treat coloured patterns in chapter 8 [GS1987], which was not read.

There are exactly 46 types of two-colour symmetries of the plane, after H. J. Woods [ARTLANDIA] (search summary only). This claim rests on a secondary source only.

No source read gives a rule that reads the number of index-2 subgroups, and so the number of two-colourings, from an orbifold signature (Section 14.1). Semiperfect colourings (Section 3.2) are a different concept: they concern colourings whose colour symmetry group has index 2, not colourings with two colours.

## 7. Algorithms and Software

### 7.1. Low-Index Subgroup Enumeration

The low-index subgroups algorithm lists the conjugacy classes of subgroups of a finitely presented group whose index is at most a bound. GAP describes its method as "systematically running through a tree of all potential coset tables of G of length at most index", and its running time "will grow exponentially with the value of index" [GAP, Section 47.10-1]. Section 7.17 gives the search at step level.

Cannon and Havas write that the algorithm was "discovered independently in the sixties by Sims and Schaps" and "enumerates all n-row standard coset tables using a combination of coset enumeration and backtrack search methods". They add that "In favourable circumstances, the low index algorithm may be used to find all subgroups having index up to 100 or even more" [CANNON-HAVAS1992, pp. 4-5]. Because the algorithm applies to infinite groups, "it provides us with a tool for proving that an fp-group is infinite" [CANNON-HAVAS1992, p. 5].

Campbell, Havas, and Robertson contrast two methods: "Neubuser describes a method for computing all subgroups of low index by systematically forcing coincidences in larger coset tables", while the implementations in GAP, Magma, and quotpic "use another method, due to Sims, which does a backtrack search through incomplete coset tables" [CHR2007, p. 362].

Magma's implementation makes "coset definitions ... systematically at each open position, and the relations are then traced" [MAGMA] (search summary only). Conder and Dobcsanyi published on the subject in Math. Comp. 74 (2005) [CONDER2005]. The research for this document recorded no claim from that paper.

### 7.2. GAP

These GAP functions compute subgroups of low index and their coset tables [GAP]. The local copy of Chapter 47 states no GAP release (Section 15):

- `LowIndexSubgroupsFpGroup( G[, H], index[, excluded] )`: computes "representatives of the conjugacy classes of subgroups of the finitely presented group G that contain the subgroup H of G and that have index less than or equal to index". "If not given, H defaults to the trivial subgroup." With `excluded`, it returns only subgroups that "do not contain any conjugate of any of the group elements defined by these words" (Section 47.10-1).
- `LowIndexSubgroupsFpGroupIterator`: the iterator form of the same computation. "If one is interested only in one or a few subgroups up to a given index then preferably the iterator should be used" (Section 47.10-1).
- `LowIndexSubgroups( G, index )`: Section 39.19-6. Its description was read through a summary only.
- `CosetTable( G, H )`: "returns the coset table of the finitely presented group G on the cosets of the subgroup H". "The coset table is standard", which means "the words w_1, w_2, ... are length-plus-lexicographic ordered" (Sections 47.6-1 and 47.7). Section 7.14 gives the standard forms.
- `FactorCosetAction( G, H )`: "returns the action of G on the cosets of its subgroup H" (Section 47.6-3).
- `CosetTableFromGensAndRels( fgens, grels, fsgens )`: "the workhorse that performs a Todd-Coxeter coset enumeration" (Section 47.6-5). Section 7.16 gives its options and limits.
- `StandardizeTable( table[, standard] )` and the global variable `CosetTableStandard` (Sections 47.7-1 and 47.7-2).
- `AugmentedCosetTableMtc`, `AugmentedCosetTableRrs`, and `RewriteWord`: the Reidemeister-Schreier functions of Section 47.9 (Section 7.18).

GAP notes that "in addition to the built-in coset enumerator there is the GAP package ACE", and that "GAP provides an interactive Todd-Coxeter in the GAP package ITC" [GAP, Section 47.6].

The argument `H` of `LowIndexSubgroupsFpGroup` matches step 3 of Section 5.5, which keeps the subgroups that contain two given generators. Frettloh's paper does not say whether he used it.

### 7.3. The Python Package `low_index`

The package `low_index` by Culler, Dunfield, and Gorner "implements a variant of Charles Sims' Low Index Subgroups algorithm" [LOWINDEX, README] (read through a summarizing tool, which quoted only this part). The research's paraphrase of the rest of the sentence is that the algorithm enumerates the conjugacy classes of subgroups of finite index below a given bound.

Its entry point is `permutation_reps(rank = 3, short_relators = [], long_relators = [], max_degree = 4)`, shown in the README for the free group of rank 3 up to index 4 [LOWINDEX, README]. A summarizing tool reported 653 results for that call, but the number did not appear in the text that the tool quoted, so it is unverified (Section 14.1, item 27). Each result is a list of one permutation per generator. That reading is inferred from the examples, not stated.

The accompanying note, "LOW INDEX, Revisited and Implemented", describes the algorithm "in terms of elementary covering space theory" [LOWINDEX, abstract]. It says that "our implementation actually outputs a list of permutation representations", "given by the standard right action on the vertices of the cover" [LOWINDEX, Section 4]. Its Sections 4.1 and 4.2 are empty headings. Its benchmark table gives 0.96 s and 17.43 s for the knot group `K11n34` at indices 7 and 8, against 5.09 s and 104.08 s for GAP, and 0.74 s and 11.75 s for `K15n12345`, against 7.71 s and 117.59 s for GAP [LOWINDEX, Section 5]. It cites Sims, Chapter 5 [SIMS1994] [LOWINDEX, Section 1]. Section 7.17 gives its account of the search tree and the canonical base point.

### 7.4. Other Implementations

- SymPy: `low_index_subgroups(G, N, Y)` returns one coset table for each conjugacy class of subgroups up to index `N` that contain the subgroup that `Y` generates [SYMPY-FPG, lines 733-787]. Sections 7.9 to 7.18 describe its code, which cites [HOLT2005] and [SIMS1994] (Section 7.8).
- SymmHub: `lib/sublib/src/sublib_core.js` implements a "Sims-style low-index coset-table backtrack search" (line 15), and `lib/sublib/src/reidemeister.js` implements Reidemeister-Schreier rewriting [SYMMHUB]. Section 7.19 describes the search and how it differs from the other sources, and Section 8.3 describes the catalogue that it generates.

### 7.5. Hall's Counting Formula

Matei and Suciu cite "the following well-known formula of Marshall Hall" [MATEI2002, Section 8.1, eq. (8.2)] [HALL1949]. With $a_k(G)$ the number of subgroups of index $k$ and $h_k(G) = |\mathrm{Hom}(G, S_k)|$, and $a_1 = h_1 = 1$:

$$a_k(G) = \frac{h_k(G)}{(k-1)!} - \sum_{l=1}^{k-1} \frac{h_{k-l}(G)}{(k-l)!} \, a_l(G)$$

The formula is typeset as ar5iv renders the source, and the primary text by Hall was not read. It counts all subgroups of index $k$, not conjugacy classes and not colour symmetry groups up to conjugacy.

### 7.6. Achievable Indices

Edmonds, Ewing, and Kulkarni "determine the possible finite indices of torsion-free subgroups of a Fuchsian group, with respect to its torsion" [BCCC2025, introduction, citing EEK1982]. This rests on a secondary source, and the theorem statement itself was not obtained. They set aside normal subgroups, a question that "appears to involve deeper number theoretic considerations" [EEK1982-BULL] (search summary only).

Burns and Solitar prove some of these results algebraically [BURNS1983], and Izquierdo solved the analogue for NEC groups [IZQUIERDO1993] (both known only by citation, without the statements). A torsion-free subgroup is a special case. No source read gives the set of all achievable indices for an arbitrary orbifold group (Section 14.1).

For a prime number $p$ of colours, Conway, Burgiel, and Goodman-Strauss tabulate the colour types of the plane groups for $p \ne 2$ in Table 13.1 [CBGS2008, p. 164]:

| Group | Types and the primes for which they occur |
| --- | --- |
| `*632` | `*632^3/*333` and `*632^3/2222`, for $p = 3$ |
| `632` | `632^3/333` and `632^3/2222`, for $p = 3$, and `632^p/o` for $p \equiv 1 \pmod 6$ |
| `*442`, `4*2` | No cases |
| `442` | `442^p/o` for $p \equiv 1 \pmod 4$ |
| `*333` | `*333^3/o` and `*333^3/333`, for $p = 3$ |
| `3*3` | `3*3^3/o` and `3*3^3/*333`, for $p = 3$ |
| `333` | `333^3/333` and `333^3/o`, for $p = 3$, and `333^p//o` for $p \equiv 1 \pmod 3$ |
| The other nine plane groups | Rows with $p \equiv 1 \pmod 2$ |

The threefold types are in Table 12.1 [CBGS2008, p. 156], and the book states that the threefold cases missing from Table 13.1 "are precisely those that were done in the previous chapter" [CBGS2008, p. 161]. The twofold types are in Table 11.1 (Section 6). For the spherical groups, a $p$-fold colouring exists "only if p divides its order", and `*532` and `532` have one fivefold colouring and no threefold one [CBGS2008, pp. 166-169, Table 13.2, p. 169]. The book ends its discussion of colourings after the prime numbers of colours [CBGS2008, p. 169], and the research found no table for a composite number of colours in it. It gives no table for hyperbolic groups, which have "infinitely many starting groups" [CBGS2008, p. 227].

### 7.7. Practical Limits

- Frettloh: "Without further refinements, this takes weeks using GAP on a usual PC" [FRETTLOH2008, Section 3] (condensed extraction, page not re-checked). "Practically, computing with finitely presented groups is not efficient. Therefore, there is for each (p^q) a limit on the number of colours to be considered" [FRETTLOH2008, p. 4].
- GAP: the time of `LowIndexSubgroupsFpGroup` grows exponentially with the index [GAP, Section 47.10-1].
- `low_index`: the timings of Section 7.3.
- SymmHub: at most 24 colours (Section 8.6), and the search limits of Section 12.8 of `docs/research/symmhub.md`.
- Coset enumeration: no computable bound on the number of cosets that an enumeration defines (Section 7.15), and the default limits of GAP, SymPy, and ACE (Section 7.16).

### 7.8. Sources for Coset Enumeration

This subsection is non-normative. It says which sources Sections 7.9 to 7.21 rest on and how each was read.

| Source | Kind | How it was read |
| --- | --- | --- |
| [HOLT2013] | Lecture notes, Section 4.2 to Section 6, pp. 14-23 | These pages from a local copy |
| [HULPKE2010] | Lecture notes, Sections III.6 and III.7 and the pages after them to p. 84 | These pages from a local copy |
| [BROWN2011] | Lecture handout | In full from a local copy |
| [HAVAS1991] | Paper on strategies | In full from a local copy |
| [CANNON-HAVAS1992] | Survey paper | In full from a local copy |
| [CHR2007] | Addendum to [NEUBUSER1982], a survey with no algorithm | In full from a local copy |
| [GAP] | Chapter 47 of the reference manual | In full from a local copy |
| [LOWINDEX] | The LaTeX source of the note | In full from a local copy. The README was read through a summarizing tool |
| [SYMPY-CT], [SYMPY-FPG] | Python source code | In full from local copies |
| [SYMMHUB] | JavaScript source code of `lib/sublib` | `sublib_core.js` and `reidemeister.js` line by line, and the section `Compatibility with the GAP-generated files` of `README.md` for one passage (Section 7.19) |
| [ACE], [ITC], [MAGMA], [SYMPY] | Manuals and documentation | Through a summarizing tool that capped quotations at about 125 characters |

The text layers of [HOLT2013] and [HULPKE2010] lose superscripts and subscripts. A quotation from them writes exponents in LaTeX or ASCII, as Section 2.4 states, and its locator says "exponents restored" where the research put them back.

SymPy is an implementation, not a mathematical authority. Its docstrings cite the handbook of Holt, Eick, and O'Brien [HOLT2005] by page, as "pg. 155 [1]", and the paper of Cannon, Dimino, Havas, and Watson [CDHW1973]. Neither the handbook nor that paper was read, so each page that SymPy cites is SymPy's claim about the handbook, and no step that only SymPy states is established by a source (Section 14.1, item 23). The same holds for SymmHub's code. The primary texts [TODD-COXETER1936], [CDHW1973], [NEUBUSER1982], [MENDELSOHN1964], [HOLT2005], and [SIMS1994] were not read (Section 14.3).

### 7.9. The Coset Table

A coset table of a subgroup $H$ of a finitely presented group $G$ "simply lists the cosets and displays the action of the generators and their inverses". "Each row corresponds to a coset, and each column corresponds to a generator or its inverse" [BROWN2011, Section 5, p. 5]. In GAP, "The rows correspond to the cosets, the columns correspond to the generators and their inverses in the ordering g_1, g_1^{-1}, g_2, g_2^{-1}", and an entry $l[i] = j$ means that the generator "takes the coset i to the coset j by multiplication from the right" [GAP, Section 47.6-1]. "Basically a coset table is the permutation representation of the finitely presented group on the cosets of a subgroup" [GAP, Section 47.6-1].

"Coset 1 is defined to be the subgroup. Every other coset is defined to be the image of a prior coset under a generator" [HULPKE2010, p. 72]. While a table is being built, "some of the entries in the table are not known. One common convention is to enter 0 in these places" [BROWN2011, p. 5]. Section 7.21 lists the other conventions.

Holt states three properties that the table keeps during an enumeration [HOLT2013, pp. 15-17, exponents restored]. The table holds a set $\Omega$ of positive integers, and each $i \in \Omega$ has a word $a_i$ in the generators and their inverses with $i = Ha_i$. Always $1 \in \Omega$ and $a_1$ is the empty word. Holt writes $ix$ for the image of $i$ under $x$:

- (R1) "ix = j if and only if jx^-1 = i".
- (R2) "If i -> a_i ... with i in Omega, then 1a_i is defined and equal to i."
- (R3) "If i -> a_i, j -> a_j and ix = j, then Ha_i x = Ha_j."

Proposition 5.1 states that (R3) remains true during the process, and after coincidences "It can be shown that (R1),(R2),(R3) remain true" [HOLT2013, pp. 16-17]. The notes give no proof of the second statement. SymPy lists the same three properties and a fourth, which numbers the cosets from 0, in the docstring of its class `CosetTable` [SYMPY-CT, lines 28-34]. That this list corresponds to Holt's is the research's own reading.

Hulpke also keeps "a table for every relator", which traces "the images of every coset under the relator generator by generator", because "the coset needs to remain fixed", and "a similar table for every subgroup generator", where "we only require the trivial coset to remain fixed" [HULPKE2010, p. 72]. On a computer, the relator and subgroup tables and the inverse columns need not be stored, because their contents can be computed "by scanning through relators ... forwards and backwards" [HULPKE2010, Note III.22, p. 75]. Havas reports that a table needs "(2g - i) x n words" for $n$ cosets and $g$ generators of which $i$ are involutions, because the inverse column of an involution can be dropped [HAVAS1991, p. 192].

A coset is live when no coincidence has eliminated it, and SymPy keeps the "Set of live cosets" as the cosets $c$ with `p[c] == c` [SYMPY-CT, lines 88-91]. "The coset table is called complete if it has no undefined entries on the live cosets" [SYMPY-CT, lines 121-128]. When all tables are closed and no deductions are pending, "the columns of the coset table give permutation images for the group generators that are consistent with all relators", which gives a homomorphism $\varphi : G \to S_n$ with $[G : S] = n$ equal to the number of rows [HULPKE2010, p. 75].

### 7.10. Definitions, Deductions, and Coincidences

Coset enumeration changes the table by three kinds of event. Cannon and Havas state the rules that drive them: "(a) coset 1 must be mapped to itself by each of h1, ..., ht" and "(b) each coset j must be mapped to itself by each of the defining relators R1, ..., Rs and by each product xk xk^-1", where $h_1, \dots, h_t$ generate $H$. "Enforcement of rules (a) and (b) yields values for some hitherto unknown coset table entries (deductions) and, also, the identification of cosets which have been multiply defined (coincidences)" [CANNON-HAVAS1992, Section 2.2, p. 3].

A definition adjoins a new coset. Holt states it as follows: "If some ix is not defined then we can choose some j > 0, j not in Omega, replace Omega by Omega u {j} and define ix = j, jx^-1 = i and put a_j = a_i x" [HOLT2013, p. 15, exponents restored]. These steps restate that definition:

1. Choose a live coset $i$ and a column $x$ with $i^x$ undefined.
2. Adjoin a new coset number $j$.
3. Set $i^x = j$ and $j^{x^{-1}} = i$.
4. Record the representative word $a_j = a_i x$.

A deduction is an entry that a relator or subgroup generator forces. Hulpke writes that when an entry "fills the last hole in a table", "we get an deduction that the image of one coset under the next generator must be the existing value on the other side of the table entry. We enter these deductions in the table as if they were definitions" [HULPKE2010, p. 73]. A coincidence is the discovery that two coset numbers name one coset: "Sometimes we get a deduction ix = j where we already know ix = k with k != j, (or kx = j for k != i). This means coset Ha_j and Ha_k (respectively Ha_i, Ha_k) are equal" [HOLT2013, p. 17, exponents restored]. Section 7.21 lists the other meanings of "deduction" in the sources.

Hulpke describes scanning a relator "forwards and backwards" in prose [HULPKE2010, p. 75], and no PDF source states the scan in steps. SymPy's code states it and cites p. 155 of [HOLT2005] [SYMPY-CT, lines 286-379]. These steps restate it for a coset $\alpha$ and a word $w$, which is a relator or a subgroup generator:

1. Find the longest prefix $s$ of $w$ for which $\alpha^s$ is defined, and write $w = sv$.
2. Find the longest suffix $t$ of $v$ for which $\alpha^{t^{-1}}$ is defined, and write $v = ut$.
3. If $t = v$, "the scan completes", and "if, in addition $\alpha^s = \alpha^{t^{-1}}$, then we say that the scan completes correctly". If it completes and the two cosets differ, they are a coincidence, which the code passes to coincidence processing (Section 7.13).
4. If $u$ is a single letter $x$, set $\beta^x = \gamma$ and $\gamma^{x^{-1}} = \beta$ for $\beta = \alpha^s$ and $\gamma = \alpha^{t^{-1}}$. "These assignments are known as deductions and enable the scan to complete correctly."
5. Otherwise "scan is incomplete and yields no information" (line 379).

### 7.11. The Relator-Based (HLT) Strategy

A strategy decides where the next definition goes. The relator-based strategy is "often called the HLT method (after Haselgrove, Leech, and Trotter). In this method we consider the vertices in numerical order, and we make sure that the graph is complete at each vertex before moving on to the next" [BROWN2011, p. 4]. Holt's second strategy is to "Go through relator tables in order filling in gaps as you go (even for long gaps)" [HOLT2013, p. 18]. Hulpke states HLT more narrowly: "If there are gaps of length 1 in a subgroup or relator table, fill these gaps (in the hope of getting immediately a consequence)" [HULPKE2010, p. 77]. Section 7.21 records these differences.

SymPy's scan-and-fill makes the definitions of the strategy: "whenever the procedure is called and the scan is incomplete then it makes new definitions to enable the scan to complete" [SYMPY-CT, lines 597-607, citing pp. 162-163 of [HOLT2005]]. The code runs the scan of Section 7.10 in a loop. When the scan is incomplete and has more than one gap, it defines a new coset at the first undefined position of the forward trace and scans again, until the scan completes or deduces [SYMPY-CT, lines 343-378].

SymPy's HLT enumeration `coset_enumeration_r` has these steps [SYMPY-CT, lines 969-1165]:

1. Scan-and-fill each subgroup generator at coset 0, which is $H$.
2. For each coset $\alpha$ in increasing order, while $\alpha$ is below the largest live coset number plus one:
   1. If $\alpha$ is live, scan-and-fill each relator at $\alpha$ in order, and stop this loop when a coincidence eliminates $\alpha$.
   2. If $\alpha$ is still live, define a new coset for each column of row $\alpha$ that is still undefined.
3. Return the table.

Holt compares the strategies: the relator-based one "leads to many more coincidences, but it is easier to program and runs fast on computers for routine examples", and "each row of each relator table only needs to be scanned once" [HOLT2013, p. 18]. "The order in which cosets are defined is explicitly prescribed by the order in which (the subgroup generators and) the group relators are processed in HLT-type procedures" [HAVAS1991, p. 192].

A variant adds lookahead: "we check all relators at all vertices (but without making new definitions) and take account of any deductions or coincidences that we can discover" [BROWN2011, p. 4]. SymPy's `look_ahead` scans every relator at every live coset without defining [SYMPY-CT, lines 654-671]. Havas reports that "It has been found to be a good general rule to use the given group relators as subgroup generators", because this "ensures the early definition of some useful cosets" [HAVAS1991, p. 193].

### 7.12. The Coset-Table-Based (Felsch) Strategy

The Felsch strategy defines the first undefined entry: "Define the next coset as the first open entry (by rows and within a row by columns) in the coset table. This guarantees that the image of each coset under each generator will be defined at some point" [HULPKE2010, p. 77]. Holt states the same strategy, which defines "cosets Ha_i is order of increasing a_i in the lenlex order" [HOLT2013, p. 17]. "After each definition, we check all relators at all vertices (without making further definitions) that could possibly be affected by the definition we've just made" [BROWN2011, p. 4]. GAP's built-in enumerator "performs a standard Felsch strategy coset enumeration" [GAP, Section 47.6-5].

Hulpke states which relators a new entry can affect. "We form a list of all cyclic permutations of all relators and their inverses and store these according to the first letter occurring in the permuted relator", and with $R^c_g$ the set of those that start with $g$, "if the image of coset y under generator g is defined as z, we scan all relators in R^c_g starting at the coset y and all relators in R^c_{g^-1} starting at z" [HULPKE2010, Note III.22, pp. 75-76, exponents restored].

SymPy's Felsch enumeration `coset_enumeration_c` adds a deduction stack, a list of pairs $(\alpha, x)$ for entries assigned and not yet checked [SYMPY-CT, lines 78-80 and 1202-1260, citing p. 166 of [HOLT2005]]. It has these steps:

1. Cyclically reduce each relator, form the cyclic conjugates of it and of its inverse, and group them by first letter into lists $R^c_x$, one for each column $x$.
2. Scan-and-fill each subgroup generator at coset 0, pushing each deduction on the stack. Then, for each column $x$ in order, process the deductions with the lists $R^c_x$ and $R^c_{x^{-1}}$ as arguments, which is $2g$ calls for $g$ generators (lines 1241-1244).
3. For each coset $\alpha$ in increasing order, if $\alpha$ is live, take each column $x$ in order. Stop when $\alpha$ is no longer live. If $\alpha^x$ is undefined, define it, push $(\alpha, x)$, and process the deductions with the lists $R^c_x$ and $R^c_{x^{-1}}$ as arguments (lines 1245-1259).

Processing the deductions takes two lists $L$ and $L'$ from the caller and has these steps [SYMPY-CT, lines 673-704]:

1. While the stack is not empty, do the following steps.
2. If the stack holds `max_stack_size` entries or more, run the lookahead of Section 7.11, empty the stack, and continue. The default of `max_stack_size` is 100 (line 54).
3. Pop $(\alpha, x)$. If $\alpha$ is live, scan $\alpha$ with each word of $L$, and stop when $\alpha$ dies.
4. Let $\beta = \alpha^x$. If $\beta$ is defined and live, scan $\beta$ with each word of $L'$.

The code scans every popped pair with the lists of its caller and does not select the lists by the popped $x$. A pair that a scan or a coincidence pushed with another column is therefore scanned with the lists of the column of the current call. This is an observation of the code, and no source read says whether it matters (Section 14.1, item 23). Hulpke's rule scans with the relators that start with the generator of the entry (Note III.22, above).

Each scan pushes the entry that it deduces, and coincidence processing pushes $(\delta, x^{-1})$ for each entry that it moves [SYMPY-CT, lines 242 and 273]. No PDF source states the deduction stack. Havas reports that implementations expand "each relator out three times to avoid having to check for end of relator during deduction processing" [HAVAS1991, p. 195], and that "Felsch-type programs generally start off by ensuring that each of the given subgroup generators forms a cycle at coset 1" so that "subgroup generators can be ignored thereafter" [HAVAS1991, p. 193].

### 7.13. Coincidence Processing

When two coset numbers name one coset, the larger is eliminated: "If we find Ha_j = Ha_k with k > j, then we replace all k by j in tables which often leads to more coincidences" [HOLT2013, p. 17, exponents restored]. "We thus keep a list of coincidences to process and add to it any such further identifications and process them one by one" [HULPKE2010, p. 76]. ACE also makes "the larger coset number" dead and places the coincidence "in a queue" [ACE, Section 3.2] (summarizing tool).

SymPy keeps the classes of coset numbers in an array `p` with "`p[\alpha] \sim \alpha` and `p[\alpha]=\alpha \iff \alpha=rep(\alpha)`", finds a representative by following `p` "until it becomes constant", and compresses each path it follows [SYMPY-CT, lines 466-534, citing p. 157 of [HOLT2005]]. Its `merge(k, lamda, q)` finds the representatives $\phi$ and $\psi$ of the two numbers and, when they differ, sets `p[max] = min` and appends the larger to the queue `q` [SYMPY-CT, lines 453-464]. The docstring says instead that "It is more efficient to choose the new representative from the larger of the two classes" (lines 430-431), and the code does not do that.

SymPy's coincidence processing for $\alpha \sim \beta$ has these steps [SYMPY-CT, lines 538-594, citing pp. 156-161 of [HOLT2005]]:

1. Create an empty queue $q$ and merge $\alpha$ and $\beta$.
2. While $q$ is not empty, remove its first coset $\gamma$.
3. For each column $x$ with $\delta = \gamma^x$ defined, clear $\delta^{x^{-1}}$, and let $\mu$ and $\nu$ be the representatives of $\gamma$ and $\delta$. Then do the first of these that applies:
   1. If $\mu^x$ is defined, merge $\nu$ and $\mu^x$.
   2. If $\nu^{x^{-1}}$ is defined, merge $\mu$ and $\nu^{x^{-1}}$.
   3. Otherwise set $\mu^x = \nu$ and $\nu^{x^{-1}} = \mu$.

The two merges in step 3 queue the consequent coincidences. The Felsch variant also pushes $(\delta, x^{-1})$ on the deduction stack, its "only line of difference" [SYMPY-CT, lines 247-282]. Havas reports that coincidence techniques due to Beetham removed the need for two extra columns, at the cost that "there is no longer easy availability of an active row list" [HAVAS1991, p. 192], and that pending coincidences can be discarded when the table collapses to one coset [HAVAS1991, p. 195].

### 7.14. Compression and Standardization

Coincidences leave gaps in the numbering: "It is customary to remove the dead rows at the end of the process and to renumber the cosets so that there are no gaps. This is often called compression in the literature" [BROWN2011, p. 5]. SymPy's `compress` lists the dead cosets, renumbers the live ones consecutively, truncates the table, and lowers each entry by the number of dead cosets below it [SYMPY-CT, lines 800-832, citing p. 167 of [HOLT2005]]. Its line 824 is a comparison with no effect, as the comment at lines 818-823 states with a link to SymPy issue 27633.

A standardized table numbers its cosets in a fixed order of first appearance. Hulpke defines: "A coset table is standardized if when running through the cosets and within each coset through the generator images (ignoring generator inverses), the cosets appear in order of the integers 1, 2, 3, ...." [HULPKE2010, Definition III.25, p. 78]. Lemma III.26 states: "There is a bijection between subgroups of G of index n and standardized coset tables for G with n cosets" [HULPKE2010, p. 78]. A table is brought into this form "by running through cosets and within cosets through generator images and reassigning new labels according to the order in which cosets appear" [HULPKE2010, p. 78].

GAP calls a table standard when, for $i < j$, "the first occurrence of i in a coset table precedes the first occurrence of j with respect to the usual row-wise ordering of the table entries", following [SIMS1994], and equivalently when the representative words "are length-plus-lexicographic ordered" [GAP, Section 47.7]. GAP's default, "lenlex", uses "all their columns as defined in Charles Sims' book", while "semilenlex" uses "only their generator columns (this was the original GAP standard)" [GAP, Section 47.7-1]. `StandardizeTable` alters the given table and takes the standard as an optional argument [GAP, Section 47.7-2]. GAP does not describe its procedure.

SymPy's `standardize` has these steps, after `compress` [SYMPY-CT, lines 752-797, citing pp. 167-168 of [HOLT2005]]:

1. Set $\gamma = 1$.
2. For each coset $\alpha$ in increasing order, and each column $x$ in the order $g_1, g_1^{-1}, g_2, g_2^{-1}, \dots$, let $\beta = \alpha^x$.
3. If $\beta \ge \gamma$, swap the labels $\gamma$ and $\beta$ in every row and column when $\beta > \gamma$, then increase $\gamma$ by 1, and stop when $\gamma$ equals the number of cosets.

The swap is `switch`, which exchanges two rows and the two labels in every entry [SYMPY-CT, lines 728-750]. The code visits every column, which is GAP's lenlex standard, while the docstring defines the standard "ignoring generator inverses", which is semilenlex. The doctest of Section 7.20 gives a lenlex table [SYMPY-CT, lines 776-784]. Hulpke's Lemma III.26 is stated for his definition, which ignores inverses (Section 14.1, item 26).

### 7.15. Termination and Correctness

Holt states what the procedure does: it "tries to prove that |G : H| is finite and to compute the coset action of G on the cosets of H. If |G : H| is infinite it will not halt. If |G : H| is finite it will succeed but there is no upper bound on the time taken" [HOLT2013, p. 15]. When it stops, Theorem 5.2 gives "an action theta of G on Omega" that "is equivalent to the coset action of G on right cosets" of $H$, with $i \mapsto Ha_i$ [HOLT2013, Theorem 5.2, p. 16]. The proof starts "When the process stops all ix are defined."

Termination for finite index needs a condition on the strategy. Holt states "(R4) for all i in Omega, a in A, ia will eventually be defined" and "Theorem 5.3. If |G : H| is finite and definition strategy satisfies (R4) then it will eventually finish" [HOLT2013, p. 18, exponents restored]. He adds that (R4) "is satisfied with 1 but not necessarily with 2 so we must occasionally use 1", where 1 is the Felsch strategy and 2 the relator-based one [HOLT2013, p. 18]. Hulpke attributes the result to Mendelsohn:

> Theorem III.24 (Mendelsohn, 1964): Suppose that [G:S] < infinity and that the strategy used guarantees that for every defined coset a and every generator g the images a^g and a^(g^-1) will be defined after finitely many steps, then the coset enumeration terminates after finitely (but not bounded!) many steps with the correct index. [HULPKE2010, pp. 77-78, exponents restored]

Hulpke gives only a proof idea and no bibliography entry for Mendelsohn's paper [MENDELSOHN1964], which was not read. The ACE manual names the condition "Mendelsohn's condition" and locates it in Lemma 1 and Theorem 2 of [NEUBUSER1982] [ACE, Chapter 3] (summarizing tool). The ITC manual states that "The Felsch strategy automatically fulfills the "Mendelsohn Condition"", while "by a pure HLT strategy the Mendelsohn Condition is not guaranteed. The CE may run into infinite loops although the index of h in g is finite" [ITC, Section 3.2] (summarizing tool). Havas warns that preferred definitions made carelessly "could lead to violation of the conditions required for guaranteed termination", that implementations therefore fill "at least a certain initial fraction of the coset table (in order row by row)" first, and that "It is easy to construct examples where omission of a control of this type does indeed lead to infinite looping" [HAVAS1991, p. 194]. No source read states whether SymPy's HLT loop, which defines every open entry of a row after its scans (Section 7.11), meets the condition (Section 14.1, item 20).

No computable bound exists on the work. "For a given coset enumeration procedure, there is no computable bound, in terms of length of input and a hypothetical index, to the number of cosets which need to be defined in the coset enumeration process to complete the enumeration. (The existence of such a bound would violate the unsolvability of the word problem for finitely presented groups.)" [CANNON-HAVAS1992, Section 2.2, p. 3]. Havas states the same, and that "Sims [15] has proved that there is no polynomial bound in terms of maximum number of cosets for the number of coset tables which can derived by simple coset table operations" [HAVAS1991, p. 192]. Neither paper proves the statement or cites a proof beyond the parenthesis.

GAP states that "A coset enumeration will not finish if the subgroup does not have finite index, and even if it has it may take many more intermediate cosets than the actual index of the subgroup is" [GAP, Section 47.6-6]. Cannon and Havas measure performance "by the maximum number of cosets that are simultaneously defined and by the total number of cosets defined during the course of the enumeration" [CANNON-HAVAS1992, p. 4]. If the enumeration terminates for a subgroup $H$ known to be finite, "we immediately deduce that G is finite and, moreover, we obtain a bound on the order of G" [CANNON-HAVAS1992, p. 4].

No source read states what a run that reaches a limit on the number of cosets shows. The sources state only what the implementations do then (Section 7.16), and ACE's manual shows a finite-index enumeration that overflows at a small limit and succeeds at a larger one [ACE, Appendix B.4] (summarizing tool). This document therefore does not state that reaching a limit shows anything about the index (Section 14.1, item 19).

### 7.16. Limits and Defaults of Implementations

This table gives the limit on the number of cosets of each implementation and what it does on reaching it.

| Implementation | Strategy | Limit and default | On reaching the limit |
| --- | --- | --- | --- |
| GAP | Felsch by default [GAP, Section 47.6-5] | Option `max`, "The limit of the number of cosets to be defined", default `CosetTableDefaultMaxLimit` = 4096000. "(Due to the algorithm the actual limit used can be a bit higher than the number given.)" `CosetTableDefaultLimit` = 1000 is the initial allocation [GAP, Sections 47.6-5 to 47.6-7] | "an error is raised and the user is asked whether she wants to continue", with an offer of a new limit of 8192000. With `silent`, it returns `fail`. A limit of 0 or infinity runs until "they either get a result or exhaust the whole available space" [GAP, Sections 47.6-5 and 47.6-6] |
| SymPy | HLT in `coset_enumeration_r`, Felsch in `coset_enumeration_c` | `coset_table_max_limit` = 4096000, "default limit for the number of cosets allowed in a coset enumeration", overridden by the keyword `max_cosets` [SYMPY-CT, lines 47-49] | `define` raises `ValueError` with the text "the coset enumeration has defined more than %s cosets". With `incomplete=True`, the enumeration returns the partial table [SYMPY-CT, lines 152-156, 1160-1163, and 1255-1258] |
| ACE | Several, including HLT and Felsch [ACE, Chapter 5] | `max` "Sets the maximum coset number that can be defined", 0 or at least 2, and by default "all of the workspace is used" [ACE, Section 4.17] | `OVERFLOW`, "out of table space" [ACE, Appendix A.2] |
| Magma | Not recorded | Not recorded | The index returned is "0, if the enumeration fails to complete", or an incomplete coset space [MAGMA] |

The rows for ACE and Magma were read through a summarizing tool. SymPy compares the limit with the length of the table, which counts every row ever defined, dead rows included, until compression. That reading is the research's own, from the code. SymPy's documentation gives 500 as the default of `max_stack_size`, while the code sets 100 [SYMPY] (summarizing tool) [SYMPY-CT, line 54].

### 7.17. The Low-Index Subgroup Search

The search runs through a tree of partial coset tables. SymPy describes it as "a backtrack search over incomplete Coset Tables, rather than over forced coincidences", citing [SIMS1994] [SYMPY-FPG, lines 737-738]. In Hulpke's account, a node is "a partially completed, standardized coset table, involving k <= n cosets. (The initialization is with the empty coset table.) If the table is in fact complete, it yields a subgroup" [HULPKE2010, p. 79]. Culler, Dunfield, and Gorner state the tree in terms of covering spaces: "The nodes of the Sims tree are covering subgraphs of B and the root of the tree is B, viewed as a degree 1 covering of itself", and a terminal node is kept when "the base vertex is canonical and the relators all lift to all vertices" [LOWINDEX, Section 4]. Their note does not state how a node is extended.

Hulpke's Algorithm III.27, "a basic version of the low index algorithm without elimination of conjugates", takes $G$ and an index $n$ and outputs "All subgroups of G of index up to n, given by coset tables" [HULPKE2010, pp. 79-80]. Its table starts empty, and the research reads the first branch as adding coset 1, which the algorithm does not state. SymPy's root table already holds one row of undefined entries, coset 0 [SYMPY-FPG, line 774] [SYMPY-CT, lines 63-70]. These steps restate the extension of a node, from [HULPKE2010, pp. 79-80] and [SYMPY-FPG, lines 790-838]:

1. If the table is complete, check the long relators at every live coset (SymPy only). Output the table if they all hold.
2. Otherwise let $x$ and $g$ be the coset and the column of the first undefined entry, by rows and within a row by columns.
3. For each existing coset $y$ in increasing order with $y^{g^{-1}}$ undefined, and then a new coset $k + 1$ if the table has $k < n$ cosets, copy the table, set $x^g = y$ and $y^{g^{-1}} = x$, and push $(x, g)$ on an empty deduction stack.
4. Process the deductions with the short relators. If a coincidence or a contradiction arises, abandon the copy. Hulpke: "if a coincidence occurs, we know that we made an invalid choice, and abandon this partial table, backtracking" [HULPKE2010, p. 79].
5. Check that each subgroup generator of the argument `Y` fixes coset 0 (SymPy only).
6. If the copy passes the first-in-class test, recurse on it.

SymPy's deductions in the search use `scan_check`, which returns false when a scan completes incorrectly, without coincidence processing, and writes the deduction of a single gap [SYMPY-CT, lines 382-424]. The deduced entry is not pushed on the deduction stack, which the research observed in the code. No source read says whether this is intended or whether it affects the result (Section 14.1, item 23).

SymPy splits the relators by length, with `len_short_rel = 5`. The relators of length at most 5 prune "branches of the search tree", and the longer ones are "only checked at the last step for complete coset tables" [SYMPY-FPG, lines 776-784]. Hulpke states the reason and no threshold: "as long relators rarely yield a deduction but only are conditions to test, it can make sense to only consider the shorter (whatever this means) relators for the determination of coset tables and simply test each table obtained afterwards for the remaining relators" [HULPKE2010, Performance III.28, p. 81]. The package `low_index` takes the two sets of relators from its caller [LOWINDEX, README], and Magma states that "A relator designated as long is not used during the construction of a coset table" [MAGMA] (both through a summarizing tool).

The first-in-class test keeps one table from each conjugacy class. Hulpke defines "a (somehow arbitrary) order on coset tables which tests conveniently for partial coset tables", which is "lexicographic, considering the table row by row", and tests "for each (partial) table" "whether it can be the smallest in its class" [HULPKE2010, pp. 80-81]. A conjugate subgroup is the stabilizer of another coset $x$, and its table arises by relabelling with $x$ as coset 1 [HULPKE2010, pp. 80-81]. SymPy's `first_in_class`, which "is taken directly from the function on page 208 of [Sim94]", has these steps [SYMPY-FPG, lines 841-954]:

1. For each coset $\alpha$ other than coset 0, skip $\alpha$ unless $\alpha^w = \alpha$ for every subgroup generator $w$ of `Y`.
2. Relabel the cosets by first appearance, starting with $\alpha$ as coset 0, while reading the table row by row and column by column.
3. At each entry, compare the relabelled entry of the row that corresponds to the current row with the entry $\gamma$ of the original table. If either is undefined, go to the next $\alpha$. If the relabelled entry is smaller, return false. If it is larger, go to the next $\alpha$. If they are equal, continue.
4. Return true when every $\alpha$ is done.

The docstring states that if the test "returns False, then no descendant of C can have that property, and so we can abandon C" [SYMPY-FPG, lines 841-859]. Culler, Dunfield, and Gorner define a canonical vertex as one with "minimal complexity among all vertices", where the complexity is a tuple ordered lexicographically, and extend it to partial graphs "by defining V_s to be 0 if the slot s is empty" [LOWINDEX, Section 3.4]. Their rule thus compares empty slots as 0, while SymPy stops at an undefined entry. No source read proves that pruning a partial table by either rule loses no class (Section 14.1, item 22).

GAP's search takes a subgroup $H$ and returns only subgroups that contain it, and with `excluded` only subgroups that "do not contain any conjugate" of the given elements [GAP, Section 47.10-1]. GAP does not say how it uses either argument during the search. SymPy's `first_in_class` skips a base point unless the conjugate "definitely contains the given subgroup" [SYMPY-FPG, lines 911-924], and a comment there says that it supports only subgroup generators that are single generators or their inverses [SYMPY-FPG, lines 916-917]. The search as a whole scans any word of `Y` at coset 0 (lines 834-836). Hulpke's algorithm has no subgroup argument. The `low_index` note takes a covering subgraph $C$ as an input, which is a starting node of the search and not a subgroup [LOWINDEX, Section 4].

The result is one table or one subgroup for each conjugacy class:

- GAP returns "the list of these subgroups", and "By default, the algorithm computes no generating sets for the subgroups" [GAP, Section 47.10-1].
- SymPy returns a list of complete coset tables, one for each class, in the order of the depth-first search [SYMPY-FPG, lines 751-787].
- The `low_index` note states that "each conjugacy class of subgroups of G with index at most N contains a unique representative", and that its implementation "actually outputs a list of permutation representations" with the canonical vertex labelled 1 [LOWINDEX, Section 4].
- Cannon and Havas: the algorithm "outputs either a generating set for each subgroup of index n, or a list of all transitive permutation representations of G having degree n" [CANNON-HAVAS1992, p. 5].

No mathematical source read states a rule for the number of subgroups in a class. SymmHub's code computes it as the number of distinct standardizations of the table over all base points [SYMMHUB, `sublib_core.js`, lines 373-384]. Hulpke gives no algorithm for normal subgroups and suggests computing quotients instead [HULPKE2010, Note III.29, p. 81].

### 7.18. Reidemeister-Schreier Rewriting

Reidemeister-Schreier rewriting gives a presentation of a subgroup of finite index from a complete coset table. Hulpke chooses representatives $t_1 = 1, \dots, t_n$ of the cosets and forms the Schreier generators, each the product of $t_i g_j$ with the inverse of the representative of the coset of $t_i g_j$ [HULPKE2010, pp. 81-82]. "If the coset representative t_i is defined as t_i = t_j g_x, the Schreier generator s_j,x is trivial by definition. We call the pair (j, x) "redundant"", which leaves $n(m - 1) + 1$ generators for $m$ generators of $G$ [HULPKE2010, p. 82]. Theorem III.30 (Reidemeister) gives the subgroup as generated by the Schreier generators, with the relators $\rho(t_x r_y t_x^{-1})$ for every representative $t_x$ and every relator $r_y$ of $G$, where $\rho$ rewrites a word in the Schreier generators [HULPKE2010, Theorem III.30, p. 82]. Corollary III.31 is the Nielsen-Schreier rank formula [HULPKE2010, p. 82].

"To perform this rewriting in practice, it is easiest to form an augmented coset table by storing (for entries that are not definitions) in position for coset x and generator g also the appropriate Schreier generator s", and "We then scan every relator at every coset and collect the occurring Schreier generators" [HULPKE2010, p. 83]. The result has "many trivial Schreier generators and relators", so it is "typically" processed by Tietze transformations [HULPKE2010, Note III.34, p. 83]. Hulpke names "a natural limit (a few thousand) on the subgroup index for which rewriting is feasible" [HULPKE2010, Performance III.38, p. 84].

Holt builds the transversal from the table: "Scan the rows, and underline new numbers, regard the underline of numbers as definitions of these numbers", even when the enumeration already marked definitions, "since otherwise might not get Schreier transversal" [HOLT2013, pp. 19-20]. Each entry that is not a definition gets a Schreier generator, and "If rows of table are cyclic conjugate then so are the resulting relators, so only need one of them" [HOLT2013, p. 21]. The idea of the proof of Corollary 6.7 warns that "this is not a formal proof" [HOLT2013, p. 21].

SymPy's `reidemeister_presentation` has these steps [SYMPY-FPG, lines 1141-1362]:

1. Enumerate the cosets with the HLT strategy if no table is given, then compress and standardize the table.
2. Read the table row by row. An entry equal to the next new coset number is a tree edge, with the representative of the new coset equal to that of the row times the generator. Every other entry of a generator column gets a new Schreier generator.
3. Rewrite every relator at every coset.
4. Remove relators of length 1 with their generators, and remove relators that are cyclic conjugates of others.
5. Simplify the presentation.

Step 2 recognizes a tree edge by the next new number, so the research infers that it needs a standardized table. The code does not say so. GAP's Section 47.9 names the functions `AugmentedCosetTableMtc`, `AugmentedCosetTableRrs`, and `RewriteWord` without the steps, and the augmented coset table of its Chapter 48 was not read [GAP, Section 47.9].

### 7.19. SymmHub's Implementation

SymmHub implements the low-index search and Reidemeister-Schreier rewriting in `lib/sublib/src/sublib_core.js` and `lib/sublib/src/reidemeister.js` [SYMMHUB]. Section 12.8 of `docs/research/symmhub.md` gives the data shapes, the functions, the limits, and the defects of this code. The code reading found no Todd-Coxeter enumeration in `lib/sublib`, and records the search that supports this in the same section. The header comment calls the code a "Sims-style low-index coset-table backtrack search" (`sublib_core.js`, line 15).

The search of the class `LowIndexSearch` has these steps [SYMMHUB, `sublib_core.js`, lines 199-331]:

1. Cyclically reduce each relator, drop empty ones, and keep them in input order. There is no split into short and long relators.
2. Start from a table with one coset, coset 0, and no entry defined.
3. At a node, find the first undefined entry in row-major order, with the columns in the order $g_1, g_1^{-1}, g_2, \dots$. If there is none, test canonicity again and output a copy of the table.
4. Try each existing coset $d$ in increasing order whose entry in the inverse column is undefined, including the row's own coset, and then a new coset if the table has fewer than `maxIndex` cosets.
5. For each candidate, define the entry and its inverse, propagate, test canonicity, recurse if both pass, and undo the definitions through a trail of changed entries.

Propagation scans each relator from every coset in turn. When the pass of a relator changed an entry, it restarts from the first relator, and it ends after a round that changes nothing. A scan that traces a whole relator fails unless it returns to its coset, and a scan with one gap defines that entry. A definition that clashes with an existing entry fails [SYMMHUB, `sublib_core.js`, lines 235-280]. The canonicity test `isCanonical` follows the same rule as SymPy's `first_in_class` (Section 7.17): it renumbers from each other base point, returns false when the renumbered table is smaller, and leaves a base point undecided at an undefined entry [SYMMHUB, `sublib_core.js`, lines 351-371]. The comment above it states that on a complete table the test is exactly "this table is the least in its class", and that on a partial table "a prefix that already loses still loses however it is completed, so cutting the branch there cannot discard a canonical table" [SYMMHUB, `sublib_core.js`, lines 341-346]. These are the code's own claims, and the research did not check them (Section 14.1, item 22).

This table compares the search with the sources of Section 7.17.

| Aspect | SymmHub | Sources |
| --- | --- | --- |
| Todd-Coxeter enumeration | None | GAP, SymPy, ACE (Section 7.16) |
| Coincidences | None. A clash ends the branch | A coincidence ends the branch [HULPKE2010, p. 80] |
| Deductions after a definition | Full rescan of all relators at all cosets | A deduction stack over the cyclic conjugates by first letter (Section 7.12) |
| Short and long relators | No split | SymPy splits at length 5. Hulpke writes that "it can make sense to" split |
| Subgroup argument | None | GAP `H`, SymPy `Y` |
| Canonicity on partial tables | Stops at an undefined entry | SymPy stops there. `low_index` compares empty slots as 0 |
| Index bound | `maxIndex`, at most 62 through `subgroupsData` | GAP `index`, SymPy `N` |

The comment at `sublib_core.js`, lines 21-27, states that the conventions are "identical to GAP's": rows are cosets numbered from 0, the columns are $g_1, g_1^{-1}, g_2, \dots$, and cosets are numbered in order of first appearance over all columns. The code does not check that the tables of the search are standardized in that sense (Section 14.1, item 24). The comment at lines 31-33 states that the representative of a class "agrees with GAP's representative for ~99% of subgroups", and that where it differs the two tables are conjugate. The section `Compatibility with the GAP-generated files` of `lib/sublib/README.md` states instead that "Coset permutations agree verbatim for 97% of the corpus' 110 524 subgroups", with 47 739 of 50 986 in the family `klm` and all in `wallpaper` and `*klm` (lines 310-312). Both figures are claims of the project. The research ran no comparison with GAP and read the README for this passage only.

### 7.20. Worked Examples

These examples state their numbers in the source and can serve as tests. Tables are written as their sources print them, and Section 7.21 gives the conventions. Section 14.1, item 25, lists the group orders that no source states.

These examples are enumerations by hand in the notes:

| Source | Presentation | Subgroup | Stated result |
| --- | --- | --- | --- |
| [BROWN2011, pp. 1-5] | $\langle a, b \mid a^3 = b^3 = (ab)^2 = 1 \rangle$ | $\langle a \rangle$ | Index 4, $G \cong A_4$, order 12. Four cosets defined, no coincidence, and the final table "happened to come out in standard form" |
| [HULPKE2010, Examples III.20 and III.21, pp. 72-75] | $\langle a, b \mid a^2 = b^3 = (ab)^5 = 1 \rangle$ | $\langle a, b^{-1} a b \rangle$ | Index 6, $a \mapsto (2,4)(5,6)$, $b \mapsto (1,2,3)(4,5,6)$. Definitions $1b = 2$, $1b^{-1} = 3$, $2a = 4$, $4b = 5$, $4b^{-1} = 6$. The text does not say in words that no coincidence arises, which the research infers from the example. Example III.35 gives the order of $G$ as $6 \cdot 10 = 60$ [p. 84] |
| [HULPKE2010, Example III.23, pp. 76-77] | The same | The same | From a partial table with 6 rows, the definition $1b^{-1} = 7$ leads to the coincidence of cosets 5 and 6, with "no subsequent coincidence" |
| [HOLT2013, p. 15] | $\langle x, y \mid x^2, y^3, xyxy \rangle$ | $\langle xy \rangle$ | Index 3, $x \mapsto (1,2)$, $y \mapsto (1,2,3)$, $\lvert G \rvert = 6$ |
| [HOLT2013, pp. 16-17] | $\langle x, y \mid x^3, y^4, (xy)^2 \rangle$ | $\langle y \rangle$ | Index 6, $G \cong S_4$. Definitions $1x = 2$, $2x = 3$, $3y = 4$, $4y = 5$, $5x = 6$ |
| [HOLT2013, p. 17] | $\langle a, b \mid a^{-1}ba = b^3, b^{-1}ab = a^3 \rangle$ | $\langle a \rangle$ | Index 2, $b \mapsto (1,2)$, $a \mapsto$ identity, $\lvert G \rvert = 8$, the quaternion group. Three cosets defined, with the coincidence of 2 and 3 |
| [HOLT2013, pp. 21-22] | $\langle a, b \mid a^3, b^5, (ab)^2 \rangle$ | $\langle b, ab^{-1}ab^2a^{-1} \rangle$ | Index 6, $\lvert H \rvert = 10$, $\lvert G \rvert = 60$, $G \cong A_5$ |
| [HOLT2013, pp. 22-23] | $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$ | $\langle xy, x^{-1}y^{-1}xyx \rangle$ | $H \cong D(2,3,3)$ of order 12, $\lvert G \rvert = 60$. The table has 5 rows, and the notes do not state the index in words |

Brown's final table, numbered from 1, with columns $a, a^{-1}, b, b^{-1}$ [BROWN2011, p. 5]:

```text
    a A b B
1   1 1 2 3
2   3 4 3 1
3   4 2 1 2
4   2 3 4 4
```

Hulpke's final table, with the columns $a, a^{-1}, b, b^{-1}$ [HULPKE2010, p. 75]:

```text
1: 1 1 2 3
2: 4 4 3 1
3: 3 3 1 2
4: 2 2 5 6
5: 6 6 6 4
6: 5 5 4 5
```

GAP's example of `FactorCosetAction` gives the same permutations, `[ a, b ] -> [ (2,4)(5,6), (1,2,3)(4,5,6) ]`, for the subgroup generated by `g.1` and `g.1^g.2` [GAP, Section 47.6-3]. The table that Hulpke prints after the coincidence of Example III.23 has the entry 3 in row 5, column $a^{-1}$, while row 5 has $a = 4$. With $a^2 = 1$ the entry would be 4, so the research reads it as a misprint (Section 14.2).

These examples are SymPy doctests, numbered from 0, with the columns $x, x^{-1}, y, y^{-1}$. The index is the number of live rows, because no docstring states it in words [SYMPY-CT, lines 1000-1115]:

| Label in SymPy | Relators | Subgroup | Stated result |
| --- | --- | --- | --- |
| "Example 5.1" | $x^3, y^3, x^{-1}y^{-1}xy$ | $\langle x \rangle$ | Rows `[0, 0, 1, 2]`, `[1, 1, 2, 0]`, `[2, 2, 0, 1]`, and `C.p` = `[0, 1, 2, 1, 1]`, so 5 cosets defined and 3 live |
| "Example 5.2" | $x^2, y^3, (xy)^3$ | $\langle xy \rangle$ | Rows `[1, 1, 2, 1]`, `[0, 0, 0, 2]`, `[3, 3, 1, 0]`, `[2, 2, 3, 3]` |
| "Example 5.3" | $x^2y^2, x^3y^5$ | Trivial | Rows `[1, 3, 1, 3]`, `[2, 0, 2, 0]`, `[3, 1, 3, 1]`, `[0, 2, 0, 2]`, and after `standardize` `[1, 2, 1, 2]`, `[3, 0, 3, 0]`, `[0, 3, 0, 3]`, `[2, 1, 2, 1]` (lines 776-784) |
| "Example 5.4" | $abc^{-1}, bcd^{-1}, cde^{-1}, dea^{-1}, eab^{-1}$ | $\langle a \rangle$ | One row of ten zeros, also after `compress` |
| "Exercises Pg. 161, Q2" | $x^2y^2, y^{-1}xyx^{-3}$ | Trivial | 8 rows after `compress` and `standardize`, printed at lines 1013-1024 and 1065-1080: `[1, 2, 3, 4]`, `[5, 0, 6, 7]`, `[0, 5, 7, 6]`, `[7, 6, 5, 0]`, `[6, 7, 0, 5]`, `[2, 1, 4, 3]`, `[3, 4, 2, 1]`, `[4, 3, 1, 2]` |
| "Table 1. Ex. 1" of [CDHW1973] | $t^{-1}rtr^{-2}, r^{-1}srs^{-2}, s^{-1}tst^{-2}$ | $\langle r \rangle$ | One row of six zeros |
| Cox | $a^6, b^6, (ab)^2, (a^2b^2)^2, (a^3b^3)^5$ | $\langle a \rangle$ | 500 live rows |
| B_2_4 | $a^4, b^4, (ab)^4, (a^{-1}b)^4, (a^2b)^4, (ab^2)^4, (a^2b^2)^4, (a^{-1}bab)^4, (ab^{-1}ab)^4$ | $\langle a \rangle$ | 1024 live rows |

These examples give counts that test the cost of an enumeration:

- Havas: for Sims's presentation $\langle x, y \mid x^n = y, y^n = x \rangle$ of a cyclic group of order $n^2 - 1$ over the trivial subgroup, "some versions of Felsch-type procedures lead to the definition of a total number of cosets which is exponential in n, while some versions of HLT define a total which is O(n^3)" [HAVAS1991, p. 197].
- Havas: for $G(3,7,17)$ over a subgroup of index 1, HLT reached a maximum of 122661 and a total of 196972 cosets, and Felsch 57284 and 57592 [HAVAS1991, p. 195]. The presentation is not printed.
- Cannon and Havas: a group of order 210, "essentially due to Mennicke (1959)", whose subgroup $\langle x \rangle$ has index 105, took a maximum of 1230970 and a total of 1250191 cosets with the methods of [CDHW1973], and 2854 and 2859 with the new default [CANNON-HAVAS1992, p. 4]. The exponents of its presentation are lost in the text layer, so it is not usable as a test.
- ACE: $\langle a, b \mid a^2, b^3, (ab)^5 \rangle$ over $\langle ab \rangle$ gives "INDEX = 12" with a maximum of 14 and a total of 15 cosets [ACE, Appendix B.3] (summarizing tool).

These examples are of the low-index search:

- SymPy: `low_index_subgroups` of $\langle x, y \mid x^2, y^3, (xy)^4 \rangle$ with $N = 4$ returns, in this order, `[[0, 0, 0, 0]]`, `[[0, 0, 1, 2], [1, 1, 2, 0], [3, 3, 0, 1], [2, 2, 3, 3]]`, `[[0, 0, 1, 2], [2, 2, 2, 0], [1, 1, 0, 1]]`, and `[[1, 1, 0, 0], [0, 0, 1, 1]]`, with columns $x, x^{-1}, y, y^{-1}$ [SYMPY-FPG, lines 751-761]. The tables have 1, 4, 3, and 2 rows.
- SymPy: for the same group, `first_in_class` returns true for `[[0, 0, None, None]]` and for `[[1, 1, 1, None], [0, 0, None, 1]]`, and false for `[[1, 1, 2, 1], [0, 0, 0, None], [None, None, None, 0]]` and for `[[1, 1, 1, 2], [0, 0, 2, 0], [2, None, 0, 1]]` [SYMPY-FPG, lines 865-880].
- GAP: for the group `g` that Section 47.5 defines with the relators `f.1^2, f.2^3, (f.1*f.2)^5` and calls an "fp group of size 60", `LowIndexSubgroupsFpGroup( g, TrivialSubgroup( g ), 10 )` returns 4 subgroups. That the example of Section 47.10-1 uses this `g` is the research's reading, because the example does not define it again. The second has the generators `[ a, b*a*b^-1 ]`, and the first subgroup of the iterator has index 1 [GAP, Section 47.10-1].
- GAP: for $\langle x, y, z \mid x^2, y^4, z^3, (xy)^3, (yz)^2, (xz)^3 \rangle$, the search to index 24 with the excluded elements `[ x, y, y^2, z, x*y, x*z, y*z ]` finds 5 classes of index 24, the first "of index 24 and length 8" and the others of length 24 [GAP, Section 47.10-1].
- GAP: for the relators `x^3=y^3=z^5=(xyx^2y^2)^2=(xz)^2=(yz^3)^2=1`, `LowIndexSubgroupsFpGroup(g,6)` gives 11 classes with the indices `[ 1, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6 ]`, and the image of `DefiningQuotientHomomorphism` for the sixth class has size 360 [GAP, Chapter 47, lines 1220-1227 of the local copy].
- GAP: the coset table of the subgroup generated by `g.1` and `g.2*g.1*g.2*g.1*g.2^-1` in the same `g` has the generator lists `[ 1, 4, 5, 2, 3 ]`, `[ 1, 4, 5, 2, 3 ]`, `[ 2, 3, 1, 4, 5 ]`, and `[ 3, 1, 2, 4, 5 ]`. Its rows in the default lenlex standard are `[1,1,2,3]`, `[4,4,3,1]`, `[5,5,1,2]`, `[2,2,4,4]`, `[3,3,5,5]`, and in the semilenlex standard `[1,1,2,4]`, `[3,3,4,1]`, `[2,2,3,3]`, `[5,5,1,2]`, `[4,4,5,5]` [GAP, Sections 47.6-1 and 47.7-2].

These examples are of Reidemeister-Schreier rewriting:

- SymPy: $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$ with $H = \langle xy, x^{-1}y^{-1}xyx \rangle$ gives the generators $y_1, y_2$ and the relators $y_1^2$, $y_2^3$, $y_2 y_1 y_2 y_1 y_2 y_1$. $\langle x, y \mid x^3, y^3, (xy)^3 \rangle$ with $H = \langle xy, xy^{-1} \rangle$ gives $x_0^3$, $y_0^3$, $(x_0 y_0)^3$. $\langle x, y \mid x^2y^2, y^{-1}xyx^{-3} \rangle$ with $H = \langle x \rangle$ gives $x_0^4$. $\langle x, y \mid x^3y^{-3}, (xy)^3, (xy^{-1})^2 \rangle$ with $H = \langle x \rangle$ gives $x_0^6$ [SYMPY-FPG, lines 1322-1344].
- Hulpke: the subgroup of Example III.20 is $\langle c, e \mid c^2 = e^2 = (ec)^5 = 1 \rangle$, "a dihedral group of order 10" [HULPKE2010, Example III.35, p. 84].

### 7.21. Conventions on Which Sources Differ

All the sources read use right cosets and a right action, and none uses left cosets. This table gives the other conventions.

| Source | Numbering | Columns | Undefined entry | Standard form |
| --- | --- | --- | --- | --- |
| [GAP] | From 1, coset 1 is $H$ | $g_1, g_1^{-1}, g_2, g_2^{-1}, \dots$, stored as one list per column | Not stated | Lenlex by default, semilenlex on request |
| [HULPKE2010] | From 1 | $a, a^{-1}, b, b^{-1}$ | Blank in the examples, 0 in Algorithm III.27 [p. 79] | Semilenlex (Definition III.25) |
| [HOLT2013] | From 1, $1 = H$ | Relator tables per relator | Undefined | Not stated |
| [BROWN2011] | From 1 | $a, A, b, B$, uppercase for inverses | 0 | "standard" form not detailed |
| [HAVAS1991] | "1-base array subscripting" | Generators or inverses | 0 | Not stated |
| [SYMPY-CT] | From 0, coset 0 is $H$ | $g_1, g_1^{-1}, g_2, g_2^{-1}, \dots$, one list per row | `None` | Code lenlex, docstring semilenlex |
| [LOWINDEX] | From 1, the base vertex is 1 | Slots $(1,1) < (1,-1) < \dots$ | Empty slot | All slots |
| [SYMMHUB] | From 0 | $g_1, g_1^{-1}, \dots$ | `-1` | Lenlex, by its comment |

The sources also differ on these points:

- HLT: Hulpke fills "gaps of length 1", Holt fills gaps "even for long gaps", and SymPy's scan-and-fill fills any gap (Section 7.11). ACE's HLT turns lookahead on, and ITC's searches for consequences after every definition [ACE] [ITC] (summarizing tool).
- Deduction: SymPy's scan uses the word for the single-gap case only, its deduction stack for any assignment "during a definition or "deduction process"", and ACE for any entry that a relator scan assigns [SYMPY-CT, lines 78-80 and 297-301] [ACE, Section 3.2] (summarizing tool).
- The surviving number in a coincidence: Holt, ACE, and SymPy's code keep the smaller number, and SymPy's docstring recommends the representative of the larger class (Section 7.13).
- Standard form: SymPy's docstring for `standardize` and Hulpke's Definition III.25 ignore the inverse columns, while SymPy's code, GAP's default, and SymmHub's comment use all columns (Section 7.14). SymPy's header comment numbers cosets "1 <= i <= n" while its code numbers them from 0 [SYMPY-CT, line 17].
- The date of Todd and Coxeter: Holt prints "(Todd - Coxeter 1930)" [HOLT2013, p. 15], and Havas cites the paper as 1936 [HAVAS1991, reference 16].
- Initial table of the low-index search: Hulpke starts from an empty table, and SymPy and SymmHub from one coset (Sections 7.17 and 7.19).
- Notation: Holt writes $ix$ for the image of coset $i$ under $x$, Hulpke writes $i^x$, and GAP writes the list entry $l[i]$.

## 8. Per-Pixel Colour in SymmHub

This section describes the colour computation in SymmHub source, commit f2d19cb [SYMMHUB]. File paths are relative to the root of the SymmHub repository. `docs/research/symmhub.md` describes the code in depth, and `docs/research/rendering.md` describes the fold loop and the drawing of borders.

### 8.1. The Per-Pixel Method

Bulatov describes the method as follows: "for each pixel in the image plane (voxel in the object volume) we find its" colour "by mapping the voxel back into a point in the fundamental domain (inverse transformation) and use the" colour "of that point to" colour "the pixel" [BULATOV2014, p. 330]. He credits it to von Gagern and Richter-Gebert [VONGAGERN2009], which was not read. "Each transformation carries with it its stretching factor", whose magnitude measures "the distance from the voxel to the limit set" [BULATOV2014, p. 330].

### 8.2. The Fold Loop and Its State

The fold is `iToFundamentalDomainSampler` in `lib/shaders/inversiveSampler.glsl.mjs`, lines 182-236 [SYMMHUB]. For at most `uIterations` rounds, it finds the first side of $F$ whose signed distance exceeds `EPS` = 1e-7, applies that side's pairing transform, and restarts. When no side is violated, it sets `inDomain = 1`.

The loop carries this state for one pixel [SYMMHUB, `lib/shaders/inversiveSampler.glsl.mjs`, lines 182-236]:

- `scale`: the running scale factor of the transforms applied so far.
- `refcount`: the number of folds, which the colour computation does not use.
- `inDomain`: whether the point reached $F$.
- In the variant `iToFundamentalDomainSamplerPerm24`: a colour permutation `currentPerm`.
- In `subFold`: a coset class `cls`.

The loop carries no word in the generators, no parity of the number of reflections, and no tile identifier [SYMMHUB, `lib/shaders/inversiveSampler.glsl.mjs`, lines 182-236].

The colour-permutation variant `iToFundamentalDomainSamplerPerm24` (`lib/shaders/permutations24.glsl.mjs`, lines 53-130) runs the same loop and updates `currentPerm = compose_perms(...)` on each fold (lines 114-117) [SYMMHUB].

### 8.3. Data Passed to the Shader

- `uPermData[MAX_PERM_COUNT]`: one permutation per side of $F$, with `MAX_PERM_COUNT` = 32. Each permutation is a `uvec4` that holds 24 entries of 5 bits, 6 entries per component [SYMMHUB, `lib/shaders/permutations24.glsl.mjs`, lines 8-48].
- `uPermSize`, `uUseOrbit`, `uAnchorPerm`, `uFillCells`, `uCellColors`, and `uCellColorPermIndex`: the uniforms that `lib/symhublib/VisualizationColorTiles.js` sets, lines 169-179 [SYMMHUB].
- `uAnchorClass`: the coset class of the view anchor (Section 8.5).
- `uIterations`: the bound on the rounds of the fold, default 10, with a range of 0 to 1000 in the user interface [SYMMHUB, `lib/symhublib/SymRenderer.js`, lines 393 and 1385-1393].

The pixel colour is `uCellColors[get_perm_val(currentPerm, uCellColorPermIndex)]`, multiplied by `1. - uTransparency` [SYMMHUB, `lib/shaders/colorTiles.glsl.mjs`, lines 22-63].

On the CPU, a document stores one colour permutation per generator as text. For example, `"ba ba ba "` swaps two colours on every generator of the group `KLM 4 4 2`, which gives a checkerboard parity colouring [SYMMHUB, `apps/sympix/presets/color/par-26-04-29-12-56-34-719.json`]. `strToPermutations` parses the text, with each letter read as `charCode - 'a'` [SYMMHUB, `lib/grouplib/Permutations.js`, line 108].

`sidePermutations` maps the permutations of the generators to the sides of a Dirichlet domain, whose sides are words in the generators, and `wordToPerm` computes the permutation of a word [SYMMHUB, `lib/grouplib/Permutations.js`, lines 52-69 and 87-100]. `lib/symhublib/PermutationPacking.js`, lines 11-33, packs the permutations for the GPU [SYMMHUB].

A catalogue of subgroups with their coset permutations is stored in `lib/color_groups/{klm,sklm,wallpaper}/sub_<name>.json`. For example, `klm/sub_222.json` holds `"generators": "a b", "cosets": "ab ba ab ba"` [SYMMHUB]. `lib/color_groups/generate_manifests.js` and `lib/sublib` generate it, and the presentations come from the fundamental domain by Poincare's polygon theorem (`lib/grouplib/GroupPresentation.js`, line 353) [SYMMHUB].

The palette is a cosine palette $a + b \cos(2\pi(c t + d))$ with $t = i/n$ for up to 24 colours, followed by adjustments in the OKLCH colour space and per-slot edits [SYMMHUB, `lib/symhublib/ColorTiles.js`, lines 72-207].

### 8.4. Composition Order

`compose_perms(outer_perm, inner_perm, permSize)` returns the permutation that maps each index $i$ to `outer_perm(inner_perm(i))` [SYMMHUB, `lib/shaders/permutations24.glsl.mjs`, lines 34-48]. Its loop reads `inner_val = get_perm_val(inner_perm, i)`, then `outer_val = get_perm_val(outer_perm, inner_val)`, and stores `outer_val` at index $i$.

After the fold applies the reflections of side `g` to the point, it updates the state with this code [SYMMHUB, `lib/shaders/permutations24.glsl.mjs`, lines 107-117]:

```glsl
if(useOrbit) currentPerm = compose_perms(permData[g], currentPerm, permSize);
else currentPerm = compose_perms(currentPerm, permData[g], permSize);
```

The flag `useOrbit` is a parameter of the function (`lib/shaders/permutations24.glsl.mjs`, line 59). It receives the uniform `uUseOrbit` at the call in `lib/shaders/colorTiles.glsl.mjs`, line 51, and the uniform is declared at line 20 of that file [SYMMHUB]. With $\sigma$ the state `currentPerm` and $\rho_g$ the permutation `permData[g]` of side `g`, the two updates are:

| `useOrbit` | Colouring | Update | Image of index $i$ |
| --- | --- | --- | --- |
| false | By coset | $\sigma \leftarrow \sigma \circ \rho_g$ | $\sigma(\rho_g(i))$ |
| true | By orbit | $\sigma \leftarrow \rho_g \circ \sigma$ | $\rho_g(\sigma(i))$ |

The comment above this code gives its reason. It is the code's own claim, and no source read proves it [SYMMHUB, `lib/shaders/permutations24.glsl.mjs`, lines 107-113]:

```text
accumulate the permutation
the permutation composition is done in reverse order.
We work from point on screen toward the fundamental domain composing the sequence of transformations in the process.
To get the final color permutation we need to find the permutation which corresponds to the sequence of _inverse_ transformations applied in _opposite_ order.
The reverse composition of permutations does exactly that
```

### 8.5. The Subgroup Automaton

`subFold` in `lib/shaders/overlay_subgroup.glsl.mjs`, lines 60-100, runs the same loop and updates the coset class with `cls = subNextClass(cls, g)` from a table indexed by class and side, with one texel per entry [SYMMHUB]. The table acts as an automaton on cosets. The header comment at lines 1-36 describes the data.

`applyAnchor` (`lib/shaders/inversiveSampler.glsl.mjs`, lines 127-176) applies one group element that the CPU computes in double precision, and starts the colour permutation from `uAnchorPerm` and the coset class from `uAnchorClass` [SYMMHUB].

### 8.6. Limits

- At most 24 colours, from the 5-bit packing and `MAX_COLORS_COUNT` = 24 [SYMMHUB, `lib/shaders/permutations24.glsl.mjs`, line 10].
- At most 32 side permutations, from `MAX_PERM_COUNT` = 32 [SYMMHUB, same file, line 8].
- A pixel whose fold does not reach $F$ within `uIterations` rounds is written as transparent black [SYMMHUB, `lib/shaders/colorTiles.glsl.mjs`, lines 53-56]. This is how the region near the limit circle goes blank.

### 8.7. Assembled Algorithm

These steps are assembled from the SymmHub source (Sections 8.2 to 8.5) and the low-index sources (Sections 7.2 and 7.3), and add no new derivation. That they produce a perfect colouring for every orbifold group is not established by any source read (Section 14.1).

1. Precompute, by a low-index subgroup enumeration, a homomorphism $\rho : G \to S_k$, with one permutation $\rho(s_i)$ for each side pairing $s_i$ of $F$.
2. Map the pixel to the model and apply the view anchor. Set the state $\sigma$ to the anchor's permutation, or to the identity.
3. Repeat at most $N$ times: find the first side $i$ whose half-plane test the point violates, apply the pairing transform of side $i$, and update the state as the table of Section 8.4 gives: $\sigma \leftarrow \sigma \circ \rho(s_i)$ when colouring by coset, or $\sigma \leftarrow \rho(s_i) \circ \sigma$ when colouring by orbit. Stop when no side is violated.
4. If the loop did not stop, draw the pixel transparent.
5. Colour the pixel with the palette entry at index $\sigma(j)$ for a fixed slot $j$. Alternatively, carry a coset index $c$ instead of $\sigma$, update it as $c \leftarrow T[c][i]$ from the coset table $T$ at each fold, and colour by the palette entry at index $c$.

## 9. Other Tilings Associated with a Group

### 9.1. Isohedral Tiling Types

Grunbaum and Shephard "used their notation to derive a complete set of 93 isohedral tiling types" [KAPLAN2025]. Of these, "12 cannot be expressed using the boundaries of the tiles alone", because "all tile edges act as mirrors, forcing those edges to be straight lines", and the remaining 81 are "the isohedral tiling types" [KAPLAN2025]. A type is "summarized using an _incidence symbol_ made out of two words", and the types IH77, IH80, and IH87 "share the symbol" [KAPLAN2025].

"Every isohedral tiling is necessarily periodic: its symmetry group is one of the 17 wallpaper groups", and each has "one of only 93 distinct symbols" [KAPLAN2019, p. 100]. These types are Euclidean only.

The library Tactile has a `tilingTypes` array of length 81, which omits IH00 (undefined) and IH19, IH35, IH48, IH60, IH63, IH65, IH70, IH75, IH80, IH87, IH89, and IH92 [TACTILE]. "Some tiling types have zero parameters", while "others have as many as six" [TACTILE].

Its colouring function `getColour(t1, t2, aspect)` returns "0, 1, or 2" so that "adjacent tiles aren't the same colour" [TACTILE, `lib/tactile.js`]. The code sets `col = clrg[asp]`, steps `mt1 = t1 mod nc` through `clrg[12+col]` and `mt2` through `clrg[15+col]`, with `nc = clrg[18]`. The result is a translation-invariant colouring with two or three colours that is not documented as compatible with the symmetry, and the meanings of the indices are inferred from the code. Kaplan's 2019 paper says nothing about colouring rules [KAPLAN2019].

### 9.2. Delaney-Dress Symbols

A Delaney-Dress symbol is "a connected (multi-) graph" in which each node is incident to exactly one edge of each of the colours 0, 1, and 2, "together with a labelling of its nodes by two maps m01 and m12", which record the tile degree and the vertex degree [TEGULA2020, Section 4]. Tegula defers the full conditions to Dress (1984) and Dress and Huson (1987) [DRESS].

Balke and Huson state the conditions for the two-dimensional case [BALKE1996, Section 3, p. 95]. Let $\Sigma = \langle \sigma_0, \sigma_1, \sigma_2 \mid \sigma_i^2 = 1 \rangle$, "the Coxeter group freely generated by three involutions". A finite transitive $\Sigma$-set $\mathcal{D}$ with three maps $m_{01}, m_{12}, m_{02} : \mathcal{D} \to \mathbb{N}$ is a Delaney-Dress symbol if and only if, for all $D \in \mathcal{D}$ and all $0 \le i < j \le 2$:

- (DS1) $m_{ij}(D\sigma_i) = m_{ij}(D\sigma_j) = m_{ij}(D)$.
- (DS2) $D(\sigma_i\sigma_j)^{m_{ij}(D)} = D$.
- (DS3) $m_{01}(D) \ge 2$, $m_{12}(D) \ge 3$, and $m_{02}(D) = 2$.

Condition (DS3) "avoids degeneracies like mono-gons, vertices of degree less than 3, or edges which fold back onto themselves" [BALKE1996, p. 95].

The curvature of a symbol $\mathcal{D}$ with labelling $m$ is [TEGULA2020, Section 4, unnumbered display]:

$$\mathcal{K}(\mathcal{D}, m) = \sum_{D \in \mathcal{D}} \left( \frac{1}{m_{01}(D)} + \frac{1}{m_{12}(D)} - \frac{1}{2} \right)$$

Balke and Huson define the curvature with a third term [BALKE1996, p. 96]:

$$\mathcal{K}_{(\mathcal{D}, m)} = \sum_{D \in \mathcal{D}} \left( \frac{1}{m_{01}(D)} + \frac{1}{m_{12}(D)} + \frac{1}{m_{02}(D)} - 1 \right)$$

With $m_{02}(D) = 2$ from (DS3), each summand is the summand of [TEGULA2020]. Theorem 4.1 states that the curvature is twice the orbifold Euler characteristic of the associated orbifold, $\mathcal{K}_{(\mathcal{D}, m)} = 2\chi_{\mathrm{orb}}$ [BALKE1996, Theorem 4.1, p. 97]. Balke and Huson write $\chi_{\mathrm{orb}}$ as the Euler characteristic of the underlying surface minus $(v - 1)/v$ for each cone point of order $v$ and minus $(v - 1)/(2v)$ for each corner point of order $v$, citing Conway [BALKE1996, p. 97]. The caption of their Table I states the relation the other way round, "the Euler characteristic of an orbifold is precisely twice the curvature", while Theorem 4.1 and the introduction state that the curvature is twice the Euler characteristic [BALKE1996, pp. 90, 91, and 97] (Section 13).

The geometry is "spherical, euclidean or hyperbolic, depending on whether the curvature is positive, 0 or negative" [TEGULA2020, Section 4]. In the passage to the orbifold, "reflections are mapped onto boundary segments, rotational centers are mapped onto cones, dihedral centers are mapped onto corners, and glide-reflections give rise to crosscaps" [TEGULA2020, Section 4]. Balke and Huson give the derivation: the topological realization of a symbol and its signature [BALKE1996, p. 96], formulas for the number of handles and cross-caps [BALKE1996, Section 5, p. 98], and Algorithm 5.1, which prints the boundary components of the orbifold with their corner orders, with its proof [BALKE1996, Section 5, pp. 99-100]. Cone points come from the non-trivial branching numbers of cycles, and corner points from chains [BALKE1996, p. 99].

EPINET describes the system of chambers of a Delaney-Dress symbol as "really just a triangulation of this orbifold", with the example `*246`, a triangle with angles $\pi/2$, $\pi/4$, and $\pi/6$ [EPINET-DD]. Its page on orbifold notation gives a quantity that it calls curvature, in terms of $o$ handles, $\ast$ mirror boundaries, $x$ cross-caps, cone orders $c_i$, and corner orders $m_j$, "derived from Macbeath's non-euclidean crystallographic group signature" [EPINET-ORB]:

$$K = 2 - 2o - \ast - x - \sum_i \frac{c_i - 1}{c_i} - \sum_j \frac{m_j - 1}{2 m_j}$$

This $K$ is stated for a signature, not for a symbol. Its relation to the curvature $\mathcal{K}$ of [TEGULA2020] and [BALKE1996] is open (Section 14.1, item 18).

`docs/research/orbifold-notation.md` covers signatures and their curvature in depth.

Tegula enumerates tilings by their Dress complexity $\delta$, the number of nodes of the symbol, up to $\delta = 24$, with "exactly 2,395,220,319" geometry-minimal types in total [TEGULA2020, Section 5, Table 1]:

| Count | Spherical | Euclidean | Hyperbolic |
| --- | --- | --- | --- |
| $\delta = 1$ | 5 | 3 | 4 |
| $\delta = 2$ | 13 | 15 | 22 |
| Total up to $\delta = 24$ | 2,155,818 | 1,728,488 | 2,391,336,013 |

Kolbe and Evans extend Delaney-Dress symbols to isotopy classes [KOLBE-EVANS] (search summary only).

## 10. Design (c): The Wythoff Construction

Every source in this section except Sections 10.6 and 10.7 is secondary. The primary sources were not read (Section 14.3).

### 10.1. Generator Position and Wythoff Symbol

In a Wythoff symbol, "The p, q, r values are listed before the bar if the corresponding mirror is active" [WP-WYTHOFF-SYMBOL, Description]. The position of the bar places the generator point [WP-WYTHOFF-SYMBOL, Description]:

- `p | q r`: "the generator lies on the corner p".
- `p q | r`: the generator lies "on the edge between p and q".
- `p q r |`: the generator lies "in the interior".
- `| p q r`: the snub, where "odd-numbered reflected images are ignored".

"This point must be chosen at equal distance from all edges that it does not lie on" [WP-WYTHOFF-SYMBOL, Description].

### 10.2. Ring Diagrams

In a Coxeter-Dynkin diagram, "A node is circled if the generator point is not on the mirror", and "A mirror is active (creates reflections) only with respect to points not on it" [WP-COXETER-DYNKIN, "Application to uniform polytopes"]. "There are 7 reflective uniform constructions within a general triangle", and "Uniform polytopes with one ring correspond to generator points at the corners" [WP-COXETER-DYNKIN]. "A truncated alternation is called a snub" [WP-COXETER-DYNKIN].

### 10.3. Families and Vertex Configurations

"Each symmetry family contains 7 uniform tilings, defined by a Wythoff symbol or Coxeter-Dynkin diagram", and "An 8th represents an alternation operation, deleting alternate vertices from the highest form with all mirrors active" [WP-UNIFORM-HYP].

This table gives the vertex configurations of the `(p q 2)` family. It comes from a summarized table on the page "Uniform tiling" and is not verbatim [WP-UNIFORM-TILING].

| Form | Vertex configuration |
| --- | --- |
| Regular | `p^q` |
| Truncated | `q.2p.2p` |
| Rectified | `(p.q)^2` |
| Bitruncated | `p.2q.2q` |
| Birectified | `q^p` |
| Cantellated | `p.4.q.4` |
| Omnitruncated | `4.2p.2q` |
| Snub, `sr{p,q}` | `3.3.p.3.q` |

The row for the general `(p q r)` family was inconsistent in the extraction and is omitted. No source read states which faces of each form belong to which face orbit (Section 14.1).

### 10.4. Snubs

"Snubbing is an operation of alternate truncation of the omnitruncated form" [WP-UNIFORM-TILING]. Snubs "have chiral symmetry with two forms" [WP-SNUB], and "The resulting figure has rotational symmetry only" [WP-WYTHOFF-SYMBOL].

### 10.5. Quadrilateral Domains

"The only possible fundamental domain in Euclidean 2-space that is not a simplex is the rectangle (infinity 2 infinity 2)", and "All forms generated from it become a square tiling" [WP-UNIFORM-TILING]. "Further symmetry groups exist in the hyperbolic plane with quadrilateral fundamental domains starting with (2 2 2 3)", and the page gives no construction rule [WP-UNIFORM-TILING]. The source writes the infinity sign as a symbol, replaced here by the word.

### 10.6. Per-Pixel Rendering

Zhao Liang's renderer reflects the point "repeatedly ... off mirrors until it falls within the fundamental region", and colours the pixel "based on q's position within the fundamental domain", which "allows for parallel computation on all pixels" [ZHAO]. For drawing faces from the group instead of per pixel, "The stabilizing subgroup of f_0 is the standard parabolic subgroup <i,j>", and the edges of type $i$ are obtained "by applying the coset representatives in G/H" [ZHAO].

The page gives no explicit classification of a pixel into a face, no count of face types, and no rule for snubs [ZHAO]. Its shader is "exported from Matt Zucker's excellent work on shadertoy" [ZHAO] [ZUCKER], which could not be read.

SymmHub has no input for a Wythoff symbol or a Coxeter diagram [SYMMHUB]. No such input was found in a reading of its group-construction code.

### 10.7. Status of Each Part

| Part | Status |
| --- | --- |
| Generator positions and Wythoff symbols (Section 10.1) | Secondary source only |
| Ring diagrams (Section 10.2) | Secondary source only |
| Seven forms and the snub per family (Section 10.3) | Secondary source only |
| Vertex configurations of `(p q 2)` (Section 10.3) | Secondary source, summarized, not verbatim |
| Vertex configurations of `(p q r)` | Not available |
| Snubs (Section 10.4) | Secondary source only |
| Quadrilateral domains (Section 10.5) | Secondary source only, no construction rule |
| Face-orbit rule per ring pattern | No source (Section 14.1) |
| Per-pixel face classification | No source (Section 14.1) |
| Per-pixel fold into the triangle (Section 10.6) | Primary web source [ZHAO] |
| Perfect colourings of Wythoff faces | No source beyond Lemma 3.2 for the regular and Laves tilings (Section 14.1) |

## 11. Existing Tools

This table summarizes what existing tools draw and how they colour it.

| Tool | What it draws | How it colours | Source |
| --- | --- | --- | --- |
| SymmHub | The fundamental-domain tiling, images in $F$, or simulations, for any orbifold. The general path is hyperbolic only, and spherical and Euclidean groups come from a fixed catalogue. | Perfect colourings from one permutation per generator, or from a catalogue of subgroups with coset permutations, up to 24 colours. Cosine palette with OKLCH adjustments and per-slot edits. | [SYMMHUB], `lib/orbilib/OrbifoldGeometrization.js`, `lib/orbilib/WallpaperGroups_NonNegative.js`, and Section 8 |
| KaleidoTile | Polyhedra and tilings of the sphere and the Euclidean and hyperbolic planes from a triangle group. "Within each symmetry group, any polyhedron or tiling morphs to any other." | Decorated "with images ..." or colours. The scheme for face colours is not documented, and the source was not read. | [KALEIDOTILE] |
| HyperRogue | Tilings of the hyperbolic plane in a game. | Fixed named schemes, not a general choice of subgroup. The pattern menu in `pattern2.cpp` offers "cell types", "Zebra Pattern", "Emerald Pattern", "Palace Pattern", "chessboard", "coloring", "siblings", "field pattern", and "single type", with sub-options such as "rotational symmetry", "full symmetry", "alternate coloring", and "football". The canvas colourings in `ccolor::all` include plain, random, sides, formula, threecolor, football, chessboard, seven, zebra, emerald, and palace. | [HYPERROGUE] |
| Kali | A line drawing by the user, replicated in the 17 wallpaper groups or the 7 frieze groups, labelled in Conway and crystallographic notation. | A colour chosen with colour buttons (paraphrase). No colour symmetry. | [KALI] |
| Malin Christersson's image tiling | An image cropped to a `{p,q}` polygon and reflected: "Repeated hyperbolic reflections of the centered polygon make up the tiling." | Only a background colour. | [MALINC] |
| Morenaments | Drawings in the 17 wallpaper groups. | "Colour and line thickness are set with a colour dial" (search summary). | [MORENAMENTS] |
| iOrnament | Strokes repeated in the 17 wallpaper groups. | Pens named "monochromatic", `rainbow colors`, and "shiny gold" (search summary). | [IORNAMENT] |
| Escher Web Sketch | A drawing tool from EPFL. | No documentation of colour symmetry was found. | [EWS] |
| Tactile | Isohedral tilings of the Euclidean plane (Section 9.1). | A translation-invariant colouring with two or three colours that avoids equal colours on adjacent tiles. | [TACTILE] |
| Tess | Not known. | Not known. The site refused the connection. | [TESS] |

Apart from SymmHub's subgroup catalogue and HyperRogue's fixed named patterns, the research for this document found no tool in this table that offers a number of colours derived from the symmetry group.

## 12. Comparison of Designs (a), (b), and (c)

This table compares the three designs, limited to what the sources establish. It makes no recommendation.

| Aspect | Design (a): fundamental-domain tiling | Design (b): fundamental domain cut into polygons | Design (c): Wythoff tiling |
| --- | --- | --- | --- |
| What determines the available numbers of colours | Every index of a subgroup of finite index of $G$ gives a coset colouring with that many colours, within the scope of Section 4.2 (Section 4.3). Whether other perfect colourings exist is open. | No source read treats this design. | For the regular and Laves tilings, the indices of the subgroups that contain the tile-stabilizer generators of Lemma 3.2 or Lemma 4.1 (Section 5). For other Wythoff tilings, no source. |
| Published values | Bulatov's example captions (Section 4.4). No closed rule. | None. | Frettloh's Tables 1 and 2 for nine hyperbolic tilings, the cube, and the icosahedron (Sections 5.6 to 5.8). No closed rule. |
| Proved scope | Lemma 2.1 for $T_{p,q}$ under $G_{p,q}$, said to hold more generally without detail (Section 4.2). | None. | Lemmas 3.1, 3.2, and 4.1 for `(p^q)`, `(q^p)`, and `[p.q.p.q]` under $G_{p,q}$ and $\bar{G}_{p,q}$ only (Section 5). |
| Equal colour frequencies | Not stated for this tiling. | Not stated. | Stated for regular and Laves tilings (Lemma 3.1), and stated not to hold in general for other tilings. |
| Machinery | A presentation of $G$, a low-index subgroup enumeration, and the fold loop carrying a permutation or a coset class (Sections 7 and 8). SymmHub implements this design. | Not established by any source. | A presentation of $G_{p,q}$, a low-index subgroup enumeration filtered by stabilizer generators (Section 5.5), and a classification of each pixel into a face, for which no source gives a method (Section 10.6). |
| Software precedent | SymmHub (Section 11). | None found. | KaleidoTile draws Wythoff tilings, with an undocumented colour scheme. Zhao Liang's renderer folds into the triangle. |
| Gaps left open | Correspondence between colourings and subgroups for arbitrary orbifold groups, and which indices occur (Section 14.1, items 1, 2, and 9). | Every question about colour counts and perfection (Section 14.1, item 14). | The face-orbit rule, the per-pixel face classification, perfect colourings beyond regular and Laves tilings, groups that are not triangle groups, snubs, and Euclidean counts (Section 14.1, items 5, 10, 11, 12, and 15). |

## 13. Conflicts Between Sources

- Frettloh's own statements on what is counted: Section 3 says that different colourings may share a colour symmetry group, and the conclusion speaks of a bijection between perfect colourings and subgroups [FRETTLOH2008, pp. 3 and 5]. This document reports both (Section 3.4).
- The year of Frettloh's paper: Crossref gives volume 223 with an issue dated December 2008 and online publication on 2009-09-25, and the MIT tilingsearch database cites 2009. This document uses the volume year, 2008.
- The date of Felix and Loquias's paper: the arXiv posting is dated 2010-02-02, and the paper appeared in 2008 in Z. Kristallogr. 223. This document uses 2008.
- The number of isohedral types: 93 types, of which 81 are the isohedral tiling types [KAPLAN2025]. The Tactile README calls the other 12 "boring" without a reason [TACTILE], and Kaplan's 2025 page gives the reason, mirror edges.
- Coverage of the Wythoff construction: the Wikipedia page "Wythoff construction" covers only triangles, and the page "Uniform tiling" mentions quadrilateral domains without a rule.
- The factor between curvature and Euler characteristic in [BALKE1996]: the caption of Table I (p. 91) says that the Euler characteristic is twice the curvature, while Theorem 4.1 (p. 97) and the introduction (p. 90) say that the curvature is twice the Euler characteristic. This document reports both (Section 9.2).
- The locator of the spherical counts: a condensed extraction of Frettloh's paper placed the cube and the icosahedron in Sections 1 and 2, and the full PDF places the cube on p. 4. The icosahedron was not re-checked in the PDF.

## 14. Open Questions and Unverified Claims

### 14.1. Open Questions

Each item below is a candidate statement that needs a source or a machine-checked proof before the body can state it.

1. Colourings and permutation representations: whether the perfect colourings of a fundamental-domain tiling with $k$ colours correspond to transitive permutation representations of $G$ of degree $k$, and whether they correspond up to symmetry to conjugacy classes of subgroups of index $k$. Frettloh's paper states neither. [CBGS2008] defines a colour type by a homomorphism to a group of permutations of the colours and requires the symmetries to be transitive on the colours (Section 3.3). It counts types up to isotopic reshaping and states no correspondence with conjugacy classes of subgroups.
2. The converse of Lemma 2.1: whether every perfect colouring of the fundamental-domain tiling of an arbitrary orbifold group arises from a subgroup, and whether Lemma 2.1 holds for groups with rotations, glide reflections, handles, or cross-caps. [CBGS2008] classifies the twofold and prime-fold colour types of all 17 plane groups and of the spherical groups by homomorphisms (Sections 6 and 7.6), and gives annotated symbols for hyperbolic groups (Section 3.3), but states no lemma on the coset colourings of a fundamental-domain tiling.
3. The tension in Frettloh's paper between the statement that colourings may share a colour symmetry group and the stated bijection (Section 3.4). Schwarzenberger's Section 8, which Frettloh cites on this point, was not read.
4. The tile centres: a reading of eq. (1) suggests that the `(q^p)` tile is centred at the $\pi/q$ corner of $F$, the `(p^q)` tile at the $\pi/p$ corner, and the `[p.q.p.q]` tile at the right-angle corner. The paper does not state this except for the `(q^p)` tile $F \cup aF \cup abF \cup \dots$.
5. The tile-stabilizer condition beyond regular and Laves tilings: whether a subgroup induces a perfect colouring of a Wythoff tiling exactly when it contains the stabilizer of each face, such as the standard parabolic subgroup $\langle i, j \rangle$ of Zhao Liang's page, and whether it extends to groups other than $G_{p,q}$.
6. The number of two-colourings from a signature: a formula for the dimension over $\mathbb{F}_2$ of $G/G^2[G,G]$ in terms of handles, cross-caps, mirrors, corner orders, and cone orders, and whether $[G : G^2] - 1$ holds for infinite $G$. The table of twofold types of the 17 plane groups is Table 11.1 of [CBGS2008] (Section 6), and no source read states a formula. The readers of this research found that the sum of the column "Number" of Table 11.1 equals $\lvert \mathrm{Hom}(G, \mathbb{Z}/2) \rvert - 1$, computed from the presentations, for the groups that they checked, for example 15 for `*2222`, 7 for `2222` and `**`, and 3 for `o`. That check is their own and is not a statement of the book. The total of 46 types is also their count of the rows.
7. Hall's formula at $k = 2$: substituting into Section 7.5 gives $a_2 = |\mathrm{Hom}(G, S_2)| - 1$. A searcher made this substitution, and no source states it.
8. The theorem of Edmonds, Ewing, and Kulkarni: the exact statement of the achievable indices of torsion-free subgroups, and Izquierdo's version for NEC groups.
9. Achievable indices in general: which composite indices occur for the wallpaper groups, and which indices occur for hyperbolic groups. For prime indices of the plane and spherical groups, Section 7.6 gives the tables of [CBGS2008]. The other sourced data are Frettloh's tables and Bulatov's examples.
10. The face-orbit rule for Wythoff tilings: a rule such as "a face of size p or 2p around corner p exists if and only if a mirror there is active", and the face list of snubs, with rotation-centre faces and snub triangles and a generator equidistant under the rotations. No source read states either.
11. Per-pixel classification for Wythoff tilings: no method found. Zucker's shader [ZUCKER] and the KaleidoTile source [KALEIDOTILE] are the leads.
12. The Wythoff construction beyond triangles: polygonal kaleidoscopes such as `*2222` and `*abcd`, and mixed orbifolds such as `3*3` and `2*22`.
13. Normal subgroups: whether a subgroup is normal exactly when $G/H$ acts regularly on the colours, and how the kernel $K$ relates to the core of the colour group.
14. Design (b): no source read addresses the colouring of a fundamental domain cut into several polygons, or the numbers of colours it offers.
15. Euclidean perfect colourings: the counts for Euclidean regular and Laves tilings, deferred by Frettloh to [SCHWARZENBERGER1984]. [CBGS2008] tabulates the twofold and prime-fold colour types of the 17 plane groups (Sections 6 and 7.6), but not counts for regular and Laves tilings in Frettloh's sense.
16. The index-$n$ colouring theorem of De Las Penas, Decena, and Laigo [DDL2009]: a correspondence between subgroups of index $n$ and transitive colourings with $n$ colours is known from a search summary only, and the theorem number was not retrieved.
17. Tool details: the face colouring of KaleidoTile, the behaviour of Tess, and the colour features of iOrnament, Morenaments, and Escher Web Sketch.
18. The curvature of EPINET: whether the quantity that [EPINET-ORB] calls curvature equals the orbifold Euler characteristic of the signature, so that it is half the curvature $\mathcal{K}$ of a symbol in [BALKE1996] (Section 9.2). A comparison of its terms with the defect rule of Section 6.1 of `docs/research/orbifold-notation.md` suggests so. That comparison is not a statement of any source.
19. A bounded enumeration that reaches its limit: no source read states what a run that reaches a limit on the number of cosets shows about the index, and in particular none states that it shows the index to be infinite (Section 7.15). [CANNON-HAVAS1992] and [HAVAS1991] rule out a computable bound but do not discuss a run that stops. Would close it: Chapter 5 of [SIMS1994] or of [HOLT2005], or [NEUBUSER1982].
20. Termination of the HLT strategy: whether SymPy's HLT loop, which defines every open entry of a row after its scans, meets Mendelsohn's condition, and whether any HLT variant does. Holt says that the relator-based strategy does "not necessarily" meet (R4), and ITC says that pure HLT does not guarantee the condition (Section 7.15). Would close it: Lemma 1 and Theorem 2 of [NEUBUSER1982], or Chapter 5 of [HOLT2005].
21. Mendelsohn's result in its original form: the statements read are Hulpke's Theorem III.24, which names Mendelsohn and gives a proof idea, Holt's Theorem 5.3, which gives a proof and names no author, and ACE's attribution, read through a summary, of the condition to Lemma 1 and Theorem 2 of [NEUBUSER1982] (Section 7.15). Would close it: [MENDELSOHN1964] or [NEUBUSER1982].
22. Safety of the first-in-class pruning on partial tables: SymPy's docstring, Hulpke's prose, the `low_index` note, and SymmHub's comment assert that the test never discards an ancestor of the first table of a class, and no source read proves it. The `low_index` rule compares empty slots as 0, while SymPy and SymmHub stop at an undefined entry, and no source says whether the two rules prune the same branches (Sections 7.17 and 7.19). Would close it: p. 208 and Chapter 5 of [SIMS1994], Section 5.4 of [HOLT2005], or Lean.
23. Steps that only code states: the three outcomes of a scan, scan-and-fill, the deduction stack, the coincidence queue, compression, and standardization are stated at step level only by SymPy, which cites pages of [HOLT2005] that were not read (Sections 7.10 to 7.14). Neither is it stated whether the deductions that SymPy's `scan_check` writes without pushing them on the stack can leave a short relator unchecked in the low-index search (Section 7.17), or whether it matters that SymPy's Felsch loop scans every popped deduction with the lists of its caller (Section 7.12). Would close it: Chapter 5 of [HOLT2005], pp. 149-170, and Section 5.4.
24. SymmHub's standard form and its agreement with GAP: whether the tables of SymmHub's search are standardized in the lenlex sense of GAP, as the comment at `sublib_core.js`, lines 21-27, claims, and whether its agreement with GAP's representatives is about 99%, as the comment at lines 31-33 states, or 97%, as `lib/sublib/README.md`, line 310, states (Section 7.19). Both figures are claims of the project, and the research ran no comparison. The research's own argument that the search builds standardized tables is not a statement of the code. Would close it: the comparison with the GAP files that the README describes (section `Compatibility with the GAP-generated files`), run by the research.
25. Group orders and indices of the worked examples that no source states: the order of $\langle x, y \mid x^2, y^3, (xy)^4 \rangle$ of the SymPy low-index doctest, the groups of SymPy's Examples 5.1 to 5.3 and of ACE's Appendix B.3, the index 5 of Holt's last example, which follows from $60/12$, and that GAP's example of Section 47.10-1 uses the group of Section 47.5 (Section 7.20). Would close it: [HOLT2005] for the SymPy examples, or a computation by an independent implementation recorded as such.
26. Lemma III.26 of [HULPKE2010] for the lenlex standard: Hulpke states the bijection between subgroups of index $n$ and standardized tables for his definition, which ignores the inverse columns. No source read states it for GAP's lenlex standard, which the search of Section 7.17 builds (Section 7.14). Would close it: [SIMS1994], whose standard GAP follows.
27. The count of `low_index` for the free group of rank 3: a summarizing tool reported 653 results for the README example at index 4, and the number does not appear in the text that the tool quoted (Section 7.3). Would close it: the README read directly, or a run of the package.

### 14.2. Claims Resting on Secondary Sources or Inference

- Section 3.3: the Waerden-Burckhardt framework [WP-POLYCHROMATIC].
- Section 6: the possible counts of index-2 subgroups [WP-INDEX], and the 46 two-colour types [ARTLANDIA]. The number 46 is also the count of the rows of Table 11.1 of [CBGS2008] by the readers of this research, and the book states no total (Section 14.1, item 6).
- Sections 6 and 8.3: the reading of a parity colouring, such as the checkerboard preset of SymmHub, as the colouring by the two cosets of $\Gamma^+$ is an inference. [SINGERMAN1974] states the subgroup $\Gamma^+$ and its signature but says nothing about colourings, and [CBGS2008] does not use the term parity colouring.
- Section 7.1: the statement on Magma, from a search summary.
- Section 7.3: the meaning of each result of `permutation_reps`, inferred from examples, and the README of `low_index`, read through a summarizing tool.
- Sections 7.13, 7.15 to 7.17, 7.20, and 7.21: the statements of [ACE], [ITC], [MAGMA], and [SYMPY], read through a summarizing tool.
- Sections 7.10 to 7.14 and 7.17: the steps that only SymPy's code states, which are the code's own and cite unread pages of [HOLT2005] (Section 14.1, item 23).
- Section 7.9: the correspondence of SymPy's list of properties with Holt's (R1) to (R3), which is the research's reading.
- Section 7.16: that SymPy's limit counts dead rows, which the research read off the code.
- Section 7.19: the claims of SymmHub's comments and README that its conventions are those of GAP, that its canonicity test is safe on partial tables, and that its representatives agree with GAP's for about 99% or 97% of subgroups.
- Section 7.20: the reading of the entry in row 5 of Hulpke's Example III.23 as a misprint and the reading that GAP's example of Section 47.10-1 uses the group of Section 47.5.
- Section 7.6: the scope of the theorem of Edmonds, Ewing, and Kulkarni, through [BCCC2025] and a search summary.
- Section 8.4: the reason that SymmHub's code comment gives for the reverse composition, which is the code's own claim.
- Section 9.1: the meanings of the indices in Tactile's `getColour`, inferred from code.
- Section 9.2: the work of Kolbe and Evans, from a search summary.
- Section 10, except Sections 10.6 and 10.7: Wikipedia only.
- Section 11: the rows for Morenaments and iOrnament, from search summaries, and the paraphrase for Kali.
- The source [ARXIV1806.06911]: its author and title were not recorded.
- The quotations from web sources throughout passed through a fetch tool that condensed the text and capped quotations at about 125 characters (Section 7.8), except those from [FRETTLOH2008], [KAPLAN2019], and [BULATOV2014], which come from the full PDFs.

### 14.3. Sources Not Read in Full

These sources were not read in full and are recorded in `docs/research/source-requests.md`, with what the project needs from each:

- [CBGS2008]: read in part, for the chapters on colourings and the passages cited here. Chapters 19 to 26 were not read. A passage on Wythoff triangles in chapter 21, pp. 289-290, may bear on Section 10 and items 4, 5, 10, 11, and 12.
- [GS1987]: chapter 8 on coloured patterns and chapter 6 on isohedral types.
- [EEK1982], [BURNS1983], and [IZQUIERDO1993]: the theorems on achievable indices. Needed for item 8.
- [SCHWARZENBERGER1984]: Euclidean perfect colourings and its Section 8 on colourings that share a colour group. Needed for items 3 and 15.
- [DLP2006] and [DPFP2007]: the hyperbolic coset-colouring framework and index-2 subgroups of hyperbolic symmetry groups.
- [LPD2008] and [DDL2009]: the colouring method for index 2 and the index-$n$ colouring theorem. Needed for item 16.
- [HOLT2005]: Chapter 5, pp. 149-170, with the pages that SymPy cites (pp. 152, 153, 155, 156-161, 157, 162-163, 164, 165, 166, 167, and 167-168 for coset enumeration, pp. 85-87 for the classes of coincident cosets, p. 175 for the Schreier generators, and pp. 177 to 187 for its examples) and Section 5.4 on low-index subgroups. Needed for items 19, 20, 22, 23, and 25. This document rates the request high, because the implementation of coset enumeration and of the low-index search relies on the steps of items 22 and 23.
- [SIMS1994]: Chapter 5, with the low-index method and the function on p. 208 that SymPy's `first_in_class` follows, and the definition of a standard table that GAP follows. Needed for items 19, 22, and 26. This document rates the request high, for the same reason as [HOLT2005].
- [NEUBUSER1982]: Lemma 1 and Theorem 2, which state Mendelsohn's condition, and the method of low-index subgroups by forced coincidences. Needed for items 19 to 21. Medium priority.
- [MENDELSOHN1964]: the original termination result. Needed for item 21. Medium priority.
- [CDHW1973]: the implementations of HLT, Felsch, and lookahead that [HAVAS1991] calls "a primary reference", and the presentations of its Table 1, which the examples of [HAVAS1991] and SymPy use. Needed for items 20 and 25. Medium priority.
- [CONDER2005]: applications and adaptations of the low-index procedure. Medium priority.
- [TODD-COXETER1936]: the original method. Low priority, historical.
- [HAVAS-RAMSAY2000]: a case study in coset enumeration, cited by [CHR2007]. Low priority.
- [CLM1954] and [HAREL1993]: the primary rules of the Wythoff construction. Needed for item 10.
- [ZUCKER], [KALEIDOTILE], and [NELSON2019]: per-pixel rendering of Wythoff tilings. Needed for item 11.
- [BALOGLOU]: two-colour wallpaper patterns. Needed for item 6.
- [SENECHAL1988] and [COXETER1986]: conventions and the historical method.
- [LUBOTZKY2003]: the primary text for Hall's formula.
- [DRESS]: the axioms of Delaney-Dress symbols in general dimension. [BALKE1996] was read in full and gives the two-dimensional case (Section 9.2).
- [KAPLAN2009]: the colouring of isohedral tilings.
- [VONGAGERN2009]: the per-pixel method.
- The paper by Conway and Huson on orbifold notation (2002), held locally by the project and not read.
- [TESS]: the tool's documentation.

## 15. References

Every web address was accessed on 2026-11-15, except those of the sources of Sections 7.8 to 7.21, which were accessed on 2026-11-17. Page numbers of [FRETTLOH2008] refer to arXiv:0807.4630v1.

- [ACE] Gamble, G., manual of the GAP package ACE, the coset enumerator of G. Havas and C. Ramsay, dated January 2012 as the web pass recorded it, Chapters 1, 3, 4, and 5 and Appendices A and B, <https://docs.gap-system.org/pkg/ace/htm/CHAP003.htm>. Read through a summarizing tool.
- [ARTLANDIA] Artlandia, glossary entry titled `Two-Color Symmetries`, <https://artlandia.com/wonderland/glossary/TwoColorSymmetries.html>.
- [ARXIV1806.06911] arXiv:1806.06911, 2018, Section 2, Theorem 2.1. Author and title not recorded, <https://ar5iv.labs.arxiv.org/html/1806.06911>.
- [BALKE1996] Balke, L. and D. H. Huson, "Two-dimensional groups, orbifolds and tilings", Geometriae Dedicata 60, pp. 89-106, 1996. Read in full. Page numbers are the printed pages.
- [BALOGLOU] Baloglou, G., "Isometrica", chapter 6, EMIS. Year and address not recorded.
- [BCCC2025] Bujalance, Cirre, Conder, and Costa, arXiv:2502.03594v1, 2025, introduction. Title and address not recorded.
- [BULATOV2010] Bulatov, V., "Coloring a hyperbolic tiling", part of the talk "Artistic Models of the Hyperbolic Geometry", G4G9, 2010-03-25, `https://www.bulatov.org/math/1003/index_color.html`, and the gallery `https://www.bulatov.org/math/1003/color_s542/index.html`.
- [BROWN2011] Brown, K., "Mathematics 6310: The Todd-Coxeter procedure", Cornell University, April 2011. Read in full from a local copy that records no address. The web pass found it at <https://pi.math.cornell.edu/~kbrown/6310/toddcox.pdf>. Page numbers are the PDF pages.
- [BULATOV2014] Bulatov, V., "Inversive Kaleidoscopes and their Visualization", Proceedings of Bridges 2014, pp. 329-332, 2014.
- [BURNS1983] Burns and Solitar, Proc. Amer. Math. Soc. 89 (1983) 414-418. Title not recorded.
- [CANNON-HAVAS1992] Cannon, J. and G. Havas, "Algorithms for Groups", The Australian Computer Journal 24(2), pp. 51-60, May 1992, arXiv:math/9406203v1, 1994-06-15, <https://arxiv.org/abs/math/9406203>. Read in full from a local copy of the arXiv version. Page numbers are the PDF pages of that version.
- [CBGS2008] Conway, Burgiel, and Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1-56881-220-5, <https://www.routledge.com/The-Symmetries-of-Things/Conway-Burgiel-Goodman-Strauss/p/book/9781568812205>. Read in part. Page numbers are the printed pages.
- [CDHW1973] Cannon, J. J., Dimino, L. A., Havas, G., and J. M. Watson, "Implementation and analysis of the Todd-Coxeter algorithm", Math. Comp. 27(123) (1973) 463-490. Not read. The web pass found two conflicting DOIs and verified neither.
- [CHR2007] Campbell, C. M., Havas, G., and E. F. Robertson, "Addendum to an elementary introduction to coset table methods in computational group theory", in Groups St Andrews 1981, London Mathematical Society Lecture Note Series 71, pp. 361-364, dated 2007 in the reference list of [BROWN2011]. Read in full from a local copy, whose running headers give pp. 362-364 and whose text dates itself after July 2006. The address of the copy is not recorded.
- [CLM1954] Coxeter, Longuet-Higgins, and Miller, "Uniform polyhedra", Phil. Trans. R. Soc. A 246 (1954) 401-450.
- [CONDER2005] Conder, M. and P. Dobcsanyi, "Applications and adaptations of the low index subgroups procedure", Math. Comp. 74(249) (2005). Not read. The title is as the web pass recorded it. [SYMPY-FPG], lines 770-771, spells it "Adaptions".
- [COXETER1986] Coxeter, "Coloured symmetry", in "M. C. Escher: Art and Science", North-Holland, 1986, pp. 15-33, ISBN 978-0-444-70011-7.
- [DDL2009] De Las Penas, Decena, and Laigo, arXiv:0906.3412, 2009-06-18, abstract. Title and address not recorded.
- [DLP2006] De Las Penas, Felix, and Laigo, "Colorings of hyperbolic plane crystallographic patterns", Z. Kristallogr. 221(10) (2006) 665-672, DOI 10.1524/zkri.2006.221.10.665, <https://api.crossref.org/works/10.1524/zkri.2006.221.10.665>.
- [DPFP2007] De Las Penas, Felix, and Provido, "On index 2 subgroups of hyperbolic symmetry groups", Z. Kristallogr., 2007.
- [DRESS] Dress, 1984, and Dress and Huson, 1987, on Delaney-Dress symbols, as cited in [TEGULA2020]. Titles not recorded.
- [EEK1982] Edmonds, Ewing, and Kulkarni, Invent. Math. 69 (1982) 331-346, DOI 10.1007/BF01389358. Title not recorded.
- [EEK1982-BULL] Edmonds, Ewing, and Kulkarni, announcement, Bull. Amer. Math. Soc. 6 (1982) 456-458, DOI 10.1090/S0273-0979-1982-15014-5.
- [EPINET-DD] EPINET, "Delaney-Dress symbols", <https://epinet.anu.edu.au/page/epinet2_mathematics_delaney_dress>.
- [EPINET-ORB] EPINET, "Orbifold notation", <https://epinet.anu.edu.au/page/epinet2_mathematics_orbifold_notation>.
- [EWS] EPFL, "Escher Web Sketch", <https://www.epfl.ch/schools/sb/research/iphys/teaching/crystallography/escher-web-sketch/>.
- [FELIX2008] Felix, R. P. and M. J. C. Loquias, "Enumerating and identifying semiperfect colorings of symmetrical patterns", Z. Kristallogr. 223 (2008) 483-491, DOI 10.1524/zkri.2008.0053, arXiv:1002.0536v1, <https://ar5iv.labs.arxiv.org/html/1002.0536>.
- [FRETTLOH2008] Frettloh, D., "Counting perfect colourings of plane regular tilings", Z. Kristallogr. 223(11-12) (2008) 773-776, DOI 10.1524/zkri.2008.1044, arXiv:0807.4630v1, 2008-07-29, <https://ar5iv.labs.arxiv.org/html/0807.4630>.
- [GAP] The GAP Group, "GAP Reference Manual", Chapter 47, "Finitely Presented Groups", <https://docs.gap-system.org/doc/ref/chap47.html>, read in full from a local copy, and Section 39.19-6, read through a summary. The local copy states no release or date. The web pass read "GAP Release 4.16.1", dated 2026-08-23, on the front page of the manual through a summarizing tool, so the release is unverified. Line numbers refer to the local copy.
- [GS1987] Grunbaum and Shephard, "Tilings and Patterns", Freeman, 1987, ISBN 978-0-7167-1193-3, and Dover, 2nd edition, 2016, ISBN 978-0-486-46981-2. Chapter 8 is titled `Colored Patterns and Tilings`.
- [HALL1949] Hall, M., Canad. J. Math. 1 (1949) 187-190.
- [HAREL1993] Har'El, "Uniform solution for uniform polyhedra", Geom. Dedicata 47 (1993).
- [HAVAS1991] Havas, G., "Coset enumeration strategies", Proceedings of ISSAC '91, ACM Press, pp. 191-199, 1991, arXiv:math/9406202v1, 1994-06-15, <https://arxiv.org/abs/math/9406202>. Read in full from a local copy of the arXiv version. Page numbers are the printed pages, which are the PDF pages plus 190.
- [HAVAS-RAMSAY2000] Havas, G. and C. Ramsay, "Proving a group trivial made easy: a case study in coset enumeration", Bull. Austral. Math. Soc. 62 (2000) 105-118, as [CHR2007] cites it. Not read.
- [HOLT2005] Holt, D. F., Eick, B., and E. A. O'Brien, "Handbook of Computational Group Theory", Chapman and Hall/CRC, 2005, ISBN 978-1-58488-372-2, DOI 10.1201/9781420035216. Not read.
- [HOLT2013] Holt, D., "Presentation of Groups", notes by F. Bouyer, 2013-04-23, GNU Free Documentation License 1.3. Pages 14-23 read from a local copy that records no address. The web pass found it at <https://warwick.ac.uk/fac/sci/maths/people/staff/fbouyer/presentation_of_group.pdf>. Page numbers are the printed pages, which equal the PDF pages.
- [HULPKE2010] Hulpke, A., "Notes on Computational Group Theory", Colorado State University, Spring 2010. Printed pp. 72-84 and the bibliography read from a local copy that records no address. The web pass found the versions of 2008 and 2015 at <https://archive.math.arizona.edu/asp/2008/cgtnotes3.pdf> and <https://www.math.colostate.edu/~hulpke/CGT/cgtnotes.pdf>. Page numbers are the printed pages, which are the PDF pages minus 8.
- [HYPERROGUE] HyperRogue, `pattern2.cpp` on the branch `master`, <https://raw.githubusercontent.com/zenorogue/hyperrogue/master/pattern2.cpp>, and <https://zenorogue.github.io/hyperrogue-doc/files.html>.
- [IORNAMENT] iOrnament, App Store listing, <https://apps.apple.com/app/id534529876>.
- [ITC] Felsch, V., Hippe, L., and J. Neubuser, manual of the GAP package ITC, Interactive Todd-Coxeter, Chapter 3, undated, <https://docs.gap-system.org/pkg/itc/htm/CHAP003.htm>. The authors are as [BROWN2011] cites the package, with the year 2004. Read through a summarizing tool.
- [IZQUIERDO1993] Izquierdo, Proc. London Math. Soc. (3) 67 (1993) 305-328. Title not recorded.
- [KALEIDOTILE] Geometry Games, KaleidoTile, version 4.1.5 for Mac, <https://www.geometrygames.org/KaleidoTile/index.html.en>, source archive <https://www.geometrygames.org/KaleidoTile/KaleidoTile-Src.zip>.
- [KALI] Kali, Debian package 3.1+dfsg-4, `HELP` and README, <https://sources.debian.org/data/main/k/kali/3.1+dfsg-4/HELP>, and <https://mige-web.units.it/~logar/matdue/Kali/instructions.html>.
- [KAPLAN2009] Kaplan, "Introductory Tiling Theory for Computer Graphics", Morgan and Claypool, 2009.
- [KAPLAN2019] Kaplan, "Animated Isohedral Tilings", Proceedings of Bridges 2019, pp. 99-106, 2019.
- [KAPLAN2025] Kaplan, C. S., "Isohedral Tiles, Tilings, and Notations", 2025-09-01, <https://isohedral.ca/?p=2780>.
- [KOLBE-EVANS] Kolbe and Evans, Geom. Dedicata, DOI 10.1007/s10711-020-00554-2. Title and year not recorded.
- [LOWINDEX] Culler, M., Dunfield, N., and M. Gorner, `low_index`, README, <https://github.com/3-manifolds/low_index>, read through a summarizing tool, and "LOW INDEX, Revisited and Implemented", the LaTeX source `doc/lowindex.tex`, <https://raw.githubusercontent.com/3-manifolds/low_index/master/doc/lowindex.tex>, read in full from a local copy. The file states no date. The web pass found commits of the file authored from 2022-06-22 to 2022-07-01. Sections are those of the note.
- [LPD2008] Laigo, Puzon, and De Las Penas, Proceedings of Bridges Leeuwarden 2008, pp. 311-318, 2008, titled `Coxeter Groups in Colored Tilings and Patterns`, <https://archive.bridgesmathart.org/2008/bridges2008-311.html>.
- [LUBOTZKY2003] Lubotzky and Segal, "Subgroup Growth", Birkhauser, 2003.
- [MAGMA] Magma Handbook, low index subgroups, <https://magma.maths.usyd.edu.au/magma/handbook/text/898>, and an older mirror of its pages on coset enumeration and low index subgroups, <https://www.math.ru.nl/magma/text237.html>, `text239.html`, and `text243.html`, version not stated. Read through a summarizing tool.
- [MENDELSOHN1964] Mendelsohn, N. S., "An algorithmic solution for a word problem in group theory", Canad. J. Math. 16 (1964). Not read. Cited as "Mendelsohn, 1964" by [HULPKE2010], p. 77, without an entry.
- [MALINC] Christersson, M., "Image Tiling", 2015-04-19, <https://www.malinc.se/m/ImageTiling.php>.
- [MATEI2002] Matei, D. and A. Suciu, "Hall invariants, homology of subgroups, and characteristic varieties", Int. Math. Res. Not. 2002, arXiv:math/0010046, Section 8.1, eq. (8.2), read through its ar5iv rendering.
- [MORENAMENTS] "Morenaments", exhibit sheet, <https://exponate.phaeno.de/media/pages/experimente/4916/f19b5bd2f0-1714643520/4916_Morenaments_en.pdf>.
- [NELSON2019] Nelson, R., slides, ICERM, 2019. Title not recorded.
- [NEUBUSER1982] Neubuser, J., "An elementary introduction to coset table methods in computational group theory", in Groups St Andrews 1981, London Mathematical Society Lecture Note Series 71, Cambridge University Press, 1982, pp. 1-45. Not read.
- [SCHWARZENBERGER1984] Schwarzenberger, R. L. E., "Colour symmetry", Bull. London Math. Soc. 16 (1984) 209-240.
- [SENECHAL1988] Senechal, Comput. Math. Applic. 16 (1988) 545-553, DOI 10.1016/0898-1221(88)90244-1, titled `Color symmetry`.
- [SIMS1994] Sims, C. C., "Computation with Finitely Presented Groups", Encyclopedia of Mathematics and its Applications 48, Cambridge University Press, 1994, ISBN 978-0-521-43213-7, DOI 10.1017/CBO9780511574702, Chapter 5 and p. 208. Not read.
- [SINGERMAN1974] Singerman, D., "On the structure of non-Euclidean crystallographic groups", Proceedings of the Cambridge Philosophical Society 76, pp. 233-240, 1974, DOI 10.1017/S0305004100048891, <https://doi.org/10.1017/S0305004100048891>. Read in full. Page numbers are the printed pages.
- [SINGERMAN2014] Singerman and Watson, arXiv:1408.0127, 2014, Section 2, Example 1 and Theorem 1, <https://ar5iv.arxiv.org/html/1408.0127>. Title not recorded.
- [SYMMHUB] Bulatov, V., SymmHub, MIT licence, 2025, read at commit `f2d19cb` (committed 2026-11-13) on 2026-11-15, and its files `lib/sublib/src/sublib_core.js` and `lib/sublib/src/reidemeister.js` read line by line at the same commit on 2026-11-17, with `lib/sublib/README.md` read for the section `Compatibility with the GAP-generated files` only, <https://github.com/SymmHub/SymmHub>. File paths and lines are relative to the root of the repository.
- [SYMPY] SymPy documentation, "Finitely presented groups", <https://docs.sympy.org/latest/modules/combinatorics/fp_groups.html>. Read through a summarizing tool.
- [SYMPY-CT] SymPy, `sympy/combinatorics/coset_table.py`, branch `master`, fetched 2026-11-17, <https://raw.githubusercontent.com/sympy/sympy/master/sympy/combinatorics/coset_table.py>. Read in full from the fetched copy, which states no licence. Line numbers refer to that copy.
- [SYMPY-FPG] SymPy, `sympy/combinatorics/fp_groups.py`, branch `master`, fetched 2026-11-17, <https://raw.githubusercontent.com/sympy/sympy/master/sympy/combinatorics/fp_groups.py>. Read in full from the fetched copy, which states no licence. Line numbers refer to that copy.
- [TACTILE] Kaplan, `tactile-js`, BSD-3-Clause, 2018, <https://github.com/isohedral/tactile-js>, and `lib/tactile.js`, <https://raw.githubusercontent.com/isohedral/tactile-js/master/lib/tactile.js>.
- [TEGULA2020] Zeller, R., Delgado Friedrichs, O., and D. H. Huson, "Tegula -- exploring a galaxy of two-dimensional periodic tilings", arXiv:2007.10625v1, 2020-07-21, Sections 4 and 5.
- [TESS] Tess, peda.com. The site refused the connection.
- [TODD-COXETER1936] Todd, J. A. and H. S. M. Coxeter, "A practical method for enumerating cosets of a finite abstract group", Proc. Edinburgh Math. Soc. (2) 5 (1936) 26-34, DOI 10.1017/S0013091500008221. Not read.
- [VONGAGERN2009] von Gagern and Richter-Gebert, "Hyperbolization of euclidean ornaments", Electron. J. Combin. 16 R2, 2009.
- [WP-COXETER-DYNKIN] Wikipedia, "Coxeter-Dynkin diagram", section "Application to uniform polytopes", <https://en.wikipedia.org/wiki/Coxeter%E2%80%93Dynkin_diagram>.
- [WP-INDEX] Wikipedia, "Index of a subgroup", <https://en.wikipedia.org/wiki/Index_of_a_subgroup>.
- [WP-POLYCHROMATIC] Wikipedia, "Polychromatic symmetry", <https://en.wikipedia.org/wiki/Polychromatic_symmetry>.
- [WP-SNUB] Wikipedia, "Snub (geometry)". Address not recorded.
- [WP-UNIFORM-HYP] Wikipedia, "Uniform tilings in hyperbolic plane", <https://en.wikipedia.org/wiki/Uniform_tilings_in_hyperbolic_plane>.
- [WP-UNIFORM-TILING] Wikipedia, "Uniform tiling", <https://en.wikipedia.org/wiki/Uniform_tiling>.
- [WP-WYTHOFF-SYMBOL] Wikipedia, "Wythoff symbol", section "Description", <https://en.wikipedia.org/wiki/Wythoff_symbol>.
- [ZHAO] Zhao Liang, "Coxeter groups, automata and uniform tilings", undated, <https://neozhaoliang.github.io/uniform-tilings>.
- [ZUCKER] Zucker, M., Shadertoy 3tsSzM, <https://www.shadertoy.com/view/3tsSzM>.
