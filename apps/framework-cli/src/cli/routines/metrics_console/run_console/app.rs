use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub average: f64,
    pub requests_per_sec: f64,
    pub total_requests: f64,
    pub summary: Vec<(f64, f64, String)>,
    pub starting_row: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            average: 0.0,
            requests_per_sec: 0.0,
            total_requests: 0.0,
            summary: vec![],
            starting_row: 0,
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

    pub fn set_metrics(
        &mut self,
        average: f64,
        total_requests: f64,
        summary: Vec<(f64, f64, String)>,
    ) {
        self.average = average;
        self.total_requests = total_requests;
        self.summary = summary;
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

    pub fn req_per_sec(&mut self, new_total_requests: f64) {
        self.requests_per_sec = new_total_requests - self.total_requests;
    }
}
