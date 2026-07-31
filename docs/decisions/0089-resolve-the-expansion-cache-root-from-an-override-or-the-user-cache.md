---
schema: "tiler-doc/v1"
id: "ADR-0089"
kind: "decision"
title: "Resolve the expansion-cache root from an override or the user cache"
topics: ["cache", "frontends", "proc-macros", "configuration", "artifacts"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.frontend-integration"]
evidence: ["tiler.research.cache.root-policy"]
depends_on: ["ADR-0050", "ADR-0082", "ADR-0088"]
ticket: "record-the-expansion-cache-root-policy-decision"
---

# 0089: Resolve the expansion-cache root from an override or the user cache

**Status:** accepted. Tom accepted the complete boundary packet on 2026-07-31 — the variable spelling `TILER_EXPANSION_CACHE_DIR`, the exact `off` disable, the empty-override refusal, the `$HOME/Library/Caches/ai.moderately.tiler/expansion` default, override-first total precedence, the five-tree non-private refusal set, the verbatim-override rule, and the judgement that a missing `HOME` refuses rather than silently disabling the cache. The acceptance is recorded under "Accepted (2026-07-31)" in [`choose-the-expansion-cache-root-policy`](../../tickets/choose-the-expansion-cache-root-policy.md), which also carries the exact `compile_error!` text of each refusal as Tom read it. This record states the decision and cites [the root policy note](../research/cache/root-policy.md) for the derivation; it does not re-derive it.

## Context

**Fact — the storage protocol deliberately has no default, and must not acquire one.** `tiler_cache::expansion::ExpansionCache::open(root)` takes its root from the caller, performs no I/O, and creates no directory; the crate never consults the environment. The reproducible check the research note publishes: `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `expansion/harness.rs`, `expansion/fault.rs`, and `expansion/tests.rs`, all `#[cfg(test)]`.

**Inference — the chooser therefore belongs to the frontend.** A host-relative decision inside a storage protocol is what would stop that protocol being testable without a host, so something other than [`tiler-cache`](0082-admit-tiler-cache-as-the-expansion-cache-owner.md) must choose, and the only candidate is the caller. Until 2026-07-31 there was no caller: [the build-tool exercise](../research/cache/build-tool-exercise.md) deferred the question with the trigger "the first proc-macro frontend crate", and [ADR 0088](0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) fired it by admitting `tiler` and `tiler-macros`.

**Fact — the root's privacy is a requirement of the cache, not a preference of the chooser.** `ExpansionCache::open`'s own documentation requires a root "private to the user running Tiler", because the [immutable self-validating entries](0050-use-immutable-self-validating-expansion-cache-entries.md) catch corruption and partial writes and explicitly do "not make a shared writable cache an adversarial boundary, because an attacker able to replace files can construct new internally consistent bytes". Every refusal below descends from that sentence.

**Fact — the accepted contracts already stated the *shape* of the answer.** [The frontend contract](../integration/frontends.md) says "A default macOS user cache is used rather than consumer `OUT_DIR`. A documented override supports CI and sandboxed builds", and [the Metal AOT backend contract](../backends/metal.md) says "The default lives in an OS-appropriate user cache with a CI/sandbox override, rather than consumer `OUT_DIR`". What was missing was not the shape but the exact derivation, the exact precedence, and what happens when neither input is usable. One apparent conflict — the build-tool exercise's Section 3 **Proposal** that the root "must be made explicitly rather than defaulted into a home directory" — is reconciled in the research note under `AGENTS.md`'s authority order, and this record does not reopen it: what that proposal protects is a root arriving *unstated*, and stating the derivation exactly is what preserves it.

**Fact — the surface is Tom's rather than a derivation.** Every item here is consumer-visible, which [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) routes to Tom regardless of research quality. Each was implemented first as a crate-private draft so Tom read a concrete spelling rather than a description of one.

## Decision

### 1. Exactly two inputs, override first and total

**Decided.** An expansion resolves its cache root from `TILER_EXPANSION_CACHE_DIR` when that variable is set, and otherwise from `$HOME`. There is no third input, no third step, and no fallback location.

The precedence is total rather than conditional: a stated override decides the root whether or not the default would have worked. An override that applied only when the default was unusable would make the CI case depend on the CI machine's `HOME`, which is exactly the variable a CI operator cannot rely on, and would mean no single setting is a complete answer.

The variable is named for the *expansion* cache rather than for Tiler as a whole. [ADR 0082](0082-admit-tiler-cache-as-the-expansion-cache-owner.md)'s residue foresees a runtime pipeline-state cache and a compiler plan cache; a generic `TILER_CACHE_DIR` would silently acquire them the day they exist, changing what a consumer's existing setting means. The accepted cost is length.

### 2. The default is `$HOME/Library/Caches/ai.moderately.tiler/expansion`

**Decided.** **Measurement — macOS 27.0 arm64, 2026-07-31:** `ls -ld ~/Library/Caches` reports `drwx------` owned by the invoking user. The default is therefore private by construction rather than by a check, which is what makes it satisfy `ExpansionCache::open`'s requirement with no filesystem inspection at all. The reverse-DNS component follows Apple's convention for a cache namespace, and the trailing `expansion` leaves room for a sibling cache to hold a different Tiler subject without either colliding with the other's layout.

### 3. A stated override is used verbatim, and the exact value `off` disables the cache

**Decided.** Tiler appends nothing to a stated path. Appending its own components would put the cache somewhere the consumer did not name, so a consumer inspecting or clearing the cache would be looking in the wrong place.

The single exception is the exact string `off`, which means expand, compile, and embed with no cache at all. It is matched exactly — no case folding, no synonyms, no prefixes — so a value that merely *resembles* the sentinel is refused rather than guessed at. A disable value is load-bearing rather than a convenience: a sandbox with no writable per-user location must be able to say so, and without `off` the refusals below would name a remedy such an environment cannot satisfy. Disabling costs duplicate external compiler work and never correctness, because the cache is an accelerator over a computation that is performed either way.

A disable is not an absence. It is returned because a consumer asked for it, never because a lookup quietly came back empty.

### 4. An empty override is a refusal, not an absence

**Decided.** `TILER_EXPANSION_CACHE_DIR=""` is refused rather than read as unset. An exported-but-empty variable is the residue of a script that computed a path and got nothing, and falling through to the default would hide exactly that failure — the consumer would receive a working build against a root it did not choose, which is the shape of silence this whole policy exists to eliminate.

### 5. A root in a world-writable tree is refused, and the claim's boundary is stated with it

**Decided.** A stated or derived root at or under `/tmp`, `/private/tmp`, `/var/tmp`, `/private/var/tmp`, or `/Users/Shared` is a typed refusal naming the tree it fell in.

**Measurement — macOS 27.0 arm64, 2026-07-31.** `ls -ld` reports mode `1777` — writable by every user, sticky — for `/private/tmp`, `/private/var/tmp`, `/var/tmp`, and `/Users/Shared`. `/tmp` is a symbolic link to `private/tmp` and `/var` to `private/var`, so both spellings of each pair are listed rather than resolved, because resolving one means touching the filesystem from a decision that must stay pure. `$TMPDIR` is `/var/folders/<a>/<b>/T/` with mode `0700`, owned by the invoking user, and is deliberately *not* in the set — which is what makes the refusal affordable, because the ordinary CI remedy survives it.

This is a correctness refusal rather than a style preference: the cache's integrity validation is explicitly not a defence against another user writing internally consistent bytes. Containment is compared by whole path components, so `/tmpfiles` is not under `/tmp`; a textual prefix test would have refused it.

**The boundary of the claim, decided as part of the decision.** Absoluteness and shared-tree membership are the only privacy properties decidable from a path without touching the filesystem, so a root that passes is *asserted* private by whoever named it and is not proven private by Tiler. A symbolic link into a shared tree, an ownership or mode that makes an unlisted directory writable, and a future non-macOS shared tree all pass, and each is recorded as unsupported in the research note rather than papered over. The check is worth having anyway because it converts the three mistakes a human actually makes — `/tmp/tiler-cache`, `/Users/Shared/cache`, a bare relative path — from a silent privacy loss into a compile error.

### 6. A missing `HOME` refuses rather than silently disabling the cache

**Decided, and it is the one judgement call in this record.** When no override is stated and `HOME` is unset or empty, the expansion refuses. It neither picks another location nor quietly expands without a cache.

**The counter-argument is preserved rather than dropped.** `tiler-cache`'s own preflight reasoning warns that refusing an unrecognized root "would make an optional accelerator a correctness dependency", and this refusal does exactly that in an environment where `HOME` is absent — a build that would have worked now fails. It was chosen anyway, and Tom accepted it, for three reasons: the alternative is an undiagnosable slowdown rather than an error, the remedy is one environment variable named in the message itself, and item 3's `off` makes that remedy available in every environment including one with no writable location at all. The asymmetry is the point — a refusal costs a consumer one legible action, and a silent disable costs an unbounded number of consumers an unattributed regression.

### 7. Every refusal is typed, names its input, and states both remedies

**Decided.** Four refusal shapes, distinguished so that "could not determine a cache root" is never what a consumer reads:

| Refusal | When |
| --- | --- |
| `OverrideEmpty` | `TILER_EXPANSION_CACHE_DIR` is set to an empty value |
| `NotAbsolute { source, value }` | a stated override, or `HOME`, is not an absolute path |
| `NotPrivate { source, value, shared_tree }` | the stated or derived root is at or under a world-writable tree |
| `HomeUnavailable` | no override is stated and `HOME` is unset or empty |

Each carries which input was wrong and what was wrong with it, and each rendered message names both remedies — an absolute private path, or `off` — so one compile error is a complete instruction without a document. A relative `HOME` is reported against `HOME` rather than against the joined root, because the joined root is Tiler's construction and the variable is the thing the consumer can correct. The exact accepted text of all four lives in the boundary packet on [`choose-the-expansion-cache-root-policy`](../../tickets/choose-the-expansion-cache-root-policy.md) and in the `Display` implementation in `crates/tiler-macros/src/cache_root.rs`; this record does not make a third copy of it.

A relative root is refused for a reason specific to procedural macros: a proc macro runs in the build tool's working directory rather than the consumer's, and `cargo` and `rust-analyzer` need not agree on it, so a relative root would name different directories within one project.

### 8. The policy reads exactly two variables, and that bound is the one-root guarantee

**Decided.** The two-variable observation is not an implementation detail of the current resolver; it is the mechanism by which `cargo` and `rust-analyzer` resolve the same root for the same user.

`rust-analyzer` populates a proc macro's environment from the crate graph it loaded rather than only from the editor's process environment, so `CARGO_PKG_NAME` is present under both drivers and does not distinguish them, while `std::env::current_exe()` does. A policy able to tell the drivers apart could give them different roots, and two roots for one project is precisely the split that makes an editor recompile what a terminal build already published. Reading any third name — `CARGO_MANIFEST_DIR`, `PWD`, anything a driver sets differently — reintroduces that split whether or not it was intended to.

The bound is therefore structural rather than conventional: observation is separated from decision, the environment snapshot carries exactly two names, and a test asserts the resolver reads exactly those two across the whole presence cross-product of the two variables. That cross-product is itself decided rather than incidental — a third read is most naturally written as a *fallback*, and a check that asserted only the both-present case would not see one.

### 9. The root is not part of cache identity

**Decided.** A resolution is a pure function of the composed subject, so moving the root changes where entries live and never what they mean. The root is absent from the cache key, generated Rust embeds completed bytes and never names a cache path, and an already-compiled binary is unaffected by the cache's location or its deletion.

One consequence is stated rather than discovered later: changing the root does not invalidate a Cargo fingerprint, so an otherwise fresh crate is not re-expanded against the new root. That is not a defect under this policy — a stale fingerprint elides an expansion whose result would have been identical.

## Consequences

- The frontend contract states the exact derivation rather than only its shape, and the root half of [Q-ART-004](../open-questions.md#q-art-004--expansion-cache-root-accounting-and-gc-policy) is closed. Its accounting and collection half is untouched and remains with [`decide-the-expansion-cache-collection-schedule`](../../tickets/decide-the-expansion-cache-collection-schedule.md); this record decides *where* the cache lives and nothing about how much of it is kept.
- `implementation_status` is `partial`, and the gap is exact. The resolver exists in `crates/tiler-macros/src/cache_root.rs` with unit tests over every case including every refusal, and nothing calls it: `tensor!` has no grammar and opens no cache, and `tiler-macros` holds no edge to `tiler-cache` (`grep -n 'tiler-cache' crates/tiler-macros/Cargo.toml` reports no match). An accepted policy is not an exercised one, and adding that edge is [`prototype-inline-proc-macro-frontend`](../../tickets/prototype-inline-proc-macro-frontend.md)'s to justify as a dependency-graph change.
- `ExpansionCache::preflight` is still not called on a resolved root. It reports the filesystem properties the publication protocol assumes and deliberately refuses nothing, so it composes with this decision rather than duplicating it: this record decides a root from a path alone, and preflight reports what only I/O can establish. Wiring the two belongs to the slice that opens a cache.
- The evidence behind this record is evidence about a pure decision function. Seventeen unit tests cover the derivation, the precedence, every refusal, and the negatives that keep each check honest; eleven perturbations were applied one at a time and each turned the suite red through the test meant to catch it. Nothing here exercises a real user-cache directory, a sandboxed build, or an actual `rust-analyzer` session reading a root a `cargo` build wrote, and the mode measurements are one host on one date. The research note states that boundary in full.
- **Reopening trigger.** A supported platform other than macOS, which needs its own shared-tree list and its own user-cache derivation — and where an empty list must not be mistaken for the privacy property holding. A second Tiler cache reaching consumers would ask for its own variable rather than generalizing this one, which is the cost item 1 accepted deliberately.

## Alternatives considered

Seven were eliminated, each against the same three tests: the privacy requirement, reachability under both build tools, and whether identical invocations still share compiler work. [The root policy note](../research/cache/root-policy.md) carries each elimination in full.

**A home-directory default with no override.** Eliminated by accepted contract, which requires a documented override for CI and sandboxed builds, and independently by the sandbox case: an environment with no writable `$HOME/Library/Caches` would have no way to state one.

**Environment-only, with no default.** Eliminated because it contradicts the accepted "a default … is used", and because it fails asymmetrically between the two drivers in the ordinary macOS setup — a variable exported from a shell reaches a terminal `cargo build` but need not reach a `rust-analyzer` launched from the Dock, so one project would cache in the terminal and refuse in the editor. Refusing is fail-closed and therefore not *wrong*, which is what makes this the tempting option; it is still the option that makes the accelerator unavailable by default.

**A root derived from `CARGO_MANIFEST_DIR`.** The strongest candidate, and the one the prior measurement points at, because it is the variable measured present under both drivers. Eliminated on three grounds, none of them reachability: a checkout directory is not private to one user; `CARGO_MANIFEST_DIR` names the consumer *package* directory, so a per-package root confines sharing to one package when the frontend contract requires that identical invocations share compiler work across rustc processes; and it writes into the consumer's source tree, which `git status` then reports and every consumer's `.gitignore` has to chase.

**A root derived from the target directory — `OUT_DIR` or `CARGO_TARGET_DIR`.** Eliminated by measurement: [`OUT_DIR` is a build-script variable measured absent in the proc-macro process](../research/macro-environment/proc-macro-build-environment.md), and `CARGO_TARGET_DIR` is unset unless a user sets it, so neither is a derivation at all. The frontend contract already names `OUT_DIR` as the rejected option, and even where reachable it would be wrong: `cargo clean` would delete the cache, turning a cross-invocation accelerator into a per-build-directory one.

**A manifest key — `[package.metadata.tiler] cache-root = "…"`.** Eliminated because it requires the macro to locate and parse the consumer's `Cargo.toml` including workspace inheritance, adding a TOML dependency to the proc-macro crate and reading an input Cargo does not track for freshness. It is also the wrong *place*: a CI or sandbox override is a property of a machine, and a checked-in manifest is shared by every collaborator, so the one thing the override exists for is the one thing a manifest key cannot express.

**Macro syntax — `tensor! { cache_root = "…", … }`.** Eliminated because it puts a host filesystem path into committed source, non-portable across every machine that builds the crate. Worse, invocation tokens are the identity subject, so two developers' paths would make one program two invocations. An accelerator's location is machine configuration rather than program text.

**Detecting the driver and choosing per driver.** Possible — `std::env::current_exe()` is measured to distinguish `rustc` from `rust-analyzer-proc-macro-srv`. Eliminated because it inverts the goal: the value of one root is that the editor resolves what the terminal already published, which the build-tool exercise measured happening. Item 8 is the structural form of this elimination.

## Traceability

[The root policy note](../research/cache/root-policy.md) is the evidence and the reasoning: the derivation, the precedence, the mode measurements, every elimination, the measurement boundary, and the five unsupported cases. [The build-tool exercise](../research/cache/build-tool-exercise.md) set the trigger this decision answers and supplies the reachability measurements the eliminations reason from; [the proc-macro build environment research](../research/macro-environment/proc-macro-build-environment.md) supplies the absent-variable measurements.

[The frontend and proc-macro integration contract](../integration/frontends.md) is the normative home and states this policy in its Compiler cache section. [The Metal AOT backend contract](../backends/metal.md) states the same default's shape for the backend's own reader and deliberately does not restate the derivation, so that one subject keeps one authority. ADR 0082 owns the cache crate whose `open` contract requires a private root, ADR 0050 owns the entry protocol that makes integrity validation a corruption defence rather than an adversarial boundary, ADR 0088 admitted the crate that owns the chooser, and ADR 0075 is why the spellings are Tom's acceptance rather than a derivation. The work records are [`choose-the-expansion-cache-root-policy`](../../tickets/choose-the-expansion-cache-root-policy.md) for the choice, the implementation, and the accepted packet, and [`record-the-expansion-cache-root-policy-decision`](../../tickets/record-the-expansion-cache-root-policy-decision.md) for this record and the contract propagation.
