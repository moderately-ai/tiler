---
id: retire-the-device-translation-policy-from-the-cache-spike-and-its-citing-records
title: Retire the device-translation policy from the cache spike and its citing records
status: todo
priority: p2
dependencies: []
related: [route-or-refuse-the-device-translation-execution-policy]
scopes: [research/cache, research/extensions, research/target-profiles]
shared_scopes: []
paths: []
tags: [artifacts, spikes, documentation]
---
## User-visible outcome

Nothing in the repository still names `ArtifactExecutionPolicy::RequiresDeviceTranslation` as an existing value: the one spike that declares it compiles again, and the three records that describe the vocabulary as a two-valued dichotomy with an unroutable member describe what it is now.

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

## Closes when

The cache spike compiles and its declaration reads `NativeImage`; the three records describe the current vocabulary with their surviving claims intact; and no file outside this ticket, the retiring ticket, and the two closed records names `RequiresDeviceTranslation` as a value that exists.
