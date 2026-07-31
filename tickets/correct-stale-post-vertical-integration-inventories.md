---
id: correct-stale-post-vertical-integration-inventories
title: Correct stale post-vertical integration inventories
status: todo
priority: p1
dependencies: []
related: [prototype-metal-aot-slice, prototype-inline-aot-integration-proof, prototype-candle-metal-adapter, correct-the-stale-post-vertical-implementation-status]
scopes: [contracts/integrations, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, integration, inventory, correctness]
---
## User-visible outcome

Integration contracts, Metal AOT source documentation, and still-open inline/Candle ticket briefs use the current inventory of eleven production crates plus two prototype workspace members and the current delivered component boundaries, so future work starts from the code that exists rather than obsolete crate counts and private-identity assumptions.

## Why this is a correctness ticket

- **Fact:** `docs/integration/candle.md` retains an obsolete six-crate inventory, `crates/tiler-metal-aot/src/family.rs` retains an explanation falsified by the implemented compilation-identity boundary, and the inline AOT and Candle adapter tickets retain obsolete private-status or eight-crate premises. The correct documentation path is singular `docs/integration/`, not `docs/integrations/`.
- **Fact (corrected 2026-07-31):** the workspace has **eleven** production crates plus two prototype members. This ticket was written saying "nine production crates plus two prototype members and still has no frontend or Candle crate", which was true when filed and is now stale in one direction only: `admit-the-tiler-facade-and-proc-macro-crate-boundary` (`done`) admitted `tiler` and `tiler-macros` on 2026-07-31, so **a frontend facade now exists** — carrying only the `tensor!` re-export and its generated-path anchor, with no grammar and no runtime types — while **there is still no Candle crate**. The completed Metal producer, cache, artifact, and bounded runtime proof do not satisfy the open inline or Candle outcomes, and neither does the facade's existence.
- **Inference:** these inventories affect dependency direction and work scope; a worker following them can place integration ownership in the wrong crate or rebuild a delivered identity seam.

## Implementation keys

- Read each edited integration contract, source file, and ticket in full. Derive the workspace inventory from Cargo metadata and the identity ownership from construction sites.
- Correct only current premises; preserve historical measurements and ticket history as history.
- Keep component implementation separate from complete integration: Metal AOT, cache, artifacts, and the bounded runtime proof exist, while proc-macro composition, family delivery, embedding, and Candle adaptation remain open.
- Treat any corrected public source documentation as a public module boundary and present it to Tom before acceptance.
- Prove new inventory and absence checks can fail, then run targeted `tiler-metal-aot` tests and Clippy, `tkt lint`, local documentation checks, and one batch `make full`.
- Derive and assert the exact package-name population from `cargo metadata --no-deps`: eleven production crates and the two prototype packages. Remove one expected name once and observe the count/name check fail before restoration.
- Correct current source and ticket claims that Metal AOT compilation identity or the compiler boundary remain private: `promote-the-metal-aot-compilation-identity` and `prototype-public-compiler-api` are done, and public `PreparedCompilation::identity()` is consumed by `tiler-build`. Preserve the still-missing frontend/Candle crates and incomplete family-delivery/integration work.

## Closes when

All named current inventories agree with Cargo metadata and the delivered construction sites; open ticket outcomes no longer rely on private or missing component assumptions; no completed component is promoted into a complete inline or Candle path; any crate-root public rustdoc correction is reviewed by Tom, while private `family.rs` prose alone is not mislabeled as a public module boundary; and the targeted and full gates pass.

## What was corrected (2026-07-31, base `01363ef`)

**Measurement — the derived package population.** `cargo metadata --no-deps` on this branch reports thirteen packages: eleven production crates (`tiler`, `tiler-artifact`, `tiler-build`, `tiler-cache`, `tiler-compiler`, `tiler-ir`, `tiler-macros`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime`) and two prototype members (`tiler-prototype-compile`, `tiler-prototype-run`). Every count below is that derivation, not a recount of the prior text. `ls crates/` holds no `tiler-candle`, and `grep -rn candle --include=Cargo.toml --include=Cargo.lock .` reports no match.

Each correction, with the completed ticket that made the prior premise stale:

- `docs/integration/candle.md` — "six `crates/*/Cargo.toml`" to eleven, with the absence claim converted into the one-line `grep` a reader can rerun rather than an enumeration they must trust; `ticketsplease.toml:122` to `:124` (the line moved); and one added sentence recording that `tiler`/`tiler-macros` now exist while carrying no grammar, no runtime types, and no Candle path, so a reader does not read the contract's frontend-macro prose as delivered. Stale by `admit-the-tiler-facade-and-proc-macro-crate-boundary` and by intervening scope-map edits. The Candle absences the section rests on — no manifest dependency, no adapter, no admitted crate — were re-verified and stand.
- `crates/tiler-metal-aot/src/family.rs` — the `#[allow(dead_code)]` reason claimed the frontend proc-macro crate "does not exist" and that `prototype-public-compiler-api`'s closing condition was Tom's pending acceptance of a public boundary. Both false: `prototype-public-compiler-api` and `promote-the-metal-aot-compilation-identity` are `done`, `CompilationIdentity` is public, and `tiler-build` consumes it through `PreparedCompilation::identity` at `crates/tiler-build/src/metal_assembly.rs:119`. The `allow` itself is still correct and stays — `ArtifactFamilySelection` has no non-test caller anywhere in `crates/` — so the reason now says what is actually absent: the caller, not the crate. `promote-artifact-family-selection-for-the-frontend` and `prototype-inline-proc-macro-frontend` are named as its owners.
- `tickets/prototype-inline-aot-integration-proof.md` — the "Dependency reality" section recorded `promote-the-metal-aot-compilation-identity` as `in-progress` on unmerged commit `4f8ce90` and concluded the cache-sharing half had no reachable identity producer because `CompilationIdentity`/`as_bytes` were `pub(crate)`. Corrected to the delivered state, with the 2026-07-28 reading preserved as superseded history rather than deleted.
- `tickets/prototype-candle-metal-adapter.md` — "`ls crates/` returns eight crates" to the derived thirteen-package population, and `ticketsplease.toml:122` to `:124`. The admission this ticket owns is unchanged: none of the three crates admitted since is a Candle path.

**Nothing here required Tom.** The only source edit is a `reason` string inside `mod family`, which is crate-private with every item `pub(crate)`, so it is not a public module boundary. `crates/tiler-metal-aot/src/lib.rs` and the public `identity` module doc were both read in full for the same class of staleness and found accurate — `identity.rs`'s "Public boundary and construction authority" section already describes the promoted state — so no crate-root public rustdoc correction was made, and none is pending review.

**The population check is filed, not absorbed.** `check-the-workspace-package-population` owns placing it beside `crates/tiler/tests/dependency_direction.rs`, whose scope is `implementation/frontend` and outside this ticket's. That ticket carries the derived population, the two `workspace_members` ID forms a parse must handle, and the failure-proof requirement. **Evidence the check discriminates**, run here against the equivalent derivation: unperturbed it reports 13 expected / 13 derived and passes; dropping `tiler-macros` from the expected list reports 12 expected / 13 derived and fails on both the count and the name comparison; restoring it passes again.

## Graph maintenance

- Link each correction to the completed ticket that made the prior premise stale.
- Split newly discovered drift outside the declared scopes rather than broadening this ticket silently.
- `docs/architecture.md`'s "nine reusable libraries" and `docs/status.md`'s absence block are the same drift in a scope this ticket does not hold; `record-the-frontend-crate-admission-in-the-design-corpus` already owns both and was deliberately not absorbed. `docs/integration/frontends.md` was left untouched for the same reason — that ticket holds `contracts/integrations` and owns Tom's two decisions about its examples.
- Close this ticket when the integration contracts, source documentation, and live briefs agree.
