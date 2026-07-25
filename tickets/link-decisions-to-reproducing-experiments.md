---
id: link-decisions-to-reproducing-experiments
title: Give a decision a metadata edge to the experiment that reproduces it
status: in-progress
priority: p3
dependencies: []
related: [preserve-non-exhaustive-visibility-probe]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, metadata]
claimed_from: todo
assignee: agent-navigation
lease_expires_at: 1784996387
---
`scripts/docs.py` types the `evidence` relation so a `decision` may only cite a `research` record (`type_rules` in `validate_graph`), and `supports` so an `experiment` may only cite `research`. There is therefore no metadata edge from a decision to the experiment that reproduces its measurement.

**Where it shows.** ADR 0074's convention 5 amendment rests on two measurements of Rust's `#[non_exhaustive]` behaviour. `preserve-non-exhaustive-visibility-probe` checked the harness in at `spikes/extensions/non-exhaustive-visibility/` and linked it from the amendment's body. `validate_links` does check that body link resolves, so it cannot rot silently — but the generated ADR catalog in `docs/decisions/README.md` renders `evidence:` from frontmatter alone, so a reader of the catalog sees the ADR's research evidence and never learns that a runnable reproduction exists.

**The question is whether that is a gap worth closing, and it is not obvious.** Routing an experiment through the research record it `supports` is the current design and is arguably correct: research is where a claim is argued, and an experiment supports the argument rather than the decision. Against that, ADR 0074's amendment cites the measurement directly and has no research record between it and the harness, so the indirection has nothing to route through.

Options: relax the `evidence` type rule to admit `experiment`; add a distinct `reproduced_by` relation for decisions; or record that the body link is the intended mechanism and that the catalog is deliberately frontmatter-only. Whichever is chosen, state it in `docs/document-metadata.md` so the next author is not left guessing why an accepted decision cannot cite its own harness in metadata.

Low priority: nothing is currently unverified or broken by this, and one accepted ADR is affected.
