---
id: pin-the-bf16-spike-admitted-operation-verdict-to-its-own-enum
title: Pin the bf16 spike's admitted-operation verdict to its own enum
status: done
priority: p2
dependencies: []
related: [replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The verdict is a literal tautology

Split out of `replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses`, whose scope is `implementation/cache` and could not reach `spikes/numerics/**`. That ticket repaired the other three findings; this is the fourth, **re-verified by reading both files in full on 2026-08-07 at base `aebd16c0`** rather than relayed from the audit.

**Fact.** `spikes/numerics/bf16-second-dtype/src/perturb.rs:201-217`:

```rust
pub fn fused_operations_are_unexpressible() -> Perturbation {
    let admitted = [Operation::Multiply, Operation::Add];
    Perturbation {
        subject: "the admitted operation vocabulary",
        detected: admitted.len() == 2,
```

`admitted` is a local array literal two lines above, so `admitted.len() == 2` is `2 == 2`, decided at compile time. There is **no falsifying input**: the function takes no arguments and reads no state, and no `variant_count`, exhaustive match, or any other construct links it to `Operation`.

**Fact.** `Operation` (`src/corpus.rs:31-58`) is a fieldless two-variant enum — `Multiply`, `Add` — with exhaustive matches in `apply` and `as_str`. Adding `Operation::FusedMultiplyAdd` forces an arm in each of those two (a build error until written) and leaves this function reporting `[DETECTED]` alongside the nine perturbations in the same module that genuinely can fail — in a module whose own header reads "Every check this spike reports would still pass if it could not fail."

## The repair

Declare the array at its own enum's variant count, so a widened `Operation` is a build error **at the claim** rather than a green tautology beside it:

```rust
const ADMITTED: [Operation; mem::variant_count::<Operation>()] =
    [Operation::Multiply, Operation::Add];
```

and assert the entries are pairwise distinct, which is what stops the length from being satisfied by a repeated variant that left the new one unlisted. `detected` then states the distinctness and the absence of a fused variant rather than an array length.

**`variant_count` is sound for this enum, and the check is not optional.** The nuance that made `variant_count` the *wrong* recommendation elsewhere in the same audit — a payload-carrying enum, where it yields a smaller number and a green test — does not apply: `Operation` carries no payload, so `variant_count::<Operation>()` is exactly 2 today.

`crates/tiler-metal` already uses this idiom (`#![feature(variant_count)]` at the crate root; `const OPS: [BinaryOp; variant_count::<BinaryOp>()]` in `src/tests.rs`), and `crates/tiler-cache` adopted it for `Phase::KILL_POINTS` under the parent ticket, gated as `#![cfg_attr(test, feature(variant_count))]`. Here it is needed in the binary itself, so the gate is unconditional at `src/main.rs`.

## Scope and how to run it

`research/numerics` (`docs/research/numerics/**`, `spikes/numerics/**`). **The spike is its own workspace root** (`[workspace]` in its `Cargo.toml`) and `make` never reaches it, so the gate does not cover this change. Run it manually from the spike directory, as `spikes/numerics/bf16-second-dtype/README.md` documents:

```sh
CARGO_TARGET_DIR=./target cargo run
```

The binary's only product is a verdict and every failing stage exits non-zero, so the run is the check.

## Watch it fail

Add `Operation::FusedMultiplyAdd` with its `apply` and `as_str` arms and confirm the build now fails at the array declaration; then restore. Quote the message. Confirm the unperturbed spike still exits zero.

## Outcome — done, 2026-08-07

Landed at merge `ec71c2f9` (worker commit `2bd50e89`). `spikes/**` only, carries the green gate.

### The repair sketch in this ticket was the wrong instrument, and the worker said so

It proposed `ADMITTED.len()` plus a separate pairwise-distinctness assertion. **Distinctness was the right concern and the wrong mechanism**: a length is still a length, and the residual verdict would remain a compile-time-true expression that no longer names what it is about.

What landed instead is an array **pattern** — `const ADMITTED: [Operation; variant_count::<Operation>()]` with `detected: matches!(ADMITTED, [Operation::Multiply, Operation::Add])`. That subsumes distinctness (two entries repeating one variant cannot match) and turns "the author updated the list" from a red line into a **build error at the claim**.

### The finding reproduced, which is what makes the repair trustworthy

Three configurations, each perturbing the subject:

1. Variant added, `ADMITTED` untouched → **two** compile errors, `E0527` on the pattern and `E0308` on the array length.
2. Variant also listed in `ADMITTED` → length satisfied, and the claim **still refuses**: `E0527`, exit 101.
3. **The pre-repair file against the same widened enum** → compiles with only a `dead_code` warning, exits **0**, and prints `[DETECTED] … no fused multiply-add variant exists` and `VERDICT: every stage agreed and every perturbation was detected` — while `Operation::FusedMultiplyAdd` exists in the enum.

That third run is the defect demonstrated rather than argued.

### `variant_count` is sound here, and the worker checked rather than assumed

`Operation` is `enum Operation { Multiply, Add }` — fieldless, no explicit discriminants, not `#[non_exhaustive]` — so `variant_count` counts its inhabitants exactly. **The payload trap that broke the sibling recommendation does not apply**, and that was verified rather than inherited from the brief.

### Two things it surfaced

**This ticket's `shared_scopes` was `[]`**, contradicting the brief, so the worker correctly declined to edit the ticket at all rather than take a guard escape. Now corrected to `[project/tickets]` — the same class of gap found on two other tickets today.

**The spike's nested `Cargo.lock` had drifted**: the `tiler-digest` extraction landed in `crates/` without updating it, so `tiler-artifact` and `tiler-ir` now reach `tiler-digest` rather than `sha2` directly. The first `cargo run` rewrote it. The README treats that lock as the recoverable record of the dependency set a recorded run was taken under, so it was committed rather than left dirty. Note the gated `Cargo.lock` is the **root** one — this nested lock is never read by any `make` recipe, since no recipe reaches the spike.

The spike's own run is exit 0 with all ten perturbations detected, and its 1.0 GB local `target/` was removed afterwards.
