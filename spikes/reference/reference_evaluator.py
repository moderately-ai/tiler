#!/usr/bin/env python3
"""Bit-oriented reference evaluator for one representative Tiler graph."""

import math
import struct
from dataclasses import dataclass


class ReferenceError(Exception):
    pass


class WitnessFailure(RuntimeError):
    """A reference-evaluator witness did not hold."""


def require(condition, message):
    if not condition:
        raise WitnessFailure(message)


def f32_bits(value):
    return struct.unpack("<I", struct.pack("<f", value))[0]


def bits_f32(bits):
    return struct.unpack("<f", struct.pack("<I", bits))[0]


def f16_bits(value):
    if math.isnan(value):
        return 0x7E00
    try:
        return struct.unpack("<H", struct.pack("<e", value))[0]
    except OverflowError:
        return 0xFC00 if math.copysign(1.0, value) < 0 else 0x7C00


def bits_f16(bits):
    return struct.unpack("<e", struct.pack("<H", bits))[0]


def strict_f32_add(left_bits, right_bits):
    left = bits_f32(left_bits)
    right = bits_f32(right_bits)
    if math.isnan(left) or math.isnan(right):
        return 0x7FC00000
    return f32_bits(left + right)


@dataclass(frozen=True)
class Tensor:
    dtype: str
    shape: tuple[int, ...]
    bits: tuple[int, ...]

    def __post_init__(self):
        count = math.prod(self.shape)
        if any(extent < 0 for extent in self.shape) or count != len(self.bits):
            raise ReferenceError("invalid-tensor-shape")


def cast_f32_to_f16(value):
    if value.dtype != "f32":
        raise ReferenceError("cast-source-dtype")
    return Tensor("f16", value.shape, tuple(f16_bits(bits_f32(bits)) for bits in value.bits))


def cast_f16_to_f32(value):
    if value.dtype != "f16":
        raise ReferenceError("cast-source-dtype")
    return Tensor("f32", value.shape, tuple(f32_bits(bits_f16(bits)) for bits in value.bits))


def broadcast_last_axis(value, output_shape):
    if value.dtype != "f32" or len(value.shape) != 1 or not output_shape:
        raise ReferenceError("unsupported-broadcast-signature")
    if value.shape[0] != output_shape[-1]:
        raise ReferenceError("broadcast-extent-mismatch")
    outer = math.prod(output_shape[:-1])
    return Tensor("f32", tuple(output_shape), value.bits * outer)


def add_f32(left, right):
    if left.dtype != "f32" or right.dtype != "f32" or left.shape != right.shape:
        raise ReferenceError("add-signature-mismatch")
    return Tensor(
        "f32",
        left.shape,
        tuple(strict_f32_add(a, b) for a, b in zip(left.bits, right.bits, strict=True)),
    )


def reshape_row_major(value, output_shape):
    if math.prod(value.shape) != math.prod(output_shape):
        raise ReferenceError("reshape-element-count-mismatch")
    return Tensor(value.dtype, tuple(output_shape), value.bits)


def evaluate_pipeline(x, bias):
    """Evaluate the canonical graph and return two ordered graph outputs.

    rounded = cast_f16(x); widened = cast_f32(rounded)
    biased = widened + broadcast_last_axis(bias, x.shape)
    view = reshape_row_major(biased, (3, 2))
    outputs = [biased, view]
    """
    if x.shape != (2, 3) or x.dtype != "f32":
        raise ReferenceError("pipeline-x-signature")
    if bias.shape != (3,) or bias.dtype != "f32":
        raise ReferenceError("pipeline-bias-signature")
    rounded = cast_f32_to_f16(x)
    widened = cast_f16_to_f32(rounded)
    biased = add_f32(widened, broadcast_last_axis(bias, x.shape))
    view = reshape_row_major(biased, (3, 2))
    return biased, view


def tensor_f32(shape, values):
    return Tensor("f32", tuple(shape), tuple(f32_bits(value) for value in values))


def test_materialization_rounding_is_observable():
    midpoint = 1.0 + 2.0**-11
    x = tensor_f32((2, 3), [midpoint, 2.0, -0.0, 3.0, -4.0, float("nan")])
    bias = tensor_f32((3,), [0.0, 0.5, -0.0])
    biased, view = evaluate_pipeline(x, bias)
    require(
        biased.shape == (2, 3) and view.shape == (3, 2),
        "ordered outputs had incorrect shapes",
    )
    require(biased.bits == view.bits, "reshape changed row-major element bits")
    require(
        biased.bits[0] == f32_bits(1.0),
        "halfway f16 conversion did not round to even",
    )
    require(biased.bits[1] == f32_bits(2.5), "bias addition produced incorrect bits")
    require(biased.bits[5] == 0x7FC00000, "NaN result was not canonicalized")


def test_removing_the_cast_boundary_changes_the_answer():
    midpoint = 1.0 + 2.0**-11
    x = tensor_f32((2, 3), [midpoint] * 6)
    bias = tensor_f32((3,), [0.0, 0.0, 0.0])
    biased, _ = evaluate_pipeline(x, bias)
    without_boundary = add_f32(x, broadcast_last_axis(bias, x.shape))
    require(
        biased.bits[0] != without_boundary.bits[0],
        "removing the f16 materialization boundary did not change the answer",
    )


def test_bad_broadcast_and_reshape_fail_with_stable_codes():
    value = tensor_f32((2,), [1.0, 2.0])
    try:
        broadcast_last_axis(value, (2, 3))
    except ReferenceError as error:
        require(
            str(error) == "broadcast-extent-mismatch",
            f"bad broadcast returned unstable error code: {error}",
        )
    else:
        raise WitnessFailure("bad broadcast accepted")
    try:
        reshape_row_major(value, (3,))
    except ReferenceError as error:
        require(
            str(error) == "reshape-element-count-mismatch",
            f"bad reshape returned unstable error code: {error}",
        )
    else:
        raise WitnessFailure("bad reshape accepted")


if __name__ == "__main__":
    tests = [value for name, value in sorted(globals().items()) if name.startswith("test_")]
    for test in tests:
        test()
    print(f"reference evaluator: {len(tests)} cases passed")
