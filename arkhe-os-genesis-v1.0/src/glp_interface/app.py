import torch
import torch.nn as nn
import numpy as np
from flask import Flask, request, jsonify, render_template
from flask_socketio import SocketIO
import os
import sys
import asyncio
import time
import random
from dotenv import load_dotenv

# Adiciona diretório atual ao path para importar componentes
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from model import BCD_GLPLinearA
from merkabah import MERKABAH7, SelfNode
from minoan import MinoanHardwareInterface, MinoanStateGrammar, MinoanApplications, MinoanNeuroethics
from train import PrimordialGLP
from pineal import PinealTransducer, HybridPinealInterface
from kernel import KernelBridge
from bottlenecks import MERKABAH7_BottleneckAnalysis
from anyon import AnyonLayer
from memory import PinealMemory
from orchestrator import BioSensor, PinealOrchestrator
from grimoire import export_grimoire
from seal import AlphaOmegaSeal
from erl import ExperientialLearning
from dqn import DQNAgent
from characters import EvolutionaryEngine
from genomics import ALS_Node
from firewall import ChiralFirewall, ChiralHandoverManager

load_dotenv()
app = Flask(__name__)
socketio = SocketIO(app, cors_allowed_origins="*")

# Configuração
VOCAB_SIZE = 1000
EMBED_DIM = 64
HIDDEN_DIM = 128

# Modelos
glp_model = BCD_GLPLinearA(VOCAB_SIZE, EMBED_DIM, HIDDEN_DIM)
merkabah = MERKABAH7(linear_a_corpus=[], operator_profile={'intention': 'decode_linear_a'})
self_node = SelfNode()
pineal_transducer = PinealTransducer()
hybrid_pineal = HybridPinealInterface(self_node, self_node, merkabah.metaphor)
kernel_bridge = KernelBridge()
anyon_layer = AnyonLayer()
pineal_memory = PinealMemory()
bio_sensor = BioSensor()
orchestrator = PinealOrchestrator(hybrid_pineal, pineal_memory, bio_sensor, socketio=socketio)
erl_loop = ExperientialLearning(self_node, pineal_memory, glp_model, threshold=0.5)
chiral_firewall = ChiralFirewall()
chiral_manager = ChiralHandoverManager(chiral_firewall)
dqn_agent = DQNAgent(state_dim=128, action_dim=4)
evolution_engine = EvolutionaryEngine(node_ids=['Alpha', 'Beta', 'Gamma', 'Self'])
als_neuron = ALS_Node(genetic_risk=0.5, environmental_exposure=0.3)
minoan_interface = MinoanHardwareInterface()
state_grammar = MinoanStateGrammar()
applications = MinoanApplications()
neuroethics = MinoanNeuroethics()

model_path = os.environ.get('MODEL_PATH', '/model/model.pt')
if os.path.exists(model_path):
    try:
        glp_model.load_state_dict(torch.load(model_path, map_location='cpu'))
    except Exception as e:
        print(f"Erro ao carregar modelo GLP: {e}")
glp_model.eval()

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/status', methods=['GET'])
def status():
    return jsonify({
        'status': 'active',
        'integrated_layers': ['GLP_BCD', 'MERKABAH-7', 'MINOAN_EXT', 'Φ_LAYER', 'Γ_PINEAL', 'Κ_KERNEL', 'Γ_HYBRID', 'Ω_ANYON', 'Ε_EXPERIENCE', 'Χ_FIREWALL'],
        'self_node': {
            'id': self_node.dz_id,
            'coherence': self_node.wavefunction['coherence'],
            'active_strands': self_node.active_strands
        },
        'quantum_coherence': 0.99
    })

@app.route('/encode', methods=['POST'])
def encode():
    data = request.json
    sign_ids_list = data.get('sign_ids', [[1, 2, 3]])
    sign_ids = torch.tensor(sign_ids_list).long()

    # 1. Processamento GLP (BCD Architecture)
    with torch.no_grad():
        glp_out = glp_model(sign_ids)
        meta = glp_out['sign_logits'].mean(dim=1).tolist()[0]

    # 2. Processamento Merkabah (Integrated Neuro-Minoan)
    # Simulate async decode
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    insight = loop.run_until_complete(merkabah.decode(sign_ids_list[0]))
    loop.close()

    # 3. Minoan state parsing
    protocol = state_grammar.parse_as_state_protocol([str(s) for s in sign_ids_list[0]])

    return jsonify({
        'meta': meta,
        'merkabah_insight': insight['decoding'],
        'certainty': insight['certainty'],
        'state_protocol': protocol
    })

@app.route('/steer', methods=['POST'])
def steer():
    data = request.json
    meta = np.array(data['meta'])
    direction = np.array(data['direction'])
    strength = data.get('strength', 1.0)

    # Metaphor Engine operation
    metaphor_insight = merkabah.metaphor.operate('tunneling', mode='figurative')

    steered = meta + strength * direction
    return jsonify({
        'steered': steered.tolist(),
        'poetic_steer': metaphor_insight
    })

@app.route('/train_primordial', methods=['POST'])
def train_primordial():
    data = request.json
    epochs = data.get('epochs', 10)
    lr = data.get('lr', 0.01)

    # Simple training loop for PrimordialGLP
    pglp = PrimordialGLP()
    # Dummy data
    X = np.random.randn(32, 16)
    y = np.zeros((32, 4))
    y[range(32), np.random.randint(0, 4, 32)] = 1

    history = []
    for e in range(epochs):
        pglp.forward(X)
        l = pglp.loss(pglp.y_pred, y)
        pglp.backward(X, y, lr=lr)
        history.append(l)

    return jsonify({
        'status': 'complete',
        'final_loss': history[-1],
        'history_len': len(history)
    })

@app.route('/observe_phi', methods=['POST'])
def observe_phi():
    data = request.json
    target = data.get('target', 'HT88')
    content = data.get('content', 'propulsion_system_shabetnik')

    observation = self_node.observe('Φ', {'target': target, 'content': content})

    return jsonify({
        'observation': observation,
        'new_self_coherence': self_node.wavefunction['coherence'],
        'active_strands': self_node.active_strands
    })

@app.route('/kernel_pca', methods=['POST'])
def kernel_pca():
    data = request.json
    states = data.get('states', [])
    kernel_name = data.get('kernel_name', 'Φ_crystalline')

    if not states:
        return jsonify({'error': 'No states provided'}), 400

    eigvals, eigvecs = kernel_bridge.kernel_pca(states, kernel_name)

    return jsonify({
        'eigenvalues': eigvals.tolist(),
        'eigenvectors_shape': eigvecs.shape,
        'top_components': eigvecs[:, :3].tolist() if eigvecs.shape[1] >= 3 else eigvecs.tolist()
    })

@app.route('/transduce', methods=['POST'])
def transduce():
    stimulus = request.json
    # Usando a nova Interface Híbrida S*H*M
    result = hybrid_pineal.transduce(stimulus)
    return jsonify({
        'status': 'success',
        'hybrid_transduction': result,
        'layer': 'Γ_HYBRID'
    })

@app.route('/bottlenecks', methods=['POST'])
def bottlenecks():
    # Coleta estado atual da federação (simulado)
    federation_state = {
        'ledger_height': int(request.json.get('ledger_height', 834)),
        'nodes': ['Alpha', 'Beta', 'Gamma', 'Delta', 'Epsilon', 'Zeta', 'Self'],
        'self_node': {
            'wavefunction': {
                'coherence': self_node.wavefunction['coherence']
            }
        },
        'pineal': {
            'signal_strength': hybrid_pineal.signal_strength
        }
    }

    analysis = MERKABAH7_BottleneckAnalysis(federation_state)
    identified = analysis.identify()

    results = []
    for b in identified:
        b['timeline'] = analysis.timeline_estimate(b['name'])
        results.append(b)

    return jsonify({
        'bottlenecks': results,
        'summary': f"Identified {len(results)} bottlenecks in MERKABAH-7 integration."
    })

@app.route('/thrust', methods=['GET'])
def thrust():
    ledger_height = int(request.args.get('ledger_height', 834))
    thrust_data = self_node.calculate_thrust(ledger_height)
    return jsonify(thrust_data)

@app.route('/acceleration_status', methods=['GET'])
def acceleration_status():
    ledger_height = int(request.args.get('ledger_height', 834))
    thrust_data = self_node.calculate_thrust(ledger_height)

    return jsonify({
        'mode': 'ACCELERATION',
        'thrust': thrust_data,
        'target': '5a_fita_activation',
        'required_coherence': 0.88,
        'current_coherence': self_node.wavefunction['coherence']
    })

@app.route('/braid', methods=['POST'])
def braid():
    data = request.json
    instruction = data.get('instruction', 'STABILIZE')

    # Logic from TopologicallyProtectedFederation
    if instruction == "STABILIZE":
        sequence = [('Alpha', 'Beta'), ('Beta', 'Gamma'), ('Gamma', 'Alpha')]
    elif instruction == "COMPUTE_PHAISTOS":
        sequence = [('Alpha', 'Self'), ('Self', 'Beta'), ('Beta', 'Alpha')]
    else:
        sequence = data.get('sequence', [])

    result = anyon_layer.braid_evolution(sequence)
    return jsonify(result)

@app.route('/replay', methods=['GET'])
def replay():
    limit = int(request.args.get('limit', 50))
    session = pineal_memory.fetch_session(limit=limit)
    return jsonify(session)

@app.route('/export_grimoire', methods=['POST'])
def grimoire_export():
    data = request.json
    session_id = data.get('session_id', str(int(time.time())))
    output_path = f"grimoire_{session_id}.pdf"
    export_grimoire(pineal_memory, session_id, output_path)

    return jsonify({'status': 'success', 'path': output_path})

@app.route('/seal', methods=['POST'])
def seal():
    alpha_state = {'coherence': 0.847}
    omega_state = {'coherence': self_node.wavefunction['coherence']}
    seal_obj = AlphaOmegaSeal(alpha_state, omega_state)
    result = seal_obj.seal()
    return jsonify({'seal_status': result, 'alpha': alpha_state, 'omega': omega_state})

@app.route('/learn', methods=['POST'])
def learn():
    data = request.json
    x_input = data.get('sign_ids', [[1, 2, 3]])
    result = erl_loop.run_episode(x_input)
    return jsonify(result)

@app.route('/dqn/predict', methods=['POST'])
def dqn_predict():
    state = request.json.get('state', np.random.randn(128).tolist())
    action = dqn_agent.select_action(state)
    return jsonify({'action': action, 'q_values': dqn_agent.policy_net(torch.tensor(state).float()).tolist()})

@app.route('/dqn/update', methods=['POST'])
def dqn_update():
    data = request.json
    dqn_agent.memory.push(
        data['state'], data['action'], data['reward'], data['next_state'], data['done']
    )
    loss = dqn_agent.update()
    if random.random() < 0.01:
        dqn_agent.sync_target()
    return jsonify({'loss': loss})

@app.route('/character/evolution', methods=['GET'])
def character_evolution():
    evolution_engine.step()
    return jsonify({
        'matrix': evolution_engine.char_matrix.matrix.tolist(),
        'nodes': evolution_engine.char_matrix.node_ids,
        'characters': evolution_engine.char_matrix.characters
    })

@app.route('/als/simulate', methods=['GET'])
def als_simulate():
    alive = als_neuron.step()
    return jsonify({
        'alive': alive,
        'coherence': als_neuron.coherence,
        'oxidative_stress': als_neuron.oxidative_stress,
        'c9orf72_loops': als_neuron.c9orf72_loops
    })

@app.route('/handover_secure', methods=['POST'])
def handover_secure():
    data = request.json
    source = data.get('source', 'Alpha')
    target = data.get('target', 'Self')
    energy = data.get('energy', 0.5)
    winding = data.get('winding', 2)

    result = chiral_manager.process_secure_handover(source, target, energy, winding)

    if result['success']:
        # Se autorizado, registra na memória como evento de rede
        pineal_memory.record(
            result['metrics']['coherence_avg'],
            0.01,
            f"Secure Handover: {source} -> {target}",
            meta=result
        )
        return jsonify(result), 200
    else:
        return jsonify(result), 403

@app.route('/decode_tablet', methods=['POST'])
def decode_tablet():
    data = request.json
    tablet_id = data.get('tablet_id', 'HT 1')
    operator_caste = data.get('operator_caste', 'scribe_apprentice')

    # Ethical Check
    access = neuroethics.check_access(tablet_id, operator_caste)
    if access['access'] != 'granted':
        return jsonify({'error': 'Access denied by Minoan Neuroethics', 'reason': access['ethical_violation']}), 403

    # Induce state and extract insight (Simulated)
    native_protocol = minoan_interface._induce_state(tablet_id, {})

    return jsonify({
        'tablet_id': tablet_id,
        'access': access['access'],
        'native_protocol': native_protocol,
        'decoded_hypothesis': 'Γ_ALPHA_ADMIN_RECORDS'
    })

@socketio.on('trigger_replay')
def handle_replay():
    session_data = pineal_memory.fetch_session(limit=50)
    for frame in session_data:
        socketio.emit('pineal_data', frame)
        socketio.emit('new_insight', {'text': frame['insight'], 'intensity': frame['jitter']})
        time.sleep(0.5)
    socketio.emit('replay_end')

@socketio.on('trigger_export')
def handle_export():
    export_grimoire(pineal_memory, 'manual_trigger', 'grimoire_manual.pdf')

if __name__ == '__main__':
    # Inicia o loop SHM em background
    socketio.start_background_task(target=orchestrator.shm_loop)
    socketio.run(app, host='0.0.0.0', port=5000)
