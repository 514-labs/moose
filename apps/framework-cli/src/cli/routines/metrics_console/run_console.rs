use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time;

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

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(f64, f64, Vec<(f64, f64, String)>)>(10);

    tokio::spawn(async move {
        loop {
            let (average, total_requests, summary) = client::client().await.unwrap();
            let _ = tx.send((average, total_requests, summary)).await;
            tokio::time::sleep(time::Duration::from_millis(1000)).await;
        }
    });

    // Start the main loop.
    while app.running {
        tokio::select! {
            received = rx.recv() => {
                if let Some(v) = received {
                    app.req_per_sec(v.1);
                    app.set_metrics(v.0, v.1, v.2);
                };
            }
            // Handle events.
            event = tui.events.next() => { match event?{
                    Event::Tick => app.tick(),
                    Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
                }
            }
        }

        // Render the user interface.
        tui.draw(&mut app)?;
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
