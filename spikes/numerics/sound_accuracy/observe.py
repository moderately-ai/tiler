#!/usr/bin/env python3
"""Adversarial observations for the sound-analyzer spike.

These observations can falsify an alleged universal bound. They are not a
proof that a bound holds.
"""

from __future__ import annotations

import itertools
import json
import math
import struct
from decimal import Decimal, localcontext


def f32(value: float | Decimal) -> float:
    return struct.unpack("!f", struct.pack("!f", float(value)))[0]


def f16(value: float) -> float:
    return struct.unpack("!e", struct.pack("!e", value))[0]


def d(value: float) -> Decimal:
    return Decimal.from_float(value)


def f32_add(left: float, right: float) -> float:
    return f32(left + right)


def f32_mul(left: float, right: float) -> float:
    return f32(left * right)


def f32_div(left: float, right: float) -> float:
    return f32(left / right)


def observed(candidate: float, reference: Decimal) -> Decimal:
    return abs(d(candidate) - reference)


def max_affine_mix() -> Decimal:
    values = {
        "x": [-2.0, -1.0, -2.0**-23, 0.0, 2.0**-23, 1.0, 2.0],
        "y": [-3.0, -1.0, -2.0**-23, 0.0, 2.0**-23, 1.0, 3.0],
        "z": [-1.0, -2.0**-24, 0.0, 2.0**-24, 1.0],
    }
    result = Decimal(0)
    for x, y, z in itertools.product(values["x"], values["y"], values["z"]):
        candidate = f32_add(f32_mul(f32(x), f32(y)), f32(z))
        reference = d(f32(x)) * d(f32(y)) + d(f32(z))
        result = max(result, observed(candidate, reference))
    return result


def max_explicit_fma() -> Decimal:
    values = [-2.0, -1.0, -2.0**-23, 0.0, 2.0**-23, 1.0, 2.0]
    result = Decimal(0)
    for x, y, z in itertools.product(values, values, values):
        # All sampled products and sums are exactly representable in binary64,
        # so this is one final f32 rounding rather than mul-plus-add rounding.
        candidate = f32(math.fma(f32(x), f32(y), f32(z)))
        reference = d(f32(x)) * d(f32(y)) + d(f32(z))
        result = max(result, observed(candidate, reference))
    return result


def max_divide_sqrt() -> Decimal:
    xs = [1.0, math.nextafter(1.0, 2.0), 1.5, 2.0, 3.0, 4.0]
    ys = [1.0, math.nextafter(1.0, 2.0), 1.5, 2.0]
    result = Decimal(0)
    with localcontext() as context:
        context.prec = 100
        for x, y in itertools.product(xs, ys):
            xf, yf = f32(x), f32(y)
            candidate = f32_div(f32(math.sqrt(xf)), yf)
            reference = d(xf).sqrt() / d(yf)
            result = max(result, observed(candidate, reference))
    return result


def main() -> None:
    cancellation = f32_add(f32_add(f32(16777216.0), f32(1.0)), -16777216.0)
    materialization_inputs = [
        f32(1.0),
        f32(1.0 + 2.0**-11),
        f32(1.0 + 2.0**-10),
    ]
    materialized = max(
        observed(f32_add(f32(f16(x)), -1.0), d(x) - Decimal(1))
        for x in materialization_inputs
    )

    values = [f32(100000000.0), f32(1.0), f32(-100000000.0), f32(1.0)]
    left = f32_add(f32_add(f32_add(values[0], values[1]), values[2]), values[3])
    tree = f32_add(f32_add(values[0], values[1]), f32_add(values[2], values[3]))
    reduction_reference = sum((d(value) for value in values), Decimal(0))

    result = {
        "evidence_class": "empirical-adversarial",
        "claim": "counterexample search only; never a worst-case certificate",
        "oracle": "Python Decimal precision 100 over exact binary inputs",
        "observed_max_absolute_error": {
            "affine_mix": str(max_affine_mix()),
            "cancellation": str(observed(cancellation, Decimal(1))),
            "divide_sqrt": str(max_divide_sqrt()),
            "explicit_fma": str(max_explicit_fma()),
            "materialized_f16": str(materialized),
            "reduce_left": str(observed(left, reduction_reference)),
            "reduce_tree": str(observed(tree, reduction_reference)),
            "relational_ratio": "0",
        },
        "sample_counts": {
            "affine_mix": 245,
            "divide_sqrt": 24,
            "explicit_fma": 343,
            "materialized_f16": 3,
            "fixed_witnesses": 4,
        },
    }
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
