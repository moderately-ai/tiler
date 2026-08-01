---
id: expand-a-delivering-region-with-the-cache-disabled
title: Expand a delivering region when the expansion cache is disabled
status: done
priority: p2
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/cache, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, cache, inline-dx]
---
## Why this exists

ADR 0089 accepted `TILER_EXPANSION_CACHE_DIR=off` as meaning "expand with no cache at all", and `CacheRootDecision::Disabled`'s own documentation spells it "expand, compile, embed, and cache nothing". `prototype-inline-aot-integration-proof` narrowed that: a region whose `deliver` statement selects an artifact family refuses under `off`, with `AotRefusal::CacheDisabled` naming the remedy. A region stating `fallback-only`, or stating nothing, is unaffected — it opens no cache at all.

**Fact.** `tiler_cache::expansion::ExpansionCache` has no store-nothing mode. Its constructor is `open(root)`, and every unusable-root path reaches `Resolution::Uncached` through an *attempted* publication rather than through a stated one. `tiler_build::accept_or_publish_metal_plan` requires an `&ExpansionCache`, so there is no cache-free route through the sequencing it owns.

**Inference.** The honest options are a `disabled` constructor on `ExpansionCache` whose every resolution is `Uncached`, or a cache-free path through `tiler-build`. The first is smaller and matches an outcome variant that already exists; both are public-boundary additions.

## Closes when

`off` compiles, embeds, and stores nothing for a region stating a selected family, and a test proves it publishes no file — with the deliberate-failure check that the same expansion *does* publish when a root is stated. `AotRefusal::CacheDisabled` and its diagnostic are removed in the same change, and the narrowing note in the proof's boundary packet is retired.

## Outcome

**Supported.** `ExpansionCache::disabled()` is the store-nothing mode the ticket's Fact said did not exist, and `open_cache` maps `CacheRootDecision::Disabled` onto it instead of refusing. A delivering region under `off` runs the same eight expansion steps, resolves to `Resolution::Uncached`, and embeds the artifact.

### The elimination

The ticket named two candidates and both were tested against correctness and maintainability rather than size.

**A cache-free path through `tiler-build` (rejected).** `accept_or_publish_single_payload_metal_artifact` wraps its correspondence checks *around* `get_or_publish`: the pre-publication decode-and-compare runs inside the miss closure, deliberately before the cache's own governed decode, and a second check runs on the returned resolution. A cache-free route either duplicates that closure — a second authority for exactly the payload-correspondence checks the architectural contract concentrates scrutiny on — or hoists it and adds a second public `tiler-build` function whose natural return is still `Resolution::Uncached`. It is strictly larger, adds an authority that can drift, and lands the same variant. It also falls outside this ticket's scopes (`implementation/build` is not held), which is corroboration rather than the reason.

**A `disabled` constructor on `ExpansionCache` (chosen).** One constructor; `accept_or_publish_metal_plan`'s `&ExpansionCache` parameter stays satisfied; the outcome variant that already means "compiled, validated, not stored" carries it; nothing below the cache changes.

Two further candidates were eliminated before either. A temporary-directory root publishes files and leaks a cleanup obligation, which is the literal opposite of "cache nothing". A frontend that bypassed `tiler-build` entirely would restate emission, compilation, and identity correspondence — the same second-authority defect, larger.

### Why the guarantee is structural rather than remembered

`disabled()` takes no root and stores `Storage::Disabled`, so a disabled cache holds no `Layout` and therefore no `PathBuf`. Every filesystem operation in the crate now reaches its paths through `ExpansionCache::layout() -> Option<&Layout>`, and `publish`, `create_temporary`, `quarantine`, `prepare_directories`, `acquire_lock`, and `release_lock` take `&Layout` as a parameter — so publication is not something a disabled cache declines to do, it is something it cannot express.

`resolve` needed almost no new code: `read_entry` misses with `MissReason::Disabled`, the lock step yields `None` with `PublicationRefusal::Disabled`, and the existing fall-open path produces `Uncached` with the built envelope. One route to `Uncached`, not two.

Every other public operation has a defined answer rather than a fabricated path: `lookup` misses, `evict` reports `Absent`, `sweep_temporaries` and `purge` report empty, `account` accounts for nothing (which makes `collect` total without stating the mode), and `preflight` reports `root: None` with every row `NotRun`, so `all_probed_properties_hold()` is false rather than a vacuous pass.

### Evidence, watched both ways

`a_disabled_cache_delivers_the_region_and_publishes_no_file` (tiler-macros) runs one program over one scratch directory under both environments in order: under `off` the directory holds no bundle, then under the same directory stated as a root it holds exactly one, and both expansions embed byte-identical items.

Four deliberate perturbations were run and each failed as designed:

1. Hand the `off` half the stated-root environment → `a disabled expansion must publish no file` fails, naming the bundle found.
2. Wire `disabled()` to `Storage::Rooted(temp_dir()/…)` → both tiler-cache tests fail on `root()` being `Some`. The tiler-macros test *passes* under this perturbation, in 0.3 s instead of 1.4 s because the hidden cache served a hit. That is the measurement boundary, stated: a directory watch cannot see writes elsewhere, and the `root() == None` assertion is what covers it.
3. `read_entry` returns `MissReason::Absent` when disabled → both tiler-cache tests fail.
4. `resolve` omits `PublicationRefusal::Disabled` → the report assertion fails, because an absent refusal states the result *was* published.

`a_disabled_cache_answers_every_namespace_operation_without_a_root` covers the other seven operations under the tightest collection bound there is (retain no entry), so an accounting that reported anything would be selected for removal rather than pass vacuously.

### Public surface, for the acceptance packet

- **New** `ExpansionCache::disabled()` — a constructor taking no root.
- **New** `MissReason::Disabled` and `PublicationRefusal::Disabled` — variants on two already-`#[non_exhaustive]` enums. The second exists because `CacheReport::publication_refusal()` returning `None` documents that the result *was* published, which would be false.
- **Changed** `ExpansionCache::root() -> Option<&Path>` and `PreflightReport::root() -> Option<&Path>`. Both are consumed only inside `tiler-cache` today.
- **Removed** `AotRefusal::CacheDisabled` and its consumer-visible diagnostic. `AotRefusal` is `pub(crate)`, so this is a removal of a *diagnostic* rather than of a Rust public item; the message named this ticket as its remedy and no longer has anything to say.

### Found and not fixed

`crates/tiler-macros/src/cache_root.rs` carries a crate-level `#![allow(dead_code, reason = "…")]` whose reason states the cache-root resolver "is not yet reached" and that "every region states `FallbackOnly`". Both were already false before this change — `prototype-inline-aot-integration-proof` landed the caller — so this is pre-existing stale prose rather than drift introduced here. Left alone to keep the diff to the ticket; worth a narrow follow-up.

### Commands

`cargo fmt`; `cargo check --workspace --all-targets`; `cargo nextest run -p tiler-cache -p tiler-macros -p tiler -p tiler-build` (301 passed); `cargo clippy` per package with warnings denied; `cargo test --doc`; `tkt lint`; `git diff --check`; `tkt guard --base 189491a`; `make full`.

**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted the surface: `ExpansionCache::disabled()`, the two `Disabled` reason variants (additive on `#[non_exhaustive]` enums), the `root()` accessors returning `Option`, and the removal of the superseded `AotRefusal::CacheDisabled` diagnostic. The coordinator also removed `cache_root.rs`'s stale crate-level `dead_code` allow in the same landing — the resolver has been production-called since the AOT proof, so the allow and its falsified reason came off cleanly.
