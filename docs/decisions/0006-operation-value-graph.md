---
schema: "tiler-doc/v1"
id: "ADR-0006"
kind: "decision"
title: "Model semantic programs as operation/value graphs"
topics: ["semantics", "graph", "ir"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.semantic-graph.contract-memo"]
ticket: "semantic-graph-contract"
---

# 0006: Model semantic programs as operation/value graphs

**Status:** accepted

## Context

Tensor programs naturally contain fan-out, shared producers, several named
results, and operations with more than one result. Logical operation boundaries
must remain distinct from physical materialization and fusion boundaries.
Calling the representation a tree, a single-output plan, or a hypergraph alone
obscures some of these requirements.

Compiler precedents commonly expose operations and individually typed values.
A value with several consumers can be viewed mathematically as a multi-edge or
hyperedge, while an explicit value entity provides simpler use-def chains and
multi-result handling.

## Decision

Represent a semantic program as a pure acyclic operation/value graph:

- an operation has a stable operation key, ordered operands, canonical semantic
  attributes, and one or more ordered results in the initial pure graph;
- every value is an input or exactly one operation result and has an inferred
  tensor type;
- values may have several consumers;
- program results are an ordered named list of value references rather than
  synthetic `Output` operations;
- operations may have several individually addressable result values;
- canonical identity excludes arena IDs, insertion order, source spans,
  registry addresses, and derived use lists;
- only result-reachable pure operations participate in canonical identity.

Named atomic tensor operations remain visible in the semantic graph until
region exploration chooses a physical implementation. A fused scalar
expression is formed in iteration/access lowering rather than assumed at
frontend ingestion. A hypergraph may index overlapping region candidates, but
is not the public semantic or physical-program IR.

The initial graph is effect-free. Stateful or mutating operations require a
future explicit effect/resource-token and alias model rather than textual order
or a boolean side-effect annotation.

## Consequences

- Separate `Multiply`, `Add`, `Gelu`, and `Reduce` nodes do not imply
  intermediate buffers.
- Shared values, multiple graph results, and multi-result operations are
  represented without tuples invented solely for IR convenience.
- Region formation and materialization remain physical choices.
- Extensions lacking required semantic or lowering capabilities become
  conservative boundaries rather than being transformed on trust.
- Graph construction, verification, canonical serialization, and hashing need
  deterministic operation/value handling.

The supporting precedents, invariants, identity rules, and worked multi-output
examples are recorded in the
[semantic graph contract memo](../research/semantic-graph/contract-memo.md).

## Alternatives considered

A node-only DAG conflates an operation with its result and breaks for
multi-result operations. A single composite `Map` at ingestion hides named
operation semantics and prematurely chooses a fused expression boundary. A
literal hypergraph API can represent fan-out but does not replace typed values,
use-def chains, or explicit region-boundary identity.
