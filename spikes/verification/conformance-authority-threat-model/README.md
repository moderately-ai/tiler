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

**Corrected verdict after independent review.** The smallest sound first-profile authority for accidental contributors, repository-writing automation, and an ordinary implementation reviewer is protected combined review: every change touching a protected authority class requires fresh approval from an honest protected policy owner. A separate-work-item rule adds no enforcement by itself, and GitHub `CODEOWNERS`/branch protection does not natively reject a pull request merely because it mixes authority and implementation paths. The earlier packet incorrectly eliminated protected combined review and credited the split with a power it did not have.

**Conditional frontier.** Four independent properties create the real frontier. `P` is protected combined review. `M` is an optional trusted external mixed-diff check that rejects any work item containing both authority and implementation/evidence path classes; it adds mechanically enforced subject separation, not protection from a dishonest protected policy owner. `K` adds a versioned, independently signed authority manifest that transitively binds all five authority classes and protects acceptance after coherent repository rewrite while its key/root survive. `T` adds independently witnessed append-only publication when post-publication equivocation, repository-history replacement, or signing-key misuse must be detectable. The products `P`, `P+M`, `P+K`, `P+M+K`, `P+K+T`, and `P+M+K+T` are nondominated only under the corresponding threat/strictness requirement.

**Recommendation under current evidence.** Start with `P`, protected combined review, while explicitly excluding a dishonest protected policy owner and rules administrator/bypass actor. It is the lowest-cost placement that prevents A0/A1/A2 from self-authorizing an authority rewrite. Add `M` only if Tom accepts mechanical mixed-diff prohibition as a first-profile policy; add `K` when repository/host-authorized rewrite enters scope or external consumers must validate independently; add `T` when non-equivocation or witnessed last-good recovery enters scope.

**Two ordered decisions for Tom, asked one at a time in the downstream carrier.**

1. Is an actor who can rewrite and merge every repository authority an in-scope adversary for `GoalProfileV1`? **No** selects the `P` family; **yes** requires the `P+K` family before an accepted `qualify` claim.
2. Within the selected family, must the first profile mechanically reject every mixed authority-plus-implementation/evidence work item? **No — recommended absent contrary evidence** uses protected owner judgment; **yes** adds `M`, a required check whose classifier, configuration, and enforcement are outside ordinary repository-writer authority.

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
- A4 defeats every local negative control even if all of them continue to pass. An external protected reviewer can restrain A4 only while A4 lacks A2/A3 authority; an external signature root can restrain A4 only while A4 lacks A6 and cannot replace the verifier's pinned root.
- A5 corruption is not the same as A5 withholding: `K` rejects altered or substituted bytes and a rollback relative to client-retained monotone version state, but no signature can make a withholding store available. Freeze detection additionally needs an accepted expiry/freshness rule and trusted time; before expiry, a store can hide an unseen update without the client knowing one exists.
- A5 equivocation is not ordinary corruption. `K` can authenticate each of two signer-authorized histories and therefore does not prove they are globally unique. A log without independent monitoring or a witnessed checkpoint is not a non-equivocation authority; compromise of the log plus every witness/checkpoint holder defeats that property.
- A6 defeats signature prevention for its threshold. Transparency may make misuse attributable and detectable after monitoring; it cannot make an attacker-signed malicious policy invalid merely because it is public.

## Threat and power matrix

`P` means prevents acceptance under the stated assumptions, `D` detects after or during the attempt, `A` supplies attributable evidence, `R` has a defined recovery path, and `—` supplies none. Cells name the smallest external placement capable of the property: `PR` is protected combined owner review, `MIX` is the separately trusted mixed-diff classifier required by the host, `SIG` is the independently signed five-class manifest with retained monotone version state, and `LOG` is append-only publication plus an independently held witnessed checkpoint and monitor.

| attack | A0 | A1 | A2 | A3 | A4 | A5 | A6 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| shrink denominator or delete tombstone | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A if no key | SIG: P/D/A | — | LOG: D/A only |
| weaken required/optional/N/A or exception | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A if no key | SIG: P/D/A | — | LOG: D/A only |
| weaken verifier or report schema | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A only if verifier identity is bound | SIG: P/D/A only if verifier identity is bound | — | LOG: D/A only |
| replace independent oracle or tolerance | PR: P/D/A | PR: P/D/A | distinct oracle/policy approver: P/D/A | SIG: P/D/A only if oracle identity is bound | SIG: P/D/A only if oracle identity is bound | — | LOG: D/A only |
| reset evidence baseline to weaker evidence | PR: P/D/A | PR: P/D/A | distinct policy approver: P/D/A | SIG: P/D/A only if lineage is bound | SIG: P/D/A only if lineage is bound | LOG: D/A/R from retained checkpoint | LOG: D/A only |
| combine authority and implementation/evidence in one work item | MIX: P/D/A | MIX: P/D/A | MIX: P/D/A | MIX blocks the form if no bypass; separate malicious updates remain | MIX blocks the form while external; separate malicious updates remain | — | — |
| rewrite repository history and all local checks coherently | protected host: P/A | protected host: P/A | protected policy owner: P/A | SIG: P/D/A | SIG: P/D/A | signed/checkpointed recovery material: R | LOG: D/A only |
| corrupt or substitute stored authority bytes | — | — | — | — | — | SIG: P/D; no unique attribution | — |
| serve rollback/frozen authority state | — | — | — | — | — | SIG retained version: P/D rollback; expiry/freshness: eventual D freeze | — |
| withhold authority or log data | reviewer outage: D, fail closed | reviewer outage: D, fail closed | alternate reviewer: R | cached SIG verifies an existing valid claim only | cached SIG verifies an existing valid claim only | no availability P; timeout/expiry D; publication stops | surviving threshold/recovery keys: R |
| present two signer-authorized histories to different clients | PR audit only: D if observed | PR audit only: D if observed | PR audit only: D if observed | SIG authenticates both; no non-equivocation | SIG authenticates both; no non-equivocation | LOG+witness: D/A | LOG+witness: D/A |

The matrix is deliberately asymmetric. A protected maintainer who lacks the signing key is constrained by `SIG`; a reviewer or signer who is malicious within their accepted role is not made honest by tooling. `MIX` prevents a coupled subject form and makes the two reviews separately attributable; it does not stop A3 from approving the same semantic weakening in two work items. `SIG` supplies stored-content integrity and rollback rejection relative to retained state, but not store availability. Recovery is discussed separately because restoring service and deciding which state was last good are different problems.

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

### M — trusted external mixed-diff exclusion, always combined with P

`M` is not `CODEOWNERS` and is not a repository-local script that a pull request can edit. It is a host-required check or equivalently trusted service whose classifier, configuration, and enforcement are outside ordinary repository-writer authority. It classifies the eventually accepted five authority path sets and implementation/evidence path sets, fails when one work item touches both populations, prints the exact paths/classes, and cannot be bypassed by A0/A1/A2. The path population cannot be configured until the inventory and command-contract work identifies the concrete owners.

- **Prevention/detection/attribution/recovery:** prevents the specific coupled-subject form in which A0/A1/A2 modifies implementation/evidence and its approval authority in the same work item, even when reviewers miss the coupling. It creates two separately reviewable and attributable histories. It does not stop A3 from approving the same malicious semantic change in two work items, and it adds no protection against an authorized bypass/rules administrator unless that actor is outside the check's trust boundary.
- **Cost/offline/availability:** all `P` cost plus an external classifier/service, a complete maintained path taxonomy, required-check configuration, negative controls for each path class, and service availability on every relevant merge. Offline development remains possible; merge blocks while the check is unavailable. A repository-local classifier does not meet this definition for A4.
- **Recovery/rotation:** rotate the service identity/configuration through the host authority, preserve known-good classifier versions, re-evaluate open changes after classifier repair, and require post-review for any emergency bypass. A stale or incomplete path population is a fail-closed defect, not a reason to treat an unmatched authority path as implementation.
- **Unavoidable trust:** the classifier and its configuration, external required-check enforcement, complete path ownership, protected owner judgment on the separated authority change, and host bypass control.
- **Strongest counterargument:** it adds a service and path taxonomy but no new protection against a dishonest protected policy owner; its gain is review isolation and mechanical compliance with a proposed process invariant.
- **Reversal evidence:** subject perturbations in which mixed diffs repeatedly escape `P` review, or Tom's acceptance of “no authority and implementation in one work item” as a normative first-profile rule, justify `M`. Evidence that the path taxonomy cannot be complete or that the service availability dominates profile work argues for `P` alone.
- **Disposition:** retained only as the optional `P+M` strictness frontier; not recommended by default on current evidence.

### S3 — independent signed authority manifest alone

Clients pin an external public root and accept only a monotonically versioned canonical manifest signed by the required role threshold. The manifest binds the exact identities/digests of the five classes, the schema and resolver version, role/threshold metadata, validity/expiry policy, and predecessor/root version. A TUF-style root is the reference design vocabulary, not a proposal to import TUF wholesale: TUF specifies offline root keys, threshold signatures, versioned root continuity signed by old and new thresholds, rollback/freeze checks, and out-of-band recovery if a root threshold is compromised.

- **Prevention/detection/attribution/recovery:** prevents A0–A4 from authorizing changed bytes while they lack the threshold keys, even if they rewrite the repository. Against A5 it rejects corrupted/substituted stored bytes and rejects a rollback older than client-retained monotone version state; an accepted expiry/freshness rule can eventually expose freeze. It authenticates signer approval but does not supply review quality, separation of duties, availability, or non-equivocation; A6 can sign a malicious manifest, and two valid signer-authorized histories can exist without an external checkpoint.
- **Cost/offline/availability:** canonicalization, manifest/resolver schema, client pinning and retained version state, key ceremony/storage, signer tooling, verifier integration, rotation drills, freshness/expiry policy if freeze detection is required, and recovery documentation. Existing signed manifests can verify offline from cached roots; new authority publication requires threshold keys. A withholding store can still block updates, so alternate distribution or cached valid state is an availability decision rather than a signature property. No kernel cost; audit/qualification adds fixed manifest parsing plus digest/signature verification, not measured here.
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

### K — independent signed five-class-manifest add-on, combined with P

`P+K = P+S3`, with the external policy signer distinct from ordinary repository-writing automation and from the repository host authority whose compromise is in scope. The signer signs a reviewed canonical manifest; the signature does not replace review. A threshold design is strongly preferred if the protected-maintainer threat is genuinely in scope, because a one-key system simply moves all authority to one secret. `P+M+K` adds mixed-diff exclusion as an orthogonal strictness property; `K` does not silently inherit it.

- **Prevention/detection/attribution/recovery:** adds prevention against A3/A4 repository rewrites while the signer threshold and client anchor survive. It attributes manifest authorization to a key/role. It does not detect two signer-authorized histories without a checkpoint and cannot protect a compromised signing threshold.
- **Cost/offline/availability:** all `P` cost plus schema/canonicalization, pinning, keys, verifier work, rotation and recovery drills. Existing accepted manifests verify offline. New qualification authority is unavailable when threshold keys are unavailable; this fails closed rather than falling back to repository HEAD or an unsigned profile. `P+M+K` additionally pays the external mixed-diff service and path-taxonomy cost.
- **Unavoidable trust:** policy reviewers, signer threshold, custody/recovery root, client anchor distribution, canonical resolver, and host for availability.
- **Strongest counterargument:** key loss or premature schema binding can stall a research project more reliably than the threat it mitigates.
- **Reversal evidence:** if Tom puts A3/A4 inside `GoalProfileV1` scope, or an external consumer must validate independently of GitHub, this option becomes necessary rather than optional. If the universe work cannot yet name stable five-class identities, bounded schema research must finish before adoption.

### T — witnessed-transparency add-on, combined with P+K

`P+K+T = P+K+S4`. Publish every accepted manifest, rotation, revocation, and recovery event; require a witnessed checkpoint/inclusion bundle before the update becomes an accepted qualification authority.

- **Prevention/detection/attribution/recovery:** preserves `P+K` content-integrity and retained-version rollback protection, then adds durable independent detection/attribution for log deletion, equivocation/split histories, and visible key misuse after monitoring. Recovery can identify a witnessed last-good state. It does not make a withholding service available, cannot make malicious reviewed/signed content correct, and fails if the signer threshold plus log/witness trust set are compromised. `P+M+K+T` adds mixed-diff exclusion separately.
- **Cost/offline/availability:** highest operational and external availability cost. Existing bundles verify offline; new authority publication blocks while required log/witness quorum is unavailable. The log adds no Tiler runtime/kernel cost but adds network and monitoring cost to authority publication and qualification.
- **Unavoidable trust:** everything in `P+K` plus log/witness keys, checkpoint distribution, monitor operation, and incident response.
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
| P / S1 protected combined review | **Frontier:** lowest host/operations cost that stops A0/A1/A2 while the protected policy owner and rules administrator are trusted; mixed diffs rely on owner judgment. |
| S2 unprotected prose/ticket split | **Eliminated:** repository writer can erase or ignore the split, and native `CODEOWNERS` review does not reject mixed path classes. |
| P+M trusted mixed-diff exclusion | **Strictness frontier:** mechanically prevents A0/A1/A2 mixed authority-plus-implementation work items and makes the histories separately attributable; higher service/path-taxonomy/availability cost and no added protection against dishonest A3. |
| profile/exception-only signature | **Eliminated:** verifier/oracle/denominator/baseline bypass keeps signature valid. Silently incomplete protection is a correctness defect. |
| S3 signed manifest alone | **Eliminated:** authenticates a signer but supplies no independent semantic review; `P+K` adds required judgment control. |
| S4 log alone | **Eliminated:** records bad leaves as faithfully as good ones and cannot authorize policy. |
| P+K protected review plus independent signed manifest | **Threat-coverage frontier:** protects acceptance from A3/A4 repository/host rewrite while key/root survive; higher key/schema/recovery cost and no mixed-diff prohibition unless `M` is added. |
| P+M+K | **Combined frontier:** carries both the mixed-diff property and signed-root threat coverage; pays both costs and is not credited with either review quality or non-equivocation. |
| P+K+T | **Audit frontier:** adds witnessed non-equivocation, independently durable history, and last-good recovery evidence; greatest external service/availability cost and no mixed-diff prohibition unless `M` is added. |
| P+M+K+T | **Maximum surviving combination:** carries every frontier property and every associated trust/availability burden. |
| bounded research/deferral | **Not an accepted-authority candidate:** safe only while qualification remains unavailable and reports `Unknown`/nonzero. |

No one frontier option dominates across threat coverage, mechanical subject separation, and operations. `P` is the current recommendation because it is sound for A0/A1/A2 under an explicit honest-protected-owner boundary and nothing measured establishes that a mixed-diff service is worth its new availability/maintenance authority. `P+M` is stricter but does not stop a dishonest owner; this is the real tradeoff the earlier packet hid. Once A3/A4 is in scope, the `P+K` family becomes the smallest sound threat-coverage family. `T` remains nondominated only when independently witnessed non-equivocation/history recovery is required. `M` is an orthogonal Tom-selected policy in any family, not a prerequisite smuggled into `K` or `T`.

## Cost, offline, unavailability, and recovery comparison

No wall-time or monetary cost was measured because no mechanism was installed and provider/key choices are undecided. The table therefore reports operations and asymptotic work, not fabricated timing.

| property | P protected combined review | P+M mixed-diff exclusion | P+K signed-root family | P+K+T transparency family |
| --- | --- | --- | --- | --- |
| one-time setup | ruleset/protected paths, protected owner, stale-approval and bypass policy | `P` plus trusted external classifier, complete two-class path taxonomy, required-check configuration and negative controls | `P` plus canonical five-class manifest, client root/version state, signing roles/thresholds, key ceremony, recovery drill | `P+K` plus log, inclusion/consistency verification, monitor/witness, checkpoint distribution, incident runbook |
| routine authority update | combined diff permitted; protected owner freshly approves every authority change | authority and implementation/evidence must use separate work items; both pass external classification | reviewed manifest plus threshold signing and repository publication; mixed diff permitted unless `M` is added | signed publication plus log inclusion and witnessed checkpoint; mixed diff permitted unless `M` is added |
| repository size | ownership/policy text and host config reference | classifier version/reference and path taxonomy; external implementation remains outside writer authority | manifest, signatures, roots, rotations; bounded by retained authority versions | inclusion bundles/checkpoints or references; history grows with updates |
| Tiler host cost | review-time only; audit reads accepted local state | `P` plus one required external classification per merge; unmeasured | fixed canonical parse/digests/signature verification during audit/qualify; unmeasured | `P+K` plus inclusion/checkpoint verification and optional network fetch; unmeasured |
| runtime/kernel cost | none | none | none if authority is checked before build/qualification | none if authority is checked before build/qualification |
| offline development | full; cannot accept an authority change offline | full; merge cannot complete without the check | full against cached root/manifest; cannot mint a new accepted manifest without threshold | full against cached inclusion bundle/checkpoint; cannot publish a new accepted version offline |
| authority unavailable | block authority-changing merge/qualification; keep last accepted version | block all classified relevant merges; no fallback to reviewer-only path while `M` is required | verify last valid cached version; block new publication, rotation, or qualification identity | verify last valid cached bundle; block new accepted publication until required inclusion/witness proof exists |
| reviewer recovery | rotate protected team/account through host owner; post-review emergency bypass | same plus re-run classifier on every bypassed/open change | same | same |
| key loss/compromise | host credentials are the effective authority | classifier service/configuration credentials add another authority | rotate below threshold; old+new threshold continuity; out-of-band recovery after threshold compromise | same, plus log revocation/rotation event and monitor alert |
| stored-content corruption | ordinary Git object/hash checks only; no independent root after coherent rewrite | same | signature/digest rejects altered/substituted bytes | same plus independently witnessed inclusion for accepted leaf |
| rollback/freeze | retained clones/review history only | same | retained version rejects rollback; accepted expiry/freshness eventually detects freeze | same plus witnessed history/checkpoints improve last-good attribution |
| withholding | no prevention; host outage blocks merge | no prevention; classifier outage blocks merge | no prevention; cached valid state may continue within policy, but updates stop | no prevention; cached valid bundle may continue within policy, but publication stops |
| equivocation | no independent non-equivocation property | same | signatures authenticate each signer-authorized history; do not show global uniqueness | witness/monitor detects inconsistent histories unless trust set is compromised |

### Fail-closed availability rules

1. Unavailable protected reviewer: ordinary research/implementation may continue on branches, but no policy/exception/oracle/verifier/baseline update becomes accepted and `qualify` cannot cite it.
2. Unavailable `M` classifier: if mixed-diff exclusion is required, relevant merges block; no fallback to protected-owner-only review is allowed until Tom changes the accepted policy.
3. Unavailable signing threshold: the last unexpired/policy-valid cached authority can still verify; no unsigned or repository-default replacement is accepted. If expiry is adopted, expiry yields a named unavailable/unknown authority, not automatic extension.
4. Unavailable transparency service: if `T` is required, a new manifest remains pending until an inclusion proof and required witnessed checkpoint exist. Existing offline bundles remain verifiable under their stated validity policy.
5. Recovery bypass: recovery changes the root of trust; it cannot be treated as an ordinary implementation merge. An emergency repository merge without restored authority may repair code but cannot mint a conformance-green claim.

## Independent subject perturbations

These are required tests for the eventual mechanism; none was implemented in this research ticket. Each perturbation changes the protected subject, not the assertion, and each expected message names the independent property that must reject. The downstream [perturbation-suite ticket](../../../tickets/design-the-conformance-denominator-and-receipt-perturbation-suite.md) already owns executable realization after the authority decision and command contracts.

| protected property | subject perturbation | expected `P` result | additional `M` result | additional `K`/`T` result |
| --- | --- | --- | --- | --- |
| denominator closure | add an owner-registered feature without a universe disposition, then separately delete an existing tombstone | protected owner approval required; `audit` names missing/disappeared key | if paired with implementation/evidence, classifier rejects and prints both path classes | manifest digest/identity mismatch; `T` also lacks accepted inclusion for successor |
| exception strictness | change one required cell to optional/N/A and separately remove its reason/expiry | protected owner must approve; `regress` names authority change rather than progress | mixed implementation/exception work item is rejected; an exception-only work item proceeds to owner review | signature failure until reviewed successor; successor version must advance |
| verifier strictness | change `qualify` to treat missing/Unknown receipt as success | verifier path requires protected owner approval; negative case must report exact missing obligation | mixed verifier/implementation work item is rejected | bound verifier identity differs even when profile bytes do not |
| oracle independence | replace a hand-written expected result with output recomputed by the implementation, then separately widen a tolerance | oracle path requires protected owner approval; oracle perturbation must fail expected comparison | mixed oracle/implementation work item is rejected | bound oracle/tolerance identity differs while implementation identity remains unchanged |
| evidence monotonicity | replace accepted baseline with a receipt set missing one formerly sufficient case | protected owner approval required; `regress` prints lost cell | mixed implementation/baseline work item is rejected | lineage/digest and successor version differ; old signed baseline remains recoverable |
| mixed-diff classification | in one work item edit one member of each protected authority class together with one implementation/evidence path, separately per class | protected owner may approve; this deliberately shows `P` has no mechanical mixed-diff property | required check rejects every subject pair and prints both matched classes; removing one class makes it pass to review | no additional result unless signed manifest changes |
| latest-push review | obtain approval, then push protected-authority edit | stale approval is dismissed and merge blocked | classifier reruns against new diff | new content also lacks accepted manifest signature |
| repository rewrite | change all five local authorities and every local check so suite reports green | `P` fails only if host protection/approver is outside attacker power; this is explicit limit | external classifier may reject mixed form but cannot stop same rewrite separated across work items | pinned root rejects rewritten unsigned/untrusted manifest |
| stored-byte corruption | flip one byte of a signed manifest without changing signature | no independent result after coherent repository rewrite | no independent result | `K` reports digest/signature failure before semantics; no store attribution claimed |
| rollback | serve an older, otherwise valid manifest after client retained a newer version | no independent result after history rewrite | no independent result | monotone retained version rejects rollback |
| freeze | serve last valid manifest until its accepted freshness/expiry boundary passes | no independent result unless host history observed | no independent result | before boundary update may be unknowably hidden; at boundary typed stale/freeze result blocks new claim |
| withholding | make authority/signing/log service unreachable in separate trials | no accepted authority-changing merge | no classified merge; no reviewer-only fallback | cached valid authority may verify within policy; new accepted publication fails unavailable, and no availability prevention is claimed |
| signing-key compromise | sign malicious five-class manifest with accepted threshold | `P` may reject if independent review remains uncompromised | mixed form may be rejected; separated malicious work remains | `K` signature checks pass; `T` records/attributes event but does not prevent it |
| log equivocation | present two valid inclusion histories to separate clients | no dedicated result | no dedicated result | independent witness/checkpoint consistency fails; lone log may pass both views |

Each independent property needs its own perturbation and captured failure text. A single patch that changes all five subjects proves only that something is guarded. Counts must come from the concrete typed registries after the inventory work, not from a hand-written expected total.

## Recovery cases and unavoidable trust

### Reviewer or host-account compromise

Revoke the account/team membership, remove bypass, invalidate pending approvals by requiring fresh review after the latest push, inspect provider audit history, and re-review every authority version approved during the exposure window. The last known-good authority must be identified independently of the compromised reviewer's assertion. `P` and `P+M` have no external cryptographic answer if the same host/history was rewritten; that is their accepted threat limit.

### Signing-key loss below threshold

Use surviving authorized keys to publish a versioned successor removing the lost key and adding a replacement. A TUF-style transition is signed by both the old and new accepted thresholds so older clients can verify continuity. Test this ceremony before depending on it; a policy that has never demonstrated recovery has an unknown recovery property.

### Signing threshold compromise

Assume malicious manifests may have been validly signed. Stop new qualifications, identify a last-good version through independent review and any witnessed record, distribute a new root out of band, and re-qualify descendants. TUF's specification warns that recovery after threshold root compromise is nearly impossible for already affected unattended clients; this packet therefore does not claim automatic recovery.

### External store corruption or substitution

Reject any bytes whose bound digest/signature does not verify under the retained root. Fetching from another store restores availability if one exists; it does not change which bytes are authoritative. The verification failure does not uniquely attribute corruption to the store because transport, disk, and caller substitution can produce the same invalid bytes.

### External rollback or freeze

Reject a manifest older than client-retained monotone version state. Detecting freeze requires an accepted freshness/expiry rule plus trusted time; before that boundary, a client cannot infer that the store hid an update it never observed. Recover by fetching a newer valid successor from another distribution path or continue only under the explicitly valid cached-authority policy.

### External withholding

Signatures do not make a service available. Use cached valid authority only within its accepted validity policy, try an independently authorized mirror if designed, and block new publication/qualification once freshness or required inclusion cannot be established. Do not downgrade from `P+K`/`P+K+T` to repository HEAD or from `P+M` to reviewer-only merge because the external authority is unavailable.

### External equivocation or log compromise

Two signer-authorized histories may both pass `P+K`; detect their divergence only relative to independent monitors/witnesses/checkpoints. Restore service from signed manifests plus checkpoints held outside the compromised store and quarantine the log. If the attacker also controls every checkpoint/witness and the clients' root distribution, no mechanism in this frontier supplies recovery.

### Unavoidable trust statement

Every option ends at some authority. `P` trusts human judgment and host access control. `M` additionally trusts an external classifier, complete path taxonomy, and required-check availability. `K` additionally trusts canonical resolution, a signer threshold, custody/recovery, retained client state, and the clients' pinned root. `T` additionally trusts monitors/witnesses and checkpoint distribution. None protects against a coalition controlling all of its terminal authorities, and none establishes semantic correctness without the independent conformance oracles required by the correctness contract.

## External-authority limit

This limit is normative for interpreting the packet: a repository-local check can constrain only actors who cannot rewrite, replace, or skip it. If an actor can rewrite every repository authority and the expected results together, a green repository report is merely an attacker-chosen fixed point. An external rule or key helps only while the attacker cannot also change the rule, exercise its bypass, satisfy its signing threshold, replace the client's trust anchor, or control every transparency witness. Claims beyond that boundary are unsupported, not “best effort.”

## Follow-up graph: nothing implicit

No new descendant ticket is required. The existing conformance-progress graph already separates every necessary next step, so duplicating it would create competing authorities.

| work | existing owner and dependency | immediate or blocked |
| --- | --- | --- |
| enumerate concrete universe/profile authority objects and protected populations | [`inventory-the-closed-world-conformance-claim-universe-by-owner`](../../../tickets/inventory-the-closed-world-conformance-claim-universe-by-owner.md) | **Immediate research; in progress.** Supplies the path/identity population this packet deliberately does not invent. |
| choose the `P` versus `P+K` threat family, then whether to add `M`, and define who may approve authority changes | [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md), depending on this ticket and the inventory | **Tom decision/evidence blocked.** Must ask the two ordered questions above and cannot name a complete path classifier before inventory. |
| define fail-closed `audit`/`regress`/`qualify` semantics and machine failures | [`design-the-conformance-audit-regress-and-qualify-command-contracts`](../../../tickets/design-the-conformance-audit-regress-and-qualify-command-contracts.md), dependent on the authority decision and receipt/universe contracts | **Decision blocked.** Owns verifier behavior, not this threat model. |
| bind identities for signed manifest if `K` is selected | authority decision ticket must create a bounded signature-schema/recovery child after the five concrete identities exist | **Tom/evidence blocked and not authorized now.** No key, schema, signing code, or host configuration belongs in profile implementation. |
| exercise each subject perturbation and authority outage | [`design-the-conformance-denominator-and-receipt-perturbation-suite`](../../../tickets/design-the-conformance-denominator-and-receipt-perturbation-suite.md) | **Contract blocked.** Depends on authority, receipts, profile, and command contracts. |
| assemble `GoalProfileV1` | [`assemble-the-first-versioned-conformance-goal-profile`](../../../tickets/assemble-the-first-versioned-conformance-goal-profile.md) | **Blocked.** Must consume the accepted authority rather than choose it. |
| render authority and evidence without hiding Unknown | [`design-the-machine-readable-and-explorable-conformance-report`](../../../tickets/design-the-machine-readable-and-explorable-conformance-report.md) | **Blocked.** Report is a projection, never the authority. |

Immediate work can continue on inventory, obligation algebra, evidence semantics, and other already-dispatched research. Host protection, signing, transparency, and accepted profile assembly remain Tom/decision/evidence blocked.

## Recommendation, adoption trigger, and unsupported threats

**Proposal.** For `GoalProfileV1`, adopt `P` after the downstream decision accepts the concrete included/excluded authority paths and roles: protected pull-request review, fresh protected-owner approval after the latest authority push, no routine bypass, and protected ownership of the rule/ownership file. Combined authority/implementation diffs remain permitted and explicitly rely on owner judgment. Treat unavailable authority as `Unknown`/nonzero. Do not call this protection against a dishonest protected maintainer or host administrator.

**Optional `M` decision.** If Tom accepts mechanical “no authority and implementation/evidence in one work item” as first-profile policy, add a trusted external mixed-diff required check after the inventory names complete path classes. The classifier and enforcement cannot be ordinary repository-writable code. `M` blocks that coupled form, not a malicious A3 who makes the same change in separate work items.

**Trigger for `K`.** Before the first external release/compliance claim, before an external consumer must validate independently of GitHub, when more than one actor may merge protected authority, when a protected rules bypass/admin compromise enters scope, or after any observed coherent history/authority rewrite, require a bounded signed-manifest design and recovery exercise. The manifest must bind all five authority classes and client-root continuity.

**Trigger for `T`.** Add witnessed transparency only when multiple independent consumers need non-equivocation, public audit of authority changes, durable detection of signer misuse, or recovery from repository-history replacement. A log without monitoring/witnessing does not satisfy the trigger.

**Unsupported threats after `P`:** malicious/compromised protected policy reviewer, protected maintainer or host admin, repository host compromise with rules/history rewrite, mixed-diff reviewer confusion, and offline verification after that rewrite.

**Unsupported threats after `P+M`:** every `P` threat except the mixed-diff form; a malicious owner can split the same weakening across work items, and classifier/path-taxonomy compromise can suppress the check.

**Unsupported threats after `P+K`:** compromised signer threshold, compromised/out-of-band client root distribution, malicious policy reviewers plus signer threshold, canonicalization/resolver bugs, and two valid signed histories without a checkpoint.

**Unsupported threats after `P+K+T`:** coalition controlling signer threshold plus every required witness/checkpoint path, compromised client root distribution, malicious but validly reviewed/signed/logged semantics, and denial of service.

## Primary sources and what they establish

- [GitHub, About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches) — required pull-request/code-owner review, stale approval dismissal, latest-push approval, signed-commit option, force-push/deletion controls, and the provider's bypass model.
- [GitHub, About code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners) — recognized `CODEOWNERS` locations, protected-owner review, and the need to protect `CODEOWNERS` itself. Its path-owner rule requests/accepts review for touched owned paths; it does not describe a mutually exclusive two-path-class predicate, which is why `M` is a separate inferred mechanism rather than a claimed native feature.
- [GitHub, Creating rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository) — explicit bypass actors and ruleset operation.
- [The Update Framework specification v1.0.27](https://theupdateframework.github.io/specification/v1.0.27/) — root/role thresholds, offline root-key recommendation, versioned old-plus-new root continuity, rollback/freeze handling, rotation, and the out-of-band limit after threshold compromise.
- [Sigstore security model](https://docs.sigstore.dev/about/security/) — append-only transparency, signed tree state, inclusion/auditability, and the requirement for monitoring for long-term trust.
- [Sigstore threat model](https://docs.sigstore.dev/about/threat-model/) — monitors checking append-only behavior and cross-monitor consistency, plus TUF-root recovery for service compromise.
- [Transparency.dev, witness network](https://blog.transparency.dev/can-i-get-a-witness-network) — witnesses validate consistency and help detect split views; they do not validate leaf legitimacy.

No access control, paywall, or inaccessible primary source narrowed this packet.
