---
id: rename-the-apple-numerical-record-past-one-dtype
title: Rename the Apple GPU numerical-behaviour record
status: todo
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype]
scopes: [research/apple-targets, contracts/navigation, contracts/decisions, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, apple-targets]
---
`docs/research/apple-targets/numerical-behaviour.md` is titled "Apple GPU f32
numerical behaviour", but its later findings measure `f16` and `bf16` as well.
The title now names the record's origin rather than its extent.

The rename is a separate ticket because the title is copied into manually
maintained catalogs and prose citations across navigation, decisions, backend,
and integration documents. Nothing regenerates or validates those copies, so
the ticket must update every checked-in occurrence in the same change. The
record carries an explicit stale-title note that this ticket removes.

**What closes this.** The frontmatter `title` and the `#` heading name what the record measures without naming one dtype; every catalog block that quotes it has been updated by hand, since nothing regenerates them; the three prose citations read correctly; and the record's "this record's title is stale" note is gone rather than reworded.
