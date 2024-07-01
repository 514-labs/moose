use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub enum Data {
    Send(Statistics),
    Count(Statistics),
    Latency(Statistics),
    Receive(tokio::sync::oneshot::Sender<Statistics>),
}

#[derive(Clone)]
pub struct Metrics {
    pub tx: tokio::sync::mpsc::Sender<Data>,
}

pub struct Statistics {
    pub count: String,
    pub latency: String,
}

impl Statistics {
    fn clone(&self) -> Statistics {
        let output = Statistics {
            count: (&self.count).clone(),
            latency: (&self.latency).clone(),
        };
        output
    }
}

impl Metrics {
    pub async fn send_data(&self, data: Data) {
        let data_to_send = match data {
            Data::Send(v) => v,
            _ => Statistics {
                count: "none".to_string(),
                latency: "none".to_string(),
            },
        };
        self.tx.send(Data::Send(data_to_send)).await;
    }

    pub async fn receive_data(&self) -> Statistics {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel::<Statistics>();
        self.tx.send(Data::Receive(resp_tx)).await.unwrap();

        let res = resp_rx.await;

        res.unwrap()
    }

    pub async fn controller(self: Arc<Metrics>, mut rx: tokio::sync::mpsc::Receiver<Data>) {
        let mut data = Statistics {
            count: "TEST".to_string(),
            latency: "TEST".to_string(),
        };
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    Data::Send(v) => data = v,
                    Data::Receive(v) => {
                        let _ = v.send(data.clone());
                    }
                    Data::Count(v) => {
                        data.count = v.count;
                    }
                    Data::Latency(v) => {
                        data.latency = v.latency;
                    }
                };
                println!("Inputted: {}", data.latency);
            }
        });
    }

    pub fn copy(&self) -> Metrics {
        let new_struct = Metrics {
            tx: self.tx.clone(),
        };
        new_struct
    }
}
