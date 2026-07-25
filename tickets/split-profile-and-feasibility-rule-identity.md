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
