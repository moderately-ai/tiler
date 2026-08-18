---
id: size-the-deferred-subject-axis-censuses-from-the-type
title: Size the deferred-subject axis censuses from the type
status: done
priority: p2
dependencies: [carry-subgroup-width-through-exact-prepared-entry-equality]
related: [generalize-deferred-target-provenance-beyond-capability-axes]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A widened `CapabilityAxis` vocabulary cannot silently shrink the test populations that guard the deferred-subject canonical ordering and the append-only proposal-identity encoding.

## Why this exists — filed 2026-08-18 from the multi-lens audit of the deferred-provenance landing (merge `3dc10348`)

**Fact.** `crates/tiler-compiler/src/target/feasibility.rs` (test module) and `crates/tiler-compiler/src/frontier.rs` (test module `deferred_subject_identity`) each carry a hand-written `CANONICAL_AXES: [CapabilityAxis; 7]`, and the frontier module additionally hand-writes an axis→relation map (`capability_relation`). The properties they guard are load-bearing for identity: `capability_axis_descriptor_tags_ascend_with_the_derived_order` proves the canonical-key sort preserves the pre-enum `(phase, axis)` order, and `capability_records_keep_their_pre_enum_bytes_exactly` / `atomic_deferred_subjects_are_structurally_disjoint_from_every_axis` prove byte preservation and escape disjointness — all feeding `encode_proposal_identity`'s iteration order and bytes.

**Fact.** An eighth `CapabilityAxis` variant added to the enum but omitted from these arrays passes every one of those checks silently — the AGENTS.md "population that silently shrinks" failure mode. Mitigation exists but is indirect: `tag()` and `key()` are exhaustive matches, so a new variant forces *some* edit, but nothing forces the census arrays to grow.

**Fact.** `tiler-compiler` already uses `core::mem::variant_count` (its `lib.rs` sizes `request::BudgetResource::ALL` from the enum), so the sizing pattern and any required feature gate are already present in this crate. (The audit report claimed the crate had no `variant_count` use; that claim was false — verified 2026-08-18 by `grep -rn variant_count crates/tiler-compiler/`.)

## Required content

- Assert `CANONICAL_AXES.len() == core::mem::variant_count::<CapabilityAxis>()` (or size the arrays from a single shared, type-sized source) in both test modules, so a widened vocabulary is a red test at the census rather than a silently shrunk population. Deduplicate the two hand-written arrays and the hand-written relation map if one shared spelling is cleanly reachable from both modules.
- Perturb the subject, not the assertion: demonstrate with a scratch eighth variant that the hardened census goes red, and quote the failure text.
- Secondary, same files: `capability_records_keep_their_pre_enum_bytes_exactly` builds its "legacy" bytes from the new encoder's own expression (`predicate.requirement().required()`), while the pre-enum encoder wrote `predicate.required().value()`; the two agree only because `Quantity::value()` is the identity unwrap for every axis. Either derive the legacy bytes independently of the new accessor chain or document at the test that it is a regression pin against future structural change, not an independent pre-enum control.

## Closes when

Both censuses are type-sized (or floor-asserted with a printed census), the scratch-variant perturbation's failure text is recorded, and the byte-control's independence status is either repaired or truthfully documented.

## Fact audit at base `9680b579b547d0f92f19627b18e90745d4db4be4` — worker-axis-census, 2026-08-18

**Fact 1 — imprecise.** The frontier array is where the ticket says it is: `frontier.rs`, inside `mod deferred_subject_identity`, anchored by `const CANONICAL_AXES: [CapabilityAxis; 7] = [`. The feasibility one is **not in a test module**. It sits at module scope in `crates/tiler-compiler/src/target/feasibility.rs` under the doc line `The canonical axis order. This is the single source of truth for evaluation`, above `mod tests` (which opens at the file's only `#[cfg(test)]`). All three of its use sites are nevertheless inside that test module; the file's crate-level `#![allow(dead_code,` is why a production const used only by tests raises no warning.

The imprecision is load-bearing rather than cosmetic. `crates/tiler-compiler/src/lib.rs` gates the nightly feature as `#![cfg_attr(test, feature(variant_count))]`, so the production array's *type* cannot be written `[CapabilityAxis; core::mem::variant_count::<CapabilityAxis>()]` the way `request::BudgetResource::ALL` and `boundary.rs`'s `CARRIERS` are — both of those are `#[cfg(test)]` items. The sizing therefore had to be an assertion in the test module rather than the array's own length, and the ticket's parenthetical "or size the arrays from a single shared, type-sized source" is what was achievable.

**Fact 2 — verified, by measurement.** Scratch `ScratchEighthAxis` added to `CapabilityAxis` with every wildcard-free match arm supplied — `tag`, `key`, `relation`, `quantity` in `feasibility.rs` and `capability_relation` in the frontier test module, five in total — and omitted from both census arrays:

```sh
cargo nextest run -p tiler-compiler
#      Summary [  12.859s] 957 tests run: 957 passed, 1 skipped
```

The first attempt failed to compile at `crates/tiler-compiler/src/frontier.rs:7018` with `E0004: non-exhaustive patterns: feasibility::CapabilityAxis::ScratchEighthAxis not covered`, which is the ticket's "mitigation exists but is indirect" observed directly: the relation map forces an arm, and once that arm is written nothing else objects. Scratch reverted; tree byte-identical (`git status --porcelain` empty at `9680b579`).

**Fact 3 — verified.** `grep -rn variant_count crates/tiler-compiler/` returns ten hits across `domains.rs`, `request.rs`, `boundary.rs`, `lib.rs`, and `physical.rs`. The audit report's contrary claim was false, as the ticket already records.

## What landed

**One census, not two.** The frontier duplicate is deleted; `mod deferred_subject_identity` now imports `crate::target::feasibility::CANONICAL_AXES`, which widened from private to `pub(crate)`. That one-word production change is the whole non-test diff, and it is what makes a single shared type-sized source reachable — the alternative was two arrays each carrying its own assertion, which is the duplication the ticket asked to remove.

**Completeness is a build error, coverage is the existing test.** `CANONICAL_AXES_COVER_THE_CAPABILITY_VOCABULARY` in `feasibility.rs`'s test module asserts `CANONICAL_AXES.len() == core::mem::variant_count::<CapabilityAxis>()`. Length alone would admit an array that repeats one axis and omits another, so it composes with `capability_axis_descriptor_tags_ascend_with_the_derived_order`, which already asserts strict `<` over `windows(2)`: full length plus strict ascent under the derived `Ord` is exactly the vocabulary, once each, in declaration order.

**`capability_relation` stays hand-written.** It is not a census that can shrink — it is a wildcard-free match, so a widened vocabulary is already a build error at the arm, as Fact 2's first compile shows. Consolidating it onto `CapabilityAxis::relation` would be worse than redundant: `DeferredPredicate::new` validates the pair against that same production map, so a fixture deriving its relation from it would construct trivially and assert nothing. Kept and documented rather than deduplicated.

## Perturbation — the census goes red

Same scratch eighth variant, same five match arms, against the hardened tree. `cargo nextest run -p tiler-compiler` fails to build with this as the *only* error:

```text
error[E0080]: evaluation panicked: CANONICAL_AXES has stopped naming every CapabilityAxis: the evaluation order, the deferred-subject byte control, and the escape-disjointness census all range over it, and each would pass over the axes that remain.
    --> crates/tiler-compiler/src/target/feasibility.rs:3972:64
     |
3972 |       const CANONICAL_AXES_COVER_THE_CAPABILITY_VOCABULARY: () = assert!(
     |  ________________________________________________________________^
3973 | |         CANONICAL_AXES.len() == core::mem::variant_count::<CapabilityAxis>(),
...
     | |_____^ evaluation of `target::feasibility::tests::CANONICAL_AXES_COVER_THE_CAPABILITY_VOCABULARY` failed here
```

Named rather than `const _` so the diagnostic carries the anchor; the run confirms a named module-level const is evaluated without any use forcing it. The perturbation was reverted from the index and the working tree confirmed to hold only the hardening.

## Byte control — repaired, and what stays shared

Repaired rather than documented-as-a-pin, because the repair was available: `capability_predicate` requires a named `FIXTURE_REQUIRED: u64 = 1`, and the legacy reconstruction now spells `FIXTURE_REQUIRED.to_be_bytes()` instead of `predicate.requirement().required().to_be_bytes()`. The quantity therefore comes from the value the fixture put in rather than from the expression under test. `1` is admissible on every axis: the boolean axes admit `value <= 1`, the exact axis `value > 0`, ceilings anything.

Two fields stay shared with the encoder, and the test now says so: `CapabilityAxis::key` and `PreparedEntryTargetRequirement::canonical_bytes`. Both were shared pre-enum too — they are the axis's one governed spelling and the requirement's own governed encoding — so restating either would assert a second opinion about bytes this module does not own. The claim in the test's doc comment is now exactly that, not the broader "independent pre-enum control".

**Unverified.** The byte control was not perturbed. Breaking its subject means editing `encode_deferred_predicate` (for example writing the required quantity little-endian), and that edit was refused by the environment's tool policy; retrying it would have been working around the refusal rather than around a mistake. Its liveness is inherited rather than demonstrated: it compares against real encoder output and passes, and the change re-sourced one of three reconstructed fields to a value that is equal today. A reviewer with encoder-edit permission should make the little-endian perturbation and confirm `capability_records_keep_their_pre_enum_bytes_exactly` reddens.

**Verified at integration — 2026-08-18, by the coordinator.** The little-endian perturbation was made in the lane worktree at this commit: `encode_deferred_predicate`'s capability arm changed `to_be_bytes` to `to_le_bytes` on the required quantity alone. `cargo nextest run -p tiler-compiler -E 'test(capability_records_keep_their_pre_enum_bytes_exactly)'` failed with `assertion `left == right` failed: the grid-axis capability record moved under the subject vocabulary`, the two byte vectors differing exactly at the eight-byte required-quantity field (`… 1, 0, 0, 0, 0, 0, 0, 0 …` against `… 0, 0, 0, 0, 0, 0, 0, 1 …`) with every framed field identical. The perturbation was reverted and the tree confirmed byte-identical to the delivered commit. The byte control is demonstrated live, not inherited.

## Commands

```sh
cargo nextest run -p tiler-compiler          # 957 passed, 1 skipped
cargo clippy -p tiler-compiler --all-targets -- -D warnings
cargo fmt --check
git diff --check
tkt lint
tkt guard tkt/size-the-deferred-subject-axis-censuses-from-the-type --format json
```
