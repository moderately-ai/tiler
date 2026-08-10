#!/usr/bin/env python3
"""Mark a Phase B repair integrated and refresh content hash + optional stale audit."""
from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ticket-id", required=True)
    ap.add_argument("--repair-commit", required=True)
    ap.add_argument(
        "--ledger",
        default="docs/research/documentation/ticket-audit-2026-08-10/ledger.json",
    )
    ap.add_argument(
        "--mark-stale",
        action="store_true",
        help="Set audit_state=stale so a re-audit is required after content change",
    )
    ap.add_argument("--notes", default="")
    args = ap.parse_args()

    ledger_path = Path(args.ledger)
    doc = json.loads(ledger_path.read_text(encoding="utf-8"))
    row = next((t for t in doc["tickets"] if t["ticket_id"] == args.ticket_id), None)
    if row is None:
        print(f"REJECT unknown ticket {args.ticket_id}")
        return 2

    ticket_path = Path(row["ticket_file"])
    if not ticket_path.exists():
        print(f"REJECT missing ticket file {ticket_path}")
        return 2

    new_hash = sha256_file(ticket_path)
    row["ticket_content_hash"] = new_hash
    row["repair_state"] = "integrated"
    row["repair_commit"] = args.repair_commit
    row["repair_owner"] = row.get("repair_owner") or "phase-b"
    if args.notes:
        row["notes"] = (row.get("notes") or "") + (("; " if row.get("notes") else "") + args.notes)
    if args.mark_stale:
        row["audit_state"] = "stale"
        row["report_path_or_id"] = None
        row["report_hash"] = None

    ledger_path.write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    jsonl = ledger_path.with_suffix(".jsonl")
    with jsonl.open("w", encoding="utf-8") as f:
        for t in doc["tickets"]:
            f.write(json.dumps(t, sort_keys=True) + "\n")
    print(
        json.dumps(
            {
                "ticket_id": args.ticket_id,
                "repair_state": row["repair_state"],
                "repair_commit": row["repair_commit"],
                "ticket_content_hash": new_hash,
                "audit_state": row["audit_state"],
                "updated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
