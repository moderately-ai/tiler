---
id: promote-the-symbolic-index-profile-to-a-public-boundary
title: Promote the sourced-extent and semi-affine index profile to a reviewed public boundary
status: todo
priority: p1
dependencies: []
related: [implement-shapeenv-index-bindings, implement-shapeenv-core, decide-shapeenv-builder-attachment, decide-symbolic-extent-error-siting, decide-domain-dimension-symbolic-view, decide-shapeenv-module-path, represent-semi-affine-index-expressions-in-the-ir]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, api]
---
## User-visible outcome

An external frontend can construct one immutable ShapeEnv-backed index region, inspect every dimension through a total sourced-extent view, and express proven-positive semi-affine coefficients and divisors through one coherent public vocabulary. Static callers retain a simple constructor, invalid source replacement is unrepresentable, unsupported analyses decline explicitly, and Tom reviews one exact module/type/call-site boundary before acceptance.

## Recorded authority

Tom approved promotion in principle on 2026-07-25. The implementation remained private because its constructor, error, extent-view, module-path, and semi-affine divisor shapes were still duplicated across five decision tickets. Those are not independent compatibility surfaces: the same sourced-extent authority must serve construction, inspection, and semi-affine expressions or the public API mints competing constant-or-symbol types.

## Correctness-derived design

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

The exact review packet must include the full `ShapeEnv` dependency closure above, not only the two headline types. This is implementation plus promotion: current divisors are `u64`, classification is only `Affine | QuasiAffine`, and the construction types remain crate-private.

## Graph maintenance

- `decide-shapeenv-builder-attachment`, `decide-symbolic-extent-error-siting`, `decide-domain-dimension-symbolic-view`, `decide-shapeenv-module-path`, and `represent-semi-affine-index-expressions-in-the-ir` are superseded after their surviving requirements are preserved here.
- Keep `widen-shapeenv-factorization-fragment` deferred: promotion exposes the current decided fragment and does not imply fully launch-dynamic factorizations.
- Split only a genuinely independent implementation unit whose public types do not need to be reviewed with this boundary.
- Advance the affected ShapeEnv/index identity domains for symbolic divisor semantics and recompute pinned identities on the merged tree; visibility-only changes do not independently justify a version bump.
