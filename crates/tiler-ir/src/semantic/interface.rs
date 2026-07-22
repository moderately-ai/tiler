use super::error::BuildError;
use std::marker::PhantomData;

use super::handles::{GraphId, Value, ValueId, ValueIndex};

/// Maximum UTF-8 byte length of one stable semantic interface key.
pub const MAX_INTERFACE_KEY_BYTES: usize = 1_024;

/// One of the semantic program's ordered interfaces.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InterfaceKind {
    /// The program input interface.
    Input,
    /// The program output interface.
    Output,
}

impl std::fmt::Display for InterfaceKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => formatter.write_str("input"),
            Self::Output => formatter.write_str("output"),
        }
    }
}

/// A zero-based position in the ordered input interface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputIndex(u32);

impl InputIndex {
    pub(super) fn from_len(len: usize) -> Option<Self> {
        u32::try_from(len).ok().map(Self)
    }

    /// Returns the fixed-width interface position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A stable input-interface key. Display names are deliberately separate.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputKey(String);

impl InputKey {
    /// Creates a nonempty stable input key.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::EmptyInterfaceKey`] for an empty key.
    pub fn new(value: impl AsRef<str>) -> Result<Self, BuildError> {
        let value = value.as_ref();
        validate_interface_key(value, InterfaceKind::Input)?;
        Ok(Self(value.to_owned()))
    }

    /// Validates and retains an already-owned stable input key without copying it.
    ///
    /// # Errors
    ///
    /// Returns a typed error before retaining the string when it is empty or oversized.
    pub fn from_owned(value: String) -> Result<Self, BuildError> {
        validate_interface_key(&value, InterfaceKind::Input)?;
        Ok(Self(value))
    }

    /// Returns the exact key bytes as UTF-8 text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A stable output-interface key. Output order and keys are semantic identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutputKey(String);

impl OutputKey {
    /// Creates a nonempty stable output key.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::EmptyInterfaceKey`] for an empty key.
    pub fn new(value: impl AsRef<str>) -> Result<Self, BuildError> {
        let value = value.as_ref();
        validate_interface_key(value, InterfaceKind::Output)?;
        Ok(Self(value.to_owned()))
    }

    /// Validates and retains an already-owned stable output key without copying it.
    ///
    /// # Errors
    ///
    /// Returns a typed error before retaining the string when it is empty or oversized.
    pub fn from_owned(value: String) -> Result<Self, BuildError> {
        validate_interface_key(&value, InterfaceKind::Output)?;
        Ok(Self(value))
    }

    /// Returns the exact key bytes as UTF-8 text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_interface_key(value: &str, interface: InterfaceKind) -> Result<(), BuildError> {
    if value.is_empty() {
        return Err(BuildError::EmptyInterfaceKey { interface });
    }
    if value.len() > MAX_INTERFACE_KEY_BYTES {
        return Err(BuildError::InterfaceKeyTooLong {
            interface,
            bytes: value.len(),
            limit: MAX_INTERFACE_KEY_BYTES,
        });
    }
    Ok(())
}

/// A transient selector for one output declared on a specific semantic draft.
///
/// The selector survives successful commitment, but is neither serializable nor
/// durable semantic identity. Resolving it against an unrelated program fails
/// closed even when that program has an output with the same key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OutputSelector {
    pub(super) origin: GraphId,
    pub(super) key: OutputKey,
}

/// A typed selector for one output declared on a specific semantic draft.
pub struct Output<T> {
    selector: OutputSelector,
    marker: PhantomData<fn() -> T>,
}

impl<T> Output<T> {
    pub(super) const fn from_verified(selector: OutputSelector) -> Self {
        Self {
            selector,
            marker: PhantomData,
        }
    }

    /// Returns the stable semantic output key.
    #[must_use]
    pub const fn key(&self) -> &OutputKey {
        self.selector.key()
    }

    /// Explicitly erases static type evidence from this selector.
    #[must_use]
    pub fn erase(self) -> OutputSelector {
        self.selector
    }

    pub(super) const fn selector(&self) -> &OutputSelector {
        &self.selector
    }
}

impl<T> Clone for Output<T> {
    fn clone(&self) -> Self {
        Self {
            selector: self.selector.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Output<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("Output")
            .field(&self.selector)
            .finish()
    }
}

impl OutputSelector {
    /// Returns the stable semantic output key carried by this selector.
    #[must_use]
    pub const fn key(&self) -> &OutputKey {
        &self.key
    }
}

#[derive(Clone, Debug)]
pub(super) struct ProgramInput {
    pub(super) key: InputKey,
    pub(super) value: ValueIndex,
}

/// A borrowed entry in the ordered input interface.
#[derive(Clone, Copy, Debug)]
pub struct ProgramInputRef<'a> {
    pub(super) owner: GraphId,
    pub(super) input: &'a ProgramInput,
}

impl ProgramInputRef<'_> {
    /// Returns the stable interface key.
    #[must_use]
    pub fn key(&self) -> &InputKey {
        &self.input.key
    }

    /// Returns the value defined by this input.
    #[must_use]
    pub const fn value(&self) -> ValueId {
        ValueId {
            owner: self.owner,
            index: self.input.value,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ProgramOutput {
    pub(super) key: OutputKey,
    pub(super) value: ValueIndex,
}

/// A borrowed entry in the ordered, named output interface.
#[derive(Clone, Copy, Debug)]
pub struct ProgramOutputRef<'a> {
    pub(super) owner: GraphId,
    pub(super) output: &'a ProgramOutput,
}

/// A borrowed program output with exact registry-checked type evidence.
pub struct TypedProgramOutputRef<'a, T> {
    output: ProgramOutputRef<'a>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for TypedProgramOutputRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TypedProgramOutputRef<'_, T> {}

impl<T> std::fmt::Debug for TypedProgramOutputRef<'_, T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("TypedProgramOutputRef")
            .field(&self.output)
            .finish()
    }
}

impl<'a, T> TypedProgramOutputRef<'a, T> {
    pub(super) const fn from_verified(output: ProgramOutputRef<'a>) -> Self {
        Self {
            output,
            marker: PhantomData,
        }
    }

    /// Returns the stable output key.
    #[must_use]
    pub fn key(&self) -> &OutputKey {
        self.output.key()
    }

    /// Returns the exactly typed output value.
    #[must_use]
    pub const fn value(&self) -> Value<T> {
        Value::from_verified(self.output.value())
    }

    /// Returns the unknown-typed borrowed output view.
    #[must_use]
    pub const fn erase(self) -> ProgramOutputRef<'a> {
        self.output
    }
}

impl ProgramOutputRef<'_> {
    /// Returns the stable interface key.
    #[must_use]
    pub fn key(&self) -> &OutputKey {
        &self.output.key
    }

    /// Returns the value exposed by this output.
    #[must_use]
    pub const fn value(&self) -> ValueId {
        ValueId {
            owner: self.owner,
            index: self.output.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_keys_are_bounded_before_retention() {
        struct BorrowedOnly<'a>(&'a str);
        impl AsRef<str> for BorrowedOnly<'_> {
            fn as_ref(&self) -> &str {
                self.0
            }
        }

        let oversized = "x".repeat(MAX_INTERFACE_KEY_BYTES + 1);
        assert_eq!(
            InputKey::new(BorrowedOnly(&oversized)),
            Err(BuildError::InterfaceKeyTooLong {
                interface: InterfaceKind::Input,
                bytes: MAX_INTERFACE_KEY_BYTES + 1,
                limit: MAX_INTERFACE_KEY_BYTES,
            })
        );
        assert_eq!(
            OutputKey::new(BorrowedOnly(&oversized)),
            Err(BuildError::InterfaceKeyTooLong {
                interface: InterfaceKind::Output,
                bytes: MAX_INTERFACE_KEY_BYTES + 1,
                limit: MAX_INTERFACE_KEY_BYTES,
            })
        );

        let owned = String::from("owned-input");
        let pointer = owned.as_ptr();
        assert_eq!(
            InputKey::from_owned(owned).unwrap().as_str().as_ptr(),
            pointer
        );
        let owned = String::from("owned-output");
        let pointer = owned.as_ptr();
        assert_eq!(
            OutputKey::from_owned(owned).unwrap().as_str().as_ptr(),
            pointer
        );
    }
}
