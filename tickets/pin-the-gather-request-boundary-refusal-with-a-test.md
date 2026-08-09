---
id: pin-the-gather-request-boundary-refusal-with-a-test
title: Pin the gather's request-boundary refusal with a test that names the key
status: todo
priority: p2
dependencies: []
related: [admit-the-indirect-access-class-into-the-index-layer, admit-an-indirect-gather-family-for-tied-embedding-lookup, accept-adr-0107-indirect-gather-semantic-family]
scopes: [implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, gather, testing, fail-closed]
---
## User-visible outcome

The claim that a program stating `tiler::gather-f32@1` fails closed at the request boundary is discharged by a test that compiles such a program and observes the refusal, rather than by a policy-table inventory that never reaches either the fusion authority or the boundary.

## Why this exists

[ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md)'s acceptance note states, as the mitigation for its own accepted counterpoint, that "the fail-closed boundary is tested rather than asserted, with `classify` returning `None` so no region derives legality". That mitigation is what the reader-trap risk was accepted against, so it is load-bearing.

**Fact, verified at `ceda5be0`.** `rg -n 'gather_f32_op' crates/tiler-compiler` returns no hits, and `rg -n 'gather' crates/tiler-compiler/tests` returns no hits. No test compiles a gather program, and none names the key against `FusionNumericalCapabilities::classify` or against the request boundary.

**Fact.** The only compiler-side coverage is `every_unplanned_operation_is_registered_and_consumes_no_dimension` and `the_capability_table_names_exactly_the_admitted_operations`, both in `crates/tiler-compiler/src/policy.rs`'s `#[cfg(test)] mod tests`. They assert over `operation_capability` (`policy.rs`, `pub(crate) fn operation_capability`), which is a **different authority** from `classify` (`fusion_legality.rs`, `fn classify`) — a distinction `policy.rs`'s own prose draws for the BF16 rows: "that table is a different authority from this one". Neither test constructs a program, and neither reaches `request.rs`.

So the assertion in the record and the check in the tree are about different things, which is the shape [AGENTS.md](../AGENTS.md) warns about under "Verify that a check reaches its subject at all".

**Fact — the observable refusal has two ordered layers.** `verify_request` calls
`require_compile_profile_dispatch` for every canonical program value type before
`select_supported_strategy`. `TargetProfileBuilder::governed` declares F32
dispatchability and no `tiler::u32@1` row, so an end-to-end compile of the governed
gather signature first returns `RequestError::DTypeNotDispatchable` for the exact
U32 type. If that target gate is bypassed, `recognized_program_arithmetic` would
next return `dtype-recognized` because it recognizes only F32 and BF16. Neither is
the later operation-set refusal.

The exact target-side perturbation is additive dispatch authority, not a semantic
change: a test-only target derived from the governed profile adds a
`Dispatchable` row for `gather_index_resolved_type()` while retaining the governed
F32 row. The unchanged gather program must then advance from
`DTypeNotDispatchable` to `dtype-recognized`. Changing the gather signature would
change the program rather than perturb the refusal's subject and is not evidence
for this boundary.

**Fact — the later operation-set boundary is independently reachable.** A private
unit check in `request.rs` can call `recognize_program_outputs` with
`ArithmeticType::F32`, deliberately bypassing the program-wide arithmetic gate.
`recognize_output` then offers the gather root to `plan_elementwise`; it is neither
a structural read nor an `ElementwiseFamily`, so the walk returns
`mismatch("operation-set")`. `recognize_epilogue_producer` is not this program's
path, and an arbitrary `IndexRealizationLaw` stub would not prove that the real
request-recognition route changed.

## What this must deliver

- An end-to-end compile test over the governed target that builds a semantic
  program containing one `tiler::gather-f32@1` occurrence and asserts the exact
  earliest refusal: `RequestError::DTypeNotDispatchable` for
  `tiler::u32@1`. Compile the **same program** against a test-only target carrying
  both the governed F32 dispatch row and an exact `Dispatchable` row for
  `gather_index_resolved_type()`, and assert that it advances to
  `dtype-recognized`. This check is about target dtype admission, not storage;
  adding `StorageScalar::U32` alone cannot make it pass.
- A private `request.rs` unit test over the same program that calls
  `recognize_program_outputs(&program, &laws_of(&program), ArithmeticType::F32)`
  and asserts `operation-set`. Passing F32 is intentional: it bypasses the
  program-wide U32 arithmetic refusal and pins the later real recognition route.
- An independent `fusion_legality.rs` unit test that asserts
  `FusionNumericalCapabilities::governed().classify(&gather_f32_op()) == None`.
- Perturb the three subjects separately and quote each failure: remove the exact
  U32 dispatch row from the widened test target so the unchanged end-to-end
  program falls back to `DTypeNotDispatchable`; teach the real request-recognition
  walk to recognize the gather so the private `operation-set` assertion changes;
  and add a gather
  `FusionOperationRole` so only the `classify(None)` assertion changes. Do not
  change the semantic gather signature, use an arbitrary realization-law stub as
  evidence for the recognition path, or let one perturbation redden more than its
  own check.

## Non-goals

Changing the refusal, the family, or the policy inventory. Admitting anything into the index layer — [ADR 0108](../docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md) remains proposed and was returned for a complete representation comparison.

`contracts/decisions` is declared only for the dated ADR 0107 evidence correction required below. Preserve the accepted record and append the test-backed correction; do not rewrite its accepted body.

## Closes when

All three checks exist, each has been shown to fail under its own subject
perturbation with the message quoted, and ADR 0107's dated correction is updated
to point at them. The tests preserve the current no-admission behavior.
