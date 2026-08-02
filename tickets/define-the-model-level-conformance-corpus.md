---
id: define-the-model-level-conformance-corpus
title: Define the model-level conformance corpus and its refusals
status: done
priority: p2
dependencies: [land-the-model-level-qualification-record, measure-the-model-level-comparison-envelope-under-the-target-realization]
related: [prove-the-c1-complete-model-execution, test-the-autoregressive-state-failure-cases, build-the-model-level-measurement-harness, define-the-model-level-regression-policy]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, testing, language-model, qwen, metal]
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
- Record why two expected rows are deliberately absent, so a later reader does not add them: a subnormal weight is unreachable, because a BF16 subnormal widens to an F32 **normal**; and a NaN or infinite weight is a one-line check against the widened bytes the fixture already digests, owned by [`ingest-the-checkpoint-as-f32-program-inputs`](ingest-the-checkpoint-as-f32-program-inputs.md) rather than by a conformance corpus. (**Correction — 2026-08-02.** The subnormal ground stated here is false; the row is still absent and the reason is now measured. See the outcome below.)
- For every row that expects a refusal, name the site the refusal comes from and whether it exists today. Several do not; the corpus records that as a row a build cannot yet fail rather than as a row that passes.

## Explicit non-goals

No harness — [`build-the-model-level-measurement-harness`](build-the-model-level-measurement-harness.md) owns it. No threshold and no regression policy. No B1-length correctness row: the workload profile makes C1 the only fully retainable row, and a B1 accuracy comparison retains a bounded summary under a separately derived bound.

## Closes when

Every row has exact inputs, one of the three expected outcomes, the boundary it derives from, and — for a refusal row — the site and whether that site exists; the two deliberate absences are recorded with their grounds; and no row duplicates one the state-failure suite already owns.

## Outcome

The corpus is fixed in [the L8 qualification record](../docs/research/program-planning/model-level-qualification.md#the-conformance-corpus-fixed--2026-08-02), as a dated section below the transferred span rather than an edit to it.

**Thirteen rows**, each with complete token IDs, `T`/`C`/`S` per execution, `capacity`, and its bound weight set: C1 and A-prompt-1, A-token-low, A-token-high, A-token-out, A-eos-in-prompt, A-tie, A-tiled-guard, A-mask-value, A-capacity, A-position-range, A-cursor-consistent, and A-fallback-after-commit. The last is required by the L8 record's *Feasibility* section in prose and was not in its table. Every row but two is a delta from C1, so a refusal is evidence about the one thing the row changed. **Required outcome and status today are kept apart on every row**: six require a pass, four a refusal, two a disagreement, one is `Unknown` — and *every* row's Tiler-side status is `Unknown`, because L6's five refusals stand and no Tiler execution of this workload exists.

**Refusal sites, checked by reading `crates/` at `54833c9`.** Four do not exist: the gather's bounds boundary, an integer storage carrier for the `[T]` token-ID operand, the runtime instance's capacity check, and the model-level preflight. Two exist in part: the shape environment's bounded-extent mechanism without the program that would declare the bound, and `RoutingPolicy::StablePriority` without a second packaged variant to discriminate. One is absent by construction — A-cursor-consistent has no refusal, which is the row's subject. The bind refusals every pass row rests on are implemented. **The corpus records one false green:** a build today refuses A-token-out for the wrong reason — `StorageScalarMismatch`, because no integer carrier exists — so a harness recording only `refused` would report a mechanism that does not exist as covered.

**A-tie was searched and not found.** [The corpus reachability probe](../spikes/program-planning/qwen3-corpus-reachability/README.md) evaluated 19 prompts and 330 positions through the pinned reference: 0 bit-identical top-two pairs. The structural route is live — 28 duplicate embedding-row groups over 2,226 of 151,936 rows, so any prompt whose greedy token is one of those produces a tie by construction — and the searched drivers did not reach it, the best-placed group member ranking 86,718th and sitting 17.45 logits below the maximum. The row stays `Unknown` and [`search-a-tie-demonstrating-prompt-for-the-model-level-corpus`](search-a-tie-demonstrating-prompt-for-the-model-level-corpus.md) is filed `deferred` with two activation triggers.

**The absences, with the grounds verified rather than restated — and one of them was wrong.** The subnormal-weight row's inherited ground, that a BF16 subnormal widens to an F32 normal, is **false**: widening preserves the class in all 254 cases over an exhaustive 65,536-pattern population, which is what [the BF16 conversion record](../docs/research/numerics/bf16-computation-accumulator-and-conversion.md)'s stage 5 already measured and what [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md)'s `bf16`-flushes/`f16`-preserves explanation already depends on. The clause is true of binary16 and was carried into the language-model branch. The row is still absent, on a measured ground: **0 subnormal, 0 infinite, and 0 NaN stored values over all 596,049,920 elements of all 310 tensors** of the pinned revision. The NaN-or-infinite absence is confirmed as stated — `host.tsv` carries `weights.widened.sha256` over all 310 widened tensors, so it is one line against bytes already digested. A **third** absence is recorded so a reader does not add it: the fully-masked row and the masked-position signed zero are block-level rows that [`retain-the-c1-attention-block-conformance-evidence`](retain-the-c1-attention-block-conformance-evidence.md) already owns.

**Corrections landed for the false clause** at its four reachable sites: [L1's dtype-boundary paragraph](../docs/research/program-planning/first-metal-lm-workload.md), [L6's I-B row](../docs/research/program-planning/complete-model-ingestion-and-execution.md), [the conformance fixture's P-flush paragraph](../spikes/program-planning/qwen3-conformance-fixture/README.md), and this ticket, [`ingest-the-checkpoint-as-f32-program-inputs`](ingest-the-checkpoint-as-f32-program-inputs.md), and [`measure-the-model-level-comparison-envelope-under-the-target-realization`](measure-the-model-level-comparison-envelope-under-the-target-realization.md). **Two copies are deliberately not corrected**: the clause inside the L8 record's transferred span and its source in [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md), because editing either breaks the byte-identity that makes the span quotable against its source. The dated section carries the correction, which is the convention that record states for itself.

**A-cursor-consistent is here and not duplicated.** The state suite's case 7 is the *inconsistent* incorrect-position case, which a position-selecting rotary slice makes unrepresentable; A-cursor-consistent is what survives after that slice lands, and the record states the distinction so the two are not merged.
