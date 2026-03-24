# characters.py — Evolutionary Characters (Layer Ω-Phylogeny)
import numpy as np
import time

class CharacterMatrix:
    """
    Representação do genoma informacional do hipergrafo.
    Mapeia traços (caracteres) para cada nó da federação.
    """
    def __init__(self, node_ids):
        self.node_ids = node_ids
        # Caracteres: Coerência, Jitter, Quiralidade, Frequência de Handover
        self.characters = ['coherence', 'jitter', 'chirality', 'frequency']
        self.matrix = np.random.rand(len(node_ids), len(self.characters))

    def get_traits(self, node_id):
        if node_id not in self.node_ids:
            return None
        idx = self.node_ids.index(node_id)
        return dict(zip(self.characters, self.matrix[idx]))

    def update_trait(self, node_id, character, value):
        if node_id in self.node_ids and character in self.characters:
            idx = self.node_ids.index(node_id)
            c_idx = self.characters.index(character)
            self.matrix[idx, c_idx] = value

class SSMDPCharacterModel:
    """
    Stochastic State Model Decision Process para evolução de caracteres.
    Define como os traços degradam ou se adaptam sob pressão.
    """
    def __init__(self, transition_noise=0.01):
        self.noise = transition_noise

    def evolve(self, current_matrix):
        """Aplica dinâmica evolutiva (deriva neutra + ruído)."""
        # Simula evolução temporal
        evolution = np.random.normal(0, self.noise, size=current_matrix.shape)
        return current_matrix + evolution

class CharacterDisplacement:
    """
    Lógica de especialização funcional.
    Nós que interagem frequentemente divergem seus caracteres para reduzir a redundância.
    """
    def __init__(self, alpha=0.1):
        self.alpha = alpha

    def apply(self, matrix, interactions):
        """
        Ajusta a matriz baseada na intensidade das interações.
        interactions: matriz NxN de frequência de handover.
        """
        new_matrix = matrix.copy()
        for i in range(len(matrix)):
            for j in range(i + 1, len(matrix)):
                if interactions[i, j] > 0.5:
                    # Se interagem muito, empurra os caracteres em direções opostas
                    dist = np.linalg.norm(new_matrix[i] - new_matrix[j])
                    if dist < 0.2:
                        diff = (new_matrix[i] - new_matrix[j]) * self.alpha
                        new_matrix[i] += diff
                        new_matrix[j] -= diff
        return new_matrix

class EvolutionaryEngine:
    """Unifica a matriz, o modelo de processo e o deslocamento."""
    def __init__(self, node_ids):
        self.char_matrix = CharacterMatrix(node_ids)
        self.model = SSMDPCharacterModel()
        self.displacement = CharacterDisplacement()
        self.interactions = np.zeros((len(node_ids), len(node_ids)))

    def step(self):
        # 1. Evolução natural
        self.char_matrix.matrix = self.model.evolve(self.char_matrix.matrix)
        # 2. Deslocamento de caracteres baseado em interações
        self.char_matrix.matrix = self.displacement.apply(self.char_matrix.matrix, self.interactions)

    def record_interaction(self, id1, id2):
        if id1 in self.char_matrix.node_ids and id2 in self.char_matrix.node_ids:
            idx1 = self.char_matrix.node_ids.index(id1)
            idx2 = self.char_matrix.node_ids.index(id2)
            self.interactions[idx1, idx2] += 0.1
            self.interactions[idx2, idx1] += 0.1
