#!/bin/bash
# Executado na primeira inicialização do nó

set -e

echo "🌀 Bootstrapping Arkhe node..."

# Aguarda serviços subirem
sleep 10

# Testa handover básico
curl -X POST http://localhost:8080/handover \
  -H "Content-Type: application/json" \
  -d '{"to":"genesis","payload":"hello"}'

# Testa fibras evolutivas (GLP)
echo "🧬 Testando Genoma Evolutivo..."
curl -s http://localhost:5000/character/evolution > /dev/null

echo "🧠 Testando Q-Function..."
STATE=$(python3 -c "import random; print([random.random() for _ in range(128)])")
curl -s -X POST http://localhost:5000/dqn/predict -H "Content-Type: application/json" -d "{\"state\": $STATE}" > /dev/null

echo "✅ Bootstrap concluído"
