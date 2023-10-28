use std::fmt::Display;

use crate::{commands, psbts_serde};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Mnemonic(#[from] commands::SeedError),

    #[error(transparent)]
    Locktime(#[from] commands::LocktimeError),

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

    #[error(transparent)]
    PsbtDecodeError(#[from] psbts_serde::DecodeError),

    #[error(transparent)]
    Balance(#[from] commands::BalanceError),

    #[error(transparent)]
    Miniscript(#[from] miniscript::Error),
}

#[derive(Debug)]
pub struct Codex32ErrorWrapper(pub(crate) codex32::Error);

impl std::error::Error for Codex32ErrorWrapper {}

impl Display for Codex32ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
