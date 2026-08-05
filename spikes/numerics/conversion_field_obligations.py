#!/usr/bin/env python3
"""Enumerate which conversion-contract fields each ordered float pair owes.

Question this answers: `RQ-OP-04` of the mature operation and signature
taxonomy — does ADR 0091's BF16/binary32 two-directional-families shape, in
which one direction owes no field and the other owes all of them, hold for
every conversion pair?

Method. For each ordered pair of binary floating formats it decides four
predicates from the formats' stated layouts alone, with no measurement and no
execution:

  rounding    some in-range source value is not exactly representable in the
              destination — the destination's significand is narrower, or the
              destination's finest quantum is coarser than the source's;
  overflow    some finite source magnitude exceeds the destination's largest
              finite magnitude;
  underflow   the destination's finest quantum is coarser than the source's, so
              a source value either falls below the destination's smallest
              nonzero or lands inexactly in its subnormal range;
  nan-mapping the source carries more NaN payload bits than the destination, so
              no payload-preserving map is total.

Population. The five layouts below are exactly the binary float rows the
mature dtype taxonomy states, restricted to formats using the IEEE
all-ones-exponent-reserved convention. Twenty ordered pairs. The catalog's
FN/FNUZ formats are deliberately absent: they lack infinities or lack a signed
zero, so they owe field classes this script does not model, and their defining
documents (OCP OFP8, OCP MX) retain no local copy. See the record.

Boundary. This is a derivation over stated layouts, not a measurement. It
claims nothing about any target, any rounding realization, or any format whose
value set is not the one its layout implies.

Run: python3 spikes/numerics/conversion_field_obligations.py
Standard library only.
"""

from fractions import Fraction as F
from itertools import permutations

# name -> (exponent bits, trailing significand bits); bias is the IEEE
# 2^(e-1) - 1 for every row here, so it is derived rather than restated.
LAYOUTS = {
    "f16": (5, 10),
    "bf16": (8, 7),
    "f32": (8, 23),
    "f64": (11, 52),
    "f128": (15, 112),
}


def params(name):
    exp_bits, t = LAYOUTS[name]
    bias = (1 << (exp_bits - 1)) - 1
    emax = (1 << exp_bits) - 2 - bias
    emin = 1 - bias
    return {
        "t": t,
        "emin": emin,
        "emax": emax,
        # largest finite magnitude
        "maxfin": F(2) ** emax * (2 - F(2) ** (-t)),
        # exponent of the finest representable quantum (the subnormal step)
        "qmin": emin - t,
        # NaN payload bits, excluding the quiet bit
        "payload": t - 1,
    }


P = {name: params(name) for name in LAYOUTS}


def owed(src, dst):
    """Fields the ordered conversion src -> dst owes, as a sorted-by-role tuple."""
    s, d = P[src], P[dst]
    coarser_significand = s["t"] > d["t"]
    coarser_quantum = s["qmin"] < d["qmin"]
    fields = []
    if coarser_significand or coarser_quantum:
        fields.append("rounding")
    if s["maxfin"] > d["maxfin"]:
        fields.append("overflow")
    if coarser_quantum:
        fields.append("underflow")
    if s["payload"] > d["payload"]:
        fields.append("nan-mapping")
    return tuple(fields)


def show(fields):
    return "{}" if not fields else "{" + ", ".join(fields) + "}"


def main():
    pairs = list(permutations(LAYOUTS, 2))
    print(f"population: {len(pairs)} ordered pairs over {len(LAYOUTS)} stated layouts")
    print()
    for a, b in pairs:
        print(f"  {a:5s} -> {b:5s}  {show(owed(a, b))}")

    classes = {}
    for a, b in pairs:
        classes.setdefault(owed(a, b), []).append(f"{a}->{b}")
    print()
    print(f"distinct owed-field-set classes: {len(classes)}")
    for fields, members in sorted(classes.items(), key=lambda kv: (len(kv[0]), kv[0])):
        print(f"  {show(fields):46s} {len(members):2d}  {', '.join(members)}")

    print()
    print("per unordered pair, do the two directions' field sets intersect?")
    non_disjoint = 0
    seen = set()
    for a, b in pairs:
        if (b, a) in seen:
            continue
        seen.add((a, b))
        fa, fb = set(owed(a, b)), set(owed(b, a))
        inter = tuple(sorted(fa & fb))
        if inter:
            non_disjoint += 1
        print(f"  {a:5s}/{b:5s}  intersection = {show(inter)}")
    print()
    print(
        f"unordered pairs whose two directions are NOT disjoint: "
        f"{non_disjoint} of {len(seen)}"
    )


if __name__ == "__main__":
    main()
