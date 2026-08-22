---
id: harden-the-physical-selection-run-s-length-frame-and-its-limit-census
title: Harden the physical-selection run's length frame and its limit census
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, unfireable-checks]
---
## User-visible outcome

The physical-selection run's wire framing is checked in release builds rather than only in debug, and a decoder-limit kind added later fails the test that claims to census them.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit. Neither item is a live defect; both are guards weaker than they read.

**Fact — the run's length frame is computed arithmetically and guarded only by `debug_assert`.** `crates/tiler-artifact/src/program/model.rs`, anchor `push_len(bytes, row.canonical_key_bytes());` — the audit reports this is the **only** such site in the crate. Every sibling run (deferred keys, payload keys, route requirements) materializes each key and frames it with `push_slice`, so the length is derived from the actual bytes. **If `canonical_key_bytes` and `push_canonical_key` ever disagree on framing, the encoded run is misframed and a decoder recovers a *different artifact* rather than failing** — precisely the failure the v22 schema step exists to prevent. The irrefutable destructure catches field *addition* but not a change to an existing field's framing.

Debug test runs do trip the assert, so there is no live defect. The cheap hardening is promoting it to `assert_eq!`: the design's cost argument is about avoiding multi-MiB *copies*, not about a `usize` comparison per row. **Re-derive that cost claim rather than accepting my summary of it.**

**Fact — a check whose docstring claims a census it does not implement.** `crates/tiler-artifact/src/program/codec/tests/selected_physical_implementations.rs`, anchor `no_physical_specific_decoder_byte_budget_is_declared`. Its docstring says it is *"a census of the limit vocabulary's own rendering rather than prose, so a kind added later fails here"* — but the body iterates a **hardcoded two-element array**, and `CodecLimitKind` has neither an `ALL` constant nor `variant_count` sizing. A kind added later passes silently. Worse, the loop's assertion cannot fail for the two kinds it does iterate; only the trailing rendering pin is real. `PhysicalProposalKind::ALL` in the same crate shows the correct shape.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict, including the "only such site in the crate" claim — **re-derive that census yourself and say which unit you report**.
- Promote the framing guard so it holds in release, or say why it must not and what does hold instead.
- Size the limit census **from the type**. `core::mem::variant_count` makes a widened vocabulary a build error at the enumeration rather than a census that silently shrinks.
- **Perturb both, subject not assertion, and quote the output**: make `canonical_key_bytes` disagree with `push_canonical_key` and show the guard firing; add a `CodecLimitKind` variant and show the census failing. Before trusting either, state what it would take for it to say *no* and confirm that case is reachable.
- State whether any identity value moves. **Expected: none** — a guard and a census add no byte — but rederive, and **stop and report** if one does.

## Non-goals

Changing the run's encoding, ordering, or contents; re-deriving the v22 step; and any second budget authority, which the accepted packet forbids by name.

## Closes when

The framing disagreement is caught outside debug builds with the failure quoted, the limit census is sized from the type and watched failing on an added variant, and no identity value has moved.
