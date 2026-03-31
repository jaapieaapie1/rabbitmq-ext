<?php

beforeAll(function () {
    $conn = connect();
    $consumer = $conn->consume('test_consume_each');
    while (($msg = $consumer->next(500)) !== null) {
        $msg->ack();
    }
    $consumer->cancel();
    $conn->close();
});

test('each() receives all published messages', function () {
    $conn = connect();
    $queue = 'test_consume_each';
    $n = 5;

    for ($i = 1; $i <= $n; $i++) {
        $conn->publish('', $queue, "each-$i");
    }

    $received = [];
    $consumer = $conn->consume($queue);
    $consumer->each(function (RabbitMQ\Message $msg) use (&$received, $n) {
        $received[] = $msg->getBody();
        $msg->ack();
        return count($received) < $n;
    }, 5000);
    $consumer->cancel();

    expect($received)->toHaveCount($n);
    for ($i = 1; $i <= $n; $i++) {
        expect($received[$i - 1])->toBe("each-$i");
    }

    $conn->close();
});

test('returning false from callback stops iteration', function () {
    $conn = connect();
    $queue = 'test_consume_each';

    $conn->publish('', $queue, 'stop-1');
    $conn->publish('', $queue, 'stop-2');
    $conn->publish('', $queue, 'stop-3');

    $received = [];
    $consumer = $conn->consume($queue);
    $consumer->each(function (RabbitMQ\Message $msg) use (&$received) {
        $received[] = $msg->getBody();
        $msg->ack();
        return false;
    }, 5000);
    $consumer->cancel();

    expect($received)->toHaveCount(1);

    // Drain remaining
    $consumer2 = $conn->consume($queue);
    while ($msg = $consumer2->next(500)) {
        $msg->ack();
    }
    $consumer2->cancel();
    $conn->close();
});

test('each() returns on timeout when queue is empty', function () {
    $conn = connect();
    $consumer = $conn->consume('test_consume_each');

    $start = microtime(true);
    $consumer->each(function (RabbitMQ\Message $msg) {
        $msg->ack();
        return true;
    }, 500);
    $elapsed = (microtime(true) - $start) * 1000;

    $consumer->cancel();
    $conn->close();

    expect($elapsed)->toBeLessThan(3000);
});
