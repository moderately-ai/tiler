---
id: derive-the-artifact-numerical-and-fenced-space-populations
title: Derive the artifact numerical and fenced-space populations
status: todo
priority: p2
dependencies: [derive-the-payload-carrying-enum-populations-in-the-injectivity-module]
related: [derive-the-payload-carrying-enum-populations-in-the-injectivity-module]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The artifact repeats all three under-sized populations

**Fact.** `crates/tiler-artifact/src/program/codec/tests.rs`, in `every_governed_tag_table_round_trips`, hand-enumerates the three `SubnormalMode` inhabitants after the anchor `Both flush behaviours are enumerated` and the four `ExceptionalValueAssumption` inhabitants immediately after the permission loop. Neither list has a type-derived size. A new `FlushedZeroSign` or `ValueDomainProvenance` can therefore leave the claimed round trip short after the artifact's exhaustive tag tables are updated.

**Fact.** `crates/tiler-artifact/src/program/tests.rs` declares `const FENCED_SPACES: [FencedSpaces; 4]`. The test `the_artifact_synchronization_encoding_is_injective_over_its_whole_domain` computes `POPULATION` through `FENCED_SPACES.len()`, so a new `FencedSpaces` field leaves both the enumeration and its claimed 649-value population short.

**Fact — stale terminal claim to repair.** [`prove-the-exhaustible-encoder-injectivity-claims-natively`](prove-the-exhaustible-encoder-injectivity-claims-natively.md), at `the seven artifact tag tables`, calls the artifact enumerations complete and concludes they need nothing. The left-inverse argument is valid only over a complete enumeration; the two payload lists do not establish completeness.

**Fact — stale terminal claim to preserve as corrected history.** [`size-the-four-hand-written-metal-all-arrays-from-their-types`](size-the-four-hand-written-metal-all-arrays-from-their-types.md), at `exhaustive_injectivity.rs already argues its own exclusions`, calls the IR payload enumerations consistent. Their values are complete today, but their hand-written lengths are not coupled to the payload vocabularies. The dependent IR ticket repairs that mechanism; this ticket must not re-edit the IR copy.

**Fact — pending claim this ticket supersedes for the artifact half.** [`derive-the-payload-carrying-enum-populations-in-the-injectivity-module`](derive-the-payload-carrying-enum-populations-in-the-injectivity-module.md) originally omitted the artifact numerical and fence copies. That ticket now records the split and this dependency, so artifact and IR work cannot be scheduled as though either alone closes the cross-crate claim.

## What closes this

- Give each artifact payload-carrying population its own private exhaustive outer-arm census whose contribution expressions also form the declared array length: `1` for the current unit arm and `variant_count` of the named payload for the current payload arm. Make the round-trip test consume those arrays. Do not expose or share the IR test helper: the crate-boundary duplication is deliberate and each copy must fail independently.
- Give the artifact's `FENCED_SPACES` copy its own exhaustive destructure whose named fields pass through one const-generic bool array, with that array's length deriving the product. Updating the sole census for a new field must automatically grow the required product.
- Perturb `FlushedZeroSign`, `ValueDomainProvenance`, a fieldless outer variant of each payload-carrying enum, and a third `FencedSpaces` field independently. Repair unrelated matches and constructors far enough to show the artifact population mechanism's own diagnostic; for the fence case, repair the census too while intentionally leaving the enumeration short. Quote each diagnostic and restore.
- Amend the stale terminal claims above with dated corrections and record the exact verification commands.
- Change no production vocabulary, codec tag, identity byte, artifact domain, schema, or public surface.

The dependency on the IR ticket is sequencing, not scope expansion: it lets this copy follow the established private pattern while keeping `implementation/artifact` conflict-free from the IR edit. The related edges are symmetric and non-blocking for discovery; only this ticket depends on the IR repair, so the graph has no cycle.
