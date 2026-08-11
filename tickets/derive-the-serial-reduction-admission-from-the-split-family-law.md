---
id: derive-the-serial-reduction-admission-from-the-split-family-law
title: Derive the serial reduction admission from the split family law
status: in-progress
priority: p3
dependencies: []
related: [admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: sol-serial-admission
lease_expires_at: 1786414720
---
## User-visible outcome

A reduction family states its contributor tensor, empty-domain obligation, and reassociation consumption in one place read by all three topologies; a fifth family becomes one `SplitFamily` arm instead of one match arm per topology.

## Why this exists (vocabulary audit 2026-08-06; the auditor hand-verified no serial/parallel divergence exists today — this is prophylactic, and saying so is the point)

At base `678f805ed641183067f8a63754b169f21575c6b7`, `SplitFamily`'s own doc states the law ("a family admitted by one admission and not the other would otherwise be a difference nobody states"); `multi_pass_family` and `cooperative_family` implement it; the **five** serial fold arms under `fn verify_access_and_semantics` do not — nine byte-identical conjuncts repeat across all five beside the family-derived axes, order, contributor-tensor obligation, and empty-domain obligation. The fused family additionally refuses `contraction`, while `consumes_reassociation` is deliberately parallel-only and is not a serial residual. The empty-domain rule is also spelled four times outside `empty_domain_is_satisfied` whose doc claims coverage "at each admission". The four multi-topology families (sum, fused, squared, maximum) are the `SplitFamily` population; `SquaredSerialSumThenEpilogue` is a fifth serial-only arm that repeats the nine shared conjuncts and adds epilogue residuals while both parallel tables answer `None`. VERIFY the nine-conjunct claim by reading the five arms before starting; any shared derivation must either carry ThenEpilogue as serial-only post-checks or leave it as an explicit residual arm rather than silently drop it.

## Boundaries

`consumes_reassociation` has no serial meaning and must not acquire one (a serial fold spends nothing). The extrema non-emptiness precondition folds into `empty_domain_is_satisfied`, not a fourth rule. No admission widens or narrows — bit-identical admitted sets, pinned by the unchanged canonical-identity tests. All types private; no boundary, no encoder.

## Graph repair — 2026-08-10

Related [`admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary`](admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary.md) is **done** at base `678f805ed641183067f8a63754b169f21575c6b7`. Its `DeclaredDomain` and fused declared-input widenings are already present in the serial, multi-pass, and cooperative admissions, so this refactor now derives their agreement after that landing instead of sequencing concurrently with it. There was no hard dependency either way.

**Census correction.** Prior prose said "four serial arms"; at base `c99ac54950f2` the serial match has five fold arms (`serial_fold_families` asserts `families.len() == 5`). Production comment "at all four" above the serial match is the same stale census and is not authority.

## Closes when

One shared conjunction with a per-family derivation function; a test asserting per-family agreement across the three admissions, watched failing under a perturbed serial read tensor; identity pins unchanged; the five-family unit population still forces every serial fold arm.

## Implementation evidence — 2026-08-10

**Fact.** Complete-file audit at the exact base verified five serial fold arms, four parallel-shared families, nine common serial conjuncts, four duplicated `+0.0` empty-domain checks, maximum's separate non-empty check, and `SquaredSerialSumThenEpilogue` as the only serial-only family. It also verified that `consumes_reassociation` was read only by the two parallel gates. The graph statement above was false because the related ticket was already done; the phrase "three genuinely differing family facts" was imprecise because reassociation consumption is not a serial fact. Both statements were repaired before implementation.

**Fact.** `split_family` is now the single family table. The serial gate reads its axes, order, contributor tensor, and empty-domain contract through one conjunction; both parallel gates read the same table and derive their topology-specific contributor tensor. `consumes_reassociation` remains read only by the parallel gates. The epilogue family is explicitly `SerialOnly`, and its validity, one-input, and non-identity-root residuals remain in the serial gate. A fused program requesting contraction remains refused by all three admissions.

**Evidence.** `shared_families_admit_the_same_contributor_tensors_in_every_topology` checks all four shared families against first input, nonzero input, intermediate, and output roles through the production serial, partial, and cooperative gates. `the_scalar_program_population_derives_five_serial_and_four_parallel_families` is sized by `variant_count::<ScalarProgram>()`, so a vocabulary widening without a new classification is a build error. Changing only the production serial read predicate to `Exactly(FIRST_INPUT)` made the agreement test fail with `serial strict serial sum reading Input { ordinal: InputOrdinal(1) }`, `left: false`, `right: true`. Independently changing the epilogue family from `SerialOnly` to `Split { final_pass: false }` made its refusal test fail with `left: Split { final_pass: false }`, `right: SerialOnly`. Both production perturbations were restored and the controls reran green.

**Identity evidence.** A sorted census of every 16–64-character lowercase hexadecimal run under `crates/` is byte-for-byte identical between the exact base and this tree. The only `schedule/model.rs` change is the private verifier-name documentation; no encoder, domain, version, tag, pin, or public type changed. The complete `tiler-ir` nextest run passed all 1,012 tests, including every recorded canonical-identity test.
