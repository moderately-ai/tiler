---
id: stop-copying-the-carried-payload-through-the-envelope-projection
title: Stop copying the carried payload through the envelope projection
status: review
priority: p2
dependencies: []
related: [measure-artifact-decoder-allocation-amplification]
scopes: [implementation/artifact]
shared_scopes: [project/tickets, research/artifacts, contracts/navigation]
paths: []
tags: [artifact, codec, performance]
claimed_from: todo
assignee: agent-payload-copy
lease_expires_at: 1786051648
---
`VerifiedArtifactProgram::encode` peaks at **4.99x** the envelope it produces —
335,609,762 bytes for a 67,222,947-byte envelope carrying a 64 MiB object, and
403,253,928 bytes requested in total. The decoder of the same envelope now peaks
at 1.00x, so the producer side is the worse amplifier by a factor of five.

Measured in
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md),
recorded in
[the research note](../docs/research/artifacts/decoder-allocation-amplification.md).
`measure-artifact-decoder-allocation-amplification` was decoder-scoped and filed
this rather than widening itself.

## Where the copies are

The `largest_blocks` column of the encode rows names them: the final envelope,
and then the carried object **four times over**, all live at once. All four are
in `ArtifactEnvelope::project`'s path through `crates/tiler-artifact/src/program/codec/model.rs`:

1. `project_payloads` clones `data.payload_content[*payload]`, which carries the
   object, into `carried`.
2. `project_sections` clones `content.code` again into its `encoded` table.
3. `project_sections` clones it a third time pushing it into `contents`.
4. `project_sections` builds `index: BTreeMap<(u8, Vec<u8>), u32>` by cloning
   **every** `contents` entry as its key, and then clones the code a fifth time
   per `index[&(tag, code.clone())]` lookup.

Then the encoder writes it into the output buffer.

## What the floor is

One. `project` takes `&ArtifactProgramData`, so the payload content cannot be
moved out and must be copied at least once; the encoder's output buffer is the
envelope the caller asked for. So the reachable shape is roughly 2x the object,
against today's 5x live and 6x requested.

The cheap part is items 3-5: `contents` is sorted and deduplicated, so a
`binary_search_by` over borrowed `(tag, &[u8])` keys replaces the owned-key
`BTreeMap` outright and removes two whole copies plus a transient one per lookup.
Items 1-2 need the projection to thread one owned buffer instead of two.

## Why it is worth doing

This is the publication path. Every artifact `tiler-macros` embeds and every
bundle the expansion cache publishes pays it, and it scales with the compiled
object — which for a real `metallib` is the whole point of the envelope.

## Closes when

`project` copies each carried object at most once before the encoder writes it,
the spike's encode rows are re-run and recorded beside the retained ones with the
new ratio, no canonical order or section identity changes (the section table is
content-addressed and its dedup semantics must be preserved exactly, including
two payloads that carry equal objects sharing one section), and `make full`
passes.

## Outcome — 2026-08-06

Done. `ArtifactEnvelope::project` now holds **one** copy of each distinct carried
object, which is the floor its signature admits: it reads
`&ArtifactProgramData`, and a `Section` owns its bytes.
`VerifiedArtifactProgram::encode` of a 64 MiB object fell from **4.99x** the
envelope to **2.00x**.

### The copy census, old to new

Each row is one live copy of the carried object unless marked transient.

| Site | Was | Is |
| --- | --- | --- |
| `project_payloads` | cloned each `PayloadContent` to reorder the payload table | returns `Vec<Option<&PayloadContent>>`; reordering copies nothing |
| `project_sections` | cloned `content.code` into an `encoded` staging table | the staging table (`subjects`) holds compilation subjects only |
| `project_sections` | cloned it again pushing it into `contents` | pushes `Cow::Borrowed` |
| `project_sections` | cloned **every** `contents` entry into an owned `BTreeMap<(u8, Vec<u8>), u32>` key | `binary_search_by` over the sorted, deduplicated table with a borrowed key |
| `project_sections` | cloned it once more per `index[&(tag, code.clone())]` lookup — transient | the search key is borrowed |
| `Section::bytes` | the copy the encoder reads | unchanged: `Cow::into_owned` on the distinct survivors |

Two structural consequences worth naming. `ProjectedSections::programs` stopped
being a `BTreeMap<Vec<u8>, u32>` keyed by program identity bytes and became a
`Vec<u32>` aligned with the declared variant table, because the one caller walks
those variants in order — which also removed the `expect` on a map lookup that
could not fail. And `sort_unstable` + `dedup` are unchanged, so the
content-addressed dedup is preserved structurally rather than by argument.

### Measurement — the spike's encode rows

`spikes/artifacts/decoder-allocation`, run manually from its own directory:

```sh
cd spikes/artifacts/decoder-allocation
cargo build --release
./target/release/artifact-decoder-allocation --record macos-27.0-2026-08-06-projection
```

Recorded as
`results/decoder-allocation-macos-27.0-2026-08-06-projection.tsv`, beside the
three retained runs. Same host provenance as all of them: Apple M4 Max
(`Mac16,6`), 14 logical cores, macOS 27.0 (Darwin 27.0.0, build `26A5388g`),
`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, release profile.

Peak live during `VerifiedArtifactProgram::encode`:

| Object bytes | Envelope | Comparator | x env | Projection | x env |
| --- | --- | --- | --- | --- | --- |
| 0 | 114,083 | 340,479 | 2.98 | 340,479 | 2.98 |
| 1 MiB | 1,162,659 | 5,308,322 | 4.57 | 2,437,631 | 2.10 |
| 16 MiB | 16,891,299 | 83,951,522 | 4.97 | 33,894,911 | 2.01 |
| 64 MiB | 67,222,947 | 335,609,762 | 4.99 | **134,558,207** | **2.00** |

The 64 MiB row also fell 403,252,849 to 134,761,372 bytes requested and 237 to
213 allocator calls, and its `largest_blocks` went from
`67222947 67108864 67108864 67108864` to `67222947 67108864 111020 56320`. That
column records only the four largest requests, so before the change it was the
envelope and as many object copies as the array had room for; the peak and
requested totals are what count them — envelope plus four live and envelope plus
five requested, against one of each now.

The no-object row does not move because its peak is the identity derivation and
the manifest rather than a section; what the change removed there appears only
as 56,021 fewer bytes requested and 20 fewer calls. The same holds for every
arena-bearing encode row.

### Byte identity — the envelope did not move

Two independent checks, because this is a pure allocation change.

**Thirteen fixtures, digest-equal before and after.** A temporary probe encoded
`default`, `guarded`, `two_variant_artifact` in both declaration orders,
`partial_window`, `bf16_pointwise`, `f32_pointwise`, `strict_affine_u4`,
`requiring`, two `carried_artifact` shapes, and `twice_delivered_artifact` with
a shared object and with an empty one — 39,812 to 186,642 bytes each — and
digested the bytes with the codec's own governed digest. All thirteen digests and all thirteen lengths are identical
at `6cc4c242` and at the delivered commit. The probe was removed before commit;
its readings are the two runs recorded here.

**All 93 spike rows.** `diff` of the projection TSV against the comparator TSV
reports exactly the 9 encode rows and nothing else. Every row of both, encode
rows included, reports the same `envelope_bytes`.

### Tests

`two_payloads_carrying_equal_objects_share_one_section` is new, and so is the
`twice_delivered_artifact` fixture behind it: a one-variant artifact whose
single entry names two delivery positions, realized by two payloads with
different compilation subjects and one object. It requires two subject sections,
exactly one object section, both payloads' `PayloadSections::code` naming it,
and a clean round trip. The invariant the ticket names had no coverage before —
an envelope that framed one object twice would have grown by a whole compiled
library per delivery position and passed every other case in the suite.

Watched failing: with the projection's `contents.dedup()` removed the test
reports `left: 2, right: 1` at the object-section count.

### Checks

```sh
cargo fmt --all -- --check
cargo check -p tiler-artifact --all-targets
cargo clippy -p tiler-artifact --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-artifact
cargo nextest run -p tiler-artifact          # 251 passed, 2 skipped
make full
tkt lint                                     # ok: no problems found
git diff --check
tkt guard tkt/stop-copying-the-carried-payload-through-the-envelope-projection --format json
```

`make full` green on the delivered tree: workspace nextest 2,872 passed and 7
skipped, all doctests, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace`, the
release numerical run at 999 passed and 3 skipped, `ticketsplease lint`, and
shellcheck.

The only delta after that gate is this ticket's own frontmatter and the two
paragraphs above, which touch none of `crates/`, `prototypes/`, `Cargo.toml`,
`Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or
`deps.sh`, so the gate carries under the rule AGENTS.md states for exactly that
case. `tkt lint` was rerun on the final tree.

### Scopes added, with reasons

- `project/tickets` (shared) — this Outcome and the filed remainder.
- `research/artifacts` (shared) — the spike re-run, its README, and the research
  note the ticket asked to have the new rows recorded in. The codec worker added
  both for the same file set.
- `contracts/navigation` (shared) — `docs/research/README.md` is a navigation
  catalog under `ticketsplease.toml`, and the note's row there still read
  `pending` after `replace-the-codec-arena-content-key-with-the-existing-comparator`
  moved its `disposition` to `partially-adopted`. Leaving it would have made this
  ticket's Section 9 the second edit to a note whose catalog row disagrees with
  its own frontmatter. `tkt guard` reported the scope under-declared before it was
  added.

### What this does not reach, and where it went

`ArtifactProgramBuilder::build` copies the same objects once more, into the
`ArtifactProgramData` the artifact owns: `assemble(&self, ...)` clones
`self.payload_content` because `build` promises the **intact builder** back
inside `ArtifactVerificationError` and so cannot move out of it before the
diagnostics are known. That is a different function with a different reason, it
is outside this harness's measured window (the fixture is built before
`measure_twice` opens), and undoing it is a contract question rather than a
mechanical fix. Filed as
[`stop-copying-the-carried-payload-through-the-builder-assemble`](stop-copying-the-carried-payload-through-the-builder-assemble.md),
which has to add a `build` phase to the harness before it can claim a number.

### Files

- `crates/tiler-artifact/src/program/codec/model.rs` — the projection
- `crates/tiler-artifact/src/program/codec/tests.rs` — the dedup case and its fixture
- `spikes/artifacts/decoder-allocation/results/decoder-allocation-macos-27.0-2026-08-06-projection.tsv` — new retained run
- `spikes/artifacts/decoder-allocation/README.md` — four retained files, encode table
- `docs/research/artifacts/decoder-allocation-amplification.md` — Section 9, headline, status
- `docs/research/README.md` — the note's catalog disposition, stale at `pending` since it became `partially-adopted`
- `tickets/stop-copying-the-carried-payload-through-the-builder-assemble.md` — the remainder
