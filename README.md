A basic Wake-on-LAN client implementation.

Run the CLI to broadcast a [magic
packet](https://en.wikipedia.org/wiki/Wake-on-LAN#Magic_packet) to wake up the
provided MAC Address.

# Build

```
$ cargo build --release
```

# Run

```
$ wakeonlan 01:23:45:67:89:AB
```
