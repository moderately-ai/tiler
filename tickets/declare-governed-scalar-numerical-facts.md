---
id: declare-governed-scalar-numerical-facts
title: Declare the governed scalar operations' numerical facts instead of leaving them empty
status: in-progress
priority: p1
dependencies: []
related: [register-governed-scalar-reference-evaluation, reconcile-single-contributor-strict-sum-nan-canonicalization]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, ir, contracts, milestone-0b]
claimed_from: todo
assignee: agent-ir2
lease_expires_at: 1784997589
---
The three governed scalar definitions declare *no* numerical facts, so the rounding and NaN-canonicalization rules a scalar reference oracle and an external index-access lowering provider must implement exist nowhere in the scalar authority.

**Fact (inspected source).** `crates/tiler-ir/src/index/scalar.rs::standard_definition` builds every governed scalar contract with `CanonicalValue::record([])` for both the `facts` and the `conformance` argument of `ScalarOperationContract::new`. All three of `tiler.scalar::constant-f32@1`, `multiply-f32@1`, and `add-f32@1` therefore carry an empty facts record and an empty conformance record. Their only normative statement is the `NormativeDefinitionRef` string — for example `"IEEE 754-2019 binary32 addition; tiler.scalar::add-f32@1"` — which names the IEEE operation and says nothing about which NaN payload a result carries.

**Fact — the semantic layer one level up does declare them.** `crates/tiler-ir/src/semantic/registry.rs` gives `tiler::multiply-f32@1` and `tiler::add-f32@1` the `arithmetic_f32_facts()` record: `"binary32-round-to-nearest-ties-even"`, `CANONICAL_F32_ARITHMETIC_NAN_BITS`, and a contraction-permitted boolean of `false`. `tiler::strict-serial-sum-f32@1` declares `"strict-left-fold"`, `"binary32-each-step"`, and the same canonical NaN bits. Each also carries a `standard_conformance(...)` identity. The scalar operations the governed lowerings emit to realize those very families declare none of it.

**Inference — the requirement currently lives only in implementations.** `register-governed-scalar-reference-evaluation` shipped `FrozenScalarReferenceRegistry::standard()`, whose `add`/`multiply` canonicalize an arithmetic NaN and whose `constant` does not. That behaviour is correct — it is what the tensor-level oracle does, and its test suite pins it against explicit non-canonical NaN bit patterns — but it was derived from the *semantic* operation facts and from reading `tiler_reference::binary`, not from anything the scalar authority states. A second reference capability for `tiler.scalar::add-f32@1`, or a third-party index-access lowering provider emitting it, has nothing to conform to and would be within its declared contract to propagate the host payload instead. That is a divergence the registry would admit without complaint, because `legality::check_authority_conformance` checks *which* scalar operations a region reaches, never what they compute.

**What this ticket must produce.**

1. Populate `facts` on the three governed scalar definitions with the rounding and canonical-NaN statements their semantic counterparts already carry, so the scalar authority is self-contained. `constant-f32` must state the opposite rule explicitly — an exact payload is reproduced verbatim and is *not* canonicalized — because "no fact" and "no canonicalization" are currently indistinguishable and only one of them is true.
2. Give each a `conformance` identity, as `standard_conformance` does at the semantic layer, so a capability can name which conformance revision it implements.
3. Decide whether the scalar facts should be *derived* from the semantic operation facts rather than restated. Two independently written records that must agree are a drift hazard of exactly the kind this repository has already been bitten by; if they stay separate, add a test that the governed scalar facts and the semantic operation facts agree on the canonical NaN payload.

**Expect an identity rebaseline, and treat it as the evidence.** `encode_definition` encodes both `facts` and `conformance` into every scalar definition, so `CanonicalScalarRegistrySnapshotIdentity` changes when they stop being empty. That cascades into `FrozenScalarReferenceRegistry::standard()`'s canonical identity, into `CanonicalLoweringRegistryIdentity`, and into any fixture that pins them. Record the before and after rather than only asserting the new values: a snapshot identity that did *not* move would mean the facts were not encoded.

Blocked-adjacent to `reconcile-single-contributor-strict-sum-nan-canonicalization`, which decides whether a zero-arithmetic-step fold canonicalizes. Do not write a scalar fact that presumes that answer.
