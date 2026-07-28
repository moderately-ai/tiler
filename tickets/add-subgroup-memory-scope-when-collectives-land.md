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

## The trigger has no mechanism, and the mapping is already half-written (2026-07-28)

**The premise still holds, checked against the current tree.** `ExecutionScope` is `Subgroup | Workgroup` (`crates/tiler-ir/src/kernel/model.rs:338`) and `MemoryScope` is `Workgroup | Device` (`:357`), so subgroup-level memory visibility remains inexpressible.

**The Metal mapping this ticket asks for is already written, and it is dead.** `barrier_call` binds `ExecutionScope::Subgroup => "simdgroup_barrier"` at `crates/tiler-metal/src/emit.rs:1118`, and the very next check throws that binding away: the visibility match admits only `(Workgroup, Workgroup)` and returns `BarrierRejection::MemoryVisibility` for everything else (`:1132-1142`). So the work at trigger time is **removing a rejection**, not writing a mapping — the `call` string is already correct for a subgroup barrier and only the memory scope it needs to pair with is missing from the vocabulary.

**Nothing will announce the trigger.** All four matches in `barrier_call` and `fence_flag` end in a wildcard arm — `emit.rs:1119`, `:1134`, `:1147`, `:1180` — so adding `MemoryScope::Subgroup` to the IR compiles cleanly in `tiler-metal` and every subgroup barrier keeps being rejected, silently and at run time, by the arm at `:1134`. The vocabulary widening this ticket exists to catch is exactly the change that no build error will point at. Note this is not an argument for deleting the wildcards: they are what make an unhandled scope a typed `UnsupportedBarrier` rather than a panic, and `MemoryScope` is `#[non_exhaustive]`, so an out-of-crate match needs one regardless.

**Fix: an exhaustive tripwire in the IR, in the style already used for the body-shaping vocabulary.** `crates/tiler-ir/src/kernel/tests.rs:354::body_shaping_vocabulary_is_closed` is the pattern — a test-only exhaustive `match` whose only job is to fail to compile when the vocabulary widens, with a comment naming the ticket that must then act. Adding the same for `MemoryScope` and `ExecutionScope`, naming this ticket, converts "someone remembers" into a build error. It is deliberately a spelling check and not a semantic one; it cannot tell that a widened vocabulary admits a new barrier, only that the vocabulary widened, which is the point at which a human has to look.

**Who owns it:** whoever lands subgroup collectives (shuffles, reductions, ballots) into the supported profile — the same change that fires the trigger — because that is the first moment the tripwire would have to be updated rather than merely added. Adding the tripwire earlier is cheap and loses nothing.
