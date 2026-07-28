---
id: scope-optimized-metal-lm-inference
title: Scope the optimized Metal language-model inference initiative
status: done
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

## Outcome — the ladder is in the durable roadmap, and every rung carries a trigger (2026-07-27)

**The reconciliation found the graph already right and the roadmap silent.** The nine child tickets exist, are correctly linked, and are dependency-ordered — verified by reading each one's `deps`, not by trusting the `related` list. What was missing was the durable half: `docs/roadmap.md` mentioned attention only inside milestone 6's contraction question and carried no capability ladder at all, so "the durable roadmap and live ticket graph agree" was false in the direction nobody checks.

**Added to `docs/roadmap.md`: an "Optimized Metal language-model inference" section**, placed before the operation-family support matrix and written in that matrix's own idiom so the two can be read together:

- **The eight-rung ladder** (L1–L8, with L3′ running beside L3), each rung naming its owning ticket, its activation trigger, and its maturity today.
- **Every rung is "none", and the section says so plainly.** The support matrix records four strict-`f32` operations as the supported profile; a transformer needs contraction, softmax, layer normalization, and a residual add, none above R2. Nothing here is partially built, and recording a rung as started would be the exact overstatement this ticket exists to prevent.
- **Inference, not training**, stated as a scope position with the trigger that would reopen it.
- **Four prerequisites owned elsewhere**, cross-referenced rather than restated: the milestone 6 keyed-family question that L3 cannot be specified without, the dtype taxonomy, milestone 2Q's quantized-value proof, and the numerical contracts. This is the "identify existing tickets that already supply prerequisites rather than duplicating them" deliverable.
- **Four explicitly deferred capabilities with reconsideration triggers** — training, distributed execution, speculative decoding, and unconstrained dynamic shapes — each stating that no seam is reserved for it, so a reader cannot mistake deferral for a reservation.

**Every one of the nine children now carries its own activation trigger**, naming its rung, what must fire first, and what it rests on. They previously carried none, so the ordering existed only as `deps` edges and a reader opening one ticket could not tell whether it was startable. Each also says why starting early is harmful: a rung's scope derives from the rung below, so beginning ahead of the trigger means deriving a surface from an assumption.

**The remaining closing criteria were checked rather than assumed.** No child is implementation-shaped — all nine are tagged `research`, `design`, or `spike`, and the one spike is a bounded experiment. Every link added to the roadmap was resolved against the filesystem; none is broken. Nothing in this change authorizes implementation, which is what the ticket asked for: it is a visibility and sequencing artefact and says so in its own first paragraph.

**No additional tickets were filed.** The reconciliation justified none — the nine existing children cover every named subsystem, and filing more would have been scope invented to look productive.
