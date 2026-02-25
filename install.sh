#!/bin/bash
set -e

# Estética de Vanguarda
GREEN='\033[0;32m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BLUE}${BOLD}🚀 RustSkill Installer${NC}"
echo -e "${BLUE}-----------------------------------${NC}"

# 1. Detectar Versão (Pega a última tag do seu repo)
REPO="cleitonaugusto/rustskill"
VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    VERSION="v0.1.1" # Fallback caso a API do GitHub falhe
fi

# 2. Definir URL do Binário (Baseado no nome que o seu GitHub Action gera)
URL="https://github.com/$REPO/releases/download/$VERSION/rustskill-linux-x86_64"

echo -e "📦 Baixando RustSkill ${GREEN}$VERSION${NC}..."

# 3. Download e Instalação
curl -L -o rustskill_temp $URL
chmod +x rustskill_temp

# Pergunta educada para mover para o PATH
echo -e "🔧 Instalando em /usr/local/bin (pode solicitar sua senha)..."
sudo mv rustskill_temp /usr/local/bin/rustskill

echo -e "\n${GREEN}${BOLD}✅ Instalação Concluída!${NC}"
echo -e "Agora você pode rodar: ${BLUE}rustskill list${NC}\n"