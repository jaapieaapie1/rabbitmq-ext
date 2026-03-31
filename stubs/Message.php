<?php

namespace RabbitMQ;

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
     * @throws MessageException
     * @throws ConnectionException
     */
    public function ack(): void {}

    /**
     * Negative-acknowledge the message.
     *
     * @throws MessageException
     * @throws ConnectionException
     */
    public function nack(bool $requeue = true): void {}

    /**
     * Reject the message.
     *
     * @throws MessageException
     * @throws ConnectionException
     */
    public function reject(bool $requeue = true): void {}
}
