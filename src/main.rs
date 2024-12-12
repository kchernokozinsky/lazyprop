use clap::Parser;
use cli::Cli;
use color_eyre::Result;

use lazyprop::{app::App, cli};
#[tokio::main]
async fn main() -> Result<()> {
    lazyprop::errors::init()?;
    lazyprop::logging::init()?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
