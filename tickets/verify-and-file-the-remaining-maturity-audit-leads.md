---
id: verify-and-file-the-remaining-maturity-audit-leads
title: Verify and file the remaining maturity-audit leads
status: todo
priority: p3
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Six leads from the 2026-08-07 maturity audit, verified to differing degrees

The audit returned ten findings. **Four were verified by the coordinator and filed**; one more (`manifest schema`) is filed separately. These six remain, and **each must be re-verified before it becomes a repair** — the audit's base moved three times mid-run, and its own line numbers had already drifted once.

Filed as one ticket rather than six because each is small and they share a method. **Do not repair any of them on this ticket's say-so.**

1. **ADR 0074 asserts the gate runs a probe the gate cannot reach.** It says `spikes/extensions/run.py --self-test` is run by the repository gate and that the gate "compiles the workspace on every invocation". The `Makefile` header reportedly says *"Spikes deliberately have no target."* Load-bearing because convention 5b's Measurement closes by claiming the comparison "forces a fresh run at the next pin migration" — if nothing runs it, that mechanism does not exist and the measurement survives only by the toolchain pin happening not to have moved.
2. **`docs/dtype-support.md` routes BF16's remaining rungs to "the live tickets", and the audit reports that population as empty.** **The coordinator could not confirm this at the stated precision** — a scan of tickets referenced from that file found 16 non-terminal, but they are integer, quantized and sub-byte owners rather than the BF16 set under D-4. So the narrow claim may hold and the broad reading does not. **Check D-4's own eight owners specifically** before concluding anything.
3. **`docs/artifact-abi.md` contradicts itself on its own domain count** — "seven", "eight", and "four domain separators" in one document. Partly covered by `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check`, which reconciles counts against the true population; **coordinate rather than duplicating**.
4. **ADR 0076 contradicts its own dated correction inside the same bullet** — the correction says the harness "is no longer collected by anything", then the same bullet closes "reproduced by the harness the gate runs", and a second passage repeats it. A research record reportedly agrees with the correction. The conclusion survives; two surviving sentences are the stale ground.
5. **`docs/artifact-abi.md`'s "the five that carry no cooperative tile" is nine of ten.** The paragraph names five goldens; the directory holds ten and exactly one stages a barrier. The *claim* survives and is stronger (1 of 10); the enumeration no longer names its population.
6. **Two ledger cells say "until X lands" for a ticket that landed and recorded the opposite** — the named ticket is `done` and its Outcome says the thing is structural and permanent, so readers are sent to a closed ticket expecting a ruled-out fix.

## Method

For each: re-read the source in full at your base, state **verified / false / imprecise**, and separate whether the *conclusion* survives from whether its *stated ground* does. That distinction has mattered repeatedly here — a false premise does not automatically make a false conclusion, and several of these are conclusions that survive on other grounds.

Then **file each survivor as its own ticket in its own scope** rather than repairing across scopes from here. This ticket holds only `project/tickets`.

## Closes when

Each of the six carries a verified verdict; every survivor is filed in its owning scope with its evidence; and any that did not survive verification is recorded as refuted so a later audit does not re-raise it.
