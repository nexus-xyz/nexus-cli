# Arkhe OS – Genesis Package v1.0

Este pacote instala o sistema operacional Arkhe em um novo nó, permitindo que ele se conecte à federação de hipergrafos.

## Requisitos
- Linux (kernel 5.4+) com Docker 20.10+ (incluindo plugin docker compose)
- 4 GB RAM, 10 GB de disco
- Para DoubleZero (opcional): IP público sem NAT, suporte a GRE/BGP
- Conexão com internet (para Ethereum e atualizações)

## Instalação
```bash
git clone https://arkhe.io/genesis arkhe-os
cd arkhe-os
cp .env.example .env
# Edite .env com suas credenciais (INFURA, etc.)
chmod +x install.sh
sudo ./install.sh
```

O script irá:
1. Verificar dependências
2. Gerar chaves criptográficas (identidade do nó)
3. Configurar Base44 (entidades, funções, agentes)
4. Implantar contrato Ethereum (se necessário)
5. Configurar DoubleZero (se habilitado no .env)
6. Iniciar serviços (GLP, swarm, handover listener)

## Pós‑instalação
- Acesse o console: `arkhe console`
- Veja o status: `arkhe status`
- Conecte‑se a outros nós: `arkhe handshake --peer <endereço>`

## Documentação completa
Veja a pasta `docs/`.
