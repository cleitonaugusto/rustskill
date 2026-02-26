```markdown
# Changelog

Todas as mudanças notáveis no projeto **RustSkill** serão documentadas neste arquivo.

## [0.2.0] - 2026-02-26

### ✨ Adicionado
- Comando `audit` com motor de busca híbrida.
- Suporte a leitura de manifestos `Cargo.toml` e `package.json`.
- Sistema de "Motivos" (Reasoning) para recomendações de skills.
- Integração de `triggers` dinâmicos vindos do Registry Global.

### 🔧 Alterado
- Refatoração do comando `Add` para suporte a tokens em chamadas de API Premium.
- Melhoria no sistema de cache-busting do `registry.json`.

### 🚀 Corrigido
- Erro de inferência de tipo no parser de JSON para ecossistemas Node.js.

