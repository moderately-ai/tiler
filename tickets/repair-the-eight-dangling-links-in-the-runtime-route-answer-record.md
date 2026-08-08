---
id: repair-the-eight-dangling-links-in-the-runtime-route-answer-record
title: Repair the eight dangling links in the runtime route answer record
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786167946
---
## What is broken

`docs/research/runtime/backend-scoped-route-requirement-answers.md` carries **eight** markdown links that resolve to nothing. Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`; this file is the single largest concentration of the fourteen defects that run found.

Two distinct mistakes, both mechanical:

**Five ADR links written as bare filenames**, as if the record lived in `docs/decisions/`. Each names a real accepted ADR, so the target exists and only the prefix is missing:

- `](0090-compose-backends-per-responsibility-rather-than-per-backend.md)`
- `](0086-require-attributable-or-attested-native-translation.md)`
- `](0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md)`
- `](0074-use-explicit-public-api-conventions.md)`
- `](0075-scope-public-boundary-approval-by-change-category.md)`

Each resolves against `docs/research/runtime/` and finds nothing; each exists at `docs/decisions/<same-name>`, verified 2026-08-08.

**Three links with the wrong number of `..` segments:**

- `](../research/runtime/backend-scoped-route-requirement-answers.md)` — a self-link written from `docs/`, resolving to `docs/research/research/runtime/...`
- `](../architecture.md)` — resolves to `docs/research/architecture.md`; the file is `docs/architecture.md`
- `](../artifact-abi.md)` — resolves to `docs/research/artifact-abi.md`; the file is `docs/artifact-abi.md`

## Why it matters

Every one of these is a reader sent nowhere from a live research record, and five of them are the ADRs that record's conclusions rest on. The record is not superseded, so it is exactly the population `AGENTS.md` says a reader follows into the tree.

## Closes when

`make citations` reports no link failure in this file. Repair the link, not the check: each intended target already exists, so the fix is the relative path.

## Verify

```sh
./check-citations.sh 2>&1 | grep -A2 backend-scoped-route-requirement-answers
```
