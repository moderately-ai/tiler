---
id: repair-the-self-referential-link-in-the-concatenate-fusion-record
title: Repair the self-referential link in the concatenate fusion record
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/indexing]
shared_scopes: []
paths: []
tags: []
---
## What is broken

`docs/research/indexing/concatenate-fusion-role-and-lowering.md:146` links to itself with a target written from `docs/`:

```
[Concatenate fusion role and lowering](research/indexing/concatenate-fusion-role-and-lowering.md)
```

It resolves to `docs/research/indexing/research/indexing/concatenate-fusion-role-and-lowering.md`, which does not exist.

Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`.

## The judgement this needs

The link sits inside a blockquoted trigger-check bullet that reads "[Concatenate fusion role and lowering] ran the elimination on 2026-08-05 at `d5960e81`". A document citing itself in the third person is usually text that was **moved** here from a document that legitimately linked to it — most likely the deferred-question record whose trigger this bullet answers. Before repairing the path, check whether the bullet belongs here at all; if it does, the self-reference should probably become plain text rather than a link to the page the reader is already on.

## Closes when

`make citations` reports no link failure in this file.
