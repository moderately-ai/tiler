---
id: resync-the-kani-encoder-injectivity-shim-after-index-arithmetic
title: Resync the Kani encoder-injectivity shim after IndexArithmetic
status: in-progress
priority: p3
dependencies: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [verification, spike, kani, identity, maintenance]
claimed_from: todo
assignee: sol-kani-resync
lease_expires_at: 1786414719
---

## User-visible outcome

The `spikes/verification/kani-encoder-injectivity/` shim copies and harness domains match live sources again after `IndexArithmetic` landed on `ResourceRequirements` / `push_resources`, so `./guard.sh` exits 0 and the complete `push_resources_*` Kani proofs attach to current encoder text (or the spike documents a deliberate freeze with a re-probe condition).

## Why this exists

The parent spike's `guard.sh` is a text-tie over 28 copied items. At audit base and current tree it fails with two real drifts: live `ResourceRequirements` gained `index_arithmetic: IndexArithmetic`, and `push_resources` writes `index_arithmetic_tag(...)` before synchronization. The guard is doing its job; the land-time complete `push_resources_injective` and `push_resources_prefix_free_tail_4` proofs no longer attach to live source text. Tensor-role and component-role copies still match. No product crate change is required solely to restore the tie — only spike copies, harness domain arithmetic, and re-run measurements under the already-authorized host Kani install.

Reproduce: `cd spikes/verification/kani-encoder-injectivity && ./guard.sh` → exit 1, `2 of 28 copied items have drifted from their sources` naming `ResourceRequirements` and `push_resources`.

## The work

1. Update the shim copies of `ResourceRequirements`, `IndexArithmetic` (and any newly required helpers/tags such as `index_arithmetic_tag`), and `push_resources` so token content matches live sources under the existing `@source:` markers.
2. Adjust harness domain counts if the new field multiplies the input domain (land-time domain lacked `IndexArithmetic`; a single-variant enum scales by 1 but changes encoding width and unwind needs).
3. Re-run `./guard.sh` (expect exit 0 and population assert still 28 or an updated asserted population with the same fail-closed discipline).
4. Re-run the affected Kani harnesses (`push_resources_injective`, `push_resources_prefix_free_tail_4`, and any new related proof); record wall/CBMC times, check counts, and unwinding-assertion results on the stated host.
5. Update the spike README (and research record if domain claims move) so land-time domain arithmetic is not left as a live claim about current sources.
6. Watch `guard.sh` fail on at least one planted drift before trusting the re-sync.

## Closes when

`./guard.sh` is clean, the re-run measurements are recorded, and the complete resources proofs either re-attach to live sources or the record states why they do not.

## Non-goals

- Changing production encoders or identity domains.
- Filing a make-gated guard (parent deliberately left the guard un-gated; reopening that is a separate product choice for Tom).
- The `push_slice` symbolic-byte framing experiment (`spike-kani-push-slice-framing-over-a-symbolic-byte-run`).
