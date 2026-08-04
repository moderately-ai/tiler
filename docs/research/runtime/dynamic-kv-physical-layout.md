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
same two capacity-sized pool banks, and both copy kernels issue exactly one load
and one store for every live coordinate. Capacity-strided routing nevertheless
requires a larger capacity-derived accessible bounding span than the dense live
payload. It adds that routing requirement and a consequential schema/public
surface without improving measured B1 access time. It is rejected as strictly
more mechanism for no retained benefit.

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

At B1-first (`8192/8320`), the median of five per-round GPU medians is 750.500
us for the selected capacity-sized exact-live row, 750.500 us for its compact
allocation control, 750.500 us capacity-strided head-major, and 779.208 us
sequence-major. At B1-last (`8320/8320`) it is 761.708, 761.708, 761.958, and
791.375 us. The selected pooled exact-live and capacity-strided rows differ by
less than 0.1%; sequence-major is 3.8–3.9% slower. C1 timings sit near the
device timestamp/launch floor and rank nothing. The common output is live-sized,
so this measures input allocation length and address order, not output pooling.

**Measurement.** A compact-allocation exact-live policy requests 1,032,192
bytes across one layer's pinned C1 lifecycle `S=10…18` and 8,724,676,608 bytes
across its 129 B1 extents; allocator-call medians are 72.042 us and 1,675.958
us. The survivor's two capacity-sized banks request 294,912 and 136,314,880
bytes once, with medians 13.250 and 17.333 us. Capacity-strided measures 14.458
and 17.083 us; sequence-major 14.250 and 17.708 us. These stable-pool rows are
indistinguishable at this boundary. Stable reuse is therefore compatible with
exact-live addressing.

**Measurement boundary.** This is one Apple9 GPU/toolchain row, F32, one-layer
scaled allocation populations, and four exact live/capacity cells: C1 `10/18`
and `18/18`, B1 `8192/8320` and `8320/8320`. The access kernel reads every
live coordinate into a common live-sized output; it is not complete attention.
Allocation pages are not
touched. Timings do not transfer to another GPU, dtype, batching scheme,
allocator, or full 28-layer resident population and are never multiplied by 28.

## Consequences and exact byte quantities

The semantic graph remains `[8,C,128] -> [8,S,128]`, `S=C+T`. Structured
kernel and artifact work need the governed semantic live-extent operand only.
The runtime state owns two capacity-sized buffers per logical K/V member, their
active bank, valid extent, generation, device/context, and poison state. It
routes an exact live prefix and publishes the replacement bank atomically.

Three byte quantities must remain separate. There are 56 logical K/V members
across 28 layers. Both head-major copy kernels issue exactly `8*live*128` F32
loads and stores per member, so their model-wide payload transfer is identical:
`56*8*128*4*(C+S)`. It is 2,293,760 bytes at C1 prefill (`C=0,S=10`),
8,028,160 bytes at C1 final (`C=17,S=18`), and 3,816,587,264 bytes at
B1 final (`C=8319,S=8320`).

Routing must instead make every addressed byte accessible. An exact-live dense
resource's accessible span equals its live payload. A capacity-strided
resource's bounding span is `((8-1)*capacity+live)*128*4` bytes per member;
the absent `C=0` input contributes no resource. Summed across the model's
members and old/new resources, the capacity-strided spans are 3,899,392 bytes
at C1 prefill, 8,228,864 bytes at C1 final, and 3,816,787,968 bytes at B1
final. These larger spans are a routing/preflight requirement, not additional
copy-kernel loads or stores.

Finally, both head-major candidates reserve the same two capacity-sized banks:
`2*capacity*56*8*128*4`, or 8,257,536 bytes for C1 and 3,816,816,640 bytes
for B1. Reservation is neither payload transfer nor an accessible span. The
retained arithmetic oracle computes and distinguishes all three quantities;
none is a resident-process measurement.

All four independently wrong address interpretations fail the retained oracle:
the two exact-live allocation policies, capacity-strided head-major, and
sequence-major. The physical-root carrier tickets are obsolete because their
candidate did not survive. The KV artifact/runtime ticket depends directly on
the live-extent carrier and must not add a KV-specific stride schema. No Tom
decision remains on physical layout; Tom still owns acceptance of the
live-extent carrier's tested consequential public/schema spelling when that
implementation reaches review.

## Unsupported cases

Only rank-three F32, batch 1, eight heads, width 128, fixed positive capacity,
out-of-place publication, and one device/context are selected. Ragged or
batched cursors, paging, growing capacity, prefix sharing, external storage,
multiple devices, in-place mutation, and layouts required by another backend
remain separate architecture. The selected representation reserves no generic
stride surface for them.
