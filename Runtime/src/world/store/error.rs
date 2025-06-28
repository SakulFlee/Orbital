use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum StoreError {
    InvalidIndex { index: u128 },
    NoActiveEntry,
}

impl Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::InvalidIndex { index } => {
                write!(f, "StoreError::InvalidIndex: #{index}")
            }
            StoreError::NoActiveEntry => {
                write!(f, "StoreError::NoActiveCamera")
            }
        }
    }
}

impl Error for StoreError {}
