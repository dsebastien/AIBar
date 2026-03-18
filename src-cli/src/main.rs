use aibar_providers::models::ProviderId;
use aibar_providers::registry;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aibar", about = "AIBar CLI - AI usage monitoring")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show usage for a provider
    Usage {
        /// Provider name (e.g., claude, codex, cursor)
        #[arg(short, long)]
        provider: Option<String>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// List all supported providers
    Providers,
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Usage { provider, format: _ } => {
            if let Some(name) = provider {
                println!("Fetching usage for {}...", name);
                // TODO: implement provider fetch
            } else {
                println!("Fetching usage for all enabled providers...");
                // TODO: implement all-provider fetch
            }
        }
        Commands::Providers => {
            println!("Supported providers:");
            for id in ProviderId::all() {
                let desc = registry::get_descriptor(*id);
                println!("  {} - {}", desc.cli_config.name, desc.metadata.display_name);
            }
        }
        Commands::Version => {
            println!("aibar-cli {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
