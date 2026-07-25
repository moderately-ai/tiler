---
id: correct-the-surviving-stale-one-contract-claims
title: Correct the surviving stale one-contract claims outside ADR 0080
status: todo
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

**Fact — the surviving governed copy.** `docs/roadmap.md:326` reads that the fused-multiply-add permission "is `NumericalPermission::Forbidden` in the only numerical contract the compiler registers today (`StrictF32NumericalContract::governed` in `crates/tiler-compiler/src/request.rs`)". Reproduce: `grep -rn "only numerical contract" docs/`, which returns exactly this one line.

**Inference — the conclusion is unaffected and only the premise's arithmetic is stale**, which is the same finding ADR 0080 recorded for `docs/numerical-semantics.md` and `docs/compiler/optimizer.md`. Both registered contracts set `contraction: NumericalPermission::Forbidden`, so a device or library GEMM built on fused multiply-add accumulate still does not implement the declared semantics under either. The sentence should name the profile function rather than one constant, exactly as ADR 0080's own statement of the fact now does.

## Also stale, in `project/tickets` rather than a contract

Two ticket bodies carry the same premise and are listed so a reader is not left to rediscover them. Neither is a governed contract and neither changes a conclusion, so correcting them is lower value than the roadmap line and is bundled here rather than split again.

- `tickets/settle-contraction-chain-distributivity-permission.md:19` states `StrictF32NumericalContract::governed` "is the only numerical contract the compiler registers and sets `reassociation` to `NumericalPermission::Forbidden`". Both registered contracts do.
- `tickets/scope-einsum-contraction-support.md:39` states the permission is `Forbidden` "in the only numerical contract the compiler registers".

`correct-the-optimizer-one-variant-permission-claim` is a separate live ticket over a *different* stale claim in the same neighbourhood — that `NumericalPermission` has exactly one variant — and is not superseded by this one. Read it before editing `docs/compiler/optimizer.md`, because both tickets can reach that file.

## Closes when

No governed document states that the compiler registers one numerical contract, `grep -rn "only numerical contract" docs/` returns nothing, the roadmap sentence's conclusion about fused multiply-add accumulate is preserved unchanged, and `uv run --locked python scripts/docs.py render` plus `uv run --locked python scripts/check_repository.py` pass.

**Do not** weaken ADR 0080's restored quotation while doing this. That record quotes the exact stale wording under a `superseded-quotation` marker, which the gate reads as a claim that the span appears in none of the documents that paragraph links. The roadmap is not one of them, and the span there differs, so this work does not touch that obligation — but re-check `scripts/docs.py validate` rather than assuming it.
