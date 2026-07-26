import importlib.util
import sys
import tomllib
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


def citing(tmp_path: Path, body: str, name: str = "0077-test", status: str = "proposed"):
    """A contract citing one decision of the given status, both on disk under docs/."""
    (tmp_path / "docs/decisions").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/decisions" / f"{name}.md").write_text("", encoding="utf-8")
    decision = docs.Record(
        Path(f"docs/decisions/{name}.md"),
        {"id": "ADR-0077", "kind": "decision", "decision_status": status},
        "",
    )
    contract = docs.Record(Path("docs/contract.md"), {"id": "tiler.c", "kind": "contract"}, body)
    return docs.validate_proposal_disclosure([contract, decision], tmp_path)


def test_contract_citing_a_proposed_decision_without_disclosure_fails(tmp_path: Path):
    errors = citing(tmp_path, "The driver is admitted by [ADR 0077](decisions/0077-test.md).\n")

    assert len(errors) == 1
    assert errors[0].startswith("docs/contract.md:1: contract cites docs/decisions/0077-test.md")
    assert "without disclosing that the decision is only proposed" in errors[0]


def test_disclosed_citation_and_accepted_decision_both_pass(tmp_path: Path):
    """The corpus fixture: naming the status, and what stands until acceptance, asserts nothing."""
    disclosed = (
        "No accepted ADR yet records that admission, and [ADR 0077](decisions/0077-test.md) "
        "is the proposed record of it. That supersession takes effect when Tom accepts it.\n"
    )
    assert citing(tmp_path, disclosed) == []
    settled = "Admitted by [ADR 0077](decisions/0077-test.md).\n"
    assert citing(tmp_path, settled, status="accepted") == []


def test_citation_resolves_through_a_fragment_suffix(tmp_path: Path):
    errors = citing(tmp_path, "See [ADR 0077](decisions/0077-test.md#consequences).\n")

    assert len(errors) == 1


def test_a_filename_containing_proposed_is_not_a_disclosure(tmp_path: Path):
    """Only prose discloses; a destination is blanked before the word is sought."""
    errors = citing(
        tmp_path,
        "The driver is admitted by [ADR 0077](decisions/0077-proposed-driver.md).\n",
        name="0077-proposed-driver",
    )

    assert len(errors) == 1


def test_disclosure_reaches_across_a_list_but_not_a_preceding_paragraph(tmp_path: Path):
    """The outermost block containing the link must disclose, so a sibling item may carry it."""
    assert (
        citing(
            tmp_path,
            "Undecided records:\n\n"
            "- These entries are proposed and not yet accepted.\n"
            "- [ADR 0077](decisions/0077-test.md) admits the driver.\n",
        )
        == []
    )

    errors = citing(
        tmp_path,
        "These entries are proposed and not yet accepted.\n\n"
        "- [ADR 0077](decisions/0077-test.md) admits the driver.\n",
    )

    assert len(errors) == 1


def test_only_contracts_are_checked_for_disclosure(tmp_path: Path):
    """A research record weighing an undecided proposal is doing its job, not asserting it."""
    (tmp_path / "docs/decisions").mkdir(parents=True)
    (tmp_path / "docs/decisions/0077-test.md").write_text("", encoding="utf-8")
    decision = docs.Record(
        Path("docs/decisions/0077-test.md"),
        {"id": "ADR-0077", "kind": "decision", "decision_status": "proposed"},
        "",
    )
    research = docs.Record(
        Path("docs/research/note.md"),
        {"id": "tiler.r", "kind": "research"},
        "Weighed against [ADR 0077](../decisions/0077-test.md).\n",
    )

    assert docs.validate_proposal_disclosure([research, decision], tmp_path) == []


def ticketing(tmp_path: Path, status: str, dependencies: str, decision_ticket: str = "draft-adr"):
    (tmp_path / "tickets").mkdir(exist_ok=True)
    (tmp_path / "tickets/dependent.md").write_text(
        f"---\nid: dependent\nstatus: {status}\ndependencies: {dependencies}\n---\nBody\n",
        encoding="utf-8",
    )
    decision = docs.Record(
        Path("docs/decisions/0078-test.md"),
        {
            "id": "ADR-0078",
            "kind": "decision",
            "decision_status": "proposed",
            "ticket": decision_ticket,
        },
        "",
    )
    return [e for e in docs.validate_tickets([decision], tmp_path) if "tickets/" in e]


def test_dispatchable_ticket_may_not_depend_on_a_proposed_adrs_drafting_ticket(tmp_path: Path):
    """The measured regression: `tkt ready` reads dependency status, and drafting is `done`."""
    errors = ticketing(tmp_path, "todo", "[draft-adr]")

    assert len(errors) == 1
    assert "todo ticket depends on draft-adr" in errors[0]
    assert "still-proposed docs/decisions/0078-test.md" in errors[0]
    assert "accept-adr-NNNN-* node instead" in errors[0]


def test_depending_on_the_acceptance_node_is_the_repaired_form(tmp_path: Path):
    assert ticketing(tmp_path, "todo", "[accept-adr-0078-public-extension-seams]") == []


def test_every_dispatchable_and_open_status_is_checked(tmp_path: Path):
    """A state added to the workflow must be classified here, not silently exempted."""
    workflow = tomllib.loads((REPOSITORY / "ticketsplease.toml").read_text(encoding="utf-8"))
    states = workflow["workflow"]["states"]
    reachable = {
        name for name, body in states.items() if body["category"] in {"dispatchable", "open"}
    }

    assert reachable == docs.DISPATCHABLE_OR_OPEN
    for status in sorted(docs.DISPATCHABLE_OR_OPEN):
        assert len(ticketing(tmp_path, status, "[draft-adr]")) == 1, status


def test_a_parked_acceptance_node_may_depend_on_its_drafting_ticket(tmp_path: Path):
    """The node exists to wait on the decision, and it is parked, so it never reaches `ready`."""
    assert ticketing(tmp_path, "awaiting-decision", "[draft-adr]") == []


def test_the_drafting_ticket_of_an_accepted_decision_may_be_depended_on(tmp_path: Path):
    (tmp_path / "tickets").mkdir()
    (tmp_path / "tickets/dependent.md").write_text(
        "---\nid: dependent\nstatus: todo\ndependencies: [draft-adr]\n---\nBody\n",
        encoding="utf-8",
    )
    accepted = docs.Record(
        Path("docs/decisions/0078-test.md"),
        {
            "id": "ADR-0078",
            "kind": "decision",
            "decision_status": "accepted",
            "ticket": "draft-adr",
        },
        "",
    )

    assert docs.validate_tickets([accepted], tmp_path) == []


def test_a_proposed_decision_without_a_drafting_ticket_is_an_error(tmp_path: Path):
    """An absent `ticket` would otherwise exempt the record from Check B silently."""
    (tmp_path / "tickets").mkdir()
    anonymous = docs.Record(
        Path("docs/decisions/0078-test.md"),
        {"id": "ADR-0078", "kind": "decision", "decision_status": "proposed"},
        "",
    )

    errors = docs.validate_tickets([anonymous], tmp_path)

    assert len(errors) == 1
    assert "must name the ticket that drafted it" in errors[0]


def test_frontmatter_offset_makes_reported_lines_file_relative(tmp_path: Path):
    path = tmp_path / "README.md"
    path.write_text(
        '---\nschema: "tiler-doc/v1"\nid: "tiler.portal.test"\nkind: "portal"\n'
        'title: "Test"\ntopics: ["test"]\n---\n\n# Test\n',
        encoding="utf-8",
    )

    record, errors = docs.parse(path, tmp_path)

    assert errors == []
    heading = record.body.splitlines().index("# Test")
    reported = heading + record.offset + 1
    assert path.read_text(encoding="utf-8").splitlines()[reported - 1] == "# Test"
    assert reported == 9
