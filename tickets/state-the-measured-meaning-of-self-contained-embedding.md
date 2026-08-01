---
id: state-the-measured-meaning-of-self-contained-embedding
title: State the measured meaning of self-contained embedding in the frontend contract
status: todo
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
