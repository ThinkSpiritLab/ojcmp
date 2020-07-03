# ojcmp

[![crates.io](https://img.shields.io/crates/v/ojcmp.svg)](https://crates.io/crates/ojcmp) ![Test](https://github.com/ThinkSpiritLab/ojcmp/workflows/Test/badge.svg)

> online judge comparer

## Status

Maintaining `0.2.2`

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
ojcmp 0.2.2

USAGE:
    ojcmp [FLAGS] [OPTIONS] --std <path>

FLAGS:
    -a, --all          Reads all bytes of user file even if it's already WA
    -b, --backtrace    Prints stack backtrace when fatal error occurs
    -h, --help         Prints help information
    -V, --version      Prints version information

OPTIONS:
        --eps <eps>      Eps for float comparing
    -m, --mode <mode>    CompareMode ("normal"|"strict"|"spj_float") [default: normal]
    -s, --std <path>     Std file path
    -u, --user <path>    User file path. Reads from stdin if it's not given
```

## Return Value

| type   | value                                      |
| ------ | ------------------------------------------ |
| code   | errno                                      |
| stdout | "AC" / "WA" / "PE"                         |
| stderr | error message and optional stack backtrace |

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

### Mode: SpjFloat

Compare two streams of float numbers which are splitted by [ascii whitespaces](https://infra.spec.whatwg.org/#ascii-whitespace).

Parse any float number as f64 (aka `double` in C language).

Ascii whitespaces between two float numbers are considered as a single separator symbol.

Use CLI option `--eps` to specify eps value, for example `--eps 1e-3`.

There is no "PE" in this mode.

## Change Log

- v0.2.2 Fix bug in nan handling. (yank v0.2.1)
- v0.2.1 Add spj_float mode. (yanked)
- v0.2.0 Add strict mode. No break changes.

+ v0.1.3 No functional changes
+ v0.1.2 Fix algorithm bug
+ v0.1.1 Use unsafe static buffer