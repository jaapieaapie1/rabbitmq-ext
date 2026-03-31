mod connection;
mod consumer;
mod error;
mod exceptions;
mod message;
mod protocol;
mod worker;

#[cfg(not(test))]
use ext_php_rs::prelude::*;

#[cfg(not(test))]
#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<exceptions::RabbitMQException>()
        .class::<exceptions::ConnectionException>()
        .class::<exceptions::ConsumeException>()
        .class::<exceptions::PublishException>()
        .class::<exceptions::MessageException>()
        .class::<connection::Connection>()
        .class::<consumer::Consumer>()
        .class::<message::Message>()
}

/// Stub symbols for PHP/Zend C API that ext-php-rs references.
/// These are never called during tests — they only satisfy the linker.
#[cfg(test)]
mod php_stubs {
    use std::ffi::c_void;

    #[unsafe(no_mangle)]
    pub static mut executor_globals: [u8; 4096] = [0; 4096];
    #[unsafe(no_mangle)]
    pub static mut zend_ce_exception: usize = 0;
    #[unsafe(no_mangle)]
    pub static mut zend_ce_traversable: usize = 0;
    #[unsafe(no_mangle)]
    pub static mut zend_empty_string: usize = 0;
    #[unsafe(no_mangle)]
    pub static mut zend_one_char_string: [usize; 256] = [0; 256];

    #[unsafe(no_mangle)]
    pub extern "C" fn __zend_malloc(_size: usize) -> *mut c_void { std::ptr::null_mut() }
    #[unsafe(no_mangle)]
    pub extern "C" fn gc_possible_root(_zv: *mut c_void) {}
    #[unsafe(no_mangle)]
    pub extern "C" fn instanceof_function_slow(_: *const c_void, _: *const c_void) -> u8 { 0 }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_array_count(_ht: *const c_void) -> u32 { 0 }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_hash_get_current_data_ex(_ht: *const c_void, _pos: *mut c_void) -> *mut c_void { std::ptr::null_mut() }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_hash_get_current_key_type_ex(_ht: *const c_void, _pos: *mut c_void) -> i32 { 0 }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_hash_get_current_key_zval_ex(_ht: *const c_void, _key: *mut c_void, _pos: *mut c_void) {}
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_hash_move_forward_ex(_ht: *const c_void, _pos: *mut c_void) -> i32 { 0 }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_is_iterable(_zv: *const c_void) -> bool { false }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_lookup_class_ex(_name: *const c_void, _key: *const c_void, _flags: u32) -> *mut c_void { std::ptr::null_mut() }
    #[unsafe(no_mangle)]
    pub extern "C" fn zend_objects_store_del(_obj: *mut c_void) {}
    #[unsafe(no_mangle)]
    pub extern "C" fn zval_ptr_dtor(_zv: *mut c_void) {}
}
