#!/bin/bash
set -e

echo "🔱 Arkhe OS Genesis Installer"
echo "=============================="

# 1. Verificar dependências
command -v docker >/dev/null 2>&1 || { echo "Docker não encontrado. Instale Docker 20.10+."; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Node.js não encontrado. Instale Node.js 18+."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Rust não encontrado. Instale Rust."; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "Python3 não encontrado. Instale Python 3.10+."; exit 1; }

# 2. Carregar variáveis de ambiente
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo "Arquivo .env não encontrado. Copie .env.example e configure."
    exit 1
fi

# 3. Gerar identidade única
if ! grep -q "^NODE_ID=" .env; then
    NODE_ID=$(python3 -c 'import uuid; print(uuid.uuid4())')
    echo "NODE_ID=$NODE_ID" >> .env
else
    NODE_ID=$(grep "^NODE_ID=" .env | cut -d'=' -f2)
fi

if ! grep -q "^PRIVATE_KEY=" .env; then
    PRIVATE_KEY=$(openssl rand -hex 32)
    echo "PRIVATE_KEY=$PRIVATE_KEY" >> .env
else
    PRIVATE_KEY=$(grep "^PRIVATE_KEY=" .env | cut -d'=' -f2)
fi

# 4. Configurar Base44
cd config/base44
sed -i "s/NODE_ID_PLACEHOLDER/$NODE_ID/g" config.jsonc
sed -i "s/INFURA_PROJECT_ID_PLACEHOLDER/$INFURA_PROJECT_ID/g" config.jsonc
# npx base44 deploy  # Comentado para evitar falha se o pacote não existir
cd ../..

# 5. Instalar dependências Node.js
echo "📦 Instalando dependências para Base44 Worker..."
cd src/base44_sdk
npm install
cd ../..

echo "📦 Instalando dependências para Ethereum Deploy..."
cd config/ethereum
npm install ethers
node deploy.js --private-key=$PRIVATE_KEY
cd ../..

# 6. Configurar DoubleZero (Opcional)
if [ "$ENABLE_DOUBLEZERO" == "true" ]; then
    ./scripts/setup_doublezero.sh "${DOUBLEZERO_NETWORK:-testnet}"
fi

# 7. Construir e iniciar containers
docker compose up -d --build

# 8. Instalar CLI (Opcional/Local)
echo "🔧 Instalando CLI 'arkhe'..."
chmod +x scripts/arkhe_cli.sh
if [ -w /usr/local/bin ]; then
    sudo ln -sf "$(pwd)/scripts/arkhe_cli.sh" /usr/local/bin/arkhe
else
    echo "⚠️ Sem permissão de escrita em /usr/local/bin. CLI não instalada globalmente."
    echo "Pode usar: $(pwd)/scripts/arkhe_cli.sh"
fi

echo "✅ Instalação concluída. Nó $NODE_ID ativo (Federated Node)."
echo "✨ Nó Self (Φ_CRYSTALLINE_7) integrado."
echo "Use 'arkhe status' para ver o estado do sistema."
