---
schema: "tiler-doc/v1"
id: "tiler.research.cache.root-policy"
kind: "research"
title: "The expansion cache root policy"
topics: ["cache", "frontend", "proc-macros", "artifacts"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "adopted"
adopted_by: ["ADR-0089"]
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.frontend-integration", "tiler.contract.metal-backend"]
depends_on: ["tiler.research.cache.build-tool-exercise", "tiler.research.macro-environment.build-environment"]
ticket: "choose-the-expansion-cache-root-policy"
---

# The expansion cache root policy

**Status:** the choice is made and implemented in `tiler-macros`, and Tom accepted the complete consumer-visible surface on 2026-07-31 under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md); [ADR 0089](../../decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) is the accepted record.

This note closes the root half of [Q-ART-004](../../open-questions.md#q-art-004--expansion-cache-root-accounting-and-gc-policy). It does not touch the accounting and collection half, which [`decide-the-expansion-cache-collection-schedule`](../../../tickets/decide-the-expansion-cache-collection-schedule.md) holds.

## The question, and why nobody could answer it before now

**Fact.** `tiler_cache::expansion::ExpansionCache::open(root)` takes its root from the caller, performs no I/O, and creates no directory; the crate never consults the environment. The reproducible check, unchanged from [the build-tool exercise](build-tool-exercise.md): `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `expansion/harness.rs`, `expansion/fault.rs`, and `expansion/tests.rs`, all `#[cfg(test)]`.

**Inference.** That absence is deliberate and worth preserving: a host-relative decision inside a storage protocol is what would stop the protocol being testable without a host. So *something else* must choose, and the only candidate is the caller. Until 2026-07-31 there was no caller — the build-tool exercise recorded the deferral with the trigger "the first proc-macro frontend crate". [ADR 0088](../../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) admitted `tiler` and `tiler-macros` on that date and fired it.

## What was already decided, and one apparent conflict resolved

Three constraints bound the answer before any of this work started.

**Fact — the root must be private.** `ExpansionCache::open`'s own documentation: "The root must be private to the user running Tiler. Integrity validation handles accidents, partial writes, and non-cooperating cleanup; it does not make a shared writable cache an adversarial boundary, because an attacker able to replace files can construct new internally consistent bytes."

**Fact — the accepted contracts already state the default's *shape*.** [`docs/integration/frontends.md`](../../integration/frontends.md) (`contract_status: accepted`): "A default macOS user cache is used rather than consumer `OUT_DIR`. A documented override supports CI and sandboxed builds." [`docs/backends/metal.md`](../../backends/metal.md) (`contract_status: accepted`) says the same thing in its own words: "The default lives in an OS-appropriate user cache with a CI/sandbox override, rather than consumer `OUT_DIR`."

**Fact — the build-tool exercise's Section 3 says something that reads as the opposite.** Its **Proposal** paragraph is that the choice "must be made explicitly rather than defaulted into a home directory". A macOS user cache *is* in a home directory, so the two texts collide on their face, and the collision matters because the ticket dispatching this work restated the Proposal's wording while citing `frontends.md` as its source.

**Inference — the accepted contracts win, and the Proposal survives with its substance intact.** `AGENTS.md` ranks merged contracts above proposed design text, so an accepted contract is not overturned by a research note's proposal. What the proposal was protecting is nonetheless real and is preserved here: the objectionable thing is a root that arrives *unstated* — inherited from whatever a `dirs`-style crate picks, undocumented, unnamed in any record, and unrefusable when it turns out to be wrong. This note states the derivation exactly, names every input, and refuses rather than substituting. That satisfies both texts, and no accepted sentence is contradicted.

**Fact — the one relevant measurement.** [The build-tool exercise](build-tool-exercise.md) measured that a root derived from `CARGO_MANIFEST_DIR` is reachable under both `cargo` and `rust-analyzer`; that `CARGO_PKG_NAME` does *not* distinguish the two drivers, because the analyzer populates a proc macro's environment from the crate graph it loaded; that `std::env::current_exe()` does distinguish them; and that both drivers shared one working directory in the recorded run. [The macro-environment note](../macro-environment/proc-macro-build-environment.md) measured `OUT_DIR`, `PROFILE`, `TARGET`, `HOST`, `RUSTC`, and every probed `CARGO_CFG_TARGET_*` **absent** during Cargo-driven expansion, with `CARGO_MANIFEST_DIR` and `CARGO_PKG_NAME` present.

## The decision

**Proposal.** An expansion resolves its cache root from exactly two environment variables, in this precedence, and from nothing else:

1. `TILER_EXPANSION_CACHE_DIR`, when set. An absolute path is the root **verbatim**; the exact value `off` means expand with no cache; every other value — including empty — is a typed refusal.
2. Otherwise `$HOME/Library/Caches/ai.moderately.tiler/expansion`.

A root that cannot be derived, is relative, or lies in a tree macOS makes world-writable is a typed refusal a consumer reads at the invocation. There is no third step, no fallback location, and no silent disable.

The implementation is `crates/tiler-macros/src/cache_root.rs`, a crate-private module whose decision function is pure over an environment snapshot. Observation (`RootEnvironment::from_process`) is separated from decision (`resolve`) so that every case, including every refusal, is reachable from a unit test with no filesystem, no process, and no build tool.

### Why the precedence is override-first and total

An override that only applies when the default is unusable would make the CI case depend on the CI machine's `HOME`, which is exactly the variable a CI operator cannot rely on. Override-first also makes the variable a complete answer: setting it determines the root, and no second condition can override the override.

**An empty override is a refusal, not an absence.** Treating `TILER_EXPANSION_CACHE_DIR=""` as unset would silently paper over the common failure it actually represents — a script that computed a path and got nothing. The consumer would receive a working build against a root it did not choose, which is the shape of silence this whole policy exists to eliminate.

### Why a disable value exists

A sandbox with no writable per-user location must be able to say so. Without `off`, the only way to build there would be to point the override at a scratch directory that exists solely to be ignored, and the refusal for a missing `HOME` would name no remedy the environment can actually satisfy. `off` also keeps the refusals honest: every one of them names two remedies, and one of them always works.

`off` is matched exactly — no case folding, no synonyms, no prefixes — so a value that merely resembles the sentinel is refused rather than guessed at. The cache is an optional accelerator, so `off` costs duplicate compiler work and never correctness.

### What "private" can and cannot be decided from a path

**Fact — the default is private by construction.** **Measurement, macOS 27.0 arm64, 2026-07-31:** `ls -ld ~/Library/Caches` reports `drwx------` owned by the invoking user. That is what makes the default satisfy `ExpansionCache::open`'s requirement without any check at all.

**Measurement, same host and date.** `ls -ld` reports mode `1777` — writable by every user, sticky — for `/private/tmp`, `/private/var/tmp`, `/var/tmp`, and `/Users/Shared`. `/tmp` is a symbolic link to `private/tmp` and `/var` to `private/var`. `$TMPDIR` is `/var/folders/<a>/<b>/T/` with mode `0700`, owned by the invoking user.

**Inference.** A root at or under one of those five paths is writable by another user of the machine, and the cache's integrity validation is explicitly not a defence against another user writing internally consistent bytes. Refusing them is therefore a correctness refusal rather than a style preference — and it is affordable precisely because macOS's own per-user temporary directory is not among them, so the ordinary CI remedy (`$TMPDIR`) survives.

**The boundary of the claim, stated rather than implied.** Absoluteness and shared-tree membership are the *only* privacy properties decidable from a path without touching the filesystem. A root that passes is *asserted* private by whoever named it, not proven private by Tiler. Three cases pass this check while being non-private in fact, and each is recorded as unsupported below rather than papered over. The check is worth having anyway: it converts the three mistakes a human actually makes — `/tmp/tiler-cache`, `/Users/Shared/cache`, a bare relative path — from a silent privacy loss into a compile error.

### The root is not part of cache identity

**Inference.** A resolution is a pure function of the composed subject, so moving the root changes where entries live and never what they mean. Two consequences follow, and both were already recorded elsewhere: generated Rust embeds completed bytes and never names a cache path (`docs/backends/metal.md`), and changing the root does not invalidate a Cargo fingerprint, so an already-fresh crate is not re-expanded (build-tool exercise, Section 7). Neither is a defect under this policy — a stale Cargo fingerprint elides an expansion whose result would have been identical.

## Alternatives eliminated

Each is eliminated against the same three tests: the privacy requirement, reachability under both drivers, and whether identical invocations still share compiler work.

**A home-directory default with no override.** Eliminated by accepted contract, which requires a documented override for CI and sandboxed builds, and independently by the sandbox case: an environment with no writable `$HOME/Library/Caches` would have no way to state one.

**Environment-only, with no default.** Every consumer sets `TILER_EXPANSION_CACHE_DIR` or gets no cache. Eliminated on two counts. It contradicts the accepted "a default … is used", and it fails asymmetrically between the two drivers in the ordinary macOS setup: a variable exported from a shell reaches a terminal `cargo build` but need not reach a `rust-analyzer` launched from the Dock, so the same project would cache in the terminal and refuse in the editor. Refusing is fail-closed and therefore not *wrong*, which is what makes this the tempting option; it is still the option that makes the accelerator unavailable by default.

**A root derived from `CARGO_MANIFEST_DIR`** — for instance `<manifest>/target/tiler-cache` or `<manifest>/.tiler-cache`. This is the strongest candidate and the one the prior note's measurement points at, because it is the variable *measured* present under both drivers, and the spike's own macro uses it. Eliminated on three independent grounds, none of them reachability:

- **Privacy.** `CARGO_MANIFEST_DIR` names a checkout directory. A shared build agent, a group-writable source tree, or a repository on a shared volume gives another user write access to it, and `ExpansionCache::open` requires a root private to one user. Reachability was never the binding constraint; privacy is.
- **It destroys the sharing the cache exists for.** `CARGO_MANIFEST_DIR` is the *consumer package* directory, so two packages in one workspace get two roots and two checkouts of one project get two more. `frontends.md` requires that "identical invocations share external compiler work even when expanded in different rustc processes"; a per-package root confines sharing to one package. The key is complete, so cross-project sharing is safe — and it is the entire value of a content-addressed cache.
- **It writes into the consumer's source tree**, which `git status` then reports and every consumer's `.gitignore` has to chase. The accepted contract already rejected the `OUT_DIR` form of this for a related reason.

**A root derived from the target directory** — `OUT_DIR`, or `CARGO_TARGET_DIR`. Eliminated by measurement: `OUT_DIR` is a build-script variable and was measured **absent** in the proc-macro process, and `CARGO_TARGET_DIR` is unset unless a user sets it, so neither is a derivation at all. `frontends.md` names `OUT_DIR` as the rejected option explicitly. Even where reachable it would be wrong: `cargo clean` would delete the cache, turning a cross-invocation accelerator into a per-build-directory one.

**A manifest key** — `[package.metadata.tiler] cache-root = "…"`. Eliminated because it requires the macro to locate and parse the consumer's `Cargo.toml`, including workspace inheritance, adding a TOML dependency to the proc-macro crate and reading an input Cargo does not track for freshness. It is also the wrong *place*: a CI or sandbox override is a property of the machine, and a checked-in manifest is shared by every collaborator, so the one thing the override exists for is the one thing a manifest key cannot express.

**Macro syntax** — `tensor! { cache_root = "…", … }`. Eliminated because it puts a host filesystem path into committed source, where it is non-portable across every machine that builds the crate. Worse, invocation tokens are the identity subject: two developers' paths would make one program two invocations. The cache is an internal accelerator whose paths generated Rust never refers to, and an accelerator's location is machine configuration rather than program text.

**Detecting the driver and choosing per driver.** `std::env::current_exe()` is measured to distinguish `rustc` from `rust-analyzer-proc-macro-srv`, so this is *possible*. Eliminated because it inverts the goal: the value of one root is that the editor resolves what the terminal already published, which the build-tool exercise measured happening. Two roots for one project is exactly the split that makes an editor recompile work a build already did. The implementation encodes this structurally — the environment snapshot carries two names, and a test asserts the resolver reads exactly those two, in every combination of their presence.

## Refusals, and the exact text a consumer sees

Every refusal names the offending input, what was wrong with it, and both remedies, so one compile error is a complete instruction without a document.

| Refusal | When |
| --- | --- |
| `OverrideEmpty` | `TILER_EXPANSION_CACHE_DIR` is set to an empty value |
| `NotAbsolute { source, value }` | a stated override, or `HOME`, is not an absolute path |
| `NotPrivate { source, value, shared_tree }` | the stated or derived root is at or under a world-writable tree |
| `HomeUnavailable` | no override is stated and `HOME` is unset or empty |

The rendered text is stated in full in the module's `Display` implementation and in the boundary packet on the ticket, so that a reviewer reads what a consumer reads rather than a paraphrase.

## Evidence, and how it could have failed

**Seventeen unit tests** in `crates/tiler-macros/src/cache_root.rs` cover the derivation, the precedence, every refusal, the negatives that keep each check honest (a sibling of a shared tree is accepted; a value resembling `off` is not a disable; `$TMPDIR` remains usable), and the two-variable observation bound.

**Measurement — every check was perturbed and watched to fail, 2026-07-31.** Eleven perturbations were applied one at a time to `cache_root.rs`, the package's tests run under each, and the file restored: removing the empty-override guard; matching `off` loosely; removing the absoluteness check; removing the shared-tree check; comparing shared-tree containment textually rather than by component; treating an empty `HOME` as set; removing the relative-`HOME` early return; reading a third environment variable; changing the user-cache components; and dropping a remedy from each of two refusal messages. Each turned the suite red through the test intended to catch it, and the restored tree is green.

**One perturbation found a real defect in a test rather than confirming it.** The two-variable observation check originally asserted the read names for a single snapshot with both variables present. Perturbing the source with a *conditional* third read — `lookup(HOME).or_else(|| lookup("CARGO_MANIFEST_DIR"))` — left the suite **green**, because the fallback never ran in the one case being asserted. The test now covers the whole presence cross-product of the two variables and fails on that perturbation. This is the exact failure `AGENTS.md` requires perturbation for: the check looked like it constrained the policy and did not.

**What this evidence does not establish.** It is evidence about a pure decision function. Nothing here exercises `ExpansionCache::open` on a resolved root, a real macOS user cache directory being created, a sandboxed build, or an actual `rust-analyzer` session reading a root a `cargo` build wrote — the last of those is measured in the build-tool exercise against *its* root, not against this policy's. The filesystem mode measurements are one host on one date.

## Unsupported cases, stated rather than approximated

- **A symbolic link into a shared tree is not detected.** `TILER_EXPANSION_CACHE_DIR=/Users/me/link` where `link` points at `/tmp/shared` passes, because resolving it means touching the filesystem from a decision that must stay pure. Refusing it belongs to a validation step that already does I/O, not to the chooser.
- **Ownership and mode are not checked.** An absolute path outside the named trees that happens to be group- or world-writable passes. Privacy is asserted by whoever named the root.
- **A non-macOS shared tree is not enumerated.** The list is macOS's, and Tiler develops on macOS only. A future platform needs its own list, and the list being empty must not be mistaken for the property holding.
- **`ExpansionCache::preflight` is not called.** It reports the filesystem properties the publication protocol assumes and deliberately refuses nothing; wiring it to a resolved root belongs to the slice that opens a cache.
- **Nothing calls the resolver, and nothing here can.** `tensor!` has no grammar and opens no cache, so the policy is stated and tested but never exercised by an expansion. The frontend cannot even reach the cache to try: `grep -n 'tiler-cache' crates/tiler-macros/Cargo.toml` reports no match, so `tiler-macros` holds no edge to `tiler-cache`. Adding one is `prototype-inline-proc-macro-frontend`'s to justify, and it is a dependency-graph change rather than a detail.

## Outcome

- **An architectural decision, drafted here and recorded separately.** `docs/decisions/[0-9]*.md` is the `contracts/decisions` scope, which the ticket implementing this did not hold, so writing the ADR from that branch would have been a scope escape. [`record-the-expansion-cache-root-policy-decision`](../../../tickets/record-the-expansion-cache-root-policy-decision.md) carried [ADR 0089](../../decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) and the propagation into `docs/integration/frontends.md`. Recording where the boundary is beats taking a scope that another ticket may be holding.
- **An implementation.** `crates/tiler-macros/src/cache_root.rs`, crate-private, awaiting its first caller in `prototype-inline-proc-macro-frontend`.
- **An accepted public boundary.** Tom accepted the complete packet — the variable spelling, the `off` value, the derived path, the precedence, the refusal set, and the refusal text — on 2026-07-31, recorded in ADR 0089.
