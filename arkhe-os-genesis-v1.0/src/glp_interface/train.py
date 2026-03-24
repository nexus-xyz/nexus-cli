import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np

class QuantumActionLoss(nn.Module):
    """
    Perda baseada no funcional de ação.
    """
    def __init__(self, alpha_kinetic=1.0, alpha_potential=1.0, alpha_tunnel=0.5):
        super().__init__()
        self.alpha_kinetic = alpha_kinetic
        self.alpha_potential = alpha_potential
        self.alpha_tunnel = alpha_tunnel

    def forward(self, predictions, targets, model_states):
        # Perda de predição
        potential = F.cross_entropy(
            predictions['sign_logits'].view(-1, predictions['sign_logits'].size(-1)),
            targets.view(-1)
        )

        # Regularização cinética
        states = model_states['tunneled_states']
        kinetic = ((states[:, :, 1:, :] - states[:, :, :-1, :])**2).mean()

        # Termo de tunelamento
        tunnel_energy = -torch.log(predictions['tunneling_strength'] + 1e-8)

        total = (self.alpha_potential * potential +
                self.alpha_kinetic * kinetic +
                self.alpha_tunnel * tunnel_energy)

        return total, {
            'potential': potential.item(),
            'kinetic': kinetic.item(),
            'tunnel': tunnel_energy.item()
        }

def measure_quantum_coherence(model, test_sequences):
    """
    Mede 'coerência quântica' do modelo em sequências de Linear A.
    """
    model.eval()
    with torch.no_grad():
        out1 = model(test_sequences, return_wavefunction=True)

        masked = test_sequences.clone()
        mask = torch.rand_like(masked.float()) > 0.3
        masked[mask.long()] = 0 # assuming 0 is PAD

        out2 = model(masked, return_wavefunction=True)

        wf1 = F.normalize(out1['tunneled_states'].flatten(1), dim=1)
        wf2 = F.normalize(out2['tunneled_states'].flatten(1), dim=1)

        fidelity = (wf1 * wf2).sum(dim=1)**2

    return {
        'fidelity_under_perturbation': fidelity.mean().item(),
        'quantum_regime': 'coherent' if fidelity.mean() > 0.8 else 'decoherent'
    }

def analyze_confinement(cooc_matrix):
    """
    Diagonaliza M* e verifica se espectro é consistente
    com quantum dot vs. poço quadrado infinito vs. oscilador harmônico
    """
    eigenvals = np.linalg.eigvalsh(cooc_matrix)

    # Spacing ratio: s_n = (E_{n+1} - E_n) / (E_n - E_{n-1})
    spacings = np.diff(eigenvals)
    # Evitar divisão por zero
    spacings = np.where(spacings == 0, 1e-9, spacings)
    ratios = spacings[1:] / spacings[:-1]

    mean_ratio = np.mean(ratios[:10]) if len(ratios) >= 10 else np.mean(ratios)

    return {
        'mean_spacing_ratio': mean_ratio,
        'confinement_regime': 'harmonic' if 0.8 < mean_ratio < 1.2 else 'square'
    }

class PrimordialGLP:
    """
    Treinamento primordial: sem frameworks, apenas matemática e suor.
    """
    def __init__(self, input_dim=16, hidden1=32, hidden2=16, output=4):
        # Inicialização Xavier (Glorot) - manual
        self.W1 = np.random.randn(input_dim, hidden1) * np.sqrt(2.0 / input_dim)
        self.b1 = np.zeros(hidden1)
        self.W2 = np.random.randn(hidden1, hidden2) * np.sqrt(2.0 / hidden1)
        self.b2 = np.zeros(hidden2)
        self.W3 = np.random.randn(hidden2, output) * np.sqrt(2.0 / hidden2)
        self.b3 = np.zeros(output)

    def relu(self, x):
        return np.maximum(0, x)

    def softmax(self, x):
        # Estabilização numérica
        x_shifted = x - np.max(x, axis=1, keepdims=True)
        exp_x = np.exp(x_shifted)
        return exp_x / np.sum(exp_x, axis=1, keepdims=True)

    def forward(self, X):
        # Camada 1
        self.z1 = X @ self.W1 + self.b1
        self.a1 = self.relu(self.z1)

        # Camada 2
        self.z2 = self.a1 @ self.W2 + self.b2
        self.a2 = self.relu(self.z2)

        # Camada 3 (saída)
        self.z3 = self.a2 @ self.W3 + self.b3
        self.y_pred = self.softmax(self.z3)

        return self.y_pred

    def loss(self, y_pred, y_true):
        # Cross-entropy manual
        n_samples = y_true.shape[0]
        # X-entropy expects y_true to be one-hot or indices.
        # Assuming y_true is one-hot
        log_probs = -np.log(y_pred[range(n_samples), y_true.argmax(axis=1)] + 1e-15)
        return np.mean(log_probs)

    def backward(self, X, y_true, lr=0.01):
        n_samples = X.shape[0]

        # Gradiente da cross-entropy + softmax
        dy_pred = self.y_pred.copy()
        dy_pred[range(n_samples), y_true.argmax(axis=1)] -= 1
        dy_pred /= n_samples

        # Gradientes da camada 3
        dW3 = self.a2.T @ dy_pred
        db3 = np.sum(dy_pred, axis=0)

        # Gradientes da camada 2
        da2 = dy_pred @ self.W3.T
        dz2 = da2 * (self.z2 > 0)
        dW2 = self.a1.T @ dz2
        db2 = np.sum(dz2, axis=0)

        # Gradientes da camada 1
        da1 = dz2 @ self.W2.T
        dz1 = da1 * (self.z1 > 0)
        dW1 = X.T @ dz1
        db1 = np.sum(dz1, axis=0)

        # Atualização (SGD)
        self.W3 -= lr * dW3
        self.b3 -= lr * db3
        self.W2 -= lr * dW2
        self.b2 -= lr * db2
        self.W1 -= lr * dW1
        self.b1 -= lr * db1

        return np.mean([np.linalg.norm(dW1), np.linalg.norm(dW2), np.linalg.norm(dW3)])
