---
id: reconcile-research-evidence-provenance
title: Reconcile research evidence provenance
status: done
priority: p1
dependencies: [repair-numerical-witness-integrity, repair-macro-and-embedding-harness-integrity, repair-cache-experiment-harness-integrity, repair-apple-target-experiment-integrity, repair-shape-and-runtime-experiment-integrity, enforce-repository-validation-gate-integrity]
related: []
scopes: [contracts/navigation, research/program-planning, research/shapes, research/target-profiles, research/documentation, research/embedding, research/apple-targets, research/macro-environment, research/runtime, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, documentation, evidence]
---

Reconcile the documentation evidence graph and retained records after the
underlying harness repairs. This is evidence curation, not permission to
promote a proposal or local observation into a portable guarantee.

## Required outcome

- Add tracked executable support or remove/narrow `executable-model` for:
  `tiler.research.program-planning.abi-expression-ownership`,
  `tiler.research.program-planning.general-compilation-boundary`,
  `tiler.research.shapes.constraint-prover-boundary`,
  `tiler.research.shapes.shape-environment-contract`, and
  `tiler.research.target-profiles.physical-feasibility-model`.
- Reconcile embedding claims about retained raw `size -m` and build streams,
  Apple compatibility claims about retained commands/artifacts/results,
  macro-environment trace claims, runtime benchmark metadata/samples, and
  stable/nightly shape summary generation.
- Retain the exact prompts and outputs supporting the blank-agent qualitative
  acceptance report, or reclassify it as a nonreproducible narrative review.
  The structural docs validator is not evidence of reader interpretation.
- Correct the reduction exhaustive-domain and empty-domain claims and all
  sound-accuracy oracle/observed-maximum provenance after rerunning the fixed
  harnesses.
- Ensure every completed ticket's acceptance language matches the artifacts
  actually retained. Preserve historical status, but add an explicit
  corrective link when a later audit invalidates a claimed guarantee.

## Acceptance

Every evidence class and record named by this ticket must have a traversable
supporting edge, exact bounded domain or procedure, tracked entrypoint, and
retained result sufficient for the stated claim. Run the complete documentation
renderer/validator, Ruff, pytest, `tkt lint`, and a repository-wide search for
stale versions of these specific evidence claims.

## Outcome

- Removed unsupported executable-evidence labels from five architectural
  reports and stated the exact partial-implementation boundary in each.
- Reconciled Apple, embedding, macro-environment, runtime, stable/nightly
  shape, reduction, and accuracy claims against retained artifacts. Added a
  producer-bound Apple schema-v2 record, isolated shape measurements, and a
  deterministic region-accuracy result with tool and source provenance.
- Reclassified the unretained blank-agent review as informational narrative
  evidence and removed it from the documentation experiment's support claims.
- Added explicit corrective links to the completed tickets whose historical
  acceptance language was superseded or narrowed by the integrity audits.
- Closure review further narrowed runtime/ABI/blank-agent wording, governed the
  retained region-accuracy result with schema and mutation checks, converted
  the target-feasibility test list into a future conformance requirement, and
  made Apple command, terminal-status, and reproducibility claims derived
  validator facts rather than trusted record fields.
- Regenerated both catalogs and passed the complete repository validation gate,
  including documentation validation, Ruff, 142 Python tests, the full Rust
  gate, shell validation, and ticketsplease lint.
