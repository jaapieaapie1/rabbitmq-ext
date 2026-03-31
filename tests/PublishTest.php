<?php

afterAll(function () {
    $conn = connect();
    $consumer = $conn->consume('test_publish');
    while ($msg = $consumer->next(500)) {
        $msg->ack();
    }
    $consumer->cancel();
    $conn->close();
});

test('publish to default exchange succeeds', function () {
    $conn = connect();
    $conn->publish('', 'test_publish', 'hello');
    $conn->close();
    expect(true)->toBeTrue();
});

test('publish with headers succeeds', function () {
    $conn = connect();
    $conn->publish('', 'test_publish', 'with headers', ['x-foo' => 'bar', 'x-baz' => 'qux']);
    $conn->close();
    expect(true)->toBeTrue();
});

test('publishAsync succeeds', function () {
    $conn = connect();
    $conn->publishAsync('', 'test_publish', 'async message', ['x-async' => 'true']);
    $conn->close();
    expect(true)->toBeTrue();
});

test('publish to non-existent exchange throws PublishException', function () {
    $conn = connect();
    $conn->publish('nonexistent_exchange_that_does_not_exist', 'test_publish', 'fail');
    $conn->close();
})->throws(RabbitMQ\PublishException::class);
