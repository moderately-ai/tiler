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

## Prose source form

A paragraph is one source line. Do not insert a newline into a paragraph to hold it inside a column width: the renderer wraps, so a hand-wrapped paragraph fixes a presentation choice into the source, where every later edit must either reflow lines the change did not touch or leave the paragraph ragged. The rule governs the prose inside a list item, a table cell, a block quotation, and a footnote exactly as it governs a top-level paragraph. `scripts/docs.py render` already emits generated catalog entries in this form.

It governs prose only, and says nothing about the newlines that carry structure. A fenced code block keeps whatever line breaks its content needs. A heading, a front-matter key, a table row, and a list marker each occupy their own line, and the newline between two list items is structure rather than a wrap — the rule never asks for two list items to be joined, only for the sentence inside one item not to be broken. A line may also pass any width because a link destination or an identifier admits no break point, which is not a violation because no wrap was available to omit.

Most of the corpus predates this rule and is hard-wrapped near 78 columns. A document keeps the form it has until something other than wrapping justifies rewriting it: editing one paragraph of a hard-wrapped document neither obliges an author to reflow the file nor licenses leaving the rewritten paragraph wrapped. Write the paragraph you touch as one line and leave its neighbours alone. The file is then internally inconsistent, and that is the accepted transitional state, because reflowing a whole document in order to change a sentence buries the substantive edit in a diff nobody can review and discards the `git blame` record of every line the change never touched.

The paragraph, not the file, is the unit that converts, so a paragraph that is itself half-wrapped is a defect rather than a transitional state. Do not append an unwrapped sentence to a hard-wrapped paragraph and leave the join ragged; rewrite that whole paragraph as one line, which is the convention, or match the paragraph's existing wrapping while it stands.

No repository check enforces this, and none is proposed while the corpus is mixed. Of the two mechanical shapes available only one states the rule at all: a maximum-line-length check asserts the opposite convention and must never be added, whereas a check that no paragraph contains a line break states it exactly. The locked CommonMark parser makes the second one precise rather than heuristic — the predicate is a `softbreak` inside a `paragraph` token, so a wide table row or a fenced block cannot trip it. It is still not worth adding today, because it would fail on most of the corpus, and narrowing it to newly authored documents would need either a stored per-file exemption list that every conversion and every parallel branch must edit, or a diff base that a whole-tree gate does not have. Adding the check becomes correct and cheap once the corpus is uniform; until then review enforces the rule.

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

Contract `governed_by` is derived from decision `applies_to`; research `reproduced_by` is derived from experiment `supports`. These backlink fields are invalid in stored v1 frontmatter. `related` is symmetric, stored only once on the lexicographically smaller source ID, and licensed only for the navigational kinds marked in the table below. A contract, decision, research report, or experiment already owns a directed predicate for every association it can make, so recording one as `related` would discard the direction that names which document supersedes, refines, or depends on the other. A generic `links` or `deps` field is invalid. Human Markdown still links the important route in prose; frontmatter does not replace explanation.

### A decision does not cite an experiment in metadata

A decision reaches the experiment that reproduces a measurement through the research record it cites, and owns no edge of its own to a harness. `evidence` admits only a `research` target and is deliberately not relaxed to admit an `experiment`. A decision able to name a harness directly is no longer pushed to say anywhere what bounded universe, environment, and procedure make the measurement carry the weight the decision puts on it, and naming that boundary is exactly the research record's job — an experiment establishes what was observed, not that the observation generalizes. A stored `reproduced_by` on a decision fails for a second and independent reason: the name is already the derived research backlink defined above, so one key would be stored on one kind and calculated on another. `depends_on` carries no type rule and would therefore accept a decision-to-experiment target, but it asserts document dependency rather than reproduction and no generated catalog renders it; it is not the spelling for this.

The mechanism a decision uses instead is an ordinary body link to the checked-in harness, which is enforced rather than conventional: `validate_links` resolves every local link in every governed document, so a decision citing a harness that has moved or been deleted fails the repository gate. Prose is also the only place the qualification fits — which fixture carries which claim, on which toolchain, and what a refresh would destroy — and none of that survives a one-line catalog entry.

The generated ADR catalog stays frontmatter-only for the same reason, and the cheapest mechanical alternative was measured before being rejected rather than assumed unworkable. Rendering each decision's experiments transitively, as the experiments supporting the research records it already cites, needs no schema change and no decision edited — and it names the wrong harness for the case that raised the question. **Measurement — the corpus at `ab67a8d`.** ADR 0074's convention 5 amendment rests on `spikes/extensions/non-exhaustive-visibility/`, whose experiment record is `tiler.spike.extensions`; that record `supports` `operation-extension-surface`, `operation-extension-api`, and `proc-macro-extension-visibility`, and ADR 0074's `evidence` names none of the three. The transitive derivation therefore renders `tiler.spike.extensions.semantic-foundation-api-v2` for ADR 0074 — a real experiment, reached by a real edge, reproducing none of the measurements the amendment rests on. Thirty-nine of seventy-eight decisions reach some experiment that way, which is often enough for the line to be read as authoritative wherever it appears. The route that is correct is already two rendered links: the ADR catalog names each decision's research records, and the research catalog names each research record's experiments.

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

All kinds may use `depends_on`, `refines`, and `supersedes` where their typed meaning applies. `related` is not among them; the optional column above is its exhaustive licence. Present arrays are nonempty, contain unique homogeneous scalar values, and use no empty placeholder. A reproducible experiment requires nonempty `entrypoints` and `evidence_classes` plus a `last_verified` date. Those field rules bind on every experiment record carrying the field rather than on a reproducible one alone: `last_verified` is an ISO `YYYY-MM-DD` date no later than today, and entrypoints are normalized repository-root POSIX paths to existing regular files; absolute paths, backslashes, `.`/`..`, directories, and repo escapes are invalid.

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

This document owns metadata shape, relationship semantics, and the source form of governed Markdown. It does not own the architectural content being indexed, ticketsplease's ticket schema, or the meaning of evidence inside a research report.

Quotations attributed between governed documents are **not** mechanically checked. A rule that did so was retired: it mined 471 quoted spans on this corpus and checked 42 of them, so a green gate proved nothing about the other 429 while reading as though quotations were verified. A check whose population it cannot name is the shape this repository is elsewhere required to distrust. Verifying that a quotation still says what its source says is review work, and a paragraph quoting another document should cite the commit that last moved the wording.
