---
id: relax-clippy-for-prototypes
title: Disable too_many_lines and stop linting the prototypes
status: done
priority: p3
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets, implementation/runtime]
paths: []
tags: [tooling]
---
Tom's request, after `clippy::too_many_lines` blocked a `Display` impl during `prototype-metal-runtime-proof`.

## What changed

**`clippy::too_many_lines` is off workspace-wide**, in `[workspace.lints.clippy]` and mirrored into `prototypes/serial-sum-run`, which cannot inherit `[lints]` because it needs `unsafe_code = "deny"` rather than the workspace's `forbid`.

The reasoning is worth keeping, because it is not "the lint was inconvenient". Every function it flagged was an exhaustive `match` over a deliberately wide error enum. The only way under a line limit there is a wildcard arm, and that wildcard is exactly what stops a newly added variant from being a build error — it would render as whatever the catch-all says instead. A length limit is a proxy for complexity, and in this shape it argued for trading a real correctness property for a number. Genuinely long functions should still be split; they should not be split by adding a catch-all.

**`make lint` no longer covers `prototypes/`.** Both prototype packages are excluded from the Clippy pass. They are non-published, experimental, and rewritten or deleted as the slice they prove moves, so holding them to the crates' style bar produced edits rather than defects.

**The exclusion is narrower than it looks, and that is the point.** `make build` still compiles them with `--all-targets` and `make test` still runs them, so a prototype that stops building or stops passing still fails the gate. Only the style pass skips them. The `rustc` lints in their own manifests — including `missing_docs` and the `unsafe_code = "deny"` that makes ADR 0079's single admitted site work — are unaffected, because those are enforced by the compiler during the ordinary build rather than by Clippy.

## Known consequence

A style regression in a prototype is now invisible to the gate. That is accepted deliberately rather than overlooked: the prototypes are the code most likely to be discarded, and they are already the only place in the workspace permitted to diverge on lints at all.

The site-local `#[allow(clippy::too_many_lines)]` added minutes earlier in `prototypes/serial-sum-run/src/proof.rs` is removed, since it is now redundant twice over.

## Verification

`make full` green: 965 nextest tests, 11 doc-tests, rustdoc, the release numerical tests, `tkt lint`, shellcheck. `make lint` confirmed to still cover all eight crates under `crates/`.
