---
id: re-audit-adr-implementation-status-after-the-runtime-and-metal-landings
title: Re-audit ADR implementation status after the runtime and Metal landings
status: in-progress
priority: p2
dependencies: []
related: [close-remaining-adr-status-drift, re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening, raise-the-adopted-research-records-to-their-landed-implementation-status]
scopes: [contracts/decisions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, status-drift, graph-repair]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785611640
---
## User-visible outcome

No accepted ADR reads `not-started` beside implemented, gate-covered production code, and no exclusion from the 2026-07 status audit rests on a premise the tree has since falsified.

## Why this exists

**Fact — ADR 0051 reads `not-started` and its decision is implemented.** `docs/decisions/0051-make-runtime-routing-commit-one-way.md:9` reads `implementation_status: "not-started"`. The one-way commit is production code: `Preflight` at `crates/tiler-runtime/src/load/route.rs:579`, declared neither `Clone` nor `Copy` with the comment "A route that could be duplicated could be committed twice, and 'committed once' is the property ADR 0051 asks for", and the infallible `Preflight::commit` at `:740`. The module header at `:7` states the invariant directly, and compile-fail doc-tests at `:671` and `:711` pin double-commit and mint-after-commit.

**Fact — the exclusion's stated premise is falsified.** [`close-remaining-adr-status-drift`](close-remaining-adr-status-drift.md):26 excluded ADR 0051 with the reason "a checked one-way routing policy exists in the compiler-produced plan at `crates/tiler-compiler/src/program.rs:511`, but every clause of the decision is a runtime-launcher behaviour and **no runtime exists**". `crates/tiler-runtime` exists. Its research record, `docs/research/runtime/runtime-execution-contract.md`, still reads `disposition: "adopted"` with `implementation_status: "spike-only"`.

**Fact — ADR 0086 reads `not-started` and calls a `done` ticket blocked.** `docs/decisions/0086-require-attributable-or-attested-native-translation.md:9` reads `not-started`, and `:57` reads "`construct-and-bind-the-first-authoritative-metal-compile-profile` remains blocked." That ticket is `done`, and the refusal the record decides is production code: `MetalHostPredicate::NativeTranslationAuthority` at `crates/tiler-metal/src/applicability.rs:397`, the `MetalHostEligibility` receipt holding the uninhabited authority at `:695-703`, the `UnknownNativeTranslationAuthority` refusal at `:800` reporting the rule `metal.host-applicability.unknown-translation-authority` at `:840`, and `native_translation_authority()` at `:1061` which the comment at `:1056` states can never return a value because the type is uninhabited.

**Fact — five further exclusions were never re-examined, and the audit says so.** `close-remaining-adr-status-drift.md:31` records that after `re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening` superseded two of the eight exclusions, "the other six exclusions in that bullet and the `undetermined` ADR 0034 finding are untouched and were not re-examined." Excluding ADR 0051 (this ticket's headline), those five are **ADR 0012**, **ADR 0014**, **ADR 0020**, **ADR 0022**, and **ADR 0025**, plus the ADR 0034 `undetermined` finding at `:27`.

## Work

Read each record in full against the current tree and move `implementation_status` where the evidence supports it, applying the rule [`close-remaining-adr-status-drift`](close-remaining-adr-status-drift.md) established and [`re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening`](re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening.md) applied: a bump requires the decision's central mechanism to exist in `crates/`, not a type-system reservation or an architectural seam. Give each bumped record an `Implementation boundary` section naming the realized and unrealized clauses, as that precedent did. Correct ADR 0086:57's blocked claim in the same change, and raise `docs/research/runtime/runtime-execution-contract.md`'s `implementation_status` if the ADR bump supports it.

## Boundaries

- The field is a **retained high-water mark**, not a live mirror of the working tree (`docs/document-metadata.md:63`). Never lower one.
- Distinguish the four maturity claims AGENTS.md separates — a type-system reservation, an architectural seam, implemented support, and a tested guarantee. An exclusion re-examined and confirmed is a real outcome; record the confirmation with its current evidence rather than leaving the stale premise standing.
- Scope is `contracts/decisions` plus `research/runtime` for the one research record. The six adopted research records that read `spike-only` against `partial` ADRs are [`raise-the-adopted-research-records-to-their-landed-implementation-status`](raise-the-adopted-research-records-to-their-landed-implementation-status.md)'s; do not absorb them.
- `docs/decisions/README.md` is generated and `implementation_status` is not a catalog input, per `close-remaining-adr-status-drift.md:29`; confirm rather than assume.

## Closes when

ADR 0051 and ADR 0086 carry statuses their production code supports; ADR 0086:57's blocked claim is corrected; each of the five untouched exclusions and the ADR 0034 finding is either bumped or re-confirmed against current evidence with the stale premise replaced; and no accepted ADR is left reading `not-started` beside gate-covered code.
