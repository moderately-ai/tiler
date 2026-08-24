---
id: cost-protected-review-versus-signed-conformance-authority
title: Cost protected review versus signed conformance authority
status: review
priority: p1
dependencies: []
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification, contracts/navigation]
paths: []
tags: [research, decision, conformance-progress, security]
claimed_from: todo
assignee: conformance-authority-sol
lease_expires_at: 1787605340
---
# Cost protected review versus signed conformance authority

## Goal

A Pareto-complete threat model and authority decision packet for preventing denominator, exception, verifier, and evidence-baseline manipulation from masquerading as conformance progress.

## Work

1. Name distinct actors and powers: accidental contributor error; an LLM agent with repository write access; a reviewer; a maintainer who may merge protected paths; an actor able to rewrite profile, verifier, tests, and baseline together; and compromise of any signing key or external store.
2. Enumerate the status quo, path-protected human review, split approval for profile/exception versus implementation changes, an independently signed profile/exception root, an append-only external transparency record, and deferral.
3. For each option state exactly which attacks it detects, prevents, or cannot address; operational setup; key/reviewer recovery; offline development behavior; rotation and revocation; repository and host cost; and failure mode when the authority is unavailable.
4. Treat “the repository can restrain an actor authorized to rewrite all repository checks” as an eliminated false claim.
5. Compare survivors on correctness, fail-closed strictness, maintainability, compatibility, host cost, and recovery burden. State each survivor's strongest counterargument and reversal evidence.
6. Recommend the smallest authority sufficient for the first profile, and name a trigger for adopting a stronger one.

## Non-goals

- Do not install hooks, branch protection, signing infrastructure, or CI.
- Do not claim cryptography provides review quality or protects a compromised signer.
- Do not let an availability failure silently downgrade a required authority.

## Stop conditions

Stop for Tom when every sound option requires authority outside ordinary repository writes, or when the chosen threat model changes who is permitted to approve project policy.

## Acceptance

- The threat matrix distinguishes prevention, detection, attribution, recovery, and unavoidable trust.
- Only nondominated authority options reach Tom, with costed deployment and strongest counterarguments.
- The packet states the first-profile recommendation and the evidence that would reverse it.
- Follow-up dependencies are explicit; no security work is hidden inside profile implementation.

## Outcome

Initial research content commit: `ac99577facf54528f674eb3011c1d644c5ee6140`. Mixed-diff review repair: `cdb9adc381a855dc3d7d3f01653f703d04e04902`. Signed-authority repair: `18ab839e293056e6a2bf3ec3ae503f7cc8871e60`. Complete transparency-product repair: `3aa62d2ac5d35e7b7d247985de25f0dcaf11712a`.

The retained [conformance authority threat model and decision packet](../spikes/verification/conformance-authority-threat-model/README.md) covers denominator, exception/profile, verifier, oracle, and evidence-baseline manipulation. The [research record](../docs/research/verification/conformance-authority-threat-model.md) supplies the governed traceability edge, and both hand-maintained catalogs carry the new records.

### Exact-base Fact audit

- **False:** the current repository/local report already has an enforcement boundary. A full-tree exact-name census at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a` found no tracked workflow, `CODEOWNERS` at any path (including `docs/CODEOWNERS`), hook, or security policy; GitHub reported no protection on `main` and no ruleset.
- **Verified but imprecise as enforcement:** `AGENTS.md` and ADR 0075 require human review and gates, but those prose rules are not a server-side prevention mechanism.
- **Verified as a proposal, not accepted policy:** the root spike says an implementation change cannot change its own profile, oracle, exception ledger, and baseline in the same work item. The downstream authority ticket still owns adoption.
- **False:** protected owner review and separate policy/implementation work items are distinct enforced authorities under the current GitHub primitives. Native protected review and `CODEOWNERS` can require an owner for matched paths, but do not reject a mixed authority-plus-implementation diff. A distinct split requires a trusted external mixed-diff classifier and required check, or it remains reviewer judgment.
- **Verified:** owner-derived universe, profile/exception policy, verifier, independent oracle, and evidence-baseline lineage are distinct authority classes.
- **False/materially incomplete:** signing only the profile/exception root covers this ticket's threat. A writer can weaken the denominator enumerator, verifier, oracle authority, or baseline lineage without invalidating such a signature. A surviving signed manifest must bind all five classes.
- **False by the actor's defined power:** repository-local checks can restrain an actor authorized to rewrite all repository authorities together.
- **False:** cryptography supplies review quality, protects a compromised signing threshold, or makes a logged leaf legitimate.
- **False:** every sound signed authority also requires protected review to supply policy judgment. Automated/unreviewed signing is not a semantic authority, but designated policy approvers can review the semantic authority diff and make threshold signing the approval act. Standalone `K` is sound for A0–A4 while that threshold, canonical resolver, and client root remain honest.
- **False:** witnessed transparency requires a `K`-signed policy leaf. While A3/host compromise is excluded, `P` can authorize an exact commit/content closure and an independent witness can authenticate and log the exact latest-state host approval. This yields `P+T` without granting it `K`'s host-rewrite resistance.
- **False on the bounded current row:** existing Git signatures provide a reusable conformance authority. HEAD and the latest 20 commits were unsigned, and there was no configured signing rule; commit signing would still need role, threshold, root, recovery, and protected-object design.

### Independent-review corrections, frontier, and recommendation

The three independent review corrections are retained: the split needs its own enforcement; policy-approver-held signing is a standalone authority; and transparency may witness either an authenticated `P` leaf or a `K` leaf. The packet now derives the full product rather than sampling combinations.

- `P`: protected combined human-owner review — the smallest current recommendation for the first profile while the protected policy owner and rules/bypass administrator are trusted. It permits combined authority/implementation diffs and relies on owner judgment.
- `K`: policy-approver-held threshold signing of a canonical, versioned five-class manifest — independently sound for A0–A4 and client-side repository-rewrite resistance while A6/root/resolver compromise is excluded. It needs no protected-review machinery but pays key/schema/recovery cost and may let unauthoritative repository content merge before authority-aware refusal.
- `P+K`: both independently required — defense in depth and better premerge feedback, not a prerequisite for `K`. Its A3-only/A6-only tolerance requires independent administration and a checked binding between the exact host-approved source and signed manifest.
- `M`: optional external mixed-diff classifier/attestation bound to the selected base. It adds mechanical work-item separation but not protection from a dishonest selected policy approver using separate changes.
- `T`: optional witnessed transparency over the selected authenticated leaf. `P+T` binds exact commit/content closure plus authenticated host approval; `K+T` logs the signed manifest; `P+K+T` binds both. Recovery after deletion also requires retained accepted content/approval proof or an authorized mirror.

The complete frontier is the `3 × 2 × 2` product of bases `{P, K, P+K}`, optional `{M}`, and optional `{T}`: twelve explicitly enumerated conditional survivors, including `P+T` and `P+M+T`. No-base products, unenforced split, incomplete/automated signatures, unauthenticated `P` log leaves, and unbound `P+K` workflows are eliminated. Tom's ordered branch is: (1) host rewrite in scope selects a `K` base, otherwise `P`; (2) whenever `K` is introduced, choose standalone `K` versus independently bound `P+K`; (3) decide `M`; then (4) add `T` to the already selected base without changing it. A future trigger that introduces `K` resumes question 2; `T` never silently forces `K`. The current narrow recommendation remains `P` without `M` or `T`.

External-store compromise is analyzed as separate corruption/substitution, rollback, freeze, withholding, and equivocation powers. `K` rejects corrupted bytes and retained-state rollback; `P+T` rejects content/receipt/checkpoint mismatches; freeze needs a freshness boundary and trusted time; no approval proof makes a withholding store available; and checkpoints cannot recover deleted bytes without retained accepted content/approval proof or an authorized mirror.

No preselection mechanism descendant was created because that would pre-choose Tom's decision. The carrier must create bounded implementation/operations children for every selected `P`, `M`, `K`, and `T`, plus `P+K` composition ownership when selected. The `T` child depends on the selected leaf authority (`P`, `K`, or their composition) and must prove exact leaf authentication, retention, recovery, and negative controls. Immediate research may continue; all mechanism implementation, accepted profile assembly, and authority-bound qualification remain Tom/decision/evidence blocked.

Unsupported after `P`: dishonest protected owner/host administrator, coherent history rewrite, and mixed-diff confusion. `P+T` adds witnessed history but still trusts the host and additionally trusts host-receipt authentication; it cannot make malicious host approval invalid. Unsupported after `K`: threshold/root/resolver compromise, confusing unauthoritative merged states, mixed-diff confusion, withholding, and equivocation. `M` removes only the mixed-work-item form. `P+K` still fails under coalition/correlated compromise or unchecked binding. Every `T` family still fails under approval-plus-leaf-authenticator/witness coalition, missing retained accepted content after deletion, malicious valid authorization, and denial of service.

### Checks

Run against the coherent research tree and repeated after the Outcome/review transition:

- `tkt lint` — `ok: no problems found`.
- `make citations` — exit 0; every checked local link and pinned citation resolved. On the repaired research commit the census reported 1,330 pinned citations, 7,662 resolved local links, and 69 live spike records.
- `git diff --check` — exit 0.
- `tkt guard tkt/cost-protected-review-versus-signed-conformance-authority --base 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a --format json` — the committed research diff affects exactly the three declared shared scopes `contracts/navigation`, `project/tickets`, and `research/verification`; `under_declared` is empty, `conflict` is false, and `warnings` is empty. Severity is `warn` because these additive shared scopes intersect the repository's existing shared-scope population; no exclusive collision or scope violation was reported.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
