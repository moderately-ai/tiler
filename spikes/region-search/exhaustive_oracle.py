#!/usr/bin/env python3
"""Tiny exhaustive oracle for Tiler fusion-region search.

This is intentionally exponential and restricted to tiny test DAGs. It is a
specification witness for future bounded heuristics, not a production planner.
"""

from dataclasses import dataclass
from itertools import combinations


@dataclass(frozen=True)
class Node:
    name: str
    inputs: tuple[str, ...] = ()
    fusible: bool = True
    duplicable: bool = False


@dataclass(frozen=True)
class Graph:
    nodes: tuple[Node, ...]
    outputs: tuple[str, ...]
    blocked_edges: frozenset[tuple[str, str]] = frozenset()

    def by_name(self):
        return {node.name: node for node in self.nodes}

    def users(self):
        result = {node.name: set() for node in self.nodes}
        for node in self.nodes:
            for value in node.inputs:
                if value in result:
                    result[value].add(node.name)
        return result


@dataclass(frozen=True)
class RegionCandidate:
    nodes: frozenset[str]
    boundary_inputs: frozenset[str]
    boundary_outputs: frozenset[str]


@dataclass(frozen=True)
class RegionRejection:
    nodes: frozenset[str]
    reason: str


def _connected(graph: Graph, selected: frozenset[str]) -> bool:
    neighbors = {name: set() for name in selected}
    for node in graph.nodes:
        if node.name not in selected:
            continue
        for source in node.inputs:
            if source in selected:
                neighbors[node.name].add(source)
                neighbors[source].add(node.name)
    reached = set()
    work = [next(iter(selected))]
    while work:
        name = work.pop()
        if name not in reached:
            reached.add(name)
            work.extend(neighbors[name] - reached)
    return reached == set(selected)


def _has_path_leaving_and_reentering(graph: Graph, selected: frozenset[str]) -> bool:
    users = graph.users()
    for start in selected:
        work = [(start, False)]
        seen = set()
        while work:
            name, left = work.pop()
            state = (name, left)
            if state in seen:
                continue
            seen.add(state)
            for user in users[name]:
                next_left = left or user not in selected
                if user in selected and next_left:
                    return True
                work.append((user, next_left))
    return False


def classify_region(graph: Graph, selected: frozenset[str], maximum_nodes: int = 8):
    by_name = graph.by_name()
    if not selected or not selected <= by_name.keys():
        return RegionRejection(selected, "unknown-or-empty")
    if len(selected) > maximum_nodes:
        return RegionRejection(selected, "search-bound")
    if any(not by_name[name].fusible for name in selected):
        return RegionRejection(selected, "operation-boundary")
    if len(selected) > 1 and not _connected(graph, selected):
        return RegionRejection(selected, "disconnected")
    if _has_path_leaving_and_reentering(graph, selected):
        return RegionRejection(selected, "non-convex")
    for source, target in graph.blocked_edges:
        if source in selected and target in selected:
            return RegionRejection(selected, "incompatible-internal-edge")

    users = graph.users()
    boundary_inputs = set()
    boundary_outputs = set()
    for name in selected:
        node = by_name[name]
        boundary_inputs.update(value for value in node.inputs if value not in selected)
        if name in graph.outputs or any(user not in selected for user in users[name]):
            boundary_outputs.add(name)
    return RegionCandidate(selected, frozenset(boundary_inputs), frozenset(boundary_outputs))


def enumerate_regions(graph: Graph):
    names = tuple(node.name for node in graph.nodes)
    accepted = []
    rejected = []
    for count in range(1, len(names) + 1):
        for subset in combinations(names, count):
            result = classify_region(graph, frozenset(subset))
            (accepted if isinstance(result, RegionCandidate) else rejected).append(result)
    return accepted, rejected


def enumerate_exact_partitions(graph: Graph, candidates: list[RegionCandidate]):
    all_nodes = frozenset(node.name for node in graph.nodes)
    result = []

    def visit(uncovered, chosen):
        if not uncovered:
            result.append(tuple(chosen))
            return
        anchor = min(uncovered)
        for candidate in candidates:
            if anchor in candidate.nodes and candidate.nodes <= uncovered:
                visit(uncovered - candidate.nodes, chosen + [candidate])

    visit(all_nodes, [])
    return result


def enumerate_duplication_plans(graph: Graph, candidates: list[RegionCandidate]):
    """Enumerate covers where only explicitly duplicable nodes may overlap."""
    names = {node.name for node in graph.nodes}
    duplicable = {node.name for node in graph.nodes if node.duplicable}
    plans = []
    for count in range(1, len(candidates) + 1):
        for chosen in combinations(candidates, count):
            occurrences = {name: 0 for name in names}
            for candidate in chosen:
                for name in candidate.nodes:
                    occurrences[name] += 1
            if any(occurrences[name] == 0 for name in names):
                continue
            overlaps = {name for name, amount in occurrences.items() if amount > 1}
            if overlaps <= duplicable:
                plans.append((chosen, frozenset(overlaps)))
    return plans


@dataclass(frozen=True)
class Implementation:
    name: str
    region: frozenset[str]
    applicability: frozenset[str]
    estimated_cost: int
    registers: int


def implementation_frontier(implementations: list[Implementation]):
    """Remove dominance only inside identical coverage/applicability classes."""
    frontier = []
    for candidate in implementations:
        dominated = any(
            other is not candidate
            and other.region == candidate.region
            and other.applicability == candidate.applicability
            and other.estimated_cost <= candidate.estimated_cost
            and other.registers <= candidate.registers
            and (other.estimated_cost, other.registers)
            != (candidate.estimated_cost, candidate.registers)
            for other in implementations
        )
        if not dominated:
            frontier.append(candidate)
    return frontier


def test_chain_convexity_and_partitions():
    graph = Graph((Node("a"), Node("b", ("a",)), Node("c", ("b",))), ("c",))
    accepted, rejected = enumerate_regions(graph)
    accepted_sets = {candidate.nodes for candidate in accepted}
    assert frozenset(("a", "b", "c")) in accepted_sets
    assert any(item.nodes == frozenset(("a", "c")) and item.reason == "disconnected" for item in rejected)
    partitions = enumerate_exact_partitions(graph, accepted)
    assert len(partitions) == 4  # [abc], [ab][c], [a][bc], [a][b][c]


def test_non_convex_region_is_rejected():
    # a -> b -> d, while a -> c -> d. Selecting a,b,d leaves a path through c.
    graph = Graph(
        (Node("a"), Node("b", ("a",)), Node("c", ("a",)), Node("d", ("b", "c"))),
        ("d",),
    )
    result = classify_region(graph, frozenset(("a", "b", "d")))
    assert isinstance(result, RegionRejection) and result.reason == "non-convex"


def test_shared_producer_multi_output_and_duplication():
    graph = Graph(
        (Node("p", duplicable=True), Node("left", ("p",)), Node("right", ("p",))),
        ("left", "right"),
    )
    accepted, _ = enumerate_regions(graph)
    whole = next(candidate for candidate in accepted if candidate.nodes == frozenset(("p", "left", "right")))
    assert whole.boundary_outputs == frozenset(("left", "right"))
    duplicated = enumerate_duplication_plans(graph, accepted)
    assert any(
        overlap == frozenset(("p",))
        and {candidate.nodes for candidate in plan}
        == {frozenset(("p", "left")), frozenset(("p", "right"))}
        for plan, overlap in duplicated
    )


def test_blocked_edge_preserves_unfused_coverage():
    graph = Graph((Node("a"), Node("b", ("a",))), ("b",), frozenset((("a", "b"),)))
    accepted, rejected = enumerate_regions(graph)
    assert {item.nodes for item in accepted} == {frozenset(("a",)), frozenset(("b",))}
    assert any(item.reason == "incompatible-internal-edge" for item in rejected)
    assert len(enumerate_exact_partitions(graph, accepted)) == 1


def test_frontier_keeps_guarded_alternative_and_removes_dominance():
    region = frozenset(("a", "b"))
    generic = Implementation("generic", region, frozenset(), 20, 8)
    worse = Implementation("worse", region, frozenset(), 30, 10)
    aligned = Implementation("aligned", region, frozenset(("alignment>=16",)), 8, 12)
    frontier = implementation_frontier([generic, worse, aligned])
    assert {item.name for item in frontier} == {"generic", "aligned"}


if __name__ == "__main__":
    tests = [value for name, value in sorted(globals().items()) if name.startswith("test_")]
    for test in tests:
        test()
    print(f"region-search oracle: {len(tests)} witnesses passed")

