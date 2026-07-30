---
id: design-model-ingestion-and-complete-execution
title: Design model ingestion and complete supported-model execution
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, design-autoregressive-state-and-kv-cache]
related: [prototype-public-compiler-api, prototype-candle-metal-adapter, prototype-inline-proc-macro-frontend]
scopes: [contracts/integrations, contracts/navigation, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [design, frontend, model, weights, integration, language-model]
---
## User-visible outcome

A consumer can hand Tiler a supported model (architecture, config, weights) and inputs, and receive logits — through a typed boundary that keeps every consumer format and Candle type out of compiler semantics.

Define how a consumer supplies a supported model architecture, configuration,
weights, and inference inputs and receives logits without making a consumer
format or Candle type part of compiler semantics.

## Required design

- Select the bounded model-description and weight-container boundary required
  by the representative workload.
- Map configuration and weights into typed semantic program inputs with
  complete identity, shape, dtype, layout, and validation rules.
- Define whole-model composition across layers, entrypoints, artifacts,
  runtime instances, and persistent decode state.
- State unsupported-model, unsupported-operation, and fallback behavior before
  routing commit.
- Separate tokenizer and sampling concerns from compiler ownership while
  identifying the integration contract needed to produce and consume logits.
- Define complete-model reference comparison and failure reporting.

## Ticket-producing outcome

File delivery tickets for model description or adapter work, weight validation
and binding, whole-model graph construction, artifact/program orchestration,
consumer integration, and a complete supported-model execution proof. Reuse
the existing public compiler, macro, Candle, artifact, and runtime tickets where
they already own a prerequisite.

## Closes when

One supported model can be described end to end without an unowned boundary;
the frontend/runtime dependency direction remains consumer-independent; every
unsupported case has an explicit behavior; and the complete-model vertical is
represented by scoped, dependency-ordered tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L6** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L2 and L5 both deliver.

**Rests on:** L2 and L5.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Derive the model boundary from that exact revision rather than from a generic transformer. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).
