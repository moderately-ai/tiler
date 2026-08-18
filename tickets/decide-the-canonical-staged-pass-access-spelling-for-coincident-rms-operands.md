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
5. **Verified — admission changes three authorities even though no grammar must step.** The RMS registered realization-law row must move from revision 1 to 2; the RMS lowering capability must move from revision 1 to 2 without raising every governed capability; and the governed physical provider must move from revision 1 to 2 because its proposal population changes. The realization and lowering registry identities are folded into every governed request subject; the law row is folded into every RMS occurrence's refinement subject, so the law and capability revisions move every reached RMS subject, refinement, and executable identity — distinct-operand occurrences included — and the physical-provider revision moves selected-provider provenance for every governed plan, not only RMS. Existing semantic, refinement, sequence, schedule-v6, kernel-v8, kernel-program-v12, artifact-v18, and manifest-18.0 grammars already distinguish all affected structures, so no enum tag, identity-domain, or schema step is justified. *(Repaired 2026-08-18: the audit's `kernel-program-v11` was correct at `e81905ac`; `e5f1720d` stepped the domain to `tiler.kernel-program.v12` for unrelated stage-ownership reasons, and the blast-radius sentence was widened — see the independent review below.)*
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

### A compiler-materialized second operand copy

*(Added by the 2026-08-18 independent review.)* Eliminated for correctness. Admitting `rms_norm(x, x)` by synthesizing a copy occurrence or a duplicate buffer would compile a program nobody wrote: recognition binds the authored operands (`recognize_staged_family` carries one `BoundaryRead` per authored operand position), semantic identity retains two ordered uses of one value, and a materialized duplicate is an implicit copy with unstated placement, lifetime, and identity. It also costs one buffer and one redundant load per reached occurrence where the coalesced admission costs nothing.

Reversal evidence: none plausible; a copy that becomes semantically meaningful would be an authored broadcast or reindex occurrence, which is already a different program.

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
- Preserve every existing distinct-operand RMS schedule-region and kernel canonical identity byte-for-byte; those carry no semantic or authority correlation and must not move. Enumerate rather than preserve the identities that fold the three revisions, because they necessarily move for distinct-operand RMS too: the refinement subject and receipt through the registered law row, occurrence and lowering provenance through the capability revision, and — for every governed plan, not only RMS — selected-provider provenance through the physical-provider revision. *(Rewritten 2026-08-18; the prior blanket-preservation spelling was unsatisfiable — see the independent review below.)*

## Exact decision question

Accept the recommended coalesced-access admission and its three revision moves, or keep the current typed refusal? Typed deferral is equivalent to retaining the refusal until named reversal evidence appears.

Do not present this question until an independent exact-current review confirms the two-survivor frontier, identity consequences, graph, and testable subject perturbations.

## Non-goals

No general tensor-alias analysis, in-place mutation, global operand or declared-input deduplication, runtime buffer ownership redesign, new public access map, domain/schema step, or other staged-family widening is implicit in this decision.

## Graph consequence

[`admit-coincident-rms-operands-with-one-coalesced-pass-access`](admit-coincident-rms-operands-with-one-coalesced-pass-access.md) is the blocked implementation carrier. It must not move until Tom accepts admission here. If refusal or deferral wins, close that carrier without production edits and retain the current fail-closed path.

## Independent review — 2026-08-18 at `973a4a56f1e8e5d7f3d7c58dd1fa7828868fa43f`

This is the independent exact-current review the decision question requires; it discharges that precondition. Every Fact was re-derived by reading the named sources in full at this base, not by re-checking the 2026-08-17 audit's text, and every anchor below was grepped against the file it names before this section was committed.

**Source movement since the audit base.** `git log --oneline e81905ac..973a4a56 -- crates/` lists seven commits touching 38 files. None touches the RMS behavioral chain — `semantic/rms_norm.rs`, `semantic/standard_operations.rs`, `request.rs`, `index/refinement.rs`, `index/law.rs`, `governed.rs`, `capability.rs`, `legality.rs`, `lowering.rs`, and `physical.rs` are all absent from the diff — and `schedule/model.rs` moved by one comment. One Fact drifted: `e5f1720d` stepped the kernel-program identity domain to `tiler.kernel-program.v12` (`crates/tiler-ir/src/program/model.rs`, anchor `tiler.kernel-program.v12`). Fact 5 was repaired in place; v12 folds strictly more canonical stage ownership than v11, so the conclusion that admission justifies no further step survives re-derivation.

**Per-Fact verdicts.**

1. **Verified.** `F32RmsNorm::apply` (`crates/tiler-ir/src/semantic/standard_operations.rs`, anchor `value.erase(), weight.erase()`) passes two ordered positions with no distinct-handle requirement; `push_operation` (`crates/tiler-ir/src/semantic/program.rs`, anchor `ValueRole::OperationOperand`) stores both operand indices; `RmsNormF32::infer` (`crates/tiler-ir/src/semantic/rms_norm.rs`) checks arity, F32 types, shape equality, attribute count, axis, and eps, never handle inequality; `recognize_staged_family` (`crates/tiler-compiler/src/request.rs`, anchor `declared.contains(operand)`) maps both operands of `rms_norm(x, x)` to `BoundaryRead::Input` at ordinal 0; `RmsNormF32Reference::evaluate` (`crates/tiler-reference/src/rms_norm.rs`) accepts two ordered tensors of one shape and computes the pinned formula.
2. **Verified, with one stated precision.** The coalescing anchor `.position(|candidate| *candidate == value)` sits in `IndexRefinementSubject::derive`, which yields one input boundary and `operands() == [0, 0]` for the coincident subject. `realize_root_mean_square_scale` (`crates/tiler-ir/src/index/law.rs`, anchor `rms-scale-arity`) and `RootMeanSquarePlan::derive` (`crates/tiler-compiler/src/governed.rs`, same anchor) each destructure `([value, weight], [result])` before their `[0, 1]` check, so the one-boundary subject refuses at arity. `resolve_lowering` runs before any cover (`crates/tiler-compiler/src/pipeline/planning.rs`, anchor `Lowering-capability resolution precedes every cover`), making `physical.rs`'s `value_input == weight_input` guard unreachable for this subject; `LoweringError::Refine` maps to `refinement-refused` (`crates/tiler-compiler/src/lowering.rs`, anchor `"refinement-refused"`) under phase `"lowering"` (`planning.rs`, `lowering_failure`). Empirically confirmed at this base by a disposable probe (two integration tests, deleted after the run; `git status` shows `crates/` byte-identical): `rms_norm(x, x)` over one `[2, 2]` input under `staged_rms_profile(RmsRealizationFixture::Discharging)` and `STRICT_F32` refuses with class `UnsupportedCapability { rule: "refinement-refused" }`, while the distinct-operand control compiles under the identical request. Precision: on a profile with no installed elementary rsqrt realization — including bare `TargetProfile::governed()` — every RMS occurrence, coincident or not, refuses earlier with `accuracy.elementary.no-installed-realization`; the refinement refusal is the first *alias-specific* stop on any profile that admits distinct-operand RMS at all, which is the population this decision is about.
3. **Verified.** `AccessOrdinal` is region-local and positional, "deliberately not an interface key and not a semantic value" (`crates/tiler-ir/src/schedule/handles.rs`); `canonical_operand_interface` (`crates/tiler-compiler/src/legality.rs`) writes one entry per ordered operand use, both naming input 0; `bind_operands` (`crates/tiler-ir/src/index/refinement.rs`, anchor `boundary can be claimed by several stages`) binds each ordered operand position to every stage input carrying that boundary; the aliased-operand verification test (same file, anchor `aliases one input into both operands`) exercises exactly two uses binding one input tensor.
4. **Verified.** `PointwiseF32ExpressionBuilder::input` (`crates/tiler-ir/src/schedule/pointwise.rs`, anchor `returns the leaf already minted`) shares one leaf across several uses by construction, so a two-access pass whose dense leaf is used twice in `weight * (value * root)` is representable; `StagedInputSource::Occurrence`/`Intermediate` spell both source lists today (`law.rs`); dense and mapped reads of one declaration stay separate accesses because each read carries its own `LogicalAccess` (`physical.rs`, anchors `LogicalAccess::LinearIdentity` and `BroadcastReplication`), so no general declared-input deduplication follows.
5. **Verified after two in-place repairs (recorded in the Fact).** The RMS law row registers at revision 1 (`crates/tiler-ir/src/semantic/registry.rs`, anchor `register_index_realization_law(operation, 1, law)`; the registration API takes a per-operation revision, so a lone RMS step is expressible). The RMS lowering capability registers at the shared constant `GOVERNED_CAPABILITY_REVISION` = 1 (`governed.rs`), so moving only RMS to 2 requires breaking that constant out per capability — the carrier ticket already names this ("do not raise unrelated law or capability rows merely because their current implementation shares a constant"). The governed physical provider registers at `GOVERNED_PHYSICAL_REVISION` = 1 (`crates/tiler-compiler/src/frontier.rs`). Both registry identities fold into every governed request subject (`request.rs`, anchor `authorities.installed.registry_identity()` and the `realization_registry` field beside it).
6. **Verified.** With only the physical guard deleted, `resolve_lowering` still refuses at refinement before any cover, frontier, or physical derivation runs, and the law and the governed provider each independently refuse the one-boundary subject; nothing is admitted.

**Identity derivation, reproduced independently.** The law row encodes `(operation, provider, revision, law)` (`registry.rs`, `encode_index_realization_law_row_for`) and is folded into every RMS occurrence's subject identity (`refinement.rs`, anchor `encode_optional_law_row`). So the law-revision move alone moves every RMS subject — distinct-operand occurrences included — and everything that folds it: refinement resolution and receipt, occurrence identity (which separately folds the capability revision through `encode_occurrence_identity` in `legality.rs`), kernel-program identity (v9 folded per-occurrence refinement evidence), artifact identity, and cache subjects. The physical-provider revision is one `ProviderIdentity` for every governed proposal, so selected-provider provenance moves for every governed plan, not only RMS. What genuinely stays byte-identical for distinct operands is the schedule-region and kernel canonical structure, which carries no semantic or authority correlation (ADR 0070; `schedule/handles.rs`, anchor `no semantic correlation at all`). The prior final evidence bullet asserted blanket preservation of "every existing distinct-operand RMS structural identity" modulo only provider provenance, which this derivation shows is unsatisfiable as written; it was rewritten to name the preserved and the enumerated-moving populations exactly. The three moves are complete and minimal: no schedule-, kernel-, program-, artifact-, or conformance-layer authority is law- or family-versioned separately (the `SquaredSerialSumThenEpilogue` vocabulary and all verification paths are generic), and neither recognition nor refinement verification changes behavior under admission (the aliased-boundary machinery already handles one boundary claimed by two uses).

**Frontier confirmation.** The four eliminations hold at this base with the sources above; the one-buffer-per-read invariant (`crates/tiler-ir/src/kernel/lower.rs`, anchor `one buffer per read`) grounds the KIR/artifact-only-coalescing elimination, and the accepted AccessOrdinal decision's own control ("Repeated local positions remain distinct even when they project to the same declared ordinal") confirms that decision preserves authored positions without requiring one access per semantic operand use. One materially distinct admission spelling was missing and has been added as eliminated: a compiler-materialized second operand copy. One refusal variant was considered and deliberately not added to the option set: refusing `rms_norm(x, x)` earlier (at semantic construction or recognition) with a sharper diagnostic. It is not an admission spelling, and it would supersede the accepted semantic contract — whose rules deliberately never read handle identity (Fact 1) — to remove a legal, reference-defined program permanently; it differs from the retain option only in diagnostic placement, which the retain option already concedes. The frontier therefore remains exactly two survivors, and the recommendation's dominance argument among admission spellings stands.

**Perturbation reachability.** Each Required-evidence check was tested for a reachable "no". The four-operand-binding and three-pass-buffer pins genuinely distinguish the coalesced spelling: a two-access implementation yields six operand bindings (three stage-input claims times two uses) and four pass accesses, so the pin fails against the wrong spelling. The weight-distinct perturbation fails if coalescing over-generalizes, because the distinct subject would lose its third read. The different-relation perturbation fails against a dedup rule keyed on declared input rather than exact value plus identical dense relation; the two-mapped-reads epilogue population from the AccessOrdinal controls makes it constructible today. The scalar second-use perturbation yields bitwise disagreement because rewiring or dropping the second multiply changes a quadratic term to a linear one. The three revision moves are independently observable in the distinct identity owners named above, with the capability move additionally requiring the per-capability constant split. The end-to-end alias build and reference/oracle comparison are constructible now — the semantic, recognition, and reference layers already accept the subject, as the disposable probe and the existing aliased-operand tests show.

**Verdict: ready for Tom with the named repairs, which are made in place.** Discrepancies found, by severity: (1) moderate — the final Required-evidence bullet's blanket identity-preservation claim was unsatisfiable given the law-row folding; rewritten. (2) minor — Fact 5's `kernel-program-v11` citation drifted to v12 after the audit base; repaired with provenance. (3) minor — Fact 5's provenance sentence understated the physical-provider revision's blast radius (every governed plan's selected provenance, not only RMS); widened. (4) minor, addition — one eliminated admission spelling (compiler-materialized copy) was missing; added. None of the repairs changes the decision question, the frontier, or the recommendation.
