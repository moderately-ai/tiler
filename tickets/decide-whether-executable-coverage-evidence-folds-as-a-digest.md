---
id: decide-whether-executable-coverage-evidence-folds-as-a-digest
title: Decide whether executable-coverage evidence folds as a digest
status: review
priority: p2
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound, bind-stage-coverage-to-index-refinement-identity, site-the-governed-digest-so-layered-identity-encoders-can-reach-it]
scopes: [contracts/decisions, implementation/ir, implementation/build, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [identity, decision, program-planning, deferred]
claimed_from: todo
assignee: agent-identity-campaign
lease_expires_at: 1786061768
---
## User-visible outcome

If the kernel-program boundary ever admits programs large enough for the quadratic term in program identity to matter, the choice between keeping the embedded `SemanticGraphIdentity` and folding it as a digest is a recorded decision with its measurement already in hand — rather than a 64 MiB refusal discovered by whoever first fuses too much.

**This was a deferral until 2026-08-06; trigger 4 has fired.** See the trigger check log at the end.

## Why this exists

**Measurement, 2026-08-05, retained at [`spikes/program-planning/identity-growth/`](../spikes/program-planning/identity-growth/README.md).** Kernel-program identity is exactly `134n² + 3650n + 710` bytes over the whole domain the ordinary compilation path admits — an exact fit reproducing all seven measured points to the byte, whose quadratic coefficient *is* the per-operation slope of the graph identity it embeds. The mechanism is confirmed rather than inferred: one whole graph identity per coverage record, one record per operation. Solving against `MAX_PROGRAM_IDENTITY_BYTES` (64 MiB, `crates/tiler-ir/src/program/mod.rs:429`) puts the refusal at **695 operations**.

**Inference — the margin holds today, and it holds because of a partition chosen for another reason.** [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md#whole-model-composition-three-programs-thirty-executions-three-identities) proposes one forward pass as three programs, the largest being the decoder layer at **≥ 51 operations** — 535,394 fitted bytes, **0.80% of the bound, a ×125 margin**. That partition is grounded in artifact-identity reuse, and its own derivation says the ground is "artifact-identity reuse rather than size". The size consequence is a second, independent reason the same cut is correct, and nothing currently records it where a later reader of the partition would look.

**Inference — the contingency is what this ticket preserves.** The same forward pass compiled as *one* program is [≥ 1,068 occurrences](../docs/research/shapes/transformer-operation-and-shape-surface.md#occurrence-inventory-for-one-forward-pass), which the fitted curve puts at **≈ 149 MiB, about 2.3× the bound**. So the encoding is not safe by construction; it is safe under a program-boundary decision that a later cost or scheduling argument could reopen without knowing this ceiling exists.

**Fact — the redundancy a digest-form proposal would rest on is already provable.** `encode_identity` (`crates/tiler-ir/src/program/model.rs:1842`) writes the program's one `SemanticGraphIdentity` above the stage section, and the builder proves every `CoveredOccurrence` names that same graph — the destructuring at `model.rs:1856` binds and discards `graph` for exactly that reason, with a comment saying so. The per-record copy determines nothing the encoding has not already fixed at the program layer.

**Fact — nothing can approach the bound today.** `DeterministicBudgets::governed` caps `semantic_operations` at 8 (`crates/tiler-compiler/src/request.rs:821`), roughly two orders of magnitude below the largest contemplated program and nearly two below the refusal point.

**Superseded 2026-08-06, twice, and the second half is what fires this ticket.** The budget is no longer 8: `36d05128` raised `semantic_operations` to **62**, sized to the decoder-layer program. And the 64 MiB program-identity bound is no longer the consumer that binds first: [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md) measured that the envelope stores the per-occurrence coverage evidence **four times** (the framed program-subject section, the manifest's per-entry stage subjects, and twice inside the carried identity run), so the envelope's fixed content passes the **1,048,576-byte per-invocation embedding ceiling between 32 and 33 semantic operations** — `4 × (134·32² + 3650·32 + 719) = 1,018,940`, `4 × (134·33² + 3650·33 + 719) = 1,068,380` — against the 695 operations at which the identity bound refuses. Tom's 2026-08-06 digest decision on the manifest's carried identity run ([`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md)) cuts the multiplier to two, moving the crossing to **between 50 and 51** operations — `2 × (134·50² + 3650·50 + 719) = 1,036,438` against `2 × (134·51² + 3650·51 + 719) = 1,070,806`. *Corrected 2026-08-06: this sentence read "roughly 46–47 operations", which was an arithmetic error — solving `2 × (134n² + 3650n + 719) = 1,048,576` gives `n ≈ 50.35`, and the research note's own "~50" was right. The conclusion the figure supports is unchanged and is now tighter rather than looser: the crossing lands **at** the ≥ 51-operation decoder layer rather than under it, and still under the admitted budget of 62.* The extrapolation boundary is the record's Section 5: the multiplier is measured on one fixture, the curve is fitted on 2..=8 and extrapolated, so what this licenses is the ordering, not the exact crossing.

## Triggers

Any one of these fires this ticket. Each is checkable in one command.

1. **A program boundary that admits more than ~350 operations** — half the fitted refusal point, so that the margin is under 2× rather than under 125×. Check: `grep -n 'semantic_operations' crates/tiler-compiler/src/request.rs`.
2. **A measured per-operation graph-identity slope materially above 134 bytes** for a realistic family mix, which moves the refusal point down proportionally to its square root. Check: rerun `spikes/program-planning/identity-growth` after the operation families widen, and compare the reported `graph_bytes(n)` slope.
3. **A decision to compile a whole model, or a whole multi-layer span, as one semantic program** — that is, any supersession of the per-layer program boundary in [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md). Check: `grep -n 'program boundary' docs/research/program-planning/complete-model-ingestion-and-execution.md`.
4. **A program boundary that admits more than ~30 operations**, which is where the four-fold envelope restatement of coverage evidence passes the 1,048,576-byte per-invocation embedding ceiling — roughly 21× earlier in operation count than trigger 1's bound and below the ≥ 51-operation decoder-layer program the roadmap contemplates (about ~46 once the manifest digest decision lands and the multiplier halves). The multiplier and the crossing are [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md) Section 5, with the extrapolation boundary stated there. Check: `grep -n 'semantic_operations' crates/tiler-compiler/src/request.rs`, against 30 rather than 350.

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

## Outcome — 2026-08-06, the choice is made and its execution is blocked on a boundary Tom retains

**Tom delegated the choice** on 2026-08-06 at the live session's decision round, relayed by the orchestrator, under named criteria: "the most performant, correct, long term maintainable/extensible/quality choice". This is that derivation. **Nothing in the tree has moved for it**: no identity domain steps, no pin moves, and the encoding is unchanged.

**The choice: fold the graph identity as a digest inside the coverage record** — candidate 2. Carried by [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md), `decision_status: proposed`.

**The derivation core.** The mechanism is confirmed at one site: `encode_executable_coverage_identity` (`crates/tiler-ir/src/index/refinement.rs`) opens with `push_slice(&mut bytes, subject.graph.as_bytes())`, and the fitted `graph_bytes(n) = 134n + 149` against `program_bytes(n) = 134n² + 3650n + 719` makes the equality of the two 134s the proof that the quadratic term *is* that per-record restatement. The index-region identity carries no graph, which the coefficient independently confirms — a second graph-sized term per record would make it 268.

- **Correctness, ordered first.** The encoded copy has **no reader anywhere in the workspace**: nothing parses `IndexRefinementExecutableCoverageIdentity` bytes, the type has no decoder and no byte constructor, and every `as_bytes()` touch is a write into a larger identity, a capacity length, or a measurement. The one graph check that exists — `ForeignCoverageGraph` in the program builder — reads a *separate, unencoded* in-memory field on `CoveredOccurrence` and is unaffected by any encoding change. A digest keeps the record's meaning exactly: it still says "this occurrence of *this* graph" and still refuses two records naming one ordinal in different graphs.
- **Maintainability and extensibility.** The record stays a well-defined standalone value, so `IndexRefinementReceipt::executable_coverage_identity()` keeps meaning what it says. Future fields added to the record cost their own size once rather than once per operation.
- **Performance.** Program identity becomes `3525n + 719` — **linear**. The embedding-ceiling crossing moves from between 50 and 51 operations to **between 148 and 149**, and the 64 MiB program bound from 695 operations to roughly 19,000. The shape change matters more than either number: doubling a program doubles its envelope instead of quadrupling it.

**Why the other two lose.** *Keep the embedded identity* leaves a quadratic whose ceiling binds **at** the roadmap's own ≥ 51-operation decoder layer, below the admitted budget of 62, with no typed refusal — the failure mode is an artifact that compiles and cannot be embedded. *Stop restating the graph at all* is executable today with no digest and is marginally cheaper (`3493n + 719`), and it is rejected on correctness: the coverage encoder's own doc records that the operation key, signature, attributes, and boundary shapes are deliberately not re-encoded because the (graph, occurrence) **pair** already determines them, so deleting the graph leaves a bare ordinal that determines nothing and makes two receipts for occurrence *k* in unrelated graphs byte-equal. ADR 0071's amended clause rests on that same pair. It buys 32 bytes per record — **0.9% of the linear term** — for a public identity that is no longer injective on its stated subject.

**The strongest counterpoint, stated rather than buried.** The chosen option puts a digest inside a canonical layered identity's preimage, so that identity's injectivity rests on collision resistance rather than on encoding injectivity alone — and the 2026-07-27 decision on `decide-whether-layered-subject-digests-exist-as-hashes` holds that "canonical bytes are the only layered identity Tiler has". The answer is that the decision's target is a second *identity value* for a layer, one a consumer could compare instead of the canonical bytes, whose agreement "could only ever be argued and never checked"; this mints no such value — no type, no accessor, never held, never compared, not liftable — so there is nothing to disagree. That is a real disanalogy and it is also an argument, which is why this is a record for Tom rather than a change made here.

**The blocker, and it is architectural rather than a preference.** `IndexRefinementExecutableCoverageIdentity` is minted in `tiler-ir`, which is the workspace's **bottom crate** — its only dependencies are `num-bigint`, `num-integer`, `num-traits`, and no workspace crate sits below it. The one governed digest lives in `tiler-artifact`, which depends **on** `tiler-ir`, so the dependency cannot be reversed. Executing the choice therefore needs either a second hashing implementation inside `tiler-ir` — which `codec/digest.rs` forbids in terms, and which ADR 0082 records as already decided in the other direction when the expansion cache's previous owner "proved unable to reach the governed digest" — or relocating the governed digest below `tiler-ir`, which is a consequential public crate, module, and type boundary that AGENTS.md reserves to Tom.

**So this ticket does not close.** It needs Tom to accept ADR 0104's choice *and* answer one atomic question: should the governed digest move below `tiler-ir` so layered identity encoders can fold a bounded-width reference, or does the coverage encoding keep its quadratic until another lever removes it? Filed as [`site-the-governed-digest-so-layered-identity-encoders-can-reach-it`](site-the-governed-digest-so-layered-identity-encoders-can-reach-it.md).

**One correction rides with this.** The trigger-4 paragraph's "roughly 46–47 operations" was an arithmetic error; the crossing is 50/51. Corrected in place above with the derivation shown.

## Trigger check log

- 2026-08-05 — **not fired.** All three triggers evaluated at `5f14cd11`. `semantic_operations` is 8, two orders of magnitude below trigger 1's ~350. No family widening has landed, so trigger 2's slope is the measured 134. The per-layer boundary stands as a pending proposal, so trigger 3 has not moved. Reproduce: `grep -n 'semantic_operations: ' crates/tiler-compiler/src/request.rs`.
- 2026-08-06 — **trigger 4 fired at the moment it was added** ([`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md) landed it from the manifest-growth attribution). `DeterministicBudgets::governed` caps `semantic_operations` at **62** — `grep -n 'semantic_operations' crates/tiler-compiler/src/request.rs` — which is past the ~30-operation embedding-ceiling crossing (and past the ~46 post-digest one), so the boundary already admits programs whose envelopes could not be embedded. Triggers 1 and 3 remain unfired (62 < ~350; the per-layer boundary stands); trigger 2 is currently **unevaluable** — the identity-growth ladder refuses on its own wall probe now that the budget passed 8, owned by [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](widen-the-identity-growth-ladder-to-the-governed-operation-budget.md). Per this ticket's own procedure, the atomic question goes to Tom with the measurement as evidence.
