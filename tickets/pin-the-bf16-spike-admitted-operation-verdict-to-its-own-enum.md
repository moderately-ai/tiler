---
id: pin-the-bf16-spike-admitted-operation-verdict-to-its-own-enum
title: Pin the bf16 spike's admitted-operation verdict to its own enum
status: todo
priority: p2
dependencies: []
related: [replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses]
scopes: [research/numerics]
shared_scopes: []
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
