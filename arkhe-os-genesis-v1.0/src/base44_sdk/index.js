let { Base44 } = require('@base44/sdk');
if (typeof Base44 !== 'function') {
    Base44 = class {
        constructor() {
            this.entities = { NodeState: { create: async (d) => console.log('Mock Base44:', d) } };
        }
    };
}
const axios = require('axios');
require('dotenv').config();

const base44 = new Base44({
    projectId: process.env.BASE44_PROJECT_ID || 'default',
    apiKey: process.env.BASE44_API_KEY
});

async function updateNodeState(coherence, satoshi) {
    const coreHost = process.env.CORE_HOST || 'arkhe-core:8080';
    const res = await axios.post(`http://${coreHost}/status`, { coherence, satoshi });
    console.log('Estado atualizado:', res.data);
}

async function main() {
    const coreHost = process.env.CORE_HOST || 'arkhe-core:8080';
    setInterval(async () => {
        try {
            const { data } = await axios.get(`http://${coreHost}/status`);
            await base44.entities.NodeState.create({
                nodeId: data.id,
                coherence: data.coherence,
                satoshi: data.satoshi,
                timestamp: Date.now()
            });
            console.log('Heartbeat enviado para Base44');
        } catch (err) {
            console.error('Erro no heartbeat:', err.message);
        }
    }, 60000); // a cada minuto
}

main();
