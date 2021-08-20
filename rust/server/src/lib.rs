#![allow(dead_code)]

extern crate anyhow;
extern crate backtrace;
extern crate futures;
extern crate grpcio;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate pb_convert;
extern crate protobuf;
extern crate rayon;
extern crate risingwave_proto;
extern crate thiserror;
extern crate tokio;
extern crate typed_builder;

#[macro_use]
mod error;
#[macro_use]
mod util;
mod alloc;
mod array;
mod buffer;
mod catalog;
mod execution;
mod executor;
mod expr;
pub(crate) mod service;
mod storage;
mod task;
mod types;
mod vector_op;

pub mod server;
