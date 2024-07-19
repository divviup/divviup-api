## divviup - the Divvi Up command line tool

`divviup` is a command line (CLI) tool for doing all basic operations on both the Divvi Up API and Distributed Aggregation Protocol (DAP) API endpoints. It's only likely to work if the leader aggregator is [Janus](https://github.com/divviup/janus). See `divviup --help` for details on all of the commands.

To get the tool, either check out this project and build it yourself:

```sh
cargo run --package divviup-cli -- help
```

If you run into problems with the `aws-lc-rs` dependency, you can try building with `--no-default-features --features common,ring`.

Or you can download a binary for your OS and host architecture from our [releases](https://github.com/divviup/divviup-api/releases).

A [complete tutorial for the divviup tool](https://docs.divviup.org/command-line-tutorial/) is available.
