# RabbitMQ-ext

A PHP extension for consuming RabbitMQ queues, written in Rust using [ext-php-rs](https://github.com/davidcole1340/ext-php-rs) and [lapin](https://github.com/amqp-rs/lapin).

## Usage

### Consuming messages

```php
use RabbitMQ\Connection;

$connection = new Connection('amqp://guest:guest@localhost:5672');
$consumer = $connection->consume('my-queue', prefetchCount: 50);

$consumer->each(function ($message) {
    echo $message->getBody() . PHP_EOL;

    $message->ack();

    return true; // return false to stop consuming
});

$connection->close();
```

### Consuming with a timeout

```php
$consumer = $connection->consume('my-queue');

while ($message = $consumer->next(timeoutMs: 5000)) {
    echo $message->getRoutingKey() . ': ' . $message->getBody() . PHP_EOL;
    $message->ack();
}

// null returned — timed out or consumer was cancelled
$connection->close();
```

### Publishing messages

```php
$connection = new Connection('amqp://guest:guest@localhost:5672');

// Publish and wait for broker confirmation
$connection->publish('my-exchange', 'routing.key', '{"event":"order.created"}', [
    'x-request-id' => 'abc-123',
]);

// Fire-and-forget (confirmed before connection closes)
$connection->publishAsync('my-exchange', 'routing.key', 'payload');

$connection->close();
```

### Handling failures

```php
$consumer = $connection->consume('my-queue');

$consumer->each(function ($message) {
    try {
        processMessage($message->getBody());
        $message->ack();
    } catch (\Throwable $e) {
        $message->nack(requeue: false); // dead-letter the message
    }

    return true;
});
```
