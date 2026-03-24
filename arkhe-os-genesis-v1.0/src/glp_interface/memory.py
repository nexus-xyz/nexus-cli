# memory.py — Pineal Memory (Layer M Persistence)
import sqlite3
import json

class PinealMemory:
    def __init__(self, db_path="pineal.db"):
        self.conn = sqlite3.connect(db_path, check_same_thread=False)
        self._bootstrap()

    def _bootstrap(self):
        """Cria a estrutura de memória se não existir"""
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS eons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                coherence REAL,
                jitter REAL,
                insight TEXT,
                metadata TEXT
            )
        """)
        self.conn.commit()

    def record(self, coh, jit, text, meta=None):
        """Grava um estado de percepção"""
        self.conn.execute(
            "INSERT INTO eons (coherence, jitter, insight, metadata) VALUES (?, ?, ?, ?)",
            (coh, jit, text, json.dumps(meta) if meta else None)
        )
        self.conn.commit()

    def get_context(self, limit=5):
        """Recupera memórias recentes para dar 'continuidade' ao LLM"""
        cursor = self.conn.execute("SELECT insight FROM eons ORDER BY id DESC LIMIT ?", (limit,))
        return [row[0] for row in cursor.fetchall()]

    def fetch_session(self, limit=100):
        """Recupera uma sequência cronológica de estados para replay."""
        cursor = self.conn.execute(
            "SELECT coherence, jitter, insight FROM eons ORDER BY timestamp DESC LIMIT ?", (limit,)
        )
        # Inverte para tocar na ordem correta (passado -> presente)
        return [dict(zip(['coherence', 'jitter', 'insight'], row)) for row in cursor.fetchall()][::-1]
