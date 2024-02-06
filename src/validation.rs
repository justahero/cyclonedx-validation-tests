use std::collections::BTreeMap;

use indexmap::IndexMap;

#[derive(Debug, PartialEq)]
pub enum SpecVersion {
    V1_3,
    V1_4,
    V1_5,
}

pub trait Validate {
    fn validate(
        &self,
        version: SpecVersion,
        parent: Result<(), ValidationErrors>,
    ) -> Result<(), ValidationErrors>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new<T: ToString>(message: T) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorsKind {
    /// Collects all field validation errors in context of a struct
    Struct(Box<ValidationErrors>),
    /// Collects all child elements in context of a list, e.g. `Vec`
    List(BTreeMap<usize, Box<ValidationErrors>>),
    /// Contains the list of validation errors for a single field, e.g. struct field.
    Field(Vec<ValidationError>),
    /// Represents an error for an Enum variant.
    Enum(ValidationError),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidationErrors {
    /// Maps a name to a set of context errors.
    inner: IndexMap<String, ValidationErrorsKind>,
}

impl ValidationErrors {
    /// Generates an empty list of errors.
    pub fn new() -> Self {
        ValidationErrors {
            inner: IndexMap::new(),
        }
    }

    /// Returns new [`ValidationContext`] with results for all nested fields.
    pub fn merge(
        field: &str,
        child: Result<(), ValidationErrors>,
    ) -> Result<(), ValidationErrors> {
        todo!("")
    }

    /// Adds a single [`ValidationError`] to a named struct field.
    pub fn add_field(&mut self, field: &str, error: ValidationError) {
        if let ValidationErrorsKind::Field(ref mut vec) = self
            .inner
            .entry(field.to_string())
            .or_insert_with(|| ValidationErrorsKind::Field(vec![]))
        {
            vec.push(error);
        } else {
            panic!("Found a non-field ValidationErrorsKind");
        }
    }

    /// Adds a single [`ValidationError`] associated with an enum variant.
    pub fn add_enum(&mut self, variant: &str, error: ValidationError) {
        self.inner
            .entry(variant.to_string())
            .or_insert_with(|| ValidationErrorsKind::Enum(error));
    }

    pub fn has_error(result: &Result<(), ValidationErrors>, field: &str) -> bool {
        match result {
            Ok(()) => false,
            Err(ref errors) => errors.contains_key(field),
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{ValidationErrors, ValidationError};

    #[test]
    fn has_error() {
        let mut errors = ValidationErrors::new();
        errors.add_field("test", ValidationError::new("missing"));
        let result = std::result::Result::Err(errors);

        assert!(ValidationErrors::has_error(&result, "test"));
        assert!(!ValidationErrors::has_error(&result, "nope"));
    }

    #[test]
    fn contains_key() {
        let mut errors = ValidationErrors::new();
        errors.add_field("test", ValidationError::new("missing"));

        assert!(errors.contains_key("test"));
        assert!(!errors.contains_key("haha"));
    }

    #[test]
    fn is_empty() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());

        errors.add_field("hello", ValidationError::new("again"));
        assert!(!errors.is_empty());
    }
}
