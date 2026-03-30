use crate::error::to_php_exception;
use crate::message::Message;
use crate::protocol::{Command, WorkerEvent};
use crossbeam_channel::Receiver;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[php_class]
#[php(name = "RabbitMQ\\Consumer")]
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
        let result = if timeout_ms <= 0 {
            // Blocking receive.
            self.delivery_rx
                .recv()
                .map(Some)
                .map_err(|_| to_php_exception("Consumer channel closed"))
        } else {
            match self
                .delivery_rx
                .recv_timeout(Duration::from_millis(timeout_ms as u64))
            {
                Ok(event) => Ok(Some(event)),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => Ok(None),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    Err(to_php_exception("Consumer channel closed"))
                }
            }
        };
        result
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
            WorkerEvent::Error(msg) => Err(to_php_exception(msg)),
        }
    }
}

#[php_impl]
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

    pub fn each(
        &mut self,
        callback: ZendCallable,
        timeout_ms: Option<i64>,
    ) -> Result<(), PhpException> {
        if self.cancelled {
            return Ok(());
        }
        let timeout = timeout_ms.unwrap_or(0);

        loop {
            match self.recv_event(timeout)? {
                Some(event) => match self.event_to_message(event)? {
                    Some(msg) => {
                        let mut zval = msg.into_zval(false).map_err(|e| {
                            to_php_exception(format!("Failed to convert message: {e}"))
                        })?;
                        let result = callback.try_call(vec![&mut zval])?;

                        if let Some(false) = result.bool() {
                            break;
                        }
                    }
                    None => {
                        self.cancelled = true;
                        break;
                    }
                },
                None => break,
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
            .map_err(|_| to_php_exception("Connection closed"))
    }
}
