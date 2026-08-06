---
id: attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget
title: Attribute the canonical-manifest growth and decide whether the encoding owes a budget
status: review
priority: p2
dependencies: []
related: [re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps, re-price-the-envelope-band-consumers-against-the-re-derived-band, decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest, add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral, widen-the-identity-growth-ladder-to-the-governed-operation-budget]
scopes: [research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [research, artifacts, measurement, encoding]
claimed_from: todo
assignee: agent-manifest-growth
lease_expires_at: 1786053357
---
## User-visible outcome

The 4× growth of the artifact envelope's fixed content between 2026-08-04 and 2026-08-06 is attributed to the changes that caused it, and the repository states whether the canonical encoding owes itself a size budget — or records why unbounded manifest growth is acceptable while a 1 MiB per-invocation embedding ceiling stands.

## The measurement this starts from, taken rather than owed

**Measurement** ([the hot-path note §9.1](../docs/research/cache/hot-path-efficiency.md), 2026-08-06). One unchanging fixture's zero-object envelope was built at two commits and both framings parsed: fixed content is 28,527 bytes at `194744e6` (2026-08-04) and 114,043 at `8bd720b8` — +65,363 bytes of canonical manifest, +20,153 of `KernelProgramSubject` section, `BackendPayloadMetadata` byte-identical as the control. On real producer output the effect is a 4.4× envelope band move (32,136–47,803 → 141,532–159,037) with every `metallib` byte-identical; on the largest member the canonical manifest is 76.3% of the envelope and the carried compiled object under a twentieth. The `MANIFEST_SCHEMA` steps in the interval (12.0 → 14.0) change no lengths, so the growth is in what the manifest describes, not the framing.

**Why it matters now, not later.** The macro embedding ceiling headroom fell from "more than an order of magnitude" to 15.17% consumed ([the embedding note](../docs/research/embedding/self-contained-embedding.md) §5): roughly two-thirds of one more threefold growth exhausts the 1 MiB per-invocation gate. The cache-hit cost is now ~90% envelope validation, and [the hot-path note §9.7](../docs/research/cache/hot-path-efficiency.md) closes with "if the hit path is ever worth attacking again, the lever is the encoding, not this crate". Three consumers price against envelope size (macro embedding, expansion-cache steady state, cache-hit latency), and none of them owns the number that drives all three.

## What this must produce

1. **The attribution, as a bounded experiment.** Rebuild the hot-path fixture (or an equivalent unchanging fixture) at the intermediate commits between `194744e6` and `8bd720b8` and attribute the +85,516 fixed bytes to the changes that added them, separating what each buys (identity coverage, delivered-realization evidence, staged-coverage encoding, …) from what it costs. The re-derivation deliberately did not do this; its fixture-rebuild method is recorded in [its ticket](re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md) and reuses directly.
2. **The decision surface, drafted not decided.** Whether the encoding owes a budget (a tracked fixed-overhead number with a check that fails on unexplained growth, per the make-new-checks-fail discipline), owes compression or elision for derivable content, or deliberately owes nothing while the ceiling stands — compare on correctness, maintainability, and the three consumers' costs, give the strongest counterpoint, recommend one. Anything touching the canonical encoding is an identity-domain change and a public boundary: draft and park for Tom.

## Non-goals

Changing the encoding; re-deciding the 1 MiB ceiling or the 30-day cache window (owned elsewhere); re-running the consumer re-pricing ([`re-price-the-envelope-band-consumers-against-the-re-derived-band`](re-price-the-envelope-band-consumers-against-the-re-derived-band.md) owns it).

## Closes when

The growth is attributed with per-change sizes on the unchanging fixture, each attributed change names what the bytes buy, and the budget question is answered or parked for Tom with the evidence and a recommendation.

## Outcome — 2026-08-06, based at `f38813da`

**The record is [`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md)**, sited beside the envelope model and the decoder-allocation note rather than inside the latter: that note's metric is what a decode *allocates*, this one's is what the wire *is*, and a reader asking "why is the envelope this big" looks in `docs/research/artifacts/`. The harness is [`spikes/artifacts/manifest-growth-attribution/`](../spikes/artifacts/manifest-growth-attribution/README.md).

### The attribution — three landings of 107, zero residual

**Measurement.** The population is the **107** first-parent landings touching `crates/` in `194744e6..8bd720b8`, plus both endpoints: **109 builds**, each a rebuild of the hot-path fixture from a `git archive` extraction of that commit's whole tree. The two endpoints reproduce the published 28,527 and 114,043 and their section splits to the byte, which is the oracle the rest rests on.

| Landing | Date | Fixed content | Change | Share | Landings since the last move |
| --- | --- | ---: | ---: | ---: | ---: |
| `194744e6` (base) | 2026-08-04 | 28,527 | — | — | — |
| `f52c23b8` *Integrate proof-bound stage coverage as a reviewed draft* | 2026-08-05 | 109,103 | **+80,576** | **94.22%** | 14 |
| `69a69201` *Integrate the delivered-realization wiring's final unit* | 2026-08-05 | 114,025 | **+4,922** | **5.76%** | 26 |
| `f8dfa8f6` *Integrate the four-walls lift* | 2026-08-06 | 114,043 | **+18** | **0.02%** | 36 |
| `8bd720b8` (head) | 2026-08-06 | 114,043 | — | — | 28 |

80,576 + 4,922 + 18 = **85,516**, the published difference exactly. **104 landings moved it by zero**, including every BF16 widening, the multi-output admission, the contraction and elementwise families, and the region-search changes.

### What each landing bought, and the multiplicity that priced it

| Commit | Fixed content | Manifest body | Identity run | `KernelProgramSubject` | `BackendPayloadMetadata` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `194744e6` | 28,527 | 9,359 | 13,339 | 2,750 | 2,974 |
| `07b8875e` | 28,527 | 9,359 | 13,339 | 2,750 | 2,974 |
| `f52c23b8` | 109,103 | 29,503 | 53,627 | 22,894 | 2,974 |
| `eed99219` | 109,103 | 29,503 | 53,627 | 22,894 | 2,974 |
| `69a69201` | 114,025 | 31,964 | 56,088 | 22,894 | 2,974 |
| `8bd720b8` | 114,043 | 31,964 | 56,097 | 22,903 | 2,974 |

- **`f52c23b8`, +80,576.** Buys: every executable stage names the proof-derived reached-only index-refinement evidence for each occurrence it covers, so neither planning nor replay can substitute unrelated evidence and a proof gap cannot be encoded as verified coverage ([`bind-stage-coverage-to-index-refinement-identity`](bind-stage-coverage-to-index-refinement-identity.md)). Costs: **20,144 bytes of program identity, stored four times** — the framed `KernelProgramSubject` section, the manifest body's per-entry stage subjects, the canonical-identity run's verbatim fold of that section, and the same run's restatement of those stage subjects. `80,576 = 4 × 20,144`, and each copy is read off `encode_entry` and `encode_identity`'s `push_variant` rather than inferred.
- **`69a69201`, +4,922.** Buys: a delivered-realization record required of every artifact, so a reference comparison reads the means rather than inferring them; `ARTIFACT_DOMAIN` → `v15`, `MANIFEST_SCHEMA` → `13.0`. Costs: the record's 2,453 canonical bytes framed to 2,461 and carried **twice** — manifest body and identity run — for `4,922 = 2 × 2,461`. This is the landing that changed the fixture, and it changed it because the encoding began requiring what the fixture had not declared.
- **`f8dfa8f6`, +18.** Buys: kernel-program `v10`, folding declared publishing-copy contracts. Costs: **9 bytes** in the section — an unconditional eight-byte zero count plus one byte for `tiler.kernel-program.v9\0` → `…v10\0` — carried twice, `18 = 2 × 9`. The smallest possible demonstration that the doubling is structural.

### The finding the attribution surfaced

**Measurement.** At `8bd720b8` the largest single component of the 114,043-byte envelope is the **56,097-byte canonical-identity run the manifest carries at its end, 49.2%**. **Fact:** `encode_identity` reads the envelope and never the manifest, so the run is a pure function of the content above it; `decode` re-derives and compares it, rejecting a mismatch as `ArtifactIdentityMismatch`; and `DecodedArtifact::identity` returns the derivation, documented "re-derived, never read from the bytes". **No consumer in the workspace reads the carried preimage.**

**Inference, with its bound stated in the record.** Combining the measured multiplicity of four with the fitted `134n² + 3650n + 719` from [`spikes/program-planning/identity-growth`](../spikes/program-planning/identity-growth/README.md), the envelope's fixed content passes the **1,048,576-byte per-invocation embedding ceiling between 32 and 33 semantic operations**, against **695** for the 64 MiB program-identity bound — about **21× earlier**. **Fact:** the governed `semantic_operations` budget is **62**, raised from 8 by `36d05128` *inside this interval* and sized deliberately to the decoder-layer program, which puts the largest program this profile admits at ≈ 2.83× the ceiling; the roadmap's ≥ 51 lands at ≈ 2.04×. The conclusion survives deleting the quadratic term entirely — 62 operations are still 86.6% of the ceiling on the linear term alone — so what is extrapolated is the crossing point, not the ordering.

**Measurement, and one defect it found.** Re-running that harness at `f38813da` **exits 1** on its own wall probe: `THE WALL MOVED: 9 operations compiled to a 44423-byte identity, so the governed semantic-operations budget is no longer 8`. Before refusing it establishes two things worth keeping: its whole ladder is **+9 bytes at every point** and its fitted constant is 719 rather than 710 — exactly the `f8dfa8f6` nine bytes, arriving independently at the program layer — and `134·9² + 3650·9 + 719 = 44,423` reproduces the wall probe's measured identity to the byte, so the curve holds one point outside the domain it was fitted on. Nothing under `spikes/program-planning/` is edited from here; the re-run and the moved verdict are [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](widen-the-identity-growth-ladder-to-the-governed-operation-budget.md)'s.

### The decision surface, and the recommendation

**Recommended: (a), a byte pin on the existing golden.** Add the encoded envelope length as a third pinned constant to `the_standard_metal_path_publishes_its_recorded_identities` in `crates/tiler-build/src/metal_plan.rs`, which already pins two identities on the real producer path, already carries a superseded-value ledger, and already documents regeneration on the merged tree. **Measured cost: it would have fired 3 times in 107 landings, and all three were identity-domain steps that recomputed pinned identities in that same file anyway** — so it adds no rebaseline event over this interval, only one number to three that were already happening.

**Strongest counterpoint, demonstrated from this sweep's own ladder rather than argued.** A pin on one fixture at one fixed operation count measures a coefficient, not a curve. **`36d05128` raised the governed operation budget from 8 to 62 — admitting by size a program this encoding puts at ~2.8× the per-invocation ceiling — and moved this fixture's fixed content by exactly zero.** It is one of the 104 flat rows. **The answer is not to drop the pin** (it is nearly free) but to re-rank it: the item worth doing first is the trigger, because the deferral that owns the curve is sized entirely against the 64 MiB bound and its first trigger is ~350 operations, roughly eleven times past where the embedding consumer already refuses and unfired at the 62 the budget already carries. The budget question's answer is the pin; the attribution's most consequential finding is not the budget question.

**Rejected as the answer, filed as decisions:** (b1) replacing the manifest's carried identity preimage with its digest — 49.2% of today's fixed content and half of every future addition, artifact identity unmoved, `MANIFEST_SCHEMA` major step — is an identity-domain change and Tom's; and it does not answer the question anyway, because halving a constant does not buy an order of magnitude against a quadratic (it moves the crossing to ~50 operations, still below ≥ 51). (c) is what was in force across this interval and is what produced a 4.4× move nobody attributed for two days.

### Filed rather than absorbed

- [`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md) — `todo`, the (b1) decision node, with the three facts a proposal needs and the ADR 0074 convention 2 objection it has to answer.
- [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md) — `todo`, carrying the exact trigger text for [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md), whose body is another ticket's and is not edited from here.
- [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](widen-the-identity-growth-ladder-to-the-governed-operation-budget.md) — `todo`, `research/program-planning`, the stale ladder found by running the harness and watching its own wall probe refuse.

### Owed catalog rows, `contracts/navigation`, outside this branch's scopes

Verbatim, for whoever holds the scope. In `docs/research/README.md`, under `### Artifacts, build, and toolchains`, in the section's alphabetical position by title (immediately after the `The expansion cache under Cargo and rust-analyzer` row):

```markdown
- [Where the artifact envelope's fixed content came from](artifacts/manifest-fixed-content-growth.md) — pending; bounded-measurement, primary-source-synthesis; informs: [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md); experiments: [Which landings moved the artifact envelope's fixed content](../../spikes/artifacts/manifest-growth-attribution/README.md)
```

In `spikes/README.md`, under `### Artifacts, build, and toolchains`, in the section's alphabetical position by title (immediately after the `What validating one artifact envelope allocates` row):

```markdown
- [Which landings moved the artifact envelope's fixed content](artifacts/manifest-growth-attribution/README.md) — reproducible; bounded-measurement; supports: [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md)
```

### Owed cross-references, outside this branch's scopes

Neither is edited from here, and each is a one-sentence pointer rather than a rewrite:

- `docs/research/cache/hot-path-efficiency.md` §9.1 (`research/cache`) ends "Attributing it to individual changes is not done here and is not this note's question." That attribution now exists and should point at it.
- `docs/research/embedding/self-contained-embedding.md` §1 (`research/embedding`) says "Naming the responsible changes needs the same fixture rebuilt at intermediate commits, which is not done here." Same.

### Files changed, and the scope statement

`docs/research/artifacts/manifest-fixed-content-growth.md` (new), `spikes/artifacts/manifest-growth-attribution/{README.md,probe.rs,probe.sh,sweep.sh,results/fixed-content-macos-27.0-2026-08-06.tsv}` (new), `tickets/decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md` (new), `tickets/add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md` (new), `tickets/widen-the-identity-growth-ladder-to-the-governed-operation-budget.md` (new), and this ticket. **Nothing outside `docs/`, `spikes/`, and `tickets/`; no crate, prototype, manifest, or gate file is touched.** Every path is inside `research/artifacts` (`docs/research/artifacts/**`, `spikes/artifacts/**`) or the shared `project/tickets`.

### Reproduce

```sh
cd spikes/artifacts/manifest-growth-attribution
./sweep.sh 194744e6 8bd720b8 > results/fixed-content-macos-27.0-2026-08-06.tsv
```

Apple M4 Max, macOS 27.0 (Darwin 27.0.0), `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` from the `nightly-2026-07-19` pin — which `rust-toolchain.toml` does not change anywhere in the interval, so all 109 builds used one compiler. Dev profile, because the measured quantity is a byte count. The retained run drove two copies over disjoint halves of the commit list with separate scratch roots; `sweep.sh` is the single-worker form and produces the same rows.

### Scratch cleaned

Every extraction is removed by `probe.sh` on success; the scratch roots and their shared target directories were deleted after the run. The working tree was never checked out to a historical commit.
