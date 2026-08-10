---
id: derive-the-metal-fenced-space-population
title: Derive the Metal fenced-space population
status: done
priority: p2
dependencies: []
related: [derive-the-artifact-numerical-and-fenced-space-populations]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Fact audit — split from the artifact population repair on 2026-08-08

**Fact.** `crates/tiler-metal/src/synchronization_requirement_tests.rs`, at `const FENCES: [FencedSpaces; 4]`, hand-enumerates the product of the two current boolean fields. `POPULATION` derives through `FENCES.len()`, so a third field can leave both the enumeration and the claimed 648-value population short.

**False source claim.** The adjacent comment, at `A third flag would leave this list at four entries`, says the population assertion would then fail. The assertion derives from the same short list and remains 648; it cannot detect the widened type.

**Boundary.** This is an independent Metal test population, not an artifact encoder or identity change. [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md) owns its artifact copy and must not absorb this one.

## What closes this

- Derive the field count from one exhaustive `FencedSpaces` destructure and size the enumeration as the boolean product, following the private IR/artifact pattern without sharing test support across crates.
- Add a third boolean field as a temporary subject perturbation. Repair constructors and the field census while intentionally leaving the enumeration short; require the Metal population mechanism itself to report an eight-versus-four array size, quote the diagnostic, and restore.
- Correct the false comment and keep the 648 current count only as a consequence of the derived population.
- Change no production field, synchronization encoding, Metal behavior, public surface, identity, or artifact file.

## Outcome — implemented on the ticket branch, 2026-08-10

At base `703c8ea344d8e520487104904342c99bf18f2ef7`, both stated facts were
verified: `const FENCES: [FencedSpaces; 4]` was a hand-maintained product and
`POPULATION` multiplied `FENCES.len()`, while the adjacent comment falsely
claimed that the population assertion would notice a third field. The related
artifact ticket and the private IR census use the same deliberate crate-local
shape; the Metal copy remains private to this test module.

Commit `7666475c9ac930ad7cb1fd18e572447d443a47ac` adds a private
`bool_field_count` const helper and one exhaustive `FencedSpaces` destructure.
The bool-typed array from that destructure derives `FENCED_SPACE_FIELD_COUNT`,
and `FENCES` now has length `1 << FENCED_SPACE_FIELD_COUNT`. The current four
fences still make the subject domain 648 values, but only as a consequence of
the type-derived product. The replacement comment correctly says that, after a
third field is added to the census, the four-entry list itself fails the
derived eight-entry declaration.

### Subject perturbation

A temporary production `FencedSpaces::constant: bool` field was added. Its
constructors, `FencedSpaces::NONE`, the decoder construction required to build
the Metal test dependency closure, and this Metal field census were repaired;
the Metal enumeration deliberately remained four entries. `cargo check -p
tiler-metal --tests` then reported at the `FENCES` declaration:

```text
expected an array with a size of 8, found one with a size of 4
```

All perturbation edits were restored. The delivered diff changes neither the
production type nor synchronization encoding or behavior, artifacts, identity,
or public surface.

### Verification

- `cargo nextest run -p tiler-metal` — 131 passed.
- `cargo test -p tiler-metal --doc` — 9 passed, including four compile-fail tests.
- `cargo check -p tiler-metal --all-targets`
- `cargo clippy -p tiler-metal --all-targets -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-metal --no-deps`
- `cargo fmt --all -- --check`
- `make check`
- `make citations` — 1,189 pinned citations and 6,440 local links resolved.
- `tkt lint --format json`, `git diff --check`, and `tkt guard tkt/derive-the-metal-fenced-space-population --format json`

The type-derived product covers only independent boolean fields. A non-boolean
field is intentionally rejected at the bool-typed census rather than being
assigned a speculative cardinality.
