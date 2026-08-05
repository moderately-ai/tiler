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

**Trigger** (rewritten 2026-08-04 by the deferred sweep; the superseded wording
was "when subgroup collectives (shuffles, reductions, ballots) enter the
supported profile", refuted twice below): **the first schedule declaring a
staged allocation through threadgroup memory whose writer *and every reader* lie
in one subgroup** — a subgroup-private scratch tile. That is the only case in
which a publication narrower than the threadgroup is both sufficient and cheaper
than the one `required_subject` already derives. At that point add
`MemoryScope::Subgroup`, map it to `simdgroup_barrier` in `tiler-metal`, and
decide explicitly what `MemoryScope::Device` means for a barrier that cannot
provide it — either remove it from the barrier vocabulary, or document that it is
only meaningful outside a kernel body.

Until then, do **not** relax the rejection. A barrier that claims broader
visibility than the hardware primitive provides is a data race the verifier would
have blessed.

## The trigger has no mechanism, and the mapping is already half-written (2026-07-28)

**The premise still holds, checked against the current tree.** `ExecutionScope` is `Subgroup | Workgroup` (`crates/tiler-ir/src/kernel/model.rs:338`) and `MemoryScope` is `Workgroup | Device` (`:357`), so subgroup-level memory visibility remains inexpressible.

**The Metal mapping this ticket asks for is already written, and it is dead.** `barrier_call` binds `ExecutionScope::Subgroup => "simdgroup_barrier"` at `crates/tiler-metal/src/emit.rs:1118`, and the very next check throws that binding away: the visibility match admits only `(Workgroup, Workgroup)` and returns `BarrierRejection::MemoryVisibility` for everything else (`:1132-1142`). So the work at trigger time is **removing a rejection**, not writing a mapping — the `call` string is already correct for a subgroup barrier and only the memory scope it needs to pair with is missing from the vocabulary.

**Nothing will announce the trigger.** All four matches in `barrier_call` and `fence_flag` end in a wildcard arm — `emit.rs:1119`, `:1134`, `:1147`, `:1180` — so adding `MemoryScope::Subgroup` to the IR compiles cleanly in `tiler-metal` and every subgroup barrier keeps being rejected, silently and at run time, by the arm at `:1134`. The vocabulary widening this ticket exists to catch is exactly the change that no build error will point at. Note this is not an argument for deleting the wildcards: they are what make an unhandled scope a typed `UnsupportedBarrier` rather than a panic, and `MemoryScope` is `#[non_exhaustive]`, so an out-of-crate match needs one regardless.

**Fix: an exhaustive tripwire in the IR, in the style already used for the body-shaping vocabulary.** `crates/tiler-ir/src/kernel/tests.rs:354::body_shaping_vocabulary_is_closed` is the pattern — a test-only exhaustive `match` whose only job is to fail to compile when the vocabulary widens, with a comment naming the ticket that must then act. Adding the same for `MemoryScope` and `ExecutionScope`, naming this ticket, converts "someone remembers" into a build error. It is deliberately a spelling check and not a semantic one; it cannot tell that a widened vocabulary admits a new barrier, only that the vocabulary widened, which is the point at which a human has to look.

**Who owns it:** whoever lands subgroup collectives (shuffles, reductions, ballots) into the supported profile — the same change that fires the trigger — because that is the first moment the tripwire would have to be updated rather than merely added. Adding the tripwire earlier is cheap and loses nothing.

## The trigger names the wrong construct, and the line citations are stale (2026-08-01)

**The premise still holds and the trigger does not.** `ExecutionScope` is `Subgroup | Workgroup` and `MemoryScope` is `Workgroup | Device`, so subgroup-level memory visibility remains inexpressible — that part is unchanged. But the trigger reads "when subgroup collectives (**shuffles**, reductions, ballots) enter the supported profile", and [the subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md) derives that a shuffle is not the construct that fires this.

**Why a shuffle does not fire it.** **Fact — Metal Shading Language Specification 4.1, §6.10.2.** "SIMD-group functions allow threads in a SIMD-group to share data **without using threadgroup memory or requiring any synchronization operations, such as a barrier**." A shuffle names its source lane and its destination register in one operation that is both the transfer and the ordering, so a shuffle-tree reduction derives no `VisibilityEdge`, declares no `SynchronizationPoint`, and never reaches `barrier_call` at all. It cannot exercise the dead `ExecutionScope::Subgroup => "simdgroup_barrier"` binding, and it needs no `MemoryScope::Subgroup`.

**What does fire it.** A staged handoff *between* simdgroups within one threadgroup — the two-level reduction now filed as [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md). That is the first construct whose values cross invocations through an allocation at a scope narrower than the workgroup, which is what a subgroup memory scope means. **The trigger should be narrowed to that ticket**, and the reductions-and-ballots half of the trigger should be re-examined too: the subgroup tier record derives that subgroup reduction collectives are unusable for a different reason entirely — neither Metal nor WGSL states their combine order — so "reductions enter the supported profile" is not a near-term event.

**Every line citation in the 2026-07-28 addendum is stale.** Checked at `8252312`: `ExecutionScope` and `MemoryScope` are at `crates/tiler-ir/src/kernel/model.rs:562` and `:581`, not `:338` and `:357`; `barrier_call`'s subgroup binding is at `crates/tiler-metal/src/emit.rs:1298` with the visibility match at `:1312-1322`, not `:1118` and `:1132-1142`; `fence_flag` is at `:1353`. The claimed tripwire pattern `crates/tiler-ir/src/kernel/tests.rs:354::body_shaping_vocabulary_is_closed` was not re-verified and should be before it is cited as a model. The addendum's *arguments* are unaffected — the wildcard arms are still there and nothing still announces the trigger — but a reader should not follow its line numbers.

> **This correction has itself drifted, corrected 2026-08-04 by the stale-claim sweep at base `c4b4bdb9`.** Every number in the paragraph above is now wrong too, which is the recurrence a reader should expect from a line citation rather than a surprise. Current sites, each read rather than searched: `ExecutionScope` is `crates/tiler-ir/src/kernel/model.rs:595` and `MemoryScope` is `:614`; `barrier_call` is `crates/tiler-metal/src/emit.rs:1601` with its subgroup binding at `:1604` and the visibility-match rejection at `:1618-1626`, the `BarrierRejection::MemoryVisibility` return being `:1622`; `fence_flag` is `emit.rs:1659`. The tripwire pattern the 2026-07-28 addendum cited unverified **does exist and is now verified**: `body_shaping_vocabulary_is_closed` is at `crates/tiler-ir/src/kernel/tests.rs:853`, not `:354`, so it is sound to cite as a model. Reproduce with `grep -n 'pub enum ExecutionScope\|pub enum MemoryScope' crates/tiler-ir/src/kernel/model.rs`, `grep -n 'fn barrier_call\|fn fence_flag\|simdgroup_barrier' crates/tiler-metal/src/emit.rs`, and `grep -n 'fn body_shaping_vocabulary_is_closed' crates/tiler-ir/src/kernel/tests.rs`. **The premise and every argument in this ticket are unchanged**, and were re-read to confirm it: subgroup-level memory visibility is still inexpressible, the `simdgroup_barrier` binding is still dead behind the visibility match, and the wildcard arms still mean nothing announces the trigger. Line drift, not a changed claim. [`close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire`](close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire.md) is the dispatchable owner of the tripwire and carries its own citations; those had drifted as well and were corrected in the same sweep.

## The proposed narrowing is wrong too: a handoff *between* subgroups needs *workgroup* visibility (2026-08-01, second addendum)

**The premise still holds; the correction directly above does not.** The addendum above proposed narrowing the trigger to "a staged handoff *between* simdgroups within one threadgroup — the two-level reduction now filed as [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md)", on the ground that this is "the first construct whose values cross invocations through an allocation at a scope narrower than the workgroup". [The two-level subgroup-then-workgroup reduction](../docs/research/scheduling/two-level-subgroup-workgroup-reduction.md) §3 derives that the ground is backwards, and the correction belongs here because a wrong trigger makes unreachable work look reachable.

**Fact — Metal Shading Language Specification 4.1, §6.16.2, page 300 (2026-06-04).** The `thread_scope` enumeration is `thread_scope_thread`, `thread_scope_simdgroup`, `thread_scope_threadgroup`, `thread_scope_device`, and "Informally, the thread scope on a synchronization operation defines the set of threads with which this operation may synchronize, or which may synchronize with the operation."

**Fact — MSL 4.1, §6.10.1, page 216.** "The scope argument (see section 6.16.2) specifies which threads can observe the memory accesses to the address space identified by flags. The accesses become visible within the same threadgroup, within the same SIMD-group, or across all threads on the device."

**Inference, in one line.** A subgroup memory scope publishes *within one SIMD-group*. In the two-level reduction, lane 0 of SIMD-group `g` stages a partial that the committing invocation — which is in SIMD-group 0 — reads, so for every `g ≠ 0` the writer and the reader are in different SIMD-groups and a subgroup-scoped publication does not reach the reader. **Crossing a boundary requires reaching across it, so a handoff between SIMD-groups requires threadgroup-scoped visibility** — `SynchronizationScope::Workgroup` and `MemoryScope::Workgroup`, which is exactly the subject `required_subject` (`crates/tiler-ir/src/schedule/synchronization.rs`) already derives for any handoff over workgroup staging. The two-level reduction therefore does **not** fire this ticket, and neither the original trigger (shuffles) nor the narrowing above (a handoff between simdgroups) names a construct that would.

**What would fire it, stated so it can be recognized:** the first schedule declaring a staged allocation through threadgroup memory whose writer *and every reader* lie in one subgroup — a subgroup-private scratch tile — because that is the only case in which a publication narrower than the threadgroup is both sufficient and cheaper than the one already derived. Nothing in the work graph currently proposes such a schedule.

**Consequence for this ticket.** It stays `deferred`, and its trigger has now been refuted twice, each time by the construct that was expected to fire it. The trigger line in the body above should be rewritten to the sentence in the previous paragraph; that rewrite is deliberately *not* made here, because changing a deferral's activation condition is a graph decision and this addendum is the evidence for it rather than the decision itself. The 2026-07-28 addendum's tripwire proposal is unaffected and remains the right mechanism whenever the trigger does fire.

## The tripwire is built, and the claim that justified it was half wrong (2026-08-05)

**The 2026-07-28 addendum's proposed fix now exists.** [`close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire`](close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire.md) landed `barrier_scope_vocabulary_is_closed` — a test-only exhaustive match over `ExecutionScope` and `MemoryScope` in `crates/tiler-ir/src/kernel/tests.rs`, consumed by `the_barrier_scope_vocabularies_are_still_closed`, whose doc comment names **this** ticket. Widening either vocabulary now breaks that build with the owner written beside the error. The visibility match in `crates/tiler-metal/src/emit.rs` carries the reciprocal pointer, so the wildcard arm a reader lands on says where the announcement lives. Nothing was admitted and nothing relaxed: the rejection this ticket protects is untouched, and `MemoryScope` is still `Workgroup | Device`.

**Correction — "nothing will announce the trigger" was true only downstream.** The 2026-07-28 addendum says adding `MemoryScope::Subgroup` to the IR "compiles cleanly in `tiler-metal`", which is exactly right, and then generalizes to "the vocabulary widening this ticket exists to catch is exactly the change that no build error will point at", which is not. **Measurement — the perturbation, run 2026-08-05 at base `692d323e` on the tripwire's branch, `cargo check -p tiler-ir -p tiler-metal --all-targets` under the pinned toolchain.** Adding `MemoryScope::Subgroup` to `crates/tiler-ir/src/kernel/model.rs` already failed at two sites *before* the tripwire existed: `MemoryScope::tag` (`model.rs`, the identity encoding) and `verify::barrier_subject` (`crates/tiler-ir/src/kernel/verify.rs`), whose own doc comment states the property — "`#[non_exhaustive]` has no effect inside the defining crate, so widening either vocabulary is a build error here". Satisfying both of those and re-running left `tiler-metal` checking **clean** (exit 0) while `tiler-ir`'s lib test failed at the tripwire alone. The same procedure on `ExecutionScope` gave the same result.

**Inference — what the tripwire contributes is the owner, not the build error.** A developer widening `MemoryScope` was always going to be stopped twice inside `tiler-ir`, and the natural repair at both sites — a tag byte and a `SynchronizationScope` projection — is local, plausible, and completely silent about `tiler-metal`, where the widened scope compiles and every barrier naming it is then rejected at run time. The tripwire's value is that the third break names this ticket. That is a narrower claim than the addendum's, and it is the one the measurement supports.

**Current citations, replacing the 2026-08-04 sweep's, which drifted again as it predicted.** Read at `692d323e` with the tripwire applied: `ExecutionScope` is `crates/tiler-ir/src/kernel/model.rs:609` and `MemoryScope` is `:628`; `barrier_call` is `crates/tiler-metal/src/emit.rs:1613` with its subgroup binding at `:1616`; `fence_flag` is `:1678`; `verify::barrier_subject` is `crates/tiler-ir/src/kernel/verify.rs:374`; the tripwire is `crates/tiler-ir/src/kernel/tests.rs:966` and its test `:987`. Reproduce with `grep -n 'pub enum ExecutionScope\|pub enum MemoryScope' crates/tiler-ir/src/kernel/model.rs`, `grep -n 'fn barrier_call\|fn fence_flag\|simdgroup_barrier' crates/tiler-metal/src/emit.rs`, and `grep -n 'fn barrier_subject' crates/tiler-ir/src/kernel/verify.rs`. **These will drift too**, which is why the tripwire's own doc comment cites constructs rather than lines, and why a reader should prefer the grep to the number.

## Trigger check log

- 2026-08-04 — **not fired**, and the edge question the second addendum left open is decided here. **The trigger is rewritten** to the sentence the two-level reduction record derived — the superseded wording is preserved inline above so a reader can see what was refuted rather than only that something was. **No frontmatter edge to [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) is added, and the ground is that the edge would encode a refuted claim.** The 2026-08-01 first addendum proposed narrowing the trigger to that ticket; the second addendum refuted it from MSL 4.1 §6.16.2 and §6.10.1 — a handoff *between* SIMD-groups needs threadgroup-scoped visibility, which `required_subject` already derives — and that ticket is now `done` without this capability, which is the outcome the refutation predicts. An edge is a scheduling claim that one ticket's completion bears on another's readiness; adding one whose premise is disproved would make unreachable work look reachable, which is exactly the failure the addendum was written to prevent. **Nothing in the graph currently proposes a subgroup-private scratch tile**, so there is no ticket to point at and the rewritten trigger stays a corpus-state condition rather than an edge. The 2026-07-28 tripwire proposal — an exhaustive test-only match over `MemoryScope` and `ExecutionScope` naming this ticket, in the style of `body_shaping_vocabulary_is_closed` — remains the right announcement mechanism and is still unbuilt; it is what would make the vocabulary widening a build error rather than a silent run-time rejection. Recheck: the premise, unchanged — `ExecutionScope` is `Subgroup | Workgroup` and `MemoryScope` is `Workgroup | Device` in `crates/tiler-ir/src/kernel/model.rs`.
- 2026-08-05 — **not fired**, and the announcement mechanism the line above calls "still unbuilt" is now built; see the 2026-08-05 addendum. The trigger is a subgroup-private scratch tile, and it is unreachable by construction rather than merely unproposed: `required_subject` (`crates/tiler-ir/src/schedule/synchronization.rs`) derives *every* staged handoff's subject with `execution_scope` and `visibility_scope` both fixed at `SynchronizationScope::Workgroup` — they are literals in the constructor, not a function of the tile — so no schedule this builder can produce declares a publication narrower than the threadgroup. Firing this trigger therefore requires a change to that derivation first, which is a larger event than a new schedule and would be impossible to land quietly. Recheck: `grep -n 'visibility_scope' crates/tiler-ir/src/schedule/synchronization.rs` — the sole derivation site still reads `SynchronizationScope::Workgroup`.
