---
schema: "tiler-doc/v1"
id: "tiler.portal.docs"
kind: "portal"
title: "Tiler documentation"
topics: ["orientation", "architecture"]
---

# Tiler documentation

Tiler is an experimental ahead-of-time compiler toolkit for tensor iteration
spaces. The documentation separates normative contracts, durable decisions,
research evidence, executable experiments, and proposed work.

## Ten-minute orientation

1. [Project status](status.md): what is decided, measured, unimplemented, and
   currently awaiting a decision.
2. [Vision and scope](vision.md): product goals and non-goals.
3. [Design map](design-map.md): the compiler layers and the owner of each
   question.
4. [System architecture](architecture.md): components and dependency direction.

Use the [glossary](glossary.md) whenever a typed compiler term is unfamiliar.

## Navigate by task

- **Understand semantic meaning:** [IR](ir.md),
  [operation extensions](operation-extensions.md), and
  [numerical semantics](numerical-semantics.md).
- **Understand optimization:** [optimizer](compiler/optimizer.md),
  [fusion and scheduling](compiler/fusion-and-scheduling.md), and
  [cost model](compiler/cost-model.md).
- **Understand artifacts and execution:** [artifact ABI](artifact-abi.md),
  [Metal](backends/metal.md), [frontends](integration/frontends.md), and
  [Candle](integration/candle.md).
- **Inspect accepted choices:** use the [thematic ADR index](decisions/README.md).
- **Audit evidence or reproduce a claim:** use the
  [research catalog](research/README.md) and
  [experiment catalog](../spikes/README.md).
- **Continue work:** read [work tracking](work-tracking.md) and
  [AGENTS.md](../AGENTS.md).

## Deep design sequence

This complete sequence is for a detailed architecture review, not a prerequisite
for locating one fact:

1. [Vision and scope](vision.md)
2. [System architecture](architecture.md)
3. [IR stack and invariants](ir.md)
4. [Operation extensions](operation-extensions.md)
5. [Numerical semantics](numerical-semantics.md)
6. [Optimizer model](compiler/optimizer.md)
7. [Fusion and scheduling](compiler/fusion-and-scheduling.md)
8. [Cost model](compiler/cost-model.md)
9. [Frontend integration](integration/frontends.md)
10. [Metal AOT backend](backends/metal.md)
11. [Artifact and kernel ABI](artifact-abi.md)
12. [Candle integration](integration/candle.md)
13. [Correctness and testing](correctness-and-testing.md)
14. [Roadmap](roadmap.md)

## Authority model

- Accepted ADRs govern durable choices and rationale.
- Normative contracts own detailed current behavior within their stated status.
- Research reports establish evidence and limitations, not authority by
  themselves.
- Spikes establish only the bounded claim they measured or modeled.
- Ticketsplease owns live work status; the roadmap is proposed progression.

The exact metadata and relationship rules are defined in
[documentation metadata](document-metadata.md). A disagreement among an
accepted ADR, its normative owner, and indexed evidence is a documentation bug,
not a choice left to the reader.

For a `mixed` contract, accepted ADRs and explicitly labeled accepted sections
are authoritative; otherwise unmarked field-level schemas and API detail remain
proposed. This fail-closed convention avoids making a whole evolving design
normative merely because it incorporates accepted boundaries.
