//! An out-of-tree consumer routes one inline region twice, and the only thing
//! that differs between the two runs is one dtype-dispatchability row.
//!
//! # What this establishes that `inline_region_dispatches` cannot
//!
//! That file proves the seam runs end to end. This one proves the *dtype row is
//! load-bearing on that path*: the row an expansion emits reaches the adapter,
//! the adapter's answer decides the route, and a route whose stated rows do not
//! admit the region's dtype stops before the payload is ever looked at.
//!
//! Both halves are needed and neither implies the other. A run that only showed
//! the refusal would be evidence about a broken artifact rather than about the
//! row, so the accepted neighbour is the same region, the same operands, the
//! same adapter, and the same artifact with the row present.
//!
//! # Where the perturbation is applied, and why that is the honest place
//!
//! `RuntimeAdapter::bind_execution_context` is the authority the loader settles
//! a route against — an integration answers it, and this crate's `route` module
//! documents that an adapter returning `RegionRequest::declared_environment`
//! verbatim has chosen producer-declared equality. The baseline below does
//! exactly that, so the environment it publishes *is* the emitted rows. The
//! probe returns the same environment with one row withheld, which is the
//! smallest possible change to what the emitted fact says.
//!
//! It is deliberately not a claim about this machine. Nothing here observes a
//! device, and ADR 0086 refuses the applicability receipt that would let a host
//! offer this profile in the first place; what a withheld row models is a
//! producer whose profile never declared the dtype, which under the loader's
//! fail-closed rule is refused exactly as an explicit refusal is.

use std::cell::RefCell;
use std::rc::Rc;

use tiler::artifact::program::ArithmeticType;
use tiler::runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler::runtime::load::{
    DTypeDispatchResolution, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest,
    Preflight, PreparedEntryObservation, RoutedDispatch, RoutedEntry, TargetEnvironmentObservation,
    TargetEnvironmentSupport, TargetPropertyRequest,
};
use tiler::value::{
    AdapterCapability, DispatchAdapter, RegionRequest, ResultRequest, StorageScalar, Tensor,
    TensorAdapter, ValueMetadata,
};

/// The dtype this region's every value computes in, and therefore the one row
/// the perturbation withholds.
const REGION_DTYPE: ArithmeticType = ArithmeticType::F32;

/// This consumer's own tensor.
#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
    bytes: Vec<u8>,
}

impl Buffer {
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
}

/// This consumer's own error.
#[derive(Debug)]
struct Refused(&'static str);

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for Refused {}

/// What one run observed, and what it was told to state.
#[derive(Debug, Default)]
struct Journal {
    /// Whether this run withholds the region's dtype row from what it states.
    withhold_region_dtype: bool,
    /// Every adapter stage the route reached, in order.
    stages: Vec<&'static str>,
    /// How the environment this run published resolves the region's dtype.
    published: Option<DTypeDispatchResolution>,
}

type Context = Rc<RefCell<Journal>>;

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
        Ok(Executor {
            journal: Rc::clone(context),
            request,
        })
    }
}

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

    /// Registers no schema: this consumer's backend claims no ADR 0013
    /// authority, so claimed cells would filter while `Unclaimed` routes stay
    /// routable.
    fn target_environment_support(&self) -> TargetEnvironmentSupport<'_> {
        TargetEnvironmentSupport::Unsupported
    }

    /// Unreachable while no schema is registered; unavailable regardless.
    fn observe_target_environment(
        &mut self,
        _: &LiveExecutionContext,
    ) -> TargetEnvironmentObservation {
        TargetEnvironmentObservation::Unavailable {
            reason: "no target-environment schema is registered".to_owned(),
        }
    }

    /// States the emitted environment, with or without the region's dtype row.
    ///
    /// The withheld case removes exactly one entry and leaves the profile
    /// reference, backend, representation, and every other row identical, so a
    /// difference in outcome is a difference about that entry.
    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Refused> {
        self.record("bind");
        let mut environment = self.request.declared_environment().clone();
        if self.journal.borrow().withhold_region_dtype {
            environment.dtype_dispatch.remove(&REGION_DTYPE);
        }
        self.journal.borrow_mut().published = Some(environment.classify_dtype(REGION_DTYPE));
        Ok(environment)
    }

    fn validate_payload(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedEntry<'_>,
    ) -> Result<(), Refused> {
        self.record("validate-payload");
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
    ) -> PreparedEntryObservation {
        self.record("observe-prepared-entry");
        PreparedEntryObservation::Unrecognized
    }

    fn plan_dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &Preflight<'_>,
    ) -> Result<(), Refused> {
        self.record("plan-dispatch");
        Err(Refused("this consumer sizes no device storage"))
    }

    fn allocate_dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedDispatch<'_>,
    ) -> Result<(), Refused> {
        self.record("allocate-dispatch");
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

/// Routes the one region under test and reports what the run observed.
///
/// The region text is written once and routed twice, because two copies of it
/// would let the two runs drift into being about different programs.
fn route(withhold_region_dtype: bool) -> (Buffer, Vec<&'static str>, DTypeDispatchResolution) {
    let journal: Context = Rc::new(RefCell::new(Journal {
        withhold_region_dtype,
        ..Journal::default()
    }));
    let left = [1.5_f32, -2.0, 0.25, 8.0];
    let right = [4.0_f32, 3.0, -16.0, 0.5];
    let addend = [0.5_f32, 1.0, 2.0, -3.0];
    let a = Tensor::<Host>::new(Buffer::f32s(&left), Rc::clone(&journal));
    let b = Tensor::<Host>::new(Buffer::f32s(&right), Rc::clone(&journal));
    let c = Tensor::<Host>::new(Buffer::f32s(&addend), Rc::clone(&journal));

    let d = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        deliver macos;
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    };
    let produced = d.expect("a pre-commit refusal is a fallback, not a region failure");

    let observed = journal.borrow();
    (
        produced,
        observed.stages.clone(),
        observed
            .published
            .expect("the route asked this consumer for an execution environment"),
    )
}

fn main() {
    // The accepted neighbour. The adapter echoes the emitted rows back
    // unchanged, so the environment the loader classifies against carries the
    // producer's declared verdict for this region's dtype.
    let (accepted, stages, published) = route(false);
    println!("baseline: stages {stages:?}, published {published:?}");
    assert_eq!(
        published,
        DTypeDispatchResolution::Dispatchable,
        "the expansion must emit a dispatchable row for the dtype its own region computes in",
    );
    assert_eq!(
        stages,
        ["bind", "validate-payload"],
        "with the row present the route reaches this backend's own payload obligation",
    );

    // The perturbation: the same everything, one row withheld.
    let (refused, stages, published) = route(true);
    println!("withheld: stages {stages:?}, published {published:?}");
    assert_eq!(
        published,
        DTypeDispatchResolution::Unknown,
        "a withheld row must resolve as unmeasured rather than as a permissive default",
    );
    assert_eq!(
        stages,
        ["bind"],
        "a dtype the stated rows do not admit is refused before the payload is looked at, so the \
         backend is never asked about bytes it must not be handed",
    );

    // Both runs left the region on its declared semantic fallback, because both
    // refusals arrived before the one-way routing commit. The refusal is a
    // *route* outcome and not a region failure, which is what makes the row
    // safe to fail closed on.
    assert_eq!(accepted.scalar, StorageScalar::F32);
    assert_eq!(accepted.extents, vec![4]);
    assert_eq!(
        refused, accepted,
        "a refused route returns the same declared result"
    );
}
