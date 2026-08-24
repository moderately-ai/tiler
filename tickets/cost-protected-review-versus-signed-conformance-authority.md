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
lease_expires_at: 1787604109
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

Initial research content commit: `ac99577facf54528f674eb3011c1d644c5ee6140`. Mixed-diff review repair: `cdb9adc381a855dc3d7d3f01653f703d04e04902`. Signed-authority Pareto repair: `18ab839e293056e6a2bf3ec3ae503f7cc8871e60`.

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
- **False on the bounded current row:** existing Git signatures provide a reusable conformance authority. HEAD and the latest 20 commits were unsigned, and there was no configured signing rule; commit signing would still need role, threshold, root, recovery, and protected-object design.

### Independent-review corrections, frontier, and recommendation

The first independent review found that the packet credited its protected split with an enforcement property it had not defined or costed. The second found that it conflated automated signing with policy-approver-held signing and incorrectly made `P` mandatory for every sound signed placement. The repaired packet exposes both errors and rebuilds the conditional frontier.

- `P`: protected combined human-owner review — the smallest current recommendation for the first profile while the protected policy owner and rules/bypass administrator are trusted. It permits combined authority/implementation diffs and relies on owner judgment.
- `K`: policy-approver-held threshold signing of a canonical, versioned five-class manifest — independently sound for A0–A4 and client-side repository-rewrite resistance while A6/root/resolver compromise is excluded. It needs no protected-review machinery but pays key/schema/recovery cost and may let unauthoritative repository content merge before authority-aware refusal.
- `P+K`: both independently required — defense in depth and better premerge feedback, not a prerequisite for `K`. Its A3-only/A6-only tolerance requires independent administration and a checked binding between the exact host-approved source and signed manifest.
- `M`: optional external mixed-diff classifier/attestation, producing `P+M`, `K+M`, or `P+K+M`. It adds mechanical work-item separation but not protection from a dishonest selected policy approver using separate changes.
- `T`: optional witnessed transparency over a `K` family, producing `K+T`, `K+M+T`, `P+K+T`, or `P+K+M+T`. It adds non-equivocation/history evidence; recovery after deletion additionally requires retained signed content or an authorized mirror.

The packet eliminates the local status quo, unenforced prose split, `M` alone, profile/exception-only signatures, automated/unreviewed signing, and transparency alone. It prepares ordered Tom decisions, asked one at a time: (1) must the profile resist protected-policy/host rewrite — no selects `P`, yes requires `K`; (2) if `K` is selected, must `P` also be a required acceptance condition — no selects standalone `K`, yes selects independently bound `P+K`; (3) must mixed work items be mechanically rejected — yes adds `M`; and (4) does the profile require witnessed non-equivocation/public audit/last-good history — yes adds `T`. The current narrow recommendation remains `P`, with no `M` or `T`, until stronger threat or policy requirements are accepted.

External-store compromise is analyzed as separate corruption/substitution, rollback, freeze, withholding, and equivocation powers. A signed manifest rejects corrupted bytes and retained-state rollback, but freeze detection needs an accepted freshness boundary and trusted time; signatures do not make a withholding store available; and witnessed checkpoints detect/attribute split or deleted histories but cannot recover deleted bytes without independently retained signed content or an authorized mirror.

No preselection mechanism descendant was created because that would pre-choose Tom's decision. The decision carrier is now explicitly required to create bounded implementation/operations children for every selected `P`, `M`, `K`, and `T` property. The packet states each child's dependencies, authority, non-goals, negative controls, recovery evidence, and stop conditions, plus an exact `P+K` composition owner when both are selected. Immediate research may continue; all mechanism implementation, accepted profile assembly, and authority-bound qualification remain Tom/decision/evidence blocked.

Unsupported after `P`: dishonest protected owner/host administrator, coherent history rewrite, mixed-diff confusion, and independent offline validation after rewrite. Unsupported after `K`: dishonest/compromised threshold, client-root or resolver compromise, merged unauthoritative states confusing non-aware users, mixed-diff signer confusion, withholding, and equivocation. `M` removes only the mixed-work-item form. `P+K` still fails under coalition/correlated authority compromise or an unchecked composition binding. Any `K+T` family still fails under approval-plus-witness coalition, missing retained signed content after deletion, malicious but validly authorized/logged semantics, and denial of service.

### Checks

Run against the coherent research tree and repeated after the Outcome/review transition:

- `tkt lint` — `ok: no problems found`.
- `make citations` — exit 0; every checked local link and pinned citation resolved. On the repaired research commit the census reported 1,330 pinned citations, 7,662 resolved local links, and 69 live spike records.
- `git diff --check` — exit 0.
- `tkt guard tkt/cost-protected-review-versus-signed-conformance-authority --base 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a --format json` — the committed research diff affects exactly the three declared shared scopes `contracts/navigation`, `project/tickets`, and `research/verification`; `under_declared` is empty, `conflict` is false, and `warnings` is empty. Severity is `warn` because these additive shared scopes intersect the repository's existing shared-scope population; no exclusive collision or scope violation was reported.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
