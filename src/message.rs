use crate::error::to_php_exception;
use crate::protocol::Command;
use ext_php_rs::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

/// `RabbitMQ\Message` — represents a single delivered message.
#[php_class]
#[php(name = "RabbitMQ\\Message")]
pub struct Message {
    delivery_tag: u64,
    routing_key: String,
    exchange: String,
    body: Vec<u8>,
    headers: HashMap<String, String>,
    command_tx: UnboundedSender<Command>,
    acked: Mutex<bool>,
}

impl Message {
    pub fn new(
        delivery_tag: u64,
        routing_key: String,
        exchange: String,
        body: Vec<u8>,
        headers: HashMap<String, String>,
        command_tx: UnboundedSender<Command>,
    ) -> Self {
        Self {
            delivery_tag,
            routing_key,
            exchange,
            body,
            headers,
            command_tx,
            acked: Mutex::new(false),
        }
    }

    fn ensure_not_acked(&self) -> Result<(), PhpException> {
        let mut acked = self.acked.lock().unwrap();
        if *acked {
            return Err(to_php_exception("Message already acknowledged"));
        }
        *acked = true;
        Ok(())
    }
}

#[php_impl]
impl Message {
    pub fn get_body(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn get_routing_key(&self) -> &str {
        &self.routing_key
    }

    pub fn get_exchange(&self) -> &str {
        &self.exchange
    }

    pub fn get_delivery_tag(&self) -> u64 {
        self.delivery_tag
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn ack(&self) -> Result<(), PhpException> {
        self.ensure_not_acked()?;
        self.command_tx
            .send(Command::Ack {
                delivery_tag: self.delivery_tag,
            })
            .map_err(|_| to_php_exception("Connection closed"))
    }

    pub fn nack(&self, requeue: Option<bool>) -> Result<(), PhpException> {
        self.ensure_not_acked()?;
        self.command_tx
            .send(Command::Nack {
                delivery_tag: self.delivery_tag,
                requeue: requeue.unwrap_or(true),
            })
            .map_err(|_| to_php_exception("Connection closed"))
    }

    pub fn reject(&self, requeue: Option<bool>) -> Result<(), PhpException> {
        self.ensure_not_acked()?;
        self.command_tx
            .send(Command::Reject {
                delivery_tag: self.delivery_tag,
                requeue: requeue.unwrap_or(true),
            })
            .map_err(|_| to_php_exception("Connection closed"))
    }
}
