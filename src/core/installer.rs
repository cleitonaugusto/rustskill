use console::style;
use std::env;
use std::fs;

/// Instala a instrução da skill no diretório do Cursor com blindagem de diretórios
pub fn install_to_cursor(content: &str, file_name: &str, skill_name: &str) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let rules_path = current_dir.join(".cursor").join("rules");

    // 2. Criação Robusta: Garante que a estrutura existe antes de gravar
    if !rules_path.exists() {
        fs::create_dir_all(&rules_path)?;
    }

    let mut safe_name = if file_name.trim().is_empty() || file_name == "null" {
        skill_name.replace("/", "-").to_lowercase()
    } else {
        file_name.to_lowercase()
    };

    // Remove caracteres proibidos em sistemas de arquivos
    safe_name = safe_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");

    // Garante a extensão .mdc
    if !safe_name.ends_with(".mdc") {
        safe_name.push_str(".mdc");
    }

    let full_path = rules_path.join(&safe_name);

    let managed_content = format!(
        "---\ndescription: Skill gerenciada pelo RustSkill: {}\nglobs: [\"**/*\"]\n---\n\n{}",
        skill_name, content
    );

    fs::write(&full_path, managed_content)?;

    // 6. LOG DE CONFIRMAÇÃO OBRIGATÓRIO (O seu Debug visual)
    println!(
        "{} Skill '{}' injetada com sucesso!",
        style("🚀").blue(),
        style(skill_name).cyan()
    );
    println!(
        "   {} Caminho: {}",
        style("↳").dim(),
        style(full_path.display()).dim().italic()
    );

    Ok(())
}
