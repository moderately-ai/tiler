---
id: admit-a-parallel-topology-for-the-identity-less-extrema-fold
title: Admit a parallel topology for the identity-less extrema fold
status: todo
priority: p2
dependencies: [admit-the-softmax-family]
related: [implement-parallel-reduction-strategies, realize-parallel-reduction-strategies-on-metal, admit-the-softmax-family, design-attention-program-vertical]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reductions, extrema, softmax, scheduling]
---
## User-visible outcome

A softmax's maximum pass can be split across partitions or folded cooperatively, which is legal without any numerical permission — so the one pass of the operation that a schedule may parallelize freely stops being the one it cannot parallelize at all.

## Why this is filed

**Fact.** `admit-the-softmax-family` registered `ScalarProgram::StrictSerialMaximum` and admitted it under `ReductionTopology::Serial` alone. `every_topology_but_the_serial_one_is_refused_for_the_extrema_fold` in `crates/tiler-ir/src/schedule/builder.rs` asserts the refusal, with the unmodified serial fixture as the control.

**Inference — the refusal is fail-closed, not a legality claim, and the distinction is the whole reason this is a ticket rather than a defect.** The pinned extrema family is associative and commutative on *every* binary32 input: NaN is absorbing and `-0.0 < +0.0` is a total order, so any tree over the same contributors gives the same bits. `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` states exactly that, and it is the asymmetry the L3′ derivation names — "a fused softmax may parallelize its first pass freely and its second pass only under a permission". The schedule vocabulary currently delivers the opposite of that asymmetry: the *sum* has a multi-pass and a cooperative form and the *maximum* has neither.

**Fact — what actually blocks it is a staged-partial contract, and it is one obligation rather than a family of them.** Both existing parallel paths destructure an `empty_identity_bits` from the scalar program: `verify_multi_pass_semantics` requires it to be `+0.0` and `verify_cooperative_semantics` requires the same, and both do so *because a partition with no contributors must commit something*. The extrema family has no identity — no binary32 value `i` satisfies `Maximum(i, x) == x` for every `x`, because any candidate is itself a possible contributor — so there is nothing to commit and the two verifiers cannot be widened by supplying a constant.

## Required delivery

- **A partial-carrying contract that does not rest on an identity.** [Numerical semantics](../docs/numerical-semantics.md) already names the mechanism — "parallel partials carry `has_value` unless nonemptiness or observably neutral padding is proved" — and this is its first consumer. The choice between proving every partition non-empty at schedule time and carrying an explicit `has_value` is the ticket's design question, and it must be *made* rather than inherited from whichever the sum happens to use.
- **The multi-pass form**, with the partial pass reading the original input as the serial one does, and the final pass folding partials under the same extrema family. A partial that squared or scaled would be a different operation; a partial that *summed* would be a defect this ticket must show cannot be constructed.
- **The cooperative form**, or an explicit statement of why it waits. The staged fold's seed is the first slot, which is admissible for an identity-less family only if every slot was written — the same `TailPolicy::Exact` argument the sum's cooperative tile rests on, restated for a family that cannot pad.
- **The lowering and the emission.** `emit_maximum_reduction` in `crates/tiler-ir/src/kernel/lower.rs` refuses a zero-contributor domain and drives no partitioned addressing; `reduction_prologue` answers `None` for this program, which is what keeps the cooperative lowering from reaching it today. Both are the honest current state and both move here.
- **Evidence that a tree and the serial fold agree bit for bit**, over a corpus containing both zeros, both infinities, and a NaN — the only operands at which associativity could fail. `the_pinned_extrema_family_is_associative_and_commutative_on_every_operand` in `crates/tiler-reference/src/softmax/tests.rs` proves it of the scalar combiner; this owes the same at the schedule level, over an actual split.
- **A perturbation that fires.** Splitting the *sum* pass without reassociation must still refuse, in the same test, or the widening will read as having relaxed both passes.

## Non-goals

Relaxing the denominator sum's order obligations. The two passes' permissions are deliberately separate — `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` and `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` are two facts for exactly this reason — and a change that let one topology decision cover both would be wrong in one direction whichever way it went. A standalone `Maximum`-reduction *key* is also out of scope: this widens a topology for an embedded fold, and the support matrix's `Minimum`/`Maximum` row stays at R2.

## Reconsideration trigger

Active now for correctness of the vocabulary's claims; it becomes a *performance* trigger when a softmax reaches a compiled program, because the maximum pass is then a full serial walk of `S` contributors per row across 448·`T` rows and 28 occurrences, on the workload's one growing extent.
