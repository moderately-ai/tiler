---
id: prototype-semantic-reference-slice
title: Implement the serial Sum semantic and reference slice
status: in-progress
priority: p0
dependencies: [prototype-workspace-scaffold, prototype-shaped-value-api]
related: []
scopes: [implementation/ir, research/semantic-graph, research/shapes, implementation/workspace, research/documentation]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions, contracts/navigation]
paths: [AGENTS.md, CLAUDE.md, .gitignore]
tags: [implementation, prototype, semantics, vertical-slice]
claimed_from: todo
assignee: codex
lease_expires_at: 1784563216
---
Build and evaluate the complete target-independent input to the first value
proof:

- typed `f32` input, constant, multiply, add, strict serial `Sum`, and named
  output nodes;
- recoverable construction, immutable completion, graph and shape validation,
  canonical contributor order, and deterministic semantic identity;
- the normative host reference evaluator, including canonical arithmetic-NaN,
  signed-zero, infinity, subnormal, contraction-sensitive, empty-domain,
  singleton, overflow, and malformed-graph cases; and
- an ordinary public experimental Rust construction path whose types preserve
  the accepted semantic/physical and operation/property boundaries.

The slice succeeds when equivalent construction orders produce the same
identity and result, invalid programs produce specific typed diagnostics, and
the program can be consumed by the next compiler slice without any frontend,
optimizer, artifact, Metal, or runtime dependency.

This is the integration gate for the dependency-ordered owner/commit,
resolved-type registry, exact typed-handle, shape-evidence spike, and checked
shaped-value tickets. It does not reimplement those components. It migrates the
bounded evaluator and operation set onto them, versions semantic identity away
from the prototype's implicit graph-wide `f32` assumption, completes malformed
and numerical boundary cases, and proves the assembled public path.

## Current checkpoint

An initial untyped implementation, adversarial tests, semantic lifecycle ADR,
contributor bootstrap, and repository validation gates are present in the
working tree. The accepted API review exposed the prerequisite tickets above;
the current code is a tested draft rather than the completed slice. Tom's only
remaining blocking review is the consequential shape-evidence spelling after
its measurement spike. The
contributor bootstrap changes are cross-cutting proof infrastructure discovered
while implementing this slice; they do not add semantic product scope.

The first API-review decision is recorded in ADR 0059: exact nominal `Value<T>`
is the primary authoring capability over a runtime-typed heterogeneous graph;
`ValueId` has `unknown`, never `any`, semantics; and resolved numerical choices
remain explicit rather than ambient promotion. The current implementation must
be revised after the remaining typed-surface limitations are reviewed.

ADR 0060 additionally binds built-in and external Rust type markers through the
explicit frozen registry. Marker traits carry no semantic authority, duplicate
marker/key claims fail before graph construction, and process-local `TypeId`
lookup never enters durable identity.

ADR 0061 retains canonical `Value<T>` while adding optional graph-checked
`ShapedValue<T, E>` refinements and typed multi-value shape witnesses as early
core components. Shape evidence is non-authoritative, explicitly weakened,
checked when refined, and must delegate to the same builder-centered semantic
admission path. The initial bounded spike must cover rank and exact-static
evidence, forgery resistance, foreign-graph rejection, compile-fail behavior,
and compile-time cost before the public spelling is stabilized.

ADR 0062 defines `T` as one complete shape-independent resolved semantic value
type, not necessarily one primitive `TypeKey`. Primitive, parameterized complex,
and encoded-numeric/quantized values share `Value<T>`; the frozen registry binds
the marker to the complete canonical identity, while runtime quantization
parameters remain ordinary graph operands.

ADR 0063 keeps graph ownership as opaque runtime-checked handle metadata rather
than a mandatory Rust lifetime or generative brand. Every handle-consuming API
must reject foreign values or witnesses before indexing or mutation, preserve
builder state on failure, exclude owner tokens from durable identity, and test
coincident local indices plus owner-token exhaustion.

ADR 0064 requires successful commitment to prune unreachable draft state,
compact live storage, assign a distinct completed-program owner, and invalidate
all draft handles. Typed output selectors and retained provenance cross the
boundary without stabilizing arena indices. The internal commit remap keeps an
additive future `build_with_report` possible, but ordinary `build` does not pay
for or expose a mandatory correlation report.
