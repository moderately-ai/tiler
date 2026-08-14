# Tom decision queue

Operational queue for the continuous-delivery coordinator. Ticket files remain the authority; this file records presentation order, holds, exact release triggers, and the current recommendation so a later cycle does not rediscover or prematurely present a packet.

Updated 2026-08-13 at main `c9da757e` plus the pending staged-evidence repair delta.

## 1. Atomic subgroup realization surface — held for exact-surface repair

- Tickets: `accept-the-atomic-subgroup-realization-surface` and `minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance` (`p1`).
- Hold evidence: the packet omits public methods/types/traits; `SubgroupTransfer::from_tag` has no production consumer; `SubgroupRealizationError::UndefinedTransfer` is unreachable from every public constructor; the error violates ADR 0074's non-exhaustive convention; and no test proves a present subgroup reaches kernel identity.
- Current recommendation: remove the speculative decoder and unreachable error, privatize the raw tag, retain the consumed key/subject encoder, conform the error, add the missing identity evidence, then rewrite the packet exactly. A separate artifact ticket now blocks subgroup schedule derivation until `Some` can round-trip.
- Release trigger: the repair lands under independent exact-commit review and the acceptance packet is rebuilt through the complete option/readiness gate. Present it first only after that repair.

## 2. Live-extent artifact envelope row — held

- Ticket: `accept-the-live-extent-artifact-envelope-row` (`p1`, `awaiting-decision`).
- Hold evidence: the draft row is currently attached to a fixed `[2,3]` semantic interface while tests execute bindings 14/15. It is unresolved whether `{ key, axis, value_type }` remains a complete row once the symbolic semantic source is carried.
- Current recommendation: do not accept the row yet.
- Release trigger: `associate-live-extent-operands-with-symbolic-semantic-interface-axes` produces an independently reviewed minimum complete schema/identity derivation and the packet is rewritten against that exact commit.

## 3. Host-bounded physical-frontier sink — held before presentation

- Tickets: `replace-provider-offer-with-a-host-bounded-frontier-sink`; branch-local `accept-the-host-bounded-physical-frontier-sink` at preserved draft `54e272baa525027a6f6f9d982bd3bd7c387597fb`.
- Hold evidence: request-wide limit 256 was calibrated on one target; the admitted request holds up to 16. Review also found request exhaustion downgraded to target/candidate outcomes, provider-order-dependent error precedence, a `u64::MAX` counter escape, and stale authority-count documentation.
- Current recommendation: accept a host-owned bounded emission surface in principle, but do not accept this exact packet/value yet.
- Release trigger: `calibrate-the-physical-frontier-provider-and-outcome-budgets` supplies a full-request authority/value; the preserved branch is rebased and all four review findings receive subject perturbations; independent exact-commit review passes; packet is updated on main.

## 4. Materialized producer in a serial-reduction contributor — held for carrier comparison

- Ticket: `admit-a-materialized-producer-in-a-serial-reduction-contributor` (`p3`, `todo`).
- Hold evidence: option 7 can enlarge every unboxed serial-sum value without forcing broad `NormalizedOutput` matches to classify the new state. A boxed produced-sum variant sharing a fold core may preserve old layout and improve exhaustiveness; a narrower bare-producer slice also trades support for smaller state. `pipeline/verify.rs` contains an uncensused `prologue.is_none()` numerical-proof exemption that would include a materialized arm unless repaired. The staged-family positive also stops first at missing governed elementary authority.
- Current recommendation: do not present a carrier yet. Preserve the one-edge sides rule, prototype/measure the bare, boxed-top-level, and boxed-rare-payload survivors, audit every broad serial-sum/output consumer, and repair the staged evidence with a caller-declared RMS row.
- Release trigger: exact layout/host-memory evidence, complete consumer/refusal/identity census including numerical verification, `drive-staged-materialization-boundary-tests-past-elementary-accuracy` closed, independent review, and a repeated Pareto gate.
