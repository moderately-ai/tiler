#!/usr/bin/env python3
"""Run and validate the pinned Daisy sound-accuracy profiles."""

from __future__ import annotations

import contextlib
import csv
import hashlib
import json
import multiprocessing
import os
import re
import resource
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
from decimal import Decimal, InvalidOperation
from multiprocessing.connection import Connection
from pathlib import Path

UNKNOWN_EXIT = 10
ANSI_ESCAPE = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
SCALA_CLASSPATH = re.compile(r'^SCALACLASSPATH="([^"]*)"$', re.MULTILINE)
MAX_DIAGNOSTIC_BYTES = 1 << 20
MAX_RESULT_BYTES = 1 << 20
MAX_RESULT_ROWS = 64
MAX_FIELD_CHARS = 4096
MAX_PROVENANCE_FILES = 4096
MAX_PROVENANCE_DIRECTORIES = 4096
MAX_PROVENANCE_BYTES = 512 << 20
PROVENANCE_TIMEOUT_SECONDS = 30


class Unknown(RuntimeError):
    """A stable failure to produce complete proof evidence."""

    def __init__(self, reason: str, profile: str, detail: str) -> None:
        super().__init__(detail)
        self.reason = reason
        self.profile = profile
        self.detail = detail


def fail_unknown(reason: str, profile: str, detail: str) -> None:
    """Stop analysis without publishing proof evidence."""
    raise Unknown(reason, profile, detail)


def finite_decimal(value: str, field: str, profile: str) -> Decimal:
    """Parse a required finite analyzer number or return Unknown."""
    try:
        parsed = Decimal(value)
    except InvalidOperation:
        fail_unknown("analyzer_diagnostic", profile, f"invalid {field}: {value!r}")
    if not parsed.is_finite():
        fail_unknown("analyzer_diagnostic", profile, f"non-finite {field}: {value!r}")
    return parsed


def sha256_file(path: Path, maximum: int, *, profile: str) -> tuple[str, int]:
    """Hash one bounded regular file without retaining it in memory."""
    digest = hashlib.sha256()
    size = 0
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_NONBLOCK)
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            os.close(descriptor)
            fail_unknown("analyzer_provenance", profile, f"not a regular file: {path}")
        if metadata.st_size > maximum:
            os.close(descriptor)
            fail_unknown(
                "analyzer_provenance",
                profile,
                f"{path} exceeds remaining {maximum}-byte provenance budget",
            )
        with os.fdopen(descriptor, "rb") as source:
            while chunk := source.read(1 << 20):
                digest.update(chunk)
                size += len(chunk)
                if size > maximum:
                    fail_unknown(
                        "analyzer_provenance",
                        profile,
                        f"{path} exceeds remaining {maximum}-byte provenance budget",
                    )
    except OSError as error:
        fail_unknown("analyzer_provenance", profile, f"cannot hash {path}: {error}")
    return digest.hexdigest(), size


def checked_revision(daisy_root: Path, expected: str) -> str:
    """Verify the Daisy checkout revision and tracked-clean state in-process."""
    profile = "preflight"
    try:
        revision = subprocess.run(
            ("git", "-C", str(daisy_root), "rev-parse", "--verify", "HEAD"),
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        ).stdout.strip()
        subprocess.run(
            ("git", "-C", str(daisy_root), "diff-index", "--quiet", "HEAD", "--"),
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as error:
        fail_unknown("analyzer_provenance", profile, f"cannot verify Daisy checkout: {error}")
    if revision != expected:
        fail_unknown(
            "analyzer_provenance",
            profile,
            f"Daisy checkout is {revision}, expected {expected}",
        )
    return revision


def analyzer_provenance(daisy_root: Path, spike_dir: Path, revision: str) -> dict[str, object]:
    """Fingerprint the concrete analyzer inputs selected by the launcher."""
    profile = "preflight"
    launcher = daisy_root / "daisy"
    byte_count = 0
    launcher_digest, launcher_size = sha256_file(
        launcher, MAX_PROVENANCE_BYTES - byte_count, profile=profile
    )
    byte_count += launcher_size
    try:
        launcher_text = read_bounded(
            launcher,
            launcher_size,
            profile=profile,
            kind="Daisy launcher",
        ).decode("utf-8")
    except (OSError, UnicodeError) as error:
        fail_unknown("analyzer_provenance", profile, f"cannot read Daisy launcher: {error}")
    matched = SCALA_CLASSPATH.search(launcher_text)
    if matched is None:
        fail_unknown("analyzer_provenance", profile, "Daisy launcher has no literal SCALACLASSPATH")

    entries = tuple(Path(raw) for raw in matched.group(1).split(os.pathsep) if raw)
    if not entries:
        fail_unknown("analyzer_provenance", profile, "Daisy launcher classpath is empty")

    closure = hashlib.sha256()
    file_count = 0
    directory_count = 0
    classpath_bytes = 0
    for entry in entries:
        entry = entry if entry.is_absolute() else daisy_root / entry
        if not entry.exists():
            fail_unknown("analyzer_provenance", profile, f"missing classpath entry: {entry}")
        closure.update(f"entry\0{entry}".encode())
        if entry.is_file():
            files = iter((entry,))
        else:
            discovered: list[Path] = []
            for directory, directories, names in os.walk(entry):
                directory_count += 1
                if directory_count > MAX_PROVENANCE_DIRECTORIES:
                    fail_unknown(
                        "analyzer_provenance",
                        profile,
                        f"classpath exceeds {MAX_PROVENANCE_DIRECTORIES} directories",
                    )
                directories.sort()
                discovered.extend(Path(directory) / name for name in sorted(names))
                if file_count + len(discovered) > MAX_PROVENANCE_FILES:
                    fail_unknown(
                        "analyzer_provenance",
                        profile,
                        f"classpath exceeds {MAX_PROVENANCE_FILES} fingerprinted files",
                    )
            files = iter(discovered)
        for path in files:
            file_count += 1
            if file_count > MAX_PROVENANCE_FILES:
                fail_unknown(
                    "analyzer_provenance",
                    profile,
                    f"classpath exceeds {MAX_PROVENANCE_FILES} fingerprinted files",
                )
            file_digest, file_size = sha256_file(
                path, MAX_PROVENANCE_BYTES - byte_count, profile=profile
            )
            byte_count += file_size
            classpath_bytes += file_size
            if byte_count > MAX_PROVENANCE_BYTES:
                fail_unknown(
                    "analyzer_provenance",
                    profile,
                    f"classpath exceeds {MAX_PROVENANCE_BYTES} fingerprinted bytes",
                )
            logical_name = f"{entry}\0{path.relative_to(entry) if entry.is_dir() else path.name}"
            closure.update(logical_name.encode())
            closure.update(bytes.fromhex(file_digest))

    java = shutil.which("java")
    if java is None:
        fail_unknown("analyzer_provenance", profile, "Daisy launcher requires java on PATH")
    java_path = Path(java).resolve()
    java_digest, java_size = sha256_file(
        java_path, MAX_PROVENANCE_BYTES - byte_count, profile=profile
    )
    byte_count += java_size

    inputs: dict[str, str] = {}
    for name in ("daisy_runner.py", "scalar_regions.scala", "mixed-precision.txt"):
        digest, input_size = sha256_file(
            spike_dir / name, MAX_PROVENANCE_BYTES - byte_count, profile=profile
        )
        byte_count += input_size
        inputs[name] = digest

    return {
        "name": "Daisy",
        "source_revision": revision,
        "launcher_sha256": launcher_digest,
        "launcher_bytes": launcher_size,
        "classpath_sha256": closure.hexdigest(),
        "classpath_files": file_count,
        "classpath_directories": directory_count,
        "classpath_bytes": classpath_bytes,
        "java_path": str(java_path),
        "java_sha256": java_digest,
        "java_bytes": java_size,
        "input_sha256": inputs,
        "fingerprinted_bytes": byte_count,
    }


def provenance_worker(
    connection: Connection,
    daisy_root: Path,
    spike_dir: Path,
    revision: str,
) -> None:
    """Collect provenance in an isolated process with a parent-owned deadline."""
    try:
        connection.send(("ok", analyzer_provenance(daisy_root, spike_dir, revision)))
    except Unknown as unknown:
        connection.send(("unknown", (unknown.reason, unknown.profile, unknown.detail)))
    except BaseException as error:  # noqa: BLE001 - worker must fail closed across all exits.
        connection.send(("error", repr(error)))
    finally:
        connection.close()


def bounded_analyzer_provenance(
    daisy_root: Path, spike_dir: Path, revision: str
) -> dict[str, object]:
    """Collect analyzer provenance under a hard wall-clock deadline."""
    context = multiprocessing.get_context("spawn")
    receiver, sender = context.Pipe(duplex=False)
    process = context.Process(
        target=provenance_worker,
        args=(sender, daisy_root, spike_dir, revision),
    )
    process.start()
    sender.close()
    process.join(PROVENANCE_TIMEOUT_SECONDS)
    if process.is_alive():
        process.kill()
        process.join()
        receiver.close()
        fail_unknown(
            "analyzer_timeout",
            "preflight",
            f"provenance collection exceeded {PROVENANCE_TIMEOUT_SECONDS} seconds",
        )
    if not receiver.poll():
        receiver.close()
        fail_unknown(
            "analyzer_provenance",
            "preflight",
            f"provenance worker exited with status {process.exitcode} without a result",
        )
    kind, payload = receiver.recv()
    receiver.close()
    if kind == "ok":
        return payload
    if kind == "unknown":
        reason, profile, detail = payload
        fail_unknown(reason, profile, detail)
    fail_unknown("analyzer_provenance", "preflight", f"provenance worker failed: {payload}")


def read_bounded(path: Path, maximum: int, *, profile: str, kind: str) -> bytes:
    """Read at most one byte beyond a governed external-output limit."""
    try:
        with path.open("rb") as source:
            contents = source.read(maximum + 1)
    except OSError as error:
        fail_unknown("analyzer_diagnostic", profile, f"cannot read {kind}: {error}")
    if len(contents) > maximum:
        fail_unknown("analyzer_resource_limit", profile, f"{kind} exceeds {maximum} bytes")
    return contents


def limit_child_files() -> None:
    """Apply a hard per-file ceiling to the analyzer process tree."""
    resource.setrlimit(resource.RLIMIT_FSIZE, (MAX_DIAGNOSTIC_BYTES, MAX_DIAGNOSTIC_BYTES))


def parse_results(
    path: Path,
    expected: tuple[str, ...],
    profile: str,
) -> dict[str, dict[str, str]]:
    """Parse a complete Daisy CSV result set, failing closed on ambiguity."""
    rows: dict[str, dict[str, str]] = {}
    read_bounded(path, MAX_RESULT_BYTES, profile=profile, kind="result CSV")
    old_field_limit = csv.field_size_limit(MAX_FIELD_CHARS)
    try:
        try:
            with path.open(newline="", encoding="utf-8") as source:
                parsed_rows = csv.reader(source, delimiter=";")
                for row_number, row in enumerate(parsed_rows, start=1):
                    if row_number > MAX_RESULT_ROWS:
                        fail_unknown(
                            "analyzer_resource_limit",
                            profile,
                            f"result CSV exceeds {MAX_RESULT_ROWS} rows",
                        )
                    if not row or all(not field.strip() for field in row):
                        continue
                    if len(row) != 5:
                        fail_unknown(
                            "analyzer_diagnostic", profile, f"malformed result row: {row!r}"
                        )
                    function, absolute, relative, real_range, elapsed = (
                        field.strip() for field in row
                    )
                    if function not in expected:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"unexpected function result: {function!r}",
                        )
                    if function in rows:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"duplicate function result: {function!r}",
                        )

                    absolute_value = finite_decimal(absolute, "absolute error", profile)
                    if absolute_value < 0:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"negative absolute error: {absolute!r}",
                        )
                    if relative:
                        relative_value = finite_decimal(relative, "relative error", profile)
                        if relative_value < 0:
                            fail_unknown(
                                "analyzer_diagnostic",
                                profile,
                                f"negative relative error: {relative!r}",
                            )

                    if not (real_range.startswith("[") and real_range.endswith("]")):
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"malformed real range: {real_range!r}",
                        )
                    endpoints = real_range[1:-1].split(",")
                    if len(endpoints) != 2:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"malformed real range: {real_range!r}",
                        )
                    lower = finite_decimal(endpoints[0].strip(), "real-range lower bound", profile)
                    upper = finite_decimal(endpoints[1].strip(), "real-range upper bound", profile)
                    if lower > upper:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"reversed real range: {real_range!r}",
                        )
                    try:
                        elapsed_ms = int(elapsed)
                    except ValueError:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"invalid elapsed time: {elapsed!r}",
                        )
                    if elapsed_ms < 0:
                        fail_unknown(
                            "analyzer_diagnostic",
                            profile,
                            f"negative elapsed time: {elapsed!r}",
                        )

                    rows[function] = {
                        "absolute_error": absolute,
                        "relative_error": relative,
                        "real_range": real_range,
                    }
        except (OSError, UnicodeError, csv.Error) as error:
            fail_unknown("analyzer_diagnostic", profile, f"cannot parse result file: {error}")
    finally:
        csv.field_size_limit(old_field_limit)

    missing = sorted(set(expected) - rows.keys())
    if missing:
        fail_unknown("missing_result", profile, f"missing function results: {', '.join(missing)}")
    return {name: rows[name] for name in expected}


def run_profile(
    daisy_root: Path,
    spike_dir: Path,
    timeout: int,
    profile: str,
    expected: tuple[str, ...],
    extra_arguments: tuple[str, ...] = (),
) -> dict[str, dict[str, str]]:
    """Run one bounded Daisy profile and require complete parsed evidence."""
    output_dir = daisy_root / "output"
    if not output_dir.is_dir():
        fail_unknown(
            "analyzer_diagnostic", profile, f"missing Daisy output directory: {output_dir}"
        )

    result_path: Path | None = None
    diagnostic_path: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=output_dir,
            prefix="tiler-sound-accuracy-",
            suffix=".csv",
            delete=False,
        ) as result_file:
            result_path = Path(result_file.name)
        with tempfile.NamedTemporaryFile(
            dir=output_dir,
            prefix="tiler-sound-accuracy-diagnostic-",
            suffix=".log",
            delete=False,
        ) as diagnostic_file:
            diagnostic_path = Path(diagnostic_file.name)
    except OSError as error:
        if result_path is not None:
            result_path.unlink(missing_ok=True)
        fail_unknown("analyzer_diagnostic", profile, f"cannot create result file: {error}")

    arguments = (
        str(daisy_root / "daisy"),
        "--silent",
        "--no-stdout-print",
        f"--results-csv={result_path.name}",
        # Daisy's pinned MultiStringOption parser accepts colon-separated names
        # without brackets. Its help text shows brackets, but that revision
        # strips only an opening bracket and would retain the closing one.
        f"--functions={':'.join(expected)}",
        "--precision=Float32",
        "--analysis=dataflow",
        "--rangeMethod=interval",
        "--errorMethod=affine",
        *extra_arguments,
        str(spike_dir / "scalar_regions.scala"),
    )
    try:
        try:
            with diagnostic_path.open("wb") as diagnostic_sink:
                process = subprocess.Popen(
                    arguments,
                    cwd=daisy_root,
                    stdout=diagnostic_sink,
                    stderr=subprocess.STDOUT,
                    start_new_session=True,
                    preexec_fn=limit_child_files,
                )
        except (OSError, subprocess.SubprocessError) as error:
            fail_unknown("analyzer_diagnostic", profile, f"cannot start Daisy: {error}")

        try:
            process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
            output = read_bounded(
                diagnostic_path,
                MAX_DIAGNOSTIC_BYTES,
                profile=profile,
                kind="analyzer diagnostic",
            ).decode("utf-8", errors="replace")
            fail_unknown(
                "analyzer_timeout",
                profile,
                f"Daisy exceeded {timeout} seconds; partial output: {output[-500:]!r}",
            )

        output = read_bounded(
            diagnostic_path,
            MAX_DIAGNOSTIC_BYTES,
            profile=profile,
            kind="analyzer diagnostic",
        ).decode("utf-8", errors="replace")
        diagnostic = ANSI_ESCAPE.sub("", output).strip()
        if process.returncode != 0 or diagnostic:
            detail = diagnostic[-1000:] or f"Daisy exited with status {process.returncode}"
            fail_unknown("analyzer_diagnostic", profile, detail)
        return parse_results(result_path, expected, profile)
    finally:
        if result_path is not None:
            result_path.unlink(missing_ok=True)
        if diagnostic_path is not None:
            diagnostic_path.unlink(missing_ok=True)


def profiles(spike_dir: Path) -> tuple[tuple[str, tuple[str, ...], tuple[str, ...]], ...]:
    """Return the exact analyzer profiles and required function results."""
    return (
        (
            "dataflow-interval-affine-f32",
            (
                "affine_mix",
                "cancellation",
                "divide_sqrt",
                "explicit_fma",
                "relational_ratio",
                "reduce_left",
                "reduce_tree",
            ),
            (),
        ),
        (
            "dataflow-interval-affine-mixed-f16",
            ("materialized_f16",),
            (f"--mixed-precision={spike_dir / 'mixed-precision.txt'}",),
        ),
    )


def main(arguments: list[str] | None = None) -> int:
    """Run all profiles and emit either proved evidence or stable Unknown."""
    arguments = sys.argv[1:] if arguments is None else arguments
    if len(arguments) != 4:
        print(
            "usage: daisy_runner.py DAISY_ROOT SPIKE_DIR TIMEOUT_SECONDS REVISION",
            file=sys.stderr,
        )
        return 2
    daisy_root = Path(arguments[0]).resolve()
    spike_dir = Path(arguments[1]).resolve()
    revision = arguments[3]
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        print("Daisy revision must be a full lowercase Git object id", file=sys.stderr)
        return 7

    try:
        timeout = int(arguments[2])
    except ValueError:
        print("TILER_DAISY_TIMEOUT_SECONDS must be an integer from 1 through 3600", file=sys.stderr)
        return 6
    if not 1 <= timeout <= 3600:
        print("TILER_DAISY_TIMEOUT_SECONDS must be an integer from 1 through 3600", file=sys.stderr)
        return 6

    try:
        revision = checked_revision(daisy_root, revision)
        provenance = bounded_analyzer_provenance(daisy_root, spike_dir, revision)
        results = {}
        for name, functions, extra in profiles(spike_dir):
            results[name] = run_profile(daisy_root, spike_dir, timeout, name, functions, extra)
            checked_revision(daisy_root, revision)
            if bounded_analyzer_provenance(daisy_root, spike_dir, revision) != provenance:
                fail_unknown(
                    "analyzer_provenance",
                    name,
                    "analyzer identity changed during profile execution",
                )
    except Unknown as unknown:
        print(
            json.dumps(
                {
                    "status": "unknown",
                    "reason": unknown.reason,
                    "profile": unknown.profile,
                    "detail": unknown.detail,
                },
                sort_keys=True,
            ),
            file=sys.stderr,
        )
        return UNKNOWN_EXIT

    print(
        json.dumps(
            {
                "status": "proved",
                "analyzer": provenance,
                "timeout_seconds_per_profile": timeout,
                "profiles": results,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
