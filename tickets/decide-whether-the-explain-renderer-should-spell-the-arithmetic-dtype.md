---
id: decide-whether-the-explain-renderer-should-spell-the-arithmetic-dtype
title: Correct the explain renderer's stale arithmetic-dtype claim
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, explainability, drift]
---
## User-visible outcome

`render_honourability`'s documentation says what the renderer actually writes: the complete resolved dtype, not the separate arithmetic-width enum. Identity continues to encode both.

## Why this exists (drift audit 2026-08-06)

**Fact — verified 2026-08-09 at source anchor `fn render_honourability`.** The doc claims every part is written, including arithmetic dtype, while the parameter is `_arithmetic`. The renderer instead writes `resolved_type`: its nominal key when available, otherwise its canonical bytes. `encode_honourability` still encodes the separate arithmetic enum as well, so identity is unaffected.

**Fact — history eliminates the fork.** Commit `d1046e45` deliberately added `resolved_type`, changed renderer schema 6's spelling from the arithmetic key to the complete resolved dtype, left the renderer version unchanged, and renamed the parameter `_arithmetic`. The stale sentence is the only contrary evidence. Restoring the redundant arithmetic spelling would now be a new presentation change, not restoration of an accidentally deleted field.

Correct the documentation only. Do not step `EXPLAIN_RENDERER_VERSION`, change rendered bytes, or change canonical identity.

## Closes when

The doc says the complete resolved dtype and declaring profile are rendered, while the separate arithmetic enum remains canonical identity input; no renderer or schema version moves.
