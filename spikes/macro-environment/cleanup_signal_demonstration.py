#!/usr/bin/env python3
"""Demonstrate the two process-group cleanup properties against any spike harness.

The `EPERM` state that motivates best-effort cleanup is reached by a race, but
the resulting behaviour is not: this driver injects the two signal outcomes
directly, so both properties are observable deterministically and comparably
across harness revisions.

- **A refused group signal** (`--case refused`) reproduces what a sandbox and an
  exited-but-unreaped group leader both produce. A harness that does not
  tolerate it crashes out of its own cleanup instead of reporting the bound it
  was enforcing.
- **An undelivered group signal** (`--case undelivered`) reproduces cleanup that
  raises nothing and stops nothing. An unbounded reap then waits out the child's
  natural exit and observes it "gone", so a caller checking that a limit was
  enforced passes while nothing was enforced. A bounded reap returns inside its
  grace period and leaves the unenforced state visible.

The driver loads a harness by path, so it also runs against an earlier revision
extracted with `git show <revision>:<path>`. `--entry` names the harness's
bounded-run function; `capture` and `run` cover every spike harness that takes a
command list. The daisy analyzer runner takes an analyzer directory instead and
is covered by its own tests.

    spikes/macro-environment/cleanup_signal_demonstration.py \
      --module spikes/macro-environment/probe.py --entry capture --case refused
"""

from __future__ import annotations

import argparse
import contextlib
import importlib.util
import os
import signal
import sys
import tempfile
import time
from pathlib import Path

RECORD_PID = "import os, pathlib, sys\npathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\n"
SLEEPING_CHILD = RECORD_PID + "import time\ntime.sleep(30)\n"
CHILD_SECONDS = 30
DEADLINE_SECONDS = 1.0


def load(path: Path) -> object:
    """Load one harness revision by path without importing its package."""
    spec = importlib.util.spec_from_file_location("harness_under_demonstration", path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"cannot load harness: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def refuse_group_signal(_group: int, _number: int) -> None:
    raise PermissionError("group signalling refused")


def swallow_group_signal(_group: int, _number: int) -> None:
    return None


def child_is_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


def bound_harness(module: object) -> None:
    """Apply the shortest deadline each harness spelling accepts."""
    if hasattr(module, "TIMEOUT_SECONDS"):
        module.TIMEOUT_SECONDS = 1
    if hasattr(module, "HARNESS_DEADLINE"):
        module.HARNESS_DEADLINE = time.monotonic() + DEADLINE_SECONDS


def drive(module: object, entry: str, command: list[str]) -> str:
    """Run the harness's bounded entry point and classify how it returned."""
    runner = getattr(module, entry)
    try:
        runner(command)
    except PermissionError as error:
        return f"CRASHED OUT OF CLEANUP: PermissionError: {error}"
    except Exception as error:  # noqa: BLE001 - any harness-declared failure is the good outcome.
        return f"reported its own bound: {type(error).__name__}"
    return "UNEXPECTED: the harness returned successfully"


def refused_case(module: object, entry: str) -> str:
    bound_harness(module)
    command = [sys.executable, "-c", f"import time; time.sleep({CHILD_SECONDS})"]
    original = os.killpg
    os.killpg = refuse_group_signal
    try:
        return drive(module, entry, command)
    finally:
        os.killpg = original


def undelivered_case(module: object, entry: str) -> str:
    with tempfile.TemporaryDirectory(prefix="tiler-cleanup-demonstration-") as scratch:
        pid_path = Path(scratch) / "pid"
        bound_harness(module)
        original = os.killpg
        os.killpg = swallow_group_signal
        started = time.monotonic()
        try:
            drive(module, entry, [sys.executable, "-c", SLEEPING_CHILD, str(pid_path)])
        finally:
            os.killpg = original
        elapsed = time.monotonic() - started
        pid = int(pid_path.read_text())
        alive = child_is_alive(pid)
        with contextlib.suppress(ProcessLookupError, ChildProcessError):
            os.kill(pid, signal.SIGKILL)
            os.waitpid(pid, 0)
    verdict = "unenforced state stays visible" if alive else "SILENT FALSE PASS"
    return (
        f"cleanup took {elapsed:.2f}s against a {CHILD_SECONDS}s child, "
        f"child_alive={alive} -> {verdict}"
    )


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--module", type=Path, required=True)
    parser.add_argument("--entry", choices=("capture", "run"), required=True)
    parser.add_argument("--case", choices=("refused", "undelivered"), required=True)
    options = parser.parse_args(sys.argv[1:] if arguments is None else arguments)
    module = load(options.module)
    if options.case == "refused":
        print(refused_case(module, options.entry))
    else:
        print(undelivered_case(module, options.entry))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
