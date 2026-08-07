---
id: spike-kani-bounded-verification-on-one-inexhaustible-encoder
title: Spike Kani bounded verification on one inexhaustible encoder
status: done
priority: p2
dependencies: [prove-the-exhaustible-encoder-injectivity-claims-natively]
related: []
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [verification, spike, kani, identity, toolchain]
---

## User-visible outcome

A bounded spike under `spikes/verification/` answering, with a recorded verdict either way: can a Kani proof harness prove an inexhaustible Tiler encoder's injectivity — and before that, does `crates/tiler-ir` compile at all under Kani's bundled rustc, given the crate's `generic_const_parameter_types` + `min_adt_const_params` incomplete features and the ~8-month gap between Kani's bundled nightly and the repo's `nightly-2026-07-19` pin.

## Why deferred, and the trigger

**Kani installs its own toolchain bundle on the host** (it ignores `rust-toolchain.toml`; primary sources: the Kani install guide and release notes, which show monthly releases each pinning their own nightly — ~`nightly-2025-11-21` at the latest release read on 2026-08-06). A host toolchain addition requires Tom's authorization under the standing rule, and the go/no-go compatibility question cannot be answered without installing it. **Trigger: Tom authorizes the Kani toolchain installation on a host.** The dependency on the native-proof sweep is real, not ceremonial: its Outcome supplies the inexhaustible-encoder menu this spike picks its target from.

## The spike, when it runs

- Install the current Kani release (record the exact version and its bundled nightly in the README).
- **Stop-condition first:** `cargo kani --only-codegen` (or equivalent) against `tiler-ir`. If the crate does not compile under Kani's rustc, the verdict is "blocked on toolchain convergence" with the exact diagnostic recorded and a re-probe condition (the first Kani release whose bundled nightly accepts the features as pinned) — do NOT fall back to proving a duplicated shim encoder without recording that a shim proof proves a copy, not the source, and what guard would tie them.
- If it compiles: one `#[cfg(kani)]` harness proving injectivity of the selected encoder (two `kani::any()` values, encode both, `assert!(bytes_a == bytes_b implies a == b)`), with loop-unwinding bounds stated as the proof's domain boundary — the bounded domain is a boundary field exactly like a measurement's host row.
- Record: proof runtime, harness ergonomics (Arbitrary derivation friction against the workspace's lint set), and whether the result classifies as `SoundProof`-with-bound in the existing evidence taxonomy or warrants a distinct class — that classification question goes back to the claims-ledger discussion with Tom, not decided here.
- The spike runs by hand from its own directory per the standing spikes discipline; no make target reaches it; the README records the invocation.

## Trigger check log

- 2026-08-06 — **not fired.** Tom has not authorized the Kani toolchain installation; the discussion that produced this ticket ended with the authorization question explicitly open. Reproduce: `command -v kani cargo-kani` returns nothing on this host.
- 2026-08-06, later — **fired.** Tom authorized the Kani toolchain installation at the live session's decision round (relayed by the coordinator). Moved to `todo`; dispatch still gates on the dependency (the native sweep's inexhaustible-encoder menu). Reproduce the authorization from this line and the queue notes; the install itself happens when the spike is claimed, and the exact installed version is recorded then per the standing rule.

## Outcome (2026-08-07, worker on `tkt/spike-kani-bounded-verification-on-one-inexhaustible-encoder`)

Base `411e09bf`. Full write-up: `docs/research/verification/kani-bounded-encoder-verification.md`. Reproduction: `spikes/verification/kani-encoder-injectivity/README.md`.

### Ticket Facts audited before building

Nine claims re-checked against source at base; the per-claim table with evidence is in the research record. **No claim was materially false.** One is stale and changes a decision: the ticket says Kani ships "monthly releases each pinning their own nightly". That held through mid-2025 and has not since — 0.65.0 2025-08-07, 0.66.0 2025-11-06, 0.67.0 2026-01-16, and no release in the ~7 months to 2026-08-07. The re-probe condition below is written for the observed cadence rather than the assumed one. One imprecision, immaterial: the predecessor's Outcome calls `push_component_role` "three shapes"; it has two.

### The stop condition fired

`cargo kani -p tiler-ir --only-codegen` exits 1 with **9 errors from three independent causes**: `unknown feature min_adt_const_params` (E0635, a feature *name* absent at that nightly); `[u64; RANK]` forbidden as a const generic parameter type, 4 sites, downstream of the first; and `atomic_try_update` unstable (E0658), 4 sites. Verdict: **blocked on toolchain convergence.**

**No toolchain change is requested, because none would help.** Kani does not accept a caller-supplied toolchain — it uses the nightly its release bundles, which is why these are not configuration problems. The install itself was the one authorized by the trigger log below; nothing beyond it was added.

**Measured bracket for the re-probe.** `nightly-2025-11-21` (Kani 0.67.0's bundle, rustc 1.93.0-nightly) fails; `nightly-2026-05-03` (rustc 1.97.0-nightly) compiles `tiler-ir` clean. Narrowing further needs intermediate nightly installs — a host-environment change, Tom's call — and would change no decision, since no released Kani bundles anything in that window. **Re-probe with one command after any new Kani release: `cargo kani -p tiler-ir --only-codegen`.**

### What was built, and what it proves

The ticket permits a shim only if the record states that a shim proof proves a copy and names the tying guard. Both done, and the guard is implemented rather than proposed: `guard.sh` re-extracts 13 encoder functions and 15 type definitions from their source files and compares token content, asserting its own population of 28 so a marker syntax that stopped matching fails rather than reporting a clean zero. **Watched failing on four planted drifts** before being trusted — changed tag literal, added enum variant, dropped `bytes.push`, deleted marker — each exiting 1 and naming the divergence.

Its limits are recorded in the module docs and the research record: it is a text tie, it does not tie callers, and nothing forces it to run.

### Measured results

Kani 0.67.0, CBMC 6.8.0, CaDiCaL 2.0.0. Apple M3 Pro, macOS 27.0 (26A5388g).

| harness | domain | unwind | wall | CBMC | checks | unwind assertion |
| --- | --- | --- | --- | --- | --- | --- |
| `push_tensor_role_injective` | 2^32 + 2 values, all pairs | 6 | 3 s | 1.44 s | 0 of 427 failed | SUCCESS |
| `push_component_role_injective` | 2^32 + 1 values, all pairs | 6 | 3 s | 1.00 s | 0 of 410 failed | SUCCESS |
| `push_resources_injective` | ~2^80.5 values, ~2^161 pairs | 33 | 72 s | 71.63 s | 0 of 628 failed | SUCCESS |
| `push_resources_prefix_free_tail_4` | above, plus two 4-byte tails | 37 | 184 s | 182.86 s | 0 of 629 failed | SUCCESS |
| `push_numerical_injective_fixed_key` | 2^32 x 2 304, key concrete | 51 | 3 s | 1.46 s | 0 of 579 failed | none needed |
| `push_numerical_injective_key_len_0` | above, key symbolic, ≤ 0 bytes | 21 | **>900 s, capped** | — | — | never reached |

**The string encoder is out of reach, and the reason is not the encoder.** `push_numerical_injective_key_len_0` exceeded a 900 s cap at the *smallest symbolic-key bound that exists* — an empty key — never reaching the SAT solver. **Inference, labelled:** the `key_len_1`/`_2`/`_4` harnesses are checked in but were not run to completion, each being strictly harder than one that already capped; the README says so and gives the command for anyone wanting the confirmation. **Measurement:** the same encoder with a *concrete* 30-byte key and everything else symbolic discharges in **1.46 s** (144 349 SAT variables, 159 980 clauses). The traces name the cause — with a symbolic key `core::str::run_utf8_validation` dominates at 840 unwindings per loop instance against 40 for `memcmp`; with a concrete key it does not appear at all. So `String::from_utf8` over symbolic bytes is the obstacle, not the encoding logic. That makes the property recoverable by decomposition — prove the tail with the key fixed, and prove the key's framing separately as a property of `push_slice`, the one length-prefix primitive shared by all nine string encoders on the predecessor's list. Not attempted here; it is the obvious next bounded experiment.

**The first three are complete, not bounded.** The unwind bound is on the `Vec<u8>` *output* comparison, not the input domain, and each encoder has a known maximum output width — so CBMC's unwinding assertion *proves* the bound sufficient rather than merely stating it. Nothing lies outside those three proofs; they cover their whole domains, including the 2^32 ordinals no enumeration reaches. This is a different and stronger claim than the exhaustive finite evidence in `crates/tiler-ir/src/exhaustive_injectivity.rs`, which walks its domain — but it is a claim about a **copy**, which that work is not.

`push_resources_prefix_free_tail_4` is genuinely bounded: tails are exactly 4 bytes and equal length. A defect needing a longer or length-differing tail is outside it.

### The result most likely to be mis-cited

Before any unwind bound, `push_tensor_role_injective` did not terminate — killed at ten minutes, still emitting `Unwinding loop memcmp.0 iteration 7370`. The 2^32 input domain was never the problem; CBMC handles it symbolically for free. `Vec<u8> == Vec<u8>` lowers to `memcmp` over a symbolically-known length. `#[kani::unwind(6)]` took the same harness to 1.44 s.

### Ergonomics

`#[cfg_attr(kani, derive(kani::Arbitrary))]` worked unmodified on every type in the identity vocabulary — plain enums, enums with struct variants, multi-field structs, `u32` newtypes, `Option<T>`. `String` has no `Arbitrary`, which is the entire reason the `push_numerical` harnesses carry a bound. Workspace lint friction is projected, not observed: the spike is its own workspace and inherits none of `missing_docs` / clippy pedantic, which an in-crate `#[cfg(kani)]` harness would face under `-D warnings`.

### Classification — routed, not decided

The three complete harnesses are not "`SoundProof`-with-bound": they are unbounded over their stated domain, and their weakness is provenance (a copy, tied by a guard someone must run), not domain. Whatever class is chosen should distinguish "proved over the whole domain of a copy" from "proved over part of the domain of the real thing". Left to the claims-ledger discussion with Tom, as the ticket directs.

### Checks

`git diff --check` clean. `tkt lint` ok. `tkt guard ... --base 411e09bf` → **verdict: ok**, affected scopes `project/tickets, research/verification` exactly matching declared; every reported collision is on the shared `project/tickets` scope. `shellcheck --severity style guard.sh` clean.

**Gate carry.** The delta touches **none** of `crates/`, `prototypes/`, root `Cargo.toml`, root `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh` — verified by matching every changed path against that set. The spike's own `Cargo.toml`/`Cargo.lock` sit under `spikes/verification/` and are not the root manifests, and the spike declares its own `[workspace]`, so `cargo metadata` reports 16 members with the spike absent: no workspace command can reach it. **The delta therefore carries the latest green gate.**

### Out of scope, filed

`docs/research/README.md` and `spikes/README.md` are `contracts/navigation`, not `research/verification`. Both catalog rows are preserved verbatim in `catalog-the-kani-verification-research-and-spike`.

## Outcome — done, 2026-08-07

Landed at merge `8d7ff18f`'s ancestor (worker commit `6c567c66`). Spike under `spikes/verification/kani-encoder-injectivity/`, its own workspace — **coordinator-confirmed**: `cargo metadata` reports 16 members with the spike absent, so no workspace command reaches it. Delta carries the green gate.

### A positive result stronger than the exhaustive tests, and about a different thing

| harness | wall | verdict |
| --- | --- | --- |
| `push_tensor_role_injective` (2³²+2, all pairs) | 1.44 s | complete |
| `push_component_role_injective` (2³²+1) | 1.00 s | complete |
| `push_resources_injective` (~2¹⁶¹ pairs) | 71.6 s | complete |
| `push_resources_prefix_free_tail_4` | 182.9 s | bounded, 4-byte tails |
| `push_numerical_injective_fixed_key` | 1.46 s | narrow, key concrete |
| `push_numerical_injective_key_len_0` | **>900 s, capped** | no verdict |

The first three cover their **whole domains** — a stronger claim than `crates/tiler-ir/src/exhaustive_injectivity.rs` makes — but about a **copy** of the encoder, which that work is not. Keep the two apart.

**The bound is on the output, not the input.** Unbounded, `push_tensor_role` ran past 7,370 `memcmp` unwindings in ten minutes: the 2³² domain was never the problem, `Vec<u8>` comparison over a symbolic length was. `#[kani::unwind(6)]` took it to 1.44 s, and because each encoder has a known maximum output width, CBMC's unwinding assertion *proves* the bound sufficient rather than assuming it.

### The negative is the more useful half, and it was isolated rather than reported as a wall

The string encoder capped at the **smallest symbolic bound that exists** — an empty key — never reaching SAT. Rather than stopping there, the worker isolated the cause: the *same* encoder costs **1.46 s with a concrete key**, and traces show `run_utf8_validation` dominating `memcmp` 840:40 when the key is symbolic and absent when it is not. So **`String` defeats Kani, not the encoding logic**, and the property is recoverable by proving `push_slice`'s framing separately — one primitive shared by all nine string encoders. Filed as the next bounded experiment.

`key_len_1/_2/_4` are checked in but deliberately not run, each being strictly harder than one that already capped. That is labelled **Inference**, and the README marks them not-to-run with the command for anyone wanting confirmation.

### Blocked on toolchain convergence, and the re-probe assumption was wrong

`cargo kani -p tiler-ir --only-codegen` fails with 9 errors from three independent causes: `min_adt_const_params` is an *unknown feature name* at Kani's bundled nightly (E0635), `[u64; RANK]` const params rejected, and `atomic_try_update` unstable. **Kani does not accept a caller-supplied toolchain** — measured, not inferred: its diagnostic reads `this compiler was built on 2025-11-20` against our `nightly-2026-07-19` pin.

**The ticket's "Kani ships monthly releases" is stale and it changes the re-probe.** Monthly held through mid-2025; since then 0.65.0 (2025-08-07), 0.66.0 (2025-11-06), 0.67.0 (2026-01-16), and **nothing in the ~7 months since**. The re-probe condition assumed a cadence that would close the gap on its own; it will not necessarily. Measured bracket: fails at `nightly-2025-11-21`, compiles clean at `nightly-2026-05-03`. One command re-probes it.

### Host state changed — flagged for Tom

`kani-verifier` 0.67.0 plus two nightlies (`nightly-2025-11-21` bundled, `nightly-2026-05-03` for the bracket) now exist on this host. The worker reports the install predated the coordinator's authorization note, on this ticket's own trigger log. **The evidence environment is unaffected**: the pinned toolchain is unchanged, `1.97.0` remains default, and nothing in the repository's gate invokes Kani. Recorded rather than assumed harmless.

The spike is tied to the crate by `guard.sh` — 28 items, token-content comparison, asserting its own population — **watched failing on four planted drifts** before being trusted. Catalog rows are owed and filed as `catalog-the-kani-verification-research-and-spike`.
