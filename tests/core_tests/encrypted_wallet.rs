use crate::{create_random_extended, setup};
use bitcoin::{Amount, Network};
use bitcoind::bitcoincore_rpc::RpcApi;
use dinasty::client_ext::ClientExt;
use std::collections::HashMap;

/// Test a wallet with keys encrypted with a passphrase
#[test]
fn encrypted_wallet() {
    let (node, node_address, core_connect) = setup();
    let passphrase = "thisisapassphrase";
    let xprv = create_random_extended(Network::Regtest);

    // setup encrypted wallet
    let desc = format!("tr({xprv}/0/*)");
    let desc = node.client.add_checksum(&desc).unwrap();
    let desc_change = format!("tr({xprv}/1/*)");
    let desc_change = node.client.add_checksum(&desc_change).unwrap();
    let client = node
        .client
        .create_blank_wallet("encrypted", &core_connect, false, Some(passphrase))
        .unwrap();
    client.wallet_passphrase(passphrase);
    client.import_descriptor(&desc, false).unwrap();
    client.import_descriptor(&desc_change, true).unwrap();
    client.wallet_lock();
    let first = client.get_new_bech32m_address(Network::Regtest).unwrap();

    node.client.generate_to_address(1, &first).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap();

    let mut outputs = HashMap::new();
    outputs.insert(node_address.to_string(), Amount::from_sat(100_000));

    let psbt = client
        .wallet_create_funded_psbt(&[], &outputs, None, None, None)
        .unwrap();

    let error = client.wallet_process_psbt(&psbt.psbt, None, None, None);
    assert!(format!("{error:?}")
        .contains("Error: Please enter the wallet passphrase with walletpassphrase first."));

    client.wallet_passphrase(passphrase);

    let psbt = client
        .wallet_process_psbt(&psbt.psbt, None, None, None)
        .unwrap();
    let psbt = client.finalize_psbt(&psbt.psbt, None).unwrap();

    client
        .send_raw_transaction(psbt.hex.as_ref().unwrap())
        .unwrap();
}
