---
id: research-readiness-gate
title: Run the research-to-implementation readiness gate
status: awaiting-decision
priority: p1
dependencies: [synthesize-core-contracts, synthesize-optimizer-contracts, synthesize-artifact-contracts]
related: []
scopes: [contracts/core, research/macro-environment]
shared_scopes: [contracts/compiler, contracts/artifacts, contracts/integrations]
paths: []
tags: [tiler-research, gate, decision]
---
Audit the synthesized design for contradictions, missing invariants, unmeasured feasibility claims, and decisions that would force crate or IR boundaries to change. Rank remaining unknowns by architecture impact and experimental cost, then propose the smallest defensible implementation slice.

This ticket does not authorize implementation. It is complete only after Tom reviews the gate, unresolved blockers remain explicit, and the roadmap records whether the project is ready for scaffolding, needs another research wave, or must narrow scope.

## Audit progress

Two independent read-only audits found the architecture coherent enough for a
bounded, explicitly unstable implementation slice. Three identity/delivery
gaps were resolved before presenting the gate:

- the full executable program is authoritative only in neutral artifact
  sections, Metal carries mappings/code only, and all stored/external digests
  are non-recursive;
- stable input/output interface keys and a bounded Tiler-owned canonical
  attribute identity encoding replace diagnostic names/provider serialization;
  and
- selected Apple-family build failures are delivered through governed
  consumer-target `cfg` diagnostics, while unrelated targets and explicit
  `FallbackOnly` use the semantic fallback without proc-macro target inference.

Two atomic decisions remain. First, choose the first Metal value-proof workload.
The current reduction-free reindex/pointwise slice validates plumbing but weakly
demonstrates fusion value. Pulling forward the researched, still-proposed strict
serial f32 `Sum` profile would prove a one-dispatch/no-intermediate result
against a split baseline while leaving parallel/reassociated/SIMD/multi-pass
reductions deferred. Second, after that workload is concrete, Tom explicitly
authorizes, narrows, or declines its implementation. Crate layout and MSRV are
follow-up decisions only if implementation is authorized.
