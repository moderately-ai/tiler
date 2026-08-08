---
id: derive-the-payload-carrying-enum-populations-in-the-injectivity-module
title: Derive the payload-carrying enum populations in the injectivity module
status: done
priority: p2
dependencies: []
related: [derive-the-artifact-numerical-and-fenced-space-populations]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Source-first Fact audit at base `812a45e3`

Every verdict below was re-read from the named source before implementation. Anchors are copied from source rather than from rendered prose.

| Ticket claim | Verdict | Source and evidence |
| --- | --- | --- |
| The two numerical populations have hand-written lengths | **Verified.** | `crates/tiler-ir/src/exhaustive_injectivity.rs`, at `pub(crate) const SUBNORMAL_MODES: [SubnormalMode; 3]` and `pub(crate) const EXCEPTIONAL_ASSUMPTIONS: [ExceptionalValueAssumption; 4]`. `PERMISSIONS` beside them is sized by `variant_count`. |
| The module says every plain-enum array is type-sized | **Verified but incomplete.** | The header anchor is `every array over a plain enum is sized by`. That statement literally excludes payload-carrying enums, but the later anchor `neither list can silently stop covering the vocabulary` incorrectly relies on it for the numerical lists copied across the artifact boundary. |
| The schedule and kernel length assertions guard those populations | **False: all four assertions are tautologies.** | `crates/tiler-ir/src/schedule/model.rs` and `crates/tiler-ir/src/kernel/model.rs` each contain `assert_eq!(SUBNORMAL_MODES.len(), 3);` and `assert_eq!(EXCEPTIONAL_ASSUMPTIONS.len(), 4);`. An array declared `[T; 3]` or `[T; 4]` has that length by type, regardless of whether it still covers its domain. |
| A direct `variant_count` substitution is correct | **False.** | `crates/tiler-ir/src/schedule/numerics.rs` defines one unit `SubnormalMode` inhabitant plus one `FlushToZero` inhabitant per `FlushedZeroSign`, and one unit `ExceptionalValueAssumption` inhabitant plus one `AssumeAbsent` inhabitant per `ValueDomainProvenance`. Their current populations are therefore `1 + variant_count::<FlushedZeroSign>()` and `1 + variant_count::<ValueDomainProvenance>()`, not the outer enums' two variants. Those expressions alone would still miss a new outer variant, so the implementation must derive the sum from an exhaustive outer-arm census. |
| `BEHAVIOUR_POPULATION` is independently safe | **False.** | `crates/tiler-ir/src/numerics/tests.rs`, at `const BEHAVIOUR_POPULATION: usize = SUBNORMAL_MODES.len()`, inherits both short arrays. The preceding `all_behaviours` comment says all five arrays are `variant_count`-sized, which is false for these two. |
| `FENCES` is exhaustive because a third field makes `SUBJECT_POPULATION` fail | **False.** | `crates/tiler-ir/src/exhaustive_injectivity.rs`, at `A third flag would leave this list`, promises that failure, but `SUBJECT_POPULATION` multiplies `FENCES.len()`. A third `FencedSpaces` field leaves both the enumeration and the derived count at four, so the 648 assertion stays green after construction and encoder sites are repaired. |
| The artifact copy is safe under the same reasoning | **False and out of this implementation scope.** | `crates/tiler-artifact/src/program/codec/tests.rs`, at `Both flush behaviours are enumerated`, hand-lists both payload populations without a type-derived size. `crates/tiler-artifact/src/program/tests.rs` declares `const FENCED_SPACES: [FencedSpaces; 4]` and derives 649 through its `.len()`. Follow-up is [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md). |

The omitted `FENCES` defect is the same private file, invariant, and `implementation/ir` scope as the numerical defects, so repairing the ticket to include it does not change the implementation boundary. The artifact copies stay separate.

## What closes this

- Define each payload-carrying population from one exhaustive outer-arm census whose contribution expression is also the term summed into the declared array length: `1` for the current unit arm and `variant_count` of the named payload for the current payload arm. A new outer arm must first fail the census and then, once assigned a contribution, grow the required array length.
- Replace the misleading header and `all_behaviours` prose with the actual guarantee for fieldless enums, payload-carrying enums, and struct products.
- Tie `FENCES` to an exhaustive, bool-typed field census of `FencedSpaces`, then derive the two-inhabitant-per-field product from that census. A new field must fail at this population mechanism, not only at a constructor or encoder.
- Perturb `FlushedZeroSign`, `ValueDomainProvenance`, both payload-carrying outer enums, and `FencedSpaces` independently. Repair unrelated exhaustive encoder/constructor errors in each temporary tree far enough to show a population-specific diagnostic, quote it, then restore.
- Change no production vocabulary, encoder tag, identity byte, domain, or public surface.

## Outcome — implemented on the ticket branch, 2026-08-08

The private `exhaustive_enum_population!` invocation for each payload-carrying enum is one compile-time source for both outer shape and summed inhabitant population. Its exhaustive match requires every outer arm to name a contribution, while those exact expressions form the array length: `1` for each current unit arm and `variant_count` of the current payload vocabulary for each payload arm. The arrays retain their deterministic order and current values. `BEHAVIOUR_POPULATION` continues to derive from those arrays and its false five-`variant_count` explanation now distinguishes the three fieldless spaces from the two exhaustive arm sums.

`FENCED_SPACE_FIELD_COUNT` exhaustively destructures `FencedSpaces::NONE` and passes the named fields through one const-generic bool array. The array both type-checks the census and supplies its length to `1 << FENCED_SPACE_FIELD_COUNT`; extending the census therefore changes the required product automatically. This is private test support. No production vocabulary, constructor, encoder, tag, identity byte, versioned domain, schema, or public item changed.

The artifact work was not absorbed. [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md) owns `implementation/artifact`, depends on this repair, and relates back symmetrically. `tkt why` reports `dependency_ordered: true`; there is no reverse dependency and therefore no cycle. The stale terminal claims in [`size-the-four-hand-written-metal-all-arrays-from-their-types`](size-the-four-hand-written-metal-all-arrays-from-their-types.md) and [`prove-the-exhaustible-encoder-injectivity-claims-natively`](prove-the-exhaustible-encoder-injectivity-claims-natively.md) carry dated corrections rather than silently retaining their false conclusions.

### Subject perturbations

Each temporary change was applied independently, all unrelated exhaustive matches or struct literals that would otherwise fail were satisfied, and `cargo check -p tiler-ir --all-targets` was run before restoring it.

- Add `FlushedZeroSign::NegativeZero` and teach all five in-crate exhaustive encoders/readers a temporary distinct case: `error[E0308]: mismatched types` at `SUBNORMAL_MODES`, `expected an array with a size of 4, found one with a size of 3`. This was the only error.
- Add `ValueDomainProvenance::ExternalAttestation` and teach all four in-crate exhaustive encoders/readers a temporary distinct case: `error[E0308]: mismatched types` at `EXCEPTIONAL_ASSUMPTIONS`, `expected an array with a size of 5, found one with a size of 4`. This was the only error.
- Add fieldless `SubnormalMode::TreatAsZero` and teach all five ordinary in-crate exhaustive encoders/readers a temporary distinct case: `error[E0004]: non-exhaustive patterns: schedule::numerics::SubnormalMode::TreatAsZero not covered` at the macro-generated census match. After naming its contribution as `1`, the diagnostic moved to `SUBNORMAL_MODES`: `expected an array with a size of 4, found one with a size of 3`. Each phase had only the quoted error.
- Add fieldless `ExceptionalValueAssumption::AssumePresent` and teach all ordinary in-crate exhaustive encoders/readers and expectations a temporary distinct case: `error[E0004]: non-exhaustive patterns: schedule::numerics::ExceptionalValueAssumption::AssumePresent not covered` at the macro-generated census match. After naming its contribution as `1`, the diagnostic moved to `EXCEPTIONAL_ASSUMPTIONS`: `expected an array with a size of 5, found one with a size of 4`. Each phase had only the quoted error.
- Add a third boolean `FencedSpaces::constant` field, update `NONE`, `is_empty`, every in-crate struct literal, and the census array itself, while intentionally leaving `FENCES` short: `error[E0308]: mismatched types` at `FENCES`, `expected an array with a size of 8, found one with a size of 4`. This was the only error.

All five perturbations were restored. The source-scoped command `rg -n 'NegativeZero|ExternalAttestation|TreatAsZero|AssumePresent|constant: false|pub constant' crates/tiler-ir/src` returned no matches; ticket quotations are outside that search population.

### Verification

- `cargo fmt --all -- --check`
- `cargo check -p tiler-ir --all-targets`
- `cargo nextest run -p tiler-ir` — 991 passed, 0 skipped
- `cargo test -p tiler-ir --doc` — 17 passed, 1 ignored across the two doctest runs
- `cargo clippy -p tiler-ir --all-targets -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-ir --no-deps`
- `tkt lint --format json`
- `make citations` — 934 pinned citations and 6,257 local links resolved
- `git diff --check`
