---
id: carry-the-target-profile-descriptor-identity-into-the-plan
title: Carry the target profile's descriptor identity into the plan
status: in-progress
priority: p0
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler, artifact, identity]
claimed_from: todo
assignee: agent-coordinator
lease_expires_at: 1784992968
---
The third instance of one shape, found while building the first out-of-crate artifact assembler: a governed key reaches the plan and the exact identity beside it does not.

**Fact.** `tiler_artifact::program::TargetProfileRef` (`crates/tiler-artifact/src/program/keys.rs:183-193`) requires `key: TargetProfileKey` **and** `descriptor: TargetProfileDescriptorDigest`. Its doc states why: "ADR 0043 requires both the governed key and the exact descriptor identity: a profile key alone is not evidence that a variant is legal on a device that advertises the same key under a different descriptor."

**Fact.** `PrototypeTargetProfile` (`crates/tiler-compiler/src/request.rs:343-351`) carries `key: &'static str` and the axis limits. It has no descriptor identity, and `Compilation::target_profile_key()` is the only profile fact on the session boundary.

**Fact — the concept already exists.** `CheckedTargetProfile` in `crates/tiler-compiler/src/feasibility.rs:286` is documented as "An immutable checked target profile with versioned identity" and exposes `identity() -> ProfileIdentity` at `feasibility.rs:343`, exercised by `feasibility::tests::checked_profile_exposes_canonical_facts_and_versioned_identity`. So this is plumbing rather than a missing notion: the identity is computed and then not carried to where an artifact needs it.

**Inference — why an assembler must not invent one.** `TargetProfileRef` appears twice in a packaged artifact: on the variant, and as the payload's `compatibility` contract. Both enter artifact identity. A fabricated descriptor digest would make an artifact assert that a variant is legal against a profile descriptor that was never checked, which is exactly the claim the field's own documentation says a key alone cannot support. `AGENTS.md` requires unsupported cases to reject explicitly rather than be silently approximated.

## Scope

Carry the checked profile's `ProfileIdentity` — or the canonical descriptor bytes it is derived from — from feasibility into `ArtifactConstructionPlan`, and expose it on `tiler_compiler::session` beside the target profile key. Decide explicitly whether the compiler hands out a digest or the canonical descriptor bytes for the consumer to digest, and state which; the same ownership argument that settled `name-the-resolved-lowering-capability` applies, since this value enters artifact identity.

Check whether `PrototypeTargetProfile` and `CheckedTargetProfile` are two spellings of one profile before adding a third path between them. If they are, say so and reconcile them rather than threading an identity from one into the other.

## Closes when

A caller outside `tiler-compiler` can build a `TargetProfileRef` for both the variant and the payload compatibility contract with no invented value; the ownership choice is stated; and `uv run --locked python scripts/check_repository.py` passes.

## Correction to this ticket's own premise, from reading the source

The ticket above proposed carrying "the checked profile's `ProfileIdentity` — or the canonical descriptor bytes it is derived from". Reading both types shows that is two different values, and `ProfileIdentity` is the wrong one for the descriptor.

**Fact — `crates/tiler-compiler/src/feasibility.rs:194-213`.** `ProfileIdentity` is `{ key: &'static str, version: u32 }`, whose accessors are documented as "The governed profile key" and "**The feasibility-rule version**". It is a key plus a rule version, not a digest over anything.

**Fact — `crates/tiler-artifact/src/program/keys.rs:196-203`.** `FeasibilityRuleSetRef` is `{ key: FeasibilityRuleSetKey, revision: u32 }`, documented as "The feasibility rule set under which a plan variant was assessed" with a "Nonzero output-affecting revision".

**Inference — `ProfileIdentity` supplies `FeasibilityRuleSetRef`, not `TargetProfileDescriptorDigest`.** The shapes and the meanings agree exactly: governed key plus nonzero rule revision. `VariantSpec` requires a `FeasibilityRuleSetRef` too, and it was on the list of values an assembler had no source for — so this reading supplies one of them outright, with no new plumbing beyond exposure.

**Fact — the descriptor is still missing.** `TargetProfileDescriptorDigest` must identify the profile *descriptor*: the facts themselves. `CheckedTargetProfile` (`feasibility.rs:292-296`) holds `facts: Vec<CapabilityFact>` in a canonical order its constructor enforces — sorted by `(axis, phase)`, unique per pair — so a canonical encoding of those facts is the right subject. **No such encoder exists**; `CheckedTargetProfile` exposes `identity()` and `facts()` and nothing that digests them.

**Fact — a second, separate profile type exists.** `PrototypeTargetProfile` (`crates/tiler-compiler/src/request.rs:343-351`) carries `key` plus scalar limits and is the request-side profile; `CheckedTargetProfile` is the feasibility-side one. They are not two spellings of one type. Whether the descriptor digest should cover the checked facts, the request-side limits, or a reconciliation of both is now the ticket's real question, and it should be settled before an encoder is written — digesting the wrong subject would produce a stable, wrong descriptor identity.

**Consequence.** This ticket splits cleanly into an exposure half that needs no decision (`FeasibilityRuleSetRef` from `ProfileIdentity`) and a design half that does (what the descriptor's canonical subject is). The first can land immediately; the second must not be settled by an assembler's convenience.
