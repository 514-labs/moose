use std::rc::Rc;

use ratatui::{
    layout::Alignment,
    style::{Color, Style, Stylize},
    Frame,
};
use ratatui::{prelude::*, widgets::*};

use crate::cli::routines::metrics_console::run_console::app::{App, State, TableState};

const INFO_TEXT: &str =
    "(Q) QUIT | (↑) SCROLL UP | (↓) MOVE DOWN | (TAB) SWITCH TABLE | (ENTER) VIEW ENDPOINT DETAILS";

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    match &app.state {
        State::Main() => {
            let outer_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Max(2),
                    Constraint::Max(2),
                    Constraint::Max(2),
                    Constraint::Fill(30),
                    Constraint::Fill(30),
                    Constraint::Fill(30),
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
            let bytes_overview_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .split(outer_layout[2]);

            render_main_page_details(frame, &outer_layout);
            render_overview_metrics(app, frame, &inner_layout);
            render_overview_bytes_data(app, frame, &bytes_overview_layout);
            render_endpoint_table(app, frame, outer_layout[3]);
            render_clickhouse_sync_table(app, frame, outer_layout[4]);
            render_flows_messages_table(app, frame, outer_layout[5]);
        }
        State::PathDetails(state) => {
            let outer_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Max(2),
                    Constraint::Max(2),
                    Constraint::Max(2),
                    Constraint::Fill(91),
                    Constraint::Max(3),
                ])
                .split(frame.size());

            let body_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(40),
                    Constraint::Percentage(50),
                    Constraint::Fill(10),
                ])
                .split(outer_layout[3]);

            let top_row_data_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(outer_layout[1]);

            let bottom_row_data = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(outer_layout[2]);

            render_path_overview_data(&*app, frame, &top_row_data_layout, &bottom_row_data, state);
            render_path_page_details(frame, &outer_layout, state.clone());

            let scale_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(2),
                    Constraint::Percentage(46),
                    Constraint::Percentage(3),
                    Constraint::Percentage(46),
                    Constraint::Percentage(2),
                ])
                .split(body_layout[2]);

            render_sparkline_chart(&*app, frame, &body_layout, &scale_layout, state);
            render_bar_chart(app, frame, &body_layout);
        }
    }
}

fn render_endpoint_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    match &app.table_state {
        TableState::Endpoint => render_active_endpoint_table(app, frame, layout),

        _ => render_passive_endpoint_table(app, frame, layout),
    }
}

fn render_active_endpoint_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    let mut rows: Vec<Row> = vec![];

    for x in &app.summary {
        rows.push(
            if *app.path_requests_per_sec.get(&x.path).unwrap_or(&0.0) > 0.0 {
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
                    format!(
                        "{}",
                        app.kafka_messages_in_total
                            .get(&x.path)
                            .unwrap_or(&("".to_string(), 0.0))
                            .1
                    ),
                ])
                .bold()
                .magenta()
            } else {
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
                    format!(
                        "{}",
                        app.kafka_messages_in_total
                            .get(&x.path)
                            .unwrap_or(&("".to_string(), 0.0))
                            .1
                    ),
                ])
                .green()
                .not_bold()
            },
        )
    }

    let widths = [
        Constraint::Fill(22),
        Constraint::Fill(20),
        Constraint::Fill(20),
        Constraint::Fill(20),
    ];
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.endpoint_starting_row));

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().green())
        .header(
            Row::new(vec![
                "PATH",
                "LATENCY (ms)",
                "NUMBER OF REQUESTS",
                "KAFKA MESSAGES SENT",
            ])
            .style(Style::new().bold())
            .bottom_margin(1)
            .underlined(),
        )
        .block(Block::bordered().title("ENDPOINT METRICS TABLE").bold())
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, layout, &mut table_state)
}

fn render_passive_endpoint_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    let mut rows: Vec<Row> = vec![];

    for x in &app.summary {
        rows.push(
            if *app.path_requests_per_sec.get(&x.path).unwrap_or(&0.0) > 0.0 {
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
                    format!(
                        "{}",
                        app.kafka_messages_in_total
                            .get(&x.path)
                            .unwrap_or(&("".to_string(), 0.0))
                            .1
                    ),
                ])
                .bold()
                .magenta()
            } else {
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
                    format!(
                        "{}",
                        app.kafka_messages_in_total
                            .get(&x.path)
                            .unwrap_or(&("".to_string(), 0.0))
                            .1
                    ),
                ])
                .white()
                .not_bold()
            },
        )
    }

    let widths = [
        Constraint::Fill(22),
        Constraint::Fill(20),
        Constraint::Fill(20),
        Constraint::Fill(20),
    ];
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.endpoint_starting_row));

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().white())
        .header(
            Row::new(vec![
                "PATH",
                "LATENCY (ms)",
                "NUMBER OF REQUESTS",
                "KAFKA MESSAGES SENT",
            ])
            .style(Style::new().bold())
            .bottom_margin(1)
            .underlined(),
        )
        .block(Block::bordered().title("ENDPOINT METRICS TABLE").bold());

    frame.render_widget(table, layout)
}

fn render_clickhouse_sync_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    match app.table_state {
        TableState::Kafka => render_active_clickhouse_sync_table(app, frame, layout),
        _ => render_passive_clickhouse_sync_table(app, frame, layout),
    }
}

fn render_passive_clickhouse_sync_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    let mut rows: Vec<Row> = vec![];

    let mut sorted_messages: Vec<_> = app.kafka_messages_out_total.iter().collect();
    sorted_messages.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    for item in &sorted_messages {
        rows.push(
            Row::new(vec![
                format!("{}", item.0.to_string()),
                format!("{}", item.2),
                format!("{}", {
                    let mut lag: Option<f64> = None;
                    for value in &app.kafka_messages_in_total {
                        if table_equals_path(value.0.clone(), item.0.clone()) {
                            lag = Some(value.1 .1 - item.2);
                            break;
                        }
                    }
                    match lag {
                        Some(value) => value.to_string(),
                        None => "NA".to_string(),
                    }
                }),
                format!(
                    "{}",
                    app.kafka_messages_out_per_sec
                        .get(&item.0)
                        .unwrap_or(&("".to_string(), 0.0))
                        .1
                ),
            ])
            .bold()
            .white(),
        )
    }

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().white())
        .header(
            Row::new(vec!["DATE MODEL", "MESSAGES READ", "LAG", "MESSAGES/SEC"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(
            Block::bordered()
                .title("KAFKA TO TABLE SYNC PROCESS")
                .bold(),
        );

    frame.render_widget(table, layout)
}

fn render_active_clickhouse_sync_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    let mut rows: Vec<Row> = vec![];

    let mut sorted_messages: Vec<_> = app.kafka_messages_out_total.iter().collect();
    sorted_messages.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    for item in &sorted_messages {
        rows.push(
            Row::new(vec![
                format!("{}", item.0.to_string()),
                format!("{}", item.2),
                format!("{}", {
                    let mut lag: Option<f64> = None;
                    for value in &app.kafka_messages_in_total {
                        if table_equals_path(value.0.clone(), item.0.clone()) {
                            lag = Some(value.1 .1 - item.2);
                            break;
                        }
                    }
                    match lag {
                        Some(value) => value.to_string(),
                        None => "NA".to_string(),
                    }
                }),
                format!(
                    "{}",
                    app.kafka_messages_out_per_sec
                        .get(&item.0)
                        .unwrap_or(&("".to_string(), 0.0))
                        .1
                ),
            ])
            .bold()
            .green(),
        )
    }

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.kafka_starting_row));

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().green())
        .header(
            Row::new(vec!["DATA MODEL", "MESSAGES READ", "LAG", "MESSAGES/SEC"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(
            Block::bordered()
                .title("KAFKA TO TABLE SYNC PROCESS")
                .bold(),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, layout, &mut table_state)
}

fn render_flows_messages_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    match app.table_state {
        TableState::Flows => {
            render_active_flows_messages_table(app, frame, layout);
        }
        _ => {
            render_passive_flows_messages_table(app, frame, layout);
        }
    }
}

fn render_active_flows_messages_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    let mut rows: Vec<Row> = vec![];

    let mut sorted_messages: Vec<_> = app.flows_messages_in.iter().collect();
    sorted_messages.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    for item in &sorted_messages {
        rows.push(
            Row::new(vec![
                format!("{}", item.0.to_string()),
                format!("{}", item.1),
                format!("{}", app.flows_messages_out.get(item.0).unwrap_or(&0.0)),
            ])
            .bold()
            .green(),
        )
    }

    let widths = [
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ];
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.flows_table_starting_row));

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().green())
        .header(
            Row::new(vec!["STREAMING PATH", "MESSAGES IN", "MESSAGES OUT"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(
            Block::bordered()
                .title("STREAMING FUNCTIONS KAFKA MESSAGES")
                .bold(),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, layout, &mut table_state)
}

fn render_passive_flows_messages_table(app: &mut App, frame: &mut Frame, layout: Rect) {
    let mut rows: Vec<Row> = vec![];

    let mut sorted_messages: Vec<_> = app.flows_messages_in.iter().collect();
    sorted_messages.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    for item in &sorted_messages {
        rows.push(
            Row::new(vec![
                format!("{}", item.0.to_string()),
                format!("{}", item.1),
                format!("{}", app.flows_messages_out.get(item.0).unwrap_or(&0.0)),
            ])
            .bold()
            .white(),
        )
    }

    let widths = [
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ];
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.kafka_starting_row));

    let table = Table::new(rows, widths)
        .widths(widths)
        .column_spacing(1)
        .style(Style::new().white())
        .header(
            Row::new(vec!["STREAMING PATH", "MESSAGES IN", "MESSAGES OUT"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(
            Block::bordered()
                .title("STREAMING FUNCTIONS KAFKA MESSAGES")
                .bold(),
        );

    frame.render_widget(table, layout)
}

fn table_equals_path(path: String, table: String) -> bool {
    let mut path_vec: Vec<&str> = path.split(['/', '.']).collect();
    path_vec.remove(0);
    let table_vec: Vec<&str> = table.split('_').collect();
    table_vec == path_vec
}

fn render_overview_metrics(app: &mut App, frame: &mut Frame, layout: &Rc<[Rect]>) {
    let average_lat = Block::new()
        .title(format!(
            "AVERAGE LATENCY: {} ms",
            (app.average * 1000.0).round() / 1000.0
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .white()
        .padding(Padding::top(50));
    let total_req = Block::new()
        .title(format!("TOTAL NUMBER OF REQUESTS: {}", app.total_requests))
        .title_alignment(Alignment::Center)
        .bold()
        .white();

    let req_per_sec = Block::new()
        .title(format!("REQUESTS PER SECOND: {}", app.requests_per_sec))
        .title_alignment(Alignment::Center)
        .bold()
        .white();

    frame.render_widget(average_lat, layout[0]);
    frame.render_widget(total_req, layout[1]);
    frame.render_widget(req_per_sec, layout[2]);
}

fn render_overview_bytes_data(app: &mut App, frame: &mut Frame, layout: &Rc<[Rect]>) {
    let bytes_in_per_sec = Block::new()
        .title(format!(
            "DATA IN: {}",
            format_bytes(app.main_bytes_data.bytes_in_per_sec as f64)
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .white();

    let bytes_out_per_sec = Block::new()
        .title(format!(
            "DATA OUT: {}",
            format_bytes(app.main_bytes_data.bytes_out_per_sec as f64)
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .white();

    frame.render_widget(bytes_in_per_sec, layout[0]);
    frame.render_widget(bytes_out_per_sec, layout[2]);
}

fn render_main_page_details(frame: &mut Frame, layout: &Rc<[Rect]>) {
    let info_footer = Paragraph::new(Line::from(INFO_TEXT).green())
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Green)),
        );

    let block = Block::new()
        .title("METRICS CONSOLE")
        .title_alignment(Alignment::Center)
        .bold()
        .borders(Borders::TOP)
        .green();

    frame.render_widget(block, layout[0]);
    frame.render_widget(info_footer, layout[6]);
}

fn render_path_overview_data(
    app: &App,
    frame: &mut Frame,
    top_layout: &Rc<[Rect]>,
    bottom_layout: &Rc<[Rect]>,
    state: &String,
) {
    let average_latency_block = Block::new()
        .title(format!(
            "AVERAGE LATENCY: {}ms ",
            (((app.summary[app.endpoint_starting_row].latency_sum
                / app.summary[app.endpoint_starting_row].request_count)
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
            "NUMBER OF REQUESTS: {}",
            (app.summary[app.endpoint_starting_row].request_count)
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .borders(Borders::NONE)
        .white();

    let path_req_per_sec_block = Block::new()
        .title(format!(
            "REQUESTS PER SECOND: {}",
            (app.path_requests_per_sec.get(state).unwrap_or(&0.0))
        ))
        .title_alignment(Alignment::Center)
        .bold()
        .borders(Borders::NONE)
        .white();

    if state.starts_with("ingest") {
        let bytes_in_per_sec_block = Block::new()
            .title(format!(
                "DATA IN: {}",
                format_bytes(
                    (*app
                        .parsed_bytes_data
                        .path_bytes_in_per_sec_vec
                        .get(state)
                        .unwrap_or(&0)) as f64
                )
            ))
            .title_alignment(Alignment::Center)
            .bold()
            .borders(Borders::NONE)
            .white();
        frame.render_widget(bytes_in_per_sec_block, bottom_layout[1]);
    } else {
        let bytes_in_per_sec_block = Block::new()
            .title(format!(
                "DATA OUT: {}",
                format_bytes(
                    *app.parsed_bytes_data
                        .path_bytes_out_per_sec_vec
                        .get(state)
                        .unwrap_or(&0) as f64
                )
            ))
            .title_alignment(Alignment::Center)
            .bold()
            .borders(Borders::NONE)
            .white();
        frame.render_widget(bytes_in_per_sec_block, bottom_layout[1]);
    }

    frame.render_widget(average_latency_block, top_layout[0]);
    frame.render_widget(request_count_block, top_layout[1]);
    frame.render_widget(path_req_per_sec_block, bottom_layout[0]);
}

fn render_path_page_details(frame: &mut Frame, layout: &Rc<[Rect]>, state: String) {
    let title = Block::new()
        .title(state.to_uppercase())
        .title_alignment(Alignment::Center)
        .bold()
        .borders(Borders::TOP)
        .green();

    let info_footer = Paragraph::new(Line::from("(ESC) EXIT DETAILED VIEW | (Q) QUIT").green())
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Green)),
        );

    frame.render_widget(title, layout[0]);
    frame.render_widget(info_footer, layout[4]);
}

fn render_bar_chart(app: &mut App, frame: &mut Frame, layout: &Rc<[Rect]>) {
    let mut chart_data_vec: Vec<Bar> = vec![];
    let bucket_data = app.path_detailed_data.clone().unwrap();

    for each in bucket_data {
        if each.count > 0.0 {
            chart_data_vec.push(
                Bar::default()
                    .value(each.count as u64)
                    .label((each.less_than.to_string() + "s").into()),
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
                .title("NUMBER OF REQUESTS WITH LATENCY UNDER _ SECONDS")
                .bold(),
        )
        .data(bar_group)
        .bar_width(1)
        .direction(Direction::Horizontal)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(Style::default().black().on_green().bold());

    frame.render_widget(bar_chart, layout[0]);
}
fn render_sparkline_chart(
    app: &App,
    frame: &mut Frame,
    chart_layout: &Rc<[Rect]>,
    scale_layout: &Rc<[Rect]>,
    state: &String,
) {
    let chart: Sparkline;
    let mut top_paragraph: Paragraph = Paragraph::new("<-0");
    let mut middle_paragraph: Paragraph = Paragraph::new("<-0");
    let mut bottom_paragraph: Paragraph = Paragraph::new("<-0");
    if !app.requests_per_sec_vec.contains_key(state) {
        chart = Sparkline::default()
            .block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .title(format!("REQUESTS PER SECOND OVER THE PAST {} SEC", {
                        if (app.viewport.width as f64 * 0.48) > 150.0 {
                            150
                        } else {
                            (app.viewport.width as f64 * 0.48) as usize
                        }
                    })),
            )
            .data(&[0])
            .style(Style::default().fg(Color::Green));
    } else {
        // This unwrap is safe because we know the key exists
        chart = Sparkline::default()
            .block(
                Block::new()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .title(format!("REQUESTS PER SECOND OVER THE PAST {} SEC", {
                        if (app.viewport.width as f64 * 0.48) > 150.0 {
                            150
                        } else {
                            (app.viewport.width as f64 * 0.48) as usize
                        }
                    })),
            )
            .data(match &app.requests_per_sec_vec.get(state) {
                Some(v) => {
                    v.get(
                        ((app // This unwrap is safe because we know the key exists
                            .requests_per_sec_vec
                            .get(state)
                            .unwrap_or(&vec![0; 0])
                            .len() as f64)
                            - (app.viewport.width as f64 * 0.48))
                            as usize..,
                    )
                    .unwrap_or(&v[0..])
                }
                None => &[],
            })
            .style(Style::default().fg(Color::Green));

        top_paragraph = Paragraph::new(
            Line::from(format!(
                "<-{}",
                match &app.requests_per_sec_vec.get(state) {
                    Some(v) => v
                        .get(
                            (app.requests_per_sec_vec
                                .get(state)
                                .unwrap_or(&vec![0; 0])
                                .len() as f64
                                - (app.viewport.width as f64 * 0.48))
                                as usize..
                        )
                        .unwrap_or(&v[0..])
                        .iter()
                        .max()
                        .unwrap_or(&0),
                    None => &0,
                }
            ))
            .green(),
        )
        .left_aligned();

        middle_paragraph = Paragraph::new(
            Line::from(format!(
                "<-{}",
                match &app.requests_per_sec_vec.get(state) {
                    Some(v) =>
                        v.get(
                            (app.requests_per_sec_vec
                                .get(state)
                                .unwrap_or(&vec![0; 0])
                                .len() as f64
                                - (app.viewport.width as f64 * 0.48))
                                as usize..
                        )
                        .unwrap_or(&v[0..])
                        .iter()
                        .max()
                        .unwrap_or(&0)
                            / 2,
                    None => 0,
                }
            ))
            .green(),
        )
        .left_aligned();

        bottom_paragraph = Paragraph::new(Line::from("<-0").green()).left_aligned();
    }

    frame.render_widget(chart, chart_layout[1]);
    frame.render_widget(top_paragraph, scale_layout[0]);
    frame.render_widget(middle_paragraph, scale_layout[2]);
    frame.render_widget(bottom_paragraph, scale_layout[4]);
}

fn format_bytes(bytes: f64) -> String {
    if (bytes / 1000000000.0) > 1.0 {
        format!("{:.3} GB/s", (bytes / 1000000000.0))
    } else if (bytes / 1000000.0) > 1.0 {
        format!("{:.3} MB/s", (bytes / 1000000.0))
    } else if (bytes / 1000.0) > 1.0 {
        format!("{:.3} KB/s", (bytes / 1000.0))
    } else {
        format!("{:3} B/s", bytes)
    }
}
