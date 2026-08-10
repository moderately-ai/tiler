---
id: wire-the-bf16-reference-to-the-realization-it-is-told
title: Wire the BF16 reference to the realization it is told
status: done
priority: p2
dependencies: []
related: [accept-the-bf16-subnormal-resolution-carrier, carry-a-bf16-subnormal-realization-the-reference-can-be-told, conform-the-bf16-vertical-end-to-end, give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject, route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge]
scopes: [implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, bf16]
---
## User-visible outcome

A BF16 evaluation performed under a flushing contract returns the flushing answer, so a BF16 candidate compiled for the measured Apple9 row can be qualified against a reference that was told what that row does.

## The decision this implements

**Arm A, accepted by Tom on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, on [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md), which carries the full reasoning and the two arms it was chosen between. The format is **derived at the point of use** — the BF16 capability knows its own format by construction — rather than declared as a new subject on `NumericalRealization`. **No `implementation/ir` edit, no identity move.**

## The work, which is small because the machinery landed

Everything this needs already exists in `main`:

- `Bf16SubnormalRealization` and its two application sites, `Bf16Format::accept_operand` (before the decode) and `Bf16Format::commit` (after the single rounding), in `crates/tiler-reference/src/bf16.rs`.
- `Bf16BinaryReference::combine_under`, which takes a realization per evaluation.
- `ReferenceEvaluationRequest` conformance surface in `crates/tiler-reference/src/registry.rs` (at delivery the bare `conformance()` getter; later `conformance_for`), already carried on every request.

What is missing is one link: `impl ReferenceOperation for Bf16BinaryReference::evaluate` calls `self.combine(left, right)`, and `combine` delegates with `Bf16SubnormalRealization::preserving()`. **Build the realization from the request's two `SubnormalMode`s and call `combine_under` instead.**

Read [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md)'s decision section in full before starting — it records why a mixed-width refusal is deliberately *not* part of this, and adding one would reintroduce an unreachable check the decision rejected.

## What this must not do

- **No mixed-width refusal.** A multi-format region cannot be constructed — `region_arithmetic_type` in `crates/tiler-ir/src/schedule/model.rs` is a total function from `ScalarProgram` to one `ArithmeticType` — so such a refusal is unreachable and cannot be watched failing. The decision drops it explicitly.
- **No subject added to `NumericalRealization`.** That is arm B, closed against a trigger in [`subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types`](subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types.md).
- **No change to `BF16_FACT_SUBNORMALS`.** Its unconditional `preserved-…` states what the operation *means*; a flushing realization is a declared deviation a region's contract carries, not a second opinion about semantics. Weakening it is the authority substitution ADR 0076 forbids, and it would move the registry snapshot and every identity derived from it.
- **No widening of the binary32 conformance object to stand in for a BF16 one**, and no approximation of BF16 flushing with the binary32 modes. Read only the two format-agnostic `SubnormalMode` values; the `f32` appliers are not for this family.
- No change to the exact-rational arithmetic or its single rounding.

## Required evidence

- A BF16 evaluation under a flushing conformance returns the flushing answer, and the same evaluation under `strict()` returns the preserving one — driven through `ReferenceOperation::evaluate`, not by calling `combine_under` directly, since the link being added is precisely the one `evaluate` was missing.
- Watched failing: revert the link so `evaluate` reaches `preserving()` again, observe the flushing case fail, restore. Capture both outputs.
- The seven-case counterexample population in `crates/tiler-reference/src/bf16/tests.rs` still passes unchanged — those cases pin `combine_under` directly and must not move.
- No pinned identity moves. Confirm rather than assume: the declared facts are untouched, so nothing should.

## Also owed, and easy to miss

[Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) carries an **exception paragraph** recording that the declared-contract comparison rule has one family that cannot follow it, reproduced with `grep -n conformance crates/tiler-reference/src/bf16.rs`. That exception stops being true when this lands, and [`carry-a-bf16-subnormal-realization-the-reference-can-be-told`](carry-a-bf16-subnormal-realization-the-reference-can-be-told.md) states in terms that closing the fork must retire it in the same change or it becomes a stale disclosure of a gap that no longer exists. `contracts/numerics` is required for that file — add the scope.

The `bf16` module header also names the fork as parked and must be updated to record what was decided.

## Closes when

`ReferenceOperation::evaluate` applies the realization it is told, the flushing case is watched failing under the preserving link, the correctness-and-testing exception paragraph is retired, the module header states the resolved decision, and the package's checks pass.

## Worker outcome — 2026-08-07, `agent-bf16-wire`

Base `61414b91`, branch `tkt/wire-the-bf16-reference-to-the-realization-it-is-told`. Scope `contracts/numerics` added for `docs/correctness-and-testing.md`, per the section above.

### The link

`<Bf16BinaryReference as ReferenceOperation>::evaluate` now reads `request.conformance()`'s two format-agnostic `SubnormalMode`s, builds a `Bf16SubnormalRealization` from them, and calls `combine_under`. The `f32` appliers are not reached, the exact-rational arithmetic and its single rounding are untouched, `BF16_FACT_SUBNORMALS` is unchanged, no mixed-width refusal was added, and no `tiler-ir` or `tiler-compiler` file was touched.

### Two superseded internal paths removed, because they became dead code

`Bf16BinaryReference::combine` and `Bf16SubnormalRealization::preserving` were each reached from exactly one non-test site — `combine` from `evaluate`, `preserving` from `combine` — so wiring the link left both `pub(crate)` items with no caller outside `#[cfg(test)]`, which the workspace's denied `dead_code` refuses. They are removed rather than allowed: the preserving reading is now what the *strict* conformance resolves to, so a second spelling of it beside that route would be a value nothing derives. Their four test call sites take `combine_under(..., PRESERVING)`, where `PRESERVING` is a `const` in the test module spelling both modes out. The seven counterexample cases, their four hand-derived answers, and every `combine_under` assertion over them are unchanged; the one assertion that moved is `the_capability_evaluates_under_the_realization_it_is_given`'s "the registered path is the preserving one", which stopped being true — the registered path is now whatever the conformance states, which the new check below asserts.

### Evidence

`bf16::tests::the_registered_capability_evaluates_under_the_conformance_it_is_told` drives all seven counterexamples under all four readings — 28 evaluations, counted — through `ReferenceEvaluator::under`, so the answer comes out of `ReferenceOperation::evaluate` and the registered dispatch rather than out of `combine_under`. Each reading must produce its own hand-derived answer and none of the other three's.

**Watched failing.** Reverting only the link — `evaluate` building `Bf16SubnormalRealization::new(SubnormalMode::Preserve, SubnormalMode::Preserve)` and ignoring the request — makes it fail on every reading but `preserved`, 42 recorded disagreements across all seven cases, each naming the other reading's answer it returned instead. Restored, the same filter passes. Both outputs are in the worker report.

**No identity moved, measured rather than asserted.** `FrozenReferenceRegistry::standard().canonical_identity()` was written to a file at this branch's HEAD and at base `61414b91` by the same throwaway probe, in a detached worktree at the base commit: 1,432,876 bytes both times, `sha256 8e67f70e88e262b87c2417379257d259f960431364a86525fbf18b4044c8d2f1` both times. The probe was deleted and the worktree removed. The provider revision stays at 7 and `crates/tiler-reference/src/standard.rs` is not in the diff.

### Left open, deliberately

No capability checks that the conformance it was handed was *stated about its own format*. `from_realization` discards the subject and has no caller, and every conformance in the tree is `strict()` or a test's `new()`, so the window is unreachable today. It is recorded in the module header and in the second [Correctness and testing](../docs/correctness-and-testing.md#semantic-authority) paragraph, and closes at `from_realization`'s first caller.

Coordinator note: [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md) says that obligation is "Recorded on [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`]". It is not — `grep -n subject` on that file returns two unrelated lines, and the ticket is `done`. Adding it there is outcome expansion on a closed node, so this branch cites the carrier's own decision section instead and leaves the graph repair to the coordinator.

**Correction — 2026-08-10.** The residual above is historical delivery narrative, not live Fact. The subject check and production bridge closed on [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md) and [`route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge`](route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge.md) (both `done`). `evaluate` now obtains conformance via `ReferenceEvaluationRequest::conformance_for(ArithmeticType::Bf16)` rather than the bare `conformance()` getter named at delivery; present-tense claims that the subject is still unchecked, that the window is unreachable, or that `from_realization` has no caller must not be re-read as describing the current tree.

## Outcome — delivered 2026-08-07 at `8f0d4f5d`, merged as `a440c708`

`<Bf16BinaryReference as ReferenceOperation>::evaluate` reads the two format-agnostic `SubnormalMode`s off `request.conformance()`, builds a `Bf16SubnormalRealization`, and calls `combine_under`. The binary32 appliers are not reached, the exact-rational arithmetic and its single rounding are untouched, `BF16_FACT_SUBNORMALS` is unchanged, and no mixed-width refusal was added — all four as the accepted arm requires. Neither `crates/tiler-ir/` nor `crates/tiler-compiler/` was touched.

**Two removals the brief did not anticipate, and they are correct.** Wiring the link left `Bf16BinaryReference::combine` and `Bf16SubnormalRealization::preserving` reachable only from `#[cfg(test)]`, which the workspace's denied `dead_code` refuses in the lib build. The worker removed them rather than reaching for an `#[allow]`, on the ground that the preserving reading is now what the *strict* conformance resolves to, so a second spelling beside that route derives from nothing. The coordinator reviewed the diff and agrees: an `#[allow]` would have preserved a shorthand for a value the wiring now derives, which is the kind of vestigial path this pre-alpha repository removes rather than keeps.

**Evidence.** `the_registered_capability_evaluates_under_the_conformance_it_is_told` drives all seven counterexamples under all four readings — 28 evaluations, counted — through `ReferenceEvaluator::under`, so the answers come out of the registered dispatch rather than from `combine_under` directly, which is the link that was missing. Each reading must produce its own answer *and* none of the other three's. Watched failing with `evaluate` reverted to a hard-coded preserving realization: 42 disagreements across all seven cases, with the preserving reading the only one still passing — the correct half. Restored green.

**No identity moved, measured rather than asserted.** `FrozenReferenceRegistry::standard().canonical_identity()` was written at this branch's HEAD and at base `61414b91` from a detached worktree: 1,432,876 bytes and `sha256 8e67f70e…4c8d2f1` at both. Provider revision stays 7.

**The doc exception is retired.** `docs/correctness-and-testing.md`'s "One registered family cannot yet follow that rule…" paragraph is replaced by "Every registered family now follows that rule…", plus a second paragraph naming what is still unchecked — that no capability verifies the conformance it was handed was stated about its own format. That window is unreachable today because every conformance in the tree is `strict()` or a test's `new()`, and closing it belongs to [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md).

`make full` exit 0 on the branch and again on the merged tree: 2,951 workspace tests, 1,031 release numerical, `tkt lint`, shellcheck.

**Correction — 2026-08-10.** The residual paragraph above is historical Outcome text from this ticket's delivery, not a live residual. The subject check and the BF16 vertical's production bridge closed on [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md) and [`route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge`](route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge.md) (both `done`). Post-wire, `evaluate` reads via `conformance_for(ArithmeticType::Bf16)`; do not re-read the delivery-time bare `request.conformance()` spelling, or the "still unchecked / unreachable / no caller" wording, as present-tense Fact about the current tree.
