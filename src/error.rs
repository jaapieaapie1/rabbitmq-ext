use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::ce;

/// Convert a lapin or internal error into a PhpException.
pub fn to_php_exception(msg: impl Into<String>) -> PhpException {
    PhpException::new(msg.into(), 0, ce::exception())
}
