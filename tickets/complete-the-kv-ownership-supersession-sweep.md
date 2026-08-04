---
id: complete-the-kv-ownership-supersession-sweep
title: Complete the KV-ownership supersession sweep across the research corpus
status: done
priority: p1
dependencies: []
related: [supersede-the-runtime-owned-kv-state-design, reclassify-language-model-work-as-a-conformance-track, design-autoregressive-state-and-kv-cache, establish-a-dynamic-kv-physical-layout-authority, spike-first-metal-contraction-vertical, bind-repeated-invocations-over-caller-retained-tensors]
scopes: [research/runtime, research/shapes, research/scheduling, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, supersession, consumer-neutral, kv-cache, corpus-consistency]
---
## User-visible outcome

No sentence in the research corpus still places a KV cache, a capacity, a cursor, a valid length, or a "state contract" at Tiler's runtime, and no record still blocks a Tiler scheduling decision on a Tiler-owned state model that was withdrawn.

## Why this exists, and why it is one ticket rather than four

[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md) (done, 2026-08-04) corrected the ownership tables, the record headers, and the ticket chain. Its stated outcome — "every KV/model-state ownership claim has an explicit disposition" — is not yet supported: ten sentences survive outside every correction marker, in four research areas. Found on 2026-08-04 while inventorying the language-model nodes for [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md); each line below was read directly rather than taken from a search hit.

Seven of the ten sit in scopes that ticket held and three do not, and an ownership sweep executed in two halves is worse than one executed once: a corpus where seven sentences say "the consumer" and three still say "the runtime instance" reads as a live disagreement rather than as a finished correction. So the whole sweep is filed here, undivided.

## The exact lines

**Withdrawn refusals still asserted at the runtime — `research/runtime`.**

- `docs/research/runtime/autoregressive-state-and-kv-cache.md:187` — "a context beyond the state's own `capacity` refuses at the runtime instance, and the two are different refusals with different remedies". Line 284 of the same file withdrew this refusal and [the L8 record](../docs/research/program-planning/model-level-qualification.md) cites it as withdrawn; the sentence itself was never edited, so the file both withdraws and asserts it.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md:274` — "the only quantity that could refuse this is the state's own valid length, held at the runtime instance". Same object, same owner, uncorrected.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md:144` — "there are three caches in this stack and the KV state must stay out of all of them". There is no such object in the stack to keep out of them; the generic rule that survives is about what a compilation or expansion cache may key on.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md:189` — "Batching would make `capacity` and the cursor per sequence within a governed storage population, which is a different state model … It is not reserved here and would be new architectural work." Frames per-sequence capacity and cursor as prospective *Tiler* architectural work.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md:311` — "the first state contract is an ordinary dense-decoder KV cache and this is it". This is the strongest of the ten: it names a decoder KV cache as Tiler's first state contract, which is the product-goal framing the reclassification rejects rather than only an ownership slip.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md:140` — "The cursor's granularity must equal the program-boundary granularity. … Per-layer programs need per-layer cursors; one program per step needs one." The rows above and below it were corrected to "Consumer"; this normative rule was not, so it reads as a Tiler contract. Lower confidence than the five above: it may be correct as a *consumer* rule and need only its owner named.
- `docs/research/runtime/dynamic-kv-physical-layout.md:83` — "the new bank plus cursor publish atomically". The file header corrected the pool's owner; this clause still has Tiler publishing a cursor.
- `docs/research/runtime/dynamic-kv-physical-layout.md:227` — "The KV artifact/runtime ticket depends directly on the live-extent carrier and must not add a KV-specific stride schema." That ticket was rewritten and renamed to [`bind-repeated-invocations-over-caller-retained-tensors`](bind-repeated-invocations-over-caller-retained-tensors.md); the reference is stale as well as KV-named.

**A live blocker on a withdrawn model — `research/scheduling`.**

- `docs/research/scheduling/first-metal-contraction-realizations.md:180` — "**Structures 2 and 3.** The attention score and value contractions wait on L5's state model, as derived above."
- `docs/research/scheduling/first-metal-contraction-realizations.md:42` — the derivation the line above cites, which excludes contraction index structures 2 and 3 because their operands are produced by that model.

This pair is the most consequential of the ten, because it is not stale prose: it is the recorded reason two of the workload's three contraction index structures are unscheduled. Under the supersession those operands are ordinary caller-bound tensors, so whatever still blocks structures 2 and 3 has to be restated in terms that survive — the extent symbols, the residency predicate, and the decode-step shapes — or the exclusion has to be lifted. **Do not simply delete the blocker**: establish what the real one is first, because "the reason evaporated" and "the reason was misattributed" have different consequences for the schedule.

**Rung attributions that still name a Tiler-owned cache — `research/shapes`.**

- `docs/research/shapes/transformer-operation-and-shape-surface.md:120` — "The one place a concatenate genuinely appears is the KV-cache append, which crosses the state boundary and belongs to L5." There is no Tiler state boundary for it to cross.
- `docs/research/shapes/transformer-operation-and-shape-surface.md:181` — "**The KV-cache state model.** L5's." The uncorrected twin of line 174, which the same supersession's integrator did correct.
- `docs/research/shapes/transformer-operation-and-shape-surface.md:76` — the "KV cache append" table row, which frames "the state surface" as an unsolved Tiler prerequisite.
- `docs/research/shapes/sequence-extending-tensor-family.md:133` — "If L5 relaxes it for a runtime-owned cache, the relaxation is a named contract with its own identity". Contemplates a runtime-owned cache as a live Tiler design branch.
- `docs/research/shapes/sequence-extending-tensor-family.md:24` — "L5 … owns the state model and inherits this record's result." Line 138 of the same file was corrected on 2026-08-04 and this one was not, so the file contradicts itself.

**Rung attributions in `research/program-planning`.** Weaker than the above and possibly already true under the conformance-track reading, where L5 designs the *consumer's* retained tensors; check each and either correct it or record why it stands.

- `docs/research/program-planning/first-attention-program-vertical.md:36`, `:162`, `:384` — three "the KV cache … is L5's" attributions.
- `docs/research/program-planning/complete-model-ingestion-and-execution.md:158` — a model-cursor and per-member generation rule stated with no owner, two lines above the row corrected to "Consumer".

## Two signpost lines the same sweep must finish

`reclassify-language-model-work-as-a-conformance-track` moved six of the eight rung status lines from "rung Lx of the language-model inference ladder" to "… conformance track", matching the wording the supersession had already given L5. Two are outside every scope it held and are still unmoved:

- `docs/research/shapes/transformer-operation-and-shape-surface.md:19` (L2)
- `docs/research/scheduling/first-metal-contraction-realizations.md:19` (L3)

Reproduce the remainder with `grep -rn 'language-model inference ladder' docs/`, which must return nothing when this ticket closes.

## Required discipline

Preserve the original rationale at every site; date each correction; withdraw a claim explicitly rather than deleting the sentence that carried it, exactly as the supersession did in the records it did reach. No measurement, byte figure, elimination, or oracle moves — only owners and the two blocker restatements.

## Closes when

`grep -rn 'language-model inference ladder' docs/` returns nothing; no sentence in `docs/research/` places a cache, capacity, cursor, valid length, generation, or state contract at Tiler's runtime without a dated withdrawal attached to that sentence; the contraction structure 2/3 exclusion names a blocker that survives the supersession or is lifted; and the stale ticket reference at `dynamic-kv-physical-layout.md:227` names the renamed ticket.

## Outcome

**All enumerated sites corrected, 2026-08-04.** Every line was located by content rather than by the cited offset; several had moved. Original wording is preserved at every site with a dated correction attached, so a reader can check each derivation rather than take the correction on trust.

### Structures 2 and 3 — the exclusion is lifted, and the two halves of its ground differ

`docs/research/scheduling/first-metal-contraction-realizations.md` recorded two contraction index structures as unscheduled because "the schedule that produces those operands is the KV-state model L5 owns and L2 recorded as absent in both of its candidate mechanisms". Read in full, that is two claims, and they fail differently — which is why the correction states both rather than deleting the sentence.

- **The dependency was misattributed.** Nothing in Tiler produces those operands across invocations: the consumer holds them and binds them as ordinary program inputs. [The L4 prefill block](../docs/research/program-planning/first-attention-program-vertical.md) had already shown the same thing without needing the supersession — at `S = T` the block computes its own `K` and `V`, so both structures' operands are program-internal values. [`admit-the-attention-contraction-structures`](admit-the-attention-contraction-structures.md) states this in its own "Why it is separate from the projection structure" section and admitted both structures on that basis.
- **The undefined production was discharged by evidence, not evaporated.** "Absent in both candidate mechanisms" was L2's accurate reading of its own moment. [The sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) then eliminated the windowed write and the outside-the-program blit and selected the semantic `Concatenate`, which [`admit-the-sequence-extension-concatenate-family`](admit-the-sequence-extension-concatenate-family.md) registered as `tiler::concatenate-f32@1`; the layout the record refused to prejudge was selected by measurement in [Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md), whose supersession moved only the pool's owner and left the packing and address order untouched.

**No new scheduling blocker exists and no ticket was filed.** What survives the lift is a measurement boundary the record already states elsewhere — the batched forms are unmeasured deliberately — and [`realize-the-attention-contractions-on-metal`](realize-the-attention-contractions-on-metal.md) already owns their first measurements. Two of the record's own results carry to that work as inputs rather than as obstacles: structure 3 contracts over `S`, so `tiled`'s `K ≡ 0 (mod 16)` precondition is evaluated per binding and holds at one of C1's nine executions; and the K-padding neutrality obligation this profile records as untriggered is reached immediately by a contracted `S`, discharged by the existing refusal rather than by padding.

### Folded in, because these files were already under correction here

- **Two ladder signposts** (`transformer-operation-and-shape-surface.md`, `first-metal-contraction-realizations.md`) moved to "language-model conformance track", completing [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md)'s eight.
- **A stale support-matrix rung** in the same L3 status line, which read "the row sits at R3". [`admit-the-attention-contraction-structures`](admit-the-attention-contraction-structures.md) flagged it as out of its own scope and left it; the row has since reached R4 and then R6 for a whole-program contraction occurrence (`docs/roadmap.md`, the tensor-contraction row).
- **Nine residuals the enumeration missed**, all found by the closure sweep and all in scopes this ticket already holds. Five keep `autoregressive-state-and-kv-cache.md` self-consistent with its own corrected sentences — the P3 prefill candidate's "runtime object", the "live runtime state length" launch-preflight gap, Case 2's trailing "stale runtime state" inference, and two "state contract" attributions. Four are in `research/program-planning`: two traceability links reading "L5's state contract", a `CustomOp1` arity sentence reading "the three retained outputs the KV state needs", and a capacity definition reading "each KV state's fixed logical capacity".

### Deliberately left, with reasons

- `docs/research/placement/device-placement-and-memory-domains.md`, `docs/research/artifacts/target-neutral-artifact-envelope.md`, `docs/research/transfers/transfer-synchronization-and-resource-lifetime.md`, `docs/research/extensions/backend-provider-composition.md` — "runtime instance" and "runtime state" there are generic device-identity and materialized-placement concepts with no KV subject, and all four are outside this ticket's scopes.
- `docs/research/shapes/transformer-operation-and-shape-surface.md` "Sequence-extending state write" and its twin in the sequence-extending record — a requirement/occurrence name describing what the *reference model* does, not an ownership claim about Tiler.
- `docs/research/numerics/sources/**` — vendored third-party specifications.
- `docs/research/runtime/autoregressive-state-and-kv-cache.md` lines 54–85 — the "state contract, one line each" block, left in its original wording under the section's own explicit "superseded as a Tiler object" marker and preamble, exactly as the first supersession pass intended.

### Closure checks

Run from the repository root; each states the population it covers so a null result is distinguishable from a check that did not run.

```sh
# 1. The reclassification's own closure condition.
grep -rn 'language-model inference ladder' docs/
#    Returns nothing. Proof it can say no: the same grep against the pre-sweep
#    file returns one hit —
#    `git show d9eba031:docs/research/scheduling/first-metal-contraction-realizations.md | grep -c 'language-model inference ladder'` → 1.

# 2. The ownership pattern sweep, over a named and counted population.
grep -rlE 'runtime instance|runtime-owned|runtime owns|the runtime state|runtime state|KV state|KV-state|LiveStateScope|state contract|poisoned|state boundary|state model|state surface|state write' docs/research/
#    11 files of 81 under docs/research/. Every hit line was read: each is a
#    dated withdrawal, a quotation inside one, a generic runtime/device concept,
#    or a vendored source. Proof it can say no:
#    `git show d9eba031:docs/research/scheduling/first-metal-contraction-realizations.md | grep -c "wait on L5's state model"` → 1,
#    against 0 in the working tree.

# 3. The two ownership-verb and valid-length residual sweeps.
grep -rnE 'at the runtime|the runtime (owns|holds|publishes|retains|advances|keeps)|runtime-held' docs/research/
grep -rn 'valid length\|valid range' docs/research/
#    Every remaining hit is corrected, generic, or vendored; enumerated above.

# 4. The renamed ticket reference.
grep -n 'bind-repeated-invocations-over-caller-retained-tensors' docs/research/runtime/dynamic-kv-physical-layout.md
```
