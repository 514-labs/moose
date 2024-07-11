use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

mod app;
mod client;
mod event;
mod handler;
mod tui;
mod ui;

use app::App;
use event::Event;
use handler::handle_key_events;

pub async fn run_console() -> app::AppResult<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = event::EventHandler::new(250);
    let mut tui = tui::Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        let (average, total_requests, summary) = client::client().await.unwrap();
        app.set_metrics(average, total_requests, summary);

        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
