use clap::Parser;

mod router;
mod liteweb;
mod instagram;
mod telegram;
mod whatsapp;
mod search;

#[derive(Parser)]
#[command(name = "red-server", about = "RedBrowser content proxy server")]
struct Args {
    /// Listen address for the protocol service
    #[arg(short, long, default_value = "127.0.0.1:8400")]
    listen: String,

    /// Max concurrent page renders
    #[arg(long, default_value = "20")]
    max_renders: usize,

    /// Image quality (1-100)
    #[arg(long, default_value = "75")]
    image_quality: u8,

    /// Image max width in pixels
    #[arg(long, default_value = "800")]
    image_max_width: u32,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp_millis()
        .init();

    let config = router::ServerConfig {
        image_quality: args.image_quality,
        image_max_width: args.image_max_width,
        max_renders: args.max_renders,
    };

    let server = router::Server::new(config).await;
    log::info!("RedBrowser server starting on {}", args.listen);

    // For now, start a simple TCP listener that accepts connections,
    // reads length-prefixed CBOR requests, and writes responses.
    // In production this will be wired into nooshdaroo-server's smux streams.
    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    log::info!("Listening on {}", args.listen);

    loop {
        let (stream, peer) = listener.accept().await?;
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(e) = router::handle_connection(stream, &server).await {
                log::debug!("connection from {}: {}", peer, e);
            }
        });
    }
}
