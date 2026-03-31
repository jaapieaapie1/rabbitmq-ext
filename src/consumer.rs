use crate::error::{connection_exception, consume_exception};
use crate::message::Message;
use crate::protocol::{Command, WorkerEvent};
use crossbeam_channel::Receiver;
#[cfg(not(test))]
use ext_php_rs::convert::IntoZval;
#[cfg(not(test))]
use ext_php_rs::prelude::*;
#[cfg(test)]
use ext_php_rs::exception::PhpException;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\Consumer"))]
pub struct Consumer {
    consumer_tag: String,
    delivery_rx: Receiver<WorkerEvent>,
    command_tx: UnboundedSender<Command>,
    cancelled: bool,
}

impl Consumer {
    pub fn new(
        consumer_tag: String,
        delivery_rx: Receiver<WorkerEvent>,
        command_tx: UnboundedSender<Command>,
    ) -> Self {
        Self {
            consumer_tag,
            delivery_rx,
            command_tx,
            cancelled: false,
        }
    }

    fn recv_event(&self, timeout_ms: i64) -> Result<Option<WorkerEvent>, PhpException> {
        if timeout_ms <= 0 {
            // Blocking receive.
            self.delivery_rx
                .recv()
                .map(Some)
                .map_err(|_| consume_exception("Consumer channel closed"))
        } else {
            match self
                .delivery_rx
                .recv_timeout(Duration::from_millis(timeout_ms as u64))
            {
                Ok(event) => Ok(Some(event)),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => Ok(None),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    Err(consume_exception("Consumer channel closed"))
                }
            }
        }
    }

    fn event_to_message(&self, event: WorkerEvent) -> Result<Option<Message>, PhpException> {
        match event {
            WorkerEvent::Delivery {
                delivery_tag,
                routing_key,
                exchange,
                body,
                headers,
            } => Ok(Some(Message::new(
                delivery_tag,
                routing_key,
                exchange,
                body,
                headers,
                self.command_tx.clone(),
            ))),
            WorkerEvent::ConsumerCancelled => Ok(None),
            WorkerEvent::Error(msg) => Err(consume_exception(msg)),
        }
    }
}

#[cfg_attr(not(test), php_impl)]
impl Consumer {
    pub fn next(&mut self, timeout_ms: Option<i64>) -> Result<Option<Message>, PhpException> {
        if self.cancelled {
            return Ok(None);
        }
        let timeout = timeout_ms.unwrap_or(0);
        match self.recv_event(timeout)? {
            Some(event) => {
                let msg = self.event_to_message(event)?;
                if msg.is_none() {
                    self.cancelled = true;
                }
                Ok(msg)
            }
            None => Ok(None),
        }
    }

    #[cfg(not(test))]
    pub fn each(
        &mut self,
        callback: ZendCallable,
        timeout_ms: Option<i64>,
    ) -> Result<(), PhpException> {
        if self.cancelled {
            return Ok(());
        }
        let timeout = timeout_ms.unwrap_or(0);

        while let Some(event) = self.recv_event(timeout)? {
            match self.event_to_message(event)? {
                Some(msg) => {
                    let mut zval = msg
                        .into_zval(false)
                        .map_err(|e| consume_exception(format!("Failed to convert message: {e}")))?;
                    let result = callback.try_call(vec![&mut zval])?;

                    if let Some(false) = result.bool() {
                        break;
                    }
                }
                None => {
                    self.cancelled = true;
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), PhpException> {
        if self.cancelled {
            return Ok(());
        }
        self.cancelled = true;
        self.command_tx
            .send(Command::Unsubscribe {
                consumer_tag: self.consumer_tag.clone(),
            })
            .map_err(|_| connection_exception("Connection closed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_consumer() -> (Consumer, crossbeam_channel::Sender<WorkerEvent>, tokio::sync::mpsc::UnboundedReceiver<Command>) {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let consumer = Consumer::new("test-tag".into(), event_rx, cmd_tx);
        (consumer, event_tx, cmd_rx)
    }

    fn delivery_event(tag: u64) -> WorkerEvent {
        WorkerEvent::Delivery {
            delivery_tag: tag,
            routing_key: "rk".into(),
            exchange: "ex".into(),
            body: b"body".to_vec(),
            headers: HashMap::new(),
        }
    }

    #[test]
    fn recv_event_with_timeout_returns_event() {
        let (consumer, tx, _cmd_rx) = make_consumer();
        tx.send(delivery_event(1)).unwrap();
        let result = consumer.recv_event(100).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn recv_event_with_timeout_returns_none_on_timeout() {
        let (consumer, _tx, _cmd_rx) = make_consumer();
        let result = consumer.recv_event(10).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn recv_event_blocking_returns_event() {
        let (consumer, tx, _cmd_rx) = make_consumer();
        tx.send(delivery_event(1)).unwrap();
        let result = consumer.recv_event(0).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn event_to_message_delivery() {
        let (consumer, _tx, _cmd_rx) = make_consumer();
        let msg = consumer.event_to_message(delivery_event(5)).unwrap();
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.get_delivery_tag(), 5);
        assert_eq!(msg.get_routing_key(), "rk");
        assert_eq!(msg.get_exchange(), "ex");
    }

    #[test]
    fn event_to_message_consumer_cancelled() {
        let (consumer, _tx, _cmd_rx) = make_consumer();
        let msg = consumer.event_to_message(WorkerEvent::ConsumerCancelled).unwrap();
        assert!(msg.is_none());
    }

    #[test]
    fn next_returns_none_when_cancelled() {
        let (mut consumer, _tx, _cmd_rx) = make_consumer();
        consumer.cancelled = true;
        let result = consumer.next(Some(100)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn next_sets_cancelled_on_consumer_cancelled() {
        let (mut consumer, tx, _cmd_rx) = make_consumer();
        tx.send(WorkerEvent::ConsumerCancelled).unwrap();
        let result = consumer.next(Some(100)).unwrap();
        assert!(result.is_none());
        assert!(consumer.cancelled);
    }

    #[test]
    fn next_with_timeout_returns_none_on_empty_channel() {
        let (mut consumer, _tx, _cmd_rx) = make_consumer();
        let result = consumer.next(Some(10)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cancel_sends_unsubscribe() {
        let (mut consumer, _tx, mut cmd_rx) = make_consumer();
        consumer.cancel().unwrap();
        match cmd_rx.try_recv().unwrap() {
            Command::Unsubscribe { consumer_tag } => assert_eq!(consumer_tag, "test-tag"),
            other => panic!("expected Unsubscribe, got {other:?}"),
        }
        assert!(consumer.cancelled);
    }

    #[test]
    fn cancel_already_cancelled_is_noop() {
        let (mut consumer, _tx, mut cmd_rx) = make_consumer();
        consumer.cancelled = true;
        consumer.cancel().unwrap();
        assert!(cmd_rx.try_recv().is_err());
    }

    #[test]
    fn recv_event_with_timeout_returns_error_on_disconnect() {
        let (consumer, tx, _cmd_rx) = make_consumer();
        drop(tx);
        // PhpException construction panics without PHP runtime;
        // the panic proves the disconnect error path is reached.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| consumer.recv_event(100)));
        assert!(result.is_err());
    }

    #[test]
    fn event_to_message_error_returns_err() {
        let (consumer, _tx, _cmd_rx) = make_consumer();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            consumer.event_to_message(WorkerEvent::Error("boom".into()))
        }));
        assert!(result.is_err());
    }
}
