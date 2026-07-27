---
id: intern-the-lowering-registry-s-shared-authority-identities
title: Intern the lowering registry's shared authority identities
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [performance]
---

The canonical explain subject was 20,194 bytes for a five-operation program. It is hashed byte-at-a-time once per compilation to derive the explain writer's request qualifier, and compared whenever a record's evidence is bound to its compilation, so its size is paid on the compile path rather than only when a trace is rendered.

## Measurement — the byte budget

Measured by `the_explain_subject_byte_budget` (`crates/tiler-compiler/src/request.rs`), which is retained as the evidence for this ticket:

| component | bytes | share |
| --- | --- | --- |
| **lowering registry identity** | **15,583** | **77.2%** |
| registry snapshot | 1,496 | 7.4% |
| reached definitions | 1,294 | 6.4% |
| semantic graph | 879 | 4.4% |
| admission provenance | 451 | 2.2% |
| target honourability declarations | 56 | 0.3% |
| keys, shapes, budgets, contracts, framing | 435 | 2.2% |

**Fact: the registry snapshot appeared five times inside the registry identity**, 7,480 bytes of the 15,583. Counted in the encoded bytes rather than inferred from the API, because the question is how many times a shared value was *written*.

**Inference: this is the defect `expr_key` had.** `encode_capability` embedded each capability's four authority identities in full, and capabilities registered against one registry overwhelmingly name the same ones. The governed profile has five capabilities and one registry snapshot.

## Change

Each distinct authority identity is now written once into a count-prefixed pool in ascending byte order, and capabilities refer to it by fixed-width position — the shape `compute_graph_identity` already uses, and the one that took kernel-program identity from 13,309 bytes to 3,118.

`REGISTRY_IDENTITY_TAG` steps to `v2`: the subject is unchanged and only its spelling moved, which is exactly why a `v1` identity must miss.

**Injectivity.** The pool is complete and count-prefixed before any capability refers to it, and it is ordered by content rather than by registration, so a fixed-width position determines its referent exactly as an inline copy did. Two registries differing in any sub-identity differ in the pool; two differing only in which capability names which differ in the positions.

## Measurement — after

| | before | after |
| --- | --- | --- |
| lowering registry identity | 15,583 B | **11,207 B** |
| canonical explain subject | 20,194 B | **15,818 B** (−21.7%) |

## What the byte accounting had to become

The registry checks a running identity-byte budget as each capability is added. It can no longer be exact: whether a value ends up shared is not known until the registry is closed. It is now a **conservative upper bound** — each capability's own identities counted in full, plus the fixed-width positions it always writes — and the `debug_assert` became `<=` rather than `==`. The direction is the one that matters: a registry admitted by the budget always encodes within it.

That assertion earned its keep immediately. The first version of the bound omitted the pool's framing, and for single-capability registries the interned form is *larger* than the un-interned one by the pool count plus four positions. Sixteen tests failed on it rather than silently encoding past a budget that no longer covered the encoding.

## Golden moved

`explain.rs` — `request=bb089e78b94e892c` → `request=107be925f836ea4e`. Regenerate by running `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and reading the `left` value the assertion reports. The rationale is recorded at the site.

## Remaining

The subject is still 15,818 bytes, and the registry identity is still 71% of it. The pool now holds each *distinct* authority identity once, so what remains is genuine content rather than restatement — shrinking it further means shrinking the authority identities themselves, which is a different subject and a different ticket.
