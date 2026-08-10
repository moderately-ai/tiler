#!/usr/bin/env python3
"""Claim pending ledger tickets for a wave (status filter optional)."""
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--worker-prefix", required=True)
    ap.add_argument("--limit", type=int, required=True)
    ap.add_argument(
        "--status",
        action="append",
        dest="statuses",
        help="repository_status filter; repeatable. Default: any pending",
    )
    ap.add_argument(
        "--inclusion",
        action="append",
        dest="inclusions",
        help="inclusion_reason filter; repeatable",
    )
    ap.add_argument(
        "--ledger",
        default="docs/research/documentation/ticket-audit-2026-08-10/ledger.json",
    )
    args = ap.parse_args()
    path = Path(args.ledger)
    doc = json.loads(path.read_text(encoding="utf-8"))
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    claimed = []
    n = 0
    for row in doc["tickets"]:
        if n >= args.limit:
            break
        if row["audit_state"] != "pending":
            continue
        if args.statuses and row["repository_status"] not in args.statuses:
            continue
        if args.inclusions and row["inclusion_reason"] not in args.inclusions:
            continue
        worker = f"{args.worker_prefix}-{n:02d}"
        row["audit_state"] = "claimed"
        row["assigned_worker"] = worker
        row["assignment_time"] = now
        claimed.append(row)
        n += 1
    path.write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    # jsonl refresh of claimed only is incomplete; rewrite full jsonl
    jsonl = path.with_suffix(".jsonl")
    with jsonl.open("w", encoding="utf-8") as f:
        for row in doc["tickets"]:
            f.write(json.dumps(row, sort_keys=True) + "\n")
    print(json.dumps({"claimed": len(claimed), "tickets": claimed}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
