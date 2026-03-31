<?php

function connect(): RabbitMQ\Connection
{
    $uri = getenv('RABBITMQ_URI') ?: 'amqp://guest:guest@localhost:5672';

    return new RabbitMQ\Connection($uri);
}
