use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum LoaderError {
    NotDoneProcessing,
    EmptyResult,
    DisconnectedChannelNoData,
}

impl Error for LoaderError {}

impl Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::NotDoneProcessing => write!(f, "Loader is not yet done processing"),
            LoaderError::EmptyResult => {
                write!(f, "The loader worker finished, but no data was returned")
            }
            LoaderError::DisconnectedChannelNoData => {
                write!(f, "The loader worker disconnected without sending any data")
            }
        }
    }
}
