use clap::Parser;
use dinasty::{inner_main, stdin::read_stdin, Cli};

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let cli = Cli::parse();
    let stdin = cli.command.needs_stdin().then(|| read_stdin());
    match inner_main(cli, stdin) {
        Ok(r) => println!("{r}"),
        Err(e) => eprintln!("{e}"),
    }
}
