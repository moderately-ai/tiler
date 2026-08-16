---
id: refresh-envelope-digest-coverage-spike-lockfile-after-tiler-digest-extraction
title: Refresh envelope digest coverage spike lockfile after tiler-digest extraction
status: in-progress
priority: p1
dependencies: []
related: [replace-flat-selected-lowering-capability-keys-with-structured-subjects, repair-envelope-digest-coverage-spike-after-identity-digest-manifest-step, site-the-governed-digest-so-layered-identity-encoders-can-reach-it]
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [maintenance, evidence, cache, dependencies]
claimed_from: todo
assignee: worker-envelope-lock
lease_expires_at: 1786927429
---
## User-visible outcome

The envelope-digest coverage spike's committed dependency graph agrees with the manifests it builds, so a locked check succeeds without rewriting tracked evidence.

## Exact-base evidence

Verified at exact implementation base `6ee6c133b2d56e9a403dc126190558b168c754ce`. The implementation-base Fact audit found the earlier `98669e8ea9cafc91b3a9139ff821781560c526bd` provenance imprecise for this checkout but found no change to the ticket's purpose: `crates/tiler-artifact/Cargo.toml` says `` `sha2` was here until ADR 0104 moved the governed digest to `tiler-digest` ``, and `crates/tiler-ir/Cargo.toml` also declares `tiler-digest.workspace = true`. The spike's checked-in `spikes/cache/envelope-digest-coverage/Cargo.lock` still records `tiler-artifact` depending directly on `sha2`, carries no `tiler-digest` package, and omits the `tiler-ir` to `tiler-digest` edge.

From that exact-base worktree:

```sh
cargo check --manifest-path spikes/cache/envelope-digest-coverage/Cargo.toml --offline
git status --short
git diff --numstat -- spikes/cache/envelope-digest-coverage/Cargo.lock
```

Observed output includes:

```text
Locking 1 package to latest Rust 1.99.0-nightly compatible version
Adding tiler-digest v0.0.0 (.../crates/tiler-digest)
 M spikes/cache/envelope-digest-coverage/Cargo.lock
9	1	spikes/cache/envelope-digest-coverage/Cargo.lock
```

The exact generated delta replaces `tiler-artifact`'s stale `sha2` edge with `tiler-digest`, adds the local `tiler-digest` package whose implementation dependency is `sha2`, and adds `tiler-digest` to `tiler-ir`. A plain check exits zero while mutating tracked evidence; `--locked` is therefore the closing control, not an optional stricter mode.

## Required delivery

- Re-audit the complete spike manifest and lockfile at the implementation base against the local path crates' actual manifests.
- Refresh only the spike-owned lockfile to the dependency graph Cargo resolves; do not change dependency declarations to fit stale locked bytes.
- Run `cargo check --manifest-path spikes/cache/envelope-digest-coverage/Cargo.toml --locked`, then prove a second identical command leaves `git status --short` empty for the lockfile.
- Temporarily remove the `tiler-digest` package or one required local edge from the refreshed lockfile, run the locked command, and quote Cargo's refusal before restoring it.
- Run targeted formatting where applicable, `tkt lint`, `make citations`, `git diff --check`, and `tkt guard` against the exact base.

## Non-goals

Changing the production digest crate, workspace dependency topology, artifact identity or schema, the spike's manifest/source/measurement records, or repairing the separate digest-manifest locator panic owned by `repair-envelope-digest-coverage-spike-after-identity-digest-manifest-step`.

## Implementation evidence

At exact base `6ee6c133b2d56e9a403dc126190558b168c754ce`, an offline check regenerated exactly the recorded 9-addition/1-deletion graph repair. The refreshed lock changes `tiler-artifact` from `sha2` to `tiler-digest`, adds the local `tiler-digest` package with its `sha2` edge, and adds the `tiler-ir` to `tiler-digest` edge. Its SHA-256 is `b28c9d24dcbc600d89d177d55342a47e8d3b13b2d7d32a47fdedf1097d985948`.

Two successive locked checks exited zero, and the second preserved both that digest and the 9-addition/1-deletion diff. As an independent negative control, removing only the generated `tiler-digest` package stanza made the same locked check exit 101 with ``cannot update the lock file ... because --locked was passed to prevent this``; restoring the stanza restored the exact digest above. A locked dependency-tree read confirmed both `tiler-artifact -> tiler-digest -> sha2` and `tiler-ir -> tiler-digest`.

## Closes when

The committed spike lockfile is exactly the path-dependency graph resolved from current manifests, the locked check succeeds twice without a tracked rewrite, the deliberately stale negative control fails closed, no file outside `spikes/cache/envelope-digest-coverage/Cargo.lock` and the owning ticket changes, and an evidence-sensitive review reports no findings.
