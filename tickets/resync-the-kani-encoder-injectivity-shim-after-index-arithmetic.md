---
id: resync-the-kani-encoder-injectivity-shim-after-index-arithmetic
title: Resync the Kani encoder-injectivity shim after IndexArithmetic
status: done
priority: p3
dependencies: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [verification, spike, kani, identity, maintenance]
---

## User-visible outcome

The `spikes/verification/kani-encoder-injectivity/` shim copies and harness domains match live sources again after `IndexArithmetic` landed on `ResourceRequirements` / `push_resources`, so `./guard.sh` exits 0 and the complete `push_resources_*` Kani proofs attach to current encoder text (or the spike documents a deliberate freeze with a re-probe condition).

## Why this exists

The parent spike's `guard.sh` is a text-tie over 28 copied items. At audit base and current tree it fails with two real drifts: live `ResourceRequirements` gained `index_arithmetic: IndexArithmetic`, and `push_resources` writes `index_arithmetic_tag(...)` before synchronization. The guard is doing its job; the land-time complete `push_resources_injective` and `push_resources_prefix_free_tail_4` proofs no longer attach to live source text. Tensor-role and component-role copies still match. No product crate change is required solely to restore the tie — only spike copies, harness domain arithmetic, and re-run measurements under the already-authorized host Kani install.

The two reported drifts are not the whole copied population the re-sync needs: `IndexArithmetic` and `index_arithmetic_tag` have no existing `@source:` markers, so the guard cannot report them as drift. A faithful self-contained shim adds both and deliberately raises the fail-closed population from 28 to 30.

Reproduce: `cd spikes/verification/kani-encoder-injectivity && ./guard.sh` → exit 1, `2 of 28 copied items have drifted from their sources` naming `ResourceRequirements` and `push_resources`.

## The work

1. Update the shim copies of `ResourceRequirements`, `IndexArithmetic` (and any newly required helpers/tags such as `index_arithmetic_tag`), and `push_resources` so token content matches live sources under the existing `@source:` markers.
2. Adjust harness domain counts if the new field multiplies the input domain (land-time domain lacked `IndexArithmetic`; a single-variant enum scales by 1 but changes encoding width and unwind needs).
3. Re-run `./guard.sh` (expect exit 0 and the deliberately updated population assert of 30 with the same fail-closed discipline).
4. Re-run the affected Kani harnesses (`push_resources_injective`, `push_resources_prefix_free_tail_4`, and any new related proof); record wall/CBMC times, check counts, and unwinding-assertion results on the stated host.
5. Update the spike README (and research record if domain claims move) so land-time domain arithmetic is not left as a live claim about current sources.
6. Watch `guard.sh` fail on at least one planted drift before trusting the re-sync.

## Closes when

`./guard.sh` is clean, the re-run measurements are recorded, and the complete resources proofs either re-attach to live sources or the record states why they do not.

## Non-goals

- Changing production encoders or identity domains.
- Filing a make-gated guard (parent deliberately left the guard un-gated; reopening that is a separate product choice for Tom).
- The `push_slice` symbolic-byte framing experiment (`spike-kani-push-slice-framing-over-a-symbolic-byte-run`).

## Fact audit at dispatched base `678f805e` (2026-08-10)

- **Verified:** both audit base `c99ac54950f2` and dispatched base `678f805e` reproduce exactly the stated two drifts out of 28 guarded items.
- **Verified:** the live source anchors are `pub index_arithmetic: IndexArithmetic` in `crates/tiler-ir/src/schedule/model.rs` and `bytes.push(index_arithmetic_tag(index_arithmetic));` immediately before `push_synchronization` in `crates/tiler-artifact/src/program/model.rs`.
- **Verified:** `IndexArithmetic` has exactly one current variant, `CompleteU64`. It multiplies the domain by one but widens the maximum encoded record from 32 to 33 bytes.
- **Verified:** tensor-role and component-role copies still matched, and the guard's exhaustive report named no other drift.
- **Imprecise, repaired above:** the two newly required copied symbols could not appear in the old guard's two-drift report because they had no markers. Adding `IndexArithmetic` and `index_arithmetic_tag` raises the fail-closed population to 30.
- **False in the parent record, repaired there and in the research/spike records:** the resource cardinality omitted 69 fixed-width head bits. The actual population is about 2^149.512 values and 2^299.024 ordered pairs, not the previously recorded smaller exponents.

No Fact in this maintenance ticket was false. The parent correction changes neither this ticket's purpose nor an identity or public boundary.

## Progress on worker branch (2026-08-10)

The shim copies current `IndexArithmetic`, `ResourceRequirements`, `index_arithmetic_tag`, and `push_resources`; `./guard.sh` reports `30 copied items match their sources.` The complete resource harness uses unwind 34 for the 33-byte maximum record, and the bounded four-byte-tail harness uses unwind 38.

Kani 0.67.0 / CBMC 6.8.0 / CaDiCaL 2.0.0 on Apple M3 Pro, macOS 27.0 (26A5388g), `aarch64-apple-darwin`:

| harness | wall | verification | checks | unwinding assertion |
| --- | --- | --- | --- | --- |
| `push_resources_injective` | 92.85 s | 91.239296 s | 0 of 628 failed | SUCCESS |
| `push_resources_prefix_free_tail_4` | 271.60 s | 269.94125 s | 0 of 629 failed | SUCCESS |

Both retain their prior domain classification: the first covers the whole type-derived resource domain; the second is bounded to two equal-length four-byte tails. Each run classified six checks unreachable and discharged every reachable check. The single-run timings are host-bounded tractability measurements, not performance baselines.

The guard was made to fail by changing the copied `index_arithmetic_tag` payload from `0x01` to `0x02`; it exited 1 with `DRIFT: index_arithmetic_tag` and `1 of 30 copied items have drifted from their sources.` The copied source was restored before the proofs and final guard run.

### Checks and gate carry

- `cargo kani --harness push_resources_injective` and `cargo kani --harness push_resources_prefix_free_tail_4` succeeded with the measurements above.
- In the spike workspace, `cargo fmt -- --check`, `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps` succeeded.
- `./guard.sh` reported all 30 copied items matching; `shellcheck --severity style guard.sh` succeeded.
- From the repository root, `make citations`, `tkt lint --format json`, and `git diff --check` succeeded.

The delta changes only `docs/research/verification/**`, `spikes/verification/**`, and `tickets/**`. It touches none of `crates/`, `prototypes/`, root `Cargo.toml`, root `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh`, so it carries the latest green full gate under the repository rule. The edited spike-local `Cargo.toml` remains outside the root workspace: root `cargo metadata --no-deps --format-version 1` reports 16 members and no package named `kani-encoder-injectivity`.
