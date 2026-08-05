---
id: correct-two-stale-delivery-spans-in-the-frontends-contract
title: Correct the two stale delivery spans in the frontends contract
status: in-progress
priority: p2
dependencies: []
related: [draft-an-adr-for-the-inline-delivery-statement, accept-the-inline-artifact-family-profile-syntax, first-authoritative-ios-metal-compile-declaration]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, frontend, inline-dx, apple-targets, staleness]
claimed_from: todo
assignee: agent-delivery-spans
lease_expires_at: 1785934519
---
## Why this exists

Found while drafting [ADR 0098](../docs/decisions/0098-state-an-inline-regions-delivery-policy-with-a-named-profile-or-a-family-list.md) under [`draft-an-adr-for-the-inline-delivery-statement`](draft-an-adr-for-the-inline-delivery-statement.md). That ticket held `contracts/decisions` and `contracts/navigation` and **not** `contracts/integrations`, which is what `docs/integration/**` maps to, so the defects were recorded rather than taken. Both are in [the frontend contract](../docs/integration/frontends.md), both are in its "The accepted spelling" subsection under Target policy, and both make the file contradict itself rather than merely age.

Neither defect touches the accepted spelling. The grammar, the two productions, and both vocabularies are correct as stated; what is wrong is a floor in an example and a paragraph about what an expansion does with a selected family.

## Defect 1 — the family-list example spells a refused floor

The subsection's second example reads `deliver macos 14.0, ios 17.0;`. **Those numbers are refused at this commit.** The profile Metal language standard is MSL 4.0, whose governed floor is 26.0 on both families, so `deliver macos 14.0;` is a typed refusal at the version token.

Reproduce against the byte-compared golden — `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.stderr` ends with:

> error: this artifact family and deployment minimum are not a governed Metal target: MSL 4.0 on macos requires deployment minimum 26.0, got 14.0

`DeliveredFamily::governed_minimum` in `crates/tiler-macros/src/delivery.rs` returns `DeploymentMinimum::new(26, 0)` for both families and says why in its own doc comment: "Under MSL 3.1 the same two rows read 14.0 and 17.0, so a standard change splits this arm rather than editing one number." 14.0 and 17.0 are the MSL 3.1-era numbers the deciding ticket was written against. `crates/tiler-macros/src/grammar.rs`'s module header already spells its example `deliver macos 26.0, ios 26.0;`, so the contract is the one document still carrying the old pair.

**Fix the numbers, and state the rule that stops them going stale again.** The floors are the driver's governed table rather than a frontend constant, so the contract should say a profile resolves to *the governed floor for the standard Tiler compiles with* and treat any spelled number as dated evidence about a standard. ADR 0098's decision 2 states it that way and is the wording to reuse rather than reinvent.

## Defect 2 — the refusal paragraph is contradicted by its own file

The paragraph beginning "**A statement selecting a family is refused today, and the refusal is the contract working.**" asserts that "No expansion runs the offline driver yet" and that "`deliver fallback-only;` and the statement's absence are consequently the only spellings an expansion completes."

**All three claims are false at this commit**, and the same file refutes them in three places: its status paragraph says "a selected buildable family is *delivered*, not refused"; its Landed list cites `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` for an expansion that "compiles, identifies, caches, embeds, and routes a one-entry bundle"; and its Parked list says `deliver ios;` and `deliver macos-and-ios;` are refused *because no iOS compile declaration exists*, which is a different and narrower reason than the one this paragraph gives.

The named mechanism is gone too: the paragraph's refusal was `DeliveryRefusal::BackendCompilationUnavailable`, and `grep -rn "BackendCompilationUnavailable" crates/` reports no match. `stated_delivery` in `crates/tiler-macros/src/delivery.rs` documents the removal — "It no longer refuses a selection that invokes the backend compiler."

**Replace it with what actually refuses**, which is four things at three layers and is worth stating as such: an iOS family for want of a measured compile declaration ([`first-authoritative-ios-metal-compile-declaration`](first-authoritative-ios-metal-compile-declaration.md) is the work), a symbolic-extent region under a selected family ([`carry-symbolic-extents-into-the-semantic-program`](carry-symbolic-extents-into-the-semantic-program.md) is the work), a deployment minimum below the governed floor, and every vocabulary and syntax mistake at the token responsible. The `deliver_selects_an_artifact_family` golden is the authority for the first three and is worth citing so the next reader can check the paragraph rather than trust it.

## Closes when

Both spans state what the tree does, checkable against the goldens named above; the family-list example spells a floor the driver accepts; the contract says the floors are the driver's rather than the frontend's; and no remaining sentence in the file contradicts its own status paragraph or its Landed and Parked lists on whether a selected macOS family delivers.

## Graph maintenance

If ADR 0098 has been accepted by the time this lands, the contract may cite it for the delivery statement the way it already cites ADR 0089 for the cache root; if it is still `proposed`, cite the deciding ticket instead and do not describe the record as accepted.
