---
schema: "tiler-doc/v1"
id: "tiler.contract.document-metadata"
kind: "contract"
title: "Documentation metadata and traceability"
topics: ["documentation", "governance"]
contract_status: "accepted"
implementation_status: "implemented"
evidence: ["tiler.research.documentation.information-architecture-audit", "tiler.research.documentation.blank-agent-acceptance-audit"]
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

`implementation_status` names the highest implementation maturity the record's
own decided behaviour has reached. It is a retained high-water mark, not a live
mirror of the working tree: superseding a decision updates `decision_status`
alone and never lowers `implementation_status`. On a `superseded` decision the
field is therefore read historically — the maturity the work reached while the
decision was in force — while the superseding decision carries the present
maturity of the contract that replaced it. A superseded decision keeps
`implemented` when its work was built and later replaced; it reads
`not-started` only when it was superseded before any of its work was built.

| Evidence class | Meaning |
| --- | --- |
| `primary-source-synthesis` | A conclusion traced to named specifications, papers, or inspected source revisions. |
| `executable-model` | Checked code exercises a proposed contract; it is not the production implementation. |
| `bounded-measurement` | An observation holds only for the recorded inputs, environment, and procedure. |
| `exhaustive-finite` | Every member of an explicitly named finite universe was checked. |
| `sound-proof` | A stated property follows within the documented formal model and assumptions. |
| `normative-guarantee` | A governing specification promises the property within its stated scope. |
| `unknown` | Available evidence does not establish the claim. This class cannot be combined with another. |

These classes are categories, not a total strength ordering. Reports and
experiment guides must name the bounded universe, assumptions, environment, or
normative scope that makes the selected class honest.

In a `mixed` contract, only accepted-ADR-derived invariants and sections
explicitly labeled accepted are normative. Unmarked field-level schemas and API
detail default to proposed. Authors should split a contract when that default
would make ordinary reading ambiguous.

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
invalid in stored v1 frontmatter.
`related` is symmetric, stored only once on the lexicographically smaller source ID, and licensed only for the navigational kinds marked in the table below.
A contract, decision, research report, or experiment already owns a directed predicate for every association it can make, so recording one as `related` would discard the direction that names which document supersedes, refines, or depends on the other.
A generic `links` or `deps` field is
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
meaning applies.
`related` is not among them; the optional column above is its exhaustive licence.
Present arrays are nonempty, contain unique homogeneous scalar values, and use no empty placeholder.
A reproducible experiment requires nonempty `entrypoints` and `evidence_classes` plus a `last_verified` date.
Those field rules bind on every experiment record carrying the field rather than on a reproducible one alone: `last_verified` is an ISO `YYYY-MM-DD` date no later than today, and entrypoints are normalized repository-root POSIX paths to existing regular files; absolute paths, backslashes, `.`/`..`, directories, and repo escapes are invalid.

An accepted decision has at least one `applies_to` contract and one `evidence`
research record. An accepted contract has an inbound accepted decision. Every
`superseded` decision is the target of at least one decision `supersedes` edge,
and every decision named as a `supersedes` target is itself `superseded`, so the
successor that carries the present state is always reachable and the retained
historical `implementation_status` stays legible rather than contradicting the
current tree. Adopted or partially adopted research has an `informs` or
`adopted_by` destination. `unknown` is exclusive when used as an evidence class.

Live ticket status and calculated backlinks never appear in document
frontmatter. Ticketsplease owns workflow state. Generated catalog sections are
derived from metadata and checked in for ordinary GitHub reading.

## Validation and catalog updates

The documentation validator uses the locked `markdown-it-py` CommonMark parser
in the repository development environment. The canonical repository gate
invokes it together with its mutation tests and the other governed checks:

```sh
uv run --locked python scripts/check_repository.py
```

After changing cataloged metadata, regenerate the checked-in views and validate
the result:

```sh
uv run --locked python scripts/docs.py render
uv run --locked python scripts/check_repository.py
```

CI runs the same complete gate on the supported macOS arm64 and Ubuntu x64
profiles. `render --check` is available when a caller needs only the
deterministic generated-block freshness check.

## Ownership

This document owns metadata shape and relationship semantics. It does not own
the architectural content being indexed, ticketsplease's ticket schema, or the
meaning of evidence inside a research report.
