use std::fs;
use console::style;

/// Instala a instrução da skill no diretório do Cursor
pub fn install_to_cursor(content: &str, file_name: &str, skill_name: &str) -> anyhow::Result<()> {
    let mut path = std::env::current_dir()?;

    // --- DIFERENCIAL RUSTSKILL: Validação de Contexto Profissional ---
    // Verificamos se estamos em um projeto real antes de "sujar" a pasta
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

    // Adiciona o nome do arquivo ao caminho final
    path.push(file_name);

    // Escreve o conteúdo (instrução da skill) no arquivo
    fs::write(&path, content)?;

    println!(
        "{} Skill '{}' registrada localmente.",
        style("📂").blue(),
        skill_name
    );

    Ok(())
}