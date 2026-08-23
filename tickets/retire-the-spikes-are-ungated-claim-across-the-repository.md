---
id: retire-the-spikes-are-ungated-claim-across-the-repository
title: Retire the spikes-are-ungated claim across the repository
status: todo
priority: p2
dependencies: []
related: [correct-the-spike-portals-false-claim-that-no-make-target-reaches-spikes, decide-whether-the-citation-checker-should-reach-spike-records]
scopes: [implementation/workspace, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, spikes, gates, misattribution]
---
## User-visible outcome

No live record claims `spikes/` is reached by nothing, and no record attributes that claim to `AGENTS.md`, which says the opposite — so a reader cannot conclude from any portal, ADR, or ticket that a rotted spike link is theirs alone to catch.

## Why this exists

Filed 2026-08-22 after `worker-portal` repaired four portals and found the conflation is **a class of roughly 40 sites, not the two instances its ticket expected**. It repaired the three further sites inside its own scope and reported the rest rather than widening.

**Fact — the worst variant is misattributed authority.** Tickets attribute the ungated claim to `AGENTS.md`. **`AGENTS.md` makes no such claim anywhere**, and says the opposite at its line 263: `make citations` resolves every local markdown link in *"an open ticket, a live document, **or a retained spike record**"*. Verified by the coordinator at `56119040`: `grep -c "Nothing gates" AGENTS.md` returns **0**, and six tickets referencing `AGENTS.md` carry the ungated claim. A false claim is a repair; a false claim wearing canonical authority is what stops the next reader checking.

**Fact — `Makefile:7` is the upstream quote and is literally true.** It reads *"Spikes deliberately have no target."* Pin-quoted by **13 files** — verified by the coordinator — of which the delivering lane reports 7 are live, including ADRs 0074 and 0076 and four tickets, the rest dated audit transcripts. **The sentence is correct and must not be retired**: no `make` target builds or runs a spike, deliberately, because AGENTS.md keeps exploratory dependencies out of the build gate. What is wrong is downstream consumption of it as *"nothing reaches spikes"*.

**Fact — the distinction to restore is threefold, and the delivering lane reproduced each half.** `make citations` **checks** spike markdown links — a broken one fails with `no tracked file or directory at …`. Spike **pinned citations** are **declined by decision**, raising a declined count while the run stays green; its negative control put the same citation in root `README.md` and got exit 2, proving declination rather than a matcher that cannot parse the form. And **nothing builds or runs** a spike: `cargo metadata` reports 16 workspace packages, none under `spikes/`, which appear only in `Cargo.toml`'s `exclude`.

**Reported by the sweep, unverified by the coordinator:** `docs/decisions/0090` says spikes "gate nothing"; roughly six further `docs/research/**` sites; about twenty more tickets; and one site marked **"Verified"** in a verification table.

## Required work

- Re-audit every Fact at your base with a per-Fact verdict, and **re-derive the site census yourself** — the ~37 out-of-scope sites are agent-reported and the coordinator verified only the misattribution, the `Makefile` quote, and its 13 pin-quotes. **Say which spellings you searched for and why that set is complete**; a census is only as complete as its search vocabulary, which is how a sibling ticket closed green over live sites this week.
- Repair the misattributions **first**. They are the highest-severity subset because they borrow authority the source does not grant.
- Leave `Makefile:7` alone and repair its consumers. If a consumer's conclusion survives on the true reading, re-evidence it rather than withdrawing it.
- **Preserve retired wording in dated corrections**; grep counts cannot shrink, and expecting them to is a false progress signal.
- An **accepted ADR** carrying the claim is not a free edit: repair the implementation-status prose, never the decision, and say which you touched.

## A hazard this lane surfaced that applies to whoever takes it

The delivering lane's `AGENTS.md` copy **in its session context was stale** — it lacked the "or a retained spike record" clause the on-disk file carries. **Read the file, not your context.** That is exactly the failure mode this ticket exists to clean up, one level up.

## Non-goals

Changing the checker's declared scope, which is an accepted decision; adding any build or test target over `spikes/`, which AGENTS.md forbids; and re-repairing the four portals already corrected.

## Closes when

No live record claims spikes are unreached, no record attributes that claim to `AGENTS.md`, `Makefile:7` still stands with its consumers reading it correctly, every correction preserves what it replaced, and the census is re-derived with its search vocabulary stated.
