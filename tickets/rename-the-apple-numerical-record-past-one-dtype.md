---
id: rename-the-apple-numerical-record-past-one-dtype
title: Rename the Apple numerical record past one dtype
status: todo
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype]
scopes: [research/apple-targets, contracts/navigation, contracts/decisions, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, apple-targets]
---
`docs/research/apple-targets/numerical-behaviour.md` is titled "Apple GPU f32 numerical behaviour" and, since its findings 21 to 23, measures `f32` and `f16`. The title now names the record's origin rather than its extent.

The rename is a separate ticket only because of where the string lives. `widen-the-apple-numerical-probe-to-a-second-dtype` declared `research/apple-targets`, and the title is reproduced in generated catalog blocks in `docs/research/README.md`, `docs/decisions/README.md`, and `spikes/README.md` (`contracts/navigation`) and in prose citations in `docs/decisions/0076-declare-target-honourable-numerical-realizations.md` (`contracts/decisions`), `docs/backends/metal.md` (`contracts/artifacts`), and `docs/integration/candle.md` (`contracts/integrations`). Renaming from inside that ticket would have been a scope escape, and leaving the generated blocks stale would have failed the renderer. The record carries an explicit note saying so, which this ticket removes.

**What closes this.** The frontmatter `title` and the `#` heading name what the record measures without naming one dtype; every catalog block that quotes it has been updated by hand, since nothing regenerates them; the three prose citations read correctly; and the record's "this record's title is stale" note is gone rather than reworded.
