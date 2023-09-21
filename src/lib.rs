#![doc = include_str!("../README.md")]

use age::{secrecy::ExposeSecret, x25519::Identity};
use bitcoin::{psbt::PartiallySignedTransaction, Network};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use commands::{Commands, CoreConnectOptional};
use error::Error;
use key_origin::XprvWithSource;
use std::{
    fmt::Display,
    fs,
    io::Read,
    path::Path,
    str::{from_utf8, FromStr},
};
use stdin::StdinData;

use crate::core_connect::CoreConnect;

pub mod client_ext;
pub mod commands;
pub mod core_connect;
pub mod error;
pub mod key_origin;
pub mod stdin;
pub mod test_util; // pub because needed in doctest

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, env)]
    #[arg(default_value_t = Network::Bitcoin)]
    pub network: Network,

    #[clap(flatten)]
    pub core_connect: CoreConnectOptional,
}

pub fn inner_main(cli: Cli, stdin: Option<StdinData>) -> Result<String, Error> {
    Ok(match cli.command {
        Commands::Seed { codex32_id } => {
            let dices = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;

            commands::seed(&dices, codex32_id)?
        }
        Commands::Xkey { path } => {
            let mnemonic_or_codex32 = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;

            commands::key(&mnemonic_or_codex32, &path, cli.network)?
        }
        Commands::Import {
            wallet_name,
            with_private_keys,
        } => {
            let descriptor = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;
            commands::import(&core_connect, &descriptor, &wallet_name, with_private_keys)?
        }
        Commands::Descriptor {
            public,
            only_external,
        } => {
            let key = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let key = XprvWithSource::from_str(&key)?;

            commands::descriptor(key, public, only_external)
        }
        Commands::Refresh {
            wallet_name,
            older_than_blocks,
        } => {
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;
            let psbts = commands::refresh(&core_connect, &wallet_name, older_than_blocks)?;
            let pstbs_text = psbts.iter().map(ToString::to_string).collect::<Vec<_>>();
            pstbs_text.join("\n").to_string()
        }
        Commands::Locktime {
            wallet_name,
            to_public_descriptor: heir_descriptor_public,
            locktime_future,
        } => {
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;
            let psbts = commands::locktime(
                &core_connect,
                &wallet_name,
                &heir_descriptor_public,
                locktime_future,
            )?;
            psbts
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        }
        Commands::Sign {
            wallet_name,
            psbt_file,
        } => {
            let descriptor = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;

            let psbt = fs::read_to_string(&psbt_file)?;

            let psbts = psbt
                .split('\n')
                .map(|p| PartiallySignedTransaction::from_str(p))
                .collect::<Result<Vec<_>, _>>()?;
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;

            let signed_psbts: Vec<_> =
                commands::sign(&core_connect, &descriptor, &wallet_name, &psbts)?
                    .iter()
                    .map(ToString::to_string)
                    .collect();

            signed_psbts.join("\n").to_string()
        }

        Commands::Broadcast { psbt_file } => {
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;

            let psbt = fs::read_to_string(&psbt_file)?;

            let psbts = psbt
                .split('\n')
                .map(|p| PartiallySignedTransaction::from_str(p))
                .collect::<Result<Vec<_>, _>>()?;

            commands::broadcast(&core_connect, &psbts)?
        }

        Commands::Identity { private } => {
            let key = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let key = XprvWithSource::from_str(&key)?;

            let identity = commands::identity(&key, cli.network)?;
            if private {
                identity.to_string().expose_secret().to_string()
            } else {
                format!("{}", identity.to_public())
            }
        }
        Commands::Encrypt { recipients } => {
            let plain_text = stdin.ok_or(Error::StdinExpected)?.to_vec();

            let armored_cipher_text = commands::encrypt(&plain_text, recipients)?;
            armored_cipher_text.to_string()
        }
        Commands::Decrypt { encrypted_file } => {
            let str = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let identity = Identity::from_str(&str);
            let identity = match identity {
                Ok(identity) => identity,
                Err(id_e) => match XprvWithSource::from_str(&str) {
                    Ok(xprv) => commands::identity(&xprv, cli.network)?,
                    Err(xprv_e) => {
                        return Err(Error::DecryptError {
                            input: str.to_string(),
                            identity_parse_err: id_e.to_string(),
                            xprv_parse_err: xprv_e,
                        })
                    }
                },
            };

            let file_content = std::fs::read_to_string(encrypted_file)?;
            commands::decrypt(&file_content, &identity)?
        }
        Commands::Qr {
            file,
            qr_version,
            border,
            empty_lines,
            avoid_structured,
        } => {
            let file_content = if file == Path::new("-") {
                stdin.ok_or(Error::StdinExpected)?.to_string()?
            } else {
                std::fs::read_to_string(file)?
            };

            commands::qr(
                &file_content,
                qr_version,
                border,
                empty_lines,
                avoid_structured,
            )?
        }
        Commands::Bech32 {
            file,
            lowercase,
            hrp,
            with_checksum,
        } => {
            let file_content = if file == Path::new("-") {
                stdin.ok_or(Error::StdinExpected)?.to_vec()
            } else {
                let mut buffer = vec![];
                let mut file = fs::File::open(file)?;
                file.read_to_end(&mut buffer)?;
                buffer
            };
            let hrp = hrp.unwrap_or("data".to_string());
            commands::bech32(&hrp, &file_content, lowercase, with_checksum)?
        }
        Commands::GenerateCompletion { shell } => {
            let mut result = vec![];
            generate(shell, &mut Cli::command(), "dinasty", &mut result);
            from_utf8(&result)
                .expect("generate completion non utf8 chars")
                .to_string()
        }
    })
}

#[derive(thiserror::Error, Debug)]
pub struct IncompatibleNetwork {
    left: Network,
    right: Network,
}
fn xpub_compatibility(left: Network, right: Network) -> Result<(), IncompatibleNetwork> {
    match (left, right) {
        (Network::Bitcoin, Network::Bitcoin) => Ok(()),
        (_, Network::Bitcoin) => Err(IncompatibleNetwork { left, right }),
        (Network::Bitcoin, _) => Err(IncompatibleNetwork { left, right }),
        _ => Ok(()),
    }
}
impl Display for IncompatibleNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Incompatible networks {} {}", self.left, self.right)
    }
}
