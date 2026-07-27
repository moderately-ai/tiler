---
id: decide-whether-distributivity-directions-share-one-permission
title: Decide whether factoring and expansion share one permission
status: deferred
priority: p3
dependencies: [decide-whether-to-admit-a-distributivity-permission]
related: [settle-contraction-chain-distributivity-permission]
scopes: [contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, decision]
---
Activate only if Tiler admits a distributivity permission.

Factoring `sum(x * c)` into `sum(x) * c` and expanding it back have the same
algebraic identity but different structural preconditions and error behavior.
Determine from concrete rewrite and numerical evidence whether one permission
honestly grants both directions or whether each direction needs a distinct
caller authorization.

## Closes when

The accepted numerical contract states one or two permissions with the evidence
that distinguishes them, and every admitted rewrite checks the corresponding
direction explicitly.
