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
    use std::str::FromStr;

    use age::x25519::Identity;

    use crate::{
        commands::{self, encrypt},
        key_origin::XprvWithSource,
    };

    use super::decrypt;

    fn mock_identity() -> Identity {
        let expected = "[8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g";

        let key_with_source = XprvWithSource::from_str(expected).unwrap();
        commands::identity(&key_with_source, bitcoin::Network::Regtest).unwrap()
    }

    #[test]
    fn roundtrip() {
        let identity = mock_identity();
        let data = "ciao mamma";
        let encrypted = encrypt(data, vec![identity.to_public()]).unwrap();
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
