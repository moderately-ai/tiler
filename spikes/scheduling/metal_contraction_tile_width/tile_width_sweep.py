#!/usr/bin/env python3
"""Driver for the contraction tile-width sweep.

The governing protocol is `PROTOCOL-2026-08-22-contraction-tile-width.md`, which
was committed before this file existed. Every population constant below is that
document's, restated here so the harness can fail against it rather than trust
it. Where this file and the protocol disagree, the protocol is authoritative and
the disagreement is a defect in this file.

Three modes:

  validate  Compile, prepare, and check. Runs dispatches but reads no wall
            clock and emits no timing. Valid on a loaded host.
  timing    The frozen sweep. Requires a quiet host; the driver reads the
            quiet-host gate itself and refuses rather than trusting the caller.
  gate      Read the quiet-host gate alone and report its verdict, reading no
            wall clock and dispatching nothing. Exit 0 admits, exit 1 refuses.
  perturb   Break the subject one property at a time and show the check's own
            failure text. Perturbing the assertions would prove only that they
            execute.

No third-party package is needed in any mode. The operand reconstruction is
pure Python because the oracle here is cross-variant rather than a host oracle:
the property that matters for a comparison sweep is that every variant consumed
the same operands, which is checked by digest equality across variants at a
cell, and additionally against an independent reconstruction on the small
validation cells.
"""

import argparse
import fcntl
import hashlib
import os
import re
import shutil
import struct
import subprocess
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# The frozen population. These are assertions, not configuration.
# ---------------------------------------------------------------------------

SQUARE_WIDTHS = [1, 2, 4, 8, 16, 32]
RECTANGULAR_PAIRS = [
    (1, 8), (2, 8), (4, 8),
    (1, 16), (2, 16), (4, 16), (8, 16),
    (1, 32), (2, 32), (4, 32), (8, 32), (16, 32),
]
FROZEN_VARIANT_COUNT = 18
FROZEN_CELL_COUNT = 14
FROZEN_PIPELINE_COUNT = 21

# Group A -- the M-sweep at fixed N and K, which exists because the retained
# record has no cell between M = 1 and M = 10.
GROUP_A = [(f"a_m{m}", m, 8192, 1024) for m in (1, 2, 4, 8, 16, 32, 128)]

# Group B -- the pinned workload cells, so the sweep speaks to the workload
# rather than to a synthetic shape.
GROUP_B = [
    ("w_decode_kv", 1, 1024, 1024),
    ("w_prefill_q", 10, 2048, 1024),
    ("w_prefill_mlp_in", 128, 3072, 1024),
    ("w_prefill_mlp_out", 128, 1024, 3072),
    ("w_prefill_o", 128, 1024, 2048),
    ("w_vocab_slice", 1, 8192, 1024),
    ("t_vocab_full", 1, 151936, 1024),
    ("t_prefill_mlp_512", 512, 3072, 1024),
]

FROZEN_CONTRACTED_EXTENTS = {1024, 2048, 3072}

# Small cells for the validation leg. K stays inside the frozen contracted
# extents; M = 3 and M = 17 are deliberately indivisible by every tile height
# above 1, so the masking path is exercised rather than assumed.
VALIDATION_CELLS = [(f"v_m{m}", m, 256, 1024) for m in (1, 2, 3, 16, 17, 32, 128)]

OPERAND_SEED = 20260822
SIGNED_ZERO_SOURCE = "const:80000000,00000000"

METAL_FLAGS = [
    "-target", "air64-apple-macos26.0",
    "-std=metal4.0",
    "-O2",
    "-fmetal-math-mode=safe",
    "-fmetal-math-fp32-functions=precise",
    "-ffp-contract=off",
]

# ---------------------------------------------------------------------------
# The quiet-host gate.
#
# Amended 2026-08-22 (see the protocol's "Pre-run amendment" section) before any
# dispatch was timed. The superseded gate was a single absolute threshold,
# `LOAD_GATE = 0.5` on the one-minute load average, which this host cannot
# satisfy at any time: its idle one-minute load is 2.0-2.4, a floor that does
# not decay. That gate did not delay a run, it foreclosed one.
#
# Every component below fails closed. A component whose input cannot be read
# REFUSES, because an unreadable probe and a quiet host produce the same silence
# and guessing "quiet" is the direction that admits a contaminated measurement.
# ---------------------------------------------------------------------------

# Primary discriminator: the resource competing work actually consumes.
# Measured on the bench host 2026-08-22 over 60 consecutive one-second samples:
# the quiet mode is 97.2-99.7% idle, and an episodic desktop-session burst
# occupies 84.2-89.8% idle (~1.1 of 11 cores). The floor sits in the empty band
# between those two modes.
GATE_CPU_IDLE_FLOOR_PERCENT = 95.0
GATE_CPU_SAMPLES = 10

# The metric is `GPUEndTime - GPUStartTime`, so the GPU is the contended device.
# Measured 0% across ten samples on the quiet bench host, with one reading of 1%.
GATE_GPU_UTILIZATION_CEILING_PERCENT = 5

# Lagging indicator, kept because it sees sustained work that is blocked rather
# than running. It is a ceiling ABOVE this host's recorded baseline, never an
# absolute quiet figure -- that distinction is the whole amendment.
GATE_LOAD_BASELINE = (
    "one-minute 1.86-2.47 over 20+ observations 2026-08-22; retained in-tree at "
    "spikes/program-planning/physical-frontier-budget-calibration/results/ as "
    "{ 2.22 2.39 2.26 } on 2026-08-13 and { 2.18 2.23 2.24 } on 2026-08-14, same host"
)
GATE_LOAD_CEILING = 3.5

# So two measurement sessions cannot share the device. `flock` is released by the
# kernel when the holder dies, so a crashed run leaves no stale lock behind.
GATE_LOCK_PATH = "/tmp/tiler-contraction-tile-width-sweep.lock"


def variant_name(tile_m, tile_w):
    return f"contract_tiled_m{tile_m}_w{tile_w}"


def sweep_variants():
    """The 18 frozen tiled variants, square arm first."""
    variants = [(w, w) for w in SQUARE_WIDTHS] + list(RECTANGULAR_PAIRS)
    if len(variants) != FROZEN_VARIANT_COUNT:
        raise SystemExit(
            f"population floor: built {len(variants)} tiled variants, "
            f"the protocol freezes {FROZEN_VARIANT_COUNT}"
        )
    for tile_m, tile_w in variants:
        if tile_w % tile_m != 0:
            raise SystemExit(f"variant ({tile_m}, {tile_w}) violates TILE_M | TILE_W")
        if tile_m * tile_w > 1024:
            raise SystemExit(f"variant ({tile_m}, {tile_w}) exceeds 1024 threads")
    return variants


def sweep_cells():
    """The 14 distinct frozen cells. Group A and group B share exactly one."""
    seen = {}
    order = []
    for name, m, n, k in GROUP_A + GROUP_B:
        key = (m, n, k)
        if key in seen:
            continue
        seen[key] = name
        order.append((name, m, n, k))
    if len(order) != FROZEN_CELL_COUNT:
        raise SystemExit(
            f"population floor: built {len(order)} distinct cells, "
            f"the protocol freezes {FROZEN_CELL_COUNT}"
        )
    for name, _m, _n, k in order:
        if k not in FROZEN_CONTRACTED_EXTENTS:
            raise SystemExit(
                f"cell {name} has K={k}, outside the frozen contracted extents "
                f"{sorted(FROZEN_CONTRACTED_EXTENTS)}"
            )
    return order


# ---------------------------------------------------------------------------
# Operand reconstruction. Mirrors `host.m` exactly.
# ---------------------------------------------------------------------------

MASK64 = (1 << 64) - 1


def splitmix64(x):
    x = (x + 0x9E3779B97F4A7C15) & MASK64
    z = x
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
    return z ^ (z >> 31)


def prng_bytes(seed, count):
    out = bytearray()
    step = 0x2545F4914F6CDD1D
    for index in range(count):
        bits = splitmix64((seed + index * step) & MASK64)
        magnitude = ((bits >> 40) & 0xFFFFFF) - 8388608
        out += struct.pack("<f", magnitude / 16777216.0)
    return bytes(out)


def prng_digest(seed, count):
    return hashlib.sha256(prng_bytes(seed, count)).hexdigest()


# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

def run(cmd, **kwargs):
    completed = subprocess.run(cmd, capture_output=True, text=True, **kwargs)
    if completed.returncode != 0:
        sys.stderr.write(f"command failed: {' '.join(cmd)}\n{completed.stdout}\n{completed.stderr}\n")
        raise SystemExit(1)
    return completed


def toolchain_facts():
    """Every toolchain fact is reported with the invocation that produced it.

    A bare `xcrun ... metal --version` answers for whatever `xcode-select -p`
    points at, which is not necessarily what this run compiles with. The frozen
    commands pin DEVELOPER_DIR; both answers are recorded so a reader can see
    which one the record rests on.
    """
    facts = {}
    facts["developer_dir_env"] = os.environ.get("DEVELOPER_DIR", "<unset>")
    facts["xcode_select_p"] = run(["xcode-select", "-p"]).stdout.strip()
    facts["metal_version_as_invoked"] = run(
        ["xcrun", "--sdk", "macosx", "metal", "--version"]
    ).stdout.splitlines()[0].strip()
    facts["metal_version_invocation"] = (
        f"DEVELOPER_DIR={facts['developer_dir_env']} xcrun --sdk macosx metal --version"
    )
    facts["sdk_version"] = run(["xcrun", "--sdk", "macosx", "--show-sdk-version"]).stdout.strip()
    facts["sdk_build"] = run(["xcrun", "--sdk", "macosx", "--show-sdk-build-version"]).stdout.strip()
    xcodebuild = run(["xcodebuild", "-version"]).stdout.split()
    facts["xcode"] = " ".join(xcodebuild[:2]) if len(xcodebuild) >= 2 else "unknown"
    facts["host_os"] = (
        run(["sw_vers", "-productVersion"]).stdout.strip()
        + " "
        + run(["sw_vers", "-buildVersion"]).stdout.strip()
    )
    facts["host_arch"] = run(["uname", "-m"]).stdout.strip()
    facts["host_cpu"] = run(["sysctl", "-n", "machdep.cpu.brand_string"]).stdout.strip()
    facts["offline_flags"] = " ".join(METAL_FLAGS)
    return facts


def load_average():
    """One-minute load average. A lagging indicator, never a quiet threshold."""
    return os.getloadavg()[0]


def cpu_idle_samples(samples=GATE_CPU_SAMPLES):
    """Instantaneous CPU idle percentages, one per second.

    `top`'s FIRST sample is a since-boot average, not an instantaneous reading,
    so one extra sample is requested and discarded. This is not a stylistic
    preference: on this bench host `top -l 1` read 80.40% and 69.18% idle within
    minutes of sustained samples reading 98.8-99.4%, so a gate written against
    `top -l 1` reads a different quantity than its threshold describes and
    refuses a host that is in fact quiet.

    Returns fewer than `samples` entries when the output cannot be parsed, which
    the gate treats as a refusal.
    """
    completed = subprocess.run(
        ["top", "-l", str(samples + 1), "-s", "1", "-n", "0"],
        capture_output=True, text=True,
    )
    idle = [float(v) for v in re.findall(r"([0-9.]+)%\s+idle", completed.stdout)]
    return idle[1:]


def gpu_utilization_percent():
    """Peak `Device Utilization %` over every IOAccelerator node, or None.

    None means the field could not be read at all, and the gate refuses on it.
    A probe that silently stops matching -- a renamed ioreg key, a restructured
    class -- otherwise reports the admitting value forever, which is exactly how
    the deferred-trigger audit's `system_profiler` check read "not fired" for
    months after macOS renamed the data type it polled.
    """
    completed = subprocess.run(
        ["ioreg", "-r", "-d", "1", "-w", "0", "-c", "IOAccelerator"],
        capture_output=True, text=True,
    )
    found = re.findall(r'"Device Utilization %"=([0-9]+)', completed.stdout)
    if not found:
        return None
    return max(int(v) for v in found)


def acquire_measurement_lock():
    """Take the exclusive measurement lock, or return None if another run holds it.

    The returned handle must outlive the run: closing it releases the lock.
    """
    handle = open(GATE_LOCK_PATH, "w")
    try:
        fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        handle.close()
        return None
    handle.write(f"{os.getpid()}\n")
    handle.flush()
    return handle


def quiet_host_gate(phase, lock_handle=None):
    """Decide whether the host is quiet enough for a wall-clock measurement.

    Returns `(admitted, report_lines, fields)`. `phase` labels the recorded
    fields so the start and end reads stay distinguishable in `environment.tsv`.
    Passing `lock_handle=None` skips the lock component, which is what the end
    read wants: the run already holds the lock it took at the start.
    """
    lines = []
    fields = {}
    refusals = []

    idle = cpu_idle_samples()
    if len(idle) < GATE_CPU_SAMPLES:
        fields[f"gate.{phase}.cpu_idle_mean_percent"] = "UNREADABLE"
        lines.append(
            f"  CPU idle, mean of {GATE_CPU_SAMPLES} x 1 s   UNREADABLE  REFUSE")
        refusals.append(
            f"CPU idle is unreadable: parsed {len(idle)} of {GATE_CPU_SAMPLES} samples "
            f"from `top`. An unreadable probe refuses; it does not pass.")
    else:
        mean_idle = sum(idle) / len(idle)
        fields[f"gate.{phase}.cpu_idle_mean_percent"] = f"{mean_idle:.2f}"
        fields[f"gate.{phase}.cpu_idle_min_percent"] = f"{min(idle):.2f}"
        ok = mean_idle >= GATE_CPU_IDLE_FLOOR_PERCENT
        lines.append(
            f"  CPU idle, mean of {len(idle)} x 1 s   {mean_idle:6.2f}%  "
            f"(floor {GATE_CPU_IDLE_FLOOR_PERCENT}%, min sample {min(idle):.2f}%)  "
            f"{'pass' if ok else 'REFUSE'}")
        if not ok:
            refusals.append(
                f"CPU idle averaged {mean_idle:.2f}% over {len(idle)} s, below the "
                f"{GATE_CPU_IDLE_FLOOR_PERCENT}% floor. Work is competing for CPU, and on a "
                f"unified-memory device that competes for bandwidth with the dispatch "
                f"being timed.")

    gpu = gpu_utilization_percent()
    fields[f"gate.{phase}.gpu_utilization_percent"] = "UNREADABLE" if gpu is None else str(gpu)
    if gpu is None:
        lines.append("  GPU device utilization         UNREADABLE  REFUSE")
        refusals.append(
            "GPU utilization is unreadable: no `Device Utilization %` field was found "
            "on any IOAccelerator node. The field may have been renamed. An unreadable "
            "probe refuses, because a renamed key and an idle GPU look identical.")
    else:
        ok = gpu <= GATE_GPU_UTILIZATION_CEILING_PERCENT
        lines.append(
            f"  GPU device utilization         {gpu:6d}%  "
            f"(ceiling {GATE_GPU_UTILIZATION_CEILING_PERCENT}%)  {'pass' if ok else 'REFUSE'}")
        if not ok:
            refusals.append(
                f"GPU device utilization is {gpu}%, above the "
                f"{GATE_GPU_UTILIZATION_CEILING_PERCENT}% ceiling. Another workload holds the "
                f"device this measurement times.")

    load = load_average()
    fields[f"gate.{phase}.load_average_1min"] = f"{load:.2f}"
    ok = load <= GATE_LOAD_CEILING
    lines.append(
        f"  load average, one minute       {load:6.2f}   "
        f"(ceiling {GATE_LOAD_CEILING}, recorded idle baseline {GATE_LOAD_BASELINE})  "
        f"{'pass' if ok else 'REFUSE'}")
    if not ok:
        refusals.append(
            f"One-minute load average is {load:.2f}, above the {GATE_LOAD_CEILING} ceiling. "
            f"This ceiling is relative to this host's recorded idle baseline of "
            f"{GATE_LOAD_BASELINE}, not an absolute quiet figure.")

    if lock_handle is not None:
        held = lock_handle is not False
        fields[f"gate.{phase}.measurement_lock"] = GATE_LOCK_PATH
        lines.append(
            f"  exclusive measurement lock     {GATE_LOCK_PATH}  "
            f"{'held' if held else 'REFUSE'}")
        if not held:
            refusals.append(
                f"Another measurement session holds {GATE_LOCK_PATH}. Two sweeps sharing "
                f"one device measure each other.")

    return (not refusals), lines, fields, refusals


def build(work_dir, source_dir):
    work_dir.mkdir(parents=True, exist_ok=True)
    air = work_dir / "kernels.air"
    lib = work_dir / "kernels.metallib"
    host = work_dir / "host"
    run(["xcrun", "--sdk", "macosx", "metal", "-c", str(source_dir / "kernels.metal"),
         "-o", str(air)] + METAL_FLAGS)
    run(["xcrun", "--sdk", "macosx", "metallib", str(air), "-o", str(lib)])
    run(["xcrun", "--sdk", "macosx", "clang", "-fobjc-arc", "-O2",
         str(source_dir / "host.m"), "-o", str(host),
         "-framework", "Foundation", "-framework", "Metal"])
    return lib, host


def sha256_file(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


# ---------------------------------------------------------------------------
# Manifest and execution
# ---------------------------------------------------------------------------

def manifest_line(case_id, kernel, m, n, k, tile_m, tile_w, operand, reps, emit="none"):
    return "\t".join([
        "case", case_id, kernel, str(m), str(n), str(k),
        str(tile_m), str(tile_w), operand, str(reps), emit,
    ])


def execute(host, lib, lines, work_dir):
    manifest = work_dir / "manifest.tsv"
    manifest.write_text("\n".join(lines) + "\n")
    completed = subprocess.run(
        [str(host), str(lib), str(manifest), str(work_dir)],
        capture_output=True, text=True,
    )
    if completed.returncode != 0:
        sys.stderr.write(completed.stdout + completed.stderr)
        raise SystemExit(f"host exited {completed.returncode}")
    observations = {}
    for line in completed.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        observations[key] = value
    if observations.get("host.status") != "complete":
        raise SystemExit("host did not report host.status=complete")
    return observations


# ---------------------------------------------------------------------------
# validate
# ---------------------------------------------------------------------------

def mode_validate(args, source_dir):
    work_dir = Path(args.work_dir).resolve()
    variants = sweep_variants()
    cells = sweep_cells()

    print("== frozen population, printed before anything runs ==")
    print(f"tiled variants          {len(variants)}  (floor {FROZEN_VARIANT_COUNT})")
    print(f"  square                {len(SQUARE_WIDTHS)}  {SQUARE_WIDTHS}")
    print(f"  rectangular           {len(RECTANGULAR_PAIRS)}  {RECTANGULAR_PAIRS}")
    print(f"sweep cells             {len(cells)}  (floor {FROZEN_CELL_COUNT})")
    print(f"sweep case rows         {len(variants) * len(cells) + len(cells)}"
          f"  ({len(variants)} tiled + 1 direct) x {len(cells)} cells")
    print(f"validation cells        {len(VALIDATION_CELLS)}")

    facts = toolchain_facts()
    print("\n== toolchain, each fact with the invocation that produced it ==")
    print(f"  invocation            {facts['metal_version_invocation']}")
    for key in ("developer_dir_env", "xcode_select_p", "metal_version_as_invoked",
                "xcode", "sdk_version", "sdk_build", "host_cpu", "host_os", "host_arch"):
        print(f"  {key:24s}{facts[key]}")
    print("\n== quiet-host gate, read but NOT enforced in this mode ==")
    print("   validate reads no wall clock, so a loaded host is valid here.")
    admitted, gate_lines, _fields, gate_refusals = quiet_host_gate("validate")
    for line in gate_lines:
        print(line)
    print(f"  verdict                        {'ADMIT' if admitted else 'REFUSE'}"
          f"  ({len(gate_refusals)} refusal(s)); --mode timing would "
          f"{'proceed' if admitted else 'abort'} right now")

    lib, host = build(work_dir, source_dir)
    print(f"\ncompiled: metallib {sha256_file(lib)[:16]}  host {sha256_file(host)[:16]}")

    # Every function is exercised on the validation cells: the 18 sweep
    # variants, the direct reference, the verbatim tiled reference, and the
    # deliberately wrong twin.
    lines = []
    for case_name, m, n, k in VALIDATION_CELLS:
        operand = f"prng:{OPERAND_SEED}"
        lines.append(manifest_line(f"{case_name}__direct", "contract_direct", m, n, k, 0, 0, operand, 0))
        lines.append(manifest_line(f"{case_name}__reference", "contract_tiled_reference", m, n, k, 16, 16, operand, 0))
        for tile_m, tile_w in variants:
            lines.append(manifest_line(
                f"{case_name}__m{tile_m}_w{tile_w}", variant_name(tile_m, tile_w),
                m, n, k, tile_m, tile_w, operand, 0))
    # The signed-zero case, which is the only operand set under which a +0.0
    # accumulator seed is observable at all.
    zc = "z_signed_zero"
    lines.append(manifest_line(f"{zc}__direct", "contract_direct", 16, 256, 1024, 0, 0, SIGNED_ZERO_SOURCE, 0))
    lines.append(manifest_line(f"{zc}__m16_w16", "contract_tiled_m16_w16", 16, 256, 1024, 16, 16, SIGNED_ZERO_SOURCE, 0))
    lines.append(manifest_line(f"{zc}__zero_seed", "contract_tiled_m16_w16_zero_seed", 16, 256, 1024, 16, 16, SIGNED_ZERO_SOURCE, 0))

    obs = execute(host, lib, lines, work_dir)

    prepared = int(obs["environment.prepared_pipeline_count"])
    print(f"\n== check 3: prepared pipelines {prepared} (floor {FROZEN_PIPELINE_COUNT}) ==")
    if prepared != FROZEN_PIPELINE_COUNT:
        raise SystemExit(
            f"population floor: device prepared {prepared} pipelines, the protocol "
            f"freezes {FROZEN_PIPELINE_COUNT}; a variant failed to compile and the "
            f"sweep would have silently shrunk"
        )

    failures = []

    # Check 1 -- byte-identity against the retained kernel.
    print("\n== check 1: (16,16) variant against the verbatim retained kernel ==")
    for case_name, _m, _n, _k in VALIDATION_CELLS:
        ref = obs.get(f"case.{case_name}__reference.result_sha256")
        got = obs.get(f"case.{case_name}__m16_w16.result_sha256")
        ok = ref is not None and ref == got
        print(f"  {case_name:8s} reference {str(ref)[:16]}  m16_w16 {str(got)[:16]}  "
              f"{'identical' if ok else 'DIFFERS'}")
        if not ok:
            failures.append(f"byte-identity failed at {case_name}: {ref} != {got}")

    # Check 2 -- cross-variant oracle.
    print("\n== check 2: every variant against `direct`, bit for bit ==")
    mismatches = 0
    checked = 0
    for case_name, _m, _n, _k in VALIDATION_CELLS:
        expected = obs.get(f"case.{case_name}__direct.result_sha256")
        for tile_m, tile_w in variants:
            got = obs.get(f"case.{case_name}__m{tile_m}_w{tile_w}.result_sha256")
            checked += 1
            if got != expected:
                mismatches += 1
                failures.append(
                    f"oracle failed at {case_name} variant ({tile_m},{tile_w}): {got} != {expected}")
    print(f"  {checked} variant-cell comparisons, {mismatches} mismatched")

    # Unwritten-element floor.
    unwritten_total = sum(
        int(v) for key, v in obs.items() if key.endswith(".unwritten_count")
    )
    print(f"\n== unwritten elements across every case: {unwritten_total} ==")
    if unwritten_total != 0:
        failures.append(f"{unwritten_total} output elements were never written")

    # The oracle must REJECT the deliberately wrong twin.
    print("\n== check 2b: the oracle must reject the deliberately wrong twin ==")
    zd = obs.get(f"case.{zc}__direct.result_sha256")
    zg = obs.get(f"case.{zc}__m16_w16.result_sha256")
    zz = obs.get(f"case.{zc}__zero_seed.result_sha256")
    print(f"  direct              {zd}")
    print(f"  m16_w16             {zg}  {'accepted' if zg == zd else 'REJECTED'}")
    print(f"  m16_w16_zero_seed   {zz}  {'ACCEPTED (bad)' if zz == zd else 'rejected'}")
    if zg != zd:
        failures.append("signed-zero case: the (16,16) variant disagreed with `direct`")
    if zz == zd:
        failures.append(
            "signed-zero case: the +0.0-seeded twin was NOT rejected, so the oracle "
            "cannot separate a strict fold from a seeded one and is demonstrating nothing")

    # Operand custody, reconstructed independently on the validation cells.
    print("\n== operand digests against an independent reconstruction ==")
    for case_name, m, n, k in VALIDATION_CELLS[:3]:
        want_a = prng_digest(OPERAND_SEED, m * k)
        want_b = prng_digest(OPERAND_SEED ^ 0xA5A5A5A5A5A5A5A5, n * k)
        got_a = obs.get(f"case.{case_name}__direct.operand_a_sha256")
        got_b = obs.get(f"case.{case_name}__direct.operand_b_sha256")
        ok = (want_a == got_a) and (want_b == got_b)
        print(f"  {case_name:8s} {'agree' if ok else 'DISAGREE'}")
        if not ok:
            failures.append(f"operand reconstruction disagrees at {case_name}")

    print()
    if failures:
        for f in failures:
            print(f"FAILURE: {f}")
        raise SystemExit(f"validation failed with {len(failures)} failure(s)")
    print("validation passed: population floors met, byte-identity holds, the oracle "
          "accepts every variant and rejects the wrong twin, and no output went unwritten.")
    print("NO WALL CLOCK WAS READ. This run makes no timing claim of any kind.")


# ---------------------------------------------------------------------------
# perturb -- break the subject, show the failure text
# ---------------------------------------------------------------------------

PERTURBATIONS = {
    "wrong-threadgroup": (
        "dispatch contract_tiled_m16_w16 with a threadgroup height of 8, which is "
        "not the height its template was instantiated with"),
    "zero-seed-under-oracle": (
        "submit the +0.0-seeded twin as if it were a sweep variant, under PRNG "
        "operands where the defect is invisible, and then under signed zeros"),
}


def mode_perturb(args, source_dir):
    work_dir = Path(args.work_dir).resolve()
    lib, host = build(work_dir, source_dir)
    name = args.perturbation
    if name not in PERTURBATIONS:
        raise SystemExit(f"unknown perturbation {name}; known: {sorted(PERTURBATIONS)}")
    print(f"== perturbation: {name} ==\n{PERTURBATIONS[name]}\n")

    if name == "wrong-threadgroup":
        m, n, k = 32, 256, 1024
        operand = f"prng:{OPERAND_SEED}"
        lines = [
            manifest_line("p__direct", "contract_direct", m, n, k, 0, 0, operand, 0),
            manifest_line("p__correct", "contract_tiled_m16_w16", m, n, k, 16, 16, operand, 0),
            manifest_line("p__wrong", "contract_tiled_m16_w16", m, n, k, 8, 16, operand, 0),
        ]
        obs = execute(host, lib, lines, work_dir)
        expected = obs["case.p__direct.result_sha256"]
        for tag in ("p__correct", "p__wrong"):
            got = obs.get(f"case.{tag}.result_sha256")
            unwritten = obs.get(f"case.{tag}.unwritten_count")
            verdict = "accepted by the oracle" if got == expected else "REJECTED by the oracle"
            print(f"  {tag:12s} sha {got}  unwritten {unwritten}  {verdict}")
        return

    if name == "zero-seed-under-oracle":
        m, n, k = 16, 256, 1024
        for label, operand in (("prng operands", f"prng:{OPERAND_SEED}"),
                               ("signed-zero operands", SIGNED_ZERO_SOURCE)):
            lines = [
                manifest_line("q__direct", "contract_direct", m, n, k, 0, 0, operand, 0),
                manifest_line("q__zero_seed", "contract_tiled_m16_w16_zero_seed", m, n, k, 16, 16, operand, 0),
            ]
            obs = execute(host, lib, lines, work_dir)
            expected = obs["case.q__direct.result_sha256"]
            got = obs["case.q__zero_seed.result_sha256"]
            verdict = "accepted -- the defect is INVISIBLE here" if got == expected else "REJECTED"
            print(f"  {label:24s} direct {expected[:16]}  twin {got[:16]}  {verdict}")
        return


# ---------------------------------------------------------------------------
# gate -- read the quiet-host gate alone
# ---------------------------------------------------------------------------

def mode_gate(args, source_dir):
    """Read the gate and report, dispatching nothing and reading no wall clock.

    This mode exists so the gate can be exercised -- in both directions -- without
    running the sweep, and so an operator can see WHY a host was refused instead of
    inferring it from an aborted run.
    """
    print("== quiet-host gate ==")
    print(f"   host {run(['hostname', '-s']).stdout.strip()}, "
          f"{run(['sysctl', '-n', 'hw.ncpu']).stdout.strip()} cores, "
          f"{run(['sysctl', '-n', 'machdep.cpu.brand_string']).stdout.strip()}")
    lock = acquire_measurement_lock()
    admitted, lines, _fields, refusals = quiet_host_gate(
        "probe", lock_handle=lock if lock is not None else False)
    for line in lines:
        print(line)
    print()
    if admitted:
        print("verdict: ADMIT -- a timing run may start now.")
        print("NO WALL CLOCK WAS READ. This mode makes no timing claim of any kind.")
        return
    for reason in refusals:
        print(f"REFUSED: {reason}")
    raise SystemExit(
        f"verdict: REFUSE -- {len(refusals)} refusal(s). A timing run started now would "
        f"measure this host's competing work as well as its kernels.")


# ---------------------------------------------------------------------------
# timing
# ---------------------------------------------------------------------------

def mode_timing(args, source_dir):
    work_dir = Path(args.work_dir).resolve()
    out_dir = Path(args.out).resolve()
    variants = sweep_variants()
    cells = sweep_cells()

    lock = acquire_measurement_lock()
    print("== quiet-host gate, at start ==")
    admitted, gate_lines, start_fields, refusals = quiet_host_gate(
        "start", lock_handle=lock if lock is not None else False)
    for line in gate_lines:
        print(line)
    if not admitted:
        for reason in refusals:
            print(f"REFUSED: {reason}")
        raise SystemExit(
            f"quiet-host gate refused at start with {len(refusals)} refusal(s). This is a "
            f"wall-clock measurement and AGENTS.md's idle-host discipline applies; no "
            f"timing is admissible here. Quiesce the host and re-read the gate with "
            f"`--mode gate` rather than re-running until it passes."
        )
    print("  verdict                        ADMIT\n")

    facts = toolchain_facts()
    lib, host = build(work_dir, source_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    # A/B interleaved by manifest order: round-robin over realizations within a
    # cell, never a contiguous block per realization.
    rows = {}
    for round_index in range(args.rounds):
        lines = []
        for case_name, m, n, k in cells:
            operand = f"prng:{OPERAND_SEED}"
            lines.append(manifest_line(
                f"{case_name}__direct__r{round_index}", "contract_direct",
                m, n, k, 0, 0, operand, args.reps))
            for tile_m, tile_w in variants:
                lines.append(manifest_line(
                    f"{case_name}__m{tile_m}_w{tile_w}__r{round_index}",
                    variant_name(tile_m, tile_w), m, n, k, tile_m, tile_w, operand, args.reps))
        obs = execute(host, lib, lines, work_dir)
        for key, value in obs.items():
            rows[key] = value
        print(f"round {round_index} complete")

    print("\n== quiet-host gate, at end ==")
    end_admitted, end_lines, end_fields, end_refusals = quiet_host_gate("end")
    for line in end_lines:
        print(line)

    # Emit the raw and the settled tables. "Settled" is the minimum over rounds
    # 1..N-1; round 0 is reported separately because one warm-up dispatch does
    # not remove a pair's first-encounter cost.
    timing_path = out_dir / "timing.tsv"
    summary_path = out_dir / "timing-summary.tsv"
    with timing_path.open("w") as raw, summary_path.open("w") as summary:
        raw.write("cell\tm\tn\tk\trealization\ttile_m\ttile_w\tround\trep\tmicroseconds\n")
        summary.write("cell\tm\tn\tk\trealization\ttile_m\ttile_w\tstatus\twaste_factor\t"
                      "settled_min_microseconds\tround0_min_microseconds\t"
                      "settled_spread_percent\tuseful_gflop_per_second\tissued_gflop_per_second\t"
                      "result_sha256\toracle\n")
        for case_name, m, n, k in cells:
            expected = None
            for r in range(args.rounds):
                expected = rows.get(f"case.{case_name}__direct__r{r}.result_sha256", expected)
            realizations = [("direct", 0, 0)] + [
                (f"m{tm}_w{tw}", tm, tw) for tm, tw in variants]
            for label, tile_m, tile_w in realizations:
                per_round = []
                status = None
                digest = None
                for r in range(args.rounds):
                    cid = f"{case_name}__{label}__r{r}"
                    status = rows.get(f"case.{cid}.status", status)
                    digest = rows.get(f"case.{cid}.result_sha256", digest)
                    reps = []
                    for rep in range(1, args.reps + 1):
                        v = rows.get(f"case.{cid}.gpu_seconds.{rep}")
                        if v is None:
                            continue
                        micro = float(v) * 1e6
                        reps.append(micro)
                        raw.write(f"{case_name}\t{m}\t{n}\t{k}\t{label}\t{tile_m}\t{tile_w}"
                                  f"\t{r}\t{rep}\t{micro:.3f}\n")
                    per_round.append(min(reps) if reps else None)
                if status != "ok" or not per_round or per_round[0] is None:
                    summary.write(f"{case_name}\t{m}\t{n}\t{k}\t{label}\t{tile_m}\t{tile_w}"
                                  f"\t{status}\t\t\t\t\t\t\t\t\n")
                    continue
                settled = [v for v in per_round[1:] if v is not None]
                best = min(settled) if settled else per_round[0]
                spread = ((max(settled) - min(settled)) / min(settled) * 100.0) if settled else 0.0
                useful = 2.0 * m * n * k
                waste = 1.0 if tile_m == 0 else (-(-m // tile_m)) * tile_m / m
                oracle = "match" if digest == expected else "MISMATCH"
                summary.write(
                    f"{case_name}\t{m}\t{n}\t{k}\t{label}\t{tile_m}\t{tile_w}\tok\t{waste:.4f}\t"
                    f"{best:.3f}\t{per_round[0]:.3f}\t{spread:.2f}\t"
                    f"{useful / best / 1e3:.1f}\t{useful * waste / best / 1e3:.1f}\t"
                    f"{digest}\t{oracle}\n")

    env_path = out_dir / "environment.tsv"
    with env_path.open("w") as env:
        env.write("key\tvalue\n")
        for key, value in sorted(facts.items()):
            env.write(f"{key}\t{value}\n")
        for key, value in sorted(rows.items()):
            if key.startswith("environment.") or key.startswith("pipeline."):
                env.write(f"{key}\t{value}\n")
        for key, value in sorted(start_fields.items()):
            env.write(f"{key}\t{value}\n")
        for key, value in sorted(end_fields.items()):
            env.write(f"{key}\t{value}\n")
        env.write(f"gate.cpu_idle_floor_percent\t{GATE_CPU_IDLE_FLOOR_PERCENT}\n")
        env.write(f"gate.gpu_utilization_ceiling_percent\t{GATE_GPU_UTILIZATION_CEILING_PERCENT}\n")
        env.write(f"gate.load_ceiling_1min\t{GATE_LOAD_CEILING}\n")
        env.write(f"gate.load_recorded_idle_baseline\t{GATE_LOAD_BASELINE}\n")
        env.write(f"timing_rounds\t{args.rounds}\n")
        env.write(f"timing_reps_per_round\t{args.reps}\n")
        env.write(f"kernels_sha256\t{sha256_file(source_dir / 'kernels.metal')}\n")
        env.write(f"host_source_sha256\t{sha256_file(source_dir / 'host.m')}\n")
        env.write(f"driver_sha256\t{sha256_file(source_dir / 'tile_width_sweep.py')}\n")

    manifest_path = out_dir / "manifest.tsv"
    with manifest_path.open("w") as man:
        man.write("key\tvalue\n")
        for producer in ("kernels.metal", "host.m", "tile_width_sweep.py"):
            man.write(f"producer.sha256.{producer}\t{sha256_file(source_dir / producer)}\n")
        for result in (env_path, summary_path, timing_path):
            man.write(f"result.sha256.{result.name}\t{sha256_file(result)}\n")

    print(f"\nwrote {timing_path}\n      {summary_path}\n      {env_path}\n      {manifest_path}")
    if not end_admitted:
        for reason in end_refusals:
            print(f"REFUSED: {reason}")
        raise SystemExit(
            f"quiet-host gate refused at end with {len(end_refusals)} refusal(s): the host "
            f"stopped being quiet during the run. The tables were written so the run is "
            f"inspectable, but no timing claim may rest on them."
        )
    print("  verdict                        ADMIT")


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mode", required=True,
                        choices=("validate", "timing", "gate", "perturb"))
    parser.add_argument("--work-dir", default="work")
    parser.add_argument("--out", default="results-timing")
    parser.add_argument("--rounds", type=int, default=5)
    parser.add_argument("--reps", type=int, default=7)
    parser.add_argument("--perturbation", default="wrong-threadgroup")
    args = parser.parse_args()

    source_dir = Path(__file__).resolve().parent
    if shutil.which("xcrun") is None:
        raise SystemExit("xcrun not found; this harness needs an Xcode command-line toolchain")

    if args.mode == "validate":
        mode_validate(args, source_dir)
    elif args.mode == "gate":
        mode_gate(args, source_dir)
    elif args.mode == "perturb":
        mode_perturb(args, source_dir)
    else:
        mode_timing(args, source_dir)


if __name__ == "__main__":
    main()
