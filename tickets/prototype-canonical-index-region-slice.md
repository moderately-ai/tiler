---
id: prototype-canonical-index-region-slice
title: Implement the canonical index-region slice
status: in-progress
priority: p0
dependencies: [prototype-shared-compiler-ir-ownership, correct-semantic-identity-layering, harden-semantic-registry-and-program-construction]
related: [harden-compiler-verifier-subject-binding-and-totality]
scopes: [implementation/ir, implementation/compiler, implementation/workspace, project/tickets, contracts/foundation, contracts/navigation, contracts/decisions, contracts/numerics, implementation/reference, research/indexing, contracts/optimizer, contracts/artifacts, research/artifacts, research/extensions]
shared_scopes: []
paths: [.gitignore]
tags: [implementation, compiler-foundation, indexing]
claimed_from: todo
assignee: gpt-sol-index
lease_expires_at: 1784741818
---
Implement the public checked static-extent index-region profile needed by the
first supported operations: typed interned iteration expressions, a generic
typed scalar operation/value SSA graph, logical access relations, lexical
reduction dimensions, bounds and write-ownership evidence, reachable
compaction, and canonical identity without target-specific scheduling.

## Required outcome

The implementation must use exact arbitrary-precision integer semantics,
static parallel and reduction dimensions, interval proofs plus
resource-bounded finite fallback, and a structural permutation rule plus
bounded exhaustive fallback for ordinary writes. It must handle zero and
rank-zero domains, require one complete write for every output boundary,
reject malformed or unproved relations, and expose only opaque verified
products and borrowed views. The first access profile is explicitly
out-of-place.

Scalar computation must not be a closed `F32` expression enum. Model it as
typed scalar operation/value SSA with a distinct namespaced and versioned
`ScalarOpKey`, bounded host-canonical attributes, ordered operands, and one or
more ordered results carrying complete `ResolvedValueType`. A checked frozen
scalar-definition registry owns arities, attribute schema, normative identity,
and deterministic result inference for ordinary scalar applications.
Providers emit only through the checked builder; asserted result types,
unchecked opaque payloads, `Any`, and downcasting are forbidden.

Reduction is a structural nested region, not an ordinary scalar application.
It owns ordered lexical reduction dimensions, initial state, contributors, a
checked nested scalar operation/value body, and ordered results. The body
receives typed state and contributor parameters and yields the next state, so
N-state, multi-operation reducers such as argmax fit without redesign. The
first supported traversal is an exact lexicographic left fold whose empty
result is its initial state. Alternative ordering freedoms remain explicit
later numerical/legality contracts rather than being implied by a combiner.

Registry fixtures must prove zero-operand constants, ordinary applications,
multi-result operations, and exact serial reduction through at least one
non-`f32` external nominal dtype. The downstream initial executable profile
may remain governed strict `f32`; that is a capability support limit, not an
intrinsic scalar-IR type restriction. F32 convenience APIs, if retained, must
delegate to the generic builder rather than define canonical storage.

Accesses carry explicit lexical evaluation domains and ordered typed tensor
boundaries. Canonical `IndexRegion` identity commits only to structural index,
access, scalar, constraint, and output content. It excludes builder ownership,
arena numbering, provider addresses, callbacks, proof caches, target choices,
and semantic-program or semantic-region correlation identity.

This verifier proves the structural index relation, not that the relation is a
correct implementation of a semantic operation. Operation capabilities emit
the relation and compiler-owned legality evidence must separately bind the
generated region to its selected authoritative semantic source. That evidence
consumes separately revalidated, region-bound scalar authority evidence and
adds selected lowering-provider provenance. The scalar receipt keeps reached
provider-independent definitions distinct from provider-attributed admission
provenance; neither receipt alone proves semantic equivalence.
Treating structural checks or a correlation identity as semantic sourceability
would admit wrong-coordinate implementations and would make external
operations impossible to support soundly.

Adversarial review found one remaining authority obligation. Scalar authority
evidence must separately retain semantic type-definition and type-admission
projections for every boundary type, ordinary SSA result, reduction state,
contributor, result, and every `Type` or `FloatBits` dependency reachable from
scalar definition attributes, defaults, facts, and conformance values. It must
also retain complete scalar-registry snapshot provenance as compilation-
environment identity without admitting that snapshot into structural
`IndexRegion` identity. Tests must distinguish type-provider-only and scalar-
provider-only revisions and prove that neither can disappear behind an
otherwise identical structural region.

The accepted dynamic contract is not misrepresented as complete. The semantic
crate does not yet implement `ShapeEnv`, so symbolic root bindings,
semi-affine symbolic coefficients/divisors, and typed index-domain predicates
are split into `implement-shapeenv-index-bindings` and
`implement-index-domain-predicates`. This profile leaves additive public seams
for those authorities and rejects unsupported dynamic construction rather than
introducing an index-local duplicate.

## Fixed-point audit corrections

The codebase audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd`
identified additional obligations owned by this still-open ticket:

- Make domain cardinality zero-absorbing before overflow checks. A domain such
  as `[u64::MAX, 2, 0]` is empty, and empty accesses must reach the vacuous
  proof path rather than fail a proof budget.
- Alpha-canonicalize iteration dimensions. Declaration order, arena numbering,
  and bound-variable spelling must not change structural identity; every
  access domain, free expression, and reduction reference must encode the
  canonical renaming.
- Enforce operand, result, state, contributor, yield, scalar-depth, expression-
  depth, integer-magnitude, registry, retained-byte, and proof-plan budgets
  before cloning, encoding, callback work, or aggregate allocation. Index
  expression traversal must be iterative or explicitly depth-bounded.
- Make failed nested/reduction builder operations transactional with respect to
  builder identifiers and observable allocation state.
- Distinguish scalar expression depth from scalar value-count diagnostics.
- Add `#[non_exhaustive]` or an equivalent forward-compatible inspection
  boundary to public index/scalar reserved-variant views, and replace exhaustive public
  struct literals such as `ScalarInferenceRequest` and
  `ScalarOperationContract` with additive construction APIs.
- Implement `Error::source` for nested scalar/index causes and bound provider
  diagnostics.

Acceptance tests must include late-zero domains, alpha-equivalent regions built
in different orders, oversized and infinite inputs, deep expression trees,
failed nested-builder reuse, public downstream compile tests for additive API
evolution, and complete nested error chains.

Transactional rollback covers graph-visible handles, arenas, retained work,
and canonical semantic state. A failed reducer closure deliberately consumes
its private anti-forgery owner nonce: body-local handles can escape the closure,
so rewinding or reusing that nonce could make a stale failed-attempt handle
valid in a later reduction. These private nonces are not structural identity or
otherwise observable semantic state.
