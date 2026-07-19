# 0010: Make conversion behavior a typed semantic contract

**Status:** accepted

## Context

A source dtype and destination dtype do not uniquely determine a numeric
conversion. Observable choices can include rounding, overflow, NaN behavior,
signed zero, subnormal handling, and out-of-range float-to-integer behavior.
Those choices can differ between frontends, compilers, and hardware.

Fusion may remove the store and reload that happened to realize a conversion
in an unfused program. Tiler therefore cannot define conversion behavior by a
physical materialization boundary or an ambient backend default.

## Decision

Every semantic numeric conversion carries a resolved, typed conversion
contract. Conversion families such as floating-point narrowing,
floating-point-to-integer, integer narrowing, and quantization define only the
fields relevant to their semantics. Numeric conversion and bit
reinterpretation are distinct operations.

Frontends may expose named presets for ergonomics. Before canonical semantic
admission, those presets are expanded to stable, versioned contracts. Later
compiler phases do not reinterpret them using ambient frontend, compiler, or
device defaults.

A backend classifies the contract as exactly supported, exactly emulatable,
supported only under an already declared relaxation, or unsupported. It cannot
silently substitute a different conversion.

## Consequences

- Reference evaluation, fallback, fusion, and generated kernels share one
  conversion meaning.
- Removing materialization cannot remove observable rounding or exceptional
  value behavior.
- Conversion extensions add specialized contract families rather than fields
  to a universal optional-attribute bag.
- Exact portability may require emulation or rejection on some targets.
- Preset names and their expansion are versioned inputs to explanation and
  canonical identity.

## Alternatives considered

Encoding only source and destination dtype is concise but leaves observable
behavior implicit. A graph-wide conversion policy reduces repetition but
cannot accurately describe programs containing conversions with different
requirements. A universal structure containing every possible conversion
field makes invalid combinations representable and weakens diagnostics.
