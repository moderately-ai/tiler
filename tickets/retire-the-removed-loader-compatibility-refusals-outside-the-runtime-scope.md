---
id: retire-the-removed-loader-compatibility-refusals-outside-the-runtime-scope
title: Retire the loader's removed compatibility refusals from the spike and the recorded measurements
status: in-progress
priority: p2
dependencies: []
related: [select-executable-variants-across-registered-backend-families]
scopes: [research/target-profiles]
shared_scopes: []
paths: []
tags: [runtime, backend-providers, documentation, spikes]
claimed_from: todo
assignee: worker-retire-the-r
lease_expires_at: 1785567332
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
