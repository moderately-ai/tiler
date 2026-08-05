---
id: admit-a-parallel-topology-for-the-identity-less-extrema-fold
title: Admit a parallel topology for the identity-less extrema fold
status: review
priority: p2
dependencies: [admit-the-softmax-family]
related: [implement-parallel-reduction-strategies, realize-parallel-reduction-strategies-on-metal, admit-the-softmax-family, design-attention-program-vertical]
scopes: [implementation/ir, contracts/navigation, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reductions, extrema, softmax, scheduling]
claimed_from: todo
assignee: agent-extrema-fold
lease_expires_at: 1785950113
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

## Added scopes and why they belong here

`contracts/navigation` and `research/numerics` were added by the implementing worker, not by a widening of the outcome. Both hold documents that name this ticket and assert the refusal it removes: `docs/roadmap.md` states in two rows that "the extrema fold is admitted under the serial topology alone" and that "a parallel topology for the *maximum* pass is … refused today", and `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md` records the same narrowing. Landing the admission without correcting them would leave three live authorities asserting the opposite of the code. Neither scope was held by any live ticket at the time (`tkt list --status in-progress` named `implementation/compiler`, `contracts/numerics`, `implementation/build`, `implementation/reference`, and `research/program-planning`), and the base commit is the one that closed the navigation-domains ticket. The edits are additive corrective paragraphs in the established "corrected by" idiom; no ADR, contract, or public boundary moved.

`tickets/admit-the-softmax-family.md` also carries the stale sentence, in its follow-on list. It is deliberately left alone: a closed ticket's body is the record of its own moment, and the graph edge to this ticket carries the update.

## Delivered

Landed on `tkt/admit-a-parallel-topology-for-the-identity-less-extrema-fold`.

**The design question, answered.** The choice between proving every partition non-empty at schedule time and carrying an explicit `has_value` is settled as the **proof**, and the derivation is short enough to state: `ContributorPartition::covers` already requires `partitions * contributors_per_partition` to equal the contributor count *exactly* and refuses a zero partition count, and the cooperative admission requires the same product times a nonzero round count. Add the family's own precondition — a non-empty domain — and every factor of a nonzero product is nonzero, so each partition on each round folds at least one contributor and each staged partial is a real maximum. A carried `has_value` would be a runtime flag that is constantly true: storage in every staged slot, a branch in every combine, and a field the verifier could never reject a wrong value of. `empty_domain_is_satisfied` in `crates/tiler-ir/src/schedule/builder.rs` records the derivation beside the check.

**What was admitted.** `EmptyDomainContract` and `SplitFamily` replace the inline destructuring that made the refusal structural: each family now states whether it commits an identity (and which bits) or owes a non-empty domain, and whether splitting it *spends* the reassociation permission. The extrema fold answers `NoIdentity` and `consumes_reassociation: false`, so `ReductionTopology::MultiPass` (both passes) and `ReductionTopology::CooperativeWorkgroup` admit it **under a strict contract** — which is the asymmetry the L3′ derivation names, and not merely a wider admission. The partial pass reads the original scores and the final pass folds the staged partials under the same family; a sum reading the scores is refused, because every sum admitted as a partial pass reads an intermediate or carries a prologue.

**The lowering.** `reduction_prologue` became `reduction_fold`, returning a prologue *and* a `ReductionCombiner`; the cooperative emission threads the combiner through all three levels it folds at — each participant's contributor share, the staged set, and the round accumulator of a loop-carried tile. The multi-pass forms needed no lowering change: `emit_maximum_reduction` already drives `ReadAddressing::Partitioned` and already refuses a zero-contributor domain, which is the identity-less-ness restated where the lowering could still emit.

**Identity verdict: no step, and not even a new tag.** No tag was added, no field moved, no field was inserted. A newly admitted region encodes under the existing scalar-program tag `0x28` and the existing topology tags `0x33`/`0x35`, each in its existing position with its existing field layout, so every previously encodable region maps to exactly the same bytes — pinned by `the_strict_f32_region_has_its_recorded_canonical_identity`, whose constant is unchanged. Per-tag injectivity is preserved because the newly reachable byte strings carry a `(0x28, 0x33)` or `(0x28, 0x35)` tag pair that no earlier region could produce: the scalar-program tag separates the family and the topology tag separates the split, and `a_split_extrema_region_has_its_own_canonical_identity` shows the five neighbouring regions pairwise distinct.

**Evidence, and the perturbations watched failing.** `a_split_of_the_extrema_fold_agrees_with_the_serial_fold_bit_for_bit` enumerates every assignment of `{+0.0, -0.0, 1.0, -1.0, +inf, -inf, NaN}` to the six contributor positions of the split a *verified* region declares, and requires the tree and the serial fold to agree bit for bit; its two controls are an absorbing sum sequence that the same split boundaries *do* change, and a corpus pair where the folded family differs from `maxNum`. Watched failing: setting `consumes_reassociation: true` for the extrema breaks all three strict admissions; disabling the `NoIdentity` arm admits an empty split; and replacing each of the three cooperative combiner sites with `F32Add` breaks the corresponding lowering count.

**One test named above no longer exists, deliberately.** `every_topology_but_the_serial_one_is_refused_for_the_extrema_fold`, cited under "Why this is filed", asserted a refusal this ticket removes; it survived the widening green, because it set a parallel topology on the *serial* fixture and so still failed on shape rather than on topology — a test whose name had stopped describing what it proved. It is now `a_topology_that_describes_no_fold_is_refused_for_the_extrema_family`, covering `ReductionTopology::None` and `ReductionTopology::Contraction`, which remain refused and for reasons different in kind from each other.

**Measurement boundary.** The family restated in the schedule tests is a restatement, not the authority — `tiler-ir` cannot call `tiler-reference`'s `maximum_f32`, because the dependency runs the other way — so the bit-for-bit claim is over that restatement, guarded by the `maxNum` control. No device executes any of this; the evidence is the schedule verifier's admissions and the structured-kernel body's operations.

## Reconsideration trigger

Active now for correctness of the vocabulary's claims; it becomes a *performance* trigger when a softmax reaches a compiled program, because the maximum pass is then a full serial walk of `S` contributors per row across 448·`T` rows and 28 occurrences, on the workload's one growing extent.
