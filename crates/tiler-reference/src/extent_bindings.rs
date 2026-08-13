//! Authenticated extent bindings for one reference evaluation.
//!
//! Derived once from the exact program environment and the declared input
//! tensors. There is no constructor that accepts a second cursor scalar.

use std::collections::BTreeMap;

use tiler_ir::semantic::{InputKey, SemanticProgram};
use tiler_ir::shape::{BindingSource, ShapeSymbol, SourcedExtent};

use super::error::EvaluationError;
use super::tensor::InputBinding;

/// One evaluation's immutable map from declared symbols to authenticated values.
///
/// Constructed only from the exact program and its declared inputs. A callback
/// reads through [`Self::resolve`] and cannot install a competing scalar.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtentBindingContext {
    values: BTreeMap<ShapeSymbol, u64>,
    unsupported: BTreeMap<ShapeSymbol, &'static str>,
}

impl ExtentBindingContext {
    /// An empty context: every symbol is undeclared.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            values: BTreeMap::new(),
            unsupported: BTreeMap::new(),
        }
    }

    /// Derives bindings from the program environment and the ordered inputs.
    ///
    /// [`BindingSource::Static`] and [`BindingSource::InputDimension`] have a
    /// complete authority path here. Every other kind is recorded as a named
    /// refusal and fails only if a later resolve names it.
    pub(crate) fn derive(
        program: &SemanticProgram,
        inputs: &[InputBinding<'_>],
    ) -> Result<Self, EvaluationError> {
        let Some(sources) = program.extent_sources() else {
            return Ok(Self::empty());
        };
        let mut values = BTreeMap::new();
        let mut unsupported = BTreeMap::new();
        for (symbol, binding) in sources.environment().bindings() {
            match binding.source() {
                BindingSource::Static(extent) => {
                    values.insert(symbol.clone(), extent.get());
                }
                BindingSource::InputDimension { input, axis } => {
                    let tensor = input_tensor(program, inputs, input)?;
                    let position = usize::try_from(axis.get())
                        .map_err(|_| EvaluationError::MalformedProgram)?;
                    let extent = tensor
                        .shape()
                        .extents()
                        .get(position)
                        .ok_or(EvaluationError::MalformedProgram)?;
                    values.insert(symbol.clone(), extent.get());
                }
                BindingSource::InterfaceParameter { .. } => {
                    unsupported.insert(symbol.clone(), "interface-parameter");
                }
                BindingSource::TargetProperty { .. } => {
                    unsupported.insert(symbol.clone(), "target-property");
                }
            }
        }
        Ok(Self {
            values,
            unsupported,
        })
    }

    /// Returns the authenticated value this context holds for `extent`.
    ///
    /// # Errors
    ///
    /// Returns [`ExtentBindingError`] when the extent names a symbol this
    /// context does not authenticate.
    pub fn resolve(&self, extent: &SourcedExtent) -> Result<u64, ExtentBindingError> {
        if let Some(value) = extent.as_static() {
            return Ok(value.get());
        }
        let Some(symbol) = extent.symbol() else {
            return Err(ExtentBindingError::UnrecognizedSource);
        };
        if let Some(value) = self.values.get(symbol) {
            return Ok(*value);
        }
        if let Some(kind) = self.unsupported.get(symbol) {
            return Err(ExtentBindingError::Unsupported {
                symbol: symbol.clone(),
                kind,
            });
        }
        Err(ExtentBindingError::Undeclared {
            symbol: symbol.clone(),
        })
    }
}

/// Why one sourced extent could not be resolved in this evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExtentBindingError {
    /// The symbol is not declared by the program's environment.
    Undeclared {
        /// The symbol the extent named.
        symbol: ShapeSymbol,
    },
    /// The symbol's root binding has no authenticated value source here.
    Unsupported {
        /// The symbol the extent named.
        symbol: ShapeSymbol,
        /// The unsupported binding kind.
        kind: &'static str,
    },
    /// A future source kind this evaluator has not been taught.
    UnrecognizedSource,
}

fn input_tensor<'a>(
    program: &SemanticProgram,
    inputs: &'a [InputBinding<'a>],
    key: &InputKey,
) -> Result<&'a super::tensor::Tensor, EvaluationError> {
    for (declaration, binding) in program.inputs().zip(inputs) {
        if declaration.key() == key {
            return Ok(binding.tensor());
        }
    }
    Err(EvaluationError::MalformedProgram)
}

#[cfg(test)]
mod tests {
    use super::ExtentBindingContext;
    use tiler_ir::shape::{Extent, SourcedExtent};

    #[test]
    fn the_empty_context_has_no_caller_supplied_cursor_table() {
        let context = ExtentBindingContext::empty();
        assert_eq!(
            context.resolve(&SourcedExtent::Static(Extent::new(4))),
            Ok(4)
        );
        assert!(context.values.is_empty());
        assert!(context.unsupported.is_empty());
    }
}
