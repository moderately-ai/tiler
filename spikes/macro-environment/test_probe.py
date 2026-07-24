"""Malformed-output and predicate tests for the macro-environment harness."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

import probe
import pytest
from probe import ProbeFailure, parse_cfg, parse_trace_line, validate_record, verify_result

RESULTS = Path(__file__).with_name("results")
RECORD_PID = "import os, pathlib, sys\npathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\n"


def valid_trace_line() -> str:
    fields = [
        "version=1",
        "selection_hex=74617267657473203d205b6d61636f732c20696f735f6465766963652c20696f735f73696d756c61746f725d",
        "fingerprint_hex=78636f64652d61",
        "cache=miss",
    ]
    for name in (
        "HOST",
        "TARGET",
        "CARGO_BUILD_TARGET",
        "CARGO_CFG_TARGET_ARCH",
        "CARGO_CFG_TARGET_OS",
        "CARGO_CFG_TARGET_ENV",
        "CARGO_CFG_TARGET_FAMILY",
        "OUT_DIR",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "RUSTC",
        "SDKROOT",
        "MACOSX_DEPLOYMENT_TARGET",
        "IPHONEOS_DEPLOYMENT_TARGET",
    ):
        fields.append(f"env.{name}.absent=1")
    fields.extend(
        (
            "env.CARGO_MANIFEST_DIR.hex=2f746d702f666978747572652f636f6e73756d6572",
            "env.CARGO_PKG_NAME.hex=74696c65722d656e7669726f6e6d656e742d70726f62652d636f6e73756d6572",
        )
    )
    return "\t".join(fields)


def test_trace_parser_and_predicates_accept_complete_record() -> None:
    record = parse_trace_line(valid_trace_line())
    validate_record(
        record,
        fingerprint="xcode-a",
        cache="miss",
        consumer_dir=Path("/tmp/fixture/consumer"),
    )


@pytest.mark.parametrize(
    "mutation",
    (
        lambda line: line.replace("version=1", "version=2", 1),
        lambda line: line + "\tcache=hit",
        lambda line: line.replace("selection_hex=", "selection_hex=z", 1),
        lambda line: line.replace("\tenv.HOST.absent=1", "", 1),
        lambda line: line.replace("cache=miss", "cache=maybe", 1),
    ),
)
def test_trace_parser_rejects_malformed_or_incomplete_output(mutation) -> None:
    with pytest.raises(ProbeFailure):
        parse_trace_line(mutation(valid_trace_line()))


def test_predicates_reject_wrong_fingerprint_and_environment() -> None:
    record = parse_trace_line(valid_trace_line())
    with pytest.raises(ProbeFailure, match="fingerprint"):
        validate_record(
            record,
            fingerprint="xcode-b",
            cache="miss",
            consumer_dir=Path("/tmp/fixture/consumer"),
        )
    record["environment"]["TARGET"] = "aarch64-apple-darwin"
    with pytest.raises(ProbeFailure, match="TARGET"):
        validate_record(
            record,
            fingerprint="xcode-a",
            cache="miss",
            consumer_dir=Path("/tmp/fixture/consumer"),
        )


@pytest.mark.parametrize("output", ("", 'target_os="macos"\n\n', ' target_os="macos"', "x\nx"))
def test_cfg_parser_rejects_missing_malformed_or_duplicate_output(output: str) -> None:
    with pytest.raises(ProbeFailure):
        parse_cfg(output)


def test_retained_result_verifier_rejects_malformed_output(tmp_path: Path) -> None:
    malformed = tmp_path / "result.json"
    malformed.write_text('{"schema":"wrong","success":true}')
    with pytest.raises(ProbeFailure, match="schema"):
        verify_result(malformed)


def test_command_capture_rejects_output_while_streaming(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(probe, "MAX_OUTPUT_BYTES", 64)
    probe.start_deadline()
    try:
        with pytest.raises(ProbeFailure, match="stdout exceeds 64 bytes"):
            probe.capture(["python3", "-c", "print('x' * 1000)"])
    finally:
        probe.signal.setitimer(probe.signal.ITIMER_REAL, 0)


def test_command_capture_stops_the_producer_it_rejects(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setattr(probe, "MAX_OUTPUT_BYTES", 64)
    child_pid = tmp_path / "pid"
    command = [
        sys.executable,
        "-c",
        RECORD_PID + "while True:\n    print('x' * 1000, flush=True)\n",
        str(child_pid),
    ]
    probe.start_deadline()
    started = time.monotonic()
    try:
        with pytest.raises(ProbeFailure, match="stdout exceeds 64 bytes"):
            probe.capture(command)
    finally:
        probe.signal.setitimer(probe.signal.ITIMER_REAL, 0)
    elapsed = time.monotonic() - started

    # Observe the rejection on the producer, not on the kill having been
    # attempted. This child never stops on its own, so it is still running unless
    # the group signal landed, and the bounded reap keeps an undelivered signal
    # from being mistaken for a stopped child.
    assert elapsed < probe.CLEANUP_REAP_SECONDS
    assert child_pid.is_file()
    with pytest.raises(ProcessLookupError):
        probe.os.kill(int(child_pid.read_text()), 0)


def test_command_capture_enforces_deadline_while_streaming(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    child_pid = tmp_path / "pid"
    command = [sys.executable, "-c", RECORD_PID + "import time\ntime.sleep(30)\n", str(child_pid)]
    monkeypatch.setattr(probe, "HARNESS_DEADLINE", time.monotonic() + 1.0)
    started = time.monotonic()

    with pytest.raises(ProbeFailure, match="command exceeded deadline"):
        probe.capture(command)
    elapsed = time.monotonic() - started

    # A 30-second child stopped within the deadline plus one cleanup grace period
    # is direct evidence the deadline was enforced rather than waited out.
    assert elapsed < 1.0 + probe.CLEANUP_REAP_SECONDS
    assert child_pid.is_file()
    with pytest.raises(ProcessLookupError):
        probe.os.kill(int(child_pid.read_text()), 0)


def test_command_capture_enforces_deadline_after_capture_pipes_close(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    child_pid = tmp_path / "pid"
    command = [
        sys.executable,
        "-c",
        RECORD_PID + "import os, time\nos.close(1)\nos.close(2)\ntime.sleep(30)\n",
        str(child_pid),
    ]
    monkeypatch.setattr(probe, "HARNESS_DEADLINE", time.monotonic() + 1.0)
    started = time.monotonic()

    with pytest.raises(ProbeFailure, match="command exceeded deadline"):
        probe.capture(command)
    elapsed = time.monotonic() - started

    # Both pipes reach end of file at once, so this reaches the deadline through
    # the final wait rather than the streaming loop.
    assert elapsed < 1.0 + probe.CLEANUP_REAP_SECONDS
    assert child_pid.is_file()
    with pytest.raises(ProcessLookupError):
        probe.os.kill(int(child_pid.read_text()), 0)


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

    probe.kill_process_group(process)

    assert process.returncode is not None
    process.stdout.close()


def test_kill_process_group_stops_child_when_group_signal_is_refused(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def refuse_group_signal(_group: int, _number: int) -> None:
        raise PermissionError("group signalling refused")

    monkeypatch.setattr(probe.os, "killpg", refuse_group_signal)
    process = subprocess.Popen(
        [sys.executable, "-c", "import time; time.sleep(30)"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )

    probe.kill_process_group(process)

    # A refused group signal must not fail the harness, and must not leave a
    # runaway child: cleanup falls back to the process the harness owns.
    assert process.returncode == -probe.signal.SIGKILL


def test_overall_timeout_handler_fails_closed() -> None:
    with pytest.raises(ProbeFailure, match="overall deadline"):
        probe.overall_timeout_handler(0, None)


def test_overall_alarm_reaps_child_after_capture_pipes_close(tmp_path: Path) -> None:
    child_pid = tmp_path / "pid"
    probe.start_deadline()
    probe.signal.setitimer(probe.signal.ITIMER_REAL, 0.2)
    command = [
        "python3",
        "-c",
        "import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); "
        "os.close(1); os.close(2); time.sleep(10)",
        str(child_pid),
    ]
    try:
        with pytest.raises(ProbeFailure, match="overall deadline"):
            probe.capture(command)
    finally:
        probe.signal.setitimer(probe.signal.ITIMER_REAL, 0)
    assert child_pid.is_file()
    with pytest.raises(ProcessLookupError):
        probe.os.kill(int(child_pid.read_text()), 0)


def retained_result(name: str, tmp_path: Path) -> tuple[Path, dict[str, object]]:
    result = json.loads((RESULTS / name).read_text())
    path = tmp_path / name
    return path, result


def test_native_verifier_rejects_missing_provenance_and_step_tampering(tmp_path: Path) -> None:
    path, result = retained_result("native-2026-07-24.json", tmp_path)
    del result["provenance"]["rustc_verbose"]
    path.write_text(json.dumps(result))
    with pytest.raises(ProbeFailure, match="rustc provenance"):
        verify_result(path)

    path, result = retained_result("native-2026-07-24.json", tmp_path)
    result["steps"][0]["name"] = "not-cold"
    result["steps"][0]["before"] = 999
    path.write_text(json.dumps(result))
    with pytest.raises(ProbeFailure, match="step matrix"):
        verify_result(path)


def test_family_verifier_rejects_branch_and_raw_cfg_tampering(tmp_path: Path) -> None:
    path, result = retained_result("family-cfg-2026-07-24.json", tmp_path)
    result["host_cfg"]["target_os"] = 'target_os="linux"'
    result["host_cfg"]["raw_cfg"] = [
        line.replace('target_os="macos"', 'target_os="linux"')
        for line in result["host_cfg"]["raw_cfg"]
    ]
    result["matching_family_diagnostic"] = False
    result["required_compile_status"] = 0
    result["required_compile_stderr"] = ""
    path.write_text(json.dumps(result))
    with pytest.raises(ProbeFailure, match="unexpected target_os"):
        verify_result(path)

    path, result = retained_result("family-cfg-2026-07-24.json", tmp_path)
    result["target_predicates"]["aarch64-apple-ios-sim"]["raw_cfg"].append(
        'target_abi="contradictory"'
    )
    path.write_text(json.dumps(result))
    with pytest.raises(ProbeFailure, match="exactly one target_abi"):
        verify_result(path)
