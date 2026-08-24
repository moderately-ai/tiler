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
lease_expires_at: 1787601327
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

Research content commit: `ac99577facf54528f674eb3011c1d644c5ee6140`.

The retained [conformance authority threat model and decision packet](../spikes/verification/conformance-authority-threat-model/README.md) covers denominator, exception/profile, verifier, oracle, and evidence-baseline manipulation. The [research record](../docs/research/verification/conformance-authority-threat-model.md) supplies the governed traceability edge, and both hand-maintained catalogs carry the new records.

### Exact-base Fact audit

- **False:** the current repository/local report already has an enforcement boundary. At `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a` there was no tracked workflow, `CODEOWNERS`, active hook, or signing policy; GitHub reported no protection on `main` and no ruleset.
- **Verified but imprecise as enforcement:** `AGENTS.md` and ADR 0075 require human review and gates, but those prose rules are not a server-side prevention mechanism.
- **Verified as a proposal, not accepted policy:** the root spike says an implementation change cannot change its own profile, oracle, exception ledger, and baseline in the same work item. The downstream authority ticket still owns adoption.
- **Verified:** owner-derived universe, profile/exception policy, verifier, independent oracle, and evidence-baseline lineage are distinct authority classes.
- **False/materially incomplete:** signing only the profile/exception root covers this ticket's threat. A writer can weaken the denominator enumerator, verifier, oracle authority, or baseline lineage without invalidating such a signature. A surviving signed manifest must bind all five classes.
- **False by the actor's defined power:** repository-local checks can restrain an actor authorized to rewrite all repository authorities together.
- **False:** cryptography supplies review quality, protects a compromised signing threshold, or makes a logged leaf legitimate.
- **False on the bounded current row:** existing Git signatures provide a reusable conformance authority. HEAD and the latest 20 commits were unsigned, and there was no configured signing rule; commit signing would still need role, threshold, root, recovery, and protected-object design.

### Frontier and recommendation

The packet eliminates local status quo, protected review without a split, an unprotected prose split, a profile/exception-only signature, signature without independent review, and a log without review/signature. Three conditional frontier placements remain:

1. `R`: protected review plus externally enforced, distinct policy/implementation approval lanes — smallest and recommended for the first profile if protected policy maintainer and host-admin compromise are explicitly out of scope.
2. `K`: `R` plus an independently signed, versioned manifest binding all five authority classes — smallest survivor when an actor able to rewrite and merge every repository authority is in scope.
3. `T`: `K` plus an independently witnessed append-only record and monitoring — survivor only when non-equivocation, durable public audit, or post-compromise last-good recovery is required.

The packet prepares exactly one Tom decision for the downstream authority ticket: **is an actor who can rewrite and merge every repository authority an in-scope adversary for `GoalProfileV1`?** “No” selects `R` and records the stronger-authority trigger; “yes” requires `K` before an accepted `qualify` claim. `T` has a later evidence trigger and is not part of this first decision.

No descendant was created. The existing conformance-progress graph already owns the universe inventory, authority decision, command contracts, subject perturbations, first profile, and report projection. Duplicating those tickets would create competing authorities. Immediate research may continue; protection/signing/transparency implementation, accepted profile assembly, and authority-bound qualification remain Tom/decision/evidence blocked.

Unsupported after `R`: malicious/compromised protected policy reviewer, host rules administrator/bypass, repository-host history rewrite, and offline verification after that rewrite. Unsupported after `K`: compromised signer threshold or client root distribution, canonical resolver bugs, and two valid signed histories without a checkpoint. Unsupported after `T`: coalition compromise of signer threshold plus every required witness/checkpoint path, malicious but validly reviewed/signed/logged semantics, and denial of service.

### Checks

Run against the coherent research tree and repeated after the Outcome/review transition:

- `tkt lint` — `ok: no problems found`.
- `make citations` — exit 0; every checked local link and pinned citation resolved. On the research commit the census reported 1,330 pinned citations, 7,660 resolved local links, and 69 live spike records.
- `git diff --check` — exit 0.
- `tkt guard tkt/cost-protected-review-versus-signed-conformance-authority --base 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a --format json` — the committed research diff affects exactly declared `contracts/navigation`, `project/tickets`, and `research/verification`; `under_declared` is empty, `conflict` is false, and `warnings` is empty. Severity is `warn` because the two additive shared scopes intersect the repository's existing shared-scope population; no exclusive collision or scope violation was reported.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
