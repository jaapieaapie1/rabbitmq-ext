<?php

test('RabbitMQ\\Exception extends \\Exception', function () {
    expect(is_subclass_of(RabbitMQ\Exception::class, \Exception::class))->toBeTrue();
});

test('ConnectionException extends RabbitMQ\\Exception', function () {
    expect(is_subclass_of(RabbitMQ\ConnectionException::class, RabbitMQ\Exception::class))->toBeTrue();
    expect(is_subclass_of(RabbitMQ\ConnectionException::class, \Exception::class))->toBeTrue();
});

test('ConsumeException extends RabbitMQ\\Exception', function () {
    expect(is_subclass_of(RabbitMQ\ConsumeException::class, RabbitMQ\Exception::class))->toBeTrue();
    expect(is_subclass_of(RabbitMQ\ConsumeException::class, \Exception::class))->toBeTrue();
});

test('PublishException extends RabbitMQ\\Exception', function () {
    expect(is_subclass_of(RabbitMQ\PublishException::class, RabbitMQ\Exception::class))->toBeTrue();
    expect(is_subclass_of(RabbitMQ\PublishException::class, \Exception::class))->toBeTrue();
});

test('MessageException extends RabbitMQ\\Exception', function () {
    expect(is_subclass_of(RabbitMQ\MessageException::class, RabbitMQ\Exception::class))->toBeTrue();
    expect(is_subclass_of(RabbitMQ\MessageException::class, \Exception::class))->toBeTrue();
});
