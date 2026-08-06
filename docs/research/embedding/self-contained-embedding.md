---
schema: "tiler-doc/v1"
id: "tiler.research.embedding.self-contained"
kind: "research"
title: "Self-contained byte-literal embedding under Cargo and rust-analyzer"
topics: ["embedding", "proc-macros", "cache", "cargo", "rust-analyzer", "artifacts"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.frontend-integration", "tiler.contract.artifact-abi"]
depends_on: ["tiler.research.embedding.artifact-costs", "tiler.research.cache.build-tool-exercise"]
ticket: "prototype-macro-embedding-and-cargo-behavior"
---

# Self-contained byte-literal embedding under Cargo and rust-analyzer

[The embedded-artifact cost note](embedded-artifact-costs.md) decided the representation — one proc-macro byte-string literal per payload, never one integer literal per byte — and measured what it costs in build time, peak RSS, binary size, and retained copies. It did not establish the property the representation was chosen *for*. [ADR 0004](../../decisions/0004-inline-macro-aot-bundles.md) makes each inline invocation a self-contained AOT bundle, and "self-contained" is a claim about what the expanded code *reaches*, which no size measurement can settle. This note settles it, and measures how the two build tools behave around it.

The harness is [`spikes/embedding/self_contained.py`](../../../spikes/embedding/self_contained.py) over the Cargo workspace at `spikes/embedding/self-contained/`, whose proc macro resolves a real artifact envelope through the real public [`ExpansionCache`](../../../crates/tiler-cache/src/expansion/store.rs) and emits it as one [`Literal::byte_string`] token. Recorded results are [`self-contained-embedding-macos-27.0-2026-07-31.tsv`](../../../spikes/embedding/results/self-contained-embedding-macos-27.0-2026-07-31.tsv) (15 rows) and [`self-contained-diagnostics-macos-27.0-2026-07-31.txt`](../../../spikes/embedding/results/self-contained-diagnostics-macos-27.0-2026-07-31.txt) (7 failure classes, verbatim).

## The headline, before the evidence

**Measurement.** A consumer built and ran with every Tiler-produced file deleted from the filesystem — all twelve envelopes and sidecars, and the whole cache root, sixteen files in total — and printed the exact length and checksum of the artifact the producer had written. The crate it ran from declares no dependency at all, names no proc macro, and contains no `include_bytes!`, no path, and no build script. The payload travels as one byte-string literal of 68,076 source bytes carrying 36,838 payload bytes.

**Measurement.** Deleting the entire expansion-cache root after expansion changed nothing: zero expansions ran, the build succeeded, and the program printed the same bytes. This is the load-bearing half — if a deleted cache had broken the build, the embedding would have been a reference wearing a literal's clothes.

**Measurement, and the result that most changes how the cache should be read.** The two drivers agree on every axis, and the *reason* they agree is that the expansion cache's subject is narrower than either tool's freshness notion. A source edit re-expands under Cargo and reads nothing from disk; a **toolchain change re-expands under both drivers and also reads nothing**, because Cargo's fingerprint carries the compiler and the cache subject does not. Two crates embedding the same artifact produce two expansions and **one** envelope read; two crates embedding different artifacts produce two of each.

**Fact.** This is spike-side evidence about a representation, not a measurement of the production frontend. `tiler::tensor!` states [`ArtifactDeliveryPolicy::FallbackOnly`](../../../crates/tiler-macros/src/delivery.rs), which [ADR 0053](../../decisions/0053-gate-artifact-delivery-by-consumer-family.md) defines as invoking no backend compiler; it embeds no bytes and opens no cache. [`cache_root.rs`](../../../crates/tiler-macros/src/cache_root.rs) remains crate-private, but it is no longer uncalled: the delivering expansion path (`tiler_macros::aot::deliver`) resolves the root through it and opens the expansion cache, so the cache-side behaviour this note measures now has a production caller on the `deliver` route while the `tensor!` fallback route still opens nothing. Section 7 states what would move the embedding half.

## 1. A carried compiled payload, which is new here

**Fact.** [The build-tool exercise](../cache/build-tool-exercise.md) listed "a carried compiled payload" among the cases it did not reach: "the envelope declares its payload by descriptor rather than carrying object bytes", so no compiled backend object had ever travelled through a cache entry. This spike closes that.

**Measurement.** The envelopes come from `prototypes/serial-sum-compile`, which drives `tiler-build` through deterministic MSL emission and `xcrun`, and writes six proof members carrying genuine compiled `metallib` objects. On the recorded host the authoritative profile is `tiler.metal.macos-apple9.msl4-0.f32.v1`, AOT target `air64-apple-macos26.0` under `metal4.0`, deployment minimum macOS 26.0.

| Member | Entries | `metallib` bytes | Envelope bytes |
| --- | ---: | ---: | ---: |
| `empty-domain.selected` | 1 | 3,491 | 32,136 |
| `empty-domain.materialized` | 2 | 6,662 | 45,683 |
| `singleton.selected` | 1 | 3,603 | 34,030 |
| `singleton.materialized` | 2 | 7,078 | 46,445 |
| `nontrivial.selected` | 1 | 3,763 | 36,838 |
| `nontrivial.materialized` | 2 | 7,158 | 47,803 |

**Inference.** Every cache hit in the rows below was validated by the real `decode_artifact` against bytes carrying a real compiled Metal object, so the validation this note relies on is the production validator over production-shaped input rather than over a descriptor.

### The band, re-derived 2026-08-06 — the envelope column moved and the `metallib` column did not

The six rows above are a **measured claim about the encoding of 2026-07-31**, and [`spikes/cache/hot-path-efficiency`](../cache/hot-path-efficiency.md) borrowed their endpoints as the sizes it prices a cache hit against. That spike stopped running when one envelope's fixed overhead rose above both of them, so [`re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps`](../../../tickets/re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md) re-derived the band the way it was derived here: run the producer, read the envelope length of every member it publishes, take the minimum and the maximum. The 2026-07-31 rows above are left exactly as they were, because they are evidence taken at their own commit.

**Measurement.** `cargo run -p tiler-prototype-compile --release -- --out <dir>/serial-sum` at `8bd720b8` on 2026-08-06, same host, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, `MANIFEST_SCHEMA` `14.0`. The producer now publishes **eight** members rather than six: the three reduction classes times two plan roles, plus a `2x2x3` contraction and the L3 `w_decode_kv` cell, neither of which existed when the band above was taken.

| Member | Entries | `metallib` bytes | Envelope bytes, 2026-07-31 | Envelope bytes, 2026-08-06 |
| --- | ---: | ---: | ---: | ---: |
| `empty-domain.selected` | 1 | 3,491 | 32,136 | 143,106 |
| `empty-domain.materialized` | 2 | 6,662 | 45,683 | 158,401 |
| `singleton.selected` | 1 | 3,603 | 34,030 | 141,532 |
| `singleton.materialized` | 2 | 7,078 | 46,445 | 155,695 |
| `nontrivial.selected` | 1 | 3,763 | 36,838 | 146,324 |
| `nontrivial.materialized` | 2 | 7,158 | 47,803 | 159,037 |
| `contraction.selected` | 1 | 3,891 | — | 90,737 |
| `contraction-w-decode-kv.selected` | 1 | 3,891 | — | 89,250 |

**Measurement, and the single most informative cell in the table.** Every one of the six `metallib` counts is **byte-identical** to the count recorded on 2026-07-31. The compiled objects did not move at all, so no part of the envelope growth is backend output, an optimization-level change, or a Metal toolchain difference — it is entirely artifact encoding, and this is a measured attribution rather than an inferred one. The authoritative profile did move, from `tiler.metal.macos-apple9.msl4-0.f32.v1` to `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` with a 1,999-byte canonical descriptor, which is the BF16 widening of the total-map vocabularies landing in the identity every manifest carries.

**Measurement — where the bytes are, at this commit.** Parsing the framing header and the section table of each published envelope (magic, then the 8-byte total, the 8-byte manifest length, and the 4-byte section count, then the `4 + 8` framing preceding each section's bytes) attributes them exactly. For `nontrivial.materialized`, the largest member: 159,037 bytes are a 69-byte header, a **121,413-byte canonical manifest** (76.3%), a 23,741-byte `KernelProgramSubject` section (14.9%), a 6,620-byte `BackendPayloadMetadata` section (4.2%), the 7,158-byte `BackendPayloadCode` section carrying the `metallib` (4.5%), and 36 bytes of section framing. The carried compiled object is **under a twentieth** of the envelope that carries it; the canonical manifest and the packaged kernel program's canonical identity are the envelope.

**Inference — the band, and which band a consumer should take.** Over the population the 2026-07-31 rows measured, the six reduction-class members, the band is now **141,532 to 159,037 bytes**. Over everything this producer publishes today it is **89,250 to 159,037 bytes**, and the two contraction members are the low end because a contraction packages a smaller kernel program than a two-stage materialized reduction, not because it carries less object. A consumer that wants "the sizes this slice's producer emits" should take the eight-member band; a consumer that wants to reproduce a cost at these lengths has to check that its own producer can reach them, which is exactly what stopped the hot-path sweep — see [that note's Section 1](../cache/hot-path-efficiency.md#1-environment-and-procedure).

**Measurement — the same split, taken at both ends of the interval on one unchanging fixture.** [The hot-path spike](../cache/hot-path-efficiency.md#91-why-the-envelope-moved-measured-at-both-ends) builds one envelope carrying zero object bytes, so its length is pure fixed content, and building that harness at its own 2026-08-04 commit and at `8bd720b8` gives 28,527 and 114,043 bytes. Parsing both attributes the whole difference: **+65,363 bytes of canonical manifest, +20,153 bytes of `KernelProgramSubject` section, and a `BackendPayloadMetadata` section byte-identical across the two**. The unchanged section is the control that makes the other two readable.

**Fact — what this re-derivation does not settle.** It does **not** attribute the growth across the interval to particular commits. The manifest schema moved only from `12.0` to `14.0` between the 2026-08-04 measurement and this one, and `14.0` is a re-ordering that changes no length, so the schema steps cannot account for the bulk: the growth is in what the manifest *describes* rather than in how the codec frames it. Naming the responsible changes needs the same fixture rebuilt at intermediate commits, which is not done here. **Done 2026-08-06:** [Where the artifact envelope's fixed content came from](../artifacts/manifest-fixed-content-growth.md) rebuilt it at all 107 intermediate landings and attributes the growth to three of them exactly.

## 2. Self-containment, demonstrated rather than argued

**Measurement.** The demonstration is `standalone-cold-everything-deleted`. `cargo rustc -- -Zunpretty=expanded` renders the tokens rustc was handed — so the literal inspected is the one the macro emitted, not one the driver reconstructed. That source becomes a crate with an empty `[dependencies]` table, every Tiler-produced file is deleted and *proved* deleted, and the crate is built and run from an empty target directory with `--offline`. It printed `slot=a len=36838 fnv1a=cde14cdbcc31cb32`, which is exactly what the driver computed independently from the producer's file before deleting it.

**Measurement.** Four properties of the expansion were checked, each able to fail (see section 6):

- exactly **one** byte-string literal, 68,076 source bytes for 36,838 payload bytes, a ratio of **1.848**;
- no `include_bytes!`, `include_str!`, `env!`, `option_env!`, `std::fs`, or `embed_macro::`;
- no occurrence of the run root, the artifact directory, or the cache root as text;
- a build that links with no dependency of any kind.

**Inference.** The one-literal check is the only place the *accepted representation* is asserted rather than assumed. An expansion emitting one integer literal per byte would be self-contained, would link, and would run — it would pass every other check here — so the count is what distinguishes the decided form from the adverse control the cost note measured at 3.5× the wall time and 3.5× the peak RSS.

**Fact, and a limitation of the route.** rustc's expanded output names internal `std` items (`::core::panicking::assert_failed`, `::std::io::_print`), so the generated crate declares `#![feature(prelude_import, panic_internals, print_internals)]`. That is a property of re-compiling *any* `-Zunpretty=expanded` output, not of the payload: nothing in the byte literal needs a feature gate, and the same three gates would be needed by an expansion carrying no artifact at all. It does mean the standalone route is nightly-only, which is a property of the *demonstration*, never of the embedding.

## 3. Deleting the cache root cannot break expanded code

**Measurement.** Four rows separate the two halves of the deletion question, because they fail differently.

| Scenario | Expansions | Envelope reads | Cache outcome | Result |
| --- | ---: | ---: | --- | --- |
| `artifacts-deleted-reexpand` | 1 | 0 | hit | 12 Tiler-produced files deleted; the cache stood in for the file |
| `cache-deleted-no-reexpand` | 0 | 0 | — | cache root deleted, envelopes already gone; build and run unaffected |
| `cache-deleted-reexpand` | 1 | 1 | published | envelopes restored; republished from disk |
| `standalone-cold-everything-deleted` | 1 | 0 | hit | 16 files deleted; dependency-free crate built cold and ran |

**Inference.** The artifact is an input to the *expansion*, and the cache is an input to the *expansion*, and neither is an input to the *expanded code*. `artifacts-deleted-reexpand` and `cache-deleted-reexpand` are the two single-deletion cases and each is survived by the other resource; `cache-deleted-no-reexpand` is the both-deleted case, and it is survived by nothing, because nothing was needed.

**The deletion is proved, which is the trap the ticket names.** A check that passes because the path was wrong proves nothing. So each deletion requires the same path to hold at least one file *before* removal and none afterwards, and reports the count. A mistyped path fails the before-check, before anything is removed — it cannot reach the after-check and pass for free. `test_self_contained.py` runs that helper against a nonexistent path and an empty directory and requires both to raise.

## 4. The four axes, each its own row

**Measurement.** Cargo and rust-analyzer, across the four axes the ticket names separately because they fail differently. Reproduced from the recorded fixture; `expansions` counts macro invocations, `reads` counts those that had to open the envelope.

| Axis | Driver | Expansions | Reads | Cache | Seconds | What it says |
| --- | --- | ---: | ---: | --- | ---: | --- |
| source edit, cold | cargo | 1 | 1 | published | 14.9 | fresh target directory and empty cache |
| source edit, warm | cargo | 1 | 0 | hit | 0.2 | an edit always re-expands; the cache spares the read |
| toolchain change | cargo | 1 | 0 | hit | 18.0 | a changed compiler re-expands exactly as a changed source does |
| repeated across crates | cargo | 2 | 1 | 1 published, 1 hit | 7.5 | the second crate reuses |
| unique across crates | cargo | 2 | 2 | 2 published | 10.8 | no contention; each key is its own |
| repeated across crates, cold | rust-analyzer | 2 | 1 | 1 published, 1 hit | 19.7 | one server process, same reuse |
| repeated across crates, warm | rust-analyzer | 2 | 0 | 2 hits | 5.3 | entries published by Cargo are hit by the analyzer |
| unique across crates, cold | rust-analyzer | 2 | 2 | 2 published | 5.4 | one server, two keys, two reads |
| toolchain change | rust-analyzer | 2 | 0 | 2 hits | 12.0 | a different proc-macro server hits the pin's entries |

**The `seconds` column is a scenario duration, and is not a timing claim.** Each is one observation of a whole scenario, and every row but the two warm ones is dominated by compiling the fixture's dependencies or by the analyzer loading a crate graph, not by expansion. Cargo's cross-crate rows also wipe their target directory to reach the cold state the axis names. The column is recorded because a scenario's cost is worth knowing and a missing column reads as a hidden one; the quantities this note draws conclusions from are `expansions` and `reads`, which are counts of observed events. [The cost note](embedded-artifact-costs.md) owns build-time measurement, with repetitions and medians.

**Reused rather than re-measured.** [The build-tool exercise](../cache/build-tool-exercise.md) already established, and this note does not repeat: that `cargo` expands in `rustc` with one short-lived process per crate while `rust-analyzer` expands in one long-lived `rust-analyzer-proc-macro-srv`; that the two share a working directory in every recorded row; that `CARGO_PKG_NAME` does not distinguish them and `std::env::current_exe()` does; that a per-key lock survives `SIGKILL` to its holder; and that three concurrent Cargo builds produce one compilation per key against twelve under an unusable root. Every row above re-confirms the process pattern incidentally — the `processes` and `cwds` columns are recorded — but the concurrency and crash halves are that note's, measured there and not duplicated here.

**Measurement, and the finding worth carrying forward.** *Cargo's freshness notion and the cache's subject are deliberately different sizes, and the toolchain axis is where that becomes visible.* Building the identical sources with `nightly-2026-07-20` after `nightly-2026-07-19`, in the same target directory with no source edit, re-expanded — Cargo's fingerprint carries the compiler — and read nothing from disk, because the composed subject is a function of the artifact identity alone. The analyzer row is the same result reached from the other side: a `nightly-2026-07-20` proc-macro server resolved entries a `nightly-2026-07-19` process had published, as validated hits.

**Inference.** This is correct under the current design and is the behaviour to preserve deliberately. A resolution is a pure function of the composed subject, so an entry published by one compiler is not *stale* for another — the artifact it holds was produced by the offline Metal toolchain and has no Rust compiler in its identity at all. It would stop being correct the moment anything about the Rust toolchain influenced the bytes an expansion embeds, which is the same hazard [the build-tool exercise's section 7](../cache/build-tool-exercise.md) records for a changed cache root and [the proc-macro environment note](../macro-environment/proc-macro-build-environment.md) records for a changed Metal toolchain, arriving by a third route.

**Measurement.** Cross-crate expansions genuinely overlap under Cargo — one intersecting pair in each of the two cross-crate rows, from windows recorded by two different `rustc` processes — and never under the analyzer, whose single server expands sequentially. Both are the expected shapes and neither is an artifact of the harness assuming them: overlap is computed from recorded wall-clock windows, so a scenario that serialized would report zero rather than claiming a concurrency it did not reach.

## 5. The gates, as numbers

### Size

**Measurement.** The largest artifact this slice produces is **47,803 bytes**. The ceiling is **1,048,576 bytes (1 MiB) per invocation**, restated unchanged from [the cost note](embedded-artifact-costs.md), which also sets the second gate at 32 invocations or 3.2 MiB of direct bytes in one consumer package, whichever is reached first — stated in that note as a round figure, and quoted here as one rather than converted to a byte count it never gave. The largest real artifact is therefore **4.56%** of the per-invocation ceiling, and the one measured here (36,838 bytes) is **3.51%**.

**Measurement.** One byte-string literal of 36,838 payload bytes renders as 68,076 bytes of source text, a ratio of **1.848**. Extrapolated to the ceiling, a 1 MiB payload is roughly 1.85 MiB of source text handed to rustc as a single token.

**Inference.** The ceiling is not in danger from this workload and is not thereby validated for a larger one. What the measurement bounds is the *distance*: real Tiler artifacts on this target sit more than an order of magnitude below the gate, so the gate is currently a guard against a future payload class rather than a constraint on the present one. The crate-wide second gate is untouched here — this fixture embeds one artifact per crate — and remains what [the cost note](embedded-artifact-costs.md) says it is: work a proc macro cannot do, owned by an integration size test.

**Measurement, 2026-08-06, and the distance is the number that moved.** At the re-derived band the largest member this producer publishes is **159,037 bytes**, which is **15.17%** of the 1,048,576-byte per-invocation ceiling, and the member this note embedded is now 146,324 bytes, **13.96%**. The two figures above, 4.56% and 3.51%, are what the same members measured on 2026-07-31 and are retained as that run's evidence rather than corrected.

**Inference.** The order of magnitude of headroom is gone; roughly two-thirds of a threefold growth would exhaust the per-invocation gate. The gate itself is unchanged and nothing here proposes moving it — what changed is that "more than an order of magnitude below" is no longer a true description of the distance, so a reader deciding a payload class against this ceiling should read the 2026-08-06 row. At the 1.848 source-text ratio measured above, a 159,037-byte payload is roughly 294 KB of source text in one token. Every corpus site that quotes the old band as a live measurement is listed in [`re-price-the-envelope-band-consumers-against-the-re-derived-band`](../../../tickets/re-price-the-envelope-band-consumers-against-the-re-derived-band.md), which owns the ones outside this note's scope.

### Diagnostics

**Measurement.** Seven failure classes, every one reached by a build that had to fail. A class whose build *succeeded* fails the run, so none of these texts describes an unreachable path. The complete rustc rendering — span, caret, source line, and the macro-backtrace note — is in [the diagnostics fixture](../../../spikes/embedding/results/self-contained-diagnostics-macos-27.0-2026-07-31.txt); every class renders at the invocation's own span in `consumer-a/src/main.rs`. The messages, verbatim:

`directory-unstated`, and the same sentence naming `TILER_EMBED_MEMBER_A` for `slot-unstated`:

> `embed!` requires `TILER_EMBED_DIR` to be set to a non-empty value, and will not substitute a default: an embedding that silently changed which artifact it carried would produce a consumer binary nobody named. Set `TILER_EMBED_DIR`, or set `TILER_EMBED_CACHE` to `off` to expand without a cache

`cache-root-unstated`:

> `embed!` requires `TILER_EMBED_CACHE` to be set to a non-empty value, and will not substitute a default: a cache root that arrives unstated is a cache that quietly relocates, and a developer sees only that builds became slow. Set it to an absolute directory path only you can write, or to `off` to expand without a cache

`cache-root-relative`:

> `TILER_EMBED_CACHE` is set to `relative/cache`, which is not an absolute path. A proc macro runs in the build tool's working directory rather than yours, and `cargo` and `rust-analyzer` need not agree on it, so a relative root would name different directories in one project. Set `TILER_EMBED_CACHE` to an absolute directory path only you can write, or to `off` to expand without a cache

`member-unavailable`, the state reached when both the envelope and any cache entry standing in for it are gone:

> `embed!` cannot carry `serial-sum.tiler.nontrivial.selected`: <path> could not be read (No such file or directory (os error 2)), and no cache entry stood in for it. The artifact is an input to this expansion, so it must exist the first time a build expands this invocation; it is not needed afterwards, because the bytes are already in the expanded code. Re-run the producer, or point `TILER_EMBED_DIR` at a directory that holds it

`invalid-artifact`:

> `embed!` read <path> for `serial-sum.tiler.nontrivial.selected`, but those bytes are not a decodable Tiler artifact (Malformed { detail: "TotalLengthMismatch { declared: 36838, actual: 18419 }" }); embedding them would put a payload in the consumer's binary that no runtime could accept. Re-run the producer that writes it

`ceiling-exceeded`:

> `embed!` refuses to carry `serial-sum.tiler.nontrivial.selected`: it is 36838 bytes and this invocation's ceiling is 1024 bytes. The ceiling is a measured product bound rather than a Rust or linker limit, and every emitted copy counts against it, so crossing it is an explicit decision with a new measurement behind it. Raise `TILER_EMBED_CEILING_BYTES`, or split the region so each invocation carries less

**Inference.** Two of these are load-bearing beyond their text. `invalid-artifact` is produced by truncating a real envelope to half its length and carries the cache's own `decode_artifact` rejection through to the consumer, so the validation on the embedding path is demonstrably not decorative. `ceiling-exceeded` is reached by lowering the ceiling to 1,024 bytes rather than by manufacturing a 1 MiB artifact — the ceiling is overridable *for that reason*, because a refusal no test can reach is a refusal no reader should believe.

## 6. How this experiment could have failed

A uniform pass over a heterogeneous population is the signature to distrust, so each control names the population it covers.

**Every deletion is proved in both directions.** The helper requires the exact path to hold files before removal and none afterwards, and returns the count. Section 3 states why; `test_self_contained.py` requires it to raise on a nonexistent path and on an empty directory.

**Every scenario declares its expansion count.** Events are one file per expansion rather than appended lines, because two `rustc` processes write at once and an interleaved append can drop a record with no reader noticing. A scenario that expanded nothing is not the same observation as one where everything hit, and the counts distinguish them: `cache-deleted-no-reexpand` requires **zero**, and would fail if the build had re-expanded, which is the only way that row could look correct for the wrong reason.

**The consumer's output is checked against the producer's file, not only against itself.** The expansion records a length and checksum beside the payload and the program compares the linked bytes against those — which catches corruption between emission and link and nothing else, since both numbers come from the macro. The driver computes the length and FNV-1a from the producer's file independently, and requires the program's printed values to equal *those*. `test_self_contained.py` pins the checksum against three known vectors, so the Rust and Python implementations cannot drift into agreeing with each other about a different function.

**The representation is asserted, not assumed.** Exactly one byte-string literal is required. The test file requires a per-byte expansion — the cost note's adverse control — to be rejected, and requires a literal containing escaped quotes and backslashes to be scanned to its true end.

**Two checks failed during development and were fixed rather than relaxed.** The self-containment predicate first rejected the fixture's own documentation, because it matched the bare word `embed!` in a doc comment rather than a call form; the forbidden list is now call forms, and the structural half — an empty `[dependencies]` table and a linking `--offline` build — carries what the text check gave up. The diagnostic capture first filtered stderr by line shape and silently dropped the numbered source line, publishing a caret under nothing as though it were the rendering a consumer sees; it is now verbatim from the first `error` line.

**Concurrency is observed, not assumed.** Cross-process overlap is computed from recorded wall-clock windows. The cross-crate rows report one overlapping pair each; had the two crates serialized, the count would read zero rather than the scenario claiming a workload it never reached.

**What this does not establish.** Every scenario passing says these properties held for these populations on this host with these two toolchains. It is not evidence for another host, a release profile, a payload class larger than ~47 KiB, or a live editor session.

## 7. What was not reached, and what it would need

| Not reached | Why | What it would need |
| --- | --- | --- |
| The production `tiler::tensor!` carrying bytes | It states `FallbackOnly`, which ADR 0053 defines as invoking no backend compiler, so it embeds nothing and opens no cache; there is no production embedding to measure | `generate-cfg-gated-artifact-family-delivery`, the slice that first compiles a selected family and consumes [`cache_root.rs`](../../../crates/tiler-macros/src/cache_root.rs) |
| A real LSP session with incremental edits | `analysis-stats` loads the project and expands once per run; it holds no Cargo fingerprint, so "a source edit" is not a state it can be in | An LSP client harness, or an editor driven by script — the same boundary [the build-tool exercise](../cache/build-tool-exercise.md) records |
| Payloads above 47,803 bytes, and since 2026-08-06 above 159,037 | This slice's producer emitted envelopes of 32,136–47,803 bytes when this note was written and emits 89,250–159,037 at `8bd720b8`; nothing here reaches the 1 MiB ceiling with real bytes, though the re-derived band closes the distance to 15.17% of it | A larger real program. [`measure.py`](../../../spikes/embedding/measure.py) reaches 1 MiB with synthetic payloads and measures size, not self-containment |
| Release profile, linker folding, constant-section size | Every build here is dev-profile and single-artifact-per-crate | [The cost note](embedded-artifact-costs.md) owns that matrix; re-running it against these envelopes would attach it to real bytes |
| The crate-wide 32-invocation / 3.2 MiB gate | A proc macro cannot discover a reliable crate-wide total, and this fixture embeds one artifact per crate | An integration size test or CI binary-size job, as the cost note already states |
| A toolchain change across a stable boundary | `tiler-ir` requires nightly dependent-array const parameters, so both measured toolchains are nightlies one day apart | A stable MSRV, which the workspace deliberately does not declare |
| Any toolchain not already installed | This driver never installs, selects, or mutates a toolchain; it refuses and says what is missing | Authorization, then a re-run — the axis is otherwise recorded as unreached rather than assumed |
| Cancellation, server restart, and concurrency above two | The analyzer scenarios run to completion and the cross-crate scenarios use two crates | [The build-tool exercise](../cache/build-tool-exercise.md) covers the kill case for the cache at concurrency three; an LSP client for cancellation |
| A standalone rebuild on stable Rust | rustc's `-Zunpretty=expanded` output names internal `std` items and needs three feature gates | Nothing about the payload; a demonstration that hand-writes the consumer's `main` around the lifted literal would drop the gates and, with them, the evidence that rustc rendered the tokens |
| Linux, and any host but the measured one | One macOS host with one Metal toolchain was available | The corresponding filesystem and toolchain probes |

## 8. Measurement environment

**Measurement.** macOS 27.0 build 26A5388g, arm64, Apple M4 Max, 14 logical cores, 36 GiB. `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, the `rust-toolchain.toml` pin `nightly-2026-07-19`; the toolchain-change axis moves to `nightly-2026-07-20`, `rustc 1.99.0-nightly (9f36de775 2026-07-19)`. `rust-analyzer` from the rolling `nightly` toolchain, driving the pin's `libexec/rust-analyzer-proc-macro-srv` — and, for the toolchain axis, `nightly-2026-07-20`'s. Xcode 26.6 build 17F113; Apple `metal` version 32023.883 (`metalfe-32023.883`) from `MetalToolchain-v17.6.109.0`. Every build is dev-profile and `--offline`.

```sh
python3 spikes/embedding/self_contained.py --record macos-27.0-2026-07-31
uv run --with pytest pytest spikes/embedding/test_self_contained.py
```

The run installed, selected, and changed no toolchain component. It is an observation about one host and one set of tool versions, not a portable guarantee.

**Measurement, and what reproducing it showed.** The recorded fixture was produced twice on this host, the second time after deleting the fixture's `Cargo.lock` and target directory so the run rebuilt from nothing. Every count reproduced exactly — expansions, envelope reads, cache outcomes, processes, overlaps, working directories, the payload sizes, and the 68,076-byte literal — and every diagnostic reproduced byte for byte apart from the run root's own path. Only the `seconds` column moved, by as much as 3× on the dependency-dominated rows, which is the concrete reason section 4 declines to draw a conclusion from it. A third run failed outright with `No space left on device` on a nearly full disk, and stopped rather than recording a scenario it had not completed; the harness now uses three target directories instead of six, and [the spike README](../../../spikes/embedding/README.md) states the space it needs.

## 9. Outcomes

1. **Bounded experiment, preserved.** `spikes/embedding/self-contained/` and [`self_contained.py`](../../../spikes/embedding/self_contained.py), with declared populations, proved deletions, and a perturbation suite in [`test_self_contained.py`](../../../spikes/embedding/test_self_contained.py). Results and the verbatim diagnostics are under `spikes/embedding/results/`.
2. **A gap in a neighbouring record is closed.** [The build-tool exercise](../cache/build-tool-exercise.md) lists "a carried compiled payload" as not reached, on the ground that its envelope declared its payload by descriptor. The envelopes here carry compiled `metallib` objects through `get_or_publish`, validated on every hit by the real `decode_artifact`. That note is not edited from here — it is another ticket's record and the correction belongs in its own change — and the claim is superseded by evidence either way.
3. **A contract sentence this evidence supports, not yet written.** [`docs/integration/frontends.md`](../../integration/frontends.md) states the inline-DX invariant that each invocation is a self-contained AOT and embedding unit. Nothing there yet says what "self-contained" was measured to mean — that a deleted cache root and a deleted artifact set leave expanded code untouched. Adding it needs the contracts scope and is not taken here.
4. **The band re-derived, 2026-08-06, and the consumers filed.** Section 1's envelope column is a measured claim other records borrow, and re-running the producer at `8bd720b8` moved it to 89,250–159,037 bytes over eight members and 141,532–159,037 over the six this note originally measured, with the `metallib` column byte-identical. The 2026-07-31 rows are retained. [`spikes/cache/hot-path-efficiency`](../cache/hot-path-efficiency.md) has been re-run at the new endpoints in the same change; the sites outside those two records that quote 32,136–47,803 or the 4.56% ceiling headroom as live figures are carried by [`re-price-the-envelope-band-consumers-against-the-re-derived-band`](../../../tickets/re-price-the-envelope-band-consumers-against-the-re-derived-band.md).
5. **Deferred, with triggers.**
   - *A production embedding to measure.* Trigger: `generate-cfg-gated-artifact-family-delivery` landing, at which point the frontend first compiles a selected family and the questions in this note apply to `tiler::tensor!` rather than to a fixture.
   - *The size gates against a real payload class near the ceiling.* Trigger: a Tiler program whose envelope exceeds a few hundred KiB. Until one exists, the 1 MiB per-invocation and 3.2 MiB per-package bounds stay as [the cost note](embedded-artifact-costs.md) set them, with the distance measured here recorded rather than treated as validation.
   - *An LSP client harness.* Trigger: a decision that editor-side scheduling and incremental-edit behaviour is worth its own gate. Nothing measured here depends on the answer.

[`Literal::byte_string`]: https://doc.rust-lang.org/proc_macro/struct.Literal.html#method.byte_string
