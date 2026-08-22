---
id: split-the-schedule-builder-test-monolith-into-focused-modules
title: Split the schedule builder test monolith into focused modules
status: todo
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue, split-the-artifact-program-test-monoliths-into-focused-modules]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [refactor, maintainability, tests]
---
## User-visible outcome

`crates/tiler-ir/src/schedule/builder/tests.rs` stops being a 7,142-line read-in-full obligation on anyone touching tiler-ir. It becomes a `tests/` directory of subject-named modules with a shared `support/` fixture module, so a worker opens the few hundred lines covering the thing they are changing.

## Why this one, and why now

**Fact — measured at base `ba46f2b2`.** The file is 7,142 lines and has been touched by exactly one commit since `1ab21ef7` — `46bf1319`, the one that created it. Low churn is not an argument against this lane: the file was created by the first tranche, which moved the builder's whole test body into one new file under a zero-test-edit constraint. That was the right call then and is why it is here now. Its four section banners — `Cooperative workgroup tiles`, `The two-dimensional staging relation`, `Operand-sharing cooperative contraction`, `Partitioned-copy region evidence` — are the start of the mapping. `structural_relation_tests.rs` is precedent inside this very directory for a second named test module beside `tests`.

**The seam is already decided by the production side.** `crates/tiler-ir/src/schedule/builder/tests.rs`'s parent directory holds ten non-test children — `contraction`, `copy`, `coverage`, `diagnostics`, `elementwise`, `family`, `intrinsic`, `proof`, `reduction`, `tile` — plus the existing 157-line `structural_relation_tests.rs` sibling. Mirroring those is the seam: a test belongs with the module whose behaviour it asserts. That is a reading task, not a mechanical one — misfiling a test is the failure mode here, so state the mapping rule in `tests/mod.rs` and put anything that spans several children in a named cross-cutting module rather than forcing it into one.

**The layout is settled precedent, not a new invention.** [`split-the-artifact-program-test-monoliths-into-focused-modules`](split-the-artifact-program-test-monoliths-into-focused-modules.md) landed exactly this shape: `tests/mod.rs` declares the children and a `support/` module, then re-exports the fixtures with each item's **original visibility** — 15 `pub(crate)`, 32 `pub(super)`, none lost, widened, or narrowed — so children reach them through their own `use super::*` and out-of-suite consumers import the same names at the same reach. Read that tree before starting. It also generated the split mechanically and reproduced the committed tree byte for byte from the pre-split file; do the same, because it is what makes "pure motion" a check rather than a claim.

**Do not widen visibility to make the split work.** A fixture that only this suite uses stays module-local. The artifact lane recorded thirteen exported names with no consumer outside the suite and deliberately did not narrow them, because narrowing is a separate decision; the inverse applies here — do not create the same debt.

## What must not move

Pure code motion, and the split is only reviewable because of it. No test name, assertion, fixture body, identity constant, encoder byte, pin, or golden changes. A test module's *path* does not change when its file does, so every `mod::path::test_name` stays what it was — state the before/after test count and confirm the two lists are identical. Report a token-stream or dedent-normalized comparison against the pre-split file, as the request and artifact lanes did, so nothing in the diff is a hand edit a reviewer has to re-derive.

## Gates

The first tranche pinned these, and one of its recorded lessons was wrong; the corrected list is:

- `cargo nextest run -p tiler-ir` — the package's own suite, and the test count must be identical before and after.
- `cargo nextest run -p tiler -p tiler-compiler` — the two workspace-invariant scanners live in *different* packages. `workspace_unsafe_sites` is a target of `tiler`; `cited_names_resolve` is a target of `tiler-compiler` and has never lived anywhere else. The census ticket's "both live outside every split's package" is corrected there; `-p tiler` alone would skip the second.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --document-private-items` — the no-private form cannot see intra-doc links in private modules, which is exactly the population a split moves.
- `make citations` — every file move rots line-only citations and only this gate sees it. Fourteen rotted across the request and target splits alone.
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `git diff --check`, `tkt lint`, `tkt guard` against the true base.

## Closes when

`crates/tiler-ir/src/schedule/builder/tests.rs` is a directory whose largest member is under 1,500 lines, the before/after test lists are identical, the gates above are green, and the delivery note records the mapping rule, the per-file line counts, and any test that did not have an obvious home.
