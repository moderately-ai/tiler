---
schema: "tiler-doc/v1"
id: "tiler.research.artifacts.manifest-fixed-content-growth"
kind: "research"
title: "Where the artifact envelope's fixed content came from"
topics: ["artifacts", "codec", "identity", "measurement", "limits"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["bounded-measurement", "primary-source-synthesis"]
informs: ["tiler.contract.artifact-abi"]
depends_on: ["tiler.research.cache.hot-path-efficiency", "tiler.research.embedding.self-contained"]
ticket: "attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget"
---

# Where the artifact envelope's fixed content came from

**Status:** the growth is attributed to the individual landings that caused it, with a zero residual; the budget question is answered with a recommendation and the two identity-domain proposals it surfaced are parked for Tom
**Ticket:** `attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget`

[The hot-path note's Section 9.1](../cache/hot-path-efficiency.md#91-why-the-envelope-moved-measured-at-both-ends) measured one unchanging fixture's fixed content at **28,527 bytes on 2026-08-04** and **114,043 on 2026-08-06** and said in terms that attributing the difference to individual commits "is not done here". [The embedding note's re-derivation](../embedding/self-contained-embedding.md#the-band-re-derived-2026-08-06--the-envelope-column-moved-and-the-metallib-column-did-not) said the same thing from the other side. This note does it, and then answers the question the attribution was taken for: whether the canonical encoding owes itself a size budget.

Everything measured below comes from [`spikes/artifacts/manifest-growth-attribution/`](../../../spikes/artifacts/manifest-growth-attribution/README.md), which rebuilds the hot-path fixture at every gated landing in the interval and reads its fixed content through the public [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) view.

## The headline, before the evidence

**Measurement.** Of the **107** first-parent landings that touch `crates/` between `194744e6` and `8bd720b8`, exactly **three** moved the envelope's fixed content. One of them moved it by **+80,576 bytes — 94.2% of the whole +85,516**. The other two are +4,922 and +18. The three sum to the measured difference exactly; the residual is **zero bytes**, and 104 landings moved it by nothing at all.

**Measurement.** The +80,576 landing is [`f52c23b8`](#2-the-attribution-landing-by-landing), *Integrate proof-bound stage coverage as a reviewed draft*. What the bytes buy is that every executable stage names the proof-derived reached-only index-refinement evidence for each semantic occurrence it covers, so neither planning nor replay can substitute unrelated evidence. That evidence added **20,144 bytes** to the packaged kernel program's canonical identity — and the envelope stored it **four times**.

**Measurement, and it is the finding this note exists to hand over.** At `8bd720b8` the largest single component of a 114,043-byte fixed-content envelope is the **56,097-byte canonical-identity run the manifest carries at its end — 49.2% of the whole envelope**. It is a complete function of the rest of the manifest, every decode re-derives it and compares, and [`DecodedArtifact::identity`](../../../crates/tiler-artifact/src/program/codec/view.rs) documents that it is "re-derived, never read from the bytes". No consumer in the workspace reads the carried copy.

**Inference, and it is why a byte budget is not the interesting answer.** The bytes that grew are per-occurrence coverage evidence, whose size at the IR layer is measured as `134n² + 3650n + 719` for `n` semantic operations ([the identity-growth spike](../../../spikes/program-planning/identity-growth/README.md), re-run at this record's base). Multiplying that by the envelope's measured factor of four puts the **1 MiB per-invocation embedding ceiling between 32 and 33 semantic operations** — against the **695 operations** at which the 64 MiB program-identity bound binds. **The governed `semantic_operations` budget is already 62**, raised inside this very interval and sized deliberately to the decoder-layer program, which puts the largest program this profile admits at roughly **2.8× the per-invocation ceiling**. The consumer that binds first is not the one the existing deferral was sized against, and it does not bind in the future.

## 1. Procedure, and what makes the fixture unchanging

**The fixture.** [`spikes/cache/hot-path-efficiency`](../../../spikes/cache/hot-path-efficiency/README.md)'s `EnvelopeFactory` compiles one governed serial-sum program and encodes one artifact carrying **zero** object bytes, so the encoded length is pure fixed content and no figure here contains a compiled object. The harness prints that length itself, and Section 9.1's two endpoints were taken by building that commit's own copy of it — the method this sweep reuses at every intermediate point.

**Measurement — the fixture is genuinely unchanging, and this is checkable rather than assumed.** `spikes/cache/hot-path-efficiency/harness/src/envelope.rs` changed exactly **once** in the whole interval, at `002b1d63`, integrated at `69a69201`; `git log --oneline 194744e6..8bd720b8 -- spikes/cache/hot-path-efficiency/harness/src/envelope.rs` returns that one commit and no other. That single change is itself one of the three attributed landings, and Section 3 states what forced it. Every other point in the ladder measures the same source against a different workspace.

**The population, and why it is first-parent.** `git log --first-parent --oneline 194744e6..8bd720b8 -- crates/` gives 107 landings. First-parent because each of those was gated green before publication while a commit inside a merged branch need not build at all; `crates/` because a landing that touches no crate cannot move an encoding. Both interval endpoints are probed too, so the sweep is **109 builds**, and the two endpoints are the oracle: the sweep is only trusted because it reproduces 28,527 and 114,043 — and their `KernelProgramSubject` and `BackendPayloadMetadata` splits — to the byte.

**The instrument.** For each commit, `git archive` extracts that commit's whole tree into a scratch directory — the working tree is never checked out to a historical commit, so this runs beside other branches without touching any of them — and one extra binary is added to the fixture's harness reaching its `envelope.rs` by path. Every quantity is read through the public decoded view rather than by parsing the wire, so a framing change between two probed commits cannot be silently attributed to content. The manifest length is the one derived column, `total − 69 − Σ(section bytes + 12)`, which is Section 9.1's parse.

**Measurement — the harness refuses rather than skipping.** A commit that does not resolve, does not extract, or does not build exits non-zero with its log, because a silent skip in a ladder reads as "no change" — which is the one thing this sweep must never say by accident. Watched failing: `./probe.sh deadbeef` exits 1 at the commit resolution with `deadbeef does not resolve to a commit`, and `./probe.sh` with no argument exits 2. Three landings genuinely refused to build on the first pass, all three because the fixture had not yet merged into `main` at that point in the interval; `probe.sh` now supplies it from `194744e6` and all three measure 28,527.

**Environment.** Apple M4 Max, 14 logical cores, macOS 27.0 (Darwin 27.0.0). `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` from the `nightly-2026-07-19` pin, which `rust-toolchain.toml` does not change anywhere in the interval — `git log --oneline 194744e6..8bd720b8 -- rust-toolchain.toml` is empty, so every one of the 109 builds used one compiler. Dev profile: the quantity measured is a byte count, which no optimizer moves.

## 2. The attribution, landing by landing

**Measurement, 2026-08-06.** Fixed content at each of the 107 landings and both endpoints, retained at [`results/fixed-content-macos-27.0-2026-08-06.tsv`](../../../spikes/artifacts/manifest-growth-attribution/results/fixed-content-macos-27.0-2026-08-06.tsv). Only the rows that move are shown; the ladder is flat between them.

| Landing | Date | Fixed content | Change | Share of the growth | Landings since the last move |
| --- | --- | ---: | ---: | ---: | ---: |
| `194744e6` (interval base) | 2026-08-04 | 28,527 | — | — | — |
| `f52c23b8` *Integrate proof-bound stage coverage as a reviewed draft* | 2026-08-05 | 109,103 | **+80,576** | **94.22%** | 14 |
| `69a69201` *Integrate the delivered-realization wiring's final unit* | 2026-08-05 | 114,025 | **+4,922** | **5.76%** | 26 |
| `f8dfa8f6` *Integrate the four-walls lift* | 2026-08-06 | 114,043 | **+18** | **0.02%** | 36 |
| `8bd720b8` (interval head) | 2026-08-06 | 114,043 | — | — | 28 |

**The sum is the measurement rather than a reconciliation.** 80,576 + 4,922 + 18 = **85,516**, which is Section 9.1's figure exactly. There is no residual to report and no unattributed remainder: **104 of the 107 landings moved the number by zero**, including every landing that widened BF16, admitted multi-output, added the contraction and elementwise families, and changed the compiler's region search.

**One thing that could look like a contradiction, and is not.** [The embedding note](../embedding/self-contained-embedding.md#the-band-re-derived-2026-08-06--the-envelope-column-moved-and-the-metallib-column-did-not) records the authoritative target profile moving from `tiler.metal.macos-apple9.msl4-0.f32.v1` to `…msl4-0.f32-bf16.v1`, and a variant's profile key and descriptor are manifest content, so that move must have cost bytes. It cost none *here* because it is not in this interval: `git log -S f32-bf16 --first-parent 194744e6..8bd720b8 -- crates/` is empty, and the note's comparison spans 2026-07-31 to 2026-08-06 while this one starts on 2026-08-04. What this sweep attributes is the 28,527 → 114,043 move and nothing before it.

### The split at each jump

**Measurement.** The same probe, decomposed. `manifest body` is the manifest less the canonical-identity run it ends with — the run's own eight-byte length prefix counts in the body, so the two columns partition the manifest exactly — and the two named sections are the framed section bytes without their 12 bytes of framing.

| Commit | Fixed content | Manifest body | Identity run | `KernelProgramSubject` | `BackendPayloadMetadata` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `194744e6` | 28,527 | 9,359 | 13,339 | 2,750 | 2,974 |
| `07b8875e` (last before jump 1) | 28,527 | 9,359 | 13,339 | 2,750 | 2,974 |
| `f52c23b8` | 109,103 | 29,503 | 53,627 | 22,894 | 2,974 |
| `eed99219` (last before jump 2) | 109,103 | 29,503 | 53,627 | 22,894 | 2,974 |
| `69a69201` | 114,025 | 31,964 | 56,088 | 22,894 | 2,974 |
| `8bd720b8` | 114,043 | 31,964 | 56,097 | 22,903 | 2,974 |

**`BackendPayloadMetadata` is 2,974 bytes at every row, and it is the control that makes the others readable.** It is the one framed run whose content nothing in the interval touched, and it did not move a byte across 109 builds.

## 3. What each landing bought, and what it cost

### `f52c23b8` — proof-bound stage coverage, +80,576 bytes

**Fact — what it buys.** [`bind-stage-coverage-to-index-refinement-identity`](../../../tickets/bind-stage-coverage-to-index-refinement-identity.md) replaced `StageData::coverage`'s bare `Vec<SemanticOccurrence>` — a graph-local ordinal and nothing else — with a `CoveredOccurrence` record naming the proof-derived reached-only index-refinement evidence for each occurrence a stage claims to implement. The ticket's own option table records why a record and not a pair: it is the only candidate under which a half-populated value is unconstructible, so the builder can refuse a transposition with a typed reason instead of packaging a stage that claims evidence for the wrong occurrence. A compiler proof gap produces no receipt and cannot be encoded as valid coverage. The landing stepped three identity domains — `v9`, `stage-v2`, `artifact-stage-v3` — and moved both pinned artifact identities.

**Measurement — what it cost, and the multiplicity is the whole of it.** The evidence added **20,144 bytes** to the packaged kernel program's canonical identity. The envelope grew by **80,576 = 4 × 20,144**, and the split says where each copy is:

| Copy | Where | Change |
| --- | --- | ---: |
| 1 | the framed `KernelProgramSubject` section — the packaged program's canonical identity | +20,144 |
| 2 | the manifest body, in each variant entry's artifact stage subject | +20,144 |
| 3 | the canonical-identity run, which folds the whole program-subject section verbatim | +20,144 |
| 4 | the canonical-identity run again, in its own restatement of the entries' stage subjects | +20,144 |

**Fact, read from the encoders rather than inferred from the arithmetic.** Copy 1 is the section itself. Copy 2 is `encode_entry`'s `push_slice(bytes, entry.stage.as_bytes())` in [`codec/encode.rs`](../../../crates/tiler-artifact/src/program/codec/encode.rs). Copies 3 and 4 are both inside `encode_identity`'s `push_variant` in [`program/model.rs`](../../../crates/tiler-artifact/src/program/model.rs): it opens with `push_slice(bytes, &envelope.sections()[node_at(variant.program_section)].bytes)`, which is the program-subject section byte for byte, and writes `push_slice(bytes, entry.stage.as_bytes())` once per entry below it.

### `69a69201` — the delivered-realization record, +4,922 bytes

**Fact — what it buys.** `002b1d63` made a delivered-realization record **required** of every artifact, so a reference comparison reads the means by which a numerical contract was honoured rather than inferring them. This is the one landing that changed the fixture, and it changed it because the encoding began requiring what the fixture had not been declaring: the harness's `assemble` gained a `declare_realization` call, and its record is derived from the packaged program's own scheduled realization rather than stated, so the fixture cannot describe a contract its plan does not schedule.

**Measurement — two copies, and the doubling is exact.** The record's canonical encoding is 2,453 bytes, which [the re-derivation ticket](../../../tickets/re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md) measured directly and which `002b1d63`'s own message states as "2,453 canonical bytes carried twice". It appears as 2,461 framed bytes — 2,453 plus its eight-byte length prefix — in the manifest body, and again in the canonical-identity run, for **4,922 = 2 × 2,461**. The sections did not move, because the record is manifest content and not a framed section. The landing stepped `ARTIFACT_DOMAIN` to `tiler.artifact-program.v15`, because two artifacts delivering one contract by different means were previously indistinguishable, and `MANIFEST_SCHEMA` to `13.0`, because every artifact now writes the record's framed run and no `12.0` reader can frame any of them. This is the figure the corpus already carried as "roughly 4.9 KB", now attributed to its landing.

### `f8dfa8f6` — the publishing-copy declaration, +18 bytes

**Fact — what it buys.** The kernel-program identity domain stepped `v9` → `v10` to fold declared publishing-copy contracts. [The pinned-identity test's own comment](../../../crates/tiler-build/src/metal_plan.rs) states the mechanism and states it as a cost: "the section is written unconditionally, so a zero-copy program grows an eight-byte zero count and every program's bytes move", and it says why the step was taken that way rather than as an appended conditional section — an appended one would leave the section's presence positionally ambiguous and constrain every future appended section.

**Measurement.** The `KernelProgramSubject` section grew **9 bytes**: the eight-byte zero count, plus one byte because the domain separator `tiler.kernel-program.v9\0` became `tiler.kernel-program.v10\0`. The manifest body did not move, because both additions are at the program level rather than in the per-stage subjects the entries carry. The canonical-identity run grew 9 with the section it folds, for **18 = 2 × 9**.

**This row is worth more than its size.** It is a landing whose cost is nine bytes of genuinely new declaration and nine bytes of restatement, and it is the smallest possible demonstration that the doubling is structural rather than a property of large additions.

## 4. Where the bytes are now, and which of them are derivable

**Measurement — the composition at `8bd720b8`.**

| Part | Bytes | Share |
| --- | ---: | ---: |
| Framing header | 69 | 0.06% |
| Manifest body, including the identity run's own length prefix | 31,964 | 28.03% |
| **Canonical-identity run inside the manifest** | **56,097** | **49.19%** |
| `KernelProgramSubject` section | 22,903 | 20.08% |
| `BackendPayloadMetadata` section | 2,974 | 2.61% |
| `BackendPayloadCode` section | 0 | — |
| Section framing (3 × 12) | 36 | 0.03% |
| **Fixed content** | **114,043** | |

### The composition after the digest step, measured 2026-08-06

**Measurement, and it is this note's own harness re-run at both ends of the change.** [ADR 0103](../../decisions/0103-declare-the-manifests-artifact-identity-by-digest.md) replaced the trailing identity preimage with a thirty-two-byte digest under a fourth governed envelope domain, at manifest schema `15.0`. `probe.sh` at this branch's base `eee734cf` and at the changed tree give:

| Part | At `eee734cf` | After the step | Change |
| --- | ---: | ---: | ---: |
| Framing header | 69 | 69 | — |
| Manifest, less the trailing run | 31,956 | 31,956 | — |
| **The trailing identity run, with its length prefix** | **56,113** | **32** | **−56,081** |
| `KernelProgramSubject` section | 22,911 | 22,911 | — |
| `BackendPayloadMetadata` section | 2,974 | 2,974 | — |
| Section framing (3 × 12) | 36 | 36 | — |
| **Fixed content** | **114,059** | **57,978** | **−56,081, −49.17%** |

**The base is not this note's `8bd720b8` endpoint, and the difference is accounted for.** `eee734cf` carries the staged-realization step that took the kernel program to `tiler.kernel-program.v11`, which added eight bytes to the program-subject section and eight to the identity that folds it — 114,043 → 114,059, the same nine-plus-restatement shape Section 3's smallest row records.

**Measurement — the derived identity did not move, and neither did any pin.** The probe reads the identity through the public decoded view, and it is 56,105 bytes on both sides. The complete workspace suite passes at the step with no identity constant, golden, or ledger value recomputed. That is Section 6's (b1) claim — the wire moves and the subject does not — checked rather than argued.

**The multiplicity falls from four to two, which is the whole of the saving.** Section 3's four copies were the program-subject section, the manifest's per-entry stage subjects, the identity run's verbatim fold of the section, and the identity run's restatement of those stage subjects. The last two *are* the identity run, so removing it removes exactly copies 3 and 4 and leaves 1 and 2 untouched — which is why the measured reduction is the run and its prefix to the byte, rather than approximately so.

**Measurement — how each part grew.** Manifest body ×3.4 (9,359 → 31,964), identity run ×4.2 (13,339 → 56,097), `KernelProgramSubject` ×8.3 (2,750 → 22,903), `BackendPayloadMetadata` ×1.0. The identity run's *share* barely moved, 46.8% → 49.2%, which is the doubling stated as a ratio: everything added to the manifest is added to the identity that folds it.

**Fact — the identity run is derivable, and that is not an inference.** [`decode`](../../../crates/tiler-artifact/src/program/codec/decode.rs) parses the manifest, builds the envelope, validates it, then calls `envelope.canonical_identity()` and compares the **derived** bytes against the carried run, rejecting a disagreement as `ArtifactIdentityMismatch`. `encode_identity` reads the envelope — schema, routing policy, the three semantic subjects, the interface, providers, payloads, the arena, the variants, the realization record — and reads nothing from the manifest's own encoding, so the run is a pure function of the content that precedes it. [`DecodedArtifact::identity`](../../../crates/tiler-artifact/src/program/codec/view.rs) returns the re-derivation and documents that it is "re-derived, never read from the bytes", so **no consumer in the workspace reads the carried preimage at all**.

**Fact — what the carried run is for.** It is a *declaration*: a producer states the identity it believes it stamped, and a decoder refuses when its own derivation disagrees. That refusal is real and is not the same class as the manifest digest or the canonicity backstop — it fires when the producer's two paths disagree about one artifact, which is exactly the "two definitions that only happened to agree" hazard [the decoder-allocation note's Section 5](decoder-allocation-amplification.md#5-the-arena-content-key-table-and-the-schema-step-that-removed-it) found the crate carrying for canonical arena order. What the refusal does not need is the whole preimage: a digest of the derived identity under its own domain refuses the identical set of disagreements in 32 bytes.

**Fact — the packaged program's own identity is not derivable, and is the other half.** The `KernelProgramSubject` section carries the identity alone; the program is not carried. Nothing else in the envelope determines it, so it is a genuine subject rather than a restatement. Copies 3 and 4 of Section 3 are restatements *of it*; the section is not.

## 5. What this means for the three consumers

**Measurement, restated from the notes that own it.** Envelope size prices three things and none of them owns the number: a validated cache hit is 89.4–90.3% fail-closed integrity over every byte ([the hot-path note](../cache/hot-path-efficiency.md#93-what-dominates-a-hit--the-same-components-a-more-validation-bound-hit)); a macro embedding is 15.17% of the 1 MiB per-invocation ceiling at the largest current member ([the embedding note](../embedding/self-contained-embedding.md#size)); and the expansion cache's 30-day steady state is 0.9–1.6 GB at the same entry count that was 200–400 MB ([the collection design](../cache/bounded-collection.md), corrected there).

**Inference — the growth that matters next is in program size, not in landings.** Every byte this note attributes is per-occurrence coverage evidence on a program whose operation count never changed. [`spikes/program-planning/identity-growth`](../../../spikes/program-planning/identity-growth/README.md) measured what happens when it does: kernel-program identity is exactly `134n² + 3650n + 710` bytes for `n` semantic operations, an exact fit over the domain reachable when it ran, quadratic because one whole `SemanticGraphIdentity` is embedded per coverage record and there is one record per operation.

**Measurement — that harness re-run at this record's base, and it establishes three things before it refuses.** `cd spikes/program-planning/identity-growth && cargo run --release` at `f38813da` exits **1** on its own wall probe, which is the refusal its record designed: `THE WALL MOVED: 9 operations compiled to a 44423-byte identity, so the governed semantic-operations budget is no longer 8 and this ladder is no longer the whole reachable domain.` Before that, (i) its whole ladder is **+9 bytes at every point** against the retained result — 8,546 → 8,555, 12,866 → 12,875, 38,486 → 38,495 — and its fitted constant is **719 rather than 710**, which is exactly the nine bytes `f8dfa8f6` landed, arriving at the program layer from a different harness and confirming Section 3's smallest row independently; (ii) `134·9² + 3650·9 + 719 = 44,423` reproduces the wall probe's measured identity **to the byte**, so the curve holds at a point outside the domain it was fitted on; and (iii) the retained result and every figure derived from it — the refusal point, the ×125 margin, and the deferral's triggers — were computed against a wall that has since moved. Filed as [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](../../../tickets/widen-the-identity-growth-ladder-to-the-governed-operation-budget.md); the figures below use the re-run's `+ 719` constant where it matters and the retained `+ 710` where it does not, and the difference is nine bytes in a six-digit number.

**Inference — the ceiling that binds first is the embedding ceiling, and it binds far earlier than the bound that has an owner.** Taking the measured multiplier of four from Section 3 and the fitted curve above, the envelope's fixed content passes the **1,048,576-byte per-invocation ceiling between 32 and 33 semantic operations** — `4 × (134·32² + 3650·32 + 719) = 1,018,940` and `4 × (134·33² + 3650·33 + 719) = 1,068,380`. The same curve puts the 64 MiB `MAX_PROGRAM_IDENTITY_BYTES` refusal at **695 operations**. The embedding ceiling therefore binds **about 21× earlier in operation count**, and it binds without a typed refusal from the artifact layer at all — the codec's own governed budgets, `MAX_MANIFEST_BYTES` at 64 MiB and `MAX_ENVELOPE_BYTES` at 256 MiB, are three orders of magnitude above anything measured here.

**Fact — and the governed budget already admits a program past it, as of a landing inside this very interval.** `DeterministicBudgets::governed`'s `semantic_operations` is **62**, not the 8 the identity-growth spike's ladder was bounded by. `36d05128` (*Integrate the budgets widening D-18 decided*, 2026-08-05, the 53rd landing in this sweep) sized the five program-scoped bounds "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit", and its comment states the derivation: 62 is "the decode row's occurrence count". That landing moved this fixture's fixed content by **zero**, because the fixture's own program is nowhere near the bound — which is exactly the blind spot a fixed-fixture pin has.

**Inference — so the ceiling is not a future risk, it is a present one.** At the governed maximum of 62 operations the fitted curve gives `134·62² + 3650·62 + 719 = 742,115` bytes of program identity, and four times that is **2,968,460 bytes — 2.83× the 1,048,576-byte per-invocation ceiling, before a single object byte is carried**. The roadmap's decoder layer at ≥ 51 operations is `4 × 535,403 = 2,141,612`, roughly 2.0×.

**Why that conclusion survives the fit being wrong.** Drop the quadratic term entirely — set the coefficient to zero and keep only the linear part the measured domain actually constrains — and 62 operations still give `4 × (3650·62 + 719) = 908,076` bytes, **86.6% of the ceiling with the term that dominates above `n ≈ 27` removed**. The claim that a decoder-layer-sized program does not fit the per-invocation embedding ceiling under this encoding does not depend on the extrapolated term.

**The bound on that inference, stated plainly.** Two of its three inputs are not measurements of the same thing. The multiplier of four is measured, on this fixture, for the coverage increment of Section 3; it is structural — section, plus the identity's fold of the section, plus the entries' stage subjects in the manifest, plus the identity's restatement of them — but it is not measured at any other program shape. The curve is fitted to a different program family over 2..=8 operations, which was the whole reachable domain when it ran and is not any more. Its widest fitted point is 8, its quadratic term does not overtake its linear term until `n ≈ 27`, and the crossing at 32 is therefore four times beyond the widest measurement and in the region the fitted domain constrains least. Two things push back on that and neither removes it: the re-run's 9-operation wall probe reproduces the curve to the byte, and Section 5's linear-only check clears the conclusion at 62 without the extrapolated term. Its record also says the direction of error is unfavourable: richer families raise the per-operation slope and *lower* the crossing. The `≥ 51` is a lower bound taken from a pending proposal, not an observation. **What this licenses is the ordering — the embedding ceiling binds first, by a wide margin — and not the number 32.**

## 6. The decision surface: does the encoding owe itself a budget?

Three options, compared on correctness, maintainability, and the three consumers. Anything that changes what the manifest carries is an identity-domain change and a public boundary, so options (b) are drafted for Tom and not taken here.

### (a) A tracked fixed-overhead budget with a check that fails on unexplained growth

**Where it would live.** `the_standard_metal_path_publishes_its_recorded_identities` in [`crates/tiler-build/src/metal_plan.rs`](../../../crates/tiler-build/src/metal_plan.rs) already pins the artifact identity and the expansion-cache subject of one artifact produced by the real Metal path, already carries a ledger of superseded values in its doc comment, and already documents how to regenerate them on the merged tree. A third pinned constant — that artifact's encoded envelope length — rides exactly that discipline and needs no new machinery, no new fixture, and no new command.

**What the number is and who updates it.** The encoded length of that test's artifact, recomputed on the tree the change lands into, exactly as its two identity pins already require and for the same stated reason: two branch-local rebaselines cannot compose.

**Measurement — the cost, which is the part that is usually guessed.** It would have fired **3 times in 107 gated landings** over this interval, 2.8%. **All three were identity-domain steps that recomputed pinned identities anyway**, on their own commit messages: `f52c23b8` "stepped v9/stage-v2/artifact-stage-v3 with both pins moved", `69a69201` stepped `ARTIFACT_DOMAIN` to `tiler.artifact-program.v15` and `MANIFEST_SCHEMA` to `13.0` with "nine pins recomputed on this tree", and `f8dfa8f6` stepped the kernel program "to v10 with both artifact pins recomputed and ledgered". So the pin adds **no rebaseline event at all** over this interval — it adds one number to a rebaseline that was already happening three times, and is silent on the other 104 landings.

**Correctness.** None. It is a tripwire over a quantity no contract constrains, and it can neither reject an artifact nor change one.

**Implemented 2026-08-06 at `562b02e543e177509575d2f50a9a002e1bd78859`.** `the_standard_metal_path_publishes_its_recorded_identities` pins its published envelope's fixed content — **64,699 bytes**, its 64,707-byte envelope less the eight object bytes the fixture's fake toolchain emits — beside its two identity pins, with the ledger paragraph, Section 7's counterpoint carried at the pin, and the check watched failing under a one-byte lengthening of `MANIFEST_DOMAIN` that left both identity assertions passing. That figure is this test's own fixture and not the ladder's: this note's 57,978 is `spikes/cache/hot-path-efficiency`'s governed serial-sum program, so the two are different programs and are not comparable point to point.

**Maintainability.** One more number in an existing ledger, moved by the landings that already move that ledger. The [make-new-checks-fail discipline](../../../AGENTS.md) is satisfiable directly: introduce the pin with a deliberately wrong constant, watch it fail, then correct it.

**The consumers.** It prices none of them. What it buys is that the number driving all three stops being nobody's: the growth this note attributes ran for two days across 107 landings and was noticed by the cache spike refusing to run, not by anyone watching the encoding.

### (b) Elision or compression of derivable manifest content

Two independent levers, at two layers, with very different sizes and owners.

**(b1) — the artifact layer, sized by this note.** Replace the manifest's trailing canonical-identity **preimage** with its digest under a stated domain. Section 4 establishes the three facts a proposal needs: the run is a pure function of the manifest content above it, every decode re-derives it and compares, and no consumer reads the carried copy. **It removes 56,065 of 114,043 bytes — 49.2% — today, and half of every future manifest addition**, because the doubling of Section 3 becomes a single copy. `MANIFEST_SCHEMA` takes a major step, since the trailing run changes width and meaning and a reader of the earlier schema would frame it wrongly. **Artifact identity does not move**: `encode_identity` reads the envelope and not the manifest, so no pinned identity, no cache subject, and no expansion-cache key changes — only the wire bytes, which is the shape of the `14.0` arena step, where the schema stepped because the wire was permitted to move while identity provably did not. What it costs is that a reader holding only the wire can no longer lift the identity without running the derivation, and today no such reader exists inside the workspace. **The objection to answer is the one the IR-layer deferral already names** — [ADR 0074 convention 2](../../decisions/0074-use-explicit-public-api-conventions.md) says a canonical identity is opaque bytes a receiving crate never re-derives locally, so a digest standing where canonical bytes stood needs an argument that the site is a fold input rather than an identity a consumer compares. At *this* site the argument is available and is a fact rather than a position: the carried run is compared by the crate that is the authority for it, against bytes that same crate derives, and every public reader already reads the derivation instead. It is a declaration a producer makes to its own decoder, not an identity crossing a boundary.

**(b2) — the IR layer, already owned and already deferred.** Fold the per-record `SemanticGraphIdentity` as a digest, collapsing the quadratic term to linear. [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](../../../tickets/decide-whether-executable-coverage-evidence-folds-as-a-digest.md) carries this at `deferred` with three triggers and the redundancy argument already proved at the program layer. Nothing in this note reopens it, and this note does not duplicate it.

**Correctness.** (b1) preserves every refusal the codec makes today — the manifest digest, the canonicity backstop, and the producer-disagreement refusal — and weakens none of them, because the comparison it changes is between two things the decoder holds rather than between the wire and the world. That claim has to be argued rather than asserted, which is why this is Tom's and not a change made here.

**Maintainability.** One major `MANIFEST_SCHEMA` step with its ledger obligations, taken once. Against it: the doubling this note measures is the mechanism by which *every* future manifest addition costs twice what it declares, and (b1) removes it permanently rather than once.

**The consumers.** All three, immediately and by the same factor: a hit's fail-closed integrity runs over 49% fewer bytes, an embedded artifact is 49% smaller against a fixed ceiling, and the cache's steady state halves at the same entry count.

**Inference, and it is the reason neither lever is the answer on its own.** (b1) changes the multiplier from four to two, which moves the embedding-ceiling crossing of Section 5 from ~32 operations to ~50 — still below the decoder layer's ≥ 51 and well below the governed budget's 62. **A 49% one-off saving does not buy an order of magnitude against a quadratic.** The lever that changes the shape of the curve is (b2), and (b1) is a large constant factor in front of it.

**Both were decided on 2026-08-06, and the crossings are now exact rather than approximate.** (b1) landed as [ADR 0103](../../decisions/0103-declare-the-manifests-artifact-identity-by-digest.md); the composition table above is its measurement. Solving the ceiling against the multiplier of two gives the crossing **between 50 and 51 operations** — `2 × (134·50² + 3650·50 + 719) = 1,036,438` and `2 × (134·51² + 3650·51 + 719) = 1,070,806` — which confirms the "~50" this paragraph estimated. (b2) was chosen as [ADR 0104](../../decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) and, as this paragraph first stood, **had not landed**: folding the per-record graph identity was predicted to make program identity `3525n + 719`, moving the crossing to between 148 and 149 operations and turning the curve linear, but it needed a governed digest inside `tiler-ir`, which is the workspace's bottom crate and reached none. Siting the digest was a crate-boundary question that record put to Tom.

**Landed 2026-08-06, and the prediction is now a measurement.** Tom answered the boundary question by admitting `tiler-digest` as a new bottom crate below `tiler-ir`, and the fold executed with it. Measured on the ordinary compilation path over 2..=8 operations with a nine-operation probe outside the fitted domain, kernel-program identity is exactly **`3525n + 727`, quadratic coefficient zero** — the predicted linear coefficient to the unit, over a constant eight bytes higher than this note's arithmetic because the `tiler.kernel-program.v11` staged-realization step landed between the two and adds an unconditional eight-byte zero count to every program. The same tree measured `134n² + 3650n + 727` immediately before the fold, so **every `+ 719` in Sections 5 and 6 of this note is eight bytes low**, including the 50/51 crossing solved directly above; no conclusion moves, because eight bytes doubled is sixteen against a 1,048,576-byte ceiling. Recomputed on the measured constants: the crossing against the multiplier of two is **between 148 and 149 operations** — `2 × (3525·148 + 727) = 1,044,854` and `2 × (3525·149 + 727) = 1,051,904` — the 64 MiB program bound moves from 695 operations to **19,038**, and the governed budget of 62 operations gives **219,277 bytes**, whose doubling is **41.8%** of the ceiling where the quadratic encoding stood at 283%. So the state this note now describes is (b1) done, (b2) done, and the encoding linear.

**What that does not license.** The fit is exact on 2..=8 operations and the nine-operation probe reproduces it to the byte, which is one point outside the domain and not a widened ladder; every crossing above is still an extrapolation across two orders of magnitude, and every coefficient is a property of one unary `f32` multiply-chain family. The harness that produced it also still refuses on its own wall probe, because the governed `semantic_operations` budget moved from 8 to 62 and its ninth point now compiles instead of refusing — a finding it is built to report, owned by [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](../../../tickets/widen-the-identity-growth-ladder-to-the-governed-operation-budget.md). That re-run must fit a linear curve rather than a quadratic one and carry the 727 constant; the spike's retained 2026-08-05 result is superseded in both respects and in every row of its ladder.

### (c) Deliberately no budget while the ceiling stands

**What it rests on.** The ceiling is 15.17% consumed and the codec's own hard budgets are three orders of magnitude away and fail closed with typed refusals. Every one of the three landings was a deliberate, reviewed identity-domain step, and the largest of them *was* noticed on the day: the review of `bind-stage-coverage-to-index-refinement-identity` filed the identity-growth measurement, which became the fitted curve Section 5 uses.

**Why it is not enough.** It was in force across this interval and it is what produced a 4.4× envelope move nobody attributed for two days. The review noticed the growth **at the IR layer**, against the 64 MiB program bound; nothing carried that observation to the envelope consumers, and the three notes that price against envelope size each recorded a number that had already moved. "Somebody noticed" and "the consumers knew" are different states, and only the second is a control.

**Correctness, maintainability, consumers.** No correctness content, zero maintenance, and it prices no consumer — which is precisely its whole cost. **The trigger that would reopen it** is worth stating even though (c) is not the recommendation: the fixed content passing a named fraction of the 1,048,576-byte per-invocation ceiling. This fixture's 114,043 bytes are 10.9% of it, and the embedding note's largest real member is at 15.17% because it also carries a `metallib`.

**And on the evidence of Section 5 that trigger has already fired.** "While the ceiling stands" is a claim about programs the profile admits, not about one fixture, and the governed `semantic_operations` budget was raised to 62 inside this very interval. The reassuring 10.9% is a property of a small serial-sum program, not of the encoding.

## 7. Recommendation

**Proposal — take (a), in its cheapest form, and treat neither (b) as an alternative to it.**

A byte pin on the existing golden costs one line in a ledger that already exists, would have fired three times in 107 landings, and turns the attribution this note performed by hand into something a landing does for free. It is the only one of the three options that is not an identity-domain change, so it is the only one that can be acted on without Tom. (b1) is a large, cheap, well-evidenced reduction and belongs to Tom as a decision node, not to a recommendation here. (c) is what was in force and is what this ticket exists because of.

**The strongest counterpoint, and this sweep contains its own demonstration.** *A pin on one fixture at one fixed operation count measures a coefficient, not a curve.* Every byte this note attributes is per-occurrence evidence over a program whose operation count never moved, so the pin sits green while the identical encoding becomes unusable at ~32 operations. The demonstration is in the ladder: **`36d05128` raised the governed `semantic_operations` budget from 8 to 62 — admitting, by size, a program whose envelope this encoding puts at ~2.8× the per-invocation ceiling — and moved this fixture's fixed content by exactly zero.** It is one of the 104 flat rows. A check that cannot fire on the failure mode that will actually occur is worse than no check when it is read as coverage, and the ledger discipline that makes a pin *informative* — writing down what the bytes bought — is exactly the part no check can enforce.

**The answer to it, which is not to drop the pin but does re-rank it.** The pin and the curve answer different questions and the repository needs both, and the pin is nearly free. But the item this attribution found that is worth doing *first* is not a check at all: it is a **trigger**. The deferral that owns the curve was sized entirely against the 64 MiB program-identity bound, and its first trigger is a program boundary above ~350 operations — roughly eleven times past the point at which the embedding ceiling already refuses, and unfired at the 62 the budget already carries. That gap is filed as [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](../../../tickets/add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md), and the harness whose curve both triggers depend on is stale by its own wall probe, filed as [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](../../../tickets/widen-the-identity-growth-ladder-to-the-governed-operation-budget.md). **The budget question's answer is the pin; the attribution's most consequential finding is not the budget question.**

## 8. What this note does not establish

**One fixture, one program shape.** One variant, one payload, one governed serial-sum program at a fixed operation count. The multiplier of four is measured for one landing's increment on that fixture; nothing here measures it at another program shape, another variant count, or another entry count.

**No timing, and no allocation.** Every figure is a byte count. What those bytes cost to validate is [the hot-path note](../cache/hot-path-efficiency.md)'s and what they cost to hold is [the decoder-allocation note](decoder-allocation-amplification.md)'s; neither is re-derived here.

**The ceiling arithmetic is an extrapolation over a fit to a different family.** Section 5 states its three inputs and which of them are measurements. Two things bound how far it can be wrong and neither removes the extrapolation: the fitted curve reproduces the 9-operation point outside its domain to the byte, and the conclusion at 62 operations survives deleting the quadratic term entirely. What it licenses is that a decoder-layer-sized program does not fit the per-invocation ceiling under this encoding, and not the crossing point 32.

**No program above 9 operations was compiled here.** The 62-operation figure is arithmetic over a fit, not an observation, and nothing in this note establishes that a 62-operation program compiles at all — only that its identity, if it did, would not fit the ceiling. Measuring it is [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](../../../tickets/widen-the-identity-growth-ladder-to-the-governed-operation-budget.md)'s, and the retained `compile_ms` column suggests the ladder may not reach that far affordably.

**Nothing about intervals other than this one.** 107 landings over two days on one branch's first-parent chain. The 2.8% move rate is that interval's, and a period of concentrated identity work would produce a different one.

**No encoding was changed, and no boundary was moved.** Both (b) proposals are drafted and parked.

## 9. Outcomes

1. **The growth is attributed with a zero residual.** Three landings of 107; +80,576, +4,922, +18; summing to the +85,516 [the hot-path note](../cache/hot-path-efficiency.md#91-why-the-envelope-moved-measured-at-both-ends) and [the embedding note](../embedding/self-contained-embedding.md#the-band-re-derived-2026-08-06--the-envelope-column-moved-and-the-metallib-column-did-not) both recorded as unattributed. Both of those notes are outside this branch's scopes and are not edited from here; the cross-reference each owes is listed on this note's ticket.
2. **A bounded experiment, preserved.** [`spikes/artifacts/manifest-growth-attribution/`](../../../spikes/artifacts/manifest-growth-attribution/README.md), with its extraction method, its refusal behaviour watched failing, its stated population of 107, and its retained 109-row ladder.
3. **One identity-domain proposal drafted and parked for Tom.** (b1), the manifest's carried identity preimage as a digest: 49.2% of today's fixed content, half of every future addition, artifact identity unmoved, `MANIFEST_SCHEMA` major step. Carried by [`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](../../../tickets/decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md).
4. **One trigger gap filed rather than absorbed.** [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](../../../tickets/add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md) carries the arithmetic and the exact trigger text; the deferral it amends is another ticket's body and is not edited from here.
5. **One defect found in a retained experiment, by running it.** [`spikes/program-planning/identity-growth`](../../../spikes/program-planning/identity-growth/README.md) exits 1 at this record's base on its own wall probe, because `36d05128` raised the governed operation budget from 8 to 62 inside this interval. Filed as [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](../../../tickets/widen-the-identity-growth-ladder-to-the-governed-operation-budget.md), which owns the re-run, the new retained result, and the moved verdict. Nothing under `spikes/program-planning/` is edited from here.
6. **The budget question is answered, and it is not the most consequential thing the attribution found.** Recommend the pin, with its blind spot demonstrated from this sweep's own ladder rather than argued. The recommendation is this note's; the two encoding changes it declines to fold in are Tom's.
7. **Two catalog rows this branch's scopes cannot reach.** This record and the spike each need a line in a `contracts/navigation` catalog; both are recorded verbatim on this note's ticket.

## Traceability

- Ticket: `attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget`
- Experiment: [`spikes/artifacts/manifest-growth-attribution/`](../../../spikes/artifacts/manifest-growth-attribution/README.md)
- Measured interval: `194744e6..8bd720b8`, base of this record `f38813da`
- Prior measurements this note attributes: [hot-path Section 9.1](../cache/hot-path-efficiency.md#91-why-the-envelope-moved-measured-at-both-ends), [embedding Section 1](../embedding/self-contained-embedding.md#the-band-re-derived-2026-08-06--the-envelope-column-moved-and-the-metallib-column-did-not)
- Curve this note extrapolates: [`spikes/program-planning/identity-growth/`](../../../spikes/program-planning/identity-growth/README.md)
