---
id: state-and-check-a-bf16-numerical-contract
title: State and check a BF16 numerical contract
status: review
priority: p1
dependencies: [declare-the-bf16-rows-on-the-authoritative-metal-profile]
related: [admit-bf16-into-the-schedule-and-kernel-vocabulary, design-the-bf16-computation-and-accumulator-contract]
scopes: [implementation/compiler, implementation/ir, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, bf16, numerics, target-profiles]
claimed_from: todo
assignee: agent-bf16-contract
lease_expires_at: 1785945916
---
## User-visible outcome

A caller can state the pure-BF16 numerical contract that the registered
constant/multiply/add family already defines, and target feasibility checks that
contract against the exact BF16 arithmetic subject. On the measured macOS
Apple9 profile, a strict subnormal-preserving request is refused with a typed
numerical gap naming BF16 and the profile's measured sign-preserving flush; a
flush-accepting request reaches the next independently unsupported layer. No
F32 contract entry, profile row, or arithmetic behaviour is inherited.

## Why this is a separate boundary

**Fact, at `aa09b5e`.** `ScalarArithmetic::new` can now construct the validated
`(ArithmeticType::Bf16, Bf16::resolved_type())` subject, and
`TargetProfileBuilder::declare_measured_*_subnormal_behaviour` can state its
complete exclusive tables. `declare-the-bf16-rows-on-the-authoritative-metal-profile`
can therefore add the measured BF16 rows without changing this ticket's scope.

**Fact.** The public request contract cannot ask the corresponding question.
`NumericalContract` contains one dimension vector documented as "Every
resolution is stated for `f32`"; its only entry point is
`NumericalContractBuilder::strict_f32`; and `NumericalContract::resolve`
constructs `StrictF32NumericalContract`. A pure-BF16 semantic program is refused
at the request boundary with `dtype-f32` before target numerical feasibility,
so adding a BF16 profile row does not make a strict BF16 preservation refusal
observable through `compile`.

**Inference.** Treating the existing `STRICT_F32` contract as a BF16 contract
would silently transfer per-dtype behaviour across a boundary the retained
Apple measurements prove differs (`f16` preserves where `f32` and BF16 flush).
Adding a test-only target resolver in `tiler-build` would prove a path no caller
can use. The missing work is therefore a compiler numerical-contract boundary,
not part of transcribing the measured profile rows.

## Scope keys

- Run the AGENTS.md elimination over the durable shape: a per-arithmetic
  contract, a contract entry carrying its exact `ScalarArithmetic`, or another
  consumer-neutral representation. Reject any shape that defaults an omitted
  dtype, makes one contract silently apply to every float width, or lets the
  profile choose program meaning.
- Preserve every existing F32 contract key and meaning byte-for-byte. New BF16
  meaning receives a distinct, canonical identity; enumerate and recompute
  every moved pin on the tree the identity step lands into.
- Keep computation and accumulator types on the operation facts, as accepted by
  ADR 0091. This ticket states numerical permissions and required behaviours;
  it does not introduce implicit promotion or a mixed-precision operation.
- The request must check the BF16 contract before physical planning. A strict
  preservation requirement must report the profile's measured
  `DeclaredUnhonourable` fact, not degrade into `Unknown`, a generic dtype
  refusal, or a late Metal emission error.
- Consequential public constructor, builder, or call-site changes remain Tom's
  decision under ADR 0075. A tested implementation is a draft until accepted.

## Required evidence

- A pure-BF16 constant/multiply/add program states a strict preserving contract
  and the macOS Apple9 profile refuses it with `ArithmeticType::Bf16`,
  `Bf16::resolved_type()`, `SubnormalMode::Preserve`, `Unsupported`, and the
  honoured sign-preserving flush, all from the exact measured source.
- The same program under a sign-preserving-flush contract passes numerical
  feasibility and reaches the next named unsupported layer without implying
  schedule, kernel, lowering, or runtime support.
- An F32 contract does not answer for BF16 and a BF16 contract does not change
  any existing F32 compile outcome or key.
- A mutation deleting the BF16 profile declaration turns the refusal into
  `Unknown`; a mutation substituting F32 for the subject is detected.
- Every identity movement and public-boundary consequence is enumerated, with
  targeted fmt/check/clippy/nextest/doc-tests, `tkt lint`, `git diff --check`,
  and `tkt guard` green.

## Closes when

The pure-BF16 numerical contract is statable and checked before physical
planning; the measured macOS row produces the named preservation refusal and a
flush-accepting request passes that dimension; no neighbouring dtype or F32
identity moves silently; and Tom has accepted every consequential public
boundary.

## Added scopes, and why each is required

- `implementation/ir`. The scope key "new BF16 meaning receives a distinct,
  canonical identity" cannot be met inside the compiler: `canonical_contract_key`
  mints through `F32NumericalContractKey`, which lives in
  `crates/tiler-ir/src/schedule/numerics.rs` and hard-codes the `f32` domain,
  arithmetic tag, and NaN payload. Minting a BF16 key in the compiler instead
  would put contract-key grammar in two crates and produce an identity
  `tiler_ir::index::refinement::NumericalContractIdentity` cannot validate. The
  edit adds `Bf16NumericalContractKey` beside its `f32` sibling and factors the
  shared dimension writer; no `f32` byte moves.
- `contracts/artifacts`. `docs/artifact-abi.md` is the identity ledger and it
  names the numerical-contract key domain. Adding a second domain without moving
  the ledger sentence leaves a stale assertion the next reader builds on. The
  scope name was read from `ticketsplease.toml` after a first attempt declared
  `contracts/foundation` from memory and `tkt guard` refused the branch for
  under-declaration; `docs/artifact-abi.md` maps to `contracts/artifacts`.

Both are scheduling metadata for already-authorized work, declared under the
AGENTS.md rule that an agent adds every required scope autonomously.

**Live-scope overlap, verified rather than assumed.** `implementation/ir` is held
exclusively by `root-cause-the-intermittent-leaky-test-in-the-workspace-gate`.
Its branch had zero commits at the time of this edit —
`git log --oneline main..tkt/root-cause-the-intermittent-leaky-test-in-the-workspace-gate`
was empty and `git merge-base` resolved to this ticket's own base
`e9ef24dcb106a71696d702cf2be60cf7a403fe95` — so file-level disjointness against
its *actual* diff is vacuous rather than informative. The integrator must re-run
that check before merging, treating `crates/tiler-ir/src/schedule/numerics.rs`
and `crates/tiler-ir/src/schedule/mod.rs` as the files at risk.

## Graph maintenance

- Depends on `declare-the-bf16-rows-on-the-authoritative-metal-profile`, which
  owns the measured fact this request must consume. The provider does not depend
  on its first consumer; keeping the edge this direction avoids a cycle.
- Related to `admit-bf16-into-the-schedule-and-kernel-vocabulary`: this ticket's
  first positive request may still stop at that independently unsupported layer,
  and must name rather than absorb it.
- Related to `design-the-bf16-computation-and-accumulator-contract`, whose
  accepted outcome keeps accumulator width on operation identity and forbids a
  fused BF16 operation. Do not reopen either decision here. **Fact.** Neither was
  reopened: the contract states numerical permissions and required behaviours
  only, and no computation or accumulator type moved onto it.
- **Fact — the named unsupported layer is the recognizer, not the schedule
  vocabulary.** A flush-accepting BF16 request clears numerical feasibility and
  is then refused by `select_supported_strategy`'s `dtype-f32` rule, a
  whole-request `UnsupportedCapability`. It never reaches
  `admit-bf16-into-the-schedule-and-kernel-vocabulary`'s layer, because
  recognition sits above it. That ticket's relation stands and its wall is the
  next one after this one, not this one.
- **Fact — the measured ledger's BF16 rows cover the two subnormal dimensions
  only.** So on `FIRST_MACOS_APPLE9`'s own rows a flush-accepting BF16 contract
  meets `Unknown` on the first remaining consumable dimension (contraction)
  rather than passing every dimension. That is the correct answer for the
  measurement's boundary and is asserted as its own case; widening those rows is
  the measured-profile ticket's business, not this one's.
- Split out, because this ticket's scopes cannot reach either:
  `bind-the-bf16-contract-refusal-to-the-authoritative-apple9-rows`
  (`implementation/build` — `FIRST_MACOS_APPLE9` lives in a crate that depends on
  the compiler) and
  `move-the-navigation-docs-onto-the-two-contract-key-domains`
  (`contracts/navigation` — held by a live ticket at the time of this edit).
