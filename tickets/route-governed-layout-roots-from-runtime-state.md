---
id: route-governed-layout-roots-from-runtime-state
title: Route governed layout roots from runtime state
status: todo
priority: p1
dependencies: [define-the-runtime-kv-state-boundary, draft-governed-affine-layout-roots-through-kernel-and-artifact]
related: [bind-the-kv-cache-through-the-artifact-and-runtime-interface]
scopes: [implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, layout, abi, kv-cache, routing, public-boundary]
---
## User-visible outcome

Runtime preflight derives every payload layout operand from the owned KV state's observed storage and freezes the exact bytes into the committed dispatch, so a lying or stale layout refuses before program work and a backend cannot silently bind a different stride.

## Authority and boundary

Consume [the dynamic KV physical-layout record](../docs/research/runtime/dynamic-kv-physical-layout.md), the accepted one-way commit, and the accepted exact-final-use retention rules. The exact runtime and adapter methods are a consequential public boundary and remain Tom's to accept after a tested draft. A caller-supplied stride or capacity list is forbidden: the state boundary and decoded artifact population are the two authorities being joined.

## Required work

- Extend the decoded route/preflight path with the complete root population the artifact declares. Derive each value from the matching owned state member's physical descriptor and live storage observation; reject missing, extra, reordered, wrong-member, wrong-generation, wrong device/context, wrong type, arithmetic overflow, and unavailable backend mechanism.
- Validate the initial head-major KV root as `head_stride = capacity × 128` elements, prove the live extent does not exceed capacity and heads cannot overlap, prove every reached address lies in the observed allocation and routed accessible span, and freeze canonical parameter bytes before `RoutingCommit`. `RoutedBinding` offset/count remain range facts rather than a second layout spelling, and the span must use the governed strided maximum rather than dense logical payload bytes.
- Carry the frozen parameter block only through the consuming committed authority. It must not be clonable or forgeable independently, and the backend binds exactly its governed transport during committed allocation/dispatch.
- A mismatch before commit is a typed adapter refusal with fallback still permitted. A backend binding mismatch or post-commit storage-observation contradiction is a terminal adapter failure and poisons the complete KV state transaction; it never retries another route.
- Keep old and replacement resource populations disjoint and retained through exact final device use. Publication replaces all survivor-defined resources and the one model cursor atomically only after observed terminal success.
- Prove that runtime values do not enter artifact, library, or pipeline cache identity and that one prepared pipeline serves capacities 18 and 8,320 and every C1 `C`/`S`.

## Required evidence

- With `capacity = 18, C = 14, S = 15`, both old and replacement bindings freeze `head_stride = 2,304` elements and address `(1,0,0)` at byte 9,216; their exact reached spans are 71,680 and 72,192 bytes inside the 73,728-byte allocations.
- Missing, swapped, caller-restated, stale-generation, wrong-scope, overflowed, range-exceeding, and backend-misbound roots fail at their specified stage. Remove each check deliberately and watch its negative fail.
- Two state capacities and all nine C1 executions retain one artifact identity, one payload subject, and one pipeline-cache key.
- Targeted artifact/runtime/build tests, `tkt lint`, `git diff --check`, guard, and the full gate pass.

## Unsupported cases

Only the accepted carrier's bounded positive-stride vocabulary and the initial batch-1 rank-three F32 KV descriptor. Ragged batches, paging, growing capacity, external storage, multiple devices, overlap, and in-place publication refuse or remain outside this ticket.

## Closes when

Tom has accepted the tested consequential runtime surface; preflight derives and freezes the complete population without caller restatement; all negatives fail at the correct boundary; one pipeline covers the two named capacities and all C1 steps; and the KV artifact/runtime binding ticket depends on this landed authority.
