---
id: replace-the-digest-note-s-restated-domain-count-with-its-type
title: Replace the digest note s restated domain count with its type
status: in-progress
priority: p3
dependencies: []
related: [repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check, correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix]
scopes: [implementation/digest]
shared_scopes: [project/tickets]
paths: []
tags: [identity, digest, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786182057
---

The `tiler-digest` header was repaired on 2026-08-08 to point at the moved union check — and the repair **restated the domain count in prose**, which is the exact rot schedule that repair existed to break.

## Facts

**Reported by a sibling worker, not coordinator-verified.** `crates/tiler-digest/src/lib.rs`'s domain-separation note names "eighteen domains" in prose alongside naming `tiler_artifact::domains::GovernedDomain` as what sizes the population.

**Context that makes this worth fixing rather than shrugging at.** That same note previously said **eight**, and went stale when the population grew. The repair correctly added the type as the standing authority — and then also wrote the number, so the note now carries both a self-maintaining reference and a hand-maintained figure that will disagree the next time a domain lands. A later reader has no way to tell which side is authoritative.

## What closes this

The prose count removed, leaving `GovernedDomain` as the sole statement of the population — so a disagreement between prose and type becomes impossible rather than merely detectable. Keep the container split only if it is derived; if it is written by hand it has the same defect one level down.

**Do not replace "eighteen" with a corrected number.** That is the move this ticket exists to prevent. If a magnitude genuinely helps the reader, bind it to a commit as a historical fact — the sibling did exactly that, pinning its one surviving figure to `d48a33af` so it cannot rot.

**Beware the count you inherit.** "8 of 18" has been repeated across several tickets and **appears in no source file**; the authoritative statement is `crates/tiler-artifact/src/domains.rs`'s module header saying the retired check covered **8 of 11**. Read the source rather than a sibling ticket.

**Non-goals:** do not edit `crates/tiler-artifact/**` — read it to describe it correctly. Do not restate the union check's design; the note points at it, which is right.

Cite by searchable anchor, run the anchor's grep before committing to it, and **name the count of neighbouring claims you checked** so a clean result is distinguishable from an unexamined one.
