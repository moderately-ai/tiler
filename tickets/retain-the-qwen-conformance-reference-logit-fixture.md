---
id: retain-the-qwen-conformance-reference-logit-fixture
title: Retain the Qwen conformance reference logit fixture
status: todo
priority: p1
dependencies: [define-first-metal-lm-workload]
related: [derive-transformer-operation-and-shape-surface, design-model-level-qualification-and-optimization]
scopes: [research/program-planning, implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [research, language-model, conformance, fixture, qwen, numerics]
---
## User-visible outcome

The conformance row of the first Metal language-model workload has retained, reproducible reference evidence: for the pinned `Qwen/Qwen3-0.6B-Base` checkpoint and the recorded 10-token prompt, an F32 reference produces the logits of every prefill position and every decode step, and a small checked-in record lets any later rung detect drift without re-deriving what the reference should have said.

## Evidence prerequisite

[`docs/research/program-planning/first-metal-lm-workload.md`](../docs/research/program-planning/first-metal-lm-workload.md) supplies everything this ticket consumes and must not re-derive: the pinned model revision and per-file SHA-256 manifest, the storage policy and acquisition route, the exact conformance prompt and its token IDs, the decode budget and termination rule, the tie policy, and the effective F32 numerical policy. Run against that document, not against remembered configuration.

## Required work

- Acquire the checkpoint by the recorded route and **verify `model.safetensors` locally against `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba` before any other step**. That digest is currently the repository API's Git-LFS object id, recorded as an identity to check rather than a digest computed from bytes; this ticket is what converts it into a locally reproduced fact, and a mismatch is a stop, not a warning.
- Evaluate the pinned `transformers` v4.51.0 `Qwen3ForCausalLM` reference in F32 on CPU, with `attn_implementation="eager"` passed explicitly. The eager path is the definitional one; an SDPA or Flash path is a different implementation whose reduction order is not fixed by the pinned source, so a fixture produced under an unpinned attention implementation is not reproducible evidence.
- Compute logits for **every** prefill position — `logits_to_keep=0` in the pinned reference — and for every decode step, greedy, terminating on EOS 151643 or the 8-step budget.
- Retain as checked-in evidence, small enough to review: a SHA-256 over each position's exact F32 logit bit pattern, the full-precision top-32 logits and indices per position, the greedy token and runner-up gap per position, any position where the top two logits are bit-identical, the 18 emitted token IDs, and the exact environment (Python, torch, transformers versions and the host row).
- Keep the complete F32 logit bytes as regenerable local data outside version control, and add the one narrow `.gitignore` entry that covers exactly that directory — nothing broader.
- Preserve the producer under `spikes/program-planning/` with a README recording the exact hand-run invocation, per the repository's spike discipline. No `make` target reaches it.

## Additionally required by the same run, because it costs one more evaluation

Record the reference's own F32 sensitivity envelope for this exact prompt and checkpoint: evaluate the reference a second time in float64 and round to F32 at the observable, and retain the per-position logit deviation between the two orderings. This is the measurable half of the model-level comparison budget, it needs no Tiler execution, and it is the smallest deviation any correct F32 realization could be required to fall inside. It does not set the budget — `design-model-level-qualification-and-optimization` owns that — it supplies the evidence that rung would otherwise have to produce from scratch.

## Explicit non-goals

No Tiler execution, no Metal work, no operation-family derivation, no comparison threshold, and no benchmark row. The B1 matrix is a performance row and is not fixtured here.

## Reconsideration trigger

If the pinned checkpoint revision, the conformance prompt, the decode budget, or the pinned reference revision is superseded, this fixture is re-derived rather than patched, and the superseding change says which retained rows survived. If the workload selection itself is superseded, this ticket is closed rather than migrated.

## Closes when

The checkpoint digest is locally verified against the manifest, the retained record reproduces from the checked-in producer on a stated host, the sensitivity envelope is retained beside it, and `docs/research/program-planning/first-metal-lm-workload.md` links the fixture from its conformance-row section.
