# Colourings of Tilings Associated with a Symmetry Group

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-23 |
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
- HLT strategy: The relator-based strategy of coset enumeration, after Haselgrove, Leech, and Trotter (Section 7.11). [HOLT2005] spells the first name Hazelgrove (pp. 149 and 162).
- Felsch strategy: The coset-table-based strategy, which defines the first undefined entry of the table (Section 7.12).
- deduction stack: The list of entries assigned and not yet checked against the relators, in the Felsch strategy (Section 7.12) [HOLT2005, p. 165] [SIMS1994, p. 234].
- compression: The removal of dead rows and the renumbering of the live cosets without gaps (Section 7.14) [BROWN2011, p. 5].
- standardized coset table: A table whose cosets are numbered in the order of their first appearance, read row by row (Section 7.14) [HOLT2005, p. 167] [SIMS1994, p. 133].
- lenlex, semilenlex: GAP's standard forms, which read all columns or only the generator columns (Section 7.14) [GAP, Section 47.7-1].
- low-index subgroup enumeration: The algorithm of Sims that lists the subgroups of a finitely presented group up to a bound on the index, up to conjugacy [GAP, Section 47.10-1] [LOWINDEX].
- first-in-class test: The test of the low-index search that keeps a table only if no relabelling from another base point gives a smaller table (Section 7.17).
- Schreier generator: A generator of a subgroup formed from a coset representative, a generator of the group, and the representative of their product (Section 7.18).
- representative set: The set of polygons, one from each class of polygons that the Escher renderer colours alike, whose size the first deliverable reports to the user as the available number of colours. This is a project term, and no source defines it.
- face orbit: The set of the images of one face of a tiling under its symmetry group.
- Wythoff construction: The construction of a polyhedron or tiling whose vertices are the images of one point of a reflection triangle under the group that the reflections in its sides generate (Section 10.1) [CLM1954, p. 406] [HAREL1993, p. 6].
- generator point: The point of the fundamental triangle whose images under the group are the vertices of a Wythoff tiling (Section 10.1). [CLM1954] and [HAREL1993] call it C, and Har'El calls it the point object [HAREL1993, p. 6]. Martelli calls it the seed point [MARTELLI2026, Section 5.3].
- active mirror: A mirror of the fundamental triangle that does not contain the generator point. Its node is the one that [CBGS2008, p. 389] rings (Sections 10.1 and 10.2).
- Wythoff symbol: One of the symbols `p | q r`, `p q | r`, `p q r |`, and `| p q r`, each of which fixes the position of the generator point in the triangle with the angles $\pi/p$, $\pi/q$, and $\pi/r$ (Section 10.1) [CLM1954, pp. 406-408].
- ringed node: A node of a Coxeter-Dynkin diagram that is circled because the generator point does not lie on its mirror (Section 10.2) [CBGS2008, p. 389] [SCHULTE2016, Section 3.2]. Martelli says encircled [MARTELLI2026, Section 5.3].
- vertex configuration: The cyclic list of the numbers of sides of the faces at a vertex, in the order in which they occur around it [HAREL1993, p. 2]. [CLM1954] gives the same list as the labels of the sides of a vertex figure (p. 404).
- Coxeter-Wythoff subdiagram: A subdiagram of a ringed diagram in which each connected component contains at least one ringed node (Section 10.2) [MARTELLI2026, Proposition 38].
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
- The order of the numbers in a Wythoff symbol: [CLM1954] states that the order on each side of the bar does not matter (Section 10.1). Its tables and those of [HAREL1993] print the 2 first after the bar, as in `3 | 2 5`, and its general statements print it in either place, as in `q | p 2` (p. 406) and `p | 2 q` (p. 413). This document writes a general symbol as [CLM1954] states it and quotes a table row as printed.
- The order of a vertex configuration: the lists that [HAREL1993] prints in its formulas and in its tables differ for some forms, such as `(p.2r.q.2r)` on p. 8 and `(6. 6. 3)` for `2 3|3` on p. 15, and the paper does not comment. [CLM1954] gives the order in a drawing of the vertex figure (p. 434). This document quotes each list in the order of its source.

### 2.4. Spelling of Names and Symbols

Names with accented letters are written in ASCII transliteration: Frettloh, Grunbaum, De Las Penas, Gorner, Dobcsanyi, Neubuser, Schlafli, Poincare, and Mobius. Orbifold signatures, Schlafli symbols, vertex configurations, tiling symbols, and Wythoff symbols are written as code spans, such as `*642`, `{7,3}`, `3.3.p.3.q`, `(7^3)`, and `p | q r`.

[CLM1954] prints the symbol of a quasi-regular polyhedron with $p$ above $q$ inside braces. This document writes it `{p;q}`, as in `r{3;4}` and `s{3;5}`. [CBGS2008] writes angle brackets and a centred dot in its archifold symbols, written here as `<` and `>` and as `.`, as in `<0>*p^q` and `[0][1][2].(2p,2q,4)`.

Quoted text keeps the spelling of its source, and its mathematical symbols are written in ASCII, such as "in" for the element sign and "pi" for the Greek letter. Where a source title or a web address contains a spelling that the prose checker rejects, it is written as a code span.

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
| [HOLT2005] | Book, Chapter 5, pp. 149-198, and the pages of Chapters 2 and 4 that it relies on | From a copy that the owner supplied, through a garbled text layer, with most pages of pseudocode and examples also checked against page images (Section 15 lists which) |
| [SIMS1994] | Book, Section 3.8, most of Chapter 4, Chapter 5, and the Appendix on pp. 576-577 | From a copy that the owner supplied, through its text layer, with some pages also read from page images (Section 15) |

The text layers of [HOLT2013] and [HULPKE2010] lose superscripts and subscripts. A quotation from them writes exponents in LaTeX or ASCII, as Section 2.4 states, and its locator says "exponents restored" where the research put them back. The text layer of [HOLT2005] also drops Greek letters, set symbols, and most presentations, so the research checked the pages with pseudocode, symbols, or tables against page images, except the pages that Section 15 lists as read from the text layer only. The text layer of [SIMS1994] turns Greek letters into Latin ones, flattens exponents, and loses the blank cells of tables. Its quotations write the symbols in ASCII, and a locator says "symbols restored" where the research put a symbol back.

In Sections 7.9 to 7.21, "Holt" names the lecture notes [HOLT2013], and "the handbook" names the book of Holt, Eick, and O'Brien [HOLT2005]. The two number their results differently (Section 7.15).

SymPy is an implementation, not a mathematical authority. Its docstrings cite the handbook by page, as "pg. 155 [1]", and the paper of Cannon, Dimino, Havas, and Watson [CDHW1973]. The handbook was read, so the sections below cite its own pages for the steps that SymPy took from it, and this table records what each page that SymPy cites holds [SYMPY-CT] [SYMPY-FPG] [HOLT2005]:

| SymPy cites | For | What the printed page holds |
| --- | --- | --- |
| p. 152 | `is_complete` | The definition of a complete table, Properties 1 to 4, and the initialization |
| p. 153 | `define` | DEFINE, Proposition 5.1, and the prose definition of a scan, of its completion, and of a deduction |
| p. 155 | `scan` | SCAN and the proof of Theorem 5.2 |
| pp. 156-161 | `coincidence` | The coincidence routines are on pp. 157-158. Pages 160-161 hold Example 5.4, the strategy, Property 5, and Theorem 5.5 on termination |
| p. 157 | `merge`, `rep` | REP and MERGE, with the choice of the smaller number and its reason |
| pp. 85-87 | `rep`, `merge`, `minimal_block` | Chapter 4: MINIMALBLOCK, REP and MERGE with a size array, and the remark on the larger class. Not coset enumeration |
| pp. 162-163 | `scan_and_fill` | The relator-based method and SCANANDFILL |
| p. 164 | `look_ahead`, `coset_enumeration_r` | COSETENUMERATIONR and LOOKAHEAD |
| p. 165 | `define_c`, `scan_c`, `scan_and_fill_c` | Prose only, on the Felsch policy, the deduction stack, and the lists of cyclic conjugates. There is no pseudocode for the modified DEFINE, SCAN, and COINCIDENCE |
| p. 166 | `process_deductions`, `coset_enumeration_c` | PROCESSDEDUCTIONS and COSETENUMERATIONC, from which the code departs in the choice of lists (Section 7.12) |
| p. 167 | `compress`, `switch` | COMPRESS, SWITCH, and the definition of a standardized table |
| pp. 167-168 | `standardize` | SWITCH, STANDARDIZE, and a worked table |
| p. 175 | `define_schreier_generators` | DEFINESCHREIERGENERATORS and its worked table |
| p. 177 | Example 5.6 | Example 5.6 |
| p. 183 | Examples 5.8 and 5.9 | Both examples. Example 5.9 ends on p. 184 |
| "Exercises Q2. Pg 187" | A doctest of `fp_groups.py` | Exercise 1 of Section 5.3 on p. 187. Its group is that of Exercise 2 on p. 161 |
| "Exercises Pg. 161, Q2" | A doctest | Exercise 2 of Section 5.1 |
| "Section 5.4" | `low_index_subgroups` | The low-index method, pp. 188-197 |

SymPy's description of `first_in_class` as "taken directly from the function on page 208 of [Sim94]" repeats the handbook's own sentence [HOLT2005, p. 193]. [CDHW1973] was not read, so a step that SymPy's code states and neither book states is still the code's own (Section 14.1, item 23). The same holds for SymmHub's code. The primary texts [TODD-COXETER1936], [CDHW1973], [NEUBUSER1982], and [MENDELSOHN1964] were not read (Section 14.3).

### 7.9. The Coset Table

A coset table of a subgroup $H$ of a finitely presented group $G$ "simply lists the cosets and displays the action of the generators and their inverses". "Each row corresponds to a coset, and each column corresponds to a generator or its inverse" [BROWN2011, Section 5, p. 5]. In GAP, "The rows correspond to the cosets, the columns correspond to the generators and their inverses in the ordering g_1, g_1^{-1}, g_2, g_2^{-1}", and an entry $l[i] = j$ means that the generator "takes the coset i to the coset j by multiplication from the right" [GAP, Section 47.6-1]. "Basically a coset table is the permutation representation of the finitely presented group on the cosets of a subgroup" [GAP, Section 47.6-1].

"Coset 1 is defined to be the subgroup. Every other coset is defined to be the image of a prior coset under a generator" [HULPKE2010, p. 72]. While a table is being built, "some of the entries in the table are not known. One common convention is to enter 0 in these places" [BROWN2011, p. 5]. Section 7.21 lists the other conventions.

Holt states three properties that the table keeps during an enumeration [HOLT2013, pp. 15-17, exponents restored]. The table holds a set $\Omega$ of positive integers, and each $i \in \Omega$ has a word $a_i$ in the generators and their inverses with $i = Ha_i$. Always $1 \in \Omega$ and $a_1$ is the empty word. Holt writes $ix$ for the image of $i$ under $x$:

- (R1) "ix = j if and only if jx^-1 = i".
- (R2) "If i -> a_i ... with i in Omega, then 1a_i is defined and equal to i."
- (R3) "If i -> a_i, j -> a_j and ix = j, then Ha_i x = Ha_j."

Proposition 5.1 states that (R3) remains true during the process, and after coincidences "It can be shown that (R1),(R2),(R3) remain true" [HOLT2013, pp. 16-17]. The notes give no proof of the second statement.

The handbook defines a coset table as a quintuple $(\tau, \chi, p, n, M)$, whose rows are the live cosets $\Omega$ and whose columns are the set $A = X \cup X^{-1}$ of the generators and their inverses [HOLT2005, pp. 151-152]. Here $p[\alpha] = \alpha$ exactly when $\alpha$ is live, $n$ is the largest coset number used so far, and $M$ is the limit on the number of cosets. It states four properties [HOLT2005, p. 152]:

1. $1 \in \Omega$, and $\tau(1)$ is the empty word.
2. $\alpha^x = \beta$ if and only if $\beta^{x^{-1}} = \alpha$.
3. $\alpha^x = \beta$ implies $H\tau(\alpha)x = H\tau(\beta)$.
4. $1^{\tau(\alpha)} = \alpha$.

The notes' (R1), (R2), and (R3) are the handbook's Properties 2, 4, and 3, and its Property 1 is the initialization. The word $\tau(\alpha)$ serves the proofs and "is not part of the implementation" [HOLT2005, pp. 152-153]. Proposition 5.1 proves that a deduction keeps Properties 1 to 4, and Proposition 5.4 proves that coincidence processing keeps them [HOLT2005, pp. 153-154 and 158-159]. The docstring of SymPy's class `CosetTable` lists the handbook's four properties in its words, with "0 in Omega" in place of "1 in Omega" [SYMPY-CT, lines 28-34].

Sims calls a coset table "the transition table for a coset automaton", whose edges come in pairs: "If (sigma, x, tau) is in E, then (tau, x^-1, sigma) is in E" [SIMS1994, p. 171, symbols restored]. His setting differs from the other sources. The group is $F/N$, where $F$ is a free product of cyclic groups of order 2 and of infinite order, given by a rewriting system $R$, and $N$ is the normal closure of a finite set $V$ of cyclically reduced words [SIMS1994, pp. 153-154 and 217-218]. The alphabet $X$ holds the inverse of each generator of infinite order as a separate letter, and an involution $x$ has $x^{-1} = x$ and one column. A relation $x^2 = 1$ therefore goes into $R$, and a power such as $x^3$ goes into $V$ as a written-out word. Generators of finite order greater than 2 get no special treatment in the main text, and the historical note says that "The paper (Atkinson 1981) was the first to suggest special treatment for generators of finite order greater than 2" [SIMS1994, p. 266].

Hulpke also keeps "a table for every relator", which traces "the images of every coset under the relator generator by generator", because "the coset needs to remain fixed", and "a similar table for every subgroup generator", where "we only require the trivial coset to remain fixed" [HULPKE2010, p. 72]. On a computer, the relator and subgroup tables and the inverse columns need not be stored, because their contents can be computed "by scanning through relators ... forwards and backwards" [HULPKE2010, Note III.22, p. 75]. Havas reports that a table needs "(2g - i) x n words" for $n$ cosets and $g$ generators of which $i$ are involutions, because the inverse column of an involution can be dropped [HAVAS1991, p. 192]. The handbook says the same: "when there is a relator x^2, we know that x =_G x^-1, and we can save time and space by omitting the column for x^-1 in the coset table" [HOLT2005, p. 154, exponents restored]. It adds that the column of $x^{-1}$ "is necessary for the computation of a presentation of H on the set of Schreier generators", unless the trivial Schreier generators are omitted [HOLT2005, p. 174]. Its procedures all loop over $A$, so it gives no pseudocode for the reduced columns, and it does not say how a scan treats $x$ as its own inverse.

The handbook assumes that the relators and the subgroup generators are reduced words [HOLT2005, p. 151]. Neither book treats a relator of one letter as a topic. The handbook mentions relators of length 1 and 2 only for Tietze transformations [HOLT2005, p. 185], and Sims treats only involutions through $R$ (above).

A coset is live when no coincidence has eliminated it, and SymPy keeps the "Set of live cosets" as the cosets $c$ with `p[c] == c` [SYMPY-CT, lines 88-91]. "The coset table is called complete if it has no undefined entries on the live cosets" [SYMPY-CT, lines 121-128], which is the handbook's definition [HOLT2005, p. 152]. When all tables are closed and no deductions are pending, "the columns of the coset table give permutation images for the group generators that are consistent with all relators", which gives a homomorphism $\varphi : G \to S_n$ with $[G : S] = n$ equal to the number of rows [HULPKE2010, p. 75].

### 7.10. Definitions, Deductions, and Coincidences

Coset enumeration changes the table by three kinds of event. Cannon and Havas state the rules that drive them: "(a) coset 1 must be mapped to itself by each of h1, ..., ht" and "(b) each coset j must be mapped to itself by each of the defining relators R1, ..., Rs and by each product xk xk^-1", where $h_1, \dots, h_t$ generate $H$. "Enforcement of rules (a) and (b) yields values for some hitherto unknown coset table entries (deductions) and, also, the identification of cosets which have been multiply defined (coincidences)" [CANNON-HAVAS1992, Section 2.2, p. 3].

A definition adjoins a new coset. Holt states it as follows: "If some ix is not defined then we can choose some j > 0, j not in Omega, replace Omega by Omega u {j} and define ix = j, jx^-1 = i and put a_j = a_i x" [HOLT2013, p. 15, exponents restored]. The handbook's DEFINE takes the next number instead of any free one [HOLT2005, p. 153]. These steps restate it for a live coset $\alpha$ and a column $x$ with $\alpha^x$ undefined:

1. If $n = M$, abort.
2. Set $n = n + 1$ and $\beta = n$, and mark $\beta$ live.
3. Set $\alpha^x = \beta$ and $\beta^{x^{-1}} = \alpha$.

"DEFINE always uses n+1", where $n$ is the largest number used so far, so a number that a coincidence frees is not reused before compression [HOLT2005, p. 167]. The notes' word $a_j = a_i x$ is the handbook's $\tau(\beta) = \tau(\alpha)x$, which the handbook writes as a comment that "is not part of the implementation" [HOLT2005, pp. 152-153]. Sims's DEFINE in Chapter 4 also takes $n + 1$ and does not reuse a number [SIMS1994, p. 176], and he notes that reusing deleted numbers can save considerable space with some data structures [SIMS1994, p. 231]. His Appendix describes the recycling of state numbers (Section 7.14).

A deduction is an entry that a relator or subgroup generator forces. Hulpke writes that when an entry "fills the last hole in a table", "we get an deduction that the image of one coset under the next generator must be the existing value on the other side of the table entry. We enter these deductions in the table as if they were definitions" [HULPKE2010, p. 73]. A coincidence is the discovery that two coset numbers name one coset: "Sometimes we get a deduction ix = j where we already know ix = k with k != j, (or kx = j for k != i). This means coset Ha_j and Ha_k (respectively Ha_i, Ha_k) are equal" [HOLT2013, p. 17, exponents restored]. Section 7.21 lists the other meanings of "deduction" in the sources.

Hulpke describes scanning a relator "forwards and backwards" in prose [HULPKE2010, p. 75]. The handbook states the scan in prose and as the 15-line procedure SCAN [HOLT2005, pp. 153 and 155], and SymPy's code follows it and cites p. 155 [SYMPY-CT, lines 286-379]. These steps restate it for a coset $\alpha$ and a word $w$, which is a relator or a subgroup generator:

1. Find the longest prefix $s$ of $w$ for which $\alpha^s$ is defined, and write $w = sv$.
2. Find the longest suffix $t$ of $v$ for which $\alpha^{t^{-1}}$ is defined, and write $v = ut$.
3. If $t = v$, "the scan completes", and it completes correctly if $\alpha^s = \alpha^{t^{-1}}$ [HOLT2005, p. 153]. If it completes and the two cosets differ, they are a coincidence, which SCAN passes to coincidence processing (Section 7.13) [HOLT2005, p. 155].
4. If $u$ is a single letter $x$, set $\beta^x = \gamma$ and $\gamma^{x^{-1}} = \beta$ for $\beta = \alpha^s$ and $\gamma = \alpha^{t^{-1}}$. "These assignments are known as a deduction" [HOLT2005, p. 153].
5. Otherwise the "scan is incomplete and yields no information" [HOLT2005, p. 155] [SYMPY-CT, line 379].

The handbook adds that "because of Property 2, it can only complete correctly when s=w, t=empty" [HOLT2005, p. 153, symbols restored]. Proposition 5.3 states that two cosets $\alpha^s$ and $\alpha^{t^{-1}}$ found by a scan of a relator name the same coset of $H$, and leaves its proof "to the reader" [HOLT2005, p. 156].

Sims's primitive is the two-sided trace TWO_SIDED_TRACE [SIMS1994, pp. 181-183]. It traces forwards from $\sigma$ as far as possible and backwards from the end as far as possible, which leaves a gap $T$. If $T$ is empty and the two states differ, it calls COINCIDENCE. If $T$ has one letter, it calls JOIN, which writes a deduction. If $T$ is longer, a partial trace "terminates unsuccessfully, without modifying A", and a full trace, chosen by a flag, defines a coset at the start of the gap and repeats. Propositions 5.1, 5.3, 5.4, and 5.5 of Chapter 4 prove the effect of JOIN, COINCIDENCE, and the trace on the subgroup of the automaton, and the proof of Proposition 5.5 is left as an exercise [SIMS1994, Chapter 4].

### 7.11. The Relator-Based (HLT) Strategy

A strategy decides where the next definition goes. The relator-based strategy is "often called the HLT method (after Haselgrove, Leech, and Trotter). In this method we consider the vertices in numerical order, and we make sure that the graph is complete at each vertex before moving on to the next" [BROWN2011, p. 4]. Holt's second strategy is to "Go through relator tables in order filling in gaps as you go (even for long gaps)" [HOLT2013, p. 18]. Hulpke states HLT more narrowly: "If there are gaps of length 1 in a subgroup or relator table, fill these gaps (in the hope of getting immediately a consequence)" [HULPKE2010, p. 77]. Section 7.21 records these differences.

The handbook's SCANANDFILL makes the definitions of the strategy [HOLT2005, p. 163]. It runs the scan of Section 7.10 in a loop. When the scan completes, it processes a coincidence if the two cosets differ and returns. When the scan leaves one gap, it writes the deduction and returns. Otherwise it defines a new coset at the first undefined position $f^{x_i}$ of the forward trace and scans again. As printed, its line 13, which calls COINCIDENCE when the two traces meet, has no return, unlike its lines 9 and 16, and the handbook does not comment on it. SymPy's scan-and-fill follows it: "whenever the procedure is called and the scan is incomplete then it makes new definitions to enable the scan to complete" [SYMPY-CT, lines 343-378 and 597-607, citing pp. 162-163 of [HOLT2005]]. Sims's full two-sided trace does the same (Section 7.10).

The handbook's HLT enumeration COSETENUMERATIONR has these steps [HOLT2005, p. 164]:

1. Start with coset 1 only, and scan-and-fill each subgroup generator at coset 1.
2. For each live coset $\alpha$ in increasing order:
   1. Scan-and-fill each relator at $\alpha$ in order, and stop this loop when a coincidence eliminates $\alpha$.
   2. If $\alpha$ is still live, define $\alpha^x$ for each $x \in A$ for which it is undefined (lines 9 and 10).
3. Return the table.

A loop over $\Omega$ is coded as a loop over $1, \dots, n$ that skips the dead cosets, with $n$ the current largest number [HOLT2005, p. 163]. Sims's HLT has the same shape: for $\sigma = 1, 2, \dots$ while $\sigma$ is live, a full two-sided trace of every relator at $\sigma$, then a definition of every undefined $\sigma^x$, and a standardization at the end [SIMS1994, pp. 227-228]. It traces the subgroup generators at coset 1 at the start only. Its variant CHLT traces the cyclic permutations of the relators [SIMS1994, p. 245].

SymPy's HLT enumeration `coset_enumeration_r` has these steps [SYMPY-CT, lines 969-1165]:

1. Scan-and-fill each subgroup generator at coset 0, which is $H$.
2. For each coset $\alpha$ in increasing order, while $\alpha$ is below the largest live coset number plus one:
   1. If $\alpha$ is live, scan-and-fill each relator at $\alpha$ in order, and stop this loop when a coincidence eliminates $\alpha$.
   2. If $\alpha$ is still live, define a new coset for each column of row $\alpha$ that is still undefined.
3. Return the table.

Holt compares the strategies: the relator-based one "leads to many more coincidences, but it is easier to program and runs fast on computers for routine examples", and "each row of each relator table only needs to be scanned once" [HOLT2013, p. 18]. "The order in which cosets are defined is explicitly prescribed by the order in which (the subgroup generators and) the group relators are processed in HLT-type procedures" [HAVAS1991, p. 192].

A variant adds lookahead: "we check all relators at all vertices (but without making new definitions) and take account of any deductions or coincidences that we can discover" [BROWN2011, p. 4]. SymPy's `look_ahead` scans every relator at every live coset without defining [SYMPY-CT, lines 654-671]. The handbook's LOOKAHEAD does the same with the relators $R$ and not their cyclic conjugates, and a live coset that a scan eliminates ends its scans [HOLT2005, p. 164]. The handbook runs it in HLT when "a call of DEFINE aborts due to lack of space", renumbers the cosets if space was recovered, and resumes the main loop at the coset where it stopped [HOLT2005, p. 164]. Sims's lookahead runs partial two-sided traces of every relator at every state, and "Typically, the enumeration is abandoned when the space released falls below some preset threshold" [SIMS1994, pp. 231-232]. Havas reports that "It has been found to be a good general rule to use the given group relators as subgroup generators", because this "ensures the early definition of some useful cosets" [HAVAS1991, p. 193].

### 7.12. The Coset-Table-Based (Felsch) Strategy

The Felsch strategy defines the first undefined entry: "Define the next coset as the first open entry (by rows and within a row by columns) in the coset table. This guarantees that the image of each coset under each generator will be defined at some point" [HULPKE2010, p. 77]. Holt states the same strategy, which defines "cosets Ha_i is order of increasing a_i in the lenlex order" [HOLT2013, p. 17]. "After each definition, we check all relators at all vertices (without making further definitions) that could possibly be affected by the definition we've just made" [BROWN2011, p. 4]. GAP's built-in enumerator "performs a standard Felsch strategy coset enumeration" [GAP, Section 47.6-5].

Hulpke states which relators a new entry can affect. "We form a list of all cyclic permutations of all relators and their inverses and store these according to the first letter occurring in the permuted relator", and with $R^c_g$ the set of those that start with $g$, "if the image of coset y under generator g is defined as z, we scan all relators in R^c_g starting at the coset y and all relators in R^c_{g^-1} starting at z" [HULPKE2010, Note III.22, pp. 75-76, exponents restored].

The handbook states the Felsch policy as defining $\alpha^x$ for the first undefined entry, the first live $\alpha$ and then the first $x$ in a fixed order of $A$, which is equivalent to defining $1^w$ in the shortlex order of $w$ [HOLT2005, p. 165]. It keeps a deduction stack. DEFINE, SCAN, and COINCIDENCE are modified to push the pair $(\alpha, x)$ "each time the value of a^x is defined or altered", and because the changes come in inverse pairs "we only need save the first alteration of such a pair" [HOLT2005, p. 165, exponents restored]. The handbook gives no pseudocode for the three modified procedures. $R^c$ is the set of the cyclic conjugates of the relators and their inverses after cyclic reduction, and $R^c_x$ holds those that start with $x$. A new entry $\alpha^x = \beta$ is processed by scanning $\alpha$ with each word of $R^c_x$ and $\beta$ with each word of $R^c_{x^{-1}}$ [HOLT2005, p. 165]. Its PROCESSDEDUCTIONS takes the whole family of lists and has these steps [HOLT2005, p. 166]:

1. While the stack is not empty, do the following steps.
2. If the stack is full, run LOOKAHEAD and empty the stack.
3. Otherwise pop $(\alpha, x)$. If $\alpha$ is live, scan $\alpha$ with each word of $R^c_x$, and stop when $\alpha$ dies.
4. Let $\beta = \alpha^x$. If $\beta$ is live, scan $\beta$ with each word of $R^c_{x^{-1}}$, and stop when $\beta$ dies.

The handbook gives the stack no size, and says only that "a fixed amount of space is generally reserved" [HOLT2005, p. 165]. Its COSETENUMERATIONC cyclically reduces the relators, forms the lists, scans and fills each subgroup generator at coset 1, processes the deductions once, and then, for each live coset $\alpha$ and each $x \in A$ with $\alpha^x$ undefined, defines $\alpha^x$ and processes the deductions [HOLT2005, p. 166].

Sims's PROCESS_DEDUCTIONS pops $(\sigma, x)$ and, if $\sigma$ is live, runs a partial two-sided trace at $\sigma$ of each cyclic permutation of a relator that starts with $x$, and stops when $\sigma$ dies [SIMS1994, p. 234]. He writes that "whenever (sigma, x) is pushed on the stack, the pair (sigma^x, x^-1) is usually assumed to be pushed as well, but not actually stored", that duplicates on the stack are allowed, and that "A queue or other structure would also work" [SIMS1994, p. 234, symbols restored]. DEFINE, JOIN, and COINCIDENCE push the entries that they write [SIMS1994, pp. 177 and 234]. He forms the cyclic permutations of the relators only, not of their inverses, because the stack holds both ends of each edge [SIMS1994, p. 234]. His FELSCH defines the next undefined entry in the order $\sigma = 1, 2, \dots$ and then $x \in X$ [SIMS1994, p. 235].

SymPy's Felsch enumeration `coset_enumeration_c` adds a deduction stack, a list of pairs $(\alpha, x)$ for entries assigned and not yet checked [SYMPY-CT, lines 78-80 and 1202-1260, citing p. 166 of [HOLT2005]]. It has these steps:

1. Cyclically reduce each relator, form the cyclic conjugates of it and of its inverse, and group them by first letter into lists $R^c_x$, one for each column $x$.
2. Scan-and-fill each subgroup generator at coset 0, pushing each deduction on the stack. Then, for each column $x$ in order, process the deductions with the lists $R^c_x$ and $R^c_{x^{-1}}$ as arguments, which is $2g$ calls for $g$ generators (lines 1241-1244).
3. For each coset $\alpha$ in increasing order, if $\alpha$ is live, take each column $x$ in order. Stop when $\alpha$ is no longer live. If $\alpha^x$ is undefined, define it, push $(\alpha, x)$, and process the deductions with the lists $R^c_x$ and $R^c_{x^{-1}}$ as arguments (lines 1245-1259).

Processing the deductions takes two lists $L$ and $L'$ from the caller and has these steps [SYMPY-CT, lines 673-704]:

1. While the stack is not empty, do the following steps.
2. If the stack holds `max_stack_size` entries or more, run the lookahead of Section 7.11, empty the stack, and continue. The default of `max_stack_size` is 100 (line 54).
3. Pop $(\alpha, x)$. If $\alpha$ is live, scan $\alpha$ with each word of $L$, and stop when $\alpha$ dies.
4. Let $\beta = \alpha^x$. If $\beta$ is defined and live, scan $\beta$ with each word of $L'$.

The code scans every popped pair with the lists of its caller and does not select the lists by the popped $x$. A pair that a scan or a coincidence pushed with another column is therefore scanned with the lists of the column of the current call. The code also processes the deductions once for each column after the subgroup generators, where the handbook processes them once [HOLT2005, p. 166, line 8]. Both books select the lists by the popped letter, as Hulpke's rule does (Note III.22, above) [HOLT2005, p. 166, lines 7 and 11] [SIMS1994, p. 234]. SymPy's code therefore departs from both books, including the one that it cites. Neither book discusses the variant, so whether it matters is open (Section 14.1, item 23).

Each scan pushes the entry that it deduces, and coincidence processing pushes $(\delta, x^{-1})$ for each entry that it moves [SYMPY-CT, lines 242 and 273]. This agrees with the handbook's prose on the modified procedures [HOLT2005, p. 165], which gives no lines to compare with. SymPy's default of 100 for the size of the stack is the code's own (line 54). Havas reports that implementations expand "each relator out three times to avoid having to check for end of relator during deduction processing" [HAVAS1991, p. 195], and that "Felsch-type programs generally start off by ensuring that each of the given subgroup generators forms a cycle at coset 1" so that "subgroup generators can be ignored thereafter" [HAVAS1991, p. 193].

### 7.13. Coincidence Processing

When two coset numbers name one coset, the larger is eliminated: "If we find Ha_j = Ha_k with k > j, then we replace all k by j in tables which often leads to more coincidences" [HOLT2013, p. 17, exponents restored]. "We thus keep a list of coincidences to process and add to it any such further identifications and process them one by one" [HULPKE2010, p. 76]. ACE also makes "the larger coset number" dead and places the coincidence "in a queue" [ACE, Section 3.2] (summarizing tool).

The handbook's REP finds the representative of a number by following the array $p$ and then compresses the path, and its MERGE of two numbers finds their representatives $\phi$ and $\psi$ and, when they differ, sets $p[\max] = \min$ and appends the larger to the queue [HOLT2005, p. 157]. It keeps the smaller number on purpose: "virtually all implementations of coset enumeration to date have chosen to use the smaller of the two (integer) representatives ... rather than the theoretically more efficient choice of the representative from the larger of the two classes", which would need an array of class sizes [HOLT2005, p. 157]. Sims's MERGE also sets $\mu = \min(\phi, \psi)$ and $p[\max] = \mu$ [SIMS1994, p. 190]. He gives another reason: "It is more difficult to ensure termination of the coset enumeration procedures discussed in Chapter 5 if p[sigma] may be larger than sigma" [SIMS1994, p. 189, symbols restored]. He states the cost of the two rules, at most a constant times $n G(n)$ operations for $n$ unions with the weighted rule, where $G$ is the inverse of the tower function, and $n \log n$ with the rule of the smaller number [SIMS1994, p. 189].

SymPy's `rep` and `merge` follow the handbook [SYMPY-CT, lines 453-534, citing pp. 85-87 and p. 157 of [HOLT2005]]. Its docstring describes the array with "`p[\alpha] \sim \alpha` and `p[\alpha]=\alpha \iff \alpha=rep(\alpha)`" and follows it "until it becomes constant". These two quotations are from pp. 85-86, on the array of the Chapter 4 procedure MINIMALBLOCK, and not from p. 157 [HOLT2005, pp. 85-86]. The docstring also says that "It is more efficient to choose the new representative from the larger of the two classes" (lines 430-431), and the code does not do that. That sentence is the handbook's remark on MINIMALBLOCK, whose MERGE keeps an array of class sizes [HOLT2005, p. 86], while the MERGE of coset enumeration does not.

The handbook's COINCIDENCE for $\alpha \sim \beta$ has these steps, and SymPy's coincidence processing follows them line for line except step 3 [HOLT2005, p. 158] [SYMPY-CT, lines 538-594, citing pp. 156-161 of [HOLT2005]]:

1. Create an empty queue $q$ and merge $\alpha$ and $\beta$.
2. While $q$ is not empty, remove its first coset $\gamma$.
3. Remove $\gamma$ from the live cosets $\Omega$. The steps of SymPy's code that the research recorded have no such step.
4. For each column $x$ with $\delta = \gamma^x$ defined, clear $\delta^{x^{-1}}$, and let $\mu$ and $\nu$ be the representatives of $\gamma$ and $\delta$. Then do the first of these that applies:
   1. If $\mu^x$ is defined, merge $\nu$ and $\mu^x$.
   2. If $\nu^{x^{-1}}$ is defined, merge $\mu$ and $\nu^{x^{-1}}$.
   3. Otherwise set $\mu^x = \nu$ and $\nu^{x^{-1}} = \mu$.

The two merges in step 4 queue the consequent coincidences. Proposition 5.4 proves that afterwards only the least representatives of the classes of the congruence that $\alpha \sim \beta$ generates remain live, that a live entry maps to the representative of its old value, and that Properties 1 to 4 hold [HOLT2005, pp. 158-159]. Sims's COINCIDENCE has the same steps over a queue of dead states with union-find, and it pushes the edges that it adds on the deduction stack when a flag asks for it [SIMS1994, pp. 190-191]. He argues its correctness on pp. 191-192. The Felsch variant also pushes $(\delta, x^{-1})$ on the deduction stack, its "only line of difference" [SYMPY-CT, lines 247-282]. Havas reports that coincidence techniques due to Beetham removed the need for two extra columns, at the cost that "there is no longer easy availability of an active row list" [HAVAS1991, p. 192], and that pending coincidences can be discarded when the table collapses to one coset [HAVAS1991, p. 195].

### 7.14. Compression and Standardization

Coincidences leave gaps in the numbering: "It is customary to remove the dead rows at the end of the process and to renumber the cosets so that there are no gaps. This is often called compression in the literature" [BROWN2011, p. 5]. SymPy's `compress` lists the dead cosets, renumbers the live ones consecutively, truncates the table, and lowers each entry by the number of dead cosets below it [SYMPY-CT, lines 800-832, citing p. 167 of [HOLT2005]]. Its line 824 is a comparison with no effect, as the comment at lines 818-823 states with a link to SymPy issue 27633.

The handbook's COMPRESS works differently [HOLT2005, p. 167]. It counts the live cosets with $g$, and when the live coset $\alpha$ is the $g$-th one and $g \neq \alpha$, it copies each entry $\beta = \alpha^x$ to $g^x$, with $\beta = g$ when $\beta = \alpha$, and repairs the inverse entry $\beta^{x^{-1}} = g$. It then sets $n = g$ and marks $1, \dots, n$ live. The handbook gives no proof or example for COMPRESS. An implementation compresses when it runs out of space, and "If we still have M=n after calling COMPRESS, then we have to give up" [HOLT2005, pp. 170-171]. Chapters 3 to 5 of Sims have no separate compression step, and his STANDARDIZE removes the dead numbers while it standardizes [SIMS1994, pp. 193-195]. His Appendix on implementation says that when no rows are free, "the entire table can be standardized using the procedure of Section 4.7, or compressed as follows", and it prints a procedure COMPRESS with an example [SIMS1994, pp. 576-577]. It also describes the recycling of state numbers. The research did not compare Sims's COMPRESS with the handbook's.

A standardized table numbers its cosets in a fixed order of first appearance. Hulpke defines: "A coset table is standardized if when running through the cosets and within each coset through the generator images (ignoring generator inverses), the cosets appear in order of the integers 1, 2, 3, ...." [HULPKE2010, Definition III.25, p. 78]. Lemma III.26 states: "There is a bijection between subgroups of G of index n and standardized coset tables for G with n cosets" [HULPKE2010, p. 78]. A table is brought into this form "by running through cosets and within cosets through generator images and reassigning new labels according to the order in which cosets appear" [HULPKE2010, p. 78].

GAP calls a table standard when, for $i < j$, "the first occurrence of i in a coset table precedes the first occurrence of j with respect to the usual row-wise ordering of the table entries", following [SIMS1994], and equivalently when the representative words "are length-plus-lexicographic ordered" [GAP, Section 47.7]. GAP's default, "lenlex", uses "all their columns as defined in Charles Sims' book", while "semilenlex" uses "only their generator columns (this was the original GAP standard)" [GAP, Section 47.7-1]. `StandardizeTable` alters the given table and takes the standard as an optional argument [GAP, Section 47.7-2]. GAP does not describe its procedure.

The handbook's STANDARDIZE "reorders the elements of Omega such that, if we scan the coset table first by elements of Omega and then by elements of A, then the cosets occur in ascending numerical order. This is the order in which they would occur with a coset-based enumeration that completed with no coincidences. We shall call the resulting coset table standardized" [HOLT2005, p. 167]. $A$ holds the inverse columns, and the worked table on p. 168 needs a swap because of an entry of an inverse column. The handbook's standard is therefore the lenlex standard, over all columns. Its STANDARDIZE needs a compressed table and has these steps [HOLT2005, pp. 167-168]:

1. Set $g = 2$.
2. For each coset $\alpha$ in increasing order, and each column $x \in A$ in order, let $\beta = \alpha^x$.
3. If $\beta \ge g$, swap the labels $g$ and $\beta$ when $\beta > g$, then increase $g$ by 1. Stop when $g = n$.

The handbook argues that once $1, \dots, n - 1$ are in place, "n must also occur in the correct place" [HOLT2005, p. 168]. The swap SWITCH exchanges the rows of two cosets and their labels in every entry. As printed, its loop runs over $x \in X$, the generators, and not over $A$ [HOLT2005, p. 167]. On a table that stores the inverse columns, that loop would not exchange them, so the research reads it as a misprint for $A$. The research found no erratum.

Sims defines a standard automaton for a fixed reduction ordering of the words: with $W_\sigma$ the least word with $1^{W_\sigma} = \sigma$, the states are $1, \dots, n$, the initial state is 1, and "If sigma < tau then W_sigma precedes W_tau" [SIMS1994, Section 3.8, p. 131, symbols restored]. Proposition 8.1 states that every finite, accessible, deterministic automaton "is isomorphic to a unique automaton which is standard", and its proof leaves the uniqueness as an exercise [SIMS1994, pp. 131-132]. For the length-plus-lexicographic ordering, Proposition 8.3 states that standard is the same as edge standard, in which "the first occurrence of tau as a third component comes before the first occurrence of tau + 1" when the edges are read row by row [SIMS1994, pp. 133-134, symbols restored]. The alphabet is ordered with the inverse letters, as $x < x^{-1} < y < y^{-1}$ [SIMS1994, pp. 195 and 204], so this is the lenlex standard over all columns that GAP follows. Proposition 8.7 proves that, for this ordering, one automaton precedes another exactly when its transition matrix comes first in the lexicographic order of the entries read row by row, with an undefined entry 0 before every state [SIMS1994, p. 136]. LENLEX_STND standardizes by a breadth-first search over the edges, and Proposition 8.6 states that it returns the unique standard automaton isomorphic to its input, with the proof left as an exercise [SIMS1994, pp. 135-136].

Sims's STANDARDIZE is one pass in place [SIMS1994, p. 195]:

1. Set $\sigma = 1$ and $\tau = 1$.
2. While $\sigma \le \tau$, for each $x \in X$ in order with $\sigma^x$ defined, let $\rho = \sigma^x$. If $\rho > \tau$, increase $\tau$ by 1, and if then $\rho > \tau$, swap $\tau$ and $\rho$ in every edge.
3. Increase $\sigma$ by 1.

It makes at most $|E| - 1$ swaps, each costing about $|X|$ [SIMS1994, p. 196]. The standardizing strategies of Section 5.4 swap during the enumeration, and the table that they produce is in general not completely standard, so a final pass is needed [SIMS1994, p. 244].

SymPy's `standardize` has these steps, after `compress` [SYMPY-CT, lines 752-797, citing pp. 167-168 of [HOLT2005]]:

1. Set $\gamma = 1$.
2. For each coset $\alpha$ in increasing order, and each column $x$ in the order $g_1, g_1^{-1}, g_2, g_2^{-1}, \dots$, let $\beta = \alpha^x$.
3. If $\beta \ge \gamma$, swap the labels $\gamma$ and $\beta$ in every row and column when $\beta > \gamma$, then increase $\gamma$ by 1, and stop when $\gamma$ equals the number of cosets.

These are the handbook's steps numbered from 0. The swap is `switch`, which exchanges two rows and the two labels in every entry [SYMPY-CT, lines 728-750]. The code visits every column, as the handbook and Sims do, which is GAP's lenlex standard. The docstring instead defines the standard "ignoring generator inverses", which is semilenlex and is neither book's definition. The doctest of Section 7.20 gives a lenlex table, and its rows before and after `standardize` are the handbook's rows of Example 5.3 numbered from 0 [SYMPY-CT, lines 776-784] [HOLT2005, pp. 157 and 168]. Hulpke's Lemma III.26 is stated for his definition, which ignores inverses. The handbook's Proposition 5.6 and a sentence of Sims state the same bijection for the lenlex standard (Section 7.17 and Section 14.1, item 26).

### 7.15. Termination and Correctness

Holt states what the procedure does: it "tries to prove that |G : H| is finite and to compute the coset action of G on the cosets of H. If |G : H| is infinite it will not halt. If |G : H| is finite it will succeed but there is no upper bound on the time taken" [HOLT2013, p. 15]. When it stops, Theorem 5.2 gives "an action theta of G on Omega" that "is equivalent to the coset action of G on right cosets" of $H$, with $i \mapsto Ha_i$ [HOLT2013, Theorem 5.2, p. 16]. The proof starts "When the process stops all ix are defined."

The handbook states the aim in the same way: "Its aim is to attempt to verify that |G:H| is finite. It will terminate only when |G:H| is finite, and so it cannot correctly be described as an algorithm", and "if the procedure does terminate, then it will have verified that |G:H| is finite" [HOLT2005, p. 149]. Its Theorem 5.2 proves that if Properties 1 to 4 hold, the table is complete, coset 1 scans correctly under every subgroup generator, and every live coset scans correctly under every relator, then $|G : H| = |\Omega|$ and the table gives "the action of G on the cosets of H by right multiplication" [HOLT2005, pp. 154-155]. Sims's Proposition 1.2 proves that if a coset enumeration procedure terminates, its table describes the subgroup $H$ [SIMS1994, p. 223].

Termination for finite index needs a condition on the strategy. Holt states "(R4) for all i in Omega, a in A, ia will eventually be defined" and "Theorem 5.3. If |G : H| is finite and definition strategy satisfies (R4) then it will eventually finish" [HOLT2013, p. 18, exponents restored]. He adds that (R4) "is satisfied with 1 but not necessarily with 2 so we must occasionally use 1", where 1 is the Felsch strategy and 2 the relator-based one [HOLT2013, p. 18]. Hulpke attributes the result to Mendelsohn:

> Theorem III.24 (Mendelsohn, 1964): Suppose that [G:S] < infinity and that the strategy used guarantees that for every defined coset a and every generator g the images a^g and a^(g^-1) will be defined after finitely many steps, then the coset enumeration terminates after finitely (but not bounded!) many steps with the correct index. [HULPKE2010, pp. 77-78, exponents restored]

Hulpke gives only a proof idea and no bibliography entry for Mendelsohn's paper [MENDELSOHN1964], which was not read. The ACE manual names the condition "Mendelsohn's condition" and locates it in Lemma 1 and Theorem 2 of [NEUBUSER1982] [ACE, Chapter 3] (summarizing tool). The ITC manual states that "The Felsch strategy automatically fulfills the "Mendelsohn Condition"", while "by a pure HLT strategy the Mendelsohn Condition is not guaranteed. The CE may run into infinite loops although the index of h in g is finite" [ITC, Section 3.2] (summarizing tool).

The handbook numbers these statements differently from the notes. The notes' (R4) is its Property 5, with three parts, and the notes' Theorem 5.3 is its Theorem 5.5 [HOLT2005, p. 161]. With $\bar{\Omega}$ the set of cosets "not eliminated in any call of COINCIDENCE", Property 5 reads [HOLT2005, p. 161, symbols restored]:

1. "For each alpha in Omega-bar and each x in A, alpha^x will at some stage be defined".
2. "For each w in Y, SCAN(C, 1, w) is called at some stage".
3. "For each alpha in Omega-bar and each w in R, SCAN(C, alpha, w) is called at some stage".

Theorem 5.5 states: "Assume that Property 1 to Property 5 hold throughout a coset enumeration of G with respect to the subgroup H, and that |G:H| is finite. Then the coset enumeration procedure will eventually terminate." The handbook proves it in ten lines through an infinite coset table, and writes that "The ingenious idea of using an infinite coset table in the proof of the proposition below is taken from Theorem 2 of [Neu82]" [HOLT2005, p. 161]. The theorem does not mention the limit $M$, at which DEFINE aborts [HOLT2005, p. 153], so it applies to an unbounded table. The handbook leaves this implicit. Ward "presents a simple example in which |G:H| is finite, but in which certain apparently reasonable strategies for choosing definitions result in the enumeration never completing" [HOLT2005, p. 160].

The handbook states that its HLT procedure meets the condition: "It is clear that the conditions of Property 5 hold for this function, and so the results of the last section ensure that it will perform as required. Note that the lines 9,10 ensure that Property 5(i) holds. This line has not always been included in past implementations, with the consequence that the enumeration would not terminate even when |G:H| was finite on certain examples, such as those described by Ward" [HOLT2005, p. 164]. Its Exercise 3 removes lines 9 and 10 and asks why the procedure then never terminates on Ward's group $\langle x, y \mid x^3yxyx^{-3}, y^3xyxy^{-3} \rangle$ of order 3, and Exercise 4 asks for an example that fails even when the relators are first replaced by cyclically reduced conjugates [HOLT2005, p. 173]. The handbook verifies Property 5 for its Felsch procedure [HOLT2005, p. 166], and makes no statement on termination with lookahead.

Sims proves his own form of the result [SIMS1994, pp. 223-224]. A coset enumeration procedure starts with coset 1, traces each subgroup generator at coset 1, and then defines entries, traces relators, and stops when the table is complete and compatible with the relators. "The sequence of operations is subject to two conditions: First, if termination is possible, then it must occur. Second, for every integer sigma which is ever in Sigma, one of the following holds: (a) At some time p[sigma] becomes less than sigma, so sigma is deleted from Sigma. (b) Eventually sigma^x is defined for all x in X, and sigma^V = sigma for all V in V" [SIMS1994, p. 223, symbols restored]. Proposition 1.3 states: "If H has finite index in F, then `CE(X, R, U, V; A)` terminates" [SIMS1994, p. 223]. Its proof by contradiction uses Lemmas 1.4 and 1.5 [SIMS1994, p. 224]. Sims writes of his HLT that it "clearly satisfies the definition of a coset enumeration procedure. Thus Propositions 1.3 and 1.4 apply to HLT" [SIMS1994, pp. 227-228]. The research reads "1.3 and 1.4" as a misprint or a loose reference, because the results that apply are Propositions 1.2 and 1.3 and Lemma 1.4. His FELSCH carries the comment "A coset enumeration procedure" with no proof that the conditions hold [SIMS1994, pp. 232-235]. He also states that the small-gap variant of the Felsch strategy, "Followed strictly ... does not define a coset enumeration procedure in the sense of Section 5.1" [SIMS1994, p. 239].

Neither book names Mendelsohn in its account of coset enumeration. The handbook does not contain the name at all, and Sims lists Mendelsohn's papers of 1964 and 1970 in his bibliography and cites only the paper of 1970, on another topic, in the historical notes of Chapter 6 [SIMS1994, pp. 295 and 591].

Havas warns that preferred definitions made carelessly "could lead to violation of the conditions required for guaranteed termination", that implementations therefore fill "at least a certain initial fraction of the coset table (in order row by row)" first, and that "It is easy to construct examples where omission of a control of this type does indeed lead to infinite looping" [HAVAS1991, p. 194]. SymPy's HLT loop, which defines every open entry of a row after its scans (Section 7.11), has the shape of the handbook's COSETENUMERATIONR with its lines 9 and 10. Whether the code meets the condition is a question about the code, which no source answers (Section 14.1, item 20).

No computable bound exists on the work. "For a given coset enumeration procedure, there is no computable bound, in terms of length of input and a hypothetical index, to the number of cosets which need to be defined in the coset enumeration process to complete the enumeration. (The existence of such a bound would violate the unsolvability of the word problem for finitely presented groups.)" [CANNON-HAVAS1992, Section 2.2, p. 3]. Havas states the same, and that "Sims [15] has proved that there is no polynomial bound in terms of maximum number of cosets for the number of coset tables which can derived by simple coset table operations" [HAVAS1991, p. 192]. Neither paper proves the statement or cites a proof beyond the parenthesis. The handbook states that it is "not hard to construct presentations, even of the trivial group, in which the maximum value of |Omega| is arbitrarily large", which it calls "an inevitable consequence of the fact that deciding whether |G:H| is finite is an unsolvable problem in general" [HOLT2005, p. 162, symbols restored]. It gives no proof. Sims writes that "there is no algorithmic solution to this problem" of deciding whether $H$ has finite index [SIMS1994, p. 218]. On the running time as a function of memory, "We do not have a good answer to this question. However, there is some evidence that the worst-case running time may not be bounded by a polynomial function of memory size" [SIMS1994, p. 224]. His Proposition 1.6 states a lower bound of that kind for free groups and leaves the proof to the reader [SIMS1994, p. 226].

GAP states that "A coset enumeration will not finish if the subgroup does not have finite index, and even if it has it may take many more intermediate cosets than the actual index of the subgroup is" [GAP, Section 47.6-6]. Cannon and Havas measure performance "by the maximum number of cosets that are simultaneously defined and by the total number of cosets defined during the course of the enumeration" [CANNON-HAVAS1992, p. 4]. If the enumeration terminates for a subgroup $H$ known to be finite, "we immediately deduce that G is finite and, moreover, we obtain a bound on the order of G" [CANNON-HAVAS1992, p. 4].

The handbook states that a run that reaches a limit on the number of cosets shows nothing about the index. It writes: "When the index is infinite, it will run forever in theory, but in practice it will usually run out of space. Of course it may also run out of space when |G:H| is finite but very large, or even when |G:H| is small, but the presentation is a difficult one. So, we learn nothing when this happens" [HOLT2005, p. 149]. Sims does not state this. He writes that "When that memory is filled, the program must stop without producing a result" [SIMS1994, p. 224], which says that such a run gives no result, and he names infinite index and exhausted memory as two reasons that an enumeration does not terminate: "But suppose an enumeration does not terminate, either because the subgroup has infinite index or memory is exhausted before the index can be determined" [SIMS1994, p. 247]. Neither book proves a statement about such a run. Sims's Exercise 5.3 poses, without an answer, a related question on the incomplete table that a run limited by memory returns [SIMS1994, p. 251]. ACE's manual shows a finite-index enumeration that overflows at a small limit and succeeds at a larger one [ACE, Appendix B.4] (summarizing tool). This document therefore states, on the handbook's authority, that reaching a limit shows nothing about the index, and in particular not that the index is infinite (Section 14.1, item 19).

### 7.16. Limits and Defaults of Implementations

This table gives the limit on the number of cosets of each implementation and what it does on reaching it.

| Implementation | Strategy | Limit and default | On reaching the limit |
| --- | --- | --- | --- |
| GAP | Felsch by default [GAP, Section 47.6-5] | Option `max`, "The limit of the number of cosets to be defined", default `CosetTableDefaultMaxLimit` = 4096000. "(Due to the algorithm the actual limit used can be a bit higher than the number given.)" `CosetTableDefaultLimit` = 1000 is the initial allocation [GAP, Sections 47.6-5 to 47.6-7] | "an error is raised and the user is asked whether she wants to continue", with an offer of a new limit of 8192000. With `silent`, it returns `fail`. A limit of 0 or infinity runs until "they either get a result or exhaust the whole available space" [GAP, Sections 47.6-5 and 47.6-6] |
| SymPy | HLT in `coset_enumeration_r`, Felsch in `coset_enumeration_c` | `coset_table_max_limit` = 4096000, "default limit for the number of cosets allowed in a coset enumeration", overridden by the keyword `max_cosets` [SYMPY-CT, lines 47-49] | `define` raises `ValueError` with the text "the coset enumeration has defined more than %s cosets". With `incomplete=True`, the enumeration returns the partial table [SYMPY-CT, lines 152-156, 1160-1163, and 1255-1258] |
| ACE | Several, including HLT and Felsch [ACE, Chapter 5] | `max` "Sets the maximum coset number that can be defined", 0 or at least 2, and by default "all of the workspace is used" [ACE, Section 4.17] | `OVERFLOW`, "out of table space" [ACE, Appendix A.2] |
| Magma | Not recorded | Not recorded | The index returned is "0, if the enumeration fails to complete", or an incomplete coset space [MAGMA] |

The rows for ACE and Magma were read through a summarizing tool. Neither book gives a default limit. The handbook's DEFINE aborts when $n = M$ [HOLT2005, p. 153]. An implementation may then "give up, or we might attempt to reuse any space available from cosets that have been eliminated" [HOLT2005, p. 163], by lookahead in HLT [HOLT2005, p. 164] or by COMPRESS, after which it gives up if $n = M$ still holds [HOLT2005, pp. 170-171]. Sims abandons a lookahead that releases too little space (Section 7.11), and his variant of Section 5.7 with a limit $M$ on the live states "halts, reports its failure, and returns the current coset table" when it needs more [SIMS1994, p. 261]. SymPy compares the limit with the length of the table, which counts every row ever defined, dead rows included, until compression. That reading is the research's own, from the code. The handbook's limit counts the same way, because its $n$ is the largest number used so far and DEFINE always takes $n + 1$ [HOLT2005, pp. 151 and 167]. SymPy's documentation gives 500 as the default of `max_stack_size`, while the code sets 100 [SYMPY] (summarizing tool) [SYMPY-CT, line 54].

### 7.17. The Low-Index Subgroup Search

The search runs through a tree of partial coset tables. SymPy describes it as "a backtrack search over incomplete Coset Tables, rather than over forced coincidences", citing [SIMS1994] [SYMPY-FPG, lines 737-738]. In Hulpke's account, a node is "a partially completed, standardized coset table, involving k <= n cosets. (The initialization is with the empty coset table.) If the table is in fact complete, it yields a subgroup" [HULPKE2010, p. 79]. Culler, Dunfield, and Gorner state the tree in terms of covering spaces: "The nodes of the Sims tree are covering subgraphs of B and the root of the tree is B, viewed as a degree 1 covering of itself", and a terminal node is kept when "the base vertex is canonical and the relators all lift to all vertices" [LOWINDEX, Section 4]. Their note does not state how a node is extended.

The handbook describes Sims's method, a search "over incomplete coset tables rather than over forced coincidences", as "Section 5.6 (plus some earlier sections) of [Sim94]" describes it, and credits the first practical methods, with forced coincidences, to Dietze and Schaps and to Sims. It refers the reader to [NEUBUSER1982] for "an accessible description" of the forced coincidence approach [HOLT2005, p. 189]. Its time grows "more than exponential in N" [HOLT2005, p. 188]. Sims notes that "this straightforward approach is not efficient enough to be of much use. To get a practical procedure, we need to incorporate ideas from coset enumeration, such as the deduction stack and two-sided traces" [SIMS1994, p. 253].

Hulpke's Algorithm III.27, "a basic version of the low index algorithm without elimination of conjugates", takes $G$ and an index $n$ and outputs "All subgroups of G of index up to n, given by coset tables" [HULPKE2010, pp. 79-80]. Its table starts empty, and the research reads the first branch as adding coset 1, which the algorithm does not state. The handbook starts from "the empty coset table with a single coset 1, and no transitions defined" [HOLT2005, pp. 190-191], and Sims from a matrix of one row of undefined entries [SIMS1994, pp. 253-254]. SymPy's root table likewise holds one row of undefined entries, coset 0 [SYMPY-FPG, line 774] [SYMPY-CT, lines 63-70]. Because no coset is ever eliminated, the live cosets of a node are always $1, \dots, n_C$ for some $n_C \le N$ [HOLT2005, p. 190].

The handbook's procedures are LOWINDEXSUBGROUPS, DESCENDANTSUBGROUPS, TRYDESCENDANT, and FIRSTINCLASS [HOLT2005, pp. 191-194], and Sims's are LOW_INDEX and FIRST_CONJ [SIMS1994, pp. 208-209 and 253-254]. These steps restate the extension of a node, from [HULPKE2010, pp. 79-80], [HOLT2005, p. 192], [SIMS1994, pp. 253-254], and [SYMPY-FPG, lines 790-838]:

1. If the table is complete, check the long relators at every live coset. Output the table if they all hold. The handbook's DESCENDANTSUBGROUPS does this in its lines 2 to 5 [HOLT2005, p. 192], and Sims checks the long relators "as a filter on the output" [SIMS1994, p. 253].
2. Otherwise let $x$ and $g$ be the coset and the column of the first undefined entry, by rows and within a row by columns.
3. For each existing coset $y$ in increasing order with $y^{g^{-1}}$ undefined, $y = x$ included, and then a new coset $k + 1$ if the table has $k < n$ cosets, copy the table, set $x^g = y$ and $y^{g^{-1}} = x$, and push $(x, g)$ on an empty deduction stack [HOLT2005, p. 192] [SIMS1994, p. 254].
4. Process the deductions with the short relators. If a coincidence or a contradiction arises, abandon the copy. Hulpke: "if a coincidence occurs, we know that we made an invalid choice, and abandon this partial table, backtracking" [HULPKE2010, p. 79]. The handbook's PROCESSDEDUCTIONSCHECK abandons the copy when a scan "completes incorrectly" [HOLT2005, pp. 191-192], and Sims abandons it on any coincidence [SIMS1994, p. 254].
5. Check that each subgroup generator fixes the first coset. The handbook gives this as a variant with a line inserted in TRYDESCENDANT [HOLT2005, p. 196], and Sims runs partial two-sided traces of the subgroup generators at state 1 [SIMS1994, p. 254]. SymPy checks the words of its argument `Y` at coset 0.
6. If the copy passes the first-in-class test, recurse on it.

SymPy's deductions in the search use `scan_check`, which returns false when a scan completes incorrectly, without coincidence processing, and writes the deduction of a single gap [SYMPY-CT, lines 382-424]. The deduced entry is not pushed on the deduction stack, which the research observed in the code. The handbook's SCANCHECK "returns false (rather than calling COINCIDENCE) if the scan completes incorrectly" and is "a straightforward modification of SCAN", but the handbook does not display it, and it does not say whether the entries that it deduces are pushed [HOLT2005, p. 191]. Sims's LOW_INDEX pushes the entries that it defines, and for the partial two-sided traces of the subgroup generators it states that they add "any deductions discovered to the deduction stack" [SIMS1994, p. 254]. For the traces of the short relators it says only "Process deductions as in the Felsch strategy" and "If further deductions are found then make the corresponding entries in S" [SIMS1994, p. 254], and JOIN pushes the entry that it writes in the Felsch strategy [SIMS1994, pp. 177 and 234]. That the search pushes the deductions of the short-relator traces is therefore the research's inference. No source says whether SymPy's choice affects the result (Section 14.1, item 23).

SymPy splits the relators by length, with `len_short_rel = 5`. The relators of length at most 5 prune "branches of the search tree", and the longer ones are "only checked at the last step for complete coset tables" [SYMPY-FPG, lines 776-784]. Hulpke states the reason and no threshold: "as long relators rarely yield a deduction but only are conditions to test, it can make sense to only consider the shorter (whatever this means) relators for the determination of coset tables and simply test each table obtained afterwards for the remaining relators" [HULPKE2010, Performance III.28, p. 81]. The handbook follows Sims in splitting the relators into short ones $R_1$ and long ones $R_2$, and writes that "Unfortunately there appear to be currently no reliable heuristics for deciding exactly how short is short!" [HOLT2005, p. 190]. Sims writes that "The output from LOW_INDEX depends only on V1 U V2. However, the running time will depend on the division" [SIMS1994, p. 253]. The package `low_index` takes the two sets of relators from its caller [LOWINDEX, README], and Magma states that "A relator designated as long is not used during the construction of a coset table" [MAGMA] (both through a summarizing tool).

The handbook states that the leaf tables already satisfy the short relators: "as will be explained in detail later, we shall know already that alpha scans correctly under all w in R1" [HOLT2005, p. 191, symbols restored]. It gives no argument beyond the use of the deduction stack, and no proof that every scan of a short relator at every coset has run by the time a table is complete.

The first-in-class test keeps one table from each conjugacy class. Hulpke defines "a (somehow arbitrary) order on coset tables which tests conveniently for partial coset tables", which is "lexicographic, considering the table row by row", and tests "for each (partial) table" "whether it can be the smallest in its class" [HULPKE2010, pp. 80-81]. A conjugate subgroup is the stabilizer of another coset $x$, and its table arises by relabelling with $x$ as coset 1 [HULPKE2010, pp. 80-81]. The handbook states that "the conjugates of H are just the stabilizers G_alpha of the points alpha in Omega" [HOLT2005, p. 190, symbols restored]. It orders the tables lexicographically by their transition matrices, where "we will think of undefined entries alpha^x_C as being equal to 0", and keeps the least complete table of each class, which is "necessarily standardized" [HOLT2005, p. 193, symbols restored]. Its FIRSTINCLASS takes its code from Sims: "The code below is taken directly from the function on page 208 of [Sim94]" [HOLT2005, p. 193]. Sims's FIRST_CONJ "assumes that < is a length-plus-lexicographic ordering" and returns "false if the test shows that A cannot be extended to a complete automaton describing a subgroup first in its conjugacy [class]. Otherwise, FIRST_CONJ returns true" [SIMS1994, pp. 208-209]. SymPy's `first_in_class`, which "is taken directly from the function on page 208 of [Sim94]", has these steps [SYMPY-FPG, lines 841-954]. FIRSTINCLASS and FIRST_CONJ have steps 2 to 4 [HOLT2005, p. 194] [SIMS1994, pp. 208-209], and each book gives step 1 as a variant for a subgroup generating set [HOLT2005, p. 196] [SIMS1994, p. 253]:

1. For each coset $\alpha$ other than coset 0, skip $\alpha$ unless $\alpha^w = \alpha$ for every subgroup generator $w$ of `Y`.
2. Relabel the cosets by first appearance, starting with $\alpha$ as coset 0, while reading the table row by row and column by column.
3. At each entry, compare the relabelled entry of the row that corresponds to the current row with the entry $\gamma$ of the original table. If either is undefined, go to the next $\alpha$. If the relabelled entry is smaller, return false. If it is larger, go to the next $\alpha$. If they are equal, continue.
4. Return true when every $\alpha$ is done.

The docstring states that if the test "returns False, then no descendant of C can have that property, and so we can abandon C" [SYMPY-FPG, lines 841-859]. Culler, Dunfield, and Gorner define a canonical vertex as one with "minimal complexity among all vertices", where the complexity is a tuple ordered lexicographically, and extend it to partial graphs "by defining V_s to be 0 if the slot s is empty" [LOWINDEX, Section 3.4]. Their rule thus compares empty slots as 0, while SymPy stops at an undefined entry.

Both books state that the pruning is safe and argue it without a proof. The handbook writes that "If FIRSTINCLASS returns false, then no descendant of D can have that property, and so we can abandon D", that "This test will be conclusive if D is complete" [HOLT2005, p. 192], and that "If either of these two entries are undefined, then we cannot draw any conclusions, so we move onto the next alpha" [HOLT2005, p. 193, symbols restored]. Sims states the condition: "Let W be the first word at which f_A and f_Ai differ. Suppose that f_Ai(W) and f_A(W) are both defined, that f_Ai(W) < f_A(W), and that 1^V is determined in both A and A_i for all words V preceding W", with the conclusion that "no descendant of T can correspond to a subgroup of finite index in F which is first in its conjugacy class" [SIMS1994, p. 208, symbols restored]. Here $A_i$ is the automaton of the partial table with $i$ as the base state, standardized. He justifies it only through Example 9.3 (Section 7.20) [SIMS1994, pp. 207-208]. Both books stop at an undefined entry, as SymPy and SymmHub do, although the handbook's order counts an undefined entry as 0 [HOLT2005, pp. 193-194, line 8] [SIMS1994, pp. 208-209]. Neither book discusses the rule of `low_index`, and no source read proves that pruning a partial table by either rule loses no class (Section 14.1, item 22).

Each book also names a saving that it states without proof. The handbook notes that the exit for a larger entry in its line 14 "will happen for that value of alpha in any descendant of the table C", so such base points could be passed down, and does not use this [HOLT2005, p. 193, symbols restored]. Sims writes that a base point that passed the test in a node need not be tested again in a child [SIMS1994, p. 257].

GAP's search takes a subgroup $H$ and returns only subgroups that contain it, and with `excluded` only subgroups that "do not contain any conjugate" of the given elements [GAP, Section 47.10-1]. GAP does not say how it uses either argument during the search. SymPy's `first_in_class` skips a base point unless the conjugate "definitely contains the given subgroup" [SYMPY-FPG, lines 911-924], and a comment there says that it supports only subgroup generators that are single generators or their inverses [SYMPY-FPG, lines 916-917]. The search as a whole scans any word of `Y` at coset 0 (lines 834-836). The handbook's variant for subgroups that contain $\langle Y \rangle$ inserts two lines, one that scans each $w \in Y$ at coset 1 in TRYDESCENDANT and one that skips a base point $\alpha$ with $\alpha^w \neq \alpha$ in FIRSTINCLASS [HOLT2005, p. 196]. For subgroups that avoid given elements, "There seems to be no easy way of incorporating such conditions into the body of the search", so such tests run on the complete tables in DESCENDANTSUBGROUPS [HOLT2005, p. 196]. Sims requires $1^U = 1$ for every subgroup generator $U$ and tests a base point $i$ only if $i^U = i$ for all of them, with no restriction to single generators [SIMS1994, pp. 252-253]. Hulpke's algorithm has no subgroup argument. The `low_index` note takes a covering subgraph $C$ as an input, which is a starting node of the search and not a subgroup [LOWINDEX, Section 4].

The result is one table or one subgroup for each conjugacy class:

- GAP returns "the list of these subgroups", and "By default, the algorithm computes no generating sets for the subgroups" [GAP, Section 47.10-1].
- SymPy returns a list of complete coset tables, one for each class, in the order of the depth-first search [SYMPY-FPG, lines 751-787].
- The `low_index` note states that "each conjugacy class of subgroups of G with index at most N contains a unique representative", and that its implementation "actually outputs a list of permutation representations" with the canonical vertex labelled 1 [LOWINDEX, Section 4].
- Cannon and Havas: the algorithm "outputs either a generating set for each subgroup of index n, or a list of all transitive permutation representations of G having degree n" [CANNON-HAVAS1992, p. 5].

The handbook states the correspondence behind the output as Proposition 5.6: "For any n>0, there is a one-one correspondence between standardized complete coset tables for G with |Omega|=n and subgroups of G of index n, in which the subgroup H corresponding to a coset table C is the stabilizer G_1 in the group action corresponding to C" [HOLT2005, p. 189, symbols restored]. Its justification is the prose before it: a complete table is a transitive action (Theorem 2.52), the action is equivalent to the action on the cosets of $G_1$ (Proposition 2.23), and a standardized table is a canonical representative of the actions that are equivalent by an equivalence that fixes 1 [HOLT2005, pp. 22, 37, and 189]. It gives no further proof. Sims states the correspondence for $G = F/N$ in one sentence: "subgroups of finite index in G are in one-to-one correspondence with the finite, standard, complete coset automata with respect to R which are compatible with V" [SIMS1994, p. 252], with no proof or proposition number. The results that support it, with their proofs or with the proofs left as exercises, are Propositions 1.1, 2.1, 4.9, 4.10, 8.1, and 8.3 [SIMS1994, pp. 131-134, 160, 173-174, and 218]. Listing the standard complete tables with at most $n$ rows "is equivalent to finding all subgroups of F with index at most n", and listing those first in their class "is equivalent to finding representatives for the conjugacy classes of such subgroups" [SIMS1994, pp. 203-204], both stated without proof. Sims states the search problem as listing the standard complete tables with at most $n$ states that are compatible with the relators, fix state 1 under the subgroup generators, and precede or equal the standardized table from every base point that the generators fix. He gives no proof that this lists one table for each class [SIMS1994, p. 252].

No mathematical source read states a rule for the number of subgroups in a class. The handbook states that the tables $C_\alpha$ from the base points $\alpha$ are the standardized tables of the conjugates of $H$, and does not say that the number of distinct ones is the size of the class [HOLT2005, p. 193]. SymmHub's code computes it as the number of distinct standardizations of the table over all base points [SYMMHUB, `sublib_core.js`, lines 373-384]. Hulpke gives no algorithm for normal subgroups and suggests computing quotients instead [HULPKE2010, Note III.29, p. 81].

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

Step 2 recognizes a tree edge by the next new number, as the handbook's DEFINESCHREIERGENERATORS does. The handbook runs it on a standardized complete table, and the entry $P[\alpha, x]$ is empty when $\alpha^x$ is the next new coset, and otherwise a new Schreier generator $y_{\alpha, x}$ for $x \in X$ or its inverse for $x \in X^{-1}$ [HOLT2005, p. 175]. It calls the standardized table "not strictly necessary", but says that it simplifies the code [HOLT2005, p. 173]. For a relator of the form $w^r$ with $r > 1$, the rewritten relators at the cosets are cyclic conjugates of one another [HOLT2005, p. 177]. Sims counts "1 + 9(2 - 1) = 10" Schreier generators for a subgroup of index 9 of a group with two generators [SIMS1994, p. 261]. GAP's Section 47.9 names the functions `AugmentedCosetTableMtc`, `AugmentedCosetTableRrs`, and `RewriteWord` without the steps, and the augmented coset table of its Chapter 48 was not read [GAP, Section 47.9].

### 7.19. SymmHub's Implementation

SymmHub implements the low-index search and Reidemeister-Schreier rewriting in `lib/sublib/src/sublib_core.js` and `lib/sublib/src/reidemeister.js` [SYMMHUB]. Section 12.8 of `docs/research/symmhub.md` gives the data shapes, the functions, the limits, and the defects of this code. The code reading found no Todd-Coxeter enumeration in `lib/sublib`, and records the search that supports this in the same section. The header comment calls the code a "Sims-style low-index coset-table backtrack search" (`sublib_core.js`, line 15).

The search of the class `LowIndexSearch` has these steps [SYMMHUB, `sublib_core.js`, lines 199-331]:

1. Cyclically reduce each relator, drop empty ones, and keep them in input order. There is no split into short and long relators.
2. Start from a table with one coset, coset 0, and no entry defined.
3. At a node, find the first undefined entry in row-major order, with the columns in the order $g_1, g_1^{-1}, g_2, \dots$. If there is none, test canonicity again and output a copy of the table.
4. Try each existing coset $d$ in increasing order whose entry in the inverse column is undefined, including the row's own coset, and then a new coset if the table has fewer than `maxIndex` cosets.
5. For each candidate, define the entry and its inverse, propagate, test canonicity, recurse if both pass, and undo the definitions through a trail of changed entries.

Propagation scans each relator from every coset in turn. When the pass of a relator changed an entry, it restarts from the first relator, and it ends after a round that changes nothing. A scan that traces a whole relator fails unless it returns to its coset, and a scan with one gap defines that entry. A definition that clashes with an existing entry fails [SYMMHUB, `sublib_core.js`, lines 235-280]. The canonicity test `isCanonical` follows the same rule as SymPy's `first_in_class` (Section 7.17): it renumbers from each other base point, returns false when the renumbered table is smaller, and leaves a base point undecided at an undefined entry [SYMMHUB, `sublib_core.js`, lines 351-371]. The comment above it states that on a complete table the test is exactly "this table is the least in its class", and that on a partial table "a prefix that already loses still loses however it is completed, so cutting the branch there cannot discard a canonical table" [SYMMHUB, `sublib_core.js`, lines 341-346]. These are the code's own claims, and the research did not check them. Both books state the same claim for the same rule without a proof (Section 7.17 and Section 14.1, item 22).

This table compares the search with the sources of Section 7.17.

| Aspect | SymmHub | Sources |
| --- | --- | --- |
| Todd-Coxeter enumeration | None | GAP, SymPy, ACE (Section 7.16) |
| Coincidences | None. A clash ends the branch | A coincidence ends the branch [HULPKE2010, p. 80] [SIMS1994, p. 254], and the handbook never eliminates a coset in the search [HOLT2005, p. 190] |
| Deductions after a definition | Full rescan of all relators at all cosets | A deduction stack over the cyclic conjugates by first letter (Section 7.12) [HOLT2005, p. 192] [SIMS1994, p. 254] |
| Short and long relators | No split | SymPy splits at length 5. Hulpke writes that "it can make sense to" split. Both books split with no threshold [HOLT2005, p. 190] [SIMS1994, p. 253] |
| Subgroup argument | None | GAP `H`, SymPy `Y`, the handbook's variant with $Y$ [HOLT2005, p. 196], and Sims's $U$ [SIMS1994, p. 252] |
| Canonicity on partial tables | Stops at an undefined entry | SymPy and both books stop there [HOLT2005, p. 194] [SIMS1994, p. 209]. `low_index` compares empty slots as 0 |
| Index bound | `maxIndex`, at most 62 through `subgroupsData` | GAP `index`, SymPy `N` |

The comment at `sublib_core.js`, lines 21-27, states that the conventions are "identical to GAP's": rows are cosets numbered from 0, the columns are $g_1, g_1^{-1}, g_2, \dots$, and cosets are numbered in order of first appearance over all columns. That is the definition of the handbook and of Sims (Section 7.14). The code does not check that the tables of the search are standardized in that sense (Section 14.1, item 24). The comment at lines 31-33 states that the representative of a class "agrees with GAP's representative for ~99% of subgroups", and that where it differs the two tables are conjugate. The section `Compatibility with the GAP-generated files` of `lib/sublib/README.md` states instead that "Coset permutations agree verbatim for 97% of the corpus' 110 524 subgroups", with 47 739 of 50 986 in the family `klm` and all in `wallpaper` and `*klm` (lines 310-312). Both figures are claims of the project. The research ran no comparison with GAP and read the README for this passage only.

### 7.20. Worked Examples

These examples state their numbers in the source and can serve as tests. Tables are written as their sources print them, and Section 7.21 gives the conventions. Section 14.1, item 25, lists what remains of the group orders that no source states.

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
| [HOLT2013, pp. 22-23] | $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$ | $\langle xy, x^{-1}y^{-1}xyx \rangle$ | $H \cong D(2,3,3)$ of order 12, $\lvert G \rvert = 60$. The table has 5 rows, and the notes do not state the index in words. The handbook states index 5 for the same presentation and subgroup (Example 5.6, below) |
| [HOLT2005, Example 5.1, pp. 150-151] | $\langle x, y \mid x^3, y^3, x^{-1}y^{-1}xy \rangle$ | $\langle x \rangle$ | Index 3. $G$ is "a direct product of two cyclic groups of order 3", so its order is 9. Rows `1 1 2 3`, `2 2 3 1`, `3 3 1 2` with the columns $x, x^{-1}, y, y^{-1}$ |
| [HOLT2005, Example 5.2, pp. 154 and 172] | $\langle x, y \mid x^2, y^3, (xy)^3 \rangle$ | $\langle xy \rangle$ | Index 4, with the columns $x, y, y^{-1}$. On p. 172, $\lvert H \rvert = 3$ and $\lvert G \rvert = 12$, with $x \mapsto (1,2)(3,4)$ and $y \mapsto (1,3,2)$ |
| [HOLT2005, Example 5.3, pp. 156-157 and 168] | $\langle x, y \mid x^2y^2, x^3y^5 \rangle$ | Trivial | $\lvert G \rvert = 4$ and $x =_G y$, with the coincidences $6 \sim 1$ and $5 \sim 4$. Rows `2 4 2 4`, `3 1 3 1`, `4 2 4 2`, `1 3 1 3` on p. 157, and after the swap of 3 and 4, `2 3 2 3`, `4 1 4 1`, `1 4 1 4`, `3 2 3 2` on p. 168 |
| [HOLT2005, Example 5.4, p. 160] | $\langle a, b, c, d, e \mid ab = c, bc = d, cd = e, de = a, ea = b \rangle$ | $\langle a \rangle$ | Index 1, "G has order 11", with $b = a^4$, $c = a^5$, $d = a^9$, and $e = a^3$ |
| [HOLT2005, Exercise 1, p. 161] | Free on $x, y$ | $\langle x, y^2, yxyx^{-1}y^{-1}, yx^2yx^{-3}y^{-1}, yx^4y^{-1}, yx^3yx^{-2}y^{-1} \rangle$ | Index 5 |
| [HOLT2005, Exercise 2, p. 161] | $\langle x, y \mid x^2y^2, y^{-1}xyx^{-3} \rangle$ | Trivial | $\lvert G \rvert = 8$, the quaternion group, and "possible to do this without any coincidences" |
| [HOLT2005, Exercise 1, p. 172] | $\langle x, y \mid x^2, y^3, (xy)^4 \rangle$ | $\langle xy \rangle$ | Index 6, $G \cong S_4$ |
| [HOLT2005, Exercise 2, p. 172] | $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$ | $\langle x, yx^{-1}y^2 \rangle$ | Index 5, image $A_5$, $\lvert G \rvert = 60$ |
| [HOLT2005, Exercise 3, p. 173] | $\langle x, y \mid x^3yxyx^{-3}, y^3xyxy^{-3} \rangle$, from Ward | Trivial | $\lvert G \rvert = 3$. HLT without lines 9 and 10 never terminates |
| [SIMS1994, Example 1.2, pp. 220-221] | $\langle x, y \mid x^2 = y^3 = (xy)^3 = 1 \rangle$ | $\langle x \rangle$ | Index 6, $\lvert K \rvert = 2$, order $6 \cdot 2 = 12$. The column of $x$ is $(1)(2,4)(3,5)(6)$. The text layer loses the blanks of the printed tables |
| [SIMS1994, Example 1.3, p. 221] | $\langle x, y \mid x^3 = y^3 = (xy)^3 = (xy^{-1})^3 = 1 \rangle$ | $\langle xy \rangle$ | Index 9, $\lvert K \rvert = 3$, $\lvert G \rvert = 27$ |
| [SIMS1994, Example 1.4, p. 222] | $\langle x, y \mid x \rangle$ | Trivial | Infinite index. GENERAL_CE runs forever |
| [SIMS1994, Example 2.1, pp. 228-230] | $\langle a, b \mid a^3, b^4, (ab)^2 \rangle$ | $\langle b \rangle$ | Index 6, $\lvert G \rvert = 24$. The column of $b$ is $(1)(2,3,5,4)(6)$ |
| [SIMS1994, Example 2.2, pp. 230-231] | $\langle x, y \mid x^2 = y^3 = xyxy^{-1} = (xy)^7 = 1 \rangle$ | Trivial | One state: the group "is trivial" |
| [SIMS1994, Examples 3.1 and 3.2, pp. 235-238] | The data of Examples 2.1 and 2.2 | The same | FELSCH gives the same final tables as HLT |
| [SIMS1994, pp. 241-244] | $\langle a, b \mid a^3, b^4, (ab)^2 \rangle$ | $\langle bab^{-1}a \rangle$ | The standardizing HLT and Felsch strategies reach the same final table of 12 rows (Table 5.4.16). The text does not state the index in words |
| [SIMS1994, Example 5.5 of Chapter 4, p. 186] | Free on $x, y$ | $\langle xyx, x^{-2}y, xy^2x^{-1}y \rangle$ | "a subgroup of index 2 in F", with $x$ swapping cosets 1 and 2 and $y$ fixing both. Without the third generator, BASIC_CE gives an automaton with 4 states |

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

The labels "Example 5.1" to "Example 5.4" and "Exercises Pg. 161, Q2" name the examples of the handbook with the same presentations and subgroups, which state the indices and orders in the table above [HOLT2005, pp. 150-161 and 172]. SymPy's rows of Example 5.1 are the handbook's numbered from 0. The examples "Cox" and "B_2_4" and "Table 1. Ex. 1" of [CDHW1973] are not in Chapter 5 of the handbook.

These examples give counts that test the cost of an enumeration:

- Havas: for Sims's presentation $\langle x, y \mid x^n = y, y^n = x \rangle$ of a cyclic group of order $n^2 - 1$ over the trivial subgroup, "some versions of Felsch-type procedures lead to the definition of a total number of cosets which is exponential in n, while some versions of HLT define a total which is O(n^3)" [HAVAS1991, p. 197].
- Havas: for $G(3,7,17)$ over a subgroup of index 1, HLT reached a maximum of 122661 and a total of 196972 cosets, and Felsch 57284 and 57592 [HAVAS1991, p. 195]. The presentation is not printed.
- Cannon and Havas: a group of order 210, "essentially due to Mennicke (1959)", whose subgroup $\langle x \rangle$ has index 105, took a maximum of 1230970 and a total of 1250191 cosets with the methods of [CDHW1973], and 2854 and 2859 with the new default [CANNON-HAVAS1992, p. 4]. The exponents of its presentation are lost in the text layer, so it is not usable as a test.
- ACE: $\langle a, b \mid a^2, b^3, (ab)^5 \rangle$ over $\langle ab \rangle$ gives "INDEX = 12" with a maximum of 14 and a total of 15 cosets [ACE, Appendix B.3] (summarizing tool). Sims's Exercise 2.3 asks for the same index and prints no answer [SIMS1994, p. 232].
- The handbook gives costs for groups whose presentations the research did not read from the page images, so they are not usable as tests [HOLT2005, pp. 169-170]. For $M_{12}$ of order 95040, HLT with lookahead reached a maximum of 125293 and a total of 315238 cosets, and Felsch 95040 and 97060. For $SL(2,13)$ of order 2184, Felsch defined over 3 million cosets and HLT with lookahead a maximum of 2 million. For $SL(2,17)$ of order 4896, HLT with lookahead reached a maximum of 10 million and Felsch over 40 million. For the first group of order 6561 of the paper that the handbook cites as `[HNO95]`, Felsch defined just over 14000 cosets and HLT with lookahead over 86000, about 40000 with cyclic conjugates. For the eighth, Felsch defined 30 million and HLT with lookahead failed at 100 million. For B. H. Neumann's presentation of the trivial group, Felsch needs about 160 cosets and HLT about 1600 [HOLT2005, p. 197].
- Sims gives the total and the largest number of live states of each of his ten strategies in Tables 5.5.2 to 5.5.7 [SIMS1994, pp. 248-251]. The counts depend on the order of the generators, the subgroup generators, and the relators. For $\langle x, y \mid x^2 = y^3 = (xy)^5 = 1 \rangle$ of order 60 over the trivial subgroup, with $x \mapsto (1,2)(3,4)$ and $y \mapsto (2,3,5)$, strategies 3, 4, 7, 8, 9, and 10 define 60 states in total (Example 5.1). Higman's group $\langle r, s, t \mid t^{-1}rt = r^2, r^{-1}sr = s^2, s^{-1}ts = t^2 \rangle$ is trivial, and strategy 9 defines 588 states in total, all live at once (Example 5.2). For $G_k = \langle x, y \mid x^2 = y^3 = (xy)^7 = (xyxy^{-1})^k = 1 \rangle$, which is "finite for k <= 8", "The orders of G_1, ..., G_8 are 1, 1, 1, 168, 1, 1092, 1092, and 10752" (Example 5.3, from the page image). $\langle a, b \mid a^8 = b^7 = (ab)^2 = (a^{-1}b)^3 = 1 \rangle$ has order 10752, and $\langle a^2, a^{-1}b \rangle$ has index 448 (Example 5.4). Sims's presentation $B_n = \langle x, y \mid x^n = y, y^n = x \rangle$ gives a cyclic group of order $n^2 - 1$, with Table 5.5.6 for $n = 10, 12, 14, 15, 16$ (Example 5.5). The text layer garbles its relators, which the research reads as $x^n y^{-1}$ and $y^n x^{-1}$ without a check against the page image. Table 5.5.4 splits numbers in the text layer, so that "15 14" means 1514.

These examples are of the low-index search:

- SymPy: `low_index_subgroups` of $\langle x, y \mid x^2, y^3, (xy)^4 \rangle$ with $N = 4$ returns, in this order, `[[0, 0, 0, 0]]`, `[[0, 0, 1, 2], [1, 1, 2, 0], [3, 3, 0, 1], [2, 2, 3, 3]]`, `[[0, 0, 1, 2], [2, 2, 2, 0], [1, 1, 0, 1]]`, and `[[1, 1, 0, 0], [0, 0, 1, 1]]`, with columns $x, x^{-1}, y, y^{-1}$ [SYMPY-FPG, lines 751-761]. The tables have 1, 4, 3, and 2 rows.
- The handbook: the same group with $N = 4$, $R_1 = \{x^2, y^3\}$, $R_2 = \{(xy)^4\}$, and $A = \{x, y, y^{-1}\}$ has four classes, of index 1, 2, 3, and 4, which the search finds in the order of index 1, 4, 3, and 2, the order of SymPy's doctest. "It is not hard to prove that G is isomorphic to Sym(4)", so its order is 24 [HOLT2005, Example 5.10, pp. 194-196]. The exercise on p. 197 lists the classes in the same order.
- Sims: for the free product $F$ of a cyclic group of order 2 and an infinite cyclic group, with $X = \{x, y, y^{-1}\}$ in the order $x < y < y^{-1}$ and the columns $x, y, y^{-1}$, there are 11 complete standard tables with at most 3 rows, read from the page image and listed below. "F has one subgroup of index 1, three subgroups of index 2, and seven subgroups of index 3" [SIMS1994, Example 9.2, pp. 205-206]. Proposition 9.3 gives a recursive formula for the number $h(n)$ of complete standard tables with $n$ rows. Its proof is two lines, which count the homomorphisms of $F$ into $S_n$ in two ways and refer to Exercise 8.7 of Chapter 3 for the method, and the text after it gives $h(1) = 1$, $h(2) = 3$, $h(3) = 7$, $h(4) = 23$, and $h(5) = 71$ for this $F$ [SIMS1994, p. 207]. Proposition 9.1 defines $g(e, n)$ and gives its recurrence, and leaves the proof as an exercise. The text after it counts the homomorphisms of $F$ into $S_n$ as $n! \, g(2, n)$ for this $F$, and gives $g(2, n) = 1, 2, 4, 10, 26$ for $n = 1, \dots, 5$ [SIMS1994, p. 206].
- Sims: FIRST_CONJ on the same $F$ with $n = 3$ keeps tables 1, 2, 3, 4, 5, 6, and 8 of the list below. "Thus F has one conjugacy class of subgroups of index 1, three classes of subgroups of index 2, and three classes of subgroups of index 3" [SIMS1994, p. 209].
- Sims: on the same $F$ with $n \ge 4$, a partial table whose first two rows are `2 3 4` and `1 2 2` has, from base state 2 and standardized, the first row `2 1 1`, which precedes `2 3 4`, so no complete table that extends it is first in its class [SIMS1994, Example 9.3, pp. 207-208].
- Sims: $\langle x, y \mid x^3, y^3, (xy)^2 \rangle$ with $n = 4$, the order $x < x^{-1} < y < y^{-1}$, and every relator short, gives three complete tables at the leaves, with the columns $x, x^{-1}, y, y^{-1}$: `1 1 1 1`, then `1 1 2 3`, `3 4 3 1`, `4 2 1 2`, `2 3 4 4`, and then `2 3 3 2`, `3 1 1 3`, `1 2 2 1` [SIMS1994, Example 6.1 and Figure 5.6.1, pp. 254-256, from the page image]. These are classes of index 1, 4, and 3. The text does not name the group or state the number of classes in words. The table of 4 rows has the rows of Brown's final table for the same presentation (above). Three further nodes of the figure, `2 3 1 1`, `3 1 3 ?`, `1 2 ? 2` and `2 3 3 4`, `3 1 ? ?`, `1 2 4 1`, `? ? 1 3` and `2 3 4 ?`, `3 1 ? ?`, `? ? ? 1`, have no descendants.
- Sims: $\langle a, b \mid a^2 = b^3 = (ab)^7 = 1 \rangle$ with $X = \{a, b, b^{-1}\}$, the subgroup generator $abab^{-1}$, and $n = 10$ outputs `1 1 1` and two tables of 7 rows, with the columns $a, b, b^{-1}$, read from the page image: `1 2 3`, `2 3 1`, `4 1 2`, `3 5 6`, `5 6 4`, `7 4 5`, `6 7 7`, and `1 2 3`, `2 3 1`, `4 1 2`, `3 5 6`, `7 6 4`, `6 4 5`, `5 7 7`. "Therefore <a, b | a^2 = b^3 = (ab)^7 = 1> has only two conjugacy classes of proper subgroups of index at most 10 which contain a conjugate of the element abab^-1", and the search tree "has 24 nodes" [SIMS1994, Example 6.2, p. 256].
- Sims: Higman's trivial group with every relator short gives a search tree of 548 nodes at $n = 8$, which "reports finding only G", and of 8179 nodes at $n = 12$ [SIMS1994, Example 6.3, pp. 256-257]. The counts of nodes depend on how the deductions are processed, so they only roughly check an implementation.
- Sims: $\langle a, b \mid a^{11} = b^5 = bab^{-1}a^{-3} = 1 \rangle$ has order 55, with "four classes altogether, consisting of subgroups of index 1, 5, 11, and 55". At $n = 11$ the lenlex search tree has 244 nodes, and with a basic wreath-product ordering it has 18 [SIMS1994, Example 6.4, pp. 257-259, with p. 257 from the page image].- Sims: $M_{11} = \langle a, b, c \mid a^{11} = b^5 = c^4 = (ac)^3 = c^{-1}bcb^{-2} = bab^{-1}a^{-3} = 1 \rangle$ of order 7920 has "three conjugacy classes of subgroups of index at most 12", of index 1, 11, and 12. At $n = 12$ the search tree has 54 nodes with a basic wreath-product ordering and 82928 with lenlex [SIMS1994, Example 6.5, pp. 259-260].

The 11 complete standard tables of Sims's Example 9.2, with the columns $x, y, y^{-1}$ and one row per line [SIMS1994, pp. 205-206]:

```text
(1)  1 1 1
(2)  1 2 2 / 2 1 1
(3)  1 2 2 / 3 1 1 / 2 3 3
(4)  1 2 3 / 2 3 1 / 3 1 2
(5)  1 2 3 / 3 3 1 / 2 1 2
(6)  2 1 1 / 1 2 2
(7)  2 1 1 / 1 3 3 / 3 2 2
(8)  2 2 2 / 1 1 1
(9)  2 2 3 / 1 3 1 / 3 1 2
(10) 2 3 2 / 1 1 3 / 3 2 1
(11) 2 3 3 / 1 2 2 / 3 1 1
```
- SymPy: for the same group, `first_in_class` returns true for `[[0, 0, None, None]]` and for `[[1, 1, 1, None], [0, 0, None, 1]]`, and false for `[[1, 1, 2, 1], [0, 0, 0, None], [None, None, None, 0]]` and for `[[1, 1, 1, 2], [0, 0, 2, 0], [2, None, 0, 1]]` [SYMPY-FPG, lines 865-880].
- GAP: for the group `g` that Section 47.5 defines with the relators `f.1^2, f.2^3, (f.1*f.2)^5` and calls an "fp group of size 60", `LowIndexSubgroupsFpGroup( g, TrivialSubgroup( g ), 10 )` returns 4 subgroups. That the example of Section 47.10-1 uses this `g` is the research's reading, because the example does not define it again. The second has the generators `[ a, b*a*b^-1 ]`, and the first subgroup of the iterator has index 1 [GAP, Section 47.10-1].
- GAP: for $\langle x, y, z \mid x^2, y^4, z^3, (xy)^3, (yz)^2, (xz)^3 \rangle$, the search to index 24 with the excluded elements `[ x, y, y^2, z, x*y, x*z, y*z ]` finds 5 classes of index 24, the first "of index 24 and length 8" and the others of length 24 [GAP, Section 47.10-1].
- GAP: for the relators `x^3=y^3=z^5=(xyx^2y^2)^2=(xz)^2=(yz^3)^2=1`, `LowIndexSubgroupsFpGroup(g,6)` gives 11 classes with the indices `[ 1, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6 ]`, and the image of `DefiningQuotientHomomorphism` for the sixth class has size 360 [GAP, Chapter 47, lines 1220-1227 of the local copy].
- GAP: the coset table of the subgroup generated by `g.1` and `g.2*g.1*g.2*g.1*g.2^-1` in the same `g` has the generator lists `[ 1, 4, 5, 2, 3 ]`, `[ 1, 4, 5, 2, 3 ]`, `[ 2, 3, 1, 4, 5 ]`, and `[ 3, 1, 2, 4, 5 ]`. Its rows in the default lenlex standard are `[1,1,2,3]`, `[4,4,3,1]`, `[5,5,1,2]`, `[2,2,4,4]`, `[3,3,5,5]`, and in the semilenlex standard `[1,1,2,4]`, `[3,3,4,1]`, `[2,2,3,3]`, `[5,5,1,2]`, `[4,4,5,5]` [GAP, Sections 47.6-1 and 47.7-2].

These examples are of Reidemeister-Schreier rewriting:

- SymPy: $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$ with $H = \langle xy, x^{-1}y^{-1}xyx \rangle$ gives the generators $y_1, y_2$ and the relators $y_1^2$, $y_2^3$, $y_2 y_1 y_2 y_1 y_2 y_1$. $\langle x, y \mid x^3, y^3, (xy)^3 \rangle$ with $H = \langle xy, xy^{-1} \rangle$ gives $x_0^3$, $y_0^3$, $(x_0 y_0)^3$. $\langle x, y \mid x^2y^2, y^{-1}xyx^{-3} \rangle$ with $H = \langle x \rangle$ gives $x_0^4$. $\langle x, y \mid x^3y^{-3}, (xy)^3, (xy^{-1})^2 \rangle$ with $H = \langle x \rangle$ gives $x_0^6$ [SYMPY-FPG, lines 1322-1344].
- Hulpke: the subgroup of Example III.20 is $\langle c, e \mid c^2 = e^2 = (ec)^5 = 1 \rangle$, "a dihedral group of order 10" [HULPKE2010, Example III.35, p. 84].
- The handbook: $\langle x, y \mid x^2, y^3, (xy)^6 \rangle$ with $H = [G, G]$ has index 6, and $H$ is free abelian of rank 2, which makes $G$ "an infinite virtually abelian group" [HOLT2005, Example 5.5, pp. 173-177]. Its table on p. 174, with the columns $x, x^{-1}, y, y^{-1}$, has the rows `2 2 3 4`, `1 1 5 6`, `5 5 4 1`, `6 6 1 3`, `3 3 6 2`, and `4 4 2 5`. The doctest of SymPy's `define_schreier_generators` uses this table numbered from 0.
- The handbook: $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$ with $H = \langle xy, x^{-1}y^{-1}xyx \rangle$ has index 5, with the rows `2 3 4 2`, `3 1 1 5`, `1 2 5 4`, `4 4 3 1`, and `5 5 2 3` on p. 177. The Schreier presentation reduces to $\langle b, c \mid b^2, c^3, (bc)^3 \rangle$ [HOLT2005, Example 5.6, pp. 177-178]. SymPy's `reidemeister_presentation` cites this example. The modified coset enumeration of Example 5.7 gives $\langle a, b \mid a^2, b^3, (ab)^3 \rangle$ for the same subgroup [HOLT2005, p. 182].
- The handbook: $\langle x, y \mid x^3, y^3, (xy)^3 \rangle$ with $H = \langle xy, xy^{-1} \rangle$ has $H = G$, one coset, and the presentation $\langle a, b \mid a^3, (ba^{-1})^3, (ba^{-1}b)^3 \rangle$, with no order stated [HOLT2005, Example 5.8, p. 183].
- The handbook: $\langle x, y \mid x^3y^{-3}, (xy)^3, (xy^{-1})^2 \rangle$ with $H = \langle x \rangle$ has 4 cosets, "H is cyclic of order 6 and |G|=24", and $\langle xy \rangle$ has index 8 [HOLT2005, Example 5.9, pp. 183-184].
- The handbook: $\langle x, y \mid x^2y^2, y^{-1}xyx^{-3} \rangle$ with $H = \langle x \rangle$ has index 2, and $H$ is cyclic of order 4 [HOLT2005, Exercise 1, p. 187]. SymPy labels this example "Exercises Q2. Pg 187". $\langle x, y \mid x^2, y^4, (xy)^4 \rangle$ has $G/[G, G] \cong C_2 \times C_4$, and $[G, G]$ is free abelian of rank 2 [HOLT2005, Exercise 2, p. 187].
- Sims: for $\langle x, y \mid x^3, y^3, (xy)^3 \rangle$, a complete table of 9 rows (Table 5.7.1) gives "a subgroup H of G of index 9", with "1 + 9(2 - 1) = 10" Schreier generators. GENERATORS finds that the images of $yx$ and $xyx^{-1}yx^{-1}$ generate $H$ [SIMS1994, Example 7.1, pp. 260-262].

The research found these misprints and doubtful readings in the two books. The research reads each as stated, and neither book has an erratum that the research found:

- [HOLT2005, p. 178]: Example 5.6 says that the subgroup "has order 24, so |H|=24 and |G|=5|H|=60". The figure 24 conflicts with $5 \lvert H \rvert = 60$ and with p. 172, where the same presentation has order 12, so the research reads it as a misprint for 12.
- [HOLT2005, p. 167]: SWITCH loops over $X$ and not over $A$ (Section 7.14).
- [HOLT2005, p. 163]: line 13 of SCANANDFILL has no return after COINCIDENCE (Section 7.11).
- [SIMS1994, p. 228]: "Propositions 1.3 and 1.4" for the results that apply to HLT (Section 7.15).
- [SIMS1994, Example 5.5]: the relators of $B_n$, which the text layer garbles (above).

The handbook's p. 172 example of order 16 and the presentations of its costs on pp. 169-170 are lost in its text layer, and the research did not read them from the page images.

### 7.21. Conventions on Which Sources Differ

All the sources read use right cosets and a right action, and none uses left cosets. The handbook writes "let Omega be the set of right cosets of H in G" [HOLT2005, Example 2.3, p. 18, symbols restored], and Sims notes that "multiplying a right coset of H on the right by an element of F yields a right coset" [SIMS1994, p. 159]. This table gives the other conventions.

| Source | Numbering | Columns | Undefined entry | Standard form |
| --- | --- | --- | --- | --- |
| [GAP] | From 1, coset 1 is $H$ | $g_1, g_1^{-1}, g_2, g_2^{-1}, \dots$, stored as one list per column | Not stated | Lenlex by default, semilenlex on request |
| [HULPKE2010] | From 1 | $a, a^{-1}, b, b^{-1}$ | Blank in the examples, 0 in Algorithm III.27 [p. 79] | Semilenlex (Definition III.25) |
| [HOLT2013] | From 1, $1 = H$ | Relator tables per relator | Undefined | Not stated |
| [HOLT2005] | From 1, coset 1 is $H$ [pp. 151-152] | $A = X \cup X^{-1}$ in a fixed order that an implementation may let the user set [pp. 165 and 170], printed as $x, x^{-1}, y, y^{-1}$ [pp. 151, 168, and 174], with the column of an involution optionally omitted [p. 154] | 0 [pp. 152 and 170] | All columns, by first appearance [p. 167] |
| [SIMS1994] | From 1, state 1 is $H$ [p. 159] | The letters of $X$, with a separate letter for the inverse of a generator of infinite order and one column for an involution [pp. 153-154 and 177] | Blank in printed tables, 0 in a transition matrix, and "?" in a search [pp. 136, 139, 204, and 254] | Lenlex over all columns [Section 3.8, p. 134] |
| [BROWN2011] | From 1 | $a, A, b, B$, uppercase for inverses | 0 | "standard" form not detailed |
| [HAVAS1991] | "1-base array subscripting" | Generators or inverses | 0 | Not stated |
| [SYMPY-CT] | From 0, coset 0 is $H$ | $g_1, g_1^{-1}, g_2, g_2^{-1}, \dots$, one list per row | `None` | Code lenlex, docstring semilenlex |
| [LOWINDEX] | From 1, the base vertex is 1 | Slots $(1,1) < (1,-1) < \dots$ | Empty slot | All slots |
| [SYMMHUB] | From 0 | $g_1, g_1^{-1}, \dots$ | `-1` | Lenlex, by its comment |

The sources also differ on these points:

- HLT: Hulpke fills "gaps of length 1", Holt fills gaps "even for long gaps", and SymPy's scan-and-fill, the handbook's SCANANDFILL, and Sims's full trace fill any gap (Section 7.11). ACE's HLT turns lookahead on, and ITC's searches for consequences after every definition [ACE] [ITC] (summarizing tool).
- Deduction: SymPy's scan uses the word for the single-gap case only, its deduction stack for any assignment "during a definition or "deduction process"", and ACE for any entry that a relator scan assigns [SYMPY-CT, lines 78-80 and 297-301] [ACE, Section 3.2] (summarizing tool). The handbook calls the assignments of a single-gap scan a deduction [HOLT2005, p. 153] and pushes every entry that is "defined or altered" on its stack [HOLT2005, p. 165].
- The surviving number in a coincidence: Holt, the handbook, Sims, ACE, and SymPy's code keep the smaller number, and SymPy's docstring recommends the representative of the larger class, which the handbook calls more efficient in theory (Section 7.13). The handbook keeps the smaller number to avoid an array of class sizes, and Sims to ensure termination.
- Standard form: SymPy's docstring for `standardize` and Hulpke's Definition III.25 ignore the inverse columns, while SymPy's code, the handbook, Sims, GAP's default, and SymmHub's comment use all columns (Section 7.14). SymPy's header comment numbers cosets "1 <= i <= n" while its code numbers them from 0 [SYMPY-CT, line 17].
- The date of Todd and Coxeter: Holt prints "(Todd - Coxeter 1930)" [HOLT2013, p. 15], and Havas and the bibliography of the handbook cite the paper as 1936 [HAVAS1991, reference 16] [HOLT2005].
- Initial table of the low-index search: Hulpke starts from an empty table, and the handbook, Sims, SymPy, and SymmHub from one coset (Sections 7.17 and 7.19).
- Notation: Holt writes $ix$ for the image of coset $i$ under $x$, Hulpke and the handbook write $i^x$, Sims writes $\sigma^x$, and GAP writes the list entry $l[i]$.
- The setting: Sims enumerates over a free product of cyclic groups of order 2 and of infinite order, with the involutions in the rewriting system and one column for each involution (Section 7.9).

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

This section draws on four kinds of source, and Section 10.7 gives the grade of each part:

- Primary texts read directly: Coxeter, Longuet-Higgins, and Miller [CLM1954], read in full from a scan as page images, Har'El [HAREL1993], read in full from a typescript through its text layer, and Chapters 19 to 21 and pp. 389-390 of Conway, Burgiel, and Goodman-Strauss [CBGS2008], read as page images. Section 14.3 lists the parts of each that were read.
- Primary texts read through a summarizing tool: Coxeter's "Regular Polytopes" [COXETER1973], through the OCR text layer of a scan, and the arXiv preprints [SCHULTE2016], [MARTELLI2026], [WHITEHEAD2020], [MONSON2011], [DDS2004], and [DATTA2018].
- Source code: pywonderland [ZHAO-HELPERS] [ZHAO-TILING], mattz's spherical shader [MATTZ-WYTHOFF] and hyperbolic shader [ZUCKER], and knighty's Fragmentarium shader [KNIGHTY-FRAG].
- Secondary sources: Wikipedia, the Polytope Wiki, Egan's notes, and Freire's site. This section cites one only to corroborate a primary source, or where no primary source was read, and says so each time.

[CLM1954] and [HAREL1993] state the construction for spherical triangles. [CLM1954] extends its notation to the Euclidean plane (Section 10.3), and neither paper mentions the hyperbolic plane. [CBGS2008] states the Wythoff triangle for the group `*2pq` without naming a geometry (Section 10.1). A statement below that rests on [CLM1954] or [HAREL1993] therefore holds on the sphere, and in the Euclidean plane only where Section 10.3 says so. Its use for a hyperbolic triangle is open (Section 14.1, item 10).

The quotations of [CLM1954] and [CBGS2008] were transcribed by eye from page images, so a slip of one character is possible, and [CLM1954] prints its fractions in small print. The quotations of [HAREL1993] come from a text layer extracted by a script, which rebuilt the Greek letters from their font codes and the stacked fractions from the positions of their glyphs. The quotations from web sources and from the code of [ZUCKER] passed through a summarizing fetch tool that limits a quotation to about 125 characters, and nobody checked them character by character. An implementer checks a quotation against its source before relying on its exact wording.

### 10.1. Generator Position and Wythoff Symbol

[CLM1954] builds the construction on the spherical triangle `(p q r)`, "whose angles are pi/p, pi/q, pi/r" [CLM1954, p. 405], and ascribes it to Wythoff, "because it was he who first successfully exploited it (in four dimensions)" [CLM1954, p. 406]. Har'El places "a point object C in the Schwarz triangle PQR", which "is now reflected repeatedly to produce a polyhedron, whose vertices are the images of the point object" [HAREL1993, p. 6].

Both papers attach the same position of the generator point to each Wythoff symbol:

| Symbol | Position of the generator point | [CLM1954] | [HAREL1993] |
| --- | --- | --- | --- |
| `p \| q r` | The vertex $P$, where the angle is $\pi/p$ | "the images of one vertex, say the vertex P of the triangle PQR", p. 406 | "C is at the vertex P of PQR", pp. 7-8 |
| `p q \| r` | The point where the bisector of the angle at $R$ meets the side $PQ$ | "the initial point C must be chosen on the bisector of the angle pi/r at R", with C on the side PQ, p. 407 | "C is at the intersection of the side PQ with bisector of the opposite angle R", p. 8 |
| `p q r \|` | The incentre | "the initial point must be precisely the in-centre of the triangles", p. 407 | "C is at the incenter of PQR", p. 8 |
| `\| p q r` | A point in each triangle of one colour, for the snub (Section 10.4) | p. 408 | pp. 8-9 |

[CLM1954] gives the reason for the bisector and the incentre: "In order that these two edges may have the same length" and "In order that these three edges may all have the same length" [CLM1954, p. 407]. Neither paper states in words what the bar means [CLM1954, pp. 406-408] [HAREL1993, pp. 7-9].

The order of the numbers on each side of the bar does not matter in [CLM1954]: "p | q r, which is, of course, the same as p | r q" (p. 406), "p q | r or q p | r" (p. 407), "the numbers p, q, r may be freely permuted" for `p q r |` (p. 407), and "with free permutation again" for `| p q r` (p. 408). [HAREL1993] does not discuss the order. [CLM1954] adds a condition for `p | q r` with rational angles: the sides $PQ$ and $PR$ are "both <= 1/2 pi, so that P is a vertex of faces {q} and {r} whose centres are Q and R (rather than the antipodes of those points)" [CLM1954, p. 413]. It extends the four symbols from triangles with integer $p$, $q$, and $r$ to the Schwarz triangles with rational ones (p. 410). This document uses integers only.

[CBGS2008] states the construction for the group `*2pq` and names no geometry. The orbifold of `*2pq` "is of course a triangle", and the book labels its corners 2, p, and "qs", which looks like a misprint for q [CBGS2008, p. 289]. "Metrically, the triangle has vertex angles pi/2, pi/p, and pi/q, determining its geometry", and "Topologically, there are just seven distinct classes of points on this orbifold: the three vertices, a point on one of the three edges, and an interior point. In each case, there is a canonical representative that is the vertex of an equilateral tiling by regular polygons" [CBGS2008, p. 290]. The book shows which class gives which tiling only in Figure 21.2 (p. 290) and the figure on p. 291, and states no metric rule for the representative. In general, "The vertices of the typical polytope produced from a Coxeter diagram by Wythoff's construction are just the images of a suitably chosen point" [CBGS2008, p. 389].

The preprints define the generator point by its distances to the mirrors. Whitehead chooses "the unique point of unit length that lies on every mirror of S, and is of equal non-zero distance from every mirror not in S" [WHITEHEAD2020, Section 3]. In the compact case, Martelli takes as seed point "the well positioned point in the face that is the intersection of the facets corresponding to the unencircled nodes", which "is thus equidistant from the facets corresponding to the encircled nodes" [MARTELLI2026, Section 5.3]. His Proposition 31 ties this choice to uniformity: "A tessellation obtained from the Wythoff construction is uniform if and only if the seed p is well positioned" [MARTELLI2026, Proposition 31]. For an ideal vertex he writes "In the ideal case, we set p=v", with one encircled node only [MARTELLI2026, Sections 5.3 to 5.5].

Deza, Dutour, and Shpectorov take "a point v in S such that v in H_i if and only if i not in T", with no condition of equal distance [DDS2004, Section 3]. Wikipedia, a secondary source, states that "the one where the generator point is on all the mirrors is impossible" [WP-WYTHOFF-SYMBOL]. No primary source read states that case. [CBGS2008] lists seven classes of point and no class on all three mirrors (p. 290).

That such a point exists and is unique is stated without proof. Martelli states, as Exercise 30, "Every face f of P contains a unique point p that is equidistant from all the facets of P not containing f", and the text gives no proof [MARTELLI2026, Section 5.3]. On the sphere, [CLM1954] and [HAREL1993] fix the point by the bisector and the incentre instead. That these are Martelli's points of equal distance is the research's reading, and Section 14.1, item 30, records it.

The edges join the generator point to its mirror images. For `p | q r`, "The vertex P is joined by an edge to its image by reflexion in the opposite side QR, and the other edges meeting at P are derived from this one by the mirrors PQ and PR" [CLM1954, p. 406]. For `p q | r`, "This vertex C of the new polyhedron is joined by edges to its images by reflexion in the sides PR and QR" (p. 407). For `p q r |`, "This point is joined by edges of the polyhedron to its images by reflexion in the three sides of the triangle" (p. 407). Har'El states the rule for edges and faces together: "two vertices are considered adjacent if they belong to triangles which share an edge, and several vertices are incident to the same face if they belong to triangles which share a vertex" [HAREL1993, p. 6]. Coxeter and Whitehead state the edge rule for any dimension [COXETER1973, Section 11.6, pp. 196-198] [WHITEHEAD2020, Section 5].

For the sphere, both papers state the faces of each symbol and the corners at which they are centred:

| Symbol | Faces and their centres in [CLM1954] | Vertex configuration in [HAREL1993], pp. 7-9 |
| --- | --- | --- |
| `p \| q r` | "a face {q} with centre Q, and a face {r} with centre R", if $q$ and $r$ exceed 2 (p. 406). "each vertex of p \| q r is surrounded by {q}'s and {r}'s, arranged alternately, the number of each being the numerator of the rational number p" (p. 413) | "(q.r.q.r. ... .q.r) (2n terms)", where $p = n/d$, with the face centres $A_1$ and $A_2$ at $Q$ and $R$ |
| `p q \| r` | "a face {2r} with centre R. There is also a face {p} with centre P if p>2, and a face {q} with centre Q if q>2" (p. 407). At a vertex, "{p}, {2r}, {q}, {2r}" (p. 414) | "(p.2r.q.2r)", with the face centres at $Q$, $R$, and $P$ |
| `p q r \|` | "a {2p} with centre P, a {2q} with centre Q, and a {2r} with centre R" (p. 407) | "(2p.2q.2r)", with these centres (p. 9) |
| `\| p q r` | "the faces are {p}'s, {q}'s and {r}'s, each entirely surrounded by triangles" (p. 408) | "(3.p.3.q.3.r)", with "a {p}, a {q} and a {r} with centers at P, Q and R" and three snub triangles |

Neither paper names a face with its centre at $P$ for `p | q r`, where the generator point lies. A face at a corner with a right angle is a digon and drops out. For `p | q 2`, "the 'face' with centre R is a mere digon, which collapses to form an edge" [CLM1954, p. 406], and in Har'El's words, "in all the cases, the occurrence of alpha = pi/2 implies that the corresponding tile is a {2}, i.e., a digon, and may be discarded" [HAREL1993, p. 9]. In each face equation of [HAREL1993], $\alpha_i$ is the angle at the centre $A_i$ of a fundamental right triangle, with $n_i \alpha_i = \pi$ for an $n_i$-gon (p. 2).

The printed formulas and the table rows of [HAREL1993] differ in the order of their lists for some forms, and Section 2.3 states the order this document follows.

### 10.2. Ring Diagrams

[CBGS2008] defines the diagram and the ring. "The Coxeter diagram for a reflection group has a node for each mirror of the kaleidoscope that outlines its fundamental region. If a pair of mirrors meets at angle pi/n, n >= 4, the corresponding nodes are connected by a line marked n", and the book omits the mark for $n = 3$ and the line for $n = 2$. The suitably chosen point of Section 10.1 "is sufficiently well indicated by ringing just those nodes corresponding to mirrors not containing the point" [CBGS2008, p. 389]. The caption of Figure 21.2 says that the Wythoff-Coxeter symbol under each triangle "indicates on which sides of the kaleidoscope the point lies" [CBGS2008, p. 290].

Coxeter introduces the ring for one node: "By drawing a ring around one of the nodes, we obtain a convenient symbol for the tessellation or polyhedron ... whose vertices are all the transforms of the corresponding vertex of the fundamental region" [COXETER1973, Section 5.7, p. 86]. He adds that "the graphs for the polyhedra and honeycombs automatically contain the graphs for the various faces and cells" [COXETER1973, Section 5.7, p. 86]. Section 5.7 names `{p,q}` and `{q,p}` by ringed graphs, but the OCR text layer garbles the graphs, so which ringed node gives which tiling could not be read there. Section 10.3 takes the symbols of both tilings from [CLM1954] instead.

Section 11.6 of [COXETER1973] states the rules for one ring:

- "the removal of an unringed node from the symbol for a polytope or honeycomb leaves the symbol for an element" (p. 197).
- "there is a cell for each free end, i.e., for each unringed node that belongs to only one branch" (p. 197).
- "the symbol for a vertex is derived by removing the ringed node" (p. 198).

Coxeter calls rings on several nodes the general case: "The general case ..., where the typical vertex of the polytope may have other positions in the fundamental region, is symbolized by ringing any number of nodes" [COXETER1973, historical remarks of Chapter XI].

For Schulte and Williams, a primary source, a node is ringed exactly when the generator point lies off its mirror: "a node of the diagram for P^I is ringed if and only if its label belongs to I", where the base vertex $v$ satisfies "rho_i(v) != v for i in I and rho_i(v) = v for i not in I" [SCHULTE2016, Section 3.2]. The stabilizer of $v$ is $\langle r_i \mid i \notin I \rangle$ [SCHULTE2016, Section 4.2].

Neither [CLM1954] nor [HAREL1993] uses a diagram. This table, the research's combination of the positions of Section 10.1 with the ring rule of [CBGS2008, p. 389], gives the rings of each Wythoff symbol:

| Symbol | Mirrors that contain the generator point | Ringed nodes |
| --- | --- | --- |
| `p \| q r` | The sides $PQ$ and $PR$ through the vertex $P$ | The node of $QR$ only |
| `p q \| r` | The side $PQ$ | The nodes of $PR$ and $QR$ |
| `p q r \|` | None | All three |

The Polytope Wiki states the first row in a secondary source: "a | b c has a single ringed node, the node opposite the a edge" [POLYTOPE-WIKI].

Two preprints read the faces off subdiagrams. Martelli states that "The encircled nodes determine a seed point p in P", and that the $h$-faces of the tessellation, up to the group, "are in natural bijection with the Coxeter-Wythoff subdiagrams of D with h nodes", a subdiagram $D'$ in which "each connected component of D' contains at least one encircled node" [MARTELLI2026, Proposition 38]. The fetch tool paraphrased the wording after Proposition 38, so that quotation is not exact. Whitehead marks a mirror that contains the point with a cross, not a ring, and states that "(n-k)-faces are obtained by deleting k vertices from a {0,1}-decorated Coxeter diagram", and that "to be non-degenerate it must be the case that each connected component does not have decoration identically zero" [WHITEHEAD2020, Sections 3 and 5, Theorem 9]. He credits the rule to Champagne, Kjiri, Patera, and Sharp [CKPS1995], which was not read. Monson and Schulte state the case of one ring: the stabilizer of a face has "a connected 'active' part, which has j nodes including the ringed node" [MONSON2011, Section 3].

For two mirrors at the angle $\pi/q$, Har'El states the polygon for each position of the point. "when the point object is located on one of the mirrors, we get regular n-gon, with density d, customarily denoted by {n/d}, i. e., {q}", and when the denominator $d$ is odd and the point lies on the bisector, "it traces a single regular 2n-gon with density d, i.e., a single {2q}" [HAREL1993, p. 6]. For integer $q$, $d = 1$. Freire's site states the same rule for two ringed nodes in a secondary source [FREIRE], and Whitehead adds that "In two dimensions, the uniform polygons are exactly the regular polygons" [WHITEHEAD2020, Section 5].

M23 of `docs/design/poc-plan.md` needs a face rule at each corner of the triangle. The sources state the faces symbol by symbol, and this table, the research's tabulation, restates the faces of Section 10.1 by the mirrors through each corner:

| Mirrors through a corner with the angle $\pi/n$ | Face centred at the corner | Where the sources state it |
| --- | --- | --- |
| Neither active, so the corner is the generator point | No face is named with this centre | The vertex $P$ of `p \| q r` [CLM1954, pp. 406 and 413] [HAREL1993, pp. 7-8] |
| Exactly one active | A regular $n$-gon, or a digon that drops out when $n = 2$ | $Q$ and $R$ of `p \| q r`, and $P$ and $Q$ of `p q \| r` [CLM1954, pp. 406-407] [HAREL1993, pp. 7-9] |
| Both active | A regular $2n$-gon | $R$ of `p q \| r`, and every corner of `p q r \|` [CLM1954, p. 407] [HAREL1993, pp. 8-9] |

[CBGS2008] states the doubling for truncation in general: "truncation means cutting off the corners in such a way that each regular n-gonal face is replaced by a regular 2n-gonal one" [CBGS2008, p. 284]. In Chapter 19 it ties the size of a face to the order of a point of the orbifold in its doily construction, as in "The yellow face has seven edges and so will be a (7a)-gon in the tiling. The purple face has just one edge and so will be a b-gon in the tiling" [CBGS2008, p. 255]. The chapter covers hyperbolic tilings too (Section 10.3), and it calls the doily "a generalization of the Wythoff triangle" (p. 289), but the pages read do not tie its faces to the marks of a Wythoff triangle.

The table holds on the sphere, on the authority of [CLM1954] and [HAREL1993]. No source read states it for a Euclidean or hyperbolic triangle, and Section 14.1, item 10, keeps its use there open.

### 10.3. Families and Vertex Configurations

[CLM1954] writes "{p, q} for the regular polyhedron, whose face and vertex figure are {p} and {q}", so that "the cube is {4, 3}" [CLM1954, p. 403]. [CBGS2008] defines the Schlafli symbol in the same way, as "the Platonic solid whose faces are regular p-gons and whose vertex figures are regular q-gons" [CBGS2008, p. 269].

[CLM1954] relates the two notations. "But if r = 2, as in figure 2, the 'face' with centre R is a mere digon, which collapses to form an edge, and the polyhedron is regular: p | q 2 = {q, p}", and so "{p, q} is q | p 2 (or alternatively if q is even, 1/2 q | p p)" [CLM1954, p. 406]. Its Section 7 repeats this as "p | 2 q = {q, p}" (p. 413). So `{p,q}` is `q | p 2`: the generator point is the corner with the angle $\pi/q$, the faces `{p}` are centred at the corner with the angle $\pi/p$, the right angle gives the digon that collapses, and the node of the side opposite the generator point is the only ring (Section 10.2). The dual of a regular polyhedron has the reversed symbol: "we can read off the symbol of the dual of a polyhedron just by reversing its own symbol" [CBGS2008, p. 273]. The dual of `{p,q}` is therefore `{q,p}`, which [CLM1954] writes `p | q 2`.

The tables print these symbols with the 2 first after the bar. [HAREL1993] gives `3|2 4` for the cube and `4|2 3` for the octahedron (Table 6, p. 16), and `3|2 5` for the dodecahedron and `5|2 3` for the icosahedron (Table 7, p. 17). [CLM1954] gives `4 | 2 3 = {3, 4}` and `3 | 2 4 = {4, 3}` (Table 7, p. 434). By p. 406 of [CLM1954], the order after the bar does not matter (Section 10.1).

[CBGS2008] names the group of the Wythoff triangle "a kaleidoscopic group *2pq" (p. 289). Its table of Chapter 19 lists the group `*na2` with the symbol `<0>*(a)^n` and the example "all regular tilings" (p. 259). Neither sentence states that the group of `{p,q}` is `*pq2` in those words.

This table gives the forms of the `(p q 2)` family. The second and third columns are the general statements of [CLM1954] for the sphere. The last two columns come from Figure 21.2 and the figure on p. 291 of [CBGS2008], which shows the forms of the family of the dodecahedron, $p = 5$ and $q = 3$, labelled with the letters $p$ and $q$. The labels were read from the drawings at twice the size, and the rings were counted by eye. The figure pairs each archifold symbol with a polyhedron, and the research joined that polyhedron by name to the [CLM1954] form in the column Form.

| Form | Wythoff symbol in [CLM1954] | Faces in [CLM1954], in their order at a vertex where it is stated | Archifold symbol in [CBGS2008] | Vertex figure in [CBGS2008], p. 291 |
| --- | --- | --- | --- | --- |
| `{p,q}` | `q \| p 2` (p. 406) | Faces `{p}` and vertex figure `{q}` (p. 403) | `<0>*p^q` (p. 291) | A triangle with sides labelled p and "(q)" inside |
| `{q,p}` | `p \| q 2` (p. 406) | Faces `{q}` and vertex figure `{p}` (p. 403) | `<0>*q^p` (Figure 21.2) | Not in the figure |
| `{p;q}` | `2 \| p q` (p. 406) | `{p}, {q}, {p}, {q}` (p. 410) | `[0]*(pq)^2` (p. 291) | A rectangle p, q, p, q |
| `t{p,q}` | `2 q \| p` (p. 407) | `{2r}, {q}, {2r}` for `2 q \| r = t{r, q}` (p. 414) | `[0]*(2p,q)` (p. 291) | A triangle 2p, 2p, q |
| `t{q,p}` | `2 p \| q`, the row above with $p$ and $q$ exchanged | The row above with $p$ and $q$ exchanged | `[0]*(2q,p)` (Figure 21.2) | Not in the figure |
| `r{p;q}` | `p q \| 2` (p. 407) | `{p}, {4}, {q}, {4}` (p. 415) | `[0][1]*(p,4,q,4)` (p. 291) | A quadrilateral p, 4, q, 4 |
| `t{p;q}` | `2 p q \|` (p. 407) | `{2q}`, `{4}`, and `{2p}`, in Table 2 (p. 404) | `[0][1][2].(2p,2q,4)` (p. 291) | A triangle 2p, 4, 2q |
| `s{p;q}` | `\| 2 p q` (p. 408) | `{q}`, `{3}`, and `{p}` for $p, q > 2$, in Table 2 (p. 404) | None, because the snub "does not receive a Wythoff-Coxeter symbol" (p. 290) | A pentagon with the labels 3, p, and q visible |

Wikipedia's table of the family gives the same symbols, with other orders of the numbers and other names for the forms [WP-UNIFORM-LISTS].

These rows of [HAREL1993] are the convex members of the octahedral and icosahedral families on the sphere, with [CLM1954]'s row for each. The tetrahedral family has two convex rows in each table, `3|2 3 (3. 3. 3)` for the tetrahedron and `2 3|3 (6. 6. 3)` for the truncated tetrahedron [HAREL1993, Table 5, p. 15]. [CLM1954] places another symbol of `(3 3 2)` in another family: "3 3 2 | or 2 3 3 | is the truncated octahedron" (p. 419).

| Name in [HAREL1993] | Symbol and vertex configuration in [HAREL1993] | Row of Table 7 of [CLM1954], p. 434 |
| --- | --- | --- |
| octahedron | `4\|2 3 (3. 3. 3. 3)` | `4 \| 2 3 = {3, 4}` |
| cube | `3\|2 4 (4. 4. 4)` | `3 \| 2 4 = {4, 3}` |
| cuboctahedron | `2\|3 4 (3. 4. 3. 4)` | `2 \| 3 4 = {3;4}` |
| truncated octahedron | `2 4\|3 (6. 6. 4)` | `2 4 \| 3 = t{3, 4}` |
| truncated cube | `2 3\|4 (8. 8. 3)` | `2 3 \| 4 = t{4, 3}` |
| rhombicuboctahedron | `3 4\|2 (4. 3. 4. 4)` | `3 4 \| 2 = r{3;4}` |
| truncated cuboctahedron | `2 3 4\| (4. 6. 8)` | `2 3 4 \| = t{3;4}` |
| snub cube | `\|2 3 4 (3. 3. 3. 3. 4)` | `\| 2 3 4 = s{3;4}` |
| icosahedron | `5\|2 3 (3. 3. 3. 3. 3)` | `5 \| 2 3 = {3, 5}` |
| dodecahedron | `3\|2 5 (5. 5. 5)` | `3 \| 2 5 = {5, 3}` |
| icosidodecahedron | `2\|3 5 (3. 5. 3. 5)` | `2 \| 3 5 = {3;5}` |
| truncated icosahedron | `2 5\|3 (6. 6. 5)` | `2 5 \| 3 = t{3, 5}` |
| truncated dodecahedron | `2 3\|5 (10. 10. 3)` | `2 3 \| 5 = t{5, 3}` |
| rhombicosidodecahedron | `3 5\|2 (4. 3. 4. 5)` | `3 5 \| 2 = r{3;5}` |
| truncated icosidodecahedron | `2 3 5\| (4. 6. 10)` | `2 3 5 \| = t{3;5}` |
| snub dodecahedron | `\|2 3 5 (3. 3. 3. 3. 5)` | `\| 2 3 5 = s{3;5}` |

[HAREL1993] prints the name of the truncated icosidodecahedron as "truncated icosidodechedon", and the table above corrects it. The rows of [HAREL1993] were read from the positions of glyphs in its text layer. Table 7 of [CLM1954] is printed turned by 90 degrees, and it gives each vertex configuration only as a drawing of the vertex figure, so this table does not repeat those drawings.

[CLM1954] extends its notation to the Euclidean plane: "Our notation extends readily to tessellations filling the Euclidean plane", and "Our list, which we believe to be complete, is given in table 8" [CLM1954, p. 430]. It states no angle condition for a Euclidean triangle. These rows of Table 8 [CLM1954, p. 438] are the tessellations whose symbols have only integers or infinity, which the table prints as a sign and this document writes `inf`. The research joined each row by name to the name and face code of [CBGS2008, p. 288] in the last column.

| Row of Table 8 of [CLM1954] | Kepler's figure in Table 8 | Name and face code in [CBGS2008], p. 288 |
| --- | --- | --- |
| `4 \| 2 4 = {4, 4}` | E | quadrille, `4444` |
| `2 4 \| 4 = t{4, 4}` | V | tr. quadrille, `488` |
| `\| 2 4 4 = s{4;4}` | N | snub quadrille, `43343` |
| `6 \| 2 3 = {3, 6}` | D | deltille, `333333` |
| `3 \| 2 6 = {6, 3}` | F | hextille, `666` |
| `2 \| 3 6 = {3;6}` | P | hexadeltille, `3636` |
| `2 3 \| 6 = t{6, 3}` | S | tr. hextille, `3 12 12` |
| `3 6 \| 2 = r{3;6}` | Ii | rhombihexadeltille, `3464` |
| `2 3 6 \| = t{3;6}` | Mm | tr. hexadeltille, `4 6 12` |
| `\| 2 3 6 = s{3;6}` | L | snub hextille, `63333` |
| `2 inf \| 2 = t{2, inf}` | None | Not in the table |
| `\| 2 2 inf = s{2;inf}` | None | Not in the table |

Table 8 is printed turned by 90 degrees and was read at 300 dots per inch. Its last rows, the two "nameless tessellations" that alternate strips of squares and triangles (p. 430), have no symbol, and their Kepler and Badoureau entries may be misaligned by one row. [CBGS2008] lists an isosnub quadrille with the face code `44333`, and the research did not record which nameless row it matches.

In the hyperbolic plane, the infoboxes of the Wikipedia pages of the forms of `{7,3}` give these symbols, in a secondary source [WP-732]. Their names are Wikipedia's, in which `r{7,3}` is the form `{p;q}` and `rr{7,3}` is the form `r{p;q}` of [CLM1954].

| Form in [CLM1954] notation | Member of `(7 3 2)` | Wythoff symbol in its infobox | Vertex configuration in its infobox |
| --- | --- | --- | --- |
| `{p,q}` | `{7,3}` | `3 \| 7 2` | `7^3` |
| `t{p,q}` | `t{7,3}` | `2 3 \| 7` | `3.14.14` |
| `{p;q}` | `r{7,3}` | `2 \| 7 3` | `(3.7)^2` |
| `t{q,p}` | `t{3,7}` | `2 7 \| 3` | `7.6.6` |
| `{q,p}` | `{3,7}` | Not recorded | Not recorded |
| `r{p;q}` | `rr{7,3}` | `3 \| 7 2`, which conflicts with the convention of [CLM1954] (Section 13) | `3.4.7.4` |
| `t{p;q}` | `tr{7,3}` | `2 7 3 \|` | `4.6.14` |
| `s{p;q}` | `sr{7,3}` | `\| 7 3 2` | `3.3.3.3.7` |

The research compared these symbols with the general symbols of the first table with $p = 7$ and $q = 3$, and they agree up to the order of the numbers, except for `rr{7,3}`. No primary source read states the symbols or the vertex configurations of a hyperbolic family (Section 14.1, item 32).

The infoboxes cite Chapter 19 of [CBGS2008] and Chapter 10 of Coxeter's "The Beauty of Geometry" [COXETER1999]. Chapter 19 was read. Its section "The Hyperbolic Archimedean Tessellations" counts the permutation symbols of each rank and shows twelve tilings in the Poincare disk [CBGS2008, pp. 261-266]. It lists no forms of `(7 3 2)` and gives no Wythoff symbol. The chapter argues that a hyperbolic tiling exists for a face code, unique for its symbol, because one edge length makes the angles of the regular polygons at a vertex add to a full turn, and "With trivial modifications the argument works also in the Euclidean and spherical cases" (p. 256). [COXETER1999] was not read. Datta and Gupta state in a preprint that the triangle groups $(\pi/2, \pi/p, \pi/q)$ "generate a uniform tiling with vertex-type [p^q] whenever 1/p + 1/q < 1/2", and give no Wythoff symbols [DATTA2018, Section 1].

For a spherical triangle `(p q r)` without a right angle, Section 10.1 gives the faces of each symbol. No source read lists the forms of a hyperbolic triangle `(p q r)` (Section 14.1, item 10).

### 10.4. Snubs

[CLM1954] builds the snub from the rotations: "The remaining Archimedean solids are obtained by taking, instead of the whole group generated by reflexions, the subgroup of index 2 consisting of rotations". The spherical triangles are "alternately white and black", and the vertices lie in the white triangles, "so situated that the points in the three white triangles surrounding any black one form an equilateral triangle" [CLM1954, p. 408]. It writes `| 2 p q = s{p;q}` (p. 408). Its eq. (10.1) places the vertex, where $C$ is the corresponding point in the black triangle (pp. 420-421):

$$\sin PC \, \sin(\pi/p) = \sin QC \, \sin(\pi/q) = \sin RC \, \sin(\pi/r)$$

It adds that "we can describe C as the point whose orthogonal projections form an equilateral triangle" (p. 421). Har'El takes "only images under an even number of reflections" and builds the vertex from the Fermat point of the triangle, "assuming it exists", whose existence a footnote argues "by a continuity argument" [HAREL1993, p. 8]. Both state this for the sphere.

[CBGS2008] keeps the snub out of the Wythoff triangle: "(The snub is chiral and does not arise from the Wythoff triangle, so does not receive a Wythoff-Coxeter symbol.)" [CBGS2008, p. 290]. Section 13 records the difference. M23 of `docs/design/poc-plan.md` leaves snubs out.

### 10.5. Quadrilateral Domains

[CLM1954] and [HAREL1993] treat triangles only. [CBGS2008] enumerates the Archimedean tilings of groups with more corners through permutation symbols and face codes, such as `[0][1]*(a,2b,c,2b)^n` for `*nabc`, and does not call that enumeration a Wythoff construction [CBGS2008, pp. 259-260]. The pages read do not say which reflection polygons, such as `*3333`, give tilings by a Wythoff construction.

Wikipedia states, in a secondary source, that "The only possible fundamental domain in Euclidean 2-space that is not a simplex is the rectangle (infinity 2 infinity 2)", and "All forms generated from it become a square tiling" [WP-UNIFORM-TILING]. "Further symmetry groups exist in the hyperbolic plane with quadrilateral fundamental domains starting with (2 2 2 3)", and the page gives no construction rule [WP-UNIFORM-TILING]. The source writes the infinity sign as a symbol, replaced here by the word.

### 10.6. Per-Pixel Rendering

#### 10.6.1. The Fold

Zhao Liang's renderer reflects the point "repeatedly ... off mirrors until it falls within the fundamental region", and colours the pixel "based on q's position within the fundamental domain", which "allows for parallel computation on all pixels" [ZHAO]. The page calls this the "inverse pixel method" and describes it in prose only [ZHAO]. For drawing faces from the group instead of per pixel, "The stabilizing subgroup of f_0 is the standard parabolic subgroup <i,j>", and the edges of type $i$ are obtained "by applying the coset representatives in G/H" [ZHAO].

The page gives no explicit classification of a pixel into a face, no count of face types, and no rule for snubs [ZHAO]. It credits "a shader program exported from Matt Zucker's excellent work" at Shadertoy 3tsSzM [ZHAO]. That Shadertoy page could not be read. Section 10.6.3 reads instead the shader "Hyperbolic Wythoff explorer v2" by mattz in pywonderland [ZUCKER], and no source read states that it is the shader of 3tsSzM.

#### 10.6.2. The Generator Point

pywonderland computes the generator point by one linear solve on the mirror normals, in all three geometries [ZHAO-HELPERS] [ZHAO-TILING]. Its function `get_point_from_distance` takes the normals "stored as row vectors in `M`" and "a tuple of non-negative floats `d`", and computes "the vector `v` whose distance vector to the mirrors is `d`" [ZHAO-HELPERS]. The steps, as the code states them:

1. The rows of `M` are the normals of the mirrors, a lower-triangular factor of the Gram matrix built from `C = -np.cos(np.pi / coxeter_matrix)`. In the hyperbolic case the last diagonal entry is `-np.sqrt(abs(...))` [ZHAO-HELPERS].
2. An entry of `d` is zero exactly for a mirror that is not active. The comment `# a mirror is active iff the initial point is not on it` stands above `self.active = tuple(bool(x) for x in init_dist)` [ZHAO-TILING].
3. In the spherical and Euclidean cases the point is `p = np.linalg.solve(M, d)` [ZHAO-HELPERS].
4. In the hyperbolic case the code negates the last column first and normalizes after the solve: `N = M.copy()`, `N[:, -1] *= -1`, `p = np.linalg.solve(N, d)`, and `p = hnormalize(p)` [ZHAO-HELPERS].
5. In the Euclidean case the point is divided by its last coordinate, `return p / p[-1]`. In the hyperbolic case the code rejects an ideal point with `assert not helpers.is_ideal(v), "The initial point cannot be ideal"` [ZHAO-TILING].

Three shaders build the point from weights on the corners of the triangle, with no derivation. mattz's spherical shader writes `poly_vertex = normalize(sphere_from_bary(bary_poly_vertex.xyz));` [MATTZ-WYTHOFF]. His hyperbolic shader writes `vec3 generator = hyperNormalizeP(verts * gbary);` under the comment "compute generator position from barycentric coords", and its keys "1-7 Set generator point" choose the weights [ZUCKER]. knighty's writes `p=hnormalizew(U*pA+V*pB+W*pC);` in the hyperbolic plane [KNIGHTY-FRAG].

[CLM1954] and [HAREL1993] give no coordinates for the point of `p q | r` or `p q r |` beyond the bisector and the incentre (Section 10.1). [HAREL1993] solves instead for the angles at the point through the fundamental right triangles, with $n_i \alpha_i = \pi$, $\sum \gamma_i = \pi$, and $\cos a = \cos \alpha_i / \sin \gamma_i$, by a Newton iteration on the sphere (pp. 2-5). Only the snub has a locating equation in [CLM1954] (Section 10.4).

No source read states the generator point in trilinear or barycentric coordinates for a general triangle `(p q r)` in all three geometries, or as a closed formula through the inverse of the Gram matrix (Section 14.1, item 33). No source read states that the uniform construction sets every active entry of `d` to one constant (Section 14.1, item 34).

#### 10.6.3. Assigning a Point to a Face

No paper or book read states how to assign a folded point to a face of a Wythoff tiling, or which regions of the fundamental triangle belong to which face. [CBGS2008] shows such regions only in figures. In the second column of the figure on p. 291, each Wythoff triangle is shaded in regions coloured like the faces of the polyhedron beside it, and Figure 21.2 (p. 290) draws blue segments from the marked point to the sides. The text gives no rule for the boundaries of the regions and does not say that the segments are perpendiculars [CBGS2008, pp. 290-291].

Two implementations by one author, mattz, assign a point to a face, one on the sphere and one in the hyperbolic plane. Both are under CC BY-NC-SA 3.0, so this document copies no code from them beyond short quotations.

The shader "Wythoff explorer", Shadertoy Md3yRB, covers the sphere and three-dimensional polyhedra [MATTZ-WYTHOFF]. It was read from two third-party copies, not from Shadertoy. Its steps, as the code states them:

1. Fold the point into the triangle with `Q.M = tile_sphere(pos);`.
2. Build one cut plane for each edge of the triangle with `poly_edges[i] = normalize(cross(poly_vertex, tri_edges[i]));` in `common.glsl`. The comment in `construct()` describes it as "polyhedron edge cut plane (passes thru origin and V, perpendicular to triangle edge)", where V is the generator point.
3. For each cut plane, take the sign of `float poly_edge_dist = dot(pos, poly_edge);` and select the "triangle vertex on the same side of face edge as point" with `int vidx = (eidx + (poly_edge_dist > 0. ? 2 : 1)) % 3;`.
4. Test the point against `vec3 bisector = cross(poly_vertex, poly_edge - other_poly_edge);`, and when `bisector_dist > 0.`, record the edge and the corner as `Q.eidx = eidx; Q.vidx = vidx;`.

The header of the spherical shader credits knighty's "Polyhedron again", Shadertoy XlX3zB.

The shader "Hyperbolic Wythoff explorer v2", by mattz, was read on 2026-11-23 in the copy `src/shader-playground/glsl/wythoff_hyperbolic/main.frag` of pywonderland [ZUCKER]. Section 4.4 of `docs/research/rendering.md` gives its fold. Its steps for the face, as the code states them:

1. Set up the triangle in the hyperboloid model, with one vector for each mirror, `edges = setupTriangle(pqr.xyz);`, and its corners, `mat3 verts = hyperTriVerts(edges);`.
2. Fold by reflecting the triangle toward the point, "repeated reflections of triangle till it lands on x", in `flipTriangleToPoint`, which treats `hyperDot(x, edges[i]) < 0.` as inside and keeps only the parity of the flips.
3. Build the generator point from barycentric weights (Section 10.6.2), and one perpendicular for each mirror, under the comment "compute perpendiculars thru generator to edges".
4. In `computeFace`, measure the point against each perpendicular with `pdists = -hyperConj(x) * perps` and `pdistsTrue = asinh(abs(pdists))`. For each perpendicular that has a length, build the line `vec3 l = geodesicPerpThruPoint(perps[i], generator);`, "the line perpendicular to perps[i] passing thru generator", and take `float d = hyperDot(x, l)*hyperDot(verts[i], l) > 0. ? gdist : pdistsTrue[i];`, under the comment "if x is over that line, take distance to generator, otherwise distance to perp". The perpendicular with the least `d` is `face.x`.
5. Set `face.y = TRI_NEXT(face.x)` and `face.z = TRI_LAST(face.x, face.y)`. When the flag `BIT_PERP_SPLITS_EDGE` is set, choose by the side test `if (pdists[face.x]*hyperDot(verts[face.y], perps[face.x]) > 0.) { face = face.xzy; } else { face = face.xyz; }`. When the flag `BIT_VERT_IS_FACE` is set for `face.y`, take `face = face.xyy;`, "was a mirror edge so choose the only valid vertex along it", and otherwise `face = face.xzz;`. The reading did not record how the two flag tests nest.
6. In `mainImage`, blend the fill of the face `face.y` with that of `face.z` by the signed value `float pmin = fparity*hyperDot(x, perps[face.x]);`, where `float fparity = hyperDot(verts[face.y], perps[face.x]) < 0. ? -1. : 1.;`, through `smoothstep(-0.5, 0.5, pmin*lengthScale*0.5)`. The code does not explain the sign convention.

The header of `computeFace` states its result: "return integers such that x is nearest perpendicular," "y is nearest face, and z is second-nearest face," and "note that y == z if the perpendicular is a mirror (i.e. lies on an edge incident to a right angle)". The code indexes `verts` with `face.y`, so each face is named by a corner of the triangle. The flags `BIT_PERP_HAS_LENGTH`, `BIT_PERP_SPLITS_EDGE`, and `BIT_VERT_IS_FACE`, and the helpers `setupTriangle`, `hyperTriVerts`, `reflectPG`, `geodesicPerpThruPoint`, and `hyperDistPP`, are defined in another file, which this reading did not read.

The research reads both shaders as cutting the triangle by the perpendiculars from the generator point to the mirrors into up to three regions, one around each corner, and assigning the point to the face centred at the corner of its region. Neither file states this rule in prose beyond the header of `computeFace`, and no paper read states the decomposition (Section 14.1, item 11). No implementation read assigns a point to a face in the Euclidean plane.

These sources were checked and give no face rule:

- knighty's Fragmentarium shader folds each pixel with `t=-2.*min(0.,hdot(pos,nb)); pos+=t*nb;` and picks only the nearest edge or vertex, with `float d=min(min(da,db),min(dc,dv));` [KNIGHTY-FRAG].
- Egan's notes [EGAN].
- Nelson and Segerman, arXiv:1511.02851, which builds three-dimensional honeycombs by sphere inversions from dihedral angles.
- Arcus's polyjs, the repositories `postmath/hyperbolic-tiling`, `mountain/hyperbolic-wythoff`, and `neozhaoliang/Hyperbolic-Honeycombs`, and Don Hatch's page. Their addresses are not recorded.

#### 10.6.4. SymmHub

SymmHub has no input for a Wythoff symbol or a Coxeter diagram [SYMMHUB]. No such input was found in a reading of its group-construction code.

### 10.7. Status of Each Part

| Part | Status |
| --- | --- |
| Generator positions and Wythoff symbols (Section 10.1) | Primary, read directly: [CLM1954] pp. 406-408 and [HAREL1993] pp. 7-9, for the sphere. [CBGS2008] p. 290 names the seven classes of point for `*2pq` without naming a geometry and shows the placements only in figures |
| The generator point by equal distances to the active mirrors (Section 10.1) | Primary preprints [WHITEHEAD2020] and [MARTELLI2026], read through a summarizing tool. On the sphere, the bisector and the incentre of [CLM1954] and [HAREL1993]. Their identification with the point of equal distance, and the existence and uniqueness of the point, are open (Section 14.1, item 30) |
| Edges join the generator point to its mirror images (Section 10.1) | Primary, read directly: [CLM1954] pp. 406-407 and [HAREL1993] p. 6, for the sphere. [COXETER1973] and [WHITEHEAD2020] corroborate |
| Faces and their centres for each Wythoff symbol, and the digons that drop out (Section 10.1) | Primary, read directly: [CLM1954] pp. 406-408, 413-414, and 418, and [HAREL1993] pp. 6-9, for the sphere |
| What a ring means (Section 10.2) | Primary, read directly: [CBGS2008] p. 389. [COXETER1973] for one ring and [SCHULTE2016] for any number corroborate |
| The rings of each Wythoff symbol (Section 10.2) | The research's combination of [CLM1954] and [CBGS2008] p. 389 |
| Faces from subdiagrams (Section 10.2) | Primary preprints [MARTELLI2026] and [WHITEHEAD2020], read through a summarizing tool. The wording after Proposition 38 was paraphrased by the tool |
| Face rule at a corner of the triangle (Section 10.2) | On the sphere, the research's tabulation of the statements of [CLM1954] and [HAREL1993] for each symbol. Not stated for the hyperbolic plane (Section 14.1, item 10) |
| `{p,q}` as `q \| p 2` and its dual `p \| q 2` (Section 10.3) | Primary, read directly: [CLM1954] pp. 406 and 413, with the tables of [HAREL1993] pp. 16-17 and [CLM1954] p. 434, for the sphere, and Table 8 of [CLM1954] for the Euclidean plane. The dual by [CBGS2008] p. 273 |
| The forms of the `(p q 2)` family with their faces (Section 10.3) | Primary, read directly: [CLM1954] pp. 403-415, for the sphere, with the figures of [CBGS2008] pp. 290-291 read from drawings |
| The convex uniform polyhedra (Section 10.3) | Primary, read directly: Tables 5 to 7 of [HAREL1993] and Table 7 of [CLM1954] |
| The Euclidean tessellations (Section 10.3) | Primary, read directly: [CLM1954] p. 430 and Table 8, read from a page printed turned. The names and face codes of [CBGS2008] p. 288 joined by the research |
| The forms of `(7 3 2)` (Section 10.3) | Wikipedia only, summarized. One conflict for `rr{7,3}`, judged against [CLM1954] (Section 13 and Section 14.1, item 32) |
| The forms of a hyperbolic `(p q r)` (Section 10.3) | Not available (Section 14.1, item 10) |
| Snubs (Section 10.4) | Primary, read directly: [CLM1954] pp. 408 and 420-421 and [HAREL1993] p. 8, for the sphere. [CBGS2008] p. 290 keeps the snub out of the Wythoff triangle (Section 13) |
| Quadrilateral domains (Section 10.5) | No construction rule in a primary source read. Wikipedia only for the remarks (Section 14.1, item 12) |
| Per-pixel fold into the triangle (Section 10.6.1) | Primary web source [ZHAO] |
| The generator point by a linear solve (Section 10.6.2) | Source code only [ZHAO-HELPERS], for all three geometries. No closed formula sourced (Section 14.1, items 33 and 34) |
| Assigning a folded point to a face (Section 10.6.3) | Source code only: two shaders by one author, on the sphere [MATTZ-WYTHOFF] and in the hyperbolic plane [ZUCKER]. No paper or book states a rule, and [CBGS2008] shows the regions in a figure only (Section 14.1, item 11) |
| Perfect colourings of Wythoff faces | No source beyond Lemma 3.2 for the regular and Laves tilings (Section 14.1, item 5) |

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
| Machinery | A presentation of $G$, a low-index subgroup enumeration, and the fold loop carrying a permutation or a coset class (Sections 7 and 8). SymmHub implements this design. | Not established by any source. | A presentation of $G_{p,q}$, a low-index subgroup enumeration filtered by stabilizer generators (Section 5.5), and a classification of each pixel into a face, for which two shaders by one author, one spherical and one hyperbolic, give a method and no paper or book does (Section 10.6.3). The faces of each Wythoff symbol are sourced on the sphere (Section 10.1). |
| Software precedent | SymmHub (Section 11). | None found. | KaleidoTile draws Wythoff tilings, with an undocumented colour scheme. Zhao Liang's renderer folds into the triangle. mattz's spherical and hyperbolic shaders assign a folded point to a face (Section 10.6.3). |
| Gaps left open | Correspondence between colourings and subgroups for arbitrary orbifold groups, and which indices occur (Section 14.1, items 1, 2, and 9). | Every question about colour counts and perfection (Section 14.1, item 14). | The face rule at a corner of a hyperbolic triangle, the per-pixel face classification in a paper or book, perfect colourings beyond regular and Laves tilings, groups that are not triangle groups, snubs, and Euclidean counts (Section 14.1, items 5, 10, 11, 12, and 15). |

## 13. Conflicts Between Sources

- Frettloh's own statements on what is counted: Section 3 says that different colourings may share a colour symmetry group, and the conclusion speaks of a bijection between perfect colourings and subgroups [FRETTLOH2008, pp. 3 and 5]. This document reports both (Section 3.4).
- The year of Frettloh's paper: Crossref gives volume 223 with an issue dated December 2008 and online publication on 2009-09-25, and the MIT tilingsearch database cites 2009. This document uses the volume year, 2008.
- The date of Felix and Loquias's paper: the arXiv posting is dated 2010-02-02, and the paper appeared in 2008 in Z. Kristallogr. 223. This document uses 2008.
- The number of isohedral types: 93 types, of which 81 are the isohedral tiling types [KAPLAN2025]. The Tactile README calls the other 12 "boring" without a reason [TACTILE], and Kaplan's 2025 page gives the reason, mirror edges.
- Whether the snub belongs to the Wythoff construction: [CLM1954] gives the snub the fourth Wythoff symbol `| p q r` (p. 408), and [HAREL1993] calls it the fourth position of the point (p. 8). [CBGS2008] states that the snub "does not arise from the Wythoff triangle, so does not receive a Wythoff-Coxeter symbol" (p. 290). This document follows [CLM1954] for the symbol and treats the snub apart, as Section 10.4 does, and M23 of `docs/design/poc-plan.md` leaves it out.
- The Wythoff symbol of `rr{7,3}`: the infobox of the Wikipedia page "Rhombitriheptagonal tiling" gives "3 | 7 2", and the table of the `(p q 2)` family on Wikipedia gives `p q | 2`, which is `7 3 | 2` here [WP-732] [WP-UNIFORM-LISTS]. In the convention of [CLM1954], `3 | 7 2` is the symbol of `{7,3}` (p. 406), and the form `r{p;q}` is `p q | 2` (p. 407), as in `3 4 | 2 = r{3;4}` in its Table 7 (p. 434). This document therefore follows the table and reads the infobox as wrong. [CLM1954] states its convention for the sphere and the Euclidean plane, so its use for `(7 3 2)` rests on item 10 of Section 14.1 (Section 10.3).
- The vertex configuration of `a b | c` on the Polytope Wiki: the fetch tool returned "a.2b.a.2b" and noted that it contradicts the page's own example [POLYTOPE-WIKI]. The primary sources settle it: the faces at a vertex of `p q | r` are "{p}, {2r}, {q}, {2r}" [CLM1954, p. 414], and its vertex configuration is "(p.2r.q.2r)" [HAREL1993, p. 8] (Section 10.1).
- The DOI of [HAREL1993]: the Springer record gives 10.1007/BF01263494, and MathWorld cites 10.1007/BF01263495. The copy read prints no DOI, so it does not settle the conflict.
- The author and origin of the hyperbolic shader [ZUCKER]: the header of the copy in pywonderland names the author "mattz" only and says that the shader "is basically https://www.shadertoy.com/view/wtj3Ry with a few improvements". [ZHAO] credits Matt Zucker for the shader at Shadertoy 3tsSzM. No source read joins the name "mattz" to Matt Zucker or either Shadertoy page to the copy read, and neither Shadertoy page was read. This document and `docs/research/rendering.md` give the author as mattz, identified as Zucker, M. only through the credit of [ZHAO].
- Access to [COXETER1973]: two readers read the OCR text at the address of Section 15, and a third reported the copy as lending only, with no text. The address of Section 15 returned text.
- The versions of [MARTELLI2026]: v2 of 2026-01-16 and v3 of 2026-05-01 were read by different readers, who reported no difference in content.
- The factor between curvature and Euler characteristic in [BALKE1996]: the caption of Table I (p. 91) says that the Euler characteristic is twice the curvature, while Theorem 4.1 (p. 97) and the introduction (p. 90) say that the curvature is twice the Euler characteristic. This document reports both (Section 9.2).
- The locator of the spherical counts: a condensed extraction of Frettloh's paper placed the cube and the icosahedron in Sections 1 and 2, and the full PDF places the cube on p. 4. The icosahedron was not re-checked in the PDF.
- The order of a subgroup in the handbook: Example 5.6 calls the order of $\langle b, c \mid b^2, c^3, (bc)^3 \rangle$ 24 and concludes $\lvert G \rvert = 5 \lvert H \rvert = 60$, while p. 172 gives the same presentation order 12 [HOLT2005, pp. 172 and 178]. This document reads 24 as a misprint (Section 7.20).

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
10. The face rule of Wythoff tilings: closed for the sphere, open for the Euclidean and hyperbolic planes. On the sphere, [CLM1954] and [HAREL1993] state for each Wythoff symbol the faces with their centres at the corners, and that a face at a right angle is a digon that drops out [CLM1954, pp. 406-408, 413-414, and 418] [HAREL1993, pp. 6-9] (Section 10.1). The general rule is stated in primary form by preprints: the faces of each dimension correspond, up to the group, to the subdiagrams in which each connected component contains a ringed node [MARTELLI2026, Proposition 38] [WHITEHEAD2020, Theorem 9] (Section 10.2). These remain open:
    - The rule at a corner of a hyperbolic triangle with the angle $\pi/n$: a regular $n$-gon when exactly one of the two mirrors through the corner is active, a $2n$-gon when both are, and no face when neither is. The table of Section 10.2 restates the spherical statements by corner, which is the research's tabulation. No source read states it for a hyperbolic triangle. [CBGS2008] states the seven classes of point for `*2pq` without naming a geometry and gives the tilings of each class only in figures (pp. 290-291), and the face sizes of its doily construction are not tied to the marks of a Wythoff triangle in the pages read (p. 255). Applying the spherical statements to a hyperbolic triangle is the research's reading.
    - The Euclidean faces: [CLM1954] extends its notation to the Euclidean plane and lists the tessellations in Table 8, but does not restate the faces for a Euclidean triangle (p. 430).
    - The exact wording of Proposition 38 of [MARTELLI2026], which the fetch tool paraphrased, and the geometries that it covers, which the research did not record.
    - The forms of a hyperbolic family at a primary source (item 32).
    - The face list of snubs in the hyperbolic plane. [CLM1954] and [HAREL1993] state it for the sphere (Section 10.4). M23 of `docs/design/poc-plan.md` leaves snubs out.

    Would close it: a primary text that states the construction or its faces for a hyperbolic triangle, such as Chapter 10 of [COXETER1999], Johnson's "Geometries and Transformations" of 2018, or [CKPS1995], or [MARTELLI2026] read directly if its Proposition 38 covers the hyperbolic plane, or Lean.
11. Per-pixel classification for Wythoff tilings: partly closed for the sphere and the hyperbolic plane, by source code only, and open for the Euclidean plane. Two shaders by one author assign a folded point to a face. The spherical one cuts the folded triangle by planes through the generator point, perpendicular to each edge of the triangle, and assigns the point to the corner on its side [MATTZ-WYTHOFF]. The hyperbolic one finds the nearest of the perpendiculars from the generator point to the mirrors and assigns the point to a corner by its side of that perpendicular, with the corner index naming the face [ZUCKER] (Section 10.6.3). These remain open:
    - The decomposition of the triangle into up to three regions, one for each face around a corner, in any paper or book. [CBGS2008] shows the regions in the figure on p. 291 only and states no rule for their boundaries.
    - Whether the regions of either shader are the faces of the tiling for every pattern of active mirrors, which is the research's reading of the code.
    - The helpers and flags of the hyperbolic shader that are defined in another file, `geodesicPerpThruPoint` and `BIT_PERP_SPLITS_EDGE` among them, and how its two flag tests nest.
    - The same assignment in the Euclidean plane, for which no implementation was read.

    Would close it: a paper that states the decomposition, the other file of the hyperbolic shader read line by line, the source of [KALEIDOTILE], or Lean.
12. The Wythoff construction beyond triangles: polygonal kaleidoscopes such as `*2222` and `*abcd`, and mixed orbifolds such as `3*3` and `2*22`. Martelli states the seed point and the face correspondence for a Coxeter polyhedron in general [MARTELLI2026, Section 5.3 and Proposition 38], and the research did not record whether those statements cover the reflection polygons of the plane. [CLM1954] and [HAREL1993] treat triangles only. [CBGS2008] enumerates the Archimedean tilings of groups such as `*nabc`, `2*nab`, and `a*n2` by permutation symbols and face codes, and does not say which of them a Wythoff construction gives (pp. 259-260, Section 10.5). No source read states a Wythoff construction for the mixed orbifolds.
13. Normal subgroups: whether a subgroup is normal exactly when $G/H$ acts regularly on the colours, and how the kernel $K$ relates to the core of the colour group.
14. Design (b): no source read addresses the colouring of a fundamental domain cut into several polygons, or the numbers of colours it offers.
15. Euclidean perfect colourings: the counts for Euclidean regular and Laves tilings, deferred by Frettloh to [SCHWARZENBERGER1984]. [CBGS2008] tabulates the twofold and prime-fold colour types of the 17 plane groups (Sections 6 and 7.6), but not counts for regular and Laves tilings in Frettloh's sense.
16. The index-$n$ colouring theorem of De Las Penas, Decena, and Laigo [DDL2009]: a correspondence between subgroups of index $n$ and transitive colourings with $n$ colours is known from a search summary only, and the theorem number was not retrieved.
17. Tool details: the face colouring of KaleidoTile, the behaviour of Tess, and the colour features of iOrnament, Morenaments, and Escher Web Sketch.
18. The curvature of EPINET: whether the quantity that [EPINET-ORB] calls curvature equals the orbifold Euler characteristic of the signature, so that it is half the curvature $\mathcal{K}$ of a symbol in [BALKE1996] (Section 9.2). A comparison of its terms with the defect rule of Section 6.1 of `docs/research/orbifold-notation.md` suggests so. That comparison is not a statement of any source.
19. A bounded enumeration that reaches its limit: closed on the handbook's authority. The handbook states that a run that runs out of space shows nothing about the index, "So, we learn nothing when this happens" [HOLT2005, p. 149]. Sims states that the program stops without a result when its memory is filled, and names infinite index and exhausted memory as the two reasons that an enumeration does not terminate, but he does not state that such a run shows nothing about the index [SIMS1994, pp. 224 and 247] (Section 7.15). The handbook states it without a proof. Sims's Exercise 5.3 poses a related question on the incomplete table without an answer [SIMS1994, p. 251], and nothing in this document depends on it.
20. Termination of the HLT strategy: closed for the HLT procedures of the two books, and open for SymPy's code. The handbook states that its COSETENUMERATIONR meets Property 5, with lines 9 and 10 for Property 5(i), so Theorem 5.5, which it proves, gives termination for finite index [HOLT2005, pp. 161 and 164]. It explains that implementations without those lines failed to terminate on Ward's examples [HOLT2005, pp. 164 and 173]. The research reads Holt's "not necessarily" and ITC's warning on pure HLT as concerning a version without those lines. Sims states that his HLT satisfies the definition of a coset enumeration procedure, to which his Proposition 1.3, which he proves, applies [SIMS1994, pp. 223 and 227-228]. What remains is whether SymPy's `coset_enumeration_r`, which has the same shape (Section 7.11), equals these procedures in every step. Would close it: a comparison of the code with p. 164 of [HOLT2005], line by line, recorded as such.
21. Mendelsohn's result in its original form: open for the attribution and the original statement only. The statements read are Hulpke's Theorem III.24, which names Mendelsohn and gives a proof idea, Holt's Theorem 5.3, which gives a proof and names no author, ACE's attribution, read through a summary, of the condition to Lemma 1 and Theorem 2 of [NEUBUSER1982], and two proved results that name no author: the handbook's Theorem 5.5, which credits the idea of its proof to Theorem 2 of [NEUBUSER1982] [HOLT2005, p. 161], and Sims's Proposition 1.3 [SIMS1994, p. 223] (Section 7.15). Neither book names Mendelsohn in its account of coset enumeration. Sims lists [MENDELSOHN1964] in his bibliography only [SIMS1994, p. 591]. Would close it: [MENDELSOHN1964] or [NEUBUSER1982].
22. Safety of the first-in-class pruning on partial tables: partly closed. Both books state the rule that SymPy and SymmHub follow, which stops at an undefined entry, and state that the test never discards an ancestor of the first table of a class. The handbook argues it in prose [HOLT2005, pp. 192-193], and Sims states the exact condition and conclusion and justifies them only through Example 9.3 [SIMS1994, pp. 207-208]. Neither book proves it, and neither proves the savings that each names on top of it [HOLT2005, p. 193] [SIMS1994, p. 257] (Section 7.17). SymPy's docstring, Hulpke's prose, the `low_index` note, and SymmHub's comment assert it too. The `low_index` rule compares empty slots as 0, and no source says whether the two rules prune the same branches. The handbook orders tables with undefined entries as 0 but its FIRSTINCLASS stops at them [HOLT2005, pp. 193-194], and it does not discuss the difference. Would close it: a source that proves the claim for either rule, or Lean.
23. Steps that only code states: mostly closed. The handbook states at step level the definition [HOLT2005, p. 153], the scan and its outcomes [pp. 153 and 155], scan-and-fill [p. 163], the deduction stack and its processing [pp. 165-166], the coincidence queue [pp. 157-158], compression [p. 167], and standardization [pp. 167-168], and proves the effect of a deduction and of coincidence processing on its Properties 1 to 4 [pp. 153-154 and 158-159]. Sims states the two-sided trace, JOIN, COINCIDENCE with union-find, the deduction stack, and standardization, with proofs for the first four [SIMS1994, pp. 176-196 and 233-235] (Sections 7.10 to 7.14). These remain open:
    - Whether the deductions that SymPy's `scan_check` writes without pushing them on the stack can leave a short relator unchecked in the low-index search (Section 7.17). The handbook does not display SCANCHECK and does not say whether its deductions are pushed [HOLT2005, pp. 191-192]. Sims's search states that the deductions of the traces of the subgroup generators are added to the deduction stack, and that the search pushes the deductions of the short-relator traces is the research's inference from its reference to the Felsch strategy [SIMS1994, pp. 234 and 254].
    - Whether it matters that SymPy's Felsch loop scans every popped deduction with the lists of its caller (Section 7.12). Both books select the lists by the popped letter [HOLT2005, p. 166] [SIMS1994, p. 234], so the code departs from both, and neither discusses the variant.
    - That the handbook's COMPRESS, Sims's COMPRESS, and SymPy's `compress` give the same table. The handbook gives no proof or example for COMPRESS, Sims prints his in an Appendix with an example, and no source read compares the three (Section 7.14).
    - The steps of the modified DEFINE, SCAN, and COINCIDENCE of the handbook's Felsch procedure, which it states in prose only [HOLT2005, p. 165].

    Would close it: a source that discusses these variants, or Lean.
24. SymmHub's standard form and its agreement with GAP: whether the tables of SymmHub's search are standardized in the lenlex sense of GAP, as the comment at `sublib_core.js`, lines 21-27, claims, and whether its agreement with GAP's representatives is about 99%, as the comment at lines 31-33 states, or 97%, as `lib/sublib/README.md`, line 310, states (Section 7.19). Both figures are claims of the project, and the research ran no comparison. The research's own argument that the search builds standardized tables is not a statement of the code. Would close it: the comparison with the GAP files that the README describes (section `Compatibility with the GAP-generated files`), run by the research. Neither book addresses SymmHub. Sims's definition of the standard is the one that the comment states (Section 7.14).
25. Group orders and indices of the worked examples that no source states: mostly closed. The handbook states the order 24 of $\langle x, y \mid x^2, y^3, (xy)^4 \rangle$ of the SymPy low-index doctest [HOLT2005, p. 194], the indices and orders of the groups of SymPy's Examples 5.1 to 5.4 [HOLT2005, pp. 150, 154, 156, 160, and 172], and the index 5 of Holt's last example for the same presentation and subgroup [HOLT2005, p. 177] (Section 7.20). These remain open: the index of ACE's Appendix B.3, whose group Sims's Exercise 2.3 poses without an answer [SIMS1994, p. 232], and that GAP's example of Section 47.10-1 uses the group of Section 47.5. Would close it: the ACE manual read directly, or a computation by an independent implementation recorded as such.
26. Lemma III.26 of [HULPKE2010] for the lenlex standard: stated, without a stand-alone proof. Hulpke states the bijection between subgroups of index $n$ and standardized tables for his definition, which ignores the inverse columns. The handbook states it for its standard, which uses all columns, as Proposition 5.6, and justifies it by the prose before it [HOLT2005, p. 189]. Sims states it in one sentence for his lenlex standard, which GAP follows [SIMS1994, p. 252], and proves or leaves as exercises the results it rests on (Section 7.17). Neither book gives a stand-alone proof. Would close it: a source with a stand-alone proof, or Lean.
27. The count of `low_index` for the free group of rank 3: a summarizing tool reported 653 results for the README example at index 4, and the number does not appear in the text that the tool quoted (Section 7.3). Neither book addresses it. Sims's Proposition 9.3 counts complete standard tables, which are subgroups and not classes, for another group only [SIMS1994, p. 207]. Would close it: the README read directly, or a run of the package.
28. Valid colourings and side generators: whether, in the coset colouring of a fundamental-domain tiling by a subgroup $S$, two tiles that share a side have the same colour exactly when the generator of that side lies in $S$, so that no two tiles that share a side have the same colour exactly when $S$ contains no side generator, which in a coset table is no side generator that fixes the coset of $S$. The closest sourced statement is that "c(fF) = c(f'F) is equivalent to f and f' being in the same left coset", proved for $T_{p,q}$ under $G_{p,q}$ only [FRETTLOH2008, p. 3] (Sections 4.1 and 4.2). No source read states the criterion in terms of the side generators, or for the groups beyond the scope of Lemma 2.1. The project records it as `UNV-35` of `docs/design/poc-plan.md` and checks it on samples. Would close it: a source that states it, or Lean.
29. One valid colouring with 2 colours: whether a fundamental-domain tiling has at most one coset colouring with 2 colours in which no two tiles that share a side have the same colour. No source read states it. The project records it as `UNV-36` of `docs/design/poc-plan.md` and checks it by a test. Would close it: a source that states it, or Lean.
30. Existence and uniqueness of the generator point: that each face of the fundamental region holds exactly one point equidistant from the facets that do not contain the face. Martelli states it as Exercise 30 with no proof in the text [MARTELLI2026, Section 5.3], and Whitehead calls the point "the unique point" without a proof that the research recorded [WHITEHEAD2020, Section 3] (Section 10.1). On the sphere, [CLM1954] and [HAREL1993] fix the point of `p q | r` on the bisector of the angle at $R$ and the point of `p q r |` at the incentre, so that its edges have the same length [CLM1954, p. 407] [HAREL1993, p. 8]. That these are the points of equal distance of Martelli and Whitehead is the research's reading, and no source read states the positions for a hyperbolic triangle. Would close it: a source that proves it, or Lean.
31. The symbols of the regular tiling and its dual: closed for the sphere and the Euclidean plane. [CLM1954] states "p | q 2 = {q, p}" and "{p, q} is q | p 2" (p. 406), and its Table 8 gives `4 | 2 4 = {4, 4}`, `6 | 2 3 = {3, 6}`, and `3 | 2 6 = {6, 3}` (p. 438). The order of the numbers after the bar does not matter, as "p | q r, which is, of course, the same as p | r q" states [CLM1954, p. 406]. The tables of [HAREL1993] and [CLM1954] print the 2 first (Section 10.3). That only the mirror opposite the generator point is active is the research's combination of these positions with the ring rule of [CBGS2008, p. 389] (Section 10.2). The use of the same symbols for a hyperbolic tiling such as `{7,3}` rests on item 10.
32. The table of the forms of the `(p q 2)` family: closed for the sphere and the Euclidean plane, open for the hyperbolic plane. [CLM1954] states the symbol and the faces at a vertex of each form for general $p$ and $q$ (pp. 403-415), Tables 5 to 7 of [HAREL1993] and Table 7 of [CLM1954] give the convex polyhedra, Table 8 of [CLM1954] gives the Euclidean tessellations, and the figure on p. 291 of [CBGS2008] labels the vertex figures with $p$ and $q$ (Section 10.3). The conflict on `rr{7,3}` is judged against [CLM1954] (Section 13). What remains open is the `(7 3 2)` members and every other hyperbolic family, which rest on Wikipedia only. The section of Chapter 19 of [CBGS2008] that the infoboxes cite gives counts and pictures and no list of forms (pp. 261-266). Would close it: Chapter 10 of [COXETER1999], or another primary source that lists the forms of a hyperbolic family.
33. Closed forms of the generator point: its trilinear or barycentric coordinates for a general triangle `(p q r)` in all three geometries, and a formula that writes it as a sum of the active distances times the dual basis of the mirror normals, through the inverse of the Gram matrix. A reader of the research derived the second from the solve of [ZHAO-HELPERS], and no source states it. mattz's two shaders and knighty's shader use weights on the corners without a derivation (Section 10.6.2). [CLM1954] and [HAREL1993] give no coordinates for the point of `p q | r` or `p q r |`. [HAREL1993] solves for the angles at the point on the sphere by a Newton iteration (pp. 2-5), and [CLM1954] gives an equation only for the snub (pp. 420-421). Would close it: a source that states a formula, or Lean.
34. The distances in the linear solve: that the uniform construction sets every active entry of `d` in the solve of [ZHAO-HELPERS] to one constant, so that the solve gives the well-positioned point of [MARTELLI2026, Proposition 31]. The docstring asks only for "non-negative floats", and no source joins the two. [CLM1954] and [HAREL1993] state the condition as edges of equal length (item 30), not as distances to the mirrors. What quantity each entry of `d` is in each geometry, a distance or another function of it, is also not stated, because the docstring calls `d` a distance vector and the code solves a linear system in it. Would close it: a source that states the relation, or Lean.

### 14.2. Claims Resting on Secondary Sources or Inference

- Section 3.3: the Waerden-Burckhardt framework [WP-POLYCHROMATIC].
- Section 6: the possible counts of index-2 subgroups [WP-INDEX], and the 46 two-colour types [ARTLANDIA]. The number 46 is also the count of the rows of Table 11.1 of [CBGS2008] by the readers of this research, and the book states no total (Section 14.1, item 6).
- Sections 6 and 8.3: the reading of a parity colouring, such as the checkerboard preset of SymmHub, as the colouring by the two cosets of $\Gamma^+$ is an inference. [SINGERMAN1974] states the subgroup $\Gamma^+$ and its signature but says nothing about colourings, and [CBGS2008] does not use the term parity colouring.
- Section 7.1: the statement on Magma, from a search summary.
- Section 7.3: the meaning of each result of `permutation_reps`, inferred from examples, and the README of `low_index`, read through a summarizing tool.
- Sections 7.13, 7.15 to 7.17, 7.20, and 7.21: the statements of [ACE], [ITC], [MAGMA], and [SYMPY], read through a summarizing tool.
- Sections 7.10 to 7.14 and 7.17: the steps that SymPy's code states and neither book states, which are the code's own (Section 14.1, item 23).
- Sections 7.9 and 7.15: the correspondence of Holt's (R1) to (R4) and Theorem 5.3 with the handbook's Properties 2, 4, 3, and 5 and Theorem 5.5, which is the research's comparison of the two texts.
- Sections 7.8 to 7.21: the passages of [HOLT2005] and [SIMS1994] that the research read only through their text layers, which Section 15 lists, and the symbols that it restored in their quotations.
- Section 7.14: the reading of the loop of the handbook's SWITCH over $X$ as a misprint for $A$.
- Section 7.15: the reading of "Propositions 1.3 and 1.4" on p. 228 of [SIMS1994] as a misprint or a loose reference.
- Section 7.17: that the low-index search of [SIMS1994] pushes the deductions of the traces of the short relators, inferred from its reference to the Felsch strategy.
- Section 7.20: the reading of the order 24 in Example 5.6 of [HOLT2005] as a misprint, the reading of the relators of Example 5.5 of Chapter 5 of [SIMS1994] from the text layer, and the comparison of Sims's Example 6.1 with Brown's table.
- Section 7.16: that SymPy's limit counts dead rows, which the research read off the code.
- Section 7.19: the claims of SymmHub's comments and README that its conventions are those of GAP, that its canonicity test is safe on partial tables, and that its representatives agree with GAP's for about 99% or 97% of subgroups.
- Section 7.20: the reading of the entry in row 5 of Hulpke's Example III.23 as a misprint and the reading that GAP's example of Section 47.10-1 uses the group of Section 47.5.
- Section 7.6: the scope of the theorem of Edmonds, Ewing, and Kulkarni, through [BCCC2025] and a search summary.
- Section 8.4: the reason that SymmHub's code comment gives for the reverse composition, which is the code's own claim.
- Section 9.1: the meanings of the indices in Tactile's `getColour`, inferred from code.
- Section 9.2: the work of Kolbe and Evans, from a search summary.
- Section 10.1: that the bisector and the incentre of [CLM1954] and [HAREL1993] are the points of equal distance of [MARTELLI2026] and [WHITEHEAD2020], which is the research's reading (Section 14.1, item 30). The remark that no point lies on all three mirrors, from Wikipedia only.
- Section 10.2: the rings of each Wythoff symbol, which combine the positions of [CLM1954] with the ring rule of [CBGS2008, p. 389], and the table of faces at a corner, which restates the statements of [CLM1954] and [HAREL1993] for each symbol by the mirrors through each corner. Both are the research's. The use of either for a hyperbolic triangle reads a statement made for the sphere as holding there (Section 14.1, item 10).
- Section 10.3: the `(7 3 2)` members, from Wikipedia only, and the research's comparison of them with the general symbols of [CLM1954]. The join of the forms of [CLM1954] with the archifold symbols of [CBGS2008] and of the rows of Table 8 with the names and face codes of [CBGS2008, p. 288], both by name, is the research's. The vertex figures of [CBGS2008, p. 291] were read from drawings, and the rows of Table 8 of [CLM1954] from a page printed turned, where the last rows may be misaligned.
- Section 10.5: the remarks on quadrilateral domains, from Wikipedia only.
- Section 10.6.2: the generator point by a linear solve, from the code of [ZHAO-HELPERS] and [ZHAO-TILING] only.
- Section 10.6.3: the assignment of a folded point to a face, from the code of two shaders by one author only, the spherical one read from third-party copies [MATTZ-WYTHOFF] and the hyperbolic one from the copy in pywonderland through a summarizing tool [ZUCKER]. That the shaders cut the triangle into up to three regions, one for each face around a corner, is the research's reading of the code.
- Section 10: the quotations of [COXETER1973], which come from the text layer of an OCR scan through a summarizing tool, and of the arXiv preprints [SCHULTE2016], [MARTELLI2026], [WHITEHEAD2020], [MONSON2011], [DDS2004], and [DATTA2018], read through the same tool. The research did not record whether any of them appeared in a refereed journal.
- Section 10: the quotations of [CLM1954] and [CBGS2008], transcribed by eye from page images, with the small-print fractions of [CLM1954], and those of [HAREL1993], from a text layer whose Greek letters and stacked fractions a script rebuilt. The page images of [HAREL1993] were not viewed.
- Section 11: the rows for Morenaments and iOrnament, from search summaries, and the paraphrase for Kali.
- The source [ARXIV1806.06911]: its author and title were not recorded.
- The quotations from web sources throughout passed through a fetch tool that condensed the text and capped quotations at about 125 characters (Section 7.8), except those from [FRETTLOH2008], [KAPLAN2019], and [BULATOV2014], which come from the full PDFs, and those from [CLM1954], [HAREL1993], and [CBGS2008], which come from the copies that Section 15 describes. This includes the other quotations and code fragments of Section 10, none of which was checked character by character against its source.

### 14.3. Sources Not Read in Full

These sources were not read in full and are recorded in `docs/research/source-requests.md`, with what the project needs from each:

- [CBGS2008]: read in part, for the chapters on colourings and the passages cited here. Of Chapters 19 to 26, the research read Chapter 19 in full (pp. 251-266), pp. 269-273 and 277-278 of Chapter 20, pp. 283-292 of Chapter 21, pp. 389-390 of Chapter 26, and the index (pp. 423-426). It did not read pp. 274-276 and 279-281 of Chapter 20, pp. 293-300 of Chapter 21, with the three-dimensional tables, or the rest of Chapters 22 to 26. No open item of this document waits on those pages.
- [GS1987]: chapter 8 on coloured patterns and chapter 6 on isohedral types.
- [EEK1982], [BURNS1983], and [IZQUIERDO1993]: the theorems on achievable indices. Needed for item 8.
- [SCHWARZENBERGER1984]: Euclidean perfect colourings and its Section 8 on colourings that share a colour group. Needed for items 3 and 15.
- [DLP2006] and [DPFP2007]: the hyperbolic coset-colouring framework and index-2 subgroups of hyperbolic symmetry groups.
- [LPD2008] and [DDL2009]: the colouring method for index 2 and the index-$n$ colouring theorem. Needed for item 16.
- [NEUBUSER1982]: Lemma 1 and Theorem 2, which state Mendelsohn's condition, and the accessible description of the method of low-index subgroups by forced coincidences, for which the handbook refers the reader to this paper. Needed for item 21. Medium priority.
- [MENDELSOHN1964]: the original termination result. Needed for item 21. Medium priority.
- [CDHW1973]: the implementations of HLT, Felsch, and lookahead that [HAVAS1991] calls "a primary reference", and the presentations of its Table 1, which the examples of [HAVAS1991] and SymPy use. Medium priority.
- [CONDER2005]: applications and adaptations of the low-index procedure. Medium priority.
- [TODD-COXETER1936]: the original method. Low priority, historical.
- [HAVAS-RAMSAY2000]: a case study in coset enumeration, cited by [CHR2007]. Low priority.
- [COXETER1973]: Sections 5.7, pp. 86-88, and 11.6 and 11.7, pp. 196-201, read only in short fragments of an OCR text layer through a summarizing tool. The graphs of Section 5.7 are garbled in that layer. [CLM1954] and [CBGS2008] now carry the claims that it corroborates, so no open item waits on it, and it is not in `docs/research/source-requests.md`.
- [COXETER1999], Chapters 3 and 10, and Coxeter's "Wythoff's construction for uniform polytopes", Proc. London Math. Soc., 1935: not read. Needed for items 10 and 32, for the hyperbolic plane.
- [CKPS1995]: the face rule that [WHITEHEAD2020] credits to it. Not read. Needed for item 10.
- Coxeter and Moser, "Generators and Relations", Coxeter's "Regular Complex Polytopes", Johnson's "Geometries and Transformations" of 2018, Johnson's thesis of 1966, Section 8.2, pp. 226-237, and Shephard's "A construction for Wythoffian polytopes", Canad. J. Math.: primary texts on the construction that the research could not read. A PDF of Johnson's book could not be extracted, and Cambridge Core returned HTTP 403 for Shephard's paper. They may bear on items 10, 30, and 33.
- [ZUCKER]: the file `main.frag` was read in full through a summarizing tool. The file that defines its helpers and flags was not read for this document, and the Shadertoy originals 3tsSzM and wtj3Ry returned HTTP 403. The helpers are needed for item 11. They are free to read in pywonderland, so they are not a request, and `docs/research/rendering.md` cites `common.frag` of the same directory for `reflectPG` and its distances.
- [KALEIDOTILE] and [NELSON2019]: per-pixel rendering of Wythoff tilings. Jeff Weeks's documentation of KaleidoTile was not retrieved. Needed for item 11.
- knighty's posts on fractalforums: not retrieved. A lead for item 11.
- [BALOGLOU]: two-colour wallpaper patterns. Needed for item 6.
- [SENECHAL1988] and [COXETER1986]: conventions and the historical method.
- [LUBOTZKY2003]: the primary text for Hall's formula.
- [DRESS]: the axioms of Delaney-Dress symbols in general dimension. [BALKE1996] was read in full and gives the two-dimensional case (Section 9.2).
- [KAPLAN2009]: the colouring of isohedral tilings.
- [VONGAGERN2009]: the per-pixel method.
- The paper by Conway and Huson on orbifold notation (2002), held locally by the project and not read.
- [TESS]: the tool's documentation.

The owner supplied [CLM1954] and [HAREL1993], and both were read in full:

- [CLM1954]: every page of the scan, pp. 401-450 and Plates 1 to 6, read as page images. Tables 7 and 8 are printed turned by 90 degrees. The vertex figures of Table 7 and of Table 8 are drawings, and Section 10.3 does not transcribe them. The rows of Table 7 for the prisms, the antiprisms, and the star polyhedra (pp. 435-437) were not transcribed.
- [HAREL1993]: all 46 pages of a typescript, through its text layer. The page images were not viewed, so the layout was not checked. The rows of Tables 6 and 7 for the non-convex polyhedra were not transcribed.

## 15. References

Every web address was accessed on 2026-11-15, except those of the sources of Sections 7.8 to 7.21, which were accessed on 2026-11-17, and those whose entry gives the access date 2026-11-23. Page numbers of [FRETTLOH2008] refer to arXiv:0807.4630v1.

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
- [CBGS2008] Conway, Burgiel, and Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1-56881-220-5, <https://www.routledge.com/The-Symmetries-of-Things/Conway-Burgiel-Goodman-Strauss/p/book/9781568812205>. Read in part, from a copy that the owner supplied. Chapters 19 to 21 and pp. 389-390 of Chapter 26 were read as page images rendered from a scan without a text layer and transcribed by eye, and Section 14.3 lists the pages. Page numbers are the printed pages, which for these chapters are the PDF pages minus 17.
- [CDHW1973] Cannon, J. J., Dimino, L. A., Havas, G., and J. M. Watson, "Implementation and analysis of the Todd-Coxeter algorithm", Math. Comp. 27(123) (1973) 463-490. Not read. The web pass found two conflicting DOIs and verified neither.
- [CHR2007] Campbell, C. M., Havas, G., and E. F. Robertson, "Addendum to an elementary introduction to coset table methods in computational group theory", in Groups St Andrews 1981, London Mathematical Society Lecture Note Series 71, pp. 361-364, dated 2007 in the reference list of [BROWN2011]. Read in full from a local copy, whose running headers give pp. 362-364 and whose text dates itself after July 2006. The address of the copy is not recorded.
- [CKPS1995] Champagne, Kjiri, Patera, and Sharp, Can. J. Phys. 73 (1995) 566-584, as [WHITEHEAD2020] cites it. Title not recorded. Not read.
- [CLM1954] Coxeter, H. S. M., Longuet-Higgins, M. S., and J. C. P. Miller, "Uniform Polyhedra", Philosophical Transactions of the Royal Society of London, Series A, Mathematical and Physical Sciences 246(916), pp. 401-450, received 25 June 1953, published 13 May 1954, DOI 10.1098/rsta.1954.0003 as the web pass recorded it. Read in full from a scan that the owner supplied, a JSTOR copy of 57 pages without a text layer, as page images. The copy was not checked for a printed DOI. An image-only scan is also at <https://roice3.org/ruen/1954_uniform_polyhedra_paper.pdf>. Page numbers are the journal pages, which are the PDF pages plus 399. The fractions in small print were read at 300 dots per inch or more.
- [CONDER2005] Conder, M. and P. Dobcsanyi, "Applications and adaptations of the low index subgroups procedure", Math. Comp. 74(249) (2005). Not read. The title is as the web pass recorded it. [SYMPY-FPG], lines 770-771, spells it "Adaptions".
- [COXETER1973] Coxeter, H. S. M., "Regular Polytopes", 3rd edition, Dover, 1973, Section 5.7, p. 86, Section 11.6, pp. 196-198, and the historical remarks of Chapter XI, <https://archive.org/stream/regular-polytopes-coxeter/Regular%20Polytopes%20-%20Coxeter_djvu.txt>. Primary. Read only through the OCR text layer of a scan, through a summarizing tool, in short fragments. Accessed 2026-11-23.
- [COXETER1986] Coxeter, "Coloured symmetry", in "M. C. Escher: Art and Science", North-Holland, 1986, pp. 15-33, ISBN 978-0-444-70011-7.
- [COXETER1999] Coxeter, H. S. M., "The Beauty of Geometry", 1999, Chapters 3 and 10, as the Wikipedia pages of Section 10.3 cite it. Not read.
- [DATTA2018] Datta and Gupta, "Uniform tilings of the hyperbolic plane", arXiv:1806.11393, 2018, Section 1. Preprint. Read through a summarizing tool. Accessed 2026-11-23.
- [DDS2004] Deza, Dutour, and Shpectorov, "Hypercube embedding of Wythoffians", arXiv:math/0407527, 2004, v5 2008, Section 3. Preprint. Read through a summarizing tool. Accessed 2026-11-23.
- [DDL2009] De Las Penas, Decena, and Laigo, arXiv:0906.3412, 2009-06-18, abstract. Title and address not recorded.
- [DLP2006] De Las Penas, Felix, and Laigo, "Colorings of hyperbolic plane crystallographic patterns", Z. Kristallogr. 221(10) (2006) 665-672, DOI 10.1524/zkri.2006.221.10.665, <https://api.crossref.org/works/10.1524/zkri.2006.221.10.665>.
- [DPFP2007] De Las Penas, Felix, and Provido, "On index 2 subgroups of hyperbolic symmetry groups", Z. Kristallogr., 2007.
- [DRESS] Dress, 1984, and Dress and Huson, 1987, on Delaney-Dress symbols, as cited in [TEGULA2020]. Titles not recorded.
- [EEK1982] Edmonds, Ewing, and Kulkarni, Invent. Math. 69 (1982) 331-346, DOI 10.1007/BF01389358. Title not recorded.
- [EEK1982-BULL] Edmonds, Ewing, and Kulkarni, announcement, Bull. Amer. Math. Soc. 6 (1982) 456-458, DOI 10.1090/S0273-0979-1982-15014-5.
- [EGAN] Egan, G., "Wythoff (Technical Notes)", undated, spherical triangles only, <https://gregegan.net/APPLETS/26/WythoffNotes.html>. Secondary. Accessed 2026-11-23.
- [EPINET-DD] EPINET, "Delaney-Dress symbols", <https://epinet.anu.edu.au/page/epinet2_mathematics_delaney_dress>.
- [EPINET-ORB] EPINET, "Orbifold notation", <https://epinet.anu.edu.au/page/epinet2_mathematics_orbifold_notation>.
- [EWS] EPFL, "Escher Web Sketch", <https://www.epfl.ch/schools/sb/research/iphys/teaching/crystallography/escher-web-sketch/>.
- [FELIX2008] Felix, R. P. and M. J. C. Loquias, "Enumerating and identifying semiperfect colorings of symmetrical patterns", Z. Kristallogr. 223 (2008) 483-491, DOI 10.1524/zkri.2008.0053, arXiv:1002.0536v1, <https://ar5iv.labs.arxiv.org/html/1002.0536>.
- [FREIRE] Freire, P., "9. The Wythoff construction", Paulo's polytope site, undated, <https://www3.mpifr-bonn.mpg.de/staff/pfreire/polyhedra/wythoff.htm>. Secondary. It cites [COXETER1973] and Wenninger's book of 1974. Accessed 2026-11-23.
- [FRETTLOH2008] Frettloh, D., "Counting perfect colourings of plane regular tilings", Z. Kristallogr. 223(11-12) (2008) 773-776, DOI 10.1524/zkri.2008.1044, arXiv:0807.4630v1, 2008-07-29, <https://ar5iv.labs.arxiv.org/html/0807.4630>.
- [GAP] The GAP Group, "GAP Reference Manual", Chapter 47, "Finitely Presented Groups", <https://docs.gap-system.org/doc/ref/chap47.html>, read in full from a local copy, and Section 39.19-6, read through a summary. The local copy states no release or date. The web pass read "GAP Release 4.16.1", dated 2026-08-23, on the front page of the manual through a summarizing tool, so the release is unverified. Line numbers refer to the local copy.
- [GS1987] Grunbaum and Shephard, "Tilings and Patterns", Freeman, 1987, ISBN 978-0-7167-1193-3, and Dover, 2nd edition, 2016, ISBN 978-0-486-46981-2. Chapter 8 is titled `Colored Patterns and Tilings`.
- [HALL1949] Hall, M., Canad. J. Math. 1 (1949) 187-190.
- [HAREL1993] Har'El, Z., "Uniform Solution for Uniform Polyhedra", Department of Mathematics, Technion - Israel Institute of Technology, Geometriae Dedicata 47, pp. 57-110, 1993, as the footnote of its first page prints it, DOI 10.1007/BF01263494 as the Springer record gives it. MathWorld cites 10.1007/BF01263495 (Section 13). Read in full from a copy that the owner supplied, a typescript of 46 pages with a text layer, which prints no DOI and no publisher. Page numbers are the typescript's own, printed as "-N-" from p. 2 and equal to the PDF pages, and they are not the journal's pages, which the copy does not print. The text was extracted by a script, which rebuilt the Greek letters from the codes of a symbol font and the stacked fractions from the positions of their glyphs. The page images were not viewed. The free copies at `math.technion.ac.il/~rl/docs/uniform.pdf` and `harel.org.il` failed their TLS certificate checks on 2026-11-23.
- [HAVAS1991] Havas, G., "Coset enumeration strategies", Proceedings of ISSAC '91, ACM Press, pp. 191-199, 1991, arXiv:math/9406202v1, 1994-06-15, <https://arxiv.org/abs/math/9406202>. Read in full from a local copy of the arXiv version. Page numbers are the printed pages, which are the PDF pages plus 190.
- [HAVAS-RAMSAY2000] Havas, G. and C. Ramsay, "Proving a group trivial made easy: a case study in coset enumeration", Bull. Austral. Math. Soc. 62 (2000) 105-118, as [CHR2007] cites it. Not read.
- [HOLT2005] Holt, D. F., Eick, B., and E. A. O'Brien, "Handbook of Computational Group Theory", Chapman and Hall/CRC, 2005, ISBN 978-1-58488-372-2, DOI 10.1201/9781420035216. Read from a PDF copy that the owner supplied: Chapter 5, pp. 149-198, and pp. 18, 22-24, 37, 43, and 84-87. The text layer drops Greek letters, superscripts, set symbols, and most presentations, so the research checked the pages with pseudocode, symbols, or tables against page images, with the exceptions that follow. Pages 150-158, 160, 161, 163-168, 172-175, 177, 178, 182-184, 187, 191-194, and 196 were checked against images, and pp. 149, 159, 162, 169-171, 176, 179-181, 185, 186, 188-190, 195, 197, and 198 were read from the text layer only. The text-layer pages include the pseudocode REWRITE and the table of Example 5.5 on p. 176 and the tables of Example 5.10 on p. 195. The presentations of the costs on pp. 169-170 and of the example of order 16 on p. 172 were not read from the images. The text layer of the whole book was searched for "Mendelsohn", which does not occur. Page numbers are the printed pages, which are the PDF pages minus 14.
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
- [KNIGHTY-FRAG] knighty, Fragmentarium, "Knighty Collection", `hyperbolic-tesselation-2Din3D-colored-02.frag`, undated, `https://raw.githubusercontent.com/Syntopia/Fragmentarium/master/Fragmentarium-Source/Examples/Knighty%20Collection/hyperbolic-tesselation-2Din3D-colored-02.frag`. Source code. Licence not recorded. Accessed 2026-11-23.
- [KOLBE-EVANS] Kolbe and Evans, Geom. Dedicata, DOI 10.1007/s10711-020-00554-2. Title and year not recorded.
- [LOWINDEX] Culler, M., Dunfield, N., and M. Gorner, `low_index`, README, <https://github.com/3-manifolds/low_index>, read through a summarizing tool, and "LOW INDEX, Revisited and Implemented", the LaTeX source `doc/lowindex.tex`, <https://raw.githubusercontent.com/3-manifolds/low_index/master/doc/lowindex.tex>, read in full from a local copy. The file states no date. The web pass found commits of the file authored from 2022-06-22 to 2022-07-01. Sections are those of the note.
- [LPD2008] Laigo, Puzon, and De Las Penas, Proceedings of Bridges Leeuwarden 2008, pp. 311-318, 2008, titled `Coxeter Groups in Colored Tilings and Patterns`, <https://archive.bridgesmathart.org/2008/bridges2008-311.html>.
- [LUBOTZKY2003] Lubotzky and Segal, "Subgroup Growth", Birkhauser, 2003.
- [MAGMA] Magma Handbook, low index subgroups, <https://magma.maths.usyd.edu.au/magma/handbook/text/898>, and an older mirror of its pages on coset enumeration and low index subgroups, <https://www.math.ru.nl/magma/text237.html>, `text239.html`, and `text243.html`, version not stated. Read through a summarizing tool.
- [MENDELSOHN1964] Mendelsohn, N. S., "An algorithmic solution for a word problem in group theory", Canad. J. Math. 16 (1964) 509-516, with a correction in 17 (1965) 505, as the bibliography of [SIMS1994], p. 591, lists it. Not read. Cited as "Mendelsohn, 1964" by [HULPKE2010], p. 77, without an entry.
- [MALINC] Christersson, M., "Image Tiling", 2015-04-19, <https://www.malinc.se/m/ImageTiling.php>.
- [MARTELLI2026] Martelli, B., "An introduction to Coxeter polyhedra", arXiv:2601.07552, v2 of 2026-01-16 and v3 of 2026-05-01, Sections 1.6 and 5.3 to 5.5, Exercise 30, Proposition 31, and Proposition 38, <https://arxiv.org/html/2601.07552>. Preprint, in the style of lecture notes. Read through a summarizing tool. Accessed 2026-11-23.
- [MATEI2002] Matei, D. and A. Suciu, "Hall invariants, homology of subgroups, and characteristic varieties", Int. Math. Res. Not. 2002, arXiv:math/0010046, Section 8.1, eq. (8.2), read through its ar5iv rendering.
- [MATTZ-WYTHOFF] mattz (Zucker, M., as for [ZUCKER]), "Wythoff explorer", Shadertoy Md3yRB, CC BY-NC-SA 3.0, functions `construct()` and `tile_sphere`, and the file `common.glsl`. Source code for the sphere and three-dimensional polyhedra. Not read on Shadertoy. Read from two third-party copies: `ShaderOfWeek/WythoffExplorerB.glsl` of ruofeidu/DuEngine at commit `101cf4a`, 2018-03-07, <https://github.com/ruofeidu/DuEngine/blob/master/ShaderOfWeek/WythoffExplorerB.glsl>, and a gist by andybak of 2018-08-18, <https://gist.github.com/andybak/e0ac5557362c4611fc4d9a4b1591189e>. Its header credits knighty's "Polyhedron again", Shadertoy XlX3zB. Accessed 2026-11-23.
- [MONSON2011] Monson and Schulte, "Semiregular Polytopes and Amalgamated C-groups", arXiv:1109.1337, 2011, Section 3. Preprint. Address not recorded. Read through a summarizing tool. Accessed 2026-11-23.
- [MORENAMENTS] "Morenaments", exhibit sheet, <https://exponate.phaeno.de/media/pages/experimente/4916/f19b5bd2f0-1714643520/4916_Morenaments_en.pdf>.
- [NELSON2019] Nelson, R., slides, ICERM, 2019. Title not recorded.
- [NEUBUSER1982] Neubuser, J., "An elementary introduction to coset table methods in computational group theory", in Groups St Andrews 1981, London Mathematical Society Lecture Note Series 71, Cambridge University Press, 1982, pp. 1-45. Not read.
- [POLYTOPE-WIKI] Polytope Wiki, "Wythoff symbol", <https://polytope.miraheze.org/wiki/Wythoff_symbol>. Secondary. It cites [HAREL1993]. Accessed 2026-11-23.
- [SCHULTE2016] Schulte, E. and A. Williams, "Wythoffian Skeletal Polyhedra in Ordinary Space, I", arXiv:1610.03168, 2016, Sections 3.2 and 4.2, <https://ar5iv.labs.arxiv.org/html/1610.03168>. Preprint. Read through a summarizing tool. Accessed 2026-11-23.
- [SCHWARZENBERGER1984] Schwarzenberger, R. L. E., "Colour symmetry", Bull. London Math. Soc. 16 (1984) 209-240.
- [SENECHAL1988] Senechal, Comput. Math. Applic. 16 (1988) 545-553, DOI 10.1016/0898-1221(88)90244-1, titled `Color symmetry`.
- [SIMS1994] Sims, C. C., "Computation with Finitely Presented Groups", Encyclopedia of Mathematics and its Applications 48, Cambridge University Press, 1994, ISBN 978-0-521-43213-7, DOI 10.1017/CBO9780511574702. Read from a PDF copy that the owner supplied, through its text layer: Section 3.8, pp. 130-140, Sections 4.1, 4.2, 4.4 to 4.7, and 4.9, pp. 151-165, 171-196, and 203-210, Chapter 5, pp. 217-267, the Appendix, pp. 576-577, and the bibliography entries for Mendelsohn on p. 591. Pages 195, 205-207, 209, 223, 228, 248, and 255-257 were also read from page images, for the matrices, the tables, and the definition of a coset enumeration procedure. Pages 166-170 and 197-202 were not read, and Sections 4.10 and 4.11, pp. 210-215, were only skimmed. The text layer turns Greek letters into Latin ones, flattens exponents, and loses the blank cells of tables, so the positions of the entries of Tables 4.5.x, 5.1.x, 5.4.x, and 5.7.x are unreliable. Page numbers are the printed pages, which are the PDF pages minus 16.
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
- [WHITEHEAD2020] Whitehead, S., "Classifying Regular Polyhedra and Polytopes using Wythoff's Construction", arXiv:2001.09364, 2020, v2 2021, Sections 3 and 5 and Theorem 9, <https://ar5iv.labs.arxiv.org/html/2001.09364>. Preprint. Read through a summarizing tool. Accessed 2026-11-23.
- [WP-732] Wikipedia, "Order-3 heptagonal tiling", <https://en.wikipedia.org/wiki/Order-3_heptagonal_tiling>, and the pages of the other forms of `{7,3}` that it links, such as "Rhombitriheptagonal tiling", infoboxes only. Accessed 2026-11-23.
- [WP-INDEX] Wikipedia, "Index of a subgroup", <https://en.wikipedia.org/wiki/Index_of_a_subgroup>.
- [WP-POLYCHROMATIC] Wikipedia, "Polychromatic symmetry", <https://en.wikipedia.org/wiki/Polychromatic_symmetry>.
- [WP-UNIFORM-LISTS] Wikipedia, "Lists of uniform tilings on the sphere, plane, and hyperbolic plane", with its table of the `(p q 2)` family, <https://en.wikipedia.org/wiki/Lists_of_uniform_tilings_on_the_sphere,_plane,_and_hyperbolic_plane>. Accessed 2026-11-23.
- [WP-UNIFORM-TILING] Wikipedia, "Uniform tiling", <https://en.wikipedia.org/wiki/Uniform_tiling>.
- [WP-WYTHOFF-SYMBOL] Wikipedia, "Wythoff symbol", section "Description", <https://en.wikipedia.org/wiki/Wythoff_symbol>.
- [ZHAO] Zhao Liang, "Coxeter groups, automata and uniform tilings", blog of pywonderland, dated 2019-12-19 as the reading of 2026-11-23 recorded, <https://neozhaoliang.github.io/uniform-tilings>.
- [ZHAO-HELPERS] Zhao Liang, pywonderland, `src/uniform-tilings/helpers.py`, function `get_point_from_distance`, branch `master`, copyright 2019, <https://raw.githubusercontent.com/neozhaoliang/pywonderland/master/src/uniform-tilings/helpers.py>. Source code. The research recorded no licence for the file, and Section 6.7 of `docs/research/prior-art.md` gives MIT for the repository. Accessed 2026-11-23.
- [ZHAO-TILING] Zhao Liang, pywonderland, `src/uniform-tilings/tiling.py`, branch `master`, 2019, <https://raw.githubusercontent.com/neozhaoliang/pywonderland/master/src/uniform-tilings/tiling.py>. Source code, with the licence of [ZHAO-HELPERS]. Accessed 2026-11-23.
- [ZUCKER] mattz (Zucker, M., only through the credit of [ZHAO]), "Hyperbolic Wythoff explorer v2", CC BY-NC-SA 3.0, the copy in pywonderland, `src/shader-playground/glsl/wythoff_hyperbolic/main.frag`, branch `master`, functions `flipTriangleToPoint`, `computeFace`, and `mainImage`, <https://raw.githubusercontent.com/neozhaoliang/pywonderland/master/src/shader-playground/glsl/wythoff_hyperbolic/main.frag>. Source code. Read in full on 2026-11-23 through a summarizing fetch tool, which returned the whole file of 28,252 characters and kept its quotations short. The quotations were not checked character by character. The header says that the shader "is basically https://www.shadertoy.com/view/wtj3Ry with a few improvements" and does not mention pywonderland. [ZHAO] links Shadertoy 3tsSzM. Neither Shadertoy page was read, because Shadertoy returned HTTP 403. `docs/research/rendering.md` cites the same file under the same tag (Section 13).
