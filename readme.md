# mc-utils
> cli utility functions for minecraft

```
mc-utils 0.2.0

USAGE:
    mc-utils.exe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    player    utilities for fetching and viewing player information
    server    utilities for pinging and checking if a server is blocked by mojang
    skin      utilities for fetching and viewing skin information
    ping      alias for `server ping`
```

# Installation
Compiled binaries can be downloaded from the [releases](https://github.com/brecert/mc-utils/releases/)

# Building
To build mc-utils, you'll need to be using [rust](https://www.rust-lang.org/) v1.48.0 (nightly) or higher

to build:
```
$ git clone https://github.com/Brecert/mc-utils
$ cd mc-utils
$ cargo build --release
$ ./target/release/mc-utils --version
mc-utils 0.2.0
```
