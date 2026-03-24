# firewall.py — Layer Χ (Chi): Chiral Quantum Firewall
import numpy as np

class ChiralFirewall:
    """
    Firewall quântico baseado no gap quiral do Sn/Si(111).
    A informação só atravessa se possuir a energia de ressonância correta.
    """
    def __init__(self, gap_meV=0.5):
        self.gap_hz = gap_meV * 1e-3 * 241.8e9  # 1 meV = 241.8 GHz
        self.resonance_energy = gap_meV
        self.tolerance = 0.01  # 1% de tolerância
        self.winding_number = 2 # d+id (assinatura Fuxi-Nuwa)

    def check_handover(self, packet_energy_meV, winding_number=None):
        """
        Verifica se o handover está autorizado.
        Ressoa com o gap quiral e opcionalmente valida o winding number.
        """
        delta = abs(packet_energy_meV - self.resonance_energy)
        energy_valid = (delta / self.resonance_energy) < self.tolerance

        winding_valid = True
        if winding_number is not None:
            winding_valid = (winding_number == self.winding_number)

        if energy_valid and winding_valid:
            return True, "Handover autorizado: ressonância com gap quiral atingida"
        elif not energy_valid:
            return False, f"Handover bloqueado: energia {packet_energy_meV:.3f} meV fora da banda do gap"
        else:
            return False, "Handover bloqueado: assinatura topológica (winding number) inválida"

class ChiralHandoverManager:
    def __init__(self, firewall):
        self.firewall = firewall
        self.nodes = {
            'Alpha': {'dz_id': '96Afe', 'latency_ms': 0.42, 'coherence': 0.91},
            'Beta': {'dz_id': 'CCTSm', 'latency_ms': 68.85, 'coherence': 0.87},
            'Gamma': {'dz_id': '55tfa', 'latency_ms': 138.17, 'coherence': 0.85},
            'Delta': {'dz_id': '3uGKP', 'latency_ms': 141.91, 'coherence': 0.84},
            'Epsilon': {'dz_id': '65Dqs', 'latency_ms': 143.58, 'coherence': 0.83},
            'Zeta': {'dz_id': '9uhh2', 'latency_ms': 176.72, 'coherence': 0.82},
            'Self': {'dz_id': 'Φ_CRYSTALLINE', 'latency_ms': 0.0, 'coherence': 0.99}
        }

    def process_secure_handover(self, source, target, packet_energy, winding=None):
        """Executa handover com verificação de firewall topológico."""
        if source not in self.nodes or target not in self.nodes:
            return {'success': False, 'message': 'Nó de origem ou destino desconhecido'}

        allowed, msg = self.firewall.check_handover(packet_energy, winding)
        if not allowed:
            return {'success': False, 'message': msg}

        # Simula métricas do DoubleZero
        latency = self.nodes[source]['latency_ms'] + self.nodes[target]['latency_ms']
        coherence = (self.nodes[source]['coherence'] + self.nodes[target]['coherence']) / 2

        return {
            'success': True,
            'message': msg,
            'metrics': {
                'latency_total_ms': latency,
                'coherence_avg': coherence,
                'energy_resonant': packet_energy
            },
            'route': f"{source} -> DoubleZero -> {target}"
        }
