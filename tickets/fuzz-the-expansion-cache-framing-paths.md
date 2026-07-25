---
id: fuzz-the-expansion-cache-framing-paths
title: Fuzz the expansion cache framing and allocation paths
status: todo
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [cache, testing, correctness]
---
The second half of the research note's second follow-up gate: "fuzz every framing and bounded-allocation path".

`tiler-cache`'s bundle decoder has a rejection for every framing field, and each is covered by a directed test. Directed tests prove the checks a reader thought of. A bundle is read from a directory any process on the host may write to, so the interesting inputs are the ones nobody thought of.

## What this ticket owes

- Fuzz `bundle::decode` over arbitrary bytes and over mutations of valid bundles. The property is that it never panics, never allocates past `Limits`, and either returns a view whose sections lie inside the input or a typed rejection.
- Fuzz the entry-path parser over arbitrary strings. The property is that a parsed key round-trips to the exact text it was parsed from, so no two texts can name one entry.
- Include a resealing mutator, as `tiler-artifact`'s codec suite does: a corruption a digest catches proves only that the digest works, and the cases worth finding are the internally consistent ones.
- Decide the harness. The workspace has no fuzzing dependency and adding one is a dependency decision, so a bounded in-tree property generator may be the right answer; state which and why.
