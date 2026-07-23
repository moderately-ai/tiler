import importlib.util
import sys
from pathlib import Path

REPOSITORY = Path(__file__).parents[2]
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


def test_reference_links_and_images_validate_local_targets(tmp_path: Path):
    (tmp_path / "docs").mkdir()
    record = docs.Record(
        Path("docs/test.md"),
        {"id": "tiler.test", "kind": "portal"},
        "[contract][missing]\n\n[undefined][nowhere]\n\n[missing]\n\n"
        "[missing]: absent.md\n\n![diagram](absent.png)\n",
    )

    errors = docs.validate_links([record], tmp_path)

    assert any("broken local link absent.md" in error for error in errors)
    assert any("broken local link absent.png" in error for error in errors)


def test_commonmark_nested_links_and_images_are_validated(tmp_path: Path):
    record = docs.Record(
        Path("README.md"),
        {"id": "tiler.test", "kind": "portal"},
        "[outer [inner]](missing.md)\n\n![alt [nested]](missing.png)\n",
    )

    errors = docs.validate_links([record], tmp_path)

    assert any("broken local link missing.md" in error for error in errors)
    assert any("broken local link missing.png" in error for error in errors)


def test_link_validation_ignores_fenced_indented_and_inline_code(tmp_path: Path):
    record = docs.Record(
        Path("README.md"),
        {"id": "tiler.test", "kind": "portal"},
        "```markdown\n![example](not-present.png)\n[ref]: absent.md\n```\n\n"
        "    [indented](missing.md)\n\n`[inline](missing.md)`\n",
    )

    assert docs.validate_links([record], tmp_path) == []


def test_link_validation_rejects_duplicate_definitions_file_uris_and_html(tmp_path: Path):
    (tmp_path / "present.md").write_text("", encoding="utf-8")
    record = docs.Record(
        Path("README.md"),
        {"id": "tiler.test", "kind": "portal"},
        "[x][ref]\n\n[ref]: present.md\n[ref]: other.md\n\n"
        '<!-- --> <picture><source src="missing.png"></picture> <!-- -->\n',
    )

    errors = docs.validate_links([record], tmp_path)

    assert any("duplicate reference-style" in error for error in errors)
    assert any("raw HTML" in error for error in errors)
    assert "file URI" in docs.validate_local_target(record, "file:///tmp/private", tmp_path)


def test_related_is_licensed_only_for_navigational_kinds(tmp_path: Path):
    typed = docs.Record(
        Path("docs/decisions/0099-test.md"),
        {
            "schema": docs.SCHEMA,
            "id": "ADR-0099",
            "kind": "decision",
            "title": "Test",
            "topics": ["test"],
            "catalog_group": "documentation-governance",
            "decision_status": "accepted",
            "implementation_status": "implemented",
            "applies_to": ["tiler.contract.test"],
            "evidence": ["tiler.research.test"],
            "related": ["ADR-0100"],
        },
        "# 0099: Test\n",
    )
    navigational = docs.Record(
        Path("docs/status.md"),
        {
            "schema": docs.SCHEMA,
            "id": "tiler.portal.test",
            "kind": "portal",
            "title": "Test",
            "topics": ["test"],
            "related": ["tiler.roadmap"],
        },
        "# Test\n",
    )

    assert any(
        "unknown field related for decision" in error
        for error in docs.validate_record(typed, tmp_path)
    )
    assert not any("related" in error for error in docs.validate_record(navigational, tmp_path))


def test_chronology_orders_every_decision_by_number():
    records = [
        docs.Record(
            Path("docs/decisions/0010-later.md"),
            {"id": "ADR-0010", "kind": "decision", "title": "Later", "decision_status": "accepted"},
            "",
        ),
        docs.Record(
            Path("docs/decisions/0002-earlier.md"),
            {
                "id": "ADR-0002",
                "kind": "decision",
                "title": "Earlier",
                "decision_status": "superseded",
            },
            "",
        ),
        docs.Record(
            Path("docs/decisions/README.md"), {"id": "tiler.portal.test", "kind": "portal"}, ""
        ),
    ]

    assert docs.chronology(records).splitlines() == [
        "- [0002: Earlier](0002-earlier.md) — superseded",
        "- [0010: Later](0010-later.md) — accepted",
    ]


def test_checked_in_adr_chronology_is_generated_and_complete():
    relative, marker = docs.CHRONOLOGY
    text = (REPOSITORY / relative).read_text(encoding="utf-8")
    records, errors = docs.load(REPOSITORY)

    assert errors == []
    assert text.count(f"<!-- BEGIN GENERATED {marker} -->") == 1
    assert docs.chronology(records) in text


def test_experiment_field_rules_hold_outside_reproducible_status(tmp_path: Path):
    metadata = {
        "schema": docs.SCHEMA,
        "id": "tiler.spike.test",
        "kind": "experiment",
        "title": "Test",
        "topics": ["test"],
        "experiment_status": "planned",
        "implementation_status": "spike-only",
        "evidence_classes": ["bounded-measurement"],
        "supports": ["tiler.research.test"],
        "entrypoints": ["../outside/probe.py"],
        "last_verified": "2026-13-40",
    }
    record = docs.Record(Path("README.md"), metadata, "# Test\n")

    errors = docs.validate_record(record, tmp_path)

    assert any("last_verified must be YYYY-MM-DD" in error for error in errors)
    assert any("invalid repository-root entrypoint" in error for error in errors)


def test_reproducible_experiment_requires_canonical_date_and_entrypoint(tmp_path: Path):
    entrypoint = tmp_path / "probe.py"
    entrypoint.write_text("", encoding="utf-8")
    metadata = {
        "schema": docs.SCHEMA,
        "id": "tiler.spike.test",
        "kind": "experiment",
        "title": "Test",
        "topics": ["test"],
        "experiment_status": "reproducible",
        "implementation_status": "spike-only",
        "evidence_classes": ["bounded-measurement"],
        "supports": ["tiler.research.test"],
        "entrypoints": ["probe.py"],
        "last_verified": "20260720",
    }
    record = docs.Record(Path("README.md"), metadata, "# Test\n")

    errors = docs.validate_record(record, tmp_path)

    assert any("last_verified must be YYYY-MM-DD" in error for error in errors)


def test_frontmatter_identifiers_are_lexically_canonical(tmp_path: Path):
    metadata = {
        "schema": docs.SCHEMA,
        "id": "tiler.portal.test",
        "kind": "portal",
        "title": "Test",
        "topics": ["Not Canonical"],
        "related": [" tiler.other"],
    }
    record = docs.Record(Path("README.md"), metadata, "# Test\n")

    errors = docs.validate_record(record, tmp_path)

    assert any("invalid topic slug" in error for error in errors)
    assert any("nonempty, trimmed strings" in error for error in errors)
    assert any("invalid related stable id" in error for error in errors)


def test_malformed_question_heading_is_rejected(tmp_path: Path):
    path = tmp_path / "docs" / "open-questions.md"
    path.parent.mkdir()
    path.write_text(
        "### Q-SEM-001 — Valid\n\n- Owner: compiler\n- Close when: measured\n\n"
        "### Q-bad — Not stable\n",
        encoding="utf-8",
    )

    errors = docs.validate_questions(tmp_path)

    assert any("malformed question heading" in error for error in errors)


def _decision(number: int, status: str, supersedes: list[str] | None = None) -> "docs.Record":
    meta: dict[str, object] = {
        "id": f"ADR-{number:04d}",
        "kind": "decision",
        "decision_status": status,
    }
    if supersedes is not None:
        meta["supersedes"] = supersedes
    return docs.Record(Path(f"docs/decisions/{number:04d}-test.md"), meta, "")


def test_superseded_decision_and_replacement_reference_each_other():
    orphaned = docs.validate_graph([_decision(1, "superseded")], Path("."))
    assert any("superseded decision must be the target" in error for error in orphaned)

    unmarked = docs.validate_graph(
        [_decision(2, "accepted"), _decision(3, "accepted", ["ADR-0002"])], Path(".")
    )
    assert any("supersedes target must be superseded" in error for error in unmarked)

    consistent = docs.validate_graph(
        [_decision(4, "superseded"), _decision(5, "accepted", ["ADR-0004"])], Path(".")
    )
    assert not any("supersede" in error for error in consistent)


def test_direct_entrypoints_must_be_executable(tmp_path: Path):
    deps = tmp_path / "deps.sh"
    deps.write_text("#!/bin/sh\n", encoding="utf-8")
    internal = tmp_path / "spikes/shapes/shape-evidence/generate-workloads.sh"
    internal.parent.mkdir(parents=True)
    internal.write_text("#!/bin/sh\n", encoding="utf-8")
    script = tmp_path / "spikes/test/run.sh"
    script.parent.mkdir(parents=True)
    script.write_text("#!/bin/sh\n", encoding="utf-8")
    record = docs.Record(
        Path("spikes/test/README.md"),
        {"id": "tiler.test", "kind": "experiment", "entrypoints": ["spikes/test/run.sh"]},
        "```sh\nspikes/test/run.sh --measure\n```\n",
    )
    (tmp_path / "deps.sh").chmod(0o644)
    script.chmod(0o644)

    errors = docs.validate_executable_modes([record], tmp_path)

    assert errors == [
        "deps.sh: directly invoked entrypoint must be executable",
        "spikes/shapes/shape-evidence/generate-workloads.sh: directly invoked entrypoint "
        "must be executable",
        "spikes/test/run.sh: directly invoked entrypoint must be executable",
    ]
