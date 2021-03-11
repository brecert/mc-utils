# mc-utils
> cli utility functions for minecraft

```
mc-utils 0.1.0

USAGE:
    mc-utils.exe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help         Prints this message or the help of the given subcommand(s)
    server       utilities for pinging and checking if a server is blocked by mojang
    skin         utilities for fetching and looking at skin information
    usernames    displays the username history of a user
```

# Installation
Compiled binaries can be downloaded from the [releases](https://github.com/Brecert/mc-utils/releases/)

# Building
To build mc-utils, you'll need to be using [rust](https://www.rust-lang.org/) v1.48.0 (nightly) or higher

to build:
```
$ git clone https://github.com/Brecert/mc-utils
$ cd mc-utils
$ cargo build --release
$ ./target/release/mc-utils --version
mc-utils 0.1.0
```
