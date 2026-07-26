#!/usr/bin/env python3
"""Land a ticket branch on `main`, or land work already committed to `main`.

The rules this replaces were written down and still violated, which is what a
script is for. AGENTS.md asked an integrator to remember to gate the exact
commit being handed out, to chain the gate to the push rather than merely run
it first, to merge the branch the worker actually committed to, and to prove
`HEAD` moved instead of accepting a silent `Already up to date`. Prose cannot
refuse; this can.

Two modes, matching how work reaches `main` here:

    scripts/land.py                  # work already committed to main
    scripts/land.py tkt/<ticket>     # a sub-agent branch

Both rebase onto the current `origin/main` and run the complete gate on the
*rebased* tree, which is the only thing that catches a pair of changes that are
each green alone and red together -- two branches touching different files, one
bumping a dependency and the other holding a golden artifact captured before
it. A textual merge reports no conflict there; only a compilation sees it.

Nothing is pushed unless the gate exits zero. There is no flag to skip it.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GATE = [sys.executable, "scripts/check_repository.py"]


class LandFailure(RuntimeError):
    """The landing sequence refused to continue."""


def git(*arguments: str, capture: bool = True) -> str:
    """Run one git command, raising a typed failure on a nonzero status."""
    completed = subprocess.run(
        ["git", *arguments],
        cwd=ROOT,
        text=True,
        capture_output=capture,
        check=False,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "").strip() if capture else ""
        raise LandFailure(f"git {' '.join(arguments)} failed: {detail}")
    return (completed.stdout or "").strip()


def head() -> str:
    """Return the current commit of the checked-out branch."""
    return git("rev-parse", "HEAD")


def require_clean() -> None:
    """Refuse to land from a dirty tree.

    An uncommitted change would be gated and then left behind by the push,
    so the commit that reaches `main` is not the tree that was checked.
    """
    if git("status", "--porcelain"):
        raise LandFailure("working tree is dirty; commit or discard before landing")


def land(branch: str | None) -> int:
    """Rebase, gate the rebased tree, and push only if the gate passed."""
    require_clean()
    current = git("rev-parse", "--abbrev-ref", "HEAD")
    if current != "main":
        raise LandFailure(f"land runs from main, not {current}")

    git("fetch", "origin", capture=False)
    before = head()
    origin = git("rev-parse", "origin/main")

    if branch is None:
        # Work is already on main; rebase it onto whatever origin has gained.
        if before != origin:
            git("rebase", "origin/main", capture=False)
    else:
        worker_tip = git("rev-parse", branch)
        print(f"landing {branch} at {worker_tip[:8]}")
        git("rebase", "origin/main", branch, capture=False)
        git("switch", "main", capture=False)
        git("reset", "--hard", "origin/main", capture=False)
        git("merge", "--ff-only", branch, capture=False)

    after_rebase = head()
    if branch is not None and after_rebase == origin:
        raise LandFailure(
            f"{branch} added nothing to origin/main; HEAD did not move, which a "
            "successful-looking merge would have reported as up to date"
        )

    print(f"+ {' '.join(GATE)}", flush=True)
    gate = subprocess.run(GATE, cwd=ROOT, check=False)
    if gate.returncode != 0:
        raise LandFailure(
            f"the complete gate failed on {after_rebase[:8]}; nothing was pushed. "
            "Fix it here, on the rebased tree the push would have published."
        )

    git("push", "origin", "main", capture=False)
    print(f"landed {after_rebase[:8]}")
    return 0


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "branch",
        nargs="?",
        help="ticket branch to land; omit when the work is already committed to main",
    )
    parsed = parser.parse_args(arguments)
    try:
        return land(parsed.branch)
    except LandFailure as error:
        print(f"land failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
