---
schema: "tiler-doc/v1"
id: "tiler.portal.design-map"
kind: "portal"
title: "Design map"
topics: ["architecture", "orientation"]
---

# Design map

The shortest useful model of Tiler is:

```text
frontend program
  -> semantic operation/value graph
  -> symbolic iteration and access
  -> legal fusion/program alternatives
  -> target-aware schedules and costing
  -> structured kernels
  -> backend payload and neutral artifact program
  -> guarded runtime execution or fallback
```

Each arrow is a verifier boundary. Semantic meaning flows inward to physical
planning; target capabilities never redefine tensor semantics.

## Navigate by question

| Question | Normative owner | Evidence route |
| --- | --- | --- |
| What does a tensor program mean? | [IR](ir.md), [numerical semantics](numerical-semantics.md) | [semantic, shape, and numerical research](research/README.md) |
| How may operations extend? | [Operation extensions](operation-extensions.md) | extension/API research in the [catalog](research/README.md) |
| Which alternatives are legal and chosen? | [Optimizer](compiler/optimizer.md), [fusion and scheduling](compiler/fusion-and-scheduling.md) | optimizer and schedule research in the [catalog](research/README.md) |
| What is target feasibility? | [IR](ir.md) owns normalized requirements; backend contracts own device realization | target-profile and placement research |
| What owns kernel programs and buffers? | [IR](ir.md) owns the compiler model; [Artifact ABI](artifact-abi.md) owns serialization | program-planning research |
| What is embedded and identified? | [Artifact ABI](artifact-abi.md) | artifact, cache, embedding, and Apple research |
| When may fallback occur? | [Artifact ABI](artifact-abi.md) owns neutral routing; [Candle integration](integration/candle.md) owns the adapter | runtime execution and validation research |
| What is accepted? | [Thematic ADR index](decisions/README.md) | each ADR's evidence links |
| What remains? | [Status](status.md), [open questions](open-questions.md), [roadmap](roadmap.md) | live ticketsplease board |

For exact terminology, use the [glossary](glossary.md). For source evidence and
reproduction, continue through the [research](research/README.md) and
[experiment](../spikes/README.md) catalogs.
