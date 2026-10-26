mod api;
mod config;
mod engine;
mod providers;
mod tools;

use clap::{Parser, Subcommand};
use config::Config;
use engine::{AgentEngine, ChatSessionRequest};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "hermes-claw")]
#[command(about = "Fast, unrestricted AI connector & computer interaction bridge", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server REST & WebSocket API bridge
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Interactive CLI chat session directly in terminal
    Chat {
        #[arg(short, long, default_value = "openai")]
        provider: String,
        #[arg(short, long, default_value = "gpt-4o")]
        model: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let mut config = Config::default();

    match cli.command {
        Some(Commands::Chat { provider, model }) => {
            config.default_provider = provider;
            config.default_model = model;
            let engine = AgentEngine::new(config.clone());
            println!("--- Hermes-Claw CLI Chat Session ---");
            println!("Provider: {} | Model: {}", config.default_provider, config.default_model);
            println!("Type 'exit' to quit.\n");

            let mut history = vec![];
            let stdin = std::io::stdin();

            loop {
                use std::io::Write;
                print!("User > ");
                std::io::stdout().flush()?;

                let mut line = String::new();
                if stdin.read_line(&mut line)? == 0 {
                    break;
                }
                let input = line.trim();
                if input.eq_ignore_ascii_case("exit") || input.is_empty() {
                    break;
                }

                let req = ChatSessionRequest {
                    provider: config.default_provider.clone(),
                    model: config.default_model.clone(),
                    user_message: input.to_string(),
                    history: history.clone(),
                    max_tool_iterations: Some(5),
                };

                println!("Hermes-Claw thinking & executing tools...");
                match engine.process_chat(req).await {
                    Ok(resp) => {
                        for tool in &resp.executed_tools {
                            println!("\n[Tool Call: {}]", tool.tool_name);
                            println!("[Result]\n{}", tool.output);
                        }
                        println!("\nHermes > {}\n", resp.assistant_reply);
                        history = resp.updated_history;
                    }
                    Err(err) => {
                        println!("\nError: {}\n", err);
                    }
                }
            }
        }
        Some(Commands::Serve { port }) => {
            config.server_port = port;
            run_server(config).await?;
        }
        None => {
            run_server(config).await?;
        }
    }

    Ok(())
}

async fn run_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let port = config.server_port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let engine = Arc::new(AgentEngine::new(config));
    let app = api::create_router(engine);

    tracing::info!("Hermes-Claw server active on http://{}", addr);
    println!("🚀 Hermes-Claw listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
