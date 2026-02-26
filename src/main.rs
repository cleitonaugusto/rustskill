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
                    // Carregamos a config aqui para ter o token disponível
                    let cfg: Config = confy::load("rustskill", None).unwrap_or_default();

                    // --- LÓGICA PREMIUM (Nós já temos isso, mantenha a validação de vanguarda) ---
                    if entry.premium {
                        match &cfg.token {
                            Some(token) => {
                                println!("{} Validando acesso premium...", style("🔑").cyan());
                                if !downloader::validate_token(token).await? {
                                    println!("{} Token inválido ou expirado.", style("❌").red());
                                    return Ok(());
                                }
                            },
                            None => {
                                println!("{} Skill Premium! Faça login primeiro.", style("❌").red());
                                return Ok(());
                            }
                        }
                    }

                    let pb = ProgressBar::new_spinner();
                    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
                    pb.set_message(format!("Injetando inteligência: {}...", style(alias).cyan()));
                    pb.enable_steady_tick(Duration::from_millis(80));

                    // --- AQUI ESTÁ A CORREÇÃO: Passamos o token como segundo argumento ---
                    let skill_content = downloader::fetch_skill(&entry.id, cfg.token).await?;
                    pb.finish_and_clear();

                    installer::install_to_cursor(&skill_content.instruction, &skill_content.file_name, &skill_content.name)?;

                    let _ = track_telemetry(&skill_content.name).await;
                    println!("{} Skill {} instalada com sucesso!", style("✔").green(), style(&skill_content.name).bold());
                },
                None => {
                    println!("{} Skill '{}' não encontrada.", style("❌").red(), alias);
                }
            }
        }
        Commands::Audit => {
            println!("{} Analisando a estrutura profunda e manifestos...", style("🔍").yellow());

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
            pb.set_message("Escaneando DNA do projeto...");
            pb.enable_steady_tick(Duration::from_millis(80));

            // 1. Scan de Arquivos (O que já temos)
            let mut extensions = HashSet::new();
            for entry in WalkDir::new(".").into_iter().filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target" && name != "node_modules" && name != ".git"
            }).flatten() {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                        extensions.insert(ext.to_lowercase());
                    }
                }
            }

            // 2. EVOLUÇÃO V0.2.0: Scan de Dependências Híbrido (Rust & JS/TS)
            let mut dependencies = HashSet::new();

            // --- SCAN RUST (Cargo.toml) ---
            if let Ok(cargo_content) = fs::read_to_string("Cargo.toml") {
                if let Ok(cargo_value) = cargo_content.parse::<toml::Value>() {
                    if let Some(deps) = cargo_value.get("dependencies").and_then(|d| d.as_table()) {
                        for dep_name in deps.keys() {
                            dependencies.insert(dep_name.to_lowercase());
                        }
                    }
                    if let Some(dev_deps) = cargo_value.get("dev-dependencies").and_then(|d| d.as_table()) {
                        for dep_name in dev_deps.keys() {
                            dependencies.insert(dep_name.to_lowercase());
                        }
                    }
                }
            }

            // --- SCAN JAVASCRIPT/TYPESCRIPT (package.json) ---
            if let Ok(pkg_content) = fs::read_to_string("package.json") {
                // Usamos o turbofish ::<serde_json::Value> para o Rust saber o tipo exato
                if let Ok(pkg_value) = serde_json::from_str::<serde_json::Value>(&pkg_content) {
                    let target_keys = ["dependencies", "devDependencies"];

                    for key in target_keys {
                        if let Some(deps) = pkg_value.get(key).and_then(|d| d.as_object()) {
                            for (dep_name, _) in deps {
                                dependencies.insert(dep_name.to_lowercase());
                            }
                        }
                    }
                }
            }

            pb.finish_and_clear();

            // 3. Mapeamento de Skills Instaladas
            let mut installed_skills = HashSet::new();
            if let Ok(entries) = fs::read_dir(".cursor/rules") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        installed_skills.insert(name.replace(".mdc", ""));
                    }
                }
            }

            let registry = downloader::fetch_registry().await?;
            let mut table = Table::new();
            table.set_header(vec!["Categoria", "Skill Recomendada", "Motivo", "Status"]);

            let mut count_missing = 0;
            for skill in registry {
                let mut should_recommend = false;
                let mut reason = String::new();

                // Gatilho por Extensão (Legado)
                let id_lower = skill.id.to_lowercase();
                if id_lower.contains("rust") && extensions.contains("rs") {
                    should_recommend = true;
                    reason = "Arquivos .rs detectados".to_string();
                }

                // Gatilho por Dependência (Vanguarda v0.2.0)
                if let Some(triggers) = &skill.triggers {
                    for trigger in triggers {
                        if dependencies.contains(&trigger.to_lowercase()) {
                            should_recommend = true;
                            reason = format!("Dependência '{}' detectada", trigger);
                            break;
                        }
                    }
                }

                if should_recommend {
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
                        style(reason).dim().to_string(),
                        status
                    ]);
                }
            }

            println!("\n{table}");

            if count_missing > 0 {
                println!("\n{} Diagnóstico: {} vulnerabilidades de governança encontradas.", style("⚠️").yellow(), count_missing);
                println!("Rode {} para blindar seu projeto com as regras oficiais.", style("rustskill add <alias>").green());
            } else {
                println!("\n{} Projeto 100% em conformidade com os padrões de vanguarda!", style("✨").yellow());
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
            println!("{} Verificando credenciais de vanguarda...", style("🔑").cyan());

            if downloader::validate_token(&token).await? {
                let cfg = Config { token: Some(token.clone()) };
                confy::store("rustskill", None, cfg)?;
                println!("{} Autenticação bem-sucedida! Acesso Premium liberado.", style("✅").green());
            } else {
                println!("{} Falha na autenticação. Verifique seu token em {}",
                         style("❌").red(),
                         style("https://rustskill.com").underlined());
            }
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