---
id: stop-copying-the-carried-payload-through-the-envelope-projection
title: Stop copying the carried payload through the envelope projection
status: in-progress
priority: p2
dependencies: []
related: [measure-artifact-decoder-allocation-amplification]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [artifact, codec, performance]
claimed_from: todo
assignee: agent-payload-copy
lease_expires_at: 1786051648
---
`VerifiedArtifactProgram::encode` peaks at **4.99x** the envelope it produces —
335,609,762 bytes for a 67,222,947-byte envelope carrying a 64 MiB object, and
403,253,928 bytes requested in total. The decoder of the same envelope now peaks
at 1.00x, so the producer side is the worse amplifier by a factor of five.

Measured in
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md),
recorded in
[the research note](../docs/research/artifacts/decoder-allocation-amplification.md).
`measure-artifact-decoder-allocation-amplification` was decoder-scoped and filed
this rather than widening itself.

## Where the copies are

The `largest_blocks` column of the encode rows names them: the final envelope,
and then the carried object **four times over**, all live at once. All four are
in `ArtifactEnvelope::project`'s path through `crates/tiler-artifact/src/program/codec/model.rs`:

1. `project_payloads` clones `data.payload_content[*payload]`, which carries the
   object, into `carried`.
2. `project_sections` clones `content.code` again into its `encoded` table.
3. `project_sections` clones it a third time pushing it into `contents`.
4. `project_sections` builds `index: BTreeMap<(u8, Vec<u8>), u32>` by cloning
   **every** `contents` entry as its key, and then clones the code a fifth time
   per `index[&(tag, code.clone())]` lookup.

Then the encoder writes it into the output buffer.

## What the floor is

One. `project` takes `&ArtifactProgramData`, so the payload content cannot be
moved out and must be copied at least once; the encoder's output buffer is the
envelope the caller asked for. So the reachable shape is roughly 2x the object,
against today's 5x live and 6x requested.

The cheap part is items 3-5: `contents` is sorted and deduplicated, so a
`binary_search_by` over borrowed `(tag, &[u8])` keys replaces the owned-key
`BTreeMap` outright and removes two whole copies plus a transient one per lookup.
Items 1-2 need the projection to thread one owned buffer instead of two.

## Why it is worth doing

This is the publication path. Every artifact `tiler-macros` embeds and every
bundle the expansion cache publishes pays it, and it scales with the compiled
object — which for a real `metallib` is the whole point of the envelope.

## Closes when

`project` copies each carried object at most once before the encoder writes it,
the spike's encode rows are re-run and recorded beside the retained ones with the
new ratio, no canonical order or section identity changes (the section table is
content-addressed and its dedup semantics must be preserved exactly, including
two payloads that carry equal objects sharing one section), and `make full`
passes.
