use ratatui::{
    layout::Alignment,
    style::{Color, Style, Stylize},
    Frame,
};

use ratatui::{prelude::*, widgets::*};

use crate::cli::routines::metrics_console::run_console::app::App;

const INFO_TEXT: &str = "(q) quit | (↑) move up | (↓) move down";

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let mut summary_text = String::new();

    for path in &app.summary {
        summary_text += format!(
            "Path: {} \n \t - Average Latency: {} \n \t - Number of Requests: {} \n\n",
            path.2, path.0, path.1
        )
        .as_str();
    }

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(2),
            Constraint::Max(2),
            Constraint::Fill(80),
            Constraint::Max(3),
        ])
        .split(frame.size());

    let paragraph_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Max(30)])
        .split(outer_layout[1]);

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(paragraph_layout[0]);

    let mut rows: Vec<Row> = vec![];

    for x in &app.summary {
        rows.push(
            Row::new(vec![
                format!("{}", x.2.to_string()),
                format!(
                    "{}",
                    ((((x.0 / x.1) * 1000.0) * 1000.0).round() / 1000.0).to_string()
                ),
                format!("{}", (((x.1 * 1000.0).round()) / 1000.0).to_string()),
            ])
            .not_bold(),
        )
    }
    let widths = [Constraint::Min(1), Constraint::Min(1), Constraint::Min(1)];
    let mut table_state = TableState::default();
    table_state.select(Some(app.starting_row));

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().green())
        .header(
            Row::new(vec!["Path", "Latency (ms)", "Number of Requests"])
                .style(Style::new().bold())
                .bottom_margin(1)
                .underlined(),
        )
        .block(Block::bordered().title("Endpoint Metrics Table").bold())
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    let info_footer = Paragraph::new(Line::from(INFO_TEXT).green())
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Green)),
        );

    let block = Block::new()
        .title("Metrics Console")
        .title_alignment(Alignment::Center)
        .bold()
        .borders(Borders::TOP)
        .green();
    let average_lat = Block::new()
        .title(format!(
            "Average Latency: \n {}",
            (app.average * 1000.0).round() / 1000.0
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .white()
        .padding(Padding::top(50));
    let total_req = Block::new()
        .title(format!(
            "Total Number of Requests: \n\n {}",
            app.total_requests
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .white();

    let req_per_sec = Block::new()
        .title(format!(
            "Requests Per Second: \n\n {}",
            app.requests_per_sec
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .white();

    frame.render_widget(block, outer_layout[0]);
    frame.render_widget(average_lat, inner_layout[0]);
    frame.render_widget(total_req, inner_layout[1]);
    frame.render_widget(req_per_sec, inner_layout[2]);
    frame.render_widget(info_footer, outer_layout[3]);
    frame.render_stateful_widget(table, outer_layout[2], &mut table_state);
}
