//! A consumer reaches `tensor!` through the facade alone, in both import
//! forms and from a module that imports nothing.
//!
//! The nested module is the part that tests the expansion rather than the
//! re-export: generated tokens spell leading-`::` absolute paths, so they must
//! resolve in a scope where nothing named `tiler` is in scope locally and where
//! none of the facade's own items has been imported.

use tiler::tensor;
use tiler::value::{
    AdapterCapability, BindError, ResultRequest, StorageScalar, Tensor, TensorAdapter,
    ValueMetadata,
};

#[derive(Clone, Debug, PartialEq)]
struct Buffer {
    scalar: StorageScalar,
    extents: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Refused;

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("refused")
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

/// Nothing from `tiler` is imported here, and the region still expands: every
/// path the expansion spells is absolute and rooted at the crate the consumer
/// declared.
mod nested {
    pub fn region(
        a: super::Tensor<super::Toy>,
        b: super::Tensor<super::Toy>,
    ) -> Result<super::Buffer, super::BindError<super::Refused>> {
        tiler::tensor! {
            sym n;
            in a: f32[n], b: f32[n];
            out a * b
        }
    }
}

fn operand(extent: u64) -> Tensor<Toy> {
    Tensor::new(
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![extent],
        },
        (),
    )
}

fn main() {
    let (a, b) = (operand(7), operand(7));

    let imported = tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        out a * b
    };
    let qualified = tiler::tensor! {
        sym n;
        in a: f32[n], b: f32[n];
        out a * b
    };

    assert_eq!(imported, qualified);
    assert_eq!(imported, nested::region(a, b));
    assert_eq!(
        imported.expect("both operands report extent 7"),
        Buffer {
            scalar: StorageScalar::F32,
            extents: vec![7],
        },
    );
}
