---
id: decide-the-algebraic-capability-authority-for-contraction-splits
title: Decide the algebraic capability authority for contraction splits
status: blocked
priority: p1
dependencies: [decide-the-semantic-order-contract-for-relaxed-contractions]
related: []
scopes: [implementation/ir, implementation/compiler, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, numerics, identity, public-boundary, needs-tom]
---
## User-visible outcome

Tiler keeps both fixed contraction splits unavailable for the current `tiler::strict-tensor-contraction-f32@1` definition. No algebraic capability can make either split legal while that same definition and its reference contract affirmatively require one strict value and forbid reassociation and permutation.

This is the source-derived fail-closed result, not a request for Tom to choose a capability API. The separate semantic order-contract and result-set identity question is now [`decide-the-semantic-order-contract-for-relaxed-contractions`](decide-the-semantic-order-contract-for-relaxed-contractions.md). This ticket remains `blocked` on that prerequisite and stays out of `.ticketsplease/decision-queue.md`; if that prerequisite accepts a relaxed semantic population, this ticket reopens to decide the exact combiner authority for that population.

## Source-first Fact audit — exact base `fe60f992cc20b37a52aff815897170516490667a`

**Verified — permission is only half the accepted legality rule.** [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md), anchor `Each transformation requires two independent facts`, and [Numerical semantics](../docs/numerical-semantics.md), anchor `Reassociation requires both an operation capability`, require an applicable operation-declared algebraic capability and an independently resolved numerical permission. Reassociation needs the first for ordered regrouping; permutation additionally needs a commutativity capability.

**Imprecise and repaired — the standard contraction deliberately declares no algebraic capability, and its own semantic record affirmatively forbids both transformations.** `OperationDefinition::new` in `crates/tiler-ir/src/semantic/operation.rs`, anchor `algebraic_capabilities: OperationAlgebraicCapabilities::none()`, supplies the default; `register_standard_contraction` deliberately does not override it, and `the_contraction_declares_no_algebraic_capability` pins that absence. The same registration's normative definition fixes `binary32 products folded strictly in ascending lexicographic order`, `contraction_f32_facts` sets both `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` and `CONTRACTION_F32_FACT_PERMUTATION_PERMITTED` to `false`, and the source explains that declaring ordered associativity would hand a rewrite facts the family forbids. The current public capability record has only `ordered_associativity`; no commutativity vocabulary exists. Its declaration speaks for the operation's operands and every admitted signature, while the split would regroup or reorder the contraction's internal sequence of `left * right` products under F32 addition. The only compiler consumers of `algebraic_capabilities()` are the add/multiply logical-normalization rules in `OrderedReassociationRule::evaluate` and `OrderedReassociationRule::propose`, not the contraction physical-proposal path. A packet must therefore treat enabling the current operation-wide flag on this key as a semantic contradiction, not as the default capability-owned option.

**Imprecise reproduction and repaired — exact implementation attempt `648a372f8cbb306df43a4edfc4e14a6211cac7b1` over `07aca5cd8f67824019d8c183fd3a9584ce84b670` exposed the gap and was not merged.** Its `contraction_split_region` checks only request permissions. Its positive `contraction_membership_permission_is_decided_before_construction` therefore admits a contiguous split for a real standard contraction even though that operation declares no ordered-reassociation capability; the synthetic lane-strided path additionally consumes a commutativity capability nothing can state. Independent review classified both as a P1 semantic-authority blocker in [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md), anchor `Independent review stop — 2026-08-17`. The compiler test exists only at the preserved attempt, so the former unqualified test command would select zero matching tests at this base. Reproduce the current and historical halves separately with:

```sh
cargo test -p tiler-ir the_contraction_declares_no_algebraic_capability -- --nocapture
rg -n 'algebraic_capabilities\(\)|declares_ordered_associativity' crates/tiler-compiler/src
git show 648a372f8cbb306df43a4edfc4e14a6211cac7b1:crates/tiler-compiler/src/physical.rs | rg -n -A24 'fn contraction_split_region'
git show 648a372f8cbb306df43a4edfc4e14a6211cac7b1:crates/tiler-compiler/src/frontier.rs | rg -n -A70 'fn contraction_membership_permission_is_decided_before_construction'
```

**Verified — the physical carrier is accepted but not landed, and does not settle semantic authority.** [`decide-the-fixed-strided-contributor-membership-vocabulary`](decide-the-fixed-strided-contributor-membership-vocabulary.md) accepts `ContributorMembership::{Contiguous, LaneStrided}` and `ReductionTopology::CooperativeContractionSplit`; current Rust defines neither. The decision says which permissions each topology consumes. It does not supersede ADR 0014 or decide which operation/combiner owns the prerequisite capabilities.

## Complete source census

The following are source reads, not grep-only inferences. Searchable anchors name the load-bearing statement in each file.

- **Accepted meaning and authority:** [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md), `Each transformation requires two independent facts`; [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md), `unless a registered permission authorizes otherwise`; and [Numerical semantics](../docs/numerical-semantics.md), `Reassociation requires both an operation capability` and `ordered associativity is declared by the operation and authorized by the contract`.
- **Semantic construction, registration, validation, and identity:** `crates/tiler-ir/src/semantic/operation.rs`, `pub struct OperationAlgebraicCapabilities`; `crates/tiler-ir/src/semantic/registry.rs`, `fn encode_operation_definition`; and `crates/tiler-ir/src/semantic/contraction.rs`, `register_standard_contraction` and `fn contraction_f32_facts`. The complete contraction definition, inferencer, structure validator, fact record, registration, and tests were read, including `the_contraction_declares_no_algebraic_capability`.
- **Scalar construction and registry:** `crates/tiler-ir/src/index/scalar.rs`, `pub struct ScalarOperationDefinition`, `fn encode_definition`, `tiler.scalar-definition-projection.v2`, and `tiler.scalar-registry-snapshot.v1`; `crates/tiler-ir/src/index/law.rs`, `Standard add-f32 law`; and `crates/tiler-ir/src/index/refinement.rs`, `pub struct IndexRefinementReceipt`. The registered scalar add is exact empty-attribute `(F32, F32) -> F32`, but `ScalarOperationDefinition` carries no algebraic capability field.
- **Verified reducer construction and inspection:** `crates/tiler-ir/src/index/model.rs`, `pub enum ReductionTraversal` and the `ScalarReductionRef`, `ScalarReducerBodyRef`, `ReducerBodyOperationRef`, and reducer-value views; `crates/tiler-compiler/src/legality.rs`, `pub struct IndexRefinement`; and `crates/tiler-compiler/src/lowering.rs`, `pub(crate) struct ResolvedLowering`. The standard contraction realization forms each product separately and, above one contributor, folds them with `tiler.scalar::add-f32@1` through an exact lexicographic-left-fold reducer. That is an implementation fact retained by refinement evidence, not a semantic permission.
- **Numerical permissions and normalization:** `crates/tiler-compiler/src/policy.rs`, `const TENSOR_CONTRACTION`; `crates/tiler-compiler/src/normalize.rs`, `struct OrderedReassociationRule`; and `crates/tiler-compiler/src/fusion_legality.rs`, `would answer false`. The policy table says the contraction can *consume* the reassociation and permutation dimensions; it does not say either is algebraically or semantically available. The only production consumers of `algebraic_capabilities()` are the two checks in `OrderedReassociationRule::evaluate` and `OrderedReassociationRule::propose` for logical add/multiply chains.
- **Physical proposal and checked verification:** `crates/tiler-compiler/src/frontier.rs`, `pub struct ImplementationContext`, `pub enum StrategyDeclineCause`, `pub struct FrontierRegionSubject`, and `pub(crate) fn enumerate_frontier`; `crates/tiler-compiler/src/physical.rs`, `pub(crate) fn verify_schedule_with_feasibility`; and `crates/tiler-compiler/src/pipeline/planning.rs`, `let subject = FrontierRegionSubject::reading_intermediates`. The planning transaction retains `ResolvedLowering` while it constructs every production frontier subject, so a later accepted design could derive an exact reducer proof without trusting a provider. No current verifier joins such a proof to a split because the accepted split carrier is not landed.
- **Reference and conformance:** `crates/tiler-reference/src/contraction.rs`, `The three permissions`; `crates/tiler-reference/src/conformance.rs`, `ReassociationPermitted`; `crates/tiler-reference/src/oracle.rs`, `enum StandardScalarBinaryF32`; and `crates/tiler-reference/src/evaluate.rs`, `pub fn cooperative_grouped_sum`. The strict contraction decoder requires all three permission facts to be `false`; either order fact becoming `true` is a declaration this reference does not compute. `ReferenceNumericalConformance::from_realization` likewise refuses a permitted reassociation or permutation realization. Existing cooperative/result-set helpers are bounded evidence and do not silently replace that reference contract.
- **Identity, request, explain, artifact, and cache propagation:** `crates/tiler-ir/src/domains.rs`, `tiler.scalar-definition-projection.v2`; `crates/tiler-reference/src/identity.rs`, `CanonicalReferenceRegistryIdentity`; `crates/tiler-reference/src/oracle.rs`, `CanonicalScalarReferenceRegistryIdentity`; `crates/tiler-compiler/src/request.rs`, `const REQUEST_SCHEMA_VERSION`; `crates/tiler-compiler/src/pipeline/trace.rs`, `pub(super) fn record_frontier`; `crates/tiler-ir/src/index/refinement.rs`, `pub struct IndexRefinementExecutableCoverageIdentity`; `crates/tiler-ir/src/program/model.rs`, `pub struct CoveredOccurrence`; `crates/tiler-artifact/src/program/model.rs`, `covered.refinement().as_bytes()`; and the cache/artifact identity consumers linked from [Artifact ABI](../docs/artifact-abi.md), anchor `projection and never its complete identity`. The semantic reference identity encodes `FrozenSemanticRegistry::snapshot_identity`; the scalar reference identity separately encodes `FrozenScalarRegistry::snapshot_identity`. A capability added to either definition is therefore identity-bearing even if no request field is added. The fail-closed result here adds no field and therefore moves none of these identities.

The exhaustive current Rust census is reproducible with:

```sh
rg -n 'OperationAlgebraicCapabilities|algebraic_capabilities\(\)|declares_ordered_associativity|with_ordered_associativity' crates --glob '*.rs'
rg -n 'StrategyDeclineCause::|FrontierRejection::|PhysicalError::' crates/tiler-compiler/src --glob '*.rs'
rg -n 'CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED|CONTRACTION_F32_FACT_PERMUTATION_PERMITTED' crates docs --glob '*.rs' --glob '*.md'
```

## Decision result — no current authorizing capability

**Fact — the current operation defines one value, not a relaxed result set.** Its normative reference says `binary32 products folded strictly in ascending lexicographic order`; its two order facts are `false`; its registration deliberately withholds algebraic capability because declaring it would hand a rewrite facts the family forbids; and the independent reference decoder rejects either fact changed to `true`. A caller's numerical contract is a ceiling. It cannot turn an operation-owned `false` into authority.

**Fact — the existing `tiler::add-f32@1` declaration is real but has a different subject.** That semantic tensor operation declares ordered associativity so `OrderedReassociationRule` may consider regrouping a same-operation logical add tree, still behind independent reassociation permission. The contraction node is not such a tree: its graph operands are the two input tensors, while the affected sequence is the internal `left * right` contributor fold. Applying the contraction node's operation-wide flag would speak about the wrong operands; applying the semantic add flag directly has no typed declaration connecting that operation to the contraction's internal fold.

**Fact — the scalar realization does not widen semantic meaning.** Refinement proves that the current lowering happens to realize the strict fold with `tiler.scalar::add-f32@1`. That scalar definition has no algebraic capability today. Even if it gained one, a lowering implementation fact cannot override the semantic definition it was proved to refine. It would be necessary evidence after a semantic order transition, but is insufficient before one.

**Conclusion.** Neither `ContributorMembership::Contiguous` nor `ContributorMembership::LaneStrided` is legal for the current key under any statable numerical contract. Contiguous would regroup the declared strict fold; lane-strided would regroup and permute it. The safe and maintainable result is unchanged typed refusal. No Rust capability vocabulary, registry definition, permission, schedule, request, reference, explain schema, artifact identity, or cache identity changes in this ticket.

## Independent algebraic derivation retained for the reopening trigger

This derivation establishes that a future semantic transition is not blocked on whether the *standard F32 add combiner* is commutative. It does not authorize that transition.

The exact candidate population is `tiler.scalar::add-f32@1` at empty attributes and signature `(tiler::f32@1, tiler::f32@1) -> tiler::f32@1`, over all `2^32 × 2^32` ordered operand-bit pairs and each scalar-evaluable F32 subnormal realization. For each operand the realization applies the same unary input-subnormal map; arithmetic is binary32 round-to-nearest, ties-to-even; every arithmetic NaN is replaced with `0x7fc00000`; and the same unary result-subnormal map follows. Exceptional-value assumptions only remove operands from this population, and signed-zero elimination only coarsens observable equality; neither can create a counterexample to the strict bitwise case below. The proof does not pretend that `ReferenceNumericalConformance` evaluates a reassociation-, permutation-, or signed-zero-permitted result set—it explicitly refuses those contracts.

- **Finite values, overflow, and underflow:** the exact real sum is symmetric in the operands, the fixed rounding function is identical in both directions, and a common result-subnormal map preserves equality.
- **NaNs:** swapping quiet/signalling NaNs may change which hardware payload, sign, or quieted operand an underlying instruction selects, but both directions are NaN. Canonicalization erases that order-sensitive selection and produces `0x7fc00000` in both directions. This includes one-NaN and two-NaN pairs and does not assume payload preservation.
- **Signed zero:** like-signed zero pairs are identical. Under round-to-nearest, `+0 + -0` and `-0 + +0` have the same result sign. Input FTZ is the same unary map on each operand, including either admitted sign-preserving or always-positive zero realization, so swapping operands preserves the two preprocessed values as a multiset; result FTZ does not alter a zero.
- **Infinities:** equal-sign infinities produce that infinity in either order; a finite value and one infinity produce the same infinity; `+infinity + -infinity` and its reverse both produce NaN and therefore the same canonical payload.
- **Subnormals and FTZ:** preserve mode is covered by the finite proof. In each flush mode the symmetric per-operand preprocessing occurs before the same addition, and the common result preprocessing occurs after it. No direction-specific state exists.

Therefore scalar F32 add is bitwise commutative in the strict observable domain for every admitted subnormal mode; contracts that assume exceptional values absent narrow that domain, and signed-zero elimination weakens rather than strengthens the equality required. Ordered associativity is deliberately a different statement: floating-point addition is not bitwise associative, so its declaration only makes an ordered regrouping *eligible* when the independent numerical permission/result-set contract admits it. Neither law authorizes changing a product leaf, swapping the two operands of a multiplication, distributing multiplication over addition, fusing multiply-add, changing accumulator dtype, inventing padding, or accepting nondeterministic arrival.

## Pareto-complete option analysis

1. **Keep the current key fail-closed — sole current survivor.** Correctness and fail-closed strictness are maximal; it adds no public surface, migration, runtime work, or memory. Its cost is that both measured split candidates remain unreachable. Reversal evidence is an accepted, identity-bearing semantic order contract that changes the current `false` declarations into a typed conditional result set and installs a reference/conformance oracle for it.
2. **Declare ordered associativity on the contraction operation itself — eliminated.** It contradicts the registration comment and both order facts, and `OperationAlgebraicCapabilities` promises the law for the operation's admitted signatures/operands rather than for an unnamed internal reducer. It still cannot state commutativity. The strongest advantage is a tiny existing API; that API names the wrong algebraic object.
3. **Extend the operation-wide record with commutativity and declare both on the contraction — eliminated.** Besides the same contradiction, an accessor read as authority to swap the contraction's two tensor operands would silently admit a different program. Independent permutation permission would not repair a subject mismatch.
4. **Add ordered/commutative capabilities to `tiler.scalar::add-f32@1` and derive the actual reducer from refinement — eliminated as a complete current answer, retained as the leading post-transition seam.** It names the right arithmetic object and the proof above supports both laws at the exact signature. It remains only an implementation fact until the semantic operation declares that its strict value may become that result set. The strongest counterargument after a transition is cross-layer coupling and identity churn; reversal evidence is a pre-lowering transform or a second admitted contraction realization whose semantic combiner cannot be derived from the retained reducer.
5. **Add an explicit typed semantic reducer/combiner descriptor — not smuggled into this ticket.** It could make the operation's internal algebraic object explicit before lowering, but it also decides the semantic result set, validation, reference meaning, registry closure, and public identity. Those are exactly the separate prerequisite's purpose. A descriptor that merely duplicates the realized reducer is dominated unless a pre-lowering consumer or multiple equivalent lowerings need it.
6. **Admit only contiguous regrouping — eliminated at the current key.** Ordered-only evidence would be enough algebraically after a semantic transition, but today it still violates the operation's `reassociation-permitted: false`. It does not become sound merely by avoiding permutation.
7. **Add a relaxed contraction key — eliminated under current accepted authority.** It duplicates frontend, registry, reference, lowering, and identity verticals and contradicts ADR 0087's accepted one-key family unless that ADR is explicitly superseded. A new key becomes a frontier candidate only if the prerequisite finds the one-key conditional-result-set design unsound.
8. **Infer authority from permission, fact strings, normative prose, key spelling, or the observed scalar body — eliminated.** Each route defaults or reconstructs a fact no typed owner declared and can silently accept after an unrelated wording/lowering change.
9. **Further bounded research and deferral.** Research is useful only for the separate semantic order/result-set choice; it cannot make the current false declarations mean true. Deferral therefore has the same executable result as option 1 and is not a second frontier candidate.

The frontier has one member. There is no Tom trade-off to queue from this ticket.

## Refusal and future-consumer contract

At this base no contraction-split proposal exists, so the fail-closed state allocates no misleading numerical refusal. The dependency remains blocked before proposal construction. If the semantic prerequisite later accepts a relaxed population, the reopened authority decision must make the verifier/compiler order mechanically unique:

1. match the exact semantic operation, internal combiner role, scalar key, empty attributes, `(F32, F32) -> F32` signature, and reached registry authority;
2. require declared ordered associativity for either membership;
3. additionally require declared commutativity for `LaneStrided`;
4. independently require resolved reassociation permission for either membership;
5. independently require resolved permutation permission for `LaneStrided`; and
6. only then check shape/partition and construct or admit a schedule.

Missing declaration is `Unknown`, not false and not a numerical refusal. A later implementation must give algebraic-capability absence and numerical-permission refusal distinct typed causes and stable reason keys. It must also recheck the same facts on provider output before frontier admission; provider-side proposal filtering alone is bypassable.

No future consumer may treat `declares_commutativity()` as standalone operand-swap authority. Each call site must first bind the exact declared algebraic subject and then require the independently resolved permutation permission. The current consumer census has no commutativity accessor and only the two ordered-reassociation checks in `normalize.rs`; a future change must rerun the Rust census above and account for every new accessor/encoder/match. This rule is what prevents adding scalar-add commutativity from silently authorizing a non-contraction tensor operand swap.

## Identity, schema, host-runtime, and memory result

**Current result:** no production or contract file changes, no canonical tag, no field, no domain step, no provider revision, and no pin rebaseline. Semantic/scalar definition projections and snapshots, `CanonicalReferenceRegistryIdentity`, `CanonicalScalarReferenceRegistryIdentity`, realization-law and lowering registries, request schema and qualifier, explain renderer, refinement receipt and reached executable coverage, kernel-program/artifact/cache identities, and old bytes all remain exact.

**Deferred lower bound:** if the prerequisite later chooses a scalar-definition capability, the implementation must treat it as definition content. At minimum it must decide a fresh commutativity law tag with count framing and old-byte invariants; add the capability to `ScalarOperationDefinition` with typed construction/access; step `tiler.scalar-definition-projection.v2` and `tiler.scalar-registry-snapshot.v1` because every definition's field layout changes; update `crates/tiler-ir/src/domains.rs`; and recompute every nested request, refinement, program, artifact, explain, and cache pin. A same-key semantic-definition change moves the semantic snapshot embedded in `CanonicalReferenceRegistryIdentity`, so that outer value and its pins move even if `tiler.reference-registry.v2` and the semantic-reference provider revisions do not. A same-key scalar-definition change likewise moves the scalar snapshot embedded in `CanonicalScalarReferenceRegistryIdentity`, so that outer value and its pins move even if `tiler.scalar-reference-registry.v1` and the scalar-reference provider revisions do not. Keeping the standard scalar provider revision at `1` is only sound if the new behaviour is wholly carried by provider-independent definition bytes, following the current `StandardSemantics::identity` rule; the prerequisite/reopened ticket must rederive rather than inherit that conclusion.

The unchanged refusal has zero target/runtime allocations, dispatches, device memory, or kernel work. A future proof-derived reducer query would be a bounded host scan of one already-retained verified reducer plus one registry lookup and a constant-size authority record per eligible subject; that estimate is not an implementation authorization or performance claim.

## Unsupported population and negative controls

Everything is unsupported today because the semantic key forbids both freedoms. After any accepted transition, fail closed on a non-F32 or nonbinary combiner; nonempty attributes; a scalar key other than the explicitly declared add; multiple state, contributor, result, operation, or yield values; a non-left-fold or staged realization not covered by the accepted semantic descriptor; absent reached-registry authority; another semantic family; BF16; an extension definition without explicit laws; or any attempted inference from strings/facts.

Required subject perturbations for the later transition are retained here so a green assertion cannot substitute for exercising its subject:

- change either contraction order fact to `true` today and show the reference decoder's field-specific refusal;
- add an operation-wide contraction capability and show `the_contraction_declares_no_algebraic_capability` fail;
- remove ordered associativity from the exact combiner and show contiguous refuses under the algebraic cause even when reassociation permission is permitted;
- remove only commutativity and show contiguous remains eligible while lane-strided refuses algebraically;
- forbid reassociation with capabilities present and show a distinct numerical refusal; forbid only permutation and show the same contiguous/lane split;
- perturb membership while keeping partition counts fixed and show schedule/witness/explain identity differ;
- perturb scalar key, signature, attributes, traversal, body operand order, or reached authority one at a time and show the combiner derivation refuses;
- toggle each future capability bit independently and show its definition projection/snapshot and nested identity move; preserve the current `none` and ordered-only bytes where the accepted encoding promises that invariant.

## Release trigger and graph repair

[`decide-the-semantic-order-contract-for-relaxed-contractions`](decide-the-semantic-order-contract-for-relaxed-contractions.md) must first accept one complete answer for the existing strict key's two false order facts, point value versus result-set reference semantics, effective permission intersection, key/definition identity, schema, and unsupported population. If it keeps the key strict, this ticket closes as typed refusal and the split implementation is closed or superseded. If it admits a relaxed population, this ticket reopens at that accepted commit and selects the exact combiner capability and verifier join without revisiting the semantic question.

[`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) continues to depend directly on this algebraic-authority record and inherits the semantic-order prerequisite through it. Its preserved attempt remains evidence only. No queue row, production implementation, permission grant, or mutation of the preserved attempt is part of this result.

## Consequences and non-goals

This ticket does not implement either split, change the already accepted physical carrier, grant any numerical permission, admit distributivity/FMA/atomics/nondeterministic arrival, claim device performance, or repair the implementation attempt's separate witness/explanation coverage gap. The dependent implementation ticket retains that repair explicitly.

## Closes when

Tom has accepted an exact public/identity-bearing algebraic authority and the complete operation/dtype/signature matrix needed by both fixed contraction memberships, or has accepted a narrower fail-closed outcome with the excluded membership and reopening trigger stated. The accepted result must leave a mechanically unique verifier/compiler implementation that checks capability and permission independently.
