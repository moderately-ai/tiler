---
id: pin-the-tiler-artifact-identity-domain-bytes-the-existing-census-does-not-fix
title: Pin the tiler-artifact identity-domain bytes the existing census does not fix
status: todo
priority: p1
dependencies: []
related: [pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate, pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [identity, tests, versioning]
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
