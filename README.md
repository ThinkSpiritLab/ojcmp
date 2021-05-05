# ojcmp

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
![CI][ci-badge]

[crates-badge]: https://img.shields.io/crates/v/ojcmp.svg
[crates-url]: https://crates.io/crates/ojcmp
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[ci-badge]: https://github.com/ThinkSpiritLab/ojcmp/workflows/CI/badge.svg

> online judge comparer

## Status

Maintaining `0.4.0`

## Install

```bash
cargo install ojcmp
```

## Build

```bash
cargo build --release
```

Install by cargo

```bash
cargo install --path .
```

Install manually

```bash
cp target/release/ojcmp /usr/bin
```

## Usage

```
ojcmp 0.4.0
Nugine <nugine@foxmail.com>

USAGE:
    ojcmp <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    float     Float compare
    help      Prints this message or the help of the given subcommand(s)
    normal    Normal compare
    strict    Strict compare
```

```
ojcmp-normal 0.4.0
Normal compare

USAGE:
    ojcmp normal [FLAGS] [OPTIONS] <--std <path>|--std-fd <fd>> <--user <path>|--user-fd <fd>>

FLAGS:
    -h, --help        Prints help information
    -q, --quiet       No output printed to stdout or stderr
    -a, --read-all    Reads all bytes of user file even if it's already WA
    -V, --version     Prints version information

OPTIONS:
    -b, --buffer-size <bytes>    Buffer size (in bytes) for both std and user file [default: 65536]
    -s, --std <path>             Std file path
        --std-fd <fd>            Std file descriptor
    -u, --user <path>            User file path
        --user-fd <fd>           User file descriptor
```

## Return Value

| type              | value              |
| ----------------- | ------------------ |
| exit_code (AC)    | 0                  |
| exit_code (WA)    | 1                  |
| exit_code (PE)    | 2                  |
| exit_code (error) | 101                |
| stdout            | "AC" / "WA" / "PE" |
| stderr            | error message      |

## Current Implementation

### Mode: Normal

trim_end(file)

```rust
judge!(AC, b"1\r\n\r\n\r\n", b"1  ");
```

for each line, trim_end(line)

```rust
judge!(AC, b"1 \n", b"1");
```

for each line, check spaces between non-space chars

```rust
judge!(PE, b"1 3\n", b"1         3\n");
```

### Mode: Strict

User file must have the same bytes with std file.

The two byte streams must be exactly the same.

There is no "PE" in this mode.

### Mode: Float

Compare two streams of float numbers which are splitted by [ascii whitespaces](https://infra.spec.whatwg.org/#ascii-whitespace).

Parse any float number as f64 (aka `double` in C language).

Ascii whitespaces between two float numbers are considered as a single separator symbol.

Use CLI option `--eps` to specify eps value, for example `--eps 1e-3`.

There is no "PE" in this mode.

## Change Log

- v0.4.0 Allow passing file fd. Change return value to indicate comparison result.

+ v0.3.2 Fix unsoundness in ByteReader.
+ v0.3.1 Fix performance regression since v0.2.0. Allow unsafe code. 
+ v0.3.0 Forbid unsafe code. Use subcommands for different modes.

- v0.2.2 Fix bug in nan handling. (yank v0.2.1)
- v0.2.1 Add spj_float mode. (yanked)
- v0.2.0 Add strict mode. No break changes.

+ v0.1.3 No functional changes
+ v0.1.2 Fix algorithm bug
+ v0.1.1 Use unsafe static buffer