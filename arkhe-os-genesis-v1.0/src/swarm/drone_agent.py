import asyncio
import aiohttp
import json
import sys
import os

class DroneAgent:
    def __init__(self, node_id, core_url):
        self.node_id = node_id
        self.core_url = core_url
        self.coherence = 0.95
        self.satoshi = 10.0

    async def heartbeat(self):
        async with aiohttp.ClientSession() as session:
            data = {
                'nodeId': self.node_id,
                'coherence': self.coherence,
                'satoshi': self.satoshi
            }
            try:
                async with session.post(f'{self.core_url}/handover', json=data) as resp:
                    return await resp.json()
            except Exception as e:
                print(f"Heartbeat error: {e}")
                return None

    async def fly(self):
        print(f"Drone {self.node_id} voando...")
        await asyncio.sleep(1)
        self.coherence -= 0.01
        self.satoshi += 0.1
        await self.heartbeat()

    async def run(self):
        while True:
            await self.fly()
            await asyncio.sleep(5)

if __name__ == '__main__':
    node_id = sys.argv[1] if len(sys.argv) > 1 else os.environ.get('NODE_ID', 'drone-001')
    core_url = sys.argv[2] if len(sys.argv) > 2 else 'http://arkhe-core:8080'
    agent = DroneAgent(node_id, core_url)
    asyncio.run(agent.run())
