---
id: repair-envelope-digest-coverage-spike-after-identity-digest-manifest-step
title: Repair envelope digest coverage spike after identity-digest manifest step
status: todo
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

At exact base `98669e8ea9cafc91b3a9139ff821781560c526bd`, `spikes/cache/envelope-digest-coverage/harness/src/main.rs` still locates a variable-width canonical artifact identity at the manifest tail in `EnvelopeFrame::locate` (anchor: `let identity_span = (manifest.1 - identity.len(), manifest.1);`). The current v17 encoder already declares artifact identity by digest rather than carrying the canonical identity preimage there. The locator file is unchanged by the related structured-capability branch: `git diff 98669e8ea9cafc91b3a9139ff821781560c526bd -- spikes/cache/envelope-digest-coverage/harness/src/main.rs` is empty.

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

The underflow occurs before the corruption sweep, so the live spike supplies no coverage evidence on the current manifest. Standalone `cargo check --manifest-path spikes/cache/envelope-digest-coverage/Cargo.toml` still passes, which is why compilation alone does not expose the drift.

## Required delivery

- Re-audit the complete v17 manifest layout and every offset restatement in the spike at the implementation base.
- Replace the retired full-identity-tail assumption with a locator derived from the current identity-digest manifest field and section-descriptor framing.
- Revalidate each corruption class against the exact bytes it claims to perturb; do not merely stop the panic.
- Update the spike documentation where it still claims the manifest carries the full canonical identity.
- Run the quick and exhaustive non-recording forms, deliberately perturb at least one locator/restatement so the check fails with a named boundary, and quote that failure.

## Non-goals

Changing artifact schema or identity, removing the cache bundle digest, recording new performance evidence, or changing cache policy.

## Closes when

Both documented spike modes complete on the current manifest, the locator asserts the current digest-based layout rather than a retired preimage layout, every corruption class is shown to reach its named subject, targeted checks and `tkt lint` pass, and an evidence-sensitive review reports no findings.
