# Manifest — ticket audit 2026-08-10

- Audit base (Phase A read authority): `c99ac54950f242d88d8dfe8335332bef0cf75f2d`
- In scope: **700**
- Out of scope (older terminal): **602**
- Living authority: `ledger.json` (this file is a snapshot; re-derive counts from the ledger when they disagree)

## Phase A (complete)

- **Audited: 700 / 700**
- `audited-clean`: 342
- `audited-repair-required`: 358
- `pending` / `claimed`: 0

## Phase B (in progress)

- `repair_state=pending`: **357** (open repair work orders)
- `repair_state=integrated`: 8 (early open-board batch)
- `repair_state=not-needed`: 335

### Queue classes (see `repairs/phase-b-queue.json`)

Derived from each report's Repair required / Exact files sections. Class is a batching hint, not a hard gate.

| Class | Meaning | Count (approx at queue build) |
| --- | --- | --- |
| A | Terminal (`done`/`closed`), ticket-record repair | ~196 |
| B | Nonterminal board repairs | ~122+ |
| C | Ticket + docs residual | ~16 |
| D | Ticket + new remainder filing | ~14 |
| E | Code residual / authority escalate | ~6+ |

Strict class-A (ticket-only file list, public consequences none): **98** at queue build.

### Progress

- Wave B1: first **20** strict class-A terminal ticket-only repairs (low concurrency process pilot)
- Later waves: scale parallelism after B1 process is solid; then re-audit integrated hashes

## How to re-count

```sh
python3 - <<'PY'
import json
from collections import Counter
from pathlib import Path
doc=json.loads(Path('docs/research/documentation/ticket-audit-2026-08-10/ledger.json').read_text())
print('audit', Counter(t['audit_state'] for t in doc['tickets']))
print('repair', Counter(t.get('repair_state') for t in doc['tickets']))
print('pending repairs', sum(1 for t in doc['tickets'] if t.get('repair_state')=='pending'))
PY
```
