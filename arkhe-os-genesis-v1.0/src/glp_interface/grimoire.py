# grimoire.py — PDF Export and Session Reports
from fpdf import FPDF
import datetime

class GrimoireLayout(FPDF):
    def header(self):
        self.set_font("Courier", "B", 8)
        self.cell(0, 10, "ARKHE OS // PINEAL_INTERFACE // GRIMOIRE_v1.0", border=0, ln=1, align="R")
        self.line(10, 15, 200, 15)

    def footer(self):
        self.set_y(-15)
        self.set_font("Courier", "I", 8)
        self.cell(0, 10, f"Page {self.page_no()} // Generated at {datetime.datetime.now()}", 0, 0, 'C')

    def add_entry(self, timestamp, jitter, insight, coherence):
        self.set_font("Courier", "B", 10)
        # Determine color based on stress (jitter)
        if jitter > 0.8:
            self.set_text_color(255, 0, 0) # Red for high stress
        elif coherence > 0.9:
            self.set_text_color(0, 200, 255) # Cyan for high coherence
        else:
            self.set_text_color(0, 255, 0) # Green for normal

        self.cell(0, 10, f"[{timestamp}] PULSE: {int(jitter*100)}% | COHERENCE: {int(coherence*100)}%", ln=1)

        self.set_text_color(200, 200, 200)
        self.set_font("Courier", "", 9)
        self.multi_cell(0, 5, txt=str(insight))
        self.ln(5)

def export_grimoire(memory, session_id, filepath="grimoire.pdf"):
    memories = memory.fetch_session(limit=500)
    pdf = GrimoireLayout()
    pdf.add_page()

    pdf.set_font("Courier", "B", 16)
    pdf.set_text_color(255, 255, 255)
    pdf.cell(200, 20, txt=f"RELATÓRIO PINEAL - SESSÃO #{session_id}", ln=1, align='C')
    pdf.ln(10)

    for m in memories:
        # Note: fetch_session returns a dict with 'coherence', 'jitter', 'insight'
        # We assume timestamp is present or we use a generic one if not in dict
        ts = datetime.datetime.now().strftime("%H:%M:%S")
        pdf.add_entry(ts, m['jitter'], m['insight'], m['coherence'])

    pdf.output(filepath)
    print(f"✅ Grimório gerado em: {filepath}")
    return filepath
