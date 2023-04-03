#![feature(future_join)]

extern crate core;

mod ssh;
mod waker;
mod net;
mod bodo_connect;
mod logger;

use clap::Parser;
use futures::executor::block_on;
use std::process::exit;

use crate::bodo_connect::BodoConnect;


fn main() {
    let mut bc: BodoConnect = bodo_connect::BodoConnect::parse();
    exit(block_on(bc.main()))
}

