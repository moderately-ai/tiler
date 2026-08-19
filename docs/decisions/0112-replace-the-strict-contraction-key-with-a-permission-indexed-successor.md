---
schema: "tiler-doc/v1"
id: "ADR-0112"
kind: "decision"
title: "Replace the strict contraction key with a permission-indexed successor"
topics: ["numerics", "identity", "ir", "reference", "public-boundary"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.numerical-semantics", "tiler.contract.ir", "tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.scheduling.first-metal-contraction-realizations"]
depends_on: ["ADR-0012", "ADR-0013", "ADR-0014", "ADR-0034", "ADR-0072", "ADR-0087", "ADR-0095"]
ticket: "replace-the-standard-contraction-key-with-the-accepted-successor"
---

# 0112: Replace the strict contraction key with a permission-indexed successor

**Status:** accepted by Tom on 2026-08-18 in the live coordination session with the orchestrator, relayed first-hand by the coordinator, as three chained acceptances recorded in [`decide-the-semantic-order-contract-for-relaxed-contractions`](../../tickets/decide-the-semantic-order-contract-for-relaxed-contractions.md) (packet audited at `368dcd25`, independently reviewed at `5a48c9ce`), [`accept-the-tensor-contraction-successor-public-surface`](../../tickets/accept-the-tensor-contraction-successor-public-surface.md), and [`decide-the-algebraic-capability-authority-for-contraction-splits`](../../tickets/decide-the-algebraic-capability-authority-for-contraction-splits.md) (reopened packet `c85ef0b2`, reviewed `32e5ecbc`). Implemented on 2026-08-19 by [`replace-the-standard-contraction-key-with-the-accepted-successor`](../../tickets/replace-the-standard-contraction-key-with-the-accepted-successor.md), which records the rederived identity cascade, the recomputed pins, and the demonstrated perturbations.

## Context

The standard tensor-contraction key was `tiler::strict-tensor-contraction-f32@1`: one strict ascending-lexicographic left fold, with reassociation and permutation declared `false` as two Boolean fact fields. The accepted contiguous-split physical carrier and its fired schedule demand needed an ordered regrouping the strict definition affirmatively forbade, and the decision packet's measured artifact-hybrid control proved a same-key semantic mutation unsafe under ADR 0072's graph contract: `ArtifactProgramBuilder::push_variant` correctly joins by `semantic_graph_identity()` alone, so a definition-only change leaves a concrete old/new hybrid join possible. A meaning change therefore required a new operation key, not a fact edit.

## Decision

**Retire `tiler::strict-tensor-contraction-f32@1` from the standard vertical completely and replace it with `tiler::tensor-contraction-f32@1`, reassociation-only.**

- **Result cells.** Under a request withholding reassociation, the successor's result is the retired key's strict left fold bit for bit. Under a request permitting reassociation, it denotes the set of results of all full ordered binary trees whose in-order leaf traversal is exactly the unchanged canonical contributor sequence. Permutation, signed-zero elimination, arithmetic contraction (FMA), and distributivity (ADR 0095) remain operation-owned unsupported: no ceiling can grant them.
- **No coexistence.** The retired key has no alias, equivalence rule, fallback, or duplicate selection policy anywhere in the standard semantic, reference, law, lowering, compiler-recognition, or frontend vertical. Generic historical bytes remain decodable, but an installed standard compiler or reference has no authority to compile or execute the retired operation. ADR 0087's one-family rationale is preserved — a frontend still emits one contraction key — and ADR 0034/0072 immutable-meaning discipline binds the successor: a later incompatible change takes another key generation.
- **Typed definition descriptor.** The successor's fact record has exactly thirteen fields: the retired Boolean order fields 8 and 9 are never reused; field 14 carries a seven-row canonical record binding the ADR 0013 plan-determinism scope into provider-independent definition bytes; and field 15 carries a six-row reduction record naming the leaf primitive, the reducer primitive, the result-class rule, and the three order-freedom maxima (`reassociation: permission-gated`, `permutation: unsupported`, `signed-zero elimination: unsupported`). `ContractionF32ReductionDescriptor::decode` in `tiler-ir` is the sole decoder; a second compiler or reference decoder is forbidden, and the semantic registrar refuses to register a governed contraction definition the decoder does not validate (`RegistryError::InvalidGovernedContractionDescriptor`).
- **Algebraic authority.** The descriptor's order-freedom maxima are ADR 0014's operation-declared algebraic fact for the contraction's internal F32 reducer. `OperationAlgebraicCapabilities` stays `none()` on the successor — the operand-level record speaks for the operation's admitted signatures, and regrouping a contraction *chain* consumes distributivity, which ADR 0095 declines. The independently resolved numerical ceiling is the second fact, and `ContractionF32ReductionDescriptor::resolve` into `EffectiveContractionF32Profile` is the only join: a request may withhold a supported freedom and cannot grant an unsupported one. The compiler's `StrategyDeclineCause` gains one appended variant (`AlgebraicCapabilityUnsupported`, tag `0x06`, reason `algebraic-capability-unsupported`) so the algebraic and numerical refusal sources are never collapsed into one verdict; lane-strided contraction membership stays refused under it until a future key generation declares fold permutation. ADR 0014 needs no supersession; its implementation boundary carries a dated status note.
- **Witness and reference boundary.** Set membership is not checkable — a body claiming tree A must be checked against A even when its wrong result equals a value tree B can produce — so a plan owns one validated `OrderedContractionF32Tree`, bound to its exact semantic graph, kernel program, and occurrence by `ContractionF32PlanWitness` (static-`K` only; the live-extent contraction refuses as `LiveContributorCount`). The ordinary registered reference stays strict-only; the bounded `ContractionF32TopologyEvaluator`, owned by `standard-reference` (provider and contraction capability revision both 7 → 8), is the only relaxed reference route, always requires a witness, takes a caller-owned four-resource budget the crate never defaults, and never falls back to strict evaluation.

## Identity consequences

Every contraction occurrence's `OpKey` moves, so `tiler.semantic-graph.v3` content moves — which is exactly what closes the artifact hybrid: crossed old/successor programs fail `push_variant`'s graph-identity join as `SemanticSubjectMismatch`. Reached definitions, the semantic registry snapshot, the complete `SemanticIdentity` (except the byte-identical `ShapeEnvIdentity`), the law and lowering registries, request subjects and the pinned explain qualifier, refinement receipts, kernel programs, artifacts, and caches all move as content under unchanged domain grammars and schema versions; scalar subjects stay byte-identical. The implementing ticket records the complete rederived cascade and the recomputed pins.

## Consequences

- The demanded contiguous-split capability becomes semantically reachable: the reassociation-permitted cell is a defined result set with a witnessable member, and admission work proceeds under [`revise-contraction-split-admission-to-contiguous-only-delivery`](../../tickets/revise-contraction-split-admission-to-contiguous-only-delivery.md). Lane-strided admission stays behind the permutation/commutativity trigger.
- Plan determinism binds to ADR 0013 exactly, in definition bytes: same input bits and runtime bindings, same artifact digest and selected plan variant, and same declared target environment give identical output bits; a different artifact may select a different legal tree.
- The public surface pays a typed descriptor/profile/witness/evaluator vocabulary; nothing silently returns a strict value for a result-set request, and every excluded population — permutation, FMA, distributivity, signed-zero elimination, exceptional-value absence assumptions, live-`K` topology witnesses, padding, coordinate-dependent trees — stays typed unsupported.

## Alternatives considered

The packet's Pareto frontier retained two survivors: keeping the strict key forever (maximal narrowness, strands the fired capability) and this complete replacement. Same-key mutation was eliminated on the measured artifact-hybrid control; coexisting keys on ADR 0087's one-family result; `strict-...@2` because its name would be false; `tensor-contraction-f32@2` because generation 2 of a new name invents an absent generation 1; boolean-flip and permutation-bearing variants on correctness and dominance grounds. The full analysis, eliminations, and reversal evidence live in the accepted packet.
