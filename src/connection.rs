use crate::consumer::Consumer;
use crate::error::to_php_exception;
use crate::protocol::Command;
use crate::worker;
use ext_php_rs::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

/// `RabbitMQ\Connection` — manages the background AMQP connection.
#[php_class]
#[php(name = "RabbitMQ\\Connection")]
pub struct Connection {
    command_tx: Option<UnboundedSender<Command>>,
    worker_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

#[php_impl]
impl Connection {
    /// Connect to RabbitMQ. Blocks until connected or throws on failure.
    pub fn __construct(uri: &str) -> Result<Self, PhpException> {
        let (command_tx, handle) =
            worker::spawn_worker(uri.to_string()).map_err(|e| to_php_exception(e))?;

        Ok(Self {
            command_tx: Some(command_tx),
            worker_handle: Mutex::new(Some(handle)),
        })
    }

    /// Start consuming from a queue. Returns a Consumer object.
    pub fn consume(
        &self,
        queue: &str,
        consumer_tag: Option<&str>,
        prefetch_count: Option<i64>,
    ) -> Result<Consumer, PhpException> {
        let tx = self
            .command_tx
            .as_ref()
            .ok_or_else(|| to_php_exception("Connection closed"))?;

        let (delivery_tx, delivery_rx) = crossbeam_channel::unbounded();

        let tag = consumer_tag.unwrap_or("").to_string();
        let prefetch = prefetch_count.unwrap_or(10) as u16;

        tx.send(Command::Subscribe {
            queue: queue.to_string(),
            consumer_tag: tag.clone(),
            prefetch_count: prefetch,
            delivery_tx,
        })
        .map_err(|_| to_php_exception("Connection closed"))?;

        Ok(Consumer::new(tag, delivery_rx, tx.clone()))
    }

    /// Publish a message and wait for broker confirm.
    pub fn publish(
        &self,
        exchange: &str,
        routing_key: &str,
        body: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<(), PhpException> {
        let tx = self
            .command_tx
            .as_ref()
            .ok_or_else(|| to_php_exception("Connection closed"))?;

        let (confirm_tx, confirm_rx) = std::sync::mpsc::sync_channel(1);

        tx.send(Command::Publish {
            exchange: exchange.to_string(),
            routing_key: routing_key.to_string(),
            body: body.as_bytes().to_vec(),
            headers: headers.unwrap_or_default(),
            confirm_tx: Some(confirm_tx),
        })
        .map_err(|_| to_php_exception("Connection closed"))?;

        confirm_rx
            .recv()
            .map_err(|_| to_php_exception("Connection closed"))?
            .map_err(|e| to_php_exception(e))
    }

    /// Publish a message without waiting for confirm. The message is guaranteed
    /// to be confirmed before the connection is closed/dropped.
    pub fn publish_async(
        &self,
        exchange: &str,
        routing_key: &str,
        body: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<(), PhpException> {
        let tx = self
            .command_tx
            .as_ref()
            .ok_or_else(|| to_php_exception("Connection closed"))?;

        tx.send(Command::Publish {
            exchange: exchange.to_string(),
            routing_key: routing_key.to_string(),
            body: body.as_bytes().to_vec(),
            headers: headers.unwrap_or_default(),
            confirm_tx: None,
        })
        .map_err(|_| to_php_exception("Connection closed"))
    }

    /// Check if the connection is still alive.
    pub fn is_connected(&self) -> bool {
        self.command_tx.as_ref().is_some_and(|tx| !tx.is_closed())
    }

    /// Close the connection gracefully.
    pub fn close(&mut self) {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(Command::Shutdown);
        }
        if let Some(handle) = self.worker_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.close();
    }
}
