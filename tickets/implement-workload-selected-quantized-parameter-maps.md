---
id: implement-workload-selected-quantized-parameter-maps
title: Implement the workload-selected quantized parameter maps
status: awaiting-decision
priority: p2
dependencies: [prototype-quantized-value-vertical, scope-first-quantized-lm-profile, admit-a-strict-affine-index-realization-law]
related: [implement-first-quantized-backend-profile, implement-first-runtime-semantic-value-precondition-enforcement]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/artifact, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, parameter-maps, dtype, decision, needs-tom, public-boundary]
---
## User-visible outcome

The selected direct-input per-output-channel U8/F32 value has one exact,
shape-checked axis-0 parameter map in semantic identity, binding conformance,
reference evaluation, and logical index realization. Unsupported axes and map
families refuse by name. Producer operations, encoded-value transformations,
and executable physical/ABI projection remain separate capabilities and cannot
silently infer or default this map.

## Activation boundary

`scope-first-quantized-lm-profile` is the named first consumer and must select the exact scheme, parameter granularity, target operation, layout, and conformance corpus before this ticket becomes actionable. If that profile selects per-tensor parameters only, close this ticket as obsolete rather than inventing a map producer.

**The selection landed on 2026-07-31 and it is not per-tensor, so this ticket is actionable rather than obsolete.** [The first quantized language-model profile record](../docs/research/numerics/first-quantized-lm-profile.md) selected **per-output-channel strict-affine U8 to F32**, so the exact map to implement is:

- **Granularity:** per axis, over the weight's axis 0 — the free index `o` of the workload's contraction structure `td,od->to`. Scale is `tiler::f32@1` of shape `[D_out]`; zero point is `tiler::u8@1` of shape `[D_out]`. Both are rank 1, where every strict-affine parameter component is rank 0 today and `require_scalar_type` in `crates/tiler-ir/src/semantic/quantization.rs` enforces exactly that.
- **Not selected, and each refused by name:** per-tensor beyond the two existing proof contracts, and every per-block or per-group map along the contracted axis. The block maps were eliminated on *legality*, not accuracy — a scale that varies inside the reduction makes the fused contraction partition the contracted axis into contiguous intervals merged in order, which consumes the reassociation permission no contract registered for this workload grants — so a later reassociating contract is what would reopen them, and this ticket must not implement one speculatively.
- **Why the axis is load-bearing rather than incidental:** a per-axis map over axis 1 is a per-input-coordinate map `s[d]`, not a `D_in`-sized block. It is still inadmissible for the selected fused strict fold because it varies along the contracted coordinate `d`. The map must therefore carry which axis it projects onto, and two otherwise identical values whose maps project onto different axes must have different identities.
- **First program boundary and consumer:** the pinned `Qwen/Qwen3-0.6B-Base` workload supplies 196 compound weighted-projection inputs directly; its graph contains no `Assemble` or `Quantize`. They are consumed through [`widen-the-physical-vocabulary-for-per-axis-quantized-component-access`](widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md) and [`fuse-quantized-weight-decode-into-the-strict-contraction`](fuse-quantized-weight-decode-into-the-strict-contraction.md), both dependents of this ticket rather than part of it.

## Implementation keys

- Extend the typed `ParameterIndexMap` seam delivered by `prototype-quantized-value-vertical`; do not add a second map spelling or a raw `block_size` field.
- Replace the initial exact public U4 law variant with one governed
  `StrictAffineDequantize` law form under a fresh tag. It derives roles, selected
  scalar meaning, and parameter coordinates from the admitted input type. Do
  not reinterpret tag 8 or retain two standard law spellings for one operation.
  The registry admits one law per semantic operation; the replacement remains
  candidate-blind and refuses every unsupported scheme/map pair explicitly.
- Represent the selected coordinate projection canonically and validate it only
  against the complete logical shape. The same checked map drives semantic
  validation, reference lookup, component-shape derivation, logical index access,
  identity, and explanation.
- Keep logical code dtype, quantized scheme, parameter map, and physical packing independent. Packed nibbles do not imply block quantization, and block parameters do not imply one storage encoding.
- The map grammar may represent any axis, but the first reference/index
  capability supports only selected axis 0. Other axes refuse as missing exact
  capabilities; per-block, irregular groups, hierarchical scales, codebooks,
  masks, and outliers remain absent map families and reject by name.
- Validate component role completeness and parameter tensor shapes before
  retaining an interface value or deriving dependent work. Runtime payload
  values remain dataflow and do not become static type fields.
- Add exact reference fixtures, unsupported-map refusals, and identity
  perturbations. Perturb every new check once and observe it fail. Transform,
  projected KIR/ABI access, and packed/unaligned access fixtures belong to their
  owning successor tickets.

## Decision packet — 2026-08-09

Superseded by the current-source audit below. In particular, a shape-independent
map cannot reject an out-of-rank axis at construction, and the constructor alone
cannot make the selected value type, reference evaluator, or logical realization
constructible.

## Current-source Fact audit — 2026-08-12

Audited at exact main `1fe9d92d231c7182e2b3f2178042fb945cadaf86`.

- **Verified — selected subject.** `first-quantized-lm-profile.md`, anchors
  `Codes component`, `Scale component`, and `Zero-point component`, selects
  strict-affine U8/F32 with scale and zero point projected over logical axis 0.
- **Verified — one map exists.** `crates/tiler-ir/src/semantic/types.rs`, anchor
  `Only the producer-backed per-tensor form exists today`, has only
  `ParameterIndexMapKind::PerTensor`; the encoder and its population test both
  rely on that one inhabitant.
- **False — out-of-rank construction refusal.** `Axis::new(u32)` and
  `ParameterIndexMap` carry no logical shape. Rank is known only where
  `EncodedComponentShape::component_shape` applies the map to a `Shape`.
  Storing rank or extent in the map would duplicate the graph's existing shape
  authority and create two fields that can disagree.
- **False — axis-1 taxonomy.** Axis 1 selects `s[d]`; it is per-input-coordinate,
  not one block of size `D_in`. The legality conclusion survives because `s[d]`
  varies inside the strict reduction.
- **Verified — the constructor is insufficient.** `strict_affine_type` is
  private; `StrictAffineU4::resolved_type` and `StrictAffineU8::resolved_type`
  always construct per-tensor contracts; `StrictAffineTypeValidator` admits
  exactly those two contracts. No public governed factory can construct the
  selected type.
- **Verified — producer inference is ambiguous.** `AssembleStrictAffine` and
  `QuantizeStrictAffine` reject every attribute, require scalar parameters, and
  choose the result type from code dtype alone. A square `[N, N]` value with
  `[N]` parameters cannot reveal whether axis 0 or axis 1 was intended.
- **Verified — the selected graph needs no producer widening.** The selected
  profile's anchor `the selected profile materializes no compound value
  internally` says the compound weights are direct interface inputs and the
  graph contains neither `Assemble` nor `Quantize`. Widening those operations is
  a separate future public decision, not a prerequisite of this first consumer.
- **Verified — reference and logical realization are exact today.**
  `crates/tiler-reference/src/quantization.rs`, anchors `StrictAffineProfile` and
  `read_scale_value`, registers two exact per-tensor signatures and reads one
  scalar parameter for all codes. `IndexRealizationLaw::StrictAffineU4Dequantize`
  and `GovernedStrictAffineU4Dequantize` likewise admit only the exact U4 type
  and read scale and zero point at `[]`.
- **Verified — conformance refuses the selected map.**
  `ResolvedValueConformanceContract::derive_encoded` rejects every map unequal
  to `ParameterIndexMap::per_tensor()`, and its evidence encoder hard-codes the
  admitted map tag. The binding validator revision must change when its accepted
  language changes.
- **Imprecise — transforms and executable ABI.** ADR 0029 states the `Q o D`
  preservation rule, but live reindex/broadcast/slice operations are not an
  encoded-value transform surface. The dependent physical-vocabulary ticket
  owns executable parameter-coordinate access. This ticket cannot close either
  surface without reversing its own graph.
- **Imprecise — identity migration.** A new map tag plus fixed-width axis is
  append-injective and leaves every per-tensor resolved-type byte unchanged.
  Artifact interfaces already length-frame opaque resolved-type bytes and exact
  component shapes, so neither the resolved-type domain nor artifact schema
  steps merely because a new subject becomes representable. Complete registries,
  law authorities, receipts, request subjects, and their pins do move when their
  populations change.

## Revised decision packet — 2026-08-12

The original end-to-end packet is not decision-ready. The narrow, dependency-
correct decision is the semantic map/type/reference/index substrate below.

**Recommended exact surface:**

1. Add private `ParameterIndexMapKind::PerAxis(Axis)` and public
   `ParameterIndexMap::per_axis(Axis)`. The map owns only the zero-based logical
   axis. Encode it as a fresh tag plus fixed-width big-endian `u32`; preserve the
   existing per-tensor byte exactly. Per-block/group/expression forms remain
   absent.
2. Make map application fallible where it meets the complete logical shape:
   `parameter_shape(&Shape) -> Result<Shape, ParameterIndexMapApplicationError>`
   and one shared checked coordinate projection. Refuse
   `AxisOutOfRange { axis, rank }` before retaining a graph value, deriving a
   component obligation, allocating a reference result, or building a region.
   Zero extents remain valid. Never clamp, wrap, or infer an axis from extents.
3. Add a governed public strict-affine U8/F32 type factory taking the map; callers
   never restate the eleven-field encoded contract. The semantic grammar may
   represent any in-rank per-axis map, as ADR 0029 separates representation from
   capabilities, while the first standard reference/index capability admits
   only the selected axis-0 U8 type. Existing per-tensor U4/U8 remain admitted.
4. Replace the accepted exact-U4 public law variant with a governed
   `StrictAffineDequantize` form under a fresh append-only tag, and generalize
   its scalar/reference meaning to cover legacy per-tensor U4 and selected
   per-axis axis-0 U8. Scale and zero
   coordinate lookup is derived from the type's map, never role order, tensor
   rank resemblance, or a caller-provided second map. Other axes are
   representable but return typed missing-capability refusals.
5. Extend resolved-value conformance to derive and scan the selected rank-one
   parameter components from the same checked map; advance the governed binding
   validator revision and evidence identity. Direct interface construction must
   reject an out-of-rank map and a mismatched parameter extent before routing.
6. Do not widen `AssembleStrictAffine` or `QuantizeStrictAffine` here. Their
   future non-per-tensor form needs an explicit required map/profile authority
   (or a distinct operation key); missing must never default to per-tensor.
   Dequantize is already type-directed and is the selected graph's only consumer.
7. Do not claim encoded-value view transformation or executable ABI projection
   here. Unsupported encoded transforms remain typed refusals. The existing
   physical-vocabulary dependent owns schedule/KIR/ABI/backend projection.

**Why this is the Pareto leader.** It is exact and fail-closed, preserves the
selected direct-input graph without speculative producers, leaves old canonical
bytes stable, and adds only O(1) map storage/projection state. Reference work
remains output-dominated O(number of logical elements), with one O(rank) stride
derivation and O(1) parameter lookup per element. It also preserves ADR 0029's
ability to represent a map before every backend supports it.

**Strongest counterpoint.** The standard type grammar can represent an axis that
the first reference/index capability refuses. That is deliberate maturity
separation, not partial interpretation: construction checks shape validity;
capability resolution checks exact supported type; silence never means axis 0.
A generic pattern-based reference registry would remove that finite subset, but
no second supported axis justifies replacing the current exact-signature
registry.

**Ranked alternatives:**

1. Axis-only canonical map + fallible application + governed type factory +
   exact axis-0 U8 reference/index capability: recommended.
2. The same map with generic all-axis reference/index support: correct, but it
   requires a pattern/scheme-capability redesign or an unbounded exact-row
   population before a second axis is selected.
3. New exact axis-0 Assemble/Quantize operation keys: correct for a future
   producer, but needless for this direct-input graph and proliferates operation
   identities as maps grow.
4. Rank- or extent-bearing maps, shape inference, optional/default attributes,
   raw contract construction, or a generic map expression language: rejected.
   Each duplicates authority, admits ambiguity, or speculates beyond a consumer.

## Closes when

After Tom accepts the revised surface, the selected axis-0 U8 direct-input type
is constructible and shape-valid; conformance and reference evaluation use the
same checked map; the generalized logical law retains real receipts for legacy
per-tensor U4 and selected per-axis U8; unsupported axes and all unselected map
families reject by exact name; per-tensor bytes remain unchanged; map-axis,
out-of-rank, role, extent, and identity perturbations have each demonstrated a
failure; moved registries/authorities/pins and the validator revision are
complete; targeted tests, Clippy, `tkt lint`, `make citations`, and
`git diff --check` pass; and one `make full` passes. Producer operations,
encoded-value transforms, and executable physical projection are explicitly not
closure conditions of this narrowed ticket.

## Graph maintenance

- Update `scope-first-quantized-lm-profile` with the selected map and evidence rather than copying its choice here.
- `implement-first-quantized-backend-profile` already depends on this ticket
  through the selected delivery chain; do not add a second redundant edge.
- File separate implementation work for a second independently required map family only when its workload producer and consumer are named. Do not widen this ticket into a universal map language.
- Advance only an owning domain whose existing grammar changes. Append-only map
  and law tags preserve old bytes; complete registry/authority/request values
  and their pins still move with their populations. Recompute each moved pin on
  the merged tree.
- File explicit-map producer work only when a selected graph contains
  `AssembleStrictAffine` or `QuantizeStrictAffine`; its map/profile selection is
  required and has no default.
- File operation-specific encoded-view transformation work when the first
  encoded reindex, slice, reshape, broadcast, or concatenate consumer is selected.
  Until then those operations reject encoded values rather than promising a
  transform system this ticket cannot exercise.
