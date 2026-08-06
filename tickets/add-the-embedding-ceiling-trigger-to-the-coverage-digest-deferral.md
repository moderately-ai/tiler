---
id: add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral
title: Add the embedding-ceiling trigger to the coverage-digest deferral
status: done
priority: p2
dependencies: []
related: [decide-whether-executable-coverage-evidence-folds-as-a-digest, attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget, decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [identity, deferred, artifacts, embedding]
---
## User-visible outcome

[`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md) fires on the consumer that actually binds first. Today its triggers are sized entirely against the 64 MiB `MAX_PROGRAM_IDENTITY_BYTES` refusal, and the 1 MiB per-invocation embedding ceiling refuses roughly **21× earlier in operation count**.

## Why this exists

**Measurement** ([Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md), 2026-08-06). The per-occurrence coverage evidence that a kernel-program identity carries is stored **four times** in one artifact envelope — once as the framed `KernelProgramSubject` section, once in the manifest body's per-entry stage subjects, and twice inside the canonical-identity run, which folds the whole program-subject section verbatim and then restates the entries' stage subjects. The multiplicity was measured exactly at the landing that introduced the evidence: 20,144 bytes of program identity became 80,576 bytes of envelope.

**Measurement, retained** ([`spikes/program-planning/identity-growth`](../spikes/program-planning/identity-growth/README.md)). Kernel-program identity is exactly `134n² + 3650n + 710` bytes for `n` semantic operations, and the 64 MiB program-identity bound binds at **695 operations**.

**Inference — the crossing the deferral does not carry.** `4 × (134·32² + 3650·32 + 719) = 1,018,940` and `4 × (134·33² + 3650·33 + 719) = 1,068,380`, so the envelope's fixed content passes the **1,048,576-byte per-invocation embedding ceiling between 32 and 33 semantic operations**. The decoder-layer program that [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md) contemplates is **≥ 51 operations**, which the same arithmetic puts at ≈ 2.04 MiB of fixed content before a single object byte.

**Why this matters to the deferral rather than to the ceiling.** The deferral's trigger 1 is "a program boundary that admits more than ~350 operations". That is half the 695-operation refusal point, and it is roughly **eleven times past** the point at which the embedding consumer already fails. A boundary widened to 100 operations would leave every stated trigger unfired while making the artifact unembeddable.

**The bound on the inference, which the trigger text must carry.** The multiplier of four is measured on one fixture for one landing's increment and is structural rather than swept. The curve is fitted to a different program family over 2..=8 operations and extrapolated, and its own record says richer families raise the slope and lower the crossing. What this licenses is the ordering — the embedding ceiling binds first, by a wide margin — not the number 32.

## What this ticket owes

One trigger appended to `decide-whether-executable-coverage-evidence-folds-as-a-digest`, phrased so it is checkable in one command like the three it joins, together with the arithmetic above in that ticket's *Why this exists* and a dated `## Trigger check log` entry evaluating it. Proposed text, to be edited into that ticket rather than restated here:

> 4. **A program boundary that admits more than ~30 operations**, which is where the four-fold envelope restatement of coverage evidence passes the 1,048,576-byte per-invocation embedding ceiling — roughly 21× earlier in operation count than trigger 1's bound and below the ≥ 51-operation decoder-layer program the roadmap contemplates. The multiplier and the crossing are [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md) Section 5, with its extrapolation boundary stated there. Check: `grep -n 'semantic_operations' crates/tiler-compiler/src/request.rs`, against 30 rather than 350.

**Why it is a separate ticket rather than an edit made in passing.** The deferral is another ticket's body at `deferred`, its trigger set is what makes it a deferral rather than an open question, and expanding another ticket's outcome is the coordinator's call. The measurement is done; what is owed is the edit and one trigger-log evaluation.

## Explicit non-goals

Not moving `semantic_operations`. Not deciding the program boundary. Not deciding either digest question — [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md) owns the IR layer's and [`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md) owns the artifact layer's. Not re-deciding the 1 MiB ceiling, which the embedding notes own.

## Closes when

The trigger is on the deferral with its evidence, and one dated entry in that ticket's `## Trigger check log` records it as `fired`, `not fired`, or `unevaluable` with a reproducing command.
