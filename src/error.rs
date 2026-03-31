#[cfg(not(test))]
use crate::exceptions::{
    ConnectionException, ConsumeException, MessageException, PublishException,
};
#[cfg(not(test))]
use ext_php_rs::class::RegisteredClass;
use ext_php_rs::exception::PhpException;

#[cfg(not(test))]
pub fn connection_exception(msg: impl Into<String>) -> PhpException {
    PhpException::new(msg.into(), 0, ConnectionException::get_metadata().ce())
}

#[cfg(not(test))]
pub fn consume_exception(msg: impl Into<String>) -> PhpException {
    PhpException::new(msg.into(), 0, ConsumeException::get_metadata().ce())
}

#[cfg(not(test))]
pub fn publish_exception(msg: impl Into<String>) -> PhpException {
    PhpException::new(msg.into(), 0, PublishException::get_metadata().ce())
}

#[cfg(not(test))]
pub fn message_exception(msg: impl Into<String>) -> PhpException {
    PhpException::new(msg.into(), 0, MessageException::get_metadata().ce())
}

#[cfg(test)]
pub fn connection_exception(msg: impl Into<String>) -> PhpException {
    PhpException::default(msg.into())
}

#[cfg(test)]
pub fn consume_exception(msg: impl Into<String>) -> PhpException {
    PhpException::default(msg.into())
}

#[cfg(test)]
pub fn publish_exception(msg: impl Into<String>) -> PhpException {
    PhpException::default(msg.into())
}

#[cfg(test)]
pub fn message_exception(msg: impl Into<String>) -> PhpException {
    PhpException::default(msg.into())
}
