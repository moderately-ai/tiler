---
schema: "tiler-doc/v1"
id: "tiler.contract.document-metadata"
kind: "contract"
title: "Documentation metadata and traceability"
topics: ["documentation", "governance"]
contract_status: "accepted"
implementation_status: "implemented"
governed_by: []
evidence: []
ticket: "docs-navigation-metadata"
---

# Documentation metadata and traceability

This contract defines how a reader or tool distinguishes authority, evidence,
implementation maturity, and work history across the repository.

## Encoding

Governed Markdown begins with a `tiler-doc/v1` frontmatter block. It is a strict
YAML-compatible subset: every non-delimiter line is `key: <JSON value>`. Values
may be strings, booleans, integers, or arrays of those scalar values. Nested
maps, multiline values, aliases, tags, duplicate keys, and unknown fields are
invalid.

Every document has a stable `id` independent of its path. Paths are presentation;
IDs are graph identity. A document move changes links but not relationships.

## Kinds and status facets

Allowed `kind` values are `portal`, `contract`, `decision`, `research`,
`experiment`, `roadmap`, `questions`, and `prior-art`.

Status is kind-specific:

| Kind | Required status |
| --- | --- |
| Contract | `contract_status`: `proposed`, `accepted`, or `mixed` |
| Decision | `decision_status`: `proposed`, `accepted`, or `superseded` |
| Research | `research_status`: `open`, `complete`, or `blocked`; plus `disposition` |
| Experiment | `experiment_status`: `planned`, `reproducible`, `partial`, or `blocked` |
| Roadmap | `roadmap_status`: `proposed` or `accepted` |
| Questions | `questions_status`: `active` or `archived` |

`disposition` is one of `pending`, `adopted`, `partially-adopted`,
`informational`, `rejected`, or `superseded`. `implementation_status` is one of
`not-started`, `spike-only`, `partial`, or `implemented`. Evidence classes are
`primary-source-synthesis`, `executable-model`, `bounded-measurement`,
`exhaustive-finite`, `sound-proof`, `normative-guarantee`, and `unknown`.

## Typed relationships

Use only relationships whose direction has a defined meaning:

- `governed_by`: contract to ADR;
- `applies_to`: ADR to normative contract;
- `evidence`: contract or ADR to research;
- `informs`: research to contract;
- `adopted_by`: research to ADR;
- `reproduced_by`: research to experiment;
- `supports`: experiment to research;
- `depends_on`, `refines`, `supersedes`, and `related`: document-to-document;
- `ticket`: document to ticketsplease ticket ID.

Backlinks are derived and must not be stored as redundant reverse edges. A
generic `links` or `deps` field is invalid. Human Markdown still links the
important route in prose; frontmatter does not replace explanation.

## Required common fields

Every governed document has `schema`, `id`, `kind`, `title`, and `topics`.
Frontmatter titles must match the first level-one Markdown heading after an ADR
number prefix is removed. IDs are unique and use dotted lowercase namespaces;
ADR IDs are the fixed uppercase form `ADR-NNNN`.

Live ticket status and calculated backlinks never appear in document
frontmatter. Ticketsplease owns workflow state. Generated catalog sections are
derived from metadata and checked in for ordinary GitHub reading.

## Ownership

This document owns metadata shape and relationship semantics. It does not own
the architectural content being indexed, ticketsplease's ticket schema, or the
meaning of evidence inside a research report.
