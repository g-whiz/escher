# Group Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-23 |
| Applies to | `crates/escher-group` |

## Abstract

This document specifies the crate `escher-group`, which holds finitely presented groups, Todd-Coxeter coset enumeration with a bound on the number of cosets, and the low-index subgroup search that gives the colourings of a tiling. It defines letters, words, relators with an exponent, and presentations with named generators, the text form of each, and the presentation that Conway's rules read from an orbifold signature. It specifies the coset table that an enumeration returns, the Felsch strategy as numbered steps with the source of each step, the rule for relators of one letter, the closing pass that makes a returned table rest on sourced statements, the limit on cosets and what reaching it shows, and the standard form that makes equal subgroups give equal tables. For the search, it specifies the steps of a run, the first-in-class test, the check of every complete table, the expansion of each conjugacy class into its subgroups, the work budget and the limits on what a call stores, what a call returns when it stops at a limit, the valid colourings and the set of colourings that a call returns, the search of the classes in which given words fix no coset, the colouring ids, and the rule that turns the sides of a fold into a colour. It also states the error type, the limits on untrusted input with their memory bound and the residual risk on time, the layers of the tests with their worked examples, the dependencies, the unverified statements that the code relies on, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

The colourings of a tiling come from subgroups of finite index of its symmetry group, and every subgroup of finite index has a coset table (Sections 4.1 and 7.9 of the colouring reference). A program that lists colourings therefore needs the group as generators and relators, and a procedure that turns a presentation and a subgroup into a coset table. No usable Rust crate provides either (Section 14 of `docs/research/rust-ecosystem.md`). The procedure has no computable bound on its work, and some inputs make it define many more cosets than the index (Section 7.9), so an implementation that reads untrusted input needs limits that it states and enforces.

### 1.2. Purpose

`escher-group` is the one place where a presentation becomes a coset table. It builds the Conway presentation of a signature, accepts any other presentation within its limits, and returns either a complete coset table in a standard form or an error that claims nothing about the index. Section 3 of the architecture places the crate in the workspace with `escher-signature` as its only internal dependency. The enumeration of a group of known order is an independent check of a presentation.

The crate is also the one place where the colourings of Section 5.4 of the architecture are found, numbered, and evaluated. Its search lists the subgroups of finite index of a presentation up to a bound, gives each one a colouring id, and maps the side sequence of a fold to the colour of a tile. The search has no computable bound on its work either, so it runs within a budget of steps and reports how far it is complete.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-domain`, whose side presentation is a `Presentation` of this crate, of `escher-render`, which colours a sample with `colour_of`, and of `escher-cli`, which prints presentations, runs the enumeration, prints the colourings, and resolves a colouring id.

### 1.4. Scope

This document covers the public surface of the crate, words, relators, presentations and their text, the Conway presentation, the coset table and its standard form, the enumeration algorithm with its limit, the low-index search with its budget and limits, the colourings with their validity, their ids, and the colour of a sample, the error type, the limits on untrusted input, the tests, and the dependencies of the crate.

### 1.5. Non-Goals

These topics are owned elsewhere:

- The side presentation of a fundamental domain is a function of `escher-domain`, which builds a `Presentation` of this crate. DOM-32 to DOM-35 of Section 5.4 of `docs/design/domain.md` specify it, and `UNV-32` (Section 7 of the plan) records its unverified letter form. The side presentations of the wallpaper domains (DOM-52 of `docs/design/domain.md`) and of the spherical group domains need no code and no rule in this crate. The domain of `1` has the presentation with no generator and no relator (DOM-62 of `docs/design/domain.md`), which enumerates to index 1 and which the search accepts.
- The option `--presentation` of `escher info`, its lines, and the comparison of the enumerated order with the group order are specified by CLI-37 to CLI-39 and CLI-41 of Section 5.7 and by CLI-40 of Section 6.9 of `docs/design/cli.md`. They are proposed as `DEC-35` (Section 10.2 of the architecture).
- The fill of a tile by its colour, the side buffer of a sample, and the palette are specified by RND-36 to RND-40 of `docs/design/render.md` and by CLI-44 of `docs/design/cli.md`. The colouring lines of `escher info`, the options `--max-colours` and `--colouring`, and the text of a listing that stops at a limit are specified by CLI-42 to CLI-49 of `docs/design/cli.md`.
- Reidemeister-Schreier rewriting (Section 7.18 of the colouring reference), names of subgroups, and a marker for normal subgroups are not part of this crate.
- The search takes no subgroup argument, so it cannot list only the subgroups that contain given words. The tests filter complete tables instead (Section 10.1).
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
- low-index search: The search through a tree of partial coset tables for the subgroups whose index is at most a bound (Section 7.17 of the colouring reference).
- partial table: A coset table in which some entries are undefined. A node of the search holds one.
- run: One depth-first traversal of the search tree with a bound $n$ on the number of rows (Section 7.11).
- candidate: One value that the search tries for the first undefined entry of a node.
- contradiction: A scan of a partial table that completes with two different cosets. The search never processes it as a coincidence and abandons the candidate instead.
- class: A conjugacy class of subgroups of finite index.
- class table: The least standard table of a class in the order of GRP-37, which the first-in-class test keeps.
- first-in-class test: The test that a partial table can still extend to the least table of its class (Section 7.11).
- leaf check: The scan of every relator at every coset of a complete table before the search passes it on (GRP-33).
- base coset: The coset that a relabelling numbers 0.
- colouring: One subgroup of finite index, with its standard table, read as the colouring of the tiles by its cosets (Section 5.4 of the architecture).
- colour count: The index of the subgroup of a colouring.
- valid colouring: A colouring whose subgroup contains no generator of the presentation, tested on its table as GRP-42 states. The owner's criterion of `DEC-65` defines validity by adjacent tiles, and `UNV-35` records that the two agree.
- colouring set: The colourings that a call of the search returns, all of them, only the valid ones, or one valid colouring for each class, which `ColouringSet` selects (Sections 7.13 and 7.18).
- colouring id: The text `<colour count><letters>` that names one colouring (GRP-37).
- step: One read of a table entry by the search (GRP-29).
- search budget: The largest number of steps that one run, or one set of runs by index, may take.
- index limit: The largest colour count that a call lists.
- complete to: A call is complete to $m$ when it has found every colouring of every colour count from 1 to $m$.

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

pub struct IndexLimit { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash
impl IndexLimit {
    pub const DEFAULT: Self;
    pub const MAX: Self;
    pub fn new(index: u32) -> Result<Self>;
    #[must_use] pub fn get(self) -> u32;
}
impl Default for IndexLimit {}

pub struct SearchBudget { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash
impl SearchBudget {
    pub const DEFAULT: Self;
    pub const MAX: Self;
    pub fn new(steps: u64) -> Result<Self>;
    #[must_use] pub fn get(self) -> u64;
}
impl Default for SearchBudget {}

pub enum ColouringSet {
    All,
    Valid,
    ValidClasses,
}
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub fn count_colourings(presentation: &Presentation, set: ColouringSet, limit: IndexLimit, budget: SearchBudget) -> Result<ColouringCounts>;
pub fn colourings(presentation: &Presentation, set: ColouringSet, limit: IndexLimit, budget: SearchBudget) -> Result<Colourings>;
pub fn colourings_of_index(presentation: &Presentation, set: ColouringSet, colours: IndexLimit, budget: SearchBudget) -> Result<ColouringsOfIndex>;
pub fn free_classes_of_index(presentation: &Presentation, stabilizer: &[Word], colours: IndexLimit, budget: SearchBudget) -> Result<ColouringsOfIndex>;

pub struct ColouringCounts { /* private */ }
// derives Clone, Debug, PartialEq, Eq
impl ColouringCounts {
    #[must_use] pub fn requested(&self) -> u32;
    #[must_use] pub fn complete_to(&self) -> u32;
    #[must_use] pub fn colourings(&self, colours: u32) -> Option<u64>;
    #[must_use] pub fn classes(&self, colours: u32) -> Option<u64>;
}

pub struct Colourings { /* private */ }
// derives Clone, Debug, PartialEq, Eq
impl Colourings {
    #[must_use] pub fn requested(&self) -> u32;
    #[must_use] pub fn complete_to(&self) -> u32;
    #[must_use] pub fn list(&self) -> &[Colouring];
    #[must_use] pub fn count(&self, colours: u32) -> Option<u64>;
    #[must_use] pub fn find(&self, id: ColouringId) -> Option<&Colouring>;
}

pub struct ColouringsOfIndex { /* private */ }
// derives Clone, Debug, PartialEq, Eq
impl ColouringsOfIndex {
    #[must_use] pub fn colours(&self) -> u32;
    #[must_use] pub fn is_complete(&self) -> bool;
    #[must_use] pub fn list(&self) -> &[Colouring];
    #[must_use] pub fn find(&self, id: ColouringId) -> Option<&Colouring>;
}

pub struct Colouring { /* private */ }
// derives Clone, Debug, PartialEq, Eq, Hash
impl Colouring {
    #[must_use] pub fn id(&self) -> ColouringId;
    #[must_use] pub fn class(&self) -> ColouringId;
    #[must_use] pub fn colours(&self) -> u32;
    #[must_use] pub fn table(&self) -> &CosetTable;
    #[must_use] pub fn colour_of(&self, sides: &[u8]) -> Option<u32>;
    #[must_use] pub fn contained_generator(&self) -> Option<u8>;
}

pub struct ColouringId { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord
impl ColouringId {
    #[must_use] pub fn colours(self) -> u32;
    #[must_use] pub fn ordinal(self) -> u32;
}
impl core::fmt::Display for ColouringId {}
impl core::str::FromStr for ColouringId { type Err = Error; }

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
    InvalidColouringId,
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
| `IndexLimit` | A limit on the colour count of a search | $1 \leq$ value $\leq$ `MAX`. `DEFAULT` is 12 and `MAX` is 64 (Section 7.12) |
| `SearchBudget` | A limit on the steps of a search | $1 \leq$ value $\leq$ `MAX`. `DEFAULT` is $2^{28}$ and `MAX` is $2^{36}$ (Section 7.12) |
| `ColouringSet` | The colourings that a call returns: `All`, every subgroup of finite index, `Valid`, the valid colourings only, or `ValidClasses`, one valid colouring for each class that has one | `Valid` returns exactly the colourings of `All` whose `contained_generator()` is `None`, and `ValidClasses` exactly the colourings of `Valid` whose `class()` equals their `id()`, each in the same order, with ids numbered again from `a` (GRP-37, GRP-43, GRP-44) |
| `count_colourings` | The number of colourings and of classes of each colour count of the set up to the limit | Stores no table beyond the class that it expands (Section 7.13). Never an error for a reached budget (GRP-35) |
| `colourings` | Every colouring of the set of every colour count up to the limit | Ordered by colour count, then by id. Never an error for a reached limit of Section 7.12 (GRP-35) |
| `colourings_of_index` | Every colouring of the set of one colour count, in a `ColouringsOfIndex` | One run of that bound only (GRP-35) |
| `free_classes_of_index` | The class table of each class of one colour count in which no word of `stabilizer` fixes a coset, in a `ColouringsOfIndex` (GRP-45) | One run of that bound only. Ids ranked among the kept class tables, from `a` with no gap. Checks the words as GRP-46 states |
| `requested`, `complete_to` | The index limit of `count_colourings` or `colourings`, and how far the call is complete | $0 \leq$ `complete_to()` $\leq$ `requested()` |
| `ColouringCounts::colourings`, `classes` | The counts of one colour count | `None` for 0 and above `complete_to()`. `Some(0)` for a colour count without a colouring |
| `Colourings::count` | The number of colourings of one colour count in the list, which the search counted in the same call | `None` for 0 and above `complete_to()`. `Some(0)` for a colour count without a colouring. Equal to the number of colourings of that colour count in `list()` |
| `Colourings::find` | The colouring with an id | `None` when the list holds no colouring with that id |
| `ColouringsOfIndex` | The colourings of the one colour count `colours()` | Ordered by id. When `is_complete()` is false, the run stopped at a limit, the list is empty, and the result claims nothing. It never claims anything about another colour count |
| `ColouringsOfIndex::find` | The colouring with an id | `None` when the list holds no colouring with that id. That shows the id does not exist only when `is_complete()` is true and the id has `colours()` colours |
| `Colouring` | One subgroup of finite index | Its table is complete, compressed, and standard. Coset 0 is the subgroup |
| `Colouring::class` | The id of the colouring of its class that the relabelling from the least base coset kept (GRP-43) | With `All`, equal to `id()` of the class table, and the least id of the class when the first-in-class test is right (Section 7.13). With `Valid`, an id of the set. With `ValidClasses`, equal to `id()` (GRP-44). From `free_classes_of_index`, equal to `id()` (GRP-45) |
| `colour_of` | The colour of a tile from the sides of its fold | The rule of GRP-38. `None` when a side has no generator in the table |
| `contained_generator` | The least generator that lies in the subgroup | `None` exactly for a valid colouring (GRP-42) |
| `ColouringId` | A colouring id | Colour count from 1 to 64 and ordinal from 1 to 475254. `Ord` orders by colour count, then by ordinal |
| `Error` | Why an operation failed | Carries the operation and a private detail (Section 8) |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

Every public type is `Send`, `Sync`, and `'static`, except `WordDisplay` and `RelatorDisplay`, which borrow and are `Send` and `Sync`. A compile-time assertion test covers each (CONC-1 of the coding standard).

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, the compile-time assertion of the auto traits of Section 3.2, and, for the unit tests only, the module `common` of the integration tests, loaded through a `#[path]` attribute.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/word.rs`: `Letter` and `Word`.
- `src/presentation.rs`: `Relator`, `Presentation`, the name checks, and the display adapters.
- `src/conway.rs`: `conway_presentation`.
- `src/table.rs`: `CosetTable`, `CosetLimit`, the relabelling of Section 6.2, and the crate-private constructor of a table from its entries and the read of its entries, which the search uses.
- `src/enumerate.rs`: `enumerate`, the steps of Sections 7.1 to 7.9, the check of the expanded relator letters and the preparation of the relators and conjugate lists that the search shares, and the crate-private run statistics.
- `src/search.rs`: `IndexLimit`, `SearchBudget`, `ColouringSet`, `count_colourings`, `colourings`, `colourings_of_index`, `free_classes_of_index` with its filter of Section 7.19, the steps of Sections 7.10 to 7.14, the crate-private settings, which hold the budget, the stored limits, a switch of the first-in-class test (every node, complete tables only, or off), and the colouring set, with `All` as the default, and the crate-private search statistics and capacities.
- `src/colouring.rs`: `ColouringCounts`, `Colourings`, `ColouringsOfIndex`, `Colouring`, `ColouringId`, the ids of Section 7.15, `colour_of`, and the validity test of Section 7.18.

The integration tests form one binary, `tests/group/main.rs`, with the modules `common`, `examples`, `spherical`, `properties`, `limits`, and `search`. `main.rs` declares each module under `#[cfg(test)]`, as the integration tests of `escher-domain` do, so that the Clippy options for test code cover the panics of their helpers (TEST-10 of the coding standard). `common` holds the worked examples, the search fixtures, the random presentations, and the helpers that build names, words, presentations, and limits. `spherical` holds the spherical signatures, and `limits` holds Sims's presentation. `search` holds the published presentations, the side presentation of a rotation domain, and the helpers that read, relabel, and select the tables of a listing, which `properties` uses. The unit tests compile `common` through the `#[path]` module of `src/lib.rs`, where a helper that they do not use fails the build, so a helper that only the integration tests use belongs outside `common`. Each helper exists once, and no test module defines its own copy.

## 4. Words, Relators, and Presentations

### 4.1. Letters and Words

GRP-2. Every operation on `Letter` and `Word` MUST return without a panic for every input and MUST use no recursion, because words come from untrusted callers and can be long, and a panic or a deep recursion stops the caller (ERR-5 of the coding standard).

The generator index of a letter is a `u8`, so a letter names at most 256 generators, which equals the generator limit of Section 15. This makes the constructors infallible and keeps the column within `u16`.

`freely_reduced` pushes each letter on a stack and pops the top instead when the letter is the inverse of the top. `cyclically_reduced` reduces freely and then strips the first and last letters while they are inverse to each other. `inverse` reverses the word and inverts each letter.

### 4.2. Relators

GRP-4. `Relator::new` MUST return `InvalidPresentation` for an empty base or an exponent of 0, and `LimitExceeded` for a base of more than 4096 letters, because an empty relator has no text and an expanded length of 0, and an unbounded base makes every later step unbounded.

The checks run in this order, and the first failure is returned: an empty base, an exponent of 0, and the length of the base.

A relator stores its base and exponent and never expands them. A corner or cone order is up to `u32::MAX` (SIG-18 of the signature design), so the relator `g1^4000000000` of `(4000000000)(4000000000)` can be built and printed, and only `enumerate` refuses it, through the limit on expanded letters of Section 15.

`Relator::new` does not reduce its base. The relator $(x x^{-1})^3$ is accepted, and `enumerate` drops it as empty after cyclic reduction (step 1 of Section 7.3).

### 4.3. Presentations

GRP-5. `Presentation::new` MUST check the generator names, the number of generators and relators, and that every letter of every relator names a generator of the presentation, and MUST NOT reduce or reorder a relator, because a caller reads the relators back as it gave them and the text of Section 4.4 is what it gave.

A generator name is 1 to 32 bytes, matches `[A-Za-z][A-Za-z0-9]*`, and differs from every other name of the presentation. The names are therefore printable ASCII, and a caller can print them without escaping. A presentation has at most 256 generators and at most 1024 relators (Section 15). A presentation with no generator and no relator is valid, and it presents the trivial group.

`Presentation::new` keeps a `BTreeMap` from each name to its position. The map lets the check find an earlier equal name by lookup and read its position for the error.

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

The entry in row $c$ and column $x$ is $c^x$, the coset that $x$ takes $c$ to by multiplication from the right ([GAP] Section 47.6-1, Section 7.9 of the colouring reference). A word acts letter by letter from left to right, so `trace(c, w)` is $c^{w_1 w_2 \cdots}$. This column layout is that of [SYMPY-CT] and SymmHub, and of [GAP] apart from numbering from 1 (Section 7.21 of the colouring reference). Right cosets in tables are proposed as `DEC-36` (Section 10.2 of the architecture), and Section 5.10 of the architecture picks left cosets for colourings, so `colour_of` converts by the rule of GRP-38, under `UNV-17`.

The table is one flat `Vec<u32>` of `index() * 2 * generator_count()` entries in row-major order, beside the stored number of cosets.

### 6.2. Standard Form

GRP-15. `enumerate` MUST return a table that is complete, compressed, and standard, because a caller compares tables for equality to recognize equal subgroups, and a table numbered in the order of the run would make equal subgroups give unequal values.

Complete means that every entry of every row is defined ([SYMPY-CT] lines 121-128, Section 7.9 of the colouring reference). Compressed means that the dead rows are removed and the cosets renumbered without gaps ([BROWN2011] p. 5, Section 7.14 of the colouring reference). Standard is the lenlex standard of [GAP] Sections 47.7 and 47.7-1, which uses all columns (Section 7.14 of the colouring reference).

One relabelling does both, as the last step of the enumeration (step 11 of Section 7.3). It gives new number 0 to coset 0 and then reads the rows in the order of their new numbers. Within a row it reads the columns in order and gives the next new number to each live coset that has none. This is the procedure that [HULPKE2010] states on p. 78, read over all columns instead of the generator columns only (Section 7.14 of the colouring reference). It needs no swap, unlike SymPy's `standardize` ([SYMPY-CT] lines 752-797).

Lemma III.26 of [HULPKE2010] gives a bijection between subgroups of index $n$ and standardized tables, and it is stated for the standard that ignores the inverse columns. For the standard over all columns, Proposition 5.6 of [HOLT2005] states the bijection and justifies it only by the prose before it (p. 189), and [SIMS1994] states it in one sentence (p. 252). Neither gives a stand-alone proof (item 26 of Section 14.1 of the colouring reference). The property test "Invariance" of Section 10.1 checks the consequence that this design needs.

The invariance covers the returned table only. When two calls differ only in the order of the relators or of the subgroup generators and both return a table, the two tables are equal. Whether a call finishes within a limit is not invariant, because the number of cosets that a run defines can depend on the order, the spelling, and the length of the relators and of the subgroup generators. In a measurement of the crate on random presentations of the strategy of the tests at a limit of 2000, reversing the subgroup generators changed the number of cosets defined in about one in eleven of the cases that finished in both orders, and reversing the relators changed it in none. The documentation of `enumerate` states both parts.

### 6.3. Coset Limit

GRP-16. The enumeration MUST count every coset that it defines against the limit, including coset 0 and the cosets that a coincidence later eliminates, and MUST return `CosetLimitReached` when it needs a new coset after it has defined `limit` cosets, because a count of live cosets falls after each coincidence and so bounds neither the memory nor the time of a run.

SymPy compares its limit with the length of its table, which counts every row defined, dead rows included, by the research's own reading of the code. The DEFINE of [HOLT2005] counts the same way, because it aborts when the largest number used so far reaches the limit and always takes the next number (pp. 151, 153, and 167, Section 7.16 of the colouring reference). A table of index $n$ has $n$ rows, each defined once, so with coset 0 counted, a limit below $n$ always returns `CosetLimitReached`. `escher-cli` relies on this when it skips the enumeration of a group whose order is above the limit (CLI-39 of Section 5.7 of `docs/design/cli.md`).

The proposed default is 16384 and the proposed maximum 1048576 (`DEC-33`). The largest group order of a domain that `escher-domain` builds is 400, for `*(100)22`, so the default leaves a margin of 40 when the enumeration defines the group order and no more. The default is sized for corner orders up to 1000, at which `*(1000)22` has the group order 4000 and the margin is 4, and the owner kept it under the lower limit (`DEC-33` and `DEC-69`). In measurements of the crate and of a prototype outside the repository, every Conway and side presentation of a spherical domain defined exactly its group order of cosets, and a full index of $2^{20}$ at the maximum finished in under 2 s in a release build within the memory bound of Section 15. `CosetLimit::new` returns `LimitExceeded` for 0 or a value above `MAX`.

## 7. Coset Enumeration

### 7.1. Strategy

GRP-10. `enumerate` MUST follow the steps of Section 7.3, because each step is tied to its source or to the test that carries it, and a different strategy breaks that tie.

The proposed strategy is Felsch's (`DEC-33`), which defines the first open entry "by rows and within a row by columns", and this "guarantees that the image of each coset under each generator will be defined at some point" ([HULPKE2010] p. 77, Section 7.12 of the colouring reference). That guarantee is the hypothesis of Theorem III.24 of [HULPKE2010] and of Theorem 5.3 of [HOLT2013] with (R4), under which an enumeration of a subgroup of finite index terminates. It is also part (i) of Property 5 of [HOLT2005], whose Theorem 5.5 proves termination for a finite index when Properties 1 to 5 hold (p. 161), and part of condition (b) of [SIMS1994], whose Proposition 1.3 proves the same (p. 223). [HOLT2005] states that its Felsch procedure meets Property 5 (p. 166), while [SIMS1994] gives no proof that his FELSCH meets his conditions (pp. 232-235, Section 7.15 of the colouring reference). The theorems concern the books' procedures, not this crate's code. Holt states that the relator-based strategy does "not necessarily" meet (R4) ([HOLT2013] p. 18). [HOLT2005] states that its HLT procedure meets Property 5 because it defines every open entry of a row after its scans, and that implementations without that step failed to terminate on Ward's examples (p. 164). [GAP]'s built-in enumerator is Felsch's (Section 47.6-5).

The known cost of the choice is that a Felsch strategy scans more per definition, and that "some versions of Felsch-type procedures" define a number of cosets exponential in $n$ on Sims's presentation $\langle x, y \mid x^n = y, y^n = x \rangle$ ([HAVAS1991] p. 197, Section 7.20 of the colouring reference).

### 7.2. Data Structures

The enumeration keeps these structures, all as flat vectors with indexes and no references between elements:

- The table: one `Vec<u32>`, with entry `row * columns + column` and `u32::MAX` as the undefined marker. A row is appended at each definition, and no row is reused before the relabelling. Section 15 states how its capacity grows.
- The representatives: a `Vec<u32>` with `p[c] == c` for a live coset ([SYMPY-CT] lines 466-534, Section 7.13 of the colouring reference). The lookup is a loop with path compression. Section 15 states how its capacity grows.
- The deduction stack: a `Vec<(u32, u16)>` of a coset and a column, with a crate-private cap of 65536 entries (Section 7.6).
- The coincidence queue: a `Vec<u32>` with a read position, cleared after each coincidence and the merges that follow from it. A coset enters it only when a merge removes it, so it holds at most the cosets defined. Section 15 states how its capacity grows.
- The relators: each expanded, cyclically reduced relator and its inverse, stored twice in a row, so a cyclic conjugate is a pair of a relator and an offset with no test for the end. [HAVAS1991] reports the same device with three copies, as implementations expand "each relator out three times to avoid having to check for end of relator during deduction processing" (p. 195, Section 7.12 of the colouring reference).
- The conjugate lists: for each column $x$, the list $R_x$ of the distinct cyclic conjugates that start with $x$.

GRP-25. The conjugate lists MUST hold each distinct cyclic conjugate once, because a relator such as $(r_1 r_2)^{1000}$ has 2000 cyclic conjugates of which only 2 are distinct, and scanning duplicates multiplies the work by the exponent without changing a table. On a prototype outside the repository, the Conway presentation of `*(1000)22` gave the same table with and without duplicates and took about 700 times as long with them.

[HULPKE2010] forms "a list of all cyclic permutations of all relators and their inverses" (Note III.22, Section 7.12 of the colouring reference). SymPy's Felsch enumeration also forms the conjugates as a set before it splits them by first letter, by a reading of [SYMPY-CT] lines 1230-1240 that Section 7.12 of the colouring reference does not record.

Duplicates are removed across the whole presentation, between two relators and between a relator and an inverse, and not only within one relator. The enumeration keys each cyclically reduced word by its least rotation, keeps a set of these keys, and skips a word whose key is already in the set. For a word that it keeps, it lists the conjugates at the offsets below the shortest period of the word, which it finds with the prefix function. The lists are built in a fixed order, by relator in input order, then the relator before its inverse, then by offset, so the run is deterministic (GRP-18).

Columns of involutions are not merged. [HAVAS1991] notes that a table needs $(2g - i) \cdot n$ words when the inverse column of each of $i$ involutions is dropped (p. 192, Section 7.9 of the colouring reference). Uniform columns keep the layout of Section 6.1, and the relator $x^2$ makes the two columns of $x$ equal. The table of this design holds $2g \cdot n$ words.

### 7.3. Steps

`enumerate` runs these steps. The column "Code-only part" names what only this design or SymPy's code states at step level, and `UNV-19` covers it. The worked examples and the property test "Closing pass idle" of Section 10.1 check every code-only part (GRP-24). Every definition, in step 3 or step 5, first checks the limit as step 5 states:

| Step | What | Source | Code-only part |
| --- | --- | --- | --- |
| 1 | Check the limits of Section 15 and the letters of each subgroup generator, in the order of Section 8.2. Expand each relator, cyclically reduce it, and drop it when empty. Mark the one-letter relators. Form the conjugate lists of Section 7.2 | [HULPKE2010] Note III.22, pp. 75-76. Cyclic reduction first and the lists by first letter are [HOLT2005] pp. 165-166 (Section 7.12 of the colouring reference) | Distinct conjugates are GRP-25 |
| 2 | Start with coset 0 and every entry undefined. Apply the rule of Section 7.5 to coset 0 | [HULPKE2010] p. 72, [HOLT2005] p. 152, Section 7.9 of the colouring reference | The rule of Section 7.5, which neither [HOLT2005] nor [SIMS1994] treats (Section 7.9 of the colouring reference) |
| 3 | Process the deductions that step 2 pushed (step 6). Then scan and fill each subgroup generator at coset 0, in order. Trace the word forwards and backwards until each trace meets an undefined entry or the traces meet. Where more than one gap is left, define a new coset at the first undefined position of the forward trace, apply the rule of Section 7.5 to it, and process the deductions (step 6). Then replace the coset of each trace by its representative, because a coincidence may have removed it, and continue each trace from the position where it stopped. Repeat until the traces meet, which completes correctly or gives a coincidence (step 8), or until one gap is left, which is a deduction written both ways and pushed. After each subgroup generator, process the deductions (step 6). Because the traces resume, filling a word reads each of its letters once | Doing this first is [HAVAS1991] p. 193 and the COSETENUMERATIONC of [HOLT2005] p. 166 (Section 7.12 of the colouring reference). Scan-and-fill is the SCANANDFILL of [HOLT2005] p. 163 and the full two-sided trace of [SIMS1994] pp. 181-183 (Sections 7.10 and 7.11 of the colouring reference) | Processing the deductions after each definition, as step 5 does, and resuming the traces are this design's own. [HOLT2005] processes them once after all subgroup generators (p. 166) |
| 4 | Find the first undefined entry among the live cosets, by rows and within a row by columns. If there is none, go to step 10 | [HULPKE2010] p. 77, [HOLT2013] p. 17, [HOLT2005] p. 165, Section 7.12 of the colouring reference | The search keeps a cursor that only moves forward. When the cursor passes the last row, one full rescan of the live rows looks for a gap behind it, and the run statistics count each gap that the rescan finds. This avoids a scan from the first row at every definition |
| 5 | If `limit` cosets have been defined, return `CosetLimitReached` (GRP-16). Otherwise define a new coset $j$ with $i^x = j$ and $j^{x^{-1}} = i$, apply the rule of Section 7.5 to $j$, push $(i, x)$, and process the deductions | [HOLT2013] p. 15, and the DEFINE of [HOLT2005] p. 153, which aborts at the limit, counts every number used, and always takes the next one (pp. 151 and 167, Sections 7.10 and 7.16 of the colouring reference) | None |
| 6 | Process the deductions: while the stack is not empty, apply Section 7.6 when it holds the cap or more, and otherwise pop $(a, x)$ and scan as Section 7.4 states. Then return to the step that started the processing. After step 5 or step 10, that is step 4 | [HULPKE2010] Note III.22, [BROWN2011] p. 4, the PROCESSDEDUCTIONS of [HOLT2005] pp. 165-166, and the PROCESS_DEDUCTIONS of [SIMS1994] p. 234 (Section 7.12 of the colouring reference) | The order of the overflow (step 9) |
| 7 | Scan a word at a coset: trace the longest defined prefix forwards and the longest defined suffix backwards. The scan completes correctly, completes with two different cosets, which is a coincidence (step 8), leaves exactly one gap, which is a deduction written both ways and pushed, or yields nothing | [HULPKE2010] p. 75 in prose, the SCAN of [HOLT2005] pp. 153 and 155, and the TWO_SIDED_TRACE of [SIMS1994] pp. 181-183 (Section 7.10 of the colouring reference) | None |
| 8 | Process a coincidence of $\alpha$ and $\beta$: merge them, which finds their representatives and, when these differ, makes the smaller the representative of the larger and queues the larger. While the queue is not empty, remove its first coset $\gamma$. For each column $x$ with $\delta = \gamma^x$ defined, clear $\delta^{x^{-1}}$, and let $\mu$ and $\nu$ be the representatives of $\gamma$ and $\delta$. Then do the first of these that applies. If $\mu^x$ is defined, merge $\nu$ and $\mu^x$. Otherwise, if $\nu^{x^{-1}}$ is defined, merge $\mu$ and $\nu^{x^{-1}}$. Otherwise set $\mu^x = \nu$ and $\nu^{x^{-1}} = \mu$. Push $(\mu, x)$ for each entry written (GRP-12) | [HOLT2013] p. 17, [HULPKE2010] p. 76, the REP, MERGE, and COINCIDENCE of [HOLT2005] pp. 157-158, and the COINCIDENCE of [SIMS1994] pp. 190-191 (Section 7.13 of the colouring reference) | The pushes. [HOLT2005] pushes each entry that is defined or altered, in prose only (p. 165), and [SIMS1994] pushes the edges that COINCIDENCE adds when a flag asks for it (pp. 190-191). The design pushes $(\mu, x)$ for each entry that it writes, and SymPy pushes $(\delta, x^{-1})$ |
| 9 | Overflow of the stack: Section 7.6 | Lookahead is [BROWN2011] p. 4 and the LOOKAHEAD of [HOLT2005] p. 164, which its PROCESSDEDUCTIONS runs on a full stack (p. 166, Sections 7.11 and 7.12 of the colouring reference) | The order of emptying and lookahead, which is the reverse of [HOLT2005] p. 166 and of [SYMPY-CT] lines 673-704, and the cap of [SYMPY-CT] line 54 that this design does not use |
| 10 | Closing pass: Section 7.7. If it changes anything, go to step 4 | [HULPKE2010] p. 75, Theorem 5.2 of [HOLT2013], and Theorem 5.2 of [HOLT2005] pp. 154-155, which the handbook proves, Sections 7.9 and 7.15 of the colouring reference | The pass is this design's safeguard (GRP-14) |
| 11 | Relabel once, as Section 6.2 states, and return the table | [BROWN2011] p. 5, [HULPKE2010] p. 78, [GAP] Sections 47.7 and 47.7-1, the standard of [HOLT2005] p. 167 and of [SIMS1994] Section 3.8, Section 7.14 of the colouring reference | Relabelling in one pass without swaps is this design's own. The COMPRESS and STANDARDIZE of [HOLT2005] pp. 167-168, the STANDARDIZE of [SIMS1994] p. 195, and SymPy's `compress` and `standardize` ([SYMPY-CT] lines 752-832) use swaps or moves instead |

GRP-12. Every entry that a scan, a definition, a coincidence, or the rule of Section 7.5 writes MUST be pushed on the deduction stack, because an entry that is never scanned can leave a relator unchecked at a coset, and the closing pass then has to find the fault. On a prototype outside the repository, a run without the push of a scan's deduction made the closing pass change the table in about one random presentation in a hundred.

GRP-13. A coincidence MUST keep the smaller coset number and MUST process the further coincidences through a queue, because coset 0 is the subgroup and must survive every merge, and a recursive merge has no bound on its depth. Holt replaces the larger number by the smaller ([HOLT2013] p. 17), the MERGE of [HOLT2005] and of [SIMS1994] keeps the smaller ([HOLT2005] p. 157, [SIMS1994] p. 190), and Hulpke keeps "a list of coincidences to process" ([HULPKE2010] p. 76). The COINCIDENCE of [HOLT2005] also uses a queue (p. 158). Section 7.13 of the colouring reference records all four.

### 7.4. Scans of a Deduction

GRP-11. When it pops $(a, x)$, the enumeration MUST scan at $a$, if $a$ is live, every conjugate of $R_x$, and at $b = a^x$, if $b$ is defined and live, every conjugate of $R_{x^{-1}}$, selected by the letter $x$ of the popped entry, reading $b$ after the scans at $a$ and scanning at $b$ only when $a$ is still live, because a faithful rewrite of SymPy's variant, which scans every popped entry with the lists of its caller, never finished on the side presentation of any rotation double that a prototype outside the repository tried, and without a closing pass it returned wrong tables.

This is Hulpke's rule: "if the image of coset y under generator g is defined as z, we scan all relators in R^c_g starting at the coset y and all relators in R^c_{g^-1} starting at z" ([HULPKE2010] Note III.22, pp. 75-76, Section 7.12 of the colouring reference). SymPy's `coset_enumeration_c` passes two lists from its caller to its processing of deductions and scans each popped pair with them, whatever the column of the pair ([SYMPY-CT] lines 673-704, Section 7.12 of the colouring reference). Both books select the lists by the letter of the popped entry, as this design does ([HOLT2005] p. 166, [SIMS1994] p. 234), so SymPy departs from both. Neither book discusses SymPy's variant, so no source says whether it matters (item 23 of Section 14.1 of the colouring reference), and the prototype answers the question for this design. One of the presentations on which the variant never finished is the side presentation of `332`, $\langle s_0, s_1, s_2, s_3 \mid s_0 s_1, s_2 s_3, s_1^3, (s_2 s_0)^2, s_3^3 \rangle$, whose group has order 12.

The scans at $a$ stop when a scan kills $a$ by a coincidence, as SymPy's loop stops ([SYMPY-CT] lines 673-704, Section 7.12 of the colouring reference). Skipping the scan at $b$ then rests on GRP-12, because the coincidence pushes the entries that it writes. This is a reading of this design, which "Closing pass idle" checks.

### 7.5. One-Letter Relators

GRP-26. For each one-letter relator $x$, the enumeration MUST set $c^x = c$ and $c^{x^{-1}} = c$ at coset 0 and at each new coset $c$, and push both entries, because the global relator of a Conway presentation can be such a relator, as the next paragraphs show, and without this rule the enumeration defined about twice the group order on every Conway presentation with a boundary or a crosscap that a prototype outside the repository tried.

Cannon and Havas state the rule "each coset j must be mapped to itself by each of the defining relators" ([CANNON-HAVAS1992] Section 2.2, p. 3, Section 7.10 of the colouring reference). For a relator of one letter $x$, that rule fixes the entry of column $x$ in every row, and this step writes it when the row is created instead of waiting for a scan. When the cyclically reduced relator is $x^{-1}$, the letter is $x^{-1}$ and the same two entries result. The timing of the step is this design's own, under `UNV-19`. Neither [HOLT2005] nor [SIMS1994] treats a relator of one letter as a topic (Section 7.9 of the colouring reference). A one-letter relator also stays in the conjugate lists, so the scans of Section 7.4 check it like any other.

The global relator of every Conway presentation with exactly one Greek generator is a one-letter relator: `l1` for `*532`, `*`, and every other signature with one boundary and no other feature, `w1` for `x`, and `a1` for `o`. Without the rule, Felsch defines $c^{l_1}$ as a new coset at each coset $c$ and finds the coincidence afterwards.

### 7.6. Overflow of the Deduction Stack

GRP-27. When the deduction stack holds its cap or more at the top of step 6, the enumeration MUST first empty the stack and then run a lookahead that scans every relator at every live coset without a definition, pushing every entry that the lookahead writes, and the cap MUST be a crate-private parameter that a test can set to 1, because the overflow path never runs at the default cap on any input that a prototype outside the repository tried, and only a small cap exercises it.

The lookahead is Brown's: "we check all relators at all vertices (but without making new definitions) and take account of any deductions or coincidences that we can discover" ([BROWN2011] p. 4, Section 7.11 of the colouring reference). This design keeps the entries that the lookahead writes on the stack, so that their scans still run, which is how it reads "take account of any deductions". The PROCESSDEDUCTIONS of [HOLT2005] runs the lookahead first and then empties the stack ([HOLT2005] p. 166), and so does SymPy, which drops the entries that the lookahead pushed ([SYMPY-CT] lines 673-704, Section 7.12 of the colouring reference). The handbook gives no size for the stack (p. 165). The reverse order is a choice of this design (Section 14.2). On a prototype outside the repository, both orders gave correct tables at small caps.

The cap is checked only at the top of step 6, so one coincidence or one lookahead can push the stack past the cap. Step 3 runs step 6 after each definition and after each subgroup generator, so the cap is also checked during step 3. Section 15 states the bound on the memory of the stack that follows.

After the lookahead, step 6 checks the cap again before it pops. At a cap of 1, every entry that the lookahead pushes therefore starts another emptying and another lookahead, and no entry is ever popped, so a run at a cap of 1 is Felsch with the lookahead as its only scans. The tests compare such runs with runs at the default cap and require equal tables (Section 10.1).

### 7.7. Closing Pass

GRP-14. `enumerate` MUST return a table only after a closing pass has scanned every relator at every live coset and every subgroup generator at coset 0, without a definition, and found that each completes correctly, because the correctness of a returned table then rests on the sourced statement about closed tables and not on any step that only code states.

When "all tables are closed and no deductions are pending", "the columns of the coset table give permutation images for the group generators that are consistent with all relators", which gives a homomorphism to $S_n$ with $[G : S] = n$ equal to the number of rows ([HULPKE2010] p. 75, Section 7.9 of the colouring reference). Holt's Theorem 5.2 gives the action on the right cosets when the process stops with every entry defined ([HOLT2013] p. 16, Section 7.15 of the colouring reference). Theorem 5.2 of [HOLT2005] proves the same for a complete table that keeps its Properties 1 to 4 and scans correctly under every relator at every live coset and under every subgroup generator at coset 1 (pp. 154-155).

The notes state that the table keeps Holt's properties (R1) to (R3) through coincidences without a proof ([HOLT2013] pp. 16-17). These are Properties 2, 4, and 3 of [HOLT2005], where Property 2 is (R1): $\alpha^x = \beta$ exactly when $\beta^{x^{-1}} = \alpha$. Proposition 5.1 of [HOLT2005] proves that a deduction keeps Properties 1 to 4, and Proposition 5.4 proves that its procedure COINCIDENCE keeps them (pp. 153-154 and 158-159, Section 7.9 of the colouring reference). The propositions are about the handbook's procedures. Step 8 follows the steps of that COINCIDENCE, but no source proves that this crate's code keeps (R1), so that part stays in `UNV-19`.

The pass runs when step 4 finds no undefined entry. Each scan of a complete table either completes correctly or finds a coincidence. When it finds one, the pass processes it and the pushed deductions and returns to step 4. The crate-private run statistics count the cosets defined, the merges that joined two classes, the largest deduction stack, the changes that the pass finds, and the gaps that the rescan of step 4 finds.

On a prototype outside the repository and on the crate, the closing pass changed nothing in a correct implementation over every input tried, and on the prototype it found the faults of the variants of Sections 7.3 and 7.4 that the prototype introduced. The pass is therefore a safeguard against a defect of the code-only steps, and the property test "Closing pass idle" of Section 10.1 is what shows that those steps do their part.

Two branches guard against a failure of (R1) after a coincidence. Proposition 5.4 of [HOLT2005] proves that the handbook's COINCIDENCE keeps (R1), so the branches guard only against a defect of this crate's code, which is part of `UNV-19` (Section 13):

- The full rescan of step 4, which looks for a gap behind the cursor.
- The outcome "completes correctly" of a scan of step 7 whose backward trace reaches the position where its forward trace stopped. The same case in scan-and-fill (step 3) is reachable and normal, because there the forward trace stops where the backward trace ended.

A reading of the code finds both unreachable while (R1) holds whenever no coincidence is pending. Neither was reached when each was replaced by a panic and the whole test suite ran with 4000 cases for each property test. The design keeps both, because each turns a failure of (R1) into a correct table instead of a run that does not end. Without the rescan, a closing pass would find an incomplete scan, return to step 4, find no gap, and repeat without end. Without the outcome, the scan would report a coincidence of a coset with itself, which changes nothing, so the closing pass would also repeat without end. The run statistics count each gap that the rescan finds, and "Closing pass idle" asserts that it finds none.

### 7.8. What Reaching the Limit Shows

GRP-17. An error text, a documentation comment, or a document of this crate MUST NOT state that `CosetLimitReached` shows the index or the group to be infinite, because [HOLT2005] states that a run that runs out of space shows nothing about the index: "So, we learn nothing when this happens" (p. 149, item 19 of Section 14.1 of the colouring reference). [SIMS1994] states that such a run stops without a result (p. 224) and that an enumeration can end because memory is exhausted before the index is known (p. 247), but not that the run shows nothing about the index. ACE's manual shows a subgroup of finite index whose enumeration overflows at a small limit and succeeds at a larger one ([ACE] Appendix B.4, read through a summarizing tool, Section 7.15 of the colouring reference).

The documentation of `CosetLimitReached` states that the run defined the limit and needed more, and that a larger limit may or may not finish. Holt states that if the index is infinite the procedure "will not halt", and that for a finite index "there is no upper bound on the time taken" ([HOLT2013] p. 15, Section 7.15 of the colouring reference). A limit test of Section 10.1 shows that Sims's presentation, a finite group, reaches a limit above its order.

### 7.9. Determinism and Cost

GRP-18. The returned table MUST be a function of the presentation, the subgroup words, and the limit alone, because callers compare tables across runs, and a table that depends on a hash seed, a thread schedule, or a clock cannot be compared.

The enumeration iterates no hash map, uses no randomness, and starts no thread. The standard form of Section 6.2 also makes a returned table independent of the order of the relators and of the subgroup generators, which the test "Invariance" checks. That independence relies on the bijection between subgroups and lenlex standard tables, which [HOLT2005] and [SIMS1994] state without a stand-alone proof, and it does not extend to whether a call finishes within its limit (Section 6.2).

The sources give these two facts about cost, and no other (Section 7.2 gives the size of a table):

- For a given procedure "there is no computable bound, in terms of length of input and a hypothetical index, to the number of cosets which need to be defined" ([CANNON-HAVAS1992] Section 2.2, p. 3, Section 7.15 of the colouring reference).
- An enumeration "may take many more intermediate cosets than the actual index" ([GAP] Section 47.6-6).

### 7.10. Low-Index Search

GRP-30. `count_colourings`, `colourings`, and `colourings_of_index` MUST find one class table for each class by the runs of Sections 7.11 and 7.14, and MUST then expand each class table into the standard tables of all the subgroups of its class as Section 7.13 states, keeping only those of the requested set (GRP-43 and GRP-44), because the owner decided that every subgroup of finite index is its own colouring (`DEC-37`), and searching by class keeps the search tree small through the first-in-class test.

The search runs through a tree of partial coset tables, as Sims's method does, "a backtrack search over incomplete coset tables rather than over forced coincidences" ([HOLT2005] p. 189, Section 7.17 of the colouring reference). It follows the LOWINDEXSUBGROUPS, DESCENDANTSUBGROUPS, TRYDESCENDANT, and FIRSTINCLASS of [HOLT2005] pp. 191-194, whose FIRSTINCLASS is the FIRST_CONJ of [SIMS1994] pp. 208-209. It departs from them in five ways, each a code-only part under `UNV-19`:

1. Every entry that a scan deduces is pushed on the deduction stack (GRP-31), as the JOIN of [SIMS1994] pushes the entry that it writes (pp. 177 and 234). SymPy's `scan_check` writes a deduction without pushing it (item 23 of Section 14.1 of the colouring reference).
2. Every relator is used for deductions, with no split into short and long relators, as in Sims's Example 6.1 and in SymmHub. Sims states that "The output from LOW_INDEX depends only on V1 U V2. However, the running time will depend on the division" ([SIMS1994] p. 253).
3. A leaf check scans every relator at every coset of a complete table (GRP-33). It is lines 2 to 5 of DESCENDANTSUBGROUPS with every relator in place of the long ones ([HOLT2005] p. 192).
4. An undo trail replaces the copy of the table at each candidate, with an explicit stack of frames and no recursion (GRP-21). SymmHub undoes its definitions through a trail (Section 7.19 of the colouring reference).
5. The search takes no subgroup argument (Section 1.5).

This variant reuses the table layout of Section 6.1 and the conjugate lists of Section 7.2. It needs no coincidence processing, because the search never eliminates a coset: "the live cosets of a node are always $1, \dots, n_C$" ([HOLT2005] p. 190, Section 7.17 of the colouring reference). The setting of [SIMS1994] needs a rewriting system and one column for each involution, which GRP-9 does not use (Section 7.21 of the colouring reference).

A prototype of the search outside the repository, which used the side presentations of `escher-domain` and the fold of `escher-render`, found on the ten signatures that it rendered that each element of the group maps the picture of a subgroup onto itself, that two different subgroups always gave different pictures, and that no sampled element of the group related the pictures of two conjugate subgroups. These findings are the prototype's and not a source's. How conjugate subgroups relate to colourings is open in the colouring reference (Section 3.4 and items 1 to 3 of Section 14.1), and the design records it under `UNV-17`. The owner decided on one colouring for each subgroup in the search, and on one valid colouring for each class on the command line (`DEC-37`).

### 7.11. Steps of a Run

A run of bound $n$ runs these steps. The column "Code-only part" names what only this design, SymPy, or SymmHub states at step level, and `UNV-19` covers it:

| Step | What | Source | Code-only part |
| --- | --- | --- | --- |
| 1 | Check the limits of Section 15 as `enumerate` does in its step 1, expand and cyclically reduce each relator, drop it when empty, and form the conjugate lists of Section 7.2 | Step 1 of Section 7.3 | The distinct conjugates of GRP-25. The rule of Section 7.5 is not applied, because a candidate that breaks a one-letter relator fails at the scan of that relator |
| 2 | Start with one row, coset 0, with every entry undefined, an empty trail, and a stack of frames that holds the root | [HOLT2005] pp. 190-191, [SIMS1994] pp. 253-254, and [SYMPY-FPG] line 774 (Section 7.17 of the colouring reference) | The trail and the frames, which replace the copies of the sources |
| 3 | At a node, find the first undefined entry $(c, x)$, by rows and within a row by columns, from the slot at which its parent branched | [HOLT2005] p. 192, [SIMS1994] pp. 253-254 | Starting at the slot of the parent, before which every entry is defined |
| 4 | If no entry is undefined, the table is complete with $r$ rows. Keep it only if the run keeps tables of $r$ rows (Section 7.14). Run the leaf check of GRP-33, relabel the table as Section 6.2 states, and pass it to the expansion of Section 7.13 | The output of a complete table, [HOLT2005] p. 192, lines 2 to 5, and [SIMS1994] p. 253 | The leaf check with every relator, and the relabelling. In test builds only, the statistics count each relabelling that changes the table |
| 5 | Otherwise try each candidate $d$ in this order: each existing coset in increasing order whose entry $d^{x^{-1}}$ is undefined, $d = c$ included, then a new coset $r$ when the table has $r < n$ rows. For a candidate, note the length of the trail and the number of rows, write $c^x = d$ and $d^{x^{-1}} = c$, push $(c, x)$, and process the deductions as GRP-31 states. On a contradiction, or when the first-in-class test of GRP-32 returns false, undo to the noted length and number of rows. Otherwise descend to the new node at step 3 | [HOLT2005] p. 192, [SIMS1994] p. 254, and [HULPKE2010] p. 79 (Section 7.17 of the colouring reference) | The undo trail, and the processing of GRP-31 |
| 6 | When a node has no candidate left, return to its parent and try the next candidate of the parent. The run ends when the root has none left, or when it reaches a limit of Section 7.12 | [HOLT2005] pp. 191-192 | The stack of frames |

A frame holds the slot of its branch, the next candidate, the length of the trail, and the number of rows. Undoing sets each slot of the trail above the noted length back to undefined and restores the number of rows.

GRP-31. To process the deductions, the run MUST pop $(a, x)$ and scan every conjugate of $R_x$ at $a$ and, when $b = a^x$ is defined, every conjugate of $R_{x^{-1}}$ at $b$, as GRP-11 selects them, MUST scan every relator of the presentation, MUST write and push both entries of each deduction that a scan finds, and MUST abandon the candidate at the first contradiction, because a run that splits off the long relators or that does not push its deductions returns wrong tables without the leaf check, and on a prototype outside the repository, a run of `*732` or `*(1000)32` to 20 colours that split at 5 letters took thousands of times the steps of a run that used every relator.

A scan here is the scan of step 7 of Section 7.3 without a definition and without coincidence processing. It completes correctly, completes with two different cosets, which is a contradiction, leaves exactly one gap, which is a deduction, or yields nothing. The handbook's SCANCHECK "returns false (rather than calling COINCIDENCE) if the scan completes incorrectly" ([HOLT2005] p. 191), and Sims abandons the candidate on any coincidence ([SIMS1994] p. 254). That the search pushes the deductions of the relator traces is the research's inference from Sims's reference to the Felsch strategy, which no source states (item 23 of Section 14.1 of the colouring reference). On a prototype outside the repository, a run without the pushes kept its output right only through the leaf check, at almost three times the steps, and without the check gave wrong output on about a quarter of the presentations that it tried. In the crate, the tests catch a run without the pushes, because the leaf check then discards tables (Section 10.3).

GRP-32. The first-in-class test MUST, for each base coset $\alpha$ from 1 upward, relabel the cosets by first appearance with $\alpha$ as coset 0, reading the rows in their new order and each row by columns, and compare each relabelled entry with the entry of the original table at the same position. When either entry is undefined, or the original coset of the relabelled row is not yet known, it MUST go to the next $\alpha$. When the relabelled entry is smaller, it MUST return false, and when it is larger, it MUST go to the next $\alpha$. It returns true when every $\alpha$ is done. It MUST NOT compare an undefined entry as 0, because a prototype outside the repository that read the rule that way lost classes on most of the presentations that it tried and returned false on the second `first_in_class` doctest of SymPy, where SymPy expects true.

This is the rule of FIRSTINCLASS ([HOLT2005] p. 194), of FIRST_CONJ ([SIMS1994] pp. 208-209), of SymPy's `first_in_class`, and of SymmHub's `isCanonical`, all of which stop at an undefined entry (Sections 7.17 and 7.19 of the colouring reference). The rule of `low_index` compares an empty slot as 0 ([LOWINDEX] Section 3.4). The handbook states that the test "will be conclusive if D is complete" ([HOLT2005] p. 192). That pruning a partial table loses no class is stated and argued by both books and proved by neither (item 22 of Section 14.1 of the colouring reference), so it is part of `UNV-19`. In the crate, pruning partial tables gave the same classes and colourings as testing complete tables only on every random presentation tried, and the test on complete tables only took more nodes and steps.

GRP-33. Before a run passes on a complete table, it MUST scan every relator at every coset of the table and MUST discard the table, counting it in the crate-private statistics, unless every scan completes correctly, because a table that is passed on then rests on the sourced statement about closed tables ([HULPKE2010] p. 75 and Theorem 5.2 of [HOLT2005], Section 7.7), as GRP-14 makes a table of `enumerate` rest on it, and not on the code-only steps of the search.

The handbook states that the leaf tables already satisfy the short relators, with no argument beyond the use of the deduction stack ([HOLT2005] p. 191, Section 7.17 of the colouring reference). In the crate, the leaf check discarded no table on any input measured, and on a prototype outside the repository it took about 5 percent of the steps. No test detects its removal (Section 10.3). The check stays as the safety net that the closing pass is for `enumerate`.

### 7.12. Limits of a Call

GRP-29. The search MUST count one step for each read of a table entry, with no read left out: in the search for the first undefined entry of step 3 of Section 7.11, in the test of a candidate's entry $d^{x^{-1}}$ of step 5, in the read of $b = a^x$ before the scans at $b$, in a scan, in the first-in-class test, in the leaf check, in the relabellings of Section 7.13, and in the validity test of GRP-42. It MUST also count two steps for each entry that the expansion compares when it removes repeated relabellings. It MUST compare the count with the budget at each read and stop the run at the first read beyond the budget, because a count of reads bounds the time of every call and gives the same result on every machine, which a clock does not. Two counts are the exception and are compared with the budget only once they are complete: the expansion adds the steps of a comparison after each compared table, and the validity test adds its reads, at most one for each generator, after each tested relabelling. For a class table of $k$ rows and $c$ columns, a run therefore stops fewer than $2kc$ steps past the budget in a comparison and fewer than $c/2$ steps past it in a validity test.

The values are these:

| Limit | Default | Maximum | Reason |
| --- | --- | --- | --- |
| `IndexLimit` | 12 | 64 | The default is the owner's (Section 10.1 of the architecture). The maximum keeps an id within 2 digits and the table of a run within $64 \cdot 512$ entries |
| `SearchBudget` | $2^{28}$ steps | $2^{36}$ steps | In a release build of the crate, $2^{28}$ steps took 0.25 to 0.56 s on the side presentations measured, and a call takes at most twice the budget (GRP-35). $2^{36}$ steps are 256 times the default |
| Colourings stored by one call | 65536 | Not configurable | GRP-36 |
| Table entries stored by one call | $2^{22}$ | Not configurable | GRP-36, which bounds the stored tables at 16 MiB |

`IndexLimit::new` and `SearchBudget::new` return `LimitExceeded` for 0 or a value above `MAX`. A caller that resolves one id passes its colour count as the `IndexLimit` of `colourings_of_index`.

GRP-36. `colourings` and `colourings_of_index` MUST stop a run when storing the next colouring would make the call hold more than 65536 colourings or more than $2^{22}$ table entries, and MUST treat that stop as a stop at the budget (GRP-35), because the number of subgroups grows fast with the index and the class sizes multiply it, and an unbounded list exhausts memory before the budget ends the run. `count_colourings` stores no colouring, so only the budget stops it.

Because the stored limits stop only `colourings` and `colourings_of_index`, `colourings` can be complete to fewer colour counts than `count_colourings` on the same input. For example, with `ColouringSet::All` and the default limit and budget, the side presentation of `*664` lists to 11 and counts to 12. The stored limits count only the colourings that the call stores, so with `ColouringSet::Valid` or `ColouringSet::ValidClasses` the colourings that the filter drops count toward neither.

A caller cannot tell from the result whether the budget or a stored limit stopped a call, because both give the same result. Every text that reports such a stop, in the crate and in its callers, says that the search stopped at a limit, which covers both (CLI-47 of `docs/design/cli.md`).

### 7.13. Subgroups of a Class

GRP-34. For each class table $T$ of $k$ rows, the call MUST relabel $T$ from each base coset $\alpha$ from 0 to $k - 1$ by the relabelling of Section 6.2 started at $\alpha$, keep each distinct result once as a colouring, and give each of them the class of $T$, because the conjugates of a subgroup are the stabilizers of the cosets, and their tables arise by relabelling from another base point.

The handbook states that "the conjugates of H are just the stabilizers G_alpha of the points alpha in Omega" ([HOLT2005] p. 190), and that the tables $C_\alpha$ from the base points are the standardized tables of the conjugates of $H$ ([HOLT2005] p. 193). Hulpke states the same relabelling ([HULPKE2010] pp. 80-81, Section 7.17 of the colouring reference). That two distinct standard tables are two distinct subgroups is Proposition 5.6 of [HOLT2005], which `UNV-19` covers. No mathematical source states a rule for the number of subgroups in a class, and SymmHub's code computes it as the number of distinct standardizations over all base points (Section 7.17 of the colouring reference), as this design does.

The expansion removes repeated relabellings by comparing each new relabelling with the distinct ones kept so far, entry by entry with an exit at the first difference, and counts the reads in steps (GRP-29). Its scratch buffer holds only distinct tables, at most $k$. Each distinct table is represented by its first base coset, so with `All` the relabelling from base 0, which is $T$ itself, is always kept. With `Valid` and `ValidClasses`, $T$ is dropped when it is not valid, and GRP-43 states which colouring then names the class. A sort of the $k$ relabellings was 2 to 5 times slower for each counted step on classes with large normalizers, and its comparisons were not counted. `count_colourings` counts the distinct tables and discards them. The other two functions store them.

$T$ is the least of its results, because it passed the first-in-class test as a complete table, and the test "Least in class" checks it. With `All`, `Colouring::class` is the id of $T$, the table that the search kept for the class, and not the least id of the class computed after the expansion. The two are equal when the first-in-class test is right. Computing the least id directly would hide a defect of that test, such as swapped comparisons, which would then change no output (Section 10.3).

GRP-43. With `ColouringSet::Valid`, the expansion of GRP-34 MUST test each relabelling by GRP-42 and MUST drop one that is not valid before it stores or counts it, and the runs of Sections 7.11 and 7.14, the first-in-class test, and the leaf check MUST stay as they are for `ColouringSet::All`, because validity belongs to one subgroup and not to its class: a class table that is not valid can have valid conjugates, so a filter on the class tables would lose valid colourings, and a filter in the caller would let the colourings that it discards count toward the stored limits of GRP-36.

A relabelling and a repeat of it are the same table, so the test can run before or after the comparison of GRP-34. The filter MUST also meet these requirements:

- A class with no valid subgroup gives no colouring, adds nothing to `ColouringCounts::classes`, and takes no id.
- The class of a kept colouring is the id of the kept colouring of its class with the least base coset. With `All`, that base coset is 0 and the colouring is $T$. With `Valid`, $T$ may be dropped, and the class then names a kept colouring, never a table that the call does not return.
- A check that compares counts, in the crate or in its tests, takes both sides from the same set. `Colourings::count` equals the number of colourings of its colour count in `list()` of the same call, and the counts of the three functions are compared only between calls with the same `ColouringSet`.

GRP-44. With `ColouringSet::ValidClasses`, the expansion MUST filter as GRP-43 states and MUST keep, for each class that has at least one valid subgroup, exactly one colouring, the valid relabelling with the least base coset, because the owner decided that the command line lists one valid colouring for each class (`DEC-37`). That colouring is the one that names the class under `Valid`. The expansion MAY stop at the first valid relabelling of a class, because the later relabellings change neither the kept colouring nor its class.

Every colouring of `ValidClasses` is the one kept colouring of its class, so `Colouring::class` equals its `id()`, and `ColouringCounts::classes` equals `ColouringCounts::colourings` for each colour count.

### 7.14. One Run, Then Runs by Index

GRP-35. `count_colourings` and `colourings` with the limit $N$ MUST first run once with the bound $N$, keeping every complete table. When that run ends, the call is complete to $N$. When that run stops at a limit of Section 7.12, the call MUST discard what the run found and run the bounds $k = 1, 2, \ldots, N$ in turn, keeping from run $k$ only the tables of exactly $k$ rows, on one fresh budget of the same size that these runs share. When run $k$ stops at a limit, the call is complete to $k - 1$, and it MUST return every colouring of the colour counts 1 to $k - 1$ and nothing of a colour count of $k$ or above. A depth-first run that stops early cannot tell which colour counts it finished, while a run of bound $k$ that ends has found every table of $k$ rows.

`colourings_of_index` with the colour count $k$ runs the bound $k$ once and keeps only the tables of exactly $k$ rows. It runs no other bound, because a caller that resolves one id needs only that colour count, and the ids of a colour count are ranked among its colourings in the set (GRP-37). When the run ends, `is_complete()` of the result is true. When it stops at a limit, `is_complete()` is false and the list is empty.

An error text, a documentation comment, or a document of this crate MUST NOT state anything about the colourings of a colour count above `complete_to()`, or about any colouring when `is_complete()` is false, because a stopped run shows nothing about them, as GRP-17 states for an enumeration that reaches its limit. A `ColouringsOfIndex` states nothing about any colour count other than its own. A call that stops at a limit returns its result and not an error.

On the side presentations measured in the crate, the runs by index cost about 2 to 23 times the steps of one run at the limits 12 and 24, so a call pays for them only when the one run cannot end. Both phases count steps, so a call takes at most twice the budget, plus the overshoot of GRP-29 in each phase and the uncounted sort of Section 15.

With the default limit and budget, the search on the side presentation of a triangle group whose corner orders are all near 100, such as `*(100)(100)(100)`, stops at the budget before the default index limit of 12. Each scan of a long power relator reads up to 200 entries. A split of the relators helps little there. Scanning a power relator $w^e$ by the cycle of $w$ at the coset, with at most $N$ repetitions and a test that the length of the cycle divides $e$, would read at most $2N$ entries for such a relator. That idea is unmeasured and is not part of this design (item 6 of Section 14.3).

### 7.15. Colouring Ids

GRP-37. The colourings of one colour count of the set MUST be ordered by the lexicographic order of their standard tables, compared entry by entry in row-major order over all columns, and the id of a colouring MUST be the colour count in decimal, from 1 to 64 without a leading zero, followed by the rank of the colouring in that order among the colourings of that colour count in the set, from 1, written in lower-case bijective base 26 with 1 to 4 letters. An id MUST be assigned only within a colour count that the call is complete to. This makes the id a function of the presentation and the set alone, independent of the limit, the budget, the order in which a run finds the tables, and the thread schedule of the caller.

With `All`, the ranks run over all the subgroups of the colour count, every conjugate included, and not over the classes (`DEC-37`). With `Valid`, they run over the valid colourings only, with no gap, as the owner decided (`DEC-71`), and with `ValidClasses` over the one kept colouring of each class (GRP-44). The three sets keep the same order, so the colourings of a smaller set take the ids `a`, `b`, and so on in the order of their ids under `All`. An id therefore names a colouring of one set, and the same text can name another colouring, or none, in another set. The command line uses `ValidClasses` (Section 5.8 of `docs/design/cli.md`), and the tests use `All` where a source counts every subgroup (Section 10.1). Proposition 8.7 of [SIMS1994] proves that, for the lenlex ordering, one automaton precedes another exactly when its transition matrix comes first in the lexicographic order of the entries read row by row (p. 136, Section 7.14 of the colouring reference), and the handbook orders its tables in the same way ([HOLT2005] p. 193). The generator order of the columns is that of the presentation, which for a side presentation is `s0` upward (DOM-32 of `docs/design/domain.md`).

Bijective base 26 writes the ranks 1 to 26 as `a` to `z`, 27 as `aa`, 702 as `zz`, 703 as `aaa`, and 475254 as `zzzz`, the largest rank that 4 letters write. GRP-36 bounds a stored list at 65536 colourings, so every rank that a call assigns has 4 letters or fewer. The colour count 1 has exactly one table, so its one colouring is `1a` under `All`. That colouring is valid only for a presentation with no generator (Section 7.18).

`ColouringId` writes and reads this text, which this grammar in the notation of [RFC5234] defines:

```abnf
colouring-id = colour-count 1*4letter
colour-count = %x31-39 [DIGIT]     ; 1 to 99, then checked to be at most 64
letter       = %x61-7A             ; a to z
```

`FromStr` accepts only this text with a colour count from 1 to 64, and returns `InvalidColouringId` for anything else, such as `0a`, `03a`, `65a`, `3A`, `3`, `a`, `3aaaaa`, or `3a ` with a trailing space. It does not check that a colouring with the id exists, which only a search can tell. `Display` writes the same text, so a parsed id prints as it was read.

The id depends on the order of the generators, and so for a side presentation on the numbering of the sides of the domain. On a prototype outside the repository, with ids ranked among the class tables, swapping `s0` and `s1` renamed ids, while reordering or rewriting the relators, adding a redundant relator, changing the limit or the budget, and changing the parameter `aspect` of `*2222` did not. A change of `escher-domain` that renumbers the sides of a domain would therefore rename its ids without an error, which is why the side numbering of a built domain is part of the contract of `escher-domain` (DOM-37 of `docs/design/domain.md`). That the id does not depend on how the relators are written rests on Proposition 5.6 of [HOLT2005], under `UNV-19`.

The parity table of a reflection domain, whose row 0 is all ones, has the last id of the colour count 2 under `All`. In the crate, under the ids of GRP-37 with `All`, it is:

| Id | Signatures |
| --- | --- |
| `2a` | `*732`, `*532`, `*333`, and `*332` |
| `2c` | `*632`, `*832`, `*542`, `*432`, `*66`, `*22`, and `*(100)32` |
| `2g` | `*642`, `*442`, `*664`, and `*(100)22` |
| `2o` | `*2222` |

The parity table moves every generator off coset 0, so it is valid (GRP-42), and under `Valid` and `ValidClasses` it is `2a` of every reflection domain while `UNV-36` holds (Section 7.18).

### 7.16. Colour of a Sample

GRP-38. `Colouring::colour_of` MUST start at coset 0 of the colouring's own table, MUST then apply, for each side index of the slice in reverse order, the column of the inverse of the generator of that side, and MUST return the coset reached, or `None` when a side index names no generator of the table, because the fold finds the sides of a tile in the order in which their generators multiply, and the first of them acts last on the cosets. It is the one implementation of the rule in the workspace, and `escher-render` calls it (RND-37 of `docs/design/render.md`).

For a fold that applies the sides $i_1, \ldots, i_n$ in that order, the sample lies in the tile $gF$ with $g = s_{i_1} s_{i_2} \cdots s_{i_n}$, where $s_i$ is the generator of side $i$, which stands for $P_i^{-1}$ (Section 5.3 of the architecture and DOM-32 of `docs/design/domain.md`). `colour_of` returns the number of the right coset $S g^{-1}$ of the subgroup $S$ of the table, which the trace of $g^{-1} = s_{i_n}^{-1} \cdots s_{i_1}^{-1}$ from coset 0 reaches under the right action of GRP-9. The empty slice gives coset 0, the colour of the fundamental domain itself.

No source states this rule. That the product of the generators of the fold's sides, in fold order, maps the domain onto the tile of the sample, and that the number of $S g^{-1}$ names the left coset $gS$ by which Section 5.4 of the architecture colours the tile $gF$, are both part of `UNV-17`. Lemma 2.1 of [FRETTLOH2008] proves a coset colouring perfect only for the triangle tiling $T_{p,q}$ under $G_{p,q}$ (Sections 4.1 and 4.2 of the colouring reference). The only precedent is the comment of SymmHub that its fold needs "the sequence of _inverse_ transformations applied in _opposite_ order", which is the code's own claim (Section 8.4 of the colouring reference).

The prototype of Section 7.10 checked the rule on samples, which is no proof. The product of the generators in fold order mapped the folded point back onto the sample for 300 samples of each of 12 signatures, and the product in the reverse order did not. The rule of GRP-38 gave a perfect colouring on every class tried, with the exact permutation that GRP-41 asserts: the colour of $P_i^{-1} x$ is the image of the colour of $x$ under the inverse of generator $i$. Tracing the word forwards was not perfect on exactly the classes that are not normal. Tracing in reverse without the inverse gave a perfect colouring on every class, which was the mirror image of the right one, reflected in a wall of the starred domain. It differed only on rotation domains, because the generator of a mirror is its own inverse.

Because the first letter of $g$ acts last, a caller cannot carry one coset forward through the fold. It records the side indices and calls `colour_of` once for each sample (RND-38 of `docs/design/render.md`). The alternative is a permutation of $k$ entries carried through each fold step.

### 7.17. Determinism of the Search

GRP-39. The result of `count_colourings`, `colourings`, and `colourings_of_index` MUST be a function of the presentation, the set, the limit, and the budget alone, and the colourings of each colour count up to `complete_to()` MUST NOT depend on the limit or the budget, because `escher info` lists the ids that `escher render` later resolves in another call with another limit (CLI-43 of `docs/design/cli.md`).

The search iterates no hash map, uses no randomness, starts no thread, and measures no time. Within one run, the class tables of one colour count appeared in lexicographic order in every signature run of the prototype of Section 7.10. The design does not rely on it, because GRP-37 sorts the colourings of each colour count after the expansion. Whether a call is complete to a colour count can depend on the order and the spelling of the relators, as Section 6.2 states for whether an enumeration finishes within its limit.

### 7.18. Valid Colourings

GRP-42. A colouring MUST count as valid exactly when, for every generator $j$ of its presentation, the image of coset 0 under generator $j$ in its standard table is not coset 0, and `Colouring::contained_generator` MUST return the least generator $j$ whose image of coset 0 is coset 0, or `None` when no generator has one, because the owner's criterion of `DEC-65` speaks of adjacent tiles, which this crate cannot see, and this condition on row 0 of the table is the form of that criterion that `UNV-35` records. The test reads one entry for each generator.

No source read states that this condition on the side generators holds exactly when no two tiles that share a side have the same colour. `UNV-35` records it (Section 13), with its reading that a generator lies in the subgroup when it fixes coset 0, and the test "Valid neighbours" of `escher-render` checks it on samples (Section 8.1 of `docs/design/render.md`).

The colouring with one colour is the whole group, so it contains every generator and is valid only for a presentation with no generator, such as the side presentation of the domain of `1`, whose one tile has no neighbour. For every other presentation, `Valid` and `ValidClasses` give no colouring of the colour count 1. The owner keeps the colouring with one colour available as an exception (`DEC-65`), and the command line provides it without this crate (Section 4.6 of `docs/design/cli.md`).

Whether a domain has at most one valid colouring with 2 colours is not established by any source read (`UNV-36`). The command line relies on it to name that colouring `parity` (Section 4.6 of `docs/design/cli.md`), and the test "One valid 2-colouring" checks it (Section 10.1).

### 7.19. Classes That Fix No Coset

`free_classes_of_index` serves the whole-face fills of `escher-render`, whose arrangements are the subgroups in whose coset table no element of the stabilizer of a face other than the identity fixes a coset (Section 5.30 of `docs/design/render.md`, `UNV-42`). The caller passes those elements as words, and this crate knows nothing of faces. A filter in the caller over a listing with `ColouringSet::All` would store every conjugate toward the stored limits of GRP-36, which a listing of a large index reaches, and would rank the ids among all subgroups. `ColouringSet::ValidClasses` cannot serve, because it drops every subgroup that contains a generator, and an arrangement may contain the reflection in a side (`DEC-112`, `DEC-115`).

GRP-45. `free_classes_of_index` with the colour count $k$ MUST run the bound $k$ once, as `colourings_of_index` does (GRP-35), with the runs, the first-in-class test, and the leaf check of `ColouringSet::All`, MUST keep a class table $T$ of exactly $k$ rows only when no word of `stabilizer` fixes a coset of $T$, that is when `trace(a, w)` differs from $a$ for every coset $a$ and every word $w$, MUST keep no other relabelling of the class, MUST rank the kept tables by GRP-37 among themselves, from `a` with no gap, and MUST give each kept colouring its own id as its class, because the filter reads only the permutations of the table, which a relabelling of the cosets renames without changing whether a coset is fixed, so one table decides for every subgroup of its class (GRP-34), and the stored limits of GRP-36 then count only the kept tables.

The call counts every read of a table entry of the filter as a step and compares it with the budget at each read (GRP-29). A stop at a limit gives `is_complete()` false and an empty list, and claims nothing, as for `colourings_of_index`. The kept tables are class tables, so for a set that holds a class table $T$, `colourings_of_index` with `ColouringSet::All` lists $T$ with `class()` equal to `id()`.

An empty list of words keeps every class of the colour count. The empty word fixes every coset, so a list that holds it keeps none. The words are traced as given, without reduction.

GRP-46. `free_classes_of_index` MUST check the words of `stabilizer` against the limits of the subgroup generators of `enumerate`, at most 256 words of at most 4096 letters each (Section 15), and that every letter names a generator of the presentation, in this order: the number of words, then each word in order, its length before its letters, then the expanded relator letters, and MUST return `LimitExceeded` or `InvalidWord` with the texts of Section 8.2 before any allocation that depends on them, because the words come from the caller, and the limits of the subgroup generators already bound words of this kind.

## 8. Errors

### 8.1. Structure

GRP-22. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, and its `Display` text MUST be `cannot `, the operation, `: `, and the text of Section 8.2, because callers match on the kind, and `escher-cli` prints the text as a diagnostic that its snapshots fix.

`Error` has two private fields:

- `operation`: A `&'static str`, which is `"build presentation"` for `Relator::new`, `Presentation::new`, and `conway_presentation`, `"enumerate cosets"` for `enumerate` and `CosetLimit::new`, `"search subgroups"` for `count_colourings`, `colourings`, `colourings_of_index`, `free_classes_of_index`, `IndexLimit::new`, and `SearchBudget::new`, and `"read colouring id"` for the `FromStr` of `ColouringId`.
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
| `LimitExceeded`, for the search | An index limit or a search budget out of range, or a presentation over the expanded-letter limit of Section 15 | `the index limit 0 is not between 1 and 64`, `the search budget 0 is not between 1 and 68719476736`, `the relators expand to 120011 letters, more than 65536` |
| `InvalidWord`, for `free_classes_of_index` | A word of `stabilizer` with a letter outside the generators (GRP-46) | `stabilizer element 2 uses a generator that the presentation does not have` |
| `LimitExceeded`, for `free_classes_of_index` | More than 256 words, or a word of more than 4096 letters (GRP-46) | `the stabilizer has more than 256 elements`, `stabilizer element 2 has more than 4096 letters` |
| `InvalidColouringId` | A text that is not a colouring id of Section 7.15 | `the colouring id is not a colour count from 1 to 64 followed by 1 to 4 lower-case letters` |

With the prefix, the full texts read, for example, `cannot build presentation: the signature *(inf)22 has an infinite order`, `cannot enumerate cosets: the enumeration defined 16384 cosets without finishing`, and `cannot read colouring id: the colouring id is not a colour count from 1 to 64 followed by 1 to 4 lower-case letters`. Every text is in lower case, with no trailing punctuation (ERR-4 of the coding standard). No text cites a document or a rule identifier. The text of `InvalidColouringId` holds none of the text that it rejects, because that text comes from the user and may hold control characters (CLI-49 of `docs/design/cli.md`).

A search that stops at a limit of Section 7.12 is not an error (GRP-35), so no error kind reports it.

When an input has more than one fault, each operation returns the first in a fixed order. Section 4.2 gives the order of `Relator::new`, and Section 4.3 that of `Presentation::new`. `enumerate` checks the number of subgroup generators, then each subgroup generator in order, its length before its letters, then the expanded relator letters, and then the table entries. `CosetLimit::new`, `IndexLimit::new`, and `SearchBudget::new` have one check each, and the three search functions with a `ColouringSet` check only the expanded relator letters, because their other arguments are valid by construction. `free_classes_of_index` checks its words first, in the order of GRP-46.

## 9. Resource Limits

The crate limits the generators, relators, letters, subgroup generators, expanded letters, cosets, and table entries of every input, and the index, the steps, and the stored colourings of every search. `Relator::new` and `Presentation::new` check their limits at construction, `enumerate` checks its limits at step 1 of Section 7.3, and the search checks its limits at step 1 of Section 7.11, each before any allocation that depends on them. Section 15 states the values, the memory bound, and the residual risk on time, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 10. Testing

### 10.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard), and the integration tests form the one binary of Section 3.3. Every test stays well under a minute in a debug build. The presentation tests of `escher-domain`, which enumerate the side presentations of its domains with this crate, belong to that crate (Section 12.1 of `docs/design/domain.md`), among them the test of the spherical side orders that DOM-64 of that document names.

The tests of the search write their presentations by hand, because this crate cannot build a domain. In them, the Coxeter presentation of `*pqr` is $\langle s_0, s_1, s_2 \mid s_0^2, s_1^2, s_2^2, (s_0 s_1)^p, (s_1 s_2)^q, (s_2 s_0)^r \rangle$, and a side presentation of a rotation domain is written as `escher info --presentation` prints it.

The tests use these named input sets, in the modules that Section 3.3 names:

- The worked examples of the enumeration and of the search, with their sources, as Section 10.2 lists them.
- The spherical signatures: the 7 series for $n$ from 1 to 12, each written with one digit from 2 to 9 and in parentheses otherwise, so that $n = 1$ is `(1)`, and the 7 particular types. The 91 strings give 87 distinct canonical signatures, `1` among them.
- Sims's presentation $\langle x, y \mid x^n = y, y^n = x \rangle$ for $n$ from 3 to 8, whose enumeration defines more cosets than its order for $n \geq 5$.
- The random presentations: 2 or 3 generators, 1 to 4 relators of 1 to 6 letters, and 0 to 2 subgroup words of up to 4 letters. A wider strategy for the test of no panic also draws relators that reduce to the empty word and words with a generator that the presentation lacks.
- The published presentations of the search: the Coxeter presentations of `*732`, `*832`, `*542`, `*432`, `*642`, `*842`, and `*664`, the side presentations of `732`, `832`, and `542`, and the Conway presentations of `532`, `642`, `433`, and `2222`.

The tests form these layers:

| Layer | What it checks | What it catches |
| --- | --- | --- |
| Worked examples | The test "Worked examples" compares every fixture of Section 10.2 with its printed table or index, and another test compares the permutations that [HULPKE2010] and [HOLT2013] print. A unit test compares the Conway presentations of Section 5.4 with their text. "Spherical orders" enumerates every spherical signature over the trivial subgroup and compares the index with `group_order()`, and "Order two" does the same for `x`, `22`, and `*` | A wrong step, a wrong standard form, and a reversed commutator, a reversed conjugation, a missing global relator, or a wrong exponent in a Conway presentation |
| Table laws | Property tests. "Table laws" asserts, over the random presentations, columns that are permutations, inverse columns that are mutually inverse, every relator tracing every coset to itself, every subgroup generator fixing coset 0, a transitive action, and the standard form. "Invariance" asserts, over the finite presentations of the worked examples and of the spherical signatures up to order 24, an equal table after a permutation, rotation, or inversion of the relators or a reordering of the subgroup generators. Other property tests check the laws of words and that `enumerate` never panics | A table that breaks a law, a table that depends on the order of the input, and a panic |
| Run statistics | Unit tests that read crate-private state, mostly the run statistics. "Closing pass idle" asserts that the closing pass changes nothing and the rescan of step 4 finds no gap, at the default cap and at a cap of 1, with equal tables when both runs finish. "One-letter relators" asserts that the Conway presentations of `*532`, `x`, and `*` define exactly their group order of cosets. Others assert the sizes of the conjugate lists, equal tables at a cap of 1, and that scan-and-fill reads each letter of a long subgroup generator once | A code-only step that leaves work to the closing pass, a lost rule of Section 7.5, a duplicated conjugate, and a scan-and-fill that restarts |
| Limits | Each limit of Section 15 at its value and one above it, the limit on table entries at `CosetLimit::MAX` with 8 and 9 generators, `532` at the limits 60 and 59, Sims's presentation at the number of cosets that it defines and one less, and the infinite groups of "Infinite groups", `*632`, `2222`, `o`, and `*732`, at a limit of 1000. Long subgroup generators of 4096 letters give their index at a limit equal to it. Unit tests pin the capacities of the table, the representatives, the queue, the relabelling, and the run of a search, and the sizes of a stored and a pending colouring | A count of live cosets against the limit, a limit off by one, deductions not processed in scan-and-fill, and a vector that doubles beyond the memory bound |
| Search fixtures and published counts | "Search examples" compares the search fixtures of Section 10.2. "First in class" runs the test of GRP-32 on the partial tables of Sims's Example 9.3 and of SymPy's doctest. "Frettloh Table 1", "Frettloh Table 2", and the cube compare the classes with a fixed coset with the tables of [FRETTLOH2008], exactly up to the last printed value or the limit of the test. Other tests compare the counts of 2 colours with [SINGERMAN2014] and check that a colouring exists for each count of the captions of [BULATOV2010]. "Search agrees with enumeration" gives the Schreier generators of each colouring to `enumerate` and compares the tables | A lost class, a class table that is not the least, and a wrong count |
| Search properties | Property tests over the random presentations at limits up to 5. "Least in class" asserts that each class table is the least of its class and that a class is the distinct relabellings of its class table. "Pruning" compares the first-in-class test on every node with the test on complete tables only and with no test. "Leaf statistics" asserts that the leaf check discards no table. Other property tests assert equal ids under a rewriting of the relators, ids that no limit or budget changes, and equal counts from the three functions. Unit tests on the Coxeter presentation of `*642` check the completeness of the runs by index and of a stop at a stored limit | A pruning that loses a class, a step whose fault only the leaf check hides, an id that depends on the run, and a wrong `complete_to()` |
| Valid colourings | "Valid filter" and "One valid 2-colouring" each run as a property test over the random presentations, at limits up to 5 for "Valid filter", and as a plain test over the published presentations of the search, to 10 colours for "Valid filter". "Valid filter" asserts that `colourings` with `Valid` returns exactly the colourings of a call with `All` and the same arguments whose `contained_generator()` is `None`, with equal tables in the same order and ids numbered again from `a` with no gap in each colour count, and that `Colourings::count`, `count_colourings`, and `colourings_of_index` with `Valid` agree with that list. "Valid classes" runs over the same inputs and limits as "Valid filter" and asserts that `colourings` with `ValidClasses` returns exactly the colourings of a call with `Valid` and the same arguments whose `class()` equals their `id()`, with equal tables in the same order and ids numbered again from `a` with no gap in each colour count, that each of them has `class()` equal to `id()`, and that the counts of the three functions with `ValidClasses` agree with that list. "One valid 2-colouring" asserts that `colourings_of_index` with `Valid` and the colour count 2 lists at most one colouring. Unit tests assert that `contained_generator()` is the least generator that fixes coset 0, that `1a` is valid only for the presentation with no generator, that a class whose class table is not valid keeps its valid conjugates and takes its class id from a kept colouring, that the expansion with `ValidClasses` stops at the first valid relabelling of a class, and that the stored limits count only the colourings that `Valid` or `ValidClasses` keeps | A filter on the class tables, a filter that skips a relabelling, an id ranked among all subgroups under `Valid`, a class id of a dropped table, a class under `ValidClasses` that keeps no colouring, two colourings, or a relabelling other than its least valid one, an expansion under `ValidClasses` that relabels past the first valid table, and a count taken from another set |
| Free classes | "Free filter" runs as a property test over the random presentations at limits up to 5, with 0 to 3 random words of up to 4 letters, and as a plain test over the Coxeter presentations of `*632`, `*442`, `*732`, and `*542` with the words of the stabilizer of the corner of order $p$ and the colour count $2p$, where it also asserts the number of kept classes. Both share one set of assertions, which claim nothing when either call stops at a limit. They assert that `free_classes_of_index` returns exactly the colourings of `colourings_of_index` with `All` and the same arguments whose `class()` equals their `id()` and whose table no word maps a coset to itself, with equal tables in the same order, ids numbered again from `a` with no gap, and `class()` equal to `id()`, and that every other subgroup of a kept class also fixes no coset under the words, which checks on these inputs that one table decides for its class. Unit tests assert that an empty list keeps every class, that a list with the empty word keeps none, that the stored limits count only the kept tables, and the checks of GRP-46 in their order | A filter on a relabelling other than the class table, an id ranked among all subgroups, a filter that misses a word or a coset, and a stored limit reached by dropped tables |
| Colour rule | "Colour rule" asserts, for the colourings of the Conway presentation of `532` up to 6, that `colour_of` of a side $i$ followed by a sequence $u$ is the image of `colour_of(u)` under `Letter::inverse_of(i)`. "Word invariance" asserts that inserting a relator into a sequence of sides leaves its colour unchanged | A `colour_of` that does not follow GRP-38 |
| Failure paths and text | Every error kind and text of Section 8.2, each rule of the text form of Section 4.4, the colouring id text in both directions, the infinite orders of Section 5.3, the presentation with no generator, the auto traits of Section 3.2, and the doctests | A missing check, a wrong order of checks, and a changed text |

The tests that compare the search with a source, the search fixtures, the tests of the first-in-class test, and the other layers above the row "Valid colourings" call the search with `ColouringSet::All`, because their sources count or print every subgroup. Only "Pruning" and "Leaf statistics" also run with `Valid` and `ValidClasses`. With `ValidClasses`, "Pruning" compares the test on every node only with the test on complete tables, and not with the run that has no first-in-class test, because in that run each valid leaf is its own class.

"Colour rule" checks only that the code follows GRP-38, and "Word invariance" holds for the forward trace too, because a relator acts trivially on the cosets either way. Neither checks that GRP-38 is the right rule. Only the assertion of GRP-41 on samples of a rendering checks that, and it belongs to `escher-render`, which has the fold.

The property tests commit their regression files (TEST-9 of the coding standard). They run with the default configuration of `proptest`. "Closing pass idle" runs as a unit test, because the run statistics are crate-private, and it loads the strategy of `common` through the `#[path]` module of Section 3.3, as `escher-render` loads its golden comparison.

Some single cases of the table tests also run as tests of their own: `2*3` and `3*2` of "Spherical orders", `2222` at a limit of 50 of "Infinite groups", and three small searches, of $S_3 = \langle x, y \mid x^2, y^3, (xy)^2 \rangle$ to 3 colours and of the fixtures of [SIMS1994] p. 209 and [HOLT2005] Example 5.10.

GRP-28. Withdrawn on 2026-11-19.

This design leaves out a property test that the index of a cyclic subgroup times its order equals the order of a finite group, because it relies on Lagrange's theorem, which no section of `docs/research/` states.

GRP-24. The worked examples of Section 10.2 and the property test "Closing pass idle" MUST stay in the crate while `UNV-19` is open, because they are what checks the steps that only code states, and without them a change to a step that keeps every order right but breaks a table can pass.

GRP-41. The tests "Search examples", "First in class", "Frettloh Table 1", "Frettloh Table 2", "Least in class", "Pruning", "Leaf statistics", and "Search agrees with enumeration" MUST stay in the crate while `UNV-19` is open, and "Colour rule" MUST stay while `UNV-17` is open, because they check the code-only steps of the search, the pruning that no source proves, and the rule that no source states. A defect that keeps every count right, such as keeping the greatest table of each class, passes every other test (Section 10.3). The caller that has the fold asserts, on the samples of a rotation domain, that the colour of $P_i^{-1} x$ is the image of the colour of $x$ under `Letter::inverse_of(i)`, as the test "Exact permutation" of Section 8.1 of `docs/design/render.md` does, because only that assertion, or a golden image of a chiral colouring, tells the colouring of GRP-38 from its mirror image (Section 7.16).

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

The relations of Holt's quaternion example are written as the relators $a^{-1}bab^{-3}$ and $b^{-1}aba^{-3}$ (Section 5.1). The SymPy examples come from doctests of the HLT enumeration, so only their standard rows and their indices transfer to this design, and not their counts of cosets defined (Section 7.20 of the colouring reference).

The Holt fixture of p. 15 prints $x \mapsto (1,2)$ and $y \mapsto (1,2,3)$ for the subgroup $\langle xy \rangle$. Under the right action of this design that is the standard table of $\langle yx \rangle$, and for $\langle xy \rangle$ the standard table has $y \mapsto (0,2,1)$ when numbered from 0. The index is 3 either way, so the fixture asserts the printed table for $\langle yx \rangle$ and only the index for $\langle xy \rangle$.

The test "Search examples" compares each fixture of the search of Section 7.20 of the colouring reference in the same way. A source with one column for an involution $x$ gives the column of $x^{-1}$ equal to that of $x$. Where a source prints classes, the test compares the class tables, and where it counts subgroups, the colourings:

| Source | Presentation and call | Asserts |
| --- | --- | --- |
| [HOLT2005] Example 5.10 and the SymPy doctest | $\langle x, y \mid x^2, y^3, (xy)^4 \rangle$, `colourings` with limit 4 | One class of each colour count 1 to 4, whose class tables are the four tables of the doctest |
| [SIMS1994] Example 6.1 | $\langle x, y \mid x^3, y^3, (xy)^2 \rangle$, limit 4 | Three classes, of colour counts 1, 4, and 3, whose class tables are the printed tables with 1 subtracted from every entry |
| [SIMS1994] Example 9.2 and p. 207 | $\langle x, y \mid x^2 \rangle$, limit 5 | 1, 3, 7, 23, and 71 colourings of the colour counts 1 to 5, and within each colour count up to 3 the colourings in the order of their ids are the printed tables of that count in their printed order. Sims prints the 11 tables in the lexicographic order over all entries, which interleaves tables of 2 and 3 rows, while ids order by colour count first. The test therefore does not compare the 11 tables as one sequence |
| [SIMS1994] p. 209 | The same, limit 3 | 1, 3, and 3 classes of the colour counts 1, 2, and 3, whose class tables are the printed tables 1, 2, 3, 4, 5, 6, and 8 |
| [SIMS1994] Example 6.2 | $\langle a, b \mid a^2, b^3, (ab)^7 \rangle$, limit 10 | The classes with a coset fixed by $abab^{-1}$ are one of colour count 1 and two of colour count 7, each with a member equal to a printed table |
| [SIMS1994] Examples 6.3 and 6.4 | Higman's group at limits 8 and 12, the group of order 55 at limit 11 | Only the colour count 1, and the classes of colour counts 1, 5, and 11 |
| [SIMS1994] Example 6.5 | $M_{11}$, limit 12 | Classes of colour counts 1, 11, and 12 |
| [GAP] Section 47.10-1 | $\langle a, b \mid a^2, b^3, (ab)^5 \rangle$, limit 10 | 4 classes |
| [GAP] Chapter 47 | The group with six relators of Section 7.20 of the colouring reference, limit 6 | Classes of the colour counts `[1, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6]` |

The class tables of a source are least in their class, so they are the tables whose `class()` equals their `id()`. Sims's node counts for Higman's group, the group of order 55, and $M_{11}$ depend on how the deductions are processed (Section 7.20 of the colouring reference), so no test asserts them. A measurement of the crate reproduced them exactly with one run.

### 10.3. Defects the Tests Catch

This subsection is non-normative. It records deliberate defects that were applied by hand, one at a time, to a copy of the crate to check the tests of Section 10.1. They are not part of the test suite.

The layers of the enumeration caught a merge that keeps the larger coset, a scan without its push, SymPy's caller lists, a standard form over the generator columns only, a count of live cosets, a missing global relator, a wrong corner exponent, a missing rule of Section 7.5, a scan-and-fill that restarts or processes no deductions, and capacities that double. The layers of the search caught swapped comparisons in the first-in-class test, an undefined entry compared as 0, a search without pushed deductions, a missing candidate $d = c$, rows not restored on undo, a wrong rule for the tables that a run by index keeps, a ranking in the order of the run, and a forward trace or a reversed trace without the inverse in `colour_of`. Swapped comparisons in the first-in-class test keep the greatest table of each class, so every count stays right and only the tests that compare tables catch them. A filter of `free_classes_of_index` that skips the last coset was caught by the property test "Free filter" and not by its plain test.

Ranking the tables in the order in which a run finds them is a defect for the ids of GRP-37, because the expansion of a class does not keep that order, even though the prototype found the class tables of each run in lexicographic order (Section 7.17).

No test detects these five defects, and each part stays for the reason given:

- Dropping the scan at $b$ of Section 7.4. The scan is redundant, because the lists hold the conjugates of every relator and of its inverse, and it stays because GRP-11 follows Hulpke's rule, which scans there.
- Dropping the push of a coincidence, or the closing pass, in an implementation that has no other defect. Both stay for the soundness reasons of GRP-12 and GRP-14.
- Resuming the traces of scan-and-fill without replacing their cosets by their representatives. The lookup stays because the row of a removed coset is not updated after the removal, so its entries can be stale.
- Dropping the leaf check of the search. It stays for the reason of GRP-33.

## 11. Prototype Measurements

Withdrawn on 2026-11-22. Where a measurement of the crate or of a prototype outside the repository is the reason for a limit, a rule, or a choice, the section of that limit, rule, or choice states the conclusion with its scope. The heading stays so that Sections 12 to 17 keep their numbers.

## 12. Dependencies

GRP-23. The crate MUST declare `escher-signature` and `thiserror` as its only normal dependencies and `proptest` as its only dev-dependency, because each further dependency adds build time, licence obligations, and supply-chain risk (DEP-2 of the coding standard), and no usable crate for presentations or coset enumeration exists (Section 14 of `docs/research/rust-ecosystem.md`).

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `escher-signature` | Normal | None | `conway_presentation` reads a `Signature` |
| `thiserror` | Normal | Default | Derives `Display` and `std::error::Error`, as the other library crates do. Section 3.1 of the architecture lists it for this crate |
| `proptest` | Dev | `default-features = false`, with `std` | The property tests |

The crate sets `publish = false`, inherits the workspace fields and lints, and declares no Cargo feature. `Signature` appears in the public surface as the argument of `conway_presentation`. That is a dependency inside the workspace, which is not published (`DEC-22`). `algebraeon-groups` is under GPL-3.0-only, and no code is copied from it.

## 13. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, the sections of this document that rely on it, and what the code does about it:

| Label | What the code relies on | Relied on in | What the code does |
| --- | --- | --- | --- |
| `UNV-19` | The steps that only this design, SymPy, or SymmHub states, which the columns "Code-only part" of Sections 7.3 and 7.11 list, while [HOLT2005] and [SIMS1994] state the other steps (item 23 of Section 14.1 of the colouring reference). It also covers four statements that the sources make without a proof for this crate's code: that the coincidence processing keeps (R1), that the code terminates for a finite index, the bijection between subgroups and lenlex standard tables (item 26), and that pruning a partial table by the first-in-class test loses no class (item 22). It covers the indices of the worked examples that no source states (item 25) | 6.2, 7.1 to 7.7, 7.10 to 7.15, 10.2 | Ends every run of `enumerate` with the closing pass of GRP-14, and passes on a table of the search only after the leaf check of GRP-33, so a returned table rests on the closed-table statements of [HULPKE2010] p. 75 and of Theorem 5.2 of [HOLT2005]. Keeps the two branches of Section 7.7, and bounds every run by a limit, so termination of a run never depends on the statement. The worked examples and "Closing pass idle" check the steps of the enumeration (GRP-24), and the tests of GRP-41 those of the search. "Invariance", "Pruning", and "Least in class" check the bijection and the pruning on their inputs. The worked examples assert only the rows and indices that the sources print |
| `UNV-17` | The rule of GRP-38: that the product of the generators of a fold's sides in fold order maps the domain onto the tile of the sample, that the right coset $S g^{-1}$ numbers the left coset $gS$ of Section 5.4 of the architecture, and that the coset colouring is perfect for a group that Lemma 2.1 of [FRETTLOH2008] does not cover. Also how conjugate subgroups relate to colourings (Section 3.4 and items 1 to 3 of Section 14.1 of the colouring reference) | 6.1, 7.10, 7.16 | Implements the rule once, in `colour_of`. "Colour rule" pins the code to the rule, and the assertion of GRP-41 on the samples of `escher-render` checks the rule on rotation domains. With `All`, lists every subgroup of finite index as its own colouring (`DEC-37`), so the listing holds every coset colouring up to the limit, whatever the relation of the conjugates, and with `Valid` every valid one |
| `UNV-35` | That a colouring is valid under the owner's criterion of `DEC-65` exactly when no side generator fixes coset 0 of its table | 7.13, 7.18 | Tests the condition of GRP-42 on row 0 of each table, in `contained_generator` only. "Valid neighbours" of `escher-render` checks the equivalence on samples |
| `UNV-36` | That a domain has at most one valid colouring with 2 colours | 7.15, 7.18 | No code of the crate depends on it. The command line names that colouring `parity`, and "One valid 2-colouring" checks the statement on its inputs |
| `UNV-22` | The Conway presentation of a boundary without corners in general | 5.2 | Implements the reading of GRP-7. The test "Spherical orders" checks `n*` and `2*n` against $2/\chi$, and "Order two" checks `*` |

The crate does not rely on `UNV-41` to `UNV-46`. `free_classes_of_index` keeps the class tables in which its words fix no coset, whatever the words mean, and `escher-render` states how the whole-face fills rely on those statements (Section 10 of `docs/design/render.md`).

The crate does not rely on `UNV-32`, which concerns the side presentation of `escher-domain` (Section 7 of the plan). Its colourings of a domain are those of the side presentation that it receives, so a wrong letter order of a cycle relator would give the colourings of another presentation without an error.

## 14. Assumptions, Choices, and Points for the Owner

### 14.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what checks it:

- A relation $u = v$ of [CH] is the relator $u v^{-1}$. SymmHub's code does the same (Section 12.8 of the SymmHub reference). The tests "Spherical orders" and "Order two" check the result for every spherical type, and a source that states the conversion would settle it.
- The groups of the Euclidean and hyperbolic signatures `*632`, `2222`, `o`, and `*732` are infinite, so their enumeration reaches any limit. The test "Infinite groups" relies on it, and only reports a run that reached its limit (GRP-17). A source that states that a Euclidean or hyperbolic orbifold group is infinite would settle it.
- Writing each entry of a one-letter relator when its row is created gives the same table as finding the entry by a scan. Rule (b) of [CANNON-HAVAS1992] forces the entry (Section 7.5), and the closing pass and the test "One-letter relators" check it. Neither [HOLT2005] nor [SIMS1994] treats a relator of one letter (Section 7.9 of the colouring reference), so a source that does, or Lean, would settle it.
- The mirror of a class of a rotation side presentation, the class of the table whose generator and inverse columns are swapped, is the class that Frettloh identifies with it as conjugate in $G_{p,q}$ (Section 5.4 of the colouring reference). This is the prototype's reading of the automorphism $s_i \mapsto s_i^{-1}$, which conjugation by a reflection realizes, and no source states it. "Frettloh Table 2" relies on it and compares every count of the table under it.
- The single case of $S_3$ (Section 10.1) asserts 1, 1, and 3 colourings of 1 to 3 colours and 3 classes, which no source of the colouring reference prints, and the test takes them from the subgroups of $S_3$. A source that prints the subgroups of $S_3$ with their indices would settle it.
- The generators $c$, $b$, and $a$ of [FRETTLOH2008] are $s_0$, $s_1$, and $s_2$ of the Coxeter presentation of Section 10.1. The relators $(bc)^p$, $(ab)^q$, and $(ac)^2$ of $G_{p,q}$ (Section 5.1 of the colouring reference) match the relators of that presentation under this reading, and "Frettloh Table 1" relies on it.
- A generic sample has a trivial stabilizer, so the fold of a sample names one tile. GRP-38 relies on it, and the prototype of Section 7.16 checked it on samples.

### 14.2. Choices of This Design

Every choice in this table is a proposed default. No other document dictates it, and the owner has decided none of them:

| Choice | Alternative | Section |
| --- | --- | --- |
| Felsch only, no HLT (`DEC-33`) | HLT as a second strategy, which Holt says may not meet (R4), and which [HOLT2005] states meets its Property 5 when it defines every open entry of a row after its scans | 7.1 |
| Coset limit default 16384 and maximum 1048576, with a bound of $2^{24}$ table entries (`DEC-33`) | Other values, which would change the margin over the largest group order or the memory bound | 6.3, 15 |
| Generator names `x1`, `y1`, `a1`, `g1`, `r1`, `l1`, `z1`, `w1`, and the relator text of Section 4.4 (`DEC-34`) | The letters of [CH] with indexes | 4.4, 5.1 |
| Right cosets and the column order of [SYMPY-CT] in tables (`DEC-36`) | Left cosets, which no enumeration source uses | 6.1 |
| A relator as base and exponent, expanded only in `enumerate` | Flat words with a limit, which refuse large orders even for printing | 4.2 |
| A `u8` generator index in `Letter` | A fallible constructor over a wider index | 4.1 |
| `UnsupportedSignature` for an infinite order | A presentation without a source | 5.3 |
| Scans selected by the letter of the popped entry | SymPy's caller lists, which never finished on the rotation doubles of a prototype | 7.4 |
| A closing pass at the end of every run | No pass, which leaves correctness on the code-only steps | 7.7 |
| The rule for one-letter relators | Leaving them to the scans, which doubles the cosets defined for presentations with a boundary or crosscap | 7.5 |
| Distinct cyclic conjugates | Every conjugate, as Hulpke's note lists them | 7.2 |
| Columns of involutions not merged | Merged columns, which halve the table of a reflection group and complicate every index | 7.2 |
| A stack cap of 65536, then an empty stack and a lookahead that keeps its pushes | No cap, or the order of [HOLT2005] and SymPy, lookahead then empty | 7.6 |
| The design pushes $(\mu, x)$ in a coincidence | SymPy's push of $(\delta, x^{-1})$, which gave the same results on the prototype | 7.3 |
| The capacities of the table, the representatives, and the queue capped at limit times columns, limit, and limit, and the relabelling vectors allocated at their exact sizes | Plain doubling vectors, which exceed the memory bound | 15 |
| Step 4 with a forward cursor and one final rescan, kept although no run has reached the rescan | A scan from the first row at every definition, or no rescan, which loops without end if (R1) fails | 7.3, 7.7 |
| The deductions of step 3 processed after each definition and after each subgroup generator | Processing them once after all subgroup generators, which makes a long subgroup generator define a coset for each letter | 7.3 |
| Scan-and-fill resumes its traces after each definition | A new scan from coset 0 after each definition, which reads a number of letters quadratic in the length of the word | 7.3 |
| Duplicate conjugates removed across the whole presentation by least rotation | Removal within each relator only | 7.2 |
| No work budget for `enumerate` | A budget in letter steps with its own error kind | 15 |
| The variant of the search of Section 7.10, with every relator used, pushed deductions, a leaf check, an undo trail, and no subgroup argument (`DEC-41`) | A split into short and long relators, which the prototype found catastrophic at 5 letters, copies of the table at each candidate, or a subgroup argument | 7.10, 7.11 |
| One class table for each class, then the expansion into the subgroups of the class (`DEC-41`) | A search over all subgroups without the first-in-class test, which visits every conjugate in the tree | 7.13 |
| Repeated relabellings removed by comparison with the distinct ones kept, with an exit at the first difference, counted in steps | A sort of the relabellings of a class, slower for each counted step and with uncounted comparisons | 7.13 |
| The first-in-class test stops at an undefined entry | The rule of `low_index`, which compares it as 0 and which lost classes on the prototype under the prototype's reading of it | 7.11 |
| One run first, then runs by index on a fresh budget only when it stops (`DEC-42`) | Runs by index from the start, which cost several times the steps of one run (Section 7.14), or one run with no completeness by index | 7.14 |
| `IndexLimit` maximum 64, `SearchBudget` default $2^{28}$ and maximum $2^{36}$, and stored limits of 65536 colourings and $2^{22}$ entries (`DEC-42`) | A default of $2^{29}$, about 0.5 to 1.1 s for one run, or a limit on classes, which is the wrong quantity for the per-subgroup listings of `All` and `Valid` | 7.12 |
| A stop at a limit returns no error, only a result that is complete to a colour count or, for one colour count, not complete (`DEC-42`) | An error kind for a stop, which would hide the colourings that the call did find | 7.14 |
| Ids ranked among the colourings of the set of one colour count, in lexicographic order, in bijective base 26 with up to 4 letters, assigned only for a complete colour count (`DEC-43`) | Ranks among the class tables, which `DEC-37` replaces, or the order in which a run finds the tables | 7.15 |
| `colour_of` by the reversed trace of inverse columns from coset 0 (`DEC-40`) | A permutation carried through the fold, or a coset carried forward, which gives the forward trace | 7.16 |
| Three functions, one that counts, one that lists every colour count, and one that lists one colour count | One function with a mode argument, or an iterator, which makes a stop at a limit hard to define | 3.1 |
| `ColouringsOfIndex` as the result of `colourings_of_index`, with `is_complete()` and no `complete_to()` | `Colourings` with `complete_to()` equal to the colour count or 0, which a caller can misread as proof about the colour counts below it | 3.1, 7.14 |
| A run holds its tables as pending colourings and builds each `Colouring` once, with its final id and class | `Colouring` values built with placeholder ids and classes and patched after the ranking | 7.15, 15 |
| `Colouring::class` as the id of the class table that the search kept | No class data, a class size, which a caller can count from the list, or the least id of the class computed after the expansion, which hides a defect of the first-in-class test | 3.1, 7.13 |
| Subgroup words scanned as given, without reduction | Freely reducing them first | 7.3 |
| The set as a parameter `ColouringSet` of each of the three functions, with `All` kept for the tests against the sources | Separate functions for the valid colourings, or a filter in each caller, which counts dropped colourings toward the stored limits | 3.1, 7.13 |
| The filter in the expansion of each class, on every relabelling | A filter on the class tables, which loses the valid conjugates of a class table that is not valid | 7.13 |
| The class of a kept colouring from the kept colouring of the least base coset | The least id of the kept colourings computed after the ranking, which with `All` would hide a defect of the first-in-class test | 7.13 |
| `contained_generator` returns the least generator in the subgroup | A `bool`, which tells a test nothing about which side two neighbouring tiles of one colour share | 3.1, 7.18 |
| `free_classes_of_index` as a function of its own with a list of words, which keeps only class tables, for the arrangements of the whole-face fills (`DEC-115`) | A variant of `ColouringSet` that carries the words, which would make the set hold data and serve one caller in every function, or a filter in the caller over `All`, which stores every conjugate toward the stored limits | 3.1, 7.19 |
| `ValidClasses` as a third variant of `ColouringSet`, which keeps the colouring that names the class under `Valid` and may stop the expansion of a class at it | A filter in the caller over a `Valid` listing, which stores every valid conjugate toward the stored limits, or the least id of the class after the ranking, which needs every relabelling | 3.1, 7.13 |

### 14.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Time of an enumeration. No limit of the crate bounds the time of a run, and `escher-cli` cannot reach a slow input (Section 15). The design accepts this residual risk and recommends that a caller that takes untrusted input keep the default limit. The alternative is a limit on time or on work, such as a work budget in letter steps with its own error kind.
2. Strategy and maximum limit. The design proposes Felsch with a maximum limit of 1048576 (`DEC-33`). The alternative is another maximum, or HLT as a second strategy. The default limit of 16384 is the owner's decision (`DEC-69` of the architecture).
3. Cosets. The tables use right cosets, as every source does, while Section 5.10 of the architecture picks left cosets for colourings (`DEC-36`). The alternative is tables of left cosets, which no enumeration source supports.
4. Names. The design proposes ASCII names numbered by kind (`DEC-34`). The alternative is the letters of [CH], such as `P`, `Q`, and `lambda`, with indexes.
5. Search budget. The design proposes $2^{28}$ steps (`DEC-42`, Section 7.12). The alternative is $2^{29}$ steps, which completes more colour counts for the triangles with orders near 100.
6. Long power relators. The search at the default limit stops early for triangle groups whose corner orders are all near 100 (Section 7.14). The design accepts this. The recorded idea is the scan of a power relator by the cycle of its base, which Section 7.14 describes. It is unmeasured and would need its own design and tests.
7. Reference counts. The tests compare the search with printed counts only, because no GAP, Sage, or SymPy is available in the container. The alternative is fixtures generated offline with GAP or `low_index` and committed as data with their provenance, which would cover the Euclidean groups that no printed table covers.
8. A higher limit can complete fewer colour counts. When the one run at the limit $N$ stops, the runs by index share one budget (GRP-35), so a call with a higher limit can complete fewer colour counts than a call with a lower limit whose one run ends (Section 7.14). The design accepts this.
9. Tests kept for a reason that does not apply. TQ-11 of `docs/design/tooling.md` puts them to the owner, including the single cases of Section 10.1.
10. Withdrawn on 2026-11-22, when the owner decided on one valid colouring for each class on the command line (`DEC-37`, GRP-44). Merging the classes that a symmetry of the domain relates, such as mirror classes, is not adopted, and stays at most a possible later reduction.

## 15. Security Considerations

The threat model is a caller that passes a presentation, subgroup words, a coset limit, an index limit, a search budget, a text to read as a colouring id, side indices, or a signature from an untrusted party to this crate. The attacker can choose any value of the public types. The attacker cannot change the crate, its dependencies, or the caller.

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
| Expanded relator letters, summed over the relators, checked by `enumerate` | 65536 | Above the expanded length of the Conway presentation of every signature whose domain `escher-domain` builds, whose corner and cone orders are at most 100. The limit is sized for orders up to 1000, such as the 2023 letters of `*(1000)32` |
| Cosets | `CosetLimit::MAX`, 1048576 | Section 6.3 |
| Table entries, limit times columns | $2^{24}$ | Bounds the table at 64 MiB and its relabelled copy at as much |
| Index of a search | `IndexLimit::MAX`, 64 | Section 7.12. The table of a run then has at most $64 \cdot 512 = 2^{15}$ entries |
| Steps of a search | `SearchBudget::MAX`, $2^{36}$ | Section 7.12 |
| Colourings stored by one call | 65536 | GRP-36 |
| Table entries stored by one call | $2^{22}$ | GRP-36 |

The limits are private constants, and unit tests inside the crate read them. Expanded lengths are summed in checked `u64` arithmetic. Limit times columns is at most $2^{24}$, so every index into the table fits `usize` on every target. Entries are `u32`, and the undefined marker `u32::MAX` lies above the largest coset number.

GRP-20. The memory of one call of `enumerate` MUST stay within the bound of this section, the table, the representatives, and the coincidence queue MUST grow with their capacities capped at limit times columns, limit, and limit, and the vectors of the relabelling MUST be allocated at their exact sizes, because a vector that doubles its capacity can allocate up to twice what the limit allows, and the bound is what a caller relies on when it chooses a limit.

For a limit $L$, $c$ columns, $E$ expanded relator letters, $W$ letters of the subgroup generators in total, and $S_{max}$ the largest deduction stack of the run, the memory of a call is at most

$$\max(4 L c + 8 L + 16 S_{max},\ 8 L c + 12 L) + 24 E + 2 W$$

bytes beyond its inputs. The run has two phases, and the larger of them sets the first term:

- The enumeration: the table of `u32` entries, $4 L c$, the representatives and the coincidence queue, $4 L$ each, and the deduction stack. A stack entry takes 8 bytes, and the stack grows by doubling, so its capacity stays below twice $S_{max}$ and its term is at most $16 S_{max}$.
- The relabelling of Section 6.2: the stack and the queue are freed before it starts. The table, $4 L c$, and its relabelled copy, at most $4 L c$, stay with the representatives, the label map, and the order list, $4 L$ each.
- Both phases: the stored relators and their conjugate lists, $24 E$. Each letter is stored as a `u16` four times, the relator and its inverse each written twice in a row, which is 8 bytes. A letter starts at most two distinct conjugates, one of the relator and one of its inverse, each a span of two `u32`, which is 16 bytes.
- Both phases: the subgroup generators as columns, $2 W$, one `u16` for each letter, at most 2 MiB for 256 words of 4096 letters, kept for the whole call because steps 3 and 10 scan them.

The table, the representatives, and the queue each grow to the smaller of twice their length and their cap, $L c$, $L$, and $L$, and never below what the next entry needs. The queue holds a coset only when a merge removes it, so it never needs more than $L$ entries. The label map has one entry for each coset defined, the order list is allocated with that capacity, and the relabelled copy with the index times $c$ entries. At the maxima the bound is about 143.5 MiB, from 140 MiB for the relabelling, 1.5 MiB for $24 E$, and 2 MiB for $2 W$. At the default limit with the most columns, 512, it is about 68 MiB. Unit tests pin the capacities, and in a release build of the crate the peaks measured at `CosetLimit::MAX` stayed just below the bound.

Step 1 makes allocations outside the formula, before the table exists:

- For one relator at a time, the relator written out with its exponent as a `Word`, its freely reduced copy, its cyclically reduced copy, and its letters as columns. Each takes 2 bytes for each letter and at most three exist at once, so they take at most 6 bytes for each letter of the longest expanded relator, 384 KiB for 65536 letters.
- A prefix-function array of `usize` for one word at a time, 8 bytes for each letter, at most 512 KiB for a word of 65536 letters, freed before the next word.
- A `BTreeSet` of slice references, one for each distinct word, freed at the end of step 1.

$S_{max}$ exceeds the cap of 65536 only by the entries that one coincidence or one lookahead pushes before the next check of the cap (Section 7.6). The design has no smaller bound on that excess than the number of entries that one such pass writes. No measured run of the crate or of the prototype held more than 67 entries on the stack.

GRP-40. The memory of one call of `count_colourings`, `colourings`, or `colourings_of_index` MUST stay within the bound of this section, with the table, the trail, the deduction stack, and the frames of each run allocated at the size that the bound of that run sets, which is at most the index limit, because the stored limits of GRP-36 bound only the output, and a vector that doubles can exceed the bound.

For an index limit $L$, $c$ columns, $E$ expanded relator letters, $T \leq 2^{22}$ stored table entries, and $M \leq 65536$ stored colourings, the memory of a call is at most

$$24 E + 20 L c + 4 L^2 c + 12 L + 4 T + 160 M$$

bytes beyond its inputs, with these terms:

- $24 E$: the stored relators and their conjugate lists, as GRP-20 states.
- $20 L c$: the table of the run, $4 L c$, the trail of written slots, $4 L c$ in slots of 32 bits, the deduction stack, at most $4 L c$, with a capacity of $L c / 2$ entries of 8 bytes, and the frames, at most $8 L c$, with a capacity of $L c / 2$ frames of 16 bytes.
- $4 L^2 c$: the scratch of the expansion of Section 7.13, at most $L$ tables of $L c$ entries.
- $12 L$: the label map and the order list of the relabelling, and the list of the distinct base cosets of the expansion.
- $4 T + 160 M$: the stored tables, and the fixed part of each stored colouring. A run holds each found table as a pending colouring of 48 bytes, which a unit test pins. The pending list grows by doubling up to 65536 entries, so it takes up to 96 bytes for each colouring. The ids of the class tables take up to 8 bytes for each class. The returned list takes 56 bytes for each `Colouring`, which a unit test pins, and is allocated at its exact length while the pending list is consumed.

$T$ and $M$ are the largest numbers held at any moment of the call. While a run that stopped at a limit is discarded, they include what that run stored. At the maxima, with $L = 64$ and $c = 512$, the bound is about 36.1 MiB: 0.625 MiB for the run, 8 MiB for the scratch, 16 MiB for the stored tables, 10 MiB for the stored colourings, and 1.5 MiB for $24 E$. `count_colourings` stores nothing, so its bound has no $4 T + 160 M$. `free_classes_of_index` adds $2 W$ for its words, as `enumerate` does for its subgroup generators, at most 2 MiB, and expands no class, so its scratch of the expansion is unused. Unit tests pin the capacities, as for `enumerate`.

The budget bounds the time of a search call (GRP-35 and Section 7.14 give the bound). The sort of the colourings of each colour count is not counted in steps. It compares stored tables, so it reads at most $T \log_2 M$ entries, about $6.7 \times 10^7$ at the maxima. This closes, for the search, the residual risk on time that this section states for `enumerate`.

GRP-21. The crate MUST check every arithmetic operation on a size that comes from input, MUST contain no `unsafe` code, and MUST use no recursion, because an overflow gives a wrong table without an error, and the depth of a recursion over words or cosets is set by the input.

The time of a run of `enumerate` has no bound from these limits, and no limit of the crate bounds it. A coset limit bounds the cosets defined and not the work for each coset, and the sources give no computable bound on the work (Section 7.9). The residual risk is a caller that passes long relators with a high limit. In a release build of the crate, the time of a run grew about as $N (\alpha + \beta E)$, for $N$ cosets defined and $E$ expanded relator letters, with $\beta$ of a few nanoseconds. The slowest input measured at `CosetLimit::MAX`, a group of order $2^{20}$ with 64516 expanded letters that finishes within every limit, took several minutes. At `CosetLimit::DEFAULT`, the slowest input found, 2 generators with 16 random relators of 4096 letters, reached the limit in about 12 s.

The design therefore recommends that a caller that takes untrusted input keep the default limit. `escher-cli` cannot reach such an input, because the Conway presentation of a spherical signature of order $N$ expands to at most $2N + 11$ letters and the side presentations are short, and it does not enumerate a group whose order is above the limit (CLI-39 of Section 5.7 of `docs/design/cli.md`). A work budget in letter steps is the recorded alternative (item 1 of Section 14.3).

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library.
- Untrusted input and file paths: the values of the public types, a text read as a colouring id, and the side indices of `colour_of` are the only input. `colour_of` returns `None` for a side index without a generator and never panics. The crate reads and writes no file and keeps no state between calls.
- Build scripts: the crate has none. The procedural macro of `thiserror` runs at build time on the source of this crate only.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`.

Attacks on the build machine, the repository, or the caller are out of scope, because this crate builds nothing and prints nothing. The display of names and words is the caller's, and the names are printable ASCII by GRP-5.

A further residual risk is that a step that only code states is wrong in a way that the closing pass does not reveal. The closing pass makes every returned table closed, so such a fault can only make a run define more cosets or reach its limit, under the sourced statement of Section 7.7 and the unverified statements of `UNV-19`.

## 16. Operational Considerations

The crate declares no Cargo feature, reads no environment variable, and has no build script. It is pure Rust with no platform-specific code, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log and starts no thread. The defaults of its options are `CosetLimit::DEFAULT`, 16384 cosets, `IndexLimit::DEFAULT`, 12 colours, and `SearchBudget::DEFAULT`, $2^{28}$ steps.

Every failure is an `Error` with an `ErrorKind`:

| Failure | Kind | Recovery |
| --- | --- | --- |
| Empty relator base, exponent 0, bad or repeated name, or a relator letter outside the generators | `InvalidPresentation` | Correct the presentation |
| A subgroup generator, or a word of the stabilizer of `free_classes_of_index`, with a letter outside the generators | `InvalidWord` | Correct the words |
| An input over a limit, or a coset limit out of range | `LimitExceeded` | Reduce the input below the limit of Section 15 |
| The run defined `limit` cosets and needed more | `CosetLimitReached` | Try a larger limit. The error says nothing about the index (GRP-17) |
| A signature with an infinite order | `UnsupportedSignature` | None. The design builds no presentation for it |
| A text that is not a colouring id | `InvalidColouringId` | Give an id of the form of Section 7.15 |
| A search that stops at the budget or a stored limit | No error. The result is complete to a lower colour count, or, from `colourings_of_index` or `free_classes_of_index`, not complete (GRP-35) | Lower the index limit, or raise the budget. The result says nothing about the colour counts above `complete_to()`, and an incomplete `ColouringsOfIndex` says nothing about any colouring |

## 17. References

### 17.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.
- [RFC5234] Crocker, D., Ed., and P. Overell, "Augmented BNF for Syntax Specifications: ABNF", STD 68, RFC 5234, January 2008, <https://www.rfc-editor.org/info/rfc5234>.

### 17.2. Informative References

The tags of the mathematical and software sources resolve to the reference sections of the research documents:

- [ACE] Section 15 of `docs/research/colourings.md`.
- [BROWN2011] Section 15 of `docs/research/colourings.md`.
- [BULATOV2010] Section 15 of `docs/research/colourings.md`.
- [FRETTLOH2008] Section 15 of `docs/research/colourings.md`.
- [LOWINDEX] Section 15 of `docs/research/colourings.md`.
- [SINGERMAN2014] Section 15 of `docs/research/colourings.md`.
- [SYMPY-FPG] Section 15 of `docs/research/colourings.md`.
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
| GRP-20 | The memory of a call stays within the stated bound, with capped capacities for the table, the representatives, and the queue, and exact sizes for the relabelling | 15 |
| GRP-21 | Arithmetic on input sizes is checked, and the crate has no `unsafe` code and no recursion | 15 |
| GRP-22 | `Error` follows ERR-1, with the operation and the texts of Section 8.2 | 8.1 |
| GRP-23 | The dependencies are `escher-signature`, `thiserror`, and the dev-dependency `proptest` | 12 |
| GRP-24 | The worked examples and "Closing pass idle" stay while `UNV-19` is open | 10.1 |
| GRP-25 | The conjugate lists hold each distinct cyclic conjugate once | 7.2 |
| GRP-26 | A one-letter relator fixes its entries at coset 0 and at each new coset | 7.5 |
| GRP-27 | An overflowing stack is emptied before a lookahead that keeps its pushes, and the cap is a crate-private parameter | 7.6 |
| GRP-28 | Withdrawn on 2026-11-19. | 10.1 |
| GRP-29 | The search counts one step for each table read and two for each entry that the expansion compares, and stops at the first read beyond the budget, or fewer than $2kc$ steps past it in a comparison or a validity test of the expansion | 7.12 |
| GRP-30 | The search finds one class table for each class and expands each class into its subgroups | 7.10 |
| GRP-31 | Deductions use every relator, are pushed, are selected by letter, and a contradiction abandons the candidate | 7.11 |
| GRP-32 | The first-in-class test stops at an undefined entry and never compares it as 0 | 7.11 |
| GRP-33 | Every complete table passes a scan of every relator at every coset before it is passed on | 7.11 |
| GRP-34 | A class table is relabelled from each base coset, and each distinct result is a colouring of its class | 7.13 |
| GRP-35 | One run first, then runs by index when it stops, a result complete to a colour count, and no claim above it | 7.14 |
| GRP-36 | A call stores at most 65536 colourings and $2^{22}$ table entries, and reaching either is a stop at a limit | 7.12 |
| GRP-37 | Ids rank the colourings of one colour count of the set in lexicographic order of their tables, in bijective base 26, only for a complete colour count | 7.15 |
| GRP-38 | `colour_of` traces the inverse columns of the sides in reverse order from coset 0 of its own table (`UNV-17`) | 7.16 |
| GRP-39 | The result of a search depends on the presentation, the set, the limit, and the budget alone, and its ids on neither limit nor budget | 7.17 |
| GRP-40 | The memory of a search call stays within the stated bound | 15 |
| GRP-41 | The tests of the search and of the colour rule stay while `UNV-19` or `UNV-17` is open | 10.1 |
| GRP-42 | A colouring is valid exactly when no generator fixes coset 0 of its table, and `contained_generator` returns the least one that does (`UNV-35`) | 7.18 |
| GRP-43 | With `Valid`, the expansion drops each relabelling that is not valid before it stores or counts it, the runs stay unchanged, a class id names a kept colouring, and counts are compared within one set | 7.13 |
| GRP-44 | With `ValidClasses`, each class with a valid subgroup keeps exactly its valid relabelling with the least base coset, and the expansion may stop there | 7.13 |
| GRP-45 | `free_classes_of_index` runs one bound, keeps each class table in which no word fixes a coset, ranks the kept tables among themselves, and gives each its own id as class | 7.19 |
| GRP-46 | `free_classes_of_index` checks its words against the limits of the subgroup generators and the generators of the presentation, in order, before any allocation | 7.19 |
