---
id: decide-whether-tiler-reference-publishes-a-bit-extraction-convenience
title: Decide whether tiler-reference publishes a bit-extraction convenience
status: todo
priority: p3
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [reference, public-boundary]
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
