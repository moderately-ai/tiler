---
schema: "tiler-doc/v1"
id: "tiler.research.runtime.dynamic-kv-physical-layout"
kind: "research"
title: "Dynamic KV physical-layout authority"
topics: ["runtime", "kv-cache", "layout", "abi", "artifacts", "metal", "identity", "routing"]
catalog_group: "runtime-integration-placement"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "bounded-measurement", "executable-model"]
informs: ["tiler.contract.architecture", "tiler.contract.artifact-abi", "tiler.contract.candle-integration"]
depends_on: ["tiler.research.runtime.autoregressive-state-and-kv-cache", "tiler.research.runtime.execution-contract"]
ticket: "establish-a-dynamic-kv-physical-layout-authority"
---

# Dynamic KV physical-layout authority

**Status:** completed source research and bounded measurement derive one
implementation direction. This record accepts no public or schema boundary.
The surviving representation needs the already-filed live semantic-extent
transport, but no new physical-layout root or artifact schema.

## Inspected boundary and the actual missing operand

**Fact — exact source population at
`b4e3478d42ce21ed68e23f772b643c6370d36498`.**
`ArtifactProgramBuilder::push_stage` derives binding offset/count expressions
from a verified stage. The program model, codec, validator, and decoded view
retain those expressions. `place_bindings` evaluates them into
`RoutedBinding::{accessible_offset, accessible_bytes}`. `RuntimeAdapter` checks
the ranges before `RoutingCommit`. `BufferParameter` and admitted launch
builtins are the complete structured-kernel parameter population, and the Metal
emitter emits exactly that population. No payload-consumable scalar extent or
physical stride exists today.

**Fact.** `AbiRoot::InputExtent` already governs artifact-side range, guard,
and launch evaluation. It does not reach kernel-body address or loop arithmetic.
[`admit-live-extent-operands-to-payload-indexing`](../../../tickets/admit-live-extent-operands-to-payload-indexing.md)
owns that gap. The value comes from the existing semantic interface root and is
excluded from artifact, payload, library, and pipeline identity.

**Inference.** The earlier capacity-strided proposal solved a problem it had
created. A physical allocation may be capacity-sized while the logical K/V
payload inside it is packed densely at the current live extent. Allocation
length is a storage-pool property; it need not be the payload's head stride or
the routed accessible byte count.

## Candidates and exact negative oracles

All candidates retain one K and one V resource per layer, batch 1, eight heads,
width 128, one artifact, and one prepared pipeline.

### Exact-live head-major in capacity-sized pooled buffers — survivor

**Proposal.** Old state is packed `[8,C,128]` with head stride `C*128`; the
replacement is packed `[8,S,128]` with head stride `S*128`. Two physical buffer
banks per member are allocated once at `capacity*8*128*4` bytes and alternate
only after exact terminal success. A route exposes only the current live dense
span. The concatenate/copy payload consumes the governed semantic `C` and `S`
operands and never consumes capacity.

**Correctness.** At `capacity=18, C=14, S=15`, old `(1,0,0)` is byte 7,168 and
replacement `(1,0,0)` is byte 7,680. Substituting the capacity stride yields
byte 9,216 and must fail the coordinate oracle. Old and replacement are
disjoint buffers, so their different packings cannot alias; both remain alive
through final device use, and the new bank plus cursor publish atomically.
Routed accessible spans are exactly 57,344 and 61,440 bytes even though each
pool buffer is 73,728 bytes. `S <= capacity`, the additive `S=C+T` relation,
buffer length, device/context, generation, and poison status are preflight
checks; none is inferred from an address.

**Identity and maintainability.** The semantic extents already belong to the
program interface. The live-extent carrier makes them consumable without adding
a second physical fact, storage descriptor grammar, artifact row, or schema
step. Allocation policy stays runtime-owned. Arbitrary strides, negative or
overlapping views, permutations, ragged batches, and caller scalars remain
unsupported rather than being accidentally admitted for one KV case.

### Capacity-strided head-major — correct, then dominated

This representation uses `[8,capacity,128]` and a physical head stride of
`capacity*128`. Its exact oracle is `(1,0,0) -> 9,216` bytes at capacity 18;
using 7,168 or 7,680 addresses the wrong head while remaining in bounds. It
therefore needs a separately typed layout root carried through kernel, Metal,
artifact schema, identity, runtime preflight, and backend binding.

It has no allocation-reuse advantage over the survivor: the survivor uses the
same two capacity-sized pool banks. It touches capacity-derived reached spans
instead of compact live spans and adds a consequential schema/public surface
without improving measured B1 access time. It is rejected as strictly more
mechanism for no retained benefit.

### Capacity-sized sequence-major — correct, then dominated

This representation addresses
`sequence*8*128 + head*128 + component`. Its exact oracle is
`(head=1, sequence=0, component=0) -> 512` bytes; presenting head-major storage
to that formula first corrupts canonical output element 128 in the retained
fixture. It requires no dynamic stride and shares the survivor's stable pool.

It changes every consumer's physical order and is about 3.9% slower than both
head-major candidates at the two measured B1 copy cells. That one-device result
is not a universal GPU claim. It is enough for this elimination because the
candidate has no offsetting correctness, allocation, resource-count, identity,
or support advantage. The conventional head-major survivor preserves the
existing consumer order and is no slower in the bounded measurement.

### Per-head resources and specialization — rejected structurally

Eight per-head resources turn 56 logical K/V members into 448 retained
resources and each K/V transport into eight. Their oracle selects resource 1,
byte 0 for `(1,0,0)`; selecting resource 0 fails. They offer no correctness or
pooling property the survivor lacks and sharply reduce Metal binding headroom.

Per-`C` or per-`S` specialization creates a pipeline per decode extent.
Capacity specialization makes runtime allocation policy part of compiled
identity. A two-state oracle at capacities 18 and 8,320 requires identical
artifact, payload, and pipeline-cache keys. These families violate the stated
one-artifact/one-pipeline outcome and are not cost candidates.

## Bounded Metal evidence

**Measurement.** The retained
[dynamic KV layout spike](../../../spikes/runtime/dynamic-kv-layout/README.md)
ran on Apple M4 Max (`Mac16,6`), Apple9, macOS 27.0 build `26A5388g`, Xcode
26.6 build `17F113`, SDK 26.5, and `metalfe-32023.883`. Five rounds rotate
candidate order; each round records seven dispatches after three warmups.

At B1-first (`8192/8320`), the median of five per-round GPU medians is 750.667
us exact-live head-major, 749.750 us capacity-strided head-major, and 779.708 us
sequence-major. At B1-last (`8320/8320`) it is 761.875, 761.500, and 791.250
us. The head-major candidates differ by less than 0.3%; sequence-major is about
3.9% slower. C1 timings sit near the device timestamp/launch floor and rank
nothing.

**Measurement.** A compact-allocation exact-live policy requests 1,318,912
bytes across one layer's C1 decode lifecycle and 8,724,676,608 bytes across its
129 B1 extents; allocator-call medians are 85.333 us and 1,609.042 us. The
survivor's two capacity-sized banks request 294,912 and 136,314,880 bytes once,
with medians 10.000 and 12.167 us, indistinguishable here from the two other
stable-pool candidates. Stable reuse is therefore compatible with exact-live
addressing.

**Measurement boundary.** This is one Apple9 GPU/toolchain row, F32, one-layer
scaled allocation populations, and four exact live/capacity cells: C1 `5/18`
and `15/18`, B1 `8192/8320` and `8320/8320`. The access kernel copies every
live coordinate; it is not complete attention. Allocation pages are not
touched. Timings do not transfer to another GPU, dtype, batching scheme,
allocator, or full 28-layer resident population and are never multiplied by 28.

## Consequences and exact traffic

The semantic graph remains `[8,C,128] -> [8,S,128]`, `S=C+T`. Structured
kernel and artifact work need the governed semantic live-extent operand only.
The runtime state owns two capacity-sized buffers per logical K/V member, their
active bank, valid extent, generation, device/context, and poison state. It
routes an exact live prefix and publishes the replacement bank atomically.

For all 28 layers, exact-live out-of-place bytes touched at the first and last
C1 cells range from 2,293,760 to 8,028,160 rather than the
capacity-strided 8,257,536-byte transaction. At B1 final they are
3,816,587,264 rather than 3,816,816,640. Physical pool reservation is the same
for the two head-major candidates; these figures are reached/touched spans, not
resident-process measurements.

The physical-root carrier tickets are obsolete because their candidate did not
survive. The KV artifact/runtime ticket depends directly on the live-extent
carrier and must not add a KV-specific stride schema. No Tom decision remains
on physical layout; Tom still owns acceptance of the live-extent carrier's
tested consequential public/schema spelling when that implementation reaches
review.

## Unsupported cases

Only rank-three F32, batch 1, eight heads, width 128, fixed positive capacity,
out-of-place publication, and one device/context are selected. Ragged or
batched cursors, paging, growing capacity, prefix sharing, external storage,
multiple devices, in-place mutation, and layouts required by another backend
remain separate architecture. The selected representation reserves no generic
stride surface for them.
