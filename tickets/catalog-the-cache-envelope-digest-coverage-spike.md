---
id: catalog-the-cache-envelope-digest-coverage-spike
title: Catalog the cache envelope-digest coverage spike
status: done
priority: p2
dependencies: []
related: [decide-whether-the-bundle-envelope-section-digest-is-redundant, catalog-the-cache-hot-path-efficiency-records]
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [documentation, catalog, cache]
---
## Why this is a separate ticket

`decide-whether-the-bundle-envelope-section-digest-is-redundant` added `spikes/cache/envelope-digest-coverage/README.md` under the `research/cache` scope. The catalog that must name it is `spikes/README.md`, which is `contracts/navigation` — held by `govern-the-three-ungoverned-spike-records` for the whole of that work, and held for exactly this file. The repository rule is that a catalog is edited alongside the metadata it renders; this ticket is the deferred half, and nothing tells a reader it is missing until it lands. It is the same split `catalog-the-cache-hot-path-efficiency-records` made for the sibling spike.

The spike adds no research note — it closes an open question inside an existing record rather than adding one — but it does change what that record's catalog line renders, because the line lists the experiments supporting it and there is now a second one. Both edits are in the same file group and both are below.

## What to add

**`spikes/README.md`**, in the *Artifacts, build, and toolchains* group, alphabetically between `Expansion cache crash and race spike` and `Expansion cache hot-path efficiency probe`:

```
- [Expansion cache envelope-section digest coverage probe](cache/envelope-digest-coverage/README.md) — reproducible; exhaustive-finite, executable-model; supports: [Expansion cache hot-path efficiency](../docs/research/cache/hot-path-efficiency.md)
```

**`docs/research/README.md`**, replacing the existing `Expansion cache hot-path efficiency` line in the *Artifacts, build, and toolchains* group, which now names one experiment and must name two, in the order the new spike's `supports` and the sibling's establish (alphabetical by title):

```
- [Expansion cache hot-path efficiency](cache/hot-path-efficiency.md) — pending; bounded-measurement; informs: [Frontend and proc-macro integration](../integration/frontends.md), [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md); experiments: [Expansion cache envelope-section digest coverage probe](../../spikes/cache/envelope-digest-coverage/README.md), [Expansion cache hot-path efficiency probe](../../spikes/cache/hot-path-efficiency/README.md)
```

That second block is the existing line with one experiment inserted; everything else in it is unchanged and is transcribed from the note's own frontmatter, `disposition: "pending"` included. If `catalog-the-cache-hot-path-efficiency-records` has not landed when this ticket runs, the line to replace does not exist yet — add it whole rather than editing nothing, and check with that ticket's body that the two agree.

Both blocks are transcribed from their records' frontmatter and follow the surrounding entries' shape. Read the frontmatter before pasting rather than trusting this body: a status, disposition, or evidence class that moved between the filing of this ticket and its execution makes these lines wrong, and nothing checks them.

**The relative paths inside those blocks resolve at their destinations and not from this ticket.** The first block's paths are relative to `spikes/`, the second's to `docs/research/`. They are stated that way rather than repointed at this file, because the text that has to land is byte-identical to the text above; a link that resolved from `tickets/` would be the wrong link once pasted. Paste them unchanged and check them from the catalog they land in.

## Closes when

Both catalog entries exist, resolve, and agree with the frontmatter of the records they name.
