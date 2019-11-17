#![warn(rust_2018_idioms)]

#![cfg_attr(feature="nightly",feature(trace_macros))]
#![cfg_attr(feature="nightly",feature(log_syntax))]

include!(concat!(env!("OUT_DIR"), "/crate_top.rs"));

mod big;


