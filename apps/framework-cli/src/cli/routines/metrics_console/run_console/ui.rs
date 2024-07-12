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
    if app.state == "main" {
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
                    format!("{}", x.path.to_string()),
                    format!(
                        "{}",
                        ((((x.latency_sum / x.request_count) * 1000.0) * 1000.0).round() / 1000.0)
                            .to_string()
                    ),
                    format!(
                        "{}",
                        (((x.request_count * 1000.0).round()) / 1000.0).to_string()
                    ),
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
    } else {
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(2),
                Constraint::Max(2),
                Constraint::Fill(93),
                Constraint::Max(3),
            ])
            .split(frame.size());

        let title = Block::new()
            .title(app.state.clone())
            .title_alignment(Alignment::Center)
            .bold()
            .borders(Borders::TOP)
            .green();

        let average_latency_block = Block::new()
            .title(format!(
                "Average Latency: {}",
                (((app.summary.get(app.starting_row).unwrap().latency_sum
                    / app.summary.get(app.starting_row).unwrap().request_count)
                    * 1000000.0)
                    .round())
                    / 1000.0
            ))
            .title_alignment(Alignment::Center)
            .bold()
            .borders(Borders::NONE)
            .white();

        let request_count_block = Block::new()
            .title(format!(
                "Number of Requests: {}",
                (app.summary.get(app.starting_row).unwrap().request_count)
            ))
            .title_alignment(Alignment::Center)
            .bold()
            .borders(Borders::NONE)
            .white();

        let path_req_per_sec_block = Block::new()
            .title(format!(
                "Requests Per Second: {}",
                (app.path_requests_per_sec)
            ))
            .title_alignment(Alignment::Center)
            .bold()
            .borders(Borders::NONE)
            .white();

        let data_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(outer_layout[1]);

        let info_footer = Paragraph::new(Line::from("(esc) exit detailed view | (q) quit").green())
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(Color::Green)),
            );

        frame.render_widget(average_latency_block, data_layout[0]);
        frame.render_widget(request_count_block, data_layout[1]);
        frame.render_widget(path_req_per_sec_block, data_layout[2]);
        frame.render_widget(title, outer_layout[0]);
        frame.render_widget(info_footer, outer_layout[3]);

        let mut chart_data_vec: Vec<Bar> = vec![];
        let bucket_data = app.path_detailed_data.clone().unwrap();

        for each in bucket_data {
            if each.count > 0.0 {
                chart_data_vec.push(
                    Bar::default()
                        .value(each.count as u64)
                        .label((each.less_than.to_string() + " ms").into()),
                );
            } else {
                chart_data_vec.push(
                    Bar::default()
                        .value(0)
                        .label(each.less_than.to_string().into()),
                );
            }
        }

        let bar_group = BarGroup::default().bars(&chart_data_vec);

        let bar_chart = BarChart::default()
            .block(
                Block::bordered()
                    .title("Number of Requests With Latency Under _ ms")
                    .bold(),
            )
            .data(bar_group)
            .bar_width(1)
            .direction(Direction::Horizontal)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(Style::default().black().on_green().bold());

        frame.render_widget(bar_chart, outer_layout[2]);
    }
}
