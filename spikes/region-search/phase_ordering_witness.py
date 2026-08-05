#!/usr/bin/env python3
"""Which search formalisms reach a mutually-enabled composition, and which cannot.

This is a model of an *ordering* structure, not of Tiler's IR. It abstracts one
program into a state — which softmax fold form it uses, and whether the score
matrix is materialized — and abstracts the flash-attention decomposition into two
rewrite classes that enable each other. Nothing here reads a `SemanticProgram`,
computes a real cost, or proves any rewrite sound; the weights are ordinal and
chosen to encode the enabling relation, and the whole point is which strategies
reach the composed state given that relation.

What it establishes, and the boundary is the value: the phase-ordering hazard
separates *cost-pruning* search from *alternative-retaining* search. It does not
separate a Cascades-style memo from an e-graph — both retain the intermediate
that a greedy rewriter discards, and both reach the composition. A reader looking
here for a reason to prefer one of those two over the other will not find one, by
construction.

Run from the repository root:

    uv run python spikes/region-search/phase_ordering_witness.py
    uv run python -O spikes/region-search/phase_ordering_witness.py
"""

from dataclasses import dataclass
from itertools import permutations


class WitnessFailure(RuntimeError):
    """A phase-ordering witness did not hold."""


def require(condition, message):
    if not condition:
        raise WitnessFailure(message)


# --------------------------------------------------------------------------
# The state space
# --------------------------------------------------------------------------
#
# The naive attention chain is `S = Q Kt`, `P = softmax(S)`, `O = P V`. Two
# independent choices describe every program this model can spell:
#
#   fold        "global"    softmax reads all of S before producing any of P
#               "online"    softmax carries a running max and a running sum and
#                           rescales, so a prefix of S suffices to advance it
#   materialize  True       S is a materialized intermediate value
#                False      S never exists as a whole value
#
# The flash-class program is (online, no-S). The naive one is (global, S).


@dataclass(frozen=True)
class State:
    fold: str
    materialize: bool

    def label(self) -> str:
        return f"{self.fold}/{'S-materialized' if self.materialize else 'no-S'}"


NAIVE = State("global", True)
ENABLER = State("online", True)
FLASH = State("online", False)


def legal(state: State) -> bool:
    """A global fold cannot run without the whole of S existing somewhere.

    This is one half of the enabling relation, and it is the half an
    implementation would have to establish rather than assume: a fold whose
    combine reads the complete contributor set before emitting its first result
    forces its input to be live in full, so `(global, no-S)` is not a program
    this model can spell.
    """
    return not (state.fold == "global" and not state.materialize)


def weights(fold_penalty: int):
    """Ordinal weights encoding the other half of the enabling relation.

    Materializing S dominates: it is the quadratic intermediate. The online fold
    costs `fold_penalty` extra arithmetic (the rescale) and buys nothing while S
    is materialized, which is exactly why a cost-pruning search rejects it
    *there* — and why the composition is profitable only after both moves.

    `fold_penalty` is the single knob the falsification witness turns.
    """

    def cost(state: State) -> int:
        return (100 if state.materialize else 10) + (
            fold_penalty if state.fold == "online" else 0
        )

    return cost


COST = weights(3)
FLAT = weights(0)
DISCOUNTED = weights(-3)


# --------------------------------------------------------------------------
# The two rewrite classes
# --------------------------------------------------------------------------


def rewrite_fold(state: State) -> list[State]:
    """The algebraic class: state the shifted-max rescaling identity.

    Owned by the softmax family in Tiler's terms, and gated on a numerical
    permission this model does not represent.
    """
    if state.fold == "global":
        return [State("online", state.materialize)]
    return []


def rewrite_materialize(state: State) -> list[State]:
    """The physical class: drop the materialization edge for S.

    Owned by cover enumeration in Tiler's terms. It proposes; `legal` refuses.
    """
    if state.materialize:
        return [State(state.fold, False)]
    return []


PHASES = {"fold": rewrite_fold, "materialize": rewrite_materialize}


def successors(state: State) -> list[State]:
    out = []
    for phase in PHASES.values():
        out.extend(item for item in phase(state) if legal(item))
    return out


# --------------------------------------------------------------------------
# Four strategies, each taking its cost authority as an argument
# --------------------------------------------------------------------------


def greedy_phase_ordered(cost, order: tuple[str, ...]) -> State:
    """Run each rewrite class to fixpoint in the given order, destructively.

    A proposal replaces the current program only when it costs strictly less.
    This is the shape of an ordinary rule-application pass driven by a
    profitability check, and it is the shape that loses.
    """
    state = NAIVE
    for name in order:
        while True:
            better = [
                item
                for item in PHASES[name](state)
                if legal(item) and cost(item) < cost(state)
            ]
            if not better:
                break
            state = min(better, key=cost)
    return state


def greedy_interleaved_to_fixpoint(cost) -> State:
    """Interleave both classes to a fixpoint, still destructive and cost-pruned.

    Iterating phases to a fixed point is the standard repair for phase ordering,
    and it does not help here: no single legal step from the naive program lowers
    the cost, so the fixpoint is the starting program.
    """
    state = NAIVE
    while True:
        better = [item for item in successors(state) if cost(item) < cost(state)]
        if not better:
            return state
        state = min(better, key=cost)


def retain_all_alternatives(cost) -> State:
    """Retain every legal reachable alternative without committing, then choose.

    This is what a Cascades memo does with logical alternatives — a group holds
    them, and cost prunes physical plans *within* a group rather than the group's
    members — and what an e-graph does with e-classes, where nothing is ever
    removed. The two formalisms differ in how they store and how they extract;
    on this structure they agree.
    """
    seen = {NAIVE}
    work = [NAIVE]
    while work:
        state = work.pop()
        for item in successors(state):
            if item not in seen:
                seen.add(item)
                work.append(item)
    return min(seen, key=cost)


def retain_with_locally_pruned_frontier(cost) -> State:
    """Retain alternatives, but discard any a cheaper predecessor dominates.

    The dangerous middle. It looks like alternative retention and behaves like
    greedy rewriting, because the flash-enabling intermediate is exactly the
    alternative a local cost comparison removes.
    """
    seen = {NAIVE}
    work = [NAIVE]
    while work:
        state = work.pop()
        for item in successors(state):
            if item in seen or cost(item) >= cost(state):
                continue
            seen.add(item)
            work.append(item)
    return min(seen, key=cost)


ORDERS = tuple(permutations(PHASES))


# --------------------------------------------------------------------------
# Witnesses
# --------------------------------------------------------------------------


def test_the_enabling_relation_is_mutual():
    """Neither move alone reaches a state a cost check would keep."""
    require(
        rewrite_materialize(NAIVE) == [State("global", False)]
        and not legal(State("global", False)),
        "dropping the materialization first did not produce an illegal state",
    )
    require(
        rewrite_fold(NAIVE) == [ENABLER] and legal(ENABLER),
        "rewriting the fold first did not produce the legal enabling state",
    )
    require(
        COST(ENABLER) > COST(NAIVE),
        "rewriting the fold first did not look like a regression",
    )
    require(
        legal(FLASH) and COST(FLASH) < COST(NAIVE),
        "the composed state is not both legal and an improvement",
    )
    require(
        (COST(NAIVE), COST(ENABLER), COST(FLASH)) == (100, 103, 13),
        "the modelled weights drifted from the values the comments state",
    )


def test_every_phase_order_misses_the_composition():
    """Both orders, and the interleaved fixpoint, return the naive program."""
    require(len(ORDERS) == 2, f"expected 2 phase orders over {len(PHASES)} classes")
    for order in ORDERS:
        result = greedy_phase_ordered(COST, order)
        require(
            result == NAIVE,
            f"phase order {order} unexpectedly reached {result.label()}",
        )
    require(
        greedy_interleaved_to_fixpoint(COST) == NAIVE,
        "interleaving the phases to a fixpoint reached the composition",
    )


def test_retaining_alternatives_reaches_the_composition():
    require(
        retain_all_alternatives(COST) == FLASH,
        "alternative retention failed to reach the composed program",
    )


def test_local_cost_pruning_defeats_retention():
    """The check that gives this model its point: retention alone is not enough."""
    require(
        retain_with_locally_pruned_frontier(COST) == NAIVE,
        "locally cost-pruned retention unexpectedly reached the composition",
    )


def test_the_discrimination_comes_from_the_enabling_relation():
    """Turn the one knob and watch three of the four verdicts flip.

    Without this, a pass over four strategies proves nothing: a model in which
    the losing strategies also won would be indistinguishable from a model whose
    strategy implementations are simply wrong. Removing the online fold's penalty
    removes the cost half of the enabling relation, and discounting it inverts
    that half outright.
    """
    require(
        greedy_interleaved_to_fixpoint(FLAT) == NAIVE,
        "removing the penalty alone was expected to leave the fixpoint stuck",
    )
    for cost, name in ((FLAT, "flat"), (DISCOUNTED, "discounted")):
        require(
            retain_all_alternatives(cost) == FLASH,
            f"{name} weights lost the composition under full retention",
        )
    require(
        greedy_phase_ordered(DISCOUNTED, ("fold", "materialize")) == FLASH,
        "discounted weights did not let fold-first greedy reach the composition",
    )
    require(
        greedy_interleaved_to_fixpoint(DISCOUNTED) == FLASH,
        "discounted weights did not let the interleaved fixpoint reach it",
    )
    require(
        retain_with_locally_pruned_frontier(DISCOUNTED) == FLASH,
        "discounted weights did not let pruned retention reach the composition",
    )
    # The legality half is untouched by the knob, so materialize-first still
    # stalls one step short. That asymmetry is the reason a cost repair alone
    # does not answer the phase-ordering question.
    require(
        greedy_phase_ordered(DISCOUNTED, ("materialize", "fold")) == ENABLER,
        "materialize-first greedy was expected to stall at the enabling state",
    )
    require(
        greedy_interleaved_to_fixpoint(COST) == NAIVE,
        "the perturbation leaked into the modelled weights",
    )


if __name__ == "__main__":
    tests = [value for name, value in sorted(globals().items()) if name.startswith("test_")]
    for test in tests:
        test()
    print(f"phase-ordering witness: {len(tests)} witnesses passed")
