---
id: add-subgroup-memory-scope-when-collectives-land
title: Add a subgroup memory scope when subgroup collectives enter the profile
status: deferred
priority: p2
dependencies: []
related: [prototype-structured-kir-slice, prototype-metal-kir-lowering]
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, barriers, deferred]
---
A second gap in `tiler_ir::kernel`, also found by the first real backend.

`BarrierSpec` names execution scope and memory scope separately, which ADR 0048
requires precisely because a target builtin may fuse them. But the two vocabularies
are not symmetric: `ExecutionScope` includes `Subgroup`, while `MemoryScope`
offers only `Workgroup` and `Device`. There is therefore **no way to express
subgroup-level memory visibility**, which is exactly what Metal's
`simdgroup_barrier` establishes.

`tiler-metal` handles this correctly today — it **rejects** every subgroup barrier
with a typed `BarrierRejection` rather than widening the claim to workgroup
visibility, which would be unsound in the safe direction that matters (claiming
more synchronisation than the barrier provides). Note also that no in-kernel Metal
barrier provides device-wide visibility at all, so `MemoryScope::Device` is
likewise unrealizable in a kernel body.

Nothing is broken now: the bounded profile emits no subgroup barriers, so the
rejection path is unreachable end to end. This is a reservation whose absence
would only bite when the capability is actually needed.

**Trigger:** when subgroup collectives (shuffles, reductions, ballots) enter the
supported profile. At that point add `MemoryScope::Subgroup`, map it to
`simdgroup_barrier` in `tiler-metal`, and decide explicitly what
`MemoryScope::Device` means for a barrier that cannot provide it — either remove
it from the barrier vocabulary, or document that it is only meaningful outside a
kernel body.

Until then, do **not** relax the rejection. A barrier that claims broader
visibility than the hardware primitive provides is a data race the verifier would
have blessed.
