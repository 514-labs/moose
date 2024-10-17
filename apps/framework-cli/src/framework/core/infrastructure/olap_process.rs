use serde::{Deserialize, Serialize};

use crate::framework::aggregations::model::Aggregation;

// This is mostly a place holder to be hydrated when we move to different processes to execute individual aggregations/
// blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlapProcess {}

impl OlapProcess {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }

    pub fn from_aggregation(_aggregation: &Aggregation) -> Self {
        OlapProcess {}
    }

    pub fn expanded_display(&self) -> String {
        "Reloading Aggregations".to_string()
    }

    pub fn short_display(&self) -> String {
        self.expanded_display()
    }
}
