use console::style;
use std::fs;
use std::path::Path;

/// Instala a instrução da skill no diretório do Cursor com blindagem de diretórios e cabeçalho de proteção
pub fn install_to_cursor(content: &str, file_name: &str, skill_name: &str) -> anyhow::Result<()> {
    // 1. Validação de Contexto (O "Norte" do projeto)
    let current_dir = std::env::current_dir()?;
    if !current_dir.join("package.json").exists() && !current_dir.join("Cargo.toml").exists() {
        println!(
            "{} {}",
            style("⚠️ ").yellow(),
            style("Aviso: Nenhum manifesto de projeto (package.json/Cargo.toml) detectado.")
                .yellow()
        );
    }

    // 2. Definição e Criação de Ambiente (A "Magia" da Automação)
    let rules_path = Path::new(".cursor").join("rules");

    if !rules_path.exists() {
        println!(
            "{} Estrutura .cursor/rules não detectada. Criando ambiente de vanguarda...",
            style("📁").cyan()
        );
        fs::create_dir_all(&rules_path)?;
    }

    // 3. SEGURANÇA: Sanitizar o file_name (Evita Path Traversal)
    let safe_file_name = Path::new(file_name)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Nome de arquivo inválido na definição da skill"))?;

    let full_path = rules_path.join(safe_file_name);

    // 4. Cabeçalho de Gerenciamento (Identidade RustSkill)
    let managed_content = format!(
        "# Gerenciado pelo RustSkill - Skill: {}\n# Modificações manuais podem ser sobrescritas em atualizações.\n\n{}",
        skill_name,
        content
    );

    // 5. Gravação Final da Inteligência
    fs::write(&full_path, managed_content)?;

    println!(
        "{} Skill '{}' blindada e registrada em: {}",
        style("📂").blue(),
        style(skill_name).bold(),
        style(full_path.display()).dim()
    );

    Ok(())
}
