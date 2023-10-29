mod broadcast;
mod descriptor;
mod details;
mod identity;
mod import;
mod locktime;
mod qr;
mod refresh;
mod seed;
mod sign;

use clap::{Args, Subcommand};
use clap_complete::Shell;
use std::{net::SocketAddrV4, path::PathBuf};

pub use broadcast::{broadcast, BroadcastError};
pub use descriptor::descriptor;
pub use details::{psbt_details, BalanceError};
pub use identity::{identity, IdentityError};
pub use import::{import, ImportError};
pub use locktime::{locktime, LocktimeError};
pub use qr::qr;
pub use refresh::{refresh, RefreshError};
pub use seed::{seed, Seed, SeedError};
pub use sign::{sign, SignError};

use crate::Descriptor;

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

    /// Given a seed, an account, if public or private prints the bip86 descriptor
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd";
    /// let stdout = sh(&stdin, "dinasty -n regtest descriptor --account 0");
    /// assert_eq!(stdout, "tr([01e0b4da/86h/1h/0h]tprv8fXspLN8b22B19ViogBWdGHR4ZHkoUd7VvMpoUZCkPZtHiKLZyc9H9pgfTnZwrosXQ5hKLTdSCPerVrgtewQjTSRy1YEngEZXHNCvTodhtz/<0;1>/*)");
    /// let stdout = sh(&stdin, "dinasty -n regtest descriptor --public --account 0");
    /// assert_eq!(stdout, "tr([01e0b4da/86h/1h/0h]tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/<0;1>/*)");
    /// let stdout = sh(&stdin, "dinasty -n regtest descriptor --account 1");
    /// assert_eq!(stdout, "tr([01e0b4da/86h/1h/1h]tprv8fXspLN8b22B1N3NXdf1hURHpecGKeePXcPiTpnum95uDczoycVMfrrVyRLpgdhRrxwLZNoEz1DrRf712VBu5qmoPtLsjcqv4GTctGjb1fL/<0;1>/*)");
    /// let stdout = sh(&stdin, "dinasty -n regtest descriptor --public --account 1");
    /// assert_eq!(stdout, "tr([01e0b4da/86h/1h/1h]tpubDCDuxkQNjPhqtq5ARHKc6t5QPg8CUyqJ6uzVkLqDBQtJ47Fac1JwrMUN9Zr6c3dAD5bGxL3DihfZUisSuszupSLoanydKxT8giNcVJSo2vq/<0;1>/*)");
    /// ```
    #[clap(verbatim_doc_comment)]
    Descriptor {
        /// If the flag is provided the descriptor will contain extended public keys instead of extended private keys
        #[arg(long)]
        public: bool,

        #[arg(long)]
        account: u16,
    },

    /// Create an age recipient or identity from a seed
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = "flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden";
    /// let stdout = sh(&stdin, "dinasty -n regtest identity");
    /// assert_eq!(stdout, "age16unvc0en3dcageh7vqtdj2cvmgzp57uz5zp7pz4rllagcdr2v58scwpffw");
    /// let stdout = sh(&stdin, "dinasty -n regtest identity --private");
    /// assert_eq!(stdout, "AGE-SECRET-KEY-1FPURSK70MHN40TSPY7Q546WPJVLS0FS8V3H6XJU3CDX5D3UDM9ZQT4L4MN");
    /// ```
    #[clap(verbatim_doc_comment)]
    Identity {
        /// If the flag is provided the decryption secret key will be printed
        #[arg(long)]
        private: bool,
    },

    /// Connects to bitcoin core, importing the descriptor given from stdin.
    ///
    /// If the descriptor contains extended PRIVATE keys, the flag `--with-private-keys` must be used
    ///
    /// If the `--with-private-keys` flag is used, the given private descriptor is used as wallet passphrase
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestNode { node, core_connect_params, .. } = setup_node();
    /// let stdin = "tr([01e0b4da/86h/1h/0h]tprv8fXspLN8b22B19ViogBWdGHR4ZHkoUd7VvMpoUZCkPZtHiKLZyc9H9pgfTnZwrosXQ5hKLTdSCPerVrgtewQjTSRy1YEngEZXHNCvTodhtz/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} import --wallet-name signer --with-private-keys")).to_string();
    /// assert!(stdout.contains("ok"));
    ///
    /// let stdin = "tr([01e0b4da/86h/1h/0h]tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} import --wallet-name watch_only")).to_string();
    /// assert!(stdout.contains("ok"));
    ///
    /// let stdin = "tr([01e0b4da/86h/1h/1h]tpubDCDuxkQNjPhqtq5ARHKc6t5QPg8CUyqJ6uzVkLqDBQtJ47Fac1JwrMUN9Zr6c3dAD5bGxL3DihfZUisSuszupSLoanydKxT8giNcVJSo2vq/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} import --wallet-name heir_watch_only")).to_string();
    /// assert!(stdout.contains("ok"));
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
    /// let stdout = sh("", &format!("dinasty {core_connect_params} locktime --locktime-future 200 --from-wallet-name watch_only --to-wallet-name heir_watch_only"));
    /// let signed_psbt = stdout.to_psbts().unwrap()[0].clone();
    /// let tx = signed_psbt.extract_tx();
    /// assert_eq!(tx.lock_time, bitcoin::absolute::LockTime::from_height(301).unwrap());
    /// ```
    ///
    #[clap(verbatim_doc_comment)]
    Locktime {
        /// The name of the already existing wallet in bitcoin core used as source of UTXOs
        #[arg(long, required = true)]
        from_wallet_name: String,

        /// Recipient addresses of the created transactions will be created to this wallet_name
        #[arg(long, required = true)]
        to_wallet_name: String,

        /// Default value equals to about 4 years
        #[arg(long, default_value_t = 210_240)]
        locktime_future: i64,
    },

    /// Refresh owned UTXO with the goal of invalidating previously generated locktimed transactions
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
    /// let tx = stdout.to_psbts().unwrap()[0].clone().extract_tx();
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

    /// Connects to bitcoin core and signs the given PSBTs.
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestEnv { node, node_address, core_connect_params, watch_only, .. } = setup_node_and_wallets();
    /// # let psbt = watch_only.prepare_psbt_to(&node_address, 10_000).unwrap();
    /// # let mut file = tempfile::NamedTempFile::new().unwrap();
    /// # std::fs::write(&file, psbt).unwrap();
    /// # let psbt_file_path = file.path().display();
    /// let stdin = "tr([01e0b4da/86h/1h/0h]tprv8fXspLN8b22B19ViogBWdGHR4ZHkoUd7VvMpoUZCkPZtHiKLZyc9H9pgfTnZwrosXQ5hKLTdSCPerVrgtewQjTSRy1YEngEZXHNCvTodhtz/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty {core_connect_params} sign -w signer --psbt-file {psbt_file_path}"));
    /// let signed_psbts = stdout;
    /// let tx = signed_psbts.to_psbts().unwrap()[0].clone().extract_tx();
    /// let result = node.client.test_mempool_accept(&[&tx]).unwrap();
    /// assert!(result[0].allowed);
    /// let stdout = sh(signed_psbts, &format!("dinasty {core_connect_params} broadcast"));
    /// assert_eq!(stdout, tx.txid().to_string());
    /// ```
    #[clap(verbatim_doc_comment)]
    Sign {
        #[arg(short, long, required = true)]
        wallet_name: String,

        /// file containing one or more psbt in binary format
        #[arg(long, required = true)]
        psbt_file: PathBuf,
    },

    /// Broadcast the PSBTs given from stdin.
    ///
    /// for an example see `Sign` command
    #[clap(verbatim_doc_comment)]
    Broadcast,

    /// Convert the binary PSBTs given from stdin to new-line separated base64
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let stdin = psbts_binary();
    /// let stdout = sh(&stdin, "dinasty bin-to-b64");
    /// ```
    ///
    #[clap(verbatim_doc_comment)]
    BinToB64,

    /// Convert the base64 PSBTs given from stdin to binary PSBTs
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// let psbts_binary = psbts_binary();
    /// let stdin = psbts_binary.clone();
    /// let stdout = sh(&stdin, "dinasty bin-to-b64");
    /// let stdin = stdout;
    /// let stdout = sh(&stdin, "dinasty b64-to-bin").to_psbts().unwrap();
    /// assert_eq!(psbts_binary, dinasty::psbts_serde::serialize(&stdout));
    /// ```
    ///
    #[clap(verbatim_doc_comment)]
    B64ToBin,

    /// Gives details for each PSBTs given from stdin.
    ///
    /// For example the net balance considering the given descriptors.
    /// An 'm' on a input or output line means the script_pubkey can be created by the descriptors.
    /// An 's' on an input means there is a signature.
    ///
    /// ```
    /// # use dinasty::test_util::*;
    /// # let TestEnv { node, node_address, core_connect_params, watch_only, .. } = setup_node_and_wallets();
    /// # let stdin = watch_only.prepare_psbt_to(&node_address, 10_000).unwrap();
    /// let desc = "tr([01e0b4da/86h/1h/0h]tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/<0;1>/*)";
    /// let stdout = sh(&stdin, &format!("dinasty details --descriptor {desc}"));
    /// assert_eq!(stdout.to_string().split("\n").skip(9).next().unwrap(), "net  :  -0.000114200");
    /// ```
    ///
    #[clap(verbatim_doc_comment)]
    Details {
        /// The public descriptor (multipath) to calculate the net balance against.
        #[arg(long)]
        descriptor: Vec<Descriptor>,
    },

    /// Convert the text given on stdin into 1 or more QR codes
    #[clap(verbatim_doc_comment)]
    Qr {
        /// QR code version to be used, this is best-estimate, result version could be slightly
        /// different, adjust accordingly
        #[arg(long, default_value_t = 16)]
        qr_version: i16,

        /// Module at the border of the QR code
        #[arg(long, default_value_t = 4)]
        border: u8,

        /// Number of empty lines between one QR and the following
        #[arg(long, default_value_t = 6)]
        empty_lines: u8,

        #[arg(long)]
        label: Option<String>,
    },

    #[clap(hide = true)]
    GenerateCompletion { shell: Shell },
}

#[derive(Debug, Args)]
pub struct CoreConnectOptional {
    /// The bitcoin core node url, if not provided defaults to the network default
    /// for example "127.0.0.1:8332" for mainnet
    #[clap(long, env)]
    pub node_socket: Option<SocketAddrV4>,

    /// The bitcoin core path of the cookie for authentication, if not provided defaults to the
    /// network defaults, for example "$HOME/.bitcoin/.cookie" for mainnet
    #[clap(long, env)]
    pub node_cookie_path: Option<PathBuf>,
}
