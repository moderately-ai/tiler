# Ticket semantic audit — 2026-08-10

## Question

For every nonterminal ticket and every terminal ticket completed in the seven-day window ending 2026-08-10, does the ticket's current prose, board metadata, and graph edges match the repository at the audit base — and what exact repairs (if any) are required?

## Scope

- **Include:** all nonterminal statuses (`todo`, `ready`, `in-progress`, `review`, `deferred`, `awaiting-decision`, `blocked`); terminal tickets (`done`/`closed`) whose frontmatter `status:` line entered a terminal value in the window.
- **Exclude:** older terminal tickets (status completed before the window), even if prose was later touched.
- **Window:** 2026-08-03 inclusive through 2026-08-10 (git log `--until=2026-08-11`).
- **Audit base (Phase A read authority):** `c99ac54950f242d88d8dfe8335332bef0cf75f2d`

## How to read this record

| Path | Role |
| --- | --- |
| `inventory/scope.json` | In-scope ticket ids, content hashes, inclusion reason |
| `inventory/out-of-scope.json` | Excluded tickets and exclusion reason (no reports required) |
| `ledger.json` / `ledger.jsonl` | Living audit/repair state per in-scope ticket |
| `reports/<ticket_id>/<hash12>_<base12>.md` | Content-addressed per-ticket semantic report |
| `repairs/` | Accepted repair specs when mirrored out of reports |
| `MANIFEST.md` | Running counts and completion status |
| `scripts/` | Mechanical inventory and gates only |

A ticket is **audited** only when a complete report exists for the same ticket id, content hash, and audit base. Repository status alone never proves audit completion.

## Authority order

1. Accepted ADRs and merged contracts  
2. Current source and reproducible tests/measurements at the audit base  
3. Proposed ADRs and design documents  
4. Ticket prose and historical worker summaries  

## Status

In progress. See `MANIFEST.md` for counts.
