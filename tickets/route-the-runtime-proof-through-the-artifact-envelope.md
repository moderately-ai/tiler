---
id: route-the-runtime-proof-through-the-artifact-envelope
title: Route the runtime proof through the artifact envelope
status: todo
priority: p0
dependencies: [prototype-runtime-artifact-validation]
related: [prototype-metal-runtime-proof, prototype-metal-aot-slice, assemble-the-metal-payload-from-emission-and-compilation]
scopes: [implementation/runtime, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifact]
---
`prototypes/serial-sum-run` executes a real tensor program on an Apple M4 Max and matches `ReferenceEvaluator` bit for bit. **It does so by bypassing the artifact envelope entirely**, and this ticket exists so that shortcut is tracked rather than mistaken for delivery.

## What the spike proves, exactly

At `a56bff8`, one process carries `sum((input * 1.0) + 0.0)` over `[4, 3]` from a `SemanticProgram` through `compile_governed`, MSL emission, `xcrun` compilation to a 3,843-byte `metallib`, `MTLDevice` load, dispatch, and an exact bit comparison. Output `[40c00000, 3f800000, 7fc00000, 7f800000]` on both sides: 6.0; 1.0 from a row containing negative zero and the least subnormal; the canonical NaN from a *non-canonical* `0x7fc01234` input; and propagated positive infinity.

That is evidence about the **compiler and the emitter**: the machine code computes what the semantic program means, and the declared numerical contract holds on real hardware for values where it could have failed.

## What it does not prove, and this is the gap

The `metallib` goes from `CompiledArtifact` straight into `Device::new_library_with_data` as in-memory bytes. Nothing is packaged, encoded, decoded, or validated. Specifically **none** of these ran: canonical envelope encoding, the framing header, manifest and section digests, required-feature negotiation, re-proven model obligations, artifact identity re-derivation, declared target compatibility classification, prepared-entry feasibility, launch feasibility, or one-way routing commit. The runtime contract's monotonic validation stages are entirely unexercised, and `docs/artifact-abi.md` is explicit that parse success never implies executable compatibility — here there was no parse at all.

So the delivery mechanism is unproven end to end even though the thing it would deliver is proven correct.

## The work

Assemble the emission and compilation into a carried backend payload through `push_carried_payload`, encode the envelope, hand the runtime **bytes** rather than a `CompiledArtifact`, and have it decode, validate, classify compatibility, commit routing, and only then load and dispatch. The bit comparison must still pass, and it must pass for the same reason — a difference introduced by the envelope round trip is a defect in the envelope, not a numerical result.

The payload carrier's constructors are `pub(crate)` in `tiler-artifact`; promoting them is ADR 0075 review and is a prerequisite this ticket does not own.

## Do not

Do not delete or weaken the direct-dispatch spike when the envelope path lands. Keeping both is what distinguishes an envelope defect from a compiler defect the next time the bits disagree: if the direct path still matches the reference and the envelope path does not, the envelope is at fault, and that is a diagnostic worth retaining.

## Correction — the stated prerequisite is satisfied, and the producer half is built

Recorded from `carry-the-metal-payload-in-an-artifact-envelope`, so it is not re-derived.

**The sentence "the payload carrier's constructors are `pub(crate)` in `tiler-artifact`" is stale.** They were promoted on Tom's review on 2026-07-25, together with the codec's *capability*: `VerifiedArtifactProgram::encode`, `decode_artifact`, `DecodedArtifact` with `identity`/`features`/`routing`/`payloads`/`sections`/`variant_count`/`re_encode`, `SectionView`, `SectionPurpose`, and `ArtifactCodecFailure` are all `pub` (`crates/tiler-artifact/src/program/codec/view.rs`, re-exported at `program/mod.rs:313-317`). The envelope, encoder, decoder, and section types themselves stay `pub(crate)`; the view exposes accessors rather than fields.

**The producing half of this ticket's work already exists.** `prototypes/serial-sum-compile/src/bundle.rs` assembles a real compilation and a real `metallib` into an envelope and proves the round trip — encode, decode, byte-identical re-encode, section purposes, descriptor digest, and the derived feature set. What remains here is genuinely the consuming half: hand the runtime **bytes**, and have it decode, validate, classify compatibility, and commit routing before it loads anything.

**A measured constraint this ticket must plan around.** The envelope's reader refuses a multi-stage variant. This profile's neutral program section carries a program's canonical identity and not its dependency graph, so the projector derives `tiler.artifact.feature.multi-stage-program` and `SUPPORTED_FEATURES` deliberately omits it. The fused single-stage plan — which is what the selection policy chooses for the proof program, and what the runtime proof already dispatches — round-trips. Routing the proof through the envelope therefore works today for the plan it actually runs, and the materialized reference alternative cannot travel until `carry-reconstructable-kernel-programs-in-the-neutral-envelope` closes. Whether the runtime proof needs the reference alternative *in the envelope* or only in-process is a question this ticket should answer explicitly rather than discover.
