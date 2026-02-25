use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Estrutura que representa o conteúdo real da Skill (o arquivo .json final)
#[derive(Deserialize, Serialize, Debug)]
pub struct SkillPayload {
    pub name: String,
    pub instruction: String,
    pub file_name: String,
}

/// Estrutura que representa a metadata no Registry (o catálogo)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillInfo {
    pub url: String,
    pub category: String,
    pub premium: bool,
}

pub type Registry = HashMap<String, SkillInfo>;

// URL do Registry com o truque do cache (?v=1) para garantir que as atualizações sejam instantâneas
const REGISTRY_URL: &str = "https://raw.githubusercontent.com/cleitonaugusto/rustskill-registry/main/registry.json?v=1";

/// Função para o comando LIST: Busca o catálogo completo no GitHub
pub async fn fetch_registry() -> anyhow::Result<Registry> {
    let response = reqwest::get(REGISTRY_URL).await?;

    if !response.status().is_success() {
        anyhow::bail!("Falha ao acessar o Catálogo Global (Status: {}).", response.status());
    }

    let registry = response.json::<Registry>().await?;
    Ok(registry)
}

/// Função para o comando ADD: Resolve o Alias e baixa o Payload da Skill
pub async fn fetch_skill(input: &str) -> anyhow::Result<SkillPayload> {
    let client = reqwest::Client::new();

    // 1. Resolve a URL: Se o usuário passar um link, usa ele. Se passar "rust/clean-code", busca no Registry.
    let target_url = if input.starts_with("http") {
        input.to_string()
    } else {
        let registry = fetch_registry().await?;
        let info = registry.get(input)
            .ok_or_else(|| anyhow::anyhow!(
                "Skill '{}' não encontrada no catálogo! Tente rodar 'rustskill list' para ver as opções.",
                input
            ))?;

        info.url.clone()
    };

    // 2. Download do Payload real da Skill
    let response = client.get(&target_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Erro ao baixar a skill: O link no catálogo parece estar quebrado.");
    }

    let skill = response.json::<SkillPayload>().await?;
    Ok(skill)
}