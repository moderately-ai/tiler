---
id: repair-the-dangling-ticket-link-in-the-frontends-contract
title: Repair the dangling ticket link in the frontends contract
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786167947
---
## What is broken

`docs/integration/frontends.md:515` links to a ticket that no longer exists:

```
[`state-a-debug-retention-from-the-inline-frontend`](../../tickets/state-a-debug-retention-from-the-inline-frontend.md)
```

`tickets/state-a-debug-retention-from-the-inline-frontend.md` was removed at `3249a5a3` ("Repair four fired tickets: strike expired claims, settle the unsafe-pin posture, re-scope the retention read-back"). The sibling link on the same line, `retain-succeeding-metal-stage-tool-output`, still resolves.

Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`.

## The judgement this needs

The sentence names an owner for a capability the contract declines to deliver. Decide which is true and write that:

- the work moved to a differently-named ticket, and the link should point at it;
- the work was folded into `retain-succeeding-metal-stage-tool-output`, and the second clause should go; or
- there is no owner, and the prose should say so rather than name a ghost.

Do not repoint the link at a ticket that does not carry the obligation just to make the check green.

## Note on the shape

A contract document linking into `tickets/` is a link into a mutable work graph from an evidence record. Whether that is a good idea at all is worth a moment while you are here — a ticket is deleted or renamed far more readily than a document is.

## Closes when

`make citations` reports no link failure in this file.
