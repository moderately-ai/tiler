#!/usr/bin/env python3
"""Focused structural audit of Candle's concrete ensure_completed function."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def extract_function(source: str, signature: str) -> tuple[str, int]:
    start = source.index(signature)
    open_brace = source.index("{", start)
    depth = 0
    for position in range(open_brace, len(source)):
        if source[position] == "{":
            depth += 1
        elif source[position] == "}":
            depth -= 1
            if depth == 0:
                first_line = source.count("\n", 0, start) + 1
                return source[start : position + 1], first_line
    raise ValueError(f"unterminated function: {signature}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("commands_rs", type=Path)
    args = parser.parse_args()

    source = args.commands_rs.read_text()
    function, first_line = extract_function(
        source,
        "fn ensure_completed(cb: &CommandBuffer) -> Result<(), MetalKernelError>",
    )

    status_offsets = []
    cursor = 0
    while (offset := function.find("cb.status()", cursor)) != -1:
        status_offsets.append(offset)
        cursor = offset + 1

    wait_offsets = []
    cursor = 0
    while (offset := function.find("cb.wait_until_completed()", cursor)) != -1:
        wait_offsets.append(offset)
        cursor = offset + 1

    error_reads = function.count(".error()")
    ok_offset = function.rfind("Ok(())")
    last_wait = max(wait_offsets, default=-1)
    post_wait_status_reads = [offset for offset in status_offsets if offset > last_wait]

    result = {
        "source": str(args.commands_rs.resolve()),
        "function_first_line": first_line,
        "status_reads": len(status_offsets),
        "wait_calls": len(wait_offsets),
        "post_wait_status_reads": len(post_wait_status_reads),
        "error_reads_in_initial_status_match": error_reads,
        "success_after_waits": ok_offset > last_wait,
    }
    print(json.dumps(result, indent=2, sort_keys=True))

    expected_gap = (
        len(status_offsets) == 1
        and len(wait_offsets) == 2
        and not post_wait_status_reads
        and error_reads == 1
        and result["success_after_waits"]
    )
    if not expected_gap:
        raise SystemExit(
            "source no longer has the verified pre-wait-only status-check shape"
        )


if __name__ == "__main__":
    main()
