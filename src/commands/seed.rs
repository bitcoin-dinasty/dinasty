use bip39::Mnemonic;
use bitcoin::hashes::{sha256, Hash};
use codex32::{Codex32String, Fe};

use crate::error::Codex32ErrorWrapper;

#[derive(thiserror::Error, Debug)]
pub enum SeedError {
    #[error("The dice launches sequence is too short to achieve 256 bits of entropy: {0}<={1}")]
    TooShort(usize, usize),

    #[error("The sequence contain a character ({0}) that is not in the dictionary: 1,2,3,4,5,6")]
    NonDictionaryChar(char),

    #[error(transparent)]
    Bip39(#[from] bip39::Error),

    #[error(transparent)]
    Codex32(#[from] Codex32ErrorWrapper), // wrapping is needed becose codex32::Error doesn't support Display
}

pub fn seed(dices: &str, codex32_id: Option<String>) -> Result<String, SeedError> {
    let needed_launches = (256f64 / 6f64.log2()) as usize;

    if dices.len() < needed_launches {
        return Err(SeedError::TooShort(dices.len(), needed_launches));
    }

    for el in dices.chars() {
        if !"123456".contains(el) {
            return Err(SeedError::NonDictionaryChar(el));
        }
    }

    let result = sha256::Hash::hash(dices.as_bytes());
    let result = result.as_byte_array();
    Ok(match codex32_id {
        Some(id) => Codex32String::from_seed("ms", 0, &id, Fe::S, result)
            .map_err(Codex32ErrorWrapper)?
            .to_string(),
        None => Mnemonic::from_entropy(result)?.to_string(),
    })
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bip39::Mnemonic;
    use bitcoin::{bip32::ExtendedPrivKey, secp256k1::Secp256k1, Network};
    use codex32::Codex32String;

    #[test]
    fn test_fingerprint() {
        let secp = Secp256k1::new();
        let mnemonic = "episode girl scorpion hope any pave carry rifle limit coffee review bus";
        let expected_fingerprint = "3456016b";
        let mnemonic = Mnemonic::from_str(mnemonic).unwrap();
        let xprv = ExtendedPrivKey::new_master(Network::Bitcoin, &mnemonic.to_seed("")).unwrap();
        let fingerprint = xprv.fingerprint(&secp);
        assert_eq!(expected_fingerprint, format!("{fingerprint}"));
        let tprv = ExtendedPrivKey::new_master(Network::Testnet, &mnemonic.to_seed("")).unwrap();
        let fingerprint = tprv.fingerprint(&secp);
        assert_eq!(
            expected_fingerprint,
            format!("{fingerprint}"),
            "network influences fingerprint"
        );
    }

    #[test]
    fn match_39_93() {
        // [2023-08-31T10:36:18Z INFO  dinasty::commands::mnemonic] ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd
        // flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden

        let b39 = Mnemonic::from_str("flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden").unwrap();
        let b93 = Codex32String::from_string(
            "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd"
                .to_string(),
        )
        .unwrap();

        assert_eq!(b93.parts().data(), b39.to_entropy());
    }
}
