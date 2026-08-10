---
id: preserve-the-float-to-integer-conversion-precedent-sources
title: Preserve the float-to-integer conversion precedent sources
status: done
priority: p3
dependencies: []
related: [land-the-conversion-pair-decomposition-adr, test-the-directional-conversion-pair-generalization, preserve-the-pytorch-conversion-platform-variation-source]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, conversion, sources, preservation]
---
## User-visible outcome

Every primary source behind [ADR 0041](../docs/decisions/0041-separate-float-to-integer-conversion-families.md)'s four float-to-integer families is either cited by a preserved-source id or carries a flagged acquisition request stating what was tried and what it would decide — so re-deriving why the corpus separates strict rounded, exact, ordered saturating, and total saturating NaN-to-zero does not depend on seven live URLs staying where they are.

## The defect, stated so it can be reproduced or refuted in one line

**Fact.** [Float-to-integer conversion precedents](../docs/research/numerics/float-to-integer-conversion-precedents.md) is `disposition: adopted` and is the sole `evidence` record for ADR 0041 and co-evidence (with `tiler.research.numerics.dtype-resolution-precedents`) for ADR 0010. Its `Primary sources` list is seven bare URLs — LLVM `fptosi`, LLVM saturating conversions, WebAssembly numeric execution, the Rust reference's numeric casts, the C++ draft's `conv.fpint`, StableHLO `convert`, and the PTX conversion instructions — and it names no preserved-source id at all:

```sh
rg -n 'https?://' docs/research/numerics/float-to-integer-conversion-precedents.md
```

**Fact — three of the seven already have a pinned identity in the manifest that the record does not use.** `llvm-langref-llvmorg-22.1.8` is `LangRef.rst` at `llvm/llvm-project` commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`, and it carries both cited LLVM claims — `fptosi` appears 32 times and the section "Saturating floating-point to integer conversions" begins at line 21556, with `llvm.fptosi.sat` documented from line 21617. `stablehlo-spec-v1.18.0` carries the `convert` section. `nvidia-ptx-isa-cuda-13.3.0` is metadata-only with a recorded digest and a version-qualified archive URL. So the LLVM and StableHLO claims are re-derivable from bytes in this repository today and the record sends a reader to a live page instead.

**Inference.** This is the exact failure the [preservation record](../docs/research/numerics/sources/README.md) exists to prevent, and the lesson it exists to enforce — stated in the [mature-operation taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md) after the array-API re-check, where the moving citation also turned out to be wrong — is: "a citation to a moving path is not re-derivable, and here it was also not correct." The array API record is the worked precedent for repairing one — re-check every claim against the preserved bytes, record which held and which did not, and say so rather than asserting a clean pass.

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

## Outcome, 2026-08-06

**All seven citations now resolve to a manifest id, and every re-check held. No acquisition request was filed, because no source was unreachable** — stated explicitly because an empty flag list and an unattempted acquisition look the same from outside.

**Three needed no bytes, only naming, and were re-checked against the preserved copies rather than assumed from the id's existence.** `llvm-langref-llvmorg-22.1.8` carries both LLVM claims: `fptosi`/`fptoui` "converts its floating-point operand into the nearest (rounding towards zero) … integer value. If the value cannot fit in `ty2`, the result is a poison value", with the sharper form the record's own phrasing matches at the saturating section's opening — "return a poison value if the rounded-towards-zero value is not representable by the result type" — and both `llvm.fptoui.sat` and `llvm.fptosi.sat` opening their rule lists with "If the argument is any NaN, zero is returned" before the two clamping clauses. `stablehlo-spec-v1.18.0` carries the StableHLO claim: "If the truncated value cannot be represented in the destination type, the behavior is TBD". `nvidia-ptx-isa-cuda-13.3.0` carries the PTX claim in three parts — four integer rounding modifiers, "the result is clamped to the destination range by default; i.e, `.sat` is redundant", and the width-dependent NaN rule ("Zero if source is not `.f64` and destination is not `.s64`, `.u64`. Otherwise `1 << (BitWidth(dst) - 1)` …").

**The PTX archive URL was re-retrieved twice and reproduced its recorded digest exactly** — 3 577 575 bytes, SHA-256 `40026f79…`, identical to the 2026-07-31 record — so that fingerprint is now known reproducible across six days, which the identity caveat had deliberately not assumed. The bytes were read and discarded; no copy is checked in.

**Two precisions the re-check surfaced, neither changing a conclusion and neither touching ADR 0010 or 0041.** StableHLO and C++ each *do* specify the rounding — both truncate toward zero — and defer or leave undefined only the unrepresentable case; and five of the six systems fix the rounding identically, with PTX the sole exception. The record's prose now says so, because "leaves it TBD" read as covering the whole conversion would over-read the citation. Nothing was softened or corrected against the sources.

**Four new documents were acquired; three vendored, one metadata-only.** WebAssembly `wg-3.0` commit `9d36019973201a19f9c9ebb0f10828b2fe2374aa`: `numerics.rst` (the `trunc`/`trunc_sat` operators — `trunc_sat_u(±NaN) = 0` and `trunc_sat_s(±NaN) = 0` are the claim's second half) and `instructions.rst`, preserved for one sentence `numerics.rst` cannot carry — "Where the underlying operators are partial, the corresponding instruction will trap when the result is not defined" — which is what makes the record's word "trapping" re-derivable, plus `document/LICENSE`. Vendored under the W3C Software and Document License, whose NOTICE condition is met by the `w3c-document-license-2023` bytes this directory already held for WGSL. The Rust reference at `rust-lang/reference` commit `ad35aca481751a06afeb23820a672b0f3b11a476`, vendored with both `LICENSE-MIT` and `LICENSE-APACHE`; the `expr.as.numeric.float-as-int` rule carries all four parts of the claim including its totality. The C++ working draft **N5054** is metadata-only: ISO/IEC copyright on every page, no grant, and `cplusplus/draft` carries no `LICENSE` at all — §7.3.11 [conv.fpint]'s "The behavior is undefined (F.3.7) if the truncated value cannot be represented in the destination type" was read and the bytes discarded.

**Three identity decisions worth knowing before re-running any of this.** The Rust reference repository carries no tags, so its row is pinned by the `rust-lang/rust` `src/doc/reference` submodule commit at release 1.97.1 (the same commit as 1.97.0), following `nvidia-ptx-isa-cuda-13.3.0`'s "document as shipped in a versioned container" naming. The C++ row is the WG21-published N-numbered PDF, not the `eel.is/c++draft` rendering the record cited — that page tracks a main branch and is not published by WG21; it was retrieved anyway and its normative sentence matches N5054 word for word, which is the evidence that swapping the identity did not swap the text. And `instructions.rst` at `wg-3.0` is mostly SpecTec-generated directives now, so its preserved value is bounded to the hand-written lift sentence; the record says which future citations it would *not* support.

**Population moved 83 → 90 records (64 → 70 vendored, 18 → 19 metadata-only, pending unchanged at 1), and the verifier was watched failing on five perturbations** over the new population in a scratch copy outside the repository, each returning exit 1 with a named reason, with clean exit-0 runs before and after. The fifth is the one that justifies this wave's shape rather than merely repeating the previous wave's: all six new vendored rows deleted from the manifest while their files stayed on disk failed eight times, opening with `manifest holds 84 records, expected 90` and `vendored records: 64, expected 70` — the declared-population guard catching exactly the failure a derived count could not.

**One out-of-scope defect found and filed rather than absorbed.** The record's "Existing contracts" list makes eight claims; seven were cited and are now hardened. The eighth — "PyTorch documents platform variation" — was never cited at all, and the three preserved PyTorch documents are dtype-spelling evidence that does not carry it. Extending the population beyond the seven was this ticket's stated non-goal, so it is [preserve-the-pytorch-conversion-platform-variation-source](preserve-the-pytorch-conversion-platform-variation-source.md), whose likelier outcome is correcting the record's sentence than finding a source for it.
