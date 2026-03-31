use crate::protocol::{Command, WorkerEvent};
use crossbeam_channel::Sender;
use futures_lite::StreamExt;
use lapin::options::*;
use lapin::types::{AMQPValue, FieldTable, LongString, ShortString};
use lapin::{BasicProperties, Channel, Confirmation, Connection, Consumer, PublisherConfirm};
use std::collections::HashMap;
use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc as tokio_mpsc;

/// Spawn the background worker thread. Returns:
/// - A `tokio_mpsc::UnboundedSender<Command>` to send commands to the worker
/// - A `std::thread::JoinHandle` for the worker thread
///
/// Blocks until the AMQP connection is established or fails.
pub fn spawn_worker(
    uri: String,
) -> Result<
    (
        tokio_mpsc::UnboundedSender<Command>,
        std::thread::JoinHandle<()>,
    ),
    String,
> {
    let (command_tx, command_rx) = tokio_mpsc::unbounded_channel::<Command>();
    let (startup_tx, startup_rx) = std_mpsc::sync_channel::<Result<(), String>>(1);

    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");

        rt.block_on(worker_loop(uri, command_rx, startup_tx));
    });

    match startup_rx.recv() {
        Ok(Ok(())) => Ok((command_tx, handle)),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Worker thread died before reporting startup status".into()),
    }
}

async fn worker_loop(
    uri: String,
    mut command_rx: tokio_mpsc::UnboundedReceiver<Command>,
    startup_tx: std_mpsc::SyncSender<Result<(), String>>,
) {
    let connection = match Connection::connect(&uri, lapin::ConnectionProperties::default()).await {
        Ok(conn) => conn,
        Err(e) => {
            let _ = startup_tx.send(Err(format!("Failed to connect: {e}")));
            return;
        }
    };

    let channel = match connection.create_channel().await {
        Ok(ch) => ch,
        Err(e) => {
            let _ = startup_tx.send(Err(format!("Failed to open channel: {e}")));
            return;
        }
    };

    // Separate channel for publishing with confirm mode.
    let publish_channel = match connection.create_channel().await {
        Ok(ch) => ch,
        Err(e) => {
            let _ = startup_tx.send(Err(format!("Failed to open publish channel: {e}")));
            return;
        }
    };

    if let Err(e) = publish_channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
    {
        let _ = startup_tx.send(Err(format!("Failed to enable confirms: {e}")));
        return;
    }

    let _ = startup_tx.send(Ok(()));

    let mut consumers: HashMap<String, (Consumer, Sender<WorkerEvent>)> = HashMap::new();
    let mut pending_confirms: Vec<PublisherConfirm> = Vec::new();

    loop {
        tokio::select! {
            cmd = command_rx.recv() => {
                match cmd {
                    Some(Command::Subscribe { queue, consumer_tag, prefetch_count, delivery_tx }) => {
                        handle_subscribe(&channel, &mut consumers, queue, consumer_tag, prefetch_count, delivery_tx).await;
                    }
                    Some(Command::Ack { delivery_tag }) => {
                        let _ = channel.basic_ack(delivery_tag, BasicAckOptions::default()).await;
                    }
                    Some(Command::Nack { delivery_tag, requeue }) => {
                        let _ = channel.basic_nack(delivery_tag, BasicNackOptions { multiple: false, requeue }).await;
                    }
                    Some(Command::Reject { delivery_tag, requeue }) => {
                        let _ = channel.basic_reject(delivery_tag, BasicRejectOptions { requeue }).await;
                    }
                    Some(Command::Unsubscribe { consumer_tag }) => {
                        if consumers.remove(&consumer_tag).is_some() {
                            let _ = channel.basic_cancel(consumer_tag.as_str().into(), BasicCancelOptions::default()).await;
                        }
                    }
                    Some(Command::Publish { exchange, routing_key, body, headers, confirm_tx }) => {
                        handle_publish(&publish_channel, &mut pending_confirms, exchange, routing_key, body, headers, confirm_tx).await;
                    }
                    Some(Command::Shutdown) | None => {
                        break;
                    }
                }
            }
            delivery = next_delivery(&mut consumers) => {
                if let Some((tag, event)) = delivery {
                    match event {
                        DeliveryResult::Event(worker_event) => {
                            if let Some((_, tx)) = consumers.get(&tag) {
                                let _ = tx.send(worker_event);
                            }
                        }
                        DeliveryResult::StreamEnded => {
                            if let Some((_, tx)) = consumers.remove(&tag) {
                                let _ = tx.send(WorkerEvent::ConsumerCancelled);
                            }
                        }
                    }
                }
            }
        }
    }

    // Wait for all pending (async) publisher confirms before closing.
    for confirm in pending_confirms {
        let _ = confirm.await;
    }

    for (_, (_, tx)) in consumers.drain() {
        let _ = tx.send(WorkerEvent::Error("Connection closed".into()));
    }

    let _ = publish_channel.close(200, "Normal shutdown".into()).await;
    let _ = channel.close(200, "Normal shutdown".into()).await;
    let _ = connection.close(200, "Normal shutdown".into()).await;
}

async fn handle_subscribe(
    channel: &Channel,
    consumers: &mut HashMap<String, (Consumer, Sender<WorkerEvent>)>,
    queue: String,
    consumer_tag: String,
    prefetch_count: u16,
    delivery_tx: Sender<WorkerEvent>,
) {
    if let Err(e) = channel
        .basic_qos(prefetch_count, BasicQosOptions::default())
        .await
    {
        let _ = delivery_tx.send(WorkerEvent::Error(format!("Failed to set QoS: {e}")));
        return;
    }

    let tag = if consumer_tag.is_empty() {
        ""
    } else {
        &consumer_tag
    };

    match channel
        .basic_consume(
            queue.as_str().into(),
            tag.into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(consumer) => {
            let actual_tag = consumer.tag().to_string();
            consumers.insert(actual_tag, (consumer, delivery_tx));
        }
        Err(e) => {
            let _ = delivery_tx.send(WorkerEvent::Error(format!("Failed to consume: {e}")));
        }
    }
}

async fn handle_publish(
    channel: &Channel,
    pending_confirms: &mut Vec<PublisherConfirm>,
    exchange: String,
    routing_key: String,
    body: Vec<u8>,
    headers: HashMap<String, String>,
    confirm_tx: Option<std_mpsc::SyncSender<Result<(), String>>>,
) {
    let properties = if headers.is_empty() {
        BasicProperties::default()
    } else {
        let mut table = FieldTable::default();
        for (k, v) in &headers {
            table.insert(
                ShortString::from(k.as_str()),
                AMQPValue::LongString(LongString::from(v.as_bytes())),
            );
        }
        BasicProperties::default().with_headers(table)
    };

    match channel
        .basic_publish(
            exchange.as_str().into(),
            routing_key.as_str().into(),
            BasicPublishOptions::default(),
            &body,
            properties,
        )
        .await
    {
        Ok(confirm) => {
            if let Some(tx) = confirm_tx {
                // Blocking publish: await the confirm and send result back.
                match confirm.await {
                    Ok(Confirmation::Ack(_)) => {
                        let _ = tx.send(Ok(()));
                    }
                    Ok(Confirmation::Nack(_)) => {
                        let _ = tx.send(Err("Publish nacked by broker".into()));
                    }
                    Ok(_) => {
                        let _ = tx.send(Err("Unexpected confirm response".into()));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("Confirm error: {e}")));
                    }
                }
            } else {
                // Async publish: track the confirm for shutdown drain.
                pending_confirms.push(confirm);
            }
        }
        Err(e) => {
            if let Some(tx) = confirm_tx {
                let _ = tx.send(Err(format!("Publish failed: {e}")));
            }
        }
    }
}

enum DeliveryResult {
    Event(WorkerEvent),
    StreamEnded,
}

/// Poll all active consumers for the next delivery. If there are no consumers,
/// pends forever (the select! will wake on command_rx instead).
async fn next_delivery(
    consumers: &mut HashMap<String, (Consumer, Sender<WorkerEvent>)>,
) -> Option<(String, DeliveryResult)> {
    if consumers.is_empty() {
        std::future::pending::<()>().await;
        return None;
    }

    loop {
        for (tag, (consumer, _)) in consumers.iter_mut() {
            let mut next_fut = std::pin::pin!(consumer.next());
            match futures_lite::future::poll_once(&mut next_fut).await {
                Some(Some(Ok(delivery))) => {
                    let headers = extract_headers(delivery.properties.headers().as_ref());
                    let event = WorkerEvent::Delivery {
                        delivery_tag: delivery.delivery_tag,
                        routing_key: delivery.routing_key.to_string(),
                        exchange: delivery.exchange.to_string(),
                        body: delivery.data.clone(),
                        headers,
                    };
                    return Some((tag.clone(), DeliveryResult::Event(event)));
                }
                Some(Some(Err(_e))) => {
                    let event = WorkerEvent::Error("Delivery error".into());
                    return Some((tag.clone(), DeliveryResult::Event(event)));
                }
                Some(None) => {
                    return Some((tag.clone(), DeliveryResult::StreamEnded));
                }
                None => {}
            }
        }

        tokio::task::yield_now().await;
    }
}

fn extract_headers(headers: Option<&FieldTable>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let Some(headers) = headers {
        for (key, value) in headers.inner() {
            let str_val = match value {
                AMQPValue::LongString(s) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
                AMQPValue::ShortString(s) => s.to_string(),
                AMQPValue::Boolean(b) => b.to_string(),
                AMQPValue::ShortInt(n) => n.to_string(),
                AMQPValue::LongInt(n) => n.to_string(),
                AMQPValue::LongLongInt(n) => n.to_string(),
                AMQPValue::ShortUInt(n) => n.to_string(),
                AMQPValue::LongUInt(n) => n.to_string(),
                AMQPValue::Float(n) => n.to_string(),
                AMQPValue::Double(n) => n.to_string(),
                AMQPValue::Timestamp(n) => n.to_string(),
                other => format!("{other:?}"),
            };
            result.insert(key.to_string(), str_val);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use lapin::types::{AMQPValue, FieldTable, LongString, ShortString};

    #[test]
    fn extract_headers_none() {
        let result = extract_headers(None);
        assert!(result.is_empty());
    }

    #[test]
    fn extract_headers_empty_table() {
        let table = FieldTable::default();
        let result = extract_headers(Some(&table));
        assert!(result.is_empty());
    }

    #[test]
    fn extract_headers_long_string() {
        let mut table = FieldTable::default();
        table.insert(
            ShortString::from("key"),
            AMQPValue::LongString(LongString::from(b"hello" as &[u8])),
        );
        let result = extract_headers(Some(&table));
        assert_eq!(result.get("key").unwrap(), "hello");
    }

    #[test]
    fn extract_headers_short_string() {
        let mut table = FieldTable::default();
        table.insert(
            ShortString::from("key"),
            AMQPValue::ShortString(ShortString::from("world")),
        );
        let result = extract_headers(Some(&table));
        assert_eq!(result.get("key").unwrap(), "world");
    }

    #[test]
    fn extract_headers_boolean() {
        let mut table = FieldTable::default();
        table.insert(ShortString::from("t"), AMQPValue::Boolean(true));
        table.insert(ShortString::from("f"), AMQPValue::Boolean(false));
        let result = extract_headers(Some(&table));
        assert_eq!(result.get("t").unwrap(), "true");
        assert_eq!(result.get("f").unwrap(), "false");
    }

    #[test]
    fn extract_headers_integer_types() {
        let mut table = FieldTable::default();
        table.insert(ShortString::from("si"), AMQPValue::ShortInt(42));
        table.insert(ShortString::from("li"), AMQPValue::LongInt(100_000));
        table.insert(ShortString::from("lli"), AMQPValue::LongLongInt(-99));
        table.insert(ShortString::from("su"), AMQPValue::ShortUInt(7));
        table.insert(ShortString::from("lu"), AMQPValue::LongUInt(999));
        let result = extract_headers(Some(&table));
        assert_eq!(result.get("si").unwrap(), "42");
        assert_eq!(result.get("li").unwrap(), "100000");
        assert_eq!(result.get("lli").unwrap(), "-99");
        assert_eq!(result.get("su").unwrap(), "7");
        assert_eq!(result.get("lu").unwrap(), "999");
    }

    #[test]
    fn extract_headers_float_double() {
        let mut table = FieldTable::default();
        table.insert(ShortString::from("f"), AMQPValue::Float(1.5));
        table.insert(ShortString::from("d"), AMQPValue::Double(2.75));
        let result = extract_headers(Some(&table));
        assert_eq!(result.get("f").unwrap(), "1.5");
        assert_eq!(result.get("d").unwrap(), "2.75");
    }

    #[test]
    fn extract_headers_timestamp() {
        let mut table = FieldTable::default();
        table.insert(ShortString::from("ts"), AMQPValue::Timestamp(1_700_000_000));
        let result = extract_headers(Some(&table));
        assert_eq!(result.get("ts").unwrap(), "1700000000");
    }

    #[test]
    fn extract_headers_unknown_uses_debug() {
        let mut table = FieldTable::default();
        table.insert(ShortString::from("v"), AMQPValue::Void);
        let result = extract_headers(Some(&table));
        assert!(result.get("v").unwrap().contains("Void"));
    }

    #[test]
    fn extract_headers_multiple() {
        let mut table = FieldTable::default();
        table.insert(
            ShortString::from("a"),
            AMQPValue::LongString(LongString::from(b"one" as &[u8])),
        );
        table.insert(ShortString::from("b"), AMQPValue::Boolean(true));
        table.insert(ShortString::from("c"), AMQPValue::LongInt(42));
        let result = extract_headers(Some(&table));
        assert_eq!(result.len(), 3);
        assert_eq!(result.get("a").unwrap(), "one");
        assert_eq!(result.get("b").unwrap(), "true");
        assert_eq!(result.get("c").unwrap(), "42");
    }
}
