---
id: correct-the-opaque-identity-bound-attributed-to-the-wrong-authority-in-keys-rs
title: Correct the opaque identity bound attributed to the wrong authority in keys.rs
status: done
priority: p2
dependencies: []
related: [correct-the-artifact-abis-claim-that-nothing-asserts-the-kernel-identity-crossing]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [identity, documentation]
---

A module doc attributes a bound to the wrong authority. **The notable part is the direction: the contract document is right and the crate doc is stale**, which inverts the usual assumption that source outranks prose — so this was not findable by trusting the code over the contract.

## Facts, coordinator-verified at the merge that found it

**Fact.** `crates/tiler-artifact/src/program/keys.rs`, in the module doc under the phrase *"An opaque identity's bound belongs to whoever mints it"*, states that a `TargetProfileDescriptorDigest` **is under `MAX_OPAQUE_IDENTITY_BYTES`**. It is not: it is bounded by this crate's own `pub const MAX_TARGET_PROFILE_DESCRIPTOR_BYTES: usize = 64 * 1_024;`, declared in the same file, supplied by the `opaque_identity!` invocation, and asserted by `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` as `limit: super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`.

**Fact.** `docs/artifact-abi.md` states this **correctly** — "carries its own 64 KiB ceiling". Both facts verified by grep at `7d6c7963`.

**Fact.** The same paragraph calls that constant "crate-private" to `tiler_compiler`, which reads as though only the compiler has one. `keys.rs` declares its own as `pub`.

## Why this one is worth a ticket

The doc's own heading is *"An opaque identity's bound belongs to whoever mints it"* — and the sentence beneath it attributes the bound to someone else. It contradicts the rule it is stated to illustrate, which is the shape a reader is least likely to question.

## What closes this

The sentence naming the real authority, and the "crate-private" clause corrected so it does not imply the compiler is the only holder. Name the construction (`MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` and its `opaque_identity!` invocation) rather than restating `64 * 1_024` — a figure restated in prose rots on a schedule nobody watches, and this file already has the constant.

**Establish the treatment from history** with `git log -S` and `git show <commit>:<file>`: a claim true when written is dated beside, one never true is substituted with the retired wording quoted. That is repository practice — several ADRs state it while applying it and none decides it, so cite the practice, not an authority. A retired sentence quoted verbatim stays greppable; say inline that a later hit lands inside your note.

**Do not edit `docs/artifact-abi.md`** — `contracts/artifacts`, not this scope, and it is the half that is already right.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — three ways an anchor fails as absence: a line break inside it (doc comments wrap at 80 columns), an emphasis or backtick marker the source lacks, and unescaped brackets read as a character class.

**Check the rest of this module doc and name the count.** A sweep of the neighbouring contract read 14 claims and found this one; the module doc itself is unexamined.

## Worker audit, re-verified at `cb62784c7d8b63aa2e73c9ac490101b748abc0ec`

**All three stated Facts verified; none required repair.** Two carry a precision note.

- **Fact 1 (the bound) — verified.** `keys.rs` said the descriptor identity "is under [`MAX_OPAQUE_IDENTITY_BYTES`]"; the `opaque_identity!` invocation passes `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`, and `crates/tiler-artifact/src/program/tests.rs`'s `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` asserts `limit: super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`. *Precision:* the invocation **passes** the constant as its `$limit` argument rather than supplying it; the declaration is separate and `pub`.
- **Fact 2 (the contract is right) — verified** at this base, not only at `7d6c7963`: `grep -F "carries its own 64 KiB ceiling" docs/artifact-abi.md` returns one hit.
- **Fact 3 (crate-private) — verified.** *Precision:* the adjective is **literally true** of the compiler's constant — `pub(crate) const MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` in `crates/tiler-compiler/src/target/feasibility.rs`. The defect is the implication that the compiler is the only holder, which is how the ticket already frames the fix.

**Ever-true verdicts, from `git log -S` and `git show`.** The bound clause was **true when written**: `22616630` (2026-07-27) wrote it while the invocation genuinely passed `MAX_OPAQUE_IDENTITY_BYTES`, and `0b7e59d3` (2026-07-30) declared this crate's own constant and moved the invocation, the constant doc, and the type doc onto it *without touching the module doc*. The visibility clause was **never true as read**: `fe6d3a87` (2026-08-01) rewrote this very sentence to insert "crate-private" two days *after* this file already declared its own `pub` constant of that name — that commit edited the stale sentence and did not notice the staleness. Treated as practice prescribes: the first dated beside, the second substituted with its wording quoted.

**Module-doc census: 23 propositions read, 20 checkable against source or ADRs, 3 unfalsifiable rationale. Three of the 20 were wrong** — the two this ticket names, plus one it did not.

**New finding, same file, same defect class.** The governed-key subject list read "…which capability, which target property." That list is one-to-one with the `governed_key!` invocations and was exact at `d5b63819` (2026-07-24). It decayed twice: `d1a95e18` (2026-07-25) moved `TargetPropertyKey` to `tiler_ir::program::abi`, and `d715d5da` (2026-07-31) added `RouteFeatureKey` without extending the sentence. So the doc named a key this module no longer defines and omitted one it does. Corrected to "which backend-scoped route requirement" with a dated note, since it too was true when written.

Verified-correct and left alone: ADR 0074 §2 and ADR 0090 item 10 both say what they are cited for; the alphabet equality with `tiler_compiler::target` (identical predicate at `target.rs`); 256 versus `MAX_TARGET_PROFILE_KEY_BYTES = 128`; the absent mutual dependency (neither `Cargo.toml` names the other); `CanonicalArtifactProgramIdentity` exposing only `as_bytes`; `super::codec::budget`'s reuse-rather-than-restate rule; and the `MAX_KERNEL_IDENTITY_BYTES` bound on `BackendEntryKey`. The constant's own doc and the `TargetProfileDescriptorDigest` type doc were **already correct** and are unchanged.

## Outcome — delivered

Commit `2cc29f0c` corrected the complete `keys.rs` module documentation. The
target-profile descriptor now names this crate's own
`MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` and its `opaque_identity!` use as the
governing bound; the dated correction preserves the former shared-bound state
without presenting it as current. The compiler clause now says that the
compiler independently owns a crate-private constant rather than implying it
is the only holder. The neighbouring governed-key list was also brought back
into agreement with the module by naming the backend-scoped route requirement
instead of the relocated target-property key. Commit `a4739a41` recorded the
closure. No constant, visibility, public type, identity bytes, or codec behavior
changed.
