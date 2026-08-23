---
id: calibrate-the-physical-provider-count-at-the-installation-seam
title: Calibrate the physical provider count at the installation seam
status: in-progress
priority: p2
dependencies: []
related: [calibrate-the-physical-frontier-provider-and-outcome-budgets, replace-provider-offer-with-a-host-bounded-frontier-sink]
scopes: [research/program-planning, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, measurement, public-boundary]
assignee: worker-provcount
lease_expires_at: 1787493486
---
## User-visible outcome

The physical provider-count limit has an accepted value enforced at the authority that can actually refuse it, instead of inheriting a number from a superseded single-target table that never passed a decision gate.

## Why this exists

Split out 2026-08-22 by the coordinator when `calibrate-the-physical-frontier-provider-and-outcome-budgets` **fired its own stop condition**: *"provider count belongs to a different preflight authority than the complete budget policy."* The worker stopped rather than pushing through, which was correct. That ticket now owns the raw-outcome axis only.

**Fact, repaired 2026-08-23 at `c0f70e64` — the two axes have different enforcement points, and only one has an accepted value.** Two halves of the original wording were false and are replaced here rather than restated in new words; the audit section below carries the reads.

The raw-outcome bound is an accepted *policy* value awaiting encoding, not a live field. `DeterministicBudgets` has fourteen fields and none of them is a raw-outcome or a provider-count bound, so the request-scoped counter remains preserved-draft-only on `54e272ba`. Neither axis has a live enforcement point in the compiler today; what separates them is that one has an accepted value and the other does not.

A provider-count bound is also **not** confined to `InstalledPhysicalProviders::installed`. `MAX_OFFERED_PHYSICAL_PROVIDERS` in `crates/tiler-artifact/src/program/mod.rs` bounds the offered physical-provider population at 4,096 and refuses through `CompilationEnvironment::new` naming `OfferedPhysicalProviders`, which `assemble_plan_artifact` reaches on the production packaging path with exactly this compilation's governed-plus-installed identities. What is true of the installation seam is narrower than the original claim: it runs **before** compile and branches on three causes — unrepresentable provenance, governed-identity collision, and duplicate identity — with no count branch at all.

**Fact, verified 2026-08-23 — the `32` is superseded, not accepted.** It comes from the single-target table that ticket retains as history rather than as a live recommendation. The 2026-08-18 acceptance names only the request-scoped raw-outcome value; its decision gate enumerates raw-outcome powers and no provider count. Verify this by reading that ticket's `## Accepted policy — 2026-08-18` section in full before relying on any number. The claim is if anything understated: no `32` anywhere in `crates/` is a provider count.

**Reported by the calibration lane, verified 2026-08-23:** `installed()` has no provider-count refusal as a type bound, and the durable finite witness installs 129 identities — but that 129 lives in the **spike harness**, not in `crates/`. The lane flagged its own production-impl census as textual rather than type-enumerated, closing the gap at that base but not by construction.

## Required work

- Re-audit every Fact above at your base and report a per-Fact verdict before proposing a value.
- Decide **by reading** whether a provider-count limit belongs at the installation seam at all, or whether the raw-outcome ceiling already bounds what matters. **A limit with no honest enforcement point should be eliminated, not relocated** — that conclusion is a valid and possibly correct outcome of this ticket.
- If a limit is warranted, derive it from a measured population rather than reusing `32`, and say what it refuses that the raw-outcome ceiling does not.
- Size the production-impl population **from the type where possible** rather than textually; state which unit you report and anchor every pattern.
- Whatever you conclude, perturb the subject and quote the failure text. If you add a refusal, show it firing on a population that should be refused and *not* firing on one that should not.

## Non-goals

Changing the accepted request-scoped raw-outcome value; merging or rebasing the preserved draft `54e272ba`, which its own ticket records as unmergeable; and any wall-clock claim taken on a loaded coordination host.

## Closes when

The provider-count axis has either an accepted limit enforced where it can actually refuse, or a recorded decision that it should not exist, with the reasoning and a reconsideration trigger in both cases.

## Per-Fact audit at exact base `c0f70e64a357d2934929655baa81fe5239fbf598`

Every verdict below rests on a full read of the named file at this base. Counts are stated in the unit named beside them; `grep -c` counts lines and is labelled where used.

- **False, repaired — "the raw-outcome bound is a per-request `DeterministicBudgets` field".** It is not a field at all. `crates/tiler-compiler/src/request/budget.rs`, anchor `pub(crate) struct DeterministicBudgets {`, declares fourteen fields, ending `physical_plan_combinations`; `governed()` initializes exactly fourteen. None names a raw outcome or a provider count. The accepted request-scoped value is policy awaiting encoding, and the counter itself is still preserved-draft-only. The sibling ticket already says this at anchor `still has no provider-count or raw-outcome field`; this ticket's summary of it drifted.
- **False, repaired — "a provider-count bound could only refuse at `InstalledPhysicalProviders::installed`".** `crates/tiler-artifact/src/program/mod.rs`, anchor `pub const MAX_OFFERED_PHYSICAL_PROVIDERS`, is a live physical-provider *count* bound of 4,096. `crates/tiler-artifact/src/program/builder.rs`, anchor `fn canonical_offered_role`, collects the caller's input and refuses past the bound with `ArtifactLimitKind::OfferedPhysicalProviders` **before** sorting and deduplication, so repeated identities spend capacity rather than amplifying it. `crates/tiler-build/src/plan_artifact.rs`, anchor `let environment = CompilationEnvironment::new`, feeds it `compilation.offered_physical_providers()` — precisely the governed-plus-installed list `offered_identities` mints — and `crates/tiler-build/src/metal_plan.rs` reaches `assemble_plan_artifact` on the production path. The word **only** was the false part. The coordinator's brief repeated this and its supporting grep missed the constant: the pattern `MAX_(PHYSICAL_)?PROVIDER` cannot match `MAX_OFFERED_PHYSICAL_PROVIDERS`, because the token between `MAX_` and `PHYSICAL` is `OFFERED_`. A widened search for any const whose name contains `PROVIDER` finds it immediately.
- **Imprecise, repaired — "branches only on governed identity and duplicates".** Three branches, not two. `crates/tiler-compiler/src/physical_provider.rs`, anchor `pub fn installed(`, refuses `UnrepresentableProvenance` before either identity check. The load-bearing half — no count branch — is **verified**: the function collects `providers.into_iter().collect()` and no length is read, compared, or stored.
- **Verified — the `32` is superseded, not accepted.** Read `## Superseded single-target outcome` (its table row is the only place a provider-count `32` is recommended), `## Decision gate` (every enumerated candidate is a raw-outcome power: 256, 512, 1,024, 2,048, 4,096, 8,192, 16,384 — no provider count), and `## Accepted policy — 2026-08-18` (names request-scoped 1,024 alone). The claim is understated rather than overstated: **no `32` in `crates/` is a provider count**. Seven lines in `crates/` contain a `\b32\b` alongside `provider`; six are `Specialization::Workgroup(32)` in `crates/tiler-compiler/tests/external_physical_provider.rs` and the seventh is an explain golden in `crates/tiler-compiler/src/explain.rs` spelling `width=32` inside a record that also spells `provider=`. Both are widths. The unrelated `region_candidates_per_seed: 32` in `budget.rs` is a region-search cap and is named here so a later reader does not mistake it for this axis.
- **Verified — `BudgetResource` has no provider-count variant.** Sized from the type, not by hand: `crates/tiler-compiler/src/request/budget.rs` already carries `pub(crate) const ALL: [Self; std::mem::variant_count::<Self>()]`, so the enumeration is a build error if the vocabulary widens. Fifteen variants, ending `ExplainDetailCanonicalBytes`; none counts providers.
- **Verified — the 129-identity witness is spike-only.** `spikes/program-planning/physical-frontier-budget-calibration/src/census.rs`, anchor `type-admits-129-installed-providers`, installs 129 `Answer::Empty` providers and asserts they install. Two lines in `spikes/**/*.rs` contain `129` beside `provider`; zero lines in `crates/**/*.rs` do.

**A census-vocabulary miss of my own, recorded because it is the failure mode this repository keeps hitting.** Sizing the budget struct with `grep -c '^    pub(crate) [a-z_]*: u32,'` returned **12** against a true **14**: `region_cover_expansions` and `physical_plan_combinations` are `u64`, and hard-coding `u32` in the anchor silently dropped them in the direction that reads as clean. The number above is taken from a parse of the struct body instead.

## Decision — 2026-08-23: the provider-count axis is eliminated at the installation seam

**One option dominates and it is taken rather than presented.** No provider-count limit is added at `InstalledPhysicalProviders::installed`, and none is added anywhere else by this ticket. The ticket's own instruction applies literally: a limit with no honest enforcement point is eliminated, not relocated.

**What a count at this seam would refuse that existing authorities do not: nothing that matters, and it cannot express the one thing that does.** Split the population three ways.

- *Providers that emit outcomes.* Already refused twice over. The accepted request-scoped raw-outcome ceiling charges every proposal and decline, and `crates/tiler-compiler/src/explain.rs`, anchor `let capacity = if terminal`, already refuses on canonical detail bytes at `1024 * 1024` today — which is how the sibling ticket's own 31-specialist run refused after 527 installed-provider outcomes. A flooding provider set is stopped before it costs anything unbounded, so the count axis buys nothing here.
- *Packaging identity.* Already refused at `MAX_OFFERED_PHYSICAL_PROVIDERS`, 4,096, counted against the caller's collected input on the production packaging path. A second count authority at installation would be a *disagreeing* one: two numbers answering "how many physical providers may one compilation have" is the conflation this repository treats as a defect, not a defence.
- *Providers that emit **zero** outcomes.* This is the real gap and it is worth naming precisely, because it is genuine leverage. `crates/tiler-compiler/src/frontier.rs`, anchor `pub(crate) fn enumerate_frontier(`, calls `provider.provenance()` and then `provider.propose(&context)` for **every** provider on **every** enumeration, before any outcome exists to charge. A provider that returns an empty offer is therefore invoked, clones an identity, and is charged nothing by any authority present or accepted.

**But the installation seam structurally cannot bound that gap, and this is the decisive argument.** The cost of the zero-outcome population is `providers × enumerations`, where enumerations is distinct region subjects × target profiles × numerical-contract candidates. `installed` sees the first factor and none of the rest: it is a constructor called before the request is assembled, its signature takes only an iterator of providers, and `with_physical_providers` stores its result before targets, contracts, or a program are attached. `MAX_TARGET_PROFILES_PER_REQUEST` is 16 and `MAX_NUMERICAL_CONTRACT_PREFERENCES` is 4, but the subject count is program-dependent — the governed five-operation program alone reaches seventeen distinct subjects per target through the `frontiers_by_subject` memo in `crates/tiler-compiler/src/pipeline/planning.rs`. Any number refused at installation therefore bounds one factor of a product whose other factors are invisible to it: the same count that is generous for a one-target trivial program is ruinous for a sixteen-target one, and no single number is correct for both. That is not an implementation detail to be worked around; it is the definition of a limit in the wrong place.

**The quantity that is enforceable is the invocation charge, and it is already owned.** Charging one unit per `propose` call is request-scoped, sees both factors, and subsumes the count axis exactly: it refuses 129 empty providers on a wide request and admits them on a trivial one. [`replace-provider-offer-with-a-host-bounded-frontier-sink`](replace-provider-offer-with-a-host-bounded-frontier-sink.md) already carries this obligation at anchor `Bound zero-outcome provider invocations independently from emitted outcomes`, and that ticket depends on this one. **This decision therefore narrows its successor rather than deferring to it:** the provider-count axis has no residue at the installation seam, and the whole of the concern that motivated it is that single bullet. The conclusion does not depend on how the sink is designed — push, pull, or otherwise — only on what `installed` can see, which its signature fixes.

**Two further reasons the seam is wrong, either sufficient alone.** Adding a refusal to `installed` would widen `PhysicalProviderInstallationError`, a public enum on a surface Tom accepted on 2026-08-11 under ADR 0075 — a public-boundary change, and so not this ticket's to make, spent on a bound that cannot do its job. And a count field in `DeterministicBudgets` is likewise excluded: `budget.rs` states at anchor `Every field is written into the canonical request subject by` that a value change there moves every governed compilation's request and evidence subject, so encoding a provider count there would move request and evidence identity for every governed compilation in exchange for bounding the wrong quantity.

**Identity consequence: none.** No production code changed, no constant moved, no enum widened, no budget field added or altered. The delta is one `#[cfg(test)]` test.

**Strongest counterargument, and why it does not survive.** An unbounded installed-provider vector is a host-memory quantity the caller controls, and Tiler's other host-bounded quantities are budgeted. It fails on the facts: the vector stores one borrowed pointer and one cloned identity per provider the caller has *already* allocated, so it is strictly smaller than what the caller built to call it, and the packaging path already refuses the population at 4,096. What is genuinely unbounded is invocation work, which is the successor axis above, not the vector.

**One out-of-scope observation, reported rather than fixed.** The duplicate check in `installed` is `identities.contains(&identity)` inside the loop, so installation is quadratic in the installed count. At any population the packaging bound admits this is negligible, and the honest remedy is an ordered set rather than a count refusal — it changes no behaviour, no public surface, and no identity. It is not a defect this ticket found reason to fix and is left for a separate ticket if anyone wants it.

## Negative controls

The recorded decision is guarded by `installation_admits_any_count_and_names_no_count_refusal` in `crates/tiler-compiler/src/physical_provider.rs`. It guards two independent properties and each was perturbed separately, with the assertions unchanged and the subject broken.

- **Count branch relocated to the seam.** Inserting `assert!(installed.len() <= 32, ...)` after the collect in `installed` fails the witness half: `installed provider count exceeds 32`, panicking at `physical_provider.rs:225:9`. The vocabulary assertion runs first and stays green, so the two do not mask each other.
- **Refusal vocabulary widened.** Adding a `TooManyProviders { offered: usize }` variant to `PhysicalProviderInstallationError`, with its `Display` and `source` arms, fails the other half instead: `assertion left == right failed: the installation vocabulary is exactly unrepresentable provenance, duplicate identity, and governed identity; a fourth cause must be justified against the recorded decision that installation does not count`, `left: 4`, `right: 3`.

Both perturbations reach their subject, and the check can therefore say *no* in both directions a future reader might move it. Restored to green after each.

## Reconsideration trigger

Reopen this decision when **any** of the following fires. Each is a source condition with a command, not a judgement call.

1. **`installed` gains access to the request.** If the installation seam ever receives the target profiles, numerical-contract candidates, or the program — that is, if the signature at anchor `pub fn installed(` takes anything beyond an iterator of providers — the structural argument above dissolves and a calibrated count becomes expressible there. Check: read that signature.
2. **A zero-outcome invocation charge is declined by its owner.** If [`replace-provider-offer-with-a-host-bounded-frontier-sink`](replace-provider-offer-with-a-host-bounded-frontier-sink.md) lands without honouring `Bound zero-outcome provider invocations independently from emitted outcomes`, the gap this decision hands it is unowned and this axis must be reconsidered as the fallback. Check: `grep -n 'zero-outcome provider invocations' tickets/replace-provider-offer-with-a-host-bounded-frontier-sink.md` and read the landed sink for a per-invocation charge.
3. **The offered-population authority is removed or made optional.** If `MAX_OFFERED_PHYSICAL_PROVIDERS` stops bounding the production packaging path — deleted, raised without evidence, or bypassed by a compile route that never assembles an artifact yet retains the offered set — then the packaging half of the argument lapses. Check: `grep -rn 'MAX_OFFERED_PHYSICAL_PROVIDERS' crates/` and confirm `assemble_plan_artifact` still constructs `CompilationEnvironment`.
4. **A named consumer states an installed-provider population.** The 2026-08-18 acceptance records that an accepted policy *separating installed identities from active answerers* reopens the raw-outcome value. Such a policy would also, for the first time, give this axis a population to calibrate against. Check: the same trigger command that ticket's `## Trigger check log` already runs.

## Trigger check log

- **2026-08-23 — not fired.** All four conditions checked at `c0f70e64`. (1) `pub fn installed(` takes `impl IntoIterator<Item = &'providers dyn PhysicalImplementationProvider>` and nothing else. (2) The sink ticket is `todo` and still carries its zero-outcome bullet; `grep -c 'zero-outcome provider invocations' tickets/replace-provider-offer-with-a-host-bounded-frontier-sink.md` returns 1 line. (3) `MAX_OFFERED_PHYSICAL_PROVIDERS` is 4,096 and `assemble_plan_artifact` still builds `CompilationEnvironment` from the compilation's offered physical providers. (4) No consumer or accepted contract names an installed-provider population; the sibling ticket's own 2026-08-18 log entry is the most recent evaluation and found none.
