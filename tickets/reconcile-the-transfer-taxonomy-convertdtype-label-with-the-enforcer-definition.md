---
id: reconcile-the-transfer-taxonomy-convertdtype-label-with-the-enforcer-definition
title: Reconcile the transfer taxonomy ConvertDtype label with the enforcer definition
status: done
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

## Outcome

**They do not resolve the same way, and the split is derived from the accepted definition rather than chosen.** `RepackEncoding` stays a `PlacementEnforcer` variant; `ConvertDtype` leaves the family and is listed as an excluded neighbour. Neither of the ticket's two options was adopted wholesale: no second umbrella term was added, and the two stages were not excluded together.

**Why `RepackEncoding` needed no correction.** `docs/compiler/optimizer.md` admits storage encoding to the boundary-property list, states that "its enforcer is repacking", and cites *this memo's own* separation of `MaterializeLayout` from `RepackEncoding` as the reason the enforcer was accepted before the property was named. Encoding passes the admission test dtype fails: a producer can realize one semantic value packed or unpacked and the choice is unobservable in the value. Its table row now says so explicitly — "explicitly changes storage encoding; the represented values are unchanged" — because "changes encoding" alone reads like the neighbouring "changes represented values" and that resemblance is what made one word cover both.

**Why `ConvertDtype`'s membership was structural and not only a label.** `PlacementEnforcer` is a sum a planner selects from. A conversion inside it says a planner may introduce a conversion to satisfy a boundary, which ADR 0010 forbids and which the optimizer contract restates as "a conversion the graph does not contain may not be introduced by a schedule at all". Removing it from the sum removes that reading; renaming the word would not have.

**Why no second umbrella term.** The alternative was a family such as `ValueProducingStage` with `ConvertDtype` as its member. Rejected: `docs/numerical-semantics.md#casts` makes a cast a semantic operation carrying a resolved typed conversion contract, and its realization is ordinary lowering of an operation the graph already contains — so a second family here would be a second authority for something ADR 0010 owns, and naming it beside the enforcer family would reintroduce the very reading the split removes. It is still *named*, as an excluded neighbour with its own table, because verifier invariant 3 has to check against it and cannot check against a stage the taxonomy does not name.

**What landed** in `docs/research/transfers/transfer-synchronization-and-resource-lifetime.md`: the Outcome paragraph no longer calls conversion an enforcer and says why; `ConvertDtype` moved out of the `PlacementEnforcer` sum into an `ExcludedNeighbour` alternative with a comment saying what it is; the taxonomy table split into a value-preserving enforcer table and a one-row excluded-neighbour table; verifier invariant 3 now names the two stages separately and says which is not an enforcer; and a new section derives the whole split from the quoted definition, labelled Fact and Inference throughout.

**A sibling the same check surfaced, and did not settle.** All nine taxonomy-table rows and all eight `PlacementEnforcer` variants were read against the definition, not just the one the ticket named. The seven rows other than `RepackEncoding` and `ConvertDtype` are value-preserving by the mechanism itself, as is `RepackEncoding`. `Recompute` is the one variant the taxonomy table never described, and the definition does not settle it: it does not move an authoritative version, it re-derives one, so it is value-preserving only if the recomputation is proved to produce the same values under the effective numerical contract — the same ADR 0001 argument the enforcer definition rests on applies to two plans differing only in whether a value was recomputed. ADR 0047's acceptance of recomputation as an enforcer is preserved and not reopened; what is recorded is that the obligation making it legitimate is unstated. `qualify-recompute-value-preservation-in-the-transfer-taxonomy` owns it.

**Reach, with the exact check.** `grep -rn ConvertDtype docs spikes crates` returns this memo and `docs/compiler/optimizer.md`, and nothing else. The optimizer contract's sentence — that the taxonomy "keeps both distinct from `ConvertDtype`" — stays exactly true under the split, so no document outside `research/transfers` needed an edit. The transfers spike does not spell the name at all.
