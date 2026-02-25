use clap::{Parser, Subcommand};
use console::style;
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use serde::{Serialize, Deserialize};

// Certifique-se de que no seu module 'downloader' as Structs
// correspondam a essa nova estrutura de Lista.
use rustskill::client::downloader;
use rustskill::core::installer;

#[derive(Parser)]
#[command(name = "rustskill", version = env!("CARGO_PKG_VERSION"), about = "AI Skills Platform")]
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
    /// Lista o marketplace de skills (Global Registry)
    List,
    /// Instala uma skill usando o Alias (ex: rust/clean-code)
    Add { alias: String },
    /// Atualiza o rustskill para a versão mais recente
    Upgrade,
    /// Login com Token Premium para acessar skills restritas
    Login { token: String },
    /// Mostra detalhes técnicos de uma skill específica
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
            table.set_header(vec!["Categoria", "Skill Alias (ID)", "Acesso", "Status"]);

            for skill in registry {
                let access_tag = if skill.premium {
                    style("💎 Premium").yellow().bold().to_string()
                } else {
                    style("🆓 Grátis").dim().to_string()
                };

                table.add_row(vec![
                    style(skill.category).magenta().to_string(),
                    style(skill.id).cyan().bold().to_string(),
                    access_tag,
                    style("✔ Disponível").green().to_string()
                ]);
            }
            println!("{table}");
        }

        Commands::Add { alias } => {
            // 1. Verificar Registry para encontrar a URL pelo Alias
            let registry = downloader::fetch_registry().await?;
            let skill_entry = registry.iter().find(|s| &s.id == alias);

            match skill_entry {
                Some(entry) => {
                    // 2. Verificação de Token para Skills Premium
                    if entry.premium {
                        let cfg: Config = confy::load("rustskill", None)?;
                        if cfg.token.is_none() {
                            println!("{} Esta skill é {}! Use: {} login <token>",
                                     style("❌").red(),
                                     style("PREMIUM").yellow().bold(),
                                     style("rustskill").bold());
                            return Ok(());
                        }
                    }

                    // 3. Download e Instalação
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
                    pb.set_message(format!("Injetando inteligência: {}...", style(alias).cyan()));
                    pb.enable_steady_tick(Duration::from_millis(80));

                    let skill_content = downloader::fetch_skill(&entry.url).await?;
                    pb.finish_and_clear();

                    installer::install_to_cursor(&skill_content.instruction, &skill_content.file_name, &skill_content.name)?;

                    // Telemetria (opcional/silenciosa)
                    let _ = track_telemetry(&skill_content.name).await;

                    println!("{} Skill {} instalada com sucesso!", style("✔").green(), style(&skill_content.name).bold());
                },
                None => {
                    println!("{} Skill '{}' não encontrada no registro global.", style("❌").red(), alias);
                }
            }
        }

        Commands::Info { alias } => {
            let registry = downloader::fetch_registry().await?;
            if let Some(skill) = registry.iter().find(|s| &s.id == alias) {
                println!("\n{} Detalhes da Skill: {}", style("📦").cyan(), style(alias).bold().yellow());
                println!("{} Categoria: {}", style("📁").magenta(), skill.category);
                println!("{} Acesso: {}", style("🎫").blue(), if skill.premium { "💎 Premium" } else { "Grátis" });
                println!("{} Endpoint: {}\n", style("🔗").dim(), style(&skill.url).underlined());
                println!("{}", style("Para instalar, rode:").dim());
                println!("  rustskill add {}\n", style(alias).green());
            } else {
                println!("{} Skill '{}' não encontrada.", style("❌").red(), alias);
            }
        }

        Commands::Login { token } => {
            let cfg = Config { token: Some(token.clone()) };
            confy::store("rustskill", None, cfg)?;
            println!("{} Token autenticado com sucesso! Acesso Premium liberado.", style("🔑").green());
        }

        Commands::Upgrade => {
            println!("{} Buscando novas tecnologias...", style("🔄").cyan());
            // Lógica de self-update mantida...
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
                println!("{} Você já está na vanguarda da versão {}.", style("✔").green(), env!("CARGO_PKG_VERSION"));
            }
        }
    }
    Ok(())
}

async fn track_telemetry(skill_name: &str) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build().unwrap_or_default();

    let _ = client.post("https://api.rustskill.com/telemetry")
        .json(&serde_json::json!({
            "event": "skill_installed",
            "skill": skill_name,
            "platform": std::env::consts::OS,
            "version": env!("CARGO_PKG_VERSION")
        })).send().await;
}