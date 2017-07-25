# Base💯

Encode things into Emoji.

Base💯 can represent any byte with a unique emoji symbol, therefore it can
represent binary data with zero printable overhead (see caveats for more info).

## Usage

```
$ echo "the quick brown fox jumped over the lazy dog" | base100
⛷⛏⛅Ⓜ⛳⛸⛑⚾⛔Ⓜ⚽⛴⛱⛺⛰Ⓜ⛈⛱⛽Ⓜ⛓⛸⛪⛲⛅⛄Ⓜ⛱⛹⛅⛴Ⓜ⛷⛏⛅Ⓜ⛩⚱✅✂Ⓜ⛄⛱⛎↘
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
__printable__ characters as possible.

## Performance

```
$ cat /dev/zero | base100 | pv > /dev/null
778MiB 0:00:05 [ 178MiB/s]

$ cat /dev/zero | base64 | pv > /dev/null
4.09GiB 0:00:05 [ 850MiB/s]

$ cat /dev/zero | base100 | base100 -d | pv > /dev/null
156MiB 0:00:05 [31.7MiB/s]

$ cat /dev/zero | base64 | base64 -d | pv > /dev/null
 903MiB 0:00:05 [ 169MiB/s]
```

Encoding and decoding are fairly slow because of the algorithm's reliance on
external data structures. This could probably be remedied in the future.

## Future plans

- Allow data to be encoded with the full 1024-element emoji set
