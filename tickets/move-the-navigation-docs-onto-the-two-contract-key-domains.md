---
id: move-the-navigation-docs-onto-the-two-contract-key-domains
title: Move the navigation docs onto the two numerical-contract key domains
status: review
priority: p2
dependencies: []
related: [state-and-check-a-bf16-numerical-contract]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, bf16]
claimed_from: todo
assignee: agent-nav-domains
lease_expires_at: 1785949101
---
## User-visible outcome

Every navigation document that names the numerical-contract key domain names
both of them, and no reader is left with the pre-BF16 statement.

## Why this is a separate ticket

**Fact.** `state-and-check-a-bf16-numerical-contract` added the sibling domain
`tiler.contract.bf16.v1` beside `tiler.contract.f32.v2`, updated the two
contracts it holds (`docs/numerical-semantics.md`,
`docs/correctness-and-testing.md`) and the identity ledger
(`docs/artifact-abi.md`, `contracts/foundation`), and could not reach
`contracts/navigation`: that scope was held exclusively by the live
`derive-the-operation-family-and-signature-delivery-graph`. Editing a scope
another live ticket holds is admissible only against a verified file-level
disjointness check, and that sibling's branch had no commits to check against,
so the edit was split out rather than taken on an empty verification.

## Scope keys

- `docs/status.md` states "The numerical-contract key domain is
  `tiler.contract.f32.v2`". It is now one of two, and the `bf16` domain was
  added rather than stepped, so no pin moved — say both.
- `docs/open-questions.md` Q-SEM-001's close note describes the key as the
  encoding "under `tiler.contract.f32.v2`". Same correction; the close itself
  still holds.
- `docs/dtype-support.md` should record that a BF16 *numerical contract* is now
  statable and checked, while BF16 *execution* remains unsupported — the
  distinction the compiler test asserts rather than assumes.
- Nothing here changes an identity or a decision; it is a stale-assertion sweep.

## Required evidence

- Every occurrence found by reading, not only by grepping the domain string:
  the exact check is `rg -n 'contract\.f32\.v2|numerical-contract key domain' docs/`,
  and each hit is read in place before editing.
- `tkt lint` green.

## Closes when

No navigation document asserts a single numerical-contract key domain, and the
BF16 statable/unsupported distinction is stated where dtype support is
catalogued.

## Outcome

**Fact — the sweep found more than the three enumerated sites, and the extra
hits were pre-BF16 statements rather than domain spellings.** `grep -rn
'tiler.contract.f32' docs/` returned 5 hits at the base and returns 7 after;
`docs/artifact-abi.md` (2) and `docs/numerical-semantics.md` (1) already named
both domains, having been updated by the producing ticket. The two that asserted
one domain were the enumerated `docs/status.md` and `docs/open-questions.md`, and
both now name the sibling. The one surviving `f32`-only hit is
`docs/artifact-abi.md:233`, which is the record of the `f32` domain's *own*
`v1`-to-`v2` step and is correct as written — the paragraph two below it states
the sibling addition. It is also outside this ticket's scope
(`contracts/artifacts`).

**Fact — four further stale assertions, all in `contracts/navigation`, all
corrected here.** `docs/status.md`'s Metal-profile bullet said "no public BF16
numerical contract or BF16 backend vertical is implied"; the contract half is
now false. `docs/dtype-support.md` said `compile()` refuses a BF16 program "at
the request boundary with `dtype-f32`" — true only on a profile that declares
the dtype, since target resolution now runs above recognition — and said the
BF16 caller-contract ticket "remains reserved on that boundary". `docs/roadmap.md`
said `select_supported_strategy` refuses "before any target is consulted", that
"the *caller* still cannot ask the corresponding question", that
`state-and-check-a-bf16-numerical-contract` is `todo`, and that "no BF16 variant
exists in `KernelType`, `StorageScalar`, `KernelConstant`, or `BinaryOp`" — the
last refuted by `crates/tiler-ir/src/kernel/model.rs:112` and
`crates/tiler-ir/src/program/model.rs:350`. Its live-gate count and two ticket
statuses were recounted from the board.

**Fact — two defects found by the sweep were filed rather than absorbed.**
`record-the-landed-bf16-carrier-in-the-dtype-ledger` (`contracts/navigation`)
owns the dtype ledger's BF16 physical-carrier and kernel-vocabulary *cells*,
because moving a maturity cell is a promotion needing the landing commit read in
full rather than a navigation sweep's grep.
`correct-the-stale-bf16-compiler-doc-comments` (`implementation/compiler`, which
this ticket cannot reach) owns a doc comment naming a test that does not exist
and one attributing the BF16 refusal to the wrong boundary.
