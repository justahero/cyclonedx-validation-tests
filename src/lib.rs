mod validation;

use validation::{SpecVersion, Validate, ValidationContext, ValidationError, ValidationErrors};

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
    fn validate(&self, _version: validation::SpecVersion) -> Result<(), ValidationErrors> {
        ValidationContext::new()
            .add_field(
                "vendor",
                self.vendor.as_ref().map(|vendor| validate_vendor(&vendor)),
            )
            .add_field(
                "name",
                self.name.as_ref().map(|name| validate_string(&name)),
            )
            .add_enum("kind", Some(validate_toolkind(&self.kind)))
            .into()
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub timestamp: Option<String>,
    pub tools: Option<Vec<Tool>>,
}

impl Validate for Metadata {
    fn validate(&self, version: SpecVersion) -> Result<(), ValidationErrors> {
        let children = self.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|tool| tool.validate(version))
                .collect::<Vec<_>>()
        });

        let mut builder = ValidationContext::new().add_list("tools", children);

        match version {
            SpecVersion::V1_4 => {
                builder = builder.add_field(
                    "timestamp",
                    self.timestamp.as_ref().map(|t| validate_string(t)),
                );
            }
            _ => {
                builder = builder.add_field(
                    "timestamp",
                    self.timestamp.as_ref().map(|t| validate_timestamp(t)),
                );
            }
        }

        builder.into()
    }
}

#[derive(Debug)]
pub struct Bom {
    pub serial_number: Option<String>,
    pub meta_data: Option<Metadata>,
}

/// The implementation should be easy to digest
impl Validate for Bom {
    fn validate(&self, version: validation::SpecVersion) -> Result<(), ValidationErrors> {
        ValidationContext::new()
            .add_field("serial_number", self.serial_number.as_ref().map(|sn| validate_string(sn)))
            .add_struct("meta_data", self.meta_data.as_ref().map(|metadata| metadata.validate(version)))
            .into()
    }
}

/// Validates the bom according to a given [`SpecVersion`].
pub fn validate_bom(version: SpecVersion, bom: Bom) -> Result<(), ValidationErrors> {
    bom.validate(version)
}

#[cfg(test)]
mod tests {
    use crate::{validate_bom, validation::SpecVersion, Bom, Metadata, Tool, ToolKind};

    #[test]
    fn validate_succeeds() {
        let bom = Bom {
            serial_number: Some("1234".to_string()),
            meta_data: Some(Metadata {
                timestamp: Some(String::from("2024-01-02")),
                tools: Some(vec![Tool {
                    vendor: Some(String::from("Vendor")),
                    name: Some(String::from("dig")),
                    kind: ToolKind::ScrewDriver,
                }]),
            }),
        };

        assert!(dbg!(validate_bom(SpecVersion::V1_3, bom)).is_ok());
    }

    #[test]
    fn validate_fails() {
        let bom = Bom {
            serial_number: Some("1234".to_string()),
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

        assert!(dbg!(validate_bom(SpecVersion::V1_4, bom)).is_err());
    }
}
