#!/usr/bin/env python3
"""Build scope inventory and seed ledger for the ticket audit."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

NONTERMINAL = {
    "todo",
    "ready",
    "in-progress",
    "review",
    "deferred",
    "awaiting-decision",
    "blocked",
}
TERMINAL = {"done", "closed"}
STATUS_RE = re.compile(r"^status:\s*(\S+)", re.M)


def repo_root() -> Path:
    out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
    return Path(out.strip())


def content_hash(path: Path) -> str:
    data = path.read_bytes()
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    root = repo_root()
    audit_dir = root / "docs/research/documentation/ticket-audit-2026-08-10"
    tickets_dir = root / "tickets"
    window_since = sys.argv[1] if len(sys.argv) > 1 else "2026-08-03"
    window_until = sys.argv[2] if len(sys.argv) > 2 else "2026-08-11"
    base = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True, cwd=root).strip()

    # Terminal tickets whose status line flipped to done/closed in the window
    log = subprocess.check_output(
        [
            "git",
            "log",
            f"--since={window_since}",
            f"--until={window_until}",
            "-G",
            r"^status: (done|closed)$",
            "--name-only",
            "--pretty=format:",
            "--",
            "tickets/",
        ],
        text=True,
        cwd=root,
    )
    status_flip: set[str] = set()
    for line in log.splitlines():
        line = line.strip()
        if line.startswith("tickets/") and line.endswith(".md"):
            status_flip.add(Path(line).stem)

    files = sorted(tickets_dir.glob("*.md"))
    scope: list[dict] = []
    out_of_scope: list[dict] = []
    by_status: Counter[str] = Counter()
    inclusion_counts: Counter[str] = Counter()

    for path in files:
        text = path.read_text(encoding="utf-8")
        m = STATUS_RE.search(text)
        status = m.group(1) if m else "unknown"
        by_status[status] += 1
        tid = path.stem
        h = content_hash(path)
        rec = {
            "ticket_id": tid,
            "ticket_file": f"tickets/{tid}.md",
            "repository_status": status,
            "ticket_content_hash": h,
            "audit_base_commit": base,
        }
        if status in NONTERMINAL:
            rec["inclusion_reason"] = "nonterminal"
            scope.append(rec)
            inclusion_counts["nonterminal"] += 1
        elif status in TERMINAL and tid in status_flip:
            rec["inclusion_reason"] = "terminal-completed-in-window"
            scope.append(rec)
            inclusion_counts["terminal-completed-in-window"] += 1
        else:
            rec["exclusion_reason"] = (
                "older-terminal"
                if status in TERMINAL
                else f"unhandled-status:{status}"
            )
            out_of_scope.append(rec)
            inclusion_counts["out-of-scope"] += 1

    scope.sort(key=lambda r: r["ticket_id"])
    out_of_scope.sort(key=lambda r: r["ticket_id"])

    inv = audit_dir / "inventory"
    inv.mkdir(parents=True, exist_ok=True)
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    meta = {
        "generated_at": generated_at,
        "audit_base_commit": base,
        "window_since": window_since,
        "window_until": window_until,
        "counts": {
            "tickets_total": len(files),
            "in_scope": len(scope),
            "out_of_scope": len(out_of_scope),
            "by_repository_status": dict(sorted(by_status.items())),
            "by_inclusion": dict(sorted(inclusion_counts.items())),
        },
    }

    (inv / "scope.json").write_text(
        json.dumps({"meta": meta, "tickets": scope}, indent=2) + "\n", encoding="utf-8"
    )
    (inv / "out-of-scope.json").write_text(
        json.dumps({"meta": meta, "tickets": out_of_scope}, indent=2) + "\n",
        encoding="utf-8",
    )

    # Seed / refresh ledger: preserve existing non-pending audit_state when hash+base match
    ledger_path = audit_dir / "ledger.json"
    existing: dict[str, dict] = {}
    if ledger_path.exists():
        old = json.loads(ledger_path.read_text(encoding="utf-8"))
        for row in old.get("tickets", []):
            existing[row["ticket_id"]] = row

    ledger_rows: list[dict] = []
    for rec in scope:
        tid = rec["ticket_id"]
        prev = existing.get(tid)
        if (
            prev
            and prev.get("ticket_content_hash") == rec["ticket_content_hash"]
            and prev.get("audit_base_commit") == base
            and prev.get("audit_state")
            not in (None, "pending", "claimed", "stale")
        ):
            row = dict(prev)
            row.update(
                {
                    "repository_status": rec["repository_status"],
                    "inclusion_reason": rec["inclusion_reason"],
                }
            )
        elif (
            prev
            and (
                prev.get("ticket_content_hash") != rec["ticket_content_hash"]
                or prev.get("audit_base_commit") != base
            )
            and prev.get("audit_state")
            in ("audited-clean", "audited-repair-required", "blocked")
        ):
            row = dict(prev)
            row.update(
                {
                    "repository_status": rec["repository_status"],
                    "inclusion_reason": rec["inclusion_reason"],
                    "ticket_content_hash": rec["ticket_content_hash"],
                    "audit_base_commit": base,
                    "audit_state": "stale",
                    "notes": (prev.get("notes") or "")
                    + f"; marked stale at {generated_at}",
                }
            )
        else:
            row = {
                "ticket_id": tid,
                "ticket_file": rec["ticket_file"],
                "repository_status": rec["repository_status"],
                "inclusion_reason": rec["inclusion_reason"],
                "ticket_content_hash": rec["ticket_content_hash"],
                "audit_base_commit": base,
                "audit_state": "pending",
                "assigned_worker": None,
                "assignment_time": None,
                "report_path_or_id": None,
                "report_hash": None,
                "repair_state": "not-needed",
                "repair_owner": None,
                "repair_commit": None,
                "validation_commit": None,
                "notes": "",
            }
            if prev and prev.get("audit_state") == "claimed":
                # reclaim abandoned claims on rebuild unless we want to keep them — default reclaim
                row["notes"] = "reclaimed on inventory rebuild"
        ledger_rows.append(row)

    ledger_rows.sort(key=lambda r: r["ticket_id"])
    audit_states = Counter(r["audit_state"] for r in ledger_rows)
    ledger_doc = {
        "meta": {
            **meta,
            "audit_state_counts": dict(sorted(audit_states.items())),
        },
        "tickets": ledger_rows,
    }
    ledger_path.write_text(json.dumps(ledger_doc, indent=2) + "\n", encoding="utf-8")

    # Also write jsonl mirror
    jsonl_path = audit_dir / "ledger.jsonl"
    with jsonl_path.open("w", encoding="utf-8") as f:
        for row in ledger_rows:
            f.write(json.dumps(row, sort_keys=True) + "\n")

    print(json.dumps(meta["counts"], indent=2))
    print("audit_states", dict(audit_states))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
