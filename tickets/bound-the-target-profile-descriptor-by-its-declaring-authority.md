---
id: bound-the-target-profile-descriptor-by-its-declaring-authority
title: Bound the target-profile descriptor by its declaring authority
status: todo
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
