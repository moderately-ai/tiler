---
schema: "tiler-doc/v1"
id: "ADR-0114"
kind: "decision"
title: "Require protected, signed, separated, and witnessed conformance authority"
topics: ["conformance", "verification", "authority", "security", "governance"]
catalog_group: "documentation-governance"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.verification.conformance-authority-threat-model", "tiler.research.verification.conformance-claim-universe-by-owner"]
depends_on: ["ADR-0106"]
ticket: "record-the-pkmt-conformance-authority-architecture"
---

# 0114: Require protected, signed, separated, and witnessed conformance authority

**Status:** accepted by Tom on 2026-08-24 in the coordination conversation, relayed first-hand through [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md). Tom selected `P+K+M+T` as one comprehensive long-term architecture after reviewing the twelve-member nondominated frontier and the narrower `P` bootstrap. This record carries that accepted decision without selecting any provider, schema, algorithm, threshold, key holder, classifier, witness topology, retention service, or public API.

## Context

Conformance progress is a claim about more than user-visible operations. It includes semantic, optimizer, logical- and physical-planning, schedule, kernel, artifact, runtime, target, numerical, and performance obligations. Those subjects have different real owners. A conformance harness can join their identities and evidence, but [ADR 0106](0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) forbids it from becoming a second semantic authority or support-matrix authority.

Repository-local tests cannot contain an actor who may rewrite the feature denominator, goal policy, verifier, oracle, evidence baseline, and the checks intended to protect them in one coherent change. Nor are protected review, signatures, mixed-diff exclusion, and witnessed history interchangeable: the [authority threat model](../research/verification/conformance-authority-threat-model.md) derives four distinct properties with different compromise, outage, and recovery boundaries. The [owner inventory](../research/verification/conformance-claim-universe-by-owner.md) also finds bounded owner snapshots rather than one already-complete global universe. Any accepted architecture must therefore join owner-minted subjects and fail closed while an owner family is unknown; execution cannot manufacture the denominator it is graded against.

## Decision

Tiler's comprehensive long-term conformance authority is one architecture with four independently required properties:

- `P`: fresh protected human approval of every authority change on the exact latest source state;
- `K`: a policy-approver-held threshold signature over a canonical manifest binding all five authority classes;
- `M`: an externally governed exact-diff attestation mechanically rejecting a work item that mixes authority with implementation or evidence;
- `T`: witnessed append-only publication, monitoring, checkpoints, and independently retained accepted content sufficient to detect divergence and support the stated recovery path.

The authority closure keeps five singular classes distinct:

1. **System universe.** Feature and invariant subjects are minted by their real semantic, optimizer, planner, verifier, artifact, runtime, target, numerical, and performance owners. A global system-universe snapshot is a canonical join over those owner identities. It may report an explicitly unknown owner family; it may not invent or silently omit a subject.
2. **Goal policy and applicability.** A versioned goal profile selects obligations from one exact universe snapshot. Required/optional state, accepted exclusion, applicability, exception reason, expiry/review trigger, predecessor, and acceptance provenance belong to policy authority, not to a run.
3. **Verifier.** The exact `audit`/`regress`/`qualify` rules, schemas, and executable/source identity form their own versioned authority class. A report is evidence only under the verifier identity that interpreted it.
4. **Oracle.** Normative semantic/reference owners retain oracle authority. The conformance authority binds their exact identities and comparison contracts without copying their fields or recomputing expectations from the implementation.
5. **Evidence baseline.** Accepted receipt-set lineage binds source, universe, profile, cases, oracle, target, toolchain, environment, selected plan, artifact, terminal outcome, and comparison. Executing a run may propose evidence; it cannot accept its own baseline or normative maturity.

`K` signs a canonical manifest whose deterministic closure binds all five classes, schema/resolver version, monotone version, predecessor, signing roles/threshold, validity policy, and root version. `P` and `K` approve the **same exact source/closure identity**. Two successful but unbound workflows are not dual authority.

`M` classifies the exact predecessor/successor diff under configuration outside ordinary repository writes and binds its attestation to that same identity. It enforces work-item separation only; it supplies no semantic approval. `T` logs the exact `P+K`-approved leaf and the `M` attestation, requires inclusion and independently witnessed consistency before acceptance, and retains the accepted leaf/content plus approval proof or an authorized mirror. A checkpoint without content is detection evidence, not recovery.

### Change policy

- Adding a new owner subject changes the universe identity and requires an explicit profile disposition before `audit` can succeed.
- Removing or replacing a subject preserves a tombstone and predecessor/successor relation throughout the version-1 lineage. A future major-lineage compaction requires its own signed, protected, separated, witnessed migration proof; deletion is never inferred from absence.
- Required-to-optional, required-to-`N/A`, applicability, exception, evidence-requirement, expiry, or denominator changes create a new policy identity and are reported as authority changes, never implementation progress.
- A correctly refused unsupported route may satisfy a refusal obligation. It does not make the capability supported. Missing authority or evidence is yellow/insufficient; genuine out-of-profile gray requires the accepted policy reason.
- Evidence additions, losses, contradictions, and maturity changes move only evidence identity. They cannot alter universe or policy identity.
- Unavailable `P`, `K`, `M`, or `T` blocks a new accepted authority or qualification. No selected property silently falls back to a weaker member of the frontier.

### Bootstrap and authoritative-progress boundary

A `P`-only bootstrap may exercise discovery, reports, and review mechanics, but every output is explicitly **provisional and non-authoritative**. It cannot establish an accepted baseline, emit an authoritative `qualify` result, or be presented as authoritative progress. The same rule applies to any intermediate deployment missing `K`, `M`, or `T`. The ordinary harness may still produce honest mixed-color audits while the selected authority is unavailable; the missing authority is itself visible and qualification remains nonzero.

The rollout order is dependency sequencing, not a reduced target: establish owner identities and the evidence algebra; define canonical receipt and freshness joins; establish `P` and `K`; prove their exact-source composition; establish `M`; establish `T` with retention and recovery; then allow the command contract and first profile to define authoritative qualification. No runtime or kernel fast path consumes this authority. It governs verification, reporting, and qualification, never execution semantics or dispatch.

## Consequences

- Authoritative conformance progress and qualification remain unavailable until all four selected properties compose over all five authority classes. Partial tooling is valuable for research and honest audit, but is not silently promoted by vocabulary or presentation.
- Denominator shrinkage, applicability changes, exceptions, and evidence changes remain separately attributable. A run cannot improve its result by redefining its subject population, its verifier, or its baseline.
- The architecture deliberately accepts operational cost and availability dependencies in exchange for coherent-rewrite resistance, client-verifiable policy, work-item separation, non-equivocation, and recoverable history.
- Concrete mechanism work remains behind bounded design tickets and a Tom-only movement gate. This ADR authorizes neither infrastructure nor a public boundary.

The selected mechanism graph is:

- `P`: [`design-protected-review-authority-for-conformance-policy`](../../tickets/design-protected-review-authority-for-conformance-policy.md), followed by [`establish-protected-review-authority-for-conformance-policy`](../../tickets/establish-protected-review-authority-for-conformance-policy.md).
- `K`: [`design-threshold-signed-five-class-conformance-authority`](../../tickets/design-threshold-signed-five-class-conformance-authority.md), followed by [`establish-threshold-signed-five-class-conformance-authority`](../../tickets/establish-threshold-signed-five-class-conformance-authority.md).
- Exact-source `P+K` composition: [`design-the-exact-source-pk-conformance-authority-composition`](../../tickets/design-the-exact-source-pk-conformance-authority-composition.md), followed by [`bind-protected-review-and-signed-conformance-authority`](../../tickets/bind-protected-review-and-signed-conformance-authority.md).
- `M`: [`design-the-external-mixed-diff-conformance-attestation`](../../tickets/design-the-external-mixed-diff-conformance-attestation.md), followed by [`establish-external-mixed-diff-conformance-attestation`](../../tickets/establish-external-mixed-diff-conformance-attestation.md).
- `T`: [`design-witnessed-conformance-authority-history-and-recovery`](../../tickets/design-witnessed-conformance-authority-history-and-recovery.md), followed by [`establish-witnessed-conformance-authority-history`](../../tickets/establish-witnessed-conformance-authority-history.md).
- [`authorize-the-pkmt-conformance-authority-mechanism-implementation`](../../tickets/authorize-the-pkmt-conformance-authority-mechanism-implementation.md) is the parked Tom-only movement gate between completed design and implementation or operations.

Research may define fail-closed contracts before these external mechanisms exist. Any qualifier implementation must depend on their establishment rather than interpreting a completed design document as deployed authority.

## Counterarguments and reversal evidence

- **Against `P+K`:** two approval systems add correlated human work and outage paths. Evidence that host-side rejection adds no material error prevention and that signer/host compromise need not be independently tolerated could justify a future signed successor selecting `K` alone; it cannot be assumed during bootstrap.
- **Against `M`:** a complete externally controlled path taxonomy is operationally expensive and does not stop a dishonest authority approver using separate changes. A provider-native externally governed predicate with equivalent exact-diff coverage could replace the custom mechanism; inability to close the taxonomy blocks qualification rather than weakening the property.
- **Against `T`:** logging, witnesses, monitors, retention, and recovery drills are substantial pre-production cost. Evidence that Tiler will never have independent consumers or require non-equivocation/history could support a future authority decision removing it; a repository writer cannot make that change as evidence progress.
- **Against implementing cryptography now:** the canonical five-class closure and several owner-private identities do not yet exist. This counterargument controls sequence, not destination: the selected mechanism tickets remain dependency-blocked until those identities are decision-ready.

Any reversal is itself an authority change under this decision. It cannot be reported as evidence progress or made by weakening a bootstrap implementation in place.

## Alternatives considered

The accepted packet's nondominated frontier contained twelve combinations of the four independent properties. Tom selected the most comprehensive member because the intended horizon includes independent consumers, internal compiler features below the visible surface, coherent repository/host rewrite resistance, and auditable recovery. A `P`-only bootstrap was retained only as provisional sequencing, not as a competing long-term architecture. Treating `P`, `K`, `M`, or `T` as optional menu items is rejected because each controls a different failure class. Deferral is rejected for the product architecture because the accepted horizon decides which properties are required, while the provider and schema choices remain correctly deferred to their bounded tickets.

## Traceability

The complete option enumeration, eliminations, threat assumptions, cost model, negative controls, and independent derivation remain in [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md), [`cost-protected-review-versus-signed-conformance-authority`](../../tickets/cost-protected-review-versus-signed-conformance-authority.md), the [authority threat model](../research/verification/conformance-authority-threat-model.md), and the [owner inventory](../research/verification/conformance-claim-universe-by-owner.md). This record carries the accepted architecture and its boundaries; those research records carry the derivation.
