---
id: make-proof-sidecar-minor-version-compatibility-real-before-the-first-step
title: Make proof-sidecar minor-version compatibility real before the first step
status: deferred
priority: p2
dependencies: []
related: [decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, proof, compatibility, identity, deferred]
---
## User-visible outcome

Before the proof-sidecar format or manifest takes its first minor step, the reader either genuinely validates prior-minor bytes under their own identity/canonical encoding or the format adopts an explicit lockstep replacement rule. A parser does not claim backward-minor compatibility and then reject the old bytes during identity or canonical re-encoding.

## Verified latent gap

At exact base `62df964ef529aadee4649d4eb9c155152b8c92be`, `read_header` and `parse_manifest` accept a minor no greater than the current one, but decoded state does not retain either parsed version. `derive_identity`, `encode_manifest`, and `encode` always use the build's current constants. There is only version 1.0 today, so no current byte population is misread; a first 1.1 step would make the advertised older-minor path unevaluable without redesign.

## Trigger

Activate before any change to `SIDECAR_FORMAT.1` or `MANIFEST_SCHEMA.1`, or before a new wire feature claims backward-minor readability.

## Required decision packet

- Choose version-preserving validation/re-encoding, explicit migration to a new canonical form, or a lockstep major replacement. Do not silently normalize old bytes.
- Define which parsed versions enter content identity, canonicality comparison, failure classification, and producer output.
- Retain exact prior-version fixtures and perturb the version, identity, and re-encode subjects independently.

## Trigger check log

- 2026-08-11 — **not fired.** Both versions remain 1.0. The payload-limit decision changes admission policy only and explicitly does not step either wire version.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. Both versions are constants, so the check is exact: it returns `63:pub(super) const SIDECAR_FORMAT: (u16, u16) = (1, 0);` / `67:pub(super) const MANIFEST_SCHEMA: (u16, u16) = (1, 0);`, confirming the entry's `1.0` claim from source. **Scope the path.** An unscoped `rg -n 'MANIFEST_SCHEMA' crates/` also finds `crates/tiler-artifact/src/program/codec/encode.rs`, where a *differently owned* `MANIFEST_SCHEMA` is `(21, 0)` for the program manifest rather than the proof sidecar; a reader who greps the bare name will see `21` and read this trigger as fired. A pair other than `(1, 0)` in `proof/codec.rs` is the changed answer. Command: `rg -n 'const SIDECAR_FORMAT|const MANIFEST_SCHEMA' crates/tiler-artifact/src/proof/codec.rs`. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
