---
id: route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not
title: Route the four workspace raw conformance hashes through the digest authority
status: in-progress
priority: p3
dependencies: []
related: [site-the-governed-digest-so-layered-identity-encoders-can-reach-it]
scopes: [implementation/compiler, implementation/reference, implementation/conformance, implementation/runtime, implementation/workspace, implementation/cargo-lock, implementation/digest, contracts/artifacts, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, identity, conformance]
claimed_from: todo
assignee: worker-external-digest
lease_expires_at: 1787097598
---
## User-visible outcome

Every Cargo-workspace member that reproduces an externally specified raw SHA-256 result uses the one digest implementation, without changing retained external digest bytes or weakening the governed-domain rule.

## Decision — accepted 2026-08-12

Tom accepted [ADR 0111](../docs/decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md) in the live coordination session. Implement its separately typed raw-external boundary:

- `Digest` remains exclusively the result of a Tiler-governed, domain-separated pre-image.
- Add opaque `ExternalDigest`, with no conversion to or from `Digest` and no wire constructor.
- Add `DigestAlgorithm::digest_external_record(bytes) -> ExternalDigest` and route retained `CC_SHA256` evidence through the explicit `DigestAlgorithm::Sha256` variant, never `GOVERNED`.
- Share one private algorithm dispatch inside `tiler-digest`; do not add a second SHA implementation, a synthetic domain, or a consumer dependency on `sha2`.

## Per-Fact audit — 2026-08-12 at `02ab5153`

- **False population and close condition.** The Cargo workspace has **four** handwritten raw SHA-256 copies outside `tiler-digest`, not three: `crates/tiler-compiler/src/governed/contraction_conformance.rs`, `crates/tiler-reference/tests/contraction_profile_cells.rs`, `crates/tiler-conformance/src/envelope.rs`, and `prototypes/serial-sum-run/src/proof.rs`. `Cargo.toml` lists the prototype as a workspace member. Treating it as a reason-recorded residual would contradict this ticket's outcome, so `implementation/runtime` is now in scope and all four copies must be deleted.
- **Verified common subject.** All four copies hash exact little-endian row-major `f32` result bytes and compare them with retained external `CC_SHA256` records. Their FIPS-vector checks establish that the required bytes are ordinary, undomained FIPS SHA-256.
- **Verified producer distinction.** The three crate sites and the prototype reconstruct the bytes independently of the retained string; conformance and the prototype additionally hash device readback. Centralizing only the digest algorithm does not merge the result producer with the external expected value.
- **False future-compatibility premise in the old recommendation.** `DigestAlgorithm::GOVERNED` means the algorithm this Tiler build writes, while the external record means SHA-256 permanently. Calling `GOVERNED.digest(b"", bytes)` would silently follow a future Tiler algorithm change. The exact `Sha256` variant must be stated at every external-record call.
- **Verified authority conflict.** `tiler-digest` currently documents only two governed, domain-bearing pre-image shapes. Its tests use `b""` only to reproduce published vectors. A public empty-domain convention would make a raw external record look like a governed Tiler subject; a distinct result type preserves the subject split.
- **Verified reachability with a contract correction.** Compiler's copy is behind `#[cfg(test)]` and reference's is an integration test, so both need direct development dependencies on `tiler-digest`. Conformance and the prototype use their helpers from device-reaching paths and need direct normal dependencies. ADR 0106 and `docs/architecture.md` currently say conformance reaches `tiler-digest` transitively and deliberately does not name it; once source names the new API, that description must be corrected rather than silently contradicted.
- **Verified identity consequence.** These strings are evidence observations, not artifact or layered identities. Routing them through the same SHA-256 implementation changes no retained bytes, artifact schema, digest tag, identity domain, cache key, or canonical encoding.
- **Verified independent byte-order check.** `crates/tiler-conformance/src/envelope/tests.rs`, anchor `the_digest_helper_reproduces_the_published_vectors`, checks little-endian `1.0f32`, rejects its big-endian spelling, and rejects element reordering. The compiler, reference, and prototype also pin FIPS vectors; the shared implementation's own suite adds complete padding-residue evidence.
- **Imprecise residual description.** `spikes/cache/cache_harness.rs` and `spikes/artifacts/artifact_envelope.rs` contain local `sha256` implementations, and the decoder-allocation harness uses `sha2::Sha256` directly. They are repository-authored but are not Cargo-workspace members or gate authorities. Keep them as explicit standalone-experiment exceptions; do not claim a repository-wide grep is empty.

**Id note.** The stable ticket id still says “two”. Keep it for graph identity; the title and body carry the corrected population.

## Re-audit at the implementation base `f15a1e40` — 2026-08-18

Every Fact above was re-read at this ticket's own base before any edit. Seven verified, one imprecise and repaired.

- **“False population and close condition” — verified.** Still exactly four handwritten copies in Cargo-workspace members, and no fifth appeared in the week since the audit. `grep -rnE "0x6a09_?e667|0x428a_?2f98" --include="*.rs" crates/ prototypes/` returned the four named files and nothing else. **The audit's own grep would have missed them.** The dispatched command was `grep -rn "0x6a09e667\|sha256\|Sha256" crates/ prototypes/ --include="*.rs" -l`; the four copies spell the initial value `0x6a09_e667` with a digit separator, so only the `sha256` alternative matched them, while `0x6a09e667` matched nothing under `crates/` at all. The separator-tolerant pattern is what the delivered census uses, for exactly this reason.
- **“Verified common subject” — verified.** All four hash `to_le_bytes()` of `f32` bit patterns in row-major order and compare the lowercase result with a retained `CC_SHA256` record.
- **“Verified producer distinction” — verified.** Unchanged; centralizing the algorithm did not merge any result producer with its expected value.
- **“False future-compatibility premise” — verified as stated.** `DigestAlgorithm::GOVERNED` is `Self::Sha256` at `crates/tiler-digest/src/lib.rs`, anchor `pub const GOVERNED: Self`, documented “The algorithm this build of the workspace writes.” Every migrated caller names `Sha256`.
- **“Verified authority conflict” — verified.** Confirmed at the source: before this work `tiler-digest` exposed only `digest` and `digest_qualified`, and its only `b""` uses were test fixtures.
- **“Verified reachability with a contract correction” — verified, and both corrections were owed.** `contraction_conformance` is behind `#[cfg(test)] mod contraction_conformance;` in `crates/tiler-compiler/src/governed.rs`; the reference copy is an integration test. Conformance and the prototype use theirs from device-reaching paths. ADR 0106 and `docs/architecture.md` both carried the transitive-only statement and both now carry dated corrections.
- **“Verified identity consequence” — verified and preserved.** No retained string, artifact schema, digest tag, identity domain, cache key, or canonical encoding moved. The domain censuses are untouched; `tiler-digest` still owns no governed domain.
- **“Verified independent byte-order check” — verified.** `crates/tiler-conformance/src/envelope/tests.rs`, anchor `the_digest_helper_reproduces_the_published_vectors`, still checks little-endian `1.0f32`, rejects the big-endian spelling, and rejects element reordering. It now runs against the shared path unmodified.
- **“Imprecise residual description” — IMPRECISE, repaired.** The audit says “the three standalone spike exceptions”. The true standalone population is **eleven source files**: two carry handwritten SHA-256 (`spikes/artifacts/artifact_envelope.rs`, `spikes/cache/cache_harness.rs`, neither inside a Cargo package), and **nine** reach `sha2` directly across **seven** spike packages — `spikes/artifacts/decoder-allocation` (harness and package), `spikes/target-profiles/metal-subgroup-width-route-gate`, `spikes/target-profiles/metal-thread-execution-width`, `spikes/program-planning/qwen3-checkpoint-f32-inputs`, `spikes/program-planning/physical-frontier-budget-calibration`, and `spikes/program-planning/reduction-partition-calibration` (four files). The audit named only the decoder-allocation harness among the `sha2` users. The direction of the error matters: the residual is **larger** than recorded, so a reader taking “three” as the exception budget would have found unexplained hits. None are Cargo-workspace members, so ADR 0111's exclusion still holds for all eleven and the delivered census reaches none of them by construction rather than by exclusion.

Reproduce the repaired residual count:

```sh
grep -rlE "0x6a09_?e667" spikes/ --include="*.rs"        # 2
grep -rlE "use sha2|sha2::" spikes/ --include="*.rs"     # 9
grep -rl sha2 spikes/ --include="Cargo.toml"             # 7
```

## Delivery — 2026-08-18, commit `34425c7f`

### Digest authority

`DigestAlgorithm::compress` is now the one private dispatch; `digest_qualified` and the new `digest_external_record` both run it and differ only in the result they wrap, so no second SHA implementation exists and no consumer depends on `sha2`. `ExternalDigest` is opaque, fixed width, and exposes `as_bytes` and a lowercase `label` only — no public constructor, `from_wire`, `From`/`Into`, comparison bridge, or serialization with `Digest`. The published FIPS-vector reproduction moved onto the external path; the governed-domain, qualified-preimage, tag, padding-branch, padding-residue, and throughput tests stayed on their own subjects. Three tests were added at the owning surface: the two paths agree byte for byte on the same message (the only place a second implementation would be visible, since the result types cannot be compared), the external rendering is fixed-width lowercase, and a reproduction depends on its bytes.

### Site-by-site migration

Every site keeps its `to_le_bytes()` pre-image and its retained strings verbatim; only the algorithm source changed.

| site | edge added | evidence run |
| --- | --- | --- |
| `crates/tiler-compiler/src/governed/contraction_conformance.rs` | dev | `the_contraction_agrees_with_the_reference_and_the_retained_measurement` + 3 others: 4 passed |
| `crates/tiler-reference/tests/contraction_profile_cells.rs` | dev | 9 passed with `--run-ignored all`, including `the_staged_oracle_reproduces_every_retained_profile_digest` — all six retained cell digests reproduced |
| `crates/tiler-conformance/src/envelope.rs` | normal | 83 passed, including the byte-order and element-order case |
| `prototypes/serial-sum-run/src/proof.rs` | normal | 46 passed; the run-time `require_digest_vectors` check is retained, because a proof binary runs on a device host where the test suite need not have |

The two crate-test FIPS assertions and the prototype's run-time vector check were kept rather than deleted. They no longer duplicate the algorithm's suite — they pin the *selection*: that the variant this caller named is FIPS SHA-256, which is a different claim from `tiler-digest`'s claim about its implementation.

### Drift check

`crates/tiler-digest/tests/one_sha_implementation.rs`. It parses the workspace member array from the root manifest (floor of 16 members, each verified to hold a manifest), walks every `.rs` file under those members (floor of 200; **460** found, 458 outside `tiler-digest`), and matches a comment-stripped, whitespace-free view. It refuses any SHA-256 constant or `sha2` reach outside `tiler-digest`, and separately requires each of the four migrated callers to name `digest_external_record` and to name neither `GOVERNED` nor `digest(b""` in code. It lives in `tiler-digest` rather than beside the other workspace censuses in `crates/tiler/tests/` because the one-authority property is this crate's own; it reads sibling sources as files and adds no dependency edge.

### Documents

ADR 0111 `implementation_status` → `implemented`, with a dated implementation note. ADR 0106 §2 keeps its accepted text and gains a dated correction. `docs/architecture.md`'s dependency block now matches `cargo metadata` exactly for all sixteen members, its ADR 0111 paragraph moved from “not yet implemented” to implemented, and its conformance paragraph carries the same correction.

**A second stale claim was found while correcting the first.** ADR 0106 and `docs/architecture.md` both said `tiler-cache` was reached transitively and not named, and both were already false before this ticket: `crates/tiler-conformance/Cargo.toml` declares `tiler-cache` and explains why (`src/publication.rs` names `ExpansionCache`). The architecture dependency block also omitted `tiler-cache` from the `tiler-conformance` row. Both are corrected here because the correction to the same sentence could not be written truthfully otherwise.

### Perturbations, with the text each produced

Each was applied to the subject and reverted; every quoted line is from the run.

1. **`to_le_bytes()` → `to_be_bytes()`, conformance byte-order case.** `envelope::tests::the_digest_helper_reproduces_the_published_vectors` failed: `assertion left == right failed / left: "de620abd6d3615746360c1d15ce3a56291236d37424124054a49d25e947ffc4d" / right: "e00e5eb9444182f352323374ef4e08ebcb784725fdd4fd612d7730540b3e0c8c"`.
2. **`to_le_bytes()` → `to_be_bytes()`, compiler retained comparison.** `w_decode_kv: the reference does not reproduce the retained `direct` result / left: "7d252713146f43b9794765597e8dd59ad662e5576331ba3848673e4570368709" / right: "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f"` — the right-hand side is the retained record, unchanged.
3. **Governed non-empty-domain substituted for the external path** (`GOVERNED.digest(b"tiler.probe.result\0", ...)`). The compiler's retained comparison failed at its selection pin first: `the digest helper reproduces the published empty-string vector / left: "0244549d475dad6ba7fbbbe896015a1dd555d749474041199e3e10ff54036e9c" / right: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"`.
4. **This same perturbation initially passed the census, and that is why the matcher changed.** rustfmt wraps the call, so `GOVERNED` and `.digest` land on different lines and the single-line anchor spanned neither — the AGENTS.md failure mode for a matcher that cannot see a construct that wraps. After moving to a whitespace-free comment-stripped view it fails: `crates/tiler-compiler/...: reproduces an externally specified raw digest record and must reach it through DigestAlgorithm::digest_external_record; it no longer names that path`.
5. **Each census assertion perturbed separately**, so no one of them is carrying the others:
   - alias, external path retained: `crates/tiler-conformance/src/envelope.rs names DigestAlgorithm::GOVERNED in code, which means the algorithm this build of Tiler writes.`
   - empty domain, no alias: `crates/tiler-reference/tests/contraction_profile_cells.rs digests under the empty domain, which spells a raw external subject as a governed Tiler one`.
   - implementation constant in an unrelated member: `the Cargo workspace must hold exactly one SHA implementation, in crates/tiler-digest, and these source(s) reach or transcribe another: crates/tiler-cache/src/lib.rs contains 0x6a09_e667`.
   - direct `sha2` reach: same assertion, `crates/tiler-cache/src/lib.rs contains sha2::`.
   - member array truncated: both tests failed with `the root manifest parsed to 1 workspace member(s), which is fewer than the sixteen this census was written against`.
6. **The missing conversion is compiled, not documented.** Adding `impl From<ExternalDigest> for Digest` made the doctest report `Test compiled successfully, but it's marked compile_fail.` Two intended error codes were also wrong on first writing and said so loudly — `Some expected error codes were not found: ["E0603"]` (the real code is `E0423`) and `["E0277"]` for the comparison case (the real code is `E0308`, because `Digest` implements `PartialEq` only against itself). Both are corrected and pinned.

## Required delivery

### Digest authority

- Factor one private SHA-256 dispatch shared by governed and external calls.
- Add `ExternalDigest` as an opaque, fixed-width result with exact-byte and lowercase-label observation only. No public constructor, `from_wire`, `From`, `Into`, comparison bridge, or serialization with `Digest`.
- Add the explicit external-record method on `DigestAlgorithm`. Document that callers select the algorithm variant named by the external record and that `GOVERNED` is not that authority.
- Move the published FIPS-vector reproduction onto the external path. Retain governed-domain, qualified-preimage, tag, padding, and performance tests on their actual subjects.
- Add a compile-time or source-population check strong enough that the four migrated callers cannot drift back to `GOVERNED.digest(b"", ...)` or a local SHA implementation unnoticed.

### Consumers and dependencies

- Route all four helpers through `DigestAlgorithm::Sha256.digest_external_record` while preserving each `to_le_bytes()` pre-image and retained string.
- Delete all four compression implementations and their obsolete “cannot reach the digest” explanations.
- Add direct development dependencies for compiler/reference and direct normal dependencies for conformance/prototype. Update `Cargo.lock` and the live architecture dependency block atomically.
- Correct ADR 0106's retained transitive-only statement with a dated note rather than rewriting its historical accepted text.

### Evidence

- Re-bind the existing conformance little-endian and row-order test to the shared path.
- Preserve FIPS vector checks at the owning digest surface; consumer tests should pin their subject encoding and retained comparisons, not duplicate the algorithm's complete vector suite without a distinct purpose.
- Perturb `to_le_bytes()` to `to_be_bytes()` and show a retained comparison failing. Separately substitute a governed non-empty-domain result and show a retained comparison failing. Restore both before gates and quote both failure messages in the outcome.
- Census Cargo-workspace sources and assert the only `sha2::Sha256` implementation remains in `tiler-digest`; record the three standalone spike exceptions separately.

## Explicit non-goals

No new `tiler.*` domain, retained digest rebaseline, governed algorithm/tag change, artifact wire or identity-domain step, direct `sha2` dependency in a consumer, conversion between the two digest result types, or required edit of standalone `spikes/` trees.

No attempt to make the external result authentic. It reproduces an externally recorded digest; the provenance and trustworthiness of that record remain the evidence owner's responsibility.

## Closes when

The separately typed raw-external surface is implemented and documented; all four Cargo-workspace copies are deleted and routed through explicit SHA-256 selection; direct dependency records and architecture prose match source; retained strings are unchanged; byte-order, element-order, algorithm-selection, and domain-substitution checks reach their subjects; `tkt guard`, package tests, Clippy, rustdoc, workspace nextest/doc-tests, and the repository gate pass; and the outcome records the exact census and both demonstrated perturbation failures.
