---
id: avoid-toolchain-resolution-on-a-warm-expansion-cache-hit
title: Decide whether a warm expansion-cache hit may resolve the Apple toolchain
status: todo
priority: p2
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/frontend, implementation/metal-aot]
shared_scopes: [project/tickets, contracts/integrations]
paths: []
tags: [research, performance, inline-dx]
---
## Why this exists

`docs/integration/frontends.md` states that "warm IDE and `cargo check` expansion must avoid `xcrun`". The landed expansion-time AOT flow does not, and the reason is structural rather than an oversight.

**Measurement — macOS 27.0 arm64, `nightly-2026-07-19`, 2026-07-31.** An out-of-tree consumer crate declaring only `tiler`, holding one `deliver macos;` region, built against a private cache root with an `xcrun` shim first on `PATH` that logs every invocation and returns logging wrappers for `metal` and `metallib`:

| pass | `xcrun` calls | `metal`/`metallib` runs | cache |
| --- | ---: | ---: | --- |
| cold root, source touched | 6 | 2 | one bundle published |
| warm root, source touched | 6 | 0 | validated hit |

The six are `--find metal`, `--find metallib`, `--show-sdk-path`, `--show-sdk-version`, `--show-sdk-build-version`, and a second `--show-sdk-path`.

**Fact.** `Toolchain::prepare` performs all six before returning a `PreparedCompilation`, and `CompilationIdentity::new(request, &resolved)` folds the resolved tool versions and SDK identity into the identity the cache is keyed on. Only `prepared.compile()` — the `metal` and `metallib` runs — is inside the miss closure. The resolution therefore *precedes* the lookup by construction: the fingerprint is an input to the key that decides hit or miss.

**Inference.** Skipping the resolution on a warm expansion means deriving the key from something other than the installed toolchain, which would let a hit return an artifact built by a compiler that is no longer present. The contract sentence and the identity requirement are in tension, and which gives way is not a worker's to pick.

## Closes when

One of: the contract sentence is corrected to say what a warm expansion may do, with the identity argument recorded; or a cached toolchain fingerprint with its own validation and invalidation rule is designed and accepted; or the question is deferred with a measured trigger — `rust-analyzer` cold/warm cost remains unmeasured because the component was unavailable.

The six-call figure above is the baseline any remedy is measured against.
