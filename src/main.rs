//! WMTP Server - Main Entry Point
//! 
//! WebTransport Mail Transfer Protocol Server
//! https://wmtp.online

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("wmtp_server=debug".parse().unwrap()),
        )
        .init();

    // Print banner
    print_banner();

    // Run the server
    wmtp_server::run_server().await?;

    Ok(())
}

fn print_banner() {
    info!("╔═══════════════════════════════════════════════════╗");
    info!("║                                                   ║");
    info!("║   ██╗    ██╗███╗   ███╗████████╗██████╗           ║");
    info!("║   ██║    ██║████╗ ████║╚══██╔══╝██╔══██╗          ║");
    info!("║   ██║ █╗ ██║██╔████╔██║   ██║   ██████╔╝          ║");
    info!("║   ██║███╗██║██║╚██╔╝██║   ██║   ██╔═══╝           ║");
    info!("║   ╚███╔███╔╝██║ ╚═╝ ██║   ██║   ██║               ║");
    info!("║    ╚══╝╚══╝ ╚═╝     ╚═╝   ╚═╝   ╚═╝               ║");
    info!("║                                                   ║");
    info!("║   WebTransport Mail Transfer Protocol             ║");
    info!("║   https://wmtp.online                             ║");
    info!("║                                                   ║");
    info!("╚═══════════════════════════════════════════════════╝");
}
