---
id: carry-the-target-profile-descriptor-identity-into-the-plan
title: Carry the target profile's descriptor identity into the plan
status: done
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

## Correction to this ticket's own premise, from reading the source

The ticket above proposed carrying "the checked profile's `ProfileIdentity` — or the canonical descriptor bytes it is derived from". Reading both types shows that is two different values, and `ProfileIdentity` is the wrong one for the descriptor.

**Fact — `crates/tiler-compiler/src/feasibility.rs:194-213`.** `ProfileIdentity` is `{ key: &'static str, version: u32 }`, whose accessors are documented as "The governed profile key" and "**The feasibility-rule version**". It is a key plus a rule version, not a digest over anything.

**Fact — `crates/tiler-artifact/src/program/keys.rs:196-203`.** `FeasibilityRuleSetRef` is `{ key: FeasibilityRuleSetKey, revision: u32 }`, documented as "The feasibility rule set under which a plan variant was assessed" with a "Nonzero output-affecting revision".

**Inference — `ProfileIdentity` supplies `FeasibilityRuleSetRef`, not `TargetProfileDescriptorDigest`.** The shapes and the meanings agree exactly: governed key plus nonzero rule revision. `VariantSpec` requires a `FeasibilityRuleSetRef` too, and it was on the list of values an assembler had no source for — so this reading supplies one of them outright, with no new plumbing beyond exposure.

**Fact — the descriptor is still missing.** `TargetProfileDescriptorDigest` must identify the profile *descriptor*: the facts themselves. `CheckedTargetProfile` (`feasibility.rs:292-296`) holds `facts: Vec<CapabilityFact>` in a canonical order its constructor enforces — sorted by `(axis, phase)`, unique per pair — so a canonical encoding of those facts is the right subject. **No such encoder exists**; `CheckedTargetProfile` exposes `identity()` and `facts()` and nothing that digests them.

**Fact — a second, separate profile type exists.** `PrototypeTargetProfile` (`crates/tiler-compiler/src/request.rs:343-351`) carries `key` plus scalar limits and is the request-side profile; `CheckedTargetProfile` is the feasibility-side one. They are not two spellings of one type. Whether the descriptor digest should cover the checked facts, the request-side limits, or a reconciliation of both is now the ticket's real question, and it should be settled before an encoder is written — digesting the wrong subject would produce a stable, wrong descriptor identity.

**Consequence.** This ticket splits cleanly into an exposure half that needs no decision (`FeasibilityRuleSetRef` from `ProfileIdentity`) and a design half that does (what the descriptor's canonical subject is). The first can land immediately; the second must not be settled by an assembler's convenience.

## Retraction and a fuller reading: `ProfileIdentity` conflates two governed vocabularies

The correction above claimed "`ProfileIdentity` supplies `FeasibilityRuleSetRef`, not `TargetProfileDescriptorDigest`". That is also wrong, and reading the construction site rather than the accessors is what shows it. Recorded rather than quietly replaced, because the mistake has a pattern worth naming: both wrong claims came from matching *shapes* — `{key, u32}` against `{key, u32}` — instead of reading where the values come from.

**Fact — `crates/tiler-compiler/src/physical.rs:666`.** `let identity = ProfileIdentity::new(target.key, PROTOTYPE_FEASIBILITY_RULE_VERSION);` where `target` is a `PrototypeTargetProfile` and `PROTOTYPE_FEASIBILITY_RULE_VERSION: u32 = 1` is declared at `physical.rs:39`.

**Fact.** So `ProfileIdentity`'s `key` is the **target profile** key and its `version` is the **feasibility rule** version. Its two accessors say exactly this and were read as one identity anyway.

**Fact — no feasibility rule set key exists.** `grep -rni "rule_set\|ruleset\|FeasibilityRuleSet" crates/tiler-compiler/src/` returns nothing. The feasibility rules have a version and no name.

**Inference — the compiler can supply two of the four values an artifact needs, and they are not the two the shapes suggested.**

| Artifact value | Compiler source |
| --- | --- |
| `TargetProfileRef::key` | `PrototypeTargetProfile::key` — available |
| `TargetProfileRef::descriptor` | none; no digest or canonical descriptor encoding exists |
| `FeasibilityRuleSetRef::key` | none; the rules are unnamed |
| `FeasibilityRuleSetRef::revision` | `PROTOTYPE_FEASIBILITY_RULE_VERSION` — available |

**This is the session's defect class again, in the producer rather than between crates.** Two governed vocabularies — which target profile, and which feasibility rules assessed it — are fused into one struct in `tiler-compiler`, while `tiler-artifact` keeps them as two independent refs precisely because a profile can be re-assessed under new rules and a rule set applies across profiles. Propagating the fused identity into an artifact would encode the conflation into artifact identity.

**Consequence — the ticket needs a third piece it did not name:** the feasibility rules need a governed key of their own, not just a version. Splitting `ProfileIdentity` into a target-profile identity and a feasibility-rule-set identity is the durable fix; `split-profile-and-feasibility-rule-identity` owns it, because doing it inside this ticket would mean changing every `FactProvenance` in the feasibility layer while also adding a descriptor encoding, and those fail for different reasons.

**Descriptor subject, decided.** `TargetProfileDescriptorDigest` is an `opaque_identity!` accepting bounded non-empty bytes up to `MAX_OPAQUE_IDENTITY_BYTES = 1_024` (`keys.rs:28`), not a hash of a fixed width. So the compiler exposes the **canonical descriptor bytes** of the checked profile's facts and the consumer wraps them, rather than the compiler minting a hash. That avoids introducing a digest algorithm and a second identity that must be kept in agreement with the bytes it summarizes — the same argument that kept the signature out of the capability key. If a profile's canonical descriptor ever exceeds the bound, that is the point at which a digest becomes a real decision with a real reason, and it will fail closed rather than silently truncate.

## Outcome

**Done.** A caller outside `tiler-compiler` can build a `TargetProfileRef` — both halves, for the variant and for the payload compatibility contract — with no invented value. `Compilation::target_profile_key()` already gave the key; `PlanAlternative::target_profile_descriptor()` now gives the descriptor.

**Ownership decision, stated as the ticket required: the compiler emits canonical descriptor bytes and the consumer wraps them; it does not mint a digest.** `TargetProfileDescriptorDigest` is an `opaque_identity!` accepting bounded non-empty bytes up to `MAX_OPAQUE_IDENTITY_BYTES = 1_024`, not a fixed-width hash, so nothing requires hashing here. Emitting bytes avoids introducing a digest algorithm in the compiler and avoids a second identity that would have to be kept in agreement with the bytes it summarizes — the same argument that kept the resolved signature out of the capability key in `name-the-resolved-lowering-capability`. The test asserts the descriptor fits the governed bound, so a profile that outgrows it fails closed at that assertion rather than silently truncating, and that is the point at which a digest becomes a real decision with a real reason.

**What the descriptor covers.** `CheckedTargetProfile::canonical_descriptor` encodes a domain separator, the identity key, the rule version, and every fact's axis, bound, phase, authority, and validity scope — the whole of what makes one profile admit a candidate another rejects. Facts are already in the canonical `(axis, phase)` order the constructor enforces and are unique per pair, so the encoding is a function of the profile rather than of declaration order. It reuses `tiler_ir::identity`'s length framing, the module `relocate-abi-expressions-into-tiler-ir` consolidated.

`FactProvenance` is deliberately excluded: it cites the profile's own `ProfileIdentity`, so folding it in would make the descriptor depend on a value derived from its own subject.

`CapabilityAxis`, `FactAuthority`, and `FactValidityScope` gained governed tags written by exhaustive match rather than read from the discriminant (ADR 0074 convention 3). A descriptor is durable identity, so adding or reordering an axis must be a build error rather than a silent change to every descriptor ever produced.

**The descriptor is the assessed profile, not a matching one.** It is derived through the same `checked_target_profile` the feasibility assessment uses, from the request's own profile — which `verify_request` proves is the governed one and which `verify_kernel_program_layers` proves the program and every scheduled region were bound to. So this is not a recomputation from a key that happens to agree.

**Evidence.** New test `feasibility::tests::the_canonical_profile_descriptor_separates_profiles_sharing_a_key` asserts the property ADR 0043 actually needs: three profiles **sharing a key** — the baseline, one with a narrowed grid-axis bound, and one with an incremented rule version — produce three different descriptors, and the encoding is deterministic and domain-separated. A descriptor that only varied with the key would pass a weaker test and prove nothing. Full repository gate green.

**Split out, not deferred.** `split-profile-and-feasibility-rule-identity` (p1) owns the conflation this ticket uncovered, and is now a dependency of `carry-the-metal-payload-in-an-artifact-envelope` in its own right: an assembler still cannot build a `FeasibilityRuleSetRef`, because the feasibility rules have a version and no governed key. That is a separate missing value from the one this ticket supplied, and it fails for a different reason.
