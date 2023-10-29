#![doc = include_str!("../README.md")]

use age::secrecy::ExposeSecret;
use anyhow::Context;
use bitcoin::Network;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use commands::{Commands, CoreConnectOptional, Seed};
use error::Error;
use std::{fs, io::Read, str::FromStr};
use stdin::StdinData;

use crate::core_connect::CoreConnect;

pub mod client_ext;
pub mod commands;
pub mod core_connect;
pub mod error;
pub mod psbts_serde;
pub mod stdin;
pub mod stdout;
pub mod test_util; // pub because needed in doctest

type Descriptor = miniscript::Descriptor<miniscript::DescriptorPublicKey>;

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

pub fn inner_main(cli: Cli, stdin: Option<StdinData>) -> anyhow::Result<Vec<u8>> {
    Ok(match cli.command {
        Commands::Seed { codex32_id } => {
            let dices = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;

            commands::seed(&dices, codex32_id)?
                .to_string()
                .as_bytes()
                .to_vec()
        }

        Commands::Descriptor { public, account } => {
            let key = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let seed = Seed::from_str(&key)?;

            commands::descriptor(seed, cli.network, account, public)?
                .to_string()
                .as_bytes()
                .to_vec()
        }

        Commands::Import {
            wallet_name,
            with_private_keys,
        } => {
            let descriptor = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;
            commands::import(&core_connect, &descriptor, &wallet_name, with_private_keys)?
                .as_bytes()
                .to_vec()
        }
        Commands::Refresh {
            wallet_name,
            older_than_blocks,
        } => {
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;
            let psbts = commands::refresh(&core_connect, &wallet_name, older_than_blocks)?;
            psbts_serde::serialize(&psbts)
        }
        Commands::Locktime {
            from_wallet_name,
            to_wallet_name,
            locktime_future,
        } => {
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;
            let psbts = commands::locktime(
                &core_connect,
                &from_wallet_name,
                &to_wallet_name,
                locktime_future,
            )?;
            psbts_serde::serialize(&psbts)
        }
        Commands::Sign {
            wallet_name,
            psbt_file,
        } => {
            let descriptor = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let mut file_content = vec![];
            fs::File::open(&psbt_file)
                .with_context(|| format!("cannot open {:?}", &psbt_file))?
                .read_to_end(&mut file_content)
                .with_context(|| format!("io error on file {:?}", &psbt_file))?;

            let psbts = psbts_serde::deserialize(&file_content)?;
            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;

            let signed_psbts: Vec<_> =
                commands::sign(&core_connect, &descriptor, &wallet_name, &psbts)?;

            psbts_serde::serialize(&signed_psbts)
        }

        Commands::Broadcast => {
            let psbts = stdin.ok_or(Error::StdinExpected)?.to_psbts()?;

            let core_connect = CoreConnect::try_from((cli.core_connect, cli.network))?;

            commands::broadcast(&core_connect, &psbts)?
                .as_bytes()
                .to_vec()
        }

        Commands::Identity { private } => {
            let key = stdin.ok_or(Error::StdinExpected)?.to_single_text_line()?;
            let seed = Seed::from_str(&key)?;

            let identity = commands::identity(&seed)?;
            if private {
                identity
                    .to_string()
                    .expose_secret()
                    .to_string()
                    .as_bytes()
                    .to_vec()
            } else {
                format!("{}", identity.to_public()).as_bytes().to_vec()
            }
        }
        Commands::Qr {
            qr_version,
            border,
            empty_lines,
            label,
        } => {
            let content = stdin.ok_or(Error::StdinExpected)?.to_string()?;

            commands::qr(&content, qr_version, border, empty_lines, label)?
                .as_bytes()
                .to_vec()
        }

        Commands::GenerateCompletion { shell } => {
            let mut result = vec![];
            generate(shell, &mut Cli::command(), "dinasty", &mut result);
            result
        }
        Commands::B64ToBin => {
            let content = stdin.ok_or(Error::StdinExpected)?.to_multiline_string()?;
            let psbts: Result<Vec<_>, _> = content.iter().map(|e| e.parse()).collect();
            psbts_serde::serialize(&psbts?)
        }
        Commands::BinToB64 => {
            let content = stdin.ok_or(Error::StdinExpected)?.to_vec();
            let psbts = psbts_serde::deserialize(&content)?;
            psbts
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes()
                .to_vec()
        }
        Commands::Details { descriptor } => {
            let psbts = stdin.ok_or(Error::StdinExpected)?.to_psbts()?;

            let balances = commands::psbt_details(&psbts, &descriptor, cli.network)?;

            balances.to_string().as_bytes().to_vec()
        }
    })
}
