# Tom decision queue

Operational queue for the continuous-delivery coordinator. Ticket files remain the authority; this file records presentation order, holds, exact release triggers, and the current recommendation so a later cycle does not rediscover or prematurely present a packet.

Updated 2026-08-14 at published main `67fc9cac`.

## 1. Truthful public class for complete-explain capacity refusal — ready, presented, awaiting Tom

- Ticket: `decide-the-truthful-public-class-for-complete-explain-capacity-refusals` (`p1`, `in-progress`).
- Packet evidence: exact implementation packet `272d7e5ca7f8dbf2996e84e5f85c9e73785d88d4`, independently reviewed and merged at `7b4b991f`. A valid seven-specialist request reaches the one-MiB explain-detail byte guard but currently surfaces as `InvalidCompilerOutput`, contradicting that public class's defect-only contract.
- Current recommendation: accept the sole nondominated surface: reuse `CompileFailureClass::BudgetExhausted { resource, limit, reported }`; add report-only `BudgetResource::{ExplainDetailRecords, ExplainDetailCanonicalBytes}` and closed `BudgetRefusal::ConstructionLowerBound`; keep both limits outside `DeterministicBudgets`; preserve a distinct outer/request-wide internal capacity carrier so no candidate, contract, or target fallback and no partial output is possible.
- Strongest counterpoint: the explain limits are build constants rather than request-budget fields. The accepted public contract already says `BudgetExhausted` reports a deterministic bound "this build" declares, so a dedicated top-level class duplicates the same payload and caller action.
- Release trigger: Tom accepts or rejects that exact included/excluded surface. On acceptance, create the named implementation and evidence tickets and add the four downstream hard edges before this decision closes. On rejection, leave the ticket and both downstream paths blocked and record the requested revision.

## 2. Atomic subgroup realization surface — ready behind item 1

- Tickets: `accept-the-atomic-subgroup-realization-surface` and `minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance` (`p1`).
- Packet evidence: the exact-surface repair and independent source review landed. A fresh read-only audit at `67fc9cac` found no source drift and one ticket-only identity imprecision, now repaired: both descriptors encode phase, authority, and validity, while only the complete declaration encodes the structured source and the checked descriptor deliberately excludes source identity. Focused IR, compiler, external-API, and UI populations remain green.
- Current recommendation: do not present concurrently with item 1. Present this exact packet next; its minimized surface remains the sole nondominated public spelling and no unresolved construction, identity, error, schema, or artifact-boundary prerequisite remains.
- Release trigger: item 1 leaves the active Tom decision slot, then Tom accepts or revises the atomic packet's exact included and excluded surface.

## 3. Live-extent artifact envelope row — held

- Ticket: `accept-the-live-extent-artifact-envelope-row` (`p1`, `awaiting-decision`).
- Hold evidence: the draft row is currently attached to a fixed `[2,3]` semantic interface while tests execute bindings 14/15. It is unresolved whether `{ key, axis, value_type }` remains a complete row once the symbolic semantic source is carried.
- Current recommendation: do not accept the row yet.
- Release trigger: `associate-live-extent-operands-with-symbolic-semantic-interface-axes` produces an independently reviewed minimum complete schema/identity derivation and the packet is rewritten against that exact commit.

## 4. Host-bounded physical-frontier sink — held before presentation

- Tickets: `replace-provider-offer-with-a-host-bounded-frontier-sink`; branch-local `accept-the-host-bounded-physical-frontier-sink` at preserved draft `54e272baa525027a6f6f9d982bd3bd7c387597fb`.
- Hold evidence: the custodial idle-M3 request-wide census eliminated 256 and eliminated 16,384 as a standalone answer because complete explain capacity fires first. The raw value remains held on item 1 and on an explicit active-provider support policy. Review of preserved draft `54e272ba` also found request exhaustion downgraded to target/candidate outcomes, provider-order-dependent error precedence, a `u64::MAX` counter escape, stale authority-count documentation, uncompilable retained spikes, and incomplete calibration over admitted targets/candidates.
- Current recommendation: accept a host-owned bounded emission surface in principle, but do not accept this exact packet/value yet.
- Release trigger: item 1's accepted implementation/evidence lands; `calibrate-the-physical-frontier-provider-and-outcome-budgets` selects a full-request authority/value from an accepted support population; preserved branch `54e272ba` is returned for repair/rebase; every review finding and retained spike migration receives a subject perturbation; independent exact-commit review passes; packet is updated on main.

## 5. Materialized producer in a serial-reduction contributor — held for carrier comparison

- Ticket: `admit-a-materialized-producer-in-a-serial-reduction-contributor` (`p3`, `todo`).
- Hold evidence: option 7 can enlarge every unboxed serial-sum value without forcing broad `NormalizedOutput` matches to classify the new state. A boxed produced-sum variant sharing a fold core may preserve old layout and improve exhaustiveness; a narrower bare-producer slice also trades support for smaller state. `pipeline/verify.rs` contains an uncensused `prologue.is_none()` numerical-proof exemption that would include a materialized arm unless repaired. The staged-family positive also stops first at missing governed elementary authority.
- Current recommendation: do not present a carrier yet. Preserve the one-edge sides rule, prototype/measure the bare, boxed-top-level, and boxed-rare-payload survivors, audit every broad serial-sum/output consumer, and repair the staged evidence with a caller-declared RMS row.
- Release trigger: exact layout/host-memory evidence, complete consumer/refusal/identity census including numerical verification, `drive-staged-materialization-boundary-tests-past-elementary-accuracy` closed, independent review, and a repeated Pareto gate.
