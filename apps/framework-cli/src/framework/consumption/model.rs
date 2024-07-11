// TODO enable single api change detection for the user

#[derive(Debug, Clone, Default)]
pub struct Consumption {}

impl Consumption {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }
}
