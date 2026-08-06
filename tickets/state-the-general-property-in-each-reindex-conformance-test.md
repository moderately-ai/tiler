---
id: state-the-general-property-in-each-reindex-conformance-test
title: State the general property in each reindex conformance test
status: done
priority: p3
dependencies: []
related: []
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Every test in the undocumented head-layout and rotary conformance files carries a doc-comment naming the general IR property it establishes with the workload shape as the worked instance, so the file's subject survives being read one test at a time.

## Why this exists (leakage audit 2026-08-06: 26 of 51 tests across the five workload-named fixtures undocumented, skewed to exactly the general fail-closed checks; grouped_query_head_layout.rs 0/10, rotary_position_embedding.rs 0/6; decoder_layer.rs at 12/12 is the model)

The module headers state the general subject correctly; this propagates the framing one level down, closing the vocabulary-drift route the worked-examples discipline names.

## Non-goals

Renaming tests or fixtures (roadmap.md:374 permits workload names in fixtures freely); adding assertions; the three mostly-documented files beyond their gaps.

## Closes when

Doc-only change, every test documented, nextest green, rustdoc -D warnings clean on tiler-reference.

## Outcome (2026-08-06)

Delivered doc-only at commit `7399212c` on `tkt/state-the-general-property-in-each-reindex-conformance-test`, base `01ad1c99`.

**The population, recounted rather than carried from the audit.** The audit's numbers are wrong in both directions and are recorded here as superseded. Counting `#[test]` attributes in the five workload-named conformance fixtures under `crates/tiler-reference/tests/` gives **54 tests, not 51**, and **28 undocumented, not 26** — the audit's "26" is the *documented* count. `decoder_layer.rs` is not at 12/12: it carries 18 tests of which 15 were documented, so the model file itself had three gaps. Per file, documented before -> after:

| file | tests | documented before | after |
| --- | --- | --- | --- |
| `attention_contraction_structures.rs` | 5 | 5 | 5 (untouched) |
| `causal_self_attention_block.rs` | 15 | 6 | 15 |
| `decoder_layer.rs` | 18 | 15 | 18 |
| `grouped_query_head_layout.rs` | 10 | 0 | 10 |
| `rotary_position_embedding.rs` | 6 | 0 | 6 |
| **total** | **54** | **26** | **54** |

Verified by `awk '/^#\[test\]$/{if (prev !~ /^\/\/\//) print NR} {prev=$0}'` over each file, which reports no line in any of the five.

`serial_sum_slice.rs` (1 test, undocumented) was evaluated and deliberately excluded: it is named after IR operations rather than a workload and its header calls it a downstream-style proof of the public path, so it is not one of the five workload-named fixtures this ticket's population names. It remains the one undocumented test under `tests/` among the workload-adjacent files.

**Evidence that no assertion moved.** `git diff --stat` is 293 insertions, 0 deletions, across four files; `git diff -U0` filtered for any changed line not beginning `+///` returns nothing. `cargo nextest run -p tiler-reference` reports **285 tests run: 285 passed, 2 skipped** both at the stashed base and after the change.

Commands run in the worktree:

```sh
cargo fmt --check
cargo nextest run -p tiler-reference          # 285 passed, 2 skipped (before and after)
cargo clippy -p tiler-reference --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-reference
tkt lint                                       # ok: no problems found
git diff --check
tkt guard tkt/state-the-general-property-in-each-reindex-conformance-test --format json
```

Clippy is run with `--all-targets` because the workspace enables `clippy::pedantic`, whose `doc_markdown` lint is the one a new doc comment can actually trip; the targeted check list would not have compiled the test targets at all.

`project/tickets` was added as a shared scope so this ticket file could be edited; the declared exclusive scope stays `implementation/reference`.
