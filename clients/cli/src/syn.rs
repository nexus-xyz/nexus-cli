use crate::ui::syn_recruit::SynRecruitState;
use crate::ui::app::{App, Screen, UIConfig};
use crate::environment::Environment;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::{broadcast, mpsc};

pub async fn run_syn_recruit() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    // Create the recruitment video state
    let syn_recruit_state = SynRecruitState::new();
    
    // Create a minimal app structure for the TUI
    let (_event_sender, event_receiver) = mpsc::channel(100);
    let (shutdown_sender, _shutdown_receiver) = broadcast::channel(1);
    let (_, max_tasks_shutdown_receiver) = broadcast::channel(1);
    
    let ui_config = UIConfig::new(
        true,  // with_background_color
        1,     // num_threads
        false, // update_available
        None,  // latest_version
    );
    
    let mut app = App::new(
        None,  // node_id
        Environment::default(),
        event_receiver,
        shutdown_sender,
        max_tasks_shutdown_receiver,
        ui_config,
    );
    
    // Override the screen to show the recruitment video
    app.current_screen = Screen::SynRecruit(Box::new(syn_recruit_state));
    
    // Run the TUI
    let result = crate::ui::run(&mut terminal, app).await;
    
    // Cleanup terminal
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    
    Ok(result?)
}
