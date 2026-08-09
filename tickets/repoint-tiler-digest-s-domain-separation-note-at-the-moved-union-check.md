---
id: repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check
title: Repoint tiler-digest's domain-separation note at the moved union check
status: done
priority: p2
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [implementation/digest]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
The crate header of `crates/tiler-digest/src/lib.rs` names the union no-prefix authority under its old path and its old population. Both moved in `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check`.

**Fact — verified 2026-08-08 by reading `crates/tiler-digest/src/lib.rs` at base `6eabf97e`.** The header ends with, verbatim:

> `tiler_artifact::proof::tests::no_governed_domain_of_either_container_prefixes_another`
> is the authority for the envelope's and sidecar's eight, and
> `docs/artifact-abi.md` records the obligation normatively.

Two claims in it are now false:

1. **The path.** The test moved to `tiler_artifact::domains::no_governed_domain_of_this_crate_prefixes_another`. Nothing named `proof::tests::no_governed_domain_of_either_container_prefixes_another` exists any more, so this is a dangling symbol reference rather than a stale-but-resolvable one.
2. **The count and its framing.** "the envelope's and sidecar's eight" is wrong twice over: the population is **eighteen**, and it is no longer scoped to the two containers — it includes the artifact program's seven identity and key domains. The envelope alone admits seven, not four.

The surrounding paragraphs — that a domain belongs to the authority that decides what it names, that this crate deliberately knows none of them, and that each authority owes the check over its own set plus the argument that its set cannot prefix another's — are all still correct and should be preserved.

## Why this is a separate ticket

Scope. The originating ticket holds `implementation/artifact` and `contracts/artifacts`; `crates/tiler-digest/**` is `implementation/digest`, which it does not hold. The reference was found and characterized there rather than edited.

## Closes when

The header names the current test path and the current population, and the reasoning around it is unchanged. `docs/artifact-abi.md` under "The governed digest" is the authority to reconcile against.

## Outcome and later correction — 2026-08-09

Commit `5d4a30eb` repointed the crate header from the deleted
`proof::tests` symbol to
`tiler_artifact::domains::no_governed_domain_of_this_crate_prefixes_another`
and expanded the described artifact-owned set to include the envelope, sidecar,
and artifact-program identity/key domains. Commit `95e0bb03` closed the bounded
repair and filed the two neighbouring cross-crate defects it discovered.

A later source-first audit found that carrying the then-current cardinal and a
universal claim about the IR population merely rebuilt the same drift class.
Commit `8fda6b34` removed both: the header now names
`GovernedDomain` as the type-sized population, states no count, and limits the
test authority to the artifact crate's own admitted set. The surrounding
one-algorithm and per-authority domain-separation reasoning remains unchanged.
