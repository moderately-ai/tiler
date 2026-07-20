# Tiler design documentation

Tiler is an ahead-of-time compiler toolkit for tensor iteration spaces. It is
intended to accept declarative tensor programs, optimize them using a
semantic tensor graph plus hierarchical region/program planning, and emit
optimized kernels with a stable runtime contract. It selectively borrows
property-aware database optimizer techniques without copying a relational plan
shape. `candle-einops` is the first planned frontend and Candle Metal is the
first planned runtime integration.

This directory describes the design before implementation. Documents marked
**proposed** capture the current direction, not a compatibility promise. Open
choices are collected rather than silently resolved.

## Current design status

These documents are research and design proposals, not an established public
contract. ADR status is authoritative only for decisions explicitly reviewed
and accepted. Integration-specific choices for einops, Candle, Rust macros, and
Metal must not silently become constraints on the compiler toolkit core.

The settled product direction is a tensor-specific, consumer-independent
compiler toolkit with a public semantic graph boundary. Frontends translate
their languages into that graph; optimizers and backends operate without
depending on the originating syntax or consuming runtime. The inline Rust macro
and Metal AOT flow remain an important proposed first integration and
feasibility vehicle.

## Start here

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

## Supporting material

- [Terminology](glossary.md)
- [Open design questions](open-questions.md)
- [Lessons from `ug`](prior-art/ug.md)
- [Logical-graph and schedule-IR precedents](prior-art/logical-graphs-and-schedules.md)
- [Symbolic index and access model](research/indexing/index-access-model.md)
- [Target profiles and phased physical feasibility](research/target-profiles/physical-feasibility-model.md)
- [Architecture decisions](decisions/README.md)

## Document ownership

The documents are organized around contracts:

| Area | Establishes |
| --- | --- |
| Vision | Goals, non-goals, and success criteria |
| Architecture | Components and dependency direction |
| IR | Meaning and invariants at each lowering boundary |
| Operation extensions | Registry, identity, canonical data, trust, capability, and rewrite contracts |
| Optimizer | Equivalence rules, alternatives, and properties |
| Scheduling | Hardware mapping and fusion decisions |
| Backend | Translation into target source and compiled artifacts |
| ABI/runtime | How compiled code is validated, bound, and dispatched |
| Correctness | Semantic authority and verification strategy |

Changes that cross one of these contracts should update its document and, when
the choice is durable, add an architecture decision record.
