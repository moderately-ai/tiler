---
id: carry-the-target-profile-descriptor-identity-into-the-plan
title: Carry the target profile's descriptor identity into the plan
status: todo
priority: p0
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler, artifact, identity]
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
