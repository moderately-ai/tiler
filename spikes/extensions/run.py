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
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import tomllib
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parents[1]
TRACE = ROOT / "proc-macro-visibility" / "target" / "extensions-probe-trace.log"
TOOLCHAIN_MANIFEST = REPOSITORY / "rust-toolchain.toml"
VISIBILITY_ROOT = ROOT / "non-exhaustive-visibility"
VISIBILITY_SCHEMA = "tiler-non-exhaustive-visibility/v1"
VISIBILITY_FAIL_DIR = "consuming/tests/ui/fail"
VISIBILITY_PASS_DIR = "consuming/tests/ui/pass"
MAX_OUTPUT_BYTES = 4 << 20
MAX_INPUT_FILES = 256
MAX_INPUT_BYTES = 16 << 20
MAX_RECORD_BYTES = 256 << 10
CLEANUP_REAP_SECONDS = 5.0


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
    """Terminate a command tree and reap its leader on a best-effort basis.

    Signalling a group can fail for reasons that are not probe failures: a group
    whose only member is an exited-but-unreaped child answers `EPERM`, and a
    sandboxed execution context can refuse the syscall outright. A harness must
    not fail the run it is tidying up after, so tolerate both and fall back to the
    child this process directly owns. Bound the reap as well, so an undeliverable
    signal cannot turn a bounded failure into an unbounded wait, and so a caller
    asserting that a bound was enforced observes the child's own state rather than
    the harness having waited out its natural exit.
    """
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        with contextlib.suppress(ProcessLookupError, PermissionError):
            process.kill()
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=CLEANUP_REAP_SECONDS)


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


def pinned_toolchain_channel(manifest: Path = TOOLCHAIN_MANIFEST) -> str:
    """Read the repository's sole Rust toolchain authority."""
    try:
        toolchain = tomllib.loads(manifest.read_text(encoding="utf-8"))["toolchain"]
        channel = toolchain["channel"]
    except (OSError, UnicodeError, tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise ProbeFailure(f"cannot read the pinned toolchain channel: {error}") from error
    if not isinstance(channel, str) or not channel:
        raise ProbeFailure(f"pinned toolchain channel is not a string: {channel!r}")
    return channel


def read_measurement(path: Path) -> dict[str, object]:
    """Load one retained non-exhaustive measurement under a fixed size budget."""
    try:
        if path.stat().st_size > MAX_RECORD_BYTES:
            raise ProbeFailure(f"{path.name} exceeds {MAX_RECORD_BYTES} bytes")
        record = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ProbeFailure(f"{path.name} is not a readable JSON record: {error}") from error
    if not isinstance(record, dict) or record.get("schema") != VISIBILITY_SCHEMA:
        raise ProbeFailure(f"{path.name} is not a {VISIBILITY_SCHEMA} record")
    return record


def claim_cases(label: str, claim: dict[str, object]) -> list[str]:
    """Return the fixture paths one claim names, in either singular or plural form."""
    raw = claim["case"] if "case" in claim else claim.get("cases")
    cases = [raw] if isinstance(raw, str) else raw
    if not isinstance(cases, list) or not cases or not all(isinstance(c, str) and c for c in cases):
        raise ProbeFailure(f"{label}: claim {claim.get('id')!r} names no fixture")
    return [str(case) for case in cases]


def verify_failing_case(root: Path, label: str, claim: dict[str, object], case: str) -> None:
    """Require one compile-fail fixture to still produce its recorded diagnostic."""
    source = root / case
    if not source.is_file():
        raise ProbeFailure(f"{label}: recorded fixture is missing: {case}")
    retained = source.with_suffix(".stderr")
    if not retained.is_file():
        raise ProbeFailure(f"{label}: fixture retains no diagnostic: {case}")
    try:
        text = retained.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ProbeFailure(f"{label}: cannot read {retained.name}: {error}") from error
    lines = text.splitlines()
    first = lines[0] if lines else ""
    if first != claim.get("first_line"):
        raise ProbeFailure(
            f"{label}: {case} now reports {first!r}, recorded {claim.get('first_line')!r}"
        )
    code = claim.get("diagnostic_code")
    if code is not None and f"error[{code}]" not in first:
        raise ProbeFailure(f"{label}: {case} no longer reports {code} on its first line")
    for fragment in claim.get("required_fragments", []):
        if str(fragment) not in text:
            raise ProbeFailure(f"{label}: {case} no longer reports {fragment!r}")
    for fragment in claim.get("forbidden_fragments", []):
        if str(fragment) in text:
            raise ProbeFailure(f"{label}: {case} now reports {fragment!r}, which it must not")


def verify_compiling_case(root: Path, label: str, case: str) -> None:
    """Require one compiling fixture to exist and to claim no expected failure."""
    source = root / case
    if not source.is_file():
        raise ProbeFailure(f"{label}: recorded fixture is missing: {case}")
    if source.with_suffix(".stderr").exists():
        raise ProbeFailure(f"{label}: a compiling fixture must retain no diagnostic: {case}")


def fixture_names(root: Path, directory: str) -> set[str]:
    """Enumerate the trybuild fixtures actually present in one case directory."""
    return {f"{directory}/{path.name}" for path in (root / directory).glob("*.rs")}


def trybuild_case_names(root: Path) -> list[str]:
    """Name every trybuild case a run of this workspace must report compiling.

    The paths are relative to the consuming crate, which is how trybuild prints
    them, so each is a fragment the transcript must contain.
    """
    names = sorted(
        f"tests/ui/{path.parent.name}/{path.name}"
        for directory in (VISIBILITY_FAIL_DIR, VISIBILITY_PASS_DIR)
        for path in (root / directory).glob("*.rs")
    )
    if not names:
        raise ProbeFailure(f"{root.name} retains no trybuild case to require")
    return names


def verify_visibility_evidence(root: Path, channel: str) -> dict[str, object]:
    """Check the retained `#[non_exhaustive]` diagnostics against their record.

    The `.stderr` files under the compile-fail directory *are* the measurement,
    and one `TRYBUILD=overwrite` run rewrites every one of them. This predicate
    is what stops that from silently restating a claim as whatever the current
    compiler happens to say: each recorded diagnostic must still be present,
    the inertness case must still report nothing about the omitted variant, and
    the fixture set on disk must equal the recorded set in both directions so
    that neither a deleted case nor an unrecorded one passes unnoticed.

    The channel comparison is deliberately fail-closed. `non_exhaustive_omitted_patterns`
    is an unstable lint, so a measurement is only evidence for the compiler that
    produced it; moving the pin must demand a fresh run rather than inherit the
    old conclusion.

    This is the third retained-claims custodian in the repository and the one
    that is not a copy of the other two: it checks *every* record under
    `results/` and requires the running compiler to be among those they name,
    where both shape spikes require exactly one record. Why the three are not
    one shared module is settled by `share-the-spike-diagnostic-claims-verifier`
    and recorded in `spikes/shapes/shape-evidence/verify_evidence.py`'s module
    docstring, which also states the condition that should reopen it.
    """
    measurements = sorted((root / "results").glob("*.json"))
    if not measurements:
        raise ProbeFailure("no retained non-exhaustive measurement is checked in")
    summaries: list[dict[str, object]] = []
    channels: set[str] = set()
    for path in measurements:
        label = path.name
        record = read_measurement(path)
        toolchain = record.get("toolchain")
        if not isinstance(toolchain, dict) or not isinstance(toolchain.get("channel"), str):
            raise ProbeFailure(f"{label} records no toolchain channel")
        channels.add(str(toolchain["channel"]))
        claims = record.get("claims")
        if not isinstance(claims, list) or not claims:
            raise ProbeFailure(f"{label} records no claims")
        recorded_fail: set[str] = set()
        recorded_pass: set[str] = set()
        for claim in claims:
            if not isinstance(claim, dict):
                raise ProbeFailure(f"{label} contains a malformed claim")
            cases = claim_cases(label, claim)
            outcome = claim.get("outcome")
            if outcome == "fails":
                if len(cases) != 1:
                    raise ProbeFailure(f"{label}: a failing claim names exactly one fixture")
                verify_failing_case(root, label, claim, cases[0])
                recorded_fail.add(cases[0])
            elif outcome == "compiles":
                for case in cases:
                    verify_compiling_case(root, label, case)
                    if case.startswith(f"{VISIBILITY_PASS_DIR}/"):
                        recorded_pass.add(case)
            else:
                raise ProbeFailure(f"{label}: unknown claim outcome {outcome!r}")
        for directory, recorded in (
            (VISIBILITY_FAIL_DIR, recorded_fail),
            (VISIBILITY_PASS_DIR, recorded_pass),
        ):
            present = fixture_names(root, directory)
            if present != recorded:
                raise ProbeFailure(
                    f"{label}: {directory} fixtures {sorted(present ^ recorded)} are present "
                    "without a record or recorded without a fixture"
                )
        for retained in (root / VISIBILITY_FAIL_DIR).glob("*.stderr"):
            if not retained.with_suffix(".rs").is_file():
                raise ProbeFailure(f"{label}: orphaned retained diagnostic {retained.name}")
        summaries.append(
            {
                "measurement": label,
                "channel": toolchain["channel"],
                "rustc_version": toolchain.get("rustc_version"),
                "rustc_commit_hash": toolchain.get("rustc_commit_hash"),
                "compile_fail_cases": sorted(recorded_fail),
                "compile_pass_cases": sorted(recorded_pass),
            }
        )
    if channel not in channels:
        raise ProbeFailure(
            f"retained measurements were taken on {sorted(channels)}, but rust-toolchain.toml "
            f"now pins {channel}; re-run the probe and re-record before reusing the conclusion"
        )
    return {"pinned_channel": channel, "measurements": summaries}


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


def captured_rustc_commit(records: list[CommandResult]) -> str:
    """Read the rustc commit hash the provenance step already captured."""
    for result in records:
        if result.label != "rustc provenance":
            continue
        for line in result.output.splitlines():
            if line.startswith("commit-hash:"):
                return line.split(":", 1)[1].strip()
    raise ProbeFailure("no rustc commit hash was captured before the visibility suite")


def run_non_exhaustive_visibility(deadline: float, records: list[CommandResult]) -> None:
    channel = pinned_toolchain_channel()
    summary = verify_visibility_evidence(VISIBILITY_ROOT, channel)
    records.append(
        CommandResult(
            "non-exhaustive retained evidence",
            ("internal:verify-non-exhaustive-evidence",),
            0,
            json.dumps(summary, indent=2, sort_keys=True) + "\n",
        )
    )
    # `rust-toolchain.toml` names the channel; this compares the compiler that
    # actually ran, so a run under an overriding toolchain cannot be reported as
    # evidence for the toolchain the record names.
    measurements = summary["measurements"]
    recorded = {entry.get("rustc_commit_hash") for entry in measurements if isinstance(entry, dict)}
    running = captured_rustc_commit(records)
    if running not in recorded:
        raise ProbeFailure(
            f"the running rustc is {running}, but the retained diagnostics were captured on "
            f"{sorted(str(value) for value in recorded)}; re-record before reusing the conclusion"
        )
    result = run_command(
        "non-exhaustive visibility tests",
        [
            "cargo",
            "test",
            "--locked",
            "--manifest-path",
            str(VISIBILITY_ROOT / "Cargo.toml"),
        ],
        deadline,
    )
    records.append(result)
    require_success(result)
    # Naming each compile-fail case rejects a run whose trybuild glob silently
    # matched nothing, which would otherwise report success having compiled the
    # passing direction alone. The names are derived rather than listed here
    # because a hand-maintained list decays in the one direction that matters:
    # a case added and not listed is never asserted, and nothing reports it.
    # Deriving them is not circular — the record above already required the
    # fixture set on disk to equal the recorded set in both directions, so this
    # asserts the recorded set reached the compiler.
    require_output(result, *trybuild_case_names(VISIBILITY_ROOT), "test result: ok")


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


def sole_measurement(root: Path) -> Path:
    """Return the first retained measurement, for a self-test that mutates one."""
    measurements = sorted((root / "results").glob("*.json"))
    if not measurements:
        raise ProbeFailure("self-test found no retained measurement to mutate")
    return measurements[0]


def rewrite_once(path: Path, old: str, new: str) -> None:
    """Replace one occurrence, failing if the text a mutation targets has moved."""
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise ProbeFailure(f"self-test cannot rewrite absent text {old!r} in {path.name}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def append_text(path: Path, text: str) -> None:
    """Append to an existing retained file, failing if it is absent."""
    if not path.is_file():
        raise ProbeFailure(f"self-test cannot append to absent {path.name}")
    path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")


def rename_recorded_diagnostic_code(root: Path) -> None:
    """Re-record a fixture's message text while leaving its recorded code behind.

    This is the mutation the `diagnostic_code` field exists for: the message and
    its recording agree, so only the separately recorded code still says which
    error the claim is about.
    """
    rewrite_once(sole_measurement(root), "error[E0004]", "error[E0091]")
    rewrite_once(
        root / VISIBILITY_FAIL_DIR / "cross_crate_total_map.stderr",
        "error[E0004]",
        "error[E0091]",
    )


def visibility_self_test() -> None:
    """Check the retained non-exhaustive evidence and that tampering is rejected.

    The first call is the assertion the repository gate actually runs: it reads
    the checked-in `.stderr` files and the measurement beside them, so a silent
    `TRYBUILD=overwrite` refresh or a moved toolchain pin fails here without any
    Cargo invocation. The mutations that follow keep that assertion honest by
    proving each rejection path still fires.
    """
    channel = pinned_toolchain_channel()
    verify_visibility_evidence(VISIBILITY_ROOT, channel)
    try:
        verify_visibility_evidence(VISIBILITY_ROOT, f"{channel}-moved")
    except ProbeFailure as error:
        if "re-run the probe and re-record" not in str(error):
            raise
    else:
        raise ProbeFailure("moved-toolchain-pin self-test unexpectedly succeeded")
    tampers: tuple[tuple[str, Callable[[Path], object]], ...] = (
        ("missing measurement", lambda root: shutil.rmtree(root / "results")),
        (
            "wrong record schema",
            lambda root: rewrite_once(sole_measurement(root), VISIBILITY_SCHEMA, "wrong/v1"),
        ),
        (
            "dropped required diagnostic note",
            lambda root: rewrite_once(
                root / VISIBILITY_FAIL_DIR / "cross_crate_total_map.stderr",
                "is marked as non-exhaustive",
                "is marked as something else",
            ),
        ),
        (
            "changed diagnostic first line",
            lambda root: rewrite_once(
                root / VISIBILITY_FAIL_DIR / "cross_crate_total_map.stderr",
                "error[E0004]",
                "error[E0005]",
            ),
        ),
        ("re-recorded message text under a different code", rename_recorded_diagnostic_code),
        (
            "inertness case reporting the omitted variant",
            lambda root: append_text(
                root / VISIBILITY_FAIL_DIR / "omitted_patterns_inert_without_feature.stderr",
                "some variants are not matched explicitly\n",
            ),
        ),
        (
            "deleted retained diagnostic",
            lambda root: (root / VISIBILITY_FAIL_DIR / "omitted_patterns_denied.stderr").unlink(),
        ),
        (
            "unrecorded compile-fail fixture",
            lambda root: (root / VISIBILITY_FAIL_DIR / "unrecorded.rs").write_text(
                "fn main() {}\n", encoding="utf-8"
            ),
        ),
        (
            "compiling fixture claiming a failure",
            lambda root: (root / VISIBILITY_PASS_DIR / "cross_crate_wildcard.stderr").write_text(
                "error: invented\n", encoding="utf-8"
            ),
        ),
        (
            "orphaned retained diagnostic",
            lambda root: (root / VISIBILITY_FAIL_DIR / "orphan.stderr").write_text(
                "error: invented\n", encoding="utf-8"
            ),
        ),
    )
    for name, mutate in tampers:
        with tempfile.TemporaryDirectory(prefix="tiler-non-exhaustive-") as scratch:
            copy = Path(scratch) / "non-exhaustive-visibility"
            shutil.copytree(VISIBILITY_ROOT, copy, ignore=shutil.ignore_patterns("target"))
            mutate(copy)
            try:
                verify_visibility_evidence(copy, channel)
            except ProbeFailure:
                continue
            raise ProbeFailure(f"retained-evidence self-test accepted tampering: {name}")


def self_test() -> None:
    # Run before the timing-sensitive checks below: this one reads the retained
    # non-exhaustive evidence rather than driving subprocesses, and it is the
    # part the repository gate depends on.
    visibility_self_test()
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
    # A refused group signal is otherwise only reachable inside a sandbox that
    # denies the syscall, so inject the refusal to check that cleanup tolerates
    # it and still stops the child this harness owns.
    refused = subprocess.Popen(
        [sys.executable, "-c", "import time; time.sleep(30)"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    original_killpg = os.killpg

    def refuse_group_signal(_group: int, _number: int) -> None:
        raise PermissionError("group signalling refused")

    os.killpg = refuse_group_signal
    try:
        kill_process_group(refused)
    finally:
        os.killpg = original_killpg
    if refused.returncode != -signal.SIGKILL:
        raise ProbeFailure(
            f"refused group signal left the child unstopped: returncode {refused.returncode}"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--suite",
        choices=(
            "all",
            "non-exhaustive-visibility",
            "proc-macro-visibility",
            "semantic-foundation",
        ),
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
        if args.suite in {"all", "non-exhaustive-visibility"}:
            run_non_exhaustive_visibility(deadline, records)
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
