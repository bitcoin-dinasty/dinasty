use std::fmt::Display;

use crate::commands;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Mnemonic(#[from] commands::SeedError),

    #[error(transparent)]
    Locktime(#[from] commands::LocktimeError),

    #[error(transparent)]
    Key(#[from] commands::KeyError),

    #[error(transparent)]
    KeyOrigin(#[from] crate::key_origin::KeyOriginError),

    #[error(transparent)]
    ImportWallet(#[from] commands::ImportError),

    #[error(transparent)]
    Identity(#[from] commands::IdentityError),

    #[error(transparent)]
    Refresh(#[from] commands::RefreshError),

    #[error(transparent)]
    Sign(#[from] commands::SignError),

    #[error(transparent)]
    Broadcast(#[from] commands::BroadcastError),

    #[error(transparent)]
    Encrypt(#[from] commands::EncryptError),

    #[error(transparent)]
    Decrypt(#[from] commands::DecryptError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Psbt(#[from] bitcoin::psbt::PsbtParseError),

    #[error(transparent)]
    Cookie(#[from] crate::core_connect::CookieError),

    #[error(transparent)]
    Qr(#[from] qr_code::types::QrError),

    #[error(transparent)]
    Stdin(#[from] crate::stdin::StdinError),

    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error("Stdin is expected for this command")]
    StdinExpected,

    #[error("Decrypt error, cannot parse \"{input}\" neither age identity \"{identity_parse_err}\" nor xprv \"{xprv_parse_err}\"")]
    DecryptError {
        input: String,
        identity_parse_err: String,
        xprv_parse_err: crate::key_origin::KeyOriginError,
    },
}

#[derive(Debug)]
pub struct Codex32ErrorWrapper(pub(crate) codex32::Error);

impl std::error::Error for Codex32ErrorWrapper {}

impl Display for Codex32ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
