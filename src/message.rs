use crate::error::{connection_exception, message_exception};
use crate::protocol::Command;
#[cfg(not(test))]
use ext_php_rs::prelude::*;
#[cfg(test)]
use ext_php_rs::exception::PhpException;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

/// `RabbitMQ\Message` — represents a single delivered message.
#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\Message"))]
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
            return Err(message_exception("Message already acknowledged"));
        }
        *acked = true;
        Ok(())
    }
}

#[cfg_attr(not(test), php_impl)]
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
            .map_err(|_| connection_exception("Connection closed"))
    }

    pub fn nack(&self, requeue: Option<bool>) -> Result<(), PhpException> {
        self.ensure_not_acked()?;
        self.command_tx
            .send(Command::Nack {
                delivery_tag: self.delivery_tag,
                requeue: requeue.unwrap_or(true),
            })
            .map_err(|_| connection_exception("Connection closed"))
    }

    pub fn reject(&self, requeue: Option<bool>) -> Result<(), PhpException> {
        self.ensure_not_acked()?;
        self.command_tx
            .send(Command::Reject {
                delivery_tag: self.delivery_tag,
                requeue: requeue.unwrap_or(true),
            })
            .map_err(|_| connection_exception("Connection closed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    fn make_message() -> (Message, mpsc::UnboundedReceiver<Command>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut headers = HashMap::new();
        headers.insert("x-type".into(), "test".into());
        let msg = Message::new(
            42,
            "my.routing.key".into(),
            "my-exchange".into(),
            b"hello world".to_vec(),
            headers,
            tx,
        );
        (msg, rx)
    }

    #[test]
    fn get_body_valid_utf8() {
        let (msg, _rx) = make_message();
        assert_eq!(msg.get_body(), "hello world");
    }

    #[test]
    fn get_body_invalid_utf8() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let msg = Message::new(1, String::new(), String::new(), vec![0xFF, 0xFE], HashMap::new(), tx);
        let body = msg.get_body();
        assert!(body.contains('\u{FFFD}'));
    }

    #[test]
    fn get_body_empty() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let msg = Message::new(1, String::new(), String::new(), vec![], HashMap::new(), tx);
        assert_eq!(msg.get_body(), "");
    }

    #[test]
    fn getters_return_correct_values() {
        let (msg, _rx) = make_message();
        assert_eq!(msg.get_routing_key(), "my.routing.key");
        assert_eq!(msg.get_exchange(), "my-exchange");
        assert_eq!(msg.get_delivery_tag(), 42);
        assert_eq!(msg.get_headers().get("x-type").unwrap(), "test");
    }

    #[test]
    fn ensure_not_acked_succeeds_first_call() {
        let (msg, _rx) = make_message();
        assert!(msg.ensure_not_acked().is_ok());
    }

    #[test]
    fn ack_sends_correct_command() {
        let (msg, mut rx) = make_message();
        msg.ack().unwrap();
        match rx.try_recv().unwrap() {
            Command::Ack { delivery_tag } => assert_eq!(delivery_tag, 42),
            other => panic!("expected Ack, got {other:?}"),
        }
    }

    #[test]
    fn nack_defaults_requeue_true() {
        let (msg, mut rx) = make_message();
        msg.nack(None).unwrap();
        match rx.try_recv().unwrap() {
            Command::Nack { delivery_tag, requeue } => {
                assert_eq!(delivery_tag, 42);
                assert!(requeue);
            }
            other => panic!("expected Nack, got {other:?}"),
        }
    }

    #[test]
    fn nack_requeue_false() {
        let (msg, mut rx) = make_message();
        msg.nack(Some(false)).unwrap();
        match rx.try_recv().unwrap() {
            Command::Nack { delivery_tag, requeue } => {
                assert_eq!(delivery_tag, 42);
                assert!(!requeue);
            }
            other => panic!("expected Nack, got {other:?}"),
        }
    }

    #[test]
    fn reject_defaults_requeue_true() {
        let (msg, mut rx) = make_message();
        msg.reject(None).unwrap();
        match rx.try_recv().unwrap() {
            Command::Reject { delivery_tag, requeue } => {
                assert_eq!(delivery_tag, 42);
                assert!(requeue);
            }
            other => panic!("expected Reject, got {other:?}"),
        }
    }

    #[test]
    fn double_ack_returns_error() {
        let (msg, _rx) = make_message();
        msg.ack().unwrap();
        // PhpException construction panics without PHP runtime;
        // the panic proves ensure_not_acked fires on the second call.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| msg.ack()));
        assert!(result.is_err());
    }

    #[test]
    fn ack_on_dropped_channel_returns_error() {
        let (tx, rx) = mpsc::unbounded_channel();
        drop(rx);
        let msg = Message::new(1, String::new(), String::new(), vec![], HashMap::new(), tx);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| msg.ack()));
        assert!(result.is_err());
    }
}
