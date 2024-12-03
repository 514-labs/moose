use serde::{Deserialize, Serialize};

use crate::framework::blocks::model::Blocks;

// This is mostly a place holder to be hydrated when we move to different processes to execute individual blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlapProcess {}

impl OlapProcess {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }

    pub fn from_blocks(_blocks: &Blocks) -> Self {
        OlapProcess {}
    }

    pub fn expanded_display(&self) -> String {
        "Reloading Blocks".to_string()
    }

    pub fn short_display(&self) -> String {
        self.expanded_display()
    }
}
