//! One inline `tiler::tensor!` region whose *selected* plan needs two entries,
//! dispatched on this host's Metal device.
//!
//! # What this binary adds to the one beside it
//!
//! `src/main.rs` — the `inline-dispatch-spike` binary — takes a pointwise region
//! to a completed dispatch and reports `1/1 entry(ies) encoded`. Everything about
//! a *bundle* was therefore still unwatched on hardware: whether an artifact one
//! expansion packaged with more than one executable entry runs, whether the
//! entries run in the order that artifact declares, and whether the consumer can
//! see how many there were.
//!
//! Nothing here asks for two kernels. The region states a computation
//! (`strict_serial_sum`) and what its arithmetic may do
//! (`flush_and_reassociate_f32`), and the compiler's selection policy answers
//! with a split; under `flush_subnormals_to_zero_f32` the same text selects one
//! fused kernel. That is the whole of the trigger, and it is why this binary may
//! not reach into a plan portfolio to find a split — a hand-picked non-selected
//! alternative would be evidence about this file rather than about the compiler.
//!
//! # The reordering runs first, and is not behind a flag
//!
//! A completed two-entry dispatch that agrees with the oracle is evidence about
//! *ordering* only if the same binary can watch the ordering fail. So [`main`]
//! runs the route twice against the same device with the same operands: once
//! with the committed route's entries encoded back to front, which must return a
//! **wrong answer** rather than a refusal, and then once in the artifact's
//! declared order, which must agree with [`oracle`] bit for bit. The perturbed
//! run is deliberately not a flag — a check that runs only when someone
//! remembers to pass an argument is a check the checked-in state does not make.
//!
//! # Running it
//!
//! By hand, from this directory. No `make` target reaches a spike.
//!
//! ```sh
//! cargo run --release --bin multi-entry-dispatch-spike
//! ```

mod adapter;
mod buffer;

use std::process::ExitCode;
use std::rc::Rc;

use tiler::value::{BindError, StorageScalar, Tensor};

use crate::adapter::{Context, HostError, HostTensor, Metal, Perturbation, Session};

/// The region's one operand, as `f32[rows: 1, cols: 4]` in row-major order.
///
/// **Exactly representable under regrouping, which is what the stated contract
/// makes necessary.** `flush_and_reassociate_f32` authorizes the compiler to
/// regroup the reduction's operand sequence, and it exercises that freedom here:
/// the selected plan sums the four contributors in two stages rather than left
/// to right. A bit-for-bit comparison against this consumer's own left-to-right
/// `f32` is a statement about the *dispatch* only if every grouping produces the
/// same bits, so the operands are chosen to make that true rather than assumed
/// to be harmless.
///
/// The argument is finite and does not depend on which split the compiler chose.
/// Each value is an integer multiple of `0.25`; `x * 2.0` is exact because two is
/// a power of two, and `+ 1.0` leaves every mapped contributor an integer
/// multiple of `0.5` — `[2.0, 3.5, -3.0, 7.5]`. Every sum of a subset of those
/// four is an integer multiple of `0.5` with magnitude at most `13.0`, so it
/// needs at most five significand bits against `f32`'s twenty-four. No partial
/// sum in any association rounds, so no association can disagree.
///
/// Nothing here is subnormal, infinite, or `NaN`, so the flushing half of the
/// contract is inert on this data too — deliberately, because this run measures
/// entry ordering and not the numerical contract's own boundary.
const X: [f32; 4] = [0.5, 1.25, -2.0, 3.25];

/// The entry count this region's stated contract must select.
const EXPECTED_ENTRIES: usize = 2;

/// Slot pairs the selected two-entry plan requires be backed by one allocation.
///
/// One: the reducing stage reads the scratch the mapping stage wrote. It is
/// asserted rather than merely reported because it is what makes the entries
/// ordered at all — the reordering below is a wrong answer only because this
/// pairing exists.
const EXPECTED_SHARED_ALLOCATIONS: usize = 1;

/// The oracle: the region's own formula, in this crate's own `f32`.
///
/// Derived from nothing Tiler produced — not a reference kernel, not a sidecar,
/// not the facade's fallback — because an oracle derived from the thing under
/// test agrees with it by construction. Written left to right, which is the one
/// association a consumer reading `strict_serial_sum` would write; the region's
/// stated contract permits the compiler to pick another, and [`X`] is chosen so
/// that every association it could pick produces these exact bits.
///
/// The reduction is over `cols` alone, so the result is `f32[rows]` — one rank
/// below the operand, and one element wide here.
fn oracle(x: &[f32]) -> Vec<f32> {
    let mut total = 0.0_f32;
    for value in x {
        total += value * 2.0 + 1.0;
    }
    vec![total]
}

/// Returns the dense native-endian byte run of an `f32` slice.
fn dense_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

/// The region, stated once and dispatched under each session in turn.
///
/// `deliver macos;` is what makes the expansion compile the region ahead of
/// time, embed the resulting artifact envelope in this binary as one byte-string
/// literal, and emit a call to `bind_route_and_build`. The artifact that lands
/// there carries **two** entries and the stage dependency between them; that both
/// are packaged, and in which order, is asserted on the artifact itself by
/// `tiler-macros`' `a_split_selection_packages_every_entry_in_the_one_embedded_artifact`.
/// What this function adds is that they run.
///
/// `contract flush_and_reassociate_f32;` is the trigger and the meaning at once.
/// Three of the five statable contracts — `strict_f32`, `reassociate_f32`, and
/// `relaxed_f32` — are refused for `deliver macos;` before any bytes exist,
/// because each requires preserved input subnormals and the measured Apple `f32`
/// row flushes them in every math mode. Of the two that remain, this one
/// additionally authorizes ordered regrouping of a same-operation operand
/// sequence, and this region *contains* one: `strict_serial_sum` reduces four
/// contributors along `cols`. So the statement is the narrowest true description
/// of what this consumer accepts, and the split is the compiler's answer to it
/// rather than anything this file asked for.
///
/// `[rows: 1, cols: 4]` is the window rather than a taste, measured on the
/// declaration this run bound: at the run's date `[rows: 1, cols: 8]` and
/// `[rows: 2, cols: 4]` were refused as `NoFeasiblePlan` and
/// `[rows: 1, cols: 5]` as `InvalidCompilerOutput`. The grid-axis row has since
/// widened to a retained measurement, so those refusals are dated observations
/// and the wider windows are merely unmeasured; widening the measured window
/// belongs to the reduction-strategy work rather than here.
fn dispatch_region(session: &Context) -> Result<HostTensor, BindError<HostError>> {
    let x: Tensor<Metal> = Tensor::new(HostTensor::f32_dense(&[1, 4], &X), Rc::clone(session));
    tiler::tensor! {
        in x: f32[rows: 1, cols: 4];
        deliver macos;
        contract flush_and_reassociate_f32;
        out strict_serial_sum(x * 2.0 + 1.0, [cols])
    }
}

/// How many entries this consumer *counted*, from three independent populations.
///
/// Counted rather than assumed, and from more than one place, because the state
/// this guards against is a single-entry plan that happened to be selected: a run
/// that only checked "the dispatch completed" would pass on one and report
/// nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EntryCensus {
    /// Payloads the loader handed this adapter to validate, one per entry.
    validated: usize,
    /// Entries the committed route declares.
    declared: usize,
    /// Command encoders this adapter created for it.
    encoded: usize,
}

/// The census a route that selected the split must report.
const EXPECTED_CENSUS: EntryCensus = EntryCensus {
    validated: EXPECTED_ENTRIES,
    declared: EXPECTED_ENTRIES,
    encoded: EXPECTED_ENTRIES,
};

/// Counts the entries of one route from the journal it left behind.
///
/// Returns `None` when no dispatch completed, which is a different state from a
/// route that completed with an unexpected count and is reported as such.
fn census(session: &Context) -> Option<EntryCensus> {
    let journal = session.journal();
    let completion = journal.completion.as_ref()?;
    Some(EntryCensus {
        validated: journal
            .stages
            .iter()
            .filter(|stage| **stage == "validate-payload")
            .count(),
        declared: completion.entries,
        encoded: completion.encoded,
    })
}

/// Prints the journal a route left behind, in the order it was recorded.
fn report_journal(session: &Context) {
    let journal = session.journal();
    for stage in &journal.stages {
        println!("stage: {stage}");
    }
    for note in &journal.notes {
        println!("{note}");
    }
}

/// Reports the entry census and requires it to be the split this region selects.
///
/// Both conditions are evaluated and both are reported; neither returns early.
/// They fail together under the one perturbation that reaches them — restating
/// the region's contract as `flush_subnormals_to_zero_f32`, which selects the
/// fused one-kernel plan — and a short-circuit would leave the second unwatched
/// under the only perturbation that can watch it.
fn report_census(session: &Context) -> bool {
    let Some(census) = census(session) else {
        eprintln!("no dispatch completed, so there is no entry census to report");
        return false;
    };
    let shared = session.journal().shared_allocations;
    println!(
        "entries: {} payload(s) validated, {} declared by the committed route, {} encoded; {} \
         shared allocation(s)",
        census.validated,
        census.declared,
        census.encoded,
        shared.map_or_else(|| "no plan reported".to_owned(), |count| count.to_string()),
    );

    let split = census == EXPECTED_CENSUS;
    if !split {
        eprintln!(
            "THE SELECTED PLAN IS NOT THE SPLIT: this region's stated contract must select \
             {EXPECTED_ENTRIES} entries and this route reports {census:?}. A single-entry plan \
             that happened to be selected would dispatch and agree with the oracle while \
             establishing nothing about a bundle, which is why this count is asserted rather than \
             read off.",
        );
    }
    let paired = shared == Some(EXPECTED_SHARED_ALLOCATIONS);
    if !paired {
        eprintln!(
            "THE ENTRIES DO NOT SHARE STORAGE: the route reports {shared:?} shared allocation(s) \
             and this plan's stage dependency needs {EXPECTED_SHARED_ALLOCATIONS}. Two entries \
             touching no common buffer have no order to get wrong, so the reordering this run \
             watches would prove nothing.",
        );
    }
    split && paired
}

/// Runs the route with the entries encoded back to front, and requires a wrong answer.
///
/// The failure `crates/tiler-runtime`'s shared-allocation pairing documents as
/// the one place in this stack that fails **open**, made concrete on hardware:
/// nothing refuses, every payload validates, every pipeline builds, both entries
/// reach terminal success — and the bits are wrong, because the entry that reads
/// the shared scratch ran before the entry that writes it.
///
/// Three things must hold together, and the third is the one with teeth. The
/// region must not surface an error, because a refusal would mean this stack
/// caught the reordering and there would be nothing to watch. The route must
/// report the same entry census as the sound run, because a reordering that
/// dropped an entry would be a different perturbation. And the bytes must differ
/// from [`oracle`]'s — were they equal, the order would not be observable at all
/// and the sound run below could not claim its agreement came from it.
///
/// **What the wrong answer is is not asserted, and that is deliberate.** The
/// reducing entry reads a shared allocation the mapping entry has not written
/// yet, and Metal does not specify the contents of freshly acquired private
/// storage — so the exact value is this host's, not the contract's. Only the
/// disagreement is checked; the README records what this host produced.
fn report_reordered() -> bool {
    let Some(session) = Session::open(Some(Perturbation::ReverseEncodeOrder)) else {
        eprintln!("refused: this host reports no default Metal device");
        return false;
    };
    let session: Context = Rc::new(session);
    let outcome = dispatch_region(&session);

    let answered_wrongly = match &outcome {
        Ok(produced) => {
            let expected = dense_bytes(&oracle(&X));
            if produced.bytes() == expected.as_slice() {
                eprintln!(
                    "THE REORDERING WAS NOT OBSERVABLE: encoding the entries back to front still \
                     produced {:?}, so nothing about this route depends on the order of its \
                     entries and the sound run's agreement is not evidence about it",
                    produced.read(),
                );
                false
            } else {
                println!(
                    "reordering: WRONG ANSWER, not a refusal — the route completed and the kernel \
                     wrote {:?} where this consumer's own f32 gives {:?}",
                    produced.read(),
                    oracle(&X),
                );
                true
            }
        }
        Err(failure) => {
            eprintln!(
                "the reordering was refused rather than answered wrongly: {failure}; a consumer \
                 stage dispatched before its producer is a wrong answer this stack does not \
                 detect, so a refusal here means this run watched something else",
            );
            false
        }
    };

    // Both are reported before either verdict is returned, so a run that fails
    // still prints the census and the stages a reader would need to place it.
    let counted = report_census(&session);
    report_journal(&session);
    answered_wrongly && counted
}

/// Runs the route in the artifact's declared order, with the oracle **first**.
fn report_dispatched() -> bool {
    let Some(session) = Session::open(None) else {
        eprintln!("refused: this host reports no default Metal device");
        return false;
    };
    let session: Context = Rc::new(session);
    let outcome = dispatch_region(&session);

    let agreed = check_dispatched(&outcome, &session);
    let counted = report_census(&session);
    report_journal(&session);

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
    agreed && counted
}

/// Checks the sound run: the oracle, then the commit, then the declared interface.
fn check_dispatched(outcome: &Result<HostTensor, BindError<HostError>>, session: &Context) -> bool {
    let produced = match outcome {
        Ok(produced) => produced,
        Err(failure) => {
            eprintln!("the region did not produce a result: {failure}");
            return false;
        }
    };

    // The oracle, before any other claim. A dispatched result that does not
    // equal this consumer's own arithmetic is a wrong answer, and nothing about
    // the route having completed makes it a right one.
    let expected = oracle(&X);
    let expected_bytes = dense_bytes(&expected);
    if produced.bytes() != expected_bytes.as_slice() {
        eprintln!(
            "ORACLE DISAGREES: the kernel wrote {:?} and this consumer's own arithmetic gives \
             {expected:?}",
            produced.read(),
        );
        eprintln!("  kernel bytes:   {:02x?}", produced.bytes());
        eprintln!("  oracle bytes:   {expected_bytes:02x?}");
        return false;
    }
    println!(
        "oracle: the dispatched bytes equal this consumer's own f32 arithmetic bit for bit: {:?}",
        produced.read(),
    );

    // Only now, and only because the bytes already agreed. The oracle alone does
    // not distinguish "the kernel wrote this" from "the region fell back", so the
    // commit is checked separately and structurally: `route_with_adapter` calls
    // `Preflight::commit()` on the line before it calls `RuntimeAdapter::dispatch`
    // and nothing else calls that method, so the `dispatch` stage having run is
    // the commit, and the completion note exists only if that method returned
    // `Ok` — which is what makes the facade's outcome `RouteOutcome::Dispatched`.
    let journal = session.journal();
    if journal.stages.last() != Some(&"dispatch") {
        eprintln!(
            "the route did not reach the committed dispatch stage: {:?}",
            journal.stages,
        );
        return false;
    }
    let Some(completed) = journal
        .notes
        .iter()
        .find(|note| note.starts_with("committed route completed"))
    else {
        eprintln!(
            "the adapter recorded no completion, so no route reached RouteOutcome::Dispatched"
        );
        return false;
    };
    println!("commit: {completed}");

    // The declared interface a multi-entry bundle must not change: the region's
    // result is `f32[rows]`, one rank below its operand, whatever the selected
    // plan needed to compute it.
    assert_eq!(produced.scalar(), StorageScalar::F32);
    assert_eq!(produced.extents(), [1]);
    println!(
        "result: f32{:?}, {} byte(s)",
        produced.extents(),
        produced.bytes().len(),
    );
    true
}

fn main() -> ExitCode {
    let Some(device) = Session::open(None).map(|session| session.device().name().to_owned()) else {
        // A refusal to report rather than a failure to hide: a host with no
        // Metal device cannot run this spike, and saying so exactly is the
        // honest terminal state.
        eprintln!(
            "refused: this host reports no default Metal device, so no dispatch was attempted"
        );
        return ExitCode::FAILURE;
    };
    println!("device: {device}");
    println!(
        "region: in x: f32[rows: 1, cols: 4]; deliver macos; contract flush_and_reassociate_f32; \
         out strict_serial_sum(x * 2.0 + 1.0, [cols])",
    );

    // The reordering first. An ordered run's agreement with the oracle is
    // evidence about ordering only if the disordered one has just been watched
    // disagreeing, on the same device with the same operands.
    println!("--- perturbed: the committed route's entries are encoded back to front ---");
    if !report_reordered() {
        return ExitCode::FAILURE;
    }

    println!("--- sound: the entries are encoded in the order the artifact declares ---");
    if report_dispatched() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
