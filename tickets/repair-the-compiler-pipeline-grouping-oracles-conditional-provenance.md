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

## Coordinator re-audit at `516d42c5`, 2026-08-22 — both Facts verified, and **the defect population is one site, not six**

Every command below was run by the coordinator at this base before dispatch.

**Fact 1 — verified, and the two units agree here.** `grep -o strict_partitioned_sum crates/tiler-compiler/src/pipeline/tests.rs | wc -l` returns **6**; `grep -c` returns **6** as well. No line carries it twice, so this is one of the cases where occurrences and lines coincide — reported so a worker does not go looking for a discrepancy that is not there. Repo-wide the symbol appears in `tiler-reference` (definition, re-export, and five test callers), `tiler-conformance/src/serial_sum.rs`, and the six pipeline-test occurrences.

**Fact 2 — verified.** `crates/tiler-conformance/src/serial_sum.rs` states the condition at the anchor `that is sound only while`, and again at the anchor `is bit-identity on each`.

**The framing "the pipeline-test oracle" is imprecise, and acting on the count would do unnecessary work.** Six occurrences are **three distinct call sites**, and only one carries the defect. Read each before touching it:

- **`:8688` — this is the defect, and the only one.** The test executes the program's own three kernels, then builds `let pointwise_tensor = f32_tensor(shape, &pointwise);` where `pointwise` is `interpret_fused(&kernels[0], &values)` — **the executed prologue output of the program under test**. Both `expected_partials` and `expected` derive from it, so a prologue wrong in the implementation is wrong identically in the oracle and the assertion still passes. This is the silent-agreement failure the Inference above names.
- **`:7965` — already correct, and it is the model for the repair.** It computes `let scaled: Vec<f32> = values[..extent_usize].iter().map(|value| value * 2.0_f32 + 1.0_f32).collect();` independently in the test and feeds that to both the kernel and the oracle. Its own comment states the intent at the anchor `so the reference sees the same contributor values`. **Copy this shape rather than inventing one**; the prologue it reproduces is `value * 2.0 + 1.0`.
- **`:7901`/`:7903` — clean, and it is a guard rather than an oracle.** `the_declared_split_is_what_the_agreement_is_evidence_about` builds `scaled` independently from `REGROUPING_SENSITIVE_INPUT` and executes no kernel at all; it asserts a declared split and a neighbouring split **disagree**, so that the comparison next door is not vacuous. Do not "repair" it — it has no plan-derived input to remove, and weakening it would remove the evidence that the fixture can tell two groupings apart.

**Sibling scan — the coordinator's reading is that `crates/tiler-reference/src/tests.rs` is clean, and this is stated so you can contradict it rather than repeat it.** All five callers there build `input` from literal `f32_tensor(shape, values)` fixtures and execute no kernel, so there is no plan-derived path into any of them. Confirm this by reading each of the five, and report it as a clean result per the Required work; if you find one that is not clean, say so with the evidence — a coordinator claim is secondhand.

**One consequence for the negative control.** The Required work asks you to feed the oracle the executed prologue output and require refusal. Note that at `:8688` today that is not a refusal path at all — it is the *current* passing behaviour, so a control written as "this must fail" has to change the code under it before it can say *no*. State what makes the control reachable before trusting it, and perturb the prologue itself (which is the thing the oracle exists to catch) rather than the assertion.
