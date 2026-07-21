#!/usr/bin/env python3
"""Run the Rust workspace gate used by contributors and CI."""

from __future__ import annotations

import os
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(command: list[str], *, extra_env: dict[str, str] | None = None) -> None:
    env = os.environ.copy()
    if extra_env:
        env.update(extra_env)
    print(f"+ {shlex.join(command)}", flush=True)
    subprocess.run(command, cwd=ROOT, env=env, check=True)


def main() -> int:
    run([sys.executable, "scripts/check_workspace.py"])
    run(["cargo", "fmt", "--all", "--check"])
    run(["cargo", "check", "--workspace", "--all-targets"])
    run(["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
    run(["cargo", "test", "--workspace"])
    rustdocflags = f"{os.environ.get('RUSTDOCFLAGS', '')} -D warnings".strip()
    run(
        ["cargo", "doc", "--workspace", "--no-deps"],
        extra_env={"RUSTDOCFLAGS": rustdocflags},
    )
    run(
        [
            "spikes/shapes/nightly-dependent-static-shapes/check.sh",
            "nightly-2026-07-19",
        ]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
