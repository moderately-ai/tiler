---
id: decide-whether-tiler-reference-publishes-a-bit-extraction-convenience
title: Decide whether tiler-reference publishes a bit-extraction convenience
status: in-progress
priority: p3
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [reference, public-boundary]
claimed_from: todo
assignee: worker-bits
lease_expires_at: 1787457908
---
## User-visible outcome

Either `tiler-reference` publishes one small evaluation-to-bits convenience with an accepted signature, or the decision to leave every caller writing it is recorded with its reason.

## Why this exists

Filed 2026-08-22 by `worker-packet`. The second re-derivation on `decide-the-backend-provider-conformance-harness-public-surface` itemized the independent backend fixture and found exactly one candidate export that is both genuinely reusable and genuinely non-self-certifying — the caller cannot manufacture an oracle. It is deliberately **not** folded into that public-boundary answer, because it belongs to `tiler-reference` rather than to any conformance facade, and because bundling a small unrelated surface into a facade decision is how an unaccepted item rides along.

**Fact — what the helper is.** `crates/tiler-conformance/tests/independent_backend/workload.rs` defines `reference_bits`, which builds a dense `Tensor` from `f32` bit patterns, evaluates the same `SemanticProgram` through `ReferenceEvaluator::standard()`, destructures `TensorPayloadView::Dense`, and returns the output element bits. Its own header states the property that makes it worth having: `Nothing in this file states an expected value`.

**Fact — it is currently hand-written per caller.** Re-audit the population at your base before proposing anything; do not assume the fixture is the only site.

## Required work

- Census the callers that already do this by hand. State the spellings searched for and why that set is complete; a census anchored on one phrasing under-counts silently.
- If the population is one, the honest answer is probably to publish nothing, and recording that is the outcome.
- Any published signature is a `tiler-reference` public boundary under ADR 0075. Treat it as a labelled draft until Tom accepts its exact included and excluded surface, and do not let it default a bit order, a payload view, or an evaluator profile.

## Closes when

Either an accepted signature exists with its unsupported cases named, or the no-publication answer is recorded with the census that supports it.

## Coordinator census at `1cb2a09e`, 2026-08-22 — the "population is one" hypothesis is very likely wrong, but 36 is a ceiling, not an answer

**Fact 1 verified.** `reference_bits` is at `crates/tiler-conformance/tests/independent_backend/workload.rs`, declared `pub(crate) fn reference_bits(program: &SemanticProgram) -> Vec<u32>`.

**The Required work says "if the population is one, the honest answer is probably to publish nothing." Do not start from that assumption.** A co-occurrence census — tracked `.rs` files mentioning **both** `ReferenceEvaluator` and `TensorPayloadView::Dense`, counted as *files*, run from a Python file rather than a shell one-liner — returns **36**.

**But 36 is an upper bound on candidates, not a count of duplicated helpers, and handing it on as a population would be the exact error AGENTS.md names.** Co-occurrence in a file is not the same as hand-writing this helper, and three of the 36 are disqualified by construction: `crates/tiler-reference/src/evaluate.rs`, `quantization.rs`, and `structural.rs` are the reference crate's **own internals** — they define the evaluator and cannot be duplicate callers of a convenience over it. The crate's own `src/tests.rs`, `src/bf16/tests.rs`, and its `tests/` files are in-crate and would not need a *published* surface to reach it either.

**The population that actually bears on a publication decision is the out-of-crate one.** By crate, the 36 break down as: `tiler-reference` itself 13 (internals plus in-crate tests, mostly disqualified), `tiler-compiler` 5, `tiler-conformance` 4, `tiler-runtime` 2, `prototypes/` 2, `spikes/` 4 — roughly **17 out-of-crate files** worth reading. Read each and classify it; a file may use both symbols for entirely unrelated reasons.

**So the census the ticket asks for is a reading task, not a grep.** State the spellings you searched and why that set is complete — mine is one vocabulary and a floor: a caller that reaches bits through a different accessor, or evaluates without naming `ReferenceEvaluator` directly, lands outside it. `grep -c` counts lines and I counted **files**; say which unit you report.

**The public-boundary constraint is unchanged and is the binding one.** Any published signature is a `tiler-reference` public boundary under ADR 0075, stays a labelled draft until Tom accepts its exact included and excluded surface, and must not default a bit order, a payload view, or an evaluator profile. If the reading says publish, **stop and produce a packet for Tom rather than publishing** — the ticket's Closes-when admits "an accepted signature", and acceptance is Tom's.
