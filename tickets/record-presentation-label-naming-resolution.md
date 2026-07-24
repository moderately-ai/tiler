---
id: record-presentation-label-naming-resolution
title: Close ADR 0074's presentation-label naming question
status: done
priority: p2
dependencies: []
related: [disambiguate-presentation-label-from-semantic-key-accessors, draft-public-api-conventions-adr]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, api-hardening]
---
Accepted ADR 0074 carries an open question — "Naming for presentation-only digest labels" — that says `key()` names both a presentation digest and a stable semantic key, offers `label()` and `display_id()` as candidates, and records that **no owner is assigned**. An owner was assigned and the work is done: `disambiguate-presentation-label-from-semantic-key-accessors` merged the rename. The ADR now points at an unsettled question whose answer already shipped, which is exactly the "stale status language" the documentation-as-contract rule forbids.

This ticket holds `contracts/decisions`, which the renaming ticket did not, and is the only reason the edits were deferred rather than made alongside the code.

Three edits, all in `docs/decisions/0074-use-explicit-public-api-conventions.md`:

- **The open question at line 196 is answered.** The settled spelling is `label()`. Replace the unsettled entry with the resolution rather than deleting it, so the reasoning survives: the ADR's own first candidate won, and it won because every one of the affected doc comments already used the word ("Returns a bounded explain label", "The label is a digest of the canonical bytes and is presentation only") — the accessor now matches the contract it documents. Record that the rename was verified not to move any label value, since every label is a `format!` over unmodified canonical bytes.
- **Convention 2's citation at line 62 is stale.** It cites `RegionContentIdentity::key()` as the surface whose doc comment states the presentation-only rule. That accessor is now `RegionContentIdentity::label()`. The quoted sentence is unchanged and still accurate.
- **The "Correction to the ticket's shorthand" at line 64 names a spelling that no longer exists in `tiler-compiler`.** Its substance is unaffected — the convention remains about the role of the value rather than the spelling of the accessor, and `tiler-ir` still spells borrowed semantic keys `key()`. Rewrite it to record that the collision existed, was real, and is now closed on the compiler side, rather than describing it as a live hazard.

One finding from the rename work is worth folding into the correction, because it is a stronger example than the `key()` case the ADR already gives. The same hazard was independently realized under a second spelling: three accessors returned the presentation digest as `stable_id`, while `pipeline::ProgramAlternative::stable_id` is an author-chosen `&'static str` that **is** compared as meaning. (This ticket's original text named a `select_alternative` function that does not exist; the real chain is `select_structural_pareto` → `PortfolioSelection::selected_alternative_id` → the `BTreeSet` dedup in `verify_portfolio` → the comparison in `record_cost_and_selection` at `pipeline.rs:1522`.) So one spelling named a digest label and a value that is both an equality and a dedup input, inside a single crate, with a selection decision on the compared side. The three digest accessors are now `label`; `ProgramAlternative::stable_id` deliberately keeps its name because for it the spelling is correct. This is evidence that the convention needs to be stated as a rule about role, not propagated by imitation of whichever sibling a worker happened to read.

Do not touch the `b642007`-era evidence citation in `tickets/draft-public-api-conventions-adr.md`. It records what was verified at that exact base commit and is historical evidence, not a live contract.

Prose is not hard-wrapped. Run `uv run --locked python scripts/docs.py render` and the documentation gate before completion.

## Outcome

All three named edits landed in `docs/decisions/0074-use-explicit-public-api-conventions.md`, plus two consistency edits the three required. The record no longer points at an unsettled question whose answer shipped.

**The open question is recorded resolved, not deleted.** The entry keeps its position in the list and now opens "Naming for presentation-only digest labels — resolved 2026-07-24 by `disambiguate-presentation-label-from-semantic-key-accessors`. The settled spelling is `label()`." It restates the question as it stood — the two roles, the `label()`/`display_id()` candidates, the fixture cost, the absent owner — then records that this record's own first candidate won on evidence the question did not yet have: every affected doc comment already used the word, so `label()` makes each accessor match the contract it was already documenting instead of introducing a third vocabulary. It records the measurement that the rename moved no label value and none could, because every label is a `format!` over unmodified canonical bytes.

**Convention 2's citation now names the landed accessor.** `RegionContentIdentity::label()`, verified at `crates/tiler-compiler/src/region.rs:181-185`; the quoted doc sentence is unchanged and still exact.

**The correction paragraph records a closed hazard rather than a live one, and keeps its point.** "The convention is about the role of the value, not the spelling of the accessor" is preserved verbatim as the rule. The `key()` overload is stated in the past tense for `tiler-compiler` and the present tense for `tiler-ir`, whose borrowed semantic keys are still spelled `key()` and were deliberately left alone.

**Fact — the `stable_id` evidence is folded in as a second realized instance, with the ticket's shorthand corrected.** The rename ticket's outcome, and this ticket's body inheriting it, attribute the comparison to a function named `select_alternative`. No such function exists: `grep -rn "select_alternative" . --exclude-dir=.git --exclude-dir=target` at `7171346` matches only those two prose files and no Rust source. Reading `crates/tiler-compiler/src/pipeline.rs` in full gives the accurate chain, which is what the ADR now records: `select_structural_pareto` decides and returns `Ok(selected.stable_id)`; `PortfolioSelection::selected_alternative_id` carries it; `verify_portfolio` dedups the alternatives on `stable_id` through a `BTreeSet` and rejects with `portfolio-identity` on collision, then rejects with `portfolio-selection` unless the recorded selection equals the recomputed one; and `record_cost_and_selection` holds the literal `alternative.stable_id == selected_alternative_id`, deciding the explain `SelectionOutcome::Selected`. This makes the example sharper than the ticket framed it: `stable_id` was an equality *and* a dedup input, the two roles convention 2 says a label never has, under the same spelling that named three presentation digests, inside one crate rather than across the crate boundary the `key()` case straddles. A closing inference states why this matters — a spelling is not a mechanism, so `label()` is a legibility gain and the rule remains the role.

**Two consistency edits the three required.** The "Open questions" preamble said "These are recorded unresolved on purpose. None is settled by this ADR", which a resolved entry would have contradicted; it now says none was settled by this ADR *itself* and that a question a later ticket answers is retained with its resolution and owner rather than deleted. In "Alternatives considered", "before either is settled" became "before either was settled", since that sentence is a counterfactual about the drafting moment and one of the two is now settled.

**Fact — this is not an amendment and is not recorded as one.** The record defines an amendment as changing a stated convention. No convention changed: convention 2's rule, its scope, the record's `applies_to`, and its authority are untouched, and only a citation, an example's tense, and the evidence behind it moved. The status line at the top, which names conventions 4 and 5 as the amended ones, is therefore correct as written and was left alone. The resolved open question says explicitly which text this ticket updated, so the trail is complete without claiming a rule moved.

**Fact — the other four open questions were each checked and none is stale.** Descriptor accessor style is owned by `unify-schedule-index-region-with-verified-index-region`, still `todo`. The `tiler.contract.optimizer` question triggers on the reviewed compiler facade landing; `crates/tiler-compiler/src/lib.rs` still exports only `pub mod capability` and `pub mod legality`, both documented as reviewed *drafts*, and `prototype-public-compiler-api` is `todo`. The registry `freeze` question triggers on one real caller wanting to retry a failed freeze; reading every `.freeze()` call site in `crates/` finds none — each either propagates, unwraps, or is infallible. Mechanized conformance checking is unowned; `prototype-optimizer-conformance-gate` is a different subject, the target-neutral optimizer conformance profile, not the API-convention checks.

**Fact — no other document carried a stale digest-accessor citation.** Searching `tickets/`, `docs/`, `spikes/`, and `prototypes/` for the five renamed `key()` spellings and the three renamed `stable_id` spellings leaves only `tickets/draft-public-api-conventions-adr.md:74`, the `b642007`-era evidence line, which was deliberately left as written. `docs/ir.md` already stated the presentation-only rule without naming any accessor.

**Follow-up filed.** `correct-presentation-label-rename-citation` (p3) owns replacing the `select_alternative` clause in the merged rename ticket's outcome with the accurate call chain. That outcome is a completed peer's evidence record, so it was not rewritten here.

Gate: `uv run --locked python scripts/docs.py render` passed (178 records, no generated block changed); `uv run --locked python scripts/check_repository.py` passed on the first run with no `os.killpg` flake; `git diff --check` clean; `ticketsplease lint` ok. Left `in-progress`; not merged, not pushed.
