//! One symbol unified from two operands: one axis sources it, the other owes an
//! equality, and the region returns the declared result.
//!
//! The region this stands in for is:
//!
//! ```text
//! sym n;
//! in a: f32[n], b: f32[n];
//! out d: f32[n]
//! ```
//!
//! `FACTS` is byte-identical to what `tiler_macros::binding` emits for it, and
//! `the_emitted_facts_are_what_the_facade_fixtures_compile` in the macro crate
//! reads this file to keep that true. The obligation on `b` axis 0 is the half
//! that a single-operand fixture cannot show: `sym n` has exactly one root
//! binding, so the second occurrence is a runtime equality rather than a second
//! source.

use tiler::value::{AdapterCapability, BindError, OperandAxis, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata};

#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
}

#[derive(Debug, PartialEq)]
struct Refused(&'static str);

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for Refused {}

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

const FACTS: ::tiler::__private::RegionFacts = ::tiler::__private::RegionFacts { operands: &[::tiler::__private::OperandFacts { key: "a", storage_scalar: ::tiler::value::StorageScalar::F32, extents: &[::tiler::__private::OperandExtent::Symbolic] }, ::tiler::__private::OperandFacts { key: "b", storage_scalar: ::tiler::value::StorageScalar::F32, extents: &[::tiler::__private::OperandExtent::Symbolic] }], symbols: &[::tiler::__private::SymbolFacts { name: "n", source: ::tiler::__private::AxisRef { operand: 0usize, axis: 0usize }, obligations: &[::tiler::__private::AxisRef { operand: 1usize, axis: 0usize }] }], capabilities: &[::tiler::value::AdapterCapability::DenseRowMajorStorage, ::tiler::value::AdapterCapability::ResultConstruction], result: ::tiler::__private::ResultFacts { key: "d", storage_scalar: ::tiler::value::StorageScalar::F32, axes: &[::tiler::__private::ResultAxis::Symbol(0usize)] } };

fn operand(extent: u64) -> Tensor<Toy> {
    Tensor::<Toy>::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![extent],
        },
        (),
    )
}

fn main() {
    let a = operand(5);
    let b = operand(5);

    let bound = ::tiler::__private::bind_region::<Toy>(&FACTS, &[&a, &b])
        .expect("both operands report the same extent");
    assert_eq!(bound.values(), [5]);

    let result = ::tiler::__private::build_result::<Toy>(&FACTS, &bound, a.context())
        .expect("the adapter constructs results");
    assert_eq!(
        result,
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![5],
        },
    );

    // The obligation is a check, not a comment: one operand disagreeing refuses
    // the region and names both sides.
    let wider = operand(6);
    assert_eq!(
        ::tiler::__private::bind_region::<Toy>(&FACTS, &[&a, &wider])
            .expect_err("`b` axis 0 owes `a` axis 0 an equality"),
        BindError::InconsistentExtent {
            symbol: "n",
            source: OperandAxis {
                input: "a",
                axis: 0,
            },
            source_extent: 5,
            conflicting: OperandAxis {
                input: "b",
                axis: 0,
            },
            conflicting_extent: 6,
        },
    );
}
