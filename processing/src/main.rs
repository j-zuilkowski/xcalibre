use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xcalibre_processing::config::Config;

#[derive(Parser)]
#[command(name = "xcalibre", version, about = "xCalibre ebook processor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest an ebook file into the local job database
    Ingest {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Manage xcalibre-server authentication token
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Sync library with xcalibre-server
    Sync,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Store a service token in the OS keychain
    SetToken {
        #[arg(value_name = "TOKEN")]
        token: String,
    },
    /// Show whether a token is stored (never prints the value)
    Status,
    /// Remove the stored token from the OS keychain
    RemoveToken,
}

const SERVICE: &str = "xcalibre";
const ACCOUNT: &str = "xs_token";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest { path } => {
            let config = match Config::load() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Config error: {}", e);
                    std::process::exit(1);
                }
            };
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(
                    sqlx::sqlite::SqliteConnectOptions::new()
                        .filename(&config.db_path)
                        .create_if_missing(true),
                )
                .await?;
            sqlx::migrate!("src/db/migrations").run(&pool).await?;
            let result = xcalibre_processing::pipeline::local::import_local_book(&pool, &path).await?;
            println!("Imported: {} ({})", path.display(), result.job_id);
        }
        Commands::Auth { action } => match action {
            AuthAction::SetToken { token } => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                entry
                    .set_password(&token)
                    .map_err(|e| anyhow::anyhow!("failed to store token: {}", e))?;
                println!("Token stored.");
            }
            AuthAction::Status => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                match entry.get_password() {
                    Ok(_)  => println!("Token: stored"),
                    Err(_) => println!("Token: not set"),
                }
            }
            AuthAction::RemoveToken => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                entry
                    .delete_password()
                    .map_err(|e| anyhow::anyhow!("failed to remove token: {}", e))?;
                println!("Token removed.");
            }
        },
        Commands::Sync => {
            let config = match Config::load() {
                Ok(c)  => c,
                Err(e) => {
                    eprintln!("Config error: {}", e);
                    std::process::exit(1);
                }
            };
            let client = match xcalibre_api::client::ApiClient::from_keyring(&config.xs_url)
            {
                Ok(c) => c,
                Err(_) => {
                    println!(
                        "No xcalibre-server token. Run `xcalibre auth set-token <TOKEN>` first."
                    );
                    return Ok(());
                }
            };
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect(&format!("sqlite:{}", config.db_path.display()))
                .await?;
            sqlx::migrate!("src/db/migrations").run(&pool).await?;
            xcalibre_processing::pipeline::sync::sync_pull(&pool, &client).await?;
            xcalibre_processing::pipeline::sync::sync_status_backprop(&pool, &client).await?;
            xcalibre_processing::pipeline::retry::run_retries(&pool, Some(&client)).await?;
            println!("Sync complete.");
        }
    }
    Ok(())
}
