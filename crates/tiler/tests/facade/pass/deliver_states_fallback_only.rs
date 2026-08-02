//! An out-of-tree consumer states `deliver fallback-only;` and gets the region
//! it would have got by stating nothing.
//!
//! This is the accepted profile production compiled by a real consumer, and it
//! is the only spelling of the statement that reaches an expansion today: every
//! other profile and every family list selects an artifact family, and nothing
//! compiles one yet, so those are refusals rather than pass cases. The
//! compile-fail fixture beside this file is where they are pinned.
//!
//! The claim is *equivalence*, not merely acceptance. `fallback-only` is what a
//! region stating no `deliver` already resolves to, so the two must produce the
//! same value from the same operands; a statement that changed anything about
//! the region would break the contract that `FallbackOnly` "performs no backend
//! compiler work". The two regions below differ in exactly that one statement.

use tiler::value::{
    AdapterCapability, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata,
};

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

fn f32_tensor(extents: &[u64]) -> Tensor<Toy> {
    Tensor::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: extents.to_vec(),
        },
        (),
    )
}

fn main() {
    let (a, b, c) = (f32_tensor(&[5]), f32_tensor(&[5]), f32_tensor(&[5]));

    let stated = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n], c: f32[n];
        deliver fallback-only;
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    };
    let unstated = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n], c: f32[n];
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    };

    let stated = stated.expect("every operand reports extent 5");
    assert_eq!(
        stated,
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![5],
        },
        "a `fallback-only` region runs the semantic fallback and produces its declared result",
    );
    assert_eq!(
        stated,
        unstated.expect("every operand reports extent 5"),
        "stating `fallback-only` and stating nothing are the same policy",
    );

    // The statement is a declaration, so it sits with the other declarations
    // rather than in a fixed position among them.
    let reordered = tiler::tensor! {
        deliver fallback-only;
        sym n;
        in a: f32[n], b: f32[n], c: f32[n];
        contract flush_subnormals_to_zero_f32;
        out (a * b) + c
    };
    assert_eq!(
        reordered.expect("every operand reports extent 5"),
        stated,
        "where the statement sits in the declaration block changes nothing",
    );

    let _ = (a, b, c);
}
