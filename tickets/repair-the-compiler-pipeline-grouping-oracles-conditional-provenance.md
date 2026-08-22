---
id: repair-the-compiler-pipeline-grouping-oracles-conditional-provenance
title: Repair the compiler pipeline grouping oracle's conditional provenance
status: todo
priority: p2
dependencies: []
related: [accept-the-exact-composed-reference-session-and-event-surface]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, conformance, numerics, tests]
---
## User-visible outcome

The compiler pipeline's grouping oracle compares against a value derived independently of the plan under test, so a regression in the prologue cannot make the comparison agree with itself.

## Why this exists

Found 2026-08-22 by the composed-reference re-gate while sizing a population, and recorded separately because it is a different defect from that packet's subject. Verified by the coordinator at `b3c07259`.

**Fact — `strict_partitioned_sum` has callers well outside the crate the original claim scoped.** The composed-reference packet asserted `serial_sum` was the only caller, from a command scoped to `crates/tiler-conformance/src`. `strict_partitioned_sum` occurs **6 times on 6 lines** in `crates/tiler-compiler/src/pipeline/tests.rs` alone — reported as occurrences counted with `grep -o … | wc -l`, cross-checked against `grep -c` for lines, because the two units differ. Further callers exist in `crates/tiler-reference/src/tests.rs`.

**Fact — the conformance-crate module states its own soundness condition.** `crates/tiler-conformance/src/serial_sum.rs` records that its plan-derived comparison is sound only while the pointwise prologue is bit-identity. The pipeline-test oracle carries the same conditional provenance without carrying that statement.

**Inference — the failure is silent agreement, not a wrong answer.** An oracle fed the program's own executed prologue output agrees with the implementation it is meant to check whenever the prologue is wrong in both. That is the shared-implementation failure `docs/correctness-and-testing.md` names.

## Required work

- Re-audit both Facts at your own base and report a per-Fact verdict; re-derive the population rather than trusting the counts above, and say which unit you report.
- Repair the pipeline-test oracle so its expected value is derived without consuming the executed prologue output.
- Add one negative control: feed the oracle the executed prologue output and require refusal. Quote its failure text.
- Check the sibling callers in `crates/tiler-reference/src/tests.rs` for the same pattern. Report both findings and clean results.

## Non-goals

Changing `strict_partitioned_sum` itself; the composed-reference surface; and any change to `crates/tiler-conformance/`.

## Closes when

No grouping oracle in the compiler crate consumes the executed output of the program it checks, the negative control is watched failing, and the sibling scan is reported with its clean results.
