---
id: pin-the-admitted-unsafe-sites-in-the-workspace-gate
title: Pin the admitted unsafe sites in the workspace gate
status: review
priority: p2
dependencies: []
related: [record-the-case-by-case-unsafe-boundary, prototype-metal-runtime-execution]
scopes: [implementation/workspace, contracts/navigation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, workspace, gate, rust-api]
claimed_from: todo
assignee: agent-cache
lease_expires_at: 1785003790
---
ADR 0079 admits unsafe code case by case at an individual function or module, and records that exactly one half of that boundary is mechanically checked. This ticket closes the other half.

**Fact — what the gate pins today.** `scripts/check_workspace.py` carries `UNINHERITED_LINT_MEMBERS`, a single-entry map from `tiler-prototype-run` to the exact `[lints]` table it may declare instead of inheriting `[workspace.lints]`. It is consulted twice: `expected_member_manifest` substitutes it for `{workspace = true}` in the full manifest comparison, and a second explicit comparison reports the lint table on its own. So a second member dropping inheritance fails the gate, and so does widening that member's `unsafe_code = "deny"` to `"allow"`, adding a lint to its table, or removing one.

**Fact — what nothing checks.** No check counts, locates, or constrains `#[allow(unsafe_code, reason = ...)]` attributes inside the crate permitted to have them. At `43f685f` there are two, both in `prototypes/serial-sum-run/src/buffer.rs` on `write_f32` and `read_f32`. A third added anywhere in that crate compiles and passes `uv run --locked python scripts/check_repository.py` unchanged. ADR 0079's item 2 third property is a claim about which *crates* may diverge; it is not a claim about sites, and the ADR says so in its Consequences rather than leaving a reader to assume the check exists.

**Why it matters more than the crate half.** ADR 0079 is deliberately a case-by-case permission: a third site is a new decision, not an application of the existing one. That rule is currently enforced by review alone, and review is exactly what a gate is for when the predicate is mechanical. The crate half — the part a reviewer would notice anyway, because it changes a manifest — is the half that is already checked.

## The design question this ticket must answer

What the check pins is not obvious and should be decided by writing it, not assumed here. At least three predicates are available and they fail differently:

- **A count.** Cheapest, and it fails on the wrong thing: moving a site from one function to another passes, and adding a site while deleting an unrelated one passes.
- **File-and-item pairs.** Names each admitted site as `(path, item)`. Fails closed on an addition and on a move, at the cost of a rename churning the pin. Needs a source scan rather than a manifest read, which is a new capability for this script.
- **The attribute text.** Strongest — it pins the `reason` string too, so weakening a justification is a gate failure. Most brittle to reformatting; `rustfmt` owns the wrapping of a long `reason`.

Whichever is chosen, decide and record whether the check reads Rust source textually or parses it, and what it does about `#[allow(unsafe_code)]` appearing inside a string, a comment, or a `#[cfg]`-disabled item. A textual scan that cannot distinguish those is not obviously wrong for this repository — the whole universe is one crate and twelve grep hits — but the limitation must be stated in the script rather than discovered later.

## Closes when

`scripts/check_workspace.py` fails when an `#[allow(unsafe_code)]` site is added, moved, or removed without updating its pin; a mutation test proves that failure the way the script's existing checks are proven; ADR 0079's Consequences bullet naming this gap is amended to record that it is closed (that edit is `contracts/decisions` and needs the scope added or a split); and the full gate passes.

## Outcome

### The predicate chosen, and why

**The `(package-relative path, item signature) -> reason` triple**, combining the ticket's second and third options. `ADMITTED_UNSAFE_SITES` in `scripts/check_workspace.py` holds the two landed sites; `validate_unsafe_site_pins` scans and compares, and `main` composes it with the manifest contract as a separate phase — it reads Rust source rather than manifests or resolved metadata, so folding it into `validate_manifest_contract` would have made that function's tmp-root mutation tests report site errors about a tree that has no sources.

The count predicate is rejected for the reason the ticket predicts: it passes when a site moves, and when one is added while another is deleted. `test_a_moved_site_fails_even_though_the_count_is_unchanged` is that case, pinned.

The `reason` is pinned as well as the location, which the ticket lists as the strongest and most brittle option. It is worth the brittleness because the reason **is** ADR 0079 item 3's second condition; a check that pinned only the location would establish where the permission sits and nothing about what it claims, so weakening a justification would remain a diff nobody has to look at. The accepted cost is that a rename, a signature change, or a rewording churns the table — and each of those genuinely changes what was admitted. `rustfmt` does not reflow string literals without `format_strings`, and `rustfmt.toml` sets only `edition` and `max_width`, so ordinary reformatting does not churn it.

### Textual scan, and the limits stated in the script

The scan is textual, as the ticket permits, and both limits are in `scan_unsafe_allow_sites`' own docstring rather than left to be discovered:

- It recognizes `unsafe_code` only inside an `#[allow(…)]`/`#![allow(…)]` group that begins a line, and ignores the token on a comment line — which is what keeps `buffer.rs`'s own module documentation, which names the lint, from registering as a site.
- **Every other occurrence is reported as unaccounted-for.** A `cfg_attr`, a macro-generated attribute, a block comment, or a string literal holding the token stops the gate until someone decides what it is. That is the fail-closed direction, and it means the scan's inability to parse Rust cannot silently admit a site; it can only produce a failure that has to be resolved.

Two robustness properties are implemented and tested rather than assumed. Bracket balance is counted with double-quoted runs removed, so a `reason` containing `(` or `[` does not end the attribute early; and an attribute skipped while looking for the admitted item is skipped by its whole span, so a multi-line `#[cfg_attr(…)]` between the `#[allow]` and the signature does not leave its continuation mistaken for the item.

**Only `#[allow]` sites are pinned, and that is sufficient rather than partial.** ADR 0079 item 2 keeps `unsafe_code` at `deny` or `forbid` in every member, so an `unsafe` block that no attribute admits does not compile. The attributes are the complete set by construction, and the compiler is what makes that true.

The check additionally asserts that every pinned path lies inside a member `UNINHERITED_LINT_MEMBERS` names. A pin outside one would record a permission that cannot exist, and this is what keeps ADR 0079's crate half and site half from drifting apart.

Spike workspaces are out of range: they are Cargo workspaces excluded from this one, none is a shipping component, and `grep -rn "unsafe" --include="*.rs" spikes/` returns three lines, all `#![forbid(unsafe_code)]`.

### A finding worth recording

The obvious search for this work, `grep -rn "allow(unsafe_code" crates/ prototypes/`, returns **nothing** — both landed attributes are four lines, with `unsafe_code` on its own. That is exactly the multi-line-attribute failure `AGENTS.md`'s research standard names, encountered on the first search of the ticket, and it is why the scanner accumulates a bracket-balanced span instead of matching lines.

### Tests

Ten mutation tests in `scripts/tests/test_rust_gate_integrity.py`, each against a synthetic package tree and a synthetic pin: the admitted multi-line form is one site; an added site fails; a moved site fails while the count is unchanged; a removed site fails; a weakened reason fails; a site with no reason is rejected; a `cfg_attr` occurrence is reported; prose is not a site; a bracket inside a reason does not end the attribute; a multi-line trailing attribute is skipped whole; and a pin outside a diverging member is rejected. `test_the_checked_in_unsafe_sites_match_their_pins` asserts the real tree conforms **and** that the scan found exactly the two pinned sites — without the second assertion, a scan that reached nothing would report no violations against a table it never opened.

### Records updated

ADR 0079's Consequences bullet naming this gap is struck through and replaced with a dated closure recording the predicate, the scan limits, and the mutation evidence; its Implementation boundary now names three of item 3's four conditions as review-only rather than four, because the `reason` condition's presence and stability are now checked (whether the reason is *true* is still review's, and no check can decide that); and the "Enumerate the admitted sites" alternative records that the split ticket landed it and which option it chose. `AGENTS.md`'s unsafe clause gains the pin. `implementation_status` stays `implemented`.

## What this ticket delivered no longer exists — 2026-07-26

`e197176` replaced the Python gate with the root `Makefile` and deleted `scripts/` entire, which took `ADMITTED_UNSAFE_SITES`, `validate_unsafe_site_pins`, and all ten mutation tests in `scripts/tests/test_rust_gate_integrity.py` with it. Everything recorded above is an accurate account of work that landed and was reviewed; none of it is in the tree now, and the gap this ticket closed is open again.

The record was corrected in the same change that added this note: ADR 0079's Consequences bullet is un-struck and marked reopened, its Implementation boundary again names four review-only conditions rather than three, and the predicate rationale is preserved there for whoever re-implements the check. `AGENTS.md` already told the truth — it states that no check keeps an inventory of admitted sites — so the ADR and the working contract now agree.

**The disposition of this ticket is Tom's**, and it is not a rename away from being satisfiable: its closing condition names a file that will not exist again under the same design, because the gate it extended is gone. Reopening it to re-implement the pin against the `Makefile`, or closing it as obsolete and letting ADR 0079's reopened gap carry the debt, are different decisions about how much of the deleted enforcement is worth rebuilding. Nothing in this landing decides that.
