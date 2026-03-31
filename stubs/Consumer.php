<?php

namespace RabbitMQ;

class Consumer
{
    /**
     * Receive the next message. Returns null on timeout or consumer cancellation.
     *
     * @param int $timeoutMs 0 = block indefinitely
     * @throws ConsumeException
     */
    public function next(int $timeoutMs = 0): ?Message {}

    /**
     * Loop over messages, calling $callback for each.
     * Stops if callback returns false, consumer is cancelled, or timeout expires between messages.
     *
     * @param callable(Message): bool $callback
     * @param int $timeoutMs 0 = block indefinitely
     * @throws ConsumeException
     */
    public function each(callable $callback, int $timeoutMs = 0): void {}

    /**
     * Cancel this consumer.
     *
     * @throws ConnectionException
     */
    public function cancel(): void {}
}
