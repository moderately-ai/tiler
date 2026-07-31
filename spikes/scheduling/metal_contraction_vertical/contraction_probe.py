#!/usr/bin/env python3
"""Bounded probe for the first Metal realization of the workload's contraction.

Three legs, selected by ``--mode``:

``semantics``
    Sixteen-wide designed cases whose exact result under every named reduction
    topology is computed here in exact rational arithmetic and rounded once to
    binary32. Each realization is compared against the topology it *declares*,
    and against every other named candidate, so a disagreement says which
    evaluation the device performed rather than only that it disagreed. Standard
    library only.

``workload``
    The six structure-1 cells the pinned workload actually contains, compared
    element by element against a binary32 strict left fold computed on the host.
    Needs ``numpy``.

``timing``
    A/B interleaved GPU timings of the surviving realizations. No oracle, no
    third-party package, so it runs on a bench host with a stock interpreter.

Nothing here compiles a Tiler program, registers an operation, or plans
anything. It measures what a hand-written realization of one index structure
delivers on one device.
"""

from __future__ import annotations

import argparse
import hashlib
import math
import os
import shutil
import struct
import subprocess
import sys
import tempfile
from fractions import Fraction
from pathlib import Path

SPIKE_DIR = Path(__file__).resolve().parent

# The governed Apple9/F32 baseline the workload profile records, quoted here as
# a single list so a record can be read against the flags that produced it.
OFFLINE_FLAGS = [
    "-target",
    "air64-apple-macos26.0",
    "-std=metal4.0",
    "-O2",
    "-fmetal-math-mode=safe",
    "-fmetal-math-fp32-functions=precise",
    "-ffp-contract=off",
]

# Realization identifiers, and the reduction topology each one claims. A
# realization whose declared topology is None makes no claim this probe can
# check; that is a property of the realization, not a gap in the probe.
REALIZATIONS = {
    "direct": ("contract_direct", "strict_fold"),
    "direct_zero_seed": ("contract_direct_zero_seed", "zero_seed_fold"),
    "tiled": ("contract_tiled", "strict_fold"),
    "ksplit_contiguous": ("contract_ksplit_contiguous", "contiguous_split"),
    "ksplit_strided": ("contract_ksplit_strided", "strided_split"),
    "simdgroup": ("contract_simdgroup", None),
    "opaque_mps": ("mps", None),
}

DELIVERY_CANDIDATES = ["direct", "tiled", "ksplit_contiguous", "ksplit_strided", "simdgroup", "opaque_mps"]

# ---------------------------------------------------------------------------
# Exact binary32 arithmetic.
# ---------------------------------------------------------------------------

F32_MIN_NORMAL = Fraction(2) ** -126
F32_MIN_SUBNORMAL = Fraction(2) ** -149
F32_OVERFLOW_BOUND = Fraction(2) ** 128
CANONICAL_NAN_BITS = 0x7FC00000


def bits_of(value: float) -> int:
    return struct.unpack("<I", struct.pack("<f", value))[0]


def float_of(bits: int) -> float:
    return struct.unpack("<f", struct.pack("<I", bits & 0xFFFFFFFF))[0]


def round_to_f32(value: Fraction) -> float:
    """Round an exact rational to the nearest binary32, ties to even.

    Written out rather than routed through ``float()`` because rational to
    binary64 to binary32 is a double rounding, and a double rounding is exactly
    the kind of silent difference this probe exists to detect in other people's
    code.
    """
    if value == 0:
        return 0.0
    sign = -1 if value < 0 else 1
    magnitude = abs(value)

    exponent = magnitude.numerator.bit_length() - magnitude.denominator.bit_length()
    while Fraction(2) ** exponent > magnitude:
        exponent -= 1
    while Fraction(2) ** (exponent + 1) <= magnitude:
        exponent += 1

    quantum_exponent = max(exponent - 23, -149)
    quantum = Fraction(2) ** quantum_exponent
    scaled = magnitude / quantum
    units = scaled.numerator // scaled.denominator
    remainder = scaled - units
    if remainder > Fraction(1, 2) or (remainder == Fraction(1, 2) and units % 2 == 1):
        units += 1
    rounded = Fraction(units) * quantum
    if rounded >= F32_OVERFLOW_BOUND:
        return sign * math.inf
    return float(sign * rounded)


def flush(value: float, enabled: bool) -> float:
    """Sign-preserving flush to zero of a subnormal, per the declared realization."""
    if not enabled or value == 0.0 or math.isnan(value) or math.isinf(value):
        return value
    if abs(Fraction(value)) < F32_MIN_NORMAL:
        return math.copysign(0.0, value)
    return value


def f32_binary(left: float, right: float, exact, ftz: bool) -> float:
    left = flush(left, ftz)
    right = flush(right, ftz)
    if math.isnan(left) or math.isnan(right):
        return float_of(CANONICAL_NAN_BITS)
    try:
        result = round_to_f32(exact(Fraction(left), Fraction(right)))
    except (OverflowError, ValueError, ZeroDivisionError):
        return float_of(CANONICAL_NAN_BITS)
    return flush(result, ftz)


def f32_mul(left: float, right: float, ftz: bool = False) -> float:
    if math.isnan(left) or math.isnan(right):
        return float_of(CANONICAL_NAN_BITS)
    if math.isinf(left) or math.isinf(right):
        if left == 0.0 or right == 0.0:
            return float_of(CANONICAL_NAN_BITS)
        return math.copysign(math.inf, math.copysign(1.0, left) * math.copysign(1.0, right))
    # A zero product carries the sign of the operands. The rational path cannot
    # express that — Fraction has one zero — and the `negative_zero_seed` case
    # exists precisely because that sign is observable.
    if left == 0.0 or right == 0.0:
        return math.copysign(0.0, math.copysign(1.0, left) * math.copysign(1.0, right))
    return f32_binary(left, right, lambda a, b: a * b, ftz)


def f32_add(left: float, right: float, ftz: bool = False) -> float:
    if math.isinf(left) or math.isinf(right):
        if math.isinf(left) and math.isinf(right) and left != right:
            return float_of(CANONICAL_NAN_BITS)
        return left if math.isinf(left) else right
    if math.isnan(left) or math.isnan(right):
        return float_of(CANONICAL_NAN_BITS)
    # Signed zero: (-0.0) + (-0.0) is -0.0; every other zero sum is +0.0 under
    # round to nearest. The rational path cannot express that, so it is stated.
    if left == 0.0 and right == 0.0:
        both_negative = math.copysign(1.0, left) < 0 and math.copysign(1.0, right) < 0
        return -0.0 if both_negative else 0.0
    return f32_binary(left, right, lambda a, b: a + b, ftz)


def f32_fma(product_left: float, product_right: float, addend: float, ftz: bool = False) -> float:
    """One rounding after ``a * b + c`` — the shape ADR 0015's permission governs."""
    product_left = flush(product_left, ftz)
    product_right = flush(product_right, ftz)
    addend = flush(addend, ftz)
    if any(math.isnan(v) for v in (product_left, product_right, addend)):
        return float_of(CANONICAL_NAN_BITS)
    if math.isinf(product_left) or math.isinf(product_right) or math.isinf(addend):
        return f32_add(f32_mul(product_left, product_right, ftz), addend, ftz)
    if product_left == 0.0 or product_right == 0.0:
        # The product is exactly zero, so the fused step introduces no rounding
        # and the result is the signed sum of two zeros or of a zero and the
        # addend. Routing it through the rational path would drop the product's
        # zero sign, which is the whole subject of the `negative_zero_seed` case.
        signs = math.copysign(1.0, product_left) * math.copysign(1.0, product_right)
        return f32_add(math.copysign(0.0, signs), addend, ftz)
    exact = Fraction(product_left) * Fraction(product_right) + Fraction(addend)
    if exact == 0:
        # Exact cancellation of two nonzero terms is +0.0 under round to nearest.
        return 0.0
    return flush(round_to_f32(exact), ftz)


def exact_products(left, right, ftz: bool):
    return [f32_mul(a, b, ftz) for a, b in zip(left, right)]


# ---------------------------------------------------------------------------
# Named reduction topologies. Each returns the exact binary32 bits a realization
# claiming that topology must produce.
# ---------------------------------------------------------------------------


def fold(values, ftz: bool) -> float:
    accumulator = values[0]
    for value in values[1:]:
        accumulator = f32_add(accumulator, value, ftz)
    return accumulator


def topology_strict_fold(left, right, split, ftz):
    return fold(exact_products(left, right, ftz), ftz)


def topology_zero_seed_fold(left, right, split, ftz):
    accumulator = 0.0
    for product in exact_products(left, right, ftz):
        accumulator = f32_add(accumulator, product, ftz)
    return accumulator


def topology_fma_fold(left, right, split, ftz):
    accumulator = f32_mul(left[0], right[0], ftz)
    for a, b in zip(left[1:], right[1:]):
        accumulator = f32_fma(a, b, accumulator, ftz)
    return accumulator


def topology_fma_zero_seed_fold(left, right, split, ftz):
    """A fused left fold whose accumulator starts at +0.0.

    This is what a matrix-multiply-accumulate instruction over a zero-filled
    accumulator computes, so it is the topology a `simdgroup_float8x8` chain
    would take if it walks the contracted axis in order. Under
    `docs/numerical-semantics.md` a +0.0 start is not a defect on its own — it is
    a reduction carrying an explicit `initial` seed, which is a different
    semantic operation from the unseeded one and must be declared as such.
    """
    accumulator = 0.0
    for a, b in zip(left, right):
        accumulator = f32_fma(a, b, accumulator, ftz)
    return accumulator


def topology_contiguous_split(left, right, split, ftz):
    products = exact_products(left, right, ftz)
    span = len(products) // split
    partials = [fold(products[lane * span : (lane + 1) * span], ftz) for lane in range(split)]
    return fold(partials, ftz)


def topology_strided_split(left, right, split, ftz):
    products = exact_products(left, right, ftz)
    partials = [fold(products[lane :: split], ftz) for lane in range(split)]
    return fold(partials, ftz)


def topology_exact_products_then_round(left, right, split, ftz):
    products = exact_products(left, right, ftz)
    if any(math.isnan(p) for p in products):
        return float_of(CANONICAL_NAN_BITS)
    if any(math.isinf(p) for p in products):
        return fold(products, ftz)
    total = sum((Fraction(p) for p in products), Fraction(0))
    if total == 0:
        return fold(products, ftz)
    return flush(round_to_f32(total), ftz)


def topology_exact_dot_then_round(left, right, split, ftz):
    if ftz:
        left = [flush(v, True) for v in left]
        right = [flush(v, True) for v in right]
    if any(math.isnan(v) for v in list(left) + list(right)):
        return float_of(CANONICAL_NAN_BITS)
    if any(math.isinf(v) for v in list(left) + list(right)):
        return fold(exact_products(left, right, ftz), ftz)
    total = sum((Fraction(a) * Fraction(b) for a, b in zip(left, right)), Fraction(0))
    if total == 0:
        return fold(exact_products(left, right, ftz), ftz)
    return flush(round_to_f32(total), ftz)


def topology_reversed_fold(left, right, split, ftz):
    """The strict fold over the reversed contributor sequence.

    A reversal is a permutation, so it is a legal evaluation only under the
    permutation permission. It is modelled so that agreement with the ascending
    fold is a refutation of the descending one rather than an untested
    assumption about which direction a device walks the contracted axis.
    """
    return fold(list(reversed(exact_products(left, right, ftz))), ftz)


def topology_reversed_fma_fold(left, right, split, ftz):
    pairs = list(zip(left, right))[::-1]
    accumulator = f32_mul(pairs[0][0], pairs[0][1], ftz)
    for a, b in pairs[1:]:
        accumulator = f32_fma(a, b, accumulator, ftz)
    return accumulator


def topology_payload_propagating_fold(left, right, split, ftz):
    """A strict fold whose arithmetic NaN keeps the first NaN contributor's exact
    payload rather than canonicalizing it.

    It exists so that the canonical `tiler::canonical-arithmetic-nan-f32@1`
    pattern is a *distinguished* observation instead of the only behaviour the
    probe's own model can express. Where no operand is already NaN it coincides
    with the strict fold by construction.
    """
    for a, b in zip(left, right):
        if math.isnan(a):
            return a
        if math.isnan(b):
            return b
    return topology_strict_fold(left, right, split, ftz)


TOPOLOGIES = {
    "strict_fold": topology_strict_fold,
    "reversed_fold": topology_reversed_fold,
    "reversed_fma_fold": topology_reversed_fma_fold,
    "fma_zero_seed_fold": topology_fma_zero_seed_fold,
    "payload_propagating_fold": topology_payload_propagating_fold,
    "zero_seed_fold": topology_zero_seed_fold,
    "fma_fold": topology_fma_fold,
    "contiguous_split": topology_contiguous_split,
    "strided_split": topology_strided_split,
    "exact_products_then_round": topology_exact_products_then_round,
    "exact_dot_then_round": topology_exact_dot_then_round,
}


# ---------------------------------------------------------------------------
# Semantic corpus. Every case is 16x16x16 so that every realization's structural
# precondition is satisfied and one dispatch answers the same question of all of
# them. Only C[0, 0] carries the designed dot; every other operand entry is +0.0.
# ---------------------------------------------------------------------------

SEMANTIC_EXTENT = 16
SEMANTIC_SPLIT = 4


def semantic_cases():
    def pad(values):
        return list(values) + [0.0] * (SEMANTIC_EXTENT - len(values))

    smallest_normal = float_of(0x00800000)
    contraction_probe_operand = float_of(0x3EB97EF9)

    cases = {}

    # Execution witness. Every legal evaluation returns exactly 6.0, so a
    # realization that fails it has its whole column ruled inadmissible.
    cases["witness"] = (pad([2.0]), pad([3.0]), "every topology agrees; 2 * 3 = 6 exactly")

    # Order sensitivity. One large contributor absorbs small ones under a left
    # fold and does not under a wider or regrouped accumulation.
    big = float(2**24)
    cases["order_absorption"] = (
        pad([big] + [1.5] * 7),
        pad([1.0] * 8),
        "a 2^24 leading contributor absorbs 1.5 increments under a left fold",
    )

    # Contraction, in ADR 0015's sense. The operand and scale are the ones the
    # Apple numerical-behaviour record already measured fusing, and the order
    # matters: the accumulator has to already hold 1.0 when the inexact product
    # arrives, because that is the step a fused multiply-add would collapse. With
    # the inexact product first, the fused and unfused values coincide and the
    # case would report a fusion-free device whatever the device did.
    cases["contraction_pair"] = (
        pad([1.0, contraction_probe_operand]),
        pad([1.0, 1.5]),
        "accumulator 1.0 then an inexact product: fl(1 + fl(x*1.5)) against fl(1 + x*1.5)",
    )

    # Reassociation against permutation. `order_absorption` does not separate the
    # two split topologies — both return the same bits there — so this vector was
    # searched for until the strict fold, the contiguous split, the strided
    # split, a fused fold, and an infinitely wide accumulator over rounded
    # products all take pairwise distinct values.
    split_left = [
        0xAE8FCC10, 0xBEF8CE2C, 0x31198E79, 0xC00921CA, 0x3D9D1929, 0x44007FCA, 0x41B67583, 0xBB63D3C2,
        0x4328602B, 0x3CB35A07, 0x2D941111, 0xBAB1DBDB, 0x44ABA077, 0x394A4E66, 0x3123AC20, 0xB6037546,
    ]
    split_right = [
        0xB5D89C01, 0xB26D6247, 0x392972F3, 0x3A8BEE84, 0x30A88108, 0x348CEF8B, 0x35B65100, 0x440B2B3F,
        0xB42CEA8A, 0xAF128D46, 0xC6135504, 0xC4B244A6, 0xB43B1F42, 0x360DD62A, 0xC5B587E5, 0xB2F04F93,
    ]
    cases["split_topology"] = (
        [float_of(bits) for bits in split_left],
        [float_of(bits) for bits in split_right],
        "a searched vector separating strict, contiguous-split, strided-split, fused, and wide accumulation",
    )

    # Signed zero on the accumulator seed. Every product is -0.0, so a fold
    # seeded from the first contributor returns -0.0 and one seeded at +0.0
    # returns +0.0.
    cases["negative_zero_seed"] = (
        [-1.0] * SEMANTIC_EXTENT,
        [0.0] * SEMANTIC_EXTENT,
        "all contributors are -0.0; the accumulator seed is the only difference",
    )

    # NaN payload. The governed profile canonicalizes an arithmetic NaN to
    # 0x7fc00000; this asks what each realization actually returns.
    cases["nan_payload"] = (
        pad([float_of(0x7FC0DEAD), 1.0]),
        pad([1.0, 1.0]),
        "a non-canonical quiet NaN payload enters as a contributor",
    )

    # Infinity times zero, which forms a NaN inside the reduction rather than
    # receiving one.
    cases["infinity_times_zero"] = (
        pad([math.inf, 1.0]),
        pad([0.0, 1.0]),
        "the reduction forms inf * 0 rather than being handed a NaN",
    )

    # Subnormal result. The exact product is 2^-127; the declared Apple9/F32
    # realization flushes it to zero, so the preserving candidates must not match
    # and the flushing ones must.
    cases["subnormal_product"] = (
        pad([smallest_normal]),
        pad([0.5]),
        "the exact product 2^-127 is subnormal",
    )

    return cases


def build_semantic_operands(left_vector, right_vector):
    left = [0.0] * (SEMANTIC_EXTENT * SEMANTIC_EXTENT)
    right = [0.0] * (SEMANTIC_EXTENT * SEMANTIC_EXTENT)
    for index in range(SEMANTIC_EXTENT):
        left[index] = left_vector[index]
        right[index] = right_vector[index]
    return left, right


# ---------------------------------------------------------------------------
# Workload cells. Structure 1 (`td,od->to`) at the extents the pinned
# Qwen/Qwen3-0.6B-Base workload contains. M is the new-position count T, N is
# D_out, K is D_in. Every one of L1's six weight shape classes appears.
# ---------------------------------------------------------------------------

WORKLOAD_CELLS = [
    # id,           M,   N,      K,    weight class,       workload role
    ("w_decode_kv", 1, 1024, 1024, "[1024, 1024]", "decode k_proj / v_proj, T=1"),
    ("w_prefill_q", 10, 2048, 1024, "[2048, 1024]", "C1 prefill q_proj, T=10"),
    ("w_prefill_mlp_in", 128, 3072, 1024, "[3072, 1024]", "B1-a prefill gate_proj / up_proj"),
    ("w_prefill_mlp_out", 128, 1024, 3072, "[1024, 3072]", "B1-a prefill down_proj"),
    ("w_prefill_o", 128, 1024, 2048, "[1024, 2048]", "B1-a prefill o_proj"),
    ("w_vocab_slice", 1, 8192, 1024, "[151936, 1024] slice", "decode vocabulary projection, first 8192 rows"),
]

TIMING_CELLS = WORKLOAD_CELLS + [
    ("t_vocab_full", 1, 151936, 1024, "[151936, 1024]", "decode vocabulary projection, complete"),
    ("t_prefill_mlp_512", 512, 3072, 1024, "[3072, 1024]", "B1-b prefill gate_proj / up_proj"),
]

WORKLOAD_SPLIT = 32
WORKLOAD_SEED = 0x5445524D


# ---------------------------------------------------------------------------
# Operand stream, mirroring `host.m` exactly.
# ---------------------------------------------------------------------------

MASK64 = (1 << 64) - 1


def splitmix64(value: int) -> int:
    value = (value + 0x9E3779B97F4A7C15) & MASK64
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & MASK64
    return value ^ (value >> 31)


def prng_value(seed: int, index: int) -> float:
    bits = splitmix64((seed + index * 0x2545F4914F6CDD1D) & MASK64)
    magnitude = ((bits >> 40) & 0xFFFFFF) - 8388608
    return magnitude * (1.0 / 16777216.0)


def prng_array(seed: int, count: int):
    import numpy

    index = numpy.arange(count, dtype=numpy.uint64)
    value = (numpy.uint64(seed) + index * numpy.uint64(0x2545F4914F6CDD1D)) & numpy.uint64(MASK64)
    value = (value + numpy.uint64(0x9E3779B97F4A7C15)) & numpy.uint64(MASK64)
    value = ((value ^ (value >> numpy.uint64(30))) * numpy.uint64(0xBF58476D1CE4E5B9)) & numpy.uint64(MASK64)
    value = ((value ^ (value >> numpy.uint64(27))) * numpy.uint64(0x94D049BB133111EB)) & numpy.uint64(MASK64)
    value = value ^ (value >> numpy.uint64(31))
    magnitude = ((value >> numpy.uint64(40)) & numpy.uint64(0xFFFFFF)).astype(numpy.int64) - 8388608
    return (magnitude.astype(numpy.float32)) * numpy.float32(1.0 / 16777216.0)


# ---------------------------------------------------------------------------
# Toolchain plumbing.
# ---------------------------------------------------------------------------


def run(command, **kwargs):
    return subprocess.run(command, check=True, capture_output=True, text=True, **kwargs)


def build(work_dir: Path):
    metallib = work_dir / "kernels.metallib"
    air = work_dir / "kernels.air"
    run(["xcrun", "--sdk", "macosx", "metal", *OFFLINE_FLAGS, "-c", str(SPIKE_DIR / "kernels.metal"), "-o", str(air)])
    run(["xcrun", "--sdk", "macosx", "metallib", str(air), "-o", str(metallib)])
    host = work_dir / "contraction_host"
    run(
        [
            "xcrun",
            "--sdk",
            "macosx",
            "clang",
            "-fobjc-arc",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-O2",
            "-framework",
            "Foundation",
            "-framework",
            "Metal",
            "-framework",
            "MetalPerformanceShaders",
            str(SPIKE_DIR / "host.m"),
            "-o",
            str(host),
        ]
    )
    return metallib, host


def dispatch(host: Path, metallib: Path, manifest_lines, work_dir: Path):
    manifest = work_dir / "manifest.tsv"
    manifest.write_text("\n".join(manifest_lines) + "\n", encoding="utf-8")
    completed = subprocess.run(
        [str(host), str(metallib), str(manifest), str(work_dir)],
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        sys.stderr.write(completed.stderr)
        raise SystemExit(f"dispatch host exited {completed.returncode}")
    rows = {}
    for line in completed.stdout.splitlines():
        if "=" in line:
            key, _, value = line.partition("=")
            rows[key] = value
    if rows.get("host.status") != "complete":
        raise SystemExit("dispatch host did not report completion")
    return rows


def digest_of_floats(values) -> str:
    return hashlib.sha256(struct.pack(f"<{len(values)}f", *values)).hexdigest()


def environment_rows(extra=None):
    def capture(command):
        try:
            return subprocess.run(command, capture_output=True, text=True, check=True).stdout.strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            return "unavailable"

    rows = {
        "probe_date": os.environ.get("TILER_PROBE_DATE", ""),
        "host_os": f"{capture(['sw_vers', '-productVersion'])} {capture(['sw_vers', '-buildVersion'])}",
        "host_arch": capture(["uname", "-m"]),
        "host_name": capture(["hostname"]),
        "xcode": capture(["xcodebuild", "-version"]).replace("\n", ";"),
        "offline_compiler": capture(["xcrun", "--sdk", "macosx", "metal", "--version"]).splitlines()[0]
        if capture(["xcrun", "--sdk", "macosx", "metal", "--version"]) != "unavailable"
        else "unavailable",
        "macos_sdk_version": capture(["xcrun", "--sdk", "macosx", "--show-sdk-version"]),
        "macos_sdk_build": capture(["xcrun", "--sdk", "macosx", "--show-sdk-build-version"]),
        "offline_flags": " ".join(OFFLINE_FLAGS),
        "kernels_sha256": hashlib.sha256((SPIKE_DIR / "kernels.metal").read_bytes()).hexdigest(),
        "host_source_sha256": hashlib.sha256((SPIKE_DIR / "host.m").read_bytes()).hexdigest(),
        "probe_sha256": hashlib.sha256(Path(__file__).resolve().read_bytes()).hexdigest(),
        "python_version": sys.version.split()[0],
    }
    if not rows["probe_date"]:
        rows["probe_date"] = capture(["date", "-u", "+%Y-%m-%d"])
    rows.update(extra or {})
    return rows


def write_tsv(path: Path, header, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        handle.write("\t".join(header) + "\n")
        for row in rows:
            handle.write("\t".join(str(field) for field in row) + "\n")


def write_key_value_tsv(path: Path, mapping):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        handle.write("key\tvalue\n")
        for key in sorted(mapping):
            handle.write(f"{key}\t{mapping[key]}\n")


# ---------------------------------------------------------------------------
# Leg 1 — semantics.
# ---------------------------------------------------------------------------


def check_rounding_self_consistency():
    """Prove `round_to_f32` reproduces binary32 before anything rests on it.

    The vectors are chosen at rounding boundaries: a tie that must go to even, a
    tie that must go away from odd, an overflow, and the subnormal quantum.
    """
    checks = [
        (Fraction(1), 0x3F800000),
        (Fraction(2**24) + Fraction(1), 0x4B800000),  # tie, rounds to even -> 2^24
        (Fraction(2**24) + Fraction(3), 0x4B800002),  # tie, rounds to even -> 2^24 + 4
        (Fraction(1, 3), 0x3EAAAAAB),
        (Fraction(2) ** -127, 0x00400000),  # subnormal
        (Fraction(2) ** -149, 0x00000001),  # smallest subnormal
        (Fraction(2) ** -150, 0x00000000),  # tie at the subnormal quantum -> even -> zero
        (Fraction(2) ** 200, 0x7F800000),  # overflow
        (-Fraction(1, 3), 0xBEAAAAAB),
    ]
    failures = []
    for value, expected in checks:
        observed = bits_of(round_to_f32(value))
        if observed != expected:
            failures.append((str(value), f"{expected:08x}", f"{observed:08x}"))
    return failures


def semantic_leg(host, metallib, work_dir, out_dir):
    cases = semantic_cases()

    rounding_failures = check_rounding_self_consistency()
    if rounding_failures:
        for value, expected, observed in rounding_failures:
            sys.stderr.write(f"round_to_f32({value}) expected {expected} observed {observed}\n")
        raise SystemExit("binary32 rounding self-check failed; no classification is admissible")

    manifest = []
    operand_files = {}
    for case_id, (left_vector, right_vector, _) in cases.items():
        left, right = build_semantic_operands(left_vector, right_vector)
        left_path = work_dir / f"{case_id}.a.bin"
        right_path = work_dir / f"{case_id}.b.bin"
        left_path.write_bytes(struct.pack(f"<{len(left)}f", *left))
        right_path.write_bytes(struct.pack(f"<{len(right)}f", *right))
        operand_files[case_id] = (left_path, right_path, left, right)
        for realization, (kernel, _) in REALIZATIONS.items():
            manifest.append(
                "\t".join(
                    [
                        "case",
                        f"{case_id}.{realization}",
                        kernel,
                        str(SEMANTIC_EXTENT),
                        str(SEMANTIC_EXTENT),
                        str(SEMANTIC_EXTENT),
                        str(SEMANTIC_SPLIT),
                        f"file:{left_path},{right_path}",
                        "0",
                        "full",
                    ]
                )
            )

    rows = dispatch(host, metallib, manifest, work_dir)

    candidate_rows = []
    observation_rows = []
    # realization -> {topology label -> first case that refutes it}, built as the
    # cases are walked. Attribution is a corpus-level claim rather than a
    # per-case one: no single vector separates every named topology from every
    # other, and pretending one does is how a coincidence gets read as a match.
    refutations = {realization: {} for realization in REALIZATIONS}
    observed_cases = {realization: 0 for realization in REALIZATIONS}
    for case_id, (left_vector, right_vector, description) in cases.items():
        _, _, left, right = operand_files[case_id]
        candidates = {}
        for ftz in (False, True):
            for name, function in TOPOLOGIES.items():
                label = f"{name}+ftz" if ftz else name
                candidates[label] = bits_of(function(left_vector, right_vector, SEMANTIC_SPLIT, ftz))

        distinct = len(set(candidates.values()))
        for label in sorted(candidates):
            candidate_rows.append([case_id, label, f"{candidates[label]:08x}", description])

        for realization, (_, declared) in REALIZATIONS.items():
            key_prefix = f"case.{case_id}.{realization}"
            status = rows.get(f"{key_prefix}.status", "absent")

            # Whether agreement with the declared topology says anything on this
            # case. If some other named topology takes the same value here, the
            # agreement is consistent with the declaration and does not
            # establish it, and the record must say which of the two it is.
            if declared is None:
                separation = "n/a"
            else:
                declared_values = {candidates[declared], candidates[f"{declared}+ftz"]}
                others = {
                    value
                    for label, value in candidates.items()
                    if label not in (declared, f"{declared}+ftz")
                }
                separation = "distinguishing" if declared_values.isdisjoint(others) else "non-distinguishing"

            if status != "ok":
                observation_rows.append([case_id, realization, status, "", declared or "none", "", "", distinct, separation])
                continue
            results = rows[f"{key_prefix}.results"].split(",")
            observed = int(results[0], 16)
            unwritten = int(rows[f"{key_prefix}.unwritten_count"])
            matched = sorted(label for label, value in candidates.items() if value == observed)
            if unwritten == 0:
                observed_cases[realization] += 1
                for label, value in candidates.items():
                    if value != observed and label not in refutations[realization]:
                        refutations[realization][label] = case_id
            if declared is None:
                verdict = "no-declared-topology"
            else:
                declared_labels = [declared, f"{declared}+ftz"]
                verdict = "agrees-with-declared" if any(
                    candidates[label] == observed for label in declared_labels
                ) else "disagrees-with-declared"
            if unwritten > 0:
                verdict = f"inadmissible-unwritten-{unwritten}"
            observation_rows.append(
                [
                    case_id,
                    realization,
                    "ok",
                    f"{observed:08x}",
                    declared or "none",
                    verdict,
                    ",".join(matched) if matched else "unattributed",
                    distinct,
                    separation,
                ]
            )

    write_tsv(
        out_dir / "semantics-candidates.tsv",
        ["case", "candidate", "bits", "case_description"],
        candidate_rows,
    )
    write_tsv(
        out_dir / "semantics-observations.tsv",
        [
            "case",
            "realization",
            "status",
            "observed_bits",
            "declared_topology",
            "verdict",
            "matched_candidates",
            "distinct_candidate_values",
            "declared_separation",
        ],
        observation_rows,
    )

    all_labels = sorted(set().union(*(set(candidates_for) for candidates_for in [
        {f"{name}{suffix}" for name in TOPOLOGIES for suffix in ("", "+ftz")}
    ])))
    attribution_rows = []
    for realization, (_, declared) in REALIZATIONS.items():
        surviving = [label for label in all_labels if label not in refutations[realization]]
        for label in all_labels:
            attribution_rows.append(
                [
                    realization,
                    label,
                    "consistent" if label in surviving else "refuted",
                    refutations[realization].get(label, ""),
                    observed_cases[realization],
                    declared or "none",
                ]
            )
    write_tsv(
        out_dir / "semantics-attribution.tsv",
        ["realization", "topology", "outcome", "refuting_case", "admissible_cases", "declared_topology"],
        attribution_rows,
    )
    return rows


# ---------------------------------------------------------------------------
# Leg 2 — workload shapes against a host strict fold.
# ---------------------------------------------------------------------------


def monotone_order(bits):
    """Map binary32 bit patterns to a sign-aware monotone integer order."""
    import numpy

    signed = bits.astype(numpy.int64)
    negative = signed >= 0x80000000
    return numpy.where(negative, 0x80000000 - signed, signed + 0x80000000)


def workload_leg(host, metallib, work_dir, out_dir):
    import numpy

    manifest = []
    for case_id, m, n, k, _, _ in WORKLOAD_CELLS:
        for realization, (kernel, _) in REALIZATIONS.items():
            if realization == "direct_zero_seed":
                continue
            manifest.append(
                "\t".join(
                    [
                        "case",
                        f"{case_id}.{realization}",
                        kernel,
                        str(m),
                        str(n),
                        str(k),
                        str(WORKLOAD_SPLIT),
                        f"prng:{WORKLOAD_SEED}",
                        "0",
                        "file",
                    ]
                )
            )
    rows = dispatch(host, metallib, manifest, work_dir)

    out_rows = []
    for case_id, m, n, k, weight_class, role in WORKLOAD_CELLS:
        left = prng_array(WORKLOAD_SEED, m * k).reshape(m, k)
        right = prng_array(WORKLOAD_SEED ^ 0xA5A5A5A5A5A5A5A5, n * k).reshape(n, k)

        # The operand cross-check. Without it the oracle would be a comparison
        # against operands the driver believes were used rather than the ones the
        # device consumed.
        left_digest = hashlib.sha256(numpy.ascontiguousarray(left).tobytes()).hexdigest()
        right_digest = hashlib.sha256(numpy.ascontiguousarray(right).tobytes()).hexdigest()
        first = f"case.{case_id}.direct"
        if rows.get(f"{first}.operand_a_sha256") != left_digest or rows.get(f"{first}.operand_b_sha256") != right_digest:
            raise SystemExit(
                f"{case_id}: host operand digests disagree with the reconstruction; "
                "no comparison against this cell is admissible"
            )

        transposed = numpy.ascontiguousarray(right.T)  # [K, N], contiguous rows
        oracle = left[:, 0:1] * transposed[0][numpy.newaxis, :]
        for index in range(1, k):
            oracle = oracle + left[:, index : index + 1] * transposed[index][numpy.newaxis, :]
        # Not an `assert`: `python -O` deletes those, and this repository's
        # convention is that no verdict may be carried by a statement an
        # optimized interpreter removes. A float64 oracle would silently make
        # every realization look wrong.
        if oracle.dtype != numpy.float32:
            raise SystemExit(f"{case_id}: the host oracle is {oracle.dtype}, not binary32")

        wide = numpy.zeros((m, n), dtype=numpy.float64)
        left64 = left.astype(numpy.float64)
        transposed64 = transposed.astype(numpy.float64)
        for index in range(k):
            wide = wide + left64[:, index : index + 1] * transposed64[index][numpy.newaxis, :]

        oracle_bits = oracle.view(numpy.uint32)
        for realization, _ in REALIZATIONS.items():
            if realization == "direct_zero_seed":
                continue
            prefix = f"case.{case_id}.{realization}"
            status = rows.get(f"{prefix}.status", "absent")
            if status != "ok":
                out_rows.append([case_id, realization, m, n, k, weight_class, status, "", "", "", "", "", "", "", role])
                continue
            unwritten = int(rows[f"{prefix}.unwritten_count"])
            payload = Path(rows[f"{prefix}.result_file"]).read_bytes()
            observed = numpy.frombuffer(payload, dtype=numpy.float32).reshape(m, n)
            observed_bits = observed.view(numpy.uint32)

            identical = int((observed_bits == oracle_bits).sum())
            total = m * n
            # A raw bit-pattern subtraction is not a ULP distance across the sign
            # boundary, and a contraction over signed operands lands on both
            # sides of zero. Map to the standard monotone total order first, so
            # the count is the number of representable binary32 values between
            # the two results whatever their signs.
            ulp_gap = numpy.abs(monotone_order(observed_bits) - monotone_order(oracle_bits))
            deviation = numpy.abs(observed.astype(numpy.float64) - wide)

            verdict = "bit-identical-to-strict-fold" if identical == total else "differs-from-strict-fold"
            if unwritten > 0:
                verdict = f"inadmissible-unwritten-{unwritten}"
            out_rows.append(
                [
                    case_id,
                    realization,
                    m,
                    n,
                    k,
                    weight_class,
                    verdict,
                    f"{identical}/{total}",
                    int(numpy.median(ulp_gap)),
                    int(numpy.percentile(ulp_gap, 99.9)),
                    int(ulp_gap.max()),
                    f"{float(deviation.max()):.6e}",
                    f"{float(numpy.sqrt((wide * wide).mean())):.6e}",
                    rows[f"{prefix}.result_sha256"],
                    role,
                ]
            )

    write_tsv(
        out_dir / "workload.tsv",
        [
            "cell",
            "realization",
            "m",
            "n",
            "k",
            "weight_class",
            "verdict",
            "bit_identical",
            "median_ulp_gap_vs_strict_fold",
            "p99_9_ulp_gap_vs_strict_fold",
            "max_ulp_gap_vs_strict_fold",
            "max_abs_deviation_vs_binary64",
            "rms_result_magnitude",
            "result_sha256",
            "workload_role",
        ],
        out_rows,
    )
    return rows


# ---------------------------------------------------------------------------
# Leg 3 — interleaved timing.
# ---------------------------------------------------------------------------


def timing_leg(host, metallib, work_dir, out_dir, rounds, reps):
    candidates = [name for name in DELIVERY_CANDIDATES]
    manifest = []
    for round_index in range(rounds):
        for realization in candidates:
            kernel, _ = REALIZATIONS[realization]
            for case_id, m, n, k, _, _ in TIMING_CELLS:
                manifest.append(
                    "\t".join(
                        [
                            "case",
                            f"{case_id}.{realization}.r{round_index}",
                            kernel,
                            str(m),
                            str(n),
                            str(k),
                            str(WORKLOAD_SPLIT),
                            f"prng:{WORKLOAD_SEED}",
                            str(reps),
                            "none",
                        ]
                    )
                )
    rows = dispatch(host, metallib, manifest, work_dir)

    out_rows = []
    for case_id, m, n, k, weight_class, role in TIMING_CELLS:
        for realization in candidates:
            for round_index in range(rounds):
                prefix = f"case.{case_id}.{realization}.r{round_index}"
                status = rows.get(f"{prefix}.status", "absent")
                if status != "ok":
                    out_rows.append([case_id, realization, m, n, k, round_index, status, "", "", role])
                    continue
                samples = [
                    float(rows[f"{prefix}.gpu_seconds.{index}"])
                    for index in range(1, reps + 1)
                    if f"{prefix}.gpu_seconds.{index}" in rows
                ]
                if not samples:
                    out_rows.append([case_id, realization, m, n, k, round_index, "no-samples", "", "", role])
                    continue
                flops = 2.0 * m * n * k
                best = min(samples)
                out_rows.append(
                    [
                        case_id,
                        realization,
                        m,
                        n,
                        k,
                        round_index,
                        "ok",
                        f"{best * 1e6:.3f}",
                        f"{flops / best / 1e9:.2f}",
                        role,
                    ]
                )

    write_tsv(
        out_dir / "timing.tsv",
        ["cell", "realization", "m", "n", "k", "round", "status", "min_gpu_microseconds", "gflop_per_second", "workload_role"],
        out_rows,
    )

    # Round 0 is reported separately rather than averaged in. One warm-up
    # dispatch per manifest line is not enough to remove the first-encounter cost
    # of a (cell, realization) pair, and the retained rows show it: on the bench
    # host round 0 runs several times slower than rounds 1..N-1 for some pairs
    # while those later rounds agree within about one percent. Folding it in
    # would hide a real effect inside an average.
    summary_rows = []
    for case_id, m, n, k, weight_class, role in TIMING_CELLS:
        for realization in candidates:
            samples = {
                row[5]: float(row[7])
                for row in out_rows
                if row[0] == case_id and row[1] == realization and row[6] == "ok"
            }
            if not samples:
                statuses = {row[6] for row in out_rows if row[0] == case_id and row[1] == realization}
                summary_rows.append([case_id, realization, m, n, k, sorted(statuses)[0], "", "", "", "", role])
                continue
            settled = [value for index, value in samples.items() if index > 0]
            if not settled:
                settled = list(samples.values())
            best = min(settled)
            spread = (max(settled) - best) / best if best > 0 else 0.0
            flops = 2.0 * m * n * k
            summary_rows.append(
                [
                    case_id,
                    realization,
                    m,
                    n,
                    k,
                    "ok",
                    f"{best:.3f}",
                    f"{flops / (best * 1e-6) / 1e9:.1f}",
                    f"{samples.get(0, float('nan')):.3f}",
                    f"{spread * 100:.2f}",
                    role,
                ]
            )
    write_tsv(
        out_dir / "timing-summary.tsv",
        [
            "cell",
            "realization",
            "m",
            "n",
            "k",
            "status",
            "settled_min_microseconds",
            "settled_gflop_per_second",
            "round0_min_microseconds",
            "settled_spread_percent",
            "workload_role",
        ],
        summary_rows,
    )
    return rows


# ---------------------------------------------------------------------------


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--mode",
        choices=["semantics", "workload", "correctness", "timing", "all"],
        default="all",
        help="`correctness` is semantics then workload, which is what one host row retains together",
    )
    parser.add_argument("--out", required=True, help="result directory to write")
    parser.add_argument("--work-dir", default=None, help="scratch directory; a temporary one by default")
    parser.add_argument("--rounds", type=int, default=5, help="interleaved A/B rounds in timing mode")
    parser.add_argument("--reps", type=int, default=7, help="timed dispatches per round, after one warm-up")
    arguments = parser.parse_args()

    out_dir = Path(arguments.out).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    temporary = None
    if arguments.work_dir:
        work_dir = Path(arguments.work_dir).resolve()
        work_dir.mkdir(parents=True, exist_ok=True)
    else:
        temporary = tempfile.mkdtemp(prefix="tiler-contraction-")
        work_dir = Path(temporary)

    try:
        metallib, host = build(work_dir)
        expansions = {
            "all": ["semantics", "workload", "timing"],
            "correctness": ["semantics", "workload"],
        }
        modes = expansions.get(arguments.mode, [arguments.mode])
        environment = {}
        for mode in modes:
            if mode == "semantics":
                rows = semantic_leg(host, metallib, work_dir, out_dir)
            elif mode == "workload":
                rows = workload_leg(host, metallib, work_dir, out_dir)
            else:
                rows = timing_leg(host, metallib, work_dir, out_dir, arguments.rounds, arguments.reps)
            for key, value in rows.items():
                if key.startswith("environment.") or key.startswith("pipeline."):
                    environment[key] = value
        extra = {key.split(".", 1)[1] if key.startswith("environment.") else key: value for key, value in environment.items()}
        extra["modes"] = ",".join(modes)
        if "timing" in modes:
            extra["timing_rounds"] = arguments.rounds
            extra["timing_reps_per_round"] = arguments.reps
        write_key_value_tsv(out_dir / "environment.tsv", environment_rows(extra))

        manifest_rows = {}
        for path in sorted(out_dir.glob("*.tsv")):
            if path.name == "manifest.tsv":
                continue
            manifest_rows[f"result.sha256.{path.name}"] = hashlib.sha256(path.read_bytes()).hexdigest()
        for source in ("kernels.metal", "host.m", "contraction_probe.py"):
            manifest_rows[f"producer.sha256.{source}"] = hashlib.sha256((SPIKE_DIR / source).read_bytes()).hexdigest()
        write_key_value_tsv(out_dir / "manifest.tsv", manifest_rows)
        sys.stdout.write(f"wrote {out_dir}\n")
    finally:
        if temporary:
            shutil.rmtree(temporary, ignore_errors=True)


if __name__ == "__main__":
    main()
