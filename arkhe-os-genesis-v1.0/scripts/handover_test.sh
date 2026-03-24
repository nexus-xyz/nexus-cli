#!/bin/bash
# Teste de handover entre dois nós

if [ $# -ne 1 ]; then
    echo "Uso: $0 <target_node_url>"
    exit 1
fi

TARGET=$1
curl -X POST http://localhost:8080/handover \
  -H "Content-Type: application/json" \
  -d "{\"to\":\"$TARGET\",\"payload\":\"teste\"}"
