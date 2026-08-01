---
id: retain-the-qwen-conformance-reference-logit-fixture
title: Retain the Qwen conformance reference logit fixture
status: done
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

## Outcome

All four conditions are met. The fixture is [`spikes/program-planning/qwen3-conformance-fixture/`](../spikes/program-planning/qwen3-conformance-fixture/README.md); the retained record was `results/2026-07-31-c1-conformance-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0/`. **Superseded, not invalidated:** [`retain-the-c1-model-attribution-fixture`](retain-the-c1-model-attribution-fixture.md) extended the same producer with L6's attribution surface, which changed the producer's own digest and therefore the manifest that record validates against, so it was regenerated as `results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0/` and the 2026-07-31 directory removed. Every conformance file this outcome describes regenerated byte-identically across the extension — `sequence.tsv`, `positions.tsv`, `top32.tsv`, `envelope.tsv` — so each measurement below still names exactly the bytes that produced it.

**The checkpoint digest is a locally reproduced fact.** All nine manifest files were acquired with `hf download Qwen/Qwen3-0.6B-Base --revision da87bfb608c14b7cf20ba1ce41287e8de496c0cd` into the Hugging Face cache outside this repository and hashed with `shasum -a 256`. Every digest and every byte size matched, including `model.safetensors` at 1,192,135,096 bytes hashing to `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba`. The workload profile's manifest row now reads `local` rather than `API-reported LFS object id`, and the producer re-runs the whole verification on every invocation and stops rather than warns on a mismatch.

**The evaluated implementation was checked, not assumed.** The installed `transformers` 4.51.0 `modeling_qwen3.py`, `configuration_qwen3.py`, and `modeling_rope_utils.py` hash byte-for-byte to the profile's pinned-commit digests, so the PyPI distribution and git commit `0720e206` are the same source for these files. This also supplied `modeling_rope_utils.py`'s size, 32,952 bytes, which the profile's table previously left as `—`.

**The run.** F32 on CPU, `attn_implementation="eager"` passed explicitly, `logits_to_keep=0`, greedy under the declared lowest-index tie policy, `torch.set_num_threads(1)`. 18 retained logit vectors: 10 prefill positions plus 8 decode passes. Termination was the 8-step budget; EOS 151643 never appeared. The 18-token sequence is the 10 prompt tokens followed by `576, 3974, 13876, 38835, 34208, 916, 279, 15678` — the base model restarts the pangram. **No position has a bit-identical top-two pair, and exactly one index attains the maximum everywhere**, so the tie branch is declared and implemented but unexercised by this row.

**The sensitivity envelope, measured two ways because one reading is not enough.** A float64 pass through the pinned reference is *not* uniformly float64: three unconditional float32 spellings — `modeling_qwen3.py:73` in `Qwen3RMSNorm.forward`, the softmax at line 162, and the RoPE table's `.float()` calls at lines 336–344 — become downcasts at model dtype float64, at the three most cancellation-prone stages. In an unmodified float64 pass they round identically to the F32 pass and contribute zero deviation. The fixture therefore retains `f64_unmodified` (the pinned reference verbatim) and `f64_promoted` (those three sites promoted, line-for-line). Largest whole-vocabulary per-position deviation: **2.048e-4** unmodified, **2.007e-4** promoted; over the top-32 entries 7.82e-5 and 7.44e-5, at most 78 ULP for both. The greedy token agrees at every position under both. Under 3% of each position's logits are bit-identical between orderings. No budget is set; `design-model-level-qualification-and-optimization` still owns that.

**Reproducibility and drift detection, both demonstrated rather than asserted.** `produce_fixture.py --compare` regenerated the complete production and all six retained files matched byte-for-byte. `verify_fixture.py` runs 2,721 counted checks needing no model, and was proven able to fail: an altered logit digest against a stale manifest failed on both the manifest re-hash and the logit-byte re-hash; an altered greedy token with a *consistently re-hashed* manifest still failed on the top-32 head and the decode-chain cross-checks; `--compare` reported the same edit as a differing file. The record was restored and re-verified clean afterwards.

**Where the one gitignore entry went.** `spikes/program-planning/qwen3-conformance-fixture/.gitignore`, ignoring `/local-work/` and nothing else. That is the repository's established shape for this — `spikes/apple-targets/.gitignore` ignores exactly `/local-work/` for the same reason — and it keeps the rule beside the directory it describes, which is what the profile's storage policy asks for. The root `.gitignore` is therefore untouched and this ticket's `implementation/workspace` scope went unused. The entry covers both the regenerable F32 logit bytes and the spike's uv environment, which is why the recorded invocation passes `UV_PROJECT_ENVIRONMENT=local-work/venv`: a default `.venv/` in the spike directory would be ignored by nothing.

**Measurement boundary.** Every digest is bound to the host row in `environment.tsv` — Apple M4 Max, macOS 27.0 build 26A5388g, `torch` 2.6.0, Python 3.11.12, single-threaded. Single-threaded evaluation removes intra-op reduction-order variation between runs on this host; it does not make the digests portable to another CPU, BLAS, or thread count, and a mismatch elsewhere is expected rather than a defect. The envelope qualifies one prompt, one checkpoint, and one host. The CPU reference preserves subnormals while the qualified Metal row flushes them, and that divergence source is named in the profile and not measured here.

**Follow-up this work revealed and did not absorb.** The spike catalog block in `spikes/README.md` needs a row for this experiment under "Physical planning and lowering", the way `spikes/numerics/sound_accuracy/` and `spikes/target-profiles/scalar-cpu-vertical/` each carry one. That file belongs to the `contracts/navigation` scope, which this ticket does not declare, so the row is deliberately not added here.
