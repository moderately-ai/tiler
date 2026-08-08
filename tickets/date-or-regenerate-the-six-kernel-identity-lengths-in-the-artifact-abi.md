---
id: date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi
title: Date or regenerate the six kernel identity lengths in the artifact ABI
status: in-progress
priority: p2
dependencies: []
related: [replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, abi, identity, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786176329
---

The last unowned quantity in `docs/artifact-abi.md` after its byte figures were retired. Same defect class, one paragraph over.

## Facts

**Reported by the worker that retired the neighbouring figures; NOT coordinator-verified — re-measure before relying on any of it.** The "Governed budgets" paragraph, anchored at *"the three identities never shared a subject"*, states six kernel-identity lengths — 736, 1,483, 1,845, 1,700, 2,279, and the pinned 1,121 — in the present tense, bounded to a host but to **no commit and no date**. Five are said to carry no pin.

**Reported: the one pinned value cannot catch drift in the others.** Its assertion is only that the value exceeds `MAX_OPAQUE_IDENTITY_BYTES`, so it is a floor rather than an equality and would stay green across a change that moved every length.

**Reported: the kernel identity has stepped since.** `tiler.kernel.v7` and `tiler.kernel-program.v11` are the current domains, so the figures are plausibly stale — but plausibly is not measured, and the sibling found figures that moved **downward** by tens of thousands where everyone assumed growth.

**Reported: regenerating needs** the serial-`f32`-sum kernel at one contributor and ranks 3–8, via `crates/tiler-conformance/src/serial_sum.rs`.

## What closes this

Either the six regenerated from their construction and restated **with the commit and date they were measured at**, or the paragraph rewritten to state the property without the figures — whichever the paragraph's argument actually needs. Read the sibling ticket's route first: it retired rather than refreshed, because every property the figures supported was already pinned, and it kept the structural account that does not decay.

**Do not derive any figure arithmetically.** The sibling measured four offsets that moved in opposite directions across a single identity step; a uniform correction would have been wrong at all of them.

**You cannot add a pin yourself** — assertions live in `crates/**`, outside this scope. If pinning is the right answer, name which construction should assert which value and file it rather than widening.

If you regenerate, say which host and which commit, and keep the measurement bounded to them. `AGENTS.md` is explicit that measurements bound claims but do not prove unmeasured universals; six lengths from one kernel shape are evidence about that shape.

This is reported to be the **last** unowned quantity in the document — its census found 21 sites carrying a quantity, of which 2 were unowned and these are they. Confirm that census rather than trusting it, and report the count either way.

## Outcome

**Per-Fact audit at base `68ba010ab117fb6840b5473154e2fbf83db5a46f`, each Fact re-read at that base rather than inherited.**

| Ticket claim | Verdict | Evidence |
| --- | --- | --- |
| the paragraph anchored *"the three identities never shared a subject"* states six kernel-identity lengths — 736, 1,483, 1,845, 1,700, 2,279, 1,121 — in the present tense, bounded to a host but to no commit and no date | **verified** | the paragraph read in full at this base; it said "measures 736 bytes" and "On this checkout (Apple M4 Max, macOS, the pinned toolchain)" and named neither commit nor date |
| five of the six carry no pin | **imprecise — none of the six carries a pin, including 1,121** | `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` builds `vec![0x5a; 1_121]`, a fabricated byte vector, and asserts its length exceeds `MAX_OPAQUE_IDENTITY_BYTES`. That is an assertion about a literal, not about a kernel. No compilation occurs in it and none can — see the structural finding below |
| the one pinned value is a floor that would stay green across a change moving every length | **verified, and it did exactly that** | the real value moved 1,121 → 1,309 and the test is still green at this base |
| the kernel identity has stepped since; `tiler.kernel.v7` and `tiler.kernel-program.v11` are current | **verified** | `KERNEL_DOMAIN` is `b"tiler.kernel.v7\0"` in `crates/tiler-ir/src/kernel/model.rs`; the kernel-program `v11` step is the paragraph anchored *"folding the declared staged-realization contracts"* in `docs/artifact-abi.md` |
| regenerating needs the serial-`f32`-sum kernel at one contributor and ranks 3–8 via `crates/tiler-conformance/src/serial_sum.rs` | **false** | that crate's `serial_sum_program` is `pub(crate)` and builds `Shape::from_dims([rows, columns])` reducing `Axis::new(1)` — rank 2 only, so it cannot produce a rank-3 through rank-8 program at all, and it is unreachable from outside the crate. Its `compile_under` also takes a `BoundMetalCompileDeclaration`, not the `compile_governed` route the figures were first taken on. The construction the figures actually came from is the sweep recorded in [`bound-the-backend-entry-key-by-the-identity-it-carries`](bound-the-backend-entry-key-by-the-identity-it-carries.md): `SemanticProgram`s built at varying shape and reduced-axis sets, compiled through `compile_governed(_, FLUSH_SUBNORMALS_TO_ZERO_F32)`, reading `VerifiedKernel::canonical_identity().as_bytes().len()` off the selected plan |
| this is the last unowned quantity in the document | **imprecise** — see the census below; it is the last of its exact class, but the sibling's own repair left a second measurement stated as unpinned | |

**Measurement, at `68ba010a`, Apple M4 Max, macOS 27.0 (26A5388g), toolchain `nightly-2026-07-19`.** A temporary probe module appended to `prototypes/serial-sum-compile/src/main.rs` built each shape and reduced-axis set directly, compiled it under both routes, and printed `VerifiedKernel::canonical_identity().as_bytes().len()` for the selected plan's kernels. Run as `cargo nextest run -p tiler-prototype-compile -E 'test(temporary_probe_for_kernel_identity_lengths)' --no-capture`; the probe was reverted before any commit and `git status --porcelain` is empty. **Nothing was derived arithmetically** — every row is a separate compilation.

| input shape | reduced axes | stated | measured at `68ba010a` | delta |
| --- | --- | --- | --- | --- |
| `[4, 1]` | `[1]` | 736 | **924** | +188 |
| `[4, 2]` / `[4, 3]` / `[4, 4]` / `[4, 8]` | `[1]` | 1,121 | **1,309** | +188 |
| `[4, 3, 3]` | `[2]` | 1,483 | **1,671** | +188 |
| `[4, 3, 3, 3]` | `[3]` | 1,845 | **2,033** | +188 |
| `[4, 3, 3, 3, 3]` | `[1, 2, 3, 4]` | 1,700 | **1,888** | +188 |
| `[4, 3, 3, 3, 3, 3, 3, 3]` | `[1, 2, 3, 4, 5, 6, 7]` | 2,279 | **2,467** | +188 |

All six are stale. Every row of the 2026-07-25 sweep was re-run, not only the six the document quotes: `[1, 3]` reducing `[1]` is 1,217 (was 1,029), `[4, 3, 3]` reducing `[1, 2]` is 1,502 (was 1,314), `[4, 3, 3, 3]` reducing `[1, 2, 3]` is 1,695 (was 1,507), and ranks 6 and 7 reducing all but one are 2,081 and 2,274 (were 1,893 and 2,086). Both slopes are unchanged — +362 per rank reducing one axis, +193 per rank reducing all but one — and the one-to-two-contributor step is +385 in both trees. Only the constant offset moved.

**The uniform +188 is a coincidence of this step and is recorded as one.** The sibling's `v15 -> v16` step moved four offsets in opposite directions, so a reader must not add 188 to a later reading; the document now says so in terms.

**A route difference the reproducing procedure would otherwise hide.** At `68ba010a`, `compile_governed` refuses `[4, 3, 3]` reducing `[2]`, `[64, 3]`, and `[4096, 3]` as `NoFeasiblePlan` before a plan composes — all three compiled under that route in July — so it can no longer reach the rank-3 and rank-4 single-axis rows. Under `BoundMetalCompileDeclaration::first_macos_apple9()`, the declaration `prototypes/serial-sum-compile` actually compiles under, every row reaches a plan. Where both routes admit a shape they agree on the length exactly, so this is a reachability difference and not an identity difference.

**Route — regenerate and date, and it is deliberately *not* the sibling's route.** [The sibling](replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin.md) retired rather than refreshed because every property its nine figures evidenced was already pinned, so the digits carried nothing and only decayed. That premise fails here. The paragraph's two load-bearing claims — that the step is between one contributor and two, and that identity grows with structure without a small ceiling — are pinned by nothing, and the one test the sibling's census credited asserts a fabricated literal against a constant. Retiring the digits would delete the only evidence for the argument rather than deferring to a pin, which is the sibling's form without its substance. What these figures *are* is a historical justification for a bound that has already moved, and a historical measurement is owned by a date, a commit, a host, and a reproducing construction rather than by an assertion — the "dated measurement" class the sibling's own census accepts for six other sites in this file. So the paragraph now states its structural claims without digits, and carries the July and the `68ba010a` readings side by side in a dated table with the host, the commit, the construction, and the route caveat.

**Proposed pin, for a `crates/**` ticket — filed as [`pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound`](pin-the-serial-sum-kernel-identitys-crossing-of-the-opaque-identity-bound.md).** The property to assert is the two-sided inequality, not a length: a one-contributor serial `f32` sum's canonical kernel identity is *under* `MAX_OPAQUE_IDENTITY_BYTES` and a two-contributor one is *over* it. That is the whole of the argument the bound was changed on, it fails loudly from either direction, and it does not decay when the offset moves again — which a length pinned to 1,309 would, exactly as 1,121 did.

**The reason it cannot be pinned where the census assumed, and it is structural.** `crates/tiler-artifact/Cargo.toml` carries no `tiler-compiler` edge and says why in a comment: `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` walks `Cargo.lock`, which merges normal and development edges, so a dev edge there would put the compiler in the consumer's closure and fail that test against ADR 0081 item 2. The crate that owns `MAX_OPAQUE_IDENTITY_BYTES` therefore *can never* compile a real reduction to compare against it, and `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` fabricating its length is a consequence of that boundary rather than a shortcut. The assertion belongs in a crate that already reaches both — `crates/tiler-conformance`, whose `serial_sum.rs` holds the program builder and the compile route, or `prototypes/serial-sum-compile`'s test module.

**Three stale statements found outside this scope**, all carried into the follow-up ticket: `crates/tiler-artifact/src/program/tests.rs`, the comment calling the 1,121-byte case "the measured one, not a chosen one … the canonical kernel identity of a serial `f32` sum reducing two or more contributors" — false at this base, it is 1,309; `prototypes/serial-sum-compile/src/main.rs`, `COLUMNS`'s comment stating the identity "measures 1,121 bytes"; and `prototypes/serial-sum-run/src/proof.rs` at two sites stating the same value in the present tense.

## Numeral census, `docs/artifact-abi.md`, whole file at `68ba010a`

Enumerated mechanically, then every survivor resolved by reading its site. `[0-9][0-9,]*(\.[0-9]+)?` yields **728 digit-run occurrences over 154 distinct tokens**; a spelled-numeral pass yields **582 occurrences over 16 distinct words**, of which `one`/`two`/`three`/`four`/`zero` alone account for 524, almost all of them ordinary determiners in prose. Mechanically stripping dates, `ADR NNNN` references, decision-record slugs and links, `tiler.*.vN` domain tags, `N.0` schema versions, and hex literals leaves **104 candidate lines**, every one read.

**Set aside as identifiers rather than quantities:** dates, ADR and decision-record references, domain-version tags, manifest and component schema versions (including the `2.0` / `1.0` / `{1, 0}` step at the section-descriptor item and the `16.0` manifest step), hex tag literals, the numbered stage list and the stage references that index it, "unsigned-64" as a coordinate-space name, "SHA-256" as an algorithm name, and the dependency and platform version strings `sha2` 0.11.0, macOS SDK 26.5, macOS 27.0, MSRV 1.89, CUDA compute capability 7.0, FIPS 180-4.

**The remainder falls on 21 sites carrying a quantity**, which reproduces the sibling's count at its own base; the three paragraphs added to the file since `97282def` — the `tiler.semantic-graph.v3` supersession clause, the retirement correction, and the three-of-five shape-environment paragraph — carry no new quantity, only domain tags, dates, and counts of items in the document's own prose. Sites holding figures of more than one class are named under each.

- **Pinned in a workspace test or crate constant — 9 sites.** The two framing-header tables' `69` (`HEADER_BYTES` in `crates/tiler-artifact/src/program/codec/encode.rs` and in `.../proof/codec.rs`), one per table. `tiler_digest::DIGEST_BYTES` as the thirty-two-byte digest width, at the ADR 0103 objection paragraph. `FIXED_CONTENT_BYTES` = `65_313`, the terminal total of the `v15 -> v16` block. `DIFFERING_CARRIER_POSITIONS` = `68` at the differing-position paragraph. The `seven` / `four` / `eighteen` governed-domain counts, asserted against `core::mem::variant_count`, at the governed-digest paragraph and at the union no-prefix paragraph. The two "Governed budgets" paragraphs, each naming a crate constant enforced on both sides. The proof sidecar's own budgets paragraph. The identity-growth ladder's region-shape budgets `62` / `80` / `3` in `DeterministicBudgets::governed`.
- **Spec constant, or this document's own wire definition rather than a measurement of it — 8 sites.** The two framing-header tables' offsets and widths. The canonical NaN payloads `0x0000_7fc0` and `0x7fc0_0000` with their thirty-two and sixteen bits, and the `bf16` contract domain's sixteen-bit restatement. The `0x01` digest-algorithm tag. The header flip sweep, whose `69` is the header width above. The eighteen separator bytes of the `tiler.schedule.v4` step and the five goldens carrying no cooperative tile. Subgroup widths 32 and 64 with their five and six combine steps. The `28` pinned provenance bytes, which the same paragraph derives from its own grammar — two `u16` positions plus three fixed-width run lengths. The embedding gate's 1 MiB per invocation, 32 invocations, and 3.2 MiB per package, a declared gate rather than an observation. The four hashing sites.
- **Dated measurement, anchored to a named commit, host, or retained artifact — 6 sites.** The `v15 -> v16` step block. The ADR 0103 manifest-digest block at commit `eee734cf`. The identity-growth ladder re-run 2026-08-07 and retained at a named `spikes/program-planning/identity-growth/results/…/growth.tsv`. The decoder-allocation block linked to its research note. The 30 retained proof cases on one Apple M4 Max corpus. **And the kernel-identity paragraph, which this ticket moved into this class from the one below.**
- **Explicitly retired in place by a dated correction — 2 sites.** The `3525n + 727` ladder and its derived figures. The `40` and `67` earlier readings of the differing-position count.
- **Unowned — 0 sites.**

**One reconciliation the ticket's "last unowned quantity" claim needs, and it is not this paragraph.** The sibling's repair left the forged pair's *equal identity length* and its *count of four differing positions* asserted by nothing, and said so in the document, naming the constant that should carry them. That is a measurement with no pin, no date, and no commit. It is not *unowned* in the class sense — the document states its own gap and proposes the assertion, which is the resolution the sibling ticket prescribed for exactly that case — but a reader counting bare unpinned numbers will find it, and it is the only one left. It belongs to the sibling's proposed `crates/**` pin, not to this ticket.
