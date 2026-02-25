use std::fs;
use std::path::Path;
use console::style;

/// Instala a instrução da skill no diretório do Cursor com cabeçalho de proteção
pub fn install_to_cursor(content: &str, file_name: &str, skill_name: &str) -> anyhow::Result<()> {
    let mut path = std::env::current_dir()?;

    // --- DIFERENCIAL RUSTSKILL: Validação de Contexto Profissional ---
    if !path.join("package.json").exists() && !path.join("Cargo.toml").exists() {
        println!(
            "{} {}",
            style("⚠️ ").yellow(),
            style("Aviso: Nenhum manifesto de projeto (package.json/Cargo.toml) detectado.").yellow()
        );
    }
    // -----------------------------------------------------------------

    // Monta o caminho: .cursor/rules/
    path.push(".cursor");
    path.push("rules");

    // Cria os diretórios de forma recursiva (se não existirem)
    fs::create_dir_all(&path)?;

    // SEGURANÇA: Sanitizar o file_name para evitar Path Traversal
    let safe_file_name = Path::new(file_name)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Nome de arquivo inválido na definição da skill"))?;

    path.push(safe_file_name);

    // --- MELHORIA: Cabeçalho de Gerenciamento ---
    let managed_content = format!(
        "# Gerenciado pelo RustSkill - Skill: {}\n# Modificações manuais podem ser sobrescritas em atualizações.\n\n{}",
        skill_name,
        content
    );

    // Escreve o conteúdo (instrução da skill) no arquivo
    fs::write(&path, managed_content)?;

    println!(
        "{} Skill '{}' registrada localmente em: {}",
        style("📂").blue(),
        skill_name,
        path.display()
    );

    Ok(())
}