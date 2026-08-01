---
id: generalize-payload-provenance-beyond-the-apple-shape
title: Generalize payload provenance beyond the Apple shape
status: review
priority: p2
dependencies: []
related: [route-or-refuse-the-device-translation-execution-policy, prototype-a-bounded-scalar-cpu-backend-vertical, state-the-backend-payload-validation-obligation-normatively]
scopes: [implementation/artifact, contracts/artifacts, implementation/build, implementation/runtime, implementation/metal-aot, research/target-profiles, research/extensions, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [artifacts, backend-providers, provenance, consumer-neutrality]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785612420
---
## User-visible outcome

A non-Apple backend can state its payload's provenance in terms that mean something for it, instead of filling Apple-shaped fields with values that have no referent on its target.

## Why this exists

**Fact — the provenance record is Apple-shaped.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):125, under item 14 at `:123`, names it among four vocabulary gaps each measured by the CPU vertical: "`PayloadProvenance` requires Apple-shaped fields with no CPU meaning". The type is at `crates/tiler-artifact/src/program/codec/payload.rs:144`, carried on the payload at `:209`, encoded at `:304`, and decoded at `:474`/`:507`; its Apple-specific companions include `PayloadSdkIdentity`, and every production construction site is a Metal one (`crates/tiler-build/src/metal_payload.rs:339`, `crates/tiler-build/src/metal_assembly.rs:350`).

**Fact — the CPU target is now live, not hypothetical.** [`prototype-a-bounded-scalar-cpu-backend-vertical`](prototype-a-bounded-scalar-cpu-backend-vertical.md) is `done` and carried a second backend end to end, and the CPU vector-lane tier's ADR 0093 is accepted with its three implementation tickets filed. The seam is being pressed by real work rather than reserved against a future one.

**Fact — the condition the CPU vertical set for filing has passed.** `prototype-a-bounded-scalar-cpu-backend-vertical.md:62` withheld a ticket per seam because "filing a ticket per seam before that contract exists would pre-empt its design". The contract exists and ADR 0090 was accepted on 2026-07-31 (`:19`).

**Inference — this is the consumer-neutrality invariant, applied to a field.** AGENTS.md requires the compiler core to stay independent of Metal runtime objects and other consumer-specific types, and requires unsupported cases to reject explicitly rather than approximate. A backend forced to mint an SDK identity it has no notion of is approximating — and an approximated field that enters durable identity is worse than a missing one, because it makes two unlike artifacts comparable.

## Boundaries

- Provenance enters artifact identity. Any reshaping states exactly what moves, what stays put, and whether the identity domain steps; an appended-tag widening that moves no previously encodable payload's bytes is the shape to aim for.
- Generalizing must not weaken what Apple provenance currently proves. A field a Metal payload must carry stays required for a Metal payload; the generalization is about which backend owes which field, not about making all of them optional.
- The normative statement of the payload-validation obligation is [`state-the-backend-payload-validation-obligation-normatively`](state-the-backend-payload-validation-obligation-normatively.md)'s; this ticket shapes the provenance record, not the validation schedule.

## Closes when

An out-of-crate non-Metal backend constructs a payload whose provenance is complete and meaningful for its target, with no field defaulted to an Apple-shaped placeholder; a payload omitting a field its own backend owes is refused by name, observed failing; the identity consequences are stated exactly; and ADR 0090:125's sentence naming this seam is corrected.

## Outcome

**The record now carries a `PayloadPlatform`, and which fields a payload owes follows the shape it declares.** `PayloadProvenance` lost `deployment_major`, `deployment_minor`, and `sdk` and gained `platform: PayloadPlatform`, whose two variants are `Unversioned` — the toolchain resolved against no versioned SDK, owes no platform field, and may state none — and `VersionedSdk { deployment_major, deployment_minor, sdk }`, which owes all four. Every payload additionally owes a toolchain, a target, a family, a language, and a role and a version per listed tool component. An owed field left empty is `ArtifactBuildError::IncompletePayloadProvenance { field }`, raised in `PayloadMetadata::identity` — the single derivation both payload push paths reach — and re-proven on decode; a tagged encoding that fills a platform position anyway is `ArtifactCodecError::PlatformFieldWithoutPlatform { field }`.

**Nothing became optional.** `crates/tiler-build/src/metal_assembly.rs` declares `VersionedSdk` and owes exactly what it owed before, now to a check rather than to a convention. `crates/tiler-build/tests/custom_backend/backend.rs` — an out-of-crate, non-Metal backend that nothing in `crates/` knows about — declares `Unversioned` and is complete without an SDK; `a_non_metal_backend_states_no_sdk_and_still_owes_the_rest` asserts both halves, including that dropping any of its six owed fields is refused by that field's name.

**Identity: an appended-tag widening that moved no previously encodable payload's bytes.** The versioned-SDK shape keeps the untagged encoding exactly — the deployment minimum in its two `u16` positions ahead of the component list, the three SDK runs after it — and the unversioned shape writes those positions as two zeroes and three empty runs, then appends one tag byte after the obligation list. Injectivity is argued per tag: the untagged grammar is self-delimiting, so the number of bytes one record occupies is a function of its own bytes; a versioned encoding is that grammar followed by nothing and an unversioned one is the same grammar followed by exactly one byte, so the two classes are disjoint whatever their other fields hold. Within the untagged class the map is the previous, unchanged one; within the tagged class the platform positions are constants and the rest is that same injective grammar. No tag is admitted for the versioned shape, and a tagged encoding that states a platform field is refused rather than normalized, because one record with two spellings is two payload identities. The set of encodable versioned records *shrank* — an empty SDK name is now a refusal — and a refusal is not a move.

**Empirically:** `crates/tiler-build/src/metal_plan.rs`'s pinned artifact identity and cache subject did not move. The whole workspace is green with both pins unchanged from the base commit, which is the strongest available evidence for the appends-only claim, because the Metal artifact identity folds the payload digest. Note for integration: `main` rebaselined those same two pins after `cbec2d4` for the unrelated `tiler.schedule.v4` step, so the merged tree carries `main`'s values and this change does not move them either — recompute rather than pick a side.

**Measurement boundary.** Every claim above is about this tree at `cbec2d4` plus this branch, on macOS arm64 under the pinned toolchain. The Metal-pin result is evidence about *that* Metal fixture, not a proof over all payloads; the injectivity claim is general but is an argument about the grammar rather than a measurement.

## Graph maintenance

**Scopes added, with the derivation.** `implementation/build`, `implementation/runtime`, `implementation/metal-aot`, `research/target-profiles`, `research/extensions`, and `contracts/decisions` were added because a public type change forces an edit at every construction *and read* site of the field it replaced, and because "Closes when" names the ADR. Those sites are `crates/tiler-build/{src/metal_assembly.rs, src/metal_payload.rs, examples/identity_join_producer.rs, tests/custom_backend/{backend.rs, main.rs}}`, `crates/tiler-runtime/tests/adapter_route/fixture.rs`, `prototypes/serial-sum-run/src/proof.rs`, `prototypes/serial-sum-compile/src/main.rs`, and `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs`. `docs/backends/cpu.md` was already inside `contracts/artifacts`. No live ticket held any added scope except `contracts/decisions`, and the ADR-0090 edit was verified file-level disjoint against that holder's *landed* work: `git diff --name-only cbec2d4..main -- 'docs/decisions/0090*'` is empty, while the same range over the whole tree is long and includes 0012, 0014, 0020, 0022, 0034, 0051, and 0086 — that ticket's stated worklist. The two `prototypes/serial-sum-compile` sites were found by the compiler rather than by grep, because they read the replaced fields instead of naming a type; a grep-only survey would have missed them.

**Filed:** [`restore-the-scalar-cpu-vertical-spike-against-the-current-crates`](restore-the-scalar-cpu-vertical-spike-against-the-current-crates.md). The CPU vertical spike's provenance construction was updated here, but the spike does not compile and did not compile at `cbec2d4` either — ten pre-existing errors from `BackendEntryRef::payload` becoming `payloads` and `DecodedProgram::decode` gaining a delivery-position argument, verified present at the base by restoring the base source and re-checking against this branch's crates. It was therefore not re-run, and its result fixture was **not** hand-edited with computed numbers; finding 7 in its README records the closure and disclaims the stale byte counts.
