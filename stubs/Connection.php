<?php

namespace RabbitMQ;

class Connection
{
    /**
     * Connect to RabbitMQ. Blocks until connected or throws on failure.
     *
     * @throws ConnectionException
     */
    public function __construct(string $uri) {}

    /**
     * Start consuming from a queue.
     *
     * @throws ConnectionException
     */
    public function consume(string $queue, string $consumerTag = "", int $prefetchCount = 10): Consumer {}

    /**
     * Publish a message and wait for broker confirm.
     *
     * @param array<string, string> $headers
     * @throws ConnectionException
     * @throws PublishException
     */
    public function publish(string $exchange, string $routingKey, string $body, array $headers = []): void {}

    /**
     * Publish a message without waiting for confirm.
     * The message is guaranteed to be confirmed before the connection is closed.
     *
     * @param array<string, string> $headers
     * @throws ConnectionException
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
