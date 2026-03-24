import torch

class LinearAToPaperCoder:
    def __init__(self, glp_model):
        self.glp = glp_model

    def extract_manifold(self, sign_ids, positions):
        """
        Extrai variedade de representação para análise de difeomorfismos.
        """
        self.glp.eval()
        with torch.no_grad():
            outputs = self.glp(sign_ids, positions)

        return {
            'tablet_repr': outputs['tablet_repr'],
            'logits': outputs['logits']
        }
