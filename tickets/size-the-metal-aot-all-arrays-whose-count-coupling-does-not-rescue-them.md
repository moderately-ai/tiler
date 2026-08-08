---
id: size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them
title: Size the metal AOT ALL arrays whose count coupling does not rescue them
status: todo
priority: p1
dependencies: []
related: [size-the-four-hand-written-metal-all-arrays-from-their-types]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [enumeration, tests]
---

The paired counterparts of the four `ALL` arrays just repaired in `tiler-metal`. **A coupling that looks like it guards these does not**, which is why this is p1 rather than a mechanical follow-up.

## Facts

**Coordinator-verified at `ad837786`.** `crates/tiler-metal-aot/src/input.rs` declares `pub const ALL: [Self; 9]`, `[Self; 10]`, and `[Self; 12]`; `diagnostic.rs` declares another. None is sized from its type.

**Reported by the worker that repaired the `tiler-metal` side, not coordinator-verified.** `ApplePlatform::ALL [Self; 10]` and `MslVersion::ALL [Self; 12]` are the counterparts of `MetalPlatform` and `MslLanguageVersion`. `target_correspondence` couples the two crates via `const _: [(); FAMILY_COUNT] = [(); ApplePlatform::COUNT];`.

**The finding that makes this p1.** That coupling does **not** rescue the driver side: an omitted variant leaves `ApplePlatform::COUNT` at 10 and the equality still holds. A check that compares two counts cannot detect both shrinking together, and cannot detect one that never grew. Verify this yourself before relying on it — if the coupling *does* catch an omission, this drops to p3.

**Known obstacle, reported.** `core::mem::variant_count` needs `#![feature(variant_count)]` at the crate root, which `tiler-metal-aot` does not carry. Adding a nightly feature gate to a crate is a change worth stating explicitly in your report rather than slipping in — `rust-toolchain.toml` is the version authority and accepted features already require nightly, so this is permitted, but say that you did it and why.

## What closes this

Each `ALL` sized from its type, matching the spelling now used in `tiler-metal`. **Do not add a `const` assertion that `ALL.len() == variant_count::<Self>()`** — once `ALL` is declared with that length the assertion is a tautology and can never fail. The sibling repair proved this by perturbation: the array-length error fires at the declaration and the assertion's message never appears. It removed the existing tautology rather than replicating it, and so should you.

**Perturb each array separately and quote the failure text**, with a **control** showing what happens under the hand-written length — the sibling's controls are what proved the point, since three of its four enums produced only `E0004` at exhaustive matches and no array error at all, while the fourth was completely silent: 122 tests passed with a variant omitted. Report which of yours are silent and which merely fail late.

**Public boundary:** if any `ALL` or `COUNT` is reachable outside the crate, it is a labelled draft under ADR 0075 — name included and excluded sets and **do not decide**. Values must not change; only their derivation.

**One graph conflict to report, not resolve:** `state-the-search-constant-provenance-the-caps-audit-found-bare` names `MetalHostPredicate::ALL` in its outcome and closes-when, but that work has already landed in a crate that ticket does not scope. Flag it for the coordinator rather than editing it.
