---
id: correct-the-online-single-pass-softmax-fold-legality-fact
title: Correct the online single-pass softmax fold-legality fact
status: review
priority: p2
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity, correctness]
claimed_from: todo
assignee: agent-softmax-fact
lease_expires_at: 1785978598
---
## User-visible outcome

`tiler::softmax-f32@1`'s registered facts stop asserting that the online single-pass form is a reassociation, so a scheduler reading them cannot consume the reassociation permission and believe the rewrite legal.

## Why this exists

**Fact, read in full at `crates/tiler-ir/src/semantic/softmax.rs`.** The module header states: "The online single-pass form is a reassociation, which is a legality question and not a cost one. Rescaling a running sum whenever the maximum changes regroups the contributor sequence of the *sum*, so it is legal exactly where reassociation is granted." `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM`'s doc comment repeats it, and the registered fact value is the string `a-reassociation-of-the-sum-and-not-a-free-implementation-choice`.

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md), which landed after that text was written, derives the opposite in its Part 2: the online fold is a Horner nesting rather than a re-parenthesized sum. Unrolling it gives contributors `exp(x_j - m_j) * prod_{k>j} exp(m_{k-1} - m_k)`, which share no floating-point value with the two-pass fold's `exp(x_j - m_V)`, so no reassociation permission and no permutation permission reaches the rewrite. It consumes distributivity — for which [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) declines a permission — and the elementary-function identity [the elementary-identity dimension record](../docs/research/numerics/elementary-identity-rewrite-dimension.md) names.

**Inference — the error runs in the dangerous direction, which is why this is p2 rather than tidying.** The doc comment's own stated purpose is that the fact exists "so that a scheduler reaching for it has to consume the permission". A scheduler that reads it consumes *reassociation* and believes itself legal under a registered contract that permits reassociation — which is exactly the false inference [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) item 5 exists to prevent, present in the tree, in identity-carrying data. Today nothing reads the fact, so nothing is currently wrong on any execution path; what is wrong is a claim the next reader will act on.

## This is an identity-domain step and is executed completely or not at all

**Fact.** `encode_operation_definition` (`crates/tiler-ir/src/semantic/registry.rs:2811`) writes `definition.canonical_facts().value()` into the definition projection, currently `tiler.semantic-definition-projection.v5`. That projection feeds the registry snapshot identity, which the compiler's explain request qualifier pins — [Numerical semantics](../docs/numerical-semantics.md) records a prior fact-record change advancing "the definition projection to v5, the registry snapshot to v7, and the standard semantic provider to revision 7".

So the change moves a pinned identity. Per AGENTS.md the whole step lands in one commit: the fact string, the version at its owning layer if the encoding changes, the ledger documents, and every pinned identity recomputed on the tree the step lands into with each moved pin enumerated in the report. **A changed fact value with unmoved pins is worse than no change**, because it is a stepped meaning under an unstepped identity.

Whether a *value* change inside an existing record shape advances the projection version at all, or only the resulting digests, is the first thing to establish by reading rather than to assume: a version counts rendering revisions, and this is not one.

## What the corrected claim says

The fact should state what the fold actually consumes rather than deleting the warning, because the warning's purpose — that a scheduler reaching for the form must not treat it as free — is correct and only its *reason* is wrong. The replacement names the distributivity dimension and the elementary-function identity, states that no reassociation or permutation permission reaches the rewrite, and stays a fact string rather than acquiring a structured vocabulary the tree does not have. The module header's paragraph and the constant's doc comment move with it, and the wording follows the refusal discipline the dimension record's Part 7 specifies: a rewrite consuming more than one missing dimension names all of them.

## Non-goals

Admitting any permission; adding an elementary-identity dimension to any type; implementing the online fold; changing `docs/numerical-semantics.md`, which is `contracts/numerics`.

## Closes when

The registered fact, its doc comment, and the module header agree with the certified-bounds derivation; every pinned identity the change moves is recomputed in the same commit and enumerated; and the retained softmax tests still assert the wall they assert today.

## Outcome

**The corrected fact, and what grounds each clause.** `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` now registers `not-a-reassociation-of-the-sum-but-a-horner-nesting-consuming-distributivity-which-no-permission-grants-and-the-subordinate-exponentials-elementary-function-identity-which-no-declared-dimension-names-so-no-reassociation-or-permutation-permission-reaches-it`. The Horner-nesting reading and the "shares no floating-point value with the two-pass contributor" step are [the certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 2; that distributivity is a named dimension no permission grants is [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) plus [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md), both `accepted`; that the exponential's functional equation is named by *no declared dimension* is [the elementary-identity record's](../docs/research/numerics/elementary-identity-rewrite-dimension.md) Part 1, and the wording deliberately stops there rather than calling it a dimension, because that record's ADR is a draft with a live carrier and no `docs/decisions/` entry names it — checked with `grep -rln 'elementary identity' docs/decisions/`, which returns nothing. Naming both freedoms rather than one is that record's Part 7 requirement.

**The identity-domain step, executed completely, and the version question settled by reading rather than assumed.** No encoding version moved, and each of the three was checked at its owning site. `tiler.semantic-definition-projection.v5` (`crates/tiler-ir/src/semantic/registry.rs:1784`) and `tiler.semantic-registry.v7` (`:2656`) count *rendering* revisions; this change adds, removes, and reorders nothing, and `CanonicalValueData::Utf8` encodes as tag `7` followed by `push_slice` (`crates/tiler-ir/src/semantic/types.rs:996`), so a payload of any length stays injective under the unchanged rendering. The standard semantic provider stays at revision 7 on its own documented rule (`registry.rs:2240-2255`): the revision moves only "for a change this registry's *content* encoding cannot already carry", the projection folds every definition's facts, and "bumping it for a change the projection already carried would invalidate every pinned provenance for an authority change that did not happen".

**One pinned identity moved, and it is enumerated.** `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`'s request qualifier, `45467875b9574962` → `a95ad77532352d7f`, rebaselined in `crates/tiler-compiler/src/explain.rs` in the same commit with its ledger entry. The request subject folds the frozen semantic registry snapshot, which encodes every registered definition's facts, so a fact-value change must move it. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and read the reported `left`. The population was surveyed before editing — `grep -rnE '"[0-9a-f]{16}"|request=[0-9a-f]{16}' crates/ --include='*.rs'` and `grep -rlE '\b[0-9a-f]{64}\b' crates/ --include='*.rs'` — and the whole-workspace run (2675 tests, green) is the check that no other pin folds these bytes.

**`implementation/compiler` was added to this ticket's scopes, and the reason is that the step cannot be completed without it.** The only pinned identity a registered-fact change reaches is a test expectation in `tiler-ir`'s downstream crate, so an `implementation/ir`-only claim can land the fact but not the pin, and a stepped meaning under an unmoved pin is the half-step AGENTS.md rates worse than none. **The disjointness verdict against the live holder is *vacuous*, not clear:** `widen-compile-governed-s-error-to-the-target-compile-failure` holds `implementation/compiler` exclusively and its branch has no commits, so `git diff --name-only de377fb1...tkt/widen-compile-governed-s-error-to-the-target-compile-failure` returns nothing and evidences nothing about what it will touch. The integrator sequences these two and recomputes this digest on the merged tree rather than taking either side's value.
