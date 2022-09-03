use serde::{Deserialize, Serialize};

/// Returned when an Identifier fails during validation
#[derive(thiserror::Error, Debug, Clone)]
pub enum IdValidationError {
    #[error("ID `{0}` is missing a colon: expected format `namespace:value` (example: `blocky:grass_block`")]
    MissingColon(String),
    
    #[error("ID `{0}` is invalid and contains too many colons: expected format `namespace:value` (example: `blocky:grass_block`")]
    TooManyColons(String),
    
    #[error("ID `{id:?}` contains invalid characters: {invalid_chars:?}")]
    InvalidCharacters {
        id: String,
        invalid_chars: String
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Identifier {
    namespace: String,
    name: String
}

impl Identifier {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: String::from(namespace),
            name: String::from(name)
        }
    }

    pub fn from(id: &String) -> Result<Self, IdValidationError> {
        Self::from_str(&id)
    }

    pub fn from_str(id: &str) -> Result<Self, IdValidationError> {
        let splits: Vec<&str> = id.split(":").collect();

        // there should only be 2 strings; the id's namespace, and the name of the object
        if splits.len() != 2 {
            if splits.len() == 1 {
                Err(IdValidationError::MissingColon(String::from(id)))
            } else {
                Err(IdValidationError::TooManyColons(String::from(id)))
            }
        } else {
            let id = Self::new(splits[0], splits[1]);

            id.validate()?;

            Ok(id)
        }
    }

    pub fn get_namespace(&self) -> String { self.namespace.clone() }
    
    pub fn get_name(&self) -> String { self.name.clone() }

    pub fn as_string(&self) -> String { format!("{}:{}", self.namespace, self.name) }

    pub fn validate(&self) -> Result<(), IdValidationError> {
        let valid_chars = "abcdefghijklmnopqrstuvwxyz0123456789_-.";

        let invalid_chars_namespace = self.namespace.chars().filter(|x| {
            !valid_chars.chars().collect::<Vec<char>>().contains(x)
        }).collect::<String>();

        let invalid_chars_name = self.name.chars().filter(|x| {
            !valid_chars.chars().collect::<Vec<char>>().contains(x)
        }).collect::<String>();
    
        if invalid_chars_namespace.len() > 0 || invalid_chars_name.len() > 0 {
            Err(IdValidationError::InvalidCharacters {
                id: self.as_string(),
                invalid_chars: invalid_chars_namespace + &invalid_chars_name
            })
        } else {
            Ok(())
        }
    }
}