//! The complete inline AOT workflow, in an ordinary consumer crate.
//!
//! This is the end-to-end proof. Compiling this file runs, inside `rustc`, the
//! whole flow `docs/integration/frontends.md` specifies: the region is parsed,
//! constructed and verified as a public logical program, optimized through
//! `tiler_compiler::session`, emitted as Metal, given a complete artifact
//! identity, looked up in the expansion cache, compiled by `xcrun metal` and
//! `xcrun metallib` on a miss, published atomically, read back, and embedded in
//! this binary as one byte-string literal — after which the produced binary runs
//! and routes those bytes through the loader before taking its fallback.
//!
//! **Every absence the accepted inline developer experience names is checkable
//! by reading this crate**, which is the whole of what a consumer writes:
//!
//! - no `build.rs` — this crate has one source file and a manifest;
//! - no registry — the adapter below is a type, and nothing global learns it
//!   exists;
//! - no source scan — the expansion sees the tokens of its own invocation and
//!   nothing else;
//! - no Cargo subcommand and no prepare step — `cargo build` is the whole
//!   procedure;
//! - no runtime source JIT — nothing here compiles anything at run time; the
//!   Metal source was compiled while this file was.
//!
//! **What it deliberately does not exercise is the storage seam.** The adapter
//! below declines a device by refusing to bind an execution context, which keeps
//! this file about the *producer* half — compilation, identity, caching, and
//! embedding — and leaves the consumer half to
//! `inline_region_dispatches.rs`, whose adapter is handed the region's own bytes
//! and drives the loader's comparisons. Both are needed: this one would still
//! pass if the seam handed over nothing, and that one would still pass if the
//! artifact were a fixture rather than a compilation.
//!
//! # This costs a Metal compilation on a cold cache
//!
//! Deliberately. A fixture that avoided the driver would prove something about a
//! fixture. The second build of the same subject — this file's, or anyone
//! else's, in any process — is a validated cache hit that compiles nothing.

use tiler::runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler::runtime::load::{
    ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight, RoutedDispatch,
    RoutedEntry, TargetPropertyRequest,
};
use tiler::value::{
    AdapterCapability, DispatchAdapter, RegionRequest, ResultRequest, StorageScalar, Tensor,
    TensorAdapter, ValueMetadata,
};

/// The consumer's own tensor value. Tiler never learns what it is.
#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
    bytes: Vec<u8>,
}

impl Buffer {
    fn dense(scalar: StorageScalar, extents: Vec<u64>) -> Self {
        let elements: u64 = extents.iter().product();
        let width = match scalar {
            StorageScalar::U8 => 1,
            StorageScalar::F32 => 4,
        };
        let len = usize::try_from(elements * width).expect("a region extent fits a usize");
        Self {
            scalar,
            extents,
            bytes: vec![0; len],
        }
    }
}

/// The consumer's own error.
#[derive(Debug)]
struct Refused(&'static str);

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for Refused {}

/// A type, not a registration: nothing global learns it exists.
struct Toy;

impl TensorAdapter for Toy {
    type Value = Buffer;
    type Context = ();
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

    fn build(_: &(), request: &ResultRequest<'_>) -> Result<Buffer, Refused> {
        Ok(Buffer::dense(
            request.storage_scalar(),
            request.extents().to_vec(),
        ))
    }
}

/// A device authority that declines to bind one.
///
/// The correct answer for a consumer that links no backend, and the reason this
/// file's route still ends on the semantic fallback: a refusal at the first
/// stage arrives before the routing commit, so ADR 0051 permits the fallback and
/// the region produces its declared result.
struct NoBackend;

impl RuntimeAdapter for NoBackend {
    type Refusal = Refused;
    type Failure = Refused;
    type Completion = ();

    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Refused> {
        Err(Refused("this consumer links no backend"))
    }

    fn validate_payload(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedEntry<'_>,
    ) -> Result<(), Refused> {
        unreachable!("no context was bound")
    }

    fn observe_live_device(
        &mut self,
        _: &LiveExecutionContext,
        _: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        unreachable!("no context was bound")
    }

    fn prepare_entries(
        &mut self,
        _: &LiveExecutionContext,
        _: &[RoutedEntry<'_>],
    ) -> Result<(), Refused> {
        unreachable!("no context was bound")
    }

    fn observe_prepared_entry(
        &mut self,
        _: &LiveExecutionContext,
        _: TargetPropertyRequest<'_>,
    ) -> u64 {
        unreachable!("no context was bound")
    }

    fn plan_dispatch(&mut self, _: &LiveExecutionContext, _: &Preflight<'_>) -> Result<(), Refused> {
        unreachable!("no context was bound")
    }

    fn allocate_dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedDispatch<'_>,
    ) -> Result<(), Refused> {
        unreachable!("no route ever committed")
    }

    fn dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedDispatch<'_>,
    ) -> Result<(), Refused> {
        unreachable!("no route ever committed")
    }
}

impl DispatchAdapter for Toy {
    type Refusal = Refused;
    type Failure = Refused;
    type Dispatch<'region> = NoBackend;

    fn storage(value: &Buffer) -> Result<&[u8], Refused> {
        Ok(&value.bytes)
    }

    fn storage_mut(value: &mut Buffer) -> Result<&mut [u8], Refused> {
        Ok(&mut value.bytes)
    }

    fn dispatcher(_: &(), _: RegionRequest<'_>) -> Result<NoBackend, Refused> {
        Ok(NoBackend)
    }
}

fn operand() -> Tensor<Toy> {
    Tensor::new(Buffer::dense(StorageScalar::F32, vec![4]), ())
}

fn main() {
    let (a, b, c) = (operand(), operand(), operand());

    // The approved region, with the accepted `deliver` spelling. Every extent
    // is literal because a selected family is compiled ahead of time, and a
    // symbolic extent has no shape to compile against.
    let d = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        deliver macos;
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    };

    assert_eq!(
        d.expect("the operands honour the region's declared interface"),
        Buffer::dense(StorageScalar::F32, vec![4]),
    );

    // The same region without the statement is `fallback-only`, compiles no
    // artifact at all, and produces the same value. The pair is what makes the
    // delivery an *optimization* of an available semantic computation rather
    // than a different computation.
    let (a, b, c) = (operand(), operand(), operand());
    let plain = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    };
    assert_eq!(
        plain.expect("the fallback region binds"),
        Buffer::dense(StorageScalar::F32, vec![4]),
    );

    // The second whole-program shape a region can denote: a reduction. It runs
    // the same eight steps and costs a second Metal compilation on a cold cache,
    // and that is what it is here to prove — that a region a consumer *writes*
    // reaches a shape the compiler recognizes, rather than only a program a test
    // assembled by hand. Both extents are 2 because the bound declaration's
    // measured grid-axis capacity is four threads.
    let x: Tensor<Toy> = Tensor::new(Buffer::dense(StorageScalar::F32, vec![2, 2]), ());
    let summed = tiler::tensor! {
        in x: f32[rows: 2, cols: 2];
        deliver macos;
        contract flush_subnormals_to_zero_f32;
        out strict_serial_sum(x * 2.0 + 1.0, [cols])
    };
    assert_eq!(
        summed.expect("the operand honours the region's declared interface"),
        Buffer::dense(StorageScalar::F32, vec![2]),
        "the region's result is `f32[rows]`, one rank below its operand",
    );
}
