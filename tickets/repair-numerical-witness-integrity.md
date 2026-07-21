---
id: repair-numerical-witness-integrity
title: Repair numerical witness integrity
status: done
priority: p0
dependencies: []
related: []
scopes: [research/numerics, contracts/numerics, research/cost-model, research/reference, research/region-search, implementation/workspace, contracts/navigation]
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
- Make the existing Daisy wrapper enforce wall-clock bounds, parse and validate
  the required proof/result fields, and treat diagnostic or incomplete output
  as `Unknown` even when the external analyzer exits zero. Do not fabricate a
  nominal FPTaylor wrapper: the repository has no such executable yet, and the
  deferred `spike-hermetic-fptaylor-certificate-checking` ticket owns that
  separate certificate experiment.

## Acceptance

Run every witness with ordinary Python and `python -O`; both must produce the
same verdict. Regenerate retained numerical results with exact interpreter,
dependency, precision, sample-domain, and algorithm provenance. Documentation
must distinguish normative reference, exhaustive finite evidence, empirical
samples, and unavailable observations.

## Outcome

- All six Python witness programs use explicit verdict checks and produce
  byte-identical output under ordinary and optimized Python. The bounded
  `spikes/numerics/check_witnesses.py` entrypoint also rejects executable
  `assert` syntax structurally, then enforces mode parity collectively.
- The explicit-FMA observer now uses exact rational arithmetic plus a locally
  checked IEEE binary32 ties-to-even rounding oracle. Every Decimal reference
  runs at 100 digits, and the equality-constrained ratio is an executed
  five-sample corpus with a retained maximizing witness. Its checked-in result
  records the exact interpreter/host, source and algorithm identity, numerical
  policy, complete finite domains, counts, witnesses, and outputs.
- The reduction probe exactly enumerates the documented 24 permutations and
  includes genuine empty-contributor results.
- `mpmath==1.3.0` is part of the locked development environment and is checked
  by bootstrap/workspace policy.
- Daisy execution is a bounded, strictly parsed adapter with stable `Unknown`
  results, process-group timeout coverage, resource ceilings, and fingerprints
  for the launcher, classpath contents, selected Java executable, and inputs in
  addition to in-run Git verification of the source revision. Every profile
  requires identical pre/post fingerprints; the documented spike limitation
  reserves immutable staged execution for production proof ingestion. The
  retained checkout could not rerun Daisy because its old launcher referenced
  missing source-resource and deleted temporary dependency-cache entries; this
  produced `Unknown` and is recorded as an unavailable fresh proof, not
  silently promoted to success.
- The ordinary pytest gate includes the Daisy integration suite and the
  aggregate ordinary/optimized witness-and-retained-fixture check. Successful,
  zero-exit diagnostic, nonzero, incomplete-result, timeout, parser,
  provenance, and resource-limit paths are covered.

Validation included the full Rust gate, documentation/Python gates, sixteen
Daisy adapter tests, ShellCheck, ordinary/optimized witness parity, ticket lint,
and diff checks on 2026-07-21.
