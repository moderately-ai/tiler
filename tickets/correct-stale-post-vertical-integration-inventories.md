---
id: correct-stale-post-vertical-integration-inventories
title: Correct stale post-vertical integration inventories
status: in-progress
priority: p1
dependencies: []
related: [prototype-metal-aot-slice, prototype-inline-aot-integration-proof, prototype-candle-metal-adapter, correct-the-stale-post-vertical-implementation-status]
scopes: [contracts/integrations, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, integration, inventory, correctness]
claimed_from: todo
assignee: loop-correct-stale
lease_expires_at: 1785519401
---
## User-visible outcome

Integration contracts, Metal AOT source documentation, and still-open inline/Candle ticket briefs use the current inventory of nine production crates plus two prototype workspace members and the current delivered component boundaries, so future work starts from the code that exists rather than obsolete crate counts and private-identity assumptions.

## Why this is a correctness ticket

- **Fact:** `docs/integration/candle.md` retains an obsolete six-crate inventory, `crates/tiler-metal-aot/src/family.rs` retains an explanation falsified by the implemented compilation-identity boundary, and the inline AOT and Candle adapter tickets retain obsolete private-status or eight-crate premises. The correct documentation path is singular `docs/integration/`, not `docs/integrations/`.
- **Fact:** the current workspace has nine production crates plus two prototype members and still has no frontend or Candle crate. The completed Metal producer, cache, artifact, and bounded runtime proof do not satisfy the open inline or Candle outcomes.
- **Inference:** these inventories affect dependency direction and work scope; a worker following them can place integration ownership in the wrong crate or rebuild a delivered identity seam.

## Implementation keys

- Read each edited integration contract, source file, and ticket in full. Derive the workspace inventory from Cargo metadata and the identity ownership from construction sites.
- Correct only current premises; preserve historical measurements and ticket history as history.
- Keep component implementation separate from complete integration: Metal AOT, cache, artifacts, and the bounded runtime proof exist, while proc-macro composition, family delivery, embedding, and Candle adaptation remain open.
- Treat any corrected public source documentation as a public module boundary and present it to Tom before acceptance.
- Prove new inventory and absence checks can fail, then run targeted `tiler-metal-aot` tests and Clippy, `tkt lint`, local documentation checks, and one batch `make full`.
- Derive and assert the exact package-name population from `cargo metadata --no-deps`: nine production crates and the two prototype packages. Remove one expected name once and observe the count/name check fail before restoration.
- Correct current source and ticket claims that Metal AOT compilation identity or the compiler boundary remain private: `promote-the-metal-aot-compilation-identity` and `prototype-public-compiler-api` are done, and public `PreparedCompilation::identity()` is consumed by `tiler-build`. Preserve the still-missing frontend/Candle crates and incomplete family-delivery/integration work.

## Closes when

All named current inventories agree with Cargo metadata and the delivered construction sites; open ticket outcomes no longer rely on private or missing component assumptions; no completed component is promoted into a complete inline or Candle path; any crate-root public rustdoc correction is reviewed by Tom, while private `family.rs` prose alone is not mislabeled as a public module boundary; and the targeted and full gates pass.

## Graph maintenance

- Link each correction to the completed ticket that made the prior premise stale.
- Split newly discovered drift outside the declared scopes rather than broadening this ticket silently.
- Close this ticket when the integration contracts, source documentation, and live briefs agree.
