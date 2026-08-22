---
id: harden-the-physical-selection-run-s-length-frame-and-its-limit-census
title: Harden the physical-selection run's length frame and its limit census
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, unfireable-checks]
claimed_from: todo
assignee: worker-harden
lease_expires_at: 1787443883
---
## User-visible outcome

The physical-selection run's wire framing is checked in release builds rather than only in debug, and a decoder-limit kind added later fails the test that claims to census them.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit. Neither item is a live defect; both are guards weaker than they read.

**Fact — the run's length frame is computed arithmetically and the encoder's own guard is debug-only.** `crates/tiler-artifact/src/program/model.rs`, anchor `push_len(bytes, row.canonical_key_bytes());`. Every sibling run (deferred keys, payload keys, route requirements) materializes each key and frames it with `push_slice`, so the length is derived from the actual bytes. The irrefutable destructure catches field *addition* but not a change to an existing field's framing.

Census re-derived 2026-08-22 at base `b6248f91`: **verified**. `push_len(` occurs **76 times** in `crates/tiler-artifact/src` (unit: occurrences, `grep -ro 'push_len(' crates/tiler-artifact/src | wc -l`). Enumerating the argument of every one, 75 pass either an element count or the `.len()` of the very byte slice about to be written (`section.bytes.len()`, `manifest.len()`, `payload.len()`), plus three test literals; `row.canonical_key_bytes()` is the sole argument derived arithmetically rather than from bytes that already exist. The three sibling `debug_assert_eq!(bytes.len(), exact, ...)` guards at the provider, payload, and deferred key encoders are a *different class*: their sizing feeds only `Vec::with_capacity`, and the caller frames the returned `Vec` with `push_slice`, so a mismatch there costs a reallocation and cannot misframe.

**Correction (2026-08-22, worker-harden).** Two claims this ticket previously carried were not supported when measured; they are restated here rather than repeated above.

- *Withdrawn as stated:* that a framing disagreement makes "a decoder recover a different artifact rather than failing". Perturbing the subject at base — writing `provider.name()` with `extend_from_slice` instead of `push_slice`, so the declared frame exceeds the bytes written by 8 — and running `cargo nextest run -p tiler-artifact --release`, the decoder **refused**: `the_run_round_trips_through_both_read_views` failed at its `expect("its own bytes decode")`, and four refusal tests reported `Limit { resource: TextBytes, actual: 8314596455175302504, limit: 4096 }`. So the observed failure is a refusal that names the **wrong subject** and a nonsense length — identity bytes read as a text budget — not a silently different artifact. Whether some other desync could realign into a valid-but-different artifact was not tested and is unproven in either direction; the hardening rests on the refusal being unnameable, not on silent acceptance.
- *Imprecise as stated:* "guarded only by `debug_assert`". The **encoder** guard is debug-only, but `program::tests::selected_physical_implementations::the_row_key_is_its_exact_seven_field_grammar` carries a release-active `assert_eq!(key.len(), row.canonical_key_bytes(), "the arithmetic sizing and the written bytes are one definition")`, and it failed in that same release run with `left: 153, right: 161`. Because `canonical_key_bytes` is shape-independent, any structural desync reaches that fixture row too. So the gate already reddens; what was missing is fail-closed behaviour **at the write site** in a release binary, so the encoder refuses to emit a misframed artifact rather than emitting one and leaving its own decoder to reject it under an unrelated budget.

Cost claim re-derived, **verified**: the design's argument (`push_canonical_key` doc, anchor `is what keeps a multi-MiB identity from being copied twice`) is about not materializing the key into a temporary `Vec`. Promoting the guard adds one further `canonical_key_bytes()` call per row — six `as_bytes().len()` reads over `Arc<[u8]>` borrows plus six adds and one comparison, copying nothing — with rows bounded by `MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS` (= `MAX_VARIANT_ENTRIES` = 4,096). Nothing in it scales with identity size, so the copy argument does not reach it.

**Fact — a check whose docstring claims a census it does not implement.** `crates/tiler-artifact/src/program/codec/tests/selected_physical_implementations.rs`, anchor `no_physical_specific_decoder_byte_budget_is_declared`. Its docstring says it is *"a census of the limit vocabulary's own rendering rather than prose, so a kind added later fails here"* — but the body iterates a **hardcoded two-element array**, and `CodecLimitKind` has neither an `ALL` constant nor `variant_count` sizing. A kind added later passes silently. Worse, the loop's assertion cannot fail for the two kinds it does iterate; only the trailing rendering pin is real. `PhysicalProposalKind::ALL` in the same crate shows the correct shape.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict, including the "only such site in the crate" claim — **re-derive that census yourself and say which unit you report**. *(Done; see the census and correction above.)*
- Promote the framing guard so it holds in release, or say why it must not and what does hold instead.
- Size the limit census **from the type**. `core::mem::variant_count` makes a widened vocabulary a build error at the enumeration rather than a census that silently shrinks.
- **Perturb both, subject not assertion, and quote the output**: make `canonical_key_bytes` disagree with `push_canonical_key` and show the guard firing; add a `CodecLimitKind` variant and show the census failing. Before trusting either, state what it would take for it to say *no* and confirm that case is reachable.
- State whether any identity value moves. **Expected: none** — a guard and a census add no byte — but rederive, and **stop and report** if one does.

## Non-goals

Changing the run's encoding, ordering, or contents; re-deriving the v22 step; and any second budget authority, which the accepted packet forbids by name.

## Closes when

The framing disagreement is caught outside debug builds with the failure quoted, the limit census is sized from the type and watched failing on an added variant, and no identity value has moved.
