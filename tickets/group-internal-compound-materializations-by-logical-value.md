---
id: group-internal-compound-materializations-by-logical-value
title: Group internal compound materializations by producer-derived logical value
status: deferred
priority: p2
dependencies: [prototype-quantized-value-vertical]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, compound-values, artifact]
---

## User-visible outcome

When a selected physical program actually produces a compound value internally, every component remains attached to that producer-derived logical value through program construction, stage access, allocation, artifact projection, and lifetime analysis. Components may use separate storage, but they cannot be detached, regrouped, or confused with matching components of another value.

## Why this is deferred

- **Fact:** [`prototype-quantized-value-vertical`](prototype-quantized-value-vertical.md) already proves complete role-addressed compound interface inputs and outputs and rejects an ungrouped internal component.
- **Fact:** [the selected quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) is `input(compound weight) → DequantizeStrictAffine → Contraction`. It contains no `Quantize` or `Assemble`; the selected compound weight is an interface input, not an internally materialized result.
- **Fact:** [`admit-strict-affine-quantize-physical-candidate`](admit-strict-affine-quantize-physical-candidate.md) closed obsolete under its own trigger after that selection. It is not a future consumer for this ticket.
- **Inference:** without a selected internal producer and a real downstream consumer, choosing a public result-binding surface, stage topology, or identity/schema grammar would reserve a producer-less abstraction. The public-boundary decision drafted on 2026-08-08 was therefore removed rather than left as an authorization request.

## Current boundary to preserve

`MaterializedOrigin` currently distinguishes `ProgramInput { key }` from compiler-produced `Internal`. Its source contract explicitly says that it does **not** identify which semantic value a temporary realizes; that attribution remains compiler-owned refinement evidence rather than target-neutral program structure. This ticket must preserve that authority split, but it does not preselect the future representation, identifier spelling, result-binding cardinality, public surface, stage accounting, or identity/schema steps.

## Reconsideration trigger

Move this ticket back to dispatchable work only when an accepted workload/profile or implementation ticket selects all of the following:

- an operation that produces a compound value inside the physical program;
- the exact semantic occurrence and ordered result that own the value;
- a real later consumer of that complete result;
- the resolved value type, logical shape, component roles, parameter maps, and physical topology required by that producer/consumer pair; and
- a demonstrated failure in the current `MaterializedOrigin` plus compiler-owned attribution path that cannot be closed without grouping internal components.

Activation quantization, requantization, or another future producer may fire the trigger, but none is assumed here. When it fires, re-read the actual producer, consumer, construction sites, `MaterializedOrigin` contract, verification path, artifact projection, and identity encoders before deciding the smallest boundary. Do not pre-name an identifier, ask Tom to choose transport for already-derived result type/shape facts, or prescribe version steps before the changed grammar exists.

## Closes when

The triggered, selected internal producer's complete result reaches its real consumer with producer-derived grouping through every required program and artifact boundary; cross-result substitutions reject; identity changes only for result-affecting grammar that actually lands; unsupported producers remain named; and every new check is fault-proved before the normal package and repository gates pass.

## Trigger check log

- 2026-08-08 — **not fired.** `rg -n 'input\(compound weight\).*DequantizeStrictAffine.*Contraction' docs/research/numerics/first-quantized-lm-profile.md` identifies the selected path as a direct compound input followed by decode and contraction, and the same record explicitly excludes activation quantization, mixed precision, and KV-cache quantization. No selected internal compound producer exists.
