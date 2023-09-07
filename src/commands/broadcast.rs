use crate::core_connect::{ConnectError, CoreConnect};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoind::bitcoincore_rpc::{self, RpcApi};

#[derive(thiserror::Error, Debug)]
pub enum BroadcastError {
    #[error(transparent)]
    Connect(#[from] ConnectError),

    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),
}
pub fn broadcast(
    core_connect: &CoreConnect,
    psbts: &[PartiallySignedTransaction],
) -> Result<String, BroadcastError> {
    let client = core_connect.client()?;
    let mut result = vec![];

    for psbt in psbts {
        let tx = psbt.clone().extract_tx();
        let txid = client.send_raw_transaction(&tx)?;
        result.push(txid);
    }

    Ok(result
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n"))
}
