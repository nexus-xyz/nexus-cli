# als_simulator.py — modelo hipotético baseado no hipergrafo
import random

class ALS_Node:
    """Um neurônio motor modelado como nó do hipergrafo."""
    def __init__(self, genetic_risk=0.0, environmental_exposure=0.0):
        self.coherence = 1.0           # começa perfeito
        self.sod1_mutation = genetic_risk
        self.c9orf72_loops = 0
        self.oxidative_stress = 0.0
        self.env = environmental_exposure

    def step(self):
        # Estresse oxidativo cresce com exposição ambiental e mutações
        self.oxidative_stress += 0.01 * (self.env + self.sod1_mutation)

        # Expansões C9orf72 ocorrem estocasticamente
        if random.random() < 0.001 * self.sod1_mutation:
            self.c9orf72_loops += 1

        # Coerência cai com estresse e loops
        self.coherence -= (0.005 * self.oxidative_stress + 0.01 * self.c9orf72_loops)

        # Morte quando coerência < 0.2
        if self.coherence < 0.2:
            return False  # nó morreu
        return True

if __name__ == "__main__":
    # Teste rápido
    neuron = ALS_Node(genetic_risk=0.5, environmental_exposure=0.3)
    steps = 0
    while neuron.step() and steps < 1000:
        steps += 1
        if steps % 100 == 0:
            print(f"Step {steps}: Coherence={neuron.coherence:.4f}, Stress={neuron.oxidative_stress:.4f}, Loops={neuron.c9orf72_loops}")

    if neuron.coherence < 0.2:
        print(f"💀 Neurônio motor degradado após {steps} passos.")
    else:
        print(f"✅ Neurônio motor estável após {steps} passos.")
