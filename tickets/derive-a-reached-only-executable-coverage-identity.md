---
id: derive-a-reached-only-executable-coverage-identity
title: Derive a reached-only executable coverage identity
status: in-progress
priority: p1
dependencies: [place-index-refinement-evidence-under-an-ir-owned-verifier, canonicalize-index-refinement-occurrence-ordinals]
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/decisions, contracts/artifacts, research/program-planning]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [design, implementation, identity]
claimed_from: ready
assignee: agent-coverage-resume
lease_expires_at: 1785873940
---
## User-visible outcome

Executable stage coverage is minted only from a completed IR refinement receipt and identifies the selected, reached proof needed to replay that coverage, without making an unused registry/provider row invalidate an otherwise identical executable artifact.

## Facts and stop evidence

`IndexRefinementReceiptIdentity` currently nests complete semantic and scalar snapshot provenance through three routes: `encode_subject_identity` writes `SemanticCapabilityAuthority::registry_snapshot`; `encode_authority_identity` writes the same semantic snapshot plus `CanonicalScalarRegistrySnapshotIdentity`; and `encode_receipt_identity` writes `ScalarAuthorityEvidence::{semantic_snapshot, scalar_snapshot}`. The preserved `bind-stage-coverage-to-index-refinement-identity` draft folds that opaque identity into both independent stage encoders.

ADR 0072 requires selected plan/artifact provenance to exclude unused providers. `crates/tiler-artifact/src/program/tests.rs::an_unused_semantic_provider_revision_does_not_change_identity` is the existing executable invariant. The preserved draft broad test cannot honestly satisfy both statements: its compiler fixture first refuses the nonstandard semantic/scalar authority pairing, and any correctly minted receipt over those exact registries would then change when the unused provider revision moves.

## Candidate projections and elimination

1. **Fold the existing complete receipt identity.** Non-forgeable and replay-resistant, but eliminated: complete snapshots make unused semantic/scalar rows change program and artifact identity, directly violating ADR 0072 and the existing unused-provider invariant.
2. **Let program/artifact callers assemble a reached tuple from receipt accessors.** Can exclude snapshots, but eliminated: a caller can cross an occurrence, region, numerical contract, law row, scalar definition projection, or provider projection from different receipts. That recreates the substitution gap the coverage binding exists to close.
3. **Mint a second opaque executable-coverage projection only from a completed receipt.** Survives. IR owns construction and retains graph plus canonical occurrence binding, governed numerical contract, exact region, operation-specific law/provider row, reached semantic definition and admission projections, reached scalar definition and admission projections, reached type definition and admission projections, exact operand/result bindings, and residual proof identities. It excludes the complete semantic registry, scalar registry, and law-registry snapshots: those remain request/verifier authority, not selected executable provenance. A proof gap cannot mint either receipt or coverage identity.
4. **Change the receipt identity itself to the reached-only projection.** Eliminated. It reduces duplication only by conflating two equality questions that ADR 0072 separates: whether a receipt was minted under the exact frozen verifier/request authority, and whether selected executable evidence is unchanged by unused authority. It also silently changes the already-landed receipt v1 subject and its `PartialEq` semantics. Keeping a strict receipt plus a reached executable projection preserves both questions explicitly and lets future registry extensions remain cache-stable without weakening replay validation.

## Derived recommendation

Candidate 3 is the current sole safe recommendation: it keeps receipt verification strict while giving executable identity the ADR 0072 subject it actually owns. This is a consequential public IR/program boundary and remains a tested draft for Tom before acceptance.

## Implementation keys

- Add an opaque named executable-coverage identity/projection whose only public constructor is proof-derived from a completed `IndexRefinementReceipt`; do not expose a raw-byte or independently-fielded constructor.
- Define and document the exact retained reached subjects and why each prevents replay/substitution. Preserve canonical graph/occurrence stability across equivalent authoring orders.
- Keep complete snapshots in verifier/request authority only; never encode unused registry/provider rows in kernel-program or artifact identity.
- Advance every owning IR receipt/projection, kernel-program stage/program, and artifact stage/program domain exactly once on the merged tree; enumerate and recompute every pin.
- Preserve the two independent program/artifact stage encoders.

## Deliberate perturbations

- Same graph, same reached operations/providers, but an unused semantic provider revision changes: executable coverage, kernel program, and artifact identities remain equal.
- Same reached scalar set, but an unused scalar definition/provider changes: identities remain equal.
- Reached semantic or scalar provider revision changes: identity changes.
- Region, numerical contract, canonical occurrence, law row/provider, operand/result binding, or residual proof changes one at a time: identity changes.
- Cross two completed receipts with equal shapes/interfaces: no public construction can form the crossed coverage; builder rejects foreign graph/duplicate occurrence.
- Equivalent graphs authored in different valid insertion orders mint equal canonical occurrence-bound executable identities.
- Proof pending/refusal has no executable coverage spelling.

## Closes when

The reached-only proof-derived public draft is accepted; all perturbations pass; both independent encoders and identity ledgers agree; the blocked stage-coverage ticket can consume it without weakening ADR 0072; targeted affected-crate checks and the full gate pass.
