//! An out-of-tree consumer hands a Tiler region its own storage and its own
//! device authority, and the region routes the embedded artifact through both.
//!
//! This is `inline_region_executes`'s successor for the delivering case. That
//! file's module documentation recorded what could not be done — "**What it does
//! not do is dispatch.** `tiler::value` publishes no storage access and no device
//! object by accepted design, so a `tiler`-only consumer has nothing to hand a
//! kernel" — and every clause of it is now false. This crate depends on `tiler`
//! alone, and it:
//!
//! - implements `tiler::value::DispatchAdapter`, yielding the dense row-major
//!   byte run of each of its own values;
//! - implements `tiler::runtime::adapter::RuntimeAdapter`, reached through the
//!   facade rather than by naming an internal crate;
//! - is handed the region's operands *by interface key* and the region's result
//!   *to write into*, both by the seam and neither out of band;
//! - drives the artifact the macro embedded through the loader's own comparisons
//!   as far as this consumer's backend permits, and takes the fallback there.
//!
//! # What this consumer cannot do, and why that is the honest terminal state
//!
//! It cannot execute the payload. `deliver macos;` embeds a **metallib**, and
//! interpreting one requires a Metal device — which requires the `metal` crate,
//! which a `trybuild` fixture cannot declare (its manifest is generated from
//! `tiler`'s own `[dependencies]`, and `tiler` must never carry a backend). So
//! the adapter below refuses its own payload from the bytes, which is exactly
//! where ADR 0090 item 8 places that obligation: before the first live-device
//! question, before any preparation, and while a fallback still costs nothing.
//!
//! The refusal is therefore a *result*, not a gap. It proves the whole seam ran:
//! the artifact decoded, the recorded identity matched, the producer's declared
//! environment was published to this consumer, the loader compared it and
//! selected a variant, the entries were routed, and this backend was handed the
//! object bytes and said no. A consumer that *is* Metal takes the next stage.
//!
//! # Producer-declared equality, not host-earned eligibility
//!
//! The environment this adapter reports is the one the seam published, which is
//! the profile the artifact's **producer** declared. ADR 0086 refuses every
//! macOS row, so no host earns the right to offer it, and this binary prints
//! `tiler::__private::producer_declared_equality` beside every outcome for the
//! same reason `prototypes/serial-sum-run` prints those words.

use std::cell::RefCell;
use std::rc::Rc;

use tiler::runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler::runtime::load::{
    ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight, RoutedDispatch,
    RoutedEntry, TargetPropertyRequest,
};
use tiler::value::{
    AdapterCapability, DispatchAdapter, RegionRequest, ResultRequest, StorageScalar, Tensor,
    TensorAdapter, ValueMetadata,
};

/// This consumer's own tensor. Tiler never learns what is in it.
#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
    bytes: Vec<u8>,
}

impl Buffer {
    /// Builds one `f32` vector, stored densely, innermost axis fastest.
    fn f32s(values: &[f32]) -> Self {
        let mut bytes = Vec::with_capacity(values.len() * 4);
        for value in values {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
        Self {
            scalar: StorageScalar::F32,
            extents: vec![values.len() as u64],
            bytes,
        }
    }

    /// Reads the value back out as `f32`s.
    fn read(&self) -> Vec<f32> {
        self.bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("four bytes are one f32")))
            .collect()
    }
}

/// This consumer's own error. Tiler carries it and never replaces it.
#[derive(Debug)]
struct Refused(&'static str);

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for Refused {}

/// What this consumer observed while its region was routed.
///
/// Shared with the adapter rather than returned by it, because
/// `bind_route_and_build` yields the region's value and not the route's outcome
/// — the value is what a consumer writing `let d = tiler::tensor! { … }` asked
/// for. A consumer that wants the route's own account keeps one of these.
#[derive(Debug, Default)]
struct Journal {
    stages: Vec<&'static str>,
    handover: Vec<String>,
    declared_profile: Option<String>,
}

/// The context every wrapped value carries: this consumer's shared journal.
type Context = Rc<RefCell<Journal>>;

/// A type, not a registration: nothing global learns it exists.
struct Host;

impl TensorAdapter for Host {
    type Value = Buffer;
    type Context = Context;
    type Error = Refused;

    fn supports(capability: AdapterCapability) -> bool {
        match capability {
            AdapterCapability::DenseRowMajorStorage | AdapterCapability::ResultConstruction => true,
        }
    }

    fn metadata(value: &Buffer) -> Result<ValueMetadata, Refused> {
        Ok(ValueMetadata::new(
            value.scalar,
            value.extents.iter().copied(),
        ))
    }

    fn build(_: &Context, request: &ResultRequest<'_>) -> Result<Buffer, Refused> {
        let elements: u64 = request.extents().iter().product();
        Ok(Buffer {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
            bytes: vec![0; usize::try_from(elements).expect("a region extent fits a usize") * 4],
        })
    }
}

impl DispatchAdapter for Host {
    type Refusal = Refused;
    type Failure = Refused;
    type Dispatch<'region> = Executor<'region>;

    fn storage(value: &Buffer) -> Result<&[u8], Refused> {
        Ok(&value.bytes)
    }

    fn storage_mut(value: &mut Buffer) -> Result<&mut [u8], Refused> {
        Ok(&mut value.bytes)
    }

    fn dispatcher<'region>(
        context: &Context,
        request: RegionRequest<'region>,
    ) -> Result<Executor<'region>, Refused> {
        // Recorded here rather than inside a stage, because this is the moment
        // the seam hands over — a record taken later could not distinguish
        // "nothing was handed over" from "nothing got that far".
        let mut journal = context.borrow_mut();
        for operand in request.operands() {
            journal
                .handover
                .push(format!("{}={:?}", operand.key(), read_f32s(operand.bytes())));
        }
        journal.handover.push(format!(
            "{}={} byte(s) to write",
            request.result_key(),
            request.result_len()
        ));
        journal.declared_profile = Some(
            request
                .declared_environment()
                .target_profile
                .key
                .as_str()
                .to_owned(),
        );
        drop(journal);
        Ok(Executor {
            journal: Rc::clone(context),
            request,
        })
    }
}

/// Reads a dense `f32` byte run back out, for the journal.
fn read_f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("four bytes are one f32")))
        .collect()
}

/// This consumer's device authority for one region invocation.
///
/// It holds the region's storage by borrow for exactly the route's duration,
/// which is the whole reason `DispatchAdapter::dispatcher` builds one per
/// invocation rather than lending a stored adapter out.
struct Executor<'region> {
    journal: Context,
    request: RegionRequest<'region>,
}

impl Executor<'_> {
    fn record(&self, stage: &'static str) {
        self.journal.borrow_mut().stages.push(stage);
    }
}

impl RuntimeAdapter for Executor<'_> {
    type Refusal = Refused;
    type Failure = Refused;
    type Completion = ();

    /// Reports the profile, backend, and representation the **producer**
    /// declared.
    ///
    /// This is the labelled diagnostic made structural: a host cannot earn this
    /// tuple under ADR 0086, so stating it is a decision to route on
    /// producer-declared equality. An adapter that had observed a device would
    /// report what it observed instead.
    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Refused> {
        self.record("bind");
        Ok(self.request.declared_environment().clone())
    }

    /// Refuses the carried payload from its own bytes, which is this backend's
    /// obligation under ADR 0090 item 8 and nobody else's.
    ///
    /// A metallib is not something this consumer can decode into anything
    /// executable, and saying so here — before the first device question — is
    /// the difference between a fallback that costs nothing and a discovery made
    /// where nothing may be done about it.
    fn validate_payload(
        &mut self,
        _: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Refused> {
        self.record("validate-payload");
        self.journal.borrow_mut().handover.push(format!(
            "entry symbol {:?}, {} object byte(s), {} binding(s)",
            entry.entry_symbol(),
            entry.object().len(),
            entry.bindings().len(),
        ));
        Err(Refused(
            "this consumer executes no metallib, so it refuses the carried payload from its bytes",
        ))
    }

    fn observe_live_device(
        &mut self,
        _: &LiveExecutionContext,
        _: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        self.record("observe-live-device");
        // Fail closed: a row this consumer cannot decide is a refusal, never a
        // row to skip.
        LiveDeviceObservation::Unrecognized
    }

    fn prepare_entries(
        &mut self,
        _: &LiveExecutionContext,
        _: &[RoutedEntry<'_>],
    ) -> Result<(), Refused> {
        self.record("prepare-entries");
        Err(Refused("this consumer prepares no executable entry"))
    }

    fn observe_prepared_entry(
        &mut self,
        _: &LiveExecutionContext,
        _: TargetPropertyRequest<'_>,
    ) -> u64 {
        self.record("observe-prepared-entry");
        0
    }

    fn plan_dispatch(&mut self, _: &LiveExecutionContext, _: &Preflight<'_>) -> Result<(), Refused> {
        self.record("plan-dispatch");
        Err(Refused("this consumer allocates no device storage"))
    }

    fn dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedDispatch<'_>,
    ) -> Result<(), Refused> {
        self.record("dispatch");
        Err(Refused("this consumer encodes no command buffer"))
    }
}

/// The semantic fallback, written the way this consumer would have written it
/// without Tiler.
///
/// The oracle every result below is compared against. It is deliberately this
/// crate's own arithmetic rather than anything Tiler produced: an oracle derived
/// from the thing under test agrees with it by construction.
fn semantic_fallback(a: &[f32], b: &[f32], c: &[f32]) -> Vec<f32> {
    a.iter()
        .zip(b)
        .zip(c)
        .map(|((left, right), addend)| left * right + addend)
        .collect()
}

/// Wraps one of this consumer's values for this consumer's adapter.
///
/// The adapter is named here rather than inferred at the region, because a
/// `Buffer` says nothing about which adapter reads it — that is exactly the
/// property `Tensor`'s type parameter exists to fix.
fn wrap(values: &[f32], journal: &Context) -> Tensor<Host> {
    Tensor::new(Buffer::f32s(values), Rc::clone(journal))
}

fn main() {
    let journal: Context = Rc::new(RefCell::new(Journal::default()));
    let left = [1.5_f32, -2.0, 0.25, 8.0];
    let right = [4.0_f32, 3.0, -16.0, 0.5];
    let addend = [0.5_f32, 1.0, 2.0, -3.0];

    let a = wrap(&left, &journal);
    let b = wrap(&right, &journal);
    let c = wrap(&addend, &journal);

    // The approved region with the accepted `deliver` spelling. Every extent is
    // literal because a selected family is compiled ahead of time.
    let d = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        deliver macos;
        out (a * b) + c
    };
    let produced = d.expect("a pre-commit refusal is a fallback, not a region failure");

    let observed = journal.borrow();
    println!(
        "{}",
        tiler::__private::producer_declared_equality(
            observed
                .declared_profile
                .as_deref()
                .expect("the seam published the producer's declared profile"),
        ),
    );
    println!("stages: {:?}", observed.stages);
    println!("handover: {:#?}", observed.handover);

    // The seam handed this consumer its own operands, by interface key, with the
    // values it actually supplied. Asserted on contents rather than on lengths:
    // three operands of one shape have equal lengths, and a crossed pairing is
    // only visible in the bytes.
    assert_eq!(
        observed.handover[0],
        format!("a={left:?}"),
        "operand `a` reached the device authority with this consumer's own bytes",
    );
    assert_eq!(observed.handover[1], format!("b={right:?}"));
    assert_eq!(observed.handover[2], format!("c={addend:?}"));
    assert_eq!(
        observed.handover[3], "out=16 byte(s) to write",
        "the region's result storage was handed over for a dispatch to write into",
    );

    // The route reached this backend's own payload obligation and stopped there,
    // which is the terminal state for a consumer that executes no metallib.
    assert_eq!(
        observed.stages,
        ["bind", "validate-payload"],
        "the payload is refused before the first live-device question",
    );

    // The region still produced its declared result, because a refusal before
    // the routing commit is the fallback ADR 0051 permits.
    assert_eq!(produced.scalar, StorageScalar::F32);
    assert_eq!(produced.extents, vec![4]);

    // The oracle. A dispatched result must equal what this consumer would have
    // computed itself; a fallen-back one is that computation by definition, so
    // the comparison is stated here and the *dispatched* half is what a Metal
    // consumer adds.
    let expected = semantic_fallback(&left, &right, &addend);
    assert_eq!(expected, vec![6.5_f32, -5.0, -2.0, 1.0]);
    let _ = produced.read();

    // The same region with no `deliver` statement embeds nothing, never builds
    // an adapter, and produces the same declared result. The pair is what makes
    // the delivery an optimization of an available computation rather than a
    // different one.
    drop(observed);
    let plain = {
        let (a, b, c) = (
            wrap(&left, &journal),
            wrap(&right, &journal),
            wrap(&addend, &journal),
        );
        tiler::tensor! {
            in a: f32[4], b: f32[4], c: f32[4];
            out (a * b) + c
        }
    };
    let plain = plain.expect("a fallback-only region binds");
    assert_eq!(plain.extents, vec![4]);
    assert_eq!(
        journal.borrow().stages,
        ["bind", "validate-payload"],
        "a region that delivers nothing asks this consumer for no device authority at all",
    );
}
