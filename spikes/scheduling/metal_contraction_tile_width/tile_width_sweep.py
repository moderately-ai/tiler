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
  timing    The frozen sweep. Requires an idle host; the caller is responsible
            for the load gate, which this driver checks and refuses.
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
import hashlib
import os
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

LOAD_GATE = 0.5


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
    """One-minute load average, for the gate a timing run needs."""
    return os.getloadavg()[0]


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
    print(f"  load average (1 min)  {load_average():.2f}  (timing gate is < {LOAD_GATE})")

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
# timing
# ---------------------------------------------------------------------------

def mode_timing(args, source_dir):
    work_dir = Path(args.work_dir).resolve()
    out_dir = Path(args.out).resolve()
    variants = sweep_variants()
    cells = sweep_cells()

    start_load = load_average()
    if start_load >= LOAD_GATE:
        raise SystemExit(
            f"load gate: one-minute load average is {start_load:.2f}, the protocol "
            f"requires < {LOAD_GATE} at start. This is a wall-clock measurement and "
            f"AGENTS.md's idle-host discipline applies; no timing is admissible here."
        )

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

    end_load = load_average()

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
        env.write(f"load_average_1min_start\t{start_load:.2f}\n")
        env.write(f"load_average_1min_end\t{end_load:.2f}\n")
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
    if end_load >= LOAD_GATE:
        raise SystemExit(
            f"load gate: one-minute load average rose to {end_load:.2f} by the end of "
            f"the run, above the {LOAD_GATE} the protocol requires. The tables were "
            f"written so the run is inspectable, but no timing claim may rest on them."
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mode", required=True, choices=("validate", "timing", "perturb"))
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
    elif args.mode == "perturb":
        mode_perturb(args, source_dir)
    else:
        mode_timing(args, source_dir)


if __name__ == "__main__":
    main()
