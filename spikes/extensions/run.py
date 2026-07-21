#!/usr/bin/env python3
"""Run the extension experiments with bounded, checked subprocesses."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import selectors
import shlex
import signal
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parents[1]
TRACE = ROOT / "proc-macro-visibility" / "target" / "extensions-probe-trace.log"
MAX_OUTPUT_BYTES = 4 << 20
MAX_INPUT_FILES = 256
MAX_INPUT_BYTES = 16 << 20


class ProbeFailure(RuntimeError):
    """An extension experiment did not satisfy its explicit success predicate."""


def overall_timeout_handler(_signum: int, _frame: object) -> None:
    """Interrupt Python-side work when the complete-suite deadline expires."""
    raise ProbeFailure("overall extension-suite timeout expired")


def require_time(deadline: float, activity: str) -> None:
    """Fail before beginning or continuing Python-side bounded work."""
    if time.monotonic() >= deadline:
        raise ProbeFailure(f"overall timeout expired during {activity}")


@dataclass(frozen=True)
class CommandResult:
    label: str
    command: tuple[str, ...]
    returncode: int
    output: str


def kill_process_group(process: subprocess.Popen[bytes]) -> None:
    """Terminate a command tree and reap its leader."""
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGKILL)
    process.wait()


def run_command(
    label: str,
    command: list[str],
    deadline: float,
    *,
    output_limit: int = MAX_OUTPUT_BYTES,
) -> CommandResult:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise ProbeFailure(f"overall timeout expired before {label}")
    try:
        process = subprocess.Popen(
            command,
            cwd=REPOSITORY,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
    except OSError as error:
        raise ProbeFailure(f"cannot start {label}: {error}") from error
    if process.stdout is None:
        raise ProbeFailure(f"{label} capture pipe is missing")
    os.set_blocking(process.stdout.fileno(), False)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    output = bytearray()
    try:
        while selector.get_map():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                kill_process_group(process)
                raise ProbeFailure(f"overall timeout expired during {label}")
            for key, _ in selector.select(remaining):
                chunk = os.read(key.fileobj.fileno(), 65536)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                if len(output) + len(chunk) > output_limit:
                    kill_process_group(process)
                    raise ProbeFailure(f"{label} output exceeded {output_limit} bytes")
                output.extend(chunk)
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            kill_process_group(process)
            raise ProbeFailure(f"overall timeout expired during {label}")
        try:
            returncode = process.wait(timeout=remaining)
        except subprocess.TimeoutExpired as error:
            kill_process_group(process)
            raise ProbeFailure(f"overall timeout expired during {label}") from error
    finally:
        if process.poll() is None:
            kill_process_group(process)
        selector.close()
        process.stdout.close()
    return CommandResult(
        label,
        tuple(command),
        returncode,
        bytes(output).decode("utf-8", errors="replace"),
    )


def require_success(result: CommandResult) -> None:
    if result.returncode != 0:
        raise ProbeFailure(
            f"{result.label} exited {result.returncode}, expected success\n{result.output}"
        )


def require_output(result: CommandResult, *fragments: str) -> None:
    missing = [fragment for fragment in fragments if fragment not in result.output]
    if missing:
        raise ProbeFailure(f"{result.label} omitted required output {missing!r}\n{result.output}")


def require_cycle_rejection(result: CommandResult) -> None:
    if result.returncode == 0:
        raise ProbeFailure("cycle fixture unexpectedly succeeded")
    require_output(result, "cyclic package dependency")


def input_identity(deadline: float) -> dict[str, object]:
    """Identify every executable extension-fixture input under a fixed budget."""
    files = []
    for directory, directory_names, file_names in os.walk(ROOT):
        require_time(deadline, "extension input traversal")
        directory_names[:] = sorted(name for name in directory_names if name != "target")
        for name in sorted(file_names):
            path = Path(directory) / name
            if path.suffix in {".py", ".rs", ".sh"} or path.name in {
                "Cargo.toml",
                "Cargo.lock",
            }:
                files.append(path)
                if len(files) > MAX_INPUT_FILES:
                    raise ProbeFailure(f"extension inputs exceed {MAX_INPUT_FILES} files")
    files.sort()
    if len(files) > MAX_INPUT_FILES:
        raise ProbeFailure(f"extension inputs exceed {MAX_INPUT_FILES} files")
    total = 0
    records = []
    for path in files:
        require_time(deadline, "extension input hashing")
        contents = path.read_bytes()
        total += len(contents)
        if total > MAX_INPUT_BYTES:
            raise ProbeFailure(f"extension inputs exceed {MAX_INPUT_BYTES} bytes")
        records.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "bytes": len(contents),
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
        )
    if not records:
        raise ProbeFailure("extension input set is empty")
    return {"files": records, "total_bytes": total}


def run_provenance(deadline: float, records: list[CommandResult]) -> None:
    for label, command in (
        ("source revision", ["git", "rev-parse", "HEAD"]),
        ("source status", ["git", "status", "--short"]),
        ("rustc provenance", ["rustc", "--version", "--verbose"]),
        ("cargo provenance", ["cargo", "--version", "--verbose"]),
    ):
        result = run_command(label, command, deadline)
        records.append(result)
        require_success(result)
        if not result.output.strip() and label != "source status":
            raise ProbeFailure(f"{label} returned no provenance")
    records.append(
        CommandResult(
            "extension input identity",
            ("internal:hash-extension-inputs",),
            0,
            json.dumps(input_identity(deadline), indent=2, sort_keys=True) + "\n",
        )
    )


def run_operation_api(deadline: float, records: list[CommandResult]) -> None:
    result = run_command(
        "operation API tests",
        [
            "cargo",
            "test",
            "--locked",
            "--manifest-path",
            str(ROOT / "operation-api" / "Cargo.toml"),
        ],
        deadline,
    )
    records.append(result)
    require_success(result)
    require_output(result, "test result: ok")


def run_proc_macro_visibility(deadline: float, records: list[CommandResult]) -> None:
    manifest = ROOT / "proc-macro-visibility" / "Cargo.toml"
    for attempt in (1, 2):
        result = run_command(
            f"proc-macro visibility tests, attempt {attempt}",
            ["cargo", "test", "--locked", "--manifest-path", str(manifest)],
            deadline,
        )
        records.append(result)
        require_success(result)
        require_output(result, "proc_macro_sees_only_its_linked_provider_graph", "test result: ok")

    cycle = run_command(
        "reverse-dependency cycle fixture",
        [
            "cargo",
            "metadata",
            "--locked",
            "--manifest-path",
            str(ROOT / "proc-macro-visibility" / "cycle" / "consumer" / "Cargo.toml"),
        ],
        deadline,
    )
    records.append(cycle)
    require_cycle_rejection(cycle)


def run_semantic_foundation(deadline: float, records: list[CommandResult]) -> None:
    manifest = ROOT / "semantic-foundation-api-v2" / "Cargo.toml"
    check = run_command(
        "semantic foundation workspace check",
        [
            "cargo",
            "check",
            "--locked",
            "--manifest-path",
            str(manifest),
            "--workspace",
            "--all-targets",
        ],
        deadline,
    )
    records.append(check)
    require_success(check)
    consumer = run_command(
        "semantic foundation consumer",
        [
            "cargo",
            "run",
            "--locked",
            "--manifest-path",
            str(manifest),
            "-p",
            "semantic-api-consumer",
        ],
        deadline,
    )
    records.append(consumer)
    require_success(consumer)


def render_trace(records: list[CommandResult], verdict: str) -> str:
    lines = ["schema: tiler-extension-probe/v1", f"verdict: {json.dumps(verdict)}"]
    for result in records:
        lines.extend(
            (
                "",
                f"## {result.label}",
                f"command: {shlex.join(result.command)}",
                f"returncode: {result.returncode}",
                "output:",
                result.output.rstrip(),
            )
        )
    return "\n".join(lines).rstrip() + "\n"


def self_test() -> None:
    good = CommandResult("good", ("probe",), 0, "test result: ok")
    require_success(good)
    require_output(good, "test result: ok")
    malformed = (
        lambda: require_output(CommandResult("missing", ("probe",), 0, ""), "required"),
        lambda: require_cycle_rejection(CommandResult("cycle", ("probe",), 0, "")),
        lambda: require_cycle_rejection(CommandResult("cycle", ("probe",), 1, "wrong error")),
    )
    for check in malformed:
        try:
            check()
        except ProbeFailure:
            pass
        else:
            raise ProbeFailure("malformed-output self-test unexpectedly succeeded")
    try:
        run_command(
            "timeout self-test",
            [sys.executable, "-c", "import time; time.sleep(10)"],
            time.monotonic() + 0.05,
        )
    except ProbeFailure as error:
        if "timeout" not in str(error):
            raise
    else:
        raise ProbeFailure("timeout self-test unexpectedly succeeded")
    try:
        run_command(
            "output-limit self-test",
            [sys.executable, "-c", "print('x' * 1000)"],
            time.monotonic() + 5,
            output_limit=64,
        )
    except ProbeFailure as error:
        if "output exceeded 64 bytes" not in str(error):
            raise
    else:
        raise ProbeFailure("output-limit self-test unexpectedly succeeded")
    try:
        require_time(time.monotonic() - 1, "deadline self-test")
    except ProbeFailure as error:
        if "overall timeout" not in str(error):
            raise
    else:
        raise ProbeFailure("overall-deadline self-test unexpectedly succeeded")
    try:
        overall_timeout_handler(signal.SIGALRM, None)
    except ProbeFailure as error:
        if "extension-suite timeout" not in str(error):
            raise
    else:
        raise ProbeFailure("overall-timeout handler self-test unexpectedly succeeded")
    with tempfile.TemporaryDirectory(prefix="tiler-extension-alarm-") as scratch:
        child_pid = Path(scratch) / "pid"
        signal.signal(signal.SIGALRM, overall_timeout_handler)
        signal.setitimer(signal.ITIMER_REAL, 0.2)
        try:
            run_command(
                "process-alarm self-test",
                [
                    sys.executable,
                    "-c",
                    "import os,pathlib,sys,time; "
                    "pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); time.sleep(10)",
                    str(child_pid),
                ],
                time.monotonic() + 5,
            )
        except ProbeFailure as error:
            if "extension-suite timeout" not in str(error):
                raise
        else:
            raise ProbeFailure("process-alarm self-test unexpectedly succeeded")
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
        if not child_pid.is_file():
            raise ProbeFailure("process-alarm child did not start")
        try:
            os.kill(int(child_pid.read_text()), 0)
        except ProcessLookupError:
            pass
        else:
            raise ProbeFailure("process-alarm child survived timeout cleanup")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--suite",
        choices=("all", "proc-macro-visibility", "semantic-foundation"),
        default="all",
    )
    parser.add_argument("--timeout-seconds", type=float, default=300.0)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if not 1 <= args.timeout_seconds <= 900:
        parser.error("--timeout-seconds must be between 1 and 900")
    if args.self_test:
        self_test()
        print("extension probe harness: self-test passed")
        return 0

    deadline = time.monotonic() + args.timeout_seconds
    signal.signal(signal.SIGALRM, overall_timeout_handler)
    signal.setitimer(signal.ITIMER_REAL, args.timeout_seconds)
    records: list[CommandResult] = []
    verdict = "failed"
    try:
        run_provenance(deadline, records)
        if args.suite == "all":
            run_operation_api(deadline, records)
        if args.suite in {"all", "proc-macro-visibility"}:
            run_proc_macro_visibility(deadline, records)
        if args.suite in {"all", "semantic-foundation"}:
            run_semantic_foundation(deadline, records)
        verdict = "passed"
    except ProbeFailure as error:
        verdict = f"failed: {error}"
        raise
    finally:
        require_time(deadline, "trace publication")
        TRACE.parent.mkdir(parents=True, exist_ok=True)
        temporary_trace = TRACE.with_name(f".{TRACE.name}.{os.getpid()}.tmp")
        temporary_trace.write_text(render_trace(records, verdict), encoding="utf-8")
        os.replace(temporary_trace, TRACE)
        signal.setitimer(signal.ITIMER_REAL, 0)
    print(f"extension probes: {args.suite} passed; trace: {TRACE.relative_to(REPOSITORY)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ProbeFailure as error:
        print(f"extension probe failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
