---
id: state-the-measured-meaning-of-self-contained-embedding
title: State the measured meaning of self-contained embedding in the frontend contract
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contract]
---
## User-visible outcome

The frontend contract says what "a self-contained AOT and embedding unit" was measured to mean, so a reader learns the property from the contract rather than having to find the research note that established it.

## Why

`docs/integration/frontends.md` states the accepted inline-DX invariant that each invocation is a self-contained AOT and embedding unit. It does not say what self-containment was measured to be. `docs/research/embedding/self-contained-embedding.md` now establishes it on one recorded host: a consumer builds and runs with every Tiler-produced artifact and the entire expansion-cache root deleted; the payload travels as exactly one byte-string literal; and the artifact and the cache are inputs to the expansion, never to the expanded code.

Adding the sentence needs the `contracts/integrations` scope, which `prototype-macro-embedding-and-cargo-behavior` did not hold.

## Closes when

1. The contract states the measured property, cites the research note, and names its measurement boundary rather than generalizing it into a portable guarantee.
2. The distinction the evidence turns on is preserved: the artifact must exist the first time a build expands an invocation, and is not needed afterwards.
3. Whether the statement is normative or descriptive is decided explicitly. If it would be a new accepted invariant rather than a description of one already accepted, it is Tom's under ADR 0075 and goes to him before acceptance.

## Outcome (2026-07-31)

**Fact.** `docs/integration/frontends.md` now states, beside the sentence that asserts self-containment, what the property was measured to be: the sixteen-file deletion demonstration, the one-byte-string-literal representation, and the distinction the evidence turns on — the artifact and cache are inputs to the expansion and never to the expanded code, so the artifact must exist at first expansion and is needed by nothing afterwards. The statement cites the research note and names its boundary (one recorded host, toolchain, and artifact population; not a portable guarantee).

**Fact — normative versus descriptive, decided explicitly.** The sentence is descriptive of an invariant ADR 0004 and this contract already accept, and says so ("describing the invariant ADR 0004 already accepts"). It adds no new obligation, so no ADR 0075 escalation was required; had it been written as a new invariant it would have been Tom's.

**Fact — one adjacent staleness corrected in the same file.** The generated-paths paragraph still named `::tiler::__private::expansion_anchor()`, which the frontend landing removed; it now names `RegionFacts` and `bind_and_build`, the surface Tom accepted on 2026-07-31, with the anchor's removal recorded.
