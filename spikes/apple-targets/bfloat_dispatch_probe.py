#!/usr/bin/env python3
"""Does the iOS Simulator refuse the `bfloat` *type* or `bfloat` *arithmetic*?

Finding 26 of the numerical-behaviour record states that the simulator compiles
and links every `bfloat` module and then fails pipeline creation. `bfloat_support`
in `numerical_probe.py` establishes that on every gate run, but it asks with
`multiply_two_bf16` alone, so it cannot separate two readings of the refusal:

- the simulator has no `bfloat` support at all, so a kernel that merely *declares*
  `bfloat` buffers and copies between them is refused too; or
- the simulator lowers the type but not the arithmetic, so an arithmetic-free
  kernel would dispatch.

This probe asks the arithmetic-free kernel directly. It is deliberately **not**
part of the gate: the refusal path costs minutes of `XPC` retries per case, and
what the gate needs — that the simulator's `bf16` cases are refused and its `f32`
and `f16` cases are not — the retained record already carries on every run.

**The control is the point, and it runs last as well as first.** A refusal
observed after two faults could be the simulator's GPU compiler service
degrading rather than a fact about `bfloat`, and that reading is only excluded by
showing a kernel that worked before the faults still works after them. So
`materialize_f16` is dispatched, then the two `bfloat` kernels, then
`materialize_f16` again. A run whose trailing control fails establishes nothing
about `bfloat` and must be discarded rather than reported.

Usage, from the repository root:

    uv run python spikes/apple-targets/bfloat_dispatch_probe.py [work-directory]

Omitting the directory uses a fresh temporary one. Exits nonzero when the
trailing control fails, because that is the case whose result is not evidence.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import numerical_probe as probe  # noqa: E402

CONTROL = "materialize_f16"
"""The kernel dispatched before and after the `bfloat` attempts.

`f16` and not `f32`: it is the same 16-bit element width the `bfloat` kernels
use, so a width-related dispatch fault would show up here too and could not be
mistaken for a format-related one.
"""

SUBJECTS = ("materialize_bf16", "multiply_one_bf16")
"""The `bfloat` kernels asked, arithmetic-free first.

`materialize_bf16` emits no floating-point operation at all — it declares
`bfloat` buffers, loads, and stores — so a refusal of *this* kernel cannot be
about arithmetic. `multiply_one_bf16` folds to zero operations at `-O2` as well
and is included because it still names `bfloat` in an arithmetic expression the
front end then removes.
"""


def _run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, capture_output=True, text=True, check=False)


def _booted_simulator() -> str:
    """Boot and return a spawnable iOS simulator UDID, or raise."""
    simctl = _run(["xcrun", "-f", "simctl"]).stdout.strip()
    if not simctl:
        raise SystemExit("simctl was not found by xcrun")
    listed = json.loads(_run([simctl, "list", "devices", "available", "-j"]).stdout)["devices"]
    for runtime, devices in sorted(listed.items()):
        if "iOS" not in runtime or not devices:
            continue
        udid = str(devices[0]["udid"])
        _run([simctl, "boot", udid])
        for _ in range(60):
            if _run([simctl, "spawn", udid, "/usr/bin/true"]).returncode == 0:
                return udid
        raise SystemExit(f"{udid} booted but never became spawnable")
    raise SystemExit("no available iOS simulator device")


def _dispatch(simctl: str, udid: str, host: Path, work: Path, name: str) -> tuple[bool, str]:
    """Compile, link, and dispatch one kernel; return whether it ran and the detail."""
    kernel = probe.BY_NAME[name]
    family = probe.FAMILY_BY_NAME["ios-simulator"]
    source = work / f"{name}.metal"
    source.write_text(kernel.source(), encoding="utf-8")
    library = work / f"{name}.metallib"
    compiled = _run(
        [
            "xcrun",
            "-sdk",
            family.sdk,
            "metal",
            f"-std={probe.MSL_VERSION}",
            "-target",
            family.target,
            "-O2",
            "-ffp-contract=off",
            "-fmetal-math-mode=safe",
            str(source),
            "-o",
            str(library),
        ]
    )
    if compiled.returncode != 0:
        return False, f"offline compilation failed: {compiled.stderr.strip()}"
    manifest = work / f"{name}.manifest.tsv"
    # The harness's own manifest writer, private though it is, rather than a
    # second copy of the format. A duplicate would drift from the dispatch host
    # silently, and this probe's whole value is that it asks the same host the
    # same way the gate does and differs only in which kernel it names.
    manifest.write_text(
        probe._manifest_line(name, kernel.dtype, library, probe.ENTRY_POINT, None) + "\n",  # noqa: SLF001
        encoding="utf-8",
    )
    dispatched = _run(
        [simctl, "spawn", udid, str(host), "batch", str(manifest), *probe.operand_arguments()]
    )
    if dispatched.returncode != 0:
        return False, dispatched.stderr.strip()
    values = [
        line.split("=", 1)[1]
        for line in dispatched.stdout.splitlines()
        if line.startswith("result=")
    ]
    return True, " ".join(values)


def main(arguments: list[str] | None = None) -> int:
    argv = sys.argv[1:] if arguments is None else arguments
    directory = Path(argv[0]) if argv else Path(tempfile.mkdtemp(prefix="tiler-bfloat-dispatch-"))
    directory.mkdir(parents=True, exist_ok=True)

    udid = _booted_simulator()
    simctl = _run(["xcrun", "-f", "simctl"]).stdout.strip()
    family = probe.FAMILY_BY_NAME["ios-simulator"]
    host = directory / "numerical_probe_host"
    probe.resolve().build_host(host, family.sdk)

    print(f"work directory: {directory}")
    print(f"simulator udid: {udid}")

    order = (CONTROL, *SUBJECTS, CONTROL)
    outcomes: list[tuple[str, bool, str]] = []
    for index, name in enumerate(order):
        ran, detail = _dispatch(simctl, udid, host, directory, name)
        outcomes.append((name, ran, detail))
        position = (
            "leading control" if index == 0 else "trailing control" if name == CONTROL else ""
        )
        print(f"\n=== {name} {position}".rstrip())
        print(f"  {'dispatched' if ran else 'REFUSED'}: {detail}")

    leading, trailing = outcomes[0], outcomes[-1]
    if not (leading[1] and trailing[1]):
        print(
            "\nthe control did not dispatch on both sides, so this run says nothing about bfloat",
            file=sys.stderr,
        )
        return 1
    refused = [name for name, ran, _ in outcomes[1:-1] if not ran]
    print(f"\ncontrol dispatched before and after: {leading[1] and trailing[1]}")
    print(f"bfloat kernels refused: {refused or 'none'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
