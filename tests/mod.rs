use bitcoin::{bip32::ExtendedPrivKey, Network};

use bitcoind::bitcoincore_rpc::RpcApi;
use dinasty::core_connect::CoreConnect;
use rand::prelude::*;

mod core_tests;
mod dinasty_tests;

pub fn create_random_extended(network: Network) -> ExtendedPrivKey {
    let mut rng = rand::thread_rng();
    let seed: [u8; 32] = rng.gen();
    ExtendedPrivKey::new_master(network, &seed[..]).unwrap()
}

pub fn setup() -> (bitcoind::BitcoinD, bitcoin::Address, CoreConnect) {
    let node = bitcoind::BitcoinD::new(bitcoind::exe_path().unwrap()).unwrap();
    let node_address = node
        .client
        .get_new_address(None, None)
        .unwrap()
        .assume_checked();
    let core_connect = CoreConnect::from((&node, Network::Regtest));
    (node, node_address, core_connect)
}
