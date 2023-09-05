use std::io::BufRead;

use clap::Parser;
use dinasty::{inner_main, Cli};

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let cli = Cli::parse();
    let stdin = if cli.command.needs_stdin() {
        read_stdin()
    } else {
        vec![]
    };
    match inner_main(cli, &stdin) {
        Ok(r) => println!("{r}"),
        Err(e) => eprintln!("{e}"),
    }
}

fn read_stdin() -> Vec<String> {
    let mut lines = std::io::stdin().lock().lines();
    let mut user_input = vec![];
    while let Some(Ok(line)) = lines.next() {
        user_input.push(line);
    }
    user_input
}
