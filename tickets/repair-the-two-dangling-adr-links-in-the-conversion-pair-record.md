---
id: repair-the-two-dangling-adr-links-in-the-conversion-pair-record
title: Repair the two dangling ADR links in the conversion pair record
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/numerics]
shared_scopes: []
paths: []
tags: []
---
## What is broken

`docs/research/numerics/conversion-family-decomposition-across-pairs.md` links to two ADRs as bare filenames, as if the record lived in `docs/decisions/`:

- `](0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)`
- `](0041-separate-float-to-integer-conversion-families.md)`

Both resolve against `docs/research/numerics/` and find nothing. Both files exist at `docs/decisions/<same-name>`, verified 2026-08-08, so only the relative prefix is wrong.

Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`.

## Closes when

`make citations` reports no link failure in this file. The intended targets exist; the fix is the relative path, not the check.
