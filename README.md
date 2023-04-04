# bodo_connect
* A library for mapping your hosts in the world wide web.
* A tool for connecting to any host of your map using ssh

It supports:
* Waking the host before connecting to it.
* Retry connection until ssh returns 0
* Automatic subnetwork detection for avoiding unneeded jump hosts

## Installing
Install it with `cargo install bodo_connect`.
Or build it simply by cloning this repo using cargo:
```shell
git clone https://github.com/topongo/bodo_connect-rs bodo_connect
cd bodo_connect
cargo build --release
target/release/bodoConnect
```

### *NB*
For legacy reasons, the binary for this crate is named `bodoConnect` and not `bodo_connect`.

## Features
* **cmd**: (required for binary) actual binary in action
* **serde**: (required for binary) parse networkmap from json file
* **log**: enable logging (it doesn't automatically set to level Debug, it must be manually done if the **cmd** feature is off)
* **wake**: enable waking hosts by either making a GET request or executing the `wol` command on the master host of the target network.

## Command Usage
```
Usage: bodoConnect [OPTIONS] <HOST> [EXTRA]...

Arguments:
  <HOST>      Host to connect to
  [EXTRA]...  Extra argument(s), if no -S or -R is used, it will be passed to the remote machine as command

Options:
      --networkmap <NETWORKMAP>  Select different networkmap.json file
  -w, --wake                     Wake host before connecting
  -t, --tty                      Pass -t parameter to ssh (force tty allocation)
  -d...                          Set verbosity level
  -q, --quiet                    Don't log anything
  -n, --dry                      Send to stdout the generated command without executing it
  -R, --rsync                    [WIP] Creates rsync commands
  -S, --sshfs                    [WIP] Creates sshfs commands
  -l, --loop                     Retry connection until ssh returns 0
  -h, --help                     Print help
  -V, --version                  Print version
```

## Networkmap
A little clarification on how a networkmap is structured:

* `NetworkMap`: a list of `Subnets`
* `Subnet`: 
    * An physical/abstract local network. It is identified by its `subdomain` and it contains a list of `hosts` (and optionally by a static external ip address, `eip`).
* `Host`:
    * Identified by its `name`, that must be unique in the whole network map, it must contain:
        * An `ip` address
        * A `port`
        * A `user`
    * It can contain also an `eport`.
        * If set, the host becomes the subnet `master`, so it is considered to be always powered on and exposed for incoming ssh connection from foreign hosts
        * There can only be one master host per subnet
* `Waker`: an optional structure that defines how a host can be wakened. It supports:
    * http(s):
        * GET method
        * ~~POST method~~ (coming soon)
    * wol (wake on lan)

[Example](networkmap.example.md) of a newtorkmap
