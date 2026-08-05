---
id: refresh-the-spike-catalog-entry-for-the-c1-attribution-fixture
title: Refresh the spike catalog entry for the C1 attribution fixture
status: in-progress
priority: p3
dependencies: []
related: [retain-the-c1-model-attribution-fixture, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, catalog, navigation]
claimed_from: todo
assignee: agent-c1-catalog
lease_expires_at: 1785948034
---
## User-visible outcome

The spike catalog at `spikes/README.md` names the C1 fixture by what it now holds, so a reader looking for the attribution surface finds it from the catalog rather than only from the research records that cite it.

## Why this exists

**Fact.** [`retain-the-c1-model-attribution-fixture`](retain-the-c1-model-attribution-fixture.md) extended the fixture with L6's attribution surface and retitled it *Qwen3-0.6B-Base C1 conformance and attribution reference fixture*, adding `tiler.research.program-planning.complete-model-ingestion-and-execution` to its `supports` list. `spikes/README.md` line 64 still reads "Qwen3 conformance reference logit fixture" and names only the workload profile as supported.

**Why it was not fixed in that change.** `spikes/README.md` belongs to `contracts/navigation`, and that ticket held `research/program-planning` and `project/tickets` only. Editing it would have been a scope escape, which is worse than a stale catalog line; AGENTS.md asks for a filed ticket rather than a silent absorption.

## Required content

One catalog line, matched to the spike's own frontmatter: the current title, and both supported records.

## Closes when

`spikes/README.md`'s entry agrees with `spikes/program-planning/qwen3-conformance-fixture/README.md`'s `title` and `supports` fields, and no other catalog line disagrees with the spike frontmatter it describes.
