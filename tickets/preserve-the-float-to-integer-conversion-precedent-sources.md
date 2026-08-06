---
id: preserve-the-float-to-integer-conversion-precedent-sources
title: Preserve the float-to-integer conversion precedent sources
status: in-progress
priority: p3
dependencies: []
related: [land-the-conversion-pair-decomposition-adr, test-the-directional-conversion-pair-generalization]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, conversion, sources, preservation]
claimed_from: todo
assignee: agent-precedent-sources
lease_expires_at: 1786032770
---
## User-visible outcome

Every primary source behind [ADR 0041](../docs/decisions/0041-separate-float-to-integer-conversion-families.md)'s four float-to-integer families is either cited by a preserved-source id or carries a flagged acquisition request stating what was tried and what it would decide — so re-deriving why the corpus separates strict rounded, exact, ordered saturating, and total saturating NaN-to-zero does not depend on seven live URLs staying where they are.

## The defect, stated so it can be reproduced or refuted in one line

**Fact.** [Float-to-integer conversion precedents](../docs/research/numerics/float-to-integer-conversion-precedents.md) is `disposition: adopted` and is the sole `evidence` record for ADRs 0010 and 0041. Its `Primary sources` list is seven bare URLs — LLVM `fptosi`, LLVM saturating conversions, WebAssembly numeric execution, the Rust reference's numeric casts, the C++ draft's `conv.fpint`, StableHLO `convert`, and the PTX conversion instructions — and it names no preserved-source id at all:

```sh
rg -n 'https?://' docs/research/numerics/float-to-integer-conversion-precedents.md
```

**Fact — three of the seven already have a pinned identity in the manifest that the record does not use.** `llvm-langref-llvmorg-22.1.8` is `LangRef.rst` at `llvm/llvm-project` commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`, and it carries both cited LLVM claims — `fptosi` appears 32 times and the section "Saturating floating-point to integer conversions" begins at line 21556, with `llvm.fptosi.sat` documented from line 21617. `stablehlo-spec-v1.18.0` carries the `convert` section. `nvidia-ptx-isa-cuda-13.3.0` is metadata-only with a recorded digest and a version-qualified archive URL. So the LLVM and StableHLO claims are re-derivable from bytes in this repository today and the record sends a reader to a live page instead.

**Inference.** This is the exact failure the [preservation record](../docs/research/numerics/sources/README.md) exists to prevent, and it states the lesson from a case where the moving citation also turned out to be wrong: "a citation to a moving path is not re-derivable, and here it was also not correct." The array API record is the worked precedent for repairing one — re-check every claim against the preserved bytes, record which held and which did not, and say so rather than asserting a clean pass.

## What the work is

- Re-check the two LLVM claims and the StableHLO claim against the preserved bytes named above and rewrite the record's source lines to name the ids, following the array-API and Linalg re-check precedent in the preservation record: state that the re-check happened, what it covered, and whether any claim failed it.
- Confirm the PTX claim against the recorded metadata-only identity, or flag it as an acquisition request if the archived page no longer serves the digested bytes. A digest mismatch on a rendered documentation page is evidence to investigate, not proof the specification changed — the record already says so.
- For WebAssembly, the Rust reference, and the C++ draft, attempt acquisition and follow the manifest's licence-aware discipline: vendor where the document's own terms permit dissemination, record bibliographic identity plus a retrieval fingerprint plus an official acquisition route where they do not, and flag as a named acquisition request anything unreachable, stating what was tried and what the source would decide.
- Update `expected-sources.tsv`, the declared population counts at the top of `verify-sources.sh`, and the record's own prose in the same change, as the preservation record's own instructions require.
- Run `docs/research/numerics/sources/verify-sources.sh` and **watch it fail** on a deliberate perturbation before trusting a pass, because the population counts are what make a lost row distinguishable from a manifest that agrees with itself.

## Explicit non-goals

- Reopening ADR 0010 or ADR 0041. Both are accepted; this is provenance hardening, and a claim that fails its re-check is a correction to the research record with the failure recorded, not a decision to revisit.
- Extending the population beyond this one record's citations. Other research records with live-URL citations are their own tickets.
- Acquiring IEEE 754-2019 or the two OCP specifications. Those are already classified metadata-only with reviewed licences and recorded acquisition routes, and none of the seven sources above is one of them.

## Closes when

Every one of the seven citations resolves to a manifest id, a metadata-only identity with a route, or a flagged acquisition request naming what was tried; the verifier passes on the updated population and was watched failing on a perturbation; and the record states which claims were re-checked against preserved bytes and which were not.
