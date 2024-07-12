use prometheus_parse::HistogramCount;

use super::client::{parsing_histogram_data, ParsedMetricsData, PathMetricsData};
use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    pub state: String,
    pub running: bool,
    pub average: f64,
    pub requests_per_sec: f64,
    pub total_requests: f64,
    pub summary: Vec<PathMetricsData>,
    pub starting_row: usize,
    pub path_detailed_data: Option<Vec<HistogramCount>>,
    pub path_requests_per_sec: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: "main".to_string(),
            running: true,
            average: 0.0,
            requests_per_sec: 0.0,
            total_requests: 0.0,
            summary: vec![],
            starting_row: 0,
            path_detailed_data: vec![].into(),
            path_requests_per_sec: 0.0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_metrics(&mut self, parsed_data: ParsedMetricsData) {
        self.average = parsed_data.average_latency;
        self.total_requests = parsed_data.total_requests;
        self.summary = parsed_data.paths_data_vec;
        self.path_detailed_data = parsing_histogram_data(
            self.summary[self.starting_row].path.clone(),
            parsed_data.histogram_vec,
        )
    }

    pub fn down(&mut self) {
        if (self.starting_row + 1) < self.summary.len() {
            self.starting_row += 1;
        }
    }
    pub fn up(&mut self) {
        if self.starting_row > 0 {
            self.starting_row -= 1;
        }
    }

    pub fn req_per_sec(&mut self, new_total_requests: f64, path_metrics: &Vec<PathMetricsData>) {
        self.requests_per_sec = new_total_requests - self.total_requests;
        for value in path_metrics {
            if value.path == self.state {
                self.path_requests_per_sec =
                    value.request_count - self.summary[self.starting_row].request_count;
            }
        }
    }

    pub fn set_state(&mut self, state: String) {
        self.state = state;
    }
}
