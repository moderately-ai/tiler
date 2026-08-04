---
schema: "tiler-doc/v1"
id: "tiler.spike.runtime.dynamic-kv-layout"
kind: "experiment"
title: "Dynamic KV physical-layout comparison"
topics: ["runtime", "kv-cache", "layout", "metal", "allocation"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.runtime.dynamic-kv-physical-layout"]
entrypoints: ["spikes/runtime/dynamic-kv-layout/run.sh", "spikes/runtime/dynamic-kv-layout/host.m", "spikes/runtime/dynamic-kv-layout/kernels.metal", "spikes/runtime/dynamic-kv-layout/check_arithmetic.py"]
last_verified: "2026-08-04"
ticket: "establish-a-dynamic-kv-physical-layout-authority"
---

# Dynamic KV physical-layout comparison

## Question and candidates

For the pinned batch-1 KV member `[8, live, 128]`, does exact-live
head-major, capacity-strided head-major, or capacity-sized sequence-major
storage dominate once address-walk cost and allocation reuse are measured?
Four rows separate three representations from one allocation control:
`exact-head-compact`, `exact-head-pooled`, `capacity-head`, and
`sequence-major`. The selected exact-live candidate packs logical rows at
`live * 128` inside capacity-sized physical buffers; the compact row isolates
what changes if those buffers are instead sized to the live payload. All rows
use one K and one V resource per layer and one compiled kernel across extents.

The access kernel reads every live F32 coordinate into one common live-sized
dense output. Each row therefore performs the same semantic work and is checked
against the same coordinate-valued CPU oracle; the timing isolates input
allocation length and address order rather than output allocation policy. Five
rounds rotate row order; each round has three warmups followed by seven timed
dispatches. The allocator leg measures both an individual K/V pair and a
complete one-layer decode lifecycle. It does not touch the allocated pages, so
it measures Metal allocator/pool call cost and requested virtual bytes, not
resident memory or copy cost.

## Reproduce

From the repository root, using the repository's governed Xcode installation:

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  spikes/runtime/dynamic-kv-layout/run.sh /tmp/tiler-dynamic-kv-layout
```

The harness offline-compiles under `metal4.0` for macOS 26 with safe math,
precise F32 functions, and contraction disabled. It creates and removes its own
temporary build directory. No repository gate reaches this spike.

The separate exact arithmetic oracle is reproducible without the GPU:

```sh
python3 spikes/runtime/dynamic-kv-layout/check_arithmetic.py
```

## Retained result

[`results/2026-08-04-apple9-m4max-macos27-xcode26.6-metal32023.883/`](results/2026-08-04-apple9-m4max-macos27-xcode26.6-metal32023.883/)
records the raw round medians, allocation measurements, environment, producer
digests, exact model-wide arithmetic, and fail-capable address and arithmetic
oracle perturbations. The host is an
Apple M4 Max (`Mac16,6`), macOS 27.0 build `26A5388g`, Xcode 26.6 build
`17F113`, SDK 26.5, and `metalfe-32023.883`.

At B1-first (`live=8192`, `capacity=8320`), the median of the five per-round
GPU medians is 750.500 us for both compact and pooled exact-live head-major,
750.500 us capacity-strided head-major, and 779.208 us sequence-major. At
B1-last (`8320/8320`) the same statistic is 761.708, 761.708, 761.958, and
791.375 us. The selected pooled exact-live row and capacity-strided head-major
differ by less than 0.1%; sequence-major is 3.8–3.9% slower in this copy walk.
C1 dispatches fall near a timestamp/launch floor and are not used to rank
candidates.

For one layer's whole lifecycle, compact exact-live allocation requests
1,032,192 bytes across pinned C1 `S=10…18` and 8,724,676,608 bytes across the
129 B1 extents; its median allocator time is 72.042 us and 1,675.958 us. Two
preallocated capacity-sized banks request 294,912 and 136,314,880 bytes once
and take 13.250 us and 17.333 us under the exact-live address recipe. The
capacity-strided row is 14.458 and 17.083 us and the sequence-major row 14.250
and 17.708 us. The stable policies are indistinguishable at this boundary;
stable pool reuse therefore does not require a hidden capacity stride.

## Checks that can say no

Each row is rerun with an independently wrong address interpretation. Compact
exact-live uses the smaller neighbouring head stride, pooled exact-live uses the
capacity stride, capacity-strided uses the live stride, and sequence-major
receives head-major storage. All corruptions stay in bounds. The oracle rejects
the first three at canonical output element 1,280 and sequence-major at 128. A
negative command returning success makes `run.sh` fail. Exact terminal
command-buffer success is required before either timed readback or the oracle.
The arithmetic oracle separately rejects conflating the two-bank pool
reservation with payload transfer bytes.

## Measurement boundary

This is one Apple9 GPU, one OS/toolchain row, F32, batch 1, eight heads, width
128, and four exact workload cells: C1 `10/18` and `18/18`, B1 `8192/8320` and
`8320/8320`. The access kernel is a full live-coordinate copy, not attention or
the complete concatenate program. The allocation lifecycle is scaled to one
layer so it never creates the model's multi-gigabyte population; requested byte
counts are exact for that layer but timings must not be multiplied by 28.
Buffers are not page-touched in the allocation leg. Results establish neither
another Apple family nor a universal cache-locality or allocator guarantee.
