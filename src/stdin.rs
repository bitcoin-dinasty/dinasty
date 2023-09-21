use std::{io::Read, path::Path};

use crate::commands::Commands;

pub struct StdinData(Vec<u8>);

#[derive(thiserror::Error, Debug)]
pub enum StdinError {
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("One text line expected in stdin, found {0}")]
    Not1Lines(usize),
}

impl StdinData {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn to_single_text_line(self) -> Result<String, StdinError> {
        let string = String::from_utf8(self.0)?;
        let iter = string.split("\n");
        let len = iter.count();
        if len != 1 {
            return Err(StdinError::Not1Lines(len));
        }
        let first = string.split("\n").next().expect("length checked");
        Ok(first.to_string())
    }
    pub fn to_string(self) -> Result<String, StdinError> {
        Ok(String::from_utf8(self.0)?)
    }
    pub fn to_multiline_string(self) -> Result<Vec<String>, StdinError> {
        let string = self.to_string()?;
        Ok(string.split("\n").map(ToString::to_string).collect())
    }
    pub fn to_vec(self) -> Vec<u8> {
        self.0
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
            Commands::Qr { file, .. } => file == Path::new("-"),
            Commands::Bech32 { file, .. } => file == Path::new("-"),
            _ => true,
        }
    }
}