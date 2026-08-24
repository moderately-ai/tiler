---
schema: "tiler-doc/v1"
id: "tiler.spike.verification.conformance-authority-threat-model"
kind: "experiment"
title: "Conformance authority threat model and decision packet"
topics: ["verification", "conformance", "authority", "security", "governance"]
experiment_status: "partial"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
supports: ["tiler.research.verification.conformance-authority-threat-model"]
ticket: "cost-protected-review-versus-signed-conformance-authority"
---

# Conformance authority threat model and decision packet

**Verdict.** The smallest sound first-profile authority is protected human review plus a real policy/implementation approval split, provided an actor who controls the repository host's protection or bypass authority is explicitly outside the first-profile threat boundary. That placement prevents an accidental contributor or repository-writing LLM from changing its own denominator, exceptions, verifier, oracle, or accepted evidence baseline. It does not restrain the protected policy approver, a host rules administrator or bypass actor, or an actor who can replace the clients' external trust anchor.

**Conditional frontier.** If an actor able to rewrite every repository authority and merge the result is in scope, protected review is not sound enough. The next nondominated placement is a versioned, independently signed authority manifest that transitively binds all five authority classes, combined with protected split approval. A signature over only the goal profile and exception ledger is not a survivor: the attacker can weaken the denominator enumerator, verifier, oracle authority, or evidence-baseline lineage without invalidating that signature. An externally witnessed append-only record is a third survivor only when post-publication equivocation, repository-history replacement, or signing-key misuse must be independently detectable.

**Exactly one decision for Tom.** Is an actor who can rewrite and merge every repository authority an in-scope adversary for `GoalProfileV1`?

- **No — recommended for the first profile.** Adopt protected split approval, record the excluded host-admin/authorized-maintainer threat, and trigger the signed-root ticket before the profile is used as a release/compliance claim or that threat enters scope.
- **Yes.** Require the independent signed authority manifest before `qualify` may make an accepted first-profile claim. Do not substitute signed Git commits or a signature covering only policy rows.

This is a threat-scope choice, not a request to implement either mechanism. The existing [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md) ticket is the decision carrier after the universe inventory is available.

## Exact base, bounded observations, and authority

This packet was derived at exact dispatch base `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a` on 2026-08-24. Repository observations are measurements of that commit and the named GitHub repository at that time; GitHub configuration can change without changing the commit. External mechanism descriptions are primary-source synthesis, not an implementation endorsement.

The governing project evidence is ordered as follows.

1. [`AGENTS.md`](../../../AGENTS.md) requires the Pareto decision gate, full source reading, exact-base Fact audits, subject perturbation, scope-aware dispatch, and Tom review for consequential policy or authority changes.
2. The root conformance spike states the anti-greenwashing invariant at the searchable anchor `An implementation change cannot change its own goal profile` and expressly says, at `No in-repository mechanism is claimed`, that repository-local controls cannot defend against a writer able to rewrite the profile, verifier, baseline, and tests together.
3. [`Correctness and testing`](../../../docs/correctness-and-testing.md) requires independent semantic/reference/backend comparisons and subject perturbations that prove checks can say no. A verifier and its oracle therefore cannot be treated as interchangeable policy files.
4. [ADR 0106](../../../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) says the conformance member is `Not a second semantic authority` and has `No support-matrix authority`; the protected authority cannot be invented inside that crate.
5. [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md) applies the existing fail-closed disposal of unavailable authority: `Unknown` cannot become an executable or manifest claim merely because the desired authority is inconvenient.
6. [ADR 0075](../../../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) and the coordinator rules in `AGENTS.md` establish the repository's human-review pattern, but ADR 0075 remains `implementation_status: "not-started"`; prose review policy is not itself host enforcement.

### Exact-base and live-host commands

These commands were run from the dispatched worktree before editing. Their output is intentionally summarized rather than copied as an unbounded terminal log.

```sh
git branch --show-current
git rev-parse HEAD
git status --short
git rev-list --left-right --count 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a...HEAD
tkt claims --format json
```

They reported the claimed branch `tkt/cost-protected-review-versus-signed-conformance-authority`, exact HEAD `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`, a clean tree, `0 0` divergence from the dispatch base, and a live `conformance-authority-sol` claim.

```sh
git ls-tree -r --name-only HEAD .github .githooks CODEOWNERS SECURITY.md
git config --get core.hooksPath
git config --get commit.gpgsign
git cat-file commit HEAD
git log --format='%H %G? %GK %GS' -n 20
gh api repos/moderately-ai/tiler/branches/main/protection
gh api repos/moderately-ai/tiler/rulesets
gh repo view moderately-ai/tiler --json defaultBranchRef,isPrivate,viewerPermission
```

The tracked-enforcement census returned no paths; the two Git configuration queries returned no value; the current commit object carried no `gpgsig`; all 20 bounded log rows reported Git's `N` signature status; the branch-protection endpoint returned HTTP 404 with `Branch not protected`; the ruleset endpoint returned `[]`; and the public repository reported `main` as default with the querying principal holding admin permission. The common hooks directory contained only `.sample` files and there was no active repository hook. [`Documentation metadata and traceability`](../../../docs/document-metadata.md), at the source anchor `there is no CI`, independently states the repository policy status. These observations establish the current repository/host row, not an immutable GitHub guarantee and not absence from every possible external service.

## Per-Fact audit before analysis

The ticket carries no section labelled `Fact`; its goal, work list, non-goals, and acceptance criteria are requirements. The premises those requirements depend on were re-read at the exact base and audited here so proposal text is not silently promoted into evidence.

| premise used by the ticket | verdict | exact-base evidence and consequence |
| --- | --- | --- |
| A repository-local conformance report is already protected by a repository enforcement mechanism. | **False.** | The tracked-path census found no workflow, `CODEOWNERS`, active hook, or security policy; current `main` had no branch protection or ruleset. Local `make` and `tkt` commands are voluntary checks for an actor who can rewrite or skip them. |
| The repository's ordinary process includes human review and full gates. | **Verified, but imprecise as enforcement.** | `AGENTS.md` anchors `Tom retains decisions about` and `gate → push → dispatch`; ADR 0075 describes conditional coordinator authority. These are governing process rules, not a server-side prevention mechanism. |
| `An implementation change cannot change its own goal profile, oracle, exception ledger, and evidence baseline in the same work item.` | **Verified as a proposal, not accepted policy.** | The root spike contains that exact anti-greenwashing item. No accepted ADR or host rule implements it yet. The decision ticket downstream of this work owns adoption. |
| The owner-derived universe, goal profile, evidence baseline, verifier, and oracle are separable authorities. | **Verified.** | The root spike separates owner-derived universe from goal policy and executed evidence, requires independent oracle comparison, and binds evidence to profile and oracle/reference authority. Correctness-and-testing independently rejects self-derived expected values. |
| A signature over `profile/exception` alone addresses the complete manipulation threat in this ticket. | **False; the ticket wording is materially incomplete.** | Verifier, denominator enumerator, oracle authority, and evidence-baseline lineage remain mutable while the signed policy bytes stay unchanged. A surviving external root must bind all five authority identities or a manifest/commit that transitively and unambiguously binds them. |
| The repository can restrain an actor authorized to rewrite every repository check and authority. | **False by the actor's defined power.** | Every in-repository predicate, pin, test, expected failure, and historical baseline is in the actor's write set. Only a trust anchor, reviewer/ruleset, checkpoint, or witness outside that write set can constrain or expose the rewrite. |
| Cryptography supplies review quality or remains authoritative after its signer is compromised. | **False.** | A valid signature establishes that the holder of an accepted key signed the bound bytes. It does not establish that those bytes are a good policy, and a threshold-signing attacker can authorize malicious bytes. This is also an explicit ticket non-goal. |
| Existing repository commits supply a signing authority that can be reused without design work. | **False on the bounded row.** | HEAD and the latest 20 commits were unsigned, and no local signing requirement or host signed-commit rule was configured. More importantly, a commit signature would bind repository content but would not by itself identify the policy approver, role threshold, accepted root version, recovery path, or client trust anchor. |
| An append-only log alone decides whether an authority update is legitimate. | **False.** | Transparency proves inclusion and, with monitoring/witnessing, append-only consistency. It does not judge the leaf. A malicious but valid signer can transparently log a bad authority update. |

The materially false `profile/exception` completeness premise repairs the option rather than changing the ticket's goal: the signed option below is an authority-manifest root covering denominator, exception policy, verifier, oracle authority, and baseline lineage.

## The five protected authority classes

The authority unit is not “all conformance files.” It is five versioned meanings whose concrete paths are intentionally deferred until the inventory and command contracts settle them.

| class | what must be bound | manipulation that can fake progress |
| --- | --- | --- |
| Denominator | owner-derived universe identity, owner inventories, inclusion/tombstone rules, and the enumerator/schema identity | omit a feature, silently shrink a typed population, or redirect discovery to the goal profile |
| Exception/policy | goal-profile identity, required/optional/N/A state, exception reason, approver, expiry/review trigger, and change lineage | turn a red required cell into optional/N/A or delete its tombstone |
| Verifier | `audit`/`regress`/`qualify` rule identity, report schema identity, and executable/source identity that applies those rules | accept missing receipts, collapse `Unknown` to pass, or stop checking a protected field |
| Oracle | normative reference authority, independent expected fixture identity, comparison contract, and permitted-divergence policy | recompute expectations from the implementation, widen tolerances, or replace an exact oracle with agreement |
| Evidence baseline | accepted receipt-set identity, source/profile/case/oracle/target/toolchain/environment lineage, and monotone replacement rules | bless weaker evidence as the new baseline, drop a formerly required receipt, or compare against an attacker-selected history |

**Inference.** A single authority manifest is sufficient to bind the five classes only if resolution is deterministic and complete: every bound identifier maps to one canonical byte representation, the transitive closure cannot float through branch names or mutable URLs, and unknown fields/versions fail closed. Hashes provide content identity; signatures provide approval by an accepted key; neither supplies semantics.

## Actor and power model

The actors are powers, not biographies. A person or agent can occupy more than one row; combining rows combines their powers.

| actor | powers | powers explicitly absent from this row | primary failure mode |
| --- | --- | --- | --- |
| A0 accidental contributor | proposes repository changes and runs or forgets local checks | cannot approve own protected authority change; no host bypass; no external key | omission, copy/paste error, stale baseline, unintended denominator shrink |
| A1 repository-writing LLM/automation | reads and writes any repository path on its branch, may run/modify local checks, opens work items | no independent protected approval, no ruleset bypass, no external key | reward-driven greenwashing across implementation plus local verifier |
| A2 ordinary implementation reviewer | can approve implementation changes in the role GitHub accepts; may be mistaken, inattentive, malicious, or socially engineered | is not the protected policy owner; cannot change host rules or use policy-signing key unless separately assigned | approval of bad implementation/evidence or failure to notice an authority co-change |
| A3 protected policy reviewer/maintainer | can approve and merge protected-authority work and, if configured, dismiss/reapprove reviews; this row includes any granted bypass | no independent offline signing key or witnessed checkpoint unless separately assigned | authorized policy rewrite; emergency bypass becoming normal path |
| A4 omnirepository actor | can replace denominator, policy, verifier, oracle, tests, baseline, pins, history, and all repository-local checks together | cannot alter an externally pinned reviewer rule, public key, or witnessed checkpoint unless combined with A3/A5/A6 | internally self-consistent false green and rewritten history |
| A5 external-store compromise | can withhold, delete, corrupt, or equivocate about remotely stored authority/log data | no signing threshold and no client-anchor rewrite unless separately compromised | availability loss, rollback/freeze, split view, evidence deletion |
| A6 signing-key compromise | can produce valid signatures for every role whose threshold it satisfies | cannot erase a monitored/witnessed history or replace independently pinned recovery keys unless separately compromised | malicious update that passes signature verification |

### Power interactions that must not be blurred

- A2 is not independent when the reviewer is also the implementation author or when the latest push is not reapproved. GitHub documents both stale-approval dismissal and “approve most recent reviewable push”; the safer anti-hijack setting is stale dismissal.
- A3 is an unavoidable trust point for protected review. “Do not allow bypassing” narrows ordinary bypass, but a host owner able to edit the rule remains outside the rule's own control.
- A4 defeats every local negative control even if all of them continue to pass. An external protected reviewer can restrain A4 only while A4 lacks A2/A3 authority; an external signature root can restrain A4 only while A4 lacks A6 and cannot replace the verifier's pinned root.
- A5 plus every witness/checkpoint holder can present a consistent false history or destroy availability. A log without independent monitoring or a witnessed checkpoint is not a non-equivocation authority.
- A6 defeats signature prevention for its threshold. Transparency may make misuse attributable and detectable after monitoring; it cannot make an attacker-signed malicious policy invalid merely because it is public.

## Threat and power matrix

`P` means prevents acceptance under the stated assumptions, `D` detects after or during the attempt, `A` supplies attributable evidence, `R` has a defined recovery path, and `—` supplies none. Cells name the smallest external placement capable of the property: `PR` is protected split review, `SIG` is the independently signed five-class manifest, and `LOG` is an append-only record plus an independently held witnessed checkpoint and monitor.

| attack | A0 | A1 | A2 | A3 | A4 | A5 | A6 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| shrink denominator or delete tombstone | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A if no key | SIG: P/D/A | — | LOG: D/A only |
| weaken required/optional/N/A or exception | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A if no key | SIG: P/D/A | — | LOG: D/A only |
| weaken verifier or report schema | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A only if verifier identity is bound | SIG: P/D/A only if verifier identity is bound | — | LOG: D/A only |
| replace independent oracle or tolerance | PR: P/D/A | PR: P/D/A | distinct oracle/policy approver: P/D/A | SIG: P/D/A only if oracle identity is bound | SIG: P/D/A only if oracle identity is bound | — | LOG: D/A only |
| reset evidence baseline to weaker evidence | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A only if lineage is bound | SIG: P/D/A only if lineage is bound | LOG: D/A/R from retained checkpoint | LOG: D/A only |
| rewrite repository history and all local checks coherently | protected host: P/A | protected host: P/A | distinct external approver: P/A | SIG: P/D/A | SIG: P/D/A | LOG: D/A/R | LOG: D/A only |
| withhold current authority | reviewer outage: D, fail closed | reviewer outage: D, fail closed | alternate reviewer/recovery: R | SIG cached root verifies existing claim | SIG cached root verifies existing claim | expiry/checkpoint: D; publication stops | surviving threshold/recovery keys: R |
| publish two inconsistent authority histories | PR audit only: D if observed | PR audit only: D if observed | PR audit only: D if observed | SIG authenticates both; no non-equivocation | SIG authenticates both; no non-equivocation | LOG+witness: D/A | LOG+witness: D/A |

The matrix is deliberately asymmetric. A protected maintainer who lacks the signing key is constrained by `SIG`; a reviewer or signer who is malicious within their accepted role is not made honest by tooling. Recovery is discussed separately because restoring service and deciding which state was last good are different problems.

## Candidate option analysis

### S0 — status quo: prose process plus repository-local gates

- **Prevention/detection/attribution/recovery:** catches ordinary mistakes only when contributors run the gates and reviewers apply the prose. It cannot prevent A1/A4 from editing or skipping the gate, does not provide durable approval attribution beyond ordinary Git history, and has no separate recovery anchor.
- **Cost/offline/availability:** zero new setup and fully offline. There is no new authority to become unavailable, which is also why there is no external authority to resist a coherent rewrite.
- **Unavoidable trust:** every repository writer and merger plus the correctness of all local checks.
- **Strongest counterargument:** Tiler is pre-production and a human already coordinates every landing; infrastructure cost may exceed present exposure.
- **Reversal evidence:** none can make this sound for A1/A4 because the absence is structural. Evidence could only narrow the threat model to accidental edits, which contradicts the ticket's explicit repository-writing actor.
- **Disposition:** eliminated for failing the named threat.

### S1 — path-protected human review alone

Configure an external repository ruleset/branch protection to require pull requests, protected-owner approval, stale approval dismissal, no force push/deletion, and no ordinary bypass for the eventual five authority path sets. Protect the `CODEOWNERS`/rule-definition path itself. GitHub's current documentation says protected branches can require approving or code-owner reviews, dismiss stale approvals, require the latest reviewable push, block force pushes/deletion, and disable bypass; rulesets also have explicit bypass lists. These are provider capabilities, not current repository state.

- **Prevention/detection/attribution/recovery:** prevents A0/A1 from landing protected changes without review and attributes approval through the host. It does not prevent a single approved work item from weakening policy and implementation together, and does not constrain A2/A3 or a host administrator who changes/bypasses the rule. Host history helps diagnose ordinary bypass but is not an independent root against host compromise.
- **Cost/offline/availability:** one-time ruleset and ownership configuration, then one online protected review per authority-touching change. Offline development remains possible; accepted merge/qualification waits for host access and a reviewer. No Tiler runtime or kernel cost.
- **Recovery/rotation:** rotate code-owner teams and reviewer accounts in the provider; emergency bypass must be narrowly held, logged, and followed by independent review. If every protected reviewer is unavailable, authority changes stop; old accepted profiles remain usable.
- **Strongest counterargument:** a careful code-owner review already sees the combined diff, so forbidding co-change may add latency without adding a new person.
- **Reversal evidence:** a subject perturbation showing one protected reviewer reliably rejects implementation-plus-policy greenwashing across representative diffs could reduce the value of split approval, but cannot make the same reviewer independent of their own compromise.
- **Disposition:** eliminated as a standalone option because it does not enforce the root spike's “implementation cannot change its own authority” invariant.

### S2 — split policy/exception approval without path protection

Declare that policy/exception and implementation changes use different work items and approvers, but store and enforce that declaration only in the repository.

- **Prevention/detection/attribution/recovery:** useful review discipline for A0, but A1/A4 can change or ignore the discipline. A2 remains a trust point; distinct approvers reduce single-reviewer compromise only if the roles are enforced externally.
- **Cost/offline/availability:** two work items or approval lanes and additional human latency, no host/runtime cost. Fully offline as prose; therefore not enforceable against the named repository writer.
- **Strongest counterargument:** no platform setup, and ticketsplease already models separate work.
- **Reversal evidence:** an external workflow that enforces the split would turn this into the combined survivor below, not rescue this option as stated.
- **Disposition:** eliminated as weaker than the same split under protected review.

### S3 — independent signed authority manifest alone

Clients pin an external public root and accept only a monotonically versioned canonical manifest signed by the required role threshold. The manifest binds the exact identities/digests of the five classes, the schema and resolver version, role/threshold metadata, validity/expiry policy, and predecessor/root version. A TUF-style root is the reference design vocabulary, not a proposal to import TUF wholesale: TUF specifies offline root keys, threshold signatures, versioned root continuity signed by old and new thresholds, rollback/freeze checks, and out-of-band recovery if a root threshold is compromised.

- **Prevention/detection/attribution/recovery:** prevents A0–A4 from authorizing changed bytes while they lack the threshold keys, even if they rewrite the repository. It authenticates signer approval but does not supply review quality, separation of duties, or non-equivocation; A6 can sign a malicious manifest, and two valid histories can exist without an external checkpoint.
- **Cost/offline/availability:** canonicalization, manifest/resolver schema, client pinning, key ceremony/storage, signer tooling, verifier integration, rotation drills, and recovery documentation. Existing signed manifests can verify offline from cached roots; new authority publication requires threshold keys. No kernel cost; audit/qualification adds fixed manifest parsing plus digest/signature verification, not measured here.
- **Recovery/rotation:** normal rotation uses a versioned successor authorized by both old and new thresholds. A compromised sub-threshold key is revoked by a surviving threshold. A compromised threshold requires an out-of-band client root update and may make safe recovery impossible for unattended clients; a lost threshold makes new publication unavailable.
- **Strongest counterargument:** it is disproportionate for one pre-production maintainer and can create a fragile key-recovery ceremony before the five concrete identities exist.
- **Reversal evidence:** the universe/command design may show no canonical transitive binding is possible without freezing premature schemas; that would defer signatures. Conversely, a release/compliance consumer that must validate after repository compromise makes this cost necessary.
- **Disposition:** eliminated alone because a signer can approve policy and implementation without independent review; survives only in combination.

### S4 — append-only external transparency record alone

Publish authority updates to a cryptographically verifiable append-only log. The useful security property requires inclusion proofs, consistency monitoring, and an independently distributed or witnessed checkpoint; an inclusion promise from the same store does not prevent a split view. Sigstore's security documentation likewise says long-term trust requires monitoring and that transparency supplies auditability, while witnesses/monitors establish consistent append-only views rather than judging leaf legitimacy.

- **Prevention/detection/attribution/recovery:** detects deletion, rollback, or equivocation after a trusted checkpoint and makes signer activity auditable. It does not prevent or invalidate a bad but logged update, does not establish policy quality, and without a signed leaf can attribute only the publisher/log, not the policy approver.
- **Cost/offline/availability:** external service or self-hosted log, monitor/witness, checkpoint distribution, retention and incident response, network publication, and a client inclusion/consistency path. Cached signed leaf plus inclusion bundle/checkpoint can verify offline; new publication should remain pending when the log is unavailable rather than silently omitting transparency.
- **Recovery/rotation:** reconstruct from retained signed leaves and independently held checkpoints, rotate log/witness keys through an already trusted root, and quarantine the compromised log. Compromise of the log plus every witness/checkpoint holder defeats non-equivocation.
- **Strongest counterargument:** a public log adds privacy, availability, and operations burden while doing nothing to stop a valid signer from making a bad decision.
- **Reversal evidence:** multiple independent consumers requiring durable public audit, or observed equivocation/history rewrite, would justify it; a single private pre-production profile does not.
- **Disposition:** eliminated alone and deferred as an add-on until its detection benefit is in scope.

### R — protected review plus split policy/implementation approval

`R = S1 + an externally enforced S2`. The policy lane owns denominator inclusion/tombstones, requirement/exception state, verifier acceptance semantics/schema, oracle authority/tolerances, and evidence-baseline admission. The implementation lane may supply new behavior and receipts but cannot weaken those authorities. At least one policy approver must be distinct from the latest implementation push; to resist a malicious A2 rather than only mistakes, policy and implementation require distinct human identities.

- **Prevention/detection/attribution/recovery:** prevents A0/A1 and an implementation reviewer from self-authorizing greenwashing; host review records attribute the approval. It does not constrain A3 or a compromised host administrator and provides no offline proof after host/history replacement.
- **Cost/offline/availability:** smallest setup and cognitive cost among sound options for the A0/A1 boundary. One extra review lane/work item on authority changes; normal implementation not touching authorities pays only the implementation review. Local `audit`/`regress` can run offline against the last accepted authority; no new accepted profile or baseline may be minted while the policy authority is unavailable.
- **Recovery/rotation:** provider-managed reviewer/team rotation; previous accepted authority remains usable; no authority update during total reviewer outage. Emergency bypass does not create accepted conformance authority until independently post-reviewed.
- **Unavoidable trust:** protected policy reviewer, host owner/rules configuration, implementation reviewer for their lane, and the reviewer's ability to judge semantics.
- **Strongest counterargument:** it does not survive the exact coherent rewrite actor the root spike warned about if that actor can also merge or administer protections.
- **Reversal evidence:** any requirement to verify an accepted profile after repository/host compromise, any additional untrusted merger, release/compliance use, or a demonstrated protected-rule bypass moves the first profile to `K`.

### K — protected split review plus independent signed five-class manifest

`K = R + S3`, with the external policy signer distinct from ordinary repository-writing automation. The signer signs a reviewed canonical manifest; the signature does not replace review. A threshold design is strongly preferred if the protected-maintainer threat is genuinely in scope, because a one-key system simply moves all authority to one secret.

- **Prevention/detection/attribution/recovery:** adds prevention against A3/A4 repository rewrites while the signer threshold and client anchor survive. It attributes manifest authorization to a key/role. It does not detect two signer-authorized histories without a checkpoint and cannot protect a compromised signing threshold.
- **Cost/offline/availability:** all `R` cost plus schema/canonicalization, pinning, keys, verifier work, rotation and recovery drills. Existing accepted manifests verify offline. New qualification authority is unavailable when threshold keys are unavailable; this fails closed rather than falling back to repository HEAD or an unsigned profile.
- **Unavoidable trust:** policy reviewers, signer threshold, custody/recovery root, client anchor distribution, canonical resolver, and host for availability.
- **Strongest counterargument:** key loss or premature schema binding can stall a research project more reliably than the threat it mitigates.
- **Reversal evidence:** if Tom puts A3/A4 inside `GoalProfileV1` scope, or an external consumer must validate independently of GitHub, this option becomes necessary rather than optional. If the universe work cannot yet name stable five-class identities, bounded schema research must finish before adoption.

### T — protected split review, signed manifest, and witnessed transparency

`T = K + S4`. Publish every accepted manifest, rotation, revocation, and recovery event; require a witnessed checkpoint/inclusion bundle before the update becomes an accepted qualification authority.

- **Prevention/detection/attribution/recovery:** preserves `K` prevention and adds durable detection/attribution for rollback, deletion, split histories, and visible key misuse after monitoring. Recovery can identify a witnessed last-good state. It still cannot make malicious reviewed/signed content correct and fails if the signer threshold plus log/witness trust set are compromised.
- **Cost/offline/availability:** highest operational and external availability cost. Existing bundles verify offline; new authority publication blocks while required log/witness quorum is unavailable. The log adds no Tiler runtime/kernel cost but adds network and monitoring cost to authority publication and qualification.
- **Unavoidable trust:** everything in `K` plus log/witness keys, checkpoint distribution, monitor operation, and incident response.
- **Strongest counterargument:** it creates a small supply-chain service for a pre-production, single-repository policy whose consumers do not yet exist.
- **Reversal evidence:** multiple independent consumers, public release attestations, witnessed signer misuse, a need for non-equivocation, or repository history replacement makes the add-on valuable.

### B — bounded research, and D — deferral

Further bounded research is a work state, not an authority placement. It is sound only while no accepted `qualify` claim is emitted. The useful bounded questions are the five-class canonical manifest closure, concrete protected paths after the universe inventory, and a dry-run cost/recovery exercise; all are already owned or naturally descend from the existing decision/design graph. Deferral is likewise sound only if the first profile remains explicitly unaccepted and every unavailable-authority result is `Unknown`/nonzero rather than green.

- **Strongest counterargument:** delaying authority while implementation proceeds creates facts and file layouts that will pressure the later policy to bless the status quo.
- **Reversal evidence:** the inventory and command-contract tickets producing stable identities and a minimal protected path set ends the reason to defer.
- **Disposition:** available as a fail-closed scheduling state, not a frontier authority for an accepted profile.

## Eliminated options and Pareto frontier

The decision dimensions are correctness, fail-closed strictness, long-term maintainability/compatibility, Tiler host/runtime cost, operational/recovery cost, and the protected threat population. Kernel performance is unaffected by all candidates because authority verification is outside kernel execution.

| candidate | reason eliminated or retained |
| --- | --- |
| S0 local status quo | **Eliminated:** A1/A4 can rewrite or skip every control. Lowest cost does not compensate for failing the named correctness threat. |
| S1 protected review alone | **Eliminated:** one combined work item can weaken authority and implementation together; strictly weaker than `R` on separation with only incremental review-lane cost. |
| S2 unprotected split | **Eliminated:** repository writer can erase the split; dominated by `R`. |
| profile/exception-only signature | **Eliminated:** verifier/oracle/denominator/baseline bypass keeps signature valid. Silently incomplete protection is a correctness defect. |
| S3 signed manifest alone | **Eliminated:** authenticates a signer but supplies no independent review/separation; `K` adds required judgment control. |
| S4 log alone | **Eliminated:** records bad leaves as faithfully as good ones and cannot authorize policy. |
| R protected split review | **Frontier for A0/A1/A2 accidental or single-lane compromise:** minimal host/operations cost; does not cover A3/A4 with merge/admin power. |
| K R + independent signed manifest | **Frontier when repository/host rewrite is in scope:** higher key/schema/recovery cost; lacks non-equivocation and signer-misuse history. |
| T K + witnessed transparency | **Frontier when non-equivocation/post-compromise public audit is in scope:** greatest detection/recovery and greatest external service/availability cost. |
| bounded research/deferral | **Not an accepted-authority candidate:** safe only while qualification remains unavailable and reports `Unknown`/nonzero. |

No one frontier option dominates across threat coverage and operations. Within the recommended first-profile boundary, `R` dominates stronger placements because `K` and `T` add no protection against the in-scope A0/A1 actors that `R` does not already supply, while adding material key/schema/service recovery cost. Once A3/A4 is in scope, `R` is no longer correctness-tier and `K` becomes the smallest survivor. `T` remains nondominated only when equivocation/history-rewrite detection is required.

## Cost, offline, unavailability, and recovery comparison

No wall-time or monetary cost was measured because no mechanism was installed and provider/key choices are undecided. The table therefore reports operations and asymptotic work, not fabricated timing.

| property | R protected split | K signed root combination | T transparency combination |
| --- | --- | --- | --- |
| one-time setup | ruleset/protected paths, protected ownership, two approval roles, bypass policy | `R` plus canonical five-class manifest, client root pin, signing roles/thresholds, key ceremony, recovery drill | `K` plus log, inclusion/consistency verification, monitor/witness, checkpoint distribution, incident runbook |
| routine authority update | separate policy work item and protected approval; implementation evidence cannot co-change authority | reviewed manifest plus threshold signing and repository publication | signed publication plus log inclusion and witnessed checkpoint |
| repository size | ownership/policy text and host config reference | manifest, signatures, roots, rotations; bounded by number of authority versions retained | inclusion bundles/checkpoints or references; history grows with updates |
| Tiler host cost | review-time only; audit reads accepted local state | fixed canonical parse/digests/signature verification during audit/qualify; unmeasured | `K` plus inclusion/checkpoint verification and optional network fetch; unmeasured |
| runtime/kernel cost | none | none if authority is checked before build/qualification | none if authority is checked before build/qualification |
| offline development | full; cannot accept an authority change offline | full against cached root/manifest; cannot mint a new accepted manifest without threshold | full against cached inclusion bundle/checkpoint; cannot publish a new accepted version offline |
| authority unavailable | block authority-changing merge/qualification; keep last accepted version | verify last accepted version; block new publication, rotation, or qualification identity | verify last accepted bundle; block new accepted publication until required inclusion/witness proof exists |
| reviewer recovery | rotate protected team/account through host owner; post-review emergency bypass | same | same |
| key loss/compromise | not applicable; host credentials remain the keys in practice | rotate below threshold; old+new threshold continuity; out-of-band recovery after threshold compromise | same, plus log revocation/rotation event and monitor alert |
| external store loss/compromise | host outage blocks merge; local clones retain history but no independent authority | cached signed roots/manifests remain verifiable; store can freeze but not forge without key | reconstruct from signed leaves and independent checkpoints; quarantine log; all-witness compromise remains fatal to non-equivocation |

### Fail-closed availability rules

1. Unavailable protected reviewer: ordinary research/implementation may continue on branches, but no policy/exception/oracle/verifier/baseline update becomes accepted and `qualify` cannot cite it.
2. Unavailable signing threshold: the last unexpired/policy-valid cached authority can still verify; no unsigned or repository-default replacement is accepted. If expiry is adopted, expiry yields a named unavailable/unknown authority, not automatic extension.
3. Unavailable transparency service: if `T` is required, a new manifest remains pending until an inclusion proof and required witnessed checkpoint exist. Existing offline bundles remain verifiable under their stated validity policy.
4. Recovery bypass: recovery changes the root of trust; it cannot be treated as an ordinary implementation merge. An emergency repository merge without restored authority may repair code but cannot mint a conformance-green claim.

## Independent subject perturbations

These are required tests for the eventual mechanism; none was implemented in this research ticket. Each perturbation changes the protected subject, not the assertion, and each expected message names the independent property that must reject. The downstream [perturbation-suite ticket](../../../tickets/design-the-conformance-denominator-and-receipt-perturbation-suite.md) already owns executable realization after the authority decision and command contracts.

| protected property | subject perturbation | expected `R` failure | additional `K`/`T` failure |
| --- | --- | --- | --- |
| denominator closure | add an owner-registered feature without a universe disposition, then separately delete an existing tombstone | protected policy-path review required; `audit` names missing/disappeared key | manifest digest/identity mismatch; `T` also lacks accepted inclusion for successor |
| exception strictness | change one required cell to optional/N/A and separately remove its reason/expiry | policy lane rejects implementation co-change; `regress` names authority change rather than progress | signature failure until reviewed successor; successor version must advance |
| verifier strictness | change `qualify` to treat missing/Unknown receipt as success | verifier path requires policy approval; negative case must report the exact missing obligation | bound verifier identity differs even when profile bytes do not |
| oracle independence | replace a hand-written expected result with output recomputed by the implementation, then separately widen a tolerance | oracle path requires independent policy approval; oracle perturbation must fail expected comparison | bound oracle/tolerance identity differs while implementation identity remains unchanged |
| evidence monotonicity | replace the accepted baseline with a receipt set missing one formerly sufficient case | baseline admission cannot share the implementation work item; `regress` prints the lost cell | lineage/digest and successor version differ; old signed baseline remains recoverable |
| latest-push review | obtain approval, then push a protected-authority edit | stale approval is dismissed and merge is blocked | new content also lacks the accepted manifest signature |
| repository rewrite | change all five local authorities and every local check so the suite reports green | `R` fails only if host protection/approver is outside attacker power; this is the explicit limit | pinned root rejects the rewritten unsigned/untrusted manifest |
| rollback/freeze | restore an older valid signed authority version | `R` may miss it after history rewrite | monotone root/version rejects rollback; expiry or freshness policy reports freeze |
| signing-key compromise | sign a malicious five-class manifest with the accepted threshold | `R` may still reject if independent review remains uncompromised | `K` signature checks pass; `T` records/attributes the event but does not prevent it |
| log equivocation | present two valid inclusion histories to separate clients | no dedicated result | independent witness/checkpoint consistency fails; a lone log without witnesses may pass both views |
| authority outage | disconnect host, remove reviewer availability, withhold signer, or withhold log in separate trials | no accepted authority-changing merge | existing cached authority verifies; new accepted publication fails with a typed unavailable-authority result |

Each independent property needs its own perturbation and captured failure text. A single patch that changes all five subjects proves only that something is guarded. Counts must come from the concrete typed registries after the inventory work, not from a hand-written expected total.

## Recovery cases and unavoidable trust

### Reviewer or host-account compromise

Revoke the account/team membership, remove bypass, invalidate pending approvals by requiring fresh review after the latest push, inspect provider audit history, and re-review every authority version approved during the exposure window. The last known-good authority must be identified independently of the compromised reviewer's assertion. `R` has no external cryptographic answer if the same host/history was rewritten; that is its accepted threat limit.

### Signing-key loss below threshold

Use surviving authorized keys to publish a versioned successor removing the lost key and adding a replacement. A TUF-style transition is signed by both the old and new accepted thresholds so older clients can verify continuity. Test this ceremony before depending on it; a policy that has never demonstrated recovery has an unknown recovery property.

### Signing threshold compromise

Assume malicious manifests may have been validly signed. Stop new qualifications, identify a last-good version through independent review and any witnessed record, distribute a new root out of band, and re-qualify descendants. TUF's specification warns that recovery after threshold root compromise is nearly impossible for already affected unattended clients; this packet therefore does not claim automatic recovery.

### External store or log compromise

Withhold/freeze is an availability failure; rollback is detected only relative to client-retained version/freshness state; equivocation is detected only relative to independent monitors/witnesses/checkpoints. Restore service from signed manifests plus checkpoints held outside the compromised store. If the attacker also controls every checkpoint/witness and the clients' root distribution, no mechanism in this frontier supplies recovery.

### Unavoidable trust statement

Every option ends at some authority. `R` trusts human judgment and host access control. `K` additionally trusts canonical resolution, a signer threshold, custody/recovery, and the clients' pinned root. `T` additionally trusts monitors/witnesses and checkpoint distribution. None protects against a coalition controlling all of its terminal authorities, and none establishes semantic correctness without the independent conformance oracles required by the correctness contract.

## External-authority limit

This limit is normative for interpreting the packet: a repository-local check can constrain only actors who cannot rewrite, replace, or skip it. If an actor can rewrite every repository authority and the expected results together, a green repository report is merely an attacker-chosen fixed point. An external rule or key helps only while the attacker cannot also change the rule, exercise its bypass, satisfy its signing threshold, replace the client's trust anchor, or control every transparency witness. Claims beyond that boundary are unsupported, not “best effort.”

## Follow-up graph: nothing implicit

No new descendant ticket is required. The existing conformance-progress graph already separates every necessary next step, so duplicating it would create competing authorities.

| work | existing owner and dependency | immediate or blocked |
| --- | --- | --- |
| enumerate concrete universe/profile authority objects and protected populations | [`inventory-the-closed-world-conformance-claim-universe-by-owner`](../../../tickets/inventory-the-closed-world-conformance-claim-universe-by-owner.md) | **Immediate research; in progress.** Supplies the path/identity population this packet deliberately does not invent. |
| choose `R` versus `K` and define who may approve authority changes | [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md), depending on this ticket and the inventory | **Tom decision/evidence blocked.** Must answer the one threat-scope question above. |
| define fail-closed `audit`/`regress`/`qualify` semantics and machine failures | [`design-the-conformance-audit-regress-and-qualify-command-contracts`](../../../tickets/design-the-conformance-audit-regress-and-qualify-command-contracts.md), dependent on the authority decision and receipt/universe contracts | **Decision blocked.** Owns verifier behavior, not this threat model. |
| bind identities for signed manifest if `K` is selected | authority decision ticket must create a bounded signature-schema/recovery child after the five concrete identities exist | **Tom/evidence blocked and not authorized now.** No key, schema, signing code, or host configuration belongs in profile implementation. |
| exercise each subject perturbation and authority outage | [`design-the-conformance-denominator-and-receipt-perturbation-suite`](../../../tickets/design-the-conformance-denominator-and-receipt-perturbation-suite.md) | **Contract blocked.** Depends on authority, receipts, profile, and command contracts. |
| assemble `GoalProfileV1` | [`assemble-the-first-versioned-conformance-goal-profile`](../../../tickets/assemble-the-first-versioned-conformance-goal-profile.md) | **Blocked.** Must consume the accepted authority rather than choose it. |
| render authority and evidence without hiding Unknown | [`design-the-machine-readable-and-explorable-conformance-report`](../../../tickets/design-the-machine-readable-and-explorable-conformance-report.md) | **Blocked.** Report is a projection, never the authority. |

Immediate work can continue on inventory, obligation algebra, evidence semantics, and other already-dispatched research. Host protection, signing, transparency, and accepted profile assembly remain Tom/decision/evidence blocked.

## Recommendation, adoption trigger, and unsupported threats

**Proposal.** For `GoalProfileV1`, adopt `R` after the downstream decision accepts the concrete included/excluded authority paths and roles: protected pull-request review, stale approval dismissal, no routine bypass, protected ownership of the rule/ownership file, and distinct policy/implementation approval lanes. Treat unavailable authority as `Unknown`/nonzero. Do not call this protection against a protected maintainer or host administrator.

**Trigger for `K`.** Before the first external release/compliance claim, before an external consumer must validate independently of GitHub, when more than one actor may merge protected authority, when a protected rules bypass/admin compromise enters scope, or after any observed coherent history/authority rewrite, require a bounded signed-manifest design and recovery exercise. The manifest must bind all five authority classes and client-root continuity.

**Trigger for `T`.** Add witnessed transparency only when multiple independent consumers need non-equivocation, public audit of authority changes, durable detection of signer misuse, or recovery from repository-history replacement. A log without monitoring/witnessing does not satisfy the trigger.

**Unsupported threats after `R`:** malicious/compromised policy reviewer, protected maintainer or host admin, repository host compromise with rules/history rewrite, and offline verification after that rewrite.

**Unsupported threats after `K`:** compromised signer threshold, compromised/out-of-band client root distribution, malicious policy reviewers plus signer threshold, canonicalization/resolver bugs, and two valid signed histories without a checkpoint.

**Unsupported threats after `T`:** coalition controlling signer threshold plus every required witness/checkpoint path, compromised client root distribution, malicious but validly reviewed/signed/logged semantics, and denial of service.

## Primary sources and what they establish

- [GitHub, About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches) — required pull-request/code-owner review, stale approval dismissal, latest-push approval, signed-commit option, force-push/deletion controls, and the provider's bypass model.
- [GitHub, About code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners) — protected-owner review and the need to protect `CODEOWNERS` itself.
- [GitHub, Creating rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository) — explicit bypass actors and ruleset operation.
- [The Update Framework specification v1.0.27](https://theupdateframework.github.io/specification/v1.0.27/) — root/role thresholds, offline root-key recommendation, versioned old-plus-new root continuity, rollback/freeze handling, rotation, and the out-of-band limit after threshold compromise.
- [Sigstore security model](https://docs.sigstore.dev/about/security/) — append-only transparency, signed tree state, inclusion/auditability, and the requirement for monitoring for long-term trust.
- [Sigstore threat model](https://docs.sigstore.dev/about/threat-model/) — monitors checking append-only behavior and cross-monitor consistency, plus TUF-root recovery for service compromise.
- [Transparency.dev, witness network](https://blog.transparency.dev/can-i-get-a-witness-network) — witnesses validate consistency and help detect split views; they do not validate leaf legitimacy.

No access control, paywall, or inaccessible primary source narrowed this packet.
