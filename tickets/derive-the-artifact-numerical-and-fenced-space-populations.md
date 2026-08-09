---
id: derive-the-artifact-numerical-and-fenced-space-populations
title: Derive the artifact numerical and fenced-space populations
status: todo
priority: p2
dependencies: [derive-the-payload-carrying-enum-populations-in-the-injectivity-module]
related: [derive-the-payload-carrying-enum-populations-in-the-injectivity-module, pin-the-admitted-unsafe-sites-in-the-workspace-gate, derive-the-metal-fenced-space-population]
scopes: [implementation/artifact, implementation/frontend, implementation/workspace, contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The artifact repeats all three under-sized populations

**Fact.** `crates/tiler-artifact/src/program/codec/tests.rs`, in `every_governed_tag_table_round_trips`, hand-enumerates the three `SubnormalMode` inhabitants after the anchor `Both flush behaviours are enumerated` and the four `ExceptionalValueAssumption` inhabitants immediately after the permission loop. Neither list has a type-derived size. A new `FlushedZeroSign` or `ValueDomainProvenance` can therefore leave the claimed round trip short after the artifact's exhaustive tag tables are updated.

**Fact.** `crates/tiler-artifact/src/program/tests.rs` declares `const FENCED_SPACES: [FencedSpaces; 4]`. The test `the_artifact_synchronization_encoding_is_injective_over_its_whole_domain` computes `POPULATION` through `FENCED_SPACES.len()`, so a new `FencedSpaces` field leaves both the enumeration and its claimed 649-value population short.

**Fact — the terminal correction already landed.** [`prove-the-exhaustible-encoder-injectivity-claims-natively`](prove-the-exhaustible-encoder-injectivity-claims-natively.md), at the source-safe anchor `The left-inverse argument is valid, but two enumerations`, already retracts completeness for the two payload lists and points here. Preserve that dated correction; this ticket owes no further edit to the terminal record.

**Fact — the IR correction already landed.** [`size-the-four-hand-written-metal-all-arrays-from-their-types`](size-the-four-hand-written-metal-all-arrays-from-their-types.md), at the source-safe anchor `` was right, but `exhaustive_injectivity.rs` was not consistent ``, already preserves the historical exclusion and retracts the false IR-consistency conclusion. The dependency repaired the IR mechanism and is `done`; this ticket must not re-edit either terminal record or the IR copy.

**Fact — completed dependency and established pattern.** [`derive-the-payload-carrying-enum-populations-in-the-injectivity-module`](derive-the-payload-carrying-enum-populations-in-the-injectivity-module.md) records the split, landed the private IR pattern, and is done. This ticket owns only the artifact copies.

**Fact — one artifact crate-root claim is also stale.** `crates/tiler-artifact/src/lib.rs`, at `sizes this crate's exhaustive-injectivity enumerations`, describes only fieldless enums. The payload-arm census and the struct-field product use different type-derived mechanisms and must be named there when they land.

**Fact — integration adds one governed source generator.** The established pattern is a private, test-only `macro_rules! exhaustive_enum_population` in the artifact codec test module, invoked twice in that same file. The workspace unsafe inventory currently pins sixteen private local producers; this exact path/name makes seventeen. The producer emits only const population declarations and adds no unsafe, attribute, source-load, nested-macro, export, public, or identity authority. The inventory pin, root manifest and architecture count, ADR 0079's current-population correction, and the completed unsafe ticket's dated integration note must move together while preserving their earlier counts as history.

**Fact — a separate Metal copy remains.** `crates/tiler-metal/src/synchronization_requirement_tests.rs` declares `const FENCES: [FencedSpaces; 4]`, derives `POPULATION` through `FENCES.len()`, and falsely says a third flag would make that assertion fail. [`derive-the-metal-fenced-space-population`](derive-the-metal-fenced-space-population.md) owns that independent `implementation/metal` repair. This ticket must not absorb it or claim workspace-wide closure.

## What closes this

- Give each artifact payload-carrying population its own private exhaustive outer-arm census whose contribution expressions also form the declared array length: `1` for the current unit arm and `variant_count` of the named payload for the current payload arm. Make the round-trip test consume those arrays. Do not expose or share the IR test helper: the crate-boundary duplication is deliberate and each copy must fail independently.
- Give the artifact's `FENCED_SPACES` copy its own exhaustive destructure whose named fields pass through one const-generic bool array, with that array's length deriving the product. Updating the sole census for a new field must automatically grow the required product.
- Perturb `FlushedZeroSign`, `ValueDomainProvenance`, a fieldless outer variant of each payload-carrying enum, and a third `FencedSpaces` field independently. Repair unrelated matches and constructors far enough to show the artifact population mechanism's own diagnostic; for the fence case, repair the census too while intentionally leaving the enumeration short. Quote each diagnostic and restore.
- Pin the exact new private macro producer in the workspace unsafe inventory and update its current sixteen-to-seventeen population statements. Rename the actual definition and both calls while the artifact still compiles; require the inventory to report one unpinned definition and both unsupported invocations, then restore.
- Correct the artifact crate-root explanation. Preserve the already-landed terminal corrections and leave the Metal residual to its related ticket.
- Change no production vocabulary, codec tag, identity byte, artifact domain, schema, or public surface.

The dependency on the IR ticket is sequencing, not scope expansion: it lets this copy follow the established private pattern while keeping `implementation/artifact` conflict-free from the IR edit. The related edges are symmetric and non-blocking for discovery; only this ticket depends on the IR repair, so the graph has no cycle.
