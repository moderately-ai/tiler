---
id: apply-the-declared-numerical-conformance-on-every-reference-evaluation-path
title: Apply the declared numerical conformance on every reference evaluation path
status: todo
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, conformance]
---
## User-visible outcome

Every reference path that answers for a compiled candidate honours the two subnormal dimensions the candidate's contract resolved, instead of one of three paths honouring them and the other two silently computing the strict reading.

## Why this exists, and why it bites today

**Fact — the declared conformance is applied at three sites in the workspace, all in one module.** `grep -rn 'apply_to_operand\|apply_to_result' crates/ --include='*.rs'` returns fourteen lines at `4cf593e7`: eleven inside `crates/tiler-reference/src/conformance.rs` itself (the two method definitions and nine of its own test assertions) and `crates/tiler-reference/src/oracle.rs:754`, `:755`, `:761`. No other file matches.

**Fact — the semantic evaluator was never told a contract.** `grep -c 'ReferenceNumericalConformance' crates/tiler-reference/src/evaluate.rs` returns `0`. `ReferenceEvaluator::evaluate` computes the strict reading unconditionally, so `ReferenceNumericalConformance::from_realization`'s refusal — which exists precisely so an oracle cannot silently answer a question it was not asked — cannot fire on that path at all.

**Fact — the declared-order reduction oracle takes no conformance either.** `strict_partial_sums` and `strict_partitioned_sum` (`crates/tiler-reference/src/evaluate.rs:484`, `:615`) have signature `(input, axes, partitions, contributors_per_partition)` and fold with a bare host `+` through `canonicalize_arithmetic_f32`.

**Inference — so under `FLUSH_AND_REASSOCIATE_F32` the oracle in use discharges the reassociation dimension correctly and drops the two subnormal dimensions the same contract resolves to a sign-preserving flush.** That contract flushes on both dimensions; the oracle preserves. It fails closed rather than open — a preserving reference disagrees with a flushing device — so the risk is a correct implementation refused and the disagreement misattributed to the grouping.

**Fact — the obligation is already written down, as prose in a test header rather than as an object.** `crates/tiler-reference/tests/contraction_conformance.rs:44-46` states that "A device comparison against this oracle is a comparison against the strict reading, and the flushing dimension has to be declared on the comparison rather than absorbed here." Nothing in the tree is that declaration.

**Fact — it is invisible in every case that exists.** The M4 Max row's operands (`0x3f400000, 0x3e800000, 0x33400000, 0x33000000`) and the CPU-side `REGROUPING_SENSITIVE_INPUT` scaled by `2x + 1` are all normal, so the preserving and flushing readings agree on every value any current case produces.

## What this ticket must produce

- Every reference path that can be asked about a compiled candidate either carries a `ReferenceNumericalConformance` or documents, at the definition, why its subject cannot be affected by either subnormal dimension. Answering by omission is what this ticket exists to end.
- A subnormal-producing case at the reduction shape, which is the population that proves the change can fail: a partial sum that is subnormal under one declared split and not under another, with the exact bit patterns written out.
- The check watched failing before it is believed — revert the threading and confirm the new case refuses.

## Explicit non-goals

Widening `from_realization`'s acceptance (its refusal is correct and [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) says why); any order-witness object; any new dimension; any device run.

## Closes when

No reference path silently answers a contract it was not told, the subnormal-sensitive reduction case exists with exact bits, and the new refusal has been observed.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md), which found the three-site count while deriving what object bounds a program under a permissive contract.
