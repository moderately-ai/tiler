---
id: define-the-model-level-conformance-corpus
title: Define the model-level conformance corpus and its refusals
status: in-progress
priority: p2
dependencies: [land-the-model-level-qualification-record, measure-the-model-level-comparison-envelope-under-the-target-realization]
related: [prove-the-c1-complete-model-execution, test-the-autoregressive-state-failure-cases, build-the-model-level-measurement-harness, define-the-model-level-regression-policy]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, testing, language-model, qwen, metal]
claimed_from: todo
assignee: agent-conformance-r2
lease_expires_at: 1785686685
---
## User-visible outcome

The model-level correctness corpus exists as named rows with the exact outcome each must produce — a pass, a typed refusal, or a detected disagreement — so that a qualification run reports which rows ran and which of them said no, rather than a rate.

## Evidence prerequisite

The L8 qualification record's *The adversarial corpus, derived from refusals that already exist* section supplies the rows and the boundary each is derived from. Every row traces to a refusal [`design-attention-program-vertical`](design-attention-program-vertical.md), [`design-autoregressive-state-and-kv-cache`](design-autoregressive-state-and-kv-cache.md), or [`design-model-ingestion-and-complete-execution`](design-model-ingestion-and-complete-execution.md) already owns; this ticket does not invent hazards, it fixes their inputs and expected outcomes.

## Required work

- Fix each row's exact inputs — token IDs, `T`, `C`, `S`, capacity, and the bound weight set — so that a row is reproducible from the ticket rather than from a reader's reconstruction.
- State, per row, which of three outcomes it must produce: `refused` with the typed reason and phase, `failed` with the execution ordinal and the token in flight, or `disagreed` with the observable and the position. A row whose expected outcome is "pass" states which observables it exercises and which it leaves untouched.
- **Include `A-cursor-consistent`, which no other suite can reach.** [`test-the-autoregressive-state-failure-cases`](test-the-autoregressive-state-failure-cases.md) covers the refusable and the *inconsistent* state failures; the L5 record states that after a single cursor authority removes the inconsistency mode, "a wrong `C` produces a consistently wrong program that only the conformance oracle detects". That row belongs here and must not be duplicated into the state suite.
- **Include `A-tie`.** The C1 row leaves the tie branch unexercised — at all 18 positions exactly one index attains the maximum and no top-two pair is bit-identical — so a demonstrating row has to be constructed, or the corpus records that no prompt producing one was found and the branch stays declared-and-untested.
- Record why two expected rows are deliberately absent, so a later reader does not add them: a subnormal weight is unreachable, because a BF16 subnormal widens to an F32 **normal**; and a NaN or infinite weight is a one-line check against the widened bytes the fixture already digests, owned by [`ingest-the-checkpoint-as-f32-program-inputs`](ingest-the-checkpoint-as-f32-program-inputs.md) rather than by a conformance corpus.
- For every row that expects a refusal, name the site the refusal comes from and whether it exists today. Several do not; the corpus records that as a row a build cannot yet fail rather than as a row that passes.

## Explicit non-goals

No harness — [`build-the-model-level-measurement-harness`](build-the-model-level-measurement-harness.md) owns it. No threshold and no regression policy. No B1-length correctness row: the workload profile makes C1 the only fully retainable row, and a B1 accuracy comparison retains a bounded summary under a separately derived bound.

## Closes when

Every row has exact inputs, one of the three expected outcomes, the boundary it derives from, and — for a refusal row — the site and whether that site exists; the two deliberate absences are recorded with their grounds; and no row duplicates one the state-failure suite already owns.
