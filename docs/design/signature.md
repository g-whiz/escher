# Signature Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-22 |
| Applies to | `crates/escher-signature` |

## Abstract

This document specifies the crate `escher-signature`, which turns the text of an orbifold signature into a validated value in canonical form and classifies it. It defines the public types and operations, the grammar of the accepted text, the parsing algorithm with the byte span of every error, the checks that reject malformed and bad signatures, the canonical form as a numbered algorithm, the exact computation of the Euler characteristic, and the classification into spherical, Euclidean, and hyperbolic signatures with the derived quantities. It also states the error type, the resource limits, the tests, the dependencies, the unverified statements that the code relies on, and the choices that no other document dictates.

## 1. Introduction

### 1.1. Problem

A signature arrives as text from a user. The same group has many spellings, because the cone points, the boundaries, and the corners of a boundary can be permuted, rotated, and reversed, a handle can be traded for two crosscaps, and an order of 1 can be written or omitted (Section 4.1 of the reference). No primary source defines a text form for an order of 10 or more or for an infinite order, or a canonical string (Sections 3.4, 2.4, and 5 of the reference). Every later crate keys its results on the signature, so two spellings of one group that give two values produce two sets of parameter names and two colouring lists for one group.

### 1.2. Purpose

`escher-signature` is the one place where text becomes a `Signature`. It accepts only text whose meaning is unambiguous, rejects every other input with an error kind and a byte span, stores the signature in one canonical form, and computes its Euler characteristic exactly. Section 3 of the architecture places the crate in the workspace as a root with no internal dependency.

### 1.3. Audience

The primary readers are the agent that implements the crate and the reviewer who checks it against this document. The secondary readers are the authors of the design documents of `escher-group`, `escher-domain`, and `escher-cli`, which consume a `Signature`.

### 1.4. Scope

This document covers the public surface of the crate, the input grammar, parsing, validation, the canonical form, the Euler characteristic, the classification, the error type, the resource limits, the tests, and the dependencies of the crate.

### 1.5. Non-Goals

These topics are owned elsewhere:

- The command line, the information text, the diagnostics with their caret line, the exit status, and the snapshot tests of `escher info` are specified in `docs/design/cli.md`.
- The Conway presentation and the colourings belong to `escher-group`.
- The fundamental domain and the names of its shape parameters belong to `escher-domain` (Section 5.6 of the architecture).
- Names of the groups in other notations are omitted (`DEC-20`).
- The construction of the general non-compact signatures is a roadmap workstream (`DEC-19`). This crate parses and classifies them.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms of Section 2.2 of `docs/writing-documentation.md`, the terms of Section 2.2 of `docs/research/orbifold-notation.md`, such as handle, crosscap, mirror boundary, cone point, corner point, defect, bad orbifold, and frieze group, and the terms signature, canonical signature, and general non-compact signature of Section 2.1 of `docs/design/architecture.md`. It cites `docs/research/orbifold-notation.md` as the reference, `docs/design/architecture.md` as the architecture, `docs/design/poc-plan.md` as the plan, and `docs/rust-coding-standards.md` as the coding standard. A rule of Section 4.1 of the reference is cited by its bare number, such as rule 5, and a rule of Section 4.2 of this document is cited as a reading rule, such as reading rule 5.

It adds these terms:

- feature: One handle, crosscap, mirror boundary, cone point, or corner point of a signature.
- order: The integer $n \geq 2$ or the value $\infty$ of a cone point or a corner point.
- order token: The text of one order in the input: one digit, a parenthesized digit string, `(inf)`, or the infinity sign U+221E.
- canonical string: The text that `Display` writes for a `Signature` (Section 7.3).
- trivial signature: The signature with no feature, written `1`, which names the trivial group (Section 3.5 of the reference).
- compact: Having no infinite order (Section 7.1 of the reference).
- size: The number $2h + c + b + k + l$ of a signature with $h$ handles, $c$ crosscaps, $b$ boundaries, $k$ cone points, and $l$ corners, counted after every order of 1 is dropped.
- span: A byte range `start..end` of the input text, on character boundaries.
- position: The state of the scanner relative to the boundaries read so far, one of before every boundary, in a boundary, and after a boundary (Section 5.1).

## 3. Public Surface

### 3.1. Items

SIG-1. The crate MUST export exactly the items of this section from its root, each through one path, and MUST give no type a public field, because a public field or a second constructor lets a caller build a `Signature` that breaks the invariants of Section 3.2, and every later crate relies on them (API-10 of the coding standard).

This listing shows each public item with its signature. Bodies are omitted:

```rust
pub struct Signature { /* private */ }
// derives Clone, Debug, PartialEq, Eq, Hash
impl core::str::FromStr for Signature { type Err = Error; }
impl core::fmt::Display for Signature {}

impl Signature {
    #[must_use] pub fn handles(&self) -> u32;
    #[must_use] pub fn cone_orders(&self) -> &[Order];
    #[must_use] pub fn boundaries(&self) -> &[Boundary];
    #[must_use] pub fn crosscaps(&self) -> u32;
    #[must_use] pub fn euler_characteristic(&self) -> Rational;
    #[must_use] pub fn geometry(&self) -> Geometry;
    #[must_use] pub fn is_compact(&self) -> bool;
    #[must_use] pub fn is_frieze(&self) -> bool;
    #[must_use] pub fn is_general_non_compact(&self) -> bool;
    #[must_use] pub fn group_order(&self) -> Option<core::num::NonZeroU64>;
    #[must_use] pub fn area_over_pi(&self) -> Option<Rational>;
    #[must_use] pub fn deformation_dimension(&self) -> Option<u32>;
}

pub struct Boundary { /* private */ }
// derives Clone, Debug, PartialEq, Eq, Hash
impl Boundary {
    #[must_use] pub fn corner_orders(&self) -> &[Order];
}

pub struct Order { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord
impl Order {
    #[must_use] pub fn finite(self) -> Option<u32>;
    #[must_use] pub fn is_infinite(self) -> bool;
}
impl core::fmt::Display for Order {}

pub struct Rational { /* private */ }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash
impl Rational {
    #[must_use] pub fn numerator(self) -> i64;
    #[must_use] pub fn denominator(self) -> core::num::NonZeroU64;
}
impl core::fmt::Display for Rational {}

#[expect(clippy::exhaustive_enums, reason = "the three geometries of constant curvature are a closed set")]
pub enum Geometry { Spherical, Euclidean, Hyperbolic }
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub struct Error { /* private */ }
// derives Clone, Debug, PartialEq, Eq, and std::error::Error and Display through thiserror
impl Error {
    #[must_use] pub fn kind(&self) -> ErrorKind;
    #[must_use] pub fn span(&self) -> core::ops::Range<usize>;
}

#[non_exhaustive]
pub enum ErrorKind {
    Empty,
    UnexpectedCharacter,
    UnclosedParenthesis,
    ZeroOrder,
    MultiDigitOrder,
    AmbiguousOrder,
    LimitExceeded,
    BadOrbifold,
    PositiveNonCompact,
    Overflow,
}
// derives Clone, Copy, Debug, PartialEq, Eq, Hash

pub type Result<T> = core::result::Result<T, Error>;
```

### 3.2. Meaning and Invariants

Each item guarantees its invariant by construction, so no caller checks it:

| Item | Meaning | Invariant |
| --- | --- | --- |
| `Signature` | A valid signature in canonical form | Every check of Section 6 passed, the computations of Sections 8.2, 9.3, and 9.5 did not overflow, and the features are those of Section 7.2. Two values are equal exactly when their canonical strings are equal |
| `FromStr` | The only constructor. It parses, validates, and canonicalizes | Returns an `Error` for every input that is not a valid signature, and never panics |
| `Display` for `Signature` | The canonical string of Section 7.3 | The string parses back to an equal `Signature` |
| `handles()` | The number of handles | 0 whenever `crosscaps()` is greater than 0 (Section 7.2, step 2) |
| `cone_orders()` | The cone point orders | Sorted in descending order, with no order of 1 |
| `boundaries()` | The mirror boundaries | Each in the form of Section 7.2, step 4, and the list in descending order |
| `crosscaps()` | The number of crosscaps | The crosscaps of the input plus twice its handles when the input has a crosscap, and 0 otherwise |
| `Boundary::corner_orders()` | The corner orders of one boundary, in cyclic order | May be empty, with no order of 1 |
| `Order` | A cone or corner order | Finite with $2 \leq n \leq$ `u32::MAX`, or infinite (Section 3.1 of the reference, `DEC-25`) |
| `Order::finite()` | The order when it is finite | `Some(n)` has $n \geq 2$ |
| `Ord` for `Order` | The total order of the canonical form | Finite orders by value, and the infinite order above every finite order (Section 7.1) |
| `Display` for `Order` | The text of one order | One digit below 10, `(n)` from 10, and `(inf)` when infinite |
| `Rational` | An exact rational number | The denominator is at least 1, the numerator and the denominator have no common factor, and zero is `0/1` |
| `Display` for `Rational` | `n/d`, or `n` when the denominator is 1 | For example `-1/84`, `0`, and `2` |
| `Geometry` | The result of the classification (Section 3.4 of the architecture, `DEC-26`) | A closed set, so it is exhaustive under a lint exception (API-9 of the coding standard) |
| `Error` | Why the text is not a valid signature | `span()` lies within `0..=text.len()`, on character boundaries |
| `ErrorKind` | What a caller matches on | `Copy` and `#[non_exhaustive]` (ERR-1 of the coding standard) |

`Order` is an opaque struct over a private enum with the variants `Finite(u32)` and `Infinite`, in that order, so that the derived `Ord` puts the infinite order above every finite order. A public enum would let any crate write `Finite(1)`.

Every public type is `Send`, `Sync`, and `'static`, and a compile-time assertion test covers each (CONC-1 of the coding standard). `Geometry` has no `Display`, because the command-line text is unstable (`DEC-23`) and belongs to `escher-cli`.

### 3.3. Source Layout

The crate has these private modules, each a file named after the module:

- `src/lib.rs`: The crate-level documentation, with a `# Features` section that states that the crate has no Cargo feature, the re-exports, and the unit test "Auto traits" of Section 12.1.
- `src/error.rs`: `Error`, `ErrorKind`, `Result`, and the private detail enum.
- `src/order.rs`: `Order`.
- `src/rational.rs`: `Rational` and the private greatest common divisor.
- `src/parse.rs`: The scanner of Section 5, which returns the raw features.
- `src/signature.rs`: `Signature`, `Boundary`, the constructor, the accessors, and `Display`.
- `src/canonical.rs`: The algorithm of Section 7.2.
- `src/classify.rs`: Sections 8 and 9.

## 4. Input Grammar

### 4.1. Formal Grammar

SIG-2. `Signature::from_str` MUST accept exactly the texts that the grammar of this section and the reading rules of Section 4.2 accept, and MUST reject every other text, without a panic, with the error kind and span that Sections 5 and 6 give, because a text that is accepted silently becomes behaviour that callers depend on, and a panic on input stops the caller's whole program (ERR-5 of the coding standard).

This grammar in Augmented Backus-Naur Form (ABNF) [RFC5234] is the authoritative definition of the accepted text. Its terminals are the Unicode code points of the text, and spans count the bytes of its UTF-8 encoding:

```abnf
signature   = 1*item
item        = handle / crosscap / mirror / order
handle      = %x6F / %x4F                  ; o or O
crosscap    = %x78 / %x58                  ; x or X
mirror      = %x2A                         ; *
order       = digit-order / paren-order / infinity
digit-order = DIGIT                        ; one digit is one order
paren-order = %x28 1*DIGIT %x29            ; ( digits )
infinity    = %x28 %x69.6E.66 %x29         ; (inf), lower case only
            / %x221E                       ; the infinity sign
```

The grammar uses numeric terminals throughout, because a quoted string in ABNF matches without regard to case, and `(INF)` is not accepted.

### 4.2. Reading Rules

The grammar does not express these rules. Each reading rule names its source, or states that it is a choice of this design:

1. An order before the first `*` is a cone point (Section 3.2 of the reference).
2. An order after a `*`, with no `o` or `x` between that `*` and the order, is a corner of that boundary (Section 3.2 of the reference).
3. An order of 1 is accepted and dropped wherever it stands, except as reading rule 5 states, and reading rule 4 does not apply to it (rule 6 and Section 3.5 of the reference, `DEC-8`).
4. An order other than 1 that follows a `*` with an `o` or an `x` between that `*` and the order is ambiguous, and the text is rejected (`DEC-8`). The meaning of "ambiguous" is a choice of this design.
5. A digit `1` that is not in parentheses and is directly followed by another digit is rejected. This is a choice of this design.
6. `o` and `x` may stand anywhere, and `O` and `X` are accepted with the same meaning (`DEC-8`, Section 11.1 of the reference).
7. An order of 10 or more needs parentheses, because one digit is one order. A parenthesized order may also be below 10, and leading zeros are read by value, so `(07)` is 7. Both are choices of this design.
8. An order of 0 is rejected, because an order is an integer of at least 2 (Section 3.1 of the reference).
9. A text in which every order is 1 and that has no other feature is the trivial signature (Section 3.5 of the reference, `DEC-8`).
10. The empty text is rejected (`DEC-8`).
11. Whitespace is rejected everywhere, including at either end. This is a choice of this design.
12. Of the characters outside ASCII, only the infinity sign U+221E is accepted, and only on its own, not inside parentheses (`DEC-8`).

SIG-3. The parser MUST drop an order of 1 before it applies reading rule 4, because an order of 1 means the same under both readings that reading rule 4 guards against, and otherwise rule 6 of the reference, which lets an order of 1 be written anywhere, would turn the valid `*2x` into the rejected `*2x1`.

SIG-4. The parser MUST reject an order that reading rule 4 calls ambiguous, because the two readings name different groups and the caller cannot see which one was taken.

SIG-5. The parser MUST reject a digit `1` that reading rule 5 describes, with the kind `MultiDigitOrder`, because [CH] prints orders of 10 or more without a delimiter (Section 3.4 of the reference), so `2*12` has the reading `2*(12)`, with group order 48, and the reading `2*2`, with group order 8 (Section 8.2 of the reference), and a silent choice of one names the wrong group.

Rationale for the choices in these rules: reading rule 4 exists because SymmHub reads an order after an `o` or an `x` as a cone point even after a boundary (Section 11.1 of the reference), while the layout of Section 3.2 of the reference reads an order after a `*` as a corner of that boundary. So `*2x3` has two readings, and rejecting it is the sounder option. Reading rule 11 exists because a space has two candidate meanings. It could be insignificant, or it could be the delimiter of the tables of [CBG], such as `*2 3 12` (Section 3.4 of the reference), and skipping it would drop the only delimiter between `3` and `12`.

Reading rule 5 has these consequences:

- `11` and `*11` are rejected, although [CBG] writes `11` for the trivial group and [CH] writes `*11` for `*` (Section 3.5 of the reference). Under the other reading they are `(11)` and `*(11)`, so they are ambiguous too.
- `1`, `*1`, `1*`, `221`, `2*1`, and `(1)` are accepted, with the order of 1 dropped.
- The rule fires on every `1` that a digit follows. It therefore detects an unparenthesized order from 10 to 19 and from 100 to 199, which covers every spelling of Table AI of [CH] that Section 3.4 of the reference lists. It also rejects `213`, which could mean 21 and 3. It cannot detect other multi-digit orders written without parentheses, so `23` always reads as two cone points. The canonical string reveals the reading that the parser took.

### 4.3. Examples

These texts are accepted. Each shows the canonical string that Section 7 gives:

| Text | Canonical string | Reason |
| --- | --- | --- |
| `2o3` | `o32` | Reading rules 1 and 6 |
| `x*2` | `*2x` | Reading rule 6 |
| `*2x*3` | `*3*2x` | The second `*` opens a new boundary |
| `*2x` and `*2x1` | `*2x` | Reading rule 3 |
| `XO` | `xxx` | Reading rule 6, and step 2 of Section 7.2 |
| `2*(12)` | `2*(12)` | Reading rule 7 |
| `*22(inf)` and `*22∞` | `*(inf)22` | Reading rule 12, and Section 7.1 |
| `1`, `(1)`, and `(1)(1)` | `1` | Reading rule 9 |

These texts are rejected. Section 5.5 gives the kind and the span of each:

| Text | Reason |
| --- | --- |
| `*2x3` and `*o2` | Reading rule 4 |
| `2*12`, `11`, and `*11` | Reading rule 5 |
| `(INF)`, `(in)`, and `(∞)` | Reading rule 12 and the grammar |
| `*7 32` | Reading rule 11 |
| `0` and `(00)` | Reading rule 8 |
| The empty text | Reading rule 10 |

## 5. Parsing

### 5.1. Scanner

The parser is a hand-written scanner over `text.char_indices()` with one character of lookahead (Section 9.1 of `docs/research/rust-ecosystem.md`). It reads no slice by a computed index (TYPE-9 of the coding standard) and uses checked arithmetic for every count and value (TYPE-5).

Its private state is the counts of handles and crosscaps, the list of cone orders, the list of boundaries with their corner orders, the size, and the position. The position is an enum with the variants `BeforeBoundary`, `InBoundary`, and `AfterBoundary`.

`from_str(text)` runs these steps:

1. If `text.len()` exceeds 1024, return `LimitExceeded` with the span `0..text.len()`.
2. If `text` is empty, return `Empty` with the span `0..0`.
3. Read items until the end of the text:
   - `o` or `O`: add a handle, add 2 to the size, and change `InBoundary` to `AfterBoundary`.
   - `x` or `X`: add a crosscap, add 1 to the size, and change `InBoundary` to `AfterBoundary`.
   - `*`: push a new empty boundary, add 1 to the size, and set the position to `InBoundary`.
   - A digit: an order token of one character, handled as Section 5.3 states. When the digit is `1` and the next character is an ASCII digit, return `MultiDigitOrder` with the span of those two digits, and do not apply Section 5.3.
   - U+221E: an infinite order token of three bytes, handled as Section 5.3 states.
   - `(`: a parenthesized token, read as Section 5.2 states.
   - Any other character: return `UnexpectedCharacter` with the span of that character.
4. Canonicalize as Section 7.2 states.
5. Compute $\chi$ and $2\chi$ on the canonical features as Section 8.2 states. On a failure of Section 8.3, return `Overflow` with the span `0..text.len()`.
6. Run the checks of Section 6.
7. For a spherical signature, compute the group order as Section 9.3 states. When the division leaves a remainder or the quotient does not fit `NonZeroU64`, return `Overflow` with the span `0..text.len()`.
8. For a compact hyperbolic signature, compute the deformation dimension as Section 9.5 states. When it does not fit `u32`, return `Overflow` with the span `0..text.len()`.

After each change to the size, the scanner returns `LimitExceeded` with the span of the current character or token when the size exceeds 64.

### 5.2. Parenthesized Tokens

A parenthesized token starts at the byte `a` of its `(`, and the scanner reads it one character at a time in these steps:

1. If the next character is an ASCII digit, read the whole run of ASCII digits. Accumulate the value in `u32` with `checked_mul(10)` and `checked_add`. On overflow, set a flag and keep reading digits.
2. Otherwise, if the next character is `i`, require `n` and then `f`. The token is then infinite.
3. Require `)` after the digits or after `inf`.
4. When a character differs from the one that a step requires, return `UnexpectedCharacter` with the span of that character. This covers `()` at the `)`, `(a` at the `a`, `(12a)` at the `a`, `(in)` at the `)`, `(INF)` at the `I`, and `(∞)` at the `∞`.
5. When the text ends before the token is complete, return `UnclosedParenthesis` with the span `a..text.len()`.

### 5.3. Order Tokens

The scanner handles an order token with the span `s` in this order. An infinite token skips steps 1 to 3:

1. If the value overflowed `u32`, return `LimitExceeded` with the span `s`.
2. If the value is 0, return `ZeroOrder` with the span `s`.
3. If the value is 1, drop the token and continue (SIG-3).
4. If the position is `AfterBoundary`, return `AmbiguousOrder` with the span `s`.
5. Add 1 to the size, with the limit check of Section 5.1.
6. Push the order to the cone orders when the position is `BeforeBoundary`, or to the last boundary when it is `InBoundary`.

### 5.4. Order of Errors

SIG-6. The parser MUST return the first error in text order, and only after the scan has finished without an error MUST it canonicalize, compute $\chi$, run the checks of Section 6, compute the group order, and compute the deformation dimension, in that order, because a caller draws a caret under the span, and a rule that picks another error would point the caret at a later part of the text than the first problem, and would make the result depend on the order of the code.

After a scan without an error, the errors arise in this order, each with the whole text as its span:

1. `Overflow` from the computation of $\chi$ and $2\chi$ (Section 8.3).
2. `BadOrbifold` (Section 6, check 1).
3. `PositiveNonCompact` (Section 6, check 2).
4. `Overflow` from the group order (Section 9.3).
5. `Overflow` from the deformation dimension (Section 9.5).

The characteristic is computed on the canonical features, so equivalent signatures run the same computation and succeed or fail together (Section 8.3).

### 5.5. Error Examples

These inputs give these errors. The span of every error after the scan is the whole text:

| Input | Kind | Span |
| --- | --- | --- |
| The empty text | `Empty` | `0..0` |
| `*7q2` | `UnexpectedCharacter` | `2..3` |
| `*7 32` | `UnexpectedCharacter` | `2..3` |
| `()` | `UnexpectedCharacter` | `1..2` |
| `(INF)` | `UnexpectedCharacter` | `1..2` |
| `(in)` | `UnexpectedCharacter` | `3..4` |
| `(∞)` | `UnexpectedCharacter` | `1..4` |
| `(12` | `UnclosedParenthesis` | `0..3` |
| `0` | `ZeroOrder` | `0..1` |
| `(00)` | `ZeroOrder` | `0..4` |
| `11` | `MultiDigitOrder` | `0..2` |
| `*11` | `MultiDigitOrder` | `1..3` |
| `2*12` | `MultiDigitOrder` | `2..4` |
| `*2x3` | `AmbiguousOrder` | `3..4` |
| `*o2` | `AmbiguousOrder` | `2..3` |
| `(4294967296)` | `LimitExceeded` | `0..12` |
| 1025 bytes | `LimitExceeded` | `0..1025` |
| `23` | `BadOrbifold` | `0..2` |
| `2(inf)` | `PositiveNonCompact` | `0..6` |
| `∞` | `PositiveNonCompact` | `0..3` |

## 6. Validation

The crate runs these checks, in this order, on the canonical features of Section 7.2, after Section 8.2 has computed $\chi$:

1. Bad orbifold. The check applies only when no order is infinite, because Section 7.2 of the reference states the four families for integer orders. The signature is bad when it has no handle and no crosscap and one of these holds:
   - No boundary and exactly one cone point, `m` with $m > 1$.
   - No boundary and exactly two cone points of different orders, `mn` with $m \neq n$.
   - No cone point, exactly one boundary, and exactly one corner, `*m` with $m > 1$.
   - No cone point, exactly one boundary, and exactly two corners of different orders, `*mn` with $m \neq n$.
2. An infinite order with a positive Euler characteristic, such as `(inf)`, `2(inf)`, `*(inf)`, and `*2(inf)`.

SIG-7. The crate MUST reject a bad orbifold with `BadOrbifold`, because a bad orbifold corresponds to no group (Section 7.2 of the reference, `DEC-24`), and a later crate that receives one has nothing to build.

SIG-8. The crate MUST reject a signature with an infinite order and $\chi > 0$ with `PositiveNonCompact`, because no source gives its geometry (item 15 of Section 12 of the reference), and any classification of it would be invented (`DEC-10`).

A signature with an infinite order whose shape is one of the four families gets `PositiveNonCompact` and not `BadOrbifold`, because no source calls it a bad orbifold. Every compact signature with $\chi > 0$ that passes check 1 is spherical, because the four families are the only signatures that do not correspond to groups (Sections 7.1 and 7.2 of the reference).

## 7. Canonical Form

No primary source defines a canonical string (Section 5 of the reference). The form of this section is a project convention under `UNV-20`, and `DEC-9` sets its outline.

### 7.1. Order of Orders

An infinite order compares above every finite order, and finite orders compare by value. So the canonical string of the frieze written `*22(inf)` is `*(inf)22`. [CBG] advises listing larger numbers first and writes `∞32` and `*∞32` (Sections 5 and 7.1 of the reference), while the frieze table of Section 8.3 of the reference writes the form `22∞` of the series. Section 15.3 lists this order as a point for the owner.

The canonical strings, the parameter names that `escher-domain` derives from the canonical signature (Section 5.6 of the architecture), and the snapshots of `docs/design/cli.md` all depend on this order, and each changes if the order changes.

### 7.2. Algorithm

SIG-9. The constructor MUST store the features in the form that this algorithm produces, because the derived `PartialEq` and `Hash` then compare groups, and two equivalent spellings that stored different forms would give two sets of parameter names for one group (Section 5.6 of the architecture).

The input is the counts and lists of the scanner after a scan without an error. Canonicalization runs before the computation of $\chi$ and the checks of Section 6, so both see the canonical features. Each step names the rule of Section 4.1 of the reference that it applies:

1. Rule 6. The parser has already dropped every order of 1 (Section 3.5 of the reference).
2. Rule 5. If the crosscap count $c$ is greater than 0, set $c$ to $c + 2h$ and the handle count $h$ to 0. So `ox` and `xxx` both give `xxx`, `oox` gives `xxxxx`, and `o` alone stays `o`. This is the rule of `DEC-9`, which is a choice of this design. The new count $c + 2h$ is computed with checked arithmetic. A count that does not fit `u32` is a `LimitExceeded` error for the size, with the span of the whole text. The size limit of Section 16 makes such a count impossible, so no input reaches this error.
3. Rule 1. Sort the cone orders in descending order, with the order of Section 7.1.
4. Rules 3 and 4. For a sequence $s$ of corner orders, its largest rotation is the lexicographically greatest of its rotations, and it is $s$ itself when $s$ is empty. Sequences compare element by element with the order of Section 7.1, and a proper prefix is smaller than the longer sequence, which is the order of Rust slices of `Order`.
   - If $c > 0$, replace each boundary $s$ by the greater of the largest rotation of $s$ and the largest rotation of the reverse of $s$, then sort the boundaries in descending order. Rule 4 allows the reversal of a single boundary when a crosscap is present.
   - If $c = 0$, let $A$ be the list of the largest rotations of the boundaries, sorted in descending order, and $B$ the list of the largest rotations of the reversed boundaries, sorted in descending order. Keep the greater of $A$ and $B$. The two lists have the same length, and they compare element by element, each element a sequence compared as above. Rule 4 allows here only the reversal of every boundary at once. Tegula makes the same comparison (Section 11.2 of the reference).
5. Rule 2. The sort of step 4 puts the boundaries in descending order.

Ties need no rule, because two rotations that compare equal are the same sequence, and two lists that compare equal are the same list. The largest rotation is found by comparing every rotation, which costs at most 4096 element comparisons at the size limit of 64, so no faster algorithm is needed (SIMP-4 of the coding standard).

These examples follow from the steps. The tests of Section 12 assert each of them:

| Input | Canonical string | Rule |
| --- | --- | --- |
| `*236` | `*632` | 3, and 4 for every boundary |
| `*2345` | `*5432` | 3, and 4 for every boundary |
| `ox` | `xxx` | 5 |
| `oox` | `xxxxx` | 5 |
| `o` | `o` | 5 does not apply without a crosscap |
| `*2345*6432x` and `*2345*2346x` | `*6432*5432x` for both | 4 for a single boundary with a crosscap |
| `*2345*6432` | `*6432*5234` | 4 only for every boundary at once |
| `*2345*2346` | `*6432*5432` | 4 only for every boundary at once |
| `*1` and `1*` | `*` | 6 |
| `**2` | `*2*` | 2 |
| `*23(inf)` | `*(inf)32` | 3 and 4, with Section 7.1 |

### 7.3. Output String

SIG-10. `Display` MUST write the canonical string of this section, and that string MUST parse back to an equal `Signature`, because `escher-cli` prints it and later crates and snapshots are keyed on it, so a string that does not round-trip makes a printed signature unusable as input.

`Display` writes the features in the order of [CH] (Section 3.2 of the reference):

1. `o` once for each handle.
2. Each cone order.
3. For each boundary, `*` and then each corner order.
4. `x` once for each crosscap.
5. `1` when nothing else was written (`DEC-8`, Section 3.5 of the reference).

Each order is written as `Display` for `Order` writes it (Section 3.2). The longest canonical string has 64 features of 12 bytes each, `(4294967295)`, which is 768 bytes and below the length limit of 1024 bytes, so every canonical string parses again.

### 7.4. Properties

The canonical form has these properties. A property test and a bounded exhaustive test check them (Section 12.1). Neither test is a proof:

| Property | Statement | What it rests on |
| --- | --- | --- |
| Idempotent | Parsing the canonical string gives an equal `Signature` and the same string | The algorithm |
| Round trip | `signature.to_string().parse()` returns an equal `Signature` | The text conventions of `UNV-20`: parentheses, `(inf)`, and `1` |
| Invariant | Two texts within the length limit that differ by rules 1 to 6 give equal values, or errors of the same kind | The rules are primary (Section 4.1 of the reference). The invariance of this algorithm is tested |
| Sound | Equal canonical forms name the same group | Each step applies only rules 1 to 6 |
| Separating | Different canonical forms name different groups | The statement of [CBG] that the rules are complete (Section 4.1 of the reference), and the tested invariance |
| Infinite orders | An infinite order is permuted, rotated, and reversed like a finite one | No source states rules 1 to 4 for infinite orders (Section 15.1) |

Invariance holds only within the length limit, because rule 6 can lengthen a text past 1024 bytes, such as `*2` followed by 400 copies of `(1)`.

### 7.5. Consequences for Other Crates

A signature with a crosscap never reports a handle, so `oox` reports 0 handles and 5 crosscaps. The design document of `escher-domain` derives its parameter names from these counts (Section 5.6 of the architecture) and accounts for this.

## 8. Euler Characteristic

### 8.1. Rational Type

`Rational` holds a private `i64` numerator and a private `NonZeroU64` denominator, in lowest terms. A private constructor takes a numerator and a denominator as `i128`, reduces them, fixes the sign, and narrows them, and it fails when a reduced part does not fit. A private method returns the absolute value and fails when the numerator is `i64::MIN`. The type has no arithmetic operator and no `Ord`. A private Euclidean greatest common divisor on unsigned integers serves it.

The type is written in the crate, because the crate table of Section 3.1 of the architecture gives it only `thiserror` (DEP-2 of the coding standard), and a type from `num-rational` in the public surface would make that crate a public dependency (DEP-3).

### 8.2. Algorithm

The crate evaluates the defect rule of Section 6.1 of the reference over one common denominator, so it needs no rational addition. It runs on the canonical features of Section 7.2, after canonicalization and before the checks of Section 6. Let $h$, $c$, and $b$ be the numbers of handles, crosscaps, and boundaries, $k$ and $l$ the numbers of cone points and corners, finite or infinite, $A_i$ the finite cone orders, and $a_j$ the finite corner orders.

The finite orders are taken in one fixed sequence: the cone orders as `cone_orders()` lists them, then the corner orders of each boundary as `corner_orders()` lists them, boundary by boundary as `boundaries()` lists them. Every operation is checked and works in `i128`:

1. $L$ is the least common multiple of the finite orders, starting at 1, built one order $n$ at a time in the fixed sequence as $L \leftarrow (L / \gcd(L, n)) \cdot n$.
2. $D = 2L$.
3. The numerator is

   $$N = D\,(2 - 2h - c - b - k) - L\,l + \sum_{i} \frac{D}{A_i} + \sum_{j} \frac{L}{a_j}$$

   which is $D$ times the defect rule: a finite cone point of order $A$ has the defect $1 - 1/A$, a finite corner of order $a$ has $1/2 - 1/(2a)$, an infinite cone point has 1, and an infinite corner has $1/2$ (Section 6.1 of the reference). The crate sums it in this order: it computes $2 - 2h - c - b - k$, multiplies it by $D$, subtracts $L\,l$, adds each $D / A_i$ in the fixed sequence, and then adds each $L / a_j$ in the fixed sequence.
4. The Euler characteristic is $\chi = N / D$, reduced and narrowed to `Rational`.
5. Twice the characteristic is $2\chi = N / L$, reduced and narrowed to `Rational`, because `area_over_pi` needs it. This step multiplies nothing, so no intermediate value of it can overflow.
6. The crate checks that the absolute values of the reduced $\chi$ and $2\chi$ fit `Rational`, which holds exactly when neither reduced numerator is `i64::MIN`.

### 8.3. Overflow

SIG-11. The crate MUST compute $\chi$ exactly with checked arithmetic and return `Overflow` instead of a value whenever an operation does not fit, or when the reduced $\chi$, the reduced $2\chi$, or the absolute value of either does not fit `Rational`, because a wrong $\chi$ gives a wrong geometry without any error (`DEC-25`), and a value that fits at construction leaves `area_over_pi` without a failure path.

SIG-12. The crate MUST compute the intermediate values of Section 8.2 in `i128` and narrow only the reduced results, because narrowing an intermediate value rejects signatures whose characteristic fits `Rational`.

`Overflow` from the characteristic therefore arises only when an `i128` operation of Section 8.2 overflows, or when the reduced $\chi$, the reduced $2\chi$, or the absolute value of either does not fit `Rational`. The computation runs on the canonical features in the fixed sequence of Section 8.2, and equivalent signatures have the same canonical features (Section 7.2). So equivalent signatures run the same operations in the same order and succeed or fail together.

These signatures show where the boundary lies:

| Signature | Result |
| --- | --- |
| `(2147483647)(2147483649)2` | Accepted. $2L$ exceeds `i64::MAX`, while the reduced $\chi$ and $2\chi$ fit |
| `(41)(37)(32)(31)(29)(27)(25)(23)(19)(17)(13)(11)7` | Accepted, with $\chi = -2258057581140506927/219060189739591200$ |
| `(43)(41)(37)(32)(31)(29)(27)(25)(23)(19)(17)(13)(11)7` | `Overflow`, because the reduced numerator of $\chi$ does not fit `i64` |
| `(3037000493)(3037000499)x` | `Overflow`, because the reduced $\chi$ fits but the reduced numerator of $2\chi$ does not |

Rejecting a signature whose $2\chi$, or the absolute value of $\chi$ or $2\chi$, does not fit `Rational`, as the last row does for $2\chi$, is a choice of this design. Construction rejects such a signature with `Overflow`, and Section 9.4 states what this gives `area_over_pi`.

### 8.4. Recomputation

`Signature` does not store $\chi$, $2\chi$, the group order, or the deformation dimension, because TYPE-3 of the coding standard forbids storing a value that other fields determine. One private function recomputes them from the stored canonical features, with the computations of Sections 8.2, 9.3, and 9.5 that the constructor ran. It holds the only `expect` of the crate, in the form of ERR-6 of the coding standard: a lint exception whose reason says that the constructor accepted the signature only after the same computation on the same features succeeded, and a message that starts with `internal error:`. The public accessors call that function and contain no `expect` themselves.

## 9. Classification

### 9.1. Methods

SIG-13. Each classification method MUST return the value that this section defines, and MUST return `None` where it defines no value, because a value for a case that no source covers would present an invented number as a fact.

| Method | Definition | Source |
| --- | --- | --- |
| `euler_characteristic()` | $\chi$ by the defect rule (Section 8) | Section 6.1 of the reference |
| `geometry()` | `Spherical` when $\chi > 0$, `Euclidean` when $\chi = 0$, and `Hyperbolic` when $\chi < 0$ | Section 7.1 of the reference. For a signature with an infinite order, `DEC-10` and `UNV-21` |
| `is_compact()` | No order is infinite | Section 7.1 of the reference |
| `is_frieze()` | Some order is infinite and $\chi = 0$ | Sections 2.2, 7.1, and 8.3 of the reference |
| `is_general_non_compact()` | Section 9.2 | Section 2.1 of the architecture, `DEC-19` |
| `group_order()` | $2/\chi$ when spherical, else `None` (Section 9.3) | Section 7.3 of the reference |
| `area_over_pi()` | $2\lvert\chi\rvert$ when spherical or hyperbolic, else `None` (Section 9.4) | Section 7.3 of the reference |
| `deformation_dimension()` | Section 9.5, for a compact hyperbolic signature only, else `None` | Section 10.1 of the reference, and item 15 of Section 12 of the reference for the restriction to compact signatures |

The name `area_over_pi` carries the unit of its value, the area divided by $\pi$.

### 9.2. General Non-Compact Signatures

Section 2.1 of the architecture defines the term. `is_general_non_compact()` reads that definition literally and returns true exactly when all of these hold:

- Some order is infinite.
- $\chi < 0$, which excludes the 7 friezes, whose $\chi$ is 0.
- The signature is not a triangle group with ideal corners, which is the form `*pqr`: no handle, no crosscap, no cone point, and exactly one boundary with exactly three corners.
- The signature is not a rotation subgroup of such a group, which is the form `pqr`: no handle, no crosscap, no boundary, and exactly three cone points.

A corner or cone point of infinite order counts toward the three corners and the three cone points, so `*(inf)(inf)(inf)`, the triangle group whose three corners are all ideal, is not a general non-compact signature.

By this definition, `3*(inf)` and `(inf)(inf)(inf)(inf)` are general non-compact signatures, while `*23(inf)`, `(inf)32`, `*22(inf)`, `*(inf)(inf)(inf)`, and the compact `*732` are not.

### 9.3. Group Order

`group_order()` returns $2/\chi$, which Section 7.3 of the reference states is the group order. The constructor computes it for every spherical signature, after the checks of Section 6, from the reduced $\chi$ as twice its denominator divided by its numerator, in checked `i128` arithmetic. When the division leaves a remainder or the quotient does not fit `NonZeroU64`, the constructor returns `Overflow`, and the text of that error names both causes (Section 10.2). So `group_order()` has no failure path and returns `None` only for a signature that is not spherical. The tables and the property test "Any text" of Section 12.1 assert `Some` for each spherical signature that they parse. The result is a `NonZeroU64`, because a group order is at least 1 and `*22n` has the order $4n$ (Section 8.2 of the reference), which exceeds `u32` for large $n$.

### 9.4. Area

`area_over_pi()` returns $2\lvert\chi\rvert$ for every spherical and every hyperbolic signature, including a non-compact hyperbolic one. Section 7.3 of the reference states the formula for an elliptic or hyperbolic orbifold without recording whether it covers a non-compact one, so the value for a non-compact signature rests on `UNV-21`. Section 15.3 raises this with the owner.

The method has no failure path, because step 6 of Section 8.2 has checked that the absolute value of the reduced $2\chi$ fits `Rational`.

### 9.5. Deformation Dimension

For a compact hyperbolic signature, `deformation_dimension()` returns the dimension of the Teichmuller space of [TH], Corollary 13.3.7 (Section 10.1 of the reference), with $k$ cone points and $l$ corners:

$$\dim T(O) = -3\chi(X_O) + 2k + l = -6 + 6h + 3c + 3b + 2k + l$$

The second form reads $\chi(X_O)$ as $2 - 2h - c - b$, the part of the defect rule that comes from `o`, `x`, and `*` (Section 15.1). The method returns `None` for a Euclidean signature, because the counts of Section 10.2 of the reference are open (`UNV-10`), and for a non-compact signature (`UNV-21`). The plan states 0 for `*732` and 1 for `*3333` (Section 5.1 of the plan), and [CBG] states 13 for `4*235*74xx` (Section 10.1 of the reference).

The constructor computes the dimension of every compact hyperbolic signature with checked arithmetic, after the checks of Section 6, and returns `Overflow` when the value does not fit `u32`, which includes a negative value. So `deformation_dimension()` has no failure path. No cited source states that the value is never negative for a compact hyperbolic signature. Section 15.1 records this as an assumption, and the property test "Dimension sign" of Section 12.1 checks it.

## 10. Errors

### 10.1. Structure

SIG-14. The crate MUST implement `Error` as ERR-1 of the coding standard requires, as an opaque struct that carries the operation that failed, with `kind()` returning a `Copy`, `#[non_exhaustive]` `ErrorKind`, because a reviewer checks the error against that rule and callers match on the kind, never on the text.

`Error` has these private fields:

- `operation`: A `&'static str`, which is `"parse orbifold signature"` for every error of this crate.
- `detail`: A private enum with one variant for each cause, which carries the data that the text of Section 10.2 shows, such as the unexpected character, the ambiguous order, the limit that was exceeded, or the value that did not fit.
- `span`: The `core::ops::Range<usize>` of Section 5.

`kind()` computes the kind from `detail` with a `match` that has no wildcard arm, so the kind is not stored twice (TYPE-3 and TYPE-7 of the coding standard). `thiserror` derives `Display` and `std::error::Error` on `Error` and on the detail enum (Section 3.1 of the architecture). `Display` for `Error` writes `cannot `, the operation, `: `, and the text of the detail (SIG-15). No error of this crate has a cause, so `source()` returns `None` and the struct has no source field. No type from a dependency appears in the error (ERR-2).

The error stores no part of the input text beyond the one character or order that its text shows. The caller holds the text, and echoing untrusted text is a decision for the caller, which `docs/design/cli.md` makes for `escher-cli`.

### 10.2. Kinds and Text

SIG-15. `Display` for `Error` MUST be `cannot parse orbifold signature: ` followed by the text of the detail in this table, because `escher-cli` prints it as the diagnostic and its snapshots fix it.

| Kind | Detail carries | Text after the prefix |
| --- | --- | --- |
| `Empty` | Nothing | `the signature is empty` |
| `UnexpectedCharacter` | The character | `unexpected character 'q'`, with the character in Rust debug form, so a control character is escaped |
| `UnclosedParenthesis` | Nothing | `unclosed parenthesis` |
| `ZeroOrder` | Nothing | `order 0 is not allowed` |
| `MultiDigitOrder` | Nothing | `a digit 1 followed by another digit is ambiguous, write an order of 10 or more in parentheses, such as (12)` |
| `AmbiguousOrder` | The order | `order 3 after a boundary and a handle or crosscap is ambiguous`, with the bare value for an order of 10 or more, such as `order 12 after ...` and not `(12)`, and with `inf` for an infinite order |
| `LimitExceeded` | Which limit | `the signature is longer than 1024 bytes`, `order is larger than 4294967295`, or `the signature has more than 64 features` |
| `BadOrbifold` | Nothing | `bad orbifold, no symmetry group has this signature` |
| `PositiveNonCompact` | Nothing | `an infinite order needs an Euler characteristic of 0 or less` |
| `Overflow` | Which value | `the Euler characteristic does not fit exact 64-bit arithmetic`, `the group order is not a whole number or does not fit a 64-bit unsigned integer`, or `the deformation dimension does not fit a 32-bit unsigned integer` |

Every fixed text is in lower case, except the proper name "Euler", and has no trailing punctuation (ERR-4 of the coding standard). The character that `UnexpectedCharacter` quotes can be in any case. The public accessors are `kind()` and `span()` only. The order or character in the text has no accessor, because no caller needs one.

## 11. Resource Limits

The crate limits the length of the text, each order, and the size of a signature. The scanner checks the length at step 1 of Section 5.1, the size after each change to it (Section 5.1), and each order at step 1 of Section 5.3. Section 16 states the values and their reasons, because Section 5.9 of the architecture places the limits in the Security Considerations.

## 12. Testing

### 12.1. Tests

The unit tests sit beside the code (TEST-1 of the coding standard). The integration tests form one binary, `crates/escher-signature/tests/signature/main.rs`, with the modules `parsing`, `tables`, `canonical`, `properties`, `enumeration`, and `common` (TEST-10). Its root has a `//!` line, because `missing_docs` covers test roots. The snapshot tests of the item "Snapshots" of the plan live in `escher-cli`, and `docs/design/cli.md` specifies them. Every error text of this crate that a known input produces passes through one of those snapshots, so this crate has no `insta` dependency. No known input produces the `Overflow` of the group order or of the deformation dimension. So the unit test "Error text" checks their texts, and the unit test "Classification functions" checks their failure paths on the private functions directly.

Each test below names the item of the field "Tests and acceptance" of Section 5.1 of the plan that it serves:

| Test | Plan item | Kind and place | Asserts | Source |
| --- | --- | --- | --- | --- |
| Euclidean table | Tables | Table test, `tables` | The 17 signatures have $\chi = 0$, are Euclidean and compact, and display as the table writes them | Section 8.1 of the reference |
| Spherical table | Tables | Table test, `tables` | The 7 series for $n$ from 1 to 12 and the 7 particular groups are spherical with the listed order. Each order $n$ is written as one digit below 10 and in parentheses from 10, and $n = 1$ is written `(1)`, because `11` and `*11` are rejected | Sections 7.3 and 8.2 of the reference |
| Frieze table | Tables | Table test, `tables` | The 7 friezes, written with `(inf)` and with U+221E, have $\chi = 0$, are Euclidean, friezes, not compact, and not general non-compact | Section 8.3 of the reference |
| Characteristic examples | Tables | Table test, `tables` | `4*3` gives $-1/12$, `*3333` gives $-1/3$, `*532` gives $1/60$ and order 120, and `*22(inf)` gives 0 | Section 6.4 of the reference |
| Bad families | Tables, failure paths | Table test, `tables` | `m`, `mn`, `*m`, and `*mn` for small orders return `BadOrbifold` with the whole span, while `mm` and `*mm` are accepted | Section 7.2 of the reference |
| Area and dimension | Tables | Table test, `tables` | The area over $\pi$ is $2/3$ for `*3333`, $1/42$ for `*732`, $1/30$ for `*532`, $1/3$ for `3*(inf)`, and `None` for `*632`. The dimension is 0 for `*732` and `4*3`, 1 for `*3333`, 13 for `4*235*74xx`, and `None` for the Euclidean `*632` and `o`, the spherical `*532` and `1`, and the non-compact `*22(inf)`, `3*(inf)`, and `*23(inf)` | Section 7.3 of the reference for `*3333`, Section 5.1 of the plan for `*732` and `*3333`, and Section 10.1 of the reference for `4*235*74xx`. The areas of `*532` and `3*(inf)` and the dimension of `4*3` follow from the formulas of Sections 9.4 and 9.5 |
| Order two | Tables | Table test, `tables` | `x`, `22`, and `*` are three different signatures, each of order 2, and `*632` differs from `632` | Sections 4.2 and 8.2 of the reference |
| General non-compact | Tables | Table test, `tables` | The values of Section 9.2. Signatures that combine a triangle shape with something else are general non-compact, such as `o*23(inf)`, `x*23(inf)`, `2*23(inf)`, `(inf)32*`, and `*732*(inf)` | Section 2.1 of the architecture |
| Euclidean enumeration | Enumeration | Depth-first enumeration, `enumeration` | Over up to 2 handles, up to 3 crosscaps, up to 5 cone points, up to 3 boundaries of up to 5 corners each, and orders 2 to 8, the canonical strings with $\chi = 0$ are exactly the 17 of Section 8.1 of the reference. A branch stops when $\chi < 0$, because every defect of Section 6.1 of the reference is positive | Section 32 of [FG], Sections 7.1 and 8.1 of the reference |
| Equivalence classes | Properties | Bounded exhaustive test, `enumeration` | A test-side model applies rules 1 to 5 in both directions, with the reversal of a single boundary only when a crosscap is present, to every signature within two bounds and groups them into classes. Each class has exactly one canonical string, no two classes share one, and each canonical string lies in its own class. The bounds are, first, up to 2 cone points with orders in 2, 3, and $\infty$, up to 3 boundaries with up to 4 corners in total across all boundaries with orders in 2, 3, 4, and $\infty$, and $2h + c \leq 4$, and second, no cone point, up to 2 boundaries with up to 6 corners in total across all boundaries with orders in 2, 3, and 4, and $2h + c \leq 3$. The corners are counted in total and not per boundary, because a bound per boundary gives billions of signatures for the first bound | Section 4.1 of the reference |
| Random equivalence | Properties | `proptest`, `properties` | A random sequence of rule applications on a test-side model gives an equal `Signature`, an equal string, and an equal $\chi$, or errors of the same kind. The model permutes cone points and boundaries, rotates, reverses every boundary, reverses a single boundary only when a crosscap is present, applies rule 5 in both directions, which trades a handle for two crosscaps and two crosscaps for a handle while a crosscap remains, and inserts an order of 1 only where the grammar accepts it, as `(1)` anywhere or as `1` before a character that is not a digit, while the text stays within 1024 bytes | Section 4.1 of the reference |
| Layout variants | Properties | `proptest`, `properties` | Changes of layout that the grammar accepts give an equal `Signature`: upper case, `(7)` for `7`, U+221E for `(inf)`, and an `o` or an `x` moved to another place where the grammar accepts it | `DEC-8` |
| Dimension sign | Properties | `proptest`, `properties` | A test-side model generates compact signatures of every geometry, with up to 4 handles, up to 6 crosscaps, up to 10 cone points, and up to 4 boundaries of up to 6 corners each, with orders 2 to 12, so the size stays within 52. Each text parses or fails with `BadOrbifold` and no other kind. Each parsed signature is compact, a hyperbolic one has a `Some` deformation dimension that equals the value that the formula of Section 9.5 gives from the features that the public accessors return, and any other has `None` | Section 9.5, and the assumption of Section 15.1 |
| Round trip | Properties | `proptest`, `properties` | The canonical form is idempotent, and `to_string` followed by `parse` round-trips | `DEC-9` |
| Any text | Failure paths | `proptest`, `properties` | For any string, `parse` returns without a panic, an error has a span inside the text on character boundaries, and every spherical result has `Some` group order | ERR-5 and TEST-6 of the coding standard |
| Kinds and spans | Failure paths | Unit tests in `parse.rs`, and `parsing` | Every row of the table of Section 5.5, 65 features, a run of `o` at the handle weight, and the four rows of the table of Section 8.3 | Sections 5 and 8.3 |
| Error text | Failure paths | Unit test in `error.rs` | The `Display` of every kind starts with `cannot parse orbifold signature: ` and, apart from the proper name "Euler" and the character that `UnexpectedCharacter` quotes, is in lower case | ERR-4 of the coding standard |
| Canonical examples | Risks of the plan | Table test, `canonical` | Every row of the tables of Sections 4.3 and 7.2, and `1`, `*`, and `x` | Sections 3.5 and 4.1 of the reference |
| Rational | None | Unit tests in `rational.rs` | Reduction, sign, zero, `Display`, the failure to narrow, and the absolute value, which fails only for the numerator `i64::MIN` | Section 8.1 |
| Classification functions | Failure paths | Unit tests in `classify.rs` | Step 6 of Section 8.2 rejects a reduced numerator of `i64::MIN` for $\chi$ and for $2\chi$ and accepts one that fits. The group order is 120 for $\chi = 1/60$ and 1 for $\chi = 2$, and fails for $3/5$ and for $1/(2^{64} - 1)$. The deformation dimension is 0 for `732` and 2 for `2223`, and fails for `22` and for the trivial signature | Sections 8.2, 9.3, and 9.5. The values follow from the formulas of those sections |
| Auto traits | None | Compile-time assertion in a unit test in `lib.rs`, which calls a generic function with the bounds `Send + Sync + 'static` once for each public type | Each public type is `Send`, `Sync`, and `'static` | CONC-1 of the coding standard |
| Doctests | None | Doctests | The examples of the crate-level documentation | Sections 7.4 and 7.5 of `docs/writing-documentation.md` |

The property tests commit their regression files (TEST-9 of the coding standard). They take their configuration from the one helper `proptest_config()` in `common`, which returns the default configuration of `proptest`.

SIG-16. Withdrawn on 2026-11-19.

## 13. Dependencies

SIG-17. The crate MUST declare `thiserror` as its only normal dependency, because the crate table of Section 3.1 of the architecture gives it only `thiserror` and each further dependency adds build time, licence obligations, and supply-chain risk (DEP-2 of the coding standard). It MUST expose no type of a dependency in its public surface, because such a type makes the dependency a public dependency whose upgrade breaks the crate (ERR-2 and DEP-3).

The crate declares these dependencies, each from one entry in the `[workspace.dependencies]` table of the root `Cargo.toml` (DEP-2):

| Dependency | Kind | Features | Why |
| --- | --- | --- | --- |
| `thiserror` | Normal | Default | Derives `Display` and `std::error::Error` (Section 3.1 of the architecture) |
| `proptest` | Dev | `default-features = false`, with `std` | The property tests (Section 7.1 of the architecture). The features `fork` and `timeout` are off, because no test uses them |

The crate sets `publish = false` (`DEC-22`), inherits the workspace fields and lints, and declares no Cargo feature.

These facts were verified on 2026-11-16 with `cargo deny check` and `just check` on the workspace, whose `Cargo.lock` fixes these versions:

- The resolved versions are `thiserror` and `thiserror-impl` 2.0.21, `syn` 3.0.10, `proc-macro2` 1.0.107, `quote` 1.0.47, and `unicode-ident` 1.0.27, and for the tests `proptest` 1.12.0, `rand` 0.10.4, `rand_core` 0.10.1, `rand_xorshift` 0.5.0, and `getrandom` 0.4.3.
- No crate appears in two versions, so `syn` 2 and 3 do not both occur.
- With every dependency of this crate, including `proptest` and the other dev-dependencies, `cargo deny check` passes and reports no unused licence. `Unicode-3.0` comes from `unicode-ident`. Section 8 of `docs/design/tooling.md` states the allow-list, the settings that make `deny.toml` cover dev-dependencies, and why `Apache-2.0` and `Unicode-3.0` are compatible with GPL-3.0-or-later.
- The highest minimum Rust version that a dependency declares is 1.88, for `proptest`, below the workspace MSRV of 1.99.
- The feature set of `proptest` in the table builds and runs the property tests.

## 14. Unverified Statements

PLAN-3 of the plan requires this list. Each row names an unverified statement of Section 7 of the plan that the code relies on, and what the code does about it:

| Label | What the code relies on | What the code does |
| --- | --- | --- |
| `UNV-20` | The canonical string, parentheses for orders of 10 or more, the spellings `(inf)` and U+221E for an infinite order, and `1` on its own as the trivial signature | Implements the conventions of Sections 4 and 7 as project conventions. The tests check their internal consistency, which is idempotence, the round trip, and invariance, because no source exists to compare them with |
| `UNV-21` | The geometry of a signature with an infinite order and $\chi \neq 0$ | Classifies it by the sign of $\chi$ when $\chi < 0$ (`DEC-10`), rejects it when $\chi > 0$ (Section 6), returns its area (Section 9.4), and returns no deformation dimension (Section 9.5) |
| `UNV-10` | The deformation counts of Euclidean signatures | Avoids them. `deformation_dimension()` returns `None` for a Euclidean signature |

## 15. Assumptions, Choices, and Points for the Owner

### 15.1. Assumptions

These readings of this design rest on no sentence of a source. Each names what would settle it:

- Rules 1 to 4 apply to an infinite order as they apply to a finite one, and the completeness of the rules that [CBG] states covers signatures with an infinite order. The canonical form and its separating property rely on this. A source that states the rules for non-compact signatures would settle it (`UNV-20`, `UNV-21`).
- $\chi(X_O) = 2 - 2h - c - b$ in Corollary 13.3.7 of [TH]. This is the part of the defect rule of Section 6.1 of the reference that comes from `o`, `x`, and `*`, set against formula 13.3.4 of [TH] in Section 6.2 of the reference. No sentence of the reference states it in one line. The three stated values of Section 9.5 test it, and a source that states the Euler characteristic of the underlying surface in these terms would settle it.
- The deformation dimension $-6 + 6h + 3c + 3b + 2k + l$ is never negative for a compact hyperbolic signature. No cited source states it. The constructor rejects a negative value with `Overflow` (Section 9.5), and the property test "Dimension sign" of Section 12.1 checks that no compact hyperbolic signature meets that rejection. A source that states the dimension as non-negative for every compact hyperbolic orbifold would settle it.

### 15.2. Choices of This Design

No other document dictates these choices:

| Choice | Section |
| --- | --- |
| An order other than 1 is ambiguous when it follows a `*` with an `o` or an `x` in between. For example `*2x3` is rejected, while `*2x*3` is accepted | 4.2, reading rule 4 |
| An unparenthesized `1` directly followed by a digit is rejected, which narrows `DEC-8`. For example `2*12` is rejected | 4.2, reading rule 5 |
| An order of 1 is dropped before the ambiguity check, so `*2x1` is `*2x` | 4.2, reading rule 3 |
| Whitespace is rejected everywhere | 4.2, reading rule 11 |
| `(inf)` in lower case only, U+221E only on its own, parenthesized orders below 10 accepted, and leading zeros read by value | 4.2, reading rules 7 and 12 |
| Every handle becomes two crosscaps when a crosscap is present (`DEC-9`). For example `oox` becomes `xxxxx` | 7.2, step 2 |
| A crosscap count $c + 2h$ that does not fit `u32` is `LimitExceeded` for the size | 7.2, step 2 |
| An infinite order sorts above every finite order | 7.1 |
| Without a crosscap, the boundaries are the greater of the two sorted lists before and after reversing every boundary | 7.2, step 4 |
| $\chi$ is computed on the canonical features, in a fixed order, over one common denominator in `i128`, and returned with an `i64` numerator and a `NonZeroU64` denominator, and $\chi$, $2\chi$, and the absolute value of each must fit | 8 |
| $\chi$, $2\chi$, the group order, and the deformation dimension are not stored, and one private `expect` recomputes them | 8.4 |
| The deformation dimension is computed at construction, and a value that does not fit `u32` is `Overflow` | 9.5 |
| `area_over_pi` as the name, with a value for a non-compact hyperbolic signature | 9.1, 9.4 |
| `is_general_non_compact` as a method with the predicate of Section 9.2 | 9.2 |
| The group order is computed at construction from the reduced $\chi$, and a quotient that is not an integer or does not fit `NonZeroU64` is `Overflow`, so `group_order` cannot fail and returns a `NonZeroU64` | 9.3 |
| Ten error kinds, with one `LimitExceeded` for three limits and `PositiveNonCompact` apart from `BadOrbifold` | 6, 10.2 |
| The span is a `Range<usize>`, and the error holds no input text beyond the character or order that its text shows | 10.1 |
| `Order` is opaque, `Geometry` has no `Display`, and `FromStr` is the only constructor | 3 |
| The limits of 1024 bytes and 64 features, with a handle counted twice | 16 |

### 15.3. Points for the Owner

None of these blocks the crate. Each states what the design does until the owner decides:

1. Order of the infinite order. The design sorts it above every finite order, which gives `*(inf)22` and `(inf)22` and follows the advice of [CBG] and its `*∞32`. Sorting it below would give `*22(inf)` and `22(inf)`, which matches the frieze table as printed. A later change alters the canonical strings, the parameter names of Section 5.6 of the architecture, and the snapshots of `docs/design/cli.md` (Section 7.1).
2. Rejection of unparenthesized multi-digit orders. The design rejects a `1` directly followed by a digit, so `11` and `*11` are rejected although `DEC-8` accepts an order of 1 and [CBG] and [CH] write them. This is the sounder option, because the other reading of such a text names a different group (Section 4.2, reading rule 5).
3. Area of a non-compact hyperbolic signature. The design returns it under `UNV-21` (Section 9.4). The stricter option returns it for compact signatures only, until the reference cites a source for the non-compact case.

## 16. Security Considerations

The threat model is a caller that passes text from an untrusted party to `Signature::from_str`. The attacker can choose any `&str`. The attacker cannot change the crate, its dependencies, or the caller.

The attacks in scope are exhaustion of memory or time, a panic that stops the caller, and an arithmetic overflow that yields a wrong value. The crate counters them with these limits, the checked arithmetic of Section 8.3, and the rule that it never panics on input (SIG-2).

SIG-18. The crate MUST enforce the limits of this table and return `LimitExceeded` when an input exceeds one, because the input is untrusted, and without a limit a crafted signature makes this crate and every later crate spend unbounded memory and time (Section 11 of the architecture).

| Limit | Value | Reason for the number |
| --- | --- | --- |
| Length of the text | 1024 bytes, checked before the scan | It bounds the scan and the memory, and it exceeds the 768 bytes of the longest canonical string, so every canonical string parses. The number is a choice of this design |
| Order | `u32::MAX`, 4294967295 | `DEC-25`. No operation of this crate costs more for a larger order. Later crates set their own limits for costs that grow with the order |
| Size | 64, with a handle counted twice | It bounds the quadratic canonicalization and the size of the input to later crates. The largest signature in the reference, `4*235*74xx`, has size 10 (Section 10.1 of the reference). A handle counts twice so that rule 5 does not change the size. The number is a choice of this design |

The limits are private constants, and unit tests inside the crate read them.

The other topics of Section 3.2 of `docs/writing-documentation.md` apply as follows:

- Unsafe code: the crate has none, because the workspace denies `unsafe_code`, and it calls no native library.
- Untrusted input and file paths: the text is the only input. The crate reads and writes no file and keeps no state between calls.
- Build scripts: the crate has none. The procedural macro of `thiserror`, with `syn`, `quote`, and `proc-macro2`, runs at build time on the source of this crate only.
- Dependencies: `Cargo.lock` fixes every version, and cargo-deny checks licences, advisories, duplicates, and sources under the policy of Section 8 of `docs/design/tooling.md`, for the dev-dependencies as well as the normal ones.

Attacks on the build machine, the repository, or the caller, and the display of the untrusted text, are out of scope, because this crate neither builds nor prints anything, and `docs/design/cli.md` governs what `escher-cli` echoes.

The residual risk is a misreading that the grammar cannot detect. An order of 20 or more written without parentheses, such as `23` meant as one order, reads as several orders and names a different group without an error. The canonical string shows the reading that the parser took. A second residual risk is a classification of a non-compact signature that rests on `UNV-21` and turns out to be wrong.

## 17. Operational Considerations

The crate declares no Cargo feature, reads no environment variable, and has no build script. It is pure Rust with no platform-specific code, and the supported platform of the proof of concept is Linux (Section 12 of the architecture). The MSRV is the one that the workspace declares in `rust-version`. The crate emits no log.

Every failure is an `Error` with an `ErrorKind` and a span:

| Failure | Kind | Recovery |
| --- | --- | --- |
| Empty text | `Empty` | Supply a signature |
| A character outside the grammar, including whitespace | `UnexpectedCharacter` | Remove or replace the character at the span |
| Unclosed `(` | `UnclosedParenthesis` | Close the parenthesis |
| Order 0 | `ZeroOrder` | Use an order of at least 2, or omit the order |
| A digit 1 directly followed by a digit | `MultiDigitOrder` | Write the order in parentheses, such as `(12)` |
| An order after a boundary and a handle or crosscap | `AmbiguousOrder` | Move the order before the boundary as a cone point, or the handle or crosscap after the corners |
| Text, order, or size over a limit | `LimitExceeded` | Reduce the input below the limit of Section 16 |
| Bad orbifold | `BadOrbifold` | None. No group has the signature |
| An infinite order with $\chi > 0$ | `PositiveNonCompact` | None. The design classifies no such signature |
| Arithmetic that does not fit | `Overflow` | For the Euler characteristic, use smaller orders with fewer distinct prime factors. For the group order or the deformation dimension, none |

## 18. References

### 18.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC5234] Crocker, D., Ed. and P. Overell, "Augmented BNF for Syntax Specifications: ABNF", STD 68, RFC 5234, January 2008, <https://www.rfc-editor.org/info/rfc5234>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 18.2. Informative References

The tags of the mathematical sources resolve to the reference sections of `docs/research/orbifold-notation.md`:

- [CBG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [CH] Section 13.1 of `docs/research/orbifold-notation.md`.
- [FG] Section 13.1 of `docs/research/orbifold-notation.md`.
- [TH] Section 13.1 of `docs/research/orbifold-notation.md`.

## Appendix A. Requirement Summary

| Label | Requirement | Section |
| --- | --- | --- |
| SIG-1 | The crate exports exactly the items of Section 3.1, with no public field | 3.1 |
| SIG-2 | `from_str` accepts exactly the grammar and reading rules and rejects everything else, without a panic, with the stated kind and span | 4.1 |
| SIG-3 | An order of 1 is dropped before the ambiguity check | 4.2 |
| SIG-4 | An ambiguous order is rejected | 4.2 |
| SIG-5 | An unparenthesized `1` directly followed by a digit is rejected with `MultiDigitOrder` | 4.2 |
| SIG-6 | The first error in text order is returned, and after a clean scan the crate canonicalizes, computes $\chi$, runs the checks, computes the group order, and computes the deformation dimension, in that order | 5.4 |
| SIG-7 | A bad orbifold is rejected with `BadOrbifold` | 6 |
| SIG-8 | An infinite order with $\chi > 0$ is rejected with `PositiveNonCompact` | 6 |
| SIG-9 | The constructor stores the canonical form | 7.2 |
| SIG-10 | `Display` writes the canonical string, which parses back to an equal value | 7.3 |
| SIG-11 | $\chi$ is exact and checked, and an overflow, or a reduced $\chi$, $2\chi$, or absolute value of either that does not fit, is an error | 8.3 |
| SIG-12 | Intermediate values are `i128`, and only reduced results are narrowed | 8.3 |
| SIG-13 | Each classification method returns its defined value, or `None` | 9.1 |
| SIG-14 | `Error` follows ERR-1, with the operation that failed | 10.1 |
| SIG-15 | The `Display` text of each kind is the one stated | 10.2 |
| SIG-16 | Withdrawn on 2026-11-19. | 12.1 |
| SIG-17 | `thiserror` is the only normal dependency, and no dependency type is public | 13 |
| SIG-18 | The length, order, and size limits are enforced with `LimitExceeded` | 16 |
