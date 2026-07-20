---
id: research-readiness-gate
title: Run the research-to-implementation readiness gate
status: done
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

Tom selected the strict serial f32 `Sum` value proof and authorized the bounded,
explicitly unstable implementation prototype. ADR 0055 records the scope:
semantic-to-runtime vertical evidence, not a broad or production Metal backend.
Parallel/reassociated/SIMD/multi-pass reductions remain deferred. Crate layout
and MSRV are the next prerequisite decisions before scaffolding.

## Outcome

- The architecture passed independent contradiction, provenance, and
  blank-agent acceptance audits.
- Identity, artifact-family delivery, numerical, cache, and runtime boundaries
  have accepted contracts and bounded executable evidence.
- ADR 0055 selects a strict serial f32 `Sum` with a resolved pointwise prologue
  as the first Metal-backed value proof.
- Tom authorized this bounded prototype phase; it does not stabilize APIs or
  authorize broader backend and operation coverage.
