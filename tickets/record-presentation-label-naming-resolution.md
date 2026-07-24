---
id: record-presentation-label-naming-resolution
title: Close ADR 0074's presentation-label naming question
status: in-progress
priority: p2
dependencies: []
related: [disambiguate-presentation-label-from-semantic-key-accessors, draft-public-api-conventions-adr]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, api-hardening]
claimed_from: todo
assignee: agent-record-presentation-label-naming-resolution
lease_expires_at: 1784917695
---
Accepted ADR 0074 carries an open question — "Naming for presentation-only digest labels" — that says `key()` names both a presentation digest and a stable semantic key, offers `label()` and `display_id()` as candidates, and records that **no owner is assigned**. An owner was assigned and the work is done: `disambiguate-presentation-label-from-semantic-key-accessors` merged the rename. The ADR now points at an unsettled question whose answer already shipped, which is exactly the "stale status language" the documentation-as-contract rule forbids.

This ticket holds `contracts/decisions`, which the renaming ticket did not, and is the only reason the edits were deferred rather than made alongside the code.

Three edits, all in `docs/decisions/0074-use-explicit-public-api-conventions.md`:

- **The open question at line 196 is answered.** The settled spelling is `label()`. Replace the unsettled entry with the resolution rather than deleting it, so the reasoning survives: the ADR's own first candidate won, and it won because every one of the affected doc comments already used the word ("Returns a bounded explain label", "The label is a digest of the canonical bytes and is presentation only") — the accessor now matches the contract it documents. Record that the rename was verified not to move any label value, since every label is a `format!` over unmodified canonical bytes.
- **Convention 2's citation at line 62 is stale.** It cites `RegionContentIdentity::key()` as the surface whose doc comment states the presentation-only rule. That accessor is now `RegionContentIdentity::label()`. The quoted sentence is unchanged and still accurate.
- **The "Correction to the ticket's shorthand" at line 64 names a spelling that no longer exists in `tiler-compiler`.** Its substance is unaffected — the convention remains about the role of the value rather than the spelling of the accessor, and `tiler-ir` still spells borrowed semantic keys `key()`. Rewrite it to record that the collision existed, was real, and is now closed on the compiler side, rather than describing it as a live hazard.

One finding from the rename work is worth folding into the correction, because it is a stronger example than the `key()` case the ADR already gives. The same hazard was independently realized under a second spelling: three accessors returned the presentation digest as `stable_id`, while `pipeline::ProgramAlternative::stable_id` is an author-chosen `&'static str` that **is** compared as meaning — `select_alternative` decides the selected alternative with `alternative.stable_id == selected_alternative_id`. So one spelling named a digest label and a compared name inside a single crate, and the compared side was a selection decision. The three digest accessors are now `label`; `ProgramAlternative::stable_id` deliberately keeps its name because for it the spelling is correct. This is evidence that the convention needs to be stated as a rule about role, not propagated by imitation of whichever sibling a worker happened to read.

Do not touch the `b642007`-era evidence citation in `tickets/draft-public-api-conventions-adr.md`. It records what was verified at that exact base commit and is historical evidence, not a live contract.

Prose is not hard-wrapped. Run `uv run --locked python scripts/docs.py render` and the documentation gate before completion.
