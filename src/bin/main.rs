#![allow(non_snake_case)]
use futures::executor::block_on;
use std::process::exit;

use bodo_connect::cmd;
use clap::Parser;

#[tokio::main]
async fn main() {
    let mut cmd = cmd::Cmd::parse();
    #[cfg(feature = "sync")]
    if !(cmd.migrate_to_yaml || cmd.pull_config || cmd.push_config) {
        cmd.check_host();
    }
    #[cfg(not(feature = "sync"))]
    if !cmd.migrate_to_yaml {
        cmd.check_host();
    }
    exit(match block_on(cmd.main()) {
        Ok(..) => 0,
        Err(e) => {
            e.print_error();
            e.exit_code()
        }
    })
}
