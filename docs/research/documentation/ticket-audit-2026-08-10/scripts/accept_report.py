#!/usr/bin/env python3
"""Structurally gate a report and update the ledger row."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

REQUIRED = [
    "Ticket:",
    "Exact audit base:",
    "Ticket content hash:",
    "Files read in full:",
    "Per-Fact verdicts:",
    "Board and graph verdict:",
    "Repair required:",
    "Recommended audit_state:",
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ticket-id", required=True)
    ap.add_argument("--report", required=True, type=Path)
    ap.add_argument(
        "--ledger",
        default="docs/research/documentation/ticket-audit-2026-08-10/ledger.json",
    )
    ap.add_argument("--force-state", choices=["audited-clean", "audited-repair-required", "blocked"])
    args = ap.parse_args()
    text = args.report.read_text(encoding="utf-8")
    missing = [r for r in REQUIRED if r not in text]
    if missing:
        print("REJECT structural missing:", missing, file=sys.stderr)
        return 2
    m = re.search(r"Recommended audit_state:\s*(\S+)", text)
    if not m and not args.force_state:
        print("REJECT no Recommended audit_state", file=sys.stderr)
        return 2
    state = args.force_state or m.group(1).strip()
    if state not in ("audited-clean", "audited-repair-required", "blocked"):
        print("REJECT bad state", state, file=sys.stderr)
        return 2

    doc = json.loads(Path(args.ledger).read_text(encoding="utf-8"))
    row = next((t for t in doc["tickets"] if t["ticket_id"] == args.ticket_id), None)
    if not row:
        print("REJECT ticket not in ledger", file=sys.stderr)
        return 2
    # hash match: report should mention the content hash
    if row["ticket_content_hash"] not in text and row["ticket_content_hash"][:12] not in text:
        print("REJECT content hash not cited in report", file=sys.stderr)
        return 2
    if row["audit_base_commit"] not in text and row["audit_base_commit"][:12] not in text:
        print("REJECT audit base not cited in report", file=sys.stderr)
        return 2

    rh = hashlib.sha256(text.encode()).hexdigest()
    row["audit_state"] = state
    row["report_path_or_id"] = str(args.report)
    row["report_hash"] = rh
    if state == "audited-clean":
        # Do not clobber an already-integrated repair (post-repair re-audit).
        if row.get("repair_state") != "integrated":
            row["repair_state"] = "not-needed"
    elif state == "audited-repair-required":
        if row.get("repair_state") in (None, "not-needed"):
            row["repair_state"] = "pending"
    elif state == "blocked":
        if row.get("repair_state") != "integrated":
            row["repair_state"] = "blocked"

    Path(args.ledger).write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    jsonl = Path(args.ledger).with_suffix(".jsonl")
    with jsonl.open("w", encoding="utf-8") as f:
        for r in doc["tickets"]:
            f.write(json.dumps(r, sort_keys=True) + "\n")
    print(json.dumps({"ok": True, "ticket_id": args.ticket_id, "audit_state": state, "report_hash": rh}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
