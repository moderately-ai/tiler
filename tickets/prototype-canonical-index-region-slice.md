---
id: prototype-canonical-index-region-slice
title: Implement the canonical index-region slice
status: review
priority: p0
dependencies: [prototype-shared-compiler-ir-ownership]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/workspace, project/tickets, contracts/foundation, contracts/navigation, contracts/decisions, contracts/numerics]
shared_scopes: []
paths: [.gitignore]
tags: [implementation, compiler-foundation, indexing]
claimed_from: todo
assignee: codex
lease_expires_at: 1784648951
---
Implement the public checked static-extent index-region profile needed by the
first supported operations: typed interned iteration expressions, specialized
structural scalar expressions, logical access relations, lexical reduction
dimensions, bounds and write-ownership witnesses, reachable compaction, and
canonical identity without target-specific scheduling.

## Outcome

The draft uses exact arbitrary-precision integer semantics, static parallel and
reduction dimensions, interval proofs plus resource-bounded finite fallback,
and a structural permutation rule plus bounded exhaustive fallback for
ordinary writes. It preserves F32 operand order and exact
constant/empty-identity bits, handles zero and rank-zero domains, requires one
complete write for every output boundary, rejects malformed or unproved
relations, and exposes only opaque verified products and borrowed views. The
first profile is explicitly out-of-place and F32-scalar-only.
Accesses carry explicit lexical evaluation domains; ordered tensor-boundary and
semantic-program correlation identities prevent equal-shaped bindings from
colliding. The correlation is derived from an authentic completed
`SemanticProgram`; arbitrary caller bytes are not accepted. Generic region
selection may later narrow this conservative whole-program dependency.

This verifier proves the structural index relation, not that the relation is a
correct implementation of a semantic operation. Operation capabilities emit
the relation and the compiler legality layer must retain separate evidence
binding it to the authoritative semantic region. Treating structural checks as
semantic sourceability would admit wrong-coordinate implementations and would
make external operations impossible to support soundly.

The accepted dynamic contract is not misrepresented as complete. The semantic
crate does not yet implement `ShapeEnv`, so symbolic root bindings,
semi-affine symbolic coefficients/divisors, and typed index-domain predicates
are split into `implement-shapeenv-index-bindings` and
`implement-index-domain-predicates`. This profile leaves additive public seams
for those authorities and rejects unsupported dynamic construction rather than
introducing an index-local duplicate.
