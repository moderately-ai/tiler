---
id: preserve-the-array-api-standard-2025-12-as-a-numerics-source-record
title: Preserve the array API standard 2025.12 as a numerics source record
status: in-progress
priority: p3
dependencies: []
related: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [sources, provenance, numerics]
claimed_from: todo
assignee: agent-array-api
lease_expires_at: 1785934520
---
## User-visible outcome

The Python array API standard **2025.12** has a proper row in the numerics source record, so the operation taxonomy's citation of it is re-derivable without a version-qualified fetch of a `latest` path that moves.

## Why this exists

**Fact, 2026-08-04.** `docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md` cites the array API standard 2025.12 as one of two metadata-only citations, and its promotion-policy finding is load-bearing (the record's no-promotion posture rests partly on the standard leaving cross-kind promotion deliberately unspecified). No local copy exists and no digest was recorded — the taxonomy worker fetched it and said so rather than implying a pin.

## What this ticket owes

Follow the source-record discipline in `docs/research/numerics/sources/README.md` exactly: acquire the 2025.12 document, review its own licence text for a redistribution grant (it is expected to be permissive, in which case the verdict is **vendored** rather than metadata-only — but the verdict comes from the licence text read in the acquired copy, never from expectation), record the digest over the exact retrieved bytes, and update the manifest row, the declared population counts in `verify-sources.sh`, and the README record in the same change. Run the verifier and watch it pass; perturb one recorded digest once and watch it fail before trusting the pass.

## Closes when

The verifier passes over the incremented declared population, the record entry states the verdict with its licence ground, and the taxonomy record's "Primary sources and preservation boundary" section is updated to drop the metadata-only caveat for this source.
