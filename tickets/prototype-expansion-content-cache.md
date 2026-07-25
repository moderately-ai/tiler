---
id: prototype-expansion-content-cache
title: Implement the expansion content cache
status: done
priority: p1
dependencies: [prototype-neutral-artifact-codec, prototype-apple-aot-driver, repair-cache-experiment-harness-integrity]
related: [decide-the-expansion-cache-owner-and-digest-authority, implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/metal-aot, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, cache, proc-macro, inline-dx]
---
Implement complete content identity, one immutable bounded bundle per key, validation on every hit, stable per-key advisory locking, post-lock recheck, unique same-filesystem temporary publication, atomic rename, corruption recovery, limits/diagnostics, and race/crash/unwritable tests. Generated code never depends on the cache.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

Complete content identity includes the strengthening `prototype-apple-aot-driver` explicitly deferred here: `tiler_metal_aot::ArtifactProvenance` currently records the portable compiler fingerprint as the `metal`/`metallib` **version strings** plus the resolved local tool paths, and documents that two component builds can report the same front-end version. A content digest of the tool binaries and SDK (and of the produced `metallib`) is what makes artifact identity sound **across hosts**; the driver crate deliberately stayed dependency-free and left it to this cache. Decide and record whether cross-host identity requires that digest, and if so add it here rather than leaving version strings as the only cross-host discriminator.

## Outcome

### What landed

`crates/tiler-metal-aot/src/identity.rs`, a crate-private module under ADR 0074 convention 7, plus its `mod identity;` declaration in `crates/tiler-metal-aot/src/lib.rs`. It is the **complete compilation key subject** ADR 0050 requires the cache to store one immutable bundle per: the canonical, domain-separated, length-prefixed byte subject naming every input that determines the `metallib` bytes a compilation produces, together with the toolchain-evidence class that bounds where an entry keyed by it may be reused. Twelve unit tests; the whole repository gate is green.

The subject is **bytes and not a digest**, following `crates/tiler-metal-aot/src/family.rs`'s precedent exactly: this crate's dependency closure is empty by decision (ADR 0077 item 2), the governed digest is `tiler.digest.sha-256.v1` in `tiler-artifact`, and a digest derived here would be a second identity authority over the same subject. The caller that owns the governed algorithm digests these bytes.

Completeness is enforced by a mechanism rather than by the field list: `CompileRequest`, `ResolvedToolchain`, `SdkIdentity`, and `ResolvedTool` are each destructured irrefutably in the encoder, so a field added to any of them fails to compile until someone decides whether it reaches identity. The exact ordered `compile_flags()` and `link_flags()` are encoded rather than the structured choices they derive from, so an output-affecting flag added to the driver reaches identity with no edit to the encoder. The SDK selector is encoded separately because it is an invocation input that never appears in the flags.

### The cross-host question, answered

**Decision: yes, cross-host identity requires a content digest of the `metal` and `metallib` binaries and of the SDK, and the driver cannot supply one.** Reported versions plus the SDK's canonical name, version, and build identifier are not a sufficient discriminator, for the reason `CompilerFingerprint` already documents — two component builds can report the same front-end version. The resolved tool **paths cannot close the gap and are therefore excluded from the subject**: a path states where a host keeps a file, not what the file contains, so including it would split two hosts with identical toolchains at different paths and would still not join two hosts with different toolchains at the same path. It buys no soundness and costs every legitimate hit.

Rather than leave version strings as an unmarked cross-host discriminator, the identity carries an explicit `ToolchainEvidence` class. Its one inhabited value, `ReportedVersions`, yields `IdentityReuseScope::SameHost`, and `require_cross_host_reuse()` is a typed refusal (`IdentityError::CrossHostReuseUnsupported`) rather than an approximation. A content-digest class is a **type-system reservation, not implemented support and not a tested guarantee**: adding it is a compile error at the encoder, and because the class tag reaches the bytes, no entry published under the weak class can ever be served to a consumer requiring the strong one. The operational consequence, stated on the type: a cache root under `SameHost` must be host-local, never a shared or network volume.

### What did not land, and why it is blocked rather than deferred

The protocol half — namespace, advisory locking, post-lock recheck, unique same-filesystem temporary publication, atomic rename, bundle framing, validation on every hit, corruption recovery, limits, GC, and the race/crash/unwritable tests — **is blocked on an authority a worker cannot resolve**, and every candidate home is closed:

- **A new `crates/tiler-cache`**, which this ticket's own body authorizes, is blocked twice. ADR 0075 is accepted and places "a new publicly reachable namespace — a new crate" in the always-ask-Tom category; and `docs/architecture.md`'s accepted packaging profile says the profile "deliberately omits frontend, proc-macro, Candle, generalized cache, and reusable Metal-runtime crates until the proof reaches those boundaries", which ADR 0077 item 5 restates. The clause in this ticket's body granting it workspace admission does not outrank either.
- **`tiler-metal-aot`**, which `docs/architecture.md`'s Component ownership table names as the owner of the "cross-process content cache, atomic publication, byte embedding", cannot satisfy the assignment. That same row decides the crate's forbidden dependencies are "Every workspace and third-party dependency", `scripts/check_workspace.py` pins `"tiler-metal-aot": []`, and ADR 0050 requires readers to validate "section lengths/digests … on every hit". The governed digest is `pub(crate)` in `tiler-artifact` and unreachable from here even if the closure were opened. **The row is internally unsatisfiable**, and proposed ADR 0077 item 1 separately says this crate "does not implement the expansion cache" at all.
- **`tiler-artifact`** has the digest and the envelope but is outside this ticket's declared scopes and contradicts the ownership table.

`decide-the-expansion-cache-owner-and-digest-authority` (`awaiting-decision`) carries the conflict, the three options with what each enables and prevents, and a recommendation. `implement-the-expansion-cache-protocol` (`blocked` on it) carries the whole remaining protocol. The block was ticketed rather than resolved by picking a side, per `AGENTS.md`.

### Which of the five cache correctness properties this ticket establishes

| Property | State |
| --- | --- |
| Complete cache and artifact identity | **Implemented and tested**, for the driver's half of the key, with the cross-host bound made explicit rather than assumed. |
| Validation on every cache hit | **Not implemented.** It needs the governed section digests; that is the constraint the owner question is fighting. |
| Immutable entries | **Not implemented.** Belongs to the protocol. |
| Atomic publication | **Not implemented.** Belongs to the protocol. |
| Crash/race behaviour | **Not implemented and not claimed.** `spikes/cache/cache_harness.rs` exercises the accepted protocol against the spike's miniature frame on one measured host; nothing here strengthens that, and no test in this change is evidence for a process-crash property. |

### Retraction

This ticket's body states that "if the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update". That premise does not survive checking against ADR 0075 and the accepted packaging profile, and is retracted rather than acted on. `Cargo.toml`, `Cargo.lock`, `scripts/check_workspace.py`, and `ticketsplease.toml`'s `[scope_crates]` are unchanged; `implementation/cache` still has no crate mapping, which remains correct while no cache crate exists.
