---
id: prototype-index-region-reference-oracle
title: Implement the generic IndexRegion reference oracle
status: in-progress
priority: p0
dependencies: [prototype-canonical-index-region-slice, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/reference]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, reference, indexing, oracle]
claimed_from: todo
assignee: agent-prototype-index-region-reference-oracle
lease_expires_at: 1784823820
---
Implement a slow generic checked IndexRegion oracle in tiler-reference. Resolve
registered scalar evaluators without downcasting, execute ordered multi-result
SSA and N-state lexical reductions, preserve exact dtype bits and empty-domain
semantics, and fail closed for missing authority. Fusion legality must compare
against this independent path rather than a graph-specific host expression.

Index-expression arithmetic is a decided constraint, not an open choice.
`tiler_ir::index::IndexInteger` deliberately exposes no public arithmetic (only
`from_i128`/`from_u64`/`from_sign_magnitude`/`to_sign_magnitude`), while
coefficients admit large magnitudes and intermediates can cancel past `i128`.
Implement exact evaluation inside `tiler-reference` over the sign-magnitude
representation, rejecting oversized intermediates with a typed fail-closed
error rather than saturating or wrapping. Do not widen `tiler-ir`'s public
surface to obtain arithmetic: this ticket does not hold `implementation/ir`,
that scope is contended by the p0 spine, and exposing checked `IndexInteger`
arithmetic is a separate reviewed boundary decision. Adding a bignum dependency
is permitted — `implementation/cargo-lock` is declared for exactly that — but
prefer the bounded in-crate path if it satisfies the admitted domain.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

`tiler-reference` now contains a generic slow oracle for verified index regions, in two new private modules re-exported from the crate root: `arithmetic` (bounded exact integer evaluation) and `oracle` (capability registry plus evaluator).

**Arithmetic (the decided constraint).** `arithmetic::ExactInteger` implements sign-and-magnitude arithmetic over little-endian 64-bit limbs directly on `IndexInteger::to_sign_magnitude`, with checked addition, checked multiplication, and Euclidean floor division/modulo by a positive `u64`. No bignum dependency was added and `Cargo.lock` is unchanged: the bounded in-crate path covers the admitted domain, and it keeps the oracle's arithmetic independent of the arbitrary-precision library the structural verifier itself uses, so one shared defect cannot make the oracle agree with a wrong coordinate. Every evaluated index value is admitted against `MAX_INDEX_INTEGER_BYTES + 16`, derived from what one normalized linear combination over governed coefficients and `u64`-valued children can produce; a larger intermediate (for example scaling an already maximal floor division) is rejected as `IndexRegionEvaluationError::ResourceExceeded { resource: IndexIntegerBytes, .. }` rather than saturated or wrapped. A multiplication whose product would exceed the bound is rejected from operand bit lengths before any quadratic work.

**Evaluation.** `IndexRegionEvaluator::evaluate` revalidates the region against the caller-supplied `FrozenScalarRegistry` first and returns the resulting `ScalarAuthorityEvidence` alongside the ordered output tensors, so a region the authority cannot admit never reaches an executable capability. Scalar applications resolve by `(ScalarOpKey, ReferenceSignature)` against registered `Arc<dyn ScalarReferenceOperation>` implementations; there is no downcast and no `Any`. Each capability stores the reached scalar-definition projection and its admitting scalar provider at registration and re-projects them against the region's authority at evaluation, so a changed definition or admission provider rejects as `ScalarCapabilityAuthorityMismatch`. Callback attributes carry registered schema defaults resolved, matching what construction-time inference observes. Reductions execute the declared `ExactLexicographicLeftFold`, evaluating initial state at the enclosing point and contributors per bound point, with an empty bound domain yielding the initial state. Coordinates, write coverage, and write injectivity are checked independently of the structural verifier's own proofs: a duplicated or missing output element rejects as `DuplicateWrite`/`IncompleteWrite`.

**Bounded profile.** Compound (non-dense) boundary or scalar representations, symbolic extents/shapes, and any future traversal, expression form, or value definition reject explicitly through `UnsupportedRegionFeature`. Governed resources are evaluation steps, combined host recursion depth, evaluated index magnitude, and aggregate output elements. Scalar values are represented as rank-zero `Tensor` values so the existing registered value validators apply unchanged; a validator that imposes a whole-tensor (rather than element-wise) invariant would therefore reject produced scalars, which is a consequence of this representation choice rather than an accident.

**Remaining gates.**

- Every new public item is a draft pending Tom's boundary review: `ScalarReferenceOperation`, `ScalarReferenceRequest`, `ScalarReferenceOutputs`, `ScalarReferenceRegistryBuilder`, `FrozenScalarReferenceRegistry`, `CanonicalScalarReferenceRegistryIdentity`, `ScalarReferenceRegistryError`, `IndexRegionEvaluator`, `IndexRegionAuthority`, `IndexRegionInput`, `IndexRegionEvaluation`, `IndexRegionEvaluationError`, `IndexReferenceResource`, `UnsupportedRegionFeature`, and `ScalarCapabilityAttribution`. The registry deliberately takes an admitting `ProviderIdentity` per registration (mirroring `tiler_ir::index::ScalarRegistryBuilder`) instead of duplicating the semantic reference registry's provider-transaction surface.
- `IndexRegionAuthority` pairs the region's scalar and semantic registries because `FrozenScalarRegistry` exposes no accessor for the semantic registry it was composed with. The oracle checks that pairing by comparing `ScalarAuthorityEvidence::semantic_snapshot` with the supplied registry's snapshot identity. A `tiler-ir` accessor would remove the parameter; that needs `implementation/ir`.
- Tests live in `crates/tiler-reference/src/oracle.rs` rather than a dedicated integration target. `scripts/check_workspace.py` enumerates admitted integration-test targets in `EXPECTED_TESTS`, and that file is in `implementation/workspace`, which this ticket does not hold. Promoting the oracle proof to a downstream-style integration test is a follow-up for whoever holds that scope.
- `docs/correctness-and-testing.md` still says the generic slow evaluator "remains owned by" this ticket. Updating that sentence needs `contracts/numerics`, which this ticket does not hold.
- The oracle's magnitude bound is exercised by unit tests on the arithmetic path. No verified region in the current profile is known to require an intermediate above it; a region that does would have to pass structural verification through the exhaustive rather than the interval bounds path.
