---
id: preserve-the-pytorch-conversion-platform-variation-source
title: Preserve the PyTorch float-to-integer platform-variation source
status: in-progress
priority: p3
dependencies: []
related: [preserve-the-float-to-integer-conversion-precedent-sources]
scopes: [research/numerics]
shared_scopes: [project/tickets]
tags: [research, numerics, conversion, sources, preservation]
paths: []
claimed_from: todo
assignee: agent-pytorch-source
lease_expires_at: 1786039087
---
## User-visible outcome

The one claim in [float-to-integer conversion precedents](../docs/research/numerics/float-to-integer-conversion-precedents.md) that no source supports — that PyTorch documents platform variation in float-to-integer conversion — either names a preserved-source id like its seven neighbours, or is removed as unsupported, so that no sentence in the `evidence` record for ADRs 0010 and 0041 rests on nothing at all.

## The defect, stated so it can be reproduced or refuted in one line

**Fact.** The record's "Existing contracts" list makes eight claims and its "Primary sources" list covers seven of them. The uncovered one is the fourth bullet's middle clause: "C++ makes an unrepresentable result undefined. StableHLO leaves it TBD, and **PyTorch documents platform variation**." The C++ and StableHLO halves of that bullet are cited; the PyTorch half never was, in either the pre- or post-hardening source list.

```sh
rg -n 'PyTorch|pytorch' docs/research/numerics/float-to-integer-conversion-precedents.md
```

**Fact.** This was found while executing [the provenance-hardening ticket](preserve-the-float-to-integer-conversion-precedent-sources.md), which closed all seven cited URLs into manifest ids on 2026-08-06. That ticket's stated population was the seven citations and its explicit non-goal was extending beyond them, so the gap is filed here rather than absorbed there.

**Fact — the existing PyTorch rows do not cover it.** Three PyTorch documents are already preserved at `pytorch/pytorch` `v2.13.0`: `pytorch-tensor-attributes-v2.13.0`, `pytorch-complex-numbers-v2.13.0`, and `pytorch-scalar-type-v2.13.0`. All three are dtype spelling and exposure evidence, cited for the `torch.dtype` table, the complex spellings, and the internal scalar widths. None of them documents conversion behaviour, so the claim is not re-derivable from bytes this repository already holds. Reproduce with `grep -rniE 'platform|undefined|out of range|saturat' docs/research/numerics/sources/pytorch-v2.13.0/`, which returns exactly two lines on 2026-08-06, both in `ScalarType.h` and both unrelated — a `TORCH_CHECK(false, "Bits types are undefined")` guard and the `ScalarType::Undefined` enum case. Two hits rather than zero is why the expected output is written out here: a reader who ran the grep, saw matches, and stopped would draw the opposite conclusion.

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
