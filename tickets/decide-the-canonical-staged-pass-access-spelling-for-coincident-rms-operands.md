---
id: decide-the-canonical-staged-pass-access-spelling-for-coincident-rms-operands
title: Decide the canonical staged-pass access spelling for coincident RMS operands
status: in-progress
priority: p1
dependencies: [admit-the-rms-normalization-family, accept-the-root-mean-square-scale-realization-law, decide-the-full-list-access-coordinate-for-out-of-list-references]
related: [repair-retired-declared-input-order-authority-in-request-and-physical-comments, admit-coincident-rms-operands-with-one-coalesced-pass-access]
scopes: [contracts/decisions, contracts/foundation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, identity, normalization, schedule]
claimed_from: todo
assignee: worker-staged-pass-review
lease_expires_at: 1787061091
---
## User-visible outcome

Tiler either retains the typed refusal for `rms_norm(x, x)` or accepts one exact canonical staged-pass access spelling for two semantic RMS operands which bind the same declared input. No implementation may admit the population by accidentally choosing between two local accesses and one coalesced access.

## Exact-base Fact audit — 2026-08-17 at `e81905ac03db2a7c69762f2793b07b12ac1f32b1`

The behavioral audit was derived at `3969f46cc94ad296bba46885b2688f8a6124bb55`. The only relevant later source movement through this base is the comments-only declared-input-authority repair in `physical.rs` and `request.rs`; the construction, refinement, schedule, kernel, program, reference, and identity paths below are unchanged.

1. **Verified — the semantic occurrence is legal, ordered, and reference-defined.** `F32RmsNorm::apply` supplies value and weight as two operand positions without requiring distinct handles. `SemanticProgramBuilder::push_operation` and semantic identity retain both positions. `RmsNormF32::infer` checks arity, F32 types, shapes, attributes, axis, and epsilon, not handle inequality. Request recognition projects `rms_norm(x, x)` as ordered reads `[Input(0), Input(0)]`. The registered reference receives two ordered tensors and computes the accepted formula when both resolve to the same value.
2. **False as the first current refusal — physical construction is not where ordinary compilation stops.** `IndexRefinementSubject::derive`, anchor `.position(|candidate| *candidate == value)`, coalesces equal semantic values into one boundary and retains `operands() == [0, 0]`. Both `realize_root_mean_square_scale`, anchor `rms-scale-arity`, and `RootMeanSquarePlan::derive` currently require two distinct boundary records before checking `[0, 1]`. The governed lowering therefore refuses under nested `rms-scale-arity`, surfaced publicly as `UnsupportedCapability { phase: "lowering", rule: "refinement-refused" }`. The later `value_input == weight_input` guard is real but unreachable until the law and governed lowering admit this subject.
3. **Imprecise — two local accesses do not uniquely preserve semantic operand authority.** Public `AccessOrdinal` is a schedule-access coordinate, not an operand-use coordinate. The canonical operand interface already retains `[0, 0]`, and `OperandBinding` deliberately binds each ordered operand use to every stage input carrying that occurrence-local value. Existing aliased-operand tests prove two operand uses can bind one input tensor. A second dense pass access would add structure, not restore otherwise-lost operand order.
4. **Verified — current grammars can represent the canonical admission without a new public type.** A coalesced realization can retain fold sources `[Occurrence(0)]`, pass sources `[Occurrence(0), Intermediate(0)]`, one dense pass leaf used twice in `weight * (value * root)`, four ordered operand bindings, and three pass buffer bindings. The same existing types also keep a distinct dense read and a mapped read of one declaration separate, so this is not a general declared-input deduplication rule.
5. **Verified — admission changes three authorities even though no grammar must step.** The RMS registered realization-law row must move from revision 1 to 2; the RMS lowering capability must move from revision 1 to 2 without raising every governed capability; and the governed physical provider must move from revision 1 to 2 because its proposal population changes. The realization and lowering registry identities are folded into every governed request subject, while reached RMS executable and selected-provider provenance also move. Existing semantic, refinement, sequence, schedule-v6, kernel-v8, kernel-program-v11, artifact-v18, and manifest-18.0 grammars already distinguish all affected structures, so no enum tag, identity-domain, or schema step is justified.
6. **Verified — deleting only the physical guard would admit nothing.** The law and governed lowering would still refuse the one-boundary subject. A complete implementation must update both authorities, choose the canonical coalesced realization, then remove or make unreachable the physical guard with exact end-to-end evidence.

The original decision purpose survives, but the source audit removes the supposed two-access-versus-one-access public representation tradeoff. Among admission spellings, one coalesced pass access dominates.

## Eliminated admission spellings

### Two operand-position pass accesses

This can be made correct, but it is dominated under current authority. Both pass reads have the same value, shape, dense relation, lifetime, and policy; ordered semantic operands and alias structure already survive as `[0, 0]` plus two `OperandBinding`s per reading stage. The extra access adds one schedule access, KIR buffer/load, program binding, artifact ABI slot, runtime routed binding, and an arbitrary tie order. The AccessOrdinal decision preserves positions a producer authors; it does not require one access per semantic operand use.

Reversal evidence: an accepted rule requiring one access per operand use, or a real consumer needing distinct use-local relation, storage, or provenance for exact same-value dense reads.

### Explicit operand-to-access map

Dominated. The accepted law identity, ordered operand interface, staged source list, and refinement receipt already preserve the use association. A second public map would duplicate authority and add validation, identity, and storage without admitting another correct current program.

Reversal evidence: a correctness-bearing consumer that cannot derive its needed use-local fact from those existing authorities.

### Two logical accesses coalesced only in KIR or the artifact

Dominated. It requires a new checked many-access-to-one-buffer relation contrary to the current one-buffer-per-read invariant, while schedule-level coalescing expresses the same meaning with less state.

### A dedicated alias-only physical provider

Dominated as a compatibility variant. It preserves the current governed provider revision by adding another provider and selection path solely for a pre-production identity convenience. Revising the one governed provider is simpler and more maintainable.

## Pareto frontier

### Retain the typed refusal

Correct, fail-closed, zero implementation and host-runtime cost. The exact reason remains nested `rms-scale-arity` and public `refinement-refused`, which is less actionable than the eventual physical guard but truthful.

Strongest counterargument: the semantic layer, accepted law shape, request boundary, and registered reference already define this ordinary alias population; refusal leaves a natural, fully meaningful program unsupported.

Reversal evidence in its favour: numerical disagreement on `rms_norm(x, x)`, failure of exact alias refinement, or evidence that the accepted law intended distinct value identities rather than two ordered operand uses.

### Admit one coalesced pass access — recommended

Widen the accepted RMS law and governed lowering to accept one boundary with ordered operands `[0, 0]`. The canonical two stages are:

- fold sources `[StagedInputSource::Occurrence(0)]`;
- pass sources `[StagedInputSource::Occurrence(0), StagedInputSource::Intermediate(0)]`;
- one dense pass input leaf used twice in `weight * (value * root)`.

The verified receipt retains four operand bindings—both semantic uses associated with the one input tensor in each stage—and the pass retains two reads plus its write. Exact same-value plus identical dense relation is the only coalescing case; distinct operands retain the existing three-read pass, and one declaration read densely and through another relation remains two accesses.

This adds no public type or field and no runtime alias/ownership rule. It changes the admitted population and three authority revisions named above. Host memory is lower than the eliminated two-access admission by one access/buffer/binding row per reached alias occurrence; kernel work also avoids a redundant identical load.

Strongest counterargument: it can look like forbidden operand deduplication and does not pre-authorize a hypothetical future per-use access policy.

Reversal evidence: a consumer requiring different access policy for the two identical dense uses, or a subject perturbation showing that the accepted scalar program or operand receipts lose ordered-use identity under coalescing.

## Required evidence if admission is accepted

- Build exact `rms_norm(x, x)` and prove semantic/request operands remain two ordered uses while the refinement subject has one input and `[0, 0]`.
- Evaluate it through the registered reference and the direct RMS oracle with bitwise expected results.
- Pin the two source lists, one shared pass leaf used twice, exactly four operand bindings, and exactly three pass buffer bindings.
- Perturb only weight to a distinct value and recover the existing three-read pass and its identities.
- Perturb one read to a different logical relation and prove it remains distinct; do not turn this into a global input-deduplication pass.
- Perturb the scalar expression's second use and show bitwise disagreement.
- Move the RMS law revision, RMS capability revision, and governed physical-provider revision independently and prove the exact registry/request/proposal/executable identities each owns.
- Preserve every existing distinct-operand RMS structural identity except the selected provenance that the required provider revision intentionally moves.

## Exact decision question

Accept the recommended coalesced-access admission and its three revision moves, or keep the current typed refusal? Typed deferral is equivalent to retaining the refusal until named reversal evidence appears.

Do not present this question until an independent exact-current review confirms the two-survivor frontier, identity consequences, graph, and testable subject perturbations.

## Non-goals

No general tensor-alias analysis, in-place mutation, global operand or declared-input deduplication, runtime buffer ownership redesign, new public access map, domain/schema step, or other staged-family widening is implicit in this decision.

## Graph consequence

[`admit-coincident-rms-operands-with-one-coalesced-pass-access`](admit-coincident-rms-operands-with-one-coalesced-pass-access.md) is the blocked implementation carrier. It must not move until Tom accepts admission here. If refusal or deferral wins, close that carrier without production edits and retain the current fail-closed path.
