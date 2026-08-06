---
id: admit-a-multi-region-index-realization-law
title: Admit a multi-region index realization law
status: review
priority: p1
dependencies: []
related: [lower-a-two-region-occurrence-through-one-index-access-capability, admit-the-rms-normalization-family, admit-the-softmax-family, reach-a-verified-kernel-through-the-structural-families, accept-the-multi-region-index-realization-surface]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, lowering, normalization]
claimed_from: todo
assignee: agent-multiregion-law
lease_expires_at: 1785995912
---
## User-visible outcome

An operation whose canonical realization needs more than one index region — a reduction producing a shared intermediate, then an elementwise pass consuming it — can carry an `IndexRealizationLaw`, be verified against an ordered region sequence, and mint a refinement receipt that binds every region. This is the authority a normalization or a softmax needs before any *capability* vocabulary for region sequences can mean anything.

## Why this is filed

Filed from the discovery stop on [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md), whose premise was that widening the compiler-side `IndexAccessLoweringProvider` would release `tiler::rms-norm-f32@1`. Measurement falsified that: the refusal arrives before the provider is driven, from `tiler-ir`. The evidence is `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs`, whose `refining_the_normalization_refuses_before_the_provider_is_driven` observes a driven-provider count of exactly zero.

**Fact.** `crates/tiler-ir/src/semantic/registry.rs` registers an `IndexRealizationLaw` for exactly nine operations, and the normalization and the softmax are deliberately absent. The comment above that list states the intent: absence "fails closed later".

**Fact.** `FrozenIndexRealizationLawRegistry::resolve` (`crates/tiler-ir/src/index/refinement.rs`) returns `IndexRefinementVerificationError::MissingRealizationLaw` for an operation with no law row, and `refine_index_region` (`crates/tiler-compiler/src/legality.rs`) calls `resolve` *before* `emit_region`. So no provider runs for a lawless family.

**Fact.** `IndexRealizationLaw::realize` (`crates/tiler-ir/src/index/law.rs`) builds one `IndexRegionBuilder` and returns one `VerifiedIndexRegion`. `ResolvedIndexRealization::verify` (`crates/tiler-ir/src/index/refinement.rs`) takes one `&VerifiedIndexRegion` and requires `expected.canonical_identity() == region.canonical_identity()`.

**Inference — the sequence vocabulary must exist here first.** Verification is an identity comparison against a law-reconstructed region. An ordered region sequence has no canonical identity for that comparison to consume, and `IndexRefinementReceipt` binds one region's operands and results. A capability that declared a region sequence today would therefore have nothing able to certify it: the declaration would be a type-system reservation wearing the shape of implemented support, which is the conflation the architectural contract forbids.

**Fact — the two-region shape exists at a different layer.** `KernelSubprogram` / `SubprogramStage` (`crates/tiler-compiler/src/frontier.rs`) is an ordered chain with an internal intermediate, and `derive_subprogram_boundary_contract` proves the chain well formed. That operates on `VerifiedScheduledRegion` (the physical/schedule IR), not on `tiler_ir::index::VerifiedIndexRegion` (the index-refinement IR). It is a model to mirror, not a mechanism to reuse — and conflating the two IRs is what made the original ticket look reachable from `implementation/compiler`.

## Closes when

1. `IndexRealizationLaw` can express an ordered sequence of canonical regions with a named intermediate between them, and the intermediate's shape, ownership, and lifetime are explicit contracts rather than implied by stage order.
2. `realize` and `verify` agree on a canonical identity for the *sequence*, so a candidate sequence is compared as a whole and a truncated or reordered one is refused with a typed reason.
3. `IndexRefinementReceipt` binds every region in the sequence, and the reached-scalar containment check covers the union of the sequence's regions rather than one region's.
4. The law encoding stays append-only per tag, with per-tag injectivity reasoning recorded at the encoding site — `IndexRealizationLaw::encode` already carries tags 1..=8 and the tag-8 comment is the precedent to follow.
5. A deliberate perturbation — a law declaring one region for a two-region occurrence, and a sequence whose intermediate is never read — each refuses with a typed reason rather than minting a receipt.

## Non-goals

Registering the normalization's or the softmax's own law, which belongs to the family tickets once this vocabulary exists; widening `select_supported_strategy`, owned by [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md); and any compiler-side capability surface, which is [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md)'s once unblocked.

## Decision boundary

The law enum, the sequence identity encoding, and the receipt's public shape are all `tiler-ir` public surface. A tested implementation is a draft; acceptance of the exact interface is Tom's, and reaches him as an `awaiting-decision` acceptance node carrying the surface and its evidence.

Filed as [`accept-the-multi-region-index-realization-surface`](accept-the-multi-region-index-realization-surface.md), which carries the exact surface, the choices worth objecting to, and a pointer to the evidence below.

## Outcome

**The vocabulary exists and the wall it was filed against is gone at this layer.** An occurrence whose canonical realization is a fold feeding a pass over the fold's materialized result now carries a law, verifies against an ordered region sequence compared as a whole, and mints a receipt binding every region. What is *not* here is the normalization's own registration — that is the ticket's stated non-goal, and the derivation below says why it could not have been done here even had it been in scope.

**Fact — no standard operation carries the new form, and the brief's instruction to register one was refused.** The dispatch brief said "the standard provider registers a multi-region law for the normalization family it covers". The ticket's own Non-goals section says the opposite, and the ticket wins: registering it would also have broken `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs::the_normalization_resolves_no_index_realization_law`, which asserts the normalization resolves `MissingRealizationLaw` and lives in a scope this ticket does not hold.

**Inference — the normalization's law is not implementable at all until a governed reciprocal square root exists.** `crates/tiler-ir/src/index/scalar.rs` registers exactly ten governed scalar keys: the `f32` constant, multiply, add, divide, exp, NaN-canonicalization, the strict-affine U4 decode, and three `bf16` rows. There is no `sqrt` and no `rsqrt`. `crates/tiler-ir/src/semantic/rms_norm.rs` records that the family's one inexact step *is* `rsqrt`, and that it carries a resolved ADR 0042 accuracy contract. Admitting that scalar is a new semantic surface with its own tolerance, which is [`admit-the-rms-normalization-family`](admit-the-rms-normalization-family.md)'s work and Tom's boundary. Verify the absence in one line: `grep -n 'fn .*_scalar_op' crates/tiler-ir/src/index/scalar.rs`.

**Inference — that eliminated the atomic-family design and left one survivor.** Three candidates were tested against the architectural contract. (1) A generic `Staged { stages: Vec<IndexRealizationLaw> }` composition combinator: rejected, because `IndexRealizationLaw`'s own doc says "deliberately not a universal IR. Each variant is an atomic template", and a recursive composition with a shape-derivation sub-language is exactly that. (2) An atomic `StagedRmsNormF32`: eliminated by the missing scalar above. (3) The sequence vocabulary with no law able to produce one: eliminated by the ticket's own Inference — a declaration nothing can certify is the type-system reservation wearing the shape of implemented support. The survivor is one *named* two-stage template, `StagedStrictSerialSumThenPointwiseF32`, whose fold is the governed strict serial sum and whose pass is a governed pointwise binary over the fold's result. It is the reduction-then-elementwise shape the normalization needs, stated in scalars that exist.

### The law form and its identity, per site

**`IndexRealizationLaw::encode`, tag 9** (`crates/tiler-ir/src/index/law.rs`). Append-only: tags 1..=8 and their payloads are byte-for-byte unchanged, and no registered row carries tag 9, so the law sidecar every registry has ever encoded is untouched. Injectivity at the site: the discriminating byte comes first; the payload is a fixed-width attribute identifier followed by the self-delimiting scalar encoding. Tag 4 writes an attribute and tag 2 writes a scalar, but no earlier tag writes both, so this form is reachable from exactly one variant. Asserted in `the_staged_law_tag_is_append_only_and_distinct`, which also separates two staged rows differing only in scalar and only in attribute.

**`CanonicalIndexRegionSequenceIdentity`** (`crates/tiler-ir/src/index/sequence.rs`). A one-stage sequence's identity **is** its region's identity, byte for byte — the deliberate choice that keeps every existing receipt unmoved. Longer chains are written under `tiler.ir.index-region-sequence.v1\0`; a region identity carries its own distinct domain tag in the same leading position, so the preimages are disjoint. Within the tagged form the stage count, every region identity, and every source list are length-prefixed and ordered, so a truncated, extended, reordered, or differently-wired chain each render distinct bytes. Asserted in `a_one_stage_sequence_is_its_region_byte_for_byte`, `a_chain_is_never_confusable_with_one_of_its_regions`, and `reversing_or_rewiring_a_chain_changes_its_identity`.

**Receipt and executable-coverage identity** (`crates/tiler-ir/src/index/refinement.rs`). Domain-separated rather than extended: a one-stage realization keeps `RECEIPT_IDENTITY_TAG` / `EXECUTABLE_COVERAGE_IDENTITY_TAG` and the exact field order it has always written; a staged one is written under `…staged-receipt.v1\0` / `…staged-executable-coverage.v1\0` and additionally carries the sequence identity, a count-prefixed run of per-stage scalar authorities, and a stage ordinal on every binding and proof. The four tags are pairwise non-prefixing, asserted directly rather than argued, in `a_staged_occurrence_verifies_and_binds_every_region`.

**Why the sources must be folded into the identity, not just the regions.** Two chains can have byte-identical per-stage region identities and still be different realizations, because nothing in a region says which of its input boundaries is the handed value. `reversing_or_rewiring_a_chain_changes_its_identity` exhibits exactly that pair and asserts the stage identities are equal before asserting the sequence identities differ — so the test would fail if the fixture stopped exhibiting the case.

### Pins

**Measurement — no pinned identity moved, and none could have.** The explain qualifier `request=6dd42be71c6745fe` (`crates/tiler-compiler/src/explain.rs:4149`) folds the law-registry identity, which is `LAW_REGISTRY_IDENTITY_TAG || semantic snapshot || scalar snapshot || the count-prefixed sidecar run over **registered** rows` (`FrozenIndexRealizationLawRegistry::from_semantic`, and `encode_index_realization_law_sidecar` in `crates/tiler-ir/src/semantic/registry.rs`). This branch registers no row, so the sidecar is unchanged; adding an unregistered enum variant reaches no encoder. The other surveyed pins — `STRICT_F32_REGION_IDENTITY_HEX` and its `v4` sibling, `FAMILY_ORDER_IDENTITY_FIXTURE`, the governed target-profile descriptor, the `tiler-build` artifact and cache-subject digests, and the seven `tiler-metal` goldens — all sit downstream of region or schedule bytes this branch does not touch. Observed, not assumed: `cargo nextest run --workspace` → **2757 passed, 0 failed, 7 skipped** both before the branch's first edit and after the last. No recompute ticket is required.

**Measurement — `crates/tiler-compiler` was not edited.** It could not be: `implementation/compiler` is not this ticket's scope. Two of its `const fn` accessors (`legality.rs:333` and `:339`) forward to `PendingIndexRefinementReceipt::scalar_authority` and `::region`, so both had to stay `const fn`. That constraint is what produced the leading/last split in `VerifiedIndexRegionSequence` and in both receipts — which turned out to be the better design anyway, because it makes "a realization has at least one region" a type invariant instead of an `expect` at every read.

### Correctness

**The chain is derived and checked, never declared and believed.** `VerifiedIndexRegionSequence::try_new` mirrors `derive_subprogram_boundary_contract` (`crates/tiler-compiler/src/frontier.rs`) one IR down: a non-final stage must publish exactly one value, that value must be read by the *immediately* following stage, the producing and consuming boundaries must agree on element type and static shape, and a value handed on and never read is refused. Ownership and lifetime are enforced rather than annotated — an intermediate is named by no `Occurrence` source, is produced by a non-final stage whose writes therefore never leave, and has exactly one reader, so it cannot stay live across a stage that does not mention it.

**Containment covers the union.** `verify_sequence` revalidates every stage against the scalar registry and refuses the whole realization if any stage reaches an operation outside the lowering's admission. Watched failing: `an_unadmitted_scalar_in_an_earlier_stage_refuses_the_realization` admits only the multiply the *pass* reaches and observes `ScalarAuthorityConformance` from the fold's additions.

**One budget, not one per stage.** `assess_finite_domains` grew a ledger-taking sibling so the stages of one realization share the caller's budget. Re-funding it per stage would have let an n-stage realization spend n times the bound its caller named — a fail-closed limit silently weakened by the arrival of a second stage.

**A defect found and fixed along the way.** `verify` originally reached the single-region/staged mismatch through the ordinary interface check, and reported `OperandInterface` naming the pass's handed boundary. True, but it sends a reader to the provider's tensor list instead of to the arity of what was asked for. `ResolvedIndexRealization::verify` now refuses a staged law up front with `SemanticRealizationLawRefused { rule: "staged-law-requires-region-sequence" }`, before anything looks at the region.

### Refusals observed failing

Every assertion below was watched failing before it was made to pass; three of them failed for reasons that corrected the design or the claim rather than the fixture.

- **Rubber stamp / cross-family** — `a_chain_that_does_not_realize_the_occurrence_is_refused`: the chain built for the *other* pointwise scalar, identical in wiring and structurally valid, refused with `SemanticRealizationSequenceMismatch`. This is the case where every interface agrees, so only the whole-chain comparison can say no.
- **Wrong order** — same test: the reversal is *structurally sound* (`try_new` accepts it), and the ordered interface check names the disagreeing boundary one statement before the identity comparison. Asserted at the exact position. My first version of this assertion demanded the identity refusal and failed; the finding is that reordering is caught earlier and more specifically, and the identity's order-sensitivity is asserted separately in `sequence.rs`.
- **A law declaring one region for a two-region occurrence** — `region_count_disagreements_refuse_in_both_directions`, both directions, plus the compiler-facing `verify` entry point.
- **A sequence whose intermediate is never read** — `an_unread_or_unavailable_handed_value_refuses`, alongside a wrong-producer claim and a double claim on one handed value.
- **Chain malformation** — unread handoffs, interface disagreement, a non-final stage publishing two values, source-list arity, empty, and over-wide realizations, each with a named error and an exact position.

The two failures worth recording: `a_handed_boundary_disagreeing_on_shape_refuses` first failed because the fixture region could not build at all (`CoordinateOutOfBounds`) — the perturbation was in the wrong place, and moving it to the *producer's* published shape is what made both regions individually valid and the composition the only thing wrong. And the reversal test asserted a refusal that does not happen.

### Commands run

`cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-ir --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir --no-deps` (which caught a private intra-doc link no other check saw — the exact failure AGENTS.md warns about); `cargo nextest run --workspace`; `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

### Remainder

None inside this ticket's outcome. The two follow-ons are already filed: [`accept-the-multi-region-index-realization-surface`](accept-the-multi-region-index-realization-surface.md) parks the public boundary for Tom, and [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) is unblocked — its premise was falsified by the discovery stop, and the vocabulary its corrected premise needs now exists. Its own first question is the one this ticket could not answer from `implementation/ir`: whether a staged receipt should keep exposing a single-region accessor at all.
