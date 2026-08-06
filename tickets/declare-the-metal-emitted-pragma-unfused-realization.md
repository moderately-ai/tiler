---
id: declare-the-metal-emitted-pragma-unfused-realization
title: Declare the Metal emitted-pragma unfused realization
status: done
priority: p2
dependencies: []
related: [probe-the-bf16-contraction-pragma-on-the-metal-runtime-path, design-the-bf16-computation-and-accumulator-contract, declare-metal-numerical-honourability]
scopes: [contracts/numerics, contracts/decisions, research/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, bf16, contraction, apple-targets, contract]
---
## User-visible outcome

Four documents stop saying that the pragma's runtime effect is unmeasured, because it has been measured, and the contract states what a Metal profile may now declare about contraction and by what means — so that a consumer reading an unfused BF16 guarantee can tell an emitted-pragma realization from a natively unfused one.

## What was measured, and where the evidence lives

**Measurement — finding 33 of [the Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md), 2026-08-02.** `#pragma METAL fp contract(off)` inserted at file scope — one 31-byte line, the sources otherwise unchanged — survives `newLibraryWithSource:options:` to runtime compiler `metalfe-32023.921` and unfuses the written multiply/add pair in **all twelve** cells of `{bf16, f32, f16} × {Relaxed, Fast} × {Default, Size}`, with the unperturbed neighbour still returning finding 30's fused value in every one of those cells and every case carrying an `executed` witness. The harness, the exact changed bytes, the retained rows, and the measurement boundary are in [`spikes/apple-targets/contraction-pragma-runtime-probe/README.md`](../spikes/apple-targets/contraction-pragma-runtime-probe/README.md).

**Inference.** An unfused BF16 contract is therefore honourable on the measured Apple runtime row *by emitting different source*, which is [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md)'s `SupportedWithExactEmulation` shape — the one outcome that changes the emitted program rather than the verdict — and not by any `MTLCompileOptions` setting, which still has no contraction property. The `disproved` predicate the contracts currently record is the right verdict for the *unperturbed* emission and the wrong one for an emission that carries the pragma.

## The stale sentences, each with its exact location

Every one of these was written while the answer was unmeasured and is now false or incomplete. Each is a claim a later reader acts on, so none may be left standing.

- `docs/numerical-semantics.md` (`contracts/numerics`), the `Measurement — an unfused BF16 guarantee is currently unhonourable on the measured Apple runtime path` paragraph: its final sentence says the pragma's runtime effect is unmeasured. The refusal it states is correct for an emission without the pragma and must be restated against that condition rather than deleted.
- `docs/research/numerics/bf16-computation-accumulator-and-conversion.md` (`research/numerics`), twice: the contraction-half paragraph asserts the same "unmeasured", and the deferral list carries this question with its closing evidence and trigger. The deferral is now closed by exactly the evidence it named, so it moves to a resolved measurement rather than being deleted.
- `docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md` (`contracts/decisions`), the four-deferrals paragraph, which adopts this deferral by reference and names the probe ticket as its owner.
- `docs/backends/metal.md` (`contracts/artifacts`), the source-emission rules ("a translation-unit-wide flag is legal only when it stays within every affected operation contract") and the `Numerical compiler realization` section, which today enumerate offline flag spellings and no source-level control at all.

## The decision inside this, which is Tom's rather than the sweep's

Whether Tiler's Metal emitter is **authorized to write a contraction pragma into emitted source** is a contract change, not a stale-sentence repair, and it carries at least three consequences the sweep must present rather than settle:

- A file-scope pragma is translation-unit-wide, which is precisely the shape `docs/backends/metal.md` already restricts: it is legal only when it stays within every affected operation contract, so a region mixing an unfused-contract operation with one that permits contraction cannot be honoured by one file-scope pragma. Whether the answer is block-scope emission, region splitting, or refusal is a design question with a worked example owed to it.
- The realization must be **recorded distinguishably**. An emitted-pragma unfused result and a natively unfused one are the same bits and different provenance; ADR 0076's whole reason for keeping `SupportedWithExactEmulation` separate is that emulation changes the emitted program, so an artifact that does not say which one it got has lost the fact the outcome exists to carry.
- The measured property belongs to `metalfe-32023.921` and to one pragma spelling in one placement. Finding 8's reason to keep re-measuring applies: a target fact resting on a source-level control the runtime compiler honours today needs the compiler build in its provenance, exactly as the offline flags do.

## Closes when

Each of the four locations above states the measured answer with its boundary, the ADR 0091 deferral is recorded as resolved by finding 33, and either the emitter authorization is decided by Tom and written into `docs/backends/metal.md` with the distinguishable-realization rule, or it is filed as its own explicitly deferred ticket with the trigger that reopens it. A sweep that repairs the "unmeasured" sentences and leaves the authorization question unstated has done half the step.

## Outcome — 2026-08-06

The measurement was verified before anything was written from it. Finding 33 of `docs/research/apple-targets/numerical-behaviour.md` (lines 504–530) and `spikes/apple-targets/contraction-pragma-runtime-probe/README.md` were both read in full: twelve cells, `summary.control_fused` 12, `summary.pragma_unfused` 12, `summary.pragma_fused` 0, 24 executed witnesses, spelling `#pragma METAL fp contract(off)` at file scope, runtime compiler `metalfe-32023.921` (offline driver `metalfe-32023.883`). The record's per-width table and the probe's candidate-derivation table agree in both directions at all three widths. Neither file was edited — `research/apple-targets` is not this ticket's scope.

Five sites landed, one more than the ticket enumerated:

- `docs/numerical-semantics.md` — the refusal paragraph is conditioned on an emission without the pragma rather than deleted, and a second Measurement paragraph states finding 33's result, its `SupportedWithExactEmulation` composition, and the full boundary.
- `docs/research/numerics/bf16-computation-accumulator-and-conversion.md` — the contraction-half paragraph's "unmeasured" tail is replaced by the measured result with its inference and boundary; the deferral entry becomes a resolved measurement naming what closed it, the answer, the boundary, and what reopens it.
- `docs/decisions/0091-…` — the four-deferrals paragraph records the first deferral resolved; `decision_status` is untouched. **Beyond the ticket's list:** the third Consequences bullet ("currently unhonourable") is now false as written, so a labelled post-acceptance note beneath the list conditions it. The accepted bullet itself is left verbatim, because it is the text the acceptance covered.
- `docs/backends/metal.md` — a new `### The emitted contraction pragma is a declared realization, not an inherited default` subsection, and the MSL-emission bullet's translation-unit-wide rule extended to pragmas with a pointer to it.

**The decision's premise is refuted by the source, and this is the item for Tom.** The 2026-08-01 decision below dissolves the translation-unit-scope concern on "the current fact that Tiler emits one kernel per translation unit". That is not a current fact: `tiler_metal::emit::emit_translation_unit(kernels: &[&VerifiedKernel], …)` takes a slice and emits one entry point per kernel, and `docs/backends/metal.md`'s own "Expansion-time offline compilation" section says one invocation aggregates all entry points needed by its own one- or **multi-kernel** plans. Writing that premise into an accepted contract as a checked assumption would have planted a false fact, so the carve-out was derived differently and the refutation recorded in place. The replacement derivation is stronger and reaches the same authorization: contraction is a *permission* under ADR 0015, not a requirement, so a translation-unit-wide `contract(off)` cannot violate a kernel whose contract permits contraction — an unexercised permission is not a violation — and kernel count is therefore not load-bearing. The one construct that could break is ADR 0015's required `Fma`; `tiler_ir::kernel::BinaryOp` has no fused variant and `binary_realization` refuses an unrecognized op, so no such kernel is emittable. That is the checked assumption, with its one-line recheck and its reopen clause in the contract. The authorization itself is unaffected; Tom may want to know the premise he cited was wrong.

## Graph maintenance

- Filed by [probe-the-bf16-contraction-pragma-on-the-metal-runtime-path](probe-the-bf16-contraction-pragma-on-the-metal-runtime-path.md), which holds `research/apple-targets` and could reach neither the contracts nor `research/numerics`. The probe ticket recorded the measurement where it could and did not absorb the contract consequence.
- The authorization is recorded and unimplemented: no emitter writes the pragma, and `realization_requirements` still discharges a `Forbidden` contraction as a compiler-flag requirement. That implementation is filed at `deferred` with its activation trigger and trigger-check log as [emit-the-contraction-pragma-as-a-declared-metal-realization](emit-the-contraction-pragma-as-a-declared-metal-realization.md), because the offline `-ffp-contract=off` selection discharges the requirement on every Tiler path today and the board must not offer non-work.
- **Report-back, not editable from here.** `docs/decisions/README.md`, `docs/roadmap.md`, and `docs/status.md` are `contracts/navigation`, held live. Nothing found in this sweep requires a catalog line to move: ADR 0091's `decision_status` and catalog group are unchanged, no ADR was added or superseded, and no support-matrix rung criterion was satisfied — the `Cast and convert` row stays at R2 and no numerical row moved, because this landing states a realization and registers nothing.
- Four exclusive scopes is a scheduling cost, and it is deliberate: the four locations state one fact in four voices, and repairing them in separate branches produces four half-statements that each pass their own guard. Split it only if the authorization decision is separated out, which is the one part that is genuinely a different question.
- Nothing depends on this today. It becomes blocking the moment a BF16 contract declaring contraction forbidden is offered to a Metal profile — the same trigger the closed deferral carried, which this ticket now inherits.

## Decided 2026-08-01 — the emitter may write the pragma, as a declared identity-bearing fact

**Tom decided at the live session, witnessed and executed by the coordinator:** the Metal emitter may write `#pragma METAL fp contract(off)`, recorded as its own distinguishable realization row rather than folded into the natively-unfused claim. The translation-unit-scope concern dissolves on the current fact that Tiler emits one kernel per translation unit — that carve-out enters `docs/backends/metal.md` with its derivation, and the one-kernel-per-unit premise is stated as a checked assumption so a future multi-kernel unit re-opens the question rather than silently violating it. The probe's measurement boundary governs the claim: one pragma spelling, file scope, one runtime compiler build, macOS only.
