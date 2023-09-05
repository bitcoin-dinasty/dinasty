use std::io::Write;

use age::{
    armor::{ArmoredWriter, Format},
    x25519::Recipient,
};

#[derive(thiserror::Error, Debug)]
pub enum EncryptError {
    #[error(transparent)]
    Age(#[from] age::EncryptError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("At least one recipient is required")]
    NoRecipients,
}

pub fn encrypt(plain_text: &str, recipients: Vec<Recipient>) -> Result<String, EncryptError> {
    let recipients: Vec<_> = recipients
        .into_iter()
        .map(|r| Box::new(r) as Box<dyn age::Recipient + Send>)
        .collect();
    let encryptor =
        age::Encryptor::with_recipients(recipients).ok_or(EncryptError::NoRecipients)?;

    let mut result = vec![];

    let mut armored_writer = ArmoredWriter::wrap_output(&mut result, Format::AsciiArmor)?;

    let mut encryption_writer = encryptor.wrap_output(&mut armored_writer)?;
    encryption_writer.write_all(plain_text.as_bytes())?;
    encryption_writer.finish()?;
    armored_writer.finish()?;

    let result = String::from_utf8(result)?;
    Ok(result)
}
