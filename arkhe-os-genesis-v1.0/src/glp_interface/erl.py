# erl.py — Experiential Learning (Layer Ε)
import numpy as np
import torch

class ExperientialLearning:
    """
    Experiential Learning (ERL): O loop de reflexão e refinamento.
    Resolve a estagnação permitindo que o sistema aprenda com seus próprios erros.
    """

    def __init__(self, self_node, memory, glp_model, threshold=0.5):
        self.self_node = self_node
        self.memory = memory
        self.model = glp_model
        self.tau = threshold

    def evaluate(self, y):
        """
        Simula a avaliação do ambiente.
        Em produção: retorno de sensores reais ou validação por pares.
        """
        # Exemplo: recompensa baseada na norma do vetor (coerência)
        reward = np.mean(np.abs(y))
        return "feedback_signal", float(reward)

    def reflect(self, x, y, f, r, memory):
        """
        Reflexão metafórica sobre o porquê do resultado ser baixo.
        """
        context = memory.get_context()
        # Simulação de reflexão: gera um vetor de ajuste (delta)
        delta = np.random.normal(0, 0.1, size=y.shape)
        return delta

    def refine(self, y, delta):
        """
        Aplica o refinamento ao resultado anterior.
        """
        return y + delta

    def distill(self, y_target, x):
        """
        Destila o conhecimento refinado de volta para o modelo.
        """
        # Simulação de Fine-tuning
        print(f"[ERL] Destilando conhecimento para o modelo (Target shape: {y_target.shape})")
        # Em um sistema real: gradient descent step para que f(x) -> y_target
        return True

    def run_episode(self, x_input):
        """
        Executa um episódio completo de ERL.
        """
        # 1. Primeira tentativa (Forward pass)
        # Convert input to tensor if needed
        if not isinstance(x_input, torch.Tensor):
            x_t = torch.tensor(x_input).long()
        else:
            x_t = x_input

        with torch.no_grad():
            out = self.model(x_t)
            # Pegamos os logits ou a representação latente
            y1 = out['sign_logits'].mean(dim=1).numpy() if 'sign_logits' in out else np.random.randn(1, 128)

        # 2. Avaliação
        f1, r1 = self.evaluate(y1)
        print(f"[ERL] Primeira avaliação: r={r1:.4f}")

        y2, r2, delta = None, r1, None
        reward_reflect = 0

        # 3. Reflexão e Refinamento se r < tau
        if r1 < self.tau:
            print(f"[ERL] Recompensa abaixo do limiar ({self.tau}). Iniciando reflexão...")
            delta = self.reflect(x_t.numpy(), y1, f1, r1, self.memory)
            y2 = self.refine(y1, delta)
            f2, r2 = self.evaluate(y2)
            print(f"[ERL] Segunda avaliação após refinamento: r={r2:.4f}")

            if r2 > self.tau:
                self.memory.record(r2, np.var(delta), f"Refined Insight: {f2}", meta={'delta': delta.tolist()})
                reward_reflect = r2
            else:
                reward_reflect = 0
        else:
            print("[ERL] Recompensa satisfatória. Episódio concluído.")
            reward_reflect = 0

        # 4. Destilação se houve melhora
        if y2 is not None and r2 > r1:
            self.distill(y2, x_t)

        return {
            'r1': r1,
            'r2': r2,
            'improved': y2 is not None and r2 > r1,
            'delta_magnitude': float(np.linalg.norm(delta)) if delta is not None else 0.0
        }
