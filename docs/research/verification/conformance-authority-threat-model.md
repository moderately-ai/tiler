---
schema: "tiler-doc/v1"
id: "tiler.research.verification.conformance-authority-threat-model"
kind: "research"
title: "Conformance authority threat model"
topics: ["verification", "conformance", "authority", "security", "governance"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.correctness-and-testing"]
ticket: "cost-protected-review-versus-signed-conformance-authority"
---

# Conformance authority threat model

**Status:** complete research; authority decision pending

**Reviewed:** 2026-08-24 at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`

## Traceability

- **Work record:** [`cost-protected-review-versus-signed-conformance-authority`](../../../tickets/cost-protected-review-versus-signed-conformance-authority.md).
- **Complete threat model, measurements, sources, negative controls, and decision packet:** [Conformance authority threat model and decision packet](../../../spikes/verification/conformance-authority-threat-model/README.md).
- **Decision carrier:** [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md).
- **Governing contract informed:** [Correctness and testing](../../correctness-and-testing.md), whose independent-oracle and subject-perturbation requirements determine why a signed policy file alone is insufficient.

## Result

**Measurement.** At the exact research base, a tree-wide exact-name/prefix census carries no tracked workflow, `CODEOWNERS` in any GitHub-recognized location, tracked hook, or signing policy; the current GitHub `main` branch reported no branch protection and no ruleset; the current commit and latest 20 commits carried no Git signature status. This is a bounded current-state observation, not an absence claim about every external system.

**Fact.** The root conformance spike already rejects the claim that a repository can restrain an actor able to rewrite its profile, verifier, baseline, tests, and checks together. The accepted correctness contract requires independent oracles and subject perturbation; [ADR 0106](../../decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) denies the conformance member semantic and support-matrix authority. Therefore a green conformance report cannot be its own approval authority.

**Inference.** The protected object has five classes: owner-derived denominator, exception/profile policy, verifier rules/schema, oracle/reference authority, and accepted evidence-baseline lineage. An external signature that binds only profile and exceptions is bypassable by changing any of the other three classes, so it is not a sound survivor for the ticket's stated threat.

**First independent-review correction.** Protected combined owner review and a separate-work-item rule were incorrectly treated as distinct enforced powers. Native `CODEOWNERS`/branch protection requires owner approval for protected paths but does not reject a pull request because authority and implementation classes co-occur. The split gains a mechanical property only through a trusted external classifier/attestation with complete path coverage outside ordinary repository writes; even then it blocks a coupled work-item form, not a dishonest selected policy approver using separate changes.

**Second independent-review correction.** Automated signing and policy-approver-held threshold signing were incorrectly conflated. An automated signer over repository-selected state supplies integrity but no independent semantic judgment. By contrast, designated policy signers can review the semantic authority diff and make their threshold signatures the approval act. Standalone `K` is therefore sound against A0–A4 while its signer threshold, canonical resolver, and client root remain honest; `P+K` adds independent host-side rejection and compromise tolerance, not missing review quality.

**Final Pareto correction.** Witnessed transparency was incorrectly restricted to `K` leaves. `T` needs an authenticated accepted-authority leaf, not necessarily a policy-signing key. While A3/host compromise is excluded, `P` can authorize an exact commit/content closure and an independent witness can authenticate the exact latest-state host approval before logging it. `P+T` and `P+M+T` are therefore nondominated audit families that avoid `K` policy-key/manifest machinery while retaining `P`'s host trust limit.

**Proposal.** The current lowest-cost narrow first-profile placement remains protected combined owner review (`P`), conditional on protected-owner and host-admin compromise being out of scope. Standalone policy signing (`K`) is the smallest placement when coherent repository/host rewrite or independent client validation is required. `P+K` is retained when either independently controlled authority must survive the other's compromise or premerge rejection justifies the extra machinery. Independently add the trusted mixed-diff predicate (`M`) and witnessed transparency (`T`) to the selected base. The complete conditional product has twelve survivors: three bases `{P, K, P+K}` times optional `{M}` times optional `{T}`. Signed `K` content rejects corruption and retained-state rollback; every `T` family detects witnessed divergence, cannot prevent withholding, and cannot recover deleted bytes without retained accepted content/approval proof or an authorized mirror.

## Ordered decisions and explicit implementation ownership

Tom should answer one question at a time in the downstream decision ticket. First: **must `GoalProfileV1` resist protected-policy/host rewrite?** “No” selects `P`; “yes” introduces `K`. Second, whenever `K` is introduced: **must `P` also be a required acceptance condition?** “No” selects standalone `K`; “yes” selects independently controlled `P+K` and requires both approvals over the same exact source identity. Third: **must mixed authority-plus-implementation/evidence work items be mechanically rejected?** “No” relies on the selected policy approver's judgment; “yes” adds `M`. Fourth: **does the profile require witnessed non-equivocation/public audit/last-good history?** “No” defers `T`; “yes” adds `T` to the already selected `P`, `K`, or `P+K` base without changing it. If a later trigger introduces `K` into a `P` family, resume at the standalone `K` versus `P+K` question; never let `T` silently force `K`.

Research completion authorizes no hook, branch rule, signing key, schema, transparency service, CI, or profile implementation. After Tom selects properties, the decision carrier must create bounded implementation/operations children for every selected `P`, `M`, `K`, and `T` property. The `T` child depends on the selected leaf authority owner (`P`, `K`, or their composition) and must prove the concrete authenticated leaf/receipt path. The packet gives dependencies, authority, non-goals, evidence, recovery, and stop conditions. Unavailable authority remains fail closed.
