---
id: decide-whether-the-citation-checker-should-reach-spike-records
title: Decide whether the citation checker should reach spike records
status: todo
priority: p2
dependencies: []
related: [reach-every-spike-record-from-the-experiment-catalog, state-the-spike-currency-convention-where-readers-look]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [gates, documentation, spikes, blind-spot]
---
## User-visible outcome

Either a rotted link inside a retained spike record fails `make citations`, or the repository states plainly that it does not — so nobody reads a green gate as covering a population it never scans.

## Why this exists

Found 2026-08-22 by `worker-tileprotocol`, **by perturbation rather than by reading**: it introduced a deliberately broken link under `spikes/` and `make citations` returned **exit 0, green**. It then resolved all sixteen links in its own documents by hand, because the gate could not.

**Fact — `spikes/**` is outside the checker's population, and the script says so.** `check-citations.sh` records its scanned set as `tickets/**`, `docs/**`, and the tracked markdown at the repository root. Verified by the coordinator at `97e7fef1`. This is a stated scope, not an accident — which is exactly why the decision below is a decision and not a bug fix.

**Why it matters more than it looks.** AGENTS.md's own description of this gate says it "resolves every local markdown link in an open ticket or a live document, so a catalog row or cross-reference that points at nothing fails the gate". A reader can take that as covering the evidence a document cites. It does not cover the spike records themselves — and the spike currency convention that landed recently tells readers to go and read a spike's own dated claim. If the links *inside* those records rot silently, the route the convention sends readers down is the one nothing guards.

**This is the class AGENTS.md names.** *"Verify that a check reaches its subject at all."* Several checks here could not: `cargo doc` cannot fail for a `#[cfg(test)]` module or an integration test under `tests/`; a `^(pub )?`-anchored grep cannot match `pub(crate)`. This is one more, found the way that section prescribes — by asking what it would take for the check to say *no*, and discovering the case was unreachable.

## The decision

Three options, and **the middle one may well be right**:

1. **Extend the checker to `spikes/**`.** Closes the hole. Cost: spike records cite exploratory material and third-party sources, and some deliberately point at paths that do not exist yet; a green gate would start depending on their hygiene. Weigh this against AGENTS.md's standing position that spikes must not silently become repository gates.
2. **Extend it to spike records only** — the `README.md` and `PROTOCOL-*.md` files a document or catalog links *to* — leaving harness sources and results alone. Narrower, and matches what the currency convention actually asks readers to follow.
3. **Record the exclusion explicitly** in `check-citations.sh`'s own stated declinations and in AGENTS.md's description of the gate, and leave the behaviour alone. Costs nothing and removes the false impression; closes no hole.

## Required work

- Re-audit the Fact at your base and report a verdict. **Reproduce the perturbation yourself** — break a link under `spikes/`, run `make citations`, and quote the exit status and output — rather than trusting this ticket.
- Size the population before choosing: how many links live under `spikes/`, how many resolve today, and how many are deliberate forward references. Say which unit you report.
- Choose by reading, state what each option admits and what it costs, and **name the option you rejected and why**.
- If you extend the checker, perturb it in both directions: a broken spike link must fail, and a spike record with only sound links must not. Quote both.
- If you choose option 3, the deliverable is the recorded exclusion in both places, and **say explicitly that there is no check to perturb** rather than leaving that obligation silently unmet.

## Non-goals

Adding `spikes/` to `Cargo.toml` members, to `make full`, or to any build gate — AGENTS.md forbids it and that is settled. Repairing individual spike links, which belongs to the catalog ticket. Changing the currency convention.

## Closes when

A broken link inside a retained spike record either fails the gate with its output quoted, or the exclusion is stated in `check-citations.sh` and in AGENTS.md's description of what `make citations` covers — and in either case the population was sized before the choice was made.
