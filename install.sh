#!/bin/bash
set -e

# Cores para o terminal
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Instalando RustSkill (Cleiton Augusto Edition)...${NC}"

# 1. Detectar SO e Arquitetura
OS_TYPE="$(uname -s)"
ARCH_TYPE="$(uname -m)"
BINARY_NAME=""

if [ "$OS_TYPE" = "Linux" ]; then
    BINARY_NAME="rustskill-linux-x86_64"
elif [ "$OS_TYPE" = "Darwin" ]; then
    BINARY_NAME="rustskill-macos-arm64"
else
    echo -e "${RED}❌ SO não suportado automaticamente pelo script. Baixe o .exe no site.${NC}"
    exit 1
fi

# 2. Pegar a versão mais recente via GitHub API
REPO="cleitonaugusto/rustskill"
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then LATEST_TAG="v0.1.2"; fi

# 3. Download
URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$BINARY_NAME"
echo -e "📥 Baixando $BINARY_NAME de $LATEST_TAG..."
curl -L -o rustskill_temp "$URL"
chmod +x rustskill_temp

# 4. Mover para o PATH
echo -e "🔧 Movendo para /usr/local/bin..."
sudo mv rustskill_temp /usr/local/bin/rustskill

echo -e "${GREEN}✅ RustSkill instalado com sucesso! Digite 'rustskill list' para começar.${NC}"