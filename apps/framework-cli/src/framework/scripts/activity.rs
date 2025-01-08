use prost_types::Duration as TemporalDuration;
use std::{collections::HashMap, time::Duration};
use temporal_sdk_core::protos::coresdk::common::RetryPolicy;

/**
 * Activity is a orchestrator concept found in temporal.
 *
 * We interact with the temporal "activities" by scehduling them in a workflow.
 *
 *
 */
use temporal_sdk_core::protos::coresdk::{
    common::Payload,
    workflow_commands::{ActivityCancellationType, ScheduleActivity},
};

// Used to pass data from activity to activity. There are limits to the size of the data
//
// For this I think we should only pass meta data about a data frames in arrow format.
pub struct ScriptPayload {
    pub metadata: HashMap<String, Vec<u8>>,
    pub data: Vec<u8>,
}

pub struct ScriptRetryPolicy {
    pub initial_interval: Option<Duration>,
    pub backoff_coefficient: f64,
    pub maximum_interval: Option<Duration>,
    pub maximum_attempts: i32,
    pub non_retryable_error_types: Vec<String>,
}

pub struct Activity {
    pub id: String,
    pub _type: String,
    pub namespace: String,
    pub task_queue_name: String,
    pub header_fields: HashMap<String, Payload>,
    pub arguments: Vec<Payload>,
    pub schedule_to_close_timeout: Option<Duration>,
    pub schedule_to_start_timeout: Option<Duration>,
    pub start_to_close_timeout: Option<Duration>,
    pub heartbeat_timeout: Option<Duration>,
    pub retry_policy: Option<ScriptRetryPolicy>,
    pub cancellation_type: ActivityCancellationType,
}

impl Activity {
    pub fn builder() -> ActivityBuilder {
        ActivityBuilder::default()
    }

    pub fn schedule(&self, payload: ScriptPayload) -> ScheduleActivity {
        ScheduleActivity {
            activity_id: self.id.clone(),
            activity_type: self._type.clone(),
            namespace: self.namespace.clone(),
            task_queue: self.task_queue_name.clone(),
            header_fields: self.header_fields.clone(),
            arguments: vec![Payload {
                data: payload.data,
                metadata: payload.metadata,
            }],
            schedule_to_close_timeout: self.schedule_to_close_timeout.map(|d| TemporalDuration {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            }),
            schedule_to_start_timeout: self.schedule_to_start_timeout.map(|d| TemporalDuration {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            }),
            start_to_close_timeout: self.start_to_close_timeout.map(|d| TemporalDuration {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            }),
            heartbeat_timeout: self.heartbeat_timeout.map(|d| TemporalDuration {
                seconds: d.as_secs() as i64,
                nanos: d.subsec_nanos() as i32,
            }),
            retry_policy: self.retry_policy.as_ref().map(|p| RetryPolicy {
                initial_interval: p.initial_interval.map(|d| TemporalDuration {
                    seconds: d.as_secs() as i64,
                    nanos: d.subsec_nanos() as i32,
                }),
                backoff_coefficient: p.backoff_coefficient,
                maximum_interval: p.maximum_interval.map(|d| TemporalDuration {
                    seconds: d.as_secs() as i64,
                    nanos: d.subsec_nanos() as i32,
                }),
                maximum_attempts: p.maximum_attempts,
                non_retryable_error_types: p.non_retryable_error_types.clone(),
            }),
            cancellation_type: self.cancellation_type as i32,
        }
    }
}

#[derive(Default)]
pub struct ActivityBuilder {
    id: String,
    _type: String,
    namespace: String,
    task_queue_name: String,
    header_fields: HashMap<String, Payload>,
    arguments: Vec<Payload>,
    schedule_to_close_timeout: Option<Duration>,
    schedule_to_start_timeout: Option<Duration>,
    start_to_close_timeout: Option<Duration>,
    heartbeat_timeout: Option<Duration>,
    retry_policy: Option<ScriptRetryPolicy>,
    cancellation_type: ActivityCancellationType,
}

impl ActivityBuilder {
    pub fn new(
        id: impl Into<String>,
        type_: impl Into<String>,
        namespace: impl Into<String>,
        task_queue: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            _type: type_.into(),
            namespace: namespace.into(),
            task_queue_name: task_queue.into(),
            ..Default::default()
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn _type(mut self, _type: impl Into<String>) -> Self {
        self._type = _type.into();
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn task_queue_name(mut self, task_queue_name: impl Into<String>) -> Self {
        self.task_queue_name = task_queue_name.into();
        self
    }

    pub fn schedule_to_close_timeout(mut self, timeout: Duration) -> Self {
        self.schedule_to_close_timeout = Some(timeout);
        self
    }

    pub fn schedule_to_start_timeout(mut self, timeout: Duration) -> Self {
        self.schedule_to_start_timeout = Some(timeout);
        self
    }

    pub fn start_to_close_timeout(mut self, timeout: Duration) -> Self {
        self.start_to_close_timeout = Some(timeout);
        self
    }

    pub fn heartbeat_timeout(mut self, timeout: Duration) -> Self {
        self.heartbeat_timeout = Some(timeout);
        self
    }

    pub fn retry_policy(mut self, policy: ScriptRetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    pub fn build(self) -> Activity {
        Activity {
            id: self.id,
            _type: self._type,
            namespace: self.namespace,
            task_queue_name: self.task_queue_name,
            header_fields: self.header_fields,
            arguments: self.arguments,
            schedule_to_close_timeout: self.schedule_to_close_timeout,
            schedule_to_start_timeout: self.schedule_to_start_timeout,
            start_to_close_timeout: self.start_to_close_timeout,
            heartbeat_timeout: self.heartbeat_timeout,
            retry_policy: self.retry_policy,
            cancellation_type: self.cancellation_type,
        }
    }
}
