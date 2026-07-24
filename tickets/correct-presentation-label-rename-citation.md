---
id: correct-presentation-label-rename-citation
title: Correct the select_alternative citation in the presentation-label rename outcome
status: done
priority: p3
dependencies: []
related: [disambiguate-presentation-label-from-semantic-key-accessors, record-presentation-label-naming-resolution]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, evidence]
---
The merged outcome of `disambiguate-presentation-label-from-semantic-key-accessors` cites a function that does not exist. It states that `pipeline::ProgramAlternative::stable_id` "*is* compared as meaning — `select_alternative` decides the selected alternative with `alternative.stable_id == selected_alternative_id`".

**Fact — there is no `select_alternative` anywhere in the workspace.** `grep -rn "select_alternative" . --exclude-dir=.git --exclude-dir=target` at commit `7171346` matches only ticket prose: that outcome line and the body of `record-presentation-label-naming-resolution`, which inherited the shorthand from it. No Rust source matches.

**Fact — the substance the outcome asserts is correct, and the citation conflates two functions.** `ProgramAlternative::stable_id` is genuinely compared as meaning, in `crates/tiler-compiler/src/pipeline.rs`. `select_structural_pareto` makes the decision and returns `Ok(selected.stable_id)`; `PortfolioSelection::selected_alternative_id` carries it; `verify_portfolio` dedups the alternatives on `stable_id` through a `BTreeSet` and rejects with `portfolio-identity` on collision, then rejects with `portfolio-selection` unless the recorded selection equals the recomputed one; and `record_cost_and_selection` is where the literal `alternative.stable_id == selected_alternative_id` comparison lives, deciding the explain `SelectionOutcome::Selected`. So the comparison the outcome quotes is real but sits in the explain recorder, while the selection decision it attributes to that comparison is made elsewhere.

The durable contract is already correct: `record-presentation-label-naming-resolution` verified the path by reading `pipeline.rs` and recorded the accurate function names in ADR 0074 convention 2. What remains is the evidence trail in the completed ticket, which a future reader tracing the claim would find unreproducible.

Replace the `select_alternative` clause in that outcome with the accurate call chain above. Do not otherwise rewrite the outcome; every other fact in it was independently re-verified against source at `7171346` and holds.

## Outcome

Done by the coordinator at merge time rather than dispatched, because the ticket holds no exclusive scope and the fix is a two-sentence correction in `tickets/`, which the coordinator already holds on `main`.

**Both occurrences corrected, not one.** The ticket named the merged outcome of `disambiguate-presentation-label-from-semantic-key-accessors`; the same false citation had also propagated into the body of `record-presentation-label-naming-resolution`, because the coordinator relayed the claim from the first agent's report into the second agent's brief without reading `pipeline.rs`. That is the actual origin of the error, and it is worth naming: an unverified claim in a report became an instruction in a brief, and only the receiving agent's refusal to inherit it caught the problem.

Each correction states the real chain — `select_structural_pareto` → `PortfolioSelection::selected_alternative_id` → the `BTreeSet` dedup in `verify_portfolio` → the `alternative.stable_id == selected_alternative_id` comparison in `record_cost_and_selection` at `pipeline.rs:1522` — and is marked as an after-the-fact correction rather than silently rewritten, so the evidence trail shows what was believed and what replaced it.

**Verified independently before applying.** `grep -rn "select_alternative" . --exclude-dir=.git --exclude-dir=target` matches only the two prose files and no Rust source; `BTreeSet` appears at `pipeline.rs:1578` with the `portfolio-identity` rejection at 1584. The durable contract in ADR 0074 was already accurate and needed no change.
