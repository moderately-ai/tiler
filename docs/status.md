---
schema: "tiler-doc/v1"
id: "tiler.portal.status"
kind: "portal"
title: "Project status"
topics: ["status", "orientation"]
related: ["tiler.questions.open", "tiler.roadmap"]
---

# Project status

Tiler has entered a bounded prototype phase. The semantic, optimizer,
scheduling, numerical, artifact, cache, and runtime boundaries have substantial
accepted decisions and bounded executable evidence. An initial untyped
semantic/reference draft exposed incorrect provisional public boundaries; the
dependency-ordered v2 correction is now implemented through the semantic and
reference crate boundary. Graph ownership, recoverable commitment,
output-reachable compaction, origin-bound output selectors, independent type
authority, open operation registration, generic typed values, and exact
reference-capability dispatch are compile-checked for the bounded profile.
Shape evidence remains on the active dependency path.
Target-neutral compilation, Metal AOT, and device execution remain
unimplemented and no public API is stabilized.

## Authorized prototype

Tom selected and authorized the bounded strict serial `f32` `Sum` Metal value
proof in [ADR 0055](decisions/0055-use-a-serial-sum-for-the-first-metal-value-proof.md).
The prototype must pass through the documented semantic, reference, optimizer,
schedule, structured-kernel, artifact, and guarded-runtime boundaries; a
handwritten standalone Metal kernel is insufficient. ADR 0065 supersedes ADR
0056's four-crate count by extracting `tiler-reference`; ADR 0067 supersedes
the stable Rust 1.89 floor and selects the exact `nightly-2026-07-19` pin plus
one dependent-array `StaticShape<RANK, EXTENTS>` family. The active frontier is
the [nightly shape conformance
harness](../tickets/spike-nightly-arbitrary-rank-shape-evidence.md), checked
shaped-value work, and their integration by the
[`semantic/reference slice`](../tickets/prototype-semantic-reference-slice.md).
That is followed by the dependency-ordered
[`materialized target-neutral baseline`](../tickets/prototype-target-neutral-baseline-slice.md),
[`target-neutral fusion selection`](../tickets/prototype-target-neutral-fusion-slice.md),
[`Metal AOT bundle`](../tickets/prototype-metal-aot-slice.md), and
[`Metal execution proof`](../tickets/prototype-metal-runtime-proof.md) slices.

This chain is an architectural value proof. It does not by itself complete the
inline proc-macro/cache exit for Milestone 0B, Candle integration, or the full
Milestone 2 product profile.

## Evidence boundary

- Executable spikes validate specific representations, protocols, numerical
  cases, and toolchain behavior.
- Apple compatibility is bounded to measured toolchains and hosts; old-OS and
  device-family runtime coverage is not universal.
- Sound region-accuracy analysis is feasible for a narrow trusted-analyzer
  profile; independent certificate checking remains deferred.
- Multi-device, external-storage, and richer transfer work is deliberately
  deferred, not silently ready.

## Live work state

Ticketsplease is authoritative for current workflow state:

```sh
tkt rollup
tkt ready
tkt tracks
tkt reconcile
```

See the [work-tracking guide](work-tracking.md) before claiming work. The
[roadmap](roadmap.md) describes proposed progression, not completed support.
