---
id: repair-the-subprogram-resources-first-stage-numerical-doc
title: Repair the subprogram-resources first-stage numerical doc
status: todo
priority: p3
dependencies: []
related: [admit-an-explicit-non-arithmetic-region-and-delivery-state, admit-the-partitioned-copy-scheduled-region]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, maintainability]
---
## Outcome

`subprogram_resources` documents the check its code performs, so a reader is not told a value is inherited where it is in fact refused on disagreement.

## Fact — the doc predates the arm it now sits beside, at `41a018fbc7e33e9a573a63a61264f49e5f41717a`

In `crates/tiler-compiler/src/frontier.rs`, `subprogram_resources` still opens by explaining that the numerical dimensions are taken from the first stage because every stage of an admitted subprogram implements the same request contract, which `verify_schedule_with_feasibility` proved for each separately. Its two neighbouring paragraphs — on synchronization and on index arithmetic — instead explain why those are *refused* on disagreement rather than peaked, and each names the reason: an atomic subject and a capability have no defined maximum.

The numerical requirement joined that second group when the `RegionNumericalRequirements` sum landed. The code now carries `if peak.numerical != stage.numerical {` and returns `None`, with an inline comment giving the same reasoning as its two neighbours.

So the value still comes from stage zero and the sentence is not false — but it explains the choice by an argument made elsewhere, at a call site that now also checks it locally, and it reads as though no local check exists. A reader auditing whether a copy-armed stage could inherit an arithmetic stage's requirement is sent to `verify_schedule_with_feasibility` rather than to the guard three lines below.

## Required work

Move the numerical paragraph into the shape its two neighbours use: state that the requirement is a classed sum with no defined maximum, that disagreement is refused rather than inherited, and that the refusal is unreachable today because every stage of an admitted subprogram implements one request contract. Keep the inline comment or the doc paragraph, not two statements of the same fact.

This is a comment repair. No behaviour changes, and the check the doc describes already exists.

## Closes when

The doc states the refusal its code performs, the three requirement paragraphs explain their choices in one consistent shape, and no second statement of the same reasoning is left behind.
