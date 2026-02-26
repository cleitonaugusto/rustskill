use clap::{Parser, Subcommand};
use comfy_table::Table;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::time::Duration;
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
    Audit {
        #[arg(long)]
        fix: bool,
    },
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
            println!(
                "{} Consultando o Marketplace Global...",
                style("🔍").yellow()
            );
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
                    style("✔ Disponível").green().to_string(),
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

                    // --- LÓGICA PREMIUM ---
                    if entry.premium {
                        match &cfg.token {
                            Some(token) => {
                                println!("{} Validando acesso premium...", style("🔑").cyan());
                                if !downloader::validate_token(token).await? {
                                    println!("{} Token inválido ou expirado.", style("❌").red());
                                    return Ok(());
                                }
                            }
                            None => {
                                println!(
                                    "{} Skill Premium! Faça login primeiro.",
                                    style("❌").red()
                                );
                                return Ok(());
                            }
                        }
                    }

                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?,
                    );
                    pb.set_message(format!(
                        "Injetando inteligência: {}...",
                        style(alias).cyan()
                    ));
                    pb.enable_steady_tick(Duration::from_millis(80));

                    // --- AQUI ESTÁ A CORREÇÃO: Passamos o token como segundo argumento ---
                    let skill_content = downloader::fetch_skill(&entry.id, cfg.token).await?;
                    pb.finish_and_clear();

                    installer::install_to_cursor(
                        &skill_content.instruction,
                        &skill_content.file_name,
                        &skill_content.name,
                    )?;
                }
                None => {
                    println!("{} Skill '{}' não encontrada.", style("❌").red(), alias);
                }
            }
        }
        Commands::Audit { fix } => {
            println!(
                "{} Analisando ecossistemas Python, Go, Rust e Node...",
                style("🔍").yellow()
            );

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}")?);
            pb.set_message("Escaneando DNA poliglota do projeto...");
            pb.enable_steady_tick(Duration::from_millis(80));

            let mut extensions = HashSet::new();
            let mut dependencies = HashSet::new();

            // 1. Scan de Arquivos (Extensões & Raio-X de Código v0.4.0)
            for entry in WalkDir::new(".")
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !["target", "node_modules", ".git", "venv", "dist", "build"]
                        .contains(&name.as_ref())
                })
                .flatten()
            {
                if entry.file_type().is_file() {
                    let path = entry.path();

                    // Captura Extensão
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        extensions.insert(ext_lower.clone());

                        // RAIO-X: Se for arquivo de código, faz o Deep Scan de Imports
                        if ["rs", "py", "go", "js", "ts", "tsx"].contains(&ext_lower.as_str()) {
                            if let Ok(code) = fs::read_to_string(path) {
                                for line in code.lines().take(100) {
                                    // Analisa as primeiras 100 linhas (onde ficam os imports)
                                    let line = line.trim();

                                    // Padrão Python/JS/TS: import x ou from x import
                                    if line.starts_with("import ") || line.starts_with("from ") {
                                        let parts: Vec<&str> = line.split_whitespace().collect();
                                        if parts.len() >= 2 {
                                            let dep = parts[1]
                                                .split('.')
                                                .next()
                                                .unwrap()
                                                .replace([';', '"', '\''], "");
                                            dependencies.insert(dep.to_lowercase());
                                        }
                                    }
                                    // Padrão Rust: use x::y
                                    else if line.starts_with("use ") {
                                        if let Some(dep) = line.split_whitespace().nth(1) {
                                            let dep =
                                                dep.split("::").next().unwrap().replace(';', "");
                                            dependencies.insert(dep.to_lowercase());
                                        }
                                    }
                                    // Padrão Go: import "x"
                                    else if line.starts_with("import \"") {
                                        let dep = line.replace(
                                            ['i', 'm', 'p', 'o', 'r', 't', ' ', '"', ';'],
                                            "",
                                        );
                                        if let Some(short_name) = dep.split('/').last() {
                                            dependencies.insert(short_name.to_lowercase());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Scan Híbrido de Manifestos (Otimizado v0.3.0)

            // --- RUST (Cargo.toml) ---
            if let Ok(content) = fs::read_to_string("Cargo.toml") {
                if let Ok(cargo) = content.parse::<toml::Value>() {
                    for sec in ["dependencies", "dev-dependencies"] {
                        if let Some(deps) = cargo.get(sec).and_then(|d| d.as_table()) {
                            for name in deps.keys() {
                                dependencies.insert(name.to_lowercase());
                            }
                        }
                    }
                }
            }

            // --- JS/TS (package.json) ---
            if let Ok(content) = fs::read_to_string("package.json") {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    for key in ["dependencies", "devDependencies"] {
                        if let Some(deps) = pkg.get(key).and_then(|d| d.as_object()) {
                            for (name, _) in deps {
                                dependencies.insert(name.to_lowercase());
                            }
                        }
                    }
                }
            }

            // --- PYTHON (requirements.txt) ---
            if let Ok(content) = fs::read_to_string("requirements.txt") {
                for line in content
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                {
                    let dep = line
                        .splitn(2, |c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                        .next()
                        .unwrap()
                        .trim();
                    dependencies.insert(dep.to_lowercase());
                }
            }

            // --- GO (go.mod) ---
            if let Ok(content) = fs::read_to_string("go.mod") {
                for line in content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with("//"))
                {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "require" {
                        let full_path = parts[1].to_lowercase();
                        dependencies.insert(full_path.clone());

                        if let Some(short_name) = full_path.split('/').last() {
                            dependencies.insert(short_name.to_string());
                        }
                    } else if parts.len() >= 1
                        && line.contains('/')
                        && !["module", "go", "replace"].contains(&parts[0])
                    {
                        let full_path = parts[0].to_lowercase();
                        dependencies.insert(full_path.clone());
                        if let Some(short_name) = full_path.split('/').last() {
                            dependencies.insert(short_name.to_string());
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

            // --- CORREÇÃO AQUI: Definimos o registry antes de usar no loop ---
            let registry = downloader::fetch_registry().await?;

            let mut table = Table::new();
            table.set_header(vec!["Categoria", "Skill Recomendada", "Motivo", "Status"]);

            let mut count_missing = 0;
            // Agora o 'registry' existe e o loop abaixo vai funcionar
            for skill in &registry {
                let mut should_recommend = false;
                let mut reasons = Vec::new();

                let id_lower = skill.id.to_lowercase();

                // 1. Checagem por Triggers Específicos
                if let Some(triggers) = &skill.triggers {
                    for trigger in triggers {
                        let t_lower = trigger.to_lowercase();
                        // Checa se a dependência existe ou se um arquivo com esse nome/extensão existe
                        if dependencies.contains(&t_lower) || extensions.contains(&t_lower) {
                            should_recommend = true;
                            reasons.push(format!("Gatilho '{}' detectado", trigger));
                        }
                    }
                }

                // 2. Checagem por Ecossistema
                if !should_recommend {
                    if id_lower.contains("rust") && extensions.contains("rs") {
                        should_recommend = true;
                        reasons.push("Ecossistema Rust detectado".to_string());
                    } else if id_lower.contains("python") && extensions.contains("py") {
                        should_recommend = true;
                        reasons.push("Ecossistema Python detectado".to_string());
                    } else if id_lower.contains("go") && extensions.contains("go") {
                        should_recommend = true;
                        reasons.push("Ecossistema Go detectado".to_string());
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
                        style(&skill.category).magenta().to_string(),
                        style(&skill.id).cyan().bold().to_string(),
                        style(reasons.join(", ")).dim().to_string(),
                        status,
                    ]);
                }
            }

            println!("\n{table}");

            if count_missing > 0 {
                if *fix {
                    println!(
                        "\n{} Iniciando Auto-Cura de vanguarda...",
                        style("🛠️").cyan()
                    );

                    println!(
                        "{} Debug: {} extensões e {} dependências mapeadas.",
                        style("ℹ").blue(),
                        extensions.len(),
                        dependencies.len()
                    );

                    let _ = fs::create_dir_all(".cursor/rules");
                    let cfg: Config = confy::load("rustskill", None).unwrap_or_default();

                    for skill in &registry {
                        let file_id = skill.id.replace("/", "-");

                        if installed_skills.contains(&file_id) {
                            continue;
                        }

                        let mut should_install = false;
                        let id_l = skill.id.to_lowercase();

                        // 1. Checagem de Extensões
                        if (id_l.contains("rust") && extensions.contains("rs"))
                            || (id_l.contains("python") && extensions.contains("py"))
                            || (id_l.contains("go") && extensions.contains("go"))
                        {
                            should_install = true;
                        }

                        // 2. Checagem de Triggers
                        if !should_install {
                            if let Some(triggers) = &skill.triggers {
                                for t in triggers {
                                    if dependencies.contains(&t.to_lowercase()) {
                                        should_install = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if should_install {
                            println!("{} Baixando skill: {}...", style("⏳").blue(), skill.id);

                            match downloader::fetch_skill(&skill.id, cfg.token.clone()).await {
                                Ok(content) => {
                                    if let Err(e) = installer::install_to_cursor(
                                        &content.instruction,
                                        &content.file_name,
                                        &content.name,
                                    ) {
                                        println!(
                                            "{} Erro ao instalar {}: {}",
                                            style("❌").red(),
                                            skill.id,
                                            e
                                        );
                                    }
                                }
                                Err(e) => {
                                    println!(
                                        "{} Erro ao baixar {}: {}",
                                        style("❌").red(),
                                        skill.id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    println!("\n{} Projeto blindado com sucesso!", style("✨").yellow());
                }
            } else {
                println!(
                    "\n{} Diagnóstico: {} vulnerabilidades encontradas.",
                    style("⚠️").yellow(),
                    count_missing
                );
                println!(
                    "Rode {} para auto-cura imediata.",
                    style("rustskill audit --fix").green()
                );
            }
        }
        Commands::Info { alias } => {
            let registry = downloader::fetch_registry().await?;
            if let Some(skill) = registry.iter().find(|s| &s.id == alias) {
                println!(
                    "\n{} Detalhes da Skill: {}",
                    style("📦").cyan(),
                    style(alias).bold().yellow()
                );
                println!("{} Categoria: {}", style("📁").magenta(), skill.category);
                println!(
                    "{} Acesso: {}",
                    style("🎫").blue(),
                    if skill.premium {
                        "💎 Premium"
                    } else {
                        "Grátis"
                    }
                );
                println!(
                    "{} Endpoint: {}\n",
                    style("🔗").dim(),
                    style(&skill.url).underlined()
                );
            } else {
                println!("{} Skill '{}' não encontrada.", style("❌").red(), alias);
            }
        }

        Commands::Login { token } => {
            println!(
                "{} Verificando credenciais de vanguarda...",
                style("🔑").cyan()
            );

            if downloader::validate_token(&token).await? {
                let cfg = Config {
                    token: Some(token.clone()),
                };
                confy::store("rustskill", None, cfg)?;
                println!(
                    "{} Autenticação bem-sucedida! Acesso Premium liberado.",
                    style("✅").green()
                );
            } else {
                println!(
                    "{} Falha na autenticação. Verifique seu token em {}",
                    style("❌").red(),
                    style("https://rustskill.com").underlined()
                );
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
                println!(
                    "{} Atualizado para {}!",
                    style("✔").green(),
                    status.version()
                );
            } else {
                println!(
                    "{} Versão {} já é a mais recente.",
                    style("✔").green(),
                    env!("CARGO_PKG_VERSION")
                );
            }
        }
    }
    Ok(())
}
