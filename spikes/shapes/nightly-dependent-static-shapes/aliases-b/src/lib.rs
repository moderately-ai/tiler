//! Independently authored literal and generated static-shape aliases.

use nightly_shape_api::StaticShape;

/// Independently authored literal spelling of the common matrix shape.
pub type LiteralMatrix = StaticShape<2, { [2, 3] }>;
/// Stable-proc-macro output spelling of the same common matrix shape.
pub type MacroMatrix = nightly_shape_macro::static_shape!(2, 3);
