#!/usr/bin/env python3
"""Check the hand-derived online-softmax rescaling bound against exact evaluation.

The bound this probe checks is derived in
`docs/research/numerics/certified-bounds-as-rewrite-permissions.md` from the
standard model of floating-point arithmetic and the classical summation results
restated by Boldo, Jeannerod, Melquiond and Muller (Acta Numerica 32, 2023,
equations 2.5a, 4.6, 4.7 and 4.8), preserved under
`docs/research/numerics/sources/acta-numerica-fp-2023/`.

What the probe establishes and what it does not
-----------------------------------------------
A derivation says the computed normalizer never leaves a stated bracket. This
probe evaluates both folds in exact binary32 semantics against a 120-digit
decimal reference and checks that claim on a named finite corpus. A violation
would refute the derivation; agreement over this corpus does not prove it, and
the record labels the derivation `sound-proof` on its algebra rather than on
these numbers. What the numbers add is the other half a bound needs to be
usable: how loose it is, which is the quantity that decides whether a caller's
tolerance can actually be met.

Every arithmetic operation below is computed exactly in `Decimal` and then
rounded once to binary32, so the simulation carries no host floating-point
behaviour of its own. `exp` is correctly rounded to binary32, which is the
strongest admissible implementation and therefore the case in which any looseness
belongs to the fold rather than to the elementary function.

Run from the repository root, both modes:

    python3 spikes/numerics/online_softmax_bound_probe.py
    python3 -O spikes/numerics/online_softmax_bound_probe.py

Verdicts are explicit checks rather than `assert`, so optimized Python cannot
discard them. Either command exits nonzero instead of publishing JSON when a
derived bound is violated or a recorded constant moves.
"""

from __future__ import annotations

import hashlib
import json
import math
import platform
import struct
import sys
from decimal import Decimal, getcontext
from fractions import Fraction
from pathlib import Path

# Working precision for the reference. Far beyond binary32's 24 significand bits,
# so the reference's own error cannot reach the reported ratios: the smallest
# quantity compared below is a binary32 unit roundoff, about 6e-8.
ORACLE_DIGITS = 120
getcontext().prec = ORACLE_DIGITS

# binary32 unit roundoff, exactly 2**-24. Held as a Fraction so every bound below
# is computed in exact rational arithmetic and never through a host float.
U = Fraction(1, 2**24)

# Relative error of the `exp` routine the folds below call, as a multiple of U.
# The probe rounds `exp` correctly to binary32, so its relative error is bounded
# by U itself; the derivation carries a symbolic eps_exp because a target's
# realization is generally weaker, and instantiating it at U here is deliberately
# the most favourable case for the rewrite.
EPS_EXP = U

# The size of the corpus below, written here rather than read back from
# `corpus()`, so that a case lost from that function fails instead of agreeing
# with itself. A population check whose two sides come from one source cannot
# say no, which the first perturbation run of this probe demonstrated by
# deleting a case and still exiting zero.
DECLARED_CASES = 22


def to_binary32(value: Decimal) -> float:
    """Rounds one exact decimal to binary32, round-to-nearest-ties-to-even.

    Routed through the host's own binary32 conversion, which is correctly
    rounded, rather than through a hand-written rounding this probe would then
    have to justify. The value handed in is exact, so no double rounding occurs:
    `float(Decimal)` is correctly rounded to binary64 and `struct` rounds that to
    binary32, and the 120-digit working precision keeps the two roundings from
    landing on a binary32 tie boundary for any value this corpus produces.
    """
    return struct.unpack("f", struct.pack("f", float(value)))[0]


def exact(value: float) -> Decimal:
    """Returns the exact decimal value of one finite binary32 number."""
    return Decimal(value)


def fl_add(left: float, right: float) -> float:
    return to_binary32(exact(left) + exact(right))


def fl_mul(left: float, right: float) -> float:
    return to_binary32(exact(left) * exact(right))


def fl_sub(left: float, right: float) -> float:
    return to_binary32(exact(left) - exact(right))


def fl_exp(argument: float) -> float:
    """Returns `exp(argument)` correctly rounded to binary32."""
    return to_binary32(exact(argument).exp())


def gamma(count: int) -> Fraction:
    """Returns the classical `gamma_h = h*u / (1 - h*u)` of Acta Numerica (4.7).

    Defined only while `h*u < 1`, which the caller is required to have checked;
    the guard is here rather than in the caller because a silently negative
    gamma would turn every bound below into a vacuous one that passes.
    """
    product = count * U
    if product >= 1:
        raise ValueError(f"gamma is undefined at h={count}: h*u = {float(product)} >= 1")
    return product / (1 - product)


def two_pass_normalizer(logits: list[float]) -> float:
    """Safe softmax, Algorithm 2 of Milakov and Gimelshein: max pass, then sum."""
    peak = logits[0]
    for value in logits[1:]:
        peak = max(peak, value)
    total = 0.0
    for value in logits:
        total = fl_add(total, fl_exp(fl_sub(value, peak)))
    return total


def online_normalizer(logits: list[float]) -> float:
    """Online softmax, Algorithm 3: one pass, rescaling the running sum.

    `d_j = d_{j-1} * exp(m_{j-1} - m_j) + exp(x_j - m_j)`, with the first step
    written out because `m_0 = -inf` makes its rescale factor a literal zero
    rather than an operation whose rounding the analysis would have to carry.
    """
    peak = logits[0]
    total = fl_exp(fl_sub(logits[0], peak))
    for value in logits[1:]:
        previous_peak = peak
        peak = max(peak, value)
        rescale = fl_exp(fl_sub(previous_peak, peak))
        total = fl_add(fl_mul(total, rescale), fl_exp(fl_sub(value, peak)))
    return total


def reference_normalizer(logits: list[float]) -> Decimal:
    """Returns the exact real `sum_j exp(x_j - max_k x_k)` over exact binary32 inputs.

    This is the governed real lift the region-accuracy contract names as a
    reference: the inputs are the exact binary32 values, and every operation
    after them is exact. It is deliberately not "the two-pass result", because a
    rewrite compared against another implementation measures a difference rather
    than an error.
    """
    peak = max(logits)
    total = Decimal(0)
    for value in logits:
        total += (exact(value) - exact(peak)).exp()
    return total


def argument_spread(logits: list[float]) -> Fraction:
    """Returns `A = max_j |x_j - max_k x_k|`, the logit spread the bound is in.

    Exact, because it multiplies `u` in every bound below and a spread computed
    in binary32 would put a rounding inside the constant that bounds roundings.
    """
    peak = max(logits)
    return max(abs(Fraction(value) - Fraction(peak)) for value in logits)


def exp_of_fraction(value: Fraction) -> Decimal:
    """Returns `exp(value)` at oracle precision from an exact rational."""
    return (Decimal(value.numerator) / Decimal(value.denominator)).exp()


def two_pass_bound(count: int, spread: Fraction) -> Decimal:
    """Bound on the two-pass fold's relative forward error.

    `(1 + eps_exp) * exp(A*u) * (1 + gamma_{V-1}) - 1`. The three factors are the
    elementary function's own relative error, the amplification of the one
    rounding in each `x_j - m_V` through `exp`, and the summation of `V` positive
    terms by a fold of height at most `V - 1` (Acta Numerica 4.8, whose
    `sum |x_i|` equals the reference here because every term is positive).
    """
    exp_term = exp_of_fraction(spread * U)
    elementary = Decimal(1) + Decimal(EPS_EXP.numerator) / Decimal(EPS_EXP.denominator)
    summation = Decimal(1) + Decimal(gamma(count - 1).numerator) / Decimal(
        gamma(count - 1).denominator
    )
    return elementary * exp_term * summation - 1


def online_bound(count: int, spread: Fraction) -> Decimal:
    """Bound on the online fold's relative forward error.

    Same three factors, with two counts changed. The earliest term passes through
    at most `V` calls to `exp` — its own, plus one rescale factor per later step —
    and through at most `2*(V-1)` roundings, because every later step applies one
    multiply and one add to it where the two-pass fold applies one add. The
    argument-perturbation factor is *unchanged*: the rescale arguments telescope,
    `|x_j - m_j| + sum_{k>j} |m_{k-1} - m_k| = |x_j - m_V|`, so their perturbations
    sum to the same spread the two-pass fold already carries. That telescoping is
    the reason the rewrite's price is in `V` alone and not in `V*A`.
    """
    exp_term = exp_of_fraction(spread * U)
    eps = Decimal(EPS_EXP.numerator) / Decimal(EPS_EXP.denominator)
    elementary = (Decimal(1) + eps) ** count
    height = gamma(2 * (count - 1)) if count > 1 else Fraction(0)
    summation = Decimal(1) + Decimal(height.numerator) / Decimal(height.denominator)
    return elementary * exp_term * summation - 1


def rewrite_price(count: int) -> Decimal:
    """The admission quantity: what the rewrite adds, with the common factors cancelled.

    `(1 + eps_exp)^(V-1) * (1 + gamma_{2(V-1)}) / (1 + gamma_{V-1}) - 1`. The
    spread factor is absent by construction — it is common to both folds — which
    is why this quantity is a function of the contributor count and the target's
    elementary accuracy alone, and therefore instantiable from a shape and a
    target profile without knowing any input value.
    """
    if count < 2:
        return Decimal(0)
    eps = Decimal(EPS_EXP.numerator) / Decimal(EPS_EXP.denominator)
    wide = gamma(2 * (count - 1))
    narrow = gamma(count - 1)
    wide_d = Decimal(1) + Decimal(wide.numerator) / Decimal(wide.denominator)
    narrow_d = Decimal(1) + Decimal(narrow.numerator) / Decimal(narrow.denominator)
    return (Decimal(1) + eps) ** (count - 1) * wide_d / narrow_d - 1


def corpus() -> list[tuple[str, list[float]]]:
    """The named finite population this probe checks, and why each member is in it.

    Every case is stated rather than sampled, because a bound is refuted by a
    worst case and a random draw does not know where one is.
    """
    cases: list[tuple[str, list[float]]] = []

    # The rescaling is exercised only when the running maximum actually moves.
    # A strictly increasing input moves it at every step, which is the input that
    # makes every one of the V-1 rescale factors a non-trivial rounding.
    for count in (2, 8, 64, 512):
        for span in (1.0, 20.0, 80.0):
            cases.append(
                (
                    f"increasing-v{count}-span{span:g}",
                    [to_binary32(Decimal(span) * Decimal(i) / Decimal(count - 1)) for i in range(count)],
                )
            )

    # The mirror image: the maximum is found first and never moves again, so every
    # rescale factor is exactly 1 and the online fold degenerates toward the
    # two-pass one. This is where the bound should be loosest, and saying so is
    # the point of including it.
    for count in (8, 64, 512):
        cases.append(
            (
                f"decreasing-v{count}",
                [to_binary32(Decimal(40) * Decimal(count - 1 - i) / Decimal(count - 1)) for i in range(count)],
            )
        )

    # All equal: every term is exactly 1 and the reference is exactly V, so any
    # departure is entirely the fold's.
    for count in (8, 64, 512):
        cases.append((f"uniform-v{count}", [1.5] * count))

    # One dominant logit with a long tail far below it. The reference is close to
    # 1 and every other term is near the underflow of the ratio, which is the
    # regime an attention softmax is usually in.
    for count in (64, 512):
        values = [to_binary32(Decimal(-30))] * (count - 1) + [to_binary32(Decimal(30))]
        cases.append((f"dominant-tail-v{count}", values))

    # A sawtooth, so the running maximum moves on some steps and not others and
    # the rescale factors alternate between 1 and a genuine rounding.
    for count in (64, 512):
        values = [
            to_binary32(Decimal(i % 7) - Decimal(3) + Decimal(i) / Decimal(count))
            for i in range(count)
        ]
        cases.append((f"sawtooth-v{count}", values))

    return cases


def evaluate(name: str, logits: list[float]) -> dict[str, object]:
    count = len(logits)
    spread = argument_spread(logits)
    reference = reference_normalizer(logits)
    if reference <= 0:
        raise ValueError(f"{name}: the reference normalizer is not positive")

    two_pass = Decimal(two_pass_normalizer(logits))
    online = Decimal(online_normalizer(logits))

    two_pass_error = abs(two_pass - reference) / reference
    online_error = abs(online - reference) / reference
    observed_price = abs(online - two_pass) / reference

    return {
        "case": name,
        "contributors": count,
        "argument_spread": float(spread),
        "two_pass_relative_error": _magnitude(two_pass_error),
        "two_pass_bound": _magnitude(two_pass_bound(count, spread)),
        "two_pass_bound_over_observed": _ratio(two_pass_bound(count, spread), two_pass_error),
        "online_relative_error": _magnitude(online_error),
        "online_bound": _magnitude(online_bound(count, spread)),
        "online_bound_over_observed": _ratio(online_bound(count, spread), online_error),
        "observed_price": _magnitude(observed_price),
        "derived_price": _magnitude(rewrite_price(count)),
    }


def _ratio(bound: Decimal, observed: Decimal) -> str:
    """Formats bound/observed, or names the case where the observed error is zero."""
    if observed == 0:
        return "observed-zero"
    return f"{bound / observed:.4G}"


def _magnitude(value: Decimal) -> str:
    """Formats one non-negative decimal, spelling an exact zero as `0`.

    `Decimal(0)` retains the exponent of whatever arithmetic produced it, so a
    computed zero formats as `0.000000E+102` under `%E` — a value a reader has to
    decode before seeing that it is zero, in the rows where zero is the finding.
    """
    if value == 0:
        return "0"
    return f"{value:.6E}"


def check(rows: list[dict[str, object]]) -> list[str]:
    """Returns the failures. Explicit rather than `assert`, so `-O` cannot drop them."""
    failures: list[str] = []
    for row in rows:
        name = row["case"]
        if Decimal(str(row["two_pass_relative_error"])) > Decimal(str(row["two_pass_bound"])):
            failures.append(f"{name}: two-pass error exceeds its derived bound")
        if Decimal(str(row["online_relative_error"])) > Decimal(str(row["online_bound"])):
            failures.append(f"{name}: online error exceeds its derived bound")
        # The price is what a caller's tolerance is compared against, so it is
        # checked separately from the two absolute bounds it was factored out of.
        if Decimal(str(row["observed_price"])) > Decimal(str(row["derived_price"])):
            failures.append(f"{name}: observed price exceeds the derived price")
    if len(rows) != DECLARED_CASES:
        failures.append(
            f"population mismatch: evaluated {len(rows)} cases, expected {DECLARED_CASES}"
        )
    return failures


def main() -> int:
    source = Path(__file__).resolve()
    rows = [evaluate(name, logits) for name, logits in corpus()]
    failures = check(rows)
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        print(f"{len(failures)} check(s) failed over {len(rows)} cases.", file=sys.stderr)
        return 1

    record = {
        "probe": source.name,
        "probe_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "oracle": {"library": "decimal (Python standard library)", "digits": ORACLE_DIGITS},
        "format": {"name": "binary32", "unit_roundoff": "2**-24"},
        "elementary_function": {
            "name": "exp",
            "implementation": "correctly rounded to binary32 from a 120-digit decimal",
            "eps_exp": "2**-24",
        },
        "host": {
            "python_implementation": platform.python_implementation(),
            "python_version": platform.python_version(),
            "machine": platform.machine(),
            "system": platform.system(),
        },
        "declared_cases": DECLARED_CASES,
        "evaluated_cases": len(rows),
        "results": rows,
    }
    print(json.dumps(record, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
