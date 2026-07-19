# 0011: Resolve numerical permissions per operation

**Status:** accepted

## Context

A single graph-wide `exact` or `fast` mode is simple, but numerical freedoms
are not uniformly relevant or desirable. A program may permit contraction for
a multiply-add while forbidding reduction reassociation and approximate
transcendentals. Treating `fast` as one switch can silently relax unrelated
operations.

Fully independent per-operation policy is precise but needs an outer authority
that limits what frontend defaults and optimizer passes may enable.

## Decision

The program numerical policy is a ceiling: the maximum relaxation authorized
anywhere in the program. Each operation carries resolved effective permissions
for its applicable numerical dimensions. Effective permissions combine the
program ceiling, any tighter per-operation restriction, and the operation's
declared capabilities; they can never exceed the ceiling.

Named user-facing modes may initialize the ceiling, and frontends may expose
region or operation overrides. Before semantic optimization, all such controls
resolve to the same canonical per-operation representation. Later passes do
not consult ambient modes or frontend state.

Every semantic rewrite and physical alternative declares which effective
permission it consumes. Backend compiler flags are derived from the resolved
program and cannot grant additional freedoms.

## Consequences

- One program can safely optimize numerically different regions under different
  effective permissions.
- Enabling contraction does not implicitly permit reassociation,
  approximations, or exceptional-value assumptions elsewhere.
- Canonical identity includes both the policy ceiling and resolved
  per-operation permissions.
- Explain output can identify the exact permission that admitted or rejected an
  alternative.
- Frontend APIs may offer global, regional, or local controls without changing
  compiler-core semantics.

## Alternatives considered

A graph-wide exact/fast enum is compact but over-broad. Per-operation policy
without a graph ceiling allows local defaults to exceed the user's overall
authorization. Deferring permission resolution until backend lowering makes
logical rewrite legality depend on the selected target.
