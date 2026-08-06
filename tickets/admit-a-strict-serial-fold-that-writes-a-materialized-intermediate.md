---
id: admit-a-strict-serial-fold-that-writes-a-materialized-intermediate
title: Admit a strict serial fold that writes a materialized intermediate
status: todo
priority: p2
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary, admit-a-reduction-over-a-declared-input-tensor]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, schedule]
---
## User-visible outcome

A `ScalarProgram::StrictSerialSum` region under a `ReductionTopology::Serial` may commit its result to `TensorRole::Intermediate` instead of only to `TensorRole::Output`, so `sum(x * x) * scale` has a producer region for the value its epilogue reads.

## Why this exists

**Fact — this is the second of the two walls `materialized_intermediate_epilogue_wall.rs` measured, and the only one still standing.** That file's table records three rows: an elementwise epilogue reading `TensorRole::Intermediate` (admitted by [`admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`](admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md)), a contraction writing one (already admitted by `verify_contraction`, which accepts `TensorRole::Intermediate | TensorRole::Output` for the owning write), and a serial fold writing one — refused, and pinned by `a_strict_serial_sum_region_cannot_write_a_materialized_intermediate` with a control that differs only in the write's role.

**Fact — the refusing rule, read from the verifier.** `verify_access_and_semantics` in `crates/tiler-ir/src/schedule/builder.rs` admits the serial `StrictSerialSum` arm only under `write.tensor == TensorRole::Output`. Every other serial arm — `FusedMultiplyAddSerialSum`, `SquaredSerialSum`, `StrictSerialMaximum` — carries the same requirement. `multi_pass_family`'s partial pass is the one fold that writes an intermediate today, and it is a different topology declaring a split rather than a fold whose result another region consumes.

**Inference — the consequence for the epilogue chain.** `contract(a, b) * 2.0` needs only the read, and both of its halves are expressible now. `sum(x * x) * scale` needs this write as well, so `admit-elementwise-epilogues-over-a-materialized-intermediate` can deliver its contraction shape and its published-and-consumed copy stage without this ticket, and not its reduction shape.

## Boundaries

- **A widening of an admitted write, not a relaxation.** A fold committing to an intermediate must prove its ownership, its bounds, and its contributor relation exactly as the output-writing one does; only which boundary tensor receives the committed value moves.
- **Decide the arm set deliberately, and state the decision.** The three prologue-carrying serial families sit beside the bare sum in the same match. Widening only `StrictSerialSum` leaves a reader unable to derive why `SquaredSerialSum` may not stage its result; widening all four is a larger claim than the epilogue chain needs. Either is defensible — the derivation is what must be recorded, in the shape [`admit-a-reduction-over-a-declared-input-tensor`](admit-a-reduction-over-a-declared-input-tensor.md) used for `ContributorTensor`.
- **The multi-pass and cooperative arms already answer this question separately** and must not be folded into the same predicate by accident: their pass distinction is about which tensor holds the *contributors*, not about where the result is committed.
- **Identity.** `push_tensor_role` already writes a distinct self-delimiting tag for `Intermediate`, and the write role already reaches `encode_identity` through both the access list and the ownership proof, so admitting the value is expected to be appends-only under per-tag framing. Determine that at the owning encoding site rather than inheriting this sentence.

## Closes when

A `StrictSerialSum` region under a serial topology whose owning write targets `TensorRole::Intermediate` verifies, with its ownership and bounds proofs discharged exactly as the output-writing control's are; a region that widened the rule too far is observed refusing; and `materialized_intermediate_epilogue_wall.rs`'s serial-sum assertion is inverted in the same change that lifts it, rather than deleted.
