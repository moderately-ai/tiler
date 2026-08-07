---
id: stop-copying-the-carried-payload-through-the-builder-assemble
title: Stop copying the carried payload through the builder's assemble
status: review
priority: p2
dependencies: []
related: [stop-copying-the-carried-payload-through-the-envelope-projection, measure-artifact-decoder-allocation-amplification]
scopes: [implementation/artifact]
shared_scopes: [research/artifacts, project/tickets]
paths: []
tags: [artifact, codec, performance]
claimed_from: todo
assignee: agent-payload-build
lease_expires_at: 1786073447
---
`ArtifactProgramBuilder::build` copies every carried object once more, on the
same publication path
[`stop-copying-the-carried-payload-through-the-envelope-projection`](stop-copying-the-carried-payload-through-the-envelope-projection.md)
reduced to one.

`build(self)` calls `assemble(&self, declared)`, which writes
`payload_content: self.payload_content.clone()` into the `ArtifactProgramData`
it returns (`crates/tiler-artifact/src/program/builder.rs`). The borrow is
forced by the method's own contract: `build` returns the **intact builder**
inside `ArtifactVerificationError` when verification fails, so nothing may be
moved out of it before the diagnostics are known — and the diagnostics are
derived from the assembled data.

So a producer that builds and then encodes one artifact carrying an `n`-byte
object holds `n` in the builder, `n` in the artifact data, `n` in the projected
section table, and `n + manifest` in the encoder's output buffer. The projection
ticket took the third of those from four copies down to one; this is the second,
and it is untouched because it sits in a different function under a different
constraint.

## Why this is a design question rather than a mechanical fix

Three shapes are available and they are not equivalent:

1. `assemble(&mut self, ...)` taking the content with `std::mem::take`. Cheapest,
   and it silently guts the builder the error path promises to return intact — a
   caller that recovered from a failed `build` would find its payloads gone.
2. Assemble by move and reconstruct the builder on the error path. The builder
   holds `subject`, `expression_types`, and interning state the data does not, so
   this is not a reconstruction that exists today.
3. Move the object bytes only, behind a type that states the builder is spent
   for payloads. A public-boundary change on `ArtifactVerificationError`'s
   recoverability contract.

Tom owns the third; the first is a correctness regression stated as a
performance win.

## Measure it first

The retained spike rows do not cover this: the harness builds its fixture
*outside* the measured window, so
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md)
reports the encode and not the build. A `build` phase has to be added to the
harness before the size of this is a measurement rather than a reading of the
source.

## Closes when

The publication path's build step is measured, the copy is either removed under
a contract that stays true or the retention is recorded with its reason, and
`make full` passes.

## Outcome — 2026-08-06

Done, under shape 2. `ArtifactProgramBuilder::build` now **lends** the draft's
tables to the `ArtifactProgramData` it verifies and takes them back before the
failure path boxes the builder, so the carried objects are held once rather than
twice and the recoverability contract is unchanged — no public surface moved.
Peak live during `build` of a 64 MiB object fell from **2.00×** the envelope to
**1.00×**.

### Measured first, because the harness could not see this

`spikes/artifacts/decoder-allocation` gained a `build` phase. `build` consumes
its builder, so unlike every other phase there the measured call cannot be
repeated over one value and assembling a draft inside the window would charge
the row for declaring the carried object. So `EnvelopeFactory::draft` was split
out of `artifact` (which is now `draft` then `build`, so the two cannot describe
different artifacts), exactly `REPETITIONS` drafts are assembled outside every
window, and each call pops one. Popping allocates nothing.

Two runs, both from this branch, taken on either side of the crate change:

```sh
cd spikes/artifacts/decoder-allocation
cargo build --release
./target/release/artifact-decoder-allocation --record macos-27.0-2026-08-06-build-before
# ... crate change ...
cargo build --release
./target/release/artifact-decoder-allocation --record macos-27.0-2026-08-06-build-after
```

Peak live during `ArtifactProgramBuilder::build`:

| Object bytes | Envelope | Before | ×env | After | ×env | Requested, before → after | Calls |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 49,448 | 204,599 | 4.14 | 149,628 | 3.03 | 319,791 → 264,820 | 353 → 243 |
| 1 MiB | 1,098,024 | 2,301,751 | 2.10 | 1,198,204 | 1.09 | 2,416,943 → 1,313,396 | 355 → 244 |
| 16 MiB | 16,826,664 | 33,759,031 | 2.01 | 16,926,844 | 1.01 | 33,874,223 → 17,042,036 | 355 → 244 |
| 64 MiB | 67,158,312 | 134,422,327 | 2.00 | **67,258,492** | **1.00** | 134,537,519 → 67,373,684 | 355 → 244 |
| 4,000-node chain | 89,533 | 651,681 | 7.28 | 464,607 | 5.19 | 1,676,650 → 1,489,576 | 40,426 → 40,315 |

`largest_blocks` at 64 MiB read `67108864 67108864 47786 39699` and now reads
`67108864 47786 39699 23893` — the object was live twice inside `build`, as the
data's copy and as the envelope section the identity derivation projects, and is
live once. The remaining one is the floor
`stop-copying-the-carried-payload-through-the-envelope-projection` established.

The object-free rows fall by a constant 54,971 peak bytes and 110 calls at zero
arena nodes, because the payload table is not the only thing that stopped being
copied: the expression arena, the variant table with its whole kernel program,
the payload descriptors and the selected providers are lent with it.

No wall clock is reported. The metric is bytes and allocator calls, which are
properties of the program rather than of the machine — the harness measures every
call twice after a warm-up and asserts the two readings identical, and both runs
passed that. This host was loaded (coordination host, concurrent agent builds),
which is exactly why no timing appears.

**Not comparable to the four earlier retained runs.** `09d1666a` (`Declare the
manifest's artifact identity by digest, and choose the coverage fold`) shrank
every envelope this sweep produces — 64,635 bytes at zero arena nodes, more with
depth — between the `projection` run and this branch's base. The two runs above
are on one tree and are compared only to each other. Recorded in the spike README
as the fifth and sixth retained files, with that caveat stated there too.

### The decision, derived rather than picked

Shape 1 (`std::mem::take` and leave the builder gutted) is the correctness
regression this ticket already named; not taken.

Shape 2 was said to require reconstructing a builder that holds `subject`,
`expression_types` and interning state the data does not. Reading `build` shows
that framing is stronger than the situation needs: the reconstruction is only
required if the builder is rebuilt *from* the data. It is not. `build` takes
`self` by value, and the `data` it assembled is still owned and unconsumed on
every failure path — `verify_artifact`, `packaged_entry_positions`,
`ArtifactEnvelope::project` and `encode_identity` all borrow it, and it is moved
only into the `Ok` arm's `VerifiedArtifactProgram`. So the builder can simply
**lend** its tables and take the same allocations back:

- `assemble(&mut self, ...)` `std::mem::take`s `providers`, `payloads`,
  `payload_content`, `expressions`, `expression_types` and `variants`;
- `reclaim(&mut self, data)` destructures `ArtifactProgramData` exhaustively and
  puts all six back, immediately before the builder is boxed.

Nothing observes the builder in between: `build` consumes it, and no method it
calls afterwards reads `self`. The fields the ticket named — the interning map,
`expression_phases`, `expression_interface_only`, `subject`,
`delivery_positions` — are never touched, which is precisely why restoring the
six is sufficient rather than approximate. Nothing about
`ArtifactVerificationError` moved, so shape 3 was not needed and remains Tom's if
it is ever wanted for a different reason.

Three fields are still copied, each with its reason recorded at the site:
`semantic` is four identity digests with no empty value to leave behind;
`inputs` and `outputs` are projections of `self.subject`, which also carries the
numerical realization and target profile the data does not, so returning them
would mean rebuilding a `PortfolioSubject` rather than restoring one; and
`realization` is remapped from the producer's declared entry space into the
canonical one *inside* `build`, so the value the data ends with is deliberately
not the value the builder must keep.

### Artifact identity did not move

Three independent checks, because this is a pure producer-memory change.

- **The workspace suite's pinned identities.** `cargo nextest run --workspace`
  and `cargo test --workspace --doc` are green, and the artifact identity pins,
  goldens, and `tiler-macros` AOT fixtures they carry are unchanged.
- **All 102 spike data rows.** The `build-after` run differs from `build-before`
  in exactly the 9 `build` rows, one per shape, and every row of both reports the
  same `envelope_bytes`.
- **The harness's own refusal.** The `build` phase builds one further draft per
  shape *outside* the window and compares its whole envelope byte for byte
  against the bytes every other row of that shape measures, so a `draft` that had
  drifted from `artifact` fails the run rather than being measured.

### Tests

`a_recovered_builder_rebuilds_the_artifact_byte_for_byte`, in
`crates/tiler-artifact/src/program/codec/tests.rs`. One draft carrying an object,
complete but for its provider selection, is cloned; one clone is completed and
built, the other is built (refused with `MissingSelectedProvider`), recovered
through `into_parts`, then completed and built. The two envelopes must be equal
byte for byte. The assertion is the whole envelope rather than the presence of a
payload because a table left behind would not merely lose bytes — the corrected
build would package a different artifact and say so with a different identity.

The crate had **no** test covering a recovery that carries anything: the one
existing recovery case,
`rejects_an_artifact_that_selected_no_provider`, recovers a draft holding a
descriptor-only payload and an empty arena.

Watched failing under two perturbations of `reclaim`:

- dropping the `payload_content` restore — both this test and
  `rejects_an_artifact_that_selected_no_provider` panic at
  `codec/model.rs:1351` with `index out of bounds: the len is 0 but the index is
  0`, because `payloads` and `payload_content` are index-aligned;
- restoring `payload_content` with each `code` cleared, which is the subtler
  "returned but not intact" failure — the byte assertion fires, and it is
  spelled with `assert!` plus both lengths rather than `assert_eq!` so a real
  failure reports sizes instead of printing two whole envelopes.

### Checks

```sh
cargo fmt --all -- --check
cargo clippy -p tiler-artifact --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-artifact
cargo nextest run -p tiler-artifact          # 247 passed, 1 skipped
make full
tkt lint
git diff --check
tkt guard tkt/stop-copying-the-carried-payload-through-the-builder-assemble --format json
```

`make full` green on the delivered tree: workspace nextest 2,893 passed and 7
skipped, all doctests, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace`, the
release numerical run at 1,003 passed and 3 skipped, `ticketsplease lint`, and
shellcheck.

The spike is a separate workspace and no `make` target reaches it, so its own
`cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings` were
run from `spikes/artifacts/decoder-allocation`.

### Files

- `crates/tiler-artifact/src/program/builder.rs` — `build`, `assemble`, `reclaim`
- `crates/tiler-artifact/src/program/codec/tests.rs` — the recovery case
- `spikes/artifacts/decoder-allocation/harness/src/main.rs` — the `build` phase, `REPETITIONS`, its three refusals
- `spikes/artifacts/decoder-allocation/harness/src/envelope.rs` — `draft` split out of `artifact`
- `spikes/artifacts/decoder-allocation/results/decoder-allocation-macos-27.0-2026-08-06-build-before.tsv` — new retained run
- `spikes/artifacts/decoder-allocation/results/decoder-allocation-macos-27.0-2026-08-06-build-after.tsv` — new retained run
- `spikes/artifacts/decoder-allocation/README.md` — six retained files, build table, the consuming-phase method, two boundary notes
- `spikes/artifacts/decoder-allocation/Cargo.lock` — stale since `tiler-digest` was split out; the spike does not build without the refresh
- `docs/research/artifacts/decoder-allocation-amplification.md` — Section 10, headline, status
