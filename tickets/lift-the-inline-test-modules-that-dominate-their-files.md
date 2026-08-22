---
id: lift-the-inline-test-modules-that-dominate-their-files
title: Lift the inline test modules that dominate their files
status: todo
priority: p3
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue, split-the-compiler-target-module-into-cohesive-submodules]
scopes: [implementation/compiler, implementation/ir, implementation/build]
shared_scopes: []
paths: []
tags: [refactor, maintainability, tests]
---
## User-visible outcome

Four files stop reporting themselves as large source modules when they are mostly test text, so `wc -l` — the instrument the module census runs on — stops lying about where the code is.

## Why these four, and not the other ninety

**Fact — measured at base `ba46f2b2`.** 94 files carry a trailing inline `#[cfg(test)] mod … { … }`, totalling 63,331 lines, and 53 files declare `mod tests;` against a sibling file. The two populations overlap in exactly two files — `crates/tiler-compiler/src/pipeline.rs` and `crates/tiler-macros/src/aot.rs`, each of which has lifted its main suite and kept a small named one inline — so both conventions are live and neither is a deprecation of the other. In four files the inline block is 80% or more of the file:

| file | total | inline tests | share |
| --- | ---: | ---: | ---: |
| `crates/tiler-compiler/src/target.rs` | 3,817 | 3,498 | 92% |
| `crates/tiler-ir/src/semantic/accuracy/domain.rs` | 1,034 | 933 | 90% |
| `crates/tiler-ir/src/index/sourced.rs` | 2,558 | 2,262 | 88% |
| `crates/tiler-build/src/metal_plan.rs` | 2,300 | 1,862 | 81% |

**`metal_plan.rs` is the most-touched file over 2,000 lines in the workspace** — 13 commits between `1ab21ef7` and `ba46f2b2`, ahead of `pipeline/tests.rs` at 11 and `physical.rs` at 10 — and 81% of what a worker opens to reach it is test text. That is the strongest single case in this ticket and it is invisible to a size ranking.

**These four and not the rest, on purpose.** Lifting all twenty inline modules over 1,000 lines would churn 41,686 lines for a change no compiler, linter, or rustdoc can observe, and it would not reduce anyone's concern count. The argument for *these* is different and narrower: at 80% and up the file's headline size overstates its source half by five to twelve times — `target.rs` reads 3,817 for 319 lines of code, `accuracy/domain.rs` 1,034 for 101 — and the census that orders every other split in [`keep-a-module-size-and-complexity-census-with-a-split-queue`](keep-a-module-size-and-complexity-census-with-a-split-queue.md) reads exactly that headline. `target.rs` is the case the first tranche already recorded as a follow-up, correctly declined in-tranche under its zero-test-edit constraint. The remaining files with large inline modules should have them lifted **as part of their own concern split**, when a lane is opening the file anyway — not in a standalone churn pass.

## The seam, and the one thing to get right

Move each inline `#[cfg(test)] mod … { … }` body into a sibling file — `target/tests.rs`, `accuracy/domain/tag_injectivity_tests.rs`, `index/sourced/tests.rs`, `metal_plan/tests.rs` — replacing the block with `#[cfg(test)]` + `mod tests;`. A module's path does not change when its file does, so every existing test path survives. `crates/tiler-compiler/src/target/` already exists and only gains a file; the other three parents become directories, which is what rots their citations.

**One of the four modules is not named `tests`,** and the file it moves to must take its actual name. `crates/tiler-ir/src/semantic/accuracy/domain.rs` declares `tag_injectivity_tests`; the other three declare `tests`. The convention is not universal elsewhere either — `index/scalar.rs` has `governed_fact_tests`, `kernel/model.rs` has `injectivity_tests`, `pipeline.rs` has `explain_capacity_scope_tests` — so read the `mod` line rather than assuming. Renaming a module would change every test path inside it, which is exactly what this lane must not do.

**Leave the parent's `#[cfg(test)] use` shims where they are.** `target.rs` in particular carries a block of imports that exist only so the test module's `use super::*` resolves, and it says so in a comment. They stay in the parent. A child module in a separate file *does* reach the parent's private `#[cfg(test)]` imports through `use super::*` — visibility is by module path, not by file — so this is pure motion with no import edits.

**Fact — both directions of that were run rather than assumed** (throwaway crate, `rustc` 1.97.0, 2026-08-22). A parent holding a `#[cfg(test)]` import of `std::sync::Arc` plus `#[cfg(test)] mod tests;`, with the child doing `use super::*` and naming `Arc`, compiles and the test passes. Deleting only the parent's shim — perturbing the subject, not the assertion — and rerunning gives:

```text
error[E0425]: cannot find type `Arc` in this scope
 --> src/parent/tests.rs:5:17
error[E0433]: cannot find type `Arc` in this scope
 --> src/parent/tests.rs:5:28
help: consider importing this struct
  |
1 + use std::sync::Arc;
```

The shim is load-bearing and reachable from the child, which is what makes leaving it in the parent the correct move rather than the lazy one.

**The diff will be large and must be mechanical.** Every moved line loses four columns of indentation. Verify with a dedent-normalized comparison against the pre-move file — the moved body must be byte-identical modulo the dedent and whatever `rustfmt` re-flows once the four columns are freed, and the request lane's note is the model for reporting exactly which lines `rustfmt` touched and why.

## What must not move

No test name, assertion, fixture body, or `#[cfg(test)]` item changes, and no visibility is widened. The before/after test lists must be identical, by name.

## Also fix, because the lane is already in the file

`crates/tiler-compiler/src/target.rs` carries a stale comment: the sentence anchored `two crate-private children of this cluster` sits above three `pub(crate) mod` declarations — `accuracy`, `feasibility`, `honourability` — and the sentence after it describes only the latter two. The first tranche recorded it and did not fix it. Fix it here, in its own commit, and say what the correct count and reading are.

## Scheduling note

Three exclusive scopes for one lane is a wide footprint, and it is deliberate: the decision this ticket makes is a *convention* — where a lifted module lives, what it is called, and whether its parent's `#[cfg(test)]` shims move with it — and splitting the lane per crate would fork that decision three ways. If the scopes collide with live work, sequence the lane rather than splitting it, or take `implementation/compiler` first since `target.rs` is both the largest case and the one the first tranche already recorded.

## Gates

- `cargo nextest run -p tiler-compiler -p tiler-ir -p tiler-build` — identical test counts before and after.
- `cargo nextest run -p tiler -p tiler-compiler` — `workspace_unsafe_sites` is a target of `tiler` and `cited_names_resolve` is a target of `tiler-compiler`; neither package is a substitute for the other.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --document-private-items`.
- `make citations` — three of the four files become directories, which rots every line-only citation naming them.
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `git diff --check`, `tkt lint`, `tkt guard`.

## Closes when

The four inline modules are sibling files, leaving source halves of 319 (`target.rs`), 296 (`index/sourced.rs`), 438 (`metal_plan.rs`), and 101 (`accuracy/domain.rs`) — verify those four numbers at your own base before trusting them, since three days moved this ticket's predecessor by fourteen lines — the test lists are identical, the `target.rs` comment is repaired, and the gates above are green. Record the new census rows so [`keep-a-module-size-and-complexity-census-with-a-split-queue`](keep-a-module-size-and-complexity-census-with-a-split-queue.md) can be updated from the delivery note rather than re-derived.
