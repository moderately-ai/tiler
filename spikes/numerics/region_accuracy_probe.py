#!/usr/bin/env python3
"""Bounded empirical witness for region-level accuracy contract hazards."""

from __future__ import annotations

import json
import math
import struct
from dataclasses import dataclass

import mpmath

mpmath.mp.dps = 100


class ProbeFailure(RuntimeError):
    """Raised when an adversarial witness no longer has its required shape."""


def require(condition: bool, message: str) -> None:
    """Fail in ordinary and optimized Python when a witness is invalid."""
    if not condition:
        raise ProbeFailure(message)


def f32(value: float | mpmath.mpf) -> float:
    return struct.unpack("!f", struct.pack("!f", float(value)))[0]


def f16(value: float) -> float:
    return struct.unpack("!e", struct.pack("!e", value))[0]


def f32_add(left: float, right: float) -> float:
    return f32(mpmath.mpf(left) + mpmath.mpf(right))


def f32_mul(left: float, right: float) -> float:
    return f32(mpmath.mpf(left) * mpmath.mpf(right))


def ordered_f32(value: float) -> int:
    bits = struct.unpack("!I", struct.pack("!f", value))[0]
    return (~bits & 0xFFFFFFFF) if bits & 0x80000000 else bits | 0x80000000


def ulp_gap(left: float, right: float) -> int | None:
    if not math.isfinite(left) or not math.isfinite(right):
        return None
    return abs(ordered_f32(left) - ordered_f32(right))


@dataclass(frozen=True)
class Error:
    absolute: float
    relative: float | None
    ulp: int | None


def error(candidate: float, reference: float) -> Error:
    absolute = abs(candidate - reference)
    relative = None if reference == 0.0 else absolute / abs(reference)
    return Error(absolute, relative, ulp_gap(candidate, reference))


def left_reduce(values: list[float]) -> float:
    result = f32(0.0)
    for value in values:
        result = f32_add(result, value)
    return result


def tree_reduce(values: list[float]) -> float:
    level = values
    while len(level) > 1:
        level = [f32_add(level[i], level[i + 1]) for i in range(0, len(level), 2)]
    return level[0]


def as_dict(value: Error) -> dict[str, float | int | None]:
    return {"absolute": value.absolute, "relative": value.relative, "ulp": value.ulp}


def main() -> None:
    require(
        mpmath.__version__ == "1.3.0",
        f"unsupported mpmath version {mpmath.__version__}; expected 1.3.0",
    )
    require(mpmath.mp.dps == 100, "the high-precision oracle must use 100 digits")

    # Removing an f16 materialization changes a later result even without
    # reassociation. The input is exactly halfway between adjacent f16 values.
    x = f32(1.0 + 2.0**-11)
    strict_materialized = f32_add(f32(f16(x)), -1.0)
    fused_boundary_elided = f32_add(x, -1.0)
    require(strict_materialized == 0.0, "f16 materialization must round x to 1.0")
    require(
        fused_boundary_elided != 0.0,
        "eliding the f16 materialization must preserve the halfway residual",
    )

    # The preferred expression depends on the named reference. Under strict
    # f32 operation semantics, (a+b)-a is zero; under a real-valued reference,
    # the mathematically equivalent b is one.
    a, b = f32(2.0**24), f32(1.0)
    strict_cancellation = f32_add(f32_add(a, b), -a)
    algebraic_candidate = b
    require(strict_cancellation == 0.0, "strict f32 cancellation witness changed")
    require(algebraic_candidate == 1.0, "real-algebra candidate witness changed")

    # Reduction order is an input to the candidate identity, not noise in the
    # measurement. Both results differ from the high-precision sum.
    reduction_input = [f32(1.0e8), f32(1.0), f32(-1.0e8), f32(1.0)]
    reduction_real = float(sum(mpmath.mpf(value) for value in reduction_input))
    reduction_left = left_reduce(reduction_input)
    reduction_tree = tree_reduce(reduction_input)
    require(reduction_real == 2.0, "high-precision reduction reference changed")
    require(reduction_left == 1.0, "left reduction topology witness changed")
    require(reduction_tree == 0.0, "tree reduction topology witness changed")

    # Near zero, relative error has no value unless the contract supplies a
    # zero policy or uses a mixed absolute/relative metric.
    zero_relative = error(fused_boundary_elided, strict_materialized)
    require(
        zero_relative.relative is None,
        "relative error at an exact-zero reference must remain undefined",
    )

    result = {
        "evidence": {
            "class": "empirical-adversarial",
            "oracle": "mpmath-1.3.0-100-decimal-digits",
            "claim": "witnesses only; not a worst-case certificate",
        },
        "materialization": {
            "strict_reference": strict_materialized,
            "boundary_elided": fused_boundary_elided,
            "candidate_error": as_dict(zero_relative),
        },
        "reference_choice": {
            "strict_f32_reference": strict_cancellation,
            "real_reference": 1.0,
            "algebraic_candidate": algebraic_candidate,
            "candidate_vs_strict": as_dict(error(algebraic_candidate, strict_cancellation)),
            "strict_vs_real": as_dict(error(strict_cancellation, 1.0)),
        },
        "reduction_topology": {
            "real_reference": reduction_real,
            "left": reduction_left,
            "tree": reduction_tree,
            "left_error": as_dict(error(reduction_left, reduction_real)),
            "tree_error": as_dict(error(reduction_tree, reduction_real)),
        },
    }
    print(json.dumps(result, indent=2, sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    main()
