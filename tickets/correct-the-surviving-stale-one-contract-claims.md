---
id: correct-the-surviving-stale-one-contract-claims
title: Correct the surviving stale one-contract claims outside ADR 0080
status: done
priority: p2
dependencies: []
related: [restore-adr-0080-verbatim-quotation, correct-the-optimizer-one-variant-permission-claim]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics]
---
[ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) corrected two citations of a fact that stopped being true on 2026-07-25 at 08:12, and said it did so "rather than leaving a fourth stale copy". Found while restoring that record's verbatim quotation: at least one governed copy survives outside the two it corrected, and it is not in `contracts/decisions`.

**Fact — the compiler registers two contracts, not one.** `crates/tiler-compiler/src/request.rs`'s `StrictF32NumericalContract::governed_profile` returns `[Self::governed(), Self::governed_flush_to_zero()]`, and `is_governed` tests membership in that array rather than equality with one constant. The second differs only in `input_subnormals` and `result_subnormals`, both `SubnormalMode::FlushToZero { zero_sign: FlushedZeroSign::PreservesSign }`. Reproduce: `grep -n "fn governed_profile" -A 3 crates/tiler-compiler/src/request.rs`.

**Fact — the surviving governed copy.** `docs/roadmap.md:326` reads that the fused-multiply-add permission "is `NumericalPermission::Forbidden` in the only numerical contract the compiler registers today (`StrictF32NumericalContract::governed` in `crates/tiler-compiler/src/request.rs`)". Reproduce: `grep -rn "only numerical contract" docs/`. **Retracted — this ticket said that command "returns exactly this one line". It returns two**: `docs/roadmap.md:326` and `docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md:68`, the second being the stale wording ADR 0080 deliberately retains under its `superseded-quotation` marker. The narrower reproduce that isolates the defect is `grep -rn "only numerical contract" docs/ | grep -v superseded-quotation`.

**Inference — the conclusion is unaffected and only the premise's arithmetic is stale**, which is the same finding ADR 0080 recorded for `docs/numerical-semantics.md` and `docs/compiler/optimizer.md`. Both registered contracts set `contraction: NumericalPermission::Forbidden`, so a device or library GEMM built on fused multiply-add accumulate still does not implement the declared semantics under either. The sentence should name the profile function rather than one constant, exactly as ADR 0080's own statement of the fact now does.

## Also stale, in `project/tickets` rather than a contract

Two ticket bodies carry the same premise and are listed so a reader is not left to rediscover them. Neither is a governed contract and neither changes a conclusion, so correcting them is lower value than the roadmap line and is bundled here rather than split again.

- `tickets/settle-contraction-chain-distributivity-permission.md:19` states `StrictF32NumericalContract::governed` "is the only numerical contract the compiler registers and sets `reassociation` to `NumericalPermission::Forbidden`". Both registered contracts do.
- `tickets/scope-einsum-contraction-support.md:39` states the permission is `Forbidden` "in the only numerical contract the compiler registers".

`correct-the-optimizer-one-variant-permission-claim` is a separate ticket over a *different* stale claim in the same neighbourhood — that `NumericalPermission` has exactly one variant — and is not superseded by this one. Read it before editing `docs/compiler/optimizer.md`, because both tickets can reach that file.

## Closes when

No governed document states that the compiler registers one numerical contract, `grep -rn "only numerical contract" docs/ | grep -v superseded-quotation` returns nothing, the roadmap sentence's conclusion about fused multiply-add accumulate is preserved unchanged, and `uv run --locked python scripts/docs.py render` plus `uv run --locked python scripts/check_repository.py` pass.

**Retracted — this ticket originally required the unfiltered `grep -rn "only numerical contract" docs/` to return nothing.** That condition is unsatisfiable without breaking ADR 0080, whose `superseded-quotation` marker exists precisely to retain the stale wording verbatim; the filter above is the correct form and the paragraph directly below already said so.

**Do not** weaken ADR 0080's restored quotation while doing this. (Verified: the marker's obligation is untouched — the edits below never altered ADR 0080, and `scripts/docs.py validate` passes.) That record quotes the exact stale wording under a `superseded-quotation` marker, which the gate reads as a claim that the span appears in none of the documents that paragraph links. The roadmap is not one of them, and the span there differs, so this work does not touch that obligation — but re-check `scripts/docs.py validate` rather than assuming it.

## Outcome

The surviving governed copy is corrected and the sibling class was swept. `docs/roadmap.md:326` now states the invariant — the fused-multiply-add permission is `Forbidden` in *every* numerical contract the compiler registers, naming `StrictF32NumericalContract::governed_profile` and both of its entries — rather than a count of registered constants. The sentence's conclusion about a device or library GEMM built on fused multiply-add accumulate is preserved verbatim. **Fact, read in full at `2305c4a`:** `governed_profile` returns `[governed(), governed_flush_to_zero()]`, `is_governed` tests membership in it, and both entries set `contraction` and `reassociation` to `NumericalPermission::Forbidden`; the two differ only in their two subnormal fields.

**The sibling sweep found one further genuinely wrong governed statement, in a second defect class.** `docs/open-questions.md:298` (Q-SEM-015's gate) asserted that `crates/tiler-compiler/src/capability.rs` and `crates/tiler-compiler/src/legality.rs` "are draft authorities with no in-crate production caller, so no occurrence can resolve a lowering provider". **Fact:** `crates/tiler-compiler/src/pipeline.rs:913` calls `resolve_lowering`, and `crates/tiler-compiler/src/lowering.rs:35-36` imports from both modules, so both are reached from the ordinary compile path. `correct-roadmap-capability-wiring-claims` corrected exactly this claim in `docs/roadmap.md` at two sites and left `docs/open-questions.md` behind — the same survivor pattern this ticket exists for, one class over. The gate itself is unaffected and now rests on the invariant that no registered lowering capability covers a *contraction* occurrence, which a fifth registered provider cannot silently falsify.

**Counts about the compiler's registered vocabulary that were checked and are still true**, each read at the construction site rather than inferred: the standard registry registers exactly one value type, `tiler::f32@1`, and four operations (`registry.rs`'s `StandardSemantics::register`); `tiler::strict-serial-sum-f32@1` is the only registered reduction and `OrderedReduction` the only reduction fusion role (`fusion_legality.rs`'s `FusionNumericalCapabilities::governed`, which registers four roles across four families); `OperationEffect` has exactly one variant, `Pure` (`semantic/operation.rs:792`); and `governed_index_access_capabilities` returns `[GovernedIndexAccess; 4]`, so "four governed index-access providers" holds. `docs/roadmap.md` lines 396, 398, 407, and 430 are therefore correct as written and were left alone — improving a true statement is not what this ticket is for.

**Corrections outside `docs/`, in `project/tickets`.** Each uses the corpus's established `Corrected by` marker rather than a silent rewrite, because all four tickets are `done` and their bodies are historical records a reader cites: `settle-contraction-chain-distributivity-permission` at two sites (the one-contract premise, and a `Fact` asserting `NumericalPermission` "has exactly one variant, `Forbidden`" — falsified by `widen-numerical-vocabulary-and-complete-identity` adding `Permitted`); `scope-einsum-contraction-support` at two sites (the one-contract premise, and the same falsified capability-reachability claim); and `correct-the-optimizer-one-variant-permission-claim`, whose own `## Outcome` re-verified and recorded the one-contract claim on 2026-07-24 and went stale the next morning.

**Deferred as merely improvable rather than wrong.** `tickets/select-numerical-contract-and-compose-feasibility.md:69` calls `governed` "currently the only contract the compiler registers", but that is the problem statement the same ticket then solves at lines 81, 102, and 140 by adding the second contract; it is internally coherent as a before-state. `tickets/declare-metal-numerical-honourability.md:59` and `tickets/widen-numerical-vocabulary-and-complete-identity.md:97` say "the registered contract is still `Preserve`/`Preserve`" about the specific golden-emission path, which uses `governed()`; singular there names the contract that path resolves, not the size of the registered set.

**Two of this ticket's own claims were wrong and are retracted in the body above.** Its reproduce command was said to return exactly one line; it returns two, the second being ADR 0080's deliberately retained `superseded-quotation`. Its `Closes when` consequently demanded that the unfiltered grep return nothing, which is unsatisfiable without breaking the marker it also told the worker not to weaken.

`uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass. ADR 0080 was never edited, and `validate_quotations` reports its `superseded-quotation` obligation satisfied.
