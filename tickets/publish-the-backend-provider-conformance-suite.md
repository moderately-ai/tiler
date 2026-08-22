---
id: publish-the-backend-provider-conformance-suite
title: Publish the backend-provider conformance suite
status: deferred
priority: p1
dependencies: [exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, decide-the-backend-provider-conformance-harness-public-surface, package-selected-physical-implementation-provenance-in-artifact-identity, carry-required-compilation-selection-identity-on-compile-profile-contexts, make-explain-dispositions-assertable-by-a-conformance-suite]
related: [compile-extension-spike-fixtures-in-the-gate, audit-backend-authoring-against-all-thirteen-responsibilities]
scopes: [implementation/conformance, implementation/compiler, implementation/build, implementation/artifact, implementation/runtime, contracts/numerics, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, testing, conformance]
---
## User-visible outcome

Third-party backend authors receive a reusable conformance harness that proves the host can reject invalid providers and that a passing provider composes deterministically through compilation, artifacts, routing, and execution.

## Exact-current Fact audit — 2026-08-17 at `d002cd55406522922e5eb750c8c4d9033dde4469`

- **False — an accepted reusable owner already exists.** ADR 0106 and the complete `tiler-conformance` crate header deliberately make every module test-only and export no public item. The crate is the correct cross-layer evidence owner, but publishing a third-party harness is a consequential new facade rather than extraction under an accepted surface. `decide-the-backend-provider-conformance-harness-public-surface` is the required prerequisite, and this implementation now declares `implementation/conformance`.
- **False — a missing runtime-adapter registry is malformed.** ADR 0090 deliberately makes the runtime adapter an explicitly supplied, independently selected value. `route_with_adapter` cannot be called without one; there is no ambient registry to be empty or missing. Conformance must test an explicitly supplied wrong, refusing, or incompatible adapter, not invent discovery.
- **False — an empty physical-provider offer is invalid.** `ProviderOffer` documents an empty offer as a legitimate local result. Malformed provider output, a typed decline, an empty offer, and a globally absent feasible plan are distinct subjects and must remain distinct.
- **Imprecise — all thirteen historical matrix rows are current public responsibilities.** The matrix remains the audit index, but scalar lowering is retired under ADR 0105 and opaque-call registration remains compiler-owned. The exact current conformance packet must count the active external rows, print the retired/internal exclusions, and fail if either population silently moves; it must not report a uniform thirteen-row pass.
- **Verified prerequisite gap.** The reusable end-to-end suite cannot truthfully bind complete selected physical provenance or compilation-selection provenance until the two carrier tickets named in `dependencies` land. Explain dispositions must either gain a structured assertion surface or be explicitly excluded by the accepted conformance facade before this suite reports its bounded coverage.

## Implementation keys

- Extract tests only after the three-provider vertical identifies the real public contracts; do not design a mock-only alternate API.
- Cover provider identity/revision stability, deterministic registration/freeze, duplicate and ambiguous authority, legitimate empty offers, malformed proposals, verifier bypass attempts, forged provenance/resources, unstable emission, payload/entry mismatch, explicitly supplied adapter refusals or incompatibility, incompatible target/representation, backend-aware routing, routing commit, and asynchronous resource lifetime.
- Separate semantic-equivalence obligations that require provider-supplied reference/conformance evidence from structural properties Tiler can rederive.
- Require each check to have a deliberate perturbation that fires; count the discovered test population so a glob matching nothing cannot pass.
- Supply an external-provider-shaped passing fixture and multiple failing fixtures that compile/run only against public surfaces.
- Document exact maturity: passing this suite is conformance to the bounded provider contract, not certification of arbitrary mathematical correctness or performance.
- Keep nextest as the test runner and retain required doc-tests separately.
- Present the exact public conformance-harness facade, types, and call sites to Tom before acceptance.
- Prefer an existing accepted public crate or module only when the accepted composition ADR assigns it that ownership. If a new crate is required, file and complete a separate crate-admission ticket before scaffolding it.

## Closes when

Every public provider component has positive and negative conformance coverage, every new check has demonstrated its failure path, the harness is consumer-neutral, documentation states its limits, targeted nextest and per-package Clippy pass, and one final `make full` passes.

## Graph maintenance

- Link the suite from the provider-composition contract, public API docs, correctness contract, and example providers.
- File backend-specific performance qualification separately; conformance must not turn cost measurements into correctness authority.
- Keep untrusted/dynamically loaded plugin certification deferred.


## Deferred — 2026-08-18

The owning decision (`decide-the-backend-provider-conformance-harness-public-surface`) was accepted as an exact typed deferral: no public conformance spelling now. This carrier defers on that decision's named reopening trigger rather than dispatching on the decision ticket's completion.

## Trigger check log

- 2026-08-18 — **not fired.** The trigger is one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects. No such fixture exists in-tree. *(Confirmed 2026-08-22 — this entry was later accused of quoting the decision's weaker wording. It does not: the sentence it quotes is the operative reopening condition, matching the acceptance record verbatim in substance. Its verdict for 2026-08-18 stands, and its reproduce command, which roots at `crates/tiler-conformance/` and could then only ever report absence, now has a non-empty root.)* Reproduce: enumerate independently authored fixtures under `crates/tiler-conformance/` and compare their structural/execution subjects for a shared non-self-certifying pair.

- 2026-08-22 (superseded by the entry below, kept verbatim) — **not fired, and the trigger this log quotes is not the one the decision numbers.** The second fixture now exists: `crates/tiler-conformance/tests/independent_backend/` landed at merge `829bd1f0`, 3,348 lines over five files, twelve device-free tests, `tiler-reference` as sole oracle, with the self-certifying, producer-minted-key, non-terminal-use, and census perturbations each quoted failing. It does **not** satisfy Trigger 1 as numbered, which reads *"A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject"*. No bounded extraction was performed — it was the delivering ticket's stated non-goal — and only one in-gate fixture carries the execution subject: `crates/tiler-build/tests/custom_backend` structurally cannot, because `crates/tiler-build/Cargo.toml` declares neither `tiler-runtime` nor `tiler-reference` (six tiler dependencies: artifact, cache, compiler, ir, metal, metal-aot). The other fixture asserting the execution subject is the backend-provider-portfolio spike, which sits outside `make full`. Reproduce: `grep -A 25 '^\[dependencies\]' crates/tiler-build/Cargo.toml | grep 'tiler-'` and confirm the two absences; then enumerate fixtures under `crates/tiler-conformance/tests/` and check which assert an execution subject. **The 2026-08-18 entry above quotes the decision's reversal-evidence paragraph, not its numbered Trigger 1, and the two are not equivalent** — one is satisfied by this delivery and one is not. That ambiguity is [`reconcile-the-conformance-decisions-two-statements-of-its-own-trigger`](reconcile-the-conformance-decisions-two-statements-of-its-own-trigger.md); this entry is recorded against the numbered trigger because reopening an accepted decision on an ambiguous trigger is the riskier direction, and is to be re-evaluated when that ticket resolves.

- 2026-08-22 (supersedes the entry above) — **fired.** The ambiguity that entry recorded is resolved in [`reconcile-the-conformance-decisions-two-statements-of-its-own-trigger`](reconcile-the-conformance-decisions-two-statements-of-its-own-trigger.md), and the resolution reverses its verdict. The decision states **no necessary** reopening condition: numbered Trigger 1 leads *"Partial-facade reopening trigger, sufficient on its own"*, the recommendation closes that Trigger 1 *"is sufficient to reopen this decision"* and that its *"triggers are independent rather than an all-or-nothing conjunction"*, and `.ticketsplease/decision-queue.md` item 14 presents the extraction as *"sufficient to reopen a partial facade independently"*. Two sufficient conditions are not rival readings of one trigger, so the earlier entry's framing — that one wording must govern — was malformed. The blocker the record itself names is the subject and the fixture, not the extraction: candidate D1 is *"eliminated at this base solely by the missing neutral subject/second independent fixture, not by the carriers"*. The acceptance provenance agrees and states the condition once: *"The recorded reopening trigger is one second independently authored backend fixture sharing the same neutral, non-self-certifying structural and execution subjects as the portfolio; that evidence alone reopens the partial-facade question."*

  That condition is satisfied. `crates/tiler-conformance/tests/independent_backend/` carries both subjects — `the_assembled_artifact_carries_facts_this_backend_never_supplied` and `the_routed_result_agrees_with_the_reference_oracle` — and the comparator the condition names, `spikes/runtime/backend-provider-portfolio`, carries both itself: it assembles through `assemble_plan_artifact` and routes through `route_with_adapter` against `tiler-reference`.

  **The earlier entry's second ground does not reach this trigger.** Its manifest fact is true and re-verified at `77cd0104`: `crates/tiler-build/Cargo.toml` declares six tiler dependencies — artifact, cache, compiler, ir, metal, metal-aot — and neither `tiler-runtime` nor `tiler-reference`, so no test target under `crates/tiler-build/` can route a plan or reach the oracle. The inference drawn from it does not follow, because no wording of the condition names `custom_backend` or requires both fixtures to sit inside `make full`. The comparator every wording names is the portfolio. A reading that disqualifies the retained spike would leave zero fixtures and make the condition unsatisfiable by construction, and the decision record counts that spike as the first fixture throughout.

  **What firing does and does not authorize.** It reopens the partial-facade question for re-presentation to Tom. It authorizes no public export, no `pub mod`, and no new crate; the accepted deferral's stop boundary is unchanged. This entry does not move this ticket's `status`, which is the coordinator's call.

  Reproduce (run at the repaired tree; every number is a count of matching **lines**, not occurrences):

  ```sh
  D=tickets/decide-the-backend-provider-conformance-harness-public-surface.md
  P=crates/tiler-conformance/tests/independent_backend
  S=spikes/runtime/backend-provider-portfolio
  echo "operative condition stated:    $(grep -c 'that evidence alone reopens the partial-facade question' $D)"
  echo "Trigger 1 lead, sufficiency:   $(grep -c 'sufficient on its own' $D)"
  echo "D1 elimination reason:         $(grep -c 'missing neutral subject/second independent fixture' $D)"
  echo "trigger restatements (census): $(grep -c 'independently authored' $D)"
  echo "fixture structural subject:    $(grep -c 'fn the_assembled_artifact_carries_facts_this_backend_never_supplied' $P/main.rs)"
  echo "fixture execution subject:     $(grep -c 'fn the_routed_result_agrees_with_the_reference_oracle' $P/main.rs)"
  echo "portfolio structural seam:     $(grep -rl 'assemble_plan_artifact' $S/src | wc -l | tr -d ' ')"
  echo "portfolio execution seam:      $(grep -rl 'route_with_adapter' $S/src | wc -l | tr -d ' ')"
  echo "tiler-build runtime/reference: $(grep -A 25 '^\[dependencies\]' crates/tiler-build/Cargo.toml | grep -c 'tiler-runtime\|tiler-reference')"
  ```

  Actual output when run at this commit:

  ```text
  operative condition stated:    2
  Trigger 1 lead, sufficiency:   2
  D1 elimination reason:         2
  trigger restatements (census): 10
  fixture structural subject:    1
  fixture execution subject:     1
  portfolio structural seam:     2
  portfolio execution seam:      3
  tiler-build runtime/reference: 0
  ```

  The first four counts are **2** and the census is **10** because the dated correction on the decision record quotes the retired wording, as this repository's convention requires. Before the repair they were 1, 1, 1, and 5. A count that *shrank* here would mean the correction had deleted what it was supposed to preserve, so a falling number is the failure signal, not progress. The last line is the negative control that carries the manifest fact: it must stay **0**, and it would become non-zero the moment `crates/tiler-build` gained either edge — which would not change this verdict, since the manifest fact was never load-bearing for it. What this block cannot do is read for meaning: it confirms the sentences are present and the subjects exist, and the judgement that the condition is satisfied rests on the reading recorded above.
