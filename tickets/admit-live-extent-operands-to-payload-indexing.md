---
id: admit-live-extent-operands-to-payload-indexing
title: Admit live extent operands to payload indexing
status: review
priority: p1
dependencies: [admit-symbolic-extents-at-the-compiler-request-boundary]
related: [deliver-an-artifact-family-from-a-symbolic-region, bind-repeated-invocations-over-caller-retained-tensors, accept-the-live-extent-operand-public-surface, carry-live-extent-operands-through-the-artifact-envelope, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n, prove-a-schedule-verified-live-contraction-consumes-s, admit-symbolic-extents-through-compiler-region-formation]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, kernel, artifact, runtime, identity, public-boundary]
---
## User-visible outcome

One compiled payload consumes a live symbolic input extent in its address and loop arithmetic, so changing `C` or `S` changes what the kernel indexes without compiling another payload or pipeline.

## Exact gap and ownership

**Fact at `b4e3478d42ce21ed68e23f772b643c6370d36498`; re-verified at `c99ac54950f242d88d8dfe8335332bef0cf75f2d`.** `AbiRoot::InputExtent` already lets artifact expressions evaluate accessible ranges, guards, preconditions, and launch geometry from runtime-bound extents. `place_bindings` evaluates binding expressions into `RoutedBinding::{accessible_offset, accessible_bytes}` (with transport and binding). Launch geometry is published separately by `evaluate_launch` as `RoutedLaunch` on `RoutedEntry::launch`, not by `place_bindings`. `BufferParameter` plus admitted launch builtins are the complete structured-kernel/Metal parameter population; no live input extent reaches the kernel body. The symbolic compiler-request ticket promises only that a program reaches planning or declines by the right reason, and the artifact-family delivery ticket discusses host-side range/launch evaluation. Neither owns a payload-consumable live scalar. This ticket is the missing prerequisite, not extra layout work.

**Correction — 2026-08-10.** The prior place_bindings sentence attributed launch publication to `place_bindings`; launch is `evaluate_launch` / `RoutedLaunch`. Wording matches `docs/research/runtime/dynamic-kv-physical-layout.md`. Reproduce: `rg -n "fn place_bindings|fn evaluate_launch" crates/tiler-runtime/src/load.rs`.

An input extent is resolved from the program interface's existing `AbiRoot::InputExtent`; it is never derived from a workload-named storage descriptor, and after the 2026-08-04 KV supersession no such descriptor exists to derive one from. The dynamic-KV layout research selected exact-live dense packing in capacity-sized pooled buffers, so `C` and `S` themselves are the only address operands that path needs; no physical layout-root carrier follows this ticket.

## Scopes note — 2026-08-10

`contracts/integrations` was removed: this ticket does not own frontend-integration contract prose (`docs/integration/**`); deliver owns that path under that scope. Payload compilation subject identity is carried transitively via kernel identity bytes folded into compilation identity; `implementation/metal-aot` is not declared unless a worker must edit `crates/tiler-metal-aot/**` directly (add that scope only if such an edit is required).

## Required work

- Add a bounded typed structured-kernel operand for an existing governed input-extent root. Prove its symbol/interface/axis association against the scheduled region and refuse undeclared, foreign, late-phase, unused, duplicate, wrong-axis, wrong-type, or unbounded occurrences.
- Lower symbolic address and loop expressions to that operand without specializing its live value. Remove the migrated implicit-static path rather than letting one kernel choose between a baked extent and a parameter.
- Carry the root declaration, use sites, type, phase, canonical order, and read-only parameter transport through kernel identity, Metal emission, payload compilation subject, artifact construction/codec/decode/validation, and routed preflight. Runtime binds it from the same authoritative `AbiFacts` input extent used by range and launch evaluation; callers do not provide another list.
- Freeze canonical parameter bytes before `RoutingCommit`; the committed authority owns them and the backend binds exactly the declared transport. Missing backend support refuses during preparation, while a post-commit mismatch is terminal.
- Execute every kernel/program/artifact/schema identity step whole with ledgers and recomputed pins. The live extent value is excluded from artifact, payload, library, and pipeline identity.
- Make `deliver-an-artifact-family-from-a-symbolic-region` consume this capability rather than claiming that host-side launch/range evaluation makes a symbolic payload executable.

## Required evidence

- One artifact, payload subject, and pipeline handles dense F32 `[2,N]` at `N = 14` and `N = 15`; semantic `(row = 1, column = 0)` addresses bytes 56 and 60 respectively from the bound input extent.
- A bounded direct contraction consumes `S` as its contributor-loop bound and performs exactly `S` loads. Replacing the bound value by the neighbouring extent changes the oracle, while baking either value changes identity and fails the no-specialization assertion.
- Omitted, swapped-symbol, wrong-axis, late-phase, overflowing, misordered, and backend-misbound operands fail at the named layer. Remove each new check and watch its negative fail.
- Existing artifact range and launch expressions resolve from the same bound fact; a deliberate disagreement between host-side and payload use refuses before program work rather than running two meanings of `S`.
- Targeted IR/compiler/artifact/Metal/runtime/build tests, exact identity blast radius, `tkt lint`, `git diff --check`, guard, and the full gate pass.

## Unsupported cases

Only bounded unsigned input-axis extents available by `LiveDevicePreflight`. Dynamic target properties, physical layout facts, arbitrary caller scalars, unbounded loops, ragged per-row extents, negative values, and cursor/capacity specialization are outside this ticket and refuse.

## Public boundary

The exact structured-kernel operand, artifact row/view, routed parameter, and adapter binding methods are consequential public/schema surfaces. Produce the tested draft and put it to Tom; do not self-accept or release dependents before acceptance.

## Closes when

The labelled draft at `9a8f53c937dc9b9f777a1d4b361cadc1a0b0316e` is integrated after Tom accepts the public surface, every remainder below is `done`, and both [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md) and [`bind-repeated-invocations-over-caller-retained-tensors`](bind-repeated-invocations-over-caller-retained-tensors.md) can consume the capability without a second scalar authority. Do not set this ticket `done` on the draft commit alone.

## Split — 2026-08-13 at base `209e0f9fd5a18486039d859a5f47ccf260f0f8cf`

Review of `9a8f53c9` stopped short of a complete outcome. The kernel, schedule, Metal, and routed-runtime draft is preserved on `tkt/admit-live-extent-operands-to-payload-indexing`. The remainder is split rather than merged as done.

- [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md) — Tom packet; `awaiting-decision`; only Tom closes it.
- [`carry-live-extent-operands-through-the-artifact-envelope`](carry-live-extent-operands-through-the-artifact-envelope.md) — envelope construction/codec/decode/validation, blocked on acceptance.
- [`prove-one-live-extent-artifact-payload-and-pipeline-at-two-n`](prove-one-live-extent-artifact-payload-and-pipeline-at-two-n.md) — `N = 14` / `N = 15` artifact, payload, and pipeline evidence.
- [`prove-a-schedule-verified-live-contraction-consumes-s`](prove-a-schedule-verified-live-contraction-consumes-s.md) — `LiveContraction` E2E, blocked on acceptance.
- [`admit-symbolic-extents-through-compiler-region-formation`](admit-symbolic-extents-through-compiler-region-formation.md) — `compile()` path. **Correction of the review comment:** the first refuse is strategy selection (`RequestError::UnsupportedSymbolicExtent`, `rule: "symbolic-extent"` in `crates/tiler-compiler/src/request.rs`), not region formation. Region formation is the later fail-closed gate.

Do not merge `9a8f53c9` as done. Do not release the deliver or bind-repeated dependents until the accepted capability exists.
