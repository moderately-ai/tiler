---
id: cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check
title: Cover the fifth envelope digest domain in the union no-prefix check
status: done
priority: p1
dependencies: []
related: [date-the-artifact-abis-metal-golden-enumeration-to-its-step]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## A hashed domain that a normative obligation requires be checked, and is not

`docs/artifact-abi.md` is normative and explicit: *"**A new governed domain in either container must be added to the union check**, and adding one to the envelope-local check alone does not discharge this obligation."*

**`PAYLOAD_IDENTITY_DOMAIN` discharges neither half.** Coordinator-verified — `grep -rn PAYLOAD_IDENTITY_DOMAIN crates/tiler-artifact/src` returns exactly two lines: its declaration in `program/codec/payload.rs` (`b"tiler.artifact-envelope.payload-identity.v1\0"`) and its single hash site in the same file. It appears in **neither** the union check nor the envelope-local one, and `proof/tests.rs`'s union test hard-codes `let domains: [&[u8]; 8]`.

> **Worker correction, 2026-08-08 at base `6eabf97e`.** Everything above verified. **The count below was wrong, and understated the gap.** The envelope admits **seven** governed domains and the crate **eighteen**, so the union check covered **8 of 18**, not 8 of 9. Corrected text follows; the original claim is retained here because the repair is the point.
>
> ~~So the envelope hashes under **five** digest domains rather than four, the crate under **nine** rather than eight, and **8 of 9 are checked**.~~
>
> `PAYLOAD_IDENTITY_DOMAIN` was not the only omission. `grep -rn 'b"tiler\.' crates/tiler-artifact/src` returns three more admitted domains absent from both checks — `PAYLOAD_METADATA_DOMAIN` declared two lines above it in the same file, `MANIFEST_DOMAIN` in `program/codec/encode.rs`, and, in a family the ticket did not consider at all, the seven `tiler.artifact-program.*` and `tiler.artifact.*` identity and key domains in `program/model.rs`, `program/realization.rs`, and `program/requirement.rs`.
>
> **Framing tags are governed, and the existing check already agreed.** The union list included the sidecar's `MANIFEST_DOMAIN` and `IDENTITY_DOMAIN`, which are framing tags rather than digest arguments, while excluding the envelope's structurally identical `MANIFEST_DOMAIN` — an asymmetry with no stated reason. The obligation covers them because such a tag is the leading run of a canonical byte sequence that is digested, compared, or recognised. The sharpest case is recognition rather than hashing: `model.rs`'s `RecordedArtifactProgramIdentity::from_bytes` admits bytes by `starts_with(ARTIFACT_DOMAIN)`, so a domain prefixing that separator would let another subject's bytes pass as an artifact identity with no digest involved.
>
> True population: envelope **7**, proof sidecar **4**, program identity **7**, total **18**.

## The property holds today; the check does not establish it

The auditor compared the strings by hand: `…payload-identity…` prefixes nothing in the set and nothing prefixes it. **So this is not a live collision** — it is an unchecked one, which is exactly what the normative sentence exists to prevent. A future domain could collide with `PAYLOAD_IDENTITY_DOMAIN` and no test would notice.

**Verify that by hand yourself before relying on it**, and then make the check say it rather than a reader having to.

> **Worker confirmation, 2026-08-08.** Verified over all **18**, not just the 9 the ticket assumed, and the conclusion holds: no live collision. The margin is thinner than the prose suggests. `tiler.artifact-envelope.manifest.v1\0` and `tiler.artifact-envelope.manifest-digest.v1\0` are separated only by the `.`/`-` at one position and by the NUL terminators; without terminators, `…manifest` would prefix `…manifest-digest.v1`. The union check now asserts the terminator as well as the prefix relation for exactly that reason.

## Requirements

- **Add it to the union check** — the authority the document names — and to the envelope-local check, since the document treats those as two halves rather than alternatives.
- **The hard-coded `8` is the defect's own cause.** A count literal beside a list is what let a ninth domain be added without anything failing. Derive the population, or floor and assert it, so the next domain is a build error or a red test rather than a silent omission. This repository has hit that shape repeatedly.
- **Correct the counts in the prose too.** `docs/artifact-abi.md` currently contradicts itself: one passage says "these four and the envelope's three are the **seven** the union no-prefix obligation covers", another in the same document says "the crate's **eight** governed domains", and a third says "four domain separators". Reconcile all of them against the true population.

  **The count sites are five, not three — verified 2026-08-07 at base `7c371155` by `verify-and-file-the-remaining-maturity-audit-leads`.** Beside the three named above, the "four domain separators" claim is made twice more, in passages about the *public surface* rather than about the digest: once in the codec-promotion Fact and once in the wire-form Fact. Both are the same falsified count reached in a different context, and the `Closes when` phrase "three inconsistent counts" would let them survive a sweep that reads it literally. Reconcile all five.

  > **Citation repair, 2026-08-08.** This bullet originally pinned those two sites by quoting their then-current text, `"including the four digest domain separators"` and `"The framing magic, the four domain separators, the schema versions"`. Both anchors were repaired by this ticket's own work and no longer occur, so quoting them here failed `make citations`. They now read `docs/artifact-abi.md "including all seven of the envelope's domain separators"` and `docs/artifact-abi.md "The framing magic, the seven domain separators"`. This is the expected cost of citing a passage a ticket exists to change, and the anchors failed loudly rather than resolving to the wrong place.

  > **Worker correction, 2026-08-08 at base `6eabf97e`.** All five verified present and all five corrected. **Five is still an undercount**: the sweep also has to reach three *ordinal* claims that name no cardinal and so survive a search for "four" as a count — `"a fourth governed domain"` in the schema-`15.0` Fact, `"the fourth envelope domain"` in the identity-digest pre-image Fact, and the section heading `"The sidecar's four governed digest domains"`, whose four is right but whose "digest" is wrong now that two of the four are named as framing tags. Eight sites, and the two ordinals were repaired by naming the domain instead of its position, because an ordinal into a set that grows is a count that goes stale without ever looking like one.
  >
  > Two further sites are **out of scope and filed rather than edited**: `crates/tiler-digest/src/lib.rs`'s crate header names the moved test path and "the envelope's and sidecar's eight" (`implementation/digest` — `repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check`), and `docs/decisions/0103-…`'s consequence states the obligation is "over the crate's **eight** domains … over the union of both containers" (`contracts/decisions` — `decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim`).
- **Watch it fail**: add a probe domain that prefixes an existing one and confirm the union check reddens by name. Perturb the subject, not the assertion.

## Closes when

Every domain the crate hashes appears in the union check; the population is derived or floored rather than a literal; all five of `docs/artifact-abi.md`'s inconsistent counts agree with the code; and the check has been watched failing on a planted prefix collision.

## Scheduling note — one file, two live claims

`date-the-artifact-abis-metal-golden-enumeration-to-its-step` also holds `contracts/artifacts` and also edits `docs/artifact-abi.md`, for an unrelated subject (the Metal golden corpus named at the `tiler.schedule.v5` step). The subjects do not overlap but the scope and the file do, so the two must be sequenced rather than run concurrently.
