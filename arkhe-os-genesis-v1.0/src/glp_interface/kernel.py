# kernel.py — Camada Κ (Kappa): Kernel Methods
import numpy as np

class KernelBridge:
    """
    Conecta as camadas do MERKABAH-7 via teoria de kernels.
    Cada camada define um kernel que mede similaridade entre estados.
    """

    def __init__(self):
        self.kernels = {
            'A_hardware': self._latency_kernel,
            'B_simulation': self._glp_kernel,
            'C_metaphor': self._semantic_kernel,
            'D_hypothesis': self._bayesian_kernel,
            'E_observer': self._consensus_kernel,
            'Φ_crystalline': self._coherence_kernel,
            'Γ_pineal': self._transduction_kernel
        }

    def _latency_kernel(self, node1, node2):
        """Kernel exponencial baseado em latência."""
        # node1, node2 should have a 'latency' attribute or key
        lat1 = getattr(node1, 'latency', node1.get('latency', 0) if isinstance(node1, dict) else 0)
        lat2 = getattr(node2, 'latency', node2.get('latency', 0) if isinstance(node2, dict) else 0)
        lat = abs(lat1 - lat2)
        return np.exp(-lat / 10.0)

    def _glp_kernel(self, state1, state2):
        """Kernel RBF sobre representações GLP."""
        wf1 = state1.get('wavefunction', np.array(state1)) if isinstance(state1, dict) else np.array(state1)
        wf2 = state2.get('wavefunction', np.array(state2)) if isinstance(state2, dict) else np.array(state2)
        coh = state1.get('coherence', 1.0) if isinstance(state1, dict) else 1.0

        # Ensure they are the same length
        min_len = min(len(wf1), len(wf2))
        diff = np.linalg.norm(wf1[:min_len] - wf2[:min_len])
        return np.exp(-diff**2 / (2 * coh**2))

    def _semantic_kernel(self, s1, s2): return 1.0 # Stub
    def _bayesian_kernel(self, h1, h2): return 1.0 # Stub
    def _consensus_kernel(self, c1, c2): return 1.0 # Stub

    def _coherence_kernel(self, phi1, phi2):
        """Kernel de coerência quântica (fidelidade)."""
        wf1 = phi1.get('wavefunction', np.array(phi1)) if isinstance(phi1, dict) else np.array(phi1)
        wf2 = phi2.get('wavefunction', np.array(phi2)) if isinstance(phi2, dict) else np.array(phi2)

        min_len = min(len(wf1), len(wf2))
        overlap = np.abs(np.dot(wf1[:min_len].conj(), wf2[:min_len]))
        return overlap**2

    def _transduction_kernel(self, t1, t2): return 1.0 # Stub

    def combine_kernels(self, weights):
        """
        Combinação convexa de kernels (multi-view learning).
        """
        def combined_kernel(x, y):
            value = 0.0
            for name, kernel in self.kernels.items():
                value += weights.get(name, 0.0) * kernel(x.get(name, {}), y.get(name, {}))
            return value
        return combined_kernel

    def _compute_gram_matrix(self, states, kernel_name):
        N = len(states)
        K = np.zeros((N, N))
        kernel_func = self.kernels.get(kernel_name, lambda x, y: 1.0)
        for i in range(N):
            for j in range(i, N):
                val = kernel_func(states[i], states[j])
                K[i, j] = K[j, i] = val
        return K

    def kernel_pca(self, states, kernel_name='Φ_crystalline'):
        """
        Aplica kernel PCA para extrair componentes principais no espaço de Hilbert.
        """
        K = self._compute_gram_matrix(states, kernel_name)
        N = len(states)
        one_n = np.ones((N, N)) / N
        K_centered = K - one_n @ K - K @ one_n + one_n @ K @ one_n

        eigvals, eigvecs = np.linalg.eigh(K_centered)
        idx = np.argsort(eigvals)[::-1]
        return eigvals[idx], eigvecs[:, idx]
