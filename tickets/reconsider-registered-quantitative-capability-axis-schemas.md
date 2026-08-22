---
id: reconsider-registered-quantitative-capability-axis-schemas
title: Reconsider registered quantitative capability-axis schemas
status: deferred
priority: p3
dependencies: []
related: [own-or-close-the-adr-internal-open-questions, prototype-a-bounded-scalar-cpu-backend-vertical, declare-cpu-vector-realization-facts-in-the-target-profile, construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [research/target-profiles, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, target-profiles, extensions, deferred]
---
## User-visible outcome

If real backend evidence outgrows the compiler-owned quantitative-axis
vocabulary, Tiler re-evaluates a governed extension schema from concrete rows
rather than either blocking ecosystem work indefinitely or stabilizing an
extension protocol speculatively.

## Accepted starting point

Tom decided on 2026-08-03 in the T3 Code orchestration conversation that
quantitative capability axes remain compiler-owned and exhaustive for the
initial profile. [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
records the decision and its grounds. A required quantitative fact is therefore
added as a reviewed typed compiler variant with a host-owned comparison.

The alternative preserved here is not an arbitrary provider-defined key or an
opaque comparison callback; those fail correctness and deterministic identity
and remain eliminated. The only surviving alternative was a frozen per-request
registry binding a governed axis key to a host-validated quantity, relation,
validation rule, and canonical identity encoding, with the host retaining every
comparison.

## Activation triggers

Deferred rather than dispatchable. Reopen this question only when either
evidence threshold is met:

1. An independently authored target profile is blocked by a genuinely
   quantitative target fact that the compiler does not name; or
2. A second backend demonstrates a quantitative row whose validation,
   comparison, and identity schema is materially shared with a CPU row.

A request for vector width, mask/tail behaviour, or scalable-vector realization
does not fire either trigger: ADR 0093 derives those as one exact atomic
realization subject rather than an ordered quantity.

## Work after activation

- Preserve the concrete rows and their producer/consumer evidence before
  proposing an abstraction.
- Compare adding typed compiler variants against a registered host-validated
  schema on correctness, deterministic identity, total-map maintenance,
  out-of-tree authorship, and review cost.
- Specify registry freezing, duplicate/conflict refusal, canonical encoding,
  unknown-schema rejection, and which crate owns each public type before
  drafting implementation.
- Return every consequential public registration, trait, type, or call-site
  boundary to Tom under ADR 0075; this ticket records no advance acceptance.

## Dependency policy

This deferred reconsideration is not a prerequisite for existing target-profile
or backend work. Current work adds measured quantitative rows through the
accepted compiler-owned vocabulary. If an activation trigger fires, the blocked
consumer may depend on this ticket then; no speculative dependency is added now.

## Closes when

After a trigger fires, the concrete multi-backend evidence eliminates all but
one ownership model, the result is recorded in a durable contract or accepted
ADR, every identity and validation consequence is enumerated, and any public
boundary has Tom's explicit acceptance.

## Trigger check log

- 2026-08-04 — **not fired.** Trigger 1 is unmet: no independently authored target profile exists at all, let alone one blocked by an unnamed quantitative fact. Trigger 2 is unmet: [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) is `todo`, so there is no second backend's quantitative row to compare — and the ticket already records that a vector-width request would not fire either trigger under ADR 0093.
- 2026-08-09 — **not fired.** Caller-installed physical providers now exist, but they are not target-profile authors and do not satisfy trigger 1. `declare-cpu-vector-realization-facts-in-the-target-profile` remains `todo`, so trigger 2 still has no second backend row whose validation/comparison/identity schema can be compared. The compiler-owned quantitative table has grown within the accepted typed vocabulary; growth alone is not evidence that the schema must become registrable.
- 2026-08-10 — **not fired.** Board status correction: [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) is `awaiting-decision`, not `todo` (the 2026-08-09 log line misstated that status; ADR 0093's 2026-08-09 board correction already records the schedule-vocabulary and target-fact tickets as `awaiting-decision`). Trigger 1 remains unmet: no independently authored target profile is blocked by an unnamed quantitative fact; caller-installed physical providers are not target-profile authors. Trigger 2 remains unmet: the vector realization subject is non-quantitative under ADR 0093, and no second backend has demonstrated a *new* quantitative row whose validation, comparison, and identity schema is materially shared with a CPU row in the sense that would reopen registered schemas. Growth within the typed compiler-owned quantitative vocabulary alone does not fire either trigger.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. The entry's board reading is greppable and **its recorded value is now false**: it says [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) is `awaiting-decision`, while the command returns `status: blocked`. Recorded, not acted on. The axis half is `rg -n -e '^[[:space:]]+[A-Z][A-Za-z]*,' $(rg -l 'enum CapabilityAxis' crates/)`, which reports the seven governed axes; an eighth quantitative axis is the changed answer. Note that the axis enum lives in `crates/tiler-compiler/src/target/feasibility.rs` and **not** in `target.rs`, which is the stale path a sibling deferral was still grepping. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Reason repaired — 2026-08-22; verdict unchanged, neither trigger fired.** The 2026-08-04 entry's ground for trigger 1 is **dead at this base**, and the retired wording is quoted rather than removed so the count of it cannot shrink: it reads `no independently authored target profile exists at all, let alone one blocked by an unnamed quantitative fact`. One now exists. `crates/tiler-conformance/tests/independent_backend/nodefold.rs` declares the profile `tiler.test.nodefold-host-v1` through `pub(crate) fn nodefold_profile`, and its module documentation states the authorship — anchor `stated by a producer that lives outside every crate in` the workspace, compiling against public surfaces alone. It is an integration-test binary of `crates/tiler-conformance`, **not** a `#[cfg(test)]` module, so it exercises the public boundary a third-party profile author would. Landed 2026-08-22 by `9e55e763` and renamed by `cc15cbdc`. The 2026-08-09 and 2026-08-10 entries' narrower ground — that caller-installed physical providers are not target-profile authors — was correct and is now beside the point, because a target-profile author has arrived.

  **Trigger 1 is unmet on its second clause, which no earlier entry had to reach.** The clause is *blocked by a genuinely quantitative target fact that the compiler does not name*, and this profile is blocked by nothing. `fn nodefold_profile` declares one fact per governed axis — seven calls, `declare_max_threads_per_grid_axis` through `declare_local_memory_bytes` — and returns `builder.build()`. `rg -c quantitative crates/tiler-conformance/` reports no file at all. The compiler-owned vocabulary was sufficient for an outside author on its first attempt, which is evidence *against* reopening rather than for it. **Trigger 2 is unmet:** [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) is `blocked` — the entry above already carries that correction against its own `awaiting-decision` — so no second backend quantitative row exists to compare, and ADR 0093 keeps the vector subject non-quantitative.

  **The axis command supplied above is broken in both directions and is replaced.** `rg -n -e '^[[:space:]]+[A-Z][A-Za-z]*,' $(rg -l 'enum CapabilityAxis' crates/)` does not report the seven governed axes. Run at this base it returns **58 lines**: it scans the whole of `crates/tiler-compiler/src/target/feasibility.rs` and matches every indented capitalised word ending in a comma anywhere in it — unrelated enum variants, and `PRESERVE` / `FORBIDDEN` / `None` arguments deep inside the test region. It simultaneously **drops one of the seven it claims to report**, because `[A-Za-z]*` cannot cross the digits in `IndexArithmeticU64`. A census that both over- and under-counts cannot report a firing, and the over-count hides the under-count.

  **Sized from the type instead.** `CapabilityAxis` is `pub(crate)`, and the typed census already exists beside it: anchor `CANONICAL_AXES_COVER_THE_CAPABILITY_VOCABULARY` in `crates/tiler-compiler/src/target/feasibility.rs` is a `const` assertion that `CANONICAL_AXES.len() == core::mem::variant_count::<CapabilityAxis>()`, so a widened vocabulary is a build error at the enumeration rather than a population that silently shrinks. The readable roll-call is `rg -n 'pub\(crate\) const CANONICAL_AXES' -A 10 crates/tiler-compiler/src/target/feasibility.rs`, which prints the array header `[CapabilityAxis; 7]` and all seven names: `GridAxisThreads`, `WorkgroupThreads`, `BufferBindings`, `DeviceAddressSpace`, `LocalMemoryBytes`, `IndexArithmeticU64`, `DeviceAddressWidthBits`. An eighth quantitative axis is still the changed answer — and the array header carries the count, so the answer changes even if the reader miscounts the names.
