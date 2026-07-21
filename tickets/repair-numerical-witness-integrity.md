---
id: repair-numerical-witness-integrity
title: Repair numerical witness integrity
status: todo
priority: p0
dependencies: []
related: []
scopes: [research/numerics, contracts/numerics, research/cost-model, research/reference, research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [research, correctness, numerics]
---

Repair numerical experiment programs and evidence claims found unsound by the
fixed-point audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd`.

## Required outcome

- Replace removable Python `assert` verdicts with explicit checked failures in
  the cost-model, reference, region-search, reduction, and region-accuracy
  witness programs. Optimized Python must not print a passing verdict after
  deleting the checks.
- Make the sound-accuracy observer executable in the governed locked Python
  environment; do not rely on unavailable `math.fma`. Establish the exact FMA
  oracle used.
- Apply the stated 100-digit Decimal context to every claimed oracle operation,
  not only divide/sqrt, or correct the evidence label and rerun measurements.
- Execute an actual relational-ratio sample corpus before publishing an
  empirical maximum; a literal zero is not a witness.
- Align the reduction probe and README on the exact 24-element permutation
  universe and add a genuine empty-contributor witness. A singleton `-0.0`
  case is not an empty reduction.
- Make the region-accuracy dependency part of the locked environment or record
  an explicit unavailable measurement; preserve bounded execution and result
  provenance.
- Make Daisy/FPTaylor wrappers enforce wall-clock bounds, parse and validate the
  required proof/result fields, and treat diagnostic or incomplete output as
  `Unknown` or failure even when the external analyzer exits zero.

## Acceptance

Run every witness with ordinary Python and `python -O`; both must produce the
same verdict. Regenerate retained numerical results with exact interpreter,
dependency, precision, sample-domain, and algorithm provenance. Documentation
must distinguish normative reference, exhaustive finite evidence, empirical
samples, and unavailable observations.
