use clap::{Parser, Subcommand};
use console::style;
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use serde::{Serialize, Deserialize};

use rustskill::client::downloader;
use rustskill::core::installer;

#[derive(Parser)]
#[command(name = "rustskill", version = "1.0.0", about = "AI Skills Platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lista o marketplace de skills
    List,
    /// Instala uma skill (ex: rust/clean-code)
    Add { url: String },
    /// Atualiza o rustskill para a versão mais recente
    Upgrade,
    /// Login com Token Premium
    Login { token: String },
    /// Mostra detalhes de uma skill específica
    Info { alias: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List => {
            println!("{} Consultando o Marketplace Global...", style("🔍").yellow());
            let registry = downloader::fetch_registry().await?;

            let mut table = Table::new();
            table.set_header(vec!["Categoria", "Skill Alias", "Acesso", "Status"]);

            for (alias, info) in registry {
                let access_tag = if info.premium {
                    style("★ Premium").yellow().bold().to_string()
                } else {
                    style("Grátis").dim().to_string()
                };

                table.add_row(vec![
                    style(info.category).magenta().to_string(),
                    style(alias).cyan().bold().to_string(),
                    access_tag,
                    style("✔ Disponível").green().to_string()
                ]);
            }
            println!("{table}");
        }

        Commands::Add { url } => {
            let cfg: Config = confy::load("rustskill", None)?;
            if cfg.token.is_none() {
                println!("{} Login necessário! Use: {} login <token>", style("❌").red(), style("rustskill").bold());
                return Ok(());
            }

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
            pb.set_message("Baixando inteligência...");
            pb.enable_steady_tick(Duration::from_millis(80));

            let skill = downloader::fetch_skill(url).await?;
            pb.finish_and_clear();

            installer::install_to_cursor(&skill.instruction, &skill.file_name, &skill.name)?;
            track_telemetry(&skill.name).await;

            println!("{} Skill {} injetada com sucesso!", style("✔").green(), style(&skill.name).bold());
        }

        Commands::Info { alias } => {
            let registry = downloader::fetch_registry().await?;
            if let Some(info) = registry.get(alias) {
                println!("\n{} Detalhes da Skill: {}", style("📦").cyan(), style(alias).bold().yellow());
                println!("{} Categoria: {}", style("📁").magenta(), info.category);
                println!("{} Acesso: {}", style("🎫").blue(), if info.premium { "★ Premium" } else { "Grátis" });
                println!("{} Endpoint: {}\n", style("🔗").dim(), style(&info.url).underlined());
                println!("{}", style("Para instalar, rode:").dim());
                println!("  rustskill add {}\n", style(alias).green());
            } else {
                println!("{} Skill '{}' não encontrada.", style("❌").red(), alias);
            }
        }

        Commands::Login { token } => {
            let cfg = Config { token: Some(token.clone()) };
            confy::store("rustskill", None, cfg)?;
            println!("{} Logado com sucesso!", style("🔑").green());
        }

        Commands::Upgrade => {
            println!("{} Iniciando atualização para a vanguarda...", style("🔄").cyan());

            let status = self_update::backends::github::Update::configure()
                .repo_owner("cleitonaugusto")
                .repo_name("rustskill")
                .bin_name("rustskill")
                .show_download_progress(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .build()?
                .update()?;

            if status.updated() {
                println!("{} Atualizado para {}! O futuro chegou.", style("✔").green(), status.version());
            } else {
                println!("{} Você já está usando a tecnologia mais recente.", style("✔").green());
            }
        }
    }
    Ok(())
}

async fn track_telemetry(skill_name: &str) {
    let client = reqwest::Client::new();
    let _ = client.post("https://api.rustskill.com/telemetry")
        .json(&serde_json::json!({
            "event": "skill_installed",
            "skill": skill_name,
            "platform": std::env::consts::OS,
            "version": env!("CARGO_PKG_VERSION")
        })).send().await;
}