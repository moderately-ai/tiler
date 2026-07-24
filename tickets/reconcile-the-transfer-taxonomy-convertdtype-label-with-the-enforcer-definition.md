---
id: reconcile-the-transfer-taxonomy-convertdtype-label-with-the-enforcer-definition
title: Reconcile the transfer taxonomy ConvertDtype label with the enforcer definition
status: todo
priority: p3
dependencies: []
related: [reconcile-dtype-cast-enforcer-with-boundary-properties, transfer-synchronization-and-resource-lifetime-contract]
scopes: [research/transfers]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, transfers, numerics]
---
`docs/compiler/optimizer.md` now states that an enforcer may change only how a boundary value is stored, addressed, placed, or delivered, never which values it carries, and that a dtype cast is consequently not an enforcer (`reconcile-dtype-cast-enforcer-with-boundary-properties`).

`docs/research/transfers/transfer-synchronization-and-resource-lifetime.md` uses the word differently. Its Outcome says "Dtype conversion and encoding-changing repacking remain separate enforcers", and its "Enforcer and mechanism taxonomy" table lists `ConvertDtype` with value/encoding effect "explicitly changes represented values/dtype". The doc's intent is sound and is not in dispute: verifier invariant 3 requires that "encoding or dtype changes use their separately typed enforcers", which exists precisely to stop a transfer folding a conversion into a copy, and ADR 0047 agrees that "Transfer does not silently convert encoding". Only the label is in conflict — the table applies one word to value-preserving movement stages and to a value-changing conversion stage.

That document is explicitly a proposal that no accepted ADR and no normative contract has incorporated, so this is a terminology reconciliation ahead of incorporation rather than a correction to an accepted contract. Decide whether the taxonomy needs a second term for a value-changing program stage, or whether `ConvertDtype` and `RepackEncoding` should be presented as excluded neighbours of the enforcer family rather than members of it. Note that `RepackEncoding` is value-preserving and `ConvertDtype` is not, so they may not resolve the same way.
