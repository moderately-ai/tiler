---
id: close-the-fmt-blind-spot-over-the-trybuild-facade-fixtures
title: Close the fmt blind spot over the trybuild facade fixtures
status: done
priority: p3
dependencies: []
related: []
scopes: [implementation/workspace, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Why this exists

**Fact, found by `correct-the-four-thread-grid-rationales-the-measured-row-falsified` on 2026-08-05.** `crates/tiler/tests/facade/pass/*.rs` are `trybuild` fixtures rather than cargo targets, so `cargo fmt --all --check` never reaches them: `deliver_compiles_embeds_and_routes.rs:161` carries an over-100-column line that `rustfmt --edition 2024` flags and the gate never sees. Every facade fixture shares the blind spot.

## What this ticket owes

A gate step that formats-checks the fixture population (an explicit `rustfmt --check` over the facade glob in the Makefile, or an equivalent that names its population), the one pre-existing violation fixed in the same change so the new check starts green, and the check watched failing once against a deliberately misformatted fixture. Compile-fail fixtures whose byte-compared goldens depend on exact source spans must be excluded deliberately and the exclusion recorded — a reformat there moves caret columns and breaks goldens, which is worse than the blind spot.

## Closes when

The gate reaches every facade fixture the goldens permit, the exclusion set is recorded with its reason, and the check was watched failing.
