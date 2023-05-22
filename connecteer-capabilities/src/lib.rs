#![feature(impl_trait_in_assoc_type, generators, generator_trait)]
#![no_std]

//#![warn(clippy::pedantic)]

mod connection;
pub mod gen_utils;
pub mod identity;
mod middleware;
mod pipeline;
mod sealed;

pub use connection::Connection;
pub use identity::Base;
pub use middleware::Middleware;
pub use pipeline::Pipeline;
