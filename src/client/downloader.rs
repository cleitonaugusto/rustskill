use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub id: String,
    pub category: String,
    pub url: String,
    pub premium: bool,
    //pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SkillPayload {
    pub name: String,
    pub instruction: String,
    pub file_name: String,
}

const BASE_REGISTRY_URL: &str = "https://raw.githubusercontent.com/cleitonaugusto/rustskill-registry/main/registry.json";

pub async fn fetch_registry() -> anyhow::Result<Vec<SkillEntry>> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let url_with_cache_bust = format!("{}?t={}", BASE_REGISTRY_URL, ts);
    let response = reqwest::get(&url_with_cache_bust).await?;

    if !response.status().is_success() {
        anyhow::bail!("❌ Falha ao acessar o Catálogo Global (Status: {}).", response.status());
    }

    Ok(response.json::<Vec<SkillEntry>>().await?)
}

pub async fn fetch_skill(input: &str) -> anyhow::Result<SkillPayload> {
    let client = reqwest::Client::new();

    let target_url = if input.starts_with("http") {
        input.to_string()
    } else {
        let registry = fetch_registry().await?;
        let entry = registry.iter().find(|s| s.id == input)
            .ok_or_else(|| anyhow::anyhow!("Skill '{}' não encontrada!", input))?;
        entry.url.clone()
    };

    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    // Lógica para evitar duplicar o '?' na URL
    let separator = if target_url.contains('?') { "&" } else { "?" };
    let skill_url = format!("{}{}{}t={}", target_url, separator, "", ts);

    let response = client.get(&skill_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("❌ Erro ao baixar a skill: O link parece estar offline.");
    }

    Ok(response.json::<SkillPayload>().await?)
}