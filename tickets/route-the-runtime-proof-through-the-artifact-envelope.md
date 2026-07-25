---
id: route-the-runtime-proof-through-the-artifact-envelope
title: Route the runtime proof through the artifact envelope
status: done
priority: p0
dependencies: [prototype-runtime-artifact-validation, expose-the-dispatch-record-on-a-decoded-artifact]
related: [prototype-metal-runtime-proof, prototype-metal-aot-slice, assemble-the-metal-payload-from-emission-and-compilation]
scopes: [implementation/runtime, implementation/metal-aot, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
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

## Blocked 2026-07-25 — the consuming half is built; the assembler is unreachable

Attempted from `implementation/runtime` and `implementation/artifact` after `admit-the-device-free-runtime-validation-crate` and `prototype-runtime-artifact-validation` landed. The loader this ticket needs now exists and is not the obstacle.

**Fact — every input is in the process except one.** `crates/tiler-runtime` provides `DecodedProgram::decode`, `preflight(&ExecutionEnvironment, &CanonicalArtifactProgramIdentity)`, and the infallible `Preflight::commit`, whose `RoutedDispatch::object()` returns the exact object bytes the envelope carries — byte for byte what the producer handed to `push_carried_payload`, because the decoder strips the framing and the section body *is* the object (`crates/tiler-artifact/src/program/codec/model.rs:758-769`, `codec/decode.rs:203-207`). `prototypes/serial-sum-run` already holds everything an `ExecutionEnvironment` needs: `compilation.target_profile_key()` and `target_profile_descriptor()` for the `TargetProfileRef`, and `"tiler.metal"`/`"metallib"` for the backend and representation. The single missing value is the `VerifiedArtifactProgram` — needed both to `encode()` the envelope and to supply `preflight`'s `expected` identity.

**Fact — only one assembler exists and it is private to a binary crate.** `grep -rn "ArtifactProgramBuilder" crates prototypes --include "*.rs"` returns exactly one non-`tiler-artifact` user, `prototypes/serial-sum-compile/src/bundle.rs`. That package's `Cargo.toml` declares `[[bin]]` and no `[lib]`, and `src/main.rs:23-25` declares `mod bundle; mod payload; mod target;` as private modules. No other package can name them.

**Split into [`share-the-serial-sum-artifact-assembler`](share-the-serial-sum-artifact-assembler.md)**, which carries the three ways of fixing that, their costs, and a recommendation. In short: a `[lib]` target on `tiler-prototype-compile` is the smallest change that leaves one assembler, but it creates a public namespace on a package that has none and requires relaxing `scripts/check_workspace.py`'s `expected_member_manifest` and `expected_targets`, both of which hard-code one `[[bin]]` and no `[lib]` for a `tiler-prototype-*` package. Promoting the assembler to its own library crate is a new crate admission and therefore Tom's. Duplicating roughly 300-340 lines of identity-bearing assembly into the runner is rejected outright: two independently maintained descriptions of one compilation is the exact defect this ticket exists to remove.

**Fact — a cold two-process handoff does not avoid the assembler, it costs two other things.** Writing the envelope from the producer and reading it in the runner leaves the runner with no `expected` identity to bind against except one re-derived from the same bytes, which is vacuous; and it still cannot obtain the entry symbol, because `decode_metadata` is `pub(crate)` (`crates/tiler-artifact/src/program/codec/payload.rs:292`) and no public accessor parses the payload-metadata section. Binding by identity is available only to a consumer holding the program it compiled, which is the single-process shape this ticket describes.

## The question this ticket asked is answered: only in-process

"Whether the runtime proof needs the reference alternative *in the envelope* or only in-process." **Only in-process, and the envelope must not be widened for it.**

The reference alternative is the materialized plan. The proof's independent oracle is `ReferenceEvaluator`, which evaluates the *`SemanticProgram`* directly — `prototypes/serial-sum-run/src/main.rs:127-158` — and the semantic program is not what an envelope carries at all; the envelope carries one packaged plan's kernel-program identity. So the bit comparison never needed the materialized alternative to travel, and the multi-stage refusal recorded on `carry-reconstructable-kernel-programs-in-the-neutral-envelope` does not gate this ticket. The envelope needs to carry exactly the plan that is dispatched, which is the fused single-stage one, and that round-trips today.

## What must still be true when this lands

Unchanged by the block, and restated because the enabling ticket must not quietly relax it: the device must load the bytes `RoutedDispatch::object()` returns and nothing the process held before, the bit comparison must still pass, and the direct-dispatch path must be retained beside the envelope path as the diagnostic that separates an envelope defect from a compiler defect.

## Outcome

**The envelope path dispatches on hardware; one governed bound keeps it off the proof's own program.** Landed on `tkt/route-the-runtime-proof-through-the-artifact-envelope`, over `tkt/route-the-runtime-loader-through-the-dispatch-record`.

### The blocking dependency was refuted, not satisfied

`share-the-serial-sum-artifact-assembler` is **closed as obsolete**. Its own "Fact — a cold handoff does not avoid this" made two claims and both are retracted on that ticket with the evidence. In short: `decode_metadata` being `pub(crate)` stopped mattering when `decode_artifact` began parsing it eagerly and publishing the result through `DecodedEntry::backend_symbol` and `transport_slots`; and `preflight` documents a second source for the expected identity — one *recorded* beside cached bytes — which that ticket did not consider. The runner needs no assembler and no producer module. The producer writes a file; the runner reads it. Nothing else crosses.

The rejected option (a), a `[lib]` on `tiler-prototype-compile`, would also have left a `CompiledArtifact` in the runner's process that could be loaded instead of the envelope's. **The cheaper option was the one that kept the bypass reachable.**

### What supplies the expected identity

A sidecar. `--out <path>` writes `<path>` and `<path>.identity`, the latter from `VerifiedArtifactProgram::canonical_identity()` — derived from the program the producer *built*, not re-read from the encoding. That is `preflight`'s "recorded when it cached these bytes" case, and it is not vacuous: **measured**, flipping one byte of the sidecar yields `runtime.program-mismatch: expected an artifact of 12655 identity bytes, loaded one of 12655, and they differ`. It is worth exactly what the sidecar is worth and resists no adversary who rewrites both files; nothing unsigned could, and that is stated in the source rather than glossed.

`CanonicalArtifactProgramIdentity` turned out to have **no public constructor**, so a cold consumer could not state one at all. `preflight` now takes `expected: &[u8]`. Split to [`state-an-expected-artifact-identity-from-recorded-bytes`](state-an-expected-artifact-identity-from-recorded-bytes.md).

### Measurement — the run

Apple M4 Max, this checkout. `cargo run -p tiler-prototype-compile -- --out <p>` then `cargo run -p tiler-prototype-run -- --artifact <p>`:

```text
device: Apple M4 Max
compiled 3843 bytes of metallib
artifact: …/serial-sum.tiler (32329 bytes), expected identity 12655 bytes
decoded: 1 variant(s), required features ["tiler.artifact.feature.embedded-payload-code"]
the artifact declares a 4 by 1 input
routed: symbol "tiler_kernel_cca3c1e98be4e752", 3667 object byte(s), 4 thread(s) in groups of 1
  abi slot 0 -> transport 0, 16 byte(s), ProgramInput(InputKey("input"))
  abi slot 1 -> transport 1, 16 byte(s), ProgramOutput([OutputKey("result")])
direct    4x3: [40c00000, 3f800000, 7fc00000, 7f800000] against [40c00000, 3f800000, 7fc00000, 7f800000]
envelope  4x1: [3f800000, 00000000, 7fc00000, 7f800000] against [3f800000, 00000000, 7fc00000, 7f800000]
bit-for-bit agreement: direct on 4 element(s), envelope on 4 element(s)
```

**The 3,667 against 3,843 is the proof the bypass is gone on that path.** They are different byte counts because they are different objects from different processes: the envelope path loaded what the producer packaged, and the direct path loaded what this process compiled. A runner still bypassing the envelope would have shown one number twice. The symbol, both transport indices, both byte ranges, and the launch geometry are all read from the decoded record.

Every refusal was exercised rather than assumed: a missing artifact, a missing argument, a flipped sidecar byte (`program-mismatch`), and a flipped envelope byte (`artifact.integrity: SectionDigestMismatch { section: 0 }`). All four exit non-zero.

### The limit that remains, and it is not the one this ticket predicted

The ticket planned around the multi-stage refusal. That was correct and irrelevant — the fused plan round-trips as expected. The actual blocker is different and was found by running: **the producer cannot package the program the proof dispatches.** A `BackendEntryKey` is bounded at `MAX_OPAQUE_IDENTITY_BYTES` = 1,024 and a non-degenerate serial sum's canonical kernel identity measures 1,113 bytes — 728 at one contributor, then flat at 1,113 for two, three, four, and eight. It is the reduction *structure* that crosses the bound. Split to [`bound-the-backend-entry-key-by-the-identity-it-carries`](bound-the-backend-entry-key-by-the-identity-it-carries.md) with the full table and three options; it is not applied here because every option changes a governed contract and one of them changes what a backend entry key means.

**What was refused rather than done.** Reducing the runner's program to the degenerate shape so the envelope could carry it would have silently weakened a landed numerical proof — the three-contributor row is what makes serial ordering, subnormal flushing and NaN canonicalization observable together. Raising the bound would have guessed a governed value. Digesting the identity in the producer would have minted an identity for a subject the artifact layer says it is not the authority for. All three are the shortcut this ticket exists to avoid.

**What was done instead.** The runner's envelope path takes its shape from whatever the artifact declares rather than fixing one, so the delivery mechanism is proven on hardware today and the numerical claim stays on its own three-column program, each compared against the oracle's evaluation of the program that path ran. When the bound ticket closes, the producer's `COLUMNS` returns to 3 and the two paths run one program **with no change to the runner** — the convergence is already in the code, not a follow-up edit.

### Honest scope of the claim

The delivery mechanism — encode, decode, digest and identity validation, feature negotiation, target classification of both the variant's and the payload's declared profiles, one-way routing commit, dispatch from carried bytes — is **proven end to end on hardware**. It is proven for the single-contributor program, which is the only one the envelope currently admits. The three-contributor program is still delivered by the direct path alone, so for *that* program the envelope is not yet load-bearing. Those are two different claims and the ticket is done against the first, with the second tracked.

### What must still be true, restated and held

The device loads the bytes `RoutedDispatch::object()` returns and nothing the process held before — held. The bit comparison passes — held, on both paths. The direct path is retained beside the envelope path as the diagnostic — held, and its program is unchanged.
