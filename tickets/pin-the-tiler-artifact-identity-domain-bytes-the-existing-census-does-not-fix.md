---
id: pin-the-tiler-artifact-identity-domain-bytes-the-existing-census-does-not-fix
title: Pin the tiler-artifact identity-domain bytes the existing census does not fix
status: in-progress
priority: p1
dependencies: []
related: [pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate, pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [identity, tests, versioning]
claimed_from: todo
assignee: sol-artifact-domain-bytes
lease_expires_at: 1786383181
---
`crates/tiler-artifact/src/domains.rs` proves that every governed domain constant
is enumerated, every enum variant is classified into a container, and no two
current values are in a prefix relation. It does **not** independently pin the
bytes of any domain: [`GovernedDomain::bytes`](../crates/tiler-artifact/src/domains.rs)
returns the live constants themselves, so changing one constant changes both
the encoder and every value those tests inspect.

## Fact audit at `ee4fe66b`, 2026-08-09

The complete 551-line domain module and the complete crate root were read before
this ticket was filed.

**Fact — the population is complete and type-sized.** `GovernedDomain::ALL` is
sized by `variant_count::<GovernedDomain>()`; `bytes` and `container` are
wildcard-free; the const block accounts for all variants; and
`every_governed_domain_declared_in_the_source_is_enumerated` scans every
`_DOMAIN: &[u8]` declaration. The current population is 18: seven envelope,
four proof-sidecar, and seven artifact-program domains.

**Fact — those controls establish population and separation, not spelling.**
The no-prefix test calls `domain.bytes()`, the per-container test counts enum
members, and the source census compares declarations with the same values
returned by `bytes()`. None compares a live domain with an independently stated
expected byte string. A deliberate or accidental `v1` to `v0` edit can therefore
remain invisible to this module whenever no unrelated digest golden happens to
fold that domain.

**Fact — this is the remainder the two completed census tickets explicitly left
outside their crates.**
[`pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate`](pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate.md)
reported that the artifact module pins no value, and
[`pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach`](pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach.md)
called it a separate ticket. No such ticket existed before this one.

## What closes this

- Measure the package baseline first by reverting each of the 18 live domain
  spellings in isolation and recording which existing tests fail. Do not infer
  that all or none have incidental golden coverage.
- Add one independently stated exact-byte pin for every
  `GovernedDomain::ALL` member. Keep the population type-sized and the mapping
  wildcard-free so a new variant cannot compile without a spelling decision;
  do not introduce a second hand-written length.
- Make a legitimate domain step cost exactly the live declaration edit plus
  its one expected-byte edit. The failure must name the enum member, expected
  bytes, and observed bytes so the second edit is located rather than hunted.
- Revert each domain separately after the repair and quote the exact failure.
  Also widen the enum without updating the expected mapping and demonstrate the
  compile-time exhaustiveness failure. Restore every perturbation.
- Report the final 18-member census. Do not change any domain, encoder, schema,
  identity golden, public item, or artifact behavior in this ticket.

## Boundary

This is private test-only work under `implementation/artifact`. It does not
reopen the accepted no-prefix contract or the separate cross-crate prose repair,
and it does not absorb other crates' domain populations. A discovered live
spelling error or required identity step changes the purpose and is a stop for a
separate identity-authority ticket.

## Worker record at `03bfab6f`

### Fact audit

All three Facts above are verified at this exact base. The complete domain
module and crate root were read. `GovernedDomain` has 18 variants,
`GovernedDomain::ALL` has type `[Self; variant_count::<Self>()]`, `bytes` and
`container` are wildcard-free, and the source census sees 18 governed `&[u8]`
declarations. `cargo test -p tiler-artifact domains -- --nocapture` ran the
three original tests and all passed. The two related completed tickets both
name this exact-byte gap as outside their crate-local censuses, and this ticket's
file is absent from the parent of its creation commit `575ac8d8`.

### Baseline before the pin

The unmodified package baseline was 252 passed and 1 skipped. Each row below
changed only the live declaration to its immediately preceding version, ran
`cargo nextest run -p tiler-artifact --no-fail-fast`, and restored it before the
next row. Four have incidental coverage; fourteen remained green.

| isolated source revert | failures before this pin |
| --- | --- |
| `EnvelopeManifest` `v1` -> `v0` | **0 — 252 passed** |
| `EnvelopeManifestDigest` `v1` -> `v0` | **0 — 252 passed** |
| `EnvelopeSectionDigest` `v1` -> `v0` | 1 — `program::codec::tests::a_bf16_artifact_round_trips_and_its_carrier_enters_identity` (`left: 67`, `right: 68`) |
| `EnvelopeEnvelopeDigest` `v1` -> `v0` | **0 — 252 passed** |
| `EnvelopeIdentityDigest` `v1` -> `v0` | **0 — 252 passed** |
| `EnvelopePayloadMetadata` `v1` -> `v0` | **0 — 252 passed** |
| `EnvelopePayloadIdentity` `v1` -> `v0` | **0 — 252 passed** |
| `SidecarManifest` `v1` -> `v0` | **0 — 252 passed** |
| `SidecarManifestDigest` `v1` -> `v0` | **0 — 252 passed** |
| `SidecarPayloadDigest` `v1` -> `v0` | **0 — 252 passed** |
| `SidecarIdentity` `v1` -> `v0` | **0 — 252 passed** |
| `ProgramArtifact` `v16` -> `v15` | 1 — `program::codec::tests::an_encoded_envelope_round_trips_to_an_equal_model` (its `starts_with(b"tiler.artifact-program.v16\0")` assertion) |
| `ProgramStageKey` `v3` -> `v2` | 1 — `program::tests::each_artifact_stage_key_generation_is_separated_from_the_last` (`assertion failed: !current.starts_with(V2)`) |
| `ProgramPayloadKey` `v1` -> `v0` | **0 — 252 passed** |
| `ProgramProviderKey` `v2` -> `v1` | 1 — the bf16 carrier-difference test above (`left: 67`, `right: 68`) |
| `ProgramDeferredKey` `v2` -> `v1` | **0 — 252 passed** |
| `ProgramDeliveredRealization` `v2` -> `v1` | **0 — 252 passed** |
| `ProgramRouteRequirement` `v1` -> `v0` | **0 — 252 passed** |

### What landed and independent failure evidence

`GovernedDomain::pinned_bytes` is a private, test-only, wildcard-free match
restating the exact bytes of all 18 members. The new
`every_governed_domain_has_its_exact_pinned_bytes` test ranges over the existing
type-sized `ALL`; it introduces no second length and reports the member,
expected bytes, observed bytes, and the location of the required second edit.

Each source revert above was repeated alone after the pin. All 18 failed the new
test with exit 101. The exact member-specific part of each diagnostic was:

| member | exact failure fields |
| --- | --- |
| `EnvelopeManifest` | `EnvelopeManifest's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.manifest.v1\0"`; `observed bytes: "tiler.artifact-envelope.manifest.v0\0"` |
| `EnvelopeManifestDigest` | `EnvelopeManifestDigest's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.manifest-digest.v1\0"`; `observed bytes: "tiler.artifact-envelope.manifest-digest.v0\0"` |
| `EnvelopeSectionDigest` | `EnvelopeSectionDigest's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.section-digest.v1\0"`; `observed bytes: "tiler.artifact-envelope.section-digest.v0\0"` |
| `EnvelopeEnvelopeDigest` | `EnvelopeEnvelopeDigest's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.envelope-digest.v1\0"`; `observed bytes: "tiler.artifact-envelope.envelope-digest.v0\0"` |
| `EnvelopeIdentityDigest` | `EnvelopeIdentityDigest's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.identity-digest.v1\0"`; `observed bytes: "tiler.artifact-envelope.identity-digest.v0\0"` |
| `EnvelopePayloadMetadata` | `EnvelopePayloadMetadata's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.payload-metadata.v1\0"`; `observed bytes: "tiler.artifact-envelope.payload-metadata.v0\0"` |
| `EnvelopePayloadIdentity` | `EnvelopePayloadIdentity's exact domain bytes moved:`; `expected bytes: "tiler.artifact-envelope.payload-identity.v1\0"`; `observed bytes: "tiler.artifact-envelope.payload-identity.v0\0"` |
| `SidecarManifest` | `SidecarManifest's exact domain bytes moved:`; `expected bytes: "tiler.proof-sidecar.manifest.v1\0"`; `observed bytes: "tiler.proof-sidecar.manifest.v0\0"` |
| `SidecarManifestDigest` | `SidecarManifestDigest's exact domain bytes moved:`; `expected bytes: "tiler.proof-sidecar.manifest-digest.v1\0"`; `observed bytes: "tiler.proof-sidecar.manifest-digest.v0\0"` |
| `SidecarPayloadDigest` | `SidecarPayloadDigest's exact domain bytes moved:`; `expected bytes: "tiler.proof-sidecar.payload-digest.v1\0"`; `observed bytes: "tiler.proof-sidecar.payload-digest.v0\0"` |
| `SidecarIdentity` | `SidecarIdentity's exact domain bytes moved:`; `expected bytes: "tiler.proof-sidecar.identity.v1\0"`; `observed bytes: "tiler.proof-sidecar.identity.v0\0"` |
| `ProgramArtifact` | `ProgramArtifact's exact domain bytes moved:`; `expected bytes: "tiler.artifact-program.v16\0"`; `observed bytes: "tiler.artifact-program.v15\0"` |
| `ProgramStageKey` | `ProgramStageKey's exact domain bytes moved:`; `expected bytes: "tiler.artifact-program.stage.v3\0"`; `observed bytes: "tiler.artifact-program.stage.v2\0"` |
| `ProgramPayloadKey` | `ProgramPayloadKey's exact domain bytes moved:`; `expected bytes: "tiler.artifact-program.payload.v1\0"`; `observed bytes: "tiler.artifact-program.payload.v0\0"` |
| `ProgramProviderKey` | `ProgramProviderKey's exact domain bytes moved:`; `expected bytes: "tiler.artifact-program.provider.v2\0"`; `observed bytes: "tiler.artifact-program.provider.v1\0"` |
| `ProgramDeferredKey` | `ProgramDeferredKey's exact domain bytes moved:`; `expected bytes: "tiler.artifact-program.deferred.v2\0"`; `observed bytes: "tiler.artifact-program.deferred.v1\0"` |
| `ProgramDeliveredRealization` | `ProgramDeliveredRealization's exact domain bytes moved:`; `expected bytes: "tiler.artifact-program.delivered-realization.v2\0"`; `observed bytes: "tiler.artifact-program.delivered-realization.v1\0"` |
| `ProgramRouteRequirement` | `ProgramRouteRequirement's exact domain bytes moved:`; `expected bytes: "tiler.artifact.route-requirement.v1\0"`; `observed bytes: "tiler.artifact.route-requirement.v0\0"` |

Every diagnostic ended with this exact locator:
``A deliberate domain step costs the live declaration edit plus this member's one
`GovernedDomain::pinned_bytes` arm edit.`` Moving both the live
`EnvelopeManifest` declaration and its one pin arm `v1` -> `v0`, then running
the four domain tests, passed all four. That demonstrates the legitimate-step
cost is exactly those two edits.

Finally, adding a `Probe` variant to the enum alone made `cargo test -p
tiler-artifact --no-run` fail with `expected an array with a size of 19, found
one with a size of 18` at `GovernedDomain::ALL`, and with `error[E0004]:
non-exhaustive patterns: GovernedDomain::Probe not covered` independently at
each of `bytes`, `pinned_bytes`, and `container`. The `pinned_bytes` error points
at its match and proposes the missing `GovernedDomain::Probe` arm. The probe was
then restored.

The supported population remains exactly the 18 governed domains of this crate:
seven envelope, four proof-sidecar, and seven artifact-program domains. This
test does not claim coverage of other crates or of this crate's classified
non-domain literals.
