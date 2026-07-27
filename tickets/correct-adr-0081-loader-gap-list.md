---
id: correct-adr-0081-loader-gap-list
title: Correct ADR 0081's loader gap list against the projected dispatch record
status: done
priority: p3
dependencies: []
related: [route-the-runtime-loader-through-the-dispatch-record, expose-the-dispatch-record-on-a-decoded-artifact, carry-reconstructable-kernel-programs-in-the-neutral-envelope, correct-artifact-abi-reconstruction-ownership]
scopes: [contracts/decisions, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contract, runtime, artifact]
---
[ADR 0081](../docs/decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md)'s last Consequences bullet states four things about what the loader cannot do. Three of them are now false, and the fourth names a `done` ticket as their present owner.

**Fact — the stale sentence, reproducible in one line.** `grep -n "cannot say which buffer a binding slot addresses" docs/decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md` returns the `implementation_status` bullet. It reads that the loader "cannot evaluate an applicability guard, a binding's accessible byte range, or a launch formula, because those rows are held in the envelope and reachable only through a `VerifiedArtifactProgram` no decode produces; it cannot say which buffer a binding slot addresses, because `BindingData` carries no value reference; and it cannot read a payload's entry symbol, because the payload-metadata section has no public parser."

**Fact — what the code does.** [`expose-the-dispatch-record-on-a-decoded-artifact`](expose-the-dispatch-record-on-a-decoded-artifact.md) landed `DecodedVariant::applicability_guard`, `DecodedBinding::accessible_bytes`, `DecodedEntry::launch_threads`, `DecodedExpr::evaluate`, `DecodedBinding::target`, `DecodedArtifact::payload_metadata`, and `DecodedEntry::backend_symbol`, all `pub` in `crates/tiler-artifact/src/program/codec/view.rs`. [`route-the-runtime-loader-through-the-dispatch-record`](route-the-runtime-loader-through-the-dispatch-record.md) then routed the loader through them, and recorded the same three clauses as stale where they appear in `crates/tiler-runtime/src/load/route.rs` — but it held `implementation/runtime` only and could not reach `contracts/decisions`, so the ADR still carries them.

**Fact — the ownership half.** `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is `done`; Tom decided on 2026-07-25 that a decoded envelope is a dispatch record and never a reconstruction, so it is no longer anyone's open question. The same pattern in `docs/artifact-abi.md` was corrected under [`correct-artifact-abi-reconstruction-ownership`](correct-artifact-abi-reconstruction-ownership.md), which holds `contracts/artifacts` and could not reach this record either.

**Why this is not a wording nit.** An accepted ADR is a current decision, and a reader has every reason to believe its `implementation_status` rationale. This one makes implemented, public, exercised capability look absent, and points at a closed ticket for work nobody is doing.

## Scope

Rewrite that bullet against the code, keeping the `partial` status honest rather
than rounding it up. The decoded dispatch record now exposes and validates
guards, ranges, launch formulas, binding targets, symbols, execution order, and
dependency obligations. The remaining gaps are that runtime preflight and
selection still cover one entry rather than a complete multi-entry route
([`preflight-every-entry-of-a-multi-stage-route`](preflight-every-entry-of-a-multi-stage-route.md)),
and a binding addressing part of a value is still unpackageable
([`carry-the-byte-offset-of-a-partial-binding-view`](carry-the-byte-offset-of-a-partial-binding-view.md)).
Name those live owners.

**Preserve the ADR's decision status and rationale.** Correcting a factual sentence inside an accepted record is not a superseding decision and must not be written as one. If the correction turns out to change what the ADR *decided* rather than what it *described*, stop and say so.

**Check ADR 0071 for the same pair of failures while holding this scope.** `grep -n "belongs to" docs/decisions/0071-use-checked-builders-for-shared-compiler-ir.md` returns a boundary paragraph reading that whether a decoded artifact is "permanently a dispatch record" belongs to `carry-reconstructable-kernel-programs-in-the-neutral-envelope`, "which weighs both and also owns reconciling [the artifact ABI contract]". Both halves are overtaken: the question is decided, and the ABI contract is reconciled.

## Closes when

No sentence in either record states a capability the code has as absent, or names a `done` ticket as a present owner; both records' `decision_status` and rationale are untouched; any catalog block quoting either record is updated by hand in the same change; and `make full` passes.

## Outcome — corrected, and one of this ticket's own instructions was stale (2026-07-27)

**Every claim was re-verified against the source before editing.** All seven accessors the ticket named are `pub` in `crates/tiler-artifact/src/program/codec/view.rs`, and `crates/tiler-runtime/src/load/route.rs` carries its own retraction of the four matching clauses it used to make.

**This ticket told me to name a `done` ticket as a live owner, which is the exact defect it was filed to fix.** Its Scope section says the remaining gaps are multi-entry preflight, owned by `preflight-every-entry-of-a-multi-stage-route`, and the partial binding view. That first ticket is `done`: its outcome records that `accept_entry` became `accept_entries`, that the loader now routes every entry in `DecodedVariant::execution_order` and derives shared storage before the routing commit, and that the four per-entry obligations moved inside the per-entry loop. So the corrected bullet names **one** surviving gap, `carry-the-byte-offset-of-a-partial-binding-view` (`in-progress`), and credits the multi-entry work as done. Writing the instruction as given would have reproduced the stale-owner error one ticket down.

**ADR 0081.** The bullet listed five capabilities as absent that are all implemented and exercised. It now states what the loader does, names the accessors and the routing module so a reader can check rather than trust, states the one surviving gap and its live owner, and says why `partial` remains the honest status rather than rounding up.

**ADR 0071.** Both halves of the boundary paragraph were overtaken. The question it deferred is decided, and the ABI-contract divergence it pointed at was reconciled under `correct-artifact-abi-reconstruction-ownership` (`done`). The paragraph now records that this entry still does not decide those questions — which remains true — while making clear nothing is waiting on them.

**A fifth site, outside the ticket's original scopes, found by sweep.** `crates/tiler-artifact/src/program/codec/decode.rs` said the same `done` ticket "owns closing that, and until it does…", framing a decided design as a temporary limitation. `implementation/artifact` was added to this ticket rather than filing a new one: the fix is a single paragraph, and a dispatch for it would cost more than the change and add a merge. That scope-reach problem is precisely what this ticket documents as the reason the ADR stayed stale — `route-the-runtime-loader-through-the-dispatch-record` held `implementation/runtime` and could not reach `contracts/decisions` — so widening the scope by one is what stops the cycle rather than continuing it.

**Both ADRs keep `decision_status: accepted`, their decisions, and their rationale.** Only descriptions changed. `docs/decisions/README.md` carries two catalog lines each for 0071 and 0081, both title-and-status only, and neither title nor status moved, so no catalog block needed editing. The remaining mentions of the `done` ticket in `docs/artifact-abi.md` and `view.rs` cite it as where a decision *was made*, which is correct historical attribution and was left alone.
