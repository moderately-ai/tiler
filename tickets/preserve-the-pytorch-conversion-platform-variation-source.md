---
id: preserve-the-pytorch-conversion-platform-variation-source
title: Preserve the PyTorch float-to-integer platform-variation source
status: done
priority: p3
dependencies: []
related: [preserve-the-float-to-integer-conversion-precedent-sources]
scopes: [research/numerics]
shared_scopes: [project/tickets]
tags: [research, numerics, conversion, sources, preservation]
paths: []
---
## User-visible outcome

The one claim in [float-to-integer conversion precedents](../docs/research/numerics/float-to-integer-conversion-precedents.md) that no source supports — that PyTorch documents platform variation in float-to-integer conversion — either names a preserved-source id like its seven neighbours, or is removed as unsupported, so that no sentence in the `evidence` record for ADRs 0010 and 0041 rests on nothing at all.

## The defect, stated so it can be reproduced or refuted in one line

**Fact.** The record's "Existing contracts" list makes eight claims and its "Primary sources" list covers seven of them. The uncovered one is the fourth bullet's middle clause: "C++ makes an unrepresentable result undefined. StableHLO leaves it TBD, and **PyTorch documents platform variation**." The C++ and StableHLO halves of that bullet are cited; the PyTorch half never was, in either the pre- or post-hardening source list.

```sh
rg -n 'PyTorch|pytorch' docs/research/numerics/float-to-integer-conversion-precedents.md
```

**Fact.** This was found while executing [the provenance-hardening ticket](preserve-the-float-to-integer-conversion-precedent-sources.md), which closed all seven cited URLs into manifest ids on 2026-08-06. That ticket's stated population was the seven citations and its explicit non-goal was extending beyond them, so the gap is filed here rather than absorbed there.

**Fact — the existing PyTorch rows do not cover it.** Three PyTorch documents are already preserved at `pytorch/pytorch` `v2.13.0`: `pytorch-tensor-attributes-v2.13.0`, `pytorch-complex-numbers-v2.13.0`, and `pytorch-scalar-type-v2.13.0`. All three are dtype spelling and exposure evidence, cited for the `torch.dtype` table, the complex spellings, and the internal scalar widths. None of them documents conversion behaviour, so the claim is not re-derivable from bytes this repository already holds. Reproduce with `grep -rniE 'platform|undefined|out of range|saturat' docs/research/numerics/sources/pytorch-v2.13.0/`, which returns exactly two lines on 2026-08-06, both in `ScalarType.h` and both unrelated — a `TORCH_CHECK(false, "Bits types are undefined")` guard and the `ScalarType::Undefined` enum case. Two hits rather than zero is why the expected output is written out here: a reader who ran the grep, saw matches, and stopped would draw the opposite conclusion. **Correction from this ticket's own work:** that command now returns six lines, because it vendored a fourth PyTorch file into the same directory. Two are the `ScalarType.h` hits above; three of the four new ones are `_tensor_docs.py` docstrings about `index_put_` accumulation and about mutating a tensor shared with NumPy, still unrelated; the fourth, line 5165, is the sentence this ticket was filed to find. The stale count is left in place above with the correction beside it, because the reasoning it illustrates — that a nonzero hit count is not an answer — is exactly what the new hits demonstrate.

## What the work is

- Establish whether PyTorch documents this at all, at the pinned `v2.13.0` revision, and where — the casting sections of `docs/source/tensor_attributes.md` and `docs/source/tensors.md`, `Tensor.to`, `Tensor.int`, or the ATen conversion implementation, in that order of authority. The claim is about *documented* variation, so an implementation detail that is merely observed is not evidence for it.
- If it is documented, preserve the exact file at the same `v2.13.0` commit `cf30153c4c131c8164ee7798e5022d810682e2cb` the three existing rows use, so PyTorch does not appear in this manifest at two revisions, and add the id to the record's source list. `pytorch-license-v2.13.0` and `pytorch-notice-v2.13.0` are already preserved and need no second row.
- **If it is not documented, that is the more likely outcome and the more useful one:** report it, and correct the record's sentence rather than hunting for a source that would justify it. A claim in an adopted evidence record that its own ecosystem does not make is a defect in the record, and the correction is a research-record fix with the finding stated — not a reopening of ADR 0010 or ADR 0041, whose rationale rests on the seven verified precedents and not on this clause.
- Update `expected-sources.tsv`, the declared population counts at the top of `verify-sources.sh`, and the preservation record's prose in the same change if a row is added, and watch `verify-sources.sh` fail on a deliberate perturbation before trusting a pass.

## Explicit non-goals

- Reopening ADR 0010 or ADR 0041.
- Re-checking the seven citations that the hardening ticket already closed against preserved bytes on 2026-08-06. Each held; that work is done and its evidence is in the preservation record.
- Extending the population to any other PyTorch document.

## Closes when

The PyTorch clause either names a manifest id whose bytes carry it, or is gone from the record with the reason recorded; and, if a row was added, the verifier passes on the updated population and was watched failing on a perturbation.

## Outcome

**The acquisition succeeded, which is not what this ticket predicted.** PyTorch does document the variation, and the claim holds close to verbatim, so the sentence was preserved rather than corrected. It is documented in a *method docstring in Python source* rather than in the documentation directory — `torch/_tensor_docs.py`, the `Tensor.to` docstring — which is why the search that filed the gap missed it: every earlier look was inside `docs/source/`, and the three preserved PyTorch documents are all from there or from `c10/`. The note reads, at commit `cf30153c4c131c8164ee7798e5022d810682e2cb`: "According to `C++ type conversion rules`, converting floating point value to integer type will truncate the fractional part. If the truncated value cannot fit into the target type (e.g., casting ``torch.inf`` to ``torch.long``), the behavior is undefined and the result may vary across platforms."

**Documented, not merely observed — the distinction this ticket turned on.** A docstring is source, so "documented" needed a second check: `https://docs.pytorch.org/docs/2.13/generated/torch.Tensor.to.html` returned HTTP 200 and 224 472 bytes carrying the sentence verbatim. The `stable` alias for the same page returned 1 395 bytes of client-side redirect to `../../2.13/`, which both confirms `stable` was 2.13 on the retrieval date and repeats the shell finding already recorded against the older PyTorch rows.

**Preserved as `pytorch-tensor-docs-v2.13.0`** — `pytorch-v2.13.0/_tensor_docs.py`, 142 383 bytes, sha256 `b1a46e328a74e39383dbecf04fb48620af70c7780467cf923864d1b5cf992e99`, at the same `v2.13.0` pin the three existing rows use, so PyTorch is not in the manifest at two revisions and no second licence row was needed (`torch/` is PyTorch's own BSD-3 source, not the `third_party` the preserved `NOTICE` covers). Retrieved twice by independent routes — `raw.githubusercontent.com` at the commit, and the commit tarball — byte-identical; and `git hash-object` over the bytes is `96daedc701125a152d84b92f62138d8ea1a3c591`, the blob SHA the GitHub contents API reports for that path at that commit, so the preserved copy is the repository's own object rather than a matching download.

**Two precisions, neither reopening ADR 0010 or ADR 0041.** PyTorch does not reach its undefinedness independently — the note attributes it to C++ conversion rules — so it is C++'s residue imported by reference and belongs in the same bullet rather than as an eighth distinct answer; and it fixes the rounding as truncation, like the five other truncating systems. Both are recorded in the research record's new "eighth claim" paragraph.

**The negative half of the search is recorded rather than dropped**, because "documented" is a claim about absence everywhere else: `docs/source/tensor_attributes.md`, `tensors.md`, `notes/numerical_accuracy.md`, `type_info.md`, and `notes/faq.md` at this commit carry no such statement (`grep -niE 'platform|implementation.defined|undefined behavio|out of range|saturat|overflow'` over those five returns four hits, all in `numerical_accuracy.md` and all about accumulation and cross-device reproducibility), and `torch/_torch_docs.py` — the free-function docstrings, deliberately **not** preserved — carries only QR and SVD non-uniqueness notes on the same pattern. **No acquisition request is filed:** nothing this ticket needed was unreachable.

**Population moved 90 → 91 records (70 → 71 vendored; metadata-only and pending unchanged at 19 and 1), and the verifier was watched failing on six perturbations** over the new population in a scratch copy outside the repository, each returning exit 1 with a named reason, with clean exit-0 runs before and after: deleted vendored file; mutated manifest digest; stray file inside the shared `pytorch-v2.13.0/` directory (which a directory-keyed check would have passed, since that directory was already declared); the row deleted while its bytes stayed on disk (three failures, opening `manifest holds 90 records, expected 91`); the row reclassified to `metadata-only` (four failures at once); and a mutation of the preserved bytes themselves — one appended newline, manifest untouched — which is the one this row most needed, a Python module being one of only two documents here a formatter would recognize as its own input.

The record's claim-to-source mapping is now complete: eight claims, eight resolutions.
