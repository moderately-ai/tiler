---
schema: "tiler-doc/v1"
id: "ADR-0107"
kind: "decision"
title: "Admit an indirect gather as a semantic family above the index language"
topics: ["indexing", "semantics", "operation-families", "gather", "ir"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.transformer-operation-and-shape-surface"]
depends_on: ["ADR-0046", "ADR-0075", "ADR-0087"]
ticket: "admit-an-indirect-gather-family-for-tied-embedding-lookup"
---

# 0107: Admit an indirect gather as a semantic family above the index language

**Status:** accepted by Tom on 2026-08-07, in the interactive orchestration session, as a direct answer to the decision presented with its trade-off and counterpoint. Not relayed.

**What was accepted, stated so it is not over-read.** The family is admitted as a semantic operation and as nothing below it, and the acceptance covers precisely that: a **registered, reference-evaluated, unplannable** family is a legitimate delivered state rather than a half-landing to be finished. The index-expression vocabulary stays unchanged, which is the record's substance and not a deferral inside it.

**What acceptance did not commit to.** Not the public boundary: under [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) the key, the gathered-axis attribute, `GatherAxis`, `GatherError`, and `gather_result_shape` remain a **labelled draft** until their exact included and excluded surface is separately accepted. Not the index-layer question, which [`admit-the-indirect-access-class-into-the-index-layer`](../../tickets/admit-the-indirect-access-class-into-the-index-layer.md) held as a decision in its own right and which [ADR 0108](0108-site-a-data-dependent-index-coordinate-on-the-expression.md) now answers — in shape, deciding that such a coordinate would be an expression form rather than a field on the access, and in timing, admitting neither yet. That record extends this one and changes nothing decided here.

**The counterpoint accepted alongside it.** A registered family that no program can plan is a trap for a reader who reasonably takes registration to imply reachability — records are read less often than registries. That risk was stated and accepted; the mitigation is that the fail-closed boundary is tested rather than asserted, with `classify` returning `None` so no region derives legality, and six perturbations each firing their own gate.

**Dated correction, 2026-08-08 — the mitigation above overstates its evidence, and the decision is unaffected.** The sentence is retained verbatim because it is what was put to Tom and accepted; this paragraph records what a re-read found rather than editing the claim under him. `grep -rn 'gather_f32_op' crates/tiler-compiler/` returns nothing, and `crates/tiler-compiler/tests/` mentions gather zero times, so **no test names this key against `classify` or against the request boundary**. The six perturbations are real and each does fire a gate, but five of those gates are in `tiler-ir` and `tiler-reference`; the sixth is `every_unplanned_operation_is_registered_and_consumes_no_dimension` in `crates/tiler-compiler/src/policy.rs`, which asserts over `operation_capability` — a different authority from `classify`, as that file's own prose says of the BF16 rows — and which builds no program. What is genuinely tested is that the family is registered, carries no realization law, refuses out-of-range and signed indices, and holds no capability row. What is asserted and not tested is the compile-time refusal itself. [`pin-the-gather-request-boundary-refusal-with-a-test`](../../tickets/pin-the-gather-request-boundary-refusal-with-a-test.md) owns the repair. The counterpoint's *acceptance* stands: the risk was named and taken, and the correction narrows the mitigation rather than withdrawing it.

**Dated correction, 2026-08-10 — the missing fail-closed evidence is now direct, and the decision is unchanged.** [`a_governed_gather_refuses_at_dispatch_before_arithmetic_recognition`](../../crates/tiler-compiler/src/request/tests.rs) compiles one real Gather program and pins both ordered request layers: the governed target first refuses its exact U32 index type under `DTypeNotDispatchable`, while the same program against a test-only profile carrying that exact dispatch row advances to `dtype-recognized`. [`gather_is_absent_from_the_real_request_recognition_operation_set`](../../crates/tiler-compiler/src/request/tests.rs) bypasses only that program-wide arithmetic check and reaches the real output recognizer's later `operation-set` refusal. Independently, [`gather_is_absent_from_the_governed_fusion_roles`](../../crates/tiler-compiler/src/fusion_legality.rs) names the Gather key against `FusionNumericalCapabilities::classify` and receives `None`. Each check was made to fail by perturbing its own subject: removing the U32 target row, teaching the real recognizer the Gather key, and adding a Gather fusion role. No production capability, recognition rule, fusion role, public boundary, or identity changed; this paragraph repairs only the evidence overstatement recorded above and leaves the accepted body verbatim.

**Link repair — 2026-08-19 by [`repair-the-accepted-decision-records-the-splits-and-retirements-falsified`](../../tickets/repair-the-accepted-decision-records-the-splits-and-retirements-falsified.md), on two link targets in the correction above and on nothing it states.** Both test names were linked to `crates/tiler-compiler/src/request.rs`, which still exists as the spine of a `crates/tiler-compiler/src/request/` module tree — so the links resolved while pointing at a file that no longer holds either test, which the citation gate cannot catch because it checks that a path resolves and not that it holds what the sentence claims. Each test was relocated at its own name rather than by assuming the split: `a_governed_gather_refuses_at_dispatch_before_arithmetic_recognition` and `gather_is_absent_from_the_real_request_recognition_operation_set` are both in `crates/tiler-compiler/src/request/tests.rs`, and the two links now name that file. `gather_is_absent_from_the_governed_fusion_roles` is unmoved in `crates/tiler-compiler/src/fusion_legality.rs`. All three tests, their perturbations, and the evidence claim above are unchanged.

**Dated correction, 2026-08-08 — ADR 0108 was returned for revision, and this decision's no-admission boundary is unchanged.** The proposed expression route rested on three findings that a source re-read contradicted: an append-only access tag can preserve old identity bytes; `IndexRegionBuilder::prepare_access` establishes rank equality before the verifier's coordinate/extent `zip` sites; and the existing unknown reasons neither promise eventual closure nor make a data-dependent bound undecidable in principle. `decide_gather_index` is factored for reuse by a future host-side validator, as this record's named-enforcement-boundary rule permits. The proposed expression node was also incomplete as a nested logical read and its public-boundary census counted private `IndexNode` while omitting authoring and validation surfaces. The accepted Context's conclusions that the gap is “structural rather than a vocabulary choice” and that “the obstacle is the access record's shape, not a missing expression form” are therefore not established by the cited facts: those facts diagnose the current absence but leave a complete nested read/value expression and an append-only tagged access open for comparison. Likewise, the accepted rejected-alternative paragraph establishes the non-weakening obligation but does not prove that every complete expression candidate would weaken the direct verifier; a bare node remains insufficient, and either complete candidate must prove preservation end to end. The accepted “What acceptance did not commit to” paragraph and final Consequences bullet also describe the former proposed ADR 0108 as having decided the expression shape and left only timing open; both statements are retained verbatim and retired by this correction. None of those corrections changes what Tom accepted here: gather remains a semantic family and nothing below it. The completed research ticket no longer holds a live decision; [ADR 0108](0108-site-a-data-dependent-index-coordinate-on-the-expression.md) remains `proposed`, and [`revise-adr-0108-with-a-complete-data-dependent-index-vertical`](../../tickets/revise-adr-0108-with-a-complete-data-dependent-index-vertical.md) owns both the representation and timing questions.

## Context

The pinned `Qwen/Qwen3-0.6B-Base` profile's *first* operation is a tied embedding
lookup: `[T]` token IDs select rows of a `[151936, 1024]` F32 matrix. It is one
occurrence per forward pass against 253 contractions, so its cost is negligible
and its expressibility is not — with no admitted access class the model's first
operation cannot be stated at all, and the alternative is a different product
boundary rather than a different implementation. That boundary question was
decided elsewhere and the gather stays inside.

[ADR 0046](0046-separate-logical-access-from-storage-addressing.md) bounds the
initial index-expression vocabulary and "rejects iteration-by-iteration
multiplication and tensor-data-derived indices". A token ID read from an operand
and used as a row coordinate is exactly a tensor-data-derived index. The same ADR
separately reserves that "data-dependent gather, scatter, sparse iteration, and
data-dependent cardinality require later explicit IR contracts", and its
consequences state that "indirect operations remain addable without weakening the
verifier for the initial direct-access language".

This record is that later explicit contract for the gather half. It is subordinate
to ADR 0046 rather than an amendment to it, and the condition ADR 0046 attaches is
what fixes its shape: nothing here may weaken the direct-access verifier.

The implemented index layer is narrower than the contract paragraph and the gap is
structural rather than a vocabulary choice. `IndexNode` has five variants and every
operand of every one is a literal, a domain-dimension ordinal, or one declared
shape symbol; `IndexExprClass` has three variants and no data-dependent member; and
`AccessData` carries a single tensor ordinal, so an access has nowhere to name a
second tensor as a coordinate source. The obstacle is the access record's shape,
not a missing expression form.

## Decision

An indirect gather is admitted as a **semantic operation family**, and as nothing
below the semantic layer.

`tiler::gather-f32@1` takes a `tiler::f32@1` source and a `tiler::u32@1` index
operand, carries one named gathered axis as a canonical unsigned attribute, and
derives its result by composing the index operand's shape into the position the
gathered axis occupied: for a source of rank `n` gathered on axis `a` by an index
operand of rank `m`, the result has rank `n - 1 + m`. The result shape is derived
and never declared. A rank-zero index operand is admitted and drops the gathered
axis; a rank-zero source is refused.

The family is one key carrying its gathered axis as a typed attribute rather than
a key per source rank or per index rank, on the rule
[ADR 0087](0087-model-contraction-as-one-keyed-family-with-an-index-structure.md)
applied to the contraction and for its three transferring reasons: a frontend must
never choose among keys, a per-class key set grows without bound, and generalizing
a fixed key later migrates every identity that named it. The admitted index
identity is likewise a property of the signature rather than of the key, so
widening it is an additive registration under this key.

**The index-expression vocabulary is unchanged.** No `IndexNode` variant reads
tensor data, no `IndexExprClass` member is added, and `AccessData` still carries
one tensor ordinal. An occurrence of this family therefore reaches no index region,
no lowering capability resolves it, and no fusion role classifies it. A program
stating one fails closed at the request boundary. That is the decision, not a
deferral inside it: admitting a data-dependent form into the index language is a
separate question whose answer must satisfy ADR 0046's non-weakening condition, and
this record deliberately does not answer it.

**Bounds are a semantic precondition discharged at a named enforcement boundary.**
Every index element must lie in `0..extent` of the gathered axis. The values are
tensor data, so the obligation is not decidable at construction; it is proved
statically or validated at a named boundary, and a semantic validation failure is
never a plan miss. An out-of-range index is refused naming the element position,
the value, and the extent. It is **never clamped to the axis and never wrapped
modulo its extent**: both conventions are attested in primary sources and they
return a different tensor for one program rather than a different diagnostic, so
inheriting either would make a frontend's meaning depend on which specification its
author had read. The named boundary that exists today is the reference evaluator;
no physical plan has one, which is a second reason no occurrence reaches a plan.

**Duplicate indices are admitted and the duplicate-write rule is stated, not
implemented.** The read map may be many-to-one, which ADR 0046 already permits for
reads. The corresponding write rule belongs to scatter: a scatter's write map may
not be many-to-one without either an exclusive-ownership proof or an explicit
combining contract. Stating it here is what makes admitting scatter later additive
rather than a reinterpretation of this record.

**Determinism.** The result is a total function of the source, the index operand,
and the gathered axis. There is no accumulation, no reassociation freedom, no
ordering choice, and no reduction, so the family declares no numerical permission
and needs none. Every result element is a source element unchanged, so an
exceptional payload crosses a gather exactly as it left the source.

**A signed index operand is refused by name** rather than admitted and then bounded
below zero, because a signed index raises negative indexing — a second convention
the primary authorities diverge on. Refusing the type refuses the question;
admitting the type and rejecting negative values would answer it silently.

Under [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) the
key, the attribute, and the vocabulary above are a **labelled draft** until Tom
accepts their exact included and excluded surface.

## Consequences

- The workload's first operation is expressible and reference-evaluable. Nothing
  composed from it compiles, plans, or runs.
- ADR 0046 stays `accepted` with its index-expression rejection intact. The two
  records agree because this one admits no expression form.
- Q-SHAPE-007's gather half gains bounds, determinism, and validation rules for
  reads, and states the duplicate-write rule it does not implement. The scatter
  half stays reserved and unfired.
- A family registered with no realization law, no lowering capability, and no
  fusion role is the fail-closed default rather than a gap: `classify` returns
  `None`, `derive_member` yields no legality, and the request boundary refuses.
- The reference crate acquires its first non-float value type, `tiler::u32@1`,
  because a reference capability is keyed by an exact resolved signature. It admits
  no integer arithmetic and buys a coordinate carrier rather than an integer
  profile.
- Admitting the access class *below* the semantic layer remains open and is now
  bounded by a stated condition rather than by an absence. [ADR
  0108](0108-site-a-data-dependent-index-coordinate-on-the-expression.md) since
  decided the shape that admission would take and deferred taking it, so what
  remains open is the timing and its trigger rather than the design.

## Alternatives considered

**Admit a data-dependent `IndexNode` variant now.** It would let the family reach
an index region, and it would weaken the verifier for the direct-access language —
every bounds proof, interval propagation, and totality argument in the index layer
is written over expressions whose operands are literals, dimensions, and symbols.
ADR 0046 admits indirect operations on exactly the condition that this does not
happen, so taking this route would put the two records in conflict rather than in
sequence.

**Move the lookup outside Tiler and accept materialized `[T, 1024]` activations.**
Cheaper, and it was eliminated on the product boundary rather than here: it does not
remove the embedding matrix from the boundary, because the tied matrix is still the
vocabulary projection's weight; it saves no time, one gather against 253
contractions; and it would make the consumer perform the model's first operation.

**A key per source rank, or per index identity.** Rejected on ADR 0087's three
reasons, which transfer intact.

**Clamp or wrap an out-of-range index.** Both are attested in primary sources and
both return a plausible tensor for a program that named a coordinate it does not
have, which is the failure mode this corpus refuses. Admitting either would also
make the family's meaning depend on which convention a frontend author had read.

**Admit signed index identities and reject negative values at the bound.** It
answers the negative-indexing convention in one direction without saying so, and it
makes the refusal a property of the data rather than of the signature.
