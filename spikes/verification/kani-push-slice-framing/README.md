---
schema: "tiler-doc/v1"
id: "tiler.spike.verification.kani-push-slice-framing"
kind: "experiment"
title: "Kani bounded verification of push_slice framing"
topics: ["verification", "kani", "identity", "injectivity", "length-framing"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.verification.kani-push-slice-framing"]
entrypoints: ["spikes/verification/kani-push-slice-framing/guard.sh", "spikes/verification/kani-push-slice-framing/src/lib.rs"]
last_verified: "2026-08-10"
ticket: "spike-kani-push-slice-framing-over-a-symbolic-byte-run"
---

# Kani bounded verification of `push_slice` framing

A recorded spike, run by hand from this directory. No `make` target or workspace
manifest reaches it. The full interpretation is [Kani bounded verification of
`push_slice` framing](../../../docs/research/verification/kani-push-slice-framing.md).

## Verdict

**Measurement.** Kani proves both injectivity and strict-prefix freedom for all
ordered pairs of byte runs of length 0 through 4, with every active byte
symbolic. Both harnesses use unwind 13; CBMC reports the `memcmp` unwinding
assertion `SUCCESS`, so no comparison path inside this bounded domain is cut
off.

**Fact.** The proof subject is a copy. Kani 0.67.0 still cannot compile the live
`tiler-ir` crate, and this experiment did not change the host or toolchain to
work around that refusal. `guard.sh` compares exactly the copied `push_len` and
`push_slice` against `crates/tiler-ir/src/identity.rs`. Its two-item population
is independent of the predecessor spike's currently stale
`ResourceRequirements` and `push_resources` copies.

**Fact.** The four-byte model bound belongs to the proof, not the construction.
An eight-byte fixed-width length followed by that many payload bytes is
prefix-free for every representable slice length. Kani quantifies the smaller
domain of 4,311,810,305 semantic byte runs here; it does not prove the property
for a fifth byte or longer input.

## Exact environment

- Kani `cargo-kani 0.67.0`.
- CBMC `6.8.0 (cbmc-6.8.0)`.
- Bundled Cargo `1.93.0-nightly (5c0343317 2025-11-18)`.
- Bundled rustc `1.93.0-nightly (53732d5e0 2025-11-20)`, host
  `aarch64-apple-darwin`, LLVM 21.1.5.
- Apple M4 Max, macOS 27.0 (26A5388g), arm64.

This host row differs from the predecessor's M3 Pro row. The host was not idle, and timings below describe these repetitions only; they are not a portable performance claim.

## Reproducing

The Kani installation was authorized and installed for the predecessor spike.
Do not install or update anything for this reproduction.

From this directory:

```sh
./guard.sh
cargo kani --harness push_slice_injective_len_4
cargo kani --harness push_slice_prefix_free_len_4
```

Repeated runs were observed while the coordination host was active. The table reports ranges rather than selecting the fastest reading; CBMC time has three observations per harness, while `/usr/bin/time -p` wall time has two injectivity observations and three prefix-free observations. The final repetition is parenthesized.

| harness | semantic domain | unwind | checks | CBMC across 3 runs | measured wall | unwind assertion |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `push_slice_injective_len_4` | all ordered pairs, each run 0–4 bytes | 13 | 381, 0 failed, 6 unreachable | 2.5190804–5.717403 s (5.717403 s) | 3.51–8.58 s (8.58 s) | `memcmp.unwind.0: SUCCESS` |
| `push_slice_prefix_free_len_4` | all ordered pairs, each run 0–4 bytes | 13 | 375, 0 failed, 6 unreachable | 2.3663561–8.462593 s (8.462593 s) | 3.36–10.49 s (10.49 s) | `memcmp.unwind.0: SUCCESS` |

Kani reports three unsupported constructs during codegen (`caller_location` once,
foreign functions twice) and warns that a reachable one would fail verification.
Both harnesses nevertheless verify successfully; each summary reports six
unreachable checks.

## What the two checks mean

The injectivity harness says equal encodings force equal lengths and equal bytes
at every active position. The symbolic arrays also contain bytes after their
symbolic lengths; those bytes are storage padding, not part of either slice, and
the conclusion intentionally ignores them.

The prefix-free harness says neither complete encoding is a strict prefix of the
other. That is the compositional property a caller needs before writing another
field: one framed byte run cannot consume bytes belonging to the following
field.

## Subject perturbations

Both checks were made to fail by changing the copied subject, never an
assertion, and then restored.

1. Deleting `bytes.extend_from_slice(value)` from the copied `push_slice` made
   `guard.sh` report `DRIFT: push_slice` and `1 of 2 framing copies have
   drifted.` The injectivity harness reported `1 of 380 failed` and
   `Failed Checks: "equal framing must carry equal active bytes"`.
2. Restoring the payload and deleting `push_len(bytes, value.len())` made the
   guard report the same copied-item drift. The prefix-free harness reported
   `2 of 373 failed` and twice named `Failed Checks: "one framed byte run is a
   strict prefix of another"`.

The final checked-in subject is restored; `./guard.sh` reports `2 framing copies
match live source.`

## Provenance and scope limits

The guard compares function-token content after dropping comments, attributes,
visibility, and formatting. It catches payload or prefix writes being removed,
reordered, or changed. It does not compile the live crate, tie callers, prove
that every string encoder still calls `push_slice`, or run automatically.

The predecessor's native census names nine string-encoder **categories**, not
nine concrete function instances or fields. This result supplies their shared
framing lemma; it does not separately prove the non-string tail of every
category. The predecessor proved only `push_numerical`'s tail with the key held
fixed. Evidence-taxonomy classification remains Tom's decision.
