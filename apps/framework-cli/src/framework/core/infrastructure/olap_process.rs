use serde::{Deserialize, Serialize};

// This is mostly a place holder to be hydrated when we move to different processes to execute OLAP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlapProcess {}

impl OlapProcess {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }

    pub fn expanded_display(&self) -> String {
        "OLAP Process".to_string()
    }

    pub fn short_display(&self) -> String {
        self.expanded_display()
    }
}
