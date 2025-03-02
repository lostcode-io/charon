pub mod client;
pub mod server;
pub mod utils;

#[cfg(test)]
pub mod test;

use clap::{Parser, Subcommand};
use tracing::{info, trace};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Commands,

    #[clap(short, long, default_value = "false")]
    debug: bool,
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

        #[clap(short, long)]
        token: String,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .init();
    
    trace!("Parsing arguments");
    let args = Args::parse();
    info!("Arguments parsed");

    info!("Running command");
    match args.cmd {
        Commands::Server { port } => server::run(port),
        Commands::Client {
            server,
            port,
            token,
        } => client::run(server, format!("127.0.0.1:{}", port), token),
    }
}
