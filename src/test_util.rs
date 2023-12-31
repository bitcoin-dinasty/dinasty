use crate::core_connect::CoreConnect;
use crate::stdin::StdinData;
use crate::stdout::StdoutData;
use crate::{commands, inner_main, Cli};
use bitcoin::{Address, Network};
use bitcoind::bitcoincore_rpc::Client;
use bitcoind::BitcoinD;
use clap::Parser;

// Re-exports so that doc tests don't need the import
pub use crate::client_ext::ClientExt;
pub use bitcoind::bitcoincore_rpc::RpcApi;
pub use std::io::Write;
pub use std::str::FromStr;

/// Emulate the shell by parsing the given command with the clap struct `Cli`
pub fn sh<T: AsRef<[u8]>>(stdin: T, command: &str) -> StdoutData {
    let stdin = (!stdin.as_ref().is_empty()).then(|| StdinData::new(stdin.as_ref().to_vec()));

    let cli = Cli::try_parse_from(command.split(' ')).unwrap();

    let bytes = inner_main(cli, stdin).unwrap();
    StdoutData::new(bytes)
}

pub struct TestNode {
    pub node: BitcoinD,
    pub node_address: Address,
    pub core_connect: CoreConnect,
    pub core_connect_params: String,
}

/// Launch a bitcoin core node in regtest mode
pub fn setup_node() -> TestNode {
    let node = bitcoind::BitcoinD::new(bitcoind::exe_path().unwrap()).unwrap();
    let node_address = node
        .client
        .get_new_address(None, None)
        .unwrap()
        .assume_checked();
    let core_connect = CoreConnect::from((&node, bitcoin::Network::Regtest));
    let core_connect_params = core_connect.sh_params();
    TestNode {
        node,
        node_address,
        core_connect,
        core_connect_params,
    }
}

pub struct TestWallets {
    pub signer: Client,
    pub watch_only: Client,
    pub heir_watch_only: Client,
}

/// Creates two wallets in the node: "watch_only" and "signer".
///
/// These wallets generate the same addresses but the wallet "signer" obviously has the private
/// keys, while "watch_only" has only the public descriptor.
///
/// Another difference is that the signer key use the private descriptor as passphrase and after
/// the creation the wallet is locked (will require wallet_passphrase call to unlock)
///
fn setup_wallets(node: &BitcoinD) -> TestWallets {
    let core_connect: CoreConnect = (node, Network::Regtest).into();

    let desc = "tr([01e0b4da/86h/1h/0h]tprv8fXspLN8b22B19ViogBWdGHR4ZHkoUd7VvMpoUZCkPZtHiKLZyc9H9pgfTnZwrosXQ5hKLTdSCPerVrgtewQjTSRy1YEngEZXHNCvTodhtz/<0;1>/*)";
    commands::import(&core_connect, desc, "signer", true).unwrap();

    let desc = "tr([01e0b4da/86h/1h/0h]tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/<0;1>/*)";
    commands::import(&core_connect, desc, "watch_only", false).unwrap();

    let desc = "tr([01e0b4da/86h/1h/1h]tpubDCDuxkQNjPhqtq5ARHKc6t5QPg8CUyqJ6uzVkLqDBQtJ47Fac1JwrMUN9Zr6c3dAD5bGxL3DihfZUisSuszupSLoanydKxT8giNcVJSo2vq/<0;1>/*)";
    commands::import(&core_connect, desc, "heir_watch_only", false).unwrap();

    let signer = core_connect.client_with_wallet("signer").unwrap();
    let watch_only = core_connect.client_with_wallet("watch_only").unwrap();
    let heir_watch_only = core_connect.client_with_wallet("watch_only").unwrap();

    let address = watch_only
        .get_new_bech32m_address(bitcoin::Network::Regtest)
        .unwrap();
    assert_eq!(
        address.to_string(),
        "bcrt1pccadr74cd29xf5y0eax2dwnfvjeqwa65c9h09f7cw6c2h6c7rjyswwdgx4"
    );
    let address_signer = signer
        .get_new_bech32m_address(bitcoin::Network::Regtest)
        .unwrap();
    assert_eq!(address, address_signer);

    let address_heir_watch_only = heir_watch_only
        .get_new_bech32m_address(bitcoin::Network::Regtest)
        .unwrap();
    assert_ne!(address, address_heir_watch_only);

    node.client.generate_to_address(101, &address).unwrap();

    signer.wallet_lock();

    TestWallets {
        signer,
        watch_only,
        heir_watch_only,
    }
}

pub struct TestEnv {
    pub node: BitcoinD,
    pub node_address: Address,
    pub core_connect: CoreConnect,
    pub core_connect_params: String,
    pub signer: Client,
    pub watch_only: Client,
}

pub fn setup_node_and_wallets() -> TestEnv {
    let node: TestNode = setup_node();
    let wallets = setup_wallets(&node.node);

    TestEnv {
        node: node.node,
        node_address: node.node_address,
        core_connect: node.core_connect,
        core_connect_params: node.core_connect_params,
        signer: wallets.signer,
        watch_only: wallets.watch_only,
    }
}

pub fn psbt_base64() -> String {
    let psbt_str = include_str!("../test_data/psbt_base64");
    psbt_str.to_string()
}

pub fn psbts_binary() -> Vec<u8> {
    let psbt_bytes = include_bytes!("../test_data/psbts_binary");
    psbt_bytes.to_vec()
}
