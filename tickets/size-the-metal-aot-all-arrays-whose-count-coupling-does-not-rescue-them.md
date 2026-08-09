---
id: size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them
title: Size the metal AOT ALL arrays whose count coupling does not rescue them
status: done
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

## Per-Fact audit (worker, re-read at base `c81f9257`)

All four Facts hold. Two need refining.

**Verified — the four declarations and their lengths.** `crates/tiler-metal-aot/src/input.rs` declares `pub const ALL: [Self; 9]` for `AppleSdk`, `[Self; 10]` for `ApplePlatform`, `[Self; 12]` for `MslVersion`; `diagnostic.rs` declares `[Self; 2]` for `CompileStage`. Each is immediately followed by `pub const COUNT: usize = Self::ALL.len()`, so every `COUNT` is derived from the array rather than the type.

**Imprecise — "none is sized from its type" is true, but the four were not equally exposed.** `CompileStage::ALL` was not a bare literal: it wrapped the literal in `match Self::Metal { Self::Metal | Self::Metallib => [...] }` under a comment claiming it "goes non-exhaustive instead, which is the same completeness the matches below rely on". That guard does fire — but it constrains the *pattern*, not the array. Measured: adding an `AirPackPerturb` stage raises `E0004` at the guard, and widening the alternation to `Self::Metal | Self::Metallib | Self::AirPackPerturb` — the smallest edit that silences it — leaves the two-element literal compiling, `cargo check -p tiler-metal-aot --all-targets` at exit 0 and `62 tests run: 62 passed`. So the guard was weaker than its comment claimed, and the ticket's flat framing understates the difference between the three bare literals and the fourth while reaching the correct fix for all four. The guard is removed rather than kept beside `variant_count`: it constrains nothing the length does not, and it additionally errors spuriously when a stage *is* added correctly to both.

**Verified — the counterparts and the coupling spelling.** `crates/tiler-metal/src/target_correspondence.rs` carries `const FAMILY_COUNT: usize = MetalPlatform::COUNT;` then `const _: [(); FAMILY_COUNT] = [(); ApplePlatform::COUNT];`, and the same shape for `LANGUAGE_COUNT` against `MslVersion::COUNT`.

**Verified — the coupling does not rescue the driver side. This stays p1.** Perturbing `ApplePlatform` with a `BridgeOsPerturb` family, satisfying every `E0004` the compiler raises (four in `input.rs`, one in `family.rs`, one in a test match, and `driver_family_index` in `tiler-metal`), and leaving `ALL` at ten: `cargo check -p tiler-metal --all-targets` exits 0 and `cargo nextest run -p tiler-metal` reports `122 tests run: 122 passed, 0 skipped`. The `const _` equality holds because `ApplePlatform::COUNT` *is* `ALL.len()`, so the omitted family never moves either side.

**Verified — the feature gate was absent.** `crates/tiler-metal-aot/src/lib.rs` began at `//! Bounded offline Apple Metal AOT compiler driver for Tiler.` with no `#![feature]` attribute. `#![feature(variant_count)]` is added above the module docs, carrying the same explanatory comment shape `tiler-metal`'s root uses.

**`variant_count` fits all four.** Each of the four enums is fieldless and each `ALL` enumerates its variants one-to-one — no variant×payload combinations, no deliberate subset, no ordered route. Read in full to confirm rather than counted from the literals.

## Measured: which are silent, which fail late (worker, at `c81f9257`)

Adding a variant to any of the four always raises `E0004` first, because all four are matched exhaustively somewhere. That is not the failure mode the hand-written length permits. The failure mode is an author closing those arms — which is what the compiler instructs them to do — and leaving `ALL` short. Measured from that state, **all four are fully silent**, which is worse than the `tiler-metal` sibling, where three of four eventually reddened a distant test.

- `AppleSdk`: variant added, `selector` and the test-only index map given arms, `ALL` left at nine → `cargo check -p tiler-metal-aot --all-targets` exit 0, `62 tests run: 62 passed`. `every_sdk_selector_appears_once_in_the_canonical_inventory` cannot catch it: it sizes `seen` from `AppleSdk::COUNT` and iterates `AppleSdk::ALL`, so it checks the list against itself and the missing variant is never constructed.
- `ApplePlatform`: as above, plus `tiler-metal` → exit 0, `122 tests run: 122 passed`.
- `MslVersion`: measured at **workspace** scope. One `E0004` workspace-wide (`driver_language_index`); with it given an arm and `ALL` left at twelve, `cargo check --workspace --all-targets` exits 0 and `cargo nextest run --workspace` reports `3190 tests run: 3190 passed, 8 skipped`. Nothing anywhere in the workspace is a guard.
- `CompileStage`: guard alternation widened as above → exit 0, `62 tests run: 62 passed`.

**After the repair, each perturbation fails at its own declaration.** `AppleSdk` `expected an array with a size of 10, found one with a size of 9`; `ApplePlatform` `expected an array with a size of 11, found one with a size of 10`; `MslVersion` `expected an array with a size of 13, found one with a size of 12`; `CompileStage` `expected an array with a size of 3, found one with a size of 2`. Each is `error[E0308]: mismatched types` pointing at the `pub const ALL: [Self; core::mem::variant_count::<Self>()]` line. To show the array error is not merely coincident with the `E0004` wave, the `AppleSdk` case was re-run with every exhaustive match satisfied — the exact state that was previously exit 0 with 62 tests green — and it now fails with that single `E0308` and nothing else.

No `const` assertion that `ALL.len() == variant_count::<Self>()` was added at any of the four sites: once `ALL` is declared with that length the comparison is a tautology, and the sibling already demonstrated that compilation stops at the declaration without reaching such an assertion's message.

## Draft public surface (ADR 0075 — reported, not decided)

All four `ALL` and the three `COUNT` constants are `pub` on `pub` types in `pub` modules, so every one is reachable outside the crate and its **value** is contract. This change alters only derivation; all seven values are unchanged (9, 10, 12, 2 and 9, 10, 12).

*Included* (out-of-crate readers observed at this base): `ApplePlatform::COUNT` sizes `PINNED_MAP` and is asserted twice in `crates/tiler-macros/src/family_cfg/tests.rs`; `ApplePlatform::COUNT` and `MslVersion::COUNT` are compared against their `tiler-metal` counterparts in `target_correspondence.rs`, which also iterates `ApplePlatform::ALL` and `MslVersion::ALL`. *Excluded* (no out-of-crate reader found): `AppleSdk::ALL`, `AppleSdk::COUNT`, and `CompileStage::ALL`. `CompileStage` itself is reachable out of crate through `DriverError::ToolFailure` and `StageOutputs::stage`, but its `ALL` is not read outside this crate. Tom decides the boundary.

## Graph conflict (reported, not resolved)

`state-the-search-constant-provenance-the-caps-audit-found-bare` was amended on 2026-08-08 with a note in its **User-visible outcome** saying the `MetalHostPredicate::ALL` half is already landed. Its **`## Closes when`** was not amended and still opens `The ALL constant derives from variant_count`, so the ticket still names completed work in the section that governs closure. Left for the coordinator.

## What closes this

Each `ALL` sized from its type, matching the spelling now used in `tiler-metal`. **Do not add a `const` assertion that `ALL.len() == variant_count::<Self>()`** — once `ALL` is declared with that length the assertion is a tautology and can never fail. The sibling repair proved this by perturbation: the array-length error fires at the declaration and the assertion's message never appears. It removed the existing tautology rather than replicating it, and so should you.

**Perturb each array separately and quote the failure text**, with a **control** showing what happens under the hand-written length — the sibling's controls are what proved the point, since three of its four enums produced only `E0004` at exhaustive matches and no array error at all, while the fourth was completely silent: 122 tests passed with a variant omitted. Report which of yours are silent and which merely fail late.

**Public boundary:** if any `ALL` or `COUNT` is reachable outside the crate, it is a labelled draft under ADR 0075 — name included and excluded sets and **do not decide**. Values must not change; only their derivation.

**One graph conflict to report, not resolve:** `state-the-search-constant-provenance-the-caps-audit-found-bare` names `MetalHostPredicate::ALL` in its outcome and closes-when, but that work has already landed in a crate that ticket does not scope. Flag it for the coordinator rather than editing it.

## Outcome

Implemented and closed in `b3cd69c5` (`Size the metal AOT ALL arrays from their types`, 2026-08-08). `AppleSdk::ALL`, `ApplePlatform::ALL`, `MslVersion::ALL`, and `CompileStage::ALL` now derive their declared lengths from `core::mem::variant_count::<Self>()`; the crate root carries the required nightly feature gate. The misleading `CompileStage` dummy-match guard was removed because it constrained only the pattern, not the array population. All four `ALL` and three `COUNT` values remain 9, 10, 12, 2 and 9, 10, 12 respectively.

The independent widen-and-omit perturbations above each reached `E0308` at its own declaration after the repair and were restored. No enum, public value, compiler-driver behaviour, tag, or identity changed. The public constants remain the same existing reviewed-draft surface; this derivation repair did not accept or widen them.

The reported graph conflict was resolved in `cb127f79`: the stale `MetalHostPredicate::ALL` closes-when clause in [`state-the-search-constant-provenance-the-caps-audit-found-bare`](state-the-search-constant-provenance-the-caps-audit-found-bare.md) is struck as completed work. No unresolved ticket edge or closure condition remains from this repair.
