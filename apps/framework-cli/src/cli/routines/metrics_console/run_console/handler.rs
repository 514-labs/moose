use crate::cli::routines::metrics_console::run_console::app::State;
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
            if !app.summary.is_empty() {
                app.set_state(State::PathDetails(
                    app.summary[app.starting_row].path.to_string(),
                ));
            }
        }
        KeyCode::Esc => match app.state {
            State::Main() => {
                app.quit();
            }
            State::PathDetails(_) => {
                app.set_state(State::Main());
            }
        },
        _ => {}
    }
    Ok(())
}
