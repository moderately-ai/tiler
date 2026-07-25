---
id: implement-shapeenv-index-bindings
title: Bind ShapeEnv sources into canonical index regions
status: review
priority: p1
dependencies: [implement-shapeenv-core, implement-shapeenv-constraints]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing]
claimed_from: todo
assignee: agent-shapes2
lease_expires_at: 1785002816
---
Extend canonical index domains and expressions with sourceable ShapeEnv
InputDimension, InterfaceParameter, and phased TargetProperty bindings. Preserve
mathematical-integer identity, phase ordering, guards/proofs, and explicit
rejection of free, ambiguous, tensor-data-derived, or too-late sources. Do not
create an index-local duplicate symbol authority.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

**Done for index domains; tensor boundaries and semi-affine coefficients are split, not implied.** `crates/tiler-ir/src/index/sourced.rs` is new and `crates/tiler-ir/src/index/{builder,model}.rs` and `crates/tiler-ir/src/shape/env{,/constraint}.rs` changed. Full repository gate green.

### The defect found first: `BindingSource` had a second vocabulary

**Fact.** [ADR 0008](../docs/decisions/0008-typed-root-bindings.md) is accepted and names the root-binding classes in a code block: `Static`, `InputDimension`, `InterfaceParameter`, `TargetProperty`. `implement-shapeenv-core` implemented them as `StaticValue`, `InputMetadata`, `CallerParameter`, `TargetProperty`, taking the names from `docs/ir.md`'s descriptive prose ("static values, input metadata, caller parameters") rather than from the ADR's normative list. This ticket's own text uses the ADR's names, which is how the divergence surfaced.

**Fact.** The same construct also violated a rule the accepted contract states is not stylistic. `docs/research/shapes/shape-environment-contract.md`: "extent values, signed shape intermediates, symbol IDs, axis indices, input indices, interface-parameter indices, target property keys, binding phases, and physical index widths must not be accidentally mixed merely because their representations are primitive types. … This is a correctness and readability invariant, not only an API-style preference." `InputMetadata { input: String, axis: u32 }` and `TargetProperty { key: String, version: u32 }` used primitives where `InputKey`, `Axis`, and `TargetPropertyKey` already exist in this crate.

**Decided: rename and retype.** The variants are now ADR 0008's, and each field is the crate's existing governed type. This matters to *this* ticket rather than being incidental tidying: an `InputDimension` binding that names an `InputKey` can be checked against the input a region actually declares, and one that names a `String` cannot.

**Decided: drop `TargetProperty::version`.** `TargetPropertyKey` is already the crate's stable versioned key — ADR 0008 asks for "a stable versioned key", and `crate::program::abi::AbiRoot::TargetProperty` carries the key with no separate version. A second version channel beside it was a second authority over one fact. The `ShapeEnv` domain separator moves to `tiler.shape-env.v3` because a binding now encodes to different bytes; as with `v1`→`v2`, no durable reader observed the earlier version.

`InterfaceParameterKey` is added as the crate's *first* definition of that concept, not a second — an input tensor's key is `InputKey` and a device property's is `TargetPropertyKey`, and the contract keeps all three apart.

### What the index layer now does

**A domain dimension's extent is a `SourcedExtent`: a literal, or a `ShapeSymbol` declared in one `ShapeEnv`.** There is no index-local symbol table, no index-local binding, and no way to name an extent without the environment that declares it — `docs/ir.md`'s requirement that unsupported dynamic cases "reject rather than entering an index-local symbol or untyped predicate escape hatch".

**The four rejections land in three different places, and the module says which is which.** Free and ambiguous sources are already impossible inside a verified `ShapeEnv`, so re-deciding them here would be a second authority; what the index layer checks is that the symbol belongs to *this* region's environment. Tensor-data-derived sources are **unrepresentable**, which is deliberately a weaker claim than "rejected" and is stated as one. Too-late sources are the index layer's own check.

**The phase ceiling is the first place the availability ladder bears weight.** **Fact:** the accepted contract admits only bindings "evaluable on the host before any device work begins", and properties available after preparing a pipeline "cannot initially determine semantic output shapes: doing so would create a dependency from shape to plan/pipeline and back to shape". **Inference:** an index-domain extent is upstream of the launch geometry, so the same rule binds it, making `LiveDevicePreflight` the last admissible phase. The corpus states this for *semantic* extents and nowhere for index domains — the gap is real and the inference is written down in the module rather than assumed. The check is a comparison because ADR 0043 documents its order as total and load-bearing.

**A symbolic extent is not an opaque hole.** `ShapeEnv::extent_interval` was added — recomputed, never stored, so no derived solver state can reach identity — and returns the closed interval every model confines a symbol to. Three outcomes follow, and they are genuinely different:

- the environment **bounds** the extent: interval propagation proves the access, and the retained evidence says `Interval`, so nothing walked a domain of unknown size;
- the environment **determines** it (a one-point interval, which is sound because the interval contains every model): enumeration and the write-permutation argument become available, exactly as for a literal;
- the environment does **neither**: the access is refused.

That refusal is deliberately *not* `ProofResourceLimit`, which `docs/ir.md` defines as meaning "the enumeration stopped — not that the region was disproved". It reuses `BoundsNotProven`/`WriteOwnershipNotProven`, which the same contract classifies as refusals, and which `verify_access_exhaustively` already emits for the analogous unrepresentable-element-count case. A more specific diagnostic would be better and needs a public enum variant, so it is `name-the-unprovable-symbolic-extent-diagnostic` rather than a silent omission.

**Identity names the environment.** A region retains the `ExtentSources` it resolved against and folds that environment's `ShapeEnvIdentity` into its own canonical bytes; the region domain separator moves to `tiler.index-region.v5`. A symbolic extent encodes its *symbol*, never a resolved value, because the accepted contract keeps `graph identity`, `interface identity`, and `specialized identity` distinguishable and folding a bound value would collapse the first into the last.

**`static_extent()` now returns `None` for a symbolic dimension**, which is precisely what `docs/ir.md` reserved. No public constructor produces one, so every region a public caller can build still answers `Some` for every dimension; the observable public behaviour is unchanged.

### Draft status, and what was not done because it is owner-reserved

Everything new is `pub(crate)` under ADR 0074 convention 7, matching both dependency tickets. No `pub` item, public trait, `unsafe` block, or dependency was added. Three consequences were handled rather than worked around: the unprovable-extent diagnostic wanted a public enum variant (split); the additive borrowed view `docs/ir.md` reserves is present as `pub(crate) sourced_extent()` awaiting promotion (`promote-the-symbolic-index-profile-to-a-public-boundary`); and the draft `dead_code` allowances name the review as the thing that removes them, so nobody satisfies the lint by promoting the boundary unreviewed.

### Split, with the reason

- `bind-shapeenv-sources-into-tensor-boundaries-and-coefficients` — boundary shapes are still `Shape`, so a *dynamically shaped output* is inexpressible; and coefficients/divisors are still literal, so ADR 0046's semi-affine class is unimplemented. Both were in this ticket's reading of "domains and expressions"; landing domains alone is a smaller claim that is fully true.
- `name-the-unprovable-symbolic-extent-diagnostic` — public API, owner-reserved.
- `promote-the-symbolic-index-profile-to-a-public-boundary` — covers all three ShapeEnv drafts, not just this one.

### Evidence

Seven tests in `crate::index::sourced`, each naming a contract clause, each rejection paired with its accepted neighbour so the refusal is evidence about the input. The bounded/too-wide pair differs only in the constraint `n <= 4` versus `n <= 5`. The determined/bounded pair shows one symbol proving write ownership under `n == 4` and failing under `1 <= n <= 4`. The unbounded case asserts both that it *is* refused and that it is *not* the proof-resource diagnostic, which are the two ways it could have gone wrong. The fixture reduces over a symbolic axis rather than writing to one, because a write must cover every parallel dimension — that constraint is what makes the bounded-but-undetermined case reachable at all, and it is the realistic shape of the feature.

195 `tiler-ir` unit tests pass. `uv run --locked python scripts/check_repository.py` passes on macOS arm64 at the pinned nightly.

### Stale citation left behind

`widen-shapeenv-factorization-fragment`'s Outcome cites `BindingSource::StaticValue` and `BindingSource::CallerParameter`. Those are now `Static` and `InterfaceParameter`. The ticket is `awaiting-decision` and assigned to another worker, so its text was left alone rather than edited underneath them; its argument is unaffected, since it turns on *which* bindings are compile-time determined and not on their spelling.
