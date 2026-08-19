---
id: split-the-schedule-builder-into-cohesive-submodules
title: Split the schedule builder into cohesive submodules
status: done
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, maintainability, ir]
---
## User-visible outcome

`crates/tiler-ir/src/schedule/builder.rs` (10,554 lines at filing, the workspace's second-largest source file) becomes a `schedule/builder/` directory of cohesive submodules, each small enough to read in one sitting, with the public surface, behaviour, and every identity byte unchanged.

## Why this exists

Filed 2026-08-19 from Tom's module-size directive in the live session. A 10k-line builder makes full-file reads — the repository's own review standard — disproportionately expensive, and hides seams between construction, validation, and proof concerns.

## Required work

- Read the file in full first; derive the split from its actual cohesion seams (construction steps, validation families, proof plumbing, error surfaces), not from line counts. Prefer a handful of well-named submodules over many fragments.
- Convert `builder.rs` to a `builder/` directory module. **Do not edit `schedule/mod.rs`**: the existing `mod builder;` declaration must keep working unchanged — this fence exists because a concurrent migration branch adds lines to `schedule/mod.rs` and the two diffs must stay disjoint.
- Pure code motion: no public item added, removed, renamed, or re-signatured; internal visibility widened only to the minimum (`pub(super)`/`pub(crate)`) the split forces, each widening noted in the delivery report. Keep module docs with the code they document; write a short module doc for each new submodule saying what lives there and why the seam is where it is.
- **No identity, pin, encoding, or behaviour movement.** Every existing test — including every pinned digest and golden — must pass byte-identically with zero test edits. A split that forces a pin or test change is wrong; stop and report rather than adjusting the test.

## Evidence and checks

`cargo check -p tiler-ir`, `cargo nextest run -p tiler-ir`, `cargo test -p tiler-ir --doc`, clippy with warnings denied, `cargo fmt --check`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir`, `tkt lint`, `git diff --check`, `tkt guard` against the true base. Report the submodule inventory (name, line count, one-line charter), every visibility widening, and confirmation that no test file changed.

## Non-goals

Behaviour or API changes, moving code between crates, touching `schedule/mod.rs` or any other schedule file, and splitting other large files (each has its own ticket).

## Closes when

The directory split lands with all gates green, zero test-file edits, and the reader-facing inventory recorded in the delivery report.

## Delivery note — 2026-08-19, `46bf1319f6b8cbdcc3e8aa26e18eda9801289a7d`

**Measurement — `make citations` goes red on this branch and the repair is outside this ticket's fence.** Fifteen pinned line-only citations in seven documents name `crates/tiler-ir/src/schedule/builder.rs:NNN`; deleting the file makes every one of them unresolvable (`check-citations` reports `has 0 lines`). Three are accepted ADRs — [`0012-physical-reduction-topology`](../docs/decisions/0012-physical-reduction-topology.md) `:391`, [`0014-reassociation-vs-permutation`](../docs/decisions/0014-reassociation-vs-permutation.md) `:831`, [`0022-reduction-identities-and-initial-values`](../docs/decisions/0022-reduction-identities-and-initial-values.md) `:403` — and the rest are live research documents: [`permitted-divergence-oracle`](../docs/research/reference/permitted-divergence-oracle.md) `:416` and `:4767`, [`plan-freedom-sites`](../docs/research/reference/plan-freedom-sites.md) `:620-632`, `:906`, `:909`, `:1029`, `:1516`, [`two-dimensional-cooperative-staging-relation`](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) `:1147`, `:1671`, `:4484`, and two 2026-08-10 ticket-audit reports at `:1516` and `:664`. This ticket holds `implementation/ir` and `project/tickets` only, so the repair needs its own ticket carrying a documentation scope; it should convert each citation to a quoted anchor rather than a new line number, since every one of these is exactly the line-only form that rotted here.

Nothing in the split forced a test, pin, or golden to move: the four identity constants and every `fn` name are byte-identical to the base file, and the two `#[cfg(test)]` modules keep their paths (`schedule::builder::tests`, `schedule::builder::structural_relation_tests`) because a module's file location does not change its path.
