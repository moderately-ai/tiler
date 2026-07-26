---
id: correct-adr-0081-loader-gap-list
title: Correct ADR 0081's loader gap list against the projected dispatch record
status: todo
priority: p3
dependencies: []
related: [route-the-runtime-loader-through-the-dispatch-record, expose-the-dispatch-record-on-a-decoded-artifact, carry-reconstructable-kernel-programs-in-the-neutral-envelope, correct-artifact-abi-reconstruction-ownership]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contract, runtime, artifact]
---
[ADR 0081](../docs/decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md)'s last Consequences bullet states four things about what the loader cannot do. Three of them are now false, and the fourth names a `done` ticket as their present owner.

**Fact — the stale sentence, reproducible in one line.** `grep -n "cannot say which buffer a binding slot addresses" docs/decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md` returns the `implementation_status` bullet. It reads that the loader "cannot evaluate an applicability guard, a binding's accessible byte range, or a launch formula, because those rows are held in the envelope and reachable only through a `VerifiedArtifactProgram` no decode produces; it cannot say which buffer a binding slot addresses, because `BindingData` carries no value reference; and it cannot read a payload's entry symbol, because the payload-metadata section has no public parser."

**Fact — what the code does.** [`expose-the-dispatch-record-on-a-decoded-artifact`](expose-the-dispatch-record-on-a-decoded-artifact.md) landed `DecodedVariant::applicability_guard`, `DecodedBinding::accessible_bytes`, `DecodedEntry::launch_threads`, `DecodedExpr::evaluate`, `DecodedBinding::target`, `DecodedArtifact::payload_metadata`, and `DecodedEntry::backend_symbol`, all `pub` in `crates/tiler-artifact/src/program/codec/view.rs`. [`route-the-runtime-loader-through-the-dispatch-record`](route-the-runtime-loader-through-the-dispatch-record.md) then routed the loader through them, and recorded the same three clauses as stale where they appear in `crates/tiler-runtime/src/load/route.rs` — but it held `implementation/runtime` only and could not reach `contracts/decisions`, so the ADR still carries them.

**Fact — the ownership half.** `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is `done`; Tom decided on 2026-07-25 that a decoded envelope is a dispatch record and never a reconstruction, so it is no longer anyone's open question. The same pattern in `docs/artifact-abi.md` was corrected under [`correct-artifact-abi-reconstruction-ownership`](correct-artifact-abi-reconstruction-ownership.md), which holds `contracts/artifacts` and could not reach this record either.

**Why p1 rather than a wording nit.** An accepted ADR is a current decision, and a reader has every reason to believe its `implementation_status` rationale. This one makes implemented, public, exercised capability look absent, and points at a closed ticket for work nobody is doing.

## Scope

Rewrite that bullet against the code, keeping the `partial` status honest rather than rounding it up: state which gaps genuinely remain — a variant that dispatches more than one stage is still refused ([`carry-the-stage-execution-order-in-the-envelope`](carry-the-stage-execution-order-in-the-envelope.md)), and a binding addressing part of a value is still unpackageable ([`carry-the-byte-offset-of-a-partial-binding-view`](carry-the-byte-offset-of-a-partial-binding-view.md)) — and name those live owners.

**Preserve the ADR's decision status and rationale.** Correcting a factual sentence inside an accepted record is not a superseding decision and must not be written as one. If the correction turns out to change what the ADR *decided* rather than what it *described*, stop and say so.

**Check ADR 0071 for the same pair of failures while holding this scope.** `grep -n "belongs to" docs/decisions/0071-use-checked-builders-for-shared-compiler-ir.md` returns a boundary paragraph reading that whether a decoded artifact is "permanently a dispatch record" belongs to `carry-reconstructable-kernel-programs-in-the-neutral-envelope`, "which weighs both and also owns reconciling [the artifact ABI contract]". Both halves are overtaken: the question is decided, and the ABI contract is reconciled.

## Closes when

No sentence in either record states a capability the code has as absent, or names a `done` ticket as a present owner; both records' `decision_status` and rationale are untouched; and `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
