---
id: pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach
title: Pin the tiler-compiler identity domain spellings the ir census does not reach
status: done
priority: p1
dependencies: []
related: [pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [identity, tests, versioning]
---

`tiler-ir` now carries a source census pinning its identity domain spellings. The scan is `CARGO_MANIFEST_DIR`-rooted, so it reaches that crate only. **`tiler-compiler` is the largest uncovered population.**

## Facts

**Correct the framing before you start — an earlier version of this finding was overstated by the coordinator and repaired by measurement.** It is **not** true that no test asserts an identity domain. Several do: `crates/tiler-ir/src/kernel/tests.rs` binds `b"tiler.kernel.v7\0"` and asserts two identities open with it; `crates/tiler-ir/src/semantic/catalog/tests.rs` asserts `starts_with(b"tiler.value-type-descriptor.v1\0")`; and `STRICT_F32_REGION_IDENTITY_HEX` opens with the hex of `tiler.schedule.v5\0`. Both of those first two were coordinator-verified.

**The true finding is that coverage is incidental.** It exists wherever someone pinned a digest over a subject that happens to fold a domain, and it reports only that *a digest moved*. Measured on `tiler-ir` before the census landed: reverting `GRAPH_DOMAIN` v3→v2 failed **2** tests, `INDEX_REGION_DOMAIN` v11→v10 failed **4**, and `OBLIGATION_DOMAIN` v2→v1 failed **0** of 3,184. Assume `tiler-compiler` has the same uneven shape; **measure it, do not assume either extreme**.

**Fact — imprecise as reported, corrected at `7134a732`.** The population of **25** is not every `tiler.`-spelled literal: it is the distinct byte-string literals matched by `b"tiler\.[A-Za-z0-9._:-]+(?:\\0)?"`. All 25 are NUL-terminated identity-domain candidates. There are 26 occurrences under `src/` because the boundary-property unit test restates that live domain, and 30 across the complete walked `src/` plus `tests/` population because four legality domains are each restated once in an integration test. Ordinary string literals such as `tiler.cost.structural.v1` are a separate, larger classified non-domain population. Nineteen of the 25 domains are named constants and six are inline literals with no constant behind them, so a `variant_count` enumeration cannot reach the actual population. The four reported live spellings are verified in `request.rs`'s `canonical_explain_subject_bytes`, `target.rs`'s `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN`, `target/feasibility.rs`'s `PROFILE_DESCRIPTOR_DOMAIN`, and `boundary.rs`'s `PROPERTY_SET_IDENTITY_TAG`.

## Fact audit at exact base `7134a732`

- **Verified — the IR census is crate-rooted and does not reach this crate.** `crates/tiler-ir/src/domains.rs` was read in full: `scan_crate_sources` roots itself at `env!("CARGO_MANIFEST_DIR")`, walks only that manifest's `src/` and `tests/`, and excludes its own pin table. `crates/tiler-ir/src/lib.rs` admits it only as a private `#[cfg(test)] mod domains`. `crates/tiler-compiler/src/lib.rs` had no corresponding module at this base.
- **Verified — `tiler-compiler` is the largest population with no exact-byte census.** Re-running `rg -o --no-filename 'b"tiler\.[A-Za-z0-9._:-]+(?:\\0)?"' crates/<crate>/src | sort -u | wc -l` gives compiler 25, cache 6, and no other crate lacking a domain module above 2. `tiler-artifact` gives 27 byte-string candidates but was read separately: its `domains` module establishes source completeness and prefix separation for 18 governed domains while deliberately never pinning their bytes, the different gap this ticket reports but does not widen into.
- **Verified — direct and incidental IR coverage exists.** `kernel::tests::a_bf16_kernel_and_its_f32_sibling_do_not_share_identity` binds `b"tiler.kernel.v7\0"` and checks both identities open with it; `semantic::catalog::tests::every_descriptor_has_a_distinct_reproducible_fingerprint` checks `b"tiler.value-type-descriptor.v1\0"`; and `schedule::builder::the_strict_f32_region_has_its_recorded_canonical_identity` checks a hex golden opening with `tiler.schedule.v5\0`. The sibling ticket `pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate.md` and the complete `tiler-ir/src/domains.rs` retain the measured isolated reverts: graph 2 failures, index region 4, obligation 0 of 3,184. This is historical measurement, not inferred current compiler coverage; the compiler baseline is measured below before its guard is added.
- **Verified after correcting the population wording above — the named compiler domains resolve at live construction sites.** `request.rs` writes request-subject `v6`; `target.rs` writes declaration `v11`; `target/feasibility.rs` writes descriptor `v10`; and `boundary.rs` writes property-set `v3`. Their construction and direct test sites were read before implementation. The source command returns 25 distinct byte spellings, not 25 total `tiler.`-spelled literals.

## What closes this

A census for this crate, modelled on `crates/tiler-ir/src/domains.rs` — read it first, and read its module documentation for why it is shaped as it is.

**Do not reach for `variant_count` without checking whether it fits.** It is right for `tiler-artifact`, where every domain is a named constant a variant can mirror. It was **wrong** for `tiler-ir`, where 15 of 60 pinned spellings are inline literals no constant names — an enum could have named 45 of 60 while reporting a complete population, which is the exact failure the enumeration exists to prevent. Determine which case `tiler-compiler` is, by counting, and say so.

**Measure the baseline first.** Revert each versioned domain in turn and record what fails, as the sibling did. That tells you which domains already have incidental coverage and which have none, and it is the evidence that this work is needed rather than the assumption.

**Perturb each guard separately and quote the failure text.** The sibling ran nine perturbations, four of which reddened exactly one assertion — including a self-exclusion guard proving the scan cannot satisfy itself from its own pin table, and a shadowing guard catching an admitted prefix that would swallow a pinned domain. Both are failure modes a census invites; carry them across or argue why they do not apply.

**A byte pin costs two edits on a deliberate step**, not one — the constant plus its table row. That is the floor, because a pin compares a value against a second copy. Make both assertions name the file, the spelling, and the table so the second edit is located rather than hunted. A per-domain version *floor* would cost zero edits and was rejected: it cannot see a revert to any version at or above the floor.

**Report the crates still uncovered with their counts.** `tiler-artifact` has a `domains` module that checks completeness and no-prefix but **never a value**, so an artifact domain can still be reverted silently there — that is a separate ticket, not this one. Do not widen.

## Worker record at `7134a732`

### Baseline before the guard

The unmodified package baseline was 803 passed and 1 skipped. Each row below changed only the live source spelling to its immediately preceding version (`v1` to `v0` where `v1` is the first recorded version), ran `cargo nextest run -p tiler-compiler --no-fail-fast`, and restored the source before the next row. The 13 green rows are the measured exact-byte gap this census closes.

| isolated source revert | failures before this census |
| --- | --- |
| `tiler.compiler.boundary-property-set.v3` → `v2` | 1 — `boundary::tests::identity_is_independent_of_assembly_order_and_separates_distinct_contracts` |
| `tiler.compiler.fusion-legality-content.v1` → `v0` | **0 — 803 passed** |
| `tiler.compiler.fusion-legality-occurrence.v1` → `v0` | **0 — 803 passed** |
| `tiler.compiler.index-refinement-content.staged.v1` → `v0` | 1 — `two_region_occurrence_lowering::a_chain_and_a_region_encode_under_disjoint_identity_domains` |
| `tiler.compiler.index-refinement-content.v2` → `v1` | 1 — the same integration test |
| `tiler.compiler.index-refinement-occurrence.staged.v1` → `v0` | 1 — the same integration test |
| `tiler.compiler.index-refinement-occurrence.v2` → `v1` | 1 — the same integration test |
| `tiler.compiler.lowering-capability-registry.v2` → `v1` | 1 — `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` |
| `tiler.compiler.physical-implementation-proposal.v2` → `v1` | 1 — `frontier::tests::the_recognized_region_subjects_keep_their_exact_proposals` |
| `tiler.compiler.region-content.v1` → `v0` | **0 — 803 passed** |
| `tiler.compiler.region-cover.v1` → `v0` | **0 — 803 passed** |
| `tiler.compiler.region-occurrence.v1` → `v0` | **0 — 803 passed** |
| `tiler.compiler.request-subject.v6` → `v5` | 1 — `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` |
| `tiler.compiler.selected-physical-plan.v2` → `v1` | **0 — 803 passed** |
| `tiler.compiler.selected-physical-portfolio.v1` → `v0` | **0 — 803 passed** |
| `tiler.explain.compilation.v1` → `v0` | **0 — 803 passed** |
| `tiler.explain.trace.v1` → `v0` | **0 — 803 passed** |
| `tiler.program-alternative.v2` → `v1` | **0 — 803 passed** |
| `tiler.target-profile.cost-row.v1` → `v0` | **0 — 803 passed** |
| `tiler.target-profile.declaration.v11` → `v10` | 2 — the deterministic explain trace and `physical::tests::the_governed_descriptor_bytes_do_not_move` |
| `tiler.target-profile.descriptor.v10` → `v9` | **0 — 803 passed** |
| `tiler.target-profile.dtype-dispatchability.v2` → `v1` | 2 — the same explain and physical tests |
| `tiler.target-profile.evaluation-order-preservation.v1` → `v0` | **0 — 803 passed** |
| `tiler.target-profile.fact-sources.v4` → `v3` | 2 — the same explain and physical tests |
| `tiler.target-profile.synchronization-realization.v1` → `v0` | 2 — the same explain and physical tests |

### What landed and why it is source-sized

`crates/tiler-compiler/src/domains.rs` is a private test-only census, declared from `lib.rs` without a public item or feature. Its file-floor check sees 63 Rust files across `src/` and `tests/`; it then proves it removed its own pin table and scans the remaining 62. The file-state-aware lexical walk reads 138 `tiler.`-spelled literal occurrences in 89 distinct spellings while distinguishing line comments, nested block comments, cooked and raw strings (including multiline forms), C strings, and character literals. Cooked strings are decoded before namespace recognition, so a hexadecimal or Unicode escape and a backslash-newline continuation cannot conceal an evaluated `tiler.` prefix; malformed or unsupported escapes fail closed. Cooked `c"…"` and raw `cr"…"` bodies receive Rust's implicit NUL before classification, so an admitted non-domain prefix cannot swallow a C-string domain. Twenty-five distinct exact byte domains are pinned with exact per-tree counts: 26 strict occurrences in `src/`, and 30 across `src/` plus `tests/` after the four integration-test restatements. Nineteen domains are named constants and six are inline literals (`request`, `program-alternative`, two `region`, and two `explain`). The six inline domains are why `variant_count` does not fit: a type can mirror the 19 constants while silently omitting almost a quarter of the exact domain population. The scanner asserts a 50-file floor and a 100-literal floor, requires every NUL-terminated literal to be an exact pin, and requires every remaining spelling to be either an exact pin, an exact classified non-domain, or a classified non-domain namespace.

The source-to-pin assertion catches a changed spelling at its source location. The reverse assertion compares every pin's exact `src/` and `tests/` occurrence counts, so a retained unit or integration-test spelling cannot mask a missing live declaration and an already-pinned replacement cannot hide the change. The tables are sorted and duplicate-free, exact non-domain literals cannot equal a pin, and admitted prefixes cannot prefix a pin. `src/domains.rs` must be found and removed exactly once before scanning, so the pin table cannot satisfy its own occurrence check.

### Independent fail-capable evidence

Every perturbation below changed its subject, was run alone, and was restored. Unless stated otherwise the command was `cargo nextest run -p tiler-compiler -E 'test(/domains::/)' --no-fail-fast`.

Review first established both reported false greens on commit `a5810b7c`: placing `let _ = "https://example.invalid"; let _ = b"tiler.unclassified.domain.v1\0";` on one live-source line made all four census tests pass, and replacing only live `PROPERTY_SET_IDENTITY_TAG` with the already-pinned region-cover tag while its unit-test spelling remained also made all four pass. Those are scanner reach and population-accounting failures, not assertion perturbations. Both now fail below; the permanent fifth test exercises the lexical states directly.

Re-review then established a third false green on commit `9fb11c25`: adding live `b"\x74iler.escaped-prefix-domain.v1\0"`, whose evaluated bytes begin `tiler.`, made all five tests pass because cooked strings were decoded only after their source spelling already matched the prefix. The final scanner decodes before recognition. Re-running the direct source census after that repair remains 25 distinct strict spellings, 26 `src/` occurrences, and four more `tests/` occurrences; this module and its permanent escaped-prefix fixtures remain self-excluded.

The same re-review exposed C-string termination as another forward escape: live `c"tiler.target.cooked-c-domain.v1"` and `cr#"tiler.target.raw-c-domain.v1"#` each independently made all five tests pass because their written bodies are non-NUL and match the admitted `tiler.target.` prefix, although Rust evaluates each with an implicit terminator. The final scanner records their evaluated NUL and both subjects now fail below. No C-string domain exists in the walked source population at this revision, so the named 25/26/30 counts remain unchanged.

| subject perturbation | isolated verdict and quoted failure text |
| --- | --- |
| Add `"tiler.unclassified.guard-probe"` to `boundary.rs` | classification only: ```boundary.rs:2700: the literal `tiler.unclassified.guard-probe` is neither pinned in `src/domains.rs`'s `PINNED_IDENTITY_DOMAINS` nor classified by its `ADMITTED_NON_DOMAIN_LITERALS` or `ADMITTED_NON_DOMAIN_PREFIXES`.``` |
| Add NUL-terminated `b"tiler.test.domain-probe.v1\0"` inside an otherwise admitted fixture namespace | exact-domain candidacy only: ```boundary.rs:2700: the literal `tiler.test.domain-probe.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned even inside an admitted namespace.``` |
| Put a live strict literal after a same-line URL containing `//` | scanner reach: ```boundary.rs:2701: the literal `tiler.unclassified.domain.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned.``` |
| Put a live strict literal after same-line `/* // */` | scanner reach through block-comment state: ```boundary.rs:2700: the literal `tiler.unclassified.block-comment-domain.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned.``` |
| Add live `b"\x74iler.escaped-prefix-domain.v1\0"` | evaluated cooked bytes: ```boundary.rs:2700: the literal `tiler.escaped-prefix-domain.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned.``` |
| Split a live prefix as `b"t\` newline `iler.continued-prefix-domain.v1\0"` | evaluated continuation: ```boundary.rs:2700: the literal `tiler.continued-prefix-domain.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned.``` |
| Add live cooked `c"tiler.target.cooked-c-domain.v1"` | implicit C terminator: ```boundary.rs:2700: the literal `tiler.target.cooked-c-domain.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned even inside an admitted namespace.``` |
| Add live raw `cr#"tiler.target.raw-c-domain.v1"#` | implicit raw-C terminator: ```boundary.rs:2700: the literal `tiler.target.raw-c-domain.v1\0` is neither pinned … A NUL-terminated literal is always an exact-domain candidate and must be pinned even inside an admitted namespace.``` |
| Replace only live boundary-property v3 with already-pinned region-cover v1 while retaining its unit-test v3 | exact `src/` counts: ```boundary-property-set.v3\0 in src/: expected 2 occurrence(s), found 1 at […/boundary.rs:2731]``` and ```region-cover.v1\0 in src/: expected 1 occurrence(s), found 2 at […/boundary.rs:105, …/cover.rs:79]```. |
| Replace only live legality content v2 with already-pinned region-cover v1 while retaining integration-test content v2 | per-tree counts: ```index-refinement-content.v2\0 in src/: expected 1 occurrence(s), found 0 at []``` and ```region-cover.v1\0 in src/: expected 1 occurrence(s), found 2 at […/cover.rs:79, …/legality.rs:78]```. The retained `tests/` occurrence cannot supply the missing live row. |
| Move the inline explain domain outside `tiler.` while retaining its pin | reverse occurrence check: ```tiler.explain.compilation.v1\0 in src/: expected 1 occurrence(s), found 0 at [] … A deliberate step costs the source edit plus its row edit in that table.``` |
| Revert inline explain `v1` → `v0`, source only | both directions: ```explain.rs:2222: the literal `tiler.explain.compilation.v0\0` is neither pinned …``` and ```tiler.explain.compilation.v1\0 in src/: expected 1 occurrence(s), found 0 at [].``` |
| Duplicate the `tiler.explain.compilation.v1\0` pin row | table shape only: ```PINNED_IDENTITY_DOMAINS is out of order or repeats itself at `tiler.explain.compilation.v1\0` and `tiler.explain.compilation.v1\0`.``` |
| Widen admitted `tiler.target.` to `tiler.target` | prefix shadow only: ```the admitted non-domain prefix `tiler.target` covers the pinned identity domain `tiler.target-profile.cost-row.v1\0`, so that domain's exact spelling is no longer compared against its row in `PINNED_IDENTITY_DOMAINS`.``` |
| Add the cost-row domain to the sorted exact non-domain table | exact-classification collision only: ```the admitted non-domain literal `tiler.target-profile.cost-row.v1\0` equals the pinned identity domain `tiler.target-profile.cost-row.v1\0`, so the same spelling is classified both ways.``` |
| Rename `domains.rs` to `domain_pins.rs` and point the private module at it | self-exclusion only: ```the walk did not find this module at `…/src/domains.rs`, so it removed nothing. The pin table in that file restates every spelling it pins; reading it would let `PINNED_IDENTITY_DOMAINS` satisfy its own occurrence assertion.``` |
| Temporarily give all 14 integration-test sources a non-`.rs` suffix | file floor only: ```the walk found 49 `.rs` file(s) … fewer than the floor of 50. A walk that stopped finding files reports an empty population as intact.``` |
| Move 57 routine test/cost/prototype fixture literals outside `tiler.` without changing the file population | literal floor only: ```the scan read 81 `tiler.`-spelled literal(s) across 62 source file(s), fewer than the floor of 100. The scanner has stopped recognising literals it once read.``` |

The legitimate-step case is reachable too: moving both the inline explain source and its pin row from `v1` to `v0` made all five census tests pass. Moving only the source produced the two locating failures above. The cost is therefore exactly the source edit plus one pin-row edit.

### Still outside this crate

The compiler census evaluates individual Rust literal tokens; it does not execute fragment constructors. `rg -n -g '!domains.rs' 'concat!|concat_bytes!|include_bytes!' crates/tiler-compiler/src crates/tiler-compiler/tests` excludes the census's self-documentation and returns exactly `crates/tiler-compiler/src/explain.rs:3859:            concat!(`. The surrounding `deterministic_trace_is_sealed_and_rendered_separately` expected-output invocation was read and constructs no identity domain; there are no `concat_bytes!` or `include_bytes!` hits. This is an explicit unsupported boundary rather than a completeness claim: a future identity domain must be written as one literal, or the same change must widen the scanner and add fail-capable evidence for `concat!`, `concat_bytes!`, or `include_bytes!` as applicable. Treat a future search hit that constructs an identity as a failed census boundary; a green literal census alone must not admit it.

This work does not widen beyond `tiler-compiler/{src,tests}`. Re-running the sibling's distinct-literal census over each other crate's `src/` gives: `tiler-artifact` 28 total spellings / 27 strict byte candidates, `tiler-cache` 7 / 6, and `tiler-build`, `tiler-conformance`, `tiler-digest`, `tiler-metal-aot`, and `tiler-reference` 2 / 2 each; `tiler` and `tiler-macros` 1 / 1 each; `tiler-metal` and `tiler-runtime` 0 / 0. These are source populations, not claims that every row is an identity domain. `tiler-artifact` separately enumerates 18 governed domains for completeness and no-prefix but pins no value, so its exact-byte gap remains the named separate ticket rather than being absorbed here.
