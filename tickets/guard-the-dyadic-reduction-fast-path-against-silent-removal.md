---
id: guard-the-dyadic-reduction-fast-path-against-silent-removal
title: Guard the dyadic reduction fast path against silent removal
status: in-progress
priority: p3
dependencies: []
related: [bound-the-exact-rational-gcd-cost-in-certified-enclosures]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [performance, test-coverage]
claimed_from: todo
assignee: sol-dyadic-guard
lease_expires_at: 1786410519
---
## The gap, reported by the worker who created it

[`bound-the-exact-rational-gcd-cost-in-certified-enclosures`](bound-the-exact-rational-gcd-cost-in-certified-enclosures.md) removed **62.9 %** of the Stein iterations in one workload and **100 %** in another by reducing a dyadic exact rational with a shift instead of a gcd loop. The change is value-preserving by construction — same divisor, same integer pair — which is exactly what makes it invisible to every existing test.

**If someone deletes the fast path, every test still passes.** Only re-running the census notices, and the census instrumentation is deliberately not committed: it is a counter in a hot path that answered one question. So the cost improvement has **no in-tree regression guard**, and the ticket that landed it said so plainly rather than leaving it to be discovered.

## What was already rejected, so it is not re-proposed

The landing worker considered and rejected two guards, and the reasons stand:

- **A wide-operand test whose runtime blows up without the fast path** — either too weak to matter (tens of milliseconds, lost in noise) or an effective hang. There is no useful middle.
- **A timing assertion** — inadmissible on this host, which `AGENTS.md` reserves timing for the idle M3 Pro and which runs concurrent agent builds.

So this needs a *mechanism*, not another attempt at the same two shapes. That is why it is a separate ticket rather than a follow-up edit.

## What might work, none of it settled

- **Count rather than time.** The census counted Stein iterations by instrumenting the loop. A permanently committed counter — behind `#[cfg(test)]` or a feature — would let a test assert "this workload performs zero gcd loop iterations", which is a *counted* property that survives moving hosts, unlike a timing. The cost is a counter in a hot path; whether that is acceptable in the shipped path or only under `cfg` is the question.
- **Assert the branch is taken** rather than its cost: a test that reduces a known dyadic pair and observes, through some visible effect, that the shift path ran. Needs an observable the fast path has and the general one does not — which today it does not, since the answers are identical by design.
- **Accept the gap and record it** where a reader of the fast path will see it. Legitimate, and cheaper than a bad guard — but it should be a decision rather than a default.

## What this owes

A guard that fails when the fast path is removed, **or** a recorded decision that no admissible guard exists with the reason, stated at `reduction_divisor` where the next reader will find it. Whichever lands, the census's reproduction recipe stays cited from that site so re-measuring is one command rather than a rediscovery.

## Explicit non-goals

No change to the reduction's behaviour, no new fast path, and no revisiting of the two hypotheses that measurement rejected — the symmetric dyadic-magnitude case at 0.066 % of iterations and the word-sized-operand path at 0.51 %. Those were measured and declined, not deferred.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration, from a gap the landing worker identified in its own work and reported rather than leaving for a reviewer. It is p3 because the fast path is correct and landed; what is missing is protection against a later change silently undoing it.

## Outcome

Implemented 2026-08-10 without changing the reduction's value, public surface, or identity. The exact-base Fact audit at `fbf7f32ea8093e01a53c226f3c27cb9664f91813` verified every claim above against the completed parent ticket, the full rational implementation and accuracy tests, `num-bigint 0.4.8`'s `BigUint` gcd and trailing-zero implementations, the module construction and consumers, and the live ticket graph; no repair or scope expansion was required.

The guard is a test-only thread-local census around the real `reduction_divisor` subject. Every helper invocation increments an exact total, and the sole general-gcd mechanism increments a general count. Focused tests require a dyadic normalization, zero magnitude, and both widest admitted decode cases to perform zero general reductions; a separate non-dyadic case requires the general mechanism and its unchanged reduced value. Thread-local state prevents parallel unit tests from contributing to each other's counts, and every observation is behind `#[cfg(test)]`, so non-test builds carry no counter or branch.

The `num_integer::Integer` trait capability is deliberately scoped inside the test-only `observed_general_reduction_divisor` and the non-test general arm; unrelated floor and ceiling calls use fully qualified trait syntax. This ties the observation to the general mechanism rather than to a manually labelled dyadic arm without adding a wrapper call to production builds: routing the dyadic case through the named observed mechanism increments the count, while replacing the actual shift expression with a raw `magnitude.gcd(denominator)` does not silently bypass it — that subject perturbation failed compilation with:

```text
error[E0599]: no method named `gcd` found for reference `&BigUint` in the current scope
help: trait `Integer` which provides `gcd` is implemented but not in scope
```

Independently deleting only the total-call observation, while leaving the shift and assertions intact, made `a_dyadic_reduction_never_enters_the_general_gcd_path` fail with:

```text
left: ReductionPathCounts { total: 0, general: 0 }
right: ReductionPathCounts { total: 1, general: 0 }
```

The original operand-population census remains deliberately temporary. Its exact recipe and `cargo nextest run -p tiler-reference --test gcd_census_temp --no-capture` command are now cited at `reduction_divisor`, so the measured 62.9 % / 100 % shares remain reproducible without adding a production counter.
