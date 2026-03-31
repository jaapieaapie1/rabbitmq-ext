<?php

test('connect succeeds and isConnected returns true', function () {
    $conn = connect();
    expect($conn->isConnected())->toBeTrue();
    $conn->close();
});

test('close works and isConnected returns false after', function () {
    $conn = connect();
    $conn->close();
    expect($conn->isConnected())->toBeFalse();
});

test('connection to invalid URI throws ConnectionException', function () {
    new RabbitMQ\Connection('amqp://guest:guest@localhost:59999');
})->throws(RabbitMQ\ConnectionException::class);
