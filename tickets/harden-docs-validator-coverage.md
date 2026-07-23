---
id: harden-docs-validator-coverage
title: Harden documentation validator coverage
status: done
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation, contracts/decisions, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, tooling]
---
Three verified gaps where docs.py enforces less than document-metadata.md promises, letting status-bearing prose drift silently:

- the hand-maintained "Chronological index" in docs/decisions/README.md is invisible to the validator and stale (ends at ADR 0059 while the corpus reaches 0072); fold it into the generated-catalog machinery (extend the renderer and markers) or delete it in favour of the generated thematic catalog, then regenerate;
- docs.py licenses `related` frontmatter on every kind while document-metadata.md's kind-field table licenses it on four kinds only, and live instances exist on unlicensed kinds (ADR-0056, ADR-0070, one research doc); reconcile contract and validator in one deliberate direction and migrate the instances; and
- entrypoints/last_verified well-formedness and date checks run only when experiment_status is "reproducible" while the metadata contract states the field rules unconditionally; validate them on every experiment record.

Also correct the stale status prose this ticket's scope owns: docs/status.md line 88 still names the completed verifier subject-binding correction as "the immediate compiler frontier" (that ticket is done and merged), and docs/roadmap.md still lists it as pending work. Point both at the current frontier.

Lock each closed gap with a scripts/tests case so the gate cannot regress. Run the full documentation gate before completion. This ticket exclusively holds `contracts/navigation`; `repair-research-evidence-residuals` also needs it for two catalog-regenerating frontmatter fixes, so merge this ticket first and let that one follow.

## Outcome

All four gaps are closed and each is locked by a `scripts/tests/test_docs.py` case.

**Chronological index — folded into the generated machinery rather than deleted.** The thematic catalog groups ADRs by `catalog_group` and never shows the number ordering that the on-disk `NNNN-*.md` naming uses, so deleting the index would have removed the only number-to-title route in the corpus. `docs.py` gained a `CHRONOLOGY` marker, a `chronology()` renderer keyed on the already-validated `ADR-NNNN` id, and a `generated()` block list so one file can carry more than one generated block. The checked-in section now covers ADRs 0001-0072 and `render --check` fails whenever it drifts.

**`related` — restricted to the four kinds the contract licenses, not licensed everywhere.** The contract's own closing rule ("All kinds may use `depends_on`, `refines`, and `supersedes`") already omits `related` deliberately, and every live instance on an unlicensed kind proved to be a mis-encoded typed edge rather than a genuine untyped association, so licensing would have entrenched the defect. `related` moved out of `COMMON` into the `portal`, `roadmap`, `questions`, and `prior-art` field sets. Three instances migrated:

- ADR-0056 `related: ["ADR-0065", "ADR-0070"]` deleted. Both halves restated, backwards, supersessions its own status line already names. ADR-0065 already carried `supersedes: ["ADR-0056"]`; ADR-0070 did not, so `supersedes: ["ADR-0056"]` was added there, matching its Decision section ("This decision supersedes only ADR 0056's retained compiler-to-artifact edge").
- ADR-0070 `related: ["ADR-0071"]` became `refines: ["ADR-0070"]` on ADR-0071, which constrains construction of exactly the shared IR layers ADR-0070 places in `tiler-ir`. This follows the ADR-0066-refines-ADR-0044 precedent.
- `tiler.research.shapes.public-static-shape-spelling` `related` became `depends_on: ["tiler.research.shapes.stable-rust-shape-evidence"]`; the spelling study takes the feasibility report's facts and retained harness as given.

The `related` field otherwise keeps its symmetric lexicographic-storage rule, and the surviving instances on `docs/status.md`, `docs/open-questions.md`, and `spikes/indexing/index-access-model/README.md` are all licensed navigational kinds.

**Experiment field rules now bind on every experiment record.** Presence of `entrypoints`/`last_verified` stays a `reproducible`-only requirement, but ISO-date, future-date, and repository-root entrypoint well-formedness are validated wherever the field exists, so a `planned`, `partial`, or `blocked` record can no longer park a malformed value until promotion. No live record changed status as a result; all 24 experiment records are already `reproducible`.

**Frontier prose corrected against the live board.** `harden-compiler-verifier-subject-binding-and-totality` is `done` and merged in `f14e7ad`. `docs/status.md` now records it among the integrated corrections and names the actual frontier: the three dependency-satisfied p0 authorities `prototype-typed-explain-infrastructure`, `prototype-operation-capability-registry`, and `prototype-index-region-reference-oracle`, which `tkt ready` shows may proceed in parallel. `docs/roadmap.md` stage 1 no longer lists the completed verifier work and now reflects that capability registration and the index oracle are unblocked alongside typed explain rather than gated behind it.

`docs/document-metadata.md` records both contract changes: the `related` licence is now stated once with its rationale and the kind table named as its exhaustive authority, and the experiment field rules are stated as binding on every record carrying the field.

`uv run --locked python scripts/check_repository.py` passes end to end, covering documentation render/validate, Ruff format and lint, 147 pytest tests (four new), ShellCheck, ticketsplease lint, and the full Rust gate. `tkt lint`, `git diff --check`, and `tkt guard tkt/harden-docs-validator-coverage` are clean.
