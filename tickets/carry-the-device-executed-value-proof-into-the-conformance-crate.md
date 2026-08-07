---
id: carry-the-device-executed-value-proof-into-the-conformance-crate
title: Carry the device-executed value proof into the conformance crate
status: in-progress
priority: p1
dependencies: [conform-the-bf16-vertical-end-to-end]
related: [admit-the-conformance-crate-to-the-workspace, decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access, publish-an-l3-contraction-cell-through-the-accepted-route, integrate-the-contraction-vertical-into-the-runtime, survey-what-belongs-in-the-conformance-crate]
scopes: [implementation/conformance, implementation/runtime, implementation/workspace, implementation/cargo-lock]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, runtime, migration]
claimed_from: todo
assignee: agent-value-proof
lease_expires_at: 1786125786
---
## User-visible outcome

The workspace's one cross-layer *executed* comparison — a program built in the shared IR, planned, lowered, compiled to a metallib, packaged, validated, dispatched on a real device, and compared bit for bit against `tiler-reference` — becomes a red test in `make full` instead of a `cargo run` somebody remembers to type.

## Why this exists

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which read the population.

**Fact — the run already exists and it is outside the gate.** `prototypes/serial-sum-run/src/proof.rs` is 8,159 lines. Its `run()` narrative dispatches on the GPU by two independent paths (a direct in-memory metallib, and an envelope decoded by `tiler-runtime`) and compares both against `tiler-reference`'s evaluation of the same semantic program. It is reached only by `cargo run -p tiler-prototype-run -- --artifact <base>`. `Makefile`'s `test` target is `cargo nextest run --workspace --locked` plus `cargo test --workspace --doc`; `full` adds rustdoc, a release run of two packages, `ticketsplease lint`, and `shellcheck`. No target invokes that binary. Reproduce with `grep -n "prototype-run" Makefile`, which returns only the two `--exclude` flags on the Clippy line.

**Fact — what *is* gated there is device-free by construction.** `proof.rs`'s `#[cfg(test)] mod tests` (from `:5402`) states in its own header that it is a loader fixture which "reaches no device", substituting a synthetic payload for a real `xcrun` link. That module runs in the gate. The device half does not.

**Fact — the tree it lives in is deliberately held to a lower bar.** `Makefile`'s `lint` recipe excludes `tiler-prototype-run`, `tiler-prototype-compile`, and `tiler-prototype-candle` from `cargo clippy -- -D warnings`, with a comment recording that prototypes are "non-published, experimental, and deleted or rewritten as the slice they prove moves". [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s Context is that this is the wrong home for load-bearing conformance, and Tom decided the crate on exactly that ground.

**Fact — the evidence at stake is not hypothetical.** [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md) (done, 2026-08-05) dispatched `w_decode_kv` (`1x1024x1024`) through the accepted AOT and runtime route and matched the retained `direct` SHA-256 `79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f` from the M4 Max probe record, after checking six environment fields against the record's own `environment.tsv`. [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) (done) carries five operand cases bit-compared against the reference, including the `negative-zero-fold` `0x80000000` counterexample. [Correctness and testing](../docs/correctness-and-testing.md#reduction-matrix) records the corpus's only device observation of a different-but-permitted reassociated answer, driven through this same binary. Every one of those is a `cargo run` away from being unnoticed drift.

## What moves

The device-reaching narrative and the comparisons it makes: the serial-sum value proof's direct and envelope paths, the contraction member, the `contraction-w-decode-kv` L3 cell with its retained-digest comparison, the fail-closed envelope probes as they apply to a *real* artifact, the host-observation and applicability refusal, and the grouping-sensitive reassociation case.

It arrives as `#[test]`s, not as a binary narrative. The reporting obligation [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) item 1 states travels with it: a host without the measured environment runs the deterministic reference and structural half and **reports the measured half as unavailable, naming what was missing** — never a silent skip, never a claimed pass. The existing `RetainedComparison` unavailable-predicate shape and the six-field environment match in `publish-an-l3-contraction-cell-through-the-accepted-route` are the precedent, not a new invention.

## What does not move, and where each goes instead

- **`proof.rs`'s `#[cfg(test)] mod tests` loader fixture.** Layer-local: it assembles a synthetic envelope to assert `tiler-runtime`'s refusal *classes*. Its own header names `tiler-runtime` as "the better home", eliminated at the time only because a `tiler-ir` dev-dependency edits `Cargo.lock`. This ticket holds `implementation/cargo-lock` and `implementation/runtime`, so relocating it into `crates/tiler-runtime/tests/` is **in scope here** and is the disposition. Do not carry it into the conformance crate: it reaches no device and compares nothing against the oracle, so it would be exactly the layer-local test [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s third anti-goal refuses.
- **`prototypes/serial-sum-compile`.** It is the *producer*, and this ticket does not hold `implementation/metal-aot`. If the run is to produce its artifact in process rather than read the producer's file, that is a second ticket and a scope this one must not pre-declare.

## The unsafe rule this inherits, and it is decided rather than open

[`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md) is `done`: **`deny`, with named `#[allow(unsafe_code)]` exceptions at individual sites, never at the crate**, and the only admitted justification is FFI memory management with Metal — concretely the raw pointer `metal::Buffer::contents` returns. A **single narrow module** owns every unsafe site and exposes a safe API; the conformance logic itself contains no `unsafe`. The site population is named and counted where a reader can find it. `prototypes/serial-sum-run/src/buffer.rs` is the shape and is 89 lines with two reasoned sites — carry it, do not re-derive it. The crate manifest's comment above `[lints]` must be replaced by the decision.

If [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) has already landed that module, this ticket extends its site population rather than opening a second one, and states the new count.

## The fork this ticket must not decide

**What remains of `prototypes/serial-sum-run` afterwards is Tom's.** [ADR 0056](../docs/decisions/0056-use-four-libraries-and-two-proof-executables.md) admits proof executables, and [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s Consequences record "three non-published proof/integration executables" and that this member "remains the only member whose code reaches a live device". Emptying it changes both statements. Present the fork with the diff in hand — retire the member, keep it as a thin demonstration driver over the conformance crate's own machinery, or keep it whole and accept the duplication — and let Tom pick. Do not delete the member on a worker's judgement.

## Closes when

The device-executed comparisons run under `cargo nextest run --workspace` from `crates/tiler-conformance`, an unmeasured host reports the measurement boundary as unavailable and names what was missing rather than skipping or passing, the L3 cell's retained-digest comparison was watched refusing under a deliberate perturbation before it was trusted, the unsafe site population is named and counted, the loader fixture has been relocated to `crates/tiler-runtime/tests/`, and the fork above is presented to Tom rather than resolved.
