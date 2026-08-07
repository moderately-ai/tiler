---
id: accept-the-partitioned-concatenate-realization-law
title: Accept the partitioned concatenate realization law
status: awaiting-decision
priority: p2
dependencies: []
related: [lower-the-concatenate-occurrence-through-partitioned-writes, accept-the-softmax-realization-law, accept-the-partitioned-write-ownership-proof-boundary, accept-the-sub-domain-write-domain-surface, accept-the-partitioned-result-binding-boundary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing, concatenate]
---
## What is being accepted

One further variant of the public `#[non_exhaustive]` `IndexRealizationLaw`, landed as a labelled draft by [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

New in `tiler_ir::index`:

```rust
pub enum IndexRealizationLaw {
    // ... eleven existing variants, none changed ...
    PartitionedConcatenate {
        axis_attribute: AttributeFieldId,
    },
}

impl IndexRealizationLaw {
    pub const fn concatenate_f32() -> Self;
}
```

Nothing else in the public surface moves. No existing variant's shape, payload, or encoding tag changes; the constructor is `const` because its one field is a compile-time attribute identifier. The new encoding tag is **12**, appended.

One registration moves with it: the standard semantic provider now registers this law for `tiler::concatenate-f32@1`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for that family. `family_realizes_region_sequence` answers `false` for it — this is a single-region law.

## What is *excluded* from this surface

- **The seven registered index-access capabilities** are `tiler-compiler`'s, crate-private, and reach no public item: `governed_index_access_capabilities` is `pub(crate)` and `GovernedConcatenateF32` is a private type. What is externally visible is that a governed registry now resolves `tiler::concatenate-f32@1` at each admitted arity, under seven provider identities named `tiler::governed-index-access.concatenate-f32.arity-N@1`. Those identity strings *are* durable — they enter the lowering-registry identity and therefore the explain request subject — but they are not a Rust surface.
- **The emission helpers** — `ConcatenatePlan`, `emit_partitioned_concatenate`, `declare_shared_concatenate_domain` — are private to `crates/tiler-ir/src/index/law.rs`, exactly as the staged emitters are. Nothing outside that module builds an expected region.
- **No new proof form, diagnostic, or write contract.** The region this law builds is stated entirely in vocabulary [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) and [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) already landed and already filed for acceptance. This ticket adds a *producer* of that vocabulary, not more of it.

## The choices worth objecting to

- **The non-concatenated axes share one dimension each, rather than each root carrying a private copy.** Both spellings verify. The family admits an occurrence only when every operand agrees on those axes, so one dimension per such axis is the region's own statement of that agreement; a private copy per root would put `n · (rank − 1)` dimensions into the canonical identity that are pairwise equal by construction. The honest counter-argument: the landed contract's phrase is "several iteration domains, one per root", and sharing means the roots' domains *intersect* rather than being disjoint sets of dimensions. That reading is not what the contract decided — it eliminated the sub-*range* annotation in favour of a per-access domain that is any *subset* of the region's parallel set, and two subsets may share members — but if a reviewer wants the roots' domains disjoint as well, this is the thing to say so about, and the cost of changing it is one region shape and every identity derived from it.
- **The partition is keyed by operand, not by distinct input.** `concat(x, x)` is one input boundary and two members at two offsets. This follows from operand order being semantic, and it is what makes the offsets a prefix sum over `operands` rather than over `inputs`.
- **The occurrence's attribute record must be exactly the one field the law names**, as the two staged templates demand of theirs. `concatenate_axis` would tolerate a record carrying more than it reads, and tolerance here is the silent-wrongness path.
- **Three of the law's refusal rules have no watched perturbation, and two of them are unreachable.** `concatenate-result-shape` (the re-derived result disagreeing with the declared one) and `concatenate-operand-binding` (an operand position outside the input boundaries) cannot be reached from a *verified* occurrence, because `IndexRefinementSubject::derive` builds both from the family's own inferencer; `concatenate-result-arity` needs a multi-result subject and no registered family has one. They are stated anyway because a law is interpreted against a subject rather than against the inferencer that produced it — the same call [`accept-the-softmax-realization-law`](accept-the-softmax-realization-law.md) made for `softmax-reduced-axis-rank`. If a reviewer would rather have no unreachable check than an untested one, this is the second occasion to say so.
- **Seven provider identities rather than one.** The registry refuses a second signature under one `(family, operation, provider)` triple as a `ConflatedCapabilityKey`, so seven arities under one provider identity do not register at all. The alternative is a variadic `LoweringSignature`, which is a resolver redesign and is not proposed here.

## The identity consequence, and where it landed

Registering the law moves the count-prefixed law sidecar and therefore `FrozenIndexRealizationLawRegistry`'s identity; registering seven capabilities moves the lowering-registry identity. Both are folded into the explain request subject, which moves that pin from `7bba54bcb59ec2cc` to `0aa252e0bfa16451`. That is the pin working rather than collateral damage: two requests built against different realization and lowering authorities are different requests. The semantic snapshot identity is computed without the sidecar and does not move, so every artifact and kernel-program identity derived from it is byte-identical — measured as an empty diff over the 36 distinct sixty-four-hex literals in `crates/**/*.rs` against the base.

## Evidence

The deriving ticket's Outcome carries: the iteration-domain decision with its contract citation, the seven registrations and why seven providers, refinement at every admitted arity, the zero-extent prefill case, the value-joined-to-itself case, the two watched-failing offset displacements, the tag-12 injectivity reasoning, the pin movement old→new with its survey, and the commands.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the variant is in use inside `tiler-ir` and labelled a draft at its definition.
