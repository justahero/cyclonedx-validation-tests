use std::collections::BTreeMap;

use indexmap::{map::Entry::Vacant, IndexMap};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SpecVersion {
    V1_3,
    V1_4,
    V1_5,
}

/// TODO: the `Result` is not meant to be used as shortcut or to raise errors, rather to collect all errors
/// avoid using `?` operator
pub type ValidationResult = Result<(), ValidationErrors>;

#[derive(Debug)]
pub struct ValidationContext {
    state: ValidationResult,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            state: std::result::Result::Ok(()),
        }
    }

    pub fn add_field(self, field_name: &str, error: Option<Result<(), ValidationError>>) -> Self {
        if let Some(Err(error)) = error {
            Self {
                state: ValidationErrors::merge_field(self.state, field_name, Err(error)),
            }
        } else {
            self
        }
    }

    pub fn add_enum(self, enum_name: &str, error: Option<Result<(), ValidationError>>) -> Self {
        if let Some(Err(error)) = error {
            Self {
                state: ValidationErrors::merge_enum(self.state, enum_name, Err(error)),
            }
        } else {
            self
        }
    }

    pub fn add_list(
        self,
        field_name: &str,
        children: Option<Vec<Result<(), ValidationErrors>>>,
    ) -> Self {
        if let Some(children) = children {
            Self {
                state: ValidationErrors::merge_list(self.state, field_name, children),
            }
        } else {
            self
        }
    }

    pub fn add_struct(
        self,
        struct_name: &str,
        errors: Option<Result<(), ValidationErrors>>,
    ) -> Self {
        if let Some(Err(errors)) = errors {
            Self {
                state: ValidationErrors::merge_struct(self.state, struct_name, Err(errors)),
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

    /// Returns [`ValidationErrors`] with possible validation error
    pub fn merge_field(
        parent: Result<(), ValidationErrors>,
        field_name: &str,
        error: Result<(), ValidationError>,
    ) -> ValidationResult {
        match error {
            Ok(()) => parent,
            Err(error) => {
                parent
                    .and_then(|_| Err(ValidationErrors::new()))
                    .map_err(|mut parent_errors| {
                        parent_errors.add_field(field_name, error);
                        parent_errors
                    })
            }
        }
    }

    /// Returns new [`ValidationErrors`] with possible validation error for enum.
    pub fn merge_enum(
        parent: Result<(), ValidationErrors>,
        enum_name: &str,
        enum_error: Result<(), ValidationError>,
    ) -> ValidationResult {
        match enum_error {
            Ok(()) => parent,
            Err(error) => {
                parent
                    .and_then(|_| Err(ValidationErrors::new()))
                    .map_err(|mut parent_errors| {
                        parent_errors.add_enum(enum_name, error);
                        parent_errors
                    })
            }
        }
    }

    /// Returns new [`ValidationErrors`] with results for all nested fields.
    pub fn merge_struct(
        parent: Result<(), ValidationErrors>,
        struct_name: &str,
        child: Result<(), ValidationErrors>,
    ) -> ValidationResult {
        match child {
            Ok(()) => parent,
            Err(errors) => {
                parent
                    .and_then(|_| Err(ValidationErrors::new()))
                    .map_err(|mut parent_errors| {
                        parent_errors.add_nested(
                            struct_name,
                            ValidationErrorsKind::Struct(Box::new(errors)),
                        );
                        parent_errors
                    })
            }
        }
    }

    pub fn merge_list(
        parent: Result<(), ValidationErrors>,
        field_name: &str,
        children: Vec<Result<(), ValidationErrors>>,
    ) -> ValidationResult {
        let child_errors = children
            .into_iter()
            .enumerate()
            .filter_map(|(index, result)| result.err().map(|errors| (index, Box::new(errors))))
            .collect::<BTreeMap<_, _>>();

        if child_errors.is_empty() {
            parent
        } else {
            parent
                .and_then(|_| Err(ValidationErrors::new()))
                .map_err(|mut parent_errors| {
                    parent_errors.add_nested(field_name, ValidationErrorsKind::List(child_errors));
                    parent_errors
                })
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
