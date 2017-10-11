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

## Caveats

Base💯 is *very* space inefficient. It bloats the size of your data by around 3x,
and should only be used if you have to display encoded binary data in as few
__printable__ characters as possible. It is, however, very suitable for human
interaction. Encoded hashes and checksums become very easy to verify at a glance,
and take up much less space on a terminal.

## Performance

base💯's performance is very competitive with other encoding algorithms.

### Scalar Performance

```
$ base100 --version
base💯 0.3.0

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

On a machine supporting AVX2, base💯 gains around 25% performance through a
decoder written in hand-tuned x86-64 assembly. Support for SSE2 will come soon,
and I will happily support AVX-512 if some compatible hardware finds its way into
my hands. Work on a vectorized encoder for AVX2 and SSE2 is also underway.

Please note that the below benchmarks were taken on a significantly weaker
machine than the above benchmarks, and cannot be directly compared.

```
$ base100 --version
base💯 0.3.0-dirty

$ base64 --version
base64 (GNU coreutils) 8.28

$ cat /dev/zero | pv | ./base100 | ./base100 -d > /dev/null
 [ 191MiB/s]

$ cat /dev/zero | pv | base64 | base64 -d > /dev/null
 [ 110MiB/s]
```

In this scenario, base💯 compares very favorably to GNU base64.

## Future plans

- Allow data to be encoded with the full 1024-element emoji set
- Add further optimizations and ensure we're abusing SIMD as much as possible
- Add multiprocessor support