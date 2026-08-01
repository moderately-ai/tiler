---
id: restore-the-scalar-cpu-vertical-spike-against-the-current-crates
title: Restore the scalar CPU vertical spike against the current crates
status: review
priority: p2
dependencies: []
related: [generalize-payload-provenance-beyond-the-apple-shape, prototype-a-bounded-scalar-cpu-backend-vertical]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, cpu, maintenance]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785626991
---
## User-visible outcome

The retained scalar CPU vertical compiles and runs again against the current `crates/`, and the result fixture it cites is a measurement of the code beside it rather than of a superseded one.

## Why this exists

**Fact — the spike does not compile, and it did not compile before the provenance work touched it.** At `cbec2d4`, `CARGO_TARGET_DIR=./target cargo check` from `spikes/target-profiles/scalar-cpu-vertical` fails with 13 errors. Ten of them are unrelated to payload provenance and are the subject of this ticket:

- `BackendEntryRef` has no field named `payload` — it is now `payloads`, from the delivery-position step (`src/vertical.rs:379`).
- `DecodedProgram::decode` takes two arguments — it gained a `delivery: usize` parameter (`crates/tiler-runtime/src/load.rs:229`) — at nine call sites in `src/vertical.rs`.

The remaining three were the provenance fields, and [`generalize-payload-provenance-beyond-the-apple-shape`](generalize-payload-provenance-beyond-the-apple-shape.md) fixed those in the same commit that made them necessary: the spike now states `PayloadPlatform::Unversioned`. It stopped there deliberately — repairing the delivery-position drift is a different change, needs a decision about which delivery position the spike's single-position artifact resolves, and would rebaseline cited evidence that the provenance ticket had no mandate to move.

**Fact — the result fixture is therefore stale in a way its own README now records.** `results/2026-07-31-macos-arm64.json` states `payload_bytes: 265`, `envelope_bytes: 20953`, and `artifact_identity_bytes: 9753`. The payload's canonical subject shrank by the three SDK text runs and grew by one appended platform tag, so at least the first of those is wrong, and the other two fold it. No number was hand-edited: a computed measurement is not a measurement.

**Inference — this is what a retained spike costs, and the cost is the point.** `AGENTS.md` records that only re-running a spike detects drift from the source beside it. Two API steps landed without it, and nothing reported that until a third change tried to compile it.

## Closes when

`cargo check` and `cargo run` both succeed from the spike's own directory under the invocation its README records; the run writes `results/` and every byte count in the fixture is from that run; the README's `last_verified` and `verified_at_commit` name the commit that ran it; and finding 7's closure note stops disclaiming the fixture because the fixture is current.

## Graph maintenance

File any further API drift found while repairing this as its own ticket rather than absorbing it — the value of this exercise is the enumeration, not a green build.

## Outcome

**The spike compiles, runs, and its fixture is a measurement of the code beside it.** Base `2119b20`, macOS arm64, the pinned nightly, from `spikes/target-profiles/scalar-cpu-vertical` with `CARGO_TARGET_DIR=./target`: `cargo check` exit 0 with no warnings, `cargo fmt --check` clean, `cargo clippy --all-targets` clean under the spike's own lint set, and `cargo run -- results/2026-07-31-macos-arm64.json` exit 0 with twelve `f32` elements agreeing bit for bit with `tiler-reference`.

**The drift was exactly the ten errors this ticket enumerated, and no more.** `BackendEntryRef { payload }` → `payloads: vec![payload]` at `src/vertical.rs:379`, and the nine `DecodedProgram::decode` call sites took a delivery position. Nothing else in `crates/` had moved under the spike, so **no further-drift ticket was filed** — the enumeration this ticket exists for came back empty beyond its own list.

**The delivery position is zero, derived rather than defaulted.** A delivery position is the ordered slot a consumer's build target resolves to; `crates/tiler-artifact/src/program/model.rs`'s `BackendEntryRef` and `docs/artifact-abi.md` both record that two positions are one compilation, one plan, one kernel program, and two separately compiled objects. `assemble` pushes exactly one carried payload and names it once per entry, so this artifact declares one position, `DecodedProgram::decode` refuses every other index by construction, and zero is the only member of the set rather than the first of several. `DecodedProgram::decode` deliberately has no default for the several-object case, and the spike is not in it. The constant is named `SOLE_DELIVERY` and documented at its definition, matching `prototypes/serial-sum-run`, `prototypes/candle-metal-adapter`, and `crates/tiler-runtime/tests/{identity_join,adapter_route}`. The run now prints the position and the count it was loaded against.

**Fixture, old → new**, every byte count from the run:

| Field | `e2da98f` | `2119b20` |
| --- | --- | --- |
| `target_profile_descriptor_bytes` | 797 | 865 |
| `plan` | `program-alternative:986779d4106ea633` | `program-alternative:72d49e71d668fff8` |
| `envelope_bytes` | 20,953 | 21,296 |
| `artifact_identity_bytes` | 9,753 | 9,969 |
| `reference_registry_identity_bytes` | 446,768 | 912,256 |
| `payload_bytes` | 265 | 265 (unchanged) |
| `deferred_prepared_entry_predicates`, `elements`, `output_bits`, `host`, every key | — | all unchanged |

**This ticket's own `payload_bytes` prediction was wrong, and the README records the correction.** `payload_bytes` is the length of the serialized scalar image — the payload's `code` — which no provenance field is part of; the provenance record folds into the payload descriptor's canonical key and therefore into artifact identity and the envelope, both of which did move. The deltas are otherwise unattributable: 232 commits separate the two bases, 64 touching `crates/` across 158 files, so the README's table states *that* the numbers moved and not *what moved them*, under the same byte-count boundary the previous table set.

**Determinism, measured rather than assumed.** Three consecutive runs at this base — the third after a full rebuild, following the perturbation below — produced byte-identical fixtures and byte-identical 47-line run bodies (`diff` exit 0 on both, across all three pairs). Nothing in the run reads a clock, a hash seed, or the filesystem beyond its own output argument.

**The comparison was re-proved able to say no.** The reference registry identity nearly doubled this interval (446,768 → 912,256 bytes), so the oracle is not the one the previous re-run used and the README's own rule required the perturbation again: replacing the `CanonicalizeF32Nan` arm in `src/interpret.rs` with an identity made the run exit 1 naming exactly one differing element — `0x7fc01234` where the reference requires `0x7fc00000` — with every other element still agreeing. Perturbation reverted; `git diff` over `src/interpret.rs` is empty.

**No research record cited the stale counts.** `grep -rn "20,953\|20953\|9,753\|9753\|446,768\|446768\|(797)\|797 byte\|265 byte\|(265)\|986779d4106ea633" docs/` returns exactly one line, an unrelated nanosecond timestamp in `docs/research/cache/supported-filesystems.md`. `docs/research/target-profiles/` cites the vertical's findings and prose only, never its byte counts, so nothing under the held scope needed correcting.

**Files touched:** `spikes/target-profiles/scalar-cpu-vertical/{src/vertical.rs, README.md, results/2026-07-31-macos-arm64.json}` and this ticket. `Cargo.lock` did not move.
