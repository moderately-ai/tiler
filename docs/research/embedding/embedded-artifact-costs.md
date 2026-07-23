---
schema: "tiler-doc/v1"
id: "tiler.research.embedding.artifact-costs"
kind: "research"
title: "Direct embedded-artifact costs across Rust crates"
topics: ["embedding", "artifacts", "rustc", "binary-size"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.frontend-integration", "tiler.contract.artifact-abi"]
adopted_by: ["ADR-0004"]
ticket: "embedded-artifact-costs"
---

# Direct embedded-artifact costs across Rust crates

**Status:** bounded host measurement and initial budget; linker behavior is
observed, not guaranteed

## Decision

Emit each manifest or compiled payload as one proc-macro byte-string literal.
Do not emit one integer literal token per byte. Keep invocation-local bundling
as the correctness model, count every emitted copy against the embedding
budget, and treat any compiler or linker folding as an opportunistic size win.

The initial measured envelope is at most 1 MiB of direct bytes per macro
invocation and at most 32 invocations or 3.2 MiB of direct bytes in one consumer
package, whichever is reached first. Crossing either boundary is not known to
be incorrect; it requires an explicit diagnostic/override and a new measurement
case before becoming a supported default. Independent macros cannot enforce a
crate-wide sum, so the integration fixture or CI binary-size job owns that
second gate.

On the comparable macOS release fixture, gate `__TEXT,__const` at total logical
emitted bytes plus 64 KiB. This conservative gate assumes zero deduplication and
allows fixed runtime constants and alignment. Report unique bytes, logical
emitted bytes, literal count, final binary bytes, and constant-section bytes in
size diagnostics. Never budget against unique bytes alone.

These are initial product/CI bounds inside the measured matrix, not Rust or
Metal limits and not portable timing thresholds.

## Reproducible experiment

[`measure.py`](../../../spikes/embedding/measure.py) creates a dependency-free
stable proc macro. The macro deterministically constructs payload bytes and
returns either:

- one `Literal::byte_string` token per artifact; or
- one `Literal::u8_unsuffixed` token per byte, the adverse control.

The consumer reads every byte through `read_volatile`, which retains content
without requiring different artifacts to have different addresses. `same`
expands every invocation in the binary crate. `cross` expands one invocation in
each dependency crate and links them into the same binary. Payload scanning of
the final Mach-O counts exact retained copies; `size -m` records sections.

The historical decision run was:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 spikes/embedding/measure.py \
  --preset decision \
  --output docs/research/embedding/measurements/2026-07-20-macos-arm64
```

Every matrix cell had three independent fresh targets. Cargo commands used
`--offline`; `/usr/bin/time -l` recorded wall time and command-tree resource
usage. The stateful freshness sequence used one target per boundary: fresh,
no-op, unrelated app edit, no-op, one artifact edit, and no-op. Exact expanded
commands and host metadata are in
[`metadata.json`](measurements/2026-07-20-macos-arm64/metadata.json); per-run
metrics are in [`results.json`](measurements/2026-07-20-macos-arm64/results.json)
and [`results.csv`](measurements/2026-07-20-macos-arm64/results.csv). Freshness
metrics are in
[`freshness.json`](measurements/2026-07-20-macos-arm64/freshness.json).

### Retained-evidence boundary

The 2026-07-20 directory retains 81 derived result rows, 12 derived freshness
rows, their CSV projections, and schema-v1 metadata. A strict verifier confirms
that all required metrics have valid shapes, the CSV and JSON views agree, and
the retained files match the digests in
[`integrity.json`](measurements/2026-07-20-macos-arm64/integrity.json).

Contrary to the original harness documentation, the run did **not** retain raw
Cargo stdout/stderr, `size -m` output, or generated source workspaces. Schema v1
also omitted the source revision, harness and input digests, inherited
Cargo/Rust environment, executable identities, and command deadlines. The
quantitative tables below therefore remain bounded historical derived evidence;
they are not a claim that the exact run can be independently reconstructed or
that it was reproduced on the current toolchain.

The repaired schema-v2 harness fails closed on missing or malformed required
metrics, applies both per-command and complete-run hard deadlines, records
source/input/tool/environment provenance, retains raw outputs for new runs, and
requires a fresh output directory. An atomically published `complete.json`
marker identifies every retained evidence file outside the optional
`--keep-work` debugging tree and is the only successful terminal state. Its
smoke and negative paths were verified on 2026-07-21; the historical
decision matrix was deliberately not rerun on a changed toolchain.

## Measurement environment

- Mac16,6, 14 logical CPUs, 36 GiB memory
- macOS 27.0 build 26A5378n, arm64
- `rustc 1.97.0 (2d8144b78 2026-07-07)`, LLVM 22.1.6
- Cargo 1.97.0 (c980f4866 2026-06-30)
- Apple `ld` project 1267; Xcode 26.6 build 17F113
- Python 3.14.6 used only to generate fixtures, launch commands, and summarize
  outputs

The run installed or changed no toolchain component.

## Token representation and single-artifact scaling

Values are medians of three fresh release builds. RSS is the maximum reported
by `/usr/bin/time`; binary and constant-section sizes were stable across the
three release builds.

| Payload | Representation | Literal tokens | Wall s | Peak RSS MiB | Final KiB | `__TEXT,__const` KiB |
|---:|---|---:|---:|---:|---:|---:|
| 10 KiB | byte string | 1 | 0.24 | 141 | 437 | 21 |
| 10 KiB | per byte | 10,240 | 0.26 | 140 | 437 | 21 |
| 100 KiB | byte string | 1 | 0.26 | 140 | 518 | 111 |
| 100 KiB | per byte | 102,400 | 0.35 | 141 | 518 | 111 |
| 1 MiB | byte string | 1 | 0.48 | 135 | 1,453 | 1,035 |
| 1 MiB | per byte | 1,048,576 | 1.69 | 468 | 1,453 | 1,035 |

At 1 MiB the per-byte form took 3.5 times the wall time and 3.5 times the peak
RSS of the byte-string form. Both linked to the same release bytes, so the
penalty was entirely on compilation. The 10 KiB and 100 KiB cases are dominated
by fixed Cargo, proc-macro-crate, rustc, and linker costs; they do not justify a
per-byte representation.

A 1 MiB byte string in the dev profile measured 0.31 s, 131 MiB peak RSS, a
1,500 KiB binary, and a 13.6 MiB target tree. The corresponding release target
tree was 4.2 MiB. The derived fixture reports differing debug output hashes and
stable release hashes. Because neither the binaries, raw inspection output, nor
generated workspaces were retained, attributing that difference to absolute
paths in debug metadata is an unverified historical explanation, not evidence.

## Counts, identity, and observed folding

All rows below embed 100 KiB per invocation. `Occurrences` is the number of
exact copies found for an identical payload; for unique payloads, every distinct
payload occurred once.

| Case | Logical bytes | Wall s | Peak RSS MiB | Final KiB | Const KiB | Occurrences |
|---|---:|---:|---:|---:|---:|---:|
| same, release, 8 identical | 800 KiB | 0.28 | 139 | 1,228 | 811 | 8 |
| same, release, 8 unique | 800 KiB | 0.27 | 138 | 1,228 | 811 | 1 each |
| same, release, 32 identical | 3.125 MiB | 0.34 | 137 | 1,729 | 1,311 | 13 |
| same, release, 32 unique | 3.125 MiB | 0.33 | 167 | 3,648 | 3,211 | 1 each |
| same, dev, 8 identical | 800 KiB | 0.28 | 133 | 1,082 | 612 | 6 |
| cross, dev, 8 identical | 800 KiB | 0.31 | 135 | 1,280 | 812 | 8 |

Default release codegen (`codegen-units = 16`, no LTO) retained all eight
identical copies. At 32 same-crate invocations it retained 13, consistent with
folding inside some codegen partitions; that explanation is an inference, not
a contract. The dev same-crate build retained six while the cross-crate build
retained eight. Identity did not affect the eight-copy release binary size.
Both same-crate eight-copy release configurations, identical and unique, ran
once in the count submatrix and again in the boundary submatrix, so the wall
time and RSS of those two rows are medians of six fresh builds; every other row
in this table is a median of three. The boundary table below reports the
identical configuration's three boundary-submatrix builds alone, which is why
its wall time and RSS differ from the corresponding row here.

The result needed by the architecture is negative: neither identical literals,
same-crate placement, nor a release profile guarantees one stored copy.

## Crate boundaries, codegen units, and LTO

These release rows embed eight identical 100 KiB payloads.

| Boundary / settings | Wall s | Peak RSS MiB | Final KiB | Const KiB | Occurrences | rlibs KiB | Target MiB |
|---|---:|---:|---:|---:|---:|---:|---:|
| same, CGU 16, LTO off | 0.26 | 141 | 1,228 | 811 | 8 | 0 | 3.7 |
| cross, CGU 16, LTO off | 0.29 | 141 | 1,228 | 811 | 8 | 1,671 | 5.4 |
| same, CGU 1, LTO off | 0.34 | 117 | 518 | 111 | 1 | 0 | 2.3 |
| cross, CGU 1, LTO off | 0.34 | 117 | 1,228 | 811 | 8 | 1,670 | 5.4 |
| same, CGU 16, thin LTO | 1.67 | 207 | 1,224 | 811 | 8 | 0 | 3.7 |
| cross, CGU 16, thin LTO | 1.71 | 200 | 1,224 | 811 | 8 | 1,701 | 5.4 |
| same, CGU 1, fat LTO | 1.73 | 199 | 462 | 112 | 1 | 0 | 2.2 |
| cross, CGU 1, fat LTO | 1.83 | 197 | 462 | 112 | 1 | 1,701 | 3.9 |

One same-crate codegen unit folded the bytes without LTO, but did not fold
across dependency rlibs. Fat LTO folded both. Thin LTO folded neither in this
fixture and cost roughly 5--6 times the no-LTO wall time. These outcomes can
change with rustc, LLVM, linker, flags, symbol reachability, and source shape;
Tiler must not enable LTO or reduce codegen units solely to obtain deduplication.

Cross-crate placement did not change the default release final binary, but it
added about 1.67 MiB of rlibs and 1.7 MiB to the target tree for 800 KiB of
payload. Dev target trees were 11.1 MiB same-crate and 14.5 MiB cross-crate for
the identical case. Crate separation is therefore a freshness and ownership
choice with intermediate-storage cost, not a free deduplication mechanism.

## Freshness and incremental behavior

The dev fixture embedded eight identical 100 KiB payloads.

| Boundary / phase | Historically reported Cargo work | Wall s | Peak RSS MiB | Target MiB |
|---|---|---:|---:|---:|
| same / fresh | macro + app | 0.27 | 134 | 11.1 |
| same / no-op | none | 0.01 | 21 | 11.1 |
| same / unrelated app edit | app, including its eight expansions | 0.11 | 81 | 13.0 |
| same / one artifact edit | app, including its eight expansions | 0.11 | 99 | 14.2 |
| cross / fresh | macro + eight blobs + app | 0.31 | 134 | 14.5 |
| cross / no-op | none | 0.02 | 22 | 14.5 |
| cross / unrelated app edit | app only; no literal expansion | 0.09 | 83 | 15.1 |
| cross / one artifact edit | one blob + app | 0.12 | 91 | 15.5 |

The retained freshness rows contain timings, RSS, target sizes, and a derived
work label, but no Cargo stdout/stderr, package freshness stream, proc-macro
trace, or generated workspace. They support the quantitative rows only; they
do not independently establish which packages rebuilt or how many macro
expansions occurred. Those work descriptions are historical reports awaiting a
fresh schema-v2 decision run. The observed target-size growth likewise remains
a bounded derived measurement, not a causal attribution.

This does not change the separate proc-macro environment result: deleting an
external artifact cache or changing an untracked external toolchain does not by
itself make Cargo rerun an otherwise fresh expansion.

## Diagnostics and gates

The frontend or AOT layer should report, per invocation:

- manifest bytes, payload bytes per artifact family, and total logical emitted
  bytes;
- byte-string literal count and unique content digest count;
- whether the 1 MiB default invocation budget was crossed;
- an explicit opt-in name when a larger direct payload is accepted.

The integration size test should aggregate all invocations because a proc macro
cannot discover a reliable crate-wide total. It should fail if the measured
release `__TEXT,__const` exceeds logical emitted bytes plus 64 KiB, or if the
consumer crosses 32 invocations / 3.2 MiB without a reviewed expanded matrix.
It should print both logical and actual sizes on failure. A smaller-than-logical
section is an observed optimization, never a new budget baseline.

## Limitations

- The retained 2026-07-20 fixture is derived-only evidence: its raw command
  outputs and generated inputs are unavailable, and exact regeneration was not
  attempted on the changed toolchain.
- This is one prerelease macOS host, one stable Rust/LLVM pair, Apple Mach-O,
  and one linker. ELF, COFF, other Apple releases, and future toolchains remain
  unmeasured.
- `/usr/bin/time -l` measures the Cargo command tree, not rustc phases in
  isolation. Three medians reduce noise but do not make wall or RSS portable.
- The deterministic payload is high-entropy data. Real manifests and metallibs
  may differ in alignment, section placement, or compiler treatment.
- The text proxy in the JSON estimates a textual spelling only. Proc macros
  return in-memory tokens; no claim is made that the proxy equals rustc's token
  storage.
- Exact payload scanning distinguishes retained full copies, not suffix/prefix
  pooling, compression, relocation overhead, or every possible constant split.
- The fixture keeps content live through volatile reads, but a different API,
  visibility, address observation, or optimization context may link differently.
- Timings are compilation-only. Runtime Metal library creation and pipeline
  costs belong to the runtime performance matrix.

## Traceability

The bounded measurements inform the [frontend integration](../../integration/frontends.md)
and ADR 0004. The [embedding spike](../../../spikes/embedding/README.md) owns
reproduction. Thresholds are initial diagnostics, not universal linker guarantees.
