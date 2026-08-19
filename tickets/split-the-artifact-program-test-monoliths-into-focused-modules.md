---
id: split-the-artifact-program-test-monoliths-into-focused-modules
title: Split the artifact program test monoliths into focused modules
status: in-progress
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [implementation/artifact, contracts/navigation, research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, maintainability, artifact, tests]
claimed_from: todo
assignee: worker-split-artifact-tests
lease_expires_at: 1787146418
---
## User-visible outcome

`crates/tiler-artifact/src/program/tests.rs` (7,936 lines, ~278 items at filing) and `crates/tiler-artifact/src/program/codec/tests.rs` (5,795 lines) become directories of focused test modules grouped by the property family they guard (identity pins, builder refusals, codec round-trips, decode validation, retained-environment subjects, …), so a reviewer auditing one property family reads one small file.

## Why this exists

Filed 2026-08-19 from Tom's module-size directive. These monoliths hold the artifact's correctness-bearing evidence — including identity pins the accepted symbolic-packaging migration (item 24) will soon need to re-derive — and their size makes the read-the-tests-in-full obligation needlessly expensive exactly where it matters most.

## Required work

- Read both files in full first; group tests by the property family each guards, keeping shared fixtures/helpers in one clearly named support module rather than duplicating them. Preserve every test byte-for-byte in assertion content — this is file reorganization, not test revision.
- Convert each `tests.rs` into a `tests/` directory module; the declaring `mod tests;` lines and `#[cfg(test)]` gating keep working unchanged. Production code is untouched.
- Test names must not change (they are cited from tickets and docs by name); if a helper must be renamed to resolve a collision, record it. Zero assertion, pin, or golden changes.
- Fence: only the two named test files (becoming directories) and any support module they extract may change; no production `.rs` file in the crate moves.

## Evidence and checks

`cargo nextest run -p tiler-artifact` with the same test count before and after (state both counts — a shrinking population is a defect), `cargo test -p tiler-artifact --doc`, clippy warnings-denied, `cargo fmt --check`, `tkt lint`, `git diff --check`, `tkt guard`. Report the module inventory and the before/after test-count equality.

## Non-goals

New tests, assertion changes, production-code edits, and other crates' test monoliths.

## Closes when

Both directories land with identical test populations, all gates green, and the inventory recorded.

## Two scopes added for the citation repairs the split forced

`contracts/navigation` and `research/documentation` were added because deleting `crates/tiler-artifact/src/program/codec/tests.rs` rots every line-number citation into it, and `make citations` gates on those resolving. Four citations were affected and all four are repaired here; none of them is a claim about the artifact crate's behaviour, so the added scopes are scheduling metadata rather than an expansion of the outcome.

- `docs/roadmap.md` pinned `a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity` to `codec/tests.rs`:2584. **That line number was already wrong at the base**: the test began at line 4131 and line 2584 was inside `a_partial_binding_window_survives_encode_and_decode`. It only resolved because the file was long enough. Repaired to the anchor form against the test's new home, `crates/tiler-artifact/src/program/codec/tests/carriers.rs`.
- Three occurrences of `codec/tests.rs`:541 under `docs/research/documentation/ticket-audit-2026-08-10/` are *quotations of a retired citation* inside a repair record and its report — each sentence says the citation was replaced as stale. They are re-spelled in the shape `check-citations.sh` reserves for exactly that case: the path in a code span, the retired line number as a bare `:541` suffix outside it, so a quoted retired citation is no longer demanded to resolve. No claim changed.

## Fact audit at base 9bcc2d863fe84884102f92fa2b42fe2816d5f73a — 2026-08-19

- **Verified.** `crates/tiler-artifact/src/program/tests.rs` is 7,936 lines and `crates/tiler-artifact/src/program/codec/tests.rs` is 5,795 lines (`wc -l`).
- **Verified.** Both declaring lines are `#[cfg(test)]`-gated: `pub(crate) mod tests;` in `crates/tiler-artifact/src/program/mod.rs` and `mod tests;` in `crates/tiler-artifact/src/program/codec/mod.rs`. Neither moved.
- **Imprecise.** "~278 items at filing" for `program/tests.rs`. A brace-depth parse finds **236** top-level items, 97 of them `#[test]`. The figure is approximate in the ticket and nothing depended on it; recorded so a later reader does not treat 278 as a census.

## Landed inventory — 2026-08-19

Test population is unchanged: `cargo nextest run -p tiler-artifact` reports **340 tests run, 1 skipped** both before and after. The declared population is 341 in both trees (the skipped one is the `#[ignore]`d `hot_path_decode_profile_loop`), cross-checked against a source scan of every `#[test]` in the crate at the base commit.

`crates/tiler-artifact/src/program/tests/` — 97 tests:

| module | tests | property family |
| --- | --- | --- |
| `support/` (8 files) | 0 | fixtures shared across modules and re-exported to the codec, proof, and retained suites |
| `bf16_pointwise` | 2 | the pointwise producer path at two arithmetic widths |
| `stage_keys` | 2 | stage-key generation and the kernel-program subject it encodes |
| `construction` | 6 | what a verified artifact is and what a consumer reads off it |
| `recorded_identity` | 5 | recorded artifact-identity assertions and their refusals |
| `identity_determinism` | 4 | identity is deterministic and ignores declaration order |
| `provenance` | 7 | reached versus unused provenance (ADR 0072) |
| `foreign_handles` | 3 | cross-program subjects and another builder's handles are refused |
| `insertion_rules` | 13 | one negative case per insertion-time builder rule |
| `whole_artifact_rules` | 11 | one case per whole-artifact rule plus the delivery positives |
| `expressions` | 8 | ABI evaluation, phases, arena growth, program-ABI adoption |
| `governed_keys` | 5 | governed key alphabets and opaque-identity bounds |
| `extent_operands` | 8 | live input-extent operand association with the interface |
| `baked_extents` | 4 | baked `[2, N]` neighbours and host-side extent preconditions |
| `route_requirements` | 5 | route-requirement vocabulary, satisfaction, and subjects |
| `identity_encoders` | 4 | this crate's tag tables and finite-domain encoder injectivity |
| `plan_determinism` | 10 | plan-determinism claims (ADR 0013) and every refusal |

`crates/tiler-artifact/src/program/codec/tests/` — 154 tests:

| module | tests | property family |
| --- | --- | --- |
| `support` | 0 | envelope projection, encoding, byte forgery, and their artifacts |
| `vocabularies` | 3 | governed tag tables and digest domains |
| `round_trip` | 10 | canonical form, determinism, declaration-order independence |
| `subgroup` | 5 | the conditional subgroup-realization carrier |
| `selected_providers` | 6 | structured selected-capability rows |
| `byte_corruption` | 16 | incompetent forgeries: bytes, truncation, framing, schema |
| `forged_models` | 18 | competent forgeries refused by a named cause |
| `canonical_order` | 9 | non-canonical spellings refused rather than normalized |
| `expression_arena` | 11 | the ABI arena driven directly through the parser |
| `carried_payloads` | 11 | carried subjects and objects, and the dispatch record |
| `binding_targets` | 6 | a decoded binding's target, component, and access type |
| `carriers` | 9 | carrier/access-type pairs through encoding and identity |
| `payload_sections` | 8 | payload objects and subjects as content-addressed sections |
| `provenance` | 6 | payload provenance fields and the platform block |
| `section_descriptors` | 7 | section purpose, disposition, schema, compatibility contract |
| `route_requirements` | 7 | route-requirement rows through the envelope |
| `extent_operands` | 7 | live input-extent rows and their transports |
| `plan_determinism` | 9 | scope cells and target-environment records (ADR 0013) |
| `hot_path` | 6 | reproducible cost measurements; no timing assertions |
