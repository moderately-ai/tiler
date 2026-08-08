---
id: pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate
title: Pin the identity domain strings so a reverted domain reddens the gate
status: todo
priority: p1
dependencies: []
related: [resolve-semantic-shape-inference-over-symbolic-extents, size-the-four-hand-written-metal-all-arrays-from-their-types]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, tests, versioning]
---

**No test asserts any identity domain string.** A domain can be reverted, mistyped, or left un-stepped and the full gate stays green. Identity domains are this repository's core versioning mechanism, so the one thing that must not drift silently is the thing nothing checks.

## Facts, coordinator-verified at `3ac5dbf6`

**Fact.** `grep -rn` for `tiler.semantic-graph.v3` across `crates/` returns three occurrences: the declaration `const GRAPH_DOMAIN: &[u8] = b"tiler.semantic-graph.v3\0"` in `crates/tiler-ir/src/semantic/identity.rs`, and two doc comments. `tiler.index-region.v11` likewise returns four: the declaration in `crates/tiler-ir/src/index/builder.rs` and three doc comments. **Every occurrence is a declaration or prose. None is an assertion.**

**Fact — demonstrated, not inferred.** The worker that stepped `tiler.semantic-precondition-obligation.v1` to `v2` reverted the constant to `v1` and ran the suite: **3,181 tests passed**. The revert is invisible to the gate.

**Fact — the reason a length check would not have caught it either.** That same step moved three obligation subjects; two were **length-identical** across the domain change, because a rank-zero boundary has no extent to tag and `v1` and `v2` are the same width. Only comparing bytes shows the domain moved. A pin that compares lengths is not a pin.

## Why p1

Every other identity discipline in this repository rests on the domain being right: `derive_identity`, artifact publication, cache subjects, and the manifest schema all fold a domain string. A wrong domain silently makes two different subjects share an identity or one subject present as two. It is the highest-consequence, lowest-visibility value in the tree, and `AGENTS.md` already demands "name and count populations so 'nothing ran' cannot look green" — this population is named nowhere and counted nowhere.

## What closes this

An assertion over the domain strings that fails when one changes. **Prefer a census over a list**: a hand-written list of domains is the same defect one level up, and `AGENTS.md` says to size enumerations from the type rather than by hand. Something that enumerates the declared domain constants and asserts each expected value — the shape `crates/tiler-artifact/src/domains.rs` already uses for its governed set, which is the nearest precedent and worth reading first.

**Do not pin only the domains this crate owns.** `tiler.index-region.v11` is declared in `tiler-ir` but three of its four mentions are elsewhere, including `crates/tiler-build/src/metal_plan.rs`. Report which domains fall outside `implementation/ir` rather than reaching into other scopes.

**Perturb each domain separately and quote the failure text.** Revert one constant at a time and show what the assertion said; a perturbation that reddens everything cannot show which pin is load-bearing. Then confirm the reverse case is reachable — state what it would take for this check to say *no* when a domain is correct.

**A deliberate step must stay cheap.** The point is not to make version steps hard; it is to make them *visible*. If the assertion forces a worker to edit five places to step one domain, it will be worked around. Design it so a legitimate step is one edit and an accidental revert is a failure, and say in the report which of those two you optimized for.
