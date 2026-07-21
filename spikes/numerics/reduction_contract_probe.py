#!/usr/bin/env python3
"""Dependency-free witnesses for Tiler's proposed reduction contract.

This is a semantic spike, not backend conformance evidence.  It rounds every
binary addition to IEEE binary32, matching the initial f32 Add boundary for
finite values on ordinary IEEE hosts.
"""

from __future__ import annotations

import itertools
import math
import struct


def f32(value: float) -> float:
    return struct.unpack(">f", struct.pack(">f", value))[0]


def bits(value: float) -> int:
    return struct.unpack(">I", struct.pack(">f", f32(value)))[0]


def add(left: float, right: float) -> float:
    return f32(float(f32(left)) + float(f32(right)))


def check(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def left_fold(values: list[float], initial: float | None = None) -> float:
    if initial is None:
        if values:
            accumulator = f32(values[0])
            rest = values[1:]
        else:
            accumulator = f32(0.0)
            rest = []
    else:
        accumulator = f32(initial)
        rest = values
    for value in rest:
        accumulator = add(accumulator, value)
    return accumulator


def balanced(values: list[float]) -> float:
    if not values:
        return f32(0.0)
    level = [f32(value) for value in values]
    while len(level) > 1:
        following: list[float] = []
        for offset in range(0, len(level), 2):
            if offset + 1 == len(level):
                following.append(level[offset])
            else:
                following.append(add(level[offset], level[offset + 1]))
        level = following
    return level[0]


def assert_bits(actual: float, expected: float, label: str) -> None:
    actual_bits = bits(actual)
    expected_bits = bits(expected)
    check(
        actual_bits == expected_bits,
        f"{label}: got 0x{actual_bits:08x}, expected 0x{expected_bits:08x}",
    )


def main() -> None:
    # Parenthesization changes a finite f32 sum.
    reassociation = [1.0e20, -1.0e20, 3.25]
    assert_bits(left_fold(reassociation), 3.25, "canonical left fold")
    assert_bits(
        add(reassociation[0], add(reassociation[1], reassociation[2])), 0.0, "right-associated tree"
    )

    # A leaf permutation changes the result without changing the left-deep tree.
    permutation = [1.0e20, 3.25, -1.0e20]
    assert_bits(left_fold(permutation), 0.0, "canonical permutation witness")
    assert_bits(
        left_fold([permutation[0], permutation[2], permutation[1]]), 3.25, "permuted left fold"
    )

    # An initial value is one logical contributor, never one seed per partial.
    assert_bits(left_fold([1.0, 2.0], initial=10.0), 13.0, "seed once")
    partials = [left_fold([1.0], initial=10.0), left_fold([2.0], initial=10.0)]
    assert_bits(left_fold(partials), 23.0, "invalid duplicated seed witness")

    # The empty-sum result +0 is not neutral padding for an unseeded -0 singleton.
    negative_zero = f32(-0.0)
    check(
        bits(left_fold([negative_zero])) == 0x80000000,
        "unseeded negative-zero singleton must preserve its sign",
    )
    assert_bits(left_fold([]), 0.0, "empty unseeded sum identity")
    assert_bits(balanced([]), 0.0, "empty balanced sum identity")
    assert_bits(
        left_fold([], initial=negative_zero),
        negative_zero,
        "empty seeded sum returns its single seed",
    )
    check(
        bits(add(negative_zero, 0.0)) == 0x00000000,
        "positive-zero padding must observably change negative zero",
    )

    # Contiguous balanced partials only reassociate; lane-strided partials also
    # permute. Enumerating permutations makes the independent result sets visible.
    values = [1.0e20, -1.0e20, 3.0, 4.0]
    contiguous = add(left_fold(values[:2]), left_fold(values[2:]))
    strided = add(left_fold(values[::2]), left_fold(values[1::2]))
    check(
        bits(contiguous) != bits(strided),
        "contiguous and lane-strided partials must expose different orderings",
    )
    permutations = list(itertools.permutations(values))
    check(len(permutations) == 24, f"expected 24 permutations, got {len(permutations)}")
    permutation_results = {bits(left_fold(list(order))) for order in permutations}
    check(
        len(permutation_results) > 1,
        "the exhaustive permutation corpus must expose multiple results",
    )

    # Scratch narrowing inserts a visible conversion boundary.
    accumulator = left_fold([1.0, math.ldexp(1.0, -20)])
    narrowed = struct.unpack(">e", struct.pack(">e", accumulator))[0]
    restored = f32(narrowed)
    check(
        bits(accumulator) != bits(restored),
        "f16 scratch narrowing must change the selected witness",
    )

    # A balanced tree is stable when selected, but differs from a strict fold.
    tree_values = [1.0e20, 3.25, -1.0e20, 7.0]
    check(
        bits(balanced(tree_values)) != bits(left_fold(tree_values)),
        "balanced and left-fold topologies must differ on the witness",
    )

    print("reduction contract probe: all witnesses passed")


if __name__ == "__main__":
    main()
