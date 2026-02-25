use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct SkillPayload {
    pub name: String,
    pub instruction: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillInfo {
    pub url: String,
    pub category: String,
    pub premium: bool,
}

pub type Registry = HashMap<String, SkillInfo>;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/cleitonaugusto/rustskill-registry/main/registry.json";

pub async fn fetch_registry() -> anyhow::Result<Registry> {
    let response = reqwest::get(REGISTRY_URL).await?;
    if !response.status().is_success() {
        anyhow::bail!("Falha no Catálogo (Status: {}).", response.status());
    }
    let registry = response.json::<Registry>().await?;
    Ok(registry)
}

pub async fn fetch_skill(input: &str) -> anyhow::Result<SkillPayload> {
    let client = reqwest::Client::new();
    let target_url = if input.starts_with("http") {
        input.to_string()
    } else {
        let registry = fetch_registry().await?;
        let info = registry.get(input)
            .ok_or_else(|| anyhow::anyhow!("Skill '{}' não encontrada!", input))?;
        info.url.clone()
    };

    let response = client.get(&target_url).send().await?;
    let skill = response.json::<SkillPayload>().await?;
    Ok(skill)
}