---
id: disambiguate-presentation-label-from-semantic-key-accessors
title: Disambiguate presentation-label accessors from semantic-key accessors
status: in-progress
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening, identity]
claimed_from: todo
assignee: agent-disambiguate-presentation-label-from-semantic-key-accessors
lease_expires_at: 1784915749
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
