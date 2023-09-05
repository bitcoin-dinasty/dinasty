use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

use bitcoin::Network;
use bitcoind::{
    bitcoincore_rpc::{self, Auth, Client, RpcApi},
    BitcoinD, ConnectParams,
};

use crate::CoreConnectOptional;

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Invalid network: node:{node} cli:{cli}")]
    InvalidNetwork { node: Network, cli: Network },

    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),
}

pub struct CoreConnect {
    pub node_socket: SocketAddrV4,

    pub node_cookie_path: PathBuf,

    pub network: Network,
}

impl CoreConnect {
    /// Create an rpc client, checking it's on the same network as the given `network`
    pub(crate) fn client(&self) -> Result<Client, ConnectError> {
        let client = Client::new(
            &format!("http://{}", self.node_socket),
            Auth::CookieFile(self.node_cookie_path.clone()),
        )?;
        Self::check_network(&client, self.network)?;

        Ok(client)
    }

    /// Create an rpc client using `wallet_name`, checking it's on the same network as the given `network`
    pub fn client_with_wallet(&self, wallet_name: &str) -> Result<Client, ConnectError> {
        let client = Client::new(
            &format!("http://{}/wallet/{}", self.node_socket, wallet_name),
            Auth::CookieFile(self.node_cookie_path.clone()),
        )?;
        Self::check_network(&client, self.network)?;

        Ok(client)
    }

    fn check_network(client: &Client, network: Network) -> Result<(), ConnectError> {
        let info = client.get_blockchain_info()?;
        let node_network = Network::from_core_arg(&info.chain).unwrap();
        if node_network != network {
            Err(ConnectError::InvalidNetwork {
                node: node_network,
                cli: network,
            })
        } else {
            Ok(())
        }
    }

    pub fn sh_params(&self) -> String {
        format!(
            "--network {} --node-socket {} --node-cookie-path {}",
            self.network,
            self.node_socket,
            self.node_cookie_path.display()
        )
    }
}

impl From<(&ConnectParams, Network)> for CoreConnect {
    fn from(value: (&ConnectParams, Network)) -> Self {
        Self {
            node_socket: value.0.rpc_socket,
            node_cookie_path: value.0.cookie_file.clone(),
            network: value.1,
        }
    }
}

impl From<(&BitcoinD, Network)> for CoreConnect {
    fn from(value: (&BitcoinD, Network)) -> Self {
        Self {
            node_socket: value.0.params.rpc_socket,
            node_cookie_path: value.0.params.cookie_file.clone(),
            network: value.1,
        }
    }
}

impl TryFrom<(CoreConnectOptional, Network)> for CoreConnect {
    type Error = CookieError;

    fn try_from(value: (CoreConnectOptional, Network)) -> Result<Self, Self::Error> {
        let (option, network) = value;
        Ok(Self {
            node_socket: option
                .node_socket
                .unwrap_or_else(|| default_node_socket(network)),
            node_cookie_path: match option.node_cookie_path {
                Some(p) => p,
                None => default_node_cookie_path(network)?,
            },
            network,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CookieError {
    #[error("Cookie path {0} does not exist")]
    PathNotExist(PathBuf),

    #[error("Cannot retrieve home directory, specify cookie path manually")]
    HomeDirUnknown,
}
fn default_node_cookie_path(network: Network) -> Result<PathBuf, CookieError> {
    let mut path = home::home_dir().ok_or(CookieError::HomeDirUnknown)?;
    path.push(".bitcoin");
    match network {
        Network::Bitcoin => (),
        Network::Testnet => path.push("testnet3"),
        Network::Signet => path.push("signet"),
        Network::Regtest => path.push("regtest"),
        _ => panic!("Unknown network variant"),
    };
    path.push(".cookie");
    if path.exists() {
        Ok(path)
    } else {
        Err(CookieError::PathNotExist(path))
    }
}

fn default_node_socket(network: Network) -> SocketAddrV4 {
    let port = match network {
        Network::Bitcoin => 8332,
        Network::Testnet => 18332,
        Network::Signet => 38332,
        Network::Regtest => 18443,
        _ => panic!("Unknown network variant"),
    };
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)
}
