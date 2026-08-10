---
id: spike-kani-push-slice-framing-over-a-symbolic-byte-run
title: Spike Kani proof of push_slice framing over a symbolic byte run
status: done
priority: p2
dependencies: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder, prove-the-exhaustible-encoder-injectivity-claims-natively, catalog-the-kani-push-slice-framing-research-and-spike]
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [verification, spike, kani, identity, length-framing]
---

## User-visible outcome

A bounded Kani experiment under `spikes/verification/` (or an extension of the existing kani-encoder-injectivity spike) that proves the length-prefix framing property of `push_slice` over a symbolic byte run — the shared framing half of the recovery path for the nine string-encoder **categories** in the native sweep. The predecessor measured the `String::from_utf8` obstacle on `push_numerical`; extending that cost attribution to the other categories is an inference, not nine measurements. The nine count names encoder categories, not concrete function instances or individual string fields: `push_numerical` has three copies, while several of the named categories encode more than one string.

## Why this exists

`spike-kani-bounded-verification-on-one-inexhaustible-encoder` measured that `push_numerical_injective_key_len_0` exceeds a 900 s cap at the smallest symbolic-key bound (empty key) without reaching SAT, while the same encoder with a concrete 30-byte key discharges in ~1.46 s. Traces attribute the cost to `core::str::run_utf8_validation`, not the encoder. The parent Outcome and research record name the decomposition: prove the numerical tail with the key fixed (done there), and prove the key's framing separately as a property of `push_slice` — one primitive shared by all nine string encoders on the native-sweep inexhaustible list. That decomposition was **not** attempted on the parent spike; the parent's close-time "Filed as the next bounded experiment" wording was false until this ticket was filed on 2026-08-10 (see parent Correction).

Primary sources: parent ticket Outcome (done, 2026-08-07); `docs/research/verification/kani-bounded-encoder-verification.md` section on the string encoder; `spikes/verification/kani-encoder-injectivity/README.md`.

## The spike, when it runs

- Reuse the host Kani install already authorized for the parent spike (record version + bundled nightly). Do not request a new host toolchain.
- One `#[cfg(kani)]` harness (or small family) proving injectivity / prefix-freeness of `push_slice` over a symbolic byte run with an explicit length bound stated as the proof's domain boundary.
- State how the bound argument differs from the complete-copy proofs on finite-width encoders: fixed-width length framing is prefix-free for every representable slice length, while the Kani result quantifies only the payload lengths and bytes admitted by the harness's explicit bound.
- Prefer tying the harness to live `crate::identity::push_slice` if the primary path unblocks; otherwise a guarded copy with the same provenance discipline as the parent spike (`guard.sh` or successor), recording that a shim proof proves a copy.
- Record: proof runtime, unwind bounds, whether CBMC's unwinding assertion discharges, and how the result classifies relative to the parent's "complete proof of a copy" vs `SoundProof`-with-bound taxonomy question (still routed to Tom; do not decide the class here).
- Hand-run from the spike directory; no make target; README records the invocation.

## Closes when

A measured Kani verdict (success, timeout, or blocked-with-diagnostic) for the framing property is recorded under `spikes/verification/` with a research write-up or an extension of the existing Kani research record, the domain bound is explicit, and any copy/provenance limits are stated.

## Non-goals

- Unblocking `cargo kani -p tiler-ir` (still gated on Kani/toolchain convergence; re-probe is one command after a new Kani release, not this ticket).
- Re-syncing the parent spike's `ResourceRequirements` / `push_resources` copies after `IndexArithmetic` (separate maintenance remainder if filed).
- Deciding the evidence-taxonomy class for complete-copy proofs.

## Fact audit — 2026-08-10, exact base `49d38237`

The parent measurements, UTF-8 cost attribution, unattempted decomposition, current Kani 0.67.0 install, current nine-error live-crate codegen refusal, and unrelated two-item parent-guard drift all re-verified. The native sweep names nine string-encoder **categories**, all reaching `crate::identity::push_slice`; it does not name nine concrete function instances or nine individual string fields. The predecessor measured symbolic-UTF-8 cost only for `push_numerical`, so the user-visible outcome now distinguishes that measurement from the inference that the other categories would pay the same construction cost.

**Correction.** The earlier instruction that “a length prefix is prefix-free by construction only up to the quantified prefix bound” was imprecise. The eight-byte fixed-width length prefix makes the construction prefix-free for every representable slice length. What is bounded is the model check: its symbolic arrays and lengths quantify only the harness domain up to `N`. This correction narrows the evidence claim without changing the spike's purpose or authority.

## Spike result — 2026-08-10

Reproduction: `spikes/verification/kani-push-slice-framing/README.md`. Research interpretation: `docs/research/verification/kani-push-slice-framing.md`.

**Measurement.** Two Kani 0.67.0 / CBMC 6.8.0 harnesses quantify every ordered pair of byte runs of length 0 through 4. Across repeated runs on the active coordination host, injectivity discharged 381 checks in 3.51–8.58 s wall and strict-prefix freedom discharged 375 checks in 3.36–10.49 s wall; both reported 0 failures, 6 unreachable checks, and `memcmp.unwind.0: SUCCESS`. The explicit domain contains 4,311,810,305 semantic byte runs; five bytes and longer are outside the Kani result.

**Provenance.** The live-crate stop condition still produces the same nine compilation errors, so the spike proves copies of `push_len` and `push_slice`. A new independent guard compares exactly those two copies to `crates/tiler-ir/src/identity.rs` and succeeds with population 2. The predecessor guard's `ResourceRequirements` / `push_resources` failure remains 2 of 28 and is unrelated.

**Deliberate failures.** Removing the copied payload append made the independent guard report `DRIFT: push_slice` / `1 of 2 framing copies have drifted` and made Kani report `Failed Checks: "equal framing must carry equal active bytes"`. Restoring it and removing the copied length write made the guard fail on the same subject and made Kani report `Failed Checks: "one framed byte run is a strict prefix of another"` twice. Both subjects were restored; the final guard is green.

**Decision boundary.** This result supplies a bounded shared-framing lemma over a guarded copy. It does not decide its evidence-taxonomy class and does not prove every non-string tail in the nine categories. Catalog changes live outside this ticket's scope and are preserved in [`catalog-the-kani-push-slice-framing-research-and-spike`](catalog-the-kani-push-slice-framing-research-and-spike.md).
