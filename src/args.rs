use clap::Parser;

/// mgdocker - A simple web interface for managing docker containers and images
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Port that the server will run on
    #[arg(short, long, default_value_t = 8080)]
    pub port: u32,
    /// Host that the server will run on
    #[arg(long, default_value = "localhost")]
    pub host: String,
}
