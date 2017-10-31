[![crates.io](https://img.shields.io/crates/v/base100.svg)](https://crates.io/crates/base100)

# Base💯

Encode things into Emoji.

Base💯 can represent any byte with a unique emoji symbol, therefore it can
represent binary data with zero printable overhead (see caveats for more info).

## Usage

```
$ echo "the quick brown fox jumped over the lazy dog" | base100
👫👟👜🐗👨👬👠👚👢🐗👙👩👦👮👥🐗👝👦👯🐗👡👬👤👧👜👛🐗👦👭👜👩🐗👫👟👜🐗👣👘👱👰🐗👛👦👞🐁
```

Base💯 will read from stdin unless a file is specified, will write UTF-8 to
stdout, and has a similar API to GNU's base64. Data is encoded by default,
unless `--decode` is specified; the `--encode` flag does nothing and exists
solely to accommodate lazy people who don't want to read the docs (like me).

```
USAGE:
    base100 [FLAGS] [input]

FLAGS:
    -d, --decode     Tells base💯 to decode this data
    -e, --encode     Tells base💯 to encode this data
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <input>    The input file to use
```

## Installation

To install base💯, use cargo:

```shell
$ cargo install base100
```

base💯 also has an AVX-accelerated implementation, delivering up to 4x faster
performance. If you have a capable CPU and nightly rust, install it as such:

```shell
$ RUSTFLAGS="-C target-cpu=native" cargo install base100 --features simd
```

## Performance

base💯's performance is very competitive with other encoding algorithms.

### Scalar Performance

```
$ base100 --version
base💯 0.4.1

$ base64 --version
base64 (GNU coreutils) 8.28

$ cat /dev/urandom | pv | base100 > /dev/null
 [ 247MiB/s]

$ cat /dev/urandom | pv | base64 > /dev/null
 [ 232MiB/s]

$ cat /dev/urandom | pv | base100 | base100 -d > /dev/null
 [ 233MiB/s]

$ cat /dev/urandom | pv | base64 | base64 -d > /dev/null
 [ 176MiB/s]
```

In both scenarios, base💯 compares favorably to GNU base64.

### SIMD Performance

On a machine supporting AVX2, base💯 gains a 4x performance boost via some
hand-tuned x86-64 assembly. Support for SSE2 will come soon, and I will happily
support AVX-512 if some compatible hardware finds its way into my hands.

To receive this speedup: you must use:
- An AVX2-capable processor (Newer than Broadwell, or Zen)
- Nightly Rust

To build the SIMD-accelerated version, simply go to your project directory and
type

```shell
$ RUSTFLAGS="-C target-cpu=native" cargo +nightly build --release --features simd
```

Please note that the below benchmarks were taken on a significantly weaker
machine than the above benchmarks, and cannot be directly compared.

```
$ base100 --version
base💯 0.4.1

$ base64 --version
base64 (GNU coreutils) 8.28

$ cat /dev/zero | pv | ./base100 > /dev/null
 [1.14GiB/s]

$ cat /dev/zero | pv | base64 > /dev/null
 [ 479MiB/s]

$ cat /dev/zero | pv | ./base100 | ./base100 -d > /dev/null
 [ 412MiB/s]

$ cat /dev/zero | pv | base64 | base64 -d > /dev/null
 [ 110MiB/s]
```

In this scenario, base💯 compares very favorably to GNU base64.

## Caveats

Base💯 is *very* space inefficient. It bloats the size of your data by around 3x,
and should only be used if you have to display encoded binary data in as few
__printable__ characters as possible. It is, however, very suitable for human
interaction. Encoded hashes and checksums become very easy to verify at a glance,
and take up much less space on a terminal.

## Future plans

- Allow data to be encoded with the full 1024-element emoji set
- Add further optimizations and ensure we're abusing SIMD as much as possible
- Add multiprocessor support
