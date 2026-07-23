#!/usr/bin/env python3
"""Validate Tiler's documentation graph and render checked-in catalogs."""

from __future__ import annotations

import argparse
import json
import posixpath
import re
import stat
import sys
import urllib.parse
from collections import defaultdict
from dataclasses import dataclass
from datetime import date
from pathlib import Path, PurePosixPath

from markdown_it import MarkdownIt

SCHEMA = "tiler-doc/v1"
KINDS = {
    "portal",
    "contract",
    "decision",
    "research",
    "experiment",
    "roadmap",
    "questions",
    "prior-art",
}
GROUPS = {
    "foundation-semantics-extensions": "Foundation, semantics, and extensions",
    "numerical-operations": "Numerical operations",
    "dtypes-quantization": "Dtypes and quantization",
    "physical-planning-lowering": "Physical planning and lowering",
    "artifacts-build-toolchains": "Artifacts, build, and toolchains",
    "runtime-integration-placement": "Runtime, integration, and placement",
    "documentation-governance": "Documentation governance",
}
ENUMS = {
    "contract_status": {"proposed", "accepted", "mixed"},
    "decision_status": {"proposed", "accepted", "superseded"},
    "research_status": {"open", "complete", "blocked"},
    "experiment_status": {"planned", "reproducible", "partial", "blocked"},
    "roadmap_status": {"proposed", "accepted"},
    "questions_status": {"active", "archived"},
    "disposition": {
        "pending",
        "adopted",
        "partially-adopted",
        "informational",
        "rejected",
        "superseded",
    },
    "implementation_status": {"not-started", "spike-only", "partial", "implemented"},
}
EVIDENCE = {
    "primary-source-synthesis",
    "executable-model",
    "bounded-measurement",
    "exhaustive-finite",
    "sound-proof",
    "normative-guarantee",
    "unknown",
}
COMMON = {
    "schema",
    "id",
    "kind",
    "title",
    "topics",
    "depends_on",
    "refines",
    "supersedes",
}
FIELDS = {
    "portal": {"related"},
    "contract": {"contract_status", "implementation_status", "evidence", "ticket"},
    "decision": {
        "decision_status",
        "implementation_status",
        "applies_to",
        "evidence",
        "catalog_group",
        "ticket",
    },
    "research": {
        "research_status",
        "disposition",
        "implementation_status",
        "evidence_classes",
        "informs",
        "adopted_by",
        "catalog_group",
        "ticket",
    },
    "experiment": {
        "experiment_status",
        "implementation_status",
        "evidence_classes",
        "supports",
        "entrypoints",
        "last_verified",
        "ticket",
    },
    "roadmap": {"roadmap_status", "related"},
    "questions": {"questions_status", "related"},
    "prior-art": {"informs", "related"},
}
REQUIRED = {
    "portal": set(),
    "contract": {"contract_status", "implementation_status"},
    "decision": {
        "decision_status",
        "implementation_status",
        "applies_to",
        "evidence",
        "catalog_group",
    },
    "research": {
        "research_status",
        "disposition",
        "implementation_status",
        "evidence_classes",
        "informs",
        "catalog_group",
    },
    "experiment": {"experiment_status", "implementation_status", "evidence_classes", "supports"},
    "roadmap": {"roadmap_status"},
    "questions": {"questions_status"},
    "prior-art": set(),
}
ARRAYS = {
    "topics",
    "depends_on",
    "refines",
    "supersedes",
    "related",
    "evidence",
    "applies_to",
    "evidence_classes",
    "informs",
    "adopted_by",
    "supports",
    "entrypoints",
}
RELATIONS = {
    "depends_on",
    "refines",
    "supersedes",
    "related",
    "evidence",
    "applies_to",
    "informs",
    "adopted_by",
    "supports",
}
MARKERS = {
    "decision": (Path("docs/decisions/README.md"), "ADR TOPICS"),
    "research": (Path("docs/research/README.md"), "RESEARCH CATALOG"),
    "experiment": (Path("spikes/README.md"), "EXPERIMENT CATALOG"),
}
CHRONOLOGY = (Path("docs/decisions/README.md"), "ADR CHRONOLOGY")
STABLE_ID = re.compile(r"(?:ADR-\d{4}|[a-z0-9]+(?:[.-][a-z0-9]+)*)")
SLUG = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")
ISO_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")


@dataclass(frozen=True)
class Record:
    path: Path
    meta: dict[str, object]
    body: str

    @property
    def id(self) -> str:
        return str(self.meta["id"])


def governed(root: Path) -> list[Path]:
    paths = [root / "README.md"]
    paths += sorted((root / "docs").rglob("*.md"))
    paths += sorted((root / "spikes").rglob("README.md"))
    return [p for p in paths if p.is_file()]


def parse(path: Path, root: Path) -> tuple[Record | None, list[str]]:
    rel = path.relative_to(root)
    text = path.read_text(encoding="utf-8")
    errors: list[str] = []
    if not text.startswith("---\n"):
        return None, [f"{rel}:1: governed Markdown must begin with ---"]
    end = text.find("\n---\n", 4)
    if end < 0:
        return None, [f"{rel}:1: unterminated frontmatter"]
    meta: dict[str, object] = {}
    for line_no, line in enumerate(text[4:end].splitlines(), 2):
        match = re.fullmatch(r"([a-z][a-z0-9_]*): (.+)", line)
        if not match:
            errors.append(f"{rel}:{line_no}: expected key: <JSON value>")
            continue
        key, raw = match.groups()
        if key in meta:
            errors.append(f"{rel}:{line_no}: duplicate field {key}")
            continue
        try:
            value = json.loads(raw)
        except json.JSONDecodeError as exc:
            errors.append(f"{rel}:{line_no}: invalid JSON value: {exc.msg}")
            continue
        scalar = isinstance(value, (str, bool, int)) and not isinstance(value, float)
        array = isinstance(value, list) and all(
            isinstance(v, (str, bool, int)) and not isinstance(v, float) for v in value
        )
        if not scalar and not array:
            errors.append(f"{rel}:{line_no}: value must be a scalar or flat scalar array")
            continue
        meta[key] = value
    return Record(rel, meta, text[end + 5 :]), errors


def load(root: Path) -> tuple[list[Record], list[str]]:
    records, errors = [], []
    for path in governed(root):
        record, found = parse(path, root)
        errors.extend(found)
        if record:
            records.append(record)
    return records, errors


def validate_record(record: Record, root: Path) -> list[str]:
    m, p, errors = record.meta, record.path, []
    kind = m.get("kind")
    if kind not in KINDS:
        return [f"{p}: unknown kind {kind!r}"]
    allowed = COMMON | FIELDS[str(kind)]
    for key in sorted(set(m) - allowed):
        errors.append(f"{p}: unknown field {key} for {kind}")
    for key in sorted({"schema", "id", "kind", "title", "topics"} | REQUIRED[str(kind)]):
        if key not in m:
            errors.append(f"{p}: missing required field {key}")
    for key, value in m.items():
        if key not in ARRAYS and not isinstance(value, str):
            errors.append(f"{p}: {key} must be a string")
        if isinstance(value, str) and (not value or value != value.strip()):
            errors.append(f"{p}: {key} must be a nonempty, trimmed string")
    if m.get("schema") != SCHEMA:
        errors.append(f"{p}: schema must be {SCHEMA!r}")
    identifier = m.get("id")
    id_ok = isinstance(identifier, str) and (
        re.fullmatch(r"ADR-\d{4}", identifier)
        if kind == "decision"
        else re.fullmatch(r"[a-z0-9]+(?:[.-][a-z0-9]+)*", identifier)
    )
    if not id_ok:
        errors.append(f"{p}: invalid stable id {identifier!r}")
    for key in ARRAYS:
        if key not in m:
            continue
        value = m[key]
        if not isinstance(value, list) or not value:
            errors.append(f"{p}: {key} must be a nonempty array")
        elif any(not isinstance(v, str) for v in value) or len(value) != len(set(value)):
            errors.append(f"{p}: {key} must contain unique strings")
        elif any(not v or v != v.strip() for v in value):
            errors.append(f"{p}: {key} must contain nonempty, trimmed strings")
    for topic in m.get("topics", []):
        if isinstance(topic, str) and not SLUG.fullmatch(topic):
            errors.append(f"{p}: invalid topic slug {topic!r}")
    ticket = m.get("ticket")
    if isinstance(ticket, str) and not SLUG.fullmatch(ticket):
        errors.append(f"{p}: invalid ticket slug {ticket!r}")
    for relation in RELATIONS:
        for target in m.get(relation, []):
            if isinstance(target, str) and not STABLE_ID.fullmatch(target):
                errors.append(f"{p}: invalid {relation} stable id {target!r}")
    for key, values in ENUMS.items():
        if key in m and m[key] not in values:
            errors.append(f"{p}: invalid {key} {m[key]!r}")
    if kind in {"decision", "research"} and m.get("catalog_group") not in GROUPS:
        errors.append(f"{p}: invalid catalog_group {m.get('catalog_group')!r}")
    if p.match("docs/decisions/*.md") and p.name != "README.md" and kind != "decision":
        errors.append(f"{p}: ADR path requires decision kind")
    if p.match("docs/research/**/*.md") and p.name != "README.md" and kind != "research":
        errors.append(f"{p}: research path requires research kind")
    if p.match("docs/prior-art/*.md") and p.name != "README.md" and kind != "prior-art":
        errors.append(f"{p}: prior-art path requires prior-art kind")
    if p.match("spikes/**/README.md") and kind not in {"experiment", "portal"}:
        errors.append(f"{p}: spike README requires experiment or portal kind")
    classes = m.get("evidence_classes", [])
    if isinstance(classes, list):
        for value in classes:
            if value not in EVIDENCE:
                errors.append(f"{p}: invalid evidence class {value!r}")
        if "unknown" in classes and len(classes) != 1:
            errors.append(f"{p}: unknown evidence is exclusive")
    heading = next(
        (line[2:].strip() for line in record.body.splitlines() if line.startswith("# ")), None
    )
    expected = re.sub(r"^\d{4}:\s*", "", heading or "")
    if expected != m.get("title"):
        errors.append(f"{p}: title {m.get('title')!r} does not match H1 {heading!r}")
    if kind == "experiment":
        if m.get("experiment_status") == "reproducible":
            for key in ("evidence_classes", "entrypoints", "last_verified"):
                if key not in m:
                    errors.append(f"{p}: reproducible experiment requires {key}")
        # Only presence is reproducible-only. The field rules bind wherever a
        # value exists so a planned, partial, or blocked record cannot park a
        # malformed date or an unresolvable entrypoint until it is promoted.
        if "last_verified" in m:
            verified = m["last_verified"]
            try:
                if not isinstance(verified, str) or not ISO_DATE.fullmatch(verified):
                    raise ValueError
                if date.fromisoformat(verified) > date.today():
                    errors.append(f"{p}: last_verified is in the future")
            except ValueError:
                errors.append(f"{p}: last_verified must be YYYY-MM-DD")
        for entry in m.get("entrypoints", []):
            posix = PurePosixPath(str(entry))
            if (
                posix.is_absolute()
                or ".." in posix.parts
                or "." in posix.parts
                or "\\" in str(entry)
                or posix.as_posix() != entry
                or not (root / posix).is_file()
            ):
                errors.append(f"{p}: invalid repository-root entrypoint {entry!r}")
    return errors


def ticket_ids(root: Path) -> set[str]:
    return {p.stem for p in (root / "tickets").glob("*.md")}


def contains_cycle(graph: dict[str, list[str]]) -> bool:
    visiting: set[str] = set()
    visited: set[str] = set()

    def walk(node: str) -> bool:
        if node in visiting:
            return True
        if node in visited:
            return False
        visiting.add(node)
        cyclic = any(walk(nxt) for nxt in graph[node])
        visiting.remove(node)
        visited.add(node)
        return cyclic

    return any(walk(node) for node in list(graph))


def validate_graph(records: list[Record], root: Path) -> list[str]:
    errors: list[str] = []
    by_id: dict[str, Record] = {}
    for record in records:
        if record.id in by_id:
            errors.append(f"{record.path}: duplicate id {record.id} (also {by_id[record.id].path})")
        by_id[record.id] = record
    tickets = ticket_ids(root)
    type_rules = {
        "applies_to": {"contract"},
        "evidence": {"research"},
        "informs": {"contract"},
        "adopted_by": {"decision"},
        "supports": {"research"},
    }
    edges: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for record in records:
        m = record.meta
        if "ticket" in m and m["ticket"] not in tickets:
            errors.append(f"{record.path}: missing ticket {m['ticket']!r}")
        for relation in RELATIONS:
            for target_id in m.get(relation, []):
                target = by_id.get(str(target_id))
                if not target:
                    errors.append(f"{record.path}: unresolved {relation} target {target_id!r}")
                    continue
                if relation in type_rules and target.meta["kind"] not in type_rules[relation]:
                    errors.append(
                        f"{record.path}: {relation} cannot target {target.meta['kind']} {target_id}"
                    )
                if relation == "related" and record.id >= target.id:
                    errors.append(
                        f"{record.path}: related edge must be stored on "
                        "lexicographically smaller id"
                    )
                if relation in {"depends_on", "refines", "supersedes"}:
                    edges[relation].append((record.id, target.id))
    accepted_adrs = [
        r
        for r in records
        if r.meta.get("kind") == "decision" and r.meta.get("decision_status") == "accepted"
    ]
    inbound = {target for r in accepted_adrs for target in r.meta.get("applies_to", [])}
    for record in records:
        m = record.meta
        if (
            m.get("kind") == "contract"
            and m.get("contract_status") == "accepted"
            and record.id not in inbound
        ):
            errors.append(f"{record.path}: accepted contract has no inbound accepted ADR")
        if (
            m.get("kind") == "decision"
            and m.get("decision_status") == "accepted"
            and (not m.get("applies_to") or not m.get("evidence"))
        ):
            errors.append(f"{record.path}: accepted decision requires applies_to and evidence")
        if (
            m.get("kind") == "research"
            and m.get("disposition") in {"adopted", "partially-adopted"}
            and not (m.get("informs") or m.get("adopted_by"))
        ):
            errors.append(f"{record.path}: adopted research requires informs or adopted_by")
    for name, relation_edges in edges.items():
        graph: dict[str, list[str]] = defaultdict(list)
        for source, target in relation_edges:
            graph[source].append(target)
        if contains_cycle(graph):
            errors.append(f"metadata graph: {name} contains a cycle")
    return errors


MARKDOWN = MarkdownIt("commonmark")
COMMENT_ONLY_HTML = re.compile(r"(?:\s*<!--(?:(?!-->)[\s\S])*-->)*\s*")
DIRECT_INTERNAL_ENTRYPOINTS = {Path("spikes/shapes/shape-evidence/generate-workloads.sh")}


def validate_local_target(record: Record, raw: str, root: Path) -> str | None:
    parsed = urllib.parse.urlsplit(raw)
    if parsed.scheme.lower() == "file":
        return f"{record.path}: file URI is not allowed: {raw}"
    if parsed.scheme or parsed.netloc:
        return None
    target = urllib.parse.unquote(parsed.path)
    if not target:
        return None
    try:
        path = (root / record.path.parent / target).resolve()
    except (OSError, ValueError) as error:
        return f"{record.path}: invalid local link {raw}: {error}"
    try:
        path.relative_to(root.resolve())
    except ValueError:
        return f"{record.path}: link escapes repository: {raw}"
    if not path.exists():
        return f"{record.path}: broken local link {raw}"
    return None


def validate_links(records: list[Record], root: Path) -> list[str]:
    errors = []
    for record in records:
        environment: dict[str, object] = {}
        tokens = MARKDOWN.parse(record.body, environment)
        references = environment.get("references", {})
        if not isinstance(references, dict):
            references = {}
        for definition in references.values():
            if not isinstance(definition, dict) or not isinstance(definition.get("href"), str):
                errors.append(f"{record.path}: malformed reference-style link definition")
                continue
            error = validate_local_target(record, definition["href"], root)
            if error:
                errors.append(error)
        duplicate_references = environment.get("duplicate_refs", [])
        if duplicate_references:
            errors.append(f"{record.path}: duplicate reference-style link definitions")
        for token in tokens:
            if token.type in {"html_block", "html_inline"} and not COMMENT_ONLY_HTML.fullmatch(
                token.content
            ):
                errors.append(f"{record.path}: raw HTML is outside governed Markdown")
            for child in token.children or []:
                if child.type == "html_inline" and not COMMENT_ONLY_HTML.fullmatch(child.content):
                    errors.append(f"{record.path}: raw HTML is outside governed Markdown")
                if child.type in {"link_open", "image"}:
                    attribute = "href" if child.type == "link_open" else "src"
                    target = child.attrGet(attribute)
                    if target is not None:
                        error = validate_local_target(record, target, root)
                        if error:
                            errors.append(error)
    return errors


def validate_executable_modes(records: list[Record], root: Path) -> list[str]:
    """Require executable mode for root and metadata-declared script entrypoints."""
    errors = []
    entrypoints = {Path("deps.sh"), *DIRECT_INTERNAL_ENTRYPOINTS}
    for record in records:
        for value in record.meta.get("entrypoints", []):
            relative = Path(str(value))
            path = root / relative
            command = re.compile(rf"^\s*{re.escape(relative.as_posix())}(?:\s|$)", re.MULTILINE)
            if path.is_file() and command.search(record.body):
                entrypoints.add(relative)
    for relative in sorted(entrypoints):
        path = root / relative
        if not path.is_file():
            errors.append(f"{relative}: directly invoked entrypoint is missing")
        elif not path.stat().st_mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH):
            errors.append(f"{relative}: directly invoked entrypoint must be executable")
    return errors


def validate_tickets(root: Path) -> list[str]:
    errors = []
    for path in sorted((root / "tickets").glob("*.md")):
        text = path.read_text(encoding="utf-8")
        end = text.find("\n---\n", 4)
        header = text[4:end] if text.startswith("---\n") and end >= 0 else ""
        status = re.search(r"^status: ([a-z-]+)$", header, re.MULTILINE)
        if status and status.group(1) == "done" and "\n## Outcome\n" not in text:
            errors.append(f"{path.relative_to(root)}: done ticket requires ## Outcome")
    return errors


def validate_questions(root: Path) -> list[str]:
    path = root / "docs/open-questions.md"
    text = path.read_text(encoding="utf-8")
    valid = re.compile(r"^### (Q-[A-Z]+-\d+(?:-[A-Z])?) — .+$", re.MULTILINE)
    matches = list(valid.finditer(text))
    errors, seen = [], set()
    for line_no, line in enumerate(text.splitlines(), 1):
        if re.match(r"^###\s+Q", line) and not valid.fullmatch(line):
            errors.append(f"docs/open-questions.md:{line_no}: malformed question heading")
    headings = list(re.finditer(r"^### .+$", text, re.MULTILINE))
    for match in matches:
        qid = match.group(1)
        if qid in seen:
            errors.append(f"docs/open-questions.md: duplicate question {qid}")
        seen.add(qid)
        next_heading = next(
            (heading.start() for heading in headings if heading.start() > match.start()), len(text)
        )
        block = text[match.end() : next_heading]
        if not re.search(r"^- Owner(?:/track(?:ing)?|/tracking)?:", block, re.MULTILINE):
            errors.append(f"docs/open-questions.md: {qid} lacks owner")
        if not re.search(r"^- (?:Close(?: when)?|Run when|Trigger):", block, re.MULTILINE):
            errors.append(f"docs/open-questions.md: {qid} lacks closure or trigger")
    if not matches:
        errors.append("docs/open-questions.md: no stable question IDs")
    return errors


def catalog(records: list[Record], kind: str) -> str:
    selected = [r for r in records if r.meta.get("kind") == kind]
    by_id = {r.id: r for r in records}
    grouped: dict[str, list[Record]] = defaultdict(list)
    for record in selected:
        group = record.meta.get("catalog_group")
        if kind == "experiment":
            support_groups = {
                by_id[s].meta.get("catalog_group")
                for s in record.meta.get("supports", [])
                if s in by_id
            }
            group = sorted(support_groups)[0] if support_groups else "documentation-governance"
        grouped[str(group)].append(record)
    lines = []
    portal_dir = MARKERS[kind][0].parent.as_posix()

    def link(target_id: str) -> str:
        target = by_id[target_id]
        href = posixpath.relpath(target.path.as_posix(), portal_dir)
        return f"[{target.meta['title']}]({href})"

    experiments_by_research: dict[str, list[str]] = defaultdict(list)
    for candidate in records:
        if candidate.meta.get("kind") == "experiment":
            for supported in candidate.meta.get("supports", []):
                experiments_by_research[str(supported)].append(candidate.id)
    for group in GROUPS:
        items = sorted(grouped.get(group, []), key=lambda r: (str(r.meta.get("title")), r.id))
        if not items:
            continue
        lines += [f"### {GROUPS[group]}", ""]
        for record in items:
            if kind == "decision":
                label = f"{record.id[4:]}: {record.meta['title']}"
                href = record.path.relative_to(Path("docs/decisions"))
                contracts = ", ".join(link(str(item)) for item in record.meta["applies_to"])
                evidence = ", ".join(link(str(item)) for item in record.meta["evidence"])
                detail = (
                    f"{record.meta['decision_status']}; contracts: {contracts}; "
                    f"evidence: {evidence}"
                )
            elif kind == "research":
                label = str(record.meta["title"])
                href = Path(posixpath.relpath(record.path.as_posix(), portal_dir))
                detail = (
                    f"{record.meta['disposition']}; {', '.join(record.meta['evidence_classes'])}"
                )
                destinations = [
                    str(item)
                    for item in record.meta.get("informs", []) + record.meta.get("adopted_by", [])
                ]
                if destinations:
                    detail += "; informs: " + ", ".join(link(item) for item in destinations)
                reproduced = sorted(experiments_by_research.get(record.id, []))
                if reproduced:
                    detail += "; experiments: " + ", ".join(link(item) for item in reproduced)
            else:
                label = str(record.meta["title"])
                href = record.path.relative_to(Path("spikes"))
                detail = (
                    f"{record.meta['experiment_status']}; "
                    f"{', '.join(record.meta['evidence_classes'])}"
                )
                detail += "; supports: " + ", ".join(
                    link(str(item)) for item in record.meta["supports"]
                )
            lines.append(f"- [{label}]({href.as_posix()}) — {detail}")
        lines.append("")
    return "\n".join(lines).rstrip()


def chronology(records: list[Record]) -> str:
    """Render every ADR in number order; the validated `ADR-NNNN` id is the sole key."""
    directory = CHRONOLOGY[0].parent.as_posix()
    decisions = sorted((r for r in records if r.meta.get("kind") == "decision"), key=lambda r: r.id)
    return "\n".join(
        f"- [{record.id[4:]}: {record.meta['title']}]"
        f"({posixpath.relpath(record.path.as_posix(), directory)}) — "
        f"{record.meta['decision_status']}"
        for record in decisions
    )


def generated(records: list[Record]) -> list[tuple[Path, str, str]]:
    blocks = [(path, marker, catalog(records, kind)) for kind, (path, marker) in MARKERS.items()]
    return [*blocks, (CHRONOLOGY[0], CHRONOLOGY[1], chronology(records))]


def render(root: Path, records: list[Record], check: bool) -> list[str]:
    errors = []
    for relative, marker, body in generated(records):
        path = root / relative
        text = path.read_text(encoding="utf-8")
        begin, end = f"<!-- BEGIN GENERATED {marker} -->", f"<!-- END GENERATED {marker} -->"
        replacement = f"{begin}\n{body}\n{end}"
        updated, count = re.subn(
            re.escape(begin) + r".*?" + re.escape(end), replacement, text, flags=re.DOTALL
        )
        if count != 1:
            errors.append(f"{relative}: expected exactly one generated {marker} block")
        elif updated != text:
            if check:
                errors.append(f"{relative}: generated catalog is stale; run scripts/docs.py render")
            else:
                path.write_text(updated, encoding="utf-8")
    return errors


def validate(root: Path, check_render: bool = True) -> list[str]:
    records, errors = load(root)
    for record in records:
        errors += validate_record(record, root)
    errors += validate_graph(records, root)
    errors += validate_links(records, root)
    errors += validate_executable_modes(records, root)
    errors += validate_tickets(root)
    errors += validate_questions(root)
    if check_render:
        errors += render(root, records, True)
    return sorted(set(errors))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("validate", "render"))
    parser.add_argument("--check", action="store_true", help="with render, fail instead of writing")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    root = args.root.resolve()
    records, parse_errors = load(root)
    if args.command == "render":
        errors = parse_errors + render(root, records, args.check)
    else:
        errors = validate(root)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"documentation {args.command} passed ({len(records)} records)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
