use crate::core_connect::CoreConnect;
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

/// Emulate the shell by parsing the given command with the clap struct `Cli`.
pub fn sh(stdin: &str, command: &str) -> String {
    let stdin: Vec<_> = stdin.split('\n').map(ToString::to_string).collect();

    let cli = Cli::try_parse_from(command.split(' ')).unwrap();
    inner_main(cli, &stdin).expect(command)
}

// TODO return also core connect params, make a struct, make setup_node_and_wallet returning both
/// Launch a bitcoin core node in regtest mode
pub fn setup_node() -> (BitcoinD, Address, CoreConnect) {
    let node = bitcoind::BitcoinD::new(bitcoind::exe_path().unwrap()).unwrap();
    let node_address = node
        .client
        .get_new_address(None, None)
        .unwrap()
        .assume_checked();
    let core_connect = CoreConnect::from((&node, bitcoin::Network::Regtest));
    (node, node_address, core_connect)
}

pub struct TestWallets {
    pub signer: Client,
    pub watch_only: Client,
}

/// Creates two wallets in the node: "watch_only" and "signer".
///
/// These wallets generate the same addresses but the wallet "signer" obviously has the private
/// keys, while "watch_only" has only the public descriptor.
///
/// Another difference is that the signer key use the private descriptor as passphrase and after
/// the creation the wallet is locked (will require wallet_passphrase call to unlock)
///
pub fn setup_wallets(node: &BitcoinD) -> TestWallets {
    let core_connect: CoreConnect = (node, Network::Regtest).into();

    let desc = "tr([01e0b4da/0']tpubD8GvnJ7jbLd3VPJsgE9o8nuB2uVJpU1DmHfFCPkVQsZiS9RL5ttWmjjNDzrQWcCy5ntdC8umt4ixDTsL7w9JYhnqKaYRTKH4F7yHVBqwCt3/<0;1>/*)";
    commands::import(&core_connect, desc, "watch_only", false).unwrap();

    let desc = "tr([01e0b4da/0']tprv8batdt5VSxwNbvH5naVCjPF4TsyNf8pKBz4TusiBzbmKbfAZTW4vbF7W3sjCDgs7oG56fKaBFLUNeQ8DuHABtUzA83BY3DeWpoGKM9zLYV8/<0;1>/*)";
    commands::import(&core_connect, desc, "signer", true).unwrap();

    let watch_only = core_connect.client_with_wallet("watch_only").unwrap();
    let signer = core_connect.client_with_wallet("signer").unwrap();

    let address = watch_only
        .get_new_bech32m_address(bitcoin::Network::Regtest)
        .unwrap();
    assert_eq!(
        address.to_string(),
        "bcrt1pr3sacyj3hs2a4lnwq6zyeqw94ftm08kghvzjfge89gqdgz3lvuxs2jc7fh"
    );
    let address_signer = signer
        .get_new_bech32m_address(bitcoin::Network::Regtest)
        .unwrap();
    assert_eq!(address, address_signer);

    node.client.generate_to_address(101, &address).unwrap();

    signer.wallet_lock();

    TestWallets { signer, watch_only }
}
