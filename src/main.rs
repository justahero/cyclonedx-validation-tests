use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Email(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Phone(pub String);

#[derive(Debug, Validate, Deserialize)]
pub struct OrganizationalContact {
    #[validate(custom = "validate_name")]
    pub name: Option<String>,
    #[validate(custom = "validate_email")]
    pub email: Option<Email>,
    #[validate(custom = "validate_phone")]
    pub phone: Option<Phone>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct Metadata {
    #[validate(custom = "validate_date")]
    pub timestamp: Option<String>,
    #[validate]
    pub authors: Vec<OrganizationalContact>,
}

fn main() {
    println!("Hello, world!");
}

fn validate_date(date: &str) -> Result<(), ValidationError> {
    if !date.eq("2024/02/04") {
        return Err(ValidationError::new("unsupported date format"));
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
    Ok(())
}

fn validate_email(email: &Email) -> Result<(), ValidationError> {
    if email.0 != "lisa@example.com" {
        return Err(ValidationError::new("email unknown"));
    }
    Ok(())
}

fn validate_phone(phone: &Phone) -> Result<(), ValidationError> {
    if !phone.0.starts_with("555") {
        return Err(ValidationError::new("email unknown"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_validates() {
        let metadata = Metadata {
            timestamp: Some("2024-02-04".to_string()),
            authors: vec![
                OrganizationalContact {
                    name: Some("lisa".to_string()),
                    email: Some(Email("lisa@example.com".to_string())),
                    phone: Some(Phone("012345678".to_string())),
                },
                OrganizationalContact {
                    name: Some("bart".to_string()),
                    email: Some(Email("bar@example.com".to_string())),
                    phone: Some(Phone("555-1234".to_string())),
                },
            ],
        };

        let result = metadata.validate();
        dbg!(&result);
        let errors = result.unwrap_err();
        println!("ERRORS: {:#}", errors);
    }
}
