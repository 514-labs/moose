use prometheus_parse::HistogramCount;
use ratatui::layout::Rect;

use super::client::{parsing_histogram_data, ParsedMetricsData, PathMetricsData};
use std::{collections::HashMap, error};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum State {
    Main(),
    PathDetails(String),
}

pub struct App {
    pub viewport: Rect,
    pub state: State,
    pub running: bool,
    pub average: f64,
    pub requests_per_sec: f64,
    pub total_requests: f64,
    pub summary: Vec<PathMetricsData>,
    pub starting_row: usize,
    pub path_detailed_data: Option<Vec<HistogramCount>>,
    pub path_requests_per_sec: HashMap<String, f64>,
    pub requests_per_sec_vec: HashMap<String, Vec<u64>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            viewport: Rect::new(0, 0, 0, 0),
            state: State::Main(),
            running: true,
            average: 0.0,
            requests_per_sec: 0.0,
            total_requests: 0.0,
            summary: vec![],
            starting_row: 0,
            path_detailed_data: vec![].into(),
            path_requests_per_sec: HashMap::new(),
            requests_per_sec_vec: HashMap::new(),
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
        if !self.summary.is_empty() {
            self.path_detailed_data = parsing_histogram_data(
                self.summary[self.starting_row].path.clone(),
                parsed_data.histogram_vec,
            )
        }
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

        // Initializes variables and vec for each path in summary to keep unwraps in ui.rs safe
        for path in &self.summary {
            if !self.path_requests_per_sec.contains_key(&path.path) {
                self.path_requests_per_sec
                    .insert(path.path.clone(), path.request_count);
            }

            if !self.requests_per_sec_vec.contains_key(&path.path) {
                self.requests_per_sec_vec
                    .insert(path.path.clone(), vec![0; 150]);
            }

            for item in path_metrics {
                if item.path == path.path {
                    self.path_requests_per_sec
                        .insert(path.path.clone(), item.request_count - path.request_count);
                }
            }
            self.requests_per_sec_vec
                .get_mut(&path.path)
                .unwrap()
                .push(*self.path_requests_per_sec.get(&path.path).unwrap() as u64);
            while self.requests_per_sec_vec.get(&path.path).unwrap().len() > 150 {
                self.requests_per_sec_vec
                    .get_mut(&path.path)
                    .unwrap()
                    .remove(0);
            }
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}
