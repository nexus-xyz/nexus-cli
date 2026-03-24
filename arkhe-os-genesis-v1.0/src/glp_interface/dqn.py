# dqn.py — Q-Function Approximation (Layer Φ-DQN)
import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
import random
from collections import deque

class QFunctionApproximator(nn.Module):
    """
    Estimador de valor Q generalizado.
    Usa a arquitetura do GLP para prever o valor de um handover (ação)
    dado o estado (coerência, satoshi, etc.).
    """
    def __init__(self, input_dim=128, action_dim=4):
        super(QFunctionApproximator, self).__init__()
        self.fc = nn.Sequential(
            nn.Linear(input_dim, 64),
            nn.ReLU(),
            nn.Linear(64, action_dim)
        )

    def forward(self, x):
        return self.fc(x)

class ExperienceReplay:
    """
    Banco de memórias para quebrar correlações temporais.
    Integrado com o ledger pineal_memory.db.
    """
    def __init__(self, capacity=1000):
        self.memory = deque(maxlen=capacity)

    def push(self, state, action, reward, next_state, done):
        self.memory.append((state, action, reward, next_state, done))

    def sample(self, batch_size):
        return random.sample(self.memory, batch_size)

    def __len__(self):
        return len(self.memory)

class DQNAgent:
    """
    Agente DQN com Target Network para estabilização.
    """
    def __init__(self, state_dim, action_dim, lr=1e-3, gamma=0.99, epsilon=0.1):
        self.policy_net = QFunctionApproximator(state_dim, action_dim)
        self.target_net = QFunctionApproximator(state_dim, action_dim)
        self.target_net.load_state_dict(self.policy_net.state_dict())
        self.target_net.eval()

        self.optimizer = optim.Adam(self.policy_net.parameters(), lr=lr)
        self.memory = ExperienceReplay()

        self.gamma = gamma
        self.epsilon = epsilon
        self.action_dim = action_dim

    def select_action(self, state):
        if random.random() < self.epsilon:
            return random.randint(0, self.action_dim - 1)
        with torch.no_grad():
            state_t = torch.tensor(state).float()
            q_values = self.policy_net(state_t)
            return q_values.argmax().item()

    def update(self, batch_size=32):
        if len(self.memory) < batch_size:
            return

        batch = self.memory.sample(batch_size)
        states, actions, rewards, next_states, dones = zip(*batch)

        states = torch.tensor(np.array(states)).float()
        actions = torch.tensor(actions).long()
        rewards = torch.tensor(rewards).float()
        next_states = torch.tensor(np.array(next_states)).float()
        dones = torch.tensor(dones).float()

        current_q = self.policy_net(states).gather(1, actions.unsqueeze(1))
        next_q = self.target_net(next_states).max(1)[0].detach()
        target_q = rewards + (self.gamma * next_q * (1 - dones))

        loss = nn.MSELoss()(current_q.squeeze(), target_q)
        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()

        return loss.item()

    def sync_target(self):
        """Sincroniza a rede alvo (Target Network)."""
        self.target_net.load_state_dict(self.policy_net.state_dict())
