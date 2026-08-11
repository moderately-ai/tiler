---
id: promote-the-symbolic-index-profile-to-a-public-boundary
title: Promote the sourced-extent and semi-affine index profile to a reviewed public boundary
status: done
priority: p1
dependencies: []
related: [implement-shapeenv-index-bindings, implement-shapeenv-core, decide-shapeenv-builder-attachment, decide-symbolic-extent-error-siting, decide-domain-dimension-symbolic-view, decide-shapeenv-module-path, represent-semi-affine-index-expressions-in-the-ir]
scopes: [implementation/ir, implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, api]
---
## User-visible outcome

An external frontend can construct one immutable ShapeEnv-backed index region, inspect every dimension through a total sourced-extent view, and express proven-positive semi-affine coefficients and divisors through one coherent public vocabulary. Static callers retain a simple constructor, invalid source replacement is unrepresentable, unsupported analyses decline explicitly, and Tom reviews one exact module/type/call-site boundary before acceptance.

## Recorded authority

Tom approved promotion in principle on 2026-07-25. The implementation remained private because its constructor, error, extent-view, module-path, and semi-affine divisor shapes were still duplicated across five decision tickets. Those are not independent compatibility surfaces: the same sourced-extent authority must serve construction, inspection, and semi-affine expressions or the public API mints competing constant-or-symbol types.

## Implementation keys

- Keep the existing static `IndexRegionBuilder::new(registry)` path. Add a distinct `IndexRegionBuilder::new_with_shape_environment(registry, Arc<ShapeEnv>)` constructor so the environment is fixed before any symbolic dimension exists and cannot be replaced. Remove the current repeatable consuming `with_shape_environment` draft rather than preserving two attachment paths. Do not add `Option<ShapeEnv>` to every static call site.
- Preserve layered typed errors. `SymbolicExtentError` distinguishes source-environment refusal, structural index refusal, and shape-vocabulary refusal without reporting one authority's limit under another name. Provide ergonomic conversion only where it preserves those causes.
- Publish one total `SourcedExtent`-style view for `DomainDimensionRef`. Do not expose a pair of optional accessors whose totality is held only by a test and can silently fail when a third source kind arrives.
- Re-export the accepted subset flat from `tiler_ir::shape` and the corresponding index surface from its existing public module, matching the repository's explicit re-export precedent. Keep private constraint internals private; do not publish a module merely because its file exists.
- Use the same sourced constant-or-symbol vocabulary for `FloorDiv` and `Modulo` divisors. Do not double public variants into static and symbolic pairs or mint a second divisor enum. A cheap `as_constant`/typed refusal preserves affine-only pass ergonomics.
- Add `IndexExprClass::SemiAffine` and handle every internal exhaustive classification site. Symbolic divisor positivity comes only from semantic ShapeEnv constraints, never a variant guard. A pass that cannot analyze the nonlinear class declines with a typed reason rather than approximating.
- Keep the public type as narrow as the admitted profile: a sourced extent is not a general expression tree, and promotion does not imply arbitrary nonlinear ShapeEnv solving.
- Enumerate the complete external-construction dependency closure in the draft: the accepted `ShapeEnv` builder/model, `ShapeSymbol`, frontend-constructible binding sources and typed keys, the bounded constraint vocabulary needed to prove positivity, identity/read-only views, and every error required to construct or inspect the accepted subset. Keeping any required link `pub(crate)` would make the advertised frontend path unreachable.

## Evidence

- A second environment cannot replace the first before or after symbolic construction.
- Static construction remains unchanged and allocation behavior does not regress.
- Source, structure, and shape failures retain distinct typed causes.
- Static and symbolic dimensions are both visible through one total view; perturbing the source variants forces exhaustive consumers to update.
- Static and symbolic divisors share one representation, unproved positivity rejects, guards alone do not prove positivity, and at least one affine-only analysis declines semi-affine input.
- Every internal `IndexExprClass` match is exhaustive without wildcard arms.
- Every new check is perturbed once and observed failing before restoration; targeted `tiler-ir` tests, per-package Clippy, and `make full` pass.

## Public review boundary

Before acceptance Tom reviews the exact constructors/typestate, exported module paths, `SourcedExtent`, `SymbolicExtentError`, semi-affine expression view, class, diagnostics, and representative frontend call sites. Remove or narrow draft `dead_code` allowances only for the accepted subset.

The exact review packet must include the full `ShapeEnv` dependency closure above, not only the two headline types. This is implementation plus promotion: before this work divisors were `u64`, classification was only `Affine | QuasiAffine`, and the construction types were crate-private.

**Nothing here is self-accepted.** The implementation below is a concrete draft. Tom reviews, before acceptance: `IndexRegionBuilder::new` and `new_with_shape_environment` and the absence of any environment setter; the exported paths listed under *Implementation outcome*; `SourcedExtent` and `SourcedShape` including that one vocabulary carries domain extents, boundary axes, and divisors; `SymbolicExtentError` and its three authorities and `From` conversions; `ExtentSourceError::DivisorNotProvedPositive`; `DomainDimensionRef::extent` and `TensorRef::shape` replacing the optional-accessor pair; `IndexExprClass::SemiAffine` and `join`; `ExtentSources`' read-only queries; `LoweringEmitError::Extent` and `UnsupportedRegionFeature::SymbolicIndexDivisor`; and the representative frontend call sites — the `divided_copy` fixture in `crates/tiler-ir/src/index/sourced.rs` and `a_semi_affine_divisor_is_declined_rather_than_resolved` in `crates/tiler-reference/tests/index_region_oracle.rs`, which authors an environment and a symbolic-divisor region through the public surface alone.

Draft `dead_code` allowances were removed only where the accepted subset made them false: the module-level allowances on `shape/env.rs` and `index/sourced.rs` and the per-item allowances on the builder's symbolic constructors, `VerifiedIndexRegion::extent_sources`, and the two borrowed views are gone because those items are now reachable. The one remaining allowance, on `index/predicate.rs`'s `expression_class_is_stateable`, is unrelated to this boundary and stays.

## Implementation outcome

**Scope correction, made rather than absorbed silently.** The ticket declared `implementation/ir` alone, and that under-declares the work it specifies. Changing `IndexExprView`'s divisor to the sourced vocabulary and replacing the optional-accessor pair with a total view moves types that `tiler-compiler` and `tiler-reference` consume, and the evidence clause "at least one affine-only analysis declines semi-affine input" names passes that live in those two crates and nowhere else. `implementation/compiler` and `implementation/reference` are therefore declared above. The edits in them are exactly the migration plus the two typed declines; no other behaviour in either crate changed.

**The exact public surface.** `tiler_ir::shape` re-exports flat from the still-`pub(crate)` `env` module: `SymbolScope`, `ShapeSymbol`, `InterfaceParameterKey`, `BindingSource`, `FactProvenance`, `RootBinding`, `ShapeEnvBuilder`, `ShapeEnv`, `ShapeEnvIdentity`, `ShapeEnvError`, `ExtentTerm`, `ExtentRelation`, `SemanticInputConstraint`, `GuardApplicability`, `VariantGuard`, `FragmentViolation`, `ConstraintConflict`, `ExtentInterval`. `tiler_ir::index` re-exports flat from the now-private `sourced` module: `SourcedExtent`, `SourcedShape`, `ExtentSources`, `ExtentSourceError`, `SymbolicExtentError`, `EXTENT_PHASE_CEILING`. `IndexRegionBuilder` gains `new_with_shape_environment`, `symbolic_dimension`, and `sourced_tensor`; `floor_div` and `modulo` take a `SourcedExtent` and return `SymbolicExtentError`. `DomainDimensionRef::extent` and `TensorRef::shape` replace `static_extent` and `static_shape`; `VerifiedIndexRegion::extent_sources` is public. `IndexExprClass` gains `SemiAffine` and a `join`. `tiler-compiler`'s `LoweringEmitError` gains `Extent(SymbolicExtentError)`; `tiler-reference`'s `UnsupportedRegionFeature` gains `SymbolicIndexDivisor`.

**Dependency-closure audit.** Every link an external frontend needs to construct and inspect the accepted subset is reachable: a scope (`SymbolScope::new`), a symbol (`ShapeSymbol::new`), all four binding sources with their governed keys (`InputKey`, `InterfaceParameterKey`, `TargetPropertyKey`, `Extent`) and `AvailabilityPhase` from `tiler_ir::program::abi`, `FactProvenance`, `RootBinding::new`, the builder's `declare`/`bind`/`require`/`guard`/`build`, the relation constructors and their `ShapeEnvError` refusals, and the read-only queries. `tiler-reference`'s new oracle test builds one end to end through the public surface alone, which is the check rather than the claim. Deliberately still crate-internal, each because publishing it would create a second authority rather than complete a path: the canonical encoders (`ShapeSymbol::encode`/`encoded_len`, `SourcedExtent::encode`/`encoded_len`, `SourcedShape::encode`/`encoded_len`), `SourcedShape::from_shape`/`sourced` (a boundary is stated through the builder, which enforces the index layer's rank and byte limits), and `ExtentSources::new` (a region acquires its environment once, at its constructor).

**Two named deviations from the implementation keys.** First, the ticket names a `as_constant` divisor projection; `SourcedExtent::as_static` already is that projection and adding a second near-identical accessor would be the duplicate the guidance warns against, so consumers call `as_static`. Second, the shape-vocabulary variant is `SymbolicExtentError::ShapeVocabulary`, not `Shape`: a public variant named `Shape` puts a second `Shape` in the crate's exported name table and rustc then stops printing the short path for the `Shape` type, which broke `tests/shape-evidence/fail/shape_array_rank_limit.stderr`. That was observed, bisected to this variant, and fixed by the rename rather than by re-blessing the golden.

**Not attempted.** Nothing widens the admitted profile: a sourced extent is still a literal or one declared symbol, symbolic divisor positivity still comes only from semantic input constraints, and no nonlinear `ShapeEnv` solving was added.

## Accepted (2026-07-31)

Tom accepted the exact surface as merged, including both recorded deviations (no `as_constant` beside `as_static`; `SymbolicExtentError::ShapeVocabulary` naming) and the disclosed scope widening to `implementation/compiler` and `implementation/reference` for the mechanical migration the evidence clause required. The index-region identity domain advance to v9 was verified on the merged tree by the full suite.

## Graph maintenance

- `decide-shapeenv-builder-attachment`, `decide-symbolic-extent-error-siting`, `decide-domain-dimension-symbolic-view`, `decide-shapeenv-module-path`, and `represent-semi-affine-index-expressions-in-the-ir` are superseded after their surviving requirements are preserved here.
- Keep `widen-shapeenv-factorization-fragment` deferred: promotion exposes the current decided fragment and does not imply fully launch-dynamic factorizations.
- Split only a genuinely independent implementation unit whose public types do not need to be reviewed with this boundary.
- Advance the affected ShapeEnv/index identity domains for symbolic divisor semantics and recompute pinned identities on the merged tree; visibility-only changes do not independently justify a version bump.

**Identity domains, as advanced.** `tiler.index-region.v8` moved to `v9`: a floor-division or modulo divisor now encodes as a tagged `SourcedExtent` where `v8` wrote eight raw bytes, so a *constant* divisor's bytes changed even though its meaning did not, and `encoded_index_node_len` reads the divisor's own length instead of a fixed thirteen. `tiler.shape-env.v3` did **not** move: no byte a shape environment encodes changed, and a domain that advanced for a visibility change alone would make two identical subjects carry different domains. No pinned identity fixture covers an index region — the exact check is `grep -rnE '"[0-9a-f]{32,}"' crates/`, whose hits are the schedule, semantic-registry, target-profile, and SHA-256 vectors, none of which folds index-region bytes — and all 1372 workspace tests plus the doc-tests pass unchanged on this branch, which is the measurement confirming the re-baseline rippled nowhere. The coordinator re-runs the same gate on the merged tree.

## Accepted-surface correction — 2026-08-11

The 2026-07-31 acceptance remains authoritative for the index construction/profile that was actually reviewed, but one supporting-surface premise is now proved false and must not be inherited by new consumers. `SourcedShape` was accepted with public enum variants and documentation claiming its crate-private constructors preserve one normalized spelling. At exact candidate `2f244dc7ff3a759d9688a482c27b48da70f37227`, safe external Rust can construct `SourcedShape::Sourced(vec![])` or an all-literal `Sourced` boundary directly. The former reaches a panic in semantic inference; the latter compares unequal to `Static` while encoding to the same canonical bytes. Private constructors never made the public variants private.

[`seal-and-validate-sourced-shapes-at-semantic-inference-boundaries`](seal-and-validate-sourced-shapes-at-semantic-inference-boundaries.md) owns the source-breaking opacity/normalization repair and preservation of existing admitted bytes. [`narrow-symbolic-inference-and-restore-host-owned-refusals`](narrow-symbolic-inference-and-restore-host-owned-refusals.md) owns the adjacent semantic-provider boundary. This correction does not revoke the accepted `ShapeEnv`/index profile, re-open the module-path decision, or authorize a compatibility fallback; it retires only the false claim that the currently public representation enforces normalization.
