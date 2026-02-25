use serde::{Deserialize, Serialize};

/// 1. Estrutura que representa o catálogo (Registry) - Nova versão em Lista
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

// URL do Registry com cache bust para atualizações em tempo real
const REGISTRY_URL: &str = "https://raw.githubusercontent.com/cleitonaugusto/rustskill-registry/main/registry.json?v=1";

/// BUSCA O CATÁLOGO COMPLETO (Usado pelo comando List e Add)
pub async fn fetch_registry() -> anyhow::Result<Vec<SkillEntry>> {
    let response = reqwest::get(REGISTRY_URL).await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "❌ Falha ao acessar o Catálogo Global (Status: {}). Verifique sua conexão.",
            response.status()
        );
    }


    let registry = response.json::<Vec<SkillEntry>>().await?;
    Ok(registry)
}

/// BAIXA A INTELIGÊNCIA DA SKILL (Resolve o ID ou URL)
pub async fn fetch_skill(input: &str) -> anyhow::Result<SkillPayload> {
    let client = reqwest::Client::new();

    // Lógica inteligente: Resolve a URL
    let target_url = if input.starts_with("http") {
        input.to_string()
    } else {

        let registry = fetch_registry().await?;
        let entry = registry.iter().find(|s| s.id == input)
            .ok_or_else(|| anyhow::anyhow!(
                "Skill '{}' não encontrada! Rode 'rustskill list' para ver as opções disponíveis.",
                input
            ))?;

        entry.url.clone()
    };

    // Download do conteúdo (Payload)
    let response = client.get(&target_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("❌ Erro ao baixar a skill: O link no catálogo parece estar offline ou quebrado.");
    }

    let skill = response.json::<SkillPayload>().await?;
    Ok(skill)
}