use aibar_providers::models::ProviderId;
use aibar_providers::registry;
use aibar_providers::traits::{FetchContext, Runtime, SourceMode};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "aibar",
    version,
    about = "AIBar CLI - AI usage monitoring from the command line"
)]
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
        format: OutputFormat,

        /// Source mode for fetching
        #[arg(short, long, default_value = "auto")]
        source: SourceModeArg,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// List all supported providers
    Providers {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
    /// Show version information
    Version,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, ValueEnum)]
enum SourceModeArg {
    Auto,
    Web,
    Cli,
    Api,
}

impl From<SourceModeArg> for SourceMode {
    fn from(arg: SourceModeArg) -> Self {
        match arg {
            SourceModeArg::Auto => SourceMode::Auto,
            SourceModeArg::Web => SourceMode::Web,
            SourceModeArg::Cli => SourceMode::Cli,
            SourceModeArg::Api => SourceMode::Api,
        }
    }
}

fn resolve_provider(name: &str) -> Option<ProviderId> {
    for id in ProviderId::all() {
        let desc = registry::get_descriptor(*id);
        if desc.cli_config.name == name
            || desc.cli_config.aliases.contains(&name)
            || desc.metadata.display_name.to_lowercase() == name.to_lowercase()
        {
            return Some(*id);
        }
    }
    None
}

#[allow(dead_code)]
fn print_usage_text(id: ProviderId, snapshot: &aibar_providers::models::UsageSnapshot) {
    let desc = registry::get_descriptor(id);
    println!("{}", desc.metadata.display_name);

    if let Some(ref primary) = snapshot.primary {
        println!(
            "  {}: {:.1}%",
            desc.metadata.session_label, primary.used_percent
        );
        if let Some(ref reset_desc) = primary.reset_description {
            println!("    {}", reset_desc);
        }
    }

    if let Some(ref secondary) = snapshot.secondary {
        println!(
            "  {}: {:.1}%",
            desc.metadata.weekly_label, secondary.used_percent
        );
        if let Some(ref reset_desc) = secondary.reset_description {
            println!("    {}", reset_desc);
        }
    }

    if let Some(ref cost) = snapshot.provider_cost {
        println!(
            "  Cost: ${:.2} / ${:.2} {}",
            cost.used, cost.limit, cost.currency_code
        );
    }

    if let Some(ref identity) = snapshot.identity {
        if let Some(ref email) = identity.email {
            println!("  Account: {}", email);
        }
        if let Some(ref plan) = identity.plan {
            println!("  Plan: {}", plan);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Usage {
            provider,
            format,
            source,
            verbose,
        } => {
            let ctx = FetchContext {
                runtime: Runtime::Cli,
                source_mode: source.into(),
                include_credits: true,
                verbose,
                ..Default::default()
            };

            if let Some(name) = provider {
                let id = resolve_provider(&name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Unknown provider '{}'. Run 'aibar providers' to see available providers.",
                        name
                    )
                })?;

                if verbose {
                    eprintln!("Fetching usage for {}...", id);
                }

                // TODO: Call the provider's fetch pipeline when strategies are fully wired
                eprintln!(
                    "Provider fetch not yet wired for {}. Use 'aibar providers' to list available providers.",
                    id
                );
                // Placeholder: show provider info
                let desc = registry::get_descriptor(id);
                match format {
                    OutputFormat::Text => {
                        println!("{} ({})", desc.metadata.display_name, desc.cli_config.name);
                        if let Some(url) = desc.metadata.dashboard_url {
                            println!("  Dashboard: {}", url);
                        }
                    }
                    OutputFormat::Json => {
                        let info = serde_json::json!({
                            "provider": id,
                            "displayName": desc.metadata.display_name,
                            "dashboardUrl": desc.metadata.dashboard_url,
                            "status": "not_yet_wired",
                        });
                        println!("{}", serde_json::to_string_pretty(&info)?);
                    }
                }
                let _ = ctx; // will be used when fetch pipeline is wired
            } else {
                match format {
                    OutputFormat::Text => {
                        println!("No provider specified. Use --provider <name>.");
                        println!("Run 'aibar providers' to see available providers.");
                    }
                    OutputFormat::Json => {
                        println!("{{\"error\": \"No provider specified\"}}");
                    }
                }
            }
        }
        Commands::Providers { format } => match format {
            OutputFormat::Text => {
                println!("Supported providers ({}):", ProviderId::all().len());
                println!();
                for id in ProviderId::all() {
                    let desc = registry::get_descriptor(*id);
                    let enabled = if desc.metadata.default_enabled {
                        " [default]"
                    } else {
                        ""
                    };
                    println!(
                        "  {:<15} {}{}",
                        desc.cli_config.name, desc.metadata.display_name, enabled
                    );
                }
            }
            OutputFormat::Json => {
                let providers: Vec<_> = ProviderId::all()
                    .iter()
                    .map(|id| {
                        let desc = registry::get_descriptor(*id);
                        serde_json::json!({
                            "id": id,
                            "name": desc.cli_config.name,
                            "displayName": desc.metadata.display_name,
                            "defaultEnabled": desc.metadata.default_enabled,
                            "dashboardUrl": desc.metadata.dashboard_url,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&providers)?);
            }
        },
        Commands::Version => {
            println!("aibar-cli {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
