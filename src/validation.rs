use std::collections::BTreeMap;

use indexmap::{map::Entry::Vacant, IndexMap};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SpecVersion {
    V1_3,
    V1_4,
    V1_5,
}

/// Contains all collected validation errors.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Passed,
    Error(ValidationErrors),
}

impl Default for ValidationResult {
    fn default() -> Self {
        ValidationResult::Passed
    }
}

impl ValidationResult {
    pub fn passed(&self) -> bool {
        matches!(self, ValidationResult::Passed)
    }

    pub fn has_errors(&self) -> bool {
        matches!(self, ValidationResult::Error(_))
    }

    pub fn errors(&self) -> Option<&ValidationErrors> {
        match self {
            ValidationResult::Passed => None,
            ValidationResult::Error(errors) => Some(errors),
        }
    }
}

impl From<ValidationResult> for ValidationErrors {
    fn from(value: ValidationResult) -> Self {
        match value {
            ValidationResult::Passed => ValidationErrors::default(),
            ValidationResult::Error(errors) => errors,
        }
    }
}

#[derive(Debug)]
pub struct ValidationContext {
    state: ValidationResult,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            state: ValidationResult::default(),
        }
    }

    pub fn add_field<T>(
        self,
        field_name: &str,
        field: impl Into<Option<T>>,
        validation: impl FnOnce(T) -> Result<(), ValidationError>,
    ) -> Self {
        if let Some(Err(error)) = field.into().map(validation) {
            Self {
                state: ValidationErrors::merge_field(self.state, field_name, error),
            }
        } else {
            self
        }
    }

    pub fn add_enum<T>(
        self,
        enum_name: &str,
        r#enum: impl Into<Option<T>>,
        validation: impl FnOnce(T) -> Result<(), ValidationError>,
    ) -> Self {
        if let Some(Err(error)) = r#enum.into().map(validation) {
            Self {
                state: ValidationErrors::merge_enum(self.state, enum_name, error),
            }
        } else {
            self
        }
    }

    pub fn add_list<'a, T, I>(
        self,
        list_name: &str,
        list: T,
        validation: impl Fn(&'a I) -> ValidationResult,
    ) -> Self
    where
        I: 'a,
        T: IntoIterator<Item = &'a I>,
    {
        let children = list.into_iter().map(validation).collect::<Vec<_>>();
        Self {
            state: ValidationErrors::merge_list(self.state, list_name, children),
        }
    }

    pub fn add_struct<T>(
        self,
        struct_name: &str,
        r#struct: impl Into<Option<T>>,
        validation: impl FnOnce(T) -> ValidationResult,
    ) -> Self {
        if let Some(ValidationResult::Error(validation_errors)) = r#struct.into().map(validation) {
            Self {
                state: ValidationErrors::merge_struct(self.state, struct_name, validation_errors),
            }
        } else {
            self
        }
    }
}

impl From<ValidationContext> for ValidationResult {
    fn from(builder: ValidationContext) -> Self {
        builder.state
    }
}

/// The trait that SBOM structs need to implement to validate their content.
pub trait Validate {
    fn validate(&self, version: SpecVersion) -> ValidationResult;
}

/// A single validation error with a message, useful to log / display for user.
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

/// Implements possible hierarchy of a structured SBOM to collect all [`ValidationError`] in.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorsKind {
    /// Collects all field validation errors in context of a struct
    Struct(ValidationErrors),
    /// Collects all child elements in context of a list, e.g. `Vec`
    List(BTreeMap<usize, ValidationErrors>),
    /// Contains the list of validation errors for a single field, e.g. struct field.
    Field(Vec<ValidationError>),
    /// Represents a single error for an Enum variant.
    Enum(ValidationError),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidationErrors {
    /// Maps a name to a set of context errors.
    pub(crate) inner: IndexMap<String, ValidationErrorsKind>,
}

#[allow(dead_code)]
impl ValidationErrorsKind {
    pub(crate) fn r#enum(error: &str) -> Self {
        Self::Enum(ValidationError::new(error))
    }

    pub(crate) fn list(errors: &[(usize, ValidationErrors)]) -> Self {
        let errors = errors
            .into_iter()
            .map(|(index, value)| (*index, value.clone()))
            .collect::<BTreeMap<_, _>>();

        Self::List(errors)
    }

    pub(crate) fn r#struct(errors: &[(&str, ValidationErrorsKind)]) -> Self {
        let errors = errors
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect::<IndexMap<_, _>>();

        Self::Struct(ValidationErrors { inner: errors })
    }
}

/// TODO remove again
impl From<Vec<(&str, ValidationErrorsKind)>> for ValidationErrors {
    fn from(errors: Vec<(&str, ValidationErrorsKind)>) -> Self {
        ValidationErrors {
            inner: errors
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect::<IndexMap<_, _>>(),
        }
    }
}

impl ValidationErrors {
    /// Generates an empty list of errors.
    pub fn new() -> Self {
        ValidationErrors {
            inner: IndexMap::new(),
        }
    }

    /// Returns [`ValidationErrors`] with possible validation error
    pub fn merge_field(
        parent: ValidationResult,
        field_name: &str,
        validation_error: ValidationError,
    ) -> ValidationResult {
        let mut errors: ValidationErrors = parent.into();
        errors.add_field(field_name, validation_error);
        ValidationResult::Error(errors)
    }

    /// Returns new [`ValidationErrors`] with possible validation error for enum.
    pub fn merge_enum(
        parent: ValidationResult,
        enum_name: &str,
        validation_error: ValidationError,
    ) -> ValidationResult {
        let mut errors: ValidationErrors = parent.into();
        errors.add_enum(enum_name, validation_error);
        ValidationResult::Error(errors)
    }

    /// Returns new [`ValidationErrors`] with results for all nested fields.
    pub fn merge_struct(
        parent: ValidationResult,
        struct_name: &str,
        validation_errors: ValidationErrors,
    ) -> ValidationResult {
        let mut errors: ValidationErrors = parent.into();
        errors.add_nested(struct_name, ValidationErrorsKind::Struct(validation_errors));
        ValidationResult::Error(errors)
    }

    pub fn merge_list(
        parent: ValidationResult,
        field_name: &str,
        children: Vec<ValidationResult>,
    ) -> ValidationResult {
        let child_errors = children
            .into_iter()
            .enumerate()
            .filter_map(|(index, result)| match result {
                ValidationResult::Passed => None,
                ValidationResult::Error(errors) => Some((index, errors)),
            })
            .collect::<BTreeMap<_, _>>();

        if child_errors.is_empty() {
            parent
        } else {
            let mut errors: ValidationErrors = parent.into();
            errors.add_nested(field_name, ValidationErrorsKind::List(child_errors));
            ValidationResult::Error(errors)
        }
    }

    /// Adds a nested object kind
    fn add_nested(&mut self, nested_name: &str, errors_kind: ValidationErrorsKind) {
        if let Vacant(entry) = self.inner.entry(nested_name.to_string()) {
            entry.insert(errors_kind);
        } else {
            panic!("Attempt to replace non-empty nested entry")
        }
    }

    /// Adds a single [`ValidationError`] for an enum variant.
    fn add_enum(&mut self, enum_name: &str, validation_error: ValidationError) {
        if let Vacant(entry) = self.inner.entry(enum_name.to_string()) {
            entry.insert(ValidationErrorsKind::Enum(validation_error));
        } else {
            panic!("Attempt to replace non-empty enum entry")
        }
    }

    /// Adds a single field [`ValidationError`].
    fn add_field(&mut self, field_name: &str, validation_error: ValidationError) {
        if let ValidationErrorsKind::Field(ref mut vec) = self
            .inner
            .entry(field_name.to_string())
            .or_insert_with(|| ValidationErrorsKind::Field(vec![]))
        {
            vec.push(validation_error);
        } else {
            panic!("Found a non-field ValidationErrorsKind");
        }
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

    #[test]
    fn is_empty() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());

        errors.add_field("hello", ValidationError::new("again"));
        assert!(!errors.is_empty());
    }
}
