use std::collections::BTreeMap;

use indexmap::IndexMap;

#[derive(Debug, PartialEq)]
pub enum SpecVersion {
    V1_3,
    V1_4,
    V1_5,
}

pub trait Validate {
    fn validate(&self, version: SpecVersion) -> Result<(), ValidationErrors>;
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
    Struct(Box<ValidationErrors>),
    List(BTreeMap<usize, Box<ValidationErrors>>),
    Field(Vec<ValidationError>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidationErrors {
    inner: IndexMap<String, ValidationErrorsKind>,
}

impl ValidationErrors {
    /// Generates an empty list of errors.
    pub fn new() -> Self {
        ValidationErrors {
            inner: IndexMap::new(),
        }
    }

    pub fn has_error(result: &Result<(), ValidationErrors>, field: &str) -> bool {
        match result {
            Ok(()) => false,
            Err(ref errors) => errors.contains_key(field),
        }
    }

    pub fn add_field(&mut self, field: &str, error: ValidationError) {
        if let ValidationErrorsKind::Field(ref mut vec) = self
            .inner
            .entry(field.to_string())
            .or_insert_with(|| ValidationErrorsKind::Field(vec![]))
        {
            vec.push(error);
        } else {
            panic!("Found a non-field ValidationErrorsKind instance");
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::{ValidationError, ValidationErrors};

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
}
