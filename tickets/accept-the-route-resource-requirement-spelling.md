---
id: accept-the-route-resource-requirement-spelling
title: Accept the route-resource-requirement spelling
status: done
priority: p2
dependencies: []
related: [rename-the-route-resource-floor-vocabulary-for-its-corrected-relation]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

Tom accepts or amends the public spelling `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation` landed as a reviewed draft on 2026-08-05 (commit `21485eed`): `tiler_artifact`'s `RouteResourceRequirement` (was `RouteResourceFloor`) and `RouteRequirement::Resource` (was `::ResourceFloor`). The elimination had one survivor per name (recorded in that ticket); wire bytes are untouched, so amendment costs a code rename only, no identity movement. Filed at `awaiting-decision`: only Tom closes an acceptance ticket.

## Decided — accepted

Accepted by Tom on 2026-08-05 at the third live decision review in the coordination session, witnessed first-hand by the coordinator: `RouteResourceRequirement` and `RouteRequirement::Resource` as landed at `21485eed`. No code moves; the draft names are the accepted names.
