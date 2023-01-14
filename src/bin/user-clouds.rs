use std::net::SocketAddr;

use clap::Parser;
use mendes::application::Server;

use user_clouds_sample::App;

// cargo run --bin user-clouds serve

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Options::parse().cmd {
        Command::Serve(opts) => serve(opts).await,
    }
}

async fn serve(opts: Serve) -> anyhow::Result<()> {
    let app = App::new().await?;

    let addr = SocketAddr::new("0.0.0.0".parse()?, opts.port);
    println!("server listening on http://{} ...", addr);
    app.serve(&addr).await?;
    Ok(())
}

/// epoxide server
#[derive(Parser)]
struct Options {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    Serve(Serve),
}

/// Epoxide server for basic data endpoints
#[derive(Parser)]
struct Serve {
    /// port to listen on
    #[clap(short, long, default_value = "8080")]
    port: u16,
}
