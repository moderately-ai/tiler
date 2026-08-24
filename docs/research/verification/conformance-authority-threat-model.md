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

**Measurement.** At the exact research base, the repository carries no tracked workflow, `CODEOWNERS`, active hook, or signing policy; the current GitHub `main` branch reported no branch protection and no ruleset; the current commit and latest 20 commits carried no Git signature status. This is a bounded current-state observation, not an absence claim about every external system.

**Fact.** The root conformance spike already rejects the claim that a repository can restrain an actor able to rewrite its profile, verifier, baseline, tests, and checks together. The accepted correctness contract requires independent oracles and subject perturbation; [ADR 0106](../../decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) denies the conformance member semantic and support-matrix authority. Therefore a green conformance report cannot be its own approval authority.

**Inference.** The protected object has five classes: owner-derived denominator, exception/profile policy, verifier rules/schema, oracle/reference authority, and accepted evidence-baseline lineage. An external signature that binds only profile and exceptions is bypassable by changing any of the other three classes, so it is not a sound survivor for the ticket's stated threat.

**Proposal.** The smallest sound first-profile placement is externally enforced protected review plus distinct policy/implementation approval lanes, conditional on protected-maintainer and host-admin compromise being out of scope. If an actor able to rewrite and merge every repository authority is in scope, add an independently signed, versioned manifest binding all five classes. Add witnessed transparency only when non-equivocation, public audit, or post-key-compromise history is required.

## The one decision

Tom should answer one threat-scope question in the downstream decision ticket: **is an actor who can rewrite and merge every repository authority an in-scope adversary for `GoalProfileV1`?** “No” selects protected split review and an explicit trigger for stronger authority. “Yes” selects the independent signed five-class manifest before qualification can make an accepted claim.

Research completion authorizes no hook, branch rule, signing key, schema, transparency service, CI, or profile implementation. The packet maps those steps to the existing conformance-progress graph and keeps unavailable authority fail closed.
