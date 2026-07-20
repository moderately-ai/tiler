#!/usr/bin/env python3
"""Build deterministic proc-macro byte-literal fixtures and record cost data.

The dependency-free stable proc macro emits the literal tokens directly. It
does not use include_bytes!, because that bypasses the token representation
whose cost this spike measures.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import tempfile
import time
from dataclasses import asdict, dataclass
from pathlib import Path


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
            add(name=f"repr-{representation}-{size}", size=size, count=1,
                representation=representation, identity="unique", boundary="same",
                profile="release")
        add(name=f"debug-byte-string-{size}", size=size, count=1,
            representation="byte-string", identity="unique", boundary="same",
            profile="dev")

    # Multiplicity: enough to reveal both linear growth and any identical folding.
    for count in (8, 32):
        for identity in ("identical", "unique"):
            add(name=f"count-{count}-{identity}", size=100 * 1024, count=count,
                representation="byte-string", identity=identity, boundary="same",
                profile="release")

    # Crate boundary and profile are crossed for the central 8 x 100 KiB case.
    for boundary in ("same", "cross"):
        for profile in ("dev", "release"):
            for identity in ("identical", "unique"):
                add(name=f"boundary-{boundary}-{profile}-{identity}", size=100 * 1024,
                    count=8, representation="byte-string", identity=identity,
                    boundary=boundary, profile=profile)

    # Keep payload constant while varying release linker/codegen settings.
    for boundary in ("same", "cross"):
        for codegen_units, lto in ((1, "off"), (16, "thin"), (1, "fat")):
            add(name=f"config-{boundary}-cgu{codegen_units}-{lto}", size=100 * 1024,
                count=8, representation="byte-string", identity="identical",
                boundary=boundary, profile="release", codegen_units=codegen_units,
                lto=lto)
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
        modules.append(f"mod blob_{index} {{\n{library_source(case.size, index, case.identity, case.representation)}}}\n")
        calls.append(f"blob_{index}::artifact()")
    return "".join(modules) + main_body(calls)


def macro_source() -> str:
    return r'''use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};

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
'''


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
        "    println!(\"{}:{}\", artifacts.len(), digest);\n"
        "}\n"
    )


def make_workspace(root: Path, case: Case) -> list[bytes]:
    payloads = [artifact_bytes(case.size, 0 if case.identity == "identical" else i, case.identity)
                for i in range(case.count)]
    app = root / "app"
    (app / "src").mkdir(parents=True)
    macro_crate = root / "embed_macro"
    (macro_crate / "src").mkdir(parents=True)
    (macro_crate / "Cargo.toml").write_text(
        package_toml("embed_macro") + "\n[lib]\nproc-macro = true\n", encoding="utf-8")
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
                library_source(case.size, index, case.identity, case.representation), encoding="utf-8")
            (crate / "Cargo.toml").write_text(
                package_toml(f"embed_blob_{index}") +
                '\n[dependencies]\nembed_macro = { path = "../embed_macro" }\n', encoding="utf-8")
            dep_lines.append(f'embed_blob_{index} = {{ path = "../blob_{index}" }}')
            calls.append(f"embed_blob_{index}::artifact()")
        deps = "\n[dependencies]\n" + "\n".join(dep_lines) + "\n"
        source = main_body(calls)
    (app / "Cargo.toml").write_text(package_toml("embed_app") + deps, encoding="utf-8")
    (app / "src/main.rs").write_text(source, encoding="utf-8")
    lto_value = "false" if case.lto == "off" else f'"{case.lto}"'
    manifest = (
        "[workspace]\nresolver = \"2\"\n"
        f"members = {json.dumps(members)}\n\n"
        "[profile.dev]\ndebug = 2\nincremental = true\n"
        f"codegen-units = {case.codegen_units}\n\n"
        "[profile.release]\ndebug = 0\nincremental = false\n"
        f"codegen-units = {case.codegen_units}\nlto = {lto_value}\n"
    )
    (root / "Cargo.toml").write_text(manifest, encoding="utf-8")
    return payloads


def parse_time(path: Path) -> tuple[float | None, int | None]:
    text = path.read_text(encoding="utf-8")
    wall_match = re.search(r"\s([0-9.]+) real\s", text)
    rss_match = re.search(r"^\s*([0-9]+)\s+maximum resident set size", text, re.MULTILINE)
    return (float(wall_match.group(1)) if wall_match else None,
            int(rss_match.group(1)) if rss_match else None)


def file_sum(root: Path, suffix: str) -> int:
    return sum(path.stat().st_size for path in root.rglob(f"*{suffix}") if path.is_file())


def macho_sections(binary: Path) -> tuple[int | None, int | None, str]:
    proc = subprocess.run(["size", "-m", str(binary)], text=True, capture_output=True, check=True)
    values: dict[str, int] = {}
    current_segment = ""
    for line in proc.stdout.splitlines():
        segment = re.match(r"Segment (\S+): (\d+)", line.strip())
        if segment:
            current_segment = segment.group(1)
            continue
        section = re.match(r"Section (\S+): (\d+)", line.strip())
        if section:
            values[f"{current_segment},{section.group(1)}"] = int(section.group(2))
    text_const = sum(value for key, value in values.items() if key.startswith("__TEXT,") and key.endswith(",__const"))
    data_const = sum(value for key, value in values.items() if key.startswith("__DATA") and key.endswith(",__const"))
    return text_const or None, data_const or None, proc.stdout


def build_command(root: Path, case: Case, target: Path, timing: Path) -> list[str]:
    return ["/usr/bin/time", "-l", "-o", str(timing), "cargo", "build", "--offline",
            "--manifest-path", str(root / "Cargo.toml"), "--target-dir", str(target),
            "--profile", case.profile]


def run_build(root: Path, case: Case, payloads: list[bytes], raw: Path,
              repetition: int) -> dict[str, object]:
    target = root / "target"
    timing = root / "time.txt"
    command = build_command(root, case, target, timing)
    started = time.monotonic()
    proc = subprocess.run(command, text=True, capture_output=True)
    elapsed = time.monotonic() - started
    label = f"{case.name}-r{repetition}"
    (raw / f"{label}.stdout").write_text(proc.stdout, encoding="utf-8")
    (raw / f"{label}.stderr").write_text(proc.stderr, encoding="utf-8")
    if proc.returncode != 0:
        raise RuntimeError(f"{case.name} failed ({proc.returncode}); see raw stderr")
    wall, rss = parse_time(timing)
    binary = target / ("debug" if case.profile == "dev" else "release") / "embed_app"
    binary_bytes = binary.read_bytes()
    occurrences = [binary_bytes.count(payload) for payload in payloads]
    text_const, data_const, size_raw = macho_sections(binary)
    (raw / f"{label}.size-m.txt").write_text(size_raw, encoding="utf-8")
    source_bytes = sum(path.stat().st_size for path in root.rglob("*.rs"))
    result = asdict(case)
    result.update({
        "command": command,
        "repetition": repetition,
        "wall_seconds": wall if wall is not None else round(elapsed, 6),
        "peak_rss_bytes": rss,
        "source_bytes": source_bytes,
        "logical_payload_bytes": case.size * case.count,
        "byte_value_literal_tokens": case.count if case.representation == "byte-string" else case.size * case.count,
        "expanded_text_proxy_bytes": sum(len(rust_literal(payload, case.representation)) for payload in payloads),
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
    })
    return result


def freshness_case(output: Path, raw: Path, boundary: str) -> list[dict[str, object]]:
    case = Case(f"freshness-{boundary}", 100 * 1024, 8, "byte-string", "identical", boundary, "dev")
    root = output / "work" / case.name
    root.mkdir(parents=True)
    payloads = make_workspace(root, case)
    target = root / "target"
    rows = []

    def phase(name: str) -> None:
        timing = root / f"time-{name}.txt"
        command = build_command(root, case, target, timing)
        proc = subprocess.run(command, text=True, capture_output=True)
        (raw / f"{case.name}-{name}.stdout").write_text(proc.stdout, encoding="utf-8")
        (raw / f"{case.name}-{name}.stderr").write_text(proc.stderr, encoding="utf-8")
        if proc.returncode != 0:
            raise RuntimeError(f"{case.name}/{name} failed; see raw stderr")
        wall, rss = parse_time(timing)
        rows.append({"boundary": boundary, "phase": name, "wall_seconds": wall,
                     "peak_rss_bytes": rss, "command": command,
                     "target_tree_bytes": sum(p.stat().st_size for p in target.rglob("*") if p.is_file())})

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
    proc = subprocess.run(command, text=True, capture_output=True, check=True)
    output = proc.stdout + (proc.stderr if include_stderr else "")
    return output.strip()


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    fields = sorted({key for row in rows for key in row if key != "command"})
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        writer.writerows({key: row.get(key) for key in fields} for row in rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--preset", choices=("smoke", "decision"), default="decision")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repetitions", type=int,
                        help="fresh builds per matrix cell (default: decision=3, smoke=1)")
    parser.add_argument("--keep-work", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    raw = output / "raw"
    raw.mkdir(exist_ok=True)
    work = output / "work"
    if work.exists():
        shutil.rmtree(work)
    work.mkdir()
    cases = smoke_cases() if args.preset == "smoke" else decision_cases()
    repetitions = args.repetitions if args.repetitions is not None else (1 if args.preset == "smoke" else 3)
    if repetitions < 1:
        parser.error("--repetitions must be positive")
    results = []
    for index, case in enumerate(cases, 1):
        print(f"[{index}/{len(cases)}] {case.name}", flush=True)
        for repetition in range(1, repetitions + 1):
            root = work / f"{case.name}-r{repetition}"
            root.mkdir()
            payloads = make_workspace(root, case)
            results.append(run_build(root, case, payloads, raw, repetition))
    freshness = []
    if args.preset == "decision":
        for boundary in ("same", "cross"):
            print(f"[freshness] {boundary}", flush=True)
            freshness.extend(freshness_case(output, raw, boundary))
    metadata = {
        "schema_version": 1,
        "preset": args.preset,
        "repetitions": repetitions,
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
    if not args.keep_work:
        shutil.rmtree(work)


if __name__ == "__main__":
    main()
