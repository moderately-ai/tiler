---
id: size-the-four-hand-written-metal-all-arrays-from-their-types
title: Size the four hand-written metal ALL arrays from their types
status: in-progress
priority: p1
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [enumeration, tests, identity]
claimed_from: todo
assignee: coord
lease_expires_at: 1786174857
---

Four `ALL` arrays over enums carry hand-written lengths. Adding a variant without adding it to `ALL` **compiles**, so the population silently stops covering its domain while every check over it stays green.

## Facts, coordinator-verified at `4361a658`

**Fact — the correct pattern already exists in one of these very files.** `crates/tiler-metal/src/applicability.rs` declares `pub const ALL: [Self; core::mem::variant_count::<Self>()]` for `MetalGpuFamily`, and states the reason in prose anchored at *"declared length is `variant_count`, so the omission is an array-length"*. A `const` block at the same site asserts `MetalGpuFamily::ALL.len() == core::mem::variant_count::<MetalGpuFamily>()`.

**Fact — four siblings do not follow it.** `MslLanguageVersion::ALL` declares `[Self; 12]`, `MetalPlatform::ALL` declares `[Self; 10]`, and `MetalFloatArithmeticType::ALL` declares `[Self; 3]`, all in `crates/tiler-metal/src/target.rs`; `MetalHostPredicate::ALL` declares `[Self; 7]` in `applicability.rs`. Each is preceded by its own `pub enum` declaration and none is length-checked against `variant_count`.

**Fact — the damage propagates.** Each of the four is immediately followed by `pub const COUNT: usize = Self::ALL.len()`. `COUNT` is therefore derived from the array rather than from the type, so an `ALL` that stops covering shrinks `COUNT` with it, and any population sized by `COUNT` shrinks silently while still reporting success.

**Inference — why p1 despite no live defect.** All four arrays are complete today; this is latent. It is p1 because the failure is invisible by construction: a widened vocabulary produces no error, no warning, and no red check, and `AGENTS.md` names exactly this — "a hand-written length, a successor chain, and a wildcard-free match can all be satisfied by an enumeration that has stopped covering its domain." These four are the hand-written-length case, sitting beside a correct example.

## What closes this

The four declarations sized from `core::mem::variant_count::<Self>()`, matching the sibling that already does it. Prefer the existing spelling over inventing a second one — two patterns for the same property is how the asymmetry arose.

**Perturb each of the four separately and quote the failure text.** Add a variant to each enum in turn without extending its `ALL`, and show the array-length error. Perturbing one and asserting the others behave the same is not evidence; `AGENTS.md` requires that where a check guards several independent properties, each is perturbed on its own. Revert each and confirm.

**Check whether any of the four is `pub` in a way that makes its length observable.** `MslLanguageVersion`, `MetalPlatform`, and `MetalFloatArithmeticType` are `pub` enums, so if `COUNT` is reachable by a consumer its value is contract, and any change to how it is derived is a **labelled draft** under ADR 0075 until Tom accepts the surface. Report the included and excluded sets; do not decide the boundary.

**Then check the rest of the workspace for the same shape.** This audit swept `crates/` for `[Self; N]`, `[&str; N]`, and `[&[u8]; N]` and found further hand-sized arrays outside `tiler-metal` — in `tiler-cache`, `tiler-conformance`, and `tiler-runtime` tests. Those are other scopes; **report them with a count** rather than editing, so the sweep's extent is on record either way.
