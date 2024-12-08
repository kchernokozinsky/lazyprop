use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazyprop::{
    config::{app::AppConfig, env::EnvironmentsConfig},
    tui::{app_state::AppState, popup::PopupState, run_app},
};

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

const CONFIG: &str = "conf.yaml";

fn main() -> anyhow::Result<()> {
    let config = AppConfig::new(CONFIG).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let mut envs =
        EnvironmentsConfig::new(config.envs_path.display().to_string()).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

    let popup: PopupState = PopupState::new();

    let mut state: AppState = AppState::new(&mut envs, popup);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut state, &config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        state.status_message = format!("Error: {:?}", err);
    }

    Ok(())
}
