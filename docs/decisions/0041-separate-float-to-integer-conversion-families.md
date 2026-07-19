# 0041: Separate float-to-integer conversion families

**Status:** accepted

## Context

Float-to-integer conversion has several independent observable choices:
rounding, finite overflow, infinities, NaN, exactness, and subnormal input
handling. Existing systems disagree or leave exceptional inputs undefined.
Although endpoint saturation is mathematically determined for ordered values,
it cannot determine a NaN result because NaN is unordered.

ADRs 0009 and 0010 require these choices to resolve before optimization. ADRs
0021 and 0033 require invalid runtime values to be proven absent or explicitly
validated. A backend poison value, host-language undefined behavior, or native
instruction convention cannot fill a missing semantic contract.

## Decision

Float-to-integer conversion is a specialized typed conversion family. Its
canonical contract identifies source and destination dtypes, the applicable
input-subnormal contract, and one valid family with only its meaningful fields.
Rounded and saturating families carry an explicit rounding rule; exact
conversion does not. This is a discriminated semantic contract, not a universal
bag of independently optional cast fields.

The recognized families are:

- **strict rounded conversion:** apply the named rounding rule, then require
  that the source is not NaN and the mathematical rounded integer is
  representable in the destination;
- **exact conversion:** require a finite, integral source whose mathematical
  value is representable, with no value-changing rounding;
- **ordered saturating conversion:** apply the named rounding rule, clamp
  finite overflow and infinities to the destination endpoints, and reject NaN;
  and
- **total saturating NaN-to-zero conversion:** the same ordered saturation plus
  an explicit mapping of every NaN to integer zero, for contracts such as Rust
  `as`, LLVM saturation, and WebAssembly saturating truncation.

For families that round, recognized deterministic rules include toward zero,
toward negative infinity, toward positive infinity, nearest ties to even, and
nearest ties away. Recognition does not imply that every
family/rounding/dtype tuple is enabled in an initial product profile or backend.
Stochastic rounding requires explicit randomness/state semantics and is not
smuggled into this pure deterministic family.

Every rejecting condition is a semantic precondition: strict conversion rejects
NaN and an unrepresentable rounded result; exact conversion rejects nonfinite,
fractional, and unrepresentable inputs; ordered saturation rejects NaN. Static
violations fail verification; residual dynamic obligations use ADR 0033
enforcement and withhold result publication until success. Failure is not a
plan miss and never authorizes NaN-to-zero or another mapping.

Range validity is evaluated after applying the rounding rule to the
mathematical source value. It is not implemented semantically by comparing
against integer endpoints rounded into the source float dtype. Both signs of
floating zero produce integer zero. Source subnormal treatment occurs according
to the resolved conversion contract before rounding or exactness/integrality
validation.

There is no ambiguous canonical `Cast(src, dst)` and no implicit default inside
semantic IR. Frontend or ergonomic presets expand to one complete versioned
family. Undefined/TBD source semantics may enter a portable exact profile only
with a proven or enforced valid domain; Tiler does not invent results for them.

Future explicit NaN replacements, `(value, validity)` results, fill-value
families, or other rounding contracts can be added through the operation-
extension path without changing existing identities.

## Consequences

- Strict conversion follows Tiler's fail-fast correctness posture.
- The word “saturating” never silently implies an arbitrary NaN mapping.
- Rust/WebAssembly-compatible NaN-to-zero remains directly representable but
  is visibly different from ordered saturation and strict conversion.
- Reference evaluation, guards, rewrites, artifacts, and backends agree on
  boundary values and exceptional inputs.
- Native target casts require conformance or fixups for the selected complete
  contract.

## Alternatives considered

Making NaN-to-zero the meaning of every saturating conversion is deterministic
but conflates an explicit ecosystem totalization with mathematical endpoint
saturation. Rejecting all totalized mappings would prevent faithful import of
well-defined source contracts. Inheriting backend behavior recreates CPU/GPU
and width-dependent divergence. One structure containing every cast concern as
optional fields makes invalid combinations representable and contradicts ADR
0010.
