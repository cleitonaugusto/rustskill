use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 1. Estrutura que representa o catálogo (Registry) -
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub id: String,        // O ID único (ex: rust/clean-code)
    pub category: String,  // Categoria para a tabela
    pub url: String,       // URL do JSON da skill real
    pub premium: bool,     // Se exige token
}

/// 2. Estrutura do conteúdo real da Skill (Payload)
#[derive(Deserialize, Serialize, Debug)]
pub struct SkillPayload {
    pub name: String,
    pub instruction: String,
    pub file_name: String,
}

// URL base sem o parâmetro de cache (será adicionado dinamicamente)
const BASE_REGISTRY_URL: &str = "https://raw.githubusercontent.com/cleitonaugusto/rustskill-registry/main/registry.json";

/// BUSCA O CATÁLOGO COMPLETO (Usado pelo comando List e Add)
pub async fn fetch_registry() -> anyhow::Result<Vec<SkillEntry>> {
    // Gerar um timestamp para "furar" o cache do GitHub de forma profissional
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let url_with_cache_bust = format!("{}?t={}", BASE_REGISTRY_URL, ts);

    let response = reqwest::get(&url_with_cache_bust).await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "❌ Falha ao acessar o Catálogo Global (Status: {}). Verifique sua conexão.",
            response.status()
        );
    }

    let registry = response.json::<Vec<SkillEntry>>().await.map_err(|e| {
        anyhow::anyhow!("Erro ao processar o catálogo. Certifique-se de que o registro no GitHub está no formato de LISTA []. Detalhe: {}", e)
    })?;

    Ok(registry)
}

/// BAIXA A INTELIGÊNCIA DA SKILL (Resolve o ID ou URL)
pub async fn fetch_skill(input: &str) -> anyhow::Result<SkillPayload> {
    let client = reqwest::Client::new();

    // Lógica inteligente: Resolve a URL
    let target_url = if input.starts_with("http") {
        input.to_string()
    } else {
        // Busca na lista de entradas pelo ID (alias)
        let registry = fetch_registry().await?;
        let entry = registry.iter().find(|s| s.id == input)
            .ok_or_else(|| anyhow::anyhow!(
                "Skill '{}' não encontrada! Rode 'rustskill list' para ver as opções disponíveis.",
                input
            ))?;

        entry.url.clone()
    };

    // Download do conteúdo (Payload) da skill
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let skill_url = format!("{}?t={}", target_url, ts);

    let response = client.get(&skill_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("❌ Erro ao baixar a skill: O link no catálogo parece estar offline ou quebrado.");
    }

    let skill = response.json::<SkillPayload>().await?;
    Ok(skill)
}