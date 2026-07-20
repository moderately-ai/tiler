---
schema: "tiler-doc/v1"
id: "ADR-0064"
kind: "decision"
title: "Compact semantic programs at commitment"
topics: ["rust", "semantics", "graph", "api", "provenance"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir"]
evidence: ["tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.semantic-graph.contract-memo"]
ticket: "prototype-semantic-owner-and-commit"
---

# 0064: Compact semantic programs at commitment

**Status:** accepted

## Context

An append-only draft may contain abandoned pure operations and construction-
local arena ordering. Preserving its arenas unchanged through `build` lets
draft handles continue to resolve, but makes completed-program iteration,
counts, memory use, and downstream pass obligations depend on dead draft state.
It also conflicts with the accepted invariant that a canonical semantic program
contains exactly the transitive closure reachable from its declared outputs.

Draft-to-program correlation remains useful for typed outputs, source
diagnostics, explainability, and possible future frontend tooling. Those needs
should not turn arena indices into stable identities or require dead operations
to remain part of compiler input.

## Decision

A successful consuming `build` verifies the draft, selects the output-reachable
semantic closure, and compacts it into dense completed-program storage. The
completed `SemanticProgram` receives a distinct owner identity. Every draft
`Value<T>`, `ShapedValue<T, E>`, `ValueId`, operation handle, and witness is
therefore invalid against the completed program, even when its subject was
retained.

Commitment computes an internal deterministic old-to-new mapping as required to
rewrite edges, interfaces, constraints, witnesses, and provenance. Ordinary
`build` may discard that mapping after construction. Its existence does not
enter semantic identity, and its draft indices are never serialized as durable
references.

Values intentionally crossing the commitment boundary use typed stable
interface selectors, conceptually `Output<T>`, derived from the declared output
key or position and exact resolved value type. Resolving such a selector against
a completed program validates the interface identity and registry binding, then
returns a new completed-program-owned `Value<T>`. A selector is not a value
handle and does not expose an arena index.

Retained operations and values preserve their admitted source and explanation
provenance according to the existing identity split. Removed draft operations
are not queryable through `SemanticProgram`. Tooling that needs removal details
may use draft inspection before commitment or a future explicit build report.

The public API reserves additive future entry points such as
`build_with_report` returning a `BuildOutcome { program, report }`. A governed
report may classify draft subjects as retained, rewritten, coalesced, or
removed and map retained subjects to completed handles or provenance records.
Ordinary `build(self) -> Result<SemanticProgram, ProgramBuildError>` remains the
minimal path; adding a report later does not change its semantics or signature.

Compaction is a construction normalization, not an optimizer freedom. It may
remove only output-unreachable pure draft state and perform representation-only
renumbering unless a future separately reviewed commitment-normalization
contract admits more. Semantic rewrites, common-subexpression elimination, and
physical planning remain in their existing later layers.

## Consequences

- Completed-program storage, iteration, and counts describe exactly the live
  semantic graph rather than draft residue.
- Draft handles cannot accidentally become stable cross-phase identity.
- Typed output selectors preserve ordinary Rust ergonomics for declared results.
- Source correlation survives through explicit provenance, while detailed
  draft remapping can be added through a build report without changing
  `SemanticProgram`.
- The prototype must replace its current unchanged-arena move, assign a new
  completed owner, and test dead-node pruning, dense remapping, output selector
  typing, provenance retention, and draft-handle rejection.

## Alternatives considered

Moving unchanged arenas preserves handles but contaminates completed input with
dead state and makes every consumer reachability-aware. Keeping holes or a
permanent remap inside `SemanticProgram` preserves live draft handles at ongoing
memory and lookup cost while encouraging them to act as stable IDs. Returning a
mandatory full build report taxes every caller and commits to a correlation
schema before a concrete frontend requires it.

## Traceability

The [IR contract](../ir.md) owns commitment, handles, interfaces, provenance,
and reachability. The [architecture contract](../architecture.md) owns the
handoff from editable draft to verified compiler input.
