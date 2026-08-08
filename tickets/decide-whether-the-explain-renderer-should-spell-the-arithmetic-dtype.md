---
id: decide-whether-the-explain-renderer-should-spell-the-arithmetic-dtype
title: Decide whether the explain renderer should spell the arithmetic dtype
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`render_honourability`'s doc and behaviour agree: either the doc says the rendered trace omits the arithmetic dtype (and why identity still distinguishes it), or the dtype returns to the rendering with the renderer version stepped.

## Why this exists (drift audit 2026-08-06)

The doc claims "every part is written, including arithmetic dtype … because honourability can differ by dtype"; the parameter is `_arithmetic`, underscore-silenced (explain.rs:2468-2476). Two records differing only in dtype render identically — the exact case the doc says cannot happen. Identity is unaffected (`encode_honourability` still folds it); only the rendered trace. Commit d1046e45 removed it and deleted the pinning test, suggesting deliberate removal with a missed doc — but restoring is an EXPLAIN_RENDERER_VERSION step, so this is a fork to decide, not a sweep item: (a) correct the doc (cheap, likely right); (b) restore + version step (identity movement, executed whole). Decide with the elimination stated; if (b), the step is complete or not started.

## Closes when

Doc and behaviour agree, with the fork's elimination recorded and any version step whole.
