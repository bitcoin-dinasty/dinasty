use std::io::Read;

use bitcoin::psbt::PartiallySignedTransaction;

use crate::{commands::Commands, psbts_serde};

pub struct StdinData(Vec<u8>);

#[derive(thiserror::Error, Debug)]
pub enum StdinError {
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("One text line expected in stdin, found {0}")]
    Not1Lines(usize),

    #[error(transparent)]
    DecodeError(#[from] psbts_serde::DecodeError),
}

impl StdinData {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn to_single_text_line(self) -> Result<String, StdinError> {
        let mut vec = self.to_multiline_string()?;
        if vec.len() != 1 {
            return Err(StdinError::Not1Lines(vec.len()));
        }
        Ok(vec.pop().expect("length checked"))
    }
    pub fn to_string(self) -> Result<String, StdinError> {
        Ok(String::from_utf8(self.0)?)
    }

    pub fn to_psbts(&self) -> Result<Vec<PartiallySignedTransaction>, StdinError> {
        Ok(psbts_serde::deserialize(&self.0)?)
    }

    pub fn to_multiline_string(self) -> Result<Vec<String>, StdinError> {
        let string = self.to_string()?;
        Ok(string
            .split("\n")
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .collect())
    }
    pub fn to_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for StdinData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub fn read_stdin() -> StdinData {
    let mut stdin = std::io::stdin().lock();
    let mut result = vec![];
    stdin.read_to_end(&mut result).expect("error reading stdin");
    StdinData(result)
}

impl Commands {
    pub fn needs_stdin(&self) -> bool {
        match self {
            Commands::Locktime { .. }
            | Commands::Refresh { .. }
            | Commands::Encrypt { .. }
            | Commands::GenerateCompletion { .. } => false,
            _ => true,
        }
    }
}
