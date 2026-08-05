#!/usr/bin/env python3
"""Exhibit binary32 counterexamples to `exp(a) * exp(b) = exp(a + b)`.

The companion question to `probe.sh`. That probe asks whether a compiler
performs the rewrite; this one asks whether performing it would change the
answer, because a freedom that changed no result would need no permission.

The exponential here is **correctly rounded** to binary32 — the strongest
implementation any target could declare. So a disagreement found here is a
property of the identity under floating-point arithmetic and not of a sloppy
`exp`, which is what makes it a statement about the rewrite rather than about a
library. Every value is computed exactly in `Decimal` at 120 digits and rounded
once to binary32; no host floating-point rounding, optimization level, or
compiler flag participates.

The argument grid is the non-positive integers, which is the region the
governed softmax's exponential admits: its arguments are `s_i - m` against the
row maximum, so the exact difference is never positive
(`SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` is `+0.0`). A counterexample
outside that region would not reach the rewrite this record is about.

Run from this directory, in either mode:

    python3 identity_counterexample.py
    python3 -O identity_counterexample.py

Standard library only. Verdicts are explicit checks rather than `assert`, so
optimized Python cannot discard them; either command exits nonzero instead of
publishing a record when the population does not match or when the claimed
smallest counterexample is not reproduced.
"""

import struct
import sys
from decimal import Decimal, getcontext, localcontext

# The grid is [-GRID, 0] in each argument, inclusive. `DECLARED_PAIRS` is a
# bare literal and deliberately not `(GRID + 1) ** 2`: a population check whose
# two sides are both computed from `GRID` would move with `GRID` and agree with
# itself, which is a check that cannot say no.
GRID = 40
DECLARED_PAIRS = 1681

# The smallest-magnitude disagreement, stated here so the run reproduces a
# claim rather than reporting whatever it happens to find. These exact bits are
# what the research record quotes.
DECLARED_SMALLEST = {
    "a": -1.0,
    "b": -1.0,
    "exp_a_bits": "0x3ebc5ab2",
    "product_bits": "0x3e0a9556",
    "exp_sum_bits": "0x3e0a9555",
}

getcontext().prec = 120


def to_binary32(value):
    """Round an exact value once to binary32, round-to-nearest ties-to-even."""
    return struct.unpack("f", struct.pack("f", float(value)))[0]


def binary32_bits(value):
    return "0x%08x" % struct.unpack("I", struct.pack("f", value))[0]


def correctly_rounded_exp(argument):
    """The binary32-correctly-rounded exponential of an exact binary32 value."""
    with localcontext() as context:
        context.prec = 120
        return to_binary32(Decimal(argument).exp())


def survey():
    """Return (population, disagreements, smallest) over the declared grid."""
    population = 0
    disagreements = []
    for a_magnitude in range(GRID + 1):
        for b_magnitude in range(GRID + 1):
            a = to_binary32(-a_magnitude)
            b = to_binary32(-b_magnitude)
            total = to_binary32(a + b)
            product = to_binary32(
                Decimal(correctly_rounded_exp(a)) * Decimal(correctly_rounded_exp(b))
            )
            folded = correctly_rounded_exp(total)
            population += 1
            if product != folded:
                disagreements.append((a_magnitude + b_magnitude, a, b, product, folded))
    disagreements.sort(key=lambda row: (row[0], -row[1]))
    return population, disagreements


def main():
    population, disagreements = survey()
    failures = []

    if population != DECLARED_PAIRS:
        failures.append(
            f"population mismatch: evaluated {population} pairs, expected {DECLARED_PAIRS}"
        )
    if not disagreements:
        failures.append("no disagreement found; the identity would need no permission")
    else:
        _, a, b, product, folded = disagreements[0]
        observed = {
            "a": a,
            "b": b,
            "exp_a_bits": binary32_bits(correctly_rounded_exp(a)),
            "product_bits": binary32_bits(product),
            "exp_sum_bits": binary32_bits(folded),
        }
        if observed != DECLARED_SMALLEST:
            failures.append(
                f"smallest counterexample mismatch: observed {observed}, "
                f"declared {DECLARED_SMALLEST}"
            )

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    print(f"population\t{population}")
    print(f"disagreements\t{len(disagreements)}")
    print(f"disagreement_fraction\t{len(disagreements) / population:.4f}")
    print("smallest_a\t%r" % disagreements[0][1])
    print("smallest_b\t%r" % disagreements[0][2])
    print("smallest_exp_a_bits\t%s" % DECLARED_SMALLEST["exp_a_bits"])
    print("smallest_product_bits\t%s" % DECLARED_SMALLEST["product_bits"])
    print("smallest_exp_sum_bits\t%s" % DECLARED_SMALLEST["exp_sum_bits"])
    print("exp_implementation\tcorrectly rounded to binary32 from Decimal at 120 digits")
    print("argument_grid\tnon-positive integers in [-%d, 0], both arguments" % GRID)
    return 0


if __name__ == "__main__":
    sys.exit(main())
