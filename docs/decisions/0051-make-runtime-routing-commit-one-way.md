# 0051: Make runtime routing commit one-way before program work

**Status:** accepted

## Context

Fallback is safe only while the consumer still owns an unexecuted semantic
operation. Pipeline preparation may fail before device work, but allocation,
partial encoding, submission, validation, and publication can have observable
resource or execution effects. Retrying an ordinary fallback after those stages
can duplicate work, hide device errors, or publish inconsistent results.

## Decision

Runtime preflight binds inputs and the live device, evaluates guards, prepares
every entry of one complete variant, validates launch/resource capabilities,
and then consumes `FallbackAuthority` at `RoutingCommit`.

Only the resulting committed execution authority may allocate program resources
or encode work. After commit, every allocation, encoding, submission,
completion, validation, and publication failure is a typed terminal execution
error. The launcher cannot recover fallback or silently route another variant.

A synchronous validation record is interpreted only after the exact submission
that produced and synchronized it reaches terminal success. A post-wait error
propagates and never becomes a validation miss or fallback condition.

## Consequences

- Typed applicability/capability misses may try another complete equivalent
  route only during precommit preparation.
- Corrupt artifacts, ABI inconsistencies, systemic preparation failures, stale
  prepared selections, and dishonest providers fail closed.
- Program allocations and partial encodings never precede a fallback decision.
- Runtime adapters must retain all resources through their exact final device
  use and expose trustworthy completion/error observation.
- Consumer integrations unable to preserve this ownership boundary do not
  implement the initial runtime profile.

## Alternatives considered

Fallback after an arbitrary runtime error is ergonomically tempting but cannot
distinguish no-work failures from partial effects. A Boolean `can_fallback`
flag is weaker than a consumed ownership token. Preloading only the library is
insufficient because function lookup and pipeline creation are separately
fallible.
