---
id: prototype-region-cover-enumeration
title: Enumerate legal complete region covers
status: todo
priority: p0
dependencies: [prototype-fusion-legality-and-numerical-proof]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning]
---
Enumerate bounded legal covers before physical program selection. Cover every
operation and named output, preserve occurrence identity and boundaries,
conservatively materialize fan-out unless duplication is explicitly legal, and
retain fused and singleton/materialized covers. This stage does not choose
implementations or claim a complete executable program.

Cover identity binds semantic graph meaning, exact region occurrences,
coverage, deliberate duplication, and proposed materialization edges. Local
physical frontiers are enumerated next; complete program selection follows
both.
