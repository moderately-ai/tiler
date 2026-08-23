---
id: repair-the-retired-schedule-builder-path-in-the-partitioned-copy-delivery-record
title: Repair the retired schedule-builder path in the partitioned-copy delivery record
status: todo
priority: p3
dependencies: []
related: [admit-the-partitioned-copy-scheduled-region, admit-an-explicit-non-arithmetic-region-and-delivery-state]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, citations]
---
## Outcome

The delivery record of [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md) cites paths that resolve, so a reader checking it does not conclude that landed content was removed.

## Fact — the cited path is retired, at `41a018fbc7e33e9a573a63a61264f49e5f41717a`

That record's Delivery section names `crates/tiler-ir/src/schedule/builder.rs` twice — once for the builder changes and once as the location of its evidence. No such file exists at this base. The builder is the directory `crates/tiler-ir/src/schedule/builder/`, holding `mod.rs`, `copy.rs`, `coverage.rs`, `diagnostics.rs`, `family.rs`, `intrinsic.rs`, `proof.rs`, `reduction.rs`, `tile.rs`, `contraction.rs`, `elementwise.rs`, `tests.rs`, and two more test modules. `assemble` is in `builder/mod.rs`.

This is the module-split false-absence trap `AGENTS.md` records: the citation fails as *absence* rather than as a rotted line number, so a reader greps the named path, gets nothing, and may conclude the delivery did not happen. `make citations` cannot catch it — the check resolves markdown links, and these are inline code spans naming source paths, not links.

## Second, smaller item in the same pass

`subprogram_resources` in `crates/tiler-compiler/src/frontier.rs` still documents its numerical handling as taken from the first stage because every stage implements the same request contract, while the code now refuses on disagreement (anchor `if peak.numerical != stage.numerical {`). The value is still stage zero's, so the sentence is imprecise rather than false, but it reads as if no local check exists when one was added beside it. Repair the prose to match.

## Closes when

Every source path cited in that delivery record resolves at the tip; the `subprogram_resources` doc states the refusal its code performs; and neither repair changes any accepted claim.
