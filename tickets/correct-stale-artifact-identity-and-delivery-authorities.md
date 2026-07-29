---
id: correct-stale-artifact-identity-and-delivery-authorities
title: Correct stale artifact identity and delivery authorities
status: todo
priority: p1
dependencies: []
related: [prototype-kernel-program-ir, prototype-neutral-artifact-codec, prototype-metal-runtime-proof, correct-the-stale-post-vertical-implementation-status]
scopes: [contracts/artifacts, contracts/decisions, research/artifacts, research/program-planning, implementation/artifact, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifact, identity, correctness]
---
## User-visible outcome

Artifact contracts, accepted ADRs, adopted research, and public module documentation distinguish historical identity revisions from the current source-derived ledger and accurately state that the reviewed codec, real Metal payload producer, and bounded runtime consumer now exist.

## Why this is a correctness ticket

- **Fact:** `docs/artifact-abi.md` still calls the compiler input private, says no sidecar producer or consumer ships, and says the implemented profile carries no backend payload even though the public compiler, Metal AOT producer, carried payload, and runtime proof are done.
- **Fact:** accepted ADRs 0072 and 0074, adopted artifact/program-planning research, and `tiler-ir`/`tiler-artifact` module documentation retain current-tense statements about earlier schedule, kernel-program, artifact, framing, or facade states.
- **Fact:** the current values must be derived separately from their minting constants and codec schemas; manifest schema, artifact identity, canonical encoding, and component schemas are different subjects.
- **Inference:** stale current-tense versions and delivery claims are cache-identity and runtime-boundary hazards, while blindly replacing every historical value would erase the evolution rationale.

## Implementation keys

- Read every edited source, contract, ADR, and research file in full. Derive identities from their construction constants and schema values from codec construction, then cross-check the current ledger in `docs/artifact-abi.md`.
- Audit the stale passages at `docs/artifact-abi.md` lines 17, 396, and 899; `docs/decisions/0072-separate-semantic-meaning-from-provider-provenance.md`; `docs/decisions/0074-use-explicit-public-api-conventions.md`; `docs/research/artifacts/target-neutral-artifact-envelope.md`; `docs/research/program-planning/abi-expression-ownership.md`; and the `tiler-ir` and `tiler-artifact` program module documentation. Line numbers are discovery hints, not authorities.
- Preserve version history and accepted rationale explicitly. Change only statements presented as current, and keep static codec validation separate from live-device/runtime evidence.
- Treat public module documentation as part of the public module boundary and present its corrected boundary to Tom before acceptance.
- Prove every new consistency check can fail, then run targeted `tiler-ir` and `tiler-artifact` tests and Clippy, local documentation checks, `tkt lint`, and one batch `make full`.

## Closes when

Every current identity and schema value is source-derived and correctly scoped to its subject; historical revisions remain legible; compiler, producer, codec, and bounded consumer claims agree with construction sites; no codec-only claim is promoted to portable runtime support; Tom has reviewed the public module documentation; and the targeted and full gates pass.

## Graph maintenance

- Link each corrected statement to the ticket that made it stale or to the identity owner whose version it names.
- Split any newly found stale authority outside the declared scopes rather than leaving an untracked cleanup note.
- Close this ticket when all named authorities agree; compatibility expansion remains separate implementation and research work.
