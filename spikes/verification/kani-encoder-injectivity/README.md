---
schema: "tiler-doc/v1"
id: "tiler.spike.verification.kani-encoder-injectivity"
kind: "experiment"
title: "Kani bounded verification of inexhaustible identity encoders"
topics: ["verification", "kani", "identity", "injectivity", "toolchain"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.verification.kani-bounded-encoder-verification"]
entrypoints: ["spikes/verification/kani-encoder-injectivity/guard.sh", "spikes/verification/kani-encoder-injectivity/src/lib.rs"]
last_verified: "2026-08-07"
ticket: "spike-kani-bounded-verification-on-one-inexhaustible-encoder"
---

# Kani bounded verification on inexhaustible Tiler encoders

A recorded spike, run by hand. No `make` target reaches this directory.

The question, from `tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md`: can a Kani proof harness prove the injectivity of a Tiler identity encoder whose input domain is too large to enumerate — and before that, does `crates/tiler-ir` compile at all under Kani's bundled rustc?

The full write-up, including the per-Fact audit of the ticket and the evidence-class discussion, is [Kani bounded verification of inexhaustible identity encoders](../../../docs/research/verification/kani-bounded-encoder-verification.md). This file is the invocation record.

## Verdict in one paragraph

**Fact.** `crates/tiler-ir` does **not** compile under Kani 0.67.0's bundled rustc, so the stop condition in the ticket fired.

**Fact.** Kani nonetheless proves the encoders' injectivity over *copies* of them, and for the finite-width encoders it proves it over the entire domain with no residual bound — including all 2^32 ordinals that the exhaustive-finite work could not reach. `push_resources`, whose domain is about 2^161 ordered pairs, discharges in 72 s.

**Fact.** The cost driver is not the input domain, which CBMC handles symbolically for free. It is the `Vec<u8>` output, whose symbolic length makes the comparison's `memcmp` unwind without bound until an explicit unwind bound is supplied — and that bound is *provably* sufficient, not merely asserted, because each encoder has a known maximum output width and CBMC checks it.

**Fact.** The one encoder carrying a `String` is out of reach, and the reason is the `String` rather than the encoder: the same `push_numerical` costs 1.46 s with a concrete key and more than 900 s with an *empty symbolic* one, because `String::from_utf8` over symbolic bytes drags CBMC through the UTF-8 validation automaton.

## Reproducing

Host: Apple M3 Pro, macOS 27.0 (build 26A5388g), `aarch64-apple-darwin`.

Install (this is a host toolchain addition; it was authorized — see the ticket's trigger check log):

```sh
cargo install --locked kani-verifier   # installed kani-verifier v0.67.0
cargo kani setup                       # installed nightly-2025-11-21-aarch64-apple-darwin under ~/.kani
```

The stop-condition probe, from the repository root:

```sh
cargo kani -p tiler-ir --only-codegen
```

The harnesses, from this directory:

```sh
cargo kani --harness push_resources_injective           # one
```

**Do not start with a bare `cargo kani`.** Three of the nine harnesses —
`push_numerical_injective_key_len_1`, `_2`, and `_4` — are expected not to
terminate in any reasonable budget: `_key_len_0` already exceeded a 900 s cap and
each of the others is strictly harder. They are checked in as the record of what
was attempted, not as a suite to run. The ones that resolve, and the one capped
result worth reproducing:

```sh
for h in push_tensor_role_injective \
         push_component_role_injective \
         push_resources_injective \
         push_resources_prefix_free_tail_4 \
         push_numerical_injective_fixed_key; do
    cargo kani --harness "$h"
done
timeout 900 cargo kani --harness push_numerical_injective_key_len_0  # expect no verdict
```

The staleness guard, from this directory:

```sh
./guard.sh
```

## Kani ignores `rust-toolchain.toml`, measured

The ticket asserted this from documentation. It is now measured, and the evidence is the failure diagnostic itself: this repository pins `nightly-2026-07-19`, and `cargo kani -p tiler-ir --only-codegen` run from the repository root reported

```
= note: this compiler was built on 2025-11-20; consider upgrading it if it is out of date
```

A compiler built on 2025-11-20 is `nightly-2025-11-21`, which is Kani's bundle and not the pin. Kani selects its own toolchain regardless of the file.

## Why there are copies here instead of a dependency on `tiler-ir`

`cargo kani -p tiler-ir --only-codegen` fails with **9 errors from three independent causes**:

| cause | sites | detail |
| --- | --- | --- |
| `error[E0635]: unknown feature min_adt_const_params` | 1 | `crates/tiler-ir/src/lib.rs:2`. The feature *name* does not exist at that nightly. |
| `` `[u64; RANK]` is forbidden as the type of a const generic parameter `` | 4 | `semantic/shape_evidence.rs:32`, `shape/evidence.rs:63,65,67`. Downstream of the first: the nightly suggests the older, broader `adt_const_params`. |
| `` error[E0658]: use of unstable library feature `atomic_try_update` `` | 4 | `index/handles.rs:13`, `kernel/handles.rs:24`, `program/handles.rs:24`, `semantic/handles.rs:14`. Stable at the repository's pin, unstable at Kani's. |

So every encoder and every type it ranges over is a **verbatim copy** in `src/lib.rs`, and `guard.sh` is the only thing tying the copies to the sources. Read the module documentation in `src/lib.rs` for what that tie does and does not cover — it is a text tie, and its three named holes matter to any claim built on these proofs.

`guard.sh` was watched failing on four planted drifts before being trusted: a changed tag literal, an added enum variant, a dropped `bytes.push`, and a deleted marker (the population check). Each produced exit 1 naming the divergence.

## Re-probe condition

The spike is unblocked when Kani bundles a nightly that compiles `crates/tiler-ir`. Measured bracket, both reproducible on this host:

- `nightly-2025-11-21` (Kani 0.67.0's bundle, rustc 1.93.0-nightly) — **fails**, 9 errors above.
- `nightly-2026-05-03` (rustc 1.97.0-nightly) — **compiles clean**: `cargo +nightly-2026-05-03 check -p tiler-ir` exits 0.

The exact threshold lies between those two dates. Narrowing it means installing intermediate nightlies, which is a host-environment change and therefore Tom's call, and it would not change the verdict: no released Kani bundles anything in that window. Kani 0.67.0 (2026-01-16) is the newest release as of 2026-08-07, and the release cadence has slowed — 0.65.0 2025-08-07, 0.66.0 2025-11-06, 0.67.0 2026-01-16, then nothing for about seven months.

**Re-probe by running `cargo kani -p tiler-ir --only-codegen` after any new Kani release.** That single command is the whole condition; it needs no interpretation of release notes.
