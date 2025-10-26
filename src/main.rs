use clap::Parser;
use just_a_simple_wayland_screenshot_tool::{Result, cli};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::DEBUG.into()),
            )
            .init();
    }

    cli::execute(args).await
}
