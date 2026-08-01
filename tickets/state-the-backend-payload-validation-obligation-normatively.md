---
id: state-the-backend-payload-validation-obligation-normatively
title: State the backend payload-validation obligation normatively
status: in-progress
priority: p2
dependencies: []
related: [route-a-custom-backend-through-an-independently-selected-adapter, generalize-payload-provenance-beyond-the-apple-shape]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifacts, backend-providers, runtime]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785627953
---
## User-visible outcome

`docs/artifact-abi.md` states the backend payload-validation obligation as normative contract text rather than deferring it, so a backend author reads what is owed and when instead of inferring it from a landed implementation.

## Why this exists

**Fact — the contract defers the statement to a condition that has since been met.** `docs/artifact-abi.md:101` records ADR 0090's acceptance and then defers: "The record was accepted on 2026-07-31: a backend validates its own payload from bytes while the preflight is still held, and the normative statement of that obligation enters this contract **when the first composed backend implements it**."

**Fact — the first composed backend implemented it.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 records the landing in its accepted status paragraph: "`implementation_status` moved to `partial` on 2026-07-31 when `route-a-custom-backend-through-an-independently-selected-adapter` landed the first implementation of item 4's adapter consequences and **item 8's payload-validation schedule** as the reviewed-draft `tiler_runtime::adapter` seam." That ticket is `done` and carried an out-of-crate non-Metal adapter fixture.

**Inference — the deferral's own trigger fired, and nothing was watching it.** A contract sentence conditioned on a future landing is only as good as the sweep that revisits it, and AGENTS.md notes the asymmetry directly: a disclosure required while a decision is proposed becomes wrong once it is accepted, and nothing checks either direction. This is the same class, one step later — the condition was met and the deferral stands.

## Work

Write the obligation into `docs/artifact-abi.md` as normative text: **a backend validates its own payload from bytes, and does so while the preflight is still held — before the first live-device question.** State why, because the why is what makes it unnegotiable rather than stylistic: a payload's `code` bytes are opaque to every check the artifact layer performs, and [ADR 0051](../docs/decisions/0051-make-runtime-routing-commit-one-way.md)'s one-way commit leaves an unvalidated payload nowhere to fail safely afterwards. Place it beside the monotonic validation stages the contract already names at `docs/artifact-abi.md:95-99`, of which backend-payload validity is one.

## Boundaries

- **Describe the obligation, not the API.** `tiler_runtime::adapter` is a reviewed experimental draft; ADR 0090:19 says every concrete public surface still comes to Tom under ADR 0075. Writing the draft's current shape into a contract would convert a reviewed draft into an accepted boundary by prose — which nothing authorizes and nobody reviewed.
- Do not restate the accepted decision or re-derive it; ADR 0090 is the record and this contract states its consequence for the ABI layer.
- Scope is `contracts/artifacts` only. No code, no ADR edit.

## Closes when

`docs/artifact-abi.md` states the obligation normatively with its derivation, the deferral sentence at `:101` is replaced rather than left beside its own discharge, and a reader can tell from the contract alone what a backend owes and at which validation stage — without reading `tiler-runtime` or naming any draft type.
