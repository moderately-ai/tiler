---
id: route-the-realization-conformance-half-into-the-conformance-crate
title: Route the realization-conformance half into the conformance crate
status: done
priority: p2
dependencies: [carry-the-device-executed-value-proof-into-the-conformance-crate]
related: [retain-contraction-conformance-evidence, publish-an-l3-contraction-cell-through-the-accepted-route, survey-what-belongs-in-the-conformance-crate, decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights, state-a-subject-on-the-contraction-publication-path-s-reference-oracle]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, contraction, migration]
---
## User-visible outcome

Five of six L3 correctness cells' retained `result_sha256` values are compared against **executed** device results inside `crates/tiler-conformance` (machine fields must match the retained row; toolchain differences are announced by name and comparison proceeds — see Outcome deviation). The sixth cell (`w_vocab_slice`) is pinned in the table but excluded from the routed set by `MAX_PROOF_PAYLOAD_BYTES` and is owned by [`decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights`](decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights.md). The spike's retained record is a gate for the five routable cells rather than a document.

## Why this is separate from its parent ticket

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md).

[`carry-the-device-executed-value-proof-into-the-conformance-crate`](carry-the-device-executed-value-proof-into-the-conformance-crate.md) *relocates* what already runs, which is one cell. This ticket *widens* it to the profile, and the widening is the part that needs its own cost statement.

**Fact — one of six is compared today.** [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md) closed on `w_decode_kv` alone and states its own boundary: "This is one cell of six and one host row." Its non-goals name "the remaining five cells, which follow the first at no architectural cost."

**Fact — the grid-axis bound no longer blocks the rest.** [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md) moved the row to a measured 268,435,456 and proved by compiling that all six cells reach a selected physical plan (`tiler_build::metal_plan::tests::the_measured_grid_axis_admits_every_l3_contraction_cell`).

## What this owns, and what it leaves where it is

[`retain-contraction-conformance-evidence`](retain-contraction-conformance-evidence.md) proposes two halves and holds four scopes because neither half had a home. This ticket takes **one** of them:

- **Realization conformance — this ticket.** The L3 cells' retained `result_sha256` against the *executed* result on a matching machine row (five of six routable; sixth blocked outside this crate). **Correction — 2026-08-10.** This is an application of [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) Decision §1's ownership of cross-layer executed evidence, not a verbatim restatement of item 1's hard-requirement sentence. That hard requirement is: a host without the measured environment runs the deterministic half and **reports the measurement boundary as unavailable, naming what was missing** — never silent skip or claimed pass. The retained-digest program is adjacent to that sentence and instances the ownership; `implementation/conformance` is the scope that carries it.
- **Reference conformance — stays.** The adversarial cases against the reference evaluator are target-independent and already live in `crates/tiler-reference/tests/contraction_conformance.rs`, whose own header says that a pass there is evidence about the semantic contract and the host reference evaluator, and disclaims any schedule, kernel, device, or model tolerance. (Repaired 2026-08-07: this sentence previously presented that as a verbatim quotation, which it was not — the file's own words are "**What a pass here is not.** It is evidence about the semantic contract and the host reference evaluator." It also holds nine `#[test]`s rather than eight; `a_nan_the_reduction_forms_itself_is_canonicalized_too` is a ninth case.) Moving them would be the layer-local migration the crate's third anti-goal refuses.

**The coordinator should narrow `retain-contraction-conformance-evidence` to its reference half when this lands**, rather than leaving two owners for one deliverable. Do not close it from here.

## Cost, so the choice is stated rather than discovered

~~The comparison is a device dispatch per cell, not a host fold, so `tiler-reference`'s measured 1.1e9-step host cost does not apply.~~ **Correction — 2026-08-10.** That framing is false as a live claim. Device dispatch is nearly free; publication cost is dominated by the host oracle (`crates/tiler-conformance/src/publication/proof.rs` `reference_bits` / `ReferenceEvaluator`), so the ~1.1e9-step fold applies in full and is why the four prefill cells sit behind `#[ignore]` (Outcome: ~30.78 s / 323 MB peak RSS for those cells vs ~0.62 s for the rest of the crate). `w_decode_kv` folds 1,048,576 steps under the default evaluator bound; the largest two cells fold 402,653,184 each. Measure the wall clock per cell on the qualified host and state whether the whole profile runs on every gate run or whether the four prefill cells sit behind `#[ignore]` with a recorded invocation — the shape `crates/tiler-reference/tests/contraction_profile_cells.rs` already uses. Either answer is defensible; picking one silently is not.

## Required evidence

- Operands generated from the probe's own `SplitMix64` stream (`WORKLOAD_SEED = 0x5445_524D`, right seed `seed ^ 0xA5A5_A5A5_A5A5_A5A5`, values `m * 2^-24`), so each digest is computed over the bytes the device consumed. Read it from `crates/tiler-compiler/src/governed/contraction_conformance.rs` rather than re-deriving it.
- Every environment field compared against `spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/environment.tsv` **before** any comparison, with a non-matching row producing a named unavailable outcome rather than a skip or a pass.
- Each digest check watched refusing before it is trusted — a comparison against a 64-character constant passes trivially if the bytes never reach it. The `RIGHT_SEED_MASK` bit-flip perturbation in `publish-an-l3-contraction-cell-through-the-accepted-route` is the recorded technique.
- The measurement boundary recorded: host, OS build, Xcode, SDK, offline compiler, GPU, family.

## Closes when

Five of six cells are compared against executed results inside `crates/tiler-conformance` (the sixth, `w_vocab_slice`, is excluded by `MAX_PROOF_PAYLOAD_BYTES` and owned by [`decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights`](decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights.md)), the per-cell cost is measured and the run/ignore choice is stated with it, every comparison was watched refusing under a deliberate perturbation, a non-matching host *machine* row is observed declining by name, and the reference half is confirmed still resident in `crates/tiler-reference`.

## Per-Fact audit, re-read at base `d8913a9d`

Every source below was read in full at this base before any edit.

**Fact — "one of six is compared today." Verified.** `crates/tiler-conformance/src/envelope.rs`'s `CONTRACTION_MEMBERS` held two entries at base, one of them `L3_CELL_CLASS = "contraction-w-decode-kv"` carrying `retained_result_sha256: Some(L3_CELL_RESULT_SHA256)`, and `envelope/tests.rs`'s `only_the_l3_cell_is_compared_against_a_retained_measurement` asserted `compared == 1` by name. [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md) does state both quoted sentences verbatim.

**Fact — "the grid-axis bound no longer blocks the rest." Verified.** `crates/tiler-build/src/metal_plan.rs`'s `the_measured_grid_axis_admits_every_l3_contraction_cell` compiles all six cells to a selected plan and pins the boundary pair at `16_384 x 16_384` and one column past it; `crates/tiler-build/src/metal_declaration.rs`'s `grid_axis_threads: 268_435_456` is the measured row. The largest cell this ticket routes launches 393,216 threads, so the bound is nowhere near binding.

**Fact — "the reference half stays in `crates/tiler-reference`." Verified in substance; one quotation imprecise, one count wrong.** The file is resident and untouched by this branch, and it does disclaim schedule, kernel, device and model tolerance. The ticket quoted its header as *"A pass here is evidence about the semantic contract and the host reference evaluator"*; the file's own words are "**What a pass here is not.** It is evidence about the semantic contract and the host reference evaluator." That read as verbatim and was not. It also holds nine `#[test]`s, not eight. **Both repaired in "What this owns" above.**

**Fact — the cost statement. Two numbers verified, the framing false.** `w_decode_kv` at 1,048,576 steps, "the largest two cells fold 402,653,184 each" (`w_prefill_mlp_in`, `w_prefill_mlp_out`), and "the four prefill cells" all check out against the record's `workload.tsv`. **But "the comparison is a device dispatch per cell, not a host fold, so `tiler-reference`'s measured 1.1e9-step host cost does not apply" is false.** The device dispatch is nearly free — the whole routed contraction run cost 0.34 s at base. What costs is the *oracle*: `crates/tiler-conformance/src/publication/proof.rs`'s `reference_bits` derives each published expectation through `ReferenceEvaluator`, so the 1.1e9-step host fold applies in full and is the only thing that makes an `#[ignore]` necessary. Measured below.

**Fact — "`retain-contraction-conformance-evidence` proposes two halves and holds four scopes." Verified.** Its frontmatter is `scopes: [implementation/reference, implementation/compiler, contracts/numerics, research/scheduling]` and it is still `todo`.

**Fact — ADR 0106 item 1's hard requirement. Verified**, and the ADR's own 2026-08-07 supersession note names this ticket as the remaining migration item.

**Stale — the ambient artifact base.** The ticket predates [`produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate`](produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate.md), which is `done`: `TILER_CONFORMANCE_ARTIFACT_BASE` is retired and `crate::publication` writes every envelope in the run. Nothing in the ticket contradicts that; recorded so a reader does not go looking for the variable.

## Outcome — five of six cells routed; the sixth is blocked by another crate's public bound

Every edit is inside `crates/tiler-conformance/` and this ticket file.

**What is compared now.** `crates/tiler-conformance/src/envelope.rs` carries `L3_CORRECTNESS_CELLS` — all six cells with their extents, fold step counts, and retained `direct` digests — and `CONTRACTION_MEMBERS` routes the adversarial `2x2x3` member plus five of them. Each routed cell publishes its own envelope and sidecar in the run, dispatches on the device, and has the SHA-256 of its **executed** bytes compared against the retained measurement.

**Measurement — Apple M4 Max, macOS 27.0 `26A5388g`, Apple9, offline compiler `Apple metal version 32023.921 (metalfe-32023.921)`, `metallib` AIR-LLD `32023.921`, SDK macosx 27.0 `26A5388f`, `arm64`, test profile.** All five routed cells reproduce their retained `direct` digests exactly:

| cell | extents | executed SHA-256 |
| --- | --- | --- |
| `w_decode_kv` | `1x1024x1024` | `79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f` |
| `w_prefill_q` | `10x2048x1024` | `1c54f5cd7265ee288ec79bcd9254243b78a95d57c3c489e5ea90bcc4298073c0` |
| `w_prefill_mlp_in` | `128x3072x1024` | `eb382840ac9e533f57e51a0ffed2d61608664ecc5869aaa9f93afa3c312696a0` |
| `w_prefill_mlp_out` | `128x1024x3072` | `124571de47ebff2f152b120afc9944b3465bffe94d8ac283a077677f61feb5f5` |
| `w_prefill_o` | `128x1024x2048` | `b99eff9042d9e4b25e3844ff0462e5e6303e57b146aa79400622885bffc5f2f6` |

**`w_vocab_slice` cannot be published at all, and the bound belongs to another crate.** `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES` is `pub` and is 16,777,216; the cell's `[8192, 1024]` weights operand is 33,554,432 bytes — exactly twice it. Observed as `Limit(ProofLimitExceeded { kind: PayloadBytes, attempted: 33554432, limit: 16777216 })` out of `ProofSidecarBuilder`. `crates/tiler-artifact/**` is outside `implementation/conformance`, and splitting the operand across cases would publish a different program rather than the cell the digest describes — so the cell is pinned in the table, excluded from the routed set by `L3CorrectnessCell::fits_one_proof_payload`, and held to that arithmetic by `envelope::tests::the_unpublishable_cell_is_named_against_the_bound_that_stops_it`. **This needs a decision in `tiler-artifact` and is not taken here.**

**The run/ignore choice, stated with its measurement.** The line is a *property* rather than a budget: publishing a cell needs the reference's expected bytes, and a fold above the evaluator's per-occurrence bound is reachable only by a caller stating a larger number. `w_decode_kv` folds 1,048,576 steps, under the bound, so the ordinary gate publishes it through exactly the evaluator every other consumer gets and authorizes nothing extra. The four prefill cells need a stated allowance and fold 1,094,713,344 steps between them; they are `envelope::tests::the_prefill_cells_carry_their_retained_digests`, `#[ignore]`d, **measured at 30.78 s wall clock and a 323 MB peak resident set** (`/usr/bin/time -l`) against 0.62 s for the whole rest of the crate. Invocation:

```sh
cargo nextest run -p tiler-conformance --run-ignored only --no-capture \
    -E 'test(the_prefill_cells_carry_their_retained_digests)'
```

**The environment row is compared before any digest is, which nothing did before.** `crates/tiler-conformance/src/retained_record.rs` reads the record's own `environment.tsv` and `workload.tsv`. Six fields are compared against the observed `MeasurementBoundary` — device, GPU family, architecture, OS, offline compiler, SDK — and every difference is printed by name on every routed run. On this host two differ:

```text
retained row: this host differs from the retained record's row in 2 of 6 compared field(s) —
offline-compiler: record "Apple metal version 32023.883 (metalfe-32023.883)", this host "Apple metal version 32023.921 (metalfe-32023.921)";
sdk: record "macosx 26.5 build 25F70", this host "macosx 27.0 build 26A5388f"
```

**A deviation from this ticket's required evidence, stated rather than absorbed.** The ticket asks that *any* non-matching field produce "a named unavailable outcome rather than a skip or a pass". Taken literally that makes the comparison permanently unmade: the record's toolchain row is already unreachable — this repository's own qualified toolchain has moved past SDK 26.5 and metal `32023.883` — so **no currently reachable host is on the record's row**, and every run would report a boundary instead of comparing. That turns the one executed cross-workspace check in this crate back into a document, which is what this ticket exists to undo. What is implemented instead: a difference in the *machine* (device name, GPU family) declines the retained comparison by name while the member still routes and is still compared bit-for-bit against its published reference; a difference in the *toolchain* is announced and the comparison proceeds, so a toolchain that did move the bits goes red with the row printed beside the three digests, which is the correctness finding the record was retained to produce. **Tom's call whether that split is right** — it is one line of `hardware` flags in `retained_record::compare`. **Correction — 2026-08-10.** No written Tom acceptance of this hardware/toolchain split was found under tickets or docs at audit base `c99ac54950f2`. The open product call remains unsplit on this `done` ticket; filing a narrow decision ticket for the policy is residual work outside this ticket-only repair wave.

**The retained digests are now a checked transcription.** `envelope::tests::the_pinned_cells_are_the_retained_records_own_direct_rows` reads the record's `workload.tsv`, filters the `direct` realization by column *name*, and compares all six cells' extents and digests against the source constants — device-free, so it holds on every host including ones that can never measure.

**Every new comparison was watched refusing, by perturbing its subject.**

1. *The per-cell digest.* Flipping the last-but-one nibble of `w_prefill_mlp_in`'s pinned digest failed the device-free record cross-check (`left: …96a0, right: …96b0`) and failed the routed run independently: `contraction-w-prefill-mlp-in case "probe-workload": the SHA-256 of the executed result bytes is …96a0 and the retained realization-probe measurement is …96b0; … This is a correctness finding about the contraction vertical, not a dispatch failure`. Reverted.
2. *The row decline, made to fire on the real route.* Reclassifying `offline-compiler` as a hardware field made this host's genuine toolchain difference trip the decline inside the routed gate run: `contraction-w-decode-kv: … the retained digest was measured on other hardware, so this run declines to compare against it: offline-compiler: record "…32023.883…", this host "…32023.921…"`, with the member still routing and still agreeing with its published reference. That is the decline path exercised end to end rather than left as dead code. Reverted.
3. *The payload-bound exclusion.* Shrinking `w_vocab_slice`'s `n` from 8192 to 4096 failed three tests at once — the routed set no longer matched the derived publishable set, the doubling assertion broke (`left: 16777216, right: 33554432`), and the record cross-check reported extents the record never measured. So the exclusion is derived from the bound rather than hand-asserted. Reverted.

Perturbation 2 also **found a real defect, now fixed**: a declined comparison printed all 1,024 result elements, because the elision keyed on whether a retained comparison existed rather than on the result's size.

**Population, and the floor moved with it.** The crate goes 57 → 68 tests, of which 65 are device-free (3 stay in `dispatch`). `portability.rs`'s `DEVICE_FREE_TEST_FLOOR` moves **53 → 64, with the population and by the same eleven**, which is what preserves its documented two-test sensitivity: the three smallest device-free modules hold two tests each, so gating any one of them must drop below the floor. The per-module counts and that reasoning are written into the constant's doc comment. It was not moved to make anything pass. The unsafe-site census is untouched at two sites, and `lints.rs` is untouched.

**The reference half is confirmed resident.** `git diff --name-only d8913a9d..HEAD -- crates/tiler-reference/` is empty; `contraction_conformance.rs` and `contraction_profile_cells.rs` are byte-identical to base.

**Checks.** `cargo fmt --all --check`; `cargo clippy -p tiler-conformance --all-targets -- -D warnings`; `cargo nextest run -p tiler-conformance` → **67 passed, 1 skipped**, the one skipped being the deliberate `#[ignore]`, run separately and green in 30.16 s; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-conformance`; `cargo test -p tiler-conformance --doc` → 0 tests. `tkt lint` ok; `make citations` resolved 261 pinned citations; `git diff --check` clean; `tkt guard … --base d8913a9d` reports affected scopes exactly equal to declared (`implementation/conformance`, `project/tickets`), every direct collision being with a `done` ticket.

**Caveat on rustdoc:** every module in this crate is `#[cfg(test)]`, so `cargo doc` documents `lib.rs`'s header and nothing else. It cannot fail for any code this ticket changed and is not coverage.

**The manual non-Apple check was re-run**, which the crate header requires whenever a module is added and this ticket added `retained_record`. Both pass:

```sh
cargo check  -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu
cargo clippy -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
```

`retained_record` carries no macOS gate, so all four of its tests are in the device-free population and the census counts them there.

### What this ticket does not close

- **`w_vocab_slice` is uncompared**, blocked on `tiler_artifact::proof::MAX_PROOF_PAYLOAD_BYTES`. Needs its own ticket holding the artifact scope.
- **The row-difference policy** above is a worker's reasoned choice on evidence this ticket predates, and is Tom's to confirm. **Correction — 2026-08-10.** Still no written acceptance or filed decision ticket for that policy (see Outcome deviation note); residual graph work, not reopened implementation.
- `retain-contraction-conformance-evidence` still needs narrowing to its reference half by the coordinator; not closed from here.

## Outcome — done, 2026-08-07

Landed at merge **`2fb699a9`** (worker commit `b26d407f`). 11 files, +1,821/−180. `make full` exit 0 on the merged tree, 1,090 release tests. `crates/tiler-reference/` is **byte-identical to base** — coordinator-confirmed by an empty `git diff --name-only`.

**Five of six L3 contraction cells now compare executed device bytes** against the retained `direct` digests, all reproducing exactly on Apple M4 Max / macOS 27.0 `26A5388g` / Apple9 / metal `32023.921`: `w_decode_kv`, `w_prefill_q`, `w_prefill_mlp_in`, `w_prefill_mlp_out`, `w_prefill_o`.

### The coordinator's brief was for a different ticket

My brief described `from_realization`, `ConformanceSubject`, and `bf16_vertical::conformance_of` as "the shape this ticket generalizes". **This ticket is the L3 `result_sha256` device comparison and touches none of that** — the brief belonged to `route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge`. The worker noticed, worked from the ticket, and said so. Exactly right.

### The cost claim in this ticket was false, and the correction matters

The ticket framed the cost as "a device dispatch per cell, not a host fold, so the 1.1e9-step host cost does not apply". **The dispatch is nearly free** — the whole routed run was 0.34 s at base. The cost is the **oracle**: `publication/proof.rs::reference_bits` evaluates every published expectation through `ReferenceEvaluator`, so the 1.1e9-step fold applies in full and is the only reason an `#[ignore]` is needed at all. Measured: **30.78 s and 323 MB peak RSS** for the four prefill cells, against 0.62 s for the rest of the crate. `w_decode_kv` folds under the reference's own per-occurrence bound, so the gate publishes it through the unmodified evaluator and authorizes nothing.

Two smaller repairs: the quoted `tiler-reference` header sentence was **not verbatim**, and that file holds **nine** `#[test]`s rather than eight. `TILER_CONFORMANCE_ARTIFACT_BASE` is stale, retired when envelope production moved in-process.

### A deliberate deviation from the ticket, and it was the right call

The ticket required any non-matching environment row to make the member "named unavailable". **Implemented literally that check is permanently dead**: the record's toolchain row is SDK 26.5 / metal `32023.883`, and this host resolves SDK 27.0 / metal `32023.921` — coordinator-verified with `xcrun metal --version` and `xcrun --show-sdk-version` — so *no* current host matches and every run would report a boundary instead of comparing.

The split landed instead: a difference in the **machine** (device, GPU family) declines by name while the member still routes and still compares against its published reference; a difference in the **toolchain** is announced and the comparison proceeds. Perturbation 2 proved the decline path is **live rather than dead code**, by reclassifying `offline-compiler` as a hardware field and watching this host's genuine toolchain difference trip the decline inside a real routed run — which also **found and fixed a real defect**, a declined comparison printing all 1,024 result elements because elision keyed on the comparison rather than the result size.

### Population and floor moved together

57 → 68 tests, 65 device-free. `DEVICE_FREE_TEST_FLOOR` moved **53 → 64, by the same eleven**, preserving the documented two-test sensitivity, with per-module counts written into the constant. Not moved to make anything pass. The unsafe census and `lints.rs` are untouched.

`retained_record.rs` now reads the spike's `environment.tsv` and `workload.tsv`, so the six pinned digests are a **checked transcription** on every host rather than a hand copy, and every row difference is printed by name before any digest is compared.

### Two things filed rather than absorbed

`w_vocab_slice` cannot route: its `[8192, 1024]` weights operand is 33,554,432 bytes against a `pub` `MAX_PROOF_PAYLOAD_BYTES` of 16,777,216 — **exactly a factor of two**, coordinator-verified. The exclusion is *derived* from that constant and pinned to the doubling, so it cannot be quietly edited. Filed as `decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights`.

And the publication path's oracle still runs under `strict()`, whose subject is `Unstated`, while artifacts compile under `FLUSH_SUBNORMALS_TO_ZERO_F32`. Unobservable on these operands — the probe stream is `m·2⁻²⁴` — but a genuine asymmetry, and closing it needs a `VerifiedScheduledRegion` the contraction path does not hold. Filed as `state-a-subject-on-the-contraction-publication-path-s-reference-oracle`.

## Fact audit — 2026-08-10

Ticket-only repair from audit report `docs/research/documentation/ticket-audit-2026-08-10/reports/route-the-realization-conformance-half-into-the-conformance-crate/add740b427b4_c99ac54950f2.md` (audit base `c99ac54950f2`). Status stays `done`; no implementation reopened.

**Live close boundary.** User-visible outcome and Closes when no longer claim all six cells. Five of six L3 cells route with retained digests (`CONTRACTION_MEMBERS` / `l3_member(0..4)`); `w_vocab_slice` remains pinned and unpublishable under `MAX_PROOF_PAYLOAD_BYTES`. Frontmatter `related` now includes `decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights` (awaiting-decision) and historical carrier `state-a-subject-on-the-contraction-publication-path-s-reference-oracle` (`done`).

**Cost framing.** Cost section's "device dispatch per cell, not a host fold" sentence is struck; Outcome and Per-Fact cost corrections remain authoritative — oracle fold cost is the gate cost.

**ADR 0106 wording.** "Verbatim item 1 hard requirement" softened to item 1 ownership vs the hard-requirement sentence (named unavailable measurement boundary).

**Row-difference policy.** Still Tom's unsplit product call; no written acceptance found; decision ticket not filed in this wave (blocked residual for a later filing decision).
