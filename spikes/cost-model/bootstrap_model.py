#!/usr/bin/env python3
"""Mechanical checks for the initial cost-model contract."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Features:
    feasible: bool
    kernels: int
    traffic_bytes: int
    operations: int
    index_operations: int
    barriers: int
    pressure_bucket: str
    source_bytes: int
    artifact_bytes: int


@dataclass(frozen=True)
class Calibration:
    dispatch_ns: float
    bytes_per_ns: float
    operations_per_ns: float
    index_ns: float
    barrier_ns: float
    pressure: dict[str, float]
    relative_error: float


def estimate(features, calibration):
    if not features.feasible:
        return None
    memory = features.traffic_bytes / calibration.bytes_per_ns
    compute = features.operations / calibration.operations_per_ns
    body = max(memory, compute)
    body += features.index_operations * calibration.index_ns
    body += features.barriers * calibration.barrier_ns
    point = features.kernels * calibration.dispatch_ns
    point += body * calibration.pressure[features.pressure_bucket]
    error = point * calibration.relative_error
    return {
        "point_ns": point,
        "lower_ns": max(0.0, point - error),
        "upper_ns": point + error,
        "compiler_source_bytes": features.source_bytes,
        "artifact_bytes": features.artifact_bytes,
    }


def robustly_better(left, right):
    return left["upper_ns"] < right["lower_ns"]


def test_infeasible_is_not_a_penalty():
    calibration = sample_calibration()
    impossible = Features(False, 1, 0, 0, 0, 0, "low", 0, 0)
    assert estimate(impossible, calibration) is None


def test_fusion_trades_dispatch_and_traffic_against_pressure():
    calibration = sample_calibration()
    split = Features(True, 2, 4096, 2048, 100, 0, "low", 500, 1000)
    fused = Features(True, 1, 2048, 2300, 160, 1, "medium", 700, 1200)
    split_cost = estimate(split, calibration)
    fused_cost = estimate(fused, calibration)
    assert split_cost and fused_cost and fused_cost["point_ns"] < split_cost["point_ns"]


def test_overlapping_intervals_are_not_false_precision():
    calibration = sample_calibration()
    a = estimate(Features(True, 1, 1024, 1000, 20, 0, "low", 10, 10), calibration)
    b = estimate(Features(True, 1, 1000, 1024, 20, 0, "low", 10, 10), calibration)
    assert a and b and not robustly_better(a, b) and not robustly_better(b, a)


def sample_calibration():
    return Calibration(1000.0, 32.0, 16.0, 0.5, 20.0, {"low": 1.0, "medium": 1.2}, 0.15)


if __name__ == "__main__":
    tests = [value for name, value in sorted(globals().items()) if name.startswith("test_")]
    for test in tests:
        test()
    print(f"bootstrap cost model: {len(tests)} contract checks passed")

