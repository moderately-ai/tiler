---
id: name-the-physical-selection-role-in-the-expansion-subject-facet-doc
title: Name the physical-selection role in the expansion subject facet doc
status: todo
priority: p3
dependencies: []
related: [repair-the-artifact-identity-prose-the-v22-run-falsified]
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, artifact, identity, cache]
---
## User-visible outcome

The expansion cache's artifact-program subject facet describes the artifact facts it keys on in the vocabulary the artifact layer actually uses, so a reader deriving the facet's contents from its doc does not miss a subject that already moves artifact identity.

## Why this exists

Found 2026-08-22 by the sibling scan of [`repair-the-artifact-identity-prose-the-v22-run-falsified`](repair-the-artifact-identity-prose-the-v22-run-falsified.md), which repaired the same two defects in `crates/tiler-artifact` but could not reach this file: `crates/tiler-cache/**` maps to `implementation/cache`, which that ticket does not hold.

**Fact — the facet doc is role-unqualified and predates the `v22` run.** `crates/tiler-cache/src/expansion/subject.rs`, anchor `requirements, and selected capability providers.` on `SubjectFacet::ArtifactProgram`. It says "selected capability providers" without saying *lowering*, which the compilation-environment role separation made ambiguous, and it names no physical-selection run. Since `tiler.artifact-program.v22` each variant carries a required non-empty run of selected physical implementations, folded into canonical artifact identity by `crates/tiler-artifact/src/program/model.rs`, anchor `push_selected_physical_implementation_run(bytes, &variant.selected_physical_implementations);`.

**Unverified by that scan, and the real question here.** Whether the facet's *derivation* — not only its prose — already covers the physical-selection run. If the facet is derived from the artifact's canonical identity it covers the run for free and only the prose is stale; if it re-enumerates artifact facts independently, then a cache keyed on it cannot distinguish two plans admitted by different physical authorities, which is the same wrong-identity defect the `v22` step exists to close and is a correctness bug rather than a doc repair. Read the construction site before deciding which this is; do not repair the prose against this ticket's Facts without that read.

## Required work

- Re-audit both Facts at your own base, then read the facet's construction and consumption sites in full.
- Decide which of the two cases above holds, and say so with the evidence.
- If it is prose only, qualify the role and name the run. If the derivation genuinely omits the run, stop, record the finding, and split the correctness repair into its own ticket rather than folding it into a doc change.

## Non-goals

Changing any encoded byte, cache key, or subject derivation under the prose-only reading. Re-deriving the `v22` step, which landed gated.

## Closes when

The facet doc states the provider role and the physical-selection run's status in the facet, the derivation question is answered from source with evidence, and any correctness remainder is filed as its own ticket.
