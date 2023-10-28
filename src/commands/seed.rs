use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::{
    bip32::{ExtendedPrivKey, Fingerprint},
    hashes::{sha256, Hash},
    secp256k1::Secp256k1,
    Network,
};
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
    Codex32(#[from] Codex32ErrorWrapper), // wrapping is needed because codex32::Error doesn't support Display

    #[error("The string '{0}' cannot be interpreted neither as Mnemonic nor as Code32")]
    NeitherMnemonicNorCodex32(String),
}

pub fn seed(dices: &str, codex32_id: Option<String>) -> Result<Seed, SeedError> {
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
        Some(id) => Seed::Codex32(
            Codex32String::from_seed("ms", 0, &id, Fe::S, result).map_err(Codex32ErrorWrapper)?,
        ),
        None => Seed::Mnemonic(Mnemonic::from_entropy(result)?),
    })
}

pub enum Seed {
    Mnemonic(Mnemonic),
    Codex32(Codex32String),
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Seed::Mnemonic(e) => write!(f, "{}", e),
            Seed::Codex32(e) => write!(f, "{}", e),
        }
    }
}

impl Seed {
    fn as_mnemonic(&self) -> Mnemonic {
        match self {
            Seed::Mnemonic(e) => e.clone(),
            Seed::Codex32(e) => {
                Mnemonic::from_entropy(&e.parts().data()).expect("guaranteed 32 bytes")
            }
        }
    }
    pub fn xprv(&self, network: Network) -> Result<ExtendedPrivKey, bitcoin::bip32::Error> {
        let mnemonic = self.as_mnemonic();
        ExtendedPrivKey::new_master(network, &mnemonic.to_seed(""))
    }

    pub fn fingerprint(&self) -> Result<Fingerprint, bitcoin::bip32::Error> {
        let secp = Secp256k1::new();
        Ok(self.xprv(Network::Bitcoin)?.fingerprint(&secp))
    }
}

impl FromStr for Seed {
    type Err = SeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(mnemonic) = s.parse::<Mnemonic>() {
            Ok(Seed::Mnemonic(mnemonic))
        } else if let Ok(codex32) = Codex32String::from_string(s.to_string()) {
            Ok(Seed::Codex32(codex32))
        } else {
            Err(SeedError::NeitherMnemonicNorCodex32(s.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bip39::Mnemonic;
    use bitcoin::{bip32::ExtendedPrivKey, secp256k1::Secp256k1, Network};
    use codex32::Codex32String;

    pub const DICES: &str = "43242535241352135351234134123421351351342134123412351341324134134213512512353513123423423433222413233";
    pub const MNEMONIC: &str = "flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden";
    pub const CODEX_32: &str =
        "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd";

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

        let b39 = Mnemonic::from_str(MNEMONIC).unwrap();
        let b93 = Codex32String::from_string(CODEX_32.to_string()).unwrap();

        assert_eq!(b93.parts().data(), b39.to_entropy());
    }

    #[test]
    fn match_39_93_all_networks() {
        let dices = DICES;
        let seed_mnemonic = super::seed(dices, None).unwrap();
        assert_eq!(seed_mnemonic.to_string(), MNEMONIC);
        let seed_codex32 = super::seed(dices, Some("leet".to_string())).unwrap();
        assert_eq!(seed_codex32.to_string(), CODEX_32);
        for network in [
            Network::Bitcoin,
            Network::Testnet,
            Network::Signet,
            Network::Regtest,
        ] {
            assert_eq!(seed_codex32.xprv(network), seed_mnemonic.xprv(network));

            assert_eq!(seed_codex32.fingerprint(), seed_mnemonic.fingerprint());
        }
    }
}
