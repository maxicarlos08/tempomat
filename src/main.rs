use clap::Parser;
use tempomat::args::TempomatCLI;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = TempomatCLI::parse();

    println!("Arguments: {args:#?}");
}
