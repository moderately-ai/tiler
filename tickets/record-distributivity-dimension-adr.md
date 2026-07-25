---
id: record-distributivity-dimension-adr
title: Record the settled distributivity dimension as an ADR
status: done
priority: p2
dependencies: []
related: [settle-contraction-chain-distributivity-permission, decide-whether-to-admit-a-distributivity-permission, record-distributivity-in-the-navigation-contracts]
scopes: [contracts/decisions]
shared_scopes: [project/tickets, contracts/numerics, contracts/optimizer]
paths: []
tags: [documentation, numerics, decision]
---
`settle-contraction-chain-distributivity-permission` resolved a durable choice by derivation: distributivity is a third numerical dimension independent of reassociation and operand permutation, a tensor-contraction chain regroup consumes all three, and the rewrite fails closed under every contract Tiler can express as a settled legality position rather than a pending one. That conclusion is now normative in `docs/numerical-semantics.md` ("Distributivity is outside the order contract") and cited by `docs/compiler/optimizer.md`, `docs/compiler/fusion-and-scheduling.md`, `docs/roadmap.md`, and `docs/open-questions.md`.

No ADR records it. `grep -rli distributiv docs/decisions/` returns nothing at `412ceae`; the highest ADR is 0077.

This is a gap in decision custody rather than in the contract text. `AGENTS.md` requires that when evidence resolves a durable choice, the contract is updated *and* an ADR is added or accepted. Five documents now depend on a settled position whose only record is a normative section and a `done` ticket outcome, so a reader cannot see it in the accepted ADR index beside the decisions it sits with.

The ADR belongs in the `numerical-operations` catalog group beside ADR 0014 (reassociation versus operand permutation) and ADR 0015 (required FMA versus optional contraction). It supersedes neither: neither claims exhaustiveness over the dimension set, and ADR 0011 already holds that one permission never implies another. It should state the dimension, its independence, the contraction-chain consequence, and that no permission is admitted.

**Scope boundary.** This is distinct from `decide-whether-to-admit-a-distributivity-permission`, which is `awaiting-decision` because it needs a product choice from Tom — whether to admit a permission at all, and whether one permission covers both directions of the identity. This ticket records only what is already derived and settled, so it is not blocked on that decision. If Tom prefers one ADR carrying both the settled dimension and the admission choice, close this into that ticket instead.

Found by `record-distributivity-in-the-navigation-contracts` while checking, as instructed, whether the owed ADR had landed before pointing navigation text at it. It had not, so the navigation contracts point at `docs/numerical-semantics.md#distributivity-is-outside-the-order-contract`. Those links should be revisited when this ADR lands.

## Outcome

[ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) is accepted and carries the six clauses the settling ticket said it should. Two stale citations found while verifying its facts are corrected in the same change.

**Decision — `accepted`, not `proposed`, and why the distinction is not a formality here.** Every clause is derived from the numerical contract's own definitions of reassociation and permutation and from ADRs 0011, 0014, and 0015; none of it is a product choice, and the one product choice in the neighbourhood is reserved by item 4 and left with `decide-whether-to-admit-a-distributivity-permission`. The status is load-bearing rather than ceremonial: `docs/document-metadata.md` states that in a `mixed` contract "only accepted-ADR-derived invariants and sections explicitly labeled accepted are normative", and `docs/numerical-semantics.md` is `contract_status: mixed`. Without an accepted ADR the distributivity subsection is a section four documents cite as settled and the metadata contract classifies as proposed. Marking the record `proposed` would leave that discrepancy exactly where it was.

**Fact — the derivation was re-checked against the tree rather than inherited from the settling ticket.** `docs/numerical-semantics.md` defines reassociation as changing grouping while preserving logical operand order, and its rule that "reassociation without permutation may combine only contiguous contributor intervals in order" presupposes a fixed contributor sequence. ADR 0014's context states the same transform in three-operand scalar terms. `NumericalPermission` in `crates/tiler-ir/src/schedule/numerics.rs` has exactly two variants, `Forbidden` and `Permitted`, and no field, variant, or capability anywhere in `crates/` names distributivity.

**Retracted — a fact both `docs/numerical-semantics.md` and `docs/compiler/optimizer.md` asserted, and which `correct-the-optimizer-one-variant-permission-claim` re-verified as recently as 2026-07-24.** Both wrote that `StrictF32NumericalContract::governed` "remains the only contract the compiler registers". It is not. `StrictF32NumericalContract::governed_profile` in `crates/tiler-compiler/src/request.rs` returns a two-element array and is the single admission authority three call sites share; `crates/tiler-compiler/src/session.rs`'s public `NumericalContract` enum lets a caller select either `StrictF32` or `FlushSubnormalsToZeroF32`. The ordering, so the correction is attributable rather than a reproach: `1ab0fd8` wrote the optimizer sentence on 2026-07-24 at 20:16 and it was true then; `aa7c4f0` unified admission behind the profile and registered the second contract on 2026-07-25 at 08:12. Both documents' *conclusions* survive untouched, because both registered contracts set `reassociation: Forbidden` — only the arithmetic in the premise moved. Both are corrected to name `governed_profile` and its two members, which is a statement that a third contract cannot silently falsify.

**Fact — three edits outside `contracts/decisions`, all additive or corrective within one sentence.** `docs/numerical-semantics.md` gains the ADR 0080 citation in its distributivity subsection and the registered-set correction. `docs/compiler/optimizer.md` gains the ADR 0080 citation on its third logical-exploration rule and the same correction. `contracts/numerics` and `contracts/optimizer` were added as shared scopes for exactly those; both were uncontended, with no `in-progress` ticket holding either. `decide-whether-to-admit-a-distributivity-permission` holds `contracts/numerics` and `contracts/decisions` but is `awaiting-decision` and therefore parked.

**Decision — the navigation links stay where they point.** `docs/roadmap.md` and `docs/open-questions.md` link to `docs/numerical-semantics.md#distributivity-is-outside-the-order-contract`, and that remains the correct destination: the contract owns the derivation and the worked counterexample, and an ADR records a decision rather than restating a normative section. The gap those links left was that a reader of the section could not see it was ADR-derived, and that is closed at the section itself rather than by repointing four inbound links. `contracts/navigation` was therefore not added and neither file was touched.

**Not folded in.** `implementation_status` is `not-started`, which is the honest value: the record defines a dimension and withholds its permission, so what exists in `crates/` is the *absence* item 5 requires a rejection to name. A reservation in the vocabulary is not implemented support.

**Measurement.** `uv run --locked python scripts/docs.py render` reported "documentation render passed (183 records)". `uv run --locked python scripts/check_repository.py` exited 0 with "complete repository validation passed". Host macOS arm64, toolchain `nightly-2026-07-19`.
