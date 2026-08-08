---
id: pin-the-gather-request-boundary-refusal-with-a-test
title: Pin the gather's request-boundary refusal with a test that names the key
status: todo
priority: p2
dependencies: []
related: [admit-the-indirect-access-class-into-the-index-layer, admit-an-indirect-gather-family-for-tied-embedding-lookup, accept-adr-0107-indirect-gather-semantic-family]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, gather, testing, fail-closed]
---
## User-visible outcome

The claim that a program stating `tiler::gather-f32@1` fails closed at the request boundary is discharged by a test that compiles such a program and observes the refusal, rather than by a policy-table inventory that never reaches either the fusion authority or the boundary.

## Why this exists

[ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md)'s acceptance note states, as the mitigation for its own accepted counterpoint, that "the fail-closed boundary is tested rather than asserted, with `classify` returning `None` so no region derives legality". That mitigation is what the reader-trap risk was accepted against, so it is load-bearing.

**Fact, verified at `cb62784c`.** `grep -rn 'gather_f32_op' crates/tiler-compiler/` returns nothing — exit 1, no hits — so the key is not named anywhere in the compiler crate. `grep -rn 'gather' crates/tiler-compiler/tests/ | wc -l` returns `0`. No test compiles a gather program, and none names the key against `FusionNumericalCapabilities::classify` or against the request boundary.

**Fact.** The only compiler-side coverage is `every_unplanned_operation_is_registered_and_consumes_no_dimension` and `the_capability_table_names_exactly_the_admitted_operations`, both in `crates/tiler-compiler/src/policy.rs`'s `#[cfg(test)] mod tests`. They assert over `operation_capability` (`policy.rs`, `pub(crate) fn operation_capability`), which is a **different authority** from `classify` (`fusion_legality.rs`, `fn classify`) — a distinction `policy.rs`'s own prose draws for the BF16 rows: "that table is a different authority from this one". Neither test constructs a program, and neither reaches `request.rs`.

So the assertion in the record and the check in the tree are about different things, which is the shape [AGENTS.md](../AGENTS.md) warns about under "Verify that a check reaches its subject at all".

## What this must deliver

- A test that builds a semantic program containing one `tiler::gather-f32@1` occurrence and asserts the compile refuses, naming the exact error. The expected site is `recognize_epilogue_producer`'s final arm in `crates/tiler-compiler/src/request.rs`, which reaches `mismatch("operation-set")` because `family_realizes_region_sequence` is false for a key carrying no registered `IndexRealizationLaw`; confirm that rather than assume it, because several other sites also spell `operation-set`.
- A test that asserts `classify` returns `None` for the key, so the sentence the record's mitigation actually makes has a check behind it.
- Perturb each separately and quote the failure: registering a stub realization law should redden the first, and inserting any `FusionOperationRole` for the key should redden the second. A perturbation that reddens both has not shown which assertion is load-bearing.

## Non-goals

Changing the refusal, the family, or the policy inventory. Admitting anything into the index layer — [ADR 0108](../docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md) holds that and defers it.

## Closes when

Both tests exist, both have been shown to fail under their own perturbation with the message quoted, and ADR 0107's dated correction is updated to point at them.
