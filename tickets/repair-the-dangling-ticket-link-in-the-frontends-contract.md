---
id: repair-the-dangling-ticket-link-in-the-frontends-contract
title: Repair the dangling ticket link in the frontends contract
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## What is broken

`docs/integration/frontends.md:515` links to a ticket that no longer exists:

```
[`state-a-debug-retention-from-the-inline-frontend`](../../tickets/state-a-debug-retention-from-the-inline-frontend.md)
```

> **Fact repair, 2026-08-08, at base `db3f4d07`.** The claim struck below read: "`tickets/state-a-debug-retention-from-the-inline-frontend.md` was removed at `3249a5a3`." **It was renamed, not removed.** `git show --name-status -M 3249a5a3` reports `R087 tickets/state-a-debug-retention-from-the-inline-frontend.md tickets/emit-from-a-populated-retention-in-the-inline-expansion.md`; the plain `--name-status` that reports it as a `D` is rename detection being off, not a deletion. The commit subject's own third clause — "re-scope the retention read-back" — names the rename. The rest of that sentence is Verified: the sibling link `retain-succeeding-metal-stage-tool-output` does resolve.

`tickets/state-a-debug-retention-from-the-inline-frontend.md` was renamed to `tickets/emit-from-a-populated-retention-in-the-inline-expansion.md` at `3249a5a3` ("Repair four fired tickets: strike expired claims, settle the unsafe-pin posture, re-scope the retention read-back"). The sibling link on the same line, `retain-succeeding-metal-stage-tool-output`, still resolves.

Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`.

## The judgement this needs

> **This framing is superseded, 2026-08-08.** All three options below presume the bullet is still outstanding and that only its citation is wrong. **It is not: the bullet's prose was false at this base, and both of its owners are `done`.** `retain-succeeding-metal-stage-tool-output` reads `status: done`, and so does the renamed `emit-from-a-populated-retention-in-the-inline-expansion` (landed at merge `08714fd7`). Against source: `grep -n 'ToolOutput::capture' crates/tiler-metal-aot/src/driver.rs` returns **two** sites, `:304` in the failure arm and `:307` as the `Ok` value, so "drops both streams on success" is false; `grep -n 'retained: stage_retention' crates/tiler-build/src/metal_cache.rs` returns `:403`, so "`accept_or_publish_delivered_metal_artifact` states `DebugRetention::none()`" is false; and `crates/tiler-macros/src/retention.rs` exists and is called from `aot.rs:698`, so "the invocation that would ask for one" is built. Repointing the link under the **Still outstanding** heading — the narrowest reading of this ticket — would have produced exactly the failure the ticket's last line forbids, in its worse form: a link that *resolves*, so the checker never flags it again, attached to prose contradicted by the code and filed under a heading that hides a delivered capability. The repair taken instead was to move the item to **Landed** with a date, per the section's own stated convention that "an item moves in the change that discharges it and says on which date it moved".

The sentence names an owner for a capability the contract declines to deliver. Decide which is true and write that:

- the work moved to a differently-named ticket, and the link should point at it;
- the work was folded into `retain-succeeding-metal-stage-tool-output`, and the second clause should go; or
- there is no owner, and the prose should say so rather than name a ghost.

Do not repoint the link at a ticket that does not carry the obligation just to make the check green.

## Note on the shape

A contract document linking into `tickets/` is a link into a mutable work graph from an evidence record. Whether that is a good idea at all is worth a moment while you are here — a ticket is deleted or renamed far more readily than a document is.

## Closes when

`make citations` reports no link failure in this file.

## Outcome audit — 2026-08-09

Delivered by `a9c2119cfe1e5bc94f053d0ad9d83f94584e67e0`. The repair did not merely redirect a ghost link: the frontends contract moved the already-delivered retention read-back from `Still outstanding` to `Landed`, names both completed owners — including the renamed `emit-from-a-populated-retention-in-the-inline-expansion` — and records the producer capture, publication retention, and expansion read-back that make the old prose false. The empty outstanding section is retained explicitly, and the caller-visible acceptance question remains linked to its separate decision ticket. The current citation gate resolves every link in the repaired bullet.
