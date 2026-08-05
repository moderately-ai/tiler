---
id: widen-compile-governed-s-error-to-the-target-compile-failure
title: Widen compile_governed's error to the target compile failure
status: todo
priority: p2
dependencies: []
related: [accept-the-public-compiler-facade-boundary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`session::compile_governed` stops silently discarding the typed refusal detail the general path retains, so a caller diagnosing a refusal through the convenience entry reads the same explanation as through `session::compile` — and the one facade item Tom excluded from the 2026-08-05 acceptance returns as a small delta.

## Why this exists

**Fact.** `compile_governed` ends in `.map_err(|failure| failure.failure)`, unwrapping the single-target wrapper (legitimate) but dropping `TargetCompileFailure::refusal` — the typed pre-trace detail naming which rule refused and why — with a doc comment that says nothing about the loss. **Fact.** The loss is reachable today: `NumericalContract::STRICT_BF16` against the governed profile produces exactly such a refusal, and `crates/tiler-compiler/tests/bf16_numerical_contract.rs` asserts the detail's content through the general path. **Fact — symptom, not design.** Both real producers migrated off this entry with fail-closed guards against returning; its remaining callers are three test call sites plus spike workspaces; deliberate narrowings in this tree state their reason and this one is silent. The facade acceptance packet (in `accept-the-public-compiler-facade-boundary`) carries the full derivation, including decline-entirely as the recorded second answer — this ticket takes the widening because spikes genuinely want a no-ceremony governed entry and the fix is one error-type change.

## What this ticket owes

- The error type widened so the refusal detail survives — the packet's named shape is returning `TargetCompileFailure` (or an equivalent that preserves `refusal`); run the small elimination at the code and state it.
- The doc comment rewritten to describe current behaviour, including the single-target unwrapping that *is* deliberate.
- The three in-crate test call sites updated; spike callers are other tickets' scopes — if the signature change breaks a spike, note it for the spike-restoration thread rather than editing spikes here.
- Watched-failing evidence: a STRICT_BF16 refusal through `compile_governed` carrying the same typed detail the general path reports, with the assertion observed failing against the pre-fix shape.
- Report the surface delta so the coordinator appends it to the facade acceptance as the excluded item's return.

## Non-goals

Any pipeline change (the two entries run identical compilation); removing the entry (recorded as the defensible second answer, not taken); accepting anything (the delta returns to Tom).

## Closes when

The refusal detail survives the convenience entry with the watched-failing evidence, the doc describes the behaviour, no caller sees a silent loss, and the delta is reported for acceptance.
