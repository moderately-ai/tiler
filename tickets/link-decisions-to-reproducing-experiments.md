---
id: link-decisions-to-reproducing-experiments
title: Give a decision a metadata edge to the experiment that reproduces it
status: done
priority: p3
dependencies: []
related: [preserve-non-exhaustive-visibility-probe]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, metadata]
---
`scripts/docs.py` types the `evidence` relation so a `decision` may only cite a `research` record (`type_rules` in `validate_graph`), and `supports` so an `experiment` may only cite `research`. There is therefore no metadata edge from a decision to the experiment that reproduces its measurement.

**Where it shows.** ADR 0074's convention 5 amendment rests on two measurements of Rust's `#[non_exhaustive]` behaviour. `preserve-non-exhaustive-visibility-probe` checked the harness in at `spikes/extensions/non-exhaustive-visibility/` and linked it from the amendment's body. `validate_links` does check that body link resolves, so it cannot rot silently — but the generated ADR catalog in `docs/decisions/README.md` renders `evidence:` from frontmatter alone, so a reader of the catalog sees the ADR's research evidence and never learns that a runnable reproduction exists.

**The question is whether that is a gap worth closing, and it is not obvious.** Routing an experiment through the research record it `supports` is the current design and is arguably correct: research is where a claim is argued, and an experiment supports the argument rather than the decision. Against that, ADR 0074's amendment cites the measurement directly and has no research record between it and the harness, so the indirection has nothing to route through.

Options: relax the `evidence` type rule to admit `experiment`; add a distinct `reproduced_by` relation for decisions; or record that the body link is the intended mechanism and that the catalog is deliberately frontmatter-only. Whichever is chosen, state it in `docs/document-metadata.md` so the next author is not left guessing why an accepted decision cannot cite its own harness in metadata.

Low priority: nothing is currently unverified or broken by this, and one accepted ADR is affected.

## Outcome

**Decided: the third option — the body link is the mechanism and the ADR catalog stays frontmatter-only.** `docs/document-metadata.md` gained a "A decision does not cite an experiment in metadata" subsection under **Typed relationships** stating the rule, rejecting each alternative with its reason, and naming the route that does work. No schema, no validator, and no ADR changed.

**Fact — the second option is not merely undesirable, it is unavailable.** `docs/document-metadata.md` already defines `reproduced_by` as the *derived* research backlink over experiment `supports`, and states that derived backlink fields are invalid in stored v1 frontmatter. A stored `reproduced_by` on a decision would put one key in the stored namespace on one kind and in the calculated namespace on another. The ticket proposed it without that collision in view.

**Fact — a decision already has an untyped edge that reaches an experiment, and it is the wrong one.** `type_rules` in `validate_graph` constrains `applies_to`, `evidence`, `informs`, `adopted_by`, and `supports`; `depends_on`, `refines`, and `supersedes` carry no target-kind rule, and `depends_on` is in `COMMON` so every kind may use it. A decision may therefore already write `depends_on: ["tiler.spike.extensions"]` today and it validates. It asserts document dependency rather than reproduction, and no generated catalog renders it — `catalog()` renders `applies_to` and `evidence` for a decision and nothing else. The subsection says so, so the next author does not discover the hole and mistake it for the intended spelling.

**Measurement — the cheapest mechanical option was tried against the data and it fails on the motivating case. Corpus at `ab67a8d`, via `scripts/docs.py`'s own loader.** Rendering a decision's experiments transitively — the experiments supporting the research records it already cites — needs no schema change and no ADR edited, and 39 of 78 decisions would gain a line. ADR 0074 gains one too, and it is wrong: its `evidence` is `tiler.research.semantic-graph.rust-construction-lifecycle` and `tiler.research.extensions.semantic-foundation-api-v2`, while the harness the convention 5 amendment rests on is recorded by `tiler.spike.extensions`, which `supports` `operation-extension-surface`, `operation-extension-api`, and `proc-macro-extension-visibility` — none of ADR 0074's evidence. The derivation therefore names `tiler.spike.extensions.semantic-foundation-api-v2`, a real experiment reached by a real edge that reproduces none of the amendment's measurements. A populated, plausible, wrong line is worse than the absence this ticket opened on.

**Retracted from the ticket's framing.** "A reader of the catalog … never learns that a runnable reproduction exists" overstates it. The ADR catalog names each decision's research records, and the research catalog renders `experiments:` for each research record (`catalog()` builds `experiments_by_research` from experiment `supports`). The reproduction is two rendered links away for the 39 decisions whose evidence is backed by a spike. What is true is narrower and is the real finding above: for ADR 0074 that route leads to a different harness, because its amendment's measurement has no research record between it and the probe.

**Not done, and deliberately.** Writing a research record whose content is "rustc emits `E0004` across a crate boundary" was considered and rejected: the measurement is a compiler behaviour retained byte for byte as a gated `trybuild` expectation with a recorded toolchain, and there is no argument to interpose. The absence of a research record here is the honest shape of the evidence, not a gap.

Gate: `uv run --locked python scripts/docs.py render` (no change) and `uv run --locked python scripts/check_repository.py` both green.
