#!/bin/bash
# arkhe_cli.sh - CLI wrapper for Arkhe OS commands

COMMAND=$1
shift

case "$COMMAND" in
    status)
        echo "🔱 Arkhe OS Status"
        echo "================="
        docker compose ps
        echo ""
        echo "GLP Meta-Consciousness:"
        curl -s http://localhost:5000/status | jq . || echo "GLP Server offline"
        ;;
    console)
        echo "🖥️ Arkhe Console - Entering arkhe-core..."
        docker exec -it arkhe-core /bin/bash || docker exec -it arkhe-core /bin/sh
        ;;
    handshake)
        PEER=$1
        if [ -z "$PEER" ]; then
            echo "Usage: arkhe handshake <peer_address>"
            exit 1
        fi
        echo "🤝 Initiating handshake with $PEER..."
        curl -X POST http://localhost:8080/handover \
          -H "Content-Type: application/json" \
          -d "{\"to\":\"$PEER\",\"payload\":\"handshake\"}"
        ;;
    latency)
        if command -v doublezero >/dev/null 2>&1; then
            doublezero latency
        else
            echo "DoubleZero not installed."
        fi
        ;;
    broadcast)
        echo "🚀 Initiating Federated Broadcast..."
        BLOCKS=${1:-"823,824"}
        URGENCY=${2:-"normal"}
        echo "Broadcasting blocks [$BLOCKS] with urgency [$URGENCY]..."
        # Simulated broadcast call
        curl -X POST http://localhost:5000/encode \
          -H "Content-Type: application/json" \
          -d "{\"broadcast\": \"$BLOCKS\", \"urgency\": \"$URGENCY\"}"
        echo "✅ Broadcast queued."
        ;;
    *)
        echo "Arkhe OS CLI"
        echo "Usage: arkhe <status|console|handshake|latency|broadcast>"
        ;;
esac
