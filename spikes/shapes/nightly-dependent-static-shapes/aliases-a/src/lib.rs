//! Static-shape aliases written through literals and constants.

use nightly_shape_api::StaticShape;

/// Public row count used by a structural const argument.
pub const PUBLIC_ROWS: u64 = 2;
const PRIVATE_COLUMNS: u64 = 3;

/// Literal spelling of the common matrix shape.
pub type LiteralMatrix = StaticShape<2, { [2, 3] }>;
/// Equivalent spelling using public and private constants.
pub type ConstMatrix = StaticShape<2, { [PUBLIC_ROWS, PRIVATE_COLUMNS] }>;
/// A reexported alias must retain the same type identity.
pub use ConstMatrix as ReexportedMatrix;
