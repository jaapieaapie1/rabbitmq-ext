mod connection;
mod consumer;
mod error;
mod message;
mod protocol;
mod worker;

use ext_php_rs::prelude::*;

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<connection::Connection>()
        .class::<consumer::Consumer>()
        .class::<message::Message>()
}
