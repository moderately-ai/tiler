---
id: decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles
title: Decide the authority and change policy for conformance universes and goal profiles
status: review
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, cost-protected-review-versus-signed-conformance-authority]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, decision, conformance-progress, conformance-authority, verification]
claimed_from: todo
assignee: codex
lease_expires_at: 1787606631
---
# Decide the authority and change policy for conformance universes and goal profiles

## Goal

A decision-ready authority and change-policy packet separating the owner-derived system universe, the human-accepted goal profile, exceptions/applicability, and the evidence snapshot so none can silently redefine another.

## Work

1. Re-audit the inventory and threat-model findings at the exact base.
2. Name the owner and minting rule for the system-universe identity, goal-profile identity, exception/applicability reasons, tombstones, profile lineage, evidence snapshot, and acceptance provenance.
3. Decide what additions, removals, replacements, required-to-optional changes, and required-to-`N/A` changes mean. Preserve denominator shrinkage as an authority event rather than evidence progress.
4. Compare the status quo, a profile-owned denominator, owner-derived universe plus profile selection, an append-only profile lineage, a signed external root, and deferral.
5. Eliminate any option that lets policy omit newly declared features silently, lets execution mint normative authority, or lets an implementation edit its own requirement and baseline unnoticed.
6. State review separation, backwards/forwards comparison rules, tombstone lifetime, profile supersession, and how an unsupported-but-correctly-refused capability is represented.
7. Present only the nondominated frontier with strongest counterarguments, reversal evidence, negative controls, and required follow-up tickets.

## Non-goals

- Do not implement a profile schema or signing mechanism.
- Do not decide which exact features the first profile includes.
- Do not turn the system universe into every source function or test.

## Stop conditions

Stop for Tom when more than one top-tier authority placement survives, or when sound enforcement requires an external authority whose ownership he has not accepted.

## Acceptance

- Universe, goal policy, applicability/exception, and evidence authorities are singular and non-overlapping.
- Every denominator-changing operation has a fail-loud identity and review consequence.
- The packet is Pareto-complete and names one recommendation or one concrete decision between frontier candidates.
- No implementation work is implied by an assumed authority.

## Exact-base Fact audit

**Measurement — 2026-08-24.** This decision was re-audited on the claimed branch at exact base `891df69e8f3fab1cc1659ab30482aef1c762713d`. The worktree was clean before this edit, `main` and `origin/main` were at `0 0`, `tkt reconcile --format json` reported no findings, and the only live claim was this ticket's Codex claim.

| premise | verdict | exact-base evidence and consequence |
| --- | --- | --- |
| Tiler already has one closed owner-emitted system universe. | **False.** | The complete owner audit at [`The closed-world conformance claim universe by owner`](../docs/research/verification/conformance-claim-universe-by-owner.md) still finds bounded owner snapshots and explicit unknown families. `git diff --name-only 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a..891df69e8f3fab1cc1659ab30482aef1c762713d -- crates` returned no paths, so no source change after that audit could have supplied the missing singular owner. The global universe must join owner-minted subjects and refuse completeness while any family remains unknown. |
| The conformance crate may decide semantic meaning or support maturity. | **False.** | [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md), anchors `Not a second semantic authority` and `No support-matrix authority`, keeps the crate an evidence consumer. It may validate and report owner receipts; it cannot mint the denominator, oracle, or normative maturity. |
| Repository-local checks can restrain an actor able to rewrite all five authority classes and their tests. | **False.** | The root spike's anchor `No in-repository mechanism is claimed` and the accepted correctness contract's independent-oracle requirement rule this out structurally. At this base the tracked tree still contains no workflow, recognized `CODEOWNERS`, tracked hook, or `SECURITY.md`; GitHub reported `main` unprotected and no rulesets; the latest 20 commits all had Git signature status `N`. |
| Protected review, signed authority, mixed-diff exclusion, and witnessed history are one interchangeable control. | **False.** | The complete [authority threat model](../docs/research/verification/conformance-authority-threat-model.md) separates `P`, `K`, `M`, and `T`: approval placement, client-verifiable signed policy, work-item-shape enforcement, and non-equivocation/history are independent properties with different outage and compromise boundaries. |
| A signature over only goal-profile and exception bytes closes the manipulation threat. | **False.** | Denominator, verifier, oracle, and evidence-baseline changes can still manufacture progress. `K` must bind a deterministic canonical closure over all five authority classes. |
| A profile may define unsupported system subjects out of existence. | **False.** | The owner audit and root spike separate system universe from goal policy. Exclusion or `N/A` is a versioned policy decision with authority and lineage; missing owner disposition is an audit failure, not gray. |

The ticket's original items are requirements rather than factual claims. Their two research dependencies are `done`; the exact-base audit above confirms that their results still decide the same question and repairs no ticket premise.

## Accepted decision — P+K+M+T

**Decision and provenance.** Tom accepted `P+K+M+T` as the comprehensive long-term conformance-authority target on 2026-08-24 in the coordination conversation, after the twelve-member Pareto frontier and the narrower `P` bootstrap were presented. The selection is one architecture with four independently required properties:

- `P`: fresh protected human approval of every authority change on the exact latest source state;
- `K`: a policy-approver-held threshold signature over a canonical manifest binding all five authority classes;
- `M`: an externally governed exact-diff attestation mechanically rejecting a work item that mixes authority with implementation or evidence;
- `T`: witnessed append-only publication, monitoring, checkpoints, and independently retained accepted content sufficient to detect divergence and support the stated recovery path.

The choice is intentionally stronger than the lowest-cost present-day recommendation. Tiler is to support long-lived independent consumers, internal compiler features below the user-visible surface, coherent repository/host rewrite resistance, strict separation of authority from implementation evidence, and auditable history without re-architecting the denominator later.

### Singular authorities and identities

1. **System universe.** Feature and invariant subjects are minted by their real semantic, optimizer, planner, verifier, artifact, runtime, target, numerical, and performance owners. A global system-universe snapshot is a canonical join over those owner identities. It may report an explicitly unknown owner family; it may not invent or silently omit a subject.
2. **Goal policy and applicability.** A versioned goal profile selects obligations from one exact universe snapshot. Required/optional state, accepted exclusion, applicability, exception reason, expiry/review trigger, predecessor, and acceptance provenance belong to policy authority, not to a run.
3. **Verifier.** The exact `audit`/`regress`/`qualify` rules, schemas, and executable/source identity form their own versioned authority class. A report is evidence only under the verifier identity that interpreted it.
4. **Oracle.** Normative semantic/reference owners retain oracle authority. The conformance authority binds their exact identities and comparison contracts without copying their fields or recomputing expectations from the implementation.
5. **Evidence baseline.** Accepted receipt-set lineage binds source, universe, profile, cases, oracle, target, toolchain, environment, selected plan, artifact, terminal outcome, and comparison. Executing a run may propose evidence; it cannot accept its own baseline or normative maturity.

`K` signs a canonical manifest whose deterministic closure binds all five classes, schema/resolver version, monotone version, predecessor, signing roles/threshold, validity policy, and root version. `P` and `K` must approve the **same exact source/closure identity**. Two successful but unbound workflows are not dual authority.

`M` classifies the exact predecessor/successor diff under configuration outside ordinary repository writes and binds its attestation to that same identity. It enforces work-item separation only; it supplies no semantic approval. `T` logs the exact `P+K`-approved leaf and the `M` attestation, requires inclusion and independently witnessed consistency before acceptance, and retains the accepted leaf/content plus approval proof or an authorized mirror. A checkpoint without content is detection evidence, not recovery.

### Change policy

- Adding a new owner subject changes the universe identity and requires an explicit profile disposition before `audit` can succeed.
- Removing or replacing a subject preserves a tombstone and predecessor/successor relation throughout the version-1 lineage. A future major-lineage compaction requires its own signed, protected, separated, witnessed migration proof; deletion is never inferred from absence.
- Required-to-optional, required-to-`N/A`, applicability, exception, evidence-requirement, expiry, or denominator changes create a new policy identity and are reported as authority changes, never implementation progress.
- A correctly refused unsupported route may satisfy a refusal obligation. It does not make the capability supported. Missing authority or evidence is yellow/insufficient; genuine out-of-profile gray requires the accepted policy reason.
- Evidence additions, losses, contradictions, and maturity changes move only evidence identity. They cannot alter universe or policy identity.
- Unavailable `P`, `K`, `M`, or `T` blocks a new accepted authority/qualification. No selected property silently falls back to a weaker member of the frontier.

### Bootstrap and authoritative-progress boundary

A `P`-only bootstrap may exercise discovery, reports, and review mechanics, but every output is explicitly **provisional and non-authoritative**. It cannot establish an accepted baseline, emit an authoritative `qualify` result, or be presented as authoritative progress. The same rule applies to any intermediate deployment missing `K`, `M`, or `T`. The ordinary harness may still produce honest mixed-color audits while the selected authority is unavailable; the missing authority is itself visible and qualification remains nonzero.

The rollout order is dependency sequencing, not a reduced target: establish owner identities and evidence algebra; define canonical receipt/freshness joins; establish `P` and `K`; prove their exact-source composition; establish `M`; establish `T` with retention/recovery; then allow the command contract and first profile to define authoritative qualification.

### Counterarguments and reversal evidence

- **Against `P+K`:** two approval systems add correlated human work and outage paths. Evidence that host-side rejection adds no material error prevention and that signer/host compromise need not be independently tolerated could justify a future signed successor selecting `K` alone; it cannot be assumed during bootstrap.
- **Against `M`:** a complete externally controlled path taxonomy is operationally expensive and does not stop a dishonest authority approver using separate changes. A provider-native externally governed predicate with equivalent exact-diff coverage could replace the custom mechanism; inability to close the taxonomy blocks qualification rather than weakening the property.
- **Against `T`:** logging, witnesses, monitors, retention, and recovery drills are substantial pre-production cost. Evidence that Tiler will never have independent consumers or require non-equivocation/history could support a future authority decision removing it; a repository writer cannot make that change as evidence progress.
- **Against implementing cryptography now:** the canonical five-class closure and several owner-private identities do not yet exist. This counterargument controls sequence, not destination: the selected mechanism tickets remain dependency-blocked until those identities are decision-ready.

The decision has one nondominated selected design and no remaining product choice. Concrete provider, threshold, key custody, classifier, witness, retention, and recovery mechanisms remain bounded design/operations work; none may invent authority or cross into implementation merely because this ticket is closed.

## Descendant graph selected by the decision

- [`record-the-pkmt-conformance-authority-architecture`](record-the-pkmt-conformance-authority-architecture.md) carries this accepted decision into the ADR/catalog/contract corpus.
- [`design-protected-review-authority-for-conformance-policy`](design-protected-review-authority-for-conformance-policy.md) and [`establish-protected-review-authority-for-conformance-policy`](establish-protected-review-authority-for-conformance-policy.md) separate `P` design from host mutation/operations.
- [`design-threshold-signed-five-class-conformance-authority`](design-threshold-signed-five-class-conformance-authority.md) and [`establish-threshold-signed-five-class-conformance-authority`](establish-threshold-signed-five-class-conformance-authority.md) separate `K` schema/ceremony design from implementation and key operations.
- [`design-the-exact-source-pk-conformance-authority-composition`](design-the-exact-source-pk-conformance-authority-composition.md) and [`bind-protected-review-and-signed-conformance-authority`](bind-protected-review-and-signed-conformance-authority.md) own the exact-source `P+K` contract and realization.
- [`design-the-external-mixed-diff-conformance-attestation`](design-the-external-mixed-diff-conformance-attestation.md) and [`establish-external-mixed-diff-conformance-attestation`](establish-external-mixed-diff-conformance-attestation.md) own `M` design and external enforcement separately.
- [`design-witnessed-conformance-authority-history-and-recovery`](design-witnessed-conformance-authority-history-and-recovery.md) and [`establish-witnessed-conformance-authority-history`](establish-witnessed-conformance-authority-history.md) own `T` design and external services/storage separately.
- [`authorize-the-pkmt-conformance-authority-mechanism-implementation`](authorize-the-pkmt-conformance-authority-mechanism-implementation.md) is the parked Tom-only movement gate between the completed design family and every implementation/operations descendant.

The command-contract ticket depends on the terminal `T` **design** owner, so research can define exact fail-closed semantics without pretending the external mechanisms are already deployed. Any later qualifier implementation must depend on the corresponding establishment tickets. No implementation/operations ticket becomes ready merely because the architectural choice is recorded.

## Refs

- [`inventory-the-closed-world-conformance-claim-universe-by-owner`](inventory-the-closed-world-conformance-claim-universe-by-owner.md)
- [`cost-protected-review-versus-signed-conformance-authority`](cost-protected-review-versus-signed-conformance-authority.md)
- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
