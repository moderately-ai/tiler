---
id: make-explain-dispositions-assertable-by-a-conformance-suite
title: Make explain dispositions assertable by a conformance suite
status: deferred
priority: p2
dependencies: [decide-the-backend-provider-conformance-harness-public-surface]
related: [audit-backend-authoring-against-all-thirteen-responsibilities, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [explainability, conformance, backend-providers, public-boundary]
---
## User-visible outcome

A backend-author conformance suite can assert that each provider disposition it must distinguish — a suite-facing superset of the ADR 0078 / operation-extensions five (admission, rejection, ambiguity, absence, exhausted proof budget), not a contract quote of that list alone; declined strategy and cost disadvantage are example optimizer outcomes the suite may also enumerate — reached the explain trace as its own outcome, without depending on rendered text the contract says is not a parse target.

## Why this is blocked rather than dispatchable

**Fact — the obligation exists and the surface to assert it against does not.** [The operation-extension contract](../docs/operation-extensions.md#public-extension-seams) makes it one of the four properties jointly admitting a seam that "every disposition — admission, rejection, ambiguity, absence, and an exhausted proof budget — is a distinct typed outcome that reaches the explain trace". [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) asks a suite to cover empty offers, malformed proposals, verifier bypass attempts, and forged provenance/resources, and to document exact maturity.

**Fact — public explain products expose no disposition iteration, and documentation forbids treating rendered text as an interface.** `ExplainReport` (`pub struct ExplainReport` in `crates/tiler-compiler/src/session.rs`) exposes only `ExplainReport::render` (`pub fn render(&self) -> String` under the "not a parse target" doc). Success-path `VerifiedCompilationExplain` (re-exported via `pub use crate::explain::VerifiedCompilationExplain;` in `session.rs`) publicly exposes `render` and `semantic_candidate_count` only — neither yields dispositions or record iteration. `mod explain;` is private in `crates/tiler-compiler/src/lib.rs`, so an out-of-crate suite cannot name `ExplainDisposition`. The `ExplainReport` doc states that the rendered form "is a diagnostic for a human reader and **not a parse target**", that the leading `tiler-explain-v<N>` changes when the rendering does, and that committing to the text "would create a second description of the trace that has to be kept in agreement with its canonical bytes, which is the duplicate-derivation hazard this whole boundary is shaped to avoid".

**Correction — 2026-08-10.** Prior line citations for `ExplainReport` (`session.rs:1137`), `render` (`:1176`), `mod explain` (`lib.rs:22`), and the re-export (`session.rs:65`) have drifted; use the symbol and doc anchors above. The prior claim that the only public accessor is a rendered string understated that `VerifiedCompilationExplain` also exposes `semantic_candidate_count`; the obligation gap is unchanged because neither product exposes dispositions. The User-visible outcome seven-item disposition list is a suite-facing superset, not a quote of ADR 0078's five-item obligation.

**Inference — so a conformance suite today has three options and each is a decision nobody has made.** Parse the rendered text, which the contract above forbids and which would create the second description it exists to prevent; add a structured accessor over the trace, which is a public boundary and Tom's under ADR 0075; or assert dispositions only indirectly through the typed errors and selected-plan provenance already public, and state in the suite's documented maturity that explain coverage is out of scope. The third is cheapest and may well be right, but choosing it silently would let a suite report full coverage of an obligation it does not check.

**That original deferral trigger has now fired, but the public choice is not accepted.** The portfolio vertical exists and `decide-the-backend-provider-conformance-harness-public-surface` now owns the exact reusable facade and coverage boundary. This ticket therefore blocks on that decision rather than depending backward on the suite that needs its result. The accepted facade must choose either a structured non-rendered assertion surface or a documented explain exclusion before this implementation can close.

## Ripens when

The backend-provider conformance suite reaches design and enumerates which dispositions it must distinguish. At that point this becomes one atomic question for Tom — structured accessor, or documented scope limit — with the cost of each stated.

## Closes when

The suite either asserts dispositions through a surface that is not the rendered text, or documents explain coverage as explicitly out of its scope with the reason; and no suite reports coverage of the disposition obligation it does not check.

## Trigger check log

- 2026-08-05 — not fired. `publish-the-backend-provider-conformance-suite` is `todo` and itself blocked on `exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`, so no suite has enumerated its dispositions. Reproduce with `grep -m1 '^status:' tickets/publish-the-backend-provider-conformance-suite.md`.
- 2026-08-09 — **not fired.** [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) remains `todo`, and its end-to-end portfolio consumer [`exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`](exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md) remains `todo`. No published suite yet consumes explain dispositions as an assertable provider contract, so this stays deferred behind that subject rather than being implemented against local trace fixtures alone.
- 2026-08-17 — **fired.** The three-provider portfolio is `done`, and `decide-the-backend-provider-conformance-harness-public-surface` now enumerates the reusable suite's exact facade and coverage choice. The old suite→explain edge was backward and is replaced by decision→explain→suite, with this ticket blocked until Tom accepts the facade's structured-accessor or explicit-exclusion answer.

- **Correction — 2026-08-22: the three entries above are preserved verbatim, and none of them evaluates the condition this ticket was actually deferred under.** Two of them also carried no reproducing command, which [AGENTS.md](../AGENTS.md) requires of every dated entry. Commands are supplied here rather than folded into those entries, so their wording and every count over it stays intact — across this repair a count that *shrank* would be the failure signal, not progress.

  **The condition this ticket was deferred under, read from the accepting record.** [The 2026-08-18 acceptance](decide-the-backend-provider-conformance-harness-public-surface.md) names this ticket by id, saying both carriers move to `deferred` `with that trigger rather than becoming dispatchable`. *That trigger* is the sentence immediately preceding it: one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects, of which the record says `that evidence alone reopens the partial-facade question`. So the acceptance does name a condition for this ticket specifically, and it is the same condition [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) logs — not an explain-specific one.

  **A second condition also bears on this ticket, and the two are not rivals.** The recommendation's numbered `Explain coverage-expansion trigger` says an `independently accepted structured, non-rendered accessor` `permits adding explain-disposition coverage later`, and that `it is not a prerequisite to publishing a facade that types the exclusion`. Both conditions are stated as **sufficient** and neither as necessary — the same shape [`reconcile-the-conformance-decisions-two-statements-of-its-own-trigger`](reconcile-the-conformance-decisions-two-statements-of-its-own-trigger.md) settled for the numbered triggers, where a weaker sufficient condition subsumes a stronger one instead of competing with it. They are therefore logged below as two independent entries with different consequences rather than weighed against each other: firing the first reopens the facade decision this ticket depends on, while firing the second widens what an already-accepted facade may claim. The record also fixes what a reopened facade does to this ticket — `an accepted partial facade that explicitly excludes explain can close it without an implementation edge`, while the `explain ticket remains blocked while no facade is accepted`.

  **What the 2026-08-17 `fired` entry was against, and whether that firing still stands.** Not the acceptance record's reopening condition, which did not exist until the following day. It evaluates this ticket's own `## Ripens when` on a substituted subject: that section names *the suite* reaching design and enumerating which dispositions it must distinguish, whereas the entry records the three-provider portfolio being `done` and [the decision ticket](decide-the-backend-provider-conformance-harness-public-surface.md) having taken ownership of the facade and coverage choice. Both of its stated grounds re-verify at their own date, so the firing stands as recorded; the substitution is the honest one, because no suite ever reached design and the coverage choice moved to that decision instead. It is therefore neither a mislabelled entry nor a missed dispatch — its one real defect is the absent command, repaired here.

  **Why the firing produced no dispatch.** The entry says so itself: `but the public choice is not accepted`. That same day `7f839294` moved this ticket from `deferred` to `blocked` and replaced its backward `publish-the-backend-provider-conformance-suite` dependency with the decision, so the firing did produce a graph repair rather than nothing. The next day the answer arrived and it was a typed deferral — `33c5db60` moved this ticket back to `deferred` — because that decision's completion means "do not build now", not "build". A ripening whose answer is *do not build* re-defers instead of dispatching. What the log then failed to do was record the new condition, which is the defect this correction repairs, not the firing.

  Reproduce the three preserved verdicts; each prints the status its entry asserts, read at the last commit of that entry's own date. Under `zsh` the braces in `${c}:tickets/…` are load-bearing, because `$c:t` is parsed as a history modifier and the command then fails with `unknown revision or path`:

  ```sh
  for d in 2026-08-05 2026-08-09 2026-08-17; do
    c=$(git rev-list -1 --before="$d 23:59:59" HEAD)
    printf '%s %s  publish=%s  portfolio=%s\n' "$d" "$(echo $c | cut -c1-8)" \
      "$(git show "${c}:tickets/publish-the-backend-provider-conformance-suite.md" | grep -m1 '^status:' | cut -d' ' -f2)" \
      "$(git show "${c}:tickets/exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md" | grep -m1 '^status:' | cut -d' ' -f2)"
  done
  ```

  Actual output at `518c56c3`:

  ```text
  2026-08-05 2019c918  publish=todo  portfolio=todo
  2026-08-09 c99ac549  publish=todo  portfolio=todo
  2026-08-17 442f5238  publish=blocked  portfolio=done
  ```

  The 2026-08-05 and 2026-08-09 `not fired` verdicts rest on both tickets being `todo`, and the 2026-08-17 `fired` verdict on the portfolio being `done`; all three re-verify. Reading the same two `status:` lines at today's tip instead reports `publish=deferred portfolio=done`, which is why this form pins each entry to its own date — a tip-only status check cannot reproduce a historical verdict and would look like a contradiction instead of a confirmation.

- 2026-08-22 — **fired**, on the reopening condition the 2026-08-18 acceptance actually deferred this carrier under. One second independently authored backend fixture now exists sharing the portfolio's neutral, non-self-certifying structural and execution subjects: `crates/tiler-conformance/tests/independent_backend/` carries `the_assembled_artifact_carries_facts_this_backend_never_supplied` and `the_routed_result_agrees_with_the_reference_oracle`, and the comparator the condition names, `spikes/runtime/backend-provider-portfolio`, carries both subjects itself through `assemble_plan_artifact` and `route_with_adapter` against `tiler-reference`. The same condition was evaluated `fired` on [`publish-the-backend-provider-conformance-suite`](publish-the-backend-provider-conformance-suite.md) on the same date, from the resolution recorded in [`reconcile-the-conformance-decisions-two-statements-of-its-own-trigger`](reconcile-the-conformance-decisions-two-statements-of-its-own-trigger.md); this entry follows that resolution rather than re-deriving it, and adds only this carrier's own consequence.

  **What firing does and does not mean here.** It reopens the partial-facade question for re-presentation to Tom; it authorizes no public export, no `pub mod`, no structured explain accessor, and no implementation on this ticket, and the accepted deferral's stop boundary is unchanged. Because the decision record says `an accepted partial facade that explicitly excludes explain can close it without an implementation edge`, the reopened question can terminate this ticket without any work landing here — so the coordinator's action is to route the reopening, not to dispatch this ticket. This entry does not move this ticket's `status` or `priority`, which are the coordinator's call.

  Reproduce (every number is a count of matching **lines**, not occurrences):

  ```sh
  D=tickets/decide-the-backend-provider-conformance-harness-public-surface.md
  C=tickets/publish-the-backend-provider-conformance-suite.md
  F=crates/tiler-conformance/tests/independent_backend/main.rs
  S=spikes/runtime/backend-provider-portfolio
  printf 'acceptance names this carrier:   %s\n' "$(grep -c 'with that trigger rather than becoming dispatchable' $D)"
  printf 'operative condition stated:      %s\n' "$(grep -c 'that evidence alone reopens the partial-facade question' $D)"
  printf 'fixture structural subject:      %s\n' "$(grep -c 'fn the_assembled_artifact_carries_facts_this_backend_never_supplied' $F)"
  printf 'fixture execution subject:       %s\n' "$(grep -c 'fn the_routed_result_agrees_with_the_reference_oracle' $F)"
  printf 'portfolio structural seam:       %s\n' "$(grep -rl 'assemble_plan_artifact' $S/src | wc -l | tr -d ' ')"
  printf 'portfolio execution seam:        %s\n' "$(grep -rl 'route_with_adapter' $S/src | wc -l | tr -d ' ')"
  printf 'sibling carrier logged fired:    %s\n' "$(grep -c '2026-08-22 (supersedes the entry above) — \*\*fired\.\*\*' $C)"
  printf 'fixture routes (tree-wide):      %s\n' "$(grep -rl 'route_with_adapter' crates/tiler-conformance/tests/independent_backend/ | wc -l | tr -d ' ')"
  printf 'fixture reaches oracle:          %s\n' "$(grep -rl 'tiler_reference' crates/tiler-conformance/tests/independent_backend/ | wc -l | tr -d ' ')"
  printf 'same two greps in main.rs only:  %s\n' "$(grep -c 'route_with_adapter\|tiler_reference' $F)"
  ```

  Actual output at `518c56c3`:

  ```text
  acceptance names this carrier:   1
  operative condition stated:      2
  fixture structural subject:      1
  fixture execution subject:       1
  portfolio structural seam:       2
  portfolio execution seam:        3
  sibling carrier logged fired:    1
  fixture routes (tree-wide):      1
  fixture reaches oracle:          1
  same two greps in main.rs only:  0
  ```

  **The last three lines are a negative control against a false absence, and it fires here.** The fixture is split across five files, so `route_with_adapter` and `tiler_reference` are absent from `main.rs` and a per-file grep reports `0` — which reads as *the fixture neither routes nor reaches the oracle* rather than *the module was split*. The tree-wide counts locate them at `crates/tiler-conformance/tests/independent_backend/nodefold_adapter.rs` and `.../workload.rs`; `assemble_plan_artifact` is called from `.../nodefold.rs`. Search the directory, never `main.rs`, and treat a `0` on the last line as expected rather than as a regression.

  `operative condition stated` is **2** rather than 1 because the decision record's dated correction quotes the retired wording, as this repository's convention requires; before that repair it was 1. The first line is the one that ties the condition to *this* ticket by id, and would fall to 0 if the acceptance paragraph were rewritten to drop this carrier. What this block cannot do is read for meaning: it confirms the sentences and the two subjects are present, while the judgement that the condition is satisfied rests on the reading above and on the resolution it cites.

- 2026-08-22 — **not fired**, on the separate `Explain coverage-expansion trigger`, which is this ticket's own subject matter and is logged apart from the entry above because the two are independent sufficient conditions with different consequences. That trigger requires an `independently accepted structured, non-rendered accessor`. None exists: `ExplainDisposition` is `pub(crate)` inside `crates/tiler-compiler/src/explain.rs`, `mod explain;` in `crates/tiler-compiler/src/lib.rs` is private, and nothing re-exports the type. No acceptance of such an accessor is recorded anywhere. Its not firing blocks nothing, since the decision states `it is not a prerequisite to publishing a facade that types the exclusion`.

  Reproduce. The definition grep is deliberately unanchored, because a pattern anchored `^pub ` or `^(pub )?` cannot match a `pub(crate)` item and would report absence instead of privacy — the dangerous direction:

  ```sh
  D=tickets/decide-the-backend-provider-conformance-harness-public-surface.md
  printf 'ExplainDisposition definition:   %s\n' "$(grep -rn 'enum ExplainDisposition' crates/tiler-compiler/src/)"
  printf 'public re-export of it:          %s\n' "$(cat crates/tiler-compiler/src/lib.rs crates/tiler-compiler/src/session.rs | grep -c 'pub use.*ExplainDisposition')"
  printf 'explain module declaration:      %s\n' "$(grep -n 'mod explain;' crates/tiler-compiler/src/lib.rs)"
  printf 'accepted-accessor trigger text:  %s\n' "$(grep -c 'independently accepted structured, non-rendered accessor' $D)"
  ```

  Actual output at `518c56c3`:

  ```text
  ExplainDisposition definition:   crates/tiler-compiler/src/explain.rs:230:pub(crate) enum ExplainDisposition {
  public re-export of it:          0
  explain module declaration:      37:mod explain;
  accepted-accessor trigger text:  1
  ```

  This entry says `not fired` when the first line reports `pub(crate)` and the second reports `0`. It would say `fired` when the definition line reports a bare `pub`, or the re-export count becomes non-zero, or `mod explain;` becomes `pub mod explain;` — and, because visibility alone is not acceptance, only once a decision record carries Tom's acceptance of that exact surface under ADR 0075. The emitted line numbers are output rather than citations, so they cannot rot; the greps that produce them are the anchors.
