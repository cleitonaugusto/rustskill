use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub id: String,
    pub category: String,
    pub url: String,
    pub premium: bool,
    pub triggers: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SkillPayload {
    pub name: String,
    pub instruction: String,
    pub file_name: String,
}

const BASE_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/cleitonaugusto/rustskill-registry/main/registry.json";
const API_BASE_URL: &str = "https://api.rustskill.com/v1";

/// Valida o token do usuário contra a API do RustSkill
pub async fn validate_token(token: &str) -> anyhow::Result<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let response = client
        .get(format!("{}/auth/validate", API_BASE_URL))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(USER_AGENT, "rustskill-cli")
        .send()
        .await?;

    Ok(response.status().is_success())
}

pub async fn fetch_registry() -> anyhow::Result<Vec<SkillEntry>> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let url_with_cache_bust = format!("{}?t={}", BASE_REGISTRY_URL, ts);
    let response = reqwest::get(&url_with_cache_bust).await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "❌ Falha ao acessar o Catálogo Global (Status: {}).",
            response.status()
        );
    }

    Ok(response.json::<Vec<SkillEntry>>().await?)
}

/// Busca o conteúdo da skill.
/// Se for premium, utiliza o token para baixar da API privada.
pub async fn fetch_skill(input: &str, token: Option<String>) -> anyhow::Result<SkillPayload> {
    let client = reqwest::Client::builder()
        .user_agent("rustskill-cli")
        .build()?;

    // 1. Localiza a skill no registry para saber se é Premium
    let registry = fetch_registry().await?;
    let entry = registry
        .iter()
        .find(|s| s.id == input || s.url == input)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' não encontrada no catálogo!", input))?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 2. Lógica de Busca Protegida (Diferencial de Mercado)
    let response = if entry.premium {
        let tk =
            token.ok_or_else(|| anyhow::anyhow!("Esta skill exige um Token Premium ativo."))?;

        // Chamada para a API Privada: o conteúdo não está no GitHub!
        client
            .get(format!("{}/skills/content/{}", API_BASE_URL, entry.id))
            .header(AUTHORIZATION, format!("Bearer {}", tk))
            .query(&[("t", ts.to_string())])
            .send()
            .await?
    } else {
        // Chamada para URL pública (GitHub/S3)
        let separator = if entry.url.contains('?') { "&" } else { "?" };
        let skill_url = format!("{}{}t={}", entry.url, separator, ts);
        client.get(&skill_url).send().await?
    };

    if !response.status().is_success() {
        if response.status() == 401 || response.status() == 403 {
            anyhow::bail!("❌ Acesso negado: Seu token não tem permissão para esta skill.");
        }
        anyhow::bail!("❌ Erro ao baixar a skill (Status: {}).", response.status());
    }

    Ok(response.json::<SkillPayload>().await?)
}
