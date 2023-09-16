mod broadcast;
mod decrypt;
mod descriptor;
mod encrypt;
mod identity;
mod import;
mod locktime;
mod qr;
mod refresh;
mod seed;
mod sign;
mod xkey;

use std::{
    net::SocketAddrV4,
    path::{Path, PathBuf},
};

use age::x25519::Recipient;
use bitcoin::bip32::DerivationPath;
pub use broadcast::{broadcast, BroadcastError};
use clap::{Args, Subcommand};
use clap_complete::Shell;
pub use decrypt::{decrypt, DecryptError};
pub use descriptor::descriptor;
pub use encrypt::{encrypt, EncryptError};
pub use identity::{identity, IdentityError};
pub use import::{import, ImportError};
pub use locktime::{locktime, LocktimeError};
pub use qr::qr;
pub use refresh::{refresh, RefreshError};
pub use seed::{seed, SeedError};
pub use sign::{sign, SignError};
pub use xkey::{key, KeyError};

#[derive(Subcommand)]
pub enum Commands {
    /// Print a seed (bip39 or bip93) given a list of dice launches.
    ///
    /// At least 91 launches of a 6-side dice are required to achieve 256 bits of entropy.
    /// 256 bits are provably over-kill and for other reasons we have 128 bits anyway, but hey you
    /// have to throw dices only once right?
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = "43242535241352135351234134123421351351342134123412351341324134134213512512353513123423423433222413233";
    /// let stdout = sh(&stdin, "dinasty seed");
    /// assert_eq!(stdout, "flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden");
    /// let stdout = sh(&stdin, "dinasty seed --codex32-id leet");
    /// assert_eq!(stdout, "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd");
    /// ```
    ///
    /// The dice sequence must be kept secret, so it should not be saved in a clear text file, and
    /// neither in your shell command history. That's why it's taken from standard input, do this:
    ///
    /// * launch `dinasty seed` command, the command will expect data from standard input, so it will wait
    /// * type your dice launch sequence: `315613535...`
    /// * press enter when you finish
    /// * press Ctrl+D to terminate the stdin
    ///
    #[clap(verbatim_doc_comment)]
    Seed {
        /// If specified the output will be encoded in Codex32 (bip93)
        #[arg(long)]
        codex32_id: Option<String>,
    },

    /// Given a mnemonic from standard input, prints the extended private key
    /// derived at the given path
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = "flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden";
    /// let stdout = sh(&stdin, "dinasty -n regtest xkey m/0h");
    /// assert_eq!(stdout, "[01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8");
    /// let stdout = sh(&stdin, "dinasty -n regtest xkey m/1h");
    /// assert_eq!(stdout, "[01e0b4da/1']tprv8batdt5VSxwNfjjgb9oLoxAHWLbwLaqPACU5uA9aSrCFQGszkzdN3Gc6Np4tvDfrMw4bELAjRpFXaiL4mmLRncYxbvZd8CxGzEbxGZjL9v1");
    /// ```
    #[clap(verbatim_doc_comment)]
    Xkey {
        /// The derivation path to derive from the given mnemonic/codex32, example `m/0h`
        path: DerivationPath,
    },

    /// Create an age recipient from a key
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = "[01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8";
    /// let stdout = sh(&stdin, "dinasty -n regtest identity");
    /// assert_eq!(stdout, "age1us3jpg4exjj2nenj5vwkns7cnnldhj48hdzvey0u5gt6dlss3vss4qdf6w");
    /// let stdout = sh(&stdin, "dinasty -n regtest identity --private");
    /// assert_eq!(stdout, "AGE-SECRET-KEY-1SQQPGC243W5CFU9PJL2K46PTQ8UZCH8AA6NYT7RT36P5720WK48QY7T6XL");
    /// ```
    #[clap(verbatim_doc_comment)]
    Identity {
        /// If the flag is provided the decryption secret key will be printed
        #[arg(long)]
        private: bool,
    },

    /// Given an extended private key and if public or private prints the descriptor
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = "[01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8";
    /// let stdout = sh(&stdin, "dinasty -n regtest descriptor");
    /// assert_eq!(stdout, "tr([01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8/<0;1>/*)");
    /// let stdout = sh(&stdin, "dinasty -n regtest descriptor --public");
    /// assert_eq!(stdout, "tr([01e0b4da/0']tpubD8GvnJ7jbLd3VPJsgE9o8nuB2uVJpU1DmHfFCPkVQsZiS9RL5ttWmjjNDzrQWcCy5ntdC8umt4ixDTsL7w9JYhnqKaYRTKH4F7yHVBqwCt3/<0;1>/*)");
    /// ```
    #[clap(verbatim_doc_comment)]
    Descriptor {
        /// If the flag is provided the descriptor will contain extended public keys instead of extended private keys
        #[arg(long)]
        public: bool,

        /// If the flag is provided the descriptor will contain only the external addresses derivation (`0/*`)
        #[arg(long)]
        only_external: bool,
    },

    /// Connects to a local instance of bitcoin core, importing the descriptor given
    /// from standard input.
    ///
    /// If the descriptor contains extended PRIVATE keys, the flag `--with-private-keys` must be used
    ///
    /// If the `--with-private-keys` flag is used, the given private descriptor is used as wallet passphrase
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestNode { node, core_connect_params, .. } = setup_node();
    /// let stdin = "tr([01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} import --wallet-name signer --with-private-keys"));
    /// assert!(stdout.contains("success\":true"));
    /// assert!(stdout.contains("error\":null"));
    ///
    /// let stdin = "tr([01e0b4da/0']tpubD8GvnJ7jbLd3VPJsgE9o8nuB2uVJpU1DmHfFCPkVQsZiS9RL5ttWmjjNDzrQWcCy5ntdC8umt4ixDTsL7w9JYhnqKaYRTKH4F7yHVBqwCt3/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} import --wallet-name watch_only"));
    /// assert!(stdout.contains("success\":true"));
    /// assert!(stdout.contains("error\":null"));
    /// ```
    ///
    /// This wallet setup example is used in other doc tests via [`crate::test_util::setup_node_and_wallets()`]
    ///
    #[clap(verbatim_doc_comment)]
    Import {
        #[arg(short, long, required = true)]
        wallet_name: String,

        #[arg(long)]
        with_private_keys: bool,
    },

    /// For every old UTXO creates a locktimed transaction to another wallet
    ///
    /// Connects to a local instance of bitcoin core, for every UTXO in `wallet_name` creates
    /// a PSBT with nlocktime equal to current chain tip + `locktime_future`
    /// the outputs are from heir_descriptor and they always start from derivation 0 (reusing a
    /// valid older timelock may cause address reuse but it's rare and better than leaving a
    /// potential gap or other solutions)
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestEnv { node, node_address, core_connect_params, watch_only, signer, .. } = setup_node_and_wallets();
    /// # assert_eq!(node.client.get_blockchain_info().unwrap().blocks, 101);
    /// # let heir_public_descriptor = "tr([01e0b4da/1']tpubD8GvnJ7jbLd3ZCmUUoTwDMpQ5N7sVv2HjW4sBgBss7zeEm8mPPSxDmDxYy4rxGZbQAcbRGwawzXMUpnLAnHcrNmZcqucy3qAyn7NZzKChpx/0/*)#04qa3cn0";
    /// let stdout = sh("", &format!("dinasty {core_connect_params} locktime -w watch_only --locktime-future 200 --to-public-descriptor {heir_public_descriptor}"));
    /// let signed_psbt = bitcoin::psbt::PartiallySignedTransaction::from_str(&stdout).unwrap();
    /// let tx = signed_psbt.extract_tx();
    /// assert_eq!(tx.lock_time, bitcoin::absolute::LockTime::from_height(301).unwrap());
    /// ```
    ///
    #[clap(verbatim_doc_comment)]
    Locktime {
        /// The name of the already existing wallet in bitcoin core used as source of UTXOs
        #[arg(short, long, required = true)]
        wallet_name: String,

        /// Recipient addresses of the created transactions will be created from this descriptor
        #[arg(long, required = true)]
        to_public_descriptor: String,

        /// Default value equals to about 4 years
        #[arg(long, default_value_t = 210_240)]
        locktime_future: i64,
    },

    /// Refresh owned UTXO with the goal of invalidating previously sent presigned transactions
    ///
    /// Connects to a local instance of bitcoin core.
    /// For every UTXO older than 'blocks' blocks create a 1-1 transaction to self, prints out a
    /// list of PSBTs
    ///
    /// This should be needed also after every owner wallet spending creating a change, so that the
    /// change is spendable by the heir.
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestEnv { node, core_connect_params, watch_only, signer, .. } = setup_node_and_wallets();
    /// # assert_eq!(node.client.get_blockchain_info().unwrap().blocks, 101);
    /// let stdout = sh("", &format!("dinasty {core_connect_params} refresh -w watch_only --older-than-blocks 10"));
    /// let signed_psbt = bitcoin::psbt::PartiallySignedTransaction::from_str(&stdout).unwrap();
    /// let tx = signed_psbt.extract_tx();
    /// let address = bitcoin::Address::from_script(&tx.output[0].script_pubkey, bitcoin::Network::Regtest).unwrap();
    /// let address_info = signer.get_address_info(&address).unwrap();
    /// assert!(address_info.is_mine.unwrap());
    /// ```
    ///
    #[clap(verbatim_doc_comment)]
    Refresh {
        /// The wallet of which UTXOs older than `older_than_blocks` must be refreshed, they will
        /// be redeposited to new addresses of the same wallet
        #[arg(short, long, required = true)]
        wallet_name: String,

        /// Default values equals to about 3 years
        #[arg(long, default_value_t = 157_680)]
        older_than_blocks: u32,
    },

    /// Connects to a local instance of bitcoin core and signs the psbts given in standard input
    /// printing it back as QR code
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestEnv { node, node_address, core_connect_params, watch_only, .. } = setup_node_and_wallets();
    /// # let psbt = watch_only.prepare_psbt_to(&node_address, 10_000).unwrap();
    /// # let mut file = tempfile::NamedTempFile::new().unwrap();
    /// # std::fs::write(&file, psbt).unwrap();
    /// # let psbt_file_path = file.path().display();
    /// let stdin = "tr([01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} sign -w signer --psbt-file {psbt_file_path}"));
    /// std::fs::write(&file, &stdout).unwrap();
    /// let signed_psbt = bitcoin::psbt::PartiallySignedTransaction::from_str(&stdout).unwrap();
    /// let tx = signed_psbt.extract_tx();
    /// let result = node.client.test_mempool_accept(&[&tx]).unwrap();
    /// assert!(result[0].allowed);
    /// let stdout = sh("", &format!("dinasty {core_connect_params} broadcast --psbt-file {psbt_file_path}"));
    /// assert_eq!(stdout, tx.txid().to_string());
    /// ```
    #[clap(verbatim_doc_comment)]
    Sign {
        #[arg(short, long, required = true)]
        wallet_name: String,

        /// file containing one psbt in base64 per line
        #[arg(long, required = true)]
        psbt_file: PathBuf,
    },

    /// Broadcast the transactions
    ///
    /// for an example see `Sign` command
    #[clap(verbatim_doc_comment)]
    Broadcast {
        /// file containing one psbt in base64 per line
        #[arg(long, required = true)]
        psbt_file: PathBuf,
    },

    /// Encrypt standard input for given recipients
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let plain_text = "plain text";
    /// let stdin = plain_text;
    /// let stdout = sh(&stdin, "dinasty encrypt -r age18addpm2vs78d96jg39yc3d2dehtzfef75nh8c6xtz5xejk6l3svs8c8kkl");
    /// assert!(stdout.starts_with("-----BEGIN AGE ENCRYPTED FILE-----"));
    /// assert!(!stdout.contains(plain_text));
    /// assert!(stdout.ends_with("-----END AGE ENCRYPTED FILE-----\n"));
    /// ```
    #[clap(verbatim_doc_comment)]
    Encrypt {
        /// Recipients of encryption, at least one is required.
        ///
        /// Must be an age public key like `age18addpm2vs78d96jg39yc3d2dehtzfef75nh8c6xtz5xejk6l3svs8c8kkl`
        #[arg(short, long, required = true)]
        recipients: Vec<Recipient>,
    },

    /// Using an identity or key provided from standard input, decrypt the content of the encrypted_file
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let plain_text = "plain text";
    /// # let stdin = plain_text;
    /// # let stdout = sh(&stdin, "dinasty encrypt -r age18addpm2vs78d96jg39yc3d2dehtzfef75nh8c6xtz5xejk6l3svs8c8kkl");
    /// # let mut file = tempfile::NamedTempFile::new().unwrap();
    /// # std::fs::write(&file, stdout).unwrap();
    /// # let encrypted_file_path = file.path().display();
    /// let stdin = "AGE-SECRET-KEY-18ZTCPKR7N22AGJYXFW5ATW6SQW2X74HCV37A686UG0ZP9MX6LF2SRHF78N";
    /// let stdout = sh(&stdin, &format!("dinasty decrypt {encrypted_file_path}"));
    /// assert_eq!(plain_text, stdout);
    /// let stdin = "[01e0b4da/0']xprv9tuwrYmA3h7J173Z81dhZjd59kZARcnJrS9M3THjWdGqp4RUU8jB5Vk48hZYDKUoRpYKfDxR5ytaBYaUn4pF5RiZbPyENrvTuhWtuXjT8AM";
    /// let stdout = sh(&stdin, &format!("dinasty decrypt {encrypted_file_path}"));
    /// assert_eq!(plain_text, stdout);
    /// ```
    #[clap(verbatim_doc_comment)]
    Decrypt { encrypted_file: PathBuf },

    /// Convert the text content of `file` into a number of QR codes such that every QR code
    /// encode at max `max_chars`
    #[clap(verbatim_doc_comment)]
    Qr {
        file: PathBuf,

        /// QR code version
        #[arg(long, default_value_t = 16)]
        qr_version: i16,

        /// Module at the border of the QR code
        #[arg(long, default_value_t = 4)]
        border: u8,

        /// Number of empty lines between one QR and the following
        #[arg(long, default_value_t = 6)]
        empty_lines: u8,
    },

    #[clap(hide = true)]
    GenerateCompletion { shell: Shell },
}

#[derive(Debug, Args)]
pub struct CoreConnectOptional {
    /// The bitcoin core node url, if not provided defaults to the network default
    /// for example "127.0.0.1:8332" for mainnet
    #[clap(long)]
    pub node_socket: Option<SocketAddrV4>,

    /// The bitcoin core path of the cookie for authentication, if not provided defaults to the
    /// network defaults, for example "$HOME/.bitcoin/.cookie" for mainnet
    #[clap(long)]
    pub node_cookie_path: Option<PathBuf>,
}

impl Commands {
    pub fn needs_stdin(&self) -> bool {
        match self {
            Commands::Locktime { .. }
            | Commands::Refresh { .. }
            | Commands::Encrypt { .. }
            | Commands::GenerateCompletion { .. } => false,
            Commands::Qr { file, .. } => {
                if file == Path::new("-") {
                    true
                } else {
                    false
                }
            }
            _ => true,
        }
    }
}
