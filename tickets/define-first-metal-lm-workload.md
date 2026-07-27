---
id: define-first-metal-lm-workload
title: Define the first representative Metal language-model workload
status: todo
priority: p1
dependencies: [scope-optimized-metal-lm-inference]
related: [derive-transformer-operation-and-shape-surface, design-model-level-qualification-and-optimization]
scopes: [research/program-planning, contracts/integrations, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, planning, language-model, workload, metal, inference]
---
Select and bound the first language-model inference workload that will drive
Tiler's capability growth. Do not use an unspecified "transformer" or
"LLM-compatible" claim as a substitute for an executable workload.

## User-visible question

What is the smallest representative language-model workload whose successful
execution would demonstrate that Tiler's compiler architecture can grow into a
useful Metal inference library?

## Required evidence and decisions

- Compare candidate model classes using their actual operation, dtype, shape,
  state, weight, and execution requirements.
- State the supported batch, prompt, sequence, and decode bounds.
- State the initial dtype and numerical requirements without preselecting
  quantization merely because it is cheaper.
- Name the initial Apple target profile and which claims require a live device.
- Define correctness and performance success measures at user-observable model
  boundaries.
- Explicitly exclude or defer training, distributed execution, speculative
  decoding, unsupported model architectures, and unbounded dynamic shapes.

Eliminate candidates that cannot test the intended architecture or that require
unrelated capabilities before presenting any genuine product choice to Tom.

## Ticket-producing outcome

File dependency-ordered follow-up tickets only for workload requirements not
already owned by the graph. Each new ticket must name the model-visible outcome
it enables, its evidence prerequisite, and its reconsideration trigger if
deferred.

## Closes when

One bounded workload profile and its success envelope are durably recorded;
the selection evidence and rejected candidates are reproducible; and every
newly exposed subsystem requirement is either linked to an existing owner,
filed as a scoped ticket, or explicitly deferred with a trigger.
