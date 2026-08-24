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

**Independent-review correction.** Protected combined owner review and a separate-work-item rule were incorrectly treated as distinct enforced powers. Native `CODEOWNERS`/branch protection requires owner approval for protected paths but does not reject a pull request because authority and implementation classes co-occur. The split gains a mechanical property only through a trusted external required check with a complete classifier outside ordinary repository writes; even then it blocks a coupled work-item form, not a dishonest protected owner who submits separate changes.

**Proposal.** The current lowest-cost first-profile placement is protected combined owner review, conditional on protected-owner and host-admin compromise being out of scope. A trusted external mixed-diff check is an orthogonal strictness choice, not a prerequisite. If an actor able to rewrite and merge every repository authority is in scope, add an independently signed, versioned manifest binding all five classes. Add witnessed transparency only when non-equivocation, public audit, or post-key-compromise history is required. Signed content rejects corruption and rollback relative to retained version state; it cannot prevent external-store withholding.

## The two ordered decisions

Tom should answer one question at a time in the downstream decision ticket. First: **is an actor who can rewrite and merge every repository authority an in-scope adversary for `GoalProfileV1`?** “No” selects the protected-review family; “yes” adds the independent signed five-class manifest before qualification can make an accepted claim. Second, within that family: **must the first profile mechanically reject every mixed authority-plus-implementation/evidence work item?** “No” relies on protected owner judgment and is recommended absent contrary evidence; “yes” adds the trusted external mixed-diff classifier and its availability/maintenance authority.

Research completion authorizes no hook, branch rule, signing key, schema, transparency service, CI, or profile implementation. The packet maps those steps to the existing conformance-progress graph and keeps unavailable authority fail closed.
