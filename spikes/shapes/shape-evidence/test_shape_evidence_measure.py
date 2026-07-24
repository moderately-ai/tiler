"""Cleanup-path tests for the shape-evidence measurement harness.

The repository gate collects these: `spikes/shapes/shape-evidence` is in the
canonical pytest `testpaths`. Run them alone with

    uv run --locked pytest spikes/shapes/shape-evidence/test_shape_evidence_measure.py
"""

from __future__ import annotations

import importlib.util
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

import pytest

MODULE_PATH = Path(__file__).with_name("measure.py")
SPEC = importlib.util.spec_from_file_location("shape_evidence_measure", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
measure = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = measure
SPEC.loader.exec_module(measure)

RECORD_PID = "import os, pathlib, sys\npathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\n"


def test_run_enforces_its_deadline(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    child_pid = tmp_path / "pid"
    monkeypatch.setattr(measure, "TIMEOUT_SECONDS", 1)
    started = time.monotonic()

    with pytest.raises(RuntimeError, match="exceeded 1s deadline"):
        measure.run(
            [sys.executable, "-c", RECORD_PID + "import time\ntime.sleep(30)\n", str(child_pid)]
        )
    elapsed = time.monotonic() - started

    # Observe enforcement on the child, not on the kill having been attempted: a
    # 30-second command stopped inside the deadline plus one cleanup grace period
    # was terminated rather than waited out to its natural exit.
    assert elapsed < 1 + measure.CLEANUP_REAP_SECONDS
    assert child_pid.is_file()
    with pytest.raises(ProcessLookupError):
        os.kill(int(child_pid.read_text()), 0)


def test_kill_process_group_tolerates_unsignalable_group() -> None:
    process = subprocess.Popen(
        [sys.executable, "-c", "pass"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    assert process.stdout is not None
    # Reading to end of file observes the child reaching exit while it remains
    # unreaped, leaving a group whose only member is an exited leader. Darwin
    # answers `killpg` on that group with EPERM.
    assert process.stdout.read() == b""

    measure.kill_process_group(process)

    assert process.returncode is not None


def test_kill_process_group_stops_child_when_group_signal_is_refused(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def refuse_group_signal(_group: int, _number: int) -> None:
        raise PermissionError("group signalling refused")

    monkeypatch.setattr(os, "killpg", refuse_group_signal)
    process = subprocess.Popen(
        [sys.executable, "-c", "import time; time.sleep(30)"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )

    measure.kill_process_group(process)

    # A refused group signal must not fail the measurement run, and must not
    # leave a runaway child: cleanup falls back to the process the harness owns.
    assert process.returncode == -signal.SIGKILL
