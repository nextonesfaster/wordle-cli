# wordle-cli (wrdl)

`wordle-cli` (binary name: `wrdl`) is a terminal-based game of [Wordle][wordle].

[![asciicast](https://asciinema.org/a/Pu1Z264wCX0MSa2jLyy9gxBSt.svg)](https://asciinema.org/a/Pu1Z264wCX0MSa2jLyy9gxBSt)

```txt
wrdl0.1.0
Sujal Bolia <sujalbolia@gmail.com>

wordle-cli (wrdl) is a terminal-based game of Wordle.

USAGE:
    wrdl [OPTIONS]

OPTIONS:
    -a, --allowed-guesses [path]    Specify path to allowed guesses file, leave blank to unset
    -h, --help                      Print help information
    -r, --reset                     Set the next word pointer to the beginning
    -V, --version                   Print version information
    -w, --words [path]              Specify path to allowed words file, leave blank to unset
```

## Installation

You need [Rust][rust] to compile `wordle-cli`.

`cargo` is usually installed with Rust. If you don't have `cargo` installed, follow [the `cargo` installation documentation][cargo].

Once you have `cargo` installed, you can simply use `cargo install` or compile from the source.

To use `cargo install`:

```sh
cargo install --git https://github.com/nextonesfaster/wordle-cli
```

`cargo` will install `wrdl` in its `bin` directory, which should already be in your `PATH`.

To compile from source:

```sh
# Clone this repository
$ git clone https://github.com/nextonesfaster/wordle-cli.git

# cd into the cloned repository
$ cd wordle-cli

# Compile using cargo with the release flag
$ cargo build --release
```

The executable will be at `./target/release/wrdl`. You can move it to your `PATH` to invoke `wrdl` from any directory.

## Configuration

`wordle-cli` uses two lists of words: valid words and allowed guesses. These words are stored in json files. See the [data](data) directory for the default lists.

Default lists are included in the binary when it is compiled. You can provide custom lists using the `-w` and `-a` options.

The application selects words in the order they are listed in the valid words file. A pointer keeping track of the next valid word increases at the end of every round—you can reset this by using the `-r` flag.

All this data is stored in a json data file.

### Location

By default, the location of this file is `$DATA_DIR/wordle-cli/data.json` where `$DATA_DIR` is as follows:

| Platform |                `$DATA_DIR`                 |
| :------: | :----------------------------------------: |
|  Linux   |         `/home/Alice/.local/share`         |
|  macOS   | `/Users/Alice/Library/Application Support` |
| Windows  |      `C:\Users\Alice\AppData\Roaming`      |

You can override this by setting the `WORDLE_CLI_DATA` environment variable as the path of the json data file. The environment variable takes precedence over the default location.

## Acknowledgement

The default valid words and allowed guesses lists are taken from [Wordle][wordle].

## License

`wordle-cli` is distributed under the terms of both the MIT License and the Apache License 2.0.

See the [LICENSE-MIT][mit] and [LICENSE-APACHE][apache] files for more details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[rust]: https://www.rust-lang.org/tools/install
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[mit]: LICENSE-MIT
[apache]: LICENSE-APACHE
[wordle]: https://www.nytimes.com/games/wordle/index.html
