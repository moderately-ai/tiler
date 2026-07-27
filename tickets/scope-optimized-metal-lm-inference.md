---
id: scope-optimized-metal-lm-inference
title: Scope the optimized Metal language-model inference initiative
status: todo
priority: p1
dependencies: []
related: [define-first-metal-lm-workload, derive-transformer-operation-and-shape-surface, spike-first-metal-contraction-vertical, scope-transformer-nonlinear-normalization-and-reductions, design-attention-program-vertical, design-autoregressive-state-and-kv-cache, design-model-ingestion-and-complete-execution, scope-first-quantized-lm-profile, design-model-level-qualification-and-optimization]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [planning, roadmap, epic, language-model, metal, inference]
---
Create the durable long-term capability map for compiling and executing an
optimized language-model inference pipeline on Metal. This is a visibility and
sequencing ticket, not authorization to implement a model compiler.

## User-visible outcome

A reader can see the evidence-gated progression from the current bounded Metal
proof to:

1. a representative language-model workload;
2. the required tensor operation and shape surface;
3. contraction and transformer-math verticals;
4. a complete attention program and transformer block;
5. stateful prefill and token decoding;
6. complete supported-model execution;
7. a selected quantized model profile; and
8. model-level correctness and performance qualification.

The map must distinguish a semantic reservation, architectural seam,
implemented support, tested guarantee, and measured performance claim. It must
also state that the initial goal is inference rather than training unless Tom
explicitly broadens the product goal.

## Deliverables

- Reconcile the initiative against the live operation-family matrix, roadmap,
  open questions, and ticket graph.
- Give every subsystem one owning discovery/design ticket, an activation
  trigger, dependencies, and a user-language completion outcome.
- Identify existing tickets that already supply prerequisites rather than
  duplicating them.
- Record explicitly deferred capabilities such as training, distributed
  execution, speculative decoding, and unconstrained dynamic shapes with
  reconsideration triggers.
- File only the additional discovery or delivery tickets justified by that
  reconciliation, with scopes and dependency edges checked by `tkt lint`.

## Closes when

The durable roadmap and live ticket graph agree on the capability ladder; every
named subsystem has one owner and an activation trigger; there are no
implementation-shaped tickets whose prerequisite evidence is still absent; and
the child discovery tickets below are correctly linked and dependency ordered.
