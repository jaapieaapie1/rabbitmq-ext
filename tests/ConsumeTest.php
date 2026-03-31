<?php

beforeAll(function () {
    $conn = connect();
    $consumer = $conn->consume('test_consume');
    while (($msg = $consumer->next(500)) !== null) {
        $msg->ack();
    }
    $consumer->cancel();
    $conn->close();
});

test('consume with next() returns messages with correct properties', function () {
    $conn = connect();
    $queue = 'test_consume';
    $n = 3;

    for ($i = 1; $i <= $n; $i++) {
        $conn->publish('', $queue, "msg-$i", ['x-index' => (string)$i]);
    }

    $consumer = $conn->consume($queue);

    for ($i = 1; $i <= $n; $i++) {
        $msg = $consumer->next(5000);
        expect($msg)->not->toBeNull();
        expect($msg->getBody())->toBe("msg-$i");
        expect($msg->getRoutingKey())->toBe($queue);
        expect($msg->getExchange())->toBe('');
        expect($msg->getDeliveryTag())->toBeGreaterThan(0);
        $msg->ack();
    }

    $consumer->cancel();
    $conn->close();
});

test('next() with timeout returns null on empty queue', function () {
    $conn = connect();
    $consumer = $conn->consume('test_consume');
    $msg = $consumer->next(500);
    expect($msg)->toBeNull();
    $consumer->cancel();
    $conn->close();
});

test('consumer cancel works without error', function () {
    $conn = connect();
    $consumer = $conn->consume('test_consume');
    $consumer->cancel();
    $conn->close();
    expect(true)->toBeTrue();
});
