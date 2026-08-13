---
id: canonicalize-atomic-target-realization-declarations
title: Canonicalize atomic target realization declarations
status: in-progress
priority: p1
dependencies: []
related: [declare-cpu-vector-realization-facts-in-the-target-profile, admit-the-first-typed-synchronization-point-and-atomic-target-authority]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, identity, canonicalization, correctness]
claimed_from: todo
assignee: worker-canonicalize-atomic
lease_expires_at: 1786585709
---
## User-visible outcome

Two target profiles declaring the same atomic realization rows in different insertion orders have one descriptor identity, while duplicate or contradictory rows refuse.

## Fact — 2026-08-11

Checked synchronization facts are sorted by their canonical `(subject, phase)` key, but the public builder's complete-descriptor path does not sort synchronization declarations before encoding. Copying that precedent into vector declarations would make insertion order identity-bearing accidentally.

## Fact audit — 2026-08-12 at `612468048d541a1017640fc5dcbe5ff9160716cf`

**Verified.** The 2026-08-11 Fact is true at this base. Re-read every builder, checked population, encoder, and uniqueness check before editing.

- **Checked path sorts by `(subject, phase)`.** `CheckedTargetProfile::new_complete` runs `synchronization.sort_by_key(SynchronizationRealizationFact::sort_key)`. The key is documented as `The canonical sort and uniqueness key: the subject and its phase` and `Deliberately *excluding* the verdict`. After the sort, adjacent same-key pairs refused as `MalformedProfile { rule: "duplicate-synchronization" }`, covering both an exact restatement and a same-key contradiction with one rule.
- **Public `canonicalize` does not sort synchronization.** It sorts quantitative, queries, scalar, dispatchability, evaluation-order, and cost-row families. There is no `self.synchronization.sort_*` call.
- **Complete-descriptor encoding follows the builder's given order.** `complete_descriptor` writes `for declared in synchronization` after `tiler.target-profile.synchronization-realization.v1`. Its comment claimed `in the canonical order the builder sorted them into`, which was false: the builder never sorted this family. The family is encoded unconditionally (count may be zero), so a profile with two rows in opposite insertion orders currently mints two complete identities.
- **Insert-time uniqueness already excludes the verdict.** `declare_synchronization_realization` refuses a second row with the same subject and phase as `DuplicateSynchronizationRealization`, whether the verdict matches or contradicts. `validate_declarations` did not re-check this family, so a mutated draft could reach `freeze`.
- **No second atomic realization family exists yet.** Vector realization is still on `declare-cpu-vector-realization-facts-in-the-target-profile`. The only repeated atomic-realization row family is synchronization.
- **Identity domains.** Complete declaration is `tiler.target-profile.declaration.v11`. Checked descriptor is `tiler.target-profile.descriptor.v10`. The family separator is `tiler.target-profile.synchronization-realization.v1`. Sorting does not step any domain: the grammar is unchanged. Bytes move only for a profile that currently encodes two or more synchronization rows in non-uniqueness-key order. The governed profile declares none; the bound Metal profile declares one. The governed pin `the_governed_descriptor_bytes_do_not_move` is therefore unmoved.

## Required delivery

- Re-read every atomic realization builder, checked fact population, descriptor encoder, and duplicate/contradiction check.
- Canonically sort each repeated row family by its complete uniqueness key before both checked and complete descriptor encoding.
- Reject exact duplicates and same-key contradictory verdicts independently; never let sort order choose the winner.
- Perturb insertion order, duplicate, contradictory verdict, phase, source, and subject independently and quote the failure/output equality.

## Closes when

Atomic realization descriptor identity is order-independent and contradictions cannot coexist in any builder or decoded population.

## Worker delivery — 2026-08-12

Landed on `tkt/canonicalize-atomic-target-realization-declarations` from base `612468048d541a1017640fc5dcbe5ff9160716cf`. Not merged, not closed.

- `TargetProfileBuilder::canonicalize` now sorts synchronization by `DeclaredSynchronizationRealization::sort_key` — `(subject, phase)` — before `complete_descriptor` encodes. That is the same uniqueness key the checked constructor already sorted on.
- Exact duplicates and same-key contradictions refuse independently. Insert-time and `validate_declarations` still surface the accepted public `DuplicateSynchronizationRealization` (a second public variant would be a new type Tom has not accepted). The checked constructor now distinguishes `duplicate-synchronization` from `contradictory-synchronization`, so a decoded population cannot keep either case.
- Perturbations, independently:
  - Insertion order: `atomic_realization_insertion_order_is_not_identity` and `checked_synchronization_rows_canonicalize_independently_of_insertion_order` — opposite declaration orders share one complete descriptor and one checked descriptor; stored rows are uniqueness-key order.
  - Exact duplicate: `an_exact_duplicate_atomic_realization_is_refused_before_insertion` quotes `DuplicateSynchronizationRealization` and inserts nothing; `an_exact_duplicate_synchronization_declaration_is_malformed` quotes `duplicate-synchronization`.
  - Contradictory verdict: both `Realized` then `Unrealizable` and the reverse quote `DuplicateSynchronizationRealization` at insert time and `contradictory-synchronization` in the checked population. Sort order does not choose a winner.
  - Phase: same subject at `CompileProfile` and `LiveDevicePreflight` coexist; reverse declaration order does not move either descriptor.
  - Source: two sources at one `(subject, phase)` refuse as `DuplicateSynchronizationRealization`. Two profiles that differ only in source revision have unequal complete descriptors (`tiler.target-profile.declaration.v11`) and equal checked descriptors (`tiler.target-profile.descriptor.v10`).
  - Subject: a neighbouring fence set, a different kind, and a different verdict each move both descriptors.
- Identity domains named above were not stepped. `the_governed_descriptor_bytes_do_not_move` still matches its pin.

Checks: `cargo nextest run -p tiler-compiler` (839 passed); `cargo test -p tiler-compiler --doc`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `tkt lint`; `git diff --check`; `tkt guard tkt/canonicalize-atomic-target-realization-declarations --base 612468048d541a1017640fc5dcbe5ff9160716cf`.
