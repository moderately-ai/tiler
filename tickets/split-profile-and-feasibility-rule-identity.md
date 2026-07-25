---
id: split-profile-and-feasibility-rule-identity
title: Split ProfileIdentity into a target profile identity and a feasibility rule set identity
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler, feasibility, identity]
---
`tiler-compiler` fuses two governed vocabularies into one struct where `tiler-artifact` keeps them apart.

**Fact — `crates/tiler-compiler/src/feasibility.rs:194-213`.** `ProfileIdentity { key: &'static str, version: u32 }`, whose accessors are documented as "The governed profile key" and "The feasibility-rule version".

**Fact — `crates/tiler-compiler/src/physical.rs:666`.** It is built as `ProfileIdentity::new(target.key, PROTOTYPE_FEASIBILITY_RULE_VERSION)`, so the key names the target profile and the version names the feasibility rules. Two different things, one struct.

**Fact — the artifact model keeps them separate.** `TargetProfileRef { key, descriptor }` and `FeasibilityRuleSetRef { key, revision }` are independent (`crates/tiler-artifact/src/program/keys.rs:183-203`). The separation is meaningful: one profile can be re-assessed under a new rule set, and one rule set applies across profiles, so neither identity determines the other.

**Fact — the feasibility rules are unnamed.** `grep -rni "rule_set\|ruleset\|FeasibilityRuleSet" crates/tiler-compiler/src/` returns nothing. There is a version constant, `PROTOTYPE_FEASIBILITY_RULE_VERSION: u32 = 1` at `physical.rs:39`, and no governed key for the rules it versions.

**Inference.** Any consumer building a `FeasibilityRuleSetRef` has no key to use, and any consumer reading `ProfileIdentity` as one identity encodes the conflation. Because these values enter artifact identity under ADR 0072, that is a false claim rather than an inconvenience: an artifact would assert that a variant was assessed under a rule set named after a target profile.

**Inference — the fusion is currently invisible because there is one of each.** One profile, one rule version. The conflation costs nothing today and becomes wrong the moment a second target family or a second rule revision appears, both of which the current goal's target-breadth axis makes near-term.

## Scope

Give the feasibility rule set its own governed key and identity, separate from the target profile's. Update `FactProvenance` and every `CapabilityFact` that currently cites `ProfileIdentity` to cite whichever of the two it actually means — that distinction is the substance of this ticket, not a mechanical rename, and each site should be read rather than pattern-matched.

Decide explicitly whether a `CapabilityFact`'s provenance names the profile that declared it, the rule set that admitted it, or both, and state the reason. A fact declared by a governed profile and a fact admitted by a rule revision are different provenance claims.

## Closes when

A target profile identity and a feasibility rule set identity exist as separate types, every provenance site names the one it means, a consumer can build both `TargetProfileRef` and `FeasibilityRuleSetRef` with no invented value, and `uv run --locked python scripts/check_repository.py` passes.

## Measured scope, and an answer to this ticket's own open question

**Size, read rather than estimated.** `grep -rn "ProfileIdentity" crates/tiler-compiler/src/` returns 16 lines across two files — 14 in `feasibility.rs`, 2 in `physical.rs`. The ticket body above says the split means "rewriting every `FactProvenance` in the feasibility layer"; that overstated it. `FactProvenance` is one struct with one field (`feasibility.rs:278-287`) and one constructor, `declared_by`. Most of the 14 sites are test fixtures.

**The provenance question, answered by what the code means.** The ticket asks whether a `CapabilityFact`'s provenance names the profile that declared it, the rule set that admitted it, or both. It should name the **target profile**. A capability fact is a *bound* — "workgroup threads ≤ 1" — and a profile is what declares that bound; the rule set governs how a bound is compared against a requirement, not what the bound is. The existing constructor is already named `declared_by` and documented "Records that a fact was declared by `profile`", so the code already means the profile and only the type is wrong.

**The genuine difficulty, which is not the rename.** `ProfileIdentity::version` is documented as "the feasibility-rule identity of the profile: two profiles that would evaluate predicates differently must not share a version" — a *profile* versioning requirement expressed through a *rule* version. So the conflation is not merely two fields in one struct; it encodes a real invariant that must survive the split. After splitting, something must still guarantee that a profile whose predicates evaluate differently is distinguishable. `carry-the-target-profile-descriptor-identity-into-the-plan` supplies the mechanism: the canonical descriptor already covers every fact's axis, bound, phase, authority, and validity, so two profiles that evaluate predicates differently already have different descriptors without relying on a shared version counter. Confirm that before removing the version from the profile's side, rather than assuming the descriptor subsumes it.

**Do not** simply rename `ProfileIdentity` to `TargetProfileIdentity` and add a second struct. The version field has to be assigned to one side or the other with a stated reason, and the invariant above is what decides it.
