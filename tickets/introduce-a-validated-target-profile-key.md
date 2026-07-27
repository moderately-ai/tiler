---
id: introduce-a-validated-target-profile-key
title: Introduce a validated target profile key
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [api]
---

Split from `admit-a-caller-declared-target-profile`, which cannot proceed until this lands. Read that ticket's "Survey correction" section first — it carries the measurement this one rests on.

## The problem

A target profile's key is `&'static str` everywhere it is named, and a caller-declared profile cannot supply one. `grep -rn "target_profile_key" crates/tiler-compiler/src` reports **56 sites**; eleven bind it as `&'static str` in `frontier.rs`, `physical.rs`, or `selection.rs`. The load-bearing ones:

- `frontier.rs:202` — `target_profile_keys: Vec<&'static str>`
- `frontier.rs:214` — `TargetApplicability::for_targets(impl IntoIterator<Item = &'static str>)`
- `ImplementationFrontier::target_profile_key() -> &'static str`
- `physical.rs` threads the same type into the schedule verifier

**Measurement:** changing `PrototypeTargetProfile`'s fields to `Cow<'static, _>` without moving this vocabulary first produces **57 compiler errors** across four files. Attempted and reverted on 2026-07-27 for exactly that reason.

## What to do

Introduce a validated `TargetProfileKey` — an opaque type with a fallible constructor, per ADR 0074 convention 2, so no caller can assemble one naming a profile that was never declared. Move the applicability vocabulary onto it, keeping every current caller passing the governed key unchanged.

**This is a behaviour-preserving change and must land as one.** No profile becomes declarable here; that is the parent ticket. The point is to make the parent's owned-profile step tractable rather than a 57-error commit spanning four files.

## The check that governs it

`physical.rs`'s `the_governed_descriptor_bytes_do_not_move` pins the governed profile's 249 canonical descriptor bytes exactly. **It must not move.** Those bytes reach the canonical explain subject and the artifact's `TargetProfileDescriptorDigest`, so a change there moves every artifact identity and invalidates every cache entry. If the key's encoding is touched at all, that is a finding rather than a rebaseline — the key is encoded through `push_slice` of its bytes, so a newtype wrapping the same bytes encodes identically and the pin should simply keep passing.

## Closes when

A validated `TargetProfileKey` exists with a fallible constructor and no public field; the applicability vocabulary in `frontier.rs`, `physical.rs`, and `selection.rs` takes it rather than `&'static str`; the governed descriptor bytes are unchanged; and `make full` passes.
