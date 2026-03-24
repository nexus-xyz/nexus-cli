#!/bin/bash
# Registra este nó na federação (envia identidade para ledger)

NODE_ID=$(cat .env | grep NODE_ID | cut -d '=' -f2)
curl -X POST https://ledger.arkhe.io/register \
  -H "Content-Type: application/json" \
  -d "{\"nodeId\":\"$NODE_ID\", \"coherence\":0.99}"
