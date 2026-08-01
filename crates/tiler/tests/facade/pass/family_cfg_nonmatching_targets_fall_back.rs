//! A consumer target matching no selected artifact family compiles the semantic
//! fallback, and a retained diagnostic for a family it is not does not break it.
//!
//! The plan this stands in for selects the iOS device family, which built, and
//! the iOS simulator family, whose toolchain was missing. On the macOS host that
//! compiles this file neither predicate holds, so:
//!
//! - the simulator's retained `compile_error!` is removed by `#[cfg]` rather than
//!   failing an unrelated target, which is what
//!   `docs/integration/frontends.md` means by "does not break an unrelated
//!   fallback-only target";
//! - the selector takes its `not(any(…))` arm and resolves to `None`, so no
//!   iOS payload is offered to a macOS consumer — the outcome
//!   `docs/backends/metal.md` forbids and which nothing downstream would catch,
//!   because an `air64-apple-ios16.0` metallib loads and dispatches on the macOS
//!   host GPU without error;
//! - the region still binds and builds through the semantic fallback.
//!
//! The block below is written the way an expansion writes it: the delivery items
//! are at column zero because generated tokens carry no indentation, and they are
//! **byte-identical** to what `tiler_macros::delivery::DeliveryPlan::items_source`
//! emits for that plan. `the_nonmatching_fixture_compiles_what_this_emitter_produces`
//! in the macro crate reads this file and asserts it. `FACTS` is likewise the
//! emitter's own text, for the region `sym n; in a: f32[n]; out d: f32[n, 2]`.

use tiler::value::{AdapterCapability, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata};

/// This consumer's own tensor. `tiler` never learns what is in it.
#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
}

/// This consumer's own error. `tiler` carries it and never replaces it.
#[derive(Debug)]
struct Refused(&'static str);

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for Refused {}

/// The adapter. A type, not a registration: nothing global learns it exists.
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
        Ok(ValueMetadata::new(value.scalar, value.extents.iter().copied()))
    }

    fn build(_: &(), request: &ResultRequest<'_>) -> Result<Buffer, Refused> {
        Ok(Buffer {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
        })
    }
}

const FACTS: ::tiler::__private::RegionFacts = ::tiler::__private::RegionFacts { operands: &[::tiler::__private::OperandFacts { key: "a", storage_scalar: ::tiler::value::StorageScalar::F32, rank: 1usize }], symbols: &[::tiler::__private::SymbolFacts { name: "n", source: ::tiler::__private::AxisRef { operand: 0usize, axis: 0usize }, obligations: &[] }], capabilities: &[::tiler::value::AdapterCapability::DenseRowMajorStorage, ::tiler::value::AdapterCapability::ResultConstruction], result: ::tiler::__private::ResultFacts { key: "d", storage_scalar: ::tiler::value::StorageScalar::F32, axes: &[::tiler::__private::ResultAxis::Symbol(0usize), ::tiler::__private::ResultAxis::Literal(2u64)] } };

fn main() {
    let a = Tensor::<Toy>::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![3],
        },
        (),
    );

    let result = {
#[cfg(all(target_os = "ios", target_abi = "sim"))]
const _: () = { ::core::compile_error!("xcrun: error: unable to find utility \"metal\""); };
const __TILER_ARTIFACT: &[u8] = b"tiler-artifact-envelope";
#[cfg(all(target_os = "ios", target_abi = ""))]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::Some(0usize);
#[cfg(not(any(all(target_os = "ios", target_abi = ""))))]
const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = ::core::option::Option::None;
        assert_eq!(
            __TILER_SELECTED_PAYLOAD, None,
            "a macOS consumer matches no selected iOS family and must take the fallback",
        );
        assert_eq!(
            __TILER_ARTIFACT, b"tiler-artifact-envelope",
            "the one envelope is embedded unconditionally; only the payload choice is gated",
        );
        ::tiler::__private::bind_and_build(&FACTS, &[&a])
    };

    assert_eq!(
        result.expect("the operand matches the region's interface"),
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![3, 2],
        },
        "the semantic fallback produced the declared result",
    );
}
