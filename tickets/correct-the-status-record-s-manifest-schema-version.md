---
id: correct-the-status-record-s-manifest-schema-version
title: Correct the status record's manifest schema version
status: todo
priority: p3
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The version drifted by one major step

`docs/status.md` states "neutral manifest schema **14.0**" and that "the manifest took a major step to **`14.0`**". `crates/tiler-artifact/src/program/codec/encode.rs` declares `MANIFEST_SCHEMA: (u16, u16) = (15, 0)` — coordinator-verified.

**The surrounding conclusion survives**; the "moved again, alone" narrative describes the step *before* the current one, and every other constant in that sentence checks out. So this is a dated measurement outliving its date rather than a false claim — state it that way, as the repository's convention distinguishes.

Follow the file's own dated-correction convention. Cite by **searchable anchor rather than line number**: `make citations` now covers `docs/**`, so a drifted citation is a red gate.

## Closes when

The stated schema matches `MANIFEST_SCHEMA`, the correction records what the retired figure measured and when, and `make citations` passes.
