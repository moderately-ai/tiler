---
id: make-runtime-routing-commit-authority-one-shot
title: Make runtime fallback authority consumable exactly once
status: done
priority: p1
dependencies: []
related: [prototype-metal-runtime-preflight, prototype-metal-runtime-execution, preflight-every-entry-of-a-multi-stage-route]
scopes: [implementation/runtime]
shared_scopes: []
paths: []
tags: [runtime, correctness, routing, fallback]
---
After Tiler commits to an artifact route, a caller must not be able to recover
or mint another authority that permits semantic fallback for the same attempt.

## Fact

`DecodedProgram::preflight` is callable through `&self`, and the decoded program
is clonable. A non-clone `Preflight` value is therefore not unique: callers can
mint more than one before consuming any of them.

## Outcome

Successful preflight yields one route-level authority whose consumption is the
only way to cross the routing commit. The authority covers every stage that may
execute and cannot be recreated from a retained or cloned decoded program.
Precommit refusals leave fallback legal; after consumption, allocation,
encoding, submission, validation, or execution failure is terminal.

## Closes when

The type and call-site structure make two commits or post-commit fallback
unrepresentable for one route, and negative tests prove repeated preflight or
commit cannot recover the authority.

## Outcome

Landed at `1ee4414`. The stated fact was exact, and the fix is two changes that together make the property structural.

### What was actually wrong

The three doc-tests on `Preflight::commit` prove that *a given* authority is single-use: it cannot be committed twice (`E0382`) and cannot be duplicated (`E0277`). They say nothing about how many authorities a caller may **mint**. With `DecodedProgram` deriving `Clone` and `preflight` taking `&self`, a caller could clone the program, preflight both, commit one, and still hold an uncommitted authority for the same attempt.

The module documented the stronger property while holding only the weaker one, which is the part worth recording: the tests were not wrong, they were answering a narrower question than the prose claimed.

### The fix

`DecodedProgram` is no longer `Clone`, and `preflight` takes `&mut self`. The exclusive borrow is not about mutation — nothing here mutates. `Preflight` borrows the program and `commit` passes that borrow into the `RoutedDispatch` it returns, so the program stays exclusively borrowed for as long as a committed route lives, and a second `preflight` does not compile. Removing `Clone` closes the escape of duplicating the program to get a fresh borrow.

**Abandoning stays legal, deliberately.** Dropping a `Preflight` without committing releases the borrow and permits another attempt. That is the fallback ADR 0051 grants before the commit; what is refused is minting a second authority while one has already been carried *across* it.

### Evidence

Two compile-fail doc-tests join the three that existed: `E0499` for preflighting again while holding a committed route, and `E0277` for cloning the program to escape the borrow.

**Both were confirmed to fail for their stated reason rather than incidentally.** A `compile_fail` test passes when the snippet fails to compile for *any* reason, so the check is the inverse one: with both halves of the property reverted, the two new tests fail with "Test compiled successfully, but it's marked `compile_fail`" while the three older tests still pass. That isolates the new tests to the new property.

### Blast radius

Nothing in the workspace was cloning a `DecodedProgram`, so removing `Clone` broke no caller. Five prototype call sites gained `mut`. 961 workspace tests pass; the hardware run is unchanged and still agrees bit-for-bit on both paths.

### Also corrected here

`crates/tiler-runtime/src/load.rs`'s module doc still said a multi-entry variant "cannot be sequenced from an artifact alone" and that the decoder already refuses such an envelope. Both became false at `carry-the-stage-execution-order-in-the-envelope`; that change corrected the error variant and the inline comment and missed the module-level claim. Fixed here rather than left standing, since it is a load-bearing statement a reader would trust and this change rewrites the surrounding docs anyway.

### What this unblocks

`preflight-every-entry-of-a-multi-stage-route`, which reshapes `Preflight` to carry several entries. Doing that first would have meant reshaping the type twice and redoing the authority work on the new shape; the dependency order the audit recorded is the right one.
