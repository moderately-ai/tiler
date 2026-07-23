---
schema: "tiler-doc/v1"
id: "ADR-0069"
kind: "decision"
title: "Use a general compilation boundary with explicit support failures"
topics: ["program-planning", "compiler-api", "capabilities", "extensions"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.optimizer"]
evidence: ["tiler.research.program-planning.general-compilation-boundary"]
ticket: "prototype-target-neutral-baseline-slice"
---

# 0069: Use a general compilation boundary with explicit support failures

**Status:** accepted

## Context

The first executable compiler model supports one materialized pointwise plus
strict serial-`Sum` graph. A public entry point named for that graph would
accurately describe current coverage, but would turn a temporary vertical
slice into the compiler's abstraction. Conversely, a general entry point must
not suggest that every valid semantic program can already be lowered.

Tiler already separates semantic representability, operation capabilities,
target feasibility, and product support. Its frozen registry and compilation
request provide the information needed to resolve those layers explicitly.

## Decision

Expose one general consumer-independent compilation boundary over
`SemanticProgram` and explicit request inputs. Do not expose graph-specific
compiler entry points, an `experimental` namespace, or a serial-Sum support
profile.

The compiler resolves installed capabilities and either returns general
target-neutral program products or a typed outcome. Failure classes distinguish
at least invalid requests, valid programs lacking a required compilation
capability, intrinsically or target-infeasible plans, exhausted bounded search,
and failures of compiler-produced IR verification. Unsupported cases fail
closed and are never approximated merely to retain a fast path.

Keep the first serial-Sum implementation private as a strategy, conformance
fixture, and explain identity. Its fixed region, stage, entry, and buffer
cardinalities are not public request or result invariants. Generalize those
seams before exposing the compiler interface.

Do not add a selectable support-policy type until there are multiple
deliberately maintained policies or an independent certification or
compatibility requirement. The compiler/provider revisions, frozen registry,
target, numerical contract, budgets, and options remain explicit identity
inputs.

## Consequences

- Frontends and external operation providers use the same compiler boundary as
  built-ins.
- Adding operation coverage does not require a new top-level entry point or
  migration away from a graph-specific module.
- Valid semantic programs can receive precise unsupported-capability or
  infeasibility diagnostics without being called malformed.
- The current executable model must remain private until its outer request,
  result, and failure seams no longer encode the serial-Sum cardinalities.
- A future support-policy abstraction remains compatible, but must be justified
  by a real selectable contract rather than speculative versioning.

## Alternatives considered

A public serial-Sum profile makes current coverage obvious but contaminates
long-term API identity. An `experimental` namespace communicates instability
without correcting that semantic mismatch. A support-policy selector is useful
for multiple maintained envelopes, but premature when compiler and provider
identity already determine the only implemented envelope.
