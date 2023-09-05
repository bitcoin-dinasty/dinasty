use std::str::FromStr;

use bitcoin::{psbt::PartiallySignedTransaction, Address, Network, Txid};
use bitcoind::bitcoincore_rpc::{
    core_rpc_json::{
        AddressType, GetDescriptorInfoResult, ImportDescriptors, ImportMultiResult, Timestamp,
    },
    jsonrpc::serde_json::{to_value, Value},
    Auth, Client, Error, RpcApi,
};

use crate::core_connect::CoreConnect;

pub trait ClientExt {
    fn add_checksum(&self, desc_without_checksum: &str) -> Result<String, Error>;

    fn import_descriptor(
        &self,
        descriptor: &str,
        internal: bool,
    ) -> Result<Vec<ImportMultiResult>, Error>;

    fn get_new_bech32m_address(&self, network: Network) -> Result<Address, Error>;
    fn send_all(&self, rec: &Address) -> Txid;
    fn wallet_passphrase(&self, passphrase: &str);
    fn wallet_lock(&self);

    fn create_blank_wallet(
        &self,
        wallet_name: &str,
        core_connect: &CoreConnect,
        disable_private_keys: bool,
        passphrase: Option<&str>,
    ) -> Result<Client, Error>;

    fn test_mempool_accept_psbts(
        &self,
        psbts: &[PartiallySignedTransaction],
    ) -> Result<Vec<bool>, Error>;

    fn prepare_psbt_to(&self, address: &Address, satoshi: u64) -> Result<String, Error>;
}

impl ClientExt for Client {
    fn add_checksum(&self, desc_without_checksum: &str) -> Result<String, Error> {
        let GetDescriptorInfoResult { checksum, .. } =
            self.get_descriptor_info(desc_without_checksum)?;

        Ok(format!("{desc_without_checksum}#{checksum}"))
    }

    fn import_descriptor(
        &self,
        descriptor: &str,
        internal: bool,
    ) -> Result<Vec<ImportMultiResult>, Error> {
        self.import_descriptors(ImportDescriptors {
            descriptor: descriptor.to_owned(),
            timestamp: Timestamp::Now,
            active: Some(true),
            range: None,
            next_index: None,
            internal: Some(internal),
            label: None,
        })
    }

    fn get_new_bech32m_address(&self, network: Network) -> Result<Address, Error> {
        let address = self.get_new_address(None, Some(AddressType::Bech32m))?;
        address
            .require_network(network)
            .map_err(|e| Error::ReturnedError(e.to_string()))
    }

    fn send_all(&self, addr: &Address) -> Txid {
        let recs = to_value([Value::from(addr.to_string())]).unwrap();
        let result: Value = self.call("sendall", &[recs]).unwrap();
        assert!(result.get("complete").unwrap().as_bool().unwrap());

        Txid::from_str(result.get("txid").unwrap().as_str().unwrap()).unwrap()
    }

    fn wallet_passphrase(&self, passphrase: &str) {
        let val: Value = self
            .call("walletpassphrase", &[passphrase.into(), 10.into()])
            .unwrap();
        println!("{val:?}");
    }

    fn wallet_lock(&self) {
        let val: Value = self.call("walletlock", &[]).unwrap();
        println!("{val:?}");
    }

    fn create_blank_wallet(
        &self,
        wallet_name: &str,
        core_connect: &CoreConnect,
        disable_private_keys: bool,
        passphrase: Option<&str>,
    ) -> Result<Client, Error> {
        self.create_wallet(
            wallet_name,
            Some(disable_private_keys),
            Some(true),
            passphrase,
            None,
        )?;
        let url = format!("http://{}/wallet/{}", core_connect.node_socket, wallet_name);

        Client::new(
            &url,
            Auth::CookieFile(core_connect.node_cookie_path.to_path_buf()),
        )
    }

    fn test_mempool_accept_psbts(
        &self,
        psbts: &[PartiallySignedTransaction],
    ) -> Result<Vec<bool>, Error> {
        let txs: Vec<_> = psbts.iter().map(|p| p.clone().extract_tx()).collect();
        let txs: Vec<_> = txs.iter().collect();
        Ok(self
            .test_mempool_accept(&txs)?
            .iter()
            .map(|r| r.allowed)
            .collect())
    }

    fn prepare_psbt_to(&self, address: &Address, satoshi: u64) -> Result<String, Error> {
        let mut outputs = std::collections::HashMap::new();
        let to = bitcoin::Amount::from_sat(satoshi);
        outputs.insert(address.to_string(), to);

        let psbt = self
            .wallet_create_funded_psbt(&[], &outputs, None, None, None)
            .unwrap();
        Ok(psbt.psbt)
    }
}
