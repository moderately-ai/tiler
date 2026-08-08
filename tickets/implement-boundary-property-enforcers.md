---
id: implement-boundary-property-enforcers
title: Implement executable boundary-property enforcers
status: deferred
priority: p1
dependencies: [implement-boundary-property-model, transfer-synchronization-and-resource-lifetime-contract, drive-an-external-physical-implementation-provider-through-compilation]
related: [device-placement-and-memory-domain-contract, implement-general-dag-partitioning]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, transfers, lifetimes]
---
Make physically compatible region implementations composable by inserting
explicit, value-preserving materialization, layout conversion, encoding
repacking, placement transfer, synchronization, and storage-handoff steps.
Verify ownership, ordering, resource lifetime, failure boundary, feasibility,
and cost. A boundary enforcer may change storage, addressing, placement, or
delivery, but never semantic dtype or tensor value.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.


## Deferred — the bounded profile admits no mismatch to enforce (2026-07-27)

**Finding.** An enforcer reconciles a producer guarantee with a consumer requirement that it does not discharge. The bounded profile contains no such pair, so there is nothing for one to reconcile.

Every boundary contract in the compiler is built at exactly two sites, `frontier.rs:557` and `frontier.rs:563`, from two **compile-time constants** — `bounded_guarantees()` and `bounded_requirements()`. There is no per-region variation: every region requires the same eight values and guarantees the same eight, and each guarantee discharges its requirement. `boundary::unsatisfied_properties` therefore cannot return a non-empty result anywhere on the production path, and `BoundaryDisagreement::UndischargedHandoff` — the rejection at `selection.rs:1472` this ticket would replace with an inserted step — is unreachable.

**Why it is not merely unreachable but unwritable.** Checking each of the eight dimensions for a mismatch the *vocabulary* could express, every one is closed off by a value the model marks `Reserved`:

| Dimension | Enforcer named by the model | Why no mismatch arises |
| --- | --- | --- |
| `StorageLayout` | layout conversion | the only guarantee is `DenseRowMajor`; the only requirement raised is `DenseRowMajor` |
| `StorageEncoding` | encoding repacking | `BitPacked` reserved — strict `f32` produces no packed value |
| `Alignment` | materialization into an aligned allocation | both sides are `F32_NATURAL`; 4 divides 4 |
| `Materialization` | materialization | `AliasView`/`OpaqueRuntimeValue` reserved — the frontier rejects both proposal bodies |
| `ExecutionAffinity` | placement transfer | one symbolic affinity, so equality always holds |
| `MemoryDomain` | placement transfer | four of five classes reserved; only `Device` is allocated and admitted |
| `Availability` | synchronization | `AfterObservedHostCompletion` reserved — ADR 0033 makes host observation a separate boundary |
| `Visibility` | coherence action | `RequiresExplicitCoherenceAction` reserved, and its own doc says it exists in order *not* to be satisfiable |

Writing the six enforcer kinds now would produce six subsystems no input can reach, exercised only by property sets a test invented for them. That is not a bounded first slice; it is untestable code with an unfalsifiable correctness argument.

**A second constraint, already recorded in the model and easy to lose in a rewrite.** `UnsatisfiedReason` (`boundary.rs:1453`) distinguishes `NotSatisfied` from `NotGuaranteed` precisely because *only the first is enforceable*: "a producer that guarantees the wrong value on a dimension may be reconciled by an enforcer that supplies the right one, while a producer silent on the dimension has made no claim an enforcer can start from." An enforcer must never discharge `NotGuaranteed`. Half the failure surface is out of scope by construction.

**The trigger, and it is checked rather than written down.** `frontier.rs::the_bounded_profile_admits_no_undischarged_boundary` asserts the two constant sets discharge each other on every canonical dimension. **When it fails, this ticket becomes startable, and the mismatch that failed it is the enforcer's first real case** — the test prints it. Its failure path was verified by perturbing the required alignment to 16 bytes and observing it report `Alignment, required 16, guaranteed 4, NotSatisfied`, so it is a check that can say no rather than one that has only ever been seen to pass. A companion test drives the relation with a genuinely unsatisfiable requirement, so an `unsatisfied_properties` that silently stopped reporting would fail too.

**What would fire it.** Any region that varies a boundary property from the profile constants: a vectorized reduction requiring `UnitStrideOnAxis` on a non-last axis (the one well-formed mismatch the current vocabulary admits against `DenseRowMajor`), a widened dtype vocabulary reaching `BitPacked` or a non-`F32_NATURAL` alignment, a second execution affinity, or a memory domain that owes a flush. Each arrives with its own ticket, and each brings the enforcer it needs — which is the right granularity for this work rather than six at once.

Do not repair that test by widening the sets back into agreement.

## Assessment restatement (2026-07-28): the trigger is necessary but no longer sufficient

The deferral above rests on the claim that every boundary contract is built at exactly two sites from two compile-time constants. **That claim is now false.** Since the opaque-call integration there are four construction sites: the two constants (`frontier.rs:566` and `:572`) plus `frontier::derive_call_boundary_contract`, which builds contracts from provider-supplied declarations through `call_declaration::required_properties_for` and `guaranteed_properties_for` — and the latter can emit `MaterializationForm::AliasView` for a call declaring `MayAliasInputs`.

**The named trigger test cannot fire from the new path.** `the_bounded_profile_admits_no_undischarged_boundary` compares only the two constants, which the opaque derivation never touches. A companion test now pins the other half: `an_opaque_declaration_can_produce_a_guarantee_the_bounded_profile_refuses` proves the new path genuinely produces a guarantee (`AliasView`) that the bounded requirements refuse — the exact mismatch class an enforcer would reconcile.

**Why this ticket stays deferred anyway, restated precisely.** The mismatch is *producible* but not *reachable*: no compile-path provider proposes an opaque call (`pipeline/planning.rs` hardcodes the one governed provider, and the registry it passes is empty), so no selection ever composes an `AliasView` guarantee against a `MaterializedBuffer` requirement. The startable condition is therefore no longer "the constant test fails" — it is **"a compile-path provider proposes an opaque call whose contract the composing consumer refuses"**, which arrives with caller-supplied physical providers. Both tests together are the tripwire: the constant test catches profile drift, the companion documents that the declaration path can already produce the mismatch, and the first refused handoff in a real compile is the enforcer's first case.

Also corrected: the integrate ticket's twice-made prediction that the constant trigger would fire during that work was wrong in both directions — the test could not fire from that path, and the variant it named (`OpaqueRuntimeValue`) is still unconstructed while the one that did become constructible is `AliasView`.

## Trigger re-evaluation after downstream opaque selection evidence (2026-07-28)

The mismatch is now reachable through the real selection authority with a test-level provider. `selection::tests::an_opaque_alias_view_is_refused_by_a_materialized_consumer` admits an opaque pointwise producer declaring `MayAliasInputs`, composes it against the scheduled reduction's `MaterializedBuffer` requirement, and observes `BoundaryDisagreement::UndischargedHandoff` naming `BoundaryProperty::Materialization`. Replacing the declaration with `Aliasing::Distinct` admits the plan and makes the test fail, so the fixture distinguishes the mismatch rather than passing on an empty frontier.

The ticket remains deferred after re-evaluation. The production compile path still has no caller-supplied physical provider or opaque-call registry, so no user compilation can reach this mismatch and no executable enforcer can yet be selected. The startable condition remains the first compile-path provider that produces such a refused handoff; the new test proves the selection and property layers are ready to identify that first case exactly.

## The restart condition is a graph edge now, 2026-08-02

**Why this changed.** Three re-evaluations above state the startable condition in prose and none of them put it in the graph, so the ticket that would fire it and the ticket waiting on it were connected by nothing a scheduler can read. Meanwhile [`implement-general-dag-partitioning`](implement-general-dag-partitioning.md) depended on *this* ticket — a `deferred` state that satisfies no dependent — which made a p1 permanently unreachable and stranded two open questions it owns (`docs/open-questions.md` Q-PLAN-002 and Q-PLAN-005). [`re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed`](re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed.md) owns the repair; this is its outcome on this side.

**Fact — the edges now match the prose.** This ticket depends on [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md), which is the "caller-supplied physical providers" the restatement above names. `implement-general-dag-partitioning` no longer depends on this ticket, because nothing it must deliver consumes an enforcer and its own Graph maintenance says it *supplies* the enforcer's first case; it is `related` in both directions instead.

**Fact — still unreachable on the production path, checked rather than carried forward.** `grep -rn 'PhysicalAuthorities' crates/` finds `PhysicalAuthorities::composed` only in `crates/tiler-compiler/src/pipeline/tests.rs`; the sole production construction is `PhysicalAuthorities::governed()` at `crates/tiler-compiler/src/pipeline.rs:591`. The deferral above stands on its own terms.

**Open question this ticket must settle before it starts, and it is not settled here.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):125 states that "out-of-crate opaque-call registration stays compiler-owned and crate-private per ADR 0078's correction" and that "no caller of any kind registers one on the compile path". The wiring gap it names as owning that, `register-opaque-calls-on-the-compile-path`, is `done` — and `composed` still has no production caller. **So whether a caller-supplied provider can produce the refused handoff at all, while registration stays crate-private, is an assumption the restart condition rests on rather than a fact it has.** If the provider work lands and the handoff is still unreachable, the restart condition is wrong rather than unmet, and it must be restated rather than waited on. Do not treat the new dependency edge as evidence that the condition is satisfiable.

## Citation drift corrected 2026-08-04, and one correction elsewhere was itself wrong

**The two-site claim above was superseded on 2026-07-28 and its line numbers have drifted twice since. Nothing about the deferral changes; only the citations do.** Read at base `c4b4bdb9`:

- `bounded_requirements` and `bounded_guarantees` are **live non-test functions in `crates/tiler-compiler/src/frontier.rs`**, at `:885` and `:911`, each taking a `carrier: StorageScalar` argument rather than being nullary compile-time constants. There are now **four** construction sites in that file, not two: `:668` and `:674` in the single-region contract derivation, and `:741` and `:760` in the chained-subprogram one — plus `frontier::derive_call_boundary_contract`, which the 2026-07-28 restatement below already added. So `frontier.rs:557`/`:563` (2026-07-27) and `:566`/`:572` (2026-07-28) are both stale.
- `UnsatisfiedReason` is at `crates/tiler-compiler/src/boundary.rs:1705`, not `:1453`.
- `BoundaryDisagreement::UndischargedHandoff` is at `crates/tiler-compiler/src/selection.rs:680`, not `:1472`.
- `the_bounded_profile_admits_no_undischarged_boundary` is at `crates/tiler-compiler/src/frontier.rs:3681`, and its companion `an_opaque_declaration_can_produce_a_guarantee_the_bounded_profile_refuses` at `:4579`. [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md) cites the first as `frontier.rs:2107`, which is stale; it is corrected there too.

Reproduce with `grep -n 'fn bounded_requirements\|fn bounded_guarantees\|bounded_guarantees(carrier)\|bounded_requirements(carrier)\|fn the_bounded_profile_admits_no_undischarged_boundary' crates/tiler-compiler/src/frontier.rs`, `grep -n 'enum UnsatisfiedReason' crates/tiler-compiler/src/boundary.rs`, and `grep -n 'UndischargedHandoff {' crates/tiler-compiler/src/selection.rs`.

**One published correction of these citations is wrong and is refuted here rather than inherited.** [`finish-the-stale-claim-sweep-over-the-candidate-status-ticket-bodies`](finish-the-stale-claim-sweep-over-the-candidate-status-ticket-bodies.md) states that `bounded_guarantees`/`bounded_requirements` "now live only inside `crates/tiler-compiler/src/boundary.rs`'s `mod tests` (`:1995`, `:2010`, `:2025`)". The `boundary.rs` names are real — that file's `#[cfg(test)] mod tests` opens at `:1995` and declares nullary test helpers of those names at `:2010` and `:2025` — but **"only" is false**: the production `frontier.rs` functions above are the ones the deferral's argument is about, and they are the carrier-parameterized pair, not the test helpers. Two functions sharing a name are not the same function, and repointing this ticket at the test module would have moved the argument onto a fixture. Reproduce the refutation in one line: `grep -rn 'fn bounded_guarantees\|fn bounded_requirements' crates/` prints four declarations across two files.

**The deferral itself is unaffected.** Both frontier functions are still derivations over one `carrier` with no per-region variation in the bounded profile, `unsatisfied_properties` still cannot return a non-empty result on the production path, and the restart condition remains the 2026-08-02 one below. Only a reader following the numbers was being misled.

## Trigger check log

- 2026-08-04 — **not fired.** The restart condition re-pointed on 2026-08-02 is the first compile-path provider producing a refused handoff, and its dependency [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) is `todo`. The production construction is still `PhysicalAuthorities::governed()` alone; `PhysicalAuthorities::composed` remains test-only. Recheck: `grep -rn 'PhysicalAuthorities::composed' crates/` finds only `crates/tiler-compiler/src/pipeline/tests.rs`.
- 2026-08-04 — **not fired**, re-confirmed by the stale-claim sweep on the same day at base `c4b4bdb9`, and the citations the deferral rests on were repaired in the section above. `drive-an-external-physical-implementation-provider-through-compilation` is still `todo`, so the restart condition is unmet for the same reason. Recheck: `grep -m1 '^status:' tickets/drive-an-external-physical-implementation-provider-through-compilation.md`.
- 2026-08-08 — **not fired, and the two entries above have gone stale in the direction that reads backwards.** Checked at base `1438c867` from `tkt/implement-parallel-reduction-strategies`. The dependency is now `done`, and both entries above offer `grep -rn 'PhysicalAuthorities::composed' crates/` as the recheck on the premise that a hit outside `pipeline/tests.rs` means the seam reached production. It has: the sole production construction is now `PhysicalAuthorities::composed` in `crates/tiler-compiler/src/pipeline.rs`'s `compile_with_physical_providers`, and `PhysicalAuthorities::governed()` appears **only** in tests — the exact inverse of what the 2026-08-04 entry records. **A reader running that command today reads the answer backwards, so it is retired as a recheck.**

  **The handoff is nevertheless still unreachable, and the reason moved.** It is no longer the provider seam; it is opaque-call registration. That production entry states the consequence in its own doc — `may propose a body and never a call` — and the registry it passes is **constructed inline in the argument list and moved straight into the call**, so no statement exists that could register into it. The `AliasView` guarantee that would produce the mismatch is derived from a *call declaration* through `frontier::derive_call_boundary_contract`, so a provider that cannot register a call cannot produce one.

  Recheck, replacing the retired command: `grep -Fn 'OpaqueCallRegistry::new()' crates/tiler-compiler/src/pipeline.rs` returns the single production construction, and reading the four lines around it shows it inline inside `compile_configured`'s argument list. **A `grep` for `.register(` is deliberately not offered here**: it was tried and it is useless — the workspace has rewrite, capability, and semantic registries with the same method name, and a `grep -v tests` filter cannot exclude an inline `#[cfg(test)] mod tests`. The structural reading is the check; the grep only locates it.

  **Both halves are already recorded in an accepted ADR at this base, so this is a reading of the graph rather than a new finding.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)'s own 2026-08-08 correction retires its row 4 claim that no out-of-crate caller can name the provider trait — it is `pub` and re-exported now — and moves row 5's production line to `PhysicalAuthorities::composed(providers, …)` inside `compile_with_physical_providers`, while stating that what row 5 records is unchanged. Anchor: `stays compiler-owned and crate-private per ADR 0078's correction`.

  **This fires the condition the 2026-08-02 section names for itself**, quoted from it: *"If the provider work lands and the handoff is still unreachable, the restart condition is wrong rather than unmet, and it must be restated rather than waited on."* The provider work has landed. The restart condition should name **out-of-crate opaque-call registration** rather than caller-supplied providers, and ADR 0090's finding that registration stays compiler-owned is what would then have to move first. Restating it is a decision about this ticket's own premise and is left to its owner rather than made from a passing branch. Also recorded, with the compile-path reading it came from, in [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md).

  **And the prediction that ticket carried is settled: it did not fire.** `the_bounded_profile_admits_no_undischarged_boundary` passes at this base — the whole `tiler-compiler` suite is green — and the multi-pass split that was expected to fire it landed without doing so, for the reason its own review packet gave: the test compares two compile-time constants a cross-stage boundary never touches. Recheck: `cargo nextest run -p tiler-compiler -E 'test(the_bounded_profile_admits_no_undischarged_boundary)'`.
