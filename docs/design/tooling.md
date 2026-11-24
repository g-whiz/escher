# Tooling Design

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-24 |
| Applies to | The repository's configuration, recipes, hooks, and continuous integration |

## Abstract

This document specifies how the deterministic checks of the Escher repository are configured and run. It defines the `just` recipes, the aggregate check that runs before every merge, the nextest profile, and the git hooks. It also specifies the custom checkers that move rules from review to a tool and the continuous integration workflow that runs all of these checks.

## 1. Introduction

### 1.1. Problem

The coding standard specifies a strict lint configuration and a set of tools, and the writing guide specifies rules for documentation. Until a tool is installed and run, every rule it would enforce is a judgement rule that a reviewer checks by hand. The repository's central principle forbids that wherever a tool can do the work.

### 1.2. Purpose

This document turns the specifications of the standards documents into installed configuration and runnable checks. It follows the principle that Hard Rule 4 of `AGENTS.md` and Section 3.5 of the writing guide state: anything a deterministic tool can check is checked by a tool and never by a reviewer.

### 1.3. Audience

The primary readers are the coding agents that add or change a recipe, a checker, a hook, a tool configuration, or a continuous integration (CI) job. The secondary readers are reviewers, and the owner of the repository, who approves its decisions.

### 1.4. Scope

This document covers the tool configuration files, the workspace manifest fields that are not lints, the recipes, the nextest configuration, the hooks, the custom checkers, the CI workflow, the cargo-deny policy, publishing readiness, and the requirements on the container image.

### 1.5. Non-Goals

Each fact has one owner, as Section 4.5 of the writing guide requires, and this document cites the owner instead of restating the fact.

- Section 4 of `docs/rust-coding-standards.md` owns the lint tables, the contents of `clippy.toml`, the list of tools with what each one checks, and the exact cargo command lines of Section 4.8. This document says which recipe runs which command, and when.
- `AGENTS.md` owns the hard rules and the development workflow.

This document does not design the application programming interface (API) of any crate. It does not specify the contents of the container image beyond the requirements of Section 10, because the image is defined outside this repository.

## 2. Requirements Language

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.1. Terminology

This document uses the terms that these documents define, and calls them the writing guide and the coding standard:

- Section 2.2 of `docs/writing-documentation.md`, such as agent, reviewer, enforced rule, and judgement rule.
- Section 2.2 of `docs/rust-coding-standards.md`, such as lint, workspace lint table, and the check.

It adds these terms:

- action: A reusable unit of steps for a GitHub Actions workflow, published in a git repository and referenced from a workflow.
- aggregate recipe: A recipe that only runs other recipes, such as `check`.
- commit SHA: The full 40-character hexadecimal hash that names one git commit.
- container image: The image, defined outside this repository, from which the development container runs.
- custom checker: A script in this repository that checks one rule of the writing guide, the coding standard, or a design document that no installed tool checks (Section 6).
- development container: The container in which agents work on this repository, run from the container image.
- GitHub Actions: The CI service of GitHub.
- hook: A git hook that pre-commit installs from `.pre-commit-config.yaml` and runs at a git event, such as a commit.
- hosted runner: A runner that GitHub provides.
- image rebuild: A rebuild of the container image from its definition.
- job: A set of steps in a workflow that runs on one runner.
- matrix: A job definition that runs once for each value of a list, such as each recipe.
- nextest profile: A named set of nextest settings in `.config/nextest.toml`, which a recipe selects with the option `--profile`.
- recipe: A named command in the root `justfile`, run with `just <recipe>`.
- recipe group: The group that the `[group]` attribute gives a recipe, under which `just --list` shows it.
- runner: The machine that runs a job.
- Software Package Data Exchange (SPDX) licence identifier: The standard short name of a licence, such as `GPL-3.0-or-later`, which Cargo's `license` field takes.
- workflow: A YAML Ain't Markup Language (YAML) file under `.github/workflows/` that defines the jobs that GitHub Actions runs.

## 3. Recipes

### 3.1. Rules

The `justfile` stays one flat file with `[group]` attributes, a help comment on every recipe, and shell that is `shellcheck`-clean, as Hard Rule 8 of `AGENTS.md` requires. It has three groups:

- `checks`: The checks, their self-tests, and `hooks-install`.
- `rust`: The recipes that change files or run tests and are not checks.
- `git`: `merge` and `worktree-prune`.

The recipe `default` has no group. Section 14.1 lists every recipe that takes parameters.

TOOL-6. An agent MUST add each recipe of Section 3.2 with the name, the group, and the membership of `check` that the table gives, and MUST update the table in the same change when one of them has to differ. Agents, hooks, and CI call recipes by name, and a recipe that differs from this table breaks every caller that follows it.

TOOL-7. A recipe that runs a command of Section 4.8 of the coding standard MUST run that command as Section 4.8 gives it, and MUST add only these two things:

1. The option `--profile` with a nextest profile of Section 4.
2. The arguments that Section 4.8 names for the recipe `coverage`.

The recipe `update-golden` is the one exception. It MUST run the command of `test` narrowed to the golden image test of `escher-render`, with `-p escher-render` in place of `--workspace`, that test as the filter of `-E`, and `ESCHER_UPDATE_GOLDEN=1` in its environment, because its purpose is to rewrite the golden files, and a failure of an unrelated test would make it fail after it has written them. It is not part of `check` and verifies nothing, so the narrowing loses no check.

An agent that has to change the command MUST change Section 4.8 in the same change. Two command lines for one check diverge, and an agent that runs the documented command then gets a result different from the recipe's.

The commands of Section 4.8 that run Clippy, build the documentation, run the doctests, run the tests, and measure coverage pass `--workspace` and `--all-features`, so each of these checks covers every member and the code of every Cargo feature. The coverage report alone leaves out the test support crate `escher-reference` (TEST-15 of the coding standard). Only the recipe `update-golden` narrows one of these commands, under the exception of TOOL-7. `check-features` builds each combination of features separately, with the feature powerset of cargo hack.

The formatting command of Section 4.8 ends with the list of every Rust file that git tracks or does not ignore. `fmt` and `check-fmt` compute that list with `git ls-files`, and it is part of the command, not an addition.

TOOL-8. The recipe `check` MUST pass in the development container without a hardware graphics processing unit (GPU) and without a display, and MUST NOT run a recipe that needs either. An agent runs `check` before every merge in the container, as the Validating Changes section of `AGENTS.md` requires, and CI runs it on a hosted runner that is not required to provide either.

TOOL-9. An agent MUST NOT add a recipe for a tool that the container image does not provide until an image rebuild adds the tool, because the recipe would fail `check` for every agent until then.

TOOL-30. `taplo.toml` MUST set `indent_string = "    "` in its `[formatting]` table and `exclude = ["target/**", ".worktrees/**"]`. `typos.toml` MUST accept each correct word that typos reports in the repository.

Otherwise `check-toml` and `check-typos` fail on files that are correct. taplo indents with two spaces by default, which disagrees with the four spaces that `.editorconfig` sets for `*.toml`. It does not read `.gitignore`, so without the exclusions it also checks the build directory and every worktree.

`typos.toml` accepts correct text in three forms: patterns, identifiers, and words. An agent adds each new entry in the narrowest form that accepts the correct text, because every wider form stops typos from checking more of the repository.

The three patterns are the entries of the `extend-ignore-re` list in the `[default]` table:

- The first matches the full name of a cited author in `docs/rust-references.md`, whose surname typos reports. The surname is correct only inside that name.
- The second matches the id of a colouring in a `Colouring` line of `escher info`, in a snapshot of `escher-cli`, from `Colouring: ` to the comma after the id. The letters of an id count in base 26 and are not words, so typos reports some of them.
- The third matches the id of an arrangement in an `Arrangement` line of `escher info` in the same way, from `Arrangement: ` to the comma after the id, because an arrangement id has the form of a colouring id.

typos skips only the text that a pattern matches, so it still checks the same letters everywhere else.

The `[default.extend-words]` table accepts words: the surname of a cited author, words of cited titles in French and Latin, a journal abbreviation, and part of the name of a cited crate and of a compression format. These words recur in many places, such as the research documents under `docs/research/` and their citation tags, in both lower and upper case. One entry accepts every occurrence, because typos accepts a word of this table wherever it occurs as a word, in any case.

The `[default.extend-identifiers]` table accepts identifiers: mathematical notation, a relator of a cited presentation written as letters, names quoted from cited code and data, and a fragment of the shell name pattern in the shebang checks. Each is a short token that is correct only as the exact standalone notation, relator, or name. typos accepts it only when a whole identifier matches, so it still checks the same letters inside another identifier.

One identifier is a two-letter fragment that occurs in two unrelated places. It is part of the regular expression that matches the shell names in the shebang checks of `scripts/justfile-recipes` and `scripts/check-shell`, and it occurs among the permutation words that the research documents quote from data files. The fragment is correct as a standalone token in each place, so the configuration accepts it as an identifier.

A document that describes this configuration describes the accepted text instead of spelling it out, because typos checks that document too. typos reads its configuration from `typos.toml`, `_typos.toml`, or `.typos.toml`, and the repository uses the first name.

No file is excluded from typos.

`typos.toml` sets no locale. typos cannot enforce Canadian spelling, because with its default settings it accepts both `colour` and `color`, so `scripts/check-prose` and its word list keep that job (Section 8 of the writing guide).

The repository has no `rustfmt.toml`, because Section 4.8 of the coding standard specifies the default rustfmt settings.

### 3.2. Recipe Table

This table lists every recipe, what it runs, and whether `check` runs it. The commands of Section 4.8 of the coding standard are named by their purpose, and Section 4.8 gives each command line.

| Recipe | Group | Runs | In `check` |
| --- | --- | --- | --- |
| `default` | none | `just --list --unsorted` | No |
| `check` | `checks` | The recipes that Section 3.3 lists | Not applicable |
| `check-prose` | `checks` | `scripts/check-prose`, extended to API documentation by Section 6.6 | Yes |
| `check-prose-self-test` | `checks` | The self-test of Section 6.1 for `scripts/check-prose` | Yes |
| `check-shell` | `checks` | `scripts/check-shell` | Yes |
| `check-commit-message` | `checks` | `scripts/check-commit-message` on one file, for the `commit-msg` hook | No |
| `check-commit-message-self-test` | `checks` | The self-test of the commit message checker | Yes |
| `hooks-install` | `checks` | `pre-commit install` | No |
| `merge` | `git` | `scripts/merge-branch`, which squash-merges a branch onto `main` with the subject and body of every commit of the branch in the message, pushes `main`, and deletes the branch on `origin` when its tip there is part of the squash (Section 14.7) | No |
| `worktree-prune` | `git` | `scripts/worktree-prune` | No |
| `fmt` | `rust` | `cargo fmt --all` with the list of every Rust file that git tracks or does not ignore after `--`, which applies the formatting that the formatting command of Section 4.8 checks, also to the files that a crate includes with `include!`, and `taplo fmt`, which formats every Tom's Obvious Minimal Language (TOML) file that `taplo.toml` does not exclude. It lists the files with `git ls-files -z --cached --others --exclude-standard -- '*.rs'` and keeps those that exist. It fails with "fmt: git ls-files failed, so the Rust files that a crate pulls in with include! cannot be listed." when `git ls-files` fails, and with "fmt: git ls-files listed no Rust file, so the Rust files that a crate pulls in with include! cannot be listed." when the list is empty | No |
| `check-rust` | `checks` | The recipes that Section 3.3 lists | Yes |
| `check-fmt` | `checks` | The formatting command of Section 4.8, with the list of files that `fmt` computes in the same way. It fails with the messages of `fmt`, which start with "check-fmt:" instead, when `git ls-files` fails or lists no Rust file | Yes, through `check-rust` |
| `check-clippy` | `checks` | The Clippy command of Section 4.8 | Yes, through `check-rust` |
| `check-doc` | `checks` | The documentation command of Section 4.8 | Yes, through `check-rust` |
| `check-doctest` | `checks` | The doctest command of Section 4.8 | Yes, through `check-rust` |
| `test` | `rust` | The nextest command of Section 4.8 with the `default` profile | Yes, through `check-rust` |
| `update-golden` | `rust` | Only the golden image test of `escher-render`, `images::golden_images_match_the_committed_files`, through the command of `test` with `-p escher-render` in place of `--workspace` and that test as the filter of `-E`, with `ESCHER_UPDATE_GOLDEN=1` in its environment, so that the test writes each image it renders over its committed PNG file (Section 14.9) | No |
| `readme-images` | `rust` | `scripts/readme-images`, which builds `escher` in release mode and runs each `escher render` and `escher render-masks` command that `README.md` shows, with its options, in `docs/images/`, so that each image there is the file that its command writes. Three `render` commands read masks that the script wrote as image fills: the polygon masks, the face mask of an arrangement with one picture, and the two face masks of an arrangement with two pictures. The script then removes the masks that `README.md` does not show | No |
| `check-deny` | `checks` | The cargo-deny command of Section 4.8 | Yes, through `check-rust` |
| `check-rust-comments` | `checks` | The comment checker of Section 6.2 | Yes, through `check-rust` |
| `check-doc-headings` | `checks` | The heading checker of Section 6.3 | Yes, through `check-rust` |
| `check-doctest-attributes` | `checks` | The doctest attribute checker of Section 6.4 | Yes, through `check-rust` |
| `check-feature-docs` | `checks` | The feature documentation checker of Section 6.5 | Yes, through `check-rust` |
| `check-rust-comments-self-test`, `check-doc-headings-self-test`, `check-doctest-attributes-self-test`, `check-feature-docs-self-test` | `checks` | The self-test of Section 6.1 for each checker of Sections 6.2 to 6.5 | Yes |
| `check-workflow-pins` | `checks` | The workflow pin checker of Section 6.8 | Yes |
| `check-workflow-pins-self-test` | `checks` | The self-test of Section 6.1 for the workflow pin checker | Yes |
| `check-doc-references` | `checks` | The document reference checker of Section 6.9 | Yes, directly after `check-workflow-pins` |
| `check-doc-references-self-test` | `checks` | The self-test of Section 6.1 for the document reference checker | Yes |
| `check-typos` | `checks` | The typos command of Section 4.8 on the whole repository, with `typos.toml` (TOOL-30) | Yes |
| `check-toml` | `checks` | The taplo command of Section 4.8, with `taplo.toml` (TOOL-30) | Yes |
| `check-workflows` | `checks` | The actionlint command of Section 4.8, which checks every workflow | Yes |
| `check-unused-deps` | `checks` | The cargo machete command of Section 4.8 | Yes, through `check-rust` |
| `check-features` | `checks` | The cargo hack command of Section 4.8 for the feature powerset, which includes each single feature | No, the CI job `ci-only` runs it |
| `check-msrv` | `checks` | The MSRV command of Section 4.8, which checks the workspace with `cargo hack check` and the toolchain that `rust-version` names | No, the CI job `ci-only` runs it, and its first local run downloads that toolchain (Section 14.3) |
| `coverage` | `rust` | The cargo llvm-cov command of Section 4.8 with the `default` profile and the arguments that Section 4.8 names for it, which write the LCOV report `target/lcov.info`. It prints no coverage summary, and it is not a gate | No, the CI job `coverage` runs it (Section 7.1) |

### 3.3. Aggregate Recipes

`check` runs these recipes, in this order:

1. `check-prose`
2. `check-shell`
3. `check-commit-message-self-test`
4. `check-prose-self-test`
5. `check-rust-comments-self-test`
6. `check-doc-headings-self-test`
7. `check-doctest-attributes-self-test`
8. `check-feature-docs-self-test`
9. `check-workflow-pins-self-test`
10. `check-doc-references-self-test`
11. `check-typos`
12. `check-toml`
13. `check-workflows`
14. `check-workflow-pins`
15. `check-doc-references`
16. `check-rust`

`check-workflows` can run in `check` only while the repository has a workflow, because actionlint exits with status 3 when it finds none. `check` needs network access (Section 14.6).

`check-rust` runs `check-rust-comments`, `check-doc-headings`, `check-doctest-attributes`, `check-feature-docs`, `check-fmt`, `check-unused-deps`, `check-deny`, `check-clippy`, `check-doc`, `check-doctest`, and `test`, in that order, so that the checks that do not compile the workspace report first.

## 4. nextest Profiles

nextest reads the file `.config/nextest.toml`, and a recipe selects a profile of it with the option `--profile` (TOOL-7).

The file defines one profile, `default`, which is the profile of `test` and `coverage`:

```toml
[profile.default]
retries = 0
```

The profile does not set `default-filter`, so it selects every test of the workspace. A filter that the command line adds with `-E` is intersected with the default filter of the profile, so it narrows the selection of the profile and never widens it. Only the recipe `update-golden` adds such a filter (TOOL-7).

TOOL-11. A profile MUST NOT retry a failed test, because a retry turns an intermittent failure into a pass, which hides the defect that caused the failure.

Zero retries is the default of nextest, and the `default` profile sets `retries = 0` explicitly so that the configuration states this rule instead of relying on the default.

TOOL-12. A recipe that runs nextest MUST pass when its profile selects no test. An empty selection is not a test failure, and a workspace can hold no test. nextest 0.9.151 fails such a run with the message "error: no tests to run" and exit status 4.

Every nextest command of Section 4.8 of the coding standard therefore passes `--no-tests=warn`, with which an empty run passes and nextest still prints that no test ran. A recipe keeps that option.

nextest does not run doctests, so `check-doctest` runs them, as Section 4.8 of the coding standard states.

The root `Cargo.toml` sets the optimization levels of the `dev` profile, from which the `test` profile of Cargo inherits. The crates of the workspace build with `opt-level = 1`, because the tests render whole images, which is slow without optimization. Every dependency builds with `opt-level = 0`, which keeps a clean build short. The exception is the dev-dependency `astro-float-num`, at `opt-level = 3` for the reason that Section 11 of `docs/design/geometry.md` states.

## 5. Git Hooks

The hooks in `.pre-commit-config.yaml` call `just` recipes, because Hard Rule 8 of `AGENTS.md` makes every check a recipe.

TOOL-13. A hook of the `pre-commit` stage MUST run only checks that do not compile the workspace. A commit hook that compiles the workspace makes every commit wait for a build, and Clippy, the tests, and the documentation build already run in `check` before every merge and in CI.

These local hooks run in the `pre-commit` stage:

| Hook | Runs when |
| --- | --- |
| `check-prose` | A staged Markdown or Rust file changes. It passes the paths of those files to the recipe |
| `check-shell` | The `justfile` or a file under `scripts/` changes |
| `check-fmt` | A staged Rust file changes |
| `check-typos` | Every commit |
| `check-toml` | A staged TOML file changes |
| `check-workflows` | A staged file under `.github/` changes |
| `check-workflow-pins` | A staged file under `.github/` changes |
| `check-doc-references` | A staged file that the checker reads changes, which is a `.rs`, `.toml`, `.yml`, `.yaml`, or `.json` file, the `justfile`, or a file under `scripts/`. It also runs when a Markdown file under `docs/` changes, because a document can define a new rule identifier (Section 6.9) |

Each of these hooks except `check-prose` runs its recipe on the whole repository, not on the staged files only. The `pre-commit` stage also runs hooks of the repository `pre-commit-hooks`, pinned to a commit SHA, which check YAML and JSON syntax, file endings, trailing whitespace, merge conflict markers, private keys, and large files. The `commit-msg` hook runs `check-commit-message`, and the `post-merge` hook runs `worktree-prune`.

The Claude Code hook in `.claude/settings.json` runs `scripts/claude-hook-prose` after every edit. The script runs `check-prose` on the edited file when the file is inside the repository and its name ends in `.md` or `.rs`, and ignores every other file. It reports the result in one of two ways:

- When `check-prose` reports findings, the script prints them and exits with status 2, which Claude Code treats as a blocking error and reports with the findings.
- When `check-prose` fails for another reason, such as a Rust file that cannot be lexed because it is only partly edited (Section 6.1), the script prints the message and exits with status 1, which Claude Code treats as a non-blocking error.

The allow list of `.claude/settings.json` lets Claude Code run these commands without asking, each as `just <recipe>` unless shown otherwise:

- `just --list` and `just check`.
- `check-rust` and each recipe that it runs.
- Each custom checker and its self-test.
- `check-shell`, `check-commit-message` and its self-test, `check-typos`, `check-toml`, `check-workflows`, and `check-features`.
- `shellcheck` and `git status`.

Claude Code asks before it runs any other recipe, such as `fmt`, `merge`, `check-msrv`, or `coverage`. `just check` and `just check-rust` still run those that they include without asking, because the allow list holds those two commands.

## 6. Custom Checkers

Several rules of the writing guide and the pinning rule of Section 7.2 can be checked deterministically, but no installed tool checks them. actionlint, for example, does not check the pinning rule.

This section specifies a small custom checker for each. The "Checked by" statements of Section 7 of the writing guide, its Section 7.11, and Section 7.2 of this document name these checkers.

### 6.1. Conventions

TOOL-14. Each custom checker MUST be a script under `scripts/` with a recipe of the same name in the `checks` group. It MUST follow the conventions that the usage text of `scripts/check-prose` states, which are:

- One `path:line: message` line for each finding.
- Exit status 0 without findings, 1 with findings, and 2 when a file cannot be read or the setup is broken, after checking every readable file.
- Usage text on `-h`.

With no paths, the checker checks every relevant file in the git repository, tracked or not ignored. A checker that differs breaks the hooks and the recipes, which rely on these exit statuses to tell a finding from a broken setup.

The one departure is a checker that needs an input to know what to look for, such as the project documents of Section 6.9. It exits with status 2 before it checks any file when that input is missing or unusable.

A file that is not valid UTF-8 is a file that cannot be read, so a checker reports it on standard error and exits with status 2. The exception is a Markdown file given to `check-prose`, in which an invalid byte is a finding.

A Rust file that ends inside a string literal, a raw string literal, a block comment, or a character literal also cannot be read. For a checker that reads documentation comments, the same holds for a file that ends inside an attribute, because such a checker skips attributes to group the comments of one item. The checker prints `<checker>: cannot read <file>: unterminated <kind> at line N` on standard error, where N is the line of the opening delimiter, and exits with status 2.

Unterminated input is a read error and not a silent end of the file. A lexer that read the rest of the file as part of the literal would report nothing after it, and the check would pass while it checked nothing there.

Other files given in the same run are still checked. While a file is only partly edited, unterminated input is normal, and Section 5 says how the Claude Code hook reports it.

Each checker prints every finding and every error message through the helper `emit` of `scripts/rust_lexer.py`, which escapes every character outside printable ASCII, both in file paths and in file content that a message repeats. The escapes are these:

- A newline, a carriage return, and a tab print as `\n`, `\r`, and `\t`.
- Every other character up to U+00FF prints as `\xNN`.
- Every character above U+00FF prints as `\uNNNN`, or as `\UNNNNNNNN` above U+FFFF.

A checked file or a file name therefore cannot send a terminal control sequence to the terminal that shows the output (Section 13). A backslash prints unchanged, so a character that the helper escapes and the same escape written literally in a file print alike.

A checker reports a failed git call and a file that cannot be read in its own words, which do not depend on the version of git or on the operating system. The self-tests pin the exact text of each error and would otherwise fail on another machine for an unrelated reason.

A failed git call reads `not inside a git repository`, `git <subcommand> exited with status N`, or `cannot run git: <cause>`, and never repeats the message of git. A read error names its cause as `is a directory`, `no such file`, `not a directory`, or `permission denied`, and only a cause outside these four uses the description that the operating system gives. When `cargo metadata` fails, `check-feature-docs` still repeats the first error line of Cargo (Section 6.5).

The checkers are written in Python 3 and run with `python3 -I`, as `scripts/check-prose` and `scripts/check-commit-message` are, because the container provides Python 3 and those checkers set the pattern.

The checkers that read Rust share the lexer `scripts/rust_lexer.py`. It splits Rust source into comments, literals, and other tokens, groups `///` and `//!` comments into the Markdown text that rustdoc reads, and finds fenced code blocks in that text.

The lexer skips a shebang line at the start of a file, also when a byte order mark precedes it, and never reads it as a comment. A first line that starts with `#![` is an inner attribute and not a shebang line. `check-workflow-pins` uses only the helpers of the lexer module for finding and reading files.

TOOL-15. Each custom checker MUST have a self-test recipe named after it with the suffix `-self-test`, which `check` runs. The recipe MUST run the checker on at least one input that it accepts and one that it rejects for each rule of the checker, and MUST fail unless the checker gives exactly the expected result.

A checker that silently stops finding anything passes `check` while it enforces nothing. The inputs MUST be stored under a name that no checker or other tool reads as a real file, because a failing input stored under its real name would be reported by the checker itself.

The self-test recipes of the custom checkers, including `check-prose-self-test`, run `scripts/self-test-checker` with the name of the checker. Each case is a directory `scripts/fixtures/<checker>/<case>/`. Its input files carry the suffix `.fixture`, which the runner removes when it copies them into a new git repository in a temporary directory, so that the checkers, which select files by name, never read a fixture as a real file.

The runner replaces placeholders while it copies, so that no committed file holds an unfinished-work marker, a carriage return, an escape character, a bell character, a byte order mark, an invalid byte, or a missing final newline. It runs the checker in that repository with the arguments in the file `arguments` of the case, in which it replaces the same placeholders, or with none. A case that holds a file named `no-git-repository` gets no git repository, and the checker runs in a temporary directory outside any repository, which tests the error for a missing repository.

A case passes when all three of these hold:

- The standard output equals the file `expected-output`.
- The standard error equals the file `expected-errors`, or is empty when the case has no such file.
- The exit status equals the number in the file `expected-status`. Without that file, the expected status is 1 when `expected-output` is not empty and 0 when it is.

The usage text of `scripts/self-test-checker` lists the placeholders. `check-commit-message-self-test` keeps its own inline form.

### 6.2. Comment Checker

This table summarizes the comment checker.

| Property | Value |
| --- | --- |
| Script and recipe | `scripts/check-rust-comments`, `check-rust-comments` |
| Rule | Section 7.2 of the writing guide, and Hard Rule 5 of `AGENTS.md` |
| Reads | Every `.rs` file |
| Reports | Every comment other than a `///` comment, a `//!` comment, or a safety comment, and every comment of any kind that contains the word `TODO` or `FIXME` |

A safety comment is a run of `//` line comments on consecutive lines, with nothing else between them, whose first line starts with `// SAFETY:`, as the illustration in Section 7.6 of the writing guide shows. A line comment that starts with `SAFETY:` in another case or spacing is reported as a safety comment that does not start with `// SAFETY:`.

The checker reports every block comment, including `/** */` and `/*! */`, and every line comment that starts with four slashes. rustdoc and `missing_docs` treat `/** */` and `/*! */` as documentation, but the writing guide defines API documentation as `///` and `//!` comments only (Section 2.2 of the writing guide).

`////` and `/*** */` are not documentation comments, so they are other comments, which Section 7.2 of the writing guide forbids. Whether a safety comment sits directly above an `unsafe` block is checked by the safety comment lints of Section 4.5 of the coding standard, not by this checker.

TOOL-16. The comment checker MUST find comments by reading Rust tokens. The characters `//` and `/*` inside a string literal, a raw string literal, or a character literal are then not reported.

A checker that reports text inside literals produces findings that no fix can remove.

The lexer also treats byte string and C string literals as literals. Documentation written as a `#[doc = ...]` attribute is a literal, not a comment, so the checker does not read it.

This illustration shows an input and the findings that the checker prints for it, assuming the file is `src/lib.rs`:

```rust
/// Returns the scale factor.
pub fn scale() -> f32 {
    // Half of the unit size.
    let url = "https://example.com/scale";
    0.5
}
```

```text
src/lib.rs:3: comment other than API documentation or a safety comment
```

The checker does not report line 4, because the slashes there are inside a string literal. A generated file in the repository is checked like any other file.

### 6.3. Heading Checker

This table summarizes the heading checker.

| Property | Value |
| --- | --- |
| Script and recipe | `scripts/check-doc-headings`, `check-doc-headings` |
| Rule | Section 7.3 of the writing guide |
| Reads | The `///` comments of every `.rs` file. Crate-level and module-level documentation, written with `//!`, is not read, because Section 7.4 of the writing guide gives it its own headings |
| Reports | A heading of any level that is not one of the level-one standard headings of Section 7.3 of the writing guide, including a standard heading with the wrong case or level, a standard heading out of the order that section gives, and a standard heading that appears twice in one item's documentation |

The checker reads both ATX headings, which start with `#`, and setext headings, which are underlined with `=` or `-`. The `///` comments that precede one item, with only attributes and other comments between them, are one item's documentation.

The checker ignores lines inside fenced code blocks, because a line that starts with `# ` in a Rust code block is a hidden line of a doctest, not a heading. It does not read documentation written as a `#[doc = ...]` attribute or a `/** */` block comment, which the comment checker of Section 6.2 rejects. Section 7.3 of the writing guide forbids every other heading in the documentation of an item, which is the rule that the checker enforces.

### 6.4. Doctest Attribute Checker

This table summarizes the doctest attribute checker.

| Property | Value |
| --- | --- |
| Script and recipe | `scripts/check-doctest-attributes`, `check-doctest-attributes` |
| Rule | Section 7.5 of the writing guide |
| Reads | The fenced code blocks in the `///` and `//!` comments of every `.rs` file, and in every file named `README.md`, which a crate includes as doctests (Section 7.9 of the writing guide). A path given on the command line that is not named `README.md` is read as Rust |
| Reports | A code block whose attributes include `ignore` or an attribute that starts with `ignore-`, such as `ignore-windows` |

The checker reports the attributes that start with `ignore-` as well, because Section 7.5 of the writing guide forbids `ignore` and these are variants of it that ignore the example on some targets.

The checker splits the info string of a code block into attributes the way rustdoc does. It splits at commas, spaces, and tabs, reads a double-quoted attribute without its quotes, and reads the text inside braces, such as `{.ignore}`, as a class or a key and not as an attribute. Whether a `no_run` example states its reason stays a judgement rule.

### 6.5. Feature Documentation Checker

This table summarizes the feature documentation checker.

| Property | Value |
| --- | --- |
| Script and recipe | `scripts/check-feature-docs`, `check-feature-docs` |
| Rule | Section 7.8 of the writing guide |
| Reads | For each workspace member with a library target, its features as `cargo metadata --no-deps --offline` reports them, and the `# Features` section of the `//!` crate-level documentation at the top of its library root, before the first item. `cargo metadata` also reports the feature that an optional dependency defines when no feature refers to that dependency with `dep:` |
| Reports | A missing `# Features` section in any library, including a library that has no features, a feature, other than `default`, that the section does not name in a code span, and a code span in the section that looks like a feature name, meaning lowercase letters, digits, `_`, `+`, and `-`, but names no feature of the package |

The checker requires the section in a library that has no features, because item 4 of Section 7.4 of the writing guide requires it in every crate. It does not read crate-level documentation written as a `#![doc = ...]` attribute, including `#![doc = include_str!(...)]`.

Unlike the other checkers, it takes the paths of Cargo manifests, and with no paths it checks the workspace of the git repository that contains the current directory. It prints paths relative to the current directory. When `cargo metadata` fails, it exits with status 2 and repeats the first error line of Cargo.

Whether the section says that each feature is on or off by default, and what it enables, stays a judgement rule, because the wording is free.

### 6.6. Prose in API Documentation

`scripts/check-prose` reads the text of the `///` and `//!` comments of `.rs` files and applies to it the rules that it applies to Markdown. This enforces Section 7.10 of the writing guide for the rules of its Section 8 that the script checks.

It reads each run of comments that documents one item as one Markdown document, and it reports each finding at its line in the Rust file. Code spans and fenced code blocks inside the comments stay exempt, as they are in Markdown.

It does not read documentation written as a `#[doc = ...]` attribute or a `/** */` block comment. A Rust file that is not valid UTF-8 cannot be read (Section 6.1). With no paths, `check-prose` checks every Markdown file and every Rust file, and its self-test is `check-prose-self-test`.

### 6.7. Moving a Rule to Enforced

TOOL-17. The change that lands a custom checker MUST do all of these in the same change, as Section 3.5 of the writing guide and Section 16 of the coding standard require for a rule that a tool can check:

- Rewrite the "Checked by" statement of the section of the writing guide that the checker enforces, and its row in Section 7.11 of the writing guide, to name the checker.
- Add the checker to the table of tools in Section 4.8 of the coding standard.
- Update the Validating Changes section of `AGENTS.md`.

A document that still calls an enforced rule a judgement rule sends a reviewer to check by hand what a tool checks, which Hard Rule 4 of `AGENTS.md` forbids. The change that extends `scripts/check-prose` also updates Section 8 of the writing guide, which describes the files that the script and its hook check.

### 6.8. Workflow Pin Checker

This table summarizes the workflow pin checker.

| Property | Value |
| --- | --- |
| Script and recipe | `scripts/check-workflow-pins`, `check-workflow-pins` |
| Rule | TOOL-20, the `pull_request_target` part of TOOL-22, and the top-level part of TOOL-19 (Section 7.2) |
| Reads | Every `.yml` and `.yaml` file under `.github/workflows/`, and every `action.yml` and `action.yaml` file under `.github/actions/`. A file named `action.yml` or `action.yaml` is read as a composite action, and any other file as a workflow |
| Reports | In every workflow and composite action, a `uses:` whose reference is not a full 40-character lowercase hexadecimal commit SHA. In every workflow, also the `pull_request_target` event under the top-level `on:` key, a workflow without a top-level `permissions:` key or with an empty one, and a top-level permission other than `read` or `none`, such as `write-all` or `contents: write` |

A local action, whose reference starts with `./`, needs no commit SHA. A `docker://` reference passes only when it is pinned by an `@sha256:` digest. The checker does not check the permissions of a job, so whether a job that requests more than read access needs it stays a judgement rule, and so does the part of TOOL-22 about secrets and publishing.

The checker reads each file line by line and does not use a YAML parser, and its usage text states the limits that follow from this:

- It reads keys only in block style, one per line, or as the value of `permissions:` in a one-line flow mapping such as `{ contents: read }`.
- It skips the text of a block scalar, such as the script of `run: |`.
- It does not read a `uses:` inside a flow mapping, or a scalar continued over several lines without a block indicator.
- It reads the text after a `#` that starts a line or follows a space as a comment, except inside quotes.

A workflow written in a form that the checker does not read therefore depends on review.

This illustration shows a workflow step and the finding that the checker prints for it, assuming the step is on line 12 of `.github/workflows/ci.yml`:

```yaml
      - uses: actions/checkout@v4
```

```text
.github/workflows/ci.yml:12: uses: 'actions/checkout@v4' is not pinned to a full 40-character commit SHA
```

### 6.9. Document Reference Checker

This table summarizes the document reference checker.

| Property | Value |
| --- | --- |
| Script and recipe | `scripts/check-doc-references`, `check-doc-references` |
| Rule | Section 7.1 of the writing guide, and Hard Rule 15 of `AGENTS.md`, for the citations that a tool can recognize |
| Reads | The project documents, which are the Markdown files under `docs/` of the git repository that contains the current directory, at any depth, tracked or not ignored. It then reads the whole text of each checked file, including comments, string literals, the help comments of recipes, and usage texts |
| Reports | The path of a project document, its file name alone, a directory under `docs/` that holds one, a section citation, and a rule identifier that a project document defines |

The checker counts these forms as a citation:

- A document path, such as `docs/design/tooling.md`, its file name alone, such as `tooling.md`, and a directory under `docs/` that holds a document, such as `docs/design/`. The directory `docs/` alone is not reported. A match must not continue a longer name, so the character before it is not a letter, a digit, `_`, `.`, or `-`.
- The word "Section" or "Sections" in any case, followed by whitespace and a number such as `3` or `3.1`. The checker reports every section citation, also one of an external document, because it cannot tell which document a number belongs to.
- A rule identifier: a prefix that a project document defines, a hyphen, and a number, such as `TOOL-14`. A match must stand alone, so `XTOOL-14`, `TOOL-14x`, and `docs/TOOL-14` are not reported.

The checker keeps no list of prefixes by hand. On every run, it collects from every project document the identifiers that stand alone in the first cell of a Markdown table row, such as `| TOOL-14 | 6.1 | ... |`.

An identifier is a run of capital letters and digits that starts with a letter, a hyphen, and a number, and the run is its prefix. The summary tables of labelled requirements have that form.

A document that adds a new prefix is therefore checked without a change to the checker. An identifier whose prefix no document defines in that way, such as `UTF-8`, `SHA-256`, or `RUSTSEC-2026-0441`, is not reported, so a fixed list cannot go stale or report a standard name. A label written only in prose, such as `[IR-8]` in a list of references, defines no prefix.

With no paths, the checker checks every `.rs`, `.toml`, `.yml`, `.yaml`, and `.json` file, the `justfile` at the root, and every file under `scripts/`, tracked or not ignored, and prints paths relative to the root of the repository. It skips its own self-test cases under `scripts/fixtures/check-doc-references/`. It reads the cases of the other checkers under `scripts/fixtures/`, which carry the suffix `.fixture`, as files under `scripts/`.

With no paths, the checker selects no Markdown file outside `scripts/`, so the documents and `README.md` may cite each other. A path given on the command line is checked whatever its name, and is printed as given.

It prints each finding in one of these forms, at the line where the citation starts. A section citation that spans two lines is reported with its whitespace joined into one space:

```text
path:line: cites the project document <path or name>
path:line: cites a section of a project document: '<citation>'
path:line: cites the rule identifier <identifier> of a project document
```

The checker follows TOOL-14, with the departure that TOOL-14 allows. It exits with status 1 when it reports a finding. It exits with status 2 after checking every readable file when a file cannot be read or is not valid UTF-8.

It also exits with status 2 before it checks any file, even when paths are given, in these cases:

- It runs outside a git repository, with `check-doc-references: not inside a git repository`.
- A project document cannot be read or is not valid UTF-8, with `check-doc-references: cannot read <document>: <cause>`.
- The repository has no project document, with `check-doc-references: no project document: the repository has no Markdown file under docs/`.
- No project document defines a rule identifier, with `check-doc-references: no rule identifier: no project document defines one in the first cell of a table row`.

The last two errors keep a change in the layout of the documents from turning the check off silently.

The cases of its self-test under `scripts/fixtures/check-doc-references/` give the checker small documents of their own, which define the prefix `ZZQ`. They cover these behaviours:

- Each form of citation in each kind of file that the checker reads.
- The forms that it must not report, such as `UTF-8`, `SHA-256`, `GPL-3.0-or-later`, a prefix defined only in prose, a prefix without a number, and a web address that contains `docs/`.
- The files that it skips, which are a Markdown file, a file of another type, and its own cases.
- Paths given on the command line.
- The errors of exit status 2 for a missing git repository, a repository without project documents, documents that define no rule identifier, a checked file that cannot be read, and a checked file that is not valid UTF-8.

Wording about the history of the project, such as "this version" or the milestone that added an item, is beyond what the checker reads, so it stays a judgement rule.

This illustration shows a recipe help comment and the findings that the checker prints for it, assuming it is on line 14 of the `justfile`:

```just
# Run the checks that Section 3.3 of docs/design/tooling.md lists
```

```text
justfile:14: cites a section of a project document: 'Section 3.3'
justfile:14: cites the project document docs/design/tooling.md
```

## 7. Continuous Integration

### 7.1. Workflow and Jobs

One workflow, `.github/workflows/ci.yml`, runs on every push to `main`, on every push to a branch that matches `feature/**`, and on every pull request. The project opens no pull requests, so a plain push of a feature branch is how a branch is verified before it merges (Section 14.7), and each such push starts one run. The `pull_request` trigger stays so that a pull request from a fork runs the same jobs, with a read-only token and no secrets (Section 7.2).

The `concurrency` group of a run on `main` is the run itself, and nothing cancels it, so every commit on `main` keeps its own result. For any other git reference, such as a feature branch or a pull request, the group is the workflow and the reference, and `cancel-in-progress` is true, so a new push cancels the earlier run for that reference.

Every job has a timeout of 60 minutes, and every `run` step uses Bash. The checkout step sets `persist-credentials: false`, so the token is not left in the git configuration of the runner for later steps.

TOOL-18. A workflow MUST run every check through a `just` recipe and MUST NOT run a check command directly. CI then runs the same commands as the development container, so a check that fails in one place has the same command to reproduce it in the other. Steps that only prepare a runner, such as installing a tool, are not checks.

This table lists the jobs, with the name that GitHub shows for each and the hosted runner label. `check-workflows` and `check-workflow-pins` check the workflow.

| Job | Name | Runner | Runs |
| --- | --- | --- | --- |
| `check` | Run every check on Linux | `ubuntu-latest`, Linux x86_64 | `just check` |
| `ci-only` | Run `check-features` on Linux, Run `check-msrv` on Linux | `ubuntu-latest`, Linux x86_64, as a matrix of the two recipes, with `fail-fast: false` | `just check-features` and `just check-msrv` |
| `coverage` | Measure test coverage on Linux | `ubuntu-latest`, Linux x86_64 | `just coverage`, then the upload of `target/lcov.info` as the artifact `coverage-lcov` |

Every job first checks out the repository and installs the Rust toolchain and the tools that it needs. Then it runs its own steps:

- The `check` job installs actionlint and runs `just check`.
- The `ci-only` job runs the recipe of its matrix entry.
- The `coverage` job runs `just coverage` and uploads the report.

The workflow defines the checkout step once, with a YAML anchor, and reuses it in the other jobs, so the step is the same in every job.

The `coverage` job runs the tests of the `default` profile under cargo llvm-cov and keeps the report. The recipe prints no coverage summary in the log of the job (Section 14.5), so the uploaded report is the only result of the job.

The action `actions/upload-artifact`, pinned by commit SHA (TOOL-20), uploads `target/lcov.info` as the artifact `coverage-lcov`, which GitHub keeps for 14 days. The step sets `if-no-files-found: error`, so the job fails when the recipe wrote no report. The job sets no coverage threshold, so the report informs a reviewer and does not gate a merge.

CI installs the tools on each runner, because no container image is published for CI to run in. Each job installs only the tools that its recipes need:

- The Rust toolchain, through rustup, which the hosted runners provide. Each job installs only the stable toolchain, in a step named `Install the stable Rust toolchain`, with the minimal profile and the components that it needs, which are rustfmt and clippy in the `check` job, none in the `ci-only` job, and rustfmt and llvm-tools in the `coverage` job.

  `check-fmt` needs rustfmt, and cargo llvm-cov needs llvm-tools.
- `just` in every job, cargo-nextest in the `check` and `coverage` jobs, shellcheck, typos, taplo, cargo-machete, and cargo-deny in the `check` job, cargo-hack in the `ci-only` job, and cargo-llvm-cov in the `coverage` job, through the action `taiki-e/install-action`, pinned by commit SHA. The action installs prebuilt binaries and verifies the SHA-256 checksum of each one against the manifest of versions and checksums that it carries. The workflow sets `fallback: none`, so the action fails instead of installing a tool by a route that its manifest does not cover.
- actionlint, in the `check` job, through an inline step. It reads the latest release from GitHub, downloads the release archive for Linux x86_64 and the checksums file of the same release, verifies the archive with `sha256sum --check --strict`, and fails on a mismatch before it extracts the archive.

Each route installs the latest version that it provides, which matches the policy of the container image, in which no tool version is pinned. A tool in CI can therefore have a different version from the same tool in the development container, and a check can pass in one and fail in the other until both have the same version.

The workflow caches nothing, so no file is restored from a cache.

### 7.2. Workflow Security

TOOL-19. A workflow MUST grant its token read access to the repository contents only, at the level of the workflow, and a job that needs more MUST request the extra permission on that job alone. A compromised step can use every permission that the token holds.

`check-workflow-pins` reports a workflow without top-level permissions and a top-level permission other than `read` or `none` (Section 6.8). Whether a job that requests more needs it is a judgement rule.

TOOL-20. A workflow MUST reference every action, including the actions that GitHub publishes, by a full commit SHA and never by a tag or a branch. The owner of an action can move a tag or a branch to different code, which then runs in every later job without any change to this repository.

actionlint does not check this, and `check-workflow-pins` enforces it (Section 6.8).

TOOL-21. A workflow step that downloads a file MUST verify it against a SHA-256 checksum that the upstream publishes and MUST stop the job before the file is used when the checksum differs, as Hard Rule 7 of `AGENTS.md` requires.

A step that installs tools through an action MUST use an action that verifies each download in this way, pinned by commit SHA (TOOL-20), which also pins the manifest of versions and checksums that the action carries. The step MUST configure the action to fail instead of installing by a route that it does not verify.

A file restored from a cache MUST be verified in the same way as a fresh download.

These requirements hold because a tampered file that is not verified runs with the permissions of the job, and a cache can hold a file that an earlier, compromised run stored.

When the upstream publishes no checksum, the step SHOULD record in the workflow the checksum of the file that the agent verified when it wrote the step. A later change in the served file is then detected, even though the first file is trusted on first use.

A checksum that comes from the same host as the file, such as the checksums file of an actionlint release, proves that the download is intact but not that it comes from the publisher, because an attacker who controls the host can replace both. A checksum in the pinned manifest of the installer action comes from the commit of the action that TOOL-20 pins instead.

Section 13 states the residual risk. Whether each step meets TOOL-21 is a judgement rule, because no tool can tell from a workflow which step downloads a file.

The action `actions/upload-artifact`, which the `coverage` job uses (Section 7.1), is pinned by commit SHA like every other action, and `check-workflow-pins` checks it.

TOOL-22. A workflow MUST NOT use the `pull_request_target` event, read a secret, or publish a crate. The workflows need no secret to check code, and a workflow that holds a secret while it runs the code of a pull request exposes that secret to the author of the pull request.

`check-workflow-pins` reports the `pull_request_target` event (Section 6.8). The rest of TOOL-22 is a judgement rule.

The workflow keeps the `pull_request` trigger for pull requests from forks. GitHub gives a run for such a pull request a read-only token and no secrets, so the code of the pull request can neither write to the repository nor read a secret. A push to `main` or to a feature branch needs write access to the repository, so the push triggers add no way for an outside author to run code in CI.

## 8. Dependency Policy

The file `deny.toml` configures cargo-deny, which Section 4.8 of the coding standard lists with what it owns. Section 4.8 also requires the licence allow-list to include `GPL-3.0-or-later` itself.

TOOL-24. The licence allow-list of `deny.toml` MUST start with `GPL-3.0-or-later` alone. An agent MUST add any other licence only in the change that adds the first dependency under it, after confirming that the licence is compatible with GPL-3.0-or-later. A licence that no dependency uses widens what a later dependency can bring in without any check failing.

TOOL-25. Every path dependency between workspace members MUST carry a version, and `deny.toml` MUST set `bans.allow-wildcard-paths = false`, because Cargo refuses to package or publish a crate whose path dependency has no version, so without the check a missing version surfaces only at the release. The value `true` is no alternative (Section 4.8 of the coding standard).

TOOL-26. `deny.toml` MUST make the check fail on every security advisory that applies to a dependency in `Cargo.lock`, on every duplicate version of a crate, and on every crate from a source other than crates.io. An agent MAY record an exception only for one advisory or one duplicate, with a reason in `deny.toml`.

Section 4.5 of the coding standard leaves the policy on duplicate versions to cargo-deny, and a policy that only warns reports nothing that the check enforces. A duplicate that no change to the workspace can remove is the expected case for an exception. An exception for a duplicate version has the form `skip = [{ crate = "name@version", reason = "..." }]` in the `[bans]` table.

TOOL-31. `deny.toml` MUST set `unused-allowed-license = "deny"` in its `[licenses]` table and `unused-ignored-advisory = "deny"` in its `[advisories]` table, because without them cargo-deny only warns about an unused entry, and the lists that TOOL-24 and TOOL-26 keep minimal then grow without any check failing.

`deny.toml` has these settings:

```toml
[graph]
all-features = true

[advisories]
unused-ignored-advisory = "deny"

[licenses]
allow = ["GPL-3.0-or-later", "Apache-2.0", "MIT", "Unicode-3.0"]
confidence-threshold = 0.8
include-dev = true
unused-allowed-license = "deny"

[bans]
multiple-versions = "deny"
multiple-versions-include-dev = true
wildcards = "deny"
allow-wildcard-paths = false

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

cargo-deny 0.20.2 leaves dev-dependencies out of the licence check and the duplicate check by default. `include-dev = true` and `multiple-versions-include-dev = true` bring them in, so that TOOL-24 and TOOL-26 hold for every dependency, including those that only tests use.

The licences other than `GPL-3.0-or-later` in `allow` are those of the dependencies in `Cargo.lock`, and each is compatible with GPL-3.0-or-later, as the Free Software Foundation's licence list states [FSF-LIST]:

| Licence | Why it is in the list | Compatibility |
| --- | --- | --- |
| `Apache-2.0` | It satisfies every `MIT OR Apache-2.0` expression | Compatible with version 3 of the GNU GPL |
| `MIT` | `strsim` carries `MIT` alone, and `clap_builder` needs it for the `clap` feature `suggestions`. The dev-dependency `astro-float-num` also carries `MIT` alone | Compatible with the GNU GPL. The list names it the Expat licence |
| `Unicode-3.0` | `unicode-ident` carries `(MIT OR Apache-2.0) AND Unicode-3.0` | Compatible with all versions of the GNU GPL |

`Apache-2.0` satisfies every expression that offers `MIT` as an alternative, so `MIT` is in the list only for a crate that offers no alternative, such as `strsim` and `astro-float-num`.

The `[advisories]` table has no `ignore` list and the `[bans]` table has no `skip` list. The change that adds an exception under TOOL-26 adds the list, with one entry per line and its reason, and the change that adds a licence under TOOL-24 extends `allow`.

cargo-deny checks the dependency graph of every target, because the `[graph]` table names no target, so a list can cover crates that a build on Linux does not compile. `all-features = true` makes it check the graph of every Cargo feature, which it reads without building anything.

`check-deny` fetches the advisory database on every run, so it needs network access (Section 14.6).

`Cargo.lock` holds `flate2` at version 1.1.9. `png` depends on `flate2` and on `miniz_oxide`, and the newest version of `flate2` brings a second version of `miniz_oxide`, which fails the duplicate check of TOOL-26. At 1.1.9, both resolve to `miniz_oxide` 0.8.9, so the check passes with no exception in `deny.toml`. A plain `cargo update` moves `flate2` to its newest version and brings the duplicate back. This command, which no recipe wraps, restores the pin:

```sh
cargo update -p flate2 --precise 1.1.9
```

Section 9 of `docs/design/render.md` states why the crate keeps the pin, and its Section 12.3 keeps a `skip` entry in `deny.toml` as the alternative for the owner to decide.

## 9. Publishing Readiness

This section defines how the repository is kept ready to publish a crate. Whether a crate is published is outside this document.

TOOL-4. The `[workspace.package]` table MUST declare the `license` field as the SPDX licence identifier `GPL-3.0-or-later`, the `rust-version` field as `1.99`, the `repository` field as the address of the repository on GitHub, and the `readme` field as the root `README.md`. Every member MUST inherit these fields from the workspace instead of setting its own, because a member with its own value can disagree with the licence that cargo-deny checks and with the minimum supported Rust version (MSRV) that Clippy reads.

TOOL-5. An agent MUST NOT change `rust-version` except in a change whose stated purpose is to raise it, and that change updates Section 14.3 and `README.md`. The MSRV is a promise to users (DEP-4 of the coding standard), and a raise hidden in another change breaks their builds without notice.

TOOL-27. Every published crate MUST carry the metadata fields that the row of `clippy::cargo_common_metadata` in Section 4.5 of the coding standard lists, inheriting those that TOOL-4 places in `[workspace.package]` and setting `description`, `keywords`, and `categories` itself. Every member that is never published MUST set `publish = false`, because Clippy reports a missing field on every member that does not set `publish = false`, so the check fails until the metadata is complete.

A `readme` that a member inherits from the workspace as `readme = "README.md"` resolves to the root file, and Cargo copies that file into the package.

Packaging one crate that depends on a member that is not published yet fails, while `cargo package --workspace` succeeds.

Cargo does not include the root `LICENSE` in a packaged crate when the manifest sets only the `license` field, so how each published crate carries the licence text is open (Section 12, TQ-6).

How a release is carried out is open (Section 12, TQ-4).

## 10. Container Image Requirements

The container image is defined outside this repository, so the changes of this section are made there and not in this repository.

TOOL-28. An agent MUST make each change to the container image in the repository that defines it. The first change in this repository after an image rebuild that adds or removes a tool MUST update the tool list in the Technology Decisions section of `AGENTS.md`. It MUST also update the status of the tool in Section 4.8 of the coding standard.

A document that calls an installed tool missing leaves its rules as judgement rules that a reviewer checks by hand. A document that calls a missing tool installed leads an agent to build on it.

TOOL-29. The container image MUST provide these, because the recipes of Section 3.2 run them:

- Every tool that Section 4.8 of the coding standard lists.
- The stable toolchain as the default, which is the only toolchain that the recipes of Section 3.2 use apart from that of `check-msrv` (DEP-4 of the coding standard).
- The component llvm-tools of the stable toolchain, which cargo llvm-cov needs.
- GitHub's Secure Shell (SSH) host keys in the user's known hosts file, so that git can verify GitHub when it pushes.

The image is not required to provide a toolchain for the declared `rust-version`, because `check-msrv` runs in CI and installs that toolchain on first use (Section 14.3). The Technology Decisions section of `AGENTS.md` lists what the image provides.

## 11. Assumptions to Confirm

Each row of this section states a fact that this document relies on and that has not been checked. A section that relies on one of them cites its identifier. The change that confirms or corrects an assumption states the result in the section that relies on it and removes the row.

No assumption is open. An identifier is never reused, so the next assumption takes TA-10.

## 12. Open Questions

Each row is a question that this document leaves open. The design document in the third column answers it, and the change that answers it removes the row.

| Identifier | Question | Owning design document |
| --- | --- | --- |
| TQ-4 | How is a release carried out: which recipe publishes, in which order the crates are published, and where the crates.io token is kept outside the repository? | This document |
| TQ-6 | How does each published crate carry the text of its licence, which the GPL requires to accompany the work, given that Cargo does not package the root `LICENSE` when a manifest sets only the `license` field (Section 9)? | This document |
| TQ-11 | Some tests are split or use small inputs for a reason that does not apply, because the owner withdrew `DEC-5` of `docs/design/architecture.md`. They are the single cases of Section 10.1 of `docs/design/group.md`, the integration test of `escher-domain` that checks the side order of `22` alone, one case of its test of the side orders of the spherical domains, and the integration tests of `escher-cli` that run one signature or one argument order of a table test on their own. They stay because they cost nothing. Should they be merged into their table tests? | `docs/design/group.md`, `docs/design/domain.md`, and `docs/design/cli.md` |

An identifier is never reused, so the next question takes TQ-12.

## 13. Security Considerations

The threat model has six parts:

1. A dependency, a build script, or a procedural macro in the dependency graph is malicious or compromised. It runs with the permissions of the developer or the job whenever a check compiles the workspace, including Clippy, the documentation build, and the tests.
2. An action that a workflow uses is compromised, or its owner moves a tag to different code.
3. An attacker on the network or at a download host tampers with a tool or the advisory database that a check or a job downloads.
4. The author of a pull request tries to read a secret or to write to the repository through CI.
5. A pre-commit hook repository changes the code that its hooks run.
6. A file or a file name that a checker reads, such as one in a pull request, holds terminal control sequences. A checker that printed them unchanged would send them to the terminal of the developer, which is terminal escape injection.

The design addresses each part:

1. For dependencies, cargo-deny rejects crates with known advisories and crates from sources other than crates.io (TOOL-26), and the allow-list admits only reviewed licences (TOOL-24). `Cargo.lock` fixes every version, and the checks run inside the development container and not on the developer's host.
2. For actions, TOOL-20 pins each one to a commit SHA, which `check-workflow-pins` enforces, and TOOL-19 limits what a compromised step can do. The checkout step does not leave the token in the git configuration of the runner (Section 7.1).
3. For downloads, TOOL-21 verifies every file before use, and the workflow restores nothing from a cache. The installer action carries its checksums in a manifest that its commit SHA pins.
4. For pull requests, TOOL-22 keeps secrets out of every workflow and forbids the event that runs pull request code with secrets, and `check-workflow-pins` reports that event. A run for a pull request from a fork gets a read-only token and no secrets (Section 7.2).
5. For hooks, `.pre-commit-config.yaml` pins the repository `pre-commit-hooks` to a commit SHA, and the local hooks run only recipes of this repository.
6. For checker output, every custom checker escapes its output (Section 6.1), so a checked file cannot send a control sequence to the terminal.

These attacks are out of scope:

- A compromise of GitHub or crates.io themselves defeats every check here, and no repository configuration can detect it.
- The build of the container image is defined and secured in the repository that defines the image.
- A compromise of the developer's machine outside the container is a matter of that machine's security, not of this repository.

The topics of unsafe code, foreign function interface invariants, untrusted input, and file paths do not apply, because this document defines no Rust code.

The residual risk has six parts:

1. A new dependency's build script and procedural macros run on the first build that includes them, before any reviewer has read them, and cargo-deny finds only vulnerabilities that are already published.
2. A checksum shows only that a file matches what the upstream published, and the first download of a file whose upstream publishes no checksum is trusted on first use. A checksum that comes from the same host as the file, as the checksum of actionlint does, proves integrity but not origin, so an attacker who controls that host can replace the file and its checksum together. Because CI installs the latest version of each tool, a new release of a tool runs in CI without any change to this repository and before anyone has reviewed it.
3. A pinned commit of an action is as trustworthy as the review it received when it was pinned, and the manifest of the installer action is as trustworthy as that commit.
4. `check-deny` fetches the advisory database on every run (Section 14.6), so a tampered database could hide an advisory.
5. `check-workflow-pins` reads workflows line by line, so a reference written in a form that it does not read (Section 6.8) can pass it. The permissions of each job, the use of secrets, and whether each download step verifies its file depend on review.
6. The escaping covers only the custom checkers of Section 6. `scripts/check-commit-message` and the external tools that the recipes run, such as shellcheck, typos, actionlint, and Cargo, print their output without this escaping, and this document does not specify how they handle control characters. Because a backslash prints unchanged, escaped output can be ambiguous to a reader, which misleads but sends no control sequence.

## 14. Operational Considerations

### 14.1. Recipes and Parameters

Section 3.2 lists every recipe. This document defines no Cargo feature.

This table lists every recipe parameter and its default. Every other recipe takes no parameters.

| Recipe | Parameter | Default |
| --- | --- | --- |
| `check-prose` | Paths of files to check | Every Markdown file and every Rust file in the repository |
| `check-rust-comments`, `check-doc-headings` | Paths of files to check | Every Rust file in the repository (TOOL-14) |
| `check-doctest-attributes` | Paths of files to check | Every Rust file and every `README.md` file in the repository (TOOL-14) |
| `check-feature-docs` | Paths of Cargo manifests to check | The workspace of the repository (Section 6.5) |
| `check-workflow-pins` | Paths of workflow and composite action files to check | Every workflow under `.github/workflows/` and every composite action under `.github/actions/` (Section 6.8) |
| `check-doc-references` | Paths of files to check | Every `.rs`, `.toml`, `.yml`, `.yaml`, and `.json` file, the `justfile`, and every file under `scripts/` except the checker's own self-test cases (Section 6.9) |
| `check-commit-message` | The commit message file | None. The parameter is required. |
| `merge` | The branch, and further arguments for `scripts/merge-branch` | None for the branch, which is required |

### 14.2. Environment Variables

This table lists every environment variable that the recipes, the hooks, or the checks read, except the variables of the snapshot library `insta` that no contributor or workflow needs to set, which the paragraph after the table points to.

| Variable | Read by | Default and behaviour when unset |
| --- | --- | --- |
| `CARGO_BUILD_WARNINGS` | Cargo, in the commands of Section 4.8 of the coding standard | The recipes set it to `deny` for those commands. Outside them it is unset, and Cargo reports warnings without failing. |
| `CLAUDE_PROJECT_DIR` | The hook in `.claude/settings.json` | Set by Claude Code. The hook does not run outside it. |
| `CI` | `insta`, in the snapshot tests that `test` and `coverage` run | Not CI. A failing snapshot test then writes a `.snap.new` file beside the committed snapshot, for review with `cargo insta review`. In CI, a failing snapshot test writes no file. |
| `INSTA_UPDATE` | `insta` | `auto`, as the row of `CI` describes. No recipe or workflow sets it. |
| `INSTA_FORCE_PASS` | `insta` | False. No recipe, hook, or workflow sets it (CLI-12 of `docs/design/cli.md`). |
| `INSTA_WORKSPACE_ROOT` | `insta` | `insta` finds the workspace root with `cargo metadata`, through the `cargo` that `CARGO` names. |
| `ESCHER_UPDATE_GOLDEN` | The golden image tests of `escher-render`, in the runs of `test`, `coverage`, and `update-golden` | Unset. A golden image test compares its rendering with the committed file. The recipe `update-golden` sets it to `1`, and the test then writes its rendering over the committed file (Section 14.9). No other recipe, hook, or workflow sets it. |

Section 8.2 of `docs/design/cli.md` gives the behaviour of the variables of `insta` that the snapshot tests depend on, including two that this table leaves out: `INSTA_REQUIRE_FULL_MATCH`, and `TF_BUILD`, which `insta` reads when `CI` is unset.

### 14.3. Platforms and Minimum Supported Rust Version

The checks run in the development container on Linux x86_64, and CI runs them on hosted Linux x86_64 runners, with tool versions that can differ from those of the container (Section 7.1).

The workspace declares `rust-version = "1.99"` (TOOL-4), which only a change of TOOL-5 raises, and `check-msrv` verifies it in the CI job `ci-only`.

cargo hack always uses the rustup toolchain named exactly like `rust-version`, which is `1.99`. It installs that toolchain with rustup when it is missing, even when the installed stable toolchain is a later patch release such as 1.99.1. The first run of `check-msrv` on a machine therefore downloads a toolchain and needs network access, and later runs use the installed toolchain.

The root `.gitattributes` holds the single line `* text=auto eol=lf`. git normalizes every file that it detects as text to LF line endings in the repository and checks it out with LF on every platform, whatever the `core.autocrlf` setting of the clone. Without it, a clone with `core.autocrlf` set to `true` checks out text files with CRLF line endings, which breaks a script or a recipe that reads a file line by line.

The self-tests of the custom checkers do not depend on the line endings of the checkout, because their fixtures create carriage returns from placeholders at run time (Section 6.1).

### 14.4. Failing Checks

This table says what a failure of each check means and what the developer does.

| Recipe | A failure means | What the developer does |
| --- | --- | --- |
| `check-fmt` | A Rust file is not formatted, including a file that a crate includes with `include!`, or `git ls-files` failed or listed no Rust file, as its message says | Runs `just fmt`. For a failed listing, runs the recipe inside the git checkout of the repository |
| `fmt` | `git ls-files` failed or listed no Rust file, as its message says, or rustfmt cannot parse a file | Runs the recipe inside the git checkout of the repository, or fixes the syntax error that rustfmt reports |
| `check-clippy`, `check-doc`, `check-doctest` | A lint fired | Follows Section 4.4 of the coding standard and the lint's row in Section 4.5 |
| `test` | A test failed | Fixes the code or the test |
| `check-deny` | A licence is not allowed, an advisory applies, a crate is duplicated, or a source is not crates.io. The exit status is the sum of 1 for advisories, 2 for bans, 4 for licences, and 8 for sources | Applies Section 8: replaces or upgrades the dependency, or records an exception that TOOL-24 or TOOL-26 permits |
| `check-prose`, `check-rust-comments`, `check-doc-headings`, `check-doctest-attributes`, `check-feature-docs` | Exit status 1: a rule of the writing guide is broken at each reported `path:line`, and the message names the problem, such as "comment other than API documentation or a safety comment" or "feature 'name' is not named in a code span in the '# Features' section". Exit status 2: a file cannot be read, including a file that ends inside an unterminated literal, comment, or attribute, `cargo metadata` failed, the checker ran with no paths outside a git repository, or the setup is broken, as the message on standard error says (Section 6.1) | For status 1, fixes the text as the rule that Section 6 names requires. For status 2, fixes the file or the setup that the message names, such as by closing the unterminated item at the reported line |
| `check-workflow-pins` | Exit status 1: a rule of Section 7.2 is broken at each reported `path:line`, such as "uses: '...' is not pinned to a full 40-character commit SHA", "workflow uses the pull_request_target event", "workflow has no top-level permissions key", or "top-level permissions '...' grant more than read access". Exit status 2: as for the other custom checkers | Pins the action to the full commit SHA of the reviewed release, removes the event, or sets the top-level permissions to `contents: read` and moves any extra permission to the job that needs it (TOOL-19) |
| `check-doc-references` | Exit status 1: code or configuration cites a project document at each reported `path:line`, with "cites the project document ...", "cites a section of a project document: '...'", or "cites the rule identifier ... of a project document". Exit status 2: as for the other custom checkers, and also a project document that cannot be read, no project document, or no document that defines a rule identifier (Section 6.9) | For status 1, removes the citation and states what the code needs on its own terms, as Section 7.1 of the writing guide requires. For status 2, fixes the file or the document that the message names, or restores the Markdown files under `docs/` and their summary tables |
| A self-test recipe | A case under `scripts/fixtures/<checker>/` gave a different standard output, standard error, or exit status from the expected one, which the recipe prints after "FAIL" with the name of the case. Exit status 2 means that the checker or its cases cannot be read | Fixes the checker, or, when the change to its behaviour is intended, the expected files of the case |
| `check-shell` | shellcheck reported a finding | Fixes the script or recipe |
| `check-typos` | typos found a spelling mistake | Fixes the word, or accepts a correct word in `typos.toml` (TOOL-30) |
| `check-toml` | A TOML file is not formatted | Runs `just fmt` |
| `check-workflows` | actionlint reported a finding in a workflow | Fixes the workflow |
| `check-unused-deps` | A dependency is declared and not used | Removes it |
| `coverage` | A test failed, or cargo llvm-cov did not write the report. In the CI job `coverage`, the upload step also fails when `target/lcov.info` does not exist | Reads the message of cargo llvm-cov, and fixes a failing test as for `test` |
| `check-features`, `check-msrv` | A feature combination does not compile, or the code needs a newer toolchain than `rust-version` | Fixes the code, or raises `rust-version` under TOOL-5 |

Clippy can report a `clippy::cargo_common_metadata` warning again after the manifest has been fixed, because it replays its cached result until a source file of the crate changes. The developer then updates the modification time of a source file of the crate, or cleans the build output of the crate, and runs the check again.

No agent weakens a configuration to make a check pass, as Hard Rule 6 of `AGENTS.md` states.

This document states no expected duration for a check.

A check that passes in the development container and fails in CI, or the reverse, can come from a difference in tool versions (Section 7.1). The developer compares the version in the log of the job with the version in the container before changing the code.

A job that GitHub cancelled before any step ran, with the annotation "The job was not acquired by Runner of type hosted even after multiple attempts", did not fail because of the change. No hosted runner picked the job up, which is a problem of runner capacity. The developer reruns that job and judges the change by the result of the rerun.

### 14.5. Logs

The recipes print the output of the tools they run and keep no log. GitHub Actions keeps the log of each job.

The recipe `coverage` prints the output of the build and the tests and no coverage summary. With the LCOV options, cargo llvm-cov ends its output with the line `Finished report saved to target/lcov.info`. The report is the LCOV file `target/lcov.info`, which the CI job `coverage` uploads as the artifact `coverage-lcov` and GitHub keeps for 14 days (Section 7.1).

### 14.6. Network Access

`just check` needs network access. `check-deny` fetches the advisory database from <https://github.com/RustSec/advisory-db> on every run, and fails with exit status 1 when it cannot.

With a database already cached, `cargo deny --offline check`, with the global option `--offline` before `check`, works without network access. `check-deny` runs the command of Section 4.8 of the coding standard without that option, so it always checks against the current database.

Outside `check`, the first run of `check-msrv` on a machine downloads a toolchain (Section 14.3).

### 14.7. Verifying and Merging a Branch

This section gives the procedure for verifying a branch in CI and merging it. The project opens no pull requests, so a push of the branch is what runs CI on it (Section 7.1). The agent follows these steps:

1. It pushes the branch with a plain push, which starts one run for the last commit of the branch.
2. It reads the run for that exact commit from the run itself, with these commands, in which `<branch>` and `<id>` are placeholders. The first prints the identifier, status, conclusion, and commit of the latest run of the branch, and the second prints each job of the run with its conclusion:

   ```sh
   gh run list --branch <branch> --limit 1 --json databaseId,status,conclusion,headSha
   gh run view <id> --json jobs
   ```

3. It runs `just merge <branch>` only when the run's `headSha` is the last commit of the branch, its conclusion is `success`, and every job succeeded.

The agent reads the run itself and never the exit status of a command whose output is piped into another program. The exit status of a pipeline is the status of its last program, so it says nothing about the run, and a merge gated on it can put a failing commit on `main`. A job cancelled for lack of a runner is rerun, as Section 14.4 states, and the merge waits for the rerun.

`just merge` builds the message of the squashed commit from every commit of the branch (Section 3.2). A branch with several work-in-progress commits is therefore merged through a fresh branch.

The agent creates a new branch from `main` that holds the identical tree of the verified branch as one commit with the final message, and runs `just merge` on the fresh branch. The fresh branch holds the tree that CI verified only when the verified branch contains the current `main`. No pushed history is rewritten.

`just merge` deletes the merged branch on `origin` when its tip there is part of the squash. With the fresh-branch procedure, the merged branch is the fresh one, so the branch that was pushed for CI stays on `origin`. Its local branch and worktree count as squash-merged once `main` holds its tree, so `just worktree-prune` removes them.

### 14.8. Copyrighted Source Material

The directory `references/` at the root of the repository holds copyrighted source material, such as the papers and books that a subject-matter reference under `docs/research/` cites. `.gitignore` ignores it, and nothing in it is ever committed, because the repository has no right to redistribute that material.

### 14.9. Updating Golden Images

A golden image test that fails under `test` writes the rendered image and an image of the differences under the directory that `CARGO_TARGET_TMPDIR` names, inside `target/`, and its output names the number of differing pixels (Section 8.4 of `docs/design/render.md`). The developer follows these steps:

1. Opens the rendered image and the difference image, and decides whether the change to the image is intended.
2. When it is not intended, fixes the code and runs `just test` again.
3. When it is intended, runs `just update-golden`, which runs only the golden image test of `escher-render` and writes each rendered image over its committed file, so that no other test affects the result, and inspects the new files.
4. Runs `just test`, which compares with the new files, and commits the changed PNG files with the change that caused them, saying in the commit message why the images changed.

A golden file that is missing, cannot be decoded, or differs from the rendered image in size, colour type, or bit depth fails that image with a message that names `just update-golden`, and the test still checks the other images. Running that recipe a second time leaves the files unchanged, byte for byte. The developer never edits a golden file by hand and never sets `ESCHER_UPDATE_GOLDEN` in another recipe, a hook, or a workflow (TEST-14 of the coding standard).

## 15. References

### 15.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 15.2. Informative References

- [FSF-LIST] Free Software Foundation, "Various Licenses and Comments about Them", section "GPL-Compatible Free Software Licenses", <https://www.gnu.org/licenses/license-list.html>.

## Appendix A. Requirement Summary

This table lists every labelled requirement of this document with a one-line summary. The labels TOOL-1, TOOL-2, TOOL-3, TOOL-10, TOOL-23, and TOOL-32 are reserved and are not assigned.

| Label | Section | Summary |
| --- | --- | --- |
| TOOL-4 | 9 | Licence, `rust-version` 1.99, repository, and readme are workspace fields that every member inherits. |
| TOOL-5 | 9 | `rust-version` changes only in a change whose purpose is to raise it. |
| TOOL-6 | 3.1 | Recipes have the name, group, and `check` membership of Section 3.2. |
| TOOL-7 | 3.1 | Recipes run the commands of Section 4.8 of the coding standard as written there, including the file list of the formatting command, adding only a nextest profile or the arguments that Section 4.8 names for `coverage`. The one exception is `update-golden`, which narrows the test command to the golden image test of `escher-render` with `ESCHER_UPDATE_GOLDEN` set, because it only rewrites the golden files and verifies nothing. |
| TOOL-8 | 3.1 | `check` passes in the container without a hardware GPU or a display. |
| TOOL-9 | 3.1 | No recipe for a tool before the image provides it. |
| TOOL-11 | 4 | No profile retries a failed test. |
| TOOL-12 | 4 | A recipe that runs nextest passes when its profile selects no test, through `--no-tests=warn`. |
| TOOL-13 | 5 | Commit hooks run only checks that do not compile the workspace. |
| TOOL-14 | 6.1 | Custom checkers follow the output and exit status conventions of `scripts/check-prose`, except that a checker whose required input is missing exits with status 2 before it checks any file. |
| TOOL-15 | 6.1 | Each custom checker has a self-test recipe in `check` that runs it on accepted and rejected inputs stored under a name that no tool reads. |
| TOOL-16 | 6.2 | The comment checker reads Rust tokens and ignores literals. |
| TOOL-17 | 6.7 | Landing a checker moves its rule to enforced in every document in the same change. |
| TOOL-18 | 7.1 | Workflows run every check through a recipe. |
| TOOL-19 | 7.2 | Workflow tokens are read-only by default, with extra permissions per job. |
| TOOL-20 | 7.2 | Every action is pinned by a full commit SHA. |
| TOOL-21 | 7.2 | Every download, and every file restored from a cache, is verified against a published SHA-256 checksum before use, and installer actions verify each download and do not fall back. |
| TOOL-22 | 7.2 | No workflow uses `pull_request_target`, reads a secret, or publishes. |
| TOOL-24 | 8 | The licence allow-list starts with `GPL-3.0-or-later` and grows only with a dependency. |
| TOOL-25 | 8 | Path dependencies between members carry a version, and `bans.allow-wildcard-paths` is `false`. |
| TOOL-26 | 8 | Advisories, duplicates, and other sources fail cargo-deny, with exceptions only per item and with a reason. |
| TOOL-27 | 9 | Published crates carry the full metadata, and unpublished members set `publish = false`. |
| TOOL-28 | 10 | Image changes happen in the repository that defines the image, and the first change after a rebuild that adds or removes a tool updates the documents. |
| TOOL-29 | 10 | The image provides the tools of Section 4.8, the stable toolchain as the default, llvm-tools, and GitHub's SSH host keys. |
| TOOL-30 | 3.1 | `taplo.toml` sets four-space indentation and excludes `target/` and `.worktrees/`, and `typos.toml` accepts the correct words that typos reports. |
| TOOL-31 | 8 | `deny.toml` denies unused licence allow-list entries and unused advisory ignores. |
