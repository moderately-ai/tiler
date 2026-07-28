---
id: prototype-quantized-value-vertical
title: Prove a quantized compound-value vertical
status: todo
priority: p2
dependencies: [implement-first-profile-numerical-policies]
related: []
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, dtype, vertical-slice]
---
## User-visible outcome

A quantized value is a typed compound contract — storage plus inseparable metadata, with layout/conversion/materialization stated as three separate contracts — proven end to end on one scheme, with every unsupported scheme refused by name. This is the vertical that decides whether quantization is a dtype or a contract, before any LM work depends on the answer.

Prove quantized values as typed compound storage/metadata contracts rather than integer dtypes alone. Cover metadata association, validation, reference semantics, layout/access lowering, conversion/materialization, identity and ABI binding while explicitly rejecting unsupported schemes and preserving future block/group formats.

## Closes when (2026-07-28)

1. **A quantized value is a typed compound contract, not an integer dtype with a convention attached.** Storage and its metadata — scale, zero point, whatever the scheme requires — are one value type, and the metadata cannot be separated from the storage it describes at any layer. A value whose metadata is carried alongside by caller discipline does not close this.
2. **The numerical contract is stated separately for each of three things, because they are three different contracts.** *Layout and access lowering:* how a packed element is addressed, and what a partial or unaligned access means. *Conversion:* the exact rounding and saturation behaviour quantizing into and dequantizing out of the scheme, including what happens at the representable boundary and on exceptional inputs. *Observable materialization:* the rounding applied when a quantized value becomes a result a caller reads. Folding these into one "quantization contract" is the failure mode — they are separately observable and they can disagree.
3. **Reference semantics exist and are compared against.** The reference oracle computes the same program over the same scheme, and the comparison's tolerance is derived from the scheme's stated conversion contract rather than chosen to make the test pass.
4. **Identity and ABI binding fold the scheme into artifact identity.** Two artifacts differing only in quantization scheme, or only in a scale that changes what the bytes mean, must have different identities. A scheme that reaches the ABI without reaching identity is a silently wrong cache hit waiting to happen.
5. **Every unsupported scheme is refused with a typed error naming it**, at the earliest layer that can name it, and never approximated by the nearest supported scheme. A test drives at least one unsupported scheme and watches the refusal fire.
6. **Block and group formats are reserved as an architectural seam with an explicit record of what broadening would require** — not implemented, not silently excluded. `AGENTS.md` distinguishes four maturity claims: a type-system reservation, an architectural seam, implemented support, and a tested guarantee. Say which of the four this ticket delivers for block/group formats, and record what a fifth reader would need to move it to the next one.
7. **`make full` passes.**

**No field is reserved before its producer exists.** `carry-the-honourability-fact-provenance-into-the-artifact-record.md:27` states the rule and the reason: "a field a producer cannot fill is the producer-less placeholder this repository has repeatedly had to retract, which is exactly why the draft omits them rather than defaulting them." That applies directly here, because a quantized-value vertical is exactly the kind of work that invites reserving a `block_size` or a `group_metadata` field ahead of any code that fills one. Omit it and record what would be needed, rather than defaulting it.

## Dependency note (2026-07-28)

`implement-first-profile-numerical-policies` is `status: in-progress` with completed but **uncommitted** work in the harness worktree `.claude/worktrees/agent-ad2893b1fba4d7f5b`. Its Outcome already states this ticket's seam explicitly, and states it as an *absence* rather than a stub: "Preserved by absence rather than by a placeholder. `ArithmeticType` names scalar float formats; a compound or quantized value is a scheme-typed `ResolvedValueType::encoded_numeric` whose conversion behaviour is its own typed contract, and `operation_capabilities` enumerates only the scalar `f32` operations this build admits, so an operation outside that table has no capability entry and therefore no effective permission to compute."

Two consequences for this ticket. The seam it must build on is `ResolvedValueType::encoded_numeric`, already named — do not introduce a second spelling for a compound value type. And the fail-closed property is currently supplied by `operation_capabilities` having no entry for a quantized operation, which means **admitting one is what removes the current protection**: every capability entry added here must arrive with its conversion contract, or the absence that was doing the work is gone and nothing replaces it.

## Graph maintenance

- **State which of the four maturity claims you delivered for block/group formats** (criterion 6) on this ticket at close — reservation, seam, implemented, or tested — and file the broadening ticket only if you can name its first consumer.
- **Scheme-into-identity** (criterion 4) advances the artifact identity domain: reason recorded at the site, and expect the determinism pins to move exactly once.
- **The compound/quantized seams in the numerical-policy work are preserved by absence** (its worktree Outcome says so explicitly) — when `rebase-and-land-the-stranded-numerical-policies-worktree` lands, check whether its eleven-dimension vocabulary gives your conversion contract a home before inventing one.
