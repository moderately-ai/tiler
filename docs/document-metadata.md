---
schema: "tiler-doc/v1"
id: "tiler.contract.document-metadata"
kind: "contract"
title: "Documentation metadata and traceability"
topics: ["documentation", "governance"]
contract_status: "accepted"
implementation_status: "implemented"
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

- `applies_to`: ADR to normative contract;
- `evidence`: contract or ADR to research;
- `informs`: research to contract;
- `adopted_by`: research to ADR;
- `supports`: experiment to research;
- `depends_on`, `refines`, `supersedes`, and `related`: document-to-document;
- `ticket`: document to ticketsplease ticket ID.

`informs` may also connect prior art to a contract. `evidence`, `informs`, and
`adopted_by` are independent predicates: evidence may support a decision without
that decision adopting the report's proposal.

Contract `governed_by` is derived from decision `applies_to`; research
`reproduced_by` is derived from experiment `supports`. These backlink fields are
invalid in stored v1 frontmatter. `related` is symmetric but stored only once on
the lexicographically smaller source ID. A generic `links` or `deps` field is
invalid. Human Markdown still links the important route in prose; frontmatter
does not replace explanation.

## Required common fields

Every governed document has `schema`, `id`, `kind`, `title`, and `topics`.
Frontmatter titles must match the first level-one Markdown heading after an ADR
number prefix is removed. IDs are unique and use dotted lowercase namespaces;
ADR IDs are the fixed uppercase form `ADR-NNNN`.

Kind-specific required fields are:

| Kind | Required beyond common | Optional typed fields |
| --- | --- | --- |
| Portal | none | `related` |
| Contract | `contract_status`, `implementation_status` | `evidence`, `ticket` |
| Decision | `decision_status`, `implementation_status`, `applies_to`, `evidence` | `ticket` |
| Research | `research_status`, `disposition`, `implementation_status`, `evidence_classes`, `informs` | `adopted_by`, `ticket` |
| Experiment | `experiment_status`, `implementation_status`, `evidence_classes`, `supports` | `entrypoints`, `last_verified`, `ticket` |
| Roadmap | `roadmap_status` | `related` |
| Questions | `questions_status` | `related` |
| Prior art | none | `informs`, `related` |

Decision and research records also require `catalog_group`. Its controlled
values are `foundation-semantics-extensions`, `numerical-operations`,
`dtypes-quantization`, `physical-planning-lowering`,
`artifacts-build-toolchains`, `runtime-integration-placement`, and
`documentation-governance`. Topics remain free faceted discovery terms;
`catalog_group` supplies one stable coarse location in generated catalogs.

All kinds may use `depends_on`, `refines`, and `supersedes` where their typed
meaning applies. Present arrays are nonempty, contain unique homogeneous scalar
values, and use no empty placeholder. A reproducible experiment requires
nonempty `entrypoints` and `evidence_classes` plus an ISO `YYYY-MM-DD`
`last_verified` date. Entrypoints are normalized repository-root POSIX paths to
existing regular files; absolute paths, backslashes, `.`/`..`, directories, and
repo escapes are invalid.

An accepted decision has at least one `applies_to` contract and one `evidence`
research record. An accepted contract has an inbound accepted decision. Adopted
or partially adopted research has an `informs` or `adopted_by` destination.
`unknown` is exclusive when used as an evidence class.

Live ticket status and calculated backlinks never appear in document
frontmatter. Ticketsplease owns workflow state. Generated catalog sections are
derived from metadata and checked in for ordinary GitHub reading.

## Validation and catalog updates

The validator uses only the Python standard library:

```sh
python3 scripts/docs.py validate
python3 -m unittest discover -s scripts/tests -v
```

After changing cataloged metadata, regenerate the checked-in views and validate
the result:

```sh
python3 scripts/docs.py render
python3 scripts/docs.py validate
```

CI runs the tests and validator. `render --check` is available when a caller
needs only the deterministic generated-block freshness check.

## Ownership

This document owns metadata shape and relationship semantics. It does not own
the architectural content being indexed, ticketsplease's ticket schema, or the
meaning of evidence inside a research report.
