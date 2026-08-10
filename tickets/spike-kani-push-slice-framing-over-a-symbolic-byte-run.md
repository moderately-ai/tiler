---
id: spike-kani-push-slice-framing-over-a-symbolic-byte-run
title: Spike Kani proof of push_slice framing over a symbolic byte run
status: in-progress
priority: p2
dependencies: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder, prove-the-exhaustible-encoder-injectivity-claims-natively]
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [verification, spike, kani, identity, length-framing]
claimed_from: todo
assignee: sol-kani-framing
lease_expires_at: 1786409872
---

## User-visible outcome

A bounded Kani experiment under `spikes/verification/` (or an extension of the existing kani-encoder-injectivity spike) that proves the length-prefix framing property of `push_slice` over a symbolic byte run — the recovery path for the nine string encoders whose injectivity is blocked by `String::from_utf8` over symbolic bytes rather than by encoding logic.

## Why this exists

`spike-kani-bounded-verification-on-one-inexhaustible-encoder` measured that `push_numerical_injective_key_len_0` exceeds a 900 s cap at the smallest symbolic-key bound (empty key) without reaching SAT, while the same encoder with a concrete 30-byte key discharges in ~1.46 s. Traces attribute the cost to `core::str::run_utf8_validation`, not the encoder. The parent Outcome and research record name the decomposition: prove the numerical tail with the key fixed (done there), and prove the key's framing separately as a property of `push_slice` — one primitive shared by all nine string encoders on the native-sweep inexhaustible list. That decomposition was **not** attempted on the parent spike; the parent's close-time "Filed as the next bounded experiment" wording was false until this ticket was filed on 2026-08-10 (see parent Correction).

Primary sources: parent ticket Outcome (done, 2026-08-07); `docs/research/verification/kani-bounded-encoder-verification.md` section on the string encoder; `spikes/verification/kani-encoder-injectivity/README.md`.

## The spike, when it runs

- Reuse the host Kani install already authorized for the parent spike (record version + bundled nightly). Do not request a new host toolchain.
- One `#[cfg(kani)]` harness (or small family) proving injectivity / prefix-freeness of `push_slice` over a symbolic byte run with an explicit length bound stated as the proof's domain boundary.
- State how the bound argument differs from the complete-copy proofs on finite-width encoders (a length prefix is prefix-free by construction only up to the quantified prefix bound).
- Prefer tying the harness to live `crate::identity::push_slice` if the primary path unblocks; otherwise a guarded copy with the same provenance discipline as the parent spike (`guard.sh` or successor), recording that a shim proof proves a copy.
- Record: proof runtime, unwind bounds, whether CBMC's unwinding assertion discharges, and how the result classifies relative to the parent's "complete proof of a copy" vs `SoundProof`-with-bound taxonomy question (still routed to Tom; do not decide the class here).
- Hand-run from the spike directory; no make target; README records the invocation.

## Closes when

A measured Kani verdict (success, timeout, or blocked-with-diagnostic) for the framing property is recorded under `spikes/verification/` with a research write-up or an extension of the existing Kani research record, the domain bound is explicit, and any copy/provenance limits are stated.

## Non-goals

- Unblocking `cargo kani -p tiler-ir` (still gated on Kani/toolchain convergence; re-probe is one command after a new Kani release, not this ticket).
- Re-syncing the parent spike's `ResourceRequirements` / `push_resources` copies after `IndexArithmetic` (separate maintenance remainder if filed).
- Deciding the evidence-taxonomy class for complete-copy proofs.
