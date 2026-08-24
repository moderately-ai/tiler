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

**Corrected verdict after the complete Pareto audit.** The smallest sound first-profile authority for accidental contributors, repository-writing automation, and an ordinary implementation reviewer is protected combined review: every change touching a protected authority class requires fresh approval from an honest protected policy owner. A separate-work-item rule adds no enforcement by itself, and GitHub `CODEOWNERS`/branch protection does not natively reject a pull request merely because it mixes authority and implementation paths. Three review corrections are preserved: the split requires its own trusted enforcement; policy-approver-held signing is a standalone approval authority; and witnessed transparency may record either an authenticated `P`-approved leaf or a `K`-approved leaf rather than requiring `K`.

**Conditional frontier.** Choose exactly one base approval placement from `P`, `K`, or independently required `P+K`, then independently choose whether to add `M` and `T`. `P` is host-protected policy-owner review. `K` is a policy-approver-held threshold signature over a versioned manifest binding all five authority classes. `M` is a trusted external mixed-diff predicate bound to the chosen acceptance path. `T` requires witnessed append-only publication of an authenticated accepted-authority leaf: for `P`, the exact commit/content plus an authenticated host-approval receipt; for `K`, the signed five-class manifest; for `P+K`, the exact binding of both. This produces twelve conditional survivors: `P`, `P+M`, `P+T`, `P+M+T`; `K`, `K+M`, `K+T`, `K+M+T`; and `P+K`, `P+K+M`, `P+K+T`, `P+K+M+T`.

**Recommendation under current evidence.** Start with `P` while explicitly excluding a dishonest protected policy owner and rules administrator/bypass actor. It is the lowest operational-cost placement that prevents A0/A1/A2 from self-authorizing an authority rewrite and rejects unapproved content before merge. `K` remains the smallest base when protected-host rewrite is in scope; `P+K` is defense in depth, not a prerequisite. Add `M` only for mechanical mixed-diff prohibition. Add `T` to whichever base was selected when non-equivocation or witnessed history enters scope: `P+T` avoids `K` policy-key/manifest machinery but requires an authenticated host-approval receipt and trusts the host policy authority, while `K+T` carries the independent signing root.

**Ordered decisions for Tom, asked one at a time in the downstream carrier.**

1. Must `GoalProfileV1` resist a protected-policy/host authority that can rewrite and merge every repository authority? **No — current recommendation** selects `P`; **yes** requires a `K` family before an accepted `qualify` claim.
2. If `K` is required, must `P` also be a required acceptance condition? **No** selects standalone `K`; **yes** selects `P+K`. “Yes” is justified only if tolerance of signing-threshold compromise or host-side rejection is worth the second authority and outage path. The controls must be independently administered, and acceptance must verify the exact host-approved source identity, for the compromise-tolerance claim.
3. Within the selected approval placement, must every mixed authority-plus-implementation/evidence work item be mechanically rejected? **No — recommended absent contrary evidence** relies on the selected policy approver's judgment; **yes** adds `M` and its external classifier/attestation, taxonomy, availability, and recovery burden.
4. Does the first profile require independently witnessed non-equivocation, public audit, or last-good history? **No** defers `T`; **yes** adds it to the already selected `P`, `K`, or `P+K` base without changing that base. A checkpoint without independently retained accepted leaf/content is detection evidence, not a recovery copy.

If a later trigger introduces `K` into an already accepted `P` or `P+M` family, resume at decision 2: choose standalone `K` versus independently required `P+K`. Never silently turn a later `K` requirement into `P+K`, and never introduce `K` merely because `T` was selected.

These are authority and strictness choices, not requests to implement a mechanism. The existing [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md) ticket is the decision carrier after the universe inventory is available.

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

These commands were run from the dispatched worktree during the initial audit and independent-review repair. Commands carrying the immutable base were re-run during repair; their output is intentionally summarized rather than copied as an unbounded terminal log.

```sh
git branch --show-current
git rev-parse HEAD
git status --short
git rev-list --left-right --count 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a...HEAD
tkt claims --format json
```

They reported the claimed branch `tkt/cost-protected-review-versus-signed-conformance-authority`, exact HEAD `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`, a clean tree, `0 0` divergence from the dispatch base, and a live `conformance-authority-sol` claim.

```sh
git ls-tree -r --name-only 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a | awk -F/ '$NF == "CODEOWNERS" || $NF == "SECURITY.md" || $0 ~ /^\.github\/workflows\// || $0 ~ /^\.githooks\// { print }'
git config --get core.hooksPath
git config --get commit.gpgsign
git cat-file commit 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a
git log --format='%H %G? %GK %GS' -n 20 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a
gh api repos/moderately-ai/tiler/branches/main/protection
gh api repos/moderately-ai/tiler/rulesets
gh repo view moderately-ai/tiler --json defaultBranchRef,isPrivate,viewerPermission
```

The tree-wide exact-name/prefix census returned no paths. It reaches root, `.github/`, and `docs/` `CODEOWNERS` locations instead of assuming the file can only be at the root; it also reaches tracked workflows, tracked hook paths, and any `SECURITY.md`. The two Git configuration queries returned no value; the current commit object carried no `gpgsig`; all 20 bounded log rows reported Git's `N` signature status; the branch-protection endpoint returned HTTP 404 with `Branch not protected`; the ruleset endpoint returned `[]`; and the public repository reported `main` as default with the querying principal holding admin permission. The current clone's common hooks directory contained only `.sample` files and there was no active repository hook. [`Documentation metadata and traceability`](../../../docs/document-metadata.md), at the source anchor `there is no CI`, independently states the repository policy status. These observations establish the exact-base tracked tree plus the current clone/host row, not an immutable GitHub guarantee and not absence from every possible external service.

## Per-Fact audit before analysis

The ticket carries no section labelled `Fact`; its goal, work list, non-goals, and acceptance criteria are requirements. The premises those requirements depend on were re-read at the exact base and audited here so proposal text is not silently promoted into evidence.

| premise used by the ticket | verdict | exact-base evidence and consequence |
| --- | --- | --- |
| A repository-local conformance report is already protected by a repository enforcement mechanism. | **False.** | The tree-wide exact-name/prefix census found no tracked workflow, `CODEOWNERS` in any recognized location, tracked hook, or security policy; current `main` had no branch protection or ruleset. Local `make` and `tkt` commands are voluntary checks for an actor who can rewrite or skip them. |
| The repository's ordinary process includes human review and full gates. | **Verified, but imprecise as enforcement.** | `AGENTS.md` anchors `Tom retains decisions about` and `gate → push → dispatch`; ADR 0075 describes conditional coordinator authority. These are governing process rules, not a server-side prevention mechanism. |
| `An implementation change cannot change its own goal profile, oracle, exception ledger, and evidence baseline in the same work item.` | **Verified as a proposal, not accepted policy.** | The root spike contains that exact anti-greenwashing item. No accepted ADR or host rule implements it yet. The decision ticket downstream of this work owns adoption. |
| Protected owner review and a separate-work-item rule are distinct enforced authorities under current GitHub primitives. | **False.** | Required code-owner review gates protected paths, but neither `CODEOWNERS` nor branch protection natively classifies and rejects a pull request because protected-authority and implementation paths co-occur. The split adds an enforced property only when a required trusted classifier outside the repository writer's authority rejects mixed diffs. |
| The owner-derived universe, goal profile, evidence baseline, verifier, and oracle are separable authorities. | **Verified.** | The root spike separates owner-derived universe from goal policy and executed evidence, requires independent oracle comparison, and binds evidence to profile and oracle/reference authority. Correctness-and-testing independently rejects self-derived expected values. |
| A signature over `profile/exception` alone addresses the complete manipulation threat in this ticket. | **False; the ticket wording is materially incomplete.** | Verifier, denominator enumerator, oracle authority, and evidence-baseline lineage remain mutable while the signed policy bytes stay unchanged. A surviving external root must bind all five authority identities or a manifest/commit that transitively and unambiguously binds them. |
| The repository can restrain an actor authorized to rewrite every repository check and authority. | **False by the actor's defined power.** | Every in-repository predicate, pin, test, expected failure, and historical baseline is in the actor's write set. Only a trust anchor, reviewer/ruleset, checkpoint, or witness outside that write set can constrain or expose the rewrite. |
| Cryptography supplies review quality or remains authoritative after its signer is compromised. | **False.** | A valid signature establishes that the holder of an accepted key signed the bound bytes. It does not establish that those bytes are a good policy, and a threshold-signing attacker can authorize malicious bytes. This is also an explicit ticket non-goal. |
| A signed authority manifest necessarily lacks independent policy judgment unless protected review is also required. | **False.** | An automated/unreviewed signing service supplies no judgment, but a threshold held by designated policy approvers can make semantic review and explicit signing the approval act. Standalone `K` is sound for A0–A4 while those approvers, their threshold keys, the canonical resolver, and client root remain honest; `P+K` adds a second host authority and merge-time control. |
| Witnessed transparency requires a `K`-signed policy leaf. | **False.** | `T` needs an authenticated accepted-authority leaf, not necessarily a policy-signing key. While A3/host compromise is excluded, a witness can authenticate and log an exact commit/content identity plus evidence that `P` approved that exact latest state. This yields `P+T`; the witness records host authorization but does not replace it or make host compromise safe. |
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
| A5 external-store compromise | four independently analyzed powers: corrupt/substitute stored bytes; serve an older/frozen view; withhold data; or equivocate across clients | no signing threshold and no client-anchor rewrite unless separately compromised | integrity refusal, rollback/freeze, availability loss, or split view respectively |
| A6 signing-key compromise | can produce valid signatures for every role whose threshold it satisfies | cannot erase a monitored/witnessed history or replace independently pinned recovery keys unless separately compromised | malicious update that passes signature verification |

### Power interactions that must not be blurred

- A2 is not independent when the reviewer is also the implementation author or when the latest push is not reapproved. GitHub documents both stale-approval dismissal and “approve most recent reviewable push”; the safer anti-hijack setting is stale dismissal.
- A3 is an unavoidable trust point for protected review. “Do not allow bypassing” narrows ordinary bypass, but a host owner able to edit the rule remains outside the rule's own control.
- A4 defeats every local negative control even if all of them continue to pass. A host-protected policy reviewer can restrain A4 only while A4 lacks A3/rules-bypass authority. A policy-approver-held signature threshold can restrain A4 only while A4 lacks A6 and cannot replace the client's pinned root. An automated signer that signs repository-selected state after repository-selected checks merely gives A4 a valid signature and is not `K`.
- A5 corruption is not the same as A5 withholding: `K` rejects altered or substituted bytes and a rollback relative to client-retained monotone version state, but no signature can make a withholding store available. Freeze detection additionally needs an accepted expiry/freshness rule and trusted time; before expiry, a store can hide an unseen update without the client knowing one exists.
- A5 equivocation is not ordinary corruption. `P` receipts or `K` signatures can each authenticate two different authority histories without proving a globally unique view. A log without independent monitoring or a witnessed checkpoint is not a non-equivocation authority; compromise of the log plus every witness/checkpoint holder defeats that property.
- A6 defeats standalone `K` prevention for its threshold. Independently required `P` can still reject the repository change, and transparency may make misuse attributable and detectable after monitoring; neither makes an attacker-signed malicious policy invalid merely because it is public.

## Threat and power matrix

`P` means prevents acceptance under the stated assumptions, `D` detects after or during the attempt, `A` supplies attributable evidence, `R` has a defined recovery path, and `—` supplies none. Cells name the smallest external placement capable of the property: `HOST` is protected combined policy-owner review; `MIX` is the separately trusted mixed-diff classifier/attestation; `KSIG` is a reviewed, policy-approver-held threshold signature over the five-class manifest with retained monotone client state; and `LOG` is append-only publication of an authenticated `HOST`- or `KSIG`-approved leaf plus independently held witnessed checkpoints and monitoring. An automated/unreviewed signature is deliberately not `KSIG` and receives no semantic-manipulation credit.

| attack | A0 | A1 | A2 | A3 | A4 | A5 | A6 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| shrink denominator or delete tombstone | HOST or KSIG: P/D/A | HOST or KSIG: P/D/A | distinct HOST or KSIG approver: P/D/A | KSIG: P/D/A if independent/no key | KSIG: P/D/A | — | HOST: P/D/A only in independent `P+K`; LOG: D/A after publication |
| weaken required/optional/N/A or exception | HOST or KSIG: P/D/A | HOST or KSIG: P/D/A | distinct HOST or KSIG approver: P/D/A | KSIG: P/D/A if independent/no key | KSIG: P/D/A | — | HOST: P/D/A only in independent `P+K`; LOG: D/A after publication |
| weaken verifier or report schema | HOST or KSIG: P/D/A | HOST or KSIG: P/D/A | distinct HOST or KSIG approver: P/D/A | KSIG: P/D/A if verifier identity bound | KSIG: P/D/A if verifier identity bound | — | HOST: P/D/A only in independent `P+K`; LOG: D/A after publication |
| replace independent oracle or tolerance | HOST or KSIG: P/D/A | HOST or KSIG: P/D/A | distinct oracle/policy approver: P/D/A | KSIG: P/D/A if oracle identity bound | KSIG: P/D/A if oracle identity bound | — | HOST: P/D/A only in independent `P+K`; LOG: D/A after publication |
| reset evidence baseline to weaker evidence | HOST or KSIG: P/D/A | HOST or KSIG: P/D/A | distinct HOST or KSIG approver: P/D/A | KSIG: P/D/A if lineage bound | KSIG: P/D/A if lineage bound | LOG: D/A; R requires separately retained accepted content/approval proof or authorized mirror | HOST: P/D/A only in independent `P+K`; LOG: D/A after publication |
| combine authority and implementation/evidence in one work item | MIX: P/D/A | MIX: P/D/A | MIX: P/D/A | MIX blocks the form if no bypass; separate malicious updates remain | MIX blocks the form while external; separate malicious updates remain | — | — |
| rewrite repository history and all local checks coherently | HOST or KSIG: P/A | HOST or KSIG: P/A | HOST or KSIG: P/A | KSIG: P/D/A | KSIG: P/D/A | checkpoint: D/A; independently retained accepted content/approval proof or authorized mirror: R | HOST: P/A only in independent `P+K`; LOG: D/A after publication |
| corrupt or substitute stored authority bytes | — | — | — | — | — | KSIG: P/D; no unique attribution | — |
| serve rollback/frozen authority state | — | — | — | — | — | KSIG retained version: P/D rollback; expiry/freshness: eventual D freeze | — |
| withhold authority or log data | reviewer outage: D, fail closed | reviewer outage: D, fail closed | alternate reviewer: R | cached KSIG verifies an existing valid claim only | cached KSIG verifies an existing valid claim only | no availability P; timeout/expiry D; publication stops | surviving threshold/recovery keys: R |
| present two accepted-authority histories to different clients | HOST/KSIG authenticate their respective leaves but not global uniqueness | same | same | HOST receipts or KSIG may authenticate both; no non-equivocation | KSIG may authenticate both; no non-equivocation | LOG+witness: D/A | LOG+witness: D/A |

The matrix is deliberately asymmetric. `HOST` and `KSIG` are alternative honest human policy-approval placements; requiring both gives defense in depth only when their control and failure modes are independent. A protected maintainer who lacks the signing threshold is constrained by `KSIG`; a signer who satisfies the accepted threshold is not made honest by tooling. `MIX` prevents a coupled subject form and makes histories separately attributable; it does not stop an accepted policy approver from authorizing the same weakening in separate work items. `KSIG` supplies stored-content integrity and rollback rejection relative to retained state, but not store availability. `LOG` can witness either base's authenticated leaf. A checkpoint proves what history was witnessed; it cannot reconstruct deleted accepted content or approval proof. Recovery is credited only where those bytes survive independently or through an authorized mirror.

## Candidate option analysis

### S0 — status quo: prose process plus repository-local gates

- **Prevention/detection/attribution/recovery:** catches ordinary mistakes only when contributors run the gates and reviewers apply the prose. It cannot prevent A1/A4 from editing or skipping the gate, does not provide durable approval attribution beyond ordinary Git history, and has no separate recovery anchor.
- **Cost/offline/availability:** zero new setup and fully offline. There is no new authority to become unavailable, which is also why there is no external authority to resist a coherent rewrite.
- **Unavoidable trust:** every repository writer and merger plus the correctness of all local checks.
- **Strongest counterargument:** Tiler is pre-production and a human already coordinates every landing; infrastructure cost may exceed present exposure.
- **Reversal evidence:** none can make this sound for A1/A4 because the absence is structural. Evidence could only narrow the threat model to accidental edits, which contradicts the ticket's explicit repository-writing actor.
- **Disposition:** eliminated for failing the named threat.

### P / S1 — path-protected combined human review

Configure an external repository ruleset/branch protection to require pull requests, protected-owner approval, stale approval dismissal, no force push/deletion, and no ordinary bypass for the eventual five authority path sets. Protect the `CODEOWNERS`/rule-definition path itself. GitHub's current documentation says protected branches can require approving or code-owner reviews, dismiss stale approvals, require the latest reviewable push, block force pushes/deletion, and disable bypass; rulesets also have explicit bypass lists. These are provider capabilities, not current repository state.

- **Prevention/detection/attribution/recovery:** prevents A0/A1 and an ordinary implementation reviewer from landing any protected-authority change without fresh approval from the protected policy owner and attributes that approval through the host. A mixed authority-plus-implementation diff remains visible to and approvable by that owner; this option relies on the owner's judgment rather than mechanically forbidding the form. It does not constrain a dishonest A3 or a host administrator who changes/bypasses the rule. Host history helps diagnose ordinary bypass but is not an independent root against host compromise.
- **Cost/offline/availability:** one-time ruleset and ownership configuration, then one online protected review per authority-touching change. Offline development remains possible; accepted merge/qualification waits for host access and a reviewer. No Tiler runtime or kernel cost.
- **Recovery/rotation:** rotate code-owner teams and reviewer accounts in the provider; emergency bypass must be narrowly held, logged, and followed by independent review. If every protected reviewer is unavailable, authority changes stop; old accepted profiles remain usable.
- **Strongest counterargument:** a mixed diff lets an implementation author present a weakened authority as necessary context and makes independent review harder; an honest owner can still miss the coupling.
- **Reversal evidence:** repeated mixed-diff negative controls or real review escapes showing that protected owner review does not reliably isolate authority semantics would justify the external `M` add-on. Conversely, no mechanism can make this option protect against a dishonest protected owner without adding another authority such as `K`.
- **Disposition:** retained as the lowest-cost frontier candidate. The root spike's separate-work-item sentence is a proposal, not an accepted invariant, and cannot be used to eliminate this option before Tom decides it.

### S2 — split policy/exception approval without a trusted mixed-diff check

Declare that policy/exception and implementation changes use different work items and approvers, but enforce that declaration only through repository prose, ticket metadata, or reviewer memory.

- **Prevention/detection/attribution/recovery:** useful review discipline for A0, but A1/A4 can change or ignore it, and GitHub `CODEOWNERS`/branch protection does not natively reject a mixed-path pull request. A2 remains a trust point; distinct approvers reduce single-reviewer compromise only if roles and mixed-diff exclusion are enforced outside ordinary repository writes.
- **Cost/offline/availability:** two work items or approval lanes and additional human latency, no host/runtime cost. Fully offline as prose; therefore not enforceable against the named repository writer.
- **Strongest counterargument:** no platform setup, and ticketsplease already models separate work.
- **Reversal evidence:** a trusted external required check with a complete path classifier would turn this into the `M` add-on below, not rescue this option as stated.
- **Disposition:** eliminated as unenforced policy, not because every split is valueless.

### M — trusted external mixed-diff exclusion, combined with P and/or K

`M` is not `CODEOWNERS` and is not a repository-local script that a work item can edit. It is a trusted classifier whose configuration and enforcement are outside ordinary repository-writer authority. It compares the exact predecessor/successor source identities, classifies the eventually accepted five authority path sets and implementation/evidence paths, fails when one work item touches both populations, and prints the matched paths/classes. With `P`, the host requires its result before merge. With standalone `K`, the policy-signing workflow and clients require a classifier attestation bound to the manifest/source identity; a repository writer cannot replace the result with a local green check. `M` alone does not authorize an authority change.

- **Prevention/detection/attribution/recovery:** prevents A0/A1/A2 from coupling implementation/evidence and its authority in one accepted work item even when the selected policy approver misses the coupling. It creates separately attributable histories. It does not stop A3 or an accepted `K` policy approver from authorizing the same semantic weakening in separate work items, and it adds no protection when its own service/configuration authority is compromised.
- **Cost/offline/availability:** an external classifier/attestor, complete maintained path taxonomy, exact-diff binding, enforcement integration, and a negative control for every authority/implementation class pair. `P+M` blocks relevant merges while unavailable; `K+M` permits repository development but blocks signing/acceptance. A repository-local classifier does not meet this definition for A4.
- **Recovery/rotation:** rotate classifier identity/configuration under an authority independent of ordinary repository writes, preserve known-good taxonomy versions, re-evaluate open changes, and invalidate attestations over superseded diffs. A stale or incomplete path population fails closed.
- **Unavoidable trust:** complete path ownership, classifier/attestor correctness and credentials, its enforcement binding, and the selected `P`/`K` policy approver.
- **Strongest counterargument:** it adds a service and taxonomy but no protection against a dishonest policy approver; its gain is review isolation and mechanical compliance with a proposed process invariant.
- **Reversal evidence:** repeated mixed-diff review escapes, or Tom's acceptance of “no authority and implementation in one work item” as normative policy, justifies `M`. Evidence that the taxonomy cannot be complete or availability dominates argues for the same approval placement without `M`.
- **Disposition:** retained as optional `P+M`, `K+M`, and `P+K+M` strictness frontiers; not recommended by default.

### S3-A — automated or unreviewed signed manifest

An automated service signs repository-selected manifest bytes after repository-selected checks, or a human key holder signs without owning and reviewing the policy decision. The manifest may still have correct canonicalization, versioning, and cryptography.

- **Prevention/detection/attribution/recovery:** rejects A5 byte corruption and retained-state rollback, but it does not prevent A1/A4 semantic greenwashing: the writer can change the five-class state and the checks that cause the automated signer to approve it. The signature attributes the action to the service/key, not to independent policy judgment.
- **Cost/offline/availability:** pays essentially all schema, key, verifier, rotation, and outage cost of `K` while omitting its human authority property.
- **Strongest counterargument:** deterministic signing avoids human latency and produces tamper-evident artifacts.
- **Reversal evidence:** none makes an unreviewed signature a semantic policy authority; assigning threshold keys to designated policy approvers and requiring their review converts it into `K`.
- **Disposition:** eliminated for the manipulation threat, while retaining its narrower stored-content integrity property.

### K / S3-H — policy-approver-held signed five-class authority

Clients pin an external public root and accept only a monotonically versioned canonical manifest signed by the required threshold of designated policy approvers. Signing is their explicit approval act after reviewing the semantic authority diff; ordinary repository writers and repository-selected automation cannot invoke the threshold. The manifest binds the exact identities/digests of all five classes, schema/resolver version, role/threshold metadata, validity policy, and predecessor/root version. A TUF-style root supplies the reference vocabulary for thresholds, offline roots, old-plus-new root continuity, rollback/freeze checks, and recovery; this does not propose importing TUF wholesale.

- **Prevention/detection/attribution/recovery:** standalone `K` prevents A0–A4 from creating an accepted authority while they lack the policy-signing threshold, even if they merge arbitrary bytes and rewrite repository history. It rejects A5 corrupted/substituted bytes and rollback older than retained client state; a freshness rule can eventually expose freeze. It attributes approval to the threshold roles. It does not prevent A6 threshold misuse, supply availability/non-equivocation, or guarantee that an authorized policy judgment is correct.
- **Cost/offline/availability:** canonical schema/resolver, semantic diff/signing workflow, pinned client root/version state, policy-approver key ceremony/storage, verifier integration, rotations and recovery drills. It avoids protected-path/ruleset machinery, but unapproved repository states may merge and fail only at authority-aware audit/qualification, which worsens contributor feedback and repository hygiene. Existing manifests verify offline; new acceptance requires the threshold. Withholding still blocks updates. No kernel cost; audit/qualification adds fixed unmeasured parsing/digest/signature work.
- **Recovery/rotation:** a successor is authorized by old and new thresholds; a sub-threshold lost/compromised key is replaced by a surviving threshold. Threshold compromise requires out-of-band client-root replacement and may be unrecoverable for affected unattended clients.
- **Unavoidable trust:** policy signers' judgment, threshold custody/recovery, canonical resolver, retained client state, and pinned-root distribution. This is the same kind of terminal honest-human assumption as `P`, placed outside repository-host control.
- **Strongest counterargument:** keys and canonical schema may be disproportionate for one pre-production maintainer, while lack of premerge rejection lets unauthoritative content accumulate in the repository.
- **Reversal evidence:** inability to define stable five-class canonical closure defers `K`; an external consumer or in-scope A3/A4 rewrite makes it necessary. Evidence that host-side rejection materially reduces operational mistakes without correlated authority failure supports adding `P`.
- **Disposition:** retained as a standalone frontier, and with optional `M`/`T`.

### S4 — append-only external transparency record alone

Publish repository-selected authority updates to a cryptographically verifiable append-only log without binding each leaf to an accepted `P` or `K` approval. The useful log property requires inclusion proofs, consistency monitoring, and an independently distributed or witnessed checkpoint; an inclusion promise from the same store does not prevent a split view. Sigstore's security documentation likewise says long-term trust requires monitoring and that transparency supplies auditability, while witnesses/monitors establish consistent append-only views rather than judging leaf legitimacy.

- **Prevention/detection/attribution/recovery:** detects deletion, rollback, or equivocation of whatever was logged after a trusted checkpoint, but does not establish that a policy approver accepted the leaf. It can attribute only the publisher/log. A checkpoint identifies missing content but cannot reconstruct it.
- **Cost/offline/availability:** pays external log, monitor/witness, checkpoint distribution, retention, incident response, and client verification cost without obtaining an approval authority.
- **Recovery/rotation:** rotation can restore log service, but accepted authority can be recovered only when independently retained content is also authenticated by a surviving `P` or `K` approval. Compromise of the log plus every witness/checkpoint holder defeats non-equivocation.
- **Strongest counterargument:** a public log adds privacy, availability, and operations burden while doing nothing to stop a valid signer from making a bad decision.
- **Reversal evidence:** multiple independent consumers requiring durable public audit, or observed equivocation/history rewrite, would justify it; a single private pre-production profile does not.
- **Disposition:** eliminated alone. It becomes `T` only when every leaf is authenticated by the selected base approval authority.

### P+K — independently required host and signing approvals

Require both fresh host-protected policy-owner approval and the independent `K` policy-signing threshold. A client/qualifier must verify that the manifest binds the exact commit/content accepted through `P`, using a live trusted-host query or an independently authenticated approval receipt cached after such a query; merely running both workflows without a checked binding is not dual authority. The host and signer controls must be independently administered for defense-in-depth claims. The same human may participate in both, but then the combination separates credentials/mechanisms rather than malicious judgment.

- **Prevention/detection/attribution/recovery:** preserves `K` client-side rewrite resistance and adds premerge rejection/attribution. It tolerates A3/host compromise while `K` survives and A6 compromise while `P` survives; it does not tolerate their coalition or correlated dishonest judgment. It still has no non-equivocation without `T` and no mechanical mixed-diff rule without `M`.
- **Cost/offline/availability:** pays both ruleset/reviewer and schema/key/client-verifier costs plus exact host-approval binding. A client that already validated and retained a bound accepted state can reverify it offline; first acceptance of a successor needs the trusted-host condition or an independently authenticated receipt. New authority updates block if either channel is unavailable.
- **Strongest counterargument:** two approval systems may duplicate the same human judgment and double outage/recovery paths without adding independent security.
- **Reversal evidence:** distinct administrators, observed host or key compromise, or material value from early merge rejection supports the combination. Strong correlation between both authorities and no need for repository hygiene favors standalone `K`.
- **Disposition:** retained as defense-in-depth/usability frontier, with optional `M`/`T`.

### T — witnessed transparency over the selected accepted-authority leaf

`T` can be added to `P`, `K`, or `P+K`, with or without `M`. Every accepted authority update, rotation, revocation, and recovery event is published as an authenticated leaf and must receive the required inclusion proof and witnessed checkpoint before acceptance.

- **`P+T` leaf:** bind the exact commit/content identity, deterministic five-class closure, predecessor, protected-owner approval of that exact latest state, and relevant host-rule identity. Authentication can be a provider-signed receipt if the selected provider actually supplies one, or an independent witness/attestor that queries the trusted host and signs the exact observed approval state before logging it. This attestor reports `P`'s decision; it does not hold policy discretion and is not `K`. Current GitHub evidence establishes approval APIs/rules, not a provider-signed portable receipt, so the selected implementation must prove one concrete authenticated-receipt path.
- **`K+T` leaf:** the policy-approver-threshold-signed five-class manifest is the authenticated leaf.
- **`P+K+T` leaf:** bind and verify both the exact host-approved source identity and `K` manifest approval. If `M` is selected, its exact-diff attestation is also bound into the leaf for `P+M+T`, `K+M+T`, or `P+K+M+T`.
- **Prevention/detection/attribution/recovery:** preserves the selected base's approval boundary, then adds durable detection/attribution for deletion, equivocation/split histories, and visible authority misuse after monitoring. `P+T` does not resist A3/host compromise: a malicious but host-approved leaf can still be logged. A checkpoint identifies a witnessed last-good leaf, but recovery requires the corresponding accepted content and approval receipt/signature retained independently or obtainable from an authorized mirror.
- **Cost/offline/availability:** all variants add log, accepted-leaf/content retention or an authorized mirror, monitors/witnesses, checkpoint distribution, incident response, and required publication availability. `P+T` additionally needs an authenticated host-approval receipt/attestation and deterministic commit-to-five-class resolution, but avoids `K` policy-key, signed-manifest, client-root rotation, and signer-recovery machinery. `K+T` pays the `K` machinery instead. Existing complete bundles verify offline; new publication blocks while the required host receipt/signature or log/witness quorum is unavailable.
- **Unavoidable trust:** the selected `P` and/or `K` authority plus leaf authentication, log/witness keys, monitor operation, checkpoint distribution, independently retained accepted content/approval proof, and incident response.
- **Strongest counterargument:** every variant creates supply-chain operations for a pre-production single-repository policy; `P+T` also creates a portable host-approval attestation scheme whose concrete provider support is not yet evidenced.
- **Reversal evidence:** multiple independent consumers, public attestations, observed equivocation/authority misuse, or required witnessed history makes `T` valuable. Failure to construct an authenticated exact `P` receipt eliminates `P+T` at implementation time rather than silently upgrading it to `K+T`.

### B — bounded research, and D — deferral

Further bounded research is a work state, not an authority placement. It is sound only while no accepted `qualify` claim is emitted. The useful bounded questions are the five-class canonical manifest closure, concrete protected paths after the universe inventory, and a dry-run cost/recovery exercise; all are already owned or naturally descend from the existing decision/design graph. Deferral is likewise sound only if the first profile remains explicitly unaccepted and every unavailable-authority result is `Unknown`/nonzero rather than green.

- **Strongest counterargument:** delaying authority while implementation proceeds creates facts and file layouts that will pressure the later policy to bless the status quo.
- **Reversal evidence:** the inventory and command-contract tickets producing stable identities and a minimal protected path set ends the reason to defer.
- **Disposition:** available as a fail-closed scheduling state, not a frontier authority for an accepted profile.

## Eliminated options and Pareto frontier

The decision dimensions are correctness, fail-closed strictness, long-term maintainability/compatibility, Tiler host/runtime cost, operational/recovery cost, and the protected threat population. Kernel performance is unaffected by all candidates because authority verification is outside kernel execution.

The complete product is derived rather than sampled. Let the base approval `B` be exactly one of `{P, K, P+K}`; independently let `m ∈ {absent, M}` and `t ∈ {absent, T}`. That yields `3 × 2 × 2 = 12` sound conditional products. Each add-on product is nondominated only when its added property is required; otherwise the same base without that add-on dominates it on cost and availability.

| base approval | neither add-on | `M` only | `T` only | `M+T` |
| --- | --- | --- | --- | --- |
| `P` | `P`: lowest-cost A0/A1/A2 host authority | `P+M`: mechanical mixed-diff separation | `P+T`: host-approved authenticated leaf plus witnessed audit, without `K` machinery | `P+M+T`: host approval, mechanical separation, and witnessed audit |
| `K` | `K`: A0–A4 rewrite-resistant client authority | `K+M`: signed policy approval plus mechanical separation | `K+T`: signed accepted leaf plus witnessed audit | `K+M+T`: signed approval, separation, and witnessed audit |
| `P+K` | `P+K`: independently bound dual authority | `P+K+M`: dual authority plus separation | `P+K+T`: dual authority plus witnessed audit | `P+K+M+T`: every selected property and every associated burden |

| candidate outside the product | disposition |
| --- | --- |
| no base: S0, `M`, `T`, or `M+T` | **Eliminated:** no policy approval authority; a classifier or log can process attacker-chosen state faithfully. |
| S2 unprotected prose/ticket split | **Eliminated:** repository writer can erase or ignore it, and native `CODEOWNERS` does not reject mixed path classes. |
| profile/exception-only signature | **Eliminated:** verifier/oracle/denominator/baseline bypass keeps the signature valid. |
| S3-A automated/unreviewed signature | **Eliminated:** repository-selected automation can sign a coherently weakened state; integrity is not policy judgment. |
| unauthenticated P log leaf | **Eliminated:** an exact commit without an authenticated exact host-approval receipt proves content identity, not `P` authorization. |
| unbound `P+K` workflows | **Eliminated:** two activities over potentially different source states are not dual authority. |
| bounded research/deferral | **Not an accepted authority:** safe only while qualification remains `Unknown`/nonzero. |

No base dominates. `P` has the least key/schema machinery and best merge-time feedback, but trusts the protected host. `K` avoids protected-host machinery and resists A3/A4 for accepting clients, but pays key/schema/recovery cost and permits unauthoritative repository states to merge. `P+K` tolerates either independently controlled authority being compromised and rejects early, but pays both costs. `P+T` and `P+M+T` are genuine audit frontiers when host compromise is excluded: they add authenticated witnessed history without `K` policy keys or signed-manifest schema. They are not credited with `K`'s host-rewrite resistance. The current recommendation remains `P` for the narrow A0/A1/A2 threat; once A3/A4 enters scope, standalone `K`, not automatically `P+K`, is the smallest sound base.

## Cost, offline, unavailability, and recovery comparison

No wall-time or monetary cost was measured because no mechanism was installed and provider/key choices are undecided. The table therefore reports operations and asymptotic work, not fabricated timing.

| property | P protected review | K policy-signing authority | P+K dual authority | M optional delta | T optional accepted-leaf delta |
| --- | --- | --- | --- | --- | --- |
| terminal honest authority | policy owner's judgment plus host rules/bypass administration | policy signers' judgment, threshold/root custody, canonical resolver and client-root distribution | both independently required; tolerates either channel's compromise alone | classifier/attestor, complete taxonomy, exact-diff binding | selected base approval plus leaf authenticator, log/witness/checkpoint and monitor trust |
| one-time setup | ruleset/protected paths, owner team, stale-approval and bypass policy | canonical five-class manifest, client root/version state, signing roles/threshold, key ceremony and recovery drill | all `P` and `K` setup plus independence audit and exact host-approval receipt/query binding | external classifier/attestor, taxonomy, enforcement binding and negative controls | log, inclusion/consistency verification, accepted-content retention or authorized mirror, monitor/witness and incident runbook; `P+T` also needs authenticated host receipt |
| routine authority update | combined diff allowed; fresh protected-owner approval before merge | semantic authority diff reviewed and threshold-signed; merge may precede acceptance | fresh host approval and threshold signature over the same exact source identity, both checked at acceptance | separate work items and bound classifier result | publish authenticated base-approved leaf plus inclusion and witnessed checkpoint |
| repository/host machinery | protected ownership/rules reference; host rule is load-bearing | manifests/signatures/roots; no protected-review mechanism required | both sets | taxonomy/attestation references; service stays outside writer authority | inclusion bundles/checkpoints or references; retained history grows |
| Tiler host cost | review-time only; audit reads accepted local state | fixed unmeasured canonical parse/digest/signature verification at audit/qualify | `K` verification plus host review-time cost | one external classification/attestation per relevant update; unmeasured | inclusion/checkpoint verification and optional network fetch; unmeasured |
| runtime/kernel cost | none | none when authority verification precedes build/qualification | none | none | none |
| offline development | full; cannot accept/merge authority change offline | full against cached root/manifest; cannot mint successor without threshold | full for an already validated and retained bound state; first successor acceptance needs live host validation or independently authenticated cached approval receipt plus threshold | full, but relevant acceptance/merge awaits attestor | full against complete cached bundle; cannot publish successor offline |
| unavailable authority | block protected authority merge/qualification; last accepted state remains | verify last valid cached state; block new publication/rotation/identity | block update if either channel is unavailable | block merge under `P+M` or signing/acceptance under `K+M`; never drop `M` silently | block new acceptance until inclusion/witness requirement is met |
| reviewer/signer recovery | rotate host team/account; post-review any emergency bypass | below-threshold rotation with old/new continuity; out-of-band root recovery after threshold compromise | perform the affected channel's recovery and revalidate through the survivor | rotate attestor/config, invalidate stale attestations, reclassify open diffs | rotate log/witness through trusted root and publish monitored recovery event |
| stored-content corruption | Git hashes only; no independent root after coherent rewrite | signature/digest rejects altered bytes | same as `K`, plus host copy may aid diagnosis | no additional content-integrity property | witnessed inclusion identifies accepted leaf; restoration still needs retained accepted bytes and approval proof/mirror |
| rollback/freeze | retained clones/review history only | retained version rejects rollback; accepted expiry/freshness eventually detects freeze | same as `K`, plus host history | no additional rollback property | witnessed history improves detection/attribution, not content recovery by itself |
| withholding | no prevention; host outage blocks merge | no prevention; cached valid state may continue within policy, updates stop | no prevention; either outage blocks updates | no prevention; attestor outage blocks its acceptance point | no prevention; cached complete bundle may continue within policy, publication stops |
| equivocation | host approval can authorize multiple histories without global uniqueness | two valid signer-authorized histories may both verify | both approvals still do not establish global client view | no additional property | witness/monitor detects inconsistent histories unless trust set compromised |

The `T` cost differs by selected base:

| `T` family | authenticated leaf and extra machinery | distinct tradeoff |
| --- | --- | --- |
| `P+T`, `P+M+T` | exact commit/content and five-class closure plus authenticated latest-state host-approval receipt/attestation; no policy-signing manifest, signer threshold, or client signing-root rotation | lowest signing-key burden, but trusts A3/host and needs a concrete portable receipt/attestor design |
| `K+T`, `K+M+T` | threshold-signed five-class manifest | independent of host approval, but pays `K` key/schema/root and recovery machinery |
| `P+K+T`, `P+K+M+T` | exact binding of both host receipt and signed manifest | tolerates either approval channel's compromise when independent; greatest setup and outage surface |

### Fail-closed availability rules

1. Unavailable protected reviewer: when `P` is selected, ordinary work may continue on branches, but no protected authority update merges or becomes accepted and `qualify` cannot cite it.
2. Unavailable `M` classifier: `P+M` blocks relevant merges and `K+M` blocks signing/acceptance; no fallback to the same placement without `M` is allowed until Tom changes policy.
3. Unavailable signing threshold: the last unexpired/policy-valid cached authority can still verify; no unsigned or repository-default replacement is accepted. If expiry is adopted, expiry yields a named unavailable/unknown authority, not automatic extension.
4. Unavailable transparency service or leaf authenticator: if `T` is required, a new accepted-authority leaf remains pending until its `P` receipt and/or `K` signature, inclusion proof, and required witnessed checkpoint exist. Existing complete offline bundles remain verifiable under their stated validity policy.
5. Recovery bypass: recovery changes the root of trust; it cannot be treated as an ordinary implementation merge. An emergency repository merge without restored authority may repair code but cannot mint a conformance-green claim.

## Independent subject perturbations

These are required tests for the eventual mechanism; none was implemented in this research ticket. Each perturbation changes the protected subject, not the assertion, and each expected message names the independent property that must reject. The downstream [perturbation-suite ticket](../../../tickets/design-the-conformance-denominator-and-receipt-perturbation-suite.md) already owns executable realization after the authority decision and command contracts.

| protected property | subject perturbation | expected `P` result | additional `M` result | additional `K` or `T` result |
| --- | --- | --- | --- | --- |
| denominator closure | add an owner-registered feature without a universe disposition, then separately delete an existing tombstone | protected owner approval required; `audit` names missing/disappeared key | if paired with implementation/evidence, classifier rejects and prints both path classes | `K` manifest identity mismatches; any `T` family lacks an authenticated included successor leaf |
| exception strictness | change one required cell to optional/N/A and separately remove its reason/expiry | protected owner must approve; `regress` names authority change rather than progress | mixed implementation/exception work item is rejected; an exception-only work item proceeds to owner review | `K` needs reviewed signed successor; `P+T` needs exact latest-state host receipt and witnessed inclusion |
| verifier strictness | change `qualify` to treat missing/Unknown receipt as success | verifier path requires protected owner approval; negative case must report exact missing obligation | mixed verifier/implementation work item is rejected | bound verifier identity differs even when profile bytes do not |
| oracle independence | replace a hand-written expected result with output recomputed by the implementation, then separately widen a tolerance | oracle path requires protected owner approval; oracle perturbation must fail expected comparison | mixed oracle/implementation work item is rejected | bound oracle/tolerance identity differs while implementation identity remains unchanged |
| evidence monotonicity | replace accepted baseline with a receipt set missing one formerly sufficient case | protected owner approval required; `regress` prints lost cell | mixed implementation/baseline work item is rejected | lineage/digest and successor identity differ; old accepted baseline is recoverable only if its content and approval proof survive |
| mixed-diff classification | in one work item edit one member of each protected authority class together with one implementation/evidence path, separately per class | protected owner may approve; this deliberately shows `P` has no mechanical mixed-diff property | required check rejects every subject pair and prints both matched classes; removing one class makes it pass to review | no additional result unless signed manifest changes |
| latest-push review | obtain approval, then push protected-authority edit | stale approval is dismissed and merge blocked | classifier reruns against new diff | `P+T` rejects receipt/source mismatch; `K` also lacks accepted successor signature |
| P-leaf authentication | obtain valid `P` approval for commit A, submit commit B with A's receipt, then separately fabricate an unauthenticated approval JSON for B | host rejects B absent fresh approval | classifier result remains bound to its own exact diff | every `P+T` family rejects the mismatched or unauthenticated receipt before log inclusion can authorize B |
| repository rewrite | change all five local authorities and every local check so suite reports green | `P` fails only if host protection/approver is outside attacker power; this is explicit limit | external classifier may reject mixed form but cannot stop same rewrite separated across work items | pinned root rejects rewritten unsigned/untrusted manifest |
| unreviewed signing | have repository automation sign a coherently changed manifest using a key outside the policy-approver role; separately model theft of enough accepted policy keys | no additional result unless `P` is also required | no additional result for separated change | unauthorized automation signature fails role/threshold; stolen accepted threshold passes and is reported as A6, proving cryptography is not review |
| stored-byte corruption | flip one byte of a `K` manifest or `P` accepted content/receipt without changing its approval proof | no independent result after coherent repository rewrite | no independent result | `K` rejects signature/digest mismatch; `P+T` rejects receipt/leaf or checkpoint mismatch; no store attribution claimed |
| rollback | serve an older, otherwise valid manifest after client retained a newer version | no independent result after history rewrite | no independent result | monotone retained version rejects rollback |
| freeze | serve last valid manifest until its accepted freshness/expiry boundary passes | no independent result unless host history observed | no independent result | before boundary update may be unknowably hidden; at boundary typed stale/freeze result blocks new claim |
| withholding | make authority/signing/log service unreachable in separate trials | no accepted authority-changing merge | no classified merge; no reviewer-only fallback | cached valid authority may verify within policy; new accepted publication fails unavailable, and no availability prevention is claimed |
| signing-key compromise | sign malicious five-class manifest with accepted threshold | `P` may reject if independent review remains uncompromised | mixed form may be rejected; separated malicious work remains | `K` signature checks pass; `T` records/attributes event but does not prevent it |
| log equivocation | present two valid inclusion histories to separate clients | no dedicated result | no dedicated result | independent witness/checkpoint consistency fails; lone log may pass both views |
| checkpoint without content | retain a witnessed checkpoint, delete the corresponding accepted leaf/content and approval proof, and make every authorized mirror unavailable | no dedicated result | no dedicated result | checkpoint still detects/attributes the missing witnessed leaf, but recovery reports content unavailable rather than claiming `R` |

Each independent property needs its own perturbation and captured failure text. A single patch that changes all five subjects proves only that something is guarded. Counts must come from the concrete typed registries after the inventory work, not from a hand-written expected total.

## Recovery cases and unavoidable trust

### Reviewer or host-account compromise

Revoke the account/team membership, remove bypass, invalidate pending approvals by requiring fresh review after the latest push, inspect provider audit history, and re-review every authority version approved during the exposure window. The last known-good authority must be identified independently of the compromised reviewer's assertion. A `P` family, including `P+T`, cannot reject a malicious host-approved successor once A3/host compromise is in scope; `T` can retain evidence of the previously witnessed history but is not a second policy authority. A `K` family continues to accept only threshold-authorized versions while its root survives.

### Signing-key loss below threshold

Use surviving authorized keys to publish a versioned successor removing the lost key and adding a replacement. A TUF-style transition is signed by both the old and new accepted thresholds so older clients can verify continuity. Test this ceremony before depending on it; a policy that has never demonstrated recovery has an unknown recovery property.

### Signing threshold compromise

Assume malicious manifests may have been validly signed. Stop new qualifications, identify a last-good version through independent review and any witnessed record, distribute a new root out of band, and re-qualify descendants. TUF's specification warns that recovery after threshold root compromise is nearly impossible for already affected unattended clients; this packet therefore does not claim automatic recovery.

### External store corruption or substitution

For `K`, reject bytes whose bound digest/signature does not verify under the retained root. For `P+T`, reject content or a host receipt that does not match the witnessed authenticated leaf and exact approval attestation. Fetching from another store restores availability if an authorized copy exists; it does not change authority. Verification failure does not uniquely attribute corruption to the store because transport, disk, and caller substitution can produce the same invalid bytes.

### External rollback or freeze

`K` rejects a manifest older than client-retained monotone version state. Any `T` family rejects a log view inconsistent with a retained witnessed checkpoint. Detecting freeze still requires an accepted freshness/expiry rule plus trusted time; before that boundary, a client cannot infer that the store hid an update it never observed. Recover by fetching a newer valid accepted leaf/content from another distribution path or continue only under the explicitly valid cached-authority policy.

### External withholding

Approval proofs do not make a service available. Use cached valid authority only within its accepted validity policy, try an independently authorized mirror if designed, and block new publication/qualification once freshness or required inclusion cannot be established. Do not downgrade any `K` family to repository HEAD, any `P+K` family to one approval, any `M` family to its unclassified counterpart, or any `T` family to its unwitnessed base because an external authority is unavailable.

### External equivocation or log compromise

Two `P`-approved or `K`-signed histories may both pass their base authority; detect their divergence only relative to independent monitors/witnesses/checkpoints. A checkpoint identifies what was witnessed but is not the accepted leaf/content or its approval proof. Restore service only from independently retained authenticated leaves/content or an authorized mirror, validate them against the retained checkpoint and selected base authority, and quarantine the log. If no valid copy survives, the checkpoint supports detection/attribution but not recovery. If the attacker also controls every checkpoint/witness and the relevant host receipt/client-root distribution, no mechanism in this frontier supplies recovery.

### Unavoidable trust statement

Every option ends at some authority. `P` trusts policy-owner judgment and host access control. Standalone `K` instead trusts policy-signer judgment, canonical resolution, threshold custody/recovery, retained client state, and pinned-root distribution. `P+K` trusts both but tolerates either one being compromised only when their control is independent. `M` trusts an external classifier/attestor, complete path taxonomy, and its enforcement availability. `T` trusts authentication of the selected base's exact accepted leaf, monitors/witnesses, checkpoint distribution, and retained accepted content/approval proof or an authorized mirror for recovery. None protects against a coalition controlling all selected terminal authorities, and none establishes semantic correctness without the independent conformance oracles required by the correctness contract.

## External-authority limit

This limit is normative for interpreting the packet: a repository-local check can constrain only actors who cannot rewrite, replace, or skip it. If an actor can rewrite every repository authority and the expected results together, a green repository report is merely an attacker-chosen fixed point. An external rule, authenticated host receipt, or key helps only while the attacker cannot also change/bypass the rule, forge the receipt/attestor, satisfy the signing threshold, replace the client's trust anchor, or control every transparency witness. Claims beyond that boundary are unsupported, not “best effort.”

## Follow-up graph: selected mechanisms get explicit owners

No mechanism child is created before Tom selects an approval placement and optional properties; doing so would pre-choose the decision. The decision carrier must create bounded implementation/operations descendants for **every selected `P`, `M`, `K`, or `T` property**. The existing graph owns shared contracts and tests, but it does not silently own provider rules, classifier service, key operations, or transparency operations.

| work | existing owner and dependency | immediate or blocked |
| --- | --- | --- |
| enumerate concrete universe/profile authority objects and protected populations | [`inventory-the-closed-world-conformance-claim-universe-by-owner`](../../../tickets/inventory-the-closed-world-conformance-claim-universe-by-owner.md) | **Immediate research; in progress.** Supplies the path/identity population this packet deliberately does not invent. |
| choose `P`, `K`, or independently required `P+K`; then independently decide `M` and `T`; define who may approve authority changes | [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md), depending on this ticket and the inventory | **Tom decision/evidence blocked.** Must traverse the ordered branch without letting `T` force `K`, record terminal trust, and create the selected-property children below. |
| define fail-closed `audit`/`regress`/`qualify` semantics and machine failures | [`design-the-conformance-audit-regress-and-qualify-command-contracts`](../../../tickets/design-the-conformance-audit-regress-and-qualify-command-contracts.md), dependent on the authority decision and receipt/universe contracts | **Decision blocked.** Owns verifier behavior, not this threat model. |
| implement and operate `P` if selected | decision carrier creates a bounded protected-path/owner/rules/bypass/reviewer-rotation child, depending on the accepted decision and inventory | **Tom blocked.** Must include latest-push and bypass negative controls, outage/recovery runbook, and exact included paths; must not choose profile semantics, `M`, `K`, or `T`. Stop when host enforcement and recovery are evidenced or fail closed as unavailable. |
| implement and operate `M` if selected | decision carrier creates a bounded external classifier/attestation child, depending on the accepted decision, complete inventory, and selected `P`/`K` acceptance binding | **Tom/evidence blocked.** Must own exact path taxonomy, predecessor/successor binding, all class-pair perturbations, outage/rotation; must not claim semantic review. Stop if the population cannot be complete or enforcement remains repository-writable. |
| implement and operate `K` if selected | decision carrier creates a bounded five-class canonical-manifest, policy-signer threshold, client-root/verifier, rotation/revocation/recovery child after concrete identities and command contracts exist | **Tom/evidence blocked.** Must distinguish unauthorized automation from policy signing and exercise threshold loss/compromise; must not add transparency or hide signing inside profile assembly. Stop if canonical closure or independent root distribution is unresolved. |
| compose `P+K` if both are selected | decision carrier assigns the exact host-approval receipt/live-query binding to the `P` or `K` child, or creates a bounded composition child depending on both | **Tom/evidence blocked.** Must prove the qualifier checks both approvals over the same source identity and exercise A3-only and A6-only perturbations; must not credit two unbound workflows as defense in depth. Stop if no independently checkable host-approval condition exists. |
| implement and operate `T` if selected | decision carrier creates a bounded log/witness/monitor/content-retention child depending on the selected leaf authority owner: `P`, `K`, or their composition | **Tom/evidence blocked.** Must authenticate the exact base-approved leaf (`P` receipt, `K` signature, or both), own inclusion/consistency, checkpoint distribution, retained accepted leaves/content or authorized mirror, withholding/equivocation and deletion-recovery drills; must not claim leaf legitimacy. Stop if leaf authentication, independent witnessing, or recoverable content retention is absent. |
| exercise each subject perturbation and authority outage | [`design-the-conformance-denominator-and-receipt-perturbation-suite`](../../../tickets/design-the-conformance-denominator-and-receipt-perturbation-suite.md) | **Contract blocked.** Depends on authority, receipts, profile, and command contracts. |
| assemble `GoalProfileV1` | [`assemble-the-first-versioned-conformance-goal-profile`](../../../tickets/assemble-the-first-versioned-conformance-goal-profile.md) | **Blocked.** Must consume the accepted authority rather than choose it. |
| render authority and evidence without hiding Unknown | [`design-the-machine-readable-and-explorable-conformance-report`](../../../tickets/design-the-machine-readable-and-explorable-conformance-report.md) | **Blocked.** Report is a projection, never the authority. |

Immediate work can continue on inventory, obligation algebra, evidence semantics, and other already-dispatched research. `P`/`M`/`K`/`T` implementation and operations, plus accepted profile assembly, remain Tom/decision/evidence blocked. The carrier creates only the selected property children; unselected properties remain documented triggers rather than hidden work.

## Recommendation, adoption trigger, and unsupported threats

**Proposal.** For the narrow current A0/A1/A2 threat, adopt `P` after the downstream decision accepts exact paths and roles: protected pull-request review, fresh protected-owner approval after the latest authority push, no routine bypass, and protected ownership of the rule/ownership file. Combined authority/implementation diffs remain permitted and rely on owner judgment. Treat unavailable authority as `Unknown`/nonzero. Do not call this protection against a dishonest protected maintainer or host administrator.

**Standalone `K` alternative and trigger.** If A3/A4 coherent repository/host rewrite enters scope, an external consumer must validate independently of GitHub, or an observed rewrite occurs, require `K`: policy-approver-held threshold signing of a canonical manifest binding all five classes and client-root continuity. Do not require `P` merely to claim that signers reviewed; `K`'s designated signers own that policy review. Its trade is key/schema/recovery operations and later rejection of unauthoritative merged content instead of host protection.

**`P+K` trigger.** Require both only when the first profile must tolerate either host-policy authority compromise or signer-threshold compromise alone, or when premerge rejection is worth a second independent authority and outage path. Record whether administration and judgment are genuinely independent; duplicated credentials or the same unchecked human decision cannot be credited as defense in depth.

**Optional `M` decision.** If Tom accepts mechanical “no authority and implementation/evidence in one work item” as first-profile policy, add the trusted external classifier after inventory names complete path classes. `P+M` binds it at merge; `K+M` binds its attestation into signing/client acceptance; `P+K+M` does both. `M` blocks the coupled form, not a malicious selected policy approver using separate changes.

**Trigger for `T`.** Add witnessed transparency to the already selected base only when multiple independent consumers need non-equivocation, public audit of authority changes, durable detection of approval misuse, or witnessed history after repository replacement. `P+T`/`P+M+T` is valid while A3/host compromise remains excluded and an exact authenticated host-approval receipt can be constructed; selecting `T` does not itself select `K`. A log without monitoring/witnessing does not satisfy the trigger.

**Unsupported threats after `P`:** malicious/compromised protected policy reviewer, protected maintainer or host admin, repository host compromise with rules/history rewrite, mixed-diff reviewer confusion, and offline verification after that rewrite.

**Unsupported threats after `P+M`:** every `P` threat except the mixed-diff form; a malicious owner can split the same weakening across work items, and classifier/path-taxonomy compromise can suppress the check.

**Unsupported threats after `P+T` or `P+M+T`:** the corresponding `P`/`P+M` terminal host threats, malicious but valid host approvals, host-receipt/attestor compromise, missing retained accepted content after deletion, and denial of service. `T` detects witnessed history divergence; it does not add `K` host-rewrite resistance.

**Unsupported threats after `K`:** compromised/dishonest policy-signing threshold, compromised client-root distribution, canonicalization/resolver bugs, merged but unauthoritative repository state confusing non-authority-aware users, mixed-diff signer confusion, withholding, and two valid signed histories without a checkpoint.

**Unsupported threats after `K+M`:** every `K` threat except the mixed-work-item form; an authorized signer can approve separated weakening, and classifier/attestation compromise can suppress the check.

**Unsupported threats after `P+K`:** coalition or correlated compromise of protected host approval plus policy-signing threshold, compromised client-root distribution, canonicalization/resolver bugs, mixed-diff confusion, withholding, and two valid signed histories without a checkpoint.

**Unsupported threats after any `T` family:** coalition controlling the selected base approval authority/authorities plus leaf authenticator and every witness/checkpoint path, malicious but validly authorized/logged semantics, deletion when no independent accepted content/approval proof or mirror survives, and denial of service. `K` variants additionally trust client-root distribution and canonical resolution.

## Primary sources and what they establish

- [GitHub, About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches) — required pull-request/code-owner review, stale approval dismissal, latest-push approval, signed-commit option, force-push/deletion controls, and the provider's bypass model.
- [GitHub, About code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners) — recognized `CODEOWNERS` locations, protected-owner review, and the need to protect `CODEOWNERS` itself. Its path-owner rule requests/accepts review for touched owned paths; it does not describe a mutually exclusive two-path-class predicate, which is why `M` is a separate inferred mechanism rather than a claimed native feature.
- [GitHub, Creating rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository) — explicit bypass actors and ruleset operation.
- [The Update Framework specification v1.0.27](https://theupdateframework.github.io/specification/v1.0.27/) — root/role thresholds, offline root-key recommendation, versioned old-plus-new root continuity, rollback/freeze handling, rotation, and the out-of-band limit after threshold compromise.
- [Sigstore security model](https://docs.sigstore.dev/about/security/) — append-only transparency, signed tree state, inclusion/auditability, and the requirement for monitoring for long-term trust.
- [Sigstore threat model](https://docs.sigstore.dev/about/threat-model/) — monitors checking append-only behavior and cross-monitor consistency, plus TUF-root recovery for service compromise.
- [Transparency.dev, witness network](https://blog.transparency.dev/can-i-get-a-witness-network) — witnesses validate consistency and help detect split views; they do not validate leaf legitimacy.

No access control, paywall, or inaccessible primary source narrowed this packet.
