---
id: thread-resolved-lowering-into-the-governed-spelling-path
title: Thread resolved lowering into the governed spelling path
status: blocked
priority: p1
dependencies: [bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence, lower-a-recognized-gather-through-a-governed-capability]
related: [decide-whether-refinement-evidence-may-reach-a-physical-provider, emit-the-indirect-gather-on-metal]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, gather, frontier]
---
## User-visible outcome

Physical planning spells a gather region using the proof the occurrence already carries, so `RegionVocabularyWall::GatherProofUnavailable` retires because the argument arrived — not because the check was relaxed.

## Why this exists

Filed 2026-08-22 from the refinement-seam packet. **The wall is a missing argument, not a missing boundary.** `spell_region` takes only a `&VerifiedTargetRequest` while the proof sits in `ResolvedLowering` in the same `plan_target` scope — the value exists at the right time and no seam carries it.

**Fact — this adds no public surface.** The packet verified item by item that every element it touches is `pub(crate)` or private: `enumerate_frontier`, `govern_spelling`, `spell_region`, `spell_output`, `verify_schedule_with_feasibility`, `verify_region_output_binding`, `gather_accesses_match`, `ResolvedLowering`. So it does not meet ADR 0075's reservation bar and is **not a Tom decision**. Re-derive that list yourself before relying on it.

**Fact — this strengthens verifier independence rather than costing it.** `verify_portfolio` in `crates/tiler-compiler/src/pipeline/verify.rs` already calls `resolve_lowering` itself. Threading planning's value in makes the verifier compare planning's retained proof against its **own** re-derivation. **Nothing borrows** — which matters, because the deliberate independence of `pipeline/verify.rs` is the property that makes verification meaningful, and a seam that let it borrow would retire that quietly.

**Three routes were eliminated before ranking, and the grounds are worth keeping.** Re-deriving the proof during planning conflates identities — the wall's own stated reason. A public proof constructor inverts deriver privacy. Dropping the proof and re-deriving at the schedule layer is tempting, since both current kinds *are* relation-derivable, but it depends on the kind set never growing, and the invocation-validation resolution is exactly a third kind that is **not** relation-derivable.

## Required work

- **Do not start until [`bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence`](bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence.md) lands.** Threading the value before the occupancy check exists would carry an unchecked proof further, which is the wrong order.
- Re-audit both Facts at your base with a per-Fact verdict, re-deriving the visibility list rather than inheriting it. **If any item turns out `pub`, stop and report** — that would make this a public-boundary change and therefore Tom's.
- Thread the already-derived `ResolvedLowering` into the spelling path **and** the proposal-verification path; land `physical::gather_region`; retire the wall.
- Perturb the subject separately for each behaviour and quote the failure text, including a control that a proof failing the occupancy check still refuses after the threading.
- State every identity domain that steps and every one that does not, derived on the merged tree.

## Non-goals

Widening `ImplementationContext`; the Metal emission, which depends on this; and re-opening the accepted data-dependent index surface.

## Closes when

A gather region is spelled from its own occurrence's proof, the wall is retired because the argument arrived, the verifier still re-derives independently, each behaviour is watched failing on its own subject, and the workspace gate is green.

## Exact-base Fact audit — 2026-08-23, `9b61b563f66112b49a65b652990bc414e8a6fbb5`, `worker-thread`

Read in full at this base before any edit: this ticket; root `AGENTS.md`; `crates/tiler-compiler/src/physical.rs` around the wall, the spelling path, and the gather binding; `crates/tiler-compiler/src/pipeline/verify.rs`; `crates/tiler-compiler/src/pipeline/planning.rs`'s plan-enumeration transaction; `crates/tiler-compiler/src/frontier.rs`'s `ImplementationContext`, `enumerate_frontier`, and `govern_spelling`; `crates/tiler-compiler/src/lowering.rs`; `crates/tiler-compiler/src/capability.rs`'s lowering facade; and the complete parent tickets [`carry-the-gather-relation-through-the-compiler-vertical`](carry-the-gather-relation-through-the-compiler-vertical.md) and [`decide-whether-refinement-evidence-may-reach-a-physical-provider`](decide-whether-refinement-evidence-may-reach-a-physical-provider.md).

| Fact as filed | Verdict | Evidence re-derived at this base |
|---|---|---|
| This adds no public surface: all eight named items are `pub(crate)` or private | **verified**, list re-derived rather than inherited | `enumerate_frontier` `pub(crate) fn` (`frontier.rs`); `govern_spelling` bare `fn` (`frontier.rs`); `spell_region` `pub(crate) fn`, `spell_output` bare `fn`, `verify_schedule_with_feasibility` `pub(crate) fn`, `verify_region_output_binding` bare `fn`, `gather_accesses_match` bare `fn` (all `physical.rs`); `ResolvedLowering` `pub(crate) struct` (`lowering.rs`). **None is `pub`.** One near-miss worth naming so a re-auditor does not trip on it: `crates/tiler-compiler/src/capability.rs` declares `pub struct ResolvedLoweringCapability`, a *different* type whose name shares a prefix — a bare grep for `ResolvedLowering` hits it and reads as a public item on this list. |
| Verifier independence holds; `verify_portfolio` calls `resolve_lowering` itself | **verified** | `crates/tiler-compiler/src/pipeline/verify.rs`, anchor `let lowering = resolve_lowering(semantic, request).map_err(|_| {`, at line 83 as filed. Its own module header states the property positively: `**Nothing here may reuse a planning intermediate.**` Planning's own call is a separate one in `pipeline/planning.rs`, anchor `let lowering = match resolve_lowering(semantic, verified) {`. The two derivations are independent and nothing threaded here would make the verifier borrow planning's. |
| The proof "sits in `ResolvedLowering` in the same `plan_target` scope — the value exists at the right time and no seam carries it" | **FALSE, and it is the reason this ticket cannot be delivered at this base.** See below. | — |

## Blocking discovery — the argument cannot arrive, because no gather ever lowers

**The wall is not only a missing argument. Behind it is a missing capability row, and that row is unfiled work.**

- `IndexAccessLoweringContext` — the compiler-side lowering facade in `crates/tiler-compiler/src/capability.rs` — exposes `dimension`, `input_tensor`, `output_tensor`, `constant`, `dimension_expr`, `linear_combination`, `floor_div`, `modulo`, `read`, `write`, `apply`, `apply_in`, `reduce`, `output`, and **no `gather_read`**. `git log -S "gather_read" -- crates/tiler-compiler/src/capability.rs` is empty: it was drafted and withdrawn inside the parent lane, never landed. No lowering provider, governed or installed, can emit a gather region.
- The governed registry holds no gather row: `grep -rn -i "gather" crates/tiler-compiler/src/governed.rs` returns one comment about a reindex that is *not* a gather, and the same grep over `capability.rs` returns zero. `GOVERNED_INDEX_ACCESS_CAPABILITIES` is still `21`.
- So `resolve_lowering` refuses every gather. `a_gather_occurrence_resolves_no_lowering_at_this_base` pins exactly that with reason `missing-capability`, carries a negative control showing the same call answers `Ok` for an elementwise fixture, and **passes at this base**.
- That refusal happens *above* physical planning. `enumerate_complete_plans` opens with `let lowering = match resolve_lowering(semantic, verified) {` and returns on the error arm before `enumerate_covers`, `enumerate_frontier`, `govern_spelling`, or `spell_region`. **No gather member set reaches the region-vocabulary check through the pipeline at all.** The one live caller of `spell_region` with gather members is a unit test that constructs the request directly.

`ResolvedLowering::unresolved_for_test`'s own documentation states the same fact from the other side. Its sentence wraps across two `///` lines and carries an inline link, so the anchor is the single-line tail `because this one refuses every gather` in `crates/tiler-compiler/src/lowering.rs` — quoting the whole sentence greps to zero there and would read as false absence.

**Consequence for each of this ticket's three deliverables.**

1. *Thread `&ResolvedLowering` into the spelling path.* Mechanically possible, but every reachable call site would pass a value no reachable path reads, because the only arm that would read it is unreachable. No check on it could fail — the defect this repository keeps finding.
2. *Land `physical::gather_region`.* It can only build a region by cloning the occurrence's statically proved bounds resolution out of `refinement.single_region()`'s gather access view. No gather has a refinement, so the function would have zero reachable callers, produce zero regions, and need `#[allow(dead_code)]`. This is verbatim the state the parent lane withdrew `IndexAccessLoweringContext::gather_read` to avoid.
3. *Retire the wall because the argument arrived.* Not achievable. The argument arrives empty for 100% of the population. Retiring the wall would either relax the check or rename the refusal — both of which the ticket's own Outcome forbids: *"a repair that merely stops refusing is a defect, not a fix."* The brief's required occupancy-refusal control would be the **only** reachable case, so the lane could produce the negative control and never the positive one.

**A test in the tree already names the ordering this ticket was dispatched against.** `a_governed_gather_refuses_at_dispatch_before_governed_lowering` says two named authorities stand between its refusal and one that could grant a gather a schedule: the governed capability row itself, and `RegionVocabularyWall::GatherProofUnavailable`, *"which is what physical planning answers once a row exists"*. The row is first. This ticket is second. Its doc comment wraps, so the anchor is the single-line fragment `authorities stand between this refusal and one that could` in `crates/tiler-compiler/src/request/tests.rs`; the sentence spanning the wrap greps to zero.

**Provenance of the false Fact, since it is worth not re-making.** It is not the coordinator's invention; it entered at [`carry-the-gather-relation-through-the-compiler-vertical`](carry-the-gather-relation-through-the-compiler-vertical.md)'s blocking discovery ("The value does exist at the right time"), was carried verbatim into [`decide-whether-refinement-evidence-may-reach-a-physical-provider`](decide-whether-refinement-evidence-may-reach-a-physical-provider.md)'s option table as B2, and reached this ticket third-hand. It was true of the *scope* — `resolve_lowering` and `enumerate_frontier` are in one function with the value live — and false of the *population*, because that value never contains a gather. Both prior lanes correctly listed the capability row as remainder item 2 and described it as "independent"; what neither did was file it as a ticket or make it an edge, so the dependency was invisible to dispatch.

**Nothing was edited under `crates/`.** No code change was landed, because every available one is either untestable plumbing or a relaxation of the check this ticket exists to strengthen.

## Repair — the missing prerequisite is now filed and linked

[`lower-a-recognized-gather-through-a-governed-capability`](lower-a-recognized-gather-through-a-governed-capability.md) owns the facade method, the `GovernedGatherF32` provider, the capability row 21 to 22, and the refinement evidence, and is now a hard dependency of this ticket. It carries one consequence this ticket does not, which is why it is a lane of its own: `CanonicalLoweringRegistryIdentity` folds the capability list and `crates/tiler-compiler/src/request/subject.rs` writes it into every request subject (`push_slice(&mut bytes, self.lowering_registry.as_bytes());`), so adding a row **steps the request-subject identity domain for every program in the repository**, not only for gathers.

This ticket's own Facts, its Outcome, and its Closes-when survive unchanged once that lands. The coordinator owns whether to move this ticket to `blocked` and whether to re-dispatch it behind the new one.

## Coordinator verification of the blocking discovery — 2026-08-23 at `9b61b563`

`worker-thread` refused to land this ticket's stated work and repaired the premise instead. **That was the correct call**, and every element of its blocker reproduces:

- `grep -c "gather_read" crates/tiler-compiler/src/capability.rs` returns **0**. `IndexAccessLoweringContext` — the only emission vocabulary any lowering provider gets — exposes no gather.
- `git log -S "gather_read" -- crates/tiler-compiler/src/capability.rs` is **empty**. The facade was drafted and withdrawn inside the parent lane and never landed, so this is not drift; it was never there.
- `GOVERNED_INDEX_ACCESS_CAPABILITIES` is **21** at `crates/tiler-compiler/src/governed.rs`, with no gather row.
- `enumerate_complete_plans` states the ordering in its own words at the anchor `Lowering-capability resolution precedes every cover`, and returns on the `resolve_lowering` error arm above `enumerate_covers`.

**So the argument this ticket exists to deliver can never arrive.** Threading it would pass a value no reachable path reads; `physical::gather_region` would have zero callers and need `#[allow(dead_code)]`, which is verbatim the state the parent lane withdrew the facade to avoid; and the required occupancy-refusal control would have a reachable negative case and **no** reachable positive one. Landing that is "the check was relaxed" wearing the costume of "the argument arrived" — the exact substitution this ticket's own outcome forbids.

**The provenance matters more than the error.** The claim entered at `carry-the-gather-relation-through-the-compiler-vertical`'s blocking discovery, was carried into an option row, and reached the coordinator's brief third-hand. It was **true of the scope and false of the population**: the value really does exist in `plan_target`, and no gather ever reaches the scope that would read it. Two prior lanes both listed the capability row as remainder and called it independent, and **neither filed it as a ticket or an edge** — so it was invisible to dispatch and this ticket was scheduled ahead of its own prerequisite. The repair is [`lower-a-recognized-gather-through-a-governed-capability`](lower-a-recognized-gather-through-a-governed-capability.md), now filed with a hard dependency edge, and it carries an identity consequence this ticket does not: the lowering registry list is folded into `CanonicalLoweringRegistryIdentity` and written by `request/subject.rs`, so adding one row steps the request-subject domain for **every program in the repository**.

Status moved to `blocked` by the coordinator, which the worker correctly left to me.

## Correction — 2026-08-23, from the prerequisite lane at `d9f1a000`

`worker-gathercap` landed [`lower-a-recognized-gather-through-a-governed-capability`](lower-a-recognized-gather-through-a-governed-capability.md). Four Facts above are now historical, and two of them **grep as absence rather than as change**, which is the dangerous direction. Re-audit all of them at your own base before editing anything.

- `grep -c "gather_read" crates/tiler-compiler/src/capability.rs` no longer returns 0: the accepted facade landed verbatim.
- `GOVERNED_INDEX_ACCESS_CAPABILITIES` is **22**, not 21, and `governed.rs` carries a `GovernedGatherF32` row.
- `a_gather_occurrence_resolves_no_lowering_at_this_base` **no longer exists under that name**, and a grep for it returns 0. It was inverted and renamed `a_gather_occurrence_resolves_a_governed_lowering_and_refines`; a gather now resolves, refines to `OccurrenceEvidence::Refined`, and its single region's gather access carries a statically proved `GatherIndexBoundsProof`.
- `a_governed_gather_refuses_at_dispatch_before_governed_lowering` is likewise renamed, to `a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary`, and its widened half now pins `("planning", "region-vocabulary")`. The anchor `authorities stand between this refusal and one that could` was rewritten and greps to 0; the surviving single-line fragment is `named authority now stands between this refusal and one that could`.

**So the premise this ticket was blocked on is satisfied**: the proof exists in `ResolvedLowering` at planning time, and the test above reads it out. What remains unchanged is this ticket's own outcome — `physical::gather_region`, the `govern_spelling` gather arm, and retiring `RegionVocabularyWall::GatherProofUnavailable` were all explicit non-goals of the prerequisite and none was touched. The wall still declines **every** gather member set unconditionally, so a statically proved gather and one owing invocation validation are still indistinguishable above lowering; separating them is this ticket's and [`admit-an-invocation-scoped-gather-index-validation-receipt`](admit-an-invocation-scoped-gather-index-validation-receipt.md)'s.

**The identity consequence is discharged, not pending.** The request-subject domain stepped for every program; exactly one pin in the workspace named it and it was recomputed on the merged tree. Adding no capability row, this ticket steps nothing further — but rederive rather than trusting that.
