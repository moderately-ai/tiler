---
id: repair-envelope-digest-coverage-spike-after-identity-digest-manifest-step
title: Repair envelope digest coverage spike after identity-digest manifest step
status: done
priority: p1
dependencies: []
related: [replace-flat-selected-lowering-capability-keys-with-structured-subjects]
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, evidence, cache, artifact]
---
## User-visible outcome

The envelope-digest coverage spike runs its documented quick and exhaustive probes against the current artifact manifest without panicking, and its corruption classes still reach the boundaries they claim.

## Exact-base evidence

Re-audited at implementation base `e2522345d571d5088ce47039e4399b7247e7bc47`. `spikes/cache/envelope-digest-coverage/harness/src/main.rs` still located a variable-width canonical artifact identity at the manifest tail in `EnvelopeFrame::locate` (retired anchor: `let identity_span = (manifest.1 - identity.len(), manifest.1);`). The current schema-18 encoder declares artifact identity by a fixed-width unframed digest rather than carrying the canonical identity preimage there (anchors: `MANIFEST_SCHEMA: (u16, u16) = (18, 0)` and `bytes.extend_from_slice(identity_digest(algorithm, identity).as_bytes());`). The locator file remained unchanged across the related structured-capability landing: `git diff --exit-code 98669e8ea9cafc91b3a9139ff821781560c526bd e2522345d571d5088ce47039e4399b7247e7bc47 -- spikes/cache/envelope-digest-coverage/harness/src/main.rs` exits zero. The earlier ticket wording's `v17` was false at the implementation base; the related landing raised the manifest to 18.0 without changing the already-stale locator.

Reproduce from the repository root:

```sh
cd spikes/cache/envelope-digest-coverage
cargo run --release -- --quick
```

Observed failure:

```text
thread main panicked at harness/src/main.rs:1067:22:
range start index 18446744073709531794 out of range for slice of length 53657
```

The underflow occurs before the corruption sweep, so the live spike supplies no coverage evidence on the current manifest. Standalone `cargo check --offline --manifest-path spikes/cache/envelope-digest-coverage/Cargo.toml` still passes after Cargo refreshes the separately stale nested lockfile, which is why compilation alone does not expose the drift. With `--locked`, both commands instead stop before compilation because that lockfile does not yet name `tiler-digest`; the lockfile repair is owned by its separate ticket and is not part of this one.

## Required delivery

- Re-audit the complete v18 manifest layout and every offset restatement in the spike at the implementation base.
- Replace the retired full-identity-tail assumption with a locator derived from the current identity-digest manifest field and section-descriptor framing.
- Revalidate each corruption class against the exact bytes it claims to perturb; do not merely stop the panic.
- Update the spike documentation where it still claims the manifest carries the full canonical identity.
- Run the quick and exhaustive non-recording forms, deliberately perturb at least one locator/restatement so the check fails with a named boundary, and quote that failure.

## Non-goals

Changing artifact schema or identity, removing the cache bundle digest, recording new performance evidence, or changing cache policy.

## Closes when

Both documented spike modes complete on the current manifest, the locator asserts the current digest-based layout rather than a retired preimage layout, every corruption class is shown to reach its named subject, targeted checks and `tkt lint` pass, and an evidence-sensitive review reports no findings.

## Outcome

**Fact — repaired locator.** `EnvelopeFrame::locate` now asserts manifest schema 18.0, derives the trailing identity declaration as exactly `DIGEST_BYTES`, reproduces it from the decoder's identity under `tiler.artifact-envelope.identity-digest.v1\0`, and locates the counted descriptor table immediately before that field from the declared section count and fixed descriptor width. It validates each descriptor's identifier and length against the framed section stream.

**Fact — corruption map.** Every `Class` now declares the exact public artifact-decoder boundary it must reach. The harness asserts the fixed/per-section class census and emits `corruption-boundary-map` only after all 35 current classes reach their named boundary. The spike README maps every header, manifest, framed-section, re-sealed, structural, and substitution family to the exact subject and required result; retained pre-schema-15 result labels remain explicitly historical.

**Measurement — non-recording runs on 2026-08-16.** `cargo run --offline --release -- --quick` completed with all 35 classes mapped and refused all `1,108/1,108` quick perturbations. `cargo run --offline --release` completed with the same class map and refused all `107,314/107,314` exhaustive perturbations. Both runs restored the unperturbed cache hit; neither used `--record` or changed the retained result files.

**Negative controls.** Changing the restated identity-digest domain from `v1` to `v2` failed before the sweep with `assertion left == right failed: the restated identity-digest domain reproduces the manifest's declaration`. Redirecting the `header-magic` class's perturbation from byte 0 to the total-length field, while leaving its declared boundary intact, failed with `corruption class \`header-magic\` expected the named artifact boundary \`BadMagic\` but reached \`artifact.malformed: TotalLengthMismatch { declared: 53656, actual: 53657 }\``. Both subjects were restored before the passing runs.

**Verification.** The spike's format check, `cargo check --offline`, `cargo test --offline`, Clippy with warnings denied, and rustdoc with warnings denied pass. `tkt lint --format json`, `make citations`, and `git diff --check` pass. Cargo refreshed the separately stale nested `Cargo.lock` during each spike command; its original bytes (SHA-256 `14b0edd8aa0afa0ae251f244d1350f62b42b59c34013237aadae51b7f96be4c0`) were restored after verification.

**Unsupported cases.** This remains exhaustive finite evidence for two one-byte perturbations at every position of one current three-section fixture, plus the named multi-byte structural/substitution classes. It does not prove arbitrary multi-byte corruptions or different section populations, change production identity/schema/cache policy, repair the nested lockfile, or record new performance evidence.
