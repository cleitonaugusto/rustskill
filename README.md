# 🦀 RustSkill CLI

> **Transforme seu Cursor em um Desenvolvedor Sênior com um único comando.**

O **RustSkill** é um marketplace de inteligência para o Cursor Editor. Ele injeta regras de especialistas (`.cursorrules`) diretamente no seu projeto, garantindo performance, segurança e arquitetura limpa desde o primeiro commit.

---

## 🚀 Por que usar o RustSkill?

Não perca tempo configurando prompts manualmente para cada projeto. Com o RustSkill, você traz a experiência de centenas de desenvolvedores sêniores para dentro do seu editor em segundos.

### ✨ Funcionalidades

- **Marketplace Global:** Consulta em tempo real ao catálogo oficial de skills.
- **Injeção Atômica:** Instala configurações `.cursorrules` sem quebrar seu workflow.
- **Camada Premium:** Suporte a skills avançadas via Token de Acesso.
- **Ultra Fast:** Desenvolvido em Rust para execução instantânea.

---

## 📦 Instalação

Para instalar o RustSkill CLI localmente, certifique-se de ter o [Rust](https://www.rust-lang.org/) instalado e execute:

```bash
# Clone o repositório
git clone [https://github.com/cleitonaugusto/rustskill.git](https://github.com/cleitonaugusto/rustskill.git)

# Entre na pasta
cd rustskill

# Instale globalmente no seu sistema
cargo install --path .
🛠️ Comandos de MestreComandoDescriçãorustskill listLista todas as skills disponíveis no Marketplace.rustskill add <alias>Injeta a skill no projeto atual (ex: rust/clean-code).rustskill info <alias>Exibe detalhes e as regras de uma skill específica.rustskill login <token>Autentica para liberar acesso às skills 💎 Premium.rustskill upgradeAtualiza o CLI para a versão mais recente.🎯 Exemplo de UsoBash# 1. Veja o que temos hoje no marketplace
rustskill list

# 2. Injete regras de Clean Architecture no seu projeto atual
rustskill add arch/clean-architecture
💎 Acesso PremiumAs skills marcadas com 💎 no marketplace contêm lógicas de arquitetura e segurança de nível Enterprise. Para obter seu token de acesso, entre em contato com o desenvolvedor através do repositório oficial.🤝 ContribuiçõesO ecossistema é alimentado pelo RustSkill Registry. Sinta-se à vontade para sugerir novas regras ou melhorias nos prompts existentes!Desenvolvido com ❤️ por Cleiton Augusto