use clap::Parser;
/// Struct for parsing command line arguments
#[derive(Parser, Debug)]
#[command(name = "Node Connector")]
#[command(about = "A CLI for connecting to an Ethereum node", long_about = None)]
pub struct Cli {
    /// NodeRecord string
    #[arg(long)]
    pub node_record: String,
}
