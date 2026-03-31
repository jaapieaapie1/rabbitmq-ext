use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::sync::mpsc as std_mpsc;

/// Commands sent from the PHP thread to the background worker.
#[derive(Debug)]
pub enum Command {
    Subscribe {
        queue: String,
        consumer_tag: String,
        prefetch_count: u16,
        delivery_tx: Sender<WorkerEvent>,
    },
    Ack {
        delivery_tag: u64,
    },
    Nack {
        delivery_tag: u64,
        requeue: bool,
    },
    Reject {
        delivery_tag: u64,
        requeue: bool,
    },
    Unsubscribe {
        consumer_tag: String,
    },
    Publish {
        exchange: String,
        routing_key: String,
        body: Vec<u8>,
        headers: HashMap<String, String>,
        /// If Some, worker sends the confirm result back. If None, fire-and-forget.
        confirm_tx: Option<std_mpsc::SyncSender<Result<(), String>>>,
    },
    Shutdown,
}

/// Events sent from the background worker to the PHP thread (per-consumer).
pub enum WorkerEvent {
    Delivery {
        delivery_tag: u64,
        routing_key: String,
        exchange: String,
        body: Vec<u8>,
        headers: HashMap<String, String>,
    },
    ConsumerCancelled,
    Error(String),
}
