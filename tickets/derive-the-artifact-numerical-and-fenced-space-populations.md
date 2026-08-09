---
id: derive-the-artifact-numerical-and-fenced-space-populations
title: Derive the artifact numerical and fenced-space populations
status: in-progress
priority: p2
dependencies: [derive-the-payload-carrying-enum-populations-in-the-injectivity-module]
related: [derive-the-payload-carrying-enum-populations-in-the-injectivity-module, pin-the-admitted-unsafe-sites-in-the-workspace-gate, derive-the-metal-fenced-space-population]
scopes: [implementation/artifact, implementation/frontend, implementation/workspace, contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: sol-artifact-population
lease_expires_at: 1786245873
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

## Outcome — implemented on the ticket branch, 2026-08-08

The two artifact payload-carrying populations now derive their array lengths
from one private exhaustive outer-arm macro, invoked once per enum. The current
arm sums are `1 + variant_count::<FlushedZeroSign>() = 3` and
`1 + variant_count::<ValueDomainProvenance>() = 4`; the governed-tag round-trip
test consumes the resulting arrays. The artifact synchronization test derives
the two-field boolean product from one exhaustive, bool-typed
`FencedSpaces` destructure, preserving the current four fences and 649 optional
synchronization subjects without a hand-maintained field count.

This is test-only population evidence. No production enum, field, constructor,
encoder, tag, identity byte, artifact domain, schema, or public item changed.
The independent Metal copy remains owned by
[`derive-the-metal-fenced-space-population`](derive-the-metal-fenced-space-population.md).

The new private producer is pinned as
`("crates/tiler-artifact/src/program/codec/tests.rs",
"exhaustive_enum_population")`. The current workspace population is seventeen;
the earlier fifteen- and sixteen-producer statements remain explicitly
historical in ADR 0079 and the completed unsafe ticket.

### Subject perturbations

Each temporary production-vocabulary change was made independently, unrelated
exhaustive encoders and constructors were repaired far enough to reach this
ticket's mechanism, and the subject was then restored.

- Adding `FlushedZeroSign::NegativeZero` made `SUBNORMAL_MODES` report
  `expected an array with a size of 4, found one with a size of 3`.
- Adding `ValueDomainProvenance::ExternalAttestation` made
  `EXCEPTIONAL_ASSUMPTIONS` report
  `expected an array with a size of 5, found one with a size of 4`.
- Adding fieldless `SubnormalMode::TreatAsZero` first reported
  `non-exhaustive patterns: SubnormalMode::TreatAsZero not covered` at the
  macro-generated census match. After assigning that arm contribution `1`,
  `SUBNORMAL_MODES` reported
  `expected an array with a size of 4, found one with a size of 3`.
- Adding fieldless `ExceptionalValueAssumption::AssumePresent` first reported
  `non-exhaustive patterns: ExceptionalValueAssumption::AssumePresent not covered`
  at the macro-generated census match. After assigning that arm contribution
  `1`, `EXCEPTIONAL_ASSUMPTIONS` reported
  `expected an array with a size of 5, found one with a size of 4`.
- Adding a third boolean `FencedSpaces::constant` field, repairing ordinary
  construction sites, and adding it to the field census while leaving the
  enumeration short made `FENCED_SPACES` report
  `expected an array with a size of 8, found one with a size of 4`.
- Renaming the actual artifact macro and both calls to
  `exhaustive_enum_population_v2` left `cargo check -p tiler-artifact
  --all-targets` green. The complete 18-test unsafe inventory reported one
  ``unpinned macro_rules! definition `exhaustive_enum_population_v2` `` and two
  ``custom macro invocation `exhaustive_enum_population_v2!` is unsupported``
  diagnostics.

Every perturbation was restored before verification.

### Verification

- `cargo nextest run -p tiler-artifact` — 252 passed, 1 skipped.
- `cargo test -p tiler-artifact --doc` — 2 passed.
- `cargo nextest run -p tiler --test workspace_unsafe_sites` — all 18 passed;
  the live census remained 426 sources, 63 Cargo targets, thirteen doctest roots,
  sixteen packages, 73 fixture plus one rustdoc tensor invocation, and four
  admitted unsafe sites.
- `cargo check -p tiler-artifact --all-targets`
- `cargo clippy -p tiler-artifact --all-targets -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-artifact --no-deps`
- `cargo fmt --all -- --check`
- `tkt lint --format json`
- `make citations` — 926 pinned citations and 6,251 local links resolved.
- `git diff --check`
- Fresh `make full` — workspace check, Clippy, rustdoc, 3,248 nextest tests
  with eight skipped, every workspace doctest, 1,116 release tests with three
  skipped, ticket lint, citations, formatting, and shellcheck all passed.

The changed-file set contains no production identity encoder, codec golden,
schema/domain constant, or Metal file; no checked identity or golden moved.
