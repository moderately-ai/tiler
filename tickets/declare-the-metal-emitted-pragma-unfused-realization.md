---
id: declare-the-metal-emitted-pragma-unfused-realization
title: Declare the Metal emitted-pragma unfused realization
status: in-progress
priority: p2
dependencies: []
related: [probe-the-bf16-contraction-pragma-on-the-metal-runtime-path, design-the-bf16-computation-and-accumulator-contract, declare-metal-numerical-honourability]
scopes: [contracts/numerics, contracts/decisions, research/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, bf16, contraction, apple-targets, contract]
claimed_from: todo
assignee: agent-pragma-realization
lease_expires_at: 1786029868
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

## Graph maintenance

- Filed by [probe-the-bf16-contraction-pragma-on-the-metal-runtime-path](probe-the-bf16-contraction-pragma-on-the-metal-runtime-path.md), which holds `research/apple-targets` and could reach neither the contracts nor `research/numerics`. The probe ticket recorded the measurement where it could and did not absorb the contract consequence.
- Four exclusive scopes is a scheduling cost, and it is deliberate: the four locations state one fact in four voices, and repairing them in separate branches produces four half-statements that each pass their own guard. Split it only if the authorization decision is separated out, which is the one part that is genuinely a different question.
- Nothing depends on this today. It becomes blocking the moment a BF16 contract declaring contraction forbidden is offered to a Metal profile — the same trigger the closed deferral carried, which this ticket now inherits.

## Decided 2026-08-01 — the emitter may write the pragma, as a declared identity-bearing fact

**Tom decided at the live session, witnessed and executed by the coordinator:** the Metal emitter may write `#pragma METAL fp contract(off)`, recorded as its own distinguishable realization row rather than folded into the natively-unfused claim. The translation-unit-scope concern dissolves on the current fact that Tiler emits one kernel per translation unit — that carve-out enters `docs/backends/metal.md` with its derivation, and the one-kernel-per-unit premise is stated as a checked assumption so a future multi-kernel unit re-opens the question rather than silently violating it. The probe's measurement boundary governs the claim: one pragma spelling, file scope, one runtime compiler build, macOS only.
