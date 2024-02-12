mod validation;

use validation::{SpecVersion, Validate, ValidationError, ValidationErrors};

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

#[derive(Debug)]
pub struct Tool {
    pub vendor: Option<String>,
    pub name: Option<String>,
}

impl Validate for Tool {
    fn validate(&self, _version: validation::SpecVersion) -> Result<(), ValidationErrors> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub timestamp: Option<String>,
    pub tools: Option<Vec<Tool>>,
}

impl Validate for Metadata {
    fn validate(&self, version: SpecVersion) -> Result<(), ValidationErrors> {
        let mut result = std::result::Result::Ok(());

        match version {
            SpecVersion::V1_4 => {
                if let Some(timestamp) = &self.timestamp {
                    result = ValidationErrors::merge_field(
                        result,
                        "timestamp",
                        validate_string(&timestamp),
                    );
                }
            }
            _ => {
                if let Some(timestamp) = &self.timestamp {
                    result = ValidationErrors::merge_field(
                        result,
                        "timestamp",
                        validate_timestamp(&timestamp),
                    );
                }
            }
        }

        result
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
        let mut result = std::result::Result::Ok(());

        if let Some(number) = &self.serial_number {
            result =
                ValidationErrors::merge_field(result, "serial_number", validate_string(&number));
        };

        if let Some(metadata) = &self.meta_data {
            result = ValidationErrors::merge(result, "meta_data", metadata.validate(version));
        }

        result
    }
}

/// Validates the bom according to a given [`SpecVersion`].
pub fn validate_bom(version: SpecVersion, bom: Bom) -> Result<(), ValidationErrors> {
    bom.validate(version)
}

#[cfg(test)]
mod tests {
    use crate::{validate_bom, validation::SpecVersion, Bom, Metadata, Tool};

    #[test]
    fn validate_succeeds() {
        let bom = Bom {
            serial_number: Some("1234".to_string()),
            meta_data: Some(Metadata {
                timestamp: Some(String::from("2024-01-02")),
                tools: Some(vec![Tool {
                    vendor: Some(String::from("Vendor")),
                    name: Some(String::from("dig")),
                }]),
            }),
        };

        assert!(validate_bom(SpecVersion::V1_3, bom).is_ok());
    }

    #[test]
    fn validate_fails() {
        let bom = Bom {
            serial_number: Some("1234".to_string()),
            meta_data: Some(Metadata {
                timestamp: Some(String::from("2024-01-02")),
                tools: Some(vec![Tool {
                    vendor: Some(String::from("Vendor")),
                    name: Some(String::from("dig")),
                }]),
            }),
        };

        assert!(dbg!(validate_bom(SpecVersion::V1_4, bom)).is_err());
    }
}
