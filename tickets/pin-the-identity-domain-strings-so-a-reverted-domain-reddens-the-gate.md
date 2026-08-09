---
id: pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate
title: Pin the identity domain strings so a reverted domain reddens the gate
status: done
priority: p1
dependencies: []
related: [resolve-semantic-shape-inference-over-symbolic-extents, size-the-four-hand-written-metal-all-arrays-from-their-types, pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach, pin-the-tiler-artifact-identity-domain-bytes-the-existing-census-does-not-fix]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, tests, versioning]
---

**No test asserted the three identity domain strings this ticket names.** A domain with no digest golden folding it can be reverted, mistyped, or left un-stepped and the full gate stays green. Identity domains are this repository's core versioning mechanism, so the one thing that must not drift silently was the thing nothing checked.

## Facts, coordinator-verified at `3ac5dbf6`, re-audited by the worker at `c0829b41`

**Fact — false as written, corrected.** The opening claim was "**No test asserts any identity domain string**". Three do directly, at `c0829b41`: `crates/tiler-ir/src/kernel/tests.rs` binds `let domain = b"tiler.kernel.v7\0"` and asserts two canonical identities start with it; `crates/tiler-ir/src/semantic/catalog/tests.rs` asserts every value-type descriptor `starts_with(b"tiler.value-type-descriptor.v1\0")`; and `crates/tiler-ir/src/index/refinement.rs` asserts a derived subject identity `starts_with(SUBJECT_IDENTITY_TAG)` and **not** `starts_with(LEGACY_SUBJECT_IDENTITY_TAG)`, pinning both `tiler.ir.index-refinement-subject.v2` and its superseded `v1`. Two more assert a domain inside a longer golden: `schedule::numerics`'s key goldens open `tiler.contract.f32.v2.` and `tiler.contract.bf16.v1.`, and `STRICT_F32_REGION_IDENTITY_HEX` in `crates/tiler-ir/src/schedule/builder.rs` opens `74696c65722e7363686564756c652e763500`, which is `tiler.schedule.v5\0`. The corrected claim is the one in the title: *some* domains were unasserted, and nothing distinguished those from the rest.

**Fact — false as written, corrected.** The claim was that a domain "can be reverted … and the full gate stays green", stated generally. Two of the three domains named are caught today, indirectly, by digest goldens that fold them. Measured at `c0829b41`, each revert applied alone and `cargo nextest run --workspace --locked` run over 3,184 tests:

| perturbation | result |
| --- | --- |
| `GRAPH_DOMAIN` `v3` → `v2` | 2 failed — `tiler-build metal_plan::tests::the_standard_metal_path_publishes_its_recorded_identities`, `tiler-compiler explain::tests::deterministic_trace_is_sealed_and_rendered_separately` |
| `INDEX_REGION_DOMAIN` `v11` → `v10` | 4 failed — the two `tiler-ir index::law::tests` chain-identity pins, `tiler-artifact program::codec::tests::a_bf16_artifact_round_trips_and_its_carrier_enters_identity`, and the `tiler-build` row above |
| `OBLIGATION_DOMAIN` `v2` → `v1` | **0 failed, 3,184 passed** |

That coverage is incidental rather than designed: it exists where someone happened to pin a digest over a subject that folds the domain, it says "a digest moved" rather than "a domain moved", and it is absent wherever no golden folds the domain.

**Fact — verified.** The obligation-domain revert is invisible to the gate. The original measurement recorded 3,181 tests; the worker reproduced it at `c0829b41` at 3,184 tests run, 3,184 passed, 8 skipped.

**Fact — verified.** A length check would not have caught it. `crates/tiler-ir/src/semantic/precondition.rs` states the step at its declaration: `v2` writes a subject boundary through `SourcedShape::encode`, which tags each extent, so a rank-zero boundary — no extents — encodes to the same length under both, and `v1` and `v2` are the same width. Only comparing bytes shows the domain moved. A pin that compares lengths is not a pin.

**Fact — imprecise, corrected.** `grep -rn` for `tiler.semantic-graph.v3` across `crates/` returns **two** occurrences at `c0829b41`, not three: the declaration in `crates/tiler-ir/src/semantic/identity.rs` and **one** doc comment, which is in `crates/tiler-artifact/src/program/codec/tests.rs`. `tiler.index-region.v11` does return four — the declaration in `crates/tiler-ir/src/index/builder.rs` and three doc comments — but "three of its four mentions are elsewhere" is wrong about *where*: two of those three are in `crates/tiler-ir/src/index/law.rs`, inside the declaring crate, and only `crates/tiler-build/src/metal_plan.rs` is outside it.

## Why p1

Every other identity discipline in this repository rests on the domain being right: `derive_identity`, artifact publication, cache subjects, and the manifest schema all fold a domain string. A wrong domain silently makes two different subjects share an identity or one subject present as two. It is the highest-consequence, lowest-visibility value in the tree, and `AGENTS.md` already demands "name and count populations so 'nothing ran' cannot look green" — this population is named nowhere and counted nowhere.

## What closes this

An assertion over the domain strings that fails when one changes. **Prefer a census over a list**: a hand-written list of domains is the same defect one level up, and `AGENTS.md` says to size enumerations from the type rather than by hand. Something that enumerates the declared domain constants and asserts each expected value — the shape `crates/tiler-artifact/src/domains.rs` already uses for its governed set, which is the nearest precedent and worth reading first.

**Do not pin only the domains one module owns.** `tiler.index-region.v11` is declared in `crates/tiler-ir/src/index/builder.rs` and mentioned at three further sites, one of which — `crates/tiler-build/src/metal_plan.rs` — is outside this crate. Report which domains fall outside `implementation/ir` rather than reaching into other scopes.

**Perturb each domain separately and quote the failure text.** Revert one constant at a time and show what the assertion said; a perturbation that reddens everything cannot show which pin is load-bearing. Then confirm the reverse case is reachable — state what it would take for this check to say *no* when a domain is correct.

**A deliberate step must stay cheap.** The point is not to make version steps hard; it is to make them *visible*. If the assertion forces a worker to edit five places to step one domain, it will be worked around. Design it so a legitimate step is one edit and an accidental revert is a failure, and say in the report which of those two you optimized for.

## Worker record, at `c0829b41`

**What landed.** `crates/tiler-ir/src/domains.rs`, a `#[cfg(test)]` module declaring no public item, plus its declaration in `crates/tiler-ir/src/lib.rs`. Four tests: `every_tiler_spelled_literal_is_pinned_or_classified`, `every_pinned_identity_domain_still_appears_in_the_source`, `both_tables_are_sorted_and_free_of_duplicates`, `no_admitted_prefix_swallows_a_pinned_domain`. Run them with `cargo nextest run -p tiler-ir --locked -E 'test(/domains::/)'`; four tests run.

**The census is over source literals, not over a type, and that is a deliberate departure from the precedent.** `crates/tiler-artifact/src/domains.rs` sizes its set with `variant_count` because every domain that crate admits is a constant a variant can name. That does not hold in `tiler-ir`. Domains here are spelled three ways: named `_DOMAIN` constants, named constants that are not called `_DOMAIN` (`RECEIPT_IDENTITY_TAG`, `EXHAUSTIVE_DERIVATION`, `SUBJECT_IDENTITY_TAG`), and **inline literals no constant names** — `tiler.schedule.v5` is written directly into `schedule::model::encode_identity`, and `tiler.resolved-value-type.v3` appears at three sites in `semantic/types.rs`. A `variant_count`-sized enum cannot reach a literal no constant names, so it would have enumerated a strict subset while reporting a complete population, which is the failure it exists to prevent. The scan reads all 131 `.rs` files under `src/` and `tests/`, finds 185 `tiler.`-spelled literals in 120 distinct spellings, and requires each to be either pinned (60 rows) or admitted by a classified non-domain namespace (11 prefixes). `AGENTS.md`'s rule for a population that cannot be typed — "assert a floor and print the census" — is the one that applies.

**Perturbations, each applied alone, each with the assertion's own text.** Six independent properties, six separate perturbations of the subject:

| perturbation | which assertion said *no* | failure text |
| --- | --- | --- |
| `OBLIGATION_DOMAIN` `v2` → `v1` | presence, and pinning | ``` `tiler.semantic-precondition-obligation.v2\0` is pinned as an identity domain of this crate, and no literal in `src/` or `tests/` spells it.``` / ``` precondition.rs:26: the literal `tiler.semantic-precondition-obligation.v1\0` is neither pinned … nor admitted by a prefix``` |
| `GRAPH_DOMAIN` `v3` → `v2` | presence, and pinning | ``` `tiler.semantic-graph.v3\0` is pinned as an identity domain of this crate, and no literal in `src/` or `tests/` spells it.``` |
| `INDEX_REGION_DOMAIN` `v11` → `v10` | presence, and pinning | ``` `tiler.index-region.v11\0` is pinned as an identity domain of this crate, and no literal in `src/` or `tests/` spells it.``` |
| inline `b"tiler.schedule.v5\0"` → `v4` in `schedule/model.rs` | presence, and pinning | ``` `tiler.schedule.v5\0` is pinned … and no literal … spells it.``` — the case a type-sized enumeration cannot reach |
| `F32_NUMERICAL_CONTRACT_KEY_DOMAIN` `v2` → `v1` (a `&str`, not a byte literal) | presence, and pinning | ``` `tiler.contract.f32.v2` is pinned … and no literal … spells it.``` |
| `ADMITTED_NON_DOMAIN_PREFIXES` row `tiler.scalar.` widened to `tiler.scalar` | prefix shadowing **only** | ``` the admitted prefix `tiler.scalar` covers the pinned identity domain `tiler.scalar`, so that domain's exact spelling is no longer compared against anything.``` |
| `collect_rust_sources` made non-recursive | the walk floor **only** | ``` the walk found 14 `.rs` file(s) across `src/` and `tests/`, fewer than the 100 this crate has.``` |
| `domains.rs` renamed to `domain_pins.rs` | the self-exclusion guard **only** | ``` the walk did not find this module at …/src/domains.rs, so it removed nothing. The pin table in this file restates every spelling it pins, so a scan that read it would satisfy the presence assertion from the pin alone.``` |
| one pin row duplicated | the table-shape guard **only** | ``` PINNED_IDENTITY_DOMAINS is out of order or repeats itself at `tiler.schedule.v5\0` and `tiler.schedule.v5\0`.``` |

**What it takes for the check to say *no* when every domain is correct: nothing.** Both census assertions quantify over a difference that is empty on a correct tree — found minus pinned, and pinned minus found — so a correct tree walks both populations and reports no member. That case is reached: all four tests pass at `c0829b41` with the module added and no other edit. The reachable failures are exactly the nine rows above.

**Cost of a legitimate step, and which side this optimized for.** Optimized for **making an accidental revert loud**, and the reason is that the brief's "one edit" is not achievable for a byte pin: a pin compares a value against a second copy of that value, so a deliberate change has to move both copies or there is no pin. The floor is two edits, and this design sits on the floor — the constant, and one row in `PINNED_IDENTITY_DOMAINS`. Both assertions name the file, the spelling, and the table in their text, so the second edit is located rather than searched for. Two domains cost one row more: `tiler.contract.f32.v2` and `tiler.contract.bf16.v1` also appear as the opening of a key golden in `schedule::numerics`, so their rows in `ADMITTED_NON_DOMAIN_PREFIXES` carry the version too, deliberately — that makes each golden a second reading of the same pin rather than a namespace that would admit a reverted domain. The rejected cheaper design was a version *floor* per domain, which costs zero edits on a step but decays: it cannot see a revert to any version at or above the recorded floor, and a guard with a known leak is what this repository keeps finding.

**Domains outside `implementation/ir`, reported rather than reached.** The scan covers `crates/tiler-ir/{src,tests}` only. Counting distinct `tiler.`-spelled string and byte-string literals under each other crate's `src/` with `grep -rhoE '\bb?"tiler\.[^"]*"' | sort -u | wc -l` at `c0829b41`: `tiler-artifact` 28, `tiler-compiler` 25, `tiler-cache` 7, `tiler-build` 2, `tiler-conformance` 2, `tiler-digest` 2, `tiler-metal-aot` 2, `tiler-reference` 2, `tiler` 1, `tiler-macros` 1, `tiler-metal` 0, `tiler-runtime` 0.

`tiler-artifact` already carries `src/domains.rs`, but its guarantee is a different one: it establishes completeness against its own sources and the no-prefix property, and it never compares a domain against an expected *value*, so an artifact domain can still be reverted there with nothing to say so. The largest genuinely uncovered population is `tiler-compiler`, whose 25 include versioned subjects that step — `tiler.compiler.request-subject.v6`, `tiler.target-profile.declaration.v11`, `tiler.target-profile.descriptor.v10`, `tiler.compiler.boundary-property-set.v3`. `crates/tiler-build/src/metal_plan.rs` mentions `tiler.index-region.v11` only in prose, which no value pin reads in any case.

Each of those crates needs its own copy of this module, because the population is per-crate source and `CARGO_MANIFEST_DIR` is what makes the scan reach it — the same reason `tiler-artifact` enumerates its subject separately from `tiler-ir`'s `exhaustive_injectivity`. That is a separate ticket per crate, not scope this one should have taken.

## Outcome and graph completion — 2026-08-09

Commit `2191c839` landed the private IR census and commit `4da9ef36` closed this
ticket. The compiler remainder it filed subsequently landed and closed under
[`pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach`](pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach.md).
The distinct artifact exact-byte remainder named above had no graph node; it is
now owned by
[`pin-the-tiler-artifact-identity-domain-bytes-the-existing-census-does-not-fix`](pin-the-tiler-artifact-identity-domain-bytes-the-existing-census-does-not-fix.md).
This completed ticket remains scoped to the IR census and does not imply either
other crate's population was covered by it.
