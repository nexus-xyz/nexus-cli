import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np
from scipy.special import hermite

class HarmonicConfinement(nn.Module):
    """
    Poço harmônico quântico para sequências.
    Estados: |n⟩ com energia E_n = ℏω(n + 1/2)
    No espaço de embedding: polinômios de Hermite × envelope gaussiano
    """
    def __init__(self, max_n=8, sigma=1.0):
        super().__init__()
        self.max_n = max_n  # número de níveis quânticos
        self.sigma = sigma  # largura do poço harmônico

        # Autofunções do oscilador harmônico (pré-computadas)
        # ψ_n(x) = (1/√(2^n n!)) * (mω/πℏ)^{1/4} * H_n(ξ) * exp(-ξ²/2)
        # onde ξ = x / (σ√2)
        self.register_buffer(
            'hermite_basis',
            self._compute_hermite_basis(max_n, 256)  # discretização da posição
        )

    def _compute_hermite_basis(self, max_n, resolution):
        x = torch.linspace(-3, 3, resolution)
        xi = x / (self.sigma * np.sqrt(2))

        basis = []
        for n in range(max_n):
            H_n = torch.tensor(hermite(n)(xi.numpy()), dtype=torch.float32)
            # Use math.factorial for initialization
            import math
            norm = (2**n * math.factorial(n) * np.sqrt(np.pi))**(-0.5)
            psi = norm * H_n * torch.exp(-xi**2 / 2)
            basis.append(psi)

        return torch.stack(basis)  # [max_n, resolution]

    def forward(self, positions, amplitudes):
        """
        positions: índices normalizados na sequência [-1, 1]
        amplitudes: ocupação de cada modo |n⟩
        """
        # Interpolação das autofunções nas posições reais
        idx = ((positions + 1) / 2 * 255).long().clamp(0, 255)
        # amplitudes: [batch, max_n] or [batch, seq_len, max_n]
        # logic depends on if amplitudes are per-sequence or per-token
        # assuming per-sequence for now as per user prompt: occupation_amplitudes based on sequence_embedding.mean(dim=1)

        # basis_sampled: [max_n, batch, seq_len]
        basis_sampled = self.hermite_basis[:, idx]

        # wavefunction calculation
        # amplitudes: [batch, max_n]
        # basis_sampled: [max_n, batch, seq_len]
        # output should be [batch, seq_len]
        wavefunction = torch.einsum('bn,nbs->bs', amplitudes, basis_sampled)
        return wavefunction


class SuperlatticeHamiltonian(nn.Module):
    """
    Múltiplos poços harmônicos acoplados.
    Cada escala = modo coletivo do cristal.
    """
    def __init__(self, embed_dim, scales=[2, 3, 5, 8, 13, 21], coupling_matrix=None):
        """
        Escalas: números de Fibonacci (proporção áurea entre poços)
        """
        super().__init__()
        self.scales = scales
        self.n_wells = len(scales)

        # Hamiltoniano de cada poço isolado
        self.wells = nn.ModuleList([
            HarmonicConfinement(max_n=min(s, 8), sigma=s/5.0)
            for s in scales
        ])

        # Camadas para projetar embedding para ocupação de cada poço
        self.amplitude_projections = nn.ModuleList([
            nn.Linear(embed_dim, min(s, 8))
            for s in scales
        ])

        # Matriz de acoplamento (tunelamento entre poços)
        if coupling_matrix is None:
            coupling = torch.exp(-torch.abs(
                torch.tensor(scales).float().unsqueeze(0) -
                torch.tensor(scales).float().unsqueeze(1)
            ) / 2.0)
            coupling = coupling - torch.diag(torch.diag(coupling))
        else:
            coupling = coupling_matrix

        self.register_buffer('coupling', coupling)

        self.omega = nn.Parameter(
            torch.tensor([1.0/s for s in scales])
        )

    def forward(self, sequence_embedding):
        batch, seq_len, dim = sequence_embedding.shape
        positions = torch.linspace(-1, 1, seq_len).unsqueeze(0).expand(batch, -1).to(sequence_embedding.device)

        # Ocupação de cada modo em cada poço (aprendido)
        well_states = []
        for i, well in enumerate(self.wells):
            # Projeta média do embedding no número de modos do poço
            amp = self.amplitude_projections[i](sequence_embedding.mean(dim=1))
            amp = F.softmax(amp, dim=-1)
            well_state = well(positions, amp) # [batch, seq_len]
            well_states.append(well_state)

        return torch.stack(well_states, dim=1)  # [batch, n_wells, seq_len]


class ResonantTunnelingAttention(nn.Module):
    """
    Tunelamento ressonante como mecanismo de atenção.
    """
    def __init__(self, n_wells, hidden_dim, temperature=0.1):
        super().__init__()
        self.n_wells = n_wells
        self.temperature = temperature

        self.S_matrix = nn.Parameter(
            torch.randn(n_wells, n_wells, hidden_dim) * 0.1
        )

        self.resonance_energy = nn.Parameter(torch.randn(n_wells, hidden_dim))
        self.resonance_width = nn.Parameter(torch.ones(n_wells, hidden_dim) * 0.1)

    def breit_wigner(self, E, E_0, Γ):
        return Γ / ((E - E_0) + 1j * Γ/2.0 + 1e-8)

    def forward(self, well_states, query_energy=None):
        # well_states: [batch, n_wells, seq_len]
        # need to project to hidden_dim or assume well_states already has it?
        # prompt says [batch, n_wells, seq_len, hidden_dim]
        # my SuperlatticeHamiltonian returns [batch, n_wells, seq_len]
        # Let's adjust well_states to have hidden_dim by broadcasting or linear layer

        batch, n_wells, seq_len = well_states.shape
        hidden = self.S_matrix.shape[-1]

        # Expand well_states to include hidden dimension
        # simplified: well_states_h [batch, n_wells, seq_len, hidden]
        well_states_h = well_states.unsqueeze(-1).expand(-1, -1, -1, hidden)

        if query_energy is None:
            query_energy = well_states_h.mean(dim=[1, 2])

        E = query_energy.unsqueeze(1)
        E_0 = self.resonance_energy.unsqueeze(0)
        Γ = self.resonance_width.unsqueeze(0)

        tunneling_amp = torch.abs(self.breit_wigner(E, E_0, Γ))

        S = F.softmax(self.S_matrix / self.temperature, dim=1)
        S = S.unsqueeze(0).expand(batch, -1, -1, -1)

        mixed_states = torch.einsum('bijh,bjsh->bish', S, well_states_h)
        output = mixed_states * tunneling_amp.unsqueeze(2)

        return output, tunneling_amp


class BCD_GLPLinearA(nn.Module):
    """
    GLP completo: B*C*D = Harmônico × Superlattice × Tunelamento
    """
    def __init__(self, vocab_size, embed_dim=64, hidden_dim=128):
        super().__init__()
        self.embedding = nn.Embedding(vocab_size, embed_dim)

        self.hamiltonian = SuperlatticeHamiltonian(
            embed_dim=embed_dim,
            scales=[2, 3, 5, 8, 13, 21]
        )

        self.tunneling = ResonantTunnelingAttention(
            n_wells=6,
            hidden_dim=hidden_dim
        )

        self.sign_predictor = nn.Linear(hidden_dim, vocab_size)
        self.geometry_probe = nn.Linear(hidden_dim, 3)

    def forward(self, sign_ids, return_wavefunction=False):
        # x: [batch, seq_len, embed_dim]
        x = self.embedding(sign_ids)

        # B*C
        well_states = self.hamiltonian(x) # [batch, n_wells, seq_len]

        # D
        tunneled, probs = self.tunneling(well_states) # [batch, n_wells, seq_len, hidden_dim]

        # Collapse
        final_state = tunneled.sum(dim=1) # [batch, seq_len, hidden_dim]

        output = {
            'sign_logits': self.sign_predictor(final_state),
            'geometry': self.geometry_probe(final_state.mean(dim=1)),
            'scale_probabilities': probs,
            'tunneling_strength': probs.std(dim=1).mean()
        }

        if return_wavefunction:
            output['well_states'] = well_states
            output['tunneled_states'] = tunneled

        return output
