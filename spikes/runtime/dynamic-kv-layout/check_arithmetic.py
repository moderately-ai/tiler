#!/usr/bin/env python3
"""Check the model-wide payload, accessible-span, and reservation arithmetic."""

import argparse
import sys


HEADS = 8
WIDTH = 128
ELEMENT_BYTES = 4
MEMBERS = 28 * 2

ROWS = (
    ("c1-prefill", 0, 10, 18, (2_293_760, 3_899_392, 8_257_536)),
    ("c1-final", 17, 18, 18, (8_028_160, 8_228_864, 8_257_536)),
    (
        "b1-final",
        8_319,
        8_320,
        8_320,
        (3_816_587_264, 3_816_787_968, 3_816_816_640),
    ),
)


def live_payload_bytes(extent: int) -> int:
    return MEMBERS * HEADS * extent * WIDTH * ELEMENT_BYTES


def capacity_bounding_bytes(extent: int, capacity: int) -> int:
    if extent == 0:
        return 0
    return MEMBERS * ((HEADS - 1) * capacity + extent) * WIDTH * ELEMENT_BYTES


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inject-pool-as-payload", action="store_true")
    args = parser.parse_args()

    print(
        "cell\tC\tS\tcapacity\tpayload_transfer_bytes\t"
        "exact_live_accessible_span_bytes\t"
        "capacity_strided_accessible_span_bytes\ttwo_bank_pool_reservation_bytes"
    )
    failures = []
    for name, old, new, capacity, expected in ROWS:
        payload = live_payload_bytes(old) + live_payload_bytes(new)
        pool = 2 * live_payload_bytes(capacity)
        if args.inject_pool_as_payload and name == "c1-prefill":
            payload = pool
        observed = (
            payload,
            capacity_bounding_bytes(old, capacity)
            + capacity_bounding_bytes(new, capacity),
            pool,
        )
        print(
            f"{name}\t{old}\t{new}\t{capacity}\t{observed[0]}\t"
            f"{observed[0]}\t{observed[1]}\t{observed[2]}"
        )
        if observed != expected:
            failures.append(f"{name}: expected {expected}, observed {observed}")

    if failures:
        print("arithmetic oracle rejected:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
