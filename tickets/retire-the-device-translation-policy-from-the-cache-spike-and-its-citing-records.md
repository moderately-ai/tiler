---
id: retire-the-device-translation-policy-from-the-cache-spike-and-its-citing-records
title: Retire the device-translation policy from the cache spike and its citing records
status: done
priority: p2
dependencies: []
related: [route-or-refuse-the-device-translation-execution-policy, restore-the-spikes-against-the-composed-numerical-contract]
scopes: [research/cache, research/extensions, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [artifacts, spikes, documentation]
---
## User-visible outcome

Nothing in the repository still names `ArtifactExecutionPolicy::RequiresDeviceTranslation` as an existing value: the one spike that declares it stops failing on that declaration, and the three records that describe the vocabulary as a two-valued dichotomy with an unroutable member describe what it is now. **Revised 2026-08-01** from "compiles again" — that spike fails on two further drifts predating this ticket, recorded under "What landed" and owned elsewhere.

## Why this exists

[`route-or-refuse-the-device-translation-execution-policy`](route-or-refuse-the-device-translation-execution-policy.md) retired the variant on 2026-08-01 after eliminating the Route-it candidate against ADR 0086's uninhabited translation authority. That ticket held `implementation/artifact`, `implementation/runtime`, and `contracts/decisions`, so it corrected the crates and ADR 0090:125 and could reach nothing else. These four sites are the remainder, split out rather than absorbed or left implied.

**Fact — one is a compile breakage, not a stale sentence.** `spikes/cache/build-tool-exercise/envelope/src/lib.rs:139` declares `execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation`. That spike is its own Cargo workspace (`[workspace]` at line 1 of `spikes/cache/build-tool-exercise/Cargo.toml`) and is not a member of the root workspace, so `make check` and `make full` never built it and stayed green through the retirement. It will not compile against the current `tiler-artifact` until this lands. `NativeImage` is the correct replacement and needs no other edit: the spike exercises envelope assembly, not delivery.

**Fact — three records describe the old shape.**

- `docs/research/extensions/backend-provider-composition.md:262` states "one of its two values is unroutable" and cites `crates/tiler-runtime/src/load.rs:468-473` returning `LoadRejection::UndeliverableExecutionPolicy`. Both the claim and the rejection variant are gone. The *surviving* half of that bullet — no spelling for an interpreted image, a JIT input, or a dynamically linked object — is still true and must be preserved rather than deleted with the rest.
- `spikes/target-profiles/scalar-cpu-vertical/README.md:118` records finding 6 as "`NativeImage` or `RequiresDeviceTranslation`". This is a *retained measurement* of what the vocabulary was when the vertical ran, so correct it the way a spike record is corrected — a dated note that the second value was retired and why the finding's substance survives — rather than rewriting the observation.
- `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:325` carries a comment reasoning that the scalar image "needs neither" source compilation nor pipeline creation, quoting the doc comment that the retirement replaced. The declaration it justifies is unchanged and still correct; the quotation is what went stale.

## Boundaries

- Do not reintroduce the variant. Wire tag `0x02` is retired and never reassigned; `crates/tiler-artifact/src/program/model.rs`'s `from_tag` and `the_retired_execution_policy_tag_is_refused_by_name` are the authorities.
- `docs/backends/cpu.md:55` needs no edit and is deliberately excluded: it says `ArtifactExecutionPolicy` "has no spelling for an interpreted image", which the retirement did not change. Checked by reading the line, not by grepping for the variant name.
- Two `done` tickets (`specify-the-consumer-neutral-backend-provider-composition-contract`:76 and `prototype-a-bounded-scalar-cpu-backend-vertical`:55) also name the variant. Leave both: they are records of what was observed when they ran, and rewriting a closed ticket's findings would forge its evidence.

## What landed, 2026-08-01, on base `29a9680`

All four sites are edited. Old → new, one line each unless stated:

1. `spikes/cache/build-tool-exercise/envelope/src/lib.rs:139` — `ArtifactExecutionPolicy::RequiresDeviceTranslation` → `ArtifactExecutionPolicy::NativeImage`.
2. `docs/research/extensions/backend-provider-composition.md:262` — the bullet keeps finding 6's measurement verbatim and replaces the **Fact — one of its two values is unroutable** half with a dated `Fact, at b480ec8` naming the retirement, the removal of `LoadRejection::UndeliverableExecutionPolicy`, and the loader's still-exhaustive `match`; the *Inference* now says the measured gap is the whole of what the bullet refuses.
3. `spikes/target-profiles/scalar-cpu-vertical/README.md` finding 6 — the observation is preserved as measured and carries a dated note that the second value was retired, why nothing is re-measured, and why the gap the finding names is untouched.
4. `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:334-344` — the stale quotation of the enum's old doc comment is replaced by what it says now (`NativeImage` means the target's own API loads these bytes as they stand; the enum answers delivery alone); the `ArtifactExecutionPolicy::NativeImage` declaration it justifies is unchanged.

**Fact — site 1's fix is proved by a deliberate perturbation, not by a green build.** `CARGO_TARGET_DIR=./target cargo check` from `spikes/cache/build-tool-exercise` reports two errors with the fix in place and three with the retired variant re-inserted, the third being `E0599: no variant, associated function, or constant named RequiresDeviceTranslation found for enum ArtifactExecutionPolicy` at `envelope/src/lib.rs:139`. The variant was re-inserted, the error observed, and the fix restored.

**Fact — both spikes are broken by drift this ticket does not own, and that drift is filed.** The cache spike's remaining two errors are `NumericalContract::FlushSubnormalsToZeroF32` (`lib.rs:59`) and `BackendEntryRef` having no field named `payload` (`lib.rs:160`); the CPU vertical fails on `NumericalContract::StrictF32` (`src/vertical.rs:779`) and so never runs. [`restore-the-spikes-against-the-composed-numerical-contract`](restore-the-spikes-against-the-composed-numerical-contract.md) owns all three and was updated with the second cache-spike site, which its enumeration did not name. Nothing here was absorbed into this branch: repairing them requires a re-run and a drift table per spike, which is that ticket's evidence obligation.

**Measurement boundary.** Site 4's edit has no compile proof, because the spike it lives in does not compile for the unrelated reason above. It is a comment, so it cannot change the program's behaviour, and the fixture is byte-identical by inspection rather than by re-run: `results/2026-07-31-macos-arm64.json` is unchanged at sha256 `7c774b159d06f489c6c8d8ab44d29ae09d277b5fbd5eb0da9e4530da05877196`, because the run stopped at compilation and never reached the point where it is written.

## Closes when

The four sites read as above; the three records describe the current vocabulary with their surviving claims intact; and no file outside this ticket, the retiring ticket, and the two closed records names `RequiresDeviceTranslation` as a value that exists — the CPU vertical's finding 6 excepted, where the Boundaries above require the observation to be retained as measured and the dated note carries what changed.

**Revised 2026-08-01.** The original sentence also required the cache spike to compile. It does not, for two reasons predating this ticket and belonging to [`restore-the-spikes-against-the-composed-numerical-contract`](restore-the-spikes-against-the-composed-numerical-contract.md); what this ticket owed is that the retirement is no longer one of its errors, which the perturbation above establishes.
