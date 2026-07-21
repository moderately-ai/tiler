#!/usr/bin/env python3
"""Bounded, fail-closed proc-macro environment experiment harness."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import platform
import re
import selectors
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent
FIXTURE = ROOT / "fixture"
EXPECTED_SELECTION = "targets = [macos, ios_device, ios_simulator]"
EXPECTED_PACKAGE = "tiler-environment-probe-consumer"
OBSERVED = (
    "HOST",
    "TARGET",
    "CARGO_BUILD_TARGET",
    "CARGO_CFG_TARGET_ARCH",
    "CARGO_CFG_TARGET_OS",
    "CARGO_CFG_TARGET_ENV",
    "CARGO_CFG_TARGET_FAMILY",
    "CARGO_MANIFEST_DIR",
    "CARGO_PKG_NAME",
    "OUT_DIR",
    "PROFILE",
    "OPT_LEVEL",
    "DEBUG",
    "RUSTC",
    "SDKROOT",
    "MACOSX_DEPLOYMENT_TARGET",
    "IPHONEOS_DEPLOYMENT_TARGET",
)
EXPECTED_ABSENT = frozenset(OBSERVED) - {"CARGO_MANIFEST_DIR", "CARGO_PKG_NAME"}
DEFAULT_TIMEOUT_SECONDS = 60
MAX_OUTPUT_BYTES = 1 << 20
HARNESS_DEADLINE: float | None = None
PROVENANCE_INPUTS = (
    Path("probe.py"),
    Path("run.sh"),
    Path("run-target.sh"),
    Path("run-family-cfg.sh"),
    Path("family_cfg_fallback.rs"),
    Path("family_cfg_required_fail.rs"),
    Path("fixture/Cargo.toml"),
    Path("fixture/Cargo.lock"),
    Path("fixture/consumer/Cargo.toml"),
    Path("fixture/consumer/src/lib.rs"),
    Path("fixture/probe-macro/Cargo.toml"),
    Path("fixture/probe-macro/src/lib.rs"),
)


class ProbeFailure(RuntimeError):
    """The experiment could not establish its declared success predicate."""


@dataclass(frozen=True)
class CommandResult:
    stdout: str
    stderr: str


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ProbeFailure(message)


def configured_timeout() -> int:
    raw = os.environ.get("TILER_PROBE_TIMEOUT_SECONDS", str(DEFAULT_TIMEOUT_SECONDS))
    try:
        value = int(raw)
    except ValueError as error:
        raise ProbeFailure("TILER_PROBE_TIMEOUT_SECONDS must be an integer") from error
    require(1 <= value <= 600, "TILER_PROBE_TIMEOUT_SECONDS must be from 1 through 600")
    return value


def overall_timeout_handler(_signum: int, _frame: object) -> None:
    """Interrupt Python-side work when the full harness deadline expires."""
    raise ProbeFailure("macro-environment harness exceeded its overall deadline")


def start_deadline() -> None:
    global HARNESS_DEADLINE
    timeout = configured_timeout()
    HARNESS_DEADLINE = time.monotonic() + timeout
    signal.signal(signal.SIGALRM, overall_timeout_handler)
    signal.setitimer(signal.ITIMER_REAL, timeout)


def remaining_timeout() -> float:
    if HARNESS_DEADLINE is None:
        return float(configured_timeout())
    remaining = HARNESS_DEADLINE - time.monotonic()
    require(remaining > 0, "macro-environment harness exceeded its overall deadline")
    return remaining


def run(
    command: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None
) -> CommandResult:
    returncode, result = capture(command, cwd=cwd, env=env)
    require(
        returncode == 0,
        f"command failed with status {returncode}: {command!r}\n{result.stderr[-2000:]}",
    )
    return result


def capture(
    command: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None
) -> tuple[int, CommandResult]:
    """Capture two bounded pipes under the harness's remaining wall deadline."""
    try:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
    except OSError as error:
        raise ProbeFailure(f"cannot execute {command!r}: {error}") from error
    require(process.stdout is not None and process.stderr is not None, "capture pipes are missing")
    streams = {process.stdout: bytearray(), process.stderr: bytearray()}
    selector = selectors.DefaultSelector()
    try:
        for stream in streams:
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ)
        while selector.get_map():
            try:
                events = selector.select(remaining_timeout())
            except ProbeFailure:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(process.pid, signal.SIGKILL)
                process.wait()
                raise ProbeFailure(f"command exceeded deadline: {command!r}") from None
            for key, _ in events:
                stream = key.fileobj
                chunk = os.read(stream.fileno(), 65536)
                if not chunk:
                    selector.unregister(stream)
                    continue
                buffer = streams[stream]
                if len(buffer) + len(chunk) > MAX_OUTPUT_BYTES:
                    with contextlib.suppress(ProcessLookupError):
                        os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                    raise ProbeFailure(
                        f"{('stdout' if stream is process.stdout else 'stderr')} exceeds "
                        f"{MAX_OUTPUT_BYTES} bytes: {command!r}"
                    )
                buffer.extend(chunk)
        try:
            returncode = process.wait(timeout=remaining_timeout())
        except subprocess.TimeoutExpired as error:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise ProbeFailure(f"command exceeded deadline: {command!r}") from error
    finally:
        if process.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        selector.close()
        process.stdout.close()
        process.stderr.close()
    return returncode, CommandResult(
        bytes(streams[process.stdout]).decode("utf-8", errors="replace"),
        bytes(streams[process.stderr]).decode("utf-8", errors="replace"),
    )


def run_allow_failure(command: list[str], *, cwd: Path | None = None) -> tuple[int, CommandResult]:
    return capture(command, cwd=cwd)


def decode_hex(value: str, field: str) -> str:
    require(len(value) % 2 == 0, f"{field} has odd-length hex")
    try:
        return bytes.fromhex(value).decode("utf-8")
    except (ValueError, UnicodeError) as error:
        raise ProbeFailure(f"{field} is not canonical UTF-8 hex") from error


def parse_trace_line(line: str) -> dict[str, object]:
    fields: dict[str, str] = {}
    for item in line.rstrip("\n").split("\t"):
        require("=" in item, f"malformed trace field: {item!r}")
        key, value = item.split("=", 1)
        require(key and key not in fields, f"duplicate or empty trace field: {key!r}")
        fields[key] = value
    require(fields.pop("version", None) == "1", "unsupported trace version")
    selection = decode_hex(fields.pop("selection_hex", ""), "selection")
    fingerprint = decode_hex(fields.pop("fingerprint_hex", ""), "fingerprint")
    cache = fields.pop("cache", "")
    require(cache in {"hit", "miss", "disabled"}, f"invalid cache state: {cache!r}")
    environment: dict[str, str | None] = {}
    for name in OBSERVED:
        hex_key = f"env.{name}.hex"
        absent_key = f"env.{name}.absent"
        nonunicode_key = f"env.{name}.nonunicode"
        present = [key for key in (hex_key, absent_key, nonunicode_key) if key in fields]
        require(len(present) == 1, f"environment field {name} must have exactly one state")
        key = present[0]
        value = fields.pop(key)
        if key == hex_key:
            environment[name] = decode_hex(value, name)
        elif key == absent_key:
            require(value == "1", f"invalid absent marker for {name}")
            environment[name] = None
        else:
            raise ProbeFailure(f"non-Unicode environment value for {name} is unsupported")
    require(not fields, f"unexpected trace fields: {sorted(fields)}")
    return {
        "selection": selection,
        "fingerprint": fingerprint,
        "cache": cache,
        "environment": environment,
    }


def parse_trace(path: Path) -> tuple[list[str], list[dict[str, object]]]:
    require(path.is_file(), f"macro trace was not produced: {path}")
    raw = path.read_bytes()
    require(len(raw) <= MAX_OUTPUT_BYTES, f"macro trace exceeds {MAX_OUTPUT_BYTES} bytes")
    try:
        lines = raw.decode("utf-8").splitlines()
    except UnicodeError as error:
        raise ProbeFailure("macro trace is not UTF-8") from error
    require(lines, "macro trace is empty")
    return lines, [parse_trace_line(line) for line in lines]


def validate_record(
    record: dict[str, object], *, fingerprint: str, cache: str, consumer_dir: Path
) -> None:
    require(record["selection"] == EXPECTED_SELECTION, "macro token selection changed")
    require(record["fingerprint"] == fingerprint, "unexpected toolchain fingerprint")
    require(record["cache"] == cache, "unexpected simulated cache attribution")
    environment = record["environment"]
    require(isinstance(environment, dict), "trace environment is malformed")
    require(environment["CARGO_MANIFEST_DIR"] == str(consumer_dir), "wrong consumer manifest")
    require(environment["CARGO_PKG_NAME"] == EXPECTED_PACKAGE, "wrong consumer package")
    for name in EXPECTED_ABSENT:
        require(environment[name] is None, f"expected {name} to be absent")


def trace_count(trace: Path) -> int:
    return len(parse_trace(trace)[1]) if trace.exists() else 0


def cargo(manifest: Path, environment: dict[str, str], *arguments: str) -> None:
    run(["cargo", *arguments, "--manifest-path", str(manifest), "--quiet"], env=environment)


def provenance() -> dict[str, object]:
    rustc = run(["rustc", "-vV"]).stdout.strip()
    cargo_version = run(["cargo", "-V"]).stdout.strip()
    revision = run(["git", "rev-parse", "HEAD"], cwd=ROOT).stdout.strip()
    return {
        "rustc_verbose": rustc,
        "cargo_version": cargo_version,
        "host": parse_host(rustc),
        "platform": platform.platform(),
        "repository_revision": revision,
        "input_sha256": input_digests(),
    }


def input_digests() -> dict[str, str]:
    return {
        str(relative): hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()
        for relative in PROVENANCE_INPUTS
    }


def parse_host(rustc_verbose: str) -> str:
    hosts = [
        line.removeprefix("host: ")
        for line in rustc_verbose.splitlines()
        if line.startswith("host: ")
    ]
    require(len(hosts) == 1 and hosts[0], "rustc -vV did not contain exactly one host")
    return hosts[0]


def write_result(result: dict[str, object], output: Path | None) -> None:
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if output is not None:
        output.parent.mkdir(parents=True, exist_ok=True)
        temporary = output.with_name(f".{output.name}.tmp-{os.getpid()}")
        temporary.write_text(encoded, encoding="utf-8")
        os.replace(temporary, output)
    print(encoded, end="")


def native_probe(output: Path | None) -> None:
    with tempfile.TemporaryDirectory(prefix="tiler-macro-env-") as scratch_text:
        scratch = Path(scratch_text)
        fixture = scratch / "fixture"
        shutil.copytree(FIXTURE, fixture)
        manifest = fixture / "Cargo.toml"
        consumer = fixture / "consumer/src/lib.rs"
        macro_source = fixture / "probe-macro/src/lib.rs"
        trace = scratch / "trace.log"
        cache = scratch / "cache"
        environment = os.environ.copy()
        environment.update(TILER_TRACE_PATH=str(trace), TILER_PROBE_CACHE=str(cache))
        steps: list[dict[str, object]] = []

        def execute(name: str, fingerprint: str, *cargo_arguments: str) -> None:
            before = trace_count(trace)
            environment["TILER_TOOLCHAIN_FINGERPRINT"] = fingerprint
            cargo(manifest, environment, *cargo_arguments)
            after = trace_count(trace)
            steps.append({"name": name, "before": before, "after": after})

        execute("cold_check", "xcode-a", "check")
        execute("no_change_check", "xcode-a", "check")
        execute("fingerprint_environment_only", "xcode-b", "check")
        consumer.write_text(consumer.read_text() + "\n// unrelated consumer edit\n")
        execute("consumer_edit_after_fingerprint", "xcode-b", "check")
        shutil.rmtree(cache)
        execute("cache_deletion_only", "xcode-b", "check")
        consumer.write_text(consumer.read_text() + "\n// second unrelated consumer edit\n")
        execute("consumer_edit_after_cache_delete", "xcode-b", "check")
        macro_source.write_text(macro_source.read_text() + "\n// macro crate edit\n")
        execute("macro_crate_edit", "xcode-b", "check")
        execute("cargo_test", "xcode-b", "test")

        expected_counts = [1, 1, 1, 2, 2, 3, 4, 7]
        require(
            [step["after"] for step in steps] == expected_counts, "expansion count matrix changed"
        )
        raw_lines, records = parse_trace(trace)
        expectations = [
            ("xcode-a", "miss"),
            ("xcode-b", "miss"),
            ("xcode-b", "miss"),
            ("xcode-b", "hit"),
            ("xcode-b", "hit"),
            ("xcode-b", "hit"),
            ("xcode-b", "hit"),
        ]
        require(len(records) == len(expectations), "unexpected final expansion count")
        for record, (fingerprint, cache_state) in zip(records, expectations, strict=True):
            validate_record(
                record,
                fingerprint=fingerprint,
                cache=cache_state,
                consumer_dir=fixture / "consumer",
            )
        write_result(
            {
                "schema": "tiler-macro-environment/v1",
                "probe": "native-freshness",
                "provenance": provenance(),
                "steps": steps,
                "trace": records,
                "raw_trace": raw_lines,
                "success": True,
            },
            output,
        )


def target_probe(target: str, output: Path | None) -> None:
    details = provenance()
    host = str(details["host"])
    require(target != host, f"requested target must differ from host target {host}")
    installed = set(run(["rustup", "target", "list", "--installed"]).stdout.splitlines())
    require(
        target in installed,
        f"requested target is not installed: {target}; installed={sorted(installed)}",
    )
    with tempfile.TemporaryDirectory(prefix="tiler-macro-target-") as scratch_text:
        scratch = Path(scratch_text)
        fixture = scratch / "fixture"
        shutil.copytree(FIXTURE, fixture)
        trace = scratch / "trace.log"
        environment = os.environ.copy()
        environment.update(
            TILER_TRACE_PATH=str(trace),
            TILER_PROBE_CACHE=str(scratch / "cache"),
            TILER_TOOLCHAIN_FINGERPRINT="target-probe",
        )
        cargo(fixture / "Cargo.toml", environment, "check", "--target", target)
        raw_lines, records = parse_trace(trace)
        require(len(records) == 1, "cross-target check must expand exactly once")
        validate_record(
            records[0],
            fingerprint="target-probe",
            cache="miss",
            consumer_dir=fixture / "consumer",
        )
        write_result(
            {
                "schema": "tiler-macro-environment/v1",
                "probe": "distinct-target",
                "provenance": details,
                "requested_target": target,
                "trace": records,
                "raw_trace": raw_lines,
                "success": True,
            },
            output,
        )


def parse_cfg(output: str) -> set[str]:
    lines = output.splitlines()
    require(
        lines and all(line and line.strip() == line for line in lines), "malformed rustc cfg output"
    )
    require(len(lines) == len(set(lines)), "duplicate rustc cfg output")
    return set(lines)


def cfg_evidence(output: str, *, target_os: str, target_abi: str) -> dict[str, object]:
    """Retain raw cfg and require one exact OS/ABI classification."""
    cfg = parse_cfg(output)
    operating_systems = sorted(item for item in cfg if item.startswith('target_os="'))
    abis = sorted(item for item in cfg if item.startswith('target_abi="'))
    require(len(operating_systems) == 1, "target cfg must contain exactly one target_os")
    require(len(abis) == 1, "target cfg must contain exactly one target_abi")
    require(operating_systems[0] == target_os, f"unexpected target_os: {operating_systems[0]}")
    require(abis[0] == target_abi, f"unexpected target_abi: {abis[0]}")
    return {
        "raw_cfg": output.splitlines(),
        "target_os": operating_systems[0],
        "target_abi": abis[0],
    }


def host_cfg_expectation(host: str) -> tuple[str, str]:
    """Map the repository's supported host families to exact Rust cfg values."""
    if host.endswith("-apple-darwin"):
        return ('target_os="macos"', 'target_abi=""')
    if "-unknown-linux-gnu" in host:
        return ('target_os="linux"', 'target_abi=""')
    raise ProbeFailure(f"unsupported host triple for family cfg evidence: {host}")


def family_cfg_probe(output: Path | None) -> None:
    details = provenance()
    expected_host_os, expected_host_abi = host_cfg_expectation(details["host"])
    host_cfg = cfg_evidence(
        run(["rustc", "--print", "cfg"]).stdout,
        target_os=expected_host_os,
        target_abi=expected_host_abi,
    )
    host_is_macos = host_cfg["target_os"] == 'target_os="macos"'
    with tempfile.TemporaryDirectory(prefix="tiler-family-cfg-") as scratch_text:
        scratch = Path(scratch_text)
        fallback = scratch / "fallback"
        required = scratch / "required"
        run(
            [
                "rustc",
                "--edition",
                "2021",
                "-D",
                "warnings",
                str(ROOT / "family_cfg_fallback.rs"),
                "-o",
                str(fallback),
            ]
        )
        run([str(fallback)])
        status, required_result = run_allow_failure(
            [
                "rustc",
                "--edition",
                "2021",
                "-D",
                "warnings",
                str(ROOT / "family_cfg_required_fail.rs"),
                "-o",
                str(required),
            ]
        )
        diagnostic = "selected macOS artifact family could not be built"
        if host_is_macos:
            require(status != 0, "matching macOS family compile unexpectedly succeeded")
            require(
                diagnostic in required_result.stderr, "matching family diagnostic was not retained"
            )
        else:
            require(status == 0, "nonmatching macOS family compile unexpectedly failed")
            require(
                diagnostic not in required_result.stderr,
                "nonmatching family emitted macOS diagnostic",
            )
            run([str(required)])

        expected = {
            "aarch64-apple-darwin": ('target_os="macos"', 'target_abi=""'),
            "aarch64-apple-ios": ('target_os="ios"', 'target_abi=""'),
            "aarch64-apple-ios-sim": ('target_os="ios"', 'target_abi="sim"'),
            "aarch64-apple-ios-macabi": ('target_os="ios"', 'target_abi="macabi"'),
        }
        observed: dict[str, dict[str, object]] = {}
        for target, (expected_os, expected_abi) in expected.items():
            cfg_output = run(["rustc", "--print", "cfg", "--target", target]).stdout
            observed[target] = cfg_evidence(
                cfg_output,
                target_os=expected_os,
                target_abi=expected_abi,
            )
        write_result(
            {
                "schema": "tiler-macro-environment/v1",
                "probe": "family-cfg",
                "provenance": details,
                "host_cfg": host_cfg,
                "matching_family_diagnostic": host_is_macos,
                "required_compile_status": status,
                "required_compile_stderr": required_result.stderr,
                "target_predicates": observed,
                "success": True,
            },
            output,
        )


def verify_result(path: Path) -> None:
    try:
        result = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ProbeFailure(f"cannot parse retained result {path}: {error}") from error
    require(isinstance(result, dict), "retained result must be a JSON object")
    require(result.get("schema") == "tiler-macro-environment/v1", "wrong result schema")
    require(result.get("success") is True, "retained result is not successful evidence")
    recorded_provenance = result.get("provenance")
    require(isinstance(recorded_provenance, dict), "retained provenance is missing")
    rustc_verbose = recorded_provenance.get("rustc_verbose")
    require(isinstance(rustc_verbose, str) and rustc_verbose, "rustc provenance is missing")
    require(
        recorded_provenance.get("host") == parse_host(rustc_verbose),
        "rustc host provenance is inconsistent",
    )
    cargo_version = recorded_provenance.get("cargo_version")
    require(
        isinstance(cargo_version, str) and cargo_version.startswith("cargo "),
        "Cargo provenance is missing",
    )
    require(
        isinstance(recorded_provenance.get("platform"), str)
        and bool(recorded_provenance["platform"]),
        "platform provenance is missing",
    )
    require(
        isinstance(recorded_provenance.get("repository_revision"), str)
        and bool(re.fullmatch(r"[0-9a-f]{40}", recorded_provenance["repository_revision"])),
        "repository revision provenance is invalid",
    )
    require(
        recorded_provenance.get("input_sha256") == input_digests(),
        "retained result does not bind the current harness inputs",
    )
    probe = result.get("probe")
    if probe == "native-freshness":
        raw_trace = result.get("raw_trace")
        trace = result.get("trace")
        steps = result.get("steps")
        require(
            isinstance(raw_trace, list) and all(isinstance(line, str) for line in raw_trace),
            "retained raw trace is malformed",
        )
        parsed = [parse_trace_line(line) for line in raw_trace]
        require(parsed == trace, "decoded trace does not match retained raw trace")
        expected_steps = [
            {"name": "cold_check", "before": 0, "after": 1},
            {"name": "no_change_check", "before": 1, "after": 1},
            {"name": "fingerprint_environment_only", "before": 1, "after": 1},
            {"name": "consumer_edit_after_fingerprint", "before": 1, "after": 2},
            {"name": "cache_deletion_only", "before": 2, "after": 2},
            {"name": "consumer_edit_after_cache_delete", "before": 2, "after": 3},
            {"name": "macro_crate_edit", "before": 3, "after": 4},
            {"name": "cargo_test", "before": 4, "after": 7},
        ]
        require(steps == expected_steps, "retained expansion step matrix changed")
        require(len(parsed) == 7, "retained native trace must contain seven expansions")
        manifest = parsed[0]["environment"]["CARGO_MANIFEST_DIR"]
        require(isinstance(manifest, str), "retained consumer manifest is missing")
        expectations = [
            ("xcode-a", "miss"),
            ("xcode-b", "miss"),
            ("xcode-b", "miss"),
            ("xcode-b", "hit"),
            ("xcode-b", "hit"),
            ("xcode-b", "hit"),
            ("xcode-b", "hit"),
        ]
        for record, (fingerprint, cache_state) in zip(parsed, expectations, strict=True):
            validate_record(
                record,
                fingerprint=fingerprint,
                cache=cache_state,
                consumer_dir=Path(manifest),
            )
    elif probe == "family-cfg":
        expected = {
            "aarch64-apple-darwin": ('target_os="macos"', 'target_abi=""'),
            "aarch64-apple-ios": ('target_os="ios"', 'target_abi=""'),
            "aarch64-apple-ios-macabi": ('target_os="ios"', 'target_abi="macabi"'),
            "aarch64-apple-ios-sim": ('target_os="ios"', 'target_abi="sim"'),
        }
        retained_predicates = result.get("target_predicates")
        require(
            isinstance(retained_predicates, dict) and set(retained_predicates) == set(expected),
            "retained target cfg set changed",
        )
        for target, (expected_os, expected_abi) in expected.items():
            evidence = retained_predicates[target]
            require(isinstance(evidence, dict), f"retained cfg evidence is malformed: {target}")
            raw_cfg = evidence.get("raw_cfg")
            require(
                isinstance(raw_cfg, list)
                and raw_cfg
                and all(isinstance(line, str) for line in raw_cfg),
                f"retained raw cfg is malformed: {target}",
            )
            require(
                evidence
                == cfg_evidence(
                    "\n".join(raw_cfg),
                    target_os=expected_os,
                    target_abi=expected_abi,
                ),
                f"retained decoded cfg does not match raw cfg: {target}",
            )
        host = recorded_provenance["host"]
        expected_host_os, expected_host_abi = host_cfg_expectation(host)
        retained_host_cfg = result.get("host_cfg")
        require(isinstance(retained_host_cfg, dict), "retained raw host cfg is missing")
        raw_host_cfg = retained_host_cfg.get("raw_cfg")
        require(
            isinstance(raw_host_cfg, list)
            and raw_host_cfg
            and all(isinstance(line, str) for line in raw_host_cfg),
            "retained raw host cfg is malformed",
        )
        require(
            retained_host_cfg
            == cfg_evidence(
                "\n".join(raw_host_cfg),
                target_os=expected_host_os,
                target_abi=expected_host_abi,
            ),
            "retained decoded host cfg does not match raw host cfg",
        )
        status = result.get("required_compile_status")
        diagnostic = "selected macOS artifact family could not be built"
        matching_family = result.get("matching_family_diagnostic")
        require(
            type(matching_family) is bool
            and matching_family == (retained_host_cfg["target_os"] == 'target_os="macos"'),
            "family diagnostic branch contradicts host target_os",
        )
        if matching_family is True:
            require(type(status) is int and status != 0, "matching family did not fail")
            require(
                diagnostic in result.get("required_compile_stderr", ""),
                "matching-family diagnostic is missing",
            )
        else:
            require(type(status) is int and status == 0, "nonmatching family did not compile")
            require(
                diagnostic not in result.get("required_compile_stderr", ""),
                "nonmatching family emitted the diagnostic",
            )
    else:
        raise ProbeFailure(f"unsupported retained probe result: {probe!r}")
    print(f"verified retained macro-environment result: {path}")


def parse_arguments(arguments: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="probe", required=True)
    for name in ("native", "family-cfg"):
        child = subparsers.add_parser(name)
        child.add_argument("--output", type=Path)
    target = subparsers.add_parser("target")
    target.add_argument("target")
    target.add_argument("--output", type=Path)
    verify = subparsers.add_parser("verify")
    verify.add_argument("result", type=Path)
    return parser.parse_args(arguments)


def main(arguments: list[str] | None = None) -> int:
    options = parse_arguments(sys.argv[1:] if arguments is None else arguments)
    try:
        start_deadline()
        if options.probe == "native":
            native_probe(options.output)
        elif options.probe == "target":
            target_probe(options.target, options.output)
        elif options.probe == "family-cfg":
            family_cfg_probe(options.output)
        else:
            verify_result(options.result)
    except ProbeFailure as error:
        print(f"macro environment probe failed: {error}", file=sys.stderr)
        return 1
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
