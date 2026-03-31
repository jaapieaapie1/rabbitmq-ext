#[cfg(not(test))]
use ext_php_rs::prelude::*;
#[cfg(not(test))]
use ext_php_rs::zend::ce;

#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\Exception", extends(ce = ce::exception, stub = "\\Exception")))]
#[derive(Default)]
pub struct RabbitMQException;

#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\ConnectionException", extends(RabbitMQException)))]
#[derive(Default)]
pub struct ConnectionException;

#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\ConsumeException", extends(RabbitMQException)))]
#[derive(Default)]
pub struct ConsumeException;

#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\PublishException", extends(RabbitMQException)))]
#[derive(Default)]
pub struct PublishException;

#[cfg_attr(not(test), php_class)]
#[cfg_attr(not(test), php(name = "RabbitMQ\\MessageException", extends(RabbitMQException)))]
#[derive(Default)]
pub struct MessageException;
