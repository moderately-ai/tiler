---
id: split-profile-and-feasibility-rule-identity
title: Split ProfileIdentity into a target profile identity and a feasibility rule set identity
status: review
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
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

## Outcome

The split landed inside `tiler-compiler`. The public-surface half did not, and is `expose-the-feasibility-rule-set-on-the-compiler-boundary`; this ticket is **not** done.

**Fact — two types now exist, and the version went to the rules.** `crates/tiler-compiler/src/feasibility.rs` defines `TargetProfileIdentity { key }` and `FeasibilityRuleSetIdentity { key, revision }`. `ProfileIdentity` is gone. The rule set has a governed key it never had — `tiler.feasibility.phased-capability-bounds.v1` — beside the revision that was previously `physical.rs`'s `PROTOTYPE_FEASIBILITY_RULE_VERSION`, and the pair is one `pub(crate) const GOVERNED_FEASIBILITY_RULE_SET`. It is a constant of the authority rather than a function of a target, because `CheckedTargetProfile::assess` applies exactly these rules to every profile; a `fn(target) -> rules` would imply a variation that cannot occur and would invite a second definition of one identity. `FeasibilityRuleSetIdentity::new` rejects an empty key and a zero revision, and the constant is built through it in a `const` whose `None` arm is a `panic!`, so a malformed governed identity is a build failure rather than something an artifact can record.

**Fact — the constant moved modules, and that is the substance rather than a tidy-up.** `PROTOTYPE_FEASIBILITY_RULE_VERSION` sat in `physical.rs` beside the prototype target profile, named as though it versioned that target. The rules it versions are `Relation`, `satisfies`, `CapabilityAxis::admits`, `authority_matches_phase`, `CheckedTargetProfile::resolve`'s preference for the most refined available fact, and the outcome precedence in `assess` — all of them in `feasibility.rs`. The revision constant now sits with them and its doc names them, so a future change to any one has the thing it must bump in view.

**Decision — provenance names the target profile.** `FactProvenance::declared_by(TargetProfileIdentity)`, as the ticket's own answer proposed, and the doc now states the reason on the type: a capability fact is a *bound*, a profile is the authority that declares a bound, and the rule set governs how a requirement is compared against that bound without ever supplying or admitting it. Citing the rule set there would attribute the claim to something that never made it. Nothing else in the crate cited `ProfileIdentity`, so this was the only provenance decision to make.

**Measurement — the invariant, confirmed by reading rather than assumed.** `ProfileIdentity::version` documented "two profiles that would evaluate predicates differently must not share a version". Reading `assess` and `canonical_descriptor` in full: `assess` reads each requirement's axis and amount, `resolve` reads only each fact's axis, phase, and bound, and the comparison those feed is `satisfies(axis.relation(), ..)` where `relation()` is a `const fn` of the axis alone. The descriptor encodes the key, the fact count, and every fact's axis, bound, phase, authority, and validity, in the canonical `(axis, phase)` order the constructor enforces and unique per pair. So equal descriptors imply equal fact multisets imply equal verdicts for every proposal and phase: the descriptor discharges the invariant structurally, where the version only asserted it — nothing ever forced a declarer to bump the version for a changed profile. `feasibility::tests::profiles_sharing_a_descriptor_return_the_same_verdicts` exercises the forward direction over five grid amounts and three phases with a deliberately reordered declaration.

**Consequence — the version left the descriptor too, and the domain went to v2.** `canonical_descriptor` no longer encodes the rule version. Keeping it would have made a profile appear to have changed when only the rules did, which is the same conflation one layer down, and the artifact layer already records the rule set as a second reference folded into variant identity (`crates/tiler-artifact/src/program/model.rs:1331-1332`). Because a descriptor is durable identity and its encoding changed meaning, `PROFILE_DESCRIPTOR_DOMAIN` is now `tiler.target-profile.descriptor.v2\0` so a `v1` encoding cannot collide with one that no longer means the same thing. The descriptor test's "differing rule version" case was replaced by a fact moved to a later phase — same key, same axes, same bounds, genuinely different verdicts — which is a case the version was supposed to cover and never did.

**Fact — validation lost a rule it could no longer state.** `CheckedTargetProfile::new` rejected `identity.version() == 0`; with no version on the profile there is nothing to reject, and the `rule: "identity"` class now covers only the empty key. The nonzero constraint did not disappear: it moved to `FeasibilityRuleSetIdentity::new`, where zero is rejected because the artifact boundary reserves it for "unset".

**Fact — the plan carries both halves, and only one is reachable from outside.** `ArtifactConstructionPlan` gained `feasibility_rule_set` beside the existing `target_profile_descriptor`, with a `pub(crate)` accessor, and `program::tests::the_plan_names_its_target_profile_and_its_feasibility_rules_separately` asserts the rule set key differs from the profile key and the revision is nonzero. `tiler_compiler::session` still exposes only the profile half, so a consumer can build `TargetProfileRef` and cannot yet build `FeasibilityRuleSetRef`. The accessor carries an `#[allow(dead_code)]` naming the follow-up.

**Why the last step was split rather than finished.** It is two `pub` methods on `session::PlanAlternative`, which is a public-boundary decision reserved to Tom under AGENTS.md, on a surface ADR 0075 marks a reviewed draft with seven deferred public-surface questions. The follow-up also carries a real question — whether the pair belongs on `PlanAlternative` or on `Compilation` — since the rules vary by neither.

**Also corrected in passing.** `physical.rs`'s doc comment for `checked_target_profile` had been orphaned onto `target_profile_descriptor` by an earlier edit, leaving the former undocumented and the latter carrying a description of something else. Both now document themselves.

**Not changed.** `tiler-artifact` and `tiler-metal-aot`: `grep -rn "ProfileIdentity" crates/` returned hits only in `tiler-compiler`, and no crate constructs a `FeasibilityRuleSetRef` outside `tiler-artifact`'s own tests.
