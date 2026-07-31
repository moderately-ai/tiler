---
id: choose-the-expansion-cache-root-policy
title: Choose the expansion-cache root policy
status: done
priority: p2
dependencies: [admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [exercise-the-expansion-cache-under-cargo-and-rust-analyzer, prototype-inline-proc-macro-frontend, prototype-macro-embedding-and-cargo-behavior]
scopes: [implementation/frontend, research/cache, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [cache, frontend, proc-macro]
---
## User-visible outcome

An inline expansion resolves a cache root by a stated, explicit policy rather than by whatever its caller happened to pass, and Q-ART-004 has a live owner instead of a `done` one.

## Why this exists

The [build-tool exercise](../docs/research/cache/build-tool-exercise.md) deferred this with the trigger "the first proc-macro frontend crate. Nothing can decide it earlier, because there is no caller to own the choice." **That trigger fired on 2026-07-31**, when `tiler` and `tiler-macros` were admitted under [ADR 0088](../docs/decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md). The caller now exists and the choice is still unmade.

`docs/open-questions.md`'s Q-ART-004 named [`prototype-expansion-content-cache`](prototype-expansion-content-cache.md) as its owner, and that ticket is `done` with the close condition unmet — the same failure mode Q-ART-008 recorded when its owner closed terminal, which is a question that reads as owned while being unowned in fact. [`add-an-expansion-cache-root-preflight`](add-an-expansion-cache-root-preflight.md) is also `done`; it owns validating whatever root is chosen, never choosing one.

## Implementation keys

**Fact — `tiler-cache` has no default and must not acquire one.** `ExpansionCache::open(root)` takes the root from its caller, performs no I/O, and creates no directory; the crate never consults the environment. The reproducible check the research note states: `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `#[cfg(test)]` modules. A host-relative decision inside a storage protocol is the boundary that keeps the protocol testable without a host, so the chooser belongs to the frontend.

**Fact — `tiler-macros` names no root today.** `grep -rn 'ExpansionCache\|cache' crates/tiler-macros/src/` reports no match. Nothing has been decided by default.

Two constraints are already decided and bound the answer:

- `ExpansionCache::open`'s own contract requires a root "private to the user running Tiler".
- A root derived from `CARGO_MANIFEST_DIR` is *measured* reachable under both build tools, which matters because `rust-analyzer` populates a proc-macro's environment from the crate graph it loaded rather than from the editor's process environment — so a variable the editor carries does not necessarily arrive. The same note measured that `CARGO_PKG_NAME` does not distinguish the two drivers and that `std::env::current_exe()` does.

The choice must be explicit rather than defaulted into a home directory, and a documented override must exist for CI and sandboxed builds — `docs/integration/frontends.md`'s Compiler cache section states both as accepted contract.

**Correction — 2026-07-31, made while doing the work.** The second half of that sentence is right and the first half is not what `frontends.md` says. Its Compiler cache section says "A default macOS user cache is used rather than consumer `OUT_DIR`. A documented override supports CI and sandboxed builds", and `docs/backends/metal.md` — also `contract_status: accepted` — says "The default lives in an OS-appropriate user cache with a CI/sandbox override". "Explicit rather than defaulted into a home directory" comes from the build-tool exercise's Section 3, where it is labelled **Proposal**; `AGENTS.md` ranks merged contracts above proposed design text, so the accepted contracts govern. What the proposal protects survives: the objectionable root is an *unstated* one, not one under `$HOME`. This ticket's answer states the derivation exactly, names every input, and refuses rather than substituting — the reconciliation is written out in [the root policy note](../docs/research/cache/root-policy.md).

## Outcome — 2026-07-31

**The policy.** `TILER_EXPANSION_CACHE_DIR` when set — an absolute path used verbatim, the exact value `off` meaning expand with no cache, anything else a typed refusal — otherwise `$HOME/Library/Caches/ai.moderately.tiler/expansion`. No third step, no fallback location, no silent disable. Implemented in `crates/tiler-macros/src/cache_root.rs` as a crate-private draft whose decision function is pure over an environment snapshot; observation is separated from decision so every refusal is reachable from a unit test with no filesystem.

**Where the reasoning lives.** [`docs/research/cache/root-policy.md`](../docs/research/cache/root-policy.md) — the derivation, the precedence, the disable value, the `1777`-mode measurements behind the non-private refusal, six eliminated alternatives with the ground each fails on, the measurement boundary, and five unsupported cases. `docs/research/cache/build-tool-exercise.md` Section 3 and its deferred item carry the outcome, and one executable check it published (`grep -rn 'ExpansionCache\|cache' crates/tiler-macros/src/` reports no match) is corrected there because this work falsified it.

**Why `CARGO_MANIFEST_DIR` lost despite being the measured-reachable one.** Reachability was never the binding constraint. A checkout directory is not private to one user, and `CARGO_MANIFEST_DIR` names the consumer *package* directory, so a root derived from it confines cache sharing to one package — against `frontends.md`'s requirement that identical invocations share compiler work across processes.

**No ADR was written from this branch, deliberately.** `docs/decisions/[0-9]*.md` is the `contracts/decisions` scope and this ticket holds `implementation/frontend`, `research/cache`, and `contracts/navigation`. [`record-the-expansion-cache-root-policy-decision`](record-the-expansion-cache-root-policy-decision.md) carries ADR 0089 and the propagation into `docs/integration/frontends.md`.

**Evidence.** Seventeen unit tests; eleven perturbations applied one at a time, each turning the suite red through the test meant to catch it, with the tree green before and after. One perturbation — a *conditional* third environment read, `lookup(HOME).or_else(|| lookup("CARGO_MANIFEST_DIR"))` — left the suite green and exposed a real defect in the two-variable observation test, which now covers the whole presence cross-product.

## Boundary packet for Tom

Every item below is consumer-visible and unaccepted under ADR 0075. Each is implemented as a concrete draft so it can be read rather than imagined.

| Item | Draft spelling | Why this and not the obvious alternative |
| --- | --- | --- |
| Override variable | `TILER_EXPANSION_CACHE_DIR` | Not `TILER_CACHE_DIR`: ADR 0082's residue foresees a runtime pipeline-state cache and a compiler plan cache, and a generic name would silently acquire them the day they exist, changing what an existing setting means. The cost is length. |
| Disable value | the exact string `off` | Matched exactly — no case folding, no synonyms — so a value that merely resembles it is a refusal rather than a guess. Strike it and the missing-`HOME` refusal names no remedy a sandbox can satisfy. |
| Default root | `$HOME/Library/Caches/ai.moderately.tiler/expansion` | `Library/Caches` is mode `0700` on macOS, which is what makes the default private by construction. The trailing `expansion` leaves room for a sibling cache. |
| Precedence | override first and total; empty override is a **refusal**, not an absence | An override that applied only when the default failed would make CI depend on the CI machine's `HOME`. `TILER_EXPANSION_CACHE_DIR=""` is the residue of a script that computed nothing, and reading it as unset would hand back a working build against a root nobody chose. |
| Refused as non-private | a root at or under `/tmp`, `/private/tmp`, `/var/tmp`, `/private/var/tmp`, `/Users/Shared` | All mode `1777` on macOS 27.0 (measured 2026-07-31). `$TMPDIR` is `/var/folders/…/T/` at `0700` and stays usable, which is what makes the refusal affordable in CI. |
| Override used verbatim | Tiler appends nothing to a stated path | Appending would put the cache somewhere the consumer did not name, so inspecting or clearing it would look in the wrong place. |

The exact text a consumer sees, in `compile_error!` form:

- **Empty override** — ``` `TILER_EXPANSION_CACHE_DIR` is set to an empty value, which states no cache root; Tiler will not read an empty override as though the variable were unset. Set it to an absolute directory path only you can write, or to `off` to expand without a cache, or unset it to use `$HOME/Library/Caches/ai.moderately.tiler/expansion` ```
- **Relative root** — ``` `TILER_EXPANSION_CACHE_DIR` is set to `relative/cache`, which is not an absolute path, so `tiler::tensor!` cannot resolve its expansion cache root. A proc macro runs in the build tool's working directory rather than yours, and `cargo` and `rust-analyzer` need not agree on it, so a relative root would name different directories in one project. Set `TILER_EXPANSION_CACHE_DIR` to an absolute directory path only you can write, or to `off` to expand without a cache ``` (the same shape names `HOME` when the default derivation is what was relative)
- **Non-private root** — ``` `TILER_EXPANSION_CACHE_DIR` resolves the expansion cache root to `/tmp/tiler-cache`, which lies under `/tmp` — a directory macOS makes writable by every user of this machine. The expansion cache requires a root private to the user running Tiler: it validates every entry against corruption, and that is not a defence against another user writing internally consistent bytes of their own. Set `TILER_EXPANSION_CACHE_DIR` to an absolute directory path only you can write — `$TMPDIR` is per-user on macOS — or to `off` to expand without a cache ```
- **No `HOME`** — ``` `HOME` is unset or empty, so `tiler::tensor!` cannot derive its default expansion cache root `$HOME/Library/Caches/ai.moderately.tiler/expansion`, and it will neither pick another location nor quietly expand without a cache. Set `TILER_EXPANSION_CACHE_DIR` to an absolute directory path only you can write, or to `off` to expand without a cache ```

**The one judgement call worth Tom's eye beyond the spellings.** A missing `HOME` *refuses* rather than silently disabling the cache. The counter-argument, which the root policy note records rather than hides: `tiler-cache`'s own preflight module warns that refusing an unrecognized root "would make an optional accelerator a correctness dependency", and this refusal does exactly that in an environment where `HOME` is absent. It was chosen anyway because the alternative is an undiagnosable slowdown, the remedy is one environment variable named in the message itself, and `off` makes the remedy always available.

## Accepted (2026-07-31)

Tom accepted the complete boundary packet as merged: `TILER_EXPANSION_CACHE_DIR` (verbatim absolute path, exact `off` disable, empty is a refusal), the `$HOME/Library/Caches/ai.moderately.tiler/expansion` default, override-first total precedence, the five-tree non-private refusal set, the verbatim-override rule, and the judgement call that a missing `HOME` refuses rather than silently disabling. `record-the-expansion-cache-root-policy-decision` now records ADR 0089 and propagates the accepted spellings into the frontend contract.

## Public boundary for Tom

Whatever surface states or overrides the root is a new publicly reachable path on the frontend and needs Tom's review under ADR 0075. Present the exact spelling, the override mechanism, and the failure text a consumer sees for an unusable root before acceptance.

## Closes when

A stated root policy is implemented in the frontend with an explicit override; an unusable or non-private root produces a typed refusal a consumer can read rather than a silent miss; the choice and its rejected alternatives are recorded where a reader finds them; Q-ART-004 is retargeted or closed against this work rather than against a terminal ticket; and the deferred item in the build-tool exercise note is updated with the outcome.

## Graph maintenance

- Do not absorb the *validation* of a chosen root: `add-an-expansion-cache-root-preflight` already delivered that and this ticket supplies it an input.
- Keep the measurement questions in `prototype-macro-embedding-and-cargo-behavior`; that ticket presumes a root exists and does not choose one.
- Q-ART-004 also names accounting and GC policy. If this ticket settles only the root, split the remainder rather than closing the question against a partial answer — `decide-the-expansion-cache-collection-schedule` is `deferred` and holds the collection half. **Done 2026-07-31:** `docs/open-questions.md` now splits Q-ART-004 explicitly, marks the root half answered, and leaves the collection half with its own owner. Nothing of the collection half was absorbed.
- **Filed 2026-07-31:** [`record-the-expansion-cache-root-policy-decision`](record-the-expansion-cache-root-policy-decision.md), holding ADR 0089 and the `docs/integration/frontends.md` propagation, because both live in scopes this ticket does not hold. It also carries one stale sentence in accepted ADR 0088 for a `contracts/decisions` holder to judge.
