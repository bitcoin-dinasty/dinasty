use std::collections::HashMap;

use bitcoin::{address::NetworkUnchecked, Address, Amount, Network};
use bitcoind::bitcoincore_rpc::{core_rpc_json::CreateRawTransactionInput, RpcApi};
use dinasty::client_ext::ClientExt;

use crate::{create_random_extended, setup};

/// A wsh descriptor (as of now June 2023, the only function supporting miniscript)
/// with a older clause, causing the script to be spendable only after the output is older than x
/// blocks
#[test]
fn wsh_older() {
    let (node, node_address, core_connect) = setup();

    let block_wait = 1000;
    let alice_prv = create_random_extended(Network::Regtest);

    let alice_wallet = format!("wsh(and_v(v:pk({alice_prv}/0/*),older({block_wait})))");
    let alice_wallet = node.client.add_checksum(&alice_wallet).unwrap();

    let alice_wallet_change = format!("wsh(and_v(v:pk({alice_prv}/1/*),older({block_wait})))");
    let alice_wallet_change = node.client.add_checksum(&alice_wallet_change).unwrap();

    let alice_client = node
        .client
        .create_blank_wallet("alice", &core_connect, false, None)
        .unwrap();

    alice_client
        .import_descriptor(&alice_wallet, false)
        .unwrap();
    alice_client
        .import_descriptor(&alice_wallet_change, true)
        .unwrap();

    let alice_address = alice_client
        .get_new_address(None, None)
        .unwrap()
        .assume_checked();

    node.client.generate_to_address(1, &alice_address).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap(); // enough to be spendable, not enought for CSV

    let unspents = alice_client
        .list_unspent(None, None, None, None, None)
        .unwrap();

    let unspent = &unspents[0];
    let input = CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: Some(block_wait),
    };

    let mut outputs = HashMap::new();
    outputs.insert(node_address.to_string(), Amount::from_sat(100_000));

    let psbt = alice_client
        .wallet_create_funded_psbt(&[input], &outputs, None, None, None)
        .unwrap();

    let psbt = alice_client
        .wallet_process_psbt(&psbt.psbt, None, None, None)
        .unwrap();
    assert!(psbt.complete);

    let psbt = alice_client.finalize_psbt(&psbt.psbt, None).unwrap();
    let non_owned_address: Address<NetworkUnchecked> =
        "bcrt1q6jaw5xdxwf2wvmymjsgnypmea3eaap8m0jsmfk9u5wt43nfpy9gspg6ze0"
            .parse()
            .unwrap();
    let non_owned_address = non_owned_address.assume_checked();
    let mut remaining = block_wait as i64 - 100;
    while remaining > 0 {
        if let Ok(result) = alice_client.test_mempool_accept(&[psbt.hex.as_ref().unwrap()]) {
            assert!(!result[0].allowed);
            assert_eq!(result[0].reject_reason, Some("non-BIP68-final".to_string()));

            if let Ok(_) = node.client.generate_to_address(100, &non_owned_address) {
                remaining -= 100;
            } // make the CSV expire
        }
    }

    let result = alice_client
        .test_mempool_accept(&[psbt.hex.as_ref().unwrap()])
        .unwrap();
    assert!(result[0].allowed);
    assert_eq!(
        result[0].fees.as_ref().unwrap().base,
        Amount::from_sat(1540)
    );
}
