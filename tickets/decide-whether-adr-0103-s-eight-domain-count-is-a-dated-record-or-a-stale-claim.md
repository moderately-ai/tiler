---
id: decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim
title: Decide whether ADR 0103's eight-domain count is a dated record or a stale claim
status: in-progress
priority: p3
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786172922
---
`docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md` states, in its consequences:

> **A fourth governed envelope digest domain is admitted.** `tiler.artifact-envelope.identity-digest.v1` joins `manifest-digest`, `section-digest`, and `envelope-digest`. It is separate from `manifest-digest` because that domain covers the manifest bytes this digest is written into. The no-prefix obligation is now over the crate's **eight** domains and is checked over the union of both containers, as the ABI contract already requires normatively.

(In the ADR, "the ABI contract" links to `docs/artifact-abi.md`. The link is flattened to plain text in this quotation because its `../` target resolves relative to `docs/decisions/`, not to `tickets/`.)

**Fact — verified 2026-08-08 at base `6eabf97e`.** The final sentence no longer describes the repository. `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check` established the true population as **eighteen** — the envelope's seven, the sidecar's four, and the artifact program's seven — and the check is no longer "over the union of both containers" but over every domain the crate admits. The count was also never eight at the time: the envelope's manifest framing tag and both payload domains were already admitted and simply uncounted.

**The first three sentences are correct and dated**, and identity-digest genuinely was the fourth *digest argument*.

## The actual question

An ADR is a dated record of a decision, not a live description of the tree, so the default answer may be to leave it and let `docs/artifact-abi.md` carry current state. But this sentence reads in the present tense and cites the contract as agreeing with it, which it now does not. The options:

1. **Leave it.** Consistent with treating an ADR as a point-in-time record. Costs a reader who reaches 0103 first a false present-tense claim.
2. **Append a dated correction** noting the population moved and pointing at the contract, preserving the original text.
3. **Edit the sentence.** Cheapest to read, but rewrites the record of what was decided.

Option 2 matches how this repository has handled superseded ADR language elsewhere and is the recommendation, but the choice is a documentation-convention call rather than a correctness one.

## Why this is a separate ticket

Scope. `docs/decisions/[0-9]*.md` is `contracts/decisions`, which the originating ticket does not hold. AGENTS.md also requires that a ticket unable to edit `docs/decisions/` hand the change over rather than fork it during transfer.

## Closes when

The convention question is answered and 0103 reflects it.

## Worker audit — per-Fact verdicts, 2026-08-08 at base `97282def`

Every source below was read in full at this base, not at `6eabf97e`.

1. **The quoted sentence — verified verbatim, but *mislocated*.** It is decision item 3 of the **Decision** section, not a consequence. The framing matters to the question this ticket asks: a consequence is a record of what followed, while a decision item is the record of what was chosen, and the ticket's own option 3 ("rewrites the record of what was decided") is only in play because the sentence sits where it does.
2. **"The true population is eighteen — the envelope's seven, the sidecar's four, and the artifact program's seven" — verified.** `crates/tiler-artifact/src/domains.rs` enumerates `GovernedDomain` with `ALL` sized by `core::mem::variant_count`, and `DomainContainer::ENVELOPE = 7`, `PROOF_SIDECAR = 4`, `PROGRAM_IDENTITY = 7`, asserted against `variant_count` in a `const` block.
3. **"The check is no longer over the union of both containers but over every domain the crate admits" — verified.** `no_governed_domain_of_this_crate_prefixes_another` iterates `GovernedDomain::ALL`, which spans all three containers. `cargo nextest run -p tiler-artifact domains::` runs three tests and passes at this base.
4. **"The count was also never eight at the time" — verified, and it understates the gap by an order.** `git grep -h -E "const [A-Z_]*_DOMAIN: &\[u8\]" 09d1666a -- crates/tiler-artifact/src | wc -l` returns **18** at the very commit that landed ADR 0103. The ticket names three uncounted domains; the artifact program's seven were uncounted as well, so the sentence was wrong by ten rather than by three.
5. **"The first three sentences are correct and dated, and identity-digest genuinely was the fourth *digest argument*" — FALSE, and this is the Fact that most changes the answer.** `tiler.artifact-envelope.payload-identity.v1` was declared and hashed on 2026-07-24, thirteen days before this ADR: `git show 09d1666a:crates/tiler-artifact/src/program/codec/payload.rs | grep -n 'digest(PAYLOAD_IDENTITY_DOMAIN'` returns `:456`. `identity-digest` was therefore the envelope's **fifth** digest domain, and the sibling ticket's own title says so. So sentence 1 of item 3 was false at its own date too, and *no* sentence in the item is both count-bearing and correct-as-dated.
6. **Newly found, same pattern, not in the ticket.** The alternatives-considered entry "Frame the digest behind a length prefix" claims the change would be "inconsistent with the three existing digest sites, none of which frames". There were four, and the fourth frames: `encode.rs` writes the payload descriptor's digest as `push_slice(bytes, payload.digest.as_bytes())`, and `tiler_ir::identity::push_slice` reserves `8 + value.len()` and writes an eight-byte length prefix. Withdrawn in the same commit.
7. **Two residual sites are out of scope and filed rather than edited** as [`reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section`](reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section.md): `docs/artifact-abi.md`'s `"Hashing occurs at exactly four sites"` (`contracts/artifacts`), which its own governed-digest section contradicts, and `encode.rs`'s `"It is a fourth domain rather than a reuse"` (`implementation/artifact`).

## Outcome — none of the three options as stated, and the rule that decides which applies

**The three options this ticket offers are not the choice the repository actually makes**, because all three assume the number was true once. Neither of ADR 0103's counts ever was, and the corpus treats a never-true claim differently from a stale one. What landed is **option 3 restricted to the false tokens, plus option 2's dated note carrying the retired wording** — the shape [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s context correction, [ADR 0074](../docs/decisions/0074-use-explicit-public-api-conventions.md)'s two, and [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md)'s item 6 all use. Option 1 is refused outright.

Landed in `docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md`:

- Item 3's "A **fourth** governed envelope digest domain is admitted" → "A governed envelope digest domain of its own is admitted", and its three-domain list completed with `payload-identity`. The ordinal is **dropped rather than advanced to fifth**, per the sibling ticket's own repair pattern.
- Item 3's "the crate's **eight** domains" → "every governed domain the crate admits". Its next clause, "checked over the union of both containers, as the ABI contract already requires normatively", is **retained unedited** — it was accurate on 2026-08-06 — and dated beside.
- The alternatives entry's "inconsistent with the three existing digest sites, none of which frames" → "inconsistent with the header's manifest digest and each section descriptor's content digest, neither of which frames".
- Two dated notes, after decision item 5 and after the alternatives, quoting every retired wording and stating why each was substituted rather than dated beside.

### The rule, for the next person meeting a present-tense number in an accepted record

**First ask whether it was ever true, because that — not the tense — selects the treatment.** The corpus is explicit and one-directional on this, and ADR 0106's context correction states both branches in one sentence: a snapshot accurate at acceptance is retained and takes a later fact alongside, "whereas this was never true at any commit, so leaving it standing would leave a reader auditing the admission to find the evidence self-contradicting at the point it is offered".

1. **True at acceptance, false now → retain the sentence, date the fact beside it.** The record's authority is over what was claimed at its date, so a reader must be able to see what it claimed. ADR 0088: "an admission record holds what was true when it was accepted, which is a reason to date a later fact beside it and **not** a reason to leave a present-tense claim standing after it became false." Same treatment at ADR 0075 ("the surrounding Fact is a snapshot of what the tree showed"), ADR 0076, ADR 0078 ("the rule it states is what generalizes and the inventory is what dated"), ADR 0079 ("dated rather than rewritten"), ADR 0090, ADR 0104, ADR 0106's status paragraph.
2. **False when written → substitute, and quote the retired wording in the note.** ADR 0106's context correction, ADR 0074's two 2026-08-08 corrections. A never-true claim is a *withdrawal*, not a supersession, and the note must say which — the two tell a reader different things about how far to trust the record.
3. **An instruction that would now be executed wrongly → substitute even if it was true.** ADR 0092 item 6, where a restatement instruction whose target had gained a qualifier would have landed the wrong amendment. A claim misinforms a reader who can check it; an instruction misdirects a worker who will act on it, and the second cannot wait for a note.
4. **Substitute the false token, not the sentence.** One sentence can hold both kinds at once, and ADR 0103's last one does: `eight` was never true and `the union of both containers` was true and dated. Do not label the whole sentence with the verdict the first clause earns.
5. **Never advance an ordinal — remove it.** "Fourth" becomes "of its own", not "fifth". An ordinal into a set that can grow is a count that goes stale without ever looking like one, and re-pointing it just resets the clock.
6. **Check whether the decision rests on the number before touching anything.** Ask whether the argument still runs with the number deleted. Here it does, at both sites, which is what made this a documentation repair rather than a decision to re-open. **A number an argument genuinely rests on is not settled by this rule** — that is a reopened decision.
7. **A retired quotation stays greppable, so disarm it where you quote it.** Both notes state that a grep finding `eight` or `fourth` in ADR 0103 lands inside the note. AGENTS.md carries the general hazard; the local instance is the correction author's to defuse, and no closing condition should ever demand that such a grep come back empty.
8. **Find one and check the siblings.** Auditing the single sentence this ticket named surfaced a second miscount of the same population in the alternatives, and two more outside this scope.

**Nothing about what ADR 0103 decides moved**, and `decision_status`, `implementation_status`, and every catalog row are untouched. The separation the corrected sentence asserts still holds: `cargo nextest run -p tiler-artifact domains::` passes at this base, checking all 153 pairs of the eighteen and finding no prefix relation.
