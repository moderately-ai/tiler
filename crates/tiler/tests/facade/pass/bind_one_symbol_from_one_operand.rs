//! An out-of-tree consumer supplies its own adapter and binds one symbol from
//! one operand, then receives the declared result as its own value.
//!
//! The region this stands in for is:
//!
//! ```text
//! sym n;
//! in a: f32[n];
//! out d: f32[n, 2]
//! ```
//!
//! The `FACTS` constant below is **byte-identical** to what
//! `tiler_macros::binding` emits for that region. It is written out by hand
//! because `tensor!` has no grammar yet, and it is kept honest by
//! `the_single_operand_facts_are_what_the_facade_fixture_compiles` in the macro
//! crate, which reads this file and asserts it contains the emitter's exact
//! output. Nothing else in this file is generated: the adapter is what an
//! arbitrary integration writes, and no part of `tiler` had to change to accept
//! it.

use tiler::value::{
    AdapterCapability, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata,
};

/// This consumer's own tensor. `tiler` never learns what is in it.
#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
    elements: Vec<f32>,
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
        // Exhaustive with no wildcard: a capability added to the profile is a
        // build error here rather than a silent decline.
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
        let count = request
            .extents()
            .iter()
            .try_fold(1_u64, |total, extent| total.checked_mul(*extent))
            .ok_or(Refused("the requested element count overflows"))?;
        let count = usize::try_from(count)
            .map_err(|_| Refused("the requested element count is not addressable"))?;
        Ok(Buffer {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
            elements: vec![0.0; count],
        })
    }
}

// Emitter output, verbatim. The macro crate's comparison asserts this file
// contains that text byte for byte, so reflowing this constant would break the
// binding between the two ends rather than tidy it.
#[rustfmt::skip]
const FACTS: ::tiler::__private::RegionFacts = ::tiler::__private::RegionFacts { operands: &[::tiler::__private::OperandFacts { key: "a", storage_scalar: ::tiler::value::StorageScalar::F32, extents: &[::tiler::__private::OperandExtent::Symbolic] }], symbols: &[::tiler::__private::SymbolFacts { name: "n", source: ::tiler::__private::AxisRef { operand: 0usize, axis: 0usize }, obligations: &[] }], capabilities: &[::tiler::value::AdapterCapability::DenseRowMajorStorage, ::tiler::value::AdapterCapability::ResultConstruction], result: ::tiler::__private::ResultFacts { key: "d", storage_scalar: ::tiler::value::StorageScalar::F32, axes: &[::tiler::__private::ResultAxis::Symbol(0usize), ::tiler::__private::ResultAxis::Literal(2u64)] } };

fn main() {
    let a = Tensor::<Toy>::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![3],
            elements: vec![0.0; 3],
        },
        (),
    );

    let bound = ::tiler::__private::bind_region::<Toy>(&FACTS, &[&a]).expect("the operand matches");
    assert_eq!(bound.values(), [3]);

    let result = ::tiler::__private::build_result::<Toy>(&FACTS, &bound, a.context())
        .expect("the adapter constructs results");
    assert_eq!(
        result,
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![3, 2],
            elements: vec![0.0; 6],
        },
        "the declared result is `f32[n, 2]` with `n` bound from `a` axis 0",
    );
}
