use crate::cli::routines::metrics_console::run_console::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') => {
            app.quit();
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        KeyCode::Down => {
            app.down();
        }

        KeyCode::Up => {
            app.up();
        }

        KeyCode::Enter => {
            app.set_state(app.summary[app.starting_row].path.to_string());
        }
        KeyCode::Esc => {
            if app.state == "main" {
                app.quit();
            } else if app.view != "main" {
                app.view = "main".to_string();
            } else {
                app.set_state("main".to_string());
            }
        }

        KeyCode::Char('g') => {
            app.view = "graphs".to_string();
        }
        _ => {}
    }
    Ok(())
}
