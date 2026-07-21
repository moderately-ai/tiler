#!/usr/bin/env python3
"""Adversarial observations for the sound-analyzer spike.

These observations can falsify an alleged universal bound. They are not a
proof that a bound holds.
"""

from __future__ import annotations

import _decimal
import _struct
import hashlib
import itertools
import json
import math
import platform
import struct
import sys
from decimal import Decimal, getcontext, localcontext
from fractions import Fraction
from pathlib import Path

DECIMAL_PRECISION = 100
getcontext().prec = DECIMAL_PRECISION


def file_identity(path: str) -> dict[str, str | int]:
    """Return a resolved path, byte count, and digest for one runtime file."""
    resolved = Path(path).resolve()
    contents = resolved.read_bytes()
    return {
        "path": str(resolved),
        "bytes": len(contents),
        "sha256": hashlib.sha256(contents).hexdigest(),
    }


def module_identity(module: object) -> dict[str, object]:
    """Identify a numeric runtime module as a built-in or a concrete file."""
    path = getattr(module, "__file__", None)
    if path is not None:
        return {"kind": "file", **file_identity(path)}
    specification = getattr(module, "__spec__", None)
    return {"kind": "built-in", "origin": getattr(specification, "origin", None)}


def f32(value: float | Decimal) -> float:
    return struct.unpack("!f", struct.pack("!f", float(value)))[0]


def f16(value: float) -> float:
    return struct.unpack("!e", struct.pack("!e", value))[0]


def d(value: float) -> Decimal:
    return Decimal.from_float(value)


def decimal_from_fraction(value: Fraction) -> Decimal:
    return Decimal(value.numerator) / Decimal(value.denominator)


def round_nearest_even(numerator: int, denominator: int) -> int:
    quotient, remainder = divmod(numerator, denominator)
    comparison = 2 * remainder - denominator
    if comparison > 0 or (comparison == 0 and quotient % 2 == 1):
        return quotient + 1
    return quotient


def scale_power_of_two(value: Fraction, exponent: int) -> Fraction:
    if exponent >= 0:
        return Fraction(value.numerator << exponent, value.denominator)
    return Fraction(value.numerator, value.denominator << -exponent)


def floor_log2(value: Fraction) -> int:
    if value <= 0:
        raise ValueError("floor_log2 requires a positive value")
    exponent = value.numerator.bit_length() - value.denominator.bit_length()
    if scale_power_of_two(value, -exponent) < 1:
        exponent -= 1
    return exponent


def fraction_to_f32_bits(value: Fraction) -> int:
    """Round an exact rational to IEEE binary32, ties to even."""
    sign = 0x80000000 if value < 0 else 0
    magnitude = abs(value)
    if magnitude == 0:
        return sign

    exponent = floor_log2(magnitude)
    if exponent < -126:
        significand = round_nearest_even(*scale_power_of_two(magnitude, 149).as_integer_ratio())
        if significand == 0:
            return sign
        if significand >= 1 << 23:
            return sign | 1 << 23
        return sign | significand

    significand = round_nearest_even(
        *scale_power_of_two(magnitude, 23 - exponent).as_integer_ratio()
    )
    if significand == 1 << 24:
        significand >>= 1
        exponent += 1
    if exponent > 127:
        return sign | 0x7F800000
    return sign | ((exponent + 127) << 23) | (significand - (1 << 23))


def f32_from_bits(value: int) -> float:
    return struct.unpack("!f", struct.pack("!I", value))[0]


AFFINE_X = tuple(f32(value) for value in (-2.0, -1.0, -(2.0**-23), 0.0, 2.0**-23, 1.0, 2.0))
AFFINE_Y = tuple(f32(value) for value in (-3.0, -1.0, -(2.0**-23), 0.0, 2.0**-23, 1.0, 3.0))
AFFINE_Z = tuple(f32(value) for value in (-1.0, -(2.0**-24), 0.0, 2.0**-24, 1.0))
FMA_VALUES = AFFINE_X
RELATIONAL_VALUES = tuple(f32(value) for value in (1.0, 1.0 + 2.0**-23, 1.5, 2.0 - 2.0**-23, 2.0))
DIVIDE_SQRT_X = (
    f32(1.0),
    f32_from_bits(0x3F800001),
    f32(1.5),
    f32(2.0),
    f32(3.0),
    f32(4.0),
)
DIVIDE_SQRT_Y = (f32(1.0), f32_from_bits(0x3F800001), f32(1.5), f32(2.0))
MATERIALIZATION_INPUTS = tuple(f32(value) for value in (1.0, 1.0 + 2.0**-11, 1.0 + 2.0**-10))
REDUCTION_VALUES = tuple(f32(value) for value in (100000000.0, 1.0, -100000000.0, 1.0))


def check(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def exact_f32_fma(left: float, right: float, addend: float) -> float:
    exact = Fraction.from_float(left) * Fraction.from_float(right) + Fraction.from_float(addend)
    return f32_from_bits(fraction_to_f32_bits(exact))


def validate_f32_rounding_oracle() -> int:
    check(
        fraction_to_f32_bits(Fraction(1) + Fraction(1, 1 << 24)) == 0x3F800000,
        "binary32 halfway case must round to the even lower significand",
    )
    check(
        fraction_to_f32_bits(Fraction(1) + Fraction(3, 1 << 24)) == 0x3F800002,
        "binary32 halfway case must round to the even upper significand",
    )
    check(
        fraction_to_f32_bits(Fraction(1, 1 << 150)) == 0,
        "half of the smallest binary32 subnormal must round to zero",
    )

    left = f32(1.0 + 2.0**-23)
    right = f32(1.0 - 2.0**-23)
    fused = exact_f32_fma(left, right, f32(-1.0))
    check(
        fused == f32(-(2.0**-46)),
        "exact fused cancellation must retain the one-rounding residual",
    )
    check(
        f32_add(f32_mul(left, right), f32(-1.0)) == 0.0,
        "the fused cancellation witness must differ from multiply then add",
    )
    return 5


def f32_add(left: float, right: float) -> float:
    return f32(left + right)


def f32_mul(left: float, right: float) -> float:
    return f32(left * right)


def f32_div(left: float, right: float) -> float:
    return f32(left / right)


def observed(candidate: float, reference: Decimal) -> Decimal:
    return abs(d(candidate) - reference)


def max_affine_mix() -> tuple[Decimal, int]:
    result = Decimal(0)
    count = 0
    for x, y, z in itertools.product(AFFINE_X, AFFINE_Y, AFFINE_Z):
        count += 1
        candidate = f32_add(f32_mul(f32(x), f32(y)), f32(z))
        reference = d(f32(x)) * d(f32(y)) + d(f32(z))
        result = max(result, observed(candidate, reference))
    return result, count


def max_explicit_fma() -> tuple[Decimal, int]:
    result = Decimal(0)
    count = 0
    for x, y, z in itertools.product(FMA_VALUES, repeat=3):
        count += 1
        xf, yf, zf = f32(x), f32(y), f32(z)
        candidate = exact_f32_fma(xf, yf, zf)
        reference = d(xf) * d(yf) + d(zf)
        result = max(result, observed(candidate, reference))
    return result, count


def max_relational_ratio() -> tuple[Decimal, dict[str, str], int]:
    result = Decimal(-1)
    witness: dict[str, str] = {}
    for value in RELATIONAL_VALUES:
        candidate = f32_div(value, value)
        reference = decimal_from_fraction(Fraction.from_float(value) / Fraction.from_float(value))
        error = observed(candidate, reference)
        if error > result:
            result = error
            witness = {
                "x": value.hex(),
                "y": value.hex(),
                "candidate": candidate.hex(),
                "reference": str(reference),
            }
    return result, witness, len(RELATIONAL_VALUES)


def max_divide_sqrt() -> tuple[Decimal, int]:
    result = Decimal(0)
    count = 0
    with localcontext() as context:
        context.prec = 100
        for x, y in itertools.product(DIVIDE_SQRT_X, DIVIDE_SQRT_Y):
            count += 1
            xf, yf = f32(x), f32(y)
            candidate = f32_div(f32(math.sqrt(xf)), yf)
            reference = d(xf).sqrt() / d(yf)
            result = max(result, observed(candidate, reference))
    return result, count


def main() -> None:
    rounding_oracle_checks = validate_f32_rounding_oracle()
    cancellation = f32_add(f32_add(f32(16777216.0), f32(1.0)), -16777216.0)
    materialized = max(
        observed(f32_add(f32(f16(x)), -1.0), d(x) - Decimal(1)) for x in MATERIALIZATION_INPUTS
    )

    left = f32_add(
        f32_add(f32_add(REDUCTION_VALUES[0], REDUCTION_VALUES[1]), REDUCTION_VALUES[2]),
        REDUCTION_VALUES[3],
    )
    tree = f32_add(
        f32_add(REDUCTION_VALUES[0], REDUCTION_VALUES[1]),
        f32_add(REDUCTION_VALUES[2], REDUCTION_VALUES[3]),
    )
    reduction_reference = sum((d(value) for value in REDUCTION_VALUES), Decimal(0))
    affine_mix, affine_mix_samples = max_affine_mix()
    divide_sqrt, divide_sqrt_samples = max_divide_sqrt()
    explicit_fma, explicit_fma_samples = max_explicit_fma()
    relational_ratio, relational_ratio_witness, relational_ratio_samples = max_relational_ratio()

    result = {
        "schema": "tiler-observation/v1",
        "evidence_class": "empirical-adversarial",
        "claim": "counterexample search only; never a worst-case certificate",
        "provenance": {
            "algorithm": "tiler-sound-accuracy-observer/v1",
            "source_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "interpreter": {
                "implementation": platform.python_implementation(),
                "version": platform.python_version(),
                "full_version": sys.version,
                "cache_tag": sys.implementation.cache_tag,
                "compiler": platform.python_compiler(),
                "executable": file_identity(sys.executable),
                "numeric_extensions": {
                    "_decimal": module_identity(_decimal),
                    "_struct": module_identity(_struct),
                    "math": module_identity(math),
                },
            },
            "host": {
                "machine": platform.machine(),
                "platform": platform.platform(),
                "kernel": platform.version(),
                "libc": platform.libc_ver(),
            },
            "dependencies": {"external": [], "python_standard_library": True},
            "decimal_precision_digits": DECIMAL_PRECISION,
            "rounding": {
                "binary16": "IEEE 754 round-to-nearest-ties-to-even via struct '!e'",
                "binary32": "IEEE 754 round-to-nearest-ties-to-even via struct '!f'",
                "explicit_fma": (
                    "exact Fraction product-plus-add followed by one local binary32 RNE"
                ),
            },
        },
        "oracle": (
            "Python Decimal precision 100 over exact binary inputs; "
            "explicit FMA uses exact Fraction arithmetic and an IEEE binary32 "
            "round-to-nearest-ties-to-even oracle"
        ),
        "observed_max_absolute_error": {
            "affine_mix": str(affine_mix),
            "cancellation": str(observed(cancellation, Decimal(1))),
            "divide_sqrt": str(divide_sqrt),
            "explicit_fma": str(explicit_fma),
            "materialized_f16": str(materialized),
            "reduce_left": str(observed(left, reduction_reference)),
            "reduce_tree": str(observed(tree, reduction_reference)),
            "relational_ratio": str(relational_ratio),
        },
        "max_witnesses": {"relational_ratio": relational_ratio_witness},
        "sample_domains": {
            "affine_mix": {
                "construction": "cartesian_product(x, y, z)",
                "x_f32_hex": [value.hex() for value in AFFINE_X],
                "y_f32_hex": [value.hex() for value in AFFINE_Y],
                "z_f32_hex": [value.hex() for value in AFFINE_Z],
            },
            "cancellation": {
                "ordered_f32_hex": [
                    value.hex() for value in (f32(16777216.0), f32(1.0), f32(-16777216.0))
                ]
            },
            "divide_sqrt": {
                "construction": "cartesian_product(x, y)",
                "x_f32_hex": [value.hex() for value in DIVIDE_SQRT_X],
                "y_f32_hex": [value.hex() for value in DIVIDE_SQRT_Y],
            },
            "explicit_fma": {
                "construction": "cartesian_product(values, repeat=3)",
                "values_f32_hex": [value.hex() for value in FMA_VALUES],
            },
            "materialized_f16": {
                "input_f32_hex": [value.hex() for value in MATERIALIZATION_INPUTS]
            },
            "reductions": {
                "ordered_f32_hex": [value.hex() for value in REDUCTION_VALUES],
                "topologies": ["left", "balanced_tree"],
            },
            "relational_ratio": {
                "constraint": "x == y",
                "values_f32_hex": [value.hex() for value in RELATIONAL_VALUES],
            },
        },
        "sample_counts": {
            "affine_mix": affine_mix_samples,
            "cancellation": 1,
            "divide_sqrt": divide_sqrt_samples,
            "explicit_fma": explicit_fma_samples,
            "materialized_f16": len(MATERIALIZATION_INPUTS),
            "reduce_left": 1,
            "reduce_tree": 1,
            "relational_ratio": relational_ratio_samples,
            "rounding_oracle_checks": rounding_oracle_checks,
        },
    }
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
