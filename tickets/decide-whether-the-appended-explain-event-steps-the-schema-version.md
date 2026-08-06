---
id: decide-whether-the-appended-explain-event-steps-the-schema-version
title: Decide whether the appended explain event steps the schema version
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [explain, identity, schema, decision]
claimed_from: todo
assignee: agent-explain-version
lease_expires_at: 1786038888
---

## The fact (doc-sweep audit 2026-08-06, coordinator-verified at source)

Event tag 13 (`SynchronizationRealization`, with renderer v7's `synchronization:` line) landed in `fece761f` under an unmoved `EXPLAIN_SCHEMA_VERSION = 9` and `EXPLAIN_RENDERER_VERSION = 7` (`crates/tiler-compiler/src/explain.rs:35-36`; the append is documented in the ledger comment at the version block and at the tag site near `:2908`). The append is byte-safe — no earlier record's tag or field layout moves — but a v9 trace's event *vocabulary* is no longer decided by its version alone: two v9 traces from different builds can differ in which tags they may contain.

## The question

Whether the explain schema's versioning contract requires a version step for an appended event tag, or whether appends-only tag additions are admissible under one version with per-tag injectivity reasoning (the discipline the schedule/kernel domains use). The version-block comment's own precedent cuts the other way: v7, v8, and v9 each stepped for additive changes, so the landed append is inconsistent with the file's own history — either the append should have stepped to v10, or the versioning rule should be restated so that a reader knows appends do not step it and must not infer vocabulary from version.

## The work

Read the explain schema's stated versioning rule and its consumers (anything that dispatches on `EXPLAIN_SCHEMA_VERSION` or decodes by tag), decide which world is correct against them, and execute it whole: either step the version with the ledger comment moved in the same commit (an identity-domain step, executed completely per AGENTS.md), or restate the versioning rule at the version block so the append discipline is explicit, with the tag-13 comment aligned. Half-measures — a stepped version with unmoved ledger text, or a restated rule that still implies version-decides-vocabulary — are worse than either whole answer.

## Closes when

The version block's stated rule, the tag-13 record, and every version consumer agree, and a reader of a v9 (or v10) trace knows exactly what its version does and does not promise.
