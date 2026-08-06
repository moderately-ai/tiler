---
id: state-the-general-property-in-each-reindex-conformance-test
title: State the general property in each reindex conformance test
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/reference]
shared_scopes: []
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
