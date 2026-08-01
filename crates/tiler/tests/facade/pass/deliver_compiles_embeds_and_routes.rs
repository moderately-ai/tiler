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
//! **What it does not do is dispatch.** `tiler::value` publishes no storage
//! access and no device object by accepted design, so a `tiler`-only consumer
//! has nothing to hand a kernel and the route below stops at the first question
//! only a device can answer. That is the honest terminal state today, and it is
//! recorded rather than hidden: the region still produces its declared result,
//! through the semantic fallback, and `RouteOutcome` says why.
//!
//! # This costs a Metal compilation on a cold cache
//!
//! Deliberately. A fixture that avoided the driver would prove something about a
//! fixture. The second build of the same subject — this file's, or anyone
//! else's, in any process — is a validated cache hit that compiles nothing.

use tiler::value::{
    AdapterCapability, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata,
};

/// The consumer's own tensor value. Tiler never learns what it is.
#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
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
        Ok(Buffer {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
        })
    }
}

fn operand() -> Tensor<Toy> {
    Tensor::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![4],
        },
        (),
    )
}

fn main() {
    let (a, b, c) = (operand(), operand(), operand());

    // The approved region, with the accepted `deliver` spelling. Every extent
    // is literal because a selected family is compiled ahead of time, and a
    // symbolic extent has no shape to compile against.
    let d = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        deliver macos;
        out (a * b) + c
    };

    assert_eq!(
        d.expect("the operands honour the region's declared interface"),
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![4],
        },
    );

    // The same region without the statement is `fallback-only`, compiles no
    // artifact at all, and produces the same value. The pair is what makes the
    // delivery an *optimization* of an available semantic computation rather
    // than a different computation.
    let (a, b, c) = (operand(), operand(), operand());
    let plain = tiler::tensor! {
        in a: f32[4], b: f32[4], c: f32[4];
        out (a * b) + c
    };
    assert_eq!(
        plain.expect("the fallback region binds"),
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![4],
        },
    );
}
