---
id: retire-the-removed-loader-compatibility-refusals-outside-the-runtime-scope
title: Retire the loader's removed compatibility refusals from the spike and the recorded measurements
status: done
priority: p2
dependencies: []
related: [select-executable-variants-across-registered-backend-families]
scopes: [research/target-profiles, research/extensions, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, backend-providers, documentation, spikes]
---
## User-visible outcome

The scalar-CPU-vertical spike compiles and runs again, and every document citing the loader's retired compatibility refusals names what the loader reports now, so a reader following a recorded measurement to the code finds it.

## Context, and why this is a separate ticket

`select-executable-variants-across-registered-backend-families` inverted the loader's selection order: host-relative ineligibility is now a filter applied before any applicability guard rather than a terminal mismatch after one. `LoadRejection::UnexecutablePayload`, `LoadRejection::IncompatibleTarget`, and `TargetDeclaration` were the terminal spelling of that predicate for a single variant and are removed; `LoadRejection::NoEligibleVariant`, `LoadRejection::NoApplicableVariant { filtered }`, `FilteredVariant`, and `VariantIneligibility` replace them. Every consumer inside `implementation/runtime` was updated in that ticket's commit. These are the ones outside its declared scopes, left rather than absorbed silently.

## Implementation keys

- `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs` matches `LoadRejection::IncompatibleTarget { declaration: TargetDeclaration::Variant, .. }` at two sites and `LoadRejection::UnexecutablePayload { .. }` at two more. The spike is its own workspace and no `make` target reaches it, so nothing in the gate catches this — it is a hand-run compile failure waiting for the next reader. Verified by reading the four sites, not by grep alone.
- `spikes/target-profiles/scalar-cpu-vertical/README.md` records that run's fail-closed probes by refusal class, naming `IncompatibleTarget` / `DescriptorMismatch`, `IncompatibleTarget` / `ProfileKeyMismatch`, and `runtime.unexecutable-payload` twice. That is a **Measurement** tied to an exact environment: re-running the spike is what makes a rewritten line true, so update the source, re-run, and record the new output rather than editing the recorded text in place.
- `docs/research/extensions/backend-provider-composition.md` and `docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md` both cite the same CPU-vertical measurement — "a host consuming `tiler.cpu.scalar-image-v2` was refused as `runtime.unexecutable-payload`" — as evidence for the normative backend obligation in ADR 0090 item 8. The obligation is unchanged and the evidence still holds; only the class name the loader reports for it moved. Correct the citation without weakening or restating the finding, and note that ADR 0090 is accepted, so this is a factual correction to a supporting citation rather than a change of decision.
- Do not rewrite the recorded outcomes of already-closed tickets under `tickets/`. Those are history, and `route-the-runtime-loader-through-the-dispatch-record.md` describing the vocabulary it introduced is still an accurate account of what that ticket did.

## Closes when

The spike builds and its fail-closed probes run by hand from its own directory, its README records the re-run output, both citing documents name the class the loader reports now, and `make full` stays green.

## Graph maintenance

- Scoped to `research/target-profiles` because that is where the spike lives; the two document edits need `research/extensions` and `contracts/decisions` added before the branch touches them.
- Not a blocker on any runtime work: the loader change is complete and gated, and this is drift in evidence that outlived its producer.

## Outcome

**Done.** The four call sites, the README-recorded classes, and both citations name what the loader reports now, and the spike runs from its own directory again.

**Base correction before any edit.** The dispatch named `e2da98f` and the worktree had been created at `3d7b31a`, which is `e2da98f`'s parent — and `e2da98f` is the commit that recorded this ticket's own claim, so working from the worktree's checkout would have committed a ticket file with no `assignee` on it and conflicted with the claim on `main`. The branch had no commits and a clean tree, so it was fast-forwarded to the named base and everything below is against `e2da98f`. Reported rather than absorbed: a worktree one commit behind its named base is a dispatch error, and the claim record is exactly the kind of thing that interval hides.

**Scopes added 2026-08-01: `research/extensions`, `contracts/decisions`, and shared `project/tickets`.** The first two are exactly what "Graph maintenance" above said the two document edits would need, and the shared one covers this file. No other open ticket declares either exclusive scope, so the additions create no contention. `research/target-profiles` covers everything else the branch touched.

**The four sites, and what each now asserts.** `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs`, all four inside `probe_fail_closed`:

| Perturbation | before | after |
| --- | --- | --- |
| another profile descriptor | `IncompatibleTarget { declaration: TargetDeclaration::Variant, classification: DescriptorMismatch }` | `NoEligibleVariant` filtering variant 0 as `AssessedProfile { classification: DescriptorMismatch }` |
| the Metal profile family | `IncompatibleTarget { declaration: TargetDeclaration::Variant, classification: ProfileKeyMismatch }` | `NoEligibleVariant` filtering variant 0 as `AssessedProfile { classification: ProfileKeyMismatch }` |
| a host executing `metallib` | `UnexecutablePayload { .. }` | `NoEligibleVariant` filtering variant 0 as `UnsupportedRepresentation` with `host_backend` `tiler.metal` and `host_representation` `metallib` |
| a host consuming `tiler.cpu.scalar-image-v2` | `UnexecutablePayload { .. }` | `NoEligibleVariant` filtering variant 0 as `UnsupportedRepresentation` with `host_backend` `tiler.cpu.scalar` and `host_representation` `tiler.cpu.scalar-image-v2` |

The shared helper `sole_ineligibility` reads `NoEligibleVariant { packaged: 1, filtered }` down to the single `FilteredVariant { variant: 0, reason }` this one-variant artifact must report, and each probe pins the reason. **The last two probes are pinned beyond their class, which is stronger than what they replaced rather than equal to it.** Both matched the one class `UnexecutablePayload { .. }` before and both would match the one class `UnsupportedRepresentation` now, so neither vocabulary separated them and each probe would have passed on the other's perturbation. The class move is what made that visible, so the host pair is asserted for each: `tiler.metal`/`metallib` for the Metal-host probe, `tiler.cpu.scalar`/`tiler.cpu.scalar-image-v2` for the representation one. The first two probes are separated by their classification exactly as they were before.

**Re-run evidence**, `CARGO_TARGET_DIR=./target cargo run` from the spike's own directory at base `e2da98f`, macOS arm64, the pinned nightly. Exit 0, twelve `f32` elements bit-identical to `tiler-reference`, all four envelope probes refusing as `runtime.no-eligible-variant` with the reasons tabled above. Two recorded quantities moved against the previous run at `63f9259` — selected plan `program-alternative:5ef3467e50acb6f7` → `986779d4106ea633`, reference registry identity 438,805 → 446,768 bytes — and the fixture was re-recorded with `results/2026-07-31-macos-arm64.json` as its argument. Envelope (20,953), artifact identity (9,753), payload (265), profile descriptor (797), element count, and zero deferred predicates all held. The README tables both moves with the superseded values retained, per the convention `restore-the-two-path-dependent-spikes-to-a-running-state` set, and states the boundary the earlier table did not: five of those seven are byte *counts*, so a changed identity of unchanged length would read the same.

**Every new check observed failing.** Three deliberate perturbations, each made, run, and reverted: swapping the descriptor probe's expected classification to `ProfileKeyMismatch` (exit 1, "did not fail closed on another profile descriptor"); swapping the Metal-host probe's expected `host_backend` to this backend's own key (exit 1, "did not fail closed on a Metal host" — the one that proves the two `UnsupportedRepresentation` probes are not interchangeable); and requiring `packaged: 2` in the helper (exit 1 at the first probe). The `CanonicalizeF32Nan` comparison perturbation was also re-run at this base, not only at `63f9259`, because the reference registry identity moved across that interval and the oracle is therefore not the one the earlier re-run used: exit 1, one differing element, `0x7fc01234` against the required `0x7fc00000`.

**Both citations corrected in place, as dated corrections rather than rewrites.** ADR 0090 item 8 and the composition record's "A backend (rows 7, 9, and 10)" each keep the original sentence and append a `**Corrected 2026-08-01:**` note naming `VariantIneligibility::UnsupportedRepresentation` and `runtime.no-eligible-variant`, citing the ticket that moved them and the commit the correction was re-measured at. ADR 0090 is accepted and its obligation is unchanged, which the note says; the composition record's note adds the one thing the terminal class could not express — on a portfolio packaging a runnable alternative, the same exclusion leaves that alternative to be selected.

**One thing fixed that the ticket did not name.** `cargo fmt --check` was already red in the spike at base, on a `RecordedArtifactProgramIdentity::from_bytes` call this change does not touch. No `make` target reaches a spike workspace, so nothing would have caught it. Reformatted, so `cargo fmt --check` is a usable signal there again; called out here rather than folded in silently.

**Not touched.** The recorded outcomes of closed tickets under `tickets/` — `route-the-runtime-loader-through-the-dispatch-record.md`, `prototype-runtime-routing-commit.md`, `gate-the-runtime-fail-closed-probes.md`, and `define-backend-device-and-execution-context-vocabulary.md` all still name the retired classes and are accurate accounts of what those tickets did. `prototypes/candle-metal-adapter`'s `TensorRefusal::IncompatibleTargetProfile` is that prototype's own type and is unrelated to the removed loader class.
