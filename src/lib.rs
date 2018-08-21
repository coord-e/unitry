pub mod protos;
pub mod device;
pub mod error;
pub mod resolve;

extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate serde_yaml;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;
