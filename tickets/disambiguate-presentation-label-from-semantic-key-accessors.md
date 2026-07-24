---
id: disambiguate-presentation-label-from-semantic-key-accessors
title: Disambiguate presentation-label accessors from semantic-key accessors
status: done
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening, identity]
---
Proposed ADR 0074 records that the method name `key()` is overloaded across the
workspace with two genuinely different roles, and leaves the naming unsettled
because no owner existed. This ticket is that owner.

The two roles, both verified in source:

- **Presentation label.** In `tiler-compiler` (`region.rs`, `cover.rs`,
  `selection.rs`) `key()` returns an owned `String` digest of canonical identity
  bytes, explicitly documented as presentation-only and never an equality or
  dedup input.
- **Stable semantic key.** In `tiler-ir` (`index/scalar.rs`, `index/model.rs`,
  `semantic/registry.rs`, `semantic/operation.rs`, `semantic/interface.rs`)
  `key()` returns a borrowed key — `&ScalarOpKey`, `&OpKey`, `&OutputKey`,
  `&ValueTypeDefinitionKey` — which *is* meaning: it is compared and encoded into
  identity.

The hazard is that a future surface naming a digest label `key()` looks exactly
like a semantic-key accessor, and the convention that distinguishes them is about
role rather than spelling — so the name actively works against the rule. Nothing
is wrong today; this is a name that invites a future correctness mistake.

Rename the presentation-label accessors to something that cannot be mistaken for
meaning (`label()`, `display_label()`, or an equivalent the ADR's naming question
settles), keep the doc comment that states the presentation-only contract, and
leave the borrowed semantic-key accessors alone. Note that these labels are used
as explain subject keys, so the change touches explain records and their fixtures:
update them together and confirm the explain trace still identifies the same
subjects, since a subject-key change is observable in explain output.

If the rename is judged not worth its churn, record that decision and the reason
on ADR 0074's open question rather than closing this silently — the ADR must not
be left pointing at an unsettled question with no recorded resolution.

## Outcome

The rename was done. `label()` is the settled spelling for a presentation-only digest label in `tiler-compiler`; it is the ADR open question's own first candidate and the word every one of these doc comments already used ("Returns a bounded explain label", "The label is a digest of the canonical bytes and is presentation only"), so the accessor now matches the contract it documents.

**Fact — eight accessors renamed, all `pub(crate)` or private, in `crates/tiler-compiler`.** Five were the `key()` digests the ticket named: `RegionContentIdentity::label()` and `RegionOccurrenceIdentity::label()` (`region.rs`), `RegionCoverIdentity::label()` (`cover.rs`), and `SelectedPlanIdentity::label()` and `SelectedPortfolioIdentity::label()` (`selection.rs`). Each keeps its presentation-only doc comment; `SelectedPortfolioIdentity` had none and gained the sibling's exact two sentences.

**Fact — the same hazard was already realized under a second spelling, and it is now closed.** Three more accessors returned the same presentation digest under the name `stable_id`: `RegionCandidate::stable_id()` (`region.rs`), `CoverRegion::stable_id()` (`cover.rs`), and `FusionNumericalProof::candidate_stable_id()` (`fusion.rs`), each backed by a `String` field holding `RegionOccurrenceIdentity::label()`. Meanwhile `pipeline::ProgramAlternative::stable_id` is an author-chosen `&'static str` (`"alternative:fused-serial-sum.v1"`) that *is* compared as meaning. **Correction, applied after this ticket merged:** no function named `select_alternative` exists — `grep -rn "select_alternative" . --exclude-dir=.git --exclude-dir=target` matches only prose. The real chain is `select_structural_pareto` returning the selection decision, `PortfolioSelection::selected_alternative_id` carrying it, `verify_portfolio` deduping alternatives on it through a `BTreeSet` and rejecting with `portfolio-identity` on a collision, and `record_cost_and_selection` deciding the explain `SelectionOutcome` with `alternative.stable_id == selected_alternative_id` at `pipeline.rs:1522`. That makes the example sharper than recorded here: `stable_id` was an equality **and** a dedup input, the two roles convention 2 says a label never has. One spelling therefore named a digest label and a compared name inside one crate, which is exactly the collision ADR 0074 convention 2 is about, one step worse than the `key()` case because the compared side is a selection decision. The three digest accessors and their fields are now `label`; `ProgramAlternative::stable_id` and its `ExpectedAlternative` mirror deliberately keep `stable_id`.

**Fact — the semantic-key accessors in `tiler-compiler` were left alone, because their role is meaning.** `CapabilityAxis::key()` (`feasibility.rs`) is a governed predicate key that `selection.rs`'s `encode_guard` writes into `SelectedPlanIdentity` bytes and `pipeline.rs` wraps in `PredicateKey`; `ProfileIdentity::key()` is a governed profile key documented as participating in plan and artifact identity; `RegionBudgetResource::key()`, `CoverBudgetResource::key()`, and `PlanBudgetResource::key()` are governed `ResourceKey` vocabularies; and the explain `RuleKey`/`SubjectKey`/`FactKey` accessors are typed keys. Nothing in `tiler-ir` was touched.

**Measurement — explain subjects still identify the same things, byte for byte.** The labels are explain subject keys, and the rename reaches three explain sites: the candidate subject in `region::record_candidate`, the `region-content` fact value on the same record, and the `VerifiedEvidenceRef::from_fusion_numerical` candidate binding that `ExplainWriter::push_detail` checks for `EvidenceSubjectMismatch`. A temporary test compiled `pipeline::tests::semantic(false)` end to end through `compile()` and wrote `VerifiedExplainTrace::render()` to a file; the same test source was run once against base `6555119` and once against the renamed tree, both on `nightly-2026-07-19`, macOS arm64. The two 61-line traces are identical, SHA-256 `78e3e6060f66e428a0cc8c35b8e6b8bc24c8d098f2e11af809f72d0d65256889`, including all seventeen `candidate:region:<digest>` subject keys and the `region-content:<digest>` fact identity. **No expected fixture value changed anywhere**, and none could: every label is a `format!("region:{:016x}", …)`-style expression over unmodified canonical bytes, so a rename of the accessor cannot move a label value. The in-source expectations that pin label *values* — `region.rs`'s `starts_with("region-content:")` and the hand-written `"region:0000000000000000"` error-display strings in `region.rs`, `fusion.rs`, and `fusion_legality.rs` — are unchanged and still pass.

**Inference — this is a presentation change, not a semantic one.** No canonical encoder was touched, no identity byte sequence changed, and no equality, ordering, or dedup decision reads a renamed accessor. The one place a label participates in a comparison is `cover::verify_cover`'s anti-drift check `region.label != candidate.label()`, which runs *after* the authoritative lookup of `region.occurrence` in a `BTreeMap<RegionOccurrenceIdentity, …>`; it is a redundant consistency assertion over bytes-resolved identity, not an identity decision. `RegionOccurrenceIdentity::label()` therefore kept its original doc comment — which states the pairwise-distinctness contract rather than "equality decisions always use `as_bytes`" — because the stronger sentence would have overstated what that one call site does.

**Follow-up this ticket does not hold.** ADR 0074 is under `contracts/decisions`, which this ticket does not declare, so two edits remain for whoever holds it: the open question "Naming for presentation-only digest labels" is now answered (`label()`, no owner needed) and should record the resolution, and convention 2's citation of `RegionContentIdentity::key()` plus the "Correction to the ticket's shorthand" paragraph now name a spelling that no longer exists — the correction's *substance* is unaffected, since the convention is still about role rather than spelling. The `b642007`-era evidence citation in `tickets/draft-public-api-conventions-adr.md` was deliberately left as written: it records what was verified at that base commit.

Gate: `uv run --locked python scripts/check_repository.py` passed, `git diff --check` clean, `ticketsplease guard` reported no scope escape. Left `in-progress` for review; not merged, not pushed.
