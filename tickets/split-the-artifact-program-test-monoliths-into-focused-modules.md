---
id: split-the-artifact-program-test-monoliths-into-focused-modules
title: Split the artifact program test monoliths into focused modules
status: in-progress
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [implementation/artifact, implementation/frontend, contracts/navigation, research/documentation]
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

## Three scopes added for the path pins the split forced

`contracts/navigation`, `research/documentation`, and `implementation/frontend` were added because moving a file that other checks pin **by path** makes those checks fail. Each repair below is a path update under an unchanged claim, so the added scopes are scheduling metadata rather than an expansion of the outcome.

`implementation/frontend` — `crates/tiler/tests/workspace_unsafe_sites.rs` pins the exact private `macro_rules!` producer population of the whole workspace by `(path, name)`, and one of its seventeen entries was `codec/tests.rs`'s `exhaustive_enum_population`. The macro now lives beside its only two invocations in `crates/tiler-artifact/src/program/codec/tests/vocabularies.rs`, and the pin names that path. Nothing else in that inventory moves: the population is still seventeen, the four admitted unsafe sites are unchanged, and the invocations are admitted again because the definition they resolve to is pinned. **This does not show up in `cargo nextest run -p tiler-artifact`** — only a workspace run reaches it, which is how it was found.

The remaining two are citation repairs. Deleting `crates/tiler-artifact/src/program/codec/tests.rs` rots every line-number citation into it, and `make citations` gates on those resolving. Four citations were affected and all four are repaired here; none is a claim about the artifact crate's behaviour.

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

### What holds "reorganization, not revision"

A brace-depth parse of the pre-split files and of the split trees compares the two item by item, matching on item name and comparing the text from each item's own doc comment and attributes onward, with `super::` depth and visibility qualifiers normalized away. It reports **442 items before and 442 after, none missing, none added, and none changed**. 332 are byte-identical; the other 110 differ only by the `super::` depth the extra module level requires, by `pub(crate)` on a fixture item that now crosses a file boundary, and by the rustfmt reflow those two cause when they push a signature past 100 columns.

The comparison was watched failing under two subject perturbations before it was trusted. Changing one byte of one string literal — `b"tiler.artifact-program.v10\0"` to `v11` in `a_recording_under_a_foreign_domain_is_refused` — moved it to `byte-identical: 160 … CHANGED: 1` and named that test and its file. Deleting `encoding_is_deterministic` outright moved it to `206 items before, 205 after` with `missing: ['encoding_is_deterministic']`. Both perturbations were reverted and the comparison returned to zero.

The split is also mechanically reproducible: re-running the generator over the two pre-split files pinned from the base commit reproduces the committed tree byte for byte, so nothing in it is a hand edit that a reader would have to re-derive.

The 47 items the pre-split file exported are all re-exported from `tests/mod.rs` with their **original visibility** — 15 `pub(crate)`, 32 `pub(super)`, none lost, none widened, none narrowed — so `crate::proof::tests`, `crate::program::retained`, and the codec suite import the same names at the same reach as before.

## Defects observed while moving, deliberately not fixed

1. **The fixture-visibility note is stale.** "The seven items this suite shares with `crate::proof::tests` are `pub(crate)`" — there are **15** `pub(crate)` items and `crate::proof::tests` imports **8** of them. The note also omits a second crate-internal consumer: `crate::program::retained`'s test module imports `SCALE_BITS`, `build_artifact`, `build_graph`, `fused_program`, `lowering_provider`, and `semantic_program`. The note is preserved verbatim and relocated to `tests/mod.rs`, beside the re-export block it describes, because it now sat above five constants that are all `pub(crate)` inside `support`.

2. **Thirteen exported names have no consumer outside this suite**, so their `pub(crate)` / `pub(super)` markers claim a reach nothing uses: `BIAS_BITS`, `CANONICAL_NAN`, `strict`, `input_shape`, `output_shape`, `fused_kernel`, `entry`, `rules`, `partial_window_program`, `CLAIM_OBJECT`, `claim_payload_content`, `realization_record`, `live_extent_program`. The last two are `pub(crate)`, which is two levels wider than any use. The surface is preserved exactly here rather than narrowed, because narrowing it is a separate decision; the compiler cannot see the over-reach, since the suite's own modules now import through the same re-exports.

3. **A `docs/roadmap.md` line citation was already wrong at the base.** It pinned `a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity` to `codec/tests.rs`:2584, but that test began at line 4131; line 2584 was inside `a_partial_binding_window_survives_encode_and_decode`. It resolved only because the file was long enough. Repaired to an anchor here — the standing case for AGENTS.md's "cite by searchable anchor, not by line number".

4. **Two malformed section banners in the pre-split `program/tests.rs`**, preserved verbatim: line 1584 is a bare separator rule with no title, opening the strict-affine block (now the top of `support/encoded.rs`), and lines 1985–1986 spell `// Artifact fixtures` followed by a closing rule with no opening one (now the top of `support/artifacts.rs`). Cosmetic.

## Gates — all green at 147a105c2728a5d2ec77d301cfddf6d94c8ac9b2

`make full` passes end to end, which subsumes every check this ticket names. Run individually as well:

- `cargo nextest run -p tiler-artifact`: 340 run, 340 passed, 1 skipped — identical to the base.
- `cargo nextest run --workspace`: 3,791 run, 3,791 passed, 8 skipped.
- `cargo test -p tiler-artifact --doc` and `cargo test --workspace --doc`: pass.
- `cargo clippy -p tiler-artifact --all-targets -- -D warnings`: clean.
- `cargo fmt --check`, `git diff --check`, `tkt lint`, `make citations`: clean.
- `tkt guard tkt/split-the-artifact-program-test-monoliths-into-focused-modules --format json`: `under_declared: []`, `conflict: false`, severity `warn` (declared-area overlaps only).

Commits: `0d73325c` the split, `17ae20b7` the workspace macro-producer pin, `147a105c` the two prose cross-references. `git diff --stat 9bcc2d86..147a105c` changes no production `.rs` file: the two test files become directories, `crates/tiler/tests/workspace_unsafe_sites.rs` moves one pinned path, and the rest is three documents and this ticket.
