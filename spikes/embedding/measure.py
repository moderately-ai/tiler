#!/usr/bin/env python3
"""Build deterministic proc-macro byte-literal fixtures and record cost data.

The dependency-free stable proc macro emits the literal tokens directly. It
does not use include_bytes!, because that bypasses the token representation
whose cost this spike measures.
"""

from __future__ import annotations

import argparse
import contextlib
import csv
import hashlib
import json
import math
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
from dataclasses import asdict, dataclass
from pathlib import Path

DEFAULT_COMMAND_TIMEOUT_SECONDS = 600
DEFAULT_OVERALL_TIMEOUT_SECONDS = 3600
METADATA_COMMAND_TIMEOUT_SECONDS = 30
MAX_CAPTURE_BYTES = 16 << 20
HARNESS_SCHEMA_VERSION = 2
HARNESS_DEADLINE: float | None = None


class MeasurementFailure(RuntimeError):
    """The harness could not produce a complete, bounded measurement."""


def require(condition: bool, message: str) -> None:
    """Reject incomplete evidence without relying on removable assertions."""
    if not condition:
        raise MeasurementFailure(message)


def overall_timeout_handler(_signum: int, _frame: object) -> None:
    """Interrupt non-subprocess harness work at the overall wall deadline."""
    raise MeasurementFailure("embedding harness exceeded its overall deadline")


@dataclass(frozen=True)
class Case:
    name: str
    size: int
    count: int
    representation: str
    identity: str
    boundary: str
    profile: str
    codegen_units: int = 16
    lto: str = "off"


def decision_cases() -> list[Case]:
    cases: dict[str, Case] = {}

    def add(**kwargs: object) -> None:
        case = Case(**kwargs)  # type: ignore[arg-type]
        cases[case.name] = case

    # Literal-token scaling and the deliberately worse per-byte-token control.
    for size in (10 * 1024, 100 * 1024, 1024 * 1024):
        for representation in ("byte-string", "per-byte"):
            add(
                name=f"repr-{representation}-{size}",
                size=size,
                count=1,
                representation=representation,
                identity="unique",
                boundary="same",
                profile="release",
            )
        add(
            name=f"debug-byte-string-{size}",
            size=size,
            count=1,
            representation="byte-string",
            identity="unique",
            boundary="same",
            profile="dev",
        )

    # Multiplicity: enough to reveal both linear growth and any identical folding.
    for count in (8, 32):
        for identity in ("identical", "unique"):
            add(
                name=f"count-{count}-{identity}",
                size=100 * 1024,
                count=count,
                representation="byte-string",
                identity=identity,
                boundary="same",
                profile="release",
            )

    # Crate boundary and profile are crossed for the central 8 x 100 KiB case.
    for boundary in ("same", "cross"):
        for profile in ("dev", "release"):
            for identity in ("identical", "unique"):
                add(
                    name=f"boundary-{boundary}-{profile}-{identity}",
                    size=100 * 1024,
                    count=8,
                    representation="byte-string",
                    identity=identity,
                    boundary=boundary,
                    profile=profile,
                )

    # Keep payload constant while varying release linker/codegen settings.
    for boundary in ("same", "cross"):
        for codegen_units, lto in ((1, "off"), (16, "thin"), (1, "fat")):
            add(
                name=f"config-{boundary}-cgu{codegen_units}-{lto}",
                size=100 * 1024,
                count=8,
                representation="byte-string",
                identity="identical",
                boundary=boundary,
                profile="release",
                codegen_units=codegen_units,
                lto=lto,
            )
    return list(cases.values())


def smoke_cases() -> list[Case]:
    return [
        Case("smoke-byte-string", 10 * 1024, 1, "byte-string", "unique", "same", "release"),
        Case("smoke-per-byte", 10 * 1024, 1, "per-byte", "unique", "same", "release"),
        Case("smoke-duplicate", 10 * 1024, 2, "byte-string", "identical", "cross", "dev"),
    ]


def artifact_bytes(size: int, index: int, identity: str) -> bytes:
    seed = 0xD1B54A32D192ED03 if identity == "identical" else 0xD1B54A32D192ED03 ^ index
    out = bytearray()
    state = seed
    while len(out) < size:
        state ^= state >> 12
        state ^= (state << 25) & ((1 << 64) - 1)
        state ^= state >> 27
        word = (state * 0x2545F4914F6CDD1D) & ((1 << 64) - 1)
        out.extend(word.to_bytes(8, "little"))
    return bytes(out[:size])


def rust_literal(data: bytes, representation: str) -> str:
    if representation == "byte-string":
        return 'b"' + "".join(f"\\x{byte:02x}" for byte in data) + '"'
    if representation == "per-byte":
        return "&[" + ",".join(f"0x{byte:02x}u8" for byte in data) + "]"
    raise ValueError(representation)


def package_toml(name: str) -> str:
    return f'[package]\nname = "{name}"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n'


def library_source(size: int, index: int, identity: str, representation: str) -> str:
    representation_ident = representation.replace("-", "_")
    return (
        "use embed_macro::embed;\n"
        "#[inline(never)]\n"
        "pub fn artifact() -> &'static [u8] {\n"
        f"    embed!({size}, {index}, {identity}, {representation_ident})\n"
        "}\n"
    )


def main_source_same(case: Case) -> str:
    modules = []
    calls = []
    for index in range(case.count):
        source = library_source(case.size, index, case.identity, case.representation)
        modules.append(f"mod blob_{index} {{\n{source}}}\n")
        calls.append(f"blob_{index}::artifact()")
    return "".join(modules) + main_body(calls)


def macro_source() -> str:
    return r"""use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};

fn artifact_bytes(size: usize, index: u64, identical: bool) -> Vec<u8> {
    let mut state = 0xd1b54a32d192ed03u64 ^ if identical { 0 } else { index };
    let mut out = Vec::with_capacity(size);
    while out.len() < size {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        for byte in state.wrapping_mul(0x2545f4914f6cdd1d).to_le_bytes() {
            if out.len() == size { break; }
            out.push(byte);
        }
    }
    out
}

#[proc_macro]
pub fn embed(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let fields: Vec<_> = input.split(',').map(str::trim).collect();
    assert_eq!(fields.len(), 4, "expected size, index, identity, representation");
    let size: usize = fields[0].parse().expect("size must be an integer");
    let index: u64 = fields[1].parse().expect("index must be an integer");
    let bytes = artifact_bytes(size, index, fields[2] == "identical");
    match fields[3] {
        "byte_string" => TokenStream::from(TokenTree::Literal(Literal::byte_string(&bytes))),
        "per_byte" => {
            let mut elements = TokenStream::new();
            for byte in bytes {
                elements.extend([TokenTree::Literal(Literal::u8_unsuffixed(byte))]);
                elements.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
            }
            TokenStream::from_iter([
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Bracket, elements)),
            ])
        }
        other => panic!("unknown representation {other}"),
    }
}
"""


def main_body(calls: list[str]) -> str:
    return (
        "#[inline(never)]\n"
        "fn witness(bytes: &[u8]) -> u64 {\n"
        "    let mut value = 0xcbf29ce484222325u64;\n"
        "    for index in 0..bytes.len() {\n"
        "        let byte = unsafe { core::ptr::read_volatile(bytes.as_ptr().add(index)) };\n"
        "        value = value.wrapping_mul(0x100000001b3) ^ u64::from(byte);\n"
        "    }\n"
        "    value\n"
        "}\n"
        "fn main() {\n"
        f"    let artifacts: [&'static [u8]; {len(calls)}] = [{', '.join(calls)}];\n"
        "    let digest = artifacts.iter().fold(0u64, |acc, bytes| acc ^ witness(bytes));\n"
        '    println!("{}:{}", artifacts.len(), digest);\n'
        "}\n"
    )


def make_workspace(root: Path, case: Case) -> list[bytes]:
    payloads = [
        artifact_bytes(case.size, 0 if case.identity == "identical" else i, case.identity)
        for i in range(case.count)
    ]
    app = root / "app"
    (app / "src").mkdir(parents=True)
    macro_crate = root / "embed_macro"
    (macro_crate / "src").mkdir(parents=True)
    (macro_crate / "Cargo.toml").write_text(
        package_toml("embed_macro") + "\n[lib]\nproc-macro = true\n", encoding="utf-8"
    )
    (macro_crate / "src/lib.rs").write_text(macro_source(), encoding="utf-8")
    if case.boundary == "same":
        members = ["app", "embed_macro"]
        deps = '\n[dependencies]\nembed_macro = { path = "../embed_macro" }\n'
        source = main_source_same(case)
    else:
        members = ["app", "embed_macro"] + [f"blob_{i}" for i in range(case.count)]
        dep_lines = []
        calls = []
        for index, _data in enumerate(payloads):
            crate = root / f"blob_{index}"
            (crate / "src").mkdir(parents=True)
            (crate / "src/lib.rs").write_text(
                library_source(case.size, index, case.identity, case.representation),
                encoding="utf-8",
            )
            (crate / "Cargo.toml").write_text(
                package_toml(f"embed_blob_{index}")
                + '\n[dependencies]\nembed_macro = { path = "../embed_macro" }\n',
                encoding="utf-8",
            )
            dep_lines.append(f'embed_blob_{index} = {{ path = "../blob_{index}" }}')
            calls.append(f"embed_blob_{index}::artifact()")
        deps = "\n[dependencies]\n" + "\n".join(dep_lines) + "\n"
        source = main_body(calls)
    (app / "Cargo.toml").write_text(package_toml("embed_app") + deps, encoding="utf-8")
    (app / "src/main.rs").write_text(source, encoding="utf-8")
    lto_value = "false" if case.lto == "off" else f'"{case.lto}"'
    manifest = (
        '[workspace]\nresolver = "2"\n'
        f"members = {json.dumps(members)}\n\n"
        "[profile.dev]\ndebug = 2\nincremental = true\n"
        f"codegen-units = {case.codegen_units}\n\n"
        "[profile.release]\ndebug = 0\nincremental = false\n"
        f"codegen-units = {case.codegen_units}\nlto = {lto_value}\n"
    )
    (root / "Cargo.toml").write_text(manifest, encoding="utf-8")
    return payloads


def read_bounded(path: Path, label: str) -> str:
    """Read one retained tool output under the experiment's evidence limit."""
    try:
        size = path.stat().st_size
    except OSError as error:
        raise MeasurementFailure(f"missing {label}: {path}: {error}") from error
    require(size <= MAX_CAPTURE_BYTES, f"{label} exceeds {MAX_CAPTURE_BYTES} bytes: {path}")
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise MeasurementFailure(f"cannot decode {label}: {path}: {error}") from error


def run_logged(
    command: list[str],
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
    *,
    cwd: Path | None = None,
) -> int:
    """Run a command tree under one deadline while retaining its exact output."""
    started = time.monotonic()
    deadline = started + timeout_seconds
    if HARNESS_DEADLINE is not None:
        deadline = min(deadline, HARNESS_DEADLINE)
    require(deadline > started, f"overall harness deadline expired before {command!r}")
    try:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
    except OSError as error:
        raise MeasurementFailure(f"cannot start {command[0]}: {error}") from error
    require(process.stdout is not None and process.stderr is not None, "capture pipes are missing")
    selector = selectors.DefaultSelector()
    total = 0
    with stdout_path.open("wb") as stdout_file, stderr_path.open("wb") as stderr_file:
        outputs = {process.stdout: stdout_file, process.stderr: stderr_file}
        for stream in outputs:
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ)
        try:
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    with contextlib.suppress(ProcessLookupError):
                        os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                    raise MeasurementFailure(
                        f"command exceeded {timeout_seconds}s after "
                        f"{time.monotonic() - started:.3f}s: {command!r}"
                    )
                for key, _ in selector.select(remaining):
                    stream = key.fileobj
                    chunk = os.read(stream.fileno(), 65536)
                    if not chunk:
                        selector.unregister(stream)
                        continue
                    if total + len(chunk) > MAX_CAPTURE_BYTES:
                        with contextlib.suppress(ProcessLookupError):
                            os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
                        raise MeasurementFailure(
                            f"command output exceeded {MAX_CAPTURE_BYTES} bytes: {command!r}"
                        )
                    outputs[stream].write(chunk)
                    total += len(chunk)
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise subprocess.TimeoutExpired(command, timeout_seconds)
            returncode = process.wait(timeout=remaining)
        except subprocess.TimeoutExpired as error:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise MeasurementFailure(
                f"command exceeded {timeout_seconds}s after {time.monotonic() - started:.3f}s: "
                f"{command!r}"
            ) from error
        finally:
            if process.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            selector.close()
            process.stdout.close()
            process.stderr.close()
    require(
        stdout_path.stat().st_size + stderr_path.stat().st_size <= MAX_CAPTURE_BYTES,
        f"command output exceeds {MAX_CAPTURE_BYTES} bytes: {command!r}",
    )
    read_bounded(stdout_path, "command stdout")
    read_bounded(stderr_path, "command stderr")
    return returncode


def parse_time(path: Path) -> tuple[float, int]:
    """Parse both required `/usr/bin/time -l` metrics exactly once."""
    text = read_bounded(path, "time output")
    wall_match = re.search(r"\s([0-9.]+) real\s", text)
    rss_match = re.search(r"^\s*([0-9]+)\s+maximum resident set size", text, re.MULTILINE)
    require(wall_match is not None, f"missing wall-clock metric in {path}")
    require(rss_match is not None, f"missing peak-RSS metric in {path}")
    wall = float(wall_match.group(1))
    rss = int(rss_match.group(1))
    require(math.isfinite(wall) and wall >= 0, f"invalid wall-clock metric in {path}")
    require(rss > 0, f"invalid peak-RSS metric in {path}")
    return wall, rss


def file_sum(root: Path, suffix: str) -> int:
    return sum(path.stat().st_size for path in root.rglob(f"*{suffix}") if path.is_file())


def parse_macho_sections(text: str) -> tuple[int, int]:
    """Require a recognized Mach-O report and its primary const section."""
    values: dict[str, int] = {}
    current_segment = ""
    segments = 0
    sections = 0
    for line in text.splitlines():
        segment = re.match(r"Segment (\S+): (\d+)", line.strip())
        if segment:
            current_segment = segment.group(1)
            segments += 1
            continue
        section = re.match(r"Section (\S+): (\d+)", line.strip())
        if section:
            require(bool(current_segment), "Mach-O section appeared before a segment")
            values[f"{current_segment},{section.group(1)}"] = int(section.group(2))
            sections += 1
    require(segments > 0 and sections > 0, "unparseable `size -m` Mach-O output")
    require("__TEXT,__const" in values, "missing required __TEXT,__const metric")
    text_const = sum(
        value
        for key, value in values.items()
        if key.startswith("__TEXT,") and key.endswith(",__const")
    )
    data_const = sum(
        value
        for key, value in values.items()
        if key.startswith("__DATA") and key.endswith(",__const")
    )
    return text_const, data_const


def macho_sections(binary: Path, raw_path: Path, timeout_seconds: int) -> tuple[int, int]:
    stderr_path = raw_path.with_suffix(".stderr")
    returncode = run_logged(["size", "-m", str(binary)], raw_path, stderr_path, timeout_seconds)
    require(returncode == 0, f"size -m failed with status {returncode}: {binary}")
    return parse_macho_sections(read_bounded(raw_path, "size -m output"))


def build_command(root: Path, case: Case, target: Path, timing: Path) -> list[str]:
    return [
        "/usr/bin/time",
        "-l",
        "-o",
        str(timing),
        "cargo",
        "build",
        "--offline",
        "--manifest-path",
        str(root / "Cargo.toml"),
        "--target-dir",
        str(target),
        "--profile",
        case.profile,
    ]


def source_identity(root: Path) -> dict[str, object]:
    """Identify every generated manifest and Rust source that enters Cargo."""
    digest = hashlib.sha256()
    files = sorted(
        path
        for path in root.rglob("*")
        if path.is_file()
        and "target" not in path.relative_to(root).parts
        and (path.name == "Cargo.toml" or path.suffix == ".rs")
    )
    records = []
    for path in files:
        relative = path.relative_to(root).as_posix()
        contents = path.read_bytes()
        file_digest = hashlib.sha256(contents).hexdigest()
        records.append({"path": relative, "bytes": len(contents), "sha256": file_digest})
        digest.update(relative.encode())
        digest.update(b"\0")
        digest.update(bytes.fromhex(file_digest))
    require(bool(records), f"generated workspace has no source inputs: {root}")
    return {"sha256": digest.hexdigest(), "files": records}


def payload_identity(payloads: list[bytes]) -> dict[str, object]:
    digests = [hashlib.sha256(payload).hexdigest() for payload in payloads]
    return {"sha256": hashlib.sha256("\n".join(digests).encode()).hexdigest(), "digests": digests}


def run_build(
    root: Path,
    case: Case,
    payloads: list[bytes],
    raw: Path,
    repetition: int,
    timeout_seconds: int,
) -> dict[str, object]:
    target = root / "target"
    timing = root / "time.txt"
    command = build_command(root, case, target, timing)
    source_inputs = source_identity(root)
    label = f"{case.name}-r{repetition}"
    returncode = run_logged(
        command,
        raw / f"{label}.stdout",
        raw / f"{label}.stderr",
        timeout_seconds,
        cwd=root,
    )
    if returncode != 0:
        raise MeasurementFailure(f"{case.name} failed ({returncode}); see raw stderr")
    wall, rss = parse_time(timing)
    binary = target / ("debug" if case.profile == "dev" else "release") / "embed_app"
    require(binary.is_file(), f"missing linked binary: {binary}")
    binary_bytes = binary.read_bytes()
    occurrences = [binary_bytes.count(payload) for payload in payloads]
    text_const, data_const = macho_sections(binary, raw / f"{label}.size-m.txt", timeout_seconds)
    source_bytes = sum(path.stat().st_size for path in root.rglob("*.rs"))
    result = asdict(case)
    result.update(
        {
            "command": command,
            "repetition": repetition,
            "wall_seconds": wall,
            "peak_rss_bytes": rss,
            "source_identity": source_inputs,
            "payload_identity": payload_identity(payloads),
            "source_bytes": source_bytes,
            "logical_payload_bytes": case.size * case.count,
            "byte_value_literal_tokens": case.count
            if case.representation == "byte-string"
            else case.size * case.count,
            "expanded_text_proxy_bytes": sum(
                len(rust_literal(payload, case.representation)) for payload in payloads
            ),
            "rlib_bytes": file_sum(target, ".rlib"),
            "rmeta_bytes": file_sum(target, ".rmeta"),
            "object_bytes": file_sum(target, ".o"),
            "target_tree_bytes": sum(p.stat().st_size for p in target.rglob("*") if p.is_file()),
            "final_binary_bytes": binary.stat().st_size,
            "macho_text_const_bytes": text_const,
            "macho_data_const_bytes": data_const,
            "payload_occurrences_min": min(occurrences),
            "payload_occurrences_max": max(occurrences),
            "distinct_payloads": len(set(payloads)),
            "binary_sha256": hashlib.sha256(binary_bytes).hexdigest(),
        }
    )
    return result


def freshness_case(
    output: Path, raw: Path, boundary: str, timeout_seconds: int
) -> list[dict[str, object]]:
    case = Case(f"freshness-{boundary}", 100 * 1024, 8, "byte-string", "identical", boundary, "dev")
    root = output / "work" / case.name
    root.mkdir(parents=True)
    payloads = make_workspace(root, case)
    target = root / "target"
    rows = []

    def phase(name: str) -> None:
        timing = root / f"time-{name}.txt"
        command = build_command(root, case, target, timing)
        returncode = run_logged(
            command,
            raw / f"{case.name}-{name}.stdout",
            raw / f"{case.name}-{name}.stderr",
            timeout_seconds,
            cwd=root,
        )
        if returncode != 0:
            raise MeasurementFailure(f"{case.name}/{name} failed ({returncode}); see raw stderr")
        wall, rss = parse_time(timing)
        rows.append(
            {
                "boundary": boundary,
                "phase": name,
                "wall_seconds": wall,
                "peak_rss_bytes": rss,
                "command": command,
                "source_identity": source_identity(root),
                "payload_identity": payload_identity(payloads),
                "target_tree_bytes": sum(
                    p.stat().st_size for p in target.rglob("*") if p.is_file()
                ),
            }
        )

    phase("fresh")
    phase("noop")
    main = root / "app/src/main.rs"
    main.write_text(main.read_text(encoding="utf-8") + "\n// unrelated edit\n", encoding="utf-8")
    phase("unrelated-edit")
    phase("noop-after-unrelated")
    artifact_source = main if boundary == "same" else root / "blob_0/src/lib.rs"
    source = artifact_source.read_text(encoding="utf-8")
    old_invocation = f"embed!({case.size}, 0, identical, byte_string)"
    new_invocation = f"embed!({case.size}, 1, unique, byte_string)"
    if old_invocation not in source:
        raise RuntimeError("macro invocation not found for artifact edit")
    artifact_source.write_text(source.replace(old_invocation, new_invocation, 1), encoding="utf-8")
    phase("one-artifact-edit")
    phase("noop-after-artifact")
    return rows


def command_output(command: list[str], include_stderr: bool = False) -> str:
    """Collect required nonempty metadata under a short deadline."""
    with tempfile.TemporaryDirectory(prefix="tiler-embedding-metadata-") as temporary:
        root = Path(temporary)
        stdout_path = root / "stdout"
        stderr_path = root / "stderr"
        returncode = run_logged(
            command,
            stdout_path,
            stderr_path,
            METADATA_COMMAND_TIMEOUT_SECONDS,
        )
        require(returncode == 0, f"metadata command exited {returncode}: {command!r}")
        output = read_bounded(stdout_path, "metadata stdout")
        if include_stderr:
            output += read_bounded(stderr_path, "metadata stderr")
    require(bool(output.strip()), f"metadata command returned no output: {command!r}")
    return output.strip()


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    fields = sorted({key for row in rows for key in row if key != "command"})
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        writer.writerows({key: row.get(key) for key in fields} for row in rows)


def inherited_environment_identity() -> dict[str, dict[str, str | int]]:
    """Identify every inherited environment value without publishing secrets."""
    values = {}
    for key, value in sorted(os.environ.items()):
        encoded = value.encode()
        values[key] = {
            "bytes": len(encoded),
            "sha256": hashlib.sha256(encoded).hexdigest(),
        }
    return values


def executable_identity(name: str) -> dict[str, object]:
    selected = shutil.which(name)
    require(selected is not None, f"required executable is not on PATH: {name}")
    path = Path(selected).resolve()
    contents = path.read_bytes()
    return {
        "path": str(path),
        "bytes": len(contents),
        "sha256": hashlib.sha256(contents).hexdigest(),
    }


def path_identity(path: Path) -> dict[str, object]:
    """Identify an executable selected by an absolute harness path."""
    require(path.is_file(), f"required executable does not exist: {path}")
    resolved = path.resolve()
    contents = resolved.read_bytes()
    return {
        "path": str(resolved),
        "bytes": len(contents),
        "sha256": hashlib.sha256(contents).hexdigest(),
    }


def repository_revision() -> str:
    repository = Path(__file__).resolve().parents[2]
    revision = command_output(["git", "-C", str(repository), "rev-parse", "HEAD"])
    require(bool(re.fullmatch(r"[0-9a-f]{40}", revision)), "git returned an invalid revision")
    return revision


def evidence_identity(output: Path) -> list[dict[str, object]]:
    """Identify every published evidence file except the completion marker."""
    records = []
    for path in sorted(output.rglob("*")):
        relative = path.relative_to(output)
        if not path.is_file() or "work" in relative.parts or path.name == "complete.json":
            continue
        contents = path.read_bytes()
        records.append(
            {
                "path": relative.as_posix(),
                "bytes": len(contents),
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
        )
    require(bool(records), "measurement produced no evidence files")
    return records


def validate_rows(rows: object, *, freshness: bool) -> list[dict[str, object]]:
    require(isinstance(rows, list) and rows, "measurement rows must be a nonempty list")
    required = {"wall_seconds", "peak_rss_bytes", "command", "target_tree_bytes"}
    if not freshness:
        required |= {
            "binary_sha256",
            "final_binary_bytes",
            "macho_text_const_bytes",
            "macho_data_const_bytes",
            "payload_occurrences_min",
            "payload_occurrences_max",
        }
    for index, row in enumerate(rows):
        require(isinstance(row, dict), f"row {index} is not an object")
        missing = sorted(required - row.keys())
        require(not missing, f"row {index} is missing metrics: {missing}")
        require(
            isinstance(row["wall_seconds"], (int, float))
            and math.isfinite(row["wall_seconds"])
            and row["wall_seconds"] >= 0,
            f"row {index} has invalid wall_seconds",
        )
        require(
            isinstance(row["peak_rss_bytes"], int) and row["peak_rss_bytes"] > 0,
            f"row {index} has invalid peak_rss_bytes",
        )
        require(isinstance(row["command"], list) and row["command"], f"row {index} has no command")
        if not freshness:
            require(
                isinstance(row["macho_text_const_bytes"], int)
                and row["macho_text_const_bytes"] > 0,
                f"row {index} has invalid Mach-O text const metric",
            )
            require(
                isinstance(row["macho_data_const_bytes"], int)
                and row["macho_data_const_bytes"] >= 0,
                f"row {index} has invalid Mach-O data const metric",
            )
            require(
                isinstance(row["binary_sha256"], str)
                and bool(re.fullmatch(r"[0-9a-f]{64}", row["binary_sha256"])),
                f"row {index} has invalid binary identity",
            )
    return rows


def validate_csv(path: Path, rows: list[dict[str, object]]) -> None:
    try:
        with path.open(newline="", encoding="utf-8") as source:
            parsed = list(csv.DictReader(source))
    except (OSError, UnicodeError, csv.Error) as error:
        raise MeasurementFailure(f"cannot parse retained CSV {path}: {error}") from error
    require(len(parsed) == len(rows), f"CSV/JSON row-count mismatch: {path}")
    require(parsed and parsed[0].keys(), f"retained CSV has no schema: {path}")
    for index, (csv_row, json_row) in enumerate(zip(parsed, rows, strict=True)):
        for key, value in csv_row.items():
            expected = "" if json_row.get(key) is None else str(json_row.get(key))
            require(value == expected, f"CSV/JSON mismatch at row {index}, field {key}: {path}")


def verify_retained(root: Path) -> dict[str, object]:
    """Validate retained derived fixtures and state their evidence boundary."""
    required = ("metadata.json", "results.json", "results.csv", "freshness.json", "freshness.csv")
    for name in required:
        require((root / name).is_file(), f"missing retained fixture: {root / name}")
    integrity_path = root / "integrity.json"
    require(integrity_path.is_file(), f"missing retained fixture: {integrity_path}")
    try:
        integrity = json.loads(integrity_path.read_text(encoding="utf-8"))
        metadata = json.loads((root / "metadata.json").read_text(encoding="utf-8"))
        results = validate_rows(
            json.loads((root / "results.json").read_text(encoding="utf-8")), freshness=False
        )
        freshness = validate_rows(
            json.loads((root / "freshness.json").read_text(encoding="utf-8")), freshness=True
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise MeasurementFailure(f"cannot parse retained fixture: {error}") from error
    require(
        isinstance(metadata, dict) and metadata.get("schema_version") == 1,
        "unexpected legacy metadata schema",
    )
    validate_csv(root / "results.csv", results)
    validate_csv(root / "freshness.csv", freshness)
    digests = {}
    for name in required:
        contents = (root / name).read_bytes()
        digests[name] = hashlib.sha256(contents).hexdigest()
    require(
        isinstance(integrity, dict) and integrity.get("schema_version") == 1,
        "unexpected legacy integrity schema",
    )
    require(
        integrity.get("verification_status") == "verified-derived-legacy",
        "unexpected legacy verification status",
    )
    require(integrity.get("result_rows") == len(results), "legacy result-row count changed")
    require(
        integrity.get("freshness_rows") == len(freshness),
        "legacy freshness-row count changed",
    )
    require(integrity.get("fixture_sha256") == digests, "legacy fixture digest mismatch")
    limitations = integrity.get("limitations")
    require(
        isinstance(limitations, list)
        and limitations
        and all(isinstance(item, str) and item for item in limitations),
        "legacy integrity limitations are missing",
    )
    return {
        "status": "verified-derived-legacy",
        "result_rows": len(results),
        "freshness_rows": len(freshness),
        "fixture_sha256": digests,
        "limitations": limitations,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    destination = parser.add_mutually_exclusive_group(required=True)
    destination.add_argument("--output", type=Path)
    destination.add_argument(
        "--verify-retained",
        type=Path,
        help="validate a retained legacy result set without running Cargo",
    )
    parser.add_argument("--preset", choices=("smoke", "decision"), default="decision")
    parser.add_argument(
        "--repetitions",
        type=int,
        help="fresh builds per matrix cell (default: decision=3, smoke=1)",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=DEFAULT_COMMAND_TIMEOUT_SECONDS,
        help="hard deadline for each Cargo and inspection command",
    )
    parser.add_argument(
        "--overall-timeout-seconds",
        type=int,
        default=DEFAULT_OVERALL_TIMEOUT_SECONDS,
        help="hard deadline for the complete measurement run",
    )
    parser.add_argument("--keep-work", action="store_true")
    args = parser.parse_args()
    if args.verify_retained is not None:
        print(json.dumps(verify_retained(args.verify_retained.resolve()), indent=2))
        return
    require(sys.platform == "darwin", "measurement execution requires macOS")
    require(1 <= args.timeout_seconds <= 3600, "--timeout-seconds must be in 1..=3600")
    require(
        1 <= args.overall_timeout_seconds <= 21600,
        "--overall-timeout-seconds must be in 1..=21600",
    )
    global HARNESS_DEADLINE
    HARNESS_DEADLINE = time.monotonic() + args.overall_timeout_seconds
    signal.signal(signal.SIGALRM, overall_timeout_handler)
    signal.setitimer(signal.ITIMER_REAL, args.overall_timeout_seconds)
    require(args.output is not None, "--output is required for measurement execution")
    output = args.output.resolve()
    if output.exists():
        require(not any(output.iterdir()), f"output directory is not empty: {output}")
    output.mkdir(parents=True, exist_ok=True)
    raw = output / "raw"
    raw.mkdir(exist_ok=True)
    work = output / "work"
    work.mkdir()
    cases = smoke_cases() if args.preset == "smoke" else decision_cases()
    repetitions = (
        args.repetitions if args.repetitions is not None else (1 if args.preset == "smoke" else 3)
    )
    if repetitions < 1:
        parser.error("--repetitions must be positive")
    results = []
    for index, case in enumerate(cases, 1):
        print(f"[{index}/{len(cases)}] {case.name}", flush=True)
        for repetition in range(1, repetitions + 1):
            root = work / f"{case.name}-r{repetition}"
            root.mkdir()
            payloads = make_workspace(root, case)
            results.append(
                run_build(
                    root,
                    case,
                    payloads,
                    raw,
                    repetition,
                    args.timeout_seconds,
                )
            )
    freshness = []
    if args.preset == "decision":
        for boundary in ("same", "cross"):
            print(f"[freshness] {boundary}", flush=True)
            freshness.extend(freshness_case(output, raw, boundary, args.timeout_seconds))
    validate_rows(results, freshness=False)
    if freshness:
        validate_rows(freshness, freshness=True)
    script = Path(__file__).resolve()
    metadata = {
        "schema_version": HARNESS_SCHEMA_VERSION,
        "preset": args.preset,
        "repetitions": repetitions,
        "command_timeout_seconds": args.timeout_seconds,
        "overall_timeout_seconds": args.overall_timeout_seconds,
        "harness": {
            "repository_revision": repository_revision(),
            "path": str(script),
            "bytes": script.stat().st_size,
            "sha256": hashlib.sha256(script.read_bytes()).hexdigest(),
        },
        "inherited_environment_identity": inherited_environment_identity(),
        "executables": {
            "cargo": executable_identity("cargo"),
            "rustc": executable_identity("rustc"),
            "size": executable_identity("size"),
            "time": path_identity(Path("/usr/bin/time")),
            "python": path_identity(Path(sys.executable)),
        },
        "host": platform.platform(),
        "uname": command_output(["uname", "-s", "-r", "-m", "-v"]),
        "sw_vers": command_output(["sw_vers"]),
        "rustc": command_output(["rustc", "-Vv"]),
        "cargo": command_output(["cargo", "-Vv"]),
        "xcode": command_output(["xcodebuild", "-version"]),
        "linker": command_output(["xcrun", "ld", "-v"], include_stderr=True),
        "hardware_model": command_output(["sysctl", "-n", "hw.model"]),
        "hardware_memory_bytes": int(command_output(["sysctl", "-n", "hw.memsize"])),
        "hardware_logical_cpus": int(command_output(["sysctl", "-n", "hw.ncpu"])),
        "python": platform.python_version(),
        "commands": [row["command"] for row in results] + [row["command"] for row in freshness],
    }
    (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    (output / "results.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
    (output / "freshness.json").write_text(json.dumps(freshness, indent=2) + "\n", encoding="utf-8")
    write_csv(output / "results.csv", results)
    if freshness:
        write_csv(output / "freshness.csv", freshness)
    validate_csv(output / "results.csv", results)
    if freshness:
        validate_csv(output / "freshness.csv", freshness)
    if not args.keep_work:
        shutil.rmtree(work)
    completion = {
        "schema_version": HARNESS_SCHEMA_VERSION,
        "status": "complete",
        "evidence": evidence_identity(output),
    }
    temporary_completion = output / ".complete.json.tmp"
    temporary_completion.write_text(json.dumps(completion, indent=2) + "\n", encoding="utf-8")
    os.replace(temporary_completion, output / "complete.json")
    signal.setitimer(signal.ITIMER_REAL, 0)


if __name__ == "__main__":
    main()
