mod validation;

use validation::{SpecVersion, Validate, ValidationContext, ValidationError, ValidationResult};

fn validate_timestamp(input: &str) -> Result<(), validation::ValidationError> {
    if input.contains("a") {
        return Err(ValidationError::new("timestamp contains char 'a'"));
    }

    Ok(())
}

fn validate_string(input: &str) -> Result<(), validation::ValidationError> {
    if input.len() > 4 {
        return Err(ValidationError::new("String is too large"));
    }

    Ok(())
}

fn validate_vendor(_input: &str) -> Result<(), validation::ValidationError> {
    Ok(())
}

fn validate_toolkind(kind: &ToolKind) -> Result<(), validation::ValidationError> {
    if matches!(kind, ToolKind::Hammer) {
        return Err(ValidationError::new("Tool must not be a hammer"));
    }
    Ok(())
}

#[derive(Debug)]
pub enum ToolKind {
    Hammer,
    ScrewDriver,
}

#[derive(Debug)]
pub struct Tool {
    pub vendor: Option<String>,
    pub name: Option<String>,
    pub kind: ToolKind,
}

impl Validate for Tool {
    fn validate(&self, _version: validation::SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("vendor", self.vendor.as_deref(), validate_vendor)
            .add_field("name", self.name.as_deref(), validate_string)
            .add_enum("kind", &self.kind, validate_toolkind)
            .into()
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub timestamp: Option<String>,
    pub tools: Option<Vec<Tool>>,
}

impl Validate for Metadata {
    fn validate(&self, version: SpecVersion) -> ValidationResult {
        let mut builder =
            ValidationContext::new().add_list("tools", self.tools.as_deref(), |tools: &[_]| {
                tools
                    .into_iter()
                    .map(|tool| tool.validate(version))
                    .collect()
            });

        match version {
            SpecVersion::V1_4 => {
                builder =
                    builder.add_field("timestamp", self.timestamp.as_deref(), validate_string);
            }
            _ => {
                builder =
                    builder.add_field("timestamp", self.timestamp.as_deref(), validate_timestamp);
            }
        }

        builder.into()
    }
}

#[derive(Debug)]
pub struct Bom {
    /// required field
    pub serial_number: String,
    /// optional field
    //pub serial_number: Option<String>,
    pub meta_data: Option<Metadata>,
}

/// The implementation should be easy to digest
impl Validate for Bom {
    fn validate(&self, version: validation::SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field(
                "serial_number",
                self.serial_number.as_ref(),
                validate_string,
            )
            .add_struct("meta_data", self.meta_data.as_ref(), |metadata: &_| {
                metadata.validate(version)
            })
            .into()

        /*
        ValidationContext::new()
            .add_field(pass_arg!("serial_number", validate_string))
            .add_struct(pass_arg!("meta_data", version))
            .into()
         */
    }
}

/// Validates the bom according to a given [`SpecVersion`].
pub fn validate_bom(version: SpecVersion, bom: Bom) -> ValidationResult {
    bom.validate(version)
}

#[cfg(test)]
mod tests {
    use crate::{validate_bom, validation::SpecVersion, Bom, Metadata, Tool, ToolKind};

    #[test]
    fn validate_succeeds() {
        let bom = Bom {
            serial_number: "1234".to_string(),
            meta_data: Some(Metadata {
                timestamp: Some(String::from("2024-01-02")),
                tools: Some(vec![Tool {
                    vendor: Some(String::from("Vendor")),
                    name: Some(String::from("dig")),
                    kind: ToolKind::ScrewDriver,
                }]),
            }),
        };

        assert!(dbg!(validate_bom(SpecVersion::V1_3, bom)).passed());
    }

    #[test]
    fn validate_fails() {
        let bom = Bom {
            serial_number: "1234".to_string(),
            meta_data: Some(Metadata {
                timestamp: Some(String::from("2024-01-02")),
                tools: Some(vec![
                    Tool {
                        vendor: Some(String::from("Vendor")),
                        name: Some(String::from("delv")),
                        kind: ToolKind::ScrewDriver,
                    },
                    Tool {
                        vendor: Some(String::from("Vendor")),
                        name: Some(String::from("dig")),
                        kind: ToolKind::Hammer,
                    },
                ]),
            }),
        };

        assert!(dbg!(validate_bom(SpecVersion::V1_4, bom)).has_errors());
    }
}
