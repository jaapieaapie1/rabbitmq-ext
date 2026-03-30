<?php

namespace RabbitMQ;

class Connection
{
    /**
     * Connect to RabbitMQ. Blocks until connected or throws on failure.
     *
     * @throws \Exception
     */
    public function __construct(string $uri) {}

    /**
     * Start consuming from a queue.
     *
     * @throws \Exception
     */
    public function consume(string $queue, string $consumerTag = "", int $prefetchCount = 10): Consumer {}

    /**
     * Publish a message and wait for broker confirm.
     *
     * @param array<string, string> $headers
     * @throws \Exception
     */
    public function publish(string $exchange, string $routingKey, string $body, array $headers = []): void {}

    /**
     * Publish a message without waiting for confirm.
     * The message is guaranteed to be confirmed before the connection is closed.
     *
     * @param array<string, string> $headers
     * @throws \Exception
     */
    public function publishAsync(string $exchange, string $routingKey, string $body, array $headers = []): void {}

    /**
     * Check if the connection is still alive.
     */
    public function isConnected(): bool {}

    /**
     * Close the connection gracefully. Waits for pending publishes to confirm.
     */
    public function close(): void {}
}

class Consumer
{
    /**
     * Receive the next message. Returns null on timeout or consumer cancellation.
     *
     * @param int $timeoutMs 0 = block indefinitely
     * @throws \Exception
     */
    public function next(int $timeoutMs = 0): ?Message {}

    /**
     * Loop over messages, calling $callback for each.
     * Stops if callback returns false, consumer is cancelled, or timeout expires between messages.
     *
     * @param callable(Message): bool $callback
     * @param int $timeoutMs 0 = block indefinitely
     * @throws \Exception
     */
    public function each(callable $callback, int $timeoutMs = 0): void {}

    /**
     * Cancel this consumer.
     *
     * @throws \Exception
     */
    public function cancel(): void {}
}

class Message
{
    public function getBody(): string {}

    public function getRoutingKey(): string {}

    public function getExchange(): string {}

    public function getDeliveryTag(): int {}

    /**
     * @return array<string, string>
     */
    public function getHeaders(): array {}

    /**
     * Acknowledge the message.
     *
     * @throws \Exception
     */
    public function ack(): void {}

    /**
     * Negative-acknowledge the message.
     *
     * @throws \Exception
     */
    public function nack(bool $requeue = true): void {}

    /**
     * Reject the message.
     *
     * @throws \Exception
     */
    public function reject(bool $requeue = true): void {}
}
