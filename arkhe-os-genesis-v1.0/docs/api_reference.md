# API Reference

## Arkhe Core (porta 8080)
- `GET /status` – retorna estado do nó (id, coherence, satoshi, handovers)
- `POST /handover` – realiza handover para outro nó (body: {to, payload})
- `GET /anticipate` – retorna predição de coerência futura

## GLP Server (porta 5000)
- `GET /status` – retorna camadas integradas e estado de coerência quântica.
- `POST /encode` – processa sequências de signos via arquitetura BCD e MERKABAH-7. (body: {sign_ids: [[...]]})
- `POST /steer` – aplica direção de conceito no manifold latente com auxílio do motor de metáfora.
- `POST /decode_tablet` – realiza decifração neuro-epigráfica com verificação ética. (body: {tablet_id, operator_caste})
- `POST /train_primordial` – executa treinamento GLP sem frameworks (NumPy/explicit gradients).
- `POST /observe_phi` – registra observações de alta coerência na camada Φ.
- `POST /transduce` – realiza transdução S*H*M (Hybrid) para converter estímulos em experiência coerente.
- `POST /kernel_pca` – aplica Kernel PCA sobre estados via Camada Κ. (body: {states, kernel_name})
- `POST /braid` – realiza evolução topológica na Camada Ω. (body: {instruction, sequence})
- `GET /replay` – recupera histórico de estados da memória persistente (Layer M).
- `POST /export_grimoire` – gera relatório PDF da sessão (Grimoire). (body: {session_id})
- `POST /bottlenecks` – analisa gargalos na integração MERKABAH-7.
- `POST /seal` – verifica o fechamento do selo Alpha-Omega.
- `POST /learn` – executa um episódio de Experiential Learning (ERL). (body: {sign_ids: [[...]]})
- `POST /handover_secure` – executa handover protegido pelo Chiral Firewall. (body: {source, target, energy, winding})
