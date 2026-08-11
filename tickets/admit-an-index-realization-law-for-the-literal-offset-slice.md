---
id: admit-an-index-realization-law-for-the-literal-offset-slice
title: Admit an index realization law for the literal-offset slice
status: done
priority: p2
dependencies: [lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability]
related: [lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability, accept-the-literal-offset-slice-realization-law]
scopes: [implementation/ir, implementation/compiler, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, slice, identity]
---
## User-visible outcome

The literal-offset `tiler::slice-f32@1` occurrence has a registered `IndexRealizationLaw` that independently reconstructs its exact one-region access relation, so refinement can compare a provider's emitted region with semantic authority rather than refusing `MissingRealizationLaw`.

## Why this exists

**Fact — exposed 2026-08-11 at `099c6e2d`.** [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) registered the exact unary-F32 capability and drove its real provider to a structurally verified region, but `refine_index_region` refused that same occurrence as `IrVerifier(MissingRealizationLaw)` before comparing the provider output with an expected realization. The source-safe query anchor `fail-closed direction: an occurrence with no` documents that behavior, and `family_realization_law(&slice_f32_op())` returns `None` at this base.

**Fact — this remainder is identity-bearing, but the exact version consequence is not permission to move every domain.** `FrozenIndexRealizationLawRegistry::from_semantic` builds its canonical identity from the semantic and scalar snapshots plus the count-prefixed `encode_index_realization_law_sidecar`. `IndexRealizationLaw` is a public, `#[non_exhaustive]` typed vocabulary whose `realize`, numerical-contract, and `encode` matches enumerate every current variant; its sequence-shape predicate names the staged variants while `realize_sequence` routes every other variant through the one-region default. The standard registry registers every first law for an operation at law-row revision `1`, and the current law encoder uses append-only tags through `12` under `tiler.ir.index-realization-law-registry.v1`. Adding a slice law therefore adds one standard sidecar row and changes the complete frozen law-registry identity (and every derived pin that retains it), while existing law-row bytes, the semantic-registry snapshot, and the `v1` domain can remain unchanged only if the new spelling is proved append-only. A new law variant is also additive growth of an existing public `#[non_exhaustive]` vocabulary; this ticket records that exact public delta for the coordinator to classify under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md), but does not accept its included/excluded surface or authorize a merge.

## What the work is

- Re-derive the exact literal slice law from the complete semantic selection grammar and the independent compiler provider; decide the smallest law variant and constructor that state `WholeAxis -> d` and `Window { offset, .. } -> d + offset` with a total relation match.
- Register that law for `slice_f32_op()` in the semantic sidecar and update every exhaustive interpretation, encoding, identity census, and public-boundary label the new variant reaches. Derive whether the first slice row is revision `1` and an append-only next tag under the existing `v1` domain; preserve every old row byte if so, and otherwise stop for an explicit domain/version decision rather than silently stepping it. Recompute every standard-law-registry consumer and pin rather than assuming the blast radius.
- Perturb the law and provider independently by dropping a nonzero offset, with the other side unchanged, and quote the refinement mismatch in both directions. Prove an exact match refines.
- Record the exact ADR 0075/current-working-contract classification and any required public acceptance carrier instead of treating the ticket, a tested draft, or a `#[non_exhaustive]` marker as acceptance authority.

## Explicit non-goals

- The compiler-local capability and provider, owned by the dependency.
- Strided or source-bearing offsets, scheduled-region vocabulary, `VerifiedKernel`, view-versus-copy planning, or backend work.
- Reusing `Reindex` by erasing the selection attribute's distinct identity or admission rules.

## Stop conditions

- Stop for Tom if the law variant's public included/excluded surface has more than one defensible shape after source derivation.
- Stop and split if the exact law needs any `IndexNode`, access, ownership, schema, or semantic slice widening beyond the admitted literal grammar.

## Implementation evidence — 2026-08-11

**Independent review correction — 2026-08-11 at exact `2f6a314d`.** The
tag-13 encoder comment said there were five older one-attribute payloads, but
the exhaustive match shows six at tags 4, 5, 6, 7, 11, and 12. The comment now
says six; no encoder byte, law row, identity, pin, or behavior changed.

**Fact.** The total source grammar admitted one dominant public shape:
`IndexRealizationLaw::Slice { selection_attribute }`, with
`IndexRealizationLaw::slice_f32()` fixing the standard attribute field. Its
one-region realization maps `WholeAxis` to the result coordinate and literal
`Window { offset, .. }` to that coordinate plus `offset`; it retains the
provider's identity write and introduces no scalar operation. No `IndexNode`,
access, ownership, operation schema, or semantic-slice grammar widened.

**Measurement.** The standard law population moved from 15 to 16 rows and the
count-retained sidecar from 1,680 to 1,766 bytes. The complete frozen-law
digest moved from
`2b382beb419307175cd2bdb516c0b316be5c0e6b0d81ed4a09c09903b89de105`
to
`ddfb4dc459d7ca538708e276ccc4897b6fd14be99b3e7a535929ea0daee202e5`
under the unchanged `tiler.ir.index-realization-law-registry.v1` domain. The
new revision-`1` row is 86 bytes with digest
`f06152d8c886ec305aeb758f8537aa399df51e7316639069189b9535dae22703`;
the semantic snapshot digest remained
`72a5c44e73a9fb76471f1f2105b80da6f51a6ba1ecc24a24e249bf25e16e8dd4`.
A permanent type-sized tag census pins tag `13`, all 15 old row widths and
digests are re-pinned byte-for-byte, and the exact slice row is pinned
separately. The package-wide census found one derived consumer pin: the
unrelated explain fixture's request qualifier moved from
`c4d76aa0d4fbe72e` to `4f6429492ac63d04` because its request subject retains
the complete realization-law registry identity; its two event lines remain
unchanged.

**Failure evidence.** With the compiler provider left exact, replacing the
law's nonzero offset by zero made the governed slice test fail with
`governed lowering must refine: IrVerifier(SemanticRealizationMismatch {
expected: CanonicalIndexRegionIdentity(...), actual:
CanonicalIndexRegionIdentity(...) })`. After restoring the law, independently
replacing the provider's nonzero offset by zero produced the same typed
refusal with the expected and actual identities reversed in authority. The
unperturbed test then refined and executed to the reference values
`[12, 13, 17, 18, 22, 23, 27, 28]`.

**Public-boundary status.** This is ADR-0075 additive growth of an existing
public `#[non_exhaustive]` vocabulary that a coordinator may merge after the
four gates. It remains a labelled draft under the operative working contract;
Tom's exact-surface decision is parked in
[`accept-the-literal-offset-slice-realization-law`](accept-the-literal-offset-slice-realization-law.md).
The implementation scope grew to `implementation/compiler` for the independent
real-provider refinement test and to `research/semantic-graph` for the required
O-06 M5 ledger update; M6 and M7 remain unchanged.

Focused checks were green before package and workspace gates:

```sh
cargo fmt --all -- --check
cargo test -p tiler-ir semantic::registry::tests::the_standard_slice_law_is_one_append_only_revision_one_row -- --exact
cargo test -p tiler-ir index::law::tests::every_law_variant_has_one_append_only_encoding_tag -- --exact
cargo test -p tiler-ir index::refinement::tests::the_family_region_sequence_query_agrees_with_the_resolved_law -- --exact
cargo test -p tiler-compiler governed::tests::the_governed_slice_region_reads_the_literal_offset_on_every_restricted_axis -- --exact
tkt lint
make citations
git diff --check
```

The completed implementation tree also passed:

```sh
cargo check -p tiler-ir -p tiler-compiler
cargo nextest run -p tiler-ir -p tiler-compiler
# 1,847 passed, 1 skipped; an unrelated public-surface test's one leaky
# verdict passed cleanly when isolated.
cargo test -p tiler-ir -p tiler-compiler --doc
cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-ir -p tiler-compiler --no-deps
make full
# workspace nextest: 3,303 passed, 8 skipped
# release tiler-reference + tiler-compiler nextest: 1,138 passed, 3 skipped
```

## Closes when

The exact literal-offset slice law is registered and identity-coherent; matching provider output refines; independent dropped-offset perturbations fail with quoted mismatches; the public draft/acceptance status is stated accurately; and the operation-family delivery graph can mark O-06 M5 fully delivered without implying scheduled-region or physical feasibility.


## Integration — 2026-08-11

Integrated reviewed candidate cf2278a4d84e81c353663bd6ff568be42dcf68c2 into main at merge commit 28377b3fba9bd566c6d70bb143a163a2ca213243. The separate exact public-boundary decision remains awaiting-decision in accept-the-literal-offset-slice-realization-law.
