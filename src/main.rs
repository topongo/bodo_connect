#[cfg(feature = "cmd")]
mod cmd;
mod net;
mod logger;
#[cfg(feature = "wake")]
mod waker;
mod ssh;

use std::process::exit;
use futures::executor::block_on;
use clap::Parser;

#[tokio::main]
async fn main() {
    let mut cmd = cmd::Cmd::parse();

    exit(match block_on(cmd.main()) {
        Ok(..) => 0,
        Err(e) => {
            e.print_error();
            e.exit_code()
        }
    })
}

