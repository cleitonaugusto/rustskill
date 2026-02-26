use clap::{Parser, Subcommand};
use console::style;
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::collections::HashSet;
use std::fs;
use serde::{Serialize, Deserialize};
use walkdir::WalkDir;

use rustskill::client::downloader;
use rustskill::core::installer;

#[derive(Parser)]
#[command(name = "rustskill", version = env!("CARGO_PKG_VERSION"), about = "AI Skills Platform - Governança de Código com IA")]
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
    /// Escaneia o projeto e sugere as skills de vanguarda necessárias
    Audit,
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
            let registry = downloader::fetch_registry().await?;
            let skill_entry = registry.iter().find(|s| &s.id == alias);

            match skill_entry {
                Some(entry) => {
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

                    let pb = ProgressBar::new_spinner();
                    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
                    pb.set_message(format!("Injetando inteligência: {}...", style(alias).cyan()));
                    pb.enable_steady_tick(Duration::from_millis(80));

                    let skill_content = downloader::fetch_skill(&entry.url).await?;
                    pb.finish_and_clear();

                    installer::install_to_cursor(&skill_content.instruction, &skill_content.file_name, &skill_content.name)?;

                    let _ = track_telemetry(&skill_content.name).await;

                    println!("{} Skill {} instalada com sucesso!", style("✔").green(), style(&skill_content.name).bold());
                },
                None => {
                    println!("{} Skill '{}' não encontrada no registro global.", style("❌").red(), alias);
                }
            }
        }

        Commands::Audit => {
            println!("{} Analisando a estrutura profunda do projeto...", style("🔍").yellow());

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
            pb.set_message("Escaneando diretórios...");
            pb.enable_steady_tick(Duration::from_millis(80));

            // 1. Scan Recursivo (Otimizado com WalkDir)
            let mut extensions = HashSet::new();
            for entry in WalkDir::new(".")
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    // Ignora diretórios que não agregam valor e pesam o scan
                    name != "target" && name != "node_modules" && name != ".git" && name != "dist"
                })
                .flatten()
            {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                        extensions.insert(ext.to_lowercase());
                    }
                }
            }

            pb.finish_and_clear();

            // 2. Mapeamento de Skills Instaladas (.cursor/rules)
            let mut installed_skills = HashSet::new();
            if let Ok(entries) = fs::read_dir(".cursor/rules") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Compara IDs normalizados (sem .mdc)
                        installed_skills.insert(name.replace(".mdc", ""));
                    }
                }
            }

            // 3. Busca o Registry e Processa Diagnóstico
            let registry = downloader::fetch_registry().await?;
            let mut table = Table::new();
            table.set_header(vec!["Categoria", "Skill Recomendada", "Status"]);

            let mut count_missing = 0;
            for skill in registry {
                let id_lower = skill.id.to_lowercase();

                // Lógica de Ativação Contextual (DNA do projeto)
                let is_needed = if id_lower.contains("rust") && extensions.contains("rs") { true }
                else if id_lower.contains("react") && (extensions.contains("tsx") || extensions.contains("jsx")) { true }
                else if id_lower.contains("sql") && extensions.contains("sql") { true }
                else if id_lower.contains("security") && (extensions.contains("env") || extensions.contains("js") || extensions.contains("ts")) { true }
                else if id_lower.contains("clean-arch") && (extensions.contains("rs") || extensions.contains("ts")) { true }
                else { false };

                if is_needed {
                    // Normalização do ID para verificar se o arquivo existe (ex: rust/clean-code -> rust-clean-code)
                    let file_id = skill.id.replace("/", "-");
                    let status = if installed_skills.contains(&file_id) {
                        style("✅ Protegido").green().to_string()
                    } else {
                        count_missing += 1;
                        style("❌ Ausente").red().to_string()
                    };

                    table.add_row(vec![
                        style(skill.category).magenta().to_string(),
                        style(&skill.id).cyan().bold().to_string(),
                        status
                    ]);
                }
            }

            println!("\n{table}");

            if count_missing > 0 {
                println!("\n{} Diagnóstico: {} skills recomendadas ainda não instaladas.", style("⚠️").yellow(), count_missing);
                println!("Rode {} para blindar seu projeto.", style("rustskill add <alias>").green());
            } else {
                println!("\n{} Parabéns! O seu projeto está com a cobertura de vanguarda completa.", style("✨").yellow());
            }
        }

        Commands::Info { alias } => {
            let registry = downloader::fetch_registry().await?;
            if let Some(skill) = registry.iter().find(|s| &s.id == alias) {
                println!("\n{} Detalhes da Skill: {}", style("📦").cyan(), style(alias).bold().yellow());
                println!("{} Categoria: {}", style("📁").magenta(), skill.category);
                println!("{} Acesso: {}", style("🎫").blue(), if skill.premium { "💎 Premium" } else { "Grátis" });
                println!("{} Endpoint: {}\n", style("🔗").dim(), style(&skill.url).underlined());
            } else {
                println!("{} Skill '{}' não encontrada.", style("❌").red(), alias);
            }
        }

        Commands::Login { token } => {
            let cfg = Config { token: Some(token.clone()) };
            confy::store("rustskill", None, cfg)?;
            println!("{} Token autenticado! Acesso Premium liberado.", style("🔑").green());
        }

        Commands::Upgrade => {
            println!("{} Buscando vanguarda...", style("🔄").cyan());
            let status = self_update::backends::github::Update::configure()
                .repo_owner("cleitonaugusto")
                .repo_name("rustskill")
                .bin_name("rustskill")
                .show_download_progress(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .build()?
                .update()?;

            if status.updated() {
                println!("{} Atualizado para {}!", style("✔").green(), status.version());
            } else {
                println!("{} Versão {} já é a mais recente.", style("✔").green(), env!("CARGO_PKG_VERSION"));
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