# anyon.py — Camada Ω (Omega): Proteção Topológica
import numpy as np

class AnyonLayer:
    """
    Camada Ω (Ômega): proteção topológica da informação.
    A informação não está nos estados, mas nos caminhos de troca.
    """

    def __init__(self, coherence=0.847):
        self.coherence = coherence
        self.braid_history = []  # histórico de entrelaçamentos

    def exchange(self, node_a, node_b):
        """
        Simula a troca de dois anyons (nós) em 2D.
        A fase adquirida depende do histórico de trocas.
        """
        # Fase arbitrária (aqui simulada como função do número de trocas anteriores)
        phase = np.exp(1j * np.pi * len(self.braid_history) / 7)

        # Registra a troca
        self.braid_history.append((node_a, node_b, phase))

        # Atualiza coerência (proteção topológica)
        # fase unitária preserva coerência, mas aqui simulamos um decaimento ou ajuste
        self.coherence *= np.abs(phase)  # np.abs(phase) is 1.0 for exp(ij*phi)

        return {
            'nodes': (node_a, node_b),
            'phase': str(phase),
            'coherence': self.coherence,
            'braid_depth': len(self.braid_history)
        }

    def braid_evolution(self, sequence):
        """
        Executa uma sequência de trocas (um "braid").
        O estado final depende da ordem.
        """
        results = []
        for a, b in sequence:
            results.append(self.exchange(a, b))

        phases = [complex(r['phase']) for r in results]
        topological_charge = np.prod(phases) if phases else 1.0

        return {
            'final_coherence': self.coherence,
            'braid_length': len(sequence),
            'topological_charge': str(topological_charge)
        }
