# Colourings of Tilings Associated with a Symmetry Group

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-15 |
| Applies to | The choice of the polygons that the Escher renderer colours, and the computation of their colours and of the available numbers of colours |

## Abstract

This document states what the literature and existing software establish about colouring the tiles of a tiling associated with a planar symmetry group so that every symmetry of the tiling permutes the colours. It gives the definitions of colourings, colour symmetry, and perfect colourings, the lemmas that turn a subgroup of finite index into a perfect colouring and the scope in which each is proved, the published tables of available numbers of colours, the algorithms and software that enumerate subgroups of low index, and the per-pixel colour computation of SymmHub. It then summarizes other tilings associated with a group, the Wythoff construction, and the colouring behaviour of existing tools, and compares three candidate designs for the polygons that Escher draws, limited to what the sources establish. Every claim that no source establishes is listed as an open question.

## 1. Scope

This document covers three candidate designs for the polygons that the first Escher deliverable draws and colours:

- Design (a): the tiling by copies of the fundamental domain, with a perfect colouring.
- Design (b): a fundamental domain cut into several polygons, each with one colour.
- Design (c): a uniform tiling from the Wythoff construction, with one colour per face orbit.

For each design it states what determines the available numbers of colours, the machinery that the colour computation needs, and the gaps that the sources leave open. It makes no recommendation.

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
- $H$: the set of $h \in G$ that appear in a pair of $C(X)$ in Frettloh's notation [FRETTLOH2008], and the stabilizer of a partition in Felix and Loquias's notation [FELIX2008].
- $K$: the symmetry group of the coloured pattern in Frettloh's introduction, the set of group elements whose triangles carry colour 1 in Frettloh's proof of Lemma 3.2 (written $K_1$, with colour classes $K_i$), and the kernel of the map to colour permutations in Felix and Loquias [FRETTLOH2008] [FELIX2008]. Section 2.3 lists this clash.
- $G_{p,q}$: the Coxeter group of the regular tiling `(p^q)`, with generators $a$, $b$, $c$ (Section 5.1).
- $\bar{G}_{p,q}$: the rotation subgroup of $G_{p,q}$ (Section 5.4).
- $G^2$: the subgroup that the source of the index-2 count writes as $G^2$ (Section 6). The research for this document did not record its definition.
- $a_k(G)$: the number of subgroups of index $k$ in $G$ (Section 7.5).
- $h_k(G)$: the number of homomorphisms from $G$ to $S_k$, that is, $|\mathrm{Hom}(G, S_k)|$ (Section 7.5).
- $\rho$: a homomorphism from $G$ to $S_k$, with one permutation $\rho(s_i)$ for each side pairing $s_i$ of $F$ (Section 8.7).
- $\sigma$: the colour permutation that the fold loop carries for one pixel (Section 8.7). The letter $\sigma$ keeps the state apart from a colour permutation $\pi_h$.

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
- low-index subgroup enumeration: The algorithm of Sims that lists the subgroups of a finitely presented group up to a bound on the index, up to conjugacy [GAP, Section 47.10-1] [LOWINDEX].
- representative set: The set of polygons, one from each class of polygons that the Escher renderer colours alike, whose size the first deliverable reports to the user as the available number of colours. This is a project term, and no source defines it.
- face orbit: The set of the images of one face of a tiling under its symmetry group.
- Wythoff construction: The construction of a uniform tiling from a reflection triangle and a generator point inside the triangle or on its boundary (Section 10) [WP-WYTHOFF-SYMBOL].
- isohedral tiling type: One of the 93 classes of tilings whose symmetry group acts transitively on the tiles, in the classification of Grunbaum and Shephard (Section 9.1) [KAPLAN2025].
- Delaney-Dress symbol: The combinatorial description of a periodic tiling that Section 9.2 defines [TEGULA2020, Section 4].

### 2.3. Conventions on Which Sources Differ

- Left or right cosets: Frettloh colours the tile $gF$ by the left coset $gS$ [FRETTLOH2008, p. 3]. Felix and Loquias use a complete set of right coset representatives of $H$ in $G$, with the left action $gP$ on partitions [FELIX2008, Sections 2 and 3]. De Las Penas, Felix, and Laigo use right coset colourings [DLP2006] (abstract summary only). SymmHub offers both orders of composition through `uUseOrbit` (Section 8.4). The order changes the formula that composes permutations along a fold.
- Definition of perfect: Section 3 gives three definitions. They agree that every symmetry of the tiling is a colour symmetry and differ in their framework. Frettloh's definition does not require equally frequent colours (Section 3.1).
- What is counted: Frettloh counts colour symmetry groups up to conjugacy, and Felix and Loquias count colourings up to the action of $G$ (Section 3.4). A table from one source is not comparable with a count from the other without conversion.
- The letter $K$: Frettloh uses $K$ for the symmetry group of the coloured pattern and $K_1$ for the colour class of colour 1, while Felix and Loquias use $K$ for a kernel (Section 2.1).
- Tiling symbols: Frettloh writes `(p^q)` for the regular tiling by $p$-gons with $q$ at each vertex, following Grunbaum and Shephard [FRETTLOH2008, p. 2]. So `(7^3)` is the heptagonal tiling.

### 2.4. Spelling of Names and Symbols

Names with accented letters are written in ASCII transliteration: Frettloh, Grunbaum, De Las Penas, Gorner, Dobcsanyi, Schlafli, Poincare, and Mobius. Orbifold signatures, Schlafli symbols, vertex configurations, tiling symbols, and Wythoff symbols are written as code spans, such as `*642`, `{7,3}`, `3.3.p.3.q`, `(7^3)`, and `p | q r`.

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

These further statements rest on secondary sources only:

- "any non-Fuchsian NEC group has an index 2 Fuchsian subgroup of orientation-preserving elements" [WP-NEC].
- There are exactly 46 types of two-colour symmetries of the plane, after H. J. Woods [ARTLANDIA] (search summary only).

Conway, Burgiel, and Goodman-Strauss classify twofold colourings in chapter 11, "Twofold Colorations", with the sections "Classifying Twofold Plane Colorings" and `Complete List of Twofold Color Types` [CBGS2008], and Grunbaum and Shephard treat coloured patterns in chapter 8 [GS1987]. Neither text was read.

No source read gives a rule that reads the number of index-2 subgroups, and so the number of two-colourings, from an orbifold signature (Section 14.1). Semiperfect colourings (Section 3.2) are a different concept: they concern colourings whose colour symmetry group has index 2, not colourings with two colours.

## 7. Algorithms and Software

### 7.1. Low-Index Subgroup Enumeration

The low-index subgroups algorithm lists the conjugacy classes of subgroups of a finitely presented group whose index is at most a bound. GAP describes its method as "systematically running through a tree of all potential coset tables of G of length at most index", and its running time "will grow exponentially with the value of index" [GAP, Section 47.10-1]. Sims and Schaps discovered the algorithm independently in the 1960s, and it handles indices "up to 100 or even more" in favourable cases [ARXIV-MATH9406203] (search summary only).

Magma's implementation makes "coset definitions ... systematically at each open position, and the relations are then traced" [MAGMA] (search summary only). Conder and Dobcsanyi published on the subject in Math. Comp. 74 (2005) [CONDER2005]. The research for this document recorded no claim from that paper.

### 7.2. GAP

These GAP 4.16.1 functions compute subgroups of low index and their coset tables [GAP]:

- `LowIndexSubgroupsFpGroup( G[, H], index[, excluded] )`: computes "representatives of the conjugacy classes of subgroups of the finitely presented group G that contain the subgroup H of G and that have index less than or equal to index". "If not given, H defaults to the trivial subgroup." With `excluded`, it returns only subgroups that "do not contain any conjugate of any of the group elements defined by these words" (Section 47.10-1).
- `LowIndexSubgroupsFpGroupIterator`: the iterator form of the same computation (Section 47.10-1).
- `LowIndexSubgroups( G, index )`: Section 39.19-6. Its description was read through a summary only.
- `CosetTable( G, H )`: "returns the coset table of the finitely presented group G on the cosets of the subgroup H". "The coset table is standard", which means "the words w_1, w_2, ... are length-plus-lexicographic ordered" (Sections 47.6 and 47.7).
- `FactorCosetAction( G, H )`: "returns the action of G on the cosets of its subgroup H" (Section 47.7).
- `CosetTableFromGensAndRels( fgens, grels, fsgens )` (Section 47.6).

The argument `H` of `LowIndexSubgroupsFpGroup` matches step 3 of Section 5.5, which keeps the subgroups that contain two given generators. Frettloh's paper does not say whether he used it.

### 7.3. The Python Package `low_index`

The package `low_index` by Culler, Dunfield, and Gorner "implements a variant of Charles Sims' Low Index Subgroups algorithm for enumerating all of the conjugacy classes of subgroups of a finitely presented group with finite index less than a given bound" [LOWINDEX, README].

Its entry point is `permutation_reps(rank = 3, short_relators = [], long_relators = [], max_degree = 4)`, which gives 653 classes for the free group of rank 3 up to index 4 [LOWINDEX, README]. Each result is a list of one permutation per generator. That reading is inferred from the examples, not stated.

The accompanying paper says that "our implementation actually outputs a list of permutation representations", "given by the standard right action on the vertices of the cover" [LOWINDEX, `doc/lowindex.tex`]. It reports times of about 12 to 17 s at index 8, against 104 to 118 s for GAP on the same examples, and cites Sims, chapter 5 [SIMS1994] [LOWINDEX].

### 7.4. Other Implementations

- SymPy: `low_index_subgroups(G, N)` returns the conjugacy classes of subgroups up to index `N` [SYMPY] (search summary only).
- SymmHub: `lib/sublib/src/sublib_core.js` implements a "Sims-style low-index coset-table backtrack search, conjugacy classes, GAP-compatible conventions", and `lib/sublib/src/reidemeister.js` implements Reidemeister-Schreier [SYMMHUB]. Section 8.3 describes the catalogue that it generates.

### 7.5. Hall's Counting Formula

Matei and Suciu cite "the following well-known formula of Marshall Hall" [MATEI2002, Section 8.1, eq. (8.2)] [HALL1949]. With $a_k(G)$ the number of subgroups of index $k$ and $h_k(G) = |\mathrm{Hom}(G, S_k)|$, and $a_1 = h_1 = 1$:

$$a_k(G) = \frac{h_k(G)}{(k-1)!} - \sum_{l=1}^{k-1} \frac{h_{k-l}(G)}{(k-l)!} \, a_l(G)$$

The formula is typeset as ar5iv renders the source, and the primary text by Hall was not read. It counts all subgroups of index $k$, not conjugacy classes and not colour symmetry groups up to conjugacy.

### 7.6. Achievable Indices

Edmonds, Ewing, and Kulkarni "determine the possible finite indices of torsion-free subgroups of a Fuchsian group, with respect to its torsion" [BCCC2025, introduction, citing EEK1982]. This rests on a secondary source, and the theorem statement itself was not obtained. They set aside normal subgroups, a question that "appears to involve deeper number theoretic considerations" [EEK1982-BULL] (search summary only).

Burns and Solitar prove some of these results algebraically [BURNS1983], and Izquierdo solved the analogue for NEC groups [IZQUIERDO1993] (both known only by citation, without the statements). A torsion-free subgroup is a special case. No source read gives the set of all achievable indices for an arbitrary orbifold group (Section 14.1).

### 7.7. Practical Limits

- Frettloh: "Without further refinements, this takes weeks using GAP on a usual PC" [FRETTLOH2008, Section 3] (condensed extraction, page not re-checked). "Practically, computing with finitely presented groups is not efficient. Therefore, there is for each (p^q) a limit on the number of colours to be considered" [FRETTLOH2008, p. 4].
- GAP: the time of `LowIndexSubgroupsFpGroup` grows exponentially with the index [GAP, Section 47.10-1].
- `low_index`: the timings of Section 7.3.
- SymmHub: at most 24 colours (Section 8.6).

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

A Delaney-Dress symbol is "a connected (multi-) graph" in which each node is incident to exactly one edge of each of the colours 0, 1, and 2, "together with a labelling of its nodes by two maps m01 and m12", which record the tile degree and the vertex degree [TEGULA2020, Section 4]. The full conditions are deferred to Dress (1984) and Dress and Huson (1987) [DRESS].

The curvature of a symbol $\mathcal{D}$ with labelling $m$ is [TEGULA2020, Section 4, unnumbered display]:

$$\mathcal{K}(\mathcal{D}, m) = \sum_{D \in \mathcal{D}} \left( \frac{1}{m_{01}(D)} + \frac{1}{m_{12}(D)} - \frac{1}{2} \right)$$

The geometry is "spherical, euclidean or hyperbolic, depending on whether the curvature is positive, 0 or negative" [TEGULA2020, Section 4]. In the passage to the orbifold, "reflections are mapped onto boundary segments, rotational centers are mapped onto cones, dihedral centers are mapped onto corners, and glide-reflections give rise to crosscaps", with the full derivation deferred to Balke and Huson [TEGULA2020, Section 4] [BALKE1996].

EPINET describes the system of chambers of a Delaney-Dress symbol as "really just a triangulation of this orbifold", with the example `*246`, a triangle with angles $\pi/2$, $\pi/4$, and $\pi/6$ [EPINET-DD]. Its page on orbifold notation gives the curvature in terms of $o$ handles, $\ast$ mirror boundaries, $x$ cross-caps, cone orders $c_i$, and corner orders $m_j$, "derived from Macbeath's non-euclidean crystallographic group signature" [EPINET-ORB]:

$$K = 2 - 2o - \ast - x - \sum_i \frac{c_i - 1}{c_i} - \sum_j \frac{m_j - 1}{2 m_j}$$

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
- The locator of the spherical counts: a condensed extraction of Frettloh's paper placed the cube and the icosahedron in Sections 1 and 2, and the full PDF places the cube on p. 4. The icosahedron was not re-checked in the PDF.

## 14. Open Questions and Unverified Claims

### 14.1. Open Questions

Each item below is a candidate statement that needs a source or a machine-checked proof before the body can state it.

1. Colourings and permutation representations: whether the perfect colourings of a fundamental-domain tiling with $k$ colours correspond to transitive permutation representations of $G$ of degree $k$, and whether they correspond up to symmetry to conjugacy classes of subgroups of index $k$. Frettloh's paper states neither.
2. The converse of Lemma 2.1: whether every perfect colouring of the fundamental-domain tiling of an arbitrary orbifold group arises from a subgroup, and whether Lemma 2.1 holds for groups with rotations, glide reflections, handles, or cross-caps.
3. The tension in Frettloh's paper between the statement that colourings may share a colour symmetry group and the stated bijection (Section 3.4). Schwarzenberger's Section 8, which Frettloh cites on this point, was not read.
4. The tile centres: a reading of eq. (1) suggests that the `(q^p)` tile is centred at the $\pi/q$ corner of $F$, the `(p^q)` tile at the $\pi/p$ corner, and the `[p.q.p.q]` tile at the right-angle corner. The paper does not state this except for the `(q^p)` tile $F \cup aF \cup abF \cup \dots$.
5. The tile-stabilizer condition beyond regular and Laves tilings: whether a subgroup induces a perfect colouring of a Wythoff tiling exactly when it contains the stabilizer of each face, such as the standard parabolic subgroup $\langle i, j \rangle$ of Zhao Liang's page, and whether it extends to groups other than $G_{p,q}$.
6. The number of two-colourings from a signature: a formula for the dimension over $\mathbb{F}_2$ of $G/G^2[G,G]$ in terms of handles, cross-caps, mirrors, corner orders, and cone orders, and whether $[G : G^2] - 1$ holds for infinite $G$. A table of the index-2 subgroups of the 17 wallpaper groups and the 46 two-colour types would also be needed [CBGS2008].
7. Hall's formula at $k = 2$: substituting into Section 7.5 gives $a_2 = |\mathrm{Hom}(G, S_2)| - 1$. A searcher made this substitution, and no source states it.
8. The theorem of Edmonds, Ewing, and Kulkarni: the exact statement of the achievable indices of torsion-free subgroups, and Izquierdo's version for NEC groups.
9. Achievable indices in general: whether every index occurs for the wallpaper groups, and which indices occur for hyperbolic groups. The only sourced data are Frettloh's tables and Bulatov's examples.
10. The face-orbit rule for Wythoff tilings: a rule such as "a face of size p or 2p around corner p exists if and only if a mirror there is active", and the face list of snubs, with rotation-centre faces and snub triangles and a generator equidistant under the rotations. No source read states either.
11. Per-pixel classification for Wythoff tilings: no method found. Zucker's shader [ZUCKER] and the KaleidoTile source [KALEIDOTILE] are the leads.
12. The Wythoff construction beyond triangles: polygonal kaleidoscopes such as `*2222` and `*abcd`, and mixed orbifolds such as `3*3` and `2*22`.
13. Normal subgroups: whether a subgroup is normal exactly when $G/H$ acts regularly on the colours, and how the kernel $K$ relates to the core of the colour group.
14. Design (b): no source read addresses the colouring of a fundamental domain cut into several polygons, or the numbers of colours it offers.
15. Euclidean perfect colourings: the counts for Euclidean regular and Laves tilings, deferred by Frettloh to [SCHWARZENBERGER1984].
16. The index-$n$ colouring theorem of De Las Penas, Decena, and Laigo [DDL2009]: a correspondence between subgroups of index $n$ and transitive colourings with $n$ colours is known from a search summary only, and the theorem number was not retrieved.
17. Tool details: the face colouring of KaleidoTile, the behaviour of Tess, and the colour features of iOrnament, Morenaments, and Escher Web Sketch.

### 14.2. Claims Resting on Secondary Sources or Inference

- Section 3.3: the Waerden-Burckhardt framework [WP-POLYCHROMATIC].
- Section 6: the possible counts of index-2 subgroups [WP-INDEX], the index-2 Fuchsian subgroup of an NEC group [WP-NEC], and the 46 two-colour types [ARTLANDIA].
- Section 7.1 and Section 7.4: the history of the algorithm, Magma, and SymPy, from search summaries.
- Section 7.3: the meaning of each result of `permutation_reps`, inferred from examples.
- Section 7.6: the scope of the theorem of Edmonds, Ewing, and Kulkarni, through [BCCC2025] and a search summary.
- Section 8.4: the reason that SymmHub's code comment gives for the reverse composition, which is the code's own claim.
- Section 9.1: the meanings of the indices in Tactile's `getColour`, inferred from code.
- Section 9.2: the work of Kolbe and Evans, from a search summary.
- Section 10, except Sections 10.6 and 10.7: Wikipedia only.
- Section 11: the rows for Morenaments and iOrnament, from search summaries, and the paraphrase for Kali.
- The source [ARXIV1806.06911]: its author and title were not recorded.
- The quotations from web sources throughout passed through a fetch tool that condensed the text and capped quotations at about 125 characters, except those from [FRETTLOH2008], [KAPLAN2019], and [BULATOV2014], which come from the full PDFs.

### 14.3. Sources Not Read in Full

These sources were not read in full and are recorded in `docs/research/source-requests.md`, with what the project needs from each:

- [CBGS2008]: chapter 11 on twofold colourings, chapters 12 and 13 on colourings with more colours and their notation, and the chapter on hyperbolic groups. Needed for items 6 and 9.
- [GS1987]: chapter 8 on coloured patterns and chapter 6 on isohedral types.
- [EEK1982], [BURNS1983], and [IZQUIERDO1993]: the theorems on achievable indices. Needed for item 8.
- [SCHWARZENBERGER1984]: Euclidean perfect colourings and its Section 8 on colourings that share a colour group. Needed for items 3 and 15.
- [DLP2006] and [DPFP2007]: the hyperbolic coset-colouring framework and index-2 subgroups of hyperbolic symmetry groups.
- [LPD2008] and [DDL2009]: the colouring method for index 2 and the index-$n$ colouring theorem. Needed for item 16.
- [HOLT2005] and [SIMS1994]: coset enumeration and low-index subgroups.
- [CLM1954] and [HAREL1993]: the primary rules of the Wythoff construction. Needed for item 10.
- [ZUCKER], [KALEIDOTILE], and [NELSON2019]: per-pixel rendering of Wythoff tilings. Needed for item 11.
- [BALOGLOU]: two-colour wallpaper patterns. Needed for item 6.
- [SENECHAL1988] and [COXETER1986]: conventions and the historical method.
- [LUBOTZKY2003]: the primary text for Hall's formula.
- [DRESS] and [BALKE1996]: the full axioms of Delaney-Dress symbols and the derivation of the orbifold from a symbol.
- [KAPLAN2009]: the colouring of isohedral tilings.
- [VONGAGERN2009]: the per-pixel method.
- The paper by Conway and Huson on orbifold notation (2002), held locally by the project and not read.
- [TESS]: the tool's documentation.

## 15. References

Every web address was accessed on 2026-11-15. Page numbers of [FRETTLOH2008] refer to arXiv:0807.4630v1.

- [ARTLANDIA] Artlandia, glossary entry titled `Two-Color Symmetries`, <https://artlandia.com/wonderland/glossary/TwoColorSymmetries.html>.
- [ARXIV-MATH9406203] arXiv:math/9406203, read through a search summary only. Author, title, and address not recorded.
- [ARXIV1806.06911] arXiv:1806.06911, 2018, Section 2, Theorem 2.1. Author and title not recorded, <https://ar5iv.labs.arxiv.org/html/1806.06911>.
- [BALKE1996] Balke and Huson, 1996, on the orbifold of a Delaney-Dress symbol, as cited in [TEGULA2020]. Title not recorded.
- [BALOGLOU] Baloglou, G., "Isometrica", chapter 6, EMIS. Year and address not recorded.
- [BCCC2025] Bujalance, Cirre, Conder, and Costa, arXiv:2502.03594v1, 2025, introduction. Title and address not recorded.
- [BULATOV2010] Bulatov, V., "Coloring a hyperbolic tiling", part of the talk "Artistic Models of the Hyperbolic Geometry", G4G9, 2010-03-25, `https://www.bulatov.org/math/1003/index_color.html`, and the gallery `https://www.bulatov.org/math/1003/color_s542/index.html`.
- [BULATOV2014] Bulatov, V., "Inversive Kaleidoscopes and their Visualization", Proceedings of Bridges 2014, pp. 329-332, 2014.
- [BURNS1983] Burns and Solitar, Proc. Amer. Math. Soc. 89 (1983) 414-418. Title not recorded.
- [CBGS2008] Conway, Burgiel, and Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1-56881-220-5, <https://www.routledge.com/The-Symmetries-of-Things/Conway-Burgiel-Goodman-Strauss/p/book/9781568812205>.
- [CLM1954] Coxeter, Longuet-Higgins, and Miller, "Uniform polyhedra", Phil. Trans. R. Soc. A 246 (1954) 401-450.
- [CONDER2005] Conder and Dobcsanyi, Math. Comp. 74 (2005). Title not recorded.
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
- [GAP] The GAP Group, "GAP Reference Manual", version 4.16.1, 2026-08-23, Sections 39.19-6, 47.6, 47.7, and 47.10-1, <https://docs.gap-system.org/doc/ref/chap47.html>.
- [GS1987] Grunbaum and Shephard, "Tilings and Patterns", Freeman, 1987, ISBN 978-0-7167-1193-3, and Dover, 2nd edition, 2016, ISBN 978-0-486-46981-2. Chapter 8 is titled `Colored Patterns and Tilings`.
- [HALL1949] Hall, M., Canad. J. Math. 1 (1949) 187-190.
- [HAREL1993] Har'El, "Uniform solution for uniform polyhedra", Geom. Dedicata 47 (1993).
- [HOLT2005] Holt, Eick, and O'Brien, "Handbook of Computational Group Theory", CRC, 2005, ISBN 978-1-58488-372-2.
- [HYPERROGUE] HyperRogue, `pattern2.cpp` on the branch `master`, <https://raw.githubusercontent.com/zenorogue/hyperrogue/master/pattern2.cpp>, and <https://zenorogue.github.io/hyperrogue-doc/files.html>.
- [IORNAMENT] iOrnament, App Store listing, <https://apps.apple.com/app/id534529876>.
- [IZQUIERDO1993] Izquierdo, Proc. London Math. Soc. (3) 67 (1993) 305-328. Title not recorded.
- [KALEIDOTILE] Geometry Games, KaleidoTile, version 4.1.5 for Mac, <https://www.geometrygames.org/KaleidoTile/index.html.en>, source archive <https://www.geometrygames.org/KaleidoTile/KaleidoTile-Src.zip>.
- [KALI] Kali, Debian package 3.1+dfsg-4, `HELP` and README, <https://sources.debian.org/data/main/k/kali/3.1+dfsg-4/HELP>, and <https://mige-web.units.it/~logar/matdue/Kali/instructions.html>.
- [KAPLAN2009] Kaplan, "Introductory Tiling Theory for Computer Graphics", Morgan and Claypool, 2009.
- [KAPLAN2019] Kaplan, "Animated Isohedral Tilings", Proceedings of Bridges 2019, pp. 99-106, 2019.
- [KAPLAN2025] Kaplan, C. S., "Isohedral Tiles, Tilings, and Notations", 2025-09-01, <https://isohedral.ca/?p=2780>.
- [KOLBE-EVANS] Kolbe and Evans, Geom. Dedicata, DOI 10.1007/s10711-020-00554-2. Title and year not recorded.
- [LOWINDEX] Culler, Dunfield, and Gorner, `low_index`, README, <https://github.com/3-manifolds/low_index>, and "Low Index, Revisited and Implemented", <https://raw.githubusercontent.com/3-manifolds/low_index/main/doc/lowindex.tex>. Year not recorded.
- [LPD2008] Laigo, Puzon, and De Las Penas, Proceedings of Bridges Leeuwarden 2008, pp. 311-318, 2008, titled `Coxeter Groups in Colored Tilings and Patterns`, <https://archive.bridgesmathart.org/2008/bridges2008-311.html>.
- [LUBOTZKY2003] Lubotzky and Segal, "Subgroup Growth", Birkhauser, 2003.
- [MAGMA] Magma Handbook, low index subgroups, <https://magma.maths.usyd.edu.au/magma/handbook/text/898>.
- [MALINC] Christersson, M., "Image Tiling", 2015-04-19, <https://www.malinc.se/m/ImageTiling.php>.
- [MATEI2002] Matei, D. and A. Suciu, "Hall invariants, homology of subgroups, and characteristic varieties", Int. Math. Res. Not. 2002, arXiv:math/0010046, Section 8.1, eq. (8.2), read through its ar5iv rendering.
- [MORENAMENTS] "Morenaments", exhibit sheet, <https://exponate.phaeno.de/media/pages/experimente/4916/f19b5bd2f0-1714643520/4916_Morenaments_en.pdf>.
- [NELSON2019] Nelson, R., slides, ICERM, 2019. Title not recorded.
- [SCHWARZENBERGER1984] Schwarzenberger, R. L. E., "Colour symmetry", Bull. London Math. Soc. 16 (1984) 209-240.
- [SENECHAL1988] Senechal, Comput. Math. Applic. 16 (1988) 545-553, DOI 10.1016/0898-1221(88)90244-1, titled `Color symmetry`.
- [SIMS1994] Sims, "Computation with Finitely Presented Groups", Cambridge University Press, 1994, ISBN 978-0-521-43213-7, chapters 4 and 5.
- [SINGERMAN2014] Singerman and Watson, arXiv:1408.0127, 2014, Section 2, Example 1 and Theorem 1, <https://ar5iv.arxiv.org/html/1408.0127>. Title not recorded.
- [SYMMHUB] Bulatov, V., SymmHub, MIT licence, 2025, read at commit `f2d19cb` (committed 2026-11-13) on 2026-11-15, <https://github.com/SymmHub/SymmHub>. File paths and lines are relative to the root of the repository.
- [SYMPY] SymPy documentation, `low_index_subgroups`. Address not recorded.
- [TACTILE] Kaplan, `tactile-js`, BSD-3-Clause, 2018, <https://github.com/isohedral/tactile-js>, and `lib/tactile.js`, <https://raw.githubusercontent.com/isohedral/tactile-js/master/lib/tactile.js>.
- [TEGULA2020] Zeller, R., Delgado Friedrichs, O., and D. H. Huson, "Tegula -- exploring a galaxy of two-dimensional periodic tilings", arXiv:2007.10625v1, 2020-07-21, Sections 4 and 5.
- [TESS] Tess, peda.com. The site refused the connection.
- [VONGAGERN2009] von Gagern and Richter-Gebert, "Hyperbolization of euclidean ornaments", Electron. J. Combin. 16 R2, 2009.
- [WP-COXETER-DYNKIN] Wikipedia, "Coxeter-Dynkin diagram", section "Application to uniform polytopes", <https://en.wikipedia.org/wiki/Coxeter%E2%80%93Dynkin_diagram>.
- [WP-INDEX] Wikipedia, "Index of a subgroup", <https://en.wikipedia.org/wiki/Index_of_a_subgroup>.
- [WP-NEC] Wikipedia, "Non-Euclidean crystallographic group". Address not recorded.
- [WP-POLYCHROMATIC] Wikipedia, "Polychromatic symmetry", <https://en.wikipedia.org/wiki/Polychromatic_symmetry>.
- [WP-SNUB] Wikipedia, "Snub (geometry)". Address not recorded.
- [WP-UNIFORM-HYP] Wikipedia, "Uniform tilings in hyperbolic plane", <https://en.wikipedia.org/wiki/Uniform_tilings_in_hyperbolic_plane>.
- [WP-UNIFORM-TILING] Wikipedia, "Uniform tiling", <https://en.wikipedia.org/wiki/Uniform_tiling>.
- [WP-WYTHOFF-SYMBOL] Wikipedia, "Wythoff symbol", section "Description", <https://en.wikipedia.org/wiki/Wythoff_symbol>.
- [ZHAO] Zhao Liang, "Coxeter groups, automata and uniform tilings", undated, <https://neozhaoliang.github.io/uniform-tilings>.
- [ZUCKER] Zucker, M., Shadertoy 3tsSzM, <https://www.shadertoy.com/view/3tsSzM>.
