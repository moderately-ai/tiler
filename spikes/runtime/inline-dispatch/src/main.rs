//! One inline `tiler::tensor!` region, dispatched on this host's Metal device.
//!
//! # What this spike is for
//!
//! Every link in the chain already had evidence separately. The facade reaches a
//! real routed entry against a real compiled `metallib`
//! (`crates/tiler/tests/facade/pass/inline_region_dispatches.rs`, whose consumer
//! refuses its own payload because a `trybuild` fixture cannot link Metal).
//! `route_with_adapter` reaches a completed dispatch on Metal hardware
//! (`prototypes/candle-metal-adapter`) and a completed dispatch with a bit-exact
//! oracle against a host interpreter (`crates/tiler-runtime/tests/adapter_route`).
//! No single artifact showed the two composed: an ordinary crate writing
//! `tiler::tensor! { … deliver macos; … }` and receiving the *kernel's* answer.
//!
//! This is that artifact, and it is out of tree for two reasons that are each one
//! line to reproduce. `crates/tiler/tests/dependency_direction.rs`'s
//! `no_package_depends_on_the_frontend` forbids any workspace package from
//! depending on `tiler`, so no member crate may be the consumer. And the root
//! manifest's `[workspace.lints.rust] unsafe_code = "forbid"` cannot be relaxed
//! by an inner attribute at any scope, so no member crate may read an
//! `MTLBuffer` — `metal` 0.33.0 publishes only the raw pointer. A separate
//! workspace costs neither, and this crate denies rather than forbids so its one
//! readback site can opt in by name under ADR 0079.
//!
//! # Producer-declared equality, not host-earned eligibility
//!
//! The environment this adapter reports is the one the seam published, which is
//! the profile the artifact's **producer** declared. ADR 0086 decides that native
//! device translation of a `metallib` during pipeline creation is a capability
//! fact whose authority is `Unknown` on every macOS row currently observable, so
//! **no host earns the right to offer that profile** — this one included. A
//! completed dispatch is therefore not a qualified host, and this binary prints
//! `tiler::__private::PRODUCER_DECLARED_EQUALITY` rendered with the governed
//! profile key beside every outcome, for the same reason
//! `prototypes/serial-sum-run` prints those words.
//!
//! # The oracle is this crate's own arithmetic
//!
//! [`oracle`] is plain Rust `f32`, written the way this consumer would have
//! written the region without Tiler. Nothing about it is derived from anything
//! Tiler produced — not from a reference kernel, not from a sidecar, not from the
//! facade's fallback — because an oracle derived from the thing under test agrees
//! with it by construction. The comparison is bit-for-bit and runs **before** any
//! other claim this binary makes.
//!
//! # Running it
//!
//! By hand, from this directory. No `make` target reaches a spike.
//!
//! ```sh
//! cargo run --release
//! cargo run --release -- --halt-after-commit
//! ```

mod adapter;
mod buffer;

use std::process::ExitCode;
use std::rc::Rc;

use tiler::value::{BindError, StorageScalar, Tensor};

use crate::adapter::{Context, HostError, HostTensor, Metal, Perturbation, Session};

/// The region's operands. Chosen so every product and sum is exactly
/// representable in `f32`, which is what makes a bit-for-bit comparison a
/// statement about the dispatch rather than about rounding.
const LEFT: [f32; 4] = [1.5, -2.0, 0.25, 8.0];
/// The second operand; see [`LEFT`].
const RIGHT: [f32; 4] = [4.0, 3.0, -16.0, 0.5];
/// The third operand; see [`LEFT`].
const ADDEND: [f32; 4] = [0.5, 1.0, 2.0, -3.0];

/// The oracle: the same formula the region declares, in this crate's own `f32`.
///
/// Deliberately not `mul_add`. The region declares `(a * b) + c`, which is a
/// multiply and then an add with a rounding between them; a fused multiply-add
/// rounds once and is a *different* computation whose result can differ in the
/// last bit. Writing the oracle as the region reads is what makes a bit-for-bit
/// disagreement evidence about the dispatch rather than about which contraction
/// each side chose.
fn oracle(a: &[f32], b: &[f32], c: &[f32]) -> Vec<f32> {
    a.iter()
        .zip(b)
        .zip(c)
        .map(|((left, right), addend)| left * right + addend)
        .collect()
}

/// Returns the dense native-endian byte run of an `f32` slice.
fn dense_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

/// Wraps one of this consumer's values for this consumer's adapter.
///
/// The adapter is named here rather than inferred at the region, because a
/// `HostTensor` says nothing about which adapter reads it — that is exactly the
/// property `Tensor`'s type parameter exists to fix.
fn wrap(values: &[f32], session: &Context) -> Tensor<Metal> {
    Tensor::new(HostTensor::f32s(values), Rc::clone(session))
}

/// The one delivering region in this binary.
///
/// `deliver macos;` is what makes the expansion compile the region ahead of time,
/// embed the resulting artifact envelope in this binary as a byte-string literal,
/// and emit a call to `bind_route_and_build` rather than to `bind_and_build`.
/// Every extent is literal because a selected family is compiled ahead of time.
///
/// The `Result` is the region's, not the route's: a refusal *before* the routing
/// commit leaves the region on its declared result and is `Ok`, and only a
/// committed dispatch that did not complete is `Err(BindError::DispatchFailed)`.
///
/// The operands are wrapped here rather than passed in, because the expansion
/// emits `&[&a, &b, &c]` over the identifiers the `in` list names — so they have
/// to be values in this scope, and a by-reference parameter would produce a
/// double reference the seam's slice cannot take.
///
/// `contract flush_subnormals_to_zero_f32;` is what this region's arithmetic
/// actually means on the target it delivers for, and it is the narrowest true
/// statement of it rather than the first one the grammar accepts. Three of the
/// five statable contracts are refused at the `deliver` target before any bytes
/// exist — `strict_f32`, `reassociate_f32`, and `relaxed_f32` each require
/// preserved input subnormals, and the measured Apple `f32` row flushes in every
/// math mode, which `crates/tiler-macros/src/aot/tests.rs`'s
/// `the_bound_declaration_admits_the_two_flushing_contracts` pins as the admitted
/// pair. Of the two that remain, `flush_and_reassociate_f32` additionally
/// authorizes ordered regrouping of a same-operation operand sequence, and this
/// region holds none: `(a * b) + c` is a pointwise chain with no reduction and no
/// operand sequence to regroup, so stating it would claim a freedom nothing here
/// exercises and nothing here measures. `flush_subnormals_to_zero_f32` names the
/// two dimensions the hardware measurably moves and refuses every other, which is
/// the meaning under which [`oracle`] is the right comparison — contraction stays
/// forbidden in both, so the "deliberately not `mul_add`" argument is unaffected
/// either way.
fn dispatch_region(session: &Context) -> Result<HostTensor, BindError<HostError>> {
    let (a, b, c) = (
        wrap(&LEFT, session),
        wrap(&RIGHT, session),
        wrap(&ADDEND, session),
    );
    tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        deliver macos;
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    }
}

/// The same region with no `deliver` statement.
///
/// It embeds nothing, builds no device authority, and routes nothing. What it
/// establishes is recorded honestly in [`report_fallback_only_region`]: the
/// facade's fallback path constructs the region's *declared* result and does not
/// evaluate the expression, so this is a comparison of shape and stored scalar
/// and not of arithmetic.
///
/// It states the same contract as [`dispatch_region`] because it is the same
/// region — the whole claim this function makes is that `deliver` is the only
/// difference between the two, and a second contract would make them two
/// programs and leave that claim unsupported.
///
/// **The statement is inert on this path, and saying so is the point.** With no
/// `deliver`, `tiler-macros`' `expand` resolves the contract and then takes the
/// branch that never calls `aot::deliver`, so nothing compiles under it and no
/// target feasibility check sees it. That is not a defect here: the fallback
/// performs no arithmetic at all, so there is no behaviour for a contract to
/// constrain. The gap is owned by
/// `tickets/check-the-stated-contract-on-the-semantic-fallback-path.md`, which is
/// `deferred` until the fallback evaluates something.
fn fallback_only_region(session: &Context) -> Result<HostTensor, BindError<HostError>> {
    let (a, b, c) = (
        wrap(&LEFT, session),
        wrap(&RIGHT, session),
        wrap(&ADDEND, session),
    );
    tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    }
}

fn main() -> ExitCode {
    let halt = std::env::args().any(|argument| argument == "--halt-after-commit");
    let perturbation = halt.then_some(Perturbation::HaltAfterCommit);

    let Some(session) = Session::open(perturbation) else {
        // A refusal to report rather than a failure to hide: a host with no
        // Metal device cannot run this spike, and saying so exactly is the
        // honest terminal state.
        eprintln!(
            "refused: this host reports no default Metal device, so no dispatch was attempted"
        );
        return ExitCode::FAILURE;
    };
    let session: Context = Rc::new(session);
    println!("device: {}", session.device().name());
    println!(
        "mode: {}",
        if halt {
            "perturbed — the submission is halted after the routing commit"
        } else {
            "sound"
        },
    );

    let outcome = dispatch_region(&session);

    let code = if halt {
        report_halted_after_commit(&outcome, &session)
    } else {
        report_dispatched(&outcome, &session)
    };

    for stage in &session.journal().stages {
        println!("stage: {stage}");
    }
    for note in &session.journal().notes {
        println!("{note}");
    }

    // Printed after the outcome and before anything a reader might mistake for
    // an eligibility claim. The words are the facade's own constant, rendered
    // with the governed profile key the seam published.
    match session.journal().declared_profile.as_deref() {
        Some(profile) => {
            println!("{}", tiler::__private::producer_declared_equality(profile));
            println!(
                "ADR 0086 refuses the host: native `metallib` translation during pipeline creation \
                 is a capability fact whose authority is Unknown on every macOS row currently \
                 observable, so no host — this one included — earns the right to offer \
                 `{profile}`. The route above was settled on producer-declared equality, NOT \
                 host-earned eligibility.",
            );
        }
        None => println!(
            "no profile was published, so the route never reached this consumer's device authority"
        ),
    }

    report_fallback_only_region(&fallback_only_region(&session));

    code
}

/// Reports the sound run, with the oracle comparison **first**.
fn report_dispatched(
    outcome: &Result<HostTensor, BindError<HostError>>,
    session: &Context,
) -> ExitCode {
    let produced = match outcome {
        Ok(produced) => produced,
        Err(failure) => {
            eprintln!("the region did not produce a result: {failure}");
            return ExitCode::FAILURE;
        }
    };

    // The oracle, before any other claim. A dispatched result that does not
    // equal this consumer's own arithmetic is a wrong answer, and nothing about
    // the route having completed makes it a right one.
    let expected = oracle(&LEFT, &RIGHT, &ADDEND);
    let expected_bytes = dense_bytes(&expected);
    if produced.bytes() != expected_bytes.as_slice() {
        eprintln!(
            "ORACLE DISAGREES: the kernel wrote {:?} and this consumer's own arithmetic gives \
             {expected:?}",
            produced.read(),
        );
        eprintln!("  kernel bytes:   {:02x?}", produced.bytes());
        eprintln!("  oracle bytes:   {expected_bytes:02x?}");
        return ExitCode::FAILURE;
    }
    println!(
        "oracle: the dispatched bytes equal this consumer's own f32 arithmetic bit for bit: {:?}",
        produced.read(),
    );

    // Only now, and only because the bytes already agreed.
    //
    // The oracle alone does not distinguish "the kernel wrote this" from "the
    // region fell back", so the commit is checked separately and structurally.
    // `route_with_adapter` calls `Preflight::commit()` on the line before it
    // calls `RuntimeAdapter::dispatch`, and nothing else calls that method — so
    // the `dispatch` stage having run is the commit, taken inside the driver
    // that owns it, and the completion note exists only if that method returned
    // `Ok`, which is what makes the facade's outcome `RouteOutcome::Dispatched`.
    let journal = session.journal();
    if journal.stages.last() != Some(&"dispatch") {
        eprintln!(
            "the route did not reach the committed dispatch stage: {:?}",
            journal.stages,
        );
        return ExitCode::FAILURE;
    }
    let Some(completed) = journal
        .notes
        .iter()
        .find(|note| note.starts_with("committed route completed"))
    else {
        eprintln!(
            "the adapter recorded no completion, so no route reached RouteOutcome::Dispatched"
        );
        return ExitCode::FAILURE;
    };
    println!("commit: {completed}");

    // There is deliberately no separate "the result is not all zeros" check.
    // The facade's fallback constructs a zero-filled declared result, so that is
    // the state a missing readback leaves behind — but this region's oracle is
    // non-zero by construction ([`LEFT`] and friends), so the comparison above
    // already refuses it, and it does: removing the readback and re-running
    // prints `ORACLE DISAGREES: the kernel wrote [0.0, 0.0, 0.0, 0.0]`. A second
    // check that cannot reach a verdict the first did not already reach is a
    // check nothing could watch fail.
    assert_eq!(produced.scalar(), StorageScalar::F32);
    assert_eq!(produced.extents(), [4]);
    println!(
        "result: f32{:?}, {} byte(s)",
        produced.extents(),
        produced.bytes().len(),
    );
    ExitCode::SUCCESS
}

/// Reports the perturbed run: a post-commit failure, watched failing.
///
/// Two things must hold, and the second is the one that matters. The region must
/// refuse with [`BindError::DispatchFailed`] — ADR 0051 permits no fallback after
/// the commit — and it must **not** hand back a value at all, because the result
/// storage holds whatever a halted dispatch left in it and returning that as
/// though the semantic fallback had produced it would be returning an incorrect
/// tensor to preserve a fast path.
fn report_halted_after_commit(
    outcome: &Result<HostTensor, BindError<HostError>>,
    session: &Context,
) -> ExitCode {
    match outcome {
        Err(BindError::DispatchFailed { detail }) => {
            println!("post-commit failure, as required: {detail}");
            println!(
                "no value was returned: the halted dispatch's result storage never reached the \
                 caller, so nothing could be mistaken for the semantic fallback's answer",
            );
            // The route still reached the committed dispatch stage — that is
            // what makes this a *post*-commit failure rather than a refusal —
            // and it still recorded no completion, which is what makes it a
            // failure rather than a dispatch.
            let journal = session.journal();
            if journal.stages.last() != Some(&"dispatch") {
                eprintln!(
                    "the perturbation refused before the commit, so it is not the post-commit \
                     case: {:?}",
                    journal.stages,
                );
                return ExitCode::FAILURE;
            }
            if journal
                .notes
                .iter()
                .any(|note| note.starts_with("committed route completed"))
            {
                eprintln!("the halted route recorded a completion, which it cannot have reached");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(other) => {
            eprintln!(
                "the perturbation produced the wrong refusal: {other}; a halt after the commit \
                 must surface as BindError::DispatchFailed",
            );
            ExitCode::FAILURE
        }
        Ok(produced) => {
            let expected = dense_bytes(&oracle(&LEFT, &RIGHT, &ADDEND));
            eprintln!(
                "the perturbation was accepted: a halted dispatch returned {:?}",
                produced.read(),
            );
            eprintln!(
                "  and the semantic fallback's value is {:?}; returning either here would be a \
                 fallback ADR 0051 forbids",
                oracle(&LEFT, &RIGHT, &ADDEND),
            );
            debug_assert_ne!(produced.bytes(), expected.as_slice());
            ExitCode::FAILURE
        }
    }
}

/// Reports what the same region without `deliver` establishes, and what it does not.
///
/// **It does not compute.** `tiler::__private::bind_and_build` checks the
/// region's operands and then calls the adapter's own `build` for the declared
/// result; nothing in the facade evaluates `(a * b) + c` on the host. So a
/// fallback-only region is a comparison of the declared interface — rank, stored
/// scalar, extents — and never of arithmetic, and the value oracle above is this
/// crate's own `f32` for exactly that reason. Stating it here rather than
/// leaving the reader to infer it from a buffer of zeros is the difference
/// between a recorded boundary and an apparent second oracle.
fn report_fallback_only_region(outcome: &Result<HostTensor, BindError<HostError>>) {
    match outcome {
        Ok(plain) => {
            assert_eq!(plain.scalar(), StorageScalar::F32);
            assert_eq!(plain.extents(), [4]);
            println!(
                "fallback-only region: same declared interface (f32{:?}, {} byte(s)), and its \
                 storage is {:?} — the facade constructs the declared result and evaluates \
                 nothing, so this is not a second value oracle",
                plain.extents(),
                plain.bytes().len(),
                plain.read(),
            );
        }
        Err(failure) => println!("fallback-only region refused: {failure}"),
    }
}
