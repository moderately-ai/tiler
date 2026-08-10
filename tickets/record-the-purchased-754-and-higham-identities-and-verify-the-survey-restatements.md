---
id: record-the-purchased-754-and-higham-identities-and-verify-the-survey-restatements
title: Record the purchased IEEE 754 and Higham identities and verify the survey restatements
status: done
priority: p2
dependencies: []
related: []
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [sources, numerics, acquisition]
---

## What Tom pulled (2026-08-06)

- `/Users/tsanterre/Downloads/IEEE Standard 754-2019.pdf` — 3,805,037 bytes, sha256 `2fe5f245fa6fd027a64067e2d91d9000f51e9c61ad23fe1914d8cae41f2b0fb4`; pdfinfo title "IEEE Std 754™-2019 (Revision of IEEE Std 754-2008) IEEE Standard for Floating-Point Arithmetic", author "Microprocessor Standards Committee of the IEEE Computer Society".
- `/Users/tsanterre/Downloads/Higham_2002_Accuracy and Stability of Numerical Algorithms (1).pdf` — 9,461,221 bytes, sha256 `7f7b3e32f946563830e2999e614fd3cba75d3694817da5fb36bbbdb80c7a4a75`.

Neither may be vendored (IEEE and SIAM for-sale copyright; the manifest rows already state this). The realistic best outcome both rows named is metadata-only with a digest over a legitimately acquired copy, which these are.

## The work

1. `ieee-754-2019` manifest row: record the digest and the acquisition note (who pulled, date, the file's pdfinfo identity) per the metadata-only row format; the bytes stay outside the repository.
2. `higham-asna-2002`: `pending-acquisition` moves to `metadata-only` with the digest and note; update the verifier populations (pending 1 to 0) and watch the verifier fail on a perturbation before trusting the pass.
3. **The reading that is the acquisition's stated purpose:** read §3.4 (the `gamma_h` notation and composition rules) and §4.2 (summation error analysis: tree-height result, recursive/pairwise/blocked cases) in Tom's copy, and verify the certified-bounds record's three foundations against the proofs rather than the survey's restatements — specifically whether `acta-numerica-fp-2023`'s restatement dropped any side condition the record's worked online-softmax bound depends on. Record held/moved per claim in the array-API re-check shape, at the record's own citation sites. The row's own warning governs: this must NOT be closed by summarizing from the table of contents or a secondary description — the sections are read or the claim is not made.
4. If §4.2's treatment of the blocked and compensated cases adds anything the record's boundary statements should carry, record it as a dated note; if nothing moves, say the re-check happened and held.

## Closes when

Both rows carry their digests and notes, the verifier passes on the stepped population after being watched failing, and the certified-bounds record states the proof-level re-check's verdict per foundation.

## Outcome (2026-08-06)

**Both digests reproduce the fingerprints this ticket recorded, checked before anything was read.** `shasum -a 256` over the two staged files returns `2fe5f245fa6fd027a64067e2d91d9000f51e9c61ad23fe1914d8cae41f2b0fb4` at 3 805 037 bytes for the IEEE standard and `7f7b3e32f946563830e2999e614fd3cba75d3694817da5fb36bbbdb80c7a4a75` at 9 461 221 bytes for the monograph — both exact matches on digest and length. `pdfinfo` on the standard returns the title, author, and 84 pages this ticket states; the monograph is 710 PDF pages, which is exactly the cited xxx+680, giving a constant printed-to-PDF offset of 30 that was verified at both named sections. **Neither file's bytes entered the repository, and no route claim is made from either staged filename**: both were relayed without URLs, so each row states its canonical acquisition route as *where to go*, on the six-document precedent in the region-search source record.

**Both rows moved, and the manifest's `pending-acquisition` class is now empty.** `ieee-754-2019` stays `metadata-only` and gains a digest where it carried `-` and the sentence "no byte stream was ever retrieved". `higham-asna-2002` moves `pending-acquisition` → `metadata-only` with its digest. `verify-sources.sh`'s declared populations step from 19/1 to 20/0 metadata-only/pending; total and vendored are unchanged, because this change adds no rows and no bytes.

**Re-check verdict — 11 sub-claims across the 3 foundations; 9 hold as stated, 1 narrows, 1 is misattributed, and no derivation moves.** The population is counted so "nothing ran" cannot look green: three sub-claims for (2.5a), three for (4.7), five for (4.8).

| # | Foundation | Sub-claim | Verdict against the monograph |
| --- | --- | --- | --- |
| 1a | (2.5a) | `RN(a op b) = (a op b)(1+eps)`, `\|eps\| <= u`, `op` in `{+,-,*,/}` | **Holds** — §2.2's boxed STANDARD MODEL (2.4), verbatim in substance |
| 1b | (2.5a) | absent overflow | **Holds** — the model's validity range, with (2.8) in the chapter notes as the overflow/underflow-aware form |
| 1c | (2.5a) | subnormal result voids the relative bound; absolute bound instead | **Holds** — (2.8)'s additive `eta` term, `\|eta\| <= (1/2)*beta^(emin-t)`. The survey states this inline where §2.2 does not, so it is the *sharper* statement |
| 2a | (4.7) | `1 + theta_h` is a product of `h` factors `(1 + delta_i)` | **Holds, and Higham is stronger** — Lemma 3.1 admits `p_i = ±1`, covering quotients too |
| 2b | (4.7) | `\|theta_h\| <= hu/(1-hu) =: gamma_h` for `h < 1/u` | **Holds** — Lemma 3.1 exactly, with `nu < 1` |
| 2c | (4.7) | `gamma_h + gamma_k + gamma_h*gamma_k <= gamma_{h+k}` | **Holds** — Lemma 3.3's last rule. Its side condition was missing from the Tiler restatement and is **restored in place**; the derivation already discharged it as `2(V-1)*u < 1` |
| 3a | (4.8) | `n-1` additions, any parenthesizing, backward error `gamma_{n-1}`, ordering-independent | **Holds exactly** — §4.2's Algorithm 4.1 and its stated backward result. **This is the sub-claim the worked bound uses** |
| 3b | (4.8) | `h = n-1` recursive | **Holds** — §4.2 |
| 3c | (4.8) | `h = ceil(log2 n)` pairwise | **Holds** — §4.2's (4.6) gives `gamma_{log2 n}` under "assume for simplicity that `n = 2^r`"; the ceiling for general `n` is §4.1's stage count, one section earlier |
| 3d | (4.8) | general `gamma_h` at binary-tree height `h` | **Narrows** — **not stated in §4.2**; "tree" and "height" do not occur in Chapter 4 at all. §4.2 proves the two endpoints and supplies the counting argument the general form follows from, so the survey's `h` form is sound and is Higham's own mechanism, but it is the survey's synthesis and not a sentence at the cited section |
| 3e | (4.8) | `h = b + n/b - 2` blocked | **Corrected** — **not in the monograph at all**; Chapter 4 has no blocked summation under any name. The survey's own (4.9) attributes it to Castaldo, Whaley and Chronopoulos (2009) and to Blanchard, Higham and Mary (2020), a Higham paper eighteen years later. Struck from the Tiler restatement rather than re-cited, because nothing derives with it |

**The question the acquisition existed to answer is answered "no", in the direction opposite to the worry.** The survey did not drop a side condition the worked bound depends on. It is the *more* explicit document on two of the three foundations: it carries `h + k < u^-1` inline on the composition rule where Higham states `nu < 1` once as a standing convention over the whole notation, and it carries the subnormal exclusion inline where §2.2's model states no such condition. **No derivation moved and none would have**: the worked bound instantiates (4.8) at `h = V - 1`, precisely the case §4.2 proves, so both moved sub-claims are ones the records cite but do not derive with.

**One relaxation the monograph supplies, recorded as a dated note rather than acted on.** Under (2.8) with gradual underflow, `eta = 0` for `op` in `{+, -}` — Higham's Problem 2.19 after Hauser (1996), the same fact as the survey's footnote 32 — so floating-point addition is *exact* when its result is subnormal. The certified-bounds record's blanket no-subnormal assumption is therefore stronger than its summation terms need. It is still needed for the rescale multiplications and for `exp`, so the assumption stays as written and is recorded as conservative rather than necessary; **nothing was rewritten on it.**

**§4.3's compensated summation was read per work item 4, and it moves no boundary statement but sharpens one.** Kahan's method gives `|mu_i| <= 2u + O(nu^2)`, a constant independent of `n` for `nu <= 1`, bounded by three conditions worth carrying: the correction identity holds only in base 2, only for `|a| >= |b|`, and not under a no-guard-digit model; and a heavily cancelling sum still has no small relative error guarantee. The certified-bounds derivation is unaffected — its baseline is an uncompensated two-pass fold — but this is a concrete instance of the tree-fold record's matched-baseline rule, since a price quoted against a compensated baseline would be a price against a fold the caller did not run.

**What IEEE 754-2019 decided, beyond supplying an identity.** Table 3.5 defines binary16/32/64/128 at `p` = 11, 24, 53, 113 and Table 3.6 defines decimal32/64/128 at `p` = 7, 16, 34 digits, so ADR 0036's and ADR 0035's pinned formats exist with the parameters the taxonomy states. ADR 0035's two supporting claims hold at §3.5.2 (one value definition, two significand encodings) and §3.5.1 (cohort members are distinct representations of one number, distinguishable only by decimal-specific operations). **What it did *not* decide is the half a numerics reader needs**: `exp` is not a Clause 5 required operation. It sits in Table 9.1 of Clause 9.2, which is *recommended* — though a conforming 9.2 operation "shall return results correctly rounded". So `eps_exp = u` is exactly what a 754-conforming `exp` would give and no target owes one, which is the provenance of the certified-bounds record's worked instantiation and confirms its own open axis. **And it answered a standing acquisition request in a third record**: §6.2.3 recommends ("should") that a quiet NaN's payload survive a widen-then-narrow round trip unchanged except for canonicalization — a recommendation, over the round trip rather than the widening alone, so the conversion-family record's observation stands as written with a documented default now named beside it.

### Commands run

- `docs/research/numerics/sources/verify-sources.sh` → `OK: 91 records verified (71 vendored, 20 metadata-only, 0 pending-acquisition).`, exit 0.
- **Watched failing on six perturbations first**, in a scratch copy outside the repository, each aimed at a way this classification move could go wrong rather than at a vendored file: the row left at `pending-acquisition` (both moved counts, exit 1); the IEEE digest field filled with a non-digest (`digest field is neither '-' nor a SHA-256 digest`, exit 1); the monograph's row reclassified `vendored` (three failures, exit 1); the row deleted from the manifest (`manifest holds 90 records, expected 91` plus its count, exit 1); and **the SIAM PDF actually copied into `higham-asna-2002/`** both with the row declaring it (`metadata-only record must not retain local bytes` plus the orphan line, exit 1) and with the row untouched (the orphan line alone, exit 1) — the two shapes of the one mistake this row's verdict exists to prevent. Unperturbed runs before and after returned exit 0.
- `docs/research/region-search/sources/verify-sources.sh` → `OK: 30 records verified (10 vendored, 20 metadata-only, 0 pending-acquisition).`, exit 0 — unedited, run as a control that the shared verifier shape still passes.
- `shellcheck docs/research/numerics/sources/verify-sources.sh`, exit 0.
- Local link and anchor check over the five edited documents: 179 local links, 3 flagged, **all three pre-existing at base `59e93632`** and none introduced here — two are the deliberately destination-relative ADR links the conversion record documents in its own transfer note, and the third is the checker mishandling a heading that contains a markdown link.
- `tkt lint`, `git diff --check`, `git diff --name-only`, `tkt guard --base 59e93632`.
- No cargo. This change touches no gate input under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh`.

### Scope

`research/numerics` (`docs/research/numerics/**`, `spikes/numerics/**`) plus the ticket file under the declared shared `project/tickets`. No scope was added and none was required. `spikes/numerics/**` was in scope and untouched: the re-check is a reading of two documents that cannot be vendored, so its reproduction is the acquisition route plus the digest rather than a harness.

**Three stale sites are outside this ticket's scopes and are reported rather than edited**, each still asserting the monograph is unread: `docs/research/reference/permitted-divergence-oracle.md:32` ("The standing `higham-asna-2002` acquisition request ... is unchanged"), `docs/research/program-planning/flash-class-capability-set.md:32` ("The two standing acquisition requests in the neighbourhood"), and `tickets/reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md:164` ("Unread and behind a purchase wall"). None carries a claim that moves — all three say no conclusion is deferred behind the book, which was true and remains true — so each needs one dated clause, not a re-derivation.

**Correction — 2026-08-10.** Two of the three named research docs no longer assert unread. `docs/research/reference/permitted-divergence-oracle.md` now states the `higham-asna-2002` acquisition request closed on 2026-08-06 (monograph bought and read at proof level). `docs/research/program-planning/flash-class-capability-set.md` now states both neighbourhood acquisition requests have since closed on 2026-08-06. Only `tickets/reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md`'s historical named-gaps bullet still contains "Unread and behind a purchase wall"; that bullet sits inside a done decision packet whose non-dependence claim ("No claim in this packet is deferred behind it") still holds. This ticket's close conditions do not require editing that packet.
