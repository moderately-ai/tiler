#!/usr/bin/env python3
"""How much of Stage A's error ordering is the calibration's doing rather than the profile's.

Min-max calibration is an ingestion-side choice, not a Tiler semantic, so a reader is
entitled to ask whether the elimination it produces survives a better one. This sweeps
three calibrations -- exact min/max, and two-sided 99.9% and 99% clipping -- over one
complete decoder layer plus the tied embedding, and reports the reconstruction error of
each (profile, calibration) pair.

The subset is bounded deliberately: the question is whether calibration reorders the
*profiles*, and a reordering would show on any representative weight set. The complete
197-tensor reading stays with `weight_error.py`.
"""

from __future__ import annotations

import argparse
from pathlib import Path

import numpy as np

from weight_error import (
    EMBEDDING_NAME,
    PROJECTION_SUFFIXES,
    Profile,
    load_f32,
    read_header,
    resolve_weights_path,
    roundtrip,
    verify_weights,
)

GRANULARITIES = (
    ("per-tensor", None),
    ("per-channel", "row"),
    ("per-group128", 128),
    ("per-group32", 32),
)
CALIBRATIONS = (("minmax", None), ("clip99.9", 0.999), ("clip99", 0.99))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    path = resolve_weights_path()
    verify_weights(path)
    header, base = read_header(path)

    names = [EMBEDDING_NAME] + sorted(
        name
        for name in header
        if name.startswith("model.layers.0.") and name.endswith(PROJECTION_SUFFIXES)
    )

    args.out.mkdir(parents=True, exist_ok=True)
    rows = []
    for name in names:
        weight = load_f32(path, header, base, name)
        reference = weight.astype(np.float64)
        denominator = float(np.sqrt((reference**2).sum()))
        for bits in (4, 8):
            for granularity_name, granularity in GRANULARITIES:
                for calibration_name, quantile in CALIBRATIONS:
                    profile = Profile(
                        f"{granularity_name}-u{bits}", bits, granularity, quantile
                    )
                    decoded, _scale, _zero = roundtrip(weight, profile)
                    residual = reference - decoded.astype(np.float64)
                    rows.append(
                        (
                            name,
                            profile.name,
                            calibration_name,
                            f"{float(np.sqrt((residual**2).sum())) / denominator:.6e}",
                        )
                    )
        del weight, reference

    with (args.out / "calibration-sensitivity.tsv").open("w") as handle:
        handle.write("tensor\tprofile\tcalibration\trelative_frobenius_error\n")
        for row in rows:
            handle.write("\t".join(row) + "\n")

    print(f"wrote {len(rows)} rows over {len(names)} tensors to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
