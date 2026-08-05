---
id: decide-whether-executable-coverage-evidence-folds-as-a-digest
title: Decide whether executable-coverage evidence folds as a digest
status: deferred
priority: p2
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound, bind-stage-coverage-to-index-refinement-identity]
scopes: [contracts/decisions, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, decision, program-planning, deferred]
---
## User-visible outcome

If the kernel-program boundary ever admits programs large enough for the quadratic term in program identity to matter, the choice between keeping the embedded `SemanticGraphIdentity` and folding it as a digest is a recorded decision with its measurement already in hand — rather than a 64 MiB refusal discovered by whoever first fuses too much.

**This is a deferral. Its triggers have not fired.** See the trigger check log at the end.

## Why this exists

**Measurement, 2026-08-05, retained at [`spikes/program-planning/identity-growth/`](../spikes/program-planning/identity-growth/README.md).** Kernel-program identity is exactly `134n² + 3650n + 710` bytes over the whole domain the ordinary compilation path admits — an exact fit reproducing all seven measured points to the byte, whose quadratic coefficient *is* the per-operation slope of the graph identity it embeds. The mechanism is confirmed rather than inferred: one whole graph identity per coverage record, one record per operation. Solving against `MAX_PROGRAM_IDENTITY_BYTES` (64 MiB, `crates/tiler-ir/src/program/mod.rs:429`) puts the refusal at **695 operations**.

**Inference — the margin holds today, and it holds because of a partition chosen for another reason.** [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md#whole-model-composition-three-programs-thirty-executions-three-identities) proposes one forward pass as three programs, the largest being the decoder layer at **≥ 51 operations** — 535,394 fitted bytes, **0.80% of the bound, a ×125 margin**. That partition is grounded in artifact-identity reuse, and its own derivation says the ground is "artifact-identity reuse rather than size". The size consequence is a second, independent reason the same cut is correct, and nothing currently records it where a later reader of the partition would look.

**Inference — the contingency is what this ticket preserves.** The same forward pass compiled as *one* program is [≥ 1,068 occurrences](../docs/research/shapes/transformer-operation-and-shape-surface.md#occurrence-inventory-for-one-forward-pass), which the fitted curve puts at **≈ 149 MiB, about 2.3× the bound**. So the encoding is not safe by construction; it is safe under a program-boundary decision that a later cost or scheduling argument could reopen without knowing this ceiling exists.

**Fact — the redundancy a digest-form proposal would rest on is already provable.** `encode_identity` (`crates/tiler-ir/src/program/model.rs:1842`) writes the program's one `SemanticGraphIdentity` above the stage section, and the builder proves every `CoveredOccurrence` names that same graph — the destructuring at `model.rs:1856` binds and discards `graph` for exactly that reason, with a comment saying so. The per-record copy determines nothing the encoding has not already fixed at the program layer.

**Fact — nothing can approach the bound today.** `DeterministicBudgets::governed` caps `semantic_operations` at 8 (`crates/tiler-compiler/src/request.rs:821`), roughly two orders of magnitude below the largest contemplated program and nearly two below the refusal point.

## Triggers

Any one of these fires this ticket. Each is checkable in one command.

1. **A program boundary that admits more than ~350 operations** — half the fitted refusal point, so that the margin is under 2× rather than under 125×. Check: `grep -n 'semantic_operations' crates/tiler-compiler/src/request.rs`.
2. **A measured per-operation graph-identity slope materially above 134 bytes** for a realistic family mix, which moves the refusal point down proportionally to its square root. Check: rerun `spikes/program-planning/identity-growth` after the operation families widen, and compare the reported `graph_bytes(n)` slope.
3. **A decision to compile a whole model, or a whole multi-layer span, as one semantic program** — that is, any supersession of the per-layer program boundary in [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md). Check: `grep -n 'program boundary' docs/research/program-planning/complete-model-ingestion-and-execution.md`.

## What to do when one fires

Put the atomic question to Tom with the measurement above as its evidence, and pause for the answer. The candidates, with what each enables and prevents:

- **Keep the embedded identity.** Coverage evidence stays self-describing: a record names the graph it is about without reference to the program carrying it, which is what makes `IndexRefinementExecutableCoverageIdentity` meaningful outside a program. Cost: a hard ceiling on program size that is an encoding artefact rather than a planning decision.
- **Fold the graph identity as a digest inside the coverage record.** The quadratic term collapses to linear and the ceiling stops binding at any contemplated size. Cost: a digest is not the canonical bytes, and [ADR 0074 convention 2](../docs/decisions/0074-public-api-conventions.md) keeps a canonical identity opaque and never re-derived — so this needs an argument that a digest at *this* site is a fold input rather than an identity a consumer compares.
- **Stop restating the graph per record at all**, naming it by the program's own folded copy. Cheapest bytes, and it makes the record meaningless outside a program — the property the current encoding was chosen for, so this needs the strongest justification of the three.

Whichever wins, moving the projection is an identity-domain step: `tiler.ir.index-refinement-executable-coverage.v1` moves at its owning layer, the ledger documents move in the same commit, and every pinned identity is recomputed on the tree the step lands into with each moved pin enumerated. Half a step is worse than none.

## Explicit non-goals

Not raising `semantic_operations`. Not deciding the program boundary — that is [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md)'s, and this ticket only says what the boundary costs if it moves. Not executing an identity-domain step before a trigger fires and the decision is Tom's and recorded.

## Closes when

A trigger has fired, Tom has answered, and the answer is an accepted ADR or an explicit re-deferral with a new trigger — or the encoding has been shown to be unable to reach the bound under any contemplated program boundary, at which point this closes as obsolete.

## Trigger check log

- 2026-08-05 — **not fired.** All three triggers evaluated at `5f14cd11`. `semantic_operations` is 8, two orders of magnitude below trigger 1's ~350. No family widening has landed, so trigger 2's slope is the measured 134. The per-layer boundary stands as a pending proposal, so trigger 3 has not moved. Reproduce: `grep -n 'semantic_operations: ' crates/tiler-compiler/src/request.rs`.
