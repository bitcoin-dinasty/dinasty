use std::{io::Read, iter};

use age::{armor::ArmoredReader, x25519::Identity};

#[derive(thiserror::Error, Debug)]
pub enum DecryptError {
    #[error(transparent)]
    Age(#[from] age::DecryptError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Passphrase based encryption is not supported, only recipient based")]
    PassphraseNotSupported,
}

pub fn decrypt(armored_cipher_text: &str, identity: &Identity) -> Result<String, DecryptError> {
    let reader = ArmoredReader::new(armored_cipher_text.as_bytes());

    let decryptor = match age::Decryptor::new(reader)? {
        age::Decryptor::Recipients(d) => d,
        age::Decryptor::Passphrase(_) => return Err(DecryptError::PassphraseNotSupported),
    };
    let mut decrypted = vec![];
    let mut reader = decryptor.decrypt(iter::once(identity as &dyn age::Identity))?;
    reader.read_to_end(&mut decrypted)?;
    let result = String::from_utf8(decrypted)?;
    Ok(result)
}

#[cfg(test)]
mod test {

    use crate::commands::{self, decrypt, encrypt, Seed};
    use age::x25519::Identity;
    use std::str::FromStr;

    pub const CODEX_32: &str =
        "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd";

    fn mock_identity() -> Identity {
        let expected = CODEX_32;

        let seed = Seed::from_str(expected).unwrap();
        commands::identity(&seed).unwrap()
    }

    #[test]
    fn roundtrip() {
        let identity = mock_identity();
        let data = "ciao mamma";
        let encrypted = encrypt(data.as_bytes(), vec![identity.to_public()]).unwrap();
        assert_eq!(
            "-----BEGIN AGE ENCRYPTED FILE-----",
            encrypted.lines().next().unwrap()
        );
        assert_eq!(
            "-----END AGE ENCRYPTED FILE-----",
            encrypted.lines().last().unwrap()
        );

        println!("{encrypted}");

        let decrypted = decrypt(&encrypted, &identity).unwrap();

        assert_eq!(data, decrypted);
    }
}
