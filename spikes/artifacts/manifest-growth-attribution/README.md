---
schema: "tiler-doc/v1"
id: "tiler.spike.artifacts.manifest-growth-attribution"
kind: "experiment"
title: "Which landings moved the artifact envelope's fixed content"
topics: ["artifacts", "codec", "identity", "measurement"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.artifacts.manifest-fixed-content-growth"]
entrypoints: ["spikes/artifacts/manifest-growth-attribution/sweep.sh", "spikes/artifacts/manifest-growth-attribution/probe.sh", "spikes/artifacts/manifest-growth-attribution/probe.rs"]
last_verified: "2026-08-06"
ticket: "attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget"
---

# Which landings moved the artifact envelope's fixed content

This harness rebuilds **one unchanging fixture** at every gated landing of an interval and reports the byte length of the artifact envelope it produces, split into the parts a reader can act on. It exists because two research notes measured the same fixture at two commits two days apart, found it had grown from 28,527 to 114,043 bytes, and each said in terms that attributing the difference to individual commits was not done there. [The research record](../../../docs/research/artifacts/manifest-fixed-content-growth.md) is what this produced.

```sh
cd spikes/artifacts/manifest-growth-attribution
./sweep.sh 194744e6 8bd720b8 > results/fixed-content-macos-27.0-2026-08-06.tsv
```

Nothing runs it automatically and no `make` target reaches `spikes/`. One landing takes roughly 30 seconds of build; the interval above is 109 builds and runs a little under an hour when two copies are driven over disjoint halves of the commit list with separate scratch roots, which is what produced the retained result. `./probe.sh <commit> [scratch-root]` is the single-commit form and prints one row.

## The fixture, and why it is this one

[`spikes/cache/hot-path-efficiency`](../../cache/hot-path-efficiency/README.md)'s `EnvelopeFactory` compiles one governed serial-sum program and encodes one artifact carrying **zero** object bytes. Its encoded length is therefore pure fixed content — no compiled object is in any figure this harness reports — and the harness already prints that length itself, which is how the two endpoint measurements this sweep reproduces were taken.

**It is genuinely unchanging over the measured interval, and that is checkable rather than assumed.** `git log --oneline 194744e6..8bd720b8 -- spikes/cache/hot-path-efficiency/harness/src/envelope.rs` returns exactly one commit, `002b1d63`, and that change is itself one of the three landings the record attributes: the encoding began *requiring* a delivered-realization record, so the fixture had to declare one. Every other point measures the same source against a different workspace.

## What each column is

`probe.rs` is added as a second binary of that harness in the extracted tree, reaching its `envelope.rs` module by path so the fixture is the same source rather than a copy of it. Every quantity is read through the **public** [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) view, so a framing change between two probed commits cannot be silently attributed to content.

| Column | What it is |
| --- | --- |
| `total` | the encoded envelope length, which for this fixture is its fixed content |
| `manifest` | `total − 69 − Σ(section bytes + 12)`, the one derived column, which is the two-end parse [the hot-path note's Section 9.1](../../../docs/research/cache/hot-path-efficiency.md#91-why-the-envelope-moved-measured-at-both-ends) states |
| `identity` | the canonical-identity run, re-derived from the decoded content — `DecodedArtifact::identity` never reads the carried copy, and a decode that reached this point already proved the two are equal |
| `variants`, `payloads` | shape controls: a moved envelope with a moved variant or payload count is a different artifact, not a grown one |
| `sections` | each framed section's purpose and its bytes without the 12 bytes of framing |

The manifest **body** — what the record's tables report — is `manifest − identity`.

## The population, and why it is first-parent

`sweep.sh` takes the **first-parent** commits of `base..tip` that touch `crates/`. First-parent because every one of those was gated green before it was published, while a commit inside a merged branch need not build at all; `crates/` because a landing that touches no crate cannot move an encoding. Both interval endpoints are probed as well, so a run is `population + 2` builds, and the count is printed as a comment ahead of the rows so a run that reached nothing cannot read as a run that found nothing.

## How this could have been vacuous, and what stops each way

**The endpoints are the oracle.** The sweep is only worth reading because it reproduces the two independently published figures — 28,527 at `194744e6` and 114,043 at `8bd720b8` — and their `KernelProgramSubject` and `BackendPayloadMetadata` splits, to the byte. A method that agreed with itself and disagreed with those would be reporting its own extraction.

**A landing that does not build refuses instead of being skipped.** A silent skip in a ladder reads as "no change", which is the one thing this sweep must never say by accident. `probe.sh` resolves the commit before extracting anything, because `git archive` sits in the left half of a pipeline where `set -e` cannot see it, and an unresolvable commit would otherwise surface four steps later as a missing Cargo manifest. Watched failing: `./probe.sh deadbeef` exits 1 saying the commit does not resolve; `./probe.sh` with no argument exits 2. Three landings in the measured interval did refuse on a first pass, all three because the fixture had not yet merged into `main` at that point; `probe.sh` now supplies it from `194744e6` and all three measure 28,527, agreeing with their neighbours.

**Nothing is checked out.** `git archive` extracts each commit's whole tree into a scratch directory, so this runs beside other agents' branches without touching the working tree, and a run that is interrupted leaves the repository exactly as it found it.

**One compiler across the whole ladder.** `git log --oneline 194744e6..8bd720b8 -- rust-toolchain.toml` is empty for the measured interval, so all 109 builds used `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`. The dev profile is used deliberately: the measured quantity is a byte count, which no optimizer moves, and the release build costs several times as much per landing.

**A shared target directory is a build-time optimization and not a measurement shortcut.** Path dependencies rebuild at every commit because their source path moves; what is reused is the registry dependencies. Every reported row comes from a fresh build of the four Tiler crates at that commit.

## Retained result

[`results/fixed-content-macos-27.0-2026-08-06.tsv`](results/fixed-content-macos-27.0-2026-08-06.tsv) — 107 first-parent landings touching `crates/` between `194744e6` and `8bd720b8`, plus both endpoints. Apple M4 Max, 14 logical cores, macOS 27.0 (Darwin 27.0.0), the `nightly-2026-07-19` pin, dev profile.

**Three landings of the 107 moved the number**, by +80,576, +4,922, and +18, summing to the published +85,516 with a zero residual. [The research record](../../../docs/research/artifacts/manifest-fixed-content-growth.md) names each one and what its bytes buy.

## Boundary

- **One fixture, one program, one variant, one payload, one operation count.** Nothing here bounds an envelope large in variants, entries, bindings, or semantic operations, and the record's extrapolation to the last of those is explicitly labelled as one.
- **One interval on one branch's first-parent chain.** The 3-in-107 move rate is that interval's.
- **Byte counts only.** No timing, no allocation, and no device. What these bytes cost to validate is [the hot-path note](../../../docs/research/cache/hot-path-efficiency.md)'s and what they cost to hold is [the decoder-allocation note](../../../docs/research/artifacts/decoder-allocation-amplification.md)'s.
- **The commit list is history.** Re-running this at a later interval measures that interval; it does not refresh the retained rows, which are evidence at the commits they were taken at.
