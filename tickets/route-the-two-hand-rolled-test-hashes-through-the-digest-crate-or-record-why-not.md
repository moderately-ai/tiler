---
id: route-the-two-hand-rolled-test-hashes-through-the-digest-crate-or-record-why-not
title: Route the two hand-rolled test hashes through the digest crate or record why not
status: todo
priority: p3
dependencies: []
related: [site-the-governed-digest-so-layered-identity-encoders-can-reach-it]
scopes: [implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The workspace has one hashing implementation, or a recorded reason it has three.

## The finding, from the digest-crate landing (2026-08-06)

**Fact.** `crates/tiler-compiler/src/governed/contraction_conformance.rs` and `crates/tiler-reference/tests/contraction_profile_cells.rs` each carry a hand-written SHA-256, justified in-file by a past ticket's scope constraint ("adding `sha2` would edit `Cargo.lock`, which this work does not own") that no longer describes the tree: `tiler-digest` is now the workspace's bottom crate owning the one governed algorithm, reachable from both sites. Both copies are test-local, self-checked against FIPS vectors, and digest conformance facts rather than identities — the landing judged them non-defects — but they are two second hashing implementations of exactly the kind the digest crate's charter exists to make structurally unnecessary.

## What this must do

Either route both through `tiler_digest` (as dev-dependencies where needed; the conformance facts they compute keep their meaning — derive whether the domain-separation discipline applies to a bare conformance hash or whether these sites legitimately hash undomained), or record at both sites why a local implementation is the right call now that the reachability excuse is gone. Rewrite the in-file justifications to current truth either way.

## Closes when

`grep -rn "fn sha256\|Sha256::" crates/ --include=*.rs | grep -v tiler-digest` returns only sites that name their reason against the current tree, or nothing.
