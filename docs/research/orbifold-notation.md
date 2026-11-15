# Orbifold Notation for Planar Symmetry Groups

| Field | Value |
| --- | --- |
| Status | Draft |
| Date | 2026-11-15 |
| Applies to | The parsing, validation, canonicalization, and classification of orbifold signatures, and the group data derived from a signature |

## Abstract

Conway's orbifold notation names each discrete group of isometries of the sphere, the Euclidean plane, or the hyperbolic plane by a short signature built from handles, cone points, mirror boundaries with corners, and crosscaps. This document states what an implementer needs to turn such a signature into facts about its group. It gives the grammar of a signature and its typographic variants, the rules under which two signatures name the same group, the Euler characteristic by the defect rule, the classification into spherical, Euclidean, and hyperbolic groups, and the complete lists of spherical, wallpaper, and frieze signatures. It also gives the presentation of the group read from the signature, the correspondence with signatures of non-Euclidean crystallographic groups, the orientation-preserving subgroup, and the dimension of the deformation space. Each claim carries its source and whether a primary source confirms it, and the claims that no source establishes are collected in one section.

## 1. Scope

This document covers the orbifold signature of a two-dimensional symmetry group as a string, the equivalences and canonical forms of signatures, the Euler characteristic and the geometry it determines, the tables of spherical, Euclidean, and frieze signatures with their names in other notations, the group presentation and related algebra, and the dimension of the space of deformations. Section 11 describes how existing software parses and names signatures.

This document leaves these topics to other documents or to its sources:

- The construction of a fundamental domain and of generating isometries in a model of each geometry.
- Rendering, colouring, and colour counts.
- Delaney-Dress symbols and tilings, except where existing software uses them to name a group (Section 11).
- Proofs. Every result is cited, and Section 5.5 of `docs/writing-documentation.md` governs claims without a source.

## 2. Notation and Conventions

### 2.1. Symbols

- $h$: The number of handles in a signature.
- $c$: The number of crosscaps in a signature.
- $b$: The number of mirror boundaries in a signature.
- $A, B, C, \ldots$: The orders of the cone points, each an integer $A \geq 2$.
- $a, b, c, \ldots$ after a `*`: The orders of the corner points on that boundary, each an integer $a \geq 2$. These letters denote corner orders only in Section 3 and in quotations from [CH]. Elsewhere $b$ is the number of mirror boundaries.
- $\chi$ or $\chi(O)$: The orbifold Euler characteristic of the orbifold $O$. [CH] writes it $\mathrm{ch}(Q)$.
- $X_O$: The underlying surface of the orbifold $O$, as [TH] uses it.
- $g$ in a formula of [CH]: The order of a finite group. In an NEC signature (Section 9.2), $g$ is instead the genus. Each formula below says which it means.
- $\infty$: An infinite-order point.

### 2.2. Terminology

- orbifold: The quotient of the sphere, the Euclidean plane, or the hyperbolic plane by a discrete group of isometries, as [CH] and [TH] use the term.
- signature: The string that names an orbifold in Conway's notation. [CH] calls it the orbifold symbol.
- handle: A feature written `o`. [CH] also calls it a "wonder".
- crosscap: A feature written `x`. [CH] also calls it a "miracle".
- mirror boundary: A boundary curve of the orbifold, written `*` and followed by the orders of its corners.
- cone point: A point of the orbifold with total angle $2\pi/A$, written as its order $A$ outside every boundary. [CH] calls the group element a gyration, and [TH] calls the point an elliptic point.
- corner point: A point on a mirror boundary with angle $\pi/a$, written as its order $a$ after a `*`. [TH] calls it a corner reflector.
- defect: The amount by which one feature of a signature lowers the Euler characteristic. [CH] uses "defect", and [FG] calls the same quantity the cost or price of a feature.
- bad orbifold: An orbifold that has no covering orbifold which is a manifold, so that no group corresponds to it ([TH], Definition 13.2.3, p. 304). An orbifold that is not bad is good.
- spherical, Euclidean, hyperbolic: The geometry of a good orbifold. [TH] uses elliptic, parabolic, and hyperbolic for the same three cases.
- frieze group: A group whose signature contains $\infty$ and has $\chi = 0$ ([CH], p. 255).
- NEC group: A non-Euclidean crystallographic group, which is a discrete group of isometries of the hyperbolic plane that may reverse orientation.
- NEC signature: The signature of an NEC group in the form of Macbeath, as Section 9.2 gives it.
- Teichmuller space: The space of deformations of the geometric structure of an orbifold, written $T(O)$ as in [TH].

### 2.3. Status of Each Claim

A claim is primary when it was checked against the text of a primary source, which is one of [CH], [TH], [FG], and [TG]. A claim is secondary when it rests only on a secondary source, and the text then says "secondary source only". A source reached only through a summary of its code, rather than quotations, is marked the same way.

A statement about what a program does is primary when it was read directly from the source code at a fixed commit, with file and line references. Such a statement describes the code only. The code is not a publication, and its comments carry no more weight than any other unchecked claim.

### 2.4. Typography

This document writes signatures as code spans in ASCII, with `o` for a handle, `*` for a mirror boundary, and `x` for a crosscap, following [TG] (Section 2, Eq. (1), p. 3). Printed glyphs of the sources appear only in code spans and only where a parser needs to recognize them.

No source defines an ASCII spelling of $\infty$ inside a signature. This document therefore writes the signatures that contain $\infty$ in LaTeX, such as $\ast 22\infty$, and defines no text form for them (Section 12, item 2).

[TH] writes surfaces with a superscript, such as the disk $D^2$. Inside code spans this document writes the superscript as `^2`, such as `D^2( ;2,3,6)`, and keeps Thurston's semicolons, which separate elliptic orders from corner orders.

### 2.5. Conventions on Which Sources Differ

These conventions differ between sources. The text of each section uses the convention named here.

| Topic | Variants | Convention in this document |
| --- | --- | --- |
| Glyphs for handle and crosscap | [CH] and [TG] use `○` or `o` for a handle and `×` or `x` for a crosscap. [FG] (Section 30, p. 62) uses `•` for a handle and `◦` for a crosscap. | [CH] and [TG]. Section 3.3 gives the mapping. |
| Reversal of a single boundary | [CH] (p. 247) and [TG] (p. 3) allow it only under conditions. [EPINET] states it without conditions (secondary source only). | [CH], as the research recommends. Section 4.1 gives the rule. |
| Direction of the boundary conjugation | [CH] writes $\lambda^{-1} P \lambda = S$. [AX-1702.02342] and [AX-1408.0127] write $c_s = e\,c_0\,e^{-1}$. [AX-1612.06810] writes $c_s = e^{-1} c_0\, e$, which matches [CH]. | Each formula keeps the convention of its source. The difference is a convention only. |
| Letters in [TH] | Formula 13.3.4 uses $n_i$ for corner orders and $m_i$ for elliptic orders. The table of Theorem 13.3.6 lists elliptic orders first. Corollary 13.3.7 uses $k$ for elliptic points and $l$ for corner reflectors. | Each formula keeps the definitions of its own statement. |
| Trivial group | [CH] and [TG] write `1`. [CH] (Appendix, p. 257) relates `*11`, `1*`, and `*` (Section 3.5). [GAVROG] writes `1*` and `1x`. | [CH]. |
| Frieze names in the International Union of Crystallography (IUC) notation | The axis convention for `p1m1` and `p11m` varies in the literature (secondary source only). | Unresolved (Section 12, item 9). |

## 3. Signature Grammar

### 3.1. Symbols and Their Meaning

Each character of a signature has one of these meanings ([CH], pp. 247-251, [TG], pp. 2-3).

| ASCII form | Printed form in [CH] | Meaning | Source |
| --- | --- | --- | --- |
| `o` | `○` | A handle | [CH], pp. 250-251 |
| `x` | `×` | A crosscap | [CH], pp. 250-251 |
| `*` | `∗` | One mirror boundary | [CH], p. 248 |
| An order $a$ after a `*` | A digit string in a smaller font | A corner point with angle $\pi/a$ on that boundary | [CH], p. 248 |
| An order $A$ not after any `*` | A digit string in a larger font | A cone point with total angle $2\pi/A$ | [CH], p. 248 |
| `*` with no order after it | `∗` | A mirror boundary with no corners | [CH], p. 248 |
| `1` | `1` | The sphere, so the trivial group | [TG], p. 2, and [CH], Table AI |
| None defined | `∞` | An infinite-order point, which makes the orbifold non-compact | [CH], p. 255 |

A cone or corner order is an integer $v_i \geq 2$ ([TG], p. 2). A boundary may have any number of corners from 0 upward ([CH], p. 248). All of these meanings are primary.

### 3.2. Layout

A signature juxtaposes "the symbols for the handles, cone points, boundaries (with corners), and crosscaps", in that order ([CH], p. 252, also p. 247). [TG] gives the same order (Section 2, Eq. (1), p. 3):

$$\mathtt{o}^{h} \quad A\,B\,C \cdots \quad \ast a_{1} a_{2} \cdots \quad \ast r\,p\,q \cdots \quad \cdots \quad \mathtt{x}^{c}$$

Here $\mathtt{o}^{h}$ is $h$ copies of `o`, $\mathtt{x}^{c}$ is $c$ copies of `x`, and the signature has $b$ boundary blocks, each a `*` followed by the orders of its corners. Orders before the first `*` are cone points, and orders after a `*` are the corners of that boundary. [CH] separates the two kinds by font size as well as by position (p. 248).

These examples show the layout. Each is a signature from the tables of Section 8:

- `632`: three cone points of orders 6, 3, and 2.
- `*632`: one mirror boundary with corners of orders 6, 3, and 2.
- `4*2`: a cone point of order 4 and a mirror boundary with one corner of order 2.
- `22*`: two cone points of order 2 and a mirror boundary with no corners.
- `**`: two mirror boundaries with no corners.
- `22x`: two cone points of order 2 and a crosscap.
- `o`: one handle.

### 3.3. Typographic Variants

The printed glyphs differ between sources, and one older glyph looks like the newer handle glyph but means a crosscap.

| Feature | [CH], 2002 | [TG], 2020 | [FG], 1991 handouts | ASCII in this document |
| --- | --- | --- | --- | --- |
| Handle | `○` | `o` | `•`, "an initial black spot" | `o` |
| Crosscap | `×` | `x` | `◦`, "a final circle" | `x` |
| Mirror boundary | `∗` | `*` | `∗` | `*` |

[FG] states that an initial black spot indicates the addition of a handle and that a final circle indicates the addition of a cross cap (Section 30, p. 62). [FG] accordingly writes `22◦` for the "diglide" and `◦◦` for the "monoglide" (Section 37.1.2, p. 82), which [CH] writes as `22×` and `××`.

A parser never reads `o` as a crosscap, because [CH] and [TG], the later sources, both use the circle or `o` for a handle. A parser that accepts the forms of [FG] maps `•` to a handle and `◦` to a crosscap.

### 3.4. Multi-Digit Orders

No primary source defines a delimiter for an order of 10 or more. [CH] separates cone points from corners only by typography (p. 248), and its Table AI (p. 256) juxtaposes multi-digit orders with visual spacing only, as in `1212` for `nn` with $n = 12$, and in `∗1212`, `2212`, `2∗12`, `∗2212`, and `120120`. Plain text of this form is ambiguous, because "1212" also reads as four cone points.

Two programs wrap such an order in parentheses, as in `(12)`. Tegula does this according to a summary of its code (secondary source only), and the SymmHub parser requires it (Section 11.1). Section 12, item 1 records the question.

### 3.5. Order 1 and the Trivial Group

An order of 1 is omitted from a signature. [CH] states this for the seven infinite spherical series with parameter $n \geq 1$: "When n = 1, it is customarily omitted from the symbol" (Table III note, p. 254). [CH] gives these identities with $a = 1$ and $b = 2$ (Appendix, p. 257):

- `221` is `22`.
- `*221` is `*22`.
- `2*1` is `2*`.
- `*11` is `1*`, which is `*`.

[CH] and [TG] write the trivial group as `1` ([TG], p. 2, [CH], Table AI). [GAVROG] writes `1*` and `1x` (secondary source only). How to treat a `1` token, and an order of 1 in a corner list other than the examples of [CH], is open (Section 12, item 4).

## 4. Equivalences Between Signatures

### 4.1. Equivalence Rules

Two signatures name the same group when one rule, or a sequence of these rules, turns one into the other. Rules 1 to 4 and rule 6 are primary from [CH], and rule 5 is primary from [TG].

1. Permute the cone points freely. [CH], p. 247: "We can freely permute the numbers A, B, C that represent gyrations". [TG], p. 3: "the cone degrees A, B, C, [...] are unordered".
2. Permute the boundary blocks freely ([CH], p. 247).
3. Rotate the corners of any one boundary cyclically, so that `*abc` is the same as `*bca` ([CH], pp. 247-248: "only the cyclic order matters").
4. Reverse the cyclic order of the corners of every boundary at once. When a crosscap is present, also reverse the cyclic order of any single boundary ([CH], p. 247: "we can always reverse the cyclic orders for all boundaries simultaneously and individually if an [crosscap] character is present"). [CH] explains that the corners of an orientable orbifold are listed in the cyclic orders induced from one fixed orientation, and that changing this orientation reverses all of them (footnote 5, p. 248).
5. When a crosscap is present, replace any handle by two crosscaps, so that `ox` is the same as `xxx`. [TG] states the rule and attributes it to Dyck, 1888 (p. 2). [CH] does not state it.
6. Omit an order of 1 (Section 3.5).

[TG] states rule 4 as: "One can flip the direction in which corners of an individual boundary component are listed if there are no other boundary components with corners, or if the surface is non-oriented" (p. 3). The research reads this as agreeing with [CH], because reversing every boundary at once changes only the one boundary that has corners. [EPINET] states reversal without any condition (secondary source only). This document follows [CH] (Section 2.5).

### 4.2. Relations That Are Not Equivalences

These relations appear in the sources but relate different groups:

- The demotions of [CH] (p. 254) replace `*AB...C` by `AB...C`, a final `*` by `x`, `**` by `o`, and `*AA` by `A*`. [CH] uses them to enumerate signatures by the sign of $\chi$, which they preserve.
- [CH] notes that the groups `x`, `22`, and `*` are isomorphic as abstract groups (p. 257). They are different symmetry groups.

## 5. Canonical Form

No primary source defines a canonical string for a signature. [CH] and [TG] state only the equivalences of Section 4.1. The canonical forms in the paper [TG] are canonical forms of Delaney-Dress graphs, defined by the lexicographically smallest traversal (Section 7, p. 10), not of orbifold names.

The only canonicalization of names found is in the code of Tegula, which Section 11.2 describes (secondary source only). It does not apply rule 4 to a single boundary when a crosscap is present. Section 12, item 3 lists the choices that remain open.

## 6. Euler Characteristic

### 6.1. Defect Rule

The Euler characteristic of a signature is 2 minus the sum of the defects of its characters ([CH], "The Defect Formula", pp. 252-253):

$$\chi = 2 - \sum_{s} \mathrm{defect}(s)$$

When the group is finite of order $g$, [CH] writes the same quantity as $2/g$, "since then the denominator is the group order" (p. 253):

$$\frac{2}{g} = 2 - \sum_{s} \mathrm{defect}(s) = \mathrm{ch}(Q)$$

The defects are these ([CH], Table I, p. 252). In the table note of [CH], "The larger font numbers on the left are those not following any [`*`]", which are the cone points.

| Character | Defect |
| --- | --- |
| `o` | $2$ |
| `*` | $1$ |
| `x` | $1$ |
| Cone point of order $N$ | $\frac{N-1}{N}$ |
| Corner point of order $N$ | $\frac{N-1}{2N}$ |
| Cone point of order $\infty$ | $1$ |
| Corner point of order $\infty$ | $\frac{1}{2}$ |

[CH] derives each defect, for example that making an interior point an $A$-fold cone point "decreases ch(Q) by 1 - 1/A" (p. 253). [FG] gives the same values as prices in the "Orbifold Shop": a handle costs 2.00, a mirror 1.00, a crosscap 1.00, a cone point $1.00 \times (n-1)/n$, and a corner reflector $0.50 \times (n-1)/n$, with the prerequisite of at least one mirror (Section 32, pp. 66-67). Spending 1.00 lowers the Euler characteristic by 1 ([FG], Section 33, p. 71). All of this section is primary.

### 6.2. Form in Terms of the Underlying Surface

[TH] defines the Euler characteristic as a sum over the cells $c_i$ of the orbifold, each weighted by the order of its group $\Gamma(c_i)$ (Definition 13.3.3, p. 311):

$$\chi(O) = \sum_{c_i} (-1)^{\dim c_i} \frac{1}{|\Gamma(c_i)|}$$

For a two-dimensional orbifold this equals the Euler characteristic of the underlying surface $X_O$ minus the contributions of the corner reflectors, of orders $n_i$, and of the elliptic points, of orders $m_i$ ([TH], formula in 13.3.4, p. 311). The letters differ from those of the table of Theorem 13.3.6 (Section 2.5).

$$\chi(O) = \chi(X_O) - \frac{1}{2} \sum_{i} \left(1 - \frac{1}{n_i}\right) - \sum_{i} \left(1 - \frac{1}{m_i}\right)$$

### 6.3. Covers and Subgroups

A $k$-sheeted cover $\tilde{O}$ of $O$ has $\chi(\tilde{O}) = k\,\chi(O)$ ([TH], Proposition 13.3.4). [CH] states the same for groups: "passing to a subgroup of index i multiplies the characteristic by i" (p. 255).

### 6.4. Examples

These examples apply the defect rule of Section 6.1. Each result is the value that its source states:

- `4*3`: $\chi = 2 - \left(\frac{3}{4} + 1 + \frac{2}{6}\right) = -\frac{1}{12}$, as [CH] states (p. 255).
- `*3333`: $\chi = 2 - \left(1 + 4 \cdot \frac{2}{6}\right) = -\frac{1}{3}$, as [CH] states (p. 255).
- Group order: [TH] states that the group $\Delta_{2,3,5}$ has Euler number $1/60$ and order $120 = 2/(1/60)$ (p. 311). [CH] lists `*532` with order 120 (Table AI, p. 256).
- $\ast 22\infty$: $\chi = 2 - \left(1 + \frac{1}{4} + \frac{1}{4} + \frac{1}{2}\right) = 0$, which places it among the frieze groups of [CH] (Table IV, p. 255).

### 6.5. Numerical Caveats

Every defect in Section 6.1 is a rational number, so $\chi$ is rational. SymmHub computes $\chi$ in floating point and compares it with thresholds (Section 11.1).

## 7. Classification

### 7.1. Sign and Geometry

The sign of $\chi$ determines the geometry ([CH], p. 253, [TH], Theorem 13.3.6, p. 312, [FG], Section 34, p. 72). All rows are primary.

| Sign of $\chi$ | Geometry | Notes |
| --- | --- | --- |
| $\chi > 0$ | Spherical, or bad | A good orbifold has a finite group. Section 7.2 lists the bad signatures. |
| $\chi = 0$ | Euclidean | A frieze group when the signature contains $\infty$ ([CH], p. 255). |
| $\chi < 0$ | Hyperbolic | |

[TH] states, for closed orbifolds: "A closed two-dimensional orbifold has an elliptic, parabolic or hyperbolic structure if and only if it is good. An orbifold O has a hyperbolic structure if and only if [$\chi(O) < 0$], and a parabolic structure if and only if [$\chi(O) = 0$]. An orbifold is elliptic or bad if and only if [$\chi(O) > 0$]" (Theorem 13.3.6, p. 312). [CH] states that "Those of characteristic 0 are precisely the Euclidean groups" and that those of negative characteristic "correspond to groups acting in the hyperbolic plane" (p. 253). The orbifold is compact exactly when $\infty$ does not appear ([CH], p. 255).

The Magic Theorem of [FG] states that the signatures with exactly zero cost left give the Euclidean patterns with a bounded fundamental domain, and that there are exactly 17 of them (Section 32, p. 67).

### 7.2. Bad Orbifolds

The bad signatures are `m` with $m > 1$, `mn` with $m \neq n$, `*m` with $m > 1$, and `*mn` with $m \neq n$. They are the only signatures that do not correspond to groups. [CH], p. 254: "The symbols [`*mn`] and mn ([$m \neq n$]), together with the particular cases [`*m`] and m ([$m > 1$]) that arise by putting n = 1, are the only ones that do not correspond to groups."

Three primary sources agree:

- [CH], p. 254, as quoted.
- [TH], table of Theorem 13.3.6, p. 312: on the underlying surface $S^2$ the elliptic orders `(n)` and `(n_1, n_2)` with $n_1 < n_2$, and on $D^2$ the corner orders `( ;n)` and `( ;n_1, n_2)` with $n_1 < n_2$. [TH] names the first case "the teardrop" (p. 304).
- [FG], Section 34, p. 72, lists the same four configurations.

[TH] gives the Euler number of a bad orbifold as "either 1 + 1/n or 1/n_1 + 1/n_2" (proof on p. 314).

### 7.3. Group Order and Area

For a spherical signature, the order of the group is

$$|G| = \frac{2}{\chi}$$

([CH], pp. 252-253, [TH], p. 311). [FG] states the same in the other direction: "if the order of symmetry is k, then the Euler characteristic of the quotient orbifold is 2/k" (Section 34, p. 72).

The Gauss-Bonnet theorem for orbifolds gives the area of the quotient ([TH], 13.3.5, p. 312):

$$\int_O K \, dA = 2\pi \chi(O)$$

"If O is elliptic or hyperbolic, then area(O) = 2 pi |chi(O)|" ([TH], p. 312, transcribed in ASCII). The statement makes no orientability assumption, so it covers mirrors and crosscaps. With curvature $-1$, a hyperbolic signature has area $-2\pi\chi$. For example, `*3333` has area $2\pi/3$ by Section 6.4.

[AX-2502.03594] (Section 2) gives the area of an NEC group as $2\pi\mu$ (secondary source only), with

$$\mu = \alpha g + k - 2 + \sum_{i} \left(1 - \frac{1}{m_i}\right) + \frac{1}{2} \sum_{i} \sum_{j} \left(1 - \frac{1}{n_{ij}}\right)$$

where $g$ is the genus, $k$ the number of period cycles, $\alpha = 2$ for sign $+$, and $\alpha = 1$ for sign $-$. The research matched this term by term with the defects of Section 6.1 and found $\mu = -\chi$. No source states that identity (Section 12, item 10).

## 8. Signature Tables

### 8.1. The 17 Euclidean Signatures

The 17 signatures are primary ([CH], Table II, p. 254, [TH], parabolic list of Theorem 13.3.6, p. 312). [TH] states: "The 17 parabolic orbifolds correspond to the 17 'wallpaper groups'" (p. 314).

The Thurston column was compiled for this document from the parabolic list of [TH] (p. 312). The list is primary, but the row-by-row matching has no source of its own. The IUC column is secondary only, from [WP-ORB] and [WP-WALL]. None of the primary sources contains IUC names.

| Signature | Thurston ([TH], p. 312) | IUC (secondary) |
| --- | --- | --- |
| `*632` | `D^2( ;2,3,6)` | `p6m` |
| `632` | `S^2(2,3,6)` | `p6` |
| `*442` | `D^2( ;2,4,4)` | `p4m` |
| `4*2` | `D^2(4;2)` | `p4g` |
| `442` | `S^2(2,4,4)` | `p4` |
| `*333` | `D^2( ;3,3,3)` | `p3m1` |
| `3*3` | `D^2(3;3)` | `p31m` |
| `333` | `S^2(3,3,3)` | `p3` |
| `*2222` | `D^2( ;2,2,2,2)` | `pmm` |
| `2*22` | `D^2(2;2,2)` | `cmm` |
| `22*` | `D^2(2,2; )`, printed `(2; 2; )` | `pmg` |
| `2222` | `S^2(2,2,2,2)` | `p2` |
| `**` | annulus `( ; )` | `pm` |
| `*x` | Mobius band `( ; )` | `cm` |
| `xx` | Klein bottle `( )` | `pg` |
| `22x` | `P^2(2,2)` | `pgg` |
| `o` | `T^2( )` | `p1` |

[TH] prints the entry for `22*` as `(2; 2; )`. The research reads it as `(2,2; )`, because the list totals 17 only with that reading.

[FG] gives Conway's nickname for each wallpaper group, such as hexascopic and tetragyro (Section 37.1.2, manuscript pp. 84-85). The research did not record the nickname of each group, so this table omits them.

### 8.2. The 14 Spherical Types

The 14 types are primary: seven particular groups and seven infinite series with a parameter $n \geq 1$ ([CH], Table III, p. 254). The orders are primary from $|G| = 2/\chi$ (Section 7.3) and are tabulated in [CH] (Table AI, p. 256). The Schoenflies and Coxeter columns are secondary only, from [WP-SPH].

| Signature | Order | Schoenflies (secondary) | Coxeter (secondary) |
| --- | --- | --- | --- |
| `nn` | $n$ | $C_n$ | $[n]^+$ |
| `*nn` | $2n$ | $C_{nv}$ | $[n]$ |
| `n*` | $2n$ | $C_{nh}$ | $[2,n^+]$ |
| `nx` | $2n$ | $S_{2n}$ | $[2^+,2n^+]$ |
| `22n` | $2n$ | $D_n$ | $[2,n]^+$ |
| `*22n` | $4n$ | $D_{nh}$ | $[2,n]$ |
| `2*n` | $4n$ | $D_{nd}$ | $[2^+,2n]$ |
| `332` | $12$ | $T$ | $[3,3]^+$ |
| `*332` | $24$ | $T_d$ | $[3,3]$ |
| `3*2` | $24$ | $T_h$ | $[4,3^+]$ |
| `432` | $24$ | $O$ | $[4,3]^+$ |
| `*432` | $48$ | $O_h$ | $[4,3]$ |
| `532` | $60$ | $I$ | $[5,3]^+$ |
| `*532` | $120$ | $I_h$ | $[5,3]$ |

The elliptic list of [TH] (Theorem 13.3.6, p. 312) agrees with this table. It lists these orbifolds by underlying surface:

- $S^2$: `( )`, `(n,n)`, `(2,2,n)`, `(2,3,3)`, `(2,3,4)`, `(2,3,5)`.
- $D^2$: `( ; )`, `( ;n,n)`, `( ;2,2,n)`, `( ;2,3,3)`, `( ;2,3,4)`, `( ;2,3,5)`, `(n; )`, `(2;m)`, `(3;2)`.
- $P^2$: `( )`, `(n)`.

[CH] gives the abstract isomorphism class of each group in its Table AI (p. 256), with the classes C, D, P, 2 x C, 2 x D, and 2 x P. Conway's manuscript in [FG] (p. 88, handwritten) gives his own names, such as $\pm I$ for `*532`, $I$ for `532`, $TO$ for `*332`, $\pm T$ for `3*2`, $\pm D_{2n}$ for `*22n`, $DD_{4n}$ for `2*n`, and $CC_{2n}$ for `nx`. These names are not Schoenflies notation.

### 8.3. The 7 Frieze Signatures

The frieze groups are the signatures that contain $\infty$ and have $\chi = 0$. They "are obtained by putting n = [$\infty$] in Table III" ([CH], p. 255, Table IV). The signatures are primary. The nicknames are primary from Conway's manuscript in [FG] (p. 86, handwritten). The IUC column is secondary only, from [WP-FRIEZE], and its axis convention is unresolved (Section 2.5).

| Signature | Nickname | IUC (secondary) |
| --- | --- | --- |
| $\infty\infty$ | hop | `p1` |
| $\infty\times$ | step | `p11g` |
| $\infty\ast$ | jump | `p11m` |
| $\ast\infty\infty$ | sidle | `p1m1` |
| $22\infty$ | spinning hop | `p2` |
| $2\ast\infty$ | spinning sidle | `p2mg` |
| $\ast 22\infty$ | spinning jump | `p2mm` |

## 9. Algebra

### 9.1. Presentation Read from the Signature

[CH] reads a presentation of the group directly from the signature ("Generators and Relations", pp. 255-257). Each feature contributes one Greek generator, which "circumnavigates that feature in the positive direction", and possibly some Latin generators. These steps build the presentation, and all of them are primary:

1. For each handle `o`, add Latin generators $X$ and $Y$ and the Greek generator $\alpha$, with $X^{-1} Y^{-1} X Y = [X, Y] = \alpha$.
2. For each cone point of order $A$, add no Latin generator. Add the Greek generator $\gamma$ with $\gamma^A = 1$.
3. For each boundary with $n$ corners of orders $a, b, \ldots, c$, add $n + 1$ Latin reflections $P, Q, R, \ldots, S$ and the Greek generator $\lambda$, with these relations:

   $$1 = P^2 = (PQ)^a = Q^2 = (QR)^b = R^2 = (RS)^c = S^2$$

   $$\lambda^{-1} P \lambda = S$$

4. For each crosscap `x`, add the Latin generator $Z$ and the Greek generator $\omega$, with $Z^2 = \omega$.
5. Add the global relation that "the product of all Greek generators is 1":

   $$\alpha \cdots \gamma \cdots \lambda \cdots \omega = 1$$

[CH] fixes the base point and the left-to-right order of the features in the global relation in its Figure 21 (p. 255). [CH] also proposes a compact notation that writes the generators into the signature (p. 257):

$$\circ^{XY} \cdots A^{\gamma} \cdots \ast^{\lambda} P\,a\,Q\,b\,R\,c\,S \cdots \times^{Z}$$

### 9.2. NEC Signatures

[CH] states that "our 'orbifold symbol' is just an elegant form of Macbeath's 'signature'" (p. 247), which is primary. The NEC signature has this form (secondary source only, [AX-1702.02342], Section 3.2, and [AX-2502.03594], Section 2):

`(g; +-; [m_1, ..., m_r]; {(n_11, ..., n_1s_1), ..., (n_k1, ..., n_ks_k)})`

Here $g$ is the genus, the sign is $+$ or $-$, the $m_i$ are the proper periods, and each parenthesized group of $n_{ij}$ is one period cycle. The research assembled this dictionary from the two secondary definitions. No source states the dictionary itself.

| NEC signature | Orbifold signature |
| --- | --- |
| Sign $+$, genus $g$ | $g$ copies of `o` |
| Sign $-$, genus $g$ | $g$ copies of `x` |
| Proper periods $m_i$ | Cone point orders |
| Each period cycle $(n_{i1}, \ldots, n_{is_i})$ | One `*` followed by $n_{i1} \ldots n_{is_i}$ |
| Empty period cycle | A `*` with no orders |

The NEC presentation has the generators $x_i$, $c_{ij}$, $e_i$, $a_i$, $b_i$ for sign $+$ or $d_i$ for sign $-$, and the long relation $x_1 \cdots x_r\, e_1 \cdots e_k \prod_i [a_i, b_i] = 1$, with $\prod_i d_i^2$ in place of the commutators for sign $-$ (secondary source only). It matches the presentation of [CH] term by term as $\gamma \leftrightarrow x_i$, $\lambda \leftrightarrow e_i$, $P, \ldots, S \leftrightarrow c_{i0}, \ldots, c_{is}$, $X, Y \leftrightarrow a, b$, and $Z \leftrightarrow d$. The direction of the boundary conjugation differs between sources (Section 2.5).

### 9.3. Orientation-Preserving Subgroup

According to [AX-1702.02342] (Section 3.2, secondary source only, read through a paraphrase), the orientation-preserving subgroup $\Lambda^+$ of an NEC group with the signature of Section 9.2 has this signature, with $\epsilon = 2$ for sign $+$ and $\epsilon = 1$ for sign $-$:

`(eps*g + k - 1; m_1, m_1, ..., m_r, m_r, n_11, ..., n_ks_k)`

In words, each cone order appears twice, each corner order becomes one cone order, and the genus is $2g + k - 1$ for sign $+$ or $g + k - 1$ for sign $-$. [AX-1612.06810] restates Singerman's Theorem 2.4 [SINGERMAN] for the case with no reflections and sign $-$: the subgroup has genus $g - 1$ "and every m_i appears two times" (secondary source only).

The primary support is partial. [TH] gives the orientation-preserving subgroup of index 2 of the reflection group of a rectangle, `*2222`, whose quotient is "a sphere, with four distinguished points", which is `2222` (Example 13.1.4, p. 298). The index rule of Section 6.3 is consistent with a subgroup of index 2. Section 12, item 5 records that the general rule needs a primary source.

### 9.4. Reflection Groups

When an orbifold "has the combinatorial type of a polygon", its fundamental group "is generated by reflections" ([TH], p. 320, primary). The research identifies this class with the single-boundary signatures `*n_1...n_s`. [AX-2502.03594] (Section 5.1, secondary source only) states the same for the NEC signature `(0; +; [-]; {(n_1, ..., n_s)})`.

[TH] (Example 13.1.3, p. 298) and [CH] (p. 252) give the case `*2222`: "the group [`*2222`] is generated by the reflections in the sides of the rectangle". The local group of a corner reflector of order $n$ is $\langle a, b : a^2 = b^2 = (ab)^n = 1 \rangle$ ([TH], Proposition 13.3.1(iii), p. 308).

Whether a signature with several boundaries, such as `**`, or any other signature that is not a polygon gives a group generated by reflections is open (Section 12, item 6).

## 10. Deformations

### 10.1. Hyperbolic Signatures

For an orbifold with $\chi(O) < 0$, the Teichmuller space is a Euclidean space of known dimension ([TH], Corollary 13.3.7, p. 318, primary): "The Teichmuller space T(O) of an orbifold O with [$\chi(O) < 0$] is homeomorphic to Euclidean space of dimension [$-3\chi(X_O) + 2k + l$], where k is the number of elliptic points and l is the number of corner reflectors."

$$\dim T(O) = -3\chi(X_O) + 2k + l$$

Here $\chi(X_O)$ is the Euler characteristic of the underlying surface, $k$ is the number of cone points, and $l$ is the number of corner points. These letters differ from those of formula 13.3.4 (Section 2.5).

The proof cuts $O$ into primitive pieces, and the lengths and twists of closed geodesics and the lengths of arcs are the parameters ([TH], proof of Corollary 13.3.7). [TH] gives these supporting cases, both primary:

- $D^2$ with corners of orders $m_1, \ldots, m_l$ is parametrized by $(\mathbb{R}_+)^{l-3}$ (p. 316).
- For an $n$-gonal orbifold $P$, $N(P) \cong D^{n-3}$ with interior $T(P)$ (Theorem 13.8.1, p. 347).

[CG] (Proposition 5.1, secondary source only) adds $2n$ to the dimension for "boundary full 1-orbifolds". The research does not define that term further.

### 10.2. Euclidean Signatures

Neither [TH] nor [CH] gives the dimension of the space of deformations of a Euclidean signature. [CH] says only that each Euclidean signature gives one group "up to isotopy (i.e., continuous variation)", for example that "any two groups of type [`*2222`] are isotopic" (pp. 252 and 254). Whether Corollary 13.3.7 of [TH] extends to $\chi = 0$ is unverified. Section 12, item 7 records the question.

## 11. Software Practice

### 11.1. SymmHub

SymmHub contains the only parser of signature strings found. This section is primary in the sense of Section 2.3: it was read directly from the source code at commit f2d19cb ([SYMMHUB]) and describes only what that code does. All paths are relative to the repository root.

The parser is the function `hashOrbifoldString` in `lib/orbilib/OrbifoldGeometrization.js`, lines 77 to 262. It returns the fields `errorMessage`, `handlesN`, `crosscapsN`, `conepoints`, `kaleidoscopes`, `eulerchar`, and `curvature`. Its grammar is a hand-written state machine that enforces no order of features:

- `o` or `O` is a handle, `x` or `X` is a crosscap, and `*` opens a new boundary.
- Digits before any `*`, or after an `o` or an `x`, are cone points. Digits after a `*`, up to the next `o`, `x`, or `*`, are corners of that boundary.
- An order of 10 or more is written in parentheses, as in `(12)`.
- The parser skips no whitespace, so a space is an error.

The parser rejects these inputs:

| Input | Lines | Message |
| --- | --- | --- |
| A single digit `0` or `1` as an order | 128-137 | `'Cone point of order ' + currentN` or `'Kaleidoscopic point of order ' + currentN` |
| The same number in parentheses | 102-111 | The same two messages |
| An unknown character | 151-154 | `'Poorly formed expression'`, with `noErrorQ = false` |
| `*n`, one boundary with one corner and no cone point | 187-191 | "Single kaleidoscopic point -- not yet implemented" |
| `*pq` with $p \neq q$ | 193-199 | "Poorly formed orbifold" |
| `n`, one cone point and no boundary | 203-211 | "Single rotation point -- not yet implemented" |
| `pq` with $p \neq q$ | 212-219 | "Poorly formed orbifold" |

The last four checks run only inside the block at line 173, `if (noErrorQ && handlesN == 0 && crosscapsN == 0)`.

One path sets an error message without marking the parse as failed. A character that is not a digit inside parentheses (lines 116-119) sets `errorMessage = 'Poorly formed expression'` and leaves `noErrorQ` unchanged. The returned `eulerchar` then stays `NaN`, because the computation at line 226 runs only when `errorMessage == ""`.

These inputs fall through the checks without an error:

- `*` alone. Its boundary has no corners, so neither the test for one corner nor the test for two corners matches. Line 228 gives $\chi = 1$, and lines 242-249 give the curvature 1.
- `*pp` and `pp`, with two equal orders.
- Every signature with a handle or a crosscap, because the block at line 173 requires none.
- Three or more cone points, two or more boundaries with no cone point, and any signature with both cone points and corners.

An earlier reading of the same commit also reports the message "Empty orbifold!" for the empty string. The direct reading did not re-check it.

The parser computes the Euler characteristic in lines 226 to 237 and tests the curvature in lines 242 to 251. Line 228 starts the computation with `eulerchar = 2 - 2 * handlesN - crosscapsN - kaleidoscopes.length`. According to the earlier reading, the full expression is

$$\mathtt{eulerchar} = 2 - 2h - c - b - \sum_{\text{cones}} \left(1 - \frac{1}{n}\right) - \frac{1}{2} \sum_{\text{corners}} \left(1 - \frac{1}{m}\right)$$

in floating point, where $b$ is the number of boundaries. It sets the curvature to 0 when $-0.001 < \chi < 0.01$, to 1 when $\chi \geq 0.01$, and to $-1$ when $\chi \leq -0.001$. Comments in the code justify these thresholds by naming `*237`, with $\chi = -1/84$, as the smallest negative value and `*235`, with $\chi = 1/60$, as the smallest positive value. These are claims of the code comments, which no primary source read for this document confirms.

The parser departs from the primary sources in two ways that an implementer can check against Sections 3.4 and 3.5. It rejects an order of 1, which [CH] uses in `*11`, and it accepts features in any order.

### 11.2. Tegula

The file `OrbifoldGroupName.java` of Tegula (husonlab/tegula, main branch) builds a name from a Delaney-Dress symbol and does not parse strings ([TEGULA], secondary source only, from a summary of the code). It produces a name with these steps:

1. Sort the cone orders in descending order.
2. Rotate each cycle of corners to its lexicographically largest rotation, in `computeLargestCyclicOrdering`.
3. Build the list of boundaries, and the list with every boundary reversed.
4. Sort each list in descending order, with `IntegerListCompareDown`.
5. Keep the larger of the two lists.
6. Write the handles as `o`, then the cone orders, then `1` if nothing else has been written, then the boundary blocks, then the crosscaps as `x`.

After it generates a name, the code rejects `*p`, `pq`, and `*pq`. The paper [TG] defines the curvature of a tiling as $K = \sum_D \left(1/m_{01} + 1/m_{12} - 1/2\right)$, with the sum and the symbols $m_{01}$ and $m_{12}$ as [TG] defines them, and states that its sign gives the geometry (p. 6, primary).

### 11.3. Gavrog

The file `delaney2d.clj` of Gavrog sorts cone orders and corner orders in descending order and handles neither cyclic rotation nor reversal ([GAVROG], secondary source only, from a summary of the code).

## 12. Open Questions and Unverified Claims

Each item states what no read source establishes. Items 1 to 9 come from the gaps that the research found, and items 10 to 15 list the claims of this document that rest on secondary sources or on matching done for this document.

1. Multi-digit orders. No primary source defines how to write an order of 10 or more in plain text (Section 3.4). Tegula and SymmHub use parentheses. [CBG] may state a convention.
2. Infinite orders in text. No source defines an ASCII spelling of $\infty$ in a signature (Section 2.4).
3. Canonical form. No primary source defines a canonical string (Section 5). An implementation has to choose an order for cone points, a normal form for each cycle of corners, an order for boundaries, the treatment of a single reversed boundary when a crosscap is present, and the delimiter for multi-digit orders. Tegula is the only precedent, and it compares only the reversal of all boundaries at once. [BH] may give rules for names.
4. Degenerate inputs. How to treat a `1` token, and an order of 1 in a corner list other than the examples of [CH] (Section 3.5), is not stated.
5. Orientation-preserving subgroup. The rule of Section 9.3 rests on secondary sources. It needs to be checked against Theorem 2.4 of [SINGERMAN].
6. Reflection groups. Which signatures with several boundaries, such as `**`, or other signatures that are not polygons give groups generated by reflections is not stated (Section 9.4). The presentation of [CH] suggests that such groups need generators that are not reflections, but no source states it.
7. Euclidean deformations. No source gives the dimension of the space of deformations of each wallpaper group, and whether [TH] Corollary 13.3.7 extends to $\chi = 0$ is unverified (Section 10.2). An unverified first-pass count from metrics invariant under the point group gave the values below, before scale is removed. These values have no source. An implementation that uses them before a source confirms them risks giving a Euclidean group the wrong number of deformation parameters. [KLR] and [SCOTT] may cover this.

   | Groups | Count before removing scale (unverified) |
   | --- | --- |
   | `p1`, `p2` | 3 |
   | `pm`, `pg`, `cm`, `pmm`, `pmg`, `pgg`, `cmm` | 2 |
   | Groups with rotations of order 3, 4, or 6 | 1 |

8. Spherical coincidences for small $n$. Coincidences such as `2*2` against `*22n` at $n = 1$, beyond those in the Appendix of [CH], are not tabulated. This wording, including the example, is as the research recorded it. It has not been checked against the Appendix of [CH].
9. Frieze IUC names. The axis convention for `p1m1` and `p11m` was not resolved against a primary source (Section 8.3).
10. Area identity. The identity $\mu = -\chi$ between the NEC area formula and the defect rule (Section 7.3) was found by matching terms, and no source states it.
11. Cross-references to other notations. The IUC names of Sections 8.1 and 8.3 and the Schoenflies and Coxeter names of Section 8.2 rest only on Wikipedia. [CBG] would give primary correspondences. The Thurston column of Section 8.1 was matched for this document.
12. NEC signatures. The form of the NEC signature and its presentation (Section 9.2) rest on secondary sources, and the dictionary was assembled for this document. [MACBEATH] and [WILKIE] are the primary sources.
13. Software. The descriptions of Tegula and Gavrog in Sections 11.2 and 11.3 rest on a summary of their code, not on quotations. The description of SymmHub in Section 11.1 is primary for what the code does (Section 2.3). Two parts of it were not re-checked in the direct reading: the message for the empty string and the full Euler characteristic expression. The claims in its code comments about `*237` and `*235` have no primary source.
14. Boundaries without corners. Step 3 of Section 9.1 states the relations for a boundary with $n$ corners. The research does not record what [CH] states for $n = 0$, which is the boundary of `**` or `22*`.
15. Signatures with $\infty$ outside the friezes. Theorem 13.3.6 of [TH] covers closed orbifolds. The research records no statement of the geometry of a non-compact signature with $\chi \neq 0$.

These sources could not be read in full. Each is recorded in `docs/research/source-requests.md` with what the project needs from it, and every claim that depends on it is unverified:

| Priority | Source | Needed for |
| --- | --- | --- |
| High | [SINGERMAN] | The signature of the orientation-preserving subgroup (item 5). |
| High | [CBG] | Primary IUC and Schoenflies correspondences, multi-digit orders, and canonical writing (items 1, 3, and 11). |
| Medium | [KLR] | Teichmuller dimensions, possibly of Euclidean orbifolds (item 7). |
| Medium | [BH] | The algorithm that computes the orbifold name from a Delaney symbol, which [TG] cites on p. 6 (item 3). |
| Medium | [MACBEATH], [WILKIE] | The primary NEC signature and presentation (item 12). |
| Medium | [SCOTT] | Section 2 on two-dimensional orbifolds, possibly with Euclidean moduli (item 7). |
| Low | [CONWAY92] | The original grammar, largely superseded by [CH]. |
| Low | [ZVC] | Presentations and Teichmuller spaces. |
| Low | [BEGG] | Chapter 0 on NEC groups. |
| Low | [HUSON93] | Orbifold names computed from Delaney symbols. |

## 13. References

### 13.1. Primary Sources

These sources were read in full. Every citation of them was accessed on 2026-11-15.

- [CH] Conway, J. H. and D. H. Huson, "The Orbifold Notation for Two-Dimensional Groups", Structural Chemistry 13(3/4), 247-257, 2002, DOI 10.1023/A:1015851621002, <https://doi.org/10.1023/A:1015851621002>.
- [FG] Conway, Doyle, Gilman, and Thurston, "Geometry and the Imagination in Minneapolis", handouts of 1991, arXiv:1804.03055v1, 2018, <https://arxiv.org/abs/1804.03055>.
- [TG] Zeller, Delgado Friedrichs, and Huson, "Tegula - exploring a galaxy of two-dimensional periodic tilings", arXiv:2007.10625v1, July 2020, <https://arxiv.org/abs/2007.10625>.
- [TH] Thurston, W. P., "The Geometry and Topology of Three-Manifolds", Chapter 13, lecture notes of 1980, electronic version 1.1, March 2002, <http://www.msri.org/publications/books/gt3m/>.

### 13.2. Secondary Sources

- [AX-1408.0127] arXiv:1408.0127. The research recorded neither the authors nor the title. <https://arxiv.org/abs/1408.0127>.
- [AX-1612.06810] arXiv:1612.06810. The research recorded neither the authors nor the title. <https://arxiv.org/abs/1612.06810>.
- [AX-1702.02342] arXiv:1702.02342. The research recorded neither the authors nor the title. <https://arxiv.org/abs/1702.02342>.
- [AX-2502.03594] arXiv:2502.03594. The research recorded neither the authors nor the title. <https://arxiv.org/abs/2502.03594>.
- [CG] Choi and Goldman, arXiv:math/0107193. The research recorded no title. <https://arxiv.org/abs/math/0107193>.
- [EPINET] EPINET, undated. The research recorded no title or URL.
- [WP-FRIEZE] Wikipedia, "Frieze group", accessed 2026-11-15, <https://en.wikipedia.org/wiki/Frieze_group>.
- [WP-ORB] Wikipedia, "Orbifold notation", accessed 2026-11-15, <https://en.wikipedia.org/wiki/Orbifold_notation>.
- [WP-SPH] Wikipedia, "List of spherical symmetry groups", accessed 2026-11-15, <https://en.wikipedia.org/wiki/List_of_spherical_symmetry_groups>.
- [WP-WALL] Wikipedia, "Wallpaper group", accessed 2026-11-15, <https://en.wikipedia.org/wiki/Wallpaper_group>.

### 13.3. Software

- [GAVROG] Gavrog, file `delaney2d.clj`. The research recorded no authors, repository URL, or revision.
- [SYMMHUB] SymmHub source, commit f2d19cb (committed 2026-11-13), file `lib/orbilib/OrbifoldGeometrization.js`, function `hashOrbifoldString`, lines 77-262, read 2026-11-15, <https://github.com/SymmHub/SymmHub>.
- [TEGULA] Tegula, repository husonlab/tegula, file `OrbifoldGroupName.java`, main branch, <https://github.com/husonlab/tegula>.

### 13.4. Sources Not Read

- [BEGG] Bujalance, Etayo, Gamboa, and Gromadzki, "Automorphism Groups of Compact Bordered Klein Surfaces", Lecture Notes in Mathematics 1439, Springer, 1990, ISBN 3-540-52941-1.
- [BH] Balke, L. and D. H. Huson, "Two-dimensional groups, orbifolds and tilings", Geometriae Dedicata 60, 89-106, 1996.
- [CBG] Conway, J. H., Burgiel, H., and C. Goodman-Strauss, "The Symmetries of Things", A K Peters, 2008, ISBN 978-1-56881-220-5.
- [CONWAY92] Conway, J. H., "The orbifold notation for surface groups", London Mathematical Society Lecture Note Series 165, 438-447, 1992, ISBN 0-521-40685-4.
- [HUSON93] Huson, D. H., "The generation and classification of tile-k-transitive tilings ...", title truncated in the research, Geometriae Dedicata 47, 269-296, 1993.
- [KLR] Kulkarni, R. S., Lee, K. B., and F. Raymond, "Deformation spaces for Seifert manifolds", Lecture Notes in Mathematics 1167, Springer, 1985.
- [MACBEATH] Macbeath, A. M., Canadian Journal of Mathematics 19, 1192-1205, 1967, DOI 10.4153/CJM-1967-108-5, <https://doi.org/10.4153/CJM-1967-108-5>.
- [SCOTT] Scott, P., "The geometries of 3-manifolds", Bulletin of the London Mathematical Society 15, 401-487, 1983, DOI 10.1112/blms/15.5.401, <https://doi.org/10.1112/blms/15.5.401>.
- [SINGERMAN] Singerman, D., "On the structure of non-Euclidean crystallographic groups", Proceedings of the Cambridge Philosophical Society 76, 233-240, 1974, DOI 10.1017/S0305004100048891, <https://doi.org/10.1017/S0305004100048891>.
- [WILKIE] Wilkie, H. C., Mathematische Zeitschrift 91, 87-102, 1966, DOI 10.1007/BF01110157, <https://doi.org/10.1007/BF01110157>.
- [ZVC] Zieschang, Vogt, and Coldewey, "Surfaces and Planar Discontinuous Groups", Lecture Notes in Mathematics 835, Springer, 1980, ISBN 3-540-10024-5.
