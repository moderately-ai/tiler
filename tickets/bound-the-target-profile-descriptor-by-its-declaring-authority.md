---
id: bound-the-target-profile-descriptor-by-its-declaring-authority
title: Bound the target-profile descriptor by its declaring authority
status: done
priority: p2
dependencies: []
related: [bound-the-backend-entry-key-by-the-identity-it-carries, carry-the-target-profile-descriptor-identity-into-the-plan]
scopes: [implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi]
---
Split from `bound-the-backend-entry-key-by-the-identity-it-carries`, which established that a received opaque identity is bounded by the authority that mints it, applied that to `BackendEntryKey`, and deliberately left this one where it was rather than inventing a number for it.

**Fact — the type is named a digest and is not one.** `crates/tiler-compiler/src/feasibility.rs:614-621` and `crates/tiler-compiler/src/session.rs:219-222` both state it in terms: the compiler emits the canonical descriptor *bytes*, and "these bytes **are** the descriptor identity rather than a hash of it", specifically so that no second identity has to be kept in agreement with what it summarizes. `TargetProfileDescriptorDigest` wraps them directly.

**Fact — it is under a digest-sized bound.** `crates/tiler-artifact/src/program/keys.rs` bounds it at `MAX_OPAQUE_IDENTITY_BYTES` = 1,024, the same constant that bounds `PayloadDigest`, which really is fixed-width under the governed digest algorithm. The sibling identity that was in the same position — `BackendEntryKey` — turned out to be refusing every non-degenerate program, and was found by running rather than by reading.

**Measurement — nothing is refused today, and the headroom is not a design.** Host: Apple M4 Max, macOS, `nightly-2026-07-19`. `Compilation::target_profile_descriptor()` measures **249 bytes** for `tiler.prototype-target-neutral-baseline.v1`, and is constant across every program shape swept in the parent ticket, because it is a property of the profile rather than of the program. So the gap is latent rather than active, which is exactly why it is a separate ticket rather than a hidden risk inside one.

**Inference — the quantity grows with the profile, and nothing checks the growth.** `feasibility.rs:628-644` records what the descriptor covers: the identity key, every capability fact's axis, bound, phase, authority, and validity scope, and every honourability fact's dimension, behaviour, means, phase, authority, and validity scope. Each fact a profile declares adds bytes. A profile declaring enough facts crosses 1,024 and packaging refuses — loudly, which is correct, but for a reason that is an artifact-layer accident rather than a compiler-side limit anyone stated.

**Fact — the parent's fix does not transfer, and the reason is the dependency direction.** `BackendEntryKey` took `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES` because `tiler-artifact` already depends on `tiler-ir`. `crates/tiler-compiler/Cargo.toml` depends on `tiler-ir` and **not** on `tiler-artifact`; the reverse edge is the one that exists conceptually, and `tiler-artifact` cannot name a `tiler-compiler` constant. Exact check: `grep -n tiler-artifact crates/tiler-compiler/Cargo.toml` is empty.

**What is already in place, and what it does not settle.** `crates/tiler-compiler/src/session.rs:886-892` asserts `descriptor.len() <= 1_024` with the literal spelled out and a message naming the artifact boundary. `carry-the-target-profile-descriptor-identity-into-the-plan` recorded the intent: "If a profile's canonical descriptor ever exceeds the bound, that is the point at which a digest becomes a real decision with a real reason, and it will fail closed rather than silently truncate." So the failure mode is safe and the authority question is open — a duplicated literal in a test is not a published bound, and it is a second authority of exactly the kind `codec/budget.rs:10-12` warns drifts.

## The decision this ticket owes

Which of these is true, with the derivation rather than the preference:

1. **The compiler publishes the bound it honours** and `tiler-artifact` documents that a descriptor identity is admitted up to whatever the declaring authority states, the way `BackendEntryKey` now works. This needs the compiler to actually enforce it where the descriptor is built (`physical.rs::target_profile_descriptor`), not only in a test.
2. **The artifact layer publishes the bound and the compiler honours it**, which inverts the rule the parent ticket established and needs a stated reason why this subject is different — the candidate reason being that no profile is portable across artifact readers unless the *reader's* bound is the contract.
3. **The descriptor becomes a genuine digest**, which is the option `carry-the-target-profile-descriptor-identity-into-the-plan` deliberately deferred to the day the bound is actually crossed, and which costs the property both source comments defend: the bytes stop being the identity and a second identity has to be kept in agreement with them.

Eliminate before presenting. A bound picked in `tiler-artifact` with no authority behind it is not a fourth option; that is the failure the parent ticket documents.

## Closes when

A target-profile descriptor is bounded by a constant its declaring authority publishes and enforces where the descriptor is built; `crates/tiler-compiler/src/session.rs`'s literal `1_024` is gone or is a reference to that constant; `TargetProfileDescriptorDigest`'s documentation no longer calls its bound provisional; and `make full` passes.

## Outcome — option 1, and options 2 and 3 are eliminated mechanically (2026-07-27)

**Option 2 — the artifact publishes and the compiler honours — is not reachable.** Exact check: `grep -n "tiler-" crates/tiler-compiler/Cargo.toml crates/tiler-artifact/Cargo.toml` shows each depends on `tiler-ir` and **neither depends on the other**. For the compiler to honour an artifact constant it would need an edge that does not exist and whose direction is backwards — the compiler produces artifacts. This is not a preference; the option cannot be spelled.

**Option 3 — a real digest — was deferred by `carry-the-target-profile-descriptor-identity-into-the-plan` until the bound is actually crossed, and nothing has crossed it.** The governed descriptor measures 249 bytes against a 1,024 bound. Taking it now would cost the property both source comments defend — the bytes stop being the identity — to solve a problem that has not occurred.

**Option 1 landed.** `tiler-compiler` publishes `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` and refuses in `physical.rs::target_profile_descriptor`, where a descriptor is minted, through a typed `FeasibilityError::DescriptorTooLong { key, actual }` that names the profile. That is the substantive difference: `tiler-artifact` can only report a length, while the compiler can name the profile that declared too much.

Adding the variant broke the exhaustive match in `feasibility_intrinsic` at compile time, which is what a closed error vocabulary is for — the new case had to be classified rather than absorbed by a wildcard. It maps to `target-profile-descriptor-too-long`, in the same class as a malformed profile: a fact about the declaration, decided before any candidate is considered, with no other plan under which it becomes describable.

### The residual, stated rather than hidden

The compiler's bound and `tiler-artifact`'s `MAX_OPAQUE_IDENTITY_BYTES` are both 1,024, and **nothing checks that they stay equal.** Neither crate depends on the other, and `grep` over every manifest shows the only things depending on both are the two serial-sum prototypes — no library crate does, so no library can host the assertion. The relationship is held by comments on both constants, each saying that changing one requires reading the other.

This is a weaker guarantee than the `BackendEntryKey` precedent, and the reason is structural rather than an omission: that key's producer is `tiler-ir`, which `tiler-artifact` already depends on, so the artifact could simply name the producer's constant. Here the producer is `tiler-compiler` and the edge does not exist. `tiler-ir` cannot host the constant either — it has **no target-profile vocabulary at all** (`grep -rn "TargetProfile" crates/tiler-ir/src` is empty), and inventing one there would put a compiler concept in the shared IR.

The artifact-side bound is therefore reframed rather than removed: it is a **codec resource ceiling**, and that crate still refuses past it, because it validates what it is handed rather than trusting where it came from.

### Verification

The refusal path is reachable, checked rather than assumed: lowering the constant to 128 — below the governed 249 — fails **33 tests** across the compile path, so the refusal propagates as a typed compiler failure rather than being swallowed. Restored to 1,024 afterwards.

`session.rs`'s literal `1_024` is now a reference to the published constant, and `TargetProfileDescriptorDigest`'s documentation no longer calls its bound provisional — it names the governing authority and says what this crate's own bound is for.
