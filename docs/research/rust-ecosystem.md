# Rust Ecosystem Survey

| Field | Value |
| --- | --- |
| Status | Snapshot |
| Date | 2026-11-15 |
| Applies to | The choice of Rust crates and related tools for the proof of concept and the later phases of Escher |

## Abstract

This document surveys the Rust crates and related tools that Escher could depend on, one job at a time, from PNG encoding to the GUI stack and Lean 4. For each job it lists the candidates, the facts that decide between them, and the research's recommendation with its reason. It then states which licences the candidates would bring into the dependency tree, what `deny.toml` would need, and which candidates risk duplicate versions of a dependency. Two findings shape the plan more than the others. No maintained Rust crate provides coset enumeration or low-index subgroup search, so the research recommends that Escher implement them itself. The official Lean installer performs no checksum verification, and Mathlib covers the upper half-plane and the basics of Coxeter systems but lacks, among other things, a metric on the Poincare disc, Fuchsian groups, and Todd-Coxeter coset enumeration.

## 1. Scope

This document covers the jobs listed in Sections 4 to 17, the licences and duplicate-version risks of the candidates in Sections 18 and 19, and the conflicts between sources in Section 20. It is a snapshot. Every version, release date, minimum supported Rust version, and maintenance state is as of 2026-11-15, and an author re-checks each one before adding the dependency, because releases and advisories change after the survey date.

Each "research recommendation" records what the research found and why. It is not a decision. The project chooses its dependencies later, in its design documents, and those documents prevail over this one.

This document does not cover the mathematics of orbifolds, tilings, or rendering, which other subject-matter references under `docs/research/` state. It does not cover the development tools that `docs/design/tooling.md` specifies, such as cargo-deny and cargo-nextest.

## 2. Notation and Conventions

### 2.1. Terminology

- minimum supported Rust version (MSRV): The oldest Rust toolchain that a crate declares it builds with, in the `rust-version` field of its manifest.
- duplicate version: Two or more semver-incompatible versions of the same crate in one resolved dependency graph, which `multiple-versions = "deny"` in `deny.toml` rejects.
- licence expression: A Software Package Data Exchange (SPDX) expression that names the licences of a crate, such as `MIT OR Apache-2.0`.
- dev-dependency: A dependency that only tests, examples, and benchmarks use, which is not part of a distributed binary.
- forbids `unsafe`: The crate root contains `#![forbid(unsafe_code)]`, so the crate itself has no `unsafe` code. It says nothing about the crate's dependencies.
- research recommendation: The option that the research found best for a job, with its reason. Section 1 states its status.

### 2.2. Source Conventions

Eight web-searcher agents gathered every fact in this document, and all sources were accessed on 2026-11-15. A source is primary unless it is marked otherwise. Primary sources are the crates.io API, a raw file in the crate's repository, docs.rs, and rustsec.org.

The research recorded URLs only. Each reference entry therefore names the publisher or repository owner as the URL shows it, with a descriptive title taken from the URL, and the access date stands in for the publication year.

The fetch tool of the agents summarized pages, so some quotations are paraphrases. A cell or sentence that the research marked carries one of these marks:

- (secondary): The fact comes from a secondary source, such as lib.rs.
- (unverified): The research did not confirm the fact.
- (inferred): The research concluded the fact from other facts and found no source that states it.
- (general knowledge): The fact comes from general knowledge, and no source was fetched for it.

"No advisory" means that the rustsec.org page for the package returned HTTP 404. That is an inference and not a confirmation that the crate has no advisory.

Nobody ran `cargo tree` for this survey. Dependency counts come from the tables of direct dependencies on crates.io, and every duplicate-version prediction in Section 19 is unverified until `cargo tree -d` and `cargo deny check` run on a real manifest.

A cell that reads "none given" in a Sources column means that the research gave no URL for the row. Section 21 lists these rows.

## 3. Constraints That Drive the Choices

These settings of the repository decide which candidates fit and what adding one costs:

| Constraint | Setting | File |
| --- | --- | --- |
| Licence of Escher | `GPL-3.0-or-later` | `Cargo.toml`, `[workspace.package]` |
| Licence allow-list | Allows only `GPL-3.0-or-later`, with `unused-allowed-license = "deny"` | `deny.toml` |
| Duplicate versions | `multiple-versions = "deny"`, with `all-features = true` in `[graph]` | `deny.toml` |
| `unsafe` code | The workspace lint tables deny `unsafe_code` in Escher's own crates | `Cargo.toml` |
| MSRV | `rust-version` is 1.99 | `Cargo.toml` |

Every dependency licence other than `GPL-3.0-or-later` has to be added to the allow-list, and because unused allowed licences are denied, the list holds only licences that are present. Section 18 lists the licences and the changes.

Because `all-features = true` is set, cargo-deny resolves every optional feature of every dependency. A duplicate version that only an optional feature brings in therefore still fails the check.

The `unsafe` policy binds Escher's own crates and not its dependencies. The tables record whether a candidate forbids `unsafe` because the project prefers the sounder of two comparable options.

Every candidate needs a Rust version below 1.99, so the MSRV excludes none of them.

## 4. PNG Encoding

The proof of concept writes PNG files. The deciding facts are control over the colour chunks (gAMA, sRGB, cHRM, and iCCP), the absence of `unsafe`, and the size of the dependency tree.

| Crate | Latest | Licence | MSRV | Maintenance | Dependencies | `unsafe` | Notes | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `png` | 0.18.1, 2026-02-14 | `MIT OR Apache-2.0` | 1.73 on crates.io, 1.81 on master | Commits to 2026-09-23. Master is about 9 months ahead of the release | `bitflags` 2, `crc32fast`, `fdeflate`, `flate2`, `miniz_oxide` 0.8. Master drops `miniz_oxide`. Pure Rust | Forbids `unsafe`. The opt-in `zlib-rs` feature adds `unsafe` to the whole `flate2` tree | Setters follow the table. No cICP writer found. 0.18.0 removed `Encoder::set_srgb` | [PNG-DEPS], [PNG-ENC], [PNG-LIB], [PNG-CHANGES] |
| `image` | 0.25.10, 2026-03-10 | `MIT OR Apache-2.0` | 1.88 (Section 20) | Very active, commits 2026-11-11 | With `default-features = false, features = ["png"]`, about 16 crates, including the always-on `moxcms` (`BSD-3-Clause OR Apache-2.0`) and `pxfm` | Does not forbid `unsafe` | `PngEncoder` sets compression, filter, ICC, and EXIF, but not gAMA or sRGB. RUSTSEC-2020-0073 and RUSTSEC-2019-0014 do not affect 0.25 | [IMAGE-DEPS], [IMAGE-PNGENC], [IMAGE-PNGRS] |
| `zune-png` | 0.5.2, 2026-03-11 | `MIT OR Apache-2.0 OR Zlib` | 1.87 | Monorepo active | `zune-core`, `zune-inflate` | "Zero unsafe outside of platform specific intrinsics" | Mainly a decoder. Control of encoder chunks and compression is not documented | [LIBRS-ZUNE-PNG] (secondary) |
| `lodepng` | 3.12.2, 2026-02-16 | `Zlib` | 1.72 | Last commit 2026-02-16 | `flate2`, `crc32fast`, `rgb`. The default C foreign function interface (FFI) feature adds `libc` | Not verified | `Info` has no gAMA or sRGB fields, only custom chunks | [LODEPNG-TOML] |
| `mtpng` | 0.4.1, 2024-04-26 | `MIT` | None stated | Sparse releases | `libz-sys` (C) and `rayon` | Not verified | Not pure Rust | [LIBRS-MTPNG] (secondary) |

The `png` encoder has these setters: `set_depth`, `set_color`, `set_compression`, `set_deflate_compression(Level(1..9))`, `set_filter`, `set_source_gamma`, `set_source_srgb`, and `set_source_chromaticities`. It writes iCCP through `Info.icc_profile` [PNG-ENC].

### 4.1. Research Recommendation

The research recommends `png`, because it is the only candidate that forbids `unsafe` and has setters for gAMA, sRGB, and cHRM. `image` adds `moxcms` and gives less control over chunks. Section 19 describes the `miniz_oxide` duplicate-version risk of `png` and how to avoid it.

## 5. Command-Line Parsing

The proof of concept is a command-line tool with subcommands, such as the command that reports information on a symmetry. The deciding facts are the quality of help and error text, maintenance, and the dependency tree.

| Crate | Latest | Licence | MSRV | Maintenance | Dependencies | `unsafe` | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `clap` (derive) | 4.6.7, 2026-09-14 | `MIT OR Apache-2.0` | 1.85 | Commit 2026-11-15 | `clap_builder` (`anstyle`, `clap_lex`, optional `anstream`, `strsim`, `terminal_size`) and `clap_derive` (`heck`, `proc-macro2`, `quote`, `syn ^3.0.2`) | `clap_builder` forbids `unsafe` | [CLAP], [CLAP-DERIVE-DEPS], [CLAP-BUILDER-LIB] |
| `bpaf` | 0.9.29, 2026-11-12 | `MIT OR Apache-2.0` | 1.68 | Active | None by default. Optional `bpaf_derive`, `owo-colors`, `supports-color` | No forbid in `lib.rs` | [BPAF] |
| `argh` | 0.1.19, 2026-03-16 | `BSD-3-Clause` | None stated | Last commit 2026-05-27, slow | `argh_derive`, `argh_shared` | Not fully checked | [ARGH] |
| `lexopt` | 0.3.2, 2026-02-28 | `MIT` | About 1.31 per lib.rs (secondary) | Maintained | None | Forbids `unsafe` | [LEXOPT-LIB] |
| `pico-args` | 0.5.0, 2022-06-04 | `MIT` | None stated | Last commit 2023-10-19, stagnant | None | Forbids `unsafe` | [PICO-ARGS] |

`lexopt` is a tokenizer only. It provides no help text, subcommands, or defaults [LEXOPT-LIB].

### 5.1. Research Recommendation

The research recommends `clap` with the derive API, because it has the best help text, error text, and suggestions, and the most active maintenance. It names `bpaf` as the lean alternative with no dependencies by default. The `syn ^3.0.2` requirement of `clap_derive` is the main duplicate-version risk of this choice (Section 19).

## 6. Complex Numbers and Small Linear Algebra in `f64`

The proof of concept computes in double precision, with complex numbers for Mobius maps and small real vectors and matrices. The deciding facts are `f64` support, a complex type, and what a later phase pulls in anyway.

| Crate | Latest | Licence | MSRV | Maintenance | `f64` and complex support | Dependencies, `unsafe`, and notes | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `num-complex` | 0.4.6, 2024-05-07 | `MIT OR Apache-2.0` | 1.60 | Stable, rarely released | `Complex<f64>`, also named `c64` | Depends only on `num-traits`. A stray version "0.1.44" dated 2025-01-25 appears on crates.io, unexplained | [NUM-COMPLEX-TOML] |
| `glam` | 0.34.3, 2026-11-07 | `MIT OR Apache-2.0` | 1.85 | Very active, with a breaking minor release every 1 to 3 months | `DVec2`, `DVec3`, and `DMat3` through `float-types`, on by default. No complex type | No default dependencies. Uses single instruction, multiple data (SIMD) intrinsics internally, and `scalar-math` disables them. 0.34.0 removed `perspective` and similar functions | [GLAM-README], [GLAM-CHANGELOG] |
| `nalgebra` | 0.35.0, 2026-05-24 | `Apache-2.0` only | 1.89 | Active | Generic scalar, so `Matrix2<Complex<f64>>` works through `num-complex` (not verified in the documentation) | 6 required dependencies (`approx`, `num-complex`, `num-rational`, `num-traits`, `simba`, `typenum`), plus `matrixmultiply` and macros by default. Uses `unsafe` internally. RUSTSEC-2021-0070 affects only versions before 0.27.1 | [NALGEBRA-DEPS], [NALGEBRA-CHANGELOG] |
| `ultraviolet` | 0.10.0, 2025-04-26 | `MIT OR Apache-2.0 OR Zlib` | None stated | Master unreleased (0.11, on `wide` 1.7) | `f64` feature (Section 20), with types prefixed `D`. No complex type. Uses rotors by design | Built on `wide` | None given |
| `cgmath` | 0.18.0, 2021-01-03 | `Apache-2.0` | None stated | Unmaintained (RUSTSEC-2026-0196), and `swap_columns` is unsound (RUSTSEC-2026-0197) | Not applicable | Avoid | [RUSTSEC-2026-0196], [RUSTSEC-2026-0197] |
| `mint` | 0.5.9, 2022-02-02 | `MIT` | 1.52.1 | Stable | Interoperability types only, with no arithmetic | Not applicable | None given |

No later phase forces a choice of linear algebra crate:

- wgpu 30 has none of `glam`, `nalgebra`, `num-complex`, or `mint` among its normal dependencies. `glam` appears only as a dev-dependency [WGPU-TOML].
- Slint's `i-slint-core` depends on `euclid` 0.22 (`MIT OR Apache-2.0`), not on `glam` [SLINT-CORE-TOML].
- `vello` and `vello_cpu` use `kurbo`, which works in `f64` (Section 13).
- `pulp` depends on `num-complex` 0.4 (Section 15).

### 6.1. Research Recommendation

The research recommends `num-complex` for `Complex<f64>`, with a small hand-written `Mat2<Complex<f64>>` for Mobius maps of a few dozen lines, and `glam` `DVec2`, `DVec3`, and `DMat3` for real vectors. Hand-written types and no `glam` at all are an alternative, because nothing later forces `glam`. It reserves `nalgebra` for broad use of generic or complex matrices, and advises against `cgmath`.

## 7. Data Parallelism

The proof of concept computes each pixel independently, so rows can be split across threads. The deciding facts are maturity, the dependency tree, and whether a crate is needed at all.

| Option | Latest | Licence | MSRV | Notes | Sources |
| --- | --- | --- | --- | --- | --- |
| `rayon` | 1.12.0, 2026-04-14 (`rayon-core` 1.13.0) | `MIT OR Apache-2.0` | 1.80 on crates.io, 1.85 on main (Section 20) | About 6 crates (`rayon-core`, `either`, `crossbeam-deque`, `crossbeam-epoch`, `crossbeam-utils`). `unsafe` internally, safe API. `par_chunks_mut` suits rows | [RAYON-VERSIONS], [RAYON-RELEASES] |
| `std::thread::scope` and `std::thread::available_parallelism` | Stable since Rust 1.63 and 1.59 | Not applicable | Not applicable | Safe and works with borrowed data. `available_parallelism` is not cached and can miscount under cgroups | [STD-SCOPE], [STD-AP] |
| `chili` | 0.2.1, 2025-03-26 | `MIT OR Apache-2.0` | None stated | `join` only, no dependencies | [CHILI] |
| `orx-parallel` | 4.1.1, 2026-10-02 | `MIT OR Apache-2.0` | None stated | A large family of `orx-*` crates, with frequent major releases | None given |
| `forte` | 1.0.0-beta.2, 2026-11-02 | `MIT OR Apache-2.0` | 1.96 | Prerelease | [FORTE] |

### 7.1. Research Recommendation

The research recommends `rayon`, because it has a small tree, is mature, and later candidates such as `vello_cpu` multithreading and `image-compare` use it too. For a fixed split by rows, `std::thread::scope` suffices with no dependency.

## 8. Colour

Escher fills polygons with colours that the user chooses, so it needs colour spaces, gamma-correct blending, and parsing of colour input. The deciding facts are `f64` support, perceptual spaces such as Oklab and OKLCH, and duplicate versions of `phf`.

| Crate | Latest | Licence | MSRV | Notes | Sources |
| --- | --- | --- | --- | --- | --- |
| `palette` | 0.7.7, 2026-08-02 | `MIT OR Apache-2.0` | 1.71 | Active again, commits 2026-11-01. Generic over `f64`. Has Oklab, Oklch, Okhsl, `LinSrgb`, `Srgb`, and a blend module. Default features are `named_from_str`, `std`, and `approx`. `phf ^0.13.1` comes through `named`. 0.7.7 dropped `fast-srgb8` and deprecated `encoding::gamma`. Does not forbid `unsafe` | [PALETTE-DEPS], [PALETTE-TOML], [PALETTE-CHANGELOG] |
| `csscolorparser` | 0.9.1, 2026-11-10 | `MIT OR Apache-2.0` | 1.85 | Parses hex, named colours, hsl, oklab, oklch, `color()`, and `calc()`. Depends on `num-traits` (with `libm`), plus `phf` 0.14 through the default feature `named-colors`. 0.9.0 removed deprecated methods | [LIBRS-CSSCOLORPARSER] (secondary), [CSSCOLORPARSER-CHANGELOG] |
| `color` (linebender) | 0.3.3, 2026-05-05 | `Apache-2.0 OR MIT` | 1.82, 1.86 in the next release | No required dependencies. Parses `CSS Color 4`. Has Oklab, Oklch, linear sRGB, P3, Rec2020, and interpolation. Workspace lint `unsafe_code = "deny"`. `peniko`, and so `vello`, depends on it. Whether its components are `f32` only is not verified | [LINEBENDER-COLOUR], [PENIKO-DEPS] |
| `colorgrad` | 0.9.0, 2026-09-28 | `MIT OR Apache-2.0` | 1.85 | Forbids `unsafe`. Gradients on top of `csscolorparser`. Switched to `f32` in 0.7.0 | [COLORGRAD-LIB] |
| `oklab` | 1.1.2, 2024-09-01 | `CC-PDDC` | None stated | Conversion only, stale | None given |

### 8.1. Research Recommendation

The research recommends `palette` with `default-features = false, features = ["std"]` for `f64` colour spaces and gamma-correct blending, and `csscolorparser` for user input. Turning off the `named` feature of `palette` avoids the `phf` 0.13 and 0.14 duplicate (Section 19). If `f32` is acceptable, `color` alone covers parsing and colour spaces, and it matches what `vello` and `peniko` use in a later phase.

## 9. Parsing the Notation

Escher reads symmetries written in Conway's orbifold notation, a short one-line string. The deciding facts are exact error positions, the dependency tree, and stability.

| Crate | Latest | Licence | MSRV | Notes | Sources |
| --- | --- | --- | --- | --- | --- |
| `winnow` | 1.0.5, 2026-11-05 | `MIT` | 1.65 | Stable 1.0, active. Its only dependency is `memchr`, which is optional. Does not forbid `unsafe`. The API for error positions is not verified | [WINNOW-DEPS] |
| `chumsky` | 0.14.1, 2026-11-13. 1.0.0-alpha.8 is stale | `MIT` | 1.65 | Breaking 0.x releases in 2026. The default feature `stacker` pulls in `psm` and `cc`, which need a C toolchain, and `default-features = false` avoids them. Requires `hashbrown` 0.16. Has an `unsafe impl` in `lib.rs`. Moved to Codeberg | [CHUMSKY], [CHUMSKY-CHANGELOG], [STACKER-DEPS] |
| `nom` | 8.0.0, 2025-01-26 | `MIT` | 1.65 | Slow but alive, last commit 2025-08-26. Depends only on `memchr`. 8.0 moved to trait-based combinators | [NOM-CHANGELOG] |
| `logos` | 0.16.1, 2026-01-30 | `MIT OR Apache-2.0` | 1.80 | A lexer only. Has unreleased commits for `syn` 3 | [LOGOS] |
| `ariadne` | 0.6.0, 2025-10-28 | `MIT` | 1.85 | Renders diagnostics. The GitHub repository is archived and moved to Codeberg, with unreleased commits | [ARIADNE] |
| `codespan-reporting` | 0.13.1, 2025-10-22 | `Apache-2.0` | 1.67 | Forbids `unsafe`. Renders diagnostics | [CODESPAN-LIB] |

### 9.1. Research Recommendation

For a notation as small as an orbifold signature, the research recommends a hand-written recursive-descent parser over `char_indices`. It gives exact byte spans, needs no dependency, and leaves the error text fully under the project's control. If combinators are wanted, `winnow` 1.0 is the best library, and the research advises against `chumsky` with default features because of its C build dependency and its frequent breaking releases.

## 10. Error Handling and Diagnostics

Escher needs error types in its libraries, error reporting in its binary, and a way to point at the position of an error in user input. The deciding facts are soundness advisories, maintenance, and the dependency tree.

| Crate | Latest | Licence | MSRV | Notes | Sources |
| --- | --- | --- | --- | --- | --- |
| `thiserror` | 2.0.21, 2026-09-23 | `MIT OR Apache-2.0` | 1.77 | Derive only. Active | [THISERROR] |
| `anyhow` | 1.0.104, 2026-07-18 | `MIT OR Apache-2.0` | 1.68 | No dependencies. `unsafe` internally. RUSTSEC-2026-0190, an unsound `downcast_mut`, is fixed in 1.0.103 and later | [RUSTSEC-2026-0190] |
| `miette` | 7.6.0, 2025-04-27 | `Apache-2.0` | 1.82, though crates.io says 1.70 (Section 20) | Renders spans with `SourceSpan` and `#[label]`. The `fancy` feature adds about 8 crates, including `backtrace`. Low activity, with no functional commit since 2025-09. Open issue #469 reports undefined behaviour in `downcast_mut`. Requires `unicode-width` 0.1 | [MIETTE-TOML], [MIETTE-DOCS], [MIETTE-ISSUES] |
| `snafu` | 0.9.2, 2026-07-21 | `MIT OR Apache-2.0` | 1.65 | Active. No span rendering | [SNAFU] |
| `eyre` and `color-eyre` | 0.6.14, 2026-08-11, and 0.6.5, 2025-05-30 | `MIT OR Apache-2.0` | 1.65 | Active again. RUSTSEC-2024-0021 is fixed. `color-eyre` always pulls in `backtrace`. No span rendering | [RUSTSEC-2024-0021] |

`ariadne` and `codespan-reporting`, which Section 9 lists, render a caret under a position in the input.

### 10.1. Research Recommendation

The research recommends `thiserror` in the library crates and `anyhow` 1.0.103 or later in the binary. For a position in user input, it recommends rendering with `codespan-reporting`, which forbids `unsafe`, or `ariadne`, and notes that a hand-written caret line also suffices because the notation is one line. `miette` is the most integrated option, but its low activity and the open report of undefined behaviour argue against it.

## 11. Testing

Escher needs snapshot tests, property-based tests, floating-point comparisons, and comparisons of rendered images with reference images. The deciding facts are licence, native code, and the version of `rand` that each crate uses.

| Crate | Latest | Licence | MSRV | Notes | Sources |
| --- | --- | --- | --- | --- | --- |
| `insta` and `cargo-insta` | 1.49.0, 2026-10-03 | `Apache-2.0` | 1.66 | Active. 1.48 changed how `--profile` is handled and the precedence of the `CI` environment variable | [INSTA-CHANGELOG] |
| `proptest` | 1.12.0, 2026-10-10 | `MIT OR Apache-2.0` | 1.88 | In "passive maintenance" but releasing. 1.12 moved to `rand` 0.10 | [PROPTEST-CHANGELOG], [PROPTEST-REPO] |
| `quickcheck` | 1.1.0, 2026-02-10 | `Unlicense OR MIT` | 1.85 | First release since 2021. Uses `rand` 0.10 | [QUICKCHECK] |
| `bolero` | 0.13.7, 2026-10-03 | `MIT` | 1.68 | Active. Uses `rand` 0.9. `cargo-bolero` needs `binutils-dev` and `libunwind-dev` | [BOLERO-README] |
| `approx` | 0.5.1, 2022-01-23, with 0.6.0-rc2 from 2026-02-05 | `Apache-2.0` | None stated | Depends on `num-traits`. `nalgebra` uses it | [APPROX] |
| `float-cmp` | 0.10.0, 2024-09-20 | `MIT` | None stated | Margins in units in the last place (ULPs) and epsilon. Last commit 2026-05-02 | [FLOAT-CMP] |
| `assert_float_eq` | 1.2.0, 2025-09-29 | `MIT OR Apache-2.0`, written as `MIT/Apache-2.0` | 1.56 | No dependencies | None given |
| `image-compare` | 0.5.0, 2025-08-18 | `MIT` | None stated | Pulls in `image` 0.25, `rayon`, `itertools`, and `thiserror`. No commits for about 15 months | [IMAGE-COMPARE-DEPS] |
| `dssim-core` | 3.5.1, 2026-08-29 | `AGPL-3.0`, or a commercial licence | None stated | The score scale changes between versions | [DSSIM-REPO] |
| `nv-flip` | 0.1.2, 2023-07-16 | `(MIT OR Apache-2.0 OR Zlib) AND BSD-3-Clause` | 1.64 | Bindings to NVIDIA's FLIP library through `nv-flip-sys`. Needs a native build, assumed to be C++ (unverified) | [NV-FLIP-REPO] |
| `pixelmatch` and `image-diff` | 2021 and 2020 | `MIT`, and `MIT OR Apache-2.0` | None stated | Abandoned | None given |

### 11.1. Research Recommendation

The research recommends `insta`, `proptest` alone so that `rand` stays at 0.10, and `approx` 0.5 or `float-cmp`. For reference images, it recommends decoding with `png` and comparing each channel with a tolerance on the maximum difference and on the count of differing pixels, in about 30 lines of test code. This avoids the AGPL licence of `dssim-core` and the native build of `nv-flip`, and the research would add `image-compare` for its MSSIM metric only if a perceptual metric proves necessary.

## 12. Reference Arithmetic for Tests

Tests of the `f64` computations need results that are known to be more accurate than `f64`, including transcendental functions such as `cosh` and `acosh`. The deciding facts are correct rounding, native code, and licence.

| Crate | Latest | Licence | Native code | Correctly rounded transcendental functions | Sources |
| --- | --- | --- | --- | --- | --- |
| `rug`, with `gmp-mpfr-sys` 1.7.1 | 1.30.0, 2026-04-27, MSRV 1.85 | `LGPL-3.0-or-later` | GMP, MPFR, and MPC, built from bundled sources. Needs `gcc`, `m4`, and `make`. No MSVC support | Yes, through MPFR | [RUG-DOCS], [GMP-MPFR-SYS-DOCS] |
| `astro-float` | 0.9.6, 2026-08-07 | `MIT` | None | Claimed for "all ... mathematical functions", including `sin`, `cos`, `cosh`, `acosh`, `exp`, and `ln` | [ASTRO-FLOAT-DOCS] |
| `dashu-float` | 0.6.2, 2026-10-02, MSRV 1.68 | `MIT OR Apache-2.0` | None | Claimed from 0.6.0 through a Ziv loop. Young: 0.6.1 and 0.6.2 fixed bugs in `ln` and `tanh` | [DASHU-FLOAT-CHANGELOG] |
| `malachite`, with `malachite-float` and `malachite-q` | 0.14.0, 2026-10-13, MSRV 1.90 | `LGPL-3.0-only` | None | Claims results identical to MPFR. Trigonometric functions are "the frontier" and still landing. "API is not stable yet" | [MALACHITE], [MALACHITE-MPFR] |
| `inari` | 2.0.0, 2024-08-07 | `MIT` | Elementary functions need the `gmp` feature, which uses `gmp-mpfr-sys`. Needs Haswell or later x86-64, or AArch64 | Tightest IEEE 1788 intervals | [INARI-REPO], [INARI-CONFORMANCE] |
| `twofloat` and `qd` | 0.8.4 and 0.8.0 | `BSD-3-Clause`, and `MIT` | None | No. `twofloat` is "very preliminary", and `qd` is undocumented | [TWOFLOAT-REPO] |
| `num-rational` and `num-bigint` | 0.4.2, and 0.5.1 with 0.4.8, 2026-07-05 | `MIT OR Apache-2.0` | None | Exact, with no transcendental functions. `num-bigint` 0.4.7 and 0.5.0 were yanked over a regression in division | [NUM-BIGINT-RELEASES] |
| `algebraics` | 0.3.0, 2022 | `LGPL-2.1-or-later` | None | Exact algebraic numbers. Stale | None given |

### 12.1. Research Recommendation

The research recommends `rug` as a dev-dependency if a C toolchain in CI is acceptable, because MPFR is the reference standard. It notes that the LGPL is compatible with `GPL-3.0-or-later`, and that a test-only dependency is not distributed. For pure Rust, it recommends `astro-float`, and because nobody has independently verified its claims of correct rounding, it advises checking a sample against `rug` or MPFR once. For exact rationals it recommends `num-rational` with `num-bigint` 0.4.8 or later, or 0.5.1 or later.

## 13. CPU Rasterization and Vector Output

A later phase draws polygons with borders through a 2D rasterizer and writes vector graphics such as SVG and PDF. The deciding facts are `unsafe`, alignment with the GPU stack of later phases, and shared dependencies such as `skrifa`.

| Crate | Latest | Licence | MSRV | Notes | Sources |
| --- | --- | --- | --- | --- | --- |
| `tiny-skia` | 0.12.0, 2026-02-02 | `BSD-3-Clause` | "stable" | Under linebender, slow but alive, with a push on 2026-08-20. `unsafe` for SIMD intrinsics, with no way to turn it off found. Works in `f32`. 0.12 changed `RadialGradient::new` | [TINY-SKIA-REPO] |
| `vello_cpu` | 0.4.0, 2026-11-11 | `Apache-2.0 OR MIT` | 1.89 | Forbids `unsafe`, as does `vello_common`. Depends on `fearless_simd`, `hashbrown` 0.17, and `peniko` 0.6. Has an `f32` pipeline "for test snapshots". Complex filter graphs panic. Still 0.x | [VELLO-CPU-DOCS], [VELLO-CPU-LIB] |
| `vello` (GPU) | 0.12.0, 2026-11-11 | `Apache-2.0 OR MIT` | 1.89 | Requires `wgpu ^30`. Now "experimental" for compute GPUs | [VELLO-DEPS], [VELLO-README] |
| `vello_gpu`, formerly `vello_hybrid` | 0.4.0, 2026-11-11 | `Apache-2.0 OR MIT` | 1.89 | Requires `wgpu ^30`. `vello_hybrid` 0.2.0 is stale and on wgpu 29 | [VELLO-README] |
| `kurbo` | 0.13.1, 2026-05-13 | `Apache-2.0 OR MIT` | 1.85 | Curves in `f64`. Small tree | [KURBO-CHANGELOG] |
| `lyon` | 1.0.19, 2026-03-08 | `MIT OR Apache-2.0` | None stated | Tessellation, maintained at a low rate | None given |
| `resvg` and `usvg` | 0.50.0, 2026-11-11 | `Apache-2.0 OR MIT` | 1.85 | Under linebender. Forbids `unsafe`. Can rasterize Escher's own SVG output in tests | [RESVG-CHANGELOG] |
| `svg` | 0.18.0, 2024-09-27 | `Apache-2.0 OR MIT` | None stated | No dependencies | None given |
| `pdf-writer` | 0.15.0, 2026-05-27 | `MIT OR Apache-2.0` | None stated | Forbids `unsafe`. Low level, from the typst project | [PDF-WRITER-LIB] |
| `krilla` | 0.8.2, 2026-06-04 | `MIT OR Apache-2.0` | 1.92 | High-level PDF. Forbids `unsafe`. Uses `skrifa` 0.42, while `vello` and `usvg` use 0.49 | [KRILLA-REPO] |
| `printpdf` and `lopdf` | 0.12.9 and 0.47.0 | `MIT` | 1.88 | `lopdf` had RUSTSEC-2026-0187, for versions up to 0.41 | [RUSTSEC-2026-0187] |

### 13.1. Research Recommendation

The research recommends `kurbo` for geometry and `vello_cpu` for CPU rasterization, because `vello_cpu` forbids `unsafe`, uses linebender types that match `vello` on wgpu 30, and offers an `f32` pipeline for snapshots. For SVG it recommends writing the text directly or using `svg`. For PDF it recommends `pdf-writer`, and `krilla` only if its `skrifa` duplicate is accepted (Section 19).

## 14. Group Theory

Escher works with the symmetry groups that orbifold signatures name, which can call for coset enumeration (Todd-Coxeter), permutation groups, and low-index subgroup search. The deciding facts are whether a maintained crate exists, and its licence.

### 14.1. Rust Crates

| Crate | Latest | Licence | Facts | Sources |
| --- | --- | --- | --- | --- |
| `algebraeon-groups` | 0.0.17, 2026-03-06, repository pushed 2026-10-06 | `GPL-3.0-only` | Todd-Coxeter in the HLT style with no lookahead, `Permutation`, and the conversion `into_finite_group`. Does not halt on infinite groups. No Schreier-Sims and no low-index search | [ALGEBRAEON-TC] |
| `pcgroup` | 0.2.0 | `MIT` | Finite polycyclic groups only | None given |
| `use-coxeter` | Not recorded | Not recorded | "does not implement Coxeter group operations" | None given |
| `vole` | 0.1.1 | `MPL-2.0` | Stale | None given |
| `knuth-bendix` | 0.1.0 | Not recorded | A placeholder | None given |
| `symbolica` | Not recorded | Proprietary, source-available | Incompatible with Escher's licence | [SYMBOLICA-LICENCE] |

The crate names `coxeter`, `todd-coxeter`, `permutation_group`, and `groupoid` do not exist on crates.io, and the API returns HTTP 404 for `coxeter` [COXETER-404]. The Schreier-Sims repositories found on GitHub have no licence (no URL given).

### 14.2. No Crate for Coset Enumeration or Low-Index Subgroups

No maintained Rust crate provides complete coset enumeration or low-index subgroup search. The only Todd-Coxeter implementation found is the basic one in `algebraeon-groups` (Section 14.1), which does not halt on infinite groups and has no low-index search [ALGEBRAEON-TC]. No crate for low-index subgroups or automatic groups exists (no URL given).

### 14.3. External Tools

These tools are not Rust crates, but can serve as oracles that generate test fixtures offline:

| Tool | Latest | Licence | Facts | Sources |
| --- | --- | --- | --- | --- |
| GAP | 4.16.1, 2026-08-23 | `GPL-2.0-or-later` | Rust bindings in `gap-sys` 0.2.4, whose bindings are `MIT`. How `gap-sys` links GAP is not known | [GAP-SYS] |
| libsemigroups | 3.6.2, 2026-11-12 | GPL version 3, with `-only` or `-or-later` not verified | Todd-Coxeter, Knuth-Bendix, and Sims low-index search. No Rust bindings | [LIBSEMIGROUPS-README] |
| `low_index` | 1.3, 2026-06-12 | `GPL-2.0-or-later` | C++ and Python | [LOW-INDEX-README] |
| ACE | Not recorded | Not stated | Written in C | None given |

### 14.4. Research Recommendation

The research recommends that Escher implement the pieces it needs itself, because orbifold groups need only small Todd-Coxeter runs or explicit generators. It recommends GAP or libsemigroups, run offline, to generate test fixtures. `algebraeon-groups` has a compatible licence, but its API is at 0.0.x and, as a `GPL-3.0-only` dependency, it would fix the combined work at GPL version 3.

## 15. SIMD

This section is a survey for later phases. The deciding facts are whether a stable Rust option exists, and whether it keeps Escher's own code free of `unsafe`.

| Option | Latest | Licence | MSRV | Facts | Sources |
| --- | --- | --- | --- | --- | --- |
| `std::simd` | Not applicable | Not applicable | Nightly only | Tracking issue #86656 is open and blocked on portable-simd issue #364, as of 2026-08 | [RUST-86656], [PORTABLE-SIMD-364] |
| `fearless_simd` | 1.2.0, 2026-10-25 | `Apache-2.0 OR MIT` | 1.89 | Safe API with dispatch at run time. Has `f64x2`, `f64x4`, and `f64x8`. No dependencies. `vello_cpu` already depends on it | [FEARLESS-SIMD-DOCS] |
| `pulp` | 0.22.3, 2026-06-20 | `MIT` | None stated | Safe dispatch at run time and complex SIMD types. Depends on `num-complex`, `bytemuck`, and `raw-cpuid` | [PULP-DOCS] |
| `wide` | 1.7.1, 2026-09-14 | `Zlib OR Apache-2.0 OR MIT` | 1.89 or 1.90 | Detects CPU features at build time only | [WIDE-REPO] |
| `multiversion` | 0.9.2, 2026-10-22 | `MIT OR Apache-2.0` | None stated | A macro that clones functions per CPU feature, not a library of SIMD types. Incompatible with `wide` | None given |

### 15.1. Research Recommendation

The research recommends deferring SIMD. If SIMD is needed, it recommends `fearless_simd`, because it keeps Escher's code free of `unsafe` and is already a dependency of `vello_cpu`.

## 16. GUI Stack

This section is a survey for the development phase, which uses Slint for the GUI and wgpu for rendering. The deciding facts are the Slint licence that a `GPL-3.0-or-later` application can use, and how Slint and wgpu versions fit together.

### 16.1. Slint Release

The current Slint release is 1.18.1, from 2026-09-21, with MSRV 1.92 [SLINT]. The master branch states 1.20.0 with MSRV 1.95, which is unreleased [SLINT-CHANGELOG], [SLINT-TOML]. Section 20 records the conflict.

On Linux, Slint's default renderers are FemtoVG, which uses OpenGL, and a software renderer. The Skia renderer is a heavy native build. Slint needs the system libraries xcb and xkbcommon, which a build for Wayland only does not need, and fontconfig [SLINT-BUILDING].

### 16.2. Slint Licensing Options

Slint is licensed under `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` [SLINT-DOCS]. Each option means something different for Escher:

| Option | Terms | Meaning for a `GPL-3.0-or-later` application | Sources |
| --- | --- | --- | --- |
| `GPL-3.0-only` | Free on all platforms, including embedded. "the work as a whole must be licensed under the GPL" | The combined application is effectively distributed under GPL version 3 (inferred from the GNU FAQ's rule on combined works). Escher's own source can keep its `-or-later` grant | [SLINT-FAQ], [GPL-FAQ] |
| `LicenseRef-Slint-Royalty-free-2.0` | Covers desktop, mobile, and web, and excludes embedded systems. Requires attribution through the `AboutSlint` widget or a public badge. Forbids applications that re-expose Slint's APIs. Aimed at proprietary applications | Nothing found bars a GPL application from choosing it. The pricing page says "Attribution to Slint is optional", which conflicts with the licence text (Section 20) | [SLINT-RF2], [SLINT-PRICING] |
| `LicenseRef-Slint-Software-3.0` | A paid licence, with royalties per embedded device | Relevant only if Escher ships on embedded devices (inferred) | None given |

In `deny.toml`, allowing `GPL-3.0-only` satisfies Slint's OR expression. cargo-deny can also allow the `LicenseRef-*` identifiers by name [CARGO-DENY-LICENSES].

### 16.3. wgpu Release and Slint Integration

wgpu 30.0.2, from 2026-11-12, is the current release. It is licensed `MIT OR Apache-2.0`, has a library MSRV of 1.87, and loads Vulkan at run time through `ash` and `libloading` [WGPU-CHANGELOG]. Its last two major releases changed these APIs:

- 30.0.0 (2026-07-01): `VertexState.buffers` takes `Option` values, integer interpolation is no longer flat by default, `BufferBinding` uses `TryFrom`, and `SurfaceConfiguration.color_space` is new.
- 29.0.0 (2026-03-18): `CurrentSurfaceTexture`, new constructors of `InstanceDescriptor`, and `WriteOnly` mapping.

Slint 1.18.1 has the features `unstable-wgpu-29` and `unstable-wgpu-30`, which "may be removed or changed in future minor releases" [SLINT-WGPU30]. Its API includes `slint::wgpu_30`, `BackendSelector::require_wgpu_30()`, `GraphicsAPI::WGPU30` in `set_rendering_notifier`, `Image::try_from(wgpu::Texture)`, and `Image::to_wgpu_30_texture()`. An application uses Slint's own `Device` and `Queue` [SLINT-WGPU30].

### 16.4. Research Recommendation

The research recommends Slint 1.18.1 under `GPL-3.0-only`, with its `unstable-wgpu-30` feature and wgpu 30.0.2. With this pairing, Slint, `vello` 0.12, and `vello_gpu` 0.4 all use wgpu 30 (Section 19).

## 17. Lean 4

This section is a survey for machine-checked proofs, which the project uses for results that no cited source proves. The deciding facts are how to install Lean safely, how to lay out a Lean project beside the Rust code, and which parts of the project's mathematics Mathlib covers.

### 17.1. Toolchain

The stable Lean release is v4.35.0, from 2026-10-12. v4.36.0-rc2 followed on 2026-10-27, and minor releases arrive about monthly [LEAN4-RELEASES]. The Mathlib tag v4.35.0 is from 2026-10-13 [MATHLIB-RELEASE]. Lean 4 and Mathlib are licensed `Apache-2.0`, which Apache states is compatible with GPL version 3 [APACHE-GPL].

### 17.2. Installing Lean

The official Lean installer performs no checksum verification. The documented installation runs the elan installer script, and the elan release publishes no checksum or signature files. elan v4.2.4 is from 2026-08-25 [ELAN-INIT], [ELAN-RELEASE]. This is the command as the research records it:

```sh
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh
```

The GitHub API does expose a SHA-256 digest for each release asset, which a Containerfile can pin [ELAN-RELEASE].

Debian's `elan` package is not suitable. It is version 3.1.1 on trixie, from the Lean 3 era, and 4.1.2 on sid [DEBIAN-ELAN].

### 17.3. Project Layout

A Lean project pins its toolchain with a `lean-toolchain` file. It adds Mathlib with a `[[require]]` entry that sets `name = "mathlib"` and `scope = "leanprover-community"` in `lakefile.toml`, then runs `lake exe cache get` [MATHLIB-DEPENDENCY]. The cache lives in `~/.cache/mathlib` and takes about 4 GB per version according to forum reports (secondary), and no figure from 2026 was found.

AWS keeps the Lean code of `cedar-spec` in a `cedar-lean/` subdirectory beside its Rust code, and its CI installs elan directly [CEDAR-SPEC]. `leanprover/lean-action` supports the inputs `lake-package-directory` and `use-mathlib-cache` [LEAN-ACTION]. The research concludes that a `lean/` subdirectory with its own `lakefile.toml` and `lean-toolchain` is the established pattern (inferred).

### 17.4. Mathlib Coverage

Mathlib covers these parts of the project's mathematics [MATHLIB-UHP-METRIC], [MATHLIB-COXETER]:

- The upper half-plane, in `Mathlib/Analysis/Complex/UpperHalfPlane/`, with the files `Basic`, `Metric`, `MoebiusAction`, `FixedPoints`, and others.
- The hyperbolic distance `dist` on the upper half-plane, defined as `2 * arsinh (...)`.
- Actions of $\mathrm{GL}(2,\mathbb{R})$ and $\mathrm{SL}(2,\mathbb{R})$ on the upper half-plane, with transitivity, and the fact that $\mathrm{SL}(2,\mathbb{R})$ acts by isometries.
- `ModularGroup.fd`, the fundamental domain of the modular group, in `Mathlib.NumberTheory.Modular`.
- Coxeter systems, in `Mathlib/GroupTheory/Coxeter/` with the files `Basic`, `Length`, `Inversion`, and `Matrix`: `CoxeterMatrix`, `CoxeterSystem`, length, reflections, and inversion sequences.
- Presented groups, in `Mathlib.GroupTheory.PresentedGroup`.

Mathlib does not cover these parts [MATHLIB-UHP-METRIC], [MATHLIB-COXETER]:

- A hyperbolic metric on `Complex.UnitDisc`, which has only `UnitDisc/Basic.lean`, so Mathlib has no metric for the Poincare disc model. Automorphisms of the disc were proposed in January 2026, and whether the proposal merged is unknown.
- Fuchsian groups.
- The exchange and deletion conditions and Matsumoto's theorem, which Mathlib lists as TODO. Mathlib also has no non-trivial instance of `CoxeterSystem`.
- The spherical and hyperbolic laws of cosines.
- Todd-Coxeter coset enumeration.

### 17.5. Research Recommendation

The research recommends Lean v4.35.0 installed through elan. Section 17.2 states what the installer does not verify, and Section 17.4 states what Mathlib lacks.

## 18. Licences in the Dependency Tree

### 18.1. Compatibility

This table lists the licences that the candidates would bring into the dependency tree, and whether each is compatible with distributing Escher under `GPL-3.0-or-later`:

| Licence | Brought in by | Compatible | Basis |
| --- | --- | --- | --- |
| `MIT OR Apache-2.0`, `Apache-2.0 OR MIT` | `png`, `clap`, `rayon`, `glam`, `num-complex`, `palette`, `csscolorparser`, `thiserror`, `anyhow`, `proptest`, `kurbo`, `vello_cpu`, `wgpu`, `euclid` | Yes | Either licence can be chosen. Apache states that `Apache-2.0` is compatible with GPL version 3 [APACHE-GPL] |
| `MIT` | `winnow`, `ariadne`, `lexopt`, `astro-float`, `pdf-writer` (as one of two licences) | Yes | General knowledge of the Free Software Foundation (FSF) licence list, which was not fetched |
| `Apache-2.0` only | `insta`, `approx`, `codespan-reporting`, and `nalgebra` or `miette` if chosen | Yes with GPL version 3, not with GPL version 2 | [APACHE-GPL] |
| `BSD-3-Clause` | `tiny-skia`, `argh` | Yes | General knowledge, not fetched |
| `Zlib`, inside OR expressions | `wide`, `zune-png`, `ultraviolet` | Yes | `MIT` or `Apache-2.0` can be chosen instead |
| `BSD-3-Clause OR Apache-2.0` | `moxcms`, only through `image` | Yes | `Apache-2.0` can be chosen [APACHE-GPL] |
| `Unlicense OR MIT` | `quickcheck` | Yes | `MIT` can be chosen |
| `GPL-3.0-only OR LicenseRef-Slint-*` | Slint | Yes, under `GPL-3.0-only` | The combined binary is effectively GPL version 3 (Section 16.2) |
| `LGPL-3.0-or-later`, `LGPL-3.0-only`, `LGPL-2.1-or-later` | `rug` with `gmp-mpfr-sys`, `malachite`, `algebraics` | Yes | Dev-only use avoids the obligations of distribution |
| `GPL-3.0-only` | `algebraeon-groups` | Yes | Fixes the combined work at GPL version 3 |
| `MPL-2.0` | `vole` | Yes | General knowledge |
| `AGPL-3.0` | `dssim-core` | The research advises avoiding it | Section 13 of GPL version 3 permits the combination, but adds the network clause of the AGPL |
| `CC-PDDC` | `oklab` | The research advises avoiding it | Public domain, but how cargo-deny treats this licence is uncertain |
| Proprietary | `symbolica` | No | [SYMBOLICA-LICENCE] |

`proc-macro2` and `syn` pull in `unicode-ident`, which recent versions declare as `(MIT OR Apache-2.0) AND Unicode-3.0` (general knowledge, unverified). The project should therefore expect to allow `Unicode-3.0`, which the research states is compatible with the GPL.

### 18.2. Changes to `deny.toml`

When a change adds a dependency, it adds to the `allow` list of `deny.toml` each licence that the new dependency brings into the tree (Section 3). For Slint, allowing `GPL-3.0-only` satisfies its OR expression (Section 16.2).

The exact list comes from running cargo-deny after the dependencies are added, not from the table in Section 18.1. This command reports every licence that the allow-list does not cover:

```sh
cargo deny check licenses
```

## 19. Duplicate-Version Risks

Each row predicts a duplicate version that `multiple-versions = "deny"` would reject. Because nobody ran `cargo tree`, every prediction is unverified (Section 2.2).

| Crate | Versions | Brought in by | Mitigation the research names | Sources |
| --- | --- | --- | --- | --- |
| `syn` | 2 and 3 | `clap_derive` 4.6.7 requires `syn ^3.0.2`. The `syn` versions of `thiserror-impl`, `palette_derive`, `nalgebra-macros`, and `insta` were not checked, and any on `syn` 2 duplicates `syn`, `proc-macro2`, and `quote` | Add a `skip` or `skip-tree` entry in `deny.toml`, or check the `syn` version of each crate. This is likely the first `multiple-versions` failure | [CLAP-DERIVE-DEPS] |
| `phf` | 0.13 and 0.14 | The `named` feature of `palette` 0.7.7 uses `^0.13.1`, and the `named-colors` feature of `csscolorparser` 0.9 uses 0.14 | Disable one of the two features. Because of `all-features = true`, cargo-deny sees optional features too | [PALETTE-DEPS], [LIBRS-CSSCOLORPARSER] (secondary) |
| `miniz_oxide` | 0.8 and 0.9 | `png` 0.18.1 depends on 0.8 directly, and `flate2` 1.1.11 reportedly uses 0.9 (secondary, unverified) | Pin `flate2`, or wait for a `png` release from master, which has removed the direct dependency | [PNG-DEPS], [LIBRS-FLATE2] (secondary) |
| `rand` | 0.9 and 0.10 | `bolero` 0.13.7 uses 0.9, while `proptest` 1.12 and `quickcheck` 1.1 use 0.10 | Use `proptest` only | [BOLERO-README], [PROPTEST-CHANGELOG], [QUICKCHECK] |
| `unicode-width` | 0.1 and 0.2 | `miette` 7.6 uses 0.1, while `ariadne` and the optional dependency of `clap` use 0.2 | Prefer `codespan-reporting` or `ariadne` to `miette` | [MIETTE-TOML] |
| `hashbrown` | 0.16 and 0.17 | `chumsky` 0.14 uses 0.16, and `vello_cpu` 0.4 uses 0.17. The versions that wgpu and Slint use were not checked | None named. Section 9.1 advises against `chumsky` for other reasons | [CHUMSKY-CHANGELOG], [VELLO-CPU-DOCS] |
| `skrifa` | 0.42 and 0.49 | `krilla` 0.8.2 uses 0.42, while `vello` 0.12 and `usvg` 0.50 use 0.49 | Prefer `pdf-writer`, or accept the duplicate | [KRILLA-REPO] |

These candidates were checked and are consistent with each other:

- wgpu: version 30 across Slint with `unstable-wgpu-30`, `vello` 0.12, and `vello_gpu` 0.4. The research advises avoiding `vello_hybrid` 0.2, which uses wgpu 29 and `vello_common` 0.2 [VELLO-README], [SLINT-WGPU30].
- `kurbo` and `peniko`: `kurbo` 0.13 and `peniko` 0.6 across `vello`, `vello_cpu`, and `usvg` [VELLO-DEPS], [VELLO-CPU-DOCS].
- `image`: `image-compare` pulls `image` 0.25 and `png` 0.18 into the dev-dependencies, which is compatible with a direct dependency on `png` 0.18 [IMAGE-COMPARE-DEPS].

These commands show the resolved duplicates and run every cargo-deny check, after the candidates are added to a manifest:

```sh
cargo tree -d
cargo deny check
```

## 20. Conflicts Between Sources

The research resolved only the Slint conflict and left the others open:

| Topic | One source | The other source | Handling |
| --- | --- | --- | --- |
| Slint version | crates.io and GitHub releases show 1.18.1, 2026-09-21, MSRV 1.92 [SLINT] | The master `Cargo.toml` shows 1.20.0 with MSRV 1.95, while the master CHANGELOG lists 1.19.0 as unreleased [SLINT-TOML], [SLINT-CHANGELOG] | 1.18.1 is current |
| Slint royalty-free attribution | The licence text and the terms page require attribution [SLINT-RF2] | The pricing page says "Attribution to Slint is optional", and the column it applies to is unclear [SLINT-PRICING] | Unresolved |
| `rayon` 1.12.0 MSRV | crates.io says 1.80 [RAYON-VERSIONS] | The main `Cargo.toml` says 1.85 | Unresolved. Both are below 1.99 |
| `png` MSRV | crates.io says 1.73 for 0.18.1 [PNG-DEPS] | Master says 1.81 [PNG-LIB] | Unresolved. Both are below 1.99 |
| `image` MSRV | crates.io says 1.88 for 0.25.10 [IMAGE-DEPS] | `CHANGES.md` lists the bump to 1.88 as unreleased | Unresolved. Both are below 1.99 |
| `miette` MSRV | crates.io says 1.70 | The repository says 1.82 [MIETTE-TOML] | Unresolved. Both are below 1.99 |
| `ultraviolet` default features | The master `Cargo.toml` has `f64` on by default | docs.rs says it is off by default | Unresolved |

## 21. Open Questions and Unverified Claims

The research rates its confidence as medium. Versions, dates, and licences come from registry and repository data and are reliable. Transitive dependency sizes, `unsafe` audits beyond `lib.rs`, the duplicate-version predictions, and several claims about capabilities were not verified with `cargo tree` or by reading the source.

These questions are open:

1. The exact resolved trees and duplicates, especially for `syn` 2 and 3, `miniz_oxide`, and `hashbrown`. `cargo tree -d` and `cargo deny check` answer them once the candidates are in a manifest (Section 19).
2. Whether `png` writes cICP. No setter was found, and the last 6.6 KB of `encoder.rs` was not read.
3. Whether the linebender `color` crate stores components as `f32` only, and which colour spaces `colorgrad` can blend in.
4. The API of `winnow` for error positions, the `forbid_unsafe` feature of `logos`, and the changelogs of `snafu` 0.9 and `clap` 4.6.
5. An independent check of the claims of correct rounding by `astro-float`, `dashu-float`, and `malachite`.
6. The build requirements of `nv-flip-sys`, how `gap-sys` links GAP, whether libsemigroups is `GPL-3.0-only` or `GPL-3.0-or-later`, and the licence of the C code of ACE.
7. Whether the Mathlib pull request #33368 on automorphisms of the unit disc has merged, and the current size of the Mathlib cache.
8. The `syn` versions that `thiserror-impl`, `palette_derive`, and `insta` use.

These claims rest on general knowledge, a secondary source, or an inference, and no primary source cited here establishes them:

- `flate2` 1.1.11 defaults to `miniz_oxide` 0.9 (secondary, unverified).
- `unicode-ident` is licensed `(MIT OR Apache-2.0) AND Unicode-3.0`, so `Unicode-3.0` will need to be allowed (general knowledge, unverified).
- `MIT`, `BSD-3-Clause`, and `MPL-2.0` are compatible with `GPL-3.0-or-later` (general knowledge, the FSF licence list was not fetched).
- A `GPL-3.0-or-later` application combined with `GPL-3.0-only` Slint is effectively distributed under GPL version 3 (inferred from [GPL-FAQ]).
- `nv-flip-sys` needs a C++ build (assumed, unverified).
- The `zune-png`, `mtpng`, `csscolorparser`, and `lexopt` MSRV facts, which come from lib.rs (secondary).
- The Mathlib cache takes about 4 GB per version (forum reports, secondary).
- A `lean/` subdirectory is the established layout (inferred from [CEDAR-SPEC] and [LEAN-ACTION]).

The research gave no URL for the facts in these rows and sentences: `ultraviolet`, `mint`, `orx-parallel`, `oklab`, `anyhow` apart from its advisory, `eyre` and `color-eyre` apart from the advisory, `assert_float_eq`, `pixelmatch` and `image-diff`, `qd`, `algebraics`, `lyon`, `svg`, `printpdf` and `lopdf` apart from the advisory, `pcgroup`, `use-coxeter`, `vole`, `knuth-bendix`, `multiversion`, the GAP version, ACE, the unlicensed Schreier-Sims repositories, the absence of a low-index crate, `LicenseRef-Slint-Software-3.0`, and the `rayon`, `image`, and `ultraviolet` sides of the conflicts in Section 20.

The research could not read these sources, or read them only in part:

- The GitHub commits API, which returned HTTP 403 for `rayon`, `glam`, `num-complex`, `chili`, `wide`, `inari`, `twofloat`, `qd`, `num-bigint`, and `num-rational`, so their last-commit dates are missing.
- The GitLab commits page of `rug`.
- lib.rs pages that returned placeholders.
- The documentation of `lodepng::State` (HTTP 404).
- The CHANGELOGs of `vello` and `krilla` (HTTP 404).
- The changelog of `lyon` (HTTP 404).
- The CHANGELOGs of `miette` and `malachite` beyond their first entries, which were too long.
- The text of GPL version 3 on gnu.org (HTTP 429).
- The dependencies of `nv-flip-sys` (HTTP 404).
- The page on the release cadence of Lean at lean-lang.org (HTTP 404).
- The licence page at symbolica.io, which was not fetched.
- The documentation of `UnitDisc/Basic` beyond 100,000 characters.
- The file `encoder.rs` of `png` beyond 100,000 characters.
- The end of `lib.rs` of `chumsky`.

## 22. References

- [ALGEBRAEON-TC] docs.rs, "algebraeon_groups::free_group::todd_coxeter", <https://docs.rs/algebraeon-groups/latest/algebraeon_groups/free_group/todd_coxeter/index.html>, accessed 2026-11-15.
- [APACHE-GPL] apache.org, "GPL compatibility", <https://www.apache.org/licenses/GPL-compatibility.html>, accessed 2026-11-15.
- [APPROX] crates.io, "approx" (API record), <https://crates.io/api/v1/crates/approx>, accessed 2026-11-15.
- [ARGH] crates.io, "argh" (API record), <https://crates.io/api/v1/crates/argh>, accessed 2026-11-15.
- [ARIADNE] zesterer, "ariadne" (repository), <https://codeberg.org/zesterer/ariadne>, accessed 2026-11-15.
- [ASTRO-FLOAT-DOCS] docs.rs, "astro_float 0.9.6", <https://docs.rs/astro-float/0.9.6/astro_float/>, accessed 2026-11-15.
- [BOLERO-README] camshaft, "bolero README.md", <https://raw.githubusercontent.com/camshaft/bolero/master/README.md>, accessed 2026-11-15.
- [BPAF] crates.io, "bpaf" (API record), <https://crates.io/api/v1/crates/bpaf>, accessed 2026-11-15.
- [CARGO-DENY-LICENSES] Embark Studios, "cargo-deny: licenses configuration", <https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html>, accessed 2026-11-15.
- [CEDAR-SPEC] cedar-policy, "cedar-spec" (repository), <https://github.com/cedar-policy/cedar-spec>, accessed 2026-11-15.
- [CHILI] crates.io, "chili versions" (API record), <https://crates.io/api/v1/crates/chili/versions>, accessed 2026-11-15.
- [CHUMSKY] crates.io, "chumsky" (API record), <https://crates.io/api/v1/crates/chumsky>, accessed 2026-11-15.
- [CHUMSKY-CHANGELOG] zesterer, "chumsky CHANGELOG.md", <https://codeberg.org/zesterer/chumsky/raw/branch/main/CHANGELOG.md>, accessed 2026-11-15.
- [CLAP] crates.io, "clap" (API record), <https://crates.io/api/v1/crates/clap>, accessed 2026-11-15.
- [CLAP-BUILDER-LIB] clap-rs, "clap_builder src/lib.rs", <https://raw.githubusercontent.com/clap-rs/clap/master/clap_builder/src/lib.rs>, accessed 2026-11-15.
- [CLAP-DERIVE-DEPS] crates.io, "clap_derive 4.6.7 dependencies" (API record), <https://crates.io/api/v1/crates/clap_derive/4.6.7/dependencies>, accessed 2026-11-15.
- [CODESPAN-LIB] brendanzab, "codespan-reporting src/lib.rs", <https://raw.githubusercontent.com/brendanzab/codespan/master/codespan-reporting/src/lib.rs>, accessed 2026-11-15.
- [COLORGRAD-LIB] mazznoer, "colorgrad-rs src/lib.rs", <https://raw.githubusercontent.com/mazznoer/colorgrad-rs/master/src/lib.rs>, accessed 2026-11-15.
- [COXETER-404] crates.io, "coxeter" (API record, HTTP 404), <https://crates.io/api/v1/crates/coxeter>, accessed 2026-11-15.
- [CSSCOLORPARSER-CHANGELOG] mazznoer, "csscolorparser-rs CHANGELOG.md", <https://raw.githubusercontent.com/mazznoer/csscolorparser-rs/master/CHANGELOG.md>, accessed 2026-11-15.
- [DASHU-FLOAT-CHANGELOG] cmpute, "dashu float/CHANGELOG.md", <https://raw.githubusercontent.com/cmpute/dashu/master/float/CHANGELOG.md>, accessed 2026-11-15.
- [DEBIAN-ELAN] Debian, "Package: elan", <https://packages.debian.org/stable/elan>, accessed 2026-11-15.
- [DSSIM-REPO] kornelski, "dssim" (repository), <https://github.com/kornelski/dssim>, accessed 2026-11-15.
- [ELAN-INIT] elan.lean-lang.org, "elan-init.sh", <https://elan.lean-lang.org/elan-init.sh>, accessed 2026-11-15.
- [ELAN-RELEASE] leanprover, "elan latest release" (GitHub API record), <https://api.github.com/repos/leanprover/elan/releases/latest>, accessed 2026-11-15.
- [FEARLESS-SIMD-DOCS] docs.rs, "fearless_simd", <https://docs.rs/fearless_simd/latest/fearless_simd/>, accessed 2026-11-15.
- [FLOAT-CMP] crates.io, "float-cmp" (API record), <https://crates.io/api/v1/crates/float-cmp>, accessed 2026-11-15.
- [FORTE] crates.io, "forte versions" (API record), <https://crates.io/api/v1/crates/forte/versions>, accessed 2026-11-15.
- [GAP-SYS] crates.io, "gap-sys" (API record), <https://crates.io/api/v1/crates/gap-sys>, accessed 2026-11-15.
- [GLAM-CHANGELOG] bitshifter, "glam-rs CHANGELOG.md", <https://raw.githubusercontent.com/bitshifter/glam-rs/main/CHANGELOG.md>, accessed 2026-11-15.
- [GLAM-README] bitshifter, "glam-rs README.md", <https://raw.githubusercontent.com/bitshifter/glam-rs/main/README.md>, accessed 2026-11-15.
- [GMP-MPFR-SYS-DOCS] docs.rs, "gmp_mpfr_sys", <https://docs.rs/gmp-mpfr-sys/latest/gmp_mpfr_sys/>, accessed 2026-11-15.
- [GPL-FAQ] gnu.org, "GPL FAQ", <https://www.gnu.org/licenses/gpl-faq.html>, accessed 2026-11-15.
- [IMAGE-COMPARE-DEPS] crates.io, "image-compare 0.5.0 dependencies" (API record), <https://crates.io/api/v1/crates/image-compare/0.5.0/dependencies>, accessed 2026-11-15.
- [IMAGE-DEPS] crates.io, "image 0.25.10 dependencies" (API record), <https://crates.io/api/v1/crates/image/0.25.10/dependencies>, accessed 2026-11-15.
- [IMAGE-PNGENC] docs.rs, "image::codecs::png::PngEncoder", <https://docs.rs/image/latest/image/codecs/png/struct.PngEncoder.html>, accessed 2026-11-15.
- [IMAGE-PNGRS] image-rs, "image src/codecs/png.rs", <https://raw.githubusercontent.com/image-rs/image/main/src/codecs/png.rs>, accessed 2026-11-15.
- [INARI-CONFORMANCE] docs.rs, "inari 2.0.0 conformance", <https://docs.rs/inari/2.0.0/inari/_docs/conformance/index.html>, accessed 2026-11-15.
- [INARI-REPO] unageek, "inari" (repository), <https://github.com/unageek/inari>, accessed 2026-11-15.
- [INSTA-CHANGELOG] mitsuhiko, "insta CHANGELOG.md", <https://raw.githubusercontent.com/mitsuhiko/insta/master/CHANGELOG.md>, accessed 2026-11-15.
- [KRILLA-REPO] LaurenzV, "krilla" (repository), <https://github.com/LaurenzV/krilla>, accessed 2026-11-15.
- [KURBO-CHANGELOG] linebender, "kurbo CHANGELOG.md", <https://raw.githubusercontent.com/linebender/kurbo/main/CHANGELOG.md>, accessed 2026-11-15.
- [LEAN-ACTION] leanprover, "lean-action" (repository), <https://github.com/leanprover/lean-action>, accessed 2026-11-15.
- [LEAN4-RELEASES] leanprover, "lean4 releases" (GitHub API record), <https://api.github.com/repos/leanprover/lean4/releases>, accessed 2026-11-15.
- [LEXOPT-LIB] blyxxyz, "lexopt src/lib.rs", <https://raw.githubusercontent.com/blyxxyz/lexopt/master/src/lib.rs>, accessed 2026-11-15.
- [LIBRS-CSSCOLORPARSER] lib.rs, "csscolorparser" (secondary), <https://lib.rs/crates/csscolorparser>, accessed 2026-11-15.
- [LIBRS-FLATE2] lib.rs, "flate2" (secondary), <https://lib.rs/crates/flate2>, accessed 2026-11-15.
- [LIBRS-MTPNG] lib.rs, "mtpng" (secondary), <https://lib.rs/crates/mtpng>, accessed 2026-11-15.
- [LIBRS-ZUNE-PNG] lib.rs, "zune-png" (secondary), <https://lib.rs/crates/zune-png>, accessed 2026-11-15.
- [LIBSEMIGROUPS-README] libsemigroups, "libsemigroups README.md", <https://raw.githubusercontent.com/libsemigroups/libsemigroups/main/README.md>, accessed 2026-11-15.
- [LINEBENDER-COLOUR] linebender, "`color` crate src/lib.rs", `https://raw.githubusercontent.com/linebender/color/main/color/src/lib.rs`, accessed 2026-11-15.
- [LODEPNG-TOML] kornelski, "lodepng-rust Cargo.toml", <https://raw.githubusercontent.com/kornelski/lodepng-rust/main/Cargo.toml>, accessed 2026-11-15.
- [LOGOS] crates.io, "logos" (API record), <https://crates.io/api/v1/crates/logos>, accessed 2026-11-15.
- [LOW-INDEX-README] 3-manifolds, "low_index README.rst", <https://raw.githubusercontent.com/3-manifolds/low_index/master/README.rst>, accessed 2026-11-15.
- [MALACHITE] Malachite, "Malachite", <https://www.malachite.rs/>, accessed 2026-11-15.
- [MALACHITE-MPFR] Malachite, "MPFR floats mapping", <https://www.malachite.rs/mapping/mpfr-floats/>, accessed 2026-11-15.
- [MATHLIB-COXETER] leanprover-community, "Mathlib/GroupTheory/Coxeter/Basic.lean", <https://raw.githubusercontent.com/leanprover-community/mathlib4/master/Mathlib/GroupTheory/Coxeter/Basic.lean>, accessed 2026-11-15.
- [MATHLIB-DEPENDENCY] leanprover-community, "Using mathlib4 as a dependency" (wiki), <https://github.com/leanprover-community/mathlib4/wiki/Using-mathlib4-as-a-dependency>, accessed 2026-11-15.
- [MATHLIB-RELEASE] leanprover-community, "mathlib4 latest release" (GitHub API record), <https://api.github.com/repos/leanprover-community/mathlib4/releases/latest>, accessed 2026-11-15.
- [MATHLIB-UHP-METRIC] leanprover-community, "Mathlib.Analysis.Complex.UpperHalfPlane.Metric" (documentation), <https://leanprover-community.github.io/mathlib4_docs/Mathlib/Analysis/Complex/UpperHalfPlane/Metric.html>, accessed 2026-11-15.
- [MIETTE-DOCS] docs.rs, "miette 7.6.0", <https://docs.rs/miette/7.6.0/miette/>, accessed 2026-11-15.
- [MIETTE-ISSUES] zkat, "miette issues" (GitHub API record), <https://api.github.com/repos/zkat/miette/issues>, accessed 2026-11-15.
- [MIETTE-TOML] zkat, "miette Cargo.toml", <https://raw.githubusercontent.com/zkat/miette/main/Cargo.toml>, accessed 2026-11-15.
- [NALGEBRA-CHANGELOG] dimforge, "nalgebra CHANGELOG.md", <https://raw.githubusercontent.com/dimforge/nalgebra/main/CHANGELOG.md>, accessed 2026-11-15.
- [NALGEBRA-DEPS] crates.io, "nalgebra 0.35.0 dependencies" (API record), <https://crates.io/api/v1/crates/nalgebra/0.35.0/dependencies>, accessed 2026-11-15.
- [NOM-CHANGELOG] rust-bakery, "nom CHANGELOG.md", <https://raw.githubusercontent.com/rust-bakery/nom/main/CHANGELOG.md>, accessed 2026-11-15.
- [NUM-BIGINT-RELEASES] rust-num, "num-bigint RELEASES.md", <https://raw.githubusercontent.com/rust-num/num-bigint/master/RELEASES.md>, accessed 2026-11-15.
- [NUM-COMPLEX-TOML] rust-num, "num-complex Cargo.toml", <https://raw.githubusercontent.com/rust-num/num-complex/master/Cargo.toml>, accessed 2026-11-15.
- [NV-FLIP-REPO] gfx-rs, "nv-flip-rs" (repository), <https://github.com/gfx-rs/nv-flip-rs>, accessed 2026-11-15.
- [PALETTE-CHANGELOG] Ogeon, "palette CHANGELOG.md", <https://raw.githubusercontent.com/Ogeon/palette/master/CHANGELOG.md>, accessed 2026-11-15.
- [PALETTE-DEPS] crates.io, "palette 0.7.7 dependencies" (API record), <https://crates.io/api/v1/crates/palette/0.7.7/dependencies>, accessed 2026-11-15.
- [PALETTE-TOML] Ogeon, "palette Cargo.toml", <https://raw.githubusercontent.com/Ogeon/palette/master/palette/Cargo.toml>, accessed 2026-11-15.
- [PDF-WRITER-LIB] typst, "pdf-writer src/lib.rs", <https://raw.githubusercontent.com/typst/pdf-writer/main/src/lib.rs>, accessed 2026-11-15.
- [PENIKO-DEPS] crates.io, "peniko 0.6.1 dependencies" (API record), <https://crates.io/api/v1/crates/peniko/0.6.1/dependencies>, accessed 2026-11-15.
- [PICO-ARGS] crates.io, "pico-args" (API record), <https://crates.io/api/v1/crates/pico-args>, accessed 2026-11-15.
- [PNG-CHANGES] image-rs, "image-png CHANGES.md", <https://raw.githubusercontent.com/image-rs/image-png/master/CHANGES.md>, accessed 2026-11-15.
- [PNG-DEPS] crates.io, "png 0.18.1 dependencies" (API record), <https://crates.io/api/v1/crates/png/0.18.1/dependencies>, accessed 2026-11-15.
- [PNG-ENC] docs.rs, "png::Encoder", <https://docs.rs/png/latest/png/struct.Encoder.html>, accessed 2026-11-15.
- [PNG-LIB] image-rs, "image-png src/lib.rs", <https://raw.githubusercontent.com/image-rs/image-png/master/src/lib.rs>, accessed 2026-11-15.
- [PORTABLE-SIMD-364] rust-lang, "portable-simd issue 364", <https://github.com/rust-lang/portable-simd/issues/364>, accessed 2026-11-15.
- [PROPTEST-CHANGELOG] proptest-rs, "proptest CHANGELOG.md", <https://raw.githubusercontent.com/proptest-rs/proptest/main/proptest/CHANGELOG.md>, accessed 2026-11-15.
- [PROPTEST-REPO] proptest-rs, "proptest" (repository), <https://github.com/proptest-rs/proptest>, accessed 2026-11-15.
- [PULP-DOCS] docs.rs, "pulp", <https://docs.rs/pulp/latest/pulp/>, accessed 2026-11-15.
- [QUICKCHECK] crates.io, "quickcheck" (API record), <https://crates.io/api/v1/crates/quickcheck>, accessed 2026-11-15.
- [RAYON-RELEASES] rayon-rs, "rayon RELEASES.md", <https://raw.githubusercontent.com/rayon-rs/rayon/main/RELEASES.md>, accessed 2026-11-15.
- [RAYON-VERSIONS] crates.io, "rayon versions" (API record), <https://crates.io/api/v1/crates/rayon/versions>, accessed 2026-11-15.
- [RESVG-CHANGELOG] linebender, "resvg CHANGELOG.md", <https://raw.githubusercontent.com/linebender/resvg/main/CHANGELOG.md>, accessed 2026-11-15.
- [RUG-DOCS] docs.rs, "rug 1.30.0", <https://docs.rs/rug/1.30.0/rug/>, accessed 2026-11-15.
- [RUST-86656] rust-lang, "rust issue 86656", <https://github.com/rust-lang/rust/issues/86656>, accessed 2026-11-15.
- [RUSTSEC-2024-0021] RustSec, "RUSTSEC-2024-0021", <https://rustsec.org/advisories/RUSTSEC-2024-0021.html>, accessed 2026-11-15.
- [RUSTSEC-2026-0187] RustSec, "RUSTSEC-2026-0187", <https://rustsec.org/advisories/RUSTSEC-2026-0187.html>, accessed 2026-11-15.
- [RUSTSEC-2026-0190] RustSec, "RUSTSEC-2026-0190", <https://rustsec.org/advisories/RUSTSEC-2026-0190.html>, accessed 2026-11-15.
- [RUSTSEC-2026-0196] RustSec, "RUSTSEC-2026-0196", <https://rustsec.org/advisories/RUSTSEC-2026-0196>, accessed 2026-11-15.
- [RUSTSEC-2026-0197] RustSec, "RUSTSEC-2026-0197", <https://rustsec.org/advisories/RUSTSEC-2026-0197>, accessed 2026-11-15.
- [SLINT] crates.io, "slint" (API record), <https://crates.io/api/v1/crates/slint>, accessed 2026-11-15.
- [SLINT-BUILDING] slint-ui, "slint docs/building.md", <https://raw.githubusercontent.com/slint-ui/slint/master/docs/building.md>, accessed 2026-11-15.
- [SLINT-CHANGELOG] slint-ui, "slint CHANGELOG.md", <https://raw.githubusercontent.com/slint-ui/slint/master/CHANGELOG.md>, accessed 2026-11-15.
- [SLINT-CORE-TOML] slint-ui, "slint internal/core/Cargo.toml", <https://raw.githubusercontent.com/slint-ui/slint/master/internal/core/Cargo.toml>, accessed 2026-11-15.
- [SLINT-DOCS] docs.rs, "slint", <https://docs.rs/slint/latest/slint/>, accessed 2026-11-15.
- [SLINT-FAQ] slint-ui, "slint FAQ.md", <https://raw.githubusercontent.com/slint-ui/slint/master/FAQ.md>, accessed 2026-11-15.
- [SLINT-PRICING] slint.dev, "Pricing", <https://slint.dev/pricing>, accessed 2026-11-15.
- [SLINT-RF2] slint-ui, "LicenseRef-Slint-Royalty-free-2.0", <https://raw.githubusercontent.com/slint-ui/slint/master/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md>, accessed 2026-11-15.
- [SLINT-TOML] slint-ui, "slint Cargo.toml", <https://raw.githubusercontent.com/slint-ui/slint/master/Cargo.toml>, accessed 2026-11-15.
- [SLINT-WGPU30] docs.rs, "slint::wgpu_30", <https://docs.rs/slint/latest/slint/wgpu_30/index.html>, accessed 2026-11-15.
- [SNAFU] crates.io, "snafu" (API record), <https://crates.io/api/v1/crates/snafu>, accessed 2026-11-15.
- [STACKER-DEPS] crates.io, "stacker 0.1.25 dependencies" (API record), <https://crates.io/api/v1/crates/stacker/0.1.25/dependencies>, accessed 2026-11-15.
- [STD-AP] The Rust Project, "std::thread::available_parallelism", <https://doc.rust-lang.org/std/thread/fn.available_parallelism.html>, accessed 2026-11-15.
- [STD-SCOPE] The Rust Project, "std::thread::scope", <https://doc.rust-lang.org/std/thread/fn.scope.html>, accessed 2026-11-15.
- [SYMBOLICA-LICENCE] symbolica-dev, "symbolica License.md", <https://raw.githubusercontent.com/symbolica-dev/symbolica/master/License.md>, accessed 2026-11-15.
- [THISERROR] crates.io, "thiserror" (API record), <https://crates.io/api/v1/crates/thiserror>, accessed 2026-11-15.
- [TINY-SKIA-REPO] linebender, "tiny-skia" (repository), <https://github.com/linebender/tiny-skia>, accessed 2026-11-15.
- [TWOFLOAT-REPO] ajtribick, "twofloat" (repository), <https://github.com/ajtribick/twofloat>, accessed 2026-11-15.
- [VELLO-CPU-DOCS] docs.rs, "vello_cpu 0.4.0", <https://docs.rs/vello_cpu/0.4.0/vello_cpu/>, accessed 2026-11-15.
- [VELLO-CPU-LIB] linebender, "vello vello_cpu/src/lib.rs", <https://raw.githubusercontent.com/linebender/vello/main/vello_cpu/src/lib.rs>, accessed 2026-11-15.
- [VELLO-DEPS] crates.io, "vello 0.12.0 dependencies" (API record), <https://crates.io/api/v1/crates/vello/0.12.0/dependencies>, accessed 2026-11-15.
- [VELLO-README] linebender, "vello README.md", <https://raw.githubusercontent.com/linebender/vello/main/README.md>, accessed 2026-11-15.
- [WGPU-CHANGELOG] gfx-rs, "wgpu CHANGELOG.md", <https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/CHANGELOG.md>, accessed 2026-11-15.
- [WGPU-TOML] gfx-rs, "wgpu Cargo.toml", <https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/Cargo.toml>, accessed 2026-11-15.
- [WIDE-REPO] Lokathor, "wide" (repository), <https://github.com/Lokathor/wide>, accessed 2026-11-15.
- [WINNOW-DEPS] crates.io, "winnow 1.0.5 dependencies" (API record), <https://crates.io/api/v1/crates/winnow/1.0.5/dependencies>, accessed 2026-11-15.
