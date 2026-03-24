# orchestrator.py — SHM Loop and Orchestration
import time
import threading
import numpy as np
import serial
import serial.tools.list_ports
from pythonosc import udp_client

class BioSensor:
    def __init__(self, port=None):
        self.ser = None
        self.port = port
        self.connect()

    def connect(self):
        if self.port:
            try:
                self.ser = serial.Serial(self.port, 9600, timeout=1)
                print(f"✅ [H] Hardware conectado: {self.port}")
            except:
                print(f"⚠️ [H] Erro ao abrir porta {self.port}. Modo Simulação.")
        else:
            ports = list(serial.tools.list_ports.comports())
            if ports:
                try:
                    self.ser = serial.Serial(ports[0].device, 9600, timeout=1)
                    print(f"✅ [H] Hardware conectado: {ports[0].device}")
                except:
                    print("⚠️ [H] Erro ao abrir porta. Modo Simulação.")
            else:
                print("⚠️ [H] Nenhum Arduino detectado. Modo Simulação.")

    def read(self):
        # Retorna entropia normalizada (0.0 a 1.0)
        if self.ser and self.ser.is_open:
            try:
                line = self.ser.readline().decode('utf-8').strip()
                if line:
                    val = int(line)
                    # Normaliza assumindo leitura 0-1023, ajustando o centro
                    return min(1.0, abs(val - 512) / 512.0)
            except:
                pass
        # Fallback: Ruído térmico simulado
        return np.abs(np.random.normal(0, 0.3))

    def is_alive(self):
        return self.ser is not None and self.ser.is_open

class PinealOrchestrator:
    def __init__(self, interface, memory, sensor, socketio=None):
        self.interface = interface
        self.memory = memory
        self.sensor = sensor
        self.socketio = socketio
        self.osc_client = udp_client.SimpleUDPClient("127.0.0.1", 8000)
        self.active = True

    def _stream_visuals(self, coh, jit):
        """Envia métricas para o motor gráfico via OSC e SocketIO"""
        self.osc_client.send_message("/v1/coherence", float(coh))
        self.osc_client.send_message("/v1/jitter", float(jit))

        if self.socketio:
            self.socketio.emit('pineal_data', {
                'coherence': float(coh),
                'jitter': float(jit)
            })

    def shm_loop(self):
        """O loop cardíaco do sistema (S*H*M)"""
        print("⚡ MERKABAH-7 S*H*M LOOP INICIADO")
        last_insight_time = 0

        while self.active:
            t = time.time()
            # 1. [S] SIMULAÇÃO: Onda Portadora (Theta 4Hz)
            carrier = (np.sin(t * 4 * 2 * np.pi) + 1) / 2

            # 2. [H] HARDWARE: Leitura de Jitter
            jitter = self.sensor.read()

            # Fusão S+H para Coerência
            coherence = (carrier * 0.7) + ((1.0 - jitter) * 0.3)

            # Streaming Visual
            self._stream_visuals(coherence, jitter)

            # 3. [M] METÁFORA: Geração de Insight
            # Trigger por tempo (10s) ou stress (jitter > 0.8)
            if (t - last_insight_time > 10) or (jitter > 0.8 and t - last_insight_time > 2):
                # O insight agora é gerado pela interface/LLM (Metaphor Engine)
                # No app.py, a interface será configurada.
                # Aqui simplificamos chamando um método que será injetado ou definido.
                insight_data = self.interface.transduce({'intensity': jitter})
                insight = insight_data.get('insight', "SILÊNCIO DIGITAL...")

                self.memory.record(coherence, jitter, str(insight))

                if self.socketio:
                    self.socketio.emit('new_insight', {'text': str(insight), 'intensity': float(jitter)})

                print(f"[{coherence:.2f}] {insight}")
                last_insight_time = t

            if self.socketio:
                self.socketio.sleep(0.05)
            else:
                time.sleep(0.05)

    def stop(self):
        self.active = False
