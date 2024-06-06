use crate::framework::aggregations::registry::AggregationProcessRegistry;
use crate::framework::consumption::registry::ConsumptionProcessRegistry;
use crate::framework::flows::registry::FlowProcessRegistry;

pub struct ProcessRegistries {
    pub flows: FlowProcessRegistry,
    pub aggregations: AggregationProcessRegistry,
    pub consumption: ConsumptionProcessRegistry,
}

impl ProcessRegistries {
    pub fn new(
        flows: FlowProcessRegistry,
        aggregations: AggregationProcessRegistry,
        consumption: ConsumptionProcessRegistry,
    ) -> Self {
        Self {
            flows,
            aggregations,
            consumption,
        }
    }
}
