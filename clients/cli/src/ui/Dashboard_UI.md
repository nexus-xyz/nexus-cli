= Dashboard New UI


* High-level plan
* Detailed module breakdown
* 🎨 Color design references
* All Rust skeletons (`main.rs`, `ui.rs`, `state.rs`, `stages.rs`, `metrics.rs`)
* ✅ Fully modular and extensible

---

```md
# Nexus CLI Terminal UI

> A responsive, animated terminal interface for Nexus CLI using Ratatui.

This UI runs continuously, displaying real-time logs, prover stage status, system resource usage, and zkVM task statistics in a clean, color-coded terminal layout.

---

## 🎯 Features (MVP Scope)

- ✅ Live updating CPU and memory gauges (per-process)
- ✅ Animated progress for prover stages:
  - Fetching (known-duration gauge)
  - Proving (bouncing animation)
  - Submitting (short-duration gauge)
- ✅ Log streaming (scrolling optional)
- ✅ zkVM metrics panel: tasks, success rate, runtime, durations
- ✅ Keyboard interaction (e.g. `[Q] Quit`)

---

## 🗂 Project Structure

Put all UI related into `ui/`

---

## 📦 Dependencies

```toml
# Cargo.toml
ratatui = "0.26"
crossterm = "0.27"
sysinfo = "0.30"
````

---

## 🎨 Design Color Coding

| Usage                     | Color                                        |
| ------------------------- | -------------------------------------------- |
| Section titles            | Cyan                                         |
| Fetching status           | Cyan                                         |
| Proving status (bouncing) | Yellow                                       |
| Submitting status         | Green                                        |
| CPU usage gauge           | Green → Yellow → Red (based on % thresholds) |
| zkVM success rate > 75%   | Green                                        |
| Logs                      | White                                        |
| Footer actions (`[Q]`)    | White                                        |

---

## 📂 File-by-File Skeletons

---

### ✅ `main.rs`

TODO: update the import path

```rust
use std::{io, time::{Duration, Instant}};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod ui;
mod stages;
mod metrics;
mod state;

use ui::render_ui;
use stages::ProverStage;
use state::AppState;
use metrics::update_metrics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    let mut tick = 0;

    loop {
        terminal.draw(|f| {
            render_ui(f, &state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Simulate metric updates
        state.metrics = update_metrics(&mut state.sysinfo);

        // Animate prover stages
        state.prover_stage = stages::advance_stage(&state.prover_stage, tick);

        tick += 1;
        std::thread::sleep(Duration::from_millis(1000));
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
```

---

### 🎨 `ui.rs`

TODO: update the import path


```rust
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    text::{Span, Spans},
    Frame,
};

use crate::state::AppState;
use crate::stages::ProverStage;

pub fn render_ui<B: Backend>(f: &mut Frame<B>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(1),
        ])
        .split(f.size());

    render_stage_progress(f, chunks[0], &state.prover_stage);
    render_info_and_logs(f, chunks[1]);
    render_stats(f, chunks[2], state);
    render_footer(f, chunks[3]);
}

fn render_stage_progress<B: Backend>(f: &mut Frame<B>, area: Rect, stage: &ProverStage) {
    // TODO: Render animated progress bar depending on stage type
}

fn render_info_and_logs<B: Backend>(f: &mut Frame<B>, area: Rect) {
    // TODO: Render left Info panel and right Log panel
}

fn render_stats<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
    // TODO: Render CPU/Memory (left) and zkVM panel (right)
}

fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let block = Block::default()
        .title("[Q] Quit")
        .borders(Borders::TOP)
        .style(Style::default().fg(Color::White));
    f.render_widget(block, area);
}
```

---

### 🧠 `state.rs`

TODO: update the import path


```rust
use crate::{stages::ProverStage, metrics::Metrics};
use sysinfo::System;
use std::collections::VecDeque;

pub struct AppState {
    pub tick: usize,
    pub sysinfo: System,
    pub metrics: Metrics,
    pub logs: VecDeque<String>,
    pub prover_stage: ProverStage,

    pub tasks_executed: usize,
    pub tasks_proved: usize,
    pub zkvm_runtime_secs: u64,
    pub last_task_duration: f64,
    pub last_task_status: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            sysinfo: System::new_all(),
            metrics: Metrics::default(),
            logs: VecDeque::with_capacity(100),
            prover_stage: ProverStage::Fetching { elapsed: 0, total: 20 },
            tasks_executed: 0,
            tasks_proved: 0,
            zkvm_runtime_secs: 0,
            last_task_duration: 0.0,
            last_task_status: "Pending".to_string(),
        }
    }
}
```

---

### 🧮 `metrics.rs`

TODO: update the import path


```rust
use sysinfo::{System, SystemExt, ProcessExt};

#[derive(Default, Clone)]
pub struct Metrics {
    pub cpu_percent: f32,
    pub ram_kb: u64,
    pub peak_ram_kb: u64,
}

pub fn update_metrics(sys: &mut System) -> Metrics {
    sys.refresh_processes();
    let pid = std::process::id() as i32;
    if let Some(proc) = sys.process(pid) {
        let current = proc.memory();
        let cpu = proc.cpu_usage();
        let peak = current.max(current); // placeholder
        return Metrics {
            cpu_percent: cpu,
            ram_kb: current,
            peak_ram_kb: peak,
        };
    }

    Metrics::default()
}
```

---

### ⏱️ `stages.rs`

TODO: update the import path

```rust
#[derive(Clone, Debug)]
pub enum ProverStage {
    Idle,
    Fetching { elapsed: u64, total: u64 },
    Proving { tick: usize },
    Submitting { elapsed: u64, total: u64 },
}

pub fn advance_stage(stage: &ProverStage, tick: usize) -> ProverStage {
    match stage {
        ProverStage::Fetching { elapsed, total } => {
            if elapsed + 1 >= *total {
                ProverStage::Proving { tick: 0 }
            } else {
                ProverStage::Fetching { elapsed: elapsed + 1, total: *total }
            }
        }
        ProverStage::Proving { tick } => {
            if tick + 1 > 60 {
                ProverStage::Submitting { elapsed: 0, total: 5 }
            } else {
                ProverStage::Proving { tick: tick + 1 }
            }
        }
        ProverStage::Submitting { elapsed, total } => {
            if elapsed + 1 >= *total {
                ProverStage::Idle
            } else {
                ProverStage::Submitting { elapsed: elapsed + 1, total: *total }
            }
        }
        ProverStage::Idle => ProverStage::Idle,
    }
}
```

---

## ✅ How to Run

```sh
cargo run -r -- start
```

* Press `q` to exit.
* The app cycles stages: Fetching → Proving → Submitting → Idle.

---

Let me know if you'd like me to scaffold tests or CI setup next!


Absolutely! Here's the **fully rendered terminal UI layout** — updated to reflect:

* Final layout structure
* Animated stage at the top
* INFO and LOGS panel side-by-side
* CPU + MEMORY panel side-by-side with zkVM INFO
* Footer with key hint

This matches the structure you've implemented or are about to implement:

---

---

### 🖥️ Original Nexus CLI UI Layout (Before Redesign)

```
+=============================================================================================+
|                                 === NEXUS PROVER v0.10.2 ===                                |
+-----------------------------+---------------------------------------------------------------+
|         INFO                |                           LOGS                                |
|-----------------------------|---------------------------------------------------------------|
| Node ID:        23377484    | [10:54:11] [Fetcher] Rate limited - retrying in 46s           |
| Env:            Production  | [10:54:11] Fetching task step 1 of 3                          |
| Uptime:         0d 0h 0m 5s | [10:54:12] CLI tasks are harder to solve                      |
| Version:        0.10.2      | [10:54:13] Compiling prover input...                          |
|                             | [10:54:14] Witness generation started                         |
|                             | [10:54:15] Memory footprint: 46.7 MB                          |
+=============================================================================================+
```

---

### Key Characteristics of Old UI

* Single horizontal split:

  * Left: Basic node/system info (fixed width)
  * Right: Logs (most of the screen width)
* No CPU/memory graphing
* No zkVM stats
* No animated stage indicators
* Suitable for one-shot or basic runs

---

Would you like to compare this with the current side-by-side version visually? Or generate a diff summary between old vs. new design?


### 🖥️ Nexus CLI Terminal UI Layout (New UI Plain Render)

```
+====================================================================================================+
|                             === NEXUS PROVER v0.10.2 ===                                           |
+====================================================================================================+
| STATUS:  Fetching next task (11s left)                                                             |
|                                                                                                    |
| ███████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░           |
+=======================+============================================================================+
|        INFO           |                                   LOGS                                     |
|-----------------------|----------------------------------------------------------------------------|
| Node ID:        23377484         | [10:54:11] Fetching task step 1 of 3                             |
| Env:            Production       | [10:54:12] CLI tasks are harder than web provers                |
| Uptime:         0d 0h 1m 3s      | [10:54:13] Waiting for prover input...                          |
| Version:        0.10.2           | [10:54:14] Proving...                                           |
+=======================+============================================================================+
|    CPU & MEMORY USAGE                |          zkVM INFO                                            |
|-------------------------------------+---------------------------------------------------------------|
| CPU Usage:       ██████░░░░░░░░░░   | Tasks Executed:      18                                       |
| RAM:             ███████░░░░░░░░   | Tasks Proved:        15 / 18                                  |
| Peak RAM:        █████████░░░░░░   | Success Rate:        83.3%                                    |
|                                     | zkVM Runtime:        12.3s                                    |
|                                     | Last Task:           Success                                  |
|                                     | Last Duration:       0.81s                                    |
+====================================================================================================+
| [Q] Quit                                                                                           |
+====================================================================================================+
```

---

This is the visual foundation your real-time UI will render in the terminal.



Here’s a clear and concise **diff summary** comparing your **original UI** and the new **redesigned animated UI**, broken down by layout sections and feature coverage:

---

## 🔍 Layout Comparison Summary

| Section             | 🧾 Old UI                               | ✨ New UI                                 |
| ------------------- | --------------------------------------- | ---------------------------------------- |
| **Header**          | Title only                              | Title + animated stage status            |
| **Stage Indicator** | ❌ Not shown                             | ✅ Animated (gauge or bouncing block)     |
| **Info Panel**      | ✅ Present (Node ID, Env, Version, etc.) | ✅ Same (left-aligned, clean)             |
| **Logs Panel**      | ✅ Present (right side)                  | ✅ Larger area, supports streaming        |
| **CPU Usage**       | ❌ Absent                                | ✅ Live usage bar per process             |
| **Memory Usage**    | ❌ Absent                                | ✅ Current + peak memory gauges           |
| **zkVM Info**       | ❌ Absent                                | ✅ Full panel: task count, runtime, stats |
| **Layout Style**    | Basic horizontal split                  | Modular, vertical + horizontal nesting   |
| **Colors**          | ❌ Monochrome                            | ✅ Cyan, Yellow, Green (status-aware)     |
| **Footer**          | ❌ Absent                                | ✅ `[Q] Quit` footer panel                |
| **Modularity**      | Flat rendering                          | ✅ Fully modular Ratatui layout           |

---

## 🧠 Design Philosophy Change

| Aspect        | Old UI                               | New UI                                  |
| ------------- | ------------------------------------ | --------------------------------------- |
| Focus         | Static log and config view           | Live system + prover state visibility   |
| Interactivity | None                                 | Live updates + animated feedback        |
| UX Feedback   | Minimal                              | Clear visual feedback during wait times |
| User Context  | Not much insight into what's running | Rich process and zkVM context           |

---

## 🔁 Functional Improvements

* 👁 **User always sees what the prover is doing**
* ⏱ **Time-based tasks are visualized** (e.g. Fetching)
* 🧠 **Resource usage** is visible (RAM, CPU)
* 📊 **Prover performance** and outcome is tracked (success rate, durations)
* ✅ Easier to extend (scrolling, multi-threaded stats later)

---

Would you like a visual “side-by-side” drawing of both layouts next for quick onboarding or presentation use?
