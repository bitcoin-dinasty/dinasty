use bitcoin::psbt::PartiallySignedTransaction;
use std::fmt::{Debug, Display};

use crate::psbts_serde::{self, DecodeError};

pub struct StdoutData(Vec<u8>);

#[derive(thiserror::Error, Debug)]
pub enum StdoutError {
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),
}

impl Display for StdoutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match String::from_utf8(self.0.clone()) {
            Ok(s) => f.write_str(&s),
            Err(_) => write!(f, "binary data {:?}", self.0),
        }
    }
}

impl Debug for StdoutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl StdoutData {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn to_psbts(&self) -> Result<Vec<PartiallySignedTransaction>, StdoutError> {
        Ok(psbts_serde::deserialize(&self.0)?)
    }
}

impl AsRef<[u8]> for StdoutData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl PartialEq<String> for StdoutData {
    fn eq(&self, other: &String) -> bool {
        &self.to_string() == other
    }
}

impl PartialEq<&str> for StdoutData {
    fn eq(&self, other: &&str) -> bool {
        &self.to_string() == other
    }
}
