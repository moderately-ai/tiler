---
id: replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics
title: Replace broadcast-f32 v1 with sourced broadcast-f32 v2 semantics
status: in-progress
priority: p1
dependencies: [seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, define-the-widening-relation-over-a-symbolic-broadcast-extent, resolve-semantic-shape-inference-over-symbolic-extents, retain-one-derived-proof-summary-per-shape-environment, narrow-symbolic-inference-and-restore-host-owned-refusals]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [implementation/ir, contracts/foundation, implementation/reference, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, semantics, broadcast, identity]
claimed_from: todo
assignee: worker-broadcast-f32-v2
lease_expires_at: 1786629706
---
# Replace broadcast-f32 v1 with sourced broadcast-f32 v2 semantics

## User-visible outcome

One governed broadcast occurrence can declare a positive symbolic result extent, including a symbol that later binds to one, without changing semantic operation count or silently becoming a different operation.

## Work

- Complete-replace `tiler::broadcast-f32@1` with `@2`; do not retain a compatibility registration or two built-in implementations.
- Make the mapping's declared extents sourced and give the changed grammar a new mapping domain. Reuse the shape vocabulary's canonical symbol authority rather than independently encoding a second symbol language.
- Preserve the current literal canonicality rules exactly. For symbolic mappings, require exact-environment admission and positivity; prove `FromOperand` equality and the unit source of `StretchUnit` through the same environment.
- Split context-free syntax validation from environment-dependent application. Only the semantic builder that owns the program environment may mint an occurrence and result.
- Reuse the verified environment's retained proof summary during one `O(rank)` mapping walk. Broadcast validation performs no semantic solve and introduces no derived cache or second proof authority; the environment already performed its one semantic-closure solve at construction.
- Replace the governed definition, facade, conformance population, registry row, law/provider references, documentation, identity ledgers, and pins that own the operation-key change.
- Keep zero, undeclared/foreign symbols, unproved equality, and literal-one many-to-one mappings as typed named refusals.

## Identity

The v2 key and mapping domain deliberately move every governed broadcast semantic identity and every downstream selected subject that reaches one. Unrelated operations and the admitted v1-free population must remain byte-stable. No artifact-schema step follows merely from receiving changed nested semantic identity bytes.

## Acceptance

- Literal extent one still fails under the existing canonicality rule while its reindex neighbour succeeds.
- Symbolic extents proven over `[1, upper]`, `[2, upper]`, and exactly one are admitted; a symbol whose lower bound is zero is refused before graph mutation.
- `FromOperand` and `StretchUnit` equality/unit proofs fail independently under subject perturbations.
- One high-rank mapping validation performs zero semantic solves, with nonzero summary-query censuses that would expose per-axis resolving. The owning environment's construction census remains exactly one semantic-closure solve.
- The old governed key is absent from the standard registry and source census; the new key/domain/pins are coherent.

## Source-first Fact audit — 2026-08-12, exact base `187ecabd5fe74446401d9fe1ba7d71f6f23837ad`

Re-read at this worker's base before any implementation edit. Coordinator-verified claims were treated as stale until each cited site was opened here.

**Verified — the governed key is still `tiler::broadcast-f32@1`.** `broadcast_f32_op` still constructs `OpKey::new("tiler", "broadcast-f32", 1)`. The normative definition opens `tiler::broadcast-f32@1;`. Standard registration, the law sidecar row, and `standard_conformance("broadcast-f32")` all consume that key.

**Verified — `try_standard_with_shape_environment` and `input_sourced` exist.** `SemanticProgramBuilder` still takes an `Arc<ShapeEnv>` at construction and authors a symbolic input as `Vec<SourcedExtent>`.

**Verified — the mapping is still a literal-extent v1 subject.** `BroadcastAxisMapping` stores `result_extents: Vec<Extent>`. `encode_mapping` writes each extent as `CanonicalValue::unsigned_u64`. The domain separator is still `tiler.broadcast-axis-mapping.v1`. `new` still refuses a many-to-one entry whose declared extent is below two under `RelationDoesNotWiden`, and a wholly one-to-one mapping under `NoManyToOneRelation`.

**Verified — inference is still static-only and literal-only.** `BroadcastF32::infer` still calls `request.static_operand_shape(0)` and then `mapping.result_shape`. Registration uses `OperationDefinition::new`, which is the public literal-only constructor; it is not `new_governed_environment_aware`.

**Verified — the coordinator's live-domain claim is current, and neither domain is a format step for this change.** `PINNED_IDENTITY_DOMAINS` still contains `tiler.semantic-registry.v8` and `tiler.semantic-definition-projection.v6`. Those domains encode the *rendering* of a definition, including the already-present participation tag. Changing this family's key, normative text, participation value, and mapping *content* moves snapshot and projection *digests*. It does not change how a definition is framed, so those two domains do not step. The mapping grammar does change, so `tiler.broadcast-axis-mapping.v1` steps to `v2`. No artifact-schema domain steps merely because nested semantic identity bytes move.

**Verified — one shared sourced-extent encoding already exists and can be reused without a public-boundary expansion.** `SourcedExtent::encode` is `pub(crate)` beside `ShapeSymbol::encode`. There is no decoder today. Adding a crate-private decoder next to that encoder reuses the one tag table (`0x01` static, `0x02` symbol) and the symbol authority. It does not publish `encode`, add a public CanonicalValue sourced-extent constructor, or invent a second symbol language. Stop condition not fired.

**Verified — sourced-shape sealing does not change admitted canonical bytes.** `SourcedShape` is an opaque struct over a private normalized representation. `SourcedShape::encode` still length-frames the rank and then each `SourcedExtent`. This ticket does not edit that encoder. Stop condition not fired.

**Verified — a second environment authority is not required.** `ExtentSources` is crate-private to construct, bound to the program's one `ShapeEnv`, and already offered to governed inferencers through `OperationInferenceRequest::extent_sources`. Proof queries read the retained `ExtentProofSummary` (`proves_equal`, `proves_positive`, `admit`) and increment named censuses without calling `constraint::solve`. Broadcast application can walk the mapping once against that summary. Stop condition not fired.

**Verified — host re-derivation cannot carry mapping-attribute extent failures.** `rederive_extent_source` keeps `ExtentsNotProvedEqual` only when *both* extents appear on operands, and it drops `UndeclaredSymbol` / `SourceTooLate` entirely. Declared mapping extents live in the attribute, not on the operand `ValueFact`. Environment-dependent broadcast refusals must therefore be named `BroadcastMappingError` variants (provider-attributed `broadcast.mapping.*` codes), not stamped `ExtentSourceError`s.

**Verified — `ValueFact::from_sourced` is the crate-private result constructor.** Public `ValueFact::new` still takes a static `Shape`. A symbolic result must go through `from_sourced`.

**Verified — prerequisite `narrow-symbolic-inference-and-restore-host-owned-refusals` is `done`.** The public provider path is static-only; governed environment-aware construction is crate-private.

**Imprecise as a live claim in older records — `docs/ir.md` still names `@1`.** The foundation sentence that `Reindex` and `Broadcast` are registered as `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` is true at this base and becomes false when the key steps. That is this ticket's documentation edit, not a false premise.

**Identity derivation.** Step `tiler.broadcast-axis-mapping.v1` → `v2`. Keep `tiler.semantic-registry.v8` and `tiler.semantic-definition-projection.v6`. Recompute every pin that folds the broadcast key, its definition, its participation tag (`LiteralOnly` `0x01` → `GovernedEnvironmentAware` `0x02`), or a program that contains a broadcast occurrence. Unrelated operation-definition rows and the admitted v1-free population stay byte-stable.

**Scopes added after the identity/consumer census.** `implementation/reference` owns the conformance population and live comments. `implementation/compiler` owns the numerical-capability key string and the explain-request qualifier that folds the full registry snapshot. The standard Metal artifact/cache pins were rerun and stayed byte-stable: that path does not reach a broadcast occurrence, which is the v1-free population the Identity section requires.

## Implementation record — 2026-08-12

`tiler::broadcast-f32@2` is the sole standard registration. Mapping extents are `SourcedExtent`, encoded with the crate-private `SourcedExtent::encode` / `decode` pair under `tiler.broadcast-axis-mapping.v2`. Construction stays context-free; `apply` is the one `O(rank)` environment-dependent walk and reads the retained proof summary. Registration is `new_governed_environment_aware`. `ValueFact::from_sourced` is the only result constructor.

**Identity pins recomputed on this branch.** Snapshot `15a35d501845fb22…` → `d43dc8465a4aa96b…`. Law-registry `7a7d1933feffa058…` → `bd9a853952791c43…`. Broadcast law row `5d9235ca1f0cd502…` → `d713b2f4c57bc5c6…` (width still 90). Explain request qualifier `6e91a843fd9e69b8` → `2a7703b2159e06b4`. Unrelated law rows and the standard Metal artifact/cache pins were rerun and stayed.

**Subject perturbation**, assertions unchanged: the high-rank fixture's interval was `[1, 64]` and was changed to `[0, 64]`. Failure text:

```
a rank-zero operand with positive symbolic pads is admitted: "broadcast.mapping.extent-not-proved-positive: result axis 0 declares broadcast/0::t, and a many-to-one relation requires this program's shape environment to prove that extent is at least one"
```

The interval was restored to `[1, 64]`.

## Stop conditions

Stop if sourced-shape sealing changes admitted canonical bytes, if semantic construction would require a second environment authority, or if one shared sourced-extent encoding cannot be used without an unreviewed public-boundary expansion.
