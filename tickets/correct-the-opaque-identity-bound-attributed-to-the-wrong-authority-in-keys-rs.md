---
id: correct-the-opaque-identity-bound-attributed-to-the-wrong-authority-in-keys-rs
title: Correct the opaque identity bound attributed to the wrong authority in keys.rs
status: todo
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
