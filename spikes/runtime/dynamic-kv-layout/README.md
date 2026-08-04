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
entrypoints: ["spikes/runtime/dynamic-kv-layout/run.sh", "spikes/runtime/dynamic-kv-layout/host.m", "spikes/runtime/dynamic-kv-layout/kernels.metal"]
last_verified: "2026-08-04"
ticket: "establish-a-dynamic-kv-physical-layout-authority"
---

# Dynamic KV physical-layout comparison

## Question and candidates

For the pinned batch-1 KV member `[8, live, 128]`, does exact-live
head-major, capacity-strided head-major, or capacity-sized sequence-major
storage dominate once address-walk cost and allocation reuse are measured?
All three candidates use one K and one V resource per layer and one compiled
kernel across every live extent. The exact-live candidate's logical rows are
packed at `live * 128`, but its two alternating physical buffers may each be
allocated at capacity size; allocation length and the logical address recipe
are deliberately measured as separate choices.

The access kernel copies every live F32 coordinate into one canonical dense
output. Each candidate therefore performs the same semantic work and is checked
against the same coordinate-valued CPU oracle. Five rounds rotate candidate
order; each round has three warmups followed by seven timed dispatches. The
allocator leg measures both an individual K/V pair and a complete one-layer
decode lifecycle. It does not touch the allocated pages, so it measures Metal
allocator/pool call cost and requested virtual bytes, not resident memory or
copy cost.

## Reproduce

From the repository root, using the repository's governed Xcode installation:

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  spikes/runtime/dynamic-kv-layout/run.sh /tmp/tiler-dynamic-kv-layout
```

The harness offline-compiles under `metal4.0` for macOS 26 with safe math,
precise F32 functions, and contraction disabled. It creates and removes its own
temporary build directory. No repository gate reaches this spike.

## Retained result

[`results/2026-08-04-apple9-m4max-macos27-xcode26.6-metal32023.883/`](results/2026-08-04-apple9-m4max-macos27-xcode26.6-metal32023.883/)
records the raw round medians, allocation measurements, environment, producer
digests, and three fail-capable address-oracle perturbations. The host is an
Apple M4 Max (`Mac16,6`), macOS 27.0 build `26A5388g`, Xcode 26.6 build
`17F113`, SDK 26.5, and `metalfe-32023.883`.

At B1-first (`live=8192`, `capacity=8320`), the median of the five per-round
GPU medians is 750.667 us exact-live head-major, 749.750 us capacity-strided
head-major, and 779.708 us sequence-major. At B1-last (`8320/8320`) the same
statistic is 761.875, 761.500, and 791.250 us. Exact-live and capacity-strided
head-major differ by less than 0.3%; sequence-major is about 3.9% slower in
this copy walk. C1 dispatches fall near a timestamp/launch floor and are not
used to rank candidates.

For one layer's whole lifecycle, compact exact-live allocation requests
1,318,912 bytes across C1 and 8,724,676,608 bytes across the 129 B1 extents;
its median allocator time is 85.333 us and 1,609.042 us. Two preallocated
capacity-sized banks request 294,912 and 136,314,880 bytes once and take about
10.000 us and 12.167 us under the exact-live address recipe. Those rows are
indistinguishable from the capacity-strided and sequence-major stable-pool rows
at this precision. Stable pool reuse therefore does not require a hidden
capacity stride.

## Checks that can say no

Each candidate is rerun with an independently wrong address interpretation.
The exact-live and capacity-strided cases swap their head strides while staying
in bounds; the sequence-major payload receives head-major storage. The oracle
rejects all three, with first mismatches at canonical output element 640, 640,
and 128 respectively. A negative command returning success makes `run.sh`
fail. Exact terminal command-buffer success is required before either timed
readback or the oracle.

## Measurement boundary

This is one Apple9 GPU, one OS/toolchain row, F32, batch 1, eight heads, width
128, and four exact workload cells: C1 `5/18` and `15/18`, B1 `8192/8320` and
`8320/8320`. The access kernel is a full live-coordinate copy, not attention or
the complete concatenate program. The allocation lifecycle is scaled to one
layer so it never creates the model's multi-gigabyte population; requested byte
counts are exact for that layer but timings must not be multiplied by 28.
Buffers are not page-touched in the allocation leg. Results establish neither
another Apple family nor a universal cache-locality or allocator guarantee.
