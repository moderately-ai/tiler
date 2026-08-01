---
id: avoid-toolchain-resolution-on-a-warm-expansion-cache-hit
title: Decide whether a warm expansion-cache hit may resolve the Apple toolchain
status: done
priority: p2
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/frontend, implementation/metal-aot]
shared_scopes: [project/tickets, contracts/integrations]
paths: []
tags: [research, performance, inline-dx]
---
## Why this exists

`docs/integration/frontends.md` states that "warm IDE and `cargo check` expansion must avoid `xcrun`". The landed expansion-time AOT flow does not, and the reason is structural rather than an oversight.

**Measurement — macOS 27.0 arm64, `nightly-2026-07-19`, 2026-07-31.** An out-of-tree consumer crate declaring only `tiler`, holding one `deliver macos;` region, built against a private cache root with an `xcrun` shim first on `PATH` that logs every invocation and returns logging wrappers for `metal` and `metallib`:

| pass | `xcrun` calls | `metal`/`metallib` runs | cache |
| --- | ---: | ---: | --- |
| cold root, source touched | 6 | 2 | one bundle published |
| warm root, source touched | 6 | 0 | validated hit |

The six are `--find metal`, `--find metallib`, `--show-sdk-path`, `--show-sdk-version`, `--show-sdk-build-version`, and a second `--show-sdk-path`.

**Fact.** `Toolchain::prepare` performs all six before returning a `PreparedCompilation`, and `CompilationIdentity::new(request, &resolved)` folds the resolved tool versions and SDK identity into the identity the cache is keyed on. Only `prepared.compile()` — the `metal` and `metallib` runs — is inside the miss closure. The resolution therefore *precedes* the lookup by construction: the fingerprint is an input to the key that decides hit or miss.

**Inference.** Skipping the resolution on a warm expansion means deriving the key from something other than the installed toolchain, which would let a hit return an artifact built by a compiler that is no longer present. The contract sentence and the identity requirement are in tension, and which gives way is not a worker's to pick.

## Closes when

One of: the contract sentence is corrected to say what a warm expansion may do, with the identity argument recorded; or a cached toolchain fingerprint with its own validation and invalidation rule is designed and accepted; or the question is deferred with a measured trigger — `rust-analyzer` cold/warm cost remains unmeasured because the component was unavailable.

The six-call figure above is the baseline any remedy is measured against.

## Outcome (worker `worker-warm-hit`, base `a37be43`, 2026-08-01)

**The contract sentence is corrected, and the two mechanism designs are eliminated rather than deferred.** `docs/integration/frontends.md` now states what a warm expansion may do, why identity requires it, and what it costs; the derivation below is what the correction rests on. The ticket's own deferral condition — "`rust-analyzer` cold/warm cost remains unmeasured because the component was unavailable" — is discharged: the component is available on this host and the live-session cost is measured here for the first time.

The correction is **not** "the cost is negligible". It is not negligible. It survives because the sentence's goal is unreachable as stated *and* because the sentence misidentifies where the cost is.

### Correction to this ticket's own baseline: five calls, not six

**Measurement — macOS 27.0 arm64, Apple M4 Max, `nightly-2026-07-19`, 2026-08-01.** The same out-of-tree consumer crate, private per-user cache root under `$TMPDIR`, `xcrun` shim first on `PATH` logging every invocation with its parent process:

| command | `xcrun` calls | attribution |
| --- | ---: | --- |
| warm `cargo check`, source touched | **5** | all five from the `rustc` process running the proc macro |
| warm `cargo build`, source touched | **6** | the same five, then one `--show-sdk-path` at link time |

The sixth call this ticket recorded is **rustc's own linker SDK query**, not Tiler's, which is why it appears under `cargo build` and vanishes under `cargo check` while the expansion is identical. No Tiler change removes it.

The five that *are* Tiler's are `--find metal`, `--find metallib`, `--show-sdk-path`, `--show-sdk-version`, and `--show-sdk-build-version`. `Toolchain::resolve` additionally executes the two located binaries directly to read their reported versions — deliberately *not* through `xcrun`, per `driver.rs:89-93`, so the folded version describes the binary that will produce the bytes. Those two executions are invisible to an `xcrun` shim, which is why the original transcript missed them, and they are the **two most expensive components**.

### What the observation costs

**Measurement — same host and date.** In-process timing of the real `Toolchain::resolve`, and of each component subprocess in the same loop; three repetitions of n=40 after an untimed warm-up. Measured through a scratch binary depending on `tiler-metal-aot` by path; the crate builds under the host default toolchain there rather than the repository pin, which changes nothing measured, because every figure is subprocess wall time.

| component | median | mean | range over 3×40 |
| --- | ---: | ---: | --- |
| `Toolchain::resolve()` **whole** | **52–63 ms** | **53–82 ms** | 44.3 – 96.8 |
| `xcrun --find metal` | 5.9–6.2 | 6.3–6.9 | 4.7 – 20.7 |
| `xcrun --find metallib` | 5.5–6.3 | 5.8–6.5 | 4.8 – 15.1 |
| `metal --version` (direct) | 11.8–13.0 | 12.2–15.1 | 10.1 – 31.9 |
| `metallib --version` (direct) | 9.5–12.4 | 10.4–15.6 | 7.9 – 58.9 |
| `xcrun --show-sdk-path` | 5.7–19.4 | 6.0–19.2 | 5.0 – 31.8 |
| `xcrun --show-sdk-version` | 5.8–19.6 | 6.1–22.8 | 4.7 – 80.2 |
| `xcrun --show-sdk-build-version` | 5.4–19.2 | 5.6–20.3 | 4.8 – 28.4 |
| `/usr/bin/true` (bare spawn floor) | 1.2–2.8 | 1.2–3.2 | 1.0 – 10.9 |

The third repetition ran under load and drifts the SDK rows upward; the first two agree closely. Read the medians of the first two repetitions as the figure and the spread as the noise.

**The two `--version` executions are ~24 ms of a ~58 ms resolution** — about 40% — and they are the only part that observes the toolchain at all. See below.

### The denominators

**Measurement — batch.** Warm `cargo check` of the one-region consumer, `touch src/main.rs` between passes, six passes: **0.17–0.19 s wall each**, five `xcrun` calls each. Delivering region versus the same file with the `deliver macos;` statement removed (n=8 each, two interleaved runs): 271 vs 146 ms, and 307 vs 213 ms. The delivering region's whole expansion overhead is therefore ~90–125 ms, of which the resolution is ~50–80 ms.

**Measurement — live rust-analyzer, the gap this ticket named.** `rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)` from the `nightly` toolchain, driven over a real LSP session (initialize, didOpen, then didChange edits each followed by a `textDocument/semanticTokens/full` round trip), with the pinned toolchain's `rust-analyzer-proc-macro-srv` and the counting shim. Nothing was installed for this: the analyzer binary and the proc-macro server were both already present. Expansions are counted exactly, because a resolution is exactly five shim lines.

| event | expansions | `semanticTokens` round trip |
| --- | ---: | ---: |
| edit **inside** the region | 1, 1, 1 (3 of 3, both runs) | 137–217 ms |
| the same edits, region with no `deliver` statement | 0, 0, 0 | 10–16 ms |
| edit **outside** the region | 1 of 2 observed, each run | 15–188 ms |

`analysis-stats` independently confirms the flow runs inside the analyzer: one batch analysis produced exactly five `xcrun` calls, parented to `rust-analyzer-proc-macro-srv`.

**So on the interactive path the toolchain resolution is roughly 30–45% of a ~175 ms round trip** — the largest single identifiable component, but not a majority; Tiler's own optimize, emit, assemble, and cache-validation work is the rest. A delivering region makes in-region editing about 10–15× slower to analyze than a fallback-only one.

### The finding that decides the design question

**Fact — `xcrun` is itself a cache, and Tiler has been reading it.** `xcrun(1)` documents `-n/--no-cache` ("Don't consult the cache when looking up values") and `-k/--kill-cache`. `xcrun --verbose --sdk macosx --find metal` names the store and the key:

```
xcrun: note: database key is: metal|…/MacOSX26.5.sdk||/Applications/Xcode.app/Contents/Developer|
xcrun: note: lookup resolved in '$TMPDIR/xcrun_db' : /var/run/com.apple.security.cryptexd/mnt/com.apple.MobileAsset.MetalToolchain-v17.6.109.0.IaP0Ob/Metal.xctoolchain/usr/bin/metal
```

Reading that 41,673-byte `xcrun_db` shows all five of Tiler's queries are cached keys — the two `--find` results, and `<sdk>|<DEVELOPER_DIR>|<sdklookup>|{Path,SDKVersion,ProductBuildVersion}` — together with a `<resolved path>|<toolchain-signature>` entry per tool holding `1779476302|1779476302`.

**That signature is a stat-class witness, verified rather than assumed.** `1779476302` is 2026-05-22 14:58:22, and `stat -f '%Sm'` reports exactly `May 22 14:58:22 2026` for both the cryptex mount directory and the `metal` binary inside it. So Apple's cache already carries the same witness class a Tiler-side fingerprint cache would have to use — a timestamp over the resolved tool — which is the concrete form of the elimination below: a second cache could match that witness and not improve on it, because it would be reading the same facts about the same files.

**Measurement — what the cache is worth.** `xcrun --no-cache --sdk macosx --find metal`, 20 iterations: **66.67 s total, ~3.33 s per call**, against ~6 ms cached. A ratio of roughly **550×**.

**Inference — the identity argument in this ticket's "Why this exists" is right in direction and overstated in strength.** Re-running the five `xcrun` queries each expansion does not observe the installed toolchain; it re-reads Apple's cache, whose invalidation rule is Apple's and is not documented. The part of `Toolchain::resolve` that observes the toolchain that will actually run is the two direct `--version` executions — and those are not `xcrun` calls, so the contract sentence would not have covered them even if it had been honoured.

The invariant is therefore narrower and stronger than "resolve every time": **identity must fold a fingerprint read by executing the binaries the same prepared token will execute.** `PreparedCompilation` already guarantees exactly that structurally, and `a_changed_tool_selection_fails_closed_rather_than_misattributing` is the test that holds it. The consequence that makes a stale observation harmless *between* processes: a stale fingerprint yields a stale key, and a stale key cannot collide with a fresh one, so no fresh build is ever served an entry keyed on a toolchain it did not itself resolve. Completeness of the key is what buys that, not the schedule of the resolution.

### The elimination

**A cross-process cached toolchain fingerprint — rejected.** It would be a second cache layered directly on `xcrun_db`, and its witness could be no better than the vendor's, because it would read the same selection inputs: `DEVELOPER_DIR`, `SDKROOT`, `TOOLCHAINS`, plus file identity of the resolved tools. It would fail closed exactly where Apple's cache does and fail open exactly where Apple's cache does — a second Metal toolchain asset preferred while the old one remains mounted defeats both, because neither key names the asset set. What it would add is a new on-disk cache carrying the corpus's full obligations (complete identity, validation on every hit, immutable entries, atomic publication, defined crash and race behaviour) inside a crate whose dependency closure is pinned empty by ADR 0077 item 2 and which therefore owns no digest to key one with. The saving is ~30 ms of the five `xcrun` calls; reaching the sentence's actual goal would additionally require caching the two `--version` answers, which are the only live observation of the compiler that will run. A new cache authority, in the crate least able to host one, to remove a cache read.

**A process-lifetime memoized resolution — rejected.** It cannot satisfy the sentence's `cargo check` half at all: each check is a fresh `rustc` process that expands once, so a per-process memo saves nothing there, which the five-calls-per-check measurement above shows directly. In a long-lived proc-macro server it *would* satisfy the IDE half — turning one resolution per edit into one per session — but at the cost of widening the window between observing a toolchain and executing it from one expansion to one multi-hour session, which is the window `a_changed_tool_selection_fails_closed_rather_than_misattributing` exists to keep narrow. It would also need keying by launcher path to avoid returning the system resolution to the fake-launcher fixtures in `driver.rs`, `metal_plan.rs`, `metal_payload.rs`, and `metal_assembly.rs`, and it would falsify `Toolchain::resolve`'s documented "preflight every compilation runs first".

**Deferral — rejected.** This ticket's stated deferral condition was that rust-analyzer cost was unmeasured because the component was unavailable. It is available and is now measured, so deferring would park a question the evidence answers.

**Surviving: correct the sentence.** Done, in `docs/integration/frontends.md`, with the identity derivation, the `xcrun_db` finding, the measured table, and both eliminated designs recorded so they are not re-proposed without new evidence.

### Reconsideration trigger

Reopen the cross-process fingerprint question if either holds: a consumer crate carrying several delivering regions measures the resolution above half of its warm `cargo check` wall time (today one region is ~30% of ~180 ms); or Apple documents `xcrun`'s cache invalidation rule, which would make a Tiler-side witness able to be *strictly* sounder than the vendor's rather than merely equal to it. Reopen the memoization question only alongside a decision that one compilation unit must observe one toolchain — which is a real latent incoherence (two regions in one crate today take two independent observations) and is a correctness question, not a performance one.

### Filed, not absorbed

`drop-the-unread-sdk-path-from-the-resolved-toolchain` — `--show-sdk-path` populates `SdkIdentity::path`, which compilation identity excludes by construction, the artifact payload does not carry, and no compiler or linker flag reads. It is the only part of the resolution that buys nothing; removing it is a public-field removal and therefore Tom's under ADR 0075, which is why it was filed rather than applied.

**Landed 2026-08-01.** Tom accepted the removal, and `drop-the-unread-sdk-path-from-the-resolved-toolchain` carries it. Every measurement in this outcome describes the five-call resolution as it stood at `a37be43` and stays as measured; a resolution now makes four `xcrun` calls and two `--version` executions. Nothing here needs re-deriving: the removed call was the one component that reached no identity, so the derivation, the `xcrun_db` finding, and both eliminated designs are unaffected.

### Changes

1. `docs/integration/frontends.md` — the third rust-analyzer bullet is corrected, and a new "Why a warm expansion resolves the toolchain" subsection carries the derivation, the measurements, and the eliminations.
2. `crates/tiler-macros/src/aot.rs` — the module doc no longer cites the superseded sentence, no longer describes the `--version` reads as `xcrun` invocations (they are not, and `driver.rs` says so), and states the invariant in its corrected form.
3. `tickets/prototype-inline-aot-integration-proof.md` — its measurement-boundary paragraph is corrected where it over-attributes the sixth call and describes the tension as unresolved.
4. This outcome.

**No code behaviour changed and no public boundary moved.** The two source edits are documentation; the resolution still performs exactly the five `xcrun` calls and two `--version` executions it did at `a37be43`.
