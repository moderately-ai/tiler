#!/usr/bin/env python3
"""Scores the *activated* selector against this spike's retained sweep.

`activate-measured-reduction-selection-from-a-target-cost-row` activates the
fitted work-span model **less its two decision-inert parameters**:

    fold_steps = sum over stages of max( work, depth * P )

which is `P` times `encoder = 0, step = 1` in `src/model.rs`'s `predict`. Two of
the three fitted parameters are therefore not declared on any target profile,
and this script is the evidence that dropping them changes no verdict. `step` is
one positive factor over the whole sum and provably cannot reorder anything;
`encoder` is a per-stage constant that prices dispatch count, which the
compiler's structural cost model already carries as an exact dimension, and
`src/fit.rs`'s perturbation table separately measures it inert. **Provably
order-preserving and measured-inert are different claims, and only the first is
an argument** — this is the measurement for the second.

It reads the recorded `threads:work:depth` triples rather than re-deriving a
launch geometry, so it scores the topology that was dispatched.

Run from the repository root:

    python3 spikes/program-planning/reduction-dispatch-crossover/activated_selector_check.py

Expected output, on the retained 2026-08-07 results: the held-out worst measured
penalty is 1.81x at the fitted value, 3.04x at a quarter of it, and 1.20x at four
times it. Those three numbers are exactly the ones `results/.../perturbations.txt`
reports for the *complete* three-parameter model, which is the finding.

The agreed/total counts are **not** expected to match that file's, and the
difference is a separation rule rather than a disagreement: this script resolves
a cell when the serial fold and the *best parallel* strategy are separated by two
combined standard errors, which is the binary decision the activated selector
makes, while `fit.rs` reports separation over the three-way winner as well.
"""

import math
import os
import sys

RESULTS = "results/2026-08-07-apple-m4-max-macos27.0-26A5388g/sweep.tsv"
FITTED_PARALLEL_THREADS = 1056.0


def load(path):
    header, rows = None, []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("#") or not line.strip():
                continue
            parts = line.rstrip("\n").split("\t")
            if header is None:
                header = parts
                continue
            if not parts[0].strip():
                continue
            rows.append(dict(zip(header, parts)))
    cells = {}
    for row in rows:
        stages = [tuple(int(x) for x in s.split(":")) for s in row["stages"].split("|")]
        cells.setdefault((int(row["rows"]), int(row["contributors"])), {})[row["strategy"]] = {
            "stages": stages,
            "p50": float(row["amortized_p50_us"]),
            "sd": float(row["amortized_stddev_us"]),
            "reps": int(row["reps"]),
        }
    return cells


def predict(stages, parallel_threads):
    """The activated selector: the per-stage maximum, summed."""
    return sum(max(work, depth * parallel_threads) for (_threads, work, depth) in stages)


def separated(left, right):
    error = math.sqrt(left["sd"] ** 2 / left["reps"] + right["sd"] ** 2 / right["reps"])
    return abs(left["p50"] - right["p50"]) > 2 * error


def is_fit_set(contributors):
    """The spike fits on perfect-square contributor counts and holds out the rest."""
    root = math.isqrt(contributors)
    return root * root == contributors


def score(cells, parallel_threads, fit_set):
    total = agreed = 0
    worst, worst_cell = 1.0, None
    for (rows, contributors), strategies in sorted(cells.items()):
        if len(strategies) < 3 or is_fit_set(contributors) != fit_set:
            continue
        fold = strategies["serial-fold"]
        best_parallel = min(
            (strategies["single-workgroup-tree"], strategies["multi-pass-split"]),
            key=lambda entry: entry["p50"],
        )
        if not separated(fold, best_parallel):
            continue
        total += 1
        measured_parallel = best_parallel["p50"] < fold["p50"]
        costs = {name: predict(entry["stages"], parallel_threads) for name, entry in strategies.items()}
        chosen = min(costs, key=lambda name: (costs[name], name))
        if (chosen != "serial-fold") == measured_parallel:
            agreed += 1
            continue
        penalty = strategies[chosen]["p50"] / min(fold["p50"], best_parallel["p50"])
        if penalty > worst:
            worst, worst_cell = penalty, (rows, contributors)
    return total, agreed, worst, worst_cell


def main():
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), RESULTS)
    cells = load(path)
    if not cells:
        print(f"no cells parsed from {path}", file=sys.stderr)
        return 1
    print(f"{len(cells)} cells, {sum(len(s) for s in cells.values())} measured alternatives")
    for label, parallel in (
        ("fitted", FITTED_PARALLEL_THREADS),
        ("x 0.25", FITTED_PARALLEL_THREADS / 4),
        ("x 4", FITTED_PARALLEL_THREADS * 4),
    ):
        for name, fit_set in (("fit", True), ("held-out", False)):
            total, agreed, worst, cell = score(cells, parallel, fit_set)
            where = f" at {cell[0]} x {cell[1]}" if cell else ""
            print(
                f"P {label:6s} ({parallel:7.1f})  {name:8s}  separated cells "
                f"{agreed:3d}/{total:<3d}  worst measured penalty {worst:.2f}x{where}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())
