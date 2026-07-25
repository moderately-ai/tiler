---
id: expose-the-feasibility-rule-set-on-the-compiler-boundary
title: Expose the feasibility rule set identity on the public compiler boundary
status: done
priority: p1
dependencies: [split-profile-and-feasibility-rule-identity]
related: [carry-the-target-profile-descriptor-identity-into-the-plan, prototype-public-compiler-api]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, feasibility, identity, public-surface]
---
The compiler now derives the feasibility rule set identity but no caller outside the crate can read it, so an artifact assembler still cannot build `FeasibilityRuleSetRef` without inventing a value.

**Fact — the identity exists and reaches the plan.** `split-profile-and-feasibility-rule-identity` added `crate::feasibility::FeasibilityRuleSetIdentity` and the governed constant `GOVERNED_FEASIBILITY_RULE_SET` (`crates/tiler-compiler/src/feasibility.rs`), and `ArtifactConstructionPlan` carries it with a `pub(crate)` accessor `feasibility_rule_set()` (`crates/tiler-compiler/src/program.rs`).

**Fact — it stops at the crate boundary.** `crates/tiler-compiler/src/session.rs` exposes `PlanAlternative::target_profile_descriptor()` and `Compilation::target_profile_key()`, which together let a consumer build `TargetProfileRef`. There is no counterpart for the rule set, so `FeasibilityRuleSetRef { key, revision }` (`crates/tiler-artifact/src/program/keys.rs:196-203`) has no source. The accessor added by the parent ticket carries an `#[allow(dead_code)]` naming this ticket as its reason.

**Why it was not done in the parent ticket.** Adding a public item is a boundary decision reserved to Tom (AGENTS.md, "Tom must review key public crate, module, trait, type, and call-site boundaries before they are accepted or merged"), and `session.rs` is explicitly a reviewed draft under ADR 0075 with seven deferred public-surface questions. The parent landed everything that needed no such decision.

## Scope

Add the accessor pair to `PlanAlternative` in `crates/tiler-compiler/src/session.rs` — `feasibility_rule_set_key() -> &str` and `feasibility_rule_set_revision() -> u32` — mirroring `SelectedCapability::capability_key()`/`capability_revision()`, which exist for the same reason: the value enters artifact identity under ADR 0072, so it is minted by the compiler and handed over whole rather than composed by a consumer from parts.

Decide and state whether the pair is exposed on `PlanAlternative` (per alternative, like the profile descriptor) or on `Compilation` (per target). The rules are currently one constant for the whole authority, so per-alternative exposure asserts a variation that does not exist today — but per-target exposure asserts one that will not exist either, since the rules are not a function of the target. Say which claim the surface is making.

Remove the `#[allow(dead_code)]` on `ArtifactConstructionPlan::feasibility_rule_set` once it is reached from the compile path rather than only from tests.

## Closes when

An out-of-crate consumer can build a `tiler_artifact::program::FeasibilityRuleSetRef` from `tiler_compiler::session` alone with no invented value, a test on the session boundary proves it (as `an_alternative_names_its_capabilities_and_exposes_its_abi_inputs` does for the capability pair), and `uv run --locked python scripts/check_repository.py` passes.

## Decision — Tom, 2026-07-25

**Approved: promote, and the pair goes on `Compilation`, not `PlanAlternative`.** The ticket left the placement open. `TargetProfileRef` and `FeasibilityRuleSetRef` vary by neither plan nor alternative, so siting them per-alternative would invite a reader to believe they can differ between alternatives of one compilation — a false affordance in a type. `Compilation` is where a compilation-invariant belongs.

Move the existing `PlanAlternative::target_profile_descriptor` accessor to match, so the two halves of `TargetProfileRef` are not split across two views. Unblocks `carry-the-metal-payload-in-an-artifact-envelope`, which had three of its four required identities.

## Outcome

**Landed as approved.** `Compilation` now carries all four values an assembler needs for `TargetProfileRef` and `FeasibilityRuleSetRef`: `target_profile_key()`, `target_profile_descriptor()`, `feasibility_rule_set_key()`, and `feasibility_rule_set_revision()`. `PlanAlternative::target_profile_descriptor` is gone; the two halves of `TargetProfileRef` are read from one view. The promotion is exactly the two approved methods plus the moved one — no other `pub` item was added.

**What the surface now claims, stated as the ticket asked.** Both identities are *compilation-invariant*, not per-alternative and not derived from the target. The profile descriptor is a property of the one `VerifiedTargetRequest` the target compiles under; the rule set is `GOVERNED_FEASIBILITY_RULE_SET`, a constant of the feasibility authority, and is on `Compilation` because that is the narrowest view a consumer can hold that is not narrower than the value — not because the rules are a function of the target.

**The invariance is checked, not assumed.** `pipeline::target_assessment` lifts both off the portfolio's first alternative and refuses a portfolio whose alternatives disagree, or an empty one, as `CompilerOutputError::Program(ProgramError::Structure)` under rules `portfolio-target-assessment` and `portfolio-empty`. Lifting rather than re-deriving from the request is deliberate: the value a consumer reads is the one the retained alternatives were actually built with, and there is still exactly one derivation site (`build_artifact_plan`). `verify_alternative` independently re-derives every plan from the one request, so both refusals are unreachable while that verification holds — they are the fail-closed statement of what the new siting asserts, not a second authority.

**Fact — no consumer was broken by the move.** `grep -rn "target_profile_descriptor" crates prototypes` before the change returned nine sites, all inside `tiler-compiler`; the accessor had no out-of-crate caller yet.

**Measurement.** `cargo nextest run -p tiler-compiler`: 206 passed, 0 failed. The new `session::tests::a_compilation_names_its_target_profile_and_its_feasibility_rules` asserts a non-empty descriptor within the artifact boundary's 1024-byte opaque-identity bound, a rule set key under `tiler.feasibility.` that differs from the profile key, and a nonzero revision. The full gate result is recorded on `carry-the-metal-payload-in-an-artifact-envelope`, which lands in the same branch.

`ArtifactConstructionPlan::feasibility_rule_set` lost its `#[allow(dead_code)]`: the compile path now reads it.
