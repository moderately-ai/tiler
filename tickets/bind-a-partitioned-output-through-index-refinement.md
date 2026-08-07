---
id: bind-a-partitioned-output-through-index-refinement
title: Bind a partitioned output through index refinement
status: done
priority: p1
dependencies: [admit-a-partitioned-write-ownership-contract]
related: [lower-the-concatenate-occurrence-through-partitioned-writes, accept-the-partitioned-result-binding-boundary, realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, write-ownership, public-boundary]
---
## User-visible outcome

An index region whose output is written by several roots binds through index refinement, so a partitioned region can carry a refinement receipt instead of being refused for having more roots than results.

## Why this exists

**Fact — refinement binds one root per semantic result, by count.** `bind_results` (`crates/tiler-ir/src/index/refinement.rs:2744-2790`) collects `region.outputs()` and returns `IndexRefinementVerificationError::ResultArity { region_outputs, results }` unless the two counts are equal, then zips them positionally. `ResultBinding` carries exactly one `write_access` and one `written_value`.

**Fact — a partitioned region has more roots than results.** [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) admitted several output roots over one tensor; each is a separate `OutputData` entry, so `region.outputs().len()` counts roots rather than distinct output tensors. A two-root partition of one output presents two "outputs" for one semantic result and is refused as an arity mismatch — a refusal for the wrong reason, since the region is well formed and its ownership is proved.

**Fact — the write-completeness check `bind_results` performs is already satisfied.** `access.write_ownership_proof().is_none()` is what it tests, and a partition member carries `WriteOwnershipProof::PartitionMember` rather than `None`. So the obligation this site is guarding is discharged; only the shape of the binding is wrong.

## What the work is

Decide the binding shape for a result whose region writes it in pieces: whether `ResultBinding` carries a set of write accesses, or whether roots are grouped by output tensor before the arity comparison. The second is smaller and keeps the one-binding-per-result invariant every consumer of `ResultBinding` reads; the first is more faithful to what the region holds. Whichever is chosen, `ResultBinding` is a public item and its shape is a public boundary.

Decide what a receipt records about a partition. A receipt naming one of several roots would be a claim the region does not support; naming all of them changes the receipt's identity encoding, which is an identity-domain step to execute completely or not start.

## Explicit non-goals

- The partition contract itself, which exists.
- The compiler-side lowering, which is [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md)'s.

## Closes when

A partitioned region binds through `bind_results` and produces a receipt whose content is justified for every root, or the refusal is preserved with its reason recorded and the dependent lowering told which it gets. A deliberate perturbation dropping one root from the binding is shown to fail.

## Graph maintenance

- `implementation/ir` alone: `refinement.rs` is in `crates/tiler-ir/`.
- Filed by the partition-contract ticket, which read this site in full and left it unchanged because relaxing it is a public-boundary redesign of `ResultBinding` rather than part of admitting the proof form.

## Outcome — 2026-08-07

**A partitioned output binds, and the receipt names every root.** `bind_results` (`crates/tiler-ir/src/index/refinement.rs`) groups the region's output roots by the tensor they write, compares *that* population against the semantic results, and emits one `ResultBinding` per root. The two-root partition `[0, 3)` ∪ `[3, 8)` over an eight-element output now binds where it previously returned `ResultArity { region_outputs: 2, results: 1 }` — a refusal for the wrong reason, since the write-completeness obligation the site guards (`access.write_ownership_proof().is_none()`) is discharged for each member by `WriteOwnershipProof::PartitionMember`.

### Decision 1 — binding shape: one binding per root, grouping before the arity comparison

**Derivation, from reading every consumer rather than from the ticket's framing.** The ticket says the one-binding-per-result invariant is "every consumer of `ResultBinding` reads". It is not: **no consumer anywhere in the workspace reads a `ResultBinding` field.** The complete population is `crates/tiler-compiler/src/legality.rs:314` and `:408`, both `pub fn result_bindings(&self) -> &[ResultBinding]` pass-throughs to the receipt; every other site is a test asserting `len() == 1`; nothing calls `result()`, `output_tensor()`, `write_access()`, or `written_value()`. The kernel emitter, the artifact encoder, and the reference oracle all work from the region, not from these bindings. So the ticket's stated tiebreaker — "if all consumers want per-result answers, grouping wins; if any needs per-root access, the set shape wins" — is answered by a third fact: no consumer wants either, and the deciding evidence is elsewhere.

**What decided it instead: this file already answers the identical question on the operand side.** `OperandBinding` is one binding per *reading stage*, not one per operand — `MAX_INDEX_REFINEMENT_OPERAND_BINDINGS`'s own doc comment says "aliasing can produce more bindings than distinct boundaries", and `bind_operands` pushes one binding per `(operand, expanded boundary, stage)` triple. A result written in pieces is the same many-to-one shape arriving on the other side of the interface, so it takes the same answer. **Rejected alternative: `ResultBinding` carrying a set of write accesses.** It is more faithful to what the region holds, and it was rejected because it would change a public type to serve no reader while making the common one-root case carry a one-element collection; and because the record-per-member encoding is what keeps executable-coverage identity byte-identical for every region that exists today (see Decision 2). **Rejected alternative: naming one root.** Unsupportable, as the ticket says — a receipt naming one member of a partition claims a write the region does not make alone.

**What the grouping is keyed on.** Distinct output *tensors* in first-encounter order, matched positionally against the ordered semantic results. Roots over one tensor need not be authored adjacently, so membership is resolved by tensor rather than by run. Two roots over two *different* output tensors remain two outputs and still refuse against one result.

**One pre-existing wrong admission is closed as a side effect.** The old positional zip compared root count and never checked that the roots wrote distinct tensors, so a region with two roots over one tensor and a subject with two compatible results would have bound result 0 and result 1 to the same output. That is now a `ResultArity` refusal. It was unreachable before the partition contract admitted a second root over one tensor, and no registered law or capability emits one, so nothing that ever built changes.

### Decision 2 — receipt content: every root, with no identity-domain step required

**The identity enumeration, each encoder read.** `ResultBinding` is written into exactly one identity: `encode_executable_coverage_identity` (`refinement.rs`), which writes `result`, `output_tensor.index`, `write_access.index`, `written_value.index` per binding after a record count. It is **not** written into `encode_receipt_identity`, `encode_subject_identity`, `encode_authority_identity`, `encode_resolution_identity`, or `encode_proof_identity`. Downstream of executable coverage: `IndexRefinementExecutableCoverageIdentity` → `CoveredOccurrence::from_receipt` (`crates/tiler-ir/src/program/model.rs:124-130`) → kernel-program identity and, through `tiler-artifact`'s independent stage encoder, the artifact stage key. So the receipt's result bindings **do** reach pinned identity, unlike the proof forms the partition contract added (which `encode_region` never encodes).

**The step was therefore designed out rather than executed.** One record per output root, under the encoder's existing single record count, means a result owning one root writes exactly the bytes it always wrote: first-encounter tensor order is root order, each group has one member, and the four encoded fields are unchanged. The grammar stays self-delimiting without a second nested length, and a partition's grouping stays recoverable from the repeated result ordinal, so the encoding remains injective over the binding structure. **No pinned identity, golden, digest, or ledger moved, and none needed to.** `git diff --stat` touches one file, `crates/tiler-ir/src/index/refinement.rs`; no golden, ledger, or pin file is in the diff. `cargo nextest run --workspace`: **2,916 passed, 0 failed, 7 skipped**, which exercises the 20 pinned explain digests, the six `tiler-metal/goldens/*.metal` shader identity pairs, and `the_artifact_stage_key_encodes_the_same_coverage_record_as_the_kernel_program`.

This is the ticket's *admitting* close, not its recorded-refusal fallback. The fallback was not needed because the ripple was closed by construction rather than absorbed.

### Evidence — three perturbations, each watched failing, each reverted

| Perturbation | Observed | Caught by |
| --- | --- | --- |
| Bind only the first member of each group (`members[position].iter().take(1)`) | 2 failures of 905 | `a_partitioned_result_binds_one_binding_per_root` (1 ≠ 2), `dropping_one_partition_member_changes_executable_coverage` (identical bytes) |
| Collapse the grouping key so every root joins group 0 | 1 failure | `two_distinct_output_tensors_still_disagree_with_one_result` (bound `Ok` where `ResultArity` is owed) |
| Restore the root-count arity comparison (the pre-change state) | 2 failures | both partition tests, with the exact refusal the ticket names: `ResultArity { region_outputs: 2, results: 1 }` |

`a_sole_root_binds_exactly_one_result_to_its_whole_output` passed under all three, which is the point: the dropped-member and grouping perturbations are invisible to a region with one root per output, so the single-root path is untouched.

Four tests ride with the change, all in `refinement.rs`'s test module against `bind_results` directly, with fixtures `partitioned_subject` (a real derived one-result subject) and `partitioned_region` (the landed unequal contiguous partition):

- `a_partitioned_result_binds_one_binding_per_root` — two bindings, both `result() == 0`, same `output_tensor()`, distinct `write_access()`, each carrying `WriteOwnershipProofView::PartitionMember`.
- `a_sole_root_binds_exactly_one_result_to_its_whole_output` — one binding whose access and value are the root's own, with `CoordinatePermutation` ownership.
- `two_distinct_output_tensors_still_disagree_with_one_result` — `ResultArity { region_outputs: 2, results: 1 }`.
- `dropping_one_partition_member_changes_executable_coverage` — the member set is observable in the coverage identity in both directions.

### What the dependent lowering gets

[`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) gets the **admitting** answer, not the preserved refusal. Concretely:

- Emit one write root per operand over the single output; refinement binds them all. There is no need to collapse the roots or to declare one output tensor per operand.
- `receipt.result_bindings()` will carry **one entry per operand**, all with `result() == 0` and the same `output_tensor()`. A consumer needing one answer per result groups by `ResultBinding::result`. Nothing in the compiler reads these fields today, so no lowering-side consumer needs changing.
- The zero-extent operand is a member like any other: it contributes a binding whose write is total over an empty partition, which is what `a_zero_extent_partition_member_owns_nothing_and_is_admitted` already holds at the region layer. The pinned `[8, 0, 128]`-with-`[8, T, 128]` occurrence needs no special case here.
- Its own "confirm whether the pinned explain digest moves" step is unaffected by this ticket: the result-binding encoding did not move, so any digest movement it sees will come from the region and law it registers, not from the binding.

### Commands

```sh
cargo fmt --all --check
cargo clippy -p tiler-ir --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir --no-deps
cargo nextest run --workspace     # 2916 passed, 0 failed, 7 skipped
cargo test --workspace --doc
make full
```

### Public boundary — draft, for Tom

No public type, field, method, or signature changed. What moved is the contract two public items state, filed as [`accept-the-partitioned-result-binding-boundary`](accept-the-partitioned-result-binding-boundary.md) at `awaiting-decision`: `ResultBinding` is one binding per output root rather than one per result; `write_access()` is total over the member's partition rather than always over the whole output; `ResultArity::region_outputs` counts distinct output tensors rather than roots.

### Serialized remainder

`crates/tiler-compiler/src/legality.rs:591-596` still documents its mirrored `region_outputs` as "Region output-root count." The value is unchanged and no test observes a difference, but the comment is now a claim about superseded behaviour. `implementation/compiler` carried a live claim (`region-expansion-exhaustion-loses-the-only-feasible-plan`, `agent-region-expansion`) throughout this dispatch, so the edit was not taken; it is [`realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity`](realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity.md).
