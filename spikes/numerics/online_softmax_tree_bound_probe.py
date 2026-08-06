#!/usr/bin/env python3
"""Check the hand-derived tree-fold online-softmax bound against exact evaluation.

The bound this probe checks is derived in
`docs/research/numerics/tree-fold-online-softmax-bound.md`, which generalizes the
sequential derivation in
`docs/research/numerics/certified-bounds-as-rewrite-permissions.md` to the merge
tree a parallel flash-class schedule actually folds. The classical results it
rests on are the same three restated by Boldo, Jeannerod, Melquiond and Muller
(Acta Numerica 32, 2023, equations 2.5a, 4.7 and 4.8), preserved under
`docs/research/numerics/sources/acta-numerica-fp-2023/`.

This probe imports the retained sequential probe rather than copying it
-----------------------------------------------------------------------
`online_softmax_bound_probe.py` beside this file owns the binary32 simulation,
the exact-rational `gamma`, the logit corpus, and the sequential price. Importing
it means the two probes cannot drift apart in their arithmetic, and it makes two
cross-checks possible that a copy could not support:

* the general price formula derived here, instantiated at the sequential fold's
  own counts, must equal that probe's `rewrite_price` *exactly*; and
* the degenerate left-deep merge tree over single-contributor leaves must
  reproduce that probe's `online_normalizer` and `two_pass_normalizer`
  **bit for bit**, because the merge operator specializes to Algorithm 3's
  recurrence when one side is always a fresh leaf.

Both checks fail loudly if either side moves, which is the point of running them.

What the probe establishes and what it does not
-----------------------------------------------
It evaluates a declared population of (logit set, fold shape) pairs in exact
binary32 semantics against a 120-digit decimal reference and checks the derived
bounds on each. A violation would refute the derivation; agreement over this
finite population does not prove it. The record labels the derivation
`sound-proof` on its algebra and this probe `bounded-measurement` on its numbers.

One check here is deliberately weaker than the others and is named as such. The
*divergence* between the two folds is checked against two quantities: the sum of
the two folds' bounds, which is derived and rigorous, and the rewrite price,
which is the ratio of those two bounds and therefore bounds the extra budget the
rewrite consumes rather than the realized divergence. The price check is retained
because it is the detector that fires on a structurally wrong fold, but a
violation of it would be a signal to investigate rather than a refutation, and
the record says so where it reports these numbers.

Run from anywhere; the probe resolves its own directory:

    python3 spikes/numerics/online_softmax_tree_bound_probe.py
    python3 -O spikes/numerics/online_softmax_tree_bound_probe.py

Verdicts are explicit checks rather than `assert`, so optimized Python cannot
discard them. Either command exits nonzero instead of publishing JSON when a
derived bound is violated, when a side condition is reached, when a cross-check
against the retained probe moves, or when the evaluated population does not match
the declared one.
"""

from __future__ import annotations

import hashlib
import json
import platform
import sys
from decimal import Decimal
from fractions import Fraction
from pathlib import Path

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

import online_softmax_bound_probe as sequential  # noqa: E402  (path set above)

# Every constant of the arithmetic model comes from the retained probe, so that
# no second definition of the format, the unit roundoff, or the elementary
# accuracy can exist to disagree with the first.
U = sequential.U
EPS_EXP = sequential.EPS_EXP

# Smallest positive binary32 normal. Below it the standard model's relative-error
# form (2.5a) does not hold, so an intermediate that lands there voids the
# derivation rather than merely loosening it, and the probe refuses instead of
# reporting a bound it cannot justify.
SMALLEST_NORMAL = 2.0**-126

# The declared size of the evaluated population, written here rather than
# recomputed from the generators, for the reason the retained probe's own
# `DECLARED_CASES` states: a population check whose two sides come from one
# source cannot say no.
DECLARED_SHAPES = 13
DECLARED_LOGIT_SETS = 14
DECLARED_ROWS = 91

# The contributor counts the general price formula is specialized at when it is
# checked against the retained probe's sequential price.
SPECIALIZATION_COUNTS = (2, 8, 64, 512, 8192)

_EXP_CACHE: dict[float, float] = {}


def fl_exp(argument: float) -> float:
    """Memoized `sequential.fl_exp`. Pure function, so the cache is invisible."""
    cached = _EXP_CACHE.get(argument)
    if cached is None:
        cached = sequential.fl_exp(argument)
        _EXP_CACHE[argument] = cached
    return cached


def is_subnormal(value: float) -> bool:
    """True for a nonzero binary32 magnitude below the smallest normal."""
    return value != 0.0 and abs(value) < SMALLEST_NORMAL


class Intermediates:
    """Counts the intermediates that reach the band where (2.5a) stops holding.

    Carried through evaluation rather than reconstructed afterwards, because the
    condition is about values the fold produced and not about the inputs.
    """

    def __init__(self) -> None:
        self.subnormal = 0

    def note(self, value: float) -> float:
        if is_subnormal(value):
            self.subnormal += 1
        return value


# ---------------------------------------------------------------------------
# Fold shapes
#
# A shape is an explicit tree, not a formula, so that the evaluated arithmetic
# and the counted depths come from one structure and a shape that is not the
# tree it claims to be cannot exist.
# ---------------------------------------------------------------------------


def build_tree(indices: list[int], shape: str) -> object:
    """Returns a nested `(left, right)` tree over `indices`, or a bare index.

    `serial` is the left-deep tree the sequential recurrence walks; `balanced`
    splits each range in half, which is the pairwise tree a register-transfer
    combine or a staged workgroup fold realizes.
    """
    if len(indices) == 1:
        return indices[0]
    if shape == "serial":
        return (build_tree(indices[:-1], shape), indices[-1])
    if shape == "balanced":
        middle = len(indices) // 2
        return (build_tree(indices[:middle], shape), build_tree(indices[middle:], shape))
    raise ValueError(f"unknown tree shape: {shape}")


def leaf_depths(tree: object, depth: int = 0) -> dict[int, int]:
    """Maps each leaf index to the number of internal nodes above it."""
    if isinstance(tree, int):
        return {tree: depth}
    left, right = tree  # type: ignore[misc]
    depths = leaf_depths(left, depth + 1)
    depths.update(leaf_depths(right, depth + 1))
    return depths


class FoldShape:
    """One declared fold shape: a block partition, an intra sum, and a merge tree.

    `blocks` gives the contributor count of each leaf state in order. A leaf state
    is formed by a *local* two-pass — its own maximum, then its own sum — which is
    what a tile, a lane's serial prefix, or a single contributor all reduce to; a
    block of size one is the single-contributor leaf and needs no special case.
    The leaf states are then merged by the rescaling operator over `merge`.
    """

    def __init__(self, name: str, blocks: tuple[int, ...], intra: str, merge: str) -> None:
        self.name = name
        self.blocks = blocks
        self.intra = intra
        self.merge = merge
        self.contributors = sum(blocks)
        self.merge_tree = build_tree(list(range(len(blocks))), merge)
        self.rescale_depth = leaf_depths(self.merge_tree)
        self.intra_trees = [build_tree(list(range(size)), intra) for size in blocks]
        self.intra_depths = [leaf_depths(tree) for tree in self.intra_trees]

    def counts(self) -> dict[str, int]:
        """The four shape parameters the bound is a function of.

        `exp_calls` and `roundings` are maxima over contributors of the number of
        elementary evaluations and of the multiply/add roundings on that
        contributor's root path; `baseline_adds` is the same maximum for the
        matched two-pass fold over the identical tree; `rescale_depth` is the
        number of rescaling merge levels on the deepest path. Each maximum is
        taken independently, which is a valid upper bound even on a ragged tree
        where two of them are attained at different contributors.
        """
        exp_calls = 0
        roundings = 0
        baseline_adds = 0
        for block, depths in enumerate(self.intra_depths):
            rescales = self.rescale_depth[block]
            for intra in depths.values():
                exp_calls = max(exp_calls, 1 + rescales)
                roundings = max(roundings, intra + 2 * rescales)
                baseline_adds = max(baseline_adds, intra + rescales)
        return {
            "exp_calls": exp_calls,
            "roundings": roundings,
            "baseline_adds": baseline_adds,
            "rescale_depth": max(self.rescale_depth.values()),
        }


def shapes() -> list[FoldShape]:
    """The declared fold shapes, and why each is here.

    Every shape is a schedule somebody would actually write, plus two controls.
    """
    declared: list[FoldShape] = []

    # V = 512. The prefill row length the attention work names, at the block
    # sizes a flash-class kernel picks.

    # The degenerate left-deep tree over single-contributor leaves. This is
    # Algorithm 3, and it is here to be compared bit for bit against the retained
    # probe's own sequential fold.
    declared.append(FoldShape("v512-alg3-serial", (1,) * 512, "balanced", "serial"))
    # The pure pairwise merge tree: the cheapest rescale depth reachable at all.
    declared.append(FoldShape("v512-binary-tree", (1,) * 512, "balanced", "balanced"))
    # Block-local two-pass, then a pairwise merge across blocks, at the three
    # block sizes the ticket names. Both readings of "B" are covered, because
    # 16x32 and 32x16 both appear.
    declared.append(FoldShape("v512-block16-tree", (16,) * 32, "balanced", "balanced"))
    declared.append(FoldShape("v512-block32-tree", (32,) * 16, "balanced", "balanced"))
    declared.append(FoldShape("v512-block64-tree", (64,) * 8, "balanced", "balanced"))
    # The flash-realistic shape: blocks reduced in parallel, then merged by a
    # *sequential* outer loop, which is what a loop-carried streaming schedule
    # does and which is where the rescale depth stops being logarithmic.
    declared.append(FoldShape("v512-block32-serial-outer", (32,) * 16, "balanced", "serial"))
    # The opposite mix: a long serial prefix inside each block, a tree across
    # them. Its rescale depth is tiny and its baseline height is large, which
    # separates the two shape parameters from each other.
    declared.append(FoldShape("v512-block64-serial-intra", (64,) * 8, "serial", "balanced"))
    # Control: one block is the two-pass fold itself. No merge occurs, so the
    # derived price is exactly zero and the observed divergence must be too.
    declared.append(FoldShape("v512-one-block", (512,), "balanced", "balanced"))
    # A ragged, deliberately unbalanced tree. Block sizes differ by a factor of
    # 256 and the merge tree is a caterpillar, so the three maxima in `counts`
    # are attained at different contributors.
    declared.append(
        FoldShape("v512-ragged-caterpillar", (1, 1, 2, 4, 8, 16, 32, 64, 128, 256), "balanced", "serial")
    )

    # V = 64. A second contributor count, so no reported dependence on the shape
    # parameters can be an artefact of one V.
    declared.append(FoldShape("v64-alg3-serial", (1,) * 64, "balanced", "serial"))
    declared.append(FoldShape("v64-binary-tree", (1,) * 64, "balanced", "balanced"))
    declared.append(FoldShape("v64-block8-tree", (8,) * 8, "balanced", "balanced"))
    declared.append(FoldShape("v64-one-block", (64,), "balanced", "balanced"))

    return declared


# ---------------------------------------------------------------------------
# Evaluation, in exact binary32 semantics
# ---------------------------------------------------------------------------


def sum_over_tree(terms: list[float], tree: object, notes: Intermediates) -> float:
    """Sums `terms` in the grouping `tree` states, with one rounding per node."""
    if isinstance(tree, int):
        return terms[tree]
    left, right = tree  # type: ignore[misc]
    return notes.note(
        sequential.fl_add(sum_over_tree(terms, left, notes), sum_over_tree(terms, right, notes))
    )


def block_state(values: list[float], tree: object, notes: Intermediates) -> tuple[float, float]:
    """Forms one leaf state by a block-local two-pass.

    The block's own maximum is exact — a selection among representable values —
    and every term is `exp` of one rounded difference against it. A block of size
    one produces `exp(0)`, which the derivation charges an `eps_exp` for even
    where the implementation would return an exact `1`; the bound is conservative
    there rather than special-cased.
    """
    peak = values[0]
    for value in values[1:]:
        peak = max(peak, value)
    terms = [notes.note(fl_exp(notes.note(sequential.fl_sub(value, peak)))) for value in values]
    return peak, sum_over_tree(terms, tree, notes)


def merge(
    left: tuple[float, float], right: tuple[float, float], notes: Intermediates
) -> tuple[float, float]:
    """`(m1,d1) + (m2,d2) = (max, d1*exp(m1-max) + d2*exp(m2-max))`, as spelled.

    Both rescale factors are evaluated, including the winning side's `exp(0)`.
    That is what the operator says, and it is also the conservative reading: a
    target whose `exp` returns an exact `1` at zero makes the winning side's
    multiply exact, which can only lower the realized error below the bound.
    """
    left_peak, left_sum = left
    right_peak, right_sum = right
    peak = max(left_peak, right_peak)
    left_scale = notes.note(fl_exp(notes.note(sequential.fl_sub(left_peak, peak))))
    right_scale = notes.note(fl_exp(notes.note(sequential.fl_sub(right_peak, peak))))
    total = notes.note(
        sequential.fl_add(
            notes.note(sequential.fl_mul(left_sum, left_scale)),
            notes.note(sequential.fl_mul(right_sum, right_scale)),
        )
    )
    return peak, total


def online_tree_normalizer(
    logits: list[float], shape: FoldShape, notes: Intermediates
) -> float:
    """Folds `logits` by `shape`'s merge tree over block-local leaf states."""
    states = []
    offset = 0
    for index, size in enumerate(shape.blocks):
        states.append(block_state(logits[offset : offset + size], shape.intra_trees[index], notes))
        offset += size

    def walk(tree: object) -> tuple[float, float]:
        if isinstance(tree, int):
            return states[tree]
        left, right = tree  # type: ignore[misc]
        return merge(walk(left), walk(right), notes)

    return walk(shape.merge_tree)[1]


def matched_two_pass_normalizer(
    logits: list[float], shape: FoldShape, notes: Intermediates
) -> float:
    """The two-pass fold summed over the *same* tree, which is the honest baseline.

    A global maximum, then one `exp` per contributor, then the identical grouping
    of additions the online fold uses. Comparing the online tree fold against a
    *sequential* two-pass would credit the rewrite for a reassociation that is a
    separate permission, so this baseline is the shape-matched one.
    """
    peak = logits[0]
    for value in logits[1:]:
        peak = max(peak, value)
    terms = [notes.note(fl_exp(notes.note(sequential.fl_sub(value, peak)))) for value in logits]

    partials: list[float] = []
    offset = 0
    for index, size in enumerate(shape.blocks):
        partials.append(
            sum_over_tree(terms[offset : offset + size], shape.intra_trees[index], notes)
        )
        offset += size
    return sum_over_tree(partials, shape.merge_tree, notes)


# ---------------------------------------------------------------------------
# The bounds
# ---------------------------------------------------------------------------


def _decimal(value: Fraction) -> Decimal:
    return Decimal(value.numerator) / Decimal(value.denominator)


def online_tree_bound(counts: dict[str, int], spread: Fraction) -> Decimal:
    """`(1 + eps_exp)^E * exp(A*u) * (1 + gamma_N) - 1`.

    `E` is the number of elementary evaluations on the deepest contributor's root
    path and `N` the number of multiply/add roundings on it. The
    argument-perturbation factor is `exp(A*u)` and not `exp(E*A*u)`: the rescale
    arguments telescope along any root path of any tree, because the subtree
    maxima above a contributor are non-decreasing and the first of them already
    dominates it, so their perturbations sum to `(m_V - x_j)*u` exactly as they do
    in the sequential chain. That is the derivation's load-bearing step and the
    reason this bound is a function of the tree's depth and of no input value.
    """
    elementary = (Decimal(1) + _decimal(EPS_EXP)) ** counts["exp_calls"]
    summation = Decimal(1) + _decimal(sequential.gamma(counts["roundings"]))
    return elementary * sequential.exp_of_fraction(spread * U) * summation - 1


def matched_two_pass_bound(counts: dict[str, int], spread: Fraction) -> Decimal:
    """`(1 + eps_exp) * exp(A*u) * (1 + gamma_{h2}) - 1` at the same tree."""
    elementary = Decimal(1) + _decimal(EPS_EXP)
    summation = Decimal(1) + _decimal(sequential.gamma(counts["baseline_adds"]))
    return elementary * sequential.exp_of_fraction(spread * U) * summation - 1


def tree_rewrite_price(counts: dict[str, int]) -> Decimal:
    """The extra relative budget the rewrite consumes over its matched baseline.

    `(1 + eps_exp)^(E-1) * (1 + gamma_N) / (1 + gamma_{h2}) - 1`, which is the
    ratio of the two bounds above with the spread factor cancelled. The spread is
    absent by construction, so the price is instantiable at compile time from the
    fold tree and the target profile alone.

    In every regular shape `E - 1 = N - h2 = D`, the rescale depth, so the price
    collapses to the first-order `D*(u + eps_exp)`. It is spelled from the counts
    rather than from `D` because a ragged tree can attain the three maxima at
    different contributors, and then the counts are the honest statement.
    """
    if counts["exp_calls"] <= 1:
        return Decimal(0)
    elementary = (Decimal(1) + _decimal(EPS_EXP)) ** (counts["exp_calls"] - 1)
    wide = Decimal(1) + _decimal(sequential.gamma(counts["roundings"]))
    narrow = Decimal(1) + _decimal(sequential.gamma(counts["baseline_adds"]))
    return elementary * wide / narrow - 1


def price_from_counts(exp_calls: int, roundings: int, baseline_adds: int) -> Decimal:
    """`tree_rewrite_price` over loose counts, for the specialization cross-check."""
    return tree_rewrite_price(
        {"exp_calls": exp_calls, "roundings": roundings, "baseline_adds": baseline_adds}
    )


# ---------------------------------------------------------------------------
# The population
# ---------------------------------------------------------------------------


def logit_sets() -> list[tuple[str, list[float]]]:
    """The retained probe's adversarial corpus, restricted to the two counts used.

    Reusing that corpus rather than restating it is deliberate: the cases were
    chosen because they exercise a moving, a frozen, and an alternating maximum,
    and a tree fold has no reason to want different logits — only different
    groupings of the same ones.
    """
    return [(name, values) for name, values in sequential.corpus() if len(values) in (64, 512)]


def evaluate(name: str, logits: list[float], shape: FoldShape) -> dict[str, object]:
    counts = shape.counts()
    spread = sequential.argument_spread(logits)
    reference = sequential.reference_normalizer(logits)
    if reference <= 0:
        raise ValueError(f"{name}/{shape.name}: the reference normalizer is not positive")

    notes = Intermediates()
    online = Decimal(online_tree_normalizer(logits, shape, notes))
    two_pass = Decimal(matched_two_pass_normalizer(logits, shape, notes))

    online_error = abs(online - reference) / reference
    two_pass_error = abs(two_pass - reference) / reference
    divergence = abs(online - two_pass) / reference

    online_bound = online_tree_bound(counts, spread)
    two_pass_bound = matched_two_pass_bound(counts, spread)

    row: dict[str, object] = {
        "case": name,
        "shape": shape.name,
        "contributors": shape.contributors,
        "blocks": len(shape.blocks),
        "argument_spread": float(spread),
        "exp_calls": counts["exp_calls"],
        "roundings": counts["roundings"],
        "baseline_adds": counts["baseline_adds"],
        "rescale_depth": counts["rescale_depth"],
        "subnormal_intermediates": notes.subnormal,
        "online_relative_error": sequential._magnitude(online_error),
        "online_bound": sequential._magnitude(online_bound),
        "online_bound_over_observed": sequential._ratio(online_bound, online_error),
        "two_pass_relative_error": sequential._magnitude(two_pass_error),
        "two_pass_bound": sequential._magnitude(two_pass_bound),
        "observed_divergence": sequential._magnitude(divergence),
        "derived_price": sequential._magnitude(tree_rewrite_price(counts)),
        "sum_of_bounds": sequential._magnitude(online_bound + two_pass_bound),
    }

    # The degenerate shape is the retained probe's own fold, so it must agree at
    # the bit. Recorded on the row rather than checked here, so the verdict is
    # visible in the published record and not only in an exit status.
    if shape.name.endswith("alg3-serial"):
        row["matches_sequential_online"] = float(online) == sequential.online_normalizer(logits)
        row["matches_sequential_two_pass"] = float(two_pass) == sequential.two_pass_normalizer(
            logits
        )
    return row


def check(rows: list[dict[str, object]], declared: list[FoldShape]) -> list[str]:
    """Returns the failures. Explicit rather than `assert`, so `-O` cannot drop them."""
    failures: list[str] = []

    for row in rows:
        label = f"{row['case']}/{row['shape']}"
        if Decimal(str(row["online_relative_error"])) > Decimal(str(row["online_bound"])):
            failures.append(f"{label}: online tree error exceeds its derived bound")
        if Decimal(str(row["two_pass_relative_error"])) > Decimal(str(row["two_pass_bound"])):
            failures.append(f"{label}: matched two-pass error exceeds its derived bound")
        # Derived and rigorous: two results each inside their own bracket can
        # differ by at most the sum of the two brackets.
        if Decimal(str(row["observed_divergence"])) > Decimal(str(row["sum_of_bounds"])):
            failures.append(f"{label}: divergence exceeds the sum of the two derived bounds")
        # Heuristic detector, not a derived bound: the price is the *ratio* of the
        # two bounds, so it bounds the extra budget rather than the realized
        # divergence. It fires on a structurally wrong fold, which is why it is
        # kept; a violation is a signal to investigate, not a refutation.
        if Decimal(str(row["observed_divergence"])) > Decimal(str(row["derived_price"])):
            failures.append(
                f"{label}: divergence exceeds the derived price (heuristic detector)"
            )
        # A subnormal intermediate voids (2.5a) outright, so the bound above would
        # be a claim the model does not support.
        if row["subnormal_intermediates"] != 0:
            failures.append(
                f"{label}: {row['subnormal_intermediates']} subnormal intermediate(s) reached,"
                " where the relative-error model does not hold"
            )
        if row["shape"].endswith("alg3-serial"):  # type: ignore[union-attr]
            if not row.get("matches_sequential_online"):
                failures.append(
                    f"{label}: the left-deep merge tree does not reproduce the retained"
                    " probe's online fold at the bit"
                )
            if not row.get("matches_sequential_two_pass"):
                failures.append(
                    f"{label}: the left-deep baseline does not reproduce the retained"
                    " probe's two-pass fold at the bit"
                )

    # The general price, specialized at Algorithm 3's own counts, must be the
    # sequential price the retained probe computes. This is what makes the tree
    # form a generalization rather than a second, unrelated formula.
    for count in SPECIALIZATION_COUNTS:
        general = price_from_counts(count, 2 * (count - 1), count - 1)
        if general != sequential.rewrite_price(count):
            failures.append(
                f"specialization mismatch at V={count}: the general price does not equal"
                " the retained probe's sequential price"
            )

    for shape in declared:
        covered = sorted(shape.rescale_depth)
        if covered != list(range(len(shape.blocks))):
            failures.append(f"{shape.name}: the merge tree does not cover its blocks exactly once")
        for index, size in enumerate(shape.blocks):
            if sorted(shape.intra_depths[index]) != list(range(size)):
                failures.append(
                    f"{shape.name}: block {index} does not cover its {size} contributors exactly once"
                )

    if len(declared) != DECLARED_SHAPES:
        failures.append(
            f"shape population mismatch: declared {len(declared)} shapes, expected {DECLARED_SHAPES}"
        )
    if len(logit_sets()) != DECLARED_LOGIT_SETS:
        failures.append(
            f"logit-set population mismatch: found {len(logit_sets())} sets,"
            f" expected {DECLARED_LOGIT_SETS}"
        )
    if len(rows) != DECLARED_ROWS:
        failures.append(f"population mismatch: evaluated {len(rows)} rows, expected {DECLARED_ROWS}")
    return failures


def main() -> int:
    source = Path(__file__).resolve()
    declared = shapes()
    rows: list[dict[str, object]] = []
    for name, logits in logit_sets():
        for shape in declared:
            if shape.contributors == len(logits):
                rows.append(evaluate(name, logits, shape))

    failures = check(rows, declared)
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        print(f"{len(failures)} check(s) failed over {len(rows)} rows.", file=sys.stderr)
        return 1

    retained = (_HERE / "online_softmax_bound_probe.py").resolve()
    record = {
        "probe": source.name,
        "probe_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "imported_probe": retained.name,
        "imported_probe_sha256": hashlib.sha256(retained.read_bytes()).hexdigest(),
        "oracle": {
            "library": "decimal (Python standard library)",
            "digits": sequential.ORACLE_DIGITS,
        },
        "format": {
            "name": "binary32",
            "unit_roundoff": "2**-24",
            "smallest_normal": "2**-126",
        },
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
        "declared_shapes": DECLARED_SHAPES,
        "evaluated_shapes": len(declared),
        "declared_logit_sets": DECLARED_LOGIT_SETS,
        "declared_rows": DECLARED_ROWS,
        "evaluated_rows": len(rows),
        "specialization_counts": list(SPECIALIZATION_COUNTS),
        "shapes": [
            {
                "shape": shape.name,
                "contributors": shape.contributors,
                "blocks": len(shape.blocks),
                "block_sizes": list(shape.blocks),
                "intra": shape.intra,
                "merge": shape.merge,
                **shape.counts(),
                "derived_price": sequential._magnitude(tree_rewrite_price(shape.counts())),
            }
            for shape in declared
        ],
        "results": rows,
    }
    print(json.dumps(record, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
