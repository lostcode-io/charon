pub mod client;
pub mod server;

use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Commands,

    #[clap(short, long, default_value = "info")]
    log: String,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Server {
        #[clap(short, long)]
        port: u16,
    },

    Client {
        #[clap(short, long)]
        port: String,

        #[clap(short, long)]
        server: String,
    },
}

fn main() {
    let args = Args::parse();

    std::env::set_var("RUST_APP_LOG", args.clone().log);
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    let args_clone = args.clone();
    match args.cmd {
        Commands::Server { port } => server::run(args_clone, port),
        Commands::Client { server, port } => {
            client::run(args_clone, server, format!("127.0.0.1:{}", port))
        }
    }
}
