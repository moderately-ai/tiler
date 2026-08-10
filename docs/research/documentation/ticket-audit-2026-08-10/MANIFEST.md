# Manifest — ticket audit 2026-08-10

- Audit base (Phase A read authority): `c99ac54950f242d88d8dfe8335332bef0cf75f2d`
- In scope: **700**
- Out of scope (older terminal): **602**
- Living authority: `ledger.json`

## Phase A

- Completed at frozen base for all 700 in-scope tickets.
- Rows may become `stale` after Phase B content changes until re-audited.

## Ledger snapshot

### audit_state
- `audited-clean`: 342
- `audited-repair-required`: 338
- `stale`: 20

### repair_state
- `integrated`: 28
- `not-needed`: 335
- `pending`: 337

- Open repairs (`pending`): **337**
- Integrated repairs: **28**

## Phase B progress

- Wave B1 complete: 20 class-A terminal ticket-only repairs integrated; marked `stale` for re-audit.
- Queue: `repairs/phase-b-queue.json`
- Next: continue class-A bulk at higher parallelism; then B/C/D; re-audit stale rows.

