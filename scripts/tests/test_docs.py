import importlib.util
import sys
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("tiler_docs", Path(__file__).parents[1] / "docs.py")
docs = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = docs
SPEC.loader.exec_module(docs)


def parse(frontmatter: str, tmp_path: Path):
    path = tmp_path / "README.md"
    path.write_text(frontmatter + "\n# Test\n", encoding="utf-8")
    return docs.parse(path, tmp_path)


def test_accepts_strict_json_subset(tmp_path: Path):
    record, errors = parse(
        """---
schema: "tiler-doc/v1"
id: "tiler.portal.test"
kind: "portal"
title: "Test"
topics: ["test"]
---""",
        tmp_path,
    )
    assert errors == []
    assert record.meta["topics"] == ["test"]


def test_rejects_duplicate_and_nested_values(tmp_path: Path):
    _, errors = parse('---\nid: "one"\nid: "two"\nextra: {"nested": true}\n---', tmp_path)
    assert any("duplicate field id" in error for error in errors)
    assert any("flat scalar array" in error for error in errors)


def test_rejects_unknown_kind_field(tmp_path: Path):
    record, errors = parse(
        """---
schema: "tiler-doc/v1"
id: "tiler.portal.test"
kind: "portal"
title: "Test"
topics: ["test"]
evidence: ["x"]
---""",
        tmp_path,
    )
    assert errors == []
    assert any(
        "unknown field evidence" in error for error in docs.validate_record(record, Path("."))
    )


def test_duplicate_ids_and_unresolved_edges_fail():
    records = [
        docs.Record(
            Path("a.md"),
            {"id": "tiler.x", "kind": "portal", "related": ["tiler.missing"]},
            "",
        ),
        docs.Record(Path("b.md"), {"id": "tiler.x", "kind": "portal"}, ""),
    ]
    errors = docs.validate_graph(records, Path("."))
    assert any("duplicate id" in error for error in errors)
    assert any("unresolved related" in error for error in errors)


def test_related_edge_has_one_canonical_direction():
    records = [
        docs.Record(Path("a.md"), {"id": "tiler.z", "kind": "portal", "related": ["tiler.a"]}, ""),
        docs.Record(Path("b.md"), {"id": "tiler.a", "kind": "portal"}, ""),
    ]
    errors = docs.validate_graph(records, Path("."))
    assert any("lexicographically smaller" in error for error in errors)


def test_dependency_cycles_fail():
    records = [
        docs.Record(
            Path("a.md"),
            {"id": "tiler.a", "kind": "portal", "depends_on": ["tiler.b"]},
            "",
        ),
        docs.Record(
            Path("b.md"),
            {"id": "tiler.b", "kind": "portal", "depends_on": ["tiler.a"]},
            "",
        ),
    ]
    errors = docs.validate_graph(records, Path("."))
    assert any("contains a cycle" in error for error in errors)
