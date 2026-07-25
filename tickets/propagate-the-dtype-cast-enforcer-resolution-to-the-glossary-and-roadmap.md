---
id: propagate-the-dtype-cast-enforcer-resolution-to-the-glossary-and-roadmap
title: Propagate the dtype-cast enforcer resolution to the glossary and roadmap
status: done
priority: p2
dependencies: []
related: [reconcile-dtype-cast-enforcer-with-boundary-properties]
scopes: [contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, numerics]
---
`reconcile-dtype-cast-enforcer-with-boundary-properties` settled that a dtype cast is not a boundary enforcer and removed it from the enforcer list in `docs/compiler/optimizer.md`. An enforcer may change only how a boundary value is stored, addressed, placed, or delivered, never which values it carries; a cast is a semantic operation carrying a resolved typed conversion contract under ADR 0010, so a schedule may neither insert one nor elide one. Two documents outside that ticket's scope still assert the old list.

`docs/glossary.md` defines "Boundary enforcer" as "Explicit materialization, layout conversion, cast, or copy that satisfies a boundary requirement." Drop `cast` from that definition; materialization, layout conversion, and copy are all value-preserving and the entry reads correctly without it.

`docs/roadmap.md` states, in the Milestone 6 evidence block, that the optimizer "lists contiguous materialization, layout conversion, and dtype cast as enforcers that supply a missing required property at a cost". That sentence is labelled **Fact** and is now a misquotation of the contract. Its surrounding inference — that layout conversion is already an enforcer rather than a new mechanism — is unaffected and should survive the correction.

Check no other occurrence survives with `grep -rn -i "enforcer" docs/` and confirm each remaining hit describes a value-preserving stage.

## Outcome

Both documents corrected. `uv run --locked python scripts/check_repository.py` passes.

**Correction to the ticket's instruction for `docs/roadmap.md`, made rather than followed literally.** The ticket asked only that dtype cast stop being listed. Dropping it would have left the sentence enumerating two of the contract's three enforcers while claiming to report what [Optimizer](../docs/compiler/optimizer.md) "lists", which is a different misquotation of the same list. `docs/compiler/optimizer.md:336-338` lists exactly contiguous materialization, layout conversion, and encoding repacking, so the roadmap now names the third rather than dropping the wrong one. The surrounding **Inference** — that layout conversion is already an enforcer rather than a new mechanism — is untouched, as the ticket required.

**`docs/glossary.md`.** `cast` dropped from the Boundary enforcer entry. One clause added recording *why* it is absent, because the ticket that settled this found the bare list was what let the cast be read back in; `docs/compiler/optimizer.md:342` remains the authority and the glossary cites the rule rather than restating its reasoning.

**Sweep — `grep -rn -i "enforcer" docs/`, every hit read.** 51 hits across 9 files after the change: `docs/research/transfers/transfer-synchronization-and-resource-lifetime.md` 23, `docs/compiler/optimizer.md` 11, `docs/research/placement/device-placement-and-memory-domains.md` 5, `docs/roadmap.md` 5, `docs/decisions/0047-model-placement-as-physical-properties.md` 2, `docs/prior-art/logical-graphs-and-schedules.md` 2, and one each in `docs/artifact-abi.md`, `docs/compiler/fusion-and-scheduling.md`, and `docs/glossary.md`. (An earlier draft of this outcome said 34 hits across 10 files; that was an eyeball estimate of the grep output and is retracted — the figures above are counted.) After this change no document asserts a dtype cast is an enforcer. The remaining hits are value-preserving or out of scope by construction: `docs/artifact-abi.md:241`, `docs/roadmap.md:218,263,272,314`, and `docs/compiler/fusion-and-scheduling.md:308` name layout/contiguous enforcers or the generic machinery; `docs/prior-art/logical-graphs-and-schedules.md:187,200` describe external systems; `docs/research/placement/device-placement-and-memory-domains.md` names transfer/import/repack; and `docs/decisions/0047-model-placement-as-physical-properties.md:60` names transfer/import, materialization/repacking, and legal recomputation.

**Already correct, and left alone.** `docs/research/transfers/transfer-synchronization-and-resource-lifetime.md:29-31` states that dtype conversion "is a separately typed stage and is deliberately **not** a member" of the enforcer family, and derives the split from the accepted definition. That record needed no propagation; its scope is `research/transfers` and was not entered.

**Measurement boundary.** This is a prose-coherence correction verified by reading every `enforcer` hit in `docs/` at this commit. Nothing mechanically prevents the list from drifting again — no check ties the roadmap's or glossary's summary to `docs/compiler/optimizer.md`'s enforcer list, and a future edit to either can desynchronize them silently.
