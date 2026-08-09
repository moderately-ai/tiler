---
id: correct-the-stale-bf16-compiler-doc-comments
title: Correct the two stale BF16 doc comments in tiler-compiler
status: closed
priority: p3
dependencies: []
related: [state-and-check-a-bf16-numerical-contract, move-the-navigation-docs-onto-the-two-contract-key-domains]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16]
closed_reason: obsolete
closed_note: The recognizer-widening landing already replaced both cited comments and tests.
---
## User-visible outcome

Two BF16 doc comments in `tiler-compiler` describe what the code does now: one
names a test that exists, and one names the boundary that actually refuses.

## Why this is a separate ticket

Found by the `contracts/navigation` sweep in
`move-the-navigation-docs-onto-the-two-contract-key-domains`, whose scopes cannot
reach `crates/`. Both are stale claims rather than wrong behaviour, and a doc
comment is load-bearing: the next worker reads it as fact.

## Scope keys

**Fact, checked at `3adc0689`.** `crates/tiler-compiler/src/pipeline/tests.rs:1682`
says the `dtype-f32` case is covered by
`a_dispatchable_bf16_profile_reaches_the_recognizer_dtype_wall`. No such test
exists — `rg -n 'a_dispatchable_bf16_profile_reaches_the_recognizer_dtype_wall' crates/`
returns that one doc-comment line and nothing else. The test it means is
`a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall` in
`crates/tiler-compiler/tests/bf16_numerical_contract.rs:384`. Repoint the name;
do not rename the test to match the comment, because the test's name states the
contract that reaches the wall and the comment's does not.

**Fact, same commit.** `crates/tiler-compiler/src/session.rs:1656`, in
`NumericalContractBuilder::strict_bf16`'s doc, says a `bf16` program "is refused
by the recognizer's `dtype-f32` rule after the contract has been assessed". That
holds only on a profile that declares `tiler::bf16@1` dispatchable. Target
resolution now runs above recognition (`crates/tiler-compiler/src/request.rs`,
the phase-split comment at ~2395), so the governed baseline — which declares
`tiler::f32@1` and is silent about `bf16` — refuses per target with
`RequestError::DTypeNotDispatchable` at disposition `Unknown` before recognition
runs, which `a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary`
asserts. Scope the sentence to the profile that reaches the recognizer rather
than deleting it; the `dtype-f32` wall is the point the paragraph is making and
it is still real.

## Required evidence

- The repointed test name resolves: one `rg` for it returns both the comment and
  the `#[test]`.
- Targeted `cargo nextest run -p tiler-compiler` and per-package Clippy green.

## Closes when

Neither comment asserts something the source refutes, and the corrected claims
each cite the site that makes them true.

## Repository audit and closure — 2026-08-09

**Both original Facts are false as current work.** The current pipeline test is `a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary`; its documentation says `The governed baseline refuses a pure-BF16 program by its own dtype row` and asserts `DTypeNotDispatchable` with `Unknown` for the BF16 subject. The positive profile test is now `a_flush_accepting_bf16_contract_reaches_a_selected_plan`; its documentation records that the old `dtype-f32` recognizer wall is gone and asserts a selected plan. `NumericalContractBuilder::strict_bf16` likewise says `Statable, and now planned`, cites both the single- and multi-occurrence positive tests, and separately explains the authoritative profile's remaining numerical-row refusal.

Neither stale test name nor the old “refused by the recognizer” sentence remains in the cited source. The correction landed before this ticket was revisited, so no compiler edit, test rename, or assertion change remains. This ticket closes as obsolete with a ticket-only record; `implementation/compiler` is removed because no compiler edit remains.
