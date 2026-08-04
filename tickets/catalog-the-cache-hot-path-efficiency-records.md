---
id: catalog-the-cache-hot-path-efficiency-records
title: Catalog the cache hot-path efficiency records
status: todo
priority: p2
dependencies: []
related: [measure-the-expansion-cache-hot-path-efficiency]
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [documentation, catalog, cache]
---
## Why this is a separate ticket

`measure-the-expansion-cache-hot-path-efficiency` added `docs/research/cache/hot-path-efficiency.md` and `spikes/cache/hot-path-efficiency/README.md` under the `research/cache` scope. Both catalogs that must name them live in `contracts/navigation`, which two other tickets held open while that work ran, so the entries could not land in the same change as the metadata behind them. The repository rule is that a catalog is edited alongside the metadata it renders; this ticket is the deferred half, and nothing tells a reader it is missing until it lands.

## What to add

**`docs/research/README.md`**, in the *Artifacts, build, and toolchains* group, alphabetically between `Expansion cache crash and race protocol` and `Proc-macro build environment and freshness`:

```
- [Expansion cache hot-path efficiency](cache/hot-path-efficiency.md) — pending; bounded-measurement; informs: [Frontend and proc-macro integration](../integration/frontends.md), [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md); experiments: [Expansion cache hot-path efficiency probe](../../spikes/cache/hot-path-efficiency/README.md)
```

**`spikes/README.md`**, in the *Artifacts, build, and toolchains* group, alphabetically after `Expansion cache crash and race spike`:

```
- [Expansion cache hot-path efficiency probe](cache/hot-path-efficiency/README.md) — reproducible; bounded-measurement; supports: [Expansion cache hot-path efficiency](../docs/research/cache/hot-path-efficiency.md)
```

Both lines are transcribed from the two records' own frontmatter and follow the surrounding entries' shape. Read the frontmatter before pasting rather than trusting this body: a status or evidence class that moved between the filing of this ticket and its execution makes these lines wrong, and nothing checks them.

**The relative paths inside those two blocks resolve at their destinations and not from this ticket.** The first block's paths are relative to `docs/research/`, the second's to `spikes/`. They are stated that way rather than repointed at this file, because the text that has to land is byte-identical to the text above; a link that resolved from `tickets/` would be the wrong link once pasted. Paste them unchanged and check them from the catalog they land in.

## Closes when

Both catalog entries exist, resolve, and agree with the frontmatter of the records they name.
