<?php

beforeAll(function () {
    // Drain any leftover messages
    $conn = connect();
    $consumer = $conn->consume('test_message');
    while (($msg = $consumer->next(500)) !== null) {
        $msg->ack();
    }
    $consumer->cancel();
    $conn->close();
});

test('ack works', function () {
    $conn = connect();
    $queue = 'test_message';

    $conn->publish('', $queue, 'ack-me');
    $consumer = $conn->consume($queue);
    $msg = $consumer->next(5000);
    expect($msg)->not->toBeNull();
    $msg->ack();
    $consumer->cancel();
    $conn->close();
});

test('nack with requeue makes message reappear', function () {
    $conn = connect();
    $queue = 'test_message';

    $conn->publish('', $queue, 'nack-me');
    $consumer = $conn->consume($queue);
    $msg = $consumer->next(5000);
    expect($msg)->not->toBeNull();
    expect($msg->getBody())->toBe('nack-me');
    $msg->nack(true);

    // Same consumer should see it redelivered
    $msg2 = $consumer->next(5000);
    expect($msg2)->not->toBeNull();
    expect($msg2->getBody())->toBe('nack-me');
    $msg2->ack();
    $consumer->cancel();
    $conn->close();
});

test('reject works', function () {
    $conn = connect();
    $queue = 'test_message';

    $conn->publish('', $queue, 'reject-me');
    $consumer = $conn->consume($queue);
    $msg = $consumer->next(5000);
    expect($msg)->not->toBeNull();
    $msg->reject(false);
    $consumer->cancel();
    $conn->close();
});

test('double ack throws MessageException', function () {
    $conn = connect();
    $queue = 'test_message';

    $conn->publish('', $queue, 'double-ack');
    $consumer = $conn->consume($queue);
    $msg = $consumer->next(5000);
    expect($msg)->not->toBeNull();
    $msg->ack();

    try {
        $msg->ack();
        throw new \RuntimeException('Expected MessageException was not thrown');
    } catch (RabbitMQ\MessageException $e) {
        // expected
    } finally {
        $consumer->cancel();
        $conn->close();
    }

    expect($e)->toBeInstanceOf(RabbitMQ\MessageException::class);
});

test('headers round-trip correctly', function () {
    $conn = connect();
    $queue = 'test_message';

    $headers = [
        'x-string' => 'hello',
        'x-number' => '42',
        'x-empty' => '',
    ];
    $conn->publish('', $queue, 'headers-test', $headers);
    $consumer = $conn->consume($queue);
    $msg = $consumer->next(5000);
    expect($msg)->not->toBeNull();

    $received = $msg->getHeaders();
    expect($received['x-string'])->toBe('hello');
    expect($received['x-number'])->toBe('42');
    expect($received['x-empty'])->toBe('');

    $msg->ack();
    $consumer->cancel();
    $conn->close();
});
