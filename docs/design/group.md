# Group Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-17 |
| Applies to | `crates/escher-group` |

## Abstract

This document specifies the crate `escher-group`, which holds finitely presented groups and Todd-Coxeter coset enumeration with a bound on the number of cosets. It defines letters, words, relators with an exponent, and presentations with named generators, the text form of each, and the presentation that Conway's rules read from an orbifold signature. It specifies the coset table that an enumeration returns, the Felsch strategy as numbered steps with the source of each step, the rule for relators of one letter, the closing pass that makes a returned table rest on sourced statements, the limit on cosets and what reaching it shows, and the standard form that makes equal subgroups give equal tables. It also states the error type, the limits on untrusted input with their memory bound and the residual risk on time, the tests with their worked examples, the figures measured on a prototype, the dependencies, the unverified statements that the code relies on, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

The colourings of a tiling come from subgroups of finite index of its symmetry group, and every subgroup of finite index has a coset table (Sections 4.1 and 7.9 of the colouring reference). A program that lists colourings therefore needs the group as generators and relators, and a procedure that turns a presentation and a subgroup into a coset table. No usable Rust crate provides either (Section 14 of `docs/research/rust-ecosystem.md`, as Section 5.7 of the plan records). The procedure has no computable bound on its work, and some inputs make it define many more cosets than the index (Section 7.9), so an implementation that reads untrusted input needs limits that it states and enforces.

### 1.2. Purpose

`escher-group` is the one place where a presentation becomes a coset table. It builds the Conway presentation of a signature, accepts any other presentation within its limits, and returns either a complete coset table in a standard form or an error that claims nothing about the index. Section 3 of the architecture places the crate in the workspace with `escher-signature` as its only internal dependency. The coset table is the base of the colourings of Section 5.4 of the architecture, and the enumeration of a group of known order is an independent check of a presentation.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-domain`, whose side presentation is a `Presentation` of this crate, and of `escher-cli`, which prints presentations and runs the enumeration.

### 1.4. Scope

This document covers the public surface of the crate, words, relators, presentations and their text, the Conway presentation, the coset table and its standard form, the enumeration algorithm with its limit, the error type, the limits on untrusted input, the tests, the prototype measurements, and the dependencies of the crate.

### 1.5. Non-Goals

These topics are owned elsewhere:

- The side presentation of a fundamental domain is a function of `escher-domain`, which builds a `Presentation` of this crate. DOM-32 to DOM-35 of Section 5.4 of `docs/design/domain.md` specify it, and `UNV-32` (Section 7 of the plan) records its unverified letter form.
- The option `--presentation` of `escher info`, its lines, and the comparison of the enumerated order with the group order are specified by CLI-37 to CLI-39 and CLI-41 of Section 5.7 and by CLI-40 of Section 6.9 of `docs/design/cli.md`. They are proposed as `DEC-35` (Section 10.2 of the architecture).
- The low-index subgroup search, the colourings with their ids, and `colour_of` belong to later work on this crate (Section 3.2 of the architecture). This document reserves no name for them.
- The rule that turns the right cosets of a coset table into the left cosets of the colourings of Section 5.4 of the architecture belongs to the design of the colourings, under `UNV-17`.
- Reidemeister-Schreier rewriting (Section 7.18 of the colouring reference) is not part of this crate.
- The relator-based (HLT) strategy of Section 7.11 of the colouring reference is not implemented (`DEC-33` of Section 10.2 of the architecture).

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md`, the terms of Section 2.2 of the colouring reference and of the notation reference, and the terms feature and order of Section 2.1 of the signature design. It cites documents by these short names:

| Document | Short name |
| --- | --- |
| `docs/research/colourings.md` | the colouring reference |
| `docs/research/orbifold-notation.md` | the notation reference |
| `docs/research/symmhub.md` | the SymmHub reference |
| `docs/design/architecture.md` | the architecture |
| `docs/design/poc-plan.md` | the plan |
| `docs/design/signature.md` | the signature design |
| `docs/rust-coding-standards.md` | the coding standard |

It adds these terms:

- generator: One of the named free generators of a presentation, numbered from 0.
- letter: A generator or the inverse of a generator.
- column: The position of a letter in a row of a coset table, $2k$ for generator $k$ and $2k + 1$ for its inverse.
- word: A finite sequence of letters, read from left to right. The empty word is the identity.
- freely reduced: Having no letter directly followed by its inverse.
- cyclically reduced: Freely reduced, and with a last letter that is not the inverse of the first.
- relator: A word that equals the identity in the group, stored as a base word and an exponent.
- expanded relator: The base of a relator written out exponent times.
- presentation: A list of generator names and a list of relators.
- Conway presentation: The presentation that Section 9.1 of the notation reference reads from a signature.
- Greek generator: The generator of [CH] that circumnavigates one feature of a signature. In a Conway presentation it is `a<i>`, `g<i>`, `l<i>`, or `w<i>` (Section 5.1).
- side presentation: The presentation of a fundamental domain with one generator for each side, which `escher-domain` builds (Section 5.4 of `docs/design/domain.md`).
- cyclic conjugate: A rotation of a word, the word that starts at one of its positions and wraps around.
- coset: A right coset $Hg$ of the subgroup $H$, named in a coset table by a number from 0.
- coset table: A table with one row for each coset and one column for each letter, whose entry in row $c$ and column $x$ is the coset $c^x$ (Section 7.9 of the colouring reference).
- entry: One cell of a coset table, defined or undefined.
- live coset: A coset number that no coincidence has eliminated. A coset number that a coincidence has eliminated is dead.
- index: The number of rows of a complete coset table, which is the index of the subgroup.
- definition, deduction, coincidence: The three events of Section 7.10 of the colouring reference.
- scan: The trace of a word from a coset forwards and backwards, with the outcomes of Section 7.10 of the colouring reference.
- deduction stack: The list of entries written and not yet scanned.
- lookahead: A scan of every relator at every live coset without a definition (Section 7.11 of the colouring reference).
- closing pass: The scan of every relator at every live coset and of every subgroup generator at coset 0 that ends every enumeration (Section 7.7).
- one-letter relator: A relator whose expanded relator, cyclically reduced, is a single letter.
- coset limit: The largest number of cosets that one enumeration may define.
- standard: Numbered in the lenlex standard of [GAP], in which the first occurrence of each coset number, read row by row over all columns, precedes that of every larger number (Section 7.14 of the colouring reference).

## 3. Public Surface

### 3.1. Items

GRP-1. The crate MUST export exactly the items of this section from its root, each through one path, and MUST give no type a public field, because a public field or a second constructor lets a caller build a word, a presentation, or a table that breaks the invariants of Section 3.2, and the enumeration and every later caller rely on them (API-10 of the coding standard).

This listing shows each public item with its signature. Bodies are omitted:

```rust
pub struct Letter { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord
impl Letter {
    #[must_use] pub fn generator(index: u8) -> Self;
    #[must_use] pub fn inverse_of(index: u8) -> Self;
    #[must_use] pub fn index(self) -> u8;
    #[must_use] pub fn is_inverse(self) -> bool;
    #[must_use] pub fn inverse(self) -> Self;
}

pub struct Word { /* private */ }
// derives Clone, Debug, Default, PartialEq, Eq, Hash
impl FromIterator<Letter> for Word {}
impl Word {
    #[must_use] pub fn new() -> Self;
    #[must_use] pub fn letters(&self) -> &[Letter];
    #[must_use] pub fn len(&self) -> usize;
    #[must_use] pub fn is_empty(&self) -> bool;
    #[must_use] pub fn inverse(&self) -> Self;
    #[must_use] pub fn freely_reduced(&self) -> Self;
    #[must_use] pub fn cyclically_reduced(&self) -> Self;
}

pub struct Relator { /* private */ }
// derives Clone, Debug, PartialEq, Eq, Hash
impl Relator {
    pub fn new(base: Word, exponent: u32) -> Result<Self>;
    #[must_use] pub fn base(&self) -> &Word;
    #[must_use] pub fn exponent(&self) -> u32;
    #[must_use] pub fn expanded_len(&self) -> u64;
}

pub struct Presentation { /* private */ }
// derives Clone, Debug, PartialEq, Eq, Hash
impl Presentation {
    pub fn new(names: Vec<String>, relators: Vec<Relator>) -> Result<Self>;
    #[must_use] pub fn generator_count(&self) -> usize;
    #[must_use] pub fn generator_names(&self) -> &[String];
    #[must_use] pub fn relators(&self) -> &[Relator];
    #[must_use] pub fn display_word<'a>(&'a self, word: &'a Word) -> Option<WordDisplay<'a>>;
    #[must_use] pub fn display_relator<'a>(&'a self, relator: &'a Relator) -> Option<RelatorDisplay<'a>>;
}

pub struct WordDisplay<'a> { /* private */ }
// derives Clone, Copy, Debug
impl core::fmt::Display for WordDisplay<'_> {}

pub struct RelatorDisplay<'a> { /* private */ }
// derives Clone, Copy, Debug
impl core::fmt::Display for RelatorDisplay<'_> {}

pub fn conway_presentation(signature: &escher_signature::Signature) -> Result<Presentation>;

pub struct CosetLimit { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash
impl CosetLimit {
    pub const DEFAULT: Self;
    pub const MAX: Self;
    pub fn new(cosets: u32) -> Result<Self>;
    #[must_use] pub fn get(self) -> u32;
}
impl Default for CosetLimit {}

pub fn enumerate(presentation: &Presentation, subgroup: &[Word], limit: CosetLimit) -> Result<CosetTable>;

pub struct CosetTable { /* private */ }
// derives Clone, Debug, PartialEq, Eq, Hash
impl CosetTable {
    #[must_use] pub fn index(&self) -> usize;
    #[must_use] pub fn generator_count(&self) -> usize;
    #[must_use] pub fn image(&self, coset: u32, letter: Letter) -> Option<u32>;
    #[must_use] pub fn trace(&self, coset: u32, word: &Word) -> Option<u32>;
    #[must_use] pub fn row(&self, coset: u32) -> Option<&[u32]>;
    #[must_use] pub fn permutation(&self, letter: Letter) -> Option<Vec<u32>>;
}

pub struct Error { /* private */ }
// derives Clone, Debug, PartialEq, Eq, and std::error::Error and Display through thiserror
impl Error {
    #[must_use] pub fn kind(&self) -> ErrorKind;
}

#[non_exhaustive]
pub enum ErrorKind {
    InvalidPresentation,
    InvalidWord,
    LimitExceeded,
    CosetLimitReached,
    UnsupportedSignature,
}
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `Letter` | A generator or its inverse | Stores the column as a private `u16`, at most 511. `Ord` orders letters by column, so generator $k$ precedes its inverse, which precedes generator $k + 1$ |
| `Letter::generator`, `Letter::inverse_of` | The constructors | Infallible for every `u8`, because $2 \cdot 255 + 1$ fits `u16` |
| `Word` | A sequence of letters | Any sequence, including the empty word. A word carries no names and may use a generator that a given presentation lacks |
| `Word::freely_reduced` | The free reduction | Freely reduced, computed in one pass with a stack |
| `Word::cyclically_reduced` | The free reduction with matching ends stripped | Cyclically reduced |
| `Relator` | A relator $w^n$ | The base is not empty, has at most 4096 letters, and is kept as given. The exponent is at least 1 |
| `Relator::expanded_len` | The length of the expanded relator | Base length times exponent, which fits `u64` because both factors fit `u32` |
| `Presentation` | Named generators and relators | Every check of Section 4.3 passed. The relators are those given, in the order given |
| `display_word`, `display_relator` | The text of Section 4.4 | `None` when the word or relator uses a generator that the presentation lacks |
| `conway_presentation` | The Conway presentation of Section 5 | Returns an error only for an infinite order |
| `CosetLimit` | A limit on cosets defined | $1 \leq$ value $\leq$ `MAX`. `DEFAULT` is 16384 and `MAX` is 1048576 (Section 6.3) |
| `CosetTable` | A complete coset table | Complete, compressed, and standard (Section 6.2). The number of cosets is stored, so a table with no columns has index 1 |
| `CosetTable::index` | The number of cosets | At least 1 |
| `image`, `trace`, `row`, `permutation` | Reads of the table | `None` when the coset is not below the index or a letter uses a generator that the table lacks |
| `permutation` | The column of a letter as a list | Element $c$ is $c^x$. It is a permutation of $0, \ldots, n - 1$ |
| `Error` | Why an operation failed | Carries the operation and a private detail (Section 8) |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

Every public type is `Send`, `Sync`, and `'static`, except `WordDisplay` and `RelatorDisplay`, which borrow and are `Send` and `Sync`. A compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, and the unit test "Auto traits" of Section 10.1.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/word.rs`: `Letter` and `Word`.
- `src/presentation.rs`: `Relator`, `Presentation`, the name checks, and the display adapters.
- `src/conway.rs`: `conway_presentation`.
- `src/table.rs`: `CosetTable`, `CosetLimit`, and the relabelling of Section 6.2.
- `src/enumerate.rs`: `enumerate`, the steps of Section 7, and the crate-private run statistics.

The integration tests form one binary, `tests/group/main.rs`, with the modules `common`, `examples`, `spherical`, `properties`, and `limits` (TEST-10 of the coding standard).

## 4. Words, Relators, and Presentations

### 4.1. Letters and Words

GRP-2. Every operation on `Letter` and `Word` MUST return without a panic for every input and MUST use no recursion, because words come from untrusted callers and can be long, and a panic or a deep recursion stops the caller (ERR-5 of the coding standard).

The generator index of a letter is a `u8`, so a letter names at most 256 generators, which equals the generator limit of Section 15. This makes the constructors infallible and keeps the column within `u16`.

`freely_reduced` pushes each letter on a stack and pops the top instead when the letter is the inverse of the top. `cyclically_reduced` reduces freely and then strips the first and last letters while they are inverse to each other. `inverse` reverses the word and inverts each letter.

### 4.2. Relators

GRP-4. `Relator::new` MUST return `InvalidPresentation` for an empty base or an exponent of 0, and `LimitExceeded` for a base of more than 4096 letters, because an empty relator has no text and an expanded length of 0, and an unbounded base makes every later step unbounded.

A relator stores its base and exponent and never expands them. A corner or cone order is up to `u32::MAX` (SIG-18 of the signature design), so the relator `g1^4000000000` of `(4000000000)(4000000000)` can be built and printed, and only `enumerate` refuses it, through the limit on expanded letters of Section 15.

`Relator::new` does not reduce its base. The relator $(x x^{-1})^3$ is accepted, and `enumerate` drops it as empty after cyclic reduction (step 1 of Section 7.3).

### 4.3. Presentations

GRP-5. `Presentation::new` MUST check the generator names, the number of generators and relators, and that every letter of every relator names a generator of the presentation, and MUST NOT reduce or reorder a relator, because a caller reads the relators back as it gave them and the text of Section 4.4 is what it gave.

A generator name is 1 to 32 bytes, matches `[A-Za-z][A-Za-z0-9]*`, and differs from every other name of the presentation. The names are therefore printable ASCII, and a caller can print them without escaping. A presentation has at most 256 generators and at most 1024 relators (Section 15). A presentation with no generator and no relator is valid, and it presents the trivial group.

The checks run in this order, and the first failure is returned: the number of generators, each name in order, the number of relators, and each relator in order.

### 4.4. Text Form

GRP-3. `WordDisplay` and `RelatorDisplay` MUST write the text of this section, because `escher-cli` prints it unchanged (CLI-38 of Section 5.7 of `docs/design/cli.md`) and its snapshots fix it.

A word is written as its letters separated by one space. A generator is written as its name, and an inverse as the name followed by `^-1`. The empty word is written `1`. Runs of a letter are not compressed, so the crosscap relator is `z1 z1 w1^-1`.

A relator is written by these rules, applied in order:

1. An exponent of 1 is written as the base alone, such as `x1^-1 y1^-1 x1 y1 a1^-1`, `l1`, or `g1^-1`.
2. A base of one generator with an exponent above 1 is written as the name, `^`, and the exponent, such as `g1^5`.
3. Any other base with an exponent above 1, including a base of one inverse letter, is written in parentheses, followed by `^` and the exponent, such as `(r1 r2)^5` or `(g1^-1)^3`.

Rule 3 makes the text of a single inverse letter unambiguous, because `g1^-1^3` could be read in two ways.

## 5. Conway Presentation

### 5.1. Steps and Names

GRP-6. `conway_presentation` MUST build the presentation of steps 1 to 5 of Section 9.1 of the notation reference ([CH] pp. 255-257), with the features in the order of the accessors of `Signature` and the generator names and relator order of this section, because the text is what `escher info` prints and an order or a name that differs between runs changes every snapshot.

The features are taken in the order handles, cone points, boundaries, and crosscaps, which is the order of the accessors of `Signature` (Section 3.1 of the signature design) and of the global relation in [CBG] (Section 9.1 of the notation reference). [CH] fixes the base point and the left-to-right order in its Figure 21 (Section 9.1 of the notation reference). A canonical signature with a crosscap has no handle (Section 3.2 of the signature design).

Each kind of generator is numbered from 1 across the whole signature. The reflections are numbered across all boundaries. These names are proposed as `DEC-34` (Section 10.2 of the architecture):

| Feature | Generators of [CH] | Names, in this order | Relators, in this order |
| --- | --- | --- | --- |
| Handle $i$ | $X$, $Y$, $\alpha$ | `x<i>`, `y<i>`, `a<i>` | `x<i>^-1 y<i>^-1 x<i> y<i> a<i>^-1` |
| Cone point $i$ of order $A$ | $\gamma$ | `g<i>` | `g<i>^A` |
| Boundary $i$ with $n \geq 1$ corners of orders $a, \ldots, c$ | $P, \ldots, S$, $\lambda$ | `r<k>` to `r<k+n>`, then `l<i>` | `P^2`, `(P Q)^a`, `Q^2`, ..., `(R S)^c`, `S^2`, then `l<i>^-1 P l<i> S^-1` |
| Crosscap $i$ | $Z$, $\omega$ | `z<i>`, `w<i>` | `z<i> z<i> w<i>^-1` |
| Global | | | The product of every Greek generator in feature order, last |

A relation $u = v$ of [CH] becomes the relator $u v^{-1}$. SymmHub's `makePresentation` does the same (Section 12.8 of the SymmHub reference), which is code precedent and not a source (Section 14.1). The direction of the commutator and of the boundary conjugation is that of [CH] (Section 9.1 of the notation reference, and the row of Section 5.10 of the architecture).

A corner relator has the base of two letters and the corner order as its exponent, and a cone relator the base of one letter and the cone order as its exponent. The global relator has exponent 1 and is omitted when the signature has no Greek generator, which is only the trivial signature `1`.

### 5.2. Boundary Without Corners

GRP-7. A boundary without corners MUST give one reflection `r<k>`, with the relators `r<k>^2` and `l<i>^-1 r<k> l<i> r<k>^-1`, because this is the reading of the three presentations of `22*`, `**`, and `*x` that [CBG] prints (pp. 129-131, Section 9.1 of the notation reference), and no source states step 3 for $n = 0$ in general (`UNV-22`).

In those three presentations, the one Latin generator of a boundary is both its first and its last. [SINGERMAN] and [MACBEATH] print no relation $c_{i0}^2 = 1$ for an empty period cycle (Section 9.2 of the notation reference). The relator `r<k>^2` follows [CBG], which writes $P^2 = 1$ for `22*` and `*x`.

### 5.3. Trivial and Non-Compact Signatures

GRP-8. `conway_presentation` MUST return `UnsupportedSignature` for a signature with an infinite order, because no section of the notation reference records the presentation of a cone point or corner of infinite order, and a presentation without a source would give colourings that no test can check.

The trivial signature `1` gives a presentation with no generator and no relator. Its coset table over the trivial subgroup has index 1 and no column.

An order of 1 never reaches this function, because the parser drops it (Section 3.2 of the signature design). `11` and `*11` are not signatures, and `(1)(1)` parses to `1`.

### 5.4. Examples

These are the generators and relators of some Conway presentations, as `escher info --presentation` prints them (Section 5.7 of `docs/design/cli.md`):

| Signature | Generators | Relators |
| --- | --- | --- |
| `*532` | `r1, r2, r3, r4, l1` | `r1^2, (r1 r2)^5, r2^2, (r2 r3)^3, r3^2, (r3 r4)^2, r4^2, l1^-1 r1 l1 r4^-1, l1` |
| `532` | `g1, g2, g3` | `g1^5, g2^3, g3^2, g1 g2 g3` |
| `3*2` | `g1, r1, r2, l1` | `g1^3, r1^2, (r1 r2)^2, r2^2, l1^-1 r1 l1 r2^-1, g1 l1` |
| `x` | `z1, w1` | `z1 z1 w1^-1, w1` |
| `*` | `r1, l1` | `r1^2, l1^-1 r1 l1 r1^-1, l1` |
| `o3*22` | `x1, y1, a1, g1, r1, r2, r3, l1` | `x1^-1 y1^-1 x1 y1 a1^-1, g1^3, r1^2, (r1 r2)^2, r2^2, (r2 r3)^2, r3^2, l1^-1 r1 l1 r3^-1, a1 g1 l1` |
| `3*22x` | `g1, r1, r2, r3, l1, z1, w1` | `g1^3, r1^2, (r1 r2)^2, r2^2, (r2 r3)^2, r3^2, l1^-1 r1 l1 r3^-1, z1 z1 w1^-1, g1 l1 w1` |
| `1` | None | None |

The global relators `l1` of `*532` and `*` and `w1` of `x` are one-letter relators, which Section 7.5 treats.

## 6. Coset Tables

### 6.1. Conventions

GRP-9. A coset table MUST use right cosets and the right action, number the cosets from 0 with coset 0 the subgroup, order its columns generator $k$ then its inverse for $k$ from 0, and store its number of cosets, because every enumeration source uses right cosets and a right action (Section 7.21 of the colouring reference), and a table with no column has no other way to report its index.

The entry in row $c$ and column $x$ is $c^x$, the coset that $x$ takes $c$ to by multiplication from the right ([GAP] Section 47.6-1, Section 7.9 of the colouring reference). A word acts letter by letter from left to right, so `trace(c, w)` is $c^{w_1 w_2 \cdots}$. This column layout is that of [SYMPY-CT] and SymmHub, and of [GAP] apart from numbering from 1 (Section 7.21 of the colouring reference). Right cosets in tables are proposed as `DEC-36` (Section 10.2 of the architecture), and Section 5.10 of the architecture picks left cosets for colourings, so the colourings convert, under `UNV-17` (Section 1.5).

The table is one flat `Vec<u32>` of `index() * 2 * generator_count()` entries in row-major order, beside the stored number of cosets.

### 6.2. Standard Form

GRP-15. `enumerate` MUST return a table that is complete, compressed, and standard, because a caller compares tables for equality to recognize equal subgroups, and a table numbered in the order of the run would make equal subgroups give unequal values.

Complete means that every entry of every row is defined ([SYMPY-CT] lines 121-128, Section 7.9 of the colouring reference). Compressed means that the dead rows are removed and the cosets renumbered without gaps ([BROWN2011] p. 5, Section 7.14 of the colouring reference). Standard is the lenlex standard of [GAP] Sections 47.7 and 47.7-1, which uses all columns (Section 7.14 of the colouring reference).

One relabelling does both, as the last step of the enumeration (step 11 of Section 7.3). It gives new number 0 to coset 0 and then reads the rows in the order of their new numbers. Within a row it reads the columns in order and gives the next new number to each live coset that has none. This is the procedure that [HULPKE2010] states on p. 78, read over all columns instead of the generator columns only (Section 7.14 of the colouring reference). It needs no swap, unlike SymPy's `standardize` ([SYMPY-CT] lines 752-797).

Lemma III.26 of [HULPKE2010] gives a bijection between subgroups of index $n$ and standardized tables, and it is stated for the standard that ignores the inverse columns. No source read states it for the lenlex standard (item 26 of Section 14.1 of the colouring reference). The test "Invariance" of Section 10.1 checks the consequence that this design needs.

### 6.3. Coset Limit

GRP-16. The enumeration MUST count every coset that it defines against the limit, including coset 0 and the cosets that a coincidence later eliminates, and MUST return `CosetLimitReached` when it needs a new coset after it has defined `limit` cosets, because a count of live cosets falls after each coincidence and so bounds neither the memory nor the time of a run.

SymPy compares its limit with the length of its table, which counts every row defined, dead rows included, by the research's own reading of the code (Section 7.16 of the colouring reference). A table of index $n$ has $n$ rows, each defined once, so with coset 0 counted, a limit below $n$ always returns `CosetLimitReached`. `escher-cli` relies on this when it skips the enumeration of a group whose order is above the limit (CLI-39 of Section 5.7 of `docs/design/cli.md`).

The proposed default is 16384 and the proposed maximum 1048576 (`DEC-33`). The largest group order of a domain that `escher-domain` builds is 4000, for `*(1000)22`, so the default leaves a margin of 4 when the enumeration defines the group order and no more. Section 11.1 records the figures of the prototype that support both values. `CosetLimit::new` returns `LimitExceeded` for 0 or a value above `MAX`.

## 7. Coset Enumeration

### 7.1. Strategy

GRP-10. `enumerate` MUST follow the steps of Section 7.3, because each step is tied to its source or to the test that carries it, and a different strategy breaks that tie.

The proposed strategy is Felsch's (`DEC-33`), which defines the first open entry "by rows and within a row by columns", and this "guarantees that the image of each coset under each generator will be defined at some point" ([HULPKE2010] p. 77, Section 7.12 of the colouring reference). That guarantee is the hypothesis of Theorem III.24 of [HULPKE2010] and of Theorem 5.3 of [HOLT2013] with (R4), under which an enumeration of a subgroup of finite index terminates (Section 7.15 of the colouring reference). Holt states that the relator-based strategy does "not necessarily" meet (R4) ([HOLT2013] p. 18). [GAP]'s built-in enumerator is Felsch's (Section 47.6-5).

The known cost of the choice is that a Felsch strategy scans more per definition, and that "some versions of Felsch-type procedures" define a number of cosets exponential in $n$ on Sims's presentation $\langle x, y \mid x^n = y, y^n = x \rangle$ ([HAVAS1991] p. 197, Section 7.20 of the colouring reference).

### 7.2. Data Structures

The enumeration keeps these structures, all as flat vectors with indexes and no references between elements:

- The table: one `Vec<u32>`, with entry `row * columns + column` and `u32::MAX` as the undefined marker. A row is appended at each definition, and no row is reused before the relabelling. Section 15 states how its capacity grows.
- The representatives: a `Vec<u32>` with `p[c] == c` for a live coset ([SYMPY-CT] lines 466-534, Section 7.13 of the colouring reference). The lookup is a loop with path compression.
- The deduction stack: a `Vec<(u32, u16)>` of a coset and a column, with a crate-private cap of 65536 entries (Section 7.6).
- The coincidence queue: a `VecDeque<u32>`. A coset enters it at most once.
- The relators: each expanded, cyclically reduced relator and its inverse, stored twice in a row, so a cyclic conjugate is a pair of a relator and an offset with no test for the end. [HAVAS1991] reports the same device with three copies, as implementations expand "each relator out three times to avoid having to check for end of relator during deduction processing" (p. 195, Section 7.12 of the colouring reference).
- The conjugate lists: for each column $x$, the list $R_x$ of the distinct cyclic conjugates that start with $x$.

GRP-25. The conjugate lists MUST hold each distinct cyclic conjugate once, because a relator such as $(r_1 r_2)^{1000}$ has 2000 cyclic conjugates of which only 2 are distinct, and scanning duplicates multiplies the work by the exponent without changing a table (Section 11.1).

[HULPKE2010] forms "a list of all cyclic permutations of all relators and their inverses" (Note III.22, Section 7.12 of the colouring reference). The prototype gave identical tables with and without duplicates (Section 11.1). SymPy's Felsch enumeration also forms the conjugates as a set before it splits them by first letter, by a reading of [SYMPY-CT] lines 1230-1240 that Section 7.12 of the colouring reference does not record. The lists are built in a fixed order, by relator in input order and then by offset, so the run is deterministic (GRP-18).

Columns of involutions are not merged. [HAVAS1991] notes that a table needs $(2g - i) \cdot n$ words when the inverse column of each of $i$ involutions is dropped (p. 192, Section 7.9 of the colouring reference). Uniform columns keep the layout of Section 6.1, and the relator $x^2$ makes the two columns of $x$ equal. The table of this design holds $2g \cdot n$ words.

### 7.3. Steps

`enumerate` runs these steps. The column "Code-only part" names what no source states at step level, with the test of Section 10.1 that carries it, and `UNV-19` covers it. Every definition, in step 3 or step 5, first checks the limit as step 5 states:

| Step | What | Source | Code-only part |
| --- | --- | --- | --- |
| 1 | Check the limits of Section 15 and the letters of each subgroup generator. Expand each relator, cyclically reduce it, and drop it when empty. Mark the one-letter relators. Form the conjugate lists of Section 7.2 | [HULPKE2010] Note III.22, pp. 75-76, Section 7.12 of the colouring reference | Cyclic reduction first is [SYMPY-CT] lines 1202-1260 and SymmHub (Section 7.19 of the colouring reference), carried by "Worked examples". Distinct conjugates are GRP-25, carried by "Distinct conjugates" |
| 2 | Start with coset 0 and every entry undefined. Apply the rule of Section 7.5 to coset 0 | [HULPKE2010] p. 72, Section 7.9 of the colouring reference | The rule of Section 7.5, carried by "One-letter relators" |
| 3 | Scan and fill each subgroup generator at coset 0. Where a scan has more than one gap, define a new coset at the first undefined position of the forward trace, apply the rule of Section 7.5 to it, and scan again, until the scan completes or deduces. Push every entry written. Then process the deductions (step 6) | Doing this first is [HAVAS1991] p. 193 (Section 7.12 of the colouring reference) | Scan-and-fill is [SYMPY-CT] lines 343-378 and 597-607 (Section 7.11 of the colouring reference), carried by "Worked examples" |
| 4 | Find the first undefined entry among the live cosets, by rows and within a row by columns. If there is none, go to step 10 | [HULPKE2010] p. 77, [HOLT2013] p. 17, Section 7.12 of the colouring reference | |
| 5 | If `limit` cosets have been defined, return `CosetLimitReached` (GRP-16). Otherwise define a new coset $j$ with $i^x = j$ and $j^{x^{-1}} = i$, apply the rule of Section 7.5 to $j$, push $(i, x)$, and process the deductions | [HOLT2013] p. 15, Section 7.10 of the colouring reference. The count of dead rows follows SymPy (Section 7.16 of the colouring reference) | The reading of SymPy's count is the research's own (Section 7.16 of the colouring reference), carried by "Finite at a larger limit" and "Limit at its value" |
| 6 | Process the deductions: while the stack is not empty, apply Section 7.6 when it holds the cap or more, and otherwise pop $(a, x)$ and scan as Section 7.4 states. Then go to step 4 | [HULPKE2010] Note III.22, [BROWN2011] p. 4 | The stack is [SYMPY-CT] lines 673-704 (Section 7.12 of the colouring reference), carried by "Worked examples" and "Closing pass idle" |
| 7 | Scan a word at a coset: trace the longest defined prefix forwards and the longest defined suffix backwards. The scan completes correctly, completes with two different cosets, which is a coincidence (step 8), leaves exactly one gap, which is a deduction written both ways and pushed, or yields nothing | Prose only in [HULPKE2010] p. 75 | The outcomes are [SYMPY-CT] lines 286-379 (Section 7.10 of the colouring reference), carried by "Worked examples" and "Closing pass idle" |
| 8 | Process a coincidence of $\alpha$ and $\beta$: merge them, which finds their representatives and, when these differ, makes the smaller the representative of the larger and queues the larger. While the queue is not empty, remove its first coset $\gamma$. For each column $x$ with $\delta = \gamma^x$ defined, clear $\delta^{x^{-1}}$, and let $\mu$ and $\nu$ be the representatives of $\gamma$ and $\delta$. Then do the first of these that applies. If $\mu^x$ is defined, merge $\nu$ and $\mu^x$. Otherwise, if $\nu^{x^{-1}}$ is defined, merge $\mu$ and $\nu^{x^{-1}}$. Otherwise set $\mu^x = \nu$ and $\nu^{x^{-1}} = \mu$. Push $(\mu, x)$ for each entry written (GRP-12) | [HOLT2013] p. 17, [HULPKE2010] p. 76, Section 7.13 of the colouring reference | The steps are [SYMPY-CT] lines 538-594. The design pushes $(\mu, x)$ for each entry that it writes, where SymPy pushes $(\delta, x^{-1})$. Carried by "Closing pass idle" |
| 9 | Overflow of the stack: Section 7.6 | Lookahead is [BROWN2011] p. 4 (Section 7.11 of the colouring reference) | The overflow policy, which differs from [SYMPY-CT] line 54 and lines 673-704 in its order, carried by "Stack cap one" |
| 10 | Closing pass: Section 7.7. If it changes anything, go to step 4 | [HULPKE2010] p. 75, [HOLT2013] Theorem 5.2, Sections 7.9 and 7.15 of the colouring reference | The pass is this design's safeguard (GRP-14), carried by "Closing pass idle" |
| 11 | Relabel once, as Section 6.2 states, and return the table | [BROWN2011] p. 5, [HULPKE2010] p. 78, [GAP] Sections 47.7 and 47.7-1, Section 7.14 of the colouring reference | SymPy's `compress` and `standardize` use swaps instead ([SYMPY-CT] lines 752-832). The relabelling is carried by "Worked examples" and "Invariance" |

GRP-12. Every entry that a scan, a definition, a coincidence, or the rule of Section 7.5 writes MUST be pushed on the deduction stack, because an entry that is never scanned can leave a relator unchecked at a coset, and the closing pass then has to find the fault. On the prototype, a run without the push of a scan's deduction made the closing pass change the table in some random presentations (Section 11.1).

GRP-13. A coincidence MUST keep the smaller coset number and MUST process the further coincidences through a queue, because coset 0 is the subgroup and must survive every merge, and a recursive merge has no bound on its depth. Holt replaces the larger number by the smaller ([HOLT2013] p. 17), and Hulpke keeps "a list of coincidences to process" ([HULPKE2010] p. 76), as Section 7.13 of the colouring reference records.

### 7.4. Scans of a Deduction

GRP-11. When it pops $(a, x)$, the enumeration MUST scan at $a$, if $a$ is live, every conjugate of $R_x$, and at $b = a^x$, if $b$ is defined and live, every conjugate of $R_{x^{-1}}$, selected by the letter $x$ of the popped entry, reading $b$ after the scans at $a$ and scanning at $b$ only when $a$ is still live, because a faithful rewrite of SymPy's variant, which scans every popped entry with the lists of its caller, never finished on the side presentation of any rotation double that the prototype tried, and without a closing pass it returned wrong tables (Section 11.1).

This is Hulpke's rule: "if the image of coset y under generator g is defined as z, we scan all relators in R^c_g starting at the coset y and all relators in R^c_{g^-1} starting at z" ([HULPKE2010] Note III.22, pp. 75-76, Section 7.12 of the colouring reference). SymPy's `coset_enumeration_c` passes two lists from its caller to its processing of deductions and scans each popped pair with them, whatever the column of the pair ([SYMPY-CT] lines 673-704, Section 7.12 of the colouring reference). No source read says whether that matters (item 23 of Section 14.1 of the colouring reference). The prototype answers the question for this design (Section 11.1). One of the presentations on which the variant never finished is the side presentation of `332`, $\langle s_0, s_1, s_2, s_3 \mid s_0 s_1, s_2 s_3, s_1^3, (s_2 s_0)^2, s_3^3 \rangle$, whose group has order 12.

The scans at $a$ stop when a scan kills $a$ by a coincidence, as SymPy's loop stops ([SYMPY-CT] lines 673-704, Section 7.12 of the colouring reference). Skipping the scan at $b$ then rests on GRP-12, because the coincidence pushes the entries that it writes. This is a reading of this design, which "Closing pass idle" checks.

### 7.5. One-Letter Relators

GRP-26. For each one-letter relator $x$, the enumeration MUST set $c^x = c$ and $c^{x^{-1}} = c$ at coset 0 and at each new coset $c$, and push both entries, because the global relator of a Conway presentation can be such a relator, as the next paragraphs show, and without this rule the enumeration defined about twice the group order on the Conway presentations of the prototype that have a boundary or a crosscap (Section 11.1).

Cannon and Havas state the rule "each coset j must be mapped to itself by each of the defining relators" ([CANNON-HAVAS1992] Section 2.2, p. 3, Section 7.10 of the colouring reference). For a relator of one letter $x$, that rule fixes the entry of column $x$ in every row, and this step writes it when the row is created instead of waiting for a scan. When the cyclically reduced relator is $x^{-1}$, the letter is $x^{-1}$ and the same two entries result. The timing of the step is this design's own, under `UNV-19`. A one-letter relator also stays in the conjugate lists, so the scans of Section 7.4 check it like any other.

The global relator of every Conway presentation with exactly one Greek generator is a one-letter relator: `l1` for `*532`, `*`, and every other signature with one boundary and no other feature, `w1` for `x`, and `a1` for `o`. Without the rule, Felsch defines $c^{l_1}$ as a new coset at each coset $c$ and finds the coincidence afterwards.

### 7.6. Overflow of the Deduction Stack

GRP-27. When the deduction stack holds its cap or more at the top of step 6, the enumeration MUST first empty the stack and then run a lookahead that scans every relator at every live coset without a definition, pushing every entry that the lookahead writes, and the cap MUST be a crate-private parameter that a test can set to 1, because the overflow path never runs at the default cap on any input that the prototype tried, and only a small cap exercises it.

The lookahead is Brown's: "we check all relators at all vertices (but without making new definitions) and take account of any deductions or coincidences that we can discover" ([BROWN2011] p. 4, Section 7.11 of the colouring reference). This design keeps the entries that the lookahead writes on the stack, so that their scans still run, which is how it reads "take account of any deductions". SymPy runs the lookahead first and then empties the stack, which drops the entries that the lookahead pushed ([SYMPY-CT] lines 673-704, Section 7.12 of the colouring reference). The order is a choice of this design (Section 14.2). On the prototype, both orders gave correct tables at small caps (Section 11.1).

The cap is checked only at the top of step 6, so one coincidence or one lookahead can push the stack past the cap. Section 15 states the bound on the memory of the stack that follows.

### 7.7. Closing Pass

GRP-14. `enumerate` MUST return a table only after a closing pass has scanned every relator at every live coset and every subgroup generator at coset 0, without a definition, and found that each completes correctly, because the correctness of a returned table then rests on the sourced statement about closed tables and not on any step that only code states.

When "all tables are closed and no deductions are pending", "the columns of the coset table give permutation images for the group generators that are consistent with all relators", which gives a homomorphism to $S_n$ with $[G : S] = n$ equal to the number of rows ([HULPKE2010] p. 75, Section 7.9 of the colouring reference). Holt's Theorem 5.2 gives the action on the right cosets when the process stops with every entry defined ([HOLT2013] p. 16, Section 7.15 of the colouring reference). That the table keeps Holt's properties (R1) to (R3) through coincidences rests on "It can be shown" ([HOLT2013] pp. 16-17), so it is part of `UNV-19`.

The pass runs when step 4 finds no undefined entry. Each scan of a complete table either completes correctly or finds a coincidence. When it finds one, the pass processes it and the pushed deductions and returns to step 4. The crate-private run statistics count the cosets defined and the changes that the pass finds.

On the prototype, the closing pass changed nothing in a correct implementation over every input tried, and it found the faults of the variants of Sections 7.3 and 7.4 that the prototype introduced (Section 11.1). The pass is therefore a safeguard against a defect of the code-only steps, and the property test "Closing pass idle" of Section 10.1 is what shows that those steps do their part.

### 7.8. What Reaching the Limit Shows

GRP-17. An error text, a documentation comment, or a document of this crate MUST NOT state that `CosetLimitReached` shows the index or the group to be infinite, because no source read states what a run that reaches its limit shows (item 19 of Section 14.1 of the colouring reference), and ACE's manual shows a subgroup of finite index whose enumeration overflows at a small limit and succeeds at a larger one ([ACE] Appendix B.4, read through a summarizing tool, Section 7.15 of the colouring reference).

The documentation of `CosetLimitReached` states that the run defined the limit and needed more, and that a larger limit may or may not finish. Holt states that if the index is infinite the procedure "will not halt", and that for a finite index "there is no upper bound on the time taken" ([HOLT2013] p. 15, Section 7.15 of the colouring reference). The test "Finite at a larger limit" of Section 10.1 shows a finite group that reaches a small limit.

### 7.9. Determinism and Cost

GRP-18. The returned table MUST be a function of the presentation, the subgroup words, and the limit alone, because callers compare tables across runs, and a table that depends on a hash seed, a thread schedule, or a clock cannot be compared.

The enumeration iterates no hash map, uses no randomness, and starts no thread. The standard form of Section 6.2 also makes the table independent of the order of the relators and of the subgroup generators, which the test "Invariance" checks. That independence relies on Lemma III.26 for the lenlex standard, which is unverified (Section 6.2).

The sources give these two facts about cost, and no other (Section 7.2 gives the size of a table):

- For a given procedure "there is no computable bound, in terms of length of input and a hypothetical index, to the number of cosets which need to be defined" ([CANNON-HAVAS1992] Section 2.2, p. 3, Section 7.15 of the colouring reference).
- An enumeration "may take many more intermediate cosets than the actual index" ([GAP] Section 47.6-6).

## 8. Errors

### 8.1. Structure

GRP-22. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, and its `Display` text MUST be `cannot `, the operation, `: `, and the text of Section 8.2, because callers match on the kind, and `escher-cli` prints the text as a diagnostic that its snapshots fix.

`Error` has two private fields:

- `operation`: A `&'static str`, which is `"build presentation"` for `Relator::new`, `Presentation::new`, and `conway_presentation`, and `"enumerate cosets"` for `enumerate` and `CosetLimit::new`.
- `detail`: A private enum with one variant for each cause, carrying the numbers that its text shows.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives `Display` and `std::error::Error` on `Error` and on the detail enum. No error of this crate has a cause, so `source()` returns `None`. No type from a dependency appears in the error (ERR-2).

The error never holds a generator name or a word. It shows positions and numbers only, and the canonical string of a signature for `UnsupportedSignature`, which is ASCII by construction (Section 7.3 of the signature design). A position in the text counts from 1.

### 8.2. Kinds and Text

| Kind | When | Text after the prefix, for example |
| --- | --- | --- |
| `InvalidPresentation` | An empty relator base, an exponent of 0, a bad or repeated name, or a relator letter outside the generators | `the relator base is empty`, `the relator exponent is 0`, `generator name 3 is not a letter followed by up to 31 letters and digits`, `generator name 4 repeats generator name 2`, `relator 3 uses a generator that the presentation does not have` |
| `InvalidWord` | A subgroup generator with a letter outside the generators | `subgroup generator 2 uses a generator that the presentation does not have` |
| `LimitExceeded` | An input limit of Section 15, or a coset limit out of range | `the relator base has more than 4096 letters`, `the presentation has more than 256 generators`, `the presentation has more than 1024 relators`, `the subgroup has more than 256 generators`, `subgroup generator 2 has more than 4096 letters`, `the relators expand to 120011 letters, more than 65536`, `the coset limit 0 is not between 1 and 1048576`, `the coset limit 1048576 times 32 columns is more than 16777216 table entries` |
| `CosetLimitReached` | Step 5 of Section 7.3 needs a coset after `limit` were defined | `the enumeration defined 16384 cosets without finishing` |
| `UnsupportedSignature` | A signature with an infinite order | `the signature *(inf)22 has an infinite order` |

With the prefix, the full texts read, for example, `cannot build presentation: the signature *(inf)22 has an infinite order` and `cannot enumerate cosets: the enumeration defined 16384 cosets without finishing`. Every text is in lower case, with no trailing punctuation (ERR-4 of the coding standard). No text cites a document or a rule identifier.

## 9. Resource Limits

The crate limits the generators, relators, letters, subgroup generators, expanded letters, cosets, and table entries of every input. `Relator::new` and `Presentation::new` check their limits at construction, and `enumerate` checks its limits at step 1 of Section 7.3, before any allocation that depends on them. Section 15 states the values, the memory bound, and the residual risk on time, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 10. Testing

### 10.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard), and the integration tests form the one binary of Section 3.3. The test "Side orders", which serves the second item of the field "Tests and acceptance" of Section 5.7 of the plan, belongs to `escher-domain` (Section 12.1 of `docs/design/domain.md`).

Each test names the item of that field that it serves, or None. Every test stays well under a minute in a debug build:

| Test | Plan item | Kind and place | Asserts | Source |
| --- | --- | --- | --- | --- |
| Spherical orders | Spherical types | Table test, `spherical` | The Conway presentation of each of the 7 series for $n$ from 1 to 12 and of the 7 particular types, enumerated over the trivial subgroup at the default limit, has an index equal to `group_order()`. The series are written with one digit below 10 and parentheses from 10, and $n = 1$ as `(1)(1)` for `nn` and `*(1)(1)` for `*nn`, because `11` and `*11` are not signatures. The 91 strings give `1` and 86 other distinct canonical signatures, and the test runs each once | Sections 7.3 and 8.2 of the notation reference |
| Order two | Order two | Table test, `spherical` | `x`, `22`, and `*` each enumerate to index 2 | [CH] p. 257, Sections 4.2 and 8.2 of the notation reference |
| Conway words | None | Unit test in `conway.rs` | The generators and relators of every row of Section 5.4, compared as text. Between them `o3*22` and `3*22x` have every kind of feature, so a reversed commutator, a reversed conjugation, a missing global relator, or a wrong exponent fails | Section 9.1 of the notation reference |
| Infinite groups | Bound | Table test, `limits` | The Conway presentations of `*632`, `2222`, `o`, and `*732` at a limit of 1000 return `CosetLimitReached` | GRP-16, GRP-17 |
| Limit at its value | Bound | Table test, `limits` | `532` finishes at a limit of 60 and returns `CosetLimitReached` at 59 | GRP-16 |
| Finite at a larger limit | Bound | Table test, `limits` | For Sims's presentation with $n$ from 3 to 8 over the trivial subgroup, the index is $n^2 - 1$ at the default limit. At a limit equal to the number of cosets that the run defines it finishes, and at one less it returns `CosetLimitReached`. For $n \geq 5$ that number exceeds the index, so a finite group reaches a limit above its order | [HAVAS1991] p. 197, Section 7.20 of the colouring reference, GRP-17 |
| Worked examples | None | Table test, `examples` | The fixtures of Section 10.2 | Section 7.20 of the colouring reference |
| One-letter relators | None | Unit test in `enumerate.rs` | The Conway presentations of `*532`, `x`, and `*`, whose global relators `l1`, `w1`, and `l1` are one-letter relators (Section 5.4), each enumerate over the trivial subgroup, and the number of cosets defined, read from the run statistics, equals the group order | GRP-26 |
| Distinct conjugates | None | Unit test in `enumerate.rs` | The conjugate lists of `(r1 r2)^1000` hold one entry in each of the lists of $r_1$, $r_1^{-1}$, $r_2$, and $r_2^{-1}$ | GRP-25 |
| Stack cap one | None | Unit test in `enumerate.rs` | With the cap set to 1, the worked examples other than Cox and B_2_4, and the Conway presentation of `*532`, give the same tables as at the default cap | GRP-27 |
| Closing pass idle | None | `proptest`, unit test in `enumerate.rs` | For random presentations with 2 or 3 generators and relators of up to 6 letters, at a limit of 2000, the closing pass changes nothing in any run, read from the run statistics, both at the default cap and at a cap of 1 | GRP-12, GRP-14, GRP-27 |
| Table laws | None | `proptest`, `properties` | For the same random presentations, a returned table has columns that are permutations, inverse columns that are mutually inverse, every relator tracing every coset to itself, every subgroup generator fixing coset 0, a transitive action, and the standard form | Sections 7.9 and 7.14 of the colouring reference |
| Invariance | None | `proptest`, `properties` | Over the finite presentations of the fixtures and of the spherical signatures up to order 24, a permutation of the relators, a rotation or an inversion of a relator, and a reordering of the subgroup generators give an equal table | GRP-15, GRP-18 |
| Word laws | None | `proptest`, `properties` | Free reduction is idempotent, a word followed by its inverse reduces to the empty word, and a cyclically reduced word has no cancelling ends | Section 4.1 |
| Text form | None | Unit tests in `presentation.rs` | Each rule of Section 4.4, including `(g1^-1)^3`, the empty word `1`, and `None` for a word with a letter outside the generators | GRP-3 |
| Input limits | None | Unit tests in `presentation.rs` and `enumerate.rs` | Each limit of Section 15 at its value is accepted and one above returns `LimitExceeded`. `Relator::new` with an exponent of 0 and an empty base, and each name rule, return `InvalidPresentation` | GRP-4, GRP-5, GRP-19 |
| Error text | None | Unit test in `error.rs` | The `Display` of every kind starts with `cannot build presentation: ` or `cannot enumerate cosets: ` and is in lower case | GRP-22 |
| Zero generators | None | Unit test in `table.rs` | The presentation with no generator enumerates to a table of index 1 with no column, and `row(0)` is an empty slice | GRP-9 |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs` | Each public type has the auto traits of Section 3.2 | CONC-1 of the coding standard |
| Doctests | None | Doctests | The examples of the crate-level documentation | Sections 7.4 and 7.5 of `docs/writing-documentation.md` |

The property tests commit their regression files (TEST-9 of the coding standard). "Closing pass idle" runs as a unit test, because the run statistics are crate-private, and it loads the `proptest` helper of `common` through a `#[path]` module, as `escher-render` loads its golden comparison.

This design leaves out a property test that the index of a cyclic subgroup times its order equals the order of a finite group, because it relies on Lagrange's theorem, which no section of `docs/research/` states.

GRP-24. The worked examples of Section 10.2 and the property test "Closing pass idle" MUST stay in the crate while `UNV-19` is open, because they are what checks the steps that only code states, and without them a change to a step that keeps every order right but breaks a table can pass.

### 10.2. Worked Examples

The test "Worked examples" compares each fixture of Section 7.20 of the colouring reference as a standard table, with 1 subtracted from every number where the source numbers from 1, and with the source's columns mapped to the order of Section 6.1. Where the source prints only rows or an index, the test asserts only that:

| Source | Presentation and subgroup | Asserts |
| --- | --- | --- |
| [BROWN2011] p. 5 | $\langle a, b \mid a^3, b^3, (ab)^2 \rangle$, $\langle a \rangle$ | Index 4 and the printed table |
| [HULPKE2010] pp. 72-75 | $\langle a, b \mid a^2, b^3, (ab)^5 \rangle$, $\langle a, b^{-1} a b \rangle$ | Index 6, the printed table, $a \mapsto (1,3)(4,5)$ and $b \mapsto (0,1,2)(3,4,5)$. Over the trivial subgroup, index 60 (Example III.35, p. 84) |
| [GAP] Sections 47.6-1 and 47.7-2 | $\langle a, b \mid a^2, b^3, (ab)^5 \rangle$, $\langle a, b a b a b^{-1} \rangle$ | Index 5 and the lenlex rows `[0,0,1,2]`, `[3,3,2,0]`, `[4,4,0,1]`, `[1,1,3,3]`, `[2,2,4,4]`, which differ from the semilenlex rows |
| [HOLT2013] p. 15 | $\langle x, y \mid x^2, y^3, xyxy \rangle$ | Index 3 over $\langle xy \rangle$. Over $\langle yx \rangle$, index 3 with $x \mapsto (0,1)$ and $y \mapsto (0,1,2)$, the table that the notes print |
| [HOLT2013] pp. 16-17 | $\langle x, y \mid x^3, y^4, (xy)^2 \rangle$, $\langle y \rangle$ | Index 6 |
| [HOLT2013] p. 17 | $\langle a, b \mid a^{-1}ba = b^3, b^{-1}ab = a^3 \rangle$, $\langle a \rangle$ | Index 2, and index 8 over the trivial subgroup |
| [HOLT2013] pp. 21-22 | $\langle a, b \mid a^3, b^5, (ab)^2 \rangle$, $\langle b, ab^{-1}ab^2a^{-1} \rangle$ | Index 6, and index 60 over the trivial subgroup |
| [HOLT2013] pp. 22-23 | $\langle x, y \mid x^3, y^5, (xy)^2 \rangle$, $\langle xy, x^{-1}y^{-1}xyx \rangle$ | 5 rows |
| SymPy "Example 5.1" | $x^3, y^3, x^{-1}y^{-1}xy$, $\langle x \rangle$ | The 3 printed rows |
| SymPy "Example 5.2" | $x^2, y^3, (xy)^3$, $\langle xy \rangle$ | The 4 printed rows |
| SymPy "Example 5.3" | $x^2y^2, x^3y^5$, trivial | The 4 rows printed after `standardize` |
| SymPy "Example 5.4" | $abc^{-1}, bcd^{-1}, cde^{-1}, dea^{-1}, eab^{-1}$, $\langle a \rangle$ | 1 row |
| SymPy "Exercises Pg. 161, Q2" | $x^2y^2, y^{-1}xyx^{-3}$, trivial | The 8 rows printed after `compress` and `standardize` |
| SymPy "Table 1. Ex. 1" of [CDHW1973] | $t^{-1}rtr^{-2}, r^{-1}srs^{-2}, s^{-1}tst^{-2}$, $\langle r \rangle$ | 1 row |
| SymPy Cox | $a^6, b^6, (ab)^2, (a^2b^2)^2, (a^3b^3)^5$, $\langle a \rangle$ | 500 rows |
| SymPy B_2_4 | The 9 relators of Section 7.20 of the colouring reference, $\langle a \rangle$ | 1024 rows |
| [ACE] Appendix B.3 | $\langle a, b \mid a^2, b^3, (ab)^5 \rangle$, $\langle ab \rangle$ | Index 12 (read through a summarizing tool) |

The relations of Holt's quaternion example are written as the relators $a^{-1}bab^{-3}$ and $b^{-1}aba^{-3}$ (Section 5.1). The SymPy examples come from doctests of the HLT enumeration, so only their standard rows and their indices transfer to this design, and not their counts of cosets defined (Section 7.20 of the colouring reference). The prototype matched every row of this table (Section 11.1).

The Holt fixture of p. 15 prints $x \mapsto (1,2)$ and $y \mapsto (1,2,3)$ for the subgroup $\langle xy \rangle$. Under the right action of this design that is the standard table of $\langle yx \rangle$, and for $\langle xy \rangle$ the standard table has $y \mapsto (0,2,1)$ when numbered from 0. The index is 3 either way, so the fixture asserts the printed table for $\langle yx \rangle$ and only the index for $\langle xy \rangle$.

### 10.3. Defects the Tests Catch

This subsection is non-normative. It lists deliberate defects that a reviewer can introduce to check the tests, with the test that fails:

| Defect | Test that fails |
| --- | --- |
| Keep the larger coset number in a merge | Worked examples, because coset 0 dies and the table is inconsistent |
| Do not push the deduction of a scan | Closing pass idle (Section 11.1) |
| Scan each popped entry with the lists of the caller, as SymPy does | Closing pass idle (Section 11.1) |
| Standardize over the generator columns only | Worked examples, through the lenlex rows of [GAP] |
| Count live cosets against the limit | Finite at a larger limit |
| Omit the global relator, use twice the order as a corner exponent, or drop the last `S^2` of a boundary | Conway words |
| Omit the rule of Section 7.5 | One-letter relators |
| Form every cyclic conjugate instead of the distinct ones | Distinct conjugates |

No test can detect three defects, because on the prototype each left every table and every closing pass unchanged: dropping the scan at $b$ of Section 7.4, dropping the push of a coincidence, and dropping the closing pass in an implementation that has no other defect (Section 11.1). They are not test evidence. The scan at $b$ is redundant because the lists hold the conjugates of every relator and of its inverse, and the closing pass and the pushes stay for the soundness reasons of GRP-12 and GRP-14.

### 10.4. Tests Under Miri

`DEC-5` and TEST-11 of the coding standard apply in this way:

| Tests | Under Miri | Attribute or configuration |
| --- | --- | --- |
| Order two, Conway words, Text form, Input limits, Error text, Zero generators, Distinct conjugates, Auto traits | Run | None |
| The single cases `2*3` and `3*2` of "Spherical orders" | Run | Separate tests outside the table test |
| One case of "Infinite groups" at a limit of 50 | Run | A separate test outside the table test |
| Word laws | Run, with fewer cases | The helper of GRP-28 |
| Spherical orders, Worked examples, Infinite groups, Limit at its value, Finite at a larger limit, One-letter relators, Stack cap one, Closing pass idle, Table laws, Invariance | Skipped | `#[cfg_attr(miri, ignore = "the enumeration runs a loop too long for the interpreter")]` |
| Doctests | Not run | Miri runs the tests through nextest, which runs no doctest |

The Conway presentation of `*532` and the small worked examples are not run under Miri, because they were too slow there on the prototype (Section 11.1). Each reason in an attribute is in plain words and cites no document.

GRP-28. The property tests MUST take their configuration from one helper in `common`, with the numbers of cases and the failure persistence that Section 4 of `docs/design/tooling.md` states, because the file persistence of `proptest` asks for the current directory, which the isolation of Miri refuses, and a full set of cases takes too long in the interpreter.

## 11. Prototype Measurements

### 11.1. Figures of the Prototype

These statements of this design rest on figures that no test of the crate measures. They come from a prototype outside the repository that implemented the steps of Section 7 with switches for the variants that this design rejects, through the public surfaces of `escher-signature`, `escher-geometry`, and `escher-domain`. It was built with cargo 1.99.1 on a host with 16 cores. Times are wall-clock times in a release build unless marked debug, within about 30 %. Each figure supports only the statement that it names:

| Statement | Section | Figures of the prototype |
| --- | --- | --- |
| Scans selected by the letter of the popped entry | 7.4 | The caller-list variant, alone, with SymPy's push, or with SymPy's main loop and a cap of 100, never finished on the side presentation of any rotation double tried (`332`, `432`, `532`, `222`, `n22` for $n$ from 3 to 12, and `(1000)22`), and reached the limit of 16384. Over 3000 random presentations, the outcome differed in 80 runs, and without a closing pass 107 tables were wrong. The selected lists defined 5220 cosets in total over those runs and the caller lists 5499 |
| The closing pass changes nothing in a correct implementation | 7.7 | No change over 23 fixtures, 193 corpus cases of the spherical sweep, the side presentations, and their Conway presentations, 8000 random presentations, and Sims's presentation for $n$ from 3 to 16, with the push of Section 7.3 or with SymPy's push. With the push of a scan's deduction removed, it changed the table in 32 of 3000 random runs, and with both the scan at $b$ and the push of a coincidence removed, in 2 of 3000 |
| Undetectable defects | 10.3 | Without the scan at $b$, or without the push of a coincidence, every table was identical and no closing pass changed anything over the fixtures, the corpus, and 3000 random presentations |
| One-letter relators | 7.5 | Without the rule, every Conway presentation with a boundary or a crosscap defined about $2N - 1$ cosets for order $N$, the largest ratio was 1.992 at `*532` (239 for 120), and `*(4096)22` needed 32767 cosets. With the rule, the run defined exactly $N$ cosets on the 193 corpus cases, with identical tables, and identical tables on 2000 random presentations |
| Distinct conjugates | 7.2 | Identical tables with and without. The Conway presentation of `*(1000)22` took 0.115 s with distinct conjugates and 81.1 s without (0.39 s and 270 s in debug), and that of `(1000)22` 9 ms and 4.3 s |
| Order of the overflow steps | 7.6 | Both orders gave correct tables at caps of 100, 4, and 1. The largest stack in any run was 67 entries. The corpus at a cap of 1 took 195 s, against 0.57 s at the default cap |
| Default limit | 6.3 | Without the rule of Section 7.5, the spherical sweep enumerated to $2/\chi$ in 1.6 ms in total (11 ms in debug), and the Conway presentation of `*(1000)22` defined 7999 cosets in 115 ms (392 ms in debug) with a peak of 0.57 MiB. The side presentations of the 52 spherical domains that `escher-domain` builds defined exactly $N$ cosets each, in 122 ms in total. `*632`, `2222`, `o`, and `*732` reached a limit of 16384 within 5 ms (36 ms in debug) and 1.2 MiB, and a limit of 1000 within 0.31 ms |
| Maximum limit | 6.3 | A full index of $2^{20}$ took 1.45 s and 76.00 MiB for $\mathbb{Z}_{32}^4$ with 8 columns, and 1.59 s and 92.01 MiB for $\mathbb{Z}_{16}^5$ with 10 columns |
| Memory bound | 15 | With the capacity of the table capped at limit times columns, the peaks of the previous row were within the bound that the formula of Section 15 gives for them, 76.5 MiB and 92.5 MiB. With a plain doubling vector, `****` with 16 columns at a limit of 540000 peaked at 98 MiB against a bound of 72.6 MiB from the same formula |
| Limit counting | 6.3 | With coset 0 counted, `532` finished at a limit of 60 and failed at 59 |
| Sims's presentation | 10.1 | Cosets defined for $n$ from 3 to 16: 8, 15, 28, 47, 83, 138, 243, 406, 718, 1205, 2138, 3597, 6393, and 10768, against orders from 8 to 255. A limit of one less than the count failed and the count succeeded, for every $n$. $n = 17$ does not finish at 16384 |
| Worked examples | 10.2 | All 23 fixtures matched in index and, where printed, in the standard table. Cox defined 500 cosets in 0.74 ms (3.0 ms in debug). B_2_4 defined 1225 cosets with 24 coincidences, in 2.0 ms (10.5 ms in debug). The [GAP] table is not the semilenlex one |
| Holt p. 15 | 10.2 | The printed table is the standard table of $\langle yx \rangle$. For $\langle xy \rangle$ it is $x \mapsto (1,2)$, $y \mapsto (1,3,2)$, numbered from 1 |
| Invariance | 6.2 | 1400 variants over 22 presentations, by relator permutation, rotation, inversion, and subgroup reordering, gave no different table |
| Conway text | 5.4 | The generators and relators that the prototype printed for `*532`, `532`, `3*2`, `x`, `*`, and `1` matched the rows of Section 5.4 character for character. Section 5.7 of `docs/design/cli.md` states which texts of `info --presentation` the prototype reproduced |
| Residual risk on time | 15 | The slowest input found within every limit, 2 generators with 16 random cyclically reduced relators of 4096 letters over the trivial subgroup at the default limit, took 20.3 s and 3.5 MiB and reached the limit. With 4 such relators it took 5.1 s, with 1 it took 1.3 s, and with 128 generators 0.25 s. The Conway presentation of a spherical signature of order $N$ expands to at most $2N + 11$ letters |
| Expanded letters | 15 | The relators of `*(1000)32` expand to 2023 letters, and those of `*(1000)22` to 2021. `*(30000)(30000)` expands to 120011 letters and is refused |
| Tests under Miri | 10.4 | With one thread: the Conway presentation of `*532` 19.7 s, the small worked examples together 15.6 s, "Infinite groups" at a limit of 50 3.8 s, `3*2` 2.7 s, `2*3` 1.7 s, and `x`, `22`, and `*` 1.3 s |

The prototype did not run SymPy or GAP. Its comparisons use the printed tables of Section 7.20 of the colouring reference and a rewrite of SymPy's routines from its source.

### 11.2. Figures of the Crate

These figures of the crate itself are to be measured. The change that adds the test that measures each one records the figure in this table:

| Figure | Test |
| --- | --- |
| Time of each test in a debug build | Every test of Section 10.1 |
| Cosets defined for each worked example | Worked examples |
| Cosets defined for the Conway presentations of `*532`, `x`, and `*` | One-letter relators |
| Cosets defined for Sims's presentation for $n$ from 3 to 8, which set the limits of "Finite at a larger limit" | Finite at a larger limit |
| Time of each test under Miri | Every test that runs under Miri in Section 10.4 |
| Peak memory at `CosetLimit::MAX`, against the bound of Section 15 | A test that the change chooses, skipped under Miri |

## 12. Dependencies

GRP-23. The crate MUST declare `escher-signature` and `thiserror` as its only normal dependencies and `proptest` as its only dev-dependency, because each further dependency adds build time, licence obligations, and supply-chain risk (DEP-2 of the coding standard), and no usable crate for presentations or coset enumeration exists (Section 14 of `docs/research/rust-ecosystem.md`).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | `conway_presentation` reads a `Signature` |
| `thiserror` | Normal | Default | Derives `Display` and `std::error::Error`, as the other library crates do. Section 3.1 of the architecture lists it for this crate |
| `proptest` | Dev | `default-features = false`, with `std` | The property tests |

The crate sets `publish = false`, inherits the workspace fields and lints, and declares no Cargo feature. `Signature` appears in the public surface as the argument of `conway_presentation`. That is a dependency inside the workspace, which is not published (`DEC-22`). `algebraeon-groups` is under GPL-3.0-only, and no code is copied from it (Section 5.7 of the plan).

## 13. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, and what the code does about it:

| Label | What the code relies on | What the code does |
| --- | --- | --- |
| `UNV-19` | The steps that only implementations or this design state: the outcomes of a scan, scan-and-fill, the deduction stack, the coincidence queue, compression and standardization as steps, the overflow policy of Section 7.6, the timing of the rule of Section 7.5, and the distinct cyclic conjugates of Section 7.2 (item 23 of Section 14.1 of the colouring reference) | Ends every run with the closing pass of GRP-14, so a returned table rests on the closed-table statement of [HULPKE2010] p. 75. The worked examples and "Closing pass idle" check the steps (GRP-24) |
| `UNV-19` | That (R1) to (R3) survive coincidences, which [HOLT2013] states without proof | The same as the previous row |
| `UNV-19` | What a run that reaches its limit shows (item 19 of Section 14.1 of the colouring reference) | Claims nothing about the index (GRP-17) |
| `UNV-19` | Termination for a finite index under Mendelsohn's condition, from lecture notes whose primary texts were not read (item 21 of Section 14.1 of the colouring reference) | Bounds every run by the coset limit, so termination of a run never depends on it |
| `UNV-19` | Lemma III.26 for the lenlex standard (item 26 of Section 14.1 of the colouring reference) | The test "Invariance" checks that equal subgroups give equal tables over its corpus |
| `UNV-19` | The group orders and indices of the worked examples that no source states (item 25 of Section 14.1 of the colouring reference) | Asserts only the rows and indices that the sources print (Section 10.2) |
| `UNV-22` | The Conway presentation of a boundary without corners in general | Implements the reading of GRP-7. The test "Spherical orders" checks `n*` and `2*n` against $2/\chi$, and "Order two" checks `*` |

The crate does not rely on `UNV-17`, because its tables use right cosets as every source does, and the conversion to the left cosets of the colourings is not part of it. It does not rely on `UNV-32`, which concerns the side presentation of `escher-domain` (Section 7 of the plan).

## 14. Assumptions, Choices, and Points for the Owner

### 14.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- A relation $u = v$ of [CH] is the relator $u v^{-1}$. SymmHub's code does the same (Section 12.8 of the SymmHub reference). The tests "Spherical orders" and "Order two" check the result for every spherical type, and a source that states the conversion would settle it.
- The groups of the Euclidean and hyperbolic signatures `*632`, `2222`, `o`, and `*732` are infinite, so their enumeration reaches any limit. The test "Infinite groups" relies on it, and only reports a run that reached its limit (GRP-17). A source that states that a Euclidean or hyperbolic orbifold group is infinite would settle it.
- Writing each entry of a one-letter relator when its row is created gives the same table as finding the entry by a scan. Rule (b) of [CANNON-HAVAS1992] forces the entry (Section 7.5), and the closing pass and the test "One-letter relators" check it. Chapter 5 of [HOLT2005] or Chapter 5 of [SIMS1994], which `UNV-19` names, would settle it.

### 14.2. Choices of This Design

Every choice in this table is a proposed default. No other document dictates it, and the owner has decided none of them:

| Choice | Alternative | Section |
| --- | --- | --- |
| Felsch only, no HLT (`DEC-33`) | HLT as a second strategy, which Holt says may not meet (R4) | 7.1 |
| Coset limit default 16384 and maximum 1048576, with a bound of $2^{24}$ table entries (`DEC-33`) | Other values, measured by the same method | 6.3, 15 |
| Generator names `x1`, `y1`, `a1`, `g1`, `r1`, `l1`, `z1`, `w1`, and the relator text of Section 4.4 (`DEC-34`) | The letters of [CH] with indexes | 4.4, 5.1 |
| Right cosets and the column order of [SYMPY-CT] in tables (`DEC-36`) | Left cosets, which no enumeration source uses | 6.1 |
| A relator as base and exponent, expanded only in `enumerate` | Flat words with a limit, which refuse large orders even for printing | 4.2 |
| A `u8` generator index in `Letter` | A fallible constructor over a wider index | 4.1 |
| `UnsupportedSignature` for an infinite order | A presentation without a source | 5.3 |
| Scans selected by the letter of the popped entry | SymPy's caller lists, which failed on the prototype | 7.4 |
| A closing pass at the end of every run | No pass, which leaves correctness on the code-only steps | 7.7 |
| The rule for one-letter relators | Leaving them to the scans, which doubles the cosets defined for presentations with a boundary or crosscap | 7.5 |
| Distinct cyclic conjugates | Every conjugate, as Hulpke's note lists them | 7.2 |
| Columns of involutions not merged | Merged columns, which halve the table of a reflection group and complicate every index | 7.2 |
| A stack cap of 65536, then an empty stack and a lookahead that keeps its pushes | No cap, or SymPy's order of lookahead then empty | 7.6 |
| The design pushes $(\mu, x)$ in a coincidence | SymPy's push of $(\delta, x^{-1})$, which gave the same results on the prototype | 7.3 |
| The capacity of the table capped at limit times columns | A plain doubling vector, which exceeds the memory bound | 15 |
| No work budget | A budget in letter steps with its own error kind | 15 |
| `thiserror` for the error | An error implemented by hand | 12 |
| The operations `build presentation` and `enumerate cosets`, and positions counted from 1 in error texts | Other wording | 8 |
| `Option` from `permutation`, `display_word`, and `display_relator` | A panic or an empty result for a letter outside the table | 3.2 |
| Run statistics crate-private | A public type for the statistics | 3.1 |
| Subgroup words scanned as given, without reduction | Freely reducing them first | 7.3 |

### 14.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Time of an enumeration. The input limits do not bound the time of a run, and the slowest input found within every limit took 20.3 s in a release build, as measured on a prototype outside the repository (Section 11.1). `escher-cli` cannot reach such an input. The design accepts this residual risk (Section 15). The alternative is a work budget in letter steps, with its own error kind, which bounds the time of every call.
2. Strategy and limits. The design proposes Felsch with a default limit of 16384 and a maximum of 1048576 (`DEC-33`). The alternative is other values, or HLT as a second strategy.
3. Cosets. The tables use right cosets, as every source does, while Section 5.10 of the architecture picks left cosets for colourings (`DEC-36`). The alternative is tables of left cosets, which no enumeration source supports.
4. Names. The design proposes ASCII names numbered by kind (`DEC-34`). The alternative is the letters of [CH], such as `P`, `Q`, and `lambda`, with indexes.

## 15. Security Considerations

The threat model is a caller that passes a presentation, subgroup words, a coset limit, or a signature from an untrusted party to this crate. The attacker can choose any value of the public types. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of memory or time, a panic that stops the caller, and an arithmetic overflow that yields a wrong table.

GRP-19. The crate MUST enforce the limits of this table, return `LimitExceeded` when an input exceeds one, and check each limit before any allocation that depends on it, because the input is untrusted, and an unchecked size lets one call allocate without bound (Section 5.9 of the architecture):

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Generators of a presentation | 256 | The `u8` index of a letter. A signature of size 64 gives at most 128 Conway generators, because $3h + k + l + 2b + 2c \leq 2 \cdot$ size |
| Relators of a presentation | 1024 | The Conway presentation of a signature of size 64 has at most 129 relators |
| Letters in a relator base | 4096 | The global relator of a Conway presentation has at most 64 letters |
| Generator name | 1 to 32 bytes, `[A-Za-z][A-Za-z0-9]*`, unique | Printable ASCII, so a caller can print a name without escaping it |
| Subgroup generators | 256 | A choice of this design |
| Letters in a subgroup generator | 4096 | The same bound as a relator base |
| Expanded relator letters, summed over the relators, checked by `enumerate` | 65536 | Above the expanded length of the Conway presentation of every signature whose domain `escher-domain` builds, whose corner and cone orders are at most 1000. Section 11.1 records the lengths for `*(1000)32` and `*(1000)22` |
| Cosets | `CosetLimit::MAX`, 1048576 | Section 6.3 |
| Table entries, limit times columns | $2^{24}$ | Bounds the table at 64 MiB and its relabelled copy at as much |

The limits are private constants, and unit tests inside the crate read them. Expanded lengths are summed in checked `u64` arithmetic. Limit times columns is at most $2^{24}$, so every index into the table fits `usize` on every target. Entries are `u32`, and the undefined marker `u32::MAX` lies above the largest coset number.

GRP-20. The memory of one call of `enumerate` MUST stay within the bound of this section, and the table MUST grow with its capacity capped at limit times columns, because a vector that doubles its capacity can allocate up to twice what the limit allows, and the bound is what a caller relies on when it chooses a limit.

For a limit $L$, $c$ columns, $E$ expanded relator letters, and a stack cap $S = 65536$, the memory of a call is at most

$$8 L c + 12 L + 8 S + 24 E$$

bytes beyond its inputs, plus 8 bytes for each entry by which the deduction stack exceeds $S$. The terms are the table and its relabelled copy, the representatives with the queue and the label map, the stack, and the stored relators with their conjugate lists. The table grows by `reserve_exact` to the smaller of twice its length and $L c$. At the maxima the bound is about 142 MiB, and at the default limit with the most columns, 512, about 66 MiB.

The stack can exceed $S$ by the entries that one coincidence or one lookahead pushes before the next check of the cap (Section 7.6), and this design has no bound on that excess smaller than the number of entries written in that pass. On the prototype, no stack came near $S$, and the measured peaks stayed within the bound with the term $8S$ (Section 11.1).

GRP-21. The crate MUST check every arithmetic operation on a size that comes from input, MUST contain no `unsafe` code, and MUST use no recursion, because an overflow gives a wrong table without an error, and the depth of a recursion over words or cosets is set by the input.

The time of a run has no bound from these limits. A coset limit bounds the cosets defined and not the scans per coset, and the sources give no computable bound on the work (Section 7.9). The residual risk is a caller that passes long relators with a high limit. The slowest input found within every limit, 2 generators with 16 random relators of 4096 letters at the default limit, took 20.3 s in a release build, as measured on a prototype outside the repository (Section 11.1). `escher-cli` cannot reach such an input, because the Conway presentation of a spherical signature of order $N$ expands to at most $2N + 11$ letters and the side presentations are short, and it does not enumerate a group whose order is above the limit (CLI-39 of Section 5.7 of `docs/design/cli.md`). A work budget in letter steps is the recorded alternative (item 1 of Section 14.3).

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library.
- Untrusted input and file paths: the values of the public types are the only input. The crate reads and writes no file and keeps no state between calls.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate builds nothing and prints nothing. The display of names and words is the caller's, and the names are printable ASCII by GRP-5.

A further residual risk is that a step that only code states is wrong in a way that the closing pass does not reveal. The closing pass makes every returned table closed, so such a fault can only make a run define more cosets or reach its limit, under the sourced statement of Section 7.7 and the unverified statements of `UNV-19`.

## 16. Operational Considerations

The crate declares no Cargo feature, reads no environment variable, and has no build script. It is pure Rust with no platform-specific code, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log and starts no thread. The default of its one option, the coset limit, is `CosetLimit::DEFAULT`, 16384.

Every failure is an `Error` with an `ErrorKind`:

| Failure | Kind | Recovery |
| --- | --- | --- |
| Empty relator base, exponent 0, bad or repeated name, or a relator letter outside the generators | `InvalidPresentation` | Correct the presentation |
| A subgroup generator with a letter outside the generators | `InvalidWord` | Correct the subgroup words |
| An input over a limit, or a coset limit out of range | `LimitExceeded` | Reduce the input below the limit of Section 15 |
| The run defined `limit` cosets and needed more | `CosetLimitReached` | Try a larger limit. The error says nothing about the index (GRP-17) |
| A signature with an infinite order | `UnsupportedSignature` | None. The design builds no presentation for it |

## 17. References

### 17.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 17.2. Informative References

The tags of the mathematical and software sources resolve to the reference sections of the research documents:

- [ACE] Section 15 of `docs/research/colourings.md`.
- [BROWN2011] Section 15 of `docs/research/colourings.md`.
- [CANNON-HAVAS1992] Section 15 of `docs/research/colourings.md`.
- [CBG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CDHW1973] Section 15 of `docs/research/colourings.md`.
- [CH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [GAP] Section 15 of `docs/research/colourings.md`.
- [HAVAS1991] Section 15 of `docs/research/colourings.md`.
- [HOLT2005] Section 15 of `docs/research/colourings.md`.
- [HOLT2013] Section 15 of `docs/research/colourings.md`.
- [HULPKE2010] Section 15 of `docs/research/colourings.md`.
- [MACBEATH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [SINGERMAN] Section 13.1 of `docs/research/orbifold-notation.md`.
- [SIMS1994] Section 15 of `docs/research/colourings.md`.
- [SYMPY-CT] Section 15 of `docs/research/colourings.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| GRP-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| GRP-2 | Operations on `Letter` and `Word` never panic and use no recursion | 4.1 |
| GRP-3 | The text of a word and of a relator has the form of Section 4.4 | 4.4 |
| GRP-4 | A relator has a non-empty base of at most 4096 letters and an exponent of at least 1 | 4.2 |
| GRP-5 | `Presentation::new` checks names, counts, and letters, and never reduces or reorders a relator | 4.3 |
| GRP-6 | `conway_presentation` builds steps 1 to 5 of the notation reference with the stated order and names | 5.1 |
| GRP-7 | A boundary without corners gives one reflection with `r^2` and `l^-1 r l r^-1` (`UNV-22`) | 5.2 |
| GRP-8 | A signature with an infinite order returns `UnsupportedSignature` | 5.3 |
| GRP-9 | A table uses right cosets, coset 0 as the subgroup, the column order generator then inverse, and a stored number of cosets | 6.1 |
| GRP-10 | `enumerate` follows the steps of Section 7.3 | 7.1 |
| GRP-11 | A popped entry is scanned with the conjugates selected by its letter | 7.4 |
| GRP-12 | Every entry written is pushed on the deduction stack | 7.3 |
| GRP-13 | A coincidence keeps the smaller number and is processed through a queue | 7.3 |
| GRP-14 | `enumerate` returns only after a closing pass finds every relator and subgroup generator closed | 7.7 |
| GRP-15 | The returned table is complete, compressed, and standard | 6.2 |
| GRP-16 | The limit counts every coset defined, coset 0 and dead cosets included, and reaching it returns `CosetLimitReached` | 6.3 |
| GRP-17 | No text states that reaching the limit shows an infinite index | 7.8 |
| GRP-18 | The table is a function of the inputs alone | 7.9 |
| GRP-19 | The input limits are enforced before any allocation that depends on them | 15 |
| GRP-20 | The memory of a call stays within the stated bound, with the table capacity capped at limit times columns | 15 |
| GRP-21 | Arithmetic on input sizes is checked, and the crate has no `unsafe` code and no recursion | 15 |
| GRP-22 | `Error` follows ERR-1, with the operation and the texts of Section 8.2 | 8.1 |
| GRP-23 | The dependencies are `escher-signature`, `thiserror`, and the dev-dependency `proptest` | 12 |
| GRP-24 | The worked examples and "Closing pass idle" stay while `UNV-19` is open | 10.1 |
| GRP-25 | The conjugate lists hold each distinct cyclic conjugate once | 7.2 |
| GRP-26 | A one-letter relator fixes its entries at coset 0 and at each new coset | 7.5 |
| GRP-27 | An overflowing stack is emptied before a lookahead that keeps its pushes, and the cap is a crate-private parameter | 7.6 |
| GRP-28 | Property tests take their configuration from one Miri-aware helper | 10.4 |
