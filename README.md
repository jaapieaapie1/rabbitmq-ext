# RabbitMQ-ext

A PHP extension for consuming RabbitMQ queues,
written in Rust using [ext-php-rs](https://github.com/davidcole1340/ext-php-rs)
and [lapin](https://github.com/amqp-rs/lapin).  

## Why

The benefit of this approach compared to implementing an RabbitMQ client in PHP
is that we can multithread and thus keep sending heartbeats even though the php
process is busy doing other things.  
This makes it so the connection stays open even when you are processing a job
that takes 30 minutes.

## Support

This package officially supports and is tested against PHP 8.1 - 8.5.

## Installing

To install the extension use PHP's official extension installer PIE.
The extension is [published on packagist](https://packagist.org/packages/jaapieaapie1/rabbitmq-ext).  
For most architecture php combinations prebuilt binaries are available
(only PIE version > 1.4 supports downloading prebuilt binaries)  
If your specific setup is not prebuilt
[Rust's buildtools](https://rustup.rs/)
are required to build this library.

```bash
pie install jaapieaapie1/rabbitmq-ext
```

For IDE's and PHPStan there are stubs available as a [composer package](https://packagist.org/packages/jaapieaapie1/rabbitmq-ext-stubs).  

```bash
composer require --dev jaapieaapie1/rabbitmq-ext-stubs
```

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
